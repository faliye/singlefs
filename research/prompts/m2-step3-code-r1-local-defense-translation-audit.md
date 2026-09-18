# m2-step3-code-r1 本地辩方提示：逐句核转述

核对表按 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」一节要求：
提示里每一句转述都与原文并排，缺一个限定词就补上。这里只列提示里标了 `(translated` /
`(translated from Chinese` 的那些句子；未标 translated 的句子是主 agent 自己对代码事实的
陈述，不是转述，不进这张表。

| 英文项（提示里的原句，节选） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Fact H quote 1（"before the new instance's root ... at most R = 3 times in the first version's geometry."） | `research/prompts/_m2-step3-code-r1-appendix.md:817` | 无：FUA、两块盘、不让 fsync 返回、不确认回退、至多 R=3 次五处都在 | 同首稿，未改 |
| Fact H quote 2（"the first version actually pays this cost 2 times ... The lead reviewing agent's own recommendation is also this constant-2 form."） | `research/prompts/_m2-step3-code-r1-appendix.md:818` | 略去了 FIRST_TRANSACTION_TXG = 3 与「条款在 D22 已定项 16 第 5 句」两处引用编号——它们是溯源指针，不是这句要辩护的限定词；核心限定词（归属改了要重算、R=3 是几何上界不是走到的次数、没走三方可推翻、主 agent 也推荐这一档）四处都在 | 同首稿，未改；引用编号不译，原句已用 `... D2 已定项 7 ...` 之外的省略号标出略去处 |
| Fact I（"the warm-up count for later writable mounts and for mounts after a rollback (decision D16 settled item 8 only defines the first writable mount; flagged 2026-09-16 by the forward-reasoning leg of the first three-way round)."） | `research/prompts/_m2-step3-code-r1-appendix.md:1520`（与 `.claude/kb/milestone/02-second-txn.md` 同文） | 无：后续挂载、回退之后、D16 已定项 8 的射程、2026-09-16、三方第一轮正推腿五处都在 | 同首稿，未改 |
| Fact F 里的代码注释译句（"the instance table unit ... belongs to tree 0, and is assigned its sequence number before the tree table (mkfs also does instance table before tree table)."） | `crates/singlefs-core/src/transaction.rs:1153` | 略去「码 3 打包记录类型 4：重写时容器身份照 mkfs（容器 0、出生代 0）」——这半句谈的是容器身份不是发号次序，与 Fact F 要证的「发号顺序」无关，不译不影响限定词 | 同首稿，未改 |
| Fact K（"an allocation record entry has only two possible states on disk: allocated, or released-with-a-release-generation-number"） | `research/prompts/_m2-step3-code-r1-appendix.md:447`（嵌在 C322 大段正文里，即 D3 已定项 7 原句） | 无：已分配、已释放+释放代两态都在；「盘上没法表达第三个态」这句在 Fact K 正文里另起一句给出，不是丢了而是分成两句 | 同首稿，未改 |
| Fact L（allocator.rs 分配器重建注释译句） | `crates/singlefs-core/src/allocator.rs:285-288` | 略去「空闲计数随 `mark_allocated` 减、与写路径同一条事件路径」——这句在附加问题（item 7）里用更完整的形式重新给出（`rebuild_from_records` 调用 `mark_allocated`），Fact L 本身只需要「开放段与游标只在内存、重开后不续」这半句，未丢限定词 | 同首稿，未改 |
| Fact N（"the first version has no clean-shutdown marker; reopening always goes through recovery."） | `crates/singlefs-core/src/mount.rs:4`；同句又见 `research/prompts/_m2-step3-code-r1-appendix.md:1507`（标「预想」） | 无：逐字对应，标了这是「预想」不是已定案，Fact N 正文里也写了「marked as a prediction, not yet a decision, pending the user」 | 同首稿，未改 |
| Fact P（"resumes right after the last record covered by the chosen root" + jsn 严格连续、断号即止） | `research/prompts/_m2-step3-code-r1-appendix.md:1373`（I-8.3 登记行） | 略去「跨实例边界即停」与「记录水位按实例代号为主比」——前者在 Fact Q 里用「restricted to the same instance as the root」重新给出，后者与本项要问的「所选根自己那条记录读不出」无关；核心限定词（严格连续、断号即止、接在所选根覆盖的最后一条之后）三处都在 | 同首稿，未改 |
| Fact S（"the chosen root does not yet have a published file version underneath it (tree table is empty): the first version's writable mount through this path only attaches to a pool that already has a file; a freshly-mkfs'd pool goes through the first-writable-mount path instead."） | `crates/singlefs-core/src/mount.rs:120` | 无：逐字对应，树表为空、只接有文件的池、刚 mkfs 的池走另一条路三处都在 | 同首稿，未改 |
| Item 7 quote（"reopening and rebuilding the previous version from disk: today the release-decision path reads same-process in-memory state (TransactionOutput) and carries no reader at all; ... referred to there as finding Y1's scope"） | `research/prompts/_m2-step3-code-r1-appendix.md:1520` | 无：同进程内存态、不带 reader、映射条目位置项带单元校验和、释放前要核、第二轮攻方腿 Y1 射程五处都在 | 同首稿，未改 |

## 没有走这张表的句子

Item 2、3、4、6 的正文陈述句（描述 `mount.rs` / `allocator.rs` 代码行为的那些句子）不是转述，
是主 agent 直接读代码之后自己写的事实陈述，不带 `(translated`，不进这张表；
它们的出处已经写在提示正文各条 Fact 后面的文件路径与行号里，读者可以直接去源码核对。

## 跑前修订（发现 corruption-check.py 对提示文件本身判红之后）

第一次跑 `ask-local.sh` 时，`corruption-check.py` 对提示文件本身报「粘连=7」，全部来自 Rust
的 `::` 路径分隔符（`TransactionUnit::IN_BUMP_ORDER`、`PoolAllocator::rebuild_from_records` 等）：
检测器的「标点粘连」判据要求标点前一个字符非字母数字、后一个字符是字母数字，`X::Y` 里第二个
冒号恰好满足这条，被当成「一句话被截断后接上了下一句」。这是检测器对代码语法的假阳性，
不是提示文本真的坏了：改法是把全部 7 处 `模块::函数` 写法换成散文形式（「the function
rebuild_from_records on PoolAllocator」这类），不改事实内容，只改语法形态，`replace-once.py`
逐处定点替换、全部命中 1 次。改完 `corruption-check.py` 单独跑提示文件判绿（粘连=0）。

同一轮里另外顺手把两处直接嵌入的原文汉字（"预想"、"挂载"，各 2 字，此前用
`(translated "预想")` 这种写法保留原词）换成纯英文注释（"the Chinese source word used there
literally means..."），严格贴住「给本地腿的提示一律用英文」——它们本不会被判红（cjk 计数本身
不触发红，`corruption-check.py` 明文「对任何中文文本都判红，那就没有判别力了」），换掉是为了
不留疑点，不是因为它们造成了判红。
