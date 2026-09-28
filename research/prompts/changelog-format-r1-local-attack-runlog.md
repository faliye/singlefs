# 本地攻方腿运行记录（changelog-format-r1）

提示文件：`research/prompts/changelog-format-r1-local-attack.md`（sha256 `4bf8465eae89c7fd5caed6d596ab43cead858dbd0d1f7275b45c80abc7f56e2a`）。
译文核对表：`research/prompts/changelog-format-r1-local-attack-translation-audit.md`。
攻击面：正文「## 要腿答的」第 1、5 条（编号判据 shape 格与 history-ordinal.sh 的缝隙；历史条目自引用会不会因搬迁失效）。

## 样本一览

| 样本 | 退出码 | 词数 | 生词清单（oov-check.py，带提示文件） | 判定 |
|---|---|---|---|---|
| `research/prompts/changelog-format-r1-local-attack-output-s1.md`（sha256 `34f07a6c2377dc661aca71e6558261a7b963e30c30bee8969c540e5f1b67706c`） | 0 | 705 | 生词=0 拼接=0（命令：`python3 research/scripts/oov-check.py <样本> <提示文件>`，原样输出：`绿 research/prompts/changelog-format-r1-local-attack-output-s1.md  生词=0 拼接=0`） | 干净 |
| `research/prompts/changelog-format-r1-local-attack-output-s2.md`（sha256 `0bc09bdf2185a02771e1b29baeee8e4fa29c00f6d57f169ceb13d24af84f606c`） | 0 | 551 | 生词=0 拼接=0（原样输出：`绿 research/prompts/changelog-format-r1-local-attack-output-s2.md  生词=0 拼接=0`） | 干净 |

两次调用都是第一次尝试就退出码 0，没有触发损坏闸，没有产生 `-output-void<n>.md` 作废副本（`ls research/prompts/changelog-format-r1-local-attack-output-void*.md` 无命中）。累计干净样本 2 份，按定义「攒到至少两份干净样本为止」已够，未继续取号。

## 通读结果（闸判绿之后仍逐段通读）

- s1（45 行，20 条编号答复加各自「This would be refuted if…」一句）：逐段通读，没有发现缺头粘连词，没有发现缺词或断句（例如列表里整词丢失只剩孤零零标点）。答复格式与提示要求一致：每条一句结论 + 一句推翻条件，没有使用 markdown 强调，没有引用代码行号或文件行号（凡引证据都按函数名、行标签「row N」「line a/b/c」指，符合提示里「不写代码行号与文件行号」的要求）。判定：干净。
- s2（69 行，同样 20 条编号答复）：同样逐段通读，没有发现缺头粘连词、缺词或断句。判定：干净。

## 算术核对（5b）

主 agent 未给这一轮的事实表答案表路径。运行记录按规定写：没有答案表，算术没比。

## 没做什么

- 不解读、不总结、不采纳两份样本里的具体答复内容，也不判它们方向是否一致；这两件事按定义交主 agent 做。
- 没有追加第三次调用：两次调用都在第一次尝试即得到干净样本，没有触发「连续五次调用拿不到两份干净样本」的停止条款，因此没有更多样本。
