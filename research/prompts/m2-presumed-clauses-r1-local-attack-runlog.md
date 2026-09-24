# m2-presumed-clauses-r1 本地攻方（K1、K3） 跑记

提示文件：`research/prompts/m2-presumed-clauses-r1-local-attack.md`
翻译核对表：`research/prompts/m2-presumed-clauses-r1-local-attack-translation-audit.md`
网关：`http://127.0.0.1:8200/v1/chat/completions`，模型 id `local`（经 `~/code/ai-center`）。三次调用均前台跑 `nice -n 19 bash research/scripts/ask-local.sh`，不用 `setsid` / `&` / `disown`。跑前用 `ps -o pid,args -u "$(id -u)" | grep -Ei "qemu-system|vm-bench|e152-file-system-benchmark|fio"` 查负载，三次都是空输出（无性能测量在跑）。

## s1 — `m2-presumed-clauses-r1-local-attack-output-s1.md`

- 退出码：0
- 词数（`wc -w`）：537
- `corruption-check.py`：绿（cjk=0 words=548 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0）
- `oov-check.py`：绿（生词=8 拼接=0；列出的生词：overturned）
- 标点：逗号 29、冒号 0、分号 0、句号 40、括号开合 15/15（配对齐）
- 通读结果：第 9 行「This would be overturned if the admission calculation differed produced different results when a valid clean-shutdown marker was present.」——`differed` 与 `produced different results` 并列在一起，读不通，像是编辑时留下的残词，不成句。四类闸认得的损坏形态都不命中（不是复读、不是成对标记落单、不是拼接、不是实词自复读），但按派发指令里「缺词、断句这类损坏……同样记带损坏」，通读时判定为带损坏。
- 判定：带损坏（不算入两份干净样本）

## s2 — `m2-presumed-clauses-r1-local-attack-output-s2.md`

- 退出码：0（首次调用因网关响应较慢超过工具默认 120 秒被工具移入后台跟踪，未使用 `setsid` / `&` / `disown`；等到系统通知任务完成后读取结果，命令本身仍是一次前台调用）
- 词数（`wc -w`）：463
- `corruption-check.py`：绿（cjk=0 words=479 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0）
- `oov-check.py`：绿（生词=11 拼接=0；列出的生词：overturned contradicting recomputing dependence）
- 标点：逗号 25、冒号 8、分号 2、句号 35、括号开合 5/5（配对齐）
- 通读结果：8 条编号答复逐条对应 K1 四行、K3 四行，没有发现缺词、断句、孤立标点这类损坏。
- 判定：干净

## s3 — `m2-presumed-clauses-r1-local-attack-output-s3.md`

- 退出码：0
- 词数（`wc -w`）：407
- `corruption-check.py`：绿（cjk=0 words=410 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0）
- `oov-check.py`：绿（生词=9 拼接=0；列出的生词：overturned candidate's）
- 标点：逗号 22、冒号 0、分号 7、句号 30、括号开合 10/10（配对齐）
- 通读结果：8 条编号答复逐条对应 K1 四行、K3 四行，没有发现缺词、断句、孤立标点这类损坏。
- 判定：干净

## 小结

三次调用、退出码全部为 0，无判红作废副本（因此没有 `-output-void*.md`）。s1 通读发现一处不属于四类闸定义、但符合派发指令「缺词、断句」类的损坏，记为带损坏，不计入干净样本；s2、s3 两份判定为干净，达到「至少两份干净样本」的门槛，停止继续抽样。
