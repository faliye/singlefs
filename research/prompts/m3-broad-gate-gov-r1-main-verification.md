# 广谱变异接进提交门禁的治理改动：第一轮判决（2026-10-02）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 C6 E4 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_m3-broad-gate-gov-r1-body.md` | — |
| 正推腿（Sonnet） | `research/prompts/m3-broad-gate-gov-r1-sonnet-output.md`（288 行） | 423f8b66 |
| 攻方腿（Opus） | `research/prompts/m3-broad-gate-gov-r1-opus-output.md`（381 行），模型 `m3-broad-gate-gov-r1-opus-model/` | af110f16 |

- 快照：冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/` 对 `research/prompts/m3-broad-gate-gov-r1-snapshot/tree-sha256.txt` 跑 `sha256sum -c`，退 0。
- `cite-check.py` 两份报告一起跑（`--root` 冻结副本与主仓，带背景材料）：「核了 120 处引文，对上 120 处，没判 0 处」。
- 主 agent 现查：
  - A1 在主仓复跑：`bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item checker-tier-broad-mutation.sh` 退 1，10 份判对，`library-version-is-staged-tree-green` 与 `library-version-is-staged-tree-unstaged-change-green` 两份「期望退出 0，实测 1」（日志 `/tmp/claude-1000/m3-r1/gov-stage-selftest.log`），与攻方 `outputs/stage-selftest-live.txt` 同；
  - A4 / G1-a 的机理：冻结副本 `scripts/admission.py:470` 起 `listed_input_files` 用 `git ls-files --cached --others`，`:688` 起 `manifest_of_files` 读磁盘上的文件内容算 sha256——标记的指纹是工作区，不是暂存区；`.claude/skills/crash-test/commit-stages.md` 第 19 行给这一道免了 `git diff --quiet`；
  - A7：驱动 `scripts/broad-mutation.py:307` 环境型结局（timeout、memory-cap）不在 `DECISIVE_OUTCOMES` 里；用户定案「卡住算抓到」原文在 `research/prompts/_tooling-broad-mutation-prune-r2-body.md:9`：「测试卡住算抓到：按单条测试限时，超时就是那条测试没过。」
- 攻方的数在冻结副本的拷贝或抽出来的纯函数上量，A3、A5、A8 用的是从阶段里抽出来的两段内嵌 python（只核过它在 12 份样本上的退出码与真阶段逐份相同）。两条腿各格的「没打中」都是一次抽样。

## 二、逐条判决

| # | 格 | 两条腿 | 判 |
|---|---|---|---|
| 1 | G5 样本（A1、A2） | 攻方量：12 份样本 2 份判错，两份绿样本判红；病根是样本 `setup.sh` 还按判法版本进指纹之前的公式造库目录指纹，门禁按今天驱动的 `version_key` 复算。只改样本指纹 12 份全对、8 个开关各红在点名那一份；另 `files-unchecked` 让 export-range-red 少一条 want | **打中（主 agent 主仓复跑坐实）**。改样本，门禁判法不改 |
| 2 | G5 没人盯的判红分支（A3） | 攻方量：两格 14 个判红分支删掉 13 个，12 份样本一份不翻面；其中 8 个（pending、version_key、合并冲突、M5、verdict=red 而退 0 时打「判绿」、N5、0 条、N7）删掉就漏判，各造了一份输入 | 打中。8 个分支各补一份红样本、配弄坏开关 |
| 3 | G1 / G2 标记按工作区（A4、G1-a） | 两条腿各自打中同一件事：门禁判暂存区，判绿后写的全绿标记按工作区内容算指纹；之后暂存工作区那份，第 1 步 `stage-marker-check` 与整轮 `--staged` 的复用都认这个标记，一版没判过的源码被复用。攻方在模型上扫五种用户动作：dirty、commit-first、untracked 两处都复用，partial 只第 1 步复用，clean 都不复用。commit-stages 第 19 行的免检不成立 | **打中**。改法：这一道与别的五道同规矩，写标记之前核门禁输入路径上工作区等于暂存区、没有未跟踪文件，不等就不写标记、报「本次未写标记」；删掉第 19 行的免检。零轮。真仓上的复现（攻方「要主 agent 跑的命令」第 1 条）提交前跑 |
| 4 | G1 两格各读一次 tree.source（A5、G1-b） | 两条腿同判：两格之间别的会话 `--sync-tree` 换走库目录，第一格核 A、第二格判绿的是 B 那一版。攻方模型上量过，正推推的；窗口小、概率没量 | 打中（形态）。改法：门禁开头读一次 `tree.source` 与它的指纹，两格都判那一版，判完再读一次、变了判红。零轮 |
| 5 | G1 全局键变了而版本指纹不变（A6、A10、G1 ⑤） | 攻方量：排除表、限时倍数、工具版本变了，全局键跟着变而版本指纹不变，已有的现判结论不重判，`--summary` 照判绿，与 E4「全局键变了就整版重判」不符；复用登记另缺 `run-with-memory-cap.sh`、`capped.sh` 与 llvm-tools 版本（推的）。正推记「规则没说」 | 打中。改法：全局键进版本指纹（或门禁另核全局键），复用登记补三样。零轮 |
| 6 | G1 超时与内存撞顶（A7） | 攻方：timeout、memory-cap 永远算没结论，门禁判红，`--changed-since` 选不中、`--item` 不带 `--rerun` 跳过、下一版不沿用，出路走不出去。用户定案「卡住算抓到」是按单条测试限时，编译步与文档测试步整步超时、内存撞顶都不在定案射程里 | 打中出路是死路；这两类结局怎么记交用户（第四节第 1 条） |
| 7 | G1 两行 total 不互核（A8） | 攻方量：汇总行 total=5091、not_run=5085，判定行 total=6 写 green，门禁判绿 | 打中。改法：门禁核两行的 total、concluded 等各数一致，不一致判红。零轮 |
| 8 | G4 重型闸（A9、G3-4） | 两条腿同判：没找到该拒不拒的；子 agent 跑驱动的每种重型用法都被拒，主 agent 不带前缀被拒。误拒两处：驱动 `--check-ignored`（不起 cargo）、门禁 `--check gate-verdict`；三份文件写「只有 `--list-items`、`--summary` 不算」，钩子还放 `--help`、`-h`、`--selftest`；`lib_heavy_tests.py:264` 还点名已不存在的 `AGENT_KINDS` | 采：两处误拒放行，三份文件的清单与钩子对齐，删掉旧名 |
| 9 | G3 句子（正推 G3-1 至 G3-8） | 正推逐句核：五份文件已无 crash-verifier、gate-triage，「六道」一致；不成立的在 `commit-stages.md`：第 19 行免检（同第 3 行）、第 22 行「五道的命令都带前缀」（同一步是六道）、第 26 行「一趟以小时计，照第 2 步后台跑」（判决推的耗时 10.6–82.0 小时超过后台时限，该走 `detached-run.sh`，推的）、第 27 行「不起 cargo」而 `--tool-versions` 起 `cargo nextest --version`；`agent-common.md:54` 内存包装清单没列驱动；指代落空一处；`main-agent.md:53` 位置在「派发提示怎么写」一节下 | 采，逐句改。驱动文件头 `:108` 旧的第二段基线红（G1-f）一并改 |
| 10 | G1 出路里的「等价变异从清单排除」（G1-d） | 正推：门禁与驱动出路里这句没有对应的排除开关，grep 零命中 | 采：出路改成今天走得通的做法（`broad-mutation-ignored-exclusions.tsv` 管的是忽略用例；等价变异没有排除表就写明没有、记欠账），不新造开关 |
| 11 | 提交没有读 `refs/sop/staged-green` 的闸（G1-e） | 正推：`pre-commit` 只判提交时间窗，钩子与 `settings.json` 里没有读 staged-green 的东西；「跑完才能提交」靠主 agent 守 | 记欠账，不在这一轮改：管的是整套提交流程，不是广谱这一道；记进 `records/2026-09-16-subagent拆分提案.md` 的欠账 |
| 12 | 第二段基线真绿（G1 ④） | 正推：`crash_identity.rs:1518` 登记表不在就 return；基线真绿没有证据，冒烟第三趟用的是 `--skip-stage-two` | 记：提交前第 2b 步真跑时看基线，红了照门禁出路 |

## 三、改法（定下的，派工具实现员，第 5 批下一波）

- 样本指纹改对；8 个漏盯分支各补红样本与弄坏开关（第 1、2 行）。
- 写标记前核工作区等于暂存区，删掉 commit-stages 第 19 行免检（第 3 行）。
- 门禁只读一次 `tree.source`、判完再核（第 4 行）。
- 全局键进版本指纹或门禁另核，复用登记补三样（第 5 行）。
- 两行各数互核（第 7 行）。
- 重型闸两处误拒、清单对齐、删旧名（第 8 行）。
- `commit-stages.md`、`agent-common.md`、`main-agent.md`、驱动文件头逐句改（第 9、10 行）。
- 第 6 行等用户定。
- 改完的形态都是零轮：改了定义与共用约束，下一次治理三方（第二轮）攻这一批改法，或用户逐份豁免。

## 四、交用户

| # | 问题 | 候选 |
|---|---|---|
| 1 | 编译步、文档测试步整步超时与内存撞顶怎么记（用户定案「卡住算抓到」只管单条测试限时） | 甲：整步超时算抓到（与单条测试同理），内存撞顶算无效（不计入，换一条变异补）；乙：两类都算抓到；丙：照今天算没结论、门禁红，补一条出路（`--changed-since` 选中它们重跑，或人工逐条判） |

用户 2026-10-02 弹窗定第 1 条：丙——这两类照旧算没结论、门禁红，补一条走得通的出路（`--changed-since` 选中它们重跑，或人工逐条判）。

## 回看决策

不涉及决策：这一轮判的是门禁、钩子、广谱变异驱动与 agent 定义，不碰文件系统的任何决策分项。
