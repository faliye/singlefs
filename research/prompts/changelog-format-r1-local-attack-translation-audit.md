# 本地攻方腿英文提示译文核对表（changelog-format-r1）

提示文件：`research/prompts/changelog-format-r1-local-attack.md`。原文出处：`research/prompts/_changelog-format-r1-body.md`「## 要腿答的」第 1、5 条。列「英文项」给的是提示文件里对应内容的行号；列「原文文件:行」是 `_changelog-format-r1-body.md` 自己的行号（现查取得）。

## 第 1 条（编号判据的缝隙）

| 英文项（提示文件行号） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 提示 20 行「The check only flags a problem when len(numbers) >= 2, meaning the literal `### <date>` heading line itself is repeated two or more times under the same section for the same date.」 | `_changelog-format-r1-body.md:79`「新 `shape` 格判「同一节里同一天的 `### ` 标题不许出现两次」」 | 无缺 | 「同一节里同一天」译成「under the same section for the same date」，「不许出现两次」换算成 `len(numbers) >= 2` 才判红，等价换算，不缺限定词 |
| 提示 20 行「It does not walk into the child `#### ` sub-headings under a `### <date>` heading and compare their text to each other at all.」 | `_changelog-format-r1-body.md:80`「但不检查「同一节同一天下，两个 `#### ` 子标题的点名词 + 编号完全一样」」 | 首稿把原文「点名词 + 编号完全一样」这个具体判据泛化成「compare their text to each other」（比较全部文本，不只是点名词+编号） | 定稿保留这句泛化表述（因为代码事实上确实完全不比较子标题文本，泛化表述仍然为真，不算错），但在附带的构造输入与问题 6–8 里逐字落实「点名词 + 编号完全一样」这一具体判据（两个子标题的点名词都是「已定项 7」、编号都是「（其一）」），把泛化表述收窄回原文的具体判据，不算丢限定词 |
| 提示 22 行「Check 2 is a separate script pair: history-ordinal.sh calls a python key-extractor, history-ordinal-keys.py...」 | `_changelog-format-r1-body.md:81`「这条是 `history-ordinal.sh` 管的，两道门禁管的范围有没有缝隙？」 | 无缺 | 「两道门禁」译成「these two gate checks」（提示第 1 行），「缝隙」译成「a gap between the scopes」，都在场 |
| 提示 68–82 行的构造输入与问题 1–5 | `_changelog-format-r1-body.md:81-82`「举一个具体的输入：两个 `#### 已定项 7（其一）：……` 挂在同一个 `### 2026-09-28` 下、内容不同」 | 无缺 | 构造输入逐字用了「已定项 7（其一）」这个中文字面串（两次，内容不同），日期用「### 2026-09-28」，与原文完全对应；两条 `#### ` 内容分别编成「持锁重试」与「释放锁后重试」两种不同正文，满足「内容不同」 |
| 提示问题 1、5「does the shape cell...let this pass」「reports a violation or shape-clean」 | `_changelog-format-r1-body.md:82`「30 号的 `shape` 格会不会放过它」 | 无缺 | 「30 号」（门禁阶段编号）没有译出，因为提示要求答复不写门禁编号、只按检查作用描述（`shape cell`），这是有意省略，不是漏译；对模型不构成缺限定词，只是不提供一个它用不上、且规则要求不许问它的编号 |
| 提示问题 9「would history-ordinal.sh report this pair of sub-headings as a collision」 | `_changelog-format-r1-body.md:83`「`history-ordinal.sh` 真的会抓到吗」 | 无缺 | 对应 |
| 提示问题 6、7「what four-part key (h2, h3_date, stem, ordinal) does keys_of produce」 | `_changelog-format-r1-body.md:83-84`「（`.claude/singlefs-ai-sop/scripts/history-ordinal-keys.py` 的 `stem_and_ordinal` 与 `keys_of`）」 | 无缺 | 两个函数名都点名，且提示 30-64 行整段抄了这两个函数的源码（与附录 `_changelog-format-r1-appendix.md:5312-5349` 逐字核对一致），不是转述 |

## 第 5 条（历史条目自引用会不会失效）

| 英文项（提示文件行号） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 提示 106 行「Prose sentences elsewhere in the knowledge base sometimes cite a specific changelog entry...」 | `_changelog-format-r1-body.md:96`「决策与实验变更史里，历史条目自己引用别的历史条目（「本文件第 N 行」「某日（其N）」这类自引用）有多少条，这一轮改没改全」 | 首稿只覆盖「某日（其N）」一类，没有覆盖「本文件第 N 行」一类 | 定稿在提示 106 行明写这一点缺口并给出理由：全仓 grep 两份变更史找「本文件第 N 行」这类同文件行号自引用，没有找到能落成表格行的具体实例（现查命令：`grep -noE '本文件第[0-9]+行' .claude/kb/decisions-history.md .claude/kb/experiments-history.md` 零命中），所以表格只覆盖「某日（其N）」一类，提示里写明这条缩窄与原因，不是无声漏掉 |
| 提示 106 行「Before this prompt was written, two such stale citations were already found and fixed... D6 and decision D22 used to cite each other's entry dated 2026-09-06 (the thirty-sixth one...)」 | `_changelog-format-r1-body.md:97`「已知修了两处主 agent 现查到的（D6/D22 互相引用「2026-09-06（其三十六）」那两处）」 | 无缺 | 对应，且明写「不问这两处已修的」 |
| 提示 106 行「The reorganization script's own self-check...does not separately check whether this class of self-citation...becomes wrong because of the renumbering.」 | `_changelog-format-r1-body.md:98`「`migrate-changelog-format.py` 的自检没有专门查这一类自引用会不会因为搬迁而失效」 | 无缺 | 对应 |
| 提示问题 19、20「can you tell... whether there are more beyond what rows 1 through 6 already show」「do you think it would be worth someone sweeping the rest」 | `_changelog-format-r1-body.md:99`「这是不是还有遗漏，值不值得再扫一遍？」 | 无缺 | 拆成两问（19 问「能不能从给定材料判断」、20 问「值不值得再扫」），对应原文两个分句 |

## 英文比原文多出来的限定词 / 括注

| 提示文件行号 | 多出来的内容 | 为什么加 |
|---|---|---|
| 提示第 1 行 | 「You are not reviewing any file-system or storage-engine code.」 | 这一轮改的是 kb 治理工具与文档组织，不碰 `crates/`；补这句是防止本地模型把两个决策示例文本（「调用方在锁外重试」这类，为了让例子看着像真实决策而编的占位内容）误当成真实的文件系统设计问题去论证，与要答的两条判据无关 |
| 提示第 68 行「This exact markdown text has never been run through either check; you are asked to simulate what each check would do with it.」 | 提示原文没有这句 | 防止模型误以为这是仓里已经跑过的真实产物、去找"实际结果"而不是自己逐行模拟算法；这是给模拟任务定性质，不是引入新判据 |
| 提示第 106 行「A text search across both changelog files for the first style...did not turn up any clear instance to build a row from」 | 见上表「首稿缺的」一栏 | 交代为什么表格不覆盖「本文件第 N 行」这一类，避免无声漏掉 |
| 提示问题 18 | 关于 `decisions-history/2026-08.md` 被这一轮删除的事实（`git status` 现查坐实：`D  .claude/kb/decisions-history/2026-08.md`） | 原文第 96-99 行没有点名这个具体例子；这是我在准备材料时另外核实到的一个真实、可验证的自引用失效候选（文件路径级，不是序号级），补一道题用它做交叉验证，不影响对原文两句问题的忠实转述，只是多给一个可判的具体样本 |
