# 三处定义改动与门禁 54 号分档：第一轮正文（2026-09-23）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、这一轮要判什么

用户 2026-09-19 定「层 0 全量不再每次门禁都跑，改在主 agent 一个任务收尾时统一跑一次」（原话「每次主 agent 执行完任务后统一执行」，`records/2026-09-19-里程碑二遗留收拢.md` 五之二第 8 问）。门禁 54 号 2026-09-23 照这句分了档，连带三处定义要改；用户同日定这三处走一轮三方（`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」、门禁 72 号）。

要判的是这一批改动本身：**三处定义改动与 54 号分档合起来，有没有一条可达的工作流历史让它们给出错的结果**——该跑全量的没跑、不该红的红了、该红的没红、或者一个 agent 照定义做会越出写范围或覆盖别人的东西。

| 格 | 改了什么 | 在哪 |
|---|---|---|
| K1 | 实验执行员的写范围加 `crates/mutations.tsv`：入库装置的变异行只追加进末尾，不改别人的行 | `.claude/agents/experiment-runner.md`「写范围」一节；`.claude/hooks/agent-write-scope.tsv` 新加一行 |
| K2 | 崩溃验证员跑 54 号时带 `--full`（不带只跑快档） | `.claude/agents/crash-verifier.md`「做什么」第 2 条 |
| K3 | 主 agent 在阶段同步那一步、这一批碰了 `crates/` 时后台跑 54 号 `--full`；整轮门禁的 54 号只核全绿标记 | `.claude/main-agent.md`「什么时候派哪个 agent」那张表「一个阶段任务结束」一行 |
| 背景 | 54 号分档：默认跑两条流的快档，再核 `<git common-dir>/singlefs-layer0-full-green` 里记的输入哈希与这批输入相等；`--full` 跑全量、判绿写标记；输入清单收窄成 `crates/ Cargo.toml Cargo.lock` | `.claude/gate.d/54-layer0-replay.sh`、`.claude/gate.d/stage-inputs.tsv` |

## 二、实现今天的样子（主 agent 的观测，2026-09-23 现查）

- 54 号读的输入：两条流的用例 `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs` 与 `second_transaction_step_zero_layer0.rs`，编译期与运行期都不读 `.claude/kb/layout/` 与 `research/results/`（`grep -n 'include_str\|include_bytes\|read_to_string\|research/\|\.claude/'` 在两份文件里零命中）；`crates/singlefs-harness/src/first_transaction_regions.rs` 提到布局表的只是一行注释。所以输入清单收窄到 `crates/ Cargo.toml Cargo.lock`。
- 标记里的输入哈希按文件内容算：`git ls-files -co --exclude-standard` 里磁盘上真有的文件，排序后逐个 sha256 再合一；未跟踪的文件也算进去（`crates/` 下有被 `pub mod` 引着、还没跟踪的源文件）。
- `gate.sh --staged` 在临时 worktree 上只拿 HEAD 加暂存区跑；标记放在 git common-dir，临时 worktree 里读得到同一份。
- 写范围闸：`.claude/hooks/agent-write-scope.sh` 按表判文件级的路径模式，判不了「只追加」。
- `crates/` 的产品代码这一轮不改；54 号跑的是 `crates/singlefs-harness` 里那两条流。

## 三、三格

### K1　实验执行员的写范围加 `crates/mutations.tsv`

要答：写范围是文件级的，「只追加这个实验的行、不改别人的行」只写在定义里、闸判不了——有没有一条可达历史让它改掉或覆盖别人的行（几个实现员在各自的仓副本里做、交补丁，补丁里也改 `crates/mutations.tsv`；门禁修复会话按表批量改名），以及那时哪道门禁会说话（33 号判原文命中一次、59 号复跑）。

### K2　崩溃验证员跑 54 号带 `--full`

要答：它与定义第 1 条「另有别的 `gate.sh` 正在跑 54 号时，不起 54 号」、与主 agent 收尾那一趟（K3）会不会撞在一起；全量在满负载的机器上要跑多久，它有没有「等」「超时」的写法；它写下的标记是不是就是整轮门禁要核的那一份。

### K3　主 agent 收尾跑 54 号全量

要答：「这一批碰了 `crates/` 就跑」这个触发条件漏不漏（一批只改了 `Cargo.lock`、只改了测试文件、别的会话改了 `crates/` 而这一批没改）；工作区与暂存区不同时（几个会话共写一个仓，别的会话有没暂存的 `crates/` 改动），收尾在工作区上跑出来的标记在 `--staged` 下对不对得上、对不上时出路句写的「在只含这一批的 worktree 里跑 `--full <它的根>`」走不走得通；标记放在 common-dir，几个会话各跑一次会不会互相覆盖。

## 四、分工（两条攻方腿的攻击面在这里分开写）

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 从用户定案与已有规则推 | K1–K3 | 每格：改动是不是用户定案与规则的直接后果（引原文整行抄）；推不出的写缺哪一句 |
| **云端攻方（Opus）** | 造工作流历史打改动 | **K3**，以及 K2 与 K3 撞在一起的那一格 | 每条历史写成可复现的命令序列，在仓的副本里真跑 54 号的判定段（用合成日志与假 cargo，照 `research/prompts/defs-gate54-tiering-implementer/selftest-driver.sh` 的做法，不真跑全量）；要么给出一条判错的历史，要么写「构造不出」并说清卡在哪一步 |
| **本地攻方** | 逐格填表 | **K1**、**K2** | 事实表逐格填：每条定义句碰到哪几条已有的定义句与门禁、给出什么不同的结果；不许只答 yes / no |

两条攻方腿互不重叠：云端攻方造跨会话、跨工作区与暂存区的历史，本地攻方枚举定义句之间的交叉。

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 改动照留 | 正推推得出，两条攻方腿在那一格都构造不出判错的历史（本地腿两次抽样一致） |
| 一格判「站不住」⇒ 改定义或改 54 号，改完再攻一轮 | 攻方给出一条可达历史，在上面该跑全量的没跑、不该红的红了、该红的没红，或 agent 照定义做会覆盖别人的东西 |
| K3 在「工作区 ≠ 暂存区」上打中 ⇒ 出路句与 `.claude/main-agent.md` 那一句一起改 | 攻方在副本上量到：收尾跑出的标记在 `--staged` 下对不上、而出路句那条路走不通 |

⚠️ 一格若打中的东西让改动与改前的做法一起出局（打中不分辨候选），按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」那一节办，另立一笔账。

## 六、各条腿交什么

- 报告先写进 `research/prompts/defs-gate54-tiering-r1-<腿名>-output.md`，**每次工具调用写进文件的内容不超过 150 行**，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `/tmp/claude-1000/<腿名>/`。
- 引 kb 与规则条款写那份文件自己的行号，去文件里现查，不从背景材料里数。
- 引产物整行抄，带文件名。
