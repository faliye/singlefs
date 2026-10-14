# 工具与治理批的零轮形态（第二题）：第三轮正文（2026-10-14）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 Z6 Z7 P1 P2 P3 P4 P5 P6 P7 S1 S2 S3 S4 S5 S6 S7 U1 U2 U3 U4 Q1 Q2 Q3 Q4 Q5 Q6 -->

## 一、这一轮要判什么

同一题的第三轮，也是最终轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：第三轮之后不开第四轮）。第一轮判决 `research/prompts/m3-defs-b9-r1-main-verification.md`（改法 P1–P7），第二轮判决 `research/prompts/m3-defs-b9-r2-main-verification.md`（改法 U1–U4）；两轮改法都已落进冻结副本（批 10、批 10 收尾、批 11，调度记录 `records/2026-10-12-里程碑三工具与治理批.md`）。

第三轮分工：「已定」项撑不撑得住 / 未定项与另立账清单的完整性 / 两轮报出的数字能不能核。**只攻前两轮站住的形态**：第二轮判「站得住」的 S2（P2 缺行出路两种写法、clerk 改名）、S4（P4 仓副本改过正文的自证按提交时的检查办）、S6（P6 doc-text 打码），以及第二轮辩方判「修到」的条目（`research/prompts/m3-defs-b9-r2-sonnet-output.md`「总表」）。第二轮改法 U1–U4 是被攻过零轮的新形态，这一轮不为它们开攻，判决里列进交用户表标「零轮」；腿顺手看到 U 改法本身的问题可以记一行，主 agent 只记不判。

被判的定义与规则文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/implementation-workflow.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b9-r3/repo/`（集成副本标签 `after-b11`，没有 `.git`）；这一题动过的 35 份文件的 sha256 在 `research/prompts/m3-defs-b9-r3-snapshot/frozen-copy-sha256.txt`。前两轮冻结副本在 `~/.local/share/singlefs/work/m3-defs-b9-r1/repo/`（`after-b9-fixup`，P 改法之前）、`~/.local/share/singlefs/work/m3-defs-b9-r2/repo/`（`after-b10-fixup`）。腿只读这三份冻结副本取被判字面；做法照主工作区的共用约束。冻结副本根下有本机私有配置 `multi-host.env`：报告、模型目录与回复里一个值都不许写出来。

| 格 | 判什么 | 对象 |
|---|---|---|
| Q1 | S2：⑮ 缺行时的两种出路（要别的数写「线程上限：N」加已判行；单独的已判行只给确实不起重活的活）与 `dispatch-prompt.py` 的 clerk 改名，在第三轮冻结副本里撑不撑得住；有没有可达工作流让「默认 31、固定不看忙闲」实际落空而规则让人以为没有 | `.claude/hooks/runner-dispatch-guard.sh` ⑮、`scripts/agents/dispatch-prompt.py`、`.claude/main-agent.md`、`.claude/agent-common.md`、各定义「输入」 |
| Q2 | S4：仓副本里正文改过的登记为 light 的自证按提交时的检查办、主 agent 在集成副本里带 `SINGLEFS_HEAVY_TESTS=commit` 跑、不弹窗；与「重型提交之外先弹窗」「仓副本自证收回放宽」两条撑不撑得住 | `.claude/rules/implementation-workflow.md`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh` |
| Q3 | S6：doc-text 各格打印路径与片段经打码（date-once、term-renames 转出的 sweep-term 输出、clock-times、prime-marks、session-names 片段挪到私有值边上） | `.claude/gate.d/doc-text.sh` |
| Q4 | 第二轮辩方判「修到」的条目（第一轮两腿打中的 Z1 围栏四形、Z4 三种日常改动、Z5 两形、Z7 文字），在第三轮冻结副本里（批 11 动过之后）还修着没有 | 同第一轮判决逐格判决表的「在哪」一列 |
| Q5 | 未定项与另立账清单完整性：这一题两轮判决的「另立一笔账」与改法表，加四条腿报告里打中的每一条，合起来是不是全部开着的线；有没有一条打中两轮都没写去向 | 两轮判决、四条腿报告 |
| Q6 | 两轮报出的数字能不能核：两轮判决、正文与调度记录里这一题引的计数（例：verify13 223 项判错 5、verify14 244 项判错 7、三道样本档 46 / 22 / 27 例、扫 832 份、排除表 89 行、各样本点名的份数、cite-check 的引文数），按引的命令或产物现算，对不对得上、口径写全没有 | 两轮判决、正文、调度记录、`~/.local/share/singlefs/work/m3-tooling-b2-integration/verify13.summary`、`verify14.summary` 与 `verify13-out/`、`verify14-out/`、`~/.local/share/singlefs/work/m3-defs-b9-r2-main-out/log/` |

## 二、实现今天的样子（主 agent 的观测，2026-10-14 现查）

- `crates/` 这一题不改。
- 集成副本复核：批 11 打上之后 `~/.local/share/singlefs/work/m3-tooling-b2-integration/verify14.summary`「派 244 项、收 244 项、判错 7 项」：`brk-long-run-exempt-*` 四项是复核脚本直接设库开关被门禁格覆盖（同 verify12）；stage-owners 缺 session-names 一行与 doc-text 在集成真树上 clock-times、term-renames 两格红，批 10 之前同样红；fixture-claims 格红在集成副本的 git 历史从 2026-10-12 起（122 个文件名、964 处正文的日期早于第一个提交）与 checker-tier-layer0-replay 缺红样本，批 11 加的样本一份都不在红项里。两棵漂移小树只记不判（原话改了字那棵退 1，原话出现两次那棵退 0）。
- 这一题前两轮与上一题三轮的另立账不在 Q1–Q4 射程里，不再交，除非 U1–U4 让它变得更糟；Q5 要把它们一并列进清单核完整性。

## 三、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 逐条核「两轮判决与正文说的数和清单，就是产物与冻结副本里的样子」 | Q5、Q6 | Q6 每个数一行：引处（文件:行）、现算命令、现算结果、对不对得上、口径缺什么；Q5 每条开着的线一行：出处、在不在两轮的另立账 / 改法表里，漏列的给出出处与打中的历史 |
| **云端攻方（Opus）** | 造工作流历史攻前两轮站住的形态 | Q1、Q2、Q3、Q4 | 每条历史写成可复现的步骤，在冻结副本拷贝里真跑钩子与脚本的判定；交「照规则做、结果错」的那一步与错在哪；前两轮与上一题腿报告里有的历史不再交，除非改后又中 |

## 四、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 照留，这一题收 | 两条腿在那一格都给不出照定义做却违反用户定案、或定义说拒而钩子不拒的可达历史；Q6 每个数都对得上 |
| 一格判「站不住」⇒ 不开第四轮，改法写进判决的交用户表、标「零轮」，由用户定 | 给出那样一条可达历史；Q6 有数对不上 |
| 打中也打中 P 改法之前那一版 ⇒ 另立一笔账 | 同一段历史在 `~/.local/share/singlefs/work/m3-defs-b9-r1/repo/` 上给出同样的错 |
| Q5 漏列 ⇒ 补进判决的另立账或交用户表 | 一条开着的线两轮判决都没写去向 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 五、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b9-r3-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b9-r3-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b9-r3-<腿名>-model/`（含 `tools/rerun.sh`，从 `scripts/agents/three-way-rerun-template.sh` 拷来、只换 `rerun_steps` 那一段；复跑经 `scripts/hosts/run-with-memory-cap.sh` 时上限给 2G；要的环境变量写进 rerun.sh 的默认值；`--work` 的草稿目录要在 `~/.local/share/singlefs/work/` 底下；输出里的绝对路径先换占位再截断）。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数；引主工作区 `research/prompts/` 下的判决与产物写仓内相对路径。
- 不跑登记的 `--selftest`，核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数；门禁格（`--check`）子 agent 起不了，读格的代码造出小树与命令写进模型目录，标「要主 agent 跑」；造的小树要齐。
- 新写的文字不用「层 0」「第一段 / 第二段」「第一条流 / 第二条流」这几种旧名（`.claude/kb/term-renames.md`）。

### 小节清单：`.claude/kb/term-renames.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 术语改名对照表 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （术语改名对照表 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 超级块 → 系统配置（2026-09-20 起，英文标识符与冻结目录 2026-09-21） | 抄 | 默认全抄（脚本生成） |
| ## 按里程碑起的名字 → 按内容起的名字（2026-09-27 起） | 抄 | 默认全抄（脚本生成） |
| ## 层 0 → 崩溃点重放（2026-10-12 起） | 抄 | 默认全抄（脚本生成） |
| ## 第一条流 / 第二条流、两段流水线、三层表的档 → 按内容起的名字（2026-10-13 起） | 抄 | 默认全抄（脚本生成） |

**出处 `.claude/kb/term-renames.md:1-8`（整段抄，未转述）**

```markdown
# 术语改名对照表

全仓改过名的概念，旧名与新名的唯一权威对照。**别处一律用新名**——`scripts/kb/sweep-term.py --check` 扫全仓，
除 `.claude/term-rename-exempt` 登记的几处之外出现旧名就判红。

这一份自己登记在那张豁免表里：写「X 改名成 Y」这句话必须写得出 X，而**旧名要有一处查得到，历史产物与旧提交里的字段才对得上**。
要查某个旧名的现名，只看这一份；要查某次改名当天改了哪些文件、为什么改，看决策变更史当天那一条。

```

**出处 `.claude/kb/term-renames.md:9-13`（整段抄，未转述）**

```markdown
## 超级块 → 系统配置（2026-09-20 起，英文标识符与冻结目录 2026-09-21）

用户定案：盘上那块 481 字节的结构不叫「超级块」——本工程的索引树是普通的树，丢了能重建，没有「超级」的东西；
那块结构是**系统配置**。2026-09-21 定案：英文标识符、文件名与冻结目录一并改，「直到全工程都搜不出来」。

```

**出处 `.claude/kb/term-renames.md:14-21`（整段抄，未转述）**

```markdown
## 按里程碑起的名字 → 按内容起的名字（2026-09-27 起）

用户定：测试、模块、装置、门禁、kb 文档不按里程碑起名（「里程碑重命名我觉得是个糟糕的决定 这个应该修改了」「全都改了」），
按内容起：第一条写流是在刚 mkfs 的空池上新建一个文件，叫「新池新建文件」（`new_pool_file_creation`）；第二条写流是同一个文件的覆盖写（`file_overwrite`）；
里程碑二做的是覆盖写、释放、回退与复用；里程碑三当天定名崩溃验证重做，现名「删除与多次 COW」（对照表那一行写了改名经过）。里程碑本身的中文名一并改（用户同日选「一起改」）。
`.claude/kb/milestone/` 与 `.claude/kb/layout/` 下按里程碑编号的文档名（`01-first-txn.md` 这类）不改（用户同日定「命名是对的」），所以连字符缩写只按具体用途登记；
跟着改的文件名、门禁名与测试文件名，对照在 `records/2026-09-27-验证两档拆分.md`「第二批」一节。

```

**出处 `.claude/kb/term-renames.md:22-26`（整段抄，未转述）**

```markdown
## 层 0 → 崩溃点重放（2026-10-12 起）

用户 2026-10-11 指出「层 0」改名成「崩溃点重放」只换了一部分，2026-10-12 定全仓照这套换：「层 0 崩溃点重放」叠用的去掉「层 0」；「层 0 全量 / 快档」写「崩溃点重放全量 / 快档」；「层 0 用例」写「崩溃枚举用例」；英文标识符（`layer0_*`、门禁名 `checker-tier-layer0-replay`）当时没动，2026-10-13 那一批改了，登在文末「第一条流 / 第二条流、两段流水线、三层表的档 → 按内容起的名字（2026-10-13 起）」一节。
不换的三类按 `.claude/term-rename-exempt` 登记：D13（验证路线） 已定项 9 分层表里与层 1、层 2 并列的「层 0」档（正则不打「层 0 冒烟」，别的写法按行登记）；实验装置、留存产物、变异名与门禁样本（整份）；代码里的字符串与原样引的旧文。

```

**出处 `.claude/kb/term-renames.md:27-339`（整段抄，未转述）**

```markdown
## 第一条流 / 第二条流、两段流水线、三层表的档 → 按内容起的名字（2026-10-13 起）

用户 2026-10-13 定：汇报与文档里按序数起的名字（「第一条流」「第二条流」「第一段」「第二段」「层 0」「层 1」「层 2」）看不出指什么，全仓换成说内容的名字，批量替换豁免三方；弹窗选「按提议的名字改」，并说「比如 我们之前跑的是第一个事务的新建 cow 现在跑第二个事务的多次 cow 我觉得这样的描述比较清楚简单」「当然这个命名你可以根据实际情况来调整」。
崩溃点重放的两条负载：第一条流 → 新池新建文件流，第二条流 → 覆盖写固定脚本流（那条脚本是覆盖写、释放、重开写行、暖机、回退、抬 F、复用各一次，到 E）。
崩溃放量两段流水线：第一段 → 录入段（流先入库），第二段 → 判定段（从库里读回判，在 GPU 或 CPU 上）。这一对不登记表行、门禁不判：同一个词在实验登记的分段、执行员按段交回、段序列、KV 的实现分段里都是别的量，2026-10-13 的 clerk 在 kb 与记录里逐行判过 314 行、真指流水线的 0 行，两条按语境写的正则打中的 19 处全是误打；靶子只在脚本的注释与输出文字里（scripts/crash 三份已换），以后写的人照新名写。
D13（验证路线） 已定项 9 三层表的三档：层 0 冒烟 → 冒烟层，层 1 常规 → 常规层，层 2 全量 → 大负载层；散落的「层 0」仍按 2026-10-12 那一节换成崩溃点重放。英文标识符（layer0_enumeration、checker-tier-layer0-replay、record 这类）这一轮不动。
2026-10-13 英文标识符改名那一批（`records/2026-10-13-复用键与全仓改名.md#3`，依赖表第 8 行）把上一句说不动的英文标识符也改了，用户原话「函数名不得以 layer0 或者类似的名字命名，必须要能看出功能和作用」：crates、脚本与门禁、文档三路改完，旧名逐个登在下面表里（裸词 `layer0`、`LAYER0`、`Layer0` 不登；实验装置、留存产物与原样引文里的照原样留，按 `.claude/term-rename-exempt` 登记）。这一批的行都写成正则：名字照字面起头，前后不紧挨 ASCII 字母、数字与下划线（名字带连字符的，连字符也算词里的字），判法同词边界，中文字紧挨着也打。不写成 `\b` 起头，是因为 `\b` 起头的行让全仓扫一遍慢得多：E178（叶容器与记录死亡的计数） 产物 `e178-leaf-and-record-death-counts-segment-two.out` 的前 2 000 000 字上，原有 56 行 0.19 秒，加这 234 行照字面起头 0.71 秒，写成词边界 2.96 秒。

<!-- term-renames:table -->
一行一对，`scripts/kb/sweep-term.py` 按这张表换、按这张表查；**加一行就自动受门禁管**。
「匹配」写 `整串`、`词边界` 或 `正则`：`词边界` 用在缩写上，免得打中别的词里的那几个字母；同一个串在别处是另一个量时写 `正则`，「旧」一列写一条 Python 正则，只打这一个概念的写法（例 `(?<![每第内外里叶二])(?<![0-9] )层 1(?![0-9])` 打「层 1 常规」，不打「每层 134」「第 1 层 2 个」「外层 16.33 ms」），竖线写成 `\|`，不用捕获组，「新」一列照字面换上，写法的全部限制在 `scripts/kb/sweep-term.py` 文件头。长的写在前面，免得短的先把长的吃掉一半。

| 旧 | 新 | 匹配 | 是什么 |
|---|---|---|---|
| 超级块 | 系统配置 | 整串 | 中文术语 |
| SUPER_BLOCK | SYSTEM_CONFIGURATION | 整串 | 常量名，带下划线 |
| SUPERBLOCK | SYSTEM_CONFIGURATION | 整串 | 常量名 |
| Super_Block | SystemConfiguration | 整串 | 大驼峰带下划线 |
| SuperBlock | SystemConfiguration | 整串 | 大驼峰，类型名 |
| Superblock | SystemConfiguration | 整串 | 大驼峰，b 小写的写法 |
| superBlock | systemConfiguration | 整串 | 小驼峰 |
| super_block | system_configuration | 整串 | 蛇形带下划线 |
| super-block | system-configuration | 整串 | 连字符形态（二进制名、产物文件名） |
| superblock | system_configuration | 整串 | 蛇形标识符、模块名、文件名 |
| sb_ | system_configuration_ | 词边界 | 缩写前缀（`sb_mac`） |
| _sb | _system_configuration | 词边界 | 缩写后缀（`tail_sb`） |
| sb | system_configuration | 词边界 | 裸缩写。`sb` 本是为 superblock 设的，概念没了缩写也去掉，同日从 SOP 的缩写表里删行 |
| harness 耗时用例 | harness 慢档用例 | 整串 | harness 档里 debug 下单条跑到 60 秒及以上、标 `#[ignore]` 的用例；用户 2026-10-01 定 harness 档分「快档 / 慢档」，与上游 test-script-discipline.md「分快慢两档」同名 |
| 耗时用例 | 慢档用例 | 整串 | 「harness 耗时用例」不带「harness」前缀的短称 |
| 轻用例 | 快档用例 | 整串 | harness 档里不标 `#[ignore]`、每次改完就跑的用例；与「慢档用例」同日改名 |
| harness 重档 | harness 慢档用例 | 整串 | harness 档里单线程跑满 60 秒的用例；「重档」与「重型测试」只差一个字、指的却是两件事，改名分开 |
| 里程碑三（名字待定） | 里程碑「删除与多次 COW」 | 整串 | 里程碑名。先定名「崩溃验证重做」，提交 4e8b0370（2026-09-29）起崩溃验证那部分并进里程碑「覆盖写、释放、回退与复用」增补 4，里程碑三改叫「删除与多次 COW」 |
| 第二个事务 | 覆盖写、释放、回退与复用 | 整串 | 里程碑名，也是那条写流的称呼 |
| 第一个事务 | 新池新建文件 | 整串 | 里程碑名，也是那条写流的称呼 |
| FIRST_TRANSACTION | NEW_POOL_FILE_CREATION | 整串 | 常量名 |
| FirstTransaction | NewPoolFileCreation | 整串 | 大驼峰，类型名与枚举成员 |
| first_transaction | new_pool_file_creation | 整串 | 蛇形标识符、模块名、文件名 |
| first-transaction | new-pool-file-creation | 整串 | 连字符形态（运行模式名、阶段文件名） |
| SECOND_TRANSACTION | FILE_OVERWRITE | 整串 | 常量名 |
| SecondTransaction | FileOverwrite | 整串 | 大驼峰，枚举成员 |
| second_transaction | file_overwrite | 整串 | 蛇形标识符、指标名 |
| second-transaction | file-overwrite | 整串 | 连字符形态（运行模式名） |
| first_txn | new_pool_file_creation | 整串 | 缩写形态的蛇形 |
| e142-first-txn-dry-run | e142-new-pool-file-creation-dry-run | 整串 | 产物文件名前缀 |
| 32-first-txn-fields | 32-new-pool-file-creation-fields | 整串 | 阶段文件名 |
| 42-first-txn-trio | 42-new-pool-file-creation-trio | 整串 | 阶段文件名 |
| 76-second-txn-hooks | 76-overwrite-release-rollback-reuse-hooks | 整串 | 阶段文件名 |
| first-txn-journal-implies-units | new-pool-file-creation-journal-implies-units | 整串 | litmus 文件名 |
| first-txn-root-implies-units | new-pool-file-creation-root-implies-units | 整串 | litmus 文件名 |
| formatted-layer0-same-as-first-txn | formatted-layer0-same-as-new-pool-file-creation | 整串 | 标签 |
| formatted-layer0-first-txn-reference | formatted-layer0-new-pool-file-creation-reference | 整串 | 标签 |
| second_txn | overwrite_release_rollback_reuse | 整串 | 缩写形态的蛇形 |
| third_txn | crash_verification_rebuild | 整串 | 缩写形态的蛇形 |
| 按条记录 | 判定库记录 | 整串 | crates 变异表那一份判定库里的记录 |
| 判法指纹 | 判法版本 | 整串 | crates 变异表判定库目录的那个摘要 |
| 底座指纹 | 判法版本 | 整串 | 「判法指纹」更早的叫法 |
| judging-fingerprint | judge-version | 整串 | `crates-mutation-rows.py` 的子命令 |
| judging_fingerprint | judge_version | 整串 | 函数名与记录字段 |
| lib_item_verdicts | lib_verdict_library | 整串 | 文件改名（属路径搬迁，`.claude/rules/path-moves.md`「怎么做」），按术语一并查旧名 |
| GATE_FORMAL_VERDICT_STORE | GATE_FORMAL_VERDICT_LIBRARY | 整串 | 形式化门禁的环境变量 |
| store-in-tmpdir | library-in-tmpdir | 整串 | `FORMAL_VERIFICATION_BREAK` 的一项 |
| 快速档 | 快档 | 整串 | E165（恢复重放有界穷举与变异杀伤界） 那一档；与「harness 慢档用例」同属快慢两档的叫法 |
| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )层 ?0(?![0-9.])(?! ?冒烟)` | 崩溃点重放 | 正则 | 中文术语；不打「第 1 层 0」「每层 0」这类树层与 D13（验证路线） 已定项 9 的「层 0 冒烟」档 |
| 第一条流 | 新池新建文件流 | 整串 | 中文术语：崩溃点重放的第一条负载 |
| 第二条流 | 覆盖写固定脚本流 | 整串 | 中文术语：崩溃点重放的第二条负载（固定脚本到 E） |
| 层 0 冒烟 | 冒烟层 | 整串 | D13（验证路线） 已定项 9 三层表第一档的标签 |
| 层 1 常规 | 常规层 | 整串 | 同表第二档的标签 |
| 层 2 全量 | 大负载层 | 整串 | 同表第三档的标签 |
| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )层 ?1(?![0-9.])(?! ?常规)` | 常规层 | 正则 | 中文术语：三层表第二档单独写时；不打「第 1 层 1」「每层 1」这类树层，带「常规」的由上一行换 |
| `(?<![第每各叶外内深一两三几多0-9])(?<!第 [0-9] )(?<!第 [0-9][0-9] )层 ?2(?![0-9.])(?! ?全量)` | 大负载层 | 正则 | 中文术语：三层表第三档单独写时；不打树层，带「全量」的由上一行换 |
| `checker-tier-layer0-replay(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]checker-tier-layer0-replay)` | checker-tier-crash-point-replay | 正则 | 门禁名、阶段文件名与样本目录名 |
| `layer0-shard-configuration-check(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shard-configuration-check)` | crash-point-replay-shard-configuration-check | 正则 | 两台分片配置判法的脚本名 |
| `layer0-shard-run(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shard-run)` | crash-point-replay-shard-run | 正则 | 崩溃点重放双机分片包装的脚本名（已删的 `layer0-shard-run-selftest.sh` 不打，它没有新名） |
| `layer0-sample-tools(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-sample-tools)` | crash-point-replay-sample-tools | 正则 | 门禁样本根里放假工具的目录名（`.layer0-sample-tools`） |
| `singlefs-layer0-logs(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]singlefs-layer0-logs)` | singlefs-crash-point-replay-logs | 正则 | 门禁在 common-dir 里的日志目录名 |
| `singlefs-layer0-shard(?![A-Za-z0-9_])(?<![A-Za-z0-9_]singlefs-layer0-shard)` | singlefs-crash-point-replay-shard | 正则 | 双机分片包装的临时目录前缀 |
| src/layer0_library.rs | src/crash_library.rs | 整串 | `Cargo.toml` 注释里引的判定库源文件 |
| `layer0_enumeration(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumeration)` | crash_state_enumeration | 正则 | 模块名、`.rs` 文件名、判定库类名与清单种类（`CrashLibraryClass::name`、`MANIFEST_KIND`） |
| `layer0_progress(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_progress)` | crash_point_replay_progress | 正则 | 模块名与 `.rs` 文件名 |
| `layer0-enumeration-library(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-enumeration-library)` | crash-state-enumeration-library | 正则 | 判定库里崩溃枚举用例那一类的子目录名 |
| `layer0-replay(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-replay)` | crash-case-replay | 正则 | 崩溃点重放门禁的格名（门禁名那一行在前，先换掉长的） |
| `LAYER0_CELL_NAME(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_CELL_NAME)` | CRASH_CASE_REPLAY_CELL_NAME | 正则 | 阶段里的格名常量 |
| `LAYER0_CELL_TITLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_CELL_TITLE)` | CRASH_CASE_REPLAY_CELL_TITLE | 正则 | 阶段里的格标题常量 |
| `LAYER0_FEATURE_CELL_NAME(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FEATURE_CELL_NAME)` | FEATURE_GATED_TESTS_CELL_NAME | 正则 | 阶段里 feature-gated-tests 格的格名常量 |
| `LAYER0_FEATURE_CELL_TITLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FEATURE_CELL_TITLE)` | FEATURE_GATED_TESTS_CELL_TITLE | 正则 | 同上，格标题 |
| `layer0-stage(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-stage)` | crash-point-replay-stage | 正则 | 重型测试闸里的种类名（`lib_heavy_tests.py` 的 STAGE_KIND） |
| `stage_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]stage_layer0)` | stage_crash_point_replay | 正则 | 重型测试闸自检里的阶段名 |
| `layer0_admission_module(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_admission_module)` | crash_point_replay_admission_module | 正则 | 阶段变量 |
| `layer0_break(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_break)` | crash_point_replay_break | 正则 | 阶段变量 |
| `layer0_cell_state(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_cell_state)` | crash_point_replay_cell_state | 正则 | 阶段变量 |
| `layer0_check_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_check_cells)` | crash_point_replay_check_cells | 正则 | 阶段变量 |
| `layer0_found(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_found)` | crash_point_replay_found | 正则 | 阶段变量 |
| `layer0_full_base(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_full_base)` | crash_point_replay_full_base | 正则 | 阶段变量 |
| `layer0_full_rc(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_full_rc)` | crash_point_replay_full_rc | 正则 | 阶段变量 |
| `layer0_input_paths_text(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_input_paths_text)` | crash_point_replay_input_paths_text | 正则 | 阶段变量 |
| `layer0_input_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_input_table)` | crash_point_replay_input_table | 正则 | 阶段变量 |
| `layer0_judge_paths(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_judge_paths)` | crash_point_replay_judge_paths | 正则 | 阶段变量 |
| `layer0_kept_rows(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_kept_rows)` | crash_point_replay_kept_rows | 正则 | 阶段变量 |
| `layer0_list_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_list_cells)` | crash_point_replay_list_cells | 正则 | 阶段变量 |
| `layer0_list_items(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_list_items)` | crash_point_replay_list_items | 正则 | 阶段变量 |
| `layer0_list_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_list_table)` | crash_point_replay_list_table | 正则 | 阶段变量 |
| `layer0_log_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_log_directory)` | crash_point_replay_log_directory | 正则 | 阶段变量 |
| `layer0_named_cases(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_named_cases)` | crash_point_replay_named_cases | 正则 | 阶段变量 |
| `layer0_named_case(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_named_case)` | crash_point_replay_named_case | 正则 | 阶段变量 |
| `layer0_named_value(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_named_value)` | crash_point_replay_named_value | 正则 | 阶段变量 |
| `layer0_registered_input_paths(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_registered_input_paths)` | crash_point_replay_registered_input_paths | 正则 | 阶段变量 |
| `layer0_registered_paths_text(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_registered_paths_text)` | crash_point_replay_registered_paths_text | 正则 | 阶段变量 |
| `layer0_report_not_run(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_report_not_run)` | crash_point_replay_report_not_run | 正则 | 阶段变量 |
| `layer0_resolved_cargo(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_resolved_cargo)` | crash_point_replay_resolved_cargo | 正则 | 阶段变量 |
| `layer0_runs_cell(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_runs_cell)` | crash_point_replay_runs_cell | 正则 | 阶段变量 |
| `layer0_sample_mode(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_mode)` | crash_point_replay_sample_mode | 正则 | 阶段变量 |
| `layer0_sample_tool_mode(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool_mode)` | crash_point_replay_sample_tool_mode | 正则 | 阶段变量 |
| `layer0_sample_tool_path(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool_path)` | crash_point_replay_sample_tool_path | 正则 | 阶段变量 |
| `layer0_sample_tool_problems(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool_problems)` | crash_point_replay_sample_tool_problems | 正则 | 阶段变量 |
| `layer0_sample_tools_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tools_directory)` | crash_point_replay_sample_tools_directory | 正则 | 阶段变量 |
| `layer0_sample_tool(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sample_tool)` | crash_point_replay_sample_tool | 正则 | 阶段变量 |
| `layer0_scratch_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_scratch_directory)` | crash_point_replay_scratch_directory | 正则 | 阶段变量 |
| `layer0_selected_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_selected_cells)` | crash_point_replay_selected_cells | 正则 | 阶段变量 |
| `layer0_shard_check_script_directory(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shard_check_script_directory)` | crash_point_replay_shard_check_script_directory | 正则 | 阶段变量 |
| `layer0_shard_configuration_note(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shard_configuration_note)` | crash_point_replay_shard_configuration_note | 正则 | 阶段变量 |
| `layer0_shard_driver(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shard_driver)` | crash_point_replay_shard_driver | 正则 | 阶段变量 |
| `layer0_sharded(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_sharded)` | crash_point_replay_sharded | 正则 | 阶段变量 |
| `layer0_stage_file_name(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_stage_file_name)` | crash_point_replay_stage_file_name | 正则 | 阶段变量 |
| `layer0_stage_repository(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_stage_repository)` | crash_point_replay_stage_repository | 正则 | 阶段变量 |
| `layer0_stage_script_path(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_stage_script_path)` | crash_point_replay_stage_script_path | 正则 | 阶段变量 |
| `layer0_start_over(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_start_over)` | crash_point_replay_start_over | 正则 | 阶段变量 |
| `layer0_static_exit(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_static_exit)` | crash_point_replay_static_exit | 正则 | 阶段变量 |
| `layer0_static_judge(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_static_judge)` | crash_point_replay_static_judge | 正则 | 阶段变量 |
| `layer0_tier(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tier)` | crash_point_replay_tier | 正则 | 阶段变量 |
| `layer0_tree_ready(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tree_ready)` | crash_point_replay_tree_ready | 正则 | 阶段变量 |
| `layer0_unknown_cases(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_unknown_cases)` | crash_point_replay_unknown_cases | 正则 | 阶段变量 |
| `layer0_unlisted_input_paths(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_unlisted_input_paths)` | crash_point_replay_unlisted_input_paths | 正则 | 阶段变量 |
| `Layer0DigitOfASegment(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0DigitOfASegment)` | CrashPointReplayDigitOfASegment | 正则 | 类型名 |
| `Layer0EnumerationTables(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0EnumerationTables)` | CrashStateEnumerationTables | 正则 | 类型名 |
| `Layer0Enumeration(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Enumeration)` | CrashStateEnumeration | 正则 | 类型名 |
| `Layer0FindingsLogBegin(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingsLogBegin)` | CrashPointReplayFindingsLogBegin | 正则 | 类型名 |
| `Layer0FindingsLogSection(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingsLogSection)` | CrashPointReplayFindingsLogSection | 正则 | 类型名 |
| `Layer0FindingsLog(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingsLog)` | CrashPointReplayFindingsLog | 正则 | 类型名 |
| `Layer0FindingSample(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingSample)` | CrashPointReplayFindingSample | 正则 | 类型名 |
| `Layer0FindingSignature(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0FindingSignature)` | CrashPointReplayFindingSignature | 正则 | 类型名 |
| `Layer0Findings(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Findings)` | CrashPointReplayFindings | 正则 | 类型名 |
| `Layer0Finding(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Finding)` | CrashPointReplayFinding | 正则 | 类型名 |
| `Layer0Item(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Item)` | CrashPointReplayItem | 正则 | 类型名 |
| `Layer0JudgeRoute(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0JudgeRoute)` | CrashPointReplayJudgeRoute | 正则 | 类型名 |
| `Layer0ObserverCounts(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0ObserverCounts)` | CrashPointReplayObserverCounts | 正则 | 类型名 |
| `Layer0Observer(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Observer)` | CrashPointReplayObserver | 正则 | 类型名 |
| `Layer0OraclesOfThePieces(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0OraclesOfThePieces)` | CrashPointReplayOraclesOfThePieces | 正则 | 类型名 |
| `Layer0Oracles(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Oracles)` | CrashPointReplayOracles | 正则 | 类型名与枚举成员（E171 判件 `JudgingPiece`） |
| `Layer0OracleViolationKind(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0OracleViolationKind)` | CrashPointReplayOracleViolationKind | 正则 | 枚举名 |
| `Layer0OracleViolation(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0OracleViolation)` | CrashPointReplayOracleViolation | 正则 | 类型名 |
| `Layer0PublishOfState(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0PublishOfState)` | CrashPointReplayPublishOfState | 正则 | 类型名 |
| `Layer0RedPass(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0RedPass)` | CrashPointReplayRedPass | 正则 | 枚举名 |
| `Layer0SegmentExpansion(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0SegmentExpansion)` | CrashPointReplaySegmentExpansion | 正则 | 类型名 |
| `Layer0SegmentOfState(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0SegmentOfState)` | CrashPointReplaySegmentOfState | 正则 | 类型名 |
| `Layer0SegmentSlice(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0SegmentSlice)` | CrashPointReplaySegmentSlice | 正则 | 类型名 |
| `Layer0StateJudgement(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StateJudgement)` | CrashStateJudgement | 正则 | 类型名 |
| `Layer0StateObserving(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StateObserving)` | CrashStateObserving | 正则 | 类型名 |
| `Layer0StatePlan(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StatePlan)` | CrashStatePlan | 正则 | 类型名 |
| `Layer0StatePosition(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0StatePosition)` | CrashStatePosition | 正则 | 类型名 |
| `Layer0Tally(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0Tally)` | CrashPointReplayTally | 正则 | 类型名 |
| `Layer0ToolchainIdentity(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0ToolchainIdentity)` | CrashPointReplayToolchainIdentity | 正则 | 类型名 |
| `Layer0VersionAtTheRoot(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0VersionAtTheRoot)` | CrashPointReplayVersionAtTheRoot | 正则 | 类型名 |
| `Layer0VersionTable(?![A-Za-z0-9_])(?<![A-Za-z0-9_]Layer0VersionTable)` | CrashPointReplayVersionTable | 正则 | 类型名 |
| `enumerate_layer0_quick_tier_versions(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_quick_tier_versions)` | enumerate_crash_states_quick_tier_versions | 正则 | 函数名 |
| `enumerate_layer0_selecting_versions(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_selecting_versions)` | enumerate_crash_states_selecting_versions | 正则 | 函数名 |
| `enumerate_layer0_selecting(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_selecting)` | enumerate_crash_states_selecting | 正则 | 函数名 |
| `enumerate_layer0_through_the_library(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_through_the_library)` | enumerate_crash_states_through_the_library | 正则 | 函数名 |
| `enumerate_layer0_versions(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_versions)` | enumerate_crash_states_versions | 正则 | 函数名 |
| `enumerate_layer0_with_expansion(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0_with_expansion)` | enumerate_crash_states_with_expansion | 正则 | 函数名 |
| `enumerate_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]enumerate_layer0)` | enumerate_crash_states | 正则 | 函数名；也是 `crash-case-check.py` 认用例的那条调用 |
| `judge_layer0_state_range_of_the_read_back_flow_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range_of_the_read_back_flow_with_slices)` | judge_crash_state_range_of_the_read_back_flow_with_slices | 正则 | 函数名 |
| `judge_layer0_state_range_of_the_read_back_flow(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range_of_the_read_back_flow)` | judge_crash_state_range_of_the_read_back_flow | 正则 | 函数名 |
| `judge_layer0_state_range_against_the_version_table_naming_the_publish(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range_against_the_version_table_naming_the_publish)` | judge_crash_state_range_against_the_version_table_naming_the_publish | 正则 | 函数名（局部闭包） |
| `judge_layer0_state_range(?![A-Za-z0-9_])(?<![A-Za-z0-9_]judge_layer0_state_range)` | judge_crash_state_range | 正则 | 函数名 |
| `layer0_enumeration_tables_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumeration_tables_with_slices)` | crash_state_enumeration_tables_with_slices | 正则 | 函数名 |
| `layer0_enumeration_tables(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumeration_tables)` | crash_state_enumeration_tables | 正则 | 函数名 |
| `layer0_judge_route(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_judge_route)` | crash_point_replay_judge_route | 正则 | 函数名 |
| `layer0_ordinal_with_the_same_landing(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_ordinal_with_the_same_landing)` | crash_state_ordinal_with_the_same_landing | 正则 | 函数名 |
| `layer0_plan_state_count(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_plan_state_count)` | crash_point_replay_plan_state_count | 正则 | 函数名 |
| `layer0_state_count_with_the_torn_third_state(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_count_with_the_torn_third_state)` | crash_state_count_with_the_torn_third_state | 正则 | 函数名 |
| `layer0_state_count_with_torn_in_place_overwrites(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_count_with_torn_in_place_overwrites)` | crash_state_count_with_torn_in_place_overwrites | 正则 | 函数名 |
| `layer0_state_count(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_count)` | crash_state_count | 正则 | 函数名 |
| `layer0_state_ranges_by_segment_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_ranges_by_segment_with_slices)` | crash_state_ranges_by_segment_with_slices | 正则 | 函数名 |
| `layer0_state_ranges_by_segment(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_state_ranges_by_segment)` | crash_state_ranges_by_segment | 正则 | 函数名 |
| `layer0_tally_from_the_library(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tally_from_the_library)` | crash_point_replay_tally_from_the_library | 正则 | 函数名 |
| `layer0_tally_under_the_expansion_of_the_plan(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_tally_under_the_expansion_of_the_plan)` | crash_point_replay_tally_under_the_expansion_of_the_plan | 正则 | 函数名 |
| `layer0_worker_threads(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_worker_threads)` | crash_point_replay_worker_threads | 正则 | 函数名 |
| `sample_layer0_states_on_the_cpu_with_threads_against_the_version_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0_states_on_the_cpu_with_threads_against_the_version_table)` | sample_crash_states_on_the_cpu_with_threads_against_the_version_table | 正则 | 函数名 |
| `sample_layer0_states_on_the_cpu_against_the_version_table(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0_states_on_the_cpu_against_the_version_table)` | sample_crash_states_on_the_cpu_against_the_version_table | 正则 | 函数名 |
| `sample_layer0_states_on_the_cpu(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0_states_on_the_cpu)` | sample_crash_states_on_the_cpu | 正则 | 函数名 |
| `visit_layer0_state_range_of_the_read_back_flow_with_slices(?![A-Za-z0-9_])(?<![A-Za-z0-9_]visit_layer0_state_range_of_the_read_back_flow_with_slices)` | visit_crash_state_range_of_the_read_back_flow_with_slices | 正则 | 函数名 |
| `visit_layer0_state_range(?![A-Za-z0-9_])(?<![A-Za-z0-9_]visit_layer0_state_range)` | visit_crash_state_range | 正则 | 函数名（局部闭包） |
| `synthetic_layer0_log(?![A-Za-z0-9_])(?<![A-Za-z0-9_]synthetic_layer0_log)` | synthetic_crash_point_replay_log | 正则 | `admission.py` 自证里合成日志的函数 |
| `run_layer0_stage_shard_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]run_layer0_stage_shard_cells)` | run_crash_point_replay_stage_shard_cells | 正则 | `admission.py` 的函数 |
| `run_layer0_stage_cells(?![A-Za-z0-9_])(?<![A-Za-z0-9_]run_layer0_stage_cells)` | run_crash_point_replay_stage_cells | 正则 | `admission.py` 的函数 |
| `layer0_lines(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_lines)` | crash_point_replay_lines | 正则 | `admission.py` 的局部名 |
| `LAYER0_FINDING_SAMPLES_KEPT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDING_SAMPLES_KEPT)` | CRASH_POINT_REPLAY_FINDING_SAMPLES_KEPT | 正则 | 常量名 |
| `LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE)` | CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE | 正则 | 常量名 |
| `LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE)` | CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE | 正则 | 常量名 |
| `LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE)` | CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE | 正则 | 常量名 |
| `LAYER0_RED_PASS_POOL_CHECKER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_POOL_CHECKER)` | CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER | 正则 | 常量名 |
| `LAYER0_RED_PASS_RECORD_CHECKER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_RED_PASS_RECORD_CHECKER)` | CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER | 正则 | 常量名 |
| `LAYER0_SAMPLING_ITEM(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SAMPLING_ITEM)` | CRASH_POINT_REPLAY_SAMPLING_ITEM | 正则 | 常量名 |
| `LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE)` | CRASH_POINT_REPLAY_WORKER_THREADS_ENVIRONMENT_VARIABLE | 正则 | 常量名 |
| `READ_FAULTS_OF_A_LAYER0_STATE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]READ_FAULTS_OF_A_LAYER0_STATE)` | READ_FAULTS_OF_A_CRASH_STATE | 正则 | 常量名 |
| `LAYER0_STAGE_CASES(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_STAGE_CASES)` | CRASH_POINT_REPLAY_STAGE_CASES | 正则 | `admission.py` 的常量 |
| `LAYER0_PARALLEL_FINISHED_PREFIX(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_FINISHED_PREFIX)` | CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX | 正则 | `admission.py` 的常量 |
| `LAYER0_THREADS_VARIABLE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_THREADS_VARIABLE)` | CRASH_POINT_REPLAY_THREADS_VARIABLE | 正则 | `admission.py` 的常量 |
| `LAYER0_START_OVER_LIBRARY_DIRECTORY_NAME(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_START_OVER_LIBRARY_DIRECTORY_NAME)` | CRASH_POINT_REPLAY_START_OVER_LIBRARY_DIRECTORY_NAME | 正则 | `admission.py` 的常量 |
| `layer0-start-over(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-start-over)` | crash-point-replay-start-over | 正则 | 上一常量的值（重开那一类库的目录名） |
| `LAYER0_SHARD_CONFIGURATION_KEYS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_CONFIGURATION_KEYS)` | CRASH_POINT_REPLAY_SHARD_CONFIGURATION_KEYS | 正则 | 两台分片配置判法的常量 |
| `SINGLEFS_LAYER0_THREADS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_THREADS)` | SINGLEFS_CRASH_POINT_REPLAY_THREADS | 正则 | 环境变量 |
| `SINGLEFS_LAYER0_FINDINGS_FILE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_FINDINGS_FILE)` | SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE | 正则 | 环境变量 |
| `SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE)` | SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE | 正则 | 环境变量 |
| `SINGLEFS_LAYER0_SHARD_PEER_EXIT_PAUSE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]SINGLEFS_LAYER0_SHARD_PEER_EXIT_PAUSE)` | SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_EXIT_PAUSE | 正则 | 环境变量（双机包装自证） |
| `LAYER0_SHARD_CONFIGURATION_FILE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_CONFIGURATION_FILE)` | CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE | 正则 | 配置判法 `--emit-assignments` 打出的变量 |
| `GATE_LAYER0_BREAK(?![A-Za-z0-9_])(?<![A-Za-z0-9_]GATE_LAYER0_BREAK)` | GATE_CRASH_POINT_REPLAY_BREAK | 正则 | 门禁弄坏开关 |
| `LAYER0_SHARD_RUN_BREAK(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_RUN_BREAK)` | CRASH_POINT_REPLAY_SHARD_RUN_BREAK | 正则 | 双机包装弄坏开关 |
| `LAYER0_SHARD_RUN_TWO_HOSTS_DRIVER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SHARD_RUN_TWO_HOSTS_DRIVER)` | CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER | 正则 | 双机包装的驱动变量 |
| `STUB_LAYER0_REUSED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]STUB_LAYER0_REUSED)` | STUB_CRASH_POINT_REPLAY_REUSED | 正则 | 门禁样本假 cargo 的桩变量 |
| `STUB_LAYER0_EXIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]STUB_LAYER0_EXIT)` | STUB_CRASH_POINT_REPLAY_EXIT | 正则 | 门禁样本假 cargo 的桩变量 |
| `LAYER0_ACQUISITION_BARRIER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_ACQUISITION_BARRIER)` | CRASH_POINT_REPLAY_ACQUISITION_BARRIER | 正则 | 计数行前缀 |
| `LAYER0B_FAST(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0B_FAST)` | CRASH_POINT_REPLAY_FIXED_SCRIPT_FAST | 正则 | 覆盖写固定脚本流快档的计数行前缀（B 是序数代称，连前缀一起换） |
| `LAYER0B(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0B)` | CRASH_POINT_REPLAY_FIXED_SCRIPT | 正则 | 覆盖写固定脚本流的计数行前缀 |
| `LAYER0_FINDINGS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDINGS)` | CRASH_POINT_REPLAY_FINDINGS | 正则 | 发现日志的汇总标记 |
| `LAYER0_FINDING(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FINDING)` | CRASH_POINT_REPLAY_FINDING | 正则 | 发现日志的逐条标记 |
| `layer0_finding_threshold(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_finding_threshold)` | crash_point_replay_finding_threshold | 正则 | 发现日志的行种类 |
| `layer0_finding_new(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_finding_new)` | crash_point_replay_finding_new | 正则 | 发现日志的行种类 |
| `layer0_findings_begin(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_findings_begin)` | crash_point_replay_findings_begin | 正则 | 发现日志的行种类 |
| `layer0_findings_summary(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_findings_summary)` | crash_point_replay_findings_summary | 正则 | 发现日志的行种类 |
| `layer0_finding(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_finding)` | crash_point_replay_finding | 正则 | 发现日志的行种类 |
| `LAYER0_FLOW_RECORDED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_FLOW_RECORDED)` | CRASH_POINT_REPLAY_FLOW_RECORDED | 正则 | 日志标记 |
| `LAYER0_IO_MIN_8192_MOUNT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_IO_MIN_8192_MOUNT)` | CRASH_POINT_REPLAY_IO_MIN_8192_MOUNT | 正则 | 计数行前缀 |
| `LAYER0_JUDGE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_JUDGE)` | CRASH_POINT_REPLAY_JUDGE | 正则 | 日志标记 |
| `LAYER0_PARALLEL_FINISHED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_FINISHED)` | CRASH_POINT_REPLAY_PARALLEL_FINISHED | 正则 | 日志标记 |
| `LAYER0_PARALLEL_LINE_ONE_FAST(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_LINE_ONE_FAST)` | CRASH_POINT_REPLAY_MULTI_RECORD_PUBLISH_FAST | 正则 | 多记录发布流快档的计数行前缀（「并行线一」是序数代称，连前缀一起换） |
| `LAYER0_PARALLEL_LINE_ONE(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_LINE_ONE)` | CRASH_POINT_REPLAY_MULTI_RECORD_PUBLISH | 正则 | 多记录发布流的计数行前缀 |
| `LAYER0_PARALLEL_START(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PARALLEL_START)` | CRASH_POINT_REPLAY_PARALLEL_START | 正则 | 日志标记 |
| `LAYER0_POSITION_ADDRESSED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_POSITION_ADDRESSED)` | CRASH_POINT_REPLAY_POSITION_ADDRESSED | 正则 | 计数行前缀 |
| `LAYER0_PROGRESS(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_PROGRESS)` | CRASH_POINT_REPLAY_PROGRESS | 正则 | 进度行标记 |
| `LAYER0_SPILL_OVER(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_SPILL_OVER)` | CRASH_POINT_REPLAY_SPILL_OVER | 正则 | 计数行前缀 |
| `LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_ACCOUNTING_LEAF_SPLIT | 正则 | 计数行前缀 |
| `LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_ACCOUNTING_ROOT_SPLIT | 正则 | 计数行前缀 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_EMPTY_LEAF_DROPPED | 正则 | 计数行前缀 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_LEAF_SPLIT | 正则 | 计数行前缀 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_ROOT_LOWERED | 正则 | 计数行前缀 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_ROOT_SPLIT | 正则 | 计数行前缀 |
| `LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW)` | CRASH_POINT_REPLAY_TREE_SPLIT_CENTRAL_MAPPING_TWO_LEVELS_SPLIT_IN_A_ROW | 正则 | 计数行前缀 |
| `LAYER0_TREE_SPLIT(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_TREE_SPLIT)` | CRASH_POINT_REPLAY_TREE_SPLIT | 正则 | 计数行前缀 |
| `LAYER0_UNLINK_AFTER_MULTI_RECORD_FAST(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_UNLINK_AFTER_MULTI_RECORD_FAST)` | CRASH_POINT_REPLAY_UNLINK_AFTER_MULTI_RECORD_FAST | 正则 | 计数行前缀 |
| `LAYER0_UNLINK_AFTER_MULTI_RECORD(?![A-Za-z0-9_])(?<![A-Za-z0-9_]LAYER0_UNLINK_AFTER_MULTI_RECORD)` | CRASH_POINT_REPLAY_UNLINK_AFTER_MULTI_RECORD | 正则 | 计数行前缀 |
| `layer0-unlink-after-the-multi-record-publish-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-unlink-after-the-multi-record-publish-stream)` | unlink-after-the-multi-record-publish-stream | 正则 | crash-case 键（`crash-case:` 后面那一段，不带前缀写的同一个用例名一样） |
| `layer0-multi-record-publish-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-multi-record-publish-stream)` | multi-record-publish-stream | 正则 | crash-case 键 |
| `layer0-position-addressed-tree-streams(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-position-addressed-tree-streams)` | position-addressed-tree-streams | 正则 | crash-case 键 |
| `layer0-tree-split-streams(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-tree-split-streams)` | tree-split-streams | 正则 | crash-case 键 |
| `layer0-first-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-first-stream)` | new-pool-file-creation-stream | 正则 | crash-case 键：新池新建文件流 |
| `layer0-second-stream(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-second-stream)` | overwrite-fixed-script-stream | 正则 | crash-case 键：覆盖写固定脚本流 |
| `a_recording_recovery_that_differs_from_the_layer0_report_is_counted(?![A-Za-z0-9_])(?<![A-Za-z0-9_]a_recording_recovery_that_differs_from_the_layer0_report_is_counted)` | a_recording_recovery_that_differs_from_the_crash_point_replay_report_is_counted | 正则 | 测试函数名 |
| `the_state_ordinal_decodes_to_the_same_persisted_set_as_layer0_mixed_radix(?![A-Za-z0-9_])(?<![A-Za-z0-9_]the_state_ordinal_decodes_to_the_same_persisted_set_as_layer0_mixed_radix)` | the_state_ordinal_decodes_to_the_same_persisted_set_as_crash_point_replay_mixed_radix | 正则 | 测试函数名 |
| `worker_threads_come_from_the_layer0_variable_then_the_checking_threads_variable_then_the_machine(?![A-Za-z0-9_])(?<![A-Za-z0-9_]worker_threads_come_from_the_layer0_variable_then_the_checking_threads_variable_then_the_machine)` | worker_threads_come_from_the_crash_point_replay_variable_then_the_checking_threads_variable_then_the_machine | 正则 | 测试函数名 |
| `a_layer0_item_name_cannot_be_a_path_segment_with_a_slash(?![A-Za-z0-9_])(?<![A-Za-z0-9_]a_layer0_item_name_cannot_be_a_path_segment_with_a_slash)` | a_crash_point_replay_item_name_cannot_be_a_path_segment_with_a_slash | 正则 | 测试函数名 |
| `layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations)` | crash_point_replay_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations | 正则 | 测试函数名（也是 `stage-inputs.tsv` 的 `test=` 列） |
| `layer0_quick_tier_matches_the_full_tally_shape(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_quick_tier_matches_the_full_tally_shape)` | crash_point_replay_quick_tier_matches_the_full_tally_shape | 正则 | 测试函数名 |
| `the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_layer0_tally(?![A-Za-z0-9_])(?<![A-Za-z0-9_]the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_layer0_tally)` | the_tail_of_c_is_keyed_apart_in_the_two_streams_and_every_block_counts_its_states_by_publish_as_the_crash_point_replay_tally | 正则 | 测试函数名 |
| `the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape(?![A-Za-z0-9_])(?<![A-Za-z0-9_]the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_layer0_shape)` | the_sigma_enumeration_prints_exhaustive_true_and_a_thread_line_of_the_crash_point_replay_shape | 正则 | 测试函数名 |
| `acquisition-barrier-layer0(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]acquisition-barrier-layer0)` | acquisition-barrier-crash-point-replay | 正则 | 池名 |
| `formatted-layer0-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-fast)` | formatted-crash-point-replay-fast | 正则 | 池名 |
| `formatted-layer0-new-pool-file-creation-reference(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-new-pool-file-creation-reference)` | formatted-crash-point-replay-new-pool-file-creation-reference | 正则 | 标签（上面 `formatted-layer0-first-txn-reference` 那一行的现名） |
| `formatted-layer0-registered(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-registered)` | formatted-crash-point-replay-registered | 正则 | 池名 |
| `formatted-layer0-same-as-new-pool-file-creation(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]formatted-layer0-same-as-new-pool-file-creation)` | formatted-crash-point-replay-same-as-new-pool-file-creation | 正则 | 标签（上面 `formatted-layer0-same-as-first-txn` 那一行的现名） |
| `parallel-line-one-layer0-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]parallel-line-one-layer0-fast)` | multi-record-publish-crash-point-replay-fast | 正则 | 池名（「并行线一」是序数代称，换成说流的名字） |
| `parallel-line-one-layer0-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]parallel-line-one-layer0-full)` | multi-record-publish-crash-point-replay-full | 正则 | 池名 |
| `unlink-after-multi-record-layer0-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]unlink-after-multi-record-layer0-fast)` | unlink-after-multi-record-crash-point-replay-fast | 正则 | 池名 |
| `unlink-after-multi-record-layer0-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]unlink-after-multi-record-layer0-full)` | unlink-after-multi-record-crash-point-replay-full | 正则 | 池名 |
| `unlink-after-multi-record-layer0-rerun(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]unlink-after-multi-record-layer0-rerun)` | unlink-after-multi-record-crash-point-replay-rerun | 正则 | 池名 |
| `spill-over-layer0(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]spill-over-layer0)` | spill-over-crash-point-replay | 正则 | 池名 |
| `layer0-b-controls(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-b-controls)` | crash-point-replay-b-controls | 正则 | 池名 |
| `layer0-c-registered(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-c-registered)` | crash-point-replay-c-registered | 正则 | 池名 |
| `layer0-d-registered(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-d-registered)` | crash-point-replay-d-registered | 正则 | 池名 |
| `layer0-e-fast(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-e-fast)` | crash-point-replay-e-fast | 正则 | 池名 |
| `layer0-e-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-e-full)` | crash-point-replay-e-full | 正则 | 池名 |
| `layer0-reuse-that-never-landed(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-reuse-that-never-landed)` | crash-point-replay-reuse-that-never-landed | 正则 | 池名 |
| `layer0-residual-record(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-residual-record)` | crash-point-replay-residual-record | 正则 | 池名 |
| `layer0-stale-tail(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-stale-tail)` | crash-point-replay-stale-tail | 正则 | 池名 |
| `layer0-threaded-sampling(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-threaded-sampling)` | crash-point-replay-threaded-sampling | 正则 | 池名 |
| `layer0-partial(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-partial)` | crash-point-replay-partial | 正则 | 池名 |
| `layer0-control(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-control)` | crash-point-replay-control | 正则 | 池名 |
| `layer0-full(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-full)` | crash-point-replay-full | 正则 | 池名；连字符也算词里的字，不打 `parallel-line-one-layer0-full`（上面单登）与门禁认的旧标记文件名 `singlefs-layer0-full-green` |
| `caught_with_the_layer0_threads_value(?![A-Za-z0-9_])(?<![A-Za-z0-9_]caught_with_the_layer0_threads_value)` | caught_with_the_crash_point_replay_threads_value | 正则 | 局部变量 |
| `layer0_threads_value(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_threads_value)` | crash_point_replay_threads_value | 正则 | 局部变量 |
| `layer0_quick_tier_matches(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_quick_tier_matches)` | crash_point_replay_quick_tier_matches | 正则 | 局部变量 |
| `sample-layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample-layer0)` | sample-crash-point-replay | 正则 | 门禁样本名（`-empty`、`-quick-tier-only`、`-forged`、`-noexec`、`-bare` 这些后缀一起换） |
| `sample_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]sample_layer0)` | sample_crash_point_replay | 正则 | 重型测试闸自检里的样本名 |
| `layer0-by-name(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-by-name)` | crash-point-replay-by-name | 正则 | `prove-red.sh` 的弄坏开关 |
| `named_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]named_layer0)` | named_crash_point_replay | 正则 | `prove-red.sh` 里的名字 |
| `demo_layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]demo_layer0)` | demo_crash_point_replay | 正则 | `prove-red.sh` 的样本目标 |
| `some_layer0_stream(?![A-Za-z0-9_])(?<![A-Za-z0-9_]some_layer0_stream)` | some_crash_point_replay_stream | 正则 | 重型测试闸自检的样本二进制名 |
| `layer0-sample(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0-sample)` | crash-point-replay-sample | 正则 | 样本假工具的版本串（`0.0.0-layer0-sample`；目录名那一行在前） |
| `layer0-demo(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0-demo)` | crash-point-replay-demo | 正则 | 双机包装自证的标签前缀 |
| `layer0-shaped-case-records-and-imports(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shaped-case-records-and-imports)` | crash-case-shaped-records-and-imports | 正则 | 两份崩溃放量脚本的自证格名 |
| `library-layer0-shape(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]library-layer0-shape)` | library-crash-case-shape | 正则 | 两份崩溃放量脚本自证里的库目录名 |
| `layer0-shape(?![A-Za-z0-9_-])(?<![A-Za-z0-9_-]layer0-shape)` | crash-case-shape | 正则 | 两份崩溃放量脚本自证里的样本文件名 |
| `layer0_shape(?![A-Za-z0-9_])(?<![A-Za-z0-9_]layer0_shape)` | crash_case_shape | 正则 | 两份崩溃放量脚本自证里的变量名 |
| `STUB_CASE_SHAPE=layer0(?![A-Za-z0-9_])(?<![A-Za-z0-9_]STUB_CASE_SHAPE=layer0)` | STUB_CASE_SHAPE=crash-case | 正则 | 两份崩溃放量脚本自证里假用例的形态 |

**跟着改的文件名**：`crates/singlefs-core/src/superblock.rs` → `system_configuration.rs`；五个实验装置、五张变异表、五个留存产物
（E100（系统配置的三段几何）、E115（系统配置字段表的完备性与字节预算）、E124（系统配置字段表按 D18 已定项 14 重算）、E126（系统配置槽宽四条候选的代价）、E147（系统配置字段表按新池新建文件字节表重算））。搬迁怎么做见 [.claude/rules/path-moves.md](../rules/path-moves.md)「怎么做」那一节。

**没改的**：`SUPERBLOCK_MAGIC` 的**值** `*b"SFSB"` 不动——那是写进盘上的魔数，改它等于改磁盘格式。常量名跟着改了。

**改名前的原件**：在 git 里。中文那一批的前一个状态见提交 `495bede` 之前；英文标识符与冻结目录那一批见 `4105e70` 之后的工作区改动。
留存产物与源码是同一次一起改的，所以 `replay.sh` 的逐字节比对改名前后都成立——这一条由改完的全量复跑证明，不是声称。

```
