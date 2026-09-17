# 运行记录：alloc-basis-r3-local-defense

提示：`research/prompts/alloc-basis-r3-local-defense.md`（51 行，4 问，Z5′ 辩方：替 G8 辩护）。
核对表：`research/prompts/alloc-basis-r3-local-defense-translation-audit.md`。

前台跑，未用 setsid / & / disown。共调用 5 次（含判红作废），按「连续五次调用拿不到两份干净的，停下照实报」封顶。

| 调用 | 文件 | ask-local.sh 退出码 | corruption-check.py | oov-check.py | 词数 | 人工通读结论 |
|---|---|---|---|---|---|---|
| 1 | output-s1.md | 0 | 绿（粘连=0、实词自复读=0） | 绿（生词=1：miscounted，正常派生词） | 503 | **带损坏，不算干净样本**：第 9 行 "not writing it anewew." —— "anewew" 疑似 "anew" 后粘一段 "ew" 的复读/粘连，两道闸都抓不到（oov-check 只扫 ≥9 字母的词，"anewew" 只有 6 字母，低于扫描下限）。拿不准是造词还是粘连，按规则一律记带损坏 |
| 2 | output-s2.md | 0 | 绿（粘连=0、实词自复读=0） | 绿（生词=14，含可疑项） | 575 | **带损坏，不算干净样本**：第 1 行 "observationalomputational"（疑似 observational + [c]omputational 掉字粘连）、第 5 行 "root-reachableachable paths"（reachable 后又粘一段 achable，典型残尾复读，与规则文档里 achievesceives 那个例子同形）。两处都在 oov-check 的生词清单里但拼接=0（规则 A–E 都没命中这两种具体形态），靠人工通读抓到 |
| 3 | output-s3.md | 0 | 绿 | 绿（生词=8，miscounted 类正常派生词） | 458 | **干净，计入第 1 份**。逐词通读未见粘连、复读、缺词或断句 |
| 4 | output-void1.md | 5（判红） | 红（实词自复读=1：prematurely） | 未跑（脚本判红即拒收，未打印正文） | — | 脚本自己判红，按定义留存为 void1，这一份不算，也不占样本号；下一步按提示建议本应改英文提示重跑，但提示本来就是英文，判红只是这次生成本身复读，直接原样重跑同一个号 |
| 5（重跑 4 号，沿用同一个文件名 s4） | output-s4.md | 0 | 绿（cjk=4，其余全 0） | 绿（生词=9，miscalculations 等正常派生词） | 462 | **计入第 2 份，但有一处需要如实报告的瑕疵**：第 7 行英文答案里嵌了一小段中文 "format parsing各写一份"，是模型把提示里 Q2 引文原文（"格式解析各写一份"）的一个片段原样抄回了英文答案，不是拼接 / 复读类损坏（不是 anewew / reachableachable 那种词内粘连），corruption-check.py 的 cjk 计数（=4）正是这四个汉字。按规则这不属于「闸抓不到的拼接损坏」，是模型引用提示原文时語言没切换干净；未见其他粘连、复读、缺词迹象，人工通读判可用，但如实标出这一处，不解读它算不算「损坏」 |

## 两份干净样本

- output-s3.md（调用 3）
- output-s4.md（调用 5，第 7 行含 4 个汉字的引文残留，已如实标出）

## 没做什么（固定会有的）

- 不解读、不总结、不采纳本地模型的答复；四问各自的辩护站不站得住，判它是主 agent 的事。
- 未对 output-s1.md、output-s2.md 里的具体论证内容做任何取舍——它们只是因为词面损坏被排除出「干净样本」，不代表其论证思路被认定无效或有效。
