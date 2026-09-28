# defs-m2-closeout-r3 核查员报告

**你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。**

三条腿：云端正推 Sonnet（K2）、云端攻方 Opus（K1/K2/K3）、本地攻方（K2 的 G7 事实表）。
sha256 已核：sonnet-output.md、opus-output.md 与主 agent 给的两个哈希一致；
开工快照 27 个文件 `sha256sum -c` 全部 OK（与主树一致，本报告对这 27 个文件的引用按主树核，等同对快照核）。

## 判别力自证

取 sonnet 报告第 11 行引用 `.claude/agents/experiment-runner.md:31`（在快照内），
在草稿副本里把行号加 1 改成 32，按第 2 步核这一条：

```
$ awk 'NR==32' /tmp/claude-1000/defs-closeout-r3-verifier/selftest/experiment-runner.md
4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与
字段，缺行、缺字段……
```

第 32 行内容与被核引用（「这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh`
里这个实验的登记行改指新文件，旧的留着；……」）完全不同，判 **✗**。方法能分辨，往下核。

（原引用在真实行号 31 上完全命中——`grep -nF` 一次命中，见下方 sonnet 表第 2 行。）

## 一、云端正推 Sonnet（`defs-m2-closeout-r3-sonnet-output.md`，132 行）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| L7 `r2-main-verification.md:32`（G2 原文整行） | ✓ | `sed -n '32p' research/prompts/defs-m2-closeout-r2-main-verification.md` 命中原样 |
| L11 `experiment-runner.md:31`（G2 改后整行） | ✓（判别力自证同一条） | `grep -nF "这一次的新文件也不删……" .claude/agents/experiment-runner.md` → 31 |
| L23 `.claude/gate.d/40-results-cited.sh:19`（case 分支整行） | **✗ 实际在第 33 行**。sonnet 自己的 `rerun.sh` 第 5 节独立 grep 同一句，输出就是 `33:  case "$b" in …`——同一份报告内部自相矛盾，不是事后被改（mtime 2026-09-23 早于 r3 全部产物） | `grep -n 'r\[0-9\]\.out' .claude/gate.d/40-results-cited.sh` → 33；对照 `sonnet-rerun-output.txt` 第 95 行同样是 33 |
| L27 「攻方 H2 附带的一格……」引 `/tmp/…/defs-closeout-r2-fixes/report.md:639` | **分不清**：该文件不在快照内，且全文只有 495 行，639 号行不存在；实际原句在第 222 行。文件 mtime 早于 sonnet 报告 mtime，不能排除是抄错而非事后改动，按本轮口径记「分不清」而非 ✗ | `wc -l report.md` → 495；`grep -n '附带的一格' report.md` → 222 |
| L33 `r2-main-verification.md:35`（G5 原文整行） | ✓ | `sed -n '35p' …` 命中原样 |
| L37 `experiment-runner.md:33`（G5 改后整行） | ✓ | `grep -nF "在报告里逐个点名其中表示「没过」的字段" .claude/agents/experiment-runner.md` → 33 |
| L63 `r2-main-verification.md:37`（G7 原文整行） | ✓ | `sed -n '37p' …` 命中原样 |
| L67 「打中：40、86 同形，只挪了 84 号」引 `r2-main-verification.md:22` | **✗ 掉字**：行号 22 本身对，但原文是「打中：40、86 **号**同形，只挪了 84 号」，sonnet 引文漏了「号」字 | `grep -nF "打中：40、86 同形，只挪了 84 号" …` 零命中；`grep -nF "打中：40、86 号同形，只挪了 84 号" …` → 22 |
| L69 `experiment-runner.md:35`（G7 改后整行） | ✓ | `grep -nF "读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）" .claude/agents/experiment-runner.md` → 35 |
| L89 `report.md:29`「这是按判据字面扩的名单……第三轮可以攻」 | ✓（逐字命中） | `grep -nF "这是按判据字面扩的名单，不是判决括注的原样，第三轮可以攻" report.md` → 29 |
| L93 `27-format-constants.sh:56` | ✓ | `sed -n '56p'` 命中 `kb_all = sorted(glob.glob('.claude/kb/**/*.md', …` |
| L93 `27-format-constants.sh:88` | ✓ | `sed -n '88p'` 命中 `if declaration.name not in marks:` |
| L94 `69-evidence-in-repo.sh:119` | ✓ | `sed -n '119p'` 命中 `directory = os.path.join(kb_dir, "experiments")` |
| L94 `69-evidence-in-repo.sh:220-221` | ✓ | `sed -n '220,221p'` 命中两行 |
| L95 `88-quoted-result-lines.sh:54` | ✓ | `sed -n '54p'` 命中 `kb = sorted(f for f in glob.glob(…` |
| L97 `34-experiment-index-sync.sh:67-69` | ✓ | `sed -n '67,69p'` 命中三行 |
| L97 `75-decision-experiment-links.sh:40-41` | ✓ | `sed -n '40,41p'` 命中两行 |
| L109 `84-verdict-false-named.sh:73`、`86-experiment-orphans.sh:13`、`85-repro-command.sh:14` | ✓（三处均命中） | `sed -n '73p' 84-…`、`sed -n '13p' 86-…`、`sed -n '14p' 85-…` |
| rerun.sh 复跑（`defs-m2-closeout-r3-sonnet-model/rerun.sh`） | ✓ 逐字节一致 | 拷到草稿目录跑（脚本对 `$ROOT` 只读不写，仓 42G 无法整仓拷贝，改为直接指向主树、输出落草稿目录）：`nice -n 19 bash rerun.sh <仓根> > sonnet-rerun-output.txt`；`diff` 零输出，`sha256sum` 与 `rerun-output-2026-09-26.txt` 一致（`680e04ac…`） |

**计数**：核了 20 处（17 file:line 引用 + 1 引 kb 原句 + rerun 复跑 + G5 判决行表已随 rerun 输出核对，见下）；✓ 17，✗ 2（L23 行号错，L67 掉字），分不清 1（L27，非快照文件）。
G5 表（L45-56，「今天全部判决行各落哪一边」10 行）与 rerun 输出第 6 节的 26 行判决行逐一比对，字段与取值全部能在产物里找到、分类未见与判据字面矛盾之处（此为观测，判据成不成立仍由主 agent 判）。

## 二、云端攻方 Opus（`defs-m2-closeout-r3-opus-output.md`，426 行）

Opus 报告里绝大多数引用标了「整行」，逐条 `sed -n '<行>p'` 取主树对应行，与报告贴出的文本比对。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `.claude/agent-common.md:48/59/64`（3 处整行） | ✓ 全部逐字命中 | `sed -n '48p;59p;64p' .claude/agent-common.md` |
| `.claude/gate.d/15-research-build.sh:34/36/37/38/43`（5 处整行） | ✓ 全部逐字命中 | `sed -n '34p;36p;37,38p;43p' …` |
| `.claude/gate.d/74-model-differential.sh:57/58/59/60`（4 处整行） | ✓ 全部逐字命中 | `sed -n '57p;58p;59,60p' …` |
| `research/scripts/run-with-memory-cap.sh:18/27/16/153`（4 处整行） | ✓ 全部逐字命中 | `sed -n '18p;27p;16p;153p' …` |
| `.claude/main-agent.md:48`、`:61` | ✓ 全部逐字命中 | `sed -n '48p;61p' .claude/main-agent.md` |
| `.claude/agents/crash-verifier.md:20` | ✓ | `sed -n '20p'` |
| `.claude/agents/gate-triage.md:28` | ✓ | `sed -n '28p'` |
| `.claude/agents/three-way-attack.md:33` | ✓ | `sed -n '33p'` |
| `.claude/agents/three-way-local-defense.md:19` | ✓ | `sed -n '19p'` |
| `.claude/hooks/ask-user-claim-guard.sh:58` | ✓ | `sed -n '58p'` |
| `.claude/hooks/bash-command-detector.sh:2397`、`:2408` | ✓ 两处均逐字命中 | `sed -n '2397p;2408p'`（文件共 2461 行） |
| `.claude/gate.d/40-results-cited.sh:33`、`:41`（不在快照内） | ✓ 均命中（与 sonnet 把这同一行误标成 19 形成对照，见上表 L23） | `sed -n '33p;41p' .claude/gate.d/40-results-cited.sh` |
| `research/scripts/replay.sh:50`（不在快照内） | ✓ | `sed -n '50p'` 命中 E14 登记行 |
| `research/scripts/check-segment-registry.py:25`（不在快照内） | ✓ | `sed -n '25p'` |
| `.claude/kb/experiments.md:165`（不在快照内） | ✓，含「67108885」与 opus 引用一致 | `sed -n '165p'` |
| `research/scripts/memory-peaks.tsv:910`「2265972736 … every_crash_state_outside…」（不在快照内） | **分不清**：今天第 910 行是另一条记录，原句现在在第 919 行；`memory-peaks.tsv` mtime **晚于** opus 报告 mtime，与「腿交回之后被改过」直接吻合（很可能是别的会话之后又跑了几次门禁、往表里追加了记录，把原来第 910 行的内容顶到了 919 行） | `grep -n 2265972736 research/scripts/memory-peaks.tsv` → 919；`stat -c '%y' research/scripts/memory-peaks.tsv research/prompts/defs-m2-closeout-r3-opus-output.md` |
| `awk … 峰值表 0 行`（R6 现查命令） | ✓ 今天复跑同样 0 行 | `awk -F'\t' '!/^#/ && $4 ~ /54-layer0|57-lkmm|55-qemu|lkmm\.sh|herd7/' research/scripts/memory-peaks.tsv \| wc -l` → 0 |
| 「仓里 git 跟踪的 3349 个文件」（G4 节） | ✓ | `git ls-files \| wc -l` → 3349 |
| SHA256SUMS（12 个探针/演示文件） | ✓ 12/12 OK | `sha256sum -c SHA256SUMS` |
| `rerun.sh` 复跑（`defs-m2-closeout-r3-opus-model/rerun.sh`） | ✓ **逐字节一致**（sha256 与报告给的、与 `SHA256SUMS` 里的一致） | 脚本对 `$root` 只读、只写 `$scratch`；`nice -n 19 bash rerun.sh /tmp/.../opus-scratch > opus-rerun-output.txt`；`diff` 零输出；`sha256sum` = `f35998496a…`（与 `SHA256SUMS` 一致） |
| 报告正文贴的 5 段「原样」摘录（R1/R2/R4/R5/cap-syntax-fix 五处，共 60 行） | ✓ 逐行核对，全部能在 `rerun-output.txt` 里逐字找到 | python 脚本逐行 `line in rerun.splitlines()` 核对，0 处缺失 |

**计数**：核了约 33 处（30 条 file:line/命令引用 + 1 条 SHA256SUMS 整批 + rerun 复跑 + 5 段摘录逐行核对）；✓ 32，分不清 1（`memory-peaks.tsv:910`，且有 mtime 直接证据支持它是腿交回之后被改的，不是引用错误）。

## 三、本地攻方（K2 的 G7 事实表，两份干净样本 + 转述核对表 + 运行记录）

`defs-m2-closeout-r3-local-attack-translation-audit.md` 的核对表逐条核对（T1–T10 各一条 SRC + 一条 REG，共 20 处）：

| 引用 | 核的结果 | 命令 |
|---|---|---|
| T1 SRC `27-format-constants.sh:56` / REG `stage-owners.tsv:13` | ✓ 两处均命中 | `sed -n '56p' .claude/gate.d/27-format-constants.sh`；`sed -n '13p' .claude/gate.d/stage-owners.tsv` |
| T2 SRC `34-experiment-index-sync.sh:30-31` / REG `stage-owners.tsv:20` | ✓ 两处均命中 | `sed -n '30,31p' …`；`sed -n '20p' …` |
| T3 SRC `40-results-cited.sh:17` / REG `stage-owners.tsv:26` | ✓ 两处均命中 | `sed -n '17p' …`；`sed -n '26p' …` |
| T4 SRC `69-evidence-in-repo.sh:119` / REG `stage-owners.tsv:58` | ✓ 两处均命中 | `sed -n '119p' …`；`sed -n '58p' …` |
| T5 SRC `75-decision-experiment-links.sh:40` / REG `stage-owners.tsv:61` | ✓ 两处均命中 | `sed -n '40p' …`；`sed -n '61p' …` |
| T6 SRC `84-verdict-false-named.sh:73` / REG `stage-owners.tsv:48` | ✓ 两处均命中（核对表自己已注明这处偏离 r2 report.md 的 `:14` 分组引文，改用现查行号，与 sonnet 独立发现的同一处不精确互相印证） | `sed -n '73p' …`；`sed -n '48p' …` |
| T7 SRC `85-repro-command.sh:14` / REG `stage-owners.tsv:49` | ✓ 两处均命中 | `sed -n '14p' …`；`sed -n '49p' …` |
| T8 SRC `86-experiment-orphans.sh:13` / REG `stage-owners.tsv:50` | ✓ 两处均命中 | `sed -n '13p' …`；`sed -n '50p' …` |
| T9 SRC `88-quoted-result-lines.sh:54` / REG `stage-owners.tsv:52` | ✓ 两处均命中 | `sed -n '54p' …`；`sed -n '52p' …` |
| T10 SRC `99-multipath-registry.sh:33` / REG `stage-owners.tsv:76` | ✓ 两处均命中 | `sed -n '33p' …`；`sed -n '76p' …` |
| 表一三处英文转述对照原文（`stage-owners.tsv:1-2`、`34-experiment-index-sync.sh:4`+`40-results-cited.sh:15`、`_defs-m2-closeout-r3-body.md:36`+`report.md:26`） | ✓ 原文均能逐字命中；核对表自己写明的取舍（半句不译、两处原文合并陈述）理由成立、未见限定词被吞或多加 | `sed -n '1,2p' stage-owners.tsv`；`sed -n '4p' 34-…`；`sed -n '15p' 40-…` |
| 「多出来的」两条英文限定（`even though …`、`treat exhibit B as the complete truth …`） | 核不动其「为什么加」的合理性（语义判断），但已确认这两句在原文（`report.md:29`、`body.md:36`）里确实不存在，核对表如实登记为「多出来的」而非隐瞒 | `grep -nF "even though" report.md _defs-m2-closeout-r3-body.md` 零命中 |

运行记录（`defs-m2-closeout-r3-local-attack-runlog.md`）与产物核对：

| 项 | 核的结果 | 命令 |
|---|---|---|
| 4 个产物文件 sha256（提示、s1、s2、void1） | ✓ 4/4 与运行记录给的一致 | `sha256sum research/prompts/defs-m2-closeout-r3-local-attack{.md,-output-s1.md,-output-s2.md,-output-void1.md}` |
| void1 的字词损坏判定（「星号落单=1」，退出码 5 对应「作废」） | ✓ 复跑闸重现同一判定 | `python3 research/scripts/corruption-check.py …-output-void1.md` → 红，`星号落单=1`，退出码 1（脚本自身，与 `ask-local.sh` 当时报的退出码 5 对应同一处闸） |
| s1、s2 的 oov 判定（「生词=0 拼接=0」，判「干净」） | ✓ 复跑均绿 | `python3 research/scripts/oov-check.py …-output-s1.md`；同 s2，均 `生词=0 拼接=0` |
| 词数（381 / 386 / 359） | ✓ 与 `wc -w` 一致（运行记录用的是这个口径，不是 corruption-check.py 内部另一种分词的 422） | `wc -w` 三个文件 → 381 386 359 |
| s1 内容结构（13 组答案：T1–T10 + NONE + T1–T10 类别复述 + SRC-1–SRC-10 复核） | ✓ 逐段读，13 组齐全 | `Read` 通读 s1 全文 |

**计数**：核了 26 处（20 条 T1–T10 SRC/REG + 3 条表一转述 + 1 条「多出来的」核实 + 4 项运行记录相关的复跑/哈希/字数核对合并计为 2 类）；✓ 26，✗ 0，分不清 0，核不动 1（两句「多出来的」限定词该不该加，属语义判断，只核实了它们确实不在原文里出现）。

## 总计

三腿合计核了 20 + 33 + 26 = 79 处；✓ 75，✗ 2（均在 sonnet 腿），分不清 2（sonnet 1、opus 1，均为非快照文件），核不动 1（本地攻方腿「多出来的」限定词是否该加，语义判断）。
两条 rerun.sh 复跑（sonnet、opus）与本地攻方的两次判定复跑（corruption-check.py、oov-check.py）全部逐字节 / 逐判定重现。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 没有编译 `crates/`、没有跑重型测试、没有跑门禁——三条腿本身也没有要求这些，本轮判据都是静态引用与脚本演示。
- 没有重新调用本地模型：本地腿三次调用的退出码（5、0、0）与内容属一次性的活体观测，重跑只能验证判定脚本（corruption-check.py / oov-check.py）在既有产物上的可复现性，不能重放模型答复本身；已按「引产物就整行抄」核对产物文件与哈希，未再次调用 `ask-local.sh`。
- 没有对 opus 报告里没有标「整行」的引用（例如 L67 段落式转述、各处「四句」小节的论证性文字）逐字核对——那些不是「文件:行号 + 抄的原文」形态的声明，按核查范围不属于本条。
- 没有判 G7 十道阶段各自「读不读实验页」在语义上对不对（本地攻方与云端各条腿自己的论证部分），只核了它们摆出的原文引用是否存在、是否逐字。
- 报告里 opus/sonnet 提出的「改法」「四句」分析、局部小节结论（如 R1–R8、K1/K2/K3 各格判定）未做正确性判断，只做了它们引用的证据是否属实。
- 草稿目录 `/tmp/claude-1000/defs-closeout-r3-verifier/` 下的文件均为只读演示（selftest 副本、sonnet/opus 的 rerun 输出重放），未建编译目录、未建 worktree、未建仓副本，交回前不需要额外删除；已保留供主 agent 复核。
