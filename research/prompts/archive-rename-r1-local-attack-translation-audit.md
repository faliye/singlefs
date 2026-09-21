# archive-rename-r1 本地攻方提示：逐句核对表

对照对象：`research/prompts/archive-rename-r1-local-attack.md`（英文提示，J5、J7）。
原文一列的行号在括注文件里现查（`grep -n` 核过，不是从背景材料数的）；
`research/prompts/_archive-rename-r1-background.md` 里的附录抄了同一段 `replay.sh` 摘录与
`.claude/term-rename-exempt` 全文，已与仓里的源文件逐字核对一致（见交回报告的核验命令）。

## 一、核出确有缺漏、已经改稿的三处

| 英文项（定稿） | 原文文件:行 | 首稿缺的 | 定稿怎么补 |
|---|---|---|---|
| Fact 3："prints an additional explanatory block, noting again that the byte for byte comparison tier has no counterpart to compare against for these rows." | `research/scripts/replay.sh:479`（"逐字节这一档没有对照物"） | 首稿把这句「逐字节这一档没有对照物」漏掉了，只留了 Fact 1 Step1 那一处等价说法，没有照原文在 Fact 3 里再说一次 | 补回一句 "noting again that the byte for byte comparison tier has no counterpart to compare against for these rows"，紧跟在 "explanatory block" 后面 |
| Row 4（豁免第 4 条）："sweeping this file would remove the tool's own ability to ever rename anything again." | `.claude/term-rename-exempt:8`（"扫了它就再也换不动东西"） | 首稿写成 "...to ever perform this kind of rename again"，把原文「再也换不动东西」（不加限定、换什么都换不动）窄化成「这一种改名换不动」，是首稿多加的限定词 | 删掉 "this kind of"，改成 "to ever rename anything again"，还原原文不加限定的范围 |
| Shared background：改名事件那一句，"the new spelling took effect for English identifiers starting 2026-09-20, and the renaming was completed across the rest of the repository by 2026-09-21." | `.claude/kb/term-renames.md:9,11`（"（2026-09-20 起，英文标识符与冻结目录 2026-09-21）"、"2026-09-21 定案：英文标识符、文件名与冻结目录一并改，「直到全工程都搜不出来」"） | 首稿把「冻结目录」误读成「改名这件事本身在 2026-09-21 冻结／定案」，写成 "with the rename considered frozen as of 2026-09-21"；核对原文后发现「冻结目录」是一个具体名词（被一并改名的一个目录对象），不是「改名决定被冻结」，首稿这处是理解错，不是漏译 | 改写成不再断言「冻结目录」具体指什么，只留两个日期与「扫到全仓搜不出旧名为止」这一层不会错的意思：英文标识符改名从 2026-09-20 起生效，到 2026-09-21 全仓扫完 |

## 二、故意省掉的引用指针（非限定词，判断依据没变，没有回补）

| 英文项（定稿里没有这一处） | 原文文件:行 | 首稿缺的 | 为什么不补 |
|---|---|---|---|
| Fact 2 结尾没有再引一句指向 show-me-test.md | `research/scripts/replay.sh:447`（"（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）"） | 这句括注是指向另一条规则的出处指针，不是描述行为本身的限定词；本地模型看不到那份规则，指针对它的判断没有输入 | 不补：Fact 2 已经把这句括注要支撑的道理（判「对不上」会让读者误以为实验真的坏了）整句译出，指针本身不改变 Group A 任何一题的可判性 |
| Row 3（豁免第 3 条）没有再引一句指向 CLAUDE.md「怎么改」那一行 | `.claude/term-rename-exempt:7`（"（CLAUDE.md「怎么改」那一行）"） | 同上，这是出处指针，不是「只能在上游改」这条限定本身 | 不补：Row 3 已经把「只能在上游仓改、就地改下次同步就没」整句译出，指针不影响 Group B 的分类判断 |
| Row 1（豁免第 1 条）没有点名 btrfs、F2FS、ZFS 三个具体名字，改写成 "three named systems are given as examples" | `.claude/term-rename-exempt:5`（"btrfs、F2FS、ZFS 等自己的术语与原文引文"） | 具体是哪三家没有译出 | 不补：Group B 的分类判据（X/Y/Z）只问「这个旧名是不是别家自己的术语」，不问是哪一家；点出具体名字不会改变第 5 题、第 6 题、第 7 题里任何一格的答案 |

## 三、首稿比原文多出来的限定词或括注（都是不改变判断的澄清性补写，未回撤）

| 英文项（定稿） | 对应原文 | 多出来的是什么 | 为什么加 |
|---|---|---|---|
| Fact 2："which would make a reader believe the experiment is broken **when it is not**" | `research/scripts/replay.sh:446`（"读的人会以为实验坏了"） | 原文没有「其实没坏」这半句，是上下文里隐含的 | 补一句把隐含的对照点写明，免得模型把「archived」这个标签本身读成「实验真坏了」的证据；不影响 Group A 的判据方向 |
| Row 2（豁免第 2 条）："what such a record documents is exactly the fact that a particular word was renamed **on a particular date**" | `.claude/term-rename-exempt:6`（"它记的就是「这个词被改掉了」这件事本身"） | 「在某个具体日期」是补出来的 | 呼应同一行前半句「按日期冻结」，把「按日期」这层意思在后半句里也点一次，不新增判据 |
| Row 5（豁免第 5 条）："to say clearly, **in words**, what is being exempted **from the sweep**" | `.claude/term-rename-exempt:9`（"才说得清豁免的是什么"） | 「in words」「from the sweep」两处是补的 | 把「说得清」具象成「用文字说清楚」，把「豁免的是什么」具象成「从清扫里豁免的是什么」，避免歧义，不改变分类判据 |
| Row 6（豁免第 6 条）："historical artifacts and old commits, **which still contain the old word**, can be matched back to what they mean today" | `.claude/term-rename-exempt:10`（"历史产物与旧提交的字段才对得上"） | 「仍然含旧名」「对应到今天的含义」两处是展开 | 把「字段才对得上」这个简短判断展开成完整的因果句，方便本地模型在没有中文语境的情况下接得住，不改变判据 |

结论：这一份提示核过一遍之后，三处真正缺限定词或译错的地方（表一）已经用 `research/scripts/replace-once.py` 定点改进
`research/prompts/archive-rename-r1-local-attack.md`（改后与原文逐字核对过，见交回报告里的命令与输出）；
其余的省略与补写（表二、表三）都不改变 J5、J7 两组问题的可判性，照实列出、不回补。
