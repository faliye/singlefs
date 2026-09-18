# 本地攻方运行记录：m2-wave1-code-r1

提示文件：`research/prompts/m2-wave1-code-r1-local-attack.md`（英文，自足；只攻正文第三节 Y4——U4 暖机的 txg / jsn 记录号 / 超级块 tail、U6 记账行数与准入、U5 出生序号，三张事实表逐格填数，不碰 Y1/Y2/Y3/Y5/Y6）。
核对表：`research/prompts/m2-wave1-code-r1-local-attack-translation-audit.md`（19 条源码/条款逐句核对，另有一段列出本轮自造的英文脚手架词）。
草稿目录：`/tmp/claude-1000/m2-wave1-code-r1-local-attack/`（本轮未用到，产物直接落地）。

两次前台调用（未用 setsid / 后台 / disown），均命中同一个网关 `http://127.0.0.1:8200/v1/chat/completions`；每次调用前 `curl -m 5 http://127.0.0.1:8200/v1/models` 确认网关能连通（返回 401 缺 key，说明网关本身在，`ask-local.sh` 自己会带 key）。

## 样本 1：`m2-wave1-code-r1-local-attack-output-s1.md`

- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-wave1-code-r1-local-attack.md > research/prompts/m2-wave1-code-r1-local-attack-output-s1.md`
- 退出码：0
- 词数：`wc -w` = 342；`wc -l` = 38
- `ask-local.sh` 内部串联的 `corruption-check.py` 与 `oov-check.py` 均未判红（退出码 0 才会落盘，脚本本身没有另外打印这两步的原样输出到 stdout 文件里）
- 本轮单独复跑 `oov-check.py`：绿，生词=5（去重后 2 个：`checkable`、`AccountingEntriesExceedOneNode`），拼接=0
  - 复核「checkable」：`grep -n` 命中 4 次，全在「no directly checkable number」这个短语里，是词典里的合法英文词（check + able），提示文件本身没有这个词，但也不是缺头粘连——手工确认 4 处上下文完整、没有断词
  - 复核「AccountingEntriesExceedOneNode」：命中 1 次，在 `W2.devices80` 那一行的准入判定位置，与提示里给的错误名逐字节相同，是要求原样复述的标识符，不是新造词也不是粘连
- 通读复核：13 个编号答案（W1 六个、W2 五个、W3 两个）全部在场，编号与提示要求的标签逐一对应，没有列表里整词缺失、没有孤零零标点的断句
- 判定：干净

## 样本 2：`m2-wave1-code-r1-local-attack-output-s2.md`

- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-wave1-code-r1-local-attack.md > research/prompts/m2-wave1-code-r1-local-attack-output-s2.md`
- 退出码：0
- 词数：`wc -w` = 447；`wc -l` = 13（每条编号答案自己独占一行，行内句子多，不是断行损坏）
- 本轮单独复跑 `oov-check.py`：绿，生词=6（去重后同样 2 个：`checkable`、`AccountingEntriesExceedOneNode`），拼接=0
  - 复核「checkable」：`grep -n` 命中 4 次，均在「provides no directly checkable number」/「no directly checkable number」这类短语里，语境完整
  - 复核「AccountingEntriesExceedOneNode」：命中 2 次（`W2.devices80` 判定值一次、同行「refuted by」那句复述一次），两处都逐字节与提示里的错误名相同
- 通读复核：13 个编号答案全部在场，标签与提示要求逐一对应，没有整词缺失、没有断句
- 判定：干净

## 作废副本

无。两次调用都在第一次尝试就判绿，`research/prompts/` 目录下 `m2-wave1-code-r1-local-attack` 前缀只有 `.md`（提示本身）、`-output-s1.md`、`-output-s2.md`、`-translation-audit.md` 四个文件（已用 `ls` 现查，见上）。

## 样本数与停止条件

两次调用（s1、s2）全部判绿，达到「至少两份干净样本」的要求；未消耗任何重试额度，未触及「连续五次调用拿不到两份干净的」这条停下条件。
