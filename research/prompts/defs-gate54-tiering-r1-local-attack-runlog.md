# 运行记录：defs-gate54-tiering-r1-local-attack（2026-09-23 UTC）

提示文件：`research/prompts/defs-gate54-tiering-r1-local-attack.md`（英文，覆盖 K1 五格、K2 四格，
共 9 个标签，每格 WHATHAPPENS + GATE 两段 + 一句 "This would be refuted by:"）。转述核对表：
`research/prompts/defs-gate54-tiering-r1-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，
前台跑，全程未用 `setsid` / `&` / `disown`。跑之前 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查过
`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`：零命中，不是性能测量在跑；
查到本机另有会话在跑 `cargo test`，与本地腿的 HTTP 调用互不相关，未等待。

每份判绿的样本另跑一次 `python3 research/scripts/oov-check.py <样本>`（只给样本一个参数，不给
提示文件），逐一通读复核生词清单；`ask-local.sh` 内部按两个参数调用同一个脚本（样本 + 提示文件），
会把提示自带的行标签（`K1SCOPEGATE`、`BATCHRENAME` 等）从生词表里排除，本记录这一步刻意只给
一个参数，因此下表「拼接」列里出现的 `BATCHRENAME(=batch+rename)` 是提示自己造的标签被复原成
`batch`+`rename` 两个真词，不是模型自己粘的词，逐条在「通读复核」列注明。

## 逐次调用

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py（单参数） | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `defs-gate54-tiering-r1-local-attack-output-s1.md` | 0 | 1685 | 红，生词=32 拼接=3（三次 `BATCHRENAME(=batch+rename)`，均是提示自带标签，非模型自粘） | 绿，全部计数为 0 | 通读发现两处真损坏：第 3 行「gate stage 33 checks the validity of mutation rows in other files (not cratesating the mutations.tsv itself)」里 `cratesating` 不成词，疑似 `crates` 与别的词粘死后又缺字；第 37 行「at the end of a cratesing cratesatch touching crates/」里 `cratesing cratesatch` 两个词都不成词，同一种粘连反复出现；另有第 3 行「the new write scope rule permits the to crates/mutations.tsv」中间缺一个名词（「permits the ___ to」），是缺词类损坏 | 带损坏（不算干净样本） |
| 2 | `defs-gate54-tiering-r1-local-attack-output-s2.md` | 0 | 943 | 红，生词=28 拼接=3（三次 `BATCHRENAME(=batch+rename)`，提示自带标签） | 绿，全部计数为 0 | 通读逐行看过 9 个标签的 WHATHAPPENS/GATE/refuted 三段，无缺词、无断句、无孤立标点、无粘连词；`sequence's`、`overlapped` 是生词表外的正常英文词（oov-check.py 自己的词表只有 29397 词，不是完整词典） | 干净 |
| 3（第一次） | `defs-gate54-tiering-r1-local-attack-output-void1.md` | 5 | （作废副本，模型侧原样输出，`wc -w` 未统计） | 未跑（脚本判红即作废，不进入单参数复核这一步） | 脚本自报：红（`corruption-check.py` 检查模型原样输出这一份，`cjk=0 words=1027…实词自复读=1`，复读词 `appending`；`ask-local.sh` 判定退出码 5） | 未通读（脚本判红即作废） | 判红作废；同一次重定向建出的 `-output-s3.md` 是空文件（0 行），沿用 s3 号重跑 |
| 3（重跑） | `defs-gate54-tiering-r1-local-attack-output-s3.md` | 0 | 893 | 红，生词=24 拼接=4（四次 `BATCHRENAME(=batch+rename)`／`BATCHRENAME's(=batch+rename)`，提示自带标签） | 绿，全部计数为 0 | 通读逐行看过 9 个标签的 WHATHAPPENS/GATE/refuted 三段，无缺词、无断句、无孤立标点、无粘连词 | 干净 |

## 干净样本清点

- 干净：s2、s3（重跑），共 2 份，达到「至少两份干净样本」的门槛，到此停止抽样。
- 带损坏：s1（`cratesating`、`cratesing cratesatch` 两处粘连缺头、外加一处缺词），不当干净样本；
  这份样本脚本自身判绿（退出码 0），损坏是通读加 `oov-check.py` 生词表复核出来的，不是脚本判红——
  与 `.claude/rules/three-way-inference.md:196`「给本地腿的提示一律用英文」一节「闸判绿之后照样
  通读，闸只认得登记过的形态」一致：`cratesating`/`cratesing`/`cratesatch` 这类粘连词切不成两个
  词表词，`oov-check.py` 的规则 A–F 没能把它们归进「拼接」计数，只落进「生词」计数，因此脚本内部
  判定（`red = len(spliced) > 0`）为绿。
- 作废：void1（第一次跑向 s3 号位置的调用），脚本自身判红（退出码 5，`实词自复读=1`：`appending`
  连续出现两次），按定义作废，不算一次观测。
- 全部调用累计 4 次（含 1 次判红作废：s1、s2、void1、s3 重跑），未超过「连续五次调用」的上限；
  退出码分布：0 出现 3 次（s1、s2、s3 重跑），5 出现 1 次（void1）；没有非 0 非 5 的退出码，
  没有网关不通，本地腿全程未缺席。
- 目录核对：`ls research/prompts/ | grep defs-gate54-tiering-r1-local-attack` 显示提示文件 1 份、
  核对表 1 份、运行记录 1 份（本文件）、`-output-s1.md` 至 `-output-s3.md` 共 3 份、
  `-output-void1.md` 1 份，与本记录逐行一致。

## 没做什么

- 不解读、不总结、不采纳三份样本（含带损坏的 s1）里对 K1SCOPEGATE 至 K2REDMARKER 九格的具体
  判词，不判多次抽样之间方向是否一致——那是主 agent 的事。
- 未跑云端攻方腿（Opus，K3 与跨会话/跨工作区历史）、云端正推腿（Sonnet，K1–K3 是不是用户定案
  与规则的直接后果）或核实员，不在这条腿的任务范围内。
- 未碰第三处改动（K3）与主 agent 收尾那一趟的跨会话历史，按分工表不许碰。
- 未读这一轮别的腿的提示与产出（`defs-gate54-tiering-r1-sonnet-output.md`、
  `defs-gate54-tiering-r1-opus-output.md`、`defs-gate54-tiering-r1-opus-model/`、
  `/tmp/claude-1000/defs54-forward/`、`/tmp/claude-1000/defs54-attack/`）与后续可能出现的
  `-verifier-output.md`、`-main-verification.md`——按主 agent 补发的禁读清单，全程未读。
- 未使用 `/tmp/claude-1000/defs54-local/` 草稿目录——这一轮没有需要落草稿的中间产物，
  提示与核对表直接定稿写进 `research/prompts/`，目录留空。
