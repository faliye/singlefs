# 工具与治理批的零轮形态：第三轮（最终轮）判决（2026-10-13）

<!-- doc-lint:not-numbers Y1 Y2 Y3 Y4 Y5 Y6 V2 V4 V8 X1 X3 X5 L4 L7 M1 M2 M3 M4 M5 M6 M7 N1 N2 N3 N4 N5 N6 F-Y1 F-Y1c F-Y2 F-Y2b F-Y3 -->

正文 `research/prompts/_m3-defs-b6-r3-body.md`；腿：正推 `research/prompts/m3-defs-b6-r3-sonnet-output.md`（sha256 ca1c2dd7…3bff，模型 `research/prompts/m3-defs-b6-r3-sonnet-model/`）、攻方 `research/prompts/m3-defs-b6-r3-opus-output.md`（sha256 d881a562…736d，模型 `research/prompts/m3-defs-b6-r3-opus-model/`）。前两轮判决 `research/prompts/m3-defs-b6-r1-main-verification.md`、`research/prompts/m3-defs-b6-r2-main-verification.md`。这一轮是最终轮，不开第四轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）。

被判的定义与规则文件（按路径）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/verification.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

## 核查记录（主 agent 做）

- 开工快照：在第三轮冻结副本根下 `sha256sum -c research/prompts/m3-defs-b6-r3-snapshot/frozen-copy-sha256.txt` 非 OK 0 行；冻结副本里比正文新的文件 0 份。
- `cite-check.py <两份报告> --root <仓根>`：「✓ 核了 351 处引文，对上 351 处，没判 0 处」（两腿引冻结副本写的是绝对路径）。
- 攻方模型经 `run-with-memory-cap.sh 2G` 复跑：20 份十九 same，`y2-env-b3` 只差长说明截断的位置，判定与退出码相同。正推模型同样复跑（草稿目录须在 `~/.local/share/singlefs/work/` 下）：九份 same；四条旧腿复跑那一份五行（r1-sonnet 11/1、r1-opus 20/1、r2-opus 不设 B3 23/2、设 B3 24/2、r2-sonnet 45/0）与腿存的逐字相同。
- 门禁格主 agent 现跑（`SINGLEFS_HEAVY_TESTS=commit`，第三轮冻结副本的拷贝）：
  - 攻方三组：abandoned-rounds 漂移树第三轮 after 红、零轮之前那一版 after 绿（腿只量了库函数，推两版都红；零轮之前那一版的门禁格不判出处那一行，所以门禁一侧是 L4 共用库带进来的，⑥ 一侧零轮之前就这样）；private-values 两棵符号链接样本绿、不点名链接，读不了的样本退 1、回溯打出 1 处（编的）私有值；heavy-thread-defaults 样本红，不点名 `positional_default.sh`、`env_sed_read.sh`、`env_grep_read.sh`、`lower_name.py`、`proc_environ.py`。
  - 正推第 6 节三格：heavy-thread-defaults 绿，「扫了 832 份……可疑写法 117 处全在排除表里（表 89 行，每行都命中）；装置自己读的线程变量 74 个」；long-run-header-exempt 绿，「6 行」；gate-structure 绿，「查了 17 道阶段、共 113 格」。与腿代跑的数逐个相同。
- 正推 Y5 点到的第三轮副本行逐字在：`stage-owners.tsv` 第 48 行、`investigator.md` 与 `tooling-writer.md` 第 26 行、`doc-text.sh` 第 503、541 行、`code-tooling.sh` 第 2039 行。Y6-34 那句「提交之外要跑先弹窗问用户」在 `.claude/rules/implementation-workflow.md` 里 0 处，出处是 `.claude/hooks/heavy-test-guard.sh`。
- V2-6 补喂：腿只喂了 clerk；主 agent 给 implementation-writer、experiment-runner、tooling-writer 各喂一份只有「预计挂钟」与「线程上限已判：用户让出 CPU，子 agent 起重活用 8 线程」的提示，三份退 2，拒的全是别的缺行（「重型测试：不跑」、定义要的输入），⑮ 一处不拒：带重活的类型只写已判行也过 ⑮。

## 逐格判决

| 格 | 判 | 依据 | 去向 |
|---|---|---|---|
| Y1 ⑮ ⑭ ⑥ 修到的条目 | 站不住（门禁格那一半、M1/M6 带进来的那一半） | 第二轮辩方 195 个标签在第三轮无一回退（攻方，修着）；行首行内三反引号被当围栏开头，遮掉下一行真的「线程上限：16」「多机：…现算…」（攻方 Y1-甲，M1/M6 带进来）；代码块示例行在错位时照判（Y1-乙，M1 没修全）；叫停登记的出处行号在记录里前面插行之后漂移，已叫停的轮重新算开着（Y1-丙：门禁一侧这一批带进来、⑥ 一侧零轮之前就这样，主 agent 跑树坐实） | 改法 N1、N2（零轮） |
| Y2 重型测试闸与 heavy-thread-defaults | 站得住（这一格） | 48 例 env 矩阵、V3 九种与 `lib.rs:8` 第三轮全对（攻方）；两层 `env -C` 合成一层（Y2-甲）、`${1:-8}` 进 `-j` 与 `env \| sed`、小写名、`/proc/self/environ`（Y2-乙）零轮之前也中 | Y2-甲、Y2-乙另立账 |
| Y3 X5 private-values 整串打码 | 站得住（这一格） | 打码本身在三份样本上不漏值（攻方）；符号链接目标带私有值不判（Y3-甲）、读不了的文件让格崩、回溯打出私有值（Y3-乙）零轮之前也中，都不经打码那条路 | Y3-甲、Y3-乙另立账，改法现成（F-Y3，交用户表） |
| Y4 V2、V4 两处文字 | 站得住，补一句文字 | V2 六句五句一致、钩子判不了也不矛盾（正推）；只有「线程上限已判」行时 ⑮ 放、共用约束没写子 agent 起重活取什么数（V2-6，主 agent 补喂三种带重活类型坐实）；V4 十句九句一致，crates 变异整表也起 GPU 测试而两张名单没列（V4-10，每行只起一个测试，并发理由不成立，弱） | 改法 N3（零轮）；V4-10 并进 V4 名单那笔另立账 |
| Y5 清单完整性 | 站不住 | 四条腿 99 个打中，22 个（11 条）两轮判决都没写去向（正推，主 agent 逐行核了出处） | 逐条去向见「Y5 漏列的去向」 |
| Y6 数字能不能核 | 站不住（只在两处措辞） | 98 个数对得上 89；对不上 2：第二轮判决「排除表比第一轮多 2 行」，数据行两边都是 66、变的是 2 个键（Y6-30）；「提交之外要跑先弹窗问用户」不是 `implementation-workflow.md` 第 53 行的原话（Y6-34）；94 / 66 / 58 是批 6 收尾那一刻的数，第三轮副本是 117 / 89 / 74（口径） | 改法 N4 |

## Y5 漏列的去向

| 条 | 打中 | 去向 |
|---|---|---|
| 1 | 「内存上限」定义要而派发闸不判（investigator、tooling-writer） | 另立账：不违反用户定案，缺了只是子 agent 停下问 |
| 2 | 已判行理由钩子不看内容，三种新类型靠它顶缺行 | 并进 N3 |
| 3 | E171 六份入口：提示只能写规范句，入口文件头与编排按现算 | 代码欠账：E171 编排改取登记值之后六份入口出豁免表；之前随 V8 甲另立账 |
| 4 | `stage-owners.tsv` 第 48 行「红了删行或改脚本」与 L7/M7 的钉冲突 | 改法 N5（零轮） |
| 5 | heavy-thread-defaults 四种从未被看过的漏（名里带 heavy_threads 算回退、f 串名、`_LANES`/`_PARALLELISM`、文件中段 `#[cfg(test)] mod tests { }` 之后不扫） | 另立账，与 Y2-乙并一笔（零轮之前也中） |
| 6 | private-values「取不到」分支原样打路径 | 并进 F-Y3 |
| 7 | 未跟踪 1 GiB 单行文件整份读的内存（推的） | 另立账 |
| 8 | 外置测试模块文件 `tests.rs` 两版都点名（攻方自标误报） | 不做：两版同，排除表兜 |
| 9 | `env -C` 的目录写在变量里 | 不做：钩子文件头写明罩不住 |
| 10 | 轮名全集三个小漏、旧行名单与登记表没有检查绑定 | 另立账，与 V4 名单并一笔「手写名单没有检查绑定」 |
| 11 | 第一轮把误伤与漏放合进同一个「站不住」 | 不做：元判断，不是开着的线 |

## 交用户表（全部被攻过零轮）

| 改法 | 改什么 | 修的格 | 量过 / 推的 |
|---|---|---|---|
| M1–M7 | 第二轮判决的七条改法，已在集成副本 `after-b8-fixup` | 第二轮 X1–X7 | 腿与主 agent 在第三轮副本上量过的部分见逐格判决；作为形态没被攻过 |
| N1 | 派发闸认代码块照 CommonMark：围栏行至多缩进 3 格，反引号围栏的语言名里不许有反引号（攻方 F-Y1，`research/prompts/m3-defs-b6-r3-opus-model/patches/runner-dispatch-guard.sh.diff`） | Y1-甲、Y1-乙 | 攻方在自己的模型上量过，195 个标签与第三轮同判（草稿量的） |
| N2 | 叫停登记行抄进出处那一行的原话，判法按「轮名 + 原话」在出处文件里找，行号只当提示（攻方 F-Y1c） | Y1-丙 | 推的，没实现 |
| N3 | 共用约束写明：提示里只有「线程上限已判」行时，子 agent 不起重活，要起就停下交回要「线程上限：N」；或 ⑮ 对带重活的类型不许只写已判行 | V2-6、Y5 第 2 条 | 推的 |
| N4 | 第二轮判决末尾追加更正：「多 2 行」改说「多 2 个键、同时少 2 个，数据行不变」；第 53 行那句改引原话「任务确实要跑时主 agent 先弹窗问用户」；调度记录批 6 那几个数注明是批 6 收尾那一版树 | Y6-30、Y6-34、Y6 口径 | 现查过 |
| N5 | `stage-owners.tsv` 第 48 行改成「红了删行；改脚本要重钉 sha256，按只缩不涨判红，交用户」 | Y5 第 4 条 | 推的 |
| F-Y3 | private-values 先判符号链接（比路径与链接文字，不跟进去）；读文件包 `except OSError`，读不了的列成红行并打码；「取不到」分支经打码（攻方 F-Y3，`…-opus-model/patches/doc-text.sh.diff`） | Y3-甲、Y3-乙、Y5 第 6 条（另立账，零轮之前也中） | 攻方模型上量过；doc-text 样本档没跑；兄弟三格读同一份文件时同样会崩，没改 |

另立一笔账（零轮之前也中，不拿来否这批改动）：Y1-丙 的 ⑥ 一侧；Y2-甲（几层 `env -C` 合一，攻方 F-Y2 现成、量过）；Y2-乙 与 Y5 第 5 条（heavy-thread-defaults 九种漏，攻方 F-Y2b 推的、只罩三种）；Y3-甲、Y3-乙（改法 F-Y3 在交用户表）；Y5 第 1、7 条；V4 名单与旧行名单、轮名全集手写没有检查绑定（含 V4-10）；V8 甲与 E171 编排（随代码欠账）。前两轮与上一题的另立账照旧。攻方顺手记的零轮形态：M6 之后「登记值的一半」挪到「多机：」下一行照放、子 agent 读得到（推的，与 X1-甲同形）；M2 排除表 89 行对第二轮 66 行，拿第二轮当基准时格判涨（提交时基准里没有这张表，不比）。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、项目规则的文字与钩子、门禁格的行为，没有一处引用或改动 `.claude/kb/decisions/` 里的分项。

## 用户定案（2026-10-13）

弹窗选「再开一题三轮」：交用户表里的零轮形态（M1–M7、N1–N5、F-Y3）当新题再攻三轮。先照 N1、N2、N3、N5、F-Y3 改进集成副本（N4 只改判决措辞，主 agent 直接追加），冻结之后开新题第一轮。
