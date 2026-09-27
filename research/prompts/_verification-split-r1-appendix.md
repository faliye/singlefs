**出处 `.claude/kb/decisions/13-验证路线.md:1-4`（整段抄，未转述）**

```markdown
## D13 验证路线 —— 已定

D13（验证路线） 管本工程拿什么验正确性：三类现成办法全做——自写参照实现做差分对拍、有界穷举崩溃测试、crash refinement 形式验证（调查见 [prior-art.md](../prior-art.md) 7.4）——并且不照抄，针对本工程特有的四样（COW 每事务发新根、Merkle 自验证、整卷 AEAD 的 nonce 唯一性、没有参照实现）自己设计验证手段。它定 oracle 怎么分、自研哪两样、崩溃点重放怎么分层与抽样、崩溃状态集合怎么定义、checker 与实现共享什么。不管的：每条不变量写什么在 [invariants.md](../invariants.md)；三样验证手段怎么落地、谁挡着谁在 [verification-build.md](../verification-build.md)；提交步骤登记在哪、结构等价类怎么分、今天几个，归 D17（实现分层与第三方管道） 已定项 2。

```

**出处 `.claude/kb/decisions/13-验证路线.md:70-84`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。层 0 的扇区取 512 字节；两条层 0 流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条层 0 流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；层 0 的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（第一个事务的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
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

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 27 号按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源只有类型（`crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`）、没有接到判定上，I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。集成测试不算共享：`crates/singlefs-checker/tests/` 下的崩溃枚举用例经 `[dev-dependencies]` 依赖 `singlefs-core` 与 `singlefs-harness` 造镜像再交给 checker 判，门禁 94 号只读 `[dependencies]` 与 `[build-dependencies]`（已定项 15）。

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

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-harness/src/crash.rs`，接在层 0 的每个崩溃状态上（门禁 54 号）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 层 0 不覆盖可写挂载：层 0 在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在层 0 的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-harness/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

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
| 层 0 冒烟 | 固定种子的最小复现负载（写请求数两位数） | **不抽样，全量** | checker 档的全量（已定项 15）：用户要求或夜间跑；提交时默认只跑快档 |
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

**出处 `.claude/kb/decisions/13-验证路线.md:316-327`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码分两档，按「什么时候跑」分，不按机制分。**harness 档**是 `crates/singlefs-harness`：单元测试与集成测试那一层，改了代码随时跑，实现员交回前跑自己动到的测试二进制；包内再分轻重，重的（随机历史长档、注入战役的大档、release 下要跑几分钟的）标 `#[ignore]`，要跑随时跑，一律经内存包装。**checker 档**是 `crates/singlefs-checker` 包里的测试（崩溃枚举用例、注入战役的快档与全量）连同 QEMU 真设备（门禁 55 号）、herd7（57 号）、crates 变异整表（59 号）、全部实验复跑（87 号）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档自己分两档：**快档**是 `cargo test --release -p singlefs-checker` 不带 `--ignored`（门禁 54 号在整轮门禁与提交时都跑它），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（54 号 `--full`），提交时不默认跑，由用户要求或夜间跑，跑法按已定项 9 分层；GPU 只接校验和那一截（E163（GPU多卡算单元校验和））、要不要接归 D24（后台重活能不能卸给 GPU）。崩溃枚举用例一律写在 `crates/singlefs-checker/tests/` 里（`research/scripts/crash-case-check.py` 判），不写进 harness；重型测试闸按包判：跑到 `singlefs-checker` 包的测试就是 checker 档。

**射程**：管 `crates/` 下测试住哪个包、什么时候跑、闸按什么认；不管每条不变量判什么（[invariants.md](../invariants.md)）、不管枚举域怎么定（已定项 4）、不管 checker 库与实现共享什么（已定项 5：checker 包的 `[dev-dependencies]` 依赖 core 与 harness 造镜像再判，不算共享）。「checker 档」与「池级 checker」是两个名字：前者是这一档的测试，后者是 `check_pool_image` 那个一元谓词（已定项 7）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`）。实验二进制与 QEMU 二进制留在 harness 包：它们不是测试，不决定什么时候跑。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；改动由主 agent 自己做完再走三方（用户同日定「这个任务你不要排subagent了 全部你来做」，随后补「改完后可以走三方腿验证」），判决出来后补进这里。

**欠**：无。

```

**出处 `.claude/rules/verification.md:1-5`（整段抄，未转述）**

```markdown
# 验证纪律：harness 随时跑，checker 档默认只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里——它说的是这个仓的验证代码住在哪、什么时候跑。
共享规则在 `.claude/singlefs-ai-sop/rules/`。

```

**出处 `.claude/rules/verification.md:6-12`（整段抄，未转述）**

```markdown
## 定义

- **harness**：`crates/singlefs-harness`。单元测试与集成测试那一层，加上它们要的脚手架库（录制器、理想模型、崩溃态枚举引擎、注入器、场景）。改了代码随时跑。判据：一条测试跑完在秒到一两分钟之间、不做全量崩溃枚举、不依赖真设备与外部工具，就是 harness 的测试。
- **checker**：`crates/singlefs-checker`。库是池级 checker（O2，单镜像一元谓词）；`tests/` 是 checker 档——直接调全量崩溃枚举函数的用例（全量那条标 `#[ignore]`，同文件的小流快档不标）、注入战役的抽样档。连同 QEMU（55 号）、herd7（57 号）、crates 变异整表（59 号）、全部实验复跑（87 号），默认只在提交时跑。判据：一条测试要枚举崩溃状态、要真设备或外部工具、或跑一次以十分钟计，就是 checker 档。

新写一条测试先按这两句归类，再决定放哪个包；拿不准的放 checker 档，再由代码三方判要不要挪回 harness。

```

**出处 `.claude/rules/verification.md:13-21`（整段抄，未转述）**

```markdown
## 两档按「什么时候跑」分，不按机制分

| 档 | 住哪 | 什么时候跑 | 谁跑 |
|---|---|---|---|
| **harness** | `crates/singlefs-harness`：单元测试与集成测试，连同录制器、理想模型、场景等脚手架 | 改了代码随时跑 | 实现员交回前跑自己动到的测试二进制（`cargo test -p singlefs-harness --test <目标>`、`--lib`）、fmt / clippy / build，经内存包装 |
| **checker 档** | `crates/singlefs-checker` 包里的测试（崩溃枚举用例、注入战役），连同 QEMU 真设备（门禁 55 号）、herd7（57 号）、crates 变异整表（59 号）、全部实验复跑（87 号） | 默认只在提交时跑；想单独跑就带 `SINGLEFS_HEAVY_TESTS=user-request` | 提交时 `crash-verifier` 与 `gate-triage` 各跑自己那一部分，命令带 `SINGLEFS_HEAVY_TESTS=commit`；其余子 agent 一律不跑 |

「checker 档」与「池级 checker」是两个名字：前者是这一档的测试，后者是 `singlefs-checker` 库里 `check_pool_image` 那个一元谓词（D13（验证路线） 已定项 7），harness 的日常用例随时调它。

```

**出处 `.claude/rules/verification.md:22-27`（整段抄，未转述）**

```markdown
## harness 里再分轻重

- 轻：每次改完跑，`cargo test -p singlefs-harness` 不带 `--ignored` 跑到的全部。
- 重：随机历史长档、注入战役的大档、release 下要跑几分钟的用例，标 `#[ignore]`，`#[ignore = "…"]` 的消息写清多久、给谁跑；要跑随时跑，一律经 `research/scripts/run-with-memory-cap.sh`，线程数按派发提示的上限。
- 直接调全量崩溃枚举函数（名字以 `enumerate_layer0` 起头、不带 `quick_tier`）的测试不算 harness 的重，它是 checker 档，写进 `crates/singlefs-checker/tests/`；`research/scripts/crash-case-check.py` 判这一条，写在 harness 里判红。

```

**出处 `.claude/rules/verification.md:28-36`（整段抄，未转述）**

```markdown
## checker 档自己分快档与全量

| 档 | 命令 | 什么时候 |
|---|---|---|
| 快档 | `cargo test --release -p singlefs-checker` 不带 `--ignored`；门禁 54 号跑它，再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，标记不作数的报「本次未跑」、不判红 | 整轮门禁与每次提交 |
| 全量 | 54 号 `--full`：在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），那一格全绿标记在就复用；分层照 D13（验证路线） 已定项 9；GPU 只接校验和那一截，要不要接归 D24（后台重活能不能卸给 GPU） | 用户要求或夜间 |

55、57、59、87 号照各自的复用判定跑（`research/scripts/stage-must-run.sh` 文件头）。

```

**出处 `.claude/rules/verification.md:37-43`（整段抄，未转述）**

```markdown
## 崩溃枚举用例住哪、怎么登记

- 写在 `crates/singlefs-checker/tests/<流的名字>.rs`，全量那条标 `#[ignore]`，同文件的快档用例不标。
- 共用的搭建模块经 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;` 这类声明指回 harness 的 `tests/common*/mod.rs`，不抄第二份。
- 全量那条登记进 `.claude/gate.d/stage-inputs.tsv` 一行 `crash-case:<名>`，第三列 `test=singlefs-checker:<测试目标>:<用例函数>`，计数行、`exhaustive=`、`threads=` 按 `research/scripts/admission.py` 文件头的写法；用例打不出的那一项不登记、在注释里写明。
- checker 包的 `[dev-dependencies]` 可以依赖 `singlefs-core` 与 `singlefs-harness`；库的 `[dependencies]` 只许 `singlefs-format`（D13（验证路线） 已定项 5，门禁 94 号判）。

```

**出处 `.claude/rules/verification.md:44-47`（整段抄，未转述）**

```markdown
## 崩溃一致性只能靠崩溃点重放验证

把块层所有写请求记下来，在**每一个**可能的崩溃点截断、重放、跑池级 checker 与记录核对器。**没跑过这个的写路径就不算验过**：单测全绿说明不了崩溃一致性。

```

**出处 `.claude/rules/verification.md:48-51`（整段抄，未转述）**

```markdown
## 功能正确性靠模型对拍

在内存里维护一个只管语义、不管性能也不管崩溃的理想文件系统（`crates/singlefs-harness/src/model.rs`），把同一串随机操作分别施加到模型和实现上，比结果。这是这个项目唯一的功能对照物，没有现成实现可以拿输出当标准答案。

```

**出处 `.claude/rules/verification.md:52-55`（整段抄，未转述）**

```markdown
## checker 即规范

不变量清单（`.claude/kb/invariants.md`）每加一条，池级 checker 就加一个检查。**「这个格式到底是什么」，答案以 checker 的源码为准，不是文档。**

```

**出处 `.claude/rules/verification.md:56-62`（整段抄，未转述）**

```markdown
## 文件系统特有的反推缺口

这个项目测的多半是对还是不对这种二选一的东西，风险在覆盖够不够：

- **「测试全绿」不等于「实现正确」。** 崩溃窗口可能只有一次写那么宽，没撞上也许只是没遍历到那个崩溃点。要说「崩溃一致性成立」，先说清这一趟本来撞不撞得上：枚举了多少个崩溃点，是不是全部。
- **「checker 没报错」不等于「镜像是好的」。** 也可能是 checker 还没实现那条检查。说这句话之前，先看 `.claude/kb/invariants.md` 里对应那条的实现状态。

```

**出处 `.claude/rules/verification.md:63-74`（整段抄，未转述）**

```markdown
## 门禁管哪一半

| 判什么 | 谁判 |
|---|---|
| 崩溃点重放跑了、快档绿、每条登记用例的全绿标记作不作数 | 54 号；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 模型对拍跑了、每段都判过 | 74 号；键同上表 |
| 崩溃枚举用例住在 checker 包、标了 `#[ignore]`、登记了 `crash-case:` | `research/scripts/crash-case-check.py`（47 号跑它的自证） |
| checker 库不依赖实现 | 94 号，只读 `[dependencies]` 与 `[build-dependencies]` |
| 谁在什么时候跑得了 checker 档 | `.claude/hooks/heavy-test-guard.sh`，判定在 `lib_heavy_tests.py`：跑到 `singlefs-checker` 包的测试、55 / 57 / 59 / 87 号、QEMU、herd7、`crates/mutations.tsv` 整表、全量 `cargo test`、整轮门禁、E152 装置算重型 |

**它们管不到的**：harness 里一条重用例该不该标 `#[ignore]`、快档抽的取样点够不够、全量该多久跑一次——这几样靠人与代码三方。

```

**出处 `.claude/rules/implementation-workflow.md:46-68`（整段抄，未转述）**

```markdown
## checker 档只在提交时跑，harness 随时跑

两档各是什么、谁跑、带什么前缀，在 `.claude/rules/verification.md`；这里只写实现改动流程里的落点：

| 场合 | 跑不跑 |
|---|---|
| 实现员交回前 | harness：自己动到的测试二进制（`cargo test -p singlefs-harness --test <目标>`、`--lib`）、fmt / clippy / build，经内存包装；checker 档一样都不跑 |
| 每次提交代码 | checker 档快档，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`crash-verifier` 跑 54 号（快档：`cargo test --release -p singlefs-checker`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）与 55、57、59 号；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
| 用户要求，或夜间 | checker 档全量：54 号 `--full`（HEAD + 暂存区的 worktree 里，逐条按输入复用）、其余重型测试；命令带 `SINGLEFS_HEAVY_TESTS=user-request`；任务确实要跑时主 agent 先弹窗问用户 |
| 其余任何时候 | checker 档不跑；子 agent 只跑 harness |

**重型测试**就是 checker 档那一批加上整机资源级的活，判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`（逐类的写法在它文件头）：跑到 `singlefs-checker` 包的测试（`cargo test -p singlefs-checker`、不挑包而包的范围含它、直接执行它的测试二进制）、54 / 55 / 57 / 59 / 87 号、`qemu-system-*` 与 `research/scripts/vm-bench.sh`、`.claude/scripts/lkmm.sh` 与 `herd7`、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `.claude/scripts/check.sh`、`gate.sh` 整轮与 `research/scripts/gate-staged.sh`、E152 装置。包装（`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里面的同样算；命令位置上执行的脚本闸读进去逐行判。由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带前缀的拒绝；主 agent 不带前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。

herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：

| 装置 | 阶段 | 判什么 |
|---|---|---|
| herd7 / LKMM | `.claude/gate.d/57-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
| QEMU 真设备 | `.claude/gate.d/55-qemu-first-transaction.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |

两道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
`--staged` 那条路（几个会话共写一个仓时）同样跑它们。

```

**出处 `records/2026-09-27-验证两档拆分.md:1-4`（整段抄，未转述）**

```markdown
# 验证代码分两档：harness 日常测试 / checker 提交时验证（singlefs-8b，用户定案）

来源：用户 2026-09-27 JST 18:2x–18:5x 在会话 singlefs-8b 里的定案，主 agent 自己做全部改动，不派 subagent（用户原话「这个任务你不要排subagent了 全部你来做」），改完之后派三方腿验证（用户随后补「三方腿都是 subagent， 改完后可以走三方腿验证」）。

```

**出处 `records/2026-09-27-验证两档拆分.md:5-13`（整段抄，未转述）**

```markdown
## 用户原话

- 「我们的 checker和harness的设计有问题。checker应该做的是 层零放量 形式化等等这些验证应该是提交时候做的，提交时候有两种选择，默认是快速检测，也可以选择全量，全量用分层，gpu可选。harness 应该做的是日常的检测，类似于代码test相关的内容。我们现在把两者混在了一起。」
- 「我们日常的代码开发中， 我们自己的代码 在写前后有实验 写后有代码三方验证 跑的时候有测试 harness测试就应该是rust代码测试这一层东西。现在把崩溃放量等等都放给了日常开发，改一行代码都要跑这里，这是很不合理的。」
- 「我觉得就两类 一个harness 一个checker 。 harness 日常改了代码随时可以跑， checker可以自己跑，但是默认只有在提交时候才跑。harness中再分重型不重型」
- 「harness就相当于单元测试 结合测试。 如果这个观点一致就开拆吧」
- 「上游那个词本仓 可以改 如果需要你直接去改上游的sop 。 我们现状也违背了我们上游独立的设计理念。」
- 「下游不应该绑定上游 上游也不应该绑定上游的 尤其是生成物，和这种状态上不能绑」

```

**出处 `records/2026-09-27-验证两档拆分.md:14-20`（整段抄，未转述）**

```markdown
## 拆之前的样子

- `crates/singlefs-harness` 一个包里装着：录制器、理想模型、场景等脚手架；崩溃态枚举引擎（`crash.rs` 5269 行）、断点续跑、崩溃注入、故障注入；实验二进制（`e158_root_choice_repair.rs` 24494 行等）；QEMU 真设备二进制；106 个测试文件 692 条用例，其中 22 条 `#[ignore]` 的放量用例散在 12 个文件里，与日常用例只靠 `#[ignore]` 与 `stage-inputs.tsv` 登记分开。
- 重型测试闸 `.claude/hooks/lib_heavy_tests.py` 靠「测试目标名含 layer0」与「包里有没有这种目标」猜哪条命令是重型，`cargo test -p singlefs-harness` 整包算重型，没有一条命令能跑 harness 的日常那一半。
- 规则 `.claude/rules/implementation-workflow.md` 要求每次提交跑层 0 全量；每条崩溃枚举用例的输入指纹是整个 `crates/`，core 改一行七格标记同时作废。D13（验证路线） 已定项 9 把层 0 定义成「写请求数两位数」的冒烟，而登记的并行线一那条流有 12230590578 个状态。
- 上游 SOP 的 `rules/test-discipline.md` 写着「崩溃一致性只能靠崩溃点重放验证」「功能正确性靠模型对拍」「checker 即规范」，`rules/evidence-discipline.md` 写着「文件系统特有的反推缺口」，`scripts/gate.sh` 把「模型对拍」「崩溃点重放」写死在未实现清单里，`scripts/selftest.sh` 拿这两个键当用例——上游绑着这个项目的验证手段。

```

**出处 `records/2026-09-27-验证两档拆分.md:21-28`（整段抄，未转述）**

```markdown
## 出口

1. 决策：D13（验证路线） 新立已定项 15（两档、crate 边界、触发），已定项 9 层 0 那一行的触发与射程改成现值，已定项 5 射程补「集成测试的 dev-dependencies 不算共享」；变更史一条。
2. 规则：新立 `.claude/rules/verification.md`（两档、harness 轻重、checker 档快档与全量、从上游收回的四节），`implementation-workflow.md`「重型测试只在提交时跑」改成指向它并按包判；`CLAUDE.md` 规则清单加一行；`main-agent.md`、`agent-common.md`、`implementation-writer`、`crash-verifier`、`gate-triage` 里的重型测试条款同步；改完走一轮三方，56 号与 72 号由判决点名。
3. 上游：`test-discipline.md` 三节与 `evidence-discipline.md` 一节从三语仓删掉、「门禁管哪一半」里点名这三样的句子改成通用说法；`gate.sh` 的未实现清单只留通用键（最终判据、命名纪律（shell）），项目键从项目根 `.claude/gate-not-implemented.tsv` 读；`selftest.sh` 的用例改用样本项目自己登记的键。只填内容，不动 VERSION、MANIFEST、CHANGELOG，填完告知发版会话。
4. 门禁与闸：`lib_heavy_tests.py` 按包判（跑到 `singlefs-checker` 包的测试就是 checker 档）；54 号快档改成 `cargo test --release -p singlefs-checker` 加逐条核标记，标记不作数报「本次未跑」不判红；94 号不读 `[dev-dependencies]`；`stage-inputs.tsv` 的 `crash-case:` 行改包名、补登记位置寻址那条放量用例；`crash-case-check.py` 改判「崩溃枚举用例要在 checker 包」；项目根新建 `.claude/gate-not-implemented.tsv`。
5. 代码：`crates/singlefs-checker/Cargo.toml` 加 `[dev-dependencies]`；8 个含放量用例的测试文件整份 `git mv` 进 `crates/singlefs-checker/tests/`，共用模块经 `#[path]` 指回 harness 的 `tests/common*/mod.rs`；实验二进制与 QEMU 二进制留在 harness（不是测试，不决定什么时候跑）。

```

**出处 `records/2026-09-27-验证两档拆分.md:29-38`（整段抄，未转述）**

```markdown
## 去向

| 项 | 去向 |
|---|---|
| 「checker」这个名字撞 O2 那个判决器 | 不改名：checker 档指 `singlefs-checker` 包里的测试，池级 checker 指 `check_pool_image` 那个一元谓词，两个名字并存；上游删掉「checker 即规范」那一节之后不再有第三处用法 |
| 实验二进制 24494 行占 harness 编译时间 | 这一轮不动；要动另立一题 |
| 层 1、层 2 没有阶段 | 照旧记在 D13（验证路线） 已定项 9 的欠 |
| 位置寻址那条放量用例没登记成 crash-case | 这一轮补登记 |
| 三方审核 | 改完之后走一轮（代码轮与定义轮合一），用户 18:5x 补充「我刚才的说明有点过了 三方腿都是 subagent， 改完后可以走三方腿验证」；56 号与 72 号由那一轮的判决点名满足，不走豁免表 |

```

**出处 `.claude/kb/decisions/13-验证路线.md:6-24`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **crash refinement 先验哪一块** | 先验「一次发布 + 一次崩溃 + 一次恢复」这个封闭单元，第一个里程碑是任一 `jsn` 连续前缀不切开发布；原候选 I-6.5（nonce 水位单调） 撤下 **状态：已定。** |
| 2 | **事务层承不承诺串行提交** | 先串行，越过交点再开并行；将来并行提交时记录按事务号顺序追加 **状态：已定。** |
| 3 | **自写参照实现（RefFS 形态）做到什么程度** | 做到极致——自己处理持久化与崩溃语义 **状态：已定。** |
| 4 | **崩溃点重放的崩溃状态集合怎么定义** | 写流按设备切段，段里每块有写的盘都被自己的屏障或 FUA 放行才关段；段内每次整写各取几态、任意组合，原地覆写多一态「新旧都读不出」；没持久放旧字节且真生成镜像；双写按设备各算 **状态：已定。** |
| 5 | **checker 与被测实现允许共享什么** | 只共享一份从 kb 生成的常量模块，其余各写一份；判定宽度由 checker 自己探测再与系统配置比对，探不到报「声明值，未探测」 **状态：已定。** |
| 6 | **三个互相独立的 oracle** | 内存模型对拍、独立解析器 + checker、独立规约执行器三个 oracle 各判一样；独立性要靠「三 oracle 合谋测试」验证，不许自称 **状态：已定。** |
| 7 | **O2（独立解析器 + checker） 判什么** | O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员；需要第二个输入的检查归记录核对器（入参含枚举器给的持久集合），故意不给编号 **状态：已定。** |
| 8 | **自研的两样** | Merkle 根链差分（相邻两代根递归比较，指针没变处不下钻）与代际增量 scrub（只遍历 birth generation 高于水位的块） **状态：已定。** |
| 9 | **崩溃点重放怎么分层** | 层 0 冒烟全量、层 1 常规按子阶段分桶抽样、层 2 全量夜间跑；夜间档是异步准入判据，先保失败可复现 **状态：已定。** |
| 10 | **层 1 抽样乘的 N 是什么** | 乘的是提交协议结构的等价类数，不是布局套数；落地是一个封闭的提交步骤枚举加穷尽 match **状态：已定。** |
| 11 | **层 1 抽样怎么调度** | 否决按欠账轮转的账本；留下三条：确定性的覆盖记录、`T1`（层 1 挂钟预算） 是输入、判别力与覆盖分开检查 **状态：已定。** |
| 12 | **冲突 1：checker 即规范与校验路径不许共享工具** | checker 做遍历与扫描两个方向，对同一条不变量各自独立判，共享的只有格式解析 **状态：已定。** |
| 13 | **冲突 2：refinement 的 spec 与对拍的 model** | spec 与 model 在表述层面就写得不同；同源性要观测，≥ 5 次不一致而判 spec 错 0 次就合并、预算移给穷举 **状态：已定。** |
| 14 | **与其他决策的连锁** | 哈希与 AEAD 在形式验证里只能当 uninterpreted function；日志结构三层绑架符号执行；checker 不必解析消息语义 **状态：已定。** |
| 15 | **验证代码分两档** | harness 是单元与集成测试、随时跑；checker 档（checker 包的测试、QEMU、herd7、变异整表、实验复跑）默认只在提交时跑快档，全量由用户要求或夜间 **状态：已定。** |

```

**出处 `.claude/kb/checks-owed.md:513-515`（整段抄，未转述）**

```markdown
| # | 简称 | 怎么还的 | 还清日期 |
|---|---|---|---|
| C577 | 系统配置没见证到的最新根，乙罩不到 | 两步：乙-配置续（取号那一写把见证值写进系统配置 tail，`research/prompts/m2-impl-c554-yi-carry-implementer-report.md`）与发布返回前加屏障（`crates/singlefs-core/src/transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`：根槽 FUA → 系统配置轮换 → 池屏障，屏障做完才返回，`research/prompts/m2-impl-c577-barrier-implementer-report.md`；新测试 `crates/singlefs-harness/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs` 钉「发布返回后每个崩溃点见证值 ≥ 末条记录计数器」，去掉屏障 `prove-red.sh` 3/3 红）。原点名的 `…known_red_form_of_closeout_row_43` 那两条用例造的是双故障形，改钉「红在 I-7.4（近 K 代块未被复用） 是已知」，账另立 C583（双故障形：每盘一槽系统配置坏加最新根一时读不出） | 2026-09-27 |
```

**出处 `.claude/gate-not-implemented.tsv:1-5`（整段抄，未转述）**

```markdown
# 项目自己接的验证手段：gate.sh 把这里的键并进末尾的未实现清单，项目本地阶段在头部写 `# gate-covers: <键>` 声明覆盖，
# 这一轮跑了且通过那一项才换成「由哪个阶段覆盖」。一行一条，制表符分隔：键、缺的是什么、覆盖之后仍要提醒的话（可空）。
# 两档怎么分、什么时候跑在 .claude/rules/verification.md；共享的「最终判据」「命名纪律（shell）」由 gate.sh 自己带，这里不登记。
崩溃点重放	记下块层的全部写请求，逐个崩溃点截断、重放、跑池级 checker 与记录核对器（.claude/rules/verification.md「崩溃一致性只能靠崩溃点重放验证」）	只说到它枚举过的那些写路径为止，别的写路径照样没验过。
模型对拍	同一串随机操作分别施加到理想模型与实现上，比结果（.claude/rules/verification.md「功能正确性靠模型对拍」）	
```
