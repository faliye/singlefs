# 转述核对表：c381-r2-local-attack（2026-09-20）

逐句核对 `research/prompts/c381-r2-local-attack.md` 里每一句转述与中文原文。kb 条款的行号现查各自的 kb 文件；`crates/` 代码的行号现查源文件（2026-09-20 工作区版本，HEAD `d1c5da4` 加未提交改动，与开工快照 `research/prompts/c381-r2-start-snapshot.sha256` 一致）；本轮自己的问题框架（失败点 P1–P8、阶段 T1–T4、两条臂 STOP / TABLE 的定义）按定义引用 `research/prompts/_c381-r2-body.md` 自己的行号，不是 kb 条款。格式：英文项 / 原文文件:行 / 首稿缺的 / 定稿。

| 英文项（提示文件里的句子，节选） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Fact 1: "...then the system config slot. ...does not return until the root slot write is persisted. ...every unit that record names must have its checksum individually verified. The system config slot's update happens only after the root slot is persisted, once per checkpoint; it is the last step of the publish sequence, and because of this the sequence of segments within one publish is unique." | `.claude/kb/decisions/16-发布语义.md:169`（「一次发布的持久顺序恒为...→根槽（FUA）→系统配置槽；fsync等根槽持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。系统配置槽在根槽持久之后再更新，每个checkpoint一次，它是发布序列的最后一步、进层0枚举，一次发布的段序列因此唯一。」） | 首稿把「再更新」直译成"updated again"，把「再」（这时才、随后）误当「又一次」，与原句「每个checkpoint一次」的单次语义冲突 | 改成"the system config slot's update happens only after the root slot is persisted"，去掉"again"这个误加的重复语义 |
| Fact 1: 不含"进层0枚举" | 同上 `16-发布语义.md:169` | 有意不译："进层0枚举"是验证装置（layer 0 崩溃点重放 harness）的内部术语，与本轮 K2、K3 判的「调用方拿到什么」「失败点落不落在失败表里」无关，译进去会让模型把与本题无关的验证装置细节当成要处理的事实 | 保留省略，不补 |
| Fact 2: "...does not confirm a rollback to the administrator. How that coverage is achieved, and how many publishes it takes, is defined elsewhere in this decision item and is not needed for this task." | `.claude/kb/decisions/16-发布语义.md:169`（「新实例在本实例写成的根覆盖两块盘之前，fsync不返回、不向管理员确认回退（做法与次数在已定项8）」） | 首稿漏译"（做法与次数在已定项8）"这个括注，直接删掉未做任何说明 | 补回一句"How that coverage is achieved, and how many publishes it takes, is defined elsewhere in this decision item"，并加"and is not needed for this task"说明为什么不展开——已定项8的暖机细节不影响K2、K3判的两件事 |
| Fact 4: "...first perform one probe write to a fixed location on the target device. If that probe write succeeds, the failure is classified as transient..." | `.claude/kb/decisions/23-journal的角色与格式.md:365`（「失败发生时先对目标设备的固定落点做一次探针写——写得进去⇒瞬时失败，走实例切换；写不进去⇒持续失败，走转只读到下次挂载」） | 无遗漏，逐句对应；「失败发生时」没有主语这一点特意保留原样（没有补成"when a write failure occurs"），因为这正是问二要模型自己判的空白，补上主语等于替它把答案定了 | 保持"when a failure occurs"，不加限定词 |
| Fact 5: "...this rule is described as a backstop of the backstop, and the real discriminant remains the probe write." | `.claude/kb/decisions/23-journal的角色与格式.md:365`（「它是兜底的兜底，真正的判别子是探针写」） | 无遗漏 | 按字面译，未增删 |
| Fact 6: "This process is guaranteed to converge because..." | `.claude/kb/decisions/23-journal的角色与格式.md:365`（「收敛靠的是切换的第一步就是一次会失败的写」） | 首稿把「收敛」译成"terminate"，与原文「收敛」（converge，指故障处置流程最终必然停在某个终态，不是单纯的「终止」）语域不同 | 改回"converge"，与原词对应 |
| Fact 7: "The reservation guards against the allocation failure path, not the write failure path." | `.claude/kb/decisions/23-journal的角色与格式.md:368`（「预留挡的是分配失败那一路，不挡写失败那一路」） | 无遗漏 | 按字面译 |
| Fact 8: 全段 | `.claude/kb/decisions/23-journal的角色与格式.md:369`（「『分配失败』按它发生在哪里拆三支...」） | 首稿漏译"不拆的话池一满就整卷只读...撞了`.claude/rules/fs-design.md`那句『释放空间这个操作本身不需要申请空间』"这半句理由 | 有意不补：这半句理由是「为什么要拆三支」的背景论证，不影响本轮要模型判的「这一条是不是写失败的判据」这件事，补进去会让Fact 8变成两件事的混合，核对表另列一行说明为什么不补 |
| Fact 9: "...collapsing R independent failure domains into one." | `.claude/kb/decisions/23-journal的角色与格式.md:370`（「不推进就是反复重写同一个槽、把R个失败域用成一个」） | 无遗漏；与`c381-r1-local-attack.md`的Fact 5逐字相同，此前已核过一轮（`research/prompts/c381-r1-local-attack-translation-audit.md`第9行），本轮直接沿用 | 沿用不改 |
| Fact 10: 不含所选根三种取法的括注 | `.claude/kb/decisions/23-journal的角色与格式.md:371`（「所选根取被重发的那个在飞checkpoint所基于的根（旧实例发布过根时是它最后发布的根，没有时是这次挂载恢复重放之后的根，回退那次发布里是R_old）」） | 有意不译：所选根具体是哪一条根的三分支判定，是K1（戊在攻方历史上拼不拼得出、切换所选根打不打得中）的射程，K1不归本地攻方腿（分工表：本地攻方分到K3、K2里丙戊两行，不许碰K1、K4、K5），译进去会把K1的材料带进只判K3、K2的这条腿 | 保留省略，不补 |
| Fact 10: "Transactions numbered greater than W are either redone under the new write order, or an error is reported to whichever caller of those transactions has not yet received a return." | 同上 `23-journal的角色与格式.md:371`（「号>W的按新写序重做或向还没返回的调用者报错」） | 无遗漏；这句本身是个「重做或报错」的析取，首稿曾想替它选一支译成单一结果，改回保留析取——这正是表3要模型自己判的空白 | 保持析取原样，不替它选一支 |
| Fact 11: 全段 | `.claude/kb/decisions/18-块里携带什么信息.md:313`（「再取新代号=max(...)+1并写进...全或无...回卷不成才只读挂载...取号之后那道屏障...不许只重发屏障就继续」） | 首稿漏译"（旧代号=取号之前这个集合里全部自证过的槽中最大的实例代号；回卷写发生在任何单元之前）"这个括注 | 有意不补：回卷时旧代号具体怎么算、回卷写发生在哪个时间点，不影响K2、K3判的「这一步失败算不算写失败、算不算表里的哪一支」，是实现细节而非判据本身 |
| Fact 12: 全句 | `crates/singlefs-core/src/mount.rs:863`（「回卷只管取号自己那几次写报错，管不到取号之后的发布失败。」） | 无遗漏；与`c381-r1-local-attack.md`的Fact 13逐字相同语义，行号从849变为863（工作区改动后行号漂移），本轮现查取新行号 | 按现查行号引用，措辞沿用 |
| Fact 13: 全段 | `crates/singlefs-core/src/transaction.rs:1265-1269`（`publish_version`函数：`allocator_before_this_publish`克隆与`outcome.is_err()`时整体换回）；六步顺序与"任一步返回错误就记失败账、把块设备错原样交回"这半句来自 `research/prompts/_c381-r2-body.md:35`（主agent对`crates/`的现查观测，本轮的前提材料，不是本地攻方腿自己重新grep代码得出） | 无遗漏；首稿曾把"the original device error is handed back to the caller unchanged"这句的来源标成本地腿自己读代码所得，核对时改标成引自body第35行的既有观测，避免本地腿的材料混入主agent自己的现查结论 | 来源标注改为`_c381-r2-body.md:35`，译文不变 |
| Fact 14: 全句 | `research/prompts/_c381-r2-body.md:39`（「grep -rn -i 'probe_write\|探针写\|read_only\|ReadOnly\|转只读\|instance_switch\|实例切换' crates/singlefs-core/src/*.rs 零命中：失败表的三样（探针写、实例切换、转只读）都没有」） | 无遗漏 | 按字面译，把中英文两种写法都点出（"in both their English and Chinese renderings"），对应原grep命令本身查的就是中英文两套词 |
| Fact 15: 全段 | `research/prompts/_c381-r2-body.md:38`（「『根槽FUA』这一步在实现里不是真REQ_FUA...候选表第三列『根槽FUA成功之后』按这个语义读：根槽写完并整盘刷过，此前已完成的写也一起落了」） | 首稿漏译"2026-09-20用户定案维持这个保守做法、不改成发真REQ_FUA"这半句（用户定案的存在与否） | 有意不补：定不定改真FUA是另一题（D25目标负载优先级一类的产品判断），与本轮K2、K3判的「这一步失败算什么、调用方拿到什么」无关，补进去是多余的历史背景 |

## 本轮问题框架（不是kb条款，按定义引用 `_c381-r2-body.md` 自己的行号）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| P1–P8 失败点列表（unit write, first barrier, journal record write, second barrier, root slot force unit access, system config slot, number taking during an ordinary mount, number taking and row writing performed by a switch itself） | `research/prompts/_c381-r2-body.md:90`（「失败点：单元写、第一道屏障、记录写两盘、第二道屏障、根槽FUA、系统配置槽两盘、取号那两次写、切换自己的取号与写行」） | 首稿把"记录写两盘"简化译成"the journal record write fails"，漏掉"两盘"（两块盘各一份）这个量词 | 补回"to its two on disk copies; zero, one, or both copies may actually have persisted"，与`c381-r1-local-attack.md`的S2定义保持同一套量词处理方式 |
| T1–T4 阶段定义（unit write partway/no record sent; record sent before FUA returns; FUA success during config rotation; number-taking writes） | `research/prompts/_c381-r2-body.md:45`（候选表表头「单元写到一半（记录一份都没发）」「记录发出之后、根槽FUA返回之前」「根槽FUA成功之后、系统配置槽轮换之中」「取号那几次系统配置槽写」） | 无遗漏；T1–T4的英文措辞直接沿用`c381-r1-local-attack.md`的S1–S4定义（两轮的四段划分文字逐字相同），核对时确认两处中文原文（`_c381-r1-background.md`与本轮`_c381-r2-body.md`）用词一致，未必要重新措辞 | 沿用r1的S1–S4英文原句，只改标号S→T |
| Arm STOP 全部四段 | `research/prompts/_c381-r2-body.md:47`（候选表丙行：「分配器退回，写入口照旧」「写入口作废：这次挂载里不许再发布，发布调用报错，调用方要重开，由恢复决定盘上是哪一版」「同左」「取号自己的写报错时回卷（今天的代码）」） | 首稿的T1、T2/T3三段里漏掉了"发布调用报错"这半句（这次失败的调用本身拿到什么），只译出了"写入口作废""调用方要重开"这些管后续调用的部分；这是起草过程中发现并改正的问题，不是核对时才发现 | 补回"the publish call that failed reports an error to its caller"（T2/T3）与"the call whose own write failed at this stage receives the original device error"（T1，引Fact 13），使表3能就"这次失败的调用拿到什么"逐格判 |
| Arm TABLE 全部四段 | `research/prompts/_c381-r2-body.md:48`（候选表戊行：「按失败表：先对目标设备的固定落点做一次探针写...第一格是不是也走这张表，是第二问」「同左」「同左」「同丙」） | 无遗漏；"第一格是不是也走这张表，是第二问"这句明确的开放问题原样保留（未替它先下结论），对应到提示里问题5 | 按字面译，指到"question 5"（对应正文问二在本轮框架里的位置） |

## 其余核对：英文比原文多出来的限定词（未改主体，逐条记为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| Fact 10: "since every transaction already completed in the open checkpoint has a number greater than W" | `.claude/kb/decisions/23-journal的角色与格式.md:371`（「开放checkpoint里已完成的事务全部>W，C287按这个W封死，注4」） | 把"注4"这个内部交叉引用换成了直接陈述该注的内容本身 | 注4本身就是这句话的出处，模型读不到`.claude/kb/decisions/23-journal的角色与格式.md`的注脚编号，直接把注4说的事实写出来等价于把引用展开，不引入新事实 |
| Failure point P3: "zero, one, or both copies may actually have persisted" | `research/prompts/_c381-r2-body.md:90`（「记录写两盘」，未展开说持久情况）；量词处理沿用 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项14 全篇「两份镜像何时算『记录在』：任一份自证校验和过即在」一带的既有事实 | 补出"zero, one, or both"这个量词范围 | 与`c381-r1-local-attack.md`的Stage S2定义（"zero, one, or two copies of the journal record may actually have persisted"）用同一套处理，避免模型把"记录写失败"理解成非黑即白的单一结果 |
| Background 段: "A separate decision item states a rule...called the failure table below" | 综合 `.claude/kb/decisions/23-journal的角色与格式.md:365-371` 与 `research/prompts/_c381-r2-body.md:15`（前提一「引这一格时整段一起引」） | 把"失败表"这个称呼直接点出来、并说明它来自事实4到9 | 提示全文反复用"the failure table"这个名字指代同一组事实，Background段先把这个名字定义清楚，减少模型把"the failure table"误认成另一份文档的风险，不引入新的实质内容 |

## 历史版本

（暂无历史）

## 追加（2026-09-20，按主 agent 指示拆分成三份）

### 为什么拆

第 1 次调用整份提示 `research/prompts/c381-r2-local-attack.md`（未改动，原样保留）时网关退出码 3，报「输出预算不足被截断」，三次内部重试全部判「正文复读跑飞」后掐断（`partial_content` 停在表 1 的 P5 附近，约 6000 字符）。诊断：这份提示一次要填的格数是 48（表1，P1–P8 × C1–C6）+ 16（表2，丙/戊 × P1–P8）+ 8（表3，丙/戊 × T1–T4）= 72 格，超出本地模型一次答得完的量，是网关判「输出预算不足」而不是字词损坏闸判红；换措辞不能解决产量问题，要拆小。原样报错抄在 `research/prompts/c381-r2-local-attack-runlog.md` 的「第 1 次调用的原样报错」一节。

### 拆成了哪三份，每份带哪几条事实

三份新文件（原提示一个字未改，仍在 `research/prompts/c381-r2-local-attack.md`）：

- `research/prompts/c381-r2-local-attack-part1.md`：事实 1–15、失败点 P1–P8、阶段 T1–T4、两条臂 STOP/TABLE 定义全部逐字保留（与原提示对应区间逐字节相同，见下「逐字核对」）；只留表 1 的前半——失败点 P1–P4 × 条款 C1–C6，24 格；问题只有 1 道（交出这张半表）。
- `research/prompts/c381-r2-local-attack-part2.md`：同样的事实/失败点/阶段/臂定义全量保留；只留表 1 的后半——失败点 P5–P8 × 条款 C1–C6，24 格；问题 2 道（交出半表；原提示问题 5，关于 P8 与 N_switch 的追问，移到这里，因为 P8 落在这一半）。
- `research/prompts/c381-r2-local-attack-part3.md`：同样的事实/失败点/阶段/臂定义全量保留；不问表 1，只问表 2（丙/戊 × P1–P8，16 行，K3 的判臂那一半）与表 3（丙/戊 × T1–T4，8 行，K2 里丙戊两行）；问题 3 道（交出表2、交出表3、原提示问题 4 那道跨行一致性追问）。

三份共用的事实/定义区块（原提示第 1–282 行，intro 指示 + Background + 事实 1–15 + P1–P8 + T1–T4 + 两条臂定义）逐字节核对：

```
diff <(sed -n '1,25p;27,282p' research/prompts/c381-r2-local-attack.md) <(sed -n '1,25p;33,290p' research/prompts/c381-r2-local-attack-part1.md)
```

三份之间只在两处偏离原提示的逐字文本，都不是重译，理由分述如下。

### 拆分时做的两处必要改动（不是重译，是结构性修补）

1. **补一句「这是三份中的第 N 份」的框架说明**，紧跟在 intro 指示段之后、Background 之前。三份措辞各自不同（点出自己拿到哪半张表、另外两份不用管），这是拆分本身带来的新增说明，不对应原文任何一句中文，无需核对表逐句核对，但按主 agent 要求在此处交代清楚为什么加：不加的话模型会以为自己要答全部 72 格，看不到问题只有一部分时可能误判提示缺失。

2. **修掉原提示里一处悬空的前向引用**：Arm TABLE 定义里「Whether this first stage, T1, is even governed by this table at all, is itself an open question, addressed separately in question 5 below」——这是起草原提示时留下的一处内部错误（现查原文，`research/prompts/c381-r2-local-attack.md` 第 278–279 行）：原提示的问题 5 问的是 P8 与 N_switch 的关系，与「T1 是否落在表里」不是同一件事；这句本来应该指向「表1本身」（表1的 P1、P2 行的 C1 列判词就是这件事的答案），不应该指向问题5。原提示已经交回、按「原样保留」不再改动，这处错误留在原文件里、本核对表记一笔；三份新文件里统一改成「addressed separately by table 1's own per row verdicts, not settled by this arm definition」，去掉指向一个不存在于任何一份新文件里的「question 5」的悬空引用，不改变这句话本身承认「T1 是否落表」是个未决问题这一实质内容。

### 表 2 任务说明的一处结构性改写（同样不是重译）

原提示的表 2 任务第三步写的是「using your own verdict from table 1's clause C1 column for this same failure point」——这句话要求引用表 1 的既有判词，而表 1 现在落在 part1 与 part2 两份**独立**调用里，与 part3 之间没有共享的模型上下文，part3 读不到 part1/part2 答过什么。为了不让 part3 依赖另外两次独立调用的产物，`c381-r2-local-attack-part3.md` 的表 2 任务改成四步：第一步让模型就地重新判一次该失败点对 fact 4（即 C1 那条核心规则）是「在表里/不在/原文不定」，判法与原提示表 1 对 C1 列的判法逐字相同（同一段指示文字，只是从「表1的一列」搬成「表2每一行自己的第一步」）；第二到第四步与原表 2 任务的第一到第三步相同。这样 part3 自成一份，不依赖 part1、part2 的产物；副作用是主 agent 事后可以拿 part1/part2 里 P1–P8 对 C1 列的判词，与 part3 表2里同样八个失败点、同一条 fact 4 重新做的判词做一次交叉核对——两次独立调用（不同抽样）对同一件事判得一不一致，是主 agent 判决时可以用的一条线索，本条腿不做这个核对、也不判它们一不一致。

### 三份各带的字词损坏检查处理方式

与原提示同样处理：前台跑 `research/scripts/ask-local.sh`，判红（退出码5）按定义作废重跑、留 void 副本；判绿之后仍跑 `oov-check.py` 通读一遍找缺头粘连词；退出码非0非5或网关不通，照样停下报「本地腿缺席」，不重试。

## 追加（2026-09-20，part3 再拆成 part3a / part3b）

part3 单份（表2+表3+3道问题）在第一次调用即触发与原整份提示同一种「输出预算不足」失败（退出码3，见 `research/prompts/c381-r2-local-attack-runlog.md`）。按主 agent 指示拆成两份，`research/prompts/c381-r2-local-attack-part3.md` 原样保留不改。

- `research/prompts/c381-r2-local-attack-part3a.md`：只留表2任务（丙/戊×P1–P8，16行）；表2里「自足化改写」那一步（就地重判一次C1列，不引用外部产物）按主 agent 明确指示原样保留，不精简。
- `research/prompts/c381-r2-local-attack-part3b.md`：只留表3任务（丙/戊×T1–T4，8行）。

原 part3 的「Questions」一节有三道问题：问题1「交出表2」、问题2「交出表3」、问题3（跨表2十六行的一致性追问，只谈表2的行，不涉及表3）。拆分时按每道问题依赖哪张表分配，不是主 agent 消息字面「表3……与那三道问题」那句可能暗示的「三道题都放 part3b」——问题1、问题3都要表2任务的说明文字在同一份文档里才谈得上（表2任务定义、P1–P8失败点定义、丙/戊臂定义虽然在两份里都逐字保留着，但「表2」这个具体表格是 part3a 才有的任务，part3b 没有表2任务，问题1、3放在 part3b 里没有对应的任务说明可循）。这里按依赖关系而非字面分配，问题1、3归 part3a（各自重新编号1、2），问题2（交出表3）单独归 part3b（编号1）。这处解读差异本条腿自行判断、未回头向主 agent 确认，交回时写明。

两份共用的事实/定义区块（原提示第1–282行）逐字节核对，只有一处偏离：各自的框架说明段（part3a："This document covers table 2 only..."；part3b："This document covers table 3 only..."），以及沿用 part1/part2/part3 已经修过的同一处悬空引用修补（"addressed separately by table 1's own per row verdicts..."），不再重复列出核对细节。

## 追加（2026-09-20，part3a 再拆成 part3a-1 / part3a-2）

part3a（表2整份，16行）第一次调用仍触发同一种「输出预算不足」失败（退出码3，见运行记录）。按主 agent 事先给的兜底指示（「part3a若还是撞输出预算……改成按失败点再切一半」）执行，`research/prompts/c381-r2-local-attack-part3a.md` 与它的 0 字节作废副本原样保留不改。

- `research/prompts/c381-r2-local-attack-part3a-1.md`：表2的P1–P4×两臂，8行。
- `research/prompts/c381-r2-local-attack-part3a-2.md`：表2的P5–P8×两臂，8行。

按主 agent「判据一个字不动,只切行」的要求，表2那四步判断的指示文字（就地重判C1列、命名阶段T1–T4、引臂定义原句、判consistent/conflict/not applicable）逐字未改，只把「for each of the eight failure points P1 through P8」改成各自的四点范围、把「sixteen rows」改成「eight rows」、把组织方式的收尾句相应改窄。

问题2（原表2十六行的一致性追问）没法在切开之后保留原样：切开后每份文档只看得到自己那8行，看不到另一半的8行，原问法「跨十六行找同一条臂在不同失败点判词前后矛盾」在单份文档里已经不成立。这里的处理是把追问收窄成「只在本文档给出的这8行内部找矛盾」，并在题面里明说「另一份文档问的是另外4个失败点，两份互相独立」。这处收窄记在这里：**跨 P1–P4 与 P5–P8 两半的一致性核对，这一轮本地腿交不出来**，如果要做，得由主 agent 把 part3a-1、part3a-2 两份的16行判词拼在一起自己核一遍，不是任何一次单独调用能做到的。
