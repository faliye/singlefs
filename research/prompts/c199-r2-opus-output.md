# C199（实例代号递增与 jsn 断号即止互相矛盾） 第二轮三方论证：反推腿（Opus）

立场：假定写材料的人的倾向（乙′（链尾锚跨实例））是错的，去找推翻它的观测；同时攻甲′（严格前缀 + 新实例先暖机）。
材料：`research/prompts/_c199-r2-background.md` 正文 1–91 行；判定以附录 B 原文为准，需要时直接读仓里原文。
模型：纯计数模型，住在 scratchpad `c199-r2-opus/`，产物整份抄在报告末尾，带 md5 与复跑命令。
标注：「核过的」= 当场读了原文或跑了模型；「推的」= 没有当场核。模型没建的格子上的「没打中」不算证据。

## 一、材料读法与原文对不上的地方（按原文判）

| # | 材料那一行 | 原文逐字 | 影响 |
|---|---|---|---|
| E1 | P15 回退只列了「不施加 R_old 之后的任何记录、取新代号、写回退行……之前没有持久效果」 | D23（journal 的角色与格式） 已定项 14「显式例外」段还有一句：「defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入」；I-7.4（近 K 代块未被复用） 逐字「被抛弃时间线的根不算，它们的独占单元可以被清扫抹头」 | 承重。两句合起来，回退那次发布自己的 COW 单元就可以落在被抛弃时间线独占的单元上——那次发布「之前没有持久效果」不成立（第三节 Y-R，零故障，三条臂都中）。核过的 |
| E2 | P7 切换只写「取新代号、写行、重发在飞 checkpoint，号 ≤ W 照旧、号 > W 重做或报错」 | D23（journal 的角色与格式） 索引第 14 项：「写行 (旧实例, 最后发布的 txg, 最后施加的事务号)、重发在飞 checkpoint……固定点单元全部按新实例重写」；同一项还有「根槽写失败重发时 checkpoint_txg 推进一格再发」 | 后一句管甲′ 暖机的空发布写失败（第四节）。核过的 |
| E3 | 乙′ 规则 ①「链首 = 所选根覆盖的最后一条记录的下一条」，材料把比较序只挂在 C287（切换收养开放 checkpoint 的事务后再崩） 上 | D22（单元原子性怎么合成） 已定项 7 根记录字段表里没有任何 journal 位置字段（实例代号 4、checkpoint_txg 8、两个 83 宽指针、自证校验和 32……），那一节逐字「恢复只施加 `(实例代号, checkpoint_txg)` 严格大于根的记录；tail 只是扫描起点的优化」 | 「覆盖」只能按水位比出来，所以 P14 的比较序直接决定乙′ 与甲的链首在哪；不是只有 C287 那一格才用到它。要按位置定链首就得给根记录加字段（判据 4）。核过的 |
| E4 | Sonnet 产出 4.2 把 C287 的新根写成 (j, T+1)，据此说两种比较序给同一个答案 | C287 那一行逐字「T+1 发布之前再崩：所选根是新实例写的 T」 | 按原文，a 的记录水位 (i, T+1) 对根 (j, T)：txg 为主 ⇒ 记录在根之后；实例代号为主 ⇒ 记录在根之前。两种序答案相反（第五节）。核过的 |
| E5 | P6「施加前逐项验证点名单元」，没说验不过怎么办 | D16（发布语义） 已定项 7 只写「必须逐项验证点名单元的校验和」；E77（发布的持久顺序） 源码 `e77_publish_order.rs:202` 逐字「施加前逐项验证点名单元；任何一项失配 ⇒ 整个事务丢弃（回旧态）」——那是单事务模型 | 一条链中间有一个事务被丢弃之后，后面的记录停还是接着施加，仓里没写。第三节 Y-B 两种都跑了 |

## 二、模型（scratchpad `c199-r2-opus/`，纯计数，零文件 I/O）

口径与第一轮一样：一条已提交的记录 = 那一刻的完整目标态（文件 → 单元）。这一轮加了三样第一轮没有的东西：单元有物理落点、可以被复用（被覆盖之后写序变了）；实例表的行与已发布谓词（码 1 那一支 `b ≤ T_pub ∨ n ≤ W`，n 取事务号）；扫描重建的现行版本（已发布的单元里写序最大者），与树里的那一版对不上就记「复活」。

| 参数 | 取值 | 对应什么 |
|---|---|---|
| head | after_last_covered / first_uncovered | 乙′ ① 字面（链首 = 按水位判覆盖的最后一条的下一条）/ D23（journal 的角色与格式） 已定项 14 字面（只施加水位严格大于根的记录） |
| order | inst / txg | 记录水位按实例代号为主 / 按 txg 为主 |
| verify_fail | stop / skip | 点名单元验不过：链停 / 丢这个事务、接着施加（第一节 E5：仓里没写） |
| protect_rb_intermediate | False / True | 回退写的中间实例行 (i, 0, 0) 带不带保护位（乙′ ⑤「中间实例行与回退行一样置不可覆盖位」的弱读 / 强读） |
| protect_abandoned_in_ring | False / True | 修法 F-A：被抛弃的根还在根环里时，它引用的单元不许复用 |
| w_reading | literal / narrow | C287（切换收养开放 checkpoint 的事务后再崩） 的两种 W |

**建了的格**：C0、C1（对照）、Y-A（回退实例的根全读不出，恢复选中被抛弃的根，乙′ 跨到回退记录）、Y-R（回退那次发布的 COW 单元落在被抛弃时间线独占的单元上，崩在根之前）、Y-B（被抛弃时间线有一条已提交、没进根的尾巴，它点名的单元被回退实例复用，然后掉 0 号盘）、Y-C（回退确认之后被抛弃的根引用的单元被复用，再让回退实例的根全读不出）、C287（逐字按那一行造）、R-TXG（回退新根取「根环最大 + 1」，被抛弃实例的开放 checkpoint 里有 txg 更大的已提交记录）。

**没建的**（这些格上的「没打中」不算证据）：环回绕与 tail、撕裂的根槽、跨多条记录的事务、码 2 / 3 的重写与固定点重写范围、行回收、N_switch 次连续切换、E51（反向链的碰撞机会有多少次） 的碰撞概率、记录作为增量而不是完整目标态、数据单元在两盘上的冗余（假定掉一块盘不丢单元）、回退从哪一版实例表起算。

**甲′ 在模型里怎么做**：`Writer.warm_up` 在接受写之前连推空发布（每次先写一条 empty 记录，D16（发布语义） 连带 1 不许绕过 journal 发根），直到本实例写过的根区域覆盖两块盘；覆盖之前的发布一概不确认。

## 三、攻乙′（第 2 问）

几何按 P10（R = 3、每区 8 槽、区域 = txg mod 3、区域 0 / 2 在 0 号盘、区域 1 在 1 号盘）。下面的序列都是乙′ 那条臂在模型里的真实走法；产物行整行抄，文件 `c199r2_model.out`。

### 3.1 Y-A：跨实例重放之后写的行，把被回退丢掉的写判成已发布（判据 1；核过的：条文 + 模型）

1. 实例 1：txg 1 发 F1（区域 1，1 号盘），txg 2 发 G1（区域 2，0 号盘）。R_old = (1, 2)。干净卸载。
2. 实例 2 干净重挂，不写行（D18（块里携带什么信息） 已定项 11「只在恢复与回退时写」「任何非干净结束的实例在下次恢复写行」）：txg 3 发 F2（0 号盘），txg 4 发 H2（1 号盘）。干净卸载。
3. 管理员回退到 (1, 2)：实例 3 写回退行 (1, 2, 0) 置 bit0、中间实例行 (2, 0, 0)（D18（块里携带什么信息） 已定项 11 逐字「回退行在 kind 0 记录的 flags 字节里置 bit0 标「回退」、不许被后来的恢复覆盖」——只有回退行带这一位）、回退记录、新根 (3, 5)（= 根环最大 4 + 1，区域 2，0 号盘），向管理员确认。
4. (3, 5) 那个槽读不出（1 个故障），崩。
5. 恢复实例 4：最新可读的根是 (2, 4)——被抛弃的根，它指向的表里没有实例 2 的行（P17）。乙′ ①：H2 的记录是 (2, 4) 覆盖的最后一条，链首 = 回退记录（代号 3 > 2、计数器 +1）⇒ 施加，状态 = R_old + 回退，这一半是对的。
6. 乙′ ⑤ 给 [2, 4) 写 (j, 所选根 txg, W_j)：实例 2 那一行 (2, 0, 0) 没有保护位 ⇒ 被写成 (2, 4, 0)。
7. 已发布谓词逐字「码 1 ⇒ b ≤ T_pub ∨ n ≤ W」：F2 的单元写序 (2, 1)、b = 3 ≤ 4 ⇒ 已发布；H2 同。扫描重建按写序最大挑 F 的现行版本 ⇒ F2，树里是 F1。

```
cell=Y-A arm=yi_prime params=default faults=1['slot(2, 1)'] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=['F:scan=F2/tree=F1', 'H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-A arm=yi_prime params=protect_rb_intermediate=True faults=1['slot(2, 1)'] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
```

判定：**打中乙′ 的最弱读法**（⑤「中间实例行与回退行一样置不可覆盖位」读成只管 ⑤ 自己写的中间行）。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「记一次输 → 只许收严 → 写明收严在哪」：收严成「回退写的 (i, 0, 0) 也置不可覆盖位」，多要求一样东西、没少要求任何东西，模型转绿（第二行）。
⚠️ 收严之后 ⑤ 的公式本身仍不对：给一个已被回退整段抛弃的实例写 `(j, 所选根 txg, ·)`，T 取所选根 txg 就是在说「它 b ≤ 4 的单元都发布了」。这一格它只是恰好被保护位挡住；回退从哪一版实例表起算（R_old 的表，还是当前的表）仓里没写——从 R_old 的表起算而那一行不在，⑤ 照样写出 (2, 4, 0)。**推的**。⇒ ⑤ 要写成「T 取被跨过的实例在终态里仍然生效的最后一个 txg，被回退抛弃的实例取 0」。

### 3.2 Y-B：被抛弃尾巴点名的单元被复用，乙′ 单故障丢已确认的回退与 fsync 已返回的 X（判据 2；模型核过，验不过的语义是推的）

1–2. 同 3.1；实例 2 发完 H2 之后，事务 T2 的记录（txg 5）带齐提交标记持久了，根没写成就崩——被抛弃时间线里没进根的那一截。
3. 回退到 (1, 2)，新根 (3, 5)（0 号盘），确认。回退记录按 P8「恢复之后从「前缀末 + 1」接着写」接在 T2 后面。
4. 实例 3 的分配器从 R_old 的账重新载入（第一节 E1）；T2 的单元 u_T 在那本账里是空闲的，按 (2, 0, 0) 判未发布，也不被任何候选根引用 ⇒ I-7.4（近 K 代块未被复用） 不保护它。实例 3 把 X 写进 u_T，发 (3, 6)（区域 0，0 号盘），fsync 返回。
5. 掉 0 号盘（1 个故障；journal 两盘镜像，还在）。1 号盘上的根：(1, 1)、(2, 4) ⇒ 选 (2, 4)。
6. 乙′：链首 = T2 的记录；施加前逐项验证点名单元（D16（发布语义） 已定项 7）⇒ u_T 现在是 X，验不过。

```
cell=Y-B arm=yi_prime params=protect_rb_intermediate=True faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-B arm=yi_prime params=protect_rb_intermediate=True,verify_fail=skip faults=1['disk0'] root=(2, 4) applied=['RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=['verify-fail:T2'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-B arm=yi_prime params=protect_rb_intermediate=True,reuse_tail=False faults=1['disk0'] root=(2, 4) applied=['T2', 'RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=['T2'] verdict=RED:abandoned_applied
```

- **验不过即停**（第一行）：丢已确认的回退与 X，1 个故障；盘上足以保住它们的东西都在——可读的根 (2, 4)、完整的记录 RB 与 X3、R_old 的单元（候选根，I-7.4 保护）。按判据 2 ⇒ **乙′ 出局**。
- **验不过就丢这个事务、接着往后施加**（第二行）才绿，而它绿只是因为下一条恰好是回退记录（完整目标态）。E77（发布的持久顺序） 源码 `e77_publish_order.rs:202` 逐字「任何一项失配 ⇒ 整个事务丢弃（回旧态）」是单事务模型；记录一旦是增量，丢掉一个事务再往后施加就是把后面的目标态放在错的基底上（判据 1）。所以实现只能停。**推的**（模型不建增量记录）。
- **u_T 没被复用**（第三行）：终态对，但施加了 T2——判据 1 字面那一格「被抛弃时间线里没有提交的那一截」。它是不是判据 1 要问的那种错，交判决。
- 同一格甲′ 单故障绿（`cell=Y-B arm=jia_prime params=default ... verdict=GREEN`），甲红（`cell=Y-B arm=jia params=default ... verdict=RED:lost_acked+wrong_state`）。
- 乙′ 的出路只有两条：F-B「被抛弃时间线里仍在 journal 环里的已提交记录，它们点名的单元不许复用」；或者让乙′ 在链上遇到回退记录时，它抛弃的那一段不施加也不验证——后者改的是臂的规则，按规矩是新臂。

### 3.3 Y-R：回退那次发布自己的 COW 单元落在被抛弃的根引用的单元上（判据 1 / 5′；零故障；三条臂都中；条文核过，分配器落不落在那一格是推的）

1–2. 同 3.1。
3. 管理员回退到 (1, 2)。发布顺序是「COW 单元/节点 → 屏障 → journal 记录 → 屏障 → 根槽」（D16（发布语义） 已定项 7），所以回退先写它的 COW 单元（新的实例表单元）。分配器「从 R_old 那棵账重新载入」（D23（journal 的角色与格式） 已定项 14），F2 的单元在那本账里是空闲的；I-7.4（近 K 代块未被复用） 逐字「被抛弃时间线的根不算，它们的独占单元可以被清扫抹头」⇒ 没有东西拦它落在 F2 的单元上。游标从 R_old 载入，而实例 2 当初正是从同一个游标往后分配的，所以落在那里是大概率，不是边角（推的）。
4. 单元写完、记录之前崩（stop_at=units），零故障。
5. 下一次挂载是普通恢复：回退的意图没有任何持久痕迹（「之前没有持久效果」），「崩了就重做」没有东西触发。最新可读的根是 (2, 4)，它的 F 指向的单元已经是回退的实例表。

```
cell=Y-R arm=jia params=stop_at=units faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=jia_prime params=stop_at=units faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=yi_prime params=stop_at=units faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=jia params=stop_at=record faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=jia_prime params=stop_at=record faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@10'] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=yi_prime params=stop_at=record faults=0[] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=['H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-R arm=jia params=protect_abandoned_in_ring=True,stop_at=units faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=jia_prime params=protect_abandoned_in_ring=True,stop_at=units faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=yi_prime params=protect_abandoned_in_ring=True,stop_at=units faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=jia params=protect_abandoned_in_ring=True,stop_at=record faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=jia_prime params=protect_abandoned_in_ring=True,stop_at=record faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@10'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=yi_prime params=protect_abandoned_in_ring=True,stop_at=record faults=0[] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=['F:scan=F2/tree=F1', 'H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-R arm=jia params=protect_abandoned_in_ring=True,protect_rb_intermediate=True,stop_at=record faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=jia_prime params=protect_abandoned_in_ring=True,protect_rb_intermediate=True,stop_at=record faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@10'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=yi_prime params=protect_abandoned_in_ring=True,protect_rb_intermediate=True,stop_at=record faults=0[] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
```

判定：一次**没确认**的回退，它的根从没持久，却已经把旧状态写坏了——判据 5′「构造得出它部分生效」，同时是判据 1「读到已被复用的单元」。它直接证伪 D23（journal 的角色与格式） 已定项 14 那句「回退与它的第一个新根同一次发布，之前没有持久效果」。三条臂都中（stop_at=units）；崩在记录之后（stop_at=record）时乙′ 按规则 ④ 接上回退记录，复用无害，甲 / 甲′ 照样中。乙′ 在 `stop_at=record,protect_abandoned_in_ring=True` 那一行的红是 3.1 的行问题，带上收严（`protect_rb_intermediate=True`）就绿。
**公共修法 F-A**：I-7.4（近 K 代块未被复用） 的「被抛弃时间线的根不算」改成「被抛弃的根离开根环之后才不算」——它们还在环里时，它们引用的单元照样不许复用、不许抹头。代价是被抛弃时间线的空间最多晚一圈根环（24 次发布）才能回收。改的是用户定案条款的一句（需用户知情）。

### 3.4 Y-C：回退确认之后复用被抛弃的根引用的单元，再让回退实例的根全部读不出（判据 1 不看故障数；模型核过）

回退确认之后，实例 3 按 I-7.4 把 X 写进 F2 的单元。甲 / 乙′ 一个故障（0 号盘），甲′ 两个故障（0 号盘 + 回退实例在 1 号盘上的根）让回退实例的根全部读不出，恢复选中被抛弃的根。

```
cell=Y-C arm=jia params=default faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=['RB', 'X3'] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:lost_acked+wrong_state+reads_reused+resurrect
cell=Y-C arm=jia_prime params=default faults=2['disk0', 'slot(1, 4)'] root=(2, 10) applied=[] final=[F=F2,G=G1,H=H2,I1=I1] notes=['stop:instance-boundary@11'] lost_acked=['RB', 'X3'] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:lost_acked+wrong_state+reads_reused+resurrect
cell=Y-C arm=yi_prime params=default faults=1['disk0'] root=(2, 4) applied=['RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=['H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-C arm=jia params=protect_abandoned_in_ring=True faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-C arm=jia_prime params=protect_abandoned_in_ring=True faults=2['disk0', 'slot(1, 4)'] root=(2, 10) applied=[] final=[F=F2,G=G1,H=H2,I1=I1] notes=['stop:instance-boundary@11'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-C arm=yi_prime params=protect_abandoned_in_ring=True faults=1['disk0'] root=(2, 4) applied=['RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=['F:scan=F2/tree=F1', 'H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-C arm=jia params=protect_abandoned_in_ring=True,protect_rb_intermediate=True faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-C arm=jia_prime params=protect_abandoned_in_ring=True,protect_rb_intermediate=True faults=2['disk0', 'slot(1, 4)'] root=(2, 10) applied=[] final=[F=F2,G=G1,H=H2,I1=I1] notes=['stop:instance-boundary@11'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-C arm=yi_prime params=protect_abandoned_in_ring=True,protect_rb_intermediate=True faults=1['disk0'] root=(2, 4) applied=['RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
```

- 甲、甲′ 都读到被复用的单元：判据 1 不看故障数 ⇒ **按字面甲′ 也在判据 1 上出局**，与 3.3 同一个根：I-7.4 放开了被抛弃的根，而 P17 让恢复分辨不出它被抛弃了。F-A 之后甲′ 只剩「丢」（两个故障，判据 2 记「丢」），甲是一个故障的「丢」。
- 乙′ 跨到回退记录，不读被抛弃的根的树；它剩下的只有 3.1 的行问题（`resurrect=['H:scan=H2/tree=None']`），F-A 加收严之后绿。

### 3.5 C287（切换收养开放 checkpoint 的事务后再崩）

两种比较序与两种 W 的全表在第五节。对乙′ 的结论：链首按水位定（第一节 E3），在「字面 W」下三条臂都复活 a 写的 X，与比较序无关；「窄读 W」下三条臂都绿。C287 不分辨三条臂。

### 3.6 没建模的格（推的；这些格上的「没打中」不算证据）

| 格 | 试过的构造与推理 | 结论 |
|---|---|---|
| N_switch 次连续切换 | 1 → 2 → 3 → 4 连切三次，最后那个实例的根读不出，乙′ 一次重放跨三个边界。D23（journal 的角色与格式） 已定项 7「号更大的记录一条都不追加」⇒ 每个切换点之后没有旧实例的记录，链是线性的；⑤ 的 W_j = 被施加的最大号，码 1 谓词与重放一致 | 没打中 |
| 固定点单元的重写范围 | ⑤ 的范围「所选根之后、重放后仍被引用的全部码 2 / 3 单元」横跨每个被跨过、根读不出的实例的每一个 checkpoint；D23（journal 的角色与格式） 索引第 14 项写切换的最坏量时逐字「固定点重做与号 > W 的数据单元重写不另要空间……重做走 checkpoint 的保留池」，那个池是按一次切换定的 | 判据 6 的一条交互：乙′ 的恢复可能在保留池里 ENOSPC，恢复做不完转只读。不是判据 1 |
| 中间实例行与后来的回退 | ⑤ 收严之后所有被跨过的行都受保护。之后管理员回退到被跨过实例 i 的一个旧根 (i, T′)（候选集按行 (i, T_sel, W_i) 判 T′ ≤ T_sel 可选），它要写 (i, T′, 0)，而 (i, T_sel, W_i) 不许被覆盖 ⇒ i 在 (T′, T_sel] 诞生的单元回退之后仍判已发布 | 取决于回退从 R_old 的表起算还是从当前的表起算——仓里没写。今天对同一个 r_old 连退两次也是同一个问题 |
| 回退行与五条口径第五条 | 乙′ ③ 删掉第五条。C124（回退行与重放下界没有会红的检查） 那条历史（恢复落回 R_old）：乙′ 从 R_old 穿过被抛弃的记录接到回退记录再整体复位，终态对；被抛弃记录点名的单元被复用、验不过即停时同 Y-B。甲 / 甲′ 在这一格把被抛弃时间线整段重放回来（第五条没有输入，第一轮 O-8），故障要很多，但判据 1 不看故障数 | 三条臂都受影响，乙′ 最轻 |
| E51（反向链的碰撞机会有多少次） 的接缝数 | 乙′ 只在下一条代号更大、计数器 = 前一条 + 1 时跨。每次恢复 / 切换 / 回退都取更大的新代号，切换点之后没有旧实例的记录，所以链尾之后的残留代号一定更小，比到代号就拒收，轮不到 previous_hash；跨实例的那一环是新写者从真实的前一条算出来的，按 E51（反向链的碰撞机会有多少次）「同源比较不构成碰撞机会」不算机会 | 乙′ 不加接缝。例外是代号重复（多主机共享存储，已宣布不防）与镜像两份不一致 |

## 四、攻甲′（第 3 问）

### 4.1 暖机做完之前崩（推的，没单独建格）

暖机做完之前什么都没确认，判据 2 不适用。新实例已经写成的根要么被选中（同一实例，链接得上），要么读不出、退回旧实例的根（旧实例的已确认发布由它自己两块盘上的根罩着）。回退那一种：回退的第一个根在暖机之前就持久了，暖机中崩 ⇒ 回退整体生效、没确认，判据 5′ 允许；暖机中那个根读不出再崩 ⇒ 回退整体不生效，也允许——**前提是回退那次发布没有写坏被抛弃的根**，而这正是 3.3 Y-R 打中的那一格，暖机救不了它（它发生在第一个根之前）。

### 4.2 暖机的空发布自己写失败（条文核过，后果是推的）

D23（journal 的角色与格式） 索引第 14 项逐字「根槽写失败重发时 checkpoint_txg 推进一格再发」，失败的处置先做探针写：写得进去 ⇒ 实例切换，写不进去 ⇒ 转只读。所以一次空发布写失败要么结束这个实例（切换，新实例从零暖机），要么只读；失败的那次写不会算进一个还活着的实例的覆盖。⇒ 没有单故障洞。代价：暖机失败都变成切换，N_switch = 3 用得更快——1 号盘根区域瞬时写失败三次，挂载就转只读；今天同样的盘可以可写挂载，只是 fsync 是单点。这是可用性，不是正确性。
⚠️ 材料写的是「本实例写过的区域覆盖两块盘」，要写成「本实例写成（FUA 返回）的根覆盖两块盘」。按切换规则字面读法无害，但这句是暖机规则的判据，不许留一个能读成「发出过写就算」的词。

### 4.3 区域到盘的映射换几何（推的）

一般几何下最坏次数 = 同一块盘在轮转序列里连续占的最长游程 + 1（Sonnet 产出 3.2 已算，不重复）。补一格：多盘池里 mkfs 把 R 个根区域全写在同一块盘上（D2（RAID 条带策略） 已定项 7 让 mkfs 逐区域写归属；I-7.6（根环区域落在互不相同的盘上） 只在 devs ≥ R 时成立，第一版 R = 3、devs = 2 就不成立），「两块不同的盘」永远达不到，fsync 永不返回。甲′ 的单盘退路「至少两个区域」要改成「根区域所在的盘只有一块时，至少两个区域」。

### 4.4 两个故障（模型核过）

```
cell=C0 arm=jia_prime params=default faults=1['slot'] root=(1, 4) applied=['X'] final=[fA=A,fB=B,fX=X] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-A arm=jia_prime params=default faults=2['disk0', 'slot(1, 3)'] root=(2, 7) applied=['F2', 'H2'] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@10'] lost_acked=['RB'] reads_reused=[] resurrect=[] abandoned_applied=['F2', 'H2'] verdict=RED:lost_acked+wrong_state+abandoned_applied
cell=Y-B arm=jia_prime params=default faults=1['disk0'] root=(3, 10) applied=['empty', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-B arm=jia_prime params=yb_second_fault=True faults=2['disk0', 'slot(1, 3)'] root=(2, 7) applied=['F2', 'H2'] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=['F2', 'H2'] verdict=RED:lost_acked+wrong_state+abandoned_applied
```

单故障（C0 一个槽、Y-B 掉 0 号盘）甲′ 都绿；要丢已确认的回退与 X 要两个故障（0 号盘 + 回退实例在 1 号盘上的最新根），判据 2 记「丢，两个故障」，交判决比较。Y-C 的两个故障在今天的 I-7.4 下还多一条「读到被复用的单元」（3.4），F-A 之后才只剩「丢」。

### 4.5 与 D16（发布语义） 已定项 6 / 7 撞不撞（条文核过）

- 已定项 6 逐字「每次发布把 checkpoint_txg 加一——fsync 触发的发布也是发布，没有「小发布不记号」的例外」：空发布各加一，轮转键就是 txg，暖机正好靠它换区域。不撞。
- 连带 1「没有任何操作可以绕过 journal 直接发根」：每次空发布先写一条记录，模型里就是这么做的。不撞。
- 已定项 7 逐字「fsync 等根槽持久之后才返回」：甲′ 在它上面多加一个条件，是收严，改用户定案的一句（需用户知情）。第一个事务的字节随之变：mkfs 之后第一次挂载先推两次空发布，第一个事务的 checkpoint_txg 从 1 变 3（Sonnet 产出 3.1），判据 4 记「变」。

### 4.6 单故障：试过、没打中的构造（一次抽样）

| 构造 | 为什么没打中 |
|---|---|
| M1 / M2 / M3 / M6 的类比：新实例（首次挂载、干净重挂、恢复）暖机之后 fsync，再坏一个槽或掉一块盘 | 暖机后本实例在两块盘上都有根，单故障剩下一个同实例的根，链在实例内接到那条记录。模型 C0、Y-B 单故障绿 |
| M7 的类比：挂载内切换后「号 ≤ W 照旧」的事务 | 它们的 fsync 本来就要等新实例的根；新实例暖机之前一个都不返回（推的，没建） |
| 回退确认后单故障 | 回退要等回退实例的根覆盖两块盘才确认。模型 Y-A 单故障下甲′ 那一格要两个故障才丢 |
| 暖机中崩、暖机空发布写失败 | 4.1、4.2 |

⇒ 甲′ 在判据 2 上**没有单故障打中序列**（模型只建了上表几格，别的格上的「没打中」不算证据）。甲′ 在判据 1 上被打中的两格（3.3 零故障、3.4 两个故障）甲同样中，根在 I-7.4 与 D23（journal 的角色与格式） 已定项 14 的分配器重新载入，F-A 一并修掉。

## 五、两件先写死的（第 4 问）

### 5.1 计数器：全池接着走（条文核过；反例是推的）

原文两处：D23（journal 的角色与格式） 已定项 9 依据表逐字「计数器 48 位 | 记录数就是 fsync 数……本机 fsync 率 2785 次/秒……⇒ 48 位撑 3202 年」；E37（日志实例代号） 口径一节逐字「计数器 48 位 ⇒ 每实例 2.8×10¹⁴ 条记录。⚠️ 这两个数是E37（日志实例代号）取的划分，不是设计定案」。⇒ P13 的读法对：「每实例」那句是 E37 的划分，3202 年是按全卷寿命、计数器不归零算的。

**按实例从头计的反例**：D23（journal 的角色与格式） 已定项 8 逐字「定长环的槽位由 `jsn` 决定」，P8「恢复之后从「前缀末 + 1」接着写」。计数器归零时新实例的第一条是 (j, 1)，它落在哪个槽由 jsn 定，不由旧前缀在哪里结束定——两句当场冲突。序列：实例 i 的最新几条记录绕回到环的低位槽，X 的记录在槽 s、被根 (i, T) 覆盖、fsync 已返回；干净卸载；实例 j 从 (j, 1) 起写，槽区间盖到 s；j 的第一个根与 (i, T) 都读不出（两个故障）⇒ 退到 (i, T−1)，要 X 那条记录，没了。全池接着走时 j 写在前缀末之后，X 的记录还在，甲从 (i, T−1) 在实例 i 内接到 X。⇒ 从头计把一个两故障下保得住的格子变成丢，而且要改 D23（journal 的角色与格式） 已定项 8 的前提（判据 3）。
**全池接着走**：没找到反例（试过：48 位在 2785 条/秒下 3202 年；按 E42（一事务几条记录） 每事务 8 条算约 400 年，2⁴⁸ ÷ (2785 × 8) 秒；残留的计数器可以与新写的相同，但代号更小，比到代号就拒收）。乙′ 必须用它（规则 ②），甲 / 甲′ 也该用它。连带把「每实例 2.8×10¹⁴」那句改成全池预算。

### 5.2 记录水位：实例代号为主（模型核过）

**结构上的理由**：jsn = 实例代号 32 位 + 计数器 48 位，按 jsn 排就是实例代号为主。journal 上实例代号只增不减（每次恢复、切换、回退都取 max + 1，写在前缀末之后），同一实例内 txg 沿 journal 不减 ⇒ 实例代号为主时，水位沿 journal 单调，「被根覆盖」的记录恒是 journal 的一个前缀，「链首 = 覆盖的最后一条的下一条」（乙′ ①、甲）与「只施加严格大于根的记录」（D23（journal 的角色与格式） 已定项 14 字面）选出同一段。txg 为主时有两处不单调：C287——旧实例把 a 追加进 T+1 之后，新实例才重发 T；R-TXG——回退的新根取「根环最大 + 1」，不看没发布的 checkpoint，而旧实例的开放 checkpoint 里已有 txg 更大的记录。两种读法在这里分道。

乙′ 那一条臂的产物行（甲、甲′ 判得一样，全表在附录）：

```
cell=C287 arm=yi_prime params=head=after_last_covered,order=inst,w_reading=literal faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=yi_prime params=head=after_last_covered,order=txg,w_reading=literal faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=yi_prime params=head=first_uncovered,order=inst,w_reading=literal faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=yi_prime params=head=first_uncovered,order=txg,w_reading=literal faults=0[] root=(2, 3) applied=['a1'] final=[X=a1,fA=A,fB=B,fP=P1] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=yi_prime params=head=after_last_covered,order=inst,w_reading=narrow faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=yi_prime params=head=after_last_covered,order=txg,w_reading=narrow faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=yi_prime params=head=first_uncovered,order=inst,w_reading=narrow faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=yi_prime params=head=first_uncovered,order=txg,w_reading=narrow faults=0[] root=(2, 3) applied=['a1'] final=[X=a1,fA=A,fB=B,fP=P1] resurrect=['X:scan=None/tree=a1'] abandoned_applied=[] verdict=RED:resurrect
cell=R-TXG arm=yi_prime params=head=after_last_covered,order=inst faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=yi_prime params=head=after_last_covered,order=txg faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=yi_prime params=head=first_uncovered,order=inst faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=yi_prime params=head=first_uncovered,order=txg faults=0[] root=(2, 3) applied=['z1'] final=[Q=q1,Z=z1,fA=A,fB=B] resurrect=['Q:scan=None/tree=q1', 'Z:scan=None/tree=z1', 'fB:scan=None/tree=B'] abandoned_applied=['z1'] verdict=RED:lost_acked+wrong_state+resurrect+abandoned_applied
```

| | 字面 W（C287 那一行按字面：W = a） | 窄读 W（W = 重发那个 checkpoint 里的最大号，a 按新写序重做） | R-TXG |
|---|---|---|---|
| 链尾锚 + 实例代号为主 | 红：a 没施加，X 按 `n ≤ W` 判已发布 ⇒ 复活 | 绿 | 绿 |
| 链尾锚 + txg 为主 | 红（同上） | 绿 | 绿 |
| 水位过滤 + 实例代号为主 | 红（同上） | 绿 | 绿 |
| 水位过滤 + txg 为主 | 绿：a 从链首之后被施加 | 红：a 被施加，而 X 按窄读 W 判未发布 | **红：把被抛弃的 z 施加在回退根上，零故障** |

- 唯一能救字面 W 的组合（水位过滤 + txg 为主），正是 R-TXG 零故障打中的那一个 ⇒ **txg 为主出局**。
- 实例代号为主的代价：它把 a 的记录 (i, T+1) 判成被 (j, T)「覆盖」，而 a 不在那个根里——于是永远没有人施加 a，C287 只能取窄读 W（C287 那一行自己写的第二种读法「W 若只取被重发那个 checkpoint 里的最大事务号……这一格封死」）。这是约束，不是反例：窄读 W 下全表绿。
- Sonnet 产出 4.2 说两种序在 C287 上答案相同，是按 (j, T+1) 算的（第一节 E4）；按原文 (j, T)，两种序对「a 在不在根之后」答案相反，只是在链尾锚的读法下终判恰好都红。
- 择根仍是 txg 为主（D22（单元原子性怎么合成） 已定项 7，根环轮转要它）。两处比较的东西不同（根对根、记录对根），各写一句，不合并。

## 六、代价（第 5 问，只写这一腿核到的与前面几节推出的）

| 臂 | 要改的已定条款 | 第一个事务的字节 | 要补的检查 |
|---|---|---|---|
| 甲 | 删 D23（journal 的角色与格式） 两句「要改前缀规则」（一句是用户定案已定项 9 的依据，需用户知情）；写明两笔代价 | 不变 | 里程碑步 6 那条验收保留 |
| 甲′ | D16（发布语义） 已定项 7「fsync 等根槽持久之后才返回」加暖机条件（需用户知情）；回退确认同条件；D23（journal 的角色与格式） 两句照甲删 | **变**：第一个事务的 checkpoint_txg 1 → 3，多两次空发布与两条空记录；无新字段 | 崩溃点重放：新实例暖机之前不许有 fsync 返回；单故障遍历（每个根槽、每块盘）下已确认发布一个不丢 |
| 乙′ | I-8.3（重放前缀严格连续）；D23（journal 的角色与格式） 已定项 14 第一条、第五条与「之前没有持久效果」（用户定案，需用户知情）；D18（块里携带什么信息） 已定项 11「其余恒 0」的论证；⑤ 的行规则（收严后）；F-B | 不变（推的：第一个事务不跨实例）；链首按实例代号为主的水位定，不用加根记录字段 | E104（扫描重建的现行版本判定） 补「跨实例重放」「回退实例的根全读不出」两个世界；Y-A、Y-B 两格的会红检查 |
| 三条臂都要 | **F-A**：I-7.4（近 K 代块未被复用）「被抛弃时间线的根不算，它们的独占单元可以被清扫抹头」改成「离开根环之后才不算」（用户定案条款，需用户知情）；计数器全池接着走，改掉「每实例 2.8×10¹⁴」；记录水位实例代号为主；C287（切换收养开放 checkpoint 的事务后再崩） 取窄读 W | 不变 | Y-R 的会红检查：回退那次发布崩在记录之前，断言恢复出的根引用的单元一个没被复用；判别力自证：关掉 F-A 必须红 |

## 七、判定表与最小改法

「核过」= 当场读了原文或跑了模型；「推」= 没当场核。「没打中」都是一次抽样，只覆盖第二节建了的格。

| 臂 | 判据 1 正确性 | 判据 2 耐久性 | 判据 3 已定条款 | 判据 4 格式 | 判据 5′ 未确认的发布 | 判据 6 交互 |
|---|---|---|---|---|---|---|
| 甲 | **打中**：Y-R 零故障读到被复用单元；Y-C 一个故障同（核过） | **打中，1 个故障**：Y-A 撤销已确认的回退，Y-B / Y-C 丢回退与 X（核过）；第一轮 M1–M7 另算 | 删两句（需用户知情） | 不变 | **打中**：Y-R 回退没确认、先写坏了旧状态（核过） | C287 字面 W 复活，与臂无关（核过）；落回 R_old 时整段重放被抛弃时间线（推） |
| 甲′ | **打中**：Y-R 零故障、Y-C 两个故障，与甲同一个根（核过）；F-A 之后建了的格全绿 | 单故障**没打中**（C0、Y-B、Y-A / Y-C 要两个故障，核过）；两个故障记「丢」 | D16（发布语义） 已定项 7（需用户知情） | **变**：txg 1 → 3 | 同甲（核过） | 暖机失败转切换、耗 N_switch（推）；根区域全在一块盘上时永不返回（推）；C287 同甲 |
| 乙′ | **打中**：Y-A 最弱读法，一个故障复活被回退丢掉的写（核过）；收严后绿，⑤ 的 T 仍错（推）；Y-R 零故障同甲 | **打中，1 个故障**：Y-B 验不过即停，丢已确认的回退与 X（模型核过，停不停是推的） | I-8.3、D23 已定项 14 三句、D18 已定项 11 一段（需用户知情） | 不变（推） | Y-R 同甲；回退记录持久即整体生效，要改 P15 那句 | 固定点重写可能撑破保留池（推）；受保护的行挡住后来的回退（推）；E51 接缝不增（推）；N_switch 没打中（推） |

**按跑前写死的失败条款**：
- 乙′ 在判据 1 上被打中（Y-A）⇒ 乙′ 出局。收严 ⑤ 救得了 Y-A，救不了 Y-B（判据 2，一个故障）——Y-B 要的是 F-B 或一条新的链规则，那是新臂。
- 甲′ 在判据 2 上没有单故障打中序列 ⇒ 那条失败条款不触发。甲′ 在判据 1 上被打中的两格甲也中，而且在任何 C199 的臂之外：病根是 I-7.4 放开被抛弃的根 + 回退的分配器从 R_old 重新载入 + P17（被抛弃的根自己的表认不出自己被抛弃）。按字面三条臂一起出局；这一腿的建议是把 Y-R / Y-C 立成 C199 之外的一笔新欠账，F-A 作为三条臂共同的前提，再按判据 3 / 4 / 6 比甲′ 与甲。
- **与材料倾向相反**：这一腿的观测支持甲′（前提 F-A），不支持乙′。

**最小改法**（按依赖顺序）：
1. F-A（三条臂都要，先于 C199 定案）：I-7.4 那一句改成「被抛弃的根离开根环之后，它们的独占单元才可以复用或抹头」。它同时让 D23（journal 的角色与格式） 已定项 14「之前没有持久效果」重新成立。
2. 两件先写死：计数器全池接着走；记录水位 `(实例代号, checkpoint_txg)` 按实例代号为主（与 jsn 同序），择根照旧 txg 为主；C287 取窄读 W。
3. 取甲′：D16（发布语义） 已定项 7 的 fsync 返回条件与回退确认条件加「本实例写成（FUA 返回）的根覆盖两块盘；根区域只在一块盘上时覆盖两个区域」；删 D23 两句；第一个事务的字节随之改。
4. 若仍要乙′：⑤ 收严（回退写的中间实例行也置不可覆盖位；T 取被跨过实例在终态里仍生效的最后一个 txg，被回退抛弃的取 0）+ F-B（被抛弃时间线里仍在 journal 环里的已提交记录，它们点名的单元不许复用），并把「点名单元验不过时链停」写成条文。缺 F-B，乙′ 在一个故障下丢已确认的回退。

## 附录：模型源码、自证与产物（整份抄）

复跑命令（scratchpad 下，纯 Python 3，零依赖）：

```
cd /tmp/claude-1000/-home-fy5090-code-singlefs/86b2ac0e-7e65-418c-97a7-f3d755730fe6/scratchpad/c199-r2-opus && python3 c199r2_model.py > c199r2_model.out && python3 c199r2_model_selftest.py
```

md5（写报告时现算；产物连跑三次 md5 相同，确定性模型，跑几遍只说明没有隐藏状态）：

```
34d3b116c219ab834653233e74174875  c199r2_model.py
72e0d8b42df012a8fda02a4b09c5d783  c199r2_model_selftest.py
4b028a13f2617e152eb6307ed3b04be8  c199r2_model.out
5dd50eec21348dab01aa3fd4249af4e3  c199r2_model_selftest.out
```

自证输出 `c199r2_model_selftest.out`：

```
selftest ok: controls green x6; yi_prime red on Y-A/Y-B and green after tightening; jia_prime green on Y-B single fault while jia red; Y-R red x3, green under F-A; C287 literal-W red / narrow-W green; R-TXG txg-first filter red / inst-first green; verify mutation flips Y-B
```

自证源码 `c199r2_model_selftest.py`：

```python
"""c199r2_model 自证：对照格全绿；被攻的臂在打中格红；规则改回去（或收严）之后转绿；拿掉验证那一步结果要变。"""
import c199r2_model as model


def verdict(cell, arm, **overrides):
    return model.run(cell, arm, overrides)


def main():
    for arm in model.ARMS:
        for cell in ("C0", "C1"):
            assert not verdict(cell, arm)["reasons"], (cell, arm)
    r = verdict("Y-A", "yi_prime")
    assert r["resurrect"] and len(r["faults"]) == 1, r
    assert not verdict("Y-A", "yi_prime", protect_rb_intermediate=True)["reasons"]
    r = verdict("Y-A", "jia")
    assert r["lost_acked"] == ["RB"] and len(r["faults"]) == 1, r
    r = verdict("Y-A", "jia_prime")
    assert r["lost_acked"] == ["RB"] and len(r["faults"]) == 2, r
    r = verdict("Y-B", "yi_prime", protect_rb_intermediate=True)
    assert r["lost_acked"] == ["RB", "X3"] and len(r["faults"]) == 1, r
    assert not verdict("Y-B", "yi_prime", protect_rb_intermediate=True, verify_fail="skip")["reasons"]
    assert not verdict("Y-B", "jia_prime")["reasons"]
    assert verdict("Y-B", "jia")["lost_acked"] == ["RB", "X3"]
    for arm in model.ARMS:
        assert verdict("Y-R", arm, stop_at="units")["reads_reused"], arm
        assert not verdict("Y-R", arm, stop_at="units", protect_abandoned_in_ring=True)["reasons"], arm
    r = verdict("Y-C", "jia_prime")
    assert r["reads_reused"] and len(r["faults"]) == 2, r
    r = verdict("Y-C", "jia_prime", protect_abandoned_in_ring=True)
    assert not r["reads_reused"] and r["lost_acked"], r
    for arm in model.ARMS:
        for order in ("inst", "txg"):
            assert verdict("C287", arm, order=order)["resurrect"], (arm, order)
            assert not verdict("C287", arm, order=order, w_reading="narrow")["reasons"], (arm, order)
        assert verdict("R-TXG", arm, head="first_uncovered", order="txg")["abandoned_applied"], arm
        assert not verdict("R-TXG", arm, head="first_uncovered", order="inst")["reasons"], arm
    model.VERIFY_NAMED_UNITS = False
    r = verdict("Y-B", "yi_prime", protect_rb_intermediate=True)
    assert r["lost_acked"] != ["RB", "X3"], r
    model.VERIFY_NAMED_UNITS = True
    print("selftest ok: controls green x6; yi_prime red on Y-A/Y-B and green after tightening; "
          "jia_prime green on Y-B single fault while jia red; Y-R red x3, green under F-A; "
          "C287 literal-W red / narrow-W green; R-TXG txg-first filter red / inst-first green; "
          "verify mutation flips Y-B")


if __name__ == "__main__":
    main()
```

模型源码 `c199r2_model.py`（474 行，分四段抄进来，段与段首尾相接）：

```python
"""C199 第二轮反推腿计数模型（纯 Python，零文件 I/O）。

三条臂：jia（甲，严格前缀）/ jia_prime（甲′，严格前缀 + 新实例暖机）/ yi_prime（乙′，链尾锚跨实例）。
只建判决要用的格：C0、C1（对照）、Y-A、Y-R、Y-B、C287、R-TXG。
记录 = 那一刻的完整目标态（文件 → 单元），与第一轮同一口径；单元有物理落点，可以被复用。
根环 R = 3、每区 8 槽、区域归属 0 / 1 / 0；数据单元假定在两盘上有冗余（掉一块盘不丢单元）。
"""
import sys
from dataclasses import dataclass

REGIONS = 3
SLOTS_PER_REGION = 8
REGION_DISK = {0: 0, 1: 1, 2: 0}
ARMS = ("jia", "jia_prime", "yi_prime")
VERIFY_NAMED_UNITS = True   # 自证用的变异开关：False = 拿掉「施加前验证点名单元」
DEFAULT_PARAMS = {
    "head": "after_last_covered",   # 乙′ ①：链首 = 所选根覆盖的最后一条的下一条；first_uncovered = 按水位过滤
    "order": "inst",                # 记录水位比较序：inst = 实例代号为主；txg = txg 为主
    "verify_fail": "stop",          # 点名单元验不过：stop = 链停；skip = 丢这个事务、接着施加
    "protect_rb_intermediate": False,  # 回退写的中间实例行 (i, 0, 0) 带不带保护位
    "protect_abandoned_in_ring": False,  # 修法 F-A：被抛弃的根还在环里时，它引用的单元不许复用
    "w_reading": "literal",         # C287：切换行的 W 取字面（最后追加的事务）还是窄读（重发那个 checkpoint 里的最大号）
    "reuse_tail": True,             # Y-B：回退实例把被抛弃尾巴点名的单元复用掉
    "stop_at": "record",
    "yb_second_fault": False,       # Y-B：甲′ 那一格再加一个故障（回退实例在 1 号盘上的最新根也读不出）            # Y-R：回退那次发布崩在哪（units = 单元写完、记录之前；record = 记录之后、根之前）
}


def slot_of(txg):
    return (txg % REGIONS, (txg // REGIONS) % SLOTS_PER_REGION)


@dataclass
class Unit:
    owner: tuple   # 写序 (实例, 单元号)
    birth: int     # 诞生代号
    file: str      # 数据块属于哪个文件；None = 元数据（码 2 / 3），扫描检查不看它
    label: str


@dataclass(frozen=True)
class Record:
    counter: int
    instance: int
    txg: int
    transaction: int
    kind: str      # user / empty / rollback / switch
    state: tuple   # ((文件, 单元号, 写序, 标签), ...) 完整目标态
    named: tuple   # ((单元号, 写序), ...) 施加前要验证的点名单元
    table: tuple   # 回退 / 切换记录带进去的实例表 ((实例, T, W, 受保护), ...)；普通记录为 None
    label: str
    abandoned: bool  # 场景标注：属于被抛弃的时间线（只给检查用，恢复看不到）


@dataclass
class Root:
    instance: int
    txg: int
    state: dict    # 文件 -> (单元号, 写序, 标签)
    table: dict    # 实例 -> (T, W, 受保护)


class Pool:
    def __init__(self):
        self.units, self.ring, self.roots = {}, {}, {}
        self.bad_slots, self.lost_disks = set(), set()
        self.next_counter, self.next_unit = 1, 1
        self.acked = []

    def write_unit(self, owner, birth, file, label, reuse=None):
        unit_id = self.next_unit if reuse is None else reuse
        if reuse is None:
            self.next_unit += 1
        self.units[unit_id] = Unit(owner, birth, file, label)
        return unit_id

    def append(self, instance, txg, transaction, kind, state, named, table, label, abandoned):
        record = Record(self.next_counter, instance, txg, transaction, kind,
                        tuple(sorted((f,) + tuple(v) for f, v in state.items())), tuple(named),
                        None if table is None else tuple(sorted((i,) + tuple(v) for i, v in table.items())),
                        label, abandoned)
        self.ring[self.next_counter] = record
        self.next_counter += 1
        return record

    def publish(self, instance, txg, state, table):
        self.roots[slot_of(txg)] = Root(instance, txg, dict(state), dict(table))

    def readable_roots(self):
        return [root for slot, root in self.roots.items()
                if slot not in self.bad_slots and REGION_DISK[slot[0]] not in self.lost_disks]

    def roots_of(self, instance):
        return [(slot, root) for slot, root in self.roots.items() if root.instance == instance]


class Writer:
    """一个实例的写路径。jia_prime 的暖机：自己的根没落到两块盘上之前不确认任何东西。"""

    def __init__(self, pool, arm, instance, txg, state, table):
        self.pool, self.arm, self.instance, self.txg = pool, arm, instance, txg
        self.state, self.table = dict(state), dict(table)
        self.transaction, self.unit_number, self.pending_ack = 0, 0, []

    def new_owner(self):
        """单元写序 (i, n) 的 n 就是事务号（D18 已定项 11：码 1 ⇒ b ≤ T_pub ∨ n ≤ W）。"""
        self.transaction += 1
        return (self.instance, self.transaction)

    def record_txn(self, file, label, reuse=None, abandoned=False):
        owner = self.new_owner()
        unit = self.pool.write_unit(owner, self.txg, file, label, reuse)
        self.state[file] = (unit, owner, label)
        self.pool.append(self.instance, self.txg, self.transaction, "user", self.state,
                         ((unit, owner),), None, label, abandoned)
        self.pending_ack.append(label)
        return unit

    def covers_two_disks(self):
        return len({REGION_DISK[slot[0]] for slot, _ in self.pool.roots_of(self.instance)}) >= 2

    def publish(self, abandoned=False):
        self.pool.publish(self.instance, self.txg, self.state, self.table)
        self.txg += 1
        if self.arm != "jia_prime" or self.covers_two_disks():
            self.pool.acked.extend(self.pending_ack)
            self.pending_ack = []

    def warm_up(self):
        if self.arm == "jia_prime":
            while not self.covers_two_disks():
                self.pool.append(self.instance, self.txg, 0, "empty", self.state, (), None, "empty", False)
                self.publish()


# ---------------- 恢复 ----------------
def wm_key(instance, txg, order):
    return (txg, instance) if order == "txg" else (instance, txg)


def choose_root(pool):
    return max(pool.readable_roots(), key=lambda root: (root.txg, root.instance))


def find_head(pool, root, params):
    """根记录里没有 journal 位置字段（D22 已定项 7），「覆盖」只能按水位比出来。"""
    order = params["order"]
    root_key = wm_key(root.instance, root.txg, order)
    records = [pool.ring[counter] for counter in sorted(pool.ring)]
    if params["head"] == "after_last_covered":
        covered = [r for r in records if wm_key(r.instance, r.txg, order) <= root_key]
        if not covered:
            return (records[0].counter if records else None), root.instance
        return covered[-1].counter + 1, covered[-1].instance
    uncovered = [r for r in records if wm_key(r.instance, r.txg, order) > root_key]
    if not uncovered:
        return None, root.instance
    previous = pool.ring.get(uncovered[0].counter - 1)
    return uncovered[0].counter, (previous.instance if previous else root.instance)


def recover(pool, arm, recovery_instance, params):
    root = choose_root(pool)
    head, current = find_head(pool, root, params)
    root_key = wm_key(root.instance, root.txg, params["order"])
    state, table, applied, notes = dict(root.state), dict(root.table), [], []
    counter = head
    while counter is not None and counter in pool.ring:
        record = pool.ring[counter]
        if record.instance < current:
            notes.append(f"stop:residue@{counter}")
            break
        if record.instance > current:
            if arm in ("jia", "jia_prime"):
                notes.append(f"stop:instance-boundary@{counter}")
                break
            current = record.instance
        counter += 1
        if params["head"] == "first_uncovered" and wm_key(record.instance, record.txg, params["order"]) <= root_key:
            continue
        if VERIFY_NAMED_UNITS and any(pool.units[unit].owner != owner for unit, owner in record.named):
            notes.append(f"verify-fail:{record.label}")
            if params["verify_fail"] == "stop":
                break
            continue
        state = {f: (u, o, l) for f, u, o, l in record.state}
        if record.table is not None:
            table = {i: (t, w, p) for i, t, w, p in record.table}
        applied.append(record)
    write_rows(arm, root, recovery_instance, applied, table)
    return root, state, table, applied, notes


def write_rows(arm, root, recovery_instance, applied, table):
    """甲 / 甲′：D18 已定项 11「行怎么写」。乙′：规则 ⑤ (j, 所选根 txg, W_j)，W_j 只算最后一条回退记录之后
    仍然生效的事务，中间实例行带保护位。受保护的行（回退行，以及按参数带保护的中间行）一律不覆盖。"""
    last_rollback = max((n for n, r in enumerate(applied) if r.kind == "rollback"), default=-1)
    for instance in range(root.instance, recovery_instance):
        if instance in table and table[instance][2]:
            continue
        if arm == "yi_prime":
            live = [r.transaction for r in applied[last_rollback + 1:] if r.instance == instance]
            table[instance] = (root.txg, max(live, default=0), True)
        else:
            done = [r.transaction for r in applied if r.instance == instance]
            table[instance] = (root.txg, max(done, default=0), False)


# ---------------- 检查 ----------------
def is_published(unit, table, now_instance, now_txg):
    """D18 已定项 11 已发布谓词，码 1 那一支：b ≤ T_pub ∨ n ≤ W。"""
    instance, number = unit.owner
    if instance > now_instance:
        return False
    if instance == now_instance:
        return unit.birth <= now_txg
    if instance not in table:
        return True
    row_txg, row_w, _ = table[instance]
    return unit.birth <= row_txg or number <= row_w


def evaluate(pool, arm, recovery_instance, params, acceptable, at_risk):
    root, state, table, applied, notes = recover(pool, arm, recovery_instance, params)
    now_txg = max(r.txg for r in pool.roots.values()) + 1
    final = {f: l for f, (u, o, l) in state.items()}
    labels = set(final.values())
    lost_acked = [label for label in sorted(at_risk) if label in pool.acked and label not in labels]
    reads_reused = sorted(l for f, (u, o, l) in state.items() if pool.units[u].owner != o)
    resurrect = []
    for file in sorted({unit.file for unit in pool.units.values() if unit.file}):
        published = [(unit.owner, unit_id) for unit_id, unit in pool.units.items()
                     if unit.file == file and is_published(unit, table, recovery_instance, now_txg)]
        scan_pick = max(published)[1] if published else None
        tree_pick = state[file][0] if file in state else None
        if scan_pick != tree_pick:
            scan_label = pool.units[scan_pick].label if scan_pick else None
            resurrect.append(f"{file}:scan={scan_label}/tree={final.get(file)}")
    abandoned_applied = [r.label for r in applied if r.abandoned]
    wrong_state = final not in acceptable
    reasons = [name for name, hit in (("lost_acked", lost_acked), ("wrong_state", wrong_state),
                                      ("reads_reused", reads_reused), ("resurrect", resurrect),
                                      ("abandoned_applied", abandoned_applied)) if hit]
    return {"root": (root.instance, root.txg), "applied": [r.label for r in applied],
            "final": sorted(final.items()), "notes": notes, "lost_acked": lost_acked,
            "reads_reused": reads_reused, "resurrect": resurrect,
            "abandoned_applied": abandoned_applied, "reasons": reasons}


# ---------------- 场景 ----------------
def latest_root_of(pool, instance):
    return max((root for _, root in pool.roots_of(instance)), key=lambda root: root.txg)


def labels_of(state):
    return {f: l for f, (u, o, l) in state.items()}


def fail_all_roots_of(pool, instance):
    """让一个实例的根全部读不出，用最少的故障：只有一个根 ⇒ 一个槽；否则 0 号盘整块掉 + 1 号盘上逐槽。"""
    slots = [slot for slot, _ in pool.roots_of(instance)]
    if len(slots) == 1:
        pool.bad_slots.add(slots[0])
        return [f"slot{slots[0]}"]
    actions = []
    if any(REGION_DISK[s[0]] == 0 for s in slots):
        pool.lost_disks.add(0)
        actions.append("disk0")
    for slot in slots:
        if REGION_DISK[slot[0]] == 1:
            pool.bad_slots.add(slot)
            actions.append(f"slot{slot}")
    return actions


def rollback(pool, arm, new_instance, r_old, params, intermediates, reuse=None, stop_at="root", warm=True):
    """D23 已定项 14 显式例外：回退行 (r_old, T_old, 0) 带保护、中间实例 (i, 0, 0)、新根 txg = 根环最大 + 1、
    分配器从 R_old 的账重新载入 ⇒ 回退那次发布的 COW 单元可以落在被抛弃时间线独占的单元上（reuse）。"""
    table = dict(r_old.table)
    table[r_old.instance] = (r_old.txg, 0, True)
    for instance in intermediates:
        table[instance] = (0, 0, params["protect_rb_intermediate"])
    writer = Writer(pool, arm, new_instance, max(r.txg for r in pool.roots.values()) + 1, r_old.state, table)
    owner = writer.new_owner()
    unit = pool.write_unit(owner, writer.txg, None, "rb-table", reuse)
    writer.state["@rb"] = (unit, owner, "RB")
    if stop_at == "units":
        return writer
    pool.append(new_instance, writer.txg, writer.transaction, "rollback", writer.state, ((unit, owner),), table, "RB", False)
    writer.pending_ack.append("RB")
    if stop_at == "record":
        return writer
    writer.publish()
    if warm:
        writer.warm_up()
    return writer


def build_abandoned_timeline(pool, arm):
    """实例 1 发 F1、G1（R_old = 它最后一个根）；干净卸载；实例 2 干净重挂（不写行），发 F2、H2——这两次后来被回退抛弃。"""
    w1 = Writer(pool, arm, 1, 1, {}, {})
    w1.warm_up()
    w1.record_txn("F", "F1"); w1.publish()
    w1.record_txn("G", "G1"); w1.publish()
    r_old = latest_root_of(pool, 1)
    w2 = Writer(pool, arm, 2, w1.txg, w1.state, w1.table)
    w2.warm_up()
    unit_f2 = w2.record_txn("F", "F2", abandoned=True); w2.publish()
    w2.record_txn("H", "H2", abandoned=True); w2.publish()
    return r_old, w2, unit_f2


def cell_c0(arm, params):
    pool = Pool()
    w = Writer(pool, arm, 1, 1, {}, {})
    w.warm_up()
    for file, label in (("fA", "A"), ("fB", "B"), ("fX", "X")):
        w.record_txn(file, label); w.publish()
    pool.bad_slots.add(slot_of(latest_root_of(pool, 1).txg))
    return pool, 2, [{"fA": "A", "fB": "B", "fX": "X"}], ["slot"], {"X"}


def cell_c1(arm, params):
    pool = Pool()
    w = Writer(pool, arm, 1, 1, {}, {})
    w.warm_up()
    w.record_txn("fA", "A"); w.publish()
    r_old = latest_root_of(pool, 1)
    w.record_txn("fB", "B", abandoned=True); w.publish()
    rollback(pool, arm, 2, r_old, params, [])
    return pool, 3, [{"fA": "A", "@rb": "RB"}], [], {"RB"}


def cell_ya(arm, params):
    pool = Pool()
    r_old, _, _ = build_abandoned_timeline(pool, arm)
    rollback(pool, arm, 3, r_old, params, [2])
    faults = fail_all_roots_of(pool, 3)
    return pool, 4, [dict(labels_of(r_old.state), **{"@rb": "RB"})], faults, {"RB"}


def cell_yr(arm, params):
    pool = Pool()
    r_old, w2, unit_f2 = build_abandoned_timeline(pool, arm)
    reuse = None if params["protect_abandoned_in_ring"] else unit_f2
    rollback(pool, arm, 3, r_old, params, [2], reuse=reuse, stop_at=params["stop_at"])
    acceptable = [labels_of(w2.state), dict(labels_of(r_old.state), **{"@rb": "RB"})]
    return pool, 4, acceptable, [], set()


def cell_yb(arm, params):
    pool = Pool()
    r_old, w2, _ = build_abandoned_timeline(pool, arm)
    unit_tail = w2.record_txn("T", "T2", abandoned=True)   # 已提交、没发布就崩：被抛弃时间线里没进根的那一截
    w3 = rollback(pool, arm, 3, r_old, params, [2])
    w3.record_txn("X", "X3", reuse=unit_tail if params["reuse_tail"] else None); w3.publish()
    pool.lost_disks.add(0)
    faults = ["disk0"]
    on_disk1 = [r for s, r in pool.roots_of(3) if REGION_DISK[s[0]] == 1]
    if params["yb_second_fault"] and on_disk1:   # 只有甲′ 的回退实例在 1 号盘上有根；别的臂这一格与单故障相同
        newest_on_disk1 = max(on_disk1, key=lambda r: r.txg)
        pool.bad_slots.add(slot_of(newest_on_disk1.txg))
        faults.append(f"slot{slot_of(newest_on_disk1.txg)}")
    return pool, 4, [dict(labels_of(r_old.state), **{"@rb": "RB", "X": "X3"})], faults, {"RB", "X3"}


def cell_c287(arm, params):
    """C287 那一行逐字：开放的 T+1 里 a 写完 X、记录已追加，b 发出单元写后失败 ⇒ 切换、写行 (旧实例, T−1, W)；
    T+1 发布之前再崩，所选根是新实例写的 T。"""
    pool = Pool()
    w1 = Writer(pool, arm, 1, 1, {}, {})
    w1.warm_up()
    w1.record_txn("fA", "A"); w1.publish()
    w1.record_txn("fB", "B"); w1.publish()
    txg_t = w1.txg
    w1.record_txn("fP", "P1")
    state_after_p, w_narrow = dict(w1.state), w1.transaction
    w1.txg += 1
    w1.record_txn("X", "a1")
    w_literal = w1.transaction
    table = dict(w1.table)
    table[1] = (txg_t - 1, w_literal if params["w_reading"] == "literal" else w_narrow, False)
    w2 = Writer(pool, arm, 2, txg_t, state_after_p, table)
    w2.transaction += 1
    pool.append(2, txg_t, w2.transaction, "switch", w2.state, (), table, "switch", False)
    pool.publish(2, txg_t, w2.state, table)
    if arm != "jia_prime":
        pool.acked.append("P1")
    base = labels_of(state_after_p)
    return pool, 3, [base, dict(base, X="a1")], [], {"P1"}


def cell_rtxg(arm, params):
    """回退新根取「根环最大 + 1」，而被抛弃的实例在开放 checkpoint 里有 txg 更大的已提交记录。"""
    pool = Pool()
    w1 = Writer(pool, arm, 1, 1, {}, {})
    w1.warm_up()
    w1.record_txn("fA", "A"); w1.publish()
    r_old = latest_root_of(pool, 1)
    w1.record_txn("fB", "B", abandoned=True); w1.publish()
    w1.record_txn("Q", "q1", abandoned=True)
    w1.txg += 1
    w1.record_txn("Z", "z1", abandoned=True)
    rollback(pool, arm, 2, r_old, params, [], warm=False)
    return pool, 3, [{"fA": "A", "@rb": "RB"}], [], {"RB"}


def cell_yc(arm, params):
    """回退已确认之后，回退实例按 I-7.4「被抛弃时间线的根不算」复用了被抛弃根引用的单元；
    再让回退实例的根全部读不出 ⇒ 恢复选中那个被抛弃的根。"""
    pool = Pool()
    r_old, w2, unit_f2 = build_abandoned_timeline(pool, arm)
    extra = 0
    while REGION_DISK[slot_of(latest_root_of(pool, 2).txg)[0]] != 1:
        extra += 1
        w2.record_txn(f"I{extra}", f"I{extra}", abandoned=True); w2.publish()
    w3 = rollback(pool, arm, 3, r_old, params, [2])
    w3.record_txn("X", "X3", reuse=None if params["protect_abandoned_in_ring"] else unit_f2); w3.publish()
    faults = fail_all_roots_of(pool, 3)
    return pool, 4, [dict(labels_of(r_old.state), **{"@rb": "RB", "X": "X3"})], faults, {"RB", "X3"}


CELLS = {"C0": cell_c0, "C1": cell_c1, "Y-A": cell_ya, "Y-R": cell_yr, "Y-B": cell_yb,
         "Y-C": cell_yc, "C287": cell_c287, "R-TXG": cell_rtxg}

PLAN = [("C0", {}), ("C1", {}),
        ("Y-A", {}), ("Y-A", {"protect_rb_intermediate": True}),
        ("Y-R", {"stop_at": "units"}), ("Y-R", {"stop_at": "record"}),
        ("Y-R", {"stop_at": "units", "protect_abandoned_in_ring": True}),
        ("Y-R", {"stop_at": "record", "protect_abandoned_in_ring": True}),
        ("Y-B", {}), ("Y-B", {"verify_fail": "skip"}), ("Y-B", {"reuse_tail": False}),
        ("Y-B", {"yb_second_fault": True}),
        ("Y-B", {"protect_rb_intermediate": True}),
        ("Y-B", {"verify_fail": "skip", "protect_rb_intermediate": True}),
        ("Y-B", {"reuse_tail": False, "protect_rb_intermediate": True}),
        ("Y-R", {"stop_at": "record", "protect_abandoned_in_ring": True, "protect_rb_intermediate": True}),
        ("Y-C", {}), ("Y-C", {"protect_abandoned_in_ring": True}),
        ("Y-C", {"protect_abandoned_in_ring": True, "protect_rb_intermediate": True})]
PLAN += [("C287", {"order": order, "head": head, "w_reading": w_reading})
         for w_reading in ("literal", "narrow") for head in ("after_last_covered", "first_uncovered")
         for order in ("inst", "txg")]
PLAN += [("R-TXG", {"order": order, "head": head})
         for head in ("after_last_covered", "first_uncovered") for order in ("inst", "txg")]


def run(cell, arm, overrides=None):
    params = dict(DEFAULT_PARAMS)
    params.update(overrides or {})
    pool, recovery_instance, acceptable, faults, at_risk = CELLS[cell](arm, params)
    result = evaluate(pool, arm, recovery_instance, params, acceptable, at_risk)
    result["faults"] = faults
    return result


def format_line(cell, arm, overrides, result):
    params = ",".join(f"{k}={v}" for k, v in sorted(overrides.items())) or "default"
    final = ",".join(f"{f}={l}" for f, l in result["final"])
    verdict = "GREEN" if not result["reasons"] else "RED:" + "+".join(result["reasons"])
    return (f"cell={cell} arm={arm} params={params} faults={len(result['faults'])}{result['faults']} "
            f"root={result['root']} applied={result['applied']} final=[{final}] notes={result['notes']} "
            f"lost_acked={result['lost_acked']} reads_reused={result['reads_reused']} "
            f"resurrect={result['resurrect']} abandoned_applied={result['abandoned_applied']} verdict={verdict}")


def main():
    lines = [format_line(cell, arm, overrides, run(cell, arm, overrides))
             for cell, overrides in PLAN for arm in ARMS]
    for line in lines:
        print(line)
    print(f"emitted={len(lines)}")


if __name__ == "__main__":
    sys.exit(main())
```

模型产物 `c199r2_model.out`（94 行 = 93 行 + 末行条数）：

```
cell=C0 arm=jia params=default faults=1['slot'] root=(1, 2) applied=['X'] final=[fA=A,fB=B,fX=X] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C0 arm=jia_prime params=default faults=1['slot'] root=(1, 4) applied=['X'] final=[fA=A,fB=B,fX=X] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C0 arm=yi_prime params=default faults=1['slot'] root=(1, 2) applied=['X'] final=[fA=A,fB=B,fX=X] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C1 arm=jia params=default faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C1 arm=jia_prime params=default faults=0[] root=(2, 7) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C1 arm=yi_prime params=default faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-A arm=jia params=default faults=1['slot(2, 1)'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=['RB'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-A arm=jia_prime params=default faults=2['disk0', 'slot(1, 3)'] root=(2, 7) applied=['F2', 'H2'] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@10'] lost_acked=['RB'] reads_reused=[] resurrect=[] abandoned_applied=['F2', 'H2'] verdict=RED:lost_acked+wrong_state+abandoned_applied
cell=Y-A arm=yi_prime params=default faults=1['slot(2, 1)'] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=['F:scan=F2/tree=F1', 'H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-A arm=jia params=protect_rb_intermediate=True faults=1['slot(2, 1)'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=['RB'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-A arm=jia_prime params=protect_rb_intermediate=True faults=2['disk0', 'slot(1, 3)'] root=(2, 7) applied=['F2', 'H2'] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@10'] lost_acked=['RB'] reads_reused=[] resurrect=[] abandoned_applied=['F2', 'H2'] verdict=RED:lost_acked+wrong_state+abandoned_applied
cell=Y-A arm=yi_prime params=protect_rb_intermediate=True faults=1['slot(2, 1)'] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=jia params=stop_at=units faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=jia_prime params=stop_at=units faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=yi_prime params=stop_at=units faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=jia params=stop_at=record faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=jia_prime params=stop_at=record faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@10'] lost_acked=[] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:reads_reused+resurrect
cell=Y-R arm=yi_prime params=stop_at=record faults=0[] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=['H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-R arm=jia params=protect_abandoned_in_ring=True,stop_at=units faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=jia_prime params=protect_abandoned_in_ring=True,stop_at=units faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=yi_prime params=protect_abandoned_in_ring=True,stop_at=units faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=jia params=protect_abandoned_in_ring=True,stop_at=record faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=jia_prime params=protect_abandoned_in_ring=True,stop_at=record faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@10'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=yi_prime params=protect_abandoned_in_ring=True,stop_at=record faults=0[] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=['F:scan=F2/tree=F1', 'H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-B arm=jia params=default faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-B arm=jia_prime params=default faults=1['disk0'] root=(3, 10) applied=['empty', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-B arm=yi_prime params=default faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-B arm=jia params=verify_fail=skip faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2', 'stop:instance-boundary@6'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-B arm=jia_prime params=verify_fail=skip faults=1['disk0'] root=(3, 10) applied=['empty', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-B arm=yi_prime params=verify_fail=skip faults=1['disk0'] root=(2, 4) applied=['RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=['verify-fail:T2'] lost_acked=[] reads_reused=[] resurrect=['F:scan=F2/tree=F1', 'H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-B arm=jia params=reuse_tail=False faults=1['disk0'] root=(2, 4) applied=['T2'] final=[F=F2,G=G1,H=H2,T=T2] notes=['stop:instance-boundary@6'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=['T2'] verdict=RED:lost_acked+wrong_state+abandoned_applied
cell=Y-B arm=jia_prime params=reuse_tail=False faults=1['disk0'] root=(3, 10) applied=['empty', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-B arm=yi_prime params=reuse_tail=False faults=1['disk0'] root=(2, 4) applied=['T2', 'RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=['F:scan=F2/tree=F1', 'H:scan=H2/tree=None'] abandoned_applied=['T2'] verdict=RED:resurrect+abandoned_applied
cell=Y-B arm=jia params=yb_second_fault=True faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-B arm=jia_prime params=yb_second_fault=True faults=2['disk0', 'slot(1, 3)'] root=(2, 7) applied=['F2', 'H2'] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=['F2', 'H2'] verdict=RED:lost_acked+wrong_state+abandoned_applied
cell=Y-B arm=yi_prime params=yb_second_fault=True faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-B arm=jia params=protect_rb_intermediate=True faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-B arm=jia_prime params=protect_rb_intermediate=True faults=1['disk0'] root=(3, 10) applied=['empty', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-B arm=yi_prime params=protect_rb_intermediate=True faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-B arm=jia params=protect_rb_intermediate=True,verify_fail=skip faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['verify-fail:T2', 'stop:instance-boundary@6'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-B arm=jia_prime params=protect_rb_intermediate=True,verify_fail=skip faults=1['disk0'] root=(3, 10) applied=['empty', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-B arm=yi_prime params=protect_rb_intermediate=True,verify_fail=skip faults=1['disk0'] root=(2, 4) applied=['RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=['verify-fail:T2'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-B arm=jia params=protect_rb_intermediate=True,reuse_tail=False faults=1['disk0'] root=(2, 4) applied=['T2'] final=[F=F2,G=G1,H=H2,T=T2] notes=['stop:instance-boundary@6'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=['T2'] verdict=RED:lost_acked+wrong_state+abandoned_applied
cell=Y-B arm=jia_prime params=protect_rb_intermediate=True,reuse_tail=False faults=1['disk0'] root=(3, 10) applied=['empty', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-B arm=yi_prime params=protect_rb_intermediate=True,reuse_tail=False faults=1['disk0'] root=(2, 4) applied=['T2', 'RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=['T2'] verdict=RED:abandoned_applied
cell=Y-R arm=jia params=protect_abandoned_in_ring=True,protect_rb_intermediate=True,stop_at=record faults=0[] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=jia_prime params=protect_abandoned_in_ring=True,protect_rb_intermediate=True,stop_at=record faults=0[] root=(2, 9) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@10'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-R arm=yi_prime params=protect_abandoned_in_ring=True,protect_rb_intermediate=True,stop_at=record faults=0[] root=(2, 4) applied=['RB'] final=[@rb=RB,F=F1,G=G1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=Y-C arm=jia params=default faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=['RB', 'X3'] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:lost_acked+wrong_state+reads_reused+resurrect
cell=Y-C arm=jia_prime params=default faults=2['disk0', 'slot(1, 4)'] root=(2, 10) applied=[] final=[F=F2,G=G1,H=H2,I1=I1] notes=['stop:instance-boundary@11'] lost_acked=['RB', 'X3'] reads_reused=['F2'] resurrect=['F:scan=F1/tree=F2'] abandoned_applied=[] verdict=RED:lost_acked+wrong_state+reads_reused+resurrect
cell=Y-C arm=yi_prime params=default faults=1['disk0'] root=(2, 4) applied=['RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=['H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-C arm=jia params=protect_abandoned_in_ring=True faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-C arm=jia_prime params=protect_abandoned_in_ring=True faults=2['disk0', 'slot(1, 4)'] root=(2, 10) applied=[] final=[F=F2,G=G1,H=H2,I1=I1] notes=['stop:instance-boundary@11'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-C arm=yi_prime params=protect_abandoned_in_ring=True faults=1['disk0'] root=(2, 4) applied=['RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=['F:scan=F2/tree=F1', 'H:scan=H2/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=Y-C arm=jia params=protect_abandoned_in_ring=True,protect_rb_intermediate=True faults=1['disk0'] root=(2, 4) applied=[] final=[F=F2,G=G1,H=H2] notes=['stop:instance-boundary@5'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-C arm=jia_prime params=protect_abandoned_in_ring=True,protect_rb_intermediate=True faults=2['disk0', 'slot(1, 4)'] root=(2, 10) applied=[] final=[F=F2,G=G1,H=H2,I1=I1] notes=['stop:instance-boundary@11'] lost_acked=['RB', 'X3'] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=RED:lost_acked+wrong_state
cell=Y-C arm=yi_prime params=protect_abandoned_in_ring=True,protect_rb_intermediate=True faults=1['disk0'] root=(2, 4) applied=['RB', 'X3'] final=[@rb=RB,F=F1,G=G1,X=X3] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=jia params=head=after_last_covered,order=inst,w_reading=literal faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=jia_prime params=head=after_last_covered,order=inst,w_reading=literal faults=0[] root=(2, 5) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=yi_prime params=head=after_last_covered,order=inst,w_reading=literal faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=jia params=head=after_last_covered,order=txg,w_reading=literal faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=jia_prime params=head=after_last_covered,order=txg,w_reading=literal faults=0[] root=(2, 5) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=yi_prime params=head=after_last_covered,order=txg,w_reading=literal faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=jia params=head=first_uncovered,order=inst,w_reading=literal faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=jia_prime params=head=first_uncovered,order=inst,w_reading=literal faults=0[] root=(2, 5) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=yi_prime params=head=first_uncovered,order=inst,w_reading=literal faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=a1/tree=None'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=jia params=head=first_uncovered,order=txg,w_reading=literal faults=0[] root=(2, 3) applied=['a1'] final=[X=a1,fA=A,fB=B,fP=P1] notes=['stop:instance-boundary@5'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=jia_prime params=head=first_uncovered,order=txg,w_reading=literal faults=0[] root=(2, 5) applied=['a1'] final=[X=a1,fA=A,fB=B,fP=P1] notes=['stop:instance-boundary@7'] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=yi_prime params=head=first_uncovered,order=txg,w_reading=literal faults=0[] root=(2, 3) applied=['a1'] final=[X=a1,fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=jia params=head=after_last_covered,order=inst,w_reading=narrow faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=jia_prime params=head=after_last_covered,order=inst,w_reading=narrow faults=0[] root=(2, 5) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=yi_prime params=head=after_last_covered,order=inst,w_reading=narrow faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=jia params=head=after_last_covered,order=txg,w_reading=narrow faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=jia_prime params=head=after_last_covered,order=txg,w_reading=narrow faults=0[] root=(2, 5) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=yi_prime params=head=after_last_covered,order=txg,w_reading=narrow faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=jia params=head=first_uncovered,order=inst,w_reading=narrow faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=jia_prime params=head=first_uncovered,order=inst,w_reading=narrow faults=0[] root=(2, 5) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=yi_prime params=head=first_uncovered,order=inst,w_reading=narrow faults=0[] root=(2, 3) applied=[] final=[fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=C287 arm=jia params=head=first_uncovered,order=txg,w_reading=narrow faults=0[] root=(2, 3) applied=['a1'] final=[X=a1,fA=A,fB=B,fP=P1] notes=['stop:instance-boundary@5'] lost_acked=[] reads_reused=[] resurrect=['X:scan=None/tree=a1'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=jia_prime params=head=first_uncovered,order=txg,w_reading=narrow faults=0[] root=(2, 5) applied=['a1'] final=[X=a1,fA=A,fB=B,fP=P1] notes=['stop:instance-boundary@7'] lost_acked=[] reads_reused=[] resurrect=['X:scan=None/tree=a1'] abandoned_applied=[] verdict=RED:resurrect
cell=C287 arm=yi_prime params=head=first_uncovered,order=txg,w_reading=narrow faults=0[] root=(2, 3) applied=['a1'] final=[X=a1,fA=A,fB=B,fP=P1] notes=[] lost_acked=[] reads_reused=[] resurrect=['X:scan=None/tree=a1'] abandoned_applied=[] verdict=RED:resurrect
cell=R-TXG arm=jia params=head=after_last_covered,order=inst faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=jia_prime params=head=after_last_covered,order=inst faults=0[] root=(2, 5) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=yi_prime params=head=after_last_covered,order=inst faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=jia params=head=after_last_covered,order=txg faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=jia_prime params=head=after_last_covered,order=txg faults=0[] root=(2, 5) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=yi_prime params=head=after_last_covered,order=txg faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=jia params=head=first_uncovered,order=inst faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=jia_prime params=head=first_uncovered,order=inst faults=0[] root=(2, 5) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=yi_prime params=head=first_uncovered,order=inst faults=0[] root=(2, 3) applied=[] final=[@rb=RB,fA=A] notes=[] lost_acked=[] reads_reused=[] resurrect=[] abandoned_applied=[] verdict=GREEN
cell=R-TXG arm=jia params=head=first_uncovered,order=txg faults=0[] root=(2, 3) applied=['z1'] final=[Q=q1,Z=z1,fA=A,fB=B] notes=['stop:instance-boundary@5'] lost_acked=['RB'] reads_reused=[] resurrect=['Q:scan=None/tree=q1', 'Z:scan=None/tree=z1', 'fB:scan=None/tree=B'] abandoned_applied=['z1'] verdict=RED:lost_acked+wrong_state+resurrect+abandoned_applied
cell=R-TXG arm=jia_prime params=head=first_uncovered,order=txg faults=0[] root=(2, 5) applied=['z1'] final=[Q=q1,Z=z1,fA=A,fB=B] notes=['stop:instance-boundary@7'] lost_acked=[] reads_reused=[] resurrect=['Q:scan=None/tree=q1', 'Z:scan=None/tree=z1', 'fB:scan=None/tree=B'] abandoned_applied=['z1'] verdict=RED:wrong_state+resurrect+abandoned_applied
cell=R-TXG arm=yi_prime params=head=first_uncovered,order=txg faults=0[] root=(2, 3) applied=['z1'] final=[Q=q1,Z=z1,fA=A,fB=B] notes=[] lost_acked=['RB'] reads_reused=[] resurrect=['Q:scan=None/tree=q1', 'Z:scan=None/tree=z1', 'fB:scan=None/tree=B'] abandoned_applied=['z1'] verdict=RED:lost_acked+wrong_state+resurrect+abandoned_applied
emitted=93
```

## 历史版本

（无）
