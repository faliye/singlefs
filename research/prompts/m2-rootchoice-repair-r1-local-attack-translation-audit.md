# 转述核对表：m2-rootchoice-repair-r1-local-attack（2026-09-22）

逐句核对 `research/prompts/m2-rootchoice-repair-r1-local-attack.md`（K4：C335 链条怎么不再是吸收态；
K6：K2–K5 四格改法会不会互相打架）里每一条 Fact / Claim / Label 与中文原文。kb 行号现查各自文件
（工作区 2026-09-22 版本；正文行号按 Read 工具显示的 `research/prompts/_m2-rootchoice-repair-r1-body.md`
自己的行号）。格式：英文项 / 原文文件:行 / 首稿缺的或改动的 / 定稿理由。

提示不引用具体文件名加行号（按共用约束「英文提示里的每一句转述」一节与派发提示「答复不写代码行号与
文件行号」），只在这份核对表里写死来源文件与行号。

本轮两处发现的真实翻译错误（不是压缩，是语义错误），已在写提示的同一遍核对里发现并改正，登记在下表：

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| Fact 2："写行之前不推抬 F 的空发布"一句 | `.claude/kb/decisions/18-块里携带什么信息.md:304` | 首稿把「抬 F」（回退下界 F，`D16（发布语义） 已定项 1`「回退下界 F」）误译成"the recovery watermark"——这个代码库里"watermark"另有所指，是 `D23（journal 的角色与格式） 已定项 14` 的 W（实例切换里被照旧的最大事务号，Fact 14 用到），F 与 W 是两个不同的量，混用会让读这份提示的模型把两件事当成一件事 | 改成"no empty publish meant to raise the rollback floor is pushed before the row is written"，不再用"watermark"这个词指代 F |
| Fact 5：切换预留公式 | `.claude/kb/decisions/28-挂载期承诺量.md:73` | 首稿漏了公式里的 `max(1, ⌈...⌉)` 外层，只译了 `⌈(rows0 + N_switch) / 每片行数⌉`——这条已标注"quoted in full"，漏掉外层的 max 不是压缩，是抄漏一层 | 补上"whichever is larger of one, or the ceiling of..."，恢复公式原有的两层结构 |

## Fact 1–16 逐条核对

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| Fact 1（C335 整行，取「简称」「定义」「前置」三列） | `.claude/kb/checks-owed.md:292` | 未译该行「怎么拦」「出处」两列（多次挂载录制流的现状、故障注入开关缺口、2026-09-14 三轮辩方腿与用户弹窗的出处） | K4/K6 的判断只用得到「链条本身」与「前置里明写的空白」，「怎么拦」是给将来测试脚本看的、「出处」是溯源用的，两者都不改变链条本身的真假；两列的缺失已在这里登记，不是无声消失 |
| Fact 1，前置列后半句 | `.claude/kb/checks-owed.md:292` | 未译「多次挂载的录制流（2026-09-17 起有了……还缺按 (区域, 槽) 让根槽读返回失败的开关）」 | 这是测试装置就绪度的事实，不是设计条款；正文第五节已写死「这一轮不许把『跑一段历史给我看』当结论的必要条件」，装置缺口不影响 K4/K6 要判的链条与冲突本身 |
| Fact 1，末句「跑前写死的另一个形态」 | `.claude/kb/checks-owed.md:292` | 无遗漏，逐字译出「读不出的槽持续到它被轮到覆写为止按设备维处理」与「用户没选」两层 | K4 判断二明令不许把这个形态当断开链条的答案，必须先把它完整地给模型 |
| Fact 2（D18 已定项 11「行怎么写」） | `.claude/kb/decisions/18-块里携带什么信息.md:304` | 见上方「真实翻译错误」表；另外「实例 0 不写」「[max(所选根的实例, 1), 新实例) 的行」两处半开区间记号原样保留，未简化 | 半开区间对 K4 claim C（rows0 单调增）与 Fact 3 的回收条件配合使用，保留精确记号 |
| Fact 3（D18 已定项 11「回收」） | `.claude/kb/decisions/18-块里携带什么信息.md:310` | 「量词取『池中每一个』不取『该实例写过的每一个』」那半句未译（解释为什么用池级量词，不用实例级量词） | 这半句是量词选择的理由，不改变量词本身（池中每一个）在 Fact 3 里已经原样译出；理由句对 K4 claim A/B 的判断不是必需输入 |
| Fact 3，temporarily-unreadable 那句 | `.claude/kb/decisions/18-块里携带什么信息.md:310` | 原文这半句本身在中文里就绕（暂时读不出的槽先允许行被删、槽恢复后被抛弃的根重新进候选集、而它引用的单元已被清扫抹头），首稿按时间顺序译成「先在暂时读不出期间删行，槽恢复后根重新进候选集，但单元已被抹」，与原文语序一致 | 未改变因果关系，只是把原文本就隐含的时间顺序显式化；这一句本身是另一笔欠账（C314）的线索，不是 C335 链条的一环，不影响 K4/K6 的判断 |
| Fact 4（D23 已定项 14，根槽写失败重发） | `.claude/kb/decisions/23-journal的角色与格式.md:370` | 无遗漏，「同一个根槽连续失败不再是一条判据」「留着它读起来像一道闸、实际是空的」两句均译出 | 按字面译 |
| Fact 5（D28 已定项 3，链重写公式） | `.claude/kb/decisions/28-挂载期承诺量.md:73` | 见上方「真实翻译错误」表 | 已改正 |
| Fact 6（D28 已定项 3 射程，吸收态与近满盘） | `.claude/kb/decisions/28-挂载期承诺量.md:79` | 只译了与 C335/近满盘相关的那一句，未译射程段前半句（「式子按两份副本各落一块盘写……三块盘以上时两份落哪两块盘没写」，这是另一笔欠账 C126 的射程） | 三块盘落点的问题与 K4/K6 判断的对象（吸收态怎么不再吸收、四格改法冲不冲突）无关，不引入 |
| Fact 7（D2 已定项 13，挂载准入表） | `.claude/kb/decisions/02-RAID条带策略.md:224-225` | 未译「可写设备数」的口径定义（探针写失败即算不可写，不按失败次数或时长算） | 这条口径定义的是「w 的下限」怎么数，K4/K6 都只用到「预留拿不到 ⇒ 只读挂载」这一条合取本身，不需要「可写设备数」的操作定义 |
| Fact 8（C331，压缩摘要） | `.claude/kb/checks-owed.md:288` | 标注为「compressed summary, not a full quotation」；未译「D23 已定项 14 注 3……会让新实例的第一条记录落在读不出那条根的记录槽上，把它唯一的记录见证盖掉（今天的规则不拿那条记录当见证，所以今天看不出后果）」这一句 | 这句是 C331 定义列里连带提到的另一个后果，与 K6 要查的「候选之间碰不碰同一个字节」无直接关系（它说的是记录槽，不是候选修法本身要碰的字节）；Fact 9 已经把 D23 已定项 14 注 3 的正文单独、完整地给出 |
| Fact 9（D23 已定项 14 注 3） | `.claude/kb/decisions/23-journal的角色与格式.md:360` | 无遗漏 | 按字面译，含「已定项 9 的 48 位本来就按全卷寿命算」被略去——这句是位宽算术的背景，不是本注的规则本身，K4/K6 都用不到 |
| Fact 10（D16 已定项 6 骑手条款 1） | `.claude/kb/decisions/16-发布语义.md:151` | 无遗漏 | 按字面译 |
| Fact 11（C332，压缩摘要） | `.claude/kb/checks-owed.md:289` | 标注为压缩摘要；未译「或落回 R_old 并施加 R_old 之后被抛弃的记录，回退被静默撤销、之后已确认的写丢掉」与「与暖机只防单故障同一类」两句 | 第一句是具体后果（写丢失），已用「the rollback... cannot help」概括，未逐字展开「写丢失」这个结果；这处压缩在此登记：K6 只需要知道候选修法碰什么字节、什么不变量，不需要这一格本身的完整故障后果链 |
| Fact 11，候选列表 | `research/prompts/_m2-rootchoice-repair-r1-body.md:48` 与 `.claude/kb/checks-owed.md:289` | 三个候选中，前两个（读实例表 / 另给见证）两处都有；第三个（维持今天、登记双故障不保）只在正文出现，checks-owed 原句只写「没有条款」 | 两个来源已在此登记：候选二取自 checks-owed 的「回退要不要多一份持久见证」，候选三取自正文自己新加的第三条 |
| Fact 12（D23 已定项 14，回退崩溃残留） | `.claude/kb/decisions/23-journal的角色与格式.md:363` | 无遗漏 | 按字面译 |
| Fact 13（C334，压缩摘要） | `.claude/kb/checks-owed.md:291` | 标注为压缩摘要；「这个定义只在……第三轮攻方腿自己的模型上量过（四段剧本 0），没被攻过」译成「measured against its own author's model, in four fixed test scenarios, and has never been attacked」，「四段剧本 0」（四个场景、零发现）简化为「四个固定测试场景」 | 「零发现」这个具体数字对 K5/K4/K6 的判断不是必需——K6 只需要知道这条定义没被独立攻过，具体发现数不改变「没被攻过」这个结论 |
| Fact 14（D23 已定项 14，实例切换定义） | `.claude/kb/decisions/23-journal的角色与格式.md:371` | 无遗漏 | 按字面译，含 W 的取值与 Fact 2 里的 F 明确区分（见上方真实翻译错误表） |
| Fact 15（D22 已定项 9，系统配置字段表） | `.claude/kb/decisions/22-单元原子性怎么合成.md:186` 与 `:204` | 数字（481 / 4096 / 3615 / 147+114+128=389 / 4 / 36 / 52 及其四项拆分 8+32+8+4）逐一核对，无遗漏、无改动 | 数字是这一格判断（系统配置槽还有多少余量）的直接依据，逐字核对过 |
| Fact 16（C245，压缩摘要） | `.claude/kb/checks-owed.md:215` | 标注为压缩摘要；未译两条已定项具体怎么说反话的逐字引文（D22 已定项 9「每次发布还要更新它的槽」与 D23 已定项 3「只在 checkpoint 持久之后写一次」），只译了两者的结论（一次发布与一次 checkpoint 的頻率差、以及由此而来的陈旧代价） | 两条已定项互相矛盾的具体措辞是 C245 自己要还的账，K6 只需要「系统配置多久写一次、住在里面的逐代量会不会陈旧」这个结论，不需要复现两条已定项打架的原始措辞 |

## K4 claim A–F、K6 label 2a–5 的拆分核对

这十二项本身不是某一句中文的逐字翻译，是本地攻方腿按 Fact 1（claim A–F）与 Fact 8/11/13（label
2a/2b/2c、3a/3b/3c、5）机械拆出来的结构化条目，核对方式是「拆出来的每一项都能在对应 Fact 里找到
逐字依据，没有凭空添加内容」。

| 英文项 | 依据的 Fact | 核对结果 |
|---|---|---|
| Claim A–F 六项 | Fact 1 | 六项合起来覆盖 Fact 1 定义列里从「第一版没有根环槽的重定位」到「吸收态」的全部箭头与分句，逐项对照：claim A = 「一个根槽持续读不出时它永远读不通」；claim B = 回收条件（Fact 3）与 claim A 合取得到「行永远删不掉」；claim C = 「每次可写挂载至少 +1 行」+ claim B ⇒ rows0 单调增；claim D = 「rows0 每涨 369 行切换预留多一片」⇒「预留拿不到」（含 Fact 6 的「不再要求近满盘」）；claim E = 「预留拿不到 ⇒ 只读挂载」；claim F = 「只读挂载」+「删行要可写挂载」⇒ 吸收态。六项之外，Fact 1 定义列没有剩下未覆盖的分句 |
| Label 2a / 2b / 2c | Fact 8 | 三个候选名字逐字对应 Fact 8 里「candidate one / candidate two / candidate three」，顺序与 Fact 8 一致（对应正文与 `.claude/kb/checks-owed.md:288` 的「计数器从全环最大 + 1 起、系统配置带 txg、或记录扫描水位」这个顺序） |
| Label 3a / 3b / 3c | Fact 11 | 三个候选名字逐字对应 Fact 11 里「candidate one / candidate two / candidate three」，顺序与正文 `_m2-rootchoice-repair-r1-body.md:48`「择根读实例表 / 不读而另给回退一个盘上的见证 / 维持今天并把这一格登记成双故障不保」一致 |
| Label 5 | Fact 13、Fact 14 | 「保持已定义不变」这一句是本地攻方腿为 K6 新加的一个「改法」标签，不是某句原文的翻译——K6 需要把 K5 的定义当成参与冲突检查的一方，即使它自己没有「候选修法」列表；这一格的实质内容（切换所选根的定义）逐字来自 Fact 13、Fact 14，「保持不变」这四个字是本地攻方腿的框架措辞，已在下一节「多出来的限定词」里登记 |
| Label 4 | K4 部分自身的作答 | 不是翻译，是要求模型在回答完 K4 六个 judgment 之后，从自己给出的六个「break」答案里挑一个来参与 K6 的比对；这一整条指令本身是本地攻方腿设计的任务结构，不对应任何一句中文原文 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| 开头一段角色设定与格式要求（"You are one of three independent reviewers..."到"name the function by its name instead"） | 无对应原文 | 整段是本地攻方腿按共用约束「英文提示里不许用 markdown 强调」「答复不写代码行号与文件行号」与派发提示「本地攻方要交什么」自己写的任务框架，不翻译任何中文句子 | 这是每一份发给本地模型的提示都要有的格式约束，属于「指令」而非「转述」，不在核对范围内，但仍列出登记 |
| Background 段（"Singlefs keeps a small fixed size ring..."） | 综合多处（`.claude/kb/decisions/22-单元原子性怎么合成.md` 根环定义、`.claude/kb/decisions/18-块里携带什么信息.md` 实例表定义） | 「absorbing state」这个词本身的定义（"a design in which a filesystem can enter a state it can never leave without outside help even though every individual physical device keeps working"）是本地攻方腿给的操作化定义，原文各处只说「吸收态」三个字、不曾给出这样一句英文定义 | 本地模型没有中文附录可读，「吸收态」若不给出操作化定义，模型可能望文生义；这句定义按 Fact 1、Fact 6 里「吸收态」出现的上下文（预留拿不到⇒只读⇒删不掉行⇒出不去）反推出来，未引入 Fact 1/6 之外的新论断 |
| K4 判断二（"break"）里"Do not propose...the project's human owner already declined that option"一句 | `.claude/kb/checks-owed.md:292`（跑前写死的另一形态）与 `research/prompts/_m2-rootchoice-repair-r1-body.md:52`（「这一格要判的是：在不推翻用户 2026-09-14 那次选择的前提下，链条上哪一环可以断开」） | 「if your break for this claim turns out to be the same design in different words, say so explicitly instead of proposing it as new」这半句是本地攻方腿加的，原文两处都没有这句 | 正文只说「不推翻用户选择」，没有明说「模型换个说法提出同一个方案算不算」；这半句是本地攻方腿为了让「不许提那个方案」这条约束在换皮改写时也拦得住而加的，不改变原文对该形态本身的否决 |
| K6 group four（label 4 与 2b/3b/5 各配一次）整段 | `research/prompts/_m2-rootchoice-repair-r1-body.md:79`（「第 3 条在 K6 上触发，只要 K2 选了『系统配置带 txg』而 K3 选了『另给回退一个盘上的见证』」） | K4 candidate 参与冲突检查这件事，以及只挑「最可能碰系统配置或根环字节的那一个」而不是六个全比，是本地攻方腿自己做的取舍，正文没有这一条 | K2×K3、K2×5、K3×5 已经覆盖了正文明写的那一条判据触发点；K4 的六个 candidate 若逐一与 2a/2b/2c/3a/3b/3c/5 全交叉，是 6×7=42 对，超出这一次调用合理的规模，因此只挑一个代表性的最高相关 candidate 参与三次比对，这一取舍在提示正文里已写明（"whichever single one...you judge most likely to write to the same on disk bytes"），交回报告里也会注明这不是穷举 |

## 命令与输出（本报告依据的现查，非某句原文的翻译，留痕核验）

行号核查命令（写提示之前与写完之后各跑一次，逐一确认未因并发改动而漂移）：
```
grep -n "根记录必须带 32 位实例代号" .claude/kb/decisions/16-发布语义.md   → 151
awk 'NR==292' .claude/kb/checks-owed.md                                   → C335 整行
awk 'NR==288' .claude/kb/checks-owed.md                                   → C331 整行
awk 'NR==289' .claude/kb/checks-owed.md                                   → C332 整行
awk 'NR==291' .claude/kb/checks-owed.md                                   → C334 整行
awk 'NR==215' .claude/kb/checks-owed.md                                   → C245 整行
grep -n "行怎么写\|回收\*\*：行可删当且仅当\|表不可读时" .claude/kb/decisions/18-块里携带什么信息.md → 304 / 310 / 311
grep -n "根槽写失败重发时" .claude/kb/decisions/23-journal的角色与格式.md  → 370
grep -n "计数器全池接着走" .claude/kb/decisions/23-journal的角色与格式.md  → 360
grep -n "实例切换.*= 挂载内做一次恢复" .claude/kb/decisions/23-journal的角色与格式.md → 371
grep -n "行宽 88 ⇒ 369 行\|近满盘 \+ 坏扇区的池上" .claude/kb/decisions/28-挂载期承诺量.md → 73 / 79
grep -n "实例切换的预留拿得到" .claude/kb/decisions/02-RAID条带策略.md    → 224、233
```
全部行号在写提示前后两次核查中一致，未发生漂移。

## 历史版本

（暂无历史，这是本轮第一次交这份提示。）
