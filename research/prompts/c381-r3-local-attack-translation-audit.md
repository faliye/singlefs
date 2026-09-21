# 转述核对表：c381-r3-local-attack（2026-09-21）

逐句核对 `research/prompts/c381-r3-local-attack-k3.md`（候选乙、K3 的逐格那一半）与
`research/prompts/c381-r3-local-attack-k1.md`（候选甲、K1 的逐格那一半）里每一句转述与中文原文。
kb 条款的行号现查各自的 kb 文件（工作区 2026-09-21 版本）；body/appendix 的行号按 Read 工具
显示的 `research/prompts/_c381-r3-body.md` / `_c381-r3-appendix.md` 自己的行号（这两份是本轮
派发给本地攻方腿的背景材料本身，不是 kb 条款，行号就是这两份文件自己的行号）。
格式：英文项 / 原文文件:行 / 首稿缺的或改动的 / 定稿理由。

## K3（候选乙：探针写改成写「这次失败的那个落点」）

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿 |
|---|---|---|---|
| Fact 1: persistence order + system config slot timing | `.claude/kb/decisions/16-发布语义.md:169`（「一次发布的持久顺序恒为COW单元/节点→屏障→journal记录→屏障→根槽（FUA）→系统配置槽…系统配置槽在根槽持久之后再更新，每个checkpoint一次…它是发布序列的最后一步」） | 有意不译「fsync等根槽持久之后才返回」与「恢复重放…必须逐项验证点名单元的校验和」两句：K3 只问落点与安不安全，不问 fsync 返回时机与恢复校验，译进去会把 K1 的材料带进只判 K3 的文档 | 只留段序与系统配置槽时机两句 |
| Fact 2: barrier 与根槽 FUA 的实际实现 | `research/prompts/d13-item4-fua-r1-user-verdicts.md:11,13,19`（「`WriteDurability::ForceUnitAccess`…与`barrier()`…调的是同一个`file.sync_data()`」「`write_all_at`走`pwrite`，不设`REQ_FUA`」「`ForceUnitAccess`的实际行为是写完再整盘刷一次…真FUA只保它自己那一次写，整盘刷把此前已完成的写也一起保了」） | 原文只说两个函数调同一个 `sync_data`、`write_all_at` 走 `pwrite`；「barrier() 只是纯 flush、自己不带任何字节」这一句是本稿的推断，不是原文逐字——推断依据：持久顺序把「屏障」列成与「单元/记录/根槽」并列的独立步骤（不是某次写自带的选项），且函数名是 `barrier()` 不是「bring durability to this write」；已在核对时单列，见下「多出来的限定词」一节 | 保留推断，但标为「本文档的推断，理由随附」，不当成原文逐字 |
| Fact 3: 根环几何、区域归属 | `.claude/kb/layout/01-first-txn.md:43-44,58,62,64`（区域内槽位公式、区域归属 0→盘0/1→盘1/2→盘0、mkfs 三区域各写一份、w2/w5 各只写一个区域） | 首稿多写了一句「if that one slot becomes unreadable…any transaction whose fsync had already returned during this mount, after that point, is lost」——这是把 `.claude/kb/decisions/16-发布语义.md:173`「每个新实例的第一个根在下一次发布之前是单点」这句**只针对新实例第一个根**的结论，错误地推广成了「任何一次根槽故障都会丢已确认事务」；同一决议里「同一实例里退一代，靠journal重放追得上」明说非首根的情形不是这样。核对时发现这处过度概括，已改 | 删掉那句过度推广，只保留「根槽不镜像，一次发布的那一代只由那一个物理槽罩着，不是两块盘各一份」这一句准确、且原文两处（169 与 173）都支持的陈述 |
| Fact 4: 候选乙定义 | `research/prompts/_c381-r3-body.md:25-26`（「探针写改成写『这次失败的那个落点』，不是固定落点。理由：它直接修掉『探针落在别处照样过』这个洞——落点级的坏会让探针写也失败，于是判持续失败，走只读，不再连切三次白做。⇒代价是把条款里『目标设备的固定落点』这四个字改掉，是改条款不是改实现」） | 首稿把「不再连切三次白做」简化译成「trigger wasted instance switch attempts」，漏掉「三次」这个具体数（N_switch=3） | 改成「trigger three wasted instance switch attempts before finally going read only anyway」，补回具体次数 |
| Fact 5: 现行的固定落点探针写（作对照） | `.claude/kb/decisions/23-journal的角色与格式.md:365`（「失败发生时先对目标设备的固定落点做一次探针写」） | 有意只译这一句机制定义，不译「写得进去⇒瞬时失败走切换；写不进去⇒持续失败走只读」的后续分支——K3 只问落点与安全性，不问判别之后走哪条支，译进去是给 K1/K4 用的材料 | 只留探针写机制本身 |
| Fact 6: 今天代码里一次发布的六步在同一闭包、任一步错都整体换回 | `research/prompts/_c381-r3-body.md:46`（「`publish_version`准入之后克隆分配器(:1265)、`publish_admitted`返回`Err`就整个换回(:1267-1269)，不分失败在哪一步；落盘闭包按单元→屏障→记录→屏障→根槽FUA→系统配置槽轮换」） | 无遗漏，按字面译 | 按字面译 |
| Fact 7: 今天代码零实现探针写/实例切换/只读 | `research/prompts/_c381-r3-body.md:49`（「全 core…`grep -rn -i 'probe_write\|探针写\|read_only\|ReadOnly\|转只读\|instance_switch\|实例切换' crates/singlefs-core/src/*.rs`零命中」） | 首稿多加一句「the current source code does not define, anywhere, at what point in time relative to fact 6's allocator restoration a probe write would actually run if it were implemented」——这是从「零命中」这个事实推出的一个空白点，不是原文逐字 | 保留，作为推断单列在下面「多出来的限定词」一节 |
| Fact 8: 落点在写出前已定 | `.claude/kb/layout/01-first-txn.md:81`（「自己那条的落点在写出前已定，D3（空间分配）已定项5的提交内生块」） | 无遗漏，按字面译 | 按字面译 |
| Fact 9: journal 环几何 | `.claude/kb/layout/01-first-txn.md:45`（「journal环，两盘互为镜像，记录4096，mkfs整环写0；196608个记录槽，在飞记录数上限=196608÷3=65536」） | 无遗漏，按字面译 | 按字面译 |
| Fact 10: 系统配置槽几何 | `.claude/kb/layout/01-first-txn.md:40-41`（「系统配置槽0…D22已定项8（每盘一份、2槽轮换）…槽数恒2、槽i偏移=i×固定结构槽距」「系统配置槽1」） | 无遗漏；「每盘一份、2槽轮换」译成「each device carries its own independent pair of two…rotations are independent of each other」，按字面展开，未增删实质 | 按字面译 |
| Fact 11: C381 已知缺陷 | `.claude/kb/checks-owed.md:341`（「发布的持久顺序是根记录FUA之后再转系统配置槽；系统配置槽那一步失败时根已落盘，恢复会选中它，而发布路径把分配器整个退回到这次发布之前」） | 无遗漏，按字面译 | 按字面译 |

## K1（候选甲：不立条款，明写「fsync 报错之后数据可以出现也可以不出现」）

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿 |
|---|---|---|---|
| Fact 1: persistence order | `.claude/kb/decisions/16-发布语义.md:169`（同 K3 Fact 1） | 有意不译「恢复重放…校验和」一句，K1 不需要 | 同 K3 Fact 1 的裁剪 |
| Fact 2: 根槽 FUA 实际行为（写+整盘刷，比真FUA强） | `research/prompts/d13-item4-fua-r1-user-verdicts.md:13`（「`ForceUnitAccess`的实际行为是写完再整盘刷一次…比真FUA强（真FUA只保它自己那一次写，整盘刷把此前已完成的写也一起保了）」） | 无遗漏，按字面译，并保留原提示（`c381-r2-local-attack.md` fact 15）已用过的同一句解读式收尾（「wherever…refers to after root slot force unit access succeeds…」），因为这是同一条 kb 事实、同一种读法，不是新内容 | 按字面译 |
| Fact 3: 今天代码六步同闭包、整体换回 | `research/prompts/_c381-r3-body.md:46` | 无遗漏，按字面译，比 K3 版本多一句「and this closure returns that error」——这是「`publish_admitted`返回`Err`」字面已经蕴含的返回值事实，不是新增内容 | 按字面译 |
| Fact 4: 零实现 | `research/prompts/_c381-r3-body.md:49` | 无遗漏；K1 版本不加 K3 那句「未定义探针写相对于换回的时序」推断，因为 K1 不问探针写 | 只译零命中事实本身 |
| Fact 5: `Mounted` 只有三个字段、`MountError` 没有只读变体 | `research/prompts/_c381-r3-body.md:47`（「`Mounted`(`:190`)只有`output`/`allocator`/`current`三个字段——挂载期没有能挂状态的对象；`MountError`里没有『这次挂载已转只读』这一种」） | 首稿把三个字段名译成「an output handle, an allocator, and a current version pointer」，替三个字段发明了未经证实的类型描述（「handle」「version pointer」原文没有）；核对时改回中性说法 | 改成「carries exactly three fields, named output, allocator, and current」，不再替字段发明类型 |
| Fact 6: 取号全或无、且是普通首次可写挂载的第一次写 | `.claude/kb/decisions/23-journal的角色与格式.md:365`（「取号要写独占打开成功的每一份系统配置、全或无」）+ `.claude/kb/layout/01-first-txn.md:60`（「取号先写进每一份系统配置、全或无，之后才动单元」） | 无遗漏，两处原文本来就是同一条规则的两次表述，合并翻译未增删实质 | 按字面译，两处出处并列标注 |
| Fact 7: C381 已知缺陷 | `.claude/kb/checks-owed.md:341` | 同 K3 Fact 11 | 按字面译 |
| Fact 8: 候选甲定义 | `research/prompts/_c381-r3-body.md:22-23`（「不立条款，明写『fsync报错之后数据可以出现也可以不出现』。理由：根已FUA落盘之后这次发布在盘上就是成立的；要让调用方的答复跟盘上结局一致，只能要求写路径在最后一步不失败，而那做不到。⇒与其立一条做不到的条款，不如明写这个语义，让调用方知道报错之后要自己去读」） | 无遗漏；「让调用方自己去读」后面补了「to find out what happened」收尾，原文没有这半句 | 补的半句只是「去读」这个动作的自然目的状语，不引入新事实，列在下面「多出来的限定词」一节 |
| T1–T4 四段定义 | `research/prompts/_c381-r3-body.md:84`（「T1单元写到一半、T2记录发出之后根槽FUA返回之前、T3根槽FUA成功之后系统配置槽轮换之中、T4取号那几次写」） | 无遗漏；四段的完整措辞沿用 `research/prompts/c381-r2-local-attack-part1.md` 已核过的 S1–S4/T1–T4 英文原句（同一套中文定义，两轮文字相同，本轮现查 r3 body 的措辞与 r2 body 一致后原样沿用，未重新措辞） | 沿用既有英文措辞 |

## 两份共用：本轮问题框架（不是 kb 条款）

失败点 P1–P8（K3 专用）与本文档的三道问题、K1 的三道问题，都是本轮新写的任务说明，不对应中文原文某一句，是本地攻方腿自己按主 agent 派发提示第「分工」节给的格数拆出来的任务框架；不逐句核对来源，但每个失败点/阶段的定义文字都来自上面已核过的 Fact 1、Fact 3（K3）或 Fact 1、Fact 2、Fact 6（K1），未引入未核过的新事实。

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| K3 Fact 2: "the standalone barrier function is only ever the flush, nothing else...those two steps have no target device offset and write no bytes of their own" | `research/prompts/d13-item4-fua-r1-user-verdicts.md:11,13` | 原文只说两个函数调同一个 `sync_data()`；「barrier 自己不带任何字节、没有目标地址」是本稿从「持久顺序把屏障列成独立步骤」加「函数名是 barrier() 不是某次写的选项参数」推出的，原文没有逐字这样说 | 这正是问二要模型自己判「屏障失败时落点是什么」的关键背景事实；不给这条推断，模型无法判断屏障是否有落点，而这条推断本身有两处原文依据支持（持久顺序的段序列、`barrier()`与`write_all_at`是两个不同函数），风险可控，列在此处供主 agent 核 |
| K3 Fact 7: "the current source code does not define, anywhere, at what point in time relative to fact 6's allocator restoration a probe write would actually run if it were implemented" | `research/prompts/_c381-r3-body.md:49` | 「零命中」是原文事实，「因此没有定义探针写相对分配器换回的时序」是本稿的推论 | 这是问二「写『失败的那个落点』安不安全」要用到的关键背景——P1（单元写）的安全判断依赖「探针写发生在分配器换回之前还是之后」，而这一时序在今天的代码里确实无定义（零实现），如实告知这个空白比让模型凭空假设一个时序更忠实 |
| K1 Fact 3: "and this closure returns that error" | `research/prompts/_c381-r3-body.md:46` | 原文写的是「`publish_admitted`返回`Err`」，本稿补了一句这个 `Err` 是从「六步共用的那个闭包」返回的 | 「返回`Err`」这件事本身已经蕴含在原文里，这里只是把「谁返回」这个主语从隐含变成明写，不引入新事实 |
| K1 Fact 8: "...it must go read the actual on disk state for itself to find out what happened" | `research/prompts/_c381-r3-body.md:23` | 原文「让调用方自己去读」没有说「去读是为了什么」 | 「去读」这个动作缺一个目的状语在英文里读起来不完整，补的目的（「查清发生了什么」）是「去读」这个动作本身唯一合理的目的，不引入候选甲原文没有的新语义 |

## 历史版本

（暂无历史，这是本轮第一次交这两份提示）
