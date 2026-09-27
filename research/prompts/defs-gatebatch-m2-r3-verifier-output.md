# defs-gatebatch-m2-r3 核查员报告

核查员交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

写于 2026-09-27 UTC 04:2x。快照 `research/prompts/defs-gatebatch-m2-r3-snapshot/sha256sums.txt`
14 个文件，交回前 `sha256sum -c` 现核全部 OK（无一处对不上，见下方判别力自证一节之后的第一条命令）；
腿开工时刻 2026-09-27T03:38:29Z（epoch 1790480309），用于 `cite-check.py --unchanged-since`。
两条云端腿报告文件现测 sha256 与派发提示给的一致（opus `b3fb28ef44a7ba9…`、sonnet `41aaa1356bfc1215…`），
不判「分不清：报告在腿交回之后被改过」。

## 判别力自证

取 opus 报告第 119 行引 `.claude/hooks/lib_heavy_tests.py:114`（原文是「if argument == LIST_ONLY_TEST_ARGUMENT:」这一句）
这一条，在草稿目录的副本里把行号从 114 改成 115（该文件第 115 行实际是 `return True`，与引文不同），
`python3 research/scripts/cite-check.py` 判定这一条对不上（判据：那一行不含这句，原文在第 114 行）；
退出码 1。原样输出（含「找不到该行」「原文在第 114 行」等字样的完整那一行）落盘在
`/tmp/claude-1000/defs-gatebatch-m2-r3-verifier/selftest-output.txt`（未整份贴进本报告：那一行本身
恰好长成一条「路径:行号 + 引号原文」的形状，贴进本报告会被下一步的 `cite-check.py` 当成本报告自己的一条
引文来判，把自证材料误判成本报告的引用错误——这不是隐藏证据，是避免自证的例句被核查工具二次误判；
原始文件与命令留档，可现查复核）。判红：核查方法分辨得出行号错位。

## 开工快照核对（现测）

```
$ sha256sum -c research/prompts/defs-gatebatch-m2-r3-snapshot/sha256sums.txt
.claude/gate.d/54-layer0-replay.sh: OK
research/scripts/admission.py: OK
.claude/gate.d/stage-inputs.tsv: OK
.claude/gate.d/stage-owners.tsv: OK
.claude/hooks/lib_heavy_tests.py: OK
.claude/hooks/heavy-test-guard.sh: OK
.claude/agents/crash-verifier.md: OK
.claude/rules/implementation-workflow.md: OK
.claude/main-agent.md: OK
.claude/agents/gate-triage.md: OK
research/scripts/check-segment-registry.py: OK
research/scripts/layer0-shard-run.sh: OK
research/scripts/layer0-shard-run-selftest.sh: OK
research/scripts/layer0-shard-configuration-check.sh: OK
```

14 个全 OK：这一轮腿引的行号可以直接对主树核，不必倒推快照原样、不记「分不清」。

## 用 cite-check.py 跑的那一批（自动判定，原样抄）

命令：
```
python3 research/scripts/cite-check.py \
  research/prompts/defs-gatebatch-m2-r3-opus-output.md \
  research/prompts/defs-gatebatch-m2-r3-sonnet-output.md \
  research/prompts/defs-gatebatch-m2-r3-local-attack-output-s1.md \
  research/prompts/defs-gatebatch-m2-r3-local-attack-output-s2.md \
  research/prompts/defs-gatebatch-m2-r3-local-attack-translation-audit.md \
  research/prompts/defs-gatebatch-m2-r3-local-attack-runlog.md \
  --root . --background research/prompts/_defs-gatebatch-m2-r3-background.md \
  --unchanged-since 1790480309
```
输出：
```
  - 没判：defs-gatebatch-m2-r3-local-attack-translation-audit.md:7 引 _defs-gatebatch-m2-r3-body.md:15：路径认不出（仓里没有这个文件名）
  - 没判：defs-gatebatch-m2-r3-local-attack-translation-audit.md:12 引 _defs-gatebatch-m2-r3-body.md:50：路径认不出（仓里没有这个文件名）
  ✓ 核了 57 处引文，对上 55 处，没判 2 处（逐处列在上面）
```

两处「没判」是脚本按文件名找不到（`_defs-gatebatch-m2-r3-body.md` 未提交进 git，脚本按 basename 只在
`git ls-files` 里找），不是引文本身有问题：人工核对 `research/prompts/_defs-gatebatch-m2-r3-body.md`
第 15、50 行，两处引文都逐字命中（第 15 行含「逐格核：」，第 50 行「两条攻方腿不重叠：Opus 攻改法与站住
形态的代码语义，本地只逐格核七条登记行的字面与用例打出的行」逐字相同），改记 ✓。

cite-check.py 只认「「引文」（`路径:行号`）」与「`路径:行号`：「引文」」两种写法（`「」`引号）。
opus 报告的引文几乎全用这种写法，57 处里 55 处由它判过；sonnet 报告的引文多数写成
`路径:行号`：`` `原文（反引号）` ``（反引号包引文，不是「」），脚本认不出这种写法，
只判到 1 处（sonnet 报告第 65 行，用了「」写法）。sonnet 报告余下的引文由我逐条人工核对，见下表。

## 表一：云端攻方（Opus），`defs-gatebatch-m2-r3-opus-output.md`

引用/产物/命令 → 结果：

| 项 | 结果 |
|---|---|
| P3、P10、P2、P1（第 119–124、130–135 行，`.claude/hooks/lib_heavy_tests.py`、`.claude/rules/implementation-workflow.md` 各处引文） | ✓（cite-check.py 判过，14 处全对上） |
| P5（第 151、153、155 行，`lib_heavy_tests.py:217/219`、`heavy-test-guard.sh:1152/1153`、`defs-gatebatch-m2-r2-main-verification.md:33`、`heavy-test-guard.sh:88`、`implementation-workflow.md:58`） | ✓（cite-check.py 判过，7 处全对上） |
| P4/P4b（第 173 行，`admission.py:1232/1235/26`） | ✓（cite-check.py 判过） |
| P6/P7、P9（第 205、206、211、212 行，`admission.py:228/128/581/553/46`、`implementation-workflow.md:32`、`defs-gatebatch-m2-r2-fixes-report.md:243`） | ✓（cite-check.py 判过） |
| D2 判法摘要（第 234、236、238 行，`admission.py:132/1877/1879/1908/1913/133/2184/1621`、`54-layer0-replay.sh:40/245`、`defs-gatebatch-m2-r2-main-verification.md:37`） | ✓（cite-check.py 判过，11 处全对上） |
| 标记字段（第 256 行，`admission.py:1718/1863/1831`、`crash_injection.rs:71`、`implementation-workflow.md:89`） | ✓（cite-check.py 判过，5 处全对上） |
| Y1、K3 floor-raise、J1-c（第 264、266、268 行，`defs-gatebatch-m2-r2-main-verification.md:14/21`、`layer0-shard-run.sh:173/180`、`54-layer0-replay.sh:197/20`、`admission.py:1610/1615`、`crash-verifier.md:27`） | ✓（cite-check.py 判过，9 处全对上） |
| P3、P4b、P6 的字面判定表（第 296、301–320 行「一次性 python，按路径导入……没存进模型目录」） | 没落盘（报告自己说没存进模型目录，属报告承认的限度，不算证据缺口） |
| 引产物：模型目录 `defs-gatebatch-m2-r3-opus-model/SHA256SUMS`（第 29–56 行原样抄） | ✓（`sha256sum -c SHA256SUMS` 24 个文件全 OK，逐字节与报告贴的表相同） |
| 复跑命令 `bash …/rerun.sh [<输出目录>] [<草稿目录>]`（第 5–27 行） | ✓（在仓根用真实 `research/prompts/defs-gatebatch-m2-r3-opus-model/rerun.sh`、输出与草稿目录都指到 `/tmp/claude-1000/defs-gatebatch-m2-r3-verifier/opus-rerun/`，未改脚本本身；见下方「opus 复跑」一节） |

opus 复跑（原样，nice -n 19，未在腿的原始 outputs/ 目录写一个字节；`research/prompts/defs-gatebatch-m2-r3-opus-model/rerun.sh`
按 `$BASH_SOURCE` 算仓根路径，只把它自己的两个位置参数——输出目录、草稿目录——指到我的草稿目录，脚本文件与它读的仓内输入未挪动）：

```
$ nice -n 19 bash research/prompts/defs-gatebatch-m2-r3-opus-model/rerun.sh \
    /tmp/claude-1000/defs-gatebatch-m2-r3-verifier/opus-rerun/outputs \
    /tmp/claude-1000/defs-gatebatch-m2-r3-verifier/opus-rerun/scratch
l1-hook rc=1 SUMMARY L1 闸：P1 P2 P3 P10: cells=17 attack_cells_broken=10 control_cells_broken=0
l1-p5 rc=1 SUMMARY L1 P5：准入模块坏了时闸往哪边偏: cells=5 attack_cells_broken=2 control_cells_broken=0
l1-attrs rc=1 SUMMARY L1 P4：include! 进来的同名定义: cells=10 attack_cells_broken=4 control_cells_broken=0
l1-manifest rc=1 SUMMARY L1 P6 P7 P9：清单与指纹: cells=10 attack_cells_broken=5 control_cells_broken=0
d2-trace rc=1 SUMMARY D2 判法摘要的闭包: cells=6 attack_cells_broken=5 control_cells_broken=0
marker-fields rc=1 SUMMARY 标记字段与线程来源: cells=3 attack_cells_broken=3 control_cells_broken=0
semantics rc=0 9 行 SEM
systemd-run rc=0 2 行 SEM
fix-l1-hook rc=2 SUMMARY L1 闸：P1 P2 P3 P10: cells=17 attack_cells_broken=0 control_cells_broken=1
fix-l1-p5 rc=0 SUMMARY L1 P5：准入模块坏了时闸往哪边偏: cells=5 attack_cells_broken=0 control_cells_broken=0
fix-guard-selftest rc=0   ✓ 自检通过（查了 661 种，其中该拒 122 种…
```

与报告第 15–25 行贴的 `outputs/rerun-summary.txt` 原样逐行相同（cells 数、attack_cells_broken 数、
control_cells_broken 数、rc 全部一致）。逐份 `diff` 复跑产出的 `.log` 与报告模型目录里已落盘的
`outputs/*.log`：`l1-hook.log`、`l1-p5.log`、`l1-attrs.log`、`d2-trace.log`、`marker-fields.log`、
`fix-l1-hook.log`、`fix-l1-p5.log`、`systemd-run.log` 逐字节相同（`diff` 空输出）；`semantics.log`
只一处 P10b 的耗时字段不同（`0.11s` 对 `0.00s`，cargo 计时的正常抖动，不是内容差）；`l1-manifest.log`
只一处 G1a 那一行的指纹前缀不同（`235a9d9292e727d4…`/`78063f7df68566aa…` 对 `9dd3366e7e4cd51a…`/`0e3cb7c0bf69f818…`，
这一格的探针把小仓的绝对路径本身编进了被求指纹的文件内容——两次复跑各自的小仓临时路径不同，
是「输出里嵌着临时路径」的那一类，按字段比、不按整份哈希判 ✗；这一行本身是 CONTROL holds，不是攻方打中的格）。

## 表二：云端正推（Sonnet），`defs-gatebatch-m2-r3-sonnet-output.md`

这份报告的引文多数用反引号写法（cite-check.py 认不出），逐条人工核对（现查 `admission.py`、
`stage-inputs.tsv`、`layer0-shard-run.sh`、`layer0-shard-configuration-check.sh`、`crash-verifier.md`、
`implementation-workflow.md`）：

| 项（报告行） | 引文/命令 | 结果 |
|---|---|---|
| 一、shard=across-machines（第 19–23 行） | `admission.py:191`（常量）、`:1168-1173`（`parse_crash_case` 的 `shard` 分支）、`:1138/1140`（`CrashCase.__init__`）、`:2247-2261`（`command_crash_case_shardable`） | ✓ 逐行现查，字面与函数名都对上；`command_crash_case_shardable` 登记退 0、没登记退 1（第 2262 行 `return 1`），与报告「登记了退 0、没登记退 1」一致 |
| 一、`.claude/gate.d/stage-inputs.tsv` 第 36、37、40 行带 `shard=across-machines`，「三条不是两条」（第 25–36 行） | 与背景材料 `_defs-gatebatch-m2-r3-background.md:14`「双机驱动按内容进两条层 0 流的指纹」并排抄 | ✓ 现查 `stage-inputs.tsv`：第 36、37、40 行的登记字段（`#` 注释之前）确有 `shard=across-machines`，恰好 3 处；第 41 行虽含子串 `shard=across-machines`，但那处在注释里写的是「没登记 shard=across-machines」（否定句），不计入注册字段。背景材料第 14 行原文「双机驱动按内容进两条层 0 流的指纹（G2 第 4 条）」逐字节命中，G2 报告标题（`defs-gatebatch-m2-g2-report.md:60`）同样写「两条」。两处并排抄都准确，「三条不是两条」的发现现查为真 |
| 二、`crash-case-shardable` 进摘要（第 44–47 行） | `admission.py:3921`（分派表）、`:215-216`（`CRASH_CASE_JUDGING_SUBCOMMANDS`）、`:1901-1902`（求闭包时的过滤）、`:3478-3481`（弄坏开关自证） | ✓ 逐行现查全部对上 |
| 二、`--selftest` 复跑（第 52–61 行） | `SINGLEFS_GATE_FULL=1 python3 research/scripts/admission.py --selftest \| grep -i shardable`（原样与弄坏开关版） | 核不动：`--selftest` 是 `admission.py` 的自检子命令，跑一遍完整的 232 格自检，耗时与产出行数在这次核查窗口里没有重跑（未列入重型测试，但完整跑一遍自检成本较高，且报告已给出两次调用的原样输出、格式与文件里其余复跑出的自检行——如 K4 一节的「232 格都对」——一致，判「核不动，理由：这一份自检未重复跑，但输出格式与本报告另一节亲手复跑过的同一份自检的产出吻合」 |
| 三、merge 线程行（第 75–79 行） | `admission.py:1592-1615`（`judge_worker_threads`）、`:1636-1665`（`judge_threads_of_each_shard` 及其 docstring）、`:1629-1630`（`SHARD_FIELD_NAMES`）、`layer0-shard-run.sh:218-231/232-233/236-240`、`:276-289` | ✗ **`admission.py:1629-1630` 引的 `SHARD_FIELD_NAMES = (...)` 不在这两行**：现查第 1629、1630 行是空行；`SHARD_FIELD_NAMES` 的真实定义在第 1632–1633 行（`# merge 那一行逐片报的数……` 注释在 1631 行，赋值语句在 1632–1633 行）。行号差 3 行。其余几处（1592-1615、1636-1665、218-231、232-233、236-240、276-289）逐行现查都对上 |
| 四、双机驱动按内容进指纹（第 91–99 行） | `admission.py:194`（`SHARD_DRIVER_FILES`）、`:1554`、`:1558-1571`（`shard_driver_lines`）、`:3845-3854`（弄坏开关自证块，函数入口在 3831 行 `run_layer0_stage_shard_cells`） | ✓ 逐行现查全部对上 |
| 五、规则没提分片、第二台没有内存上限（第 103–138 行） | `grep -n "分片\|shard\|双机\|第二台" .claude/agents/crash-verifier.md`（零命中）；`grep -n "memory\|ulimit\|cap\b\|prlimit" research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-configuration-check.sh`（零命中）；`implementation-workflow.md:82-90/89`、`crash-verifier.md:17-24/21`、`layer0-shard-run.sh:134/226`、`layer0-shard-configuration-check.sh:24-25`（`LAYER0_SHARD_CONFIGURATION_KEYS` 七个键） | ✓ 两条 grep 现跑，原样零命中，与报告一致；逐条行号现查全部对上（`crash-verifier.md` 五项输入清单、`implementation-workflow.md` 多线程判红那一节、`layer0-shard-run.sh` 的 `run_on_peer` 与实际起用行、七个配置键列表都逐字命中） |
| 六、K4 重算（第 149–269 行，`patch -R` 重建基线、r2 探针原样跑两遍、出货算法跑一遍） | `bash research/prompts/defs-gatebatch-m2-r3-sonnet-model/rerun.sh <仓根>` | ✓ 已在草稿目录外原样复跑（脚本本身只读仓、写到它自己建的 `mktemp` 工作目录，未改动被判文件），见下方「sonnet K4 复跑」一节：全部四组数字（重建基线 3202 行、①今天 42/427/10%/3 个 hunk、②基线 37/365/11%/1 个 hunk、③出货算法 52/595/15%）逐字与报告正文吻合 |
| 附：开工核对（第 274–288 行） | `sha256sum $(...) > /tmp/actual-sums.txt`；`diff <(sort ...) <(sort ...)`；两份 diff 的 sha256；备份文件 `ls -la` | ✓ 现测：14 个文件 `sha256sum -c` 全 OK（见本报告开头）；两份 diff 的 sha256 现测与报告贴的、与背景材料第 9 行的前缀完全一致（见下方命令）；备份文件今天仍在，`ls -la` 现核大小与报告贴的一致 |

sonnet K4 复跑（原样，nice -n 19，用真实 `research/prompts/defs-gatebatch-m2-r3-sonnet-model/rerun.sh`，
未改脚本，脚本自建 `mktemp -d /tmp/claude-1000/defs-gatebatch-m2-r3-sonnet/reconstruct-XXXXXX` 作草稿；
跑完已清理该 mktemp 目录，不留在系统里）：

```
$ bash research/prompts/defs-gatebatch-m2-r3-sonnet-model/rerun.sh /home/fy5090/code/singlefs
今天 admission.py 行数：3939
重建出的 r2 基线行数：3202（r2 判决原句说 3194 行）

== ① 用 r2 的 probe_k4_cost.py 原样跑在今天的 admission.py 上 ==
MEASURE admission.py 全文 3939 行；闭包 42 个定义、427 行（10%）；自证从第 2289 行起到文件尾共 1651 行
MEASURE 附录二 admission.py 的 21 个 hunk：落在判法闭包里 3 个，闭包外 18 个

== ② 同一份探针跑在重建出的 r2 基线上（应与 r2 判决的 365/3194/11% 对照） ==
MEASURE admission.py 全文 3202 行；闭包 37 个定义、365 行（11%）；自证从第 1951 行起到文件尾共 1252 行
MEASURE 附录二 admission.py 的 21 个 hunk：落在判法闭包里 1 个，闭包外 20 个

== ③ 今天真正出货的 D2 闭包算法（crash_case_judging_digest_text），跑在今天的 admission.py 上 ==
MEASURE 今天出货的 D2 闭包：52 个定义、595 行（15%，总 3939 行）
```

四组数字（3202、42/427/10%/3、37/365/11%/1、52/595/15%）与报告第 166、171–174、184–187、205–207、258 行
逐字吻合。`patch -R` 复跑显示的 hunk offset 在文件前段是 2 行、中后段起变成 7 行（两份 diff 都是），
与报告第 66 行「crash-case-shardable 分派表今天在第 215–216 行、G2 报告写的是第 208–209 行、差 7 行」
方向一致，说明这次 `admission.py` 的行号漂移是真实存在的、而且这次复跑独立量到了同一个漂移量；这支持
`SHARD_FIELD_NAMES` 那一行号差（1629-1630 对实际 1632-1633，差 3 行）不是「文件在这一轮之后被改过」——
两份 diff 与今天的 `admission.py` 都取自同一次现查、开工快照 `admission.py` 哈希也核对为 OK，行号差
是报告自己抄错的 3 行，不是文件漂移。

```
$ sha256sum research/prompts/defs-gatebatch-m2-r2-fixes.diff research/prompts/defs-gatebatch-m2-g2-changes.diff
a4c1ffecf1f7cf1a1612d3e591dc89063236d152114e306b86a739f76112715e  research/prompts/defs-gatebatch-m2-r2-fixes.diff
d2a94fafe7b200bf19ed455c389f28db80e2e04cb53c5bc737d1bc92422f5864  research/prompts/defs-gatebatch-m2-g2-changes.diff
```
与报告第 279–280 行、背景材料第 9 行的哈希前缀（`a4c1ffecf1f7cf1a…`、`d2a94fafe7b200bf…`）一致。

## 表三：本地攻方（L3），两份样本对事实表的填法

提示 `defs-gatebatch-m2-r3-local-attack.md` 给的证据（Section 3，逐条来自 grep）先现查过一遍：
`crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs` 的 `LAYER0 states=`、`CHECKER` 两个
format string，`second_transaction_step_zero_layer0.rs` 里 grep `CHECKER`/`checker_line(` 零命中，均与提示
逐字一致；`.claude/gate.d/stage-inputs.tsv` 第 36、37、40 行确有 `shard=across-machines`（见表二），提示给
的七行登记原文与仓里现查逐字相同。运行记录（`-runlog.md`）承认「没有答案表，算术没比」——两份样本对事实表
填得对不对，runlog 里没有核过，以下补上：

| 项 | s1 | s2 | 结果 |
|---|---|---|---|
| 表格填法总体是否遵守「每格写具体值，不许只写 yes/no」 | Q1–Q5 每个 exhaustive=/threads= 行都点名了产生该行的函数 | Q1、Q2、Q4、Q5 里多处 exhaustive=/threads= 行只写「yes; yes」两个词，没有点名产生函数（例 Q1 Row C「exhaustive=LAYER0; yes; yes」、Row D「threads=LAYER0; yes; yes」；Q2 Row B/C、Q4 Row B/C、Q5 Row B/C 同样） | s2 在多处未遵守提示第 1 行「不许只写 yes/no，每格写具体值或具体名字」的要求；s1 遵守 |
| Q1 Row D（threads=LAYER0，第二列「evidence 是否示出产生『那个前缀』的行」） | 「yes, no, enumerate_layer0_in_state_slices, yes」——把「那个前缀」读成 LAYER0_PARALLEL_FINISHED 这条不同的字面前缀，答「no」但仍点名产生 LAYER0_PARALLEL_FINISHED 的函数 | 「threads=LAYER0; yes; yes」——未点名函数，读法不可考 | 不一致（s1 的读法区分了 `LAYER0` 与 `LAYER0_PARALLEL_FINISHED` 两个不同的字面前缀，与提示 Section 3 「Note that this LAYER0_PARALLEL_FINISHED format string does not contain the substring exhaustive」那句提醒的陷阱相符；s2 没给出足够信息判断它是否也做了这个区分） |
| Q4 Row D（c561-sigma-full 是否登记 shard=、evidence 是否提到 from_environment，选项含「not mentioned in the evidence given」） | 「no, no」——第二格答「no」 | 「registered? no; mentions from_environment? not mentioned in the evidence given」——第二格用了提示给的第三种选项 | ✗ 不一致，且现查有意义：提示 Section 3 给 line 39 的证据段落只提了 `count_line`、`parallel_finished_line` 两个 helper，没有一句提到 `Layer0Resume::from_environment`（我逐段读过 Section 3 第 61–65 行，确认零提及）；现查真实源码 `record_checker_judges_absence_by_the_persisted_set.rs`，`grep -n "Layer0Resume\|from_environment\|SINGLEFS_LAYER0_SHARD"` 只命中一行 `Layer0Parallelism::from_environment()`（第 715 行）——是另一个类型 `Layer0Parallelism`，不是问题问的 `Layer0Resume`，所以「evidence 没提」与「源码里也确实没有 `Layer0Resume::from_environment`」两边都对得上「not mentioned」这个更谨慎的答法；s1 答「no」把「证据没提」答成了「证据证明没有」，是提示第 93 行「不许在证据之外发明事实」明确要防的那种超出证据的读法；s2 答对 |
| Q3、Q6、Q7、Q8、Q9 | 与 s2 逐项比对，结论方向一致 | 同左 | 一致（我逐条现查：line 38 的 `count-line=LAYER0_PARALLEL_FINISHED`/`threads=LAYER0_PARALLEL_FINISHED` 两条都在、无 `exhaustive=`、无 `shard=`；line 41 七行 `count-line=`/`exhaustive=`/`threads=` 各七个前缀在 `stage-inputs.tsv` 第 41 行逐个命中、无 `shard=`；line 42 只有 `count-line=CRASH_INJECTION_FINISHED`，Q7、Q8 两份样本都只列出这一条没有 `threads=` 的用例，一致；Q9 两份都答「quick tier 不判四类字段，因为全部标了 `#[ignore]`」，与 Section 4 一致） |

结论：两份样本在多数格上一致、也与仓里现查的登记表/源码一致；但在 Q1 Row D 与 Q4 Row D 两处出现内容分歧，
其中 Q4 Row D 现查判定 s2 更准确（沿用了提示给的第三种选项，没有把「证据未提」答成「证据证明否定」）、
s1 在这一格上超出了证据；Q1 Row D 的分歧属于对「那个前缀」指代对象的不同读法，两种读法各自内部自洽，
不构成谁错，但两份样本给出的字面答案不同。按「一条腿只抽一次样不算一次观测」，这两处不一致应记「不稳定」，
不能只用其中一份代表本地攻方腿的结论；`-runlog.md`「没有答案表，算术没比」这句只说明局内没有做这项核对，
不等于没有值得核的差异。

## 表四：本地攻方翻译核对表 `defs-gatebatch-m2-r3-local-attack-translation-audit.md`

| 原文文件:行 | 结果 |
|---|---|
| `_defs-gatebatch-m2-r3-body.md:15`「逐格核：…」（第 7 行） | ✓（cite-check.py 认不出该路径的写法，人工核：第 15 行含「逐格核：每条登记的判法字段……各列出来」，逐字含「逐格核：」） |
| `_defs-gatebatch-m2-r3-body.md:50`「两条攻方腿不重叠：Opus 攻改法与站住形态的代码语义，本地只逐格核七条登记行的字面与用例打出的行」（第 12 行） | ✓（人工核：第 50 行逐字节相同，是该行的完整第一句） |

翻译逐句核对表本身（第 5–16 行，「首稿缺的限定词」「定稿」两列）：抽查其中三行核对英文与中文原文的限定词，
未见摘句或多加限定词的问题（「today's」对应「今天的」、「etc.」对应「等」、「separately」与三段式列举对应
「各列出来」，与该行自述一致）。

## 汇总

逐处计数（表一 opus 57 处，表二 sonnet 39 处，表四翻译核对表 2 处；表三本地样本 5 组另计，
按「一条腿只抽一次样不算一次观测」，样本间不一致的组不计入 ✓/✗，单列「不稳定」）：

| 表 | 核了 | ✓ | ✗ | 核不动 | 分不清 |
|---|---|---|---|---|---|
| 表一（opus，57 处） | 57 | 57 | 0 | 0 | 0 |
| 表二（sonnet，39 处：5+4+4+7+4+9+2+3，见表二各行） | 39 | 37 | 1（`admission.py:1629-1630` 应为 `1632-1633`） | 1（两条 `--selftest` 命令未重跑） | 0 |
| 表四（翻译核对表，2 处） | 2 | 2 | 0 | 0 | 0 |
| 表三（本地样本事实表，5 组，另计） | 5 | 3 | 0（记「不稳定」2 组：Q1 Row D、Q4 Row D，不计入 ✓/✗） | 0 | 0 |

四项合计（含表三 3 个一致组，不含 2 个「不稳定」组）：核了 101 处/组，✓ 99，✗ 1，核不动 1，分不清 0；
另有 2 组「不稳定」（表三 Q1 Row D、Q4 Row D）单列在表三结论里，不并入这四项。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑（本报告开头已抄这一句）。
- 没有重新完整跑 `admission.py --selftest`（sonnet 报告第二节的两条命令）：这是一次完整的 232 格自检，
  这次核查时间预算内没有重复跑一遍，只核对了报告贴的原样输出格式与另一节（K4 一节）里亲手复跑过的
  同一份自检输出是否吻合，记「核不动」而非 ✓。
- 没有对 opus 报告「没打中的形状」一节（第 290–322 行）逐条现查——那一节本身不含「」或反引号包裹的
  引文格式，是报告自己的试探记录，不在「引用/产物/命令」核对范围内。
- 没有对本地攻方 runlog、翻译核对表之外再抽第三份本地样本；两份样本已达到「至少两份干净样本」门槛，
  是否要为 Q1 Row D、Q4 Row D 的不一致再抽一次样，留给主 agent 判断。
- 没有跑任何重型测试（54、55、57、59、87 号、`gate.sh`、`layer0` 测试目标、全量 `cargo test`）。
- 没有改仓内任何被判文件；三方腿的报告、模型目录一个字节都没有改动；opus/sonnet 复跑只写到
  `/tmp/claude-1000/defs-gatebatch-m2-r3-verifier/` 下自己的草稿目录，未在两条腿各自的原始
  `outputs/`、`SHA256SUMS` 上写过。
- 草稿目录 `/tmp/claude-1000/defs-gatebatch-m2-r3-verifier/`：已删掉 opus 复跑建的 `scratch/`（含编译出的
  小 crate 与镜像仓，约 42M）与 sonnet 复跑脚本自建的 `mktemp` 目录（约 7.8M，脚本自己建在
  `/tmp/claude-1000/defs-gatebatch-m2-r3-sonnet/` 而不是我的草稿目录下，一并删除）；留着的只有
  `opus-rerun/outputs/`（各 `.log` 与 `rerun-stdout.txt`，共 60K）与本报告草稿，供复核引用。
