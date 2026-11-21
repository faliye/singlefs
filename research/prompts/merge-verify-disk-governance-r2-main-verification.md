# merge-verify-disk-governance-r2：判决（2026-11-19）

正文 `research/prompts/_merge-verify-disk-governance-r2-body.md`，开工快照 `research/prompts/merge-verify-disk-governance-r2-snapshot/`。（待填：这是这一题的第几轮，前几轮的判决）

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（Sonnet） | `research/prompts/merge-verify-disk-governance-r2-sonnet-output.md` | 「打穿 0 处，推不出 1 处」：第一轮判决 1–5 站得住；判决 6「两条腿都没打中」不成立（只有攻方腿一次抽样），收尾守卫推不出；跨台删快照核动了、站得住；改法 1 拦得住判决 1 那种错；改法 2 落到验证包会挑上几百个无关测试；依赖写成 `workspace = true` 时算包漏依赖方 |
| 攻方（Opus） | `research/prompts/merge-verify-disk-governance-r2-opus-output.md` | 「打中 3 处」：改 `[[bin]]` 段挑多了（验证包 481 个）；只改 bin 源码挑少了（两臂都漏）；新加段 path 指向自动认的文件而名字不同时误退 3；撤回第一轮自己提的「整挑同包集成测试」 |

核查：

- 开工快照：`sha256sum -c research/prompts/merge-verify-disk-governance-r2-snapshot/judged-sha256.txt` 只有 `.claude/gate.d/break-switch-replay-files.tsv` 对不上，是别的会话在改这份共用登记表；这一轮判的第 72 行（「一百二十二条开关（2026-11-19」起头）此刻还在，攻方腿没引这份，辩方腿引了 2 处、只用来说它开工时已变，不拿它推结论。别的六份 OK。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/merge-verify-disk-governance-r2-sonnet-output.md research/prompts/merge-verify-disk-governance-r2-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_merge-verify-disk-governance-r2-body.md` 结局 type=succeeded exit=0，末行「✓ 核了 67 处引文、25 处裸行号，对上 66 处，没判 26 处」。
- 两条腿的模型：主 agent 把 `research/prompts/merge-verify-disk-governance-r2-opus-model/`、`research/prompts/merge-verify-disk-governance-r2-sonnet-model/` 拷到 `~/.local/share/singlefs/work/merge-verify-disk-governance-r2-main/`（`rerun.sh` 两份 sha256 与原目录相同），在副本里各交一次 `bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs <运行目录> <副本>`：攻方腿那一份 type=succeeded exit=0、`name=summary matched=12 mismatched=0`；辩方腿那一份 type=succeeded exit=0、`name=summary matched=20 mismatched=0`。两个运行目录（72M、808K）已删。

## 二、判决

1. **第一轮改法 2 挑多了（坐实，两条腿都打中）**：攻方腿模型 `name=verification_bin_intact exit=0 picked_test_targets=481 tests_rs=481 … verdict=matched`，`name=verification_bin_executable_users files=0 … verdict=matched`：验证包改一段 `[[bin]]`，同包 481 个集成测试目标全挑上，而 481 份里写了 `CARGO_BIN_EXE_` 的一份都没有，等于不经整包确认就整包跑。第一轮攻方腿撤回了这条改法。
2. **新加段 path 指向自动认的文件、名字不同时误退 3（坐实）**：`name=renamed_items exit=0 refusals=1 … expected=refused_as_shadowed verdict=matched`，而真 cargo 两个目标都列得出（`renamed_bin_list … hits=1`、`renamed_test_list … hits=1`）。两侧集合按（段，名字）比，没按根文件比。
3. **build 算包认不出 `workspace = true` 的依赖（坐实）**：辩方腿模型 `name=poly_build_packages_workspace_inherit packages=poly-core … verdict=matched`，依赖方漏了，而它的 build 会红（`poly_inherit_dependent_build_would_fail exit=101 … error[E0080]`）。本仓今天 0 处这种写法，照修。
4. **只改 bin 源码不挑起它的集成测试（坐实，改前改后两臂都漏）**：`name=bin_body_items … items=alpha:src/bin/tool.rs|--bin tool`，`name=bin_body_after_cli exit=101 … panicked`。随改法 A 一并补：起 bin 的测试照字面 `CARGO_BIN_EXE_<名>` 认，只改 bin 源码也挑。只改库源码时集成测试不挑（`alpha_libedit_cli_test_goes_red exit=101`）是 `.claude/main-agent.md:57`「只跑补丁改到的测试目标与变异行点名的测试目标（不跑整个包，也不带上依赖它们的包）」这条规则本身的选择，这一轮不改。
5. **改法 1（恢复 build、关掉调试信息）站得住**：`poly_after_build_without_debuginfo exit=101 … error[E0080]`、`poly_after_build_default_debuginfo exit=101` 两臂都红；`name=debuginfo_off … build_compiling_leaf=1 leaf_rlibs=2 target_kib=11160`、`name=debuginfo_kept … leaf_rlibs=1 target_kib=19480`：关掉调试信息时依赖另编了一份，编译目录仍小（玩具树，只当方向，不外推到本仓的数）。留着。
6. **跨台删上一次尝试的快照：两条腿都核动、都判站得住**：服务等两台清理回执确认才落结局、才重排（`task-server/task_server.py` 第 5995 行起的 `finish_attempt`），删不到在用的那一棵。第一轮判决 6 的「两条腿都没打中」写法不准（那三块只有攻方腿一次抽样），这一轮辩方腿补上一次：Checking 行当重编证据站得住；收尾守卫推不出，它给了复刻命令，主 agent 已有工具员那一份复刻（加守卫 2000 次 0 次），这一轮不另起。

## 三、改法

- A：不做。用户看了判决之后定案「先不集成测试，按改动测」：第一轮改法 2（`[[bin]]` 段有差别时挑同包集成测试）整个删掉，改 `[[bin]]` 段或只改 bin 根文件都只挑那个 bin；判决 4 那一格照旧漏，是规则的选择。
- B：新旧两侧目标集合按根文件比，旧侧某个自动目标的根文件在新侧不再是任何目标的根文件才算被顶掉、退 3。零轮。
- C：build 算包时认 `workspace = true` 的依赖（照工作区根 `[workspace.dependencies]` 的 path）。零轮。
- 攻方腿提的「build 与测试用同一档」不采：判决 5 的读数是关掉调试信息那一臂编译目录小。

改完照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」开第三轮，攻这三处修补；第三轮改的修补标零轮。

## 交用户的

- 判决 4 里「只改库源码时不挑集成测试」是现行规则的选择，跟 mainagent(A)（它对旧账、在整理 subagent 定义）早先转来的「按函数单元闭包挑测试目标」是同一件事，改规则要用户定。

## 这一轮被判过的文件

- research/prompts/_merge-verify-disk-governance-r2-body.md：背景材料；两条腿照它的「问题」两条判
- scripts/gate/merge-and-verify.sh：恢复 build 站得住；改 `[[bin]]` 挑同包集成测试挑多了；两侧目标集合按名字比误退 3；算包认不出 `workspace = true`；只改 bin 源码漏挑起它的测试
- .claude/main-agent.md：第 57 行写回 build 与脚本做的步一致
- .claude/skills/merge-verify/SKILL.md：第 22 行写回 build 与脚本一致
- .claude/hooks/lib_run_scope.py：合入后验证范围说明那一句写回 build 与脚本一致
- Cargo.toml：第 30 行注释是现状句
- .claude/gate.d/break-switch-replay-files.tsv：第 72 行条数 122 与门禁开关复跑「复跑了 122 条开关」一致

## 回看决策

不涉及决策：这一轮判的是合入后验证脚本、两份流程文字、范围声明钩子的一句说明、仓根一行注释与开关复跑登记表的条数，都不碰 `.claude/kb/decisions/` 下的任何一条。
