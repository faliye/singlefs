# m2-rollback-forward-r1 本地攻方腿：运行记录

提示文件：`research/prompts/m2-rollback-forward-r1-local-attack.md`。
核对表：`research/prompts/m2-rollback-forward-r1-local-attack-translation-audit.md`。
调用方式：前台 `nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <样本>`，未用 `setsid`、`&`、`disown`（两次调用都超过工具默认 120s 前台超时，被工具自动转后台执行，随后等待完成通知，未中途中断）。

| 样本 | 退出码 | 词数（`wc -w`） | oov-check.py 生词（去重后打印的那几个） | corruption-check.py | 判定 |
|---|---|---|---|---|---|
| `m2-rollback-forward-r1-local-attack-output-s1.md` | 0 | 793 | computable falsified contradicting falsification（共 19 处命中，去重 4 个，均为正常英文词，非缺头粘连词） | 绿（cjk=0 words=755 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0） | 干净 |
| `m2-rollback-forward-r1-local-attack-output-s2.md` | 0 | 717 | computable falsified contradicting unmount's（共 12 处命中，去重 4 个，均为正常英文词或规则派生词，非缺头粘连词） | 绿（cjk=0 words=676 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0） | 干净 |

两份都另外通读一遍全文，没有发现「列表里整个词没了、只剩孤零零标点」这类缺词/断句损坏。两份均判干净，累计干净样本 2 份，达到「至少两份」的停止条件，未再往下调用。

没有判红作废的调用，`-output-void*.md` 不存在。

样本文件本身携带的任何行号（若有）一律未核，标「模型自给、未核」——本次两份样本正文里模型没有引用任何文件行号（提示第 128-130 行明令禁止），此项不适用。
