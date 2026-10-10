# 第 37 批代码轮第二轮正文（2026-10-08）

<!-- doc-lint:not-numbers B1 B2 B3 B4 C1 C2 C3 C4 -->

## 一、题面

第一轮判决 `research/prompts/m3-formal-code-b37-r1-main-verification.md` 采了攻方打中的 B1-甲、乙、丙与 B2、B3、B4 几处，主 agent 当场改（判决第三节）。这一轮攻第一轮的修补本身，改动全文 `research/prompts/_m3-formal-code-b37-r2-diff.md`（相对第一轮冻结副本）：

- `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 的内联单测 `every_criterion_has_a_positive_control_and_every_control_is_red`：改成正式跑用的五档（r1、r2、两份表都取的 r2、r3、r5）各自断言「跑出对照的判据集合 = 写死的登记集合」且读数全红，五档并起来罩 `Criterion::ALL`。
- `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs` 三处注释（行号与 25）。
- `crates/singlefs-checker/src/image.rs` 第 212 行补两种不适用。
- `crates/mutations.tsv` 本批四行（第 3383、3407 行整行换，第 3496、3497 行追加）。

条款同第一轮（E165（恢复重放有界穷举与变异杀伤界）的阳性对照条款，I-4.1（重放后树可遍历））：`research/prompts/e165-preregistration.md` 第 5.4 节与 V2、`research/prompts/e165-r2-prereg.md` 第 5.5 节（另看第 5.4 节 J7 判在哪几档）、`research/prompts/e165-r3-prereg.md` 第 5.6 节（拆行）、`research/prompts/e165-r5-prereg.md` 第 5.3、5.4 节；正式跑的档取 `research/scripts/replay.sh` 起这个装置的驱动。

## 二、实现今天的样子（主 agent 的观测，2026-10-08）

- 冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r2/tree/`（取自改后的主工作区，清单同目录 `crates-sha256.txt`）。树里别的线的改动不判。代码引文读这棵树，写 `crates/…` 相对路径与这棵树里的行号。
- 主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 跑过：改后单测 `1 passed`；nff 17 passed；clippy、fmt 过；四行变异 `prove-red.sh` 结果与「改前的并集单测在 `ALimit => !run_profile.judges_j8()` 下仍过」的现跑，见第一轮判决第四节。

方案按 `crates/` 今天的实现来谈。

## 三、格

### C1　写死的五份集合对不对（辩方、攻方都判）
每档的集合与三份登记逐行对：每条判据在哪几档判（登记原句），写死的集合与 `main` 判定行里真判的（按 `judges_*`、`compares_the_two_record_tables`、族）是不是逐档相同；r1 只取合法、对抗两族，r2–r5 取三族，与正式跑各档起的族对不对得上；有没有一档正式跑判了、集合里没写（或写了、正式跑不判）。

### C2　第一轮判决够不够得着（辩方）
B1-甲 的打中是否成立（改前单测确实放得过、正式跑确实判了没对照）；第一轮「采攻方」的取舍有没有同样打中别的写法；把第一档 r3 直接建表与 r3 两份表都取的那一档拿掉，会不会让哪一条判据失去唯一一份读数。

### C3　新单测还能放过什么（攻方）
「只加不删」一类（某档多跑一条对照）新单测判不判；五档之外正式跑还有没有别的档（`--table scan`、反盘次序、r4、标记奇偶另一种）该进来；读数「全红」这条断言在每档有没有被空集放过；挂钟有没有明显变长。

### C4　四行变异与注释（攻方、辩方）
四行锚点各恰好一次、点名的测试筛得到、在改后单测下红在断言上、在改前单测下（换掉的两行）的行为；第 3497 行只有 nff 那条杀得掉的说法成不成立；三处注释与代码对不对得上。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | C1、C2、C4 | 复核第一轮判决：它采的打中够不够得着、改法是不是同样碍着别的、替被改掉的写法（并集判、r3 直接建表那一档）辩护 |
| 云端攻方（Opus） | C1、C3、C4，重点 C1、C3 | 攻第一轮的修补本身：找改后单测放过的坏法，找写死集合与登记、正式跑对不上的一格；不重复第一轮攻过的「并集顶替」那一角 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 照那一判；一次抽样的「没打中」只算这一次。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第三轮，换攻击面。
- 腿提的新判法、新用例算零轮；条款没写、要用户定的，交用户。
- 重型测试不跑：被判的是 checker 档装置二进制的内联测试，腿一样都不跑，读码判，要跑的写成命令交主 agent；harness 档测试可以跑。

交付：腿的报告写 `research/prompts/m3-formal-code-b37-r2-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-code-b37-r2-<腿名>-model/`（有就放，带一份 `tools/rerun.sh`）；草稿放 `~/.local/share/singlefs/work/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。

### 小节清单：`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E165 恢复重放有界穷举与变异杀伤界 —— 第一段已跑（2026-10-01），第二段已跑（2026-10-01），第 1 次重跑已跑（2026-10-01），第 2 次重跑已跑（四段，2026-10-02），第 3 次重跑已跑（三段：第一段，2026-10-02；第二段、第三段，2026-10-03），第 4 次重跑已跑（两段：第一段、第二段，2026-10-03），第 5 次重跑已跑（够判，2026-10-04：第一段、第二段、第三段与第五段两步；第四段够判后未跑） | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E165 标题之下、第一个下级标题之前的正文：第 2-19 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 这一段做了什么 | 抄 | 默认全抄（脚本生成） |
| ### 结果整行抄自产物 | 抄 | 默认全抄（脚本生成） |
| ### 对抗一族（第一段；登记修订 9 是看过对抗一族的草稿读数之后写的） | 抄 | 默认全抄（脚本生成） |
| ### 第二段：变异杀伤界 | 抄 | 默认全抄（脚本生成） |
| ### 逐行判定 | 抄 | 默认全抄（脚本生成） |
| ### 钉绝对值的断言与阳性对照 | 抄 | 默认全抄（脚本生成） |
| ### 几何敏感性 | 抄 | 默认全抄（脚本生成） |
| ### `light` 档 | 抄 | 默认全抄（脚本生成） |
| ### 变异 | 抄 | 默认全抄（脚本生成） |
| ### 第 1 次重跑（取前缀搬进核之后） | 抄 | 默认全抄（脚本生成） |
| ### 第 2 次重跑（J6 照 Qn 判、投影外根字段随版本变、多盘、由 `scan_journal` 建表） | 抄 | 默认全抄（脚本生成） |
| ### 第 3 次重跑（择根与重放合判、改坏择根序的变体、J6Qn 按「同一实例里 journal 记录的 txg 随计数器不减」拆行） | 抄 | 默认全抄（脚本生成） |
| #### 第 3 次重跑第二段（Q1：择根与重放合判，2026-10-03） | 抄 | 随父节带到 |
| #### 第 3 次重跑第三段（Q2：改坏择根序的变体有没有判别力，2026-10-03） | 抄 | 随父节带到 |
| ### 第 4 次重跑（带读故障的择根与重放合判族、第 3 次没红的三个择根变体） | 抄 | 默认全抄（脚本生成） |
| #### 第 4 次重跑第二段（Q2：第 3 次没红的三个择根变体在新一族上红不红，2026-10-03） | 抄 | 随父节带到 |
| ### 第 5 次重跑（记录格式按根环那条线 7-2 改过之后；系统配置读错取 R2 之后的带故障一族，2026-10-04） | 抄 | 默认全抄（脚本生成） |
| #### 第 5 次重跑第一段（3-甲：旧档原样重跑，2026-10-04） | 抄 | 随父节带到 |
| #### 第 5 次重跑第二段（3-乙：变异杀伤界，2026-10-04） | 抄 | 随父节带到 |
| #### 第 5 次重跑第三段（3-丙：r5 档与 J8，2026-10-04） | 抄 | 随父节带到 |
| #### 第 5 次重跑第五段 5-甲（Q1（R2）：系统配置读错取 R2 之后带故障的合法历史丢不丢，2026-10-04） | 抄 | 随父节带到 |
| #### 第 5 次重跑第五段 5-乙（Q2（R2）：第 3 次没红的三个择根变体在 R2 那一族上红不红，2026-10-04） | 抄 | 随父节带到 |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 路径与结论登记 | 抄 | 默认全抄（脚本生成） |
| ### 它答不了的 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-10-04：第 5 次重跑（记录格式按根环那条线 7-2 改过之后；系统配置读错取 R2 之后的带故障一族） | 不抄 | 历史节的子节 |
| ### 2026-10-03：第 4 次重跑登记修订 4（T1 读法措辞，只改措辞） | 不抄 | 历史节的子节 |
| ### 2026-10-03：第 4 次重跑第二段（Q2：第 3 次没红的三个择根变体在新一族上红不红） | 不抄 | 历史节的子节 |
| ### 2026-10-03：第 4 次重跑第一段（Q1：带读故障的合法历史上择根与重放丢不丢） | 不抄 | 历史节的子节 |
| ### 2026-10-03：第 3 次重跑第三段（Q2：改坏择根序的变体） | 不抄 | 历史节的子节 |
| ### 2026-10-03：第 3 次重跑第二段（择根与重放合判一族 K、K9、KW、KA） | 不抄 | 历史节的子节 |
| ### 2026-10-02：第 3 次重跑第一段（对抗一族 J6Qn 按「同一实例里 journal 记录的 txg 随计数器不减」拆行） | 不抄 | 历史节的子节 |
| ### 2026-10-02：第 2 次重跑（J6 照 Qn 判、投影外根字段随版本变、多盘、由 `scan_journal` 建表） | 不抄 | 历史节的子节 |
| ### 2026-10-02：C 域合法崩溃状态数写明切法 | 不抄 | 历史节的子节 |
| ### 2026-10-01：第 1 次重跑（取前缀搬进核之后） | 不抄 | 历史节的子节 |
| ### 2026-10-01：读钉表的参数入库 | 不抄 | 历史节的子节 |
| ### 2026-10-01：对抗一族入库，第二段已跑 | 不抄 | 历史节的子节 |
| ### 2026-10-01：建页，第一段合法历史一族已跑 | 不抄 | 历史节的子节 |

### 小节清单：`.claude/kb/invariants.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 不变量清单 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （不变量清单 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 状态 | 抄 | 默认全抄（脚本生成） |
| ## I-1 块自描述 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-7 根环 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-8 journal —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-2 校验和 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-3 空间记账 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-4 事务与崩溃一致性 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-5 分配器 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## I-6 加密不变量 —— 类 | 抄 | 默认全抄（脚本生成） |
| ## 待补 | 抄 | 默认全抄（脚本生成） |
| ## I-9 inode 树结构 —— 类 | 抄 | 默认全抄（脚本生成） |

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1-19`（整段抄，未转述）**

```markdown
## E165 恢复重放有界穷举与变异杀伤界 —— 第一段已跑（2026-10-01），第二段已跑（2026-10-01），第 1 次重跑已跑（2026-10-01），第 2 次重跑已跑（四段，2026-10-02），第 3 次重跑已跑（三段：第一段，2026-10-02；第二段、第三段，2026-10-03），第 4 次重跑已跑（两段：第一段、第二段，2026-10-03），第 5 次重跑已跑（够判，2026-10-04：第一段、第二段、第三段与第五段两步；第四段够判后未跑）

<!-- doc-lint:not-numbers J8 PQ1 PQ2 PQ3 PQ4 PQ5 PQ6 PQ7 PQ8 PQ9 PQ10 PQ12 PQ13 PQ14 PQ15 PQ19 PQ20 PQ21 PQ23 PQ24 PQ25 PF2 PF4 PF9 PF10 PF13 PV2 PV3 PV5 PV8 PV9 PV10 PV11 PV13 PH1 PH4 PH6 TF6 TV8 M1U1 M1U3 M2U0 M2U1 M2U2 M2U3 U12 U13 KF9 KFW KJ1T SN0 SN1 RNSB RI RN T1 TP D0 D1 TQ1 TQ2 TQ3 TQ7 TQ8 TQ11 TQ15 TH10 TF3 TF11 TV3 TV6 TH8 TQ9 TQ10 M2 U3 U1 U2 Q1 Q2 Q3 Q4 SQ2 SQ5 SF3 SF5 J6Qn J7a J7b J7c SQ16 SV3 SV9 SF11 RH1 M MC M6 AC Z5 M1 M2 M3 K2 K3 K4 K5 K6 K7 K8 K9 K10 J0 J1 J2 J3 J4 J5 J6 H1 H2 H3 H4 H5 V1 V2 V3 V4 V5 V6 V7 V8 V9 F1 F2 F3 F4 F5 F6 F8 F9 F10 F11 F12 F13 F14 F15 F16 F17 F18 F19 Q6 Q9 Q10 Q12 Q13 Q14 Q15 L6 L9 C6 E1 E2 A6 R0 S3 Z1 Z2 Z3 Z4 IA1 IA2 R1 R2 R3 RH4 RV6 PD5 PD6 PD10 PD11 PD12 PD13 PD14 PD19 L0 L1 C0 C1 KF3 KJ0 KJ1 KJ2 KJ3 KF1 KF2 TV4 TV10 TH4 TF9 TQ12 TQ4 TQ5 TQ6 TQ13 TQ14 TF4 TF5 TF7 TF8 TF10 TV7 TV9 U0 U10 U11 U14 M1U0 M1U2 -->

跑前登记 `research/prompts/e165-preregistration.md`（第十二节修订 1–12）；岔路单 `research/prompts/m3-formal-r1-forks.md` 的 K3–K9。被测对象是 `crates/singlefs-core/src/recovery.rs` 今天的 `replay_journal`（2026-10-01 的 sha256 `f04814901d9521a1f72bf64aae530157c643fe47f3b0a37ac06695feb9418fdb`），一个字没改。装置 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`（入库装置：直接调 `replay_journal`、`JournalRecord::to_bytes` / `parse`、真 CRC、真反向链，读者是内存里的两块盘）。

「产物」到「变异」各节（含「结果整行抄自产物」「对抗一族（第一段；登记修订 9 是看过对抗一族的草稿读数之后写的）」「第二段：变异杀伤界」「逐行判定」）写的是取前缀搬进核之前那次跑的读数，被测对象是 sha256 `f04814901d9521a1f72bf64aae530157c643fe47f3b0a37ac06695feb9418fdb` 那份 `recovery.rs`；搬家之后的重跑在「第 1 次重跑（取前缀搬进核之后）」一节，两次读数不同的是第二段的表第 448、613、1004 行。

产物：

- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out`：`--bounds segment1 --family legal`，910 行 `E7RESULT`，49 753 566 态。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out`：`--bounds light --family legal`，876 行，6 923 358 态。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family-timing.out`：两次跑的 `E7TIMING` 耗时行与量的时候的 `uptime` 原样输出，不进逐字比对。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out`：`--bounds segment1 --family adversarial`，172 行，64 513 748 态（登记修订第 9 条收窄停机条款 H4 之后跑的）。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family-timing.out`：对抗一族两次跑（16 线程、7 线程）的耗时行与 `uptime`，不进逐字比对。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out`：第二段，26 个变体（基线 1、`crates/mutations.tsv` 里 21 种不同的替换即 23 行、自造 4 条）各一行或两行，32 行。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv`：第二段的红例账，151 行（每个红的变体在每个红了的域首红的 N 上一个反例状态），格式见它的表头。
- `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds-timing.out`：第二段驱动的计时、负载与单变体量的那一次，不进逐字比对。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:20-27`（整段抄，未转述）**

```markdown
### 这一段做了什么

- **合法历史一族**（登记 5.2 的 L、L 小环 6 与 9、U 验证开与关、C 大环与小环 6、X、E1、E2、R）：生成器照 D23（journal 的角色与格式） 已定项 4、已定项 14 与 D13（验证路线） 已定项 1 引的条款造合法历史，每次发布几条记录、提交标记怎么摆全扫，记录的任意落盘子集全扫；每个状态调一次 `replay_journal`，照条款文字手写的判据 J0–J4、J6 与实现锚点 IA1、IA2 各数各的违例；C 域另在全部 s ⊆ u 的状态对上数 J5a（施加集读法）、J5b（落到的版本读法），五个对集合各报各的，并与「理想恢复」（落到照条款写的最远一版 j\*）并排。
- **对抗一族**（A、A 小环 6、A 链）：第一次跑停在登记停机条款 H4（登记 5.2 的字母表有序号 0，`crates/singlefs-core/src/journal.rs:304`「序号从 1 起（已定项 4）：读到 0 当这条记录损坏」，`parse` 拒收；修订 7）。主 agent 写了修订 9：序号不是 0 的记录照 H4 原文断言解得出，序号 0 的记录断言 `parse` 拒收、没拒就停。**修订 9 是看过对抗一族的草稿读数之后写的**；装置照它改了断言之后正式跑、入库，新产物的 `E7RESULT` 行与草稿那一份逐字节相同。序号 0 的记录进不了表，S 的 (e)「序号 0」在建出来的表上走不到（修订 7）。
- **第二段**（登记 5.5、5.6，修订 10–12）：驱动在每个变体自己的副本里改 `crates/singlefs-core/src/recovery.rs`、冷编译装置、跑 `--bounds segment2`，逐 (域, N) 与基线比；界固定为 5.6 的上沿（修订 10 去掉了放宽规则），每个域的「首红的 N」取第一个与基线不同的 N。红了的域在首红的 N 上找一个反例状态，原实现与变体的二进制各单条重放一遍。
- 装置为第二段加了两个参数（修订 12 允许，判据一条没改）：`--replay-state <域>:<N>:<序号>` 只判枚举次序里那一个状态，`--list-states <域>:<N>` 逐态各打一行；两种都单线程、不跑自检与阳性对照。加了之后第一段合法历史一族整份重跑，`E7RESULT` 行与入库那份逐字节相同。
- 照登记 5.5 的次序：① 常量回比断言与装置自检 13 项；② 阳性对照（合法历史一族 35 个读数、对抗一族 10 个）；③–⑥ 各域。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:28-82`（整段抄，未转述）**

````markdown
### 结果整行抄自产物

全部抄自 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out`，行首是那份文件自己的行号（`grep -n 'name=verdict'` 取的）。各域合计的判决行：

```
173:E7RESULT name=verdict domain=L ring_slots=64 smallest_n=1 largest_n=10 states=24186470 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=6046106 applied_ge2=1509478 domain_is_nonempty=true
246:E7RESULT name=verdict domain=L6 ring_slots=6 smallest_n=1 largest_n=6 states=3654 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=354 j6_premise_fails_judged=1272 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=1362 applied_ge2=230 domain_is_nonempty=true
355:E7RESULT name=verdict domain=L9 ring_slots=9 smallest_n=1 largest_n=9 states=1171430 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=51998 j6_premise_fails_judged=557792 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=372042 applied_ge2=87166 domain_is_nonempty=true
428:E7RESULT name=verdict domain=U_verify_on ring_slots=64 smallest_n=1 largest_n=6 states=12004314 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=750246 applied_ge2=46842 domain_is_nonempty=true
489:E7RESULT name=verdict domain=U_verify_off ring_slots=64 smallest_n=1 largest_n=6 states=12004314 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=6726648 j3_violations=0 j4_violations=6726648 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=7086030 applied_ge2=3410250 domain_is_nonempty=true
604:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 states=149796 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=24987 applied_ge2=4197 domain_is_nonempty=true
709:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 states=9700 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=215 j6_premise_fails_judged=596 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=1967 applied_ge2=229 domain_is_nonempty=true
791:E7RESULT name=verdict domain=X ring_slots=64 smallest_n=2 largest_n=7 states=216484 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=40156 applied_ge2=6628 domain_is_nonempty=true
840:E7RESULT name=verdict domain=E1 ring_slots=64 smallest_n=1 largest_n=4 states=398 j0_violations=0 failed_states=0 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=70 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=70 applied_ge2=0 domain_is_nonempty=true
871:E7RESULT name=verdict domain=E2 ring_slots=64 smallest_n=1 largest_n=3 states=42 j0_violations=0 failed_states=42 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=0 applied_ge2=0 domain_is_nonempty=true
908:E7RESULT name=verdict domain=R ring_slots=64 smallest_n=2 largest_n=5 states=6964 j0_violations=0 failed_states=0 j1_violations=956 j2_violations=956 j3_violations=956 j4_violations=956 j6_premise_holds_violations=0 j6_premise_fails_violations=0 j6_premise_fails_judged=0 ia1_violations=0 ia2_violations=0 specification_self_check_mismatches=0 applied_ge1=1808 applied_ge2=492 domain_is_nonempty=true
909:E7RESULT name=verdict scope=run bounds=segment1 family=legal states=49753566 self_checks_passed=13 control_readings=35 controls_not_red=0 void_conditions=0 halt_conditions=0
```

C 域状态对（J5a、J5b；`today` 是今天的实现，`ideal` 是理想恢复；`cut_own` 是切法甲、根槽那次写自成一段，`cut_with` 是切法乙、根槽那次写与记录同段）：

```
605:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=wide pairs=7322544 today_j5a_violations=830685 today_j5b_violations=0 ideal_j5a_violations=830685 ideal_j5b_violations=0
606:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_own_legal pairs=71579 today_j5a_violations=4969 today_j5b_violations=0 ideal_j5a_violations=4969 ideal_j5b_violations=0
607:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_own_same_segment pairs=49644 today_j5a_violations=911 today_j5b_violations=0 ideal_j5a_violations=911 ideal_j5b_violations=0
608:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_own_same_segment_stopped_in_a_record_segment pairs=48369 today_j5a_violations=0 today_j5b_violations=0 ideal_j5a_violations=0 ideal_j5b_violations=0
609:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_own_same_segment_stopped_in_a_root_segment pairs=2733 today_j5a_violations=911 today_j5b_violations=0 ideal_j5a_violations=911 ideal_j5b_violations=0
610:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_with_legal pairs=190025 today_j5a_violations=8146 today_j5b_violations=0 ideal_j5a_violations=8146 ideal_j5b_violations=0
611:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_with_same_segment pairs=144560 today_j5a_violations=911 today_j5b_violations=0 ideal_j5a_violations=911 ideal_j5b_violations=0
612:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_with_same_segment_root_write_differs pairs=48369 today_j5a_violations=911 today_j5b_violations=0 ideal_j5a_violations=911 ideal_j5b_violations=0
613:E7RESULT name=verdict domain=C ring_slots=64 smallest_n=1 largest_n=6 set=cut_with_same_segment_root_write_agrees pairs=96191 today_j5a_violations=0 today_j5b_violations=0 ideal_j5a_violations=0 ideal_j5b_violations=0
710:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=wide pairs=319689 today_j5a_violations=41511 today_j5b_violations=0 ideal_j5a_violations=41511 ideal_j5b_violations=0
711:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_own_legal pairs=2020 today_j5a_violations=475 today_j5b_violations=0 ideal_j5a_violations=475 ideal_j5b_violations=0
712:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_own_same_segment pairs=829 today_j5a_violations=119 today_j5b_violations=0 ideal_j5a_violations=119 ideal_j5b_violations=0
713:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_own_same_segment_stopped_in_a_record_segment pairs=669 today_j5a_violations=0 today_j5b_violations=0 ideal_j5a_violations=0 ideal_j5b_violations=0
714:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_own_same_segment_stopped_in_a_root_segment pairs=357 today_j5a_violations=119 today_j5b_violations=0 ideal_j5a_violations=119 ideal_j5b_violations=0
715:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_with_legal pairs=4507 today_j5a_violations=701 today_j5b_violations=0 ideal_j5a_violations=701 ideal_j5b_violations=0
716:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_with_same_segment pairs=1929 today_j5a_violations=119 today_j5b_violations=0 ideal_j5a_violations=119 ideal_j5b_violations=0
717:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_with_same_segment_root_write_differs pairs=669 today_j5a_violations=119 today_j5b_violations=0 ideal_j5a_violations=119 ideal_j5b_violations=0
718:E7RESULT name=verdict domain=C6 ring_slots=6 smallest_n=1 largest_n=5 set=cut_with_same_segment_root_write_agrees pairs=1260 today_j5a_violations=0 today_j5b_violations=0 ideal_j5a_violations=0 ideal_j5b_violations=0
```

每个不为 0 的格里 N 最小的那一行与它的见证（`shape` 是每次发布几条记录与提交标记的摆法，`on_disk` 逐条记录在不在 0 号盘、点名单元哪一态，`first` / `second` 是第一、二次调用的结局）：

```
217:E7RESULT name=violations domain=L6 ring_slots=6 n=4 criterion=J6_premise_fails violations=10 judged=10 witness=shape=1c0+1c0+1c0+1c0;on_disk=-,good,good,good;residue_at=none;chosen=0;farthest=2;verify=true;first=returned:txg102:watermark1002:applied2;second=returned:txg103:watermark1003:applied1
302:E7RESULT name=violations domain=L9 ring_slots=9 n=5 criterion=J6_premise_fails violations=38 judged=38 witness=shape=1c0+1c0+1c0+1c0+1c0;on_disk=-,good,good,good,good;residue_at=none;chosen=0;farthest=3;verify=true;first=returned:txg103:watermark1003:applied3;second=returned:txg104:watermark1004:applied1
442:E7RESULT name=violations domain=U_verify_off ring_slots=64 n=2 criterion=J2 violations=24 judged=108 witness=shape=1c0+1c0;on_disk=-,d0_unreadable;residue_at=none;chosen=0;farthest=0;verify=false;first=returned:txg101:watermark1001:applied1
444:E7RESULT name=violations domain=U_verify_off ring_slots=64 n=2 criterion=J4 violations=24 judged=108 witness=shape=1c0+1c0;on_disk=-,d0_unreadable;residue_at=none;chosen=0;farthest=0;verify=false;first=returned:txg101:watermark1001:applied1
659:E7RESULT name=violations domain=C6 ring_slots=6 n=3 criterion=J6_premise_fails violations=5 judged=5 witness=shape=1c0+1c0+1c0+1c0;on_disk=good,good,good,good;residue_at=none;chosen=0;farthest=2;verify=true;first=returned:txg102:watermark1002:applied2;second=returned:txg103:watermark1003:applied1
810:E7RESULT name=violations domain=E1 ring_slots=64 n=2 criterion=J6_premise_holds violations=2 judged=12 witness=shape=1c0+1c0;on_disk=-,good;residue_at=none;chosen=0;farthest=1;verify=true;first=returned:txg18446744073709551615:watermark1001:applied1;second=failed:chosen_root_txg_has_no_successor
874:E7RESULT name=violations domain=R ring_slots=64 n=2 criterion=J1 violations=2 judged=4 witness=shape=1c0+1c0;on_disk=-,-;residue_at=1;chosen=0;farthest=0;verify=true;first=returned:txg101:watermark9000001:applied1
500:E7RESULT name=pairs domain=C ring_slots=64 n=1 set=wide pairs=9 today_j5a=1 today_j5b=0 ideal_j5a=1 ideal_j5b=0
```

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:83-105`（整段抄，未转述）**

````markdown
### 对抗一族（第一段；登记修订 9 是看过对抗一族的草稿读数之后写的）

抄自 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out`（`grep -n 'name=verdict'`）：

```
84:E7RESULT name=verdict domain=A ring_slots=64 smallest_n=1 largest_n=4 states=57585320 a0_violations=0 failed_states=12435792 as_whole_table_violations=330240 as_consecutive_gap_continues_violations=154624 as_consecutive_gap_stops_violations=488224 readings_differing_whole_table_vs_consecutive_gap_continues=175616 readings_differing_whole_table_vs_consecutive_gap_stops=509216 readings_differing_consecutive_gap_continues_vs_consecutive_gap_stops=333600 a_structure_violations=0 a_limit_violations=0 j6_premise_holds_violations=85400 j6_premise_fails_violations=0 j6_premise_fails_judged=0 applied_ge1=1473356 states_holding_a_record_rejected_by_parse=45815520 domain_is_nonempty=true
127:E7RESULT name=verdict domain=A6 ring_slots=6 smallest_n=1 largest_n=3 states=788838 a0_violations=0 failed_states=128560 as_whole_table_violations=2048 as_consecutive_gap_continues_violations=1024 as_consecutive_gap_stops_violations=3424 readings_differing_whole_table_vs_consecutive_gap_continues=1024 readings_differing_whole_table_vs_consecutive_gap_stops=3424 readings_differing_consecutive_gap_continues_vs_consecutive_gap_stops=2400 a_structure_violations=0 a_limit_violations=0 j6_premise_holds_violations=0 j6_premise_fails_violations=496 j6_premise_fails_judged=65536 applied_ge1=25058 states_holding_a_record_rejected_by_parse=548640 domain_is_nonempty=true
170:E7RESULT name=verdict domain=AC ring_slots=64 smallest_n=1 largest_n=3 states=6139590 a0_violations=0 failed_states=1009760 as_whole_table_violations=10240 as_consecutive_gap_continues_violations=4096 as_consecutive_gap_stops_violations=16640 readings_differing_whole_table_vs_consecutive_gap_continues=6144 readings_differing_whole_table_vs_consecutive_gap_stops=18688 readings_differing_consecutive_gap_continues_vs_consecutive_gap_stops=12544 a_structure_violations=0 a_limit_violations=0 j6_premise_holds_violations=2904 j6_premise_fails_violations=0 j6_premise_fails_judged=0 applied_ge1=133024 states_holding_a_record_rejected_by_parse=4295232 domain_is_nonempty=true
171:E7RESULT name=verdict scope=run bounds=segment1 family=adversarial states=64513748 self_checks_passed=13 control_readings=10 controls_not_red=0 void_conditions=0 halt_conditions=0
```

不为 0 的 AS 与 J6 格里 N 最小的在 n = 3（n ≤ 2 全 0）；A 域 n = 3 的见证：

```
59:E7RESULT name=violations domain=A ring_slots=64 n=3 criterion=AS_consecutive_gap_continues violations=1024 judged=778034 witness=anchor=absent;slots=t+1:o1:L0:C0:x1:chain_true,t+0:o1:L1:C0:x1:chain_true,t+1:o1:L1:C1:x1:chain_true;first=returned:txg100:watermark1000:applied0;expected=txg101:watermark9003038/txg101:watermark9003038/txg101:watermark9003038
63:E7RESULT name=violations domain=A ring_slots=64 n=3 criterion=J6_premise_holds violations=464 judged=650650 witness=anchor=absent;slots=t+1:o1:L1:C1:x1:chain_true,t+2:o1:L0:C0:x1:chain_true,t+1:o1:L1:C0:x1:chain_true;first=returned:txg101:watermark9001038:applied1;expected=txg100:watermark1000/txg101:watermark9001038/txg101:watermark9001038;second=failed:more_than_one_last_record_flag
```

- 三种读法两两分得开（三个 A-互 数在三个域都不为 0），AS 三条在同一个域都不为 0。第 59 行的见证：槽 2 与所选根同 txg、带末条标志，S 认它当锚点、施加槽 3；实现的 `above` 按计数器排，第一条是槽 1（txg t + 1），不等于期望的计数器就停，一条不施加——水位之上的记录计数器比锚点小（同实例里 txg 随计数器回退）才走得到。
- A-结构、A-限 在三个域都是 0：带上前提之后，实现返回的根要么是所选根、要么取自一条同实例、txg 更大、带提交标记与末条标志的记录，报告的施加条数不超过在飞上限。
- J6 前提真的违例（A 464 起，见证第 63 行）：第一次落到的那一版，表里同 txg 带末条标志的有两条，第二次调用报 `more_than_one_last_record_flag`。
- 被 `parse` 拒收（序号 0）的记录至少一条的态数照报（`states_holding_a_record_rejected_by_parse`），A 域 57 585 320 态里 45 815 520 态。

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:106-159`（整段抄，未转述）**

````markdown
### 第二段：变异杀伤界

收尾三行抄自 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out`：

```
E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows=863 repository_recovery_sha256_at_start=f04814901d9521a1f72bf64aae530157c643fe47f3b0a37ac06695feb9418fdb repository_recovery_sha256_at_end=f04814901d9521a1f72bf64aae530157c643fe47f3b0a37ac06695feb9418fdb crates_git_status_unchanged=true copies_restored=true
E7RESULT name=verdict scope=segment2 variants=26 variants_excluding_baseline=25 red=23 red_only_by_implementation_anchor=1 not_red_within_bounds=1 invalid=0 run_did_not_finish=0 ledger_sha256=155cdb1a0882432e943d5f18b9d425b7265f7e22884d8741c900151f292b5724
E7RESULT name=done emitted=32
```

基线那一行记 `rows_found_verbatim_in_segment1_products`：基线每个 (域, N) 的行都逐字出现在第一段的两份产物里（作废条款 V7 没触发）。逐变体的结局与两族各自的首红最小值，命令与原样输出（行数按表行数，第 23 与 355 行、第 406 与 414 行是同一个变体、各一行）：

```
$ grep 'name=kill_bound' research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out | grep -v item=baseline | awk '{o="";for(i=1;i<=NF;i++){if($i~/^(item|table_row|outcome|legal_family_minimum|adversarial_family_minimum)=/)o=o" "$i};print substr(o,2)}'
item=tsv-23 table_row=23 outcome=red legal_family_minimum=2 adversarial_family_minimum=none
item=tsv-23 table_row=355 outcome=red legal_family_minimum=2 adversarial_family_minimum=none
item=tsv-24 table_row=24 outcome=red legal_family_minimum=1 adversarial_family_minimum=1
item=tsv-36 table_row=36 outcome=red legal_family_minimum=3 adversarial_family_minimum=1
item=tsv-226 table_row=226 outcome=red_only_by_implementation_anchor legal_family_minimum=2 adversarial_family_minimum=none
item=tsv-354 table_row=354 outcome=red legal_family_minimum=2 adversarial_family_minimum=none
item=tsv-406 table_row=406 outcome=red legal_family_minimum=2 adversarial_family_minimum=1
item=tsv-406 table_row=414 outcome=red legal_family_minimum=2 adversarial_family_minimum=1
item=tsv-448 table_row=448 outcome=not_red_within_bounds legal_family_minimum=none adversarial_family_minimum=none
item=tsv-512 table_row=512 outcome=red legal_family_minimum=3 adversarial_family_minimum=1
item=tsv-513 table_row=513 outcome=red legal_family_minimum=none adversarial_family_minimum=1
item=tsv-514 table_row=514 outcome=red legal_family_minimum=none adversarial_family_minimum=2
item=tsv-515 table_row=515 outcome=red legal_family_minimum=none adversarial_family_minimum=2
item=tsv-516 table_row=516 outcome=red legal_family_minimum=none adversarial_family_minimum=1
item=tsv-517 table_row=517 outcome=red legal_family_minimum=2 adversarial_family_minimum=2
item=tsv-518 table_row=518 outcome=red legal_family_minimum=2 adversarial_family_minimum=1
item=tsv-613 table_row=613 outcome=red legal_family_minimum=none adversarial_family_minimum=1
item=tsv-614 table_row=614 outcome=red legal_family_minimum=none adversarial_family_minimum=2
item=tsv-920 table_row=920 outcome=red legal_family_minimum=3 adversarial_family_minimum=1
item=tsv-921 table_row=921 outcome=red legal_family_minimum=2 adversarial_family_minimum=1
item=tsv-922 table_row=922 outcome=red legal_family_minimum=1 adversarial_family_minimum=1
item=tsv-1003 table_row=1003 outcome=red legal_family_minimum=1 adversarial_family_minimum=none
item=tsv-1004 table_row=1004 outcome=red legal_family_minimum=1 adversarial_family_minimum=1
item=Z1 table_row=3 outcome=red legal_family_minimum=1 adversarial_family_minimum=1
item=Z2 table_row=4 outcome=red legal_family_minimum=3 adversarial_family_minimum=3
item=Z3 table_row=5 outcome=red legal_family_minimum=2 adversarial_family_minimum=none
item=Z4 table_row=6 outcome=red legal_family_minimum=3 adversarial_family_minimum=3
```

界内没被条款判据抓到的两条，整行抄（第 7、11 行）：

```
7:E7RESULT name=kill_bound item=tsv-226 table=crates/mutations.tsv table_row=226 rows_sharing_this_replacement=226 outcome=red_only_by_implementation_anchor first_clause_red=none first_anchor_red=U_verify_off:2:IA2 other_difference_before_any_red=none not_red_domains=L(≤8),L6(≤6),U_verify_on(≤5),C(≤5),C6(≤5),A(≤3),A6(≤3),AC(≤3),X(≤6),E1(≤4),E2(≤3),R(≤5) legal_family_minimum=2 adversarial_family_minimum=none run_exit=0 run_finished=true domains_without_rows=none abort=none red_examples=1 red_examples_confirmed_by_single_state_replay=1 copy_restored=true rerun="bash research/scripts/e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out --ledger research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv --recheck --item tsv-226" title=只供测试的开关：关掉点名验证时_verification_passed_照加（两臂在健康镜像上报同一个数，运行时看不出走了哪一条） complete=yes
11:E7RESULT name=kill_bound item=tsv-448 table=crates/mutations.tsv table_row=448 rows_sharing_this_replacement=448 outcome=not_red_within_bounds first_clause_red=none first_anchor_red=none other_difference_before_any_red=none not_red_domains=L(≤8),L6(≤6),U_verify_on(≤5),U_verify_off(≤5),C(≤5),C6(≤5),A(≤3),A6(≤3),AC(≤3),X(≤6),E1(≤4),E2(≤3),R(≤5) legal_family_minimum=none adversarial_family_minimum=none run_exit=0 run_finished=true domains_without_rows=none abort=none red_examples=0 red_examples_confirmed_by_single_state_replay=0 copy_restored=true rerun="bash research/scripts/e165-replay-mutation-kill-bounds.sh --scratch <草稿目录> --output research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out --ledger research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv --recheck --item tsv-448" title=P6_后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮_K4-b） complete=yes
```

- 「红」照登记 5.5 的口径是「与基线不同」，不分方向。红例账第 7 列带方向（`+` 是变体多加的违例、`-` 是少加的），只是首红那个状态上的，不是整域合计。几处「红」是变体比基线少违例：R 域的 J1–J4（今天的实现在残留态上落到时间线之外，变体不施加或少施加就不落了；tsv-24、512、920、921、922、1004、Z1 的反例状态都是 `-J1`），E1 的 J6 前提真（tsv-24、512、517、921、922、Z1 是 `-J6_premise_holds`），Z2、Z4 小环上的 J6 前提假（`-J6_premise_fails`）。tsv-406（同 414）在 E1、N = 3 上是 `+J6_premise_holds`。
- 红例账 151 行，原实现与变体的二进制各单条重放，判定与逐态行相同的 151 行：`awk -F'\t' '!/^#/{t++; if($8=="true"&&$9=="true")c++} END{print t,c}'` 输出 `151 151`。
- 登记第九节甲表「推的最先改变输出的取样点」逐行对过：每一行在它推的那个 (域, N) 上都红了、判据名也对得上；另外多数变体在别的域红得更早（例 tsv-406 在 C 域 Nc = 2），tsv-1004 在 E1、N = 1 上先被「合法历史一族报错态数」这条实现锚点抓到。推 7（448 可能每个域都不红）、推 8（226 只有 IA2）量到了。

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:160-182`（整段抄，未转述）**

```markdown
### 逐行判定

| 登记的量 | 读数（合法历史一族） | 照登记第六、十节记成什么 |
|---|---|---|
| Q1 原实现 J1（返回的根） | L、L6、L9、U 验证开、X、E1、C、C6 全 0 | F1 没触发：界内原实现在合法历史域里过 M2 的「返回的根」取法 |
| Q2 原实现 J2（根加盘） | 同一批域全 0 | 原实现那一半 J1 与 J2 逐域同为 0：界内两种取法分不开（F2 的前一半；后一半要第二段的变异） |
| Q3 J3（在飞上限） | 大环、6 槽、9 槽全 0 | 原实现过 I-8.3（重放前缀严格连续） 的上限那一句 |
| Q4 J4（最远一版） | L、L6、L9、U 验证开、X、E1、C、C6 全 0 | F6 没触发：「加」那一题面对今天的实现在界内为真 |
| Q5 J0 与报错态数 | J0 全 0；报错态数只有 E2 的 42（42 态全报错，就是 IA1） | 合法历史一族（L、L 小环、U 验证开、X、E1）报错态数 0，与登记第七节甲类锚点相符，F11 没触发 |
| Q6 J5a、J5b | J5b 在五个对集合、两个环上全 0；J5a 在「宽」「甲合法」「甲同段」「乙合法」「乙同段」都不为 0（大环 830 685 / 4 969 / 911 / 8 146 / 911）；甲同段的 911 全在「停在根槽段」那一行，「停在记录段」是 0；理想恢复与原实现逐格同值 | 施加集读法的违例是读法的性质、不是实现的：任何落到最远一版的恢复都有。哪个集合算「段内」由主 agent 判；不管判哪一个，J5a 都不为 0，J5b 都为 0 |
| Q7 J6（不动点） | 前提真：L、L6、L9、U、C、C6、X 全 0，E1 是 70（第二次调用在 txg 顶格的根上报错）；前提假：大环上一个态都没有（上限咬不到），L6 354 / 1 272，L9 51 998 / 557 792，C6 215 / 596 | F8 触发在 E1：不动点读法带前提也为假，见证是第一次落到 txg = u64 最大值的那一版、第二次报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`。F9 没触发：小环上前提假的态里 J6 不为 0，前提承重。随几何翻面：前提假的格大环没有态、小环不为 0，两点都写 |
| Q8 关掉点名验证 | U 验证关：J1 是 0，J2 是 6 726 648，J4 是 6 726 648，IA2 是 0 | F12 按「根加盘」取法触发、按「返回的根」取法不触发：关掉点名验证那一种策略自己过 J1、不过 J2 |
| Q11 R 域 | J1 是 956（6 964 态里），见证 N = 2 | F17 没触发：不带「同一 (实例代号, 计数器) 只对应一份内容」这个前提时 M2 字面为假，前提承重 |
| Q12 域非空 | 每个域 `domain_is_nonempty=true`（E2 除外照登记不要求）；C 域所选根不是 R_0 的态数大环 N = 6 是 115 520 | V4 没触发 |
| Q9 A-互、AS 三条 | 三个 A-互 数在 A、A6、AC 都不为 0（A：175 616 / 509 216 / 333 600）；AS 三条在三个域都不为 0（A：330 240 / 154 624 / 488 224） | F13 没触发：三种读法界内分得开。F14 触发：今天的实现与三种读法都对不上（同一个域里三条都不为 0） |
| Q10 A-结构、A-限 | 三个域全 0 | F16 没触发：对抗域带前提之后原实现过结构与上限两条 |
| Q13 变异表的杀伤界（25 个变体） | 条款判据红 23；只被实现锚点抓到 1（表第 226 行，U 验证关 N = 2 上 IA2）；界内不红 1（表第 448 行，每个域到上沿都与基线逐行相同）；无效 0 | F18 触发：第 448 行界内不红；第 226 行另列「条款判据界内不红」。F19 不适用（修订 10 去掉了放宽）。F15 触发：第 613、614、514、448 行在合法历史一族每个域都界内不红；其中 613（A、n = 1）、614、514（A、n = 2）在对抗一族红，448 两族都不红 |
| Q13 看 K3 的四行 | 第 518、406、414、36 行：J2 在 L、L6、U、C、X、E1 上红（N = 2–4；第 36 行在 E1 上到上沿都不红），同一个 N 上 J1 不红；J1 只在 R 域红（518、406、414、36 都是 N = 3） | F2 没触发：J1 与 J2 的首红 N 在合法历史域上不等，界内分得开。F3 没触发：这几行在 R 域的 J1 上红。注意：每个域红了就不往上扫，这些域里 J1 在更大的 N 上红不红没量 |
| Q13 看 K5 的 Z1 | Z1 在 L、L6、U 验证开、X（N = 2–3）只有 J4 与基线不同；在 R（N = 2）J1、J2、J3 不同，在 U 验证关（N = 2）J2 不同，在 C、C6（Nc = 1）J5a 不同，在 E1（N = 2）J6 前提真不同，在 A、A6、AC（n = 1）AS 三条不同 | 照登记「与基线不同」的字面，F5 触发（J4 之外还在 J1、J2、J3、J5a、J6 上红）。这些差别在红例账的反例状态上都是变体少了违例（R 的 `-J1`、E1 的 `-J6_premise_holds`），C 域 J5a 的方向装置没报，读 `judge_a_legal_state` 推的是变体永远落在所选根、施加集为空、J5a 违例变 0（没量过）；按「多出违例」读，合法历史一族里只有 J4 红，那是 F4 的形态。取哪种读法由主 agent 定 |
| Q14 J6 分不分辨变异表 | 表第 406、414 行在 E1、N = 3 上多出 J6 前提真的违例（反例状态 `+J6_premise_holds`）；第 24、512、517、921、922 行与 Z1 在 E1 的反例状态上少了 J6 违例；Z2、Z4 在小环的反例状态上少了 J6 前提假的违例（都只看首红那一个状态，不是整域合计） | F10 没触发：有变异在 J6 上红，其中 406、414 是多出违例 |

推 1–推 11 里第一段量得到的：推 1（施加集读法在「记录全落、根槽落没落」那一对上为假）量到了（C 域 Nc = 1「宽」集合 9 对里 1 对）；推 2（小环上前提假时不动点为假）量到了；推 3（关掉验证过「返回的根」、不过「根加盘」）量到了；推 9（残留域里不过「返回的根」）量到了。登记没推到的一条：E1 上前提真的 J6 违例 70。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:183-189`（整段抄，未转述）**

```markdown
### 钉绝对值的断言与阳性对照

- 登记第七节乙类锚点（各域每个 N 的状态数、L 域 N ≤ 7 的见证数、C 域的对数与合法崩溃状态数，含第十二节补的小环两组）在装置里逐格 `assert_eq!`，一格不等进程当场退出；两份产物都跑到了 `name=done`。
- S 自己的对照（登记 5.4、作废条款 V3）：L、L6、L9、U 验证开、X、E1 里 S 三种读法的期望落点与 j\* 不等的态数，123 行 `name=specification_self_check` 全是 `mismatches=0`。
- 阳性对照 35 个读数全红（`controls_not_red=0`）：J0 是 1、J1 是 762、J4 是 762（都等于对照界里 j\* ≥ 1 的态数，登记第七节见证数 N = 1..5 之和）、J2 是 1 032 而 J1 是 0、J3 小环 354 而大环 0、J5a 是 225、J5b 是 35、J5a 与 J5b 在九个对集合行上逐行相等、J6 是 3 110、IA1 是 42、IA2 是 2 058。
- C 域合法崩溃状态数写明切法（产物 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out` 的 `name=legal_crash_states` 行，N 逐行相加）：切法甲（`cut_own`，根槽那次写自成一段，D16（发布语义） 已定项 7 的屏障位置）C 域 8 006、C6 域 383，共 8 389；切法乙（`cut_with`，根槽那次写与记录同段）C 域 14 737、C6 域 606，共 15 343。三方判决里引过的 15 343 是切法乙的数；引 C 域合法态数要写明切法（三方判决 `research/prompts/m3-formal-r3-main-verification.md` 第五节第一条）。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:190-201`（整段抄，未转述）**

```markdown
### 几何敏感性

| 旋钮 | 取样点 | 判定翻不翻 |
|---|---|---|
| 环槽数（在飞上限） | 64 槽（上限 21）、9 槽（3）、6 槽（2） | J1–J4 三个点都是 0，不翻；J6 前提假那一格大环没有态、小环不为 0，翻；J5a、J5b 大环与 6 槽小环同判定 |
| 历史的起始计数器 | 大环 7（不回绕）、小环等于环槽数（历史中间回绕） | 随环一起，J1–J4 不翻 |
| 所选根的 txg | 100、u64 最大值 − 1、u64 最大值 | J6 前提真那一格在 u64 最大值 − 1 上翻成不为 0 |

在飞上限的分布：大环上表里所选根之上的记录条数最大 9（L 域 N = 10），大于上限的态数 0——上限在大环上咬不到，不是「不动点在大环上成立」的证据。J6 为假的态上第三遍与第二遍相不相等（只报不判）：L6 的 N = 6 是 210 相等、62 不等，L9 的 N = 9 是 33 904 相等、7 600 不等。

没扫的旋钮：盘数（两块，记录只在 0 号盘）、记录字节数与 F、实例代号的取值、一条记录点名几个单元。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:202-205`（整段抄，未转述）**

```markdown
### `light` 档

用户 2026-10-01 定这个装置当轻量工具。`--bounds light` 的界：L 到 9、U 到 5、A 到 3，其余各域同第一段。取法：草稿试跑量到 10 线程上每态挂钟 0.24–1.2 微秒，每个域取到 3 秒之内。入库的是 `--family legal` 那一份（6 923 358 态，进程内单调钟 2.671 秒）；状态数与见证数钉的是与第一段同一张常量表。量的时候同机有别的会话在跑 31 线程的广谱变异与求解器，开跑之前 `uptime` 报的三个负载均值是 10.19、29.98、34.04（原样输出在计时产物的头两行）；第一段 `--family legal` 在同一负载下 29.061 秒。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:206-211`（整段抄，未转述）**

```markdown
### 变异

单测 19 条（`grep -c '#\[test\]' crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`；第二段加的两条管单状态重放的参数与逐态行的写法，读钉表那一批加的三条管 `--pinned-cases`：一行要八段、域与规模在范围里，从钉住的状态行重算的输入指纹与表里那一份相同，算出来不同就报漂移）；装置变异 20 条在 `crates/mutations.tsv` 里，名字以「E165 」开头（`grep -c '^E165 ' crates/mutations.tsv`）。单测与变异都没跑：checker 档包的测试归提交时的门禁 checker-tier-crates-mutation-replay。装置每次开跑先做 13 项自检，其中每一项与一条单测调同一个函数，三份第一段产物里 13 行 `name=self_check` 都是 `passed=true`。

第二段量的变异表是登记第九节甲表（`crates/mutations.tsv` 里动 `replay_journal` 那五个函数的 23 行，加自造表 `research/mutations/e165_replay_bounded_enumeration_custom.tsv` 的 Z1–Z4），驱动开工照登记的取法重选一遍、行号与登记相同才跑。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:212-284`（整段抄，未转述）**

````markdown
### 第 1 次重跑（取前缀搬进核之后）

问题单 `research/prompts/e165-r1-questions.md`（R1–R3），重跑登记 `research/prompts/e165-r1-prereg.md`（含修订 1、修订 2）。为什么重跑：取前缀的判定从 `crates/singlefs-core/src/recovery.rs` 搬进 `crates/singlefs-replay-prefix/src/lib.rs`，胶水留在 `recovery.rs` 的 `replay_journal`（里程碑三形式化接入第 3–5 批）。结论对代码轮 m3-verus-land-code-r3 的冻结副本那棵树成立（文件清单 `research/prompts/m3-verus-land-code-r3-snapshot/crates-sha256.txt`，217 份；`recovery.rs` 的 sha256 `40067ae38fc76d7945ac61ee088682d6ce5ef578955c80f3268316e7597dd504`、`lib.rs` 的 `5f29497341979dba2aad7befe067f117d9b207392da64d7e76651d3a63514190`，见 `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-kill-bounds.out` 第 30 行）；主工作区今天的 `crates/` 已不等于那份清单，重跑没有在主工作区的树上做。

**R1 相同。** 第一段在冻结副本的拷贝上重跑（产物前缀 `r1-segment1b-`）：`--bounds segment1`，合法历史一族与对抗一族各跑 10 线程与 5 线程两遍，四份与入库那份逐字节相同：合法历史一族 910 行、sha256 `eb54d5032d53d93b74bd0febdb60d1124c0bfd7f3d2750cf084f856dacd5b3c5`，对抗一族 172 行、sha256 `3ca19987eed1d99dfe2987e1c1f7a336fc98764f9fb936a4f00d7c919ee3da6e`。阳性对照（取文件中间一行、把 `violations=` 后的数加一）两族各报「不同」，第一处不同恰是改的那一行（合法历史一族第 455 行、对抗一族第 86 行）；原件比原件各报「相同」。段首段尾在拷贝根上 `sha256sum -c` 都是 217 OK、0 非 OK，`crates/` 下清单外文件 0。比对在 `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-comparison.out`；它第 3、4 行「非 OK」的数是脚本转义写错（取成了全部行数），文件末尾追加了更正。

第一段的第一遍（产物前缀 `r1-segment1-`）作废：段中别的会话改了主工作区 `crates/` 里三份文件（`crates/mutations.tsv`、`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`、`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`），段尾 `sha256sum -c` 清单是 214 OK、3 FAILED，按登记作废条款 RV6 作废。那一遍的比对本身也是相同（两族 sha256 与第二遍相同），但不认；九份产物原样留着。

**R2 不是同一批。** `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-row-selection.out` 第 153 行（行首是产物自己的行号）：

```
153:E7RESULT name=verdict scope=row_selection registered_names=23 selected=23 not_selected=0 not_selected_rows=none custom_rows=4 custom_ok=4 today_equals_registered=18 today_differs_from_registered=5 differing_rows=406+414+448+613+1004 differing_rows_with_extra_tags=406+414+448+613 differing_rows_with_missing_tags=1004 registered_version_by_the_same_method_differs_from_registered=2 rh4_rows=406+414 registered_version_equals_today_by_the_same_method=20 additional_rows_in_the_function_ranges=13
```

23 个名字全选到（`selected=23 not_selected=0`），Z1–Z4 的新锚点各命中一次（`custom_rows=4 custom_ok=4`）。照登记 9.1 写死的集合判，5 行不等（`differing_rows=406+414+448+613+1004`）；照同一判法在两版（登记时那一版与今天）上各定一遍集合再比，3 行不等：448、613、1004（`registered_version_equals_today_by_the_same_method=20`）。406、414 是停机条款 RH4：9.1 写死的是 PD19，同一判法在登记时那一版上定出的是 PD12、PD13、PD14、PD19（`registered_version_by_the_same_method_differs_from_registered=2`）；按修订 2 两种判法并列报，9.1 不改。

| 表行 | 9.1 写死的集合 | 今天的集合 | 照 9.1 判 | 两版同一判法 |
|---|---|---|---|---|
| 406、414 | PD19 | PD12、PD13、PD14、PD19 | 不等 | 相等（RH4） |
| 448 | 空 | PD10 | 不等 | 不等 |
| 613 | PD11 | PD10、PD11 | 不等 | 不等 |
| 1004 | PD5、PD6 | PD5 | 不等 | 不等 |

其余 18 行两种判法都相等（`today_equals_registered=18`）。每个变体的标签与见证输入在 `research/mutations/e165_replay_bounded_enumeration_tags.tsv`；`additional_rows_in_the_function_ranges=13` 是登记 9.2 的函数区间里、不在 23 个名字之中的表行，只列不判。

**R3 变了。** `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-comparison.out` 第 34 行（`E7RESULT name=r3_summary`）报 `compared=27 expected=27 differing=3`、`verdict_same=false`，`verdict_old="variants=26 red=23 red_only_by_implementation_anchor=1 not_red_within_bounds=1 invalid=0 run_did_not_finish=0"`、`verdict_new="variants=26 red=24 red_only_by_implementation_anchor=1 not_red_within_bounds=0 invalid=0 run_did_not_finish=0"`。27 个变体比满，3 个不同，24 个逐字段相同（结局、两族首红每个域的 N、`not_red_domains`），含 Z1–Z4 与表第 36、406、414、518 行。不同的三行（比对产物第 15、23、29 行）正是 R2 里标签集合不等的三行，没有「标签集合相等而杀伤界变了」的行：

| 变体 | 入库那份（搬家前） | 重跑 |
|---|---|---|
| 448 | `not_red_within_bounds`：每个域到上沿都与基线逐行相同 | `red`：A、A6、AC 在 N = 1，C、C6、E1、L、L6、R、U_verify_off、U_verify_on 在 N = 3，X 在 N = 4 起红，只剩 E2（≤3）不红；两族最小值 `none` 变成合法历史一族 3、对抗一族 1 |
| 613 | `red` | `red`：多红 C、C6、E1、L、L6、R、U_verify_off、U_verify_on（N = 3）与 X（N = 4），`not_red_domains` 缩到只剩 E2（≤3）；合法历史一族最小值 `none` 变成 3 |
| 1004 | `red`：A、A6、AC（N = 1）、C、C6、L、L6、R、U_verify_off、U_verify_on（N = 2）、X（N = 3）红 | 这十一个域都不红，E1 仍红；对抗一族最小值 1 变成 `none` |

照 9.1 判，406、414 的判定换了，杀伤界逐字段相同；照同一判法判，它们的判定没换。

第二段的判决行（`research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-kill-bounds.out` 第 31 行）：

```
31:E7RESULT name=verdict scope=segment2 variants=26 variants_excluding_baseline=25 red=24 red_only_by_implementation_anchor=1 not_red_within_bounds=0 invalid=0 run_did_not_finish=0 ledger_sha256=0f96b2d585cdbaa9dc2befcfaabb63999b5753f08661b04f8306a9bf6a63e5b1
```

红例账 161 行，原实现与变体的二进制各单条重放，判定与逐态行相同的是 161 行：`awk -F'\t' '!/^#/{t++; if($8=="true"&&$9=="true")c++} END{print t,c}' research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-red-examples.tsv` 输出 `161 161`。

没判的：结论只对冻结副本那棵树成立，没在主工作区今天的树上跑；406、414 的 PD 集合哪一个对（9.1 写死的还是同一判法定出的）没判；PD 标签是登记修订 1、修订 2 的约定，换约定时标签表与 9.1 要一起重对。

新产物 23 份（`ls research/results/e165-replay-bounded-enumeration-2026-10-01-r1-*` 现数）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1-adversarial-family.out` | 172 | 第一遍（已作废）：`--bounds segment1 --family adversarial`，10 线程 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1-adversarial-family-threads-second.out` | 172 | 第一遍：对抗一族 5 线程那一遍 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1-adversarial-family-threads-second-timing.out` | 16 | 第一遍：上一份的耗时行 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1-adversarial-family-timing.out` | 16 | 第一遍：上一份的耗时行 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-adversarial-family.out` | 172 | 第二遍：`--bounds segment1 --family adversarial`，10 线程 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-adversarial-family-threads-second.out` | 172 | 第二遍：对抗一族 5 线程那一遍 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-adversarial-family-threads-second-timing.out` | 12 | 第二遍：上一份的耗时行 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-adversarial-family-timing.out` | 12 | 第二遍：上一份的耗时行 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-comparison.out` | 66 | 第二遍：逐份 sha256、`cmp` 结果、阳性对照、段首段尾 `crates/` 清单核（217 OK、0 非 OK）与对 3、4 行的更正 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-legal-family.out` | 910 | 第二遍：`--bounds segment1 --family legal`，10 线程 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-legal-family-threads-second.out` | 910 | 第二遍：同一族 5 线程那一遍 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-legal-family-threads-second-timing.out` | 67 | 第二遍：上一份的耗时行 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1b-legal-family-timing.out` | 67 | 第二遍：上一份的 `E7TIMING` 耗时行（不进逐字比对） |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1-comparison.out` | 69 | 第一遍：逐份 sha256、`cmp` 结果、阳性对照、段尾 `crates/` 清单核（3 行 FAILED） |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1-legal-family.out` | 910 | 第一遍（已作废）：`--bounds segment1 --family legal`，10 线程 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1-legal-family-threads-second.out` | 910 | 第一遍：同一族 5 线程那一遍 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1-legal-family-threads-second-timing.out` | 71 | 第一遍：上一份的耗时行 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment1-legal-family-timing.out` | 71 | 第一遍：上一份的 `E7TIMING` 耗时行（不进逐字比对） |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-comparison.out` | 34 | R3 逐变体比对（按变异名认，比结局、两族首红每个域的 N、`not_red_domains`） |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-kill-bounds.out` | 32 | 第二段：26 个变体（基线 1、表里 23 行、自造 4 条）的杀伤界 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-kill-bounds-timing.out` | 36 | 第二段驱动的计时与负载，不进逐字比对 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-red-examples.tsv` | 162 | 第二段红例账，每个红的变体在每个红了的域首红的 N 上一个反例状态 |
| `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-row-selection.out` | 154 | R2 选行：驱动按变异名选行与标签集合的逐行读数 |

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:285-356`（整段抄，未转述）**

````markdown
### 第 2 次重跑（J6 照 Qn 判、投影外根字段随版本变、多盘、由 `scan_journal` 建表）

问题单 `research/prompts/e165-r2-questions.md`（Q1–Q4），重跑登记 `research/prompts/e165-r2-prereg.md`（含修订 1–4）。状态：四段都已跑。第二段补跑（Q1 后半，2c）先停在停机条款 RH1（驱动钉的 `crates/mutations.tsv` sha256 是 `ca399ef1…`，带第三、四段装置变异行的表对不上；驱动 stderr 原样：「✗ 停机条款 RH1：crates/mutations.tsv 的 sha256 不等于登记的 ca399ef16376a35f8d0bd8c19895c493f83f296fb88ad1c2892239d22968422c」），主 agent 把钉改成第四段冻结根的表 `9d5fd9fc20a555a5a98043afcd45293b1c3bc69a523e1c8d35531b69b0d278f0` 之后在冻结根 `/tmp/claude-1000/e165-r2-segment4/root/`（原地 `sha256sum -c` 清单 222 行段首段尾都过）上跑完，产物在下面「Q1 后半的计数」一段点名。单测与装置变异都没跑（checker 档，归提交时的门禁 checker-tier-crates-mutation-replay）。装置在冻结根上编（`research/prompts/e165-r2-prereg.md` 修订 2、3、4 逐个写清冻结根、清单与哈希）；r1 档三份产物 sha256 逐份等于登记值（`research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment4-profile-check.out` 前三行 `identical=true`）。

| 问题 | 结论 | 依据（产物，原样数字） |
|---|---|---|
| Q1 前半：J6 照 Qn 判，今天的实现变不变 | **变，只变在对抗一族**：取 J6Qn「前提为真」那一行（调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 10 批之前的判）。合法历史一族 11 个域都是 0；对抗一族 A 85 400、AC 2 904（A6 在「前提假」那一行 496，前提为真是 0） | `research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-adversarial-family.out` 第 122 行：`E7RESULT name=verdict domain=A ring_slots=64 smallest_n=1 largest_n=4 criteria=r2 j6qn_premise_holds_violations=85400 j6qn_premise_holds_judged=45149528 …`；合法历史一族 `grep -c 'name=verdict .*criteria=r2 ' …segment1b-legal-family.out` 为 11，`j6qn_premise_holds_violations` 十一行全 0。与 Verus 不动点前件冲不冲突在调度记录第 10 批 10-5 条查，不在这一页判 |
| Q1 后半：甲表逐行 J6 与 J6Qn 首红 | **变**（SQ2 字面：任一行任一域不同）。`kill_bound` 29 行、比了 28 行 × 13 个域 = 364 格（SV3 的数对上）：355 格 J6 与 J6Qn 首红的 N 相同，**9 格不同，全在 E1，全是 J6 红而 J6Qn 界内不红**（登记行 24、406、414、448、512、517、921、922 与 Z1）；J6Qn 比 J6 早红 0 格、晚红 0 格。登记 1003（裸加一）与 1004（加二才算溢出）两行 J6 与 J6Qn 首红同一个 N（E1 N = 2、N = 1），与推 s2 对这两行的预测一致；推 s2 的另一半「不同的只可能是 1003、1004」被推翻：不同的 9 格在别的 8 行上 | `research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out`（「Q1 后半的计数」一段的命令与原样输出）；segment2b 那一遍作废（SV3：23 行、比了 22 行，五个变体启动自检 panic），它的 22 行与这一遍同名的 23 行在六个字段上逐行相同 |
| Q2：S1 那条「其余字段等于所选根、卸载记号除外」 | **成立**（界内，两种奇偶都是）：J7a 三格、J7b、J7c 在合法历史一族、对抗一族、多盘三个域上全 0 | `grep 'name=verdict .*criteria=r2 ' <产物> \| grep -o 'j7[a-z_]*violations=[0-9]*' \| grep -vc '=0$'`：segment1b 合法历史一族、对抗一族、segment3 多盘、segment1b 两份奇偶反过来的产物，五份输出都是 0（各份 `criteria=r2` 行数 11、3、3、11、3） |
| Q3：多盘 J1–J6 红不红 | **不红**（取 J6Qn「前提为真」那一行）：M、M6、MC 三个域都是 0，J0–J4、J7 全 0。M6 的「前提假」那一行 75 492 照报（所选根之上的记录条数超过在飞上限 2 时第二次调用接着施加，机理同第一段 L6、L9、C6）；MC 的 J5 在宽对上今天的实现与理想恢复同格同值 512 750 / 0 | `research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment3-multi-disk.out` 第 327 行：`E7RESULT name=verdict domain=M6 ring_slots=6 smallest_n=1 largest_n=6 criteria=r2 j6qn_premise_holds_violations=0 j6qn_premise_holds_judged=44004 j6qn_premise_fails_violations=75492 j6qn_premise_fails_judged=156816 …` |
| Q4：记录表由 `scan_journal` 建，与直接给表同不同 | **同（今天的盘次序）**：`--table both` 下 17 个域（合法历史一族 11、对抗一族 3、多盘 3）每个 (域, N) 的 `tables_differ` 与 `rows_differ` 都是 0，共 1.15 亿态。**随盘次序翻面**（SF11、SQ16）：内存盘报 1 号盘在前时，直接建表「盘号升序」那一栏在 R 域不同（`tables_differ` 3 482、`rows_differ` 1 344，N = 2..5 依次 2、32、344、3 104），「照 `device_identities()` 次序」那一栏 0 / 0；M、MC 两栏 0。盘次序反过来时（R 域与多盘一族）代表实现的是「照 `device_identities()` 次序」那一栏：`scan_journal` 按读者交的盘表次序逐盘读（`crates/singlefs-core/src/recovery.rs` 的 `for device in reader.device_identities()`），「盘号升序」那一栏只在调用方恰好按盘号交盘表时与实现一致（代码轮判决 `research/prompts/m3-formal-code-r1-main-verification.md` 第二节装置那一格） | 「Q4 的计数」一段的命令与原样输出 |

**Q1 后半的计数**，一条命令从产物里数（`research/results/` 下，读每条 `kill_bound` 行的 `first_red_by_criterion`，逐域比 J6 与 J6Qn 的首红 N；基线行没有这一字段，跳过）原样输出：

```
grep 'name=kill_bound' e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out | awk '{it="";tr="";f=""; for(i=1;i<=NF;i++){if($i~/^item=/)it=substr($i,6); if($i~/^table_row=/)tr=substr($i,11); if($i~/^first_red_by_criterion=/)f=substr($i,24)} if(f=="")next; rows++; n=split(f,p,";J6Qn:"); sub(/^J6:/,"",p[1]); split(p[1],a,","); split(p[2],b,","); for(k in a){split(a[k],x,"=");split(b[k],y,"="); cells++; if(x[2]!=y[2]){diff++; print it, tr, x[1], "J6="x[2], "J6Qn="y[2]}}} END{print "rows="rows" cells="cells" differ="diff}'
tsv-24 24 E1 J6=2 J6Qn=none
tsv-402 402 E1 J6=3 J6Qn=none
tsv-402 410 E1 J6=3 J6Qn=none
tsv-444 444 E1 J6=3 J6Qn=none
tsv-508 508 E1 J6=3 J6Qn=none
tsv-513 513 E1 J6=3 J6Qn=none
tsv-915 915 E1 J6=2 J6Qn=none
tsv-916 916 E1 J6=2 J6Qn=none
Z1 7 E1 J6=2 J6Qn=none
rows=28 cells=364 differ=9
```

（`table_row` 是冻结根里表的行号，对登记行号：24→24、402→406、410→414、444→448、508→512、513→517、915→921、916→922，Z1 是自造表第 7 行。）产物计数：`grep -c 'name=kill_bound ' …segment2c-kill-bounds.out` 29，`name=config`、`name=restored`、`name=verdict`、`name=done` 各 1 行；`outcome=red` 27、`outcome=red_only_by_implementation_anchor` 1（表第 226 行）、`outcome=baseline` 1，28 行 `run_finished=true abort=none`；收尾判决行原样：`E7RESULT name=verdict scope=segment2 variants=27 variants_excluding_baseline=26 red=25 red_only_by_implementation_anchor=1 not_red_within_bounds=0 invalid=0 run_did_not_finish=0 ledger_sha256=3542851d68fc48358e4d419cde590e981d423c11f76933e3005968d8c172ba9e`（红例账 `…segment2c-red-examples.tsv` 166 行：表头 1 行加 165 个红例，第 8、9 列「原实现与变体单条重放都与逐态行相同」165 行全是 `true`）。基线 1 373 行，收尾打了「已还原，基线与开头逐字相同」，驱动没退 5、6、7；Z5 在 12 个域红、只红 J7b、E2 界内不红（SQ5 成立，SF5 不触发）。

**SF3 那九格（J6 红而 J6Qn 界内不红，E1）按两种形态各写一个见证**（红例账第 12 列原实现的逐态行、第 13 列变体的逐态行，原样）：

- 形态一（登记行 24、512、517、921、922 与 Z1，六格）：变体让 E1 的第一次调用一条没施加，ρ_1 的 txg 是 18446744073709551614（u64 最大值减一，`applied0`）；原实现在同一态上第一次施加到 18446744073709551615、第二次因顶格报错，原 J6 记违例而 J6Qn 不记。同一 (E1, N) 上变体 J4 红（红例账第 7 列 `+J4`）。登记行 24（`tsv-24`，E1 N = 2 序号 3，账第 13 行）：
  - 原实现：`E7RESULT name=state domain=E1 ring_slots=64 n=2 ordinal=3 counters=states,j6_premise_holds_judged,J6_premise_holds,j6_third_call_differs_or_is_missing,applied_ge1,farthest_ge1,j6qn_premise_holds_judged,first_returned_root_txg_is_the_largest,j7_judged,j7b_judged state=shape=1c0+1c0;on_disk=good,good;residue_at=none;chosen=0;farthest=1;verify=true;first=returned:txg18446744073709551615:watermark1001:applied1`
  - 变体：`E7RESULT name=state domain=E1 ring_slots=64 n=2 ordinal=3 counters=states,J4,j6_premise_holds_judged,farthest_ge1,j6qn_premise_holds_judged,j7_judged,j7c_judged state=shape=1c0+1c0;on_disk=good,good;residue_at=none;chosen=0;farthest=1;verify=true;first=returned:txg18446744073709551614:watermark1000:applied0`
- 形态二（登记行 406、414、448，三格；segment2b 没有这三行）：变体施加了不该施加的发布，ρ_1 的 txg 是 18446744073709551615（u64 最大值，`applied1`），变体的逐态行带 `J6_premise_holds` 与 `j6_third_call_differs_or_is_missing`（原 J6 记违例）而没有 J6Qn 的违例计数；同一 (E1, N) 上变体 J2、J4 都红（第 7 列 `+J2,+J4`），这个态 `farthest=0`（j\* 是 0，变体却施加了一条）。登记行 448（`tsv-444`，E1 N = 3 序号 4，账第 50 行）：
  - 原实现：`E7RESULT name=state domain=E1 ring_slots=64 n=3 ordinal=4 counters=states,j6_premise_holds_judged,j6qn_premise_holds_judged,j7_judged,j7c_judged state=shape=1c0+2c0;on_disk=-,-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg18446744073709551614:watermark1000:applied0`
  - 变体：`E7RESULT name=state domain=E1 ring_slots=64 n=3 ordinal=4 counters=states,J2,J4,j6_premise_holds_judged,J6_premise_holds,j6_third_call_differs_or_is_missing,applied_ge1,j6qn_premise_holds_judged,first_returned_root_txg_is_the_largest,j7_judged,j7b_judged state=shape=1c0+2c0;on_disk=-,-,good;residue_at=none;chosen=0;farthest=0;verify=true;first=returned:txg18446744073709551615:watermark1001:applied1`
- 「第二次调用报的是什么」两种形态都**没取到原始 `what`**（红例账只存逐态行，变体的二进制跑完就删了）；是推的：J6 记了违例而 J6Qn 没记，ρ_1 的 txg 是 u64 最大值，按 J6Qn (ii) 只有「第二次报 `UnitMalformed`、`what` 等于 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR`」这一支能让两条判据分开。这不是登记 5.2 说的「Qn 把别的报错判成相等」那一种（要那一种得是 ρ_1 的 txg 不顶格而第二次报错仍判相等；九格里靠 (ii) 判相等的态，ρ_1 的 txg 都是 u64 最大值，顶格由判据自己核过）；九格的变体缺陷都由 J4 在同一个 (E1, N) 上抓到（形态二另有 J2）。是不是按 SF3 另记交主 agent。

**Q4 的计数**，一条命令从产物里数（`research/results/` 下的 `e165-replay-bounded-enumeration-2026-10-02-r2-segment4-<名>.out`，`<名>` 是代码块每行冒号前的那个名字）原样输出：

```
table-both-legal-family: table_comparison_rows=65 states=49753566 tables_differ_sum=0 rows_differ_sum=0 tables_differ_in_reported_order_sum=0 rows_differ_in_reported_order_sum=0
table-both-adversarial-family: table_comparison_rows=10 states=64513748 tables_differ_sum=0 rows_differ_sum=0 tables_differ_in_reported_order_sum=0 rows_differ_in_reported_order_sum=0
table-both-multi-disk: table_comparison_rows=16 states=1321584 tables_differ_sum=0 rows_differ_sum=0 tables_differ_in_reported_order_sum=0 rows_differ_in_reported_order_sum=0
reversed-disk-order-legal-family: table_comparison_rows=4 states=6964 tables_differ_sum=3482 rows_differ_sum=1344 tables_differ_in_reported_order_sum=0 rows_differ_in_reported_order_sum=0
reversed-disk-order-multi-disk: table_comparison_rows=10 states=1120764 tables_differ_sum=0 rows_differ_sum=0 tables_differ_in_reported_order_sum=0 rows_differ_in_reported_order_sum=0
```

（命令：`grep 'name=table_comparison' <产物> | awk …` 把每行的 `key=value` 取出来按字段求和；R 域那四行是 `reversed-disk-order-legal-family.out` 第 104、122、140、158 行。）判别力：T-表、T-行两条阳性对照在每份 `--table both` 产物里各 3 个读数全红（L 域 N ≤ 5：`tables_differ` 2 989 = `direct_table_holds_a_record` 2 989、`rows_differ` 596 > 0；`rows_differ` 3 110 = 状态数 3 110、`tables_differ` 0）；`--table both` 去掉第四段新加的行之后与默认 r2 档同一族的输出逐行相同（`profile-check.out` 三行 `r2_table_both_profile_check` 都是 `identical_after_removing_the_new_rows=true`）；多盘一族 8 线程与 4 线程两遍整份逐字节相同。每个非对抗域的「每条记录在每块盘上都落了」那一态的扫盘表条数断言在每个 (域, N) 上都真跑到了（`fully_landed_scan_table_size_checked` 大于 0）。判决行：6 份 `--table both` 与盘次序产物 `name=void`、`name=halt` 都是 0 行；判决行里「没过」类的计数（J6 前提假 / 前提真、J5a、对抗一族 AS 系、R 域 J1–J4 的 956、`tables_differ` 3 482 与 `rows_differ` 1 344）都是登记预期内或前几段已报过的数（`grep -n 'name=verdict' <产物>` 逐行列得出，没有 `false`、`not_run` 字段）。

启动自检的修（第二段 5 个变体 panic 的成因）：r2 档启动自检里读实现结局的三项（`j6qn_control_readings`、`j7_control_readings`、`odd_marker_parity_turns_the_unmount_marker_controls`）挪成单测，断言一个字没动，`name=control` 行照报。修后在冻结根的副本上把 segment2b 那 5 个变体各编一份、跑 r2 档小界（草稿，不入库）：5 个都过了 15 项启动自检、跑完（退 1：J6Qn 的阳性对照在变体下不红，记 V2），没有一个 panic。**今天的装置因此复不出挪之前的 r2 产物**：整份比，差的只有三条 `name=self_check` 行与收尾行里的计数，`profile-check.out` 三行 `r2_profile_check` 的 `identical_apart_from_the_three_moved_self_check_rows_and_their_count_in_the_run_verdict_and_done=true`；挪之后的同命令复跑留了三份新文件（名带 `-2026-10-02` 后缀）。

产物（`research/results/` 下 56 份，现数：`ls research/results | grep -c e165-replay-bounded-enumeration-2026-10-02`；逐个点名）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-02-r2-segment1-adversarial-family-threads-second-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-adversarial-family-threads-second.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-adversarial-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-legal-family-threads-second-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-legal-family-threads-second.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-odd-marker-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-odd-marker-adversarial-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-odd-marker-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-odd-marker-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1-r1-profile-check.out` | 13 份 | 第一段前任那一遍，**作废**：段尾核主工作区的冻结清单有 5 份别人改的文件不等（登记修订 2、SV9），原样留着 |
| `e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-adversarial-family-threads-second-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-adversarial-family-threads-second.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-adversarial-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-comparison.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-legal-family-threads-second-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-legal-family-threads-second.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-odd-marker-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-odd-marker-adversarial-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-odd-marker-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-odd-marker-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-r1-profile-check.out` | 14 份：合法历史一族 1 469 行、对抗一族 260 行、两份奇偶反过来 1 471 与 260 行、`comparison.out` 39 行、`r1-profile-check.out` 6 行，其余是耗时行与另一个线程数那一遍；第一段重跑那一遍 | Q1 前半、Q2 的数；`--profile r2`，8 线程与另一个线程数两遍整份逐字节相同；奇偶反过来（`--marker-parity odd`）两族各一份。挪自检之前的装置跑的 |
| `e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-adversarial-family-2026-10-02.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-legal-family-2026-10-02.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment3-multi-disk-2026-10-02.out`（257、1 466、405 行） | 3 份 | 挪自检之后的装置跑同样三条命令的输出：判决数与旧产物一个不差，少三条 `name=self_check` 行（今天的装置能复出的是这三份） |
| `e165-replay-bounded-enumeration-2026-10-02-r2-segment2-kill-bounds-timing.out`（89 行）、`e165-replay-bounded-enumeration-2026-10-02-r2-segment2-kill-bounds.out`（24 行）、`e165-replay-bounded-enumeration-2026-10-02-r2-segment2-red-examples.tsv`（126 行）、`e165-replay-bounded-enumeration-2026-10-02-r2-segment2-row-selection.out`（159 行） | 4 份 | segment2b 那一遍的第二段，**作废**（SV3：`kill_bound` 23 行、比了 22 行；五个变体启动自检 panic 没有行），不完整（没有收尾三行），不进复跑登记；22 行当线索 |
| `e165-replay-bounded-enumeration-2026-10-02-r2-segment3-multi-disk-threads-second-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment3-multi-disk-threads-second.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment3-multi-disk-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment3-multi-disk.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment3-profile-check.out`（408、408、9 行，耗时行各 18 行） | 5 份 | 第三段多盘三个域 M、M6、MC 共 1 321 584 态（Q3）；挪自检之前的装置跑的 |
| `e165-replay-bounded-enumeration-2026-10-02-r2-segment4-table-both-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-table-both-adversarial-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-table-both-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-table-both-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-table-both-multi-disk-threads-second-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-table-both-multi-disk-threads-second.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-table-both-multi-disk-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-table-both-multi-disk.out` | 8 份 | 第四段（Q4）：`--bounds segment1 --profile r2 --table both`，三族各一份，多盘另跑 4 线程那一遍 |
| `e165-replay-bounded-enumeration-2026-10-02-r2-segment4-reversed-disk-order-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-reversed-disk-order-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-reversed-disk-order-multi-disk-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r2-segment4-reversed-disk-order-multi-disk.out` | 4 份 | 盘次序反过来（SQ16）：R 域（合法历史一族）与 M、MC（多盘一族），`--table both --disk-order reversed`；一次跑只带一族，所以分两份 |
| `e165-replay-bounded-enumeration-2026-10-02-r2-segment4-profile-check.out`（14 行） | 1 份 | 段首段尾冻结清单核（222 OK）、r1 档三份 sha256 对登记值、r2 默认档与旧产物的差、`--table both` 去新行之后与默认档逐行比、线程两遍 |
| `e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out`（33 行）、`e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-red-examples.tsv`（166 行）、`e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-row-selection.out`（159 行）、`e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds-timing.out`（222 行） | 4 份 | 第二段补跑（Q1 后半，2c）：在第四段冻结根上起驱动，`--jobs 2 --threads 4`、内存上限 8G，分 12 批前台起（单条命令不许超 240 秒，驱动按 `--item` 只补没跑的；批 10 被 VS Code 重启打断后重跑），批 12 写了收尾三行；`kill-bounds.out` 有 `name=done`，`row-selection.out` 是 `--select-rows` 的输出（除第 1 行的 `table_sha256` 与文件名外，与 segment2b 那份第 2–159 行逐行相同），`timing.out` 是十三批的 stdout/stderr 与开跑前后的 `uptime`、`ps`，不进逐字比对 |

复跑登记（`research/scripts/replay.sh`）加六行：`@driver_e165_r2_segment3_multi_disk`（指带 `-2026-10-02` 后缀的第三段新文件）与第四段五个 `@driver_e165_r2_segment4_…`（各指一份第四段产物）；第 225 行（原先指第 1 次重跑的第二段）改指 `e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out`，`driver_e165_segment2` 函数体改成 `--root` 指冻结根（环境变量 `E165_R2_FROZEN_ROOT` 可换路径，默认 `/tmp/claude-1000/e165-r2-segment4/root`）、`--jobs 2 --threads 4`。这些行在主工作区上 `cargo run`，装置同一份，`crates/` 里别的文件主工作区与冻结根可能不同，复不出来先对冻结根清单。

没答的：`scan_journal` 有提示时只读提示的槽那一支（内存盘恒交回 `None`）、盘数多于两块、两块盘大小不同，都没扫；记录字节由 `JournalRecord::to_bytes` 造（装置与实现共用编码），编码与解码同错的那一格 Q4 不罩（登记第三节）。

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:357-613`（整段抄，未转述）**

````markdown
### 第 3 次重跑（择根与重放合判、改坏择根序的变体、J6Qn 按「同一实例里 journal 记录的 txg 随计数器不减」拆行）

问题单 `research/prompts/e165-r3-questions.md`（Q1、Q2、Q3），重跑登记 `research/prompts/e165-r3-prereg.md`（含修订 1 至 3）。状态：**已跑（三段）**。第一段（Q3，对抗一族 J6Qn「前提为真」那一行按该谓词拆两行）、第二段（Q1，真 `choose_root` 选根、再调真 `replay_journal`，已确认的完整发布丢不丢）与第三段（Q2，改坏择根序的四个变体有没有判别力；驱动 r3 模式，登记修订 3）都已跑，问题单三行都判完。装置的单测（56 条，第一段加 4 条、第二段加 9 条；第三段没动装置）与 `crates/mutations.tsv` 里第一段 8 行、第二段 11 行装置变异归提交时跑，这三段的结论标「单测与装置变异未跑」。

被测对象仍是 `replay_journal`（同一张记录表、同一块内存盘、同一个读者调两次，第二次拿第一次返回的根当所选根），只加了拆行：「同一实例里 journal 记录的 txg 随计数器不减」（调度记录里立项编号 I-8.12，还没登进 `.claude/kb/invariants.md`；定义整行在 `research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行）判在「这一态交给 `replay_journal` 的那张记录表」上：表按（实例，计数器）升序摊开，同一实例里相邻两条后一条的 txg 小于前一条即不成立；`parse` 拒收的序号 0 进不了表，不看。装置里是本地写法 `record_table_satisfies_i812`，不调 `crates/` 里 Verus 前件 `journal_txg_does_not_decrease_with_the_counter_within_an_instance`（入参同是交给核的那张表）。

| 问题 | 结论 | 依据（产物，原样数字） |
|---|---|---|
| Q3：第 2 次重跑对抗一族 J6Qn「前提为真」那一行的违例，按该谓词拆开之后各多少 | **全在谓词不成立那一边**：「前提为真 · 谓词成立」那一行在对抗一族三个域、每个 n（10 个 (域, n) 格）都是 0 违例；第 2 次重跑的 85 400（A）与 2 904（AC）全部落在「前提为真 · 谓词不成立」那一行：A n = 3 是 464、n = 4 是 84 936，AC n = 3 是 2 904，A6 全是 0。拆开的四行对得回未拆的两行，三个域各一行 `split_rows_equal_the_unsplit_rows=true` | `research/results/e165-replay-bounded-enumeration-2026-10-02-r3-segment1-adversarial-family.out` 第 210、294、378 行（三个域的 `criteria=r3_i812` 判决行，原样抄在下方代码块）；每个 (域, n) 的拆行命令与原样输出在下方第二个代码块 |

三个域的 `criteria=r3_i812` 判决行（`research/results/e165-replay-bounded-enumeration-2026-10-02-r3-segment1-adversarial-family.out` 第 210、294、378 行，整行）：

```
E7RESULT name=verdict domain=A ring_slots=64 smallest_n=1 largest_n=4 criteria=r3_i812 j6qn_premise_holds_i812_holds_violations=0 j6qn_premise_holds_i812_holds_judged=23886680 j6qn_premise_holds_i812_fails_violations=85400 j6qn_premise_holds_i812_fails_judged=21262848 j6qn_premise_fails_i812_holds_violations=0 j6qn_premise_fails_i812_holds_judged=0 j6qn_premise_fails_i812_fails_violations=0 j6qn_premise_fails_i812_fails_judged=0 i812_holds=28992168 i812_fails=28593152 split_rows_equal_the_unsplit_rows=true
E7RESULT name=verdict domain=A6 ring_slots=6 smallest_n=1 largest_n=3 criteria=r3_i812 j6qn_premise_holds_i812_holds_violations=0 j6qn_premise_holds_i812_holds_judged=425782 j6qn_premise_holds_i812_fails_violations=0 j6qn_premise_holds_i812_fails_judged=168960 j6qn_premise_fails_i812_holds_violations=48 j6qn_premise_fails_i812_holds_judged=32768 j6qn_premise_fails_i812_fails_violations=448 j6qn_premise_fails_i812_fails_judged=32768 i812_holds=532838 i812_fails=256000 split_rows_equal_the_unsplit_rows=true
E7RESULT name=verdict domain=AC ring_slots=64 smallest_n=1 largest_n=3 criteria=r3_i812 j6qn_premise_holds_i812_holds_violations=0 j6qn_premise_holds_i812_holds_judged=3536486 j6qn_premise_holds_i812_fails_violations=2904 j6qn_premise_holds_i812_fails_judged=1593344 j6qn_premise_fails_i812_holds_violations=0 j6qn_premise_fails_i812_holds_judged=0 j6qn_premise_fails_i812_fails_violations=0 j6qn_premise_fails_i812_fails_judged=0 i812_holds=4116166 i812_fails=2023424 split_rows_equal_the_unsplit_rows=true
```

每个 (域, n) 上「前提为真」两格的违例数与 judged，一条命令从产物里数（`research/results/` 下，工作目录是那里）原样输出：

```
grep 'criterion=J6Qn_premise_holds_i812_' e165-replay-bounded-enumeration-2026-10-02-r3-segment1-adversarial-family.out | awk '{d="";n="";c="";v="";j=""; for(i=1;i<=NF;i++){if($i~/^domain=/)d=$i; if($i~/^n=/)n=$i; if($i~/^criterion=/)c=$i; if($i~/^violations=/)v=$i; if($i~/^judged=/)j=$i}; print d, n, c, v, j}'
domain=A n=1 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=138
domain=A n=1 criterion=J6Qn_premise_holds_i812_fails violations=0 judged=0
domain=A n=2 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=8210
domain=A n=2 criterion=J6Qn_premise_holds_i812_fails violations=0 judged=1280
domain=A n=3 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=450202
domain=A n=3 criterion=J6Qn_premise_holds_i812_fails violations=464 judged=200448
domain=A n=4 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=23428130
domain=A n=4 criterion=J6Qn_premise_holds_i812_fails violations=84936 judged=21061120
domain=A6 n=1 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=138
domain=A6 n=1 criterion=J6Qn_premise_holds_i812_fails violations=0 judged=0
domain=A6 n=2 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=8210
domain=A6 n=2 criterion=J6Qn_premise_holds_i812_fails violations=0 judged=1280
domain=A6 n=3 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=417434
domain=A6 n=3 criterion=J6Qn_premise_holds_i812_fails violations=0 judged=167680
domain=AC n=1 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=274
domain=AC n=1 criterion=J6Qn_premise_holds_i812_fails violations=0 judged=0
domain=AC n=2 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=32290
domain=AC n=2 criterion=J6Qn_premise_holds_i812_fails violations=0 judged=5120
domain=AC n=3 criterion=J6Qn_premise_holds_i812_holds violations=0 judged=3503922
domain=AC n=3 criterion=J6Qn_premise_holds_i812_fails violations=2904 judged=1588224
```

最小见证（违例数非 0 的行里 n 最小的，同一产物第 173 行 A n = 3 与第 368 行 AC n = 3，整行）：三条记录依次是 txg 比所选根大 1（带末条标志与提交标记）、大 2（不带）、大 1（带末条标志、不带提交标记），同一实例里第三条的 txg 小于第二条，第二次调用报「不止一条末条标志」：

```
E7RESULT name=violations domain=A ring_slots=64 n=3 criterion=J6Qn_premise_holds_i812_fails violations=464 judged=200448 witness=anchor=absent;slots=t+1:o1:L1:C1:x1:chain_true,t+2:o1:L0:C0:x1:chain_true,t+1:o1:L1:C0:x1:chain_true;first=returned:txg101:watermark9001038:applied1;expected=txg100:watermark1000/txg101:watermark9001038/txg101:watermark9001038;first_root_txg=101;second=failed:more_than_one_last_record_flag
E7RESULT name=violations domain=AC ring_slots=64 n=3 criterion=J6Qn_premise_holds_i812_fails violations=2904 judged=1588224 witness=anchor=absent;slots=t+1:o1:L1:C1:x1:chain_true,t+2:o1:L0:C0:x1:chain_true,t+1:o1:L1:C0:x1:chain_true;first=returned:txg101:watermark9001038:applied1;expected=txg100:watermark1000/txg101:watermark9001038/txg101:watermark9001038;first_root_txg=101;second=failed:more_than_one_last_record_flag
```

**附带读到的**（同一产物，只报）：「前提为假」两格只在 A6 n = 3 非空，谓词成立 48 违例（judged 32 768）、不成立 448 违例（judged 32 768），合起来 496，对得上 `criterion=J6Qn_premise_fails` 一行的 496（第 285、286 行）；A 与 AC 的「前提为假」两格 judged 都是 0（所选根之上的记录条数在 64 槽环上不会超过在飞上限 21）。拆分类计数 `name=i812_states` 十行（每个 (域, n) 一行，第 122 行起）逐格等于登记第七节乙类的钉值：A n = 4 谓词成立 28 459 330、不成立 28 337 152，AC n = 3 成立 4 079 970、不成立 2 017 280，A6 n = 3 成立 523 570、不成立 254 464；三个域各 n 的态数之和等于原登记 5.7 的 57 585 320、788 838、6 139 590。

**判别力与作废条款**（都在这一份产物与 `profile-check.out` 里）：阳性对照 80 个读数、`controls_not_red=0`，其中新判据 `J6Qn_i812_split` 的假结局（第一次返回所选根、第二次报「不止一条末条标志」）在对照界 A n ≤ 2、A6 n ≤ 3、AC n ≤ 2 的七个 (域, n) 上各读 10 个数（四格各违例数与 judged、成立与不成立的态数），70 个全等于登记第七节乙类钉的数；`void_conditions=0 halt_conditions=0`；r1 档三份产物与 r2 档对抗一族那一份在今天的二进制上复得出，整份 sha256 逐个等于登记值（`e165-replay-bounded-enumeration-2026-10-02-r3-segment1-profile-check.out` 四行 `old_product_check`，`identical_to_registered=true`）；r3 档对抗一族去掉新加的 123 行之后与同一份二进制 r2 档同命令输出逐行相同（同一文件 `r3_against_r2_check` 一行，`removed_total=123`、`identical=true`）；8 线程与 4 线程两遍整份逐字节相同（`threads_check` 一行，`sha256` 都是 `19092224bbcb43cd17f77594d74ec824d863bb274dd7f3f49c6f874d8c4f5eb6`）。几何敏感性：6 槽小环（A6）、64 槽环（A）、带反向链对错的 AC 三处，「前提为真 · 谓词成立」都是 0，判决不随这三个旋钮翻面。

产物（`research/results/` 下 5 份）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-02-r3-segment1-adversarial-family.out` | 380 | `--bounds segment1 --family adversarial --profile r3`，8 线程；64 513 748 态，收尾 `name=done emitted=380` |
| `e165-replay-bounded-enumeration-2026-10-02-r3-segment1-adversarial-family-threads-second.out` | 380 | 同一条命令 4 线程；与 8 线程那一份整份逐字节相同 |
| `e165-replay-bounded-enumeration-2026-10-02-r3-segment1-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-02-r3-segment1-adversarial-family-threads-second-timing.out` | 各 12 | 两遍的 `E7TIMING` 行，不进逐字比对：整遍 `elapsed_seconds=111.765 threads=8` 与 `elapsed_seconds=195.872 threads=4`（同机负载均值：开工与试跑时的 `uptime` 读到 45 与 57） |
| `e165-replay-bounded-enumeration-2026-10-02-r3-segment1-profile-check.out` | 6 行数据加注释头 | 旧产物复得（r1 档三份、r2 档对抗一族）、r3 对 r2 的逐行比（头里抄着比对脚本全文）、线程两遍比 |

装置与根：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` sha256 `2ee57fe0eddafc06a03d4873d772259aa557806f777b76ce64ffbb586a7af6ad`（加 `--profile r3`、拆行、假结局 KF3 的对照、本地的谓词判法与钉值断言、4 条新单测、改过 2 条原有单测（判全表那条改用 r3 档、档参数那条加 r3 与 r4）；改前 `a0fdc9e2…`）；产物是第一段冻结根（拷主工作区、233 个文件，清单 sha256 `96dfb902556f04c29fbfcbc06050315217133c96560ad775edc91f877289f1dc`，段首段尾 `sha256sum -c` 都退 0）上编的 release 二进制跑的，二进制 sha256 `661d8e7c3467ef71b15218d8022966beecbb59c7913cc14c8a0c1a68fcc4d6b6`。编码纪律 lint（`cargo clippy -D warnings` 带 check.sh 那几条）零告警，`cargo build` 与 `cargo test --no-run` 过。

**它答不了的（第 3 次重跑第一段）**：Q3 只回答「第 2 次重跑那批违例按该谓词落在哪一边」；谓词成立那一行为 0 不等于 Verus 的证明、spec 与可执行代码对得上，只说这 64 513 748 个对抗态里没有一个「谓词成立、前提为真」的态让第二次调用偏离第一次（界外、别的字母表没扫）。该谓词判在交给核的那张表上（`parse` 拒收的序号 0 不进表）；改判「盘上全部槽」那一读法没量（登记 9.2 把它登成一条变异，没报数）。Q1、Q2 没有数。

#### 第 3 次重跑第二段（Q1：择根与重放合判，2026-10-03）

登记 `research/prompts/e165-r3-prereg.md` 修订 2（写在第二段产物之前，依据是装置在 light 档与 K 域 `--max-u 3` 上不入库的试跑读数）。装置加 `--family root_choice`（只在 `--profile r3`；`--family all` 不含它），r1、r2 档的读者与输出一个字没动。被测对象是 `recover` 的 `Consult` 支里串起来的三步：`choose_root`、`scan_journal`、`replay_journal(…, true)`；`choose_system_configuration` 与三步之外的胶水不在里面，装置照状态的几何造一份系统配置的结构体字面交给三步。

- **状态**：一个状态 = 一块内存盘上同时摆好根环（三个区域归属 0 / 1 / 0、每区 S 槽）与 journal 环（两盘镜像）。K（环 64 槽、S = 8、每次发布 1 或 2 条记录）、K9（环 9 槽、在飞上限 3）、KW（S = 4，根环在 txg 12 回绕盖掉 mkfs 的第 0 代根）三个域是合法历史：mkfs 种第 0 代根，之后 M 次挂载，每次挂载写行与暖机的空发布、再 u_m 次用户发布，崩溃点按设备切段（一次 k 条记录的发布 4^k + 1 个崩溃点），根与记录由生成器按持久顺序造。KA 不是合法历史：根环读根环次序的前三个槽各取空或四条 (txg, 实例代号) 不同的根，journal 环空，125 态，只给择根序变体当阳性对照。
- **判据**（各报各的）：版本 = (`checkpoint_txg`, 实例代号)，txg 为主、实例代号破平局。KJ0：三步不 panic 且 `choose_root` 交回一条根；KJ1：落到的版本不早于崩溃前最后一次已确认的用户发布（已确认 = 崩溃点所在那次发布之前的用户发布，写行与暖机的空发布不是用户的 fsync）；KJ2：不早于根环里任一条自证过的根；KJ3：落到的根的六个投影字段等于理想恢复。`replay_journal` 报错时落到的版本取所选根的。另逐槽调 `read_root_ring_slot` 与生成时的真相对拍。
- **判据来源**：KJ1、KJ2 是问题单 Q1 的两句；已确认的认法照 D16（发布语义） 已定项 7；择新序照 D22（单元原子性怎么合成） 已定项 7 与 D13（验证路线） 已定项 1 M3 的版本序；链首与理想恢复照 D23（journal 的角色与格式） 已定项 14 注 1；暖机照 D16（发布语义） 已定项 8；C650（择根与重放没有合判） 的验收是「在一族状态上调真 `choose_root` 再调重放，判已确认的完整发布不丢」。

| 问题 | 结论 | 依据（产物，原样数字） |
|---|---|---|
| Q1：真 `choose_root` 选根、再调真 `replay_journal`，在界内有没有已确认的完整发布被恢复丢掉 | **不丢**（只在界内、合法、不带故障）：K、K9、KW 三个域合计 2 937 126 态，其中有已确认发布的 2 811 917 态，KJ1 与 KJ2 违例数每个 (域, M, U) 格都是 0，KJ3、KJ0 也是 0；三步不 panic、`choose_root` 没有交回空、`replay_journal` 没有报错；读槽对拍与真相逐槽一致。KA（不是合法历史）KJ2 也是 0（124 个有根的态），KJ0 的 1 是根环全空那一态，登记钉的数 | `research/results/e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out` 第 266、297、373、377、378 行（四个域与整轮的判决行，整行抄在下方代码块）；逐 (域, M, U) 格的行与计数命令在下方 |

判决行（`research/results/e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out` 第 266、297、373、377、378 行，整行）：

```
E7RESULT name=verdict domain=K ring_slots=64 slots_per_region=8 largest_mounts=3 largest_user_publishes=4 criteria=root_choice states=2931574 confirmed_states=2808855 kj0_violations=0 panicked=0 choose_root_returned_none=0 replay_failed=0 kj1_violations=0 kj1_judged=2808855 kj2_violations=0 kj2_judged=2931574 kj3_violations=0 kj3_judged=2931574 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict domain=K9 ring_slots=9 slots_per_region=8 largest_mounts=2 largest_user_publishes=2 criteria=root_choice states=5472 confirmed_states=2997 kj0_violations=0 panicked=0 choose_root_returned_none=0 replay_failed=0 kj1_violations=0 kj1_judged=2997 kj2_violations=0 kj2_judged=5472 kj3_violations=0 kj3_judged=5472 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict domain=KW ring_slots=64 slots_per_region=4 largest_mounts=1 largest_user_publishes=14 criteria=root_choice states=80 confirmed_states=65 kj0_violations=0 panicked=0 choose_root_returned_none=0 replay_failed=0 kj1_violations=0 kj1_judged=65 kj2_violations=0 kj2_judged=80 kj3_violations=0 kj3_judged=80 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict domain=KA ring_slots=64 slots_per_region=8 largest_mounts=0 largest_user_publishes=0 criteria=root_choice states=125 confirmed_states=0 kj0_violations=1 panicked=0 choose_root_returned_none=1 replay_failed=0 kj1_violations=0 kj1_judged=0 kj2_violations=0 kj2_judged=124 kj3_violations=0 kj3_judged=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict scope=run bounds=segment1 family=root_choice states=2937251 self_checks_passed=1 control_readings=186 controls_not_red=0 void_conditions=0 halt_conditions=0 profile=r3 marker_parity=even
```

每个域每条判据的格数、违例数合计与 judged 合计，一条命令从产物里数（`research/results/` 下，工作目录是那里）原样输出：

```
grep 'name=violations' e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out | awk '{d="";c="";v=0;j=0; for(i=1;i<=NF;i++){if($i~/^domain=/)d=substr($i,8); if($i~/^criterion=/)c=substr($i,11); if($i~/^violations=/)v=substr($i,12); if($i~/^judged=/)j=substr($i,8)}; r[d" "c]++; V[d" "c]+=v; J[d" "c]+=j} END{for(k in r) print k, "cells=" r[k], "violations=" V[k], "judged=" J[k]}' | sort
K9 KJ0 cells=6 violations=0 judged=5472
K9 KJ1 cells=6 violations=0 judged=2997
K9 KJ2 cells=6 violations=0 judged=5472
K9 KJ3 cells=6 violations=0 judged=5472
KA KJ0 cells=1 violations=1 judged=125
KA KJ2 cells=1 violations=0 judged=124
K KJ0 cells=15 violations=0 judged=2931574
K KJ1 cells=15 violations=0 judged=2808855
K KJ2 cells=15 violations=0 judged=2931574
K KJ3 cells=15 violations=0 judged=2931574
KW KJ0 cells=15 violations=0 judged=80
KW KJ1 cells=15 violations=0 judged=65
KW KJ2 cells=15 violations=0 judged=80
KW KJ3 cells=15 violations=0 judged=80
```

违例数非 0 的行只有一行（`grep 'name=violations' <产物> | grep -v 'violations=0 '`，产物第 375 行）：

```
E7RESULT name=violations domain=KA ring_slots=64 slots_per_region=8 mounts=0 user_publishes=0 criterion=KJ0 violations=1 judged=125 witness=choose_root_returned_none;ka_region0_slots012=[empty,empty,empty]
```

每类结果行的条数（`grep -o '^E7RESULT name=[a-z_0-9]*' <产物> | sort | uniq -c`，原样输出）：`1 name=config`、`186 name=control`、`1 name=done`、`2 name=in_flight_limit`、`37 name=root_choice_states`、`1 name=self_check`、`5 name=verdict`、`146 name=violations`，合计 379，等于 `name=done emitted=379`。37 个格 = K 15（M ≤ 3 × U 0..4）+ K9 6（M ≤ 2 × U 0..2）+ KW 15（U 0..14）+ KA 1；违例行 146 = 36 个合法历史格 × 4 条判据 + KA 2 条（KA 不判 KJ1、KJ3）。

**判别力与作废条款**（都在这一份产物里）：阳性对照 186 个读数、`controls_not_red=0`（`grep 'name=control' <产物> | awk '{print $3}' | sort | uniq -c`：`1 criterion=KJ0`、`72 criterion=KJ1`、`77 criterion=KJ2`、`36 criterion=KJ3`）。假结局不调实现：KF1（所选根取真相里版本最小的根，落点照理想恢复从它接）下每格 KJ1、KJ2 各等于登记第七节乙类 KF1 那两列，KF2（落到最后一次已确认那一版的前一版）下每格 KJ1、KJ2、KJ3 各等于「有已确认发布的态数」；KJ0 在 K 域 M = 1、U = 0 那一格的指定一态上 panic，读数 1（产物第 5 行）；KA 上本地写的五种择根序的 KJ2 读数是今天的序 0、平局取低 36、实例代号为主 18、比较不看实例 18、取第一个有效者 52（第 186 至 190 行，`must_see=equals_…` 逐个 `red_as_required=true`）。37 个格的状态数、有已确认发布的态数、读根环次序里第一个自证过的根不是最大那条的态数、同 txg 不同实例同在的态数（0）、实例代号更高而 txg 更低同在的态数（0）、两个以上实例同在的态数、理想恢复施加过的态数、所选根之上记录条数的最大值与盘上最大计数器，装置每次跑都对着独立生成器（另一份 python 脚本，不读装置）的数 `assert_eq!`，一个不符当场 panic，这一份产物是全部断言过了才写出来的；`void_conditions=0 halt_conditions=0`，读槽对拍不一致的槽数 0（每格 `slot_readings_differing_from_the_truth=0`）。

**几何敏感性与轨迹**（第八节的旋钮，各取方向相反的点）：根环每区槽数 8（K、K9）对 4（KW，回绕）：KW U = 10 与 U = 11 两格「第一个自证过的根不是最大那条」的态数是 4 与 1（产物第 348、353 行，S 没生效时两格都是 5），KJ1 至 KJ3 在这两格与别的 U 上都是 0，不随这个旋钮翻面；journal 环 64 槽（在飞上限 21）对 9 槽（上限 3）：真相里版本最大的根之上同实例的记录条数最大值在 K、K9、KW 分别是 2、2、1（`above_newest_root_max`），大于上限与等于上限的态数都是 0（只写「上限在界内咬不到今天的所选根」，K9 这一点对今天的实现没咬到，留给第三段的变体），KJ 判决两点相同；挂载次数 1、2、3：每格都是 0。这一族界内根环里同 txg 不同实例的两条根同在、实例代号更高而 txg 更低的一对根同在的态数每格都是 0：合法、不带故障的历史造不出这两种根环（登记第四节推 t1 的依据，生成器一侧的读数）。8 线程与 4 线程两遍整份逐字节相同（两份 `E7RESULT` 的 sha256 都是 `20ca1851dea7cc414fa2071f3b7bccc1a29cfe3799dc59a2ef3d036bd07423de`）。

产物（`research/results/` 下 5 份）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out` | 379 | `--bounds segment1 --family root_choice --profile r3`，8 线程；2 937 251 态，收尾 `name=done emitted=379` |
| `e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice-threads-second.out` | 379 | 同一条命令 4 线程；与 8 线程那一份整份逐字节相同 |
| `e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice-timing.out`、`e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice-threads-second-timing.out` | 各 39 | 两遍的 `E7TIMING` 行，不进逐字比对：整遍 `elapsed_seconds=68.781 threads=8` 与 `elapsed_seconds=73.919 threads=4`（同机负载均值 40 上下，两遍都被机器占满，4 线程没有比 8 线程慢出线程比） |
| `e165-replay-bounded-enumeration-2026-10-03-r3-segment2-profile-check.out` | 5 行 `old_product_check`、`r3_against_r2_check`、`threads_check` 各 1 行加注释头 | 旧产物复得（r1 档三份、r2 档对抗一族、第一段 r3 档对抗一族那一份，整份 sha256 与登记值逐个相同）、r3 对 r2 的逐行比（`removed_total=123`、`identical=true`）、这一族两遍线程比 |

装置与根：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` sha256 `acd76c379479d5e9e23bc8a1fd6552c1af4f0108655c2a69c37a4ee3c2304a58`（第一段那份 `2ee57fe0…` 之上加这一族、9 条新单测，单测共 56 条；`cargo clippy --all-targets` 带 check.sh 那几条编码纪律 lint 零告警、`rustfmt --check` 无差，`cargo build` 与 `cargo test --no-run` 过）；产物是第二段冻结根（第一段根整份拷出、盖上这份装置与 `crates/mutations.tsv`，233 个文件，清单 sha256 `c985600264b47a9ed81c8b6e538826d7899eacd4241f062f865b5c5b86a7b290`，段首段尾 `sha256sum -c` 都退 0、没有多出或少了文件）上编的 release 二进制跑的，二进制 sha256 `0dba0713491ae87c86e4fa501f323a61dbfc7569db85951e135ca21f0a32aa0a`。被测的三步是第二段冻结根里那份 `recovery.rs`（`choose_root` 与 `scan_journal`、`replay_journal` 是第 10 批合入的择根核与扫环核之后的样子，sha256 `9337dc1b50eb13afdb346ee0a58360709e3a9f2bba6e5da8948fdda5685edd18`），主工作区别的会话在改的择根胶水不在里面。装置变异：`grep -c '^E165 r3 第二段' crates/mutations.tsv` 为 11（9.2 乙的十条，加一条「KJ3 不比六个字段」），点名 6 条单测，替换文没有逐条编过。

**它答不了的（第 3 次重跑第二段）**：Q1 的「不丢」只在界内、合法历史、不带故障：根环与 journal 环的两块盘全部读得出、没有撕裂的根槽、没有掉盘、没有设备失而复得（C331（择根倒挂压过已确认的写） 那一形）、没有管理员回退与实例切换、journal 环不回绕、系统配置的内容（tail 见证、取号那一写）不在；盘数只有两块、区域归属只有 0 / 1 / 0、每区槽数只有 4 与 8、根槽宽 512、M ≤ 3、K 的 U ≤ 4、每次发布至多 2 条记录，界外没扫；零单元发布的新根段在真实写路径上取上一版的、这里两两不同，写行那次发布真实要写实例表单元并点名、这里不点名（三步只比键与投影，不读它们）。KA 不是合法历史，它上面的数不翻 Q1。判据一侧的「已确认」、理想恢复与独立生成器都照同一份登记文字写：登记文字本身错了，装置与独立生成器会一起错而逐格相等。三步里读盘那一层（`read_root_ring_slot` 读几个字节、fsid 与 extent 偏移两关）只在装置的读者上验过，没在真设备上验。D23（journal 的角色与格式） 已定项 14 注 2「择根按 txg 为主、记录水位按实例代号为主」那一处分歧在这一族的合法历史里造不出（根环里没有同 txg 不同实例的两条根）。Q2（改坏择根序的四个变体）的数在下一节（第三段）。装置的单测与装置变异未跑，这两段的结论在提交时跑完之前是产物层的读数。

#### 第 3 次重跑第三段（Q2：改坏择根序的变体有没有判别力，2026-10-03）

登记 `research/prompts/e165-r3-prereg.md` 修订 3（写在第三段产物之前，依据是驱动 r3 模式写完后没起驱动整轮的三样核对：`--list-items` 列 7 行、纯函数喂第二段产物与手改行、假 cargo 与假装置）。驱动 `research/scripts/e165-replay-mutation-kill-bounds.sh` 加 `--profile r3`（sha256 `1a3dafb007079a31e0609c53f541d05e0bc88f591955e6be709787b2e267e042`；不给 `--profile` 仍是 r2 那一套）。驱动 `--selftest` 装进仓之后由主 agent 跑，读数是「查了 67 项，0 项不过」、八个弄坏开关各退 1，这一读数没有入库的输出文件。装置一个字没动（sha256 仍是 `acd76c379479d5e9e23bc8a1fd6552c1af4f0108655c2a69c37a4ee3c2304a58`）。第三段改了 `crates/mutations.tsv` 的第 1812 行（第 2 次重跑第四段那一条的原文在装置里命中 2 次，改成带函数头的唯一锚点，意思不变，替换文没编过）。

**做法**：在第二段冻结根上，给登记 9.1 的六个变体（按变异名在根里的 `crates/mutations.tsv` 逐字找，每个恰 1 行、原文恰命中 1 次、命中处落在 `choose_root_slot`（`lib.rs`）或 `choose_root`（`recovery.rs`）的区间里）各建一份副本、各冷编一次；基线与每个变体都调装置 `--profile r3 --bounds segment1 --family root_choice`（每个二进制 2 937 251 态一遍），逐 (域, M, U) 比 KJ0 至 KJ3 的违例数与 `root_choice_states` 计数行。开头的基线先与第二段产物逐行比（TV7），收尾再编一次基线、与开头逐字相同才写收尾行（TV9）。命令（仓根下；基线 6 线程、每个变体 3 线程，按驱动设定）：`E165_KILL_BOUNDS_MEMORY_CAP=1G bash research/scripts/capped.sh 6 bash research/scripts/e165-replay-mutation-kill-bounds.sh --profile r3 --root <第二段冻结根> --scratch <草稿目录> --output research/results/e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out --jobs 2 --threads 3`，退 0。

| 变体（`crates/mutations.tsv` 第一列开头的文字；根里的行号） | 改的是 | 在 Q2 里 |
|---|---|---|
| 1823 择根核比较不看实例代号 | txg 平局时停在读根环次序里前面那条 | Q2 的「比较不看实例」 |
| 1824 择根核取第一个有效者 | 择了一个自证过的槽就不再换 | Q2 的「取第一个有效者」 |
| 1825 择根核 txg 平局时实例代号低者赢 | 平局取低 | Q2 的「同 checkpoint_txg 平局取低」 |
| 1826 择根核以实例代号为主、txg 只破平局 | 比较的主次序反过来 | Q2 的「实例代号为主」 |
| 1822 择根核平局取后者赢 | 择新键完全相同的两个槽取后一个 | 附带（TQ14） |
| 1827 择根胶水摊择新键时不带实例代号 | 核看到的实例代号都是 0 | 附带（TQ14） |

| 问题 | 结论 | 依据（产物，原样数字） |
|---|---|---|
| Q2：改坏择根序的变体（比较不看实例、取第一个有效者、平局取低、实例代号为主）每一个都在界内让 Q1 那一判（K、K9、KW 某一格上 KJ1 或 KJ2 的违例数与基线不同）红吗 | **没有**（只在界内、合法、不带故障）：四个里只有 1824（取第一个有效者）红；1823、1825、1826 三个在 K、K9、KW 共 36 个格上 KJ1、KJ2（连同 KJ0、KJ3）违例数与基线逐格相同，界内不红。三个变体在 KA 上读数都等于登记钉值（18、36、18，`ka_pin=match`）：变体确实改了择根序，只是合法历史里没有能让它露出来的根环——这一族对它们没有取样点（取样点不敏感，不是等价变异，也不是判据没有判别力） | `research/results/e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out` 第 4、6、7 行（三个不红的变体）、第 5 行（1824）、第 10 行（判决行，整行抄在下方代码块） |

每个变体一行的读数，一条命令从产物里数（`research/results/` 下，工作目录是那里）原样输出：

```
grep '^E7RESULT name=kill_bound ' e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out | grep -v 'item=baseline' | awk '{for(i=1;i<=NF;i++){if($i~/^(item|outcome|red_legal_domains|differing_cells|ka_kj2|ka_pin|ka_inferred)=/)printf "%s ",$i}; print ""}'
item=tsv-1822 outcome=not_red_within_bounds red_legal_domains=none differing_cells=KJ1:K=0of15,K9=0of6,KW=0of15;KJ2:K=0of15,K9=0of6,KW=0of15;KJ0:K=0of15,K9=0of6,KW=0of15;KJ3:K=0of15,K9=0of6,KW=0of15 ka_kj2=0 ka_pin=not_pinned ka_inferred=match 
item=tsv-1823 outcome=not_red_within_bounds red_legal_domains=none differing_cells=KJ1:K=0of15,K9=0of6,KW=0of15;KJ2:K=0of15,K9=0of6,KW=0of15;KJ0:K=0of15,K9=0of6,KW=0of15;KJ3:K=0of15,K9=0of6,KW=0of15 ka_kj2=18 ka_pin=match ka_inferred=none 
item=tsv-1824 outcome=red red_legal_domains=K,K9,KW differing_cells=KJ1:K=9of15,K9=2of6,KW=9of15;KJ2:K=15of15,K9=6of6,KW=11of15;KJ0:K=0of15,K9=0of6,KW=0of15;KJ3:K=15of15,K9=6of6,KW=11of15 ka_kj2=52 ka_pin=match ka_inferred=none 
item=tsv-1825 outcome=not_red_within_bounds red_legal_domains=none differing_cells=KJ1:K=0of15,K9=0of6,KW=0of15;KJ2:K=0of15,K9=0of6,KW=0of15;KJ0:K=0of15,K9=0of6,KW=0of15;KJ3:K=0of15,K9=0of6,KW=0of15 ka_kj2=36 ka_pin=match ka_inferred=none 
item=tsv-1826 outcome=not_red_within_bounds red_legal_domains=none differing_cells=KJ1:K=0of15,K9=0of6,KW=0of15;KJ2:K=0of15,K9=0of6,KW=0of15;KJ0:K=0of15,K9=0of6,KW=0of15;KJ3:K=0of15,K9=0of6,KW=0of15 ka_kj2=18 ka_pin=match ka_inferred=none 
item=tsv-1827 outcome=not_red_within_bounds red_legal_domains=none differing_cells=KJ1:K=0of15,K9=0of6,KW=0of15;KJ2:K=0of15,K9=0of6,KW=0of15;KJ0:K=0of15,K9=0of6,KW=0of15;KJ3:K=0of15,K9=0of6,KW=0of15 ka_kj2=18 ka_pin=not_pinned ka_inferred=match 
```

「15 格」= K（M ≤ 3 × U 0..4）、「6 格」= K9、「15 格」= KW（U 0..14），三个合法历史域共 36 格；KA 只有一格。基线（今天的实现）在这 36 格上 KJ0 至 KJ3 违例数全是 0（第二段）：1824 的每个「与基线不同」都是多出违例，不是少。

首红格（按 (M 升序, U 升序) 第一个违例数与基线不同的格，`M<挂载数>U<用户发布数>`）与 1824 在 KW 每个 U 上的读数：

```
grep '^E7RESULT name=kill_bound item=tsv-1824 ' e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out | grep -o 'first_red_by_criterion=[^ ]*'
first_red_by_criterion=KJ1:K=M1U2,K9=M1U2,KW=M1U2;KJ2:K=M1U0,K9=M1U0,KW=M1U0;KJ0:K=none,K9=none,KW=none;KJ3:K=M1U0,K9=M1U0,KW=M1U0
grep '^E7RESULT name=kill_bound item=tsv-1824 ' e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out | grep -o ' cells=[^ ]*' | tr ',' '\n' | grep 'KW\.\|KA'
KW.M1U0=0/0/6/6
KW.M1U1=0/0/5/5
KW.M1U2=0/5/5/5
KW.M1U3=0/5/5/5
KW.M1U4=0/5/5/5
KW.M1U5=0/5/5/5
KW.M1U6=0/5/5/5
KW.M1U7=0/5/5/5
KW.M1U8=0/5/5/5
KW.M1U9=0/5/5/5
KW.M1U10=0/4/4/4
KW.M1U11=0/0/0/0
KW.M1U12=0/0/0/0
KW.M1U13=0/0/0/0
KW.M1U14=0/0/0/0
KA.M0U0=1/-/52/-
```

格式是 KJ0 / KJ1 / KJ2 / KJ3 的违例数，KA 只判 KJ0 与 KJ2（另两条写 `-`），KA 的 KJ0 的 1 是根环全空那一态（基线也是 1，登记钉的数）。

判决行与收尾行（产物第 9、10、11 行，整行）：

```
E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows=379 root_recovery_sha256_at_start=9337dc1b50eb13afdb346ee0a58360709e3a9f2bba6e5da8948fdda5685edd18 root_recovery_sha256_at_end=9337dc1b50eb13afdb346ee0a58360709e3a9f2bba6e5da8948fdda5685edd18 root_lib_sha256_at_start=40cd99b58242a5e522a9e302fa83ceaed17cdd5c2ea8e8811e0e223a4649f08c root_lib_sha256_at_end=40cd99b58242a5e522a9e302fa83ceaed17cdd5c2ea8e8811e0e223a4649f08c copies_restored=true
E7RESULT name=verdict scope=segment3 variants=7 variants_excluding_baseline=6 red=1 red_only_by_kj0_or_kj3=0 not_red_within_bounds=5 invalid=0 run_did_not_finish=0 not_compared=0 ka_pin_mismatches=0 baseline_equals_segment2_product=true
E7RESULT name=done emitted=11
```

`red=1` 是 1824；`not_red_within_bounds=5` 是 1822、1823、1825、1826、1827（Q2 的三个加附带的两个）；`red_only_by_kj0_or_kj3=0`（没有只在 KJ0 或 KJ3 上红的）；`invalid=0 run_did_not_finish=0 not_compared=0`（七个变体都编过、跑到界、选行条件都满足）；`ka_pin_mismatches=0`（四个钉值 18、52、36、18 都对上）；`baseline_equals_segment2_product=true`（开头基线的 379 行与第二段产物逐行相同，所以这一族的计数与线程数无关：基线 6 线程，第二段两遍是 8 与 4 线程）。每类结果行的条数（`grep -o '^E7RESULT name=[a-z_0-9]*' <产物> | sort | uniq -c`，原样输出）：`1 name=config`、`1 name=done`、`7 name=kill_bound`、`1 name=restored`、`1 name=verdict`，合计 11，等于 `name=done emitted=11`；六个变体行（基线那一行不带这几个字段，只有 `run_exit=0` 与 `rows_identical_to_segment2_product=true`）每一行都带 `halt_conditions=0 void_conditions=0 run_exit=0 run_finished=true cells_without_rows=0 abort=none copy_restored=true`（`grep -c` 这串得 6）。

**没红的几个按 `.claude/rules/mutation-sampling.md` 三分**（问句：换一个取样点，它还同值吗）：

- **1823、1825、1826（Q2 的三个）：取样点不敏感（第三类）**，不是等价变异、不是真盲区。依据三条：① 变体有效、改到了择根序：KA 上 KJ2 读数 18、36、18 逐个等于登记钉值（`ka_pin=match`，没有 `invalid`），装置里本地写的五种择根序在第二段产物里读 36、18、18、52（`grep -c 'name=control.*local_choice'` 得 5）；② 换一个取样点就不同值：KA（根环里同 txg 不同实例、实例代号更高而 txg 更低的根可以同在）上 0 变 18、36、18；③ K 族里这两种根环一个都造不出：第二段产物里 36 个合法历史格的这两列合计都是 0，KA 的 125 态里分别是 48 与 24：

  ```
  grep 'name=root_choice_states' e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out | awk '{a=0;b=0;d="";for(i=1;i<=NF;i++){if($i~/^same_txg_different_instance_roots=/){split($i,x,"=");a=x[2]} if($i~/^higher_instance_lower_txg_roots=/){split($i,x,"=");b=x[2]} if($i~/^domain=/)d=$i}; if(d!="domain=KA"){n++; sa+=a; sb+=b}} END{print "legal_cells="n, "same_txg_different_instance_sum="sa, "higher_instance_lower_txg_sum="sb}'
  legal_cells=36 same_txg_different_instance_sum=0 higher_instance_lower_txg_sum=0
  grep 'name=root_choice_states domain=KA' e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out | grep -o ' states=[0-9]*\| same_txg_different_instance_roots=[0-9]*\| higher_instance_lower_txg_roots=[0-9]*'
   states=125
   same_txg_different_instance_roots=48
   higher_instance_lower_txg_roots=24
  ```

  反推（假如「K 族里没有取样点」是错的，本该看到什么）：变体在 K、K9、KW 的某格 `root_choice_states` 行或违例数与基线不同。实际三个变体的 `states_row_differences=none`，36 格的四条判据违例数与基线逐格相同；这是第二条读数（一边是生成器记下的根环形状，装置对着独立 python 生成器的数 `assert_eq!` 过；一边是真变体代码跑出的落点），两条同向；但两条共用生成器造出的状态，生成器漏造了这类根环两边会一起得 0，钉它的只有独立生成器的计数与 KA 上变体读数等于钉值（有这类根环时变体会红）。登记第四节推 t1 没被推翻（登记 TF4 没触发：三个变体都没在 K 族红）。登记 TF5 与 TF7 触发：三个变体在 K 族界内不红、同时在 KA 上等于钉值——两点各写一个数：K、K9、KW 36 格里 KJ1 或 KJ2 与基线不同的格数 0（1823、1825、1826 都是），KA 上 KJ2 违例数 18、36、18（基线 0）。
- **1824（取第一个有效者）：红**。K、K9、KW 三个域都红：KJ2 首红在 M1U0，KJ1 首红在 M1U2（第一次有已确认发布）；机制是登记推 t2 的读法（读根环次序里第一个自证过的槽是 mkfs 种的第 0 代根，界内不被盖掉；所选根是第 0 代根时一条记录都不施加，落到第 0 代），没逐态量；`differing_cells` 在 KJ2 上 K 15/15、K9 6/6、KW 11/15，在 KJ1 上 K 9/15、K9 2/6、KW 9/15，KJ3 与 KJ2 同，KJ0 为 0；KA 52 对上钉值。这与登记 9.1 表的预期（推 t2）一致，KW 那一半见下面「几何敏感性」。
- **1822、1827（附带）**：不红，KA 读数 0 与 18，等于登记写的推测值（`ka_inferred=match`，登记没把它们列进钉值，`ka_pin=not_pinned`）。1827 同 1823：K 族里没有取样点。1822 登记推的是「这四条判据、这两族上同值」：读数同值，但这一次没有造同键不同内容的两个槽，「所有输入上等价」没证，所以不写「等价变异」留档，只写这四条判据、这两族上同值；它不进 Q2。

**几何敏感性**（登记第八节的两个旋钮，各取方向相反的点）：

| 旋钮 | 取样点与读数 | 判决 |
|---|---|---|
| 根环里有没有同 txg 不同实例、或实例代号更高而 txg 更低的根 | 合法历史（K、K9、KW：两列全 0）：1823、1825、1826 的 KJ1、KJ2 与基线不同的格数都是 0（共 36 格）；KA（两种都有）：KJ2 违例数 18、36、18 | 判决随根环形状翻面（TF7）：合法历史上不红、KA 上红，两点各写一个数，不合成一个结论 |
| 读根环次序里第一个槽放的是不是最新的根（每区槽数 S 与回绕） | 1824：S = 8 的 K、K9，KJ2 与基线不同的格 15/15 与 6/6；S = 4、txg 12 回绕盖掉第 0 代根的 KW，11/15（U0 至 U10 红，U11 至 U14 四格 0） | 判决随 S 与回绕翻面（TF10）：同一个变体在 KW 的后四格不红 |
| journal 环槽数（在飞上限） | 1824：K（64 槽、上限 21）与 K9（9 槽、上限 3）KJ2 都是全格红；`above_chosen_root_over_limit` K9 是 0（基线也是 0） | K9 这一点对 1824 没起作用（TF10 末项，照记）：变体所选根之上的记录条数的最大值 K、K9 是 0、KW 是 4（基线 2、2、1），上限在界内咬不到 |
| 挂载次数 | 1824：K 的 M1、M2、M3 每个 (M, U) 格上 KJ2 违例数都非 0（`differing_cells` 的 KJ2 一栏 K=15of15；逐格读数在产物第 5 行 `cells=` 里） | 不随挂载次数翻面 |

KW U11 至 U14 那四格：第二段产物里这四格「第一个自证过的根不是最大那条」的态数是 1、5、5、5（命令与原样输出如下），而 1824 在这四格上 KJ1 至 KJ3 与基线都相同：

```
grep 'name=root_choice_states domain=KW' e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out | awk '{for(i=1;i<=NF;i++)if($i~/^(user_publishes|states|first_self_verified_root_is_not_the_newest)=/)printf "%s ",$i; print ""}' | sed -n '11,15p'
user_publishes=10 states=5 first_self_verified_root_is_not_the_newest=4 
user_publishes=11 states=5 first_self_verified_root_is_not_the_newest=1 
user_publishes=12 states=5 first_self_verified_root_is_not_the_newest=5 
user_publishes=13 states=5 first_self_verified_root_is_not_the_newest=5 
user_publishes=14 states=5 first_self_verified_root_is_not_the_newest=5 
```

登记 9.1 只预期 U = 11 那 4 态（最新的根就在区域 0 槽 0）不红；U12 至 U14 里五个态的第一个自证过的根都不是最新的，1824 照样不红。读数能支撑的只是：这几格变体所选根之上有记录（`above_chosen_root_max` KW 是 4），落点与基线相同。推的（没逐态量）：所选根是回绕后区域 0 槽 0 里的 txg 12 那条，与它同实例的后续记录在在飞上限（21）之内，重放把它接回最新的一版；要推翻它，看 U12 至 U14 的态上 `replay_applied` 与基线是否不同、落点是否真等于最新版本。

**它答不了的（第 3 次重跑第三段）**：

- Q2 的「没有」只在界内、合法、不带故障的 K 族：较新的根一时读不出又读得出、撕裂的根槽、掉盘、设备失而复得（C331（择根倒挂压过已确认的写） 那一形）、管理员回退与实例切换都没扫，而「同 txg 不同实例」「实例代号更高而 txg 更低」这两种根环正是那几类状态里才会冒出来（D22（单元原子性怎么合成） 已定项 7 写「设备失而复得会造出两条同 txg 的合法根」）。不许写成「这三条择根序没有分别」或「与它无关」：它们在 KA 上红、在 K 族上没有取样点，补敏感取样点（带故障的合法历史一族）这一次没建，要建是新一轮。KA 不是合法历史，它上面红不算 Q2 的「有」。
- 三个变体在 K 族界内不红，不等于它们没有任何东西盯着：`crates/mutations.tsv` 里 1823、1825、1826 各点名一条单测（`of_two_self_verified_roots_with_the_same_checkpoint_txg_choose_root_takes_the_higher_instance_although_it_sits_in_a_later_slot`、`of_two_slots_with_the_same_checkpoint_txg_the_higher_instance_in_the_later_slot_is_chosen`、`a_later_checkpoint_txg_is_chosen_over_a_higher_instance_with_an_earlier_checkpoint_txg`），那三条抓不抓这一次没跑（crates 变异整表是 checker 档重型，归提交时）；这里只说 E165 的 K 族没有把它们抓出来。
- 变体改的是 `choose_root_slot`（`lib.rs`）与 `choose_root`（`recovery.rs`）两个函数里的比较式，一次改一处；改两处以上、改别的择根代码、别的变体写法没扫。
- 判据一侧的「已确认」、理想恢复与独立生成器同第二段，都照同一份登记文字写：文字本身错了会一起错。KA 钉值是登记第四节脚本算的（脚本没入库，sha256 记在登记里），与装置里本地写的五种择根序、驱动里写死的钉值三处对得上，三处同出一份登记文字。
- 驱动的自证（67 项、八个弄坏开关）的输出没有入库文件；装置的单测与装置变异（第一段 8 行、第二段 11 行）未跑，这一段的结论在提交时跑完之前是产物层的读数。TQ13（用时）没有入库的计时产物，不报。驱动开头收尾的冻结根哈希（recovery.rs、lib.rs）相同；整份根清单段尾 `sha256sum -c` 退 0。

产物（`research/results/` 下 1 份）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out` | 11 | `--profile r3` 的驱动整轮：配置 1、基线 1、变体 6、还原 1、判决 1、完成 1；sha256 `7ae4dbe15085402b1a665e5aeaf49207c381d6ea020a3ff4857ba1ded74dba50` |

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:614-912`（整段抄，未转述）**

````markdown
### 第 4 次重跑（带读故障的择根与重放合判族、第 3 次没红的三个择根变体）

问题单 `research/prompts/e165-r4-questions.md`（Q1、Q2），重跑登记 `research/prompts/e165-r4-prereg.md`（含修订 1–4；修订 4 只改 T1 一句的措辞，在全部产物之后）。状态：**两段都已跑**：第一段 Q1（承诺内不丢）；第二段 Q2（驱动 r4 模式，三个择根变体在新一族上红不红：仍有不红的，三个都是取样点不敏感），写在「第 4 次重跑第二段」一节。

被测对象仍是 `recover` 的 `Consult` 支里串起来的三步（`choose_root`、`scan_journal`、`replay_journal(…, true)`，今天的 `crates/singlefs-core/src/recovery.rs`），一个字没改。这一次多出来的东西全在生成器一侧（可信件，不调 `mount.rs`、`transaction.rs`）：每块盘两个系统配置槽（世代号、tail、实例代号，或撕裂）、挂载开头的读故障、可写挂载读到的更新状态读不出那道闸、取号那一写与发布末尾的系统配置轮换。**闸取的是 2026-10-03 那一版实现的取甲**（一槽系统配置读不出或撕裂都判不出、按真；登记修订 1）：那一天 D23（journal 的角色与格式） 已定项 14 的原文写「判不出取 R2，权威写法在 D22（单元原子性怎么合成） 已定项 8……这里的实现还是 2026-09-28 取甲的那一版，欠 C331（择根倒挂压过已确认的写）」，所以第 4 次生成的历史是那一天真写路径写得出的历史。根环那条线第 7 批 7-2 之后实现取 R2（`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_witness_is_undecidable`），这一族的取甲闸与实现不再是同一个；R2 下的同一族在「第 5 次重跑」一节的 5-甲、5-乙 两步重判。

- **状态**：一个状态 = 一块内存盘上摆好根环（三个区域归属 0 / 1 / 0）、journal 环（两盘镜像）与每块盘两个系统配置槽，再加一个「被判时读故障」。六个域：KF（环 64 槽、每区 8 槽、每次发布 1 或 2 条记录，M ≤ 2、U ≤ 3）、KF3（k = 1，M ≤ 3、U ≤ 3）、KF9（环 9 槽、在飞上限 3，M ≤ 2、U ≤ 2）、KFW（每区 4 槽、根环回绕，M = 1、U ≤ 14）四个是带读故障的合法历史；KG 把条款的两道保护拿掉（可写挂载不判那一道闸与取号那一刻、写行 txg 只看读得出的根，C331（择根倒挂压过已确认的写） 一行原文「新实例的第一次发布从更旧的根往上数 txg」），不是合法历史，只当方向相反的取样点；KA 是 r3 原样的 125 个对抗根环。一段历史 = M 次挂载（含被拒的）、挂载 2 起开头各一项读故障、每次往下走的挂载的发布数 u_m、每次发布的 k 与提交标记摆法、崩溃点；一段历史出 5 个被判态（被判时故障：无、RN、RI、D0、D1）。
- **故障怎么摆**：落点只有根槽、系统配置槽、整块盘三类，journal 记录槽不单独加读故障（盘不在时那块盘上的记录拷贝随之读不出）。挂载 2 起开头的菜单 13 项：无，RN（根环里键最大的根所在的槽）、SN0、SN1（那块盘两槽里世代号较大的那一槽）、SB（SN0 加 SN1）、RI（最新实例的全部根所在的槽）、RNSB（RN 加 SN0 加 SN1）各配 T1（从这次挂载里第一次读那个落点起读不出、重读之前恢复，重读那一遍、取号那一刻的读与写侧的读都读得出，D23（journal 的角色与格式） 已定项 14「重读一次」的 R = 1；登记修订 4 把措辞从「第一次读读不出」改成这一句：系统配置槽落点上「只坏第一次读」时那一读被择系统配置吃掉、见证不判真、不走重读，生成器写的一直是「坏到重读之前」）与 TP（这次挂载里每一次读都读不出）；下一次挂载开头全部读得出。盘不在只摆在被判的那一次恢复上（历史里的可写挂载在读盘之前就拒，D2（RAID 条带策略） 已定项 13 的设备数下限）。
- **判据**（各报各的）：版本 = (`checkpoint_txg`, 实例代号)。KJ0：三步不 panic、读视图里有根时 `choose_root` 交回一条根；KJ1：落到的版本不早于崩溃前最后一次已确认的用户发布；**KJ1T**（这一族多出来的一条，依据 C331（择根倒挂压过已确认的写） 一行「怎么拦」一格「已确认返回的写，在之后每一次挂载选的根上都读得回」）：落到的版本是最后一次已确认的那一版本身或在它的后代上（父链：同一次挂载里每次发布的父是上一次发布，一次挂载第一次发布的父是这次挂载往下走所用的那一版）；KJ2：不早于读视图里任一读得出的根；KJ3：返回根的六个投影字段等于理想恢复（读视图里版本最大的根起，同实例逐次往后，每次发布每条记录至少一份读得出、条数累计不超在飞上限）。Q1 的「丢」= **承诺内**的被判态里 KJ1 或 KJ1T 有违例。承诺内 = 历史罩着 ∧ 被判时罩着 ∧ 不是上限截掉：历史里每次挂载开头的故障 ∈ {无、RN、SN0、SN1、SB、RI} 罩着、出现 RNSB 不罩（D22（单元原子性怎么合成） 已定项 8 写明的双故障形）；被判时故障 ∈ {无、RN、D0、D1} 罩着、RI 不罩（超出单故障，D16（发布语义） 已定项 7 射程、已定项 8）；条款的理想恢复照在飞上限丢了最后一次已确认的而去掉上限就不丢，记「上限截掉」。
- **来源**：闸的写法照 D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」那一段与 `mount.rs` 的 `newer_publish_witness`；写行 txg 与计数器照同一项注 3；系统配置轮换照 D16（发布语义） 已定项 7 与 D22（单元原子性怎么合成） 已定项 16（世代号 = 这块盘两槽里读得出的最大世代号加一、槽 = 世代号对槽数取余）；崩溃点按设备切段照 D13（验证路线） 已定项 4（根落了之后两块盘的系统配置写各取没写、写了、撕裂三态，最后一次挂载并成一个：三步不读系统配置槽）；暖机照 D16（发布语义） 已定项 8。C657（带故障的择根合判族没有） 的验收是「在带故障的择根合判状态上调真三步，三条变体要红，原实现照已确认的完整发布不丢判」。

| 问题 | 结论 | 依据（产物，原样数字） |
|---|---|---|
| Q1：带读故障的合法历史上，择根与重放合起来还丢不丢已确认的完整发布 | **承诺内不丢**（只在界内、两块盘、故障只在三类落点、闸取实现的取甲）：KF、KF3、KF9、KFW 四个合法域合计 11 228 080 态，其中承诺内 6 919 344 态，KJ1 与 KJ1T 违例数每个 (域, M, U, 被判时故障, 历史标签) 格都是 0；KJ0、KJ2、KJ3 在四个域每一格也都是 0（三步落在条款的理想恢复上，`replay_journal` 一次都没报错，读槽对拍与真相逐槽一致）。KJ1、KJ1T 的违例（合计 587 599，两条逐格同数，`kj1_and_kj1t_differing_states=0`）全在被判时 RI（最新实例的全部根一时读不出，不在承诺内）那一类，条款的理想恢复在同一批态上同样丢、逐格同数（`ideal_clause_violations` 与 `violations` 980 行里一行不差），上限截掉的态数是 0。KG（把两道保护拿掉，不是合法历史）上承诺内的 KJ1 违例 10 368、KJ1T 违例 20 628，被判时无故障的态上 KJ1T 是 648，等于本地理想恢复算出的 648（C331（择根倒挂压过已确认的写） 一行「判别力自证：今天的择新规则在这个注入下必须红」） | `research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out` 第 562、1371、1789、2735、3292、3302、3307、3308 行（四个合法域、KG、KA、C331（择根倒挂压过已确认的写） 自证与整轮的判决行，整行抄在下方代码块）；逐格的行与计数命令在下方 |
| Q2：第 3 次重跑没红的三个择根变体（比较不看实例、平局取低、实例代号为主）在新一族上红不红 | **仍有不红的**（只在界内、两块盘、故障只在三类落点、闸取实现的取甲；「红」按登记取四个合法域 KF、KF3、KF9、KFW，KG 与 KA 上的不同不算）：三个变体 v1823（比较不看实例）、v1825（平局取低）、v1826（实例代号为主）每一个在四个合法域共 410 个 (域, M, U, 被判时故障, 历史标签) 子格上，KJ1、KJ1T、KJ2（连同 KJ0、KJ3）违例数与基线逐子格相同（`red_legal_domains=none`），界内不红；三个都判**取样点不敏感**（第三类），不是等价变异、不是真盲区：KA 上 KJ2 读 18、36、18 等于登记钉值（`ka_pin=match`），KG 上 v1826 有 8、8、12 个子格（KJ1、KJ1T、KJ2）与基线不同、v1823 与 v1825 一格都没有，而「同 txg 不同实例同在」「实例代号更高而 txg 更低同在」两个计数在四个合法域每格 0（第一段）。附带的 v1824（取第一个有效者）在四个合法域都红，v1822（平局取后者赢）、v1827（胶水摊键不带实例代号）不红 | `research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out` 第 3 至 8 行（六个变体）、第 10 行（判决行）；整行与命令在「第 4 次重跑第二段」一节 |

判决行（`research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out` 第 562、1371、1789、2735、3292、3302、3308 行，整行）：

```
E7RESULT name=verdict domain=KF ring_slots=64 slots_per_region=8 largest_mounts=2 largest_user_publishes=3 bounds=segment1 criteria=root_choice_faults histories=235889 states=1179445 confirmed_states=1041885 kj0_violations=0 panicked=0 choose_root_returned_none_with_a_readable_root=0 replay_failed=0 kj1_violations=129321 kj1_judged=1041885 kj1t_violations=129321 kj1t_judged=1041885 kj2_violations=0 kj2_judged=1179445 kj3_violations=0 kj3_judged=1179445 promised_states=823500 promised_kj1_violations=0 promised_kj1t_violations=0 promised_rows_equal_the_split_rows=true kj1_and_kj1t_differing_states=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict domain=KF3 ring_slots=64 slots_per_region=8 largest_mounts=3 largest_user_publishes=3 bounds=segment1 criteria=root_choice_faults histories=1949698 states=9748490 confirmed_states=5172510 kj0_violations=0 panicked=0 choose_root_returned_none_with_a_readable_root=0 replay_failed=0 kj1_violations=437462 kj1_judged=5172510 kj1t_violations=437462 kj1t_judged=5172510 kj2_violations=0 kj2_judged=9748490 kj3_violations=0 kj3_judged=9748490 promised_states=5886372 promised_kj1_violations=0 promised_kj1t_violations=0 promised_rows_equal_the_split_rows=true kj1_and_kj1t_differing_states=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict domain=KF9 ring_slots=9 slots_per_region=8 largest_mounts=2 largest_user_publishes=2 bounds=segment1 criteria=root_choice_faults histories=59949 states=299745 confirmed_states=165420 kj0_violations=0 panicked=0 choose_root_returned_none_with_a_readable_root=0 replay_failed=0 kj1_violations=20772 kj1_judged=165420 kj1t_violations=20772 kj1t_judged=165420 kj2_violations=0 kj2_judged=299745 kj3_violations=0 kj3_judged=299745 promised_states=209152 promised_kj1_violations=0 promised_kj1t_violations=0 promised_rows_equal_the_split_rows=true kj1_and_kj1t_differing_states=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict domain=KFW ring_slots=64 slots_per_region=4 largest_mounts=1 largest_user_publishes=14 bounds=segment1 criteria=root_choice_faults histories=80 states=400 confirmed_states=325 kj0_violations=0 panicked=0 choose_root_returned_none_with_a_readable_root=0 replay_failed=0 kj1_violations=44 kj1_judged=304 kj1t_violations=44 kj1t_judged=304 kj2_violations=0 kj2_judged=379 kj3_violations=0 kj3_judged=379 promised_states=320 promised_kj1_violations=0 promised_kj1t_violations=0 promised_rows_equal_the_split_rows=true kj1_and_kj1t_differing_states=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict domain=KG ring_slots=64 slots_per_region=8 largest_mounts=2 largest_user_publishes=3 bounds=segment1 criteria=root_choice_faults histories=525710 states=2628550 confirmed_states=2338065 kj0_violations=0 panicked=0 choose_root_returned_none_with_a_readable_root=0 replay_failed=0 kj1_violations=319545 kj1_judged=2338065 kj1t_violations=338229 kj1t_judged=2338065 kj2_violations=0 kj2_judged=2628550 kj3_violations=0 kj3_judged=2628550 promised_states=1780672 promised_kj1_violations=10368 promised_kj1t_violations=20628 promised_rows_equal_the_split_rows=true kj1_and_kj1t_differing_states=18684 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict domain=KA ring_slots=64 slots_per_region=8 largest_mounts=0 largest_user_publishes=0 criteria=root_choice states=125 confirmed_states=0 kj0_violations=1 panicked=0 choose_root_returned_none=1 replay_failed=0 kj1_violations=0 kj1_judged=0 kj2_violations=0 kj2_judged=124 kj3_violations=0 kj3_judged=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
E7RESULT name=verdict scope=run bounds=segment1 family=root_choice_faults states=13856755 self_checks_passed=1 control_readings=448 controls_not_red=0 void_conditions=0 halt_conditions=0 profile=r4
```

C331（择根倒挂压过已确认的写） 的判别力自证与判别力自证四条（同一份产物第 3303 至 3307 行，整行）：

```
E7RESULT name=self_proof item=gate_refuses_every_history_of_the_four_tp_faults_and_the_gate_off_domain_refuses_none passed=true detail=SN0-TP:KF_histories=1238_KF_refused=1238_KG_refused=0,SN1-TP:KF_histories=1238_KF_refused=1238_KG_refused=0,SB-TP:KF_histories=1238_KF_refused=1238_KG_refused=0,RNSB-TP:KF_histories=1238_KF_refused=1238_KG_refused=0
E7RESULT name=self_proof item=t1_takes_the_second_read_in_the_gate_on_domain passed=true detail=SN0-T1:KF_reread=28466,SN1-T1:KF_reread=28466,SB-T1:KF_reread=28466
E7RESULT name=self_proof item=kf4_parts_kj1_from_kj1t_on_the_gate_off_domain passed=true detail=KF4_kj1=1995300,KF4_kj1t=2000970
E7RESULT name=self_proof item=torn_third_state_is_enumerated passed=true detail=KF_M2_U0_states_with_a_torn_system_slot=650
E7RESULT name=c331_proof domain=KG local_today_kj1t_without_a_judged_fault=648 pinned=648 implementation_kj1t_without_a_judged_fault=648 implementation_kj3_without_a_judged_fault=0
```

上面自证 `t1_takes_the_second_read_in_the_gate_on_domain` 说的是：SN0、SN1、SB 三项的 T1 在 KF 的 M = 2 格里各有 28 466 个历史走第二遍重读（不是 0；0 走登记第十节 TV4）。这里 T1 的意思照登记修订 4：从这次挂载里第一次读那个落点起读不出、重读之前恢复；它不是「只坏第一次读」。系统配置槽落点上两种读法不等价：实现第一次读系统配置槽在择系统配置那一步，只坏第一次读时那一读被它吃掉、见证读在后面读得出、不判真，不走重读（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log` 第 2、4、6、10 行，SN0、SN1、SB、RNSB 的 `timing=OnlyTheFirstRead` 都是 `outcome=mounted:OnTheFirstRead`）；坏到重读之前走重读那一遍（同文件第 3、5、7、11 行都是 `outcome=mounted:OnTheOneReread`），生成器写的是后一种，所以这条自证的 28 466 与 Q1 的数都是后一种的读数。

每个域每条判据的行数、违例数合计与 judged 合计，一条命令从产物里数（`research/results/` 下，工作目录是那里）原样输出：

```
grep 'name=violations' e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out | awk '{d="";c="";v=0;j=0; for(i=1;i<=NF;i++){if($i~/^domain=/)d=substr($i,8); if($i~/^criterion=/)c=substr($i,11); if($i~/^violations=/)v=substr($i,12); if($i~/^judged=/)j=substr($i,8)}; r[d" "c]++; V[d" "c]+=v; J[d" "c]+=j} END{for(k in r) print k, "rows=" r[k], "violations=" V[k], "judged=" J[k]}' | sort
KA KJ0 rows=1 violations=1 judged=125
KA KJ2 rows=1 violations=0 judged=124
KF3 KJ0 rows=120 violations=0 judged=9748490
KF3 KJ1 rows=120 violations=437462 judged=5172510
KF3 KJ1T rows=120 violations=437462 judged=5172510
KF3 KJ2 rows=120 violations=0 judged=9748490
KF3 KJ3 rows=120 violations=0 judged=9748490
KF9 KJ0 rows=60 violations=0 judged=299745
KF9 KJ1 rows=60 violations=20772 judged=165420
KF9 KJ1T rows=60 violations=20772 judged=165420
KF9 KJ2 rows=60 violations=0 judged=299745
KF9 KJ3 rows=60 violations=0 judged=299745
KF KJ0 rows=80 violations=0 judged=1179445
KF KJ1 rows=80 violations=129321 judged=1041885
KF KJ1T rows=80 violations=129321 judged=1041885
KF KJ2 rows=80 violations=0 judged=1179445
KF KJ3 rows=80 violations=0 judged=1179445
KFW KJ0 rows=150 violations=0 judged=400
KFW KJ1 rows=150 violations=44 judged=304
KFW KJ1T rows=150 violations=44 judged=304
KFW KJ2 rows=150 violations=0 judged=379
KFW KJ3 rows=150 violations=0 judged=379
KG KJ0 rows=80 violations=0 judged=2628550
KG KJ1 rows=80 violations=319545 judged=2338065
KG KJ1T rows=80 violations=338229 judged=2338065
KG KJ2 rows=80 violations=0 judged=2628550
KG KJ3 rows=80 violations=0 judged=2628550
```

一行 = 一个 (域, M, U, 被判时故障, 历史标签) 格，每格五条判据（KJ0 至 KJ3 加 KJ1T）；KF 8 个 (M, U) 格 × 10 个 (被判时故障, 历史标签) 格 = 80 行，KF3 12 × 10 = 120，KF9 6 × 10 = 60，KFW 15 × 10 = 150，KG 8 × 10 = 80。

KJ1、KJ1T 违例数非 0 的格，按 (域, 被判时故障, 历史标签) 加总，一条命令原样输出（行首是域；四个合法域里一个被判时故障不是 RI 的格都没有违例，KG 里 RN、D0、D1、无故障的格也有）：

```
grep 'name=violations' e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out | grep -E 'criterion=KJ1T? ' | awk '{d="";f="";h="";c="";v=0; for(i=1;i<=NF;i++){if($i~/^domain=/)d=substr($i,8); if($i~/^judged_fault=/)f=substr($i,14); if($i~/^history=/)h=substr($i,9); if($i~/^criterion=/)c=substr($i,11); if($i~/^violations=/)v=substr($i,12)}; if(v>0) s[d" "f" "h" "c]+=v} END{for(k in s) print k, s[k]}' | sort
KF3 RI inside_the_promise KJ1 330628
KF3 RI inside_the_promise KJ1T 330628
KF3 RI outside_the_promise KJ1 106834
KF3 RI outside_the_promise KJ1T 106834
KF9 RI inside_the_promise KJ1 18090
KF9 RI inside_the_promise KJ1T 18090
KF9 RI outside_the_promise KJ1 2682
KF9 RI outside_the_promise KJ1T 2682
KF RI inside_the_promise KJ1 112743
KF RI inside_the_promise KJ1T 112743
KF RI outside_the_promise KJ1 16578
KF RI outside_the_promise KJ1T 16578
KFW RI inside_the_promise KJ1 44
KFW RI inside_the_promise KJ1T 44
KG D0 inside_the_promise KJ1 6912
KG D0 inside_the_promise KJ1T 9720
KG D1 inside_the_promise KJ1T 1134
KG none inside_the_promise KJ1T 648
KG RI inside_the_promise KJ1 264051
KG RI inside_the_promise KJ1T 272475
KG RI outside_the_promise KJ1 45126
KG RI outside_the_promise KJ1T 45126
KG RN inside_the_promise KJ1 3456
KG RN inside_the_promise KJ1T 9126
```

（「inside_the_promise」「outside_the_promise」是历史标签：历史里每次挂载开头的故障罩着或不罩，不是承诺内的判定；承诺内要历史罩着、被判时罩着、不是上限截掉三样都成立，被判时 RI 一行恒不在承诺内。）

承诺内两行（KJ1、KJ1T 各一行，装置直接数的与从分格的行算的各报一遍，两个数在每个格逐个相等，`promised_rows_equal_the_split_rows=true`）与四个合法域的合计，一条命令原样输出：

```
grep 'name=verdict domain=K' e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out | grep -v 'domain=KG\|domain=KA' | awk '{for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]}; s+=v["states"]; c+=v["confirmed_states"]; p+=v["promised_states"]; a+=v["kj1_violations"]; b+=v["kj1t_violations"]; pa+=v["promised_kj1_violations"]; pb+=v["promised_kj1t_violations"]} END{print "states=" s, "confirmed_states=" c, "promised_states=" p, "kj1_violations=" a, "kj1t_violations=" b, "promised_kj1_violations=" pa, "promised_kj1t_violations=" pb}'
states=11228080 confirmed_states=6380140 promised_states=6919344 kj1_violations=587599 kj1t_violations=587599 promised_kj1_violations=0 promised_kj1t_violations=0
```

两个根环形状计数（`name=root_choice_fault_states` 行的 `same_txg_different_instance_roots` 与 `higher_instance_lower_txg_roots`，按域加总，一条命令原样输出）：

```
grep 'name=root_choice_fault_states' e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out | awk '{d=""; for(i=1;i<=NF;i++){split($i,kv,"="); v[kv[1]]=kv[2]; if(kv[1]=="domain")d=kv[2]}; S[d]+=v["same_txg_different_instance_roots"]; H[d]+=v["higher_instance_lower_txg_roots"]} END{for(d in S) print d, "same_txg=" S[d], "higher_instance_lower_txg=" H[d]}' | sort
KF3 same_txg=0 higher_instance_lower_txg=0
KF9 same_txg=0 higher_instance_lower_txg=0
KF same_txg=0 higher_instance_lower_txg=0
KFW same_txg=0 higher_instance_lower_txg=0
KG same_txg=0 higher_instance_lower_txg=129420
```

实现与条款的理想恢复在每个格上同数（KJ1、KJ1T 每行带条款理想恢复在这一格的违例数 `ideal_clause_violations`；一条命令原样输出）：

```
grep 'name=violations' e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out | grep -E 'criterion=KJ1T? ' | awk '{v=0;i2=0; for(i=1;i<=NF;i++){if($i~/^violations=/)v=substr($i,12); if($i~/^ideal_clause_violations=/)i2=substr($i,25)}; n++; if(v!=i2) d++} END{print "rows=" n, "rows_where_the_implementation_differs_from_the_ideal_clause=" d+0}'
rows=980 rows_where_the_implementation_differs_from_the_ideal_clause=0
```

每类结果行的条数（`grep -o '^E7RESULT name=[a-z_0-9]*' <产物> | sort | uniq -c`，原样输出）：`1 name=c331_proof`、`1 name=config`、`448 name=control`、`1 name=done`、`49 name=fake_outcomes`、`2 name=in_flight_limit`、`195 name=mount_two_fault`、`98 name=promised`、`49 name=root_choice_fault_states`、`1 name=root_choice_states`、`1 name=self_check`、`4 name=self_proof`、`7 name=verdict`、`2452 name=violations`，合计 3 309，等于 `name=done emitted=3309`。49 个格 = KF 8 + KF3 12 + KF9 6 + KFW 15 + KG 8；`name=mount_two_fault` 195 行 = 四个有 M = 2 的格的域（KF 4、KF3 4、KF9 3、KG 4，共 15 个格）× 13 项故障；`name=root_choice_states` 1 行与 `name=violations` 里 KA 的两行是 KA 的（r3 的写法）。

**判别力与作废条款**（都在这一份产物里）：阳性对照 448 个读数、`controls_not_red=0`、`void_conditions=0`、`halt_conditions=0`；`name=control` 按判据：`1 criterion=KJ0`、`147 criterion=KJ1`、`147 criterion=KJ1T`、`104 criterion=KJ2`、`49 criterion=KJ3`（每个格 KF1（所选根取读视图里键最小的根）、KF2（落到最后一次已确认那一版的父上）、KF4（落到根环真相里不在已确认那条链上的根里键最大的那条）三种假结局各数各的判据，读数都对着登记第七节乙类的钉值；KJ0 在 KF M = 1、U = 0 指定的一态上用假的 panic 结局走同一段 KJ0 计数，读数 1；KA 上本地写的六种择根序的 KJ2 读数 0、36、18、18、52、0 对着钉值）。登记第八节判别力自证四条与 C331（择根倒挂压过已确认的写） 自证全 `passed=true`（产物第 3303 至 3307 行）。每个格的状态数、历史数、被拒与走了重读那一遍的挂载数、带撕裂系统配置槽的态数、条款理想恢复丢的三类、本地择根序读数和 M = 2 的 13 项故障表，装置里 `assert_eq!` 对着登记第七节乙类的表（设计员另写的 python 生成器 `anchors_r4.py` 的输出，sha256 `5a36b72237f84c52e6901ae5affcfc5431fd4c8b123f5b9e476907b749ba1fc7`）逐格钉死，13 856 755 态与登记 5.4 写的合计相同。

**几何敏感性与轨迹**（第八节的旋钮，各取方向相反的点）：条款的两道保护开（KF）对关（KG）：承诺内 KJ1、KJ1T 违例 0 对 10 368、20 628，TQ8 的 648（只在关的一边出）；读故障时机 T1 对 TP：M = 2 的故障表里 SN0、SN1、SB、RNSB 四项的 TP 在 KF 里历史全被拒（每项 1 238，加总命令是产物里 `self_proof` 第一行）、KG 里 0 被拒，SN0、SN1、SB 三项的 T1（从这次挂载里第一次读那个落点起读不出、重读之前恢复，登记修订 4）走第二遍重读各 28 466，没有一个态因为这个旋钮在承诺内翻面；被判时故障：无、RN、D0、D1 四项在四个合法域都是 0 违例，只有 RI 非 0；挂载次数 2 对 3（KF 对 KF3，k 取 {1, 2} 对 {1}）：承诺内两条都是 0，不翻面；journal 环 64 槽（在飞上限 21）对 9 槽（上限 3）：真相里读视图最大根之上同实例记录条数的最大值最大是 6（`above_newest_root_max`），「上限截掉」的态数在四个合法域与 KG 都是 0（今天的实现没有被在飞上限咬到已确认的那一版；每个 KJ1、KJ1T 行的 `cut_by_the_limit_states` 字段合计 0），KF9 与 KF 的承诺内读数相同；每区 8 槽对 4 槽（根环回绕）：KFW 回绕后第 0 代根被盖掉，视图里一条根都没有的被判态 21 个（`choose_root` 交回空 21 个，不算 KJ0 违例），承诺内仍是 0；线程 6 对 3：两遍 `E7RESULT` 整份逐字节相同。这一族界内根环里同 txg 不同实例的两条根同在是 0，实例代号更高而 txg 更低的一对根同在在四个合法域是 0、在 KG 是 129 420 个被判态（这一族能让择根变体露出来的取样点只在拿掉保护的 KG 里）。

产物（`research/results/` 下 10 份；第一遍是装置还没做最后四处不动判据的改动时编的二进制跑的，第二遍是最后一次源码改动之后重编重跑的，两遍的 `E7RESULT` 整份逐字节相同）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out` | 3 309 | 第二遍：`--bounds segment1 --family root_choice_faults --profile r4`，6 线程；13 856 755 态，收尾 `name=done emitted=3309`；复跑登记行指它 |
| `e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-threads-second-2026-10-03-r2.out` | 3 309 | 第二遍同一条命令 3 线程；与 6 线程那一份整份逐字节相同 |
| `e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-timing-2026-10-03-r2.out`、`e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-threads-second-timing-2026-10-03-r2.out` | 各 50 | 两遍的 `E7TIMING` 行，不进逐字比对：整遍 `elapsed_seconds` 6 线程约 66、3 线程约 126（同机负载均值 9 至 13） |
| `e165-replay-bounded-enumeration-2026-10-03-r4-segment1-profile-check-2026-10-03-r2.out` | 21（含注释头） | 旧产物复得（r1 档三份、r2 档对抗一族、第一段 r3 档对抗一族与第二段 r3 档择根一族，六份各段首、段尾一行，整份 sha256 与登记第七节的值逐个相同）、这一族两遍线程比、与第一遍的比 |
| `e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults.out`、`…-r4-segment1-root-choice-faults-threads-second.out`、`…-r4-segment1-root-choice-faults-timing.out`、`…-r4-segment1-root-choice-faults-threads-second-timing.out`、`e165-replay-bounded-enumeration-2026-10-03-r4-segment1-profile-check.out` | 3 309、3 309、50、50、21 | 第一遍的同样五份（6 与 3 线程、两份计时、旧产物复得）；`E7RESULT` 整份与第二遍的两份逐字节相同 |

装置与根：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 今天 sha256 `4ecfa61fda6e2c0764e748533bba37abcfbed86c4c7129dac67d7f80539ac031`（17 623 行；第一遍用的是 `0608bda031dc90f982ddb06150079518d274ac69d773765c3acf095f1972774f`，改前 `acd76c379479d5e9e23bc8a1fd6552c1af4f0108655c2a69c37a4ee3c2304a58`、13 577 行）；r1、r2、r3 三档的生成与判据代码一行没动，只改了文档注释并进已有的一行、档解析加 `"r4"`（当 r3 档，其余族与 r3 档相同）、`USAGE`、一条单测里「r4 必须报错」换成「r5 必须报错」、`main` 的分派。产物是根（开工那一刻主工作区的拷贝，246 个文件，清单 sha256 `6c7d4e235597fdc2bb2f7d5aed6958e2156097b102bca3855d11a6f1bf6d05b8`；盖上这份装置与加了 18 行的 `crates/mutations.tsv` 之后清单 sha256 `b5858a7aa94931c1804f4a996964aad99370360e5fc0c1658e8bc1747924febc`，段尾 `sha256sum -c` 246 个全 OK）上编的 release 二进制跑的，二进制 sha256 `7bee4d2bb01e750a61bad2503d57a6d620a7276b513f2b9ffee6ce0b89dd34b8`；被测的三步是根里那份 `recovery.rs`（`264664af…`）与 `crates/singlefs-replay-prefix/src/lib.rs`（`40cd99b5…`），`mount.rs`（`444610b4…`）与 `transaction.rs`（`67d6470f…`）只在跑产物之前核了哈希（登记修订 1 的 TH10），生成器没读它们。整遍峰值 RSS 约 1.66 GB（`/usr/bin/time -v`，两遍 1 658 480 与 1 660 708 KB；第一次起跑上限 1G 撞了包装的内存上限、退 250，输出不算），第二段的内存上限要取 4G 以上。

单测 10 条（新加，在 `mod root_choice_fault_tests`）与装置变异 18 行（`crates/mutations.tsv` 末尾，名字都以「E165 r4 第一段」起头）：**单测与 `crates/mutations.tsv` 整表没跑**（checker 档，归提交时 checker-tier-crates-mutation-replay）；为证判别力，草稿工作树里把 10 条单测的函数体当普通函数由一个只在草稿副本里编的参数调用：未变异的 10 条全过，18 个变异各套一份只跑那一行登记的「必须红」的测试，18 条都 panic（草稿验证，不是门禁的结果）。

**第一段的读数与登记第四节推的对账**：推 s1（今天的三步在四个合法域每个被判态上落在条款的理想恢复上，承诺内不丢）成立（KJ3 每格 0、承诺内 KJ1 与 KJ1T 每格 0）；推 s3（四个合法域上 KJ1 与 KJ1T 逐态同判）成立（`kj1_and_kj1t_differing_states=0`）；推 s4（KG 上被判时无故障的态里 KJ1T 非 0）成立，读数 648 与本地理想恢复同数，三步落点与理想恢复逐格同值（KJ3 为 0）；推 s5（旧产物逐字节复得）成立（六份，段首段尾各一遍）；推 s2、s6 要变体，第二段才读。第四节写的「条款理想恢复丢的被判态全在『不罩』」：KF 129 321、KF3 437 462、KF9 20 772、KFW 44 全部是被判时 RI，与产物逐格同数。

#### 第 4 次重跑第二段（Q2：第 3 次没红的三个择根变体在新一族上红不红，2026-10-03）

登记 `research/prompts/e165-r4-prereg.md` 修订 3（写在第二段产物之后：对账装置与驱动的终版 sha256、`crates/mutations.tsv` 的行号，补记一次内存撞顶改上限补跑；判据一个字没动）。装置一个字没动（sha256 仍是 `4ecfa61fda6e2c0764e748533bba37abcfbed86c4c7129dac67d7f80539ac031`，17 623 行）。驱动 `research/scripts/e165-replay-mutation-kill-bounds.sh` 加 `--profile r4` 模式，由主 agent 装进仓（sha256 `3428a78ee8d41be830905e6595e3212cfa36acc49805adb45deab9f16cd06c56`，3 365 行；不给 `--profile` 仍是 r2 那一套、`--profile r3` 仍是 r3 那一套）。驱动 `--selftest` 84 项全过、五个 r4 弄坏开关各红，是主 agent 交办这一段时写的读数，没有入库的输出文件，这一段没重跑。

**做法**：在第一段根（246 个文件）上给六个变体各建一份副本、各冷编一次（按变异名在根里的 `crates/mutations.tsv` 逐字找，每个恰 1 行、原文恰命中 1 次、命中处落在 `choose_root_slot`（`crates/singlefs-replay-prefix/src/lib.rs`）或 `choose_root`（`recovery.rs`）里）；基线与每个变体都调装置 `--profile r4 --bounds segment1 --family root_choice_faults`（每个二进制 13 856 755 态、3 309 行），逐 (域, M, U, 被判时故障, 历史标签) 子格比 KJ0、KJ1、KJ1T、KJ2、KJ3 的违例数与 `root_choice_fault_states` 行。开头基线先与第一段产物逐行比（登记 TV7），收尾再编一次基线、与开头逐字相同才写收尾行（TV9）。**跑法**由主 agent 起，两趟：第一趟 `--jobs 2 --threads 3`、内存上限 4G，tsv-1823 的二进制被内存包装拒（退出码 250，那一次输出不算结果，TV8），其余五个变体与开头收尾基线得出结果，没写收尾三行；第二趟补跑 `--jobs 1 --threads 6`、内存上限 8G，只补 tsv-1823（别的五个取第一趟存下的单项结果）。所以产物里 v1824（tsv-1823）一行是 6 线程跑的、别的五行是 3 线程跑的；计数与线程数无关由第一段两遍（6 与 3 线程）整份逐字节相同保住，不是这一段新量的。两趟驱动日志原样入库（`e165-replay-bounded-enumeration-2026-10-03-r4-segment2-driver-log.out`，第 2、8、17 行写着撞顶退 250）。

名单（登记名是第 3 次重跑的编号当名字用，根里与今天的主工作区里都在 `crates/mutations.tsv` 第 1821 至 1826 行，行号只当索引）：

| 变体（登记名；`crates/mutations.tsv` 的行；驱动里的项） | 改的是 | 在 Q2 里 |
|---|---|---|
| v1823（第 1822 行；tsv-1822）择根核比较不看实例代号 | txg 平局时停在读根环次序里前面那条 | Q2 的「比较不看实例」 |
| v1825（第 1824 行；tsv-1824）择根核 txg 平局时实例代号低者赢 | 平局取低 | Q2 的「平局取低」 |
| v1826（第 1825 行；tsv-1825）择根核以实例代号为主、txg 只破平局 | 比较的主次序反过来 | Q2 的「实例代号为主」 |
| v1824（第 1823 行；tsv-1823）择根核取第一个有效者 | 择了一个自证过的槽就不再换 | 附带（第 3 次重跑已红） |
| v1822（第 1821 行；tsv-1821）择根核平局取后者赢 | 择新键完全相同的两个槽取后一个 | 附带 |
| v1827（第 1826 行；tsv-1826）择根胶水摊择新键时不带实例代号 | 核看到的实例代号都是 0 | 附带 |

每个变体一行的读数，一条命令从产物里数（`research/results/` 下，工作目录是那里）原样输出：

```
grep '^E7RESULT name=kill_bound ' e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out | grep -v 'item=baseline' | awk '{for(i=1;i<=NF;i++){if($i~/^(item|variant|outcome|red_legal_domains|differs_on|ka_pin|ka_inferred|kg_pin|kg_mismatched_cells|states_row_differences)=/)printf "%s ",$i}; print ""}'
item=tsv-1821 outcome=not_red_within_bounds variant=v1822 red_legal_domains=none differs_on=none ka_pin=not_pinned ka_inferred=match kg_pin=match kg_mismatched_cells=none states_row_differences=none 
item=tsv-1822 outcome=not_red_within_bounds variant=v1823 red_legal_domains=none differs_on=KA ka_pin=match ka_inferred=none kg_pin=match kg_mismatched_cells=none states_row_differences=none 
item=tsv-1823 outcome=red variant=v1824 red_legal_domains=KF,KF3,KF9,KFW differs_on=not_applicable ka_pin=match ka_inferred=none kg_pin=match kg_mismatched_cells=none states_row_differences=KF=M1U0:replay_applied;KF3=M1U0:replay_applied;KF9=M1U0:replay_applied;KFW=M1U0:replay_applied;KG=M1U0:replay_applied 
item=tsv-1824 outcome=not_red_within_bounds variant=v1825 red_legal_domains=none differs_on=KA ka_pin=match ka_inferred=none kg_pin=match kg_mismatched_cells=none states_row_differences=none 
item=tsv-1825 outcome=not_red_within_bounds variant=v1826 red_legal_domains=none differs_on=KG+KA ka_pin=match ka_inferred=none kg_pin=match kg_mismatched_cells=none states_row_differences=KG=M2U0:replay_applied 
item=tsv-1826 outcome=not_red_within_bounds variant=v1827 red_legal_domains=none differs_on=KA ka_pin=not_pinned ka_inferred=match kg_pin=match kg_mismatched_cells=none states_row_differences=none 
```

`states_row_differences` 是 `root_choice_fault_states` 行（生成器一侧记下的状态与根环形状计数加三步落点的几样计数）与基线不同的格与字段（每个域写第一个不同的）：v1824 在每个域的 M1U0 格上 `replay_applied`（所选根之上施加了几条记录）与基线不同，v1826 在 KG 的 M2U0 格上不同，别的四个变体一格都不同。

每个变体在每个判据上与基线不同的子格数（一个子格 = 一个 (域, M, U, 被判时故障, 历史标签) 格，四个合法域 KF 80、KF3 120、KF9 60、KFW 150，共 410 个，KG 80 个；只列 KJ1、KJ1T、KJ2，KJ0 与 KJ3 同样列在产物里），一条命令原样输出：

```
grep '^E7RESULT name=kill_bound ' e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out | grep -v 'item=baseline' | awk '{for(i=1;i<=NF;i++){f=$i; p=index(f,"="); k=substr(f,1,p-1); val=substr(f,p+1); if(k=="variant")printf "%s ",f; if(k=="differing_subcells"||k=="differing_subcells_kg"){n=split(val,a,";"); for(j=1;j<=n;j++) if(a[j]~/^KJ(1|1T|2):/) printf "%s%s ", (k=="differing_subcells_kg"?"kg-":""), a[j]}}; print ""}'
variant=v1822 KJ1:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ1T:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ2:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 kg-KJ1:KG=0of80 kg-KJ1T:KG=0of80 kg-KJ2:KG=0of80 
variant=v1823 KJ1:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ1T:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ2:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 kg-KJ1:KG=0of80 kg-KJ1T:KG=0of80 kg-KJ2:KG=0of80 
variant=v1824 KJ1:KF=26of80,KF3=46of120,KF9=13of60,KFW=27of150 KJ1T:KF=26of80,KF3=46of120,KF9=13of60,KFW=27of150 KJ2:KF=52of80,KF3=92of120,KF9=40of60,KFW=33of150 kg-KJ1:KG=26of80 kg-KJ1T:KG=26of80 kg-KJ2:KG=52of80 
variant=v1825 KJ1:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ1T:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ2:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 kg-KJ1:KG=0of80 kg-KJ1T:KG=0of80 kg-KJ2:KG=0of80 
variant=v1826 KJ1:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ1T:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ2:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 kg-KJ1:KG=8of80 kg-KJ1T:KG=8of80 kg-KJ2:KG=12of80 
variant=v1827 KJ1:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ1T:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 KJ2:KF=0of80,KF3=0of120,KF9=0of60,KFW=0of150 kg-KJ1:KG=0of80 kg-KJ1T:KG=0of80 kg-KJ2:KG=0of80
```

每个变体在 KA 上的 KJ2 违例数（基线 0）：

```
grep '^E7RESULT name=kill_bound ' e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out | grep -v baseline | grep -o 'item=tsv-18[0-9]*\|ka_kj2=[0-9]*' | paste - -
item=tsv-1821	ka_kj2=0
item=tsv-1822	ka_kj2=18
item=tsv-1823	ka_kj2=52
item=tsv-1824	ka_kj2=36
item=tsv-1825	ka_kj2=18
item=tsv-1826	ka_kj2=18
```

v1826 在 KG 上与基线不同的格（一格五个数是 KJ0 / KJ1 / KJ1T / KJ2 / KJ3 的违例数；列出与「0/0/0/0/0」不同的 KG 与 KA 格），与基线 KG 同一格的 KJ1 / KJ1T / KJ2（第一段产物，一条命令加总）并排：

```
grep '^E7RESULT name=kill_bound item=tsv-1825 ' e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out | grep -o ' cells=[^ ]*' | tr ',' '\n' | grep -v '=0/0/0/0/0$' | grep 'KG\|KA'
KG.M1U2=0/117/117/0/0
KG.M1U3=0/351/351/0/0
KG.M2U0=0/0/0/36/90
KG.M2U1=0/0/0/630/1008
KG.M2U2=0/58824/58824/7236/9558
KG.M2U3=0/309609/346005/49680/61398
KA.M0U0=1/-/-/18/-
grep 'name=violations domain=KG' e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out | grep -E 'criterion=KJ(1|1T|2) ' | awk '{m="";u="";c="";v=0; for(i=1;i<=NF;i++){split($i,kv,"="); if(kv[1]=="mounts")m=kv[2]; if(kv[1]=="user_publishes")u=kv[2]; if(kv[1]=="criterion")c=kv[2]; if(kv[1]=="violations")v=kv[2]}; s["M"m"U"u" "c]+=v} END{for(k in s) print k, s[k]}' | sort | grep -E '^M2U[0-3] |^M1U[23] '
M1U2 KJ1 117
M1U2 KJ1T 117
M1U2 KJ2 0
M1U3 KJ1 351
M1U3 KJ1T 351
M1U3 KJ2 0
M2U0 KJ1 0
M2U0 KJ1T 0
M2U0 KJ2 0
M2U1 KJ1 0
M2U1 KJ1T 0
M2U1 KJ2 0
M2U2 KJ1 53154
M2U2 KJ1T 53154
M2U2 KJ2 0
M2U3 KJ1 265923
M2U3 KJ1T 284607
M2U3 KJ2 0
```

v1826 与基线在 KG 的 M1U2、M1U3 两格相同（117、351），M2U0 起四格不同：KJ2 违例由基线的 0 变成 36、630、7 236、49 680，M2U2 与 M2U3 的 KJ1、KJ1T 也变多（53 154 变 58 824、265 923 变 309 609、284 607 变 346 005）；这些读数等于登记第七节「本地择根序」表里 v1826 那一列，驱动对这一列逐格核过（`kg_pin=match`，六个变体都 match）。

收尾行与判决行（产物第 9、10 行，整行）与每类结果行的条数（`grep -o '^E7RESULT name=[a-z_0-9]*' <产物> | sort | uniq -c`，原样输出）：

```
E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows=3309 root_recovery_sha256_at_start=264664af7a694fbda1295ac450c3d9c2198336d002b3c288b0f797731350e625 root_recovery_sha256_at_end=264664af7a694fbda1295ac450c3d9c2198336d002b3c288b0f797731350e625 root_lib_sha256_at_start=40cd99b58242a5e522a9e302fa83ceaed17cdd5c2ea8e8811e0e223a4649f08c root_lib_sha256_at_end=40cd99b58242a5e522a9e302fa83ceaed17cdd5c2ea8e8811e0e223a4649f08c copies_restored=true
E7RESULT name=verdict scope=segment2 variants=7 variants_excluding_baseline=6 red=1 red_only_by_kj0_or_kj3=0 not_red_within_bounds=5 invalid=0 run_did_not_finish=0 not_compared=0 ka_pin_mismatches=0 kg_pin_mismatches=0 baseline_equals_segment1_product=true
      1 E7RESULT name=config
      1 E7RESULT name=done
      7 E7RESULT name=kill_bound
      1 E7RESULT name=restored
      1 E7RESULT name=verdict
```

合计 11，等于 `name=done emitted=11`；`grep -c 'halt_conditions=0 void_conditions=0 run_exit=0 run_finished=true cells_without_rows=0 abort=none copy_restored=true' <产物>` 得 6（六个变体行每行都带）；基线行（第 2 行）带 `result_rows=3309`、`run_exit=0`、`rows_identical_to_segment1_product=true`、`kg_today_pin=match`。`red=1` 是 v1824；`not_red_within_bounds=5` 是 v1822、v1823、v1825、v1826、v1827（Q2 的三个加附带的两个）；`red_only_by_kj0_or_kj3=0`；`invalid=0 run_did_not_finish=0 not_compared=0`（六个变体都编过、跑到界、选行条件都满足）；`ka_pin_mismatches=0`、`kg_pin_mismatches=0`（KA 钉值 18、52、36、18 与 KG 每格的本地择根序读数都对上）；`baseline_equals_segment1_product=true`。

**没红的几个按 `.claude/rules/mutation-sampling.md` 三分**（问句：换一个取样点，它还同值吗），依据都是产物里各自的 `differs_on` 与 `differing_subcells`：

- **v1823（比较不看实例）、v1825（平局取低）：取样点不敏感（第三类）**，不是等价变异、不是真盲区。① 变体有效、改到了择根序：KA 上 KJ2 读 18、36，等于登记钉值（`ka_pin=match`，没有 `invalid`）；② 换一个取样点就不同值：KA（根环里同 txg 不同实例的根可以同在）上 0 变 18、36；③ 新一族里这种根环一个都造不出：`same_txg_different_instance_roots` 在四个合法域与 KG 合计都是 0（第一段产物，命令见上），所以在四个合法域 410 个子格与 KG 80 个子格上，`differing_subcells` 与 `differing_subcells_kg` 每条判据都是 0（`differs_on=KA`）。读法是登记推 s2：根槽落点只由 txg 定（I-7.17（根槽位置等于 txg 算出的位置）），读故障不挪根的位置，同 txg 不同实例的两条根要落在同一个槽里，内存盘上造不出（推的，没逐态量；量到的是两个计数为 0）。
- **v1826（实例代号为主）：取样点不敏感（第三类），取样点只在拿掉条款保护的历史里有**。四个合法域 410 个子格每条判据都是 0；KA 上 18（`ka_pin=match`）；KG 上 KJ1、KJ1T 各 8 个、KJ2 12 个子格（共 80 个）与基线不同（`differs_on=KG+KA`）。取样点就是「实例代号更高而 txg 更低的两条根同在」：合法域每格 0，KG 合计 129 420 个被判态（第一段）。KG 不是合法历史（拿掉了闸与写行 txg 看记录那两道保护，登记第五节），按登记它的不同不算 Q2 的「红」。
- **v1827（胶水摊键不带实例代号）：取样点不敏感（第三类）**，同 v1823：核看到的实例代号都是 0，等于「比较不看实例」；KA 上 KJ2 读 18（登记里推的值，`ka_pin=not_pinned ka_inferred=match`），合法域与 KG 子格全 0（`differs_on=KA`）。
- **v1822（平局取后者赢）：这一族与 KA 上同值，三分里记取样点不敏感（推的）、不记等价变异**。四个合法域、KG 子格全 0，KA 上 KJ2 读 0（`differs_on=none`、`ka_pin=not_pinned ka_inferred=match`）：六个域都没有一个输入让它与基线不同。它只在「读根环次序里有两个择新键（txg、实例代号）完全相同、而内容不同的自证过的槽」时才与原式择得不同；合法历史只有第 0 代根的三份是同键同内容（这一族与 KA 都没造同键不同内容的两个槽），所以界内没有取样点，不是「所有输入上同值」被证出来了。`crates/mutations.tsv` 第 1821 行点名的单测 `of_two_slots_with_the_same_choice_key_the_earlier_slot_is_chosen` 名字写的是这一格，单测体这一段没读、没跑（checker 档）。要把它从「推的」变成量到的，得加一格同键不同内容的两个自证过的槽（要新一轮）。

**红的一个：v1824（取第一个有效者）**，四个合法域都红（`red_legal_domains=KF,KF3,KF9,KFW`）：KJ2 首红在每个域的 M1U0，KJ1 与 KJ1T 首红在 M1U2；`differing_subcells` 在 KJ2 上 KF 52 of 80、KF3 92 of 120、KF9 40 of 60、KFW 33 of 150，在 KJ1 与 KJ1T 上 26、46、13、27（两条判据逐域同数），KJ0 为 0；KA 52 对上钉值，KG 上 KJ1、KJ1T 26 of 80、KJ2 52 of 80。首红格的读数与登记 9.1 表的预期一致（KF M1U0 的 KJ2 8 态、M1U2 的 KJ1 与 KJ1T 468 态，产物第 5 行 `cells=KF.M1U0=0/0/0/8/8`、`KF.M1U2=0/468/468/351/351`）。KFW 的 U11 至 U14 四个格上 v1824 的 `cells` 是 `0/0/0/0/0`，与基线相同（第 3 次重跑里 KW 域同样的四格不红）：

```
grep '^E7RESULT name=kill_bound item=tsv-1823 ' e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out | grep -o 'first_red_by_criterion=[^ ]*'
first_red_by_criterion=KJ1:KF=M1U2/none/inside,KF3=M1U2/none/inside,KF9=M1U2/none/inside,KFW=M1U2/none/inside;KJ1T:KF=M1U2/none/inside,KF3=M1U2/none/inside,KF9=M1U2/none/inside,KFW=M1U2/none/inside;KJ2:KF=M1U0/none/inside,KF3=M1U0/none/inside,KF9=M1U0/none/inside,KFW=M1U0/none/inside;KJ0:KF=none,KF3=none,KF9=none,KFW=none;KJ3:KF=M1U0/none/inside,KF3=M1U0/none/inside,KF9=M1U0/none/inside,KFW=M1U0/none/inside
grep '^E7RESULT name=kill_bound item=tsv-1823 ' e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out | grep -o ' cells=[^ ]*' | tr ',' '\n' | grep '^KFW.M1U1[0-4]'
KFW.M1U10=0/16/16/12/12
KFW.M1U11=0/0/0/0/0
KFW.M1U12=0/0/0/0/0
KFW.M1U13=0/0/0/0/0
KFW.M1U14=0/0/0/0/0
```

（首红格的写法是 `M<挂载数>U<用户发布数>/<被判时故障>/<历史标签>`，被判时故障 `none` 是无故障。）

**第二段的读数与登记第四节推的、第十节失败条款的对账**：推 s2（v1823、v1825、v1826 在四个合法域界内不红，三分第三类；v1826 在 KG 上有取样点、v1823 与 v1825 只在 KA 上有）成立：三个在 410 个合法子格上都是 0，KG 上 v1826 非 0、另两个 0；推 s6（KA 上 KJ2 读 18、52、36、18，KG 每格 KJ1、KJ1T、KJ2 等于第七节「本地择根序」表那一列）成立（`ka_pin_mismatches=0`、`kg_pin_mismatches=0`）；v1822 在 KG 与四个合法域不变、KA 读 0，v1827 取 v1823 那一列、KA 读 18，也都与登记第四节、9.1 一致。失败条款：TF5（Q2「仍有不红的」）触发，三个各记第三类、各写 KG 上有没有不同的格（v1826 有、另两个没有）；TF6（推 s2 被推翻）没触发——三个合法域都不红，同时两个根环形状计数全 0；TF7（阳性对照没对上）没触发；TV3 里「`kill_bound` 7 行」成立（产物 `variants=7`、`name=kill_bound` 7 行；`--list-items` 那一格是驱动自证的项，这一段没重跑）；TV7（第二段基线与第一段产物不同）没触发（基线行 `rows_identical_to_segment1_product=true`）；TV8 在第一趟触发过一次（tsv-1823 退 250，输出不算），补跑后 tsv-1823 的结果取第二趟的。

**几何敏感性**（登记第八节的旋钮）：

| 旋钮 | 取样点与读数 | 判决 |
|---|---|---|
| 条款的两道保护（闸与写行 txg 看记录） | 开（KF 等四个合法域）：v1823、v1825、v1826 每格 0；关（KG）：v1826 KJ1、KJ1T 8 个、KJ2 12 个子格非 0，v1823、v1825 仍是 0 | v1826 的判决随保护翻面（合法域不红、KG 红），KG 不是合法历史、不算 Q2 的红，两点各写一个数；v1823、v1825 两点都不红 |
| journal 环槽数（在飞上限）、每区槽数与回绕、挂载次数、每次发布的记录条数 | KF（64 槽、S = 8、M ≤ 2）、KF3（M ≤ 3）、KF9（9 槽）、KFW（每区 4 槽、回绕）四个合法域：v1823、v1825、v1826 每格 0；v1824 四个合法域都红，首红的 (M, U) 格四个域相同（KJ2 在 M1U0，KJ1 在 M1U2），KFW 的 U11 至 U14 四格不红（同第 3 次重跑） | 这几个旋钮对 Q2 的判决不翻面：「仍有不红的」在每一点都一样；v1824 在 KFW 的后四格同基线 |
| 读故障的时机与落点、被判时故障 | 子格按被判时故障与历史标签分，四个合法域三个变体 410 个子格全 0；v1824 红的子格里被判时故障 `none` 的格也红（首红格都是 `none`） | 不翻面 |
| 线程数 | v1824 一行 6 线程、别的五行 3 线程；整份比对靠第一段两遍同数与基线逐行等于第一段产物 | 这一段没有另跑两遍整份比 |

两个根环形状计数（`name=root_choice_fault_states` 的 `same_txg_different_instance_roots`、`higher_instance_lower_txg_roots`）用的是第一段产物（命令与原样输出在上面「第 4 次重跑」一节第一段部分）；第二段每个变体行的 `states_row_differences` 说明变体没改生成器一侧的这些计数（v1826 在 KG M2U0 格的 `replay_applied` 变，是三步落点的计数，不是生成器的）。

产物（`research/results/` 下，第二段 2 份）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out` | 11 | `--profile r4` 的驱动整轮：配置 1、基线 1、变体 6、还原 1、判决 1、完成 1；sha256 `8f842e6960356e653fac1204f8a23c359f31ec1b0cd1903cdb6ff172998d3735`；复跑登记行指它 |
| `e165-replay-bounded-enumeration-2026-10-03-r4-segment2-driver-log.out` | 28 | 两趟驱动 stdout 日志原样（第一趟 4G 撞顶、第二趟 8G 补跑），sha256 `1528a001694e5492c816ce263d521a3834c19421633cf9d77e2ed67091189a14`；不进逐字比对、不是复跑对象 |

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:913-1204`（整段抄，未转述）**

````markdown
### 第 5 次重跑（记录格式按根环那条线 7-2 改过之后；系统配置读错取 R2 之后的带故障一族，2026-10-04）

问题单 `research/prompts/m3-r1-batch9-reruns-questions.md` 第 3 行（登记切成三格：3-甲 旧档判定、3-乙 变异杀伤界、3-丙 重新登记的判定）与追加问题单 `research/prompts/m3-formal-e165-r5-questions.md` 第 1 行（登记修订 1 加的第五段，两步：5-甲 答 Q1（R2）、5-乙 答 Q2（R2）），重跑登记 `research/prompts/e165-r5-prereg.md`（含修订 1 与主 agent 的四次认定）。状态：**五格都已够判**；第四段（PQ13，J8 在真代码上的阳性对照）登记写明附带、够判后不跑，没跑。装置单测与装置变异未跑（checker 档，提交时由主 agent 跑 checker-tier-crates-mutation-replay）。

这一次的输入变化全出自根环那条线第 7 批 7-2：记录头里的新根段改成 364 字节（后三项是实例表指针、分配记录树根指针、卸载记号），施加一条记录改成「把所选根的这七个字段换成记录里的」（D23（journal 的角色与格式） 已定项 15），系统配置读错改取 R2（D22（单元原子性怎么合成） 已定项 8 射程「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过」）。被测的三步（`choose_root`、`scan_journal`、`replay_journal`）一个字没改。所有段都在同一份根上编、跑：开工那一刻主工作区 `Cargo.toml`、`Cargo.lock`、`.cargo/`、`crates/` 的拷贝 `/tmp/claude-1000/e165-r5-segment1/root`（291 个文件，清单 `root-manifest-at-start.sha256` 的 sha256 `95237ecfd9cb3722aa3fddd95b8cfdf3209fc42cbad4dfc85aec276cacc76646`）；第三段与 5-甲 在它的拷贝上盖改过的装置与 `crates/mutations.tsv`（第三段装置 `f72fc1a4…`，清单 sha256 `eb6d391e4e605ed592c431d4eed9f63088c2ce07e2314e39f3e7fed967c5bba1`；5-甲 装置 `8006b251c1011d5f39233ecdd3410cec6bad895bd3e6f49d981afeb3a38bd482`、`crates/mutations.tsv` `c9a03eb4af85a8e4c7206dfa3609593390516b80ca52be7090aced5696e84046`，清单 sha256 `47c6d3494bf6688bb6b5a8e88df87c32a7ffee8ca253cc56128c523a3a47a950`）。

| 格 | 结论 | 依据（产物，原样数字） |
|---|---|---|
| 3-甲：r1–r4 四档一个字不改，在 7-2 之后的根上照原参数重跑 16 份比对对象，逐身份比判定字段 | **判定变了，只变在 J7 字段**（登记 PF2）：16 份里 11 份（r2 档 10 份、r3 档对抗一族 1 份）有 J7 判定由 0 变非 0，J7 之外每份 0 处；变的只有 J7a 分配记录树根与 J7a 实例表指针两格（实例表指针只在 C、C6、MC 三个域），J7a fsid、J7b、J7c 一格没变；r1 档 3 份、r3 档择根、r4 档带故障共 5 份与入库产物整份逐字节相同。J7 判的是被 D23（journal 的角色与格式） 已定项 15 今天的原句取代的施加规则（「投影之外抄所选根」），今天原句的判定由 3-丙 的 J8 给 | `research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-comparison.out` 的 `name=pq1` 16 行与 `name=extended_pq1` 16 行（整行抄在下方代码块） |
| 3-乙：驱动三个模式（r2：23 个名字合成 21 种替换加 Z1–Z5；r3、r4：6 个择根变体）加 `--pins r5` 在同一份根上重跑，逐变体比首红位置 | **杀伤界与第 2、3、4 次相同**：r2 模式比了 28 行、r3 与 r4 模式各比了 6 行，`differing_variants=0 one_sided=0`；r2 模式 23 个变体首红那个 N 上红的判据名单多了 J7a 两格（登记 PQ14，附带、不算翻面），首红的域、N、`outcome` 都没变 | `research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-comparison.out` 第 32、42、52 行与第 54 至 57 行 |
| 3-丙：r5 档（`--profile r5`）上记录后三项照 D23（journal 的角色与格式） 已定项 15 生成，J8 五格判今天的施加，对 r2 登记那几份产物的 J7 旧格 | **判定与旧格相同**：三族、两种卸载记号奇偶、每个 (域, N) 共 830 格，J8 每格 0，与 J7 旧格「0 / 非 0」逐格相同（`differing=0 one_sided=0`）；J8 无出处的态 0；C8a–C8d 的 38 个对照读数全等于钉值 | `research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment3-cell-correspondence.out` 第 1072 行与五行 `name=pq9_files` |
| 5-甲（Q1（R2））：带读故障的择根与重放合判一族的生成器闸改取 R2（`--profile r4 --family root_choice_faults --gate r2`），今天的三步在 KF、KF3、KF9、KFW 上丢不丢已确认的发布 | **承诺内不丢，与第 4 次同**：34 814 150 个被判态，四个合法域承诺内 KJ1、KJ1T 违例每格 0（承诺内被判态 KF 1 358 216、KF3 18 736 200、KF9 348 544、KFW 320）；与第 4 次那份同域同判定逐个相同，`name=promised` 98 行判定变了 0 行；第一段那份取甲重跑与第 4 次那份逐字节相同，所以没有要归因的翻面（登记 PF13） | `research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5-comparison-with-r4-q1.out` 第 2 至 10 行、第 15 行 |
| 5-乙（Q2（R2））：驱动 r4 模式加 `--pins r5 --gate r2`，第 3 次没红的三个择根变体在 R2 那一族上红不红 | **仍有不红的，与第 4 次同**：v1823（比较不看实例）、v1825（平局取低）、v1826（实例代号为主）在四个合法域 410 个子格上与基线逐子格相同，界内不红；附带的 v1824 四个合法域都红，v1822、v1827 不红；六个变体的 `outcome`、`red_legal_domains` 与第 4 次逐个相同，连首红位置也相同；四个合法域「同 txg 不同实例同在」「实例代号更高而 txg 更低同在」两个计数每格 0，三个不红的仍判取样点不敏感（第三类） | `research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5-comparison-with-r4-q2.out` 第 10、20、30、40 行；`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out` 第 3 至 8、10 行 |

#### 第 5 次重跑第一段（3-甲：旧档原样重跑，2026-10-04）

开跑前照登记停机条款 PH1 在主工作区上逐条对过（`crates/singlefs-format/src/lib.rs` 三个常量仍是 364、487、64，`JournalRecord` 十七个字段，28 个函数的区间文本哈希与登记那一份逐字节相同，施加那一段 fsid 取所选根、其余九项取施加末条那一份记录，装置、驱动、自造表的 sha256 与登记相同，选行全过），七条全过才拷根。装置在根上编一次（二进制 sha256 `2b71ad72ae7980f9dc8fe3ef165e7781b69fcf04fdc28d2ecdea63fadedf3406`），照登记第十一节比对对象表每一行的参数各跑一遍，7 线程（r4 带故障一族另跑 3 线程一遍）。

PQ1（登记读法）原样输出（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-comparison.out` 第 6、9、12、15、95、112、198、221、248、328、345、372、382、403、420、423 行，整行）：

```
name=pq1 label=row1:r1-legal-family verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row2:r1-light-legal-family verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row3:r1-adversarial-family verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row4:r2-legal-family verdict_changed_j7=74 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row5:r2-adversarial-family verdict_changed_j7=13 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row6:r2-odd-marker-legal-family verdict_changed_j7=74 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=3 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row7:r2-odd-marker-adversarial-family verdict_changed_j7=13 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=3 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row8:r2-multi-disk verdict_changed_j7=20 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=2 duplicate_identities_new=2
name=pq1 label=row9:r2-table-both-legal-family verdict_changed_j7=74 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row10:r2-table-both-adversarial-family verdict_changed_j7=13 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row11:r2-table-both-multi-disk verdict_changed_j7=20 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=2 duplicate_identities_new=2
name=pq1 label=row12:r2-reversed-disk-order-legal-family verdict_changed_j7=5 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row13:r2-reversed-disk-order-multi-disk verdict_changed_j7=14 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=1 duplicate_identities_new=1
name=pq1 label=row14:r3-adversarial-family verdict_changed_j7=13 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=pq1 label=row15:r3-root-choice verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=12 duplicate_identities_new=12
name=pq1 label=row16:r4-root-choice-faults verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=261 duplicate_identities_new=261
```

**登记没写到的一格（停机条款 PH6 那一类，主 agent 2026-10-04 认定取严的那种读法）**：多盘一族（比对对象第 8、11、13 份）的 `name=verdict` 每个域另有 5 行按 `lagging_class`（`equal`、`lagging`、`crossed`、`first_device_behind`、`second_device_behind`）分，判定字段叫 `J7a_allocation_record_tree_root=` 这类、不以 `violations` 结尾，登记第一节 3-甲 的身份字段里没有 `lagging_class`。照登记字面（`name=pq1`），这 5 行与该域总行落成同一身份（`duplicate_identities` 2、2、1），J7a 在这几行的变化记进「只有数不同」；严的读法（`name=extended_pq1`）身份加 `lagging_class`、`mounts`、`user_publishes`，大写 J 开头的计数取 0 / 非 0，第 8、11、13 份的 J7 栏由 20、20、14 变成 30、30、19，别处栏仍全是 0。主 agent 认定取严的那种（`name=extended_pq1`）；两种读法 3-甲 的判决相同。严的读法原样输出（同一份文件第 427 至 499 行里的 16 行，整行）：

```
name=extended_pq1 label=row1:r1-legal-family verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row2:r1-light-legal-family verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row3:r1-adversarial-family verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row4:r2-legal-family verdict_changed_j7=74 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row5:r2-adversarial-family verdict_changed_j7=13 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row6:r2-odd-marker-legal-family verdict_changed_j7=74 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=3 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row7:r2-odd-marker-adversarial-family verdict_changed_j7=13 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=3 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row8:r2-multi-disk verdict_changed_j7=30 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row9:r2-table-both-legal-family verdict_changed_j7=74 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row10:r2-table-both-adversarial-family verdict_changed_j7=13 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row11:r2-table-both-multi-disk verdict_changed_j7=30 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row12:r2-reversed-disk-order-legal-family verdict_changed_j7=5 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row13:r2-reversed-disk-order-multi-disk verdict_changed_j7=19 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row14:r3-adversarial-family verdict_changed_j7=13 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row15:r3-root-choice verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=0 duplicate_identities_new=0
name=extended_pq1 label=row16:r4-root-choice-faults verdict_changed_j7=0 verdict_changed_elsewhere=0 one_sided_j7=0 one_sided_elsewhere=0 one_sided_exempt_rev4_self_checks=0 duplicate_identities_old=1 duplicate_identities_new=1
```

读法：变了的身份逐个列在同一份文件的 `name=pq1_changed` 行（342 行），全是 `old=0 new=<正整数>`，没有一处由非 0 变 0。第 6、7 份（两份反奇偶）各 3 行在豁免栏，是 r2 登记修订 4 挪成单测的三项自检（`j6qn_control_readings`、`j7_control_readings`、`odd_marker_parity_turns_the_unmount_marker_controls`），那两份旧产物是修订 4 之前跑的（比 r2 合法历史那份多这 3 行，`config` 行只差 `marker_parity=odd`），登记写明这三行缺了不算判定变。PQ4（附带）：每个 r2 档判据行里 J7a 分配记录树根的违例数都等于同一行的 `j7b_judged`（重建过的每一态都违例），J7a 实例表指针只在所选根随落盘变的 C（20 290）、C6（1 626）、MC（6 612）非 0，与登记推 s3 逐格对上。PQ3：r4 带故障一族 7 线程与 3 线程两遍整份逐字节相同（同一份文件第 531 行 `name=pq3 threads=7,3 identical=true`）。16 份重跑里没有一个 `red_as_required=false`，每份 `scope=run` 判决行 `controls_not_red=0`、`void_conditions=0`、`halt_conditions=0`。

第一段的产物（`research/results/` 下 35 份；比对对象的 sha256 与行数跟登记第十一节末尾的表逐行对过）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-light-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-adversarial-family.out` | 910、876、172 | r1 档三份重跑，与 `e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out` 整份逐字节相同 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-adversarial-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-odd-marker-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-odd-marker-adversarial-family.out` | 1 466、257、1 468、257 | r2 档合法历史、对抗两族与两份反奇偶（比对对象第 4 至 7 行，旧产物没有复跑行，参数取旧产物 `config` 行）；第二段 r2 模式的 V7 比对对象是前两份 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-multi-disk.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-adversarial-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-multi-disk.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-multi-disk.out` | 405、1 551、279、433、163、287 | r2 档多盘、`--table both` 三份、反盘次序两份；复跑登记行改指它们（主 agent 认定第 4 条），第 2 次重跑的旧产物留着、今天的装置在 J7a 两格上复不出 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-root-choice.out` | 380、379 | r3 档对抗一族（复跑登记行改指它）与择根一族（与 `e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out` 整份逐字节相同；第二段 r3 模式的 TV7 比对对象） |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r4-root-choice-faults.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-root-choice-faults-threads-second.out` | 3 309、3 309 | r4 档带故障一族（闸取甲）7 线程与 3 线程两遍，与 `e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out` 整份逐字节相同；第二段 r4 模式的 TV7 比对对象、5-甲 PF13 归因的取甲那一点 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-comparison.out` | 531 | 逐份 sha256、行数、PQ1（两种读法）、`pq1_changed` 342 行、PQ2、PQ3、PQ4、`keydiff` |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-light-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-odd-marker-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-odd-marker-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-multi-disk-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-multi-disk-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-multi-disk-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-root-choice-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r4-root-choice-faults-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment1-root-choice-faults-threads-second-timing.out` | 6 至 67 | 各份的 `E7TIMING` 行，不进逐字比对 |

#### 第 5 次重跑第二段（3-乙：变异杀伤界，2026-10-04）

驱动 `research/scripts/e165-replay-mutation-kill-bounds.sh` 加 `--pins r5`（三个模式各一套钉值：第一段根上 `recovery.rs` `4454f6d3…`、replay-prefix `lib.rs` `9966f42f…`、`crates/mutations.tsv` `bc2627f8…` 的 sha256 与函数区间，V7 / TV7 的比对对象换成第一段的重跑产物；不给 `--pins` 时原来三套一个字不变），跑的时候是 sha256 `164e04b1e72aa1f6acf4b663137fc88d1ff5dbb0a431ac2a9a893ae4b049c64e` 那一份（主 agent 装进仓之后 `--selftest` 99 项 0 项不过，两个弄坏开关各红；这两个读数没有入库的输出文件）。之后驱动只为 5-乙 填了 `--gate r2` 那一套的两处钉值与装置闸字段的名字（今天 sha256 `904777afe38473aec51208dcd98b60eca2552f70e418c85443206b36fbc2287c`）：两版的差全在 `R5_GATE_R2_PINS`、`R4_GATE_R2_CONFIG_FIELD`、`check_the_registered_inputs` 里两支只认占位串的分支与自证，不给 `--gate` 的三个模式走的代码没变。自造表 Z5 原地换锚点（名字、语义、替换文不动，原文由已不存在的 `unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,` 换成 `unmount_marker: record.new_unmount_marker,`，在 `crates/singlefs-core/src/recovery.rs` 恰命中 1 次；自造表今天 sha256 `f2803173b5004e332a62b3191335e2853ed7058bfbfd7cca5e7fd29753a64bc9`）。三个模式都在第一段根上跑（`--jobs 2 --threads 3`；内存上限 r2、r4 模式 8G，r3 模式 1G），四个退出码都是 0，三份都写了收尾「已还原，基线与开头逐字相同」；r2 模式基线 1 373 行里 1 240 行在第一段两份 r2 档产物里逐字找得到（`rows_found_verbatim_in_segment1_products=1240`），r3、r4 模式基线与第一段产物逐行相同（`baseline_equals_segment2_product=true`、`baseline_equals_segment1_product=true`）。

比对用草稿脚本 `compare_segment2.py`（sha256 `d2f0d80c937ec87359129e5bf5c0b0de040308dc479d4f08fb8ffe0959a4f146`；变体按变异名对，Z 行按 Z 键，不按 `item=` 与 `table_row=`；四个模式的自证各 5 格在这一次的四份产物上全过）。三条比对的收尾行（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-comparison.out` 第 32、33、42、43、52、53 行，整行）：

```
name=summary mode=r2 compared_rows=28 expected=28 differing_variants=0 one_sided=0 reported_only_differing=23
name=exit quantity=PQ5 mode=r2 code=0
name=summary mode=r3 compared_rows=6 expected=6 differing_variants=0 one_sided=0 reported_only_differing=0
name=exit quantity=PQ6 mode=r3 code=0
name=summary mode=r4 compared_rows=6 expected=6 differing_variants=0 one_sided=0 reported_only_differing=0
name=exit quantity=PQ7 mode=r4 code=0
```

PQ5 比的是每个变体的 `outcome`、每个域的条款首红 N 与锚点首红 N、界内不红的域、J6 与 J6Qn 每个域的首红 N；PQ6 比 `outcome`、KJ0 至 KJ3 每个域的首红 (M, U)、`ka_kj2`、`ka_pin`；PQ7 比 `outcome`、KJ0 至 KJ3 加 KJ1T 每个域的首红 (M, U, 被判时故障, 历史标签)、`ka_pin`、`kg_pin`、`differs_on`。三个模式每个变体每一项都相同，**3-乙 杀伤界没变**。r2 模式的 `reported_only_differing=23` 是 PQ14（附带，不算翻面）：23 个变体首红那个 N 上红的判据名单多了 `J7a_allocation_record_tree_root`（C、C6 两个域另多 `J7a_instance_table`），是 3-甲 那一处 J7 变化在基线与变体上同时出现、而变体在那一态与基线不同的结果，变体逐行写在同一份文件的 `name=variant mode=r2` 行。红例账（PQ14 的另一半，同一份文件第 54 至 57 行，整行）：

```
name=pq14_ledger old=e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-red-examples.tsv old_data_rows=165 new=e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-red-examples.tsv new_data_rows=165 note=multiset_diff_per_column_set_lines_counted_by_diff_of_sorted_columns
name=pq14_ledger_columns columns=3,4,5,6 differing_lines=0
name=pq14_ledger_columns columns=1 differing_lines=196
name=pq14_ledger_columns columns=7 differing_lines=260
```

红例账两份都是 165 个反例状态，域、N、枚举序号、「怎么不同」四列合起来逐个相同；第 1 列（变体名带表行号，表行挪了）与第 7 列（不同的计数，新账 130 行多了 J7a 两格）不同。三份判决行（整行）：

```
E7RESULT name=verdict scope=segment2 variants=27 variants_excluding_baseline=26 red=25 red_only_by_implementation_anchor=1 not_red_within_bounds=0 invalid=0 run_did_not_finish=0 ledger_sha256=e64ac7cd25495c04ca522c3dd8b0205cb534ffb0b102a3babbe65502a7ee91c1
E7RESULT name=verdict scope=segment3 variants=7 variants_excluding_baseline=6 red=1 red_only_by_kj0_or_kj3=0 not_red_within_bounds=5 invalid=0 run_did_not_finish=0 not_compared=0 ka_pin_mismatches=0 baseline_equals_segment2_product=true
E7RESULT name=verdict scope=segment2 variants=7 variants_excluding_baseline=6 red=1 red_only_by_kj0_or_kj3=0 not_red_within_bounds=5 invalid=0 run_did_not_finish=0 not_compared=0 ka_pin_mismatches=0 kg_pin_mismatches=0 baseline_equals_segment1_product=true
```

（依次是 `e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-kill-bounds.out` 第 32 行、`e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r3-mode-kill-bounds.out` 第 10 行、`e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r4-mode-kill-bounds.out` 第 10 行。）与旧的三份判决行除 `ledger_sha256` 外逐字相同：`red_only_by_implementation_anchor=1` 是表第 224 行那一条（`item=tsv-224`），与第 2 次重跑同一条；`not_red_within_bounds=5` 是 v1822、v1823、v1825、v1826、v1827，与第 3、4 次同。

第二段的产物（`research/results/` 下 8 份）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-kill-bounds.out` | 33 | r2 模式整轮：配置 1、`kill_bound` 29（基线 1、23 个名字、Z1–Z5）、还原 1、判决 1、完成 1；复跑登记行指它 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-red-examples.tsv` | 166 | r2 模式的红例账（表头 1 行加 165 个反例状态） |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r3-mode-kill-bounds.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r4-mode-kill-bounds.out` | 11、11 | r3、r4 模式整轮（基线 1、变体 6）；复跑登记行指它们 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-kill-bounds-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r3-mode-kill-bounds-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r4-mode-kill-bounds-timing.out` | 37、11、13 | 三个模式驱动 stderr 原样（每个变体的编译与跑的秒数、挂钟；变体部分 r2 1 466.7 秒、r3 331.1 秒、r4 806.2 秒），不进逐字比对 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment2-comparison.out` | 57 | PQ5、PQ6、PQ7 三条比对（每条先报两份的 sha256 与行数）与 PQ14 红例账的分列比 |

#### 第 5 次重跑第三段（3-丙：r5 档与 J8，2026-10-04）

装置加 `--profile r5`（只认合法历史、对抗、多盘三族）：生成器照 D23（journal 的角色与格式） 已定项 15「一次发布切成几条记录时，每条记录都带同一份新根段」写，第 j 次发布的每条记录带时间线第 j 版 R_j 的实例表指针、分配记录树根、卸载记号；对抗一族字母表里的记录与 R 域残留带区分号 d 的函数（实例表指针槽 200 000 + d、分配记录树根槽 300 000 + d、卸载记号取所选根 R_0 的另一值）；生成器把每条记录的「新根段投影 → 后三项」记成真相，同一投影两份不同的后三项当场停（停机条款 PH6）。J8 五格（登记 5.3）：**J8a** 实例表指针、分配记录树根、卸载记号各一行——施加过的态（返回根的投影 ≠ 所选根的投影），返回根这一字段要等于真相里新根段投影等于返回根投影的那条记录的同一字段；**J8b**——返回根的 fsid 等于所选根的；**J8c**——一条没施加的态，返回根的 fsid、实例表指针、分配记录树根、卸载记号四项都等于所选根的。r1 至 r4 档的生成器、判据、输出一个字没动（PV3）。装置在主工作区经 `research/scripts/compile-then-swap.py` 改（`d6937857…` → `f72fc1a4…`），`crates/mutations.tsv` 追加 9 行（第 2414 至 2422 行，名字里带「r5 第三段」字样）。

3-丙 新格对旧格（登记 5.3 的对应表：J8a 实例表指针 ↔ J7a 实例表指针，J8a 分配记录树根 ↔ J7a 分配记录树根，J8a 卸载记号 ↔ J7b，J8b ↔ J7a fsid，J8c ↔ J7c；旧格取第 2 次重跑那几份入库产物，按 (族, 奇偶, 域, N) 对行），`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment3-cell-correspondence.out` 第 2、421、485、908、972、1072 行，整行：

```
name=pq9_files family=legal parity=even new_lines=1532 old_lines=1466 new_cells=65 old_cells=65
name=pq9_files family=adversarial parity=even new_lines=390 old_lines=257 new_cells=10 old_cells=10
name=pq9_files family=legal parity=odd new_lines=1536 old_lines=1471 new_cells=65 old_cells=65
name=pq9_files family=adversarial parity=odd new_lines=390 old_lines=260 new_cells=10 old_cells=10
name=pq9_files family=multi_disk parity=even new_lines=421 old_lines=405 new_cells=16 old_cells=16
name=pq9_summary cells=830 differing=0 one_sided=0 j8_nonzero=0 without_a_source_nonzero=0
```

830 格（每个 (族, 奇偶, 域, N) 五个新格）每格新格与旧格都是 0，「0 / 非 0」逐格相同；J8 在今天的实现上每格 0（PQ8，PF4 没打中），「J8 无出处」的态每格 0（PQ10，J8a 没有少判的态）。合法历史一族 L 域 N ≤ 10 那一格的判决行（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family.out` 第 290 行，整行）：

```
E7RESULT name=verdict domain=L ring_slots=64 smallest_n=1 largest_n=10 criteria=r5 j6qn_premise_holds_violations=0 j6qn_premise_holds_judged=24186470 j6qn_premise_fails_violations=0 j6qn_premise_fails_judged=0 j8_judged=24186470 j8a_judged=6046106 j8a_instance_table_violations=0 j8a_allocation_record_tree_root_violations=0 j8a_unmount_marker_violations=0 j8b_violations=0 j8b_judged=24186470 j8c_violations=0 j8c_judged=18140364 j8_without_a_source=0 first_returned_root_txg_is_the_largest=0
```

阳性对照（登记 5.4，假结局不动 `crates/`）：`even` 那份 17 行、`odd` 那份 21 行，38 行全 `red_as_required=true`（`grep 'name=control criterion=J8' <两份合法历史产物> | grep -c 'red_as_required=true'` 得 38）。`even` 那份第 62 至 78 行，整行：

```
E7RESULT name=control criterion=J8a measured=domain=L,max_n=5,implementation=C8a,counter=J8a_instance_table,marker_parity=even value=762 must_see=equals_762 red_as_required=true
E7RESULT name=control criterion=J8a measured=domain=L,max_n=5,implementation=C8a,counter=J8a_allocation_record_tree_root,marker_parity=even value=762 must_see=equals_762 red_as_required=true
E7RESULT name=control criterion=J8a measured=domain=L,max_n=5,implementation=C8a,counter=J8a_unmount_marker,marker_parity=even value=620 must_see=equals_620 red_as_required=true
E7RESULT name=control criterion=J8a measured=domain=L,max_n=5,implementation=C8a,counter=J8b,marker_parity=even value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8a measured=domain=L,max_n=5,implementation=C8a,counter=J8c,marker_parity=even value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8a measured=domain=L,max_n=5,implementation=C8a,generator=r1,counter=J8a_instance_table value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8a measured=domain=L,max_n=5,implementation=C8a,generator=r1,counter=J8a_allocation_record_tree_root value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8a measured=domain=L,max_n=5,implementation=C8a,generator=r1,counter=J8a_unmount_marker value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8a_unmount_marker measured=domain=L,max_n=5,implementation=C8b,counter=J8a_unmount_marker,marker_parity=even value=142 must_see=equals_142 red_as_required=true
E7RESULT name=control criterion=J8a_unmount_marker measured=domain=L,max_n=5,implementation=C8b,generator=r1,counter=J8a_unmount_marker value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8b measured=domain=L,max_n=5,implementation=C8c,counter=J8b,marker_parity=even value=762 must_see=equals_762 red_as_required=true
E7RESULT name=control criterion=J8b measured=domain=L,max_n=5,implementation=C8c,counter=J8a_instance_table,marker_parity=even value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8b measured=domain=L,max_n=5,implementation=C8c,counter=J8a_allocation_record_tree_root,marker_parity=even value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8b measured=domain=L,max_n=5,implementation=C8c,counter=J8a_unmount_marker,marker_parity=even value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8b measured=domain=L,max_n=5,implementation=C8c,generator=r1,counter=J8b value=0 must_see=equals_0 red_as_required=true
E7RESULT name=control criterion=J8c measured=domain=L,max_n=5,implementation=C8d,counter=J8c,marker_parity=even value=2348 must_see=equals_2348 red_as_required=true
E7RESULT name=control criterion=J8c measured=domain=L,max_n=5,implementation=C8d,generator=r1,counter=J8c value=0 must_see=equals_0 red_as_required=true
```

奇偶的判别力：C8b 在 `even` 下 142、`odd` 下 620，门槛挪到两点之间（例如 400）`even` 红、`odd` 不动；「r1 生成器那一列 0」是另一半——同一个假结局在记录三项恒为 r1 常量的生成器上看不见。C8e（同一投影两份后三项必须停在 PH6）是单测，没跑（checker 档）。

**作废条款**（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment3-profile-check.out`）：PV2（r5 档对 r2 / r3 档同一条命令，去掉 J7 / J8 字段、`config` 行与档名字样之后逐行比）五对 `differing_lines=0`；PV3（第三段二进制上 r1 至 r4 档八份重跑与第一段同名产物）与 PV5（r5 档合法历史、对抗两族 7 与 3 线程）原样（第 22 至 31 行，整行）：

```
name=pv3 product=r1-legal-family identical=True rerun_lines=910 segment1_lines=910 sha256=eb54d5032d53d93b
name=pv3 product=r1-light-legal-family identical=True rerun_lines=876 segment1_lines=876 sha256=c4af67b0a22a29f0
name=pv3 product=r1-adversarial-family identical=True rerun_lines=172 segment1_lines=172 sha256=3ca19987eed1d99d
name=pv3 product=r2-legal-family identical=True rerun_lines=1466 segment1_lines=1466 sha256=5f876ca2ba7173d6
name=pv3 product=r2-adversarial-family identical=True rerun_lines=257 segment1_lines=257 sha256=e7f10a6aaf3e2829
name=pv3 product=r3-adversarial-family identical=True rerun_lines=380 segment1_lines=380 sha256=3534db6b9b64703e
name=pv3 product=r3-root-choice identical=True rerun_lines=379 segment1_lines=379 sha256=20ca1851dea7cc41
name=pv3 product=r4-root-choice-faults identical=True rerun_lines=3309 segment1_lines=3309 sha256=10ff3260ec21d23b
name=pv5 product=r5-legal-family threads=7,3 identical=True lines=1532,1532
name=pv5 product=r5-adversarial-family threads=7,3 identical=True lines=390,390
```

PV8：r5 产物里施加过至少一次的态数为 0 的只有 E2（两种奇偶各一行，登记写明 E2 除外）。五份 r5 产物 `scope=run` 判决行都是 `controls_not_red=0 void_conditions=0 halt_conditions=0`。

**主 agent 2026-10-04 认定的五处读法**（第三段与 5-甲 交回时交的，照登记文字定的读法，不是判据改动）：① PV2 / PQ12 的比法另去掉收尾判决行的 `control_readings=` 与 `name=done` 的 `emitted=` 两个计数，改用等式核（`equals_reference_minus_j7_plus_j8=True`、`difference_equals_j8_rows_minus_j7_rows=True`）；`odd` 对抗一族在 `research/results/` 里没有 r3 档的同参数产物，参照是第三段二进制在第一段根上现跑的一份 `--profile r3 --marker-parity odd --family adversarial`（草稿，没入库；那一对的 `name=pv2_reference_note` 行记了它的 sha256 前缀 `6bf36e644c98128a`）。② C8a 至 C8d 这组对照只在 `odd` 下另跑 C 域 Nc ≤ 3，C8a 两格、C8c、C8d 各要求 > 0（读数 51、51、51、100），C8b 不另跑。③ 修1.七乙类的拒因三栏另立一张钉值表（只多钉）。④ 登记 9.2 与修1.九甲表的变异多配 5 行点名第二条单测。⑤ `research/scripts/replay.sh` 第 228 至 234、237 行（`cargo run` 编主工作区）在改过的装置上仍复得出，只在同一份源码编出的二进制上比过（PV3、PV13），没经 `replay.sh` 跑。

第三段的产物（`research/results/` 下 16 份）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-multi-disk.out` | 1 532、390、421 | r5 档三族（`even`），7 线程；复跑登记行指它们 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-legal-family.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-adversarial-family.out` | 1 536、390 | r5 档合法历史、对抗两族（`odd`）；复跑登记行指它们 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family-threads-second.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family-threads-second.out` | 1 532、390 | 同一条命令 3 线程，与 7 线程那两份整份逐字节相同 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-cell-correspondence.out` | 1 072 | PQ8、PQ9、PQ10 与 PV8：每格 `name=pq9` 830 行、`name=pq10` 166 行 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-profile-check.out` | 32 | PV2、PV3、PV5 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-multi-disk-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-legal-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-adversarial-family-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family-threads-second-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family-threads-second-timing.out` | 12 至 67 | 各份的 `E7TIMING` 行，不进逐字比对 |

#### 第 5 次重跑第五段 5-甲（Q1（R2）：系统配置读错取 R2 之后带故障的合法历史丢不丢，2026-10-04）

登记修订 1（主 agent 认定第 2 条：R2 下的带故障一族并进这一次重跑）。装置加 `--gate r2`（只在 `--profile r4 --family root_choice_faults` 认；不给就是取甲，r4 档输出逐字节不变），生成器的闸照 D22（单元原子性怎么合成） 已定项 8 射程改：可写挂载读系统配置的见证**判不出 ⟺ 这一遍读视图里有系统配置槽报读错，且没有一块盘两槽都读得出且没撕裂**；取号那一刻 (a) 判不出就拒、(b) 某块盘两槽里一份读得出且没撕裂的都没有就拒（同一条射程「某块盘一份自证过的系统配置都没有时，取号那一步也拒」）。撕裂读作「读得出、自证不过、设备不报读错」，挂载开头故障菜单里的系统配置槽落点读作「设备报读错」（主 agent 认定修订 1 第 2 条；依据 D22（单元原子性怎么合成） 已定项 8 射程的 S-辨那一句与 `crates/singlefs-core/src/recovery.rs` 的 `system_configuration_slots_of_one_device_read_once`：读不出才记读错，读得出而自证不过两头都不记）。被测的三步仍不读系统配置槽，R2 只改哪些历史被拒。开工时登记修1.三那张「生成器的闸对实现逐处对一遍」九行在第一段根上按函数名重对，九行都对得上（第 7 行逐盘核落后支照登记不造，推的不可达）。装置在第三段之后接着改（`f72fc1a4…` → `8006b251…`），`crates/mutations.tsv` 再追加 13 行（第 2423 至 2435 行，名字里带「r5 第五段」字样）。7 线程整遍 182 秒、3 线程 360 秒，两遍整份逐字节相同。

Q1（R2）对第 4 次的判定（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5-comparison-with-r4-q1.out` 第 2 至 10 行、第 15 行，整行；`fourth` 是第 4 次第一段入库那一份，`gate_a_rerun` 是第 5 次第一段在同一份根上取甲的重跑）：

```
name=pq23_files r2_lines=3359 fourth_lines=3309 gate_a_rerun_lines=3309 gate_a_rerun_equals_fourth=True
name=pq23_q1 domain=KF field=promised_kj1t_violations r2=0 fourth=0 gate_a_rerun=0 same_judgement_as_fourth=True gate_a_rerun_same_as_fourth=True
name=pq23_q1 domain=KF field=promised_kj1_violations r2=0 fourth=0 gate_a_rerun=0 same_judgement_as_fourth=True gate_a_rerun_same_as_fourth=True
name=pq23_q1 domain=KF3 field=promised_kj1t_violations r2=0 fourth=0 gate_a_rerun=0 same_judgement_as_fourth=True gate_a_rerun_same_as_fourth=True
name=pq23_q1 domain=KF3 field=promised_kj1_violations r2=0 fourth=0 gate_a_rerun=0 same_judgement_as_fourth=True gate_a_rerun_same_as_fourth=True
name=pq23_q1 domain=KF9 field=promised_kj1t_violations r2=0 fourth=0 gate_a_rerun=0 same_judgement_as_fourth=True gate_a_rerun_same_as_fourth=True
name=pq23_q1 domain=KF9 field=promised_kj1_violations r2=0 fourth=0 gate_a_rerun=0 same_judgement_as_fourth=True gate_a_rerun_same_as_fourth=True
name=pq23_q1 domain=KFW field=promised_kj1t_violations r2=0 fourth=0 gate_a_rerun=0 same_judgement_as_fourth=True gate_a_rerun_same_as_fourth=True
name=pq23_q1 domain=KFW field=promised_kj1_violations r2=0 fourth=0 gate_a_rerun=0 same_judgement_as_fourth=True gate_a_rerun_same_as_fourth=True
name=pq23_q1_promised_rows rows_r2=98 rows_fourth=98 judgement_changed=0 one_sided=0 nonzero_r2=4 nonzero_fourth=4
```

四个合法域每个域的计数（同一份文件第 16 至 19 行，整行）：

```
name=pq15_pq16_pq18 domain=KF states=1903615 confirmed_states=1689525 kj0_violations=0 panicked=0 choose_root_returned_none_with_a_readable_root=0 replay_failed=0 kj1_violations=214605 kj1t_violations=214605 kj2_violations=0 kj3_violations=0 promised_states=1358216 promised_kj1_violations=0 promised_kj1t_violations=0 kj1_and_kj1t_differing_states=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
name=pq15_pq16_pq18 domain=KF3 states=29792525 confirmed_states=16865405 kj0_violations=0 panicked=0 choose_root_returned_none_with_a_readable_root=0 replay_failed=0 kj1_violations=1381541 kj1t_violations=1381541 kj2_violations=0 kj3_violations=0 promised_states=18736200 promised_kj1_violations=0 promised_kj1t_violations=0 kj1_and_kj1t_differing_states=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
name=pq15_pq16_pq18 domain=KF9 states=488935 confirmed_states=276435 kj0_violations=0 panicked=0 choose_root_returned_none_with_a_readable_root=0 replay_failed=0 kj1_violations=35937 kj1t_violations=35937 kj2_violations=0 kj3_violations=0 promised_states=348544 promised_kj1_violations=0 promised_kj1t_violations=0 kj1_and_kj1t_differing_states=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
name=pq15_pq16_pq18 domain=KFW states=400 confirmed_states=325 kj0_violations=0 panicked=0 choose_root_returned_none_with_a_readable_root=0 replay_failed=0 kj1_violations=44 kj1t_violations=44 kj2_violations=0 kj3_violations=0 promised_states=320 promised_kj1_violations=0 promised_kj1t_violations=0 kj1_and_kj1t_differing_states=0 slot_readings_differing_from_the_truth=0 domain_is_nonempty=true
```

读法：Q1（R2）答**不丢**（承诺内 KJ1、KJ1T 每格 0，PF9 没打中），与第 4 次同判定；`name=promised` 98 行判定变了 0 行，非 0 的 4 行都在拿掉闸的 KG，两份数值也相同。KJ1、KJ1T 的违例（KF 214 605、KF3 1 381 541、KF9 35 937、KFW 44）全在被判时 RI（不在承诺内），每组与条款的理想恢复同数、上限截掉 0（同一份文件的 `name=pq17` 行），与登记修1.四用 python 生成器副本推的四个数逐个相同；KJ0、KJ2、KJ3 在四个合法域每格 0（PF10 没打中），「没根」只在 KFW（21）。拒因三栏按域加总（同一份文件第 22 至 26 行）：闸 / 取号那一刻某块盘一份自证过的都没有 / 取号那一刻判不出，KF 4 090 / 248 / 0、KF3 325 688 / 27 896 / 0、KF9 1 308 / 84 / 0，「取号那一刻判不出」界内每格 0。四个合法域「同 txg 不同实例同在」「实例代号更高而 txg 更低同在」两个计数每格 0，KG 后者 129 420（PQ24）。**PF13 归因**：第 5 次第一段那份取甲重跑与第 4 次那份整份逐字节相同（`gate_a_rerun_equals_fourth=True`），所以 7-2 的实现改动在取甲下没改判定；Q1 在闸的两种取法（取甲、R2）两点同判定，这一个旋钮不翻面。

判别力自证五条与 C331（择根倒挂压过已确认的写） 的判别力自证、整轮判决行（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5-r2-gate-root-choice-faults.out` 第 3352 至 3358 行，整行）：

```
E7RESULT name=self_proof item=r2_gate_lets_part_of_the_sn_tp_histories_through_refuses_every_sb_and_rnsb_tp_history_and_the_gate_off_domain_refuses_none passed=true detail=SN0-TP:KF_histories=28776_KF_refused=310_KG_refused=0,SN1-TP:KF_histories=28776_KF_refused=310_KG_refused=0,SB-TP:KF_histories=1238_KF_refused=1238_KG_refused=0,RNSB-TP:KF_histories=1238_KF_refused=1238_KG_refused=0
E7RESULT name=self_proof item=r2_t1_takes_the_second_read_on_every_sb_and_rnsb_t1_history_and_on_some_sn_t1_histories passed=true detail=SN0-T1:KF_histories=39931_KF_reread=6879,SN1-T1:KF_histories=39931_KF_reread=6879,SB-T1:KF_histories=39931_KF_reread=39931,RNSB-T1:KF_histories=39931_KF_reread=39931
E7RESULT name=self_proof item=r2_acquisition_refuses_a_device_without_a_self_verified_slot_on_every_kf_two_mount_cell passed=true detail=KF_M2_U0:refused=8,KF_M2_U1:refused=20,KF_M2_U2:refused=56,KF_M2_U3:refused=164
E7RESULT name=self_proof item=kf4_parts_kj1_from_kj1t_on_the_gate_off_domain passed=true detail=KF4_kj1=1995300,KF4_kj1t=2000970
E7RESULT name=self_proof item=torn_third_state_is_enumerated passed=true detail=KF_M2_U0_states_with_a_torn_system_slot=245
E7RESULT name=c331_proof domain=KG local_today_kj1t_without_a_judged_fault=648 pinned=648 implementation_kj1t_without_a_judged_fault=648 implementation_kj3_without_a_judged_fault=0
E7RESULT name=verdict scope=run bounds=segment1 family=root_choice_faults states=34814150 self_checks_passed=1 control_readings=448 controls_not_red=0 void_conditions=0 halt_conditions=0 profile=r4 witness_gate=r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots
```

第一条就是把门槛挪到两点之间的那一格：SN0-TP、SN1-TP 的历史在取甲下全被拒（第 4 次每项 1 238 / 1 238），R2 下 28 776 个历史里只拒 310。登记修1.七乙类三张钉值表（格 49 行、拒因三栏 49 行、挂载 2 的故障表 52 行，摊自设计员另写的 python 生成器的输出）在装置里逐格 `assert_eq!`，进程退 0 就是全过。PV11：KG、KA 两域与 KF、KF3、KF9 的 M = 1 各格、KFW 全部共 2 208 行，与第 5 次第一段那份取甲重跑逐字相同；其余行判定字段变了的身份 0 个（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5-profile-check.out` 的 `name=pq20_*` 行）。PV13：5-甲 的二进制重跑比对对象第 1、14、15、16 行与第三段的合法历史那一份，五份整份逐字节相同（同一份文件的 `name=pv13` 行）。

5-甲 的产物（`research/results/` 下 6 份）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5-r2-gate-root-choice-faults.out` | 3 359 | `--bounds segment1 --family root_choice_faults --profile r4 --gate r2`，7 线程，34 814 150 态；复跑登记行指它，5-乙 驱动的 TV7 比对对象 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5-r2-gate-root-choice-faults-threads-second.out` | 3 359 | 同一条命令 3 线程，与 7 线程那一份整份逐字节相同（PQ25） |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5-r2-gate-root-choice-faults-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment5-r2-gate-root-choice-faults-threads-second-timing.out` | 各 50 | 两遍的 `E7TIMING` 行（整遍 182.164 秒、359.777 秒），不进逐字比对 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5-comparison-with-r4-q1.out` | 40 | PQ15 至 PQ19、PQ23 的 Q1 一半、PQ24 与 PF13 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5-profile-check.out` | 21 | PV13、PQ20 与 PV11 |

#### 第 5 次重跑第五段 5-乙（Q2（R2）：第 3 次没红的三个择根变体在 R2 那一族上红不红，2026-10-04）

驱动 r4 模式加 `--gate r2`（登记修1.九乙；只在 `--profile r4 --pins r5` 同给时认）：调装置时加 `--gate r2`，基线 `config` 行要带装置在 R2 下实际写的闸字段 `witness_gate=r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots`（登记草拟的 `gate=r2` 是第二段执行员自定的名字，装置没照它写，主 agent 认定第三段交回时定照装置对齐），TV7 的比对对象换成 5-甲 那份产物，变异表钉值取 5-甲 的根里那一份（`c9a03eb4…`，主 agent 同一次认定）；变体名单、选行、KA 与 KG 钉值、结局分类、首红次序与第 4 次一字不差。驱动在 5-甲 的根上跑（`--jobs 2`），产物与 Q2（R2）的比对（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment5-comparison-with-r4-q2.out` 第 10、11、20、21、30、31、40、41 行，整行）：

```
name=summary mode=q2 compared_rows=6 expected=6 differing_variants=0 one_sided=0 reported_only_differing=0
name=exit quantity=PQ23_q2_r2_gate_vs_fourth mode=q2 code=0
name=summary mode=q2 compared_rows=6 expected=6 differing_variants=0 one_sided=0 reported_only_differing=0
name=exit quantity=PF13_gate_a_rerun_vs_fourth mode=q2 code=0
name=summary mode=q2 compared_rows=6 expected=6 differing_variants=0 one_sided=0 reported_only_differing=0
name=exit quantity=PF13_r2_gate_vs_gate_a_rerun mode=q2 code=0
name=summary mode=r4 compared_rows=6 expected=6 differing_variants=0 one_sided=0 reported_only_differing=0
name=exit quantity=PQ21_report_only_r2_gate_vs_fourth mode=r4 code=0
```

四条比对依次是：R2 那一份对第 4 次（比对对象表第 19 行）比 `outcome` 与 `red_legal_domains`（PQ23 的 Q2 一半）；第二段 r4 模式（同一份根上闸取甲）对第 4 次、R2 对第二段 r4 模式（PF13 归因的两点）；R2 那一份对第 4 次按 r4 模式全项比（首红位置、`ka_pin`、`kg_pin`、`differs_on`，附带只报）。四条都是 6 个变体 0 处不同。每个变体一行的读数，一条命令从产物里数（`research/results/` 下，工作目录是那里）原样输出：

```
grep '^E7RESULT name=kill_bound' e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out | grep -o 'item=[^ ]*\|variant=[^ ]*\|outcome=[^ ]*\|red_legal_domains=[^ ]*\|ka_pin=[^ ]*\|kg_pin=[^ ]*\|differs_on=[^ ]*' | paste -s -d' ' | sed 's/item=/\nitem=/g'

item=baseline outcome=baseline 
item=tsv-1810 outcome=not_red_within_bounds variant=v1822 red_legal_domains=none differs_on=none ka_pin=not_pinned kg_pin=match 
item=tsv-1811 outcome=not_red_within_bounds variant=v1823 red_legal_domains=none differs_on=KA ka_pin=match kg_pin=match 
item=tsv-1812 outcome=red variant=v1824 red_legal_domains=KF,KF3,KF9,KFW differs_on=not_applicable ka_pin=match kg_pin=match 
item=tsv-1813 outcome=not_red_within_bounds variant=v1825 red_legal_domains=none differs_on=KA ka_pin=match kg_pin=match 
item=tsv-1814 outcome=not_red_within_bounds variant=v1826 red_legal_domains=none differs_on=KG+KA ka_pin=match kg_pin=match 
item=tsv-1815 outcome=not_red_within_bounds variant=v1827 red_legal_domains=none differs_on=KA ka_pin=not_pinned kg_pin=match
```

（`tsv-1810` 至 `tsv-1815` 是第一段根里 `crates/mutations.tsv` 的行号，第 4 次是 `tsv-1821` 至 `tsv-1826`；比对按变异名对。）三个不红的子格数与第 4 次逐个相同：v1823、v1825 在四个合法域 410 个子格与 KG 80 个子格上 KJ1、KJ1T、KJ2 都是 0、只在 KA 上不同（KJ2 读 18、36）；v1826 在合法域 0、KG 上 KJ1、KJ1T 各 8 个、KJ2 12 个子格不同、KA 读 18；v1824 在 KJ1、KJ1T 上 KF 26 of 80、KF3 46 of 120、KF9 13 of 60、KFW 27 of 150，KJ2 上 52、92、40、33。Q2（R2）答**仍有不红的**，与第 4 次同；三个不红的仍判**取样点不敏感（第三类）**：变体有效（`ka_pin=match`），换到 KA（v1826 另有 KG）就不同值，而 R2 那一族四个合法域里能让它们露出来的两种根环（同 txg 不同实例同在、实例代号更高而 txg 更低同在）每格 0（5-甲 产物的 PQ24）。收尾行与判决行（产物第 9、10 行）：`已还原，基线与开头逐字相同`；`E7RESULT name=verdict scope=segment2 variants=7 variants_excluding_baseline=6 red=1 red_only_by_kj0_or_kj3=0 not_red_within_bounds=5 invalid=0 run_did_not_finish=0 not_compared=0 ka_pin_mismatches=0 kg_pin_mismatches=0 baseline_equals_segment1_product=true gate=r2`。

**两趟与那句对不上的核法**：第一趟内存上限 8G（`E165_KILL_BOUNDS_MEMORY_CAP`），tsv-1812（v1824）的二进制被内存包装拒（退出码 250，那一次输出不算结果，登记修1.十一），驱动退 1，收尾报「还有 4 个变体没有行：tsv-1812 tsv-1813 tsv-1814 tsv-1815」；第二趟内存上限 16G，开头报「已有行 3 个；这一次要跑 1 个：tsv-1812」，只补了它。两句对得上，原因在驱动的写法：每个变体跑完、没有毛病就各存一份单项记录，产物里的行却按变体次序冲刷，排在前面的一个没结果就停——第一趟 tsv-1812 撞顶，排在它后面的 tsv-1813、tsv-1814、tsv-1815 跑完存了单项记录、没写进产物，所以「没有行」数的是产物里缺的 4 个；第二趟「已有行」数的是产物里已有的 baseline、tsv-1810、tsv-1811 三行，要跑的是既没有行又没有单项记录的那 1 个，tsv-1812 跑完之后 tsv-1813 至 tsv-1815 从第一趟的单项记录冲进产物。逐个核了七行各出自哪一趟、是不是退 0 的结果：每个变体的计时（编译、跑、合计秒数）在单项记录、驱动日志与进度记录里三处相同，baseline、tsv-1810、tsv-1811、tsv-1813、tsv-1814、tsv-1815 是第一趟（8G）的，tsv-1812 是第二趟（16G）的；七行都带 `run_exit=0 run_finished=true cells_without_rows=0`，单项记录里 `problem` 都是空；第二趟开头的基线与产物里记的基线行逐字相同（不同就退 6、不往下跑）。所以没有一行要照作废条款作废。两趟的驱动日志与计时原样入库（`…-first-pass-*`、`…-second-pass-*`）；进度记录（每个变体一行，含第一趟 tsv-1812 那一行 `problem=内存包装退出码 250，这一次不算结果`）原样入库为 `…-progress-log.out`。产物名照驱动收的 `--output` 原样（登记写的是 `segment5-r2-gate-kill-bounds.out` 与 `segment5-r2-gate-driver-log.out`，跑的时候起了别的名字，内容没变）；产物行里的复跑命令写着 `--root <第一段根>` 字样，实际指的是 5-甲 的根。

5-乙 的产物（`research/results/` 下 7 份）：

| 文件 | 行数 | 是什么 |
|---|---|---|
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out` | 11 | 驱动 r4 模式 `--pins r5 --gate r2` 整轮：配置 1、`kill_bound` 7（基线 1、变体 6）、还原 1、判决 1、完成 1；复跑登记行指它 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds-first-pass-driver-log.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds-second-pass-driver-log.out` | 5、3 | 两趟驱动 stdout 原样（第一趟 8G、第二趟 16G），不进逐字比对 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds-first-pass-timing.out`、`e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds-second-pass-timing.out` | 15、6 | 两趟驱动 stderr 原样（每个变体的编译与跑的秒数；第一趟变体部分挂钟 1 259.8 秒，第二趟 406.7 秒），不进逐字比对 |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds-progress-log.out` | 8 | 驱动的进度记录原样（两趟共 8 行） |
| `e165-replay-bounded-enumeration-2026-10-04-r5-segment5-comparison-with-r4-q2.out` | 41 | PQ23 的 Q2 一半、PF13 两点、PQ21 全项比（附带） |

**几何敏感性**（登记第八节与修1.八新加的旋钮）：闸的判不出取法（取甲对 R2）——Q1 两条判据在四个合法域两点都是 0，v1823、v1825、v1826 两点都不红，不翻面；卸载记号奇偶（3-丙 `even` 对 `odd`）、环槽数（大环对小环）、单盘对多盘、所选根第 0 版对随落盘变的域——J8 每格两点都是 0，不翻面；线程数——r5 档两族、R2 一族、r4 带故障一族各两遍整份逐字节相同。被谓词消费的量这一次没有新的：在飞上限的分布行、「上限截掉」的态数（四个合法域每格 0）照各档原样出，R2 一族闸的谓词按三栏拒因分别报（「取号那一刻判不出」一栏界内每格 0）。

````

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1205-1226`（整段抄，未转述）**

```markdown
### 复跑

`bash research/scripts/replay.sh E165`（登记行 `driver_e165`、`driver_e165_light`、`driver_e165_adversarial`、`driver_e165_segment2`，判据 exact）。计数与线程数无关：第一段 `--family legal` 用 10 线程与 7 线程、`--family adversarial` 用 16 线程与 7 线程各跑一遍，`E7RESULT` 行逐字节相同。

第二段的驱动 `research/scripts/e165-replay-mutation-kill-bounds.sh`：`--list-items` 列 26 个变体；每行产物带只复跑那一条的命令（`--recheck --item <变体>`，重跑之后拿新行与产物里的行比）；中途断了重跑同一条命令只补没跑的。红例账每行带单条重放的命令 `target/release/e165_replay_bounded_enumeration --replay-state <域>:<N>:<序号>`，原实现上重放的判定就是那一行第 12 列。2026-10-01 在 `--jobs 4 --threads 4` 上整段 32 分钟，同机负载均值 17–90（原样在第二段计时产物里）。

第 1 次重跑的第二段产物 `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-kill-bounds.out` 与 `research/results/e165-replay-bounded-enumeration-2026-10-01-r1-segment2-red-examples.tsv` 由同一个驱动的 r1 版跑出，复跑登记里第二段那一行（行首是实验编号，往后依次是 `@driver_e165_segment2`、空的参数列、产物名、判据）原先指它们，第 2 次重跑的 2c 之后改指 `@driver_e165_segment2||e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out|exact`；r1 那两份留着，r1 驱动钉的 `crates/mutations.tsv` sha256 是 `8c24d2c627caf36cd0931a5b0405c19c0a7f24a7086d2b0e9ede1fd58431fe38`，今天的驱动是 r2 版、只为 r2 档写，复不出它们（要复 r1 的数，用 git 里 r1 那份驱动与 r1 冻结的根）。搬家之前的第二段产物（`research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out`、`research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv`、`research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds-timing.out`）在今天的树上复不出来，留着不删。第一段的三行（`driver_e165`、`driver_e165_light`、`driver_e165_adversarial`）不动：重跑的两族与入库那份逐字节相同。

第 2 次重跑加的六行（第三段新文件一行 `@driver_e165_r2_segment3_multi_disk`，第四段五行 `@driver_e165_r2_segment4_table_both_legal`、`…_table_both_adversarial`、`…_table_both_multi_disk`、`…_reversed_disk_order_legal`、`…_reversed_disk_order_multi_disk`）都是 `--profile r2`，线程数钉 8，计数与线程数无关（多盘一族 8 与 4 线程两遍整份逐字节相同）；做法与来历写在 `research/scripts/replay.sh` 里那几个驱动函数的注释里。第四段合法历史一族与对抗一族各约 200 秒（8 线程，同机负载均值 25–35）。第 2 次重跑第二段补跑（2c）的驱动 `research/scripts/e165-replay-mutation-kill-bounds.sh` 要 `--root` 指冻结根（驱动钉的表哈希是冻结根的 `9d5fd9fc…`，主工作区的 `crates/mutations.tsv` 变了就停在 RH1），整段 12 批、每批两个变体约 2–3 分钟（同机负载 12–30）；`replay.sh` 第 225 行改指它的产物，一次起整段（不分批）估计 15 至 20 分钟（推的，没整轮量过；segment2b 那一遍整轮 778 秒，其中五个变体 0.5 秒退出；这一遍 26 个变体都跑，批内两个并行，十二批各 2–3 分钟），这一行是重型复跑，只在提交时由主 agent 跑。

第 3 次重跑第一段的产物 `e165-replay-bounded-enumeration-2026-10-02-r3-segment1-adversarial-family.out` 的复跑命令：`E165_THREADS=8 bash research/scripts/run-with-memory-cap.sh <上限> cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial --profile r3 | grep '^E7RESULT '`（计数与线程数无关，8 与 4 线程两遍整份逐字节相同；8 线程整遍 112 秒）。登记行 `E165|@driver_e165_r3_segment1_adversarial||<产物名>|exact` 与驱动函数 `driver_e165_r3_segment1_adversarial` 已在 `research/scripts/replay.sh` 里（第三段交回时加）。产物是第 3 次重跑第一段的冻结根（拷主工作区，清单 233 行）上编的二进制跑的；主工作区以后 `recovery.rs` 等别的文件变了，复不出时先对那份清单。

第 3 次重跑第二段的产物 `e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out` 的复跑命令：`E165_THREADS=8 bash research/scripts/run-with-memory-cap.sh <上限> cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family root_choice --profile r3 | grep '^E7RESULT '`（计数与线程数无关，8 与 4 线程两遍整份逐字节相同；整遍约 70 秒，内存包装记下的实测峰值 446 MiB；`<上限>` 写不超过峰值两倍的数）。登记行 `E165|@driver_e165_r3_segment2_root_choice||<产物名>|exact` 与驱动函数 `driver_e165_r3_segment2_root_choice` 已在 `research/scripts/replay.sh` 里（第三段交回时加）。产物是第二段冻结根上编的二进制跑的；主工作区的 `recovery.rs`（择根胶水还在被别的会话改）与 `crates/singlefs-replay-prefix/src/lib.rs` 以后变了，复出的数可能不同，复不出时先对第二段冻结根的清单。

第 3 次重跑第三段（Q2）的驱动：`research/scripts/e165-replay-mutation-kill-bounds.sh` 加 `--profile r3` 模式（登记 9.3、修订 3；已装进仓，sha256 `1a3dafb007079a31e0609c53f541d05e0bc88f591955e6be709787b2e267e042`），不给 `--profile` 仍是 r2 那一套。r3 模式在第二段冻结根（233 个文件，清单 sha256 `c985600264b47a9ed81c8b6e538826d7899eacd4241f062f865b5c5b86a7b290`）上给 9.1 的六个变体各建一份副本、各冷编一次，基线与每个变体都调装置 `--profile r3 --bounds segment1 --family root_choice`，逐 (域, M, U) 比 KJ0 至 KJ3 的违例数与计数行；基线先与第二段产物逐行比（不同退 6），每个变体在 KA 上的 KJ2 违例数对登记钉值（1823 → 18、1824 → 52、1825 → 36、1826 → 18）。登记行 `E165|@driver_e165_r3_segment3_root_choice_kill_bounds||<产物名>|exact` 与驱动函数 `driver_e165_r3_segment3_root_choice_kill_bounds` 已在 `research/scripts/replay.sh` 里，产物是 `e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out`（11 行，这一段已跑）；`--jobs 2 --threads 3`，整遍的挂钟只在驱动 stdout 里记过（9 次冷编加 7 次装置，没有入库的计时产物），是重型复跑，只在提交时由主 agent 跑。`--root` 默认指第二段冻结根（`E165_R3_FROZEN_ROOT` 可换），那份没了就照登记修订 2 第 10 条重建，主工作区的 `crates/` 变了驱动会停在开工核对（RH1）。

第 4 次重跑第一段的产物 `e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out` 的复跑命令：`E165_THREADS=8 bash research/scripts/run-with-memory-cap.sh <上限> cargo run -q --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- --bounds segment1 --family root_choice_faults --profile r4 | grep '^E7RESULT '`（计数与线程数无关，6 与 3 线程两遍整份逐字节相同；整遍 6 线程约 66 秒，峰值 RSS 约 1.66 GB，`<上限>` 写不超过包装峰值表里实测峰值两倍的数）。登记行 `E165|@driver_e165_r4_segment1_root_choice_faults||<产物名>|exact` 与驱动函数 `driver_e165_r4_segment1_root_choice_faults` 已在 `research/scripts/replay.sh` 里。产物是第一段根上编的二进制跑的；生成器不读 `mount.rs`、`transaction.rs`，7-2 让实现改取 R2 之后，第 5 次重跑第一段在新的根上重跑这一族仍与这一份整份逐字节相同，但这一族照的「取甲」已不是今天的写路径（R2 下的同一族是第 5 次重跑 5-甲）；复不出时先对根清单 `root-manifest-after-overlay-final.sha256`（建法见本页「第 4 次重跑」一节的装置与根）。第一遍的五份产物（不带 `-2026-10-03-r2`）留着，`E7RESULT` 与第二遍逐字节相同。

第 4 次重跑第二段（Q2）的产物 `e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out` 的驱动：`research/scripts/e165-replay-mutation-kill-bounds.sh --profile r4`（登记 9.3、修订 3；sha256 `3428a78ee8d41be830905e6595e3212cfa36acc49805adb45deab9f16cd06c56`），`--root` 指第一段根（`/tmp/claude-1000/e165-r4-segment1b/root`，清单 `root-manifest-after-overlay-final.sha256`；那份没了就照登记修订 2 的「根」条重建、用 `E165_R4_FROZEN_ROOT` 指过去），驱动钉着根里 `recovery.rs`、`lib.rs`、`crates/mutations.tsv` 的 sha256 与两个函数的区间，主工作区的 `crates/` 变了它也不受影响，但根没了就复不出。登记行 `E165|@driver_e165_r4_segment2_root_choice_faults_kill_bounds||<产物名>|exact` 与驱动函数 `driver_e165_r4_segment2_root_choice_faults_kill_bounds` 在 `research/scripts/replay.sh` 里：`E165_KILL_BOUNDS_MEMORY_CAP=8G`、`--jobs 2 --threads 3`，整轮约 10 分钟（2026-10-03 那次第一趟变体部分挂钟 536.5 秒、不含开头与收尾的基线，驱动日志第 14 行；变体各编约 50 秒、各跑约 130 秒，基线各 66 秒量级），重型复跑，只在提交时由主 agent 跑。**上限不能取 4G**：2026-10-03 第一趟 4G 时 tsv-1823 撞顶退 250（驱动日志第 8 行）；这一族整遍峰值第一段约 1.66 GB，tsv-1823 为什么超 4G 没查。

第 5 次重跑：第一段把 `replay.sh` 第 228 至 234 行改指第一段在第一段根上的重跑（第 2 次、第 3 次的旧产物留着，今天的装置在 J7a 两格上复不出它们），第 224 至 226、235、237 行不动（重跑与入库那份逐字节相同）。新加十行登记与十个驱动函数（做法与来历写在 `research/scripts/replay.sh` 那几个驱动函数的注释里）：第二段三行 `@driver_e165_r5_segment2_r2_mode_kill_bounds`、`…_r3_mode_kill_bounds`、`…_r4_mode_kill_bounds`（驱动 `--pins r5`，`--root` 默认指第一段根，函数里写了根没了怎么重建；r2、r4 模式 8G、r3 模式 1G，`--jobs 2 --threads 3`）；第三段五行 `@driver_e165_r5_segment3_legal`、`…_adversarial`、`…_multi_disk`、`…_odd_marker_legal`、`…_odd_marker_adversarial`（主工作区 `cargo run`，`--profile r5`，线程数钉 8）；5-甲 一行 `@driver_e165_r5_segment5_r2_gate_root_choice_faults`（主工作区 `cargo run`，`--profile r4 --family root_choice_faults --gate r2`）；5-乙 一行 `@driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds`（驱动 `--pins r5 --gate r2`，`--root` 默认指 5-甲 的根，内存上限 16G）。第三段与 5-甲 的产物是第三段根（装置 `f72fc1a4…`）与 5-甲 的根（装置 `8006b251…`）上编的二进制跑的；主工作区今天的装置是 `8006b251…`，5-甲 的二进制在 5-甲 的根上重跑第三段的合法历史那一份整份逐字节相同，另外四份没用它重跑过，复不出时先对着根清单查。驱动的四行都是重型复跑，只在提交时由主 agent 跑。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1227-1260`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | 今天的实现 | 调 `replay_journal`，取返回的根 | `crates/singlefs-core/src/recovery.rs:3057 replay_journal`；装置里 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:927 call_replay_journal` | 生成器造的记录字节与 `records` 表；`JournalRecord::parse` |
| 2 | 照条款写的最远一版 j\* | 声明式：从所选根起连续完整、记录条数之和不超过在飞上限的最远一版；只读生成器记下的真相 | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:827 farthest_version_of` | 本地常量（环槽数、F）；生成器的真相（哪条记录属于哪次发布、哪条在盘上）；D23（journal 的角色与格式） 已定项 14 的五条口径 |
| 3 | 照条款写的全函数 S（三种读法） | 逐条取前缀：登记 5.3 的第 1–4 条与 (a)–(k) | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1082 specification_expectation` | `records` 表（与路径 1 同一张）；本地常量；生成器记下的链值与点名单元真相；同一批条款 |
| 4 | 「同一实例里 journal 记录的 txg 随计数器不减」在对抗一族每个 (域, n) 上成立 / 不成立的态数（第 3 次重跑第一段） | 装置本地：把交给 `replay_journal` 的记录表按（实例，计数器）升序摊开、同一实例里相邻两条后一条的 txg 更小即不成立，逐态数 | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:3176 record_table_satisfies_i812` | 对抗域生成器造的记录表（`parse` 拒收的序号 0 不进表）；生成器字母表里的 txg 取法 |
| 5 | 同一个量的独立算法 | 设计员另写的脚本：对抗一族每个 (域, n) 的状态按谓词分类，动态规划与暴力枚举互核（登记第四节点名、sha256 记在登记里） | 无实现：脚本没入库，只把它的分类计数写成装置里的常量表，常量表与断言在 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6316 assert_the_pinned_i812_state_counts` 一带 | 同一字母表的文字写法（登记 5.2：txg 取 t、t + 1、t + 2，每槽 73 种取法）；同一条谓词的文字写法 |
| 6 | 今天的三步在 K、K9、KW、KA 上落到的版本（第 3 次重跑第二段） | 对内存盘调 `choose_root`、`scan_journal`、`replay_journal(…, true)`，取返回的根的 (txg, 实例代号) 与六个投影字段 | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:10127 outcome_of_the_three_steps` | 生成器造的根槽字节与记录字节（装置自己的读者 `RootChoiceDisk`）；本地常量（根环区域数、基槽、步长、chunk、区域归属、根槽宽、槽距、暖机计划、每区槽数） |
| 7 | 理想恢复（第 3 次重跑第二段） | 声明式：从给定的根起沿它的实例逐次往后，每次发布的每条记录至少一份在盘上、条数累计不超在飞上限，走到最远的那一版；只读生成器记下的真相 | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:9553 root_choice_ideal_recovery_from` | 本地在飞上限（环槽数 ÷ F）；生成器的真相（哪条记录属于哪次发布、哪几份在盘上、每次发布的六个字段）；D23（journal 的角色与格式） 已定项 14 注 1 的链首口径 |
| 8 | K、K9、KW 各格的状态数与几样计数的独立生成器（第 3 次重跑第二段） | 另写的 python 脚本（不读装置）照登记 5.3 的文字生成全部状态，数状态数、有已确认发布的态数、根环形状几样计数、理想恢复施加过的态数、所选根之上记录条数的最大值、最大计数器与两种假结局的违例数 | 无实现：脚本没入库（sha256 记在登记第四节），只把它的数写成装置里的常量表 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:11010 PINNED_ROOT_CHOICE_CELLS`，断言在 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:10857 assert_the_pinned_root_choice_cell` | 同一份登记文字（5.3 的生成规则、已确认的认法、暖机、崩溃点切法、版本序） |
| 9 | 四个择根序变体在 KA 上的 KJ2 违例数（第 3 次重跑第三段） | 两条路：真变体代码（驱动在副本里改 `choose_root_slot` 或 `choose_root` 的比较式，编出装置、经三步落点）对 KA 的 125 态；装置里本地写的五种择根序（只对着生成器记下的真相里的根环选根，不调实现）各自的 KJ2 违例数 | 真变体：`research/scripts/e165-replay-mutation-kill-bounds.sh:1207 r3_ka_pin`（读数对钉值）；本地：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:9823 chooses_among`，钉值 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:10843 pinned_root_choice_adversarial_kj2_violations` | KA 的 125 态生成器；KJ2 的判据与版本序（真相里版本最大的根）；登记 9.1 四行替换文的比较式文字 |
| 10 | 今天的三步在带读故障的历史上落到的版本（第 4 次重跑第一段） | 对内存盘（真相的盘，减去被判时读故障的落点：根槽读错，盘不在时那块盘上的读全读错）调 `choose_root`、`scan_journal`、`replay_journal(…, true)`，取返回的根的 (txg, 实例代号) 与六个投影字段 | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:13799 outcome_of_the_three_steps_on_a_faulted_disk` | 生成器造的根槽字节与记录字节（r3 同一份内存盘 `RootChoiceDisk`）；本地常量（根环区域数、基槽、步长、chunk、区域归属、根槽宽、槽距、暖机计划、每区槽数，加每块盘系统配置槽数、mkfs 世代号、可写设备数下限三样） |
| 11 | 条款的理想恢复与四条判据（读视图上，第 4 次重跑第一段） | 声明式：读视图里版本最大的根起，沿它的实例逐次往后，每次发布每条记录至少一份读得出、条数累计不超在飞上限，走到最远的一版（本体是路径 7）；KJ1、KJ1T、KJ2、KJ3 各读落点：版本序、父链上的后代、不早于读得出的根、六个字段 | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:12934 ideal_landed_version_of_a_view`（取最远一版）、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:13752 fault_criteria_of_a_landing`（四条判据） | 本地在飞上限（环槽数 ÷ F）；生成器的真相（根环、记录拷贝、已确认的发布、父链、读视图）；D13（验证路线） 已定项 1 M3 与 D22（单元原子性怎么合成） 已定项 7 的版本序；C331（择根倒挂压过已确认的写） 一行的验收句（KJ1T 的后代） |
| 12 | 每个 (域, M, U) 格的生成器一侧计数、本地择根序读数与 M = 2 故障表的独立生成器（第 4 次重跑第一段） | 设计员另写的 python 脚本 `anchors_r4.py`（不读装置，照登记 5.3 文字另写；sha256 记在登记第四节与第十三节）：按格数历史数、被判态数、被拒与重读数、撕裂态数、理想恢复丢的三类、六种本地择根序的 KJ1 / KJ1T / KJ2 与 KF1、KF2、KF4 假结局的读数 | 无实现：脚本没入库（`/tmp/claude-1000/e165-r4-designer/anchors_r4.py`），只把它的输出摊成装置里两张文本常量 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:14635 PINNED_FAULT_CELLS_TEXT`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:14688 PINNED_MOUNT_TWO_FAULT_TEXT`，断言在 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:14833 assert_the_pinned_fault_cell` 与 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:14924 assert_the_pinned_mount_two_table` | 同一份登记文字（5.3 的生成规则：闸与取号那一刻、写行 txg、已确认与父链、暖机、崩溃点切法、版本序、三个标签）；两边都没读 `mount.rs`、`transaction.rs`，闸的取法（实现的取甲）都照登记写 |
| 13 | 六个择根序变体在新一族 KA 与 KG 上的违例数（第 4 次重跑第二段） | 两条路：真变体代码（驱动在第一段根的副本里改 `choose_root_slot` 或 `choose_root` 的比较式、编出装置、经三步落点）在 KA 的 KJ2 违例数与 KG 每个 (M, U) 格的 KJ1、KJ1T、KJ2 合计；装置里本地写的六种择根序（`FaultLocalRule`，只对着生成器记下的真相里的根环选根，加理想重放，不调实现）的同一批读数 | 真变体：`research/scripts/e165-replay-mutation-kill-bounds.sh:1459 r4_ka_pin`、`research/scripts/e165-replay-mutation-kill-bounds.sh:1469 r4_kg_pin`（读数对钉值与本地列）；本地：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:15081 ka_kj2_violations_of_a_local_rule`，枚举在同一份装置的 `FaultLocalRule`（第 13676 行） | KA 的 125 态与 KG 的生成器；KJ1、KJ1T、KJ2 的判据与版本序；登记 9.1 六行替换文的比较式文字；登记第七节「本地择根序」表（设计员另写的 python 脚本 `anchors_r4.py` 的输出，sha256 记在登记里） |
| 14 | 生成器真相对今天的施加（第 5 次重跑第三段 J8） | 生成器给每条记录写后三项（时间线上第 j 次发布的记录带 R_j 的、时间线之外的带区分号的函数），同时把「新根段投影 → 后三项」记成真相；今天的实现施加出返回根（路径 1），J8 拿返回根的投影查真相、逐项比三项与 fsid | `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1005 GeneratedRecordTruth`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:3680 judge_the_fields_beyond_the_projection_against_the_generated_records`；生成器 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:496 record_fields_of_the_timeline_publish`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:513 record_fields_off_the_timeline` | 生成器同一段代码既造记录字节又记真相（生成器把后三项写错，记录与真相一起错、J8 照样 0）；记录字节经 core 的 `to_bytes` / `parse` 往返；D23（journal 的角色与格式） 已定项 15 的原句（「施加一条记录 = 把所选根的这七个字段换成记录里的」）与 D22（单元原子性怎么合成） 已定项 7 的字段表 |
| 15 | R2 闸下每个 (域, M, U) 格的生成器一侧计数、拒因三栏与 M = 2 故障表的独立生成器（第 5 次重跑 5-甲） | 两条路：装置的 R2 闸与取号那一刻（照登记修1.五文字另写）逐态生成并计数；设计员另写的 python 脚本 `anchors_s5.py`（由第 4 次的 `anchors_r4.py` 逐处替换而来，环境变量切回取甲时去掉新加的拒因分计键，49 行与 `anchors_r4.py` 的输出逐行逐键相同）照同一段文字另算一遍 | 装置：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:13948 newer_publish_is_witnessed_under_r2`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:13377 acquisition_refusal_of`；另一条无实现：脚本没入库（全文在登记修订 1 的附录里），只把它的输出摊成三张文本常量 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:15796 PINNED_FAULT_CELLS_TEXT_UNDER_R2`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:15850 PINNED_MOUNT_TWO_FAULT_TEXT_UNDER_R2`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:15908 PINNED_REFUSALS_BY_REASON_TEXT_UNDER_R2`，断言在 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:16107 assert_the_pinned_fault_cell_under_the_gate` 与 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:16230 assert_the_pinned_mount_two_table_under_the_gate` | 同一份登记文字（修1.五：R2 判不出、取号那一刻两道拒、撕裂读作「读得出、自证不过、不报读错」、拒因怎么分栏，其余照 r4 5.3）；两边都没读 `mount.rs`、`transaction.rs`，闸照条款文字与登记写 |

连起来比的断言：路径 1 对路径 2 是 J4（`judge_a_legal_state`，第 1941 行起）；路径 3 对路径 2 是 S 自己的对照（第 2028 行）；路径 4 对路径 5 是 `assert_the_pinned_i812_state_counts`（每个 (域, n) 逐格 `assert_eq!`，装置每次跑都做，十个格都等）。路径 4 与 5 共用字母表与谓词的文字写法：两边把同一条谓词读错会一起错而逐格相等，钉它的是登记第七节乙类的分类计数加 KF3 假结局四格（第一次返回所选根、第二次报错，不读实现）。三条路径共用生成器与同一批条款：生成器造错或条款被三处一起读错，三条会一起错而逐格相等；钉它的是登记第七节的状态数、见证数锚点（另一份 python 脚本算的）与阳性对照。取值是扫遍界内的定义域，不是枚举点。

第 3 次重跑第二段：路径 6 对路径 7 是 KJ3（`judge_the_outcome`，第 10240 行起：返回根的六个投影字段对理想恢复那一版的六个字段），KJ1 与 KJ2 是路径 6 落到的版本对生成器记下的真相（已确认的最后一次发布、根环里版本最大的根）；路径 7 与路径 8 的数断言在 `assert_the_pinned_root_choice_cell`（每个格逐项 `assert_eq!`，装置每次跑都做，37 个格都等）。路径 6 与 7、8 共用生成器与登记文字：生成器、已确认的认法或版本序读错了，三条会一起错而逐格相等；钉它的是阳性对照（KF1、KF2 两种假结局的读数、KA 上五种本地择根序的读数，不读实现）与独立生成器的计数。读槽对拍（`root_slot_reading_agrees_with_the_truth`，第 10343 行）把 `read_root_ring_slot` 的读数与生成时的真相逐槽比，是三步读盘那一层与生成器之间的一条独立的断言。取值是扫遍界内的定义域（每个格里的每个崩溃点），不是枚举点。

第 3 次重跑第三段：路径 9 的两条路逐个变体对上（18、52、36、18），但没有一处断言把它们直接连起来：本地那一份与钉值常量的对比在装置里（阳性对照的 `must_see=equals_…`，每次跑都核），真变体读数与钉值的对比在驱动里（`ka_pin`，不等整轮退 1），两边靠同一组钉值间接相连。两条路共用 KA 生成器、KJ2 的判据与登记 9.1 的比较式文字：生成器或判据读错，两条会一起错而逐格相等；钉它的是登记第四节算钉值的脚本（不读装置）。Q2 的 K 族那一半（变体与基线逐格比）是同一个二进制族上的两次跑相比，基线与第二段产物逐行相同（TV7），旁边钉的是第二段 36 个格的状态数与几样计数。取值是扫遍界内的 K、K9、KW 全部状态与 KA 的 125 态，不是枚举点。

第 4 次重跑第一段：路径 10 对路径 11 是 KJ3（返回根的六个投影字段对理想恢复那一版的六个字段）、KJ1、KJ1T、KJ2（落到的版本对生成器记下的真相：最后一次已确认的发布、父链、读视图里读得出的根），写在 `judge_a_state`（第 14176 行起）与 `count_the_outcome`（第 14417 行起）里，装置每次跑都做；路径 11 与 12 的数断言在 `assert_the_pinned_fault_cell`（每个格逐项 `assert_eq!`，49 个格都等）与 `assert_the_pinned_mount_two_table`（四个域各 13 项）。路径 10、11 与 12 共用同一份登记文字：闸的取法（实现的取甲）、写行 txg 取条款允许的最小值、读故障摆在哪几个落点、三个标签怎么认，读错了三条会一起错而逐格相等；钉它的是阳性对照（KF1、KF2、KF4 三种假结局与六种本地择根序的读数，不读实现；KF4 在 KG 的 M = 2、U = 3 一格两列不同）、KG 上 C331（择根倒挂压过已确认的写） 的判别力自证（本地理想恢复算出的 648 与今天的三步在同几格读到的 648 是两条路各算一遍）和读槽对拍（`fault_slot_reading_agrees_with_the_truth`，第 13863 行：被判时故障的落点必须读成 `Unreadable`，别的槽读成自证过的必须等于真相）。取值是扫遍界内的定义域（每个格里的每个崩溃点与每项故障），不是枚举点。

第 4 次重跑第二段：路径 13 的两条路逐个变体对上（KA：18、52、36、18 与 v1822 的 0、v1827 的 18；KG：每个 (M, U) 格的 KJ1、KJ1T、KJ2 合计对本地那一列，`ka_pin_mismatches=0`、`kg_pin_mismatches=0`），但没有一处断言把它们直接连起来：本地那一份与钉值常量的对比在装置里（阳性对照里 KA 上六种本地择根序的 KJ2 读数，每次跑都核），真变体读数与钉值的对比在驱动里（`ka_pin`、`kg_pin`，不等整轮退 1），两边靠同一组钉值间接相连。两条路共用 KA 与 KG 的生成器、KJ1、KJ1T、KJ2 的判据与登记 9.1 六行替换文的比较式文字：生成器或判据读错，两条会一起错而逐格相等；钉它的是登记第七节算本地择根序的 python 脚本（不读装置）。Q2 的合法域那一半（变体与基线逐子格比）是同一个二进制族上的两次跑相比，基线与第一段产物逐行相同（登记 TV7）。取值是扫遍界内的四个合法域全部状态与 KG、KA 的全部状态，不是枚举点。

第 5 次重跑第三段：路径 1 对路径 14 是 J8 五格，写在 `judge_the_fields_beyond_the_projection_against_the_generated_records` 里，装置每次跑都做。两条路共用生成器：生成器把后三项写错（例如写回 r1 常量），记录与真相一起错、J8 照样 0；钉它的是 C8a 至 C8d 的假结局钉值（762、620、142、2 348，由「第 j 版投影外三项两两不同、卸载记号奇偶交替」另算，`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family.out` 第 62 至 78 行）与「r1 生成器那一列 0」，以及 r5 档状态数等于 r3 档的断言。取值是扫遍界内三族全部状态，不是枚举点。

第 5 次重跑 5-甲：路径 15 的两条路逐格对上靠 `assert_the_pinned_fault_cell_under_the_gate` 与 `assert_the_pinned_mount_two_table_under_the_gate`（49 个格、拒因三栏 49 行、挂载 2 的故障表 52 行，逐项 `assert_eq!`，装置每次跑都做）。两条路共用登记修1.五那段文字：文字本身把 R2 写错（例如撕裂该不该算报读错），两条会一起错而逐格相等；钉它的是判别力自证五条（第一条在取甲与 R2 两点取值不同）、修1.九的 13 行装置变异（未跑）与开工时生成器的闸对实现九处逐处对过。路径 10、11 照旧（三步、理想恢复与四条判据）在 R2 生成的历史上判。取值是扫遍界内的定义域，不是枚举点。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1261-1288`（整段抄，未转述）**

```markdown
### 它答不了的

- **第 5 次重跑 3-丙 的「J8 每格 0」只在这个界与这一种取值方式里**：投影外三项只扫了一种取法（时间线上第 j 版三项两两不同、卸载记号奇偶交替；时间线之外取区分号的函数），两版实例表指针相同、分配记录树根恒空、卸载记号全是 `Written…` 这类取法没扫；一次发布几条记录带不同的后三项（合法写路径写不出）没造；`--table both`、反盘次序、带读故障与择根两族、多于两块盘在 r5 档没跑。生成器既造记录又记真相，生成器写错时 J8 照样 0，钉它的只有 C8a 至 C8d 的假结局钉值。
- **第 5 次重跑 3-甲 的「判定变了，只变在 J7」不说明今天的实现错了**：J7 判的是被 D23（journal 的角色与格式） 已定项 15 今天的原句取代的施加规则；3-甲 只比判定字段，判定不变而数变的行另报（第一段比对产物的 `name=pq2*` 行）。
- **第 5 次重跑 3-乙 的「杀伤界没变」只比首红位置与结局**：红了之后更大的 N 上别的判据红不红没量；首红那个 N 上红的判据名单（多了 J7a 两格）另报、不算翻面。
- **第 5 次重跑 5-甲、5-乙（R2 那一族）没造的**：撕裂只取「读得出、自证不过、不报读错」一种读法（撕裂也报读错的那一种在这一族里与「R2 前一半拿掉」同值，只当变异）；系统配置槽读回全 0 的混合形、一块盘两槽同时报读错（挂载开头故障菜单里没有这一项）、逐盘核落后支（推的不可达）、读错持续到下一次挂载、「只坏第一次读」那一种 T1 都没造；界与摆法同第 4 次。Q1（R2）「承诺内不丢」、Q2（R2）「仍有不红的」只说这个界内。R2 一族整遍峰值没读到（包装那一次没打实测峰值），5-乙 第一趟 tsv-1812 为什么超 8G 没查。
- **第 5 次重跑的装置单测与装置变异没跑**：新加 16 条单测与 `crates/mutations.tsv` 第 2414 至 2435 行 22 行变异（名字带「r5 第三段」的 9 行、带「r5 第五段」的 13 行）是 checker 档，登记 PV9、PV10 按提交时 checker-tier-crates-mutation-replay 的输出判；在那之前 3-丙 与 5-甲 的判别力只靠产物里的对照读数与判别力自证。第四段（PQ13：四个改坏施加的变体在 r5 档上的首红，J8 在真代码上的阳性对照）够判后未跑，J8 抓不抓得到真变体没量。
- **第 4 次重跑第一段 Q1 的「承诺内不丢」只在这个界与这些摆法里**：两块盘、区域归属 0 / 1 / 0、每区槽数 4 与 8；读故障只落在根槽、系统配置槽、整块盘三类（journal 记录槽、同一次挂载里两项以上的读故障（SB、RI、RNSB 本身除外）、故障跨两次以上挂载不恢复，都没造）；M ≤ 3（KF3，k = 1）、KF 与 KG 的 U ≤ 3、KF9 的 U ≤ 2、KFW 的 U ≤ 14，每次发布至多 2 条记录；journal 环不回绕；被判时系统配置槽的故障（三步不读它们）、取号那一写的崩溃点、系统配置其余字段与 F、管理员回退、实例切换、发布失败与原样重发、转只读、只读挂载的择根，都没造。
- **第 4 次重跑的闸取的是 2026-10-03 那一版实现的取甲**（登记修订 1）：那一次的「承诺内不丢」只对取甲生成的历史成立。7-2 之后实现取 R2，「一槽系统配置读不出而另一块盘两槽都自证过」的历史不再被拒、接着往下走；R2 下的同一族（同一个界与摆法）由第 5 次重跑 5-甲 重判，答「承诺内不丢」，它没造的形写在第 5 次重跑那一条里。
- **「只坏第一次读」这一族没枚举**（登记修订 4，代码轮第三轮判决 K4-a）：T1 在这一页里是「从这次挂载里第一次读那个落点起读不出、重读之前恢复」。只坏第一次读时，系统配置槽那一读被择系统配置吃掉，择系统配置可能择到另一槽、之后走另一条路，这一族一条都没造；Q1 的「承诺内不丢」不覆盖那一族。欠账由主 agent 记（判决里「记欠账，不建」）。
- 生成器的写行 txg 取条款允许的最小值（往下走那一遍读得出的根与记录的最大 txg 加一），实现取第三次直接读盘读到的根，可能更大；这一族的读故障不碰记录槽，造不出两者不同的态（最新的根与它那次发布的全部记录同时读不出又是 T1 才分得开）。
- 「承诺内」的三个标签是登记照条款文字落的读法：被判时 RI 不罩（一次让一个实例的全部根读不出，超出单故障）、历史里 RNSB 不罩（D22（单元原子性怎么合成） 已定项 8 写明不罩的双故障形）、RI 在历史里罩着（C331（择根倒挂压过已确认的写） 取甲为它而立）；改读法会改「承诺内」的态数。KJ1 与 KJ1T 在四个合法域逐态同判，这一族分不出两句读法；KG 上两句分得开（18 684 个态）。KG 不是合法历史，它的数不翻 Q1。
- **第 4 次重跑第二段 Q2 的「仍有不红的」只在这个界与这些摆法里**（同第一段 Q1 那一条：两块盘、区域归属 0 / 1 / 0、故障只在三类落点、闸取实现的取甲）：变体只改 `choose_root_slot`（`lib.rs`）与 `choose_root`（`recovery.rs`）两个函数里的比较式、一次改一处，别的写法没扫。不许写成「这三条择根序没有分别」或「与它无关」：v1823、v1825、v1826 在 KA 上红、v1826 在 KG 上也红，新一族的四个合法域对它们没有取样点；根槽落点只由 txg 定（I-7.17（根槽位置等于 txg 算出的位置））是推的读法，量到的只是两个根环形状计数在合法域为 0。要它们红得违反 I-7.17 的根环（KA）或拿掉条款保护（KG），那两格不是合法历史，红不算 Q2 的「红」。C657（带故障的择根合判族没有） 一行的验收是「三条变体要红」，这一次没达到（三个合法域都不红）；这一行怎么处置交主 agent。
- v1822（平局取后者赢）在六个域上都与基线同值：「取样点不敏感」是推的（存在同键不同内容的两个自证过的槽就不同值，这一族与 KA 都没造），没量成，不记等价变异留档。
- 第 4 次重跑第二段照取甲生成（登记修订 1），那一次 Q2 的读数只对取甲那一套历史成立；R2 下的同一族由第 5 次重跑 5-乙 重判（仍有不红的，与第 4 次同）。C662（带故障的合法历史一族照取甲生成） 那一行怎么处置交主 agent。
- v1824（tsv-1823）一行是 6 线程补跑的、别的五行是 3 线程跑的；第一趟 4G 撞顶（退 250）的那一次输出没用。驱动 `--selftest` 的输出（84 项、五个 r4 弄坏开关各红）没有入库文件，这一段没重跑；单测与装置变异整表仍没跑（checker 档，归提交时）；`crates/mutations.tsv` 里第 1821 至 1826 行各点名的单测抓不抓这一次没跑，这里只说 E165 的新一族没有把它们抓出来。
- 单测与装置变异整表没跑（checker 档的重型测试）；草稿副本上的等价检查只证单测体会红，TV5 按提交时跑的 checker-tier-crates-mutation-replay 判。
- 界外的输入：N 大于各域上沿、一条记录点名多个单元、空发布记录、多于两个实例、跨环一圈以上的覆盖，都没扫（登记 5.1 末段）。「0 违例」只说界内。
- 对抗域里序号 0 的记录：`parse` 拒收、进不了表，S 的 (e)「序号 0」在建出来的表上走不到；读者拿到序号 0 以外的坏记录怎么判，不在这个字母表里。
- 第二段的「首红的 N」是逐行与基线不同的第一个 N，不分方向：变体少了违例也记红（「逐行判定」Q13 的 Z1 那一行）。每个域红了就不往上看，红了之后别的判据在更大的 N 上红不红没量。
- 第 2 次重跑第二段补跑（2c）的 Q1 后半只比 J6 与 J6Qn 的首红 N（行的文字与基线同行比），不分方向；九格 J6 红而 J6Qn 界内不红，J6 红的那一半是 J6 记了「第二次顶格报错」这一类违例（形态一是原实现自己有、变体没有，形态二是变体新添），不说变体的缺陷在这一格被 J6 抓到。冻结根不是 git 仓，驱动开头收尾那两条 `git status --porcelain crates/` 都只打了「fatal: not a git repository」、输出同为空，`crates_git_status_unchanged=true` 在这棵根上没有判别力；跑的中途 `crates/` 变没变由 `sha256sum -c`（段首段尾 222 行 exit 0）与驱动的 recovery.rs、lib.rs 哈希开头结尾相同管。第二次调用的原始 `what` 没取到（红例账只存逐态行）。
- 第 448 行界内不红，是真盲区、等价变异还是取样点不敏感，登记写明不在这个实验里判（F18）。
- 红例账的方向只看首红那一个状态；整域上多了还是少了违例没有单独数。
- S 第 3 条 (b)「链上遇到水位之下的记录」取「停」：条款没逐字写，主 agent 2026-10-01 推的。合法历史一族里这一格走不走得到没有单独数。
- J6 只量了「不动点」这一种读法；K7 另外三种读法不在装置里。
- Q6 的乙同段「按停在记录段 / 停在根槽段分行」照字面分不出来（切法乙没有单独的根槽段），装置报的是「根槽那次写在两个状态上取值同不同」两行，够不够由主 agent 定。
- `replay_journal` 在 `crates/` 里的调用方除登记 5.3 写的三处之外，还有入库装置 `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`（登记第十三节列过）；三处调用除布尔开关与读者之外有没有别的差别没逐行核（停机条款 H5 只核到 grep 这一步）。

```

**出处 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1289-1305`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 支撑 | 2026-10-04 不受影响：第 5 次重跑没推翻分项依据引的读数——3-甲 里 r3 择根一族与 r4 带故障一族在 7-2 之后的根上与入库那两份整份逐字节相同，5-甲 在 R2 下带读故障的合法历史承诺内 KJ1、KJ1T 仍每格 0，5-乙 择根序变体的红与不红与第 4 次相同；3-丙 的 J8 判的是 D23（journal 的角色与格式） 已定项 15 的施加规则，不是 M1 至 M4；M3、M4 的前提、射程句与落点没动；M1 落点要不要补「R2 下带读故障的合法历史也一样」交主 agent。此前：2026-10-03 改了：M3 不动点的前提补 I-8.12（同实例记录的 txg 不减），依据引第 3 次重跑第一段（对抗一族违例全在该谓词不成立那一边）；M1 落点里「装置不调 `scan_journal` 与 `choose_root`」改成第 3 次重跑第二段的择根合判族调真三步，依据引第二段（界内、合法、不带故障时已确认的完整发布不丢）；M4 射程句与别的读数没动。2026-10-02 的回看：落点列与依据写回这个实验当轻量工具的那一次已改过；第 2 次重跑只加产物与判据（J6Qn、J7、多盘、建表法），那条分项依据引的读数一个没动；2c 补跑后重看：分项正文不引 J6、J6Qn 或首红的数；2026-10-03 第三段（Q2）回看：分项正文不引择根序变体的读数，M4 的前提、射程句与落点没动；Q2 的「没有」只说三个择根序变体在合法历史 K 族界内不红、在 KA 上红，不改 M4 落点写的「E165（恢复重放有界穷举与变异杀伤界） 的最远一版判据在界内判」；主 agent 判补写：M1 落点补「择根序的四个改坏变体在合法历史 K 族里只有取第一个有效者红，另三种由 Verus 必红世界与对抗根环 KA 罩」，依据 E165（恢复重放有界穷举与变异杀伤界） 那一条补第三段（调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 12 批 E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑第三段判 Q2 交回那一条）；2026-10-03 第 4 次重跑第一段回看：不受影响：正文 M4 射程写的「带故障的域里不排除——在飞上限会截掉记录完整的发布」与 M1 落点写的「界内、合法、不带故障时已确认的完整发布不丢」都没被这一段推翻（带故障的合法历史上承诺内 KJ1 与 KJ1T 也是 0，在飞上限没咬到已确认的那一版），要不要把带故障的读数写进 M1 的落点与 M4 的射程交主 agent；2026-10-03 第 4 次重跑第二段回看：不受影响：Q2 在新一族上的读数与 M1 落点已写的「择根序的四个改坏变体在合法历史 K 族里只有取第一个有效者红，另三种由 Verus 必红世界与对抗根环 KA 罩」同向——带读故障的四个合法域里也只有取第一个有效者（v1824）红，比较不看实例、平局取低、实例代号为主三个界内不红（仍只在 KA 上红，v1826 在 KG 上也红，KG 不是合法历史）；M4 的前提、射程句与落点没动；M1 落点那句要不要补「带读故障的合法历史也一样」交主 agent；2026-10-03 登记修订 4 回看：不受影响：只改 T1 一句的措辞（见历史版本 2026-10-03 一条），产物与条款读法没动 |
| D13（验证路线） 已定项 4 | 备料 | 2026-10-04 不受影响：第 5 次重跑的崩溃点切法照旧（r1 至 r4 档一个字没改，r5 档与 R2 那一族沿用同一份按设备切段），条款没动。此前：2026-10-03 不受影响：第二段 K 族的崩溃点按设备切段（一次 k 条记录的发布 4^k + 1 个崩溃点）照已定项 4 的字面，条款没动；第三段（Q2）的变体在同一族状态上跑，状态与崩溃点的切法没变，不受影响；2026-10-02 的回看：C 域合法态数补写两种切法的数，两种切法的字面照已定项 4 没动；2026-10-03 第 4 次重跑第一段回看：不受影响：新一族的崩溃点同样按设备切段（根落了之后两块盘的系统配置写各取没写、写了、撕裂三态，最后一次挂载并成一个），字面照已定项 4，条款没动；2026-10-03 第 4 次重跑第二段回看：不受影响：变体在同一族状态上跑，状态与崩溃点的切法没变，条款没动；2026-10-03 登记修订 4 回看：不受影响：只改 T1 一句的措辞（见历史版本 2026-10-03 一条），产物与条款读法没动 |
| D23（journal 的角色与格式） 已定项 4 | 备料 | 2026-10-04 不受影响：第 5 次重跑开工时核过头 487 字节、新根段 364 字节、每条记录点名项至多 64 个仍是 `crates/singlefs-format/src/lib.rs` 的常量值，生成的记录都过 `to_bytes` / `parse` 往返（停机条款 PH4 没触发），3-甲 里 J7 之外的判定一格没变（`parse` 新加的两判与反向链罩 487 字节头没让一条生成的记录读不出）；条款没动；这一格该不该升成支撑（依据段不引 E165（恢复重放有界穷举与变异杀伤界））交主 agent。此前：2026-10-03 不受影响：第二段 K 族的记录字段、提交标记与末条标志照已定项 4 写，条款没动；第三段（Q2）的变体只改择根的比较式，不改记录字段，不受影响；2026-10-02 的回看：第四格补句写回 D23（journal 的角色与格式），依据是三方判决、不引这一页的读数；这一页只补了 C 域合法态数的切法；2026-10-03 第 4 次重跑第一段回看：不受影响：新一族的记录字段与提交标记照已定项 4 写，没动；2026-10-03 第 4 次重跑第二段回看：不受影响：变体只改择根的比较式，不改记录字段与提交标记，条款没动；2026-10-03 登记修订 4 回看：不受影响：只改 T1 一句的措辞（见历史版本 2026-10-03 一条），产物与条款读法没动 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-04 不受影响：第 5 次重跑 r1 至 r4 档的链首、前缀口径与注 3 照旧，3-甲 里 J7 之外的判定没变；这一条「可写挂载读到的更新状态读不出」那一段写的「判不出取 R2」由 5-甲 照 D22（单元原子性怎么合成） 已定项 8 的权威写法生成，今天的三步在 R2 那一族承诺内不丢、与取甲那一点同判；J8c（一条没施加原样交回）照这一条「恢复只施加严格大于所选根的记录」在 r5 档每格 0；条款没动；这一格该不该升成支撑（依据段仍不引 E165（恢复重放有界穷举与变异杀伤界））交主 agent。此前：2026-10-03 不受影响：第二段的链首、前缀口径与注 3 的计数器、txg 取法照这一条写，今天的三步在界内与理想恢复逐格同值，条款没动；这一格该不该升成支撑（依据段现在不引 E165（恢复重放有界穷举与变异杀伤界））交主 agent；第三段（Q2）的变体只改择根序，链首、前缀口径与注 3 照第二段没动，注 2 那一处分歧的择根一侧在合法历史里仍造不出（KA 之外没有同 txg 不同实例的根），条款没动；2026-10-02 的回看：五条口径与注 1 没动，这一页只补了 C 域合法态数的切法；2026-10-03 第 4 次重跑第一段回看：不受影响：第一段的闸、写行 txg 与取号那一写照这一条今天的原文与实现的取甲写（原文写「判不出取 R2……这里的实现还是 2026-09-28 取甲的那一版」，生成器取实现的取甲，登记修订 1），今天的三步在带读故障的合法历史界内与理想恢复逐格同值，条款没动；C331（择根倒挂压过已确认的写） 改取 R2 的实现落地后这一族要重跑；这一格该不该升成支撑（依据段仍不引 E165（恢复重放有界穷举与变异杀伤界））交主 agent；2026-10-03 第 4 次重跑第二段回看：不受影响：变体只改择根序，链首、前缀口径与注 3 照第一段没动，注 2 那一处分歧的择根一侧在带读故障的合法历史里仍造不出（同 txg 不同实例的根同在计数四个合法域每格 0），条款没动；主 agent 派发这一段时判：备料格都不升支撑，这一族照取甲生成、C331（择根倒挂压过已确认的写） 改取 R2 进代码之后要重跑，账在 C662（带故障的合法历史一族照取甲生成）；2026-10-03 登记修订 4 回看：不受影响：T1 的「重读一次」（R = 1）引自这一条，生成器写的是「坏到重读之前」、实现在这种时机下走重读那一遍，条款与产物没动；改的只是登记与这一页里 T1 一句的措辞（见历史版本 2026-10-03 一条），「只坏第一次读」时系统配置槽那一读被择系统配置吃掉、这一族没枚举 |
| D16（发布语义） 已定项 7 | 备料 | 2026-10-04 不受影响：第 5 次重跑「已确认」的认法、发布末尾的系统配置轮换与被判时标签照第 4 次那一份（R2 只改闸与取号那一刻），条款没动。此前：2026-10-03 不受影响：第二段的「已确认」与崩溃点所在的段照这一条的持久顺序（fsync 等系统配置轮换持久之后才返回），条款没动；第三段（Q2）不碰「已确认」的认法与崩溃点所在的段，不受影响；2026-10-02 的回看：甲切法取这一条的屏障位置，条款没动；2026-10-03 第 4 次重跑第一段回看：不受影响：新一族的发布末尾系统配置轮换、「已确认」的认法与被判时标签（每个新实例的第一个根在下一次发布之前是单点、一个故障就够）照这一条的持久顺序与射程，条款没动；2026-10-03 第 4 次重跑第二段回看：不受影响：变体不碰「已确认」的认法与崩溃点所在的段，条款没动；2026-10-03 登记修订 4 回看：不受影响：只改 T1 一句的措辞（见历史版本 2026-10-03 一条），产物与条款读法没动 |
| D16（发布语义） 已定项 8 | 备料 | 2026-10-04 不受影响：第 5 次重跑的暖机计划照旧，条款没动。此前：2026-10-03 不受影响：第二段的暖机计划（写行那次发布的 txg 取环里最大加一、推到本实例的根覆盖两块盘、空发布至多 3 次，新池第一次挂载实付 2 次）照这一条写，回比断言对着格式常量 `WARM_UP_EMPTY_PUBLISHES` 与 `NEW_POOL_FILE_CREATION_TXG`，条款没动；第三段（Q2）不碰暖机计划，不受影响；这一格该不该升成支撑（依据段现在不引 E165（恢复重放有界穷举与变异杀伤界））交主 agent；2026-10-03 第 4 次重跑第一段回看：不受影响：新一族的暖机计划与第二段同一份，条款没动；2026-10-03 第 4 次重跑第二段回看：不受影响：变体不碰暖机计划，条款没动；2026-10-03 登记修订 4 回看：不受影响：只改 T1 一句的措辞（见历史版本 2026-10-03 一条），产物与条款读法没动 |
| D22（单元原子性怎么合成） 已定项 7 | 备料 | 2026-10-04 不受影响：第 5 次重跑 3-丙 的 J8b（返回根的 fsid 等于所选根的）与 J8a 卸载记号（这一条字段表里「正常卸载那一串写的根置 1，其余根写 0」与 D23（journal 的角色与格式） 已定项 15）在 r5 档每格 0；5-乙 改坏平局规则与主次序的三个变体在 R2 那一族四个合法域仍界内不红（与第 4 次同），这一族仍看不到「平局时实例代号高者赢」与「txg 为主」两句；条款没动，维持备料。此前：2026-10-03 不受影响：择新序（txg 为主、实例代号破平局）是第二段 KJ2 的判据，今天的择根在合法历史界内与它逐格同值，条款没动；第三段（Q2）：平局取低（1825）、实例代号为主（1826）、比较不看实例（1823）三个改坏这一条平局规则与主次序的变体，在合法历史 K 族界内不红、在 KA 上红（18、36、18），所以这一族看不到「平局时实例代号高者赢」与「txg 为主」这两句，它们靠 KA 与 `crates/mutations.tsv` 点名的单测，条款没动；主 agent 判维持备料（调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 12 批 E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑第三段判 Q2 交回那一条）；2026-10-03 第 4 次重跑第一段回看：不受影响：新一族四个合法域里今天的择根序界内与理想恢复逐格同值（KJ2 全 0），能让改坏这一条平局规则与主次序的变体露出来的两种根环（同 txg 不同实例同在、实例代号更高而 txg 更低同在）在合法域造不出（0），KG 里后者 129 420 个被判态；变体本身第二段才读，条款没动；2026-10-03 第 4 次重跑第二段回看：不受影响：Q2 读了：改坏这一条平局规则与主次序的三个变体（比较不看实例 v1823、平局取低 v1825、实例代号为主 v1826）在新一族四个合法域 410 个子格上与基线逐子格相同、界内不红，在 KA 上红（18、36、18，`ka_pin=match`），v1826 在拿掉条款保护的 KG 上也红（KJ1、KJ1T 8 个、KJ2 12 个子格，KG 不是合法历史）；所以带读故障的合法历史也看不到「平局时实例代号高者赢」与「txg 为主」这两句，它们仍靠 KA 与 `crates/mutations.tsv` 点名的单测，条款没动；主 agent 派发这一段时判备料格都不升支撑，维持备料；2026-10-03 登记修订 4 回看：不受影响：只改 T1 一句的措辞（见历史版本 2026-10-03 一条），产物与条款读法没动 |
| D22（单元原子性怎么合成） 已定项 24 | 支撑 | 2026-10-04 不受影响：第 5 次重跑 3-乙、5-乙：取第一个有效者（v1824）在 r3 模式与 R2 那一族四个合法域照旧红，首红位置与第 3、4 次相同，与依据段引的那一条同向；条款与依据段不改。此前：2026-10-03 改了：依据加这一页一条（第 3 次重跑第三段：取第一个有效者在 K、K9、KW 界内红），主 agent 判升支撑（调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 12 批 E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑第三段判 Q2 交回那一条）。此前：2026-10-03 不受影响：「先逐个验证全部候选、再在有效者中择新」的读取次序是第二段 KJ1、KJ2 要看的一端，条款没动；第三段（Q2）：取第一个有效者（1824）在 K、K9、KW 界内红（KJ2 首红 M1U0、KJ1 首红 M1U2；KW 的 U11 至 U14 四格不红），这一条读取次序在合法历史界内被这一族看着，条款没动；2026-10-03 第 4 次重跑第一段回看：不受影响：依据段引的是第 3 次重跑第三段取第一个有效者红那一条，第一段没跑变体，条款没动；2026-10-03 第 4 次重跑第二段回看：不受影响：取第一个有效者（v1824）在带读故障的四个合法域也红（`red_legal_domains=KF,KF3,KF9,KFW`，KJ2 首红 M1U0、KJ1 与 KJ1T 首红 M1U2，KFW 的 U11 至 U14 四格不红），与依据段引的第 3 次重跑第三段同向；这一条读取次序在带读故障的合法历史里也被这一族看着，条款没动，依据段不改（它引的那一条已经是这个实验），要不要追加新一族这一条交主 agent；2026-10-03 登记修订 4 回看：不受影响：只改 T1 一句的措辞（见历史版本 2026-10-03 一条），产物与条款读法没动 |
| D22（单元原子性怎么合成） 已定项 8 | 备料 | 2026-10-04 不受影响：第 5 次重跑 5-甲、5-乙 照这一条射程的 R2（判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过；某块盘一份自证过的系统配置都没有时取号那一步也拒；S-辨把撕裂与读不出分开）生成带故障一族：承诺内不丢（Q1（R2）），三个择根变体仍界内不红（Q2（R2）），两样都与取甲那一点同判；「容错承诺是单故障」那几句照旧当「承诺内」标签的依据；条款没动；这一格该不该升成支撑（依据段不引 E165（恢复重放有界穷举与变异杀伤界））交主 agent。此前：2026-10-03 不受影响：容错承诺是单故障（每盘一槽系统配置坏在承诺里，每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里）是第 4 次重跑第一段「承诺内」历史标签的依据，被判时标签也照单故障的读法，条款没动；这一格该不该升成支撑（依据段现在不引 E165（恢复重放有界穷举与变异杀伤界））交主 agent；已定项 8 射程今天写了「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（R2）」，第一段的闸取实现的取甲、不是这一句，R2 落地后这一族要重跑；2026-10-03 第 4 次重跑第二段回看：不受影响：变体只改择根比较式，不碰闸与系统配置，条款没动；Q2 的读数同样只对取甲那一套历史成立，R2 落地后要重跑（账在 C662（带故障的合法历史一族照取甲生成））；主 agent 派发这一段时判备料格都不升支撑，维持备料；2026-10-03 登记修订 4 回看：不受影响：系统配置槽落点的 T1 读法按「坏到重读之前」改写措辞（生成器与产物一直是这一种），「只坏第一次读」时系统配置槽那一读被择系统配置吃掉、择系统配置可能择到另一槽，这一族没枚举；条款与产物没动 |
| D22（单元原子性怎么合成） 已定项 16 | 备料 | 2026-10-04 不受影响：第 5 次重跑系统配置轮换与世代号的写法同第 4 次（R2 只改「读得出」的认法：撕裂的槽不算自证过），条款没动。此前：2026-10-03 不受影响：每盘两个系统配置槽、世代号从 1 起、写落的槽 = 世代号对槽数取余照这一条写进生成器（系统配置写的世代号取这块盘两槽里读得出的最大世代号加一），条款没动；这一格该不该升成支撑（依据段现在不引 E165（恢复重放有界穷举与变异杀伤界））交主 agent；2026-10-03 第 4 次重跑第二段回看：不受影响：变体不碰系统配置的轮换，条款没动；主 agent 派发这一段时判备料格都不升支撑，维持备料；2026-10-03 登记修订 4 回看：不受影响：轮换与世代号的写法没动，改的只是系统配置槽落点上 T1 一句的措辞（见历史版本 2026-10-03 一条） |
| D23（journal 的角色与格式） 已定项 15 | 备料 | 2026-10-04 不受影响：第 5 次重跑 3-丙 照这一条的原句（新根段七项、「施加一条记录 = 把所选根的这七个字段换成记录里的」、一次发布每条记录带同一份新根段）重新登记成 J8，今天的施加在 r5 档三族、两种奇偶 830 格每格 0；3-甲 里旧档 J7 的变化就是这一条把投影外三项改成从记录取的结果；条款没动；依据段不引 E165（恢复重放有界穷举与变异杀伤界），这一格该不该升成支撑交主 agent |
| D2（RAID 条带策略） 已定项 13 | 备料 | 2026-10-04 不受影响：第 5 次重跑不碰设备数下限与历史里「盘不在」的生成，条款没动。此前：2026-10-03 不受影响：可写挂载的设备数下限（w = 2）是第一段历史里不造「盘不在」的依据（可写挂载在读盘之前就拒，那次挂载在盘上是空操作），回比断言对着 `STRIPE_WIDTH_LOWER_BOUND`，条款没动；这一格该不该升成支撑（依据段现在不引 E165（恢复重放有界穷举与变异杀伤界））交主 agent；2026-10-03 第 4 次重跑第二段回看：不受影响：变体不碰设备数下限与历史的生成，条款没动；主 agent 派发这一段时判备料格都不升支撑，维持备料；2026-10-03 登记修订 4 回看：不受影响：只改 T1 一句的措辞（见历史版本 2026-10-03 一条），产物与条款读法没动 |

```

**出处 `.claude/kb/invariants.md:1-8`（整段抄，未转述）**

```markdown
# 不变量清单

**checker 是 invariants.md 的可执行形式。** invariants.md 每加一条，checker 就要加一个检查。
两者不同步的 commit 一律不收（见 [format-evolution.md](../rules/format-evolution.md)「硬约束」）。

每条不变量必须写成**可判定**的形式——能对着一个镜像回答"成立/不成立"。池级 checker 对一条不变量的判定有四种：成立、不成立、不适用（必须带理由，不许静默算成立）、部分没判（`InvariantVerdict::PartiallyJudged`，`crates/singlefs-checker/src/image.rs`）。部分没判是判到的那一部分都对得上、射程里另有一部分这张镜像上判不了，说明逐条列出没判到的是什么；它不是红，也不是成立：层 0 与崩溃放量把它算成不红，层 0 计数并进「不适用」那一格，GPU 判器只记违例位、它在那一侧与成立、不适用同是 0。今天报它的不止一条，逐条见各行（例：I-5.4（分配记录罩住的槽互不相交）、I-5.5（从 mkfs 水位建树那一版的 t1 落点）、I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））。
写不成可判定形式的，说明还没想清楚，不许进 invariants.md。

```

**出处 `.claude/kb/invariants.md:9-20`（整段抄，未转述）**

```markdown
## 状态

**新池新建文件阶段（2026-09-14 起）。** 磁盘格式已落成字节：mkfs、新池新建文件的写路径与冷启动恢复在 `crates/singlefs-core`；checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）。
池级 checker（`walk::check_pool_image`）判 84 条（`crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 的长度；状态列写「已实现」的行用 `grep -c '^| I-.*已实现' .claude/kb/invariants.md` 数，也是 84 条，其中 I-3.5（引用区间的精确性） 是部分实现：出现规则已实现（下标 65），③ ④（deadlist 那两条）未实现，I-4.4（意图只描述目标态） 在清单里（下标 73）、第一版恒报不适用；第一版 23 条，2026-09-16 里程碑「覆盖写、释放、回退与复用」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符），同日加 I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序），2026-09-28 加 I-7.14（系统配置本盘设备号等于盘在池里的身份），2026-10-02 加里程碑三二·1 那一批 16 条（I-1.12（指针出生树等于被指单元头）、I-3.12（实例表行 T 低于引用它的根的 txg）、I-3.13（记账行代段等于这一版的 txg）、I-4.1（重放后树可遍历）、I-4.9（非最新候选根引用的单元不晚于它）、I-7.5（根槽按判定宽度对齐）、I-7.15（各盘所选系统配置槽设备数相同）、I-7.16（系统配置槽世代号与槽号的关系）、I-7.17（根槽位置等于 txg 算出的位置）、I-8.1（环几何够大）、I-8.5（tail 槽有效集非空）、I-8.10（记录新根段与同代根相同）、I-8.11（点名项与被指单元头相同）、I-8.12（同实例记录的 txg 不减）、I-9.25（树表条目的 previous_snapshot_txg 取值）、I-9.26（inode 改动计数落在出生代与容器诞生代号之间）），2026-10-04 加 I-9.27（树表条目的头 ID 取值）、I-5.5（从 mkfs 水位建树那一版的 t1 落点） 与 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），2026-10-05 加里程碑三·3 删除线那五条：下标 69 的 I-9.5（删除队列里的号仍在树里）、下标 70 的 I-3.4（可用空间扣待删占用）、下标 71 的 I-4.7（删除中 inode 有账）、下标 72 的 I-4.6（中间态其余不变量成立） 与下标 73 的 I-4.4（意图只描述目标态），2026-10-06 加下标 83 的 I-3.14（实例表每片记录形态合法）），状态列写「已实现」的就是这 74 条（I-3.5（引用区间的精确性） 只算出现规则那一半，I-4.4（意图只描述目标态） 的判定恒是不适用）：除 I-4.4（意图只描述目标态） 外每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`，删除线那几份的搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`），层 0 的每个崩溃状态都跑它。
I-7.7（系统配置实例代号不低于根环） 2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 定案改成 ① ②，层 0 里 0 个状态判红；I-3.1（已分配统计对得上） 与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 的 checker 读法 2026-09-14 由用户收尾弹窗定甲，写在各自那一行。
其余条目仍写「未实现」：它们要覆盖写、释放、回退与复用（释放、重用、残留）、journal 一侧的那一类检查、快照或加密，新池新建文件的镜像上判不出东西。

违例位是 128 位（4 个 u32 字，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的 `POOL_CHECKER_VIOLATED_BIT_CAPACITY`），位 i 对 `IMPLEMENTED_INVARIANTS` 的第 i 条（`pool_checker_violated_bits` 读成 u128）；今天用了 74 位；`POOL_CHECKER_COVERED_INVARIANT_INDEXES` 列 74 个下标，就是 `IMPLEMENTED_INVARIANTS` 的每一条（下标 65 的 I-3.5（引用区间的精确性） 走读内核在回退候选集的版本都走完之后逐对判；66 是 I-9.27（树表条目的头 ID 取值）、67 是 I-5.5（从 mkfs 水位建树那一版的 t1 落点）、68 是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）；69 是 I-9.5（删除队列里的号仍在树里）、70 是 I-3.4（可用空间扣待删占用）、71 是 I-4.7（删除中 inode 有账）、72 是 I-4.6（中间态其余不变量成立），这四条由扫描内核的删除态扫描在回退候选集的每一版上判，读不全的版不置位；73 是 I-4.4（意图只描述目标态），第一版池级 checker 恒报不适用、内核不置位），没有「内核还没判」的那张表；I-9.6（水位大于两处最大号） 墓碑那一半也在扫描内核里判、并进下标 41 那一位；编译期断言覆盖表按下标升序、一条不缺一条不重（第 k 项是 k）（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行判定的位宽加宽与 GPU 内核补齐）。

<!-- invariant-count -->
现共 105 条在用（编号至 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），另有六条退役，编号不回收）。这一句是当前条数的权威登记位；「## 历史版本」里那几句「现共 N 条在用」记的是当时的数，不跟着改。

```

**出处 `.claude/kb/invariants.md:21-47`（整段抄，未转述）**

```markdown
## I-1 块自描述 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-1.1 | 块头自述逻辑地址 | 任一单元，其头记录的**类内身份**与实际引用一致——数据单元的身份是五元组（锚点偏移即它的逻辑地址）；索引节点是树 ID + 层级（+ 该树已定携带的 key 区间，D18（块里携带什么信息）已定项 2 口径）；打包记录单元（码 3）是（出生树 ID, 打包记录类型, 容器号, 容器出生代）四元组，「父级」按打包记录类型分——类型 1 走容器索引的身份引用（C103（容器索引无落点）），类型 2 走 inode 树内部节点条目的身份引用（I-9.2（条目身份与子头相符）），**类型 4（实例表）走根记录的物理指针，不属于任何树、不进容器索引**。**自证单元（journal 记录 / 根槽 / 系统配置槽 / 类型 4 实例表单元）显式豁免**：无父引用无树身份，自证纪律在 I-7（根环）/ I-8（journal）两类与 D22（单元原子性怎么合成）已定项 21；多层码 2 树（记账树、中央映射树）的索引节点，身份里的层级与 key 区间按引用它的父条目核：层级 = 父 − 1、区间落在父条目给的那一段、内部节点区间是子树覆盖区间；按位置寻址的两棵树（分配记录树、extent 树），节点头里的层级与 key 区间等于它的位置规定的那一层、那一段；内部条目的 key 是某个孩子那一段的起点，孩子按位置严格递增；分配记录叶里每条记录的起点与末槽都落在叶罩的那一段里；extent 上段叶条目的 inode 号、下段叶记录的单元号落在叶罩的那一段里，上段叶条目的标签只能是 0、1、2（标签 0 载荷全零、标签 1 载荷末 2 字节为零）；码 2 节点里的条目按 key 严格递增，同 key 两条判红（`deleted_inodes`（D8（核心索引结构） 已定项 16）里同一个号两条就归这一条，不归 I-9.24（删除队列至多一条 key））；根之下没有空节点（这一半 checker 由实二五补）。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；`walk::Walk::walk_code_two_subtree`；坏镜像用例 `the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root`；`crates/singlefs-checker/src/position_addressed.rs`；码 2 节点 key 严格递增由 `crates/singlefs-checker/src/lib.rs` 的 `check_index_node_keys` 判，`deleted_inodes` 同号两条的坏镜像 `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs` 的 `a_deletion_queue_holding_one_number_twice_reddens_only_the_key_order_invariant`） |
| I-1.2 | 块头写序已发布 | 任一单元，其头记录的写序 (i, n)（D18（块里携带什么信息）已定项 7 / 11 的字段）与诞生代号 b 按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的全局已发布谓词判成已发布：i < 挂载根实例且无该实例的行 ⇒ 已发布，有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub（所选根那个实例的 T_pub = 重放之后那个根的 txg，严格介于所选根的实例与新实例之间、又不是切换照旧事务写序主人的实例写 (i, 0, 0)，切换的所选根按 D23（journal 的角色与格式） 已定项 14 取被重发的在飞 checkpoint 所基于的根；W = **属于实例 i 的**、被这次重放施加的最大事务号，切换时是被照旧的、写序属于 i 的最大事务号；同一次恢复写出的那批行里只有所选根那个实例的 W 可能非 0，同一次切换写出的那批里只有被照旧事务的写序主人的 W 可能非 0，表上允许有多行非 0，它们来自不同次的恢复或切换；2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；i == 挂载根实例 ⇒ b ≤ 挂载根的 checkpoint_txg；i > 挂载根实例 ⇒ 判损坏。**例外**：打包记录类型 4 的实例表单元不走这条谓词，它的现行版本由根记录持有的物理指针唯一确定。**判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留**——这是扫描方向独立于遍历方向判它的依据（D13（验证路线） 已定项 12 双方向要求）；2026-09-05 前它只比诞生代号与当前 txg，崩溃后 txg 重发让那条判据对同 txg 两条时间线失效（E104（扫描重建的现行版本判定）：无实例表时崩溃世界复活 6） | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`，**只判遍历方向那一半**：走读到的每个被引用单元，头里的诞生代号 b、写序的实例代号 i 与尾段（码 1 是写序的事务号 n、码 2 / 码 3 是出生序号）要与**引用它的那条指针**记的相同——不同则「头记录的写序」不是这个单元的真实身份，谓词判的就是假输入；口径同恢复路径今天的前置检查（`crates/singlefs-core/src/recovery.rs` 五处报 I-1.2（块头写序已发布） 的 `InvariantViolated` 里「出生身份与指针不符」那三处；「诞生于根之后」那两处是谓词那一半，归 I-4.2（无被引用未提交块）），checker 按字段表自己解一份。⚠️ **扫描方向那一半（「任一单元」都按谓词判、i > 挂载根实例 ⇒ 判损坏）对着一份崩溃镜像判不了，没写**：① 判不成已发布本身不是违例——本行判据自己写着那是未发布的撕裂事务垃圾或被抛弃时间线的残留；② 「i > 挂载根实例 ⇒ 判损坏」只在恢复刚发布完新根的那一刻成立，而上一次挂载取了号、写出单元、一个根都没发布就崩时，盘上合法地留着 i > 最新根实例的在飞单元，那一支今天由 I-7.7（系统配置实例代号不低于根环） ① 按各盘系统配置的实例代号兜着。谓词本身照旧在用：`PublishedPredicate` 过滤 I-1.8（归并后版本全序） 的射程、判 I-4.2（无被引用未提交块）。打包记录类型 4 的实例表单元按本行的**例外**只免谓词那一半：根记录偏移 170 的实例表指针与链上每一片，指针里的出生 txg、实例代号、尾段与被指单元头里的值相等，仍挂这一条的遍历方向（已实现：`walk::Walk` 的 `judge_birth_identity_against_the_pointer`，被引用单元与实例表链上每一片都判；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）。坏镜像是 extent 树根头里的出生序号 +1、引用它的指针一个字节不动（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它，走读一个被引用单元都读不到的状态报「不适用」） |
| I-1.3 | 块头树 ID 一致 | 任一单元，其头记录的所属树 ID 与实际引用它的树一致。中央映射树不进树表，「实际引用它的树」取根记录（或记录新根段）里中央映射树根指针带的出生树，不写死 65540：回退到无文件那一版之后再发第一个文件版本，十二棵树从水位起重新发号（D8（核心索引结构） 已定项 8 ②），映射树的号随之变（主 agent 2026-09-23 定，C511（回退到无文件那一版之后诞生代怎么接） 第 3 步实现员查出写死 15 的读法在合法的回退历史上误红）。自证单元（含类型 4 实例表单元）与 I-1.1（块头自述逻辑地址）同口径显式豁免。⚠️ 打包记录单元（码 3）与被快照共享的数据单元头里的树 ID 是出生树，跨头共享时与引用树不同，那一情形的判法随 D9（加密） 那次「绑出生快照」的重开（C110（跨头共享与加密后元数据类的块头一致性读法））。这一射程覆盖码 3 全部打包记录类型，含类型 2（inode 树的叶，D8（核心索引结构） 已定项 6）：克隆头共享 origin 的叶时，叶头的出生树是 origin | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.4 | 块头 fsid 一致 | 任一单元，其头记录的 fsid == 系统配置 fsid（D18（块里携带什么信息）已定项 7 的字段；加密开启后与 KDF 绑定冗余，接受）。scrub 半边「本卷全部单元归属指向本卷」靠它才写得出（C74（无加密期的跨卷错向写没防线）补救的另一半） journal 记录头 2026-09-14 起也带 fsid 8（系统配置 fsid 低 8 字节，D23（journal 的角色与格式） 已定项 4）：fsid 不符的记录整条不进重放前缀、按断链处理（E142（新池新建文件的干跑） 第七次跑记的 G25） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.5 | 系统配置记管道身份 | 系统配置记录写出该镜像的管道身份（实现标识 + 版本），且非空。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：自举头段「写入者身份 20」= 实现标识 16 字节 ASCII 零补齐 + 版本 4 字节 u32，第一版写 `singlefs-rs` / 1；实现标识全零判红 | 未实现（条款已写、检查仍欠） |
| I-1.6 | 类标签登记且三处相等 | 任一单元，共同前缀偏移 6 的单元类型标签 ∈ D18（块里携带什么信息） 已定项 11 的登记表（码 0 与未登记码即判损坏）；偏移 7 的 8 个 flag 位第一版恒 0，非 0 一律拒收（前向兼容政策，D15（格式冻结政策） 判定表）；数据单元（码 1）五元组首字节 == 偏移 6；打包记录单元（码 3）类身份段首字节 == 偏移 6 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.7 | 打包容器合法与判定顺序 | 任一打包记录单元（码 3）：记录数 × 记录宽 ≤ 声明长度且声明长度 ≤ 32768 − 136（含 29 字节预留位的头，D18（块里携带什么信息） 已定项 16；2026-09-14 预留位从 28 加到 29，`cap(136, 40) = ⌊32632 / 40⌋ = 815` 与按 135 算同值，40 是第一版墓碑记录宽，D18（块里携带什么信息） 已定项 24），**先于**载荷校验和判（D23（journal 的角色与格式） 已定项 1 的 B 条同一纪律）；判定顺序恒为头校验和 → 类身份段 → 记录宽 == 登记表为该打包记录类型登记的宽（类型 2 是 140，D8（核心索引结构） 已定项 6；类型 1（墓碑）是 40、类型 5（谱系）是 16，D18（块里携带什么信息） 已定项 24；不等判损坏，只需头字段）→ I-1.2（块头写序已发布） / I-1.3（块头树 ID 一致） / I-1.4（块头 fsid 一致） → 边界 → 载荷校验和 → 逐条记录，任一步失败即中止；打包记录类型 4 的两种 kind 记录另按 D18（块里携带什么信息） 已定项 11 的实例表字段表判，不套用「记录宽 == 登记表的宽」那一步（一片记录宽恒 88，条数上限由片容量给——D18（块里携带什么信息） 已定项 11 2026-09-13 从 64 改宽）（对象级的判定——I-9.7（记录三段恒零）、I-9.8（记录 inode 等于 key）——在这条中止链之外，只让那条记录 EIO）；容器头声明一种打包记录类型、一棵出生树；打包记录类型 ∈ 第二级登记表，未登记类型的容器其物理指针不得出现在任何回收 / 复用操作的输入集合 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；类型 1、类型 5 两种记录宽 2026-10-05 删除线第 22 批补坏镜像） |
| I-1.8 | 归并后版本全序 | 码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组（字段表在 D18（块里携带什么信息） 已定项 7（码 1）与已定项 11（码 3）；归并键取这些字段但去掉载荷校验和那一段——它正是 ①「同一组的成员载荷相同」要比的那个量，进了键 ① 就恒真）之后，同 key 的各组两两全序键不等——码 1 的键是写序，码 3 的键是 (诞生代号, 实例代号, 事务号) 三元——诞生代号打头（同一实例连着两个 checkpoint 都不分配新事务号时两版写序逐字节相同），事务号留在末位做同一 checkpoint 内的平局破除（固定点会迭代，同一容器在一个 checkpoint 里不保证只写一次）；同一组的成员载荷相同（码 3 由载荷校验和判、码 1 由载荷 CRC 判），不同即判损坏、不择（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 副本归并，射程 w = 2；码 2 不进这条） | 已实现（2026-09-21，池级 checker `walk::check_pool_image` 判 ① 同一组的成员载荷相同；② 同 key 的各组两两全序键不等**只判码 3**——码 1 的键停着等 C464（I-1.8 对码 1 的全序键写窄了） 定案，checker 里显式写成 `TotalOrderKey::UndecidedForDataUnit`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-1.9 | 清扫水位内无可读垃圾 | 清扫水位（D5（快照 / 空间记账机制） 已定项 4 第 13 项）以内不存在可读垃圾；其外的可读垃圾每个 key 都被更大写序的可见版本或可见墓碑压住（管理员回退之后的镜像不在此列，见 C615（扫盘重建对挂着时的回退没有输入），出处 `research/prompts/m3-tomb-r1-main-verification.md` 第二节；C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4；扫描重建的产物未晋升期间清扫禁止，这条在重建之后失效） | 未实现 |
| I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树还不判**：D19（块指针的结构与宽度预算） 已定项 10 已定映射树 key 一律 27 字节（码 2 / 码 3 末尾补零）、条目一律 55 字节，但读者只拒窄于 55 的条目，`field_table_width_of_tree_kind` 不认种类 5，宽于 55 的映射节点不判损坏，账在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条 2026-09-27 实审 A3-checker-2 落地：`index_node_view_judging_a_zero_entry_width`（`crates/singlefs-checker/src/walk.rs:747`）在解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表（`EntryFieldTableRegistration::Registered`）时报违例；树表条目与中央映射树不在射程内，只记走读失败；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
| I-1.11 | 映射 key 与单元头相符 | 任一中央映射条目，key 的类标签属于 D18（块里携带什么信息） 已定项 11 的登记表，且等于它的位置条目指的单元头里的类标签；key 里的出生 txg、实例代号、尾段（码 1 是写序的事务号 6 字节；码 2 / 码 3 是出生序号 4 字节，其后 2 字节补零恒 0）分别等于那个单元头里的诞生代号、写序实例代号、事务号或出生序号；key 里的出生树等于那个单元头里的出生树（码 1 / 码 3 取类身份段偏移 43，码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定）。不等判损坏。 | 已实现（2026-09-27，池级 checker `walk::check_pool_image`，判定在 `crates/singlefs-checker/src/walk.rs` 的 `walk_central_mapping_entries`；key 各段的偏移按 D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 的字段表另写一份，不用实现的解析；判到的射程：类标签在登记表里且等于被指单元头的，出生 txg / 实例代号 / 尾段等于单元头的，码 2 / 码 3 尾段后 2 字节补零恒 0，码 1 / 码 3 的出生树等于单元头偏移 43，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不判；映射根全零（无文件的一版）时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（映射里数据单元那条 key 的出生 txg 3 → 4）与 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`。编号已落到代码：`crates/singlefs-checker/src/image.rs:38` 的常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 现在写的值就是这个编号本身，I-1.11（映射 key 与单元头相符）） |
| I-1.12 | 指针出生树等于被指单元头 | 任一被走读的指针，偏移 34 的出生树等于被指单元头里的出生树：码 1、码 3 取头偏移 43；码 2 的出生树取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定，不判（只写进状态列）。指针各段落点依据 D19（块指针的结构与宽度预算） 已定项 8（指针携带被指单元映射 key 除类标签外的各段）。inode 内部条目到叶容器那一类由 I-9.2（条目身份与子头相符） 罩住，不重判。 | 已实现（2026-10-02，池级 checker `walk::check_pool_image` 走读里的 `judge_pointer_birth_tree_against_the_unit_header`，只在两处调用：实例表链上每一片、extent 数据单元；码 1、码 3 取头偏移 43，码 2 不判，码 2 那一支不登记成「判了一部分」，那会让每份干净镜像整条报不适用；没有走到这两处指针的镜像报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C294（指针里的 key 各段与被指单元头相等没有不变量）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行） |

> I-1（块自描述） 类的适用范围是**任一单元**（数据与元数据都在内，
> D20（承重面：单元的原子性与自包含） 口径），判定字段的落点与宽度在
> [decisions.md](decisions.md) D18（块里携带什么信息） 已定项 7 的字段表（码 1 / 2）与已定项 11 的类身份段表（码 3）；
> 自证单元（journal 记录 / 根槽 / 系统配置槽）对 I-1.1（块头自述逻辑地址） 与
> I-1.3（块头树 ID 一致） 显式豁免，豁免的依据是它们的自证纪律另有落点
> （I-7（根环） / I-8（journal） 两类与 D22（单元原子性怎么合成） 已定项 21）。
> ⚠️ **豁免行是穷举的，按 D18（块里携带什么信息） 已定项 11 的登记表**：码 1 / 2 / 3 在字段表里有类身份段，码 16 / 17 / 18 在豁免行里点名，
> 其余码即判红（I-1.6（类标签登记且三处相等））——这是「穷举的显式分支」作用在不变量射程上。

```

**出处 `.claude/kb/invariants.md:48-82`（整段抄，未转述）**

```markdown
## I-7 根环 —— 类

**依赖 D22（单元原子性怎么合成），形态未定。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-7.1 | 根槽有效集非空 | 全部根槽位中，**自证校验和通过**的记录集合 S 非空 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.2 | 最新根可完整遍历 | S 中代号最大者，其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验，能走完全部权威态（[decisions.md](decisions.md) D21（权威态与派生态的分界） 定义：单元 + 记账），无一处失配 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.3 | 环健康性 | **环健康性**：S 中除代号最大者外，至少还存在一条更早代号的记录。不存在即判红——它意味着某次提交把上一代直接覆盖了，轮换逻辑已失效，**下一次撕裂将无路可退**。**例外只有一个**：S 全部是第 0 代（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成）已定项 8）时判绿，崩溃后回退到这个态的镜像同样判绿。⚠️ 射程：种子没被覆盖完之前，它们自己就充当「更早代号的记录」，连续两代落进同一槽的轮换 bug 要等 R × S 次发布把种子全部覆盖之后才会被这一条抓到 | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，例外（自证过的根全部是第 0 代）照判据原样实现；坏镜像建在 mkfs 刚写完那份上（三条第 0 代根的 checkpoint_txg 一起改成 1），例外那一支另有一份阳性对照；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。**「K 代」2026-09-13 起就指这个候选集**：平时回退下界 F 不动时是整个根环，盘紧时抬 F 缩到最近 4 个可退到的状态、且每块盘上最新的持久有效根都在内（D16（发布语义） 已定项 1）；原「K 是运行时策略、下限 ≥ 2」作废。[decisions.md](decisions.md) D22（单元原子性怎么合成）：**根环深度是块重用延迟的上界**（D22（单元原子性怎么合成）同时明令块复用的界不能是格式常量），不满足则环里的旧根是假的回退候选。「有效」按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的回退候选集判：(i, T) 有效 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被抛弃时间线的根不在候选集里，这只管回退目标的选择；被抛弃时间线今天只由崩溃恢复造出（管理员回退是挂着时的一次向前发布，不抛弃任何根，D23（journal 的角色与格式） 已定项 14）。它们引用的块在离开根环之前同样不许重新分配、不许抹头（2026-09-13 用户定案，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账，D23（journal 的角色与格式） 已定项 14；留着的理由是第一轮判决 H6：崩溃恢复落到旧根之后，被抛弃的根引用的单元被复用、之后恢复落在它上面读不出）。⚠️ F_生效 按 D16（发布语义） 已定项 1「生效」取根上与系统配置里读得出的最大值；checker 今天只读根上带的 F，带 F 的根全坏时会在合法状态上误红这一条与别的按回退候选集判的几条（第三轮判决 Y7，推的，没实现），欠 C556（checker 与层 0 不读系统配置里的 F）。崩溃后 defer 队列的内存态丢失，接手的是清扫准入「释放代 ≤ max(F_生效, 环里最旧有效根)」那条闸（谓词由 D16（发布语义） 已定项 1 给，2026-09-13；有效性按实例表判），保护宽度随之变 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，按每条回退候选集里更早的根各判一格：走读它引用的单元时校验和对不上或头用不了，就是它指着的块被复用或抹头；坏镜像是 mkfs 树表单元（只有第 0 代根还引用）改内容重封；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 被抛弃、还在根环里的根也判（2026-09-27，用户定「checker 现在就补被抛弃根那一半」，代码审阅第 7 条）：最新根的实例表有行时，根环里每条被判抛弃的根（有行 (i, Ti, ·) 且 T > Ti）各用一份自己的走读走一遍，只看读回来对不对——走出 I-2.1（校验和与内容匹配） 违例或走读失败就按这条根判红，判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（`walk::judge_blocks_referenced_by_abandoned_roots`，用例在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`）；根环有读不出的槽且最新根的实例表有行时，没有违例就整条报不适用、不报成立；C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一形在这一半上照实红，等 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 修好转绿。 |
| I-7.5 | 根槽按判定宽度对齐 | 每个根槽的**起始偏移**是判定宽度的整数倍，判定宽度 = max(声明的 `physical_block_size`, 声明的 `io_min`)（`io_min` 为 0 视为没声明）；mkfs 在任何写之前先拒不按这个宽度对齐的几何，非常规几何由 mkfs 拒、不再是合法镜像。**逐槽判，不是只判第 0 个**——偏移半个槽的槽照样跨 2 个原子单元（E41（根环槽几何 2×4×256） 实测：对齐时风险槽数 1，偏移 256 时是 2），撕裂会连带打掉相邻的槽而没有任何东西报警 | 已实现（2026-10-02，池级 checker `judge_root_slots_aligned_to_the_decision_width`（`crates/singlefs-checker/src/image.rs`）逐槽判：判定宽度取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的（异构池取最大，`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 1 条），是声明值、注明「声明值，未探测」，探测比对仍欠 C388（checker 的判定宽度不自己探测）；mkfs 在任何写之前拒不按这个宽度对齐的几何（`crates/singlefs-core/src/make_filesystem.rs` 的 `first_slot_not_aligned_to_the_decision_width`）；异构池一份与逐槽判的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 5 行） ⚠️ **射程注（2026-09-13，D2（RAID 条带策略） 已定项 15）**：`io_min` 大于 `physical_block_size` 时按 `io_min` 判——已定项 17 收窄后根槽与系统配置槽在它的射程内。 |
| I-7.6 | 根环区域落在互不相同的盘上 | R 个区域**在 mkfs 时逐区域写下的设备身份**（D2（RAID 条带策略） 已定项 7 用户定案逐字「归属不再按 `(r × P) mod devs` 算，mkfs 时逐区域把设备身份写下来」）取满 min(R, devs) 个不同值，且没有一块盘背超过 ⌈R / devs⌉ 个区域（鸽笼下界，D22（单元原子性怎么合成） 已定项 14，2026-09-13 用户定案）：devs ≥ R 时就是两两不同；devs < R 时不拒绝挂载，容错承诺按盘数降级（E87（固定结构的放置） 量过的 2 盘 R = 3：掉盘 1 最坏 1 代、掉盘 0 最坏 2 代）。取值不满或某盘超过下界即**拒绝挂载**——名义容错 F 个失败域而实际区域挤在同一块盘上，是 E47（根环失败域的损失） 实测过的形态（`even_span` 在 18 个几何格里只有 3/18 真的落到 4 块不同盘） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-7.7 | 系统配置实例代号不低于根环 | 可写挂载独占打开成功的那个集合里（它至少是过半，过半是准入门槛不是写入范围），每块盘的系统配置实例代号按它两槽里全部自证过（校验和过且 fsid 与本池相同）的槽读，取号用同一个读法（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者，任一根环区域、journal 记录槽、单元头读不出时这一半报「不适用」；② 各盘的实例代号（每盘取它全部自证过的槽里最大的那个）不等时，较大者不出现在任何根记录、journal 记录、单元写序里，镜像记不下这次挂载独占打开的集合时报「不适用」，第一版两盘就是全部盘。取号没做完、全或无失败回卷期间的代号不等由 ② 判成立，不另开例外（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的取号规则）。只在 devs ≥ R 时根环才是系统配置之外的独立见证，第一版 R = 3 > devs = 2 时两路见证的失败域是同一对盘 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 用户定案改成 ① ②**：checker 读每盘两槽里全部自证过的系统配置，扫描方向加读 journal 环与三类单元头的写序；层 0 全量 262165 个状态 I-7.7（系统配置实例代号不低于根环） 违例 0（旧定义「各盘择到的相等且 ≥ 根环」下取号只落一块盘的 2 个状态判红）；坏镜像三份：没人引用的实例 2 孤儿（①）、journal 环里实例 2 的记录（①）、盘 0 最大号 2 而孤儿带 2（②） |
| I-7.8 | 根记录树 ID 水位不低于全池最大树 ID | **根环里全部根记录的「树 ID 水位」字段取 max，必须 > 该池中出现过的最大树 ID。**「出现过的」由 checker 全盘扫描算出：全部单元头的索引节点类身份段里那个树 ID（D18（块里携带什么信息） 已定项 7），并上树表条目里的树 ID；**不许与运行时共用同一段代码**（D13（验证路线） 已定项 5）。取 max 的范围是「根环里**全部**根记录」不是「回退候选集」——被抛弃时间线的根仍然承载它那一代发过的号，形态与 D23（journal 的角色与格式） 已定项 14 给 checkpoint_txg 取值那句同源。**判别力**：抓两类——① 某次发布没更新水位；② 水位被实现成「本 checkpoint 发出的最高号」而不是**累计高水位**（后者的镜像是连发 R + 1 个 checkpoint 各建一棵树，第一棵的号会随根轮出环而被重发）。⚠️ **只许写成不等式**：水位是「下一个可用号」的上界，写成等式会把正常镜像判红 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 崩溃孤儿两条流（第一个文件版本那次发布的每个前缀 → 可写挂载 → 挂载之后六种动作）在 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`；中间实例行（T = 0）不排除的坏镜像是 `checker_known_bad_images.rs` 的 `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`（实例 2 的根槽全清零之后实例 3 写中间实例行 (2, 0, 0)，根环水位压低之后只红 I-7.8（根记录树 ID 水位不低于全池最大树 ID）；回退行那一道随回退行删掉，D18（块里携带什么信息） 已定项 11）；实现在 `walk::written_after_its_instance_was_last_applied` ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲；2026-09-24 主 agent 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 1 条补第二道，被攻过零轮）**：崩在发布之前的那个事务写出的孤儿节点不算「出现过」。扫描方向不数两类码 2 节点：① 诞生代号超过根环里最大 checkpoint_txg 的；② 写序实例 i 在最新根指着的实例表里有行 (i, Ti, ·)、Ti > 0、而诞生代号 > Ti 的——行说实例 i 只被施加到 Ti，之后写的没发布过；下一个实例第一次发布取的 txg 可以等于孤儿的诞生代号（D23（journal 的角色与格式） 已定项 14 注 3 取等号），只靠 ① 挡不住（第一个文件版本崩在记录落盘之前、再可写挂载的两条流上 130 格在合法状态上判红）。Ti = 0 的行（恢复写的中间实例行）不排除：中间实例发布过的号仍要算（D8（核心索引结构） 已定项 8 ②）。节点头 fsid 不是本池的照旧数。按字面数孤儿，层 0 全量 262165 个状态里 261891 个判红（水位 11 而盘上已有树 ID 11–15 的孤儿节点），见 records 2026-09-13-总审核 十一·七 ⚠️ **判法补一句（已实现：`walk::check_pool_image` 末尾在取 max 那一格之后再判最新那条根自己的水位）**：另判最新那条根的水位不低于全池最大树 ID。只取根环里全部根的 max 判不到把最新那条根的水位压低的镜像：攻方量到 checker 49 条 0 违例，而 core 的 `recover` 报 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 失败；core 重发树 11..18 的状态 checker 也 0 违例。用攻方那两份坏镜像钉住（只压低最新那条根的水位；建一次树之后经会话入口覆盖写 24 次把发树那条根轮出去、再把环里每条根的水位压到 11，只红它），被攻过零轮（`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 9、10 行） |
| I-7.9 | 回退下界 F 不高于抬 F 的上限 | **抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；新实例的第一条根没有同实例的前一条，不算抬——它带的是恢复算出的 F_生效；管理员回退是现行实例里的一次向前发布，带的 F 与前一条相同，不算抬；一次抬 F 推的几次空发布只有第一条算）根记录里那个回退下界 F（D22（单元原子性怎么合成） 已定项 7 的字段），按这条根 flags 里的**卸载记号**（D22（单元原子性怎么合成） 已定项 7 flags 位 0）分两支判：**带卸载记号的**，F 不高于根环里 txg 比它小、又落在锚在镜像最新根上的那一个一圈窗口里的最新那条根的 txg（只取这一个窗口，`crates/singlefs-checker/src/walk.rs` 5782 行一带；窗口里一条都没有时不判，同不带记号那一支的「之前的根」；盲带：F 越过洞里一条有效根时这一支看不见，欠账见 C717（回退下界上限的单窗口盲带）；正常卸载把 F 抬到现行那一版的 txg、不另判上限，D16（发布语义） 已定项 1「抬 F 的上限」）；**不带的**（准入抬 F），F 不高于**用它之前的根算出的** D16（发布语义） 已定项 1 的准入抬 F 上限：min(每块盘上最新的持久有效根的 txg, 第 4 新的非空持久有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」（两支都取）取锚在镜像最新有效根的 txg 上的那一个一圈窗口里 txg 比它小的那些（窗口 = txg > 锚 − 3S，3S 是环长；用户 2026-10-04 定「乙，两支都取两个窗口」，第 10 批起锚在被判那条根上的那个窗口被这一个罩住、不再另取，主 agent 认的读法、被攻过零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 10 批 core 簇交回那一条），窗口里没有比它小的根时那条根不判；只取锚在被判那条根上的窗口时，洞里留着的上一圈的旧根会顶上来、在合法镜像上判红（随机历史 230 段红 21、150 段红 9，并上锚在镜像最新有效根上的窗口之后各剩 1 段，红在带卸载记号那一支，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批 9-6 交回那一条）；取全环时攻方五批随机历史里每批 27–46 段红（判决 `research/prompts/m3-root-ring-code-r2-main-verification.md` 第五节）；上限照 D16（发布语义） 已定项 1「抬 F 的上限」那一行，窗口里挂载那一刻读不出、之后这个进程没写过的槽当作可能是一条非空有效根、F 不越过它，这些槽 checker 看不见，checker 只按镜像上读得出的根判（推的：读故障不留在盘上）。只判抬 F 的那一条，不在后来每张镜像上重算：F 抬过之后再回退，后来的根会把当初撑着 F 的非空根判成无效，现算的上限会低于 F，而 F 不回落（用户定案 2026-09-24，理由见实二报告：按「有效根不要求 txg ≥ F」的读法，固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与两个随机种子在合法状态上判红）；非空 = 它的树表里 inode 树、extent 树的根指针与它前一条有效根（按 txg 排）树表里的不同——比的是树表里这两棵树的条目，不比树表单元自己的落点（D16（发布语义） 已定项 1 表格后那段「「非空」从盘上怎么认」）；数「第 4 新的非空」时去重：从新到旧数，一条非空根只有在它的 (inode 树根指针, extent 树根指针) 与每一条比它新、已计入的根都不同时才计入（同一段）。⚠️ 它判的是一张镜像上 F 有没有抬过头，不判 F 该不该抬（那是准入的事，D28（挂载期承诺量） 已定项 1）。⚠️ **卸载记号盘上没有别的东西核它**：准入入口误打记号、F 又抬过头，按带记号那一支放过（第三轮判决第二节 K6：(b) 丢的正是「准入抬过头却带了卸载记号」那一格）——这一格归层 0 录制流另核「卸载记号只由卸载入口打」（C557（卸载记号只由卸载入口打没有检查）），不归这一条。⚠️ F 写进了系统配置而带新 F 的根没落盘或坏了、恢复后生效值怎么取的那一类，归层 0 与 C556（checker 与层 0 不读系统配置里的 F），不归这一条。算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判 | 已实现，两支（不带记号那一支 2026-09-24、带卸载记号那一支 2026-09-26；两支的「之前的根」2026-10-04 起取两个一圈窗口的交，第 10 批起只取锚在镜像最新有效根上的那一个）：池级 checker `walk::check_pool_image` 的 `judge_rollback_floor_raises_against_their_ceilings`，根环里每条抬 F 的根各判一格，按根记录读出的卸载记号分支（checker 读成 `UnmountMarkerView`）。**不带记号那一支**：坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（步 5 那段历史上抬到上限 11 与之后再发一版都成立、抬到 12 只红它）；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史各档不红；层 0 `crash_enumeration_fixed_script_stream.rs` 抬过 F 的三条流要求真被评估过；`crates/mutations.tsv` 7 条变异（判法拿掉、上限取大、第 4 新改第 3 新、有效按新 F 取、不限同一实例、一条都不认、「4 个不同状态」不去重）；用户 2026-09-24 定义，里程碑「覆盖写、释放、回退与复用」收口表第 26 行。**带卸载记号那一支**：坏镜像同一文件的 `marked_root_raising_the_floor_above_the_previous_root_reddens_only_the_rollback_floor_invariant`（正常卸载之后的镜像按带记号那一支判 6 ≤ 6 成立、按准入那一支会红；把那条带记号的根的 F 改成 7 只红它）；变异「checker 把每条根都读成不带卸载记号」「带记号那一支把不高于前一条根的 txg 判成低于」；记号的写读在 `crates/singlefs-core/src/root_record.rs`（写者照 `unmount_marker` 写 flags 位 0、读者拒记号以外的位），写者、读者各有一条变异（实二）。**一圈窗口**（用户 2026-10-04 定两个窗口，被攻过零轮；第 10 批起只留锚在镜像最新有效根上的那一个）：harness 用例在 `crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs`（带卸载记号的根在洞上再挂之后不许红，用例 `an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole`；两个窗口里仍照红，用例 `an_unmount_sequence_root_raising_the_floor_above_the_previous_root_inside_both_windows_still_reddens_the_rollback_floor_invariant`；准入那一支的几形同文件），`crates/mutations.tsv` 里以「第9批改钉」打头、点名带卸载记号那一支的 2 行，与以「第9批」打头、点名「一圈窗口」的 4 行；改钉之后随机历史 230 段、150 段两批 0 段红（改钉簇实现员量的，输出在 `research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/`）；GPU 判器的扫描内核两支同取锚在镜像最新有效根上的那一个窗口（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `in_the_one_revolution_window`），对拍用例与 8 行变异在 checker 档、没跑 |
| I-7.10 | 已停用·回退见证表删了 | **此编号不再使用**——2026-09-26 管理员回退改成挂着时的一次向前发布，系统配置里的回退见证表删掉（D23（journal 的角色与格式） 已定项 14、D22（单元原子性怎么合成） 已定项 9）⇒ 这一格**无对象**。曾要求：每块盘两槽里自证过的系统配置槽，槽内偏移 481 起的回退见证表解得开（条数不超过 R × S − 1、条目严格升序、目标实例代号小于新实例代号、条数之后全 0），同一个新实例代号在各盘各槽里记的回退目标相同 | 不适用 |
| I-7.11 | 已停用·见证择根删了 | **此编号不再使用**——2026-09-26 回退见证表删掉（D23（journal 的角色与格式） 已定项 14），择根不再跳过被见证抛弃的根 ⇒ 这一格**无对象**。曾要求：所选根是根环里自证过、不被回退见证表抛弃的根里 (txg, 实例代号) 最大的那一条；根环里至少一条自证过的根不被见证表抛弃；见证表里新实例代号不大于所选根实例代号的每一条，所选根指着的实例表罩得住它 | 不适用 |
| I-7.12 | 系统配置 F 不低于同盘根上的 F | 每块盘分开判：这块盘两槽里自证过（校验和过且 fsid 与本池相同，与取号同一读法，D18（块里携带什么信息） 已定项 11）的系统配置槽中回退下界 F（D22（单元原子性怎么合成） 已定项 9 的字段）的最大值，不低于这块盘根槽里每一条自证过的根带的 F（D22（单元原子性怎么合成） 已定项 7 的字段）。抬 F 那一串先把新 F 写进每块盘的系统配置、过一道屏障，才发第一条带新 F 的根（D16（发布语义） 已定项 1「抬 F 那一串」），所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里。某块盘两槽都自证不过时，那块盘不判；一块盘都判不了时整条报「不适用」。判别力：把「先写系统配置」挪到第一条根之后，崩在两者之间的状态必须红 | 已实现（2026-09-26，池级 checker `walk::check_pool_image` 的 `judge_system_configuration_floor_against_the_roots_on_each_device`：每块有自证过的系统配置槽的盘判一格，那块盘上一条根都没有时那一格成立；两槽都自证不过的盘不判，一块盘都判不了时整条报「不适用」。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `one_device_whose_system_configuration_floor_is_below_a_root_on_it_reddens_only_the_system_configuration_floor_invariant`：步 5 那段历史上抬到上限 11 的镜像一条都不红、I-7.12（系统配置 F 不低于同盘根上的 F） 真被评估过且成立，盘 1 两槽的 F 改回 0 只红它、红在盘 1 txg 16 那条带 11 的根上；变异「不比系统配置里的 F（每块盘恒成立）」「checker 不读系统配置里的 F（读成 0）」（实二）。条款仍是主 agent 2026-09-26 按 SysPre 的写序推的，没三方；层 0 每个崩溃状态都跑池级 checker，按判法每个状态都评估得到它（推的，层 0 没跑） |
| I-7.13 | 系统配置池级字段在读者收的范围里 | 任一自证过（magic 与整槽校验和对，`check_system_configuration_slot`）、incompat 位认得的系统配置槽（字段住 D22（单元原子性怎么合成） 已定项 9 的字段表，读者收的界在 D22（单元原子性怎么合成） 已定项 2 与已定项 16）：格式版本 = 1（`SYSTEM_CONFIGURATION_FORMAT_VERSION_THIS_CHECKER_READS`）、加密类型 = 0（第一版恒关，`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFF`）、固定结构槽距 ≥ 4096 字节（`FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES`）且槽 1 整槽落在根环基址之前、`physical_block_size` ∈ [457（`ROOT_RECORD_BYTES`）, 槽距]、journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1 且环末端不越过单元区起始槽号（`geometry_of` 的 `fixed_structure_slot_spacing_lies_in_the_format_range` / `physical_block_size_fits_a_root_slot` / `journal_ring_bytes_lie_in_the_supported_range`）。任一盘任一槽不满足其中一项即判红：checker 报违例、不作保，且该池其余不变量一律报「不适用」，与实现整池拒绝挂载一致（用户 2026-09-27 定系统配置越界整池拒）。**判的字段与 core 收同一张表**：格式版本、加密类型、槽距（上界用同一槽自述的根环起点）、根槽宽、环长、单元区起始槽号（偏移 417）= 环长现算的起点且在 64 槽段边界上、journal 环起点（偏移 325）与根环起点（偏移 371）等于第一版常量，共八项（D22（单元原子性怎么合成） 已定项 16 第 1 条）。R（区域数）与 S（每区槽数）越界不在这一条里，仍归「这一槽不可择」 | 已实现（2026-09-27，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_system_configuration_values_the_reader_accepts` 在 `walk::check_pool_image` 里调用，任一盘任一槽带越界值时该池其余不变量整批报「不适用」并提前返回。**八项 checker 与 core 两边都判**：checker 解槽时判格式版本与加密类型（`crates/singlefs-checker/src/lib.rs`），其余六项在 `geometry_of` 判——先判 journal 环起点（偏移 325）= `JOURNAL_RING_START_SLOT`（1024）、根环起点（偏移 371）= `ROOT_RING_BASE_SLOT`（64）（两个起点先于槽距判：槽距上界按同一槽自述的根环起点算），再判槽距、根槽宽、环长，最后判单元区起始槽号（偏移 417）= 同一槽自述的环起点 + ⌈环长 ÷ 16384⌉（`unit_area_start_slot_is_the_slot_after_the_journal_ring`）且是 `CLUSTER_SEGMENT_SLOTS`（64）的整数倍；新加的四判各报一个 `Verdict` 成员（`JournalRingStartSlotNotTheFirstVersionConstant`、`RootRingBaseSlotNotTheFirstVersionConstant`、`UnitAreaStartNotTheSlotAfterTheJournalRing`、`UnitAreaStartOffTheClusterSegmentBoundary`），都归 `value_refused_by_this_reader`（整池不作保）；常量取自 `singlefs-format`，算式 checker 自己写。core 一侧 `crates/singlefs-core/src/system_configuration.rs` 的 `parse_slot` 读出偏移 325、371 两个字段，`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts` 按格式版本、加密类型、槽距（上界用同一槽自述的根环起点，`largest_fixed_structure_slot_spacing_under_the_root_ring_base`）、根槽宽、根环起点、journal 环起点、环长、单元区起点的次序判，不收的值报 `SystemConfigurationValueOutsideWhatThisReaderAccepts` 的一个成员（两个起点是 `RootRingBaseNotTheFirstVersionSlot`、`JournalRingStartNotTheFirstVersionSlot`），经 `RecoveryFailure::SystemConfigurationValueRefused` 整池拒。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：`a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`（盘 1 槽 0 加密类型改成 1、重算整槽校验和，只红 I-7.13（系统配置池级字段在读者收的范围里）、其余不变量全报不适用），`each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field`（几何字段各一格，各只红 I-7.13（系统配置池级字段在读者收的范围里）、违例说明点名被拒的那个成员）。两边收同一张表由 `crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 钉住：新池新建文件之后的镜像上两块盘四个系统配置槽改同一个几何字段、整槽校验和重封，五格各一条用例，每格 core 的只读与可写挂载都拒成同一个 `SystemConfigurationValueRefused` 值、两块盘逐字节不变，checker 报 I-7.13（系统配置池级字段在读者收的范围里） 违例、其余不变量报不适用；根环起点写 0 那一格两边点名的成员不同（core 先判槽距上界、报槽距，checker 先判根环起点、报根环起点），照两边各自的判定次序钉（用户 2026-09-27 定「读字段，不等于常量就整池拒」，`records/2026-09-24-里程碑二收尾调度.md` 第 293 行；代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节「core 与 checker 对系统配置几何字段的读法」那一格）|
| I-7.14 | 系统配置本盘设备号等于盘在池里的身份 | 每块盘两个系统配置槽里，每个自证过（magic 与整槽校验和对）、fsid 与本池相同的槽，自举头里的本盘设备号等于这块盘在池里的身份。字段住 D22（单元原子性怎么合成） 已定项 9 的系统配置字段表（自举头「本盘设备号 4」）；实现一侧是 D18（块里携带什么信息） 已定项 11 里挂载核盘表那一句：可写挂载比盘表（`crates/singlefs-core/src/mount.rs` 的 `device_table_disagreeing_with`，`OwnDeviceNumberDiffersFromTheIdentityHandedIn`）。别的池的槽归 I-1.4（块头 fsid 一致） 判，不在这一条里 | 已实现（2026-09-28，池级 checker `crates/singlefs-checker/src/image.rs` 的常量 `SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY` 登记这个编号、列进 `IMPLEMENTED_INVARIANTS`；判定 `judge_own_device_number_is_the_device_identity` 在 `walk::check_pool_image` 里调用，每块盘每个自证过、fsid 属于本池的系统配置槽判一次，本盘设备号与 `ImageReader::devices` 给的这块盘的身份比。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：两块盘四个系统配置槽的本盘设备号互换，`devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant` 钉只红 I-7.14（系统配置本盘设备号等于盘在池里的身份），这份坏镜像也在 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 的清单里；变异表 `crates/mutations.tsv` 里对着这一判的三行（判定恒真、池级走读不调这一判、坏镜像写回原值）改法 A 的 `prove-red.sh` 各抓到（`research/prompts/m2-impl-r2-fixes-a-implementer-report.md` 第 77–78 行）。合法历史上不判违例：门禁 harness-model-differential-and-scenarios 随机历史那一趟 I-7.14（系统配置本盘设备号等于盘在池里的身份） 判绿 4155 次、不适用 0 次（同一报告第 189 行）；层 0 快档与全量没跑）|
| I-7.15 | 各盘所选系统配置槽设备数相同 | 各盘所选的系统配置槽（自证过、世代号最大者）里的设备数相同（D22（单元原子性怎么合成） 已定项 9 的字段）。不要求等于盘表：降级镜像（掉了一块盘）里 checker 手里的盘数小于设备数是合法的。形态抄 I-1.4（块头 fsid 一致） 的逐盘循环；core 的 `choose_system_configuration` 在几块盘的设备数对不上时报 `SystemConfigurationsDisagree`、整池挂不上，两边同表。 | 已实现（2026-10-02，池级 checker `walk::check_pool_image` 逐盘把各盘所选槽的设备数同第一块盘所选槽的比；坏镜像是盘 1 两个槽的设备数改成 3 并重封，只红它，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C466（几块盘的系统配置设备数对不上时 checker 判不出）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 7 行） |
| I-7.16 | 系统配置槽世代号与槽号的关系 | 每块盘上，任一可择的（自证过、池级字段在读者收的范围里）系统配置槽 i：世代号 ≥ 2 时世代号 mod 2 == i；两个槽都可择时世代号相差不超过 1，相等时只许都是 1（mkfs 写下的两槽世代号都是 1，D22（单元原子性怎么合成） 已定项 16）。带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里） 整池拒，不进这一条。 | 已实现（2026-10-02，池级 checker `judge_system_configuration_slot_generations_follow_the_rotation`（`crates/singlefs-checker/src/image.rs`）：奇偶一格、差值一格，差值那一格取「两槽都是 mkfs 写的 1，或相差恰为 1」；差值与奇偶各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 7 行与第四节用户定案第 1 条） |
| I-7.17 | 根槽位置等于 txg 算出的位置 | 根环里任一自证过的根所在的槽位 (区域 r, 槽 s)：`checkpoint_txg` ≥ 1 时 r == txg mod R 且 s == (txg div R) mod S；`checkpoint_txg` == 0（mkfs 把第 0 代种进全部区域，D22（单元原子性怎么合成） 已定项 8）时只判 s == 0，不判 r。公式是 D22（单元原子性怎么合成） 已定项 16 的定案。根环补洞判决要的「槽位与 txg 对得上」就是这一条（`research/prompts/m3-ring-r1-main-verification.md` 第二节第 6 行）。 | 已实现（2026-10-02，池级 checker `judge_root_records_live_in_the_slot_their_txg_names`（`crates/singlefs-checker/src/image.rs`）：根环里每条自证过的根逐条判，第 0 代只判槽 0；公式同 core 的 `target_for_publish`（`crates/singlefs-core/src/root_ring.rs`）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 8 行） |

> ⚠️ **I-7.3（环健康性） 的价值在于它不需要真的崩溃一次才能发现 bug。**
> 轮换逻辑写错（例如连续两代落到同一槽位）在**下一次正常提交后的 checker 扫描**里就会被抓到，
> 而不是等到某次撕裂真的发生、发现无处可退时才暴露。
> 这与「校验和会不会红」是两类判别力：前者查机制活着没有，后者查数据对不对。

> ⚠️ **I-7（根环） 类验不了「失败域独立」这个假设。** 根环的多区域布局依赖
> 「盘首与盘尾的物理故障 / 掉电撕裂事件统计独立」，而对着镜像判不出这一条——
> 它只能验证「机制按设计跑了」，不能验证「设计的假设是真的」。
> 该假设**本机无证据**，见 [decisions.md](decisions.md) D22（单元原子性怎么合成） 已定项 2。

```

**出处 `.claude/kb/invariants.md:83-124`（整段抄，未转述）**

```markdown
## I-8 journal —— 类

**依赖 D23（journal 的角色与格式），形态已定（定长环 + tail 住系统配置槽），checker 未实现。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-8.1 | 环几何够大 | 每块盘每个自证过的系统配置槽里三个字段各占一格：journal 在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋，journal 最坏占用 = 在飞上限 × 4096，安全系数 F = `JOURNAL_SAFETY_FACTOR`（格式常量，checker 与 core 都判等于，不是下界），任一不等判红；另判一格：环里同一实例、同一 `checkpoint_txg` 的自证记录（同一次发布）不多于在飞记录数上限条。`环大小 ≥ F × 任一事务的最坏 journal 占用` 是这几格的推论，不另判。mkfs 时算、mount 时复算比对，core 挂载同判。三格任一不等时与 I-7.13（系统配置池级字段在读者收的范围里） 同口径整池不作保：checker 报这一条违例，其余不变量一律报「不适用」，与实现整池拒绝挂载一致；I-7.13（系统配置池级字段在读者收的范围里） 与这一条同时不成立时走 I-7.13（系统配置池级字段在读者收的范围里） 那一支（只红 I-7.13（系统配置池级字段在读者收的范围里），这一条报不适用）；同一次发布记录数那一格不触发整池不作保。 | 已实现（2026-10-02，池级 checker：三个字段等式在 `judge_journal_ring_geometry_fields_agree`（`crates/singlefs-checker/src/image.rs`），每块盘每个可择的系统配置槽各判三格：F 等于 `JOURNAL_SAFETY_FACTOR`，在飞上限等于环长 ÷ 4096 ÷ 该槽里的 F 取整（F 为 0 判红、不除），最坏占用等于在飞上限 × 4096（乘法溢出判红）；带越界值的槽归 I-7.13（系统配置池级字段在读者收的范围里）、不可择的槽不读几何；同一次发布的记录数不多于在飞上限那一格在 `walk::judge_records_of_one_publish_within_the_in_flight_record_limit`，取所选槽记的上限。F 那一格 2026-10-03 从「F ≥ 2」改成等于格式常量，三格不等的整池不作保那一支加在 `walk::check_pool_image` 里 I-7.13（系统配置池级字段在读者收的范围里） 那一支之后、I-8.5（tail 槽有效集非空） 之前（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 3、4 行）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（`each_ring_geometry_and_root_slot_alignment_bad_image_reddens_only_its_own_invariant`：F 取 1、2、4 与只改没被择的那一槽 F 各一份；`journal_ring_geometry_fields_that_disagree_redden_only_the_ring_geometry_invariant_and_every_other_is_not_applicable`：三格不等时其余不变量报不适用）。core 挂载同判：`crates/singlefs-core/src/recovery.rs` 的 `journal_ring_geometry_fields_recompute_from_the_ring_length`，放在 `system_configuration_values_this_reader_accepts` 的最后。在飞上限那一格复算取槽里的 F、core 取格式常量，F 不等于格式常量时两边都已因 F 那一格拒，不影响判红与否；对应的账是 C28（环几何不变量未实现）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 4 行） ⚠️ **口径注（D28（挂载期承诺量） 已定项 4）**：ckpt_cost 按当时各记录树的高现算、挂载期内会长；实现上最坏占用 = 在飞上限 × 4096，由环长与 F 倒算，取的是上界、不随树高变（`research/prompts/m3-inv-a-r1-main-verification.md` 回看决策那一行）。 |
| I-8.2 | 记录头不跨原子单元 | 任一 journal 记录的头完整落在一个 `physical_block_size` 单元内。该宽度**运行时探测**，不许硬编码 | 未实现 |
| I-8.3 | 重放前缀严格连续 | 被重放的记录集合，其 `jsn` 严格连续（`jsn == expected`，断号即止；链从所选根覆盖的最后一条记录之后接，跨实例边界即停，记录水位按实例代号为主比——D23（journal 的角色与格式） 已定项 14 的注，2026-09-13），且整段落在在飞记录数上限之内。⚠️ **这一条不是前缀纪律的全部**：D23（journal 的角色与格式） 已定项 14 的前缀口径共五条，这里只落了其中三条半——**「提交标记齐全的事务才施加」与「施加前逐项验证点名单元」两条不在这条判据里**，而两者的处境不同：前者 2026-09-23 已落成 I-8.8（前缀里的事务不被切开），checker 判定与崩溃点重放那一半仍欠，记在 C37（事务与记录的对应无定义）；后者**已经有了**，只是不住这一条——它住 D16（发布语义） 已定项 7，两条故障注入自证 2026-09-14 在层 0 做出（C76（发布时序没有权威落点））。⇒ 照这一条实现恢复施加逻辑的人会漏掉那两道防线，而它们正是挡住「半个事务」与「点名单元已损坏的记录」的 | 未实现 ⚠️ **判据补一句（2026-09-13，D16（发布语义） 已定项 4）**：前缀不得停在一次发布的中间——一次发布整体施加或整体不施加。 ⚠️ **再补一句（2026-09-14，D23（journal 的角色与格式） 已定项 18 用户定案）**：「在飞记录数上限」的语义就是**这个前缀最多 N 条**（N = 环槽数 ÷ F），不是 XFS 那种「最后 N 条内算撕裂、之外算损坏」；**任一校验和不过的记录一律断链即止**——停在它之前那一条，第一版不区分撕裂与损坏、不因此拒绝挂载。 |
| I-8.4 | 重放幂等 | 任一 journal 记录被重放两次与重放一次，产生的读出内容相同（等价关系取「读出的内容相同」，不比扇区、实例代号与行） | 未实现（实现在途：取 R-换，在崩溃注入的二次崩溃上判恢复加写行那一整段；被攻过零轮，实现时先在崩溃注入上量误红；`research/prompts/m3-inv-a-r1-main-verification.md` 第四节用户定案第 2 条） |
| I-8.5 | tail 槽有效集非空 | 池里至少有一块盘的两个系统配置槽里有一个槽自证过（magic 与整槽校验和对）：tail 住这些槽（D23（journal 的角色与格式） 已定项 3），这个集合就是全部 tail 见证者。全池皆废判红（mkfs 没做完、写到系统配置之前失败的盘面也是全池皆废，照判红，故障注入的已知红清单认这一类，用户 2026-10-03 定）；单盘全废不判违例，走旁路报告：另报哪块盘少于两槽自证过、可写挂载会拒。对着只读恢复判。自证过而不可择的槽（R / S 越界、incompat 位不认得）与带越界值的槽都算自证过；I-7.13（系统配置池级字段在读者收的范围里） 带越界值整池拒、或 I-8.1（环几何够大） 三格不等整池不作保的那一支里，这一条不判、报「不适用」。有系统配置槽自证过、而全池择不出一块可择的系统配置时同样报不适用（理由：这不是一个挂得上的镜像；代码 `crates/singlefs-checker/src/walk.rs` 10022 行一带），一个槽都没自证过照旧判红。 | 已实现（2026-10-02，池级 checker `judge_some_device_carries_a_self_verified_system_configuration`（`crates/singlefs-checker/src/image.rs`）：把「不自证」（magic 或整槽校验和不对）与「自证过而不可择」分成两类，后一类计入自证过（`slot_verifies_itself`），别的不变量在全池皆废时照旧报不适用；单盘全废走旁路报告 `system_configuration_redundancy_report`，不进违例通道；自证过而不可择的四槽一条都不红的一份坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（`self_verified_system_configuration_slots_no_reader_can_choose_still_witness_the_tail`）；对应的账是 C461（checker 判「挂不上的镜像」整片报不适用）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 6 行与第四节用户定案第 3 条） |
| I-8.6 | 反向链算法 | 任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 487 字节头，`header_csum` 那 32 字节按零参与)；**本实例写出的第一条记录反向链恒 0**（D23（journal 的角色与格式） 已定项 10 / 已定项 19，2026-09-13 用户定案，链按实例算是 2026-09-14 用户定案，头宽随 D23（journal 的角色与格式） 已定项 4 现行 487）。不等的记录不进重放前缀。jsn 的计数器为 0 的记录判违例：它没有本实例内的逻辑前一条，反向链无从定义（D23（journal 的角色与格式） 已定项 18 的落点式 `(计数器 − 1) mod 槽数 × 4096` 从计数器 1 起；用户 2026-09-27 定）。⚠️ 链按实例算，所以跨实例边界的两条记录之间**不判**这一条——实例边界由实例代号挡着（I-8.3（重放前缀严格连续)） | 已实现（2026-09-21，池级 checker `walk::check_pool_image`，逐盘判：计数器 1 与跨实例边界的第一条判反向链恒 0，同实例前一条在盘上时判链值，前一条不在盘上不判；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 里环还空着的 4 个状态报不适用，其余每个崩溃状态都判）；计数器 0 判违例（2026-09-27，`walk.rs` 的 `ExpectedBackChain::CounterZeroHasNoLogicalPredecessor`，用例 `a_journal_record_whose_counter_is_zero_reddens_the_back_chain_invariant` 在 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`） |
| I-8.7 | 实例内事务号不重号 | 判环里全部自证过的记录，同一实例按 `jsn` 的计数器看：① 按计数器相邻的两条非 0 记录，事务号不减（事务号 0 是不承载事务的空发布记录，不进这个比较）；② 一个事务号的记录在计数器上连续，被别的记录（含事务号 0）隔开之后不再出现。D23（journal 的角色与格式） 已定项 7 逐字：「事务号按实例计数、从 1 起」「记录按事务号顺序追加……这是实例表行的 W 能当精确前缀的依据」；同一项定一个事务可以跨多条记录，所以相邻两条可以同号。 | 已实现（2026-09-21 立时判「非 0 事务号严格递增」；2026-09-23 按 D23（journal 的角色与格式） 已定项 7 改成不减加同号连续。池级 checker `walk::check_pool_image` 的 `judge_transaction_numbers_per_instance`：逐盘按实例判，同一实例按计数器相邻的两条非 0 记录，号不等时后一条严格大于前一条（①），号相等时两条之间不许坐着同一实例的别的记录（②，事务号 0 的也算；中间只有读不出的槽或别的实例的记录不判红）；「被别的非 0 号隔开之后又出现」一定伴着一处 ① 红，不另判；同一实例非 0 的记录不足两条时报不适用。坏镜像三份（交错两份、事务号倒退）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，每份只红这一条，违例说明标出红的是 ① 还是 ②） |
| I-8.8 | 前缀里的事务不被切开 | 判环里全部自证过的记录：③ 同一实例同一事务号的一组记录里，**带提交标记的至多一条，且它是这一组计数器最大的那一条**；提交标记那一字节只许 0 或 1；④ **一组里一条提交标记都没有时，这个实例在它后面不许再有任何记录**（事务号 0 的空发布也算；别的实例的记录不算）：D23（journal 的角色与格式） 已定项 7 逐字写「一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」，实例切换之后这个实例不再写任何记录。它与 I-8.7（实例内事务号不重号） 的 ① ② 合起来，保证 D23（journal 的角色与格式） 已定项 7 原话那个恢复算法「丢掉提交标记还没出现的那个事务的全部记录」**恰好丢掉、且只丢掉**被崩溃切开的那个尾巴；`crates/singlefs-core/src/recovery.rs` 今天那一步写成「按 jsn 扫到第一条不带提交标记的记录就停」，只对一事务一条成立，一事务跨多条记录时要改，归并行线一。四条里任一不成立，恢复要么丢掉一个已经提交的事务、要么施加半个事务，而盘上没有任何痕迹。⚠️ **今天可达镜像上它恒真**：第一版一条记录一个事务、`is_commit` 全仓没有一处写 false，所以 ③ ④ 今天都不会被可达历史违反。**判别力只能靠坏镜像**，坏镜像不做这条就是恒真的检查——恒真的检查比没有检查更糟。⚠️ **它管的是条款允许的形态，不是第一版的形态**：D23（journal 的角色与格式） 已定项 7 定的是「可以跨多条，记录头带事务边界字段」，而第一版按 C310（事务切分纪律与记录数口径打架） 一条记录一个事务。I-8.8（前缀里的事务不被切开） 挡的是**一事务跨多条记录落地那一天**。⚠️ **它只判一份镜像上的记录流自洽，不判「施加的结果等于整数个事务」**：后者要拿「这次事务该有的全部效果」当 oracle，一份镜像上判不出来（形态同 I-4.3（提交原子） 那一行），归层 0 崩溃点重放。 | 已实现（2026-09-23，池级 checker `walk::check_pool_image` 的 `judge_commit_markers_per_transaction`：逐盘，一组 = 同一块盘上实例代号与事务号都相同的非 0 记录；③ 两条以上的一组里带提交标记的至多一条、且是计数器最大那条；提交标记字节每条记录都判（事务号 0 的也算），不是 0 或 1 即红；④ 一条提交标记都没有的组，后面同实例还有任何一条记录（事务号 0 的空发布也算）即红。没有 ③ 的对象也没有 ④ 的对象 ⇒ 报不适用，提交标记字节单独成立不算成立；第一版一事务一条、每条都带提交标记，层 0 每个崩溃状态都判、全部报不适用。阳性对照（一事务两条、提交标记在 jsn 最大那条，I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 都成立）与四份坏镜像（提交标记放错位置、没提交的事务后面还有号更大的记录、没提交的事务后面跟一条空发布、提交标记字节写成 2，每份只红这一条）在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，违例说明逐条标出红的是哪条判据） |
| I-8.9 | 一次发布的记录序号连续且只有末条带标志 | 同一实例、同一 `checkpoint_txg` 的记录（一次发布写出的全部记录）：「本次发布内序号」依次是 1..N、与 jsn 同步（jsn 连号时序号也连号），记录标志位 0 恰好只在序号 N 那一条上为 1，记录标志其余位为 0；序号 0、一次发布之内跳号、没有末条或多于一条末条，都是违例。只判读得出的记录，读不出的那条不判（D23（journal 的角色与格式） 已定项 4、17，用户 2026-09-24 定，`records/2026-09-24-里程碑二收尾调度.md` 第三节「序号与末条标志的不变量」） | 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_publish_ordinals_and_last_record_flags`；阳性对照与七份坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |
| I-8.10 | 记录新根段与同代根相同 | 某块盘 journal 环里自证过的记录 J，带末条标志（记录标志位 0 = 1），且这块盘上同（实例代号, `checkpoint_txg`）带末条标志的记录只有这一条；根环里任一自证过、实例代号与 `checkpoint_txg` 都与 J 相同的根 R，J 的新根段四段与 R 的同名四段逐字节相同：新根段 [0..86) = R[36..122)、[86..172) = R[256..342)、[172..180) = R[122..130)、[180..188) = R[130..138)（树表单元指针、中央映射树根指针、树 ID 水位、回退下界 F，D23（journal 的角色与格式） 已定项 15）。只认带末条标志那一条；同（实例, txg）有两条以上不同内容的根不判，归 C88（根环的时间线判别未实现）；末条标志不唯一归 I-8.9（一次发布的记录序号连续且只有末条带标志）。 | 已实现（2026-10-02，池级 checker `walk::judge_new_root_segment_of_the_last_record_against_the_roots`：逐盘按 (实例, txg) 分组，带末条标志恰一条才判，与根环里同 (实例, txg) 的每条自证过的根比四段，一对都没比到报不适用；坏镜像取记录一侧，把 F 改一位并重封头校验和，另有树表指针、映射根指针、树 ID 水位各一份，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 4 行） |
| I-8.11 | 点名项与被指单元头相同 | 任一自证过的 journal 记录里任一点名项的每条位置条目：若它指着的那一份单元的整单元校验和与位置条目里的相符，则单元头里的类标签、诞生代号、key 尾段与点名项偏移 28、37、45 的同名字段逐字节相同（码 1 尾段 = 写序实例代号 4 + 事务号 6，码 2 / 码 3 尾段 = 实例代号 4 + 出生序号 4 + 补零 2，补零恒 0）；出生树（点名项偏移 29）只判码 1、码 3，等于头偏移 43，码 2 随 C289（码 2 的出生树取哪个字段没写） 不判。校验和不相符的位置条目不判（落点被合法复用之后，单元头就不是它点名的那个单元了）。 | 已实现（2026-10-02，池级 checker `walk::judge_named_entries_against_the_unit_headers`：点名项每条位置条目，所指那一份整单元校验和相符才比，不符不判；出生树只判码 1、码 3；一条都没比到报不适用；类标签、出生树、写序实例代号、key 尾段各一份只红它的坏镜像，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 2 行） |
| I-8.12 | 同实例记录的 txg 不减 | 同一块盘 journal 环里自证过的记录，同一实例代号的记录按 jsn 计数器从小到大，`checkpoint_txg` 不减。换实例后严格大于不判（设备失而复得下没核）。 | 已实现（2026-10-02，池级 checker `walk::judge_journal_record_txg_per_instance`：逐盘、逐实例按计数器看 txg 不减，换实例不判，没有可比的一对时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行） |

> ⚠️ **D23（journal 的角色与格式）的死锁 3 全靠 I-8.1（环几何够大），而 I-8.1（环几何够大） 的 checker 未实现。**
> 「写进本清单」不等于「有东西在拦」——取定长环那条论证依赖的是**拦得住**，不是**写下来**。
> 欠检查见 [checks-owed.md](checks-owed.md) C28（环几何不变量未实现）。

> ⚠️ **I-8.4（重放幂等） 是被实验逼出来的，不是设计时想到的。**
> [experiments.md](experiments.md) E24（恢复算法：先信 tail 会不会丢记录）测出：
> 全环扫描式恢复虽然不丢数据，却会把**陈旧 tail 以来的记录全部重放一遍**
> （窗口为空时仍重放满环 64 条）。没有幂等，「陈旧 tail 只是多做功」
> 就变成「重复施加已生效的记录」。

> ⚠️ **I-8.3（重放前缀严格连续） 与 I-8.1（环几何够大） 不许互相顶替。**
> E24（恢复算法：先信 tail 会不会丢记录）测到窗口撑爆环时（100 条 > 64 槽）
> **两种恢复算法一起丢**——恢复算法救不了几何。

> ⚠️ **I-8.4（重放幂等） 的射程有一条已实测的边界（E78（重放的起点），2026-09-02）**：
> 幂等管「同一条记录重复施加」，**管不到「旧记录的点名块已被合法复用」**——
> 陈旧 tail 的重放窗口远长于 I-7.4（近 K 代块未被复用）扣块的窗口，
> 窗口内记录的点名块换过内容后，施加时验证（E77（发布的持久顺序）证明它是承重步骤）会失配。
> 「陈旧 tail 只是多做功」因此只在不验证或有重放下界时成立；
> 重放下界的两条出路（根带 jsn 水位 / 盲放加尾删）见
> [checks-owed.md](checks-owed.md) C77（重放起点未定义）。

```

**出处 `.claude/kb/invariants.md:125-135`（整段抄，未转述）**

```markdown
## I-2 校验和 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-2.1 | 校验和与内容匹配 | 任一被引用的块，其校验和与内容匹配。按多长去比（父条目的类型段说的那一类与孩子头里的类标签不一致时）：走读往下走孩子的那一支，按父条目类型段说的那一类的长度读回比；不往下走孩子的几支（墓碑、谱系、inode 三棵树的类型段不认得、孩子自证成另一类，墓碑、谱系两棵树层级 ≤ 1 节点里类型段 0 的），每条位置条目按那一份头里类标签说的那一类的长度比，类标签不在登记表里或头读不出的，16 KiB 与 32 KiB 两种长度都对不上才算不匹配。同一块、同一对校验和，两支可以判得不同（用户 2026-10-06 定保持，`research/prompts/m3-batch29-code-r1-main-verification.md` 第四节） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算：F 之下的根引用的单元可以已被回收复用，不在它们上判——不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22（刚释放的块立即重分配） 的必红） |
| I-2.2 | 校验和算法全盘一致 | 校验和算法标识在整个文件系统内一致。**字段 2026-09-14 用户定案有了**（D22（单元原子性怎么合成） 已定项 9）：系统配置自举头段「校验和算法标识 1」，登记表 0 = 无效、1 = CRC-32C，第一版写 1；读到 0 或未登记的码一律拒绝挂载。算法的四个参数写死在 D18（块里携带什么信息） 已定项 17 | 未实现（条款已写、检查仍欠） |
| I-2.3 | 补齐字节为零且参与校验和 | 任一数据单元（码 1）或打包记录单元（码 3，D18（块里携带什么信息） 已定项 11），其声明长度（净荷字节数；补齐起点 = 头 + 声明长度）之后到单元末尾的补齐字节**恒为 0**，且校验和 / MAC 覆盖**整个单元含补齐**（[decisions.md](decisions.md) D4（校验和位置） 已定项 2：不足一个单元的 extent 补齐到 32 KiB；码 3 由头校验和（到头末尾）与载荷校验和（头之后到单元末尾）两段合起来覆盖整个单元）。补齐字节非零、或校验和只覆盖声明长度，一律判红。索引节点（码 2）从 2026-09-13 起同样在射程内（D18（块里携带什么信息） 已定项 18 用户定案）：声明长度 = 条目区字节数，之后到节点末尾的补齐字节恒为 0，载荷 CRC 覆盖 [86 + 2 × key 宽, 16384)——起点是明文头末尾，含 29 字节预留位、条目区与其后的补齐区，口径与码 1 的 105、码 3 的 107 同（预留位 2026-09-14 从 28 加到 29，含预留位的头是 115 + 2 × key 宽）。⚠️ **加密开启时判的是解密后的明文，不是盘上字节**（2026-09-07 用户定案，还清 C189（I-2.3 在加密开启时判明文还是盘上字节没定））：补齐在盘上是 `E(0)` 而不是 0，按盘上字节判必然全红；checker 本来就要解整单元，取值同一步就有。**与 D18（块里携带什么信息） 已定项 7 的载荷 CRC 方向相反是对的**——载荷 CRC 要给无密钥一侧用，所以算盘上字节；这一条只有 checker 用，所以判明文 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节、指针头部的 MAC 16 + nonce 12 与算法类型 1、压缩算法码 1、压后长度 2（D19（块指针的结构与宽度预算） 已定项 3；第一版不加密、压缩算法码登记表只有码 0、码 0 时压后长度恒 0，D9（加密） 已定项 10 / 已定项 14）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。根槽偏移 428..457 的算法类型 1 + nonce 12 + MAC 16（D22（单元原子性怎么合成） 已定项 7 的字段表）是同一条规则的第三处：加密关着时恒 0，池级 checker 对根环里每一条读得出、自证过的根判这 29 字节全 0（不按回退候选集滤），core 的根记录读者不读它们、挂载透明也不比它们（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1「算法、nonce、MAC」那一行：单镜像性质，归池级 checker）。指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（D19（块指针的结构与宽度预算） 已定项 3；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。根记录里四条指针（树表、实例表、中央映射树、分配记录树）头部有一段不是第一版的取值（MAC / nonce 非 0、算法类型非 0、压缩算法码非 0、压后长度非 0、extent 偏移非 0，五样任一）按这条根自证不过处置：core 的根记录读者（`RootRecord::parse_slot`，与 journal 新根段那四条指针同一判法）把这条根算不可择、退回上一条根（靠 journal 重放可以回到同一版，挂载照样成功），checker 红这一条而 core 挂载成功，两边各判各的、不矛盾（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 6 行）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:1174` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:1220，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:1258）；走读跟随的每条指针头部按三个判法各判一遍（`crates/singlefs-checker/src/image.rs`：`judge_pointer_mac_and_nonce_are_zero`（:1381）判 MAC 16 + nonce 12 全 0，`judge_pointer_encryption_and_compression_fields_take_the_first_version_values`（:1396）判算法类型、压缩算法码、压后长度三段各一格全 0，`judge_pointer_extent_offset_is_zero`（:1456）判 extent 偏移 0），五个入口是码 2 节点指针（`Walk::read_index_node`，walk.rs:1301 起：树表单元、树表条目的树根、中央映射树、inode 树、extent 树、分配记录树的节点都经它，根记录里树表、中央映射树、分配记录树那三条根指针也经它）、墓碑树与谱系树内部条目的子指针（`walk_the_entries_of_a_container_tree_node`，walk.rs:2150 起）、inode 内部条目子指针（`walk_inode_internal_node`，walk.rs:2443 起）、实例表链指针（`walk_instance_table_chain`，walk.rs:2836 起，根记录里的实例表指针是链的第 0 片；链指针记录有下一片而位置指针头部有一段不是第一版的取值时，这一片按 I-3.14（实例表每片记录形态合法） 第五形解不开、链不往下走，那条指针不跟随、这一处 I-2.4 不判）、extent 数据指针（`walk_extent_data_pointer`，walk.rs:3009 起），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；根槽偏移 428..457 那一处也已实现（`image::judge_root_record_algorithm_nonce_and_mac_are_zero`，image.rs:1431，`walk::check_pool_image` 经 `check_pool_image_given_the_tombstone_container_capacity_knob` 在 `valid_roots` 的结果上调它，walk.rs:10413）；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`）。**extent 偏移那一半与算法类型、压缩算法码、压后长度那几段已实现**（checker 走读跟随的每条指针都判这三格；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_pointer_head` 与 `w_judge_pointer_extent_offset` 在同样那几个入口判，根槽那一处是同文件的 `judge_root_record_algorithm_nonce_and_mac_are_zero`；core 每个读盘上指针头部的读者都判这五段（MAC / nonce、算法类型、压缩算法码、压后长度、extent 偏移）、判该结构损坏：树表条目的树根指针、inode 内部条目的子指针、extent 叶记录的数据指针、extent 上段叶条目的下段根指针与内联数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针各一处，加 journal 新根段、码 2 内部条目、根记录三处（`crates/singlefs-core/src/pointer.rs` 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from`，`PointerHead`、`NodePointer`、`DataPointer` 各一个；`grep -rn 'Pointer::read_from' crates/singlefs-core/src/` 只剩 `pointer.rs` 自己的单测一处）；影子账认单元的两个读者（`allocation_record_tree::node_pointers_as_far_as_readable`、`recovery::instance_table_page_pointers_as_far_as_readable`）经 `NodePointer::read_with_head_fields_judgement_from` 的 `NotJudgedByTheShadowLedger` 不判（影子账要多隔离、不漏隔离）；走读不跟随的指针 I-2.4 不判：根环里走读没走到的旧根，它们的树表单元只被 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 读条目取诞生 txg、不跟随条目里的树根指针，这些指针的头部没人判（测过的镜像上 12 条：三个旧树表单元各四条树根指针，`research/prompts/m3-formal-code-b30-r1-sonnet-output.md`；core 把那样一条根当恢复目标时经 `TreeTableEntry::parse` 判，推的、没造盘面验证）；坏镜像遍历偏移 32、33，core 每类读者各一条用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`，加密、压缩那几段 core 每类读者各一条在 `crates/singlefs-harness/tests/pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader.rs`，根记录四条指针各改一类字段、两种挂载都退到上一条根的在 `crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs`；池级 checker 的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：extent 偏移落在墓碑树、谱系树的容器子指针上（`an_extent_offset_on_the_container_pointer_of_the_tombstone_tree_root_reddens_only_the_pointer_head_invariant` 与谱系树那一份），算法类型、压缩算法码、压后长度各一份（`an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant`、`a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant`、`an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant`），根槽算法类型、nonce、MAC 各一份（`a_root_slot_whose_algorithm_type_is_one_reddens_only_the_header_checksum_coverage_invariant` 等三条）；对应的账是 C581（extent 偏移非 0 没有判据）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行、`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行、`research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L4 的容器子指针那一行、`research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2 的算法类型等三段与旧树表根指针两行、`research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 的根记录四条指针那一行） |
| I-2.5 | 位置条目按设备身份升序 | 任一块指针、中央映射条目的 value、journal 点名项里的位置条目数组按设备身份严格升序（D19（块指针的结构与宽度预算） 已定项 11、D23（journal 的角色与格式） 已定项 17，2026-09-13 用户定案）；两条位置条目设备身份相同或逆序，一律判损坏。⚠️ **射程（2026-09-14 补）：整条 86 字节全零的指针表示「注册了但还没有根」，整条豁免升序判定**——它内部那两条位置条目都是 (设备 0, 槽 0, 校验和 0)，按字面正是「设备身份相同」那一档。今天写出全零指针的有四处：树表里 livelist 树、稀疏旁表树与 deadlist 树那三条条目（D6（快照实现模型） 已定项 2、D5（快照 / 空间记账机制） 已定项 6 与 D5（快照 / 空间记账机制） 已定项 11 各自定案的「day-1 注册进树表，根指针为零」），以及 mkfs 与暖机的根记录里那个还没发布过的中央映射树根指针。非全零的指针不享这条豁免。 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |

```

**出处 `.claude/kb/invariants.md:136-171`（整段抄，未转述）**

```markdown
## I-3 空间记账 —— 类

**依赖 D5（快照 / 空间记账机制）。**
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**2026-09-17 起「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）。遍历的是最新根与不被抛弃、不低于 F 的有效根（RGall：候选集不再滤一圈窗口，环上有洞时洞那一槽里留着的上一圈的根只要不被抛弃、不低于 F 也走；代码 `crates/singlefs-checker/src/walk.rs` 10322 行一带）。F_生效 由 checker 自己算：取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与层 0 不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于回收门槛的（还在 defer 里，分配器还不许回收；回收门槛 = max(F_生效, 环里最旧有效根)，D16（发布语义） 已定项 1「可再分配」那一行），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15，走那条记录新根段里的实例表指针与分配记录树根（`walk::walk_version_applied_only_by_records`；用户 2026-10-02 定新根段回到三字段，实例表指针与分配记录树根随新根段换成那次发布的）。⚠️ **没量的前提**：「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`research/prompts/m2-rev-b2-implementer-report.md` 第七节第 2 条）；量它是 C699（取号跳过改认出生代号） 的第一步。 ⚠️ **钉最旧 txg 的代价（用户认的）**：读坏槽与读回全零的槽把回收门槛钉在它可能住过的最旧 txg（D16（发布语义） 已定项 1「根槽这一次读坏」），这样的合法盘面上 I-3.1（已分配统计对得上） 红在记账多的一边，到钉解开为止（这个进程写掉那一槽，或进程内记根时重读读出有效根、读出被抛弃的根，`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant` 文档注释）；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 三条慢档用例在第 10h 批改成只许红在记账多的一边（用户 2026-10-04 定钉最旧 txg、2026-10-05 定全零槽钉门槛而不并进上限，`records/2026-09-30-里程碑三开工第一轮.md` 第 10b、10c 批）。 |
| I-3.2 | 已停用·方向反了 | **此编号不再使用**——它写成「≤ 引用它的**最新**快照的代号」，方向反了：紧的约束是最**旧**的那个。旧写法被 I-3.5（引用区间的精确性） 的子判据 ② 严格蕴含，且检不出任何一类真实错误 | 不适用 |
| I-3.3 | 已停用·归属未定义 | **此编号不再使用**——「所属快照」未定义，而两种归属约定给出不同的比较符。由 I-3.6（deadlist 紧界） 取代 | 不适用 |
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ①② 合成一条**出现规则**：取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判。R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，`Reach(b)` 在 txg 序上是否连续不再单判）满足 (a) `birth(b) > R⁻.txg`；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent 树、inode 树，`deleted_inodes` 树根非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）。`birth(b)` 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配）。**缺口**：R.txg ≠ R⁻.txg + 1 时这一对之间有洞，分两种处置——最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）；不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`。R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由（同 I-4.9（非最新候选根引用的单元不晚于它） 候选集只剩一条根的写法）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中。⚠️ **射程**：缺口的来源是新实例第一条根的 txg 取根环与 journal 里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs` 取号那一步），所以写行那次发布的记录已落而根槽没落、多记录发布只落一部分之后再可写挂载，必出现缺口；另有恢复丢掉尾巴记录、根槽写失败重发跳一格（今天的代码走不到，随失败处置在后面的里程碑做）、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）。**盲区**：缺口右边那一对里，落在用户可见树子树里的新写单元（新叶）的诞生代号被改小到 R⁻.txg 以下，判不出；整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像同样判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18（块里携带什么信息） 已定项 24），`crates/` 今天没有。**谱系记录落地之后**，(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形与缺口条款并存（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 9 行）。**判别力**：诞生代号改小到恰好等于 R⁻.txg 的新写单元必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像不许红。**推翻条件**：任一合法镜像（池级 checker 与层 0 其余判定都不红）上红。 | 部分实现：出现规则（TF、β、gap、gap-row、N3）已实现，池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions`（`IMPLEMENTED_INVARIANTS` 下标 65；坏镜像在 `crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs`；变异在 `crates/mutations.tsv` 里「第8批 I-3.5」那几行）；③ ④（deadlist 那两条）未实现（实现进下一批）。出现规则取按树继承下界加 β 出现规则（用户 2026-10-02 定 ① 取 β）加缺口处置；缺口处置用户 2026-10-03 定 gap 加 N3（判决里的局部标号，只放过用户可见树里的单元）加 gap-row 两样都加。**被攻过几轮**：按树继承下界加 β 加整对不适用的缺口，在合法镜像上两轮没红（各一次抽样）；gap 加 N3 加 gap-row 的合成形、同 txg 几条并成一条、`deleted_inodes` 非全零才算继承、读不出头的单元不计入这三处措辞、谱系记录落地之后的区间形，都是零轮，终轮不再开轮，实现时再攻。实现挂在池级 checker 的主走读上（每版落点集合快照、用户可见树标签、压得住的判、新判定函数、位置键折叠、`IMPLEMENTED_INVARIANTS` 加一条、用例与变异），I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查，与这一条共用同一个「候选集有洞」判法（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 4、5 行与第四节）。② 的上界半（`birth(b) ≤ R_first.txg`）另立 I-4.9（非最新候选根引用的单元不晚于它）（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）。⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
| I-3.6 | deadlist 紧界 | 任一 deadlist(S) 中的块满足 `birth ≤ prev(S).txg`（约定 B 下的紧界）。判别力：抓「kill 时该走直接释放却进了 deadlist」这一类（泄漏方向） | 未实现 |
| I-3.4 | 可用空间扣待删占用 | 待删中 inode 没释放的单元计在记账「已分配」里（释放只进 defer），记账标签 4（待删除但意图未完成占用）恒 0，可用空间统计因此已扣除待删占用；准入式子另扣环窗口常驻项（D28（挂载期承诺量） 已定项 1）。来源 `research/prompts/m3-delete-r3-main-verification.md` 第三节判标签 4 那一格，被攻过零轮 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_pending_deletion_occupancy_of_a_version`；坏镜像两份：标签 4 那一行的值非 0、整行缺失，登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`；GPU 判器下标 70，扫描内核判）。池级 checker 只判「标签 4 恒 0」：判的每一版里，走全了的记账树都要有标签 4 那一行且值为 0，缺行判红；另两句不在这一行判：计在「已分配」里归 I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读），可用空间已扣待删占用归 I-5.2（空闲统计对得上），即空闲加已分配等于容量；判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法；出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节） |
| I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少） | 未实现 |
| I-3.8 | 实例表行唯一且低于挂载根 | 实例表 kind 0 行按实例代号唯一；行只在回收条件成立后删（整轮清扫对池中每一个落点都得出判定、其中没有该实例的未发布单元、那次清扫里没有读失败 ∧ 全部设备在线 ∧ 根环里没有该实例发布的根、且根环每个槽都读成功）；行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2） | 已实现（2026-09-16，`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_rows`：行按实例代号唯一、每行实例代号低于最新根的实例（记录形态归 I-3.14（实例表每片记录形态合法））；最新根的实例表第 0 片解不开时报不适用，理由点名这一片解不开、行没收全；回收那一半没有输入，第一版行只增不删，删行的条件没人判） |
| I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条层 0 流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判）。**判法收窄在实现途中**：今天任一候选根走不全就整条报不适用；收窄成只在比最后引用它的那条根晚的候选根都走得全时才判这一条已释放记录，攻方造出的对照格镜像（抬 F 的空发布之后再覆盖写，把最后一次空发布那条根的树表翻坏）钉进用例（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行） |
| I-3.10 | 已分配记录的分配代等于它罩住的单元的诞生代号 | 任一**未带已释放标志**的分配记录，它的分配代（D3（空间分配） 已定项 3：`value = 分配代`；盘上编码见 D3（空间分配） 已定项 7）等于它罩住的那个单元头里的**诞生代号**（D18（块里携带什么信息） 已定项 7 的类身份段）——记录与单元是同一次发布写出来的，两个数因此同源。⚠️ **射程**：① 只管未释放的记录，已释放的那一半归 I-3.9（释放代落在停止引用它的那一格区间里）；② 记录罩住多个槽时（数据单元跨两槽）取起点槽那个单元头；③ **那个槽上读不出可用的单元头时这一条没有对象**（报不适用，不报违例）——读不出本身由 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配） 管；④ **只读回退候选集里那几版的分配记录**（候选根的分配记录树、树表 0 条那一版根记录直接持有的那一片、由记录施加出来的那几版），**再加下一次挂载会先施加的那一版**：下一次挂载的恢复照 D23（journal 的角色与格式） 已定项 14 的重放规则施加的**第一次**发布（与 I-4.1（重放后树可遍历） 同一份重放，所选根取最新根），取它末条那一份新根段里树表指着的分配记录树；一次都不施加、锚点认不定、所选根的 txg 没有后继时没有这一版，前缀停在后面某次发布的点名单元上时仍取第一次（`walk::version_the_next_mount_applies_first` 与 `walk::replay_the_next_mount_would_do`；认法依据 `research/prompts/m3-formal-code-b36-r1-main-verification.md` 第二节 P1 与第三节第 1 条，取末条那一份依据 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1；主 agent 2026-09-24 按代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第四节第 2 条定接这一步，被攻过零轮；只接这一步——恢复连着施加几次发布时第二次及以后那几版读不到，欠在欠账表）；不读被实例表判抛弃的根与低于 F 的根：低于 F 的根那棵账里未释放的记录罩住的槽，在 F 抬过之后可以被合法回收、换成诞生代号不同的单元，照「任一」去判会在合法镜像上红（主 agent 2026-09-23 定，被攻过零轮）。⚠️ 「再加下一次挂载会先施加的那一版」那一段的同源前提，靠 D16（发布语义） 已定项 7 持久顺序里单元与记录之间那道屏障撑着：拿掉那道屏障、又复用回收槽时，记录已持久而复用槽上仍是旧单元，恢复按点名验证拒施加、结局正确，这一条却在那个槽上红（2026-09-28 主 agent 复跑 floorreuse4：删屏障 271 个状态红 48、带屏障 73 个状态 0，`research/prompts/k3-9-barrier-r1-main-verification.md` 第一节）；用户 2026-09-28 定屏障维持现状、checker 不改，欠在 C592（单独拿掉单元写与记录间那道屏障今天没判据会红）。**它拦的是什么**：复用一个已回收的落点时改写了那条分配记录、却把上一次的代留着（2026-09-22 增补 3 第 1 件的代码三方第一轮攻方腿造出这条变异，快档与大档都判不出，因为全仓没有一条不变量管已分配记录的分配代） | 已实现（2026-09-23，待代码三方与崩溃验证；池级 checker `walk::judge_allocation_generations_against_unit_births`，读法照射程 ④，同一条记录（盘、槽、代）只判一次，射程 ③ 按记录逐条跳过、一条都没判到才整条报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 下一次挂载会先施加的那一版由 `walk::tree_table_pointer_of_the_version_the_next_mount_applies_first` 并进读集；它的坏镜像（发布 B 的记录已落、根槽没落，B 那棵分配记录树里一条未释放记录的分配代改成诞生代号 + 1）是 `checker_known_bad_images.rs` 的 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` |
| I-3.11 | 已分配减 defer 等于最新根走读 | 逐盘：被判镜像上记账树里「已分配」那一行（记账第 1 项）减「defer 待释放」那一行（第 5 项），等于从最新有效根走读到的、这块盘上被引用的槽数。统计量的标签与行形态是 D5（快照 / 空间记账机制） 已定项 4 与已定项 10 的那几行，读的是镜像里的记账行、不读内存里的分配器；走读与 I-3.1（已分配统计对得上） 同一个走法，只取最新有效根。⚠️ 它补的是 I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 都够不着的那一种：写者把活单元记成已释放、放进 defer，已分配与空闲两个和照样对得上（alloc-basis 第三轮判决 `research/prompts/alloc-basis-r3-main-verification.md` 的 Z10）。⚠️ **判别力自证在造出来的基底上做**：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑在造出来的基底 β_syn（第 5 项为 0）上再记 1 槽 defer，I-3.11（已分配减 defer 等于最新根走读） 判红、去掉「减 defer」判绿（`research/results/e156-alloc-basis-counts-2026-09-26-r2.out` 的 `E7RESULT name=q7c1_not_subtracting_defer basis=beta_syn k=1 real_check_is_red=true without_subtracting_defer_is_red=false flips_red_to_green=true`）。可达状态里第 5 项能不能为 0 没量到：挂载时回退（回退写行那次发布）今天没有了，同一份产物里挂着回退那一版第 5 项是 356 槽（`E7RESULT name=u13a_rollback_to_oldest_candidate overwrites=72 target_txg=52 txg_before=75 txg_after=76 instance_before=1 instance_after=1 instance_table_rows_before=0 instance_table_rows_after=0 item1_slots=374 item5_slots=356 i311=holds structure_holds=true`）。隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定「核出坏的才豁免」，按单元的读法是主 agent 同日定的，D19（块指针的结构与宽度预算） 已定项 5 的隔离）。 | 已实现（池级 checker `walk::check_pool_image`：逐盘判「defer 待释放 + 最新根走读 == 已分配」，最新根走读取 `walk_root(最新根)` 刚走完那一刻、与 I-3.1（已分配统计对得上） 同一个走法；最新根下面没有记账树时报不适用。坏镜像 `live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant`（盘 0 defer 行 +1 / −1 槽，只红它，I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 照旧真被评估过且成立）；判别力自证在造出来的基底上 `on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant`（两盘 defer 挪回 0 之后再记一槽）；`crates/mutations.tsv` 5 条变异（判定恒真、不减第 5 项、`==` 放宽成 `≥` / `≤`、拿全部候选版本那一份走读比）；用户 2026-09-23 定采纳：alloc-basis 岔路单第 12 行（岔路 7）） |
| I-3.12 | 实例表行 T 低于引用它的根的 txg | 根环里任一自证过的根 R，它指着的实例表的每一行 (i, T, W) 满足 T < R 的 `checkpoint_txg`；链上某一片读不出的根不判。依据 D18（块里携带什么信息） 已定项 11（恢复写的行，所选根那个实例的 T 是重放之后那个根的 txg，中间实例写 0）与 D23（journal 的角色与格式） 已定项 14（新实例第一次发布的 `checkpoint_txg` 大于环里全部根与记录的号）。射程是每条根各自指着的那张表，不是只有挂载根那一张（那一张归 I-3.8（实例表行唯一且低于挂载根））；同一实例的行在两张表里是否逐字节相同不判，条款没有这一句。 | 已实现（2026-10-02，池级 checker `walk::judge_instance_table_rows_against_the_roots_referencing_them`：每条自证过的根各沿自己的实例表链判，链上有一片读不出的根跳过，一行都没判到时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 6 行） |
| I-3.13 | 记账行代段等于这一版的 txg | 走读到的任一版（候选根，以及由记录施加出来的那几版）的记账树里，每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`，只判走全了的树。依据 D5（快照 / 空间记账机制） 与 D16（发布语义） 已定项 6 的「记账代 = checkpoint 号」（每条根的记账树只装这条根那一代的行）。 | 已实现（2026-10-02，池级 checker 走读里的 `walk_tree_table_entry` 对每一版的记账树判，版本含由记录施加出来的，只判走全了的树；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-3.14 | 实例表每片记录形态合法 | 走读到的实例表每一片里：每条记录的 `kind` 只有 0（行）或 1（链指针记录）；链指针记录恰好是这一片的最后一条、片中间不出现，且自身合法（有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零，且位置指针头部 MAC / nonce、算法类型、压缩算法码与压后长度、extent 偏移都是第一版的取值：有一段不是第一版的取值，这一片也解不开）；`kind` 0 行的 flags 字节为 0。依据 D18（块里携带什么信息） 已定项 11（`kind` 不是 0 或 1 ⇒ 这一片解不开；flags「第一版恒 0、非 0 拒收」；链指针记录「恒为一片的最后一条」，最后一条不是合法的链指针记录同样解不开，位置指针头部的第一版取值见 D19（块指针的结构与宽度预算） 已定项 3），用户 2026-10-06 定另立；头部那一形用户 2026-10-06 定算第五种形态（岔路单 `research/prompts/m3-batch10o-code-r1-forks.md` K2-1 那一行） | 已实现（2026-10-06，`crates/singlefs-checker/src/walk.rs` 的 `judge_instance_table_page_record_forms` 判前三件、`walk_instance_table_chain` 里链指针记录那一格判最后一件，走到的每一片都判；不成立的一片解不开：行不收、链不往下走、候选集滤不全（I-9.29 的门关）；只读不判的读法与引用集合那一遍同一个读法；GPU 走读内核同判，扫描内核的引用集合那一段与只读读法、恢复内核读实例表同判 flags；有下一片而位置指针头部有一段不是第一版取值的那一形：CPU 三处读法（主走读、只读不判的读法、引用集合那一遍）经 `crates/singlefs-checker/src/walk.rs` 的 `chain_record_view` 判，GPU 走读与扫描经 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` 的 `w_chain_record_kind` 判，恢复内核读实例表同判；坏镜像两张（位置指针 MAC 不全零、extent 偏移非 0），在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`） |

> ⚠️ **I-3.5（引用区间的精确性）四条子判据双侧夹死 与 I-3.1（已分配统计对得上） 正交，两条都必须有。** I-3.1（已分配统计对得上） 是标量总账（已分配 == 遍历和），
> 而**归属错误对它不可见**——块被挂到错的 deadlist 上时，它仍然被恰好一处引用，总账依然平。
> 要检出归属错，必须有**按快照分箱的向量等式**。
> 这与 [decisions.md](decisions.md) D14（双轨（大小文件 / 持久临时）） 已定项 6 推导 D-3（多轨兼容可判据） 里「标量等式 → 按轨道分箱的向量等式，
> 判别力净增」是同一个形状。
>
> ⚠️ **数据丢失方向只在空间被重新分配之前可检出。** 该进 deadlist 却被释放的块，
> 一旦其空间被复用，症状就退化成 I-2.1（校验和与内容匹配） 校验和不匹配，即「损坏，原因不明」。
> **所以这类检查必须在崩溃点重放 harness 里紧跟着删除动作跑，不能只在全量里跑。**
>
> ⚠️ I-3.1（已分配统计对得上） 是**唯一允许全盘遍历的记账检查**——它是 checker，不是运行时路径。
> 运行时记账必须增量维护（D5（快照 / 空间记账机制）），checker 才用遍历来验证增量维护有没有出错。
> 这条对照关系是本工程验证记账正确性的核心手段。

```

**出处 `.claude/kb/invariants.md:172-190`（整段抄，未转述）**

```markdown
## I-4 事务与崩溃一致性 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-4.1 | 重放后树可遍历 | 任一崩溃点重放后，所选根（重放之后那一版）指向的树全部可完整遍历。起点含介质损坏的镜像（前缀停在一条点名单元两份没都验过的记录上）报不适用。另三处报不适用：所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版）；所选根的 checkpoint_txg 是 u64 的最大值（接在它后面的发布取不到 txg）；所选根之上一次发布都不施加（重放之后那一版就是所选根，可不可遍历归 I-7.2（最新根可完整遍历））——没有记录压在最新根之上的镜像上这一条因此恒报不适用、不报成立。重放之后那一版的树表指针全零时红：core 的根记录读者不把全零当特例，照它那两条位置条目读盘 0 槽 0、读不出（两种挂载都拒），另起的走读跟过去、记一次走读失败（`walk.rs` 的 `Walk::walk_version_applied_only_by_records` 里的 `follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`；用户 2026-10-06 弹窗定，选项原文「甲 两类都判损坏」，`records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行）；重放施加了几次发布时判的是最后那一版，前面某次发布全零而最后一版完好时不红，core 也照常挂上。重放用的读环按 (实例代号, 计数器) 建键，两块盘同一计数器坐着不同实例的记录时两条都留，与实现恢复同键（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2、8 行）。 | 已实现（2026-10-02，池级 checker `walk::judge_trees_of_the_replayed_version_are_traversable`：R-独，checker 自己照 D23（journal 的角色与格式） 已定项 14 的前缀规则复刻只读重放（`replay_the_next_mount_would_do`），施加出来的那一版（所选根换上末条记录新根段的七项：树表指针、中央映射树根指针、树 ID 水位、F、实例表指针、分配记录树根、卸载记号，只有 fsid 照所选根；D23（journal 的角色与格式） 已定项 15）另起一份走读，走读断了判红；在飞上限按环长 ÷ 4096 ÷ `JOURNAL_SAFETY_FACTOR` 现算。读环键 2026-10-03 改成 (实例代号, 计数器)（`records_the_replay_reads`），反向链那一关改按 (实例代号, 计数器 − 1) 找前一条（`record_back_chain_lets_it_into_the_replay_prefix`），消掉两块盘同一计数器坐着不同实例的记录时与实现分开的形态（推为第一版两盘池上不可达）；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 两处同改按 (实例代号, 计数器) 建键，已做（主 agent 2026-10-03 定）。坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（重放六条规则各一份、两块盘同计数器不同实例两份）；层 0 里与 core 的 `effective_root` 逐状态差分今天没有能跑的入口，要先开口子，进下一批清单（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 8、9 行）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 2 行与第四节用户定案） |
| I-4.2 | 无被引用未提交块 | 任一崩溃点重放后，不存在被引用但未提交的块 | 已实现（2026-09-22，池级 checker `walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`：走读到的每个被引用单元，头里的诞生代号与写序（D18（块里携带什么信息） 已定项 7 单元头字段表里的字段）按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判成已发布——**「未提交」取的就是「按那个谓词判不成已发布」**，谓词与 I-1.8（归并后版本全序） 扫描方向共用同一份 `PublishedPredicate`。打包记录类型 4 的实例表单元按 I-1.2（块头写序已发布） 那一行的例外不走谓词，不进这一条。坏镜像是把数据单元头里的诞生代号与引用它的 extent 记录里那条数据指针的诞生代号一起抬到最新根的 txg 之上——指针仍与头一致 ⇒ I-1.2（块头写序已发布） 不跟着红，分得开两条的就是这一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs`）；层 0 每个崩溃状态都跑它。⚠️ **「任一崩溃点重放后」那半句没实现**：池级 checker 判的是手上这一份镜像、不跑恢复，判的是「被引用 ⇒ 已发布」这个态上的性质；重放之后再判一遍归层 0） |
| I-4.3 | 提交原子 | 提交是原子的：不存在"部分生效"的事务 | 未实现（它要判的是 D23（journal 的角色与格式） 已定项 7 那两个事务边界字段——事务号与提交标记——施加之后的效果。**对着一份镜像判不出来，没硬写**：「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽——同一份字节既可能是「事务整体没生效的旧态」，也可能是「生效了一半」，池级 checker 手上没有第二个输入。判它的是层 0 崩溃点重放加 oracle：`.claude/kb/milestone/02-second-txn.md` 第 486 / 487 行那两条（崩在第二条记录之前 ⇒ 整个事务不施加、文件是旧长度；把提交标记提前到第一条记录 ⇒ 恢复施加了半个事务、I-4.3（提交原子） 红）说的就是这条路。形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查）：操作断言，不进池级 checker，恒真的检查比没有检查更糟） |
| I-4.4 | 意图只描述目标态 | 任一未完成的意图 key，其字段只描述目标状态，不含任何增量字段（"再删 N 个"一类） | 已实现（2026-10-05，在清单里的第一版恒报不适用：池级 checker 在 `walk::check_pool_image` 里对它调 `not_applicable`，理由逐字是 `crates/singlefs-checker/src/image.rs` 的 `INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION`；不要求坏镜像，`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `INVARIANTS_WITHOUT_A_BAD_IMAGE` 点名；GPU 判器下标 73，内核不置位）。第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段，checker 报不适用并写理由；logged_ops 有字段表时实现（用户 2026-10-05 定，岔路单 `research/prompts/m3-delete-checker-r1-forks.md`） |
| I-4.5 | 已停用·需两镜像 | **此编号不再使用**——幂等要比对两个镜像，单镜像判不了；判据落在 [experiments.md](experiments.md) E3（意图日志的断电语义） 判据③ | 不适用 |
| I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致（候选集的深度由 D16（发布语义） 已定项 1 的回退下界 F 定：平时整个根环，盘紧时缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内）。**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的 | 已实现（2026-09-17，池级 checker `walk::check_pool_image`，最新根与候选集里每条更早的根各判一格：从它出发的遍历有单元对不上或读不出就红；坏镜像是 mkfs 树表单元抹成零；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-4.6 | 中间态其余不变量成立 | 存在未完成意图的镜像上，其余全部不变量仍然成立（分批之间的中间态是合法状态）。「其余全部不变量仍然成立」由各条在每个被判的版本上各自判，中间态的版本也在内；I-4.6（中间态其余不变量成立） 自己判的是中间态特有的一格：每条 nlink = 0 的记录，数据单元数与 size 对得上——size 为 0 时 0 或 1 个单元，size > 0 时 ⌈size ÷ 数据单元净荷容量⌉（实现 `crates/singlefs-core/src/write_request_split.rs` 第 61 行 `div_ceil(...).max(1)`）。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size`；净荷容量 checker 自己算一份（`data_unit_payload_capacity_in_bytes`）；每条 nlink = 0 的记录判一次，一条都没有时报不适用并带理由；坏镜像：第一个文件的 size 加一个净荷（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_size_of_the_first_file_by_one_data_unit_payload`）；GPU 判器下标 72，扫描内核判；用户 2026-10-05 弹窗定「只判单元数对 size」，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第四节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`；射程只对 nlink = 0 的记录，每条记录都判的那一形不挂在这一行名下，另记欠账 C711（单元数对 size 只判了 nlink = 0 的记录）） |
| I-4.7 | 删除中 inode 有账 | 任一处于删除中的 inode，要么在删除中队列里有条目，要么其 extent 已全部释放——两者皆非的中间态不存在。「处于删除中的 inode」指两样：inode 树里 nlink = 0 的记录；extent 树里有数据单元、而 inode 树里没有 nlink ≥ 1 的记录的号。判哪几版、配对与读不全的报法照 I-9.5（删除队列里的号仍在树里） 的共用判法 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，函数 `judge_inodes_being_deleted_against_the_queue_and_their_data_units`；每个处于删除中的号判一次，一个都没有时报不适用并带理由；坏镜像：extent 树里有数据单元、inode 树里没有它的记录、号也不在队列里（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `drop_the_record_of_the_first_file_and_keep_its_data_units`）；GPU 判器下标 71，扫描内核判；对象取这两样，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节逐格判决表里 I-4.7（删除中 inode 有账） 对象那一格） |
| I-4.9 | 非最新候选根引用的单元不晚于它 | 回退候选集里不是最新的每一条根 R，它引用的单元（按落点认，读落点上单元头里的诞生代号，头校验和过）的诞生代号 ≤ R 的 `checkpoint_txg`。最新那条根归 I-4.2（无被引用未提交块），不在这一条里。依据 D16（发布语义） 已定项 7（单元先于根落盘）。 | 已实现（2026-10-02，池级 checker `walk::judge_unit_birth_against_an_older_candidate_root` 判走读里每个被引用的单元，共享子树里别的版本先走过的单元由 `merge_placements_below_a_unit_walked_by_an_earlier_version` 补判；候选集只剩一条根时报不适用，同 I-3.9（释放代落在停止引用它的那一格区间里）；坏镜像沿 I-4.2（无被引用未提交块） 那一份的造法，映射 key 也要跟着改，在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行） |

> ⚠️ **编号只往上加，不回收、不重编。** 回收一个编号会让别处的旧引用悄悄指向另一条陈述，
> 而检索只端出其中一条，不会告诉你它换了意思（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」）。
> 退役的编号留一行说明去向，比留一个无解释的空档安全。

```

**出处 `.claude/kb/invariants.md:191-213`（整段抄，未转述）**

```markdown
## I-5 分配器 —— 类
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-5.1 | 物理范围不重叠 | 不存在被两个不同引用同时占用的物理范围（除非是显式的多副本） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） 引用按版判（2026-09-27，`research/prompts/m2-rev-b1-implementer-report.md` 第六节第 1 条、`research/prompts/m2-rev-b2-implementer-report.md` 第八节）：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红，(设备, 槽) 取自位置条目的设备 ID 与物理偏移（D19（块指针的结构与宽度预算） 已定项 4），一条指针自己的两条位置条目同落点算一个引用（那一格归 I-2.5（位置条目按设备身份升序））（`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`）；中央映射条目指的物理范围算进引用集合；别的版本走过的共享子树不重走，按缓存把它下面的落点并进这一版再比、不读盘（`walk::Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`）。坏镜像在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`。 |
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-5.3 | 报出的空闲都兑现得了 | `df` 报给用户的空闲 ≤ 当下可分配 + 能在有界步数内放回的扣住量（defer 窗口里、回退下界以下的释放）。运行时的等价形态：`df` 报出 s 字节空闲时写 s 字节必须成功；删掉 s 字节之后同样大小的写在有界步数内成功（[decisions.md](decisions.md) D3（空间分配） 已定项 9，用户定案）。界随 D16（发布语义） 已定项 1 的形态定。判别力：把「放回扣住量」那一步摘掉，近满盘上删一个对象再写同样大小必须由绿转红 | 未实现 ⚠️ **口径注（2026-09-13，D28（挂载期承诺量） 已定项 4）**：`df` 要扣的 checkpoint 保留池是每次发布现算的量，发布即重算，近满盘上报出的空闲会随树高变小。 |
| I-5.4 | 分配记录罩住的槽互不相交 | 判的版本是回退候选集里每条根，加由记录施加出来、根槽从没落盘的那几版（已被一次恢复施加过、实例表里有行，树表指针与「树表 0 条那一版的分配记录树根」取自记录的新根段；认法 `walk::versions_applied_only_by_records`，另有三条都要成立：txg 不低于回退下界、它换下的单元还没到回收的时候（根环里有一条没被实例表判抛弃的根，txg 比它小）、记录进得了重放前缀；一次发布切成几条记录时新根段取带末条标志那条记录的，读不到带末条标志的那一条时退回先到的那一条）与下一次挂载先施加的那一版（下一次挂载的恢复照 D23（journal 的角色与格式） 已定项 14 的重放规则施加的**第一次**发布，取它末条那一份新根段，与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 认同一个函数 `walk::version_the_next_mount_applies_first`；一次都不施加、锚点认不定、所选根的 txg 没有后继时没有这一版，前缀停在后面某次发布的点名单元上时仍取第一次；根环里没有它的根）；这些版本的分配记录树里，同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交。复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉；留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽。同一条下另判一格：树表非空（带树）的那一版，根记录里「树表 0 条那一版的分配记录树根」那 86 字节必须全零（D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行），非零就是这一版有两棵分配记录树，判违例。这一格成立的那一次，只在这一版自己的分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4 的评估，没走到叶的版本不计。部分没判（逐版列出，不判红；判出违例照报违例）：树表指针非零而树表读不出的版本、树表指针全零的版本（core 的根记录读者照它读盘 0 槽 0、读不出），都认不出带不带树，根记录里分配记录树根全零那一格与它自己的分配记录树都没判；树表读得出、0 条而分配记录树由根记录直接持有（C512（树表 0 条的一版上被换下的实例表记在哪没有条款））、那棵树读不出的版本；带树一版自己的分配记录树没走到叶的版本。这两类由记录造出来的版本里树表指针全零的，这一条自己报部分没判（与根环里的候选根同一处置）；它们另外判损坏：由记录施加出来的那几版与根环里的根同一读法，主走读照 core 的根记录读者跟过去、记一次走读失败（用户 2026-10-06 弹窗定，选项原文「甲 两类都判损坏」，`records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行）；下一次挂载先施加的那一版全零，在重放只施加这一次发布时就是重放之后那一版，红 I-4.1（重放后树可遍历）。候选集里没有一棵分配记录树走得到叶（第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上）报不适用。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_allocation_records_disjoint`，吃一张版本表（`VersionToJudgeTheAllocationRecordsOf`：称呼、树表指针、树表 0 条那一版的分配记录树根），表里是回退候选集里每条根、由记录施加出来的那几版与下一次挂载先施加的那一版（`check_pool_image` 拼表）；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_allocation_records_disjoint_of_a_version`，候选根、由记录施加出来的版本、下一次挂载先施加的那一版各调一次，只有违例位、部分没判不置位；这两类版本上的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant` 与 `two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant`，读不出的几格各有用例（`the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged`、`an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged` 等）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant`；层 0 每个崩溃状态都判，同日在新代码上全量：第二条流 2,104,413 个状态里评估 1,842,252 个、违例 0，其余 262,161 个报不适用，第一条流 262,165 个状态里评估 4 个、违例 0（`m2-wave2-crash-verifier-report.md`「主 agent 要的两个数」）。同日立，依据 `research/prompts/m2-wave2-code-r1-main-verification.md` 第二节第 1 行：复用时跨度变了留下重叠记录，下一次可写挂载在重建分配器时 panic，而 I-5.1（物理范围不重叠） 只管树里的引用、判不出记录之间的重叠；带树一版分配记录树根全零那一格的挂法见判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法」那一行与 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2「成立要至少一棵树走到叶」那一行，树表读不出与根记录直接持有的树读不出两形见 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」那一行与 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2 第一行，射程含两类由记录造出来的版本见同一节 P2「由记录施加出来的版本与下一次挂载先施加的那一版」那一行，树表指针全零的版本见同一节 P2「树表指针全零的根」那一行与第五节第 1 条；下一次挂载先施加的那一版改取重放施加的第一次发布见 `research/prompts/m3-formal-code-b36-r1-main-verification.md` 第二节 P1 与第三节第 1 条，由记录施加出来的那一版取带末条标志那条记录的新根段见 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b，两类由记录造出来的版本树表指针全零判损坏见 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P2 与第五节第 1 条） |
| I-5.5 | 从 mkfs 水位建树那一版的 t1 落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若是从 mkfs 水位第一次建树的那一版——① V 的树表非空，且种类 1（extent 树）条目的树 ID 等于 65536；② V 的树表里每条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`——则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号等于这一版分配记录里 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、两槽都空、不在开放段里）。新池新建文件那一次是 U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）；回收之后（例如 mkfs 的实例表 U、U + 1 那两槽被回收）不再是 U + 4。V 没有数据单元时不判；不满足 ①② 的版本不判。这一类含新池新建文件，也含 mkfs 之后重挂、实例 1、2、3 从 mkfs 水位写的第一个文件（用户 2026-10-04 定案，原话在决策变更史 D3（空间分配） 节）。只判 t1，t2–t12 不判（D3（空间分配） 已定项 10 射程没有「偏离即不合法」）。已知限度：这一版盘面上复算不出的几样只能近似——已释放的记录一律当占着（回收门槛复算不出：写死从 U + 4 起扫的坏实现与回收门槛放宽的坏实现两个方向都判不出，C689（建树落点判据抓不到写死从 U + 4 起扫的坏实现））；这一版自己分配的记录当空；开放段拿这一版树表单元所在的段当它（这次挂载更早开过、已经用尽的段认不出；提交内生块回落时会误排：这要求这一版发布时每块盘上都没有全空的 64 槽段，今天的写侧在「只暖机 / 只写行的挂载接发第一个文件」这一族里量到 0 次回落（单元区 240 / 256 / 384 槽，连续挂载 k ≤ 60，一次抽样、不是证明）；分配器上留了回落计数，`crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs` 断言它是 0，哪天走进回落、用例红，回来重看开放段认法）；隔离位、抬 F 扣住位与保护期内的 R 盘上都没有，没有分配记录罩着的槽一律当空。依据 D3（空间分配） 已定项 8 射程与已定项 10 ③ ④，以及判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2 与用户 2026-10-04 对判据改法的定案（原话在决策变更史 D3（空间分配） 节）。 | 已实现（判据是 f2（按这一版分配记录现算），代码轮判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`；池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark`：候选集里每条根与由记录施加出来的那几版各自重读树表与 extent 树判，U 取所选系统配置偏移 417；GPU 判器扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_first_data_unit_of_a_version`，`IMPLEMENTED_INVARIANTS` 下标 67；判不了的几格报部分没判（`InvariantVerdict::PartiallyJudged`，没有违例时整条报它，逐版列出实例、txg 与为什么，形态同 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同））：这一版的树表读不出（含树表指针全零：core 的根记录读者照它读盘 0 槽 0、读不出）或条目宽不到 200 字节，认不出这一版是不是这一类；认出来的那一版 extent 树读不全（数据单元数不全）、分配记录树读不全（落点算不出）；种类 1 条目不止一条、种类 3（分配记录树）条目不是恰好一条，这两格的树表本身由 I-9.16（树表条目按树 ID 严格升序且合发号次序） ③ ④ 判红、这里只记没判；有数据单元而池里某块盘上一个都没有，那块盘没有对象，跳过、不记（CPU 与 GPU 一致）；GPU 判器只有违例位，这几格不置位；走读到的版本里一版都不是从 mkfs 水位第一次建树的那一版时，整条报专门的不适用理由（`walk::NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK`，`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 里那条用例的 I-5.5 一支只认它）；认出了那一版、而池里的盘上都没有它的数据单元（extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上），又没有别的版本判到时，整条报另一句专门理由（`walk::VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT`，用户 2026-10-06 弹窗定，选项原文「甲 另写一句」，`records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行）；三格的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_extent_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（数据单元从 50180 搬到 50182），合法镜像在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |

> ⚠️ **I-3.1（已分配统计对得上） 与 I-5.2（空闲统计对得上） 的等式在 zoned 布局上有一个已知缺口（[decisions.md](decisions.md) D12（目标介质） 定了多布局之后才出现）。**
> zoned 设备上一个 zone 里只要还有一块活数据就不能 reset ⇒
> **一个活快照钉住的物理空间是 zone 粒度，而它引用的块数是块粒度**。
> 于是「已分配 == 遍历所有引用得到的和」在那套布局上不成立，
> 除非把「已分配」明确定义为「被引用」，**并另立一个「不可回收」量**。
>
> 连带一条对用户可见的要求：**报给用户的「这个快照占了多少」不许被定义成引用块数之和**——
> 那个数在 zoned 上会**系统性地低于**删掉该快照真正能拿回的空间，
> 而 [pitfalls.md](pitfalls.md) 第 12 条骂的正是这类「`df` 输出对用户没有意义」。
>
> **本缺口尚未有对应条目**，补它需要先定义「不可回收空间」这个量。

```

**出处 `.claude/kb/invariants.md:214-263`（整段抄，未转述）**

```markdown
## I-6 加密不变量 —— 类

**依赖 D9（加密），17 个分项全部已定。**
**这些检查在加密未开启时应当整类跳过并显式报告为「不适用」，不许静默通过。**

<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-6.1 | nonce 不重用 | 加密开启时，全盘任意两个**已写出**的块（不论当前是否仍被引用），其 (密钥代号, nonce) 二元组不相同 | 未实现 |
| I-6.2 | 明文头字段白名单 | 加密开启时，任一单元的明文头字段集合 ⊆ 它那一类的白名单，**两张，按 D18（块里携带什么信息）已定项 11 登记表的码分**（码 1 数据单元类；码 2 / 3 元数据类；D18（块里携带什么信息）已定项 7 收口）——共同前缀两类都许：{magic / 格式版本 / flags（含单元类标签位）/ 头校验和 / **nonce 代号** / MAC}；元数据类**仅此而已**（五元组 / key 区间等身份字段一律进密文）；数据单元类另许 {fsid / 载荷 CRC}。⚠️ **2026-09-07 用户定案（D18（块里携带什么信息） 已定项 13，臂庚）按两处收窄**：① 码 1 的 {五元组 / 诞生代号 / 写序} 从「另许」里去掉，就地变密文；② **`声明长度` 从共同前缀里去掉**，三类一起变密文——它给出一个装得下的小文件的精确字节数，是收窄前泄漏面里最重的一项。字段的**位置与宽度一个都没动**，变的只是「盘上这几个字节是明文还是密文」 | 未实现 |
| I-6.3 | MAC 与密文匹配 | 加密开启时，任一被引用的块，其 MAC 与密文及关联数据匹配；**关联数据的每个字段由 checker 从权威树的 key 现算，不许从指针或明文侧读回**（需要密钥在场） | 未实现 |
| I-6.4 | 无未加密块 | 系统配置声明加密时，不存在算法类型字段标为「未加密」的数据块或元数据块 | 未实现 |
| I-6.5 | nonce 水位单调 | 记录在案的 nonce 水位 > 全盘出现过的最大 nonce（崩溃后不重用的可判定形式） | 未实现 |
| I-6.6 | MAC 长度一致 | 任一指针记录的 MAC 长度与系统配置声明的一致 | 未实现 |
| I-6.7 | 副本共用 nonce | 同一份数据的多个副本，其指针记录的 nonce 相同（nonce 未绑物理位置的可判定形式） | 未实现 |
| I-6.8 | 密文校验和匹配 | 加密开启时，任一被引用的块，其指针记录的密文校验和与盘上密文匹配（**不需要密钥**） | 未实现 |
| I-6.9 | 密文过而 MAC 败判身份错配 | 加密开启且密钥在场时，任一被引用的块，若**密文校验和通过而 MAC 失配**，判红并归类为「身份错配」——**不许归类为介质损坏，不许触发副本修复或投毒** | 未实现 |
| I-6.10 | 已停用·明文映射层不存在了 | **此编号不再使用**——2026-09-07 D9（加密） 已定项 5 改取「只做 scrub」，不设明文映射层、不设明文反向索引 ⇒ 这一格**无对象**。曾要求：加密开启且密钥在场时，checker 从权威侧重建出的明文映射层与反向索引与盘上明文副本逐条相同、差异判红不许就地修复 | 不适用 |

> ⚠️ **I-6.1（nonce 不重用） 只在「不论是否被引用」这个范围下才有判别力。** nonce 重用最危险的一批数据
> 恰恰是崩溃遗留的孤儿块——已写盘、未进树、因而不被引用。
> 把检查范围写成「被引用的块」，等于在设计上排除掉最该查的那一批。
> 因此 checker 必须两阶段：① 遍历树可达块；② **额外全盘扫描已分配物理区间**，
> 靠块的明文头识别「这是一个已写出的 singlefs 块」并取出其 nonce 纳入去重集合。
>
> ⚠️ **阶段 ② 对 D9（加密） 已定项 4（nonce 载体）提出一条硬约束：nonce 的代号部分必须放在明文头，
> 无密钥可读，不能整体埋进密文。** 埋进密文的话 checker 没有密钥就连「这块用过哪个 nonce」
> 都判断不出来，I-6.1（nonce 不重用） 的阶段 ② 与 [experiments.md](experiments.md) E5（nonce 唯一性 × 崩溃点重放） 的判据③一起失效。
> I-6.2（明文头字段白名单） 白名单里那一项因此写作「nonce 代号」而不是含糊的「序号」。
>
> ⚠️ **I-6.1（nonce 不重用） 是抗 bug 检查，不是抗攻击检查。** 阶段 ② 靠明文头 magic 枚举已写出的块，
> 而一个能写裸设备的攻击者抹掉 magic 就能让块从扫描里消失。
> 对抗场景下它给不出任何保证——[decisions.md](decisions.md) D9（加密） 已定项 6 记了适用范围。
>
> I-6.2（明文头字段白名单） 与 D18（块里携带什么信息） 已定项 3 的打架已按
> 「两张白名单、按单元类分」收口（2026-09-02，三方论证判决）；扫描器解密前先读
> 共同前缀里的类标签位分出类，再套对应那张表。**头校验和罩仍在明文的部分，MAC 罩密文**（覆盖范围见 I-2.4（头校验和覆盖范围）；加密是系统配置级全有全无），
> 加密元数据的头完整性由两者分层。仍挂在
> [checks-owed.md](checks-owed.md) C68（不变量白名单与已定条款打架） 的那一半是
> nonce 12 字节的归属三处打架与门禁比对工装。
>
> ⚠️ **I-6.2（明文头字段白名单） 的白名单是「允许出现在明文头」的清单，不是「可以拿来判定」的清单。**
> 里面每一项都只是提示：**必须在密文侧有权威副本，读者取密文侧那一份。**
> 从明文侧取期望值等于让攻击者控制判定的输入。依据见 [decisions.md](decisions.md) D9（加密） 已定项 6 的「期望值来源」一节。
>
> ⚠️ I-6.1（nonce 不重用） 与 I-6.5（nonce 水位单调） 是**唯一能把「nonce 重用」这类灾难变成可检出事实的检查**。
> nonce 重用在 ChaCha20 这类流密码下直接泄漏明文，而它不会表现为任何读写错误——
> 没有这两条检查，它永远不会被发现。验证手段见 [experiments.md](experiments.md) E5（nonce 唯一性 × 崩溃点重放）。

```

**出处 `.claude/kb/invariants.md:264-278`（整段抄，未转述）**

```markdown
## 待补

以下类别在对应决策定案前无法写出：

- 反向索引一致性（D1（数据可移动性 / 反向索引） 定案后：正向引用与反向索引必须双向可达且一致）
- 条带 / 多副本布局（D2（RAID 条带策略） 细化后）
- 快照可见性（D6（快照实现模型） 定案后）
- 索引结构自身的结构不变量，如树的有序性、平衡性（D8（核心索引结构） 定案后）
- 密钥层级与换钥（主密钥槽 80 与 KDF 标识 4 的宽度与落点 2026-09-13 由 D22（单元原子性怎么合成） 已定项 9 定死；还剩主密钥槽的内部构成与换钥不重写数据）

---


---

```

**出处 `.claude/kb/invariants.md:279-321`（整段抄，未转述）**

```markdown
## I-9 inode 树结构 —— 类

**依赖 D8（核心索引结构） 已定项 6，形态已定（码 2 内部节点 + 码 3 叶容器、记录定长 140、inode 号单调不复用），checker 未实现。**
与 I-1（块自描述） 类的分工：I-1.6（类标签登记且三处相等） 管单元自己的类标签合不合法，I-1.7（打包容器合法与判定顺序） 管一个容器自己合不合法并且任一步失败即容器级中止；
I-9（inode 树结构） 管跨单元的结构约束（谁该指向什么、号与 key 的关系）与对象级（一条记录）的判定，对象级判红只让那条记录 EIO，不进 I-1.7（打包容器合法与判定顺序） 的中止链。
**共同前置**：checker 要知道一棵树是 inode 树才能对它判这一类，而树表条目里有没有「树的种类」一段无落点（D8（核心索引结构） 已定项 8）；第一版镜像上可由打包记录类型 2 容器头里的出生树反推哪棵树是 inode 树，所以这一类今天不是恒不评估。
<!-- doc-lint:registry name-col=2 -->

| ID | 简称 | 不变量 | checker 状态 |
|---|---|---|---|
| I-9.1 | inode 树根恒码 2 | 树表指向的 inode 树根，共同前缀偏移 6 恒为 2（索引节点），哪怕树只有一片叶或是空的（零条目、key 区间 [0, 0]）——D19（块指针的结构与宽度预算） 的指针不带被指对象的类与大小，根的类若会变，读者按 16 KiB 读 32 KiB 是静默误读 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.2 | 条目身份与子头相符 | inode 树内部节点的每条条目：类型段 ∈ {0, 2}；类型段 0 ⇒ 子单元偏移 6 = 2 且身份引用的出生树、容器号、出生代三段为 0；类型段 2 ⇒ 子单元偏移 6 = 3，子头的 (出生树, 打包记录类型, 容器号, 容器出生代) 与条目的身份引用四段逐字相等，子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 2（出生树那两处 2026-09-12 随 C297（inode 内部条目里出生树两份没有定处理） 用户定案加）；其余判该条目损坏。孩子头里的类与类型段说的那一类不一致时（条目照判损坏之后）怎么往下认：两份副本里有一份头里类标签是类型段说的那一类（类型段 0 是码 2 节点，类型段 2 是码 3 容器）、按那一类长度的整单元校验和与它的位置条目对得上，就照类型段那一支往下走那一份，引用登记的槽数按那一类；另一份按同一长度读回比校验和，对不上照红 I-2.1（校验和与内容匹配），I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用） 跟着；一份都没有的不往下走，每份副本照 I-2.1（校验和与内容匹配） 那一行的长度读法判。两份各自证成两种不同的别类时取哪一类还没有条款，记在 C739（认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款）（`research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1、4 条，`research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M1 那一行）。跨头共享时出生树的读法随 C110（跨头共享与加密后元数据类的块头一致性读法） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；GPU 判器走读内核 inode 树只接类型段 2 的层级 1 条目：类型段 0 而有一份按码 2 自证过的条目落判不了（`UNDECIDABLE_TOO_DEEP`），交回 CPU 判（删除线第 30 批，第 31 批补对拍流）） |
| I-9.3 | 内部节点消息数恒零 | inode 树内部节点的消息计数恒 0（ε = 0，D11（索引节点要不要留消息缓冲区） 已定项 6 的树轴）；计数字段住节点内部布局（D15（格式冻结政策） 第 4 层，未冻结），那一层定了这条才可判 | 未实现，前置：树的种类 + 节点内部布局 |
| I-9.4 | 容器号不超最小 key | 打包记录类型 2 的容器内每条记录的 inode 号 ≥ 容器号；沿叶序容器号严格递增，且左容器的最大 key < 右容器号（这条逼出合并只许左吸收右 / 左从右借）；同一片打包记录类型 2 容器里记录号沿记录序严格递增 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；同一片容器里记录号严格递增那一句 2026-10-05 删除线第 22 批实现） |
| I-9.5 | 删除队列里的号仍在树里 | 两个方向都要成立。① `deleted_inodes` 里的每个 inode 号在同一棵树的 inode 树里存在且记录的 nlink = 0（记录在回收完成前留在 inode 树里，`deleted_inodes` 只存号）；② inode 树里 nlink = 0 的每条记录，号都在同一棵树的 `deleted_inodes` 里（队列是派生态，队列 = inode 树里 nlink = 0 的号集合，D8（核心索引结构） 已定项 16 第 3 条）。**共用判法**（I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-3.4（可用空间扣待删占用） 都照这一段，不另写）：判走读到的任一版（候选集里每条根，含由记录施加出来的那几版），不只判最新根；`deleted_inodes` 与 inode 树经树表条目偏移 116 的头 ID 配对（头 ID 的取值见 I-9.27（树表条目的头 ID 取值）），配不上的版记判不了；树表、`deleted_inodes` 或 inode 树读不全的版报 `part_of_the_range_not_judged`（射程里判不了，不判红），这是今天除 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 外各条用的报法，是否改报第四种判定「部分没判」随 C701（别的判法静默跳过读不出的部分） 那一问由用户答，答了这几条随它一起改。树表 0 条的版没有这几棵树，不判；树表指针全零的版本照 core 的根记录读者跟过去判损坏（候选集里自证过的根照 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 那一行；由记录施加出来的那几版与根环里的根同一读法，用户 2026-10-06 弹窗定，选项原文「甲 两类都判损坏」，`records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行），这一条的删除态那一遍（`walk::judge_deletion_state_of_the_versions`）对这两种版本都报射程里判不了；这一遍不扫下一次挂载先施加的那一版；树表单元相同的几版内容相同，只扫一次；没有删除的版本上 I-9.5（删除队列里的号仍在树里） 报成立（两个方向比的是队列与整棵 inode 树，干净的一版也真比过），I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立） 没有对象时报不适用并带理由 | 已实现（2026-10-05，池级 checker `walk::check_pool_image`，删除态扫描是 `judge_deletion_state_of_the_versions`；坏镜像登记在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `.chain`，搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`，逐形的用例在 `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs`；GPU 判器下标 69，扫描内核判）。两个方向各有坏镜像，「队列里的号在树里 nlink ≠ 0」那一形也有（C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 已还清）；`deleted_inodes` 的形态已定在 D8（核心索引结构） 已定项 16；判法与读法的出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节 |
| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key（回退候选集里任一版的 inode 树都算），> 该树全部已发布的（按实例表判）墓碑记录的对象 ID（墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；被崩溃恢复抛弃的时间线上写的墓碑不在所选根的墓碑树里）——2026-09-16 随 D8（核心索引结构） 已定项 6 收窄：被抛弃时间线的墓碑不算——崩溃恢复落到较旧的根之后水位随所选根那棵账载入，被抛弃时间线里发过的号会被重发，算上那些墓碑 checker 按扫描读就判红（管理员回退是挂着时的一次向前发布，水位取 max、不重发，D23（journal 的角色与格式） 已定项 14）；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判）。**两半都判**：水位 > 树内最大 key；墓碑那一半（2026-10-05 起）水位 > 所选根（最新的那条根）墓碑树里已发布的（按所选根指着的实例表判）墓碑容器里全部记录的对象 ID 最大值，函数 `largest_published_tombstone_object_identifier`，一条已发布的墓碑都没有时没有对象，墓碑树读不全报射程里判不了；坏镜像：墓碑记录的对象 ID 抬到水位（`crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` 的 `raise_the_tombstone_object_identifier_to_the_inode_number_watermark`）；GPU 判器内核一并判，并进下标 41 那一位 |
| I-9.7 | 记录三段恒零 | 打包记录类型 2 的记录偏移 108 / 112 / 120 三段（填充 4 / flags 8 / 预留 20）恒 0；非 0 判该记录 EIO，容器其余记录照读（前向兼容政策，D15（格式冻结政策） 判定表；不是损坏检出，载荷校验和照样过） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.8 | 记录 inode 等于 key | 打包记录类型 2 的记录偏移 0 的 inode 号 == 它在 inode 树里的 key；不等判红、该记录 EIO，不许静默修复 | 未实现，前置：树的种类 |
| I-9.9 | locality 三值对照 | 记录的 `locality_id` == 该对象每条 extent key 的首段；extent 树未经重建时输出红 / 绿；系统配置 `map_provenance` 标 rebuilt 时两侧同源，输出「不可判定」，不许输出「通过」；按侧判（2026-09-16）：记录那一侧是从 key 编出来的、或 key 那一侧是从记录编出来的，同样输出「不可判定」；分批重新聚簇的中间态怎么判随 C346（后台重新聚簇没有定义） 定 | 未实现 |
| I-9.10 | 对象出生代三处一致 | 记录偏移 8 的对象出生代 == 该对象每个数据单元头五元组里的对象出生代 == 该对象的墓碑记录里的对象出生代（若有；该对象的墓碑记录按 inode 号认，走读到的每一版各比这一版自己的；墓碑记录住墓碑树，D18（块里携带什么信息） 已定项 24；只对 inode 记录还在的对象判，删完之后记录不在，这一个等号无对象）；重建后两侧同源，输出「不可判定」，与 I-9.9（locality 三值对照） 同形 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判；墓碑那一个等号 2026-10-05 删除线第 22 批实现，GPU 判器第 23 批补） |
| I-9.11 | 可写克隆头有水位行 | 每个可写克隆头的树 ID 在记账里都有「inode 号水位」行（树表可枚举、记账可查）；「初值 ≥ origin 那一刻的水位」是建克隆那一刻的操作断言，转模型对拍——它的右操作数（建克隆那一刻的水位行）只住写出它的那条根自己的记账树里（D5（快照 / 空间记账机制） 已定项 2：每条根的记账树只装这条根那一代的行），对着镜像读不到 | 未实现，前置：树的种类 |
| I-9.12 | 分隔 key 落在孩子区间之外 | inode 树内部节点的条目按分隔 key 严格递增；第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界。判的字段是内部节点条目头 8 字节那个分隔 key（D8（核心索引结构） 已定项 6：条目 120 字节 = 分隔 key 8 + 身份引用 26 + 子指针 86，「分隔 key = 该孩子创建时的最小 key，条目按分隔 key 单调递增」）；孩子的 key 区间对码 3 容器由它装的记录现算，容器头不带 key 区间 | 已实现（2026-09-22，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.13 | 类型 2 容器不空 | 任一打包记录类型 2 的容器记录数 ≥ 1（空叶在重建时没有 key 范围；唯一的叶被删空时不落盘、根变成零条目） | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
| I-9.14 | 树表条目的诞生 txg 跨根不变 | 同一棵树（按树 ID）的树表条目，在**同一条时间线上**的任意两条有效根各自的树表里，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段）；C374（释放代与树表诞生 txg 只有验收断言盯着））。「同一条时间线上的有效根」就是回退候选集（D23（journal 的角色与格式） 已定项 14 + D16（发布语义） 已定项 1）：最新那条根本身；以及根环里 (i, T) 满足两条的根——按**最新那条根指着的实例表**，没有实例 i 的行、或有行 (i, Ti, Wi) 且 T ≤ Ti；并且 T ≥ 最新根带的回退下界 F。有行 (i, Ti, Wi) 且 T > Ti 的根在被崩溃恢复切掉的旧线上，**不与现行线上的根比**（C511（回退到无文件那一版之后诞生代怎么接） 2026-09-23 用户定案收窄射程）。候选集不足两条根，或同一条时间线上没有一棵树的条目出现在两个不同的树表单元里时，报「不适用」，不报成立——几条根指着同一个树表单元、拿同一份字节比自己是恒真判定。判别力：① 把树表条目的诞生 txg 改成跟着这次发布的 txg 走，必须红；② 管理员回退前后同一条线上记得不同，必须红（回退是向前发布，前后是同一条线）。跨线（含被崩溃恢复抛弃的旧线）的诞生 txg 一致由 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 判，这一条不再有「只有被切掉的那条线记得不同，不许红」那一格（用户 2026-10-04 定）。推翻条件：找到一条历史，最新根的实例表把某条根判成有效，而它不是最新根的祖先（不在同一条线上）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；层 0 每个崩溃状态都判。2026-09-23 收窄之后同文件两条：判别力 ② 的坏镜像 `birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant`（挂着时回退到 A 之后，回退那次发布 D 的树表把 extent 树（树 65536）的诞生 txg 记成 D 的 txg 9、A 那一版记 3：A 与 D 在同一条线上，只有 I-9.14（树表条目的诞生 txg 跨根不变） 红），原判别力 ③ 的正向用例 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds`（被切掉的线由崩溃恢复造：C 被抛弃之后只改 C 那片树表里的诞生 txg）要改：这张镜像上 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 会红，不能再断言每一条都成立；欠账在里程碑三开工第一轮的第 9 批（`records/2026-09-30-里程碑三开工第一轮.md`，池级 checker 簇），由实现员改它，`judge_tree_table_birth_txg` 的注释里说明判别力 ③ 的那一段跟着改；判定在 `walk::judge_tree_table_birth_txg`） |
| I-9.15 | inode 记录的 blocks 等于 ⌈size ÷ 512⌉ | 打包记录类型 2 的每条记录，偏移 48 的 blocks == ⌈偏移 40 的 size ÷ 512⌉（D8（核心索引结构） 已定项 6：blocks 是逻辑长度的 512 字节块数，不表示分到的空间；C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）；不等判红。读者要不要也判、判了报什么没有条款 | 已实现（2026-09-24，池级 checker `walk::check_pool_image` 走 inode 叶容器时逐条判，与 I-9.7（记录三段恒零） 同一处；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：blocks 写成 64 / 5 / 7 各只红它；层 0 每个走得到 inode 叶的崩溃状态都判；变异：判定恒真、按向下取整比、写路径写回 64；里程碑「覆盖写、释放、回退与复用」收口表第 46 行） |
| I-9.16 | 树表条目按树 ID 严格升序且合发号次序 | 同一张树表（D8（核心索引结构） 已定项 8 的字段表）里，① 盘上相邻两条的树 ID 严格升序；② 树表里有的种类按发号次序（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops；中央映射树不进树表）排，相邻两棵的树 ID 严格升序；③ 每一种种类码（登记过的与没登记的都算）至多一条；④ 树表非空时进树表的十一种（② 那一串）各恰好一条，读者重建一版时十二棵树的号逐棵取，缺哪一种都重建不出这一版（报「树表里没有那棵树」）。四道任一道不成立即违例；0 条的树表四道都成立，1 条的前三道成立。依据 D8（核心索引结构） 已定项 8「排序契约」与它的 ②「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」。判别力落在 ②：① 与 checker 在树表上判的 I-1.1（块头自述逻辑地址） 那组 key 严格递增是同一个谓词，只违反 ① 的镜像两条一起红；③ ④ 各有只违反它的坏镜像（一种出现两次、缺一种） | 已实现（2026-09-27，core `crates/singlefs-core/src/recovery.rs` 常量 `TREE_TABLE_ENTRIES_ORDERING_CONTRACT`，① ② 合成一个入口，`rebuild_version` 与 `walk_to_file` 在读任何一棵树之前拒；③ 是 `tree_table_entries_each_kind_at_most_once`，④ 由 `rebuild_version` 逐种取号时报「树表里没有那棵树」、`walk_to_file` 报「树表里缺一棵有根的树」；池级 checker `crates/singlefs-checker/src/walk.rs` 另写一份：`judge_tree_table_entries_ordering` 判 ① ②，`judge_tree_table_entry_kind_counts` 判 ③ ④（④ 的十一种是 `TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`），违例写明哪一版根与哪两条或哪一种，列进 `image.rs` 的 `IMPLEMENTED_INVARIANTS`；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_tree_table_entries_ordering` 同判四道，② ④ 的循环十一种；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`「互换两条的种类」那份只红 I-9.16（树表条目按树 ID 严格升序且合发号次序）；只违反 ① 的钉在 `tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`；③ ④ 的坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，用例名以 `a_tree_table_whose_` 与 `a_tree_table_without_its_` 打头（livelist 或 logged_ops 那一条改成第二条 extent、deadlist 那一条改成第二条分配记录、分配记录条目种类改成没登记的码、整条去掉分配记录 / deadlist / logged_ops 那一条），各只红 I-9.16（树表条目按树 ID 严格升序且合发号次序），去掉 logged_ops 那一条的一份走读也红；实现员报告 `research/prompts/m2-rev-tree-table-ordering-implementer-report.md`） |
| I-9.17 | 谱系树条目身份与子头相符 | 谱系树（D18（块里携带什么信息） 已定项 24）的每条条目：类型段 ∈ {0, 5}；类型段 0 ⇒ 子单元是码 2（谱系树根满长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；类型段 5 ⇒ 子单元偏移 6 = 3，子头的 (出生树, 打包记录类型, 容器号, 容器出生代) 与条目的身份引用四段逐字相等，子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 5；其余判该条目损坏。孩子头里的类与类型段说的那一类不一致时（条目照判损坏之后）怎么往下认：两份副本里有一份头里类标签是类型段说的那一类（类型段 0 是码 2 节点，类型段 5 是码 3 容器）、按那一类长度的整单元校验和与它的位置条目对得上，就照类型段那一支往下走那一份，引用登记的槽数按那一类；类型段那一支自己也不往下走的（层级 ≤ 1 节点里类型段 0 的），两份都照 I-2.1（校验和与内容匹配） 那一行不往下走的读法判（`research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K1 N1 那一行），「另一份按同一长度」那一句不管它；另一份按同一长度读回比校验和，对不上照红 I-2.1（校验和与内容匹配），I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用） 跟着；一份都没有的不往下走，每份副本照 I-2.1（校验和与内容匹配） 那一行的长度读法判。两份各自证成两种不同的别类时取哪一类还没有条款，记在 C739（认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款）（`research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1、4 条，`research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M1 那一行）。类型段 0 那一支认「孩子是码 2」时只认按自己头里那一类的长度整份自证过的副本；两份都没自证过时当读不出，「孩子是码 2」这一小句不判（用户 2026-10-07 定，原话在 `records/2026-10-01-删除与并发事务.md` 2026-10-07 第 32 批 P 交回那一条；`research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 3 条）。身份不符单独判定，不再记走读失败；与 I-1.6（类标签登记且三处相等）、I-1.7（打包容器合法与判定顺序） 重合的小句仍留作合取。与 I-9.2（条目身份与子头相符） 同形 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.18 | 删除队列里的号有谱系上的墓碑 | `deleted_inodes`（D8（核心索引结构） 已定项 16）里的每个号 N，墓碑树（D18（块里携带什么信息） 已定项 24）里都有一条在谱系上的墓碑记录 (N, N 的 inode 记录偏移 8 的对象出生代, [0, 2⁶⁴ − 1])。「在谱系上」：墓碑记录的死亡 txg 落在这一版的父链上——每一版从这一版自己的 txg 起、按这一版树表里的谱系树往回走，有谱系记录 (T_rb, T_old) 的那个 txg（T_rb）的父是 T_old，其余 txg 的父是 txg − 1（谱系记录住谱系树，D18（块里携带什么信息） 已定项 24；txg − 1 那一步核心层没有显式断言，C616（谱系父链靠 txg 连号没有断言））。不按谱系过滤的写法放过改坏（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格）；墓碑记录不按已发布与否滤 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补；对「队列里的号没有记录」那一形报判不了而不是红，I-9.5（删除队列里的号仍在树里） 的这一形坏镜像才只红自己，出处 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节最后一行） |
| I-9.19 | 谱系上的墓碑不指活对象 | 墓碑树里每条在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）指认的 (号, 对象出生代)，inode 树里有号与对象出生代都相同的记录时，那条记录的 nlink = 0 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.20 | 谱系上的墓碑不重号且记录合法 | 墓碑树里在谱系上的墓碑记录（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）按 (号, 对象出生代) 不重复，这一句只判在谱系上的；墓碑树里全部墓碑记录（不论在不在谱系上）每条区间起 < 区间止，第一版只许整对象 [0, 2⁶⁴ − 1]，每条记录的对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；每个打包记录类型 1 容器的记录数 ≤ 815（⌊(32768 − 136) ÷ 40⌋），这一句仍与前几句合取。不按谱系过滤的写法在「删 → 回退 → 再删」的合法历史上判红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判新不变量写法那一格） | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补；「对象出生代 ≤ 死亡 txg ≤ 这一版的 txg」这一句被攻过一轮（`research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N5 那一行，没打中，一次抽样）） |
| I-9.21 | 产品镜像里非最右墓碑容器装满 | 没有用只供测试的容量旋钮的镜像上，墓碑树里容器号沿叶序严格递增（根长高之后根的条目是类型段 0、容器号恒 0，见 I-9.23（墓碑树条目身份与子头相符），所以不沿根的条目读），除最右那一个之外的每个打包记录类型 1 容器记录数 == 815（开放容器满了才关，D18（块里携带什么信息） 已定项 24）。旋钮值由池级 checker 的新入口从镜像外给，旧入口当没装旋钮；判回退候选集里的每一版，墓碑树零根或只一片容器（没有非最右的容器）报默认不适用 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.22 | 测试镜像里墓碑容器不超旋钮值 | 只供测试的容量旋钮压小了一个墓碑容器装的记录数的镜像上，墓碑树里容器号沿叶序严格递增，每个打包记录类型 1 容器记录数 ≤ 旋钮值；旋钮不落盘，checker 从镜像外拿旋钮值：旋钮值由池级 checker 的新入口给，旧入口当没装旋钮；判回退候选集里的每一版，墓碑树零根报默认不适用（只一片容器照判 ≤ 旋钮值）。按产品容量判这类镜像会在合法状态上红（`research/prompts/m3-delete-r3-main-verification.md` 第三节判容量旋钮那一格） | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补，GPU 恒不置位、层 0 恒不适用：层 0 与放量只走产品入口，拿不到旋钮值，照 I-4.4 先例） |
| I-9.23 | 墓碑树条目身份与子头相符 | 墓碑树（D18（块里携带什么信息） 已定项 24）每个内部条目的类型段 ∈ {0, 1}：0 ⇒ 子单元是码 2（根长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；1 ⇒ 子单元是码 3，子头 (出生树, 打包记录类型, 容器号, 容器出生代) 与身份引用四段逐字相等、子指针头部出生树 == 身份引用出生树、子容器打包记录类型 == 1。孩子头里的类与类型段说的那一类不一致时（条目照判损坏之后）怎么往下认：两份副本里有一份头里类标签是类型段说的那一类（类型段 0 是码 2 节点，类型段 1 是码 3 容器）、按那一类长度的整单元校验和与它的位置条目对得上，就照类型段那一支往下走那一份，引用登记的槽数按那一类；类型段那一支自己也不往下走的（层级 ≤ 1 节点里类型段 0 的），两份都照 I-2.1（校验和与内容匹配） 那一行不往下走的读法判（`research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K1 N1 那一行），「另一份按同一长度」那一句不管它；另一份按同一长度读回比校验和，对不上照红 I-2.1（校验和与内容匹配），I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用） 跟着；一份都没有的不往下走，每份副本照 I-2.1（校验和与内容匹配） 那一行的长度读法判。两份各自证成两种不同的别类时取哪一类还没有条款，记在 C739（认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款）（`research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1、4 条，`research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M1 那一行）。类型段 0 那一支认「孩子是码 2」时只认按自己头里那一类的长度整份自证过的副本；两份都没自证过时当读不出，「孩子是码 2」这一小句不判（用户 2026-10-07 定，原话在 `records/2026-10-01-删除与并发事务.md` 2026-10-07 第 32 批 P 交回那一条；`research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 3 条）。身份不符单独判定，不再记走读失败；与 I-1.6（类标签登记且三处相等）、I-1.7（打包容器合法与判定顺序） 重合的小句仍留作合取。与 I-9.2（条目身份与子头相符） 同形（墓碑正推报告 `research/prompts/m3-tomb-r1-sonnet-output.md` 第 241 行 N1；根长高之后类型段 0 那一支是主 agent 按 K-R3-3 补的，被攻过零轮） | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.24 | 删除队列至多一条 key | 任何一版里 `deleted_inodes`（D8（核心索引结构） 已定项 16）至多一条 key；每个被判的版本各判一次，不遍历版本之间的历史；每条 `deleted_inodes` 条目都判，不管树表条目的头 ID 配不配得上，干净的一版报成立。依据：会话里每一次用户改动先把现行那一版的队列走空再做自己（D8（核心索引结构） 已定项 16 第 6 条的增补，用户定案），判决 `research/prompts/m3-blanks-r1-main-verification.md` 第二节「B4 + B8 队列上界与接口」一行 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补） |
| I-9.25 | 树表条目的 previous_snapshot_txg 取值 | 走读到的任一根（候选集里每条，含由记录施加出来的那几版）的树表里：树的种类不是 inode 树（种类码 ≠ 2）的条目，偏移 100 的 `previous_snapshot_txg` 恒 0；inode 树的条目为 0，或 1 ≤ p ≤ 这条根的 `checkpoint_txg`。依据 D5（快照 / 空间记账机制） 已定项 9（`previous_snapshot_txg` 只写在该头 inode 树那条树表条目里，其余条目写 0 且 0 = 不适用）；inode 树条目「≤ 根 txg」那一半的条款是用户补定的（`research/prompts/m3-inv-b-r1-main-verification.md` 第四节用户定案第 2 条）。 | 已实现（2026-10-02，池级 checker `walk_tree_table_entry` 每走到一条树表条目判一次，每个走到的版本（含记录施加出来的）都走它；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，inode 树条目的 previous_snapshot_txg 压过根 txg 那一份由 2026-10-03 补；⚠️ 树表单元被几版共享时只在第一个走到它的那一版判（`visited_units`），共享它的别的候选根漏判，与条款字面「候选集里每条」不符，代码注释自认判得更松，代码改回 kb 字面，实现进下一批（主 agent 2026-10-03 定；`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 2 行，推的，没造镜像）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行） |
| I-9.26 | inode 改动计数落在出生代与容器诞生代号之间 | 走读到的任一 inode 叶容器（码 3，打包记录类型 2）里每条记录：对象出生代（记录偏移 8）≤ 改动计数（记录偏移 88）≤ 这片容器头的诞生代号（偏移 73）。上界不是等于：容器因别的记录改动而重写时，没改的那条记录的改动计数小于容器诞生代号是合法的；下界要「创建算一次改动」，这一句条款是用户补定的。依据 D8（核心索引结构） 已定项 6 偏移 88 那一行（改动计数 = 最后一次改动所在发布的 `checkpoint_txg`）与 D18（块里携带什么信息） 已定项 9 之后的容器重写规则。 | 已实现（2026-10-02，池级 checker `walk::walk_inode_leaf_container`：每条记录判偏移 8、偏移 88 与容器头里偏移 73 的诞生代号（不是偏移 61 的容器出生代）；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，下界一份（改动计数 = 对象出生代 − 1）由 2026-10-03 补；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 5 行与第四节用户定案第 2 条） |
| I-9.27 | 树表条目的头 ID 取值 | 走读到的任一版（候选集里每条根，含由记录施加出来的那几版）的树表有条目时，逐条判条目偏移 116 的头 ID：种类码 2（inode 树）的条目等于它自己的树 ID（条目偏移 0）；种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）的条目等于这张树表里种类 2 条目的树 ID，表里有多于一条种类 2 条目时放宽成「等于其中某一条的树 ID」（收严留给多头那一批），表里没有种类 2 条目时这三种判红；种类 3、4、6、7、8、11、12 的条目等于 0；别的种类码不判。树表 0 条的版本一条也不判；条目宽不到 124 字节（头 ID 那 8 字节读不全）的树表整张不判。不拿字面 65537 比：它只是树 ID 从 mkfs 水位 65536 起发的那一版上的取值。依据 D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」与 D5（快照 / 空间记账机制） 已定项 9。 | 已实现（2026-10-04，池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_tree_table_entry_head_identifiers`，挨着 I-9.16（树表条目按树 ID 严格升序且合发号次序） 每张树表判一次、按条目计数，共享这张树表的别的版本不重判；条目宽不到 124 字节时这张树表报射程里判不了；GPU 判器走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_judge_tree_table_entry_head_identifiers`，`IMPLEMENTED_INVARIANTS` 下标 66；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（extent 树条目头 ID 写 0，只红它），从根环水位发树的合法历史在 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`） |
| I-9.28 | 全部可读根里同一树 ID 的诞生 txg 相同 | 根环里全部读得出且自证合法的根（不按实例表判有效与否，含被崩溃恢复抛弃的旧线上的根），任两条根各自的树表里同一树 ID（条目偏移 0）的条目，诞生 txg 必须相同；只有这棵树第一次出现在树表里时才设这个值（D8（核心索引结构） 已定项 8（树表条目的字段））。判定四种：有一处诞生 txg 对不上 → 不成立；各处都对得上，但有一条读得出、自证过的根树表读不出（含树表指针全零）、或条目比字段表短 → 部分没判，列出这些根（盘号、实例、txg、为什么没判）；没有一棵树出现在两个不同的树表单元里（几条根共用一个树表单元时拿同一份字节比自己，是恒真判定）→ 不适用；其余 → 成立。树表指针全零的根不是「没有树表」：core 的根记录读者不把全零当特例，照它那两条位置条目读盘 0 槽 0、读不出（可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })`），池级 checker 对根环里自证过的这样的根照 core 跟过去、判这条根损坏（红 I-2.1（校验和与内容匹配），最新根再红 I-4.8（近 K 代根校验和自洽）、I-7.2（最新根可完整遍历）、I-7.4（近 K 代块未被复用），别的候选根再红 I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用）），这一条把它与树表读不出同一处置、报部分没判并点名「树表指针全零」（用户 2026-10-05 弹窗定，选项原文「甲 checker 判它损坏」，`records/2026-10-01-里程碑三形式化接入.md` 第 33 批「第 32–33 批代码轮第一轮判决」那一条，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条）。只判根环里全部读得出、自证过的根（含被抛弃的、F 之下的），不含由记录造出来的版本：由记录施加出来的那几版与下一次挂载先施加的那一版不进这一条的比较，也不为它们报部分没判。这两类版本里树表指针全零，与根环里的根同一读法、照 core 的根记录读者跟过去判损坏（用户 2026-10-06 弹窗定 b34 四问，选项原文「甲 两类都判损坏」「甲 同改」，另一问选「乙 照今天」，出处 `records/2026-10-01-里程碑三形式化接入.md`「用户 2026-10-06 弹窗定 b34 四问」那一行）：由记录施加出来的那几版红 I-2.1（校验和与内容匹配）、I-3.1（已分配统计对得上）、I-4.8（近 K 代根校验和自洽）、I-7.4（近 K 代块未被复用）（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P2）；下一次挂载先施加的那一版全零，在重放只施加这一次发布时就是重放之后那一版，红 I-4.1（重放后树可遍历），core 的两种挂载都拒。不判 I-2.5（位置条目按设备身份升序）：那一行把整条全零的指针整条豁免，不收窄。GPU 判器的走读内核对全零的树表指针与 CPU 同判：两条位置条目照 I-2.1（校验和与内容匹配） 判、记一次走读失败，映射根不走，不判 I-2.5（位置条目按设备身份升序）（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的 `w_walk_tree_table_and_mapping`；GPU 一侧是读码推的，没在卡上跑，判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P3）。与 I-9.14（树表条目的诞生 txg 跨根不变） 的分工：I-9.14（树表条目的诞生 txg 跨根不变） 只比同一条时间线上的根，这一条不分时间线、比全部可读根。依据：树 ID 被重发（C342（树 ID 水位在根读不出时退回去重发））时，新线上的树与老线上同一个 ID 的树诞生 txg 不同，I-9.14（树表条目的诞生 txg 跨根不变） 的同线收窄看不见这一形。推翻条件：任一合法历史（池级 checker 与层 0 其余判定都不红）上红。 | 已实现（2026-10-04，欠账来源 C342（树 ID 水位在根读不出时退回去重发）；池级 checker `walk::check_pool_image` 的 `judge_tree_identifier_birth_txg_across_every_readable_root`，四种结局；GPU 判器的扫描内核挨着 I-9.14（树表条目的诞生 txg 跨根不变） 判、只置违例位；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，合法历史、部分没判、不适用、对不上照报违例的用例在 `crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs`，树表指针全零的根报部分没判并点名的用例在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged`；`crates/mutations.tsv` 里以「第9批」「第9批改钉」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）或第四种判定的各行，与以「第 9 批 9-9 GPU 判器」打头、点名 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）的 3 行）。用户 2026-10-04 定新立，同时删 I-9.14（树表条目的诞生 txg 跨根不变） 判别力第 ③ 条与它的正向用例，判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第六节 S2-1 行。被攻过零轮：攻方三段合法历史上 0 红（一次抽样），被切掉旧线的镜像上红。部分没判：用户 2026-10-04 定「树表读不出算部分没判」、加第四种判定；四种结局的细则主 agent 定，零轮，`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批「用户 2026-10-04 弹窗定（9-6 交回的两问）」那一条 |
| I-9.29 | 同号只许一个出生代 | 回退候选集里各版合起来（含由记录施加出来的那几版）同一个 inode 号只许对应一个对象出生代：每一版的 inode 记录的出生代与那一版谱系上墓碑的出生代都算；最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了（「在谱系上」照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义）。依据 D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」；判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T10、T15 两格与第四节用户定案（T15 定「写」）。 | 已实现（2026-10-05，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`，删除线第 22 批；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；GPU 判器第 23 批补；判不了的门第 25 批改成看主走读有没有沿最新根的实例表链走到末片，与候选集滤法同一遍走读（`Walk::abandoned_roots_filtering`，判决 `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条），GPU 链走满上限还没完记判不了第 27 批补） |

**不进 I-9（inode 树结构） 的**：分裂 / 合并的身份传递（左半身份不变、右半号 = 触发分裂的新记录号、出生树 = 执行分裂的树）与合并时机是操作断言，对着一个镜像判不了，走单测 / 模型对拍与崩溃点重放（C116（叶容器的分裂 / 合并纪律没有会失败的检查））；
nlink == dirent 条数的门（dirent 树存在）今天没有输入，写不成可判定形式，随 dirent 树上线时一起立。

```
