# d137-gov-r2 攻方论证腿（opus-b）：方法、看门狗声明豁免与单元归属、派发闸 ⑨、共用约束与代码一致性

日期：2026-10-30（`TZ=Asia/Tokyo date +%F`）。轮名 d137-gov-r2，第二轮；前一轮判决 `research/prompts/d137-gov-r1-main-verification.md`。
这一条腿不造、不喂任何绕过钩子的写法；只读代码与报告，调纯函数（`core_usage.declaration_excuses_low`、`judge_series`、`verdict_lines`、`alert_record`，`agent-watch.py` 的 `unit_owner`、`foreign_unit`、`verdict_counts`）算给定输入的结果。
禁读清单里的 `research/prompts/d137-gov-r2-sonnet-output.md` 与 `research/prompts/d137-gov-r2-sonnet-model/` 没读。

## 复跑命令与模型文件

复跑（整条经内存包装；`--work` 给一个新的空目录；内存队排不上时照正文「五、交付要求」带 `ulimit -v 4194304` 与 `nice -n 19` 直接跑，我交回前就是这样跑的、退 0、两份输出逐字节相同）：

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/d137-gov-r2-opus-b-model/tools/rerun.sh --repo <仓根> --frozen research/prompts/d137-gov-r2-snapshot --work <新的草稿目录>
```

它跑 `tools/declaration_cases.py`（格 2、格 5，调 `scripts/hosts/core_usage.py`）与 `tools/unit_owner_cases.py`（格 3，调 `scripts/agents/agent-watch.py`），输出与 `out/` 里同名文件逐字节比，打 `name=rerun_compare … same=yes|no`。`tools/interleave_measure.py` 读本机 `~/.local/state/singlefs/detached-runs/*/serial-declarations`，登记会变，不进复跑，`out/interleave-measure.txt` 是我跑那一次的原样。

| 文件 | sha256 |
|---|---|
| `tools/declaration_cases.py` | 09b7c0402972f3251160f61ec7350f758491862c6f0d1bb386deb0388b6269e0 |
| `tools/unit_owner_cases.py` | 2d31055868b85478d58c5ea9ef2d23582ac5939025f2507972f72fc76e5c1b12 |
| `tools/interleave_measure.py` | d21841dd7b3c09194b59f365d5ab20c774fbed892f3ce33e1433cb99d9549315 |
| `tools/rerun.sh` | 52af36b431ef3711a79e5e51253631cfa66fb815c7bc39b0358ea3aa687506e2 |
| `out/declaration-cases.txt` | 5ec7ffb6e1f5d133e7c431cdc55f3587cc0a4167d59dbbc86a86805d937f647d |
| `out/unit-owner-cases.txt` | c98128bc4d4273dc28db702225bcb380747bd5022179533c698eaef03afa483c |
| `out/interleave-measure.txt` | 39cd6013a4f87530c34bb0720c1904fa64b9920df4e1c92b9ebd76c586e7cc81 |

## 各格判定一览

被判文件此刻的 sha256 前 8 位（`sha256sum` 现取）：`scripts/hosts/core_usage.py` 33016c5b、`scripts/agents/agent-watch.py` 0c41175d、`scripts/hosts/detached-run.sh` ac015ba4（快照 `research/prompts/d137-gov-r2-snapshot/files-sha256.txt` 记的是 0a9d3e70，开工之后这一份被改过；下面引它的行号都是此刻的）、`.claude/hooks/runner-dispatch-guard.sh` 2b3ceb0a、`.claude/agent-common.md` 65769f7a、`.claude/main-agent.md` 612a3de2；不在快照里、引到的：`scripts/gate/admission.py` 90006cd7。

| 格 | 打中几条 | 编号 | 一句话 |
|---|---|---|---|
| 1 产生结论的方法 | 6 | M1–M6 | 开关核对只看前 48 个字；自证首趟红靠重跑到绿；三份大改的钩子开关表不在复跑登记里、修补件只跑了新开关与少数旧开关；dev 树 25 格底子红遮住开关多红；「回归」只是点名反例；3098 秒取自一段被截断的下界 |
| 2 看门狗声明豁免 | 2 | W1、W2 | 告警记录取整之后与判的那一行给出相反的叫不叫醒（20 万次取样里 43 次「行说叫醒、看门狗静」）；同一份声明文件里几个程序的声明互相截断、互相续命（本机 29 份登记里 15 份有不止一种理由） |
| 3 单元归属 | 1 | U1 | 判定行那一半按名字读此刻登记：同名重起之后，旧一次本会话单元的判红行被当成别的会话的、不数 |
| 4 派发闸 ⑨ | 2 | D1、D2 | 退 0 而实验键拒算（判不了「没变」）静默放行、不记检出；admission.py 自己报的「调用有错」退 2 被当「python 自己退 2 或用法错」放行，钩子进程环境里设了强制重跑理由时退 77 变退 0 静默放 |
| 5 共用约束与代码一致性 | 1 | C1 | `.claude/main-agent.md` 第 70 行两条处置在同一行判的那一行上重叠：过期满一个窗口之后那一行写「这一窗口里没有声明串行段」又写「拖过了它算数的 7200 秒」，一条说停、一条说查进度；规则纪律 1–3、7 条在新句子里没打中 |

数的来历：W1、W2、U1、C1 的结果是我的模型调仓里纯函数算的（量的，`research/prompts/d137-gov-r2-opus-b-model/out/` 下原样输出，复跑见开头）；M6 与 W2 的「本机登记」数是读本机 `~/.local/state/singlefs/detached-runs/*/serial-declarations` 此刻的样子（量的，`out/interleave-measure.txt`，不进复跑：登记会变）；D1、D2 是读代码推的，没起 admission.py。

## 格 1　产生结论的方法

各件报告在工作根下，不在仓里，cite-check 认不出这些路径、会列进「没判」；行号是我 `grep -nF` 现取的。

### M1　看门狗修补的开关核对按「前 48 个字」比，撑不住「红的都被点名」

- 说了什么：`~/.local/share/singlefs/work/d137-r1fix-watch/result.md:163`：「每条红的失败行（每行取前 48 个字」，同一句接着说「点名的都红了，红的都被点名」。
- 量了什么：19 条 agent-watch 开关的失败行截到前 48 个字，与文件头点名的「」逐项对。同一份报告里 `foreignunits` 那一条贴出的两行截完逐字相同（「✗ 自检：报告里应当有「判定行 /tmp/selftest-scratch-pass-probe」各一行）；原样输出 `d137-r1fix-watch/switch-b2/watch-foreignunits.txt` 里这两行是两个不同的失败（`verdict-foreign-mixed.log` 与 `verdict-foreign-foreign-only.log`）。
- 差在哪：前缀相同的两格对不出谁是谁；一条开关多红出一格、而那一格与点名格前 48 个字相同时，这种比法看不见。「红的都被点名」只能说到「前 48 个字的集合相同」。另：agent-watch 135 条开关跑了 19 条、core_usage 14 条跑了 6 条、detached-run.sh 43 条跑了 4 条（同一份报告「没跑的开关」一段；`~/.local/share/singlefs/work/d137-r1fix-watch/result.md:187`：「另外 116 条开关没跑（判它们点名的格这一批没碰）」——「没碰」是按点名的自证函数挑的，推的，没量）。

### M2　看门狗自证首趟红 1 格，靠重跑两趟到绿

- 说了什么：同一份报告第 130 行表格里「第一趟退 1，红 1 格」，归因「负载 168 时多数到一个还没 exec 完的中间进程，推的」，结论写自证绿。
- 量了什么：同一条命令又跑两趟，都退 0。
- 差在哪：红的那一格（selftest_heavy_tests 的「timeout 包着没带前缀的只报 timeout 那一个」）没在同负载下复现、也没核归因；三趟里一趟红是「不稳定」，不是「绿」（`.claude/rules/three-way-inference.md` 引的 `test-discipline.md`「单次观测不算数」同一个口径）。这一格不在这一批改动里，结论「这一批没把它改红」可以成立，「自证绿」不成立。

### M3　三份大改的钩子：「开关只红点名格」只核了新开关与几条旧开关，之后也没有整表复跑兜着

- 说了什么：四件钩子修补各写「新开关逐个核红、只红点名格」；第四件另写 `~/.local/share/singlefs/work/d137-r1fix-hooks-4/result.md:93`：「已有开关因为新格多红的，照实测改了文件头（C、D、F 三趟）」——读起来像旧开关里受影响的都找全了。
- 量了什么：第四件 `switch-runs/` 下 26 份（开关 × 自证）输出，其中 10 条新开关，旧开关只有 startup-files-unread、interactive-startup-files-unread、claude-launch-layers-narrow、startup-file-claude-unread、brace 三条与 braces-unexpanded 这几条；第三件 4 条新开关；第二件 6 条；第一件 1 条。
- 差在哪：此刻文件头里的开关数（`grep -oE '<前缀>_BREAK=[a-z0-9-]+' <文件> | sort -u | wc -l` 现数）：`lib_shell_words.py` 62 条、`heavy-test-guard.sh` 36 条、`bash-command-detector.sh` 67 条；四件合起来改了 1845 行（第四件贴的 `git diff --stat`）。没跑的旧开关打开之后红几格、红不红点名格，没人量过。提交前的 break-switch-replay 格兜不住：它只复跑登记表里的文件，而这三份不在表里——

```
$ grep -cE '^\.claude/hooks/(heavy-test-guard\.sh|lib_shell_words\.py|bash-command-detector\.sh)	' .claude/gate.d/break-switch-replay-files.tsv
0
```

  所以「旧开关照旧红在点名格」这一句，这一轮没有任何一步量过，以后也不会被门禁量到（`.claude/gate.d/code-tooling.sh:538` 那一格的说明是「登记表里的脚本与钩子」）。

### M4　第三件在 dev 树上核开关，底子里 25 格本来就红

- 说了什么：`~/.local/share/singlefs/work/d137-r1fix-hooks-3/result.md:41`：「每个开关只红它点名的「d137 …」格、base 的 25 格 dev-tree 符号链接伪红不变」。
- 量了什么：四条新开关在 dev 树上各跑一趟，数红格；主工作区上只跑了不设开关的自证。
- 差在哪：底子已红的 25 格，开关打开之后再红也看不出来；「只红点名格」在这 25 格上没有判别力。要判，得在主工作区（或底子全绿的树）上设开关跑。

### M5　「回归全拒」只喂了点名过的反例

- 说了什么：第二、三、四件都写「回归再喂……都还拒」，第四件 `~/.local/share/singlefs/work/d137-r1fix-hooks-4/result.md:96` 列的是 13/13、28/28、41/41。
- 量了什么：shell-words-r3 判决第 2、3、7 条点名反例与前几件 H1–H8 的点名反例。
- 差在哪：第一轮判决里「撑住」的那几批（正推腿量的「认不出一律拒 26 条、三种静默漏拒 7 条、主 agent 前缀规矩 18 条、准入退码 15 格里 14 格、撤旧根两样 29 格、⑫ 写范围 24 格」）在 `research/prompts/d137-gov-r1-sonnet-model/tools/`（`cases_heavy.py`、`cases_fuzz.py` 等）里，四件修补都没复跑（`grep -ln 'd137-gov-r1-sonnet-model\|rerun.sh' ~/.local/share/singlefs/work/d137-r1fix-*/result.md` 零命中）。1845 行切词改动之后，「撑住」那几格此刻还撑不撑住是没量的。这一条腿按派发不喂钩子，没替它们复跑。

### M6　crates-mutation-rows 派活那一段的 3098 秒取自一段被别的声明截断的下界

- 说了什么：`~/.local/share/singlefs/work/d137-serial-seconds/result.md:20`：「仅有的一段 2065 秒（debt-mutation-row-3501-3），只有这一个读数」，2065 × 1.5 取 3098；同一份报告第 14 行说量法「只取紧接「此刻并行」的配对」，第 18 行说「另 1606 段被并发的别趟声明隔断，只是下界」。
- 量了什么（我现读本机登记，`out/interleave-measure.txt` 原样）：

```
name=declaration_files files=29 files_with_more_than_one_reason=15
name=segments key=crates 变异复跑的派活 paired=0 paired_max=0 interrupted=2 interrupted_gap_max=2065
name=segments key=在卡上算：经 gpu-qui paired=1376 paired_max=14050 interrupted=1609 interrupted_gap_max=545
```

  debt-mutation-row-3501-3 那份声明文件里，派活那一行后面紧跟的是 gpu-quiesce-run 写的「此刻串行：在卡上算…」，不是「此刻并行」：2065 秒是被截断的下界，按报告自己的量法本该归「只是下界」那一堆，却被当成整段、乘 1.5 定了预计秒数。
- 差在哪：3098 是「一个下界 × 1.5」，不是「量到的最长 × 1.5」；真实长度没量到。另，「在卡上算」配对段此刻最长 14050 秒（报告当时量的是 7424；登记在那之后又多了），超过 7200 的那一截照叫醒，与用户定的「超过 7200 叫醒、查进度」一致，不算打中。

| 编号 | 分不分辨（这件修补与它不做的样子） | 系统当时看得到吗 | 对不上的是哪一句 | 修补件的改法在这一格上 |
|---|---|---|---|---|
| M1 | 分辨不了：截断比法下改前改后都可能「逐项相同」 | 原样输出里有全文，比的人没用 | 「点名的都红了，红的都被点名」 | 改用全文比就能判 |
| M2 | 不分辨：红的格不在改动里 | 看得到首趟红 | 「自证绿」 | 不涉及 |
| M3 | 分辨不了：旧开关没跑 | 文件头有表，没跑 | 「开关只红点名格」「已有开关…照实测改了文件头」 | 只核了新开关与 8 条左右旧开关 |
| M4 | 在 25 格上分辨不了 | 底子红是已知的 | 「每个开关只红它点名的格」 | 换到主工作区跑就能判 |
| M5 | 分辨不了：撑住那几批没复跑 | 第一轮模型在仓里 | 「回归都还拒」 | 只喂点名反例 |
| M6 | 分辨：下界与整段在原文件里分得开 | 下一行是什么一读就知道 | 「仅有的一段 2065 秒」 | 预计秒数取值的依据不成立 |

## 格 2　看门狗声明豁免（`declaration_excuses_low` 与它在 `agent-watch.py` 的两处调用）

判法：`scripts/hosts/core_usage.py:502`：「return undeclared <= 0 or cores * span / undeclared >= threshold」；本机重活那一半拿 verdict 调（`scripts/agents/agent-watch.py:3468`），告警记录那一半拿记录调（`scripts/agents/agent-watch.py:3531`：「if declaration_quiets_low(core, record):」）。阈值 31 / 4 = 7.75，样本每 300 秒一次（与 detached-run.sh watch 的默认间隔同），所以 span 落在 600–900 秒之间。

### 公式本身对得上（没打中，量的）

`out/declaration-cases.txt` 里 `excuse_formula` 9 行，函数结果与文件头公式逐行相同，例：

```
name=excuse_formula cores=1.0 span=600 declared=522 excused=False header_formula=False
name=excuse_formula cores=1.0 span=600 declared=523 excused=True header_formula=True
name=excuse_formula cores=0.0 span=600 declared=600 excused=True header_formula=True
```

### W1　告警记录那一半拿取整过的读数判，与判的那一行给出相反的叫不叫醒

- 出处：`scripts/hosts/core_usage.py:571`：「"alert_pid": alert_pid, "cores": round(verdict["cores"], 2),」；`scripts/hosts/core_usage.py:573`：「"span": round(verdict["span"], 1), "declared_in_window": round(verdict["declared_in_window"], 1),」。判的那一行（`declaration_note`）用的是没取整的 verdict，看门狗告警记录那一半用的是取整后的记录。
- 给定输入与结果（量的，`out/declaration-cases.txt` 末三行原样）：

```
name=record_rounding trials=200000 line_wakes_record_silent=43 line_silent_record_wakes=34
name=record_rounding_example kind=line_silent_record_wakes cores=2.4739497171667413 span=797.4581267101262 declared=543.2509334351927 record_cores=2.47 record_span=797.5 record_declared=543.3
name=record_rounding_example kind=line_wakes_record_silent cores=7.247466649228653 span=754.8397420680681 declared=48.86858865624724 record_cores=7.25 record_span=754.8 record_declared=48.9
```

  第三行：判的那一行算 7.2475 × 754.84 ÷ 705.97 ≈ 7.749 < 7.75，写「声明解释不了（看门狗照叫醒）」（`scripts/hosts/core_usage.py:544`）；告警记录里是 7.25、754.8、48.9，看门狗算 7.25 × 754.8 ÷ 705.9 ≈ 7.752 ≥ 7.75，只记不叫醒。
- 对不上的句子：`scripts/hosts/core_usage.py:67`：「告警记录那一半拿记录里的 cores、span、threshold、declared_in_window 调），两处不另立门槛。」；修补件自己的推翻条件 `~/.local/share/singlefs/work/d137-r1fix-watch/result.md` 「core_usage 判的那一行与看门狗对同一份读数给出相反的叫不叫醒」。取样落在阈值附近万分之几的带子里，频率低（20 万次 77 次，取样分布是我定的均匀分布，推的），但方向两边都有，含「行说叫醒、看门狗静」那一边。
- 只管告警记录那一半（`multi-host-run.sh --peer`、`detached-run.sh watch` 写的）；本机重活那一半拿 verdict 直接调，不取整，不受影响。

### W2　同一份声明文件里几个程序的声明互相截断、互相续命

- 出处：`scripts/hosts/core_usage.py:57`：「下一条声明（串行或并行）结束上一段串行」——判法按文件、不按写的程序；`.claude/main-agent.md:70` 写「一段声明至多算到它写的预计秒数」。同一棵树里几个程序共用一份 `SINGLEFS_SERIAL_DECLARATIONS` 在本机是常态：M6 那张表里 29 份登记 15 份有不止一种理由，debt-mutation-row-3501-3 里 crates-mutation-rows 写的派活声明被它底下 gpu-quiesce-run 写的「在卡上算」接上。
- 给定输入与结果（量的，`out/declaration-cases.txt` 的 `shared_file` 6 行原样；程序甲在第 0 秒写「此刻串行（预计 7200 秒）」、从不闭合，树 0.2 核）：

```
name=shared_file case=writer_alone now=3600 status=low cores=0.200 span=600 declared=600.0 excused=True problems=0 line_has=声明解释得了
name=shared_file case=writer_closed_by_other now=3600 status=low cores=0.200 span=600 declared=100.0 excused=False problems=0 line_has=声明解释不了
name=shared_file case=writer_alone now=8000 status=low cores=0.200 span=800 declared=0.0 excused=False problems=0 line_has=这一窗口里没有声明串行段|拖过了它算数的
name=shared_file case=writer_extended_by_other now=8000 status=low cores=0.200 span=800 declared=800.0 excused=True problems=0 line_has=声明解释得了
name=shared_file case=writer_extended_by_other now=14000 status=low cores=0.200 span=800 declared=800.0 excused=True problems=0 line_has=声明解释得了
name=shared_file case=writer_extended_by_other now=14300 status=low cores=0.200 span=800 declared=700.0 excused=False problems=0 line_has=声明解释不了|拖过了它算数的
```

  截断：程序乙在第 3000 秒写一段串行、第 3100 秒写「此刻并行」，甲还在串行，第 3600 秒照叫醒（假告警，安全的一边）。续命：乙在第 7000 秒写「此刻串行（预计 7200 秒）」，甲即使在第 7200 秒之后卡死，看门狗一直静到第 14000 秒以后——第 70 行「因声明拖过 7200 秒那一截叫醒的」那条处置，在共用文件里要推迟 6800 秒以上才轮得到（`.claude/main-agent.md` 第 70 行）。
- 差在哪：文件头与 main-agent 写的是「每段」，代码做的是「这份文件里最近的一段」；树里有几个写者时，「每段至多算到它写的预计秒数」对单个程序不成立。

### 别的边界（给定输入与结果，都对得上文件头，没打中）

| 输入（声明写在第 1000 秒，1 核，除注明外） | 结果（量的） | 对的是哪一句 |
|---|---|---|
| 没写预计秒数，第 1677 秒 | 解释不了，那一行写「拖过了它算数的」600 秒 | 文件头「没写的取一个窗口」 |
| 「（预计 0 秒）」「（预计 07 秒）」「（预计 900秒）」 | 不算声明、`problems=1`，照叫醒 | 「预计秒数不是正整数」算写坏（「07」被说成不是正整数，字面不准，方向安全） |
| 「（预计 7201 秒）」「（预计 99999 秒）」 | 与 7200 逐行相同，第 8800 秒起叫醒 | 「超过 7200 秒的照 7200 算」 |
| 第 100 秒写、预计 900 秒、0 核（跨窗口） | 第 900、1000 秒解释得了，第 1100 秒起解释不了 | 窗口按 overlap 截 |
| 开头不是秒数、理由太短、二进制垃圾 | `problems=1`，照叫醒 | 「写坏的行……判的那一行里点出来」 |
| 「此刻 并行」写错 | 写坏一行、上一段照开着（至多算到预计秒数） | 写坏的行不结束上一段 |
| 秒数写成毫秒（1500000） | 不算声明、`problems=0`，那一行只写「这一窗口里没有声明串行段」，不点出来 | 文件头写坏的行没列「将来的时刻」，字面对得上；方向安全 |

## 格 3　单元归属（`unit_owner`、`foreign_unit` 与 detached-run.sh 写的 owner-session）

登记写法：`scripts/hosts/detached-run.sh:851`：「detached_run_write_field "$record" owner-session "${CLAUDE_CODE_SESSION_ID:-}" || unwritable+=" owner-session"」。告警记录那一半先算「看不看」再算「是不是别人的」：`scripts/agents/agent-watch.py:3518`：「watched = record.get("session_id") in session_ids or (owner is not None and owner in session_ids)」，然后第 3519 行不看的直接跳过、第 3521 行 `foreign_unit` 判别人的只记一行。

### 各种归属情形（量的，`out/unit-owner-cases.txt` 原样；被看的会话是 S-me，登记目录是模型自己建的）

```
name=record_half label='own' record_unit_session=None record_session=S-other watched_session=S-me unit_owner=S-me watched=True foreign=False outcome=woken
name=record_half label='theirs' record_unit_session=None record_session=S-me watched_session=S-me unit_owner=S-other watched=True foreign=True outcome=logged-foreign
name=record_half label='empty-owner' record_unit_session=None record_session=S-me watched_session=S-me unit_owner=None watched=True foreign=False outcome=woken
name=record_half label='empty-owner' record_unit_session=None record_session=S-other watched_session=S-me unit_owner=None watched=False foreign=False outcome=skipped-silently
name=record_half label='legacy' record_unit_session=None record_session=S-me watched_session=S-me unit_owner=None watched=True foreign=False outcome=woken
name=record_half label='legacy' record_unit_session=None record_session=S-other watched_session=S-me unit_owner=None watched=False foreign=False outcome=skipped-silently
name=record_half label='unreadable' record_unit_session=None record_session=S-other watched_session=S-me unit_owner=None watched=False foreign=False outcome=skipped-silently
name=record_half label='theirs' record_unit_session=S-me record_session=S-other watched_session=S-me unit_owner=S-me watched=True foreign=False outcome=woken
name=record_half label='own' record_unit_session=S-other record_session=S-me watched_session=S-me unit_owner=S-other watched=True foreign=True outcome=logged-foreign
```

（全 14 行在文件里；`legacy` 是登记里没有 owner-session 的改动之前起的单元，`empty-owner` 是起它时没设会话号，`unreadable` 是登记目录权限 000。）

读法与文件头对照（没打中的几格）：
- 会话号取不到（owner-session 写空）、改动之前起的、登记读不了：`unit_owner` 都交 None，`foreign_unit` 交 False——与 `scripts/agents/agent-watch.py:201`：「认不出归属的（记录里没带、登记里没有或读不了、owner-session 是空的）不当成别人的，照报；」一致。
- 但「照报」只发生在写这条记录的会话（跑 watch 的那个）：记录是别的会话的 watch 写的时，起单元的会话（若真是 S-me）这边 `watched=False`，连一行都不记。文件头没说报给谁，字面不矛盾；修补件报告与调度记录第 129 行写的「改动之前起的单元一律照叫醒」读起来像起它的会话会被叫醒，实际叫醒的是跑 watch 的会话。边界，记下，不算打中。
- 别的会话的单元：只记一行（`logged-foreign`），与文件头一致。记录里带的 `unit_session_id` 优先于登记（最后两行），与 `unit_owner` 的说明一致。

### U1　判定行那一半按名字读此刻的登记：同名重起之后，旧一次本会话单元的判红行被当成别的会话的、不数

- 出处：`scripts/agents/agent-watch.py:2334`：「owners[unit.group(1)] = unit_owner(unit.group(1))   # 同一个单元的判红行一份文件里读一次登记」——判定行没有会话号，只按名字读登记目录 `<名>/owner-session`；文件头 `scripts/agents/agent-watch.py:199`：「判定行那一种读登记（DETACHED_RUN_STATE_DIRECTORY，没设照 detached-run.sh 的默认登记目录）里那个单元的 owner-session。」
- 同名重起是 detached-run.sh 认的用法：`scripts/hosts/detached-run.sh:101` 起的那一段写「观察之后同名又起过一次的，旧一次的退出码到挪走的 <名>@<起跑的秒数> 里读」；带 `--label` 的 watch 不因别的单元结束而退出（`scripts/hosts/detached-run.sh:99`：「--label <名>（可给几次）：只盯点名的那几条，别的结束不叫醒」），而在用核数照判全部在跑的登记（`scripts/hosts/detached-run.sh:1197` 那一圈不看 `--label`），所以同一份 watch 输出里会先后出现同名两次的判红行。
- 给定输入与结果（量的）：watch 输出里两行「✗ reused 在用 … 核」是 S-me 起的旧一次（登记已挪成 `reused@1000`，里面 owner-session 是 S-me），此刻 `reused/owner-session` 是别的会话重起的 S-other：

```
name=verdict_half_reused_label watched_session=S-me red_counted=1 foreign_not_counted=2 registry_now=S-other old_run_owner=S-me
```

  S-me 自己那两行判红被当成别的会话的、不数进 ✗（只数了 `own` 那一行）。反过来（旧一次是别人的、重起的是 S-me）别人的判红行会被数进来。
- 对不上的是哪一句：`.claude/main-agent.md:64` 「只报本会话起的单元（按单元带的会话号认，即 `detached-run.sh start` 写进登记的 `owner-session`；认不出归属的照报）」——判定行那一半认的不是「单元带的会话号」，是「此刻同名登记的会话号」。

| 编号 | 分不分辨 | 系统当时看得到吗 | 对不上的分句 | 修补件的改法在这一格上 |
|---|---|---|---|---|
| U1 | 分辨：改前按主会话记录里的 `--label` 认，同名两次的判红行都算本会话的（错在另一边）；此刻的判法都算此刻登记的会话，两种判法在这一格上都错 | 看得到一半：挪走的 `<名>@<秒数>` 里有旧一次的 owner-session，判定行本身没带起跑秒数，分不出是哪一次 | main-agent 第 64 行「按单元带的会话号认」 | 不起作用 |

## 格 4　派发闸 ⑨ 的分法（`designer_admission_verdict` 对 `admission.py experiment` 各结局的处置）

逐格对照（读代码，推的；没起 admission.py、没喂派发闸）：

| admission.py 的结局 | 代码怎么处置（`.claude/hooks/runner-dispatch-guard.sh`） | 文件头 ⑨ 第 66–72 行 | `.claude/agent-common.md` 第 96 行 | 用户与调度记录的口径 | 对得上吗 |
|---|---|---|---|---|---|
| 退 77（键没变） | 拒（第 1625 行起） | 拒 | 拒 | 第一轮之前就拒，口径没改 | 对得上 |
| 退 2 且 stderr 有「✗ 准入登记里没有实验键 」或「✗ <键> 的准入登记有错：」 | 拒（第 1607 行正则、第 1628 行起） | 拒 | 拒（「且标准错误里有它自己报…的句子」） | 记录第 44 行「设计员写重跑登记时准入退 2 照拒」 | 对得上 |
| 退 3（前提没齐） | 拒，不看 stderr（第 1641 行起） | 拒 | 拒 | 照拒 | 对得上 |
| 脚本不在、起不来、超时 50 秒 | 放行并记检出（第 1615–1624 行） | 判不了 | 判不了 | 判不了放行并记检出 | 对得上 |
| 退 0、2、3、77 之外的码 | 放行并记检出（第 1647 行起） | 判不了 | 判不了 | 记录第 99 行「准入退别的码照「判不了时放行并记检出」（主 agent 照派发闸现行规矩定）」 | 对得上 |
| 退 0 而实验键拒算 | 静默放行 | 退 0 算认得的结局 | 不在判不了的清单里 | — | D1 |
| 退 2 而是 admission.py 自己报的「调用有错」 | 放行并记检出，说明写「python 自己退 2 或用法错」 | 只认那两句 | 只认那两句 | 「准入判法退 2 照拒」 | D2 |

### D1　退 0 而实验键拒算（判不了「没变」）：静默放行，不记检出

- 出处：`scripts/gate/admission.py:3355`：「照放行，下一趟也判不了「没变」」，紧接第 3356 行 `return EXIT_ADMITTED, [f"{HEADER_PREFIX}name=input_fingerprint key={key} sha256=refused units={unit_count}"]`。钩子这一侧退 0 不进任何分支，第 1647 行只管「0、2、3、77 之外」。
- 给定输入与结果（推的）：一个登记了准入、闭包里有拒算单元的实验，在「输入其实没变」时派设计员写它的重跑登记 → admission.py 退 0、stdout 一行 `sha256=refused`、stderr 一行「! … 实验键拒算 …照放行」→ ⑨ 放行，检出记录里没有这一条。
- 对不上的是哪一句：派发闸的总规矩「判不了的……照常放行并记一条检出（只记）」（`.claude/agent-common.md` 第 96 行末段）；⑨ 文件头自己写「分得清「准入判法明确报了」与「判法本身跑不起来」」（`.claude/hooks/runner-dispatch-guard.sh:70`），而 admission.py 在这一结局里明说的是「判不了」，⑨ 把它当「判了、放行」。
- 系统当时看得到吗：看得到，`result.stdout` 里有 `sha256=refused`，`result.stderr` 里有「实验键拒算」。

### D2　admission.py 自己报的「调用有错」退 2，被当成「python 自己退 2 或用法错」放行；钩子进程环境里的强制重跑理由让退 77 变退 0

- 出处：admission.py 的退 2 不止两句——`scripts/gate/admission.py:205`：「退 77 research/results/ 里有一份产物头上记着同一个键；退 3 前提没齐；退 2 登记或调用有错。说明一律打到 stderr」。「调用有错」那几种：强制重跑理由设了却是空的（`scripts/gate/admission.py:3366`：「say(f"  ✗ {FORCE_RERUN_VARIABLE} 设了却是空的：强制重跑要写理由")」）、弄坏开关在自证外设了（`scripts/gate/admission.py:6549` 那一句「是弄坏开关，只在自证（--selftest 本身与它起的子进程）和门禁样本根里认」）。这两句都不合 `.claude/hooks/runner-dispatch-guard.sh:1607` 的正则，落进第 1634 行：「（python 自己退 2 或用法错），准入这一条没判，照常放行」。
- 另一半：钩子起 admission.py 不换环境（`.claude/hooks/runner-dispatch-guard.sh:1621` 没给 `env=`），`SINGLEFS_EXPERIMENT_RERUN_REASON`（`scripts/gate/admission.py:428`）在 Claude Code 进程环境里非空时，admission.py 走第 3362 行起那一段、第 3371 行退 0——键没变也放行，⑨ 不拒、不记。
- 给定输入与结果（推的）：Claude Code 进程环境里 `SINGLEFS_EXPERIMENT_RERUN_REASON=`（空）→ 退 2、stderr「✗ SINGLEFS_EXPERIMENT_RERUN_REASON 设了却是空的」→ ⑨ 放行、检出说明写「python 自己退 2 或用法错」（说明与实情不符）；设成非空 → 退 0 → ⑨ 静默放行，退 77 那一格失效。`ADMISSION_BREAK=<任意>` → 退 2、stderr「✗ ADMISSION_BREAK 是弄坏开关…」→ 同第一种。
- 对不上的是哪一句：调度口径「准入判法退 2、退 3 照拒，别的码判不了放行并记检出」（派发提示转述；记录第 44 行原话「设计员写重跑登记时准入退 2 照拒」）——这几种是准入判法自己退的 2，按口径该拒；按第一轮修补的分法（只认两句）放行。两种读法各有理由，分歧点是「判法自己报的调用有错」归「判法明确报了」还是「判法跑不起来」，文件头与共用约束都没写这一种。
- 环境变量要先在 Claude Code 进程里设着才走得到（推的：平常不会），所以实际频率低；打中的是分法的字面。

### 别的边界（读代码，推的，没打中）

- 只按 `E<号>` 一把键判：`.claude/hooks/runner-dispatch-guard.sh:1258`：「RERUN_REGISTRATION = re.compile(r"research/prompts/e(\d+)-r\d+-prereg\.md")」。准入登记里的子键（`.claude/gate.d/stage-inputs.tsv` 里的 `E142/layer0`，带 `same-run-field=` 前提）⑨ 判不到；带字母后缀的实验号（admission.py 的键形 `E[0-9]+[A-Z0-9]*` 认，正则不认）整条 ⑨ 不起。文件头写的就是「`admission.py experiment <根> E<号>`」，字面一致；此刻仓里没有字母后缀的实验提示。
- 准入登记表整份不在时 `read_registration_rows` 交空表（`scripts/gate/admission.py:575` 的说明「表不在返回空表」），admission.py 报「没登记」退 2 → ⑨ 拒；表在而读不了（权限）→ 抛异常退 1 → ⑨ 放行并记检出。同是「判法读不到登记表」，两边处置相反；主工作区里表一直在，碰不到。

| 编号 | 分不分辨 | 系统当时看得到吗 | 对不上的分句 | 修补件的改法在这一格上 |
|---|---|---|---|---|
| D1 | 改前改后都静默放行，第一轮修补没碰这一格 | 看得到（stdout 的 `sha256=refused`） | 总规矩「判不了的照常放行并记检出」 | 不起作用 |
| D2 | 分辨：改前退 2 一律拒，这几种被拒；改后放行 | 看得到（stderr 原句） | 口径「准入判法退 2 照拒」对文件头「只认两句」 | 修补把这几种从拒改成放，方向与口径相反 |

## 格 5　共用约束、主 agent 定义与代码此刻的一致性

### C1　`.claude/main-agent.md` 第 70 行：过期满一个窗口之后，同一行判的那一行同时落进「没声明」与「拖过 7200 秒」两条处置

- 原文：`.claude/main-agent.md:70`：「没声明串行段的，照那一行的出路停下那一趟、派人改装置再跑」；同一行后半：「解释不了的（判的那一行写「声明解释不了」）看门狗照叫醒：因声明拖过 7200 秒那一截叫醒的（那一行写「拖过了它算数的 7200 秒」），主 agent 查它的进度」。
- 代码：`declaration_note` 在窗口里算数的串行秒数为 0 时写 `scripts/hosts/core_usage.py:546`：「note = "；这一窗口里没有声明串行段"」，过期的段照样往后接「拖过了它算数的 …」（`scripts/hosts/core_usage.py` 第 550 行写这半句；第 466 行只要那一段声称到的时刻晚于基准就列进来）；不写「声明解释不了」。
- 给定输入与结果（量的，`out/declaration-cases.txt`；第 1000 秒写「此刻串行（预计 7200 秒）」、从不闭合、1 核，算数到第 8200 秒）：

```
name=expected_seconds expected=7200 now=8800 status=low cores=1.000 span=700 declared=100.0 excused=False problems=0 line_has=声明解释不了|拖过了它算数的
name=expected_seconds expected=7200 now=9000 status=low cores=1.000 span=600 declared=0.0 excused=False problems=0 line_has=这一窗口里没有声明串行段|拖过了它算数的
```

  第 8800 秒那一行合第 70 行「解释不了……拖过 7200 秒……查进度」；第 9000 秒起每一窗口的那一行写「这一窗口里没有声明串行段」又写「拖过了它算数的 7200 秒」，不写「声明解释不了」——按第 70 行头一条是「停下那一趟」，按后一条是「查进度、在走的照旧跑」。用户口径（`records/2026-10-29-一百三十七节那一轮调度记录.md` 第 132 行转述的原话「超过 7200叫醒 检查。继续照旧。」）落在后一条。
- 什么时候碰得到：主 agent 头一次读到这个单元的告警就在过期满一个窗口之后（看门狗晚起、上一次按 `类:` 确认过又作废、或第一次叫醒时没读到这一行）。推的，频率没量。
- 系统当时看得到吗：看得到，那一行里有「拖过了它算数的 7200 秒」；是定义的两条判据没写先后。

### 逐句对过、对得上的（没打中）

| 新句子 | 对的文件头或代码 | 结果 |
|---|---|---|
| `.claude/main-agent.md` 第 70 行「一段声明至多算到它写的预计秒数（写了超过 7200 秒的照 7200 算），没写预计秒数的算一个窗口（600 秒）」 | `scripts/hosts/core_usage.py` `counted_seconds` 与 `DECLARATION_EXPECTED_MAXIMUM_SECONDS = 7200`；格 2 的 7201、99999 两行与 7200 逐行相同 | 对得上（W2 那种共用文件除外，见格 2） |
| 同一行「声明解释得了这一窗口低核的（扣掉算数的串行秒数，剩下的时间把这一窗口的 CPU 全算给它也到得了阈值）看门狗只在报告里记一行、不叫醒」 | `scripts/hosts/core_usage.py:502` 与格 2 公式 9 行 | 对得上（W1 那种取整边界除外） |
| `.claude/main-agent.md` 第 64 行「只报本会话起的单元（按单元带的会话号认……认不出归属的照报）」 | `scripts/agents/agent-watch.py` 文件头第 196–203 行与 `unit_owner` | 告警记录那一半对得上；判定行那一半见 U1 |
| `.claude/agent-common.md` 第 96 行 ⑨ 的拒与判不了 | `.claude/hooks/runner-dispatch-guard.sh` 第 1607–1650 行 | 字面对得上；D1、D2 是两边都没写的结局 |
| `.claude/agent-common.md` 第 76、91 行「含 PATH 上没有 python3：由 shell 记这一条、退 0」 | `.claude/hooks/heavy-test-guard.sh` 第 205、817–820 行，`.claude/hooks/bash-command-detector.sh` 第 896、914 行 | 对得上 |
| `.claude/agent-common.md` 第 81 行 setsid「记一条检出；写成 `setsid --wait`」 | `.claude/hooks/bash-command-detector.sh` 第 32、134–135 行 | 对得上 |

规则纪律（`.claude/singlefs-ai-sop/rules/rules-discipline.md` 第 1–3、7 条）：这一轮两份文件新加的行（`~/.local/share/singlefs/work/d137-r1fix-defs/own-common.diff` 9 行、`own-main.diff` 3 行）里，位置指代与自称（上一节、下面、前面提到、上述、如下、同上、见下节、本条、本节、本表、本文档）与论证连词（因为、为了、所以）`grep -oE` 零命中；没有日期、旧值、「谁定的」。没打中。

| 编号 | 分不分辨 | 系统当时看得到吗 | 对不上的分句 | 修补件的改法在这一格上 |
|---|---|---|---|---|
| C1 | 分辨：改前没有「拖过 7200 秒」这条处置，不会撞 | 看得到 | 第 70 行头一条「没声明串行段的……停下」对后一条「拖过了它算数的 7200 秒……查进度」 | 共用约束那一件写的就是这两条，没定先后 |

## 我提的改法（都是推的：没实现、没在模型上量过；被攻过零轮）

| 打中 | 改法 | 修哪一格 | 状态 |
|---|---|---|---|
| W1 | core_usage 判一次、把 `declaration_excused` 写进告警记录，看门狗告警记录那一半读这个字段（缺字段的旧记录照叫醒），不再拿取整读数重算 | W1 的 77 次分歧 | 推的（没实现、没跑） |
| W2 | 声明行带写者（进程号或标签），`serial_intervals` 按写者分段算，各写者各自封顶；或至少文件头与第 70 行写明「按文件、最近的一段」 | W2 截断与续命两向 | 推的 |
| U1 | 判定行带起跑秒数（`<名>@<秒数>`），`verdict_counts` 按它找那一次的登记；做不到就同名有挪走的旧登记时当认不出、照数 | U1 | 推的 |
| D1 | admission.py 退 0 而 stdout 有 `sha256=refused` 时 ⑨ 记一条检出（照判不了放行） | D1 | 推的 |
| D2 | 交主 agent 定：「判法自己报的调用有错」归拒还是归判不了；不论哪边，第 1634 行说明照实写；起 admission.py 时去掉 `SINGLEFS_EXPERIMENT_RERUN_REASON` 与 `ADMISSION_BREAK` | D2 | 推的 |
| C1 | 第 70 行写明先后：那一行写「拖过了它算数的 7200 秒」的，不论写的是「声明解释不了」还是「这一窗口里没有声明串行段」，都照查进度那一条办 | C1 | 推的 |
| M1–M6 | 开关核对用全文比；首趟红的格在同负载下复现再下结论；三份钩子登记进 break-switch-replay 表或在修补件里跑整表；在底子全绿的树上核开关；复跑第一轮正推腿的整批样本；派活那一段重量一次整段长度再定预计秒数 | 各自那一条 | 推的 |

## 没打中的形状

- 格 2：公式 9 格、预计秒数 7 种写法 × 9 个时刻、跨窗口 5 个时刻、写坏的声明 5 种，函数结果都与文件头对得上（`out/declaration-cases.txt`）。声明预计秒数 0、超过 7200、写坏都落在安全的一边（照叫醒）。
- 格 3：会话号取不到、改动之前起的、登记读不了、别的会话的、名字不合写法、记录带的会话号与登记不同，共 14 种（`out/unit-owner-cases.txt`），`unit_owner` 与 `foreign_unit` 的结果都合文件头；`detached_run_state_directory` 与 detached-run.sh 第 392 行的默认登记目录取法一致。
- 格 4：退 77、退 2 两句、退 3、脚本不在、超时、退别的码六种处置与文件头、共用约束、调度口径逐格对得上。
- 格 5：新句子里位置指代、自称、论证连词零命中；第 76、81、91、96 行与钩子文件头对得上。
- 取样范围：W1 的随机取样是 cores 在 0–8、span 在 600–900、declared 在 0–span 上均匀取 20 万次（种子 137）；别的格都是写死的输入。

## 这条腿自己的限度

- 不造、不喂绕过钩子的写法，钩子漏拒那一半一条没攻；派发闸、重型闸、检出钩子都没喂，D1、D2 是读代码推的，没起 admission.py。
- M5 说的第一轮正推腿整批样本没替它复跑（复跑要喂钩子，不在这一条腿的范围里）；M3 说的旧开关没替它跑。
- W1、W2、U1、C1 是函数级：造的样本序列、造的登记目录，没在真单元、真看门狗下复现；本机重活那一半没碰。
- 我的模型与样本自证跑法：`scripts/hosts/run-with-memory-cap.sh 2G` 排了 182 秒没排上（slice 已占 72.0 GiB、总上限 40.1 GiB，`timeout 200` 到点退 124，没留下进程），之后照正文「五、交付要求」带 `ulimit -v` 与 `nice -n 19` 直接跑。
- 碰到一条与我无关的误拒（不攻、只记）：读代码时一条 `sed -n "$(grep -n … | cut -d: -f1),+45p" <文件>` 被重型闸拒成「sed 脚本解析不下去」，与正文二点名的 `sed -n "$((n-3)),$((n))p"` 同类，归辩方腿那一格。
- `scripts/hosts/detached-run.sh` 在开工快照之后被改过（0a9d3e70 → ac015ba4），我引的是此刻的行号；没核改了哪几行。

## 没做什么

- 没跑重型测试、没起 cargo、没碰 GPU、没跑门禁阶段；主工作区一个文件都没改（只写了报告与模型目录）。
- 不判辩方那几格（R1），没读禁读清单里的辩方腿报告与模型。
- 草稿目录 `~/.local/share/singlefs/work/d137-gov-r2-opus-b/` 里只有模型的输出与一份读本机登记的小脚本，没有仓副本、编译目录与库拷贝。
