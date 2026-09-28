**出处 `.claude/kb/decisions/13-验证路线.md:70-84`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。层 0 的扇区取 512 字节；两条层 0 流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条层 0 流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；层 0 的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，层 0 小流里记录核对器报出来）在 `research/prompts/m2-rev-b3a2-implementer-report.md` 第一、三节。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:85-101`（整段抄，未转述）**

```markdown
#### 已定项 5：只共享一份从 kb 生成的常量

**定案**：checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（`.claude/singlefs-ai-sop/rules/machine-first.md`：重复要生成，不能手抄）。**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用） 由门禁 94 号定的判据，形态是 crate 粒度的依赖闭包交集、不是符号级，「格式解析与常量除外」那条例外就是这一项在用）。

- **判定宽度不许从别处拿**：I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 的判据是挂载时探测的 `physical_block_size`，它不在镜像里。checker 自己探一次，再与系统配置记录的 mkfs 时取值比对（D22（单元原子性怎么合成） 已定项 2「mkfs 按池内最大值划、挂载时逐设备复算比对」的 checker 侧形态）。**探不到时报「声明值，未探测」，不许当成探到的**——文件当设备的镜像属于这一类，崩溃点重放的绝大多数镜像都是文件，所以这条不是边角。异构池里判定宽度是一个集合不是一个数（C17（设备几何当全局参数）），checker 要复算 mkfs 那步取最大值的聚合，不然健康的异构池上会假红。
- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
- **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 27 号按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源只有类型（`crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`）、没有接到判定上，I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。

**依据**：
- 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
- 三方对抗：材料 `_checker-sharing-vehicle-background.md`，腿的产出 `research/prompts/checker-sharing-vehicle-*`，腿况在 [verification-build.md](../verification-build.md)——正推腿从 D13（验证路线）、C12（增量语义共用） 推出「常量在例外内、其余禁止」；替代反推腿与本地腿各自构造出同一条：kb 里写错的值共享与各抄都抓不住。
- 用户定案，记在变更史。

**欠**：C94（登记的格式常量与后来的定案对不上）；C17（设备几何当全局参数）；C387（常量模块手写，不是从 kb 生成）；C388（checker 的判定宽度不自己探测）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:126-144`（整段抄，未转述）**

```markdown
#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像

**定案**：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`——持久集合是枚举器给出的这个崩溃状态里哪些写落了盘（层 0 的段内整写子集）；只看盘上字节分不开「写没落、而原地内容恰好相同」与「写落了」，判复用与缺席要拿持久集合判；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-checker-tier/src/crash.rs`，接在层 0 的每个崩溃状态上（门禁 54 号）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 层 0 不覆盖可写挂载：层 0 在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在层 0 的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-checker-tier/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

**依据**：
- 无实验：**依据是类型，不是成本**：「逐字节比对」是二元关系 `R(镜像₁, 镜像₂)`，**没有任何投入能把一元谓词变成二元关系**；I-4.5（已停用·需两镜像） 因同一条门槛退役。
- 「崩溃后镜像」是两份：随 D16（发布语义） 已定项 4 收窄问法那一轮三方论证（材料 `_d16-items134-background.md`）由用户定案，记在变更史。
- 持久集合那一样入参：用户定案，原话在变更史；判别子观测不到的实据在 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第二节按扇区豁免那一行。
- 层 0 不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」），记在变更史；崩溃注入那一侧的实现在 `research/prompts/m2-rev-b3b-implementer-report.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:169-199`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 抽样 | 触发 |
|---|---|---|---|
| 层 0 冒烟 | 固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程） | **不抽样，全量** | checker 档的全量（已定项 15）：用户要求或夜间跑；提交时默认只跑快档 |
| 层 1 常规 | 中等规模负载（写请求数上千） | 项目级固定种子 + **按崩溃点所在子阶段分桶抽样**（意图写入 / 分批推进 / 意图删除 / 纯数据写） | 每次提交 |
| 层 2 全量 | 层 1 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮 | 不抽样 | 夜间 / 发布前 |

**分桶而不是均匀随机**：均匀随机可能把点全抽到同一个子阶段，那正是 E3（意图日志的断电语义）
点名的自欺情形（崩溃点全落在批与批之间）。

**抽样之后敢说什么**：敢说「层 0 负载在全部崩溃点上通过」「层 1 抽到的这 N 个点通过」；
**不敢说**「崩溃一致性在所有可能的崩溃点上成立」，也不敢说「层 1 连续通过等价于层 2 会通过」。

**只能夜间跑的也算准入判据，但只能是异步准入判据**：
**提交前 = 快速档全绿（必要条件）；合并前 = 夜间档全绿（准入条件）。**
夜间档红了，处置不是回退，是「之后所有提交都带着一个未结的红」直到修掉。
**夜间档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个夜间档 bug 的挂钟 ≈ `log2(N) × 一天`，一个不可复现的夜间失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：今天只有层 0（门禁 54 号）；层 0 分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 54 号 `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求或夜间跑；层 1、层 2 在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；层 1 抽样乘的 N 与调度归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束，层 1、层 2 的负载与挂钟都还没有，没有可量的量；分桶防的自欺情形来自一个还没跑的实验的作废条款，不是量到的数。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 平时快档用甲二：用户定案，原话在变更史；层 0 规模三轮判决 `research/prompts/m2-layer0-scale-r3-main-verification.md`。

**欠**：层 1、层 2 在门禁里没有阶段，没有 C 编号。

```

**出处 `.claude/kb/decisions/13-验证路线.md:200-229`（整段抄，未转述）**

```markdown
#### 已定项 10：真正必须乘的 N 不是「布局套数」，是「提交协议结构的等价类数」

**定案**：层 1 抽样先缩问题，再谈调度。崩溃点的定义域是**事务层状态机**，而 `.claude/rules/fs-design.md` 有两句顶层规则：
「一个事务层，所有结构共用」与「唯一不许多样化的是事务层」。所以布局能改的只有两类：

| 差异 | 改变的是 | 判定 |
|---|---|---|
| 节点大小、条带宽度 | 一步写多少字节、某步重复几次 | **参数**，不新增步骤种类 |
| 旋转 vs SSD | seek 代价、IO 调度 | 落在设备层，**不落在事务层协议里** |
| **zoned（ZNS）** | **能不能原地覆盖根指针** | **结构性**：只能追加、根位置漂移，且多出 zone finish/reset |

**落地手段是一个封闭枚举 + 穷尽 match**：`crates/singlefs-core/src/transaction.rs` 的 `CommitStep`，步骤种类的登记位是 [layout/01-first-txn.md](../layout/01-first-txn.md) 八的段序列登记表。
任何布局要引入新的结构性步骤**必须改这个共享定义 ⇒ 编译器强制所有引用处补分支 ⇒
这个 diff 门禁 grep 得到**，可以自动打标「本次引入了新的结构等价类，需为它新开一轮独立层 2 重放」，
**不靠人记得申报**（按 diff 算门禁范围欠在 C8（门禁范围判不出来））。再加 newtype 隔离（`RotationalPtr` / `ZonedPtr`），
把 D12（目标介质） 已定项 2 里「不许共用」那一桶（记账事实的判定路径）变成编译期约束。
⚠️ **它有第二个职责（D17（实现分层与第三方管道） 加的）：它是第三方管道的接入点。**
协议归本工程所有，一条管道要引入新的结构性提交步骤必须往本仓发这个 diff。

**射程**：旋转与 SSD 在提交协议层面同构，由 D12（目标介质） 那一轮三方论证承担；zoned 那一行只说明它至少多出一个等价类。等价关系按录制流的段序列同构、今天几个，归 D17（实现分层与第三方管道） 已定项 2。
⚠️ **它钉住的只是「有哪些步骤」，钉不住「每一步的失败原子性宽度」**——
持久内存与对象存储改变的是后者，枚举接不住（缺口与补法见 D17（实现分层与第三方管道））。
`RotationalPtr` / `ZonedPtr` 这层 newtype 隔离 `crates/` 里没有（`grep -rn "RotationalPtr\|ZonedPtr" crates` 零命中）。

**依据**：
- 无实验：N 是什么由 `.claude/rules/fs-design.md` 那两句顶层规则推出，没有可量的量；等价关系怎么判、今天几个的依据在 D17（实现分层与第三方管道） 已定项 2。
- 三方论证：层 1 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。

**欠**：C8（门禁范围判不出来）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:316-327`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码按「什么时候跑」分两档，各住一个包，与池级 checker 一共三个名字：**harness 档**是 `crates/singlefs-harness`，单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），改了代码随时跑；包内再分轻重：debug 下单条跑到 60 秒及以上的是重，标 `#[ignore]`，单条用时看平常跑的时候量到的、不另外单独量，要跑随时跑，一律经内存包装；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。**checker 档**是 `crates/singlefs-checker-tier`，崩溃态枚举引擎与记录核对器、断点续跑与双机分片、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例，连同 QEMU（门禁 55 号）、herd7（57 号）、crates 变异整表（59 号）、全部实验复跑（87 号）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档里的崩溃点重放（门禁 54 号）分两档，55、57、59、87 号不分：**快档**是 `cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（装置二进制的内联单测不在快档里）（门禁 54 号在整轮门禁与提交时都跑它），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（54 号 `--full`），提交时不默认跑，由用户要求或夜间跑，跑法按已定项 9 分层；GPU 只接校验和那一截（E163（GPU多卡算单元校验和））、要不要接归 D24（后台重活能不能卸给 GPU）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被两档调用。依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（门禁 94 号第 ④ 条判，dev-dependencies 也算）；两档都要的一小段代码宁可各留一份。崩溃枚举用例一律写在 `crates/singlefs-checker-tier/tests/` 里（`research/scripts/crash-case-check.py` 判），重型测试闸按包判：跑到 `singlefs-checker-tier` 的测试就是 checker 档。

**射程**：管 `crates/` 下代码与测试住哪个包、什么时候跑、闸按什么认、三个包怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants.md](../invariants.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑、住哪个包是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；三方 `verification-split-r1`（判决 `research/prompts/verification-split-r1-main-verification.md`）攻的是拆出 `singlefs-checker-tier` 之前的那一版，拆包之后的改动被攻过零轮。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:2-4`（整段抄，未转述）**

```markdown

D24（后台重活能不能卸给 GPU） 管一件事：三个后台重活（批量压缩、全盘 scrub、大规模纠删码重建）要不要卸给 GPU。不管的：CPU 侧算法怎么选在 D9（加密） 已定项 3；scrub 本身的形态在 D9（加密） 已定项 5；后台整理与回收的队列与预算在 D26（后台整理与放置回收）；三格划分本身是 `.claude/rules/fs-design.md` 的记账纪律，D24（后台重活能不能卸给 GPU） 只引用它。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:15-27`（整段抄，未转述）**

```markdown
#### 已定项 1：暂缓，不做

**定案**：**暂缓，不做**——三个候选场景（批量压缩、全盘 scrub、大规模纠删码重建）都不排期。**暂缓不等于否掉**：算力受限那一格（批量压缩）仍然开着，只是不排期。**GPU 这条线不再继续测**，D24（后台重活能不能卸给 GPU） 停在「方向存疑、两侧都没坐实」这个状态——GPU 侧还缺一个真的 AES-GCM 核函数（要 `nvcc`）。**不要把「GPU 赢」当成已定结论去支撑别的决策。**

**射程**：承重的只有「非必需」这一条——三个候选场景全在 `.claude/rules/fs-design.md` 第三格（后台、可续做、非决策路径），不做它不影响任何硬承诺，不像 D1（数据可移动性 / 反向索引） 的缩容、D3（空间分配） 的常驻整理那样是承诺过的能力。**实测那一格不在候选清单里**：判掉的是「纯扫描」，而三个候选的 CPU 侧都要在扫描之上再算，所以标题里那句「一格实测为负收益」要读成「一个与候选无关的格」，不是「候选被实测判负」——那一格的数与判法在已定项 3。

**依据**：

- 用户定案两次（这件事非必需；GPU 这条线不再继续测），原话在变更史。
- 无实验：这条暂缓承重的只有用户定案，而实测判掉的那一格是纯扫描、不在候选清单里，那一格记在已定项 3；候选三格里两格至今无数据（纠删码重建与批量压缩），没有可量的量。

**欠**：无。

```

**出处 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:28-44`（整段抄，未转述）**

```markdown
#### 已定项 2：三条硬约束

**定案**：三条，不满足就不许做——

1. **绝不能成为必需。** 文件系统必须在没有 GPU 时完整工作、且语义完全相同。⇒ **GPU 是可选加速器，一个比特都不许进格式。**
2. **不许进前台路径。** PCIe 往返是几十微秒量级，前台读写延迟吃不起这个；它只能服务已经攒批的后台重活。
3. **它是一个新的失败域，卸出去的结果必须能被 CPU 独立复核。** GPU 算错（驱动 bug、显存 ECC 缺失、算法实现分歧）不能变成静默的数据损坏 ⇒ 与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」同构：**GPU 路径与 CPU 路径必须是两份实现**，抽样复核。

**射程**：第 1 条管的是格式，第 2 条管的是路径，第 3 条管的是验证，三条各自独立，满足不了任何一条就不许做。它们不定「值不值得做」，那是已定项 1 与已定项 3。第 3 条要的两份实现今天一份都没有（D24（后台重活能不能卸给 GPU） 暂缓，不排期）。

**依据**：

- 用户定案（GPU 是可选加速器），原话在变更史。
- 无实验：三条都是政策与验证纪律的直接推论，没有可量的量；第 2 条引的 PCIe 往返量级是公开常识，本工程没测过前台路径上的 GPU 往返。

**欠**：无。

```

**出处 `.claude/kb/milestone/03-third-txn.md:7-10`（整段抄，未转述）**

```markdown
## 出口

第一项的新结构跑通；第六项的多次 COW 历史在新结构上全量崩溃点重放零违例；第五项的 C6（块层语义假设写错）、C26（屏障数从没量过） 收口（用户 2026-09-27 定）。

```

**出处 `.claude/kb/milestone/03-third-txn.md:11-29`（整段抄，未转述）**

```markdown
## 分组与次序

| 组 | 项 |
|---|---|
| 甲　验证装置重做 | 一至五、十一 |
| 乙　文件系统本身 | 六至九 |
| 丙　欠账 | 十 |

| 段 | 做什么 | 为什么排在这 |
|---|---|---|
| 0 开工 | 列第十项的清单 | 清单决定还有哪些活并进来 |
| 1 设计 | 第一项连同第三、四、五、十一项走三方（过程文件格式与存储形态、GPU 上的 checker、屏障判红一起定）；同时单开一轮定第八项的 C118（`deleted_inodes` 树的形态无落点） 格式与回收分批 | 第一项是关键路径，后面的崩溃覆盖都骑在它上面；删除的格式设计不依赖它，可以并行 |
| 2 实现 | 第一项的新结构落地（过程落文件、GPU 算结果），层 0、崩溃枚举用例、崩溃注入、真设备崩溃注入迁进去；同时实现第八项与第七项 | 删除与第七项的代码不依赖新结构，只有崩溃覆盖依赖 |
| 3 覆盖 | 第六项与第八项的崩溃点接进新结构，跑出口 | 出口在这一段满足 |
| 4 优化 | 在新结构上剖析，再做第二项 | 剖析要在新结构上量才有意义 |
| 5 暂定 | 第九项（并发），优先级最低 | 不在出口里；做不到就转给里程碑四 |

**估计**：不含第九项三到四周，推的，没量过（参照：里程碑一建档到出口 4–5 天，里程碑二开工 11 天还没收口）。

```

**出处 `.claude/kb/milestone/03-third-txn.md:30-36`（整段抄，未转述）**

```markdown
## 一　崩溃重放的验证重新设计：公共的、模块的、单独分支的

- **原话**：「重新设计崩溃重放的验证。公共的，模块的 单独分支的。 比如a b c d， abed ，abedf，这些流程，一些部分是可以公用的。但是如果不区分出来，那就容易漏，也容易测不全。」另有同日一句：「量过会漏 说明量的不好，说明我们现在的崩溃重放 还是设计的有问题。」2026-09-27 一句：「我们现在的崩溃放量是分散的 我希望重新组织和设计到一起，然后优化剪枝」。
- **仓里已有的**：崩溃验证今天分散在几处：层 0 全量（门禁 54 号）、`.claude/gate.d/stage-inputs.tsv` 登记的崩溃枚举用例、崩溃注入（`crates/singlefs-checker-tier/src/crash_injection.rs`）、故障注入、坏盘输入，以及 QEMU 真设备（门禁 55 号，只有真实负载、没有崩溃注入；「最终判据」没有门禁阶段覆盖）。层 0 规模第二轮攻方量过，按流列的输入清单漏新加的共用文件（`research/prompts/m2-layer0-scale-r2-main-verification.md` 第二节 M3 那一行）；今天每条流都是从 mkfs 起的一整条写序列，共有的前缀在每条流里各枚举一遍，输入按整条流登记；段模型是「前面的段全持久 + 当前段任意整写子集」（D13（验证路线） 已定项 4）。
- **主 agent 的理解（待用户核）**：把全部流组织成一棵按段共享前缀的树，公共前缀、按模块的中间段、各分支自己的尾段各成一个节点；每个节点的崩溃状态只枚举一次，起点镜像是父节点全持久之后的那一版；每个节点登记它走到的代码模块，改了哪个模块只重跑走到它的节点及其下游；覆盖报告按节点列，没测到的分支看得出来；节点也是双机分片的天然单位。
- **开工前要先定的**：这几处崩溃验证怎么收进同一套设计，真设备上的崩溃注入（补「最终判据」）也收进来；节点怎么划（按段、按操作、按模块）；「节点走到哪些代码模块」怎么机械地登记（不靠手列清单——手列清单正是量过会漏的那一处）；父节点的结束镜像改了之后下游要重跑。

```

**出处 `.claude/kb/milestone/03-third-txn.md:37-42`（整段抄，未转述）**

```markdown
## 二　checker 的算法优化（不是 core 的）

- **原话**：「checker算法的优化。注意这里不是core的算法优化。checker先优化剪枝。」
- **仓里已有的**：用户 2026-09-26 另立的原则（`records/2026-09-24-里程碑二收尾调度.md` 第三节「层 0 规模第三轮判完、用户四问」那一行）：checker 的算法优化可以先于 core，core 不优化算法；实六之后跑一次剖析，量每个崩溃状态的时间花在哪（用户许可的重型测试）。
- **开工前要先定的**：按剖析的数定先优化哪一段；增量枚举（相邻状态只差一次写）、按镜像去重各自的收益——按镜像去重对记录核对器不适用（D13（验证路线） 已定项 7：记录核对器的入参含持久集合）。

```

**出处 `.claude/kb/milestone/03-third-txn.md:43-48`（整段抄，未转述）**

```markdown
## 三　`Vec<bool>` 在验证一侧重新实现

- **原话**：「Vec<bool>、state scheduling调度  在checker中的验证要考虑重新实现，按照core中的实现太重。也不利于之后gpu中的验证。」
- **仓里已有的**：验证一侧的 `Vec<bool>` 在 `crates/singlefs-checker-tier/src/crash.rs`（7 处）与 `crates/singlefs-checker-tier/src/crash_injection.rs`（5 处），`crates/singlefs-checker/src/` 里 0 处（2026-09-26 现查）。
- **开工前要先定的**：验证一侧自己的表示（位图、按段的整数子集编码这类，推的），与第四项 GPU 的数据结构一起设计。

```

**出处 `.claude/kb/milestone/03-third-txn.md:49-54`（整段抄，未转述）**

```markdown
## 四　崩溃放量的 checker 挪到 GPU：过程落文件，一次算出结果

- **原话**：「数据比较核校验部分，我认为可以转移到gpu中，这个作为可选项。默认不生效，但是本机是可以用的，不然显卡是空闲的。当然这部分需要重映射数据结构，需要论证一下代价。测试期间两台主机所有GPU都可以放空。」2026-09-27 改为必做：「这是崩溃放量的实现 是checker的部分 和core无关。」「这个肯定是要实现的，因为一旦到了亿级别的数据验证。花费的时间和实现维护的时间已经不成正比了。所以过程落文件，GPU一次计算得出结果，这样比较好。中间只走过程，过程中的结果还可以hash判定不变。」同日：「如果收益显著的话 我不介意挪动到 里程碑2 来验证」——先做极小规模的计数实验（问题单 `research/prompts/m3-gpu-dedup-r1-forks.md`），收益显著就提前到里程碑二。需要时 5 张卡都可以整卡用（用户同日定）。
- **仓里已有的**：两台主机共 5 张卡，可用合计约 96 GB：本机 RTX 5090 32 GB、RTX 5060 Ti 16 GB；另一台 RTX 5080 16 GB、RTX 5060 Ti 16 GB × 2（2026-09-27 两边 nvidia-smi 现查）；门禁 94 号要求 checker 与实现只共享常量模块。
- **开工前要先定的**：过程文件的格式（内存装置与虚拟机的崩溃落同一种；落「写序列 + 每个状态的子集掩码」而不是整份镜像，推的），与第一项、第四项一起定；过程里哪些中间结果取 hash、hash 不变就复用判定；GPU 上的 checker 与 CPU 上的 checker 怎么对拍（抽样逐格相同，不同就判红），哪一份算规范；测试期间两台主机的卡都放空，跑之前谁负责把本地模型停下、跑完再起来。

```

**出处 `.claude/kb/milestone/03-third-txn.md:91-97`（整段抄，未转述）**

```markdown
## 十一　崩溃放量的过程与结果怎么存、怎么快速读写

- **原话**：「既然说到150亿文件的读写，这里就要给里程碑3 增加一个任务任务 快速的读取和写入这150亿文件。我觉得我们需要使用数据库，KV 存储似乎是一个比较合适的方法。」「这种规模文件的读写和持久化放在文件中 不靠谱。需要解决」（用户 2026-09-27）
- **仓里已有的**：层 0 今天什么都不落盘，每个状态算完只累计进 `Layer0Tally`（`crates/singlefs-checker-tier/src/crash.rs` 的 `evaluate_state_for_versions`），断点续跑只记进度（`crates/singlefs-checker-tier/src/layer0_progress.rs`）；E161（崩溃放量的去重与分段耗时） 跑前登记第四节 4.8、4.9 算过 10¹⁰ 个状态时五卡合计每状态只摊 10.26 字节、显式存每状态掩码装不下；仓里没有任何数据库依赖（2026-09-27 现查各 `Cargo.toml`，sqlite / rocksdb / lmdb / 向量库都 0 命中）。
- **主 agent 的理解（待用户核，推的，没量过）**：要读写的不是 150 亿个文件，是 150 亿个崩溃状态；一个状态由（节点，段号，段内序号）唯一定出，镜像与持久集合都能从序号现算，真要落盘的只有每个状态的判定与去重用的键。裸文件在这个规模上缺五样：一批写一半的原子性、坏字节的校验、多机多卡写入的事务、数据与「哪个输入算出来的」元数据的绑定、按违例查询。形态：嵌入式事务 KV 存储（纯 Rust 的 redb 或 RocksDB，只进 harness），按块存不按状态存——键 =（节点，段号，序号区间，输入指纹），值 = 区间里每个状态的判定指纹编号（不同的判定向量只有几十种，另存一张表）；一块取 2¹⁶ 个状态，150 亿状态约 23 万个键、15–30 GB 值，一块正好是 GPU 的一个批、双机分片的一个单位、门禁 54 号复用判定的一个单位；违例另存一张小表。向量数据库答的是近似最近邻，这里全是精确匹配，近似命中会把两个状态当成一个，不适用。
- **开工前要先定的**：选型挪到里程碑二先做（用户 2026-09-27 定；问题单 `research/prompts/m2-crash-store-r1-forks.md`）；里程碑三接着定块多大、块键里带哪些指纹；两台机器各写各的库还是一个库，怎么合并；违例表与判定向量表的形态；与第一项、第四项一起定。

```

**出处 `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:2-4`（整段抄，未转述）**

```markdown

跑前登记 `research/prompts/e161-preregistration.md`；问题单 `research/prompts/m3-gpu-dedup-r1-forks.md`（G1–G5）。按用户 2026-09-27 定，这一趟只验装置能跑、能用，**G1–G5 各行一律未判（只跑了可行性档）**，登记第十二节 12.2 写明了收窄。入库装置 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`（`feasibility` 模式），确定性部分 13 单测全绿；`crates/mutations.tsv` 末尾追加 12 条变异（M1–M12），由提交时的门禁 59 号跑，本段没跑。

```

**出处 `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:5-11`（整段抄，未转述）**

```markdown
### 这一段做了什么

- 取样域（登记 12.2 F1）：第一条流写数少于 26 的各段全量加全部持久那一个共 37 个状态、两条流的甲二快档全域 54 + 84 个、两个 26 写段（第一条流段 6、第二条流段 9）各取段内序号最小的 4096 个。
- 每个状态上照登记 5.1 做四件事：照跑并分段计时（与 `crates/singlefs-checker-tier/src/crash.rs` 的 `evaluate_state_recording_findings` 同一次序）、在记录用的读者上再跑两遍恢复与两遍 checker（正常一遍、单元候选置空的走树一遍）、单元级检查重放、出 K2-consult、K2-ignore、K2-pair、K4-walk、K4-units、K4-full 六个键。
- 停机条款 S1–S6、登记第七节的全部锚点与 5.3 的阳性对照只在这个小域上做（登记 12.2 F2）。
- 装置端到端跑通：产物 171 行，收尾 `name=done emitted=171`，退出码 0。

```

**出处 `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:12-56`（整段抄，未转述）**

````markdown
### 结果整行抄自产物

产物 `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out`。

停机条款与阳性对照（判决行里全部 `false` 字段的原因见本节末）：

```
E7RESULT name=stop_clause_crates cell=first_small domain_states=37 crates_observed_states=37 compared_states=37 writes_table_differs=false persisted_mismatches=0 clause_S3_holds=true clause_S4_holds=true
E7RESULT name=stop_clause_crates_per_state cell=first_segment_head states=4096 clause_S4_holds=true
E7RESULT name=stop_clause_crates_per_state cell=second_segment_head states=4096 clause_S4_holds=true
E7RESULT name=stop_clause_determinism cell=first_small_and_head control_states=4133 compared_states=4133 recorded_consult_report_differs=0 recorded_ignore_report_differs=0 recorded_verdicts_differ=0 second_run_recovery_outcome_changed=0 second_run_verdict_changed=0 walk_unit_reads_outside_normal_unit_reads=0 clause_S5_holds=true clause_S6_holds=true
E7RESULT name=stop_clause_determinism cell=second_small_and_head control_states=4096 compared_states=4096 recorded_consult_report_differs=0 recorded_ignore_report_differs=0 recorded_verdicts_differ=0 second_run_recovery_outcome_changed=0 second_run_verdict_changed=0 walk_unit_reads_outside_normal_unit_reads=0 clause_S5_holds=true clause_S6_holds=true
E7RESULT name=positive_control_same_state_twice cell=first_small_and_head states=4133 pc2a_recovery_keys_changed=0 pc4a_checker_keys_changed=0 pc2d_recovery_reports_changed=0 pc4d_verdicts_changed=0 pc2a_holds=true pc4a_holds=true pc2d_holds=true pc4d_holds=true
E7RESULT name=positive_control_refeed cell=first_small refed_state_unit_positions=36 distinct_unit_contents_unchanged=true per_state_checks_grew_by_exactly_its_positions=true pc1_holds=true
E7RESULT name=positive_control_device_memory cell=first_segment_head p1_grew=32768 p3_grew=147 p2_grew=7 p4_per_state_grew=48 pc5_holds=true
E7RESULT name=positive_control_timing busy_zone=pool_checker states=1000 recovery_consult_delta_elapsed_ns=-12909772 recovery_ignore_delta_elapsed_ns=-6781508 oracle_delta_elapsed_ns=-748146 pool_checker_delta_elapsed_ns=1842313434 record_checker_delta_elapsed_ns=-3976460 unit_check_replay_delta_elapsed_ns=-72455250 busy_zone_added_at_least_1800ms=true other_zones_changed_under_1000ms=true pc3_zone_holds=true
E7RESULT name=positive_control_remove_one_position cell=first_small_and_head arm=k2_consult control_states=4133 positions=4129 discriminating_positions=1 first_ten=[3:1:4194304:512] inconsistent_without_removal=0 found_at_least_one=true
```

复用臂（只当装置能用的读数，不当判定）：

```
E7RESULT name=reuse_arm cell=first_small arm=k4_walk states=37 inconsistent=3 hits=32 distinct_keys=5 keys_over_states=0.135135 keys_at_or_above_half=false constant_key_inconsistent=28 distinct_outcomes=4 new_keys_peak_after_cold_start=2 new_keys_positive_intervals=4/10 new_keys_final=5 new_keys_last_four=[2,0,1,0] inconsistent_peak_after_cold_start=2 inconsistent_positive_intervals=2/10 inconsistent_final=3 inconsistent_last_four=[0,0,0,0]
E7RESULT name=reuse_arm cell=first_small arm=k4_units states=37 inconsistent=6 hits=33 distinct_keys=4 keys_over_states=0.108108 keys_at_or_above_half=false constant_key_inconsistent=28 distinct_outcomes=4 new_keys_peak_after_cold_start=1 new_keys_positive_intervals=4/10 new_keys_final=4 new_keys_last_four=[1,0,1,0] inconsistent_peak_after_cold_start=2 inconsistent_positive_intervals=4/10 inconsistent_final=6 inconsistent_last_four=[2,1,0,0]
E7RESULT name=reuse_arm cell=second_quick arm=k4_units states=84 inconsistent=3 hits=76 distinct_keys=8 keys_over_states=0.095238 keys_at_or_above_half=false constant_key_inconsistent=66 distinct_outcomes=3 new_keys_peak_after_cold_start=2 new_keys_positive_intervals=5/14 new_keys_final=8 new_keys_last_four=[0,2,0,0] inconsistent_peak_after_cold_start=2 inconsistent_positive_intervals=2/14 inconsistent_final=3 inconsistent_last_four=[0,0,0,0]
```

各格各臂的不一致数与命中数（从产物 `name=reuse_arm` 行数出来，命令见「复跑」）：K2-consult、K2-ignore、K2-pair、K4-full 在五格上不一致数全是 0；K4-walk 在 `first_small` 3、`first_quick` 3，K4-units 在 `first_small` 6、`first_quick` 6、`second_quick` 3，其余格 0。K4-full 在每一格命中数都是 0（每个状态一个键）。

G1 的两个计数与 G3 的粗数：

```
E7RESULT name=g1 cell=first_small states=37 distinct_unit_contents=18 per_state_unit_checks=644 r1=0.027950 r1_at_or_above_half=false unit_read_events=1610 states_without_unit_reads=0 header_scan_events=1380 header_scan_per_state_positions=534 header_scan_distinct_contents=15 ring_record_events=212 ring_record_per_state_positions=106 ring_record_distinct_contents=3 ring_record_r=0.028301 content_collisions=0 q1a_peak_after_cold_start=12 q1a_positive_intervals=3/10 q1a_final=18 q1a_last_four=[12,0,2,0]
E7RESULT name=g3 cell=first_small states=37 state_elapsed_ns=77898270 recovery_consult_elapsed_ns=6218674 recovery_consult_share_ratio=0.079830 recovery_ignore_elapsed_ns=4220050 recovery_ignore_share_ratio=0.054173 oracle_elapsed_ns=57120 oracle_share_ratio=0.000733 pool_checker_elapsed_ns=64510703 pool_checker_share_ratio=0.828140 record_checker_elapsed_ns=2880753 record_checker_share_ratio=0.036980 unit_check_replay_elapsed_ns=35012682 unit_check_replay_share_ratio=0.449466 unit_events_replay_elapsed_ns=111501157 unit_events_share_ratio=1.431368 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=29498021 walk_checker_elapsed_ns=69131624 walk_checker_of_checker_share_ratio=1.071630 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.108108
```

五格上池级 checker 占每状态总时长 0.74–0.91，两遍恢复合起来 0.07–0.17，记录核对器 0.02–0.08，oracle 不到 0.003（各格 `name=g3` 行）。Q3g（逐次单元读重放 + 头扫描按位 CRC + 记录检查）在五格上都大于每状态总时长（`unit_events_share_ratio` 1.32–1.62）；`first_small` 与 `first_quick` 上 Q3f 大于 checker 时长的状态占 10.8% 与 22.2%，超过 V3 的 1%，这两格的 Q3f–Q3i 按 V3 作废。

判决行（第 170 行）里 12 个 `false` 字段逐个说明，都在登记预期之内，不是装置坏：

- `first_segment_head_k2_consult_constant_key_separates=false`、`first_segment_head_k2_ignore_constant_key_separates=false`、`first_segment_head_k4_full_constant_key_separates=false`、`second_segment_head_k2_consult_constant_key_separates=false`、`second_segment_head_k2_ignore_constant_key_separates=false`、`second_segment_head_k4_full_constant_key_separates=false`：两个 26 写段的段头 4096 个状态结局与判定只有一种（各格 `distinct_outcomes=1`），键换成常数也分不出不同，PC2b / PC4b 在这两格测不了；同一臂在 `first_small`、`first_quick`、`second_quick` 上都是 `true`。
- `first_k4_full_pc2c_pc4c_found_a_position=false`：第一条流对照样本上 K4-full 每个状态一个键（命中 0），去掉任何一个位置都不出现同键异判，PC4c 找不到判别位置；按登记 11.1 表下一句，这不作废，只是这条臂的「不一致数 = 0」不拿来关 G4。
- `second_k2_consult_pc2c_pc4c_found_a_position=false`、`second_k2_ignore_pc2c_pc4c_found_a_position=false`、`second_k4_walk_pc2c_pc4c_found_a_position=false`、`second_k4_units_pc2c_pc4c_found_a_position=false`、`second_k4_full_pc2c_pc4c_found_a_position=false`：第二条流的对照样本只有段 9 的段头 4096 个状态（可行性档不含第二条流段 10–22），结局只有一种，去掉哪个位置都造不出不一致。
- `g1`–`g5` 五个字段写 `not_judged_feasibility_only`：可行性档不判 G 行（登记 12.2 F3）。

````

**出处 `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:57-67`（整段抄，未转述）**

````markdown
### 复跑

复跑入口是 `research/scripts/replay.sh` 里的驱动 `driver_e161_feasibility`（线程数环境变量设 10、内存上限 16G、只留 `E7RESULT` 行；计时字段以 `_elapsed_ns` 或 `_ratio` 结尾，按 `timing` 比结构）。复跑与直接跑各一条命令，10 线程约 3 分钟：

```
bash research/scripts/replay.sh E161
E161_THREADS=10 bash research/scripts/run-with-memory-cap.sh 16G cargo run --release --bin e161_crash_state_dedup_and_time_split -p singlefs-harness -- feasibility | grep '^E7RESULT '
```

各臂不一致数与命中数的数法：`grep 'name=reuse_arm' research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out | awk '{print $3, $4, $6, $7}'`。

````

**出处 `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:68-79`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | **装置自己的枚举计划** | 本文件自己的撕裂镜像表与混合进制计划，逐段把持久集合真生成出来数（取样段里逐个数） | `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:1172` `fn persisted_of`、`:2855` `fn count_distinct_persisted_sets` | 录制流写表与段（`writes_and_segments` 的输出）、基镜像、D13（验证路线） 已定项 4 的原地覆写三条判据 |
| 2 | **crates 的枚举计划** | crates 私有的 `Layer0StatePlan` 与 `layer0_state_count_with_torn_in_place_overwrites`，按段枚举交给观察者 | `crates/singlefs-checker-tier/src/crash.rs:2838` `fn persisted_writes_of_state`、`crates/singlefs-checker-tier/src/crash.rs:2692` `fn layer0_state_count_with_torn_in_place_overwrites` | 同一份录制流写表与段、基镜像、同一条 D13（验证路线） 已定项 4 |
| 3 | **登记第四节的闭式** | 3^m · 2^(n−m) − 1 逐段 | `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:1219` `fn formula_segment_state_count` | 段长与原地覆写个数（取自路径 1 的判定） |
| 4 | **装置照跑的计数** | 本文件按 `evaluate_state_recording_findings` 的次序逐步调公开函数，自己记 `Layer0Tally` | `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:2011` `fn evaluate_baseline` | crates 的 `recover`、`check_pool_image`、`check_records`、`oracle_violation_for_versions` |
| 5 | **crates 照跑的计数** | crates 的按段枚举或单状态入口 `evaluate_state_for_versions` | `crates/singlefs-checker-tier/src/crash.rs:2052` `fn evaluate_state_recording_findings` | 同路径 4 的四个公开函数 |

跨路径的比对断言：`segment_count_checks`（`:2907`）逐段比路径 1、2、3，产物 `name=anchor_segment` 行报 `three_agree`；`compare_with_the_crates`（`:2983`）逐状态比路径 1、2 交出的持久集合与写表；`tally_text`（`:2514`）拼出路径 4、5 的计数逐字比。⚠️ 路径 4、5 调的是同一组公开函数，它们相等只证明装置的调用次序与计数口径没抄错，不证明恢复或 checker 本身对；路径 1、2 共用同一份写表与同一条原地覆写判据，那条判据写错两边一起错。

```

**出处 `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:80-87`（整段抄，未转述）**

```markdown
### 它答不了的

- **G1–G5 每一行都未判**：只跑了可行性档，比值、不一致数、占比、字节数都只说明装置能出这些数。登记 5.5 的第一段（第二条流段 10–22 全量 144 万个状态、两个 26 写段按步长 2311 取样）、第二、三段都没跑；第八节几何敏感性（甲二快档对全量、线程数 1、掩码宽）、G3 的加权估计与 G5 的外推行这一趟不跑。
- **K4-walk、K4-units 在小域上出现了不一致**（「结果整行抄自产物」一节里的三行 `name=reuse_arm`）：这是登记预期会看到的那一类（第四节 4.12 说判定还读 journal 与扫描方向，推不出同键同判），但可行性档不据此判 G4，也没跑 Q4d（按不变量分的诊断）。
- **G3 的上界 Q3g 在这个装置里算大了**：它按位算头扫描的 CRC 与每次重读的 U，重放总时长超过每状态总时长；`first_small`、`first_quick` 两格 V3 触发。要判 G3 之前得先查重放口径是不是比 checker 自己做的多做了工作。
- **PC2b / PC4b 在两个段头格测不了**：段头 4096 个状态结局只有一种。
- **12 条变异没跑**：`crates/mutations.tsv` 的行归提交时门禁 59 号。

```

**出处 `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:88-95`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 4 | 备料 | 2026-09-27 不受影响：装置搬进 checker 档包 `crates/singlefs-checker-tier`（验证两档拆分），只改路径与包名，结论、数与产物不变；2026-09-27 不受影响：A1 三方逐段相等、S3 在小域上与 crates 逐状态相同，只说明装置与实现的枚举域一致；可行性档不判任何 G 行 |
| D13（验证路线） 已定项 7 | 备料 | 2026-09-27 不受影响：装置搬进 checker 档包 `crates/singlefs-checker-tier`（验证两档拆分），只改路径与包名，结论、数与产物不变；2026-09-27 不受影响：记录核对器照跑计数与 crates 相同（S4），可行性档不判 |
| D24（后台重活能不能卸给 GPU） | 不影响 | 2026-09-27 不受影响：装置搬进 checker 档包 `crates/singlefs-checker-tier`（验证两档拆分），只改路径与包名，结论、数与产物不变；2026-09-27 不受影响：D24（后台重活能不能卸给 GPU） 管的是批量压缩、scrub、纠删码重建三个后台重活，E161（崩溃放量的去重与分段耗时） 问的是崩溃放量里 checker 的工作量，可行性档也不判 |

```

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:2-8`（整段抄，未转述）**

```markdown

<!-- doc-lint:not-numbers V15 -->

跑前登记 `research/prompts/e162-preregistration.md`；问题单 `research/prompts/m2-crash-store-r1-forks.md`（S1、S2、S3）。按用户 2026-09-27 定案（登记第十二节修订六），2026-09-27 先跑了一版只验能跑、能用的可行性档（S1 每条臂杀 20 次、S2 每条臂写 10⁴ 块），S1、S2 都记「未判（只跑了可行性档）」，S1-large、四个几何取样点（G-bs10、G-batch16、G-sparse、G-random）与 S3 未跑。同一天随后续跑了 S1 的够判档（第二段）：四条臂各按登记原定的 200 次杀主格全跑完，够判条件（第六节 6.1、11.1 V1–V6）逐条过闸，**S1 这一行现在够判：四条臂在这 200 次杀里都没有翻**（丢块、坏块、确认/未确认读报错、幽灵块、打不开全部 0 次）。S2、S3 这一段仍未跑，见「S1 够判档（2026-09-27，第二段）」一节。装置写在 `research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs`（独立手写模型，不进 `crates/`，不碰 `crates/singlefs-core`、`crates/singlefs-checker`），同一天早些时候 23 单测全绿（三遍，经 `run-with-memory-cap.sh` 4G）；下午续跑 S1 够判档之前又重跑了一次同一条单测命令，仍 23 passed / 0 failed（`BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` 解开了上一版页面写的 `librocksdb-sys` 构建卡点，见「它答不了的」）；变异表 `research/mutations/e162_crash_verdict_block_store.tsv` 13 条（`wc -l` 现查，行数未变，源码本身这一段没有改动），同一天早些时候跑过 `mutate.sh` 全部**抓到**（无效 0、没红 0、内存撞顶 0、超时 0），已还原、基线仍全绿；这一段没有再改源码，没有重跑变异表。

四条被测臂：F（加固的文件，临时文件 + fsync + 改名 + 目录 fsync）、R1（redb，`set_quick_repair(true)`）、R0（redb，默认修复）、K（RocksDB，同步写 + WAL）。

```

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:9-14`（整段抄，未转述）**

```markdown
### 这一段做了什么

- **S1**：每条臂各跑杀点计划前 20 行（18 次「写中途」+ 2 次「打开中途」，种子 `seed_kill=0xe162000000000003` 写死），每次杀后重开、逐块核（Q1a–Q1f）。阳性对照 PC-L、PC-S、PC-P、PC-O 四条臂都跑；PC-T（半截文件）按登记 5.3 只定义给 F。
- **S2**：每条臂各写 10⁴ 块（`states_per_block=65536`），报持续写入速率（Q2a）、关库重开后按块键随机读的冷/热时延（Q2d）、占用与写入字节之比（Q2e）。阳性对照 PC-rate、PC-read、PC-space 都跑。S1-large、G-bs10、G-batch16、G-sparse、G-random、S3 按修订六这次可行性档不跑。
- **anchors**：登记第七节 7.2 的 22 个独立算出的锚点（覆盖 SplitMix64 参考值、CRC-32C 标准校验值、块值生成、块键编码、置换、杀点计划、算术锚点等，逐项按跑前登记 13.3 锚点脚本的原样输出钉死）逐项与装置启动自检比对。

```

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:15-71`（整段抄，未转述）**

````markdown
### 结果整行抄自产物

**S1 判决行**（`grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-feasibility.out`）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:73:E7RESULT name=verdict part=s1 arm=F tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=true commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:51:E7RESULT name=verdict part=s1 arm=R1 tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:51:E7RESULT name=verdict part=s1 arm=R0 tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:51:E7RESULT name=verdict part=s1 arm=K tier=feasibility judgement=not_judged_feasibility_tier_only kill_cycles=20 kills_inside_commit=18 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=not_applicable planned_confirmations=747 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
```

**S1 阳性对照行**（四条臂各自的 PC-S / PC-P / PC-O / PC-L；F 另有 PC-T）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=F open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=F open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=F open_failed=true opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=0 ok=true open_error=file:_FORMAT_不对
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=F write_path=file-lazy-rename write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=14 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-feasibility.out:51:E7RESULT name=positive_control control=PC-T arm=F write_kills=18 write_kills_with_read_error=16 q1b=0 extended_to_200=false ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=R1 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=R1 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=R1 open_failed=true opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=0 ok=true open_error=redb:_I/O_error:_Not_a_redb_database:_magic_number_mismatch
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=R1 write_path=redb-durability-none write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=18 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=R0 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=R0 open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=R0 open_failed=true opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=0 ok=true open_error=redb:_I/O_error:_Not_a_redb_database:_magic_number_mismatch
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=R0 write_path=redb-durability-none write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=18 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:5:E7RESULT name=positive_control control=PC-S arm=K open_failed=false opened_after_repair=not_applicable q1a=0 q1b=1 q1b_blocks=7 q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:6:E7RESULT name=positive_control control=PC-P arm=K open_failed=false opened_after_repair=not_applicable q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=1 enumerated_keys=65 ok=true
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:7:E7RESULT name=positive_control control=PC-O arm=K open_failed=true opened_after_repair=yes q1a=0 q1b=0 q1b_blocks=- q1c=0 q1d=0 q1e=0 enumerated_keys=64 ok=true open_error=rocksdb:_IO_error:_No_such_file_or_directory:_While_opening_a_file_for_sequentially_reading:_/tmp/e162-3127058/pc-o/MANIFEST-999999:_No_such_file_or_directory
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-feasibility.out:29:E7RESULT name=positive_control control=PC-L arm=K write_path=rocksdb-no-wal write_kills=18 write_kills_with_q1a=18 write_kills_with_new_q1a=18 ok=true
```

四条臂的 PC-S/PC-P/PC-O 除 `open_error` 文案不同外逐字段相同，都 `ok=true`；K 的 PC-O 另报 `opened_after_repair=yes`（`DB::repair` 修复后能重新打开），F/R1/R0 没有这个字段（登记只对 K 定义了修复动作）。

**S2 判决行**：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s2-F-feasibility.out:41:E7RESULT name=verdict part=s2 arm=F tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33530296.2 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R1-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R1 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33593904.5 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R0-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R0 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=19339984.1 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-K-feasibility.out:41:E7RESULT name=verdict part=s2 arm=K tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=48638773.4 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
```

**S2 持续写入速率、随机读时延与占用比**（`name=throughput`、`name=random_read`、`name=occupancy_summary`，各产物第 11、39、40、37 行）：

| 臂 | 持续写入速率（状态/秒） | 提交时延中位/p99/最大（µs） | 冷读中位/p99（µs） | 热读中位/p99（µs） | 占用峰值比 `peak_ratio_high` | 关库后占用比 `after_close_ratio` |
|---|---|---|---|---|---|---|
| F | 33 530 296.2 | 1529 / 5036 / 583066 | 321 / 399 | 123 / 144 | 1.0639 | 1.0625 |
| R1 | 33 593 904.5 | 1550 / 4352 / 115139 | 230 / 352 | 5 / 8 | 2.0029 | 2.0021 |
| R0 | 19 339 984.1 | 903 / 8299 / 2558476 | 229 / 366 | 5 / 11 | 2.0039 | 2.0017 |
| K | 48 638 773.4 | 804 / 1293 / 1733112 | 157 / 367 | 12 / 21 | 2.1495 | 1.0013 |

四条臂的 `q2e_flips_at_1_9e10`、`q2e_flips_at_1_5e10`、`q2e_flips_at_1_9e10_using_low` 都是 `false`：按这次可行性档的占用比乘 1.9×10¹⁰ 或 1.5×10¹⁰ 字节都还落在开跑时 `df -B1` 报的可用字节内（F 可用 1 979 726 131 200，R1 1 979 601 149 952，R0 1 979 590 852 608，K 1 980 708 077 568）。R1、R0 关库时占用约是写入字节的 2 倍（redb 两条形态接近，`peak_ratio_high` 2.0029 / 2.0039）；K 写入过程中峰值到 2.1495，但关库后掉到 1.0013（RocksDB 的合并把占用压回接近写入字节）；F 全程稳定在约 1.06。这些数只是这次可行性档 10⁴ 块、`samples=10` 的单次裸读数，未重复跑、未判定，不当最终结论；持续写入速率之间的差异（R0 明显低于其余三臂）同理。

**anchors 判决**：`research/results/e162-crash-verdict-block-store-2026-09-27-anchors.out` 第 26 行 `E7RESULT name=verdict part=anchors anchor_mismatches=0`，第 27 行 `E7RESULT name=done emitted=27`；22 个 `name=anchor` 行全部 `matches_registration=true`。

````

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:72-83`（整段抄，未转述）**

```markdown
### 判决行点名

`grep -n 'name=verdict' <产物>` 列出的判决行里，没有任何字段取值是布尔 `false`；出现的非 `true`、非零字段全部逐个如下（值全部落在登记第十二节修订六划定的这次可行性档范围内，是预期收窄，不是装置坏）：

- `judgement=not_judged_feasibility_tier_only`（8 份 S1/S2 产物全部）：这是登记修订六要求的措辞——用户 2026-09-27 定这次只验能跑、能用，S1、S2 都不据这次可行性档的数据下够判结论，字段名字面像「没有判定」，实际是设计如此。
- `pc_t_ok=not_applicable`（S1 的 R1、R0、K 三份）：PC-T（半截文件对照）按登记 5.3 只定义给 F-direct 这条候选，R1/R0/K 没有这个对照本来就该是 `not_applicable`，不是没跑。
- `v6_kills_inside_commit_ok=not_applicable`（S1 全部 4 份）：V6 的门槛是对 180 次「写中途」的杀写的（登记 11.1 作废条款 V6），可行性档只跑 20 次杀（18 次写中途），源码按 tier 直接给 `not_applicable`，与登记修订六「V6…可行性档只报杀在提交中间的次数、不判」一致。
- `s1_large=not_run_feasibility_tier`（S2 全部 4 份）：登记修订六写明这次可行性档 S1-large 不跑，字段照实报「未跑」，预期内。
- `q2a_verdict=pending_e161`（S2 全部 4 份）：Q2a 的门槛 R_AB 要用 E161（崩溃放量的去重与分段耗时） 交回的 t_state，E161（崩溃放量的去重与分段耗时） 同一天也只交了可行性档（见「它答不了的」），门槛还没有正式值可代入，字段照实报「等 E161（崩溃放量的去重与分段耗时）」。

`q1a`–`q1f`（S1，8 份中的 4 份 verdict 行）全部为 0：这次可行性档 20 次杀里没有丢块、静默坏块、确认/未确认读报错、幽灵块、打不开。`anchor_mismatches=0`：22 个锚点全部对上登记。判决行里没有任何名字表示违例、不匹配、歧义或失败的整数计数取正值。

```

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:84-104`（整段抄，未转述）**

````markdown
### 复跑

`replay.sh` 只登记了确定性的 anchors（S1、S2 带杀点与计时，两次跑本来就不同，不登记逐字节比对）：

```
bash research/scripts/replay.sh E162
```
```
E162  @driver_e162_anchors     字节一致 e162-crash-verdict-block-store-2026-09-27-anchors.out
```

直接重跑 anchors、S1、S2 的可行性档（各自独立一条命令，`<臂>` 取 `F`、`R1`、`R0`、`K` 之一；S1、S2 每次都会新起子进程杀、写，产物带时序噪声，与已入库的产物不逐字节比）：

```
bash research/scripts/run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- anchors
bash research/scripts/run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s1 <臂> feasibility
bash research/scripts/run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s2 <臂> feasibility
```

单测：`cd research && bash research/scripts/run-with-memory-cap.sh 4G cargo test --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store`（同一天早些时候 23 passed / 0 failed；本页写作时重跑这条命令被 `librocksdb-sys` 的构建环境挡住，见「它答不了的」）。

````

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:105-177`（整段抄，未转述）**

````markdown
### S1 够判档（2026-09-27，第二段）

跑前登记第十二节修订六只批了可行性档；这一段按主 agent 派发（继承用户 2026-09-27 定案「可以跑 可以用 然后把 验证交给 Milestone 2 remaining tasks 去」，来历 `records/2026-09-24-里程碑二收尾调度.md` 第三节）把 S1 从 20 次杀补到登记原定的 200 次，S1 的够判条件（第五节 5.5、第六节 6.1、第十一节 11.1 V1–V6）没有改一个字；S2、S3 这一段没有跑，留给机器空闲的另一段。

**⚠ 登记补记缺口（执行员的疏漏）**：这一段跑产物之前没有先往登记（`research/prompts/e162-preregistration.md` 第十二节）追加「修订七」记录这一段的收窄范围（按纪律该在装置跑之前写、写完再跑）。这一条只把可行性档的样本量放大到登记原定的值、第六、七、九节的判据一个字没动，不影响下面的判定；产物已经跑出来之后按纪律不许再改登记，这处缺口交主 agent 处置（补记还是接受缺口）。

#### 这一段做了什么

- **S1（够判档）**：四条臂各按登记原定的杀点计划跑满 200 次（`registered` 档，不带 `feasibility` 参数，与源码 `CRASH_MAIN_KILL_CYCLES=200` 对应），逐次杀后重开、逐块核（Q1a–Q1f）；阳性对照 PC-L、PC-S、PC-P、PC-O（F 另有 PC-T）全部重新跑过一遍新库。跑之前重跑了一次单测（见「复跑」），没有改动源码，没有重跑变异表。
- **S2、S3**：这一段未跑。

#### 结果整行抄自产物

**S1 判决行**（`grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out:258:E7RESULT name=verdict part=s1 arm=F tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=true commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out:235:E7RESULT name=verdict part=s1 arm=R1 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=179 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out:233:E7RESULT name=verdict part=s1 arm=R0 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=178 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out:234:E7RESULT name=verdict part=s1 arm=K tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
```

**S1 主格逐项汇总行**（`grep -n 'name=kill_campaign_summary campaign=S1-main' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`，200 次杀的累计计数与重开用时）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out:257:E7RESULT name=kill_campaign_summary campaign=S1-main arm=F write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=180 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=2171 reopen_median_microseconds=1100 reopen_last_microseconds=2171 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out:234:E7RESULT name=kill_campaign_summary campaign=S1-main arm=R1 write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=179 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=1407 reopen_median_microseconds=1015 reopen_last_microseconds=439 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out:232:E7RESULT name=kill_campaign_summary campaign=S1-main arm=R0 write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=178 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=399403 reopen_median_microseconds=124691 reopen_last_microseconds=472 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out:233:E7RESULT name=kill_campaign_summary campaign=S1-main arm=K write_path=registered cycles_verified=200 q1a_lost=0 q1b_corrupted=0 q1c_confirmed_read_errors=0 q1d_unconfirmed_read_errors=0 q1e_phantoms=0 q1f_open_failures=0 q1f_timeouts=0 repaired_openings=0 q1g_kills_inside_commit=180 write_phase_kills=180 confirmations_before_write_kills=6178 write_kills_with_loss=0 write_kills_with_new_loss=0 write_kills_with_read_error=0 reopen_peak_microseconds=3271087 reopen_median_microseconds=11411 reopen_last_microseconds=5818 reopen_over_60s=0 rises=- first_rise_cycle=- stopped_after_unopenable=false assertion_failures=-
```

四条臂的 `cycles_verified=200`（够判条件要的次数）、`q1a_lost`–`q1f_timeouts` 与 `repaired_openings` 全部 0；`q1g_kills_inside_commit`（180/179/178/180）都 ≥ 100（V6 门槛，180 次写中途的杀）、`write_phase_kills=180`、`confirmations_before_write_kills=6178`，与登记 A8 锚点「写中途那 180 行 n_i 之和 6178」一致。阳性对照四条臂的 PC-S、PC-P、PC-O、PC-L（F 另有 PC-T）都 `ok=true`：`q1b=1` 落在第 7 块、`q1e=1`、`open_failed=true`（K 另报 `opened_after_repair=yes`）、PC-T 这次 `write_kills_with_read_error=18`（可行性档 20 次杀里是 16，这次 200 次杀的前 20 行种子相同，`q1d`/`q1c` 累计口径不同，18/18 与登记 V5「18 次写中途的杀里 Q1c+Q1d ≥ 1」一致），与登记的钉死值都对得上、都过（`grep -n 'name=positive_control' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out` 现查，不逐行重抄）。

重开用时（Q1h，轨迹，登记不拿它判翻面）四条臂差异明显：F 中位 1100 µs、峰值 2171 µs；R1 中位 1015 µs、峰值 1407 µs；R0 中位 124 691 µs（约 125 ms）、峰值 399 403 µs（约 0.4 秒）；K 中位 11 411 µs、峰值 3 271 087 µs（约 3.27 秒：K 的 `calibration_interfered=false`，这个峰值不是校准阶段撞干扰的产物，是 S1-main 200 次杀主格里单次重开撞上本机同期别的会话在跑 `cargo test` 的极端值，属于 Q1h 轨迹的一个高点，不代表这条臂常态的重开时延）。R0（redb 默认修复、不开 quick-repair）比 R1（quick-repair）重开慢两个量级，方向与 redb 4.3.0 文档注释「默认模式崩溃后重开要把整个库走一遍，quick-repair 几乎立刻完成」（登记第四节 4.6）一致；这是轨迹观察，不是 S1 的判据。

#### 判决行点名

`grep -n 'name=verdict'` 列出的这四行判决行里：

- 没有任何字段取值是布尔 `false`。
- 没有任何字段取值是 `not_run`。
- `pc_t_ok=not_applicable`（R1、R0、K 三份）：PC-T 只定义给 F，其余三条臂本来就该是 `not_applicable`，不是没跑；与可行性档同一读法。
- `calibration_interfered=true`（仅 F 一份；R1、R0、K 三份都是 `false`）：F 的 S1-calibration 撞上 V15 的干扰重跑上限（`INTERFERENCE_MAXIMUM_RERUNS=3`，四次尝试都撞见别的会话的 `cargo test`），按登记 11.1 V15「重跑，最多三次…还有就带受干扰标记照报」处理，不作废 S1（V15 只作废「那一趟的计时」，S1 的判据看 Q1a–Q1f，不看 calibration 的计时）。
- 名字表示违例、不匹配、歧义或失败的整数计数（`q1a`、`q1b`、`q1c`、`q1d`、`q1e`、`q1f`、`repaired_openings`）四条臂全部为 `0`。
- 其余字段（`judgement=judged_by_registration`、`pc_l_ok`/`pc_s_ok`/`pc_p_ok`/`pc_o_ok=true`、`commit_waits_for_device=true`、`v6_kills_inside_commit_ok=true`、`a_s1_ok=true`、`s1_positive_quantities=none`）都是预期的「过」，不点名。

按第六节 6.1 与第十一节 11.1 V1–V6：四条臂的阳性对照都过（V1–V5 不作废），V6 的门槛（180 次写中途的杀里 Q1g ≥ 100）四条臂都过（178–180），200 次主格里 Q1a–Q1f 全部为 0——**S1 够判：redb（quick-repair 与默认修复两种形态）、RocksDB（同步写 + WAL）、加固的文件，在这次 SIGKILL 模型下四条臂都没有丢已确认的块、没有静默坏块、没有幽灵块、没有打不开**。按第十节末尾「结果反过来我接不接受」写死的读法：F 在 S1 上不翻不能说它比数据库更安全（4.10 的注：F 这几格由定义加内核语义直接推得出为 0，不是它自己的恢复代码扛住的），这里写成「四个候选在这个 SIGKILL 模型下都没有翻」，不写成「都一样安全」。

#### 复跑

单测（这一段重跑一次，确认这份改动过环境变量的构建仍然过闸）：

```
cd research && BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include bash scripts/capped.sh 4 bash scripts/run-with-memory-cap.sh 8G cargo test --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store
```

23 passed / 0 failed（上一版页面记的 `librocksdb-sys` / `bindgen` 卡点由 `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` 解开，见「它答不了的」）。

这四份决判档产物不进 `replay.sh` 逐字节比对，原因与可行性档相同：S1 每次都新起子进程按真实挂钟杀、真实盘上持久时延重开，两次跑不是同一批字节（这一段亲眼见到：F 这次撞上四次干扰重跑，可行性档那次一次没撞上，同一份代码、同一批种子，产物就是不同字节）。直接重跑（`<臂>` 取 `F`、`R1`、`R0`、`K` 之一；不带 `feasibility` 参数即登记原定的 `registered` 档，200 次杀）：

```
BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include bash research/scripts/capped.sh 4 bash research/scripts/run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store -- s1 <臂>
```

这一段登记未变，只有 `driver_e162_anchors` 一行；复跑命令跑出来的行与「结果整行抄自产物」一节抄的可行性档那次输出逐字一致：

```
bash research/scripts/replay.sh E162
```
```
E162  @driver_e162_anchors     字节一致 e162-crash-verdict-block-store-2026-09-27-anchors.out
```

````

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:178-188`（整段抄，未转述）**

```markdown
### 它答不了的

- **S1 现在够判（2026-09-27 第二段补跑），S2 仍未判**：S1 四条臂各跑满登记原定的 200 次主格杀，阳性对照与 V6 门槛都过，Q1a–Q1f 全部 0，见「S1 够判档」一节；S2 仍只跑了可行性档的 10⁴ 块（登记要 10⁵ 块才够判），且 Q2a 的判据要代入 E161（崩溃放量的去重与分段耗时） 的正式 t_state，E161（崩溃放量的去重与分段耗时） 同一天也只交了可行性档（下面 Q2a 那条）。S1-large、四个几何取样点也仍未跑，见下一条。
- **S3 完全未跑**：经网络送块的持续吞吐、网络中断重连后的一致性、跨机那一格，登记第十二节修订六写明可行性档不跑；这一段（2026-09-27 第二段）按派发只补 S1，S3 仍未跑，等机器空闲的另一段。
- **S1-large、G-bs10、G-batch16、G-sparse、G-random 四个几何取样点未跑**：占用比、随机读、写入速率在这些取样点上会不会与主取样点反向，还没有数据。
- **Q2a 的门槛还没有正式值**：R_AB = (P_A + P_B) ÷ t_state 需要 E161（崩溃放量的去重与分段耗时） 的正式 t_state，E161（崩溃放量的去重与分段耗时） 同一天也只交了可行性档（实验页 [161-崩溃放量的去重与分段耗时.md](161-崩溃放量的去重与分段耗时.md)，G1–G5 未判）。E161（崩溃放量的去重与分段耗时） 那份产物的 `name=g3 cell=first_small` 行报 `states=37 state_elapsed_ns=77898270`（`research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out:37`），除以状态数得约 2 105 359 ns/状态（约 2.11 ms/状态）——**这是单个取样格、有干扰（该行同一产物里 `calibration_interfered` 类字段与 E161（崩溃放量的去重与分段耗时） 实验页正文标注同批跑的机器上一直有编译与别的装置在跑）的粗数，只当量级参考，不代入本页任何判定**；正式 t_state 要等 E161（崩溃放量的去重与分段耗时） 按登记 6.0 的加权闭式跑完全域才有。
- **占用比只跑了一次、`samples=10` 一趟**：R1、R0 的关库占用约 2 倍写入字节、K 峰值到 2.1495 再靠合并压回 1.0013，这三个数有没有随写入量变化、会不会随几何取样点（G-bs10 的小块、G-sparse 的可压缩值）反向，这次可行性档没跑。
- **13 条变异是这份源码改动之前那一版跑出来的**（2026-09-27 早些时候；`research/mutations/e162_crash_verdict_block_store.tsv` 的抓到/无效/没红三个数是那次改动之后跑的；2026-09-27 第二段没有再改源码，没有重跑变异表——三个数仍是当天早些时候那一次的）。
- **登记第十二节缺「修订七」**：2026-09-27 第二段跑 S1 够判档之前没有先补记登记修订，见「S1 够判档」一节开头的补记缺口。
- **`librocksdb-sys` 这一次能编过，上一版遇到的卡点原因仍没查**：这一页早些时候写作时重跑单测撞到 `bindgen` 找不到 `stdbool.h`（本机只有 `libclang1-18` 运行库，没有 `clang` 可执行文件，缺 resource-dir 里的标准头）；2026-09-27 第二段设 `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` 之后单测与四条臂的 `cargo build`/`cargo run` 都过了，说明这个环境变量能绕开那处卡点，但两次之间本机环境具体起了什么变化（为什么不设这个变量就找不到 `stdbool.h`）没有查；下一次要动这份源码，先照这段的复跑命令设这个变量。

```

**出处 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md:189-194`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 4 | 备料 | 2026-09-27 不受影响：术语改名只动措辞，结论、数与产物不变（上一次回看 2026-09-27：不受影响：S1 够判档测的是判定块存在 redb/RocksDB/加固的文件哪一个、SIGKILL 下丢不丢块，不涉及崩溃状态集合的枚举域本身；`.claude/kb/decisions/13-验证路线.md` 已定项 4 的「**依据**」段仍没有引这个实验（2026-09-27 `grep -n 'E162（崩溃放量判定块存储选型）' .claude/kb/decisions/13-验证路线.md` 零命中），这一行仍按备料记，要不要升成支撑交主 agent 判） |

```

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:2-4`（整段抄，未转述）**

```markdown

跑前登记 `research/prompts/e163-preregistration.md`；问题单 `research/prompts/m2-gpu-multicard-r1-forks.md`（M1、M2、M3）。装置独立手写模型，不与 `crates/` 共用代码：`research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs`，bin `e163-gpu-multicard-crc32c`，挂在不默认打开的特性 `e163-gpu`（`wgpu = "=30.0.1"` 只开 `vulkan`/`wgsl`/`std`、`pollster = "=1.0.1"`），门禁 15 号不编它。10 条单测全绿（`cargo test --release --features e163-gpu --bin e163-gpu-multicard-crc32c`）。

```

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:5-14`（整段抄，未转述）**

```markdown
### 这一段做了什么

- **臂 C**：按位、反射 CRC-32C 参照实现（多项式 0x82f63b78、初值 0xFFFFFFFF、输出取反），源码 `e163_gpu_multicard_crc32c.rs:30-43`。
- **臂 G**：WGSL 计算着色器，逐单元一个调用，按位算同一套 CRC-32C，字节从 `array<u32>` 按 `(word >> 8*(k%4)) & 0xFF` 取，源码 `e163_gpu_multicard_crc32c.rs:216-273`（内嵌字符串 `COMPUTE_SHADER_SOURCE`）。
- 批：N = 2560 单元 × 32768 字节，种子 `0xE163092700000001`，单元内容 = splitmix64 铺满。
- R1（M1）：本机两张 NVIDIA 独显卡（RTX 5090、RTX 5060 Ti）各算 1280 个单元。
- R2（M2）：五片各 512 单元，片 0–1 本机两张卡，片 2–4 第二台三张卡（RTX 5080、RTX 5060 Ti × 2）；同一二进制两端 sha256 相同，往返文件两端 sha256 相同。
- F1、F2（M3）：在 R1 的两片配置上分别注入一位翻转（两处）与丢一条结果（一处）。
- PC-A（每次跑开头，每张卡与臂 C 都算）：跑前登记第七节 B 类八个锚点（ASCII "123456789"、空串、32×00/32×FF、升序/降序、32768×00/32768×FF），与命令核过的期望值逐一比对。

```

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:15-48`（整段抄，未转述）**

````markdown
### 结果整行抄自产物

产物：`research/results/e163-gpu-multicard-crc32c-2026-09-27-r1.out`（M1）、`research/results/e163-gpu-multicard-crc32c-2026-09-27-r2.out`（M2）、`research/results/e163-gpu-multicard-crc32c-2026-09-27-f1.out`、`research/results/e163-gpu-multicard-crc32c-2026-09-27-f2.out`（M3）。

M1（R1，本机两张卡）：
```
MERGE_INPUT_HEADER seed=0xE163092700000001 unit_count=2560
MERGE total_units=2560 shards=2
MERGE matched_count=2560
MERGE mismatched_count=0 mismatched_global_unit_identifiers=[]
MERGE missing_count=0 missing_global_unit_identifiers=[]
MERGE duplicated_count=0 duplicated_global_unit_identifiers=[]
MERGE all_reference_values_distinct=true
```
两张卡 `anchors_cpu_match=true anchors_gpu_match=true sentinel_and_gid_ok=true record_count=1280`（各一次）。**M1 判「能」**：Q1 本机两张卡都起得来（`nvidia-smi -L` 2 行，`probe` 也报 2 张）、Q2 每片不一致数 0、Q3 合并覆盖数 2560。

M2（R2，双机五张卡）：另一台 `--probe` 退出码 0、报 3 张 NVIDIA 独显卡（Q4 过）。本机两片（0、1）与另一台两片（3、4，RTX 5060 Ti × 2）都成功，`anchors_cpu_match=true anchors_gpu_match=true sentinel_and_gid_ok=true`，往返文件（片 3、4 的结果文件）两端 sha256 逐一相同、条数各 512（Q6 过）。**另一台第三张卡（RTX 5080，片 2，单元 1024–1535）两次都建不起 GPU 上下文**：`request_device` 失败，`RequestDeviceError { inner: Core(Device(OutOfMemory)) }`；当时该卡 `nvidia-smi` 显存余量仅 141 MiB（被本地服务占满，登记「不停本地模型」）。合并 4 片（0、1、3、4）：
```
MERGE total_units=2560 shards=4
MERGE matched_count=2048
MERGE mismatched_count=0 mismatched_global_unit_identifiers=[]
MERGE missing_count=512 missing_global_unit_identifiers=[1024, ..., 1535]
```
覆盖数 2048 < 2560，触发失败条款 2（Q5/Q7 门槛不过）。**M2 判「不能」（在今天的显存条件下）**：不是双机多卡协作本身走不通（3/5 张卡全部成功，跨机传输、二进制拷贝、sha256 比对、合并全部正确），而是其中一张卡当时显存不够分配一个 wgpu 设备上下文；登记写死「不停本地模型、不换配置重试」，所以这次观测按登记口径就是「不能」，翻面条件是那张卡腾出显存。

M3（F1、F2，均在 R1 两片配置上）：
```
# F1
MERGE mismatched_count=2 mismatched_global_unit_identifiers=[100, 1381]
# F2
MERGE missing_count=1 missing_global_unit_identifiers=[1480]
```
F1 注入的两个全局单元（100、1381）与报出的不一致集合逐一对上，没有多报；F2 丢的单元（1480）与报出的缺失集合逐一对上。**M3 判「会红」**：两种故障各注入一次，比对都如实报出来。

````

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:49-59`（整段抄，未转述）**

```markdown
### 判决行的点名（第四步 4c）

四份产物里 `grep -n 'name=verdict'` 命中 0 行——这个装置不写 `name=verdict` 这种判决行，判定由 `MERGE ...` 系列字段直接给出。逐份点名其中表示「没过」的字段：

- `-r1.out`：`mismatched_count=0`、`missing_count=0`、`duplicated_count=0`（都是违例类计数，取值 0，不点名；`all_reference_values_distinct=true` 是布尔真，不点名）。**没有 false / not_run 字段。**
- `-r2.out`：`r2-shard2-failure.out` 段落里的失败是 wgpu **原样报错**（`RequestDeviceError`），不是本装置自己写的判决字段，登记允许「起不来的卡报 wgpu 原样错误」；合并部分 `missing_count=512`——这是名字表示「缺失」的整数计数、且非 0，**点名**：它是显存不足导致片 2 完全没跑出结果的直接后果，在登记预期内（触发失败条款 2）。
- `-f1.out`：`mismatched_count=2`——名字表示「不匹配」的整数计数、非 0，**点名**：这是 F1 故意注入的两个翻位，比对如实报出，在登记预期内（Q8 过）。
- `-f2.out`：`missing_count=1`——名字表示「缺失」的整数计数、非 0，**点名**：是 F2 故意丢的那一条，在登记预期内（Q9 过）。

判决行点名共 3 条（`missing_count=512`、`mismatched_count=2`、`missing_count=1`），全部是登记预期内的故意注入或已知环境限制，没有 `false` / `not_run` 字段。

```

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:60-81`（整段抄，未转述）**

```markdown
### 变异

变异表 `research/mutations/e163_gpu_multicard_crc32c.tsv`（7 行：`M1_合并比对恒判相同`、`M2_合并只遍历收到的记录不按0..N查缺`、`M3_着色器用未反射的多项式`、`M4_着色器漏最后取反`、`M6_着色器按大端取字节`、`M7_结果的gid不从设备回读`、`M8_每次只派一半工作组`，行名是表内部自己的标签，不是全仓编号）。**`research/scripts/mutate.sh` 跑不了这个表**：它固定用 `cargo test --release --bin "$BIN"`，不带 `--features`，而这个 bin 挂在 `required-features = ["e163-gpu"]` 上，`cargo test` 找不到这个 bin 目标，基线直接不绿（退出码 2，`mutate: 基线就是红的，先修好再来`）——这与 E162（崩溃放量判定块存储选型） 同挂在 `required-features` 上的 bin 共享同一个工具限制，不是这次装置写错。

按 `.claude/rules/mutation-sampling.md` 逐条手动验证（改一行、`cargo test --release --features e163-gpu --bin e163-gpu-multicard-crc32c` 重编重跑、看断言红没红、改回来确认基线复绿）：

| 变异 | 抓到的测试 | 备注 |
|---|---|---|
| 合并比对恒判相同 | `compare_reference_and_shards_reports_injected_mismatch` | 抓到 |
| 合并只遍历收到的记录，不按 0..N 查缺 | `compare_reference_and_shards_reports_missing_unit` | 抓到 |
| 着色器用未反射的多项式 | `gpu_matches_cpu_reference_on_anchor_vectors` | 抓到，未反射多项式的十六进制值是 `0x1edc6f41` |
| 着色器漏最后取反 | `gpu_matches_cpu_reference_on_anchor_vectors` | 抓到 |
| 臂 C 与着色器同改成同一个错误多项式 | `crc32c_reference_matches_prereg_anchor_vectors`、`gpu_matches_cpu_reference_on_anchor_vectors`；互比类测试全绿 | 抓到，手工验证：Rust 侧 `CASTAGNOLI_REFLECTED_POLYNOMIAL`（`:24`）与 WGSL 侧 `CRC32C_REFLECTED_POLYNOMIAL`（`:234`）两处独立声明的常量都改成 `0x82f63b79`，两处相隔约 210 行、中间没有可以一起框进去的连续锚点，`mutate.sh` 一行只认一个原文→替换文的锚点，写不成一行 TSV |
| 着色器按大端取字节 | `gpu_matches_cpu_reference_on_anchor_vectors` | 抓到 |
| 结果的 gid 不从设备回读，改用固定值 | `gpu_dispatch_covers_every_unit_no_sentinel_residue`、`gpu_matches_cpu_reference_on_anchor_vectors` | 抓到 |
| 每次只派一半工作组 | 全部三条 GPU 测试 | 抓到 |
| 本机收到的结果文件截掉末行 | 手工截断 `r1-shard1.out` 末行重跑 `merge`：`missing_count=1 missing_global_unit_identifiers=[2559]`，sha256 与原文件不同 | 抓到，手工验证：这是对产物的破坏，不是对源码的破坏，`mutate.sh` 靠文本替换源码再编译，本来就框不住这一类 |

九条全部按预期变红，抓到 9 / 无效 0 / 没红 0；其中「臂 C 与着色器同改成同一个错误多项式」与「本机收到的结果文件截掉末行」两条没能写进 TSV 表，理由见各自「备注」列，都已按 `mutation-sampling.md` 的判据手工做完并确认会红，不算盲区，只是没有被计入自动化变异表的条数。

改了一个格式常量之后，九条逐条改完都确认「已还原，基线仍全绿」（10 条单测全过）。

```

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:82-85`（整段抄，未转述）**

```markdown
### 修坏 cargo test 挂起的一个真实 bug（题外，写进这份页面以防复跑重踩）

第一次以默认并行度跑 `cargo test --release --features e163-gpu --bin e163-gpu-multicard-crc32c` 时挂起 39 分钟无输出（主 agent 发现并指示排查）：三条要 GPU 的单测各自建一个 `wgpu::Instance` 且都选中同一张卡（枚举序第 0 张），默认并行的测试线程同时对同一张物理卡建 `Device`、派发、`device.poll(Wait)`，撞上后卡死在某个 GPU 侧阻塞调用里，不报错也不超时。修法：给这三条测试加一把进程内 `static GPU_TEST_MUTEX: std::sync::Mutex<()>`，序列化对 GPU 的访问；改完限时跑了 4 次，10 项全过，0.73–1.53 秒。这个修法已经在最终源码里，不是待办。

```

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:86-95`（整段抄，未转述）**

````markdown
### 复跑

`research/scripts/replay.sh` 只登记了 R1 里唯一不含计时字段、逐字节可比的一段——`merge` 的输出（每片结果文件自己带 `build_device_ms` 等计时字段，两次跑不同，不登记逐字节比对）：

```
bash research/scripts/replay.sh E163
```

判定：字节一致（`research/results/e163-gpu-multicard-crc32c-2026-09-27-r1-merge.out`，驱动函数 `driver_e163_r1_merge`，实测 2026-09-27）。装置本身不用 `e7_index_bench::Emitter` 的 `E7RESULT` 协议，驱动函数在 bash 里把 `merge` 的原始输出包一层，满足 `replay.sh` 完整性闸要的 `E7RESULT` / `name=done` 收尾行。要机器上至少两张能起上下文的 NVIDIA 独显卡；R2、F1、F2 的复跑命令见「这一段做了什么」一节的实际调用形态，不登记逐字节比对（R2 依赖另一台机器与它当时的显存状态）。

````

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:96-106`（整段抄，未转述）**

```markdown
### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | CPU 参照 | 按位、反射 CRC-32C，初值 0xFFFFFFFF，输出取反（函数体到 :43 结束） | `research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs:30` `fn crc32c_reference` | D18（块里携带什么信息） 已定项 17 的四个参数（Rust 侧常量，独立声明） |
| 2 | GPU 着色器 | WGSL 计算着色器逐单元逐位算同一套 CRC-32C，字节按 `(word >> 8*(k%4)) & 0xFF` 取（着色器源码到 :273 结束） | `research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs:216` `const COMPUTE_SHADER_SOURCE` | D18（块里携带什么信息） 已定项 17 的四个参数（WGSL 侧另写一份常量，不引用路径 1） |

跨路径断言：单测 `gpu_matches_cpu_reference_on_anchor_vectors` 把两条路径在 8 个锚点上的输出各自与 `EXPECTED_ANCHOR_VALUES`（登记第七节 B 类，非两路径互比）逐一比对；`merge` 子命令的 `compare_reference_and_shards` 把 GPU 结果与 CPU 现算的参照逐单元比对（2560 个值，扫遍整批，不是抽样点）。结论「对 R1 的全部 2560 个单元，臂 C 与臂 G 逐个相同」是全称的，X 的取值是扫遍这一批的全部 2560 个。

⚠️ 两条路径共享的是「D18（块里携带什么信息） 已定项 17 那四个参数值本身是对的」这一个前提：kb 里这四个值若本身写错，两份独立实现会按同一个错误值各写一份，在健康输入上继续互相吻合（D13（验证路线） 已定项 5 警告的那类假红）。「臂 C 与着色器同改成同一个错误多项式」这条变异演示了这一点：互比类测试全绿，只有对 `EXPECTED_ANCHOR_VALUES` 的绝对值检查抓到——这条检查（`crc32c_reference_matches_prereg_anchor_vectors`、`gpu_matches_cpu_reference_on_anchor_vectors`）不可少。

```

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:107-112`（整段抄，未转述）**

```markdown
### 它答不了的

- 显存充裕时双机五卡是否也能全部成功：这次唯一失败的那张卡（RTX 5080）显存余量仅 141 MiB，是本地服务占用造成，不是本装置或 wgpu/Vulkan 路径本身的限制；没有在显存腾出后重跑过。
- 速度、吞吐、多卡相对 CPU 的收益：登记写死「只问能不能，不设速度门槛，用时只报不判」，用时数字在产物里但没有判据。
- 单元长度、派发粒度、批大小的敏感性：只在 N=2560、每次派发 128 单元（着色器实际按 64 一组，见工作组大小）、32768 字节单元下跑过，登记第八节写明「没有几何敏感性」。

```

**出处 `.claude/kb/experiments/163-GPU多卡算单元校验和.md:113-120`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 5 | 备料 | 2026-09-27 不受影响：依据段还没有引用 E163（GPU多卡算单元校验和），只引了三方对抗材料与用户定案；E163（GPU多卡算单元校验和） 重新验证了「参数钉死才能互证独立」这条前提（「臂 C 与着色器同改成同一个错误多项式」变异逐字证明：两份实现同改错时互比测不出、只有绝对值检查抓得到），是否升成支撑交主 agent 判断 |
| D18（块里携带什么信息） 已定项 17 | 备料 | 2026-09-27 不受影响：依据段还没有引用 E163（GPU多卡算单元校验和），只引了 E144（头校验和算法的代价与判别力）、D13（验证路线） 已定项 5、用户定案；E163（GPU多卡算单元校验和） 用一份新的独立实现（WGSL 着色器）复核了这四个参数在健康输入与故障输入上都与登记值一致，是否升成支撑交主 agent 判断 |
| D24（后台重活能不能卸给 GPU） | 不影响 | 2026-09-27 不受影响：D24（后台重活能不能卸给 GPU） 管的是批量压缩、scrub、纠删码重建三个后台重活（已定项 3 的射程逐字限定），E163（GPU多卡算单元校验和） 问的是崩溃放量里 checker 单元校验和能不能切给多卡算，与那三个候选场景不是同一件事（与 E161（崩溃放量的去重与分段耗时） 对 D24（后台重活能不能卸给 GPU） 的回看同一口径） |

```

**出处 `.claude/kb/checks-owed.md:51-51`（整段抄，未转述）**

```markdown
| C50 | 阻塞标记没人维护 | **「这一条挡不挡第一行代码」只记在正文的一句 ⚠️ 里，没有任何东西在维护它。** 一条决策定案之后，别的未定项可能**因此**变成格式级，而没有一个阶段会回头问这件事 | **已还一半（2026-09-01）**：门禁阶段 `.claude/gate.d/31-blocking-verdict.sh` 强制每条未定项在登记行里带一条规范判定（是 / 否 / 无对象 + 日期 + 依据），缺就红，红绿判别力样本在 `fixtures/` 里证过。**仍欠可机检的那一半**：`decisions/` 下每个未定项各判一次「它改不改变新池新建文件写出的字节」——可机检的那一半是**依赖对**：若某未定项被一条已定条款点名为「载体 / 布局 / 位置」的一部分，而它自己没标阻塞，判红。判别力自证：把某条已标阻塞的未定项去掉标记，必须红。⚠️ **2026-09-21 评估下来这个形态会误判，没做**：当天现查，全仓只剩 5 条未定项、被别的决策跨文件点名的 4 处，其中 D17（实现分层与第三方管道） 已定项 1 第 4 条那一处字面点名了 D22（单元原子性怎么合成） 未定项 6、上下文也有「根槽写路径」「字节表」这些词，而它整句的意思是「zoned 开线时按这一条数，**不按** D22（单元原子性怎么合成） 未定项 6 乙问的答案数」——按「点名加词表」的机械判据，这一处会被判成违例。要分出它得先能认「否定式引用」，而那是语义判断。对象只有 4 处引用，做一道会误判的常驻检查不划算 ⇒ 这一条的可机检形态要重新想，或者接受它只能靠人看 | 无前置，纯欠工 | 2026-08-30 实测：D5（快照 / 空间记账机制） 的载体形态定案后，其分项 1、2（当时未定）当场变成格式级（记账 key 的字节布局），D8（核心索引结构） 已定项 2（节点大小）与 D18（块里携带什么信息） 已定项 3 的位置那一格同理——**四条都没被标，靠一次人工复核才发现** |
```

**出处 `research/prompts/m3-prune-gpu-r1-facts-landscape.md:1-303`（整段抄，未转述）**

```markdown
# m3-prune-gpu-r1 事实表甲：崩溃验证全景与可共享的前缀

调度记录 `records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 任务甲。只读代码与 kb，没编译、没跑任何测试。
行号都是 2026-09-28 这一次现查的（`grep -n` / `awk 'NR==n'`）；标「推的」的是从代码读出来的推论，没有跑出来的数撑。
状态数一律照用例里钉死的常量或门禁登记行原样引，另标出处；百分比是拿那几个常量现算的（`python3`）。

## 一　今天仓里的崩溃验证路径（问题 1）

### 1.1 总表

| # | 路径 | 入口（文件:行 函数） | 枚举什么 | 每个状态上依次调 | 结果累计 | 谁跑 |
|---|---|---|---|---|---|---|
| P1 | 层 0 第一条流（新池新建文件：取号 → 暖机 × 2 → A） | `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:498` `layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations`，经同文件 `:402` `enumerate_counting_allocation_generation_read_sets` → `crash.rs:2583` `enumerate_layer0_in_state_slices_or_one_shard` | 段模型全量（见 1.2），`FULL_STATES = 16_777_260`（同文件 `:446`），快档 `QUICK_TIER_STATES = 46`（`:450`） | 见 1.3 的层 0 流水线；另带观察者：每个状态交给 `allocation_generation_read_set_in`（`:416`–`:425`）记 I-3.10 那一类计数 | `Layer0Tally`（`crash.rs:743`），观察者计数进 `Layer0ObserverCounts`（`crash.rs:784`） | 54 号 `--full`，`crash-case:layer0-first-stream`（`.claude/gate.d/stage-inputs.tsv:36`），可双机分片 |
| P2 | 层 0 第二条流（固定脚本到 E 再正常卸载） | `crash_enumeration_fixed_script_stream.rs:797` `full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean` → `crash.rs:2583` | `FULL_STATES_THROUGH_THE_UNMOUNT = 1_662_648_564`（同文件 `:100`），两态口径 `1_662_648_449`（`:96`），快档 278（`:114`） | 层 0 流水线，无观察者（`:826` 传 `None`） | `Layer0Tally` | 54 号 `--full`，`stage-inputs.tsv:37`，可双机分片 |
| P3 | 层 0 并行线一流（多记录发布） | `crash_enumeration_multi_record_publish_stream.rs` 全量用例（`:394` 起调 `enumerate_layer0_in_state_slices_or_one_shard`） | 文件头 `:21` 写「层 0 的枚举域 1358954634 个状态」 | 层 0 流水线 | `Layer0Tally` | 54 号 `--full`，`stage-inputs.tsv:40`，可双机分片 |
| P4 | 树分裂七条流 | `crash_enumeration_tree_split_streams.rs` 全量用例，经同文件 `enumerate`（`:356` 调 `enumerate_layer0_selecting_versions`） | 每条只录分裂那一次发布，段 `[2u, 2, 1, 2]`（`:307`–`:308`）；七条合计 78905428（`stage-inputs.tsv:41` 注释） | 层 0 流水线 | 每条一份 `Layer0Tally` | 54 号 `--full`，`stage-inputs.tsv:41`，不认分片、不续跑 |
| P5 | 位置寻址树流 | `crash_enumeration_position_addressed_trees.rs:480` `full_enumeration_of_every_enumerable_position_addressed_tree_stream_is_exhaustive_and_clean`，经 `:427` `enumerate_layer0_selecting_versions` | 只录被判那一次发布或挂载；用例不钉状态数，只断言等于闭式（`:435`–`:450` 那一条 `assert_eq!`）；注释写「状态数上亿级」（`:476`，没钉数） | 层 0 流水线 | `Layer0Tally` | 54 号 `--full`，`stage-inputs.tsv:42`，不认分片、不续跑 |
| P6 | 会话推的抬 F 串 | `crash_enumeration_floor_raise_pushed_by_admission.rs`（`:153` 调 `enumerate_layer0_in_state_slices`） | 一条历史里只录抬 F 那一串，`STATES_AT_MOST = 1_000_000`（`:43`） | 层 0 流水线，外加每个状态恢复之后与再挂载之后各跑池级 checker（文件头 `:3`–`:5`） | `Layer0Tally` + 用例自己的断言 | 54 号 `--full`，`stage-inputs.tsv:38` |
| P7 | C561 σ 段全量 | `record_checker_judges_absence_by_the_persisted_set.rs:719` → 同文件 `:581` `enumerate_every_subset_of_one_segment` | σ 一段的全部子集，`assert_eq!(enumerated.states, 65_536)`（`:731`–`:733`） | 只有一遍看 journal 的恢复 + 记录核对器（`:369`–`:380` `judge`：`recover(..Consult)`、`check_records`）；不跑不看 journal 那一遍、不跑 oracle、不跑池级 checker | 用例自己的计数（`states`、`reported_missing`，`:649`–`:660`） | 54 号 `--full`，`stage-inputs.tsv:39` |
| P8 | 崩溃注入（随机历史上抽样） | `crates/singlefs-checker-tier/src/crash_injection.rs:1979` `run_crash_injection_campaign` → `:722` `inject_crashes_into_history`；快档用例 `tests/crash_injection_campaign.rs:135` | 24 段历史 × 每段 24 步 × 每段抽 4 个崩溃点（`crash_injection_campaign.rs:41`–`:45`）；域同层 0 的段模型，但**不取第三态**（`crash_injection.rs` 里 `Tearable\|torn` 0 处，模块头 `:5` 写「撕裂并进没持久」） | 见 1.4 | `CrashInjectionTally`（`crash_injection.rs:376`） | 54 号 `--full`，`stage-inputs.tsv:43`；大档 `:963` 不登记 |
| P9 | 故障注入 | `crates/singlefs-harness/src/fault_injection.rs:2723` `run_fault_injection_campaign` → `:1842` `inject_faults_into_history` → `:2173` `inject_one_fault` | 不是崩溃状态枚举：按种子在读 / 写 / 刷盘调用里摆注入点，每个注入点**整段历史重跑一遍**（`:1940`–`:1972`）；快档 24 段 × 20 步 × 4 个注入点（`tests/fault_injection_fast_tier.rs:49`–`:51`） | 重跑到注入点 → 录制流整份重建镜像（`:2263`–`:2267` `MemoryPool::with_devices` + `apply`）→ `recover(..Consult)`（`:2273`）→ 模型判定 → 池级 checker（`:2377`）；被吞的写插回去的那份镜像再跑一次 checker（`:2395` 起） | `FaultInjectionTally`（`:1364`） | harness 档，随时跑 |
| P10 | 坏盘输入 | `crates/singlefs-checker-tier/src/bad_disk_input.rs:2694` `run_bad_disk_campaign` → `:950` `feed_bad_disk_inputs_from_history` → `:1156` `feed_a_damaged_image` | 不是崩溃状态：把合法镜像按种子改坏 | 恢复（`:1163`）、可写挂载（`:1184`）、池级 checker（`:1198`），三者都包在 panic 捕获里 | `BadDiskTally`（`:556`） | checker 档快档（普通用例）；大档 `tests/bad_disk_input_campaign.rs:1035` 不登记 |
| P11 | QEMU 真设备 | `.claude/gate.d/55-qemu-device-streams.sh`，装置 `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs` 与 `new_pool_file_creation_device_log_check.rs` | 不枚举崩溃状态：六次虚机跑，设备侧 blklogwrites 日志与程序录制流逐项比（55 号文件头 `:8`–`:21`）；文件头 `:5`–`:6` 写明「今天只有真实负载与设备侧录制、没有崩溃注入」 | 冷重开读回 | 阶段自己判 | 55 号，提交时 |

P1–P7 共用同一个枚举器（`crates/singlefs-checker-tier/src/crash.rs`），P7 例外（用例自己按掩码切片，`record_checker_judges_absence_by_the_persisted_set.rs:565` `mask_slices`）。
P8、P10 共用种子切片 `crash_injection.rs:2116` `seed_slices`（`bad_disk_input.rs:2703` 调它）；P9 在 harness 档另有一份 `fault_injection.rs:2836` `seed_slices`。

另有住在 checker 档、不登记 `crash-case:`、归 54 号快档（`cargo test --release -p singlefs-checker-tier --lib --tests`）的崩溃状态用例：
按文件数的现查（`grep -c '#\[test\]'` 与 `grep -c '#\[ignore'`，逐文件）里 `ignored=0` 的那几份，例如
`crash_enumeration_acquisition_barrier.rs`（只展开两段）、`crash_enumeration_record_spill_over_stream.rs`（单元段 136 写不展开，25 个状态，文件头 `:14`–`:16`）、
`crash_enumeration_writable_mount_of_a_formatted_pool.rs`、`crash_points_of_the_rollback_publish.rs`、`crash_points_tree_identifier_watermark_orphans.rs`（后两份自己逐个造崩溃状态，不经 `crash.rs` 枚举器）。
研究侧另有一条独立实现：E142 装置（`research/e7-index-bench/src/bin/e142_new_pool_file_creation_dry_run.rs`，`stage-inputs.tsv` 的 `E142/layer0` 行），手写模型、不读 `crates/`；以及 E161 装置 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`（文件头 `:10`–`:14`：第一条流经 `build_pool`、第二条流脚本照抄 fixed_script 用例，枚举计划自己一份、与 crates 逐状态对拍）。

### 1.2 段模型与闭式（P1–P6 共用）

| 事实 | 出处 |
|---|---|
| 切段：一段录制流拆成写表与段，「当前段里每块有写的盘都被自己的屏障或 FUA 放行了才关段」 | `crates/singlefs-harness/src/memory_pool.rs:514`–`:515` 文档、`:547` `writes_and_segments_with_stream_indexes_and_entries`、关段在 `:597`–`:599` |
| 状态 = 前面的段全持久 + 当前段每次写各取几态的任意组合（去掉整段全持久）+ 最后一个「全部持久」 | `crash.rs:2177`–`:2179` 文档；`crash.rs:1813`–`:1815` |
| 每次写几态：原地覆写（不是单元写、长于一个扇区、罩住的范围原来有东西）取三态「没持久 / 新旧都读不出 / 持久」，其余两态 | `crash.rs:1484`–`:1488`，判法 `crash.rs:1566` `is_tearable_in_place_overwrite` |
| 全量一段的状态数 `3^m · 2^(n−m) − 1`；甲二快档 `3^m · 2^(k−m) ·（c>0 时 2，否则 1）− 1` | `crash.rs:1397`、`:1400`–`:1401`；实现 `crash.rs:1412` `Layer0SegmentExpansion::state_count` |
| 整条流闭式 = 各段之和 + 1 | `crash.rs:1785` `state_count_of_the_segments`（`try_fold(1u64, …)`，`:1797`） |
| 两态口径的闭式 `1 + Σ(2^|段| − 1)` | `memory_pool.rs:611` `closed_form_state_count` |
| 状态序号 → 段：各段序号区间首尾相接，`partition_point` 定段 | `crash.rs:1849`–`:1859`、`:1898` `segment_of_state` |
| 段内序号按混合进制拆到段内每次写，第一次写是最低位 | `crash.rs:1875`–`:1895` `assign_landings` |
| 撕裂态的字节：枚举开始时一次性算好，每个原地覆写的写在写表末尾接一条撕裂镜像，再接同段里在它之后、同盘重叠的写各一份重放 | `crash.rs:1631`–`:1634` 文档、`:1648` `WritesWithTornImages::of`（在 `:2634` 调一次） |

### 1.3 层 0 一个状态上依次调了什么（P1–P6）

本体 `crash.rs:1131` `evaluate_state_recording_findings`，每个状态恰好这一串：

| 次序 | 调用 | 行 |
|---|---|---|
| 1 | 由计划给出持久集合 `Vec<bool>`（工作线程里调） | `crash.rs:2143` `plan.persisted_writes_of_state(ordinal)` |
| 2 | `newest_persisted_root(writes, &persisted)`（线性扫写表） | `crash.rs:1141`；定义 `memory_pool.rs:812` |
| 3 | 建 `CrashImage { base, writes, persisted }`（不物化，见第五节） | `crash.rs:1142`–`:1146` |
| 4 | 恢复两遍：`recover(&image, JournalPolicy::Consult)`、`recover(&image, JournalPolicy::Ignore)` | `crash.rs:1147`、`:1148` |
| 5 | oracle 两遍：看 journal 那一遍判 `violations`，不看的判 `ignored_violations` | `crash.rs:1168`、`:1190`（`classified_oracle_violation_for_versions`，`:968`） |
| 6 | 池级 checker 一遍：`check_pool_image(&image)`，逐条不变量记评估 / 违例 / 不适用 | `crash.rs:1210`–`:1231` |
| 7 | 记录核对器一遍：`check_records(&image, consulted.effective_root)` | `crash.rs:1247`（→ `:105` `check_records_against`） |
| 8 | 判红的记进发现表（按签名去重） | `crash.rs:1182`–`:1266`（`Layer0Findings::record_red_state`，`:615`） |
| 9 | 有观察者时，状态的 `persisted` 与恢复报告带回调用线程、按序号交观察者 | `crash.rs:2144`–`:2156`、`:2861`–`:2874` |

累计：每片一份 `Layer0Tally`，调用线程按片号从小到大 `absorb_following_slice`（`crash.rs:859`：计数相加，「第一处」取序号最小的一片的）。字段表 `crash.rs:743`–`:779`。

P7 只做第 3、4（只 Consult）、7 步（`record_checker_judges_absence_by_the_persisted_set.rs:369`–`:380`），注释 `stage-inputs.tsv:39` 写明「不经 enumerate_layer0_in_state_slices（那一路每个状态多跑一遍不看 journal 的恢复与池级 checker、状态集合差一个）」。

### 1.4 崩溃注入（P8）一个崩溃点上依次调了什么

入口 `crash_injection.rs:722` `inject_crashes_into_history`：先整段跑一遍历史录流（`:735`），切段（`:767`–`:768`，与层 0 同一个切段函数），按种子抽崩溃点（`:774` → `:1673` `draw_crash_points`，快档 `Sampled { crash_points_per_history: 4 }`，`tests/crash_injection_campaign.rs:43`–`:45`）。崩溃点按段号排好之后逐个：

| 次序 | 调用 | 行 |
|---|---|---|
| 1 | 基线只往前叠：更早的段整段 `base.apply_writes(...)`，不为每个崩溃点从头重建 | `crash_injection.rs:800`–`:802` |
| 2 | 建整条流长度的持久集合 `vec![false; writes.len()]`，更早的段填 true，当前段按子集 | `:803`–`:809` |
| 3 | `CrashImage { base: &base, writes: 当前段那几次写, persisted: 段内子集 }`（叠加表只有当前段） | `:810`–`:815` |
| 4 | `recover(&image, JournalPolicy::Consult)` 一遍（没有 Ignore 那一遍） | `:846` |
| 5 | 模型判定 `crash_recovery_disagreement` | `:857`–`:858` |
| 6 | 记录核对器 `check_records_against(&image, &image, &writes, &persisted, …)`（整条历史的写表与持久集合） | `:872`–`:879` |
| 7 | 池级 checker `checker_violations_on(&image, …)` | `:887` |
| 8 | 第二截：`materialized_crash_image(&image)` 把崩溃镜像物化成一个池（`base.clone()` + 叠持久的写，`:1049`–`:1057`），在上面可写挂载（`:1123`）、发一次布（`:1139`），记录核对器（`:940`–`:941`）、池级 checker（`:1374`） | `:915`–`:950` |
| 9 | 第三截：挂载那一段录制流切段，取号 / 写行 / 暖机各按种子摆一个二次崩溃（`:1477` `draw_second_crash_points`）；每个二次崩溃：第二份基线同样只往前叠（`:1574`–`:1585`），`recover(..Consult)`（`:1603`）、模型（`:1615`）、记录核对器（`:1621`）、池级 checker（`:1627`） | `:951`–`:965`、`:1543` |

累计：每段历史一份 `CrashInjectionTally`，调用线程按片次序收齐后逐段 `tally.absorb`（`:2081`–`:2089`）。

## 二　共享前缀（问题 2）

### 2.1 三条从 mkfs 起的整条流，段序列逐段对照

三条流都从 `build_pool(tag)`（`crates/singlefs-harness/tests/common/mod.rs:372`）起：mkfs → 取号 → 暖机 × 2 → A（新池新建文件），基线都是 `memory_pool_after_mkfs()`（`common/mod.rs:194`，只施加 mkfs 那几步），写表都从 `mkfs_operation_count` 之后切。

| 流 | 用例里钉的段长数组 | 出处 | 段数 | 写数 |
|---|---|---|---|---|
| 第一条（P1） | `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]` | `crash_enumeration_new_pool_file_creation_stream.rs:466` | 11 | 41（`:470`） |
| 第二条（P2） | `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, …, 16, 2, 1, 2]` | `crash_enumeration_fixed_script_stream.rs:439`–`:442` | 78（`:95` 注释） | `191 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21)` = 477（`:448`） |
| 并行线一（P3） | `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2]` | `crash_enumeration_multi_record_publish_stream.rs:185` | 19 | 用例不钉写数；段长之和 115（现算） |
| 只做过 mkfs 的池（不登记，不另枚举） | 段序列与第一条逐项相同 | `crash_enumeration_writable_mount_of_a_formatted_pool.rs:214`–`:237`（基线、写表连同内容、段序列逐项 `assert`） | 11 | 同第一条 |

**相同到第几段**：三条的前 11 段（第 0–10 段：取号 2、暖机一 `2,1,2`、暖机二 `2,1,2`、A 的单元 24、A 的记录 2、A 的根槽 1、A 的系统配置轮换 2）逐段同长；第 11 段起分开（第二条是 B 的 24 个单元写，并行线一是 B 的 28 个单元写）。第二条与并行线一在第 11 段就不同，两两之间没有更长的公共前缀。

**逐字节相同**：只有「只做过 mkfs 的池」那条与第一条之间有用例逐字节核（上表末行）。第一条与第二条、并行线一之间**没有用例核写表逐字节相同**；推的：三条都由同一个 `build_pool` 走同一串确定性调用（固定内容 `file_content()`、固定写时 `FIXED_WRITE_TIME_SECONDS`、固定 `filesystem_identifier: E142_FILESYSTEM_IDENTIFIER`，`common/mod.rs:48`），`tag` 只进镜像文件名（`common/mod.rs:38`–`:41`），前 41 次写应逐字节相同；「只做过 mkfs 的池」那条用例拿两个不同 `tag` 建的池比出逐项相同，是这条推论的一个旁证。

### 2.2 公共前缀那 11 段各有几个状态

逐段分解按第一条流用例里的算式（`crash_enumeration_new_pool_file_creation_stream.rs:443`–`:445`：「取三态的只有 8 次系统配置槽写，四个系统配置槽段……各 3² − 1、A 的单元段 2²⁴ − 1，其余同两态，再加全部持久那一个：1 + 4 × 8 + 3 × 3 + 3 × 1 + (2²⁴ − 1) = 16777260」）；哪一段是哪一种写照用例自己的说明（`crash_enumeration_new_pool_file_creation_stream.rs:472`「取号 2 + 暖机两次各 5 + A 29（24 个单元写、两份记录、根槽、两块盘的系统配置槽轮换）」，`crash_enumeration_fixed_script_stream.rs:388`–`:393`）；`.claude/kb/layout/01-first-txn.md:410` 的种类串是 C577 之前的形状，今天对不上（见第六节）。每段的数是按算式拆的（推的），合计与用例钉死的 `FULL_STATES`、`QUICK_TIER_STATES` 相等。

| 段 | 写数 | 内容 | 全量状态数 | 甲二快档状态数 | 归哪次发布 |
|---|---|---|---|---|---|
| 0 | 2 | 取号两块盘的系统配置槽（三态） | 8 | 8 | txg 1 |
| 1 | 2 | 暖机一的两份记录 | 3 | 3 | txg 1 |
| 2 | 1 | 暖机一的根槽 FUA | 1 | 1 | txg 1 |
| 3 | 2 | 暖机一的系统配置槽轮换（三态） | 8 | 8 | txg 2 |
| 4 | 2 | 暖机二的两份记录 | 3 | 3 | txg 2 |
| 5 | 1 | 暖机二的根槽 | 1 | 1 | txg 2 |
| 6 | 2 | 暖机二的系统配置槽轮换（三态） | 8 | 8 | txg 3 |
| 7 | 24 | A 的 12 个单元 × 2 盘 | 2²⁴ − 1 = 16777215 | 1 | txg 3 |
| 8 | 2 | A 的两份记录 | 3 | 3 | txg 3 |
| 9 | 1 | A 的根槽 | 1 | 1 | txg 3 |
| 10 | 2 | A 的系统配置槽轮换（三态） | 8 | 8 | 第一条流里是 `after_the_last_root`；第二条流里归 txg 4 |
| — | — | 第一条流末尾「全部持久」 | 1 | 1 | `every_write_persisted` |
| 合计 | 41 | | **16777260**（= `FULL_STATES`，`:446`） | **46**（= `QUICK_TIER_STATES`，`:450`） | 第一条流按发布分钉在 `:565`：`instance1_txg1=12 instance1_txg2=12 instance1_txg3=16777227 after_the_last_root=8 every_write_persisted=1` |

第二条流用例钉的按发布分（`crash_enumeration_fixed_script_stream.rs:105`–`:110`）前三格同为 `instance1_txg1=12 instance1_txg2=12 instance1_txg3=16777227`，与上表一致。

**第一条流的整个枚举域都含在另两条流里**：第一条流最后那个「全部持久」状态，在第二条流与并行线一里就是第 11 段的空子集（段内序号 0，`crash.rs:1917`–`:1936`：前面的段全持久、当前段按序号拆，序号 0 各写都取 0 = 没持久）。所以另两条流的状态序号 `[0, 16777260)` 与第一条流的全部状态一一对应、崩溃镜像相同（镜像相同以 2.1「逐字节相同」那条推论为前提）。快档同理：第二条流第 11 段只有单元写，甲二只取「全不落」那一个（`crash.rs:1947`–`:1957`：有单元写时序号除以 2 拆到原地写、余数 1 才让单元写全落；这一段没有原地写，序号 0 就是单元写全不落），第一条流的 46 个快档状态也含在第二条流的 278 个里（推的）。

| 流 | 全量状态数 | 其中与第一条流共享的 | 占比（现算） |
|---|---|---|---|
| 第一条 | 16777260 | 16777260（它自己） | 100% |
| 第二条 | 1662648564（`crash_enumeration_fixed_script_stream.rs:100`） | 16777260 | 1.009% |
| 并行线一 | 1358954634（`crash_enumeration_multi_record_publish_stream.rs:21` 文件头） | 16777260 | 1.235% |
| 三条合计 | 3038380458 | 重复枚举 2 × 16777260 = 33554520 | 1.104% |

前缀里真正贵的是第 7 段（A 的 24 个单元写，16777215 个状态，占前缀的 99.9998%）；去掉它，前缀其余 10 段加末尾一个只有 45 个状态（现算：16777260 − 16777215）。

### 2.3 这些前缀今天是不是每条流各枚举一遍

是。依据：

| 事实 | 出处 |
|---|---|
| 每条用例各自 `prepare`：各自 `build_pool` 重跑整条写路径、各自切段、各自调枚举器 | P1 `crash_enumeration_new_pool_file_creation_stream.rs:458`–`:489`；P2 `crash_enumeration_fixed_script_stream.rs:218`–`:493`；P3 `crash_enumeration_multi_record_publish_stream.rs:147`–`:185` |
| 枚举器每一趟从状态 0 数到状态总数，没有「从某个序号起」「跳过某段」的入参；能跳过的只有 `NotExpanded`（整段不展开，只以全持久进入后面） | `crash.rs:1393`–`:1404` `Layer0SegmentExpansion`；`crash.rs:2644`–`:2647` 切片从 `plan.state_count` 起 |
| 断点续跑与分片账本按「输入指纹 + 流名 + 计划哈希」分格，计划哈希含整张写表、段、版本表、有没有观察者；不同流的格互不相认，没有跨流复用 | `layer0_progress.rs:4`–`:6` 文件头；`crash.rs:2286`–`:2290` `layer0_plan_hash` 文档 |
| 全绿标记按用例分格（`singlefs-crash-case-green.<用例名>.<输入指纹>`） | `.claude/gate.d/54-layer0-replay.sh:19` |
| 唯一「相同就不另枚举」的先例：只做过 mkfs 的池那条流，用例逐项核「基线、写表连同内容、段序列」与第一条流相同，54 号文件头写「由 cargo test 里的快用例钉住，不另枚举」 | `crash_enumeration_writable_mount_of_a_formatted_pool.rs:210`–`:237`；`.claude/gate.d/54-layer0-replay.sh:4` |
| 另一种已在用的写法「起点镜像不枚举，只录被判那一次发布」：树分裂、位置寻址、记录跨条三类流把之前那一大截施加成基线（`memory_pool_before_the_current_version`，`crates/singlefs-harness/tests/common_tree_split/mod.rs:401`–`:406`），只枚举最后那一次发布的段；它们的起点前缀（mkfs、取号、暖机、A、铺垫的发布，节点容量用只供测试的开关压小）没有哪条流枚举过 | `crash_enumeration_tree_split_streams.rs:3`–`:5`、`:298`；`crash_enumeration_position_addressed_trees.rs:3`–`:4`、`:148`；`crash_enumeration_record_spill_over_stream.rs` 文件头 |

## 三　每条流「走到哪些代码模块」有没有机械登记（问题 3）

没有。今天能找到的登记只到「整个 `crates/`」或「checker 档自己的模块」这一粒度：

| 登记 | 粒度 | 出处 |
|---|---|---|
| 8 条 `crash-case:` 行的路径列一律是 `crates/ Cargo.toml Cargo.lock`，不按用例列文件 | 整个 crates 目录 | `.claude/gate.d/stage-inputs.tsv:36`–`:43`（逐行现查，`grep -c '^crash-case:'` = 8） |
| 算一条用例的输入指纹时自动减去三类读不到的文件：① 别的测试目标独占的测试文件；② `crates/mutations.tsv`；③ 没有代码读 `CARGO_BIN_EXE_` 时各包 `src/bin/` 下的文件；其余全留，另加判法摘要、工具链、构建环境、登记行本身，分片用例再加驱动脚本 | 文件级减法，不做模块可达性 | `research/scripts/admission.py:1187`–`:1199` |
| 注释原话「登记只写整个 crates/，不按用例手列它读哪些文件：新加的共用文件（tests/common.rs、build.rs、新拆出的 crate）默认就在输入里」 | — | `research/scripts/admission.py:1200` |
| 改成整份 `crates/` 的来由：层 0 规模第二轮攻方量过按流拆的清单会漏新加的 `tests/common.rs`、`build.rs`、新拆出的 crate | — | `research/prompts/m2-layer0-scale-r2-main-verification.md:23`（M3 那一行） |
| 54 号整道阶段的路径列同样是 `crates/ Cargo.toml Cargo.lock` 加三份研究脚本 | 整个 crates | `.claude/gate.d/stage-inputs.tsv:28` |
| checker 档测试文件第一行声明 `//! checker 档模块：…`，登记的是它从 `singlefs_checker_tier::` 导入了哪几个 checker 档模块（crash、layer0_progress、crash_injection、bad_disk_input、device_log、on_device_modes），不是它走到 `singlefs-core` 的哪些模块 | checker 档模块 | `.claude/rules/verification.md:40`；判法 `research/scripts/crash-case-check.py:157`、`:194`–`:199` |
| 57 号的 litmus 锚点把一条 litmus 绑到代码里的锚点（今天指 `crates/singlefs-core/src/transaction.rs` 与 `recovery.rs`），管的是内存序模型，不是崩溃流 | litmus ↔ 代码锚点 | `.claude/gate.d/stage-inputs.tsv:30` 注释 |
| 覆盖率工具：仓里 `cargo llvm-cov` 只出现在重型测试闸的命令分类里（把它认成起测试的子命令），没有任何脚本或门禁拿它量「一条流走到哪些函数」 | — | `.claude/hooks/lib_heavy_tests.py:229`、`:372`、`:381` |

所以今天改 `crates/` 下任何一个进指纹的文件（包括与某条流无关的 `singlefs-core` 模块），8 条崩溃枚举用例的指纹都变、全绿标记都不再作数（按 `admission.py` 那段减法推的；没有逐文件量过）。
里程碑三第一项把「节点走到哪些代码模块怎么机械地登记（不靠手列清单）」列为开工前要定的（`.claude/kb/milestone/03-third-txn.md:35`）。

## 四　状态调度与 `Vec<bool>`（问题 4）

### 4.1 `Vec<bool>` 与 `[bool]` 的每一处（非测试代码）

数的命令：以 `#[cfg(test)]` 所在行为界（`crash.rs:3160`、`crash_injection.rs:2131`），`awk` 数界之前含 `Vec<bool>` 的行：crash.rs 7 行、crash_injection.rs 9 行；`vec![false|true; n]` crash.rs 2 行、crash_injection.rs 3 行；参数 `&[bool]` crash.rs 2 行、crash_injection.rs 3 行。`crates/singlefs-checker/src/` 四个文件 `Vec<bool>|[bool]` 都是 0。

| 文件:行 | 写法 | 表示什么 | 长度 | 每个状态建一次吗 |
|---|---|---|---|---|
| `crash.rs:1043` | `evaluate_state` 参数 `persisted: Vec<bool>` | 一个崩溃状态的持久集合（与写表逐条对应），单版本入口 | 写表长 | 调用方给 |
| `crash.rs:1062` | `evaluate_state_for_versions` 参数 | 同上，多版本入口（手摆状态用） | 写表长 | 调用方给 |
| `crash.rs:1134` | `evaluate_state_recording_findings` 参数 | 同上，本体；进 `CrashImage.persisted`（`:1142`–`:1146`） | 写表长 | 是（按值移进） |
| `crash.rs:1493` | `TearableInPlaceOverwrites.is_tearable_by_write` | 录制流里第 i 次写是不是原地覆写（取三态） | 录制流写数 | 否，一趟枚举建一次（`:2633`） |
| `crash.rs:1504` | `TearableInPlaceOverwrites::of` 里的局部 | 同上的构造 | 录制流写数 | 否 |
| `crash.rs:1525` | `vec![false; write_count]` | `TearableInPlaceOverwrites::none`：全部只取两态 | 录制流写数 | 否 |
| `crash.rs:1917` | `persisted_writes_of_state` 的返回值 | 状态序号 → 持久集合，按**枚举用的写表**（录制流的写 + 撕裂镜像 + 重放） | 枚举写表长 | **是**，每个状态现建一份 |
| `crash.rs:1962` | `vec![false; self.writes_with_torn_images.writes.len()]` | 上一行的本体：先全 false，再按落法填 true，撕裂态另把撕裂镜像与重放那几格填上（`:1963`–`:1983`）；它之前还有一份 `landings: Vec<WriteLandingInCrashState>`（`:1920`，三值枚举，不是 bool） | 枚举写表长 | **是** |
| `crash.rs:2115` | `FinishedSlice.observed_states: Vec<(Vec<bool>, RecoveryReport)>` | 有观察者时一片里每个状态的持久集合与恢复报告，带回调用线程 | 片长 × 枚举写表长 | 有观察者时每个状态 `clone` 一份（`:2149`） |
| `crash.rs:112` | `check_records_against` 参数 `persisted: &[bool]` | 被核记录流的持久集合 | 记录流长 | — |
| `crash.rs:194` | `unit_copy_is_missing_under_the_persisted_set` 参数 | 同上，判一份单元副本缺不缺席 | — | — |
| `memory_pool.rs:434` | `CrashImage.persisted: Vec<bool>` | 崩溃镜像 = 基线 + 这些持久了的写（按写表次序叠） | 与 `CrashImage.writes` 同长 | 随状态 |
| `memory_pool.rs:796`、`:814`、`:963` | `some_publish_persisted_without_its_root`、`newest_persisted_root`、`publishes_in` 的 `&[bool]` 参数 | 同一个持久集合，各扫一遍 | — | — |
| `crash_injection.rs:203` | `CrashPoint.persisted_within_the_segment` | 崩溃点所在那一段里第 k 个写持久没有 | 段长 | 每个崩溃点一份 |
| `crash_injection.rs:803` | `vec![false; writes.len()]` 局部 `persisted` | 整条历史流的持久集合（更早段 true、当前段按子集、更晚段 false），交记录核对器与 `newest_persisted_root` | 整条历史写表长 | 是，每个崩溃点 |
| `crash_injection.rs:1192` | `HistoryThenWritableMountRecords.persisted_in_the_history` | 第一次崩溃时历史那一段的持久集合（接缝前） | 历史写表长 | 每个崩溃点 |
| `crash_injection.rs:1248` | `persisted_with_the_mount` 返回值 | 历史 + 挂载段 + 之后那次发布拼起来的持久集合 | 三段之和 | 每次核对 |
| `crash_injection.rs:1294` | `vec![true; self.writes_of_the_mount]` | 第二截：挂载那一段全落 | 挂载写数 | 每个崩溃点 |
| `crash_injection.rs:1471` | `SecondCrashPoint.persisted_within_the_segment` | 二次崩溃所在挂载段里的子集 | 段长 | 每个二次崩溃 |
| `crash_injection.rs:1509` | 局部，`draw_second_crash_points` 里造上一行 | w 之前 true、w false、之后按种子 | 段长 | 同上 |
| `crash_injection.rs:1587` | `vec![false; mount_writes.len()]` | 整条挂载流的持久集合（二次崩溃） | 挂载写数 | 每个二次崩溃 |
| `crash_injection.rs:1693` | `BTreeSet<(usize, Vec<bool>)>` | 抽到的（段号, 段内子集）去重集合 | — | 每段历史一份 |
| `crash_injection.rs:1752` | `bits_of` 返回值 | 掩码低 n 位摊成逐写的 bool | 段长 | 每个抽样 |
| `crash_injection.rs:1758`、`:1763` | `draw_a_proper_subset` 返回值与局部 | 按种子抽一个真子集（段长 ≤ 63 抽掩码，否则逐写抽） | 段长 | 每个抽样 |

### 4.2 层 0 的状态怎么分给线程

| 事实 | 出处 |
|---|---|
| 单位是「片」= 状态序号的连续区间 `Range<u64>`，首尾相接覆盖 `[0, 状态数)` | `crash.rs:2003` `state_slices` |
| 不续跑时片长按线程数定：片数 `max(64, 16 × 线程数)`，每片至少 16 个状态 | `crash.rs:1293`–`:1297` 三个常量；`:2005`–`:2013` |
| 续跑、分片、merge 时片长只看状态数：片数取 65536，每片 `max(⌈状态数 / 65536⌉, 16)` 个状态（换线程数续跑片方案不变） | `crash.rs:2029` `LAYER0_RESUMABLE_SLICE_COUNT`、`:2032` `states_per_slice_independent_of_worker_threads`、`:2043` `slicing_of_the_run` |
| 按这个切法现算三条整条流的片长：第一条 257 个状态一片、65282 片；第二条 25371 个一片、65534 片；并行线一 20737 个一片、65533 片（`python3` 现算，没跑） | — |
| 线程：`std::thread::scope` 里起 `min(线程数, 待跑片数)` 个工作线程，共用一个 `AtomicUsize` 游标 `fetch_add(1)` 领下一片（动态领片，不预分） | `crash.rs:2742`、`:2774`–`:2811`（`:2776` scope，`:2792` fetch_add） |
| 线程数：环境变量 `SINGLEFS_LAYER0_THREADS`，没设取 `available_parallelism` | `crash.rs:1290`、`:1346` `Layer0Parallelism::from_environment` |
| 一片在一个线程上按序号逐个评：`for ordinal in slice` → `persisted_writes_of_state` → `evaluate_state_recording_findings` | `crash.rs:2124` `evaluate_state_slice`、`:2136`–`:2168` |
| 交回：`mpsc::channel` 发 `FinishedSlice`；调用线程按片号次序并（先到的暂存进 `BTreeMap`），每收一片打一行 `LAYER0_PROGRESS` | `crash.rs:2778`、`:2837`–`:2913` |
| 基线 `MemoryPool`、枚举写表、计划只读、各线程共用；每个状态自己的 `Vec<bool>` 与 `CrashImage` 在工作线程上现建 | `crash.rs:2118`–`:2119` 文档 |
| P7（C561 σ）另有一份切片：掩码区间，片数 `max(64, 16 × 线程数)`，同样 scope + 游标 | `record_checker_judges_absence_by_the_persisted_set.rs:565`–`:573`、`:592`–`:623` |
| P8 / P10：种子区间切片，片数 `min(种子数, 4 × 线程数)`，同样 scope + 游标；一片 = 若干段历史，一段历史的全部崩溃点在同一个线程上串行跑 | `crash_injection.rs:2116`–`:2129`、`:2006`–`:2037`；`bad_disk_input.rs:2703`、`:2714` |
| P9：harness 档自己一份种子切片与 scope | `fault_injection.rs:2744`、`:2836` |

### 4.3 断点续跑（`layer0_progress.rs`）的单位

| 事实 | 出处 |
|---|---|
| 单位是片：一片跑完、观察者看完、没 panic 之后，把这一片的整份 `Layer0Tally` 写成一行追加进进度文件并 `sync_data` | `layer0_progress.rs:1186` `append_finished_slice`；调用 `crash.rs:2875`–`:2881` |
| 被杀时最多丢每个线程手上那一片；读回时核过的片不再跑，末尾半行丢掉、那一片重跑 | `crash.rs:2027`–`:2028` 注释；`layer0_progress.rs:8`–`:9` |
| 进度文件按「输入指纹 + 流名 + 枚举计划哈希」分格，三样都进文件名 | `layer0_progress.rs:4`–`:6` |
| 计划哈希拼的是：基线每个扇区、枚举写表（含撕裂镜像与重放）、段与每段展开方式、被判的根、版本表、状态数、片方案、有没有观察者（分片时另拼 `shard <i>/<n>`） | `crash.rs:2286`–`:2290` |
| 进度目录由 54 号设：`<git common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>` | `.claude/gate.d/54-layer0-replay.sh:23` |

### 4.4 双机分片（`research/scripts/layer0-shard-run.sh`）的单位

| 事实 | 出处 |
|---|---|
| 单位也是片（续跑那一套切法）：第 i 台只跑 `slice_index % n == i` 的片（交错分，不是切成两大段） | `crash.rs:2680`–`:2683`；`layer0_progress.rs:167` `owns_slice`、`:186` `shard_owning_slice` |
| 驱动固定两台：本机跑 0/2、第二台跑 1/2，第二台的账本拷回本机，本机 `merge/2` 读两份账本按片号并 | `research/scripts/layer0-shard-run.sh:2`–`:4` |
| 账本与进度文件同一种片行，另加文件头（工具链、输入指纹、片方案）；merge 核两份账本的文件头相同、片不重不漏 | `layer0_progress.rs:14`–`:17`；`crash.rs:2567`–`:2569` |
| 只接登记了 `shard=across-machines` 的用例：今天是第一条、第二条、并行线一三条（P1–P3）；树分裂、位置寻址不认分片开关 | `stage-inputs.tsv:36`、`:37`、`:40`；`:41`、`:42` 注释 |
| 推的：因为交错分片，第二条流里与第一条流共享的那 661 片左右（16777260 ÷ 25371 ≈ 661.3）两台各跑一半 | 现算 |

## 五　相邻状态之间差什么、镜像怎么造（问题 5）

### 5.1 层 0：一个状态的镜像不物化，是「基线 + 持久集合」的叠加视图

| 事实 | 出处 |
|---|---|
| 状态的镜像就是 `CrashImage { base, writes, persisted }`，只借用基线与写表，自己只持有 `persisted: Vec<bool>`；没有一份按状态物化的盘面 | `crates/singlefs-harness/src/memory_pool.rs:428`–`:435`；建在 `crash.rs:1142`–`:1146` |
| 每一次读（恢复与 checker 都经它读）：先从基线读（稀疏盘 `BTreeMap` 按扇区区间查，`memory_pool.rs:49`–`:66`），再**逐条扫整张写表**，持久且同盘、与读区间重叠的写把重叠那段拷上去 | `memory_pool.rs:445`–`:473`（循环在 `:454`） |
| journal 记录槽的提示、checker 的候选单元槽与候选 journal 槽，同样每次调用逐条扫整张写表 | `memory_pool.rs:478`–`:511`、`:741`–`:751`、`:752`–`:770` |
| P1–P3 的基线是 mkfs 之后那一版（`memory_pool_after_mkfs`，`common/mod.rs:194`–`:196`），不随段前移：第二条流第 60 段上的状态，每次读都要把前面几百次持久写逐条叠一遍 | 同上；写表长 477（`crash_enumeration_fixed_script_stream.rs:448`），枚举写表再接 46 条系统配置槽写各自的撕裂镜像与重放（`:97`–`:99` 注释说取三态的是 46 次系统配置槽写；重放条数没现算） |
| 每个状态的持久集合从头现建：先 `landings = vec![NotPersisted; 录制流写数]`，把当前段之前每一段的每次写标成持久（逐写循环），再拆当前段，最后新建 `vec![false; 枚举写表长]` 按落法填 | `crash.rs:1917`–`:1985` |
| 撕裂镜像的字节不在每个状态上算：枚举开始时 `WritesWithTornImages::of` 一次性算好——克隆基线一次、按次序叠写到每个原地覆写之前取旧字节 | `crash.rs:1669`–`:1682`；调用 `crash.rs:2634` |
| 造一个状态「镜像」花的函数：`Layer0StatePlan::persisted_writes_of_state`（`crash.rs:1917`）+ 结构体字面量（`:1142`）；之后的代价摊在每一次读上：`<CrashImage as PoolReader>::read`（`memory_pool.rs:445`）→ `<MemoryPool as PoolReader>::read`（`:396`）→ `SparseDevice::read_into`（`:49`）+ 逐写 `WrittenContents::copy_range_into`（`:261`）；`ImageReader` 那一侧 `memory_pool.rs:726`–`:771` 转回同一个 `read` | 同左 |
| 一个状态上这套叠加视图被读几遍：两遍恢复、一遍池级 checker、一遍记录核对器（1.3 第 4、6、7 步）各自独立读，互不共享读出来的字节 | `crash.rs:1147`、`:1148`、`:1210`、`:1247` |

### 5.2 同一段内相邻两个状态差什么

| 事实 | 出处 |
|---|---|
| 段内序号按混合进制拆，第一次写是最低位、每位的进制是它能取几态（2 或 3） | `crash.rs:1875`–`:1895` `assign_landings` |
| 所以序号 r → r + 1 就是这个混合进制数加一：最低位那次写换一态，进位时连着换后面几次写。两个相邻状态的持久集合只在这几次写上不同，镜像只在这几次写罩住的字节区间上不同，每一处都是**整次写**（或整次写的撕裂镜像及其重放），不会差半次写（推的，从 `assign_landings` 与 `persisted_writes_of_state` 读出；没有用例量过「相邻状态平均差几次写」） | 同上；`crash.rs:1962`–`:1983` |
| 次序不是格雷码：全两态的段上，r 到 r + 1 平均要翻约 2 次写（二进制加一翻动的位数期望，推的） | — |
| 跨段：段 k 的最后一个序号是「最低位取次大、其余取最大」（全两态时 = 段内第一次写没持久、其余持久），下一个序号是段 k + 1 的序号 0（段 k 全持久、段 k + 1 全不持久），两者只差段 k 的第一次写（推的） | `crash.rs:1420`（段内状态数 = 组合数 − 1）、`:1898`–`:1901` |
| 甲二快档：有单元写的段里序号的奇偶决定「单元写全不落 / 全落」，相邻两个状态差这一段的**全部**单元写 | `crash.rs:1947`–`:1957` |
| 今天没有任何增量：每个状态的持久集合、叠加视图、两遍恢复、checker、记录核对器都从头来，不看上一个状态 | 1.3 那张表；`crash.rs:2136`–`:2168` 的循环体里没有跨状态携带的变量（只有 `tally` 与 `observed_states`） |

### 5.3 另几条路径怎么造镜像

| 路径 | 怎么造 | 出处 |
|---|---|---|
| P8 崩溃注入 | 基线随崩溃点段号只往前叠（增量），叠加视图里只放当前段那几次写，读的代价只随段长涨；但每个崩溃点为可写挂载**整份克隆**一次物化镜像（`materialized_crash_image`：`base.clone()` + 叠持久的写），二次崩溃的第二份基线再克隆一次 | `crash_injection.rs:800`–`:815`、`:915`、`:1049`–`:1057`、`:1574` |
| P7 σ | 与层 0 同一个 `CrashImage`，写表是整条历史，每个状态 `persisted.to_vec()` | `record_checker_judges_absence_by_the_persisted_set.rs:373`–`:377`、`:607`–`:610` |
| P9 故障注入 | 每个注入点整段历史重跑一遍，再从录制流整份重建镜像（`MemoryPool::with_devices` + `apply`） | `fault_injection.rs:2263`–`:2267` |
| P4、P5、spill-over | 基线是被录那一次发布之前的整份镜像（整段 `apply` 物化一次），写表只有那一次发布 | `common_tree_split/mod.rs:401`–`:406` |

## 六　顺带看到的数对不上（只记，不改）

C577（发布返回之前加屏障，轮换自成一段）之后用例里的常量改了，下面几处文字还是改之前的数。没跑 52 号，不知道它今天判不判红。

| 位置 | 写的 | 代码今天钉的 |
|---|---|---|
| `.claude/kb/layout/01-first-txn.md:410` | 第一条流 `2+2+1+2+2+1+26+2+1+2`、67108885 个状态，种类串里 `[unit_write×24,system_configuration_slot×2,barrier]` 同段 | `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]`（`crash_enumeration_new_pool_file_creation_stream.rs:466`），两态闭式 16777240（`:442`），三态 16777260（`:446`） |
| `.claude/kb/layout/01-first-txn.md:412` | 第二条流 61 段、`closed_form=6649413746`、快档 232 | 78 段（`crash_enumeration_fixed_script_stream.rs:439`–`:442`，现数 78 个数）、两态 1662648449（`:96`）、三态 1662648564（`:100`）、快档 278（`:114`） |
| `.claude/gate.d/stage-inputs.tsv:39` 注释 | σ「2^18」 | 65536（`record_checker_judges_absence_by_the_persisted_set.rs:731`–`:733`，`:712` 注释「C577 之前 σ 带上一次的轮换、2^18 个」） |
| `.claude/gate.d/stage-inputs.tsv:40` 注释 | 并行线一「枚举域 12230590578 个状态」 | 1358954634（`crash_enumeration_multi_record_publish_stream.rs:21`，同行写 12230590578 是 C577 之前的） |
| `crash_enumeration_new_pool_file_creation_stream.rs:497` ignore 串 | 「全量六千七百多万个状态」 | 16777260 |
| `crash_enumeration_fixed_script_stream.rs:796` ignore 串 | 「全量五十多亿个状态」 | 1662648564 |
| `crash.rs:2028` 注释 | 「第二条流全量五十多亿个状态时每片约八万五千个」 | 按 `:2029`–`:2037` 现算 25371 个一片 |
| `.claude/kb/milestone/03-third-txn.md:46` | `Vec<bool>` 在 crash.rs「7 处」、crash_injection.rs「5 处」（2026-09-26 现查） | 这一次按「非测试区含 `Vec<bool>` 的行」数：crash.rs 7、crash_injection.rs 9（口径见 4.1；那一次的口径没写，不知道差在口径还是代码） |

## 七　什么现象会推翻这份事实表里的结论

| 结论 | 推翻它的观测 |
|---|---|
| 三条从 mkfs 起的流前 11 段逐字节相同、第一条流的全部 16777260 个状态在另两条里各重复一遍 | 在同一次编译里分别 `prepare` 三条流，比 `writes[..41]`（连同内容）与基线：有一条不等就不成立（今天没有这条用例；只做过 mkfs 的池那条用例是唯一的逐字节比对） |
| 共享前缀上三条流的判定相同 | 同一个前缀状态在第一条与第二条流里 oracle / checker / 记录核对器判得不同。已知会不同的只有计数口径：`root_persisted_states` 按各自的被判根（第一条是 A 的根，第二条是 txg 19 的根）数，版本表不同（第一条单版本 `single_version`，`:391`），第一条多一个观察者；oracle 的 `find` 按 (txg, 实例) 精确找、`NoFile` 那一支只找更旧的版本（`crash.rs:993`–`:1019`），推的：前缀状态上结论相同 |
| 流走到哪些模块没有机械登记 | 仓里找到一份按流列 `singlefs-core` 模块或函数的登记、或一份覆盖率产物；这一次搜了 `stage-inputs.tsv`、`admission.py`、`crash-case-check.py`、`llvm-cov`/`tarpaulin`/`grcov` 全仓字面 |
| 层 0 每个状态从头建持久集合、不物化镜像、读的时候逐写叠加 | `persisted_writes_of_state` 或 `CrashImage::read` 里出现跨状态的缓存或增量；今天两处都没有（`crash.rs:1917`–`:1985`、`memory_pool.rs:445`–`:473`） |

## 八　没做什么

- 没编译、没跑任何测试或门禁（派发要求；另一会话的层 0 全量占着两台机器），所以文中所有状态数都是用例常量或现算，不是这一次跑出来的。
- 没有写一条「三条流前缀逐字节相同」的比对去跑，只做了推论（第七节第一行是它的推翻办法）。
- 没现算枚举写表里撕裂镜像之后的重放条数（第二条流 46 条撕裂镜像之外还接几条重放）。
- 没数位置寻址那几条流的状态数（用例不钉数，只断言等于闭式）。
- 相邻状态平均差几次写、每个状态花在哪一步，没有量（后者归任务乙与 E161）。
- 第六节列的不一致没改，也没判 52 号今天红不红。
```

**出处 `research/prompts/m3-prune-gpu-r1-facts-checker-gpu.md:1-327`（整段抄，未转述）**

````markdown
# m3-prune-gpu-r1 事实调查：池级 checker 流水线、可 GPU 化部分、E163 / E161 可复用件

调度记录 `records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 任务乙。只读代码、kb、产物；没编译、没跑任何测试。行号都是 `grep -n` / `sed -n` 现取的文件自己的行号。标「推的」的是读代码推出来、没量过的。

## 1　池级 checker 与记录核对器依次做什么、分三类

### 1.0 每个崩溃状态上的流水线（层 0）

`crates/singlefs-checker-tier/src/crash.rs:1131` `evaluate_state_recording_findings` 对一个状态依次做：

| 次序 | 调用 | 行 | 读什么 |
|---|---|---|---|
| 1 | `newest_persisted_root(writes, &persisted)` | crash.rs:1141 | 写表 + 持久集合 |
| 2 | 组 `CrashImage { base, writes, persisted }` | crash.rs:1142-1146 | 不拷字节，读时现叠 |
| 3 | `recover(&image, JournalPolicy::Consult)`、`recover(&image, JournalPolicy::Ignore)` | crash.rs:1147-1148 | singlefs-core 的恢复，两遍 |
| 4 | `classified_oracle_violation_for_versions` 两遍（看 / 不看 journal） | crash.rs:1168、1190；函数在 crash.rs:968 | 恢复结果 vs 录制时发布过的版本（oracle） |
| 5 | `check_pool_image(&image)` | crash.rs:1210 | 池级 checker，一元谓词 |
| 6 | `check_records(&image, consulted.effective_root)` | crash.rs:1247；函数在 crash.rs:48 | 记录核对器，四元入参 |

`CrashImage` 的读法：每次 `read` 先从基线池读、再把写表里每一条持久了的写逐条叠上去（`crates/singlefs-harness/src/memory_pool.rs:445-469`，循环 `for (write, is_persisted) in self.writes.iter().zip(&self.persisted)`），即每次读是 O(写表长 W)（推的：按代码结构，没量过）。扫描方向的候选槽 = 基线写过的扇区所在槽 ∪ 持久了的 `UnitWrite` 所在槽（memory_pool.rs:741-751），journal 候选同理（memory_pool.rs:752-770）。

### 1.1 池级 checker `check_pool_image`（`crates/singlefs-checker/src/walk.rs:5656-6255`）依次做的事

报 49 条不变量（`crates/singlefs-checker/src/image.rs:68` `IMPLEMENTED_INVARIANTS: [&str; 49]`）。按函数里的次序：

| # | 步骤（行） | 调用 | 判的不变量 | 类 | 每状态数据量（出处） |
|---|---|---|---|---|---|
| 1 | walk.rs:5657 | `chosen_system_configurations` → `verified_system_configuration_slots` → `system_configuration_slot_readings`（image.rs:571 / 487 / 456）→ `check_system_configuration_slot`（lib.rs:245）+ `geometry_of`（image.rs:241） | 择槽；槽内 magic、整槽 CRC、格式版本、incompat、加密类型、几何上下界 | a | 每盘 2 槽 × 4096 字节；整槽 CRC 用按位的 `crc32_castagnoli_bitwise`（lib.rs:89 经 `checksum_field_holds`） |
| 2 | walk.rs:5676-5689 | `judge_system_configuration_values_the_reader_accepts`（image.rs:510），再调一次 `system_configuration_slot_readings` | I-7.13 | a（逐槽） | 同上，再读一遍 |
| 3 | walk.rs:5701-5712 | 各盘择到的 fsid 逐盘比 | I-1.4 | b（跨盘） | 标量 |
| 4 | walk.rs:5728-5732 | 根环区域设备分布 | I-7.6 | b | 标量 |
| 5 | walk.rs:5738 | `valid_roots`（image.rs:613）→ `check_root_slot`（lib.rs:318） | 根槽 magic、整槽 CRC（按位）、fsid、flags | a | R × S 个根槽 × `physical_block_size` 字节（root_slot_positions，image.rs:593） |
| 6 | walk.rs:5744-5756 | `verified_system_configuration_slots` 第三次读系统配置槽 | — | a | 同 1 |
| 7 | walk.rs:5757 | `judge_own_device_number_is_the_device_identity`（image.rs:549） | I-7.14 | a（逐槽比盘身份） | 标量 |
| 8 | walk.rs:5761 | `judge_system_configuration_floor_against_the_roots_on_each_device`（walk.rs:4118） | I-7.12 | b（系统配置 F vs 同盘根） | 标量 |
| 9 | walk.rs:5767 | `judge_instance_carriers`（walk.rs:2529）→ 第四次读系统配置槽（walk.rs:2535）+ `instance_carriers`（walk.rs:2470）：重读全部根槽、整条 journal 候选槽逐条 `check_journal_record`、单元区候选槽逐槽读 4096 字节头 `unit_write_order_instance`（walk.rs:2440） | I-7.7 | b（跨全盘取最大号） | journal 每槽 4096 字节（整条 CRC 按位）；单元头每槽 4096 字节（`UNIT_HEADER_SCAN_BYTES`，walk.rs:2436），头 CRC 按位 |
| 10 | walk.rs:5769-5770 | `scanned_journal_records_by_device`（walk.rs:4996）→ `scanned_journal_records_of_device`（walk.rs:4848）：每条候选记录 `check_journal_record`（lib.rs:733）+ `back_chain_of_record_header`（lib.rs:726） | 记录自身：magic、`header_csum` 罩整条 4096（按位）、类型、长度、fsid、载荷 CRC（按位）、点名项 flags | a（逐条） | 每条 4096 字节，两次按位 CRC（整条 + 载荷）+ 311 字节头一次按位 CRC |
| 11 | walk.rs:5771-5781 | `judge_journal_back_chain`（5027）、`judge_location_order_of_journal_named_entries`（4901）、`judge_transaction_numbers_per_instance`（5099）、`judge_commit_markers_per_transaction`（5183）、`judge_publish_ordinals_and_last_record_flags`（5316） | I-8.6、I-2.5（点名项那一半）、I-8.7、I-8.8、I-8.9 | b（记录之间；I-2.5 那一半逐条，a） | 记录解出来的标量 |
| 12 | walk.rs:5782-5792 | 根环非空；`judge_root_ring_health`（4772） | I-7.1、I-7.3 | b | 标量 |
| 13 | walk.rs:5800-5808 | `Walk::starting_with` + `walk_root`（walk.rs:854）走最新根 | 见 1.2 | b（主体） | 见 1.2 |
| 14 | walk.rs:5820-5830 | 最新根走读有没有失败、有没有 I-2.1 违例 | I-4.8、I-7.4（最新根那一格） | c（候选集里的一条） | 标量 |
| 15 | walk.rs:5837-5897 | 按最新根指着的实例表剔被抛弃的根、按 F 生效值剔 F 之下的根，定候选集 `candidate_indexes`；F 生效值取各盘最新有效根的 F 与系统配置 F 的最大值（5866-5874） | — | c（定回退候选集） | 标量 |
| 16 | walk.rs:5898-5921 | 候选集里其余每条根 `walk_root(.., false)`，每条各判一格 | I-7.4、I-4.8（每条候选根），走读内部照判 1.2 那些 | c | 每条候选根一遍走读（`visited_units` 共享，走过的单元不重走，walk.rs:262-265 注释） |
| 17 | walk.rs:5925-5951 | `versions_applied_only_by_records`（5507）：由 journal 记录施加出来、根槽没落盘的那几版，每版 `walk_version_applied_only_by_records`（890） | I-7.4、I-4.8（每版） | c | 同 16 |
| 18 | walk.rs:5952-5960 | `judge_blocks_referenced_by_abandoned_roots`（5582）：重读全部根槽判读不读得出；每条被抛弃的根另起一个 `Walk` 走一遍 | I-7.4（被抛弃时间线那一半） | c | 每条被抛弃根一遍完整走读（不共享 visited） |
| 19 | walk.rs:5979-5991 | 最新根走读断没断；被引用单元数为 0 时报不适用 | I-7.2；I-1.2、I-4.2 的不适用 | b | 标量 |
| 20 | walk.rs:5994-6006 | `scanned_content_units`（4625）：单元区候选槽逐槽读 4096 字节头 `content_unit_of_header`（4572，头 CRC 按位 + fsid + 已发布谓词）；`judge_merged_version_total_order`（4672）按类身份段归并成组，组内载荷校验和字段不同时才 `payload_checksum_holds_on_disk`（4654，整单元查表 CRC） | I-1.8 | b（跨单元归并） | 单元头每槽 4096 字节；整单元只在组内不一致时读 |
| 21 | walk.rs:6008-6048 | `IndexNodeCache`（`BTreeMap<(u32, u64, u32), Option<IndexNodeView>>`，walk.rs:2762）共用；`judge_release_generation_and_tree_table_birth`（3395）、`judge_allocation_records_disjoint`（3457）、`allocation_record_node_pointers_of_the_candidate_versions`（3586）、`judge_allocation_generations_against_unit_births`（3634）、`judge_rollback_floor_raises_against_their_ceilings`（4024） | I-3.9、I-9.14、I-5.4、I-3.10、I-7.9 | c（按候选集里每条根读树表与树节点） | 按候选根读节点，缓存按 (盘, 槽, 校验和) 去重；读节点用 `read_index_node_without_judging`（2764）→ `index_node_view`（lib.rs:447），CRC 按位 |
| 22 | walk.rs:6053-6074 | 记账里的 inode 号水位 vs 遍历侧最大 inode key | I-9.6 | b | 标量 |
| 23 | walk.rs:6075-6089 | inode 树走没走到；数据单元对象出生代 vs inode 记录 | I-9.12 不适用、I-9.10 | b | 标量表 |
| 24 | walk.rs:6091-6107 | `references` 按盘排序、相邻两段不重叠 | I-5.1 | b | 引用条数 |
| 25 | walk.rs:6131-6229 | `slots_referenced_per_device`（5644）；`quarantined_slots_exempted_per_device`（4244）两遍（全部走过的版本 / 只最新根）；记账行 vs 遍历和、vs 单元区容量、减 defer | I-3.1、I-5.2、I-3.11 | c（I-3.1 对候选集并集；I-3.11 对最新根）；I-5.2 是 b | 记账行 + 引用表 |
| 26 | walk.rs:6232-6253 | `scanned_tree_identifiers`（2396）：单元区候选槽逐槽读 **16384 字节**（`node_bytes()`，walk.rs:2410）判码 2 头 CRC（按位）+ 诞生代 + 实例表；与走过的树表树 ID 并起来取最大，对根环水位 | I-7.8 | b | 单元区候选槽每槽 16384 字节 |

⚠️ 单元区候选槽整轮扫三遍（第 9、20、26 步），系统配置槽读四遍（第 1、2、6、9 步），根槽读三遍（第 5、9、18 步），journal 候选槽读两遍（第 9、10 步）。「几遍」是数代码里的调用点，推的，没在跑的时候数过。

### 1.2 走读（`Walk`，walk.rs:253）里每读一个被引用单元做的事

每跟一条指针（例：码 2 节点 `read_index_node`，walk.rs:790-852）：

| 次序 | 调用（行） | 判的 | 类 | 数据量 |
|---|---|---|---|---|
| 1 | `parse_node_pointer`（image.rs:669）；`judge_location_order`（image.rs:710）；`judge_pointer_mac_and_nonce_are_zero`（image.rs:699） | I-2.5、I-2.4（指针头 MAC 16 + nonce 12 恒 0） | a（只看父单元里那 86 字节指针） | 每指针 86 字节 |
| 2 | `note_references_of_a_pointer_followed_by_the_walk`（walk.rs:633）→ `note_reference`（561）、`judge_a_placement_referenced_in_this_version`（588） | I-5.1（同一版里同一落点被引两次） | b | 引用表 `BTreeMap<(u32, u64, u64), String>` |
| 3 | `read_referenced_unit`（image.rs:734）：两条位置条目各读一份整单元，`crc32_castagnoli_table` 比位置条目里的校验和（image.rs:746） | I-2.1 | a（一份单元字节 + 父指针里的 4 字节校验和） | 每指针 2 × 16384 或 2 × 32768 字节，查表 CRC 两次 |
| 4 | `visited_units.insert`（walk.rs:818 `if !self.visited_units.insert(placement)`）：走过的不再往下走 | — | b | — |
| 5 | `judge_unit_header`（walk.rs:663）：类标签 / flags / 类身份段首字节（I-1.6）；头校验和罩 [0, 头末)（I-2.4，`checksum_field_holds` 按位，walk.rs:700）；格式版本（I-2.4）；声明长度之后补齐为 0 + 载荷 CRC（I-2.3，查表，walk.rs:725）；头里 fsid vs 池 fsid（I-1.4）；29 字节加密预留位恒 0（I-2.4） | I-1.6、I-2.4、I-2.3、I-1.4 | a（池 fsid 是全池一个标量入参） | 头 ≤ 625 字节按位 CRC + 载荷整段查表 CRC |
| 6 | `judge_birth_identity_of_a_referenced_unit`（walk.rs:482） | I-1.2、I-4.2（按挂载根与实例表的已发布谓词） | b（要最新根的实例表） | 头里几个字段 |
| 7 | `index_node_view_judging_a_zero_entry_width`（walk.rs:761）→ `index_node_view`（lib.rs:447）→ `check_unit`（lib.rs:374）：**再判一遍**头校验和（按位）与载荷 CRC（**按位**，lib.rs:409），再核声明长度 = 条目数 × 条目宽、条目宽 ≥ key 宽、条目宽 0 时条目数 0；切出 `entries: Vec<Vec<u8>>` | I-1.10（条目宽 0 那一格） | a | 同一单元整段再按位 CRC 一次 |
| 8 | 头里树 ID vs 引用它的树（walk.rs:832） | I-1.3 | b（期望值来自父） | 标量 |
| 9 | `check_index_node_keys`（lib.rs:567）或 `check_internal_node_separators`（lib.rs:590）：key 宽 = 形态宽、条目 key 按字段序严格递增、首末条目贴紧头里 key 区间 | I-1.1 | a（形态由树的种类定，是父传下来的参数） | 条目数 × key 宽 |
| 10 | 子树覆盖区间、分隔 key 与孩子区间（`walk_code_two_subtree` 1107、`judge_separators_against_the_inode_children` 1746）、条目宽 = 字段表宽（`entry_width_holds` 1253、`position_and_entry_width_hold` 2003）、树表条目次序（`judge_tree_table_entries_ordering` 2611）、实例表行（`judge_instance_table_rows` 1874）、inode 树（1405-1780）、打包容器（`judge_packed_container` 1913）、中央映射 key vs 单元头（`walk_central_mapping_entries` 987，判在 1000、1025、1052、1066、1082）、extent 与分配记录树（2041-2376） | I-1.1、I-1.7、I-1.10、I-1.11、I-3.8、I-9.1、I-9.2、I-9.4、I-9.7、I-9.12、I-9.13、I-9.15、I-9.16 | b | 父子两层或整棵树 |

码 2 节点一次被引用，整单元 CRC 共算三遍：第 3 步查表（整单元）、第 5 步查表（载荷）、第 7 步按位（载荷，经 `check_unit`）；头校验和按位算两遍（第 5、7 步）。推的：数的是代码路径，没计时。

### 1.3 按不变量归类（49 条）

归法：a = 只看单个单元（或单个槽、单条记录、单条指针）的字节，外加全池一两个标量（池 fsid）就能判；b = 要跨单元、走树、跨盘、跨记录；c = 要按回退候选集、记账对遍历并集这类「集合对集合」判（池级 checker 里没有 oracle，oracle 在 crash.rs 那一侧）。一条不变量在几处判的，按它最重的那一处归。

| 类 | 不变量（判它的函数） |
|---|---|
| a | I-2.1（`read_referenced_unit`，但「对哪几条根判」是 c）、I-2.3、I-2.4、I-1.6（`judge_unit_header`）、I-2.5（`judge_location_order` / `judge_location_entries_order` / `judge_location_order_of_journal_named_entries`）、I-7.13（`judge_system_configuration_values_the_reader_accepts`）、I-7.14（`judge_own_device_number_is_the_device_identity`）；外加不单列编号、走读前提的「单元自证」：`check_unit`、`check_root_slot`、`check_system_configuration_slot`、`check_journal_record` |
| b | I-1.1、I-1.2、I-1.3、I-1.4、I-1.7、I-1.8、I-1.10、I-1.11、I-3.8、I-4.2、I-5.1、I-5.2、I-7.1、I-7.2、I-7.3、I-7.6、I-7.7、I-7.8、I-7.12、I-8.6、I-8.7、I-8.8、I-8.9、I-9.1、I-9.2、I-9.4、I-9.6、I-9.7、I-9.10、I-9.12、I-9.13、I-9.15、I-9.16 |
| c | I-3.1、I-3.9、I-3.10、I-3.11、I-4.8、I-5.4、I-7.4、I-7.9、I-9.14（第 15-18、21、25 步） |

按函数名 grep 各不变量字符串所在函数（`awk` 取每个 `"I-x.y"` 所在的 `fn`）现查过；I-1.11、I-7.13、I-7.14 用的是常量名（image.rs:40、50、57），按常量名查。

### 1.4 记录核对器（`crates/singlefs-checker-tier/src/crash.rs`）

入参（crash.rs:41 文档注释，那一行的前半逐字）：「记录核对器（D13（验证路线） 已定项 7，故意不给它编号；入参 (崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)）：崩溃前镜像是 `image.base`、记录流是 `image.writes`、崩溃后镜像是 `image` 本身、持久集合是 `image.persisted`」。D13 已定项 7 原文在 `.claude/kb/decisions/13-验证路线.md:133`（「需要第二个输入的核对归记录核对器，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`」起那一行），射程在 :135（「按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）」在那一行里）。

| 次序 | 调用（行） | 判什么 | 类 | 数据量 |
|---|---|---|---|---|
| 1 | `check_records`（crash.rs:48）→ `check_records_against`（crash.rs:105），层 0 两份崩溃后镜像传同一份 | — | — | — |
| 2 | `instance_table_of_the_effective_root`（crash.rs:172）：从写表里倒着找写出恢复所落根身份的那次根槽写，拿它的字节 `InstanceTableOfRootRecord::read`（walk.rs:3834）沿链读实例表 | 恢复落到的那一版哪些发布被抛弃 | c | 一条根记录 + 实例表链 |
| 3 | `publishes_in(writes, persisted, continuity)`（singlefs_harness::memory_pool）逐次发布 | — | c | 写表 W 条 |
| 4 | 第一条判据：`in_place(publish.root) && !publish.records.iter().any(in_place)` → `root_without_record`（crash.rs:139-143）；`in_place` 在崩溃态镜像上读写的区间、`write.contents.still_on_disk(&bytes)` 逐字节比（crash.rs:121-126） | 根在而记录一条都不在 | c（对记录流） | 每次发布：根槽 + 各记录的区间 |
| 5 | 第二条判据：恢复自称的 txg ≥ 这次发布且没被抛弃时，按偏移把单元副本分组，某组每份副本都缺席 → `claimed_state_missing_unit`（crash.rs:145-159） | 声称的状态缺单元 | c | 每份副本 |
| 6 | 缺席判定 `unit_copy_is_missing_under_the_persisted_set`（crash.rs:191）：按 512 字节扇区逐个看还是不是这份副本的字节；不是的，在 `copy + 1..writes.len()` 里找持久了、落在这个扇区上、过了回收谓词的更晚写（crash.rs:220-239） | 同上 | c | 每份副本 长度 ÷ 512 个扇区 × 最多 W 次后写 |
| 7 | 回收谓词 `reuse_is_not_proven_illegal_by_the_reclaim_predicate`（crash.rs:293）：扫 `writes[..later_index]` 里的根槽写与系统配置槽写取三个界（系统配置槽写要 `SystemConfiguration::parse_slot`，singlefs-core） | C513 那一格 | c | 每个后写扫一遍写表前缀，按后写下标缓存（crash.rs:207） |

记录核对器 `use` 了 `singlefs_core::system_configuration::SystemConfiguration`（crash.rs:33）、`singlefs_core::recovery`（crash.rs:19）与 `singlefs_harness::memory_pool`（crash.rs:34）：它住在 checker 档，不是池级 checker，不受门禁 94 号 ①② 约束（见第 6 节）。

## 2　(a) 类在 GPU 上算要怎么重映射；今天实现里没有 GPU 对应物的东西

### 2.1 逐项重映射（推的：读代码与 E163 装置得出的形态，没写、没量）

| (a) 项 | 今天的入参 / 出参 | GPU 上要映射成 | 着色器里要算的 |
|---|---|---|---|
| 整单元校验和 vs 位置条目（I-2.1，image.rs:746） | `reader.read(..)` 交回 `Option<Vec<u8>>`；比父指针里的 u32 | 去重后单元内容的扁平缓冲 `array<u32>`，每单元占定长槽（E163 取 32768 字节 = 8192 字，rs:236）+ 每单元长度数组（E163 `unit_lengths`，rs:231）；每状态一张「(盘, 槽) → 内容下标」表 | 整单元 CRC-32C，出一个 u32；比较留在 CPU 或另传期望值数组 |
| 单元头自证 `check_unit`（lib.rs:374） | `&[u8]` → `Result<u8, Verdict>` | 同一扁平缓冲；出参改成每单元一个 u32 判定码（`Verdict` 27 个成员都不带载荷，lib.rs:96-165，`awk 'NR>=96&&NR<=165 && /^    [A-Z][A-Za-z]+,$/' … | wc -l` 数得 27，可编成整数） | 头宽按类标签与 `unit[51]` 现算（码 2 头末 = 86 + 2k，lib.rs:354-370）；头 CRC 要把偏移 10 起的 32 字节当 0 参与（`checksum_field_holds`，lib.rs:82-92 今天是 `to_vec()` 再 `fill(0)`）；载荷 CRC；格式版本；29 字节预留位全 0 |
| `judge_unit_header` 的 I-1.6 / I-2.3 / I-2.4 / I-1.4（walk.rs:663） | `&mut self` 上累加 `Judgements`，失败时 push `String` 进 `walk_failures` | 每单元一组位旗标（每条不变量一位）+ 期望类标签（来自父指针的种类，要 CPU 先给）+ 池 fsid 低 8 字节（uniform） | 同上，再加声明长度之后补齐全 0 |
| `index_node_view` + `check_index_node_keys` / `check_internal_node_separators`（lib.rs:447、567、590） | 产出 `IndexNodeView { entries: Vec<Vec<u8>>, smallest_key: Vec<u8>, .. }`（lib.rs:432-444）；`KeySchema::fields` 每条 key 现算一个 `Vec<u64>` 再按 `Vec` 字典序比（lib.rs:533-550、573） | 不切条目：按 (条目区起点 = 115 + 2k, 条目宽, 条目数) 在缓冲里现算偏移；key 形态表（5 种，lib.rs:511-524）编成常量数组 + 树种类参数 | 逐条相邻 key 按字段比；字段宽 1/2/4/6/8 字节，6、8 字节字段在 WGSL 里要拆成两个 u32 比（见 2.2） |
| `packed_unit_view`（lib.rs:622） | `records: Vec<Vec<u8>>` | 同上按 (136, 记录宽, 记录数) 现算 | 声明长度 = 记录数 × 记录宽、记录宽 0 那一格 |
| `check_system_configuration_slot` + `geometry_of`（lib.rs:245、image.rs:241） | 4096 字节槽 → `Result<SystemConfigurationView, Verdict>` | 每盘 2 槽，量极小（E161 `g5_contents` 报 fixed_structure 总共 35328–106496 字节，见第 4 节） | 不值得单独上 GPU（推的） |
| `check_root_slot`（lib.rs:318） | R × S 槽 | 同上 | 同上 |
| `check_journal_record`（lib.rs:733）、`back_chain_of_record_header`（lib.rs:726） | 4096 字节 → `Result<JournalRecordView, Verdict>`（含 `named: Vec<NamedEntryView>`、`new_root_segment: Vec<u8>`） | 每条 4096 字节定长槽；出判定码 + 反向链要用的 311 字节头 CRC | 整条 CRC（偏移 46 那 32 字节当 0）、载荷 CRC（311 到 311 + 56 × 点名项数）、点名项 flags；E161 报 journal 去重后只有 2–7 种内容（第 4 节），量极小 |
| 扫描方向的单元头（walk.rs:2440、4572、2396） | 每候选槽读 4096 或 16384 字节，只用头 | 与整单元同一个缓冲；只算头 CRC | 头 CRC + fsid + 几个字段抽出来交回 CPU（已发布谓词要实例表，是 b） |

### 2.2 今天的实现里在 GPU 上没有对应物的东西

| 东西 | 在哪 | GPU 上的处境 |
|---|---|---|
| `&dyn ImageReader` 按需读、交回 `Option<Vec<u8>>` | image.rs:19-28；每次读经 `CrashImage` 叠写表（memory_pool.rs:445-469） | 没有 trait 对象、没有按需读盘：要在 CPU 侧先把要判的单元字节聚成扁平缓冲再派发 |
| `Vec<u8>`、`Vec<Vec<u8>>`、`to_vec()` 拷贝 | lib.rs:87、350、443、453、487、619、647；walk.rs 里普遍 | 着色器里没有堆分配；定长槽 + 偏移现算 |
| `BTreeMap` / `BTreeSet`（引用表、`visited_units`、`IndexNodeCache`、记账行、`Judgements` 本身） | walk.rs:258-302、2762；image.rs:122-128 | 没有有序映射；这些都属 b / c 类，留在 CPU（推的） |
| `Judgements::judge(.., detail: impl FnOnce() -> String)` 生成违例说明文字 | image.rs:132 | 没有 `String`、没有闭包：GPU 只能交判定码 / 位旗标 + 第一处违例的单元下标，文字由 CPU 事后按下标重算 |
| `Result<_, Verdict>` 与 `?` 早退 | lib.rs:245-289、374-427、447-503 | 没有 `Result`；改成判定码，分支按码走（WGSL 有 `if`/`switch`/`loop`） |
| `assert_eq!` / `expect` / `panic!` | lib.rs:468（`assert_eq!(offset, 86 + 2 * key_width, ..)`）、lib.rs:544（未登记字段宽 `panic!`）、各处 `try_from(..).expect` | 着色器里不能 panic；越界读的行为由 WGSL 规范定（不在这次核的范围内，未核） |
| `u64` 标量（fsid 低 8 字节、txg、树 ID、key 字段）与 6 字节整数 `read_six_byte_unsigned`（lib.rs:231） | lib.rs:228-241 | WGSL 核心没有 64 位整数；wgpu 有 native-only 特性 `SHADER_INT64`（`wgpu-types-30.0.1/src/features.rs:1125` `const SHADER_INT64 = 1 << 37;`，文档注释「Allows shaders to use i64 and u64」「Supported platforms: - Vulkan …」「This is a native only feature.」）；E163 请求的是 `wgpu::Features::empty()`（e163 rs:294），要 u64 就得开这个特性或拆成两个 u32 |
| CRC 查表 `OnceLock<[u32; 256]>` | lib.rs:58 | 可放 storage / uniform 缓冲或着色器常量数组；E163 的着色器用的是按位（rs:238-273），没有查表 |
| `#![forbid(unsafe_code)]` | 五个 crate 的 lib.rs 都有（checker lib.rs:7、core lib.rs:7、harness lib.rs:5、checker-tier lib.rs:4、format lib.rs:11） | wgpu 的 API 是安全的；E163 把 `&[u32]` 转字节用的是逐个 `to_le_bytes`（rs:326-332），没用 `unsafe` 或 bytemuck |
| wgpu 默认上限 | `wgpu-types-30.0.1/src/limits.rs:441` `max_storage_buffer_binding_size: 128 << 20, // (128 MiB)`、:443 `max_buffer_size: 256 << 20, // (256 MiB)`（`Limits::default()` 调 `defaults()`，limits.rs:344-347） | E163 用 `wgpu::Limits::default()`（rs:295）：一次绑定的单元缓冲超过 128 MiB（32768 字节槽约 4096 个单元，推的算术）就要提高 `required_limits` 或分批派发 |

## 3　E163 装置里能直接复用的件

装置 `research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs`（981 行，`wc -l`），下文「rs:」指这份文件的行号。

| 件 | 位置 | 做什么 | 搬进 `crates/` 时要改的（推的，没改没编） |
|---|---|---|---|
| WGSL 着色器 `COMPUTE_SHADER_SOURCE` | rs:216-275 | 4 个绑定：`params`（uniform，unit_count + 3 个预留）、`unit_words: array<u32>`、`unit_lengths: array<u32>`、`results: array<ResultEntry{crc, local_offset}>`（rs:229-232）；`@workgroup_size(64)`（rs:238）；一个调用算一个单元，按位反射 CRC-32C，字节按 `(word >> 8*(k%4)) & 0xFF` 取 | 槽宽写死 `WORDS_PER_UNIT_SLOT = 8192u`（rs:236，32768 字节），16384 字节的码 2 节点要么占半个空槽、要么另开一个槽宽参数；从槽首算到 `length_in_bytes`，不支持「从偏移 X 起算」（载荷 CRC 要从头末起）和「某 32 字节按 0 参与」（头校验和） |
| CPU 参照 `crc32c_reference` | rs:30-44 | 按位反射 CRC-32C，与着色器各持一份常量（rs:24、26 与 rs:234-235） | 池级 checker 已有两份（lib.rs:38 按位、lib.rs:57 查表），不用搬 |
| 锚点 `EXPECTED_ANCHOR_VALUES` / `anchor_vectors` / `anchor_mismatches` | rs:47-88 | 8 个锚点的绝对值，GPU 与 CPU 各自比绝对值（页面「路径与结论登记」表下那段：两份同改错时只有绝对值检查抓得到） | 可原样当阳性对照 |
| 适配器枚举 `new_vulkan_instance`、`nvidia_discrete_adapters` | rs:183-195 | 只开 Vulkan 后端；过滤 vendor 0x10DE 且 `DeviceType::DiscreteGpu` | 可原样 |
| 上下文 `build_gpu_context` | rs:290-324 | `request_device`（`Features::empty()`、`Limits::default()`、`MemoryHints::Performance`），再分配一块 4 字节 storage 缓冲当「起得来」的探针，建管线 | 要 u64 就加 `SHADER_INT64`；批大于 128 MiB 要调 `required_limits`（第 2.2 节） |
| 派发 `run_gpu_crc` | rs:341-421 | 建 params / input / lengths / results（整片先填哨兵 0xFF）/ staging 五块缓冲，一次 `dispatch_workgroups`，拷到 staging，`map_async` + `device.poll(PollType::Wait { timeout: None })` 回读 | `poll` 不设超时（rs:408）；每次派发都新建全部缓冲 |
| 覆盖断言 `assert_full_coverage_no_sentinel` | rs:425-430 | 每条结果的 `local_offset` = 数组下标、没有哨兵残留 | 可原样 |
| 多卡切片 `command_run_shard` | rs:656 起 | 一张卡一个进程：`--input 批文件 --start --count --adapter-index --output`，另可注入翻位 `--inject-flip` 与丢结果 `--inject-drop` | 跨机不在装置里：文件里 `ssh`/`scp` 零命中（`grep -c 'ssh\|scp'` 得 0），跨机拷贝与起进程由跑的人在外面做 |
| 结果文件 `write_shard_result_file` / `read_shard_result_file` | rs:450、504 | 每片一个文本结果文件（带全局单元号、CRC、计时字段） | 计时字段让每片文件两次跑不同，页面「复跑」一节只登记 merge 输出 |
| 合并与比对 `command_merge` → `compare_reference_and_shards` | rs:753、rs:538-561 | 按全局单元号 0..N 查：缺的、重复的、与 CPU 参照不同的各列一张表 | 可原样当「GPU 结果 vs CPU 参照逐格比」的骨架 |
| 测试互斥锁 `GPU_TEST_MUTEX` | rs:853（用在 rs:942、959、971） | 进程内 `static Mutex<()>` 序列化三条 GPU 单测 | 只管同一进程里的线程；几个测试进程同时抢同一张卡不在它射程里（推的，没量过） |

依赖与版本（`research/e7-index-bench/Cargo.toml`）：

- :466-468 `[features]` 里 `e163-gpu = ["dep:wgpu", "dep:pollster"]`；:478-482 `[dependencies.wgpu]` `version = "=30.0.1"`、`default-features = false`、`features = ["vulkan", "wgsl", "std"]`、`optional = true`；:484-486 `[dependencies.pollster]` `version = "=1.0.1"`、`optional = true`；:488-491 bin 挂 `required-features = ["e163-gpu"]`。
- `research/` 是独立 workspace（根 `Cargo.toml:12` `exclude = ["research"]`），锁在 `research/Cargo.lock`：`grep -c '^name = "wgpu' research/Cargo.lock` 得 6、根 `Cargo.lock` 得 0；`grep -rln wgpu crates/ | wc -l` 得 0。
- 装置代码不照 `crates/` 的 clippy 口径写：`grep -cE '\bas (u32|u64|usize|i64)\b'` 得 24 处 `as` 转换（如 rs:347 `count as u32`、rs:397），而 `crates/` 走的 `check.sh` 带 `-D clippy::cast_possible_truncation`（`.claude/singlefs-ai-sop/scripts/check.sh:75`）。搬进 `crates/` 要改成 `try_from`。

实验页 `.claude/kb/experiments/163-GPU多卡算单元校验和.md` 写的限制：

- M2（双机五卡）判「不能」，原因是显存（以下两处是那两行里的一句，整行太长没抄，整行看原文）：:31「**另一台第三张卡（RTX 5080，片 2，单元 1024–1535）两次都建不起 GPU 上下文**：`request_device` 失败，`RequestDeviceError { inner: Core(Device(OutOfMemory)) }`；当时该卡 `nvidia-smi` 显存余量仅 141 MiB（被本地服务占满，登记「不停本地模型」）。」；:38 结论「翻面条件是那张卡腾出显存」。
- 变异（:62 那一行的头一句）：「**`research/scripts/mutate.sh` 跑不了这个表**：它固定用 `cargo test --release --bin "$BIN"`，不带 `--features`……」。⚠️ 这句与 HEAD 里的 `mutate.sh` 已经对不上：HEAD 版（`git show HEAD:research/scripts/mutate.sh | grep -c required-features` 得 23）文件头 :16-19 写「bin 在 Cargo.toml 的 [[bin]] 里挂了 required-features 的：每次 cargo test 带 `--features <包名>/<feature>,…`」，引入它的提交是 `7a3c8e1e`（2026-09-27）。这张表在新 `mutate.sh` 下跑没跑过、几个 worker 进程同时占卡会不会重现页面 :84 那种挂起，没查（推的风险）。
- 它答不了的（:107-111）：显存充裕时五卡是否全成、速度与吞吐（登记不设速度门槛）、单元长度 / 派发粒度 / 批大小敏感性（只跑过 N = 2560、32768 字节）。
- 影响的决策（:119）：D24（后台重活能不能卸给 GPU） 记「不影响」，理由是 D24 射程只管三个后台重活。

## 4　E161 可行性档的分段计时与去重计数（整行抄）

产物 `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out`（171 行）。取样域与线程数（:1）：

```
E7RESULT name=config experiment=E161 mode=feasibility threads=10 registration=research/prompts/e161-preregistration.md domain=first_small_37+quick_54_84+strided_segment_heads_4096
```

去重计数（`name=g1`，`grep -n 'name=g1 '`，5 行）：

```
28:E7RESULT name=g1 cell=first_small states=37 distinct_unit_contents=18 per_state_unit_checks=644 r1=0.027950 r1_at_or_above_half=false unit_read_events=1610 states_without_unit_reads=0 header_scan_events=1380 header_scan_per_state_positions=534 header_scan_distinct_contents=15 ring_record_events=212 ring_record_per_state_positions=106 ring_record_distinct_contents=3 ring_record_r=0.028301 content_collisions=0 q1a_peak_after_cold_start=12 q1a_positive_intervals=3/10 q1a_final=18 q1a_last_four=[12,0,2,0]
39:E7RESULT name=g1 cell=first_segment_head states=4096 distinct_unit_contents=9 per_state_unit_checks=50324 r1=0.000178 r1_at_or_above_half=false unit_read_events=99476 states_without_unit_reads=0 header_scan_events=84264 header_scan_per_state_positions=42132 header_scan_distinct_contents=8 ring_record_events=32768 ring_record_per_state_positions=16384 ring_record_distinct_contents=2 ring_record_r=0.000122 content_collisions=0 q1a_peak_after_cold_start=0 q1a_positive_intervals=1/1 q1a_final=9 q1a_last_four=[9]
50:E7RESULT name=g1 cell=first_quick states=54 distinct_unit_contents=18 per_state_unit_checks=972 r1=0.018518 r1_at_or_above_half=false unit_read_events=2142 states_without_unit_reads=0 header_scan_events=1968 header_scan_per_state_positions=828 header_scan_distinct_contents=15 ring_record_events=348 ring_record_per_state_positions=174 ring_record_distinct_contents=3 ring_record_r=0.017241 content_collisions=0 q1a_peak_after_cold_start=12 q1a_positive_intervals=3/11 q1a_final=18 q1a_last_four=[0,0,2,0]
138:E7RESULT name=g1 cell=second_segment_head states=4096 distinct_unit_contents=23 per_state_unit_checks=165012 r1=0.000139 r1_at_or_above_half=false unit_read_events=443540 states_without_unit_reads=0 header_scan_events=387368 header_scan_per_state_positions=140436 header_scan_distinct_contents=20 ring_record_events=49152 ring_record_per_state_positions=24576 ring_record_distinct_contents=3 ring_record_r=0.000122 content_collisions=0 q1a_peak_after_cold_start=0 q1a_positive_intervals=1/1 q1a_final=23 q1a_last_four=[23]
149:E7RESULT name=g1 cell=second_quick states=84 distinct_unit_contents=58 per_state_unit_checks=6708 r1=0.008646 r1_at_or_above_half=false unit_read_events=19068 states_without_unit_reads=0 header_scan_events=16692 header_scan_per_state_positions=5868 header_scan_distinct_contents=52 ring_record_events=1544 ring_record_per_state_positions=772 ring_record_distinct_contents=7 ring_record_r=0.009067 content_collisions=0 q1a_peak_after_cold_start=9 q1a_positive_intervals=5/14 q1a_final=58 q1a_last_four=[0,8,0,0]
```

分段计时（`name=g3`，`grep -n 'name=g3 '`，5 行）：

```
37:E7RESULT name=g3 cell=first_small states=37 state_elapsed_ns=77898270 recovery_consult_elapsed_ns=6218674 recovery_consult_share_ratio=0.079830 recovery_ignore_elapsed_ns=4220050 recovery_ignore_share_ratio=0.054173 oracle_elapsed_ns=57120 oracle_share_ratio=0.000733 pool_checker_elapsed_ns=64510703 pool_checker_share_ratio=0.828140 record_checker_elapsed_ns=2880753 record_checker_share_ratio=0.036980 unit_check_replay_elapsed_ns=35012682 unit_check_replay_share_ratio=0.449466 unit_events_replay_elapsed_ns=111501157 unit_events_share_ratio=1.431368 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=29498021 walk_checker_elapsed_ns=69131624 walk_checker_of_checker_share_ratio=1.071630 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.108108
48:E7RESULT name=g3 cell=first_segment_head states=4096 state_elapsed_ns=5361069162 recovery_consult_elapsed_ns=493604274 recovery_consult_share_ratio=0.092071 recovery_ignore_elapsed_ns=441067394 recovery_ignore_share_ratio=0.082272 oracle_elapsed_ns=2454094 oracle_share_ratio=0.000457 pool_checker_elapsed_ns=3967962616 pool_checker_share_ratio=0.740143 record_checker_elapsed_ns=455165874 record_checker_share_ratio=0.084902 unit_check_replay_elapsed_ns=2512991021 unit_check_replay_share_ratio=0.468748 unit_events_replay_elapsed_ns=7079574037 unit_events_share_ratio=1.320552 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=1454971595 walk_checker_elapsed_ns=4302373569 walk_checker_of_checker_share_ratio=1.084277 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.000000
59:E7RESULT name=g3 cell=first_quick states=54 state_elapsed_ns=100154281 recovery_consult_elapsed_ns=7983871 recovery_consult_share_ratio=0.079715 recovery_ignore_elapsed_ns=6029949 recovery_ignore_share_ratio=0.060206 oracle_elapsed_ns=50789 oracle_share_ratio=0.000507 pool_checker_elapsed_ns=81293816 pool_checker_share_ratio=0.811685 record_checker_elapsed_ns=4782447 record_checker_share_ratio=0.047750 unit_check_replay_elapsed_ns=51945813 unit_check_replay_share_ratio=0.518657 unit_events_replay_elapsed_ns=149278874 unit_events_share_ratio=1.490489 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=29348003 walk_checker_elapsed_ns=86900991 walk_checker_of_checker_share_ratio=1.068974 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.222222
147:E7RESULT name=g3 cell=second_segment_head states=4096 state_elapsed_ns=19150712953 recovery_consult_elapsed_ns=1114083742 recovery_consult_share_ratio=0.058174 recovery_ignore_elapsed_ns=988126998 recovery_ignore_share_ratio=0.051597 oracle_elapsed_ns=39248714 oracle_share_ratio=0.002049 pool_checker_elapsed_ns=16482335779 pool_checker_share_ratio=0.860664 record_checker_elapsed_ns=525887270 record_checker_share_ratio=0.027460 unit_check_replay_elapsed_ns=8635287703 unit_check_replay_share_ratio=0.450912 unit_events_replay_elapsed_ns=28538927000 unit_events_share_ratio=1.490227 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=7847048076 walk_checker_elapsed_ns=17651600700 walk_checker_of_checker_share_ratio=1.070940 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.000000
158:E7RESULT name=g3 cell=second_quick states=84 state_elapsed_ns=753022700 recovery_consult_elapsed_ns=28987879 recovery_consult_share_ratio=0.038495 recovery_ignore_elapsed_ns=23935482 recovery_ignore_share_ratio=0.031785 oracle_elapsed_ns=821199 oracle_share_ratio=0.001090 pool_checker_elapsed_ns=686332409 pool_checker_share_ratio=0.911436 record_checker_elapsed_ns=12921261 record_checker_share_ratio=0.017159 unit_check_replay_elapsed_ns=363700795 unit_check_replay_share_ratio=0.482987 unit_events_replay_elapsed_ns=1220581625 unit_events_share_ratio=1.620909 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=322631614 walk_checker_elapsed_ns=728438333 walk_checker_of_checker_share_ratio=1.061349 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.000000
```

G5 的内容量（`name=g5_contents`，5 行）与显存阳性对照（`name=positive_control_device_memory`，2 行）：

```
36:E7RESULT name=g5_contents cell=first_small states=37 fixed_structure_bytes=72192 fixed_structure_distinct=22 journal_ring_bytes=12288 journal_ring_distinct=3 whole_unit_bytes=344064 whole_unit_distinct=18 unit_header_scan_bytes=61440 unit_header_scan_distinct=15 p1_bytes=489984 p1_peak_after_cold_start=324096 p1_positive_intervals=6/10 p1_final=489984 p1_last_four=[324096,0,16896,0]
47:E7RESULT name=g5_contents cell=first_segment_head states=4096 fixed_structure_bytes=38912 fixed_structure_distinct=13 journal_ring_bytes=8192 journal_ring_distinct=2 whole_unit_bytes=163840 whole_unit_distinct=9 unit_header_scan_bytes=32768 unit_header_scan_distinct=8 p1_bytes=243712 p1_peak_after_cold_start=0 p1_positive_intervals=1/1 p1_final=243712 p1_last_four=[243712]
49:E7RESULT name=positive_control_device_memory cell=first_segment_head p1_grew=32768 p3_grew=147 p2_grew=7 p4_per_state_grew=48 pc5_holds=true
58:E7RESULT name=g5_contents cell=first_quick states=54 fixed_structure_bytes=80384 fixed_structure_distinct=24 journal_ring_bytes=12288 journal_ring_distinct=3 whole_unit_bytes=344064 whole_unit_distinct=18 unit_header_scan_bytes=61440 unit_header_scan_distinct=15 p1_bytes=498176 p1_peak_after_cold_start=262656 p1_positive_intervals=7/11 p1_final=498176 p1_last_four=[69632,0,16896,0]
146:E7RESULT name=g5_contents cell=second_segment_head states=4096 fixed_structure_bytes=35328 fixed_structure_distinct=13 journal_ring_bytes=12288 journal_ring_distinct=3 whole_unit_bytes=425984 whole_unit_distinct=23 unit_header_scan_bytes=81920 unit_header_scan_distinct=20 p1_bytes=555520 p1_peak_after_cold_start=0 p1_positive_intervals=1/1 p1_final=555520 p1_last_four=[555520]
148:E7RESULT name=positive_control_device_memory cell=second_segment_head p1_grew=32768 p3_grew=147 p2_grew=66 p4_per_state_grew=48 pc5_holds=true
157:E7RESULT name=g5_contents cell=second_quick states=84 fixed_structure_bytes=106496 fixed_structure_distinct=33 journal_ring_bytes=28672 journal_ring_distinct=7 whole_unit_bytes=1048576 whole_unit_distinct=58 unit_header_scan_bytes=212992 unit_header_scan_distinct=52 p1_bytes=1396736 p1_peak_after_cold_start=213504 p1_positive_intervals=9/14 p1_final=1396736 p1_last_four=[0,180736,4096,0]
```

去重键（`name=reuse_arm`，K2-consult / K2-ignore / K2-pair / K4-walk / K4-units / K4-full 六臂 × 五格 = 30 行）在产物 :29-34、:40-45、:51-56、:139-144、:150-155，每行三百字以上，这里没抄，要引时整行取。判定行 :170 末尾：`g1=not_judged_feasibility_only g2=not_judged_feasibility_only g3=not_judged_feasibility_only g4=not_judged_feasibility_only g5=not_judged_feasibility_only`（这一段是 `grep -o` 取的片段，不是整行）——可行性档 G1–G5 一行都没判。

按上面各行算的每状态平均（算术，`python3` 从这几行解析字段相除；只是可行性档取样域上的平均，`threads=10`、同机有别的负载，量级参考）：

| 格 | 每状态逐单元检查 | 每状态单元读 | 每状态头扫描读 | 每状态环记录读 | 每状态总耗时 µs | 其中池级 checker µs | 记录核对器 µs | 恢复（看 journal）µs |
|---|---|---|---|---|---|---|---|---|
| first_small | 17.4 | 43.5 | 37.3 | 5.7 | 2105.4 | 1743.5 | 77.9 | 168.1 |
| first_segment_head | 12.3 | 24.3 | 20.6 | 8.0 | 1308.9 | 968.7 | 111.1 | 120.5 |
| first_quick | 18.0 | 39.7 | 36.4 | 6.4 | 1854.7 | 1505.4 | 88.6 | 147.8 |
| second_segment_head | 40.3 | 108.3 | 94.6 | 12.0 | 4675.5 | 4024.0 | 128.4 | 272.0 |
| second_quick | 79.9 | 227.0 | 198.7 | 18.4 | 8964.6 | 8170.6 | 153.8 | 345.1 |

读这两张表要知道的口径（出自跑前登记 `research/prompts/e161-preregistration.md` 第六节 G3，:295-308，Q3f 在 :304、Q3g 在 :305）：`unit_check_replay`（Q3f，s_lo）是「每个状态对它每个不同单元位置重放一次 U」，`unit_events_replay`（Q3g，s_hi）是「每一次单元读重放一次 U，另对每一次头扫描读做一次 `crc32_castagnoli_bitwise`（整段读到的字节），对每一次环内 4096 字节读做一次 `check_journal_record`」；U(字节) = `crc32_castagnoli_table` + 按类标签 `check_unit` / `index_node_view` + key 检查 / `packed_unit_view`（:299）。U 里的 `check_unit` 用的是按位 CRC（lib.rs:409），走读里 `judge_unit_header` 的载荷 CRC 用查表（walk.rs:725），所以 s_hi 大于 1（1.32–1.62）不说明单元级检查比整个 checker 还长，只说明重放的这份按位 CRC 比真走读里的那几道贵（推的，没拆开量）。

跑前登记第四节 4.8、4.9 原文（`research/prompts/e161-preregistration.md:174-175`，整行）：

```
| 4.8 | 显存预算：10¹⁰ 个状态时单卡每状态 3.419 字节、五卡每状态 10.26 字节 | 同 4.7 | **G5 的判定在跑之前已被算术大半定了**：凡按状态存下来的部分（子集掩码、每状态的键），每状态超过 10.26 字节，一次装入在五卡上也装不下 |
| 4.9 | 子集掩码按写表位图算（推的：W = 录制流写数 + 每次取三态的写一份撕裂镜像，同段没有与它重叠的更晚写、没有重放）：第一条流 W = 49、7 字节/状态，10¹⁰ 时 7×10¹⁰ 字节（66 757 MiB），单卡装不下、五卡装得下；第二条流整条 W = 523、66 字节/状态，10¹⁰ 时 6.6×10¹¹ 字节，五卡装不下；第二条流最大一段 2 415 919 103 个状态 × 66 字节 = 159.45 GB，按段分批也装不下 | 同 4.7 | 同 4.8：显式存每状态掩码的「一次装入」按第二条流的 W 已装不下；跑出来的数补的是 P1（去重后的内容）、P3（读集）的实际大小与每状态平均。问题单的两个候选都假定掩码要存；「掩码不存、由序号现算」不在候选里，交回时报给主 agent |
```

4.8、4.9 的「同 4.7」指 :173 那一行的出处（第十三节 13.2 的 `anchors.py`）；4.6（:172）是容量读数：本机 5090 total 32607 MiB、5060 Ti total 16311 MiB，五卡合计按 97843 MiB 算，另一台的读数那一行写「是主 agent 转述的读数，我没现查」。

## 5　E162 现状

第 5 问改派，不答。

## 6　依赖约束：门禁 94 号判什么、GPU 代码住哪、`Cargo.lock` 牵动哪些行

### 6.1 门禁 94 号（`.claude/gate.d/94-checker-implementation-disjoint.sh`）

| 条 | 行 | 判什么 | 对 GPU 代码意味着什么（推的，没造输入跑过） |
|---|---|---|---|
| ① | :9-14；实现 :146-171 | 按 `crates/*/Cargo.toml` 建内部依赖图，`singlefs-checker` 与 `singlefs-core` 各取传递闭包求交，减去 `singlefs-format` 必须为空；:148「外部 crate 在图上是叶子（本仓没有它的 Cargo.toml），仍然进闭包、仍然参与求交」 | wgpu 只进池级 checker 一侧：交集仍只有 `singlefs-format`，不红；core 一侧也引 wgpu（或引同一个 GPU 内部 crate）就红。wgpu 自己的传递依赖不在图上（:13-14「罩不到的」） |
| ② | :15-16 | 池级 checker 源码零处引 `singlefs_core` | GPU 代码写在池级 checker 里就归这一条管 |
| ③ | :17-19 | `singlefs-format` 正文不许有分支与循环 | GPU 代码不能放进 format |
| ④ | :20-22；实现 :259-271 | harness 档包的闭包里没有 `singlefs-checker-tier`，源码零处引 `singlefs_checker_tier` | GPU 代码放 checker 档不碰这一条；放池级 checker 时 harness 依赖池级 checker（`crates/singlefs-harness/Cargo.toml:9`），非可选依赖会让 harness 档每次编都带上 wgpu（编译代价，不是违规） |
| 射程 | :40 | 「五个 crate 的名字写死在这里」（python 里 :56-61 的 `CHECKER`、`CORE`、`SHARED`、`HARNESS`、`CHECKER_TIER`） | 新建第六个 crate 放 GPU 代码，它自己引不引 `singlefs_core` 不归 ② 管；只有被池级 checker 依赖时进 ① 的闭包 |

`.claude/rules/verification.md` 里相关的三句：:14 池级 checker 那一行「只依赖 `singlefs-format`」；:17「依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档」；:45「池级 checker 与实现的依赖闭包（dev-dependencies 也算）除 `singlefs-format` 之外不相交……它今天只依赖 `singlefs-format`、没有 `[dev-dependencies]`，这一句 94 号不单判」。即「池级 checker 只依赖 format」这一句今天没有门禁单判，94 号判的是交集。

现查的依赖（`[dependencies]` 段）：池级 checker 只有 `singlefs-format`（`crates/singlefs-checker/Cargo.toml:9`）；core 只有 `singlefs-format`（`crates/singlefs-core/Cargo.toml:9`）；harness 有 checker、core、format（`crates/singlefs-harness/Cargo.toml:9-11`）；checker 档有 checker、core、format、harness（`crates/singlefs-checker-tier/Cargo.toml:9-12`）。五个 crate 都 `#![forbid(unsafe_code)]`。

### 6.2 GPU 代码住哪（三个候选，推的，都没试）

| 候选 | 94 号 | 独立性（D13（验证路线） 已定项 5：checker 与实现只共享常量模块） | 别的代价 |
|---|---|---|---|
| 放池级 checker `crates/singlefs-checker`，wgpu 挂可选特性 | ①②④ 都不红（前提：core 不引 wgpu） | 有机器守：② 查源码不引 core | 用户原话要的是「和 core 无关」「checker 的部分」（`.claude/kb/milestone/03-third-txn.md:51`）；harness 依赖池级 checker，特性不开就不编 wgpu；工作区一起编时特性合并由 Cargo 定（推的，没核 resolver 2 在这个仓里的行为） |
| 放 checker 档 `crates/singlefs-checker-tier` | 不红（它不是 ① 比的两侧之一） | 没机器守：checker 档本来就依赖 core（Cargo.toml:10），GPU 判定代码若调了 core 的东西，94 号看不见 | checker 档默认只在提交时跑（`.claude/rules/verification.md:13`） |
| 新建第六个 crate，只依赖 `singlefs-format` + wgpu，被池级 checker 或 checker 档依赖 | 被池级 checker 依赖时进 ① 闭包；它自己不在 ② 的扫描范围 | 要给 94 号加第六个路径才有 ② 那种守 | 根 `Cargo.toml:5-11` 的 `members` 要加一行；94 号写死五个路径（:40） |

### 6.3 给 `crates/` 加 wgpu 会动根 `Cargo.lock`；`stage-inputs.tsv` 里登记了 `Cargo.lock` 的行

根 `Cargo.lock` 今天没有 wgpu（`grep -c '^name = "wgpu' Cargo.lock` 得 0）。`.claude/gate.d/stage-inputs.tsv` 里第二列有 `Cargo.lock` 的行（`awk -F'\t'` 按空格切第二列逐词比，14 行；非注释且有第二列的行共 16 行，另两行是 :30 `57-lkmm.sh` 与 :35 `E142/layer0`）：

| 行 | 键 | 根 `Cargo.lock` | `research/Cargo.lock` |
|---|---|---|---|
| 28 | 54-layer0-replay.sh | 有 | — |
| 29 | 55-qemu-device-streams.sh | 有 | — |
| 31 | 59-crates-mutation-replay.sh | 有 | — |
| 32 | 74-model-differential.sh | 有 | — |
| 33 | 87-replay.sh | 有 | — |
| 34 | E142 | 有 | 有 |
| 36 | crash-case:layer0-first-stream | 有 | — |
| 37 | crash-case:layer0-second-stream | 有 | — |
| 38 | crash-case:floor-raise-pushed-by-the-session | 有 | — |
| 39 | crash-case:c561-sigma-full | 有 | — |
| 40 | crash-case:layer0-multi-record-publish-stream | 有 | — |
| 41 | crash-case:layer0-tree-split-streams | 有 | — |
| 42 | crash-case:layer0-position-addressed-tree-streams | 有 | — |
| 43 | crash-case:crash-injection-fast-tier | 有 | — |

另 :35 `E142/layer0` 的路径列是 `@E142`，按表头 :12「路径写成 @<别的键> 就是那个键登记的全部路径」也含两份锁。`research/scripts/admission.py` 里 `grep -n Cargo.lock` 零命中：算崩溃枚举用例输入时减掉的只有 `crates/mutations.tsv` 与没人读的 `src/bin/`（admission.py:259、261），不减锁。推的后果：根 `Cargo.lock` 一变，上表 8 条崩溃枚举用例的全绿标记全部作废，下一次 54 号 `--full` 要全部重跑；54、55、59、74、87 号的复用判定也都失效。

## 7　D24 与 E21：定案、射程、管不管崩溃验证

D24（后台重活能不能卸给 GPU）（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md`）射程（:3，整行）：

> D24（后台重活能不能卸给 GPU） 管一件事：三个后台重活（批量压缩、全盘 scrub、大规模纠删码重建）要不要卸给 GPU。不管的：CPU 侧算法怎么选在 D9（加密） 已定项 3；scrub 本身的形态在 D9（加密） 已定项 5；后台整理与回收的队列与预算在 D26（后台整理与放置回收）；三格划分本身是 `.claude/rules/fs-design.md` 的记账纪律，D24（后台重活能不能卸给 GPU） 只引用它。

已定项索引（:9-13）：1 暂缓不排期、GPU 这条线不再继续测；2 三条硬约束；3 判据形式；4 重开的四个闸 G24.1–G24.4；5 合法性来自 fs-design.md 第三格。各分项定案原文在 :17（已定项 1）、:30-34（已定项 2 三条）、:47（已定项 3）、:60-71（已定项 4）、:84（已定项 5）。已定项 2 第 3 条（:34，整行）：

> 3. **它是一个新的失败域，卸出去的结果必须能被 CPU 独立复核。** GPU 算错（驱动 bug、显存 ECC 缺失、算法实现分歧）不能变成静默的数据损坏 ⇒ 与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」同构：**GPU 路径与 CPU 路径必须是两份实现**，抽样复核。

E21（GPU 卸载的净收益）（`.claude/kb/experiments/21-GPU卸载的净收益.md`）标题（:1，整行）：

> ## E21 GPU 卸载的净收益 —— 三段全部已测（2026-08-28）；纯扫描那格判负收益，候选三格**暂缓不排期**

它量的是 CPU 内存带宽 64.91 GB/s、GPU 单程 56.69 GB/s、来回 28.5 GB/s、纯扫描端到端 50.3 GB/s 净收益 0.78×（:56-66）；影响的决策表（:214-220）三行都只指 D24。

管不管崩溃验证这一侧（现查到的四处文字，各说各的）：

| 出处 | 原文（整行或指路） | 说了什么 |
|---|---|---|
| D24 :3 | 见上 | 射程逐字限定三个后台重活，崩溃验证不在里面 |
| `.claude/rules/verification.md:34` | checker 档全量那一行里有一句「GPU 只接校验和那一截，要不要接归 D24（后台重活能不能卸给 GPU）」 | **把崩溃验证的 GPU 那一截归到 D24**，与 D24 自己的射程对不上 |
| E163 页 :119、E161 页（`161-崩溃放量的去重与分段耗时.md`）:94 | E163 页「D24（后台重活能不能卸给 GPU） 管的是批量压缩、scrub、纠删码重建三个后台重活（已定项 3 的射程逐字限定）……与那三个候选场景不是同一件事（与 E161（崩溃放量的去重与分段耗时） 对 D24（后台重活能不能卸给 GPU） 的回看同一口径）」（:119 那一行的一段） | 实验页按「不影响」记 |
| `.claude/kb/milestone/03-third-txn.md:51` | 用户原话「这是崩溃放量的实现 是checker的部分 和core无关」「这个肯定是要实现的」（那一行里的两句） | 里程碑三把 GPU checker 定为必做 |

推的：D24 已定项 1「GPU 这条线不再继续测」与已定项 2 的三条硬约束，按 D24 自己的射程不罩崩溃验证；而 verification.md:34 那句把它指回 D24。两处要不要对齐、由谁定，是这一轮三方或用户的事；这里只报对不上。D24 已定项 2 第 3 条「GPU 路径与 CPU 路径必须是两份实现，抽样复核」与里程碑三第四项「GPU 上的 checker 与 CPU 上的 checker 怎么对拍（抽样逐格相同，不同就判红）」（03-third-txn.md:53）内容同向，但前者按射程不管后者。

## 8　几条要带走的事实，与什么会推翻它们

| 事实 | 依据 | 什么现象会推翻 |
|---|---|---|
| 每状态耗时里池级 checker 占大头：可行性档五格 `pool_checker_share_ratio` 0.740–0.911，记录核对器 0.017–0.085，两遍恢复合计 0.07–0.17 | 第 4 节 g3 五行 | 全域按登记 6.0 加权跑完之后占比落到 0.5 以下 |
| 去重之后单元内容极少：五格 `distinct_unit_contents` 9–58，`r1` 0.000139–0.027950 | 第 4 节 g1 五行 | 更大的取样域上 `q1a_final` 随状态数线性涨 |
| (a) 类在今天的走读里是「按指针边走边判」，单元字节经 `&dyn ImageReader` 按需读（image.rs:19-28），每次读经 `CrashImage` 按写表叠一遍（memory_pool.rs:445-469） | 第 1.0、2.2 节 | 找到一条不经 `CrashImage::read` 的读路径 |
| 同一个码 2 节点整单元 CRC 算三遍，其中一遍按位（`check_unit` 里 lib.rs:409） | 第 1.2 节代码路径 | 计时拆分显示 `index_node_view` 不在走读热路径上 |
| 根 `Cargo.lock` 一变，8 条崩溃枚举用例的全绿标记全部失效 | 第 6.3 节 | admission.py 实际会把锁从指纹里减掉（今天 `grep -n Cargo.lock` 零命中） |
| E163 页说 `mutate.sh` 跑不了带 `required-features` 的表，与 HEAD 的 `mutate.sh` 对不上 | 第 3 节 | HEAD 的 `mutate.sh` 实际仍不带 `--features`（它文件头与代码不一致） |
| verification.md:34 把崩溃验证的 GPU 那一截归 D24，D24 :3 射程不含它 | 第 7 节 | D24 另有一处分项把崩溃验证写进射程（全文 97 行已通读，没找到） |

## 没做什么

- 没编译、没跑任何测试或装置（派发写明「重型测试：不跑」，另一会话的层 0 全量占着两台机器）；上文所有「推的」都没量过。
- 第 5 问（E162 现状）按主 agent 改派不答。
- 没读 `walk.rs` 里每个走读子函数的全文（inode 树、extent、分配记录树、实例表链），它们的归类按函数名、判的不变量与调用关系定；「一个单元读几遍」只数到 `read_index_node` 那条路径，别的子走读没逐个数。
- 没核 WGSL 规范里越界访问的语义、没核 Cargo resolver 2 在这个工作区里的特性合并行为。
- 没核 E163 的变异表在 HEAD 的 `mutate.sh` 下跑不跑得起来。
- 没现查两台机器此刻的显存（E161 登记 4.6 的读数是 2026-09-27 的）。
- 草稿目录 `/tmp/claude-1000/m3-facts-checker-gpu/` 只放了比对用的几份文本（`quoted.txt`、`orig.txt`、`quoted_sorted.txt`、`q2.txt`、`o2.txt`），没建编译目录与仓副本。
````

**出处 `research/prompts/m3-prune-gpu-r1-facts-kv.md:1-279`（整段抄，未转述）**

````markdown
# 里程碑三第十一项事实调查：崩溃放量的过程与结果怎么存（KV 存储）

2026-09-28。调度记录 `records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 任务丁。只读代码、kb 与产物；没编译、没跑任何测试与实验、没改仓里别的文件。行号都是现取的（`grep -n` / `awk 'NR==…'`）；引产物整行抄；没量过的写「推的」。

## 一、E162（崩溃放量判定块存储选型）全貌

### 1.1 文件在哪

| 物 | 路径 | 现状 |
|---|---|---|
| 实验页 | `.claude/kb/experiments/162-崩溃放量判定块存储选型.md`（203 行） | 标题 :1「## E162 崩溃放量判定块存储选型 —— S1 够判档已交，S2 / S3 还没跑（2026-09-27）」；S3 回环半份、S4 装置都还没写进页 |
| 跑前登记 | `research/prompts/e162-preregistration.md`（1296 行） | 第一至十一节（S1–S3 原登记）；第十二节修订一至六（:426–:431）、补登 S3 跨机与 S4 掉电（:433 起，补 1–补 13）、补 14 主 agent 认定（:1038）、S4-修订一至十（:1053）、S3-修订一至十（:1070） |
| 问题单 | `research/prompts/m2-crash-store-r1-forks.md`（12 行） | S1–S4 在 :9–:12 |
| 报告 | `research/prompts/e162-s1-decisive-runner-report.md`、`e162-s3-s4-designer-report.md`、`e162-s4-apparatus-runner-report.md`（证据目录 `e162-s4-apparatus-evidence/`） | S3 回环执行员没有入库报告（`ls research/prompts \| grep -i -E 's3-loop\|e162'` 只列出这四样加登记） |
| 装置 | `research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs`（S1、S2）、`e162_verdict_store_sender.rs`、`e162_verdict_store_network.rs`（S3）、`e162_verdict_store_power_cut.rs`（S4） | `research/e7-index-bench/Cargo.toml:471`、`:495`、`:501`、`:507` 四个 `[[bin]]`；除 sender 外都 `required-features = ["e162-block-stores"]` |
| 版本 | `research/e7-index-bench/Cargo.toml:457–:463` `[dependencies.redb] version = "=4.3.0"`、`[dependencies.rocksdb] version = "=0.25.0"`，都 `optional = true`；`:466` `e162-block-stores = ["dep:redb", "dep:rocksdb"]` | `research/Cargo.lock:885–:886` redb 4.3.0、`:938–:939` rocksdb 0.25.0、`:544–:545` librocksdb-sys `0.19.0+11.8.1` |

构建注意（`research/e7-index-bench/Cargo.toml:453–:455` 注释）：`librocksdb-sys` 现编 C++，本机 bindgen 找不到 `stdbool.h`，要 `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` 才编得过；为什么以前不设也编过没查（实验页 :187）。

### 1.2 四条臂

实验页 :7 整行：

> 四条被测臂：F（加固的文件，临时文件 + fsync + 改名 + 目录 fsync）、R1（redb，`set_quick_repair(true)`）、R0（redb，默认修复）、K（RocksDB，同步写 + WAL）。

完整定义在登记 :149（R1）、:150（R0）、:151（K）、:152（F）：R1 每块一个写事务、`Durability::Immediate`、`set_quick_repair(true)`；R0 同 R1 不开 quick-repair；K `put_opt` 带 `set_sync(true)`、WAL 开、Lz4 压缩，打不开时调一次 `DB::repair`；F 一块一个文件（48 字节头含块键与 CRC-32C）、临时文件 → `sync_all` → 改名 → 父目录 `sync_all`。块键 32 字节 =（节点 u32 大端，段号 u32 大端，序号区间起点 u64 大端，输入指纹 16 字节），块值 65 536 字节、每状态 1 字节判定编号（登记「1.1 读法写死」「块」那一行）。没建的形态：RocksDB BlobDB（登记 :154）。

登记第四节 4.10：F 在 S1 的 Q1a–Q1f 上「由它的定义加内核语义推得出为 0」，对 F 不算判据；F 真正的风险（掉电）SIGKILL 量不到。

### 1.3 各段现状

| 行 | 跑没跑 | 够没够判 | 出处 |
|---|---|---|---|
| S1 主格（每臂杀 200 次） | 跑了（可行性档 20 次、够判档 200 次，2026-09-27） | **够判：四臂都没翻** | 页 :105–:151；产物 `…-s1-{F,R1,R0,K}-decisive.out` |
| S1-large（10⁵ 块大库上杀 20 次） | 没跑 | — | 页 :180、:182 |
| S2 主格（10⁵ 块） | 只跑可行性档 10⁴ 块 | 未判；Q2a 标 `pending_e161` | 页 :50–:68、:180 |
| 几何敏感性 G-bs10、G-batch16、G-sparse、G-random | 没跑 | — | 页 :182；登记 8.2（:329 起） |
| S3 回环 | 只跑了 R1 的一部分（2026-09-28），执行员按用户定停下；按调度记录这半份作废 | 未判 | 产物 `research/results/e162-verdict-store-network-2026-09-28-s3-loopback-R1.out`（168 行，没有 `name=done`，没有 S3-T 的 `name=verdict`） |
| S3 跨机 | 没写装置、没跑（S3-修订九：network bin 只有回环角色） | — | 登记 :1070 起 S3-修订九那一行 |
| S4 掉电 | 装置、单测 23 个、变异 11 条、干跑写完；没起 QEMU | 未判 | `research/prompts/e162-s4-apparatus-runner-report.md` 第 4–8 行、第 70 行 |

**S1 够判档判决行**（`grep -n 'name=verdict' research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`，整行）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s1-F-decisive.out:258:E7RESULT name=verdict part=s1 arm=F tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=true commit_waits_for_device=true calibration_interfered=true v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R1-decisive.out:235:E7RESULT name=verdict part=s1 arm=R1 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=179 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-R0-decisive.out:233:E7RESULT name=verdict part=s1 arm=R0 tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=178 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
research/results/e162-crash-verdict-block-store-2026-09-27-s1-K-decisive.out:234:E7RESULT name=verdict part=s1 arm=K tier=registered judgement=judged_by_registration kill_cycles=200 kills_inside_commit=180 pc_l_ok=true pc_s_ok=true pc_p_ok=true pc_o_ok=true pc_t_ok=not_applicable commit_waits_for_device=true calibration_interfered=false v6_kills_inside_commit_ok=true planned_confirmations=6178 a_s1_ok=true q1a=0 q1b=0 q1c=0 q1d=0 q1e=0 q1f=0 repaired_openings=0 s1_positive_quantities=none
```

S1 的结论句，实验页 :151 原句（节选到句号为止的那一句整句）：「**S1 够判：redb（quick-repair 与默认修复两种形态）、RocksDB（同步写 + WAL）、加固的文件，在这次 SIGKILL 模型下四条臂都没有丢已确认的块、没有静默坏块、没有幽灵块、没有打不开**。」页上同一段接着写「这里写成『四个候选在这个 SIGKILL 模型下都没有翻』，不写成『都一样安全』」。

S1 重开用时（Q1h，轨迹，不判翻面）四臂的汇总行在页 :130–:133；中位 F 1100 µs、R1 1015 µs、R0 124 691 µs、K 11 411 µs，峰值 K 3 271 087 µs（页 :138 说是撞上同期别的会话 `cargo test` 的极端值）。

**S2 可行性档判决行**（10⁴ 块，页 :53–:56 整行）：

```
research/results/e162-crash-verdict-block-store-2026-09-27-s2-F-feasibility.out:41:E7RESULT name=verdict part=s2 arm=F tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33530296.2 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R1-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R1 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=33593904.5 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-R0-feasibility.out:41:E7RESULT name=verdict part=s2 arm=R0 tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=19339984.1 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
research/results/e162-crash-verdict-block-store-2026-09-27-s2-K-feasibility.out:41:E7RESULT name=verdict part=s2 arm=K tier=feasibility judgement=not_judged_feasibility_tier_only blocks=10000 pc_rate_ok=true pc_read_ok=true pc_space_ok=true s2_main_write_states_per_second=48638773.4 q2a_verdict=pending_e161 s1_large=not_run_feasibility_tier
```

S2 可行性档的时延与占用（页 :61–:66 的表，页 :68 写明「单次裸读数，未重复跑、未判定，不当最终结论」）：

| 臂 | 持续写入（状态/秒） | 提交时延中位/p99/最大（µs） | 冷读中位/p99（µs） | 热读中位/p99（µs） | `peak_ratio_high` | `after_close_ratio` |
|---|---|---|---|---|---|---|
| F | 33 530 296.2 | 1529 / 5036 / 583066 | 321 / 399 | 123 / 144 | 1.0639 | 1.0625 |
| R1 | 33 593 904.5 | 1550 / 4352 / 115139 | 230 / 352 | 5 / 8 | 2.0029 | 2.0021 |
| R0 | 19 339 984.1 | 903 / 8299 / 2558476 | 229 / 366 | 5 / 11 | 2.0039 | 2.0017 |
| K | 48 638 773.4 | 804 / 1293 / 1733112 | 157 / 367 | 12 / 21 | 2.1495 | 1.0013 |

**S3 回环 R1 那半份**（`research/results/e162-verdict-store-network-2026-09-28-s3-loopback-R1.out`；已进暂存区 `git status --short` 报 `A`）：三个阳性对照的判决行整行：

```
30:E7RESULT name=verdict part=pc3_drop arm=R1 lost_confirmed=50 library_controlled_drops=50 positive_control_passed=true
55:E7RESULT name=verdict part=pc3_rate arm=R1 fast_blocks_per_second=579.634 slow_blocks_per_second=154.459 positive_control_passed=true timing_undisturbed=true
162:E7RESULT name=verdict part=pc3_torn arm=R1 interruptions=50 escalated_to_200=false silently_corrupted=28 positive_control_passed=true
```

S3-T 只写出了 sender / library / verification / rates 四行，没有判决行；rates 行开头（第 168 行，截到 `q3k_windows` 之前）：`E7RESULT name=rates arm=R1 cell=S3-T attempt=1 q3a_blocks_per_second=274.933 q3a_states_per_second=18017998.3 q3b_blocks_per_second=565.217 q3b_states_per_second=37042085.9 q3c_states_per_second=55060084.2`。**这半份按调度记录作废**（下面 1.5），数只当量级参考，不能引作结论。同批 anchors 产物 `research/results/e162-verdict-store-network-2026-09-28-anchors.out:21` 整行 `E7RESULT name=verdict part=anchors anchors_checked=20 anchor_mismatches=0`。

### 1.4 哪几格还等 E161（崩溃放量的去重与分段耗时） 的正式每状态耗时

| 格 | 门槛怎么用 t_state | 出处 |
|---|---|---|
| S2 Q2a 持续写入速率 | R_AB = (P_A + P_B) ÷ t_state；四臂判决行都是 `q2a_verdict=pending_e161` | 登记 6.0（:225 起）、:256；页 :80、:183 |
| S2 Q2b 分窗速率、Q2c 缓冲 | 都按 R_AB | 登记 6.2 表 Q2b、Q2c 两行；修订四（:429）让 Q2c 在 t_state 没到时按实测 Q2a 的 50/75/90/100% 报 |
| S2 Q2d 冷读 p99 | 门槛 = 每块状态数 × t_state（设计员加的，登记写明要主 agent 认或删） | 登记 6.2 表 Q2d 那一行 |
| 几何敏感性「门槛本身」 | t_state 与 t_state_1 各判一次 | 登记 8.2（:329 起）第二行 |
| S3 Q3a（回环与跨机） | R_B = P_B ÷ t_state | 登记 1.1「那一台 CPU 的判定产出速率（S3）」、8.2 末行 |

E161 今天只有可行性档：页 `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:1` 标题「可行性档已跑（2026-09-27）」，:81「**G1–G5 每一行都未判**…第八节几何敏感性…G3 的加权估计与 G5 的外推行这一趟不跑」。E162 页 :183 引过 E161 一格粗数：`research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out:37` 的 `states=37 state_elapsed_ns=77898270`，约 2.11 ms/状态，页上写明「只当量级参考，不代入本页任何判定」。另一种不依赖 E161 的速率（门禁 54 号实跑的整流挂钟）在第五节第 4 行。

### 1.5 第三段为什么停

`records/2026-09-24-里程碑二收尾调度.md:318` 整行：

> | 每 65 分钟查一次巡视看门狗；E162 第三段现在停（2026-09-28） | 用户：「再设置一个 每1小时零五分的定时循环任务 用来检查 1小时的定时循环任务是否在进行」——主 agent 起后台等待 65 分钟、查本会话的看门狗进程在不在再退出（退出即叫醒），每次醒来核完再起下一轮；cron 排不出每 65 分钟，不用。E162 第三段执行员报：跑前登记要求计时格开跑前等本机没有 cargo / rustc / qemu（最多 20 分钟、受干扰重跑最多三次），层 0 全量一跑计时读数多半作废（机器空闲时四条臂约 3.5–4 小时，撞车时约 35–40 小时，都是推的）；弹窗三选一，用户答「现在停，层 0 跑完再从头重跑」——执行员停掉产物任务、交回；已写出的 anchors 与 R1 半份原样留着、写明作废；层 0 跑完另派执行员从头重跑四条臂，产物另起 `-r2` | 层 0 跑完 → E162 重跑 |

「写明作废」这句在仓里只见于这一行：产物文件里没有作废标记（`grep -c 'name=verdict part=s3'` 那份产物为 0、`name=done` 为 0），实验页也还没提 S3。

之前还有一处与 S2、S3、S4 排期有关的用户定案，`records/2026-09-24-里程碑二收尾调度.md:311` 那一行里：「② KV 与 GPU 弹窗答「先跑第五步，KV 接入放提交之后（推荐）」——全量跑期间并行做 E163 GPU 重跑与 E162 的 S3 跨机、S4 断电、几何敏感性，选型定了之后 KV 接入作为里程碑二提交后的剩余任务、接完再跑一趟全量」（整行在第三节）。

### 1.6 装置住哪、为什么不进 `crates/`

- 登记 1.3（:42）：「装置走 `research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs`（独立手写模型，不与 `crates/` 共用代码，`.claude/rules/implementation-first.md` 第 4 条）」；:44「**不碰 `crates/` 下任何文件，不给 `crates/` 加依赖。**」
- `research/e7-index-bench` 是独立 Cargo 工作区：根 `Cargo.toml:2`「research/ 是实验 workspace，自己有 Cargo.toml，这里显式排除。」、`:12` `exclude = ["research"]`；`research/Cargo.toml:1–:3` `[workspace]`、`members = ["e7-index-bench"]`；锁文件各一份（`Cargo.lock` 与 `research/Cargo.lock`）。
- 为什么放那里：`records/2026-09-24-里程碑二收尾调度.md:325` 末句「实验放 `research/e7-index-bench`（独立的 Cargo 工作区），不给 `crates/` 加依赖——加依赖会改 `Cargo.lock`，而 `.claude/gate.d/stage-inputs.tsv` 里每条崩溃枚举用例都登记了它，全部复用标记会失效；接进 harness 等选型定了、随里程碑二提交一起跑全量。」（整行在第三节）
- 登记第三节（:90 起）引的 `crates/singlefs-harness/src/crash.rs`、`crates/singlefs-harness/src/layer0_progress.rs` 路径已过时：两档拆分（提交 `973e1f58`）之后这两份在 `crates/singlefs-checker-tier/src/`。登记是冻结证据，不改；引它时换成新路径（第二节的行号都是新路径现取的）。

## 二、今天崩溃枚举落不落盘

### 2.1 每个状态算完留下什么

| 面 | 在哪 | 事实 |
|---|---|---|
| 一个状态怎么评 | `crates/singlefs-checker-tier/src/crash.rs:1131` `evaluate_state_recording_findings`（`:1059` 的 `evaluate_state_for_versions` 转给它，`position = None`） | 用 `persisted: Vec<bool>` 造 `CrashImage`，跑两遍恢复（`JournalPolicy::Consult` / `Ignore`，:1147–:1148），再判 oracle、池级 checker、记录核对器；结果只做 `tally.<计数> += 1` 与记第一处违例原文 |
| 累计进什么 | `crash.rs:743` `pub struct Layer0Tally` | 字段：`states`、`states_by_publish`、`violations`、`root_persisted_states`、`no_file_states`、`file_read_states`、`failed_states`、`journal_differing_states`、`verification_ran_states`、`verification_failed_states`、`first_violation`、`ignored_violations`、`first_ignored_violation`、`record_root_without_record`、`record_claimed_state_missing_unit`、`checker_evaluated_states` / `checker_violated_states` / `checker_first_violation` / `checker_not_applicable_states`（按不变量的 BTreeMap）、`observed_states`、`observer_counts`、`findings`（:745–:779） |
| 判红的状态留什么 | `crash.rs:590` `Layer0Findings`、`:549` `Layer0FindingSignature`、`:567` `Layer0Finding`、`:557` `LAYER0_FINDING_SAMPLES_KEPT: usize = 3` | 签名 =（判红的那一遍 `Layer0RedPass`（:473，四种：看 journal 的 oracle / 不看 journal 的 oracle / 池级 checker 带违了哪几条 / 记录核对器），段号或 `all_persisted`，发布类）；每签名记状态数与最先 3 个样本（状态序号 + 违例原文）；另记 `red_states` |
| 判绿的状态留什么 | 同上 | 什么都不留，只进计数。没有「每个状态的判定」这一份东西，全仓没有按状态存的判定（`grep -rln '判定向量\|判定编号'` 只命中里程碑三 03 号文件、E162 登记与问题单、E162 装置、一份无关的 runlog） |
| 状态怎么编号 | `crash.rs:1816` `Layer0StatePlan`（文档注释 :1813–:1815）、`:1917` `persisted_writes_of_state` | 一条流一个**全局序号**：各段的状态区间首尾相接、随段号递增，最后加「全部持久」那一个；段内序号 = 序号 − 该段起点，按混合进制拆到段里每次写（原地覆写三态、其余两态）。持久集合由序号现算，不存 |

### 2.2 断点续跑的进度文件（`crates/singlefs-checker-tier/src/layer0_progress.rs`）

| 项 | 事实 | 出处 |
|---|---|---|
| 开不开 | 设了环境变量 `SINGLEFS_LAYER0_PROGRESS_DIRECTORY` 才留；设了就要同时设 `SINGLEFS_LAYER0_INPUT_FINGERPRINT` | :44–:47 |
| 文件名 | `layer0-progress-<流名>-<输入指纹>-<计划哈希>[-shard-<i>-of-<n>].txt` | :464–:477 |
| 文件头 | `layer0_progress_file format=2 input_fingerprint=… stream=… plan=… states=… slices=… states_per_slice=… has_observer=…[ shard=i/n]`，行尾 ` checksum=<CRC-32C 8 位十六进制>` | :58、:480–:500 |
| 单位 | **片**：一行一片，片行 25 个字段（`slice`、`first`、`end`、`states` … `findings`），就是那一片的 `Layer0Tally` 全部计数，不按状态 | :502–:528、:567 `slice_line` |
| 片多大 | 续跑、分片、merge 时片长只看状态数：`ceil(状态数 / 65 536)`，至少 16（`LAYER0_RESUMABLE_SLICE_COUNT = 65_536`、`LAYER0_MINIMUM_STATES_PER_SLICE = 16`）；不续跑时按线程数切（每线程 16 片、至少 64 片） | `crash.rs:2029`、`:2032–:2039`、`:1293–:1297`、`:2003` |
| 怎么写 | 打开时把核过的片重写成新文件：建 `.rewriting` → `write_all` → `sync_all` → `rename` → 目录 `sync_all`；之后每跑完一片追加一行、`sync_data` | :1139–:1159、:1186–:1197 |
| 读回 | 每行自带 CRC-32C；末尾没换行的半行丢掉；整行坏、字段缺多、解不开，整份作废；同一片出现两次计数相同去重、不同整份作废 | 模块文档 :1–:17 |
| 跑完 | 默认删掉（`Layer0ProgressFileAfterCompletion::Deleted`：「进度文件只在『没跑完』时存在」） | :102–:109、:1204–:1215 |
| 54 号放哪 | `<git common-dir>/singlefs-layer0-progress/<这条用例的输入指纹>` | `.claude/gate.d/54-layer0-replay.sh` 文件头「断点续跑」那几行 |

具体的数（按片长公式现算，算术）：并行线一那条流 12 230 590 578 个状态（`.claude/gate.d/stage-inputs.tsv:40` 注释）→ 每片 186 625 个状态；`crash.rs:2028` 注释「第二条流全量五十多亿个状态时每片约八万五千个（32 线程每秒约两万多个时一片一分钟上下，推的）」。

### 2.3 发现日志（判红那一份，用户 2026-09-27 定「全量和错误双份」）

- 环境变量 `SINGLEFS_LAYER0_FINDINGS_FILE`（`layer0_progress.rs:55`）；格式号 `FINDINGS_LOG_FORMAT = 1`（:1682）。
- 每趟枚举追加一节：`layer0_findings_begin`（format、stream、states，分片时加 shard、shard_states，:1923–:1940）；跑的过程中新签名一行、跨 10/100/1000… 台阶一行，每行 `sync_data`（:1795–:1800）；跑完把这一节截回起点重写成定稿：每签名一行 `layer0_finding`（号、签名字段、states、sample_states）加一行 `layer0_findings_summary`（:2005–:2030、:1808–:1817）。没有 summary 行 = 没跑完。
- 54 号读定稿：没 summary 判红、`red_states > 0` 判红、定稿行数与 `signatures=` 对不上判红；打 ≤ 30 行发现表（54 号文件头「发现日志」那一段）。

### 2.4 双机分片与合并（`research/scripts/layer0-shard-run.sh`，511 行）

| 步 | 事实 | 出处 |
|---|---|---|
| 怎么分 | 第 i 片跑切片序号 `slice_index % n == i` 的那些片（交错分） | `layer0_progress.rs:184–:189` `shard_owning_slice`；:121–:122 |
| 各自写什么 | 每片一份**账本** `layer0-shard-<流名>-shard-<i>-of-<n>.tally`：文件头（format=2、输入指纹、流名、计划哈希、状态数、片数、片长、shards、rustc、cargo、target）、这一片的线程行、按切片序号的片行（与进度文件同一种）、末行；临时文件 `.writing` → `sync_all` → 改名 → 目录 `sync_all` | :1241–:1256、:1274–:1300、:1352、:1402–:1421 |
| 进度 | 两片各自有进度文件（文件名带 `shard-<i>-of-<n>`），本机在 `<git common-dir>/singlefs-layer0-progress/<指纹>`，第二台在 `<PEER_REPOSITORY_DIRECTORY>/progress/<指纹>`；驱动收尾不删进度目录 | 驱动 :14–:30 |
| 合并 | ⑤ 第二台的账本 rsync 回本机进度目录（只拷 `layer0-shard-*-shard-1-of-2.tally`）；⑥ 本机 `SINGLEFS_LAYER0_SHARD=merge/2` 再跑同一条用例：不枚举，读 n 份账本、核文件头逐字段相同、核切片 0..S 每个恰好一次，按切片序号交回计数，再走用例钉死的计数断言 | 驱动 :463–:471、:473–:482；`layer0_progress.rs:1427–:1436` `Layer0MergedShardLedgers`、:1600–:1612 `read_shard_ledgers_for_merge` |
| 发现日志 | 三趟各一份（`<日志>.findings.tsv`）：第二台那一片的拷回本机；merge 那一趟重写出整流的发现表，54 号只读 merge 那一份 | 驱动 :48–:52、:352–:356、:448–:453、:476–:486 |
| 这种形态叫什么 | 「两台各跑一片、各写一份账本，本机 merge：正是用户 2026-09-27 否掉的『各写各的再合并』那一形」 | E162 登记第三节（:90 起）「两台机器今天怎么分工」那一行 |

### 2.5 门禁 54 号的复用判定

| 项 | 事实 | 出处 |
|---|---|---|
| 单位 | **一条崩溃枚举用例 × 这条用例这批输入的指纹**；一条用例整条流一个标记，不按片、不按段 | `.claude/gate.d/54-layer0-replay.sh` 文件头 `--full` 那一段；`research/scripts/admission.py:1757–:1762` |
| 指纹怎么算 | `admission.py crash-case-manifest`：登记路径（`crates/ Cargo.toml Cargo.lock`）下的文件减去这条用例读不到的（别的测试目标独占的测试文件、`crates/mutations.tsv`、没有代码读 `CARGO_BIN_EXE_` 时的 `src/bin/`），加准入模块里崩溃枚举用例的判法摘要、工具链、构建环境、这条用例的登记行；登记了 `shard=across-machines` 的再按内容加驱动与配置判法两份（`SHARD_DRIVER_FILES`，:220） | `admission.py` 文件头 :82–:85；`.claude/gate.d/stage-inputs.tsv:14–:20` 注释 |
| 标记写在哪 | git common-dir 里 `singlefs-crash-case-green.<用例名>.<输入指纹>`，不进工作树，各 worktree 共用 | `admission.py:225`、:1757–:1762 |
| 标记里有什么、何时作数 | `input_hash` 等于这一次指纹、`case` 对、`test_result` 是「1 passed; 0 failed」、登记的每个计数行恰好一行、要 `exhaustive=true` 的带着；没有发现表 | `admission.py:1931–:1947` `crash_case_marker_problems`；54 号文件头「全绿标记不带发现表」 |
| 判红 | 删这批输入那一格；「跑的过程中输入变了」判红不删 | 54 号文件头 |
| 门禁阶段的标记（55、57、59 号） | `singlefs-stage-green.<阶段文件名>.<指纹>`，过 `SINGLEFS_REUSE_HOURS`（默认 24 小时）不作数；`stage-must-run.sh` 先看它、再比 `refs/sop/staged-green` 那棵树与这一次暂存树在登记路径上变没变 | `research/scripts/stage-must-run.sh:16–:19`；`admission.py:2222–:2250` |

小结（从上面几张表读出来的，不是新的量）：今天落盘的三样——进度文件（按片的计数，跑完就删）、分片账本（按片的计数）、发现日志（只有判红的签名与每签名 3 个样本）——都不带「每个状态的判定」；门禁 54 号的复用粒度是整条流。

## 三、用户对日志与存储提过的要求（原文整行）

`records/2026-09-24-里程碑二收尾调度.md:245`（层 0 放量的发现日志）：

> | 层 0 放量的发现日志（用户 2026-09-27） | 用户原话「我觉得还有个问题 我们到时候的 崩溃放量日志 只写出错日志或者需要全量和错误双份日志 不然你读不过来了」。主 agent 现查今天的样子：`Layer0Tally`（`crash.rs:1171`）只记各类计数与每类一条 `first_violation`；54 号 `run_crash_case`（第 225–236 行）整段 cargo 输出 `tee` 进一份日志、只转 `LAYER0_PROGRESS` 行；红了只在末尾看到一条，分不出红在哪几段哪几类，中途读不到。定：**两份日志**——全量日志照旧，另加发现日志（按签名去重：不变量集合 + 段号 + 发布类 + 判红的那一遍，每签名记状态数与最先 3 个状态序号，新签名或跨 1/10/100… 台阶时追加并 fsync，跑完写汇总行；标准输出每个新签名打一行 `LAYER0_FINDING`）；断点续跑与分片 merge 合发现表；54 号读汇总与发现日志、把发现表（≤ 30 行）打进阶段输出，全量日志只给路径；双机驱动 merge 两片的发现日志。派实现员改 crates 那一半（规格 `/tmp/claude-1000/impl-layer0-findings/spec.md`，先交环境变量名与行格式），工具实现员改 54 号与驱动那一半等它交第一节后派；三方进代码轮第二轮与门禁批第三轮。 |

（这一行里的 `crash.rs:1171` 是当时 `crates/singlefs-harness/src/crash.rs` 的行号；今天 `Layer0Tally` 在 `crates/singlefs-checker-tier/src/crash.rs:743`。落地形态见第二节 2.3，交回记录在同一份调度记录 :250、:255。）

`records/2026-09-24-里程碑二收尾调度.md:325`（崩溃放量的判定存储进里程碑二）：

> | 崩溃放量的判定存储进里程碑二（2026-09-27，用户定） | 用户定：「这种规模文件的读写和持久化放在文件中 不靠谱。需要解决」「KV 存储这个功能也放入里程碑 2 做」。主 agent 定形态（推的）：嵌入式事务 KV，按块存（一块 2¹⁶ 个状态），不用向量数据库（查找全是精确匹配）；选哪个库先做计数实验（问题单 `research/prompts/m2-crash-store-r1-forks.md`：redb / RocksDB / 加固的文件三臂，杀进程重开核丢块、持续写入速率、双机合并）。实验放 `research/e7-index-bench`（独立的 Cargo 工作区），不给 `crates/` 加依赖——加依赖会改 `Cargo.lock`，而 `.claude/gate.d/stage-inputs.tsv` 里每条崩溃枚举用例都登记了它，全部复用标记会失效；接进 harness 等选型定了、随里程碑二提交一起跑全量。 |

`records/2026-09-24-里程碑二收尾调度.md:311`（分片可用性真测一次；KV 接入放提交之后）：

> | 分片可用性真测一次、红在第二台算不出指纹；用户定 KV 接入放提交之后、两台清场扩大、提交时测试次序（2026-09-28） | 用户：「没有越权 分片你要测试一下 测试可用性 然后告诉我大约预计的时间」。驱动自检 13 格绿（假 cargo、第二台是本机目录）；主 agent 用第三轮快照那棵树带 `SINGLEFS_HEAVY_TESTS=user-request` 真跑 `crash-case:layer0-first-stream`：两台工具链相同、各 32 核，树拷过去之后第二台算不出输入指纹（规范副本被 `.gitignore` 第 6 行忽略、不随树走；本机与自检都能回退到主仓，所以自检没抓到），失败在清场之前，本机本地模型服务一直 active；派 tooling-writer 修（`records/2026-09-16-subagent拆分提案.md` 第 54 行）。全量时间先报推的：三条大流合计 30.4 亿状态，按 2026-09-25 量的 13,880 个/秒（32 线程、第二条流，C577 与 49 条不变量之前）单机约 61 小时、双机约 30 小时，修好后真跑量两台速率再换成量的数。用户接着定：①「32核 至少可以跑31核」——本机那一片 `SINGLEFS_LAYER0_THREADS=31`；② KV 与 GPU 弹窗答「先跑第五步，KV 接入放提交之后（推荐）」——全量跑期间并行做 E163 GPU 重跑与 E162 的 S3 跨机、S4 断电、几何敏感性，选型定了之后 KV 接入作为里程碑二提交后的剩余任务、接完再跑一趟全量；③「两台都停 包括tts」「配置写在本地 不要公开」「要明确指定cot服务 防止错误清理systemctl 导致系统问题」——主 agent 改私有配置（仓根 `multi-host.env` 与它指向的三份私有清场脚本，单元逐个写死、两张白名单、带通配字符就拒，只停开跑前 active 的、复原只起开跑前 active 的；清场单元里原来经别的仓的脚本间接停的那几个也改成直接写死），仓里不写单元名与主机；不是服务单元的 java / node 开发服务器开跑时按进程号逐个停、起法记私有文件交用户；④「提交时候 先跑 harness 然后跑 checker 最后跑checker-tier」——第五步派崩溃验证员照这个次序。另：singlefs-8b 提交 `e5253e8a` 之后主 agent 把第一段打进主工作区（`crates/mutations.tsv` 1400 行、33 号绿，`crates/` `litmus/` 与第三轮快照逐字相同，差的只是第一段的变异表行）；singlefs-8b 转来 55 号判别力样本缺 raise-rollback-floor 档（stage-selftest 判错 2 个），用户定下次提交崩溃验证员跑 55 号带 `VM_KEEP=1` 补样本，并进第五步派发提示。第十步素材 ㊳：用户「你遇到的这些问题 稍后都要按照脚本的方式处理 修bug」——看门狗把驱动自检里一趟趟起的小分片认成没带前缀的重型测试（按 pid 确认压不住）；交回后留在后台的门禁进程没人停；派发闸要「重型测试：不跑」单独一行；内存准入排队时主 agent 看不出要等多久 | 驱动修好 → 真跑第一条流量速率 → 报用户；第五步照 ④ 的次序 |

里程碑三第十一项的原话，`.claude/kb/milestone/03-third-txn.md:93` 整行：

> - **原话**：「既然说到150亿文件的读写，这里就要给里程碑3 增加一个任务任务 快速的读取和写入这150亿文件。我觉得我们需要使用数据库，KV 存储似乎是一个比较合适的方法。」「这种规模文件的读写和持久化放在文件中 不靠谱。需要解决」（用户 2026-09-27）

### 3.1 第十一项「主 agent 的理解」（`03-third-txn.md:95`，整段标「待用户核，推的，没量过」）逐句对今天的仓

| # | 原句（逐字） | 推的还是有数 | 今天仓里对得上的事实 |
|---|---|---|---|
| a | 「要读写的不是 150 亿个文件，是 150 亿个崩溃状态」 | 推的（读法） | 状态总量仓里有三个互不相同的数：150 亿（原话）、问题单 :5 的「约 1.9 × 10¹⁰」、调度记录 :311 的「三条大流合计 30.4 亿状态」（推的）；另有一条流单独就是 12 230 590 578（`stage-inputs.tsv:40` 注释）。几个数的口径（哪几条流、快档还是全量）没有一处对齐过 |
| b | 「一个状态由（节点，段号，段内序号）唯一定出，镜像与持久集合都能从序号现算」 | 一半有代码 | 今天是（流，全局序号）→ 持久集合现算（`crash.rs:1917` `persisted_writes_of_state`），镜像由 `CrashImage{base, writes, persisted}` 现造（:1142–:1146）；「节点」今天不存在（第一项的节点树没实现） |
| c | 「真要落盘的只有每个状态的判定与去重用的键」 | 推的 | 今天什么判定都不按状态落盘（第二节）；E161 可行性档报 K4-walk、K4-units 在小域上不一致（E161 页 :82），去重键的形态没定 |
| d | 「裸文件在这个规模上缺五样：一批写一半的原子性、坏字节的校验、多机多卡写入的事务、数据与「哪个输入算出来的」元数据的绑定、按违例查询」 | 推的 | E162 S1 在 SIGKILL 下 F 臂没翻，但 F 在 Q1a–Q1f 上没有判别力（登记 4.10）；掉电（S4）没跑；多机写（S3）没有有效产物 |
| e | 「嵌入式事务 KV 存储（纯 Rust 的 redb 或 RocksDB，只进 harness）」 | 推的；「只进 harness」与今天的包划分对不上 | RocksDB 不是纯 Rust（`librocksdb-sys` 现编 C++，`research/e7-index-bench/Cargo.toml:453–:455`）；崩溃枚举引擎今天在 checker 档包（D13（验证路线） 已定项 15），见第四节 |
| f | 「值 = 区间里每个状态的判定指纹编号（不同的判定向量只有几十种，另存一张表）」 | 推的，没量过 | 仓里没有任何数量过「判定向量有几种」（第二节 2.1 那条 grep）；E162 只按 1 字节编号造合成数据（登记 4.1「本实验照问题单用 1 字节，不量种数」） |
| g | 「一块取 2¹⁶ 个状态，150 亿状态约 23 万个键、15–30 GB 值」 | 算术 + 推的 | 1.5 × 10¹⁰ ÷ 2¹⁶ = 228 881.8（登记 4.1、锚点 A-C2）；每状态 1 字节时 15 GB，「30 GB」要每状态 2 字节，出处没写（推的）；按 1.9 × 10¹⁰ 是 289 917 块、17.70 GiB（登记 4.7） |
| h | 「一块正好是 GPU 的一个批、双机分片的一个单位、门禁 54 号复用判定的一个单位」 | 推的；与今天三处的单位都不同 | 今天双机分片的单位是**片**，片长 ceil(状态数 ÷ 65 536)（并行线一那条流 186 625 个状态/片，不是 2¹⁶）；54 号复用单位是整条用例 × 输入指纹（第二节 2.5）；GPU 批今天由 E163 装置自己定（本调查没查） |
| i | 「违例另存一张小表」 | 已有类似物 | 发现日志（第二节 2.3）：按签名去重、每签名 3 个样本，已落地 |
| j | 「向量数据库答的是近似最近邻，这里全是精确匹配，近似命中会把两个状态当成一个，不适用。」 | 推的（读法） | 仓里没有向量库依赖（`grep -rn 'redb\|rocksdb\|sqlite\|lmdb\|sled\|heed' --include=Cargo.toml crates Cargo.toml \| wc -l` → `0`） |

同一段「仓里已有的」（:94）里「仓里没有任何数据库依赖（2026-09-27 现查各 `Cargo.toml`，sqlite / rocksdb / lmdb / 向量库都 0 命中）」今天只在 `crates/` 与根工作区成立：研究工作区已经有 redb 与 rocksdb（`research/e7-index-bench/Cargo.toml:457–:463`，E162 加的）。根 `Cargo.lock` 5 个包、`grep -c 'source = "registry' Cargo.lock` → `0`，即 `crates/` 今天一个外部依赖都没有。

## 四、接入约束

### 4.1 KV 库住哪个包，门禁 94 号怎么判

门禁 94 号（`.claude/gate.d/94-checker-implementation-disjoint.sh`）四条判据在文件头 :9–:23：① `singlefs-checker` 与 `singlefs-core` 各取传递闭包求交、减去 `singlefs-format` 必须为空，「外部 crate 在图上是叶子，仍然进闭包、仍然参与求交」（:12–:13），外部 crate 之间的传递依赖罩不到（:13–:14）；② checker 源码零处引 `singlefs_core`；③ 常量模块不许有分支循环；④ `singlefs-harness` 的闭包（含 dev-dependencies）里没有 `singlefs-checker-tier`。今天各包依赖（各 `crates/*/Cargo.toml` 现读）：format 无依赖；core → format；checker → format；harness → checker、core、format；checker-tier → checker、core、format、harness。

| 放进哪个包 | 94 号红不红（按判据推的，没跑 94 号） | 别的条款 |
|---|---|---|
| `singlefs-checker`（池级 checker） | 只要 core 的闭包里没有同一个外部 crate，① 不红 | D13（验证路线） 已定项 15 射程「不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）」（`.claude/kb/decisions/13-验证路线.md:320`）；`.claude/rules/verification.md:45`「它今天只依赖 `singlefs-format`、没有 `[dev-dependencies]`，这一句 94 号不单判」。门禁不拦，条款不许 |
| `singlefs-core` | 若 checker 也用同一个 KV crate，① 红 | 实现层不该带验证存储（推的）；不是候选 |
| `singlefs-harness` | ①④ 都不红（harness 不在 checker、core 的闭包里） | 里程碑三 :95 与调度记录 :325 写的是「只进 harness」「接进 harness」；但崩溃枚举引擎、进度文件、分片账本今天都在 checker 档包（`crates/singlefs-checker-tier/src/crash.rs`、`layer0_progress.rs`），harness 档「改了代码随时跑」（`.claude/rules/verification.md` 定义表），RocksDB 进 harness 会让每次 harness 构建都现编 C++（推的） |
| `singlefs-checker-tier` | ①④ 都不红（它不在 checker、core、harness 任何一个的闭包里） | 消费者（`Layer0Tally`、片、账本、发现日志）都在这个包；D13（验证路线） 已定项 15 定案把「断点续跑与双机分片」列在 checker 档（`13-验证路线.md:318`） |

「只进 harness」这句写于两档拆分（提交 `973e1f58`）之前的认识；拆分之后它与包划分的对应要重判（推的，没三方）。

### 4.2 加依赖动 `Cargo.lock`：哪些登记行会失效

`.claude/gate.d/stage-inputs.tsv`（43 行，16 条登记）里路径列含根 `Cargo.lock` 的 13 行（awk 按制表符切第二列逐词比 `Cargo.lock`）：

| 行 | 键 |
|---|---|
| :28 | `54-layer0-replay.sh` |
| :29 | `55-qemu-device-streams.sh` |
| :31 | `59-crates-mutation-replay.sh` |
| :32 | `74-model-differential.sh` |
| :33 | `87-replay.sh` |
| :34 | `E142`（另含 `research/Cargo.lock` 与 `research/e7-index-bench/Cargo.toml`） |
| :36–:43 | 八条崩溃枚举用例：`crash-case:layer0-first-stream`、`layer0-second-stream`、`floor-raise-pushed-by-the-session`、`c561-sigma-full`、`layer0-multi-record-publish-stream`、`layer0-tree-split-streams`、`layer0-position-addressed-tree-streams`、`crash-injection-fast-tier` |

这八条崩溃枚举用例的路径列都是 `crates/ Cargo.toml Cargo.lock`，所以改任何 `crates/*/Cargo.toml` 同样让它们全部失效；准入模块只减去「用例读不到的文件」（别的测试目标独占的测试文件、`crates/mutations.tsv`、`src/bin/`），不减清单与锁（`admission.py` 文件头 :82–:85、`stage-inputs.tsv:14–:18` 注释）。在研究工作区加依赖只动 `research/Cargo.lock`，登记它的只有 `E142` 那一行；`87-replay.sh` 那一行登记了 `research/e7-index-bench/` 整个目录。

构建环境进不进指纹：`admission.py` 的构建环境只收 `RUSTFLAGS`、`CARGO_*`、`RUSTC*` 这几类（文件头「输入指纹里的构建环境」一段）；`grep -c BINDGEN research/scripts/admission.py` → `0`。RocksDB 的 `BINDGEN_EXTRA_CLANG_ARGS` 这一类不进指纹；双机驱动核两台的 rustc 与 cargo 版本，`grep -c 'clang\|g++\|BINDGEN' research/scripts/layer0-shard-run.sh` → `0`，不核 C++ 工具链与 libclang（事实；它会不会造成两台构建不同是推的）。

### 4.3 「独立工作区」写在哪

| 处 | 原文 |
|---|---|
| 根 `Cargo.toml:2`、`:12` | 「research/ 是实验 workspace，自己有 Cargo.toml，这里显式排除。」、`exclude = ["research"]` |
| `research/Cargo.toml:1–:3` | `[workspace]`、`resolver = "2"`、`members = ["e7-index-bench"]` |
| `research/e7-index-bench/Cargo.toml:452–:456` | E162 两个存储候选的注释：可选依赖、挂在不默认打开的特性上，理由是 `librocksdb-sys` 现编 C++ 与 bindgen 找不到 `stdbool.h` |
| 登记 1.3（:40–:46） | 「不碰 `crates/` 下任何文件，不给 `crates/` 加依赖。」 |
| 调度记录 :325 | 「实验放 `research/e7-index-bench`（独立的 Cargo 工作区），不给 `crates/` 加依赖——加依赖会改 `Cargo.lock`……」（整行在第三节） |

## 五、设计上还开着的问题（按 `03-third-txn.md:96`「开工前要先定的」逐条）

`03-third-txn.md:96` 整行：

> - **开工前要先定的**：选型挪到里程碑二先做（用户 2026-09-27 定；问题单 `research/prompts/m2-crash-store-r1-forks.md`）；里程碑三接着定块多大、块键里带哪些指纹；两台机器各写各的库还是一个库，怎么合并；违例表与判定向量表的形态；与第一项、第四项一起定。

| # | 问题 | 今天仓里有没有能回答它的数 | 出处 |
|---|---|---|---|
| 1 | 选型：redb（R1 / R0）、RocksDB（K）、加固的文件（F） | **S1 有，够判**：四臂 200 次 SIGKILL 全 0（对 F 无判别力）。**S2 只有可行性档**（10⁴ 块，写入 19 339 984.1–48 638 773.4 状态/秒，冷读 p99 352–399 µs，占用峰值 1.0639–2.1495），Q2a 等 E161。**S3 没有有效数**（R1 半份作废）。**S4 没有**（装置写完没起虚机）。**几何敏感性、S1-large 没有** | 第一节 1.3–1.5 |
| 1a | S2 门槛要的「两机 CPU 合计判定产出速率」 | 登记的正式口径（E161 t_state × 两机核数）没有数。另有一种口径的实测：门禁 54 号第二条流「本机 23,700 个/秒、第二台 24,800 个/秒」（调度记录 :317，写「量过」）；登记 4.9 写明不用这个口径。拿它与 S2 可行性档的最低写入速率比，19 339 984.1 ÷ (23 700 + 24 800) ≈ 399 倍（只是算术，两个口径不同、S2 未判，不是判定） | 调度记录 :317；登记 :120（4.9） |
| 2 | 块多大 | 没有量过块大小的取舍。只有：E162 主取样点 2¹⁶、反向点 G-bs10（2¹⁰）没跑；两条层 0 流段长（登记 4.12 引 E161 登记 :545、:550）：写数少的段只出一个不满的块，只有 18、26 写的段满得了 2¹⁶；今天片长 ceil(状态数 ÷ 65 536) 与块大小无关（并行线一那条流 186 625 个状态/片） | 第一节 1.3；登记 :123（4.12）；第二节 2.2 |
| 3 | 块键里带哪些指纹 | 没有数。今天已有的指纹：54 号每条用例的输入指纹（sha256，`admission.py crash-case-manifest`）、进度文件名里的计划哈希、分片账本文件头的工具链（rustc、cargo、target）；E162 块键只留 16 字节指纹。相关约束：记录核对器的判定依赖持久集合，按镜像去重对它不适用（`13-验证路线.md:135`）；E161 可行性档 K4-walk、K4-units 在小域上不一致（E161 页 :82），按走到的单元复用判定出错 | 第二节 2.2、2.5 |
| 4 | 两台一个库还是各写各的、怎么合并 | 用户已定「两机公用」（问题单 :11 S3 行、登记 :9–:11）。今天的双机分片正是「各写一份账本、本机 merge」那一形（第二节 2.4）。一个库时另一台送块的吞吐：没有有效数（S3 回环 R1 半份作废，网络那一路 `q3a_states_per_second=18017998.3` 只当量级）；跨机格没写装置。要求的量级：第二台 24 800 状态/秒（调度记录 :317），按每状态 1 字节约 24.8 KB/秒（算术） | 第一节 1.3；调度记录 :317 |
| 5 | 违例表的形态 | 已有可用的现成形态：发现日志（按签名去重、每签名 3 个样本、定稿与切法无关），54 号已在读 | 第二节 2.3 |
| 6 | 判定向量表的形态 | 没有。仓里没有量过判定向量有几种；「几十种」是推的（`03-third-txn.md:95`）。今天一个状态的「判定」散在 `Layer0Tally` 的几类计数里（恢复结局三选一、两遍恢复是否不同、验证跑没跑与失败、根槽是否持久、oracle 与不看 journal 的 oracle、记录核对器两条、每条不变量评估 / 违例 / 不适用），哪几样进向量没定（推的映射） | `crash.rs:743–:779` |
| 7 | 与第一项（节点树）一起定 | 没有：节点、按节点共享前缀都没实现；状态今天按（流，全局序号）编号 | `crash.rs:1813–:1816` |
| 8 | 与第四项（GPU）一起定 | E163（GPU多卡算单元校验和） 可行性：单机两卡 2560/2560 与 CPU 相同，双机五卡里一张起不来（`.claude/kb/experiments.md:186`）；显存预算「10¹⁰ 个状态时单卡每状态 3.419 字节、五卡每状态 10.26 字节」（`research/prompts/e161-preregistration.md:174`）。块怎么对上 GPU 批没有数 | 同左 |
| 9 | 状态总量口径 | 150 亿、1.9 × 10¹⁰、30.4 亿（推的）、单条流 12 230 590 578 四个数并存，没有一处说清各指哪几条流、快档还是全量 | 第三节 3.1 a 行 |
| 10 | KV 库住哪个包 | 没有定；按判据推只有 checker 档包与 harness 过得了 94 号，池级 checker 被 D13（验证路线） 已定项 15 射程挡着（第四节 4.1） | 第四节 |
| 11 | 掉电之后块还在不在 | 没有（S4 没跑）；装置自己报的风险：验盘把设备侧日志塞进 initramfs 可能超 QEMU initrd 上限（推的，没量过） | `research/prompts/e162-s4-apparatus-runner-report.md:34` |
| 12 | 空间 | 可行性档占用比 × 1.9 × 10¹⁰ 字节都落在当时 `df -B1` 可用字节内（约 1.98 × 10¹² 字节）；1.9 × 10¹⁰ 字节块值 = 17.70 GiB（登记 4.7） | 实验页 :68；登记 :118（4.7） |

## 六、没做什么

- 没编译、没跑任何测试、实验、门禁（派发「重型测试：不跑」；另一会话的层 0 全量占着两台机器）；第四节 4.1 的「94 号红不红」是按它文件头的判据推的，没跑 94 号。
- 没读 E163 的实验页与装置（第五节第 8 行只引了索引行）；没查 GPU 批今天怎么切。
- 没读 E161 跑前登记 :545、:550 原文，段长数只经 E162 登记 4.12 转引。
- 没核调度记录 :311、:317 里速率与状态总量是怎么量出来的（照原文标「量过」或「推的」）。
- 发现两处与本任务无关、没改：`crates/singlefs-checker-tier/src/layer0_progress.rs:14` 与驱动文件头仍把双机分片叫「里程碑三第六项」，而 `03-third-txn.md` 2026-09-27 已重新编号（第六项今天是「多次 COW 下的读写一致」）；E162 登记第三节的 `crates/singlefs-harness/src/…` 路径已过时（冻结证据，不改）。
- 草稿目录 `/tmp/claude-1000/m3-facts-kv/` 里只有从仓里抽出来比对引文用的几行文本；没有编译目录、工作树或仓副本。
````

**出处 `research/prompts/m3-prune-gpu-r1-facts-k4.md:1-289`（整段抄，未转述）**

````markdown
# E161 K4-walk / K4-units 同键异判调查（2026-09-27）

只查不修。主仓一个字没改；所有改动在 `/tmp/claude-1000/investigate-e161-k4/` 的副本里（补丁留在 `probe-harness.patch`、`probe-walk.patch`）。

## 结论

1. 这 12 个不一致都出在 journal 环：同键的两边，单元区里读到的内容与所选根完全相同，差别只在 journal 环里多持久了一两条记录（`JournalRecord` 写）。池级 checker 有四条判定读这些记录，K4 的键却只收单元区里的读（装置 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:1846` `.filter(|read| read.offset >= UNIT_AREA_START_BYTES)`），环里的读（偏移 16777216 起）进不了键。
   - A 组（first_small 序号 9、10、11，K4-walk 与 K4-units 各算 3 个）：I-8.6、I-8.9 从「不适用」变「成立」。环里多了第一条自证过的记录（w2 / w3），读它的是 `scanned_journal_records_of_device`。
   - B 组（first_small 序号 150994968、150994969、150994970，只在 K4-units 里算 3 个）：I-3.10 从「不适用」变「成立」。多出的记录（w36 / w37）让 `tree_table_pointer_of_the_version_the_next_mount_applies_first` 从 none 变成 some，I-3.10 于是多读一棵分配记录树。
   - C 组（second_quick 序号 18、19、20，K4-units 3 个）：I-8.7 从「不适用」变「成立」。多出的记录是 w65 / w66。
2. 记录核对器不参与 K4 的判定：K4 两条臂的结局指纹只取 `check_pool_image` 的判定表（装置 `:2249`、`:2268`、`:2273`），`check_records`（`:2055`）不进这个指纹。
3. 最小补键：在 K4 键里加上「checker 正常那一遍在 journal 环里的读（位置 + 内容）」，四个格上的不一致都归 0。只加「journal 候选槽的答案」也能归 0，键数完全一样。补键后的不同键数：first_small 两条臂都从 5 / 4 涨到 13（状态 37 个）；second_quick K4-walk 从 7 涨到 16、K4-units 从 8 涨到 20（状态 84 个）。键数没有涨到接近状态数，但这只是这两个小格上量到的数。
4. 推翻条件：让 checker 看不见 journal 环以后，原键仍有不一致 ≥ 1；或者补了环读的键在这两格上仍有不一致 ≥ 1。这两条都在副本里量过，都是 0。另外在副本里故意造了一个环外的判定依赖，补了环读的键确实报出不一致（3 个、6 个），说明「归 0」不是计数失灵。

## 一、复现

### 1.1 现工作区复现不出：流的形状变了

先在现工作区的副本里跑（仓 HEAD `e5253e8a`，另带工作区里没提交的 crates 改动；装置文件与 HEAD 相同，`diff` 无输出）。形状行：

```
E7RESULT name=investigate_shape cell=first_small writes=41 segments=11 segment_lengths=[2,2,1,2,2,1,2,24,2,1,2] counts=[8,3,1,8,3,1,8,16777215,3,1,8] ordinals=16777260
E7RESULT name=investigate_shape cell=second_quick writes=477 segments=78 segment_lengths=[...] ... ordinals=48
```

产物里第一条流是 10 段（`research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out` 第 16 行 `sampled_segment_states=[8, 3, 1, 8, 3, 1, 150994943, 3, 1, 8]`），第 3 行 `stream=first writes=41 segments=10`，第 4 行 `stream=second writes=477 segments=64`。现在的树把 26 写那段拆成了 `2,24`，所以 `small_domain_ordinals`（写数少于 26 的段全取）一下子变成 16777260 个状态。这次运行跑到 `progress cell=first_small slices=512/65537` 时被我停了（`proc.py stop`）。
在 `c3540c02`（第一个带 e161 装置的提交）上结果一样（11 段、16777260、48）。

### 1.2 在产物时刻之前最近的 crates 提交上复现出来了

产物文件的时间是 2026-09-27；在它之前最近一次动过 crates 的提交是 `9e56db41`。复现的做法：`git archive 9e56db41` 导出这棵树，再换进 `c3540c02` 的装置文件与 `tests/common/mod.rs`（装置第一次入库就是这一版），在装置里加一个只跑两格的模式 `investigate-k4`。这个模式调的仍是 feasibility 用的 `run_cell` + `report_cell`，外加逐状态探查。

```
E161_THREADS=4 nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 /tmp/claude-1000/investigate-e161-k4/target-9e56db41/release/e161_crash_state_dedup_and_time_split investigate-k4
```

跑出的形状：first_small 10 段、`ordinals=37`；second_quick 64 段、`ordinals=84`，都与产物一致。拿两格的 `reuse_arm`、`g1`、`determinism`、`g5_contents` 行（每格 9 行）与产物逐字比：

```
first_small identical 9 lines
second_quick identical 9 lines
```

其中与这次问题有关的三行，复现结果与产物逐字相同：

```
E7RESULT name=reuse_arm cell=first_small arm=k4_walk states=37 inconsistent=3 hits=32 distinct_keys=5 ...
E7RESULT name=reuse_arm cell=first_small arm=k4_units states=37 inconsistent=6 hits=33 distinct_keys=4 ...
E7RESULT name=reuse_arm cell=second_quick arm=k4_units states=84 inconsistent=3 hits=76 distinct_keys=8 ...
```

说明：
- 产物时刻的工作区没有留下快照，`9e56db41` 加 `c3540c02` 装置这套组合不能证明就是当时那一份代码。能说的只是：这两格的 18 行非计时产物逐字对得上。
- 这套模型是确定性的。run2、run3、run4 三次跑，84 行 `reuse_arm` / `investigate_key` / `investigate_group` 完全相同。这只能说明没有隐藏状态，不能当统计上稳定来用。
- 实验页「复跑」一节的命令写的是 `-p singlefs-harness`，装置现在已搬到 `singlefs-checker-tier`（只记下，没核那条命令现在能不能跑）。
## 二、机理：逐个状态

探查输出在 `run3.out` / `run3.err`（下面贴的行里，`ring_reads=[…]` 与 `candidate_journal=[…]` 两个字段已删掉，别的字段原样）。参照态指的是同键里序号最小的那个状态（`count_arm` 的定义）。

### 2.1 不一致的状态、段号、段内序号，以及和参照态差在哪条判定

```
cell=first_small arm=k4_walk key=e2f5ea4a ordinal=9 segment=1 within=1 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_walk key=e2f5ea4a ordinal=10 segment=1 within=2 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_walk key=e2f5ea4a ordinal=11 segment=2 within=0 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_units key=ce431fc6 ordinal=9 segment=1 within=1 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_units key=ce431fc6 ordinal=10 segment=1 within=2 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_units key=ce431fc6 ordinal=11 segment=2 within=0 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_units key=3008d99a ordinal=150994968 segment=7 within=1 same_as_reference=false unit_elements=62 walk_elements=36 records=both_false verdicts_differing_from_reference=[I-3.10=Holds]
cell=first_small arm=k4_units key=3008d99a ordinal=150994969 segment=7 within=2 same_as_reference=false unit_elements=62 walk_elements=36 records=both_false verdicts_differing_from_reference=[I-3.10=Holds]
cell=first_small arm=k4_units key=3008d99a ordinal=150994970 segment=8 within=0 same_as_reference=false unit_elements=62 walk_elements=36 records=both_false verdicts_differing_from_reference=[I-3.10=Holds]
cell=second_quick arm=k4_units key=fe1356dc ordinal=18 segment=10 within=1 same_as_reference=false unit_elements=114 walk_elements=84 records=both_false verdicts_differing_from_reference=[I-8.7=Holds]
cell=second_quick arm=k4_units key=fe1356dc ordinal=19 segment=10 within=2 same_as_reference=false unit_elements=114 walk_elements=84 records=both_false verdicts_differing_from_reference=[I-8.7=Holds]
cell=second_quick arm=k4_units key=fe1356dc ordinal=20 segment=11 within=0 same_as_reference=false unit_elements=114 walk_elements=84 records=both_false verdicts_differing_from_reference=[I-8.7=Holds]
```

参照态上这几条判定的原文：

```
cell=first_small arm=k4_walk ordinal=0:
  I-3.10=NotApplicable("候选集里没有一棵分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、位置对不上）") 
  I-8.6=NotApplicable("环里没有一条自证过的记录找得到本实例内逻辑前一条：链判不了") 
  I-8.7=NotApplicable("环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来") 
  I-8.9=NotApplicable("环里一条自证过的记录都没有：没有哪一次发布的序号与末条标志可判") 
cell=first_small arm=k4_units ordinal=150994967:
  I-3.10=NotApplicable("候选集里没有一棵分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、位置对不上）") 
  I-8.6=Holds 
  I-8.7=NotApplicable("环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来") 
  I-8.9=Holds 
cell=second_quick arm=k4_units ordinal=1:
  I-3.10=Holds 
  I-8.6=Holds 
  I-8.7=NotApplicable("环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来") 
  I-8.9=Holds 
```

### 2.2 和参照态比，持久集合差在哪几次写

```
cell=first_small arm=k4_units ordinal=8 vs_reference=0 writes=[w0:+:SystemConfigurationSlot:dev0:off0:len4096,w1:+:SystemConfigurationSlot:dev1:off0:len4096]
cell=first_small arm=k4_units ordinal=9 vs_reference=0 writes=[w0:+:SystemConfigurationSlot:dev0:off0:len4096,w1:+:SystemConfigurationSlot:dev1:off0:len4096,w2:+:JournalRecord:dev0:off16777216:len4096]
cell=first_small arm=k4_units ordinal=10 vs_reference=0 writes=[w0:+:SystemConfigurationSlot:dev0:off0:len4096,w1:+:SystemConfigurationSlot:dev1:off0:len4096,w3:+:JournalRecord:dev1:off16777216:len4096]
cell=first_small arm=k4_units ordinal=11 vs_reference=0 writes=[w0:+:SystemConfigurationSlot:dev0:off0:len4096,w1:+:SystemConfigurationSlot:dev1:off0:len4096,w2:+:JournalRecord:dev0:off16777216:len4096,w3:+:JournalRecord:dev1:off16777216:len4096]
cell=first_small arm=k4_units ordinal=150994968 vs_reference=150994967 writes=[w36:+:JournalRecord:dev0:off16785408:len4096]
cell=first_small arm=k4_units ordinal=150994969 vs_reference=150994967 writes=[w37:+:JournalRecord:dev1:off16785408:len4096]
cell=first_small arm=k4_units ordinal=150994970 vs_reference=150994967 writes=[w36:+:JournalRecord:dev0:off16785408:len4096,w37:+:JournalRecord:dev1:off16785408:len4096]
cell=second_quick arm=k4_units ordinal=17 vs_reference=1 writes=[w39:+:SystemConfigurationSlot:dev0:off4096:len4096,w40:+:SystemConfigurationSlot:dev1:off4096:len4096]
cell=second_quick arm=k4_units ordinal=18 vs_reference=1 writes=[w39:+:SystemConfigurationSlot:dev0:off4096:len4096,w40:+:SystemConfigurationSlot:dev1:off4096:len4096,w65:+:JournalRecord:dev0:off16789504:len4096]
cell=second_quick arm=k4_units ordinal=19 vs_reference=1 writes=[w39:+:SystemConfigurationSlot:dev0:off4096:len4096,w40:+:SystemConfigurationSlot:dev1:off4096:len4096,w66:+:JournalRecord:dev1:off16789504:len4096]
cell=second_quick arm=k4_units ordinal=20 vs_reference=1 writes=[w39:+:SystemConfigurationSlot:dev0:off4096:len4096,w40:+:SystemConfigurationSlot:dev1:off4096:len4096,w65:+:JournalRecord:dev0:off16789504:len4096,w66:+:JournalRecord:dev1:off16789504:len4096]
```

### 2.3 checker 里面：journal 环读出来的记录、下一次挂载先施加的那一版、分配记录树指针数

这一节靠副本里 `walk.rs` 加的打印（`probe-walk.patch`，开关只在探查那一次 `check_pool_image` 调用上打开），输出在 `run3.err`。journal_records 每项是 (jsn 计数器, 实例, txg, 提交标记)：

```
INVESTIGATE_STATE cell=first_small ordinal=0 || INVESTIGATE journal_records=0:[];1:[] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=8 || INVESTIGATE journal_records=0:[];1:[] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=9 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present")];1:[] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=10 || INVESTIGATE journal_records=0:[];1:[(1, 1, 1, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=11 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present")];1:[(1, 1, 1, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=150994967 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present"), (2, 1, 2, "Present")];1:[(1, 1, 1, "Present"), (2, 1, 2, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=150994968 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present"), (2, 1, 2, "Present"), (3, 1, 3, "Present")];1:[(1, 1, 1, "Present"), (2, 1, 2, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=1
INVESTIGATE_STATE cell=first_small ordinal=150994969 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present"), (2, 1, 2, "Present")];1:[(1, 1, 1, "Present"), (2, 1, 2, "Present"), (3, 1, 3, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=1
INVESTIGATE_STATE cell=first_small ordinal=150994970 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present"), (2, 1, 2, "Present"), (3, 1, 3, "Present")];1:[(1, 1, 1, "Present"), (2, 1, 2, "Present"), (3, 1, 3, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=1
```

second_quick 的几行（journal_records 字段太长，这里删了；计数器 4 的记录在 second_quick 里出现在序号 18–83 这 66 个状态上（`grep -c '(4, '` 数出 66）；同键 fe1356dc 那一组的成员是 1、3、5、7、9、11、13、15、17、18、19、20，其中只有 18、19、20 带它。原文在 `run3.err`）：

```
INVESTIGATE_STATE cell=second_quick ordinal=1 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=1
INVESTIGATE_STATE cell=second_quick ordinal=17 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=1
INVESTIGATE_STATE cell=second_quick ordinal=18 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=2
INVESTIGATE_STATE cell=second_quick ordinal=19 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=2
INVESTIGATE_STATE cell=second_quick ordinal=20 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=2
```

### 2.4 机理，以及读了键外输入的那几行

三组的共同点：同键的两边，单元区里读到的「位置 + 内容」集合与所选根逐项相同（`unit_elements` 相同，键也相同），持久集合只差 `JournalRecord` 写。只差系统配置槽写的状态（first_small 1–8、second_quick 17）判定都与参照态相同，所以系统配置槽不是这次的原因。

- A 组，I-8.6 / I-8.9：参照态（序号 0、8）上，环里读不出一条自证过的记录，两条都是「不适用」。序号 9、10、11 在某一块盘的环槽 0 上持久了一条记录（w2 / w3，偏移 16777216），checker 读出它，这两条就判成了「成立」。
- B 组，I-3.10：参照态 150994967 上环里有计数器 1、2 两条记录，`next_mount_applies_first_tree_table=none`，分配记录树指针数 0，I-3.10「不适用」。150994968–70 在环槽 2 上多了 txg 3 的记录（w36 / w37，偏移 16785408），`tree_table_pointer_of_the_version_the_next_mount_applies_first` 于是交回 some，指针数变成 1，I-3.10 读到了未释放的记录，判成「成立」。这里走的不是 `versions_applied_only_by_records`：这几个状态上它都是 `[]`。
- C 组，I-8.7：同键那一组里，只有 18、19、20 在环槽 3 上多了计数器 4 的记录（w65 / w66，偏移 16789504）。这条记录让「同一实例的两条非 0 事务号」凑成了对，I-8.7 从「不适用」变成「成立」。I-3.10 在参照态上已经「成立」，所以不在差异里。

读了键外输入的行。K4 的键只收单元区里的读，下面这几行读的都是 journal 环，或者读的东西由 journal 环决定：

| 作用 | 复现树（9e56db41 的 `crates/singlefs-checker/src/walk.rs`） | 现工作区 `crates/singlefs-checker/src/walk.rs` |
|---|---|---|
| 扫 journal 候选槽（答案随环里持久了哪几条写变） | 4634 `fn scanned_journal_records_of_device(`，4642 `.candidate_journal_slots(device)` | 4848，4856 |
| 读环里整条 4096 字节的记录 | 4647 `let Some(bytes) = reader.read(device, offset, record_bytes) else {` | 4861 |
| check_pool_image 里取环记录、交给 I-8.x | 5529 `let journal_records_by_device =`，5531 `judge_journal_back_chain(...)`，5536 `judge_transaction_numbers_per_instance(...)` | 5769，5771，5776 |
| I-8.6 判定 | 4837 `judgements.judge("I-8.6", record.back_chain == expected_back_chain, \|\| {` | 5051 |
| I-8.7 判定 | 4924 `judgements.judge("I-8.7", record.transaction > previous_transaction, \|\| {` | 5138 |
| I-8.9 判定 | 5232 `judgements.judge("I-8.9", true, String::new);` | 5446 |
| 下一次挂载先施加的那一版：由环记录定 | 3335 `fn tree_table_pointer_of_the_version_the_next_mount_applies_first(` | 3549 |
| 把那一版的树表并进 I-3.10 要读的分配记录树 | 3391 `tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);`，调用点 5784 `let allocation_node_pointers = allocation_record_node_pointers_of_the_candidate_versions(` | 3605，调用点 6024 |
| I-3.10 判定 | 3460 `judgements.judge("I-3.10", record.generation == birth_txg, \|\| {` | 3674 |

装置这一边（现工作区的 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`，与 HEAD 相同）：
- 1846 `.filter(|read| read.offset >= UNIT_AREA_START_BYTES)`：K4-walk、K4-units 的集合只收单元区的读，环里的读（`RING_START_BYTES` ≤ 偏移 < `UNIT_AREA_START_BYTES`）与 journal 候选槽的答案都被丢掉。
- 2249 `let verdict_outcome = fingerprint_of_debug_text(&verdicts);`、2268 `set_key(root, &walk_elements),`、2273 `set_key(root, &unit_elements),`：结局只取池级 checker 的判定。2055 `let records = check_records(image, consulted.effective_root);` 的结果不进 K4 的结局，所以记录核对器里没有哪一行造成这 12 个不一致。

只看了扫描方向、持久集合、oracle 版本号三样，排除的根据如下（每样各凭一条观测）：
- 扫描方向（`candidate_unit_slots`）：把它的答案加进键，不一致数不变（first_small 3 / 6，second_quick 3），见 3.1。
- 持久集合：它确实能区分这些状态，但它是经 journal 环读传进 checker 的。只加环读就已经归 0，所以没必要再往键里放持久集合；放进去键数等于状态数（37 / 84）。
- oracle 版本号：K4 的结局里没有 oracle（装置 2249 只取 `verdicts`）。

### 2.5 为什么 K4-walk 是 3 个、K4-units 是 6 个

B 组在 K4-walk 里不同键：参照态的 `walk_elements=4`，150994968–70 是 36（2.1 那几行）。多出来的读，来自 checker 顺着「下一次挂载先施加的那一版」去读分配记录树与单元头，走树那一遍记下了它们。正常那一遍对每个候选槽本来就都读一遍（`unit_elements=62`，两边相同），集合去重之后看不出这几次读是判 I-3.10 读的，所以 K4-units 把 B 组并成了一个键。
C 组在 K4-walk 上是 0，也只是凑巧：计数器 4 的记录同时打开了「下一次挂载先施加的那一版」（指针数 1→2），走树的读集跟着变了。K4-walk 本身并没有收进 I-8.7 的输入。
旁证：让 checker 看不见环以后（第四节），first_small 上 K4-walk 的键数从 5 降到 4，B 组的三个又并回了参照态那个键。

## 三、最小补键（副本里改键，只跑 first_small 与 second_quick）

### 3.1 各种补法的不一致数与不同键数（`run3.out`，run2 / run4 逐字相同）

`added=` 是往 K4 集合里另外并进去的元素（取自 checker 正常那一遍的整份调用，也按「位置 + 答案」算）：`ring_reads` 是环里的读；`candidate_journal_slots` / `candidate_unit_slots` 是两种候选槽调用的答案；`fixed_structure` 是环起点之前的读，加上盘列表与盘大小；`all_normal_calls_as_set` 是整份调用当成集合；`persisted_set` 是把持久集合掩码与原键配对。

```
cell=first_small arm=k4_walk added=none states=37 inconsistent=3 distinct_keys=5
cell=first_small arm=k4_walk added=ring_reads states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_walk added=candidate_journal_slots states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_walk added=ring_reads_and_candidate_journal_slots states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_walk added=candidate_unit_slots states=37 inconsistent=3 distinct_keys=5
cell=first_small arm=k4_walk added=fixed_structure states=37 inconsistent=3 distinct_keys=29
cell=first_small arm=k4_walk added=all_normal_calls_as_set states=37 inconsistent=0 distinct_keys=37
cell=first_small arm=k4_walk added=persisted_set states=37 inconsistent=0 distinct_keys=37
cell=first_small arm=k4_units added=none states=37 inconsistent=6 distinct_keys=4
cell=first_small arm=k4_units added=ring_reads states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_units added=candidate_journal_slots states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_units added=ring_reads_and_candidate_journal_slots states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_units added=candidate_unit_slots states=37 inconsistent=6 distinct_keys=4
cell=first_small arm=k4_units added=fixed_structure states=37 inconsistent=3 distinct_keys=29
cell=first_small arm=k4_units added=all_normal_calls_as_set states=37 inconsistent=0 distinct_keys=37
cell=first_small arm=k4_units added=persisted_set states=37 inconsistent=0 distinct_keys=37
cell=second_quick arm=k4_walk added=none states=84 inconsistent=0 distinct_keys=7
cell=second_quick arm=k4_walk added=ring_reads states=84 inconsistent=0 distinct_keys=16
cell=second_quick arm=k4_walk added=candidate_journal_slots states=84 inconsistent=0 distinct_keys=16
cell=second_quick arm=k4_walk added=ring_reads_and_candidate_journal_slots states=84 inconsistent=0 distinct_keys=16
cell=second_quick arm=k4_walk added=candidate_unit_slots states=84 inconsistent=0 distinct_keys=11
cell=second_quick arm=k4_walk added=fixed_structure states=84 inconsistent=0 distinct_keys=47
cell=second_quick arm=k4_walk added=all_normal_calls_as_set states=84 inconsistent=0 distinct_keys=84
cell=second_quick arm=k4_walk added=persisted_set states=84 inconsistent=0 distinct_keys=84
cell=second_quick arm=k4_units added=none states=84 inconsistent=3 distinct_keys=8
cell=second_quick arm=k4_units added=ring_reads states=84 inconsistent=0 distinct_keys=20
cell=second_quick arm=k4_units added=candidate_journal_slots states=84 inconsistent=0 distinct_keys=20
cell=second_quick arm=k4_units added=ring_reads_and_candidate_journal_slots states=84 inconsistent=0 distinct_keys=20
cell=second_quick arm=k4_units added=candidate_unit_slots states=84 inconsistent=3 distinct_keys=8
cell=second_quick arm=k4_units added=fixed_structure states=84 inconsistent=3 distinct_keys=72
cell=second_quick arm=k4_units added=all_normal_calls_as_set states=84 inconsistent=0 distinct_keys=84
cell=second_quick arm=k4_units added=persisted_set states=84 inconsistent=0 distinct_keys=84
```

读法：
- 往键里至少再加一样：checker 正常那一遍在 journal 环里的读（位置 + 内容）。加上之后，两格两臂的不一致都是 0。不同键数：first_small 5→13（K4-walk）、4→13（K4-units），状态 37 个；second_quick 7→16（K4-walk）、8→20（K4-units），状态 84 个。键数与状态数之比从 0.083–0.135 涨到 0.190–0.351（`python3` 算的），离「每个状态一个键」还远，但这只是两个小格上的量。26 写大段上会怎样没量，推不出来。
- 只加 `candidate_journal_slots` 的答案，不一致与键数和加环读完全一样。不过这个答案来自枚举器（`crates/singlefs-harness/src/memory_pool.rs` 的 `candidate_journal_slots` 只看哪几次环写持久了），不带内容。这两格里恰好一样，不能拿来说明一般情况。
- 只加固定结构那一类的读，K4-units 在两格上还剩 3 个，K4-walk 在 first_small 上还剩 3 个，键数却涨到 29 / 47 / 72。补固定结构补错了地方。
- 加单元候选槽，两格两臂的数一样都不变。
- 「加环读就够」是在这两格上量出来的，不是构造上保证的。构造上保证同键同判的，只有整份调用当成集合这一种，而它在这两格上键数等于状态数（37 / 84），也就是说没有复用可言。

## 四、推翻条件，以及在副本里造出来的那两次

这个定位的说法是：同键异判全部来自池级 checker 读 journal 环（2.4 的表），而 K4 的键丢掉了环里的读。下面任何一条出现，这个说法就是错的：

1. 让 checker 看不见 journal 环以后，原 K4 键在这两格上还有不一致 ≥ 1。
2. 往键里补上环里的读以后，这两格上还有不一致 ≥ 1。
3. 某个不一致的状态，与它的参照态比，持久集合里没有任何一次 `JournalRecord` 写不同。

量到的：
- 条件 2：见 3.1，四行 `added=ring_reads` 的 `inconsistent` 都是 0。
- 条件 3：见 2.2，9 个不一致状态各自差的写里都有 `JournalRecord`。
- 条件 1：在副本的 `scanned_journal_records_of_device` 开头加了一个开关：设了 `E161_INVESTIGATE_BLIND_JOURNAL` 就交回空表（`probe-walk.patch`）。打开它跑（`run5-blind.out`）：

```
E161_INVESTIGATE_BLIND_JOURNAL=1 E161_THREADS=4 nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 <同一个二进制> investigate-k4
```
```
E7RESULT name=reuse_arm cell=first_small arm=k4_walk states=37 inconsistent=0 hits=33 distinct_keys=4 keys_over_states=0.108108 keys_at_or_a
E7RESULT name=reuse_arm cell=first_small arm=k4_units states=37 inconsistent=0 hits=33 distinct_keys=4 keys_over_states=0.108108 keys_at_or_
E7RESULT name=reuse_arm cell=first_small arm=k4_full states=37 inconsistent=0 hits=0 distinct_keys=37 keys_over_states=1.000000 keys_at_or_a
E7RESULT name=reuse_arm cell=second_quick arm=k4_walk states=84 inconsistent=0 hits=80 distinct_keys=4 keys_over_states=0.047619 keys_at_or_
E7RESULT name=reuse_arm cell=second_quick arm=k4_units states=84 inconsistent=0 hits=76 distinct_keys=8 keys_over_states=0.095238 keys_at_or
E7RESULT name=reuse_arm cell=second_quick arm=k4_full states=84 inconsistent=0 hits=0 distinct_keys=84 keys_over_states=1.000000 keys_at_or_
```

打开开关以后，两份输出里 I-8.6 / I-8.7 / I-8.9 判成 Holds 的行数（`grep -c`）：

```
run5-blind.out:0
run4.out:11
```

打开开关以后，两格上 K4-walk、K4-units 用原键的不一致都是 0，环类判定一条都没有判成 Holds。说明除了环以外，没有别的键外输入在这两格上制造不一致。

「补了环读的键不一致为 0」这个计数会不会报红，要单独验：在探查里把判定指纹换成（原判定，第一个在本格里取值有变的系统配置槽写是否持久），也就是故意让结局依赖一样既不在单元区、也不在环里的输入，再用同一套计数去数（`run4.out`）：
```
name=investigate_injected cell=first_small arm=k4_walk added=none injected_outcome_input=persisted_w0 states=37 inconsistent=6 distinct_keys=5
name=investigate_injected cell=first_small arm=k4_walk added=ring_reads injected_outcome_input=persisted_w0 states=37 inconsistent=3 distinct_keys=13
name=investigate_injected cell=first_small arm=k4_units added=none injected_outcome_input=persisted_w0 states=37 inconsistent=9 distinct_keys=4
name=investigate_injected cell=first_small arm=k4_units added=ring_reads injected_outcome_input=persisted_w0 states=37 inconsistent=3 distinct_keys=13
name=investigate_injected cell=second_quick arm=k4_walk added=none injected_outcome_input=persisted_w39 states=84 inconsistent=6 distinct_keys=7
name=investigate_injected cell=second_quick arm=k4_walk added=ring_reads injected_outcome_input=persisted_w39 states=84 inconsistent=6 distinct_keys=16
name=investigate_injected cell=second_quick arm=k4_units added=none injected_outcome_input=persisted_w39 states=84 inconsistent=9 distinct_keys=8
name=investigate_injected cell=second_quick arm=k4_units added=ring_reads injected_outcome_input=persisted_w39 states=84 inconsistent=6 distinct_keys=20
```

补了环读的键在造出来的环外依赖上报出 3 / 3 / 6 / 6 个不一致，所以第三节里那几个 0 不是计数失灵造成的。

## 五、三次推导

- 正推：K4 的键只收单元区里的读（装置 1846）；checker 的 I-8.6 / I-8.7 / I-8.9 读环里的记录，I-3.10 的读法也由环里的记录决定（2.4 的表）。那么只差环写的两个状态可以同键，判定却不同。2.2 的持久集合差与 2.1 的判定差一一对上。
- 反推：如果这个说法是错的，应当看到下面三样中的一样：补了环读仍有不一致、看不见环仍有不一致、有不一致的一对不差环写。三样都量了，都没有出现（第四节）。
- 校验：换一条不经 K4 键计数的路，直接在 checker 内部打印环记录与分配记录树指针数（2.3，`walk.rs` 里的打印），再用持久集合的写差（2.2，从枚举计划算，不经读者）对一遍。两条路都指向同一批 `JournalRecord` 写。计数路子本身会不会报红，用 4 里造出来的依赖验过。

## 六、另外看到的（没展开）

- 在现工作区（`e5253e8a` 加没提交的 crates 改动）与 `c3540c02` 上，第一条流是 11 段 `[2,2,1,2,2,1,2,24,2,1,2]`，`small_domain_ordinals` 交回 16777260 个状态，second_quick 是 48 个状态。在现在的树上直接跑 feasibility，first_small 这一格已经不是 37 个状态的小格了。S2 那几条在现在的树上判什么，我没跑，所以没核。

## 七、没做什么

- 没修，也没判该怎么改键、该不该用复用臂。
- 没跑任何重型测试、门禁、层 0，也没跑 cargo test。只跑了 e161 装置二进制加进去的 `investigate-k4` 模式，范围是 first_small 与 second_quick 两格，外加一次只打印形状的运行。现工作区上那一次误跑了大域，跑到 first_small 的 512/65537 片时被我停掉。
- first_quick 与 segment_head 两格没跑。产物里 first_quick 上 K4-walk 3 个、K4-units 6 个，是不是同一机理，没验。
- 产物当时用的那份代码没有留快照，没法逐字确认就是 9e56db41 加 c3540c02 装置这一套。能确认的只是两格 18 行非计时产物逐字一致。
- 26 写大段上补了环读之后的键数没量，4.12 说的「键数接近状态数」这次没有碰到。
- 副本里的改动没回主工作区；副本与编译目录交回前删掉，补丁与输出留在草稿目录。
````

**出处 `research/prompts/m3-prune-gpu-r1-facts-case-design.md:1-283`（整段抄，未转述）**

```markdown
# m3-prune-gpu-r1 事实表戊：崩溃用例按操作序列的设计审查

调度记录 `records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 任务戊。只读代码与 kb，没编译、没跑任何测试。
行号都是这一次现查的（`grep -n`）。状态数照用例里钉的常量或断言原样引；分段、按节点归并的数用 `/tmp/claude-1000/m3-facts-case-design/calc.py` 拿钉死的段数组现算（`python3`），标「现算」；从代码读出、没有数撑的标「推的」。

**口径**：「枚举」指层 0 段模型（前面的段全持久 + 当前段各写取两态或三态的任意组合，`crash.rs:2177`–`:2179`）在这一截上被展开；「基线」指这一截整段施加进起点镜像或只以整段持久进入后面（`Layer0SegmentExpansion::NotExpanded`）、不取子集。一个节点的状态数按「写它的那次操作」归（上一次发布的系统配置轮换归上一次发布），与用例 `states_by_publish` 按「后面最近那条根槽写」归不同；两种归法合计相同（现算）。

## 一　按操作列每条用例（问题 1）

### 1.1 门禁 54 号 `--full` 登记的全量（`.claude/gate.d/stage-inputs.tsv:36`–`:43`）

| 用例（文件:行） | 操作序列（用例自己的叫法） | 基线、不枚举的一截 | 枚举的一截：段长 → 状态数 |
|---|---|---|---|
| P1 第一条流 `crash_enumeration_new_pool_file_creation_stream.rs:498` | mkfs → 取号 → 暖机 × 2 → A（新池新建文件） | mkfs（基线 `memory_pool_after_mkfs`，`:458` 起的 `prepare`） | 取号 `2`、暖机一 `2,1,2`、暖机二 `2,1,2`、A `24,2,1,2`（`:466`）→ 全量 `FULL_STATES = 16_777_260`（`:446`） |
| P2 第二条流 `crash_enumeration_fixed_script_stream.rs:797` | mkfs → 取号 → 暖机 × 2 → A → B（覆盖写 txg 4）→ 进程重开可写挂载（取号 2、写行 txg 5、暖机 txg 6、7）→ C（txg 8）→ D（挂着时向前回退到 A，txg 9）→ 覆盖写 × 5（txg 10–14）→ 抬 F 到 11（先写系统配置、空发布 txg 15、16）→ E（txg 17）→ 正常卸载（先写 F = 17、空发布 txg 18、19）（`Script` 文档 `:73`–`:83`，`prepare` `:218`–`:384`） | mkfs | 78 段 477 写（`:439`–`:442`、`:448`）→ `FULL_STATES_THROUGH_THE_UNMOUNT = 1_662_648_564`（`:100`） |
| P3 并行线一 `crash_enumeration_multi_record_publish_stream.rs:365` | mkfs → 取号 → 暖机 × 2 → A → B（顺序写两个数据单元、两条记录）→ C（顺序写三个数据单元、三条记录）（文件头 `:3`–`:4`） | mkfs | 19 段 `[2,2,1,2,2,1,2,24,2,1,2,28,4,1,2,30,6,1,2]`（`:185`）→ 1358954634（文件头 `:21`；现算相符） |
| P4 树分裂七条 `crash_enumeration_tree_split_streams.rs:415` | mkfs → 取号 → 暖机 × 2 → A（按流给的节点容量发）→ 被录的那一次（五条空发布、根降高那条是建一个 inode，`:74`–`:81` 与 `plan` 的注释） | mkfs 到 A（含 A 的轮换）整段施加成起点（文件头 `:3`–`:4`；`common_tree_split/mod.rs:401` `memory_pool_before_the_current_version`） | 每条 `[2u, 2, 1, 2]`（`:307`–`:308`），u = 10/11/13/9/10/10/11；每条 2^(2u) − 1 + 3 + 1 + 8 + 1，七条合计 78905428（`:408`–`:410`，现算相符） |
| P5 位置寻址四条 `crash_enumeration_position_addressed_trees.rs:480` | 见 1.2 表 P5 那几行 | 被录那一次之前整段施加 | 单元写段全展开（`:482` 传恒真的展开判据）；用例不钉状态数，只断言等于闭式（`:449` 那条断言） |
| P6 会话推的抬 F `crash_enumeration_floor_raise_pushed_by_admission.rs:41` | 384 槽单元区的小盘：mkfs → 第一个文件 → 崩了再挂 → 覆盖写 × 61 → 第 62 次准入拒 → 会话推一串抬 F（先写系统配置、几次带新 F 的空发布） | 抬 F 之前整段（`base = pool.image()`） | 那一串全展开，状态数不钉、断言 ≤ `1_000_000`（`STATES_AT_MOST`，`:43`）；每个状态另起一次可写挂载跑池级 checker |
| P7 σ 全量 `record_checker_judges_absence_by_the_persisted_set.rs:719` | mkfs → … → A → B → `UOOUOMSU`（卸载重挂、覆盖写 × 2、卸载重挂、覆盖写、进程重开重挂、小文件、卸载重挂；`:77`–`:86`） | σ 之前全部 | 只有 σ：txg 25 那次发布的 16 个单元写，2^16 = 65536（`:731`–`:733`）；只跑一遍看 journal 的恢复 + 记录核对器 |
| P8 崩溃注入快档 `crash_injection_campaign.rs:135` | 随机历史 24 段 × 每段 24 步（`:41`–`:42`），操作八种见 1.3 | 历史本身整段跑；抽中的崩溃点之前整段施加（`crash_injection.rs:800`–`:802`） | 不枚举，每段抽 4 个崩溃点（`:43`–`:45`）；每个崩溃点另做可写挂载 + 一次发布、二次崩溃三处（`crash_injection.rs:15`–`:18`） |

### 1.2 不登记全量、归快档或普通 `cargo test` 的崩溃状态用例

| 用例（文件:行） | 骑在哪条操作序列上 | 枚举了哪一截 → 状态数 | 与全量的关系 |
|---|---|---|---|
| P1 快档 `crash_enumeration_new_pool_file_creation_stream.rs:590` | 同 P1 | 甲二：A 的 24 单元写只取全不落 / 全落 → 46（`:450`） | P1 全量的子集 |
| P1 阳性对照 `…new_pool_file_creation_stream.rs:662`、`:801` | 同 P1 | 手摆状态（根在单元缺、记录在单元缺） | 手摆 |
| 去掉根槽前屏障 `…new_pool_file_creation_stream.rs:751` | P1 的流摘掉「记录 → 根槽」两道屏障 | 段 `[2,2,1,2,2,1,2,24,2,1,2]` 变 `[…,24,3,2]`，只展开 ≤ 3 写的段 | 变异流（写序改了），不重复 |
| 只做过 mkfs 的池 `crash_enumeration_writable_mount_of_a_formatted_pool.rs:148` | mkfs → 进程重开可写挂载（取号 1、零单元写行 txg 1、暖机 txg 2）→ A | 24 写段不展开 → `1 + 4 × 8 + 3 × 3 + 3` = 45（`:190`） | 用例钉住基线、写表连同内容、段序列与 P1 逐项相同（`:209`–`:237`）：是 P1 状态的子集 |
| 发现日志 / 续跑 / 双机分片三份（`crash_enumeration_findings_log_…rs:507`、`crash_enumeration_resumes_…rs:151` 等六条、`crash_enumeration_sharded_…rs:346` 等十条） | 同 P1 | 甲二 46 个状态，各跑多趟（线程数、切法、分片、续跑） | 枚举器本身的检查，重复 P1 快档状态是装置需要 |
| 按设备记屏障与撕裂 `crash_segments_per_device_and_torn_in_place_overwrites.rs:380`、`:613`、`:668` | 合成的小流；P1 的流吞盘 1 一道屏障；P1 只展开最后一段 | 最后一段 3² − 1 + 1 = 9（`:762`）等 | 9 个是 P1 的子集；其余是变异流或合成流 |
| journal 提示 `crash_image_journal_hint_matches_the_full_ring_scan.rs:89` | 同 P1 | 12 个段边界 + 只落一份记录的状态 | P1 的子集，判的是另一件事（提示与全环扫描等价） |
| 取号屏障 `crash_enumeration_acquisition_barrier.rs:49` | mkfs → … → A → B → 进程重开（取号 2 → 写行 → 暖机）；段序列是 P2 发 C 之前那 28 段（`:231`–`:236`） | 只展开取号那一段与写行的单元段 → `1 + (3² − 1) + 262143` = 262152（`:223`） | **与 P2 全量逐状态重复**：文件头 `:16` 原话「全量（门禁 54 号 `--full`）已经罩着；这一条不多罩崩溃状态，多的是在平时的 `cargo test` 里展开它们」 |
| P2 快档 `crash_enumeration_fixed_script_stream.rs:661`、八线程对拍 `:728` | 同 P2 | 甲二 278（`:114`），八线程那条跑两趟 | P2 全量的子集 |
| 到 C、到 D 的段序列 `…fixed_script_stream.rs:600`、`:631` | P2 截到 C / D | 不枚举，只核段序列与闭式 | — |
| 靶向阳性对照 `…fixed_script_stream.rs:878` | 到 B 为止 | 手摆四个形状 | 手摆 |
| 残留记录 `…fixed_script_stream.rs:1177` | 基镜像预置一条残留记录（另开一池造的种子）→ … → B → 重开挂载（写行 txg 6、暖机 txg 7）→ C | B 的根槽之后、写数 < 10 的段（`:1208`–`:1210`）→ 53（`:1277`） | 起点镜像不同，不与 P2 重复 |
| 陈旧 tail `…fixed_script_stream.rs:1372` | 到 E 之后再覆盖写两次（txg 18 落 50178、txg 19 复用 A 的 50180），每次系统配置槽写的 tail 改成 2 | txg 19 单元段之后、< 10 写的段（`:1403`–`:1404`）→ 13（`:1470`） | 写的字节改过，不重复；txg 18、19 这两次覆盖写在没改 tail 的流上**没有任何用例枚举** |
| 后写没落的复用 `…fixed_script_stream.rs:1557` | 同上（不改 tail） | 手摆一个状态 | 手摆 |
| P3 快档 `crash_enumeration_multi_record_publish_stream.rs:292` | 同 P3 | 单元段不展开 → `1 + 6 × 8 + 3 × 3 + 5 + 15 + 63` = 141（`:337`） | P3 全量的子集 |
| P4 快档 `crash_enumeration_tree_split_streams.rs:397` | 同 P4 | 单元段不展开，每条 `1 + 3 + 1 + 8` = 13（`:402`），七条 91 | P4 全量的子集 |
| P5 快档 `crash_enumeration_position_addressed_trees.rs:469` | 五条都跑（`ALL`，`:110`） | 写数 < 10 的段 | 第三条（145 个数据单元）只有这一档：单元段约 300 写、记录段 290 写，全量枚举不出来（`:476`–`:477`） |
| 记录跨条 L8 `crash_enumeration_record_spill_over_stream.rs:347` | mkfs → 取号 → 暖机 × 2 → A 整段施加 → 建 N 个 inode（68 个单元、两条记录，文件头 `:7`–`:9`） | 记录段、根槽、轮换 → 25（文件头 `:15`）；单元段 136 写不展开 | 独有 |
| 回退 D 的每个前缀 `crash_points_of_the_rollback_publish.rs:136` | mkfs → A → B → 重开 → C → D（搭建函数抄自 harness 档，文件头 `:4`–`:5`） | D 的录制流每个前缀（前缀 = 按发出次序的子集） | 推的：每个前缀状态都在 P2 D 那几段的枚举域里（同一条脚本；两边字节相同没有用例核） |
| 树 ID 水位孤儿 `crash_points_tree_identifier_watermark_orphans.rs:259` | mkfs → 取号 → 暖机 → A 的每个前缀 → 可写挂载 → 六种动作（`:130`–`:137`） | A 那次发布的每个前缀 × 六种挂载后动作 | 前缀状态是 P1 的子集；挂载后动作独有 |
| C577 见证 `a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs:241`、`:340`、`:435` | 每条发布路径各一段历史 | 只在没放行那一段的系统配置槽写上取子集（文件头 `:10`–`:13`） | 判的是另一件事（见证值） |
| C513 / C556 / C561 其余格 | `record_checker_reuse_exemption_legality.rs:31`、`rollback_floor_…rs:994`、σ 那份的 `:404`–`:551` | 手摆状态；σ 的 `:506` 取 16 个 | 手摆或 P7 子集 |

### 1.3 崩溃注入的随机历史（`crates/singlefs-checker-tier/src/crash_injection.rs`）

| 事实 | 出处 |
|---|---|
| 操作只有八种：第一个文件、覆盖写、零单元发布、关会话再可写挂载、挂着时回退、抬 F、冷启动恢复、崩溃恢复抛弃最新根 | `crates/singlefs-harness/src/history.rs:375`–`:384` |
| 覆盖写的内容恒在一个数据单元以内（`ContentLength` 六种：空、一个单元以内、差一字节、正好、多一字节、整宽；后两种装不下） | `history.rs:247`–`:259` |
| 没有的操作：顺序写多个数据单元、建 inode、树分裂（产品容量下）、正常卸载、删除 | 同上八种；`grep -n 'publish_sequential_write\|publish_new_inodes\|unmount(' crates/singlefs-harness/src/history.rs` 零命中（这一次现跑） |
| 起点两种：只做过 mkfs、mkfs 同一进程里到 A；起点那一段也摆崩溃点 | `history.rs:238`–`:243`；`crash_injection.rs:11`–`:12` |
| 抽法：快档每段历史 4 个崩溃点，按段号排好、基线只往前叠；撕裂并进「没持久」 | `crash_injection_campaign.rs:43`–`:45`；`crash_injection.rs:5`、`:800`–`:815` |
| 写死历史那条：mkfs → 可写挂载 → 第一个文件 → 覆盖写 → 零单元发布 → 可写挂载 → 覆盖写，≤ 4 写的段全部真子集、更长的段各抽 8 个，崩溃点 ≥ 95 | `crash_injection_campaign.rs:372`–`:417` |

## 二　操作树（问题 2）

### 2.1 节点表

节点 = 一步用户级操作（一次发布、挂载里的一步、一串抬 F / 卸载）；从同一个起点、同一种操作出发的算同一条边。
「全量枚举」列只数门禁 54 号 `--full` 那一批（1.1）；快档与手摆另列。状态数按写它的操作归（口径见开头），数自现算，与用例钉的合计相符（第 2.2 节）。
内容不同、形状相同的两条边（同一种操作、换了内容种子或节点容量）单列成兄弟，另在「同形」列标出。

| 节点 | 父 | 操作 | 经过它的用例 | 全量枚举次数 × 每次状态数 | 快档 / 手摆 | 同形 |
|---|---|---|---|---|---|---|
| T0 | — | mkfs（两块 4 GiB） | 全部（P6 另一种几何，另起 T56） | 0（恒为基线） | 崩溃注入起点那一段抽样（`crash_injection.rs:11`–`:12`） | |
| T1 | T0 | 取号 1（同一进程） | P1、P2、P3、P4、P5 一至四条、L8、σ、水位、残留、取号屏障、陈旧 tail、回退前缀、崩溃注入（起点二） | 3 × 8 | P1/P2/P3 快档 | 另一条边 T1a「进程重开可写挂载（取号 1、零单元写行 txg 1、暖机 txg 2）」字节与 T1–T3 逐项相同（`crash_enumeration_writable_mount_of_a_formatted_pool.rs:209`–`:237`） |
| T2 | T1 | 暖机一 txg 1 | 同 T1 | 3 × 12 | 同上 | |
| T3 | T2 | 暖机二 txg 2 | 同 T1 | 3 × 12 | 同上 | |
| T4 | T3 | A 第一个文件（产品容量） | 同 T1 | 3 × 16777227（+ P1 末尾全部持久 1，它就是 P2、P3 下一段的空子集） | 水位：A 的每个前缀 × 六种挂载后动作 | T34–T37 是换了节点容量的 A |
| T5 | T4 | B 覆盖写 txg 4（释放 A 的第一个数据单元） | P2、σ、残留、取号屏障、陈旧 tail、回退前缀 | 1 × 16777227 | P2 快档 | 与 T32 同为「A 之后第一次写」，操作不同 |
| T6 | T5 | 进程重开：取号 2 | P2、取号屏障、陈旧 tail、回退前缀 | 1 × 8 | **取号屏障每次 `cargo test` 再全展开一遍**（8） | |
| T7 | T6 | 写行 txg 5（18 个单元写） | 同 T6 | 1 × 262155 | **取号屏障再展开单元段** 262143 + 1 | |
| T8 | T7 | 暖机 txg 6（16） | P2、陈旧 tail、回退前缀 | 1 × 65547 | | T9 同形 |
| T9 | T8 | 暖机 txg 7（16） | 同 T8 | 1 × 65547 | | |
| T10 | T9 | C 覆盖写 txg 8（24） | 同 T8 | 1 × 16777227 | | |
| T11 | T10 | D 挂着时向前回退到 A txg 9（20） | 同 T8 | 1 × 1048587 | 回退前缀：D 的每个前缀（`crash_points_of_the_rollback_publish.rs:136`） | |
| T12 | T11 | 覆盖写 txg 10（28，释放 D 复活的 A 的数据单元） | P2、陈旧 tail | 1 × 268435467 | | |
| T13–T16 | 依次 | 覆盖写 txg 11–14（28 × 4） | 同 T12 | 各 1 × 268435467 | | 与 T12 同形，见第三节 |
| T17 | T16 | 抬 F 到 11（先写系统配置 2、空发布 txg 15、16 各 16） | 同 T12 | 1 × 131102（8 + 65547 + 65547） | | 与 T19 同形 |
| T18 | T17 | E 覆盖写 txg 17（28，数据单元落回 50176） | 同 T12 | 1 × 268435467 | | |
| T19 | T18 | 正常卸载（先写 F = 17、空发布 txg 18、19） | P2 | 1 × 131102（+ 全部持久 1） | | |
| T20 | T18 | 覆盖写 txg 18（落 50178） | 陈旧 tail、后写没落 | **0** | 陈旧 tail 不展开它的任何一段（`:1403`–`:1404` 只展开 txg 19 单元段之后） | |
| T21 | T20 | 覆盖写 txg 19（复用 A 的 50180） | 同 T20 | **0** | 陈旧 tail（改了 tail 的字节）展开记录、根槽、轮换共 13；单元段（复用本身）从没展开；后写没落手摆 1 | |
| T22–T29 | T5 起 | σ 那条历史 `UOOUOMSU` 八步（`record_checker_judges_absence_by_the_persisted_set.rs:77`–`:86`） | σ 那份 | 只有 txg 25 那次发布的单元段 1 × 65536；**八步里其余每一截 0** | σ 那份 `:506` 16 个 | 卸载、覆盖写、重开各与 T19、T12、T6–T9 同形 |
| T30–T31 | T5 | 基镜像带残留记录 → 重开挂载（写行 txg 6、暖机 txg 7）→ C | 残留那条 | 0 | 53（`:1277`） | 与 T6–T10 同形、起点镜像不同 |
| T32 | T4 | B：顺序写两个数据单元、两条记录（txg 4） | P3 | 1 × 268435479 | P3 快档 | 与 T45 同形（内容种子不同，见 2.3） |
| T33 | T32 | C：顺序写三个数据单元、三条记录（txg 5） | P3 | 1 × 1073741895（+ 全部持久 1） | P3 快档 | |
| T34 | T3 | A（映射树容量 9, 3） | 树分裂叶分裂、根降高 | **0** | | T4 |
| T35 | T3 | A（映射树 9, 2） | 两层连着分裂 | **0** | | T4 |
| T36 | T3 | A（映射树 5, 3） | 摘空节点 | **0** | | T4 |
| T37 | T3 | A（记账树 14, 3） | 记账叶分裂 | **0** | | T4 |
| T38–T44 | T4 / T34–T37 | 七条被录发布（中央映射根分裂、叶分裂、两层分裂、摘空、根降高、记账根分裂、记账叶分裂） | P4 | 各 1 × 2^(2u) − 1 + 13（1048588 / 4194316 / 67108876 / 262156 / 1048588 / 1048588 / 4194316） | 各 13 | |
| T45 | T4 | 顺序写成两个数据单元（位置寻址第一条） | P5 一、二条 | 1 × 推的 268435480（单元数用例不钉；按 T32 同形 14 个单元、两条记录推） | 快档 | T32 |
| T46 | T45 | 顺序写回一个数据单元 | P5 第二条 | 1 × 推的 16777228（按 12 个单元推，没钉） | 快档 | |
| T47–T48 | T4 | 顺序写 144 → 145 个数据单元 | P5 第三条 | **0**（全量不收它，`:476`–`:477`） | 快档只展开 < 10 写的段 | |
| T49 | T4 | 覆盖写 k − 1 次（k ≤ 20，`:255`–`:262`） | P5 第四条 | **0**（基线） | | T12 同形 |
| T50 | T49 | 第 k 次覆盖写（两块盘各重写叶 61 与叶 62） | P5 第四条 | 1 × 推的 2^32 − 1 + 13（按 16 个单元推；用例注释 `:476` 写「约 2 × 14 写，状态数上亿级」，两者对不上，没钉数） | 快档 | |
| T51–T53 | T0 | 可写挂载（实例 1、零单元写行）→ 取号之后崩溃 × 369 → 可写挂载写 370 行（`:296`–`:297`） | P5 第五条 | T51、T52 **0**；T53 1 × 没钉数 | 快档 | |
| T54 | T4 | 建 N 个 inode（68 个单元、两条记录） | L8 | **0**（单元段 2^136 − 1 枚举不了） | 25 | |
| T55 | T4 的每个前缀 | 可写挂载 → 六种动作 | 水位 | 不是崩溃状态枚举 | 前缀数 × 6 | |
| T56–T59 | — | 小盘 mkfs → 第一个文件 → 崩了再挂 → 覆盖写 × 61 → 第 62 次准入拒 | P6 | **0**（基线） | | |
| T60 | T59 | 会话推的一串抬 F | P6 | 1 × 没钉数（≤ 10⁶） | 每个状态另起一次可写挂载 | T17 同形、几何不同 |
| 崩溃注入 | T0 或 T4 | 八种操作的随机路径 | P8 | 不枚举 | 每段 4 个抽样点 × 24 段 | — |

### 2.2 合计（现算）

| 量 | 数 | 怎么数 |
|---|---|---|
| 节点数 | 61（T0–T60），另有与 T1–T3 字节相同的一条边 T1a | 2.1 表逐行数：22 + 8 + 2 + 2 + 4 + 7 + 2 + 2 + 2 + 3 + 1 + 1 + 4 + 1 |
| 全量里钉了数的状态合计 | 3117351422 | P1 16777260 + P2 1662648564 + P3 1358954634 + P4 78905428 + P7 65536；P5、P6 用例不钉数，不在内 |
| 全量里被枚举两次及以上的节点 | T1–T4（各 3 次） | 2.1 表「全量枚举次数」列 |
| 重复的状态数 | 2 × 16777260 = 33554520，占钉了数的合计 1.076% | 与事实表甲 2.2 的 33554520 相同（甲的分母是三条整流 3038380458，1.104%） |
| 快档里重复全量的 | 取号屏障 262152（T6、T7，每次 `cargo test` 跑一遍，`crash_enumeration_acquisition_barrier.rs:223`）；回退前缀（T11 的前缀，推的：全在 P2 里）；P1 / P2 / P3 / P4 快档本来就是各自全量的子集 | 1.2 表 |
| 在任何用例里都只当基线、全量一次都没枚举的节点 | 27 个：T0、T20、T21、T22–T29（σ 那次发布的单元段除外）、T30、T31、T34–T37、T47、T48、T49、T51、T52、T54、T56–T59 | 2.1 表「全量枚举次数」为 0 的行；其中 T21、T30–T31、T48、T54 在快档里展开过小段（13、53、小段、25） |
| 同一种操作、同一类条件在同一条路径上重复出现、各自全量枚举的 | T12–T16（覆盖写 × 5，5 × 268435467 = 1342177335）、T8 与 T9（暖机 × 2）、T17 与 T19（先写系统配置再推两次空发布） | 第三节逐个判 |

### 2.3 同形兄弟的字节是否相同

| 兄弟 | 相同 | 不同 | 出处 |
|---|---|---|---|
| T1–T3 与 T1a | 基线、写表连同内容、段序列逐项相同（用例钉） | — | `crash_enumeration_writable_mount_of_a_formatted_pool.rs:209`–`:237` |
| P1、P2、P3 的 T1–T4 | 推的：同一个 `build_pool`、固定内容与写时 | 没有用例逐字节核 | 事实表甲 2.1 |
| T32（P3 的 B）与 T45（位置寻址第一条） | 同一个入口 `publish_sequential_write`、同写时 `FIXED_WRITE_TIME_SECONDS + 60` | 内容式子不同：`(index * 11 + seed * 3 + 7) % 251`（`crash_enumeration_multi_record_publish_stream.rs:54`–`:58`）与 `(index * 13 + seed * 5 + 3) % 251`（`crash_enumeration_position_addressed_trees.rs:62`–`:67`）；P3 用实例号写死 1，位置寻址取现行那一版的实例（同为 1） | 两个文件的 `sequential_write` |
| T4 与 T38、T43 的父（产品容量的 A） | `capacities(BIG_ACCOUNTING, BIG_CENTRAL_MAPPING)` 两个都等于格式那一档时交回 `FromTheNodeFormat`（`common_tree_split/mod.rs:35`–`:36`），推的：与 `build_pool` 的 A 同字节 | `TreeSplitPool` 自己一份搭建（`common_tree_split/mod.rs:247`），没有用例核它与 `build_pool` 逐字节相同 | |

## 三　同一种模块在第二条流里重复（问题 3）

段数组 `crash_enumeration_fixed_script_stream.rs:439`–`:442` 现数（`calc.py`）：28 写段 6 个、24 写段 **3 个**（派发提示写 4 个，现数是 3 个：A、B、C）、20 写段 1、18 写段 1、16 写段 6、2 写段 42（系统配置槽 23 + journal 记录 19）、1 写段 19（根槽）。单元写段合计 1662648303 个状态，占全量 1662648564 的 99.99998%（现算）。
根槽落哪块盘：区域 = txg mod 3（`crates/singlefs-core/src/root_ring.rs:114`），区域到盘 `[0, 1, 0]`（`crates/singlefs-format/src/lib.rs:265` `ROOT_RING_REGION_DEVICES`）；用例注释自己写的落盘与这个算法相符（`:235`、`:306`、`:337`）。

### 3.1 单元写段逐个

| 段 | 节点 | 这一次要测的（原文） | 出处 | txg / 根槽落盘 | 判 |
|---|---|---|---|---|---|
| 24 | T4 A | 新池新建文件；第一个文件版本释放 mkfs 的树表单元 | `.claude/kb/milestone/02-second-txn.md:609`（「第一个文件版本释放 mkfs 树表单元那一条」） | 3 / 盘 0 | 独有 |
| 24 | T5 B | 「B（覆盖写，释放第一个数据单元）」 | `02-second-txn.md:30` | 4 / 盘 1 | 独有：实例 1 里的第一次覆盖写 |
| 18 | T7 写行 | 重开之后的写行发布（实例表写行） | `:235` 注释、`:391` 注释「写行发布 18」 | 5 / 盘 0 | 独有 |
| 16 | T8、T9 暖机 | 「暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1）」 | `:235` | 6 / 盘 0；7 / 盘 1 | 两次之间差的是根槽落哪块盘（推的：第二次让新实例在两块盘上都有根） |
| 24 | T10 C | 「B 之后进程重开、可写挂载（取号、写行、暖机两次），再发布 C」 | `Script` 文档 `:73` | 8 / 盘 0 | 独有：新实例里的第一次覆盖写 |
| 20 | T11 D | 「C 之后在同一个会话里挂着的时候回退到 A 的根 (1, 3)——一次向前发布 D（txg 9，实例仍是 2…），不取号、不写行、不暖机」 | `:75`–`:76` | 9 / 盘 0 | 独有 |
| 28 | T12 覆盖写 txg 10 | 「回退之后再覆盖写五次（txg 10–14，第一次释放 D 复活的 A 的数据单元、释放代 10）」 | `:78` | 10 / 盘 1 | **独有**：释放 D 复活的 A 的数据单元；抬 F 到 11 要回收的就是这一格 |
| 28 | T13–T16 覆盖写 txg 11–14 | 同一句；它们的角色是「接着推几次非空发布把「第 4 新的非空持久有效根」攒到那个 txg 之上」，抬 F 的上限 `rollback_floor_ceiling` = min(每块盘上最新的有效根, 第 4 新的非空有效根) | `02-second-txn.md:198`、`:196` | 11 / 盘 0；12 / 盘 0；13 / 盘 1；14 / 盘 0 | **推的：四次测的条件相同，只换了 txg、根槽区域与落点**：都是回退之后、抬 F 之前、释放上一次覆盖写的数据单元的普通覆盖写；用例没有一句注释给它们各自单独的条件。它们作为抬 F 的前置是必需的（上限要到 11），作为崩溃目标没有各自的理由 |
| 16 | T17 抬 F 的两次空发布 txg 15、16 | 「再连推带新 F 的空发布直到每块盘上都有一条（生效）」；`assert_eq!(raised.publishes.len(), 2, "txg 15 落盘 0、txg 16 落盘 1")` | `02-second-txn.md:196`；`:306` | 15 / 盘 0；16 / 盘 1 | 两次不同：崩在第二次之前，新 F 只在盘 0 的根上生效（推的） |
| 28 | T18 E | 「E 的数据单元落回最低的可再分配偶数槽对：mkfs 实例表那两槽（释放代 5，抬 F 到 11 回收了）」 | `:316`–`:320` | 17 / 盘 0 | **独有**：抬 F 回收之后的复用 |
| 16 | T19 卸载的两次空发布 txg 18、19 | 「卸载推两次空发布：txg 18 落盘 0、txg 19 落盘 1（17 除以 3 余 2）」；卸载「先把 F = 17 写进每块盘的系统配置、过一道屏障，再推带卸载记号的空发布直到每块盘上都有一条」 | `:337`；`:79`–`:80` | 18 / 盘 0；19 / 盘 1 | 与 T17 同形，差卸载记号与 F 值 |

### 3.2 小段逐类

| 段类 | 次数 | 每次状态 | 条件差在哪 | 判 |
|---|---|---|---|---|
| 系统配置槽 2 写（原地覆写三态） | 23：取号 × 2、每次发布末尾轮换 × 19、抬 F 与卸载先写 F × 2 | 3² − 1 = 8 | 写进的实例代号、tail、F 各不同；哪一槽较旧随世代号轮换 | 模块相同、内容不同；共 184 个状态 |
| journal 记录 2 写 | 19 | 3 | 记录槽位置、点名的单元 | 同上；共 57 |
| 根槽 1 写（FUA） | 19 | 1 | 区域与槽 | 共 19 |

### 3.3 单元写段里的状态看得见什么

| 事实 | 出处 |
|---|---|
| 单元段在记录段之前、中间隔一道屏障：单元段里的任何状态上，这次发布的记录与根一份都没落，恢复见到的是上一版加几份没人点名的单元 | `crash_enumeration_record_spill_over_stream.rs` 文件头 `:16`–`:17`（「单元段里的状态上这次发布的记录一份都没落……那一格已有的流都罩着」） |
| 抬 F 之前没有回收：「第一版环里最旧有效根恒 0、floor 就是 F_生效」，回收只在 `reclaim_released_up_to(floor)` 里做 | `02-second-txn.md:196` |
| 推的：所以 T5、T7–T16 的单元写都落在从没写过的槽上，单元段的 2^n − 1 个子集留在盘上的只是「哪些孤儿单元在」这一维；T17 之后（txg 15 起）才可能落在回收过的槽上，E 那一格用例钉了 50176 | 由上两行推；T17、T19 的单元落点用例没钉 |
| 池级 checker 会看孤儿：I-7.8 的扫描读盘上留下的码 2 节点（孤儿）头里的树 ID | `crash_points_tree_identifier_watermark_orphans.rs` 文件头 `:8`–`:12` |
| E161 可行性档在段头（最小的 4096 个序号）上量过：`k4_walk distinct_keys=1`、`distinct_outcomes=1` | `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out:43`、`:142` |
| ⇒ 同一单元段里的子集之间「结论相同」只在段头 4096 个状态上量过（26 写段的 4096 / (2^26 − 1) ≈ 0.006%，现算）；孤儿集合不同会不会让 checker 判得不同，没有用例在整段上核 | 由上两行 |

## 四　固定脚本的线性串接：哪几步真依赖前一步（问题 4）

今天第二条流的每一段都骑在前面全部段全持久之上（`crash.rs:2177`–`:2179`）；P2 是一条线，没有分支。逐对相邻步判：

| 后一步 ← 前一步 | 真依赖？ | 依据（原文） |
|---|---|---|
| A ← 取号、暖机 × 2 | 真 | A 用暖机的最后一代根与记录（`common_tree_split/mod.rs:247` 起 `with_the_first_file_version_under` 的 `warmed_up.roots.last()`） |
| B ← A | 真 | 「B（覆盖写，释放第一个数据单元）」（`02-second-txn.md:30`） |
| 重开挂载（取号 2、写行、暖机 × 2）← B | **只是排在一起** | 挂载只要求现行那一版带文件（`:240`–`:243` `into_file_version().expect("B 之后重开，现行那一版带文件")`）；A 之后就满足。挂在 A 下面是另一格（写行写的是 (1, 3) 那一行），今天没有 |
| C ← 重开挂载 | 真 | C 是新实例里的覆盖写（`Script` 文档 `:73`） |
| D ← C | 部分：D 要的是「实例 2 的会话 + A 在候选集里」（`:75`–`:76`）；C 给的是「被抛弃的一版是本实例写的」 | D 挂在 T9（不经 C）也做得成，抛弃的就是 B 的内容那一版（推的，没有用例） |
| 覆盖写 txg 10 ← D | 真 | 「第一次释放 D 复活的 A 的数据单元、释放代 10」（`:78`） |
| 覆盖写 txg 11–14 ← txg 10 | 对自己的崩溃状态**不依赖**；对抬 F **是前置** | 上限 = min(每块盘最新有效根, 第 4 新的非空有效根)（`02-second-txn.md:196`），F = 11 要四条非空根 ≥ 11 |
| 抬 F ← 覆盖写 × 5 | 真 | 同上；「抬 F 排在回退之后：F 抬过 A 的 txg 之后，A 就不在回退候选集里了」（`02-second-txn.md:31`–`:32`；`Script` 文档 `:81`–`:82`） |
| E ← 抬 F | 真 | E 落回 50176 要「抬 F 到 11 回收了」（`:319`）；释放代 5 那一次是写行（T7） |
| 正常卸载 ← E | **只是排在一起** | 原文只要求卸载在回退之后：「卸载放在最后：它把 F 抬到现行 txg，排在回退之前的话回退候选集里就没有 A 了」（`:81`–`:82`）；对 E 没有依赖 |

**能从一个共同节点分出去做兄弟的**（推的，按上表）：

| 共同节点 | 今天在不同文件里各自一条线的兄弟 | 今天没有、按上表可以挂上去的兄弟 |
|---|---|---|
| T4（A 之后） | B（P2）、顺序写两单元（P3、P5 第一条两份）、树分裂两条（产品容量）、建 N 个 inode（L8）、位置寻址第四条的覆盖写链、σ 那条历史（经 B） | 重开挂载（不经 B）、正常卸载（实例 1 上） |
| T9（重开挂载之后） | C → D → … | D（不经 C）、正常卸载（实例 2、回退之前） |
| T17（抬 F 之后） | E → 卸载 | 卸载（不经 E）；E 之后的 txg 18、19 复用（T20、T21，今天只有陈旧 tail 的变异流展开了小段） |
| T16（txg 14 之后） | 抬 F | 正常卸载（卸载本身就抬 F 到现行 txg，与会话推的抬 F 是两条路径） |

## 五　今天任何用例都没枚举的操作组合（问题 5）

「枚举」按开头的口径；崩溃注入抽样到的不算枚举，单列一栏。

| # | 组合 | 对照哪一项 | 今天的样子 | 崩溃注入抽得到吗 | 出处 |
|---|---|---|---|---|---|
| Z1 | 删除（unlink、`deleted_inodes`、后台回收）任何一步 | 里程碑三第八项；收口表「不收口 C118」 | `crates/` 里没有删除：`crates/singlefs-core/src/inode_tree.rs:16`「这一版没有删除（`deleted_inodes` 的形态无落点，C118（`deleted_inodes` 树的形态无落点））」 | 否（八种操作里没有） | `02-second-txn.md:401` |
| Z2 | 截断成 0（近满时覆盖写成空内容，第三轮叫「删除类发布」） | `m2-safety-r3-main-verification.md:29`「**没跑**：一条流 16,777,223 个状态」、`:61` | 层 0 没有任何一条流写空内容（P2 的五次覆盖写与 E 都是 `later_content(seed)`，长 3000 + seed，`:133`–`:137`） | 抽得到（`ContentLength::Empty`，`history.rs:248`） | `m2-safety-r3-opus-output.md:270`–`:272` |
| Z3 | 抬 F 回收之后的复用，除 E 那一次之外：txg 18 落 50178、txg 19 复用 A 的 50180（环里有记录点名它） | 第六项（多次 COW） | 两次的单元段都没展开；txg 19 的记录、根槽、轮换只在改了 tail 的变异流上展开 13 个 | 抽得到（快档比重下覆盖写 52%、抬 F 16%，`history.rs:478`–`:481`） | 1.2 表「陈旧 tail」行 |
| Z4 | 多个数据单元的文件再覆盖写、在新实例 / 回退之后 / 回收之后顺序写 | 第六项 | 多单元只出现在 P3（1 → 2 → 3，实例 1）、位置寻址（2 → 1、144 → 145、实例 1）；随机历史只有一个单元以内的内容 | 否 | 1.3 表 |
| Z5 | 树分裂之后的任何一步：分裂之后再发布、回退到分裂之前那一版、分裂之后回收复用、新实例里分裂 | 第七项 | P4 每条只录分裂那一次（文件头 `:3`–`:4`），之前全施加、之后没有 | 否（产品容量下节点不分裂，推的） | `crash_enumeration_tree_split_streams.rs:3`–`:5` |
| Z6 | 根环回卷 | 第七项 | 根环 3 区 × 每区 8 槽（`lib.rs:238`、`:256`）；P2 写 19 条根，推的：不回卷；σ 那条历史过了 txg 25，推的：回卷过，但只枚举了 txg 25 的单元段 | 抽得到（24 步的历史，推的） | |
| Z7 | journal 环回卷 | 第七项 | 环 805306368 字节（`lib.rs:213`），P2 19 条记录；推的：没有一条流回卷 | 推的：24 步也不回卷 | |
| Z8 | 跨挂载的崩溃点重放（崩了再挂、再崩）：C287、C330、C282、C353 要的形态 | 收口表第 22 行「多次挂载的崩溃点重放装置」挪后 | 层 0 里跨挂载的只有 P2 一次正常重开、σ 那条历史（只一段）；二次崩溃只在崩溃注入里 | 抽得到（第三截的二次崩溃，`crash_injection.rs:15`–`:18`） | `02-second-txn.md:354` |
| Z9 | 三块盘及以上 | 收口表第 14 行 C126（切换预留的最坏量没有口径） | 全部用例两块盘 | 否 | `02-second-txn.md:345` |
| Z10 | 去掉一次 FUA 必须变红（C6） | 里程碑三第五项 | 有的是去屏障：记录 → 根槽（`crash_enumeration_new_pool_file_creation_stream.rs:751`）、盘 1 吞一道（`crash_segments_per_device_and_torn_in_place_overwrites.rs:380`）、零单元发布少一道（崩溃注入写死历史 `:368`–`:370`）；把 FUA 写换成普通写的用例 0 处（`grep -n 'FUA' crates/singlefs-checker-tier/tests/*.rs` 里没有一处带「摘 / 去掉 / 吞 / 少」） | — | `02-second-txn.md:397` |
| Z11 | 重开挂载挂在 A 之后（不经 B）、回退不经 C、正常卸载挂在实例 1 上或回退之前 | 第一项（分支） | 第四节「能分出去的兄弟」右栏，今天都没有 | 抽得到（关会话再挂、回退都在八种里） | |
| Z12 | 崩溃恢复抛弃最新根、冷启动恢复之后的下一次发布 | 第七项 | 层 0 没有这两种操作 | 抽得到（C554 乙之后抛弃根那一步被拒，`crash_injection_campaign.rs:557`–`:559`） | |
| Z13 | 建 N 个 inode 的单元段（136 写）、145 个数据单元那一次的单元段（约 300 写）与记录段（290 写） | 第七项 | 只展开小段（25；< 10 写的段） | 否 | 1.2 表 |
| Z14 | mkfs 自己那几次写 | — | 层 0 全部把 mkfs 当基线 | 抽得到（起点那一段也摆） | `crash_injection.rs:11`–`:12` |
| Z15 | 真设备上的任何崩溃状态 | 第一项「真设备上的崩溃注入（补最终判据）」 | 55 号只有真实负载与设备侧录制，没有崩溃注入 | — | 事实表甲 1.1 P11 |

## 六　判断：用例设计要不要重做（问题 6，推论，只给候选与依据）

### 6.1 先摆三个数

| 数 | 值 | 说明什么（推的） |
|---|---|---|
| 结构性重复（同一节点在两条以上全量里各枚举一遍） | 33554520，占钉了数的全量合计 1.076%（2.2） | 按操作树去重，状态数只省这么多 |
| 同一条线上同一种模块重复、条件只差 txg 与落点的 | T13–T16 四次覆盖写，4 × 268435467 = 1073741868，占 P2 的 64.6%（3.1） | 这一类不是「前缀共享」，是用例把前置步骤也当成崩溃目标 |
| 单元写段里的状态 | P2 的 99.99998%（3 节） | 省时的大头在段内等价（第二项的剪枝），不在树的形状；段内等价今天只在段头 4096 个状态上量过（3.3） |
| 全量一次都没枚举的节点 | 61 个里 27 个（2.2），另有第五节 Z1–Z15 | 用户说的「容易漏，也容易测不全」对应的是这一栏，不是重复那一栏 |

### 6.2 候选形态

| | α 一棵操作树，每个节点只枚举一次 | β 模块 × 条件格，路径只当搭建 | γ 用例结构不动，段结果按内容寻址缓存 |
|---|---|---|---|
| 节点怎么划 | 一步用户级操作一个节点（2.1 的划法）；起点镜像 = 父节点全部持久之后的那一版，物化一次（崩溃注入已经这样做：基线随段前移，`crash_injection.rs:800`–`:802`）；节点的空子集就是父节点的全部持久，只算一次 | 格 = 操作种类 × 条件轴（实例：首个 / 重开之后；回收前 / 回收后；根槽区域 0 / 1 / 2；树形；单元数；复用的槽有没有被环里的记录点名……）；每格从一条最短的搭建路径造起点，只枚举那一格 | 键 =（起点镜像指纹，这一段的写连同内容，版本表与被判的根，代码与 checker 指纹）；三条整流照旧跑，前缀字节相同的段命中缓存 |
| 三条整流 | T0–T4 共用，P2、P3 是 T4 下的两条枝 | 拆成格；T13–T16 若判同格，只留一个枚举，其余当搭建 | 不动 |
| 树分裂、位置寻址 | 今天已是「起点镜像不枚举、只录那一次」（`crash_enumeration_tree_split_streams.rs:3`–`:4`），直接挂成 T4 或 T34–T37 下的子节点 | 各自是「码 2 树形」「extent 形」条件轴上的格 | 不动 |
| 崩溃注入 | 在同一套操作字母表上随机走路，起点可取任一节点的物化镜像；抽中而树上没有的路径，按判据登记成新节点 | 按格抽样补没被枚举的格 | 不动 |
| 合并掉的状态（现算 / 推的） | 33554520；取号屏障那 262152 若也并进树，快档少一遍（它的用处是让平时 `cargo test` 展开，`crash_enumeration_acquisition_barrier.rs:16`，并不并要另判）；T32 与 T45 若统一内容种子再并一个，推的 268435480 | 在 α 之上，T13–T16：只以区域为轴时 O2（区域 2）、O3（0）、O4（1）各留一个、O5 当搭建，少 268435467（P2 的 16.1%）；不设区域轴则少 3 × 268435467 = 805306401（48.4%） | 33554520，前提是 T1–T4 三条逐字节相同（今天没有用例核，缓存的指纹不中就当场暴露） |
| 补进来的（今天没枚举的） | 按「每个节点枚举一次」，今天只当基线的节点要补：T20、T21 推的各 268435467（与 E 同形 28 写），T34–T37 推的 1879048236（映射树或记账树多一到三个节点：三个 14 单元、一个 15 单元，按 `plan` 的 `shape_before` 推，`crash_enumeration_tree_split_streams.rs:145`–`:207`），σ 那条历史八步推的约 10⁹（四次覆盖写各约 2.68 × 10⁸ 加几次挂载与卸载）；T47、T48、T54 的段 2^136、约 2^300，枚举不了，α 本身补不上 | 第五节列的格逐个成格（Z1 要先有删除的实现）；补多少取决于轴怎么定，没法估 | 0 |
| 与第一项「节点走到哪些代码模块机械登记」 | 天然按节点登记；改了某个模块只重跑走到它的节点及下游（`03-third-txn.md:35` 列的开工前要定项） | 格的定义本身就是手列的条件清单——正是「手列清单量过会漏」那一处（`03-third-txn.md:33` 引 `m2-layer0-scale-r2-main-verification.md` 第二节 M3） | 不涉及 |

### 6.3 逐条对照第三节的「只有它才有的条件」

| 条件（3.1） | α | β | γ |
|---|---|---|---|
| A 释放 mkfs 树表单元（T4） | 留：节点起点与今天 P1 相同 | 留：独成一格 | 留 |
| B 实例 1 第一次覆盖写（T5） | 留 | 留 | 留 |
| 写行、C、D（T7、T10、T11） | 留：起点 = 父节点全持久 = 今天「前面的段全持久」 | 留，前提是轴里有「首个 / 重开之后」「被抛弃的是哪个实例写的」 | 留 |
| txg 10 释放 D 复活的 A 的数据单元（T12） | 留 | 留，前提是轴里有「释放的是被回退复活的单元」；**这一格没有轴就会被并进普通覆盖写，漏** | 留 |
| E 回收之后复用 50176（T18） | 留 | 留，前提是轴里有「回收前 / 回收后」 | 留 |
| 抬 F、卸载各两次空发布「每块盘上都有一条」（T17、T19） | 留 | 留，前提是轴里有「这一串的第几次」 | 留 |
| 暖机两次的根槽落盘 0 / 1（T8、T9） | 留 | 留，前提是轴里有区域 | 留 |
| 今天 P1 的判定口径（被判的根是 A、单版本、带 I-3.10 观察者，`crash_enumeration_new_pool_file_creation_stream.rs:391`、`:402`） | 要改：同一个节点上 P1、P2、P3 的被判根与版本表不同，节点只跑一次就要按路径各出一份计数（事实表甲第七节第二行已知计数口径会不同，判定推的相同） | 同 α | 键里带被判的根与版本表，三条各算，**省不下来**；只有判定（违例与否）可以共用 |
| 记录核对器要整条流的写表与持久集合（`crash.rs:112` `check_records_against` 的入参） | 要改：物化了父镜像之后，节点仍要把路径上全部记录交给核对器（崩溃注入的 `HistoryThenWritableMountRecords` 是现成的形态，`crash_injection.rs:1188`） | 同 α | 不动 |

### 6.4 候选之间的取舍依据（推的，交判决）

- 只看「共享低」这一句：结构性重复只有 1.076%，三种形态省下的状态数都是这一量级；要 1.08% 以外的省，靠的是段内等价（第二项），或 β 那种把同类重复判成同格（要先证等价，受 `.claude/main-agent.md`「禁止」一节第一条约束：不能为省时收窄枚举域）。
- 只看「容易漏」这一句：27 个只当基线的节点与 Z1–Z15 是按路径写用例的直接后果；α 把「哪个节点没枚举」变成树上看得见的空格，β 把它变成格表上的空格但格表靠手列，γ 不回答它。
- α 与 γ 不互斥：α 定结构，γ 是 α 里「父节点结束镜像变了下游要重跑」的判法（键里带起点镜像指纹）。

## 七　什么现象会推翻这份表里的结论

| 结论 | 推翻它的观测 |
|---|---|
| 全量里只有 T1–T4 被枚举两次以上，重复 33554520 | 找到另一对全量用例在同一节点上起点镜像、写表连同内容逐字节相同（例：P5 第一条与 P3 的 B 若内容种子相同）；或 T1–T4 在三条流之间逐字节不同（那时重复是 0，事实表甲第七节第一行的比对） |
| 27 个节点在全量里一次都没枚举 | 在 `crates/singlefs-checker-tier/tests/` 或 `.claude/gate.d/stage-inputs.tsv` 之外找到另一处枚举它们的用例（这一次只搜了 checker 档 `tests/` 与 `crash_injection.rs`，harness 档的 `tests/` 没逐个读） |
| T13–T16 条件相同、只差 txg 与落点 | 在用例、`Script` 文档或 `02-second-txn.md` 里找到给其中某一次单独写的条件；或同一段的某个子集在其中一次上判红、另几次上不红 |
| 抬 F 之前的单元写都落在从没写过的槽上（3.3） | 在 P2 的写表里找到 txg 15 之前有一次单元写的偏移与更早一次写重叠（今天没核，核法：按 `writes` 的 (device, offset) 去重数一遍） |
| 第四节「只是排在一起」的两对（重开挂载 ← B、卸载 ← E） | 用例或条款里找到重开挂载要 B、卸载要 E 的原文 |
| Z1–Z15 今天没有枚举 | 找到展开了那一截的全量或快档用例 |

## 八　没做什么

- 没编译、没跑任何测试（派发要求）；所有状态数是用例常量、断言或拿段数组现算的，不是这一次跑出来的。
- P5（位置寻址）四条全量与 P6 的状态数用例不钉，表里给的是推的（按单元数推），T50 那一格推的数（按 16 个单元，2^32 量级）与用例注释的「约 2 × 14 写，状态数上亿级」对不上，没核。
- σ 那条历史八步各写几次、txg 25 属于哪一步，用例没钉，没推算；6.2 里 σ 那一路补进来的约 10⁹ 是按四次覆盖写各 28 写粗估的。
- 没逐个读 harness 档 `tests/` 里的用例（派发只点名 checker 档与 `crash_injection.rs`）；harness 档里有没有别的逐前缀造崩溃状态的，没查。
- 三条整流前 41 次写、T4 与 `TreeSplitPool` 的 A、T32 与 T45 是否逐字节相同，都没写比对去跑。
- 抬 F 之后（txg 15 起）的单元写有没有落在回收过的槽上，只知道 E 的数据单元钉了 50176，其余没核。
- 派发提示说第二条流有 4 个 24 写段，现数是 3 个（第三节开头），按现数写。
- 草稿目录 `/tmp/claude-1000/m3-facts-case-design/` 只有一份 `calc.py`（现算用），没建编译目录与仓副本。
```

**出处 `research/prompts/m3-prune-gpu-r1-forks.md:1-29`（整段抄，未转述）**

```markdown
# 崩溃放量的前缀树、等价类剪枝、GPU 与判定存储：岔路单（2026-09-28）

<!-- doc-lint:not-numbers U1 U2 U3 U4 U5 U6 P6 T1 T2 T3 T4 P0 P1 P2 P3 P4 P5 S0 S1 S2 G0 G1 G2 G3 Q1 Q2 Q3 Q4 Q5 -->

出处：`research/prompts/_m3-prune-gpu-r1-body.md` 第一节。只写问题与候选的定义，不写倾向、不写已有的数。前缀树做不做、GPU 做不做、KV 存储做不做不是岔路（用户 2026-09-28、2026-09-27 定必做）。

| # | 问题 | 候选（各自的定义） | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| U1 | 用例按什么组织 | **α** 一棵操作树，每个节点只枚举一次，兄弟分支从共同节点全持久之后物化的镜像出发。**β** 模块 × 条件格，每格从最短搭建路径造起点、只枚举那一格。**γ** 用例结构不动，段结果按内容寻址缓存 | 某种形态漏掉今天覆盖到的一个独有条件；或它补不进今天没枚举的节点 | 三种形态各对事实表戊 3.1 的每个独有条件与 2.2 的 27 个只当基线的节点逐条报「留 / 漏 / 补进」 | 开着 |
| U2 | 只当前置的步骤算不算崩溃目标 | **全算**。**只算有独有条件的**：其余当搭建、不枚举 | 被当搭建的某一步有一个别处没有的条件 | txg 11–14 四次覆盖写逐次列出它们各自的根槽区域、落点、释放对象、环里点名它的记录，看有没有一次独有 | 用户已定：全算，代价交给剪枝（2026-09-28） |
| U3 | 只当基线的节点与没覆盖的组合怎么补 | 逐条：**挂成节点全量枚举** / **靠段内精确剪枝后全量** / **等实现**（删除，节点照样设计）；不进层 1 抽样 | 某条归了全量而剪枝之后仍枚举不完 | 每条报状态数（或式子）、剪枝之后的类数估计与选定去向 | 部分用户已定：全部设计进来、不退到抽样（2026-09-28）；各条去向开着 |
| U4 | 一个节点上多条路径的判定口径 | **按路径各出一份计数、判定共用**。**节点只判一种口径** | 同一节点上两条路径的判定不同 | 在共享节点的取样状态上按三条路径各判一遍逐项比 | 开着 |
| U5 | 粒度 | 节点、复用单位、失效单位、GPU 批、KV 块、双机分片单位：**对齐成同一个单位**（取哪一级：段 / 发布 / 操作）/ **各取各的** | 某一级上改一处代码要重跑的范围远超它真走到的；或一个单位的状态数大到装不进一张卡、一个块，或小到每单位开销盖过枚举；或「没测到」在那一级上看不见；或一个单位跨不了机器 / GPU 批的整切 | 每一级按这四把尺各报一个数 | 开着 |
| U6 | 一时枚举不完的节点怎么挂 | 用户已定设 ignore 标记、慢慢开启；形态的候选：**节点登记表里带状态字段**（开着 / ignore / 等实现，门禁按表列「本次未跑」）/ **沿用测试函数上的 `#[ignore]`**（一个节点一个测试函数） | 某种形态下一个 ignore 的节点在覆盖报告或门禁输出里看不见 | 两种形态各造一个 ignore 节点，看 54 号与覆盖报告列不列出它 | 部分用户已定（2026-09-28）；形态开着 |
| T1 | 前缀树的节点按什么划 | **按段**：一段一个节点。**按发布**：一次发布的单元段、记录段、根槽段、轮换段合成一个节点。**按操作**：脚本里一步一个节点 | 某种划法下两条流里实际相同的一截被切进不同节点、不能共享；或节点数多到节点管理的开销超过节点内的枚举 | 在三条从 mkfs 起的流与树分裂、位置寻址两类流上，三种划法各报节点数、共享节点数、共享节点罩住的状态数 | 开着 |
| T2 | 子节点的起点镜像 | **物化**：父节点全持久之后的镜像物化成新基线。**叠加**：基线停在 mkfs 之后，读时逐条叠整张写表 | 两种做法在任何一个状态（含原地覆写的撕裂态）上读出的字节不同；或物化一次的代价超过它在这个节点里省下的读代价 | 在第二条流取样的状态上两种做法逐字节比读出的内容、各报每状态读耗时 | 开着 |
| T3 | 节点走到哪些代码模块怎么机械登记 | **覆盖率插桩**：每个节点跑一遍带覆盖率的构建，记走到的函数。**按录制流的种类串**：节点写种类序列当身份。**不登记**：任何 `crates/` 改动重跑全部节点 | 改一个只被某个节点走到的函数，登记法没有把那个节点列进要重跑的集合（漏）；或列进了没走到它的节点（多） | 每种登记法造一次「改一个函数」的输入，报漏几个、多几个节点 | 开着 |
| T4 | 父节点结束镜像变了，下游怎么办 | **按结束镜像指纹连锁**：节点输入指纹含父节点结束镜像的哈希。**按代码指纹整棵重跑** | 存在一处改动让父节点结束镜像变了而下游指纹没变 | 造出这样一处改动，或论证它的指纹覆盖了下游读的全部输入 | 开着 |
| P | 剪枝用哪几样（可多选） | **P0** 基线随段前移；**P1** 单元级检查按内容去重；**P2** 恢复按读集复用；**P3** 池级 checker 按读集复用（键罩全部读位置加内容）；**P4** 增量枚举（格雷码次序）；**P5** 按论证跳过（当前段只有单元写、根没写时全部子集判同一类）；**P6** 按读位置的取值组合分叉枚举类、每类记相容子集数，不逐个过状态 | 某一样在任何一个状态上给出与照跑不同的判定（精确性破）；或它省下的量小到不抵它的维护代价 | 每样在取样的状态上与照跑逐项比判定、报不一致数与省下的比例；P5 另报它是否把被验证的结论当了前提 | 开着 |
| R | 记录核对器怎么剪 | **按它读的写表下标与持久位做键复用**。**照跑** | 键数接近状态数（复用无收益）；或复用出现不一致 | 在取样的状态上报键数与不一致数 | 开着 |
| S | 验证一侧的状态表示 | **S1** （节点，段内序号）+ 按需现算的位图。**S2** 只表示等价类，状态到类的映射批量算。**S0** 照今天每状态建 `Vec<bool>` | 某种表示在一个节点上装不进内存或显存；或它让某条剪枝做不出来 | 按段长 16、24、26、28 各报每状态字节数与一个节点的总字节数 | 开着 |
| G | GPU 做哪一段 | **G1** (a) 类检查。**G2** 状态到类的映射（每状态求读位置上的内容编号向量、按向量分组）。**G3** 两者都做。**G0** 不上 GPU（对照臂） | 选定的那一段在剪枝之后占剩余每状态耗时的比例小到 GPU 端到端（含传输）净收益 < 1；或 GPU 与 CPU 抽样对拍出现不一致 | 按 P 定下的剪枝之后，报每一段的剩余耗时占比与一张卡一批的状态数、批数 | 开着 |
| GL | GPU 代码住哪 | **池级 checker 带可选特性**。**checker 档**。**新建第六个 crate**（只依赖 `singlefs-format` 与 wgpu） | 某个落点让门禁 94 号红；或让 GPU 判定代码能引到 `singlefs_core` 而没有门禁看得见 | 三个落点各报 94 号四条红不红、GPU 代码能不能引到 core | 开着 |
| Q1 | 判定存储存什么 | **每状态一格**：块 2¹⁶ 个状态、每状态 1 字节判定向量编号。**只存类**：类表 + 每节点每类的状态数 + 违例样本，状态到类的映射要时现算 | 只存类的做法下，有一种事后要回答的问题（某个状态判成什么、违例落在哪个状态）答不出或要重算整节点；或每状态一格的总字节数超出两台机器的可用盘 | 按第二条流与三条流合计的状态数报两种存法的字节数，并列出事后要回答的问题各自怎么答 | 开着 |
| Q2 | 选哪个库 | **R1** redb 带 quick-repair。**R0** redb 默认。**K** RocksDB 同步写加 WAL。**F** 加固的文件 | 按 Q1 选定的存法，某臂在杀进程或掉电后丢块、坏块、打不开；或写入速率低于判定产出速率 | E162 的 S2、S3、S4 按 Q1 选定的存法各有够判档 | 开着 |
| Q3 | 块多大、块键带哪些指纹 | 块 2¹⁰ / 2¹⁶ / 一个节点一块；指纹按 T4 选定的连锁法 | 块大小让一个块跨了两个节点，或块键漏了某样判定的输入 | 按 T1 选定的节点报每节点状态数分布 | 开着 |
| Q4 | 两台机器 | **一个库**：第二台经网络送块。**各写各的**：本机合并 | 一个库时送块吞吐低于第二台的判定产出速率；各写各的时合并丢块或重块 | E162 S3 跨机格够判；或合并做成与分片账本同一套、自证会红 | 开着 |
| Q5 | 违例与发现日志 | **只写违例**（按签名去重、每签名留样本）。**全量与违例双份** | 只写违例时某类事后排查要的数据没留下 | 列出事后排查要回答的问题，逐个看两种写法答不答得出 | 开着 |
| D | 崩溃验证一侧的 GPU 归哪条决策 | **扩 D24（后台重活能不能卸给 GPU） 射程**。**新立一条决策**，把 D24（后台重活能不能卸给 GPU） 已定项 2 的硬约束按条搬 | —（政策题） | 用户定 | 开着 |
```
