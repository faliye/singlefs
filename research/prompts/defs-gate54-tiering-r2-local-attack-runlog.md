# 运行记录：defs-gate54-tiering-r2-local-attack（2026-09-24 UTC）

提示文件：`research/prompts/defs-gate54-tiering-r2-local-attack.md`（英文，覆盖 V3 九格、
V4 四格，共 13 个标签，每格 TRACE/VERDICT 或 REPLAYRUN/REUSEDECISION/AGREE（V4NARROW、
V4WRONGPATH 另加 LATERBATCH）+ 一句 "This would be refuted by:"）。转述核对表：
`research/prompts/defs-gate54-tiering-r2-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，
前台跑，全程未用 `setsid` / `&` / `disown`。开跑前 `ps -o pid,args -u "$(id -u)"` 查过
`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`：零命中，不是性能测量在跑；
本机另有会话在跑 `cargo`（`impl-m2-instpage` 下的编译进程），与本地腿的 HTTP 调用互不相关，
未等待。四次调用分别耗时约 2 分 4 秒、1 分 33 秒、1 分 25 秒、1 分 58 秒，均在 Bash 单次
调用的时限内完成，未触发超时。

每份判绿的样本另跑一次 `python3 research/scripts/oov-check.py <样本>`（只给样本一个参数，
不给提示文件），逐一通读复核生词清单；`ask-local.sh` 内部按两个参数调用同一个脚本
（样本 + 提示文件），会把提示自带的标签从生词表里排除，本记录这一步刻意只给一个参数，
因此下表「拼接」列里出现的 `REUSEDECISION(=reuse+decision)`、`LATERBATCH(=later+batch)`
等，是这份提示自己造的标签（`REPLAYRUN`、`REUSEDECISION`、`LATERBATCH`、`WRONGPATH`、
`SELFCHANGE`）被机械拆回两个真词，不是模型自己粘的词，逐条在「通读复核」列注明。

## 提示中途改过一次（第 2 次调用判红之后、第 3 次调用之前）

第 1、2 次调用都判红，`corruption-check.py` 报「实词自复读：agree | agree」。通读作废副本
发现：这不是模型真的把同一个词连打两遍，而是提示自己的段落标签 `AGREE`（大写）后面，模型
的答案原文以小写单词 `agree` 开头（例如 `AGREE agree`、`AGREE agree because both indicate
the batch must be checked`），标签与答案首词恰好是同一个词的两种大小写，被逐字节比对的
`实词自复读` 检查判成同一个词连续出现两次——这是提示自己的标签措辞与预期答案内容重名造成
的机械触发，不是模型答复方向不一致，也不算「三方不一致」。第 2 次判红之后、第 3 次调用
之前，在 `FORMAT RULES` 一节加了一句：写 `AGREE` 这一部分时，答案的第一个词不许是单独的
`agree`，要先用几个别的词把「同不同意」写成一整句话再往下写。改动只加了这一句，未删改任何
一条 FACT 或任何一行 ROW 定义；改动前后的旧串/新串见下方「提示改动」一段，改动前两次调用
（void1、void2）不计入干净样本、也不当反例采纳。改后提示 sha256
`7b170cd152ce6fa69edc28fc3cb3ac92e9d5d2effd45f5b545c8823fa2a73273`（第 3、4 次调用共用同一份）。

提示改动（`research/scripts/replace-once.py`，命中 1 次）：
旧串结尾「...by the same refute sentence. Do not answer any part with only a single word...」
改成「...by the same refute sentence. When you write the AGREE part specifically, never let
your own very first word of that part's answer be the bare word agree by itself; put at
least a few other words first, for example by saying whether they agree or disagree as part
of a full sentence, before that word, if you use that word at all. Do not answer any part
with only a single word...」。

## 逐次调用

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py（单参数） | corruption-check.py（脚本内部） | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `defs-gate54-tiering-r2-local-attack-output-void1.md` | 5 | 1073（脚本报的词数，原样输出） | 未跑（脚本判红即作废，不进入单参数复核这一步） | 脚本自报：红（`实词自复读=2`：`agree \| agree`） | 未通读（脚本判红即作废） | 判红作废；同一次重定向建出的 `-output-s1.md` 是空文件，沿用 s1 号重跑 |
| 2（沿用 s1 号，仍判红） | `defs-gate54-tiering-r2-local-attack-output-void2.md` | 5 | 1043（脚本报的词数，原样输出） | 未跑 | 脚本自报：红（`实词自复读=2`：`agree \| agree`） | 未通读（脚本判红即作废） | 判红作废；`-output-s1.md` 再次是空文件；判红原因见上方「提示中途改过一次」，改完提示再沿用 s1 号重跑 |
| 3（s1，提示已改） | `defs-gate54-tiering-r2-local-attack-output-s1.md` | 0 | 1192 | 红，生词=11 拼接=5（`REUSEDECISION(=reuse+decision)` ×4、`LATERBATCH(=later+batch)` ×1，全部是提示自带的段落标签被拆回两个真词，非模型自粘） | 绿（脚本内部判定，退出码 0 即两项检查都过） | 通读逐行看过 13 个标签的答案，13 个标签按 V3LAYER0 至 V4SELFCHANGE 的次序齐全；未发现缺词、断句、孤立标点、真正的粘连词；V4WRONGPATH 一格没有单独给 LATERBATCH 部分（其余三个 V4 格式要求给的部分都齐），这是内容完整度的事，不算字词损坏 | 干净 |
| 4（s2） | `defs-gate54-tiering-r2-local-attack-output-s2.md` | 0 | 1064 | 红，生词=14 拼接=7（`REUSEDECISION(=reuse+decision)` ×4、`LATERBATCH(=later+batch)` ×2、另一处 `REUSEDECISION`，全部是提示自带的段落标签，非模型自粘） | 绿 | 通读逐行看过 13 个标签的答案，13 个标签按 V3LAYER0 至 V4SELFCHANGE 的次序齐全；未发现缺词、断句、孤立标点、真正的粘连词；V4NARROW 与 V4WRONGPATH 两格的 LATERBATCH 部分都给了 | 干净 |

## 干净样本清点

- 干净：s1、s2，共 2 份，达到「至少两份干净样本」的门槛，到此停止抽样。
- 作废：void1、void2（第一次与第二次调用，都沿用 s1 号），脚本自身判红（退出码 5，
  `实词自复读=2`：`agree | agree`，起因见上方「提示中途改过一次」），按定义作废，不算
  一次观测；两次作废副本由脚本留存，本记录未删。
- 全部调用累计 4 次（含 2 次判红作废），未超过「连续五次调用」的上限；退出码分布：
  0 出现 2 次（s1、s2），5 出现 2 次（void1、void2）；没有非 0 非 5 的退出码，没有网关不通，
  本地腿全程未缺席。
- 目录核对：`ls research/prompts/ | grep defs-gate54-tiering-r2-local-attack` 显示提示文件
  1 份、核对表 1 份、运行记录 1 份（本文件）、`-output-s1.md`、`-output-s2.md` 共 2 份、
  `-output-void1.md`、`-output-void2.md` 共 2 份，与本记录逐行一致。

## 没做什么

- 不解读、不总结、不采纳两份样本（s1、s2）里对 V3LAYER0 至 V4SELFCHANGE 十三格的具体
  判词，不判两次抽样之间方向是否一致——那是主 agent 的事。
- 未跑云端攻方腿（Opus，V1、V2）、云端辩方腿（Sonnet，V5）或核查员，不在这条腿的任务
  范围内；未读这一轮别的腿的提示与产出（`defs-gate54-tiering-r2-opus-output.md`、
  `defs-gate54-tiering-r2-opus-model/`、`defs-gate54-tiering-r2-sonnet-output.md`、
  `defs-gate54-tiering-r2-verifier-output.md`）——按主 agent 给的禁读清单，全程未读。
- 未碰第一轮判决站不站得住（V5）与暂存区/worktree 相关的历史（V1、V2），按分工表不许碰。
- 未使用 `/tmp/claude-1000/defs54-r2-local/` 草稿目录里的内容作为任何产出的依据——那里只
  放了核对提示原文行号时用的两份临时抠取文件（`old1.txt`、`new1.txt`），供 `replace-once.py`
  定点替换使用，不是这一轮的观测数据，提示与核对表定稿都已落进 `research/prompts/`。
