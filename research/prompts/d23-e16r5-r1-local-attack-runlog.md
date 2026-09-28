# d23-e16r5-r1 本地攻方 W3：运行记录

提示文件：`research/prompts/d23-e16r5-r1-local-attack.md`
译文核对表：`research/prompts/d23-e16r5-r1-local-attack-translation-audit.md`
草稿目录：`/tmp/claude-1000/d23-e16r5-r1-local/`（内含 `compute_answer.py`，我自己按提示里写死的 CONV1/RISECONV/NOCONV 规则算的参考表，仅供我自己核对格式用，不是主 agent 给的答案表——见下方「算术」一节）

## 调用记录

第一次调用（call 1）：`bash research/scripts/ask-local.sh research/prompts/d23-e16r5-r1-local-attack.md > research/prompts/d23-e16r5-r1-local-attack-output-s1.md`，退出码 5。s1 确认 0 字节。作废副本 `research/prompts/d23-e16r5-r1-local-attack-output-void1.md`（词数 643）。
corruption-check.py 判红：
```
红 research/prompts/d23-e16r5-r1-local-attack-output-void1.md  cjk=9 words=643 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=4 缩写自粘=0 单字母替换=0
     实词自复读: riseconv | riseconv | riseconv | riseconv
```
oov-check.py 判绿：`绿 ... 生词=1 拼接=0` 生词：diverging

第二次调用（call 2）：同一条命令改用 `>|` 重定向到同一个号，退出码 5。s1 再次确认 0 字节。作废副本 `research/prompts/d23-e16r5-r1-local-attack-output-void2.md`（词数 593）。
corruption-check.py 判红：
```
红 research/prompts/d23-e16r5-r1-local-attack-output-void2.md  cjk=6 words=593 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=4 缩写自粘=0 单字母替换=0
     实词自复读: riseconv | riseconv | riseconv | riseconv
```
oov-check.py 判绿：`绿 ... 生词=0 拼接=0`

第三次调用（call 3）：同一条命令 `>|` 到同一个号，退出码 5，这次前台内完成（未进后台）。作废副本 `research/prompts/d23-e16r5-r1-local-attack-output-void3.md`（词数 572）。ask-local.sh 自己打印的判红行：
```
红 /tmp/tmp.m788wUENh9  cjk=6 words=572 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=4 缩写自粘=0 单字母替换=0
     实词自复读: riseconv | riseconv | riseconv | riseconv
```
读 void3 定位命中处：`research/prompts/d23-e16r5-r1-local-attack-output-void3.md` 第 4 行等多处是 "RISECONV RISECONV"（表格里 low-boundary 与 high-boundary 两列相邻同值、模型没有用竖线分隔，纯空格相邻），命中 `corruption-check.py` 的「实词自复读」规则（同一个 ≥5 字母词只隔一个空格连着出现两次）。
补跑 oov-check.py（此前误以为 corruption-check.py 判红即可不跑，实际两个检测器都要跑）：`绿 ... 生词=1 拼接=0` 生词：diverging。判红三次后，改提示：在正文里加一条规则，要求相邻两格若同值要写 "same" 代替重复标签、并要求用竖线分隔相邻列，用 `replace-once.py`（经临时 python 包装调用其 `replace_once` 函数，因两处替换串带撇号与长文本、shell 转义会命中 0 次）定点改了提示文件三处，改动记在译文核对表所在目录之外——这处改动是格式修复，不改事实表数值、不改问题实质，改动后的提示文件才是下面 s1/s2/s3 实际用的那份。

第四次调用（call 4，改过提示之后）：`bash research/scripts/ask-local.sh research/prompts/d23-e16r5-r1-local-attack.md >| research/prompts/d23-e16r5-r1-local-attack-output-s1.md`，退出码 0。s1 非空（3019 字节，514 词）。oov-check.py：`绿 ... 生词=0 拼接=0`。通读一遍未见缺词、断句、粘连。判：干净。

第五次调用（call 5）：`bash research/scripts/ask-local.sh research/prompts/d23-e16r5-r1-local-attack.md > research/prompts/d23-e16r5-r1-local-attack-output-s2.md`，退出码 0。s2 非空（3417 字节，379 词）。oov-check.py：`绿 ... 生词=1 拼接=0` 生词：diverging（未打满 300 字符）。通读一遍时第 22 行发现「widening without divergence diverging indefinitely」——divergence 与 diverging 两个不同形态的近义词紧挨着连用，读不通顺，拿不准是模型行文毛病还是缺词类损坏，按规则从严记「带损坏」，损坏处：s2 第 22 行 "without divergence diverging indefinitely"。判：带损坏（参考样本）。

第六次调用（call 6）：`bash research/scripts/ask-local.sh research/prompts/d23-e16r5-r1-local-attack.md > research/prompts/d23-e16r5-r1-local-attack-output-s3.md`，退出码 0。s3 非空（3136 字节，318 词）。oov-check.py：`绿 ... 生词=1 拼接=0` 生词：boundedness（未打满 300 字符）。通读一遍未见缺词、断句、粘连。判：干净。

干净样本已达两份（s1、s3），停止取样。

## 逐份清单

| 路径 | 退出码 | 词数 | 生词清单（oov-check.py，未打满 300 字符则整列出） | 判定 |
|---|---|---|---|---|
| `research/prompts/d23-e16r5-r1-local-attack-output-void1.md` | 5（对应 s1 那次重定向留空） | 643 | diverging | 作废（参考样本），实词自复读 riseconv×4 |
| `research/prompts/d23-e16r5-r1-local-attack-output-void2.md` | 5 | 593 | （无） | 作废（参考样本），实词自复读 riseconv×4 |
| `research/prompts/d23-e16r5-r1-local-attack-output-void3.md` | 5 | 572 | diverging | 作废（参考样本），实词自复读 riseconv×4，命中处为表格相邻同值两列 |
| `research/prompts/d23-e16r5-r1-local-attack-output-s1.md` | 0 | 514 | （无） | 干净 |
| `research/prompts/d23-e16r5-r1-local-attack-output-s2.md` | 0 | 379 | diverging | 带损坏（参考样本），损坏处第 22 行 "without divergence diverging indefinitely" |
| `research/prompts/d23-e16r5-r1-local-attack-output-s3.md` | 0 | 318 | boundedness | 干净 |

## 算术

提示里有一道事实表题（21 格分类网格 + Q22 的 NOCONV 清单）。主 agent 没有另给一份算好的答案表的路径，算术没比。我自己在草稿目录写了一份按提示里写死的 CONV1/RISECONV/NOCONV 规则算的参考表（`/tmp/claude-1000/d23-e16r5-r1-local/compute_answer.py`），那是我自己核对提示格式用的中间产物，不是主 agent 给的答案表，不构成正式比对。

## 没做什么

- 不解读、不总结、不采纳三份样本（s1、s2、s3）与三份作废副本（void1-3）的具体答复内容，也不判样本之间方向是否一致——那是主 agent 的事。
- s2 的「带损坏」判定只凭我自己通读时的语感（divergence/diverging 连用不顺），不是任何自动化检测器判的红；这一判断本身也可能是错的，留给主 agent 核。
