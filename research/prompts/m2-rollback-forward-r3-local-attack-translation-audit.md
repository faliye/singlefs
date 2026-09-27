# m2-rollback-forward-r3 本地攻方 译文核对表

逐句核：英文项 / 原文文件:行 / 首稿缺的 / 定稿。原文行号取自
`/tmp/claude-1000/m2-rollback-forward-r3/kb-snapshot/.claude/kb/` 与本轮正文
`research/prompts/_m2-rollback-forward-r3-body.md` 各自文件里现查的行号。

## part1（K4 三臂算术）

| 英文项（定稿） | 原文文件:行 | 首稿缺的 | 定稿改动 |
|---|---|---|---|
| Fact 1：two disks；region 0/1/0 归属；每区 8 槽 | `_m2-rollback-forward-r3-body.md:48`「两块盘、根环三区域归属 0 / 1 / 0、每区 8 槽」 | 无缺，「0/1/0」的读法（区域序号→盘序号逐位对应）由本文顺带写明，原文未直接摊开这一读法，判定为忠实展开、非缺项 | 首稿即定稿，未改 |
| Fact 1 所在段落上文「vocabulary note」一句「each region is permanently assigned to exactly one physical disk, decided once and never changed afterward」 | kb-snapshot `decisions/22-单元原子性怎么合成.md:67`「区域位置是 mkfs 时划在盘上的既成事实，落盘归属也是……加盘/换盘不再改变任何既有区域的归属」 | 这一句是背景铺垫段落，不是编号 Fact，未单独标来源；核对后确认与该行忠实对应 | 判定：**多出的引用**——补记来源于此，供核查用；未改提示文件本身（背景段不算「事实表」逐行编号项，未加编号 Fact，只在本核对表里记来源） |
| Fact 2：区域数 R = 3，并displaystyle警告「fault tolerance target 也叫 F，与回退下界 F 是两个量」 | kb-snapshot `decisions/22-单元原子性怎么合成.md:55`「区域数 R｜3（R = F + 1，F = 2）」 | 首稿即包含这条警告 | **多出的限定**：原文本身只给出公式与数字，没有明说「这个 F 与回退下界 F 是两个不同的量」；此处主动加了这句消歧警告。理由：正文与本轮多处都用 F 记回退下界，若不消歧，模型极可能把区域公式里的 F=2 与回退下界 F 混为一谈，直接影响后续算术的准确性 |
| Fact 3：F sub d = disk d 上最新持久有效根所带的 F | `_m2-rollback-forward-r3-body.md:9`「F_d = 盘 d 上最新持久有效根所带的 F」 | 无缺（newest / persistent / valid 三个限定词均保留） | 首稿即定稿；加了 currently 一词与「record」一词，均为同义补足，未改变限定范围 |
| Fact 4：根记录带 8 字节回退下界 F；平时不动，盘紧时抬；不会因为多发布而自动回落 | kb-snapshot `decisions/22-单元原子性怎么合成.md:141`「回退下界 F｜8｜……平时不动，盘紧时抬……」 | 原句只到「平时不动，盘紧时抬」，未见「不会因为多发布而自动回落」这一分句 | **多出的限定**：「It is never automatically lowered merely by publishing more root records」是本轮为使下文的「favorable / unfavorable case」建模假设成立而补的推论性总结，不是这一行的直译；判定依据是 D16 已定项 1「生效」那一行「回落机理」的一般语义（同一份 kb 快照 `decisions/16-发布语义.md:37` 附近讨论 F 不因发布本身回落），未逐字引用，标注为「本轮推论性补注，非逐字引用」 |
| Fact 5：MAX / MAX+HOLD / SYSCFG 三条公式 | `_m2-rollback-forward-r3-body.md:16`（K4 行整行） | 首稿（写入文件前的草稿）在 MAX+HOLD 一句里加了「and not relevant to the F effective value itself」，这句在原文没有对应——它等于替 Item 2 要模型自己判断的问题先给了结论，判定为不当添加 | **已删**：改为「for the purpose of computing F effective, this design is identical to design MAX; separately, this design also holds back……」，去掉那句抢答的插入语（用 replace-once.py 定点替换，命中 1 次） |
| Fact 5 MAX+HOLD 一句里「(min_d F_d 意义下的今天规则值, max_d F_d]」这一区间的下界说明 | `_m2-rollback-forward-r3-body.md:16`「另把释放代落在 (min_d F_d 意义下的今天规则值, max_d F_d] 之间的槽回收后先扣住」 | 首稿把下界直接写成「today's baseline rule's floor value」，丢了原文括注「min_d F_d 意义下」这一层——本提示没有另外定义「今天规则值」等于什么，读者（模型）无法核实这个下界具体是多少 | **已补**：改成「falls after the value that today's baseline rule would compute (that is, the minimum of F sub d over both disks)」，把原文括注的等价说明写回去（用 replace-once.py 定点替换，命中 1 次） |
| Fact 5 SYSCFG 一句「系统配置里读得出的 F 的最大值」译成「the maximum … across both disks」 | `_m2-rollback-forward-r3-body.md:16`「F_生效 = max(max_d F_d, 系统配置里读得出的 F 的最大值)」 | 原文「系统配置里读得出的F的最大值」未显式写「across both disks」 | **多出的限定**：本轮把系统配置槽按盘各自维护（fact 6 已说明每盘两槽），"读得出的 F 的最大值" 若不点明取值范围会有歧义（是本盘内两槽的最大值，还是全池所有盘的最大值）；根据 SYSCFG 公式本身要与 max_d F_d 相提并论取外层 max，判定为全池范围，补了「across both disks」以消歧 |
| Fact 6：每盘两个系统配置槽；正常情况下内容相同（除本盘设备号字段）；择槽规则（校验和过且世代号最大），择优槽校验和不过则回退到另一槽 | kb-snapshot `decisions/22-单元原子性怎么合成.md:197`（择槽规则一句）与 `:213`（两槽同内容、盘级字段一句） | 213 行原文首先说「内容是**池级**的，每盘存一份副本」（跨盘统一），这一层在 Fact 6 里没有译出，只译出了「同一块盘内两槽同内容」这一层 | 判定：**有意省略、非误漏**——「池级、跨盘统一」描述的是今天已有的系统配置字段（格式版本、fsid 等）的性质；本轮要考的 F 是 SYSCFG 臂新提议加进系统配置的字段，SYSCFG 公式本身取 `max(max_d F_d, 系统配置里读得出的F的最大值)`，隐含承认跨盘可能不一致（否则取 max 无意义）——把「跨盘统一」这条旧字段的性质套到新字段 F 头上会与本轮要考的公式打架，因此在 Fact 6 里只保留「同一块盘内两槽同内容」这一层，「跨盘统一」那一层留在这张核对表里存查，不进提示正文 |
| Fact 6「择槽规则……自动回退到另一个槽」译成「if that preferred slot's checksum does not pass, the reader automatically falls back」 | kb-snapshot `decisions/22-单元原子性怎么合成.md:197`「择槽规则（校验和过且世代号最大，已定项 16）自动回退到另一个槽」 | 原文没有明写「回退」的触发条件是「优先槽校验和不过」 | **多出的展开、非新增限定**：择槽规则本身（校验和过 且 世代号最大）已经决定了「回退到另一槽」只会在优先槽校验和不过时发生——这是同一条规则的逻辑展开，不是新加的限定，判定为忠实展开 |
| Modeling assumption（favorable / unfavorable case） | 不出自任何文件，本轮自建的建模假设，用于把「最少要坏几个」的题意钉死成一个可算的量 | 不适用（无原文） | 首稿即定稿；提示正文里明写「this assumption is not drawn from any document」，不进 Fact 编号 |

## part2（K3 去重算术）

| 英文项（定稿） | 原文文件:行 | 首稿缺的 | 定稿改动 |
|---|---|---|---|
| Fact 1：去重规则——按新到旧逐条看，只有与「已计入的、更新的」状态都不同才计入 | `_m2-rollback-forward-r3-body.md:15`「数非空根时，一条根只有在它的 (inode 树根指针, extent 树根指针) 与每一条更新的、已计入的根都不同时才计入」 | 无缺——「更新的」「已计入的」两个限定词都保留（分别对应 running list 只含「examined earlier in the newest-to-oldest pass」与「already counted」） | 首稿即定稿，未改 |
| Fact 2：比较基准是树表里的条目，不是树表单元自己的落点 | kb-snapshot `decisions/16-发布语义.md:44`「比的是树表里这两棵树的条目、不是树表单元自己的落点」 | 原句前半「一条有效根算非空 ⟺ ……与它前一条有效根……树表里的不同」（只比「紧邻前一条」）没有译入 Fact 2 | 判定：**有意省略、非误漏**——「只比紧邻前一条」是 D16 已定项 1 原本的「非空」判定读法，与本轮 K3 要用的去重规则（Fact 1，比「每一条已计入的」）是两个不同范围的规则；若把「只比紧邻前一条」也搬进 Fact 2，会与 Fact 1 的范围互相打架，误导模型改用更窄的比较范围。因此 Fact 2 只取「比较对象是树表条目、不是物理落点」这一层，另一层（比较范围）留给 Fact 1 单独定义 |
| Definition 1–4（字母记号、oldest→newest 书写顺序、回退落回原状态记同一字母） | 不出自任何文件，本轮自建的记号约定，用来让「A-D」「A-D-A」「A-D-A-D」三个记法自足可算 | 不适用（无原文） | 首稿即定稿；提示正文明写「these definitions are not quoted from any document, only the underlying mechanism they describe is」 |
| Definition 4「a rollback root record is written with the same letter as whichever earlier position … it is rolling back to」的支撑依据 | 未逐字引用；依据是「回退」本身的语义——回退把可见内容还原成 R_old 那一版，kb-snapshot `decisions/23-journal的角色与格式.md:378`「回退……从 R_old 那棵账重新载入」一带的整体上下文，以及本轮正文 `_m2-rollback-forward-r3-body.md:9`「R_old = 回退目标」「new = 回退那次发布写出的新根」的记号定义 | 不适用（推论性支撑，非逐字引用） | 判定：**推论性定义**，非逐字摘引；已在提示里用「by the meaning of rollback itself」一句明示这是按「回退」一词本身的含义推出，不是照抄某一行 |

## 小结

首稿共发现 2 处需要改的失真（MAX+HOLD 一句抢先给出 Item 2 的结论、MAX+HOLD 区间下界丢了「min_d F_d 意义下」这层等价说明），已用 `research/scripts/replace-once.py` 各定点替换 1 次改正，回读确认命中一次、已生效。另有 3 处「多出的限定」（Fact 2 的消歧警告、Fact 5 SYSCFG 一句的「across both disks」、Fact 4 的「不会自动回落」推论）与 2 处「有意省略」（Fact 6 的跨盘统一层、part2 Fact 2 的「只比紧邻前一条」层），均在本表逐条写明理由，未回改提示正文。
