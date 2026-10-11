# E49 重跑登记（第 2 次，差分）：记录头 311 → 487 之后，反向链宽度的代价侧判据在今天的 base（483）下判出什么，产物与正文引的数变不变

写于 2026-10-04，装置改之前、这一次的任何产物之前。**差分重跑**：臂、两种读法、判据、门槛、作废与停机条款照上一次登记 `research/prompts/e49-r1-prereg.md`（sha256 `07b93955061402141e0e91be4652eeef969adf9d3f9584efdca00f64bf5d539c`，第 1 次重跑，头 311 下 base 307 / 303），连同它文件头主 agent 2026-09-24 定的那一句（**两种读法都留，判据 2 的判定以读法乙为准，读法甲照算照报**）。这里只写变了哪些输入、哪几格要重判、贴着旧值的取样点与锚点怎么换；没写到的内容「同上一次登记」。跑出数之后要改，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走。

**装置写在哪（写死）：`scripts/experiments/e49_chain_width.rs`（独立手写模型，不与 `crates/` 共用代码），不是入库装置。** `crates/` 里只有 32 位这一种链宽、只有定长 4096 这一种记录（第三节）。变异表 `scripts/mutation/tables/e49_chain_width.tsv`，复跑行 `scripts/experiments/replay.sh:140`（`E49|e49_chain_width||e49-chain-width-2026-09-25.out|exact`）。

**准入**：`python3 scripts/gate/admission.py experiment /home/fy5090/code/singlefs E49` 退 2（准入登记里没有实验键 E49），不是 77。

## 一、问题

英文名：chain_width（照抄装置文件名；上一次登记没有这一行）

**问题单（`research/prompts/m3-r1-batch9-reruns-questions.md`，sha256 `d6b328eb6beebc0073200971a8dc3971db458571458748d6bd7b4f5c497616e7`，表头与第 4 行，quote-kb 机械抄）**：

**出处 `research/prompts/m3-r1-batch9-reruns-questions.md:12-13`（整段抄，未转述）**

```markdown
| # | 实验 | 问题 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
```

**出处 `research/prompts/m3-r1-batch9-reruns-questions.md:17-17`（整段抄，未转述）**

```markdown
| 4 | E116、E43、E49、E16、E157 | `scripts/experiments/` 下 e116、e157、e16、e43、e49 的源码里还有 311 / 188 / 67 旧值（删除与并发那条线跑 format-constants 的输出 `/tmp/claude-1000/m3-writeback2/format-constants.out`）：换成今天的值之后，各自的产物与正文引的数是否变 | 产物与 `scripts/experiments/replay.sh` 登记的 exact 产物不再逐字相同 | 每个都改完、复跑、与登记产物比过，变了的出新产物并回对正文 | 开着 |
```

这份登记只回答第 4 行里的 E49。每个量对应第 4 行的翻面观测或够判条件。另有一处与第 4 行同向的条款要求：D23 已定项 8 射程逐字「现行不含反向链的头是 483（487 − 4），不在扫过的档位里 ⇒ **重算之前不许把「选 16 不省东西」「64 位也免费」「两侧都不分岔」当结论引用。**」（第二节）——这一次的产物就是那次重算；射程那句改不改由主 agent 定，不归执行员。

**读法写死**（只有一种问法；占用的两种读法照上一次登记）：

| 名字 | 写死成 |
|---|---|
| 变了的输入 | `crates/singlefs-format/src/lib.rs:223` 头 311 → 487（`:220` 新根段 188 → 364 是它的分项、`:229` 点名项 67 → 64 是它的派生）。装置读它的通道：第 125 行 `JOURNAL_HEADER_BYTES`、第 127 行 `HEADER_BYTES_BEFORE`、第 129 / 131 行 `BASE_TODAY` / `BASE_BEFORE` 两个减法；第 326 行项数上界 `ITEM_UPPER_BOUND_TODAY = 67` |
| base 的两个被判取值 | 今天 = 487 − 4 = **483**；改之前（头 311 那一代）= 311 − 4 = **307**。⇒ 装置第 125 行改 487、第 127 行 `HEADER_BYTES_BEFORE` 改 311 |
| 「原来的结论」 | 上一次登记第六节 Q49.2–Q49.4 各格在登记产物 `research/results/e49-chain-width-2026-09-25.out` 上判出的值（那时的「今天」是 307） |
| 「变不变」 | 每格在 base 483 上判一次，与登记产物里 base 307 那一格比，相同记「不变」，不同记「变」；另报与实验页结论 3 那一档（base 95）比 |
| 「产物变不变」 | 新产物与登记产物 `cmp`，再按 Q49.1 的允许集合分类（Q49.7） |
| 项数上界的方向相反取样点 | 上一次第八节把项数上界从 2000 换成「今天一条定长记录最多装的项数」67；今天是 **64**（D23 已定项 4 / 12 / 17）⇒ 第 326 行改 64（第八节） |

## 二、被测条款与它引的定义

上一次登记 2.1 / 2.2 抄的是 311 那一版的 D23 已定项 4 / 8 / 12 / 17；2.3 的原判据（实验页三节）与 2.4 的装置段落照上一次登记读（2.4 抄的第 32–38 行今天行号不变；第 70–76 行那一段今天在第 105–111 行，内容不变，现查）。这一次用 `scripts/kb/quote-kb.py` 重抄 D23 已定项 4、8、12、17（已定项 8 是实验页 `### 影响的决策` 表点名的分项，也是被测条款；表里的 D13 没有分项，不抄）。产物 `/tmp/claude-1000/m3-r1/batch9-row4-design/e49-q.md`（4 段，回读逐字节一致），整份追加如下。

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:148-177`（整段抄，未转述）**

```markdown
#### 已定项 4：记录头约束在单个原子单元内，头 487 字节

**定案**：**记录头完整落在一个原子单元内**（[invariants.md](../invariants.md) I-8.2（记录头不跨原子单元））。头 **487 字节**：

- **十个字段 78 字节**：magic 4 + 类型 2 + 算法类型 1 + 记录标志 1（位 0 = 本次发布末条，已定项 17；其余位写 0，读到非 0 当损坏） + 自述长度 4 + 点名项数 4 + `jsn` **10** + `checkpoint_txg` 8 + nonce 12 + 头部校验和 32。`jsn` 的 10 字节 = 实例代号 32 位 + 计数器 48 位（已定项 9）。`tail_lsn` 不在头里：已定项 3 逐字定了「tail 的权威副本住在一个固定位置，**不内联在记录头**」，而「内联在每条记录头的 `tail_lsn`」正是被否掉的那个 XFS 形态的定义。
- **三笔已定增量 17 字节，不在那十个字段的 78 里**：已定项 7 的事务号 + 提交标记 **9 字节**、已定项 8 的反向链 **4 字节**、已定项 13 的载荷校验和 **4 字节**。
- **本次发布内序号 4 字节**（无符号 32 位，从 1 起：一次发布切成 N 条记录时依次是 1..N，只有一条时是 1；空发布记录也写 1）：已定项 14 注 1 在所选根那条记录读不出时靠它接链首。4 字节的宽度与紧跟事务号、提交标记的落点是主 agent 按「不为省空间牺牲自包含」取的，可推翻。字段在头里的偏移：事务号 78、提交标记 86、本次发布内序号 87、反向链 91、载荷校验和 95、新根段 99、fsid 463、MAC 471、头末 487。一次发布的 N 条记录序号依次 1..N、与 jsn 同步；读者遇到这四格，都当那条记录损坏、断链即止，那次发布整体不施加：序号 0；一次发布之内跳号；锚点读得出时，下一次发布的首条序号不是 1（断在这一条）；同一 (实例代号, checkpoint_txg) 里带末条标志的那条之后还有记录（断在带标志的那条；「之后」按计数器读、不要求紧接着：计数器大于带标志那条的、读得出的同 (实例代号, checkpoint_txg) 记录都算，中间缺号不停；前提是同一实例里 journal 记录的 `checkpoint_txg` 随计数器不减，有这个前提时与「表里任何位置」的全表读法给出同样的施加结果）。与 I-8.9（一次发布的记录序号连续且只有末条带标志） 判的是同一件事。
- **已定项 15 的新根段 364 字节**（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4）。
- **fsid 8 与 MAC 16，都排在新根段之后**：
  - **fsid 8**（系统配置 fsid 的低 8 字节，与单元头那 8 字节同口径，D18（块里携带什么信息） 已定项 7）。不带的话 I-1.4（块头 fsid 一致） 要给 journal 记录写一条豁免，而全环扫描就再也挡不住「上一个文件系统留在这块盘上的记录」——mkfs 整环写 0 只在 mkfs 真跑完时成立，重 mkfs 中途崩掉就不成立。带 8 字节之后不用豁免。
  - **MAC 16**（放在 fsid 之后；nonce 12 与算法类型 1 十个字段里已有）。理由与根记录那 29 字节同一条（D22（单元原子性怎么合成） 已定项 7）：journal 记录头是自证结构，没有父指针带 MAC ⇒ 不 day-1 留位，开加密那天要么整条走明文豁免（泄漏 `checkpoint_txg`、新根段里的两条指针、点名项位置），要么就是一次布局变更；D9（加密） 已定项 3 的原则逐字是「为加密留的位所有卷都留、不挪作他用」。I-6（加密不变量）不列它。
- 登记值就是 487（`JOURNAL_HEADER_BYTES`）；十个字段的 78 另登记为 `JOURNAL_HEADER_TEN_FIELD_BYTES`。**说 78 时要写明指的是十个字段，连同这一句一起引。** 487 占 512 扇区的 95%，其后还余 **25 字节**（装不下一个点名项，每项 56 字节）；4096 上余 **3609**（64 个点名项，⌊(4096 − 487) ÷ 56⌋ = ⌊3609 / 56⌋）。
   <!-- format-const: JOURNAL_HEADER_BYTES = 487 stale=头 277 字节|的 277 字节头|合计 **277 字节**|JOURNAL_HEADER_BYTES: u64 = 277|JOURNAL_HEADER_BYTES: u64 = 84|JOURNAL_HEADER_BYTES: u64 = 86|JOURNAL_HEADER_BYTES: u64 = 78|512), 428|512), 426|512), 434|4096), 4012|4096), 4010|4096), 4018|512 上 428|512 上 426|头 86 字节|头 **307 字节**|登记值就是 307|⌊3789 / 56⌋|头 **311 字节**|余 **201 字节**|JOURNAL_HEADER_BYTES = 311|512 上余 201 -->
   <!-- format-const: JOURNAL_HEADER_TEN_FIELD_BYTES = 78 -->

**射程**：⚠️ **那个原子宽度不由本工程定**：D20（承重面：单元的原子性与自包含）已定自证单元依赖**运行时探测**的 `physical_block_size`，不许把 512 焊进格式。⚠️ **这个值是格式常量，改它要三处一起动**：kb 标记与正文、实验源码的 `const` 与钉死它的单测、重跑实验并更新产物；由门禁阶段「格式常量在 kb 与实验源码之间同步」（`.claude/gate.d/code-source-discipline.sh` 的 format-constants 格）判红。记录尺寸取 4 KiB 定长之后对齐浪费恒为 0（已定项 12）；变长记录的对齐代价表只对变长记录成立。

**依据**：

- E23（journal 几何）：十个字段（跑时还带 `tail_lsn`）的头装得进一个 512 单元，不对齐、多条记录挤同一个原子单元时除第一条外都判不了撕裂——头要完整落在一个原子单元内。
- E75（记录尺寸与环几何）：各个头部读法在 512 与 4096 两档 `physical_block_size` 上都装得进一个单元；定长 2 的幂尺寸下头永远不跨单元。
- fsid 8 与 MAC 16：用户弹窗定案，没有走三方论证，可推翻，原话在变更史。
- 本次发布内序号 4 字节：用户 2026-09-23 定（`research/prompts/m2-presumed-clauses-r1-main-verification.md` K4 的 Z6），是推的、被攻过零轮，原话在变更史。
- 用户定案 2026-09-24：头 311 落地、末条标志放在原「填充 1」那个字节、序号的读者规则（原话在变更史）。
- 用户定案 2026-09-24：锚点读得出时首条序号不是 1、末条标志之后还有同 txg 的记录，两格都当损坏断链（实二十报告 `research/prompts/m2-lastflag-implementer-report.md` 第六节 2、3），原话在变更史；无实验：两格都要 CRC-32C 碰巧对上或改出来的镜像才走得到。
- 第四格的「之后」按计数器读、缺号不停，前提是同一实例里 journal 记录的 `checkpoint_txg` 随计数器不减：三方判决 `research/prompts/m3-formal-r2-main-verification.md` 第二节第 8 行与 `research/prompts/m3-formal-r3-main-verification.md` 第二节第四格补句一行，两条腿同向；前提另立不变量的判决是 `research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 9 行；用户定案（等前提判完一起写回），原话在变更史；无实验：补的是读者规则的射程，没有新的实验读数。
- 新根段加实例表指针、分配记录树根指针与卸载记号三项（头 311 → 487、点名项 67 → 64）：三方判决 `research/prompts/m3-ring-r1-main-verification.md` 第二节第 2 行（今天的施加法在写行、首个文件、卸载三种发布上补出错根，推的）与第四节第 1 条，`research/prompts/m3-ring-r2-main-verification.md` 第二节第 2 行（首个文件、卸载两种发布补进去之后与当初会写的根逐字节相同、checker 绿，攻方在副本上量的），`research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行（两位放进记录标志字节的形态被专用清扫发布打穿；施加处不从新根段来的只有卸载记号、实例表指针、分配记录树根三项）。用户定案「回到甲」（原话在变更史）。卸载记号占 4 字节是第一轮正推腿 H1-b 的算法（86 + 86 + 4），判决没有单独定宽。无实验：补的是施加处的字节，数都在三方副本上量过，没立实验号。

**欠**：C94（登记的格式常量与后来的定案对不上）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:226-246`（整段抄，未转述）**

```markdown
#### 已定项 8：记录头带一条 32 位反向链

**定案**：**记录头加一条反向链（前一条记录的 hash），宽度 32 位。** 覆盖哪些字段是已定项 10，hash 算法是已定项 11（完整 32 位 CRC32C），写路径上的取值口径是已定项 19 ②。

- **它补的缺口**：D23（journal 的角色与格式）三条已定项（定长环 / 断号即止 / 全环扫描）**各自成立，凑在一起漏**：定长环的槽位由 `jsn` 决定 ⇒ 断号之后的残留记录不会消失、只会等着被覆盖；而恢复之后从「前缀末 + 1」接着写，**那正好接在残留序列的前面**（已定项 22）。
- **同一个字段两个职责，两个都记**：分辨时间线；以及让记录核对器能用**与正向扫描完全不同的遍历**复核同一个前缀边界（正合 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」）。只按验证独立性记，它会在「为了省字节」的权衡里第一个被砍掉。
- **一旦 hash 覆盖了实例代号（已定项 10），反向链对「分辨时间线」这个职责就是冗余的**——实例代号那条出路是精确的，不是概率的；那时链剩下的职责是遍历独立性。
- ⚠️ **一次误接受没有第二道防线**：I-8.3（重放前缀严格连续）、在飞记录数上限、`checkpoint_txg` 三条闸都不触发（两条时间线在同一个 checkpoint 窗口内，那个字段一模一样）⇒ 后果是**把一条已丢弃时间线的指针层目标态装进现行状态**，落在 I-7.4（近 K 代块未被复用） 保护窗口内时它完全合法、静默为旧数据。

**射程**：⚠️ **误接受的分母是「空洞 → 恢复 → 续写 → 再崩」这个事故序列发生的次数，不是时间**；取的场景点（每天崩 10 次、十年）不是测量，本仓还没有崩溃频率遥测。链宽不随文件变大而需要变宽：一次事故里链只在一个接缝上被考验，随文件变大受压的是序号计数器（已定项 9）与环的几何（I-8.1（环几何够大））。⚠️ 接缝数口径压在「一次事故一个接缝」这条没建模的假设上，一次恢复若留下多个断点，分母跟着变 ⇒ **32 位与已定项 11 的 CRC32C 是同一个单点**，那个口径一倒两条一起倒，重开条件对两项同时生效（已定项 11 重开条件第 1 条）。⚠️ 代价侧（2 / 4 / 8 字节链在盘上多不多占单元，E49（反向链宽度 32 还是 64） 扫过六档头宽）只取决于记录头总字节 mod 8，**是偶然不是结构**，头宽再动就要重算；现行不含反向链的头是 483（487 − 4），不在扫过的档位里 ⇒ **重算之前不许把「选 16 不省东西」「64 位也免费」「两侧都不分岔」当结论引用。** ⚠️ XFS 的 `h_prev_block` 只在写入侧赋值、恢复侧不比对它，引它时只能引「字段形态」，不能引「现役这么防」。

**依据**：

- E32（上一条时间线的残留）：定长环 + 断号即止 + 从前缀末接着写，残留记录会被下一次恢复接上去重放；两条自然实现各违反一条判据、方向相反——缺口在规范里，不在实现里。
- E36（槽位映射那一维）：槽位与序号解耦救不了，新旧记录等长时与按序号那一支逐格相同，只在记录长度恰好不同时碰巧挡住。
- E39（反向链挡不挡得住残留记录）：开链的全部格子残留为 0 且一条自己的记录都没误杀；误接受率逐档贴住 2⁻ⁿ。
- E51（反向链的碰撞机会有多少次）：碰撞机会的正确分母是接缝数（同源记录的链值构造性相等），按它 32 位全部档位够用；把每一次相邻比较都算一次机会的遍历分母是过计。
- 用户定案：加反向链「字节代价小，而它同时买到两样」；宽度「如果 32 位够用 那么就 32」，原话在变更史；三方 2026-08-30 三条腿一致指向 32 位。

**欠**：C42（残留记录冒充合法前缀）；C200（记录头改宽之后代价侧没重算）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:313-330`（整段抄，未转述）**

```markdown
#### 已定项 12：记录尺寸取 4 KiB

**定案**：**记录尺寸定为 4 KiB，定长，写进系统配置**（XFS 把同一个参数写进系统配置的 `h_size`）。

- 与已定项 4 相容：记录头 487 字节要完整落在一个 `physical_block_size` 内，而 4096 被 512 与 4096 两种探测值都整除。
- 定长记录取 2 的幂尺寸之后，I-8.2（记录头不跨原子单元） 的对齐浪费**恒为 0**：记录背靠背摆下去，起始偏移恒是单元的整数倍。
- 4 KiB 记录装 64 个点名项 <!-- format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD = 64 stale== 67 项 -->（已定项 4）；再大只抬高 I-8.1（环几何够大） 里那个「最坏事务占用」，进而抬高环大小下界，买不到别的。
- **用户口径**：先按这个数落地，写代码时再调整。

**射程**：⚠️ 余量那一格按的是「一次 fsync 一个事务」的读法（目标负载 12 项事务恰占 1 条记录），与 D16（发布语义） 已定项 5 末段的切分纪律（一个事务最多写一个单元的用户数据）不在同一口径：按切分纪律加已定项 7 一条记录一个事务号，seq 批 1 是 8 条记录、每条只装 1 到 5 个点名项，「12 项恰占 1 条」那一格不存在；余量口径与环几何怎么改归 C310（事务切分纪律与记录数口径打架），要用户定。目标负载（D25（目标负载优先级））若长大一个量级，这个数要重推，那时另立实验。环回绕与「装成一条要先在内存里缓冲整条记录」两条代价没建模。

**依据**：

- E75（记录尺寸与环几何）：4 KiB 上目标负载恰占一条记录、跨记录数 0，几个头部读法给的容量相同；环下界与另一个二进制已发表的那一格逐格相等；定长 2 的幂尺寸下对齐浪费恒 0。
- 用户定案，原话在变更史。

**欠**：C310（事务切分纪律与记录数口径打架）。

```

**出处 `.claude/kb/decisions/23-journal的角色与格式.md:480-499`（整段抄，未转述）**

```markdown
#### 已定项 17：点名项 56 字节的字段表

**定案**：**一项一个单元（新池新建文件 12 项）：位置条目 14 × 2（偏移 0）+ 单元类型标签 1（28）+ 出生树 8（29）+ 出生 txg 8（37）+ key 尾段 10（45：码 1 写序；码 2 / 码 3 实例代号 4 + 出生序号 4 + 补零 2）+ flags 1（55）= 56 <!-- format-const: JOURNAL_NAMED_ENTRY_BYTES = 56 -->；不另带载荷 CRC（位置条目各带 4 字节校验和）。重放从点名项直接凑出映射 key。**

- **记录点名的每一项自带校验和**：D4（校验和位置） 已定校验和内联进父指针，而记录点名的东西在记录持久时它的父还没发出 ⇒ 记录临时充当父，否则重放时那些单元没有任何完整性凭据；恢复重放施加前必须逐项验证点名单元（已定项 14 第四条）。
- **一次发布切成 N 个事务、N 条记录时，这次发布共享的提交内生块（extent 树节点、inode 叶容器与根、分配记录树、记账树、中央映射树、树表单元）只在最后一条记录里点名**，前面几条不重复点名它们。
- **共享的提交内生块多于最后一条记录装得下（4096 记录 64 项）时，末条再跨记录**：从最后一个事务那条记录起依次多写几条，装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志；恢复按这个标志认一次发布的边界，一次发布的记录里没读到带标志的那条，整次不施加（已定项 14 第五条）。标志放在记录头十个字段里「填充 1」那个字节（改名「记录标志 1」），位 0 = 本次发布末条；每一次只有一条记录的发布（含空发布记录），那一条的位 0 也写 1——改新池新建文件的字节（w1、w4、t13 三条记录的这一字节与头部校验和 [46, 50)；w4、t13 两条的反向链 [91, 95) 也跟着变，E142（新池新建文件的干跑） 空白 G26 逐字节核过）。用户 2026-09-24 定。

**射程**：管点名项的字节；位置条目的构成归 D19（块指针的结构与宽度预算） 已定项 4 / 11，映射 key 的形态归 D19（块指针的结构与宽度预算） 已定项 6 / 10。⚠️ 共享的提交内生块只在最后一条点名时，崩在最后一条持久之前，那些单元没有任何记录点名——重放的回收路径要认得出这一格，层 0 的小负载要覆盖它（C491（多条记录时共享内生块在哪条点名没定））。

**依据**：

- E16（journal 的角色：WAL vs 意图日志）：记录点名的东西其父此刻还没写到盘上，记录必须临时充当父——逼出「点名的每一项自带校验和」。
- E77（发布的持久顺序）：点名项自带校验和、施加前逐项验证，是单元与记录之间那道屏障得以省掉的交换条件。
- 用户定案，原话在变更史。
- 用户定案 2026-09-23：共享的提交内生块只在最后一条记录里点名，原话在变更史。
- 用户定案 2026-09-24：末条再跨记录、最后一条带标志位（原话在变更史；三方 `research/prompts/m2-treesplit-r1-main-verification.md` T6（维持拒绝时拒不拒取决于池的历史），按今天的字面认末条会施加半次发布，带标志位那一格 0 / 0）。

**欠**：无。

```

## 三、实现今天的样子

### 3.1 `crates/`（2026-10-04 现查）

| 处 | 文件:行 | 今天是什么样 |
|---|---|---|
| 头宽 | `crates/singlefs-format/src/lib.rs:223`（487，`:222` 注释分项里含反向链 4） | 487 |
| 反向链 | `crates/singlefs-core/src/journal.rs:158`（`pub back_chain: u32`）、`:174`（`pub fn back_chain_of(…) -> u32`） | 只有 32 位一种；`grep -rn 'chain_width\|CHAIN_BYTES\|BACK_CHAIN_BYTES\|链宽' crates/ --include=*.rs \| wc -l` 输出 0 |
| 记录尺寸 | `lib.rs:213`（4096）、`:226`（56）、`:229`（64） | 定长 4096；读法甲的变长记录 `crates/` 里没有 |

### 3.2 装置里读旧值的每一处（装置 643 行，sha256 `8eb13c714e218d82fd4dd6b4440cbdbc275f790bb071e873044a3443b31fe112`；对照回扫报告附录表「改」里 e49 那 21 行与 `grep -n 'BASE_TODAY\|BASE_BEFORE\|JOURNAL_HEADER_BYTES'` 现查）

| 行 | 是什么 | 这一次怎么动 |
|---|---|---|
| 119、179、388、485 | 小节注释「E49 重跑登记（记录头 311）」与「今天是 `BASE_TODAY`（307）」 | 487 / 483；保留「上一次是 311 / 307」的事实时写成事件句 |
| 124–125 | `JOURNAL_HEADER_BYTES = 311` 与注释 | 487 |
| 126–127 | `HEADER_BYTES_BEFORE = 307` | **311**（改之前那一代成了 311）；注释写「2026-09-24 到这一次格式改动之前的现行值」 |
| 129、131 | `BASE_TODAY` / `BASE_BEFORE` 两个减法 | 不动代码，值随常量走（483 / 307） |
| 292–307 | 第八节读法甲扫描 `296..=319` | **`472..=495`**（第八节） |
| 309–322 | 第八节读法乙扫描 `280..=345` | **`440..=520`** |
| 325–326 | 函数内 `const ITEM_UPPER_BOUND_TODAY: u64 = 67;` | 挪到模块级，写成 `(RECORD_BYTES_FIXED - JOURNAL_HEADER_BYTES) / ITEM_BYTES`，值 64；注释「今天一条定长记录最多装的项数」照旧 |
| 487–500 | 单测 A1–A3：`== 311`、`BASE_TODAY == 307`、`BASE_BEFORE == 303`、`RECORD_BYTES_FIXED - JOURNAL_HEADER_BYTES == 3785`、每条 67 项 | `== 487`、`483`、`307`、3609 写成加法 `JOURNAL_HEADER_BYTES + 3609 == RECORD_BYTES_FIXED`、每条 64 项 |
| 523–541 | 单测 B1（改之前四格全 0）、B2（今天四格） | B1 换成 base 307 的四格（2v4 两档 0；4v8 512 上 (31, 31)、4096 上 (4, 287)）；B2 换成 base 483 的四格与 `cap_items`（第七节） |
| 544–550 | 单测 B3 `base_311_is_free_on_all_four_cells`（M8 的取样点） | 改名 `base_487_is_free_on_all_four_cells`，取样点 487 |
| 554–565 | 单测 B4 项数上界 67 | 上界 64，两个 base 307 / 483 |
| 576–577 | 单测 B6 读法乙两个被判 base | base 307：67 / 67 / 67；base 483：64 / 64 / 64；两对都 0 |
| 589–600 | 单测 B7（P4 的 base 340 / 342） | **不动**：P4 选的是「每条项数恰好在这一对链宽之间差一」的 base，与头宽取值无关（回扫报告判「留·异量」） |
| 604–620 | 单测 B8（`296..=319` 按 mod 8 分组） | 范围换成 `472..=495`，分组值不变 |
| 623–631 | 单测 B9（`280..=345` 读法乙分岔的 base） | 范围换成 `440..=520`，期望值换成第七节 B9 |
| 633–642 | 单测 B10 判别力自证（base 307、512、4v8 的 d = 31） | 取样点换成 base 483，d 仍是 31 |

`BASES` 六档（第 108 行）、收益侧、P1 / P2 / P3 照旧，一字不动。

## 四、跑之前已经存在的数

| 数 / 话 | 出处 | 对判据的影响 |
|---|---|---|
| 上一次登记第四节整张表与第七节 B1–B10（base 303 四格全 0；base 307 的 4v8 在 512 上 31 处首个 31、4096 上 4 处首个 287；读法乙 307 上 67 / 67 / 67；P4 的 340 / 342；mod 8 规律） | 读上一次登记 | base 307 那四格这一次是「改之前」那一侧，由 S1 现跑复现，不拿登记里的数当基线 |
| 回扫报告：实验页第 1、7、9、82、147 行是现状句（「现行不含反向链的记录头是 307」一类），今天为假；附录表 e49 那 21 行 | 派发给的输入 | Q49.8 的回对清单起点 |
| 实验页小节标题：「代价：「免费」是 `base mod 8` 的偶然」「2026-09-25：记录头 311 下 base=307/303 的代…」；第一行标题里的结论 | `grep -n '^#'` 找页结构时撞见 | 是结论；判据不照它写 |
| 实验页引产物名或含旧值的行号（只取行号）：引 `e49-chain-width` 的 67、156、177、215；含 `name=` 的 180–189；含旧值的 1、7、9、82、147、166、168、180–188、196–207、217、223、224；`### 影响的决策` 219–225、`## 历史版本` 226 起 | `grep -n … \| cut -d: -f1` | Q49.8 的待查行 |
| **我推出来、又用命令算过的（第十三节命令一）**：483 mod 8 = 3，与 307 同余 ⇒ 读法甲下 base 483 四格与 base 307 同判（4v8 两档不免费、2v4 两档免费），分岔数也同是 31 / 4，只有首个分岔项数不同（512 上 37、4096 上 357）；`cap_items` 512 上 0、4096 上 64；读法乙 483 上每条 64 / 64 / 64，两对都免费；项数上界 64 与 67 在 307、483 上给出同样的分岔数 | 命令一原样输出 | ⚠️ 跑之前就存在的答案：Q49.2 / Q49.4 各格预计「不变」，变的是首个分岔项数、`cap_items` 与扫描窗口。照纪律不删、不回改判据。⚠️ 它也说明「头宽退回 311」这条变异在判定格上不改输出（第三类：取样点不敏感），要靠钉首个分岔项数的单测抓（第九节 M13） |

## 五、臂、阳性对照、真实基线

同上一次登记第五节，只换：

- **base 的两个被判取值**：**307**（改之前，头 311 那一代；怎么做：今天的源码现跑出的登记产物里本来就有 base 307 那几行，S1 复现）与 **483**（今天；怎么做：按第三节 3.2 改常量，重出产物；做完会怎样：给出今天的那一格）。两个取值都是真实发生过的格式，不存在稻草人。
- **P5 base 旋钮读到了**：新产物 `name=cost_current` 两组行的 `base=` 是 307 与 483；两组行除 `base=` 外在首个分岔项数上不同（第七节 B1 / B2：`first_4v8=31` 对 `first_4v8=37`、`287` 对 `357`）。上一次的 P5 靠「判定不同的那一格」，这一次两个 base 同余、判定相同，P5 改由首个分岔项数与 `cap_items` 判。
- P1–P4 照旧（P4 的 base 340 / 342 不读头宽）。

真实基线是 base 483。**5.4 实现量**：改 3 个常量、挪 1 个常量到模块级、2 个扫描窗口、约 12 条单测的钉值与 2 个单测名；变异表改 1 条锚点、加 2 条；`replay.sh:140` 换产物名；实验页按 Q49.8 回对、加这一次的事件小节。**一段做完**。

## 六、报哪些量与各自的判据

全部对应问题单第 4 行（E49）。上一次登记 Q49.1–Q49.6 的算法、门槛、判定一字不改，只把两个被判的 base 换成 307（改之前）与 483（今天）、扫描窗口换成第八节的；新加 Q49.7、Q49.8。判据 2 的判定以读法乙为准，读法甲照算照报（文件头那一句）。

| 格 | 怎么算（产物里的行） | 门槛 / 判定 | 第 4 行：取什么值让它翻面 |
|---|---|---|---|
| **Q49.7 产物与登记的 exact 产物逐字比** | `cmp research/results/e49-chain-width-2026-09-25.out <新产物>`；不同就 `diff` 交 Q49.1 | 相同 ⇒「产物不变」；不同 ⇒「产物变」，报不同的行数 | 第 4 行的翻面观测本身；按第三节的改动必然「变」（扫描窗口换了） |
| **Q49.1 头宽没漏进别的格** | 同上一次登记 | 允许不同的只有：`name=cost_current*`、`name=cost_fixed_record*`、`name=base_sweep*`、`name=base_sweep_items*` 这几族行（base 换了、窗口换了）与 `name=done emitted=`（读法乙窗口 66 → 81 个 base）；`config`、六档 `cost`、`cost_summary`、`risk_*`、`poscontrol_*` 逐字节相同 | 超出 ⇒ 停机 S2 |
| **Q49.2 读法甲，base 483 的四格** | `name=cost_current base=483 unit=512\|4096` 两行拆四格 | 同上一次登记（`div = 0` 免费，`> 0` 不免费） | 与 base 307 同一格判定不同 ⇒ 那一格「变」 |
| **Q49.3 读法甲，base 307 的四格** | 同上，`base=307` | 同上 | 比较的另一侧；另与实验页结论 3（base 95 两侧都免费）比一次，只报 |
| **Q49.4 读法乙，base 483 与 307** | `name=cost_fixed_record` 两行 | 同上一次登记 | 两个 base 同一对判定不同 ⇒「变」 |
| **Q49.5 D23 已定项 8 射程「只取决于记录头总字节 mod 8」** | 读法甲在 base 472..495 上按 `base mod 8` 分组 | 同上一次登记 | 有一组不一致 ⇒ F1 |
| Q49.6 附带，只报数 | `cap_items` 在 307、483 上 | 不设门槛 | — |
| **Q49.8 正文回对** | 实验页 `.claude/kb/experiments/49-反向链宽度32还是64.md` 第四节列的待查行逐行过：引登记产物的整行在新产物里逐字找到；现状句（第 1、7、9、82、147 行「现行不含反向链的记录头是 307」一类）换成新值仍是真话就改，判定格的说法按 Q49.2–Q49.4 的判定写、带上读法；「2026-09-25：记录头 311 下 base=307/303」那一节是事件句，不改，这一次另起一节写 483 / 307 | 每一行报「改 / 留（事件）/ 数没变」；`### 影响的决策` 表 2 行重新回看，日期不早于新产物。D23 已定项 8 射程「现行不含反向链的头是 483 … 重算之前不许 …」与 C200（记录头改宽之后代价侧没重算）不归执行员，交回里列给主 agent | 第 4 行的够判条件 |

**够判点**：S1 过、P1–P5 过、Q49.7 与 Q49.1 判完、Q49.2–Q49.4 各判一次、Q49.8 逐行判完 ⇒ E49 这一行够判。

## 七、钉绝对值的断言

### 7.1 出自被测条款本身的（不符走第十节 F1，不作废）

| # | 断言 | 出处（第二节已整段抄） |
|---|---|---|
| A1 | 今天的 base = 487 − 4 = **483** | 已定项 4「头 487」（分项里含反向链 4）、已定项 8「宽度 32 位」；已定项 8 射程「现行不含反向链的头是 483（487 − 4）」 |
| A2 | 改之前的 base = 311 − 4 = **307** | 已定项 4 依据行「头 311 → 487」 |
| A3 | 4096 − 487 = 3609，一条记录装 64 项（读法乙在 base 483、链 4 上的每条项数） | 已定项 4「4096 上余 **3609**（64 个点名项）」、已定项 12「4 KiB 记录装 64 个点名项」 |
| A4 | 读法甲的判定在扫描的每个 `base mod 8` 分组里一致 | 已定项 8 射程（Q49.5） |

### 7.2 独立算出、用命令核过的（第十三节命令一；不符 ⇒ 作废 V3）

命令一先复现上一次登记的旧锚点当阳性对照（base 93 的 2 对 4 在 512 上 (31, 44)、base 84 的 4 对 8 在 512 上 (31, 35)、base 307 的 4 对 8 在 512 / 4096 上 (31, 31) / (4, 287)），全对上了才用它算下面的数。

| # | 断言（读法甲：项数 0..2000） | 值 |
|---|---|---|
| B1 | base 307 四格 | 512：2 对 4 = 0、4 对 8 = **(31, 31)**；4096：2 对 4 = 0、4 对 8 = **(4, 287)**；`cap_items` 512 上 3、4096 上 67 |
| B2 | base 483 四格 | 512：2 对 4 = 0、4 对 8 = **(31, 37)**；4096：2 对 4 = 0、4 对 8 = **(4, 357)**；`cap_items` 512 上 **0**、4096 上 **64** |
| B3 | base 487 四格（M8 的取样点） | 全 0 |
| B4 | 项数上界 64 | base 307 与 base 483：512 上 4 对 8 = **1**，其余三格 0（上界 67 时同值） |
| B5 | P3：base 84、单元 512、4 对 8 | 31 处，首个 35（不变） |
| B8 | 扫描 472..495 | 按 `base mod 8`：余 0 与余 7 四格全 0；余 1–4 只有 4 对 8 分岔（512 上 31、4096 上 4）；余 5–6 只有 2 对 4 分岔（512 上 31、4096 上 4）；分组一致 |
| B10 | 第八节判别力自证的 d | 31（base 483、512、4 对 8） |

| # | 断言（读法乙：项数 0..2000） | 值 |
|---|---|---|
| B6 | base 307、483 | 每条项数（链 2 / 4 / 8）307 上 67 / 67 / 67、483 上 **64 / 64 / 64**；两对都 0 |
| B7 | P4 的两个 base | 同上一次登记：base 340：67 / 67 / 66，4 对 8 = (455, 67)；base 342：67 / 66 / 66，2 对 4 = (455, 67) |
| B9 | 扫描 440..520 | 2 对 4 分岔的只有 453、454、509、510；4 对 8 分岔的只有 449–452、505–508；其余全 0 |
| B11 | 模块级 `ITEM_UPPER_BOUND_TODAY` | **64** |

**互比配的绝对值**：Q49.2 / Q49.3 配 B1、B2、B5；Q49.4 配 B6、B7；Q49.5 配 B8；第八节配 B4、B9、B10、B11。

## 八、轨迹与几何敏感性

8.1 不适用，同上一次登记。

8.2 照上一次登记的三类扫描行与判别力自证，只换取样点：

- 读法甲 `name=base_sweep reading=甲`：`base ∈ 296..=319` → **`472..=495`**（24 个，以 483 为中，覆盖 `base mod 8` 每个余数各三次）。上一次的窗口以旧的今天 307 为中，今天的 base 不在里面。
- 读法乙 `name=base_sweep reading=乙`：`base ∈ 280..=345` → **`440..=520`**：读法乙的判定只在每条项数跨过 56 的整数倍时变，483 两侧最近的翻面点是 449–454（每条 65 → 64 那一档的边）与 505–510（64 → 63），窗口两侧各留出余量。
- `name=base_sweep_items reading=甲`：`max_items` 67 → **64**、`base ∈ {307, 483}`。这一条仍是「定义域收窄」那个方向相反的取样点；按命令一，64 与 67 在这两个 base 上同值，所以 B11 另钉常量本身（第九节 M14 靠它红）。

判别力自证：单测里把 Q49.2 那一格（base 483、512、4 对 8）的门槛从 `div = 0` 挪成 `div ≤ 31`，必须由「不免费」变「免费」；读法乙同理挪到它自己那一格。

## 九、变异

**先后**：改装置之前在今天的源码上跑 `bash scripts/mutation/mutate.sh e49_chain_width scripts/experiments/e49_chain_width.rs scripts/mutation/tables/e49_chain_width.tsv`，留日志；改完之后再跑一遍。两次各按七个符号分栏数；M1–M8、M10–M12 两次的状态必须相同，任一条变差 ⇒ V4。

| 条 | 怎么改表 | 在哪个取样点上改变输出 | 谁该红 |
|---|---|---|---|
| M8 base算成含链 | 不改（原文 `JOURNAL_HEADER_BYTES - BACK_CHAIN_BYTES_TODAY` 不含数） | base 483 → 487：B2 的 4 对 8 两格由 31 / 4 变 0 | B2 的单测 |
| M9 头宽退回307 | 原文里的 `= 311;` 换成 `= 487;`，替换文 307 不改 | base 483 → 303（余 7）：B2 两格塌成 0 | B2、A1 |
| **M13 头宽退回311**（新加，编号接 M12） | `const JOURNAL_HEADER_BYTES: u64 = 487;` → `311` | base 483 → 307：⚠️ 四格判定与分岔数都不变（同余 3），**只有首个分岔项数 37 → 31、357 → 287，`cap_items` 0 / 64 → 3 / 67，读法乙每条 64 → 67** | B2 钉首个分岔项数与 `cap_items` 的断言、A1、A3、B6 |
| **M14 项数上界写死成旧值**（新加） | 模块级 `ITEM_UPPER_BOUND_TODAY` 的式子 → `67` | `base_sweep_items` 行的 `max_items=` 64 → 67；分岔数不变（B4 同值） | B11 的单测 |

M13 是 `mutation-sampling.md` 第三类的现成例子：判定格这个取样点对它不敏感，敏感的取样点是首个分岔项数；它没被抓就补钉那几个数的断言，不许记成等价。

## 十、失败条款

F1–F4 同上一次登记，只换数：F1 的观测改成 Q49.5 在 472..495 上有一组不一致，或 `name=cost_current` 的 `base=` 不是 307 / 483；F2（结论变）的观测同上一次——按第四节的算术预计不触发，触发就照实记；F3（全部不变）的观测改成「Q49.2–Q49.4 各格 483 与 307 同判，且 P5 看得到首个分岔项数不同」；F4（两种读法判出相反）照旧：按命令一，483 上 4 对 8 读法甲判「不免费」、读法乙判「免费」，F4 预计触发，两个都如实记、每句带读法。

## 十一、作废条款与停机条款

### 作废

V1、V3、V4、V5 同上一次登记。V2 改成：307 与 483 两组 `cost_current` 行除 `base=` 外逐字节相同（而第七节 B1 / B2 说首个分岔项数不同）。另加 V6：改完之后 `bash scripts/experiments/replay.sh E49` 对新产物不逐字节一致。

### 停机（两边都查，写进报告交回）

| # | 条件 | 什么观测会让它触发 |
|---|---|---|
| S1 | 基线没立住 | 改装置之前 `replay.sh E49` 不逐字节一致。⚠️ 这一次跑时装置 311、`crates/` 487，正处在 S3 的触发状态：只当比较基线，不出结论，写死的例外 |
| S2 | 新常量漏进了别的格 | Q49.1 超出允许集合 |
| S3 | 装置与 `crates/` 对不上 | 改完之后、出产物之前 `grep -n '^pub const JOURNAL_HEADER_BYTES:' crates/singlefs-format/src/lib.rs` 不是 `223 … 487`，或 `grep -n 'pub back_chain' crates/singlefs-core/src/journal.rs` 不是 `u32` |
| S4 | 跑的那一刻条款与第二节抄的不一致 | 用 `scripts/kb/quote-kb.py` 按第二节的取法重抄 D23 已定项 4 / 8 / 12 / 17，与第二节逐字节比有不同 |

### 出新产物、改登记行

- 产物名：`research/results/e49-chain-width-<跑的那天，TZ=Asia/Tokyo date +%F>-h487.out`（旧名没有头宽标签，这一次加 `-h487`）；旧产物留着，不覆盖。
- `scripts/experiments/replay.sh:140` 只改第 4 栏，成 `E49|e49_chain_width||e49-chain-width-<日期>-h487.out|exact`。

### 够判停机

第六节「够判点」一到就停：E49 这一行交回各格「免费 / 不免费」与「变 / 不变」，每句带读法。没有第二段；没跑的量：无。

## 十二、修订

写于 2026-10-04，装置改完、单测跑完、产物一次没跑之前。**没有收严或补臂：判据、门槛、臂的定义、失败与作废条款照第一至十一节原样，一条没改。**

读数（`nice -n 19 … cargo test --release --bin e49_chain_width`，输出 `research/results/e49-unit-test-2026-10-04-h487.out`）：25 个单测全绿，退出码 0。S1（改装置之前 `bash scripts/experiments/replay.sh E43 E49`，两行都是「字节一致」）与 S3（`grep` 现查 `223 … 487`，`pub back_chain` 是 `u32`）已过；S4 用 `scripts/kb/quote-kb.py` 按第二节的行区间取法重抄 D23 已定项 4、8、12、17，与 `/tmp/claude-1000/m3-r1/batch9-row4-design/e49-q.md` `cmp` 逐字节一致。

实现层与第三节 3.2 那张表不同的两处，都只多钉、不改判据：
1. A1 / A2 那条单测多钉 `HEADER_BYTES_BEFORE == 311`，以及两个 base 加链宽回到各自的头宽（加法写法）。
2. B4 里的项数上界写字面量 64、不读 `ITEM_UPPER_BOUND_TODAY`，常量本身由新单测 B11 `item_upper_bound_today_is_sixty_four` 钉（M14 靠它红）；B1 单测的函数名由 `base_before_is_free_on_all_four_cells` 改成 `base_before_diverges_on_four_versus_eight_only`（内容换成 base 307 的四格，与 3.2 一致）。

执行顺序上的一处偏离：第九节要求「改装置之前先跑一遍 mutate.sh 留日志」。这一次派发提示把整张变异表交回主 agent 跑（共用约束「长活交回主 agent」），我没跑改之前那一遍。V4 只在「原有 M1–M8、M10–M12（第九节写死的那十一条）里有一条在改之后比改之前差」时触发，改之后这些若全是 ✅，不论改之前怎样都不会触发；改之后有一条不是 ✅ 时，才需要改之前那一遍（在 HEAD 的装置与旧表上跑，旧表即 `git show HEAD:scripts/mutation/tables/e49_chain_width.tsv`）。变异表这一次改了 M9 的原文（`= 311;` 换成 `= 487;`，替换文不改）、加了 M13、M14，锚点逐条在新装置里恰好命中一次（命中数用脚本数过，M1–M14 都是 1）。

## 十三、读过的文件与跑过的命令

五份差分重跑登记（E116 第 2 次、E157 第 2 次、E16 第 6 次、E43 第 2 次、E49 第 2 次）是同一次派发里一起写的，这一节的 13.1–13.6 五份相同：列的是这一次派发里读过的全部文件，不只这一份用到的；13.7 起是这一份自己的命令。行号是读的那一刻（2026-10-04）的。`research/results/` 下的产物一份都没读（只在 `replay.sh` 登记行里看到文件名）；实验页的结果节没读，只读了标题行、`### 影响的决策` 一节与按行号取的 `grep -n … | cut -d: -f1`。

### 13.1 规则、共用约束与脚本

| 文件 | 行 |
|---|---|
| `.claude/agent-common.md` | 1–100（全文） |
| `.claude/singlefs-ai-sop/rules/test-discipline.md` | 1–122（全文） |
| `.claude/singlefs-ai-sop/rules/evidence-discipline.md` | 1–176（全文） |
| `.claude/rules/three-way-inference.md` | 1–174（全文） |
| `.claude/rules/mutation-sampling.md` | 1–96（全文） |
| `.claude/rules/implementation-first.md` | 1–20（全文） |
| `.claude/rules/format-evolution.md` | 1–81（全文） |
| `scripts/experiments/replay.sh` | 1–60 里的注释行；`grep -nE '^(E116\|E157\|E16\|E43\|E49)[A-Z0-9]*\|'` 命中 77、91、140、156–160、193 |
| `.claude/gate.d/stage-inputs.tsv` | 1–30 的注释行；`grep -nE '^(E116\|E157\|E16\|E16R5\|E43\|E49)\b'` 零命中 |
| `scripts/kb/quote-kb.py`、`scripts/kb/replace-once.py` | 文件头注释（1–30） |

### 13.2 派发给的输入与上一次登记

| 文件 | 行 |
|---|---|
| `research/prompts/m3-r1-batch9-reruns-questions.md` | 全文（17 行） |
| `research/prompts/m3-r1-batch9-reruns-evidence/sweep-311-188-67.md` | `grep -n '^#'`；1–224；附录表 225–418 里带 e116 / e157 / e16 / e43 / e49 的行与第 83–127、170–194 行（`awk` 按行号取）；419 起 `grep` e116 / e157 / e16 / e43 / e49 / `replay.sh` 的命中行 |
| `research/prompts/e116-r1-prereg.md` | 1–524（全文） |
| `research/prompts/e157-r1-prereg.md` | 1–260 |
| `research/prompts/e157-preregistration.md` | `grep -nwE '67\|68'` 命中 52、57、156、244、288、308、311、351、414、477、491、493、527、528、540、541、558、562、580、603、659、664 |
| `research/prompts/e16-r5-prereg.md` | 1–583（全文） |
| `research/prompts/e43-r1-prereg.md` | 1–395 |
| `research/prompts/e49-r1-prereg.md` | 1–44、158–440（读的是工具落盘的那份同内容副本，行号按原文件）、490–657 |

### 13.3 kb

| 文件 | 行 |
|---|---|
| `.claude/kb/decisions/23-journal的角色与格式.md` | `grep -n '^#'`；148–177；226–246；quote-kb 抄出的已定项 1、4、8、12、15、17 整节 |
| `.claude/kb/decisions/01-…`、`03-…`、`08-…`、`18-…`、`20-…`、`21-…`、`25-…`、`27-…` | `grep -nE '^#### 已定项 '` 的命中行；quote-kb 抄出的各节（D1-3、D3-7、D8-3 / 11、D18-7、D20-4、D21-2 / 3 / 4 / 7 / 8 / 14 / 15 / 16、D25-1–5、D27-2 / 3 / 5 / 6 / 13） |
| 五个实验页（`116-…`、`157-…`、`16-journal…`、`43-…`、`49-…`） | `grep -n '^#'`；`### 影响的决策` 一节（116：136–148，157：110–122 含表下那一段，16：360–377，43：263–282，49：219–225）只取前两栏；`grep -n` 产物名、`name=`、旧值三类只取行号 |
| `.claude/kb/decisions/*.md`、`checks-owed.md`、`invariants.md` | `grep -rnE '第五次跑\|E16R5\|e16-fifth'` 命中 23:77、23:86、checks-owed:455、:657（截前 140 字）；`grep -rnE 'E49（反向链宽度 32 还是 64）'` 命中 23:235；`grep -rnE 'E43（扩展点字节上限）.*(201\|3785\|311\|自证)'` 命中 21:98 |
| `.claude/kb/layout/01-first-txn.md` | `grep` 命中 329、357 |

### 13.4 实验装置与变异表

| 文件 | 行 |
|---|---|
| `scripts/experiments/e116_pack_settle.rs` | `grep` 命中各行；260–280；535–545 |
| `scripts/experiments/e157_parallel_line_one_clauses.rs` | 1–80；255–320；495–525；`grep` 命中各行；`git blame -L75,75` |
| `scripts/experiments/e16_fifth_run_peak_under_today_widths.rs` | `grep` 命中各行；220–236；419–445；780–800；817–880 的 `grep` 命中；880–890；1060–1090；1220–1242；`git blame -L887,887` |
| `scripts/experiments/e16_journal.rs` | `grep -nwE` 旧值，命中 915 |
| `scripts/experiments/e43_extension_point_budget.rs` | 140–182；196–252；440–475；590–641；`grep` 命中各行（含 `mount_verdict(` 的 193、439、649、651、656、671–682） |
| `scripts/experiments/e49_chain_width.rs` | 32、35、70–76、104–146；276–292；600–643 的 `grep` 命中；`grep` 命中各行 |
| `scripts/mutation/tables/e116_pack_settle.tsv`、`e157_parallel_line_one_clauses.tsv`、`e49_chain_width.tsv` | 全文（截前 160–220 字） |
| `scripts/mutation/tables/e43_extension_point_budget.tsv` | 第一栏全文；`grep` 命中 16、19、20、21 |
| `scripts/mutation/tables/e16_fifth_run_peak_under_today_widths.tsv` | 第一栏全文 |

### 13.5 `crates/`

| 文件 | 行 |
|---|---|
| `crates/singlefs-format/src/lib.rs` | 215–232；`grep -nE '^pub const (…):'` 命中 20、38、41、168、171、213、216、220、223、226、229、308 |
| `crates/singlefs-core/src/journal.rs` | `grep` 命中 158、174 |
| `crates/singlefs-core/src/transaction.rs` | `grep` 命中 178、184、190、191、290、6640、6643 |
| `crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-harness/src/model.rs` | `grep` 命中 480、478 |

### 13.6 五份共用的命令（原样；读文件的 `sed` / `awk` / `grep -n` 已按行列在上面）

```
for e in E116 E157 E16 E43 E49; do nice -n 19 timeout 60 python3 scripts/gate/admission.py experiment /home/fy5090/code/singlefs $e; echo "exit=$?"; done
  # 五个都是「✗ 准入登记里没有实验键 E…（.claude/gate.d/stage-inputs.tsv）」、exit=2
nice -n 19 python3 scripts/kb/quote-kb.py /tmp/claude-1000/m3-r1/batch9-row4-design/qrow4.md 'research/prompts/m3-r1-batch9-reruns-questions.md:12-13' 'research/prompts/m3-r1-batch9-reruns-questions.md:17-17'
  # ✓ 2 段整抄进 …/qrow4.md，回读逐字节一致（同名文件先前一次取错行区间，删掉重抄）
nice -n 19 python3 scripts/kb/quote-kb.py <草稿目录>/e116-q.md / e157-q.md / e16-q.md / e43-q.md / e49-q.md '<文件>@#### 已定项 N：…' …
  # 五次都是「✓ N 段整抄进 …，回读逐字节一致」、exit=0（8 / 5 / 8 / 14 / 4 段）
git log -L75,75:scripts/experiments/e157_parallel_line_one_clauses.rs --format='%h %ad %s' --date=short
git blame -L75,75 scripts/experiments/e157_parallel_line_one_clauses.rs      # 7a3c8e1e9 2026-09-27
git blame -L887,887 scripts/experiments/e16_fifth_run_peak_under_today_widths.rs   # dbcc0d2f5 2026-09-28
sha256sum scripts/experiments/{e116_pack_settle,e157_parallel_line_one_clauses,e16_fifth_run_peak_under_today_widths,e16_journal,e43_extension_point_budget,e49_chain_width}.rs scripts/mutation/tables/{e116_pack_settle,e157_parallel_line_one_clauses,e16_fifth_run_peak_under_today_widths,e43_extension_point_budget,e49_chain_width}.tsv
  # 装置依次 9417153e… 、b96446b8… 、44bbffd5… 、95dbf822… 、db61f951… 、8eb13c71…；变异表依次 211b8b33… 、01f7f81d… 、8d9abf0c… 、6df56d56… 、ad74fb9c…
sha256sum crates/singlefs-format/src/lib.rs      # bf4f9982490a7d30a4577547779c5061d76b211096cc42a8db71d45936b4835f
TZ=Asia/Tokyo date +%F                           # 2026-10-04
```

### 13.7 命令一：独立锚点（`anchors_h487.py`，全文；五份共用同一个脚本）

脚本 sha256 `4ba238ec10072e5ed9dfdb2fcda4d6baed44ca026391a09a6f4fd2bcebd2f113`，输出 sha256 `61d46e0a418478c4b5a08f75092afd29746ca9babc610021c6925694b13283ca`；在草稿目录 `/tmp/claude-1000/m3-r1/batch9-row4-design/` 里跑，执行员要复核时可以原样拷进 `research/` 下重跑。第一节到第十一节写完之前跑过一次（判问法时），第七节的数取自这一次的输出。

```python
#!/usr/bin/env python3
# 头 487 / 新根段 364 / 点名项 64 重跑登记的独立锚点。常量手抄自 crates/singlefs-format/src/lib.rs:215-229
# 与 D23 已定项 4 / 8 / 15 / 17；不 import 任何装置，公式照各装置源码里那一行的算式手写。
from fractions import Fraction as F
from math import ceil
REC, ITEM, H_OLD, H_NEW = 4096, 56, 311, 487
print("== 共用：头宽分项、一条记录装几个点名项")
print(f"  新根段 86+86+8+8+86+86+4={86+86+8+8+86+86+4}  头 78+9+4+4+4+364+8+16={78+9+4+4+4+364+8+16}")
for h in (H_OLD, H_NEW):
    q = (REC - h) // ITEM
    print(f"  H={h}: 4096-H={REC-h} per_record={q} 余={REC-h-q*ITEM} q*56+H={q*ITEM+h} (q+1)*56+H={(q+1)*ITEM+h} 512-H={512-h} 512 上装 {(512-h)//ITEM} 项")
hs = [h for h in range(0, REC) if (REC - h) // ITEM == 64]
print(f"  装 64 项的 H 区间 [{min(hs)}, {max(hs)}]；H={min(hs)-1} 装 {(REC-min(hs)+1)//ITEM}；H={max(hs)+1} 装 {(REC-max(hs)-1)//ITEM}")
print("== E116 journal 那一项（N=100000，映射条目 61，512 档）")
N, UNIT, NODE, W = 100_000, 32768, 16384, 2
data_w = 1725 * UNIT * W
saved = N * UNIT - 1725 * UNIT
base_key = data_w + 375 * NODE * W + 113 * NODE * W
base_fill1 = data_w + 100_000 * NODE * W + 101_725 * NODE * W
base_filln = data_w + 375 * NODE * W + 226 * NODE * W
for h in (H_OLD, H_NEW):
    j = N * (h + 61)
    print(f"  H={h}: journal_w={j} per_record_bytes={h+61} key512.mw={base_key+j} key512.payback={(base_key+j)/saved:.6f} fill512.b1.mw={base_fill1+j} fill512.b1.payback={(base_fill1+j)/saved:.6f} fill512.bN.mw={base_filln+j}")
print(f"  每格 move_write 之差 = N*(487-311) = {N*(H_NEW-H_OLD)}  saved={saved}  H*(key512)={F(saved-base_key, N)-61} ≈ {float(F(saved-base_key, N)-61):.2f}")
for h in (30851, 30852):
    p = (base_key + N * (h + 61)) / saved
    print(f"  敏感性 H={h}: payback={p:.7f} ge1={int(p >= 1)}")
print("== E43 自证单元档")
for h in (H_OLD, H_NEW):
    r5, r4 = 512 - h, 4096 - h
    b = min(r5, 255)
    print(f"  H={h}: room512={r5}({r5//ITEM} 项) room4096={r4}({r4//ITEM} 项) 占512={F(h,512)*100:.4f}% bound={b} binding={'journal' if r5 < 255 else ('tie' if r5 == 255 else 'root_slot')} fits9={int(b>=9)} fits13={int(b>=13)} fits128={int(b>=128)} 864/bound={864/b:.4f}")
def slots(slot, atomic): return max(atomic // slot, 1)
for h in (H_OLD, H_NEW):
    rows = []
    for atomic in (512, 4096):
        for slot in (256, 512, 4096):
            for n in (0, 128, 512):
                if n > 4096 - 105 - 1: v = "reject_no_payload"
                elif slots(slot, atomic) != 1: v = "reject_slot_shares_atomic"
                elif n > atomic - h: v = "reject_self_witness_overflow"
                else: v = "ok"
                rows.append((atomic, slot, n, v))
    print(f"  mount_eval H={h}: " + " ".join(f"{a}/{s}/{n}={v}" for a, s, n, v in rows if v != "reject_slot_shares_atomic"))
for h in (256, 257, 258, 311, 384, 385, 487, 499, 500, 503, 504):
    b = min(512 - h, 255)
    print(f"  sens512 h={h} room={512-h} bound={b} binding={'journal' if 512-h < 255 else ('tie' if 512-h == 255 else 'root_slot')} lb9={int(b>=9)} lb13={int(b>=13)} f128={int(b>=128)} n128={'overflow' if 128 > 512-h else 'ok'}")
for h in (311, 487, 3584, 3585, 3968, 3969):
    print(f"  sens4096 h={h} room={4096-h} n128={'overflow' if 128 > 4096-h else 'ok'} n512={'overflow' if 512 > 4096-h else 'ok'}")
print("== E157")
for h in (H_OLD, H_NEW, 456, 457, 512, 513):
    q = (REC - h) // ITEM
    print(f"  H={h}: per_record={q} 余={REC-h-q*ITEM}")
print("== E49（base = 头 − 链 4）")
def units(base, c, n, u): return ceil((base + c + n * ITEM) / u)
def div_a(base, a, b, u, top=2000):
    hit = [n for n in range(top + 1) if units(base, a, n, u) != units(base, b, n, u)]
    return len(hit), (hit[0] if hit else None)
def per(base, c): return (REC - base - c) // ITEM
def recs(base, c, n): return max(1, ceil(n / per(base, c)))
def div_b(base, a, b, top=2000):
    hit = [n for n in range(top + 1) if recs(base, a, n) != recs(base, b, n)]
    return len(hit), (hit[0] if hit else None)
print("  自检（复现 r1 的旧锚点）: 93 2v4 512", div_a(93, 2, 4, 512), " 84 4v8 512", div_a(84, 4, 8, 512), " 307 4v8 512/4096", div_a(307, 4, 8, 512), div_a(307, 4, 8, 4096))
for b in (307, 483, 487):
    for u in (512, 4096):
        print(f"  甲 base={b} mod8={b%8} unit={u} 2v4={div_a(b,2,4,u)} 4v8={div_a(b,4,8,u)} cap_items={max(0, u-(b+8))//ITEM}")
for b in (307, 483):
    for u in (512, 4096):
        print(f"  甲 max_items=64 base={b} unit={u} 2v4={div_a(b,2,4,u,64)[0]} 4v8={div_a(b,4,8,u,64)[0]}  (max_items=67: 2v4={div_a(b,2,4,u,67)[0]} 4v8={div_a(b,4,8,u,67)[0]})")
for b in (307, 483, 340, 342):
    print(f"  乙 base={b} per 2/4/8={per(b,2)}/{per(b,4)}/{per(b,8)} 2v4={div_b(b,2,4)} 4v8={div_b(b,4,8)}")
print("  乙 2v4 分岔 base（440..520）:", [b for b in range(440, 521) if div_b(b, 2, 4)[0] > 0])
print("  乙 4v8 分岔 base（440..520）:", [b for b in range(440, 521) if div_b(b, 4, 8)[0] > 0])
groups = {}
for b in range(472, 496):
    cells = tuple(div_a(b, n, w, u)[0] for u in (512, 4096) for (n, w) in ((2, 4), (4, 8)))
    groups.setdefault(b % 8, set()).add(tuple(c == 0 for c in cells))
    print(f"  甲扫描 base={b} mod8={b%8} 512:2v4={cells[0]} 512:4v8={cells[1]} 4096:2v4={cells[2]} 4096:4v8={cells[3]}")
print("  472..495 mod 8 分组一致:", all(len(v) == 1 for v in groups.values()))
print("== E16R5 乙的块数（今天 64 项 / 原装置记录宽 48+56k 按 4096）与 67 项一条对照")
for bt in (63, 64, 65, 66, 67, 68, 70, 72, 73):
    print(f"  batch={bt} today64={bt+ceil(bt/64)} old67={bt+ceil(bt/67)} legacy={bt+max(1, ceil((48+ITEM*bt)/4096))} w72={bt+ceil(bt/72)}")
```

```
$ cd /tmp/claude-1000/m3-r1/batch9-row4-design && nice -n 19 python3 anchors_h487.py > anchors_h487.out
== 共用：头宽分项、一条记录装几个点名项
  新根段 86+86+8+8+86+86+4=364  头 78+9+4+4+4+364+8+16=487
  H=311: 4096-H=3785 per_record=67 余=33 q*56+H=4063 (q+1)*56+H=4119 512-H=201 512 上装 3 项
  H=487: 4096-H=3609 per_record=64 余=25 q*56+H=4071 (q+1)*56+H=4127 512-H=25 512 上装 0 项
  装 64 项的 H 区间 [457, 512]；H=456 装 65；H=513 装 63
== E116 journal 那一项（N=100000，映射条目 61，512 档）
  H=311: journal_w=37200000 per_record_bytes=372 key512.mw=166240384 key512.payback=0.051623 fill512.b1.mw=6760374400 fill512.b1.payback=2.099316 fill512.bN.mw=169943168
  H=487: journal_w=54800000 per_record_bytes=548 key512.mw=183840384 key512.payback=0.057088 fill512.b1.mw=6777974400 fill512.b1.payback=2.104781 fill512.bN.mw=187543168
  每格 move_write 之差 = N*(487-311) = 17600000  saved=3220275200  H*(key512)=96410463/3125 ≈ 30851.35
  敏感性 H=30851: payback=0.9999892 ge1=0
  敏感性 H=30852: payback=1.0000202 ge1=1
== E43 自证单元档
  H=311: room512=201(3 项) room4096=3785(67 项) 占512=60.7422% bound=201 binding=journal fits9=1 fits13=1 fits128=1 864/bound=4.2985
  H=487: room512=25(0 项) room4096=3609(64 项) 占512=95.1172% bound=25 binding=journal fits9=1 fits13=1 fits128=0 864/bound=34.5600
  mount_eval H=311: 512/512/0=ok 512/512/128=ok 512/512/512=reject_self_witness_overflow 512/4096/0=ok 512/4096/128=ok 512/4096/512=reject_self_witness_overflow 4096/4096/0=ok 4096/4096/128=ok 4096/4096/512=ok
  mount_eval H=487: 512/512/0=ok 512/512/128=reject_self_witness_overflow 512/512/512=reject_self_witness_overflow 512/4096/0=ok 512/4096/128=reject_self_witness_overflow 512/4096/512=reject_self_witness_overflow 4096/4096/0=ok 4096/4096/128=ok 4096/4096/512=ok
  sens512 h=256 room=256 bound=255 binding=root_slot lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=257 room=255 bound=255 binding=tie lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=258 room=254 bound=254 binding=journal lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=311 room=201 bound=201 binding=journal lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=384 room=128 bound=128 binding=journal lb9=1 lb13=1 f128=1 n128=ok
  sens512 h=385 room=127 bound=127 binding=journal lb9=1 lb13=1 f128=0 n128=overflow
  sens512 h=487 room=25 bound=25 binding=journal lb9=1 lb13=1 f128=0 n128=overflow
  sens512 h=499 room=13 bound=13 binding=journal lb9=1 lb13=1 f128=0 n128=overflow
  sens512 h=500 room=12 bound=12 binding=journal lb9=1 lb13=0 f128=0 n128=overflow
  sens512 h=503 room=9 bound=9 binding=journal lb9=1 lb13=0 f128=0 n128=overflow
  sens512 h=504 room=8 bound=8 binding=journal lb9=0 lb13=0 f128=0 n128=overflow
  sens4096 h=311 room=3785 n128=ok n512=ok
  sens4096 h=487 room=3609 n128=ok n512=ok
  sens4096 h=3584 room=512 n128=ok n512=ok
  sens4096 h=3585 room=511 n128=ok n512=overflow
  sens4096 h=3968 room=128 n128=ok n512=overflow
  sens4096 h=3969 room=127 n128=overflow n512=overflow
== E157
  H=311: per_record=67 余=33
  H=487: per_record=64 余=25
  H=456: per_record=65 余=0
  H=457: per_record=64 余=55
  H=512: per_record=64 余=0
  H=513: per_record=63 余=55
== E49（base = 头 − 链 4）
  自检（复现 r1 的旧锚点）: 93 2v4 512 (31, 44)  84 4v8 512 (31, 35)  307 4v8 512/4096 (31, 31) (4, 287)
  甲 base=307 mod8=3 unit=512 2v4=(0, None) 4v8=(31, 31) cap_items=3
  甲 base=307 mod8=3 unit=4096 2v4=(0, None) 4v8=(4, 287) cap_items=67
  甲 base=483 mod8=3 unit=512 2v4=(0, None) 4v8=(31, 37) cap_items=0
  甲 base=483 mod8=3 unit=4096 2v4=(0, None) 4v8=(4, 357) cap_items=64
  甲 base=487 mod8=7 unit=512 2v4=(0, None) 4v8=(0, None) cap_items=0
  甲 base=487 mod8=7 unit=4096 2v4=(0, None) 4v8=(0, None) cap_items=64
  甲 max_items=64 base=307 unit=512 2v4=0 4v8=1  (max_items=67: 2v4=0 4v8=1)
  甲 max_items=64 base=307 unit=4096 2v4=0 4v8=0  (max_items=67: 2v4=0 4v8=0)
  甲 max_items=64 base=483 unit=512 2v4=0 4v8=1  (max_items=67: 2v4=0 4v8=1)
  甲 max_items=64 base=483 unit=4096 2v4=0 4v8=0  (max_items=67: 2v4=0 4v8=0)
  乙 base=307 per 2/4/8=67/67/67 2v4=(0, None) 4v8=(0, None)
  乙 base=483 per 2/4/8=64/64/64 2v4=(0, None) 4v8=(0, None)
  乙 base=340 per 2/4/8=67/67/66 2v4=(0, None) 4v8=(455, 67)
  乙 base=342 per 2/4/8=67/66/66 2v4=(455, 67) 4v8=(0, None)
  乙 2v4 分岔 base（440..520）: [453, 454, 509, 510]
  乙 4v8 分岔 base（440..520）: [449, 450, 451, 452, 505, 506, 507, 508]
  甲扫描 base=472 mod8=0 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=473 mod8=1 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=474 mod8=2 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=475 mod8=3 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=476 mod8=4 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=477 mod8=5 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=478 mod8=6 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=479 mod8=7 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=480 mod8=0 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=481 mod8=1 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=482 mod8=2 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=483 mod8=3 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=484 mod8=4 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=485 mod8=5 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=486 mod8=6 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=487 mod8=7 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=488 mod8=0 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  甲扫描 base=489 mod8=1 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=490 mod8=2 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=491 mod8=3 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=492 mod8=4 512:2v4=0 512:4v8=31 4096:2v4=0 4096:4v8=4
  甲扫描 base=493 mod8=5 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=494 mod8=6 512:2v4=31 512:4v8=0 4096:2v4=4 4096:4v8=0
  甲扫描 base=495 mod8=7 512:2v4=0 512:4v8=0 4096:2v4=0 4096:4v8=0
  472..495 mod 8 分组一致: True
== E16R5 乙的块数（今天 64 项 / 原装置记录宽 48+56k 按 4096）与 67 项一条对照
  batch=63 today64=64 old67=64 legacy=64 w72=64
  batch=64 today64=65 old67=65 legacy=65 w72=65
  batch=65 today64=67 old67=66 legacy=66 w72=66
  batch=66 today64=68 old67=67 legacy=67 w72=67
  batch=67 today64=69 old67=68 legacy=68 w72=68
  batch=68 today64=70 old67=70 legacy=69 w72=69
  batch=70 today64=72 old67=72 legacy=71 w72=71
  batch=72 today64=74 old67=74 legacy=73 w72=73
  batch=73 today64=75 old67=75 legacy=75 w72=75
```

### 13.8 交主 agent 认的项

1. 英文名 `chain_width`（从装置文件名取）。
2. 「改之前那一档」从头 307 换成头 311（base 307）：被比的两侧是相邻两代；头 307 那一代的 base 303 不再进被判取值（它的四格在上一次的产物里）。
3. 两个扫描窗口挪到以 483 为中（472..495、440..520），旧窗口 296..319、280..345 的行从产物里消失；要两份窗口都留由主 agent 改这一条。
4. D23 已定项 8 射程里「现行不含反向链的头是 483 … 重算之前不许 …」那一句、C200 的还账计划，随新产物由主 agent 判改不改。
5. 准入登记表里没有 E49 一行（admission 退 2）。
