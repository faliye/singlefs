# 放置类防再发措施（治理文件轮，第一轮）

方案按 `crates/` 今天的实现来谈。调度记录 `records/2026-11-19-产品GPU包测试与E202装置归位.md`；措施的来历是那份记录「没能避免的原因」五条缺口与两轮设计判决（`research/prompts/gpu-tests-placement-design-r1-main-verification.md`「二、判决」第 7 条、`research/prompts/gpu-tests-placement-design-r2-main-verification.md`「二、判决」第 9 条）。

## 问题

两件工具员把五条措施做成了检查，已经换上、暂存。这一轮判它们判得对不对、拦不拦得住原来那几次发生的事、有没有误伤与漏判。开工快照 `research/prompts/gpu-placement-measures-governance-r1-snapshot/judged-sha256.txt` 点名的 20 份就是被判的文件。

## 改了什么（主 agent 的观测，`git diff --cached` 与工具员报告）

- 规则 `.claude/rules/directory-layout.md` 新一节「crates/：各包放什么」：一个包的集成测试放它自己的 `tests/`，不为测试另开包；实验装置只放实验包；验证包只装测试与门禁、崩溃入口自己起的工具；产品包不放装置二进制；不跨包 `#[path]`、`include!`；放置类弹窗选项先点名位置标准。
- 门禁 code-tooling 新格 crate-roles（判法 `scripts/gate/crate-roles-check.py`，650 行）：读 `.claude/gate.d/crate-roles.tsv`（11 个包的角色）、`crate-roles-lag.tsv`（测试包整包一行、验证包 21 个 `e<数字>_` 装置各一行）、`crate-roles-allowed.tsv`（e165 一行）；判包都登了、角色认得、产品包无 `src/bin/`、`e<数字>_` 二进制只在实验装置包、空库加测试目标的包、测试支撑包被依赖者不到 2 个、有包依赖实验装置包。
- 门禁 checker-independence-and-sync：⑧ 的允许共用加 `singlefs-unit-graph`（连同 `verus_syn`、`proc-macro2`、`quote`）与 `prune-server`，另判它们的闭包里没有 GPU 一族与验证包、产品 GPU 包；新 ⑨ 判跨包 `#[path]`、`include!`、`include_str!`、`include_bytes!` 引另一个包目录下的 `.rs`、`.wgsl`（`crate-cross-references-allowed.tsv` 4 对放行、`crate-cross-references-lag.tsv` 2 对滞后）。
- 门禁 doc-process-records 的 code-round-checklist 格：检查单八格加第九格「放的位置」，截止日 2026-11-20 之前的正文只判八格，没有日期的 8 份旧正文登进只判八格的名单。`.claude/rules/implementation-workflow.md`「代码轮正文的检查单」与 `.claude/skills/three-way/SKILL.md` 同名一节跟着改。
- 钩子 `.claude/hooks/ask-user-placement-guard.sh`（`PreToolUse` 的 `AskUserQuestion`）：问句或选项里有放置类的词、另有「包」或 `crates/` 的算放置类；要求问句点名 `directory-layout.md` 那一节、选项说明以「标准：」或「偏离：」起头、至少一个「标准：」。
- 写闸 `.claude/hooks/write-guard.sh` 第十一道加一半：`tooling-writer` 往主工作区里已有门禁的样本档目录写（改已有、新建都算）执行前拒，出路是草稿里的样本补丁、门禁换上之后由主 agent 打。
- code-tooling 的 research-script-selftests 格接上 `scripts/experiment-apparatus/e173_r2_compare.py` 的自证；三条开关头注补点名样本。
- 登记：`.claude/gate.d/stage-owners.tsv` 一行、`.claude/hooks/selftest-registry.tsv`、`.claude/settings.json` 注册、`.claude/gate.d/break-switch-replay-files.tsv` 两行、`scripts/gate/README.md`、`.claude/rules/verification-reference.md`「门禁管哪一半」三行。
- 还没落的：`.claude/agent-common.md` 与 `.claude/agents/tooling-writer.md` 第 3 步 5 处（别的会话一轮三方的快照挡着，规格在 `~/.local/share/singlefs/work/gpu-tests-placement-measures-2/pending-docs-spec.json`），这一轮不判。

## 已经量的（结局行 type= exit= 那一段）

- 自证：`crate-roles-check.py --selftest` exit=0；`ask-user-placement-guard.sh --selftest` exit=0（「查了 25 种」）；`write-guard.sh --selftest` exit=0。
- 样本档：checker-independence-and-sync、doc-process-records 两道都「判得都对」。
- 开关复跑：crate-roles-check.py 14 条、放置类钩子 10 条、写闸、三条补点名的头注，点名与实测逐格相同。
- 真仓单格：checker-implementation-disjoint、code-round-checklist、crate-roles、research-script-selftest-coverage 都绿。
- 没跑成的：code-tooling 自己改过的 9 条开关复跑全部拒算（拒算点 `scripts/gate/git-hooks-install.py:133`，别的会话在改）。

## 判据格

每格答「成立 / 不成立 / 推不出」，给依据（文件:行）；打中的写反例输入。

1. **拦不拦得住原来那几次**：拿调度记录「原因」五条逐条喂：产品包里放 `src/bin/e202_…rs`、另开一个空库加十个测试目标的 `-tests` 包、测试目标 `#[path]` 引验证包 `tests/` 下的模块、弹窗问「E202 放哪」只列偏离标准的选项、工具员先把样本改进主工作区。每一条现在由哪一道、哪一格、哪一行判红；判不红的就是打中。
2. **误伤**：今天的树上这几格都绿，靠的是 `crate-roles-lag.tsv`、`crate-roles-allowed.tsv`、`crate-cross-references-*.tsv` 与只判八格的名单。逐行看这些放行与滞后的理由站不站得住、会不会永远不缩；放置类钩子的词表与「包 / crates/」条件，会不会拦下与放置无关的弹窗、会不会放过放置类弹窗（换个说法问）。
3. **判法本身**：`crate-roles-check.py` 的「库为空」「被依赖者数」「`e<数字>_` 文件名」三种认法，⑨ 的路径解析（`concat!(env!("CARGO_MANIFEST_DIR"), …)`、相对路径、`..`），写闸第十一道认「已有门禁」的办法：各有没有绕得过去的写法。
4. **与已有检查的重叠**：`python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list` 列的门禁与钩子里，有没有已经判同一件事的；新文件的 `# gate-similar:` 写得对不对。
5. **规则正文**：`directory-layout.md` 新一节与 `.claude/singlefs-ai-sop/rules/rules-discipline.md`（只写怎么做、不写论证）合不合；与 `.claude/rules/verification.md`「定义与名字」验证包那一行（今天还写验证包装「真设备装置二进制（`src/bin/`）」）冲不冲突。

## 分工

- 正推腿：第 1、4、5 格，逐条走到文件与行；第 1 格每条原因写出它在今天的判法里走哪一支。
- 攻方腿：第 2、3 格，造绕得过去的输入（文件布局、`#[path]` 写法、弹窗措辞、写闸路径），每个写成主 agent 能照跑的样本或命令；再攻「已经量的」里每一句，照日志与命令核。
