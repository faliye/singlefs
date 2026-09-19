# m2-s1-r1 本地辩方（H7）逐句核对表

核对对象：`research/prompts/m2-s1-r1-local-defense.md`。行号是这份核对表自己的，
提示文件的行号见下表「英文项」列后括注的行区间（现查）。

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| F1（提示第 21–26 行） | `.claude/kb/decisions/23-journal的角色与格式.md:974` 「取甲：每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录。祖先不延后。」（脏叶是全部脏叶，不只被 fsync 的那个文件的：fsync = 一次全量发布） | 无 | 逐句对应：写脏叶/全部祖先/根槽/一条记录/祖先不延后/括注「脏叶是全部脏叶……一次全量发布」全部译出，未加未减 |
| F2（提示第 28–37 行） | `research/prompts/e155-preregistration.md:814-816` 甲的「怎么做」段（整段） | 无遗漏实质内容；丢弃「次序：单元→屏障→记录→屏障→根槽FUA→超级块（1831–1853）」一句 | 数据单元/两树脏节点/四样固定点/记录/根槽/超级块槽逐项译出；写序一句判定与本轮字节计算无关，未译，非误译 |
| F2 括注（tree-of-trees index 的说明）| 无单一出处，是主 agent 对「树表」这个名字的功能性说明 | — | 加了一句「that records which physical block currently holds the root of each of the other trees」——这是解释性括注，不是引文；根据是 F7 那句「改了根的树……是相邻 4 条」（同一份文件 799 行）能推出树表记的是「哪些树的根换了」，但树表本身「记的是哪个块」这一措辞没有单独一句原文可整行抄，判定为必要的功能性解释，不是事实断言 |
| F3 开头（提示第 39–40 行）「written in the strongest form its own supporters would recognize」 | `research/prompts/e155-preregistration.md:818`「wal_full（E16 的 `wal_full`，写成支持它的人认的样子）」；`research/prompts/_m2-s1-r1-background.md:69`「问二，wal_full 在本工程的最强形态。写成支持它的人认的样子」 | 首稿只查到 818 行「写成支持它的人认的样子」，未核到「最强形态」四个字的出处 | 加「最强形态」译作 strongest form，出处补在背景材料第 69 行，两处合看才对得上「strongest」这个词；单看 818 行本身没有「最强」二字 |
| F3 主体（提示第 40–51 行） | `research/prompts/e155-preregistration.md:819-820` 整段 | 丢「次序：单元→屏障→记录→屏障」一句（同 F2，与字节计算无关）；checkpoint 自己的记录数公式简化为「plus journal records for what it wrote」，未译出「⌈这次写出单元数/67⌉」的上取整公式 | checkpoint 的字节总量本轮直接引用产物里的真实数（表 B），不靠模型重新按公式推导 checkpoint 字节，所以这处简化不影响本轮要答的题；仍在这里如实记录，不算「查过」 |
| F3「不写根槽、不写超级块槽」后半 | `research/prompts/e155-preregistration.md:819`「（tail 只在 checkpoint 推进，D23 已定项 18；恢复不先信 tail）」 | 丢了这句括注的依据 | 只译了结论「不写根槽、不写超级块槽」，没有译为什么；依据与本轮字节计算无关（那是恢复算法的正确性依据，不是字节数依据） |
| F4 整段（提示第 53–68 行），除引文外 | 无单一出处，是主 agent 自己的论证：合法性论证依据 F1 的原文用词（「every fsync call」只管这次调用自己的数据可达，没说要单开一次发布），可行性框架取自 `research/prompts/_m2-s1-r1-background.md:83`「多个并发 fsync 可以并进同一次发布（组提交），这不改条款」 | — | 这一段标为主 agent 自己的推理，不是逐句翻译；已在核对表这里单列，不冒充引文 |
| F4「Speeds up many-threaded, many-dir operations by 30x or more」 | `research/prompts/_axis2-background.md:39-40`「jbd2 注释逐字：组提交『Speeds up many-threaded, many-dir operations by 30x or more』，2026-08-28 现查 `fs/jbd2/transaction.c:1928-1932`」 | 首稿只摘了「by 30x or more」半句，把「Speeds up many-threaded, many-dir operations」这半句意译成了「many-threaded workloads」，等于把一句英文原文当成了转述——违反「引原文整行抄」 | 已用 `replace-once.py` 改成整句原文「Speeds up many-threaded, many-dir operations by 30x or more」，不再意译 |
| F5 主体（提示第 70–77 行） | `.claude/kb/decisions/25-目标负载优先级.md:71`「组提交能不能救 \| 能，与加大批同因——攒进一批的操作共享同一条脊柱 \| 不能。组提交省的是根槽与记录，而根只有一个；脊柱条数一条不减」（表格形式，列头见同文件第 67 行「脊柱共享型（seq）」「脊柱不相交型（multistream）」） | 无遗漏；表格转prose时把两栏列头（脊柱共享型/脊柱不相交型）并入条件从句 | 「能」「不能」两支各自的条件都保留，未删未增 |
| F5「a spine means the leaf-to-root ancestor path...」 | 无单一出处，是「脊柱」一词在全轮材料里通用的含义，参见 `research/prompts/_m2-s1-r1-background.md:1558`「一次操作只触一条脊柱」上下文 | — | 加的定义句，因为「spine」这个英文词本身不带含义，不加这句模型无法使用 F5；判定为必要的术语解释，不是事实断言 |
| F5 末句「this finding was written before the four fixed-point trees existed...」 | 无出处，是主 agent 自己的限定 | — | 加这一句是为了不让模型把 F5 误读成「四样固定点也不能被组提交摊薄」——原文只讲根槽与记录、根本没提四样固定点，這是本轮新加的结构；已在此处标明是主 agent 的限定，不是引文 |
| F6 主体（提示第 83–90 行） | `research/prompts/e155-preregistration.md:798`「记账树：15 行全部重写（D5 已定项 2「每行每发布重写」），全部节点脏。」；`research/prompts/e155-preregistration.md:767` K5 行「记账 = 3 + 6D = 15（池级 3、每盘 6，与快照数无关）」 | 无遗漏 | 3+6D=15、池级3/每盘6、全部重写、与快照数无关，逐项译出 |
| F6「never grows with the number of files」 | 无单一出处；公式 3+6D 本身不含 P（池内文件数）项 | 首稿把「与快照数无关」直接扩写成「与文件数、快照数都无关」 | 「与文件数无关」是从公式本身（没有 P 项）推出的，不是原文逐字这么写的；判定为可从公式直接验证的推论，标注在这里而不是当成一条引文 |
| F6「so it fits in a single small node」 | `research/prompts/e155-preregistration.md:763` K1 行「记账叶 477」（叶容量 477 条） | 首稿没有这一句的出处 | 15 远小于叶容量 477，「单节点」是可核算的推论，不是引文；本轮提示文本里没有单列 K1 这一行给模型，模型如果要验证这句只能先信任它 |
| F7 整段（提示第 92–97 行，已用 replace-once 改过一次） | `research/prompts/e155-preregistration.md:799`「树表：改了根的树（extent、inode、分配记录、记账）是相邻 4 条，一个连续组。」 | 首稿写成「has at most one entry per content tree」，误把「一次发布最多脏 4 条」读成了「树表总共只有 4 条」；已发现并改写 | 改稿把「树表总条目数固定且小、与池大小无关」和「这次发布脏的那几条是相邻 4 条一组」分成两句，不再暗示树表总条目数就是 4；「总条目数固定且小」这半句本身也是推论（K5 行「树表 = 7」给出总数是 7，不是 4），已在这里记明依据 |
| F8 整段（提示第 98–107 行） | `research/prompts/e155-preregistration.md:800-801`「映射树：key = (类, 出生树, 出生 txg, …)……每个新写的受映射单元在它那个区间末端插一条（连续组）；每个旧版删一条，位置按下面的位置策略。分配记录树：key = (设备, 槽号)……」 | 无遗漏实质结论（两棵树按 key 区间插删、旧版位置看位置策略）；具体 key 的构成字段（出生树/出生 txg 等）未逐字译出 | 判定字段构成与本轮要答的题（组提交下按 k 合并是否减少节点数）无关，只译了「行为像 extent/inode 树、能不能省要看 k 次触达的位置是不是相邻」这个结论 |
| F9 整段（提示第 109–115 行） | `research/prompts/e155-preregistration.md:838`（阳性对照表，F1/P=1/seq/N=16/主几何/L均 那一格：「甲 fsync 行 − wal_full fsync 行 = 四样固定点 × 2 盘 + 根槽 + 超级块槽 × 2」「139 776」） | 无遗漏 | 差值构成与数值 139776 都逐字对应；已用 Table A row A（344576）与 Table B row A N=1（204800）核过差值 = 139776，一致 |
| F10 整段（提示第 117–123 行） | `.claude/kb/decisions/23-journal的角色与格式.md:990`「轴一那四条依据（环截断不了 / 多一条挂载期重放路径 / journal 进验证链 / 屏障一个没省）」 | 原文只给四个短语，没有逐条展开；F10 只译短语本身，展开放在 F11 | 四个短语按原序号 1–4 对应译出：ring cannot be truncated / extra mount-time replay path / journal enters verification chain / no barrier saved |
| F11 整段（提示第 125–134 行） | `research/prompts/_axis2-background.md:14`「轴一已定：每次 fsync 发根。依据：不发根 ⇒ 环截断不了（实测环峰值 48 B vs 10⁴–10⁵ B）、必须多一条挂载期重放路径、journal 进验证链；而屏障次数两侧各 2 次，一个都没省」 | 无遗漏；已在正文与此处两次注明这是另一轮背景材料，不是本轮决策记录本身，未独立复核 | 48 B、10⁴–10⁵ B、屏障两侧各 2 次，数字与结论逐条对应；括注明确标出处provenance 与未核实状态 |

## 数字表核对（不是转述，是抄数；来源文件与行号见下）

| 提示里的表格 | 来源文件:行 | 核法 |
|---|---|---|
| Table A 全部 4 行（提示第 142–145 行） | `research/results/e155-fsync-write-volume-2026-09-17-stage4.out:10`（A）、`:106`（B）、`:34`（C）、`:130`（D），均取 `policy=Lbalanced n=1` 那一行 | `jia_fsync_bytes`/`jia_calls`/`jia_barriers`/`jia_fua` 四个字段逐字段抄，未换算；四行数值与背景材料 `research/prompts/_m2-s1-r1-background.md` 第四节表（该表标注 n=16，但 `jia_*` 各字段在同一格的 n=1/16/256/4096 四行里恒定，已用命令核过）一致 |
| Table B 全部 16 行（提示第 157–172 行） | `research/results/e155-fsync-write-volume-2026-09-17-stage4.out`：A 行 10–13，B 行 106–109，C 行 34–37，D 行 130–133（每格 n=1/16/256/4096 各一行） | `write_ahead_log_full_fsync_bytes`/`_checkpoint_bytes`/`_amortized_bytes` 三个字段逐字段抄；amortized = fsync + checkpoint/N 已用四个 N 各自核对一遍算术（例：cell A n=16：204800+147968/16=214048，与产物一致） |
| 「N=16 标准参照值」一句（提示第 174–176 行） | 与 `research/prompts/_m2-s1-r1-background.md` 第 55–61 行（正文第四节表）逐格核对：F1/seq/P=1→214048、F1/seq/P=1e4→269738（产物 269738.25，四舍五入）、F8A/seq/P=1→672800、F8A/seq/P=1e4→774739（产物 774739.125，四舍五入） | 一致，背景材料表里报的是四舍五入到整数，提示里保留产物原始小数 |
| Table C（提示第 180–189 行） | `research/prompts/e155-preregistration.md:773`「`D` = 2、单元宽 16384 / 32768、记录 4096、超级块槽 4096、根槽 = `physical_block_size`（主 512，反向 4096）全部照 `crates`（第三节）」 | 六项（两盘、码 2 节点 16384、码 3 容器 32768、记录 4096、超级块槽 4096、根槽主几何 512）逐项对应同一行原文，未改动 |

## 没做什么

- 没有核对 F2/F3 里「树表」总条目数 = 7 这件事之外的 K1/K2/K3 全部字段表——本轮题目不需要模型知道叶容量等具体数字，只给了它推导 F6/F7 用得到的那几条。
- 没有把 wal_full 的 checkpoint 字节构成（三棵固定点树的具体分层字节）拆开给模型，因为 Table B 已经给了产物里的真实总数，模型不需要重新推导 checkpoint 的构成就能用来做比值。
- F4 整段（组提交在轴一条款下合法）是主 agent 自己的论证，不是某一句原文的翻译；已在上面单独一行说明，不冒充引文核对。
