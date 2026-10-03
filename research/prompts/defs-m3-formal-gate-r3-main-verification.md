# 第五道重型门禁写进规则与定义：定义三方第三轮判决（2026-10-01）

<!-- doc-lint:not-numbers G9 G10 G11 J2 K1 K2 K3 K4 L1 L2 L3 L4 Z1 Z2 -->

第三轮是最终轮（`.claude/rules/three-way-inference.md`「多轮」）：这一轮里新冒出来的改法记零轮、进第四节交用户的表，不再开轮。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 | `research/prompts/_defs-m3-formal-gate-r3-body.md`、`_defs-m3-formal-gate-r3-background.md` | — |
| 开工快照 | `research/prompts/defs-m3-formal-gate-r3-snapshot/sha256sums.txt`（9 份） | 交齐之后 `sha256sum -c` 全 OK |
| 云端辩方（Sonnet） | `research/prompts/defs-m3-formal-gate-r3-sonnet-output.md`（216 行，sha256 886047c7…1962）；模型 `…-sonnet-model/` | 没复跑：它的 L3 那一处与攻方 L3-a 是两份互不共享代码的探针，结论同向 |
| 云端攻方（Opus） | `research/prompts/defs-m3-formal-gate-r3-opus-output.md`（422 行，sha256 7a1ae9b1…84a0f4）；模型 `…-opus-model/` | 主 agent 在 `/tmp/claude-1000/defs-m3-formal-gate-r3-main-check/` 照 `rerun.sh` 复跑：`l1-output.txt`、`l1-K-output.txt`、`l1-argv-demo-output.txt`、`l3-judge-output.txt`、`l3-K4-output.txt` 与入库的逐字节相同；`l1-e165-parse-output.txt` 只差草稿路径，`l2-output.txt`、`l2-K3-output.txt` 只差草稿路径与计时；结论行原样：「T2-VERUS_ROOT=B 退 1 "verified": 0」「Z1 VERUS_ROOT 未设 → 拷贝 B（宏库截断，T2 编不过） q1 退 0 q3 退 10」 |
| 引文 | 两份腿报告 | `python3 research/scripts/cite-check.py <两份报告> --root . --background research/prompts/_defs-m3-formal-gate-r3-background.md`：「✓ 核了 55 处引文，对上 55 处，没判 0 处」 |
| 看门狗检出 | 攻方脚本 `l2-install.sh` 第 44 行 `verus-proof-check.py tool-versions $COPY` 被重型闸拒 | 攻方报告 L1-e 判误拒没打中；那一处是子命令后面跟变量、闸按「认不出就算干活」拒，方向安全，记账 |

## 二、逐格判决

| 格 | 辩方 | 攻方 | 判决 |
|---|---|---|---|
| L1 G9 | — | L1-a 打中：preflight 摘掉整词 `--force`，闸不摘，`e165-pin-red-examples.py --replay --target --force --check` 判成 check 放行而真跑 replay；L1-b `python3 -c` 改 `sys.argv` 或直接调函数照放；L1-c `__import__` / `importlib` 照放（广谱变异同形）；L1-d 记账；L1-e 误拒没打中 | **L1-a、L1-b 打中，主 agent 复跑坐实**（`l1-output.txt`、`l1-K-output.txt` 逐字节相同）。改法攻方 K1（判参数前照 preflight 摘 `--force`）、K2（`-c` 的代码里点到这两份脚本就一律算重型）——**零轮**。L1-c 与广谱变异同形，不分辨臂，并进共用库盲区那一批 |
| L2 G10 | — | L2-a、L2-b 没打中（strace 看到子目录里打开的 0 个、往安装目录写 0 次）；L2-c 打中：`VERUS_ROOT` 指到宏库截断的拷贝 Verus 编不过、`VERUS_USE_RUSTUP=0` 退 127，`tool-versions` 那一行都不变、两条问法都复用；L2-d 部分罩着（rustup 工具链内容换掉推的、没量） | **L2-c 打中，主 agent 复跑坐实**（第一节攻方那一行）。改法攻方 K3（`tool-versions` 另记 `VERUS_ROOT` 与 `VERUS_USE_RUSTUP`）——**零轮**。L2-d 记账 |
| L3 G11 | 崩溃验证员三种格走得通；门禁分诊「全是没跑成」那一支字面走不通（每道都多一行汇总 ✗、不写「内存包装退 25x」，永远记红、归「分不清」，方向安全），是 G11 自己打中自己；Verus 一侧外部杀进程（137）出路仍指「修证明」 | L3-a 同一处打中（量过，合成阶段上真的格共用库与判法打印函数）；L3-b 轻；L3-c 推的：e165「装置被信号停了」那一行写着不算判定、两份定义照字面记真红，e165 的退 253 与 gate-triage「formal-verification 的 253 按红记」字面冲突 | **L3 那一处两条腿独立同向打中**。改法攻方 K4：门禁分诊的全称只数格自己打的 ✗，不算汇总行与包装自己打的 ✗（它在合成阶段上量到全是没跑成判没跑成、混合与全红仍判红）——**零轮**。Verus 一侧 137 的出路与 E165 一侧同样改（**零轮**）；L3-c 记账 |
| L4 已定与记账 | 第一轮 D1–D5、第二轮 E4 没被推翻；记账四处没落点（J2 写法没进 `records/2026-09-16-subagent拆分提案.md` 第四十三节第 1 行；第 1 步停下频率与 QEMU 两处；排队等待最坏约 12 小时（推的）；3-12 列的旧字面）；第二轮两个数今天核不动（文件被 3-12 改过） | — | **前两轮已定的几格撑住**。四处记账照辩方落：J2 并进第四十三节那一批（与别的会话对过之后），其余三处进 `.claude/kb/checks-owed.md` 新行（排队等待那一行写明推的、没量） |

## 三、三轮结论

- 第一、二轮判出的改法（D1–D5、E1–E4 采的那几处）三轮里站住：第三轮的两条腿都没推翻它们，第三轮打中的都是那些改法的边角。
- 第三轮打中的四处（L1-a、L1-b、L2-c、L3）都有攻方量过的改法草稿，按规则不再开轮，写进第四节交用户。

## 四、交用户的（零轮）

| # | 改法 | 依据 | 方向 |
|---|---|---|---|
| 1 | K1 + K2：闸判参数前摘 `--force`；`python3 -c` 的代码里点到这两份判法脚本就算重型 | L1-a、L1-b，复跑坐实 | 不改：子 agent 换写法能绕过重型闸跑这一道 |
| 2 | K3：`tool-versions` 另记 `VERUS_ROOT` 与 `VERUS_USE_RUSTUP` | L2-c，复跑坐实 | 不改：换了 Verus 读的宏库照样复用旧判定 |
| 3 | K4：门禁分诊的全称只数格自己打的 ✗；Verus 一侧 137 的出路照包装文件头 | L3，两条腿独立同向 | 不改：全是没跑成也记红、归「分不清」（方向安全，分诊多一步人工） |

不涉及决策：这一轮只判规则、定义、钩子与门禁复用的写法，没有一条推翻或确立决策分项。
