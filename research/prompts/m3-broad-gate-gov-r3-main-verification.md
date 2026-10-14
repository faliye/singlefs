# 广谱变异接进提交门禁的治理改动：第三轮（终轮）判决（2026-10-03）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 H1 H2 H3 H4 H5 J1 J2 J3 J4 K1 F1 F2 F12 -->

## 一、这一轮的材料与证据

| 项 | 路径 | sha256 前 8 位 |
|---|---|---|
| 正文 | `research/prompts/_m3-broad-gate-gov-r3-body.md` | — |
| 辩方腿（Sonnet） | `research/prompts/m3-broad-gate-gov-r3-sonnet-output.md`（153 行），模型 `m3-broad-gate-gov-r3-sonnet-model/` | 4ab4e5af |
| 攻方腿（Opus） | `research/prompts/m3-broad-gate-gov-r3-opus-output.md`（425 行），模型 `m3-broad-gate-gov-r3-opus-model/` | 4a37f6e1 |
| 被判的实现报告 | `/tmp/claude-1000/m3-r1/batch6-broad-gate-r3fix/report.md` | fdd314e0 |

- 快照：冻结副本 `/tmp/claude-1000/m3-gov-r3-freeze/tree/` 对 `research/prompts/m3-broad-gate-gov-r3-snapshot/tree-sha256.txt` 跑 `sha256sum -c`，退 0。
- `cite-check.py` 两份报告一起跑（`--root` 冻结副本与主仓，带背景材料）：「核了 101 处引文，对上 101 处，没判 0 处」。
- 主 agent 在冻结副本里现查：`scripts/mutation/broad-mutation.py:386`「BASELINE_COLUMNS = ("step", "outcome", "seconds", "finished_at")」（没有线程数一列）；`:1836` 峰值按线程数分文件；`:1853` 超时基线取这一步绿记录；`:1940` 撞顶升档上限式；`:96`「撞顶只杀那一步」；`.claude/gate.d/checker-tier-broad-mutation.sh:558`「elif baseline_text != row["baseline"]:」、`:606` 版本对上的每一行都复核；`scripts/hosts/run-with-memory-cap.sh:574`「need = min(recorded[0], cap_bytes)」；`scripts/gate/admission.py:2311` 判不了那一支。主仓 `~/.local/share/singlefs/memory-peaks.tsv` 里「broad-mutation two-test-slow-harness」一行记 25769803776 字节、上限 24G，真库那一版 baseline.tsv 那一步「ok 936.0」。`journalctl --user` 里 scope 的 timeout / oom-kill 结局行 3 486 行。第一轮判决第 35 行（第二节第 11 行）写的去向在 `records/2026-09-16-subagent拆分提案.md` 里没有（`G1-e` 0 命中，`staged-green` 1 命中是别的事）。
- 被判的文件（按路径）：`.claude/gate.d/checker-tier-broad-mutation.sh` 与 `.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/` 下的样本、`scripts/mutation/broad-mutation.py`、`.claude/skills/crash-test/commit-stages.md`、`.claude/rules/verification.md`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`；`scripts/gate/admission.py` 没改、被驱动调。
- 复跑：第五节。

## 二、逐条判决

| # | 格 | 腿 | 判 |
|---|---|---|---|
| 1 | J2-a 基线漂移 | 攻方量：同一版换 `--threads`，基线峰值按线程数分文件、新线程数下没有，整段基线重跑、baseline.tsv 追加热编译的新绿行；门禁对这一版每一行登记都复核，驱动自己写的、证据齐的旧行之后永远「登记的基线 10.0 与那一版记的 1.2 不同」，照出路重判新行过、旧行照红；撞顶一边包装改写峰值同形 | **打中（复跑坐实）**：不该红而红，出路清不掉。驱动文件头第 115 行「同一个线程数下」与 baseline.tsv 不分线程数不符，随这一格改。改法（零轮）见第三节第 1 条 |
| 2 | J2-b 升档复核分支没有样本 | 攻方量：门禁升档复核 12 个判红分支逐个删，36 份样本一份不翻；10 份「驱动不会写」的现场原样红、删分支后绿 | **打中（复跑坐实）**：形同第一轮第 2 行。驱动一侧主 agent 跑了攻方 6 份写错版的 `--selftest`：只有 d2（没基线也算齐）「74 格里 4 格判错」，d3、d4、d5、d6 与攻方当对照的 d1（两趟结局不同不算不齐）都「74 格都对」——驱动自证的升档格连「两趟结局不同」那一支也没钉住（第五节） |
| 3 | J2-c 撞顶升档放不下 | 攻方量：包装记的基线峰值含页缓存、顶到上限，升档上限 = 2 倍原上限；升档那趟按峰值表排队、实际涨过总上限退 254（不是实现员推的 252）；真表 two-test-slow-harness 24G 下基线记成 24G，升档要 48G，大于本机 slice 总上限约 40 GiB；三处出路句「加大 --memory-cap」对无界分配方向反 | **打中（复跑坐实 254 机理；本机 40 GiB 上的数是推的）**：用户定案的出路对确定性无界分配那一类仍走不出去。交用户（第四节第 1 条） |
| 4 | J2-c 旁支 | 攻方量：无界分配那一条撞 slice 总上限时，内核杀的是 slice 里别人那一条 | 打中（不是这一批引入的）：驱动文件头第 96 行「撞顶只杀那一步」改成实情；机理改法并进第四节第 1 条丙 |
| 5 | J2-d 伪造登记行 / 手写结果行 | 攻方：照格式造的日志与登记行门禁认（门禁自己的绿样本就这么写）。辩方量：往 results.tsv 手追加一行没有标签的 caught，3 000 趟里 347 趟门禁判绿 | 打中，**不分辨臂**：库目录同一个用户可写是各候选与改前做法共用的前提，不拿它判升档登记表。另立一笔账交用户（第四节第 2 条） |
| 6 | J1-c `build_environment_lines` 共用、没人盯 | 攻方量：admission.py 那一份逐类写错 7 种（CARGO_PROFILE_*、rust-toolchain 文件、指的程序只记值、不往上层走、RUSTC、CARGO_ENCODED_RUSTFLAGS、配置里指的程序），每种都让一类构建环境从全局键消失，36 份样本一份不翻；字面值样本只钉 4 类；admission 自证本来就红一格 | **打中，收窄（复跑坐实）**：样本一侧 7 种都不翻；驱动自证 7 种也都「74 格都对」；admission 自证 7 种都比原样多红（原样 1 格，a1 2、a3 3、a4 10、a5 5、a6 2、a7 2、a8 6），盯得住，但它本来就红一格，整轮门禁里红→红翻不了面，要比格数才看得出（第五节）。所以有效的只剩「先把 admission 自证那一格旧红修绿」；改法第三节第 3 条 |
| 7 | J1-a 第 19 行退 1 两支 | 攻方量：PATH 去掉 cargo，`--marker-check` 退 1，打的是「cargo -V 与 rustc -V 跑不跑得出」；第 19 行把判不了那一支也叫去找会话协商 | 打中（一句）：第 19 行把判不了那一支单写，照门禁的 → 办 |
| 8 | J1-d RUSTUP_TOOLCHAIN | 攻方量：会话设 RUSTUP_TOOLCHAIN 时工具链从 1.98.0 变 1.100.0-nightly，`--unit-environment` 一个都不打，③ 退 7 后照第 28 行出路办不了 | 打中（条件式，频率推的）：`--unit-environment` 另比工具链变不变 |
| 9 | 第一轮第 11 行（提交没有读 staged-green 的闸）去向 | 辩方：判决写记进拆分提案欠账，那份文件里没有 | **打中（主 agent 现查坐实）**：无声消失。补进 `records/2026-09-16-subagent拆分提案.md` 欠账 |
| 10 | 第二轮攻方 H2-c（`global-key.txt` 与 tree.source 的 global_key 不是同一次写的那一支没样本） | 辩方：第二轮判决没列，今天仍没样本 | 打中：补一份红样本配开关 |
| 11 | 「下一批治理清单」没有载体 | 辩方：前两轮与这一批记「下一批」「欠账」「不采」的 20 行，散在判决、正文与调度记录，这一轮正文归档后只剩 git 里找得到 | 打中：拆分提案立一节「广谱门禁治理下一批」，第三节与本表第 13–16 行逐条进去 |
| 12 | 升档判的抓到不跨版本沿用 | 辩方：每次提交重烧「地板 900 秒加升档不小于 3600 秒」，每条约 75 分钟（推的，条数 k 没量） | 记账：首趟 ③ 之后数 ESCALATED 与 NO_CONCLUSION 定 k；k 小不改。零轮补法（升档抓到进结论表并带证据指针、门禁跨版本复核）进清单 |
| 13 | K=20 在默认参数下不咬 | 辩方算、攻方量：原限时 = max(900, 5 倍基线)，4 倍原限时恒不小于 20 倍基线；真库基线 1.5–43.5 秒，挡假抓到的是 900 / 3600 两个绝对数 | 不是打中（没有假抓到）。驱动文件头「K 为什么够」一段改成说清起作用的是哪一道 |
| 14 | J2-e 假抓到 | 攻方：K 那一道恒满足，热编基线、nice 与 cgroup 分 CPU、MemorySwapMax=0、1.5 倍窗口四样下造不出 | 没打中（一次抽样，不支撑「没问题」） |
| 15 | J3 再抽：第二轮 H1-7、H2-b | 两条腿各抽一次：H1-7 加升档结局攻方 5 400 趟、辩方 3 000 趟 0 次误判；H2-b 26 个开关翻面与文件头逐项相同 | 没打中，连同第二轮那一次是两次抽样（两条腿各一次，对齐到同一件事上一致）：记「没打中」 |
| 16 | 第二轮判决 11 行、第一轮 12 行复核 | 辩方：第二轮 11 行都站得住、没被这一批悄悄改掉；第一轮除第 11 行（本表第 9 行）外成立。另指「库目录现有各版下一次真跑退 7」不准：真库两个版本 tree.source 本来就没有 global_key，第一趟 ② 必须新建一版 | 认 |
| 17 | 数核 | 辩方在冻结副本上核：驱动自证 74 格、样本 36 份、新开关 18 个、旧开关 20 个、env-gap 7 种、admission 自证 283 格里 1 格红、真包装两句回显，都对得上 | 认；`stage-selftest.sh` 本身没跑（样本没暂存），归提交时 |

## 三、改法（定下的，都是零轮，进下一批）

1. 第 1 行：基线按线程数分开记（baseline.tsv 加线程数一列，峰值另存只追加的历史），登记行记它取的那一行；门禁按登记行找那一行复核。F2（复核不过只报不红）不采：放宽了门禁。驱动文件头第 115 行随之改。
2. 第 2、10 行：攻方那 10 份现场改成红样本、各配一个弄坏开关；H2-c 那一支补一份；驱动自证给 `escalation_evidence_problems` 的两趟结局不同、不到 K 倍、没比原来大、原来那趟缺包装那一句、升档那趟缺包装那一句五支各补一格（攻方 d1、d3–d6 当判红输入）。
3. 第 6 行：先把 admission 自证 environment-outside-stage-marker 那一格旧红修绿（修完它就盯得住那 7 种）；字面值样本的环境补上那 7 类，十六进制在驱动之外现算；驱动自证逐类换补同样几类。
4. 第 7、8 行：第 19 行判不了那一支单写；`--unit-environment` 另比工具链变不变。
5. 第 4、13、16 行措辞：驱动第 96 行、「K 为什么够」一段、实现报告那句「下一次真跑退 7」的出处改成实情。
6. 第 9、11、12 行：拆分提案立一节「广谱门禁治理下一批」，收第一轮第 11 行、第二轮第 2 行「不采」与第 8 行、两处重型闸误拒、`--unit-environment` / `--marker-check` 要前缀、selftest-registry 理由句、第二轮辩方两处小债、`exclude_re` 没走过一遍、k 没量，逐条带出处。

## 四、交用户

第三轮之后不再开轮；下面都是零轮。

| # | 问题 | 候选 |
|---|---|---|
| 1 | 确定性无界分配：升档上限放不进 slice，升档那趟 254、照旧没结论 | 甲：撞顶 K 的基线峰值改取回收不掉的那部分（anon 加 shmem）；乙：升档上限截到 slice 放得下的，截了不到 K 倍当场记「证据造不出来」；丙：升档那趟（与撞过顶的键）按自己的上限排队，放不下就等或 252；丁：出路句对撞顶写「降上限或等 slice 空出来」。四样可组合，攻方建议全做（推的，只在它的模型上量过 254 机理） |
| 2 | 库目录可写，手写结果行或造证据门禁都认（不分辨臂） | 甲：不防人，只记账，「足够的证据」防的是驱动自己判错；乙：登记行记 scope 名，门禁查 journal 的 `UNIT_RESULT`（只抬造假门槛，普通 caught 行照样能手写）；丙：结果行加带密钥的校验列、密钥放库目录之外（结果表换列、整库重建） |
| 3 | 第三节改法往下怎么走 | 甲：下一批照改，改到的定义与规则由用户逐份豁免；乙：只改脚本与样本，定义与规则里的句子改动单独交用户审 |

## 五、复跑

- 攻方 `rerun.sh` 在会话草稿目录 `gov3-verify/` 里拷模型目录、对冻结副本重跑，退 0：21 份输出 18 份逐字节相同（`samples-base.txt`、`j2a-baseline-probe.txt`、`j2a-hooks.txt`、`j2a-fix-prototypes.txt`、`j2b-sweep.txt`、`j2b-inputs.txt`、`j1c-env-gap.txt`、`j1c-sweep.txt`、`j1d-rustup-toolchain.txt`、`j1a-marker-check-no-cargo.txt`、`j2c-pagecache.txt`、`j3-combos-400.txt`、`j3-combos-5000.txt`、`j3-switches.txt` 等），`j2c-collateral.txt`、`j2c-over-slice.txt`、`j2c-wrapper-probe.txt` 三份只差临时目录路径，判定行相同（254 与「被 slice … 的总上限挤掉了」）。
- 攻方交主 agent 的 `main-agent-heavy.sh`：照它的单线程写法一趟原样 admission 自证就跑了约 50 分钟、被后台时限停在第一份（原样「283 格里 1 格判错」）；主 agent 改成脱离单元 `gov3-heavy-selftests`（会话草稿目录 `gov3-verify/heavy2.sh`，同一批拷贝，admission 九份并行、驱动逐份），20 分钟跑完，日志 `gov3-verify/heavy2.log`。驱动 `--selftest`：原样与 a1–a8、d1、d3–d6 都「74 格都对」，a9（对照，RUSTFLAGS 不进）「74 格里 3 格判错」，d2「74 格里 4 格判错」。admission `--selftest`：原样 1、a1 2、a2 1、a3 3、a4 10、a5 5、a6 2、a7 2、a8 6、a9 3 格判错（共 283 格）。a2 攻方自己的 env-gap 说它一类都没藏，与原样同是预期。

## 回看决策

不涉及决策：这一轮判的是门禁、广谱变异驱动与 skill、规则里的句子，不碰文件系统的任何决策分项。
