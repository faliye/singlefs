# 合入后验证压盘：治理文件轮第二轮（merge-verify-disk-governance-r2，2026-11-19）

第一轮判决 `research/prompts/merge-verify-disk-governance-r1-main-verification.md`（正文 `research/prompts/_merge-verify-disk-governance-r1-body.md`）。第一轮坐实三处，用户看了判决之后重判「去掉 build」那一样作废；照第一轮判决「三、改法」1–3 改完换上，这一轮攻这几处修补。改前（第一轮被判的那一版）基线在 `~/.local/share/singlefs/work/merge-verify-drop-build-step/batch7/base/`（保留原相对路径），此刻主工作区是改后。

| 文件 | 这一轮判的改动 |
|---|---|
| `scripts/gate/merge-and-verify.sh` | 恢复 build 一步（只编补丁改到的包与依赖它们的包，build 那一条设 `CARGO_PROFILE_DEV_DEBUG=0`，文件头第 137 行起）；`[[bin]]` 段有差别时一并挑同包的集成测试；新旧两侧各算目标集合，自动认的目标被同名段顶掉退 3；新开关 skip-build、build-whole-workspace、build-debuginfo-kept、manifest-bin-tests-unpicked、manifest-shadowed-targets-unchecked |
| `.claude/main-agent.md` | 第 57 行写回「build（build 只编补丁改到的包与依赖它们的包）」 |
| `.claude/skills/merge-verify/SKILL.md` | 第 22 行写回 build |
| `.claude/hooks/lib_run_scope.py` | 合入后验证范围说明那一句写回 build（只这一句，别的行不在这一轮判） |
| `Cargo.toml` | 第 30 行注释改成现状句 |
| `.claude/gate.d/break-switch-replay-files.tsv` | 第 72 行开关条数改成 122 |

## 问题

1. 辩方腿：复核第一轮判决够不够得着——判决 1（去掉 build 漏拦单态化错）、2（只改 `[[bin]]` 的 path 漏挑集成测试）、3（同名顶掉不报）三处坐实的依据成不成立；判决 6 里「两条腿都没打中」的三块（收尾守卫、删上一次尝试的快照、Checking 行当重编证据）是不是同样该判打中；改法 1 恢复 build 是不是同样会被判决 1 的那种错打中（关掉调试信息的 build 拦不拦得住）。
2. 攻方腿：攻这一轮的修补本身，不重复第一轮攻过的角度：
   - `CARGO_PROFILE_DEV_DEBUG=0` 只设在 build 那一条上，会不会让 build 与测试那一步的产物、指纹、重编核对互相打架（cargo 另编一份依赖、重编核对读 build 的 Compiling 行认错包）。
   - `[[bin]]` 段有差别时挑同包集成测试：挑多了（不用这个 bin 的集成测试）、挑少了（别的包的集成测试起这个 bin）、要特性的那几个怎么记。
   - 新旧两侧目标集合比较：会不会把该挑的判成被顶掉（误退 3）、把被顶掉的漏掉（目录形目标、`autotests = false`、`[lib]` 改名）。
   - 第一轮判决写明「核不动」的那一格：跨台时上一次尝试还在第二台上跑，删上一次尝试的快照会不会删在用的那一棵。

## 实现今天的样子

主 agent 的观测（读的是此刻主工作区的文件）：

- 这一轮没有改 `crates/` 下的代码；`Cargo.toml` 只改了一行注释。
- 验过的（换上之后在主工作区）：合入后验证脚本自证 type=succeeded exit=0、没有「✗ 自检」；`run-scope-guard.sh --selftest` type=succeeded exit=0；六条开关（build-debuginfo-kept、manifest-bin-tests-unpicked、manifest-shadowed-targets-unchecked、skip-build、build-whole-workspace、rebuild-evidence-test-only）各单跑都退 1；门禁开关复跑「复跑了 122 条开关（1 份），自证红且点名的格与实际红的格逐格相同 122 条」「现判 123、复用 0」。
- 没验的：关掉调试信息的 build 一趟实际占多少盘、多花多少时间，没起过真 cargo 量；「自动目标被顶掉」是照 cargo 推目标的规矩自己算的，没拿真 cargo 对拍。

## 判据与规则

- 合入后验证该做哪几步：`.claude/main-agent.md`「写与验分两段」验证段第 1 步与 `.claude/skills/merge-verify/SKILL.md`。
- 弄坏开关都要红在点名那一格：`.claude/singlefs-ai-sop/rules/show-me-test.md`「每一个弄坏开关都必须会红」。
- 改完再攻的那一轮攻上一轮的修补，已攻过而没打中的角度不重复：`.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」。
