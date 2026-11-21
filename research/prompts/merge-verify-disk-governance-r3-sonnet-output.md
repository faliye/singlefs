# 合入后验证压盘治理轮第三轮：正推腿报告（2026-11-19）

开工树：`sha256sum` 此刻 `scripts/gate/merge-and-verify.sh` 为 5dbf51739d699464addf4a105aa12d9fdb1952c348ea5e4bd69eeaa0baf7210e，`.claude/gate.d/break-switch-replay-files.tsv` 为 658dced79ecc62c0ff6cd863b796177d0fbb003a49461097cb167d03b0890d7e，与开工快照 `research/prompts/merge-verify-disk-governance-r3-snapshot/judged-sha256.txt` 里两行逐字相同（量的）。改前基线用 `~/.local/share/singlefs/work/merge-verify-drop-build-step/batch9/base/scripts/gate/merge-and-verify.sh` 对此刻的文件做 `diff`，输出 170 行、25 个差异块（量的，存在草稿目录 `diff.txt`）。

没有写模型。写了一份核真 cargo 目标集合的玩具脚本 `/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r3-sonnet/cargo-targets-toy.sh`，sha256 90ba897c4b98724ce0e8cc757a001c567888f2e00b12a26fbdb3e3149a8a9561，没跑（待主 agent 跑，命令在「要主 agent 跑的命令」一节）。

## 结论先说

- 打中 1 处：自证成功句里还留着被删那一支的说明，与代码相反（`scripts/gate/merge-and-verify.sh:5184`）。
- 推不出 3 处：自证里没有任何一格钉住「改了的 `[[bin]]` 段」「只改 bin 根文件」这两种补丁只挑那个 bin；「`workspace = true` 在指纹那一路也认」没有格钉住；两侧目标集合按根文件比，比判决 B 的字面多了「同一种段」一层，这一层对不对要对拍真 cargo。
- 成立：改法 B、改法 C 的实现，用户定案「先不集成测试，按改动测」的实现，文件头与代码的其余各处，被删那一支在代码、开关表、自证格里没有残留。

## 一、实现做的是不是第二轮判决改法 B、C 与用户定案

### 1.1 判决原文（现查）

读 `research/prompts/merge-verify-disk-governance-r2-main-verification.md` 的「三、改法」：

- A 行：「A：不做。用户看了判决之后定案「先不集成测试，按改动测」：第一轮改法 2（`[[bin]]` 段有差别时挑同包集成测试）整个删掉，改 `[[bin]]` 段或只改 bin 根文件都只挑那个 bin；判决 4 那一格照旧漏，是规则的选择。」
- B 行：「B：新旧两侧目标集合按根文件比，旧侧某个自动目标的根文件在新侧不再是任何目标的根文件才算被顶掉、退 3。零轮。」
- C 行：「C：build 算包时认 `workspace = true` 的依赖（照工作区根 `[workspace.dependencies]` 的 path）。零轮。」

（以上三行是原文整行，只去掉了行首的列表符。）

### 1.2 用户定案：只挑那个 bin，不挑同包集成测试 —— 成立

- 被删的分支：基线 diff 里 `bin_names = [name for section, name, _before, _after in changes if section == "bin"]` 一行与 `if bin_names and broken != "manifest-bin-tests-unpicked":` 起的 12 行（`land_on_targets` 逐个集成测试根文件、`notes.append(…集成测试目标 N 个…)`）全部在基线、不在此刻的文件（diff 块 `2127d2136`、`2145,2156d2153`，量的）。此刻 `land_manifest_changes` 里只剩落到每个段点名根文件的那一段（`scripts/gate/merge-and-verify.sh:2141` 的 `for entry in (after, before):` 起，到 `landed` 判断）。
- 行为：改 `[[bin]]` 段，新旧两侧段的根文件逐个交 `land_on_targets`；`roots_by_location` 里 `relative in targets` 时只返回这一个根（读 `scripts/gate/merge-and-verify.sh` 的 `roots_by_location` 函数，`if relative in targets and relative != rerun.DOC_TARGET: return [relative]`）；所以只落到那个 bin 的根文件。只改 bin 根文件走同一个函数，同一个结果。
- 自证里留的对应格：新加没写 path 的 `[[bin]]`，期望假 cargo 收到 `test --offline --no-fail-fast -p alpha --bin tool|`，不含 `--test case_a`（`scripts/gate/merge-and-verify.sh:3995` 起的 `narrow_case manifest-bin` 格，期望串在 3996 行 `--bin tool|`）。
- 缺口见 1.6：没有格钉「改了的段」与「只改 bin 根文件」。

### 1.3 改法 B：两侧目标集合按根文件比 —— 成立，比判决字面多一层「同一种段」

- 实现：`scripts/gate/merge-and-verify.sh:1896` 的 `new_roots = {(section, root)`，把新侧每个目标的（段，根文件）收成集合；`scripts/gate/merge-and-verify.sh:1902` 的 `if broken != "manifest-targets-compared-by-name" and (section, root) in new_roots:` 起一行 `continue`：旧侧自动认的目标，同名同根（上一行 `after[0] == root`）或根文件还在新侧同一种段的某个目标手里，都不算顶掉；其余点名并退 3（调用处 `scripts/gate/merge-and-verify.sh:2134`）。
- 判决 B 字面「不再是任何目标的根文件」；代码是「不再是同一种段（`[[test]]` 或 `[[bin]]`）里任何目标的根文件」，文件头也这样写（`scripts/gate/merge-and-verify.sh:108` 起「两侧集合按根文件比：旧侧某个自动认的目标的根文件在新侧不再是同一种段」）。差别只在跨段：旧侧自动 bin 的根文件，新侧被一段 `[[test]]` 点了，代码退 3（保守，不放行）。文件头写了「同一种段」，判决字面没写；代码与文件头一致、比判决字面窄，窄在跨段那一格。
- 判成立理由：判决 2 点的那种（新加的 `[[bin]] name = "tool-cli" path = "src/bin/tool.rs"`）按此刻代码不退 3，自证有格：`scripts/gate/merge-and-verify.sh:3999` 的 `manifest_patch "$scratch/narrow-manifest-renamed"`，期望 `--bin tool-cli|` 且输出里没有「判不出」「顶掉」。
- 此刻本仓真有这种写法：`crates/singlefs-unit-graph/Cargo.toml` 第 17–19 行 `[[bin]]`、`name = "unit-graph"`、`path = "src/bin/unit_graph.rs"`（`grep -n -A3 'unit-graph' crates/singlefs-unit-graph/Cargo.toml` 量的），文件头 110 行举的就是它。
- 跨段保守是不是对 cargo 的真实行为：我记得 cargo 只在同一种目标里按「名字或路径」去掉重复的自动目标，这是凭记忆，没读到 cargo 源码，当线索；要对拍写进「要主 agent 跑的命令」。

### 1.4 改法 C：build 算包认 `workspace = true` 的依赖 —— 成立（只在玩具树上，本仓今天用不到）

- 实现：`scripts/gate/merge-and-verify.sh:3386` 的 `def workspace_dependency_paths():` 读树根 `Cargo.toml` 的 `[workspace.dependencies]` 里写了 `path` 的项；`path_dependencies` 里 `scripts/gate/merge-and-verify.sh:3405` 的 `inherited = workspace_dependency_paths() if broken` 起一行决定开关是否生效，`scripts/gate/merge-and-verify.sh:3415` 的 `elif entry.get("workspace") is True and key in inherited:` 把 `{ workspace = true }` 的依赖按树根表里同名项的路径认进去。读法覆盖 `dependencies`、`dev-dependencies`、`build-dependencies` 与 `target.<平台>` 下这三张（`tables` 的拼法，`scripts/gate/merge-and-verify.sh` 的 `path_dependencies` 函数体）。
- 两路共用：`scripts/gate/merge-and-verify.sh:3501` 的 `needs_of = {member: path_dependencies(member)` 在 `dependents_of` 里，build 要编的包（`mode == "build-packages"`）与指纹的 `scripts/gate/merge-and-verify.sh:3522` `packages = closure(packages, path_dependencies) | dependents_of(packages)` 都走它，文件头 134 行「（path_dependencies，指纹那一路同一份）」与代码一致。
- 自证格：`scripts/gate/merge-and-verify.sh:4659` 起 `python3 - "$REBUILT" "$inherit_tree" build-packages …`，样本工作区 core、user、bystander，树根表里 `core` 写 path，user 写 `core = { workspace = true }`，补丁只改 core，期望算出「core user 」（4661 行的 ✗ 句「依赖方写 { workspace = true }（user 的 core…」）。开关 `workspace-dependencies-ignored`（文件头 402 行）让 `inherited` 为空，user 不算，格红。
- 本仓今天用不用：`grep -rn 'workspace = true' crates/*/Cargo.toml | wc -l` 输出 0，仓根 `Cargo.toml` 里 `grep -n 'workspace.dependencies' Cargo.toml` 无输出（量的，范围只限 `crates/*/Cargo.toml` 与仓根 `Cargo.toml`）。所以 C 在本仓今天没有生效的输入，只在自证的玩具树上走过，与主 agent 观测一致。
- 没有的东西：`[workspace.dependencies]` 里不带 path 的项（registry 依赖）`inherited` 里不收，等于不认，对 build 算包无影响（registry 依赖不在工作区里）；这一点不判，留给攻方腿。

### 1.5 文件头说明与代码逐处对照 —— 成立（除 1.6 点名的一处残留）

| 文件头处 | 它说什么 | 对应代码 | 判 |
|---|---|---|---|
| 108–110 行 | 两侧集合按根文件比；新加的段 path 指到自动认的文件而名字不同不算顶掉 | 1896、1902 行 | 一致 |
| 111 行 | `[[bin]]` 段有差别或补丁只改 bin 根文件，都只挑那个 bin，不另挑同包集成测试 | `roots_by_location` 的 `relative in targets` 早返回；`land_manifest_changes` 不再有挑集成测试的分支（2120 行说明） | 一致 |
| 100 行 | 「改了的段新旧两侧的根文件都落」 | 2141 行 `for entry in (after, before):` | 一致 |
| 106 行 | 自动目标集合：`autotests`/`autobins` 不是 false 才加自动目标；`src/main.rs`、`src/bin/<名>.rs`、`src/bin/<名>/main.rs`、`tests/<名>.rs`、`tests/<名>/main.rs` | `manifest_targets`（1865 行起）里 `package_section.get(automatic_key, True) is False: continue` 与 `candidates` 的拼法 | 一致 |
| 134 行、203 行 | `workspace = true` 的依赖照树根表认，build 与指纹同一份 | 3386、3415、3501、3522 行 | 一致 |
| 402 行开关说明 | 不认 `{ workspace = true }`，build 的包集合与指纹都漏 | 3405 行让 `inherited` 为空，两路共用 `path_dependencies` | 一致（但只有 build 那一格点名，见 1.6） |
| 580 行开关说明 | 按（段，名字）比，旧侧自动认的目标在新侧没有同名的就算顶掉 | 1902 行的开关条件；开关下新侧没有同名即 `after is None` 进 `shadowed` | 一致 |
| 1749–1750 行 | 开关归到 `land_manifest_changes`、`shadowed_automatic_targets` | 2122 行的函数说明也这样写 | 一致 |
| 5184 行 | 自证成功句：「包的 Cargo.toml 里 [[bin]] 段有差别时同包的集成测试一并挑上」 | 代码不挑 | **相反，见下节** |

开关数：`grep -c '^#   MERGE_AND_VERIFY_BREAK=' scripts/gate/merge-and-verify.sh` 输出 122，`grep -c '^#   SINGLEFS_WORK_ROOT_BREAK='` 输出 1，合 123；基线同两条命令输出 121 与 1，合 122；基线删一条（`manifest-bin-tests-unpicked`）、此刻加两条（`workspace-dependencies-ignored`、`manifest-targets-compared-by-name`），数目对得上，`.claude/gate.d/break-switch-replay-files.tsv` 第 72 行写「一百二十三条开关」（`grep -c '一百二十三条开关（' .claude/gate.d/break-switch-replay-files.tsv` 输出 1，量的）。
`bash -n scripts/gate/merge-and-verify.sh` 退 0（量的）。

## 二、删掉的「挑同包集成测试」那一支有没有残留

搜的范围：`scripts/` 与 `.claude/` 下全部文本文件（含 `.claude/gate.d/fixtures/`，`grep -rnF -- <词> scripts .claude -I`，输出贴 文件:行，不截断）。`research/`、`records/` 不在范围（`records/` 下那份调度记录在禁读清单里，没读、没搜）。

| 搜的词 | 命中 | 位置 |
|---|---|---|
| `manifest-bin-tests-unpicked`（开关名） | 0 | 无 |
| `manifest-bin-path`、`tool_next`（被删的自证格的名字与样本文件） | 各 0 | 无 |
| `bin_names`（被删的变量） | 0 | 无 |
| `CARGO_BIN_EXE`（在 `merge-and-verify.sh` 里） | 0 | 无；别的文件 7 处是 lib_function_units.py、mutation-prune.py 各自的内容，与这一支无关 |
| `同包集成测试` | 0 | 无 |
| `挑同包` | 2 | merge-and-verify.sh:111、:2120，都是「不另挑同包的集成测试」，说明现状 |
| `同包的集成测试` | 5 | merge-and-verify.sh:111、:2120、:3972、:3997 是「不挑」；:5184 是「一并挑上」，残留 |
| `一并挑上` | 1 | merge-and-verify.sh:5184，残留 |
| 正则 `bin.{0,40}集成测试\|集成测试.{0,40}bin` 限定 merge-and-verify.sh | 5 | 同上五行：111、2120、3972、3997、5184 |

另查别的文件有没有引这一支：`grep -rnE '段有差别|集成测试' .claude/skills/merge-verify/SKILL.md .claude/main-agent.md .claude/agents/*.md .claude/hooks/lib_run_scope.py` 命中 0（量的，范围只限这几份）。`git grep -nF 'manifest-bin-tests-unpicked' -- . ':!research' ':!records'` 命中 0 个文件（量的，范围：已跟踪文件）。

### 残留一处（打中）

`scripts/gate/merge-and-verify.sh:5184` 是 `--selftest` 通过时打的成功句，原文：
`  echo "    包的 Cargo.toml 里 [[bin]] 段有差别时同包的集成测试一并挑上、新加的 [[test]] 段与自动认的测试目标同名把它顶掉时判不出退 3 并点名被顶掉的目标、"`
与 111 行「都只挑那个 bin（--bin <名>），不另挑同包的集成测试」相反；基线里这行是对的，换上的实现没改它（diff 25 块里没有这一行，量的）。这是自证成功时给读的人看的一句总述，打出来就是错话。判：删掉的那一支在说明里**有残留**，不成立。

推翻条件：`grep -nF '有差别时同包的集成测试一并挑上' scripts/gate/merge-and-verify.sh` 命中 0 次，或该行已改成「只挑那个 bin」。

### 1.6 自证钉不住的三处（推不出，不是写错）

1. 「改了的 `[[bin]]` 段（path 改了，新旧两侧都有）只挑那个 bin」没有格钉。基线的 `manifest-bin-path` 格（改 path、期望 `--bin tool --test case_a`）随那一支一起删掉，没有换成「只挑 `--bin tool`」的格：`grep -nF 'manifest-bin-path' scripts/gate/merge-and-verify.sh` 命中 0 次；Cargo.toml 的格此刻是 8 个（`grep -n 'narrow_case manifest-' scripts/gate/merge-and-verify.sh` 输出 3991、3995、4000、4006、4011、4015、4019、4023 八行），它们是新加 `[[test]]`、新加 `[[bin]]` 没写 path、新加 `[[bin]]` 改名、同名 `[[test]]` 顶掉、新加要特性的 `[[test]]`、删掉 `[[test]]`、混改 `[dependencies]`、只加注释，没有一格是同一个段新旧两侧都在。新侧只落、旧侧只落各有格钉（新加的格、删掉的格），「新旧两侧一起落」这一种组合没有。
2. 「补丁只改 bin 的根文件，只挑那个 bin」（文件头 111 行后半句，用户定案的后半句）没有直接的格：自证里 `narrow_patch` 点到的 bin 路径只有 `crates/alpha/src/bin/shared_dump/inner.rs`（共用模块，期望两个 bin），没有一格补丁点 `src/bin/tool.rs` 这样的根文件、同包又有集成测试。被删的那一支只在 `land_manifest_changes` 里，所以删它这件事有 `manifest-bin` 格守（期望串不含 `--test case_a`）；以后在 `roots_by_location` 里给 bin 根文件另加挑集成测试，没有格会红。
3. `workspace = true` 在指纹那一路没有格：`grep -nF 'workspace = true' scripts/gate/merge-and-verify.sh` 命中的自证行只有 4647、4648、4656、4661，都在 `build-packages` 那一格里；402 行开关说明写的「build 的包集合与开跑那一刻的指纹都漏掉」后半句靠代码共用 `path_dependencies`（3501、3522 行）推出，没有格对着 `record` 做。判决 C 只说「build 算包时认」，所以这不算没做改法 C，是开关说明比格宽。

这三处各自的推翻条件：1、2 在自证里补一格后红绿相反就推翻「没有格钉」；3 在 `record` 模式里喂一棵 `workspace = true` 的树、指纹起点里没有依赖方就推翻「共用所以都认」。

## 三、数字与已定的几条（问题 1 只管换上的实现，这里只记我核到的）

- 此刻 `merge-and-verify.sh` 的开关数 123（见 1.5），与 tsv 第 72 行一致（量的）。
- 判决里的 `picked_test_targets=481`、`target_kib=11160` 对 `19480` 是攻方腿与辩方腿模型的读数，这一条腿没复算，写「没核」。

## 要主 agent 跑的命令

1. 在仓根交整份自证（背景材料里主 agent 的观测是它绿；这里的用处是确认 5184 行这句话此刻真的打出来，`task-server/task log <号> --all` 里能搜到「一并挑上」）：
   命令：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   在哪个目录起：仓根 `/home/fy5090/code/singlefs`
   看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded` 与 `exit=0`；日志里「✓ 自检通过（查了 N 种）」起头那一段成功句里有「[[bin]] 段有差别时同包的集成测试一并挑上」这半句（残留的证据）；没有「✗ 自检」。
2. 用真 cargo 列玩具包的目标，对拍两侧目标集合的假定（不编译、不联网，`cargo metadata --offline --no-deps`）：
   命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r3-sonnet -- bash cargo-targets-toy.sh /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r3-sonnet/toy-run`
   在哪个目录起：草稿目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r3-sonnet`
   看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；日志里五个包的目标清单：`renamed-bin` 里应有 `bin tool-cli src/bin/tool.rs` 且没有 `bin tool`（有就推翻「根文件换了名字，自动认的那个目标没了」，则 1.3 的「不算顶掉」连同判决 B 都得重读）；`bin-root-as-test` 里 `bin tool` 还在则跨段退 3 是多拒（保守），不在则代码对；`same-name-test` 里只有 `test case_a checks/extra.rs`；`dot-slash-path` 里看 `tests/case_a.rs` 是否另列一个目标（给攻方腿用的线索，不是这一格的判据）；`autobins-off` 里没有 `bin tool`。跑完 `toy-run` 目录由主 agent 删。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 用户定案：只挑那个 bin、不挑同包集成测试（代码） | 成立 | 挑集成测试的 12 行与 `bin_names` 在基线、不在此刻；`roots_by_location` 只返回 bin 自己的根（`scripts/gate/merge-and-verify.sh:2141` 一带） |
| 改法 B：两侧集合按根文件比 | 成立（比判决字面多「同一种段」） | `scripts/gate/merge-and-verify.sh:1896`、`:1902`；跨段退 3 偏保守，对 cargo 的真实行为要用真 cargo 对拍（待主 agent 跑） |
| 改法 C：build 算包认 `workspace = true` | 成立（本仓 0 处用到） | `scripts/gate/merge-and-verify.sh:3415`；指纹与 build 共用 `path_dependencies`；只在玩具树上有格 |
| 文件头说明与代码一致 | 成立，除一处 | 108–111、134、203、402、580、1749–1750 行逐处对上 |
| 被删那一支在代码里的残留 | 成立（没有） | 开关名、变量、自证格名、样本文件名各 0 命中 |
| 被删那一支在开关表里的残留 | 成立（没有） | 开关表里换成了 `workspace-dependencies-ignored`、`manifest-targets-compared-by-name`，数目 123 对得上 tsv |
| 被删那一支在自证说明里的残留 | **不成立** | `scripts/gate/merge-and-verify.sh:5184` 自证成功句仍写「同包的集成测试一并挑上」 |
| 全仓（`scripts/`、`.claude/`）别处引它的句子 | 成立（没有） | 搜 8 个词，只有 merge-and-verify.sh 内的命中；SKILL.md、main-agent.md、agents、lib_run_scope.py 里 0 命中 |
| 自证钉住「改了的 `[[bin]]` 段只挑那个 bin」 | 推不出 | 基线的 `manifest-bin-path` 格删后没有补；见 1.6 第 1 点 |
| 自证钉住「只改 bin 根文件只挑那个 bin」 | 推不出 | 没有补丁点 bin 根文件的格；见 1.6 第 2 点 |
| `workspace = true` 在指纹那一路也认 | 推不出（代码上推得出，没有格） | 见 1.6 第 3 点 |
| 报出的数字（481、11160 对 19480）能不能核 | 没核 | 这一条腿没复算 |

## 每条结论的推翻条件

- 「用户定案的代码成立」：`land_manifest_changes` 里再出现对 `tests/` 下目标逐个 `land_on_targets` 的分支，或 `manifest-bin` 格的期望串里出现 `--test case_a`，就推翻。
- 「改法 B 成立」：真 cargo 对 `renamed-bin` 玩具包列出 `bin tool`，就推翻「根文件还有目标认、不算顶掉」这一层。
- 「改法 C 成立」：自证格 `build-packages` 对样本工作区输出不是 `core user`（开关关掉时）或是 `core user`（开关打开时）之外的任何值，就推翻。
- 「文件头与代码一致」：在 108–111、134、203、402、580 行里任一句，读代码得出相反的行为，就推翻那一处。
- 「没有别的残留」：用 `scripts/`、`.claude/` 以外的路径，或这里没搜的词（例如「集成测试目标 N 个」的别种写法）搜到引它的句子，就推翻（范围写在第二节开头）。

## 没做什么

- 不判别的腿的格：问题 2 的攻方各角度（根文件换段、`./tests/a.rs` 与 `tests/a.rs`、目录形根文件、`autobins`/`autotests = false`、改名后的段撞自动目标、`workspace.dependencies` 不带 path、`package =` 改名、`[dev-dependencies]` 与 `[build-dependencies]`、`target.<条件>.dependencies`、工作区根不在补丁树根）只在 1.3、1.4 与玩具脚本里顺带记了线索，没有判；问题 2 第三条的数字没复算。
- 没往统一任务服务交任务；判决要引的数（自证绿、开关各红、真 cargo 目标清单）没复跑，命令在「要主 agent 跑的命令」一节；`cargo-targets-toy.sh` 只做了 `bash -n` 语法检查。
- 没有采纳或出判决；没有写模型，所以没有 `rerun.sh` 与复跑命令；没读禁读清单里的文件（另一条腿的报告、`records/2026-11-18-合入后验证压盘.md`、`~/.local/share/singlefs/work/merge-verify-drop-build-step/` 下的 `result*.md`）。
- 没有改 `scripts/gate/merge-and-verify.sh`：5184 行那一句要改，交主 agent（工具员）定点改，改完经服务交 `--selftest`（命令 1）确认。
- 草稿目录里没有仓副本与编译目录；留着 `diff.txt` 与 `cargo-targets-toy.sh`。
