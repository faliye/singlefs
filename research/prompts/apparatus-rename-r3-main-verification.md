# apparatus-rename-r3 判决（推论轮第三轮，最终轮，治理轮，2026-10-23）

正文 `research/prompts/_apparatus-rename-r3-body.md`；腿报告 `research/prompts/apparatus-rename-r3-sonnet-output.md`（辩方，复核第二轮判决 `research/prompts/apparatus-rename-r2-main-verification.md` 与它定的甲路）、`research/prompts/apparatus-rename-r3-opus-output.md`（攻方）。

## 这一轮被判过的文件

`.claude/gate.d/lib-lag-baseline.py`、`.claude/gate.d/path-moves.tsv`、`.claude/hooks/lib_long_run.py`、`scripts/gate/changed-paths.sh`、`.claude/gate.d/doc-decisions.sh`、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/code-tooling.sh`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py`、`scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/path-moves.md`、`.claude/long-run-live-hosts-exempt`、`.claude/heavy-thread-exempt`（开工快照 `research/prompts/apparatus-rename-r3-snapshot/judged-sha256.txt`）。

## 一、核查

- 开工快照：写判决之前 `sha256sum -c`，只有 `.claude/agent-common.md` 对不上（腿跑着的时候别的会话改了它派发闸那一句）。`python3 scripts/agents/snapshot-tree.py apparatus-rename-r3 --out <草稿目录>/apparatus-rename-r3-tree`：「倒推出 14 份、sha256 对上 14 份；倒推不出 1 份」，倒推不出的就是 `.claude/agent-common.md`（`snapshot-tree.py` 认不出这一轮 `judged-vs-head.diff` 用 `HEAD:<路径>` 写的 diff 头）。主 agent 拿 `judged-vs-head.diff` 里它那一段打回 HEAD 那一份，sha256 `b03b2d879dc4…` 与快照相同，铺进那棵树；别的会话改的那一句不在这一轮被判的长活豁免表那一句里。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/apparatus-rename-r3-opus-output.md research/prompts/apparatus-rename-r3-sonnet-output.md --root <草稿目录>/apparatus-rename-r3-tree --background research/prompts/_apparatus-rename-r3-background.md`：「✓ 核了 61 处引文、15 处裸行号，对上 52 处，没判 24 处（其中裸行号没有引文可比 13 处；逐处列在上面）」。没判的 11 处引文引的是快照之外的文件（`reuse_library.py:417`、`.claude/singlefs-ai-sop/scripts/gate.sh:132`、`.claude/singlefs-ai-sop/scripts/lib.sh:180`、`scripts/agents/snapshot-tree.py:129`、`research/prompts/e171-conclusion-r1-opus-model/tools/rerun.sh:11`、`:18`、`research/prompts/d16-item1-r3-main-verification.md:11`、`research/prompts/apparatus-rename-r1-main-verification.md:31`），主 agent 在主工作区逐处现查，引文都在那一行。
- 复跑：两条腿的模型目录拷进会话草稿目录，经 `scripts/hosts/run-with-memory-cap.sh 4G` 跑各自的 `tools/rerun.sh`。攻方冻结树照它的 `build_frozen.sh` 建、`agent-common.md` 换成倒推出的那一份之后 15 份 sha256 都对上；`attack_samples.out` `same=yes`，sha256 `436481066834f1326420826f83100a9e6ebb804a8a53fcadc6d32bf60d7bb476`，与腿报的相同；`probe.path_moves_check` 读主工作区、会变（此刻「395 份里 359 份」）。辩方 7 份确定性输出（`probe_pinned`、`probe_window`、`probe_close`、`probe_over`、`probe_anchor`、`probe_fix`、`probe_untested`）都 `same=yes`；`scan_slash_context`、`recount_*`、`uncovered` 读主工作区，随工作区变。
- 辩方腿中途停下没交回：报告 J1–J3 三格已写完，缺末段的输出 sha256，那一段由主 agent 复跑补上。辩方自己的两趟样本是不经内存包装、用 `nice -n 19` 起的（它报告「开头：复跑与校验」一节自己写明）；主 agent 的复跑经包装、输出相同。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| J1 已定项撑不撑得住 | **甲路在门禁一侧站得住；另打中三处**（两条腿各自独立打中前两处） | 站得住的（辩方逐条造样本，攻方没打穿）：生效窗口、第二轮 H1-1 在门禁一侧、H1-4 主形状、无序键、不许串、实据、恰好换前缀、位置配对、跨类整份不剔，以及乙（`.claude/hooks/lib_long_run.py` 第 446、448、455、476 行）；丙只拦得住半换的。打中的：① **换算不锚定在仓根**（攻方 J1-A、辩方 N4）：`.claude/gate.d/lib-lag-baseline.py` 第 197 行「PATH_BEFORE_BOUNDARY = r"(?<![A-Za-z0-9_.\-])"」让旧前缀前面接 `/` 也换，别的目录下同名的一段（`tools/docs`、`vendor/tools`）被当成只差搬迁，排除表不判涨、调用方与新增行被剔，登记表照绿；主工作区这一批里真有：`research/prompts/d16-item1-r3-main-verification.md` 抄下来的 `diff -u` 头两行被换成新路径并被剔出改动范围（规则第 5 步该留旧路径，见第三节）。② **钉值一侧照用全部登记行、门禁一侧对不生效的行一字不核**（攻方 J1-C、辩方 N1、N2）：`frozen_o3.py` 第 117 行读 `library.path_moves`，`reuse_library.py` 第 442 行同一条；一行新前缀早就有文件的假登记（攻方 `pin_never_effective_row`、辩方 `occupied_row`、`token_row`）门禁一侧 0 个问题，钉值一侧认钉住的文件换了实现；辩方 `chain_old_row`：老行加一行真搬迁，第二轮 H1-1 在钉值一侧原样还在。③ **生效窗口跟着 gate 基准走**（攻方 J1-B）：有上游而没推、或推了之后走 `--staged`（`.claude/singlefs-ai-sop/scripts/lib.sh` 第 180 行取 HEAD~1），搬迁提交之后那一行仍生效；照规则第 18 行「搬一步提交一次」搬第二步、合规地删一份搬过去的文件、重用旧名三种都红，同一棵树不带 `--staged` 判绿；方向是多判，另把①的放过窗口拉长到推上去为止。辩方的「睡着的老行醒来」（`o7b`、`o7c`）是同一族的多判 |
| J2 没管到的清单全不全 | **打中**（两条腿一致） | 上面三处哪一份规则、文件头、表头都没写；`reuse_library.py` 第 417 行「不会误放」在甲路之下不成立。调用方清单漏了 `scripts/agents/snapshot-tree.py`（第 129 行按全部行倒换）与批 6c 的 decision-links（辩方 N9）。按路径管的类只列了五样，整份搬进 `.claude/gate.d/`、`.claude/singlefs-ai-sop/rules/` 照剔（攻方 J2-5，`governed_class_gaps` 样本）。句子对不上的：规则第 16 行「只差这一段前缀」、第 18 行「搬一步提交一次」、`.claude/gate.d/path-moves.tsv` 第 4 行「只按路径前缀换」（对①）；规则第 63 行的「判不了」只写了「不在 git 仓里」一半（辩方 N10）；第 17 行没写整份搬家的不动行另剔（辩方 N11）。多判方向、出路没写的：`${V:-scripts/experiments/…}` 默认值写法照算改动（辩方 N5，两份）、新前缀下基准里有占位文件时整次真搬迁不生效而登记表照绿（辩方 N6） |
| J3 报出的数能不能核 | **部分打中**（两条腿一致） | 32、19、48、6 行、395 份里 360 份都能一条命令重算；360 另有不调库的写法数出同数（辩方 `recount_files`），这个数随工作区变（此刻 359）。140 → 21、12 → 132、119 页重算不出：改前一侧只在批 6c 草稿里的备份上算得出。算法的区分度：弄坏开关红在哪几格两条腿都逐个复现；自证调判法时基准一律写死 `"HEAD"`，分不出③；钉值一侧 26 格自证没有一格造过不生效的行，分不出②；库自证把「不生效的行不核」锁成期望值（`.claude/gate.d/lib-lag-baseline.py` 第 1469 行，辩方 N7）；`move_evidence` 里两句「这一行不是一次搬迁」正常运行走不到（第 946 行，辩方 N8）。今天被 decision-links 剔掉的页，两条腿各用不调库的写法核过，只差前缀与切换行（这一半没打中） |

## 三、主 agent 当场改了的

这一批改名本身漏了规则第 5 步的两处（攻方 J1-A 指出一处，主 agent 按「抄下来的命令输出、记那一刻事实的句子留旧路径，可执行的复跑脚本按今天的布局」逐处扫了改名在 `<别的目录>/scripts/experiment-apparatus` 写法上的全部新增行）：

- `research/prompts/d16-item1-r3-main-verification.md` 抄下来的 `diff -u` 头两行改回 `scripts/experiments`，这份文件与 HEAD 相同了。
- `research/prompts/e165-r3-prereg.md` 写当时草稿镜像路径与它 sha256 的那一句，镜像里的路径改回 `scripts/experiments`。
- 复跑脚本里指冻结副本的（`research/prompts/e171-conclusion-r1-opus-model/tools/rerun.sh` 的 `$FROZEN/…`、`e171-conclusion-r2-opus-model/tools/rerun.sh` 的 `$TODAY/…`、`m3-batch10o-code-r1-sonnet-model/rerun.sh`、`m3-close-reuse-closure-r3-sonnet-model/tools/rerun.sh` 的 `$FROZEN/…`）留新路径：那几棵冻结树已经不在，要复跑就得重建，重建出来是今天的布局。

## 四、去向

三轮都打中，每一轮打中的都是上一轮改法的新形状。这一轮是最终轮，不再开第四轮；腿提的收严都只在腿自己的样本上量过或只是推的，被攻过零轮。门禁一侧甲路的主干站得住；打中的①②是放过，③与 N3、N5、N6 是多判（安全一侧）。交用户定。

### 交用户表

| 路 | 做什么 | 代价与风险 |
|---|---|---|
| 甲：三处都收 | 收严 Z（辩方）/ 改法甲（攻方）：换算只认仓根起的前缀（行首、空白、引号、`./`、`$变量/`、`${变量}/`、本仓绝对路径），`<名字>/` 后面的同名段不换；收严 X（辩方）/ 改法戊（攻方）：不在基准那一版登记表里的新行必须此刻生效，否则判红；收严 Y（辩方）：基准那一版登记表里已有的行不再生效（关 N3）；规则第 18 行与库的出路改成「搬一步、等基准前移再登记下一步」（攻方改法丁，关③的出路句）；J2 的句子、调用方清单、按路径管的类加 `.claude/gate.d/`、`.claude/singlefs-ai-sop/rules/`；自证补「有上游、提交没推」「`GATE_BASE=HEAD~1`」两种基准与一格不生效的登记行。派一批工具员，改完不再开轮 | 全部零轮。Z 的代价辩方现数：真仓里 73 处、36 份文件的写法不再换（量的，那一刻），这一批提交时它们照算改动、要人看；X：搬迁推出去之后再补登记会红（推的）。②里的 `chain_old_row`（老行加真搬迁串到早就在的实现）三条都关不住：要关得在钉值一侧另加「一行的新前缀不许落在别的行的旧前缀下」，与旧名字以后合法地重新用起来冲突（辩方，推的），只能写进「门禁管哪一半」 |
| 乙：只收放过的一半 | 只做 X 与 Z（关①与②的新行形状），③、N3、N5、N6 这些多判与 `chain_old_row` 写进规则「门禁管哪一半」与库文件头、登记欠账；J2 的句子、调用方清单、按路径管的类、自证照甲路 | 零轮；比甲少改 Y 与出路句，③在有上游的仓里照旧多红，出路写在「门禁管哪一半」 |
| 丙：代码不动，只写清楚 | 三处与 J2 全部写进规则「门禁管哪一半」、库文件头与 `reuse_library.py` 第 417 行的注释，放过的两处登记 `.claude/kb/checks-owed.md`；这一批照现在的机制提交 | 不派工具员；①②留着：登记表加一行能让钉值核对认换了实现的文件，同名段被剔出改动范围的照旧没人看（这一批里已经发生过一处，主 agent 当场改回了） |

### 用户定案

用户在弹窗里选「甲：三处都收」。改法交批 8 工具员，改完不再开轮；甲路里的每一条都是被攻过零轮的改法。

## 回看决策

不涉及决策：被判的是门禁、钩子、装置里认目录搬迁的读法与写进定义、规则的句子，不碰 `.claude/kb/decisions/` 下任何一条决策的定案与依据。
