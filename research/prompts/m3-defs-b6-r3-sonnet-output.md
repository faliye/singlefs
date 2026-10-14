# m3-defs-b6-r3 正推腿（Sonnet）报告

日期 2026-10-13。格 Y4、Y5、Y6。立场：逐条核「两轮判决与正文说的数和清单，就是产物与冻结副本里的样子」。
记号：R1、R2、R3 = 第一、二、三轮冻结副本（`~/.local/share/singlefs/work/m3-defs-b6-r{1,2,3}/repo`，R3 是 after-b8-fixup），B3 = 零轮之前那一版（`~/.local/share/singlefs/work/m3-defs-b3-r2/repo`）。引冻结副本里的文件一律写绝对路径加那份文件自己的行号；引主工作区 `research/prompts/` 与 `records/` 写仓内相对路径。
模型目录：`research/prompts/m3-defs-b6-r3-sonnet-model/`（`tools/rerun.sh` 加 7 个小工具，`out/` 存 12 份现算输出）。

## 0 方法与边界

- 开工先读了 `.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/implementation-first.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`（开头三步表），再读背景材料与正文。没读这一轮攻方腿的报告与模型目录。
- 门禁格（`--check`）子 agent 起不了。凡是格的结局，我用两种代跑：一是把格里内嵌的 python 抽出来在被判树上原样跑（`tools/run_htd_cell.py`，不起 `gate.sh`），二是直接调库（`lib_abandoned_rounds.py`、`lib_long_run.py exempt-check`）。表里标「代跑」的就是这两种；格本身的退出码仍归主 agent。
- 冻结副本只读：我的命令都在草稿目录的拷贝里跑，或只读冻结副本；跑完逐份查 `*.pyc`，没有比今天开工更新的（冻结副本里现有的 `.pyc` 都是前几轮留下的，日期早于第二轮正文）。
- 私有配置：`multi-host.env` 一个值都没读出来、没写；`tools/pv_counts.py` 只用冻结副本 `doc-text.sh` 里的取词函数数个数。
- 开跑前 `ps` 看负载：有别的会话的 `cargo test`（经内存包装）与 harness 测试在跑，没有 `qemu-system`、`vm-bench`、`e152-file-system-benchmark`、`fio`；我的命令全加 `nice -n 19`，没起 cargo，线程上限 31 用不上。
- 复跑一律经 `bash scripts/hosts/run-with-memory-cap.sh 2G`，外面套 `scripts/hosts/capped.sh 31`。前台单条命令超过 120 秒时系统自动转后台（三次，都是 `r2-sonnet` 那条旧腿的复跑），我没起后台命令，等它完成通知。

证据代号（`research/prompts/m3-defs-b6-r3-sonnet-model/out/` 下的文件，`tools/rerun.sh` 现算）：

| 代号 | 文件 | 内容 |
|---|---|---|
| E1 | `summary-counts.txt` | `verify4/6/8/10.summary` 行数与判错行、`verify7/9-out` 弄坏开关日志数与 ✗ 行数、`verify11-out` 四份自证与两个新开关、各轮 `cells.log` 的扫描行、真树红行数 |
| E2 | `patch-counts.txt` | 批 6 至批 8 十份补丁的文件数、加减行、sha256 前后缀；六个提交的 `--shortstat`；各标签排除表行数 |
| E3 | `snapshot-check.txt` | 三份冻结副本对各自快照的 `sha256sum -c`、对 B3 的差异条目数、四份副本的排除表总行数与数据行数 |
| E4 | `cite-check.txt` | 两轮判决写的三条 `cite-check.py` 末行 |
| E5 | `htd-cell.txt` | heavy-thread-defaults 格本体在四份副本真树上的扫描行（代跑）与我自己数的文件数 |
| E6 | `pv-counts.txt` | 私有配置的词数、带 / 的词数（只打个数） |
| E7 | `model-reruns-*.txt` | 四条旧腿各自 `rerun.sh` 在拷贝上复跑的 same / DIFF 数 |
| E8 | `htd-samples.txt` | 第一轮攻方 V3 样本、第二轮攻方 X2 样本在 R3 真树的判法下点名了哪些文件 |
| E9 | `y4-feed.txt`、`y4-grep.txt` | V2 的派发闸喂法（14 例）与 V2、V4 的 grep 计数 |

判的写法：对得上 / 对不上 / 对得上但口径缺（口径栏写缺什么）/ 复核不了（写原因）。引文都是该行的原话，行号现从文件里找（`tools` 之外的生成脚本在草稿目录，产出逐行经 `cite-check.py` 核过）。

## 1 Y6：两轮判决、三轮正文与调度记录里这一题引的计数

范围：`research/prompts/m3-defs-b6-r1-main-verification.md`、`m3-defs-b6-r2-main-verification.md`、`_m3-defs-b6-r{1,2,3}-body.md`、`records/2026-10-12-里程碑三工具与治理批.md` 第 128 至 146 行（批 6 收尾以来的行，其中 128 至 130 行是批 6，被第一轮正文引作「实现今天的样子」，一并核）。「第一轮」「第二轮」这类序数、改法编号 L1–L7、M1–M7、格名 V1–V8 不当计数。一行一个数；同一个数在几处被引，各占一行。

| 编号 | 引处 | 现算（命令或产物） | 现算结果 | 判 | 口径缺什么 |
|---|---|---|---|---|---|
| Y6-01 | `research/prompts/m3-defs-b6-r1-main-verification.md:5`：「sha256 e34887bc…9931」 | `sha256sum` 正推报告 | e34887bc…9931 | 对得上 | — |
| Y6-02 | `research/prompts/m3-defs-b6-r1-main-verification.md:5`：「65be73af…9314」 | `sha256sum` 攻方报告 | 65be73af…9314 | 对得上 | — |
| Y6-03 | `research/prompts/m3-defs-b6-r1-main-verification.md:11`：「27 份非 OK 0 行」 | E3：清单行数；`sha256sum -c` | 清单 27 行，27 OK，其余 0 | 对得上 | 27 是清单行数，也等于 `diff -rq` 对零轮之前去掉 fixtures 的条目数（全部条目 46）；「动过的」按哪一种数没写 |
| Y6-04 | `research/prompts/m3-defs-b6-r1-main-verification.md:12`：「核了 120 处引文，对上 120 处，没判 0 处」 | E4：`cite-check.py` 两份报告 `--root` 第一轮冻结副本 `--background` 第一轮背景材料 | 120 / 120 / 0 | 对得上 | 判决没写 `--background` 的路径；不带它的数没算 |
| Y6-05 | `research/prompts/m3-defs-b6-r1-main-verification.md:13`：「十二行十一 same」 | E7：正推腿 `rerun.sh`，2G 包装 | 12 个文件 11 same，`lr.out` differs | 对得上 | 我复现出的差是长说明按字节截断的位置不同（我的工作路径更长，两边路径都已换成占位）；主 agent 那一次的原因我看不到，判决写的是「只差路径没换成占位」 |
| Y6-06 | `research/prompts/m3-defs-b6-r1-main-verification.md:13`：「21 份全 same」 | E7：攻方腿 `rerun.sh`，2G 包装，设 `PRE_ZERO_FROZEN` | 五次里三次 21 same（直接起）；经 `tools/rerun.sh` 嵌套起的两次 20 same + 1 DIFF（`v7-path-probe.txt` 里 14350 对 14352 个文件，工作路径更长） | 对得上，但不稳 | 那份输出数拷贝树里的文件，杂散文件（缓存）多两个就差；「全 same」依赖干净的工作目录。腿报的 8G 排队不起，复核不了 |
| Y6-07 | `research/prompts/m3-defs-b6-r1-main-verification.md:14`：「九份」 | A：攻方腿 `v3-b3.txt` 对 `v3-b6.txt` | 14 处对 5 处，多出的 9 份名字与判决所列一一对上 | 对得上（抽格里的 python 代跑；门禁格本身复核不了） | 数的是被点名的文件，不是被判行数 |
| Y6-08 | `research/prompts/m3-defs-b6-r1-main-verification.md:14`：「输出里出现样本的（编的）私有值 1 处」 | A：攻方腿 `v7-b6.txt`、`v7-b3.txt` | 零轮那一版红行 1 行带出整条编的路径；零轮之前退 0、判绿 | 对得上（抽格 python 代跑） | 「1 处」是 1 个文件路径行；值是第一轮攻方编的样本值 |
| Y6-09 | `research/prompts/m3-defs-b6-r1-main-verification.md:22`：「九种自然写法」 | A：数攻方样本里读线程变量的文件 | 9 份文件（`wrap_helper.py` 在内）；括注只列了 8 个名目 | 对得上（按文件数） | 「种」按文件数，括注没列 `wrap_helper.py` 那一种 |
| Y6-10 | `research/prompts/m3-defs-b6-r1-main-verification.md:26`：「真配置 8 个长词里 6 个」 | E6：用冻结副本 `doc-text.sh` 取词函数，只打个数 | 门槛 8 个字；配置取到 7 词（带 / 6 个），本机主机名 1 词；合计 8，带 / 6 | 对得上 | 「8 个」含本机主机名，配置本身 7 个；「长词」是 8 个字以上 |
| Y6-11 | `research/prompts/m3-defs-b6-r1-main-verification.md:27`：「E171 六份」 | E2：豁免表数据行；`ls` 目录数以 multi_host.sh 结尾的 | 表 6 行；目录里 6 份入口（另有两份 `fake_multi_host_run`，不算） | 对得上 | — |
| Y6-12 | `research/prompts/m3-defs-b6-r1-main-verification.md:27`：「E171 六份」 | A：六份现行 sha256 对 `m3-leaf-death-r3-snapshot/tree-sha256.txt` 里的钉（正推腿 `lr.out`） | 6 份全 DIFF | 对得上 | 「找得到的钉」只指这一份清单；别处有没有钉没找 |
| Y6-13 | `research/prompts/m3-defs-b6-r1-main-verification.md:27`：「第 55 行」 | A：`awk 'NR==55' .claude/main-agent.md` | 三份冻结副本第 55 行都是 ⑭ 长活那一段 | 对得上（行号） |  |
| Y6-14 | `research/prompts/m3-defs-b6-r1-main-verification.md:36`：「登记表里现有的四行」 | A：`abandoned-rounds.tsv` 数据行；库里 `LEGACY_SOURCELESS_ROUNDS` | 表 5 行（三份冻结副本同），名单 4 个名字 | 对得上 | 「现有」指无出处的四行，不是表的全部；第 5 行 m3-batch63-code-r1 带出处 |
| Y6-15 | `research/prompts/m3-defs-b6-r1-main-verification.md:41`：「三种新类型」 | A：`THREAD_LIMIT_REQUIRED_AGENTS` 九种减 `…_BEFORE_CLERK` 六种 | 多出 clerk、three-way-forward、three-way-defense 三种 | 对得上 | — |
| Y6-16 | `research/prompts/m3-defs-b6-r2-main-verification.md:5`：「sha256 badae1cd…0efc」 | `sha256sum` 辩方报告 | badae1cd…0efc | 对得上 | — |
| Y6-17 | `research/prompts/m3-defs-b6-r2-main-verification.md:5`：「96e8e4b0…7bec」 | `sha256sum` 攻方报告 | 96e8e4b0…7bec | 对得上 | — |
| Y6-18 | `research/prompts/m3-defs-b6-r2-main-verification.md:11`：「各自快照非 OK 0 行」 | E3：`sha256sum -c` | 第二轮 33 行 33 OK，第一轮 27 行 27 OK | 对得上 | — |
| Y6-19 | `research/prompts/m3-defs-b6-r2-main-verification.md:11`：「比正文新的 .pyc 0 份」 | A：`find <冻结副本> -name '*.pyc' -newer _m3-defs-b6-r2-body.md` | 第一轮、第二轮冻结副本各 0 份（第一轮副本里 1 份 .pyc 比第二轮正文旧） | 对得上 | 「正文」指第二轮正文才对得上；判决没写 |
| Y6-20 | `research/prompts/m3-defs-b6-r2-main-verification.md:12`：「85 处对上 56、没判 29」 | E4：`cite-check.py` 两份报告 `--root` 第二轮冻结副本 | 85 / 56 / 29；29 条没判全是引 `research/prompts/` 下的文件 | 对得上 | 判决括注「都是引主工作区 research/prompts/ 下的判决与模型输出」与产物一致 |
| Y6-21 | `research/prompts/m3-defs-b6-r2-main-verification.md:12`：「核了 85 处引文，对上 85 处，没判 0 处」 | E4：同上，`--root` 换主工作区 | 85 / 85 / 0 | 对得上 | 换根后第二轮冻结副本里的引文改对着主工作区的同路径文件判，这 56 条对上的是同一行；它不单独证明冻结副本 |
| Y6-22 | `research/prompts/m3-defs-b6-r2-main-verification.md:13`：「25 份二十三 same，两份 x6」 | E7：攻方腿 `rerun.sh` 设 `R1_FROZEN`、不设 `B3_FROZEN`；再设 `B3_FROZEN` 一次 | 不设：25 份 23 same + 2 DIFF；设：26 份 24 same + 2 DIFF；两份 DIFF 都是 x6 的长说明按字节截断 | 对得上 | 25 取决于没设 `B3_FROZEN`；模型目录 `out/` 存 26 份；`rerun.sh` 里没写这个默认 |
| Y6-23 | `research/prompts/m3-defs-b6-r2-main-verification.md:13`：「45 份全 same」 | E7：辩方腿 `rerun.sh`，2G 包装，两趟各 45 same | 45 same，0 DIFF | 对得上 | — |
| Y6-24 | `research/prompts/m3-defs-b6-r2-main-verification.md:14`：「仍把（编的）私有值原样打出 3 处」 | A：攻方腿 `x5-siblings.txt`（r2 副本） | clock-times 红行 1、prime-marks「二进制：」1、session-names「二进制：」1，共 3 行 | 对得上（抽格 python 代跑） | 「处」数的是三格各一行；同一个文件名在 private-values 那一格已打码 |
| Y6-25 | `research/prompts/m3-defs-b6-r2-main-verification.md:14`：「v3 九种写法第二轮全点名（第一轮只点名 3 处）」 | A：辩方腿 `v3-after.txt`、`v3-before.txt` | 第二轮 15 处，九份全在；第一轮 5 处（2 份库 + `tests.rs`、`baseline_read.py`、`if_default.sh`），九份一份都不在 | 九种对得上；「3 处」口径不清 | 括注读作「九种里只点了 3 处」会错：那 3 处不属于九种，九种在第一轮是 0 份 |
| Y6-26 | `research/prompts/m3-defs-b6-r2-main-verification.md:14`：「x2 的 8 份误判样本」 | A：辩方腿 `x2-fp-after.txt`、`x2-fp-before.txt` | 第二轮 8 份样本全点名；第一轮 0 份（只剩 2 份库）；样本目录 9 份，`fp_docstring.py` 不点名（对） | 对得上 | 括注列 7 个名目对 8 份文件（rust 比较与 match 两份） |
| Y6-27 | `research/prompts/m3-defs-b6-r2-main-verification.md:14`：「exempt-table 树红在缺路径、已写齐、sha256 对不上、旧格式四处」 | A：辩方腿 `make_gate_trees.sh` 的 exempt-table 树，`lib_long_run.py exempt-check` | 「有 4 处不对（共 5 行）」，四处正是这四类 | 对得上 | 门禁格没起，库调用与格是同一个函数 |
| Y6-28 | `research/prompts/m3-defs-b6-r2-main-verification.md:14`：「abandoned-rounds 六棵树五棵与推的相同」 | A：辩方腿六棵树的库读数 | 六棵：legacy-sourceless 不收、legacy-listed 收、other-name-unrecognised 收、other-name-recognised 不收、other-form-nomaterial 收、symlink-source 不收；与判决所列五棵一致 | 对得上（库读数） | 门禁格本身复核不了（归主 agent） |
| Y6-29 | `research/prompts/m3-defs-b6-r2-main-verification.md:14`：「X2 样本第二轮多点名 or_literal_default.sh」 | A：攻方腿 `x2-sample-r2.txt`、`x2-sample-r1.txt` | 第二轮 6 处，第一轮 5 处，多出的正是 `or_literal_default.sh` | 对得上 | 同一批输出里 `or_var_default.sh`、`or_arith_default.sh` 两版都不点名，零轮之前那一版点名（判决写作「两版都不点名」） |
| Y6-30 | `research/prompts/m3-defs-b6-r2-main-verification.md:15`：「O-3（排除表比第一轮多 2 行）」 | E3：排除表数据行；两版键集差 | 数据行第一轮 66、第二轮 66；总行数 81 对 83（多的是 1 行注释 1 行空行）；键（路径:锚串）第二轮新 2 个、少 2 个 | 对不上 | 「多 2 行」说成行数不对；对的说法是相对第一轮基准多了 2 个键、同时少了 2 个，净行数不变 |
| Y6-31 | `research/prompts/m3-defs-b6-r2-main-verification.md:21`：「第 20 行仍把 clerk 叫」 | A：`awk 'NR==20' scripts/agents/dispatch-prompt.py` | 第一轮、第二轮冻结副本第 20 行是「书记员」；第三轮已是 clerk | 对得上 |  |
| Y6-32 | `research/prompts/m3-defs-b6-r2-main-verification.md:22`：「8 种不读环境的写法」 | 同 Y6-26 | 8 份文件 | 对得上 | — |
| Y6-33 | `research/prompts/m3-defs-b6-r2-main-verification.md:23`：「48 例 env 矩阵」 | A：辩方腿 `v5-env-matrix.txt` 行数 | 48 行 | 对得上 | 「30 应拒、18 应放」是辩方报告里的拆法，判决没写 |
| Y6-34 | `research/prompts/m3-defs-b6-r2-main-verification.md:23`：「第 53 行」 | A：`awk 'NR==53' .claude/rules/implementation-workflow.md` | 第 53 行是「用户要求」一行，写「任务确实要跑时主 agent 先弹窗问用户」 | 行号对，括注里的引文不是这一行的原话 | 判决括注「提交之外要跑先弹窗问用户」是钩子拒绝说明里的话，在 `implementation-workflow.md` 里零命中（`grep -cF`） |
| Y6-35 | `research/prompts/m3-defs-b6-r2-main-verification.md:25`：「6 种路径形状无误报」 | A：辩方腿 `x5-shapes.txt` | 6 份样本；第二轮 0 行含假值、6 处打码，对照与后半段未被点名 | 对得上（抽格 python 代跑） | — |
| Y6-36 | `research/prompts/m3-defs-b6-r2-main-verification.md:33`：「写进共用约束与九份定义」 | A：`grep -lF '线程上限' .claude/agents/*.md` | 第三轮副本 9 份；类型表 `THREAD_LIMIT_REQUIRED_AGENTS` 9 种 | 对得上 | 第二轮副本未单独数 |
| Y6-37 | `research/prompts/_m3-defs-b6-r1-body.md:11`：「类型表九种」 | 同 Y6-36 | 9 种 | 对得上 | — |
| Y6-38 | `research/prompts/_m3-defs-b6-r1-body.md:22`：「27 份文件」 | 同 Y6-03 | 27 | 对得上 | — |
| Y6-39 | `research/prompts/_m3-defs-b6-r1-body.md:27`：「148 项」 | E1：`verify4.summary` 行数 | 148 行，末行「派 148 项、收 148 项、判错 1 项」（第 6 项 runner-dispatch-guard 该绿而红） | 对得上 | 正文没写这一份判错 1 项；后来的批 5 收尾已处理 |
| Y6-40 | `research/prompts/_m3-defs-b6-r1-body.md:27`：「168 项判错 1 项」 | E1：`verify6.summary` | 168 行，判错 1 行（第 138 项，开关打开后样本档不红） | 对得上 | — |
| Y6-41 | `research/prompts/_m3-defs-b6-r1-body.md:27`：「15 个与 long-run-exempt 3 个开关全红」 | E1：`verify7-out/` 日志数与每份日志里的 ✗ 行数 | heavy-thread-defaults 开关日志 15 份，long-run-exempt 3 份，每份 ✗ 行 3 至 14 行 | 对得上 | 「红」是日志里有 ✗ 行；`verify7-out` 没存退出码文件，退出码这一半复核不了；冻结副本 `code-tooling.sh` 里 thread-default 开关名 15 个、long-run-exempt 3 个，与日志份数一致 |
| Y6-42 | `research/prompts/_m3-defs-b6-r2-body.md:23`：「33 份文件」 | 同 Y6-18 | 33 | 对得上 | — |
| Y6-43 | `research/prompts/_m3-defs-b6-r2-body.md:28`：「184 项判错 4 项：1 项真红是两行陈旧排除锚串，3 项是复核脚本直接设库开关被门禁格覆盖」 | E1：`verify8.summary` 判错行；`verify8-out/015-cells-true-tree.log`；第二轮副本 `code-tooling.sh` 第 2422 至 2429 行 `library_break` 的映射 | 184 行；判错 4 行（第 15、34、35、36）；第 15 项日志「有 2 处不对」第 30、33 行锚串；34–36 是 `LIB_LONG_RUN_BREAK=exempt-*` 直接设，门禁格把 `LIB_LONG_RUN_BREAK` 重设成由 `CODE_TOOLING_BREAK` 映射来的值（默认空串），同名门禁开关第 156–158 项都红 | 对得上 | — |
| Y6-44 | `research/prompts/_m3-defs-b6-r2-body.md:28`：「long-run-exempt 四个开关各自红」 | E1：`verify9-out/brk-*.log` | 4 份日志（growth、needless、path、sha），每份 ✗ 行 3 或 4 行 | 对得上 | 同 Y6-41 的「红」口径 |
| Y6-45 | `research/prompts/_m3-defs-b6-r2-body.md:28`：「三格在集成真树上绿」 | E1：`verify9-out/cells.log` | gate-structure、heavy-thread-defaults、long-run-header-exempt 三格绿 | 对得上 | — |
| Y6-46 | `research/prompts/_m3-defs-b6-r3-body.md:13`：「33 份文件」 | 同 Y6-18 | 33 | 对得上 | — |
| Y6-47 | `research/prompts/_m3-defs-b6-r3-body.md:13`：「这四份冻结副本」 | A：`ls` 四个目录 | 第一、二、三轮冻结副本与零轮之前那一版，四个都在 | 对得上 | — |
| Y6-48 | `research/prompts/_m3-defs-b6-r3-body.md:17`：「默认 31」 | A：`.claude/heavy-thread-default` 第一条非注释行 | 31 | 对得上 | — |
| Y6-49 | `research/prompts/_m3-defs-b6-r3-body.md:18`：「48 例 env」 | 同 Y6-33 | 48 | 对得上 | — |
| Y6-50 | `research/prompts/_m3-defs-b6-r3-body.md:22`：「verify8 184 项、verify10 191 项」 | E1 | 184、191 | 对得上 | — |
| Y6-51 | `research/prompts/_m3-defs-b6-r3-body.md:27`：「191 项判错 5 项：1 项真红是三处自证样本缺排除行，4 项是复核脚本直接设库开关被门禁格覆盖」 | E1：`verify10.summary` 与 `verify10-out/015-cells-true-tree.log` | 191 行；判错 5 行（第 15、33、34、35、36）；第 15 项日志 3 处：`runner-dispatch-guard.sh:1872`、`:1873`、`detached-run.sh:1798`；33–36 同上；同名门禁开关第 158–162 项都红 | 对得上 | — |
| Y6-52 | `research/prompts/_m3-defs-b6-r3-body.md:27`：「四份自证绿」 | E1：`verify11-out/` 的 `rdg.log`、`dp.log`、`hostd.log`、`detached.log` 末行 | 派发闸 377 种、dispatch-prompt 64 项、host-dispatch 34 格、detached-run 49 格，四份都是 ✓ | 对得上 | 正文把派发闸、detached-run、host-dispatch、dispatch-prompt 并称；它们的数各用各的单位（种、项、格） |
| Y6-53 | `research/prompts/_m3-defs-b6-r3-body.md:27`：「各红 7 格、3 格」 | E1：`verify11-out/rdg-dup.log`、`rdg-cb.log` 的 ✗ 自检行 | 7 行、3 行 | 对得上 | 两份日志文件名带 dup、cb，没记是哪个开关起的；开关名从日志对不上，只能按文件名与自检名推（重复标签 7 格、代码块 3 格） |
| Y6-54 | `research/prompts/_m3-defs-b6-r3-body.md:22`：「扫 832 份」 | E5：`scan_count.py` 与格本体（抽格里的 python 代跑）在第三轮冻结副本 | 832（scripts 455、.claude/gate.d 24、.claude/hooks 17、crates 335、cargo-command-prefix 1） | 对得上 | 同一数在第一、二、三轮副本都是 832，零轮之前那一版 831 |
| Y6-55 | `records/2026-10-12-里程碑三工具与治理批.md:128`：「23 份 +468/−57」 | E2：`git apply --numstat` 批 6 补丁；`git show --shortstat` 批 6 提交 | 23 个文件，+468 −57（提交同） | 对得上 |  |
| Y6-56 | `records/2026-10-12-里程碑三工具与治理批.md:128`：「扫 832 份、可疑写法 94 处、排除表 66 行」 | E5：第二轮冻结副本无基准跑格本体；`verify7-out/cell.log` | 832 / 94 / 66（第一轮、第二轮副本都是） | 对得上（批 6 收尾那一刻） | 这三个数后来变了：批 8 收尾之后可疑写法 117、表 89 行（见 Y6-92）；记录里没写是哪一版树 |
| Y6-57 | `records/2026-10-12-里程碑三工具与治理批.md:128`：「72 → 58」 | E5：零轮之前副本 72，第一轮副本 58，第二轮副本 72，第三轮副本 74 | 72 → 58 是批 6 收窄的样子；批 7 的 L2 退回宽判法后又回到 72 | 对得上（批 6 收尾那一刻） | 「装置自己读的线程变量」数随判法变，窄判法下 58；引它的地方没标判法版本，后读者拿 58 当今天的数会错 |
| Y6-58 | `records/2026-10-12-里程碑三工具与治理批.md:128`：「少的 14 个逐个看过都不是读环境」 | 算术 72 − 58 | 14 | 算术对得上；「逐个看过都不是读环境」复核不了 | 那一半是人工逐个看，没留清单 |
| Y6-59 | `records/2026-10-12-里程碑三工具与治理批.md:128`：「第 59、60 行」 | A：`awk` 取 `.claude/rules/verification.md` 第 59、60 行 | 第一、二、三轮副本这两行都是 GPU 测试线程那两条 | 对得上（行号） | — |
| Y6-60 | `records/2026-10-12-里程碑三工具与治理批.md:129`：「派 168 项、收 168 项、判错 1 项」 | 同 Y6-40 | 168 / 168 / 1 | 对得上 | — |
| Y6-61 | `records/2026-10-12-里程碑三工具与治理批.md:129`：「第 138 项退 0」 | E1：`verify6.summary` | 第 138 项 `brk-code-tooling-thread-default-test-module-unskipped` 期望红、退 0 | 对得上 | — |
| Y6-62 | `records/2026-10-12-里程碑三工具与治理批.md:129`：「其余 167 项对」 | 算术 168 − 1 | 167 | 对得上 | — |
| Y6-63 | `records/2026-10-12-里程碑三工具与治理批.md:130`：「heavy-thread-defaults 15 个与 long-run-exempt 3 个开关全红」 | 同 Y6-41 | 15 / 3 | 对得上 | — |
| Y6-64 | `records/2026-10-12-里程碑三工具与治理批.md:131`：「快照为零轮形态动过的 27 份」 | 同 Y6-03 | 27 | 对得上 | — |
| Y6-65 | `records/2026-10-12-里程碑三工具与治理批.md:134`：「87 句一致 56、不一致 12、文字没写到 19」 | A：数正推报告第 1 至 5 节表的行首编号，同一编号去重 | 93 行、87 个编号：一致 56、不一致 12、缺 19 | 对得上 | 报告里第三类叫「缺」；它含 V2 两句（AC-7、MA-6，「钩子判不了」）和 DR-2 一类「没有机器核」，判决把 V2 判「站得住」，把「缺」改叫「文字没写到」，两种意思混在 19 里 |
| Y6-66 | `records/2026-10-12-里程碑三工具与治理批.md:134`：「复跑十二行十一 same」 | 同 Y6-05 | 11 same / 12 | 对得上 | — |
| Y6-67 | `records/2026-10-12-里程碑三工具与治理批.md:135`：「21 份全 same」 | 同 Y6-06 | 21 same（三次直接起），20 + 1 DIFF（两次经 `rerun.sh` 嵌套起） | 对得上，但不稳 | — |
| Y6-68 | `records/2026-10-12-里程碑三工具与治理批.md:135`：「漏 9 种读法」 | 同 Y6-07 | 9 | 对得上 | — |
| Y6-69 | `records/2026-10-12-里程碑三工具与治理批.md:128`：「32 份」 | E2：批 7 甲补丁 | 32 个文件（+470 −240） | 对得上 | 记录没写加减行 |
| Y6-70 | `records/2026-10-12-里程碑三工具与治理批.md:136`：「34 份」 | E2：批 7 乙补丁 | 34 个文件（+429 −53） | 对得上 | — |
| Y6-71 | `records/2026-10-12-里程碑三工具与治理批.md:136`：「13 份」 | E2：批 7 丙补丁 | 13 个文件（+17 −17） | 对得上 | — |
| Y6-72 | `records/2026-10-12-里程碑三工具与治理批.md:136`：「fuzz 3 才打得上」 | — | — | 复核不了 | 补丁对撞改动的上下文，要重打才看得到，补丁已进集成副本 |
| Y6-73 | `records/2026-10-12-里程碑三工具与治理批.md:137`：「派 184 项、收 184 项、判错 4 项」 | 同 Y6-43 | 184 / 184 / 4 | 对得上 | — |
| Y6-74 | `records/2026-10-12-里程碑三工具与治理批.md:137`：「第 30、33 行」 | A：`verify8-out/015-cells-true-tree.log` | 日志写「有 2 处不对」，第 30、33 行两条锚串在 runner-dispatch-guard.sh 里不命中 | 对得上 | — |
| Y6-75 | `records/2026-10-12-里程碑三工具与治理批.md:137`：「第 34–36 项」 | E1：`verify8.summary` | 34、35、36 三项期望红、退 0 | 对得上 | — |
| Y6-76 | `records/2026-10-12-里程碑三工具与治理批.md:137`：「第 156–158 项」 | E1：`verify8.summary` | 156、157、158 是同名 `brk-code-tooling-long-run-exempt-*`，退 1 | 对得上 | — |
| Y6-77 | `records/2026-10-12-里程碑三工具与治理批.md:137`：「其余 180 项对」 | 算术 184 − 4 | 180 | 对得上 | — |
| Y6-78 | `records/2026-10-12-里程碑三工具与治理批.md:138`：「批 7 收尾交回（`m3-tooling-b7-fixup/patch.diff` bfc7bbeb…719d，2 份）」 | E2：批 7 收尾补丁 | 2 个文件（+12 −11）；sha256 前后缀 bfc7bbeb…719d | 对得上 | — |
| Y6-79 | `records/2026-10-12-里程碑三工具与治理批.md:139`：「25 份二十三 same，两份」 | 同 Y6-22 | 25 = 23 + 2（不设 `B3_FROZEN`） | 对得上 | — |
| Y6-80 | `records/2026-10-12-里程碑三工具与治理批.md:141`：「45 份全 same」 | 同 Y6-23 | 45 | 对得上 | — |
| Y6-81 | `records/2026-10-12-里程碑三工具与治理批.md:141`：「85 处全对上」 | 同 Y6-21 | 85 / 85 | 对得上 | 同 Y6-21 的口径 |
| Y6-82 | `records/2026-10-12-里程碑三工具与治理批.md:143`：「标签 `after-b8`，86 份」 | E2：`git show --shortstat` 提交 80e5f38；三份批 8 补丁文件名并集 | 提交 86 个文件；三份补丁并集 85 个（再加甲给 `doc-process-records.sh` 的建议 diff 一个） | 对得上 | 记录没写 86 = 85 + 建议 diff 那一份 |
| Y6-83 | `records/2026-10-12-里程碑三工具与治理批.md:143`：「补 20 行」 | E2：各标签 `heavy-thread-exempt` 数据行 | after-b7-fixup 66 → after-b8 86 | 对得上 | — |
| Y6-84 | `records/2026-10-12-里程碑三工具与治理批.md:144`：「派 191 项、收 191 项、判错 5 项」 | 同 Y6-51 | 191 / 191 / 5 | 对得上 | — |
| Y6-85 | `records/2026-10-12-里程碑三工具与治理批.md:144`：「集成真树 heavy-thread-defaults 红 3 处」 | 同 Y6-51 | 3 处 | 对得上 | — |
| Y6-86 | `records/2026-10-12-里程碑三工具与治理批.md:144`：「第 33–36 项」 | E1：`verify10.summary` | 33–36 四项期望红、退 0（多一个 repin 开关） | 对得上 | — |
| Y6-87 | `records/2026-10-12-里程碑三工具与治理批.md:144`：「第 158–162 项」 | E1：`verify10.summary` | 158–162 五项同名门禁开关，退 1 | 对得上 | — |
| Y6-88 | `records/2026-10-12-里程碑三工具与治理批.md:144`：「其余 186 项对」 | 算术 191 − 5 | 186 | 对得上 | — |
| Y6-89 | `records/2026-10-12-里程碑三工具与治理批.md:145`：「批 8 收尾交回（`m3-tooling-b8-fixup/patch.diff` a08f25ca…325d，4 份）」 | E2：批 8 收尾补丁 | 4 个文件（+58 −4）；sha256 前后缀 a08f25ca…325d | 对得上 | — |
| Y6-90 | `records/2026-10-12-里程碑三工具与治理批.md:144`：「三行排除」 | E2 | after-b8 86 → after-b8-fixup 89 | 对得上 | — |
| Y6-91 | `records/2026-10-12-里程碑三工具与治理批.md:145`：「各红 7 格与 3 格」 | 同 Y6-53 | 7、3 | 对得上 | — |
| Y6-92 | `records/2026-10-12-里程碑三工具与治理批.md:145`：「三格在集成真树上绿（扫 832 份）」 | E5、E1：`verify11-out/cells.log`；第三轮冻结副本上格本体 | 三格绿；扫 832 份；可疑写法 117 处、排除表 89 行、装置自己读的线程变量 74 个 | 对得上 | 正文只引 832；117、89、74 与 Y6-56、Y6-57 引的 94、66、58（72）不是一个时点，哪个数属哪一版树要靠产物名 |
| Y6-93 | `records/2026-10-12-里程碑三工具与治理批.md:138`：「快照 33 份」 | 同 Y6-18（第二轮） | 33 | 对得上 | — |
| Y6-94 | `records/2026-10-12-里程碑三工具与治理批.md:146`：「快照 33 份」 | 同 Y6-18（第三轮） | 33 | 对得上 | — |
| Y6-95 | `records/2026-10-12-里程碑三工具与治理批.md:146`：「正文六格 Y1–Y6」 | A：数第三轮正文「一」表行数 | 6 行 Y1 至 Y6 | 对得上 | — |
| Y6-96 | `research/prompts/m3-defs-b6-r1-main-verification.md:14`：「攻方四样（两份冻结副本各一遍）」 | A：攻方腿 `gate-samples/要主agent跑.txt` 数条目；抽格 python 代跑 heavy-thread-defaults（V3 样本）与 private-values（pv-tar-mirror-red），库调用代跑 abandoned-rounds 与 long-run-header-exempt | 四样 = 四个门禁格；其中前两样的结局代跑对得上（Y6-07、Y6-08），后两样用库读数印证 | 门禁格本身复核不了（归主 agent）；代跑对得上 | 「两份冻结副本各一遍」= 零轮、零轮之前各一遍，共 8 次，代跑只做了其中会分辨的几次 |
| Y6-97 | `research/prompts/m3-defs-b6-r1-main-verification.md:14`：「正推三棵树」 | A：正推腿 `make_gate_trees.sh` 造的树，库默认读法与 `--legacy-ok` 读法、`exempt-check` | legacy-sourceless 默认不收、`--legacy-ok` 收；other-name-unrecognised 两种都收；exempt-table「有 2 处不对（共 3 行）」缺路径与已写齐，现算那一行放 | 对得上（库读数代跑） | 门禁格退出码复核不了；树里还有第四棵 symlink-source 判决没提 |
| Y6-98 | `research/prompts/m3-defs-b6-r1-main-verification.md:14`：「long-run-header-exempt、heavy-thread-defaults 两格与 `detached-run.sh long-run-check` 三条都退 0」 | A：攻方腿 `v8-rev12.txt`（零轮那一版拷贝里放修订 12 入口） | 库脚本读 rc=0、exempt-check rc=0、heavy-thread-defaults rc=0、long-run-check rc=0 | 对得上（代跑） | 输出里 4 行 rc=0 对应判决说的三条加一条库脚本读，数法没写 |

Y6 汇总（数法：用 python 数表行、按「判」栏分类，脚本在草稿目录）：98 行；对得上 89；对得上但不稳 2（Y6-06、Y6-67）；口径不清 1（Y6-25）；对不上 2（Y6-30、Y6-34）；整行复核不了 1（Y6-72）；有一半复核不了 3（Y6-07、Y6-58、Y6-96，都是门禁格本身或人工逐个看过那一半）。「口径」栏非空的 42 行里，影响读法的是下面这些，其余是补一句口径：

- Y6-30：第二轮判决写「排除表比第一轮多 2 行」，数据行两边都是 66（总行数 81 对 83，多的是 1 行注释 1 行空行）；对的说法是「键（路径:锚串）新 2 个、同时少 2 个」。
- Y6-34：引的「提交之外要跑先弹窗问用户」不在 `implementation-workflow.md` 第 53 行（那一行写的是「任务确实要跑时主 agent 先弹窗问用户」），是钩子拒绝说明里的话。
- Y6-06、Y6-67：「21 份全 same」在我的五次里三次成立（直接起），经 `tools/rerun.sh` 嵌套起的两次是 20 same 加 1 DIFF，差的是 `v7-path-probe.txt` 里数拷贝树文件的 14350 对 14352；它依赖工作目录里杂散文件，不是判定不稳。
- Y6-25：「第一轮只点名 3 处」的 3 处是 `tests.rs`、`baseline_read.py`、`if_default.sh`，不在九种之内；九种在第一轮是 0 份。
- Y6-22、Y6-79：「25 份」是不设 `B3_FROZEN` 的数，设了是 26 份（24 same + 2 DIFF）；模型目录 `out/` 存的是 26 份，`rerun.sh` 里没写这个默认值。
- Y6-56、Y6-57、Y6-92：94 / 66 / 58 是批 6 收尾那一刻第一轮副本的数；L2 退回宽判法后第二轮副本是 94 / 66 / 72，批 8 收尾后第三轮副本是 117 / 89 / 74。「72 → 58」那一对是窄判法下的数，引它的记录没标判法版本，后读者拿 58 当今天的数会错。
- Y6-65：「文字没写到 19」在正推报告里叫「缺」，里面有 V2 两句（判决判「站得住」）和「钉住无人核」一类没有机器核的句子，与「文字没写到」不是一个意思。
- Y6-53：「各红 7 格、3 格」的两份日志文件名带 dup、cb，没记是哪个开关名起的；开关名只能从自检行的名字推。
- Y6-03、Y6-18、Y6-38：「动过的 27 / 33 份」是快照清单的行数，也等于对 B3 做 `diff -rq` 去掉路径含 fixtures 的条目数（全部条目 46 / 62 / 68）；口径没写。
- Y6-10：「8 个长词」含本机主机名 1 个，配置本身 7 个；「长词」指 8 个字以上。
- Y6-14：「现有的四行」是无出处的 4 行，登记表共 5 行。
- Y6-72：「fuzz 3 才打得上」要重打补丁才看得到，复核不了。

Y6 的结论与推翻条件：数字大体能核（98 行里 92 行对得上或对得上带口径），不影响判决方向的对不上只有 Y6-30、Y6-34 两处。推翻它的现象：用同一条命令在主工作区重算，任何一行的结果与「现算结果」栏不同；或 `tools/rerun.sh` 在干净工作目录复跑，`out/` 里有一份 `differs`（除 Y6-06 说的文件数那一行和 Y6-05 说的按字节截断那一行）。

## 2 Y5：四条腿报告里每一条打中的去向

范围与记法：
- 「打中」按各腿自己的口径取：第一轮正推腿 = 报告第 1 至 5 节表里判「不一致」「缺」的 31 个编号（不一致 12、缺 19，见 Y6-65）；第一轮攻方腿 = 「各格判定一览」里的打中行加正文专问里的发现；第二轮辩方腿 = 没修到、部分修到、没动、改过头、判重判轻与专问里的发现；第二轮攻方腿 = 「各格判定一览」里判「打中」的行加专问发现。攻方自己标「没打中」的形状不算（`O1-V5-C6` 是例外：攻方写「不另算」，辩方复现了，所以列出）。
- 「去向」= 判决里的改法（L1–L7 在 `research/prompts/m3-defs-b6-r1-main-verification.md` 第 33–39 行，M1–M7 在 `research/prompts/m3-defs-b6-r2-main-verification.md` 第 33–39 行）、「另立一笔账」（两份判决各自第 41 行）、判「站得住」不改（第一轮判决第 20–27 行逐格）、主 agent 核后不算。去向栏写「无去向」的，是两份判决的逐格表、改法表、另立账和核查记录里都找不到这一条（我对着每份判决逐行搜过）。
- 一行一个编号。第一轮正推腿同一个编号在报告里出现两次的并成一行。编号前缀：S1 = 第一轮正推腿，O1 = 第一轮攻方腿，D2 = 第二轮辩方腿，A2 = 第二轮攻方腿；`-b` 结尾的是我从同一编号里拆出来的第二个发现。
- 数：99 行（S1 33、O1 19、D2 28、A2 19）；去向有写的 77 行，两轮判决都没写的 22 行（其中 5 行我标「弱」或「元判断」）。

| 编号 | 出处（报告:行） | 打中的是什么 | 去向 | 去向出处（判决:行） | 备注 |
|---|---|---|---|---|---|
| S1-AC-2 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:25`：「判在字符上不在词上」 | ⑮ 括注数词按字符判（误拒统一、万一、十分，漏放按核数取） | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:20`：「括注里中文数词按字符判」 |  |
| S1-MA-1 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:32`：「括注不套 ⑭ 的现算词表」 | 同 AC-2：括注里的说明性的词判不了内容 | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:33`：「⑮ 不再认括注」 |  |
| S1-AC-3 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:26`：「括注里面不判」 | 括注里的尖括号放行，与「别的尖括号按认不出数拒」冲突 | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:20`：「括注里的尖括号」 |  |
| S1-MA-3 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:34`：「同 AC-3」 | 同 AC-3（main-agent 一侧的文字） | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:33`：「⑮ 不再认括注」 |  |
| S1-HG-6 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:67`：「读不到登记时放行」 | 钩子注释里的尖括号一句，括注里放行 | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:33`：「⑮ 不再认括注」 |  |
| S1-CL-2 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:48`：「三分句都与钩子一致」 | 已判行顶缺行，clerk 定义没写这条路 | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:33`：「也顶缺行」 |  |
| S1-FW-2 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:50`：「三分句都与钩子一致」 | 同 CL-2（three-way-forward） | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:33`：「也顶缺行」 |  |
| S1-DF-2 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:52`：「三分句都与钩子一致」 | 同 CL-2（three-way-defense） | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:33`：「也顶缺行」 |  |
| S1-IN-1 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:53`：「同 CL-2（investigator 原有的预计挂钟行不变」 | 同 CL-2（investigator） | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:33`：「也顶缺行」 |  |
| S1-TW-1 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:55`：「同 IN-1」 | 同 CL-2（tooling-writer） | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:33`：「也顶缺行」 |  |
| S1-HG-4 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:65`：「括注里的事实与定义不符」 | 钩子注释说 forward、defense 定义许编译，定义里没有 | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:20`：「钩子注释说 three-way-forward 定义许编译，定义里没有」 |  |
| S1-DP-1 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:69`：「在同一钩子里指 kb-scribe」 | dispatch-prompt.py 把 clerk 叫「书记员」 | 第一轮判决没列；第二轮改法 M1 | `research/prompts/m3-defs-b6-r2-main-verification.md:21`：「第 20 行仍把 clerk 叫」 | 第一轮漏列，第二轮补上；第三轮副本第 20 行已是 clerk |
| S1-AC-5 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:28`：「归第 4 节（V6）」 | 出处那一行不许有别的轮名，库只认 -r 形与登记名 | 改法 L4（M4 改过头后再改） | `research/prompts/m3-defs-b6-r1-main-verification.md:36`：「按仓里认得的轮名全集判」 |  |
| S1-TI-1 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:144`：「m3-close-b1」 | m3-close-b1、OTHER-R2 照收 | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「别的轮名只认 -r<数字> 形与登记过的名字」 |  |
| S1-AC-6 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:29`：「归第 4 节（V6）」 | 日期、重复、出处共用一份判法的说法缺一半 | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「判法按它免出处」 |  |
| S1-TI-3 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:146`：「门禁传 legacy_sourceless_ok=True」 | 门禁收、⑥ 不收 | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「判法按它免出处」 |  |
| S1-HG6-1 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:155`：「对这些行不成立」 | 「不收就是红」对早于 2026-10-12 的行不成立 | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「判法按它免出处」 |  |
| S1-HG6-5 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:159`：「拒绝说明里的这一句同 HG6-1」 | 同 HG6-1（拒绝说明） | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「判法按它免出处」 |  |
| S1-LIB-1 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:160`：「自己写了例外」 | 同 HG6-1（库文档串） | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「判法按它免出处」 |  |
| S1-DPR-1 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:162`：「同 HG6-1」 | 同 HG6-1（规则文字） | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「判法按它免出处」 |  |
| S1-DPR-2 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:163`：「缺的是前提」 | 登记表头的日期是腿发出日，不是登记日 | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「判法按它免出处」 |  |
| S1-DPR-5 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:166`：「起登记的行都要出处」 | 同 DPR-2（门禁红的提示） | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「判法按它免出处」 |  |
| S1-AC-7 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:30`：「归第 2 节（V2）」 | 让出 CPU 句：已判行理由不看内容，靠主 agent | 判「站得住」不改；第二轮辩方判轻，判决没回应 | `research/prompts/m3-defs-b6-r1-main-verification.md:21`：「两句都是给主 agent 的规矩，钩子判不了，正推逐句一致」 | 第三轮 Y4 重判 V2；见 Y4 一节 |
| S1-MA-6 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:37`：「归第 2 节（V2）」 | 同 AC-7（main-agent 一侧） | 同 AC-7 | `research/prompts/m3-defs-b6-r1-main-verification.md:21`：「两句都是给主 agent 的规矩，钩子判不了，正推逐句一致」 |  |
| S1-IN-2 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:54`：「没有任何钩子判」 | 「内存上限」没有钩子判 | **无去向**（两轮都没写） | — | 第二轮辩方列「没动」；第三轮副本 investigator.md:26 与 tooling-writer.md:26 仍有这句 |
| S1-TW-2 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:56`：「同 IN-2」 | 同 IN-2（tooling-writer） | **无去向**（同 IN-2） | — |  |
| S1-MA-9 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:198`：「本轮没改这一句，词表落在它上面」 | main-agent.md 第 55 行没写词表与否定句也算 | 改法 L6 | `research/prompts/m3-defs-b6-r1-main-verification.md:27`：「main-agent.md 第 55 行没写词表与」 |  |
| S1-DR-2 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:204`：「没有机器核：exempt_table_problems 不读 sha256」 | 「被 sha256 钉住」无人核，现行文件与找得到的钉都不同 | 改法 L7 | `research/prompts/m3-defs-b6-r1-main-verification.md:27`：「无人核、现行文件与找得到的钉都不同」 |  |
| S1-LR-4 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:211`：「同 DR-2：表的写法」 | 同 DR-2（lib_long_run.py 文档串） | 改法 L7 | `research/prompts/m3-defs-b6-r1-main-verification.md:27`：「无人核、现行文件与找得到的钉都不同」 |  |
| S1-EX-3 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:214`：「真按此刻空闲现算」 | 豁免表文件头那句，「被 sha256 钉住」同 DR-2 | 改法 L7（钉的一半） | `research/prompts/m3-defs-b6-r1-main-verification.md:27`：「无人核、现行文件与找得到的钉都不同」 | 提示一侧无豁免、两边读法不一致那一半见 S1-EX-3b |
| S1-EX-3b | `research/prompts/m3-defs-b6-r1-sonnet-output.md:224`：「派发提示一侧没有豁免」 | 六份入口文件头按现算读，派发提示必须写登记值读法，两边读法不一 | **无去向**（两轮都没写） | — | 第二轮辩方列「没动」 |
| S1-SO-1 | `research/prompts/m3-defs-b6-r1-sonnet-output.md:217`：「钉住就不能改脚本，只剩删行」 | stage-owners.tsv 说「红了删行或改脚本」，钉住就不能改脚本 | 改法 L7（钉的一半） | `research/prompts/m3-defs-b6-r1-main-verification.md:27`：「无人核、现行文件与找得到的钉都不同」 | 文字那一半见 S1-SO-1b |
| S1-SO-1b | `research/prompts/m3-defs-b6-r1-sonnet-output.md:217`：「同 DR-2」 | stage-owners.tsv 第 48 行「改脚本归实验那一边」与 L7 的钉冲突 | **无去向**（两轮都没写） | — | 第三轮副本 stage-owners.tsv:48 同句仍在 |
| O1-V1-甲 | `research/prompts/m3-defs-b6-r1-opus-output.md:97`：「⑮ 括注写定义自己的原话」 | 括注抄定义原话「全仓唯一的登记位」被拒（误伤） | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:20`：「括注里中文数词按字符判」 |  |
| O1-V1-乙 | `research/prompts/m3-defs-b6-r1-opus-output.md:98`：「（忙闲看空闲核数再定）」 | 括注写按负载降、对折、双倍、第２台１６等放行（7 格） | 改法 L1 | `research/prompts/m3-defs-b6-r1-main-verification.md:33`：「⑮ 不再认括注」 |  |
| O1-V1-专问 | `research/prompts/m3-defs-b6-r1-opus-output.md:151`：「three-way-forward、three-way-defense 的多数轮只读源码、喂钩子」 | 三种新类型多数派发不带重活却被逼写线程上限 | 另立账（出路是已判行，L1 写清） | `research/prompts/m3-defs-b6-r1-main-verification.md:41`：「三种新类型（clerk、three-way-forward、three-way-defense）多数派发不带重活却要写线程上限，出路是已判行（L1 写清）」 |  |
| O1-V3 | `research/prompts/m3-defs-b6-r1-opus-output.md:99`：「九种写法零轮那一版一处都不点名」 | heavy-thread-defaults 经包装函数、别名、拷贝、vars()、use as、｜｜ X=8、&& X=8 读线程变量不点名 | 改法 L2 | `research/prompts/m3-defs-b6-r1-main-verification.md:22`：「九种自然写法」 |  |
| O1-V3-乙 | `research/prompts/m3-defs-b6-r1-opus-output.md:100`：「接 mod tests;（只是声明）之后整份不扫」 | `#[cfg(test)]` 接外置 `mod tests;` 之后的读两版都漏 | 改法 L2 | `research/prompts/m3-defs-b6-r1-main-verification.md:22`：「之后的读两版都漏」 |  |
| O1-V3-printenv | `research/prompts/m3-defs-b6-r1-opus-output.md:179`：「$(printenv SAMPLE_PRINTENV_WORKERS)」 | printenv 读线程变量两版都漏 | 第一轮判决没列（调度记录第 136 行记账）；第二轮改法 M2 | `research/prompts/m3-defs-b6-r2-main-verification.md:34`：「printenv NAME 认成读」 | 第一轮判决漏列；批 7 记录里只说「记账」 |
| O1-V3-tests.rs | `research/prompts/m3-defs-b6-r1-opus-output.md:157`：「声明的测试模块文件，两版都点名，算误报」 | tests.rs（外置测试模块文件）两版都点名，攻方自己算误报 | **无去向**（弱：两版同、攻方自标误报） | — |  |
| O1-V5-甲 | `research/prompts/m3-defs-b6-r1-opus-output.md:101`：「副本里只改入口脚本 import 的库」 | 仓副本里只改入口 import 的库，副本自证照放 | 另立账（V5 甲） | `research/prompts/m3-defs-b6-r1-main-verification.md:41`：「V5 甲副本里 import 的库被改闸看不到」 |  |
| O1-V5-乙 | `research/prompts/m3-defs-b6-r1-opus-output.md:102`：「主 agent 在副本里跑改过的脚本的自证也被拒」 | 拒绝说明让主 agent 在集成副本里跑，主 agent 不带前缀也被拒 | 改法 L3（说明写全起法）；M3 再改 | `research/prompts/m3-defs-b6-r1-main-verification.md:24`：「主 agent 不带前缀照做也被拒」 |  |
| O1-V5-丙C2 | `research/prompts/m3-defs-b6-r1-opus-output.md:103`：「env -iC <checker 档目录> cargo test」 | env 合写短选项 -iC 放行 | 改法 L3 | `research/prompts/m3-defs-b6-r1-main-verification.md:24`：「env -iC」 |  |
| O1-V5-丙C8 | `research/prompts/m3-defs-b6-r1-opus-output.md:103`：「env -S 'cargo test -p singlefs-checker-tier'」 | env -S 串放行 | 改法 L3 | `research/prompts/m3-defs-b6-r1-main-verification.md:24`：「env -S」 |  |
| O1-V5-C6 | `research/prompts/m3-defs-b6-r1-opus-output.md:359`：「目录写在变量里的（C6）钩子文件头已写罩不住，不另算」 | env -C 的目录写在变量里放行 | **无去向**（钩子文件头已写罩不住，两轮判决都没提） | — | 第二轮辩方复现 A=B=C=0；见 D2-C6 |
| O1-V6 | `research/prompts/m3-defs-b6-r1-opus-output.md:104`：「10-12 之前发腿、之后叫停的轮，门禁格不要出处判绿」 | 表头日期是腿发出日，判法按它免出处，门禁收 ⑥ 不收 | 改法 L4 | `research/prompts/m3-defs-b6-r1-main-verification.md:25`：「判法按它免出处」 |  |
| O1-V6-③ | `research/prompts/m3-defs-b6-r1-opus-output.md:210`：「③ 日期格式对而值不像日子的」 | 0000-00-00 过形状、早于切点，门禁免出处 | 改法 L4（名单替日期，名单外一律要出处） | `research/prompts/m3-defs-b6-r1-main-verification.md:36`：「改成点名：登记表里现有的四行」 |  |
| O1-V7 | `research/prompts/m3-defs-b6-r1-opus-output.md:105`：「路径命中时逐段打码打不到」 | 私有值带 / 时逐段打码打不到，红行把值原样打出 | 改法 L5 | `research/prompts/m3-defs-b6-r1-main-verification.md:26`：「私有值本身带 /」 |  |
| O1-V7-专问 | `research/prompts/m3-defs-b6-r1-opus-output.md:234`：「到 1 GiB 上限的单行约 2 GB」 | 未跟踪文件整份读：单行大文件常驻内存随最长一行涨（推的） | **无去向**（两轮都没写） | — | 第三轮副本 doc-text.sh 上限仍是 1 GiB（UNTRACKED_SIZE_LIMIT） |
| O1-V8-甲 | `research/prompts/m3-defs-b6-r1-opus-output.md:285`：「E171 修订 12：改头不改身三道全过，老实拷的被拒」 | 只改文件头一行、正文仍起现算编排，三道全过 | 改法 L7（文字：钩子只判写法）；第二轮另立账 V8 甲 | `research/prompts/m3-defs-b6-r1-main-verification.md:27`：「文件头写的读法与脚本做的事是否一致，静态判法看不到」 |  |
| O1-V8-乙M1-M3 | `research/prompts/m3-defs-b6-r1-opus-output.md:336`：「M1–M3 分（b3 放、b6 拒），M6、M7 不分（两版都放）」 | ⑭ 现算词表拒定义原话，放「一半」「余量」 | 改法 L6（词表删，改规范句） | `research/prompts/m3-defs-b6-r1-main-verification.md:38`：「只认骨架那一句规范写法」 |  |
| O1-两张词表 | `research/prompts/m3-defs-b6-r1-opus-output.md:112`：「⑭ 与 ⑮ 两张词表不一」 | ⑭ ⑮ 词表各管一半，互相不拒对方拒的 | 改法 L1 加 L6 | `research/prompts/m3-defs-b6-r1-main-verification.md:38`：「只认骨架那一句规范写法」 |  |
| D2-X1-通道 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:46`：「通道只是挪了位置」 | 括注改拒后，同一句挪进「线程上限说明：」一行照放 | 改法 M1 | `research/prompts/m3-defs-b6-r2-main-verification.md:33`：「提示里别处写的线程数、倍数、折算一概不照」 |  |
| D2-X1-DP1 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:42`：「L1 改了这份文档串里别的句子，这一处没动」 | dispatch-prompt.py 第 20 行仍叫「书记员」 | 改法 M1 | `research/prompts/m3-defs-b6-r2-main-verification.md:21`：「第 20 行仍把 clerk 叫」 |  |
| D2-X1-内存上限 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:43`：「不在 L1–L7；第一轮判决也没列」 | 「内存上限」无钩子判（没动） | **无去向**（同 S1-IN-2） | — |  |
| D2-V2-判轻 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:44`：「承重变大（见第 6 节判轻）」 | 已判行理由钩子不看内容，三种新类型靠已判行顶缺行，承重变大 | **无去向**（第二轮判决没有 V2 一格） | — | 第三轮 Y4 重判 V2；规则句无机器判，见 Y4 |
| D2-X2-printenv | `research/prompts/m3-defs-b6-r2-sonnet-output.md:85`：「L2 没承诺这一形」 | printenv 读法两版都漏 | 改法 M2 | `research/prompts/m3-defs-b6-r2-main-verification.md:34`：「printenv NAME 认成读」 |  |
| D2-X2-回来2处 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:90`：「真树上回来 2 处」 | 宽判法在真树上带回 dispatch-prompt.py:269 与 replay.sh:1892 两处误判 | 改法 M2（误判逐行判、进排除表） | `research/prompts/m3-defs-b6-r2-main-verification.md:22`：「8 种不读环境的写法回来成误判」 |  |
| D2-X2-O3 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:96`：「判：改过头（O-3）」 | 排除表涨 2 行与格的只缩不涨冲突 | 主 agent 核：主工作区 HEAD 没有这张表，提交时不比，不算 | `research/prompts/m3-defs-b6-r2-main-verification.md:15`：「不会红；不算改过头」 | 口径另见 Y6-30：行数没涨，涨的是 2 个键 |
| D2-X3-V5甲 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:108`：「定义对子 agent 说不要起，钩子仍不拦，两句并存」 | V5 甲：副本里改了入口 import 的库，钩子仍放；共用约束已写成规则 | 另立账（V5 甲） | `research/prompts/m3-defs-b6-r1-main-verification.md:41`：「V5 甲副本里 import 的库被改闸看不到」 |  |
| D2-X3-C6 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:114`：「仍罩不住的是第一轮就写在钩子文件头的那一类」 | env -C 的目录写在变量里 A=B=C=0 | **无去向**（钩子文件头已写；判决没提） | — | 同 O1-V5-C6 |
| D2-X4-部分 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:23`：「别的轮名：仓里有材料的名字修到」 | 别的轮名认全集，材料全集怎么取 | 改法 M4 | `research/prompts/m3-defs-b6-r2-main-verification.md:36`：「两样都拒」 |  |
| D2-X4-O1 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:135`：「文字与代码二选一改，不进本腿」 | O-1：`<名>-r<数字>` 形而仓里没材料的名字，改前不收，改后收 | 改法 M4 | `research/prompts/m3-defs-b6-r2-main-verification.md:24`：「O-1，改过头」 |  |
| D2-X4-文字 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:132`：「文字与代码不一致」 | 规则文字写「不只认 -r 形」，代码不再认 -r 形 | 改法 M4（规则文字照这个写） | `research/prompts/m3-defs-b6-r2-main-verification.md:36`：「两样都拒」 |  |
| D2-X4-漏取 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:133`：「漏取 main-verification 这一类，310 个里 120 个认不出」 | 全集漏取一类会丢 120/310 个名字 | **无去向**（弱：M4 并上 -r 形后多数仍被收） | — |  |
| D2-X4-腿报告 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:125`：「只有腿报告」 | 只有腿报告文件的轮不在全集里 | **无去向**（弱） | — |  |
| D2-X4-名单 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:141`：「名单写死在库里，没有检查把它与表绑在一起」 | 旧行名单与登记表没有检查绑定 | **无去向**（弱；与 V4 名单同类） | — |  |
| D2-X5-fail | `research/prompts/m3-defs-b6-r2-sonnet-output.md:156`：「有一处没经」 | doc-text.sh「取不到 path」分支把原路径打出（git 失败时） | **无去向**（M5 只写各格打印路径经共用打码，没点名这一支） | — | 第三轮副本 doc-text.sh:541 仍原样；同 A2-X5-专问 |
| D2-X6-部分 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:63`：「说明里只拦数字」 | 规范句后的说明里仍放一半、余量、中文数词 | 改法 M6 | `research/prompts/m3-defs-b6-r2-main-verification.md:38`：「规范句后面不许接任何说明」 |  |
| D2-X6-O2 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:71`：「改过头候选 O-2：规范句后面接现算词」 | O-2：规范句后接现算词，改前拒改后放 | 改法 M6 | `research/prompts/m3-defs-b6-r2-main-verification.md:26`：「O-2，改过头」 |  |
| D2-X7-sha | `research/prompts/m3-defs-b6-r2-sonnet-output.md:167`：「表里每行带 sha256 并核」 | sha256 入表并核：行尾、改名、全 0、大写一律红 | 改法 M7（CRLF 判红记为不改的多拒） | `research/prompts/m3-defs-b6-r2-main-verification.md:27`：「表是自证（攻方 X7 甲、辩方）」 |  |
| D2-X7-自证 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:168`：「表与脚本自洽，不是外部的钉」 | 同一次改动里改脚本并重钉 sha256，格不红 | 改法 M7；V8 甲另立账 | `research/prompts/m3-defs-b6-r2-main-verification.md:27`：「V8 甲，判别子观测不到，定义已写由写的人保证」 |  |
| D2-X7-c份 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:166`：「与 B 的结构一模一样」 | 规范句头 + 现算正文的入口三道全过（V8 甲没修到） | 另立账（V8 甲） | `research/prompts/m3-defs-b6-r2-main-verification.md:27`：「V8 甲，判别子观测不到，定义已写由写的人保证」 |  |
| D2-X7-stage-owners | `research/prompts/m3-defs-b6-r2-sonnet-output.md:169`：「那一半与 L7 冲突」 | stage-owners.tsv 第 48 行写「改脚本归实验那一边」，与 L7 冲突 | **无去向**（同 S1-SO-1b） | — |  |
| D2-X7-提示侧 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:170`：「提示一侧仍无豁免；文件头一侧有表」 | E171 六份入口：提示必须写规范句，文件头按现算读，两边读法不一 | **无去向**（同 S1-EX-3b） | — |  |
| D2-X7-编排 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:187`：「入口之外的编排（segment_one_units_revision_11.py、rows_orchestrator.py）不在 sha256 里」 | 改入口 exec 的编排不触发任何红 | 近似：另立账 V8 甲（同一个根：钩子看不到正文行为）；判决没单写编排不在钉里 | `research/prompts/m3-defs-b6-r2-main-verification.md:27`：「V8 甲，判别子观测不到，定义已写由写的人保证」 |  |
| D2-判重-V1 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:213`：「后者才是承重的」 | 第一轮把「误伤」与「违反固定不看忙闲的漏放」合进同一个「站不住」 | **无去向**（元判断，不是开着的线） | — |  |
| D2-判轻-V3 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:215`：「方向判对；L2 的代价判轻」 | L2 退回宽判法的代价（真树 2 处、表涨 2 行）判决写轻了 | 改法 M2 | `research/prompts/m3-defs-b6-r2-main-verification.md:22`：「8 种不读环境的写法回来成误判」 |  |
| D2-判重-V6 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:218`：「m3-close-b1 是编的名字」 | 别的轮名那一根判重：真仓 310 个名字只有 1 个非 -r 形 | 改法 M4（并上 -r 形） | `research/prompts/m3-defs-b6-r2-main-verification.md:36`：「两样都拒」 |  |
| D2-判轻-V8 | `research/prompts/m3-defs-b6-r2-sonnet-output.md:220`：「V8-乙判对；V8-甲的改法判轻」 | L7 并没有修 V8 甲，只是逼文件头写规范句 | 另立账（V8 甲） | `research/prompts/m3-defs-b6-r2-main-verification.md:27`：「V8 甲，判别子观测不到，定义已写由写的人保证」 |  |
| A2-X1-甲 | `research/prompts/m3-defs-b6-r2-opus-output.md:96`：「挪到新开的」 | 第一轮 V1-乙 的话挪进「线程上限说明：」一行照放，子 agent 仍看得到 | 改法 M1 | `research/prompts/m3-defs-b6-r2-main-verification.md:33`：「提示里别处写的线程数、倍数、折算一概不照」 |  |
| A2-X1-乙 | `research/prompts/m3-defs-b6-r2-opus-output.md:97`：「提示里代码块的喂法示例行」 | 代码块里的示例行「线程上限：31（本机）」也被当成那一行拒 | 另立账 X1 乙，并入 M1 | `research/prompts/m3-defs-b6-r2-main-verification.md:41`：「X1 乙（代码块示例行，M1 一并改）」 |  |
| A2-X1-专问 | `research/prompts/m3-defs-b6-r2-opus-output.md:155`：「子 agent 一侧：共用约束只说 N 照」 | 共用约束没说说明行里的数子 agent 不照（推的） | 改法 M1 | `research/prompts/m3-defs-b6-r2-main-verification.md:33`：「提示里别处写的线程数、倍数、折算一概不照」 |  |
| A2-X2-甲 | `research/prompts/m3-defs-b6-r2-opus-output.md:98`：「L2 只认右边是字面数字」 | `｜｜ X=$变量`、`｜｜ X=$((…))` 不点名 | 改法 M2 | `research/prompts/m3-defs-b6-r2-main-verification.md:34`：「认成钉数」 |  |
| A2-X2-乙 | `research/prompts/m3-defs-b6-r2-opus-output.md:99`：「已豁免的 45 份文件里照吞」 | 新读法写进已被「代码批」行放行的文件被吞 | 改法 M2（排除表按被判行匹配） | `research/prompts/m3-defs-b6-r2-main-verification.md:34`：「排除表按被判行匹配」 |  |
| A2-X2-丙a | `research/prompts/m3-defs-b6-r2-opus-output.md:207`：「回退的认法按子串认」 | 变量名里带 heavy_threads 就算「回退到登记值」（heavy_named.py） | **无去向**（两轮都没写；零轮之前那一版也漏，按跑前条款本该另立账） | — | 第三轮副本仍不点名（Y6 现算 htd-samples） |
| A2-X2-丙b | `research/prompts/m3-defs-b6-r2-opus-output.md:100`：「f 串拼的变量名」 | f 串拼的变量名认不出（fstring_name.py） | **无去向**（同上；第二轮记录第 14 行只提到「f 串名等」两版都不点名） | — |  |
| A2-X2-丙c | `research/prompts/m3-defs-b6-r2-opus-output.md:209`：「两处口径不一」 | `_LANES`、`_PARALLELISM` 不在线程变量名里（lanes_read.py） | **无去向**（同上） | — |  |
| A2-X2-丙d | `research/prompts/m3-defs-b6-r2-opus-output.md:210`：「整份后面都不扫」 | 文件中段 `#[cfg(test)] mod tests { }` 之后整份不扫（mid_tests.rs） | **无去向**（同上；L2 只修了外置 `mod x;`） | — |  |
| A2-X3-甲 | `research/prompts/m3-defs-b6-r2-opus-output.md:101`：「库照原文留着，放行」 | `env -S` 串里的 ${VAR} 不展开，放行 | 另立账（X3 甲） | `research/prompts/m3-defs-b6-r2-main-verification.md:41`：「改前也中，不拿来否这批改动」 | 第二轮判决另立账一句里只写「X3 甲（env -S 里 ${…} 展开）」 |
| A2-X3-乙 | `research/prompts/m3-defs-b6-r2-opus-output.md:102`：「-p 的值写在变量里，放行」 | -p 的值写在变量里，放行 | 另立账（X3 乙） | `research/prompts/m3-defs-b6-r2-main-verification.md:41`：「X3 乙（-p 的值在变量里）」 |  |
| A2-X3-丙 | `research/prompts/m3-defs-b6-r2-opus-output.md:103`：「让主 agent 直接带 user-request 前缀跑改过的自证」 | 拒绝说明让主 agent 直接带前缀跑，与「先弹窗问用户」相抵 | 改法 M3 | `research/prompts/m3-defs-b6-r2-main-verification.md:35`：「指到 .claude/rules/implementation-workflow.md 重型测试那一条的做法」 |  |
| A2-X5-甲 | `research/prompts/m3-defs-b6-r2-opus-output.md:104`：「同一道 doc-text 里 prime-marks、session-names 两格绿着也把二进制文件的路径原样打出」 | 兄弟格原样打路径，值上了输出 | 改法 M5 | `research/prompts/m3-defs-b6-r2-main-verification.md:25`：「X5 甲并进改法 M5」 |  |
| A2-X5-专问 | `research/prompts/m3-defs-b6-r2-opus-output.md:299`：「不经 shown_path（推的，没造：要 git diff 失败或文件读不了）」 | private-values 自己的 fail(取不到 path) 与读文件异常回溯带原路径 | **无去向**（同 D2-X5-fail） | — |  |
| A2-X6-甲 | `research/prompts/m3-defs-b6-r2-opus-output.md:106`：「派发闸 ⑭ 与 detached-run.sh long-run-check 都放」 | 规范句后接现算说明，⑭ 与 long-run-check 都放 | 改法 M6 | `research/prompts/m3-defs-b6-r2-main-verification.md:38`：「规范句后面不许接任何说明」 |  |
| A2-X6-乙 | `research/prompts/m3-defs-b6-r2-opus-output.md:107`：「说明里写 E171 修订 11、layer0、sha256、半角分号、文件头规范句折成两行，都拒」 | 说明里的正当词被「不含数字」或逐字挡掉（多拒） | 改法 M6（说明另起一行） | `research/prompts/m3-defs-b6-r2-main-verification.md:38`：「规范句后面不许接任何说明」 |  |
| A2-X7-甲 | `research/prompts/m3-defs-b6-r2-opus-output.md:108`：「改了豁免表里脚本的正文、把表里 sha256 换成新的」 | 重钉 sha256 不算涨 | 改法 M7 | `research/prompts/m3-defs-b6-r2-main-verification.md:39`：「同一路径的 sha256 变了也算涨」 |  |
| A2-X7-编排 | `research/prompts/m3-defs-b6-r2-opus-output.md:361`：「现算真正发生在入口 exec 的 rows_orchestrator.py 一类编排里」 | 豁免只钉入口，改入口 exec 的编排不触发红（推的） | 近似：另立账 V8 甲（同根） | `research/prompts/m3-defs-b6-r2-main-verification.md:27`：「V8 甲，判别子观测不到，定义已写由写的人保证」 |  |
| A2-X7-乙 | `research/prompts/m3-defs-b6-r2-opus-output.md:109`：「CRLF 行尾：sha256 变、判红（多拒）」 | CRLF 整份换行尾判红，是多拒 | 判决记「CRLF 判红」为已知状态，不改 | `research/prompts/m3-defs-b6-r2-main-verification.md:27`：「CRLF 判红」 |  |

### 2.1 两轮判决都没写去向的（22 行并成 11 条）

每条给出处（表里的编号）、打中的历史、第三轮冻结副本今天的样子。按正文「四、跑前条款」，Y5 漏列的处理是「补进判决的另立账或交用户表」，由主 agent 定，我不替它定。

1. 内存上限没有钩子判（S1-IN-2、S1-TW-2、D2-X1-内存上限）。历史：派 investigator 或 tooling-writer，定义「输入」要「内存上限」，提示里写不写、写几都过派发闸。R3 今天：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/agents/investigator.md:26`：「内存上限；报告路径与草稿目录」，tooling-writer.md 同一行号同一句。
2. 已判行的理由钩子不看内容，三种新类型靠它顶缺行（D2-V2-判轻；S1-AC-7、S1-MA-6 的去向是第一轮判「站得住」，但第二轮辩方提的「承重变大」没人回应）。历史：主 agent 派 clerk 时写「线程上限：8」加「线程上限已判：机器忙，负载高，让出 CPU」，钩子放行（本报告 Y4 的 V2-1，`y4-feed.txt` a3），而 V2 那句要求「只看到负载忙不算」。
3. E171 六份入口两边读法不一（S1-EX-3b、D2-X7-提示侧）。历史：派执行员跑 E171 修订 6 至 11 的某份入口，派发提示的多机一行只能写规范句（台数从 `--emit-hosts` 读、每台线程取登记值），而入口文件头按现算读、入口 exec 的编排真按 `available_threads` 现算；提示与入口各自过各自的判法，做的事和写的不一样。第一轮判决 V8 行只写了「钉」那一半。
4. `stage-owners.tsv` 第 48 行（S1-SO-1b、D2-X7-stage-owners）。历史：long-run-header-exempt 红了，照 owners 表交实验那一边「删行或改脚本」，改脚本破 sha256（L7、M7 让改脚本必须重钉并交用户）。R3 今天：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/stage-owners.tsv:48`：「表里的脚本被 sha256 钉住，红了删行或改脚本归实验那一边」，一字没动。
5. heavy-thread-defaults 四种从未被看过的漏（A2-X2-丙a 至 丙d）：变量名里带 `heavy_threads` 就算回退到登记值（`heavy_named.py`）、f 串拼变量名（`fstring_name.py`）、`_LANES` 与 `_PARALLELISM` 不在线程变量名里（`lanes_read.py`）、文件中段的 `#[cfg(test)] mod tests { }` 之后整份不扫（`mid_tests.rs`）。改前也中（零轮之前那一版同样漏，跑前条款本该另立一笔账）。第二轮核查记录第 14 行提到「f 串名等」两版都不点名，没写去向。R3 今天：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/code-tooling.sh:2039`：「CAP_OR_REGISTRY = re.compile(r"SINGLEFS_THREAD_CAP|heavy_thread_default|default_heavy_threads|heavy-threads\.sh|heavy_threads|capped\.sh")」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/code-tooling.sh:1992`：「THREAD_VARIABLE = r"[A-Z0-9_]*(?:THREADS?|THREAD_CAP|JOBS|WORKERS)[A-Z0-9_]*"」；`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/code-tooling.sh:2286`：「code.startswith("#[cfg(test)]")」。现算：E8 里第二轮攻方 X2 样本在 R3 的判法下四份都不点名。
6. private-values 自己的「取不到」分支与异常回溯带原路径（D2-X5-fail、A2-X5-专问）。历史：git diff 退非 0（基准提交不在、git 坏了）或未跟踪文件读不了时，红行把含私有值的原路径打出（推的，没造过）。R3 今天：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/doc-text.sh:541`：「fail(f"取不到 {path} 相对基准 {base[:12]} 的改动（git diff 退 {difference.returncode}）"」，第二轮 M5 只写各格打印路径经共用打码，没点名这一支。
7. 未跟踪大文件整份读的内存（O1-V7-专问）。历史：执行员把第二台的大日志拉回本机，单行 300 MiB 的文件常驻 636 MB 左右，攻方外推到 1 GiB 单行约 2 GB（推的）。R3 今天：`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/doc-text.sh:503`：「UNTRACKED_SIZE_LIMIT = 1024 * 1024 * 1024」。
8. 外置测试模块文件两版都点名（O1-V3-tests.rs，弱）：攻方自己标误报，两版同，不违反用户定案。
9. `env -C` 的目录写在变量里放行（O1-V5-C6、D2-X3-C6）：钩子文件头自己写了罩不住，攻方不另算，辩方复现三份副本都放；判决没列。
10. 轮名全集的三个小漏（D2-X4-漏取、D2-X4-腿报告、D2-X4-名单，弱）：全集漏取 main-verification 一类则 310 个里 120 个认不出；只有腿报告的轮不认；旧行名单与登记表没有检查绑定。M4 把全集并上 `<名>-r<数字>` 形之后多数仍被收。
11. D2-判重-V1：辩方说第一轮把「误伤」和「违反固定不看忙闲的漏放」合进同一个「站不住」，后者才是承重的。元判断，不是开着的线。

### 2.2 去向已写、今天仍开着的线

| 线 | 写在哪 | R3 今天 |
|---|---|---|
| V4 名单手写、没有检查与共用库绑定 | `research/prompts/m3-defs-b6-r1-main-verification.md:41`：「V4 名单手写没有检查绑定」 | 仍手写：Y4 的 V4-4、V4-7、V4-10 |
| V5 甲：副本里 import 的库被改，闸看不到 | 同上第 41 行：「V5 甲副本里 import 的库被改闸看不到」 | 辩方 D2-X3-V5甲 复现三份副本都放；共用约束已把它写成「不要起」，钩子仍不拦 |
| X3 甲：`env -S` 串里的 ${…} 展开 | `research/prompts/m3-defs-b6-r2-main-verification.md:41`：「X3 乙（-p 的值在变量里）」所在句的前半 | 第三轮 Y2 攻方腿的格，我不判 |
| X3 乙：`-p` 的值在变量里 | 同上第 41 行：「X3 乙（-p 的值在变量里）」 | 同上 |
| V8 甲：文件头读法与正文行为不一致，静态看不出 | 同上第 41 行：「V8 甲（文件头读法与正文行为不一致静态看不出）」 | D2-X7-c份 复现；E171 编排不在 sha256 里（D2-X7-编排、A2-X7-编排）是同一个根，判决没单写 |
| 三种新类型多数派发不带重活 | 第一轮判决第 41 行：「出路是已判行（L1 写清）」 | 已写清（`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/agents/clerk.md:24`：「这一行也能顶替缺的」）；Y4 的 V2-6 指出只有已判行时子 agent 取什么数没写 |
| X1 乙：代码块示例行被当成那一行 | 第二轮判决第 41 行：「X1 乙（代码块示例行，M1 一并改）」 | 已改：`y4-feed.txt` b4，代码块里的「线程上限：8」不当那一行（缺行 rc=2，不是因为 8） |

Y5 的结论与推翻条件：四条腿 99 个打中里 77 个有去向，22 个（11 条）两轮判决都没写去向；其中第 1、2、3、4、5、6 条有可复现的历史或 R3 今天的行号，第 7 条是推的，第 8 至 11 条是弱或元判断。推翻它的现象：在两份判决（含核查记录）里搜到这些条的任何一条的去向；或第 5 条的四份样本在 R3 判法下被点名（E8 里出现）。

## 3 Y4：V2、V4 两处文字在第三轮冻结副本里与钩子、脚本对得上吗

范围：V2 = 让出 CPU 的两句（`.claude/main-agent.md` 与 `.claude/agent-common.md` 各一句）；V4 = `--test-threads 1` 的文字（`.claude/rules/verification.md`、`scripts/crash/crash-point-replay.sh`、`.claude/gate.d/checker-tier-layer0-replay.sh` 的文字，对着 `scripts/crash/`、`scripts/gate/admission.py`、`crates/singlefs-checker-tier/tests/`）。判法：一致 / 不一致 / 文字没写到（钩子或脚本做了而文字没写，或文字写到的情形里有一种没写取法）。V2 的两句是给主 agent 的规矩，「明说」「负载忙不算」钩子判不了，我把「钩子不判而不矛盾」记成一致并写明无机器判。喂法：V2 用 `runner-dispatch-guard.sh --check-prompt` 喂 14 份提示（`tools/y4_feed.sh`，在 R3 的拷贝上跑，结果在 `out/y4-feed.txt`），V4 用 grep 与读代码。引文行号都在 R3。

| 编号 | 句（R3:行） | 对着的钩子 / 脚本（R3:行） | 判 | 什么现象会推翻 |
|---|---|---|---|---|
| V2-1 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/main-agent.md:53`：「用户或别的会话在消息、弹窗里明说要让出 CPU 时（只看到负载忙不算），只改新派的子 agent」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/hooks/runner-dispatch-guard.sh:496`：「return None if not reason or reason.startswith("<") or reason in NONE_WORDS else reason」 | 一致（规则句，无机器判）：已判行只核理由非空、不以 < 起头、不是 NONE_WORDS 里的词；喂 `线程上限：8` 加「机器忙，负载高，让出 CPU」「用户在消息里说让出 CPU」「x」都 rc=0，加「<理由>」「无」rc=2（`y4-feed.txt` a3、a4、a6、a7、a8）。「明说」「负载忙不算」钩子看不到 | 理由写「机器忙」被钩子退 2，则变成钩子多判了一条而定义没写 |
| V2-2 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/agent-common.md:61`：「用户或别的会话在消息、弹窗里明说要让出 CPU 时（只看到负载忙不算），主 agent 新派的写别的数，并带一行」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/hooks/runner-dispatch-guard.sh:1120`：「valve = valve_reason(prompt, "线程上限已判") is not None」 | 一致：写别的数而不带已判行 rc=2（a2），带已判行 rc=0（a3、a4） | `线程上限：8` 不带已判行也 rc=0，或带了已判行仍 rc=2 |
| V2-3 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/main-agent.md:53`：「不为改线程上限给在跑的子 agent 发消息，它们照原样跑完」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/main-agent.md:39`：「除此之外不给在跑的子 agent 发消息」 | 一致（无机器判）：SendMessage 钩子 continuation-guard.sh 里「线程」零命中（`y4-grep.txt`），只判收件人交没交回过；两句并存不冲突：39 行管定义与规则的改动，53 行管线程上限的数 | continuation-guard.sh 开始拦含「线程上限」的消息 |
| V2-4 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/agent-common.md:61`：「不为改线程上限给在跑的子 agent 发消息，在跑的照原来的 N 跑完」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/agent-common.md:61`：「提示里别处写的线程数一概不照」 | 一致（无机器判），同 V2-3 | 同 V2-3 |
| V2-5 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/main-agent.md:53`：「也能顶替缺的」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/hooks/runner-dispatch-guard.sh:1120`：「valve = valve_reason(prompt, "线程上限已判") is not None」 | 一致：clerk 提示只有已判行没有「线程上限：」行 rc=0（a5）；three-way-forward 没写线程行也没写已判行 rc=2 带「没有线程上限：N 一行」（b1） | 只有已判行的提示被 ⑮ 拒 |
| V2-6 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/agent-common.md:61`：「提示里别处写的线程数一概不照」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/hooks/runner-dispatch-guard.sh:1120`：「valve = valve_reason(prompt, "线程上限已判") is not None」 | 文字没写到：提示里只有「线程上限已判：…」没有「线程上限：」行时钩子放行（a5、a9：理由里写「用 8 线程」也放），共用约束只说 N 照「线程上限：N」那一个整数、别处的数一概不照，没写这种提示里子 agent 起重活取什么数 | 共用约束补上这种提示的取法，或钩子对只有已判行的重活类型拒 |
| V4-1 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/rules/verification.md:59`：「门禁 checker-tier-layer0-replay 带 gpu 特性那一趟」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/checker-tier-layer0-replay.sh:734`：「gpu_test_arguments=(--test-threads 1)」 | 一致：特性表里有 gpu 才加（734 行条件 `*,gpu,*`），768 行的 cargo 命令末尾接 `${gpu_test_arguments[@]…}` | 734 行的条件去掉，或 768 行不再接这个数组 |
| V4-2 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/rules/verification.md:59`：「scripts/crash/crash-point-replay.sh 开了 GPU 的单机分支带」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/scripts/crash/crash-point-replay.sh:467`：「test_threads_arguments=(--test-threads 1)」 | 一致：467 行在 `if (( gpu_switch ))` 里；487 行唯一起快档测试的 cargo 命令接 `${test_threads_arguments[@]…}`；脚本里别处没有起 GPU 测试的 cargo test | 出现第二条不接这个数组的快档 cargo test |
| V4-3 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/rules/verification.md:59`：「该脚本的双机分支（只起 scripts/crash/crash-amplification-two-hosts.sh」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/scripts/crash/crash-point-replay.sh:482`：「run_fast_tier_target_on_two_hosts "$target" "$owners"」 ；`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/scripts/crash/crash-amplification-lib.sh:172`：「cargo_test_arguments=(test --release -p singlefs-checker-tier --features "$features" --test "$test_target" -- --nocapture)」 | 一致：双机开着时只有经给定状态引擎的测试目标走 two-hosts，别的目标仍走单机路径、带 `--test-threads 1`；共用库拼参数无该参数，三个 crash-amplification 脚本里 `test-threads` 零命中（`y4-grep.txt`） | 共用库或 two-hosts 脚本里出现 `test-threads` |
| V4-4 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/rules/verification.md:59`：「scripts/gate/admission.py 拼的 crash-case 命令、崩溃放量、双机驱动还没带，改成带时同时改这一条」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/scripts/gate/admission.py:2471`：「"--", "--include-ignored", "--exact", case.function, "--nocapture"]」 | 一致：命令以 --nocapture 收尾；admission.py 里 `test-threads` 只有 5273 行的临时目录前缀一处；「同时改这一条」没有机器绑定（名单手写，第一轮判决已记账） | admission.py 拼命令处出现 --test-threads |
| V4-5 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/rules/verification.md:59`：「CPU 一侧的并行写在用例里，用例自己起多线程」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2427`：「let threads = cpu_threads_from_the_value_of_the_variable(」 | 一致：线程数在用例里读变量、按线程分段判，不经 capped.sh 的 N | 用例改成读 SINGLEFS_THREAD_CAP |
| V4-6 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/rules/verification.md:60`：「GPU 对拍测试的 CPU 一侧读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取核数（available_parallelism），不读登记的每台默认重活线程数」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:265`：「std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::MIN))」 ；`/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:86`：「std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::MIN))」 | 一致：变量名常量在 208、72 行；capped.sh 里该变量零命中（`grep` 无输出），所以 `capped.sh N` 起的进程里它是 unset，N 管不到这一处；两份测试里 heavy-thread-default、SINGLEFS_THREAD_CAP、RUST_TEST_THREADS 零命中 | capped.sh 开始设这个变量，或测试读登记值 |
| V4-7 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/rules/verification.md:60`：「代码批改成取登记值时同时改这一条」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/heavy-thread-exempt:77`：「crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:available_parallelism」 | 一致：这两份测试在排除表里登记为「代码批」（77、78 行）；「同时改」没有机器绑定 | 代码批把两份测试改成读登记值，而这一条与排除表两行没人跟着改（现在没有机器核这一对） |
| V4-8 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/scripts/crash/crash-point-replay.sh:43`：「开了 GPU 时单机路径的每一趟快档测试参数带」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/scripts/crash/crash-point-replay.sh:882`：「local extras_count=8」 | 一致：43–46 行文字对 467、487 行；114 行说「gpu 那 11 趟」= 3 个测试目标 + extras_count 8（882 行），样本档没跑，数只对到算术 | 882 行的 extras_count 改了而 114 行没改 |
| V4-9 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/checker-tier-layer0-replay.sh:59`：「特性里有 gpu 时这一趟的测试参数带」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/gate.d/checker-tier-layer0-replay.sh:734`：「gpu_test_arguments=(--test-threads 1)」 | 一致：59–62 行（带、还没带三处、CPU 判器线程不受管）对 733、734 行；还没带的三处同 V4-3、V4-4 | 同 V4-1 |
| V4-10 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/.claude/rules/verification.md:59`：「带 --test-threads 1 跑」 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b6-r3/repo/scripts/mutation/crates-mutation-rows.py:1627`：「run, seconds = run_under_wrapper(["cargo", "test", "--offline"] + with_shown_output(row.arguments.split()), temporary_directory, f"line-{row.line_number}")」 | 文字没写到（弱）：crates 变异整表也起 GPU 测试：`mutations.tsv` 3953 行里特性表含 gpu 的 473 行（命令里出现 gpu 字样的 661 行，含测试名）、`test-threads` 零命中（`y4-grep.txt`），没出现在「带」「还没带」两张名单里；其中 471 行用 `-- <过滤词>` 只起一个测试，另 2 行（第 1947、2149 行）跑整份测试二进制，但那两份各只有 1 个 `#[test]`，所以并发的理由不成立，不算违反，只是名单不全 | 整表的变异行不再各起一个测试，或把它写进「还没带」 |

Y4 的结论：
- V2：6 行里 5 行一致（无机器判），1 行文字没写到（V2-6）。V2-6 的现象：提示里只有「线程上限已判：…」没有「线程上限：」行时，钩子放行，主 agent 文字（`main-agent.md:53`）也说这一行能顶缺行，但共用约束只说子 agent 照「线程上限：N」那一个整数，没写这种提示里起重活取什么数。三种新类型（clerk、three-way-forward、three-way-defense）多数派发走这条路。
- V4：10 行里 9 行一致，1 行文字没写到（V4-10，弱：crates 变异整表也起 GPU 测试而没出现在「带」「还没带」两张名单里，但每行只起一个测试，并发的理由不成立）。两张名单手写、没有检查绑定，第一轮判决已记账（V4-4、V4-7）。
- 上一轮的 V4 数字今天仍对得上：双机分支、admission.py、崩溃放量、双机驱动 `test-threads` 零命中（admission.py 一处是临时目录前缀），单机分支与 layer0 的 gpu 特性那一趟各带一次；`crash-point-replay.sh` 第 114 行说的「11 趟」= 3 + extras_count 8（第 882 行）。
- 推翻它：V2 的任何一行，对应喂法在 R3 的钩子上换了退出码（`tools/rerun.sh` 的 `y4-feed.txt` 与存的不同）；V4 的任何一行，`y4-grep.txt` 里 `test-threads` 计数不再是 0/0/0/1，或 `crash-point-replay.sh` 第 487 行的命令不再接 `test_threads_arguments`。

## 4 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y4 V2 | 一致 5 行，文字没写到 1 行 | 两句给主 agent 的规矩钩子判不了、也不矛盾；只有已判行没有「线程上限：」行时钩子放行，共用约束没写子 agent 取什么数（V2-6） |
| Y4 V4 | 一致 9 行，文字没写到 1 行（弱） | `--test-threads 1` 带与没带的名单和代码逐项相同；crates 变异整表也起 GPU 测试，名单没列，但每行只起一个测试（V4-10） |
| Y5 | 99 个打中，77 个有去向，22 个（11 条）两轮判决都没写 | 有复现的 6 条：内存上限、已判行理由不看内容、E171 提示侧无豁免、`stage-owners.tsv` 第 48 行、X2 四种从未被看过的漏、private-values「取不到」分支；今天仍开着的另立账 5 条（V4 名单、V5 甲、X3 甲、X3 乙、V8 甲） |
| Y6 | 98 行：对得上 89，对不上 2，不稳 2，口径不清 1，整行复核不了 1，半数复核不了 3 | 对不上的是「多 2 行」（Y6-30）和一处引文位置（Y6-34）；94 / 66 / 58 是批 6 收尾那一刻的数，今天（第三轮副本）是 117 / 89 / 74 |

## 5 复跑原样输出

`tools/rerun.sh` 五段，都经 `bash scripts/hosts/run-with-memory-cap.sh 2G` 与 `bash scripts/hosts/capped.sh 31`，工作目录在草稿目录，每段与模型目录 `out/` 里存的逐份比，全是 `same`：

```
RERUN_PARTS="counts snapshot cite htd pv y4" bash <模型目录副本>/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-defs-b6-r3/repo --work <新目录> --model <模型目录副本>
name=rerun_compare file=cite-check.txt result=same
name=rerun_compare file=htd-cell.txt result=same
name=rerun_compare file=htd-samples.txt result=same
name=rerun_compare file=patch-counts.txt result=same
name=rerun_compare file=pv-counts.txt result=same
name=rerun_compare file=snapshot-check.txt result=same
name=rerun_compare file=summary-counts.txt result=same
name=rerun_compare file=y4-feed.txt result=same
name=rerun_compare file=y4-grep.txt result=same
```

`RERUN_PARTS` 取 `models-r1`、`models-r2-opus`、`models-r2-sonnet` 各一段（四条旧腿的 `rerun.sh` 在拷贝上复跑），`out/` 里存的与复跑一致：

```
name=model_rerun leg=r1-sonnet same=11 DIFF=1
name=model_rerun leg=r1-opus same=20 DIFF=1
name=model_rerun leg=r2-opus-without-B3 same=23 DIFF=2
name=model_rerun leg=r2-opus-with-B3 same=24 DIFF=2
name=model_rerun leg=r2-sonnet same=45 DIFF=0
```

四条旧腿的数与判决所说的对照：第一轮正推腿 12 份 11 same（差的 `lr.out` 是按字节截断的位置）；第一轮攻方腿 21 份在我的直接复跑里三次 21 same，经 `rerun.sh` 复跑那一次 20 same 加 `v7-path-probe.txt`（文件数 14350 对 14352）；第二轮攻方腿不设 `B3_FROZEN` 25 份（23 same 加 2 份 x6 截断），设了 26 份（24 加 2）；第二轮辩方腿 45 份全 same。

重算的核心行（原样，来自 `out/`）：

```
门槛 8 个字；配置里取到的词 7（其中带 / 6）；本机主机名词 1（带 / 0）；合计 8（带 / 6）；不足门槛没取 9
  ✓ 核了 120 处引文，对上 120 处，没判 0 处（逐处列在上面）
  ✓ 核了 85 处引文，对上 56 处，没判 29 处（逐处列在上面）
  ✓ 核了 85 处引文，对上 85 处，没判 0 处（逐处列在上面）
name=htd_cell tree=r1 scanned=832 by_dir=scripts 455、.claude/gate.d 24、.claude/hooks 17、crates 335、.claude/cargo-command-prefix 1 suspicious=94 table_rows=66 own_variables=58
name=htd_cell tree=r2 scanned=832 by_dir=scripts 455、.claude/gate.d 24、.claude/hooks 17、crates 335、.claude/cargo-command-prefix 1 suspicious=94 table_rows=66 own_variables=72
name=htd_cell tree=r3 scanned=832 by_dir=scripts 455、.claude/gate.d 24、.claude/hooks 17、crates 335、.claude/cargo-command-prefix 1 suspicious=117 table_rows=89 own_variables=74
```

## 6 要主 agent 跑的命令（门禁格本身，子 agent 起不了 `--check`）

- 命令（工作目录取第三轮冻结副本的根，带 `SINGLEFS_HEAVY_TESTS=commit`，经 `bash scripts/hosts/run-with-memory-cap.sh 2G`）：`bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults <第三轮冻结副本根>`。看末行是否写「扫了 832 份」「可疑写法 117 处」「表 89 行」「线程变量 74 个」。我的代跑（抽格里的 python，`out/htd-cell.txt`）给的是这几个数；格本身的退出码、`changed-paths.sh` 的基准取法与抽出来跑有差别（基准状态取 none，只缩不涨那一条没比）。
- 同一方式复核 Y6-92 那一行的另两格：`--check long-run-header-exempt,gate-structure`，期望「6 行」「查了 17 道阶段、共 113 格」。
- 以上都是核已知数，不是新问题；判决里引 Y6 的数之前主 agent 本来就要现跑，这里只列命令。

## 7 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决：Y5 的 22 行「无去向」只列出处与历史，补进另立账还是交用户表由主 agent 定；Y6 的「对不上」只写口径，不改判决里的数。
- 不读这一轮攻方腿的报告与模型目录（`research/prompts/m3-defs-b6-r3-opus-output.md`、`…-opus-model/`），没读主工作区里被判的定义、规则、钩子、脚本与门禁（读的都是冻结副本；`.claude/agent-common.md` 开工先读那一次读的是主工作区那一份，没拿它当证据）。
- 没跑任何登记的 `--selftest`，没起 cargo、重型测试、QEMU、`gate.sh`、门禁格 `--check`；门禁格的结局用抽格 python 或库调用代跑，已逐处标「代跑」。
- 没核的：Y6-58 的「逐个看过都不是读环境」（人工逐个看）；Y6-72 的「fuzz 3」；第一轮攻方腿报的「8G 在包装里排队不起」；Y6-08、Y6-24 里「处」数对应的门禁格输出（代跑对得上，格本身没跑）；两轮判决里「判决写的门禁格结局」凡我没代跑的（abandoned-rounds 格的 symlink-source 那棵树缺腿文件、退 77，我只核了库读数）。
- 没替四条旧腿的报告内部的数（例如辩方报告里的 310 个轮名、攻方报告里的 14352 / 14446 个文件）逐个重算，只核两轮判决、三轮正文、调度记录里引的数。
- 没改冻结副本一个字节；没碰 `multi-host.env` 的任何值；没做 git 写操作。
- 草稿目录 `~/.local/share/singlefs/work/m3-defs-b6-r3-sonnet/` 里的仓副本、编译目录与拷贝出来的模型目录已删（`rm -rf` 逐个带 `${变量:?}` 守卫），剩下的是各次复跑的 `out/` 与日志，共 6 MB，没有编译产物，不删（主 agent 要对照复跑输出时用）。

## 8 交回前核

- `python3 scripts/agents/cite-check.py research/prompts/m3-defs-b6-r3-sonnet-output.md` 末行：见下一行（交回时再跑一遍，数以交回时那一遍为准）。
  ✓ 核了 317 处引文，对上 317 处，没判 0 处（逐处列在上面）
- 报告分段写：第一段排他新建，之后每次追加不超过 150 行。
- 模型目录 `research/prompts/m3-defs-b6-r3-sonnet-model/`：`tools/` 8 个文件（`rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来、只换 `rerun_steps` 那一段；用法 `RERUN_PARTS="<段名…>"` 分段，默认全部；仓根与冻结副本用 `--repo`、`--frozen` 传；`R1_FROZEN`、`R2_FROZEN`、`B3_FROZEN`、`WORK_ROOT`、`INTEGRATION` 缺省取 `--frozen` 的同级目录）、`out/` 12 份。
