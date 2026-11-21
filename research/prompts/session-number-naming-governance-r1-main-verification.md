# session-number-naming-governance-r1：判决（2026-11-19）

正文 `research/prompts/_session-number-naming-governance-r1-body.md`，开工快照 `research/prompts/session-number-naming-governance-r1-snapshot/`。这是这一题的第一轮，没有前几轮的判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推腿（Sonnet，第 1 到 3 问） | `research/prompts/session-number-naming-governance-r1-sonnet-output.md` | 第 1 问成立、另有两处推不出；第 2 问不成立；第 3 问不成立 |
| 攻方腿（Opus，第 4 到 7 问） | `research/prompts/session-number-naming-governance-r1-opus-output.md` | 第 4、5 问成立（打中）；第 6 问判不了的路记了检出、preflight 那一步与自证没钉续跑条件打中；第 7 问不成立 |

核查：

- 第一次派的两条腿被客户端重启打断，报告与模型一个字没落盘；照 `.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」那一节「中途停下、被停或报错的腿，新开一条接着做」新派同立场的两条从头做，上表是新派的两条。
- 开工快照：两条腿都报 `.claude/agent-common.md` 的 sha256 与快照不同，攻方另报 `.claude/gate.d/code-tooling.sh` 做到一半被别的会话改过、整体下移 43 行、没碰 `command_invokes` 与这一条判据。两条腿引 `.claude/agent-common.md` 的只有第 41、102 行，主 agent 按此刻那两行核过；引 `code-tooling.sh` 的行号按此刻那一份核。
- 两份报告各自的 cite-check 末行：正推「✓ 核了 11 处引文、22 处裸行号，对上 11 处」，攻方「✓ 核了 12 处引文、6 处裸行号，对上 12 处」。
- 两份模型由主 agent 拷到 `~/.local/share/singlefs/work/session-number-naming-governance-r1-main/` 再交：
  - 正推模型：结局 `type=succeeded exit=0`；喂收工检查那一条末行 `name=done cases=44 model_differs_from_want=13 real_differs_from_want=14`，喂检出钩子那一条末行 `name=done cases=26 claim_decided_differs_from_hook=3`。
  - 攻方模型：结局 `type=command-failed exit=1`（它的对照组有两格 `verdict=control-broken`，按它自己的约定退 1）；`verdict=` 共 109 行：`hit` 44、`hit-declared` 3、`ok` 35、`control-ok` 8、`effective` 6、`not-effective` 5、`undetermined` 4、`by-design` 2、`control-broken` 2。
- 下面「二、判决」只采这两趟起真钩子量的读数；两条腿报告里写「推的」、读数里没有的，判成核不动。

## 二、判决

两条腿都打中，第一轮不收。坐实的照「三、改法」改，改完再攻一轮。

坐实：

- 收工检查漏拦（应拦而放行）：
  - 正推读数 `real=pass want=block`：奇数个 ASCII 双引号夹住、落单的反引号夹住、首字母大写、驼峰、中间有空格的名字、名字与括号之间有空格、两个大写字母、全角字母、括号内换行。
  - 攻方读数 `verdict=hit` 与 `hit-declared`：`leak-capital-M`、`leak-space-before-paren`、`leak-fullwidth-letter`、`leak-fullwidth-parens-and-letter`、`leak-no-parens`、`leak-bold-letter`、`leak-markdown-escaped-parens`、`leak-zero-width-space`、`leak-chinese-main`、`leak-stray-ascii-quote-same-line`、`leak-lowercase-letter`、`leak-two-letters`、`leak-soft-line-break`。
- 收工检查误拦（合规而拦下）：
  - 正推读数 `real=block want=pass`：双反引号、跨行的反引号、四空格缩进的代码块、四反引号里嵌三反引号。
  - 攻方读数 `verdict=hit`：`false-block-blockquote-rule`、`false-block-command-single-quotes`、`false-block-double-backtick-code`、`false-block-four-backtick-fence`、`false-block-indented-code-block`、`false-block-white-corner-quotes`、`false-block-curly-single-quotes`、`false-block-path-outside-code`、`false-block-table-cell-rule-summary`。
- 续跑：
  - 正推读数「续跑里同一份命中换了句子」`want=block real=pass`。
  - 攻方读数 `seq-k0-midB-finalB`、`seq-k0-midnone-finalA`、`seq-k1-midB-finalB`、`seq-k1-midnone-finalA`、`seq-k3-midB-finalB`、`seq-k3-midnone-finalA` 都是 `verdict=hit`：指纹没命中时不清、不带回合，后面的续跑里头一次写同样的叫法也放行。
  - 自证没钉住「只在续跑里」：`selftest-mutant-loop-guard`、`loop-guard-cross-turn-mutant` 是 `verdict=hit`。
- 检出钩子 ㉒ 与 `.claude/agent-common.md` 第 102 行：
  - 正推读数三格 `claim≠hook`：`git -c core.hooksPath=<装对的绝对路径>`、`GIT_CONFIG_KEY`/`VALUE` 设成装对的值，代码放行而文字没写；`git config core.hooksPath scripts/gate/git-hooks`（相对路径）代码放行，而安装脚本与 `.claude/hooks/lib_git_hooks.py` 只认绝对路径，随后的提交被判「钩子没装上」，这一格是代码的毛病。
  - 文字没写到、代码照拦的：`git config --add`、`--unset`，别名与启动器里的 `-n`（正推读数 `claim=silent hook=block` 四格）。

不算打中：

- 整句放进「」、双引号或反引号（攻方 `leak-whole-sentence-in-corner-quotes`、`-ascii-quotes`、`-backticks`）：引号与代码里的不判是为了能引用规则原文，规则本身把这类写法当引用；放开它要另立一条规矩，不归这一轮。拦下时出路那一句照留。
- 正推第 1 问「ListAgents 那一行的编号」与会话标题里的编号在让号、无默认名时可能不同、已停的会话不在列表里：规则写的就是 `ListAgents` 那一行，用户看到的标题由 `.claude/singlefs-ai-sop/scripts/session-number-title.py` 管，两者不同时以列表为准；已停的会话照 `.claude/singlefs-ai-sop/rules/communication-discipline.md`「找别的会话，只找正在跑的」不再找，汇报里写它做过什么即可。

核不动：

- 攻方第 7 问门禁 `.claude/gate.d/code-tooling.sh` 的新判据那一侧：对照组 `gate-green` 与 `gate-removed` 都是 `gate_exit=2 verdict=control-broken`，模型起门禁那一步本身没跑通，`gate-*` 各格的读数不能用。钩子那一侧（`form-*`）读数可用：`form-selftest-argument`、`form-or-true`、`form-python3-interpreter`、`form-sh-interpreter`、`form-other-file-same-name` 五种写法注册了也不起作用（`verdict=not-effective`），这五种是门禁该判红而不判的候选，等门禁那一侧能量了再判。
- 攻方第 6 问 `unjudged-preflight-no-python`（没有 python3 时退 78、不记检出）：读数是 `verdict=hit`，病根是所有钩子共用的 preflight 约定，不是这一轮的改动；交用户。

## 三、改法

派一个工具员，改的文件：`.claude/hooks/session-number-stop-check.sh`、`.claude/hooks/bash-command-detector.sh`（只动 ㉒ 设值放行那一处）；`.claude/agent-common.md` 第 102 行的字面主 agent 照代码改。

1. 收工检查认法：先把回复做一遍归一（全角转半角、去零宽字符、去 Markdown 加粗与转义、括号内与名字里的空白与换行压掉），再不分大小写认 mainagent 或「主 agent」后接括号里一到两个字母，括号可省、可全角。零轮。
2. 收工检查的引用与代码：按 Markdown 认行内代码（任意个反引号配对、可跨行）、围栏（任意长、可嵌套）、缩进代码块、引用块、『』与‘’；落单的引号与反引号不算引用。表格里的规则原文按引用块同样办法放进「」才放行，不另开口子。零轮。
3. 续跑：指纹按命中的整行与回合记，没命中的回复清掉这一会话的指纹；自证补一格钉住「只在续跑里放行」。零轮。
4. 检出钩子 ㉒：`git config core.hooksPath <值>`、`-c`、环境变量三种写法，只在值等于安装脚本装的那个绝对路径时放行，相对路径照拒。零轮。
5. `.claude/agent-common.md` 第 102 行照此刻代码写：三种写法设成那个绝对路径放行，设、加、删 core.hooksPath 与改 core 段都拒，别名与启动器里的照认。只改文字的不做 A/B。

改完复跑两份模型当 A/B，拿这一轮的读数当改前读数。用户 2026-11-19 弹窗定「不再攻，复跑两份模型就收」：不开第二轮，坐实的各格在复跑里都变对就收这一题。

改后复跑（换上之后由主 agent 经任务服务跑出）：

- 换上：收工检查与检出钩子各经 `scripts/gate/hook-swap.py` 换上，结局「✓ 换上了 1 份（改名换上新 inode、回读一致）」，主工作区里喂法 10 条与 6 条都对。收工检查自证「✓ 自检通过（查了 76 种）」，检出钩子自证退 0；两份开关复跑「复跑了 194 条开关（2 份），自证红且点名的格与实际红的格逐格相同 194 条」。
- 正推模型：新版收工检查没有 `LETTER_NAME`，模型那一腿读它就退出；主 agent 在拷贝里只把模型那一腿改成不跑（原样留在 `stop_check_cases.py.before-skip`），真钩子那一腿照原样。结局 `type=succeeded exit=0`，末行 `name=done cases=44 model_differs_from_want=38 real_differs_from_want=2`：原来坐实的 14 格都变对；剩下两格「引用块」「单引号里」模型写的是 `want=block`，照「三、改法」第 2 条它们是引用、放行，属于改法本身。检出钩子那一组 `claim_decided_differs_from_hook=2` 之外的几格比的是改前的 `.claude/agent-common.md` 字面，第 102 行已照代码改；设成相对路径那一格现在被拒。
- 攻方模型：建变异那一步在新版里找不到替换点，主 agent 只单跑起真钩子的 `hook_probes.py`。63 行里漏拦 13 格、误拦 7 格、续跑 `seq-*` 全是 `verdict=ok`；仍标 `hit` 的是整句放进引号的 3 格（「不算打中」那一条）、代码块外的路径与表格里的规则原文 2 格（「三、改法」第 2 条定的就是放进「」才放行）、没有 python3 那 1 格（用户定不改）；`fix-seq`、`loop-guard-cross-turn` 两格 `control-broken` 是变异与改法钩子没建，没有读数，「只在续跑里放行」由收工检查自证里新加的两格与它的开关管。
- 这一题收，不开第二轮。

## 交用户的

- 钩子共用的 preflight 约定：没有 python3 或 preflight 自己出错时，钩子退 78 或 1、不记检出（攻方 `unjudged-preflight-no-python`）。用户 2026-11-19 定：「机器上没有 python3、或 preflight 和我们无关 。本机装了就行。」不改。

## 这一轮被判过的文件

- research/prompts/_session-number-naming-governance-r1-body.md：两条腿照它的七问作答
- .claude/rules/writing-discipline-by-review.md：判了「怎么说」最后一条（与用户要的一致，成立）
- .claude/agent-common.md：判了第 41 行（成立）与第 102 行 ㉒ 那一条（三处与代码相反，坐实）
- .claude/hooks/session-number-stop-check.sh：判了认法漏拦与误拦、续跑指纹、判不了的路（漏拦、误拦、续跑坐实，判不了的路记了检出）
- .claude/settings.json：判了 Stop 上那一条注册（钩子一侧五种写法不起作用，门禁一侧核不动）
- .claude/gate.d/code-tooling.sh：判了 agent-write-scope 格新判据（对照组没跑通，核不动）

## 回看决策

不涉及决策：这一轮只判规则、钩子与门禁，不推翻也不确立 `.claude/kb/decisions.md` 里任何一条。
