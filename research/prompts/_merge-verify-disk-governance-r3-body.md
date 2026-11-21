# 合入后验证压盘：治理文件轮第三轮（merge-verify-disk-governance-r3，2026-11-19）

第二轮判决 `research/prompts/merge-verify-disk-governance-r2-main-verification.md`（第一轮判决 `research/prompts/merge-verify-disk-governance-r1-main-verification.md`）。第二轮之后用户定案「先不集成测试，按改动测」，改法 A 不做；照第二轮判决「三、改法」B、C 与用户定案改完换上，这一轮攻这几处修补，是最后一轮。改前（第二轮被判的那一版）基线在 `~/.local/share/singlefs/work/merge-verify-drop-build-step/batch9/base/scripts/gate/merge-and-verify.sh`，此刻主工作区是改后。

| 文件 | 这一轮判的改动 |
|---|---|
| `scripts/gate/merge-and-verify.sh` | 删掉「`[[bin]]` 段有差别时挑同包集成测试」那一支（连同开关 manifest-bin-tests-unpicked、它的 NOTE 行与自证格），改 `[[bin]]` 段或只改 bin 根文件都只挑那个 bin；新旧两侧目标集合按根文件比（`shadowed_automatic_targets`），新开关 manifest-targets-compared-by-name；build 算包与指纹两路认 `workspace = true` 的依赖（`workspace_dependency_paths`、`path_dependencies`），新开关 workspace-dependencies-ignored |
| `.claude/gate.d/break-switch-replay-files.tsv` | 第 72 行开关条数改成 123 |

## 问题

1. 正推腿：换上的实现做的是不是第二轮判决改法 B、C 与用户定案说的事，文件头说明与代码一致，删掉的那一支没有残留（说明、开关表、自证格、别处引它的句子）。
2. 攻方腿：攻这一轮的修补本身，不重复前两轮攻过的角度（去掉 build 漏拦单态化错与链接错、只改 `[[bin]]` 的 path 漏挑集成测试、同名顶掉不报、收尾守卫、Checking 行当重编证据、`[profile.dev]` 改键、`CARGO_PROFILE_DEV_DEBUG=0` 与测试那一步打架、跨台删上一次尝试的快照、整挑同包集成测试挑多）：
   - 按根文件比：同一份根文件在新侧换到另一种段（`[[test]]` 改成 `[[bin]]`）、根文件路径写法不同（`./tests/a.rs` 与 `tests/a.rs`、目录形 `tests/a/main.rs`）、`autobins`/`autotests = false`、改名后的段与另一个自动目标撞根文件。
   - 认 `workspace = true`：`[workspace.dependencies]` 里那一项不带 path（registry 依赖）、带 `package =` 改名、依赖写在 `[dev-dependencies]` 与 `[build-dependencies]`、`target.<条件>.dependencies` 下、工作区根不在补丁树的根。
   - 第二轮判决与前两轮报出的数字能不能核（`picked_test_targets=481`、`target_kib=11160` 对 `19480` 等），已定的几条撑不撑得住。

## 实现今天的样子

主 agent 的观测（读的是此刻主工作区的文件）：

- 这一轮没有改 `crates/` 下的代码。
- 验过的（换上之后在主工作区）：合入后验证脚本自证 type=succeeded exit=0、没有「✗ 自检」；五条开关（workspace-dependencies-ignored、manifest-targets-compared-by-name、manifest-shadowed-targets-unchecked、manifest-sections-refused、whole-package-tests）各单跑都退 1；门禁开关复跑「复跑了 123 条开关（1 份），自证红且点名的格与实际红的格逐格相同 123 条」「现判 124、复用 0」。
- 没验的：两侧目标集合是照 cargo 推目标的规矩自己算的，没拿真 cargo 对拍；认 `workspace = true` 只在自证的玩具树上走过。

## 判据与规则

- 合入后验证该做哪几步、挑哪些目标：`.claude/main-agent.md`「写与验分两段」验证段第 1 步（「只跑补丁改到的测试目标与变异行点名的测试目标（不跑整个包，也不带上依赖它们的包）」）与 `.claude/skills/merge-verify/SKILL.md`。
- 弄坏开关都要红在点名那一格：`.claude/singlefs-ai-sop/rules/show-me-test.md`「每一个弄坏开关都必须会红」。
- 改完再攻的那一轮攻上一轮的修补，已攻过而没打中的角度不重复，最多三轮：`.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」。
