# m2-agentdef-r1 本地攻方提示：转述核对表

提示文件：`research/prompts/m2-agentdef-r1-local-attack.md`（全篇零汉字，标签清单见提示本文）。
原文来源：判据原文出自 `.claude/gate.d/71-agent-def-flow-only.sh`（行号已用 `grep -n` 现查，见下）；十行样本原文出自正文 `_m2-agentdef-r1-body.md` 附录三给的特征表（行 60-75），逐格另用 `sed -n` 现读了样本指向的真实文件核对内容一致。

## 一、标签清单（英文项 / 原文文件:行 / 首稿缺的 / 定稿）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| WEISHENME | 71-agent-def-flow-only.sh:16「为什么」 | 无 | 直接标为「11 个标签词之一」，不译词义，只留标签 |
| YIJU | 71-agent-def-flow-only.sh:16「依据」 | 无 | 同上 |
| LIYOU | 71-agent-def-flow-only.sh:16「理由」 | 无 | 同上 |
| SHICE | 71-agent-def-flow-only.sh:16「实测」 | 首稿曾把 SHICE 也写进「后跟 COLON/WS」那一组（照抄依据/理由那一组的连接符要求）；核对第 19-20 行原文才发现「实测/试跑/踩过/撞上/撞过/因为/之所以」这一组在 ④ 里没有连接符要求，只有「为什么/依据/理由/原因/经过」那一组才要求后跟「：:或空白」 | 提示里 C4 明确分两组：第一组（含 SHICE）无连接符要求，第二组才要求 COLON 或 WS |
| JINGGUO | 71-agent-def-flow-only.sh:16、20「经过」 | 无 | 同上，注意 JINGGUO 同时出现在 ③ 的 11 词表与 ④ 的第二组 5 词表里 |
| YUANYIN | 71-agent-def-flow-only.sh:16、20「原因」 | 无 | 同上 |
| LAILI | 71-agent-def-flow-only.sh:16「来历」 | 无 | 只出现在 ③ 的 11 词表，不在 ④ 任一组里；提示里没有把它错放进 ④ |
| BEIJING | 71-agent-def-flow-only.sh:16「背景」 | 无 | 同上，只在 ③ |
| LISHI | 71-agent-def-flow-only.sh:10「历史」（①）、16「历史」（③） | 首稿一度想给①和③各开一个标签（因为①检查的是标题行文字「带」这个词、不要求连接符；③要求段首紧跟连接符），后来判断两处是同一个词、只是应用位置不同，合并成一个标签，在判据正文里分别说明两处的不同要求 | 单一标签 LISHI，在 C1 与 C3 两处分别写清各自的匹配条件（C1 只要「出现在标题文字里」，C3 要「段首 + 后跟连接符」） |
| YANGE | 71-agent-def-flow-only.sh:16「沿革」 | 无 | 只在 ③ |
| QIANQING | 71-agent-def-flow-only.sh:16「前情」 | 无 | 只在 ③ |
| SHIPAO | 71-agent-def-flow-only.sh:20「试跑」 | 无 | ④ 第一组，无连接符要求 |
| CAIGUO | 71-agent-def-flow-only.sh:20「踩过」 | 无 | 同上 |
| ZHUANGSHANG | 71-agent-def-flow-only.sh:20「撞上」 | 无 | 同上 |
| ZHUANGGUO | 71-agent-def-flow-only.sh:20「撞过」 | 首稿漏看第 20 行「撞过」与「因为」之间用的是顿号「、」而不是别处都用的斜杠「/」（「实测 / 试跑 / 踩过 / 撞上 / 撞过、因为 / 之所以」），险些当排版噪音直接忽略 | 提示未替模型消解这处不一致：把 ZHUANGGUO、YINWEI、ZHISUOYI 都列进同一个「组一」，不声称这处斜杠/顿号的差异有无意义，留给模型自己判断 |
| YINWEI | 71-agent-def-flow-only.sh:17、20「因为」 | 无 | ③ 里是独立的「连词」类，④ 里并入「组一」 |
| ZHISUOYI | 71-agent-def-flow-only.sh:17、20「之所以」 | 无 | 同上 |
| BIANGENGSHI | 71-agent-def-flow-only.sh:10「变更史」 | 无 | 只用于 C1 第二选项，与 LISHI 无关 |
| SHI_COPULA | 71-agent-def-flow-only.sh:16「是」 | 首稿曾以为「是」在 ④ 里也是合法连接符（照抄 ③ 的连接符集合），核对第 19-20 行才发现 ④ 第二组的连接符只有「：:或空白」两种，没有「是」 | 提示明写 C3 连接符集合含 SHI_COPULA，C4 第二组连接符集合不含 SHI_COPULA |
| COLON | 71-agent-def-flow-only.sh:16「：:」 | 无 | 覆盖全角冒号与半角冒号两种字形 |
| LPAREN | 71-agent-def-flow-only.sh:16「（(」 | 无 | 覆盖全角、半角左括号 |
| COMMA | 71-agent-def-flow-only.sh:16「，,」 | 无 | 覆盖全角、半角逗号 |
| PERIOD_FW | 71-agent-def-flow-only.sh:16「。」 | 首稿曾默认句号也有半角形式（照抄冒号/括号/逗号的两种字形模式），核对第 16、19 行才发现判据原文里句号只写了「。」一种字形，没有「.」 | 提示明写 PERIOD_FW 只覆盖全角句号，没有半角对应 |
| DUN | 71-agent-def-flow-only.sh:16「、」 | 首稿曾以为顿号是 ④ 的合法触发标点之一（因为 22 行「不认：顿号之后……」专门提它），核对第 19 行「行内在 （(，,；;。 或 —— 之后」才发现顿号根本不在 ④ 列出的触发标点集合里，「不认」那一条因此像是在排除一个本来就不会命中的情形 | 提示里 C4 的触发标点集合明确只有 LPAREN/COMMA/SEMI/PERIOD_FW/EMDASH2 五种，不含 DUN；「不认：紧跟在 DUN 之后」这条不认原样保留，但不替模型判断它是否多余，留给模型自己发现这处表面矛盾 |
| SEMI | 71-agent-def-flow-only.sh:19「；;」 | 无 | 覆盖全角、半角分号 |
| EMDASH2 | 71-agent-def-flow-only.sh:19「——」 | 无 | 十个样本里都没有用到这一类，仅在判据定义里出现 |
| CJKQ_OPEN / CJKQ_CLOSE | 71-agent-def-flow-only.sh:11-13「「」」 | 无 | 用于 C2 的引号内外判断，也用于样本里复现源文件里出现的「」原样位置 |

## 二、四条判据的转述（英文项 / 原文文件:行 / 首稿缺的 / 定稿）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| C1 只管标题行 | 71-agent-def-flow-only.sh:10 | 无 | 提示明写「C1 从不检查非标题行；非标题行不可能被 C1 判红」，这一句是首稿就有的补充限定（原文本身没有反向声明这一句，但①的定义「标题行的文字里带……」本身已隐含这层限定，补写出来是显式化，不是多加内容） |
| C2 引号例外 | 71-agent-def-flow-only.sh:12 | 无 | 逐字保留「引一条规则小节名时名字自带的日期是引用不是记录」这个举例，未省略 |
| C2「只提到目录」例外 | 71-agent-def-flow-only.sh:13 | 首稿把「只提到目录（`.claude/kb/` 后面直接是反引号或空白）」简化成了「只写目录名」,丢了「后面直接是反引号或空白」这个精确判据 | 定稿把「反引号或空白」这个精确条件原样写回（译成 "immediately followed by a punctuation mark or by whitespace"） |
| C3 只管段首 | 71-agent-def-flow-only.sh:14 | 无 | 明写「C3 只检查段落第一行」，并给出与之配套的「段落怎么切」规则（第 26 行） |
| C3 剥离前缀 | 71-agent-def-flow-only.sh:14「剥掉列表记号、`>`、`**`、⚠️ 之后」 | 首稿漏了 `>`（blockquote 记号）这一项，只写了列表记号、`**`、⚠️ 三项 | 核对原文补上 `>` 记号，定稿四项一个不少 |
| C3 连接符集合 | 71-agent-def-flow-only.sh:16 | 见上「SHI_COPULA」条 | 七种连接符（COLON/LPAREN/COMMA/PERIOD_FW/DUN/WS/SHI_COPULA）全部列出 |
| C3「不认」 | 71-agent-def-flow-only.sh:18 | 首稿的例句照抄了原文举例（背景材料路径……是输入项），后来判断这是给人看的例子、不是给模型的可比对数据，改成纯规则表述，不再引用具体例句（因为例句本身含中文，且这份提示要求样本里的汉字都要转成标签，例句里的「背景材料路径」不在标签表里，无法转标签，只能整句意译或整句去掉） | 定稿把「不认」写成纯规则（「一个标签词后面紧跟一个不在连接符列表里的字符」），不复述原文举的具体中文例子 |
| C4 触发标点集合 | 71-agent-def-flow-only.sh:19 | 见上「DUN」条 | 明确只有五种（LPAREN/COMMA/SEMI/PERIOD_FW/EMDASH2），不含 DUN |
| C4 中间可隔的内容 | 71-agent-def-flow-only.sh:19 | 无 | 「中间可隔空白与一个日期」原样保留为「只允许 WS 与一个 DATE 出现在这段间隙里，不允许别的字符」 |
| C4 两组及各自的后续要求 | 71-agent-def-flow-only.sh:20 | 见上「SHICE」「SHI_COPULA」两条 | 明确分组一（SHICE/SHIPAO/CAIGUO/ZHUANGSHANG/ZHUANGGUO/YINWEI/ZHISUOYI，无后续要求）与组二（WEISHENME/YIJU/LIYOU/YUANYIN/JINGGUO，后跟 COLON 或 WS） |
| C4 段落中间行的特例 | 71-agent-def-flow-only.sh:21 | 首稿漏译「渲染出来是同一段的后半句」这半句限定，只译了「段落中间的一行若按③的认法起头，也记在这一条里」 | 核对原文补上「so that the two lines rendered together read as one paragraph whose second half opens with that label word」 |
| C4「不认：顿号之后」 | 71-agent-def-flow-only.sh:22 | 无（但见上 DUN 条的矛盾点） | 原样保留为「不认」条款之一，不替模型消解它与「DUN 根本不在触发集合里」这一矛盾 |
| C4「不认：标签词后接别的字」 | 71-agent-def-flow-only.sh:22 | 无 | 与 C3 同一条「不认」逻辑，各自独立写出 |
| C4「不认：日期与词之间隔了别的字，由②管」 | 71-agent-def-flow-only.sh:23 | 无 | 原样保留这条转介到 C2 的说明 |
| 三条都不判的（围栏、表格行） | 71-agent-def-flow-only.sh:25 | 首稿曾把「② 照判」漏掉，只写了「表格行不判 ③④」，让人误以为表格行完全豁免 | 核对原文补回「C2 仍然照常应用于表格行」这一句 |
| 段落怎么切 | 71-agent-def-flow-only.sh:26 | 无 | 三种列表记号样式（`- `、`1. `、`3b. `）与「到下一个列表项/空行/标题/表格/围栏为止」「非列表连续非空行算一段」都逐句译出 |
| 判不到的（背景说明） | 71-agent-def-flow-only.sh:27 | 无 | 写成「背景说明，不是要检查的规则」，并明说十个样本都不是专门为这个盲点设计的，模型可以在 Part B 里借用这个思路，但不替它点名哪一个样本正好撞上这个例子 |

## 三、十个样本的转述（英文项 / 原文文件:行 / 首稿缺的 / 定稿）

逐样本另用 `sed -n` 现读了正文附录三指向的真实文件，与正文表格给的特征逐格核对，一致；下表记的是「把中文表格转成英文标签提示」这一步本身首稿漏掉、后来补上的地方，不是核正文表格对不对（正文表格的真伪不归本腿判）。

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| P1 occurrence 1 | agent-common.md:9 | 首稿把「。」之后直接写成「紧接着 WEISHENME」，漏了中间还隔着一个 CJKQ_OPEN 引号字符 | 核对 `sed -n '9p'` 原文，补全为「PERIOD_FW, CJKQ_OPEN, WEISHENME, CJKQ_CLOSE, CJKQ_OPEN, SHICE, ...」，把引号是否算「紧接着」的判断留给模型 |
| P1 occurrence 4 | agent-common.md:9 | 首稿以为「「开工先读」」这处引用有闭合的右引号，径直加了 CJKQ_CLOSE | 正文表格给的引用本身在「开工先读」处没有右引号（表格只抄到这里），定稿改成「这段引文在给出的地方就截断了，没有出现右引号」，不替表格补全它没给的东西 |
| P2 occurrence 1 | main-agent.md:8 | 无 | 「为什么这么定」逐字保留为「WEISHENME 后面没有任何间隔、直接接普通文字」，对应判据 22 行的「不认」举例本身 |
| P3 occurrence 3 | main-agent.md:20 | 首稿一度想加一句提示「这正是判据文件头第 27 行举的那个盲点例子」 | 判断这是替模型做推理，删掉这句提示；定稿只客观描述「PERIOD_FW 后面是普通文字，其中含 SHI_COPULA，但 SHI_COPULA 不是紧跟在 PERIOD_FW 后面那个字」，不点破它与「判不到的」背景说明的关系 |
| P4 occurrence 1 | three-way-defense.md:14 | 无 | 「唯一的日期在「」里」译成「the only DATE ... sits inside a CJKQ_OPEN ... CJKQ_CLOSE span」，并说明这个 span 同时也覆盖第 4 行给出的「（」之后那个点 |
| P5 附注 | implementation-writer.md:32 | 无 | 补了一句「这一行别处还有一个 WEISHENME，但它紧跟在普通动词后面、不在任何触发标点之后，所以不算一个点」，这是首稿就有的、为了不让模型漏看这处而主动标出的事实性说明，不是判断 |
| P6 occurrence 1 附注 | three-way-forward.md:27 | 无 | 补了「这句稍后还有 SHICE，但不紧跟在这个 SEMI 后面」这一提醒，避免模型把稍后出现的 SHICE 误当成这个点的内容 |
| P6 附注（YUANYIN） | three-way-forward.md:27 | 首稿漏掉了这行末尾「与原因。」里的 YUANYIN，只抄了表格给的两个点 | 核对 `sed -n '27p'` 原文，补一条附注：YUANYIN 出现在这句末尾，但前面是「与」不是触发标点，因此不成为一个点 |
| P7 occurrence 2/3 | kb-scribe.md:18 | 首稿把「「依据」紧跟在顿号之后」这条备注当成独立于「（」之后那两点之外的第三个点，但没有说清它与其后那个 LPAREN 的先后关系 | 核对 `sed -n '18p'` 原文，定稿写清顺序：先是 DUN 紧跟 YIJU（无间隔），YIJU 后面紧跟着的 LPAREN 才是表格里第二个「（」之后的点 |
| P8 opening | .../before/agents/three-way-local-defense.md:24 | 首稿把「实测（」这一开头写成「SHICE 后面隔了一个空白再接 LPAREN」（照抄了别处「实测（2026-09-16」的间隔习惯） | 核对 `sed -n '24p'` 原文，SHICE 与 LPAREN 之间没有空白，定稿改为「immediately (no gap) LPAREN」 |
| P8 日期归类 | .../before/agents/three-way-local-defense.md:24 | 无 | 明确这个日期在普通括号里，不在「」里，因此按 C2 算「引号外」，与 P4、P9、P10 几处「日期在「」里」的情形分开写 |
| P9 occurrence 3 | .../before/agents/kb-scribe.md:29 | 首稿最初把这一点写成「LPAREN, DATE, SHIPAO」（当成一次干净命中），核对 `sed -n '29p'` 原文「2026-09-17 两次试跑」才发现日期和 SHIPAO 之间还隔着「两次」两个字，不是纯粹的空白 | 定稿改写为「DATE, 然后两个字的普通文字（意思是"两次"）, 然后 SHIPAO」，把这处间隔如实标出，不再当成干净命中 |
| P9 occurrence 18 | .../before/agents/kb-scribe.md:29 | 无 | 核对同一行原文「2026-09-17 实测：」确认日期和 SHICE 之间只隔一个空白，写成「DATE, WS, SHICE, COLON」 |
| P10 occurrence 5 | .../before/agents/implementation-writer.md:27 | 首稿曾把这一点略过（表格原文写的是「2026-09-17 实测源码」，首稿一开始只抄了「2026-09-17」当日期、没往下抄 SHICE） | 核对 `sed -n '27p'` 原文补全为「COMMA, DATE, WS, SHICE, 然后普通文字（源码……）」 |
| P8/P9/P10 出处标注 | 正文附录三第 73-75 行 | 无 | 三个样本出自 `research/prompts/m2-agent-def-cleanup/before/agents/` 下的清理前快照，提示里明写「历史快照，不是今天的定义」，不让模型误以为这是当前生效的定义文本 |

## 四、多出来的限定词（英文比原文多、且写明为什么加）

| 多加的英文 | 对应原文位置 | 为什么加 |
|---|---|---|
| 「C1 never looks at anything except heading lines; a line that is not a heading line cannot be flagged by C1」 | 71-agent-def-flow-only.sh:10（原文本身没有这句反向声明） | 十个样本全部「标题：否」，若不显式声明这条反向推论，模型可能误以为还要在非标题行的文字里找「历史/变更史」这两个词 |
| 「note that this line also contains ... but it is not the token immediately after ...」（P1 occurrence 6 附近未加，P3/P5/P6/P9 occurrence 3 处加了） | 无对应原文原句，是本腿主动指出「附近还有一个标签词，但不满足紧邻条件」这一事实 | 这几处标签词与判据说的「紧邻」条件相差只有一两个字，不主动标出容易被模型看漏而误判成命中；只标事实（哪个词、为什么不算紧邻的字面理由），不下判据是否成立的结论 |
| 「this file is a historical snapshot kept for comparison, not one of today's live definitions」（P8/P9/P10） | 正文附录三第 73-75 行「出处」列本身只给了路径，没有这句说明 | 路径里的 `before/` 目录名对模型不构成语义信号；不加这句，模型可能把这三条样本当成「今天定义里现存的问题」去类比 C1 的「今天 18 份文件」范围，而这三条其实是清理前的对照 |
