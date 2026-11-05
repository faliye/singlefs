# d137-gov-r3 辩方腿（格 R1）：前两轮判决与判定过程的复核（2026-10-31）

辩方腿，只复核、不撤销也不维持判决。被判文件此刻的 sha256 与开工快照 `research/prompts/d137-gov-r3-snapshot/files-sha256.txt` 逐份相同（17 份，量的，`sha256sum` 现算）；下文引到的几份：`.claude/hooks/heavy-test-guard.sh` 846c5259、`.claude/hooks/lib_heavy_tests.py` 4ab8f57d、`.claude/hooks/bash-command-detector.sh` 207791db、`.claude/hooks/lib_shell_words.py` d33ec04b、`.claude/hooks/runner-dispatch-guard.sh` 2b5bef5f、`.claude/agent-common.md` a2ad83ba、`.claude/main-agent.md` 9e88ebfb、`scripts/hosts/core_usage.py` a7d613f8、`scripts/agents/agent-watch.py` 1970aa37、`scripts/mutation/crates-mutation-rows.py` 7e1ef696、`.claude/gate.d/break-switch-replay-files.tsv` 0600dcd4。两份判决此刻 557c7443（第一轮）、096843ca（第二轮）；分类报告 `~/.local/share/singlefs/work/d137-replay-classify/result.md` 754c869b。

## 复跑

模型目录 `research/prompts/d137-gov-r3-opus-defense-model/`。喂钩子只喂 JSON，不执行被判的命令；钩子拷贝放在草稿目录的一棵树里（`.claude/hooks` 真拷贝，别的回仓的符号链接），cwd 是草稿目录里的样本目录，每份钩子放进自己的进程组起。复跑命令（工作目录随意，冻结副本给仓根即可：`rerun_steps` 只拷它的 `.claude/hooks`，此刻与快照逐份相同）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/d137-gov-r3-opus-defense-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的草稿目录>
```

这一次复跑的原样输出在 `out/rerun-2.txt`：五份输出的 `name=` 行与 `out/` 里存的逐行相同（`name=rerun_compare … same=yes` 五行）。A 类形态计数另跑：`python3 -I tools/a-class-shape.py ~/.local/share/singlefs/work/d137-replay-classify/an/real_new.json`，与 `out/a-class-shape.txt` 逐行相同。

| 文件 | sha256 |
|---|---|
| tools/feed.py | fe35390eacecfef75377687c9ba841be82a42d1733569bed87c868408e429d28 |
| tools/feed-dispatch.py | 28d9960922378f452dda0cb8bdbe4cb3a36861f00b2af6ed296f42dd158655fb |
| tools/feed-python-missing.py | 5e78054ff0615b34298bb4db9535b14f03ac222df3063e63689e8990d7594013 |
| tools/a-class-shape.py | 3d0333fe6c27854882127511b3a078fdbad557d11da27a70163a2e755721af2f |
| tools/rerun.sh | 075663673d9f328281bfa9491c784b110ccb188a884bc6680088d1a67627ecd8 |
| tools/setup-samples.sh | 2bbb2b8acc08162130b95d55d0a236f9874e09ccdcf897bc52db26bfe096f1ec |
| tools/cases.json | 2140a3c54514c7c32d7145186001d1bf8802b022387a7165e93da835d6c8cd04 |
| tools/cases-2.json | a60d6b73da63b8bebc4e0d94514bb86417275093cedf9383392fb4d54d6d3d32 |
| tools/cases-3.json | 34c95e3272c1f56bc6cbe2ee626664f961818e173776474e1a759e67fdb00115 |
| tools/admission-samples/*.py（8 份，拷自 `d137-r2fix-dispatch/samples/`） | 见 `sha256sum tools/admission-samples/*.py` |
| out/feed-1.txt | c7d600676a260d85f91a4dade3462feb527549135b25c80c52eed526fd64f549 |
| out/feed-2.txt | 61e2f0b26fa924c9ff083ca8feb9ceb1674303a80da2d0a3b80dcb8b3e3a1118 |
| out/feed-3.txt | c69b15920dd8b244aa749ca99c69c15f6f3d8ee17ba34733f1220d2171d8ea70 |
| out/feed-dispatch.txt | 9f98f1611f37657a3899e975a192e0eb5ad26d8efa9007910147ccc244445b1f |
| out/feed-python-missing.txt | 643e5d2b6e91420f64f58581e639215fd1042589bcbc827a538f216927aa87e7 |
| out/a-class-shape.txt | d2aa6148342c13c6594fd0dbae270a0570cd9f0ebe564c1bb2dbc3464ec80a1f |
| out/rerun-2.txt | b48ac93bd5f375907b8de12622487e0c7955f751823a89d6185bc2646abe103e |

下文「喂 `<id>`」指 `out/feed-*.txt` 里 `name=feed id=<id>` 那一条（量的，喂的是此刻主工作区五份钩子的逐字节拷贝）。

## 一、前两轮每一条「打中，采」「采为线索」「撑住」修补之后站不站得住

判定三档：站住（判得够得着、修补之后此刻照判决的方向）；带保留（结论站住，但判决那一行的证据、喂法或范围有一处不成立）；够不着（判决那一行的结论此刻没有观测撑着）。第一轮判决的行号指 `research/prompts/d137-gov-r1-main-verification.md`，第二轮指 `research/prompts/d137-gov-r2-main-verification.md`。

| 判决条目 | 原判在哪 | 辩方判定 | 证据（量的，除另注） |
|---|---|---|---|
| H1 带值短选项合写 | `d137-gov-r1-main-verification.md:42` | 站住 | 喂 `r1-line27-combined-short-options` 主 agent 退 2（判决当时退 0）；拒的是 cargo test --all 本身，不是别的规矩 |
| H2 trap | `d137-gov-r1-main-verification.md:43` | 站住 | 喂 `r1-line29-trap` 退 2 |
| H3 alias | `d137-gov-r1-main-verification.md:44` | 带保留 | 判决的喂法（第 30 行一行写法）真 bash 不起 cargo（第二轮第 49 行已记）；此刻喂 `r1-line30-alias-one-line` 退 0、`e4-control-alias-newline` 退 2。「打中」靠的是第二轮补的换行写法，第一轮那一行的证据不成立 |
| H4 标准输入 | `d137-gov-r1-main-verification.md:45` | 站住 | 喂 `r1-line31-dev-stdin` 退 2；「其余几种」修补件先复现（`d137-r1fix-hooks/result.md` 第 44 行：6 种改前退 0）再改 |
| H5 cd 失败之后的相对脚本 | `d137-gov-r1-main-verification.md:46`、`d137-gov-r2-main-verification.md:50` | 带保留 | 漏拒那一半站住（喂 `control-cd-missing-heavy` 退 2）；`&&` 与 `\|\| exit` 守着的此刻放（喂 `h5-cd-missing-and-relative`、`h5-cd-missing-or-exit` 退 0）。修补的形状仍比漏拒宽：被起的脚本是 cd 之后同一条命令里 heredoc 写出的，cd 成不成都起同一份内容，照拒（第三节第 1 处） |
| H6 `~+` | `d137-gov-r1-main-verification.md:47` | 站住（照修补件读数） | `d137-r1fix-hooks/result.md` 第 46 行改前复现退 0；`d137-r1fix-hooks-2/result.md` 第 30 行改后三条退 2。我没现喂 |
| H7 多源 cat 等 | `d137-gov-r1-main-verification.md:48` | 站住（照修补件读数） | 修补件先复现（`d137-r1fix-hooks/result.md` 第 47 行）；改法与用户 2026-10-29「cat 不看源文件在不在…照拒」同向 |
| H8 改名的 shell | `d137-gov-r1-main-verification.md:49` | 站住 | 喂 `r1-line32-renamed-shell` 退 2 |
| H9 别的解释器 | `d137-gov-r1-main-verification.md:50` | 站住 | 修补件先复现 15 种退 0；修补带出的两处误拒（E6 列表相加、E7 只 compile）此刻都放：喂 `e6-python-list-concat`、`e7-python-compile-only` 退 0 |
| H10 引号里的花括号 | `d137-gov-r1-main-verification.md:51` | 站住（照修补件读数） | `d137-r1fix-hooks-4/result.md`：引号里的 `{cargo,test,--all}` 改前退 2、改后退 0 |
| H11 拒绝说明过时 | `d137-gov-r1-main-verification.md:52` | 站住 | 「分不出花括号在不在引号里」在 `.claude/hooks/bash-command-detector.sh` 里 `grep -nF` 0 处 |
| H12 准入退 2 | `d137-gov-r1-main-verification.md:53`、`d137-gov-r2-main-verification.md:52` | 站住 | 第一轮修补往放行改、与用户口径相反，第二轮按「退 2 一律照拒」改回；喂 `name=dispatch script=nonexistent.py` 等四种退 2 都 refused=True（`out/feed-dispatch.txt`） |
| H13 python3 不在 PATH | `d137-gov-r1-main-verification.md:54`、`d137-gov-r2-main-verification.md:53` | 站住 | 第二轮只照修补件自己的读数采；这一轮另喂一次：PATH 只有 cat、date、mkdir、dirname 时两份钩子都退 0、各记 1 条检出（`out/feed-python-missing.txt`） |
| 第一轮「撑住的」两段 | `d137-gov-r1-main-verification.md:56`、`:70` | 带保留 | 判决自己写「主 agent 没逐条复跑，照腿的读数记」；第二轮辩方腿在两棵树上对跑 171+232 格（第二轮第 55 行），那两棵树早于第二轮修补与件 13、15、16，此刻的钩子上没人重跑这几批（第二节第 1 条） |
| S1 共用约束写进理由 | `d137-gov-r1-main-verification.md:62` | 站住 | 「进程组组长」在 `.claude/agent-common.md` 里 `grep -nF` 0 处 |
| S2 case 模式行文件头 | `d137-gov-r1-main-verification.md:63` | 站住 | 「case 模式行被切成命令」在 `.claude/hooks/lib_heavy_tests.py` 里 0 处 |
| S3 文件头「照拒」 | `d137-gov-r1-main-verification.md:64` | 站住（照修补件读数） | `d137-r1fix-hooks-3/result.md` 第 64 行改成「源不在→目的照认不出拒」，与 H7 一起 |
| S4、S6、S7 | `d137-gov-r1-main-verification.md:65`、`:67`、`:68` | 站住 | 随 H11、H12、H13 |
| S5 统一叫法 | `d137-gov-r1-main-verification.md:66` | 带保留 | `.claude/agent-common.md` 里旧叫法 0 处；规则文件 `.claude/rules/three-way-inference.md` 第 164 行仍写「会话共用的暂存目录」（不在被判文件里，判决只点了共用约束） |
| A1 声明一次就静音 | `d137-gov-r1-main-verification.md:76`、`d137-gov-r2-main-verification.md:51` | 站住 | 看门狗那一句条件在 `scripts/agents/agent-watch.py` 里 0 处；判法挪到 `scripts/hosts/core_usage.py` 第 582 行 `declaration_excuses_low`，第 193 行声明段按一个核扣 |
| A2 BASH_ENV 进转发函数 | `d137-gov-r1-main-verification.md:77` | 站住 | 判决原喂法此刻被「同一条命令里 printf 写出」那一道先拒（喂 `r1-line33-bash-env-function`），分不出 A2 的机理修没修；改成盘上已有的脚本再喂 `r1-line33-bash-env-function-script-on-disk` 退 2，拒的是脚本里的 cargo test --all |
| A3 两处门槛不一 | `d137-gov-r1-main-verification.md:78` | 站住 | 随 A1；core_usage 那一侧判决当时没核，修补之后两边只剩 core_usage 一处判法 |
| A4 单元归属 | `d137-gov-r1-main-verification.md:79`、`d137-gov-r2-main-verification.md:54` | 站住（潜在） | 第二轮辩方腿说仓里没有真实调用；U1 的会话摘要改法覆盖同一处 |
| 攻方改法零轮 | `d137-gov-r1-main-verification.md:81` | 站住 | 修补没照搬声明占比过半，另定按一个核扣 |
| 第二轮：第一轮「撑住」没被改坏 | `d137-gov-r2-main-verification.md:55` | 带保留 | 对跑的是第一轮修补那两棵树；第二轮修补与件 13、15、16 之后没重跑这 403 格（第二节第 1 条） |
| 修补后仍漏 Y014、Y029、Z021 | `d137-gov-r2-main-verification.md:56` | 不复核 | 用户定「登账留你亲自处理」，派发提示点名不复现 |
| 过度修补 A sed 算术 | `d137-gov-r2-main-verification.md:57`（喂法第 30 行） | 站住 | 喂 `r2-line30-sed-arithmetic`、`r2a-sed-arithmetic-address` 退 0 |
| 过度修补 B sed `a\` | 同上（喂法第 32 行） | 站住 | 主 agent 当时的写法改前就放（修补件说是旧解析的巧合）；辩方腿的 foo / bar 写法此刻放：喂 `r2-defense-sed-append-foo-bar` 退 0、`r2-line32-sed-append-hello` 退 0 |
| 过度修补 C python 字面命令词 | 同上（喂法第 33 行） | 站住 | 喂 `r2-line33-python-cat-variable` 退 0 |
| 过度修补 D、E、G | 同上 | 站住（照修补件读数） | `d137-r2fix-hooks/result.md` 改前 `cases=112 bad=56`、改后 `bad=0`；G 的用时 3.0 秒到 0.2 秒；我没现喂 |
| 过度修补 F `cd` 之后的相对脚本 | 同上（喂法第 34 行） | 带保留 | 判决的喂法原样此刻仍退 2：喂 `r2-line34-exact`（主 agent）与 `r2-line34-exact-subagent` 都退 2，拒的理由换成「是同一条命令里前面的 printf 写的，写法认不出」。修补件把样本换成盘上已有的 `benign.sh`（`d137-r2fix-hooks/result.md` F 那一行），判决那一行写的「放」没做到，修补报告也没提（第三节第 2 处） |
| 方法：拿真实语料核误拒 | `d137-gov-r2-main-verification.md:58` | 带保留 | 语料再喂是在件 13、15、16 之前的钩子上做的（背景材料「二」），之后的放开与收紧没进读数 |
| M1 看门狗只比前 48 个字 | `d137-gov-r2-main-verification.md:64` | 带保留 | 判决把整表复跑交给提交前门禁；登记表第 66 行写「整表复跑归提交前」，这一轮链上没人量过 |
| M2 看门狗自证偶发红 | `d137-gov-r2-main-verification.md:65` | 站住（照修补件读数） | `d137-r2fix-watch/result.md` 查出是 fork 与 exec 之间的时序，改了等法 |
| M3 三份钩子没登开关复跑表 | `d137-gov-r2-main-verification.md:66` | 带保留 | 登记表第 67、68 行登了 heavy-test-guard、bash-command-detector，第 69 行 core_usage；判决点名的第三份 `lib_shell_words.py` 没有行（没有自己的自证，主 agent 删了那一行），它的开关只在别的钩子的自证里被打开 |
| M4 在主工作区重核新开关 | `d137-gov-r2-main-verification.md:67` | 带保留 | 第 67 行登记 heavy-test-guard 七十九条里三条「还没复跑」；第 68 行 bash-command-detector 另有「二十行…短写法开关，这一格读不到、没复跑」 |
| M5 回归只喂点名反例 | `d137-gov-r2-main-verification.md:68` | 够不着 | 判决要「用最终钩子跑四份全量复跑」；背景材料「二」写复跑用的是件 13 之前、件 15 与件 16 之前的钩子，用最终钩子再跑要等统一任务启动器。此刻没有最终钩子上的读数 |
| M6 3098 秒取自下界 | `d137-gov-r2-main-verification.md:69` | 站住 | `scripts/mutation/crates-mutation-rows.py` 第 3112 行写 7200 = min(9272 × 1.5, 7200) |
| W1 读数取整 | `d137-gov-r2-main-verification.md:70` | 站住（照修补件读数） | `scripts/hosts/core_usage.py` 第 683 行 `alert_record` 写原值；我没现跑 |
| W2 声明互相续命 | `d137-gov-r2-main-verification.md:71` | 站住（照修补件读数） | `scripts/hosts/core_usage.py` 第 458 行 `declaration_segments` 按写者分段 |
| U1 同名单元重起 | `d137-gov-r2-main-verification.md:72` | 站住（照修补件读数） | `scripts/hosts/core_usage.py` 第 629 行 `unit_session_note`、`scripts/agents/agent-watch.py` 第 3450 行 `verdict_line_owner` |
| D1 键拒算静默放 | `d137-gov-r2-main-verification.md:73` | 站住 | `name=dispatch script=key-refused.py`：refused=False、noted=True |
| D2 退 2 与残留理由 | `d137-gov-r2-main-verification.md:74` | 站住 | `admission-break.py` 退 2 照拒；`force-rerun-reason.py` 带残留理由仍 refused=True（退 77 那一拒） |
| C1 两条处置落在一行 | `d137-gov-r2-main-verification.md:75` | 站住 | `.claude/main-agent.md` 第 71 行起三条，第 72–74 行按「写了 / 没写」两个词分开，任一行只落一条 |

计数（量的，`awk` 按上表第三列数）：站住 32 行（其中「照修补件读数」9 行、「潜在」1 行）、带保留 10 行、够不着 1 行、不复核 1 行，共 44 行。

`out/feed-dispatch.txt` 里 crashed.py 与 key-refused.py 两条退 2、refused=False：那是派发闸别的规矩拒了这份极简提示（缺批次、预计挂钟这几行），⑨ 自己没拒，不影响 D1 的读法。

## 二、前两轮的判定过程漏没漏

主 agent 现喂的范围（量的，数判决里的喂法表行）：第一轮 8 行（6 条反例、2 条对照），落在 H1、H2、H3、H4、H8、A2；第二轮 7 行，落在过度修补 A、B、C、F 与 Y014、Y029。第一轮 G1 十三条里 8 条只采为线索（H5、H6、H7、H9、H10、H11、H12、H13），G2 里 S2、S3、S5 没核，A3 的 core_usage 一侧没核却写「采」。

| # | 撑在没量过的范围上的结论 | 哪一句 | 能补的那一次观测 |
|---|---|---|---|
| 1 | 「没有腿登记该拒、钩子放的回退」与「第一轮撑住的格没被改坏」 | 第二轮判决第 55 行（两棵树对跑 171+232 格）、背景材料「二」的四份全量复跑 | 四份复跑用的是件 13 之前、件 15 与件 16 之前的钩子；件 15、16 放开了九类误拒与两类零星、件 13 剥开两种派发起法。用此刻四份钩子（846c5259、4ab8f57d、207791db、d33ec04b）重跑四份全量复跑，数 `True拒→False放`，并重跑第一轮正推腿 171+232 格，数「符合变不符合」 |
| 2 | 「误拒九类与零星两类…都已由件 15、件 16 修」 | 背景材料「二」 | 分类报告自己没分 legA-r3 的六份输出：`~/.local/share/singlefs/work/d137-replay-classify/result.md:205`：「legA-r3 x1、x3fam、x3chg、x3kf、x3pre、x3s2 的完整 diff（摘要只显示 12 行）没分类」；同一份第 30 行列了 x1 三族「对->误拒」由 3→12、2→48、0→34（三族合计多 89 格，推的，按那一行的数相减）。用最终钩子跑 legA-r3 的 rerun.sh、diff 不截 12 行，逐格分类这几族 |
| 3 | A 类 207 条「全部是回放假象」 | 分类报告第 76 行 | 同一份第 155 行自己写「97 条有 mkdir / worktree / cp -r / python 之类」：目录在同一条命令里才建，活的时候钩子照样看到它不在、照样拒，不是回放才有的。另外按正则粗分（推的，`out/a-class-shape.txt`），207 条里 26 条被起的脚本是 cd 之后同一条命令里 cat 读 heredoc 写出的（cd 后接 `;` 或换行 13 条、接 `&&` 链 13 条），这 26 条的拒不靠目录在不在（第三节第 1 处）。逐条把这 26 条在目录建好的树里重喂，数退 2 的 |
| 4 | 第一轮「采为线索」的几条修补前都复现过 | 各修补件报告 | 复现量的是钩子的退出码，不是真 bash 起没起：`d137-r1fix-hooks/result.md` 第 68 行「复现已在 live 主工作区确认 rc=0」。H3 的一行写法、H5 的 `&&` 写法都是这样「复现」之后改成拒、第二轮才发现真 bash 不起（E4、H5 误拒）。真 bash 对照（`--truth`，PATH 头上放只记日志的假 cargo）是第二轮修补件才加的（`d137-r2fix-hooks/result.md` 第 23 行）。把 `d137-r1fix-hooks-2`、`-3`、`-4` 三件的样本过一遍那份 `--truth`，数「钩子拒而真 bash 没起重活」的 |
| 5 | 开关「逐个跑过」 | 登记表第 67、68 行 | 第 67 行 heavy-test-guard 三条「还没复跑」；第 68 行 bash-command-detector 文件头另有二十行短写法开关「这一格读不到、没复跑」。照门禁 break-switch-replay 格的判法补跑这 23 条 |
| 6 | 看门狗整表开关复跑 | 第二轮判决第 64 行、登记表第 66 行 | 判决把它交提交前门禁，这条链上没量过；提交前那一趟的 break-switch-replay 格读数就是补的那一次 |
| 7 | 语料误拒「约 36」、C3「约 31」 | 分类报告第 19、159 行 | 按正则粗分、没逐条展开；件 15 改完之后只喂了点名的最小复现。用最终钩子把真实语料 326 条新拒重喂一遍，数还拒的 |

修补件把「采为线索」悄悄当坐实的：没找到。第一轮 H2–H9、A2 修补件先复现（上表第 4 行的限定照旧），H10、H13 各自复现，第二轮 B、D、E、G 修补件在 `cases=112` 里先复现；Z021 没修（用户留着）。性质被改的有一处：第二轮过度修补 F 那一行的喂法，修补件换成了盘上已有脚本的样本，判决那一条命令此刻照旧拒（第一节 F 行、第三节第 2 处）。

## 三、替被修掉的一方辩护：修补里拒掉了正当命令的地方

喂法：上面「复跑」那一节的树与样本目录（`fx/light.sh`、`s.sh` 各一行 `echo hi`，`heavy.sh` 一行 `cargo test --all`），只喂 JSON。下面的「真 bash 会怎样」都是推的（照 bash 的语义推，没在真 shell 里起这些命令）。

### 第 1 处　H5 修补：cd 之后用 heredoc 写出再起的脚本，目录不在就照拒

最小复现（implementation-writer，heavy-test-guard 846c5259）：

| 喂 | 命令 | 退出码 |
|---|---|---|
| `over-cd-missing-then-heredoc-script` | `cd /nonexistent-d137r3; cat > r.sh <<'EOS'` ⏎ `echo hi` ⏎ `EOS` ⏎ `bash r.sh` | 2 |
| `over-mkdir-semicolon-cd-heredoc` | `mkdir -p made-d137r3; cd made-d137r3;` 接同样三行 | 2 |
| `control-mkdir-and-cd-heredoc` | `mkdir -p made-d137r3 && cd made-d137r3 && cat > r.sh <<'EOS'` ⏎ `echo hi` ⏎ `EOS` ⏎ `bash r.sh` | 2 |
| `control-mkdir-and-chain-heredoc`（对照） | `… && cat > r.sh <<'EOS' && bash r.sh` ⏎ `echo hi` ⏎ `EOS` | 0 |

拒绝原文：`research/prompts/d137-gov-r3-opus-defense-model/out/feed-1.txt:23`：「✗ 认不出起什么的写法被拒：bash / sh 起的脚本 r.sh 要先 cd 到 /nonexistent-d137r3，那个目录此刻盘上不在（同一条命令里前面 mkdir 建的也照不在算），cd 会失败而这一条照跑」。钩子里那句在 `.claude/hooks/heavy-test-guard.sh:1603`：「同一条命令里前面 mkdir 建的也照不在算」。

认得出的理由：前两条里 `cat > r.sh` 与 `bash r.sh` 在 cd 之后都照跑，cd 成不成都在同一个目录写、同一个目录起，起的就是 heredoc 那两行；第三条 cd 失败时 `cat` 跟着不跑，`bash r.sh` 落在原来的目录，那里此刻盘上没有 `r.sh`，bash 报找不到、什么都不起。三条在两种走向上起的东西都定得下来，照第二轮判决第 57 行的标准该放：`research/prompts/d137-gov-r2-main-verification.md:57`：「认得出起的是什么就放，这是用户口径…的另一半」。第二轮判 H5「`;` 写法站得住」（第 50 行）对盘上已有的脚本成立，对 cd 之后同一条命令里写出的脚本过宽。

在分类报告「照口径」里的量（推的，正则粗分，`out/a-class-shape.txt`）：A 类 207 条里 26 条是这一形（cd 后接 `;` 或换行 13 条：第 14014、14737、14745、14839、14852、14854、14875、14879、14881、14883、14893、14903、18779 行；接 `&&` 链而 heredoc 之后换行起脚本 13 条）。子 agent 在草稿目录里 `mkdir -p` 一个子目录、`cd` 进去、heredoc 写一份脚本再起，是常见写法（推的，没数频次）；目录在同一条命令里才建，活的时候就拒。

### 第 2 处　文件头 ③：echo / printf 写字面串再起，照「写法认不出」拒

| 喂 | 身份 | 命令 | 退出码 |
|---|---|---|---|
| `r2-line34-exact` | 主 agent 不带前缀 | 第二轮判决第 34 行原样：`printf "echo hi\n" > x.sh; cd "$PWD" && bash ./x.sh` | 2 |
| `r2-line34-exact-subagent` | implementation-writer | 同上 | 2 |
| `echo-literal-no-escape-subagent` | implementation-writer | `echo 'ls' > w2.sh && bash w2.sh` | 2 |
| `control-cat-heredoc-written`（对照） | implementation-writer | `cat > w.sh <<'EOS'` ⏎ `echo hi` ⏎ `EOS` ⏎ `bash w.sh` | 0 |
| `echo-literal-heavy-subagent`（同形起重活） | implementation-writer | `echo 'cargo test --all' > w3.sh && bash w3.sh` | 2 |

拒绝原文：`research/prompts/d137-gov-r3-opus-defense-model/out/feed-2.txt:38`：「✗ 认不出起什么的写法被拒：bash / sh 起的脚本 …/w2.sh 是同一条命令里前面的 echo 写的，写法认不出（不是 cat / tee 读 heredoc、cp / ln / install / mv、cat 源 > 目的、tee 拷出这几种）」。

口径从哪来：第二轮判决第 34 行把这一条判「放」：`research/prompts/d137-gov-r2-main-verification.md:34`：「放（过度修补 F）」。钩子文件头 ③ 把 echo / printf 写出列成认不出：`.claude/hooks/heavy-test-guard.sh:1443`：「同一条命令里先写出、再执行的脚本，写法认不出的（正推腿乙报告第五节 Z4」，共用约束第 95 行同样列了 echo / printf。用户 2026-10-29 弹窗定的表（调度记录 `records/2026-10-29-一百三十七节那一轮调度记录.md` 第 13–16 行）没有 echo / printf 写出这一项。所以「照口径」里 echo / printf 那一批（分类报告 legA-r2 79 格，legB 的 SW、S 两族里同形的几条）撑在更早一轮的 Z4 上，与第二轮判决第 34、57 行的标准相冲。

认得出的那一窄条（推的）：`echo` 不带选项、参数全是单引号里不带反斜杠的字面串，写出的就是参数以空格连起来加一个换行（bash 内建 echo 默认不解释反斜杠）；`printf` 只有一个字面格式串、里面没有 `%`，写出的就是格式串照转义解出来的那几行。照写出的内容判之后，同形起重活的照拒（上表末行此刻是按写法认不出拒，按内容判同样拒在 cargo test --all 上），漏拒那一半不跟着开。

### 第 3 处　「照口径」里认得出、但命令本身什么都不起的三种

| 喂 | 命令 | 退出码 | 真 bash（推的） |
|---|---|---|---|
| `over-cat-fd0-empty` | `cat s.sh 0> d.sh && bash d.sh` | 2 | `d.sh` 只被打开成空文件，`bash d.sh` 跑一份空脚本 |
| `over-cp-brace-operands` | `cp s.sh d{01..02}.sh && bash d01.sh` | 2 | 三个操作数、末一个不是目录，cp 报错退非 0，`&&` 后面不跑 |
| `over-ln-dangling` | `ln -s s.sh sub/l2.sh && bash sub/l2.sh` | 2 | 相对源从 `sub/` 算，链接悬空，bash 报找不到 |

这三种钩子定得下「什么都不起」，照第二轮第 57 行的标准算误拒；但它们本身就是写错的命令，拒的代价近乎零，用户 ① 对「cat 源不在」也定了照拒。辩方只把它们列出来回答「照口径里有没有认得出的」，不主张改。

### 第 4 处　查过、不算过度的

- 分类报告第四节把「python 起的脚本路径非字面」（C4）记成多拦了一步。此刻 `P=t.py; python3 $P` 退 0（喂 `python-script-variable-literal`），赋过字面值的照展开；剩下那 8 条是真算不出的路径，而 python 脚本里有按名字判的重型入口（`scripts/mutation/broad-mutation.py`、`scripts/gate/verus-proof-check.py`），路径算不出照拒站得住。
- 件 15、16 修的九类误拒与两类零星，此刻都放：喂 `e1-…`、`e2-systemd-run-light`、`e2-chrt-light`、`e3-literal-heredoc-target`、`e4-alias-one-line`、`e5-awk-file-literal-variable`、`e6-…`、`e7-…`、`e9-…`、`posix-function-uncalled`、`builtin-coproc` 都退 0；同形起重活的 `e2-control-systemd-run-heavy`、`e4-control-alias-newline` 退 2。E3、E5 的变量值里带 `$PWD` 时照拒（喂 `e3-variable-heredoc-target`、`e5-awk-file-variable` 退 2），钩子只展开赋过字面值的，与文件头一致。
- 「改对的」里靠装置读法改判的三批：legB 七条 MEMCAP 假象按登记的 H2 记成改对的、legB L-mem-watch-ws 两条（装置里 watch 起不来、腿登记成放）、legB-r2 e.J07（腿按旧口径写的应当值）。分类员各自另有理由（桩只看祖先进程、草稿里设 TERM 实测 watch 起子命令、`tail --pid` 算等了一个），没看出是装置造出来的改对；我没重喂这三批。
- 没喂、推的：件 13 剥开 `host-dispatch.py` 之后，里面那条在主工作区合入锁被占时照拒（`d137-hook-launcher/result.md` 的 case9 就是只拒在合入锁上）。派到第二台时 cargo 跑在拷过去的仓副本里，不碰主工作区的编译目录；但拷树那一刻主工作区若正在合入，拷过去的也是半截状态，拒也说得通。交主 agent 定，不算这一腿的结论。

## 四、推翻条件

- 第一节「站住」的各行：此刻五份钩子（sha256 同开头那一段）上照 `tools/rerun.sh` 重喂，`out/feed-*.txt` 里任一条 `name=` 行变了（例如 `r1-line27-combined-short-options` 退 0、`h5-cd-missing-and-relative` 退 2）；「照修补件读数」的 9 行另要那几件报告里的复现与改后读数在此刻的主工作区重跑对不上。
- H3、A2、F 三行「带保留」：真 bash 在一行写法里起了 alias 里的 cargo（H3）；`r1-line33-bash-env-function-script-on-disk` 退 0（A2 那一行改判够不着）；`r2-line34-exact` 退 0，或主 agent 说明第二轮第 34 行的「放」只指 cd 那一半、printf 写出照拒本来就对（F 改回站住）。
- 第一节「带保留」里落在范围上的几行（第一轮撑住两段、第二轮第 55、58 行、M3、M4）与第二节第 1、2、5、7 行：用此刻的钩子补跑第二节那几次观测，结果与判决的说法逐项相同（零回退、legA-r3 六份没有新的「对→误拒」、23 条开关点名对、语料还拒的都是照口径），这几行改成站住。
- M5「够不着」：主 agent 拿出用此刻四份钩子跑的四份全量复跑读数。
- 第二节第 3 行：那 26 条在目录建好的树里重喂都退 0（说明拒只在目录不在时出现），同时第三节第 1 处的最小复现在某种写法下退 0，「不靠目录在不在」那一半不成立；或真 bash 在 cd 失败时把 heredoc 写到了别处、起的不是同一份。
- 第三节第 1 处：在真 shell 里（隔离的样本目录、假 cargo）跑三条最小复现，cd 失败那一支起的不是 heredoc 那两行、也不是「找不到 r.sh」；或钩子文件头、判决里有一句明写「cd 之后写出的脚本，cd 失败的照拒」是用户定的。
- 第三节第 2 处：用户或判决里有一句明写「echo / printf 写出一律照认不出拒」（那么「照口径」成立，第二轮第 34 行的「放」是判决写错）；或 bash 内建 echo 在钩子的环境里默认解释反斜杠（`shopt xpg_echo` 开着），那一窄条要再收窄。
- 第三节第 3 处：真 bash 跑这三条起了别的东西（例如 cp 在多个操作数下照拷了 d01.sh）。
- 第三节第 4 处：`P=t.py; python3 $P` 退 2；或件 15、16 那几条最小复现此刻退 2。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；第一节的「站住」不等于第三轮维持。
- 不攻钩子漏拒那一半：没造新的绕法，没复现 Y014、Y029、Z021 与变量拼命令词那两处；喂到的漏拒形只有前两轮判决喂法表里原样的几条（核它们此刻照拒）。
- 第三节「真 bash 会怎样」都是推的，没在真 shell 里起这些命令；钩子只喂了 JSON。
- 「照修补件读数」的 9 行（H6、H7、H10、S3、过度修补 D 与 E 与 G 那一行、M2、W1、W2、U1）没现喂、没现跑，照各件报告的读数记。
- 没跑四份全量复跑、legA-r3 六份的全量 diff、真实语料重喂、开关整表复跑（第二节列的补测都没做：要统一任务启动器或超出前台时限，交主 agent）。
- A 类 26 条按正则粗分，没逐条展开核；没数这一形在别的会话记录里的频次。
- 没读这一轮另一条腿的报告与模型目录（禁读清单）。
- 没跑重型测试、没起 cargo、没碰 GPU；喂钩子每次经 `scripts/hosts/run-with-memory-cap.sh 4G`，都一次排上。

## 草稿目录

`/home/fy5090/.local/share/singlefs/work/d137-gov-r3-defense/`：喂钩子用的钩子树 `tb/`（`.claude/hooks` 真拷贝 5.7M，别的是符号链接）与两次复跑的工作目录 `rerun-1/`、`rerun-2/` 已删（删前 `du -sh`：5.7M、5.9M、5.9M），删后整个草稿目录 224K；留样本目录、假准入脚本、各次原样输出（`feed-*.txt`、`rerun-*.txt`）。
