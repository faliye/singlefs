# 给用户的汇报写会话编号：治理文件轮第一轮（session-number-naming-governance-r1，2026-11-19）

用户 2026-11-19 原话「mainagent 给我汇报的时候 要告诉我 协作对话的编号」「现在说的是 mainagent(A) 我不知道是和谁对接的」。主 agent 改了两处规则，又派工具员把它做成收工时会红的检查，都已换上。这一轮攻这一批改动。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

1. 改动做的是不是用户要的事：给用户的汇报里别的会话写编号，进仓文字与会话之间的消息照旧（正推腿）。
2. 改动哪一格会错：检查漏拦、误拦，规则两处互相打架或与别的规则打架（攻方腿）。

## 实现今天的样子

主 agent 的观测（读的是主工作区此刻的文件）：

- 这一轮没有改 `crates/` 下的代码，改动也不读 `crates/` 下的任何文件。
- 规则：
  - `.claude/rules/writing-discipline-by-review.md`「怎么说」最后一条改成：给用户的回复与汇报里指别的会话，写它在 `ListAgents` 那一行的编号（例「(5b)」），连同它改了哪个文件、做了什么，不写 mainagent 加字母；会话之间的消息照旧写 mainagent 加字母；子 agent 照旧写 subagent 加字母。
  - `.claude/agent-common.md`「写」一节那一句末尾改成「会话之间的消息照同一条；给用户的回复与汇报里别的会话写它在 `ListAgents` 那一行的编号」。
- 检查：新钩子 `.claude/hooks/session-number-stop-check.sh`，注册在 `.claude/settings.json` 的 Stop 上，登记在 `.claude/hooks/selftest-registry.tsv`。
  - 判法：去掉代码块、反引号、「」、双引号里的内容后，逐行找 mainagent 后接半角或全角括号里一个大写字母，有就退 2 并给出路；SubagentStop 或输入带 `agent_id` 的放行；续跑里命中与上一次一字不差的放行；判不了的退 0 并记一条检出。
  - 读法复用上游 `.claude/singlefs-ai-sop/scripts/claude-hooks/courtesy-phrase.py` 的 `FENCED_BLOCK`、`QUOTED_SPANS`、`last_reply` 与状态文件函数，载入路径写成字面参数。
  - 门禁 `.claude/gate.d/code-tooling.sh` 的 agent-write-scope 格加了一条判据：Stop 上没有命令指向这个钩子就红，弄坏开关 `AGENT_WRITE_SCOPE_BREAK=session-number-stop-unchecked`，红样本 `.claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-session-number-red/`。
- 验过的：
  - 钩子自证「✓ 自检通过（查了 23 种）」。
  - 它的开关复跑「复跑了 10 条开关（1 份），自证红且点名的格与实际红的格逐格相同 10 条」。
  - 门禁样本档「点名的 1 道阶段判得都对（254 个用例）」。
  - 门禁那条新开关复跑「判错的样本与文件头点名的逐条相等」。
  - 换上之后，主 agent 的一条回复里写了没加引号的 mainagent 加字母，被它当场拦下重写。
- 主 agent 已知、没改的（交腿判）：
  - 检查只判收工那一条回复，回复当中的话、发给别的会话与子 agent 的消息不判。
  - 用会话全名、两位短号之外的说法指会话（「另一个会话」「那个会话」），检查不判；规则里只说不写 mainagent 加字母与 `<做什么>那一路的会话` 这类描述。
  - 编号写对了没有、写编号时有没有一起写清改了哪个文件做了什么，检查不判。
- 同一个会话里另有一处跟着改：`.claude/agent-common.md` 第 102 行 ㉒ 那一条补上 `git merge` / `git am` 的 `--no-verify`、`git am -n`、`git config --remove-section core` 与 `--rename-section core`、设成安装脚本装的那个绝对路径放行，与检出钩子此刻的代码对齐；代码一侧已在 `research/prompts/fg-timeout-large-file-governance-r3-main-verification.md` 那一题判过，这一轮只判这一句与代码对不对得上。
- 工具员报告 `~/.local/share/singlefs/work/report-session-number-stop-check/result.md`；工具层的背景在 `.claude/kb/tooling.md`。

## 范围

- `.claude/rules/writing-discipline-by-review.md`：「怎么说」最后一条。
- `.claude/agent-common.md`：第 41 行那一句与第 102 行 ㉒ 那一条。
- `.claude/hooks/session-number-stop-check.sh`：整份。
- `.claude/settings.json`：Stop 上新加的那一条注册。
- `.claude/gate.d/code-tooling.sh`：agent-write-scope 格新加的判据与开关、它的红样本。

## 要腿判的

每问回「成立」「不成立」「推不出」，推不出的算打中；打中给能照喂的输入（一条 Stop hook 输入 JSON、一段回复文本）或 文件:行。

正推腿：

1. 两处规则说的是不是用户要的事，彼此一致，与 `.claude/rules/writing-discipline-by-review.md` 别处、`.claude/agent-common.md` 别处、写闸与检出钩子 ⑰ 管的进仓文字不打架。
2. 检查判的是不是规则说的那件事：该拦的拦、该放的放（反引号、「」、代码块、只写编号、子 agent 收工）。
3. `.claude/agent-common.md` 第 102 行 ㉒ 那一条与检出钩子此刻 ㉒ 的判法逐项对得上吗。

攻方腿：

4. 检查漏拦：造得出写了 mainagent 加字母、用户照样看不出是谁、而钩子放行的回复吗（例如换行断开、全角字母、小写、把整段放进引号、续跑绕过）。
5. 检查误拦：造得出合规的回复被拦吗（例如引用规则原文、代码块外的路径或命令里含这种写法）。
6. 判不了的路（输入坏、取不到回复、抛异常）放行并记检出，记了没有；弄坏开关有没有不红在点名格的。
7. 门禁那条新判据与它的样本：注册挪到别的 matcher、写成别的命令形态时判得对不对。

## 分工

正推腿做第 1 到 3 问，攻方腿做第 4 到 7 问。两条腿都读「范围」里的文件。要跑的探针不自己跑，写成 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 的整条。
