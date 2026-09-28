# 本地攻方运行记录：abandoned-floor-r1（X3　10 槽的账）

日期：2026-09-28。
提示文件：`research/prompts/abandoned-floor-r1-local-attack.md`（英文、纯 ASCII、没有 `*` 与反引号；18 条事实、9 题；事实行带来源，要求答复不写行号）。sha256 `be94db41b67ebdf8ba5ec7c08d47cc0d4ffd6626a207b11beafb5cb9753acd69`。
核对表：`research/prompts/abandoned-floor-r1-local-attack-translation-audit.md`（逐句对照、不译的原文、英文多出来的三节）。sha256 `3f86398551acd62905de7f5c5b254e2723b95d7c2cf818eaa3ff31731f5e03e4`。
两次调用都用 Bash 的 run_in_background 起，命令一样：
`cd /home/fy5090/code/singlefs && nice -n 19 bash research/scripts/ask-local.sh research/prompts/abandoned-floor-r1-local-attack.md > research/prompts/abandoned-floor-r1-local-attack-output-s<n>.md`

## 样本 1：`research/prompts/abandoned-floor-r1-local-attack-output-s1.md`

- 退出码：0（完成通知里的）
- 词数：`wc -w` = 922；`corruption-check.py` 自报 words=720
- `corruption-check.py <样本> <提示>`：`绿 … cjk=0 words=720 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0`
- `oov-check.py <样本> <提示>`：`绿 … 生词=0 拼接=0`；生词表为空，没打满
- 通读：102 行，没见缺头的粘连词、缺词、断句或孤零标点；答复里没写代码行号与文件行号（`grep -nEi 'line [0-9]|lines [0-9]|\.rs'` 无命中）
- 判定：干净

## 样本 2：`research/prompts/abandoned-floor-r1-local-attack-output-s2.md`

- 退出码：0（这次的完成通知后来收到了，退出码 0；主 agent 同时来消息说它可能收不到通知，收到时样本已落盘）
- 词数：`wc -w` = 1860；`corruption-check.py` 自报 words=1610
- `corruption-check.py <样本> <提示>`：`绿 … cjk=0 words=1610 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 单字母替换=0`
- `oov-check.py <样本> <提示>`：`绿 … 生词=0 拼接=0`；生词表为空，没打满
- 通读：163 行，没见缺头的粘连词、缺词、断句或孤零标点；没写行号（同上 grep 无命中）
- 判定：干净

## 作废副本

没有（两次都没判红，`ls research/prompts/abandoned-floor-r1-local-attack*` 里没有 `-output-void*`）。

## 算术比对

主 agent 没给答案表，算术没比。

## 样本数与停止条件

两次调用，两份干净样本，够「至少两份干净样本」的下限，就停了。没有参考样本。

## 没做什么

- 不解读、不总结、不采纳答复，也不比两份样本方向一不一致。
- 译文核对表里的代码行号是在冻结副本 `/tmp/claude-1000/abandoned-floor-r1/tree/crates/` 上现取的；副本的 sha256 没有对着 `research/prompts/abandoned-floor-r1-snapshot/crates-sha256.txt` 重核。
- 调查报告里在改过的副本上量的候选结局（2.3 表、3307 那次反事实、甲后面的括注「99 条全绿」）故意没给模型，理由写在核对表第二节。
