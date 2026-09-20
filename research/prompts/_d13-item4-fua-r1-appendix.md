**出处 `.claude/kb/decisions/13-验证路线.md:2-4`（整段抄，未转述）**

```markdown

D13（验证路线） 管本工程拿什么验正确性：三类现成办法全做——自写参照实现做差分对拍、有界穷举崩溃测试、crash refinement 形式验证（调查见 [prior-art.md](../prior-art.md) 7.4）——并且不照抄，针对本工程特有的四样（COW 每事务发新根、Merkle 自验证、整卷 AEAD 的 nonce 唯一性、没有参照实现）自己设计验证手段。它定 oracle 怎么分、自研哪两样、崩溃点重放怎么分层与抽样、崩溃状态集合怎么定义、checker 与实现共享什么。不管的：每条不变量写什么在 [invariants.md](../invariants.md)；三样验证手段怎么落地、谁挡着谁在 [verification-build.md](../verification-build.md)；提交步骤登记在哪、结构等价类怎么分、今天几个，归 D17（实现分层与第三方管道） 已定项 2。

```

**出处 `.claude/kb/decisions/13-验证路线.md:69-81`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：屏障把录下来的写请求流切成段，一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写也是段边界：它自成一段，它之后的写不与它同段。撕裂子集**不枚举**，一次写的任何撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像要真的生成出来喂给 checker，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验；设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内——撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度因此不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案），记在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/02-RAID条带策略.md:106-119`（整段抄，未转述）**

```markdown
#### 已定项 6：每次写用几列

**定案**：`w = clamp(攒批后的写入量 + 1, 2, 4)`，再夹「当时可写的设备数」。**「攒批后的写入量」按格宽分桶各算各的**（已定项 10 依据 2：一条条带的全部格必须等宽）——一次发布里 16 KiB 与 32 KiB 两桶各自算出自己的 `w`。**上界 4 是超级块声明的常量**，与设备数无关；`w ≥ 2` 是硬下界（零冗余的条带不许发出）。

**射程**：`w` 与已定项 8 的组大小 `g` 是两个量，恒有 `w ≤ g`；两者初值都是 4 是巧合，改一个不自动改另一个。第一版 `w` 恒 2 并被已定项 18 的加盘准入钉住，`clamp` 的上界 4 在第一版永远碰不到——它是**声明**，不是第一版走得到的取值。承重依据是 `.claude/rules/fs-design.md` 的硬要求 5（分支变量不许在操作中途改变：上界若写成 `f(设备数)`，设备数在写飞行中就变，掉盘不走已定项 4 b 的排空屏障）与硬要求 2（每条分支必须能被测试强制进入：常量把取值集封闭成 `{2,3,4}`，三条分支各自可达），不是边际算术。三处必须写明的不知道：① 「拐点 4」那套边际算术有**量纲错误**，风险项按每条条带、成本项按每份容量，同量纲的度量是期望丢数据比例 `(w−1)/C(D,2)`，**线性不是二次**，同一等权给 D=8 时上界 **6** ⇒ 4 是更保守的那一侧，不由那套算术钉住；② 等权（1 个百分点空间 ↔ 1 个百分点丢数据）在 kb 里没有出处，λ=1 给 6、λ=2 给 3、λ=0.5 给 5；③ 整套推导**只在 `parity = 1` 下成立**，parity=2 时双盘同坏不丢数据、风险轴要换成三盘同坏，同一算术给 7 ⇒ 上界随 parity 重定，而 parity 几格是未定项 21。

**依据**：

- E63（分配器每次写用几列）：小写占比 90% 时「贴合写入量」比「恒取全宽」省 61% 物理格 ⇒ 判据必须含写入量；8 盘攒批下上界取 4 与取消上界（＝8）在双盘同坏时期望丢掉 10.7% 对 25.0%，取消上界只省 13% 空间却换 2.3 倍的期望丢数据，而且每条带命中率 1.000 ⇒ 没有一个文件是完整的，失败模式从「部分受损」变成弥散。
- 三方论证：正推腿接受收窄版并抓到那套边际算术的量纲错误，反推腿判「D2（RAID 条带策略） 原文写了三样要一起给，只给一个数不算定案」⇒ 本次只定判据与上界、参与子集拆去已定项 8，本地腿主张按池算、它的小池反例在量纲修正后弱化（D=4 时 3→4 打平），原始输出 `research/results/d2-item6-local.out`。
- 用户定案，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/22-单元原子性怎么合成.md:165-182`（整段抄，未转述）**

```markdown
#### 已定项 8：固定结构的放置

**定案**：三条，都是针对 2 盘的第一版（D2（RAID 条带策略）已定项 9）：

1. **超级块每盘放一份。** 怎么更新它按已定项 21 办（原地覆写的结构要有整单元校验和，还要有真的被检查的世代号），做成槽轮换，每盘至少 2 个槽；槽的规则见已定项 16，字段见已定项 9。
2. **journal 环在两块盘上互为镜像。** 代价是稳态下每条记录写两遍。**这不是新增的开销**——D2（RAID 条带策略）已定项 6 本来就要求任何写都得 w≥2，这条定案只是把「任何写」明确到 journal 记录上。
3. **mkfs 要把第 0 代根种进全部根环区域**，并把三个根环区域整段写 0（含未种根的那 3 × (S−1) 个槽）：与 journal 整环写 0 同为镜像准备，两者都不进录制流、不算崩溃段。种满之后「刚 mkfs 完、一次都还没发布」这一格跟稳态一样，根最多退回区域深度那么多代。

**射程**：超级块自己的更新协议（槽轮换的代号、校验和多宽）跟着单元头字段表（D18（块里携带什么信息）已定项 7）的口径走，今天落在已定项 9 与已定项 16；zoned 介质上这些固定结构能不能原地轮转，属于 D22（单元原子性怎么合成） 未定项 6，还没答。

**依据**：

- E87（固定结构的放置）：2 盘 8 种失效组合穷举——超级块只放一份时掉了那块盘全池挂不上（数据、根、journal 健在也没用），每盘一份 + journal 镜像 8 格全可挂、重放窗口零丢失；不种满第 0 代根时「刚 mkfs、零次发布、掉任一盘」连一个能退回去的根都没有。
- 整段写 0：不写的话，在一块装过上一份 singlefs 的盘上重 mkfs，旧文件系统留在槽 1..S−1 里的根记录自证校验和照样通过、`checkpoint_txg` 还更大 ⇒ 第一次挂载择到的是上一个文件系统的根，而 I-7.1（根槽有效集非空） 与 I-7.3（环健康性） 全绿（推的，没量）。
- 用户定案，原话在变更史。

**欠**：C79（超级块与 journal 环的放置没人定）：挂载判定做成能强制进入的分支。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:392-404`（整段抄，未转述）**

```markdown
#### 已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根

**定案**：**由记录重建那次发布的根：记录头加「新根段」= 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 188 字节（实例表单元指针照所选根），4096 记录装 67 个点名项。施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义。已定项 14 条 2「严格大于所选根就施加」原样成立。**

**射程**：管「记录在、根槽不在」那一格恢复施加什么；两条指针的宽度随 D19（块指针的结构与宽度预算） 已定项 7 / 11 走。施加一条记录时指针层上其余的事（分配器登记、overlay）不在这一格，归 C284（施加一条记录在指针层上做什么没有定义） 与 D28（挂载期承诺量） 已定项 2。

**依据**：

- E142（第一个事务的干跑）：新根段加进记录头之后，「施加一条记录」在指针层上有了定义，忽略 journal 与查 journal 的恢复结果在一批状态上不同——journal 承重。
- 用户定案，原话在变更史。

**欠**：C284（施加一条记录在指针层上做什么没有定义）。

```

**出处 `.claude/kb/decisions/16-发布语义.md:165-181`（整段抄，未转述）**

```markdown
#### 已定项 7：发布的持久顺序

**定案**：一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 超级块槽；fsync 等根槽持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。**超级块槽在根槽持久之后再更新，每个 checkpoint 一次**，它是发布序列的最后一步、进层 0 枚举，一次发布的段序列因此唯一。**fsync 返回条件多一句**：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回、不向管理员确认回退（做法与次数在已定项 8）。

**射程**：定的是一次发布内部这几步的先后与 fsync 什么时候返回，不定屏障的真机代价、也不定根槽本身的冗余形态（D22（单元原子性怎么合成） 已定项 2）。三样已知边角：

- **「fsync 等根槽持久之后才返回」的射程**：返回之后这一代只由那一个根槽罩着（根槽不镜像）。同一实例里退一代，靠 journal 重放追得上；**每个新实例（每次可写挂载、恢复、切换、回退）的第一个根在下一次发布之前是单点**——那个槽读不出或它所在的盘掉了，恢复退到上一个实例的根，重放按严格前缀停在实例边界（D23（journal 的角色与格式） 已定项 14 的注 1），这次挂载里 fsync 已返回的事务丢，一个故障就够。要不要让新实例先把根写到两块盘上再确认，是已定项 8。
- **屏障口径**：两道 FLUSH + 根槽 FUA = 每次发布三个序点，比 D25（目标负载优先级） 推导的两个多一个——那是第二道屏障的价钱，知情接受。
- **第二道屏障买的是记录流完整性**，不是数据完整性：数据完整性的必要屏障恰一道（根槽之前）；不上第二道有 7 类「根在案而记录缺席」的状态，记录核对器与反向链的输入有洞。用户定案「需要」，两道都上。

**依据**：

- E77（发布的持久顺序）：崩溃子集穷举——不上根槽之前那道屏障，1024 个崩溃状态里 504 个违例；拿掉「重放施加前验证点名单元」这一步，b_rs 臂 63 处静默嫁接，所以它是承重步骤。
- 用户定案，原话在变更史。

**欠**：C26（屏障数从没量过）（真机每次 fsync 的屏障数仍没量）。崩溃点重放 harness 的两条自证用例（摘掉根槽前的屏障必须红、关掉重放验证必须红）随 C76（发布时序没有权威落点） 已还清。

```

**出处 `.claude/kb/decisions/17-实现分层与第三方管道.md:34-53`（整段抄，未转述）**

```markdown
#### 已定项 2：结构等价类按段序列分，今天 1

**定案**：

1. 结构等价类按录制流的段序列分：两条布局线同类 ⟺ 对每一条根槽写路径（发布、空发布、mkfs 种子、实例切换 / 管理员回退——D13（验证路线） 已定项 1 列的三种写者加空发布），两条线录制流的段序列同构：段边界（屏障与 FUA 写切出的段，D13（验证路线） 已定项 4）的位置相同、每段里出现的步骤种类**集合**相同（写单元 / 写 journal 记录 / 根槽 FUA 写 / 超级块槽原地覆写 / 屏障五种）；一步重复几次（设备数、单元数、副本数）是参数不进判据（D13（验证路线） 已定项 10）。比较只在同一路径角色、跨线之间做，一条线内部各路径不互比、不计数。
2. **等价类数 = 已开线的结构等价类数 × 失败模型数，今天 1 × 1 = 1，每开一条线重算**：失败模型数恒 1（已定项 1，合成加的是排序点不是步骤种类），今天已开线只有纯 SSD 一条（D12（目标介质） 已定项 5）。它乘的是 D12（目标介质） 要还的债第 2 / 3 条（每类各跑一遍崩溃点重放、门禁挂钟按等价类数倍算）。
3. 登记位只有一处：[layout/01-first-txn.md](../layout/01-first-txn.md) 八；表里的段序列逐字来自 E142（第一个事务的干跑） 产物的 `name=segments` 行，**这里不复述那几串数**（复述一份就是第二处登记位，而门禁 52 号只比对 layout/01-first-txn 八与产物，对复述的那一份一个字都说不上话）；实例切换 / 管理员回退（含之后的暖机）与抬 F 的空发布写成字节之前标预想，写成字节时按同一办法复核。
4. 每开一条线：跑一遍那条线的每条根槽写路径，把它的 `name=segments` 行登记进那条线的字节表，与已有各类逐路径比对，不同构即 +1。zoned 开线时按这一条数，不按 D22（单元原子性怎么合成） 未定项 6 乙问的答案数。
5. 门禁 = 登记表 + 产物里的 `name=segments` 行（`replay.sh` exact 钉住）+ E142（第一个事务的干跑） 单测 `registered_segment_sequences_match_every_recorded_path`（改任一路径里屏障或 FUA 的位置都红，mkfs 那一行也在内）。

**射程**：mkfs 的崩溃状态仍不在层 0 枚举里（E142（第一个事务的干跑） G19），钉住的只是它的段序列；「今天 1」是单元素集合上的恒等式，任何判据都给 1，条款的重量在第 1、4、5 条。

**依据**：

- E142（第一个事务的干跑）：`name=segments` 四行把每条根槽写路径的段边界与步骤种类写成字节，改任一路径里屏障或 FUA 的位置单测就红——判据落得下来，登记位有唯一出处。
- 三方判决：第一轮 `research/prompts/d17-r1-main-verification.md`（三条腿一致判「按 `CommitStep` 成员集合分」太粗）、第二轮 `research/prompts/d17-r2-main-verification.md`（三条腿都没在候选判据下数出第二条线，打中的措辞与门禁缺口全部并进定案那五条）。
- 主 agent 定案（2026-09-13，按两轮三方论证），用户定向的原话在变更史。

**欠**：C8（门禁范围判不出来）：门禁挂钟乘的正是这个数，今天没有任何东西在拦它膨胀。

```

**出处 `.claude/kb/checks-owed.md:614-619`（整段抄，未转述）**

```markdown
### 2026-09-13（其十七）：C313（FUA 写算不算崩溃段的边界） 还清；C314（回退可以复用被抛弃的根引用的单元） 条款已写；C245（超级块槽的写频率，两条条款说反话） 加枚举归属——用户八问定案
- C313（FUA 写算不算崩溃段的边界）：用户定 FUA 写算段边界，D13（验证路线） 已定项 4 补句，移到已还清。
- C314（回退可以复用被抛弃的根引用的单元）：用户取影子账，D23（journal 的角色与格式） 已定项 14 与 I-7.4（近 K 代块未被复用） 各加一句；harness 检查仍欠。
- C245（超级块槽的写频率，两条条款说反话）：超级块槽写进层 0 枚举，D16（发布语义） 已定项 7 加注。
- 开口条数（`### 已还清` 之前的行数）285 → 284。

```

**出处 `.claude/kb/checks-owed.md:628-633`（整段抄，未转述）**

```markdown
### 2026-09-13（其十四）：新立 C316（提交步骤的登记位有四处且互不相同）；C220（等价类仍是 2 与降级为待证互相欠着） 与 C313（FUA 写算不算崩溃段的边界） 各加一句——D17（实现分层与第三方管道） 已定项 2 第一轮三方论证
- C316（提交步骤的登记位有四处且互不相同）：三条腿一致打中「按成员集合分类」太粗之后，反推腿现查 `CommitStep` 在 Rust 里零命中、kb 里四处口径互不相同，立账。
- C220（等价类仍是 2 与降级为待证互相欠着）：登记的自证在两种门禁形态下恒绿或恒红，门禁形态要改成段序列登记表。
- C313（FUA 写算不算崩溃段的边界）：它成了 D17（实现分层与第三方管道） 已定项 2 的前置。
- 开口条数（`### 已还清` 之前的行数）283 → 284。

```

**出处 `.claude/kb/experiments/77-发布的持久顺序.md:1-7`（整段抄，未转述）**

```markdown
## E77 发布的持久顺序 —— 已跑（2026-09-02，崩溃子集穷举模型，18 单测 / 8 条变异全抓）：**必要屏障恰一道（根槽之前），且重放施加前必须验证点名单元**

一次发布写三类东西：COW 单元、journal 记录、根槽。它们之间的持久顺序全仓只有两句推导
（D25（目标负载优先级）自陈「这是推，不是实测」；D23（journal 的角色与格式）「已经写到盘上」
没说是发出还是持久），记录与根槽之间、单元与记录之间的顺序**零覆盖**（2026-09-02 grep 证实）。
E77（发布的持久顺序）把四种屏障摆法下的全部崩溃子集穷举了一遍。

```

**出处 `.claude/kb/experiments/77-发布的持久顺序.md:8-15`（整段抄，未转述）**

```markdown
### 判据（跑前写死，写在源码头部，跑完没改）

1. 违例 = 根槽已持久而恢复 ≠ 新态或走读失败；或恢复出部分事务；或嫁接了校验失败的单元。
   根槽未持久时新旧两态都合法（fsync 没返回）。
2. 崩溃状态数必须等于闭式（b_all 72 / b_ur 79 / b_rs 513 / b_none 1024），数错整轮作废。
3. 阳性对照：不验证点名单元的重放（naive）在 b_rs 上违例必须恰为 63（= 2⁶ − 1），不红作废。
4. 哪几道屏障必要由数字说了算，不预设结论。

```

**出处 `.claude/kb/experiments/77-发布的持久顺序.md:16-36`（整段抄，未转述）**

```markdown
### 实测（模型：6 单元 + 3 条同事务记录（提交标记在末条）+ 1 根槽；段内任意子集持久）

| 臂（屏障摆法） | 状态数 | 违例（验证重放） | 违例（naive 重放） | 记录流的洞 |
|---|---|---|---|---|
| b_all [单元][记录][根槽] | 72 | **0** | 0 | 0 |
| b_ur [单元][记录+根槽] | 79 | **0** | 0 | **7** |
| b_rs [单元+记录][根槽] | 513 | **0** | **63** | 0 |
| b_none [全自由] | 1024 | **504** | 567 | 448 |

三条结论，每条都有闭式钉着：

1. **数据完整性只需要一道屏障：根槽之前**（b_rs 违例 0、b_none 违例 504 = 根槽在而单元缺的
   全部组合）。这把 D25（目标负载优先级）那句「每次 fsync 2 个持久点」的推导在模型级证实：
   一道 FLUSH 在根槽前 + 根槽自身的 FUA，够了。
2. **单元与记录之间不需要屏障，前提是重放施加前逐项验证点名单元**——拿掉验证（naive），
   b_rs 当场出 63 处静默嫁接。⇒ **「记录点名的每项自带校验和」（D23（journal 的角色与格式）
   已定项 17）不是锦上添花，是这道屏障得以省掉的交换条件**；恢复路径必须真的去验它。
3. **记录→根槽的顺序买的是记录流完整性，不是数据**：b_ur 数据违例 0，但有 7 个状态
   「根槽在案而记录缺席」——记录核对器与反向链（D23（journal 的角色与格式）已定项 8）的输入
   在那些状态里有洞。要不要为此付第二道屏障，是决策不是实验。

```

**出处 `.claude/kb/experiments/77-发布的持久顺序.md:37-40`（整段抄，未转述）**

```markdown
### 失败条款（跑前写死）

判据 2、3 都命中（状态数逐臂等于闭式；naive 恰 63）。未触发作废。

```

**出处 `.claude/kb/experiments/77-发布的持久顺序.md:41-47`（整段抄，未转述）**

```markdown
### 它答不了的

纯枚举模型，文件操作 0 处。不回答：真实设备 FLUSH 是否生效（C6（块层语义假设写错）的射程）、
FUA 与 FLUSH 的代价、多次发布交错与环回绕（E78（重放的起点）的射程）。
「段内任意子集持久」是块层承诺的下界模型（D20（承重面：单元的原子性与自包含）：
「块层从未承诺一个 bio 不会被撕裂」）。

```

**出处 `.claude/kb/experiments/77-发布的持久顺序.md:48-58`（整段抄，未转述）**

```markdown
### 口径与复跑

- 代码 `research/e7-index-bench/src/bin/e77_publish_order.rs`
  （`cargo run --release --bin e77-publish-order`），
  原始输出 `research/results/e77-publish-order-2026-09-02.out`（12 行，收尾行 `emitted=12`）。
- 穷举无随机，5 轮同一个 md5。**18 个单测，8 条变异逐条命中、0 盲区**，
  变异表 `research/mutations/e77_publish_order.tsv`
  （`bash research/scripts/mutate.sh e77-publish-order research/e7-index-bench/src/bin/e77_publish_order.rs research/mutations/e77_publish_order.tsv`）。
- ⚠️ 变异测试先后抓出模型自己的两个盲区并已修复：恢复函数的「成功」曾是自报的
  （审计与被审计同源），走读完备性曾被审计兜底掩住——两处都以「先证明会红」的方式补上。

```

**出处 `.claude/kb/experiments/77-发布的持久顺序.md:59-70`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 4 | 支撑 | 2026-09-19 不受影响：段模型在四种屏障摆法下的状态数逐臂等于闭式，崩溃状态按屏障切段、段内任意子集持久数得对 |
| D16（发布语义） 已定项 7 | 支撑 | 2026-09-20 不受影响：不上根槽之前那道屏障，1024 个崩溃状态里 504 个违例；拿掉重放施加前的点名单元验证，b_rs 臂 63 处静默嫁接——已定项 7 那两句的数出自这里 |
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-09-19 不受影响：拿掉点名单元的验证，单元与记录同段那一臂当场出静默嫁接——前缀口径第四条（施加前逐项验证点名单元）承重 |
| D23（journal 的角色与格式） 已定项 17 | 支撑 | 2026-09-19 不受影响：「点名的每项自带校验和」是省掉单元与记录之间那道屏障的交换条件 |
| D23（journal 的角色与格式） 已定项 8 | 备料 | 2026-09-19 不受影响：记录与根槽同段时有根槽在案而记录缺席的状态，反向链与记录核对器的输入在那里有洞；要不要为此付第二道屏障是决策 |
| D25（目标负载优先级） 已定项 5 | 支撑 | 2026-09-20 不受影响：数据完整性只需要根槽之前一道屏障，D25（目标负载优先级）「每次 fsync 2 个持久点」的推导在模型级成立——屏障口径那一行的推导站得住，真机的数仍未测 |
| D20（承重面：单元的原子性与自包含） | 不影响 | 2026-09-19 不受影响：「段内任意子集持久」取的是块层承诺的下界模型，只引了 D20（承重面：单元的原子性与自包含） 那条事实 |

```

**出处 `.claude/kb/experiments/142-第一个事务的干跑.md:1-29`（整段抄，未转述）**

````markdown
## E142 第一个事务的干跑 —— 已跑（2026-09-18，确定性模型，50 单测 / 75 条变异全抓；量 5 与 `crates/` 实装比对 21/21 全等）
<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 G13 G14 G15 G16 G17 G18 G19 G20 G21 G22 G23 G24 G25 M17 M18 M67 -->

问的是：把 [layout/01-first-txn.md](../layout/01-first-txn.md) 那张字节表**原样写成字节**（mkfs + 一个 3000 字节的文件 + 一次发布，两块盘），
冷启动读回，再按 D13（验证路线） 已定项 4 的层 0 崩溃状态集合逐个恢复——目的不是量性能，
是让代码把「字节表里哪几格写不出来、哪几条已定条款在字节层面互相顶着」逼出来。装置每一处自己补的取法都报成一行 `gap`，共 **25** 行，
其中 **13 行**的正文以「已收口」开头（G1、G2、G3、G5、G6、G7、G12、G13、G15、G16、G17、G20、G23），G10 / G11 那两句里程碑验收 2026-09-13 已改写；
G24 / G25 是第七次跑新记的。**第七次跑收口了三处**：G2（C325（码 2 的 key 宽字段定位它要先知道 k））、G20（C321（码 2 头的条目宽是单值而映射树条目按类两宽））、G23（C324（码 3 打包容器要不要 32768 对齐没写）），三处都不是改个数就完事——
G2 让解析器不再需要调用方先说出 key 宽，G20 让装置删掉自创的「条目宽 0 当变长哨兵」，G23 把落点次序写成条款并逼出槽 50241 那个空洞。
**判决行的七格照旧全过**（`width_mismatches=0`、`write_list_ok=true`、`recover_full_ok=true`、`layer0_states_ok=true`、`layer0_violations=0`、`control_states_ok=true`、`control_violations_ok=true`）：28 个结构的宽度与字节表逐格相等；事务恰 21 条写、2 道屏障、1 道 FUA；冷启动读回逐字节相同、择根 (1, 3)；
层 0 主臂 262165 个状态零违例（枚举吃 mkfs 之后的全部操作：取号、暖机两次空发布、第一个事务）；一块盘不放屏障的阳性对照 2048 个状态里违例恰 1020；
八个坏字节探针与里程碑步 6 的验收逐条相符。**第八格仍是 3**：忽略 journal 与查 journal 的恢复结果在 **3 个状态**上不同——**journal 承重**（D23（journal 的角色与格式） 已定项 15 的新根段给的，第五次跑是 0）。

**复跑命令**（`exact` 模式，与留存产物逐字节比对）：

```bash
cd research && bash scripts/replay.sh E142
# 直接跑装置：
cd research && cargo run --release --bin e142-first-txn-dry-run
```

代码 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`（5087 行：格式常量、地址 newtype、稀疏设备、录制器、崩溃镜像、
每个结构的编解码、mkfs、取号、暖机、事务、恢复、层 0 枚举、探针、第十一次跑新增的手写 SHA-256 与 M_A / M_B 逐字节相减、量 5 装置↔`crates/` 比对），
第十次跑（2026-09-17）的原始输出 `research/results/e142-first-txn-dry-run-2026-09-17-genesis-tree-table-released.out`（93 行，末行 `emitted=93`；第一个文件版本重写树表时把 mkfs 那片第 0 版树表单元释放、链首无锚点时只认 txg + 1 那条之后重跑，与第九次跑的产物 `research/results/e142-first-txn-dry-run-2026-09-16-instance-boundary.out` 只差 `name=accounting` 的 `defer_queue_per_device` 0 → 16384 与 `name=allocation` 的 `mkfs_generation_records` 4 → 2，`name=layer0` 那一行一个字不变；第九次的产物留存，它与第八次跑的产物 `research/results/e142-first-txn-dry-run-2026-09-16-tree-table-200.out` 只差 `name=layer0` 那一行的 `verification_ran` 9 → 6）；
第十一次跑（2026-09-18）的原始输出 `research/results/e142-first-txn-dry-run-2026-09-18-change-count-three.out`（309 行，两段：装置自己 284 行 + `crates/` 只读快照原样 25 行，各自 `name=done` 都对得上；原有 93 行里除 `name=done` 的计数外一字不变，`width` / `segments` 与第十次跑逐字相同），
跑前登记 `research/prompts/e142-preregistration.md`（原登记）与 `research/prompts/e142-r11-prereg.md`（第十一次跑重跑登记，含「修订」三条），变异表 `research/mutations/e142_first_transaction_dry_run.tsv`（75 条），
变异复跑：`research/results/e142_first_transaction_dry_run-mutate-2026-09-16-instance-boundary.log`（第九次跑之后整张表 67 条重跑，含新加的 M67「跨实例边界的记录也施加」：67 条抓到、0 条无效、0 条没红，末行「已还原，基线仍全绿」）、`research/results/e142_first_transaction_dry_run-mutate-2026-09-16-instance-boundary-2.log`（链首规则改成接在所选根自己那条记录之后、产物一个字节不变、再把整张表重跑一遍：同样 67 条抓到、0 条无效、0 条没红，70 行）、`research/results/e142_first_transaction_dry_run-mutate-2026-09-17-genesis-tree-table-released.log`（mkfs 树表释放与链首规则改完、M10 / M54 两条锚点跟着改到当天的写法：67 条抓到、0 条无效、0 条没红，末行「已还原，基线仍全绿」）；
第十一次跑在 M1–M67 之外新增 M68–M75 八条，改动计数字段与量 5 的比较逻辑各占几条：`research/results/e142_first_transaction_dry_run-mutate-2026-09-18-old-file-content-formula.log`（改动计数与量 5 判定逻辑改完、F1 还没查清前，75 条抓到、0 条无效、0 条没红——这一份只对**改内容之前**的文件内容公式成立）、`research/results/e142_first_transaction_dry_run-mutate-2026-09-18-change-count-three.log`（F1 查清、装置文件内容改成 `index % 251` 之后重跑，同样 75 条抓到、0 条无效、0 条没红，末行「已还原，基线仍全绿」——这一份对应现在仓里的源码）。

````

**出处 `.claude/kb/experiments/142-第一个事务的干跑.md:54-72`（整段抄，未转述）**

```markdown
### 被测对象与臂

装置照 [layout/01-first-txn.md](../layout/01-first-txn.md) 零到七各节写字节：两块 **4 GiB** 的盘、超级块槽 0 / 4096、根环 1 / 4 / 7 MiB 各 8 槽、
journal 环 16 MiB 起 768 MiB（D23（journal 的角色与格式） 已定项 19 ③）、单元区从槽 50176 起（D3（空间分配） 已定项 10 ④）。
镜像大小从 1 GiB 改到 4 GiB 是 D23（journal 的角色与格式） 已定项 19 ③ 那句「环 ≤ 设备容量 ÷ 4」逼出来的——1 GiB 的盘装不下默认 768 MiB 的环。
⚠️ **超级块槽宽这一格第七次跑起按 4096 写**（格式常量，不再等于探测到的 `physical_block_size`）：D22（单元原子性怎么合成） 已定项 2 那一格 2026-09-14 由用户要求重开、走了三方论证，
装置按主 agent 的判决先写，同日用户收尾弹窗定案取格式常量 4096（决策变更史 2026-09-14（其五））；它只改超级块槽里的补齐字节数，写请求的条数、次序、落点、屏障位置都不动。根槽不受影响，仍按判定宽度 512 写。
持久顺序照 D16（发布语义） 已定项 7；崩溃状态集合照 D13（验证路线） 已定项 4
（屏障切段、段内任意整写子集、没持久的位置放旧字节、镜像双写按设备各算，FUA 写算段边界）；恢复照 D23（journal 的角色与格式） 已定项 14 的口径
（全环扫描不先信 tail、jsn 严格连续、只施加 (实例代号, checkpoint_txg) 高于根水位的记录、提交标记齐全、施加前逐项验证点名单元），
施加一条记录的语义照 D23（journal 的角色与格式） 已定项 15（换掉所选根的四个字段）；
oracle 照 E77（发布的持久顺序） 判据 1（根槽已持久而恢复不是新态或走读失败即违例；根槽未持久时新旧两态都合法）。

| 臂 | 盘 | 屏障 | 用途 |
|---|---|---|---|
| 主臂 | 2 块 | 单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → 超级块槽轮换 | 层 0 全量枚举，判违例 |
| 阳性对照 | 1 块（区域归属 0 / 0 / 0，w = 1，明知违反 D2（RAID 条带策略） 已定项 6，只作对照） | 一道都不放 | 证明 oracle 分得出「根在而单元不在」 |
| 忽略 journal 的恢复 | 同主臂 | 同主臂 | 逐状态与查 journal 的恢复比 |

```

**出处 `.claude/kb/experiments/142-第一个事务的干跑.md:73-155`（整段抄，未转述）**

````markdown
### 判决（按跑前写死的判据，整行抄自产物）

```text
E7RESULT name=config devices=2 physical_block_bytes=512 file_bytes=3000 fsid=fixed device_bytes=4294967296 journal_ring_bytes=805306368 journal_ring_slots=196608 in_flight_limit=65536 unit_area_start_slot=50176 unit_area_slots=211968 cluster_segment_slots=64 open_cluster_segment_start=50240
E7RESULT name=warm_up publishes=2 writes=10 barriers=4 fua=2 first_transaction_txg=3 last_warm_up_root_txg=2
E7RESULT name=segments path=mkfs operations=13 segments=4+1+1+1+4 closed_form=34 kinds=[unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[superblock_slot×4,barrier]
E7RESULT name=segments path=instance_acquisition operations=2 segments=2 closed_form=4 kinds=[superblock_slot×2]
E7RESULT name=segments path=warm_up operations=14 segments=2+1+2+2+1+2 closed_form=15 kinds=[journal_record×2,barrier×2]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]
E7RESULT name=segments path=transaction operations=23 segments=16+2+1+2 closed_form=65543 kinds=[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]
E7RESULT name=segments path=post_mkfs_stream operations=39 segments=2+2+1+2+2+1+18+2+1+2 closed_form=262165 kinds=[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]
E7RESULT name=mkfs_units instance_table_records=1 instance_table_row_bytes=88 tree_table_entries=0 genesis_root_watermark=11
E7RESULT name=root_record checkpoint_txg=3 instance=1 tree_identifier_watermark=19 rollback_floor=0 record_bytes=4096 back_chain=628216162
E7RESULT name=write_list writes=21 barriers=2 fua=1 named=8 units=t1@50180x32768,t2@50240x16384,t3@50242x32768,t4@50244x16384,t5@50245x16384,t6@50246x16384,t7@50247x16384,t8@50248x16384
E7RESULT name=recover_full outcome=file_read root=1:3 content_matches=true valid_records=3 above_water=0 applied=0 verification_passed=0 mapping_fallbacks=0
E7RESULT name=layer0 arm=settled_two_devices segments=2+2+1+2+2+1+18+2+1+2 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 first_violation=none
E7RESULT name=layer0_fua_not_boundary segments=2+2+3+2+19+2+3 closed_form=524314
E7RESULT name=journal_effect arm=settled_two_devices states=262165 differing_states=3
E7RESULT name=layer0_control arm=one_device_no_barriers writes=11 states=2048 closed_form=2048 violations=1020 expected_violations=1020 root_persisted_states=1024 failed=1020
E7RESULT name=mapping_key_collision same_transaction_data_units=2 distinct_keys=1 mapping_entries_first_transaction=6
E7RESULT name=named_mapping_keys named=8 rebuilt_from_named=8 mapping_entries=6 covered=6
E7RESULT name=accounting entries=15 allocated_bytes_per_device=212992 free_bytes_per_device=3472670720 empty_cluster_segments_per_device=3310 fragmentation_runs_per_device=4 inode_watermark=2 unreclaimable_per_device=0 defer_queue_per_device=16384 pending_delete=0 committed_reservation=0 sequence_all_one=true generation=3
E7RESULT name=allocation records=20 first_slot=50176 last_slot=50248 key_bytes=10 value_bytes=10 mkfs_generation_records=2
E7RESULT name=instances mkfs_instance=0 first_writable_mount_instance=1 mkfs_superblock_generation=1 acquisition_superblock_generation=2 transaction_superblock_generation=5 transaction_superblock_slot=1
E7RESULT name=back_chain records=3 chains=1:0,2:1134114971,3:628216162
E7RESULT name=verdict width_mismatches=0 write_list_ok=true recover_full_ok=true layer0_states_ok=true layer0_violations=0 control_states_ok=true control_violations_ok=true journal_differing_states=3
```

宽度对账那 28 行（`name=width`，全部 `expected == actual`）：

```text
E7RESULT name=width structure=pointer_head expected=50 actual=50
E7RESULT name=width structure=data_unit_header expected=105 actual=105
E7RESULT name=width structure=packed_unit_header expected=107 actual=107
E7RESULT name=width structure=unit_reserved expected=29 actual=29
E7RESULT name=width structure=data_unit_header_with_reserved expected=134 actual=134
E7RESULT name=width structure=packed_unit_header_with_reserved expected=136 actual=136
E7RESULT name=width structure=root_record expected=371 actual=371
E7RESULT name=width structure=tree_table_entry expected=200 actual=200
E7RESULT name=width structure=journal_header expected=307 actual=307
E7RESULT name=width structure=journal_header_ten_fields expected=78 actual=78
E7RESULT name=width structure=journal_new_root_segment expected=188 actual=188
E7RESULT name=width structure=journal_named_entry expected=56 actual=56
E7RESULT name=width structure=journal_named_entries_per_record expected=67 actual=67
E7RESULT name=width structure=data_pointer expected=88 actual=88
E7RESULT name=width structure=node_pointer expected=86 actual=86
E7RESULT name=width structure=instance_table_row expected=88 actual=88
E7RESULT name=width structure=inode_record expected=140 actual=140
E7RESULT name=width structure=inode_internal_entry expected=120 actual=120
E7RESULT name=width structure=extent_leaf_record expected=112 actual=112
E7RESULT name=width structure=allocation_record expected=20 actual=20
E7RESULT name=width structure=allocation_key expected=10 actual=10
E7RESULT name=width structure=allocation_value expected=10 actual=10
E7RESULT name=width structure=accounting_entry expected=34 actual=34
E7RESULT name=width structure=mapping_key expected=27 actual=27
E7RESULT name=width structure=mapping_entry expected=55 actual=55
E7RESULT name=width structure=location_entry expected=14 actual=14
E7RESULT name=width structure=superblock expected=481 actual=481
E7RESULT name=width structure=superblock_slot expected=4096 actual=4096
```

码 2 节点头按 D8（核心索引结构） 已定项 11 是 86 + 2 × key 宽，含 29 字节预留位 115 + 2 × key 宽（`name=index_node_header` 六行）：

```text
E7RESULT name=index_node_header tree=extent header_bytes=134 with_reserved=163
E7RESULT name=index_node_header tree=inode header_bytes=102 with_reserved=131
E7RESULT name=index_node_header tree=allocation header_bytes=106 with_reserved=135
E7RESULT name=index_node_header tree=accounting header_bytes=130 with_reserved=159
E7RESULT name=index_node_header tree=mapping header_bytes=140 with_reserved=169
E7RESULT name=index_node_header tree=tree_table header_bytes=102 with_reserved=131
```

| # | 判据 | 结果 |
|---|---|---|
| 1 | 宽度对账 | 28 行 `width` 全部 `expected == actual`；码 2 头按 D8（核心索引结构） 已定项 11 的 86 + 2 × key 宽算，五棵树加树表六行各自对得上（131 / 163 / 135 / 159 / 169 / 131） |
| 2 | 写清单 | mkfs 11 条写、2 道屏障（两盘各两个超级块槽都种上）；取号 2 条写、0 道屏障；暖机两次空发布 10 条写、4 道屏障、2 道 FUA；事务 21 条写、2 道屏障、1 道 FUA；八个单元的槽号 50180 / 50240 / 50242 / 50244 / 50245 / 50246 / 50247 / 50248 与 D3（空间分配） 已定项 10 ⑤ 的 bump 次序逐个相等，**槽 50241 空着**（t2 占 50240 之后游标停在它，而 t3 是码 3 容器、要起点 32768 对齐） |
| 3 | 读回 | 冷启动读回 3000 字节逐字节相同，择根 (1, 3)；三条记录 (1, 1)、(1, 2)、(1, 3) 都不高于根的水位 3，一条都不施加。记录头 2026-09-14 起带 fsid，把三条记录的 fsid 段全改成别的池之后扫描一条都不认，换成它们自己的 fsid 又全认——后一句证明前一句不是因为校验和坏了 |
| 4 | 层 0 主臂 | 状态数 262165 等于闭式 1 + (2² − 1) × 2 + (2¹ − 1) + (2² − 1) × 2 + (2¹ − 1) + (2¹⁸ − 1) + (2² − 1) + (2¹ − 1) + (2² − 1)（取号那两个超级块写自成一段；暖机第二次的两个超级块写与事务的 16 个单元写之间没有屏障，同一段 18 个）；违例 0。**事务根槽持久的状态只有 4 个**，读得到文件的有 7 个——多出来的 3 个是根槽没持久而事务记录两份都持久、8 个单元都验得过，靠施加记录重建出那次发布的根 |
| 5 | 阳性对照 | 2048 个状态、违例恰 1020 = 2¹⁰ − 2²：根在的 1024 个状态里，8 个单元全在的只有 4 个 ⇒ oracle 确实分得出「根在而单元不在」，不作废 |
| 6 | journal 承不承重 | **3 个状态上结果不同**（第五次跑是 0）。机制：D23（journal 的角色与格式） 已定项 15 给记录头加了新根段 **188** 字节，「施加一条记录」从此在指针层上有定义——换掉所选根的树表单元指针、中央映射树根指针、树 ID 水位与回退下界 F ⇒ 根没持久而记录持久的那几个状态里，恢复能重建出那次发布的根。G7 随之收口 |
| 7 | 探针 | 改坏最新根槽一字节 ⇒ 择回 (1, 2)、施加 jsn 3 那条记录、**照样读到文件**；改坏两份记录 ⇒ 前缀为空、文件照读；改坏数据单元载荷一份 ⇒ 走另一份读到，两份都坏 ⇒ 报校验和错、经映射再试仍失败、不返回数据；改坏数据单元头最后一字节两份 ⇒ 报校验和错、经映射再试仍失败、不返回数据；改坏两份树表 ⇒ 走读失败（报的槽号第七次跑起是 **50248**）；改坏两盘超级块槽 1 ⇒ 择槽 0、文件照读（tail 不承重） |
| 8 | 空白清单 | 25 行，全在「装置逼出来的 25 处空白」那张表里；其中 13 行以「已收口」开头（第七次跑新收 G2 / G20 / G23 三处），G24 / G25 是第七次跑新记的 |
| 9 | 装置↔`crates/` 实装比对（量 5，第十一次跑新加） | `crates/` 侧只读产出 `cargo run -p singlefs-harness --bin first_transaction_region_bytes`（21 行区域清单：8 单元 × 2 盘 + 根记录 1 + journal 记录 × 2 盘 + 超级块 × 2 盘，`region=` 与装置 `descriptive_tag()` 同名同序）。**第一次真跑触发 F1**：`equal=10`、`equal=false` 11 行；诊断报告 `research/prompts/e142-r11-f1-diagnosis.md` 查明 11 处全部同一根因——装置与 `crates/singlefs-harness/src/scenario.rs::first_file_content()` 喂给第一个事务的 3000 字节文件内容公式不一样（装置原来是 `(index*7+3)%251`，`crates/` 是 `index%251`），数据单元载荷不同沿指针链一路改到根，两边各自的校验和逐格核对都算对了（独立按位 CRC32C 核对器两边各 92 格全过），不是哪一边的写路径错。改装置对齐到 `index % 251`（`crates/` 一侧被 QEMU 日志核对器与三份测试用例、层 0 两条流钉着）之后重跑：**`name=impl_bytes_equal_summary regions=21 equal=21 unequal=0`，21/21 全等** |

````

**出处 `.claude/kb/experiments/142-第一个事务的干跑.md:193-204`（整段抄，未转述）**

```markdown
### 它答不了的

1. 层 0 只枚举段内整写子集（D13（验证路线） 已定项 4），撕裂态并进「没持久」；真实 FLUSH / FUA 是否生效归 C6（块层语义假设写错）。
2. 只有一次发布、三条记录：环回绕、实例切换、回退行一个都碰不到。判据 6 的「journal 承重」只对「记录持久而根槽没持久」这一族状态成立，
   而且只验到「换掉根的四个字段之后树走得通」，没有验任何需要树语义的施加。
3. 装置自取的预想值（根记录字段序、取号那一段的屏障、镜像大小、树表头 ID 的来源、记录头 fsid 不符怎么处置）对不对，由 gap 行交给决策，装置不判。
4. 主臂里根槽持久的状态只有 4 个，且都是前两段全持久之后的——oracle 对主臂的判别力只由阳性对照证明，主臂自己没有一个「险些出事」的状态。
5. 装置是一份代码写 mkfs、事务、恢复三边，不是 D13（验证路线） 已定项 5 要的「实现与 checker 各写一份」；它证明的是字节表写得出、读得回，不是任何一条不变量。
6. 量 5 的 21 个区域里，16 个单元（>4096 字节）两边都只抽样头尾各 32 字节配一个 sha256（5 个 ≤4096 字节的区域——根记录、journal 记录、超级块——才逐字节全比）；
   `equal=false` 时 `mismatch_bytes` 只数抽样窗口里差了几个字节，**不是整个区域的差异字节数**（F1 诊断报告核过：第十一次跑触发 F1 那一次，数据单元真实差 2995 字节，产物只报了 3）。
   `sha256` 不等而抽样的头尾恰好都对得上时，这个二进制定位不到第一处差异在哪（`compare_region` 报 `first_diff_offset=unknown_middle_of_region`）；这一次的 11 处不等全部落在头 32 字节窗口内，纯属巧合，不能保证下一次也这样。

```

**出处 `.claude/kb/experiments/142-第一个事务的干跑.md:205-240`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D5（快照 / 空间记账机制） 已定项 8 | 支撑 | 2026-09-20 不受影响：记账树 15 行的 key 与值逐字段写成字节，与 `crates/` 实装比对全等 |
| D5（快照 / 空间记账机制） 已定项 7 | 支撑 | 2026-09-20 不受影响：已分配是 13 个 16 KiB 槽、固定结构一个字节都不在其中，「已分配 = 分配记录树里落点之和」由它坐实 |
| D5（快照 / 空间记账机制） 已定项 9 | 支撑 | 2026-09-20 不受影响：树表七条条目的头 ID 与 `previous_snapshot_txg` 逐字段写出，inode 树写 12、无归属的写 0 |
| D5（快照 / 空间记账机制） 已定项 10 | 支撑 | 2026-09-20 不受影响：记账 key 里不带设备维的设备段 0xFFFF_FFFF、不带树维的树 ID 段 0 逐字节写出 |
| D5（快照 / 空间记账机制） 已定项 11 | 支撑 | 2026-09-20 不受影响：树表第 1 版 7 条与树 ID 水位 19 逐字段写出，deadlist 树 day-1 注册由它坐实 |
| D18（块里携带什么信息） 已定项 2 | 支撑 | 2026-09-19 不受影响：第六次跑写出的六棵码 2 树全部带 key 区间，不带就没有一棵树的头宽算得出来——全部码 2 节点无条件带 |
| D18（块里携带什么信息） 已定项 17 | 备料 | 2026-09-19 不受影响：跑出 G5（32 字节校验和的算法没定），装置先取 SHA-256，已定项 17 定案后改成 CRC32C 4 + 28 零 |
| D18（块里携带什么信息） 已定项 18 | 备料 | 2026-09-19 不受影响：跑出 G1、G2、G13、G15（码 2 头宽、key 宽落点、条目数与条目宽、声明长度与补齐），已定项 18 按它们定偏移表 |
| D22（单元原子性怎么合成） 已定项 16 | 支撑 | 2026-09-19 不受影响：按超级块槽轮换与根环区域公式写出第一个事务的全部固定结构字节，与 `crates/` 实装按 21 个区域比对全等 |
| D22（单元原子性怎么合成） 已定项 7 | 备料 | 2026-09-19 不受影响：装置逼出的 G4（根记录字段序与偏移没写）让已定项 7 补了偏移那一行，C308（自证单元的字段偏移没定） 仍欠 |
| D22（单元原子性怎么合成） 已定项 2 | 不影响 | 2026-09-19 不受影响：超级块槽按格式常量 4096 写、根槽按判定宽度 512 写，照已定项 2 取值 |
| D15（格式冻结政策） 已定项 4 | 支撑 | 2026-09-20 不受影响：超级块 feature bits 那 96 字节按位 0 置 1 写出来，它的空白 G12 随之收口 |
| D15（格式冻结政策） 已定项 2 | 支撑 | 2026-09-20 不受影响：树表单元按条数声明（mkfs 那版 0 条、事务那版 7 条），表长是数据不是编译期常数——开放列表这个形态在写出来的字节上成立 |
| D15（格式冻结政策） 已定项 3 | 支撑 | 2026-09-20 不受影响：单元头那六个宽度都是 8 的相邻字段逐行排着，对调任意两个而全部标量不变是可构造的——spec 只投影标量罩不住字段顺序 |
| D15（格式冻结政策） 已定项 6 | 支撑 | 2026-09-20 不受影响：冻结前清单第 1 项要扫的那张字段表就是它逐行写出又逐行核过的那张，「预想」「半定」两个词有真实对象 |
| D3（空间分配） 已定项 10 | 支撑 | 2026-09-20 不受影响：七个提交内生块的落点表、出生序号与槽 50241 那个空洞是它写出来的字节，字节表与它逐字对得上 |
| D3（空间分配） 已定项 11 | 支撑 | 2026-09-20 不受影响：t5 的条目宽、声明长度与那批记录的排布按跨度段进 value 写出来，产物与字节表对得上 |
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-09-19 不受影响：干跑时装置取「两份镜像任一份合法即在」，改坏一份记录仍读到全部合法记录，「记录在」的口径由此收口 |
| D17（实现分层与第三方管道） 已定项 2 | 支撑 | 2026-09-20 不受影响：`name=segments` 四行把每条根槽写路径的段边界与步骤种类写成字节，改任一路径里屏障或 FUA 的位置单测就红——结构等价类按段序列分这条判据落得下来，登记位有唯一出处 |
| D17（实现分层与第三方管道） 已定项 5 | 支撑 | 2026-09-20 不受影响：第一个事务的全部固定结构字节由一个只对块设备抽象编程的二进制写出来并与 `crates/` 实装逐区域比对全等——不挂载也走得到「正确提交一个事务」 |
| D23（journal 的角色与格式） 已定项 15 | 支撑 | 2026-09-19 不受影响：新根段加进记录头之后「施加一条记录」在指针层上有了定义，忽略与查 journal 的恢复结果在一批状态上不同——journal 承重 |
| D8（核心索引结构） | 备料 | 2026-09-20 不受影响：E142（第一个事务的干跑） 逼出 G1 / G13 / G20 三处空白（码 2 头 = 86 + 2 × key 宽、条目数与条目宽进头、条目宽回到单值），收口落在 D18（块里携带什么信息） 已定项 18 与 D19（块指针的结构与宽度预算） 已定项 10；D8（核心索引结构） 还没按瘦身形态改，分项号等它瘦身完再补 |
| D19（块指针的结构与宽度预算） | 备料 | 2026-09-20 不受影响：E142（第一个事务的干跑） 逼出 G3（映射 key 一律 27、条目一律 55）与 G16（出生序号从 0 起、同一棵树内码 2 与码 3 共用一个计数），两处都由用户定案写回 D19（块指针的结构与宽度预算）；它还没按瘦身形态改，分项号等它瘦身完再补 |
| D16（发布语义） 已定项 8 | 支撑 | 2026-09-20 不受影响：暖机让第一个事务的 checkpoint_txg 从 1 变 3，那两次空发布与其后的字节逐字段写出来、262162 个状态零违例，已定项 8「第一版实付 2 次」那一句的字节由它坐实 |
| D16（发布语义） 已定项 9 | 支撑 | 2026-09-20 不受影响：第一次可写挂载的暖机时树表 0 条、那两次空发布写零个单元，已定项 9「树表 0 条 ⇒ 零单元」那一句的字节由它坐实 |
| D16（发布语义） 已定项 5 | 备料 | 2026-09-20 不受影响：E142（第一个事务的干跑） 逼出 G8 / G18——事务切分纪律让一次带 8 个数据单元的 fsync 至少 8 个事务，与 D23（journal 的角色与格式） 已定项 12 算环余量时用的记录数差 8 倍，账记在 C310（事务切分纪律与记录数口径打架） |
| D13（验证路线） 已定项 4 | 备料 | 2026-09-20 不受影响：E142（第一个事务的干跑） 报出 FUA 算不算段边界两种读法的状态数（主臂 262165、另一读法 524314），C313（FUA 写算不算崩溃段的边界） 收口时用户按它定案；已定项 4 的依据段今天引的是 E77（发布的持久顺序） 的闭式对账，没引 E142（第一个事务的干跑） |
| D12（目标介质） 已定项 1 | 备料 | 2026-09-20 不受影响：已定项 1 逐字要求每套布局各占一个 incompat 位，而 E142（第一个事务的干跑） 跑出 G12——超级块 feature 位全 0、第一条布局线没有位号；收口落在 D15（格式冻结政策） 已定项 4，C183（每套布局一个 incompat 位而盘上差不出字节） 仍欠 |
| D2（RAID 条带策略） 已定项 6 | 不影响 | 2026-09-20 不受影响：阳性对照那条臂 1 块盘、w = 1 明知违反已定项 6，只为证明 oracle 分得出「根在而单元不在」；主臂两块盘照已定项 6 写 |
| D4（校验和位置） 已定项 6 | 不影响 | 2026-09-20 不受影响：只在 G15 那一行当顶着的条款引一句（码 2 的声明长度），收口落在 D18（块里携带什么信息） 已定项 18；已定项 6 定的 extent 声明长度住哪、多宽一个字没动 |
| D25（目标负载优先级） | 不影响 | 2026-09-20 不受影响：只在 G18 那一行当目标负载 12 项事务的出处引一句，判据与结论都不落在它上面 |

```

**出处 `.claude/kb/checks-owed.md:20-20`（整段抄，未转述）**

```markdown
| C6 | 块层语义假设写错 | 块层重排 / FLUSH / FUA 的语义假设本身写错 | 在 QEMU 里用真实 virtio-blk + 写缓存策略跑，**故意去掉一次 FUA**，崩溃测试必须变红；不红说明重排模型没判别力 | QEMU 负载、崩溃点重放 ⚠️ 2026-09-14 虚机档接上（门禁 55 号）：真 virtio 盘（写缓存 write back）上，程序发出的写与 FLUSH 在设备侧（QEMU blklogwrites，来宾之外、带数据）逐项原样到达、次序不变，末尾只多一个关机 FLUSH；盘 0 漏掉「单元 → journal 记录」那道屏障时设备侧比对红在盘 0 第 29 项，走页缓存时红在第 0 项。仍欠「崩溃测试必须变红」那一半：层 0 的 oracle 对漏一道屏障不红——herd7 同日判单元与根槽之间隔着哪一道屏障都够，与 E77（发布的持久顺序） b_ur 臂同形——要等记录核对器；真盘（不经 QEMU）的 FLUSH 语义也不在这一档射程里 ⚠️ 2026-09-14 模型层的「崩溃测试变红」有了：摘掉根槽前那道屏障，层 0 里记录核对器判红 1 个状态、oracle 不红；拿虚机的设备侧日志重建崩溃状态再跑记录核对器那一步还没做 | 三类验证路线共享这份假设，共享的东西没有第二条路径能查 |
```

**出处 `.claude/kb/checks-owed.md:410-410`（整段抄，未转述）**

```markdown
| C313 | FUA 写算不算崩溃段的边界 | 2026-09-13 用户定案：FUA 写算段边界，D13（验证路线） 已定项 4 补句；E142（第一个事务的干跑） 主臂按它枚举（第四次跑 262162 个状态，只数事务那 21 条写时 65543），单测 `fua_not_a_boundary_gives_524311_states` 钉住另一读法给不同的数 | 2026-09-13 |
```

**出处 `.claude/kb/checks-owed.md:409-409`（整段抄，未转述）**

```markdown
| C320 | 小节清单标「不抄」而附录里有，没人核 | `research/scripts/quote-kb.py --checklist` 加反向核对：清单标「不抄」的小节，若它自己的标题行原样出现在附录里同一个源文件的抄录块中（比如被 `--extra 文件:A-B` 整段带进来），判红退出码 6、不写出口文件；比对按源文件收窄，避免「### 已定项」这类多文件同名小标题互相误报。自证三种形态：带进来判红、没带进来是绿、跨文件同名不误报（`--selftest`，门禁 47 号跑）。在 D3（空间分配） 已定项 8 第二轮那份清单上重放：D26（后台整理与放置回收） 已定项 3 与 D3（空间分配） 已定项 9 两节判红 | 2026-09-13 |
```

**出处 `.claude/kb/layout/01-first-txn.md:380-406`（整段抄，未转述）**

```markdown
## 八、根槽写路径的段序列登记表（C316（提交步骤的登记位有四处且互不相同） 的登记位，2026-09-13 立）

**里程碑**：[01-first-txn.md](../milestone/01-first-txn.md) 步 1（mkfs 种根）与步 5（发布：暖机与第一个事务）；恢复那一步不写字节，只吃这张表切出的段。

一条线的提交协议由它全部根槽写路径的录制流段序列定（D13（验证路线） 已定项 4 切段：屏障与 FUA 写切段，段内任意整写子集），等价类按这张表的同构分：段边界的位置相同、每段里出现的步骤种类**集合**相同（D17（实现分层与第三方管道） 已定项 2）；一步重复几次（设备数、单元数）是参数不进判据。别处引提交步骤一律链到这一节，不另抄。步骤种类只有五种：写单元（含码 3 容器、实例表单元）、写 journal 记录、根槽 FUA 写、超级块槽原地覆写、屏障。表里的段序列逐字来自 E142（第一个事务的干跑） 产物的 `name=segments` 五行（`path=mkfs / instance_acquisition / warm_up / transaction / post_mkfs_stream`；取号那一行不是根槽写路径，登记在这里是因为整条流按录制流切、它的两个写落在流的开头一段），单测 `registered_segment_sequences_match_every_recorded_path` 钉住它们；表与产物之间的逐字比对由门禁 52 号做（C316（提交步骤的登记位有四处且互不相同） 2026-09-13 已还清）。

⚠️ **E142（第一个事务的干跑） 第九次跑（2026-09-16）已对**：产物 `research/results/e142-first-txn-dry-run-2026-09-16-instance-boundary.out` 的 `name=segments` 五行与第七次、第八次跑逐字相同（第九次只改恢复的前缀规则，写清单一个字不动）——树表条目加宽只改一个单元里装什么字节，不改写请求的条数、次序与屏障位置；`name=width` 28 行 expected == actual，层 0 仍 262165 个状态零违例。

| 根槽写路径 | 录制流的段序列（每段的写数与种类；「种类」那串是产物 `kinds=` 字段的原样，每段一个步骤种类多重集，段之间用 `\|` 隔开，门禁 52 号逐字比对） | 出处 | 层 0 枚举 |
|---|---|---|---|
| mkfs 种根 | [m1 实例表单元 × 2 盘 + m2 树表单元 × 2 盘 = 4 单元写] 屏障 [m3 根 FUA 区域 0] [m3 根 FUA 区域 1] [m3 根 FUA 区域 2] [m4 超级块槽 × 2 盘 × 2 槽 = 4，都是世代号 1] 屏障 ⇒ `4+1+1+1+4`，13 次操作、34 个崩溃状态，种类 `[unit_write×4,barrier]\|[root_record_fua]\|[root_record_fua]\|[root_record_fua]\|[superblock_slot×4,barrier]` | 一；E142（第一个事务的干跑） `fn mkfs`、产物 `name=segments path=mkfs` | **不在**（G19：装置从 mkfs 之后的池起枚举；段序列由单测钉住） |
| 第一次可写挂载取号（实例代号 0 → 1，不是根槽写路径） | [a1 超级块槽 × 2 盘，世代号 2] ⇒ `2`，2 次操作、4 个崩溃状态，种类 `[superblock_slot×2]`；取号两写之后一道屏障（D23（journal 的角色与格式） 已定项 16 要取号两写之后、本实例第一个非超级块写之前一道完成了的屏障；2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）：`crates/singlefs-core/src/transaction.rs` 的 `acquire_instance` 写完两份超级块自己发一道（`CommitStep::Barrier`），首次挂载上它与暖机第一次空发布开头那道背靠背、登记的段序列不变；第二次以后的挂载靠它把取号与写行那次发布的单元写隔开（2026-09-17 按代码改写）；世代号按 D22（单元原子性怎么合成） 已定项 16 逐盘 + 1，首次挂载写出的是 2 | 一（a1）；D23（journal 的角色与格式） 已定项 16；产物 `name=segments path=instance_acquisition` | 在（第六次跑起，整条流开头那一段） |
| 空发布（暖机，第一版 2 次） | 屏障 [w1 空记录 × 2 盘] 屏障 [w2 根 FUA] [w3 超级块槽 × 2 盘]（第二次同型，w4–w6）⇒ 两次合起来 `2+1+2+2+1+2`，种类 `[journal_record×2,barrier×2]\|[root_record_fua]\|[superblock_slot×2,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[superblock_slot×2]` | 零；D16（发布语义） 已定项 8；产物 `name=segments path=warm_up` | 在（第四次跑起） |
| 普通发布（第一个事务） | [t1–t8 单元 × 2 盘 = 16] 屏障 [t9 记录 × 2 盘] 屏障 [t10 根 FUA] [t11 超级块槽 × 2 盘] ⇒ `16+2+1+2`，种类 `[unit_write×16,barrier]\|[journal_record×2,barrier]\|[root_record_fua]\|[superblock_slot×2]` | 零；D16（发布语义） 已定项 7；产物 `name=segments path=transaction` | 在（E142（第一个事务的干跑） 主臂） |
| 实例切换 / 管理员回退 | 实例切换那一半 2026-09-16 已写成字节，就是「第一次之后的可写挂载（写行）」那一行（切换 = 挂载内做一次恢复再写行，D23（journal 的角色与格式） 已定项 14）；管理员回退那一半 2026-09-17 也写成字节（**装置钉住**，里程碑「第二个事务」步 4，孤立形状从第二条流推得）：[取号超级块槽 × 2 盘，世代号 4] 屏障 [实例表单元（回退行 + 中间实例行）+ 分配记录 + 记账 + 映射 + 树表 = 5 单元 × 2 盘 = 10] 屏障 [记录 × 2 盘，jsn 接在 R_old 那条之后] 屏障 [根 FUA] [超级块槽 × 2 盘] ⇒ 孤立看 `2+10+2+1+2`，与写行那一行同型、多的是内容（回退行带回退位、中间实例行、影子账只住内存不写字节），**之后接新实例的暖机**（这条脚本上一次：txg 9 落盘 0、txg 10 落盘 1）；取号之后那道屏障是 D23（journal 的角色与格式） 已定项 16（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款））（D16（发布语义） 已定项 8：新实例的根覆盖两块盘之前连推空发布，与第一次挂载同型）——与普通发布 + 空发布同型，多的是内容不是步骤 | D23（journal 的角色与格式） 已定项 14；D16（发布语义） 已定项 8 | 不在 |
| 第一次之后的可写挂载（写行） | **装置钉住**（里程碑「第二个事务」步 3 2026-09-16 落地，孤立形状从第二条流推得）：[取号超级块槽 × 2 盘，世代号 3] 屏障 [实例表单元（写行）+ 分配记录 + 记账 + 映射 + 树表 = 5 单元 × 2 盘 = 10] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [超级块槽 × 2 盘] ⇒ 孤立看 `2+10+2+1+2`；整条流里取号的 2 个写与上一次发布的 2 个超级块槽写合成 4 写一段、末尾的 2 个超级块槽写与暖机第一次的 8 个单元写合成 10 写一段（第二条流第 13–17 段 `4+10+2+1+10`）；取号之后那道屏障由取号自己发（D23（journal 的角色与格式） 已定项 16：写行那次发布有单元写，等不到空发布开头那道）；每次可写挂载都写行（实例 0 不写，第一次可写挂载要写的区间是空的），写行那次是新实例的第一次发布，之前不推抬 F 的空发布，元数据走切换预留（D18（块里携带什么信息） 已定项 11、D28（挂载期承诺量） 已定项 3；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方） | D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 16；用例 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 在（第二条流，门禁 54 号） |
| 空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元） | **装置钉住**（步 3 落地，孤立形状从第二条流推得）：[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [超级块槽 × 2 盘] ⇒ 孤立看 `8+2+1+2`，与首次挂载的暖机不同型：记账树已经存在，空发布也重写记账行连带四个固定点单元（D16（发布语义） 已定项 9），这条脚本上 c_max = 4；整条流里 8 个单元写与上一次发布的 2 个超级块槽写合成 10 写一段；实例 2 从 txg 5 起推 2 次（txg 6 落盘 0、txg 7 落盘 1），次数按「本实例的根覆盖全部区域盘」现算、上限 3——D16（发布语义） 已定项 8 只给第一次可写挂载定了常量 2，后续挂载的次数没有条款，实做的取法等用户定 | D16（发布语义） 已定项 8 / 已定项 9；用例 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 在（第二条流，门禁 54 号） |
| 抬 F 的空发布（D16（发布语义） 已定项 1：准入不够时先推空发布抬回退下界） | **装置钉住**（里程碑「第二个事务」步 5 2026-09-17 落地，孤立形状从第二条流推得）：与后续挂载的暖机空发布同型——[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [超级块槽 × 2 盘] ⇒ 孤立看 `8+2+1+2`，根记录的 F 写成目标值、记账行是回收过释放代 ≤ F 之后的账；推到每块盘上都有一条带新 F 的根为止（这条脚本上两次：txg 15 落盘 0、txg 16 落盘 1）；第一版只有测试的强制入口，准入不够的正常触发没做 | D16（发布语义） 已定项 1；用例 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 在（第二条流，门禁 54 号） |
| 只做过 mkfs 的池的可写挂载（取号、零单元写行、零单元暖机）再发第一个文件版本 | **装置钉住**（里程碑「第二个事务」步 3 2026-09-17 落地，2026-09-17 用户定案允许只做过 mkfs 的池可写挂载）：重开之后取号 [超级块槽 × 2 盘] 屏障，树表 0 条 ⇒ 写行那次发布与暖机都写零个单元，与 mkfs 同一个进程里的取号、两次暖机同型，之后第一个文件版本同第一个事务 ⇒ 整条流 `2+2+1+2+2+1+18+2+1+2`，与第一个事务那条流逐段相同、闭式 262165；树表 0 条而要写实例表行、回退到树表 0 条的根都在写之前拒绝 | D16（发布语义） 已定项 9（树表 0 条时空发布写零个单元）；用例 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs` | 与第一个事务那条流的基线镜像、写表与段序列逐项相同（快用例钉住），不另枚举；崩溃状态上重开可写挂载不在层 0 里 |

⚠️ 发布与下一次发布之间没有屏障：上一次发布的超级块槽写与下一次发布的单元写落在同一段（第一个事务的第六次跑里暖机第二次的 2 个超级块写与 16 个单元写合成 18 个写的一段，取号的 2 个超级块写与暖机第一次开头的屏障合成开头一段，整条流 `2+2+1+2+2+1+18+2+1+2`、262165 个状态，种类 `[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[unit_write×16,superblock_slot×2,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]`）；mkfs 末尾有屏障、空发布开头有屏障，那两道接缝是切开的。段序列按整条录制流切，不按路径切，所以登记表按路径给的段序列是「孤立看」的形状，整条流那一行才是层 0 枚举吃的。

⚠️ 第二条流（里程碑「第二个事务」步 0 的固定脚本，2026-09-17 做到发布 E）：取号 → 暖机 × 2 → A → B → 进程退出、重开取号 → 写行 → 暖机 × 2 → C → 进程退出、重开回退到 A 的根、取号 → D（回退行）→ 暖机 × 1 → 覆盖写 × 4 → 抬 F 的空发布 × 2 → E，第二条流的段序列 `2+2+1+2+2+1+18+2+1+18+2+1+4+10+2+1+10+2+1+10+2+1+18+2+1+4+10+2+1+10+2+1+18+2+1+18+2+1+18+2+1+18+2+1+10+2+1+10+2+1+18+2+1+2`、279 次写、2104413 个状态，没有干跑产物，装置钉住：`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` 把这个数组与闭式钉死，门禁 54 号在 release 下全量跑它、52 号核这句与用例里的数组、闭式、写数相符；到 C（26 段、789555）与到 D（33 段、791624）两个前缀各由同一份用例里只跑准备的一条钉住。前 12 段与第一条流相同，第 13 段是 B 的 2 个超级块槽写与重开取号的 2 个超级块槽写合成的 4 写一段（进程退出与重开之间没有屏障，录制流按设备连着记）；第 26 段同型（C 的 2 个超级块槽写与回退取号的 2 个）。

<!-- format-const: WARM_UP_EMPTY_PUBLISHES = 2 -->
<!-- format-const: FIRST_TRANSACTION_TXG = 3 -->

```

**出处 `.claude/kb/experiments/142-第一个事务的干跑.md:176-176`（整段抄，未转述）**

```markdown
| G17 | 已收口（2026-09-13 C313（FUA 写算不算崩溃段的边界） 用户定案：FUA 写算段边界）：装置主臂按它枚举，另一读法（524314）只报数不判 | D13（验证路线） 已定项 4 | C313（FUA 写算不算崩溃段的边界） 已还清 |
```

**出处 `.claude/kb/experiments/142-第一个事务的干跑.md:178-178`（整段抄，未转述）**

```markdown
| G19 | mkfs 种根的 13 次操作（11 写 + 2 屏障，段序列 4+1+1+1+4、34 个崩溃状态）不在层 0 枚举里：装置从 mkfs 之后的池起枚举（取号、暖机与事务），mkfs 的崩溃状态没有任何东西判；段序列另发 `name=segments path=mkfs` 一行、单测钉住 | D13（验证路线） 已定项 1（根槽的三种写者） | C316（提交步骤的登记位有四处且互不相同） |
```
