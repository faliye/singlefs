# m2-newq-r1 本地攻方提示：逐句核转述

核对方式：先写一版压缩首稿，再与原文并排，把首稿丢的限定词补回定稿；多出来的限定词单列一行并写为什么加。

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Clause A（switch 主句） | .claude/kb/decisions/23-journal的角色与格式.md:377 | 首稿丢：择根用「与恢复同一套规则」这一限定（只说 re-read disk，没说 same rule as recovery's choose_root）；丢「对着仍打开的设备句柄」；丢内存根为什么会旧的触发条件（根已落盘而系统配置槽那一步失败）；丢逐实例 k 的三元组内容（只说 rows fall in range，没给 (k, txg, W) 的具体值）；丢尾句「不转只读、不等下次挂载」与代价句 | 见提示文件 Clause A 全文，六处均已补回 |
| Clause B（注 4，W 的取法） | .claude/kb/decisions/23-journal的角色与格式.md:363 | 首稿丢「已定项19①：事务号0不进W的max」这一具体依据（只说 W=0，没说为什么、没说这是哪一条已定项的规定）；丢 W 写在哪一行（写在被照旧事务的写序实例那一行）；丢「切换的所选根按实例切换那一句取」这句收尾的交叉引用 | 见提示文件 Clause B 全文；「已定项19①」的编号在定稿里略去（该已定项不在这一轮的六条条款之内，属于范围外交叉引用，只留其结论「transaction number 0 never enters the max used for W」，不留编号） |
| Clause C（切换写的行 ①-④） | .claude/kb/decisions/18-块里携带什么信息.md:308 | 首稿丢规则①覆盖的两个具体来源（写行那次的恢复行 / 回退那次的回退行与中间实例行，首稿只说「resent as-is」没展开是哪两类行）；丢规则②末尾的例外「被重发的checkpoint里没有事务号非0的记录时②一条都不适用」（首稿完全没提这个例外） | 见提示文件 Clause C 全文，两处均已补回 |
| Clause D（已发布谓词，全局） | .claude/kb/decisions/18-块里携带什么信息.md:310 | 首稿丢码1那个「或」的语义解释（在所选根里，或被重放施加）；丢码2/3 关于重放终点之后的固定点单元全部未发布、被重放施加进来的固定点单元按新根段引用判已发布（D23已定项15）这一整段；丢码3事务号不进已发布谓词、码2写序已收窄成只有实例代号这两句；丢收尾限定「无行⇒已发布只对同一时间线成立」 | 见提示文件 Clause D 全文，四处均已补回 |
| Clause E（回退候选集） | .claude/kb/decisions/23-journal的角色与格式.md:369 | 首稿把候选集压成「an old root」，丢了候选集的两个门槛（按实例表判仍然有效 ∧ txg≥F_生效）与可选性公式（(i,T)可选⟺实例表无i的行，或有行(i,Ti,Wi)且T≤Ti）、以及「被抛弃时间线的根一条都选不中」这句排除；丢回退行与中间实例行的具体元组值 (r_old, T_old, 0) 与 (i, 0, 0) | 见提示文件 Clause E 全文，三处均已补回 |
| Clause F（三件没有条款） | .claude/kb/decisions/23-journal的角色与格式.md:386 | 首稿基本完整；核对后未发现丢限定词 | 与首稿一致，仅措辞顺畅化 |
| N5 原题 | research/prompts/_m2-newq-r1-body.md:15 | 首稿丢「改成重新读盘择根之后」这个前提从句（首稿直接说 the switch re-reads disk，丢了「较之前的取法，这是一次变更」这层）；丢「在飞」这个 checkpoint 的限定词；丢比较的基准（比内存里记的新，首稿只说 if the new root is newer，没说 newer than what） | Question 1 见提示文件 Part 5：After instance switch is changed to choose its root by re-reading the disk: when the root chosen by re-reading disk is newer than the root recorded in memory, is the in-flight checkpoint that would otherwise be resent, still resent or not? |
| N6 原题 | research/prompts/_m2-newq-r1-body.md:16 | 首稿把「读盘选出的，还是内存里记的」这两个具体候选压没了（只问 which root，没有把两个选项都摆出来）；丢「实例表行」这个具体名词（首稿只说 its row） | Question 2 见提示文件 Part 5 |
| N7 原题 | research/prompts/_m2-newq-r1-body.md:17 | 首稿丢「管理员选的」这个限定词（R_old 是谁选的，首稿只说 R_old） | Question 3 见提示文件 Part 5 |
| 表格轴定义句（行×列怎么摆） | research/prompts/_m2-newq-r1-body.md:33 | 首稿把「读盘选出的根」压成 disk root、「内存里记的根」压成 memory root，丢了「重新读盘」这个动作限定；「有、未提交/有、已提交」首稿误译成 open unconfirmed / open confirmed（用词不对，committed 不是 confirmed）；丢「在那一格给出什么动作」这个逐格限定（首稿只说 what each clause says to do，没锁定是「那一格」） | 见提示文件 Part 3、Part 4 全文，四处均已改正 |

补记（多出来的限定词 / 括注，原文没有，需要写明为什么加）：

- Part 1 背景术语段落里的 root ring、abandoned timeline、F_effective 三条释义不是某一句原文的转述，是我自己拼的辅助注（本地模型读不到任何文件，Clause E 直接用了这三个词却不展开定义，不加会让 Clause E 在自足材料里读不通）。三者的字面依据都落在 Clause E 同一段：.claude/kb/decisions/23-journal的角色与格式.md:369（"根环"出现在候选集定义里；"被抛弃时间线的根"在同一段后半；"F_生效" 标注了它的出处是 D16（发布语义） 已定项 1，我没有展开 D16 已定项 1 的正文，只把 F_effective 讲成「不透明的既定阈值，模型不需要求它的值」——因为这一轮的事实表不需要模型算出 F_effective 的具体值，只需要它知道候选集有这个门槛存在）。
- committed / uncommitted 两档 checkpoint 状态的操作化定义（Part 3 的 K2 / K3）不是某一句原文的转述，是我按 Clause B 的「被重发的checkpoint里没有事务号非0的记录（重发的是空发布）」反推出的操作化标准（自证校验和过的非0事务号记录 = committed）。这一条主 agent 的分工表只给了「有、未提交 / 有、已提交」这两个中文标签（body.md:33），没有给出判定标准，是本轮起草时补的，加的理由：不加无法让模型逐格填表，日后如与正式条款的口径不同，判据本身要重新核。
