# m2-step3-code-r1 本地辩方运行记录

提示：`research/prompts/m2-step3-code-r1-local-defense.md`；核对表：
`research/prompts/m2-step3-code-r1-local-defense-translation-audit.md`。前台跑
`nice -n 19 bash research/scripts/ask-local.sh <提示文件>`，未用 `setsid` / `&` / `disown`。

## 第一次跑：提示文件本身被判红（不是模型输出的问题）

第一次调用 `ask-local.sh` 时退出码 5，`corruption-check.py` 对**提示文件自己**报
`cjk=4 words=4481 fffd=0 ... 粘连=7`。逐一核对，7 处「粘连」全部来自 Rust 的 `::` 路径分隔符
（`TransactionUnit::IN_BUMP_ORDER`、`PoolAllocator::rebuild_from_records` 等）：检测器的标点粘连
判据要求「标点前一字符非字母数字、后一字符是字母数字」，`X::Y` 里第二个冒号恰好满足，被误判成
「一句话被截断后接上了下一句」。这是检测器对代码语法的假阳性，模型当时甚至还没被调用
（生成的临时输出文件本身是绿的：`cjk=0 words=1073 ... 粘连=0`）。

处置：把提示里全部 7 处 `模块::函数` 写法换成散文形式（如「the function rebuild_from_records
on PoolAllocator」），用 `research/scripts/replace-once.py` 逐处定点替换（7 次，全部命中 1
次），不改事实内容。顺带把两处直接嵌入的原文汉字（"预想"、"挂载"）也换成纯英文注释，贴住
「提示一律用英文」。改完单独跑 `corruption-check.py` 核对提示文件本身：`cjk=0 ... 粘连=0`，
判绿。详情见核对表文件末尾「跑前修订」一节。这一轮的输出（模型侧本来就是绿的）**不计入**
样本编号，作废件留存为 `m2-step3-code-r1-local-defense-output-void1.md`（1038 词）。

## 第二次起：提示已确认干净，逐次跑模型

| 次序 | 退出码 | 判定 | 词数 | 检测器与命中 | 处置 |
|---|---|---|---|---|---|
| 2 | 5 | 作废（模型输出损坏） | 993 | `corruption-check.py` 判红：实词自复读 1 处——`publish publish`（"the instance-table row-publish publish shows InstanceTable..."，同一个 ≥5 字母词连着出现两次） | 留存 `-output-void2.md`，不计入样本 |
| 3 | 5 | 作废（模型输出损坏） | 936 | `corruption-check.py` 判红：拼接 1 处——`fragmentationragmentation`（= fragmentation + (f)ragmentation，缺头粘连） | 留存 `-output-void3.md`，不计入样本 |
| 4 | 5 | 作废（模型输出损坏） | 839 | `corruption-check.py` 判红：拼接 1 处——`fragmentationallocation`（= fragmentation + allocation） | 留存 `-output-void4.md`，不计入样本 |
| 5 | 0 | **干净，记为 s1** | 1090 | `corruption-check.py` 绿；另跑 `oov-check.py` 复核（三方规则「闸判绿也要跑 oov-check.py」）：生词 4 个（factually / misrepresents / contradicting / mutations），均为正常英文词，非缺头粘连，判绿 | 存为 `-output-s1.md` |
| 6 | 5 | 作废（模型输出损坏） | 885 | `oov-check.py` 判红：拼接 2 处——`bootstraps`（= boots + traps）、`AlreadyReleased`（= already + released，这是本轮提示里用到的真实标识符 `ReleaseTargetAlreadyReleased` 被从中间截断复述） | 留存 `-output-void5.md`，不计入样本 |

## 结论：五次（提示确认干净之后）只拿到一份干净样本，不够两份，照实报

按 `.claude/agents/three-way-local-defense.md`「没做什么」一节的预告与
`.claude/agents/three-way-local-attack.md`「做什么」第 6 条「连续五次拿不到两份干净的，停下
照实报」：本轮从第 2 次到第 6 次（提示已确认干净）共跑了 5 次，4 次判红、1 次干净，只拿到
`m2-step3-code-r1-local-defense-output-s1.md` 一份干净样本，够不上两份。已停止，不再继续抽样。

⚠️ **这轮损坏率（5 次里 4 次红）比 2026-09-16 第一轮记录的「五次坏三次」更差**，与该文件
「样本更容易不够两份」的预告一致；四次坏样本里三次是拼接类（缺头/缺尾粘连词：
`fragmentationragmentation`、`fragmentationallocation`、`bootstraps`/`AlreadyReleased`），
一次是实词自复读（`publish publish`）——四种都不是同一次判定器判出来的巧合，`corruption-
check.py` 与 `oov-check.py` 各抓到了不同的例。

## 干净样本

`research/prompts/m2-step3-code-r1-local-defense-output-s1.md`（1090 词，
`corruption-check.py` 与 `oov-check.py` 均判绿）。只有这一份，未达两份的抽样要求，
唯一样本的答复未经第二份交叉验证。

## 没做什么

- 不解读、不总结、不采纳本地模型在 s1 里给出的任何论断（六处取法站不站得住、r2 第三节两格
  在重建路上还成不成立，都是主 agent 自己核实之后才能用的结论，这里不做）。
- 没有拿到第二份干净样本，s1 的答复没有第二份同题样本互相印证。
- 没有用 `ASK_LOCAL_ALLOW_CORRUPT=1` 强行采用任何一份判红的输出。
- 没有跑 `ASK_LOCAL_TIMEOUT` 之外的重试策略（比如换用不同的提示切分、拆成更短的多轮提问）；
  五次预算用完即停，没有超出这一步该做的范围去调整提示重新分组。
