# C329（写行那次发布之前推抬 F 的空发布没有检查） 与 C330（中间实例那一行的 T_pub 取所选根的 txg） 第一轮 · Opus 反推腿报告（2026-09-14）

立场：攻 C329 甲（写行那次发布是本实例的第一次发布、走切换预留、抬 F 排在它之后）与 C330 甲（中间实例写 (i, 0, 0)）。主攻 U1、U2，辅攻 U5；乙、丙当对照，在甲被打中的同一个构造上量。
模型在 `research/prompts/c329-c330-r1-opus-model/`：`src/main.rs` 是第一遍（剧本历史 + 四次挂载的枚举）；`attribution/` 是第二遍，它的 `build.rs` 把第一遍原样搬进来（只去掉 `//!` 行、把入口改名），加了归因开关与择根倒挂两个入口。只用 std，没有 I/O、随机源与并发，确定性。
这份报告里的数全部来自临时目录里的副本跑（复跑命令在「十一、复跑」），按提示是线索，主 agent 在入库装置上重做。

## 〇、结论表

| 判据 | C329 甲 | C330 甲 | 同一构造上的乙、丙 |
|---|---|---|---|
| U1 | 前提五与三种换故障（管理员回退、实例切换的两种读法、之后连着两次普通挂载）都关得住，四次挂载每次判都是 U1 = 0。打中一处条款没写死的：「非干净结束」全仓没有判别子，按最弱读法（看着像干净就不写行），前提五的孤儿原样翻面（U1 = 3），甲、乙同中，丙变成 U2 | 前提六的三种形态（M2 第一个根落了而超级块没落 / 两样都落了 / M2 是管理员回退）都关得住。枚举 610000 条四次挂载历史：回退那次发布的实例表以 R_old 引用的那一版为底时 U1 = 0、U2 = 0（摘掉不分辨臂的那一类之后）；以挂载时最新可读的根引用的那一版为底（条款没写死，最弱读法）时 14853 条历史有 U1，最少 5 个故障（两条根暂时读不出 + 三次崩，其中两次挂载是管理员回退），同一格乙的记录扫描水位不中 | C329 乙：前提五、回退关得住；实例切换那一格按「号 ≤ W 的单元只在内存里、不在重建态里」读，抬 F 的空发布带着 W 却没带那两个单元，U1 = 2。C329 丙：前提五、回退、切换都 U2（非干净结束的旧实例没有行 ⇒ 它已发布、被挂载根引用的单元判未发布）。C330 乙两种字面落点：超级块水位在「根落了、超级块没落」那一格中（U1 = 3），根记录带水位与今天同中；水位取「超级块 ∪ 读得出的根 ∪ 全环自证过的记录」时剧本与枚举都 0。C330 丙：判别子观测不到——前提六里所选根引用的单元与孤儿的谓词输入逐项相同、真值相反 |
| U2 | 没写出针对甲的 | 没写出「中间实例的单元被所选根引用」。实例代号按时间单调（取号 = max + 1、过半独占打开），中间实例的单元唯一被收养的路径是切换的「号 ≤ W 的事务照旧」，而按 D23（journal 的角色与格式） 第 1236 行，被照旧的都是被重发那个 checkpoint 的事务、属于所选根那个实例。连续两次切换时切换那句的 W 按字面记到中间实例名下：今天的条文 U1 = 1 + U2 = 2；甲消掉前者、后者照旧（它在所选根那个实例的行上，甲不碰） | 不分辨臂的一格（新）：重放施加进来的码 2 / 3 单元按 D23 已定项 15「换四个字段、不需要树语义」被原样收养，谓词对码 2 / 3 只看「b ≤ 所选根（重放之前）的 txg」⇒ 被挂载根引用的节点判未发布。剧本 3 个节点全中；枚举里今天、甲、三种乙都是 246267 条历史，丙 610000 条全中 |
| U3（顺带） | 写行那次走切换预留：D28（挂载期承诺量） 已定项 3 按 N_switch 次切换预留，恢复自己先用掉一份（第一个事务几何每盘 6 块、连暖机 14 块，c_max = 4），第 3 次切换拿不到预留 ⇒ 转只读。若恢复真要重写重放施加进来的码 2 / 3 单元（D18（块里携带什么信息） 第 879 行字面），写行那次的量随被重放那几次发布的节点数走，预留罩不住，而甲不许先推抬 F | — | 乙同样要让第一个根带着那次重写，量一样；差别只在甲把来源写死成切换预留 |
| U5 | 三处各要补一句：D16（发布语义） 第 377 行准入行没有例外、D28 第 82 行「预留 = N_switch × 这个最坏量」、D28 第 86 行「号 > W 的用户数据分配失败……返回 ENOSPC」。与暖机（D16 第 207 行）不冲突 | 与回退写中间实例 (i, 0, 0) 同形、与 I-3.8（实例表行唯一且低于挂载根） 不冲突；与 D23 第 691 行切换那句在连续切换时说法不同，甲的那一边是对的 | 丙若把「干净结束也写行」写在自己最后一次发布里，挂载根 (1,4) 的表里就有实例 1 的行，I-3.8 按字面红（`I-3.8_rows_not_below_mounted=[1]`） |

另有两件与候选无关、这一轮撞出来的：择根倒挂（读不出的是另一个实例的全部根时，新实例已确认的写被较旧实例的根压过去，6 个故障，甲下谓词自洽而那次写丢了，乙的超级块或记录扫描水位下不丢）；以及回退那次发布的实例表以哪一版为底全仓没写。都在正文。

## 一、现查清单

| 查了什么 | 命令 | 结果 |
|---|---|---|
| 背景材料附录与 kb 逐字节相同 | 对下列 14 行各跑 `line="$(sed -n "${n}p" "$f")"; grep -cF -- "$line" research/prompts/_c329-c330-r1-background.md` | D18（块里携带什么信息） 第 879 行、D16（发布语义） 第 207 / 248 / 377 行、D23（journal 的角色与格式） 第 691 / 1206 行、D28（挂载期承诺量） 第 82 / 86 / 88 行、`invariants.md` 第 22 / 127 行、`layout/01-first-txn.md` 第 393 行、`checks-owed.md` 第 305 / 306 行，各命中 1 次。欠账表那两行背景材料标的是 306-307，文件现在在 305-306，内容逐字相同 |
| 写行规则、已发布谓词、恢复择根 | `grep -n "写行\|已发布谓词" .claude/kb/decisions/18-块里携带什么信息.md` | 三样都在第 879 行（3440 字） |
| D16 已定项 1 / 6 / 8 | `grep -n "^###" .claude/kb/decisions/16-发布语义.md`；`awk 'NR==377'` | 已定项 1 标题第 358 行、准入行第 377 行；已定项 6 标题第 246 行、定案句第 248 行；已定项 8 标题第 205 行、定案句第 207 行 |
| D23 已定项 14、15 与切换时的 W | `grep -n "切换时的 W 取被重发的那个 checkpoint\|施加一条记录 = 把所选根的这四个字段换成记录里的" .claude/kb/decisions/23-journal的角色与格式.md` | 索引第 691 行（「实例切换 = 挂载内做一次恢复」那句也在这一行）、第 694 行（已定项 15）、正文第 1206 行（回退）、第 1236 行（注 4：切换时的 W） |
| D28 已定项 3 | `grep -n "^\*\*式子\*\*\|^\*\*暖机那一半\|^\*\*为什么固定点" .claude/kb/decisions/28-挂载期承诺量.md` | 第 82 / 86 / 88 行 |
| 择根的破平局 | `grep -n "平局\|择新" .claude/kb/decisions/22-单元原子性怎么合成.md` | 第 487 行「择新在 checkpoint_txg 平局时按它高者赢」 |
| 崩溃后 txg 重发 | `grep -n "崩溃后该号会被重发" .claude/kb/decisions/05-快照-空间记账机制.md` | 第 21 行 |
| 「干净结束」有没有定义 | `grep -rn "干净结束" .claude/kb --include=*.md \| grep -v decisions-history` | 3 处：D18 第 879 行「任何非干净结束的实例在下次恢复写行」、`layout/01-first-txn.md` 第 393 行、`checks-owed.md` 第 305 行（C329 那一行）。挂载时怎么判上一个实例是不是干净结束，一处都没写；`grep -rno "干净[^，。；）]\{0,12\}\|卸载[^，。；）]\{0,12\}" .claude/kb --include=*.md` 的全部命中里没有 D22（单元原子性怎么合成）（超级块字段表所在），即没有「干净卸载」一类的位 |
| 回退那次发布的实例表以哪一版为底 | `grep -rn "回退那次发布的实例表\|回退的实例表\|实例表以.*为底\|从 R_old 那棵账" .claude/kb --include=*.md` | 只有 D23 第 1206 行「defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入」与讲记账水位的 C143（inode 号水位在回退后会退回去重发）、C281（回退后第一次发布会复用被抛弃时间线还引用的块）、C314（回退可以复用被抛弃的根引用的单元）；实例表从哪一版起没有一处写 |
| 「恢复实例重写」的来处 | `grep -rn "恢复实例重写" .claude/kb --include=*.md` | D18 第 879 行与 E104（扫描重建的现行版本判定） 正文第 99 行「一个没发布的 checkpoint 的容器版本一律未发布，恢复实例重写它们（`rewrite_inflight_containers`）」；C284（施加一条记录在指针层上做什么没有定义） 那一行（`checks-owed.md` 第 269 行）的空间侧注已写「全登记则记录点名的码 2 祖先被登记成已分配、却按已发布谓词要重写」 |
| 树 ID 水位怎么取 | `sed -n 476p .claude/kb/decisions/08-核心索引结构.md` | 「新水位 = max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)」——与乙的「根记录带水位」同一个装置（「五、C330：U1」末段） |

## 二、模型与口径

- 一级是挂载历史：根、单元、journal 记录、实例表行、超级块里的实例代号与 txg 水位。每次发布写一个记账单元（码 2，盖掉父根的那一个；D16 已定项 9「空发布也是发布」），另加场景指定的「一直被引用的码 2 节点」与码 1。
- 择根：读得出的根里 (txg, 实例代号) 最大者（D22 第 487 行）。重放：计数器严格连续、下一条的实例代号与所选根不同即停（D23 第 1206 行那一节的注 1），施加 = 换成记录重建的根（D23 第 694 行），W = 被施加的记录里最大的码 1 事务号。
- 取号：max(超级块, 读得出的根的实例代号) + 1，取号两写都成（C322（取号那一步的屏障怎么放没有条款） 已定案，不在这一轮）。
- 写行：D18 第 879 行，给 [max(所选根的实例, 1), 新实例) 写，回退行不许被覆盖；回退写 (r_old, T_old, 0, 回退位) 与中间实例 (i, 0, 0)。
- 新实例第一次发布的 txg：今天 = 重放之后的根 + 1；回退 = 读得出的根里最大 + 1（D23 第 1206 行「根环里全部根记录 txg 的最大值 + 1」，读不出的根取不到）。
- 发布顺序：单元 → 记录 → 根 → 超级块槽（D16 已定项 7），场景定做到哪一步：只落单元 / 单元 + 记录 / 根落了而超级块没落 / 写成。
- 真值：挂载根的时间线 = 它一路继承下来的全部发布 (实例, txg)，重放重建的发布也算；引用 = 挂载根引用的现行版本（记账单元每次发布被盖掉，节点与码 1 一直被引用，切换照旧的码 1 加进重发那次的引用）。U1 违例 = 谓词判已发布 ∧ 诞生 checkpoint 不在时间线上 ∧ 没被引用；U2 违例 = 谓词判未发布 ∧ 被挂载根引用。I-3.8 按「行的实例代号 < 挂载根实例」查。
- 臂。C329 那一维四条：`today_literal`（准入不够时先推一次不带新行的抬 F 空发布，写行排第二次）、`c329_jia`、`c329_yi`（每个根都带同一批行）、`c329_bing`（今天的次序 + 无行判未发布）。C330 那一维六条：`today`、`c330_jia`、乙的三种落点——`c330_yi_superblock`（超级块水位，只在「写成」那一步更新，D16 第 513 行超级块槽在根槽之后）、`c330_yi_root_carried`（根记录带水位）、`c330_yi_record_scan`（超级块 ∪ 读得出的根 ∪ 全环记录的 txg 取最大）——与 `c330_bing_start`（单元带「本实例起点根的 txg」，起点 < T_pub 时 b 那一支不算）。C330 那一维的写行次序一律取 C329 甲。
- 故障计数：每次挂载一次崩（剧本里每次挂载都以崩收尾）+ 本次挂载暂时读不出的根数。
- 它答不了的：记录读不出、超级块槽读不出、取号失败、屏障报错、根槽写失败重发都没建；码 3 的多版与择新没建；`run_mount` 一次挂载至多两次发布（切换与多次发布另写剧本）；F 恒 0，候选集不按 F 滤；枚举里回退目标固定取候选集里第二新的那条（剧本里点名）。

更正：「二、模型与口径」里写的「D16 第 513 行超级块槽在根槽之后」行号错了，应为 D16 第 290 行（`grep -n "根槽持久之后再更新超级块槽" .claude/kb/decisions/16-发布语义.md`）；513 是背景材料文件里的行号。

## 三、C329：U1（前提五与换故障）

### 前提五复现：普通恢复，四次挂载

持久写集合（`today_literal`）：M1（实例 1）的 (1,1)(1,2)(1,3) 单元、记录、根、超级块槽全落，在飞的 (1,4) 只落了 3 个单元（记账、1 个节点、码 1 事务 2）就崩；M2 取号 2 的两份超级块落了，准入不够先推抬 F 的空发布 txg 4——单元（记账 + 1 个码 1）、记录、根 (2,4)、超级块槽全落，表里没有新行——写行那次 txg 5 只落了单元就崩；M3、M4 普通恢复，两次发布都写成，崩。每次挂载的所选根与写的行在整行里（`selected=`、`rows_written=`、`landed=`、`mounted=`）：

`first-pass.out` 第 3 行：
```text
  M1 acquired=1 landed=(1,1)(1,2)(1,3) last=(1,3) in_flight=(1,4) units_only crash
```
`first-pass.out` 第 4 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0) first_txg=4 admission_short=true reach=Complete+Some(UnitsOnly) landed=[(2,4)] | mounted=(2,4) table= U1=3 [i1t4.accounting(i1,b4,n0,Metadata,start0),i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.data2(i1,b4,n2,Data,start0)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 5 行：
```text
  M3 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=3 rows_written=(2,4,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(3,5)(3,6)] | mounted=(3,6) table=(2,4,0) U1=3 [i1t4.accounting(i1,b4,n0,Metadata,start0),i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.data2(i1,b4,n2,Data,start0)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

同一段历史换三条臂，M2 那一行：

`first-pass.out` 第 32 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0) first_txg=4 admission_short=true reach=Complete+Some(UnitsOnly) landed=[(2,4)] | mounted=(2,4) table=(1,3,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 60 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0) first_txg=4 admission_short=true reach=Complete+Some(UnitsOnly) landed=[(2,4)] | mounted=(2,4) table=(1,3,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 88 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0) first_txg=4 admission_short=true reach=Complete+Some(UnitsOnly) landed=[(2,4)] | mounted=(2,4) table= U1=0 [] U2=2 [i1t3.node0(i1,b3,n0,Metadata,start0),i1t3.data1(i1,b3,n1,Data,start0)] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 今天的字面：M2 的挂载根 (2,4) 表是空的，实例 1 没有行，(1,4) 的 3 个孤儿按「i < i_now 且无行 ⇒ 已发布」翻面（U1 = 3）；M3 选 (2,4)、只给 [2, 3) 写行，M4 之后一直是 3。前提五按条文走得出来，失败条款不触发。
- 甲：M2 的第一个根就带 (1,3,0)，孤儿 4 ≤ 3 为假 ⇒ 未发布；三次挂载都是 0。乙同。
- 丙：M2 的根不带行、实例 1 没有行，丙判「无行 ⇒ 未发布」⇒ 实例 1 已发布、被 (2,4) 引用的节点与码 1（诞生 3）判未发布，U2 = 2，M4 之后仍是 2。丙只动谓词、不动次序，它把前提五的 U1 换成了 U2。

### 换故障一：管理员回退代替普通挂载

持久写集合：M1 写成 (1,1)..(1,5)；M2 回退到 (1,3)、准入不够，今天的字面先推的抬 F 空发布 (2,6) 单元、记录、根、超级块全落（表里没有回退行），第二次只落了单元就崩；M3、M4 普通恢复。

`first-pass.out` 第 9 行：
```text
  M2 mode=rollback unreadable=[] selected=(1,5) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0,rollback) first_txg=6 admission_short=true reach=Complete+Some(UnitsOnly) landed=[(2,6)] | mounted=(2,6) table= U1=6 [i1t4.accounting(i1,b4,n0,Metadata,start0),i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.data2(i1,b4,n2,Data,start0),i1t5.accounting(i1,b5,n0,Metadata,start0),i1t5.node0(i1,b5,n0,Metadata,start0),i1t5.data3(i1,b5,n3,Data,start0)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 10 行：
```text
  M3 mode=recovery unreadable=[] selected=(2,6) after_replay=(2,6) W=0 acquired=3 rows_written=(2,6,0) first_txg=7 admission_short=false reach=Complete+Some(Complete) landed=[(3,7)(3,8)] | mounted=(3,8) table=(2,6,0) U1=6 [i1t4.accounting(i1,b4,n0,Metadata,start0),i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.data2(i1,b4,n2,Data,start0),i1t5.accounting(i1,b5,n0,Metadata,start0),i1t5.node0(i1,b5,n0,Metadata,start0),i1t5.data3(i1,b5,n3,Data,start0)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 37 行：
```text
  M2 mode=rollback unreadable=[] selected=(1,5) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0,rollback) first_txg=6 admission_short=true reach=Complete+Some(UnitsOnly) landed=[(2,6)] | mounted=(2,6) table=(1,3,0,rollback) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 93 行：
```text
  M2 mode=rollback unreadable=[] selected=(1,5) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0,rollback) first_txg=6 admission_short=true reach=Complete+Some(UnitsOnly) landed=[(2,6)] | mounted=(2,6) table= U1=0 [] U2=2 [i1t3.node0(i1,b3,n0,Metadata,start0),i1t3.data1(i1,b3,n1,Data,start0)] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 今天的字面：(2,6) 是回退实例的第一个根却不带回退行；M3 选 (2,6)、只写 (2,6,0)，被回退抛弃的 (1,4)(1,5) 共 6 个单元按「无行 ⇒ 已发布」翻面。D23 第 1206 行「回退与它的第一个新根同一次发布」已经不许这个次序，D16 第 377 行准入行没有例外——与前提五同一个缺口，只是换成回退形态。
- 甲、乙：0。丙：U2 = 2（实例 1 被 R_old 引用的节点与码 1）。

### 换故障二：实例切换代替恢复

持久写集合：M1 写成 (1,1)(1,2)(1,3)；M2 选 (1,3)，写成 (2,4)(2,5)；发布中的 txg 6 两个码 1（事务 2、3）落了、固定点写失败 ⇒ 切到实例 3，切换那句要的行是 (2, 5, 3)（D23 第 691 行「写行 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号)」）。准入不够时今天的字面与丙先推不带行的抬 F (3,6)，乙先推带行的 (3,6)，甲第一次就是重发 (3,6)；第一个根落了，第二次只落了单元就崩；M3、M4 普通恢复。
「号 ≤ W 的那两个码 1 在不在抬 F 那个根里」条文容得下两种读法：D28 第 86 行「重建态从所选根 + 施加到 W 的记录来」，而 D16 已定项 7 是一次发布的单元都落了才写记录（`OnlyInMemory`：失败在单元写那一步，两个事务没有记录，不在重建态里）；C287（切换收养开放 checkpoint 的事务后再崩） 那一行写着「事务 a 写完单元 X、记录已追加」（`InRebuildState`）。两种都跑了：

`first-pass.out` 第 14 行：
```text
  M2' switch old=2 new=3 switch_row=(2,5,3) kept=[i2t6.failed.data2(i2,b6,n2,Data,start3),i2t6.failed.data3(i2,b6,n3,Data,start3)] first=fraise second=resend(units_only) crash | mounted=(3,6) table=(1,3,0) U1=3 [i2t6.failed.accounting(i2,b6,n0,Metadata,start3),i2t6.failed.data2(i2,b6,n2,Data,start3),i2t6.failed.data3(i2,b6,n3,Data,start3)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 19 行：
```text
  M2' switch old=2 new=3 switch_row=(2,5,3) kept=[i2t6.failed.data2(i2,b6,n2,Data,start3),i2t6.failed.data3(i2,b6,n3,Data,start3)] first=fraise second=resend(units_only) crash | mounted=(3,6) table=(1,3,0) U1=1 [i2t6.failed.accounting(i2,b6,n0,Metadata,start3)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 42 行：
```text
  M2' switch old=2 new=3 switch_row=(2,5,3) kept=[i2t6.failed.data2(i2,b6,n2,Data,start3),i2t6.failed.data3(i2,b6,n3,Data,start3)] first=resend second=warmup(units_only) crash | mounted=(3,6) table=(1,3,0)(2,5,3) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 70 行：
```text
  M2' switch old=2 new=3 switch_row=(2,5,3) kept=[i2t6.failed.data2(i2,b6,n2,Data,start3),i2t6.failed.data3(i2,b6,n3,Data,start3)] first=fraise second=resend(units_only) crash | mounted=(3,6) table=(1,3,0)(2,5,3) U1=2 [i2t6.failed.data2(i2,b6,n2,Data,start3),i2t6.failed.data3(i2,b6,n3,Data,start3)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 75 行：
```text
  M2' switch old=2 new=3 switch_row=(2,5,3) kept=[i2t6.failed.data2(i2,b6,n2,Data,start3),i2t6.failed.data3(i2,b6,n3,Data,start3)] first=fraise second=resend(units_only) crash | mounted=(3,6) table=(1,3,0)(2,5,3) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 98 行：
```text
  M2' switch old=2 new=3 switch_row=(2,5,3) kept=[i2t6.failed.data2(i2,b6,n2,Data,start3),i2t6.failed.data3(i2,b6,n3,Data,start3)] first=fraise second=resend(units_only) crash | mounted=(3,6) table=(1,3,0) U1=0 [] U2=1 [i2t4.data1(i2,b4,n1,Data,start3)] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 今天的字面：实例 2 没有行，失败那次的记账单元 (2, 诞生 6) 两种读法下都翻面；`OnlyInMemory` 下两个码 1 也翻面（它们不在 (3,6) 里，却按无行判已发布）。
- 乙：`OnlyInMemory` 下 (3,6) 带着 (2,5,3)，n ≤ 3 把两个码 1 判已发布，而 (3,6) 是空发布、没引用它们，U1 = 2，M3、M4 之后仍是 2（重发那次的根一直没落）；`InRebuildState` 下 0。乙「本实例写出的每一个根，它引用的实例表都已含那批行」按字面把 W 也带进了还没收养那几个事务的根。收严的写法是「写行那次之前的根带同一批行、W 取 0」，那就又回到了「写行那次」与别的根不同。
- 甲：两种读法都 0。丙：U2 = 1 / 3（实例 2 自己已发布的码 1，`InRebuildState` 下再加被照旧的两个）。

### 换故障三：「非干净结束」按看着像干净判

持久写集合：与前提五的 M1 相同（(1,4) 只落单元、没有记录）；M2、M3、M4 普通恢复，两次发布都写成。实现按「所选根之后没有可施加的记录、也没有读不出的根」判上一个实例干净结束，不写行（最弱读法）；对照是每次都写行。

`first-pass.out` 第 23 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written= first_txg=4 admission_short=false reach=Complete+Some(Complete) landed=[(2,4)(2,5)] | mounted=(2,5) table= U1=3 [i1t4.accounting(i1,b4,n0,Metadata,start0),i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.data2(i1,b4,n2,Data,start0)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 51 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written= first_txg=4 admission_short=false reach=Complete+Some(Complete) landed=[(2,4)(2,5)] | mounted=(2,5) table= U1=3 [i1t4.accounting(i1,b4,n0,Metadata,start0),i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.data2(i1,b4,n2,Data,start0)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 79 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written= first_txg=4 admission_short=false reach=Complete+Some(Complete) landed=[(2,4)(2,5)] | mounted=(2,5) table= U1=3 [i1t4.accounting(i1,b4,n0,Metadata,start0),i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.data2(i1,b4,n2,Data,start0)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 107 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written= first_txg=4 admission_short=false reach=Complete+Some(Complete) landed=[(2,4)(2,5)] | mounted=(2,5) table= U1=0 [] U2=2 [i1t3.node0(i1,b3,n0,Metadata,start0),i1t3.data1(i1,b3,n1,Data,start0)] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 55 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0) first_txg=4 admission_short=false reach=Complete+Some(Complete) landed=[(2,4)(2,5)] | mounted=(2,5) table=(1,3,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 今天、甲、乙都中：(1,4) 的 3 个孤儿按「无行 ⇒ 已发布」翻面（U1 = 3），甲的次序救不了，因为甲只管「写行那次」排在哪，这一格根本没有写行那次。丙：U2 = 2，到 M4 涨到 4（M2、M3 自己的码 1 也因为没有行判未发布）。每次都写行时四条臂都 0。
- 判别子观测不到：M1 崩在 (1,4) 的单元落了、记录还没写的那一刻，与 M1 在 (1,3) 之后干净卸载，盘上的根、记录、超级块逐项相同，差别只在空闲空间里那 3 个孤儿，找它们要遍历（`.claude/rules/fs-design.md` 第一格不许在运行时决策路径上遍历）。
- 收严（只加要求）：删掉 D18 第 879 行「任何非干净结束的实例在下次恢复写行」里的「非干净结束的」这个限定，每次可写挂载都给 [所选根的实例, 新实例) 写行，实例 0（mkfs）除外——第一次可写挂载写不出任何一行，第一个事务的字节不变。代价是每次挂载一行 + 一次链重写，行按 D18 第 879 行的回收条件删。

### 丙的另一处：自己写自己的行

`first-pass.out` 第 114 行：
```text
C329_BING_CLEAN_END_ROW rules=c329_bing mounted=(1,4) table=(1,4,1) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[1]
```

丙「干净结束也写行 (i, 最后发布的 txg, W)」若写在该实例自己的最后一次发布里，挂载根 (1,4) 的表里就有实例 1 的行，I-3.8 按字面「行的实例代号 < 挂载根实例」红。若改成由下一次挂载写，它就是「换故障三」那条收严，与丙的谓词无关。

### 连退几格、几次抬 F、跳过的号

没有单独写剧本，按构造判：甲让写行那次成为本实例的第一个根，此后每个根都引用同一张表（D18 第 879 行「每次写行 COW 重写整条链」，之后的根指向这条链），所以「本实例已有根落盘、而 [所选根的实例, 本实例) 里有实例没有行」这个态在甲下只能来自「不写行」（换故障三），与抬 F 几次无关；连退几格只让 [所选根的实例, 新实例) 变宽；C322 跳过的号落在这个区间里。「五、C330：U1」的枚举用的就是甲的次序（准入够、不推抬 F），610000 条历史里连退两格与回退都有。

## 四、C329：U3 与 U5（写行那次发布拿不到分配）

### 切换预留被恢复自己先用掉一份

`first-pass.out` 第 475 行：
```text
RESERVATION c_max=4 per_disk: per_switch=14 reserved(N_switch=3)=42 | jia_row_publication_only=6 remaining=36 full_switches_left=2 | jia_row_publication_plus_warm_up_worst=14 remaining=28 full_switches_left=2
```
`first-pass.out` 第 476 行：
```text
RESERVATION c_max=9 per_disk: per_switch=29 reserved(N_switch=3)=87 | jia_row_publication_only=11 remaining=76 full_switches_left=2 | jia_row_publication_plus_warm_up_worst=29 remaining=58 full_switches_left=2
```
`first-pass.out` 第 477 行：
```text
WARM_UP first_txg_mod_3=0 publications_to_cover_both_disks=2
```
`first-pass.out` 第 478 行：
```text
WARM_UP first_txg_mod_3=1 publications_to_cover_both_disks=2
```
`first-pass.out` 第 479 行：
```text
WARM_UP first_txg_mod_3=2 publications_to_cover_both_disks=3
```

- D28 第 82 行的式子「预留 = N_switch × 这个最坏量（N_switch = 3，D23（journal 的角色与格式） 已定项 14）」，第 88 行「一次切换的最坏量再加「至多 R 次空发布 × 现算 c_max」」。第一个事务几何下每盘每次切换 2 + 3 × c_max 块（c_max = 4 ⇒ 14，c_max = 9 ⇒ 29），预留 42 / 87，与第 88 行「与链重写的 6 块相加后第一个事务几何下每块盘 42 / 87 块」相同（算术对账）。
- 甲「它的分配走切换预留」：恢复的写行那次自己用掉链重写 2 + 自己那次的固定点 c_max = 每盘 6 / 11 块；暖机若也记在预留上，最坏 14 / 29（首 txg ≡ 2 mod 3 时要 3 次发布才覆盖两块盘，`WARM_UP` 第三行）。两种记法下剩下的都只够 2 次整切换 ⇒ 这次挂载里的第 3 次切换走 D23 第 691 行「**切换自己的预留拿不到** ⇒ 才转只读」，而挂载准入承诺的是 N_switch = 3 次。
- 挂载序列（推理，模型只算了块数，没建切换的空间账）：M1 写成 (1,1)(1,2)(1,3) 后崩；M2 恢复，写行那次走预留写成、暖机写成；之后三次单元写失败、每次探针写都写得进去 ⇒ 三次切换，第 3 次拿不到预留 ⇒ 只读。今天的次序下写行那次走准入，预留原封不动，第 3 次拿得到。
- 收严的两种写法：D28 已定项 3 的预留按 (N_switch + 1) 份算，多出的一份专给「恢复那次写行 + 暖机」（`df` 每盘多扣 14 / 29 块）；或写行那次不走预留、走准入，但不许先推抬 F，拿不到就只读。

### 写行那次的量不止预留那一份：重放施加进来的码 2 / 3 单元

D18 第 879 行「码 2 / 3 ⇒ b ≤ T_pub（所选根之后的固定点单元一律未发布，恢复实例重写它们；那 6 字节事务号在码 2 / 3 上不参与任何判定」与 D23 第 694 行「施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义」放在一起：被重放施加的那次发布写的码 2 / 3 单元，被重建出来的根原样引用；要它们不被判未发布，恢复实例得在自己的第一个根里把它们全部重写。三种处理（M1 的 (1,4) 带 2 个码 1 与 3 个节点，记录落了、根没落就崩；M2 选 (1,3)、重放 (1,4)，W = 3）：

`first-pass.out` 第 162 行：
```text
  M2 selected=(1,3) after_replay=(1,4) W=3 acquired=2 rows_written=(1,3,3) replayed_nodes=[i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.node1(i1,b4,n0,Metadata,start0),i1t4.node2(i1,b4,n0,Metadata,start0)] row_publication_new_units=accounting1+rewritten0 | mounted=(2,6) table=(1,3,3) U1=0 [] U2=3 [i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.node1(i1,b4,n0,Metadata,start0),i1t4.node2(i1,b4,n0,Metadata,start0)] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 167 行：
```text
  M2 selected=(1,3) after_replay=(1,4) W=3 acquired=2 rows_written=(1,3,3) replayed_nodes=[i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.node1(i1,b4,n0,Metadata,start0),i1t4.node2(i1,b4,n0,Metadata,start0)] row_publication_new_units=accounting1+rewritten3 | mounted=(2,6) table=(1,3,3) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 172 行：
```text
  M2 selected=(1,3) after_replay=(1,4) W=3 acquired=2 rows_written=(1,4,3) replayed_nodes=[i1t4.node0(i1,b4,n0,Metadata,start0),i1t4.node1(i1,b4,n0,Metadata,start0),i1t4.node2(i1,b4,n0,Metadata,start0)] row_publication_new_units=accounting1+rewritten0 | mounted=(2,6) table=(1,4,3) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 原样收养（`AdoptedAsIs`）：挂载根引用的 3 个节点（诞生 4）按行 (1,3,3) 判未发布，U2 = 3，M3、M4 之后仍是 3；码 1 靠 n ≤ W 判已发布，没事。这一格对今天、甲、三种乙逐字相同（`first-pass.out` 第 220-234 行等），不分辨 C330 的任何一条臂，放在「六、C330：U2」再说。
- 恢复重写（`RewrittenByRecovery`）：U2 = 0，而写行那次多写的单元数 = 被重放那几次发布点名的码 2 / 3 单元数（`row_publication_new_units=accounting1+rewritten3`）。这个数的上界是被重放记录的点名项（D23 已定项 4：一条 4096 记录装 67 个点名项）乘被重放的记录数，与 c_max 无关、切换预留罩不住。近满盘时甲的写行那次拿不到分配，而甲不许先推抬 F ⇒ 只读；每次重挂盘上状态一样 ⇒ 一直只读（推理）。
- 行的 T_pub 取重放之后那个根的 txg（`RowTxgAfterReplay`）：U2 = 0、不多写一个单元。它改的是 D18 第 879 行「(i, **所选根的 checkpoint_txg**, …)」对所选根那个实例的取值，不是 C330 的中间实例那一行。
- 不专属于甲：乙的第一个根同样要带那次重写（带着没重写的节点落盘就是 U2），今天的次序下抬 F 那个根也一样。甲专有的只是把分配来源写死成切换预留。C284（施加一条记录在指针层上做什么没有定义） 那一行的空间侧注早就写着「全登记则记录点名的码 2 祖先被登记成已分配、却按已发布谓词要重写」，这一格是它在 D23 已定项 15 定案之后的具体形态，建议并进 C284，不另立。

### U5：逐字的冲突（每条两句）

1. D16 第 377 行：「准入不够时先推空发布抬 F，一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC」；甲：「它的分配走切换预留（D28（挂载期承诺量） 已定项 3），不走准入、不推抬 F 的空发布」。准入行没有「写行那次除外」，要补一句。
2. D28 第 82 行：「预留 = N_switch × 这个最坏量（N_switch = 3，D23（journal 的角色与格式） 已定项 14）」；甲同一句。预留按 N_switch 次切换算，甲让恢复先用掉一份（本节第一小节的算术）。
3. D28 第 86 行：「号 > W 的用户数据分配失败按 D23（journal 的角色与格式） 已定项 14 向调用者返回 ENOSPC，不阻塞切换」；甲同一句。实例切换的写行那次就是重发在飞 checkpoint 的那次，里面有号 > W 的用户数据重做，甲的整句把用户数据也划到预留名下。收严：「写行那次的元数据（实例表链重写 + 固定点）走切换预留；同一次发布里的用户数据重做照 D28 第 86 行走准入、不够就向调用者返回 ENOSPC，不推抬 F」。
4. 不冲突的：D16 第 207 行「新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回、不向管理员确认回退；做法是连推空发布」——写行那次的根也是本实例写成的根，算进覆盖，`WARM_UP` 三行给出首 txg ≡ 0 / 1 / 2 (mod 3) 时 2 / 2 / 3 次发布覆盖两块盘，与「至多 R = 3 次」相容；D23 第 1206 行「回退与它的第一个新根同一次发布」与甲同向。

## 五、C330：U1

### 前提六复现与六条臂（M1 之后四次挂载）

持久写集合（`today`，`m2_first_reach=RootNoSuperblock`）：M1 同前提五（(1,1)(1,2)(1,3) 写成，(1,4) 只落单元）；M2 选 (1,3)、取号 2、写行 (1,3,0)，第一次发布 txg 4（记账 + 1 个节点 + 1 个码 1）的单元、记录、根 (2,4) 落了，超级块槽没落就崩；M3 那条 (2,4) 暂时读不出，选 (1,3)、取号 3，第一次发布（同样三个单元）只落了单元就崩；M4、M5 全部读得出，两次发布都写成。M4 那一行之后是 M4 第一个根下实例 2、3 各单元的谓词输入：

`first-pass.out` 第 116-124 行：
```text
  M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0) first_txg=4 admission_short=false reach=RootNoSuperblock+Some(UnitsOnly) landed=[(2,4)] | mounted=(2,4) table=(1,3,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
  M3 mode=recovery unreadable=[(2, 4)] selected=(1,3) after_replay=(1,3) W=0 acquired=3 rows_written=(1,3,0)(2,3,0) first_txg=4 admission_short=false reach=UnitsOnly+None landed=[] | mounted=none(这次挂载一个根都没落，新行不生效)
  M4 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=4 rows_written=(1,3,0)(2,4,0)(3,4,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(4,5)(4,6)] | mounted=(4,6) table=(1,3,0)(2,4,0)(3,4,0) U1=3 [i3t4.accounting(i3,b4,n0,Metadata,start3),i3t4.node0(i3,b4,n0,Metadata,start3),i3t4.data1(i3,b4,n1,Data,start3)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
    INPUTS i2t4.accounting code=Metadata b=4 start=3 n=0 row(T_pub,W)=Some((4, 0)) | verdict=Published referenced=false on_timeline=true
    INPUTS i2t4.node0 code=Metadata b=4 start=3 n=0 row(T_pub,W)=Some((4, 0)) | verdict=Published referenced=true on_timeline=true
    INPUTS i2t4.data1 code=Data b=4 start=3 n=1 row(T_pub,W)=Some((4, 0)) | verdict=Published referenced=true on_timeline=true
    INPUTS i3t4.accounting code=Metadata b=4 start=3 n=0 row(T_pub,W)=Some((4, 0)) | verdict=Published referenced=false on_timeline=false
    INPUTS i3t4.node0 code=Metadata b=4 start=3 n=0 row(T_pub,W)=Some((4, 0)) | verdict=Published referenced=false on_timeline=false
    INPUTS i3t4.data1 code=Data b=4 start=3 n=1 row(T_pub,W)=Some((4, 0)) | verdict=Published referenced=false on_timeline=false
```
`first-pass.out` 第 178-184 行：
```text
  M4 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=4 rows_written=(1,3,0)(2,4,0)(3,0,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(4,5)(4,6)] | mounted=(4,6) table=(1,3,0)(2,4,0)(3,0,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
    INPUTS i2t4.accounting code=Metadata b=4 start=3 n=0 row(T_pub,W)=Some((4, 0)) | verdict=Published referenced=false on_timeline=true
    INPUTS i2t4.node0 code=Metadata b=4 start=3 n=0 row(T_pub,W)=Some((4, 0)) | verdict=Published referenced=true on_timeline=true
    INPUTS i2t4.data1 code=Data b=4 start=3 n=1 row(T_pub,W)=Some((4, 0)) | verdict=Published referenced=true on_timeline=true
    INPUTS i3t4.accounting code=Metadata b=4 start=3 n=0 row(T_pub,W)=Some((0, 0)) | verdict=Unpublished referenced=false on_timeline=false
    INPUTS i3t4.node0 code=Metadata b=4 start=3 n=0 row(T_pub,W)=Some((0, 0)) | verdict=Unpublished referenced=false on_timeline=false
    INPUTS i3t4.data1 code=Data b=4 start=3 n=1 row(T_pub,W)=Some((0, 0)) | verdict=Unpublished referenced=false on_timeline=false
```

- 今天：M4 选 (2,4)、写 (2,4,0)(3,4,0)，M3 的三个孤儿（诞生 4）按 4 ≤ 4 翻面，U1 = 3，M5 之后仍是 3。前提六按条文走得出来，失败条款不触发。
- 甲：写 (3,0,0)，0。M2 超级块也落了（第 186-197 行）、M2 是管理员回退（第 198-209 行）两种形态同样 0；今天在回退形态上同中：

`first-pass.out` 第 141 行：
```text
  M4 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=4 rows_written=(1,1,0,rollback)(2,4,0)(3,4,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(4,5)(4,6)] | mounted=(4,6) table=(1,1,0,rollback)(2,4,0)(3,4,0) U1=3 [i3t4.accounting(i3,b4,n0,Metadata,start3),i3t4.node0(i3,b4,n0,Metadata,start3),i3t4.data1(i3,b4,n1,Data,start3)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 201 行：
```text
  M4 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=4 rows_written=(1,1,0,rollback)(2,4,0)(3,0,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(4,5)(4,6)] | mounted=(4,6) table=(1,1,0,rollback)(2,4,0)(3,0,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 乙的三种落点，同一段历史的 M4 那一行：

`first-pass.out` 第 238 行：
```text
  M4 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=4 rows_written=(1,3,0)(2,4,0)(3,4,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(4,5)(4,6)] | mounted=(4,6) table=(1,3,0)(2,4,0)(3,4,0) U1=3 [i3t4.accounting(i3,b4,n0,Metadata,start3),i3t4.node0(i3,b4,n0,Metadata,start3),i3t4.data1(i3,b4,n1,Data,start3)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 249 行：
```text
  M4 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=4 rows_written=(1,3,0)(2,4,0)(3,4,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(4,5)(4,6)] | mounted=(4,6) table=(1,3,0)(2,4,0)(3,4,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 298 行：
```text
  M4 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=4 rows_written=(1,3,0)(2,4,0)(3,4,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(4,5)(4,6)] | mounted=(4,6) table=(1,3,0)(2,4,0)(3,4,0) U1=3 [i3t4.accounting(i3,b4,n0,Metadata,start3),i3t4.node0(i3,b4,n0,Metadata,start3),i3t4.data1(i3,b4,n1,Data,start3)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 358 行：
```text
  M4 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=4 rows_written=(1,3,0)(2,4,0)(3,4,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(4,5)(4,6)] | mounted=(4,6) table=(1,3,0)(2,4,0)(3,4,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

  - 超级块水位（第 238 行）：「根落了、超级块没落」这一格 M3 读到的水位还是 3，首 txg 4，U1 = 3；超级块也落了（第 249 行）首 txg 5，0。这一格可达，因为 D16 第 290 行「根槽持久之后再更新超级块槽」让根 FUA 与超级块槽写是两段（`layout/01-first-txn.md` 八那张段序列表）。
  - 根记录带水位（第 298 行）：M3 读得出的根只到 (1,3)，水位 3，与今天同中。
  - 超级块 ∪ 读得出的根 ∪ 全环记录（第 358 行）：M3 读得到 (2,4) 那条记录（计数器 4，还没被覆盖），水位 4、首 txg 5，0。记录本来就带 checkpoint_txg，这一种不加格式字段；恢复本来就先全环扫描（D23 已定项 3）。
- 丙：谓词输入六行里 `i2t4.node0` 与 `i3t4.node0`、`i2t4.data1` 与 `i3t4.data1` 两对逐项相同——(码, b = 4, 起点 = 3, n, 行 (T_pub, W) = (4, 0))——前者被 (2,4) 引用（`referenced=true`），后者是孤儿。任何只看这些输入的谓词都给两者同一个判定，必错一个：

`first-pass.out` 第 418 行：
```text
  M4 mode=recovery unreadable=[] selected=(2,4) after_replay=(2,4) W=0 acquired=4 rows_written=(1,3,0)(2,4,0)(3,4,0) first_txg=5 admission_short=false reach=Complete+Some(Complete) landed=[(4,5)(4,6)] | mounted=(4,6) table=(1,3,0)(2,4,0)(3,4,0) U1=0 [] U2=4 [i1t3.node0(i1,b3,n0,Metadata,start0),i1t3.data1(i1,b3,n1,Data,start0),i2t4.node0(i2,b4,n0,Metadata,start3),i2t4.data1(i2,b4,n1,Data,start3)] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

  `c330_bing_start`（起点 < T_pub 时 b 那一支不算）把孤儿判对了，却把 (2,4) 引用的 `i2t4.node0`、`i2t4.data1` 与实例 1 的节点、码 1 判未发布，U2 = 4；不用起点那一支就是今天，U1 = 3。实例 2、3 都从 (1,3) 起，所以把「起点根的 txg」换成「起点根的 (实例, txg)」也一样分不开。能分开它们的只有「谁是所选根那个实例」，而那正是甲写进行里的东西。按 evidence-discipline「判别子观测不到」判：丙这一形态出局，除非另在行上做标记——那就是甲。

### 枚举：四次挂载，610000 条历史

口径：M1（同前提五，(1,4) 只落 1 个码 1）之后四次可写挂载，每次选普通恢复或管理员回退（有候选才选）× 本次暂时读不出 0 / 1 / 2 条最新的根 × 做到哪一步（只落单元 / 单元 + 记录 / 根落了而超级块没落 / 第一次写成、第二次只落单元 / 两次都写成）；每次挂载有根落了就判一次，四次之后按下一次挂载的第一个根再判一次。两个开关：M1 的 (1,3) 带不带一直被引用的节点（带就混进「重放施加进来的节点」那一类 U2，归「六、C330：U2」）；回退那次发布的实例表以哪一版为底（`NewestReadableRootTable` 是第一遍的取法，`RollbackTargetTable` 以 R_old 引用的那一版为底）。M1 不带节点的 12 行：

`attribution.out` 第 90 98 107 117 125 126 133 137 138 143 147 148 行：
```text
ENUM2 base=m1_without_nodes rollback_table_base=NewestReadableRootTable rules=today histories=610000 evaluations=988735 u1_units=1069786 u2_units=1955 histories_with_u1=291529 histories_with_u2=1074 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=NewestReadableRootTable rules=c330_jia histories=610000 evaluations=988735 u1_units=104672 u2_units=1775 histories_with_u1=14853 histories_with_u2=984 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=NewestReadableRootTable rules=c330_yi_superblock histories=610000 evaluations=988735 u1_units=246382 u2_units=270 histories_with_u1=81260 histories_with_u2=135 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=NewestReadableRootTable rules=c330_yi_root_carried histories=610000 evaluations=988735 u1_units=1069786 u2_units=1955 histories_with_u1=291529 histories_with_u2=1074 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=NewestReadableRootTable rules=c330_yi_record_scan histories=610000 evaluations=988735 u1_units=0 u2_units=0 histories_with_u1=0 histories_with_u2=0 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=NewestReadableRootTable rules=c330_bing_start histories=610000 evaluations=988735 u1_units=88078 u2_units=1408162 histories_with_u1=15123 histories_with_u2=610000 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=RollbackTargetTable rules=today histories=610000 evaluations=988735 u1_units=945569 u2_units=0 histories_with_u1=279793 histories_with_u2=0 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=RollbackTargetTable rules=c330_jia histories=610000 evaluations=988735 u1_units=0 u2_units=0 histories_with_u1=0 histories_with_u2=0 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=RollbackTargetTable rules=c330_yi_superblock histories=610000 evaluations=988735 u1_units=242440 u2_units=0 histories_with_u1=81080 histories_with_u2=0 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=RollbackTargetTable rules=c330_yi_root_carried histories=610000 evaluations=988735 u1_units=945569 u2_units=0 histories_with_u1=279793 histories_with_u2=0 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=RollbackTargetTable rules=c330_yi_record_scan histories=610000 evaluations=988735 u1_units=0 u2_units=0 histories_with_u1=0 histories_with_u2=0 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_without_nodes rollback_table_base=RollbackTargetTable rules=c330_bing_start histories=610000 evaluations=988735 u1_units=0 u2_units=1410642 histories_with_u1=0 histories_with_u2=610000 evaluations_with_I-3.8_violation=0
```

| 臂 | 最新根的表为底：有 U1 / 有 U2 的历史数 | R_old 的表为底：有 U1 / 有 U2 的历史数 |
|---|---|---|
| `today` | 291529 / 1074 | 279793 / 0 |
| `c330_jia` | 14853 / 984 | 0 / 0 |
| `c330_yi_superblock` | 81260 / 135 | 81080 / 0 |
| `c330_yi_root_carried` | 291529 / 1074 | 279793 / 0 |
| `c330_yi_record_scan` | 0 / 0 | 0 / 0 |
| `c330_bing_start` | 15123 / 610000 | 0 / 610000 |

I-3.8 在 24 组枚举里一次都没红。确定性模型，每组跑一遍（跑多遍只说明没有隐藏状态）。

### 甲在枚举里唯一的一类 U1：回退那次发布的实例表取哪一版

`attribution.out` 第 99 100 101 102 行：
```text
ENUM2_EXAMPLE base=m1_without_nodes rollback_table_base=NewestReadableRootTable rules=c330_jia kind=U1 faults=5
    M2 mode=recovery unreadable=[] selected=(1,3) after_replay=(1,3) W=0 acquired=2 rows_written=(1,3,0) first_txg=4 admission_short=false reach=RootNoSuperblock+None landed=[(2,4)] | mounted=(2,4) table=(1,3,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
    M3 mode=rollback unreadable=[(2, 4), (1, 3)] selected=(1,2) after_replay=(1,1) W=0 acquired=3 rows_written=(1,1,0,rollback)(2,0,0) first_txg=3 admission_short=false reach=RootNoSuperblock+None landed=[(3,3)] | mounted=(3,3) table=(1,1,0,rollback)(2,0,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
    M4 mode=rollback unreadable=[] selected=(2,4) after_replay=(3,3) W=0 acquired=4 rows_written=(1,3,0)(3,3,0,rollback) first_txg=5 admission_short=false reach=RootNoSuperblock+None landed=[(4,5)] | mounted=(4,5) table=(1,3,0)(3,3,0,rollback) U1=5 [i1t2.accounting(i1,b2,n0,Metadata,start0),i1t3.accounting(i1,b3,n0,Metadata,start0),i1t3.data1(i1,b3,n1,Data,start0),i2t4.accounting(i2,b4,n0,Metadata,start3),i2t4.data1(i2,b4,n1,Data,start3)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 持久写集合与所选根：M2 选 (1,3)、取号 2，第一个根 (2,4) 落了（超级块没落）就崩；M3 那条 (2,4) 与 (1,3) 都暂时读不出，管理员回退——最新可读的是 (1,2)，按它的表取候选里第二新的 (1,1)，取号 3，写 (1,1,0,回退)(2,0,0)，回退根 txg = 读得出的根里最大的 2 + 1 = 3，(3,3) 落了就崩；M4 全部读得出，最新的是 (2,4)（txg 4 > 3），管理员再回退：候选集按 (2,4) 的表 {(1,3,0)}，(3,3) 在里面（表里没有实例 3 的行），回退到 (3,3)，实例表以 (2,4) 那一版为底，写出 (1,3,0)(3,3,0,回退)——实例 2 没有行、实例 1 的 T_pub 是 3。挂载根 (4,5) 的时间线是 (1,1) → (3,3) → (4,5)，于是实例 1 诞生 2、3 的单元（按 (1,3,0) 判已发布）与实例 2 的单元（无行 ⇒ 已发布）共 5 个翻面。故障：两条根暂时读不出 + 三次崩 = 5。
- 换成以 R_old（(3,3)）引用的那一版为底：表是 (1,1,0,回退)(2,0,0)，再加 (3,3,0,回退)，五个都判未发布；甲在同一个枚举上 U1 = 0、U2 = 0。
- 打中之后的三问。分不分辨臂：分辨——同一格乙的记录扫描水位不中（M3 的回退新根取 max(读得出的根, 全环记录) + 1 = 5，被抛弃的 (2,4) 排不到最前）；今天也中，而且今天还有 3 个故障就中的回退形态（`attribution.out` 第 134-136 行）。系统当时看不看得到：看得到，R_old 自己的表就在盘上，只是条款没说用它。满足的是哪个分句：U1 的触发句「写出候选下一个孤儿翻成已发布的可达状态（持久写集合与每次挂载的所选根）」，逐字满足。
- ⇒ 按「记一次输 / 只许收严 / 写明收严在哪」：甲按最弱读法（实例表以挂载时最新可读的根那一版为底）记一次输；收严是给 D23 第 1206 行回退那句补「回退那次发布的实例表以 R_old 引用的那一版为底，再写回退行与中间实例的 (i, 0, 0)」——多要求一件事，没少要求任何事；收严之后甲在同一个枚举上 0。这一句对今天与乙同样要补（它们在 R_old 的表为底时 U2 从 1074 / 135 降到 0）。
- 这一格的另一半是择根：M4 的「最新」(2,4) 其实早于 (3,3)。那是 C330 的病根（新实例的 txg 从较旧的根往上数）在择根上的样子，甲只管行，不管这一半。

### 择根倒挂：已确认的写被较旧实例的根压过去（第二遍的 `inversion` 入口）

`inversion.out` 第 1-12 行：
```text
INVERSION_ACROSS_INSTANCES rules=today
  M2 acquired=2 rows_written=(1,3,0) landed=[(2, 4), (2, 5), (2, 6), (2, 7)] all_complete
  M3 unreadable=[(2, 4), (2, 5), (2, 6), (2, 7)] selected=(1,3) after_replay=(1,3) acquired=3 rows_written=(1,3,0)(2,3,0) first_txg=4 warm_up_roots_on_disks=[1, 0] confirmed_publication=(3,6) crash
  M4 selected=(2,7) acquired=4 rows_written=(1,3,0)(2,7,0)(3,7,0) | confirmed_unit=i3t6.fsync.data2 referenced_by_mounted=false verdict=Published | mounted=(4,9) table=(1,3,0)(2,7,0)(3,7,0) U1=5 [i3t4.accounting(i3,b4,n0,Metadata,start3),i3t4.data1(i3,b4,n1,Data,start3),i3t5.accounting(i3,b5,n0,Metadata,start3),i3t6.fsync.accounting(i3,b6,n0,Metadata,start3),i3t6.fsync.data2(i3,b6,n2,Data,start3)] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
INVERSION_ACROSS_INSTANCES rules=c330_jia
  M2 acquired=2 rows_written=(1,3,0) landed=[(2, 4), (2, 5), (2, 6), (2, 7)] all_complete
  M3 unreadable=[(2, 4), (2, 5), (2, 6), (2, 7)] selected=(1,3) after_replay=(1,3) acquired=3 rows_written=(1,3,0)(2,0,0) first_txg=4 warm_up_roots_on_disks=[1, 0] confirmed_publication=(3,6) crash
  M4 selected=(2,7) acquired=4 rows_written=(1,3,0)(2,7,0)(3,0,0) | confirmed_unit=i3t6.fsync.data2 referenced_by_mounted=false verdict=Unpublished | mounted=(4,9) table=(1,3,0)(2,7,0)(3,0,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
INVERSION_ACROSS_INSTANCES rules=c330_yi_superblock
  M2 acquired=2 rows_written=(1,3,0) landed=[(2, 4), (2, 5), (2, 6), (2, 7)] all_complete
  M3 unreadable=[(2, 4), (2, 5), (2, 6), (2, 7)] selected=(1,3) after_replay=(1,3) acquired=3 rows_written=(1,3,0)(2,3,0) first_txg=8 warm_up_roots_on_disks=[0, 0] confirmed_publication=(3,10) crash
  M4 selected=(3,10) acquired=4 rows_written=(1,3,0)(2,3,0)(3,10,0) | confirmed_unit=i3t10.fsync.data2 referenced_by_mounted=true verdict=Published | mounted=(4,12) table=(1,3,0)(2,3,0)(3,10,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 持久写集合：M1 写成 (1,1)(1,2)(1,3)；M2 选 (1,3)，写成 (2,4)..(2,7)（暖机两次 + 两次带码 1 的发布）后崩；M3 那四条 (2,4)..(2,7) 全部暂时读不出，选 (1,3)（重放在计数器 4 那条实例 2 的记录处停），取号 3，暖机 (3,4)(3,5) 落在盘 1、盘 0 上（`warm_up_roots_on_disks=[1, 0]`，D16 第 207 行的条件满足），再写成 (3,6)，它的码 1 的 fsync 已返回，然后崩；M4 全部读得出。
- 今天、甲、根记录水位（第 13-16 行同今天）：M4 选 (2,7)（txg 7 > 6），M3 已确认的 `i3t6.fsync.data2` 不被挂载根引用——那次写丢了。今天另有 U1 = 5（M3 的单元按 (3,7,0) 判已发布）；甲 U1 = 0、U2 = 0，谓词自洽，丢掉的写没有任何检查报警。
- 超级块水位与记录扫描水位（第 17-20 行同第 9-12 行）：M3 首 txg 8，(3,8)(3,9) 都落在盘 0 上，要到 (3,10) 才覆盖两块盘，确认的是 (3,10)，M4 选 (3,10)，没丢。
- 故障：四条根暂时读不出 + 两次崩 = 6。四条根跨两块盘（区域 = txg mod 3、归属 0 / 1 / 0），要逐槽各坏一次；整块盘掉只能只读挂载，走不到这里。
- 它不是 U1 / U2（甲下谓词没判错），是「fsync 返回之后的写在一个可达的挂载序列里丢了」。D16 第 207 行暖机挡的是「新实例的第一个根是单点」，挡不住「较旧实例的根 txg 更高」。报出来给判决在甲与乙之间取舍用。

### 同一个装置的另两个消费者（推理，没建模）

- D23 第 1206 行回退「第一个新根的 checkpoint_txg = 根环里全部根记录 txg 的最大值 + 1」：读不出的根取不到，就是枚举里 3 个故障的回退形态与甲那一类 U1 的来处。
- D08（核心索引结构） 第 476 行树 ID 水位「新水位 = max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)」：前提六那一格下 M3 重发 M2 发过的树 ID。它「丙-2 唯一盖不住的那一格，判为不是错误」的第 ② 条依据是 I-1.2「判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留」；今天的写行下前提六的孤儿判成已发布，那条依据在这一格不成立，甲下成立。
- 记录扫描水位的剩余缺口：那条记录的两份镜像也读不出（再多两个故障）时水位退回读得出的根，前提六原样回来；甲 + 记录扫描水位叠加两边都关。

## 六、C330：U2

### 中间实例的单元能不能被所选根引用

- 实例代号按时间单调：取号 = max(独占打开集合里全部自证过的槽, 根环里全部根记录的实例代号) + 1，可写挂载要独占打开过半，同一时刻只有一个可写挂载（都在 D18 第 879 行）。实例 s 的每一个根都写在任何 i > s 的实例开始之前，根 (s, T) 引用不到 i 写的单元——除非有东西把 i 写的单元收进一个 s 的根里。
- 能收别的实例写的单元的只有两条路：重放（D23 第 1206 行那一节注 1，跨不过实例边界）与切换的「号 ≤ W 的事务照旧」（D23 第 691 行）。后者按 D23 第 1236 行「切换时的 W 取被重发的那个 checkpoint 里的最大事务号：开放 checkpoint 里已完成的事务全部 > W、按新写序重做」，被照旧的是被重发那个 checkpoint 的事务，那个 checkpoint 的成员资格在切分时已关闭（D16 第 405 行），成员都是所选根那个实例的事务。
- 撞号会打破单调；C322 的三处定案（全部自证过的槽、回卷号、屏障报错判全或无失败）之后按条文不再撞，多主机共享存储上的两个可写挂载按 D18 第 879 行「随 D9（加密） 已定项 8 那一类宣布不防」。
- 枚举：甲在 R_old 的表为底、M1 不带节点时 610000 条历史 U2 = 0（「五、C330：U1」那张表）。

### 连续两次切换：切换那句的 W 记到了谁名下

持久写集合：M1 写成 (1,1)(1,2)(1,3)；M2 选 (1,3)、写成 (2,4)(2,5)；发布中的 txg 6 两个码 1（事务 2、3）落了、固定点写失败 ⇒ 切到 3；开放的 txg 7 里事务 4 的码 1 已落。实例 3 重发 6：记账单元落了、另一个固定点写失败 ⇒ 再切到 4；这时实例 3 已把开放的事务重做成自己的事务 1（码 1 落了）。实例 4 重发 6（照旧事务 2、3）写成 (4,6)，开放的 7 按新写序重做后写成 (4,7)；M3、M4 普通恢复。第二次切换的所选根仍是 (2,5)——实例 3 一个根都没发布，D28 第 82 行「连续切换的前提是上一次的根没发布、重建态从同一个根来」。两种读法：`LiteralOldInstance` 让切换那句「写行 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号)」按字面给旧实例——第二次切换的旧实例是 3，一个中间实例——得 (3, 5, 3)，所选根那个实例 2 按 D18 第 879 行批量规则取「被这次重放施加的」= 0，得 (2, 5, 0)；`OwnerOfKeptTransactions` 把 W 给被照旧事务所属的实例，得 (2, 5, 3)、(3, 5, 0)。

`first-pass.out` 第 152 行：
```text
  M2'' switch 2->3->4 kept=[i2t6.publishing.data2(i2,b6,n2,Data,start3),i2t6.publishing.data3(i2,b6,n3,Data,start3)] rows_written_by_second_switch=(1,3,0)(2,5,0)(3,5,3) | mounted=(4,7) table=(1,3,0)(2,5,0)(3,5,3) U1=1 [i3t7.redo.data1(i3,b7,n1,Data,start5)] U2=2 [i2t6.publishing.data2(i2,b6,n2,Data,start3),i2t6.publishing.data3(i2,b6,n3,Data,start3)] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 157 行：
```text
  M2'' switch 2->3->4 kept=[i2t6.publishing.data2(i2,b6,n2,Data,start3),i2t6.publishing.data3(i2,b6,n3,Data,start3)] rows_written_by_second_switch=(1,3,0)(2,5,3)(3,5,0) | mounted=(4,7) table=(1,3,0)(2,5,3)(3,5,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 212 行：
```text
  M2'' switch 2->3->4 kept=[i2t6.publishing.data2(i2,b6,n2,Data,start3),i2t6.publishing.data3(i2,b6,n3,Data,start3)] rows_written_by_second_switch=(1,3,0)(2,5,0)(3,0,0) | mounted=(4,7) table=(1,3,0)(2,5,0)(3,0,0) U1=0 [] U2=2 [i2t6.publishing.data2(i2,b6,n2,Data,start3),i2t6.publishing.data3(i2,b6,n3,Data,start3)] corrupt=0 I-3.8_rows_not_below_mounted=[]
```
`first-pass.out` 第 217 行：
```text
  M2'' switch 2->3->4 kept=[i2t6.publishing.data2(i2,b6,n2,Data,start3),i2t6.publishing.data3(i2,b6,n3,Data,start3)] rows_written_by_second_switch=(1,3,0)(2,5,3)(3,0,0) | mounted=(4,7) table=(1,3,0)(2,5,3)(3,0,0) U1=0 [] U2=0 [] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 今天（字面读法）：实例 4 照旧的两个码 1（实例 2，诞生 6）被 (4,7) 引用，按 (2,5,0) 判未发布，U2 = 2；实例 3 重做过、又被实例 4 再重做掉的那个码 1（实例 3，n = 1）按 (3,5,3) 的 n ≤ 3 判已发布，U1 = 1。
- 甲（字面读法）：实例 3 那一行是 (3,0,0)，U1 消掉；U2 = 2 照旧，它在所选根那个实例的行上，甲不碰。换成「W 给被照旧事务的主人」，今天与甲都 0。
- 三问：U2 那一半不分辨臂（今天、甲、三种乙、丙在这一段都中，`first-pass.out` 第 150-154、270-274、330-334、390-394、450-454 行）；U1 那一半只有甲消掉。打中的是切换那句，不是甲。要补的一句：「连续切换时，写行按 [所选根的实例, 新实例) 写；被照旧事务所属的那个实例（所选根那个实例）取被重发那个 checkpoint 里它的最大事务号，其余实例——含前几次切换的实例——W = 0」，与 D18 第 879 行「同一次恢复 / 切换写出的那批行里，只有所选根那个实例的 W 可能非 0，其余恒 0」同向。

### 不分辨臂的一格：重放施加进来的码 2 / 3 单元

「四、C329：U3 与 U5」第二小节那段剧本在今天、甲、三种乙上逐字相同，U2 = 3。枚举里 M1 的 (1,3) 带一个节点、R_old 的表为底：

`attribution.out` 第 62 68 71 78 84 87 行：
```text
ENUM2 base=m1_with_nodes rollback_table_base=RollbackTargetTable rules=today histories=610000 evaluations=988735 u1_units=945569 u2_units=196509 histories_with_u1=279793 histories_with_u2=246267 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_with_nodes rollback_table_base=RollbackTargetTable rules=c330_jia histories=610000 evaluations=988735 u1_units=0 u2_units=197229 histories_with_u1=0 histories_with_u2=246267 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_with_nodes rollback_table_base=RollbackTargetTable rules=c330_yi_superblock histories=610000 evaluations=988735 u1_units=242440 u2_units=185449 histories_with_u1=81080 histories_with_u2=246267 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_with_nodes rollback_table_base=RollbackTargetTable rules=c330_yi_root_carried histories=610000 evaluations=988735 u1_units=945569 u2_units=196509 histories_with_u1=279793 histories_with_u2=246267 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_with_nodes rollback_table_base=RollbackTargetTable rules=c330_yi_record_scan histories=610000 evaluations=988735 u1_units=0 u2_units=185445 histories_with_u1=0 histories_with_u2=246267 evaluations_with_I-3.8_violation=0
ENUM2 base=m1_with_nodes rollback_table_base=RollbackTargetTable rules=c330_bing_start histories=610000 evaluations=988735 u1_units=0 u2_units=1863786 histories_with_u1=0 histories_with_u2=610000 evaluations_with_I-3.8_violation=0
```
`attribution.out` 第 69 70 行：
```text
ENUM2_EXAMPLE base=m1_with_nodes rollback_table_base=RollbackTargetTable rules=c330_jia kind=U2 faults=2
    M2 mode=recovery unreadable=[(1, 3)] selected=(1,2) after_replay=(1,3) W=1 acquired=2 rows_written=(1,2,1) first_txg=4 admission_short=false reach=RootNoSuperblock+None landed=[(2,4)] | mounted=(2,4) table=(1,2,1) U1=0 [] U2=1 [i1t3.node0(i1,b3,n0,Metadata,start0)] corrupt=0 I-3.8_rows_not_below_mounted=[]
```

- 今天、甲、三种乙都是 246267 条历史有 U2，丙 610000；把 M1 的节点拿掉之后（「五」那张表）除丙以外全 0 ⇒ 这 246267 条全是这一类。最短的一条 2 个故障：M2 那一刻 (1,3) 暂时读不出，选 (1,2)、把 (1,3) 从记录里重放回来，W = 1，写行 (1,2,1)；(1,3) 的节点（诞生 3）被挂载根引用、按 3 ≤ 2 为假判未发布。
- 条款：D18 第 879 行「码 2 / 3 ⇒ b ≤ T_pub（所选根之后的固定点单元一律未发布，恢复实例重写它们；…」对 D23 已定项 15 之前「重放重新生成祖先」的读法成立（E104 正文第 99 行的 `rewrite_inflight_containers`）；D23 第 694 行定下「施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义」之后，重建出来的根原样引用被重放那次发布写的码 2 / 3 单元，「恢复实例重写它们」成了一条没写出来的义务：写死它，写行那次的量随被重放的点名项走（「四」）；不写，就是被挂载根引用的节点判未发布，扫描重建当它是垃圾、清扫会抹掉它。另一条出路是所选根那个实例的行 T_pub 取重放之后那个根的 txg（`RowTxgAfterReplay`，U2 = 0、不多写单元，要改 D18 第 879 行「(i, **所选根的 checkpoint_txg**, …)」对所选根那个实例的取值）。建议并进 C284（施加一条记录在指针层上做什么没有定义）。

## 七、C330 甲的 U5：逐字

1. 与回退同形：D23 第 1206 行「写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)」；甲「[所选根的实例, 新实例) 里严格大于所选根实例的那些中间实例写 (i, 0, 0)，与回退写中间实例同形」。不冲突。
2. 与 I-3.8：`invariants.md` 第 127 行「行的实例代号 < 挂载根实例（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2）」；甲写的行都在 [所选根的实例, 新实例) 里，都小于挂载根实例；24 组枚举 I-3.8 零次红。不冲突。
3. 与切换那句在连续切换时说法不同：D23 第 691 行「写行 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号)」；甲「严格大于所选根实例的那些中间实例写 (i, 0, 0)」。第二次连续切换的旧实例是中间实例，两句给它两个不同的行；按「六」的剧本甲那一边对（字面那一边 U1 = 1）。要把切换那句的「旧实例」改成「所选根那个实例」，并补「六」末尾那一句。
4. 与回退候选集同向：D23 第 1206 行「(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）」。今天写 (i, T_sel, 0) 时，中间实例 txg ≤ T_sel 的根（前提六那种从更旧的根分出去、又没被选中的）按这句可选，它们不在所选根那条时间线上；甲写 (i, 0, 0) 之后一条都选不中（推理，枚举里回退目标固定取第二新的，没专门量这一格）。

更正（第二处）：「六、C330：U2」第一小节「那个 checkpoint 的成员资格在切分时已关闭（D16 第 405 行）」的行号错了，应为 D16 第 185 行（`grep -n "checkpoint 的切分只关闭成员资格" .claude/kb/decisions/16-发布语义.md`），405 同样是背景材料文件里的行号。原句逐字在附录。

## 八、U4 顺带：两笔欠账的检查怎么写才判得出

- C329 的检查（`checks-owed.md` 第 305 行）「逐状态数「本实例已有根落盘、而所选根之前非干净结束的实例没有行」的状态，必须为 0」：「非干净结束」在挂载时没有判别子（「三」换故障三），检查也就没法按字面实现。按收严之后的写法改成「本实例已有根落盘、而 [max(所选根的实例, 1), 本实例) 里有实例没有行」。判别力自证照原样（把写行挪到抬 F 那次空发布之后，计数由 0 转正）：`first-pass.out` 第 4 行就是转正的那个态（挂载根 (2,4) 的表空、实例 1 无行）。
- C330 的检查（`checks-owed.md` 第 306 行）「没被任何可达根引用的单元不许被已发布谓词判成已发布」：按字面会在合法镜像上红。每次发布都盖掉上一版记账单元，旧版诞生在已发布的 checkpoint 上、谓词判已发布，等那个根轮出环它就不被任何根引用（`first-pass.out` 第 119 行 `i2t4.accounting ... verdict=Published referenced=false on_timeline=true`）；同一个 checkpoint 里固定点迭代写两次的容器同形（D18 第 879 行「固定点会迭代、同一容器在一个 checkpoint 里不保证只写一次」）。模型的判法是「谓词判已发布 ∧ 诞生 checkpoint (实例, txg) 不在挂载根的时间线上 ∧ 没被引用」，时间线按根链与被施加的记录算。判别力自证照原样（写行给中间实例取所选根的 txg，检查必须红）：`first-pass.out` 第 118 行（今天）U1 = 3、第 178 行（甲）U1 = 0。
- 重放施加进来的码 2 / 3 单元那一格：「挂载根引用的码 2 / 3 单元，谓词必须判已发布」；判别力自证：T_pub 取重放之前的所选根（今天的字面）必须红——`first-pass.out` 第 162 行 U2 = 3，第 172 行（T_pub 取重放之后）0。
- 三条都要层 0 之外的多次挂载录制流。最短的历史：C330 回退形态 3 个故障（一条根暂时读不出 + 两次崩），甲那一类 5 个，择根倒挂 6 个，重放施加进来的节点 2 个。

## 九、按跑前写死的条款怎么判（攻方的读法）

- 失败条款：不触发。前提五（`first-pass.out` 第 4 行）与前提六（第 118 行）按今天的条文走得出来；背景材料里被转述的几句与 kb 逐字节相同（「一」第一行）。
- C329：甲在 U1、U2 上没被写出反例（前提五、回退、切换两种读法、之后连着的普通挂载都是 0）。打中甲的只有「非干净结束」按看着像干净判那一格，乙同中、丙变 U2，不分辨甲与乙。乙在切换那一格按「号 ≤ W 的单元只在内存里」读有 U1 = 2，丙在前提五就 U2。⇒ 反向接受条款第一句不触发；第二句「甲在 U1–U5 都不触发 ⇒ 按 U6 取甲」按字面也不触发，因为 U3、U5 各触发了。攻方的读法：取甲，同时收严四句——准入行给写行那次开例外、切换预留按 N_switch + 1 份算（或写行那次走准入但不许先推抬 F）、同一次发布里的用户数据重做照 D28 第 86 行走准入、删掉「非干净结束的」这个限定；重放施加进来的码 2 / 3 单元的重写量不是甲的账，并进 C284。
- C330：甲按最弱读法（回退那次发布的实例表以挂载时最新可读的根那一版为底）在 U1 上有反例（5 个故障），同一格乙的记录扫描水位不中 ⇒ 反向接受条款第一句按字面触发，指向不中的那条。但乙的两种字面落点（超级块水位、根记录带水位）在前提六上就中（`first-pass.out` 第 238、298 行），不中的只有记录扫描那一种；它是乙的收严（水位的取值集合比「超级块或根记录」多了全环记录），合「只许收严」，可它是这一轮攻方加的形态，没过正推腿。甲那一格的病根是一句没写的条款，补上（回退那次发布的实例表以 R_old 引用的那一版为底）甲在同一个枚举上 0。攻方的读法：判决在「甲 + 那一句」与「乙的记录扫描水位」之间选，两者各管一半——甲管行不管择根，择根倒挂下丢一次已确认的写（6 个故障）；记录扫描水位管 txg 不回退、两边都关，但那条记录的两份镜像也读不出时退回前提六（再多两个故障）；两个一起取，前提六与倒挂都关（推理）。丙：判别子观测不到，出局。
- U2：两个甲都没被写出反例。连续切换那一格与重放施加进来的节点那一格都不分辨臂，各记一句条款要补（「六」）。
- U4、U6 不归这条腿；U4 的顺带在「八」。

## 十、什么现象会推翻这些结论

- 有人写出一个可达的挂载序列：甲的次序下、写行那次的根落盘之后，[所选根的实例, 新实例) 里还有实例没有行，且不经「看着像干净就不写行」——C329 甲在 U1 上「没被打中」就推翻。
- 有人指出 kb 里已经写了回退那次发布的实例表以哪一版为底（我的 grep 只查了四个短语，「一」倒数第三行）——「五」里甲那一类 U1 就不是最弱读法的输。
- 在入库装置上重做枚举，甲 + R_old 的表为底出现 U1 或 U2 > 0——「甲在枚举里 0」推翻。
- 有人写出切换照旧与重放之外、能把中间实例的单元收进所选根那条时间线的第三条路，或指出 D23 第 1236 行「被重发的那个 checkpoint」可以含前一次切换实例的事务——「六」第一小节推翻，C330 甲的 U2 就可能有反例。
- 有人指出恢复的第一次发布按条文已经重写全部被重放施加进来的码 2 / 3 单元——那一格的 U2 就变成纯 U3 的量。
- 有人给出一个只看单元头与行就能分开 `i2t4.node0` 与 `i3t4.node0` 的谓词——两者的输入逐项相同（`first-pass.out` 第 120、123 行），丙「判别子观测不到」就推翻。

## 十一、复跑

```bash
B=/tmp/claude-1000/-home-fy5090-code-singlefs/72d37caa-c52c-4f5d-81fd-138d4e9f00d1/scratchpad/c329-r1-opus-build
mkdir -p "$B/model" && cp -r research/prompts/c329-c330-r1-opus-model/. "$B/model/"
cd "$B/model" && CARGO_TARGET_DIR="$B/target" cargo run --release > first-pass.out
cd "$B/model/attribution" && CARGO_TARGET_DIR="$B/target-attribution" cargo run --release --bin c329_c330_attribution > attribution.out
cd "$B/model/attribution" && CARGO_TARGET_DIR="$B/target-attribution" cargo run --release --bin inversion > inversion.out
```

- 本机 2026-09-14：第一遍 24.30 s，第二遍 93.35 s，倒挂入口不到 1 s；工具链 `cargo 1.98.0 (797e8a9bc 2026-08-05)`、`rustc 1.98.0 (88d9e12ae 2026-08-18)`。
- 偏差：第一遍实际是在 `$B` 下构建的（把 `Cargo.toml` 与 `src/main.rs` 拷过去、`CARGO_TARGET_DIR="$B/target"`），与复跑命令的 `$B/model` 等价。留存产物是从临时目录按 150 行一段拷进模型目录的，`cmp` 逐字节一致。
- 证据强度的口径：模型没有单测、没有变异表。判法自己会红的证明是同一个判法在今天的写行下红、在甲下绿（`first-pass.out` 第 4 与 32 行、第 118 与 178 行），不是断言；确定性模型，每组跑一遍，跑多遍只说明没有隐藏状态。数都在副本上，主 agent 在入库装置上重做才能引。

| 留存产物 | 行数 | sha256 |
|---|---|---|
| `first-pass.out` | 515 | `13f782a5d9269df185d51b7ed611565cc4457ab985db2d54b6e3d5dfc7b9fffa` |
| `attribution.out` | 150 | `74771bddedb652fc4b60444a0c40dbdbc0d3eeb2808fdd5fc16abeb1c0e9c5fc` |
| `inversion.out` | 24 | `a64b66e42fb71838a5fc51f26ce7a9667e0186dabf85091536e6a66e3397c93e` |

| 源码 | 行数 | sha256 |
|---|---|---|
| `src/main.rs` | 1063 | `d39a6b30348be0dfc858a82d76d9569d85b7d6dfefebdd55a60c2cb58f26f4de` |
| `attribution/build.rs` | 34 | `6e209151a1aa9e925b99e6c00bb9d6a1dcbae03978030015c0dc1d7d7518f075` |
| `attribution/src/main.rs` | 13 | `ae765d5081f42fdfd23a8a8630c32f34785be0b4fe1e4d605b4191c05aa3c988` |
| `attribution/src/attribution_items.rs` | 205 | `e9d3854dedfbcdb030494ebccf52caca6ce04fda46577f071cd53e1288c123e1` |
| `attribution/src/inversion_items.rs` | 55 | `b13bf90d2c82b14acae961fd17e4b750ee9eab083a5adbf5bff6c3aa0886ced7` |
| `attribution/src/bin/inversion.rs` | 12 | `fa0d428094902a15e6513f34c665c8e8a7d57214972957dee8200f34846f445c` |

## 附录：正文引到、背景材料附录没抄的原文（整行抄）

`.claude/kb/decisions/23-journal的角色与格式.md` 第 694 行：

~~~markdown
15. **崩在记录持久之后、根槽持久之前，恢复施加什么（2026-09-13，用户定案；两条指针 2026-09-14 随 D19（块指针的结构与宽度预算） 已定项 7 各加 3 字节）：由记录重建那次发布的根：记录头加「新根段」= 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 188 字节（实例表单元指针照所选根），头 95 → 277 → 307，4096 记录装 67 个点名项。施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义。已定项 14 条 2「严格大于所选根就施加」原样成立。** D23（journal 的角色与格式） 正文里没有已定项 15 的单独小节，条款全文就是已定项索引表第 15 行。 **状态：已定。**
~~~

`.claude/kb/decisions/23-journal的角色与格式.md` 第 1236 行：

~~~markdown
4. **切换时的 W 取被重发的那个 checkpoint 里的最大事务号**：开放 checkpoint 里已完成的事务全部 > W、按新写序重做。按第 2 条，它们的记录不在所选根之后；W 若取字面的「最后施加的事务号」，它们的单元会被谓词判成已发布而谁都不施加它们（C287（切换收养开放 checkpoint 的事务后再崩））。
~~~

`.claude/kb/decisions/22-单元原子性怎么合成.md` 第 487 行：

~~~markdown
| **实例代号** | **4** | D16（发布语义）已定项 6 骑手 1：与 D23（journal 的角色与格式）已定项 9 的 journal 实例代号**同一个计数**；择新在 checkpoint_txg 平局时按它高者赢（设备失而复得会造出两条同 txg 的合法根，checker 欠账见 [checks-owed.md](../checks-owed.md) C88（根环的时间线判别未实现）） |
~~~

`.claude/kb/decisions/05-快照-空间记账机制.md` 第 21 行：

~~~markdown
| `birth(b)` | 块被**发布**的那个 checkpoint 号 | 不是写请求发出时所在的开放 txg——崩溃后该号会被重发 |
~~~

`.claude/kb/decisions/16-发布语义.md` 第 290 行：

~~~markdown
⚠️ **超级块槽（2026-09-13 用户定案）**：根槽持久之后再更新超级块槽，每个 checkpoint 一次（C245（超级块槽的写频率，两条条款说反话） 的收口）；它是发布序列的最后一步、进层 0 枚举——E142（第一个事务的干跑） 的 t11 与第四次跑的 262162 个状态（只数事务那 21 条写时 65543）按这条算，一次发布的段序列因此唯一。
~~~

`.claude/kb/decisions/16-发布语义.md` 第 185 行：

~~~markdown
   **丢失窗口的口径改写（2026-09-05，随 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1；动的是 2026-08-31 的定案句，2026-09-13 复核收口）**：取号 = 事务被分配进某个 checkpoint 那一刻，checkpoint 的切分只关闭成员资格——触发之后不再分配事务进它，已分配的事务把各自的单元写完，发布阶段等全部单元落盘（任何 COW checkpoint 都要等最慢的那次 I/O，ZFS 的 txg sync 同样等最慢的 vdev，未在本项目验证）。事务切分纪律把一次写请求按单元切成若干事务各自取号，一个事务最多写一个单元的用户数据，**同一次请求切出的若干事务按其单元的 key 升序取号**（2026-09-13 用户定案：D3（空间分配） 已定项 8 那一轮查出乙的成段重写数全押在这一句上——请求内保持 key 序时 runs8 7270、请求内乱序时 8117；提交路径要记「同一请求内取号序与 key 序不一致的次数」、非 0 判红，落 C319（请求内单元按 key 升序发出没有条款也没有检查））；单元写的重试预算是墙钟 T_retry（具名可调参数，不是格式常量；用尽即按失败的性质走实例切换或转只读，D23（journal 的角色与格式） 已定项 14——它同时是丢失窗口的加项、挂载内实例切换的判据、以及连续切换计数的时间基准，三个用途一起送复核）。**由此崩溃最多丢的是 T_time + N_switch × (T_retry + T_redo)，不再恰是 5 秒**——一次挂载允许 N_switch 次实例切换（D23（journal 的角色与格式） 已定项 14），每次都要熬完 T_retry 再把在飞 checkpoint 重做一遍（T_redo），而重做期间开放的那个 checkpoint 发布不了、只能一直攒（第八轮反推腿 5.1）。**「上次 checkpoint 的时刻」在重发时取重发那一刻**（取原来那一刻的话，切换期间触发条件一直成立而在飞深度只有两个，触发只能空转）；E70（checkpoint 两个阈值的可行域） 给的下界不受影响；在飞 checkpoint 深度 = 开放 + 发布中两个；并发 fsync 的返回时刻被同 checkpoint 的其他事务拖住是组提交固有的代价。这是新规则 1「快照发布点钉在这个 checkpoint 全部数据变更之后」的可执行形态。
~~~

`.claude/kb/decisions/08-核心索引结构.md` 第 476 行：

~~~markdown
**② 树 ID 水位住根记录，取根环里全部根记录该字段的 max**（2026-09-06 用户定案；同日先定「住超级块取 max + 区段预留」，当日重比三条臂后重开，见变更史）。逐字形态： `新水位 = max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，**每次发布随根记录重写**。 ⚠️ **「累计」这两个字是承重的**：读成「本 checkpoint 发出的最高号」（不累计）时，树 ID 会随根轮出环被重发，而**那时被重发的号属于已发布的树**，D6（快照实现模型） 判据 9 正面命中 ⇒ 整条定案不成立。**依据**：**㈠ 回退不打穿它**——D23（journal 的角色与格式） 已定项 14 自己就在用这个装置给别的量取值：「第一个新根的 checkpoint_txg = **根环里全部根记录 txg 的最大值 + 1**」，是「全部」不是「候选」（候选集才排除被抛弃时间线的根）；同条逐字「**回退深度 ≤ 根环深度**」⇒ 能被回退跨过的根都还在环里。**㈡ 写时机天生成立**：根槽 FUA 就是发布本身，checkpoint N 里发布的树，其 ID 必 ≤ 根 N 里的水位——不需要任何预留机制。**㈢ 见证是根环 R = 3 份**，且 I-7.7（超级块实例代号不低于根环） 有现成的同形不变量可抄。**㈣ 每次发布零代价**：根记录本来就每次发布都写，字段表从 186 变 **194** 字节、512 槽余量 326 → 318（2026-09-06 的数；D22（单元原子性怎么合成） 已定项 7 今天是 242 / 余量 270）。 **被否的两条与它们的射程**：**乙 记账臂**——D23（journal 的角色与格式） 已定项 14 逐字「全部记账统计量的现行值从 R_old 那棵账重新载入」⇒ 回退后退回去重发。**「挂载时从树表扫最大值 + 1」（btrfs 那条路）**——被抛弃 checkpoint 里分配过的树 ID 不在已发布的树表里，不能单用。⚠️ **此前把根记录臂与记账臂一起用那句话否掉，是引用射程越位**：那条引用一个字也没说根记录。**这是 2026-09-06 当日重开这条定案的直接理由**，不是「丙更好」。 **丙-2 唯一盖不住的那一格，判为不是错误**：某个 checkpoint 分配了树 ID 42、写出带 42 的单元头，根没写成就崩了 ⇒ 水位随那个根一起丢 ⇒ 下次挂载重发 42，而盘上留着带 42 的孤儿单元。**判它不构成错误的依据（三方论证，主 agent 逐条现查）**：① D18（块里携带什么信息） 已定项 11 逐字「**作废一个 checkpoint 必须换实例代号**」⇒ 孤儿与活版的实例代号必不相等；② I-1.2（块头写序已发布） 逐字「**判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留**」；③ 扫描重建的两道闸（已发布谓词、择新键）与孤儿回收条件（D3（空间分配）「孤儿（没有条目）要求头里的写序实例 < 当前实例 ∧ 按实例表的已发布谓词判未发布」）**都不看树 ID**；④ **决定性的一条**：孤儿与活版**共用五元组本来就是常态**——崩溃前后同一个逻辑块两次写出的五元组逐字段相同（同树、同 inode、对象出生代在创建时固定、锚点偏移是文件内偏移），正是 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 给三类身份段统一加 10 字节写序要解决的东西⇒ 丙-2 只是把一个**已被专门解决**的碰撞从既有树扩到新建树；⑤ D6（快照实现模型） 判据 9 的理由句动词是「盘上遗留的旧 key 在新快照里**变成可见**」，而可见性作用在**树里的 key** 上，孤儿不在任何树里。⚠️ **由此要动 D6（快照实现模型） 判据 9 的射程**：它的字面「快照标识单调不回收」在孤儿场景下确实被违反，按它自己的理由句收窄成「**已发布**的快照标识单调不回收」（2026-09-06 用户定案，同一次）。 ⚠️ **inode 号水位仍要另给答案**：D5（快照 / 空间记账机制） 已定项 4 第 12 项逐字「**带**（只为可写头维护）」⇒ 每个可写头一个标量、条数随头数长，而根记录也是定长槽 ⇒ 丙-2 同样搬不过去。落点 C143（inode 号水位在回退后会退回去重发）。
~~~

`.claude/kb/checks-owed.md` 第 269 行：

~~~markdown
| C284 | 施加一条记录在指针层上做什么没有定义 | **D23（journal 的角色与格式） 已定项 14 的前缀五条口径按事务施加，前缀可以落在同一次发布的两个事务之间（E42（一事务几条记录） 那种切法）；而「施加一条记录」在指针层上具体做什么——换哪个指针、祖先谁来 COW、落点谁分配——全仓没有定义。** D23（journal 的角色与格式） 开篇「重放不要分配器」与「只施加一部分事务就要重新生成祖先」互相顶着；D13（验证路线）「核对一条记录只需格式解析 + 校验和 + 根环择新」管的是验记录，不是算期望态。记录核对器的范围与成本（D16（发布语义） 已定项 4）由它决定。⚠️ **空间侧的实例（2026-09-12，C126（切换预留的最坏量没有口径） 第一轮反推腿）**：重建态里「哪些落点算已分配」同样没定义——不登记则实例切换的重做把号 ≤ W 事务的数据当空闲覆写，全登记则记录点名的码 2 祖先被登记成已分配、却按已发布谓词要重写；C126（切换预留的最坏量没有口径） 的固定点那一项等它 ⚠️ **2026-09-13 用户定案（D16（发布语义） 已定项 4）：取「一次发布整体施加或整体不施加」，第二份 replay 停在句法层；条款已写，会红的检查仍欠。** ⚠️ 2026-09-13 总审核定案：D23（journal 的角色与格式） 已定项 15 定「施加一条记录 = 用记录头的新根段重建那次发布的根」，条款已写，会红的检查仍欠 | 定下施加在指针层上的语义，二选一：一次发布整体施加或整体不施加（记录核对器停在句法层）；或允许部分施加，并写清祖先与落点由谁定（记录核对器要自己实现树语义）。可判形式照 D13（验证路线） 已定项 1 的 M2：部分施加的结果等于某个整数个已提交事务的结果，且恢复路径不调用分配器；并写清施加一条记录时哪些落点登记进分配器（候选：码 1 登记、码 2 / 3 按已发布谓词未发布不登记），会红的形态：崩溃点重放里切换之后断言「重建态指针层引用的落点 ⊆ 分配器认为已分配的落点」 | D23（journal 的角色与格式） 补定义（纯论证工） | 2026-09-11 D16（发布语义） 三方论证反推腿 4-A，主 agent 核 D23（journal 的角色与格式） 已定项 14 五条口径与 E42（一事务几条记录） 坐实 建议列收窄：施加语义已定（整体施加或整体不施加），只欠会红的检查（切换后断言重建态引用 ⊆ 已分配）。 |
~~~

`.claude/kb/experiments/104-扫描重建的现行版本判定.md` 第 99 行：

~~~markdown
- 第五轮反推腿 2.2（固定点合写的容器写序取谁）：第六版让容器只按诞生代号与实例代号判（b ≤ T_pub），一个没发布的 checkpoint 的容器版本一律未发布，恢复实例重写它们（`rewrite_inflight_containers`）；旧版本是孤儿、按行判未发布、清扫清掉（单测 `inflight_containers_are_rewritten_by_recovery`）。
~~~


更正（第三处）：「四、C329：U3 与 U5」U5 第 2 条句末括号里那处自称，指的是同一节开头「切换预留被恢复自己先用掉一份」那一小节的算术（`first-pass.out` 第 475-476 行）。
