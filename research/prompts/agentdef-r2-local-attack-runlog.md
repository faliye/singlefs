# agentdef-r2 本地攻方 运行记录

提示文件：`research/prompts/agentdef-r2-local-attack.md`
核对表（逐句核转述 + 事实数字来源核对）：`research/prompts/agentdef-r2-local-attack-translation-audit.md`
调用方式：前台 `nice -n 19 bash research/scripts/ask-local.sh research/prompts/agentdef-r2-local-attack.md > research/prompts/agentdef-r2-local-attack-output-sN.md`，未用 `setsid` / `&` / `disown`。
禁读清单：派发消息未给，未读任何被禁文件。
草稿目录：`/tmp/claude-1000/agentdef-r2-local/`（提示与核对表的草稿在此，定稿已用 `set -o noclobber` 新建进交付路径）。

## 逐次调用

| 序号 | 退出码 | 词数（`wc -w`） | corruption-check.py | oov-check.py 生词清单 | 判定 |
|---|---|---|---|---|---|
| s1 | 0 | 495 | 绿（cjk=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0） | 生词=3：misrepresent、categories'、clarifies（三个都是正常英文词/所有格，逐一现查上下文，不是缺头粘连词，不记带损坏） | 干净 |
| s2 | 0 | 568 | 绿（cjk=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0） | 生词=4：misrepresenting、untraceable、assessment、reassessment（四个都是正常英文词，逐一现查上下文，不是缺头粘连词，不记带损坏） | 干净 |

两次调用都在第一次尝试就退出码 0，两道检测器都判绿。另按定义第 5 步通读了两份样本全文：s1（27 行，答完 Q1、Q2-A、Q2-B、Q2-C、Q3、Q4 六个标签，每题都带「推翻条件」那一句）与 s2（33 行，同六个标签全部答完）均无缺词、无断句、无列表项只剩孤零标点的迹象，判「带损坏」的迹象一处未见。没有出现退出码 5（字词损坏作废）的情形，因此没有 `-output-void*.md` 副本（现查 `research/prompts/agentdef-r2-local-attack-output-void*.md` 不存在）。样本自带的行号：两份样本正文都没有引用任何文件行号或代码行号（提示的 FORMAT RULES 一节明令禁止），因此无需标注「模型自给、未核」。

## 干净样本数

两份（s1、s2），达到「至少两份干净样本」的要求，未续跑第三次。

## 没做什么

- 没有解读、总结或采纳两份样本的答复内容，也没有比较两份样本之间方向是否一致——这些判定留给主 agent。
- 没有读任何禁读清单文件（派发消息未给禁读清单）。
- 没有跑第三次调用（两次都干净，未触发「连续五次调用拿不到两份干净的」条款）。
