# m2-runner-in-repo-harness-r1 本地攻方 运行记录

提示文件：`research/prompts/m2-runner-in-repo-harness-r1-local-attack.md`（73 行）。
核对表：`research/prompts/m2-runner-in-repo-harness-r1-local-attack-translation-audit.md`（41 行）。
调用方式：前台 `bash research/scripts/ask-local.sh research/prompts/m2-runner-in-repo-harness-r1-local-attack.md > research/prompts/m2-runner-in-repo-harness-r1-local-attack-output-s<n>.md`，未用 `setsid` / `&` / `disown`。

| 样本 | ask-local.sh 退出码 | corruption-check.py 退出码 | oov-check.py 退出码 | wc -w 词数 | corruption-check.py 自报词数 | oov-check.py 生词/拼接 | 分类 |
|---|---|---|---|---|---|---|---|
| s1 | 0 | 0 | 0 | 306 | 305 | 生词=0 拼接=0 | 干净 |
| s2 | 0 | 0 | 0 | 294 | 299 | 生词=1（`verifiesans`）拼接=0 | 带损坏 |
| s3 | 0 | 0 | 0 | 341 | 339 | 生词=0 拼接=0 | 干净 |

s2 判「带损坏」的依据：两道闸都判绿（`verifiesans` 只算生词、没算拼接），但按规则「另外通读一遍样本」的要求手工通读全文，发现样本第 3、10、17 行（行号是样本自带的，模型自给、未核）三处本该是同一句复述（REST-2 那一行，意在说「GATE-56-SCRIPT 只判形式不判内容」），三次写法不一致且前后不成词：第 3 行写成 `GATE-56 verifiesans form, not content`，第 10、17 行写成 `GATE-56 rans form, not content`——`verifiesans`、`rans` 都不是英文词，与其余两份干净样本里同一处稳定写成 `verifies form` 的写法对不上，判定为缺词/断字类损坏，不当干净样本用。

干净样本已达到两份（s1、s3），第三次调用后停止，未继续抽样至五次上限。

## 没做什么

- 没有解读三份样本回答了什么、三份之间方向是否一致——判它打没打中、三份内容怎么用是主 agent 的事，不归本报告。
- 没有对 s2 重跑或另占号「补救」——按规则退出码 0 的每次调用各占一个号，带损坏的也占号、文件留着，不重跑覆盖。
- 没有读 `research/prompts/m2-runner-in-repo-harness-r1-opus-output.md` 与 `-sonnet-output.md`：本轮派发消息没有给出这一轮的禁读清单（这是派发缺项，已在交回里向主 agent 报告），本报告按「各条腿必须互不重复」自行避开这两份云端腿的产物，未读、未引。
