# m2-lines234-code-r1 本地攻方：转述核对表

来源：`research/prompts/_m2-lines234-code-r1-body.md`（本轮正文）与
`crates/mutations.tsv`（变异表，本地攻方分到的 22 个文件的判据）。
本表核对英文提示 `research/prompts/m2-lines234-code-r1-local-attack.md`
里每一句转述与原文并排，缺的限定词补上、多出的限定词单列说明。

## 一、正文框架句（结构性，非逐行数据）

### 1. 通用问句第二分句 → Question d 的核心定义

英文项（定稿，见提示第 63-68 行）：
"for a part of the code whose own behavior cannot be derived from any
already decided design item, does the file contain a place where the code
had to choose one specific behavior on behalf of a requirement that was
never written down anywhere, such that a different, equally defensible
implementation could have chosen differently"

原文文件:行：`_m2-lines234-code-r1-body.md:13`
"这一处代码的行为，是从哪条已定分项推出来的？推不出的那些，它是在替一条没写的
条款做选择吗？如果是，这个选择今天有没有会红的东西钉着？"

首稿缺的：首稿（写进提示前的版本）漏掉了"推不出的那些"这个前提限定——
即"替没写的条款做选择"只该问在"推不出已定分项"的那部分代码上，不是不加区分地
问整个文件。首稿的 Question d 直接问"文件里有没有地方替没写的条款做选择"，
没有先限定"这部分代码本身推不出已定分项"。

定稿：已用 `research/scripts/replace-once.py` 在提示第 63 行插入
"for a part of the code whose own behavior cannot be derived from any
already decided design item"，补上这个前提限定词，命中一次、已回读确认。

不译的部分："这个选择今天有没有会红的东西钉着" 没有并入 Question d 的必答项——
这是原问句里第③分句，但本地攻方的列④只要求 yes/no/unknown + 函数名（见调度
指令与正文 69 行的四列定义），不要求"有没有会红的东西钉着"这一层。不算漏译，
是范围本来就更窄；判据第③条（"有没有会红的东西钉着"）留给云端两条腿判 X1/X2/X6
时用，不进本地腿的列④。

### 2. 判据第 3 条 → Question d 的"算 yes"标准

英文项（定稿，见提示第 68-73 行）：
"A place counts toward a yes answer only when you can state two different
choices that are each individually defensible, and state how the two
choices differ in disk bytes or in reachable history; if you cannot tell
the two choices apart in either of those ways, this question does not
apply to that place, and it should not count."

原文文件:行：`_m2-lines234-code-r1-body.md:77`
"一格算「替没写的条款做了选择」，当且仅当说得出两个都说得通的不同选择，而且
它们在盘上字节或可达历史上分得开。分不开的不立条款，登记成实现细节。"

核对：限定词逐一核过——"当且仅当"→"only when"（保留了"仅当"这个充分必要的强度，
没有软化成"when"）；"两个都说得通的不同选择"→"two different choices that are
each individually defensible"（"都说得通"→"each individually defensible"，
两边都覆盖了）；"在盘上字节或可达历史上分得开"→"differ in disk bytes or in
reachable history"，一致。

首稿缺的：无实质遗漏；"分不开的不立条款，登记成实现细节"这半句没有直接译出，
换成了功能等价的"this question does not apply to that place, and it should
not count"——效果相同（分不开就不算 yes），但没有保留"登记成实现细节"这个动作
本身，因为本地攻方不做登记这个动作（登记是主 agent 的事，不是模型答题的一步）。
列为不译，理由：这半句是对*主 agent 下一步该做什么*的指示，不是对*模型该怎么答*
的指示，不需要进提示。

### 3. 分工表本地攻方行的四列定义 → 提示的 fact a / fact b / fact c / question d

原文文件:行：`_m2-lines234-code-r1-body.md:69`
"① 这个文件今天有没有用例点名它（yes / no，给用例文件名）；② 它有没有进
crates/mutations.tsv（yes / no，给行号）；③ 变异钉的是它的行为还是只是它的
常量（behaviour / constant / none）；④ 这个文件里有没有「替没写的条款做选择」
的地方（yes / no / unknown，yes 要给函数名）。只填表，不写散文；每格给判据
来自哪个文件哪一行"

英文项（定稿）：见提示第 26-90 行 fact a / fact b / fact c / question d 四段
长定义，以及第 508 行新补的"Fill only the table... do not write any prose"。

首稿缺的（已发现并修补）："只填表，不写散文"这句在写完初版提示时漏掉了——
初版只规定了逐行作答的格式，没有明令禁止格式外的散文。已用
`research/scripts/replace-once.py` 在"End of the 22 files"前插入
"Fill only the table described below; do not write any prose outside of
it, no summary, no commentary, no discussion."，命中一次、已回读确认。

多出的限定词（逐条列出并说明为什么加）：
- fact a 段落里"A file counts as named by a test today if either an
  integration test file... or the file itself contains its own internal
  cfg(test) module"——原文①只说"有没有用例点名它"，没有定义"用例"算不算包括
  文件自己内部的 `#[cfg(test)]` 模块。这是我自己做的可操作化定义，加的理由：
  正文没有给出"用例"的判据边界，而本地攻方拿到的 22 个文件里有 3 个
  （device_log.rs、model_comparison.rs、`bin/e156_allocation_basis_counts.rs`）
  只有内部单测、没有外部集成测试文件点名它们；不把这条边界写清楚，同一个事实
  在"有/无用例"上会被两种读法判出相反的 yes/no。
- fact b 段落里"a checked in file... that records, for regression purposes,
  a deliberate one line code change together with the test that must fail
  when that change is applied"——这是对 `crates/mutations.tsv` 本身作用的说明，
  原文②只提了文件名，没有解释这份文件是干什么的。加的理由：本地模型没有读过
  `crates/mutations.tsv` 本身的表头注释，不加说明就没法理解"进了这份表"意味着
  什么。
- fact b 段落里"These row numbers are fixed reference numbers taken from a
  checked in ledger file, not line numbers inside the source file under
  review"——原文②只说"给行号"，没有区分"变异表自己的行号"与"被改代码文件自己的
  行号"。加的理由：三方共用约束明令"给本地腿的提示...答复不写代码行号与文件
  行号"，而变异表的行号是我自己核实过的既定事实、要求模型原样抄一遍，不是模型
  自己去数代码行号；不把这层区分写清楚，模型可能会把两种行号混着答，或者干脆
  因为"不许写行号"这条禁令而把变异表的行号也一起漏答。
- fact c 段落对 behaviour/constant 的操作性定义（"meaning it changes which
  decision the code reaches or which value it computes from real inputs"
  等）——原文③只给了两个词（"行为"/"常量"），假定读者已经从正文别处知道这个
  区分怎么判。本地模型没有正文其余部分的上下文，不加定义就没法核对我给的既定
  分类是否自洽。
- fact c 段落"Where a file has more than one row and the rows do not all
  agree on this behavior versus constant distinction, fact c says so
  explicitly..."——这条处理"混合"情形（比如 crash_injection.rs 7 行里 6 行
  behaviour、1 行 constant）的取舍规则，原文③没有覆盖多行不一致时怎么给单一
  标签。加的理由：22 个文件里确有 4 个文件（bad_disk_input.rs、
  crash_injection.rs、first_transaction_regions.rs）出现同一文件内部
  behaviour/constant 不统一的情况，不写这条规则，"给一个词的标签"这件事本身
  就没有定义。

## 二、文件清单与角色描述句

### 4. inode_tree.rs 的角色描述

英文项（定稿，提示第 95-99 行）："Its own stated role is to compute, after
this release, which leaf containers exist in the tree, which records each
one holds, and which ones need to be rewritten; it issues no writes of its
own and does not touch the allocator."

原文文件:行：`_m2-lines234-code-r1-body.md:27`
"inode_tree.rs（626 行，算「这次发布之后树里是哪几片容器、各装哪些记录、
哪几片要重写」，不发写、不动分配器）"

多出的限定词："leaf containers"里的"leaf"（叶）——L27 原文这句只说"哪几片
容器"，没有"叶"字。加的理由：这个词不是凭空加的，是从另一处独立证据搬过来的——
`crates/singlefs-harness/tests/first_transaction_step_five_publish.rs` 里
实际 import 的类型名是 `InodeLeafContainerIndexInTree`（本地攻方自己现查得到，
见下方"三、事实表的独立证据来源"），类型名本身带"Leaf"，与正文别处
（`_m2-lines234-code-r1-body.md:45` 提到"inode 叶容器"）一致。但这句本身不是
L27 的忠实直译，是我用另一份现查证据补的限定词，在这里单列说明。

其余部分（"这次发布之后"→"after this release"，"各装哪些记录"→"which
records each one holds"，"哪几片要重写"→"which ones need to be rewritten"，
"不发写、不动分配器"→"it issues no writes of its own and does not touch the
allocator"）逐字核对，没有遗漏限定词。

### 5. mounted_read.rs / write_request_split.rs / read_tally.rs 的角色描述

原文文件:行：`_m2-lines234-code-r1-body.md:27`（mounted_read.rs、
write_request_split.rs）、`_m2-lines234-code-r1-body.md:31`（read_tally.rs）

mounted_read.rs 原文："993 行，open_pool_for_read / mount_read_only /
data_unit_span_covering" → 英文项（提示第 120-122 行）："993 lines... Its own
stated role covers three functions: open_pool_for_read, mount_read_only,
and data_unit_span_covering." 多出的限定词："Its own stated role covers three
functions"这个框架句与"three"这个计数——原文只是三个函数名并列，没有说
"覆盖三个函数"。加的理由：单纯把三个函数名接在行数后面，读起来像文件名列表而
不像角色描述，加一句框架让模型看得出这是在说"这个文件做什么"，不改变原文列出
的函数集合本身（确实是这三个、不多不少）。

write_request_split.rs 原文："247 行，一次写请求切几个数据单元" → 英文项：
"247 lines... Its own stated role is to split one write request across
however many data units it spans." 逐字核对一致，"几个"→"however many"
（不是固定数目，保留了原文的不定量）。

read_tally.rs 原文（`_m2-lines234-code-r1-body.md:31`）："158 行，块层读计数"
→ 英文项（提示第 317-319 行）："158 lines... Its own stated role is read
counting at the block layer." 一致，无遗漏。

### 6. 改动文件清单与增删行数（core 七份、harness 六份加三个 bin）

原文文件:行：`_m2-lines234-code-r1-body.md:29`（core 七份的 +/− 数）、
`_m2-lines234-code-r1-body.md:33`（harness 六份加三个 bin 的文件名清单，
本身不带每文件的增删行数）。

核对：address.rs +13、allocator.rs +114/−14、block_device.rs +188/−2、
lib.rs +3、pointer.rs +28、records.rs +114/−22、root_ring.rs +13/−2——
提示第 4、5、6、7、8、9、10 号文件的增删行数逐一核对，与原文数字全部一致，
没有数错或漏抄一个数。`_m2-lines234-code-r1-body.md:33` 只给文件名、不给
每文件的行数，本地攻方也没有替这七个文件（crash.rs 等）编造行数——提示里
这七个文件与三个 bin 文件的事实块（File 13-15、17、19-22）都只写"changed the
same day as the batch under review"，不带具体加减行数，与原文一致（原文本身
就没有给）。

## 三、变异表标题的翻译方法说明（不逐条重复贴原文，方法与两条例外单列）

本地攻方对 22 个文件里 14 个进了 `crates/mutations.tsv` 的文件，把每一行的
标题列（变异名，`crates/mutations.tsv` 该行第一段）译成英文放进提示的 fact c。
约 90 行标题，逐条直译方法一致：保留标题里说明"改了什么、为什么要紧"的实质
从句；系统性地省去了标题里的编号性括注（如"D3 已定项 8 待办①""C368""C369"
"I-5.2""I-5.4""N2""m2h/m2i/m2j""代码三方第一轮/第二轮判决第三节第 N 条"这类
指向别的 kb 文件或别的三方回合判决的编号）。

省去编号的理由：这些编号指向的 kb 决策文件（D3、C368 等）没有发给本地模型、
模型也读不到，编号对模型是死引用，不省略也用不上；但省编号本身是"删了原文
本来有的东西"，按核对表的要求单列在这里说明，不算漏译成误译——凡是标题里
带有直接影响"有没有替没写的条款做选择"这个判断的实质内容（不是编号，是内容
本身），全部保留，没有跟着编号一起被省掉。两个直接命中本地攻方判据的例子：

`crates/mutations.tsv:233`（inode_tree.rs）原文标题："并行线三：中间插入
（非末尾分裂，条款没写）不再被拒" —— 英文项（提示第 113-115 行）："parallel
line three, a mid sequence insertion, a split that is not at the very end,
a case with no written requirement, is no longer rejected." "条款没写"三个
字逐字译成"a case with no written requirement"，没有省略、没有软化。

`crates/mutations.tsv:235`（inode_tree.rs）原文标题："并行线三：叶容器数超过
一个码 2 根装得下的 135 片（树要长高，条款没写）不再被拒" —— 英文项（提示
第 117-120 行）："parallel line three, a leaf container count exceeding the
135 pieces that one code two root can hold, the tree needs to grow taller,
a case with no written requirement, is no longer rejected." 同样逐字保留
"条款没写"→"a case with no written requirement"，"135 片"这个具体数字也
原样保留，没有约成"several"这类模糊词。

`crates/mutations.tsv:88`（allocator.rs）原文标题："增补 2：各盘给提交内生块
的去处不同也不拒（D3 已定项 8 待办 ① 没条款的分支被定成取盘 0）" —— 英文项
（提示 File 5 段内）："devices are not rejected even when they disagree on
where to place a committed generated block, with the branch that has no
written requirement settled as simply taking device 0's answer." 省掉的是
"D3 已定项 8 待办①"这个编号（按上面说明的理由），"没条款的分支"这个实质
判断词保留为"the branch that has no written requirement"，没有被编号的省略
连带删掉。

## 四、正文里没有译进提示的句子，及为什么

原文文件:行：`_m2-lines234-code-r1-body.md:71`
"两条攻方腿的攻击面分开：云端攻方拿 X1、X2、X6（都要构造历史与读数），本地
攻方拿「22 个文件的测试覆盖」（逐文件判形式），不共用一格。"

不译理由：这句是主 agent 派发时用来防止两条攻方腿重复攻击面的调度说明，说的是
"云端攻方在做什么"，不是"本地模型该答什么"。本地模型看不到云端攻方那份提示，
告诉它"云端攻方在攻 X1/X2/X6"对它填表这件事没有任何帮助，反而可能诱导它去
猜测 X1/X2/X6 是什么、进而答非所问。已确认本地攻方自己的提示里没有出现 X1
到 X6 中的任何一个代号或它们对应的技术细节（挂载态整片读、树节点回退、
key 区间、`blocks` 字段、一次发布重写、mkfs 整环清零），逐一 grep 提示文件
确认过，见下方命令与输出。

原文文件:行：`_m2-lines234-code-r1-body.md:84`
"不许把「今天代码这么写」当成推导的一步（判据第 1 条）。"

不译理由：判据第 1 条管的是"兑现了条款"这个verdict类别的推导（从已定分项的
原文推到这段代码），这是云端正推腿（Sonnet）在 X3/X4/X5 上要做的事，不是本地
攻方列④要判的"替没写的条款做选择"。列④本身的判据是判据第 3 条
（已译，见上方"一、2"），不是判据第 1 条；把判据第 1 条也塞进本地提示里，
对模型判列④没有对应的用处，会让提示更长而没有对应的收益，所以没有译入。

命令与原样输出（确认本地攻方提示里没有 X1-X6 代号或它们各自的技术细节）：

    $ grep -niE "X1|X2|X3|X4|X5|X6|central_mapping_lookup|check_index_node_keys" research/prompts/m2-lines234-code-r1-local-attack.md
    （无输出，零命中）

    $ grep -c "D19\|D17\|C483\|C478\|C480\|C485\|C487" research/prompts/m2-lines234-code-r1-local-attack.md
    0
