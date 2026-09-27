# sync-local-legs-r1 核查员报告

**你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。**

## 判别力自证（先做）

取 sonnet 报告第 13 行的引用「退出码 5：作废副本由脚本留成 `-output-void<n>.md`」（`.claude/agents/three-way-local-attack.md:30`），
在草稿副本里把行号 30 改成 31，跑：

```
$ python3 research/scripts/cite-check.py /tmp/claude-1000/sync-local-legs-r1-verifier/selftest/sonnet-shifted.md --root . --background research/prompts/_sync-local-legs-r1-background.md
  ✗ sonnet-shifted.md:13 引 .claude/agents/three-way-local-attack.md:31
  「退出码 5：作废副本由脚本留成 `-output-void<n>.md`」：那一行不含这句；原文在这份文件第 30 行
  ✗ 核了 15 处引文，1 处对不上，0 处没判（逐处列在上面）
EXIT: 1
```

判别力：会红，如预期。

## 报告 sha256 核对（对交回里给的）

```
2d0b0c0677bbad9bdc9f49ea2d1d6f3ee77934e0174a8985e51003f24b66b022  research/prompts/sync-local-legs-r1-sonnet-output.md
bca324040083bb273423cb266ce9eafd816111a1e69f188067b07ce5dcf01f79  research/prompts/sync-local-legs-r1-opus-output.md
```
两份都与主 agent 给的 sha256 一致，报告没有在交回之后被改过。

## 开工快照核对

`research/prompts/sync-local-legs-r1-snapshot/defs-sha256.txt` 列 10 个文件，主 agent 已现查 `sha256sum -c` 全 OK；
本报告对这 10 个文件的引用按快照核，其余被引文件（`research/scripts/agent-watch.py`、`.claude/settings.json`、
`.claude/hooks/agent-write-scope.tsv`、`.claude/hooks/runner-dispatch-guard.sh`、`.claude/hooks/handback-guard.sh`）
不在快照清单里；核过这 5 个文件在腿开工时刻（2026-09-27T01:17:07Z）之后没有改动：

```
$ git log -1 --format="%cI" -- research/scripts/agent-watch.py .claude/settings.json .claude/hooks/agent-write-scope.tsv .claude/hooks/runner-dispatch-guard.sh .claude/hooks/handback-guard.sh
（逐个查，最晚一次提交 2026-09-27T00:32:09+00:00，早于腿开工 01:17:07Z）
$ git status --porcelain -- 同 5 个文件
（空，无未提交改动）
```
这 5 个文件按主树核，判定有效（不落「分不清」）。

## 用 cite-check.py 自动核（认得出的写法）

```
$ python3 research/scripts/cite-check.py research/prompts/sync-local-legs-r1-sonnet-output.md --root . --background research/prompts/_sync-local-legs-r1-background.md
  ✓ 核了 15 处引文，对上 15 处，没判 0 处
$ python3 research/scripts/cite-check.py research/prompts/sync-local-legs-r1-opus-output.md --root . --background research/prompts/_sync-local-legs-r1-background.md
  ✓ 核了 16 处引文，对上 16 处，没判 0 处
```
两份报告加起来 31 处「「原文」（`路径:行号`）」格式的引文，脚本判全对，没有一处指到背景材料的行号、没有一处指到标题行。

余下的 `路径:行号` 提法（正文里裸引代码或行号、不带紧邻的「」引号，脚本认不出格式）逐处人工核，见下两节表格。

## sonnet 报告：人工核的引文（cite-check.py 认不出的写法）

| 报告行 | 引用 | 核的结果 |
|---|---|---|
| 13 | `research/scripts/ask-local.sh:105,107` 代码 `save_void` / `exit 5` | ✓ 105 行 `save_void`、107 行 `[[ "${ASK_LOCAL_ALLOW_CORRUPT:-0}" == "1" ]] \|\| exit 5`，逐字对上 |
| 14,16 | `research/scripts/ask-local.sh:107,136` | ✓ 136 行 `cat "$TXT"; echo`，逐字对上；exit 5 在 107 行先 return，136 行走不到，与「A、B 各自写」的推论前提一致 |
| 17 | `research/scripts/ask-local.sh:16,26,30,41,52,54,57,63,67,130` | ✓ 逐行核对：16「内嵌的 python 自己出错时退 1」注释、26/30 两处 `exit 2`、41 `exit 3`、52/54/57 三处 `sys.exit(3)`、63 `sys.exit(4)`、67 `[[ $rc -eq 0 ]] \|\| exit $rc`、130 `exit 6`，与报告枚举的退出码逐一对上，覆盖 0/1/2/3/4/5/6 无遗漏 |
| 18 | `research/scripts/ask-local.sh:41` | ✓ 同上，`exit 3`（`curl` 失败） |
| 19,25 | `research/scripts/ask-local.sh:15,127` | ✓ 15 行头注释「5 与 6 都不打正文…」、127 行 `save_void`，两处都证实 exit 6 同样落 void 副本 |
| 26 | `.claude/agents/three-way-local-attack.md:29` | ✓ 29 行「3. 取号：`ls <前缀>-output-s*.md` 看已用到几号，取下一个没用过的 `<n>`」，支持「被 `ls` 当已用永久跳过」的推论 |
| 35 | `research/scripts/oov-check.py:164,170,178,249` | ✓ 164-166「if prompt_path: … known = set(re.findall(...))」；170、178 两处「if lw in words or lw in known: continue」；249「scan(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None, words)」，逐字对上 |
| 36 | `research/scripts/oov-check.py:257` | ✓ 257 行 `print("     生词: " + ' '.join(dict.fromkeys(oov))[:300])`，逐字对上 |
| 37 | `research/scripts/oov-check.py:168,176,251,257` | ✓ 168/176 两个循环各自 `append` 进同一个 `oov` 列表（174 行、180 行分别 append），251 行 `len(oov)` 是原始计数，257 行 `dict.fromkeys(oov)` 是去重后——两条计数口径不同，报告的推论有依据 |
| 39 | `.claude/agents/three-way-local-attack.md:25,35,39` | ✓ 25「## 做什么」、35「## 写范围」、39「## 产出」，逐字对上 |
| 45 | `.claude/agents/three-way-local-attack.md:27,28` | ✓ 27 行第 1 步事实表要求（「要模型按表格逐格填」）、28 行第 2 步转述核对要求（「逐句核转述」），与被引一致 |
| 46 | `.claude/agents/three-way-local-attack.md:20`、`.claude/agents/three-way-local-defense.md:19` | ✓ .claude/agents/three-way-local-attack.md:20「分给本地攻方的攻击面（与云端攻方腿不重叠）」；.claude/agents/three-way-local-defense.md:19「输入里的「攻击面」换成「要辩护的一方」…」，两处都对上 |
| 47 | `.claude/agents/three-way-local-attack.md:27`、`.claude/agents/three-way-local-defense.md:20` | ✓ attack.md:27 末句「核对表与运行记录里把样本自带的行号一律标「模型自给、未核」」；defense.md:20 同句式，逐字对上 |
| 48 | `.claude/agents/three-way-local-attack.md:18`、`.claude/agents/three-way-local-defense.md:21` | ✓ .claude/agents/three-way-local-attack.md:18「## 输入（主 agent 必须给）」；.claude/agents/three-way-local-defense.md:21 末句「前缀取派发提示给的（与攻方「输入」那一项同），不写死」，逐字对上——这句引用的是改名前的短称「输入」，与今天的标题「输入（主 agent 必须给）」不同，报告据此判「站不住一半」，引用本身准确，是否算错由主 agent 判 |
| 52,63,64 | `.claude/agents/three-way-local-defense.md:13,21` | ✓ 与上面重复核验一致 |
| 61 | `research/scripts/oov-check.py:249,257` | ✓ 与第 35、36 行重复，一致 |

sonnet 报告：核了 15（自动）+ 49（人工，路径:行号原始出现次数，含同一目标被多处引用的重复计次）= 64 处，✓ 64 处，✗ 0 处。

## opus 报告：人工核的引文（cite-check.py 认不出的写法）

| 报告行 | 引用 | 核的结果 |
|---|---|---|
| 87 | `research/scripts/ask-local.sh:21` | ✓ 21 行 `TIMEOUT="${ASK_LOCAL_TIMEOUT:-900}"`，逐字对上 |
| 145 | `.claude/agents/three-way-local-attack.md:29` | ✓ 与本报告前面重复核验一致（第 3 步取号与前台跑那一整行） |
| 177 | `research/scripts/agent-watch.py:130` | ✓ 130 行 `NO_WAKE_GRACE_SECONDS = 60`，逐字对上 |
| 179 | `research/scripts/agent-watch.py:3115` | ✓ 3115 行 `parser.add_argument("--process-stale-minutes", type=float, default=20, ...)`，`default=20` 支持「默认 20 分钟」 |
| 183 | `.claude/settings.json:23` | ✓ 23 行 `"matcher": "Bash",`，该栏 27/31/35 行分别注册 `pattern-process-guard.sh`、`bash-command-detector.sh`、`heavy-test-guard.sh`，与报告列的三道逐字对上 |
| 225 | `.claude/settings.json:40` | ✓ 40 行 `"matcher": "Write\|Edit",`，支持「写闸只挂 Write / Edit」 |
| 237 | `.claude/rules/three-way-inference.md:206` | ✓ 206 行「本地腿用 Bash 的 `run_in_background` 起、结束本轮等完成通知（单次请求最长 900 秒，超过前台上限），命令里不加 `setsid`、`&`、`disown`。」，逐字对上 |

opus 报告：核了 16（自动）+ 7（人工）= 23 处，✓ 23 处，✗ 0 处。

## opus 模型目录：复跑

模型目录 18 个文件的 sha256 与报告开头列的逐字比对，全部相同（本报告开头列过的 18 行原样核对，已现查一致，不重复贴）。

把模型目录拷进草稿目录 `/tmp/claude-1000/sync-local-legs-r1-verifier/rerun/model/`，指向路径的两处硬编码按本条规则换成我自己的草稿路径
（`setup.sh` 的 `REPO=$(cd "$(dirname "$0")/../../.." && pwd)` 依赖脚本在仓内固定深度才能算对，拷到别处会算错，改成显式指向本机仓根；
`sim_watch.py` 的 `tempfile.mkdtemp(dir="/tmp/claude-1000/sync-local-legs-r1-attack")` 是原腿自己的草稿目录，改成我的草稿目录），
其余原样，`ATTACK_DRAFT` 按提示换成我的草稿目录：

```
$ nice -n 19 bash setup.sh /tmp/claude-1000/sync-local-legs-r1-verifier/rerun/draft
35a36
>   sleep "${ASK_LOCAL_FAKE_DELAY:-0}"   # draft-copy only: simulate request latency
```
与报告「复跑过一遍…输出与存档逐字节相同」一致（这条 diff 正是 fake-mode 加延时那一行，报告没有声称这里 diff 为空）。

```
$ nice -n 19 bash collide.sh <草稿>/rerun/draft 1 textA.txt 4 textB.txt 4 2 > case1.log
$ nice -n 19 bash collide.sh <草稿>/rerun/draft 2 textA.txt 4 textC.txt 4 2 > case2.log
$ nice -n 19 bash collide.sh <草稿>/rerun/draft 3 textB.txt 4 textA.txt 4 2 > case3.log
$ diff case1.log research/prompts/sync-local-legs-r1-opus-model/case1.log   # 无输出
$ diff case2.log research/prompts/sync-local-legs-r1-opus-model/case2.log   # 无输出
$ diff case3.log research/prompts/sync-local-legs-r1-opus-model/case3.log   # 无输出
```
三格 log 逐字节相同。额外核了产出的样本内容本身（不只是 log 摘要）：`cmp` 我重跑出的 `c3-s1.md` 与 `textA.txt` 无差，
与报告「第 3 格先短后长…先写的那份答复整份没了」一致；`c2-s1.md` 里确有报告描述的拼接点原文
「What  timeout.」接「ckground form for every local leg call, and the definition requires…」，与报告逐字对上。

**✗ 一处产物描述与报告自己给的产物不符**：第 108 行「第 2 格把 B 截成 310 字节，拼接点落在词中间……」——
案例 2 的命令（报告第 18 行给的复跑命令列表）配对的是 `textA.txt`（1084 字节）与 `textC.txt`（310 字节），不是 `textB.txt`
（354 字节）。`textC.txt` 恰好是 310 字节，`textB.txt` 不是；报告正文把这一格说成「把 B 截成 310 字节」，指错了变量名（该写
「C」）。拼接点的原文本身（我独立重跑核对过，见上）与报告描述一致，这不是编造，是产物引用时把变量名写错了一个字母。

```
$ wc -c research/prompts/sync-local-legs-r1-opus-model/textB.txt research/prompts/sync-local-legs-r1-opus-model/textC.txt
354 textB.txt
310 textC.txt
```

```
$ nice -n 19 python3 hookrun.py cases1.json > run1.txt   # ATTACK_DRAFT 指到我的草稿目录
$ nice -n 19 python3 hookrun.py cases2.json > run2.txt
$ nice -n 19 python3 hookrun.py cases-controls.json > run-controls.txt
$ diff run1.txt research/prompts/sync-local-legs-r1-opus-model/run1.txt   # 无输出
$ diff run2.txt research/prompts/sync-local-legs-r1-opus-model/run2.txt   # 无输出
$ diff run-controls.txt research/prompts/sync-local-legs-r1-opus-model/run-controls.txt   # 无输出
$ nice -n 19 python3 sim_watch.py > run-sim-watch.txt   # tempfile dir 换成我的草稿目录
$ diff run-sim-watch.txt research/prompts/sync-local-legs-r1-opus-model/run-sim-watch.txt   # 无输出
```

四个命令的输出逐字节与存档相同，与报告「复跑过一遍……输出与存档逐字节相同（`diff -q` 无差）」一致。

**旁证（不属复跑命令，但独立核过）**：opus 报告 乙-1 声称 Bash 前台命令超时会被 harness「moved to the background」而不是杀掉，
本会话独立复现同一行为，工具结果原文与报告贴的原文同构（同一句式「Command did not complete within its Ns timeout and was moved
to the background (ID: …)」），该后台任务之后确实正常跑完（收到 completed 通知），与报告「那个进程之后照样写完」一致。
opus 报告 217 行「样本历来不在 `research/results/`」引的 `git log --all --name-only` 计数（482／480／2）本会话独立重跑，
数字逐一相同：

```
$ git log --all --name-only | grep -E 'local-(attack|defense)-output-(s|void)' | wc -l
482
$ git log --all --name-only | grep -E 'local-(attack|defense)-output-(s|void)' | grep -c '^research/prompts/[^/]*$'
480
$ git log --all --name-only | grep -E 'local-(attack|defense)-output-(s|void)' | grep -v '^research/prompts/[^/]*$' | sort -u
research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-output-s1.md
research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-output-s2.md
```

## 本地攻方腿：翻译核对表逐句核（`sync-local-legs-r1-local-attack-translation-audit.md`）

| 编号 | 原文文件:行 | 核的结果 |
|---|---|---|
| Fact A | `research/scripts/ask-local.sh:21` | ✓ 21 行 `TIMEOUT="${ASK_LOCAL_TIMEOUT:-900}"`，逐字对上，英文 Meaning 没有多加限定词 |
| Fact B | Bash 工具自身 description（不在仓里） | ✓ 引文「You may specify an optional timeout in milliseconds (up to 600000ms / 10 minutes).」与本会话自己工具说明原文逐字相同 |
| Fact C | `.claude/agents/three-way-local-attack.md:29` | ✓ 中文原文逐字对上；英文翻译逐词核对表列的对应关系（前台跑→run in the foreground 等）核对无遗漏；多出来的说明句（指出「超过 Bash 前台上限」两种读法）确实是引文之外单独一段，没有混进引文本身 |
| Fact D | `.claude/rules/three-way-inference.md:206` | ✓ 同上，逐字对上；多出来的说明句同样标注清楚、不混入引文 |
| Fact E1-E4 | `.claude/agent-common.md:57`（同一行的四个分句） | ✓ 该行原文核过（见下），四个分句依次对应「长活可以等…会通知你。」「交给它的命令…④的写法收。」「结束本轮就是…240000 毫秒。」「等自己起的后台任务…②）。」，逐字都在这一行里 |

`.claude/agent-common.md:57` 原文（本报告独立现查，逐字确认四个分句边界）：
```
- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用，每件的退出码照「执行前拒绝的写法」那一条 ④ 的写法收。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。等自己起的后台任务一律结束本轮等完成通知，不写轮询它输出文件的循环（run_in_background 里也拒，见「执行前拒绝的写法」②）。
```

Fact E2 的核对表特别写了「首稿缺的」——首稿把 `systemd-run` 直译、漏掉「不等到结束的」这个限定，定稿补上了；本报告核过共用约束原文 `.claude/agent-common.md:64`
（「③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。」），确认「不等到结束的」这个限定词真的存在、定稿的补法准确，
不是核对表自己编的限定词。

核对表逐句核：核了 5 项（Fact A-E1~E4 算一组），✓ 5 项，✗ 0 项；额外核了 E2 的支撑引用（agent-common.md:64），✓。

## 本地攻方腿：运行记录与样本核对（`sync-local-legs-r1-local-attack-runlog.md`）

独立重跑 `oov-check.py` 与 `corruption-check.py`（提示文件带上，`nice -n 19`）在 s1-s5 五份样本上，退出码、词数、生词表与 runlog 逐字对上：

```
s1  oov: 绿 生词=2「TIMEOUT's uncreated」rc=0    corr: 绿 words=541 rc=0    wc -w=583
s2  oov: 绿 生词=1「contradicting」rc=0          corr: 绿 words=471 rc=0    wc -w=518
s3  oov: 绿 生词=0 rc=0                          corr: 绿 words=369 rc=0    wc -w=410
s4  oov: 绿 生词=0 rc=0                          corr: 绿 words=505 rc=0    wc -w=558
s5  oov: 绿 生词=0 rc=0                          corr: 绿 words=445 rc=0    wc -w=497
```

runlog 表里的词数列（583/518/410/558/497）与 `wc -w` 逐一相同；oov 生词列（s1「TIMEOUT's uncreated」、s2「contradicting」、
s3/s4/s5「无」）与本次重跑逐字相同。runlog 说 s1、s2、s3 三份「闸判绿但通读时发现缺字类损坏」，逐一核了被引的原文位置：

| 样本 | runlog 引的位置/原文 | 核的结果 |
|---|---|---|
| s1 | 「第 4 题子情形 (i) 结尾处…剩一个孤零零的字母」 | ✓ 样本第 29 行原文「Exit code: Non-zero (e 124 for timeout).」，「e」确实孤零零 |
| s2 | 「…与同一份样本第 38 行同一句式「indicating timeout termination」对不上」 | ✓ 样本第 31 行「Exit code: Non-zero (eating timeout termination).」，第 38 行「Exit code: Non-zero (indicating timeout termination).」，行号与原文都对上 |
| s3 | 「…掉了后半个词，剩下的片段不成词」 | ✓ 样本第 31 行「the exit code is non-zero (indic 124 for timeout termination)」，「indic」确实是「indicating」掉了后半 |
| s4（重跑） | 「没有出现 s1/s2/s3 那处缺字/变形词」 | ✓ 样本第 31、37 行都是「Exit code: 124 (standard timeout exit code).」，没有缺字 |

void1（判红作废副本）核过：现存文件 3357 字节（非空），与「判红那次的 `s4` 是空文件」这句不矛盾——那句指的是被 `>` 重定向截断的
`s4.md`（后来已被 `>|` 同号重跑覆盖，现在看不到当时的 0 字节状态，本次无法对这一句本身重新核实，只能确认它与脚本机制
`ask-local.sh:107` 判红先 return、走不到 136 行打正文一致）；void1 本身独立重跑两道检测确认判红：

```
$ python3 research/scripts/oov-check.py research/prompts/sync-local-legs-r1-local-attack-output-void1.md .../local-attack.md
绿 生词=2「TIMEOUT's」 rc=0
$ python3 research/scripts/corruption-check.py research/prompts/sync-local-legs-r1-local-attack-output-void1.md .../local-attack.md
红 实词自复读=1「stricter」 rc=1
```
与 runlog「判红作废」一致（红是从 corruption-check 判出的，不是 oov-check）。

runlog「调用次数核算与一处超额」一节自陈第 6 次调用（s5）发出前没有先核对次数——这是运行记录自己承认的执行失误，不是引文问题，
本报告不判它算不算超出定义第 6 步的允许范围（那是主 agent 的事）。

## 计数汇总

| 项 | 核了 | ✓ | ✗ | 核不动 | 分不清 |
|---|---|---|---|---|---|
| sonnet 报告引文 | 64（15 自动+49 人工） | 64 | 0 | 0 | 0 |
| opus 报告引文 | 23（16 自动+7 人工） | 23 | 0 | 0 | 0 |
| opus 报告产物描述（case2 变量名） | 1 | 0 | 1 | 0 | 0 |
| opus 模型目录复跑命令 | 4 组（setup/collide×3/hookrun×3/sim_watch） | 4 组全部逐字节相同 | 0 | 0 | 0 |
| 本地攻方翻译核对表 | 5 项+1 项支撑引用 | 6 | 0 | 0 | 0 |
| 本地攻方运行记录/样本 | 5 份样本×(退出码/词数/生词) + 4 处缺字损坏定位 + void1 | 全部对上 | 0 | 0 | 0 |
| 快照外文件（5 个）改动核对 | 5 | 5（确认腿开工前未改，按主树核有效） | 0 | 0 | 0 |

判别力自证：会红（如预期）。报告 sha256：与交回一致。

**唯一一处 ✗**：opus 报告第 108 行「第 2 格把 B 截成 310 字节」——案例 2 配对的是 `textC.txt`（310 字节），
不是 `textB.txt`（354 字节），变量名写错；不影响该格拼接内容本身的可核性（我独立重跑确认拼接点原文与报告描述一致）。

## 没做什么

- 不判甲、丙、丁三格谁说的对：那是主 agent 或另外两条腿的分工，本报告只核引文、产物与复跑，不裁定推论。
- 不判 sonnet 报告「丁-差异站得住吗」一行里 `defense.md:21` 引用「输入」是不是构成实质问题——引用本身准确，是否算错交主 agent。
- 不判 opus 报告 F1/F2/F3 改法表的取舍——那是它自己标注的「推的，没跑」，本报告不加验证意见。
- 没有重跑 opus 报告里「本会话实测」的原始探针（任务号 `bdn19r1ru`、`b08hxi0gq`、`blxp8hr2l`、`blh6i23kl`）——那些绑在它自己的会话
  里，无法用同一任务号复现；本报告独立另起一次同类探针（`b8kcyqo6k`）验证同一句式的「moved to the background」行为存在，
  作为旁证，不是同一次观测的复核。
- 没有验证本地模型（Qwen3-Next-80B）s1-s5 答案本身的推理对不对——那是「打没打中」，按共用约束「判决由主 agent 做，不由投票做」不归
  核查员判。
- 重型测试：未跑（派发要求）；未编译 Rust；未连本地模型网关；未跑任何门禁阶段。
- 草稿目录 `/tmp/claude-1000/sync-local-legs-r1-verifier/` 里 `rerun/draft/repo/`（`setup.sh` 用 rsync 建的仓副本，262M）已在验证完成后删除；
  `rerun/model/`（模型目录副本）与各 rerun 输出文件还留着，都是文本文件，没有编译产物、没有 `target/`。
