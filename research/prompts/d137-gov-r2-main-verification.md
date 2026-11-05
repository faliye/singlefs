# d137-gov-r2：一百三十七节那一轮的治理文件改动——第二轮判决（2026-10-29）

<!-- doc-lint:not-numbers R1 R2 H1 H2 H3 H4 H5 H6 H7 H8 H9 H10 H11 H12 H13 S1 S2 S3 S4 S5 S6 S7 A1 A2 A3 A4 M1 M2 M3 M4 M5 M6 W1 W2 U1 D1 D2 C1 Y014 Y029 Z021 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_d137-gov-r2-body.md` | 见快照 |
| 辩方腿（Sonnet，R1） | `research/prompts/d137-gov-r2-sonnet-output.md`，模型目录 `d137-gov-r2-sonnet-model/` | cda13bc2 |
| 攻方腿（Opus，R2，没交付） | `research/prompts/d137-gov-r2-opus-output.md`：读材料时被安全分类器截停，三项都没给结论 | 1e21d707 |
| 攻方论证腿（Opus，R2 里不造绕过写法的几格） | `research/prompts/d137-gov-r2-opus-b-output.md`，模型目录 `d137-gov-r2-opus-b-model/` | eb4b7593 |
| 被判文件快照 | `research/prompts/d137-gov-r2-snapshot/files-sha256.txt`（18 份） | — |

攻方腿没交付之后，主 agent 弹窗，用户 2026-10-29 定「只派论证腿攻不绕的格（推荐）」：钩子漏拒那一半这一轮没有专门的攻方腿攻，只有辩方腿复核第一轮时顺带量到的那几条（第二节 R1「修补后仍漏」）。

开工快照倒推与引文核对（主 agent 跑的原样命令）：

```
python3 scripts/agents/snapshot-tree.py d137-gov-r2 --out <草稿目录>/d137-gov-r2-tree
python3 scripts/agents/cite-check.py research/prompts/d137-gov-r2-sonnet-output.md research/prompts/d137-gov-r2-opus-output.md research/prompts/d137-gov-r2-opus-b-output.md --root <草稿目录>/d137-gov-r2-tree --background research/prompts/_d137-gov-r2-body.md
```

快照点名 18 份，倒推出 15 份、sha256 都对上；倒推不出 3 份：`scripts/hosts/detached-run.sh`（快照 0a9d3e70，此刻 ac015ba4）、`scripts/mutation/crates-mutation-rows.py`（快照 5f83450d，此刻 5e568d92）、`scripts/gate/verus-proof-check.py`（快照 f9267b9a，此刻 fe657f38）。三份都是取快照之后别的会话改的（HEAD 没变，仍是 59ce5a865；这一轮在三份里的改动都还在：verus 的预计秒数与 memory-serial-declared 正则各 1 处、detached-run 的 owner-session 17 处、crates-mutation-rows 的 serial-expected-seconds-unwritten 5 处）。腿引到这三份的几处（攻方论证腿 M6 引 crates-mutation-rows 的派活声明）主 agent 逐处人工核：派活那一段的预计秒数写法与 3098 这个数在此刻那一份里都还在。引文核了 38 处、裸行号 38 处：对上 28 处，对不上 0 处，没判 48 处（裸行号没有引文可比 27 处）。

主 agent 现喂（主 agent 身份、不带前缀、cwd 在会话草稿目录，喂此刻的 `.claude/hooks/heavy-test-guard.sh`）：

| 喂法 | 退出码 | 该怎样 |
|---|---|---|
| `n=5; sed -n "$((n-3)),$((n))p" f.txt` | 2 | 放（只打印行，过度修补 A） |
| 对照 `sed -n '2,3p' f.txt` | 0 | 放 |
| `sed '1a\<换行>hello' f.txt` | 0 | 放（B 没复现，辩方腿的写法另核） |
| `python3 -c 'import subprocess; x="f.txt"; subprocess.run(["cat", x])'` | 2 | 放（命令词字面，过度修补 C） |
| `printf "echo hi\n" > x.sh; cd "$PWD" && bash ./x.sh` | 2 | 放（过度修补 F） |
| `bash -O expand_aliases -c $'alias t="cargo test --all"\nt'` | 0 | 拒（修补后仍漏 Y014） |
| `if true; then bash; fi < <一行 cargo test --all 的脚本>` | 0 | 拒（修补后仍漏 Y029） |

主 agent 自己撞上的误拒另两条：python heredoc 里 `subprocess.run(["python3","scripts/kb/replace-once.py",…, 变量])` 被拒、`sed -n "$((n-3)),$((n))p"` 被拒（与 A 同一类）；攻方论证腿顺手记的 `sed -n "$(…),+45p"` 同类。

另：辩方腿与两个别的会话转来的读数指出，第一轮修补件 3b 在 `scripts/hosts/memory_peaks_lib.py` 自证里用 `globals()["slice_directory_of"]` 换函数，`scripts/gate/lib_function_units.py` 判成 globals-lookup 拒算，verus-proof-check.py 自证 24 格与 lib_verdict_library.py 一格因此红。主 agent 照其中一个会话的调查员给的补丁改成 `global slice_directory_of` 直接赋值，三份自证现跑都 rc=0。

## 二、逐条判决

### R1　复核第一轮判决（辩方腿）

| 第一轮条目 | 辩方腿判 | 主 agent 判 |
|---|---|---|
| H1、H2、H4、H6–H11、A2、S1–S7、A3 | 撑住 | 撑住 |
| H3 | 撑住带保留：判决里那条一行写法真 bash 不起 cargo，换行写法才是真漏 | 采保留；修补已认换行写法，第一轮判决表里那一行喂法记成不准 |
| H5 | 撑住带保留：`;` 写法站得住，`cd 不存在 && …`、`|| exit` 守着的写法真 bash 不起脚本，修补照拒是误拒 | 采，进修补（偏严误拒） |
| A1 | 撑住带保留：修补上界偏松（核数 4、窗口 600 秒、声明 300 秒时判只记，按声明段占一个核重算该叫醒） | 采，进修补 |
| H12 | 修补改出与用户「退 2 照拒」不一致的三格 | 用户 2026-10-29 定「退 2 一律照拒」，进修补 |
| H13 | 本机有 python3，没复现 | 修补件自己在去掉 python3 的 PATH 下复现过，照它的读数采 |
| A4 | 仓里没有真实调用 `detached-run.sh start` 的脚本，只是潜在 | 采为潜在；修补保留（认不出归属照叫醒，偏严） |
| 第一轮「撑住」的格有没有被修补改坏 | 第一轮正推腿 171+232 格两棵树对跑，零格由符合变不符合 | 采 |
| 修补后仍漏 | Z021（alias + eval）、Y014、Y029 | Y014、Y029 现喂坐实；Z021 采为线索；进修补 |
| 过度修补 | A sed 算术展开、B sed `a\` `i\` `c\`、C python 字面命令词带变量参数、D 字面列表先赋变量或字面串 `.split()`、E awk `cmd=` 与 `| getline`、F `cd` 之后起相对脚本、G `bash -lc` 每次约 2.8 秒并多记检出；真实命令语料 30,595 行修补前放行、修补后拒 1,893 条（扣回放假象 1,621 条，sed 1,262、python 264），无偏随机 5,000 条约 0.1% | A、C、F 现喂坐实；B 照主 agent 的写法没复现、采为线索；D、E、G 采为线索；进修补（认得出起的是什么就放，这是用户口径「认不出的一律拒」的另一半） |
| 方法 | 第一轮判决没量偏严的代价 | 采：这一轮修补要拿辩方腿的真实命令语料核误拒 |

### R2　攻修补本身与从未被看过的地方（攻方论证腿；钩子漏拒那一半没攻）

| # | 腿报的 | 主 agent 判 |
|---|---|---|
| M1 | 看门狗修补核对开关时只比失败行的前 48 个字 | 采；看门狗整表开关复跑归提交前门禁 code-tooling 的 break-switch-replay 格（照那一格的判法） |
| M2 | 看门狗自证头一趟红 1 格、再跑两趟到绿 | 采为线索；修补件里复现那一格，查清是不是负载偶发 |
| M3 | `heavy-test-guard.sh`、`lib_shell_words.py`、`bash-command-detector.sh` 三份钩子不在 `.claude/gate.d/break-switch-replay-files.tsv`（主 agent grep 现数 0 行），提交前门禁不会替它们整表复跑 | 打中，采；进修补（登记三行与限时） |
| M4 | 第三件在 dev 树上核开关，底子 25 格本来就红，多红的格看不见 | 采；修补件在主工作区（底子全绿）重核新开关 |
| M5 | 「回归全拒」只喂点名反例，第一轮正推腿那几批「撑住」的样本没复跑 | 采；主 agent 在第三轮开工前用最终钩子跑四份全量复跑 |
| M6 | crates-mutation-rows 派活那一段 3098 秒取自被别的声明截断的 2065 秒下界 | 采；进修补（重量那一段的整段时长） |
| W1 | 告警记录里读数取整，看门狗拿取整后的读数判，与判的那一行叫不叫醒相反（20 万次取样 43 次与 34 次） | 采；进修补 |
| W2 | 同一份声明文件里几个程序的声明互相截断、互相续命，看门狗比 7200 秒那条线晚叫醒 6800 秒以上 | 采；进修补 |
| U1 | 判定行那一半按名字读此刻登记，同名单元重起后本会话旧一次的判红行被当别人的 | 采；进修补 |
| D1 | admission.py 退 0 但实验键拒算（它自己说判不了没变）时直接放行，不记检出 | 采；进修补（照「判不了放行并记检出」） |
| D2 | admission.py 自己报的「调用有错」退 2 被当 python 用法错放行；钩子环境里设了强制重跑理由时退 77 变退 0、不拒不记 | 前一半随用户「退 2 一律照拒」一并改；后一半采为线索，进修补 |
| C1 | `.claude/main-agent.md` 第 70 行两条处置会落在同一行（声明过期满一个窗口后，既「没有声明串行段、该停下那一趟」又「拖过 7200 秒、查进度照旧跑」） | 采；进修补（定义） |

## 三、怎么办

这是第二轮。打中的合成一批修，修完开第三轮（最终轮）：第三轮分工照 `.claude/rules/three-way-inference.md` ——前两轮站住的形态撑不撑得住、未定项清单的完整性、报出的数能不能核；只攻前两轮站住的形态，这一轮修补新冒出的零轮形态写进第三轮判决的交用户表。修补按文件聚簇：

- 钩子与共用切词：Y014、Y029、Z021；H5 误拒；过度修补 A–G 与 `sed -n "$(…),+45p"`；M3 三份钩子登进 break-switch-replay-files.tsv；M4 在主工作区重核新开关。改完拿辩方腿的真实命令语料（`research/prompts/d137-gov-r2-sonnet-model/`）重喂，误拒条数照实报。
- 派发闸：H12 退 2 一律照拒（拒绝说明分两种出路）；D1；D2 后一半。
- 看门狗与 core_usage：A1 上界；W1；W2；U1；M2 那一格。
- 串行声明：M6 crates-mutation-rows 派活那一段重量。
- 共用约束与定义：C1，连同上面几件要跟着改的句子。

## 回看决策

不涉及决策：被判的是项目的协作钩子、定义、规则与研究脚本，不碰 `.claude/kb/decisions/` 下任何一条。

## 这一轮被判过的文件

- `.claude/hooks/lib_shell_words.py`：R1 打中（修补后仍漏 Y014、Y029、Z021；过度修补 A–G）
- `.claude/hooks/heavy-test-guard.sh`：R1 打中（同上，H5 误拒）；R2 M3（没登进开关复跑表）
- `.claude/hooks/bash-command-detector.sh`：R2 M3（没登进开关复跑表）
- `.claude/hooks/lib_heavy_tests.py`：R1 撑住（第一轮撑住的格没被改坏）
- `.claude/hooks/runner-dispatch-guard.sh`：R1 H12（与用户口径不一致）；R2 D1、D2
- `.claude/agent-common.md`：R2 格 5 撑住（新句子照规则纪律查过）
- `.claude/main-agent.md`：R2 C1 打中
- `scripts/agents/agent-watch.py`：R2 W1、U1、M1、M2
- `scripts/hosts/core_usage.py`：R1 A1 上界；R2 W1、W2
- `scripts/hosts/detached-run.sh`：R2 U1 牵到（owner-session 登记）
- `scripts/hosts/multi-host-run.sh`：两条腿都没打中
- `scripts/crash/gpu-quiesce-run.sh`：两条腿都没打中
- `scripts/crash/crash-amplification.sh`：两条腿都没打中
- `scripts/mutation/crates-mutation-rows.py`：R2 M6
- `scripts/gate/test-targets-parallel.py`：两条腿都没打中
- `scripts/gate/verus-proof-check.py`：两条腿都没打中（第一轮修补件 3b 在 memory_peaks_lib.py 带出的拒算已改）
- `.claude/gate.d/break-switch-replay-files.tsv`：R2 M3
