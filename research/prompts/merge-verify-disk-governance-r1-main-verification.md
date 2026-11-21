# merge-verify-disk-governance-r1：判决（2026-11-19）

正文 `research/prompts/_merge-verify-disk-governance-r1-body.md`，开工快照 `research/prompts/merge-verify-disk-governance-r1-snapshot/`。这一题的第一轮，之前没有判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/merge-verify-disk-governance-r1-sonnet-output.md` | 「打中 5 处」：链接残留两处说法、`Cargo.toml` 注释前提残留、广谱复用键整张变、缺省路径目录形、「同一套判据」措辞 |
| 攻方（Opus） | `research/prompts/merge-verify-disk-governance-r1-opus-output.md` | 「打中 4 处」：去掉 build 漏单态化错与链接错两处、只改 `[[bin]]` 的 path 漏挑起它的集成测试、新加 `[[test]]` 同名顶掉自动认的测试目标 |

核查：

- 开工快照：腿交齐之后 `sha256sum -c research/prompts/merge-verify-disk-governance-r1-snapshot/judged-sha256.txt` 六份全 OK，被判文件自开工以来没被改过，开工那一刻的树就是此刻的主工作区，`snapshot-tree.py` 不用倒推。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/merge-verify-disk-governance-r1-sonnet-output.md research/prompts/merge-verify-disk-governance-r1-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_merge-verify-disk-governance-r1-body.md` 结局 type=command-failed exit=1，末行「✗ 核了 23 处引文、60 处裸行号，4 处对不上，60 处没判」。对不上的 4 处都是行号漂移：`scripts/gate/admission.py` 与 `scripts/mutation/lib_mutation_units.py` 在腿交回之后被别的会话改过，引的原文逐字还在，只挪了行（`admission.py:868` 那一句此刻在第 876 行，`lib_mutation_units.py:103` 那一句在第 110 行）。这两份不在被判之列，腿引它们只当依据，内容对得上。
- 攻方腿的模型：主 agent 把 `research/prompts/merge-verify-disk-governance-r1-opus-model/` 拷到 `~/.local/share/singlefs/work/merge-verify-disk-governance-r1-main/model`（两份脚本 sha256 与原目录相同），在副本里交 `bash <副本>/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs <工作根>/merge-verify-disk-governance-r1-main/model-run <副本>`，结局 type=succeeded exit=0，末行 `name=summary matched=21 mismatched=1`。运行目录（196M）已删。
- 正推腿第 4 处（`rerun-failed-tests.py` 不认 `<名>/main.rs` 形）主 agent 现查：`scripts/gate/rerun-failed-tests.py` 第 264–265 行认 `<目录>/<名>/main.rs`，腿这一句不成立。

## 二、判决

1. **去掉 build 打穿（坐实）**。模型复跑逐格：单态化出错那一臂 `poly_after-postmono_clippy exit=0`、`poly_after-postmono_picked_test exit=0`、`poly_after-postmono_build exit=101 … first_error=error[E0080]: evaluation panicked: 值超过上限`、`poly_after-postmono_build_without_debuginfo exit=101`；链接那一臂 `poly_after-link_build exit=101 … linking with cc failed`、`poly_after-link_build_without_debuginfo exit=101`。补丁改了被依赖的包，依赖它的包里只有真编译才报的错，clippy 与挑出的测试都过，合入后验证退 0、合进主工作区，要到提交时 `.claude/singlefs-ai-sop/scripts/check.sh` 的 `cargo build --all-targets` 才红。链接那一臂的 clippy 格没对上（`poly_after-link_clippy exit=101 expected=zero … error: empty line after doc comment`，玩具树里一条无关的 lint），那一臂没干净坐实，单态化那一臂已够定案。
2. **只改 `[[bin]]` 的 path 漏挑集成测试（坐实）**：`alpha_items_binpath … items=alpha:bins/tool_next.rs|--bin tool`，而 `alpha_binpath_cli_test exit=101 … assertion left == right failed`，起这个 bin 的集成测试红了却没被挑上。
3. **新加 `[[test]]` 同名顶掉自动认的测试目标（坐实）**：`alpha_shadow_list … test=beta_guards_greeting hits=0`（改前 `alpha_before_list … hits=1`），脚本挑 `--test beta` 只跑到顶替进来的 `other_check`，一句不报。
4. **正推腿的残留说法**：`.claude/agents/implementation-writer.md:32`、`.claude/rules/implementation-workflow.md:60`「链接留给合入后验证」与合入后验证脚本第 68 行「同一套判据」：改法 1 把 build 恢复之后这几句又成立，不另改；`Cargo.toml` 第 30 行注释写的是改前那一趟的事，改成现状句。
5. **广谱变异全局键整张变**：坐实（腿算过 HEAD 与工作区两个哈希不等），只让这一次整表现判一遍、不改结局，背景材料已列，不改。
6. **收尾守卫、删上一次尝试的快照、重编核对改读 clippy 的 Checking 行**：两条腿都没打中。攻方腿写明跨台那一种（上一次尝试还在第二台上跑）没读清服务重排与第二台清理的先后，那一格核不动，不当核过。

## 三、改法

1. 合入后验证恢复 build 一步：只编补丁改到的包与依赖它们的包（恢复改前 build-packages 那一段算包的做法），那一步关掉调试信息（`CARGO_PROFILE_DEV_DEBUG=0` 只设在 build 那一条上）；恢复 skip-build 一类开关与自证格；`.claude/main-agent.md:57`、`.claude/skills/merge-verify/SKILL.md:22`、`.claude/hooks/lib_run_scope.py` 合入后验证范围说明那一句把 build 写回。量过：模型里关掉调试信息的 build 两臂都红（判决 1 引的 `*_build_without_debuginfo` 两行）。
2. 挑测试目标：`[[bin]]` 段有差别时，把同包的集成测试一并挑上。零轮（攻方腿只在自己的模型上设计过）。
3. 挑测试目标：新旧两侧各算一遍包的测试目标集合，改后少了的（被同名段顶掉的自动目标）退 3、原因点名。零轮。
4. `Cargo.toml` 第 30 行注释改成现状句。

改法 2、3 改的是 `scripts/` 下的治理脚本，不是产品代码；照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」改完再攻一轮，攻这几处修补。

## 交用户的

- 去掉 build 是用户选的「两样都做」里的一样；这一轮坐实它漏拦依赖改动包的那些包里的单态化错（链接错那一臂没干净坐实），改法 1 恢复 build 但关掉调试信息，两头都顾到。用户先前已说「你直接改就好了」，照改，交用户时写明。
- 关掉调试信息的 build 一趟实际省多少盘，没量过：要等下一趟真跑的合入后验证编出来量。

## 这一轮被判过的文件

- research/prompts/_merge-verify-disk-governance-r1-body.md：背景材料；两条腿照它的「问题」两条与「判据与规则」三条判，`task_server.py:8933`、`admission.py:867` 两处行号已漂移
- scripts/gate/merge-and-verify.sh：去掉 build 坐实漏拦单态化错；挑测试目标认 `[[bin]]` 段漏挑集成测试、新加 `[[test]]` 同名顶掉自动目标两处坐实；收尾守卫、删上一次尝试的快照、Checking 行当重编证据两条腿都判成立
- .claude/main-agent.md：第 57 行验证段第 1 步与脚本做的步一致，随改法 1 写回 build
- .claude/skills/merge-verify/SKILL.md：第 22 行与脚本一致，随改法 1 写回 build
- .claude/hooks/lib_run_scope.py：合入后验证范围说明那一句与脚本一致，随改法 1 写回 build
- Cargo.toml：`[profile.dev] debug = "line-tables-only"` 只让广谱变异全局键整张变一次，admission、变异单元与函数单元图的键都按键名去掉 debug；第 30 行注释前提残留

## 回看决策

不涉及决策：这一轮判的是合入后验证脚本、两份流程文字、范围声明钩子的一句说明与仓根调试构建的调试信息量，都不碰 `.claude/kb/decisions/` 下的任何一条。
