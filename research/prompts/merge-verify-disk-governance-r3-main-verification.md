# merge-verify-disk-governance-r3：判决（2026-11-19）

正文 `research/prompts/_merge-verify-disk-governance-r3-body.md`，开工快照 `research/prompts/merge-verify-disk-governance-r3-snapshot/`。这一题的第三轮，也是最后一轮；前两轮判决 `research/prompts/merge-verify-disk-governance-r1-main-verification.md`、`research/prompts/merge-verify-disk-governance-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/merge-verify-disk-governance-r3-sonnet-output.md` | 「打中 1 处」：自证成功句第 5184 行残留「[[bin]] 段有差别时同包的集成测试一并挑上」；推不出 3 处：没有格钉「改了的 `[[bin]]` 段只挑那个 bin」「只改 bin 根文件只挑那个 bin」，`workspace = true` 在指纹那一路没有格 |
| 攻方（Opus） | `research/prompts/merge-verify-disk-governance-r3-opus-output.md` | 「打中 3 处，但只是 cargo 那一侧量过」：没写 edition 的包（cargo 按 2015）新加段后不再认同类自动目标、脚本不看 edition；path 写成 `./tests/a.rs` 时脚本不归一、误退 3；`[patch.crates-io]` 换成工作区包时脚本不读 `[patch]`、算包漏依赖方 |

核查：

- 开工快照：`sha256sum -c research/prompts/merge-verify-disk-governance-r3-snapshot/judged-sha256.txt` 三份全 OK。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/merge-verify-disk-governance-r3-sonnet-output.md research/prompts/merge-verify-disk-governance-r3-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_merge-verify-disk-governance-r3-body.md` 结局 type=succeeded exit=0，末行「✓ 核了 10 处引文、23 处裸行号，对上 10 处，没判 23 处」。
- 正推腿第 5184 行那一句主 agent 现读坐实：第 5184 行「包的 Cargo.toml 里 [[bin]] 段有差别时同包的集成测试一并挑上」，第 111 行「都只挑那个 bin（--bin <名>），不另挑同包的集成测试」。
- 正推腿的真 cargo 对拍：主 agent 把它的 `cargo-targets-toy.sh`（sha256 前 16 位 90ba897c4b98724c）拷到 `~/.local/share/singlefs/work/merge-verify-disk-governance-r3-main/`，读过只建玩具包、只跑 `cargo metadata --offline --no-deps`，在副本里交，结局 type=succeeded exit=0：`renamed-bin` 只剩 `bin tool-cli src/bin/tool.rs`（自动的 `tool` 被同根文件的段替掉）；`bin-root-as-test` 同时有 `bin tool` 与 `test t src/bin/tool.rs`（跨种段不替）；`same-name-test` 只剩 `test case_a checks/extra.rs`（`tests/case_a.rs` 不再编）；`dot-slash-path` 只剩 `test b tests/case_a.rs`（cargo 把 `./` 归一了）；`autobins-off` 没有 bin。运行目录已删。
- 攻方腿的「本仓今天 0 处」主 agent 现查：跟踪的 `Cargo.toml` 里带 `[package]` 而没有 edition 的 0 份；`git grep -n '^\[patch' -- '*Cargo.toml'` 退 1、0 处。攻方腿没写模型（它说开工后草稿目录的写者锁被接手），脚本那一侧怎么判是它读代码推的；它留下的探针目录 `probe/`、`probe-patch/`（336K、108K，没有进程在用）主 agent 已删。

## 二、判决

1. **自证成功句残留（坐实）**：第 5184 行与文件头、代码都相反，改。
2. **路径不归一误退 3（坐实，cargo 一侧量过）**：真 cargo 把 `./tests/case_a.rs` 归一成 `tests/case_a.rs`（判决引的 `dot-slash-path` 那一行），脚本按字符串比会判自动目标被顶掉、误退 3。方向偏保守（多拒不漏），照修。
3. **没写 edition 的包（坐实于 cargo 一侧，本仓 0 处）**：攻方腿量的 cargo 行为，主 agent 没另量；本仓今天没有这种包，照修，修法与真 cargo 的对拍留作已知没验。
4. **`[patch]` 换成工作区包（本仓 0 处）**：照修。
5. **三处缺格（推不出→补格）**：改 `[[bin]]` 段只挑那个 bin、只改 bin 根文件只挑那个 bin、指纹那一路认 `workspace = true`，各补一格自证。
6. **按根文件比只在同一种段里比**：真 cargo 对拍里跨种段不替（`bin-root-as-test` 两个目标都在），实现对，不改。
7. **前两轮报的数**：验证包测试今天 483 份（第二轮报 481 是那时那棵树的数，结论不受影响）；写了 `CARGO_BIN_EXE_` 的集成测试全仓 4 份、都不在验证包里，这是用户定案「先不集成测试，按改动测」下今天会漏挑的量。11160 对 19480 是玩具树的数，第二轮判决已写只当方向、不外推，攻方腿另推了一条成因（debug = 0 时 cargo 默认剥掉可执行里的调试信息），没量，不改判。

8. **目录形的段不写 path 误退 3（坐实，接续那一条攻方腿打中、主 agent 复跑）**：前一条攻方腿停在交接点，接续的那一条在它原文上续写（`research/prompts/merge-verify-disk-governance-r3-opus-output.md`，sha256 e7a3cf1c9b3a1928…），模型主 agent 拷到 `~/.local/share/singlefs/work/merge-verify-disk-governance-r3-main/opus-model` 在副本里交，结局 type=succeeded exit=0、`name=summary matched=48 mismatched=0`（被判脚本 `name=judged_script sha256=5dbf51739d699464…`）。`name=dirform_harness_cargo … targets=…test:d:tests/d/main.rs … verdict=matched`、`name=dirform_cargo_list exit=0 hits=1`：真 cargo 认 `tests/d/main.rs` 是测试目标；而 `name=dirform_harness_root exit=0 refusals=1 refused_unlanded_d=1`、`name=dirform_bin_root … refused_unlanded_tool=1`、`name=dirform_body_root … refused_no_items`：段落到目录形根文件、或只改那份根文件的正文，脚本都判落不到测试目标、退 3。根在 `scripts/gate/rerun-failed-tests.py` 的 `test_targets_of_package` 只认 `tests/<名>.rs`，合入后验证照它判，两处算法不一致。方向偏保守（多拒不漏），本仓今天走得到，照修。运行目录（1.6M）已删。

## 三、改法

- 目录形：`scripts/gate/rerun-failed-tests.py` 的 `test_targets_of_package` 照 cargo 认 `tests/<名>/main.rs` 与 `src/bin/<名>/main.rs`，合入后验证挑目标与它共用这一份，自证补一格（段落到目录形根文件、只改那份根文件的正文，都挑出那个目标），各配一条弄坏开关。零轮。

- 第 5184 行改成与文件头一致（只挑那个 bin、不另挑集成测试）。
- 两侧目标集合与 `[[test]]`/`[[bin]]` 段的 path 先归一（去掉开头的 `./`、合并多余的分隔符，相对包目录解析）再比。零轮。
- 没写 edition 的包照 cargo 按 2015 处理：有显式同类段时不认同类自动目标。零轮。
- build 算包与指纹两路读工作区根 `[patch.<源>]` 里带 path 的项，当作 path 依赖。零轮。
- 补三格自证（判决 5）与新判法各配一条弄坏开关。

这是第三轮，这几处修补换上之后不再开下一轮，判决里标零轮；改的是 `scripts/` 下的治理脚本，不是产品代码，不做 A/B。攻方腿提的「两侧目标集合与依赖图都改用 cargo metadata 算」不采：要对补丁前后两棵树各起一次 cargo，合入后验证的快照里改前那一棵不在，零轮、没实现。

## 交用户的

- 用户定案「先不集成测试，按改动测」下，写了 `CARGO_BIN_EXE_` 的集成测试全仓 4 份、都不在验证包里，改了它们起的 bin 时合入后验证不挑它们，要到提交时的整轮门禁才跑。

## 这一轮被判过的文件

- research/prompts/_merge-verify-disk-governance-r3-body.md：背景材料；两条腿照它的「问题」两条判
- scripts/gate/merge-and-verify.sh：删掉挑集成测试那一支在代码里没有残留、自证成功句第 5184 行有残留；按根文件比只在同一种段里比与真 cargo 一致，路径不归一误退 3；没写 edition 的包与 `[patch]` 两处缺；改法 B、C 与用户定案的实现成立，三处缺自证格
- .claude/gate.d/break-switch-replay-files.tsv：第 72 行条数 123 与门禁开关复跑「复跑了 123 条开关」一致

## 回看决策

不涉及决策：这一轮判的是合入后验证脚本与开关复跑登记表的条数，都不碰 `.claude/kb/decisions/` 下的任何一条。
