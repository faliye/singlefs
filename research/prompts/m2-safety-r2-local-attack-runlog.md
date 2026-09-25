# m2-safety-r2 本地攻方：运行记录

提示文件：`research/prompts/m2-safety-r2-local-attack.md`
译文核对表：`research/prompts/m2-safety-r2-local-attack-translation-audit.md`

调用方式：`bash research/scripts/ask-local.sh research/prompts/m2-safety-r2-local-attack.md > research/prompts/m2-safety-r2-local-attack-output-s<n>.md`，四次调用全部前台跑，未用 `setsid`/`&`/`disown`。

| 调用序 | 样本/作废文件 | 退出码 | 词数（`wc -w`） | 行数（`wc -l`） | `oov-check.py` 结果 | 判定 |
|---|---|---|---|---|---|---|
| 1 | `-output-s1.md` | 0 | 400 | 24 | `绿 ... 生词=1 拼接=0`，生词：`refuteail`（退出码0） | 带损坏——`refuteail` 是缺头的粘连词（`refute` + `fail` 粘死、缺一个字母 `f`，与规则示例 `achievesceives`/`cryptographicord` 同一签名），不当干净样本 |
| 2 | `-output-s2.md` | 0 | 585 | 91 | `绿 ... 生词=11 拼接=0`，列出的生词：`shortfall`、`refutation`（退出码0） | 干净——两个生词都是词表外的正常英文词（不是粘连词），通读全文未见缺词、断句、孤立标点一类损坏 |
| 3（首次） | 作废副本 `-output-void1.md` | 5 | 194（corruption-check.py 打在 stderr 上的 words=，`wc -w` 现查该作废文件本身为 380） | — | `ask-local.sh` 内部串跑判红：`红 ... 实词自复读=1`，`实词自复读: reclaimable`（退出码5，`corruption-check.py` 判红） | 作废——按规则「判红那次重定向建出的 s<n> 是空文件，沿用同一个号重跑」，`-output-s3.md` 在这次调用后为空文件（已核 `ls -la` 大小 0） |
| 3（重跑，沿用同一个号） | `-output-s3.md` | 0 | 310 | 23 | `绿 ... 生词=0 拼接=0`（退出码0） | 干净——通读全文未见缺词、断句、孤立标点一类损坏 |

四次调用（含 1 次判红作废）已拿到 2 份干净样本（s2、s3），达到「攒到至少两份干净样本为止」的停止条件，停止调用；未用满「连续五次调用」的额度。

样本自带的行号（样本正文里出现的行序号、`wc -l` 数出的行数等）均为模型自给、未核，不当依据引用。

不写样本答了什么、几份样本方向是否一致——按定义不归本腿的职责，判它打没打中是主 agent 的事。
