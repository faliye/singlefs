# apparatus-rename-r1 判决（推论轮第一轮，治理轮，2026-10-23）

正文 `research/prompts/_apparatus-rename-r1-body.md`；腿报告 `research/prompts/apparatus-rename-r1-sonnet-output.md`（正推）、`research/prompts/apparatus-rename-r1-opus-output.md`（攻方）。

## 这一轮被判过的文件

`.claude/gate.d/lib-lag-baseline.py`、`.claude/gate.d/path-moves.tsv`、`.claude/hooks/lib_long_run.py`、`scripts/gate/changed-paths.sh`、`.claude/gate.d/doc-decisions.sh`、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/code-tooling.sh`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/fake_multi_host_run.py`、`scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/path-moves.md`（这一批的改动，范围见正文第一节；开工快照 `research/prompts/apparatus-rename-r1-snapshot/judged-sha256.txt`）。

## 一、核查

- 开工快照：写判决之前 `sha256sum -c research/prompts/apparatus-rename-r1-snapshot/judged-sha256.txt`，14 份都对得上，腿跑着的时候没人改过被判的文件。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/apparatus-rename-r1-opus-output.md research/prompts/apparatus-rename-r1-sonnet-output.md --root . --background research/prompts/_apparatus-rename-r1-background.md`：「✓ 核了 56 处引文、39 处裸行号，对上 56 处，没判 39 处（其中裸行号没有引文可比 39 处；逐处列在上面）」。
- 复跑：两条腿的模型目录拷进会话草稿目录，用攻方腿的 `build_frozen.sh` 建冻结树，各自 `tools/rerun.sh` 经 `scripts/hosts/run-with-memory-cap.sh 4G` 跑。攻方腿 8 份里 7 份 `same=yes`，`probe.out` 读的是主工作区（腿已经说明这一份会变）；正推腿 3 份 `same=yes`。
- 主 agent 另外现查：G1-1 的豁免分支，读 `.claude/gate.d/lib-lag-baseline.py` 的 `path_move_problems`，`moved_back` 第三个分支 `move.old.startswith(following.new + "/")` 在；G4-4，`.claude/gate.d/doc-experiments.sh` 第 1650 行 `REPRO_RE='cargo run --release --bin |scripts/(experiments/replay|devices/vm-bench)\.sh'` 在，全仓同类写法（正则交替、分组）只有这一处；G4-5，`records/2026-10-13-里程碑四预开发.md` 第 838 行与 `research/prompts/subagent-landing-audit-2026-10-22/writers.md` 第 316 行抄的是 `git log` 的命令与输出，`git log --all --name-only --format= -- '**/e173_*'` 现跑只列出旧路径。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| G1 误放 | **打中**（两条腿一致） | 登记表只核写法、新前缀在、旧前缀不在、出处在，不核这一行对应一次真搬迁。攻方 G1-1（两行登记靠 `moved_back` 第三个分支让「两个目录都在」通过，把命令从 A 换成 B 的文件剔出改动范围、豁免表重钉不报涨）、G1-2（删掉 A、登记 A→B，B 早就在）复跑相同；正推腿另造出排除表转行、单个词当前缀、复制一行换前缀、两行登记留旧前缀等同一类样本。G1-3（`v0:路径`、`git log -- 路径` 这类指历史的写法被换了也算只差搬迁）、G1-4（`drop_moved_lines` 按次数抵、不按位置抵）复跑相同 |
| G2 读法一致 | **部分打中** | 边界规则与提交引用的跳法几处是同一份（两条腿一致）。不一致的：倒换方向认「只换了一半」而正着换不认（攻方 G2-1）；串起来的两次搬迁、钉在两次之间时 `frozen_o3` 只整串倒（攻方 G2-2，只会多判）；`path_move_problems` 的豁免比它自己的文档宽（攻方 G2-3）；正推腿列的五处出入方向都是多算改动 |
| G3 改动范围剔除的副作用 | **打中** | 攻方 G3-1：用单文件登记把文件搬进 `.claude/agents/`、`crates/<包>/src/`，agent-def-adversarial-review、crates-adversarial-review 挑出 0 份，复跑相同。正推腿另报：拷贝而旧文件还在时只有登记表那一格拦 |
| G4 定义与规则的句子 | **打中** | `.claude/rules/path-moves.md` 写「旧前缀不在」而代码另有豁免；第 2 步只写了两类读法，漏了改动范围与新增行，规则说「整个目录」而表头与代码收单文件；`.claude/agent-common.md`、`.claude/main-agent.md` 那一句没写「按搬迁之前的路径取基准」「这一行的脚本自己搬了家」，登记表加一行不经用户就绕开了「重钉交主 agent 问用户」；第 1 步的搜法打不到拆开写的路径（`REPRO_RE` 已经漏了一处，按格的循环推算误判 83 页）；规则没写指 git 历史与抄命令输出的句子要留旧路径（已有两处成了假话）；两张豁免表的表头仍写「新路径算涨」（正推腿：`.claude/long-run-live-hosts-exempt` 第 7 行、`.claude/heavy-thread-exempt` 第 3 行）。没写进理由与来历（两条腿一致） |

## 三、去向

这一轮打中的都挂起，改法并成一批交 `tooling-writer`（批 4），改完开第二轮攻改法本身。下面每个改法都是被攻过零轮。

| 改法 | 改什么 | 关哪几条 |
|---|---|---|
| 甲 | 登记行要有搬迁实据：这一次新加的登记行（基准那一版登记表里没有的），基准里旧前缀下的每份文件此刻在新前缀下有换算之后相同的一份，新前缀下不许有基准里本来就在的文件；`path_move_problems` 去掉第三个豁免分支 | G1-1、G1-2、G2-3、G4-1 |
| 乙 | `lib_long_run.py` 的 `moved_script_unchanged` 只在这一行的脚本自己搬了家时认搬迁 | G1-1 豁免表那一半、G4-3 |
| 丙 | `frozen_o3.py` 两份与 `reuse_library.py` 倒换认钉值之前，要求此刻的内容正着换一遍不动；`frozen_o3.py` 照 `reuse_library.py` 逐次倒 | G2-1、G2-2 |
| 丁 | `drop_moved_lines` 按 diff 位置配对，只剔与同一处被删的旧行换算之后相同的那一行 | G1-4 |
| 戊 | `moved_only_paths` 对新旧路径落在不同管辖类（`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/`、`.claude/hooks/`、`crates/<包>/src/`）的整份搬家不剔 | G3-1 |
| 己 | `.claude/rules/path-moves.md`：第 1 步的搜法补上正则交替与分组、拆成几段的写法（`"scripts", "experiments"`、`os.path.join(…, "experiments")`）、相对路径（`../experiments`）、rsync 与 glob 的排除写法、把 `/` 换成别的字符拼出的名字（`scripts__experiments__…`）；第 2 步写清登记表管哪几样（只缩不涨的表、钉值、改动范围、新增行）、单文件也可以登记、登记行要有搬迁实据；第 4 步的「0 命中」改成「除第 5 步留的之外 0 命中」；第 5 步照术语改名那一节的判据，写明指 git 历史的写法（`<提交>:路径`、`git log/show/blame … -- 路径`）与抄下来的命令输出留旧路径；「门禁管哪一半」照代码写全 | G1-3、G4-1、G4-2、G4-4、G4-5 |
| 庚 | `.claude/agent-common.md`、`.claude/main-agent.md` 那一句写全条件：按搬迁之前的路径取基准里的脚本、它的 sha256 要是基准那一版表里登记的、这一行的脚本自己搬了家、登记行有搬迁实据；`.claude/long-run-live-hosts-exempt` 与 `.claude/heavy-thread-exempt` 表头照改 | G4-3 |
| 辛 | `.claude/gate.d/doc-experiments.sh` 第 1650 行 `REPRO_RE` 换成新路径（主 agent 写完判决当场改了，repro-command 格现跑绿：「点了产物的 144 页里判了 140 页」） | G4-4 那一处漏改 |

G1-3 的代码一半（跳法扩到 `<任何 rev>:路径` 与 `git log … -- 路径`）不做：正则认全 rev 写法不稳，交给己的规则一半；这一条记进第二轮的攻击面。

主 agent 当场改了的两处（G4-5 那两行假话）：`records/2026-10-13-里程碑四预开发.md` 第 838 行、`research/prompts/subagent-landing-audit-2026-10-22/writers.md` 第 316 行的路径改回 `scripts/experiments/`，两行都是抄的 `git log` 命令与输出。

## 回看决策

不涉及决策：被判的是门禁、钩子、装置里认目录搬迁的读法与写进定义、规则的句子，不碰 `.claude/kb/decisions/` 下任何一条决策的定案与依据。
