# 广谱变异接进提交门禁的治理改动：第三轮（终轮）正文（2026-10-03）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 H1 H2 H3 H4 H5 J1 J2 J3 J4 K1 -->

## 一、题面

第二轮判决 `research/prompts/m3-broad-gate-gov-r2-main-verification.md` 第三节定了改法，第四节交用户两条。用户 2026-10-03 弹窗定：第 1 条（确定性超时与内存撞顶怎么出去）「增加新的登记表，但是不能交给用户判，要能够有足够的证据自动判」；第 2 条取甲，改完开第三轮（终轮）。工具实现员照改，报告 `/tmp/claude-1000/m3-r1/batch6-broad-gate-r3fix/report.md`（「逐条结果」第 1–7 条写了改在哪、出口怎么核）；`commit-stages.md` 那四处它写成规格、由主 agent 套上。

这一轮是终轮：攻第二轮站住的改法今天落成的样子与用户定案的升档登记表，复核前两轮的结论与判定过程；前两轮攻过的角度（第一轮 G1–G5、第二轮 H1–H5 各格的题面）不重攻，只攻它们的修补与没被看过的地方。这一轮新冒出的形态是零轮，写进判决交用户表，不再开第四轮。

被判的（冻结副本里今天的样子）：

| 文件 | 这一批改了什么（报告「逐条结果」） |
|---|---|
| `.claude/gate.d/checker-tier-broad-mutation.sh` 与 `.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/` 下的样本 | `--marker-check`（第 1 条）；字面值绿样本（第 2 条）；gate-verdict 逐行复核 `escalated-verdicts.tsv`、核 ESCALATED 条数、一绿五红样本（第 7 条）；出路句（第 6 条） |
| `research/scripts/broad-mutation.py` | 自证 55 → 74 格（第 2、3、4、7 条）；`--unit-environment`（第 4 条）；`build_environment_of` 改调 `admission.py` 的 `build_environment_lines`、加文档测试变量（第 5 条）；升档重跑与证据、登记表（第 7 条，文件头「升档」一段）；`heavy_prefix_text`、退出码 8、出路句（第 6 条） |
| `.claude/skills/crash-test/commit-stages.md` | 第 19 行这一道改起门禁 `--marker-check`；第 26 行 ③ 带 `--unit-environment`、升档、「超时、内存撞顶不算定论」；第 28 行 7 的出路与 NO_CONCLUSION 的出路 |
| `.claude/rules/verification.md` | 「门禁管哪一半」广谱变异那一行补升档复核 |
| `.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh` | 拒绝说明补 `-h`（只改说明，分类没动） |
| `research/scripts/admission.py` | 没改；被驱动调 `build_environment_lines` |

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 腿读冻结副本 `/tmp/claude-1000/m3-gov-r3-freeze/tree/`（`crates/`、`litmus/`、`.claude/`、`.cargo/`、`research/scripts/`、`Cargo.*`，3 205 个文件），sha256 在 `research/prompts/m3-broad-gate-gov-r3-snapshot/tree-sha256.txt`。`crates/` 这一轮不被判，只当驱动与门禁的输入。
- 主 agent 现跑：报告「要主 agent 跑的命令」第 2 条，攻方那六种写错版驱动的 `--selftest`：gkl-no-toolchain「74 格里 1 格判错」、gkl-no-files 1、gkl-no-env 8、gkl-no-tools 2、gkl-no-limits 2、fop-path-blind 1。
- 工具实现员自己核过的（主 agent 没复跑）：驱动 `--selftest`「74 格都对」；门禁样本 36 份全对（照「cd 进样本根、根参数写同一个目录」）；新开关驱动 12 个、门禁 6 个各红在点名那几格；env-gap 7 种全局键全变；`admission.py --selftest` 改前改后逐字节相同，都是 283 格里 1 格判错（environment-outside-stage-marker，改之前就红）。
- 它自报没核的：包装真打的「超过限时 N 秒」「撞了内存上限 X」两句只照 `research/scripts/run-with-memory-cap.sh` 第 946、951 行的 echo 写，没经真包装跑；`--memory-cap` 大于约 26G 时升档 1.5 倍放不下 slice 总上限（约 40 GiB），那几条照原结局记没结论（推的）；基线峰值写不进 `baseline-peaks.<线程数>.tsv` 时每次重起都重跑基线（没造现场）；`--unit-environment` 与 `--marker-check` 在重型闸里算广谱变异、要前缀。
- 全局键写法变了：库目录现有各版下一次真跑退 7，提交时要另起一版整版现判。

## 三、格

### J1　第二轮站住的改法今天的样子撑不撑得住
第二轮判决第二节第 1、2、3、5、6 行定的改法，逐条找一个改完之后仍该红不红、或新引入的不该红而红的输入。重点：
- `--marker-check` 核的路径集合与 `stage-inputs.tsv` 那一行、与写标记前那一核是不是同一份；它退 1 / 0 / 3 三种在 `commit-stages.md` 第 19 行的读法与门禁实际退出码逐个对。
- 字面值绿样本：它写死的全局键与指纹是不是从一个独立于驱动的算法来的；驱动合法改动（比如加一类全局键行）时它怎么跟、会不会被人照驱动的输出重抄而失去独立性（推的写推的）。
- `build_environment_of` 调 `admission.py` 之后：两边共用一份定义，这一份写错时谁盯（第一轮、第二轮都打中过「样本与被判代码共用函数」，这一处是不是同形）；`--unit-environment` 打出的变量集合与全局键认的集合是不是同一份。

### J2　用户定案的升档登记表：证据够不够自动判
用户定的是「不交用户判、凭足够的证据自动判」。攻：
- 假抓到：有没有一个输入让驱动把一条不加限时 / 不限内存时会跑完且测试全过（该记没红）的变异判成 caught-by-timeout / caught-by-memory 写进登记表。K（20 倍基线时间、2 倍基线峰值）在本机负载、别的会话占 CPU 与内存、基线取的那一趟偏快 / 偏小时站不站得住；基线那一行取的是哪一趟、会不会是别的线程数下的。
- 门禁复核与驱动写证据共用什么：倍数门槛「取此刻驱动里的常量」、日志里那一句的字面；共用的那一份写错时谁盯。
- 手写行与证据伪造：门禁靠「日志在 `versions/<那一版>/escalation/` 下且 sha256 对」认驱动写的；一个照格式造出来的日志与行能不能过。用户要的是「足够的证据」，判它够不够，也给出最小的补法。
- 升档放不下（`--memory-cap` 大于约 26G）、包装自己的 251 / 252 / 254、没有整步限时的测试步不升档：这几支照原结局记没结论之后，用户定案的出路还走不走得通。

### J3　前两轮的结论与判定过程
- 第一轮第二节、第二轮第二节每一行：判的依据今天还成立吗，有没有一行的结论被这一批改法悄悄改掉了而判决没写。
- 前两轮记成「没打中」的格只抽过一次样；挑最脆弱的一两格再抽一次。
- 前两轮判决报出的数（第二轮第五节的复跑数、样本数、开关数）与这一批报告报出的数（74 格、36 份、18 个新开关、env-gap 7 种）能不能在冻结副本上核；核不动的写为什么。

### J4　没收的线：未定项与欠账清单完不完整
前两轮判决与这一批报告里每一条「不做 / 记欠账 / 下一批 / 推的没量」逐条列，标它今天落在哪（调度记录、`.claude/kb/checks-owed.md`、驱动或门禁文件头），找一条无声消失的。已知要进下一批的：第二轮第 8 行（③ 核指针是不是 ① 那个提交）；`cargo mutants --help` 与 `ast.parse` 驱动源码被重型闸误拒；`admission.py` 自证 environment-outside-stage-marker 一格旧红；`--unit-environment`、`--marker-check` 要前缀。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | 复核第二轮判决；J3、J4 | 复核第二轮判决每一行够不够得着、改法选得对不对，替这一批的做法辩护（包括升档登记表对着「交用户判」与 `exclude_re` 两个候选）；核前两轮与这一批报出的数；列未定项清单 |
| 云端攻方（Opus） | J1、J2，兼攻 J3 第二条 | 在冻结副本的拷贝上造该红不红、不该红而红、假抓到的输入；样本照「cd 进样本根、根参数写同一个目录」起，钩子与驱动只跑登记为 light 的自证；驱动真跑、写错版驱动的自证与门禁整道是重型，不跑，写成要主 agent 跑的命令 |

## 五、跑前写死的判据

- 一个输入让门禁或驱动该红不红、不该红而红 ⇒ 打中，改门禁或驱动。
- 一个输入让驱动把该记没红的变异判成升档抓到、或门禁认下一行驱动不会写的登记行 ⇒ 打中；改法要连同「证据够不够」的补法一起给，交用户时标零轮。
- 一个判红分支只有与被判代码共用同一段函数或常量的样本盯着、那一段写错时样本不翻面 ⇒ 打中，补一份不共用的样本或开关。
- 五份文件里一句话与今天的做法不符 ⇒ 打中，改那一句。
- 前两轮判决的一行结论今天不成立、或一条未定项无声消失 ⇒ 打中，判决里写去向。
- 一条腿报「没打中」只抽了一次样；这一轮新提的形态算零轮，不再开轮。

交付：
- 腿的报告写 `research/prompts/m3-broad-gate-gov-r3-<模型>-output.md`，分段落盘，每次写进文件不超过 150 行，第一段排他新建，回复只写短句；
- 攻方的副本与用例放 `research/prompts/m3-broad-gate-gov-r3-opus-model/`；草稿放 `/tmp/claude-1000/<腿名>/`；
- 引规则与定义写文件自己的行号（去冻结副本里现查）；方案按 `crates/` 与脚本今天的实现来谈。
