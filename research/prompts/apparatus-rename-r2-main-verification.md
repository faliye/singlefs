# apparatus-rename-r2 判决（推论轮第二轮，治理轮，2026-10-23）

正文 `research/prompts/_apparatus-rename-r2-body.md`；腿报告 `research/prompts/apparatus-rename-r2-sonnet-output.md`（辩方，复核第一轮判决 `research/prompts/apparatus-rename-r1-main-verification.md`）、`research/prompts/apparatus-rename-r2-opus-output.md`（攻方）。

## 这一轮被判过的文件

`.claude/gate.d/lib-lag-baseline.py`、`.claude/gate.d/path-moves.tsv`、`.claude/hooks/lib_long_run.py`、`scripts/gate/changed-paths.sh`、`.claude/gate.d/doc-decisions.sh`、`.claude/gate.d/doc-experiments.sh`、`.claude/gate.d/code-tooling.sh`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py`、`scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/rules/path-moves.md`、`.claude/long-run-live-hosts-exempt`、`.claude/heavy-thread-exempt`（开工快照 `research/prompts/apparatus-rename-r2-snapshot/judged-sha256.txt`）。

## 一、核查

- 开工快照：写判决之前 `sha256sum -c`，`.claude/gate.d/code-tooling.sh`、`.claude/agent-common.md`、`.claude/main-agent.md` 三份对不上：腿跑着的时候，别的会话的一批工具员在定点改这三份（协调记在调度记录 `records/2026-10-23-实验装置目录改名.md`）。没有倒推快照原样：下一条的引文核对是对着此刻的文件做的，腿引的每一处在此刻的文件里逐字还在，被改的地方没落在腿引的行上。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/apparatus-rename-r2-opus-output.md research/prompts/apparatus-rename-r2-sonnet-output.md --root . --background research/prompts/_apparatus-rename-r2-background.md`：「✓ 核了 49 处引文、26 处裸行号，对上 49 处，没判 26 处（其中裸行号没有引文可比 26 处；逐处列在上面）」。
- 复跑：两条腿的模型目录拷进会话草稿目录，攻方 `build_frozen.sh` 建冻结树，各自 `tools/rerun.sh` 经 `scripts/hosts/run-with-memory-cap.sh 4G` 跑。攻方 `attack_samples.out` `same=yes`，`probe.out` 读主工作区、会变；辩方 `probe_h1`、`probe_rows`、`probe_h3` 都 `same=yes`，`probe_real` 随工作区变。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| H1 修补本身 | **打中**（两条腿一致） | 根子是同一个：登记行提交以后一直生效，几行能串起来。攻方 H1-1：在基准里已有的老登记行前面插一行新行，新行的新前缀在基准里是空的，串上老行之后落到早就存在的另一份实现上，甲照过、调用方与豁免脚本从改动范围里消失、乙放行重钉、丙认两处钉值；这一批提交以后，`scripts/experiments → scripts/experiment-apparatus` 那一行就是这样的老行（推的，这一批还没提交）。辩方：登记行的键是无序的（旧前缀、新前缀）对，换次序再起一个中间目录就把老行重新指到没有实据的新目录。攻方 H1-4：老行一直生效，提交之后一个字节没改的豁免表被判涨。攻方 H1-2（A→B、B→A 来回登记，什么都没搬也满足甲）、H1-3（落点是被 `.gitignore` 忽略、早有另一份实现的目录）。辩方：甲去掉过半之后，把旧目录整个换成另一份实现再登记也满足三条，调用方只差前缀的被剔，被调用方换了实现。攻方 H1-5：戊按目录分类，挑文件的格按扩展名挑，`.md.draft → .md`、`.rs.txt → .rs` 照剔。攻方 H1-6：丁按 hunk 配对，同一个 hunk 里挪到另一节的行照剔（窄）。辩方：戊只修了路径层，跨管辖类单文件搬家在新增行那一层照旧整份剔。乙、丙站得住（辩方），误判方向都是多判 |
| H2 得出结论的方法 | **打中**（两条腿一致） | 「真登记那一行照过」是阳性对照：去掉占用核对、整个实据都不核，那一行照样过（攻方）；「726 份照剔」是同一函数新旧两版互比，不独立（辩方）。自证只覆盖第一轮打中的形状，攻方补上它的改法一、四之后自证照样全绿，分不出这一轮的形状；批 4 自己造过的「换实现」红样本被改成了断言放行（辩方，`lib-lag-baseline.py` 自证那一项）。两条腿各用不调库的独立算法重量「只差搬迁」：攻方被剔的 725 份全在独立集合里，独立算法多出的 5 份是边界写法（多判方向）；辩方独立写法比库多认 7 份，库没有多剔。两条独立算法与库共用「整串替换即相同」的定义，只算佐证 |
| H3 没看过的地方 | **打中** | 九处调用方拿到的基准都被老登记行换过，H1-4 的误判就是从这里来（攻方只量了长活豁免表一处，别的表走同一条代码路，推的）；`scripts/gate/changed-paths.sh` 第 29 行仍写「按次数抵」（攻方）；`.claude/rules/path-moves.md` 第 6 行「不留一处旧路径」与第 15 行「除第 5 步留的之外必须 0 命中」并排矛盾（两条腿一致）；`path-moves.md` 第 13 行「换了实现的不登记」没有门禁判（辩方）；第一轮 G1-3 代码一半「不做」的理由在保守方向上不成立，真仓基准里相关写法只有 9 处（辩方） |

## 三、去向

两轮都打中，而且第二轮打中的是第一轮改法的新形状，根子在「登记行永久生效、能串」这一处设计上。改法是推的，每一条都被攻过零轮；走不走第三轮、按哪一条路走，交用户定（交用户表）。

### 交用户表

| 路 | 做什么 | 代价与风险（推的，没量过） |
|---|---|---|
| 甲路：收窄登记的生效窗口 | 一行登记只在「基准里旧前缀下有文件、新前缀下没有」时生效，提交进基准以后对门禁不再起作用（E171、E170 的钉值核对照旧按它倒换）；同一个改动窗口里一条路径只许搬一次，不许串；落点要是被跟踪的文件；改名的文件只在「新路径恰好等于旧路径照登记换前缀」时剔；戊、丁两层照管辖类与位置收紧；规则句子照改。再开第三轮（最终轮）攻它 | 再派一批工具员与一轮三方；改完机制比现在简单（不再需要「只核新加的行」与串起来的换算），H1-1、H1-4 与辩方的无序键三条从根上没有了；第三轮之后新冒出来的零轮形状只能进交用户表 |
| 乙路：这一次不用登记换算门禁 | 门禁的改动范围、只缩不涨的表、新增行不再按登记表换算；这一次改名照 `.claude/rules/path-moves.md` 原来的手工做法收口：只差路径的文件登进这一批的范围与同步记录，长活豁免表 6 行的重钉由用户逐行认可；E171、E170 装置里的钉值核对仍按登记倒换（它们钉的是搬迁之前的内容，倒换有逐字节的对照）；登记表只给装置用 | 撤回批 1、2c、4 在门禁一侧的大部分改动；这一批的提交要多登记几百份文件、用户认一次豁免表重钉；以后再改目录名照样要手工收口 |

### 用户定案

用户先在弹窗里选乙路；主 agent 随后核出乙路里「豁免表 6 行的重钉由用户逐行认可」没有出路（`.claude/hooks/lib_long_run.py` 第 331、340 行：基准里没有的路径、换了的 sha256 一律判涨），再弹窗说明，用户改选「回到甲路」。改法交批 6，之后开第三轮（最终轮）。

## 回看决策

不涉及决策：被判的是门禁、钩子、装置里认目录搬迁的读法与写进定义、规则的句子，不碰 `.claude/kb/decisions/` 下任何一条决策的定案与依据。
