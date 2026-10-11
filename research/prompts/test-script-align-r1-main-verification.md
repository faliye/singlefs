# test-script-align-r1 判决（2026-10-01）

正文 `research/prompts/_test-script-align-r1-body.md`；腿：正推 `research/prompts/test-script-align-r1-sonnet-output.md`（T1、T4）、攻方 `research/prompts/test-script-align-r1-opus-output.md`（T2、T3），模型目录各自 `-model/`。

## 一、核查

- `python3 scripts/cite-check.py <两份报告> --root . --background research/prompts/_test-script-align-r1-background.md`：「✓ 核了 69 处引文，对上 69 处，没判 0 处」。
- 开工快照 `research/prompts/test-script-align-r1-snapshot/judged-sha256.txt` 判决前 `sha256sum -c`：三份对不上——`.claude/agents/crash-verifier.md`、`.claude/hooks/lib_heavy_tests.py`、`scripts/verus-proof-check.py`，都是别的会话在腿跑着时改的（这一批没碰）。攻方报告写明它核过 `lib_heavy_tests.py` 的那次改动只动 Verus 判法几段、样本档与自证判法没变，三组关键探针用改后的钩子重喂退出码相同；主 agent 下面现查的行都按今天的文件。kb 快照全对。
- 主 agent 现查坐实的：
  - T2-2：`find .claude/gate.d/fixtures/checker-tier-layer0-replay.sh/quick-features-*/ -path '*layer0-sample-tools*' -type f -printf '%m %p\n'` 出 `664 …/cargo`、`664 …/rustc`、`664 …/quick-tier`：没有执行位。
  - T2-1：`.claude/hooks/lib_heavy_tests.py` 第 141、1595、2142 行一带，样本档放行看的是样本根里有没有 `.layer0-sample-tools`；里面放什么不核。
  - T3-1、T3-2：`scripts/crash-amplification.sh` 第 658 行驱动退 78 直接记 not-run（不看 `item_red`），第 715–719 行 `stopped_by_condition` 先于第 721 行 `run_verdict` 退 78。
  - T3-3：`grep -n 'ignored_points\|states_skipped_by_ignore' scripts/crash-amplification.sh` 零命中。
  - T3-6：`scripts/mutate.sh` 第 84–86 行只登记 250、251、253 三种包装结局，252、254 落进通用分支。
  - T3-7：`scripts/mutate.sh` 第 90 行 `BROKEN_JUDGEMENT="${MUTATE_BREAK:-}"` 平常跑也读。
  - T1-1：`scripts/crash-case-check.py` 第 190–192 行判 `parallel_line_*`；`.claude/rules/verification.md` 第 18 行本仓判的形态里没有它（这一批去重时删掉了「并行线号」）。
  - T1-2：`.claude/kb/checks-owed.md` C607 一行写「用例原子化」，副本标题是「用例原子化：每一条都能单独跑」。
- 没现查、照腿的报告采的：T2-3、T2-4、T2-6、T3-4、T3-5、T3-9（攻方探针在模型目录，`rerun.sh` 可复跑）；正推 T4 各笔欠账的现状对不上（腿逐行给了 grep 命令与零命中，判决里照它改）。

## 二、判定与去向

| 格 | 判定 | 去向 |
|---|---|---|
| T2-1 54 号样本档只看标记目录在不在 | 采 | 改钩子：样本档放行要求标记目录里的假 `cargo` 可执行、内容与仓里某份 54 号样本的假 `cargo` 逐字节相同；54 号样本档自己也拒「标记目录里的 cargo 不可执行」 |
| T2-2 新样本的假工具没有执行位 | 采 | 四份新样本的 `.layer0-sample-tools/*` 改成可执行（git 模式 100755）；54 号样本档开跑前核标记目录里的假工具都可执行，不可执行退 2 |
| T2-3 自证登记按路径尾部认 | 采 | 改钩子：登记表的键按解析后的真实路径认，必须是本仓里那一份文件；别处同名尾部的脚本照旧按提交时才跑的检查拒 |
| T2-4 样本里的 `setup.sh` 不核 | 采 | 改钩子：经 `stage-selftest.sh` 喂样本时，样本目录相对 HEAD 加暂存区有改动（含 `setup.sh`），照旧拒；只放行和已提交内容逐字节相同的样本 |
| T2-5 登记为 light 的没起重活 | 只抽一次，不作「没打中」用 | C613（自证登记表靠读代码判轻重，脚本改了没人核）的「怎么拦」按它的 strace 结果改：`cargo -V`、`rustc -V` 这类只问版本的不算起重活 |
| T2-6 定义第 7 步照字面跑不了 samples-light 的样本；钩子把 `case` 模式行当成起共用库 | 采 | 定义与共用约束改成「照钩子拒绝说明里写的样本档起法跑；起不了的写明交主 agent」；`case` 误拒交改钩子那一件一并修；向上游提 `stage-selftest.sh` 能只喂点名的几道 |
| T3-1 前一项红、后一项退 78，整趟退 78 | 采 | 有红先判：任何一项红，整趟退 1，退 78 的那一项照样在汇总里点名 |
| T3-2 驱动退 78 而行里报红，记成本次未跑 | 采 | 行里报了红就记红，不论驱动退几 |
| T3-3 入口不读 ignore 那两个字段 | 采 | 入口读 `ignored_points=`、`states_skipped_by_ignore=`，成功句报跳过了多少、哪几类，照上游「按登记整批不判的一类不算剪枝……成功那句报跳过了多少」 |
| T3-4 行里没有 `red=` 或不是数也判绿；现判 0 复用 0 判全量绿 | 采 | 行里缺 `red=` 或不是非负整数判红；全量那一趟现判与复用都是 0 判红 |
| T3-5 `CRASH_AMPLIFICATION_ITEM_DRIVER` 平常跑也认 | 采 | 与弄坏开关同一套：只在自证起的子进程里认，平常跑设了退 2 |
| T3-6 `mutate.sh` 把 252 记无效、254 记抓到 | 采 | 252（排队等满）、254 各自分类，都让整轮判失败；文件头退出码表补全 |
| T3-7 `MUTATE_BREAK` 平常跑也认 | 采 | 只在自证起的子进程里认，平常跑设了退 2 |
| T3-8 54 号不认 `cfg_attr` | 潜在，今天无实例 | 记进 C604（快档不开特性，崩溃放量与 GPU 对拍编进来零条）的射程句，不改 |
| T3-9 分项驱动 `--expand-up-to` 缺值退 1 | 采 | 改成退 2 |
| T3-10、T3-11 | 只抽一次 | 不作「没打中」用 |
| T1-1 去重丢了「并行线号」的例子 | 采 | 主 agent 在 `.claude/rules/verification.md` 第 18 行补 `*_parallel_line_*` |
| T1-2 C607 节名截短 | 采 | 改成全名 |
| T1-3 C20「定期全量扫」与全量只在用户要求时跑；D13「合并前」没有触发者 | 交用户 | 这两处是决策正文的意思，不在这一轮定，弹窗问用户 |
| T1-4 进度间隔 | 一致 | 不改 |
| T1-5 三处定义对「没登记」「heavy-if-set」写法不一 | 采 | 三处统一成同一句，并点名 heavy-if-set 的意思 |
| T4 C604–C613、C620 现状栏与工作区不符、怎么拦落不成或会误判 | 采 | 照正推报告逐行改写（C604、C606、C610、C611、C612 现状；C605 改成跨趟两次跑、键层另点变异；C607 前置写明要先给四个随机二进制加注入失败的入口；C609 补产物怎么选与前置 H6；C610 改成按结构判、不按词表；C613 补「怎么认真工具」与 T2-4、T2-5；C620 去掉 nvidia-smi、PCI 加 class 过滤） |
| T4 新缺口 | 采 | 新记：取不出复用数没有棘轮；「一律进快档的几类」没逐类核；弄坏开关文件头三样没有门禁；H9 走 GPU 的计算类登记还没落地；H6 E165 钉表整行抄还欠脚本与表那一半 |
| D13 改写 | 正推判无多余改动 | 不改；「合并前」的触发者交用户（T1-3） |

这一轮采的改法都是腿或主 agent 提的，被攻过零轮；第二轮只攻这些改法与前一轮站住的形态。

## 三、被判文件里点名的定义与共用约束

`.claude/agent-common.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/crash-verifier.md`、`.claude/main-agent.md`：这一轮判了它们这一批改过的段落（T1-5、T2-6），改法见上表；`CLAUDE.md` 的两处同判。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 9 | 不影响 | 2026-10-01 不受影响：这一轮只核「夜间档」改「全量档」没有多余改动；「合并前」的触发者交用户 |
| D13（验证路线） 已定项 15 | 不影响 | 2026-10-01 不受影响：同上，只核措辞 |
