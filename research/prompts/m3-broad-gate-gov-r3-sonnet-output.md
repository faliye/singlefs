# 辩方腿报告：广谱变异接进提交门禁的治理改动，第三轮（终轮，Sonnet，复核第二轮判决；J3、J4）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 H1 H2 H3 H4 H5 J1 J2 J3 J4 K1 -->

日期 2026-10-03。冻结副本根 `/tmp/claude-1000/m3-gov-r3-freeze/tree/`（下文 F）。每一处引文（`文件:行号`）都在那份文件上 `grep -nF` 现查过，行号是那份文件自己的；
引判决与记录写那份文件自己的行号；「转述」标的不带引号。复跑材料在 `research/prompts/m3-broad-gate-gov-r3-sonnet-model/`（`README.md` 写了起法，`outputs/` 是这一轮的原样输出）。
被判的 11 个文件（驱动、门禁、admission.py、剪枝规则、commit-stages.md、verification.md、包装、detached-run.sh、登记表、两份钩子）在主仓与 F 里 `cmp` 逐字节相同（命令输出全是 same），
所以下面经「仓里那一份路径」跑的 light 自证，判的就是 F 里的文件。

## 一、总判

- 第二轮判决 11 行：每一行都够得着，选的改法对着替代方案站得住；没找到一行的结论被这一批改法悄悄改掉（逐行见第二节）。第 4 行已被用户 2026-10-03 定案取代，替这一批的做法辩护放在第三节。
- 找到三处**该进判决而没进、或进了没有落点**的（第五节）：
  1. 第一轮第 11 行（提交没有读 staged-green 的闸）说去向是 `records/2026-09-16-subagent拆分提案.md` 的欠账，那份文件里没有（`grep -c 'G1-e'` 工作区 0、HEAD 0；改写的同义句也找不到）：无声消失。
  2. 第二轮攻方 H2-c（`global-key.txt` 与 tree.source 的 global_key 不是同一次写的那一支没有样本）第二轮判决没列；今天那一支仍没有样本。
  3. 第二轮第 2 行「不采」的那一条、第 8 行「③ 核指针」都只在判决与这一轮正文里；「下一批治理清单」不是一份文件，只是散在几天调度记录里的几条，这两条哪里都没接着。
- 这一批新冒出的两个形态（零轮，写进判决交用户表）：① 升档判的抓到不进结论表、不跨版本沿用，每一版现判，同一条超时变异每次提交都要重烧「地板 + 升档限时」（推的，条数没量）；
  ② 升档登记表的证据口径只拦升档那一路：往 results.tsv 手追加一行没有标签的 caught，门禁照样判绿（我用真驱动 `summarise` 加门禁第二格抽出来的内嵌 python 量过，见第三节 D）。
- 数核：74 格、36 份样本、18 个新开关、env-gap 7 种都在 F 上核对得上（第四节）；前两轮判决里依赖旧写法的数（真库全局键 15/16 行相同、攻方 rerun 的几份输出）今天核不动，写了为什么。
- 没跑：驱动真跑、写错版驱动的 `--selftest`（闸拒仓副本）、门禁整道、`stage-selftest.sh`（10 份新样本未暂存，闸拒）。要主 agent 跑的命令在第六节。

## 二、第二轮判决逐行复核

「够得着」= 判决那一行的依据今天还成立、改法对着所有替代方案同样站得住。每行末尾写「推翻：」什么现象会翻它。

| # | 判决条目 | 辩方判定 | 证据 |
|---|---|---|---|
| 1 | H1-3a 查标记那一侧 | 站得住；改法选对（门禁加 `--marker-check`，不改 admission.py）。同样打中替代方案：改 `stage_fingerprint` 读索引会连带别的登记行，去掉这一道的标记复用则第 2b 步每次提交必跑 | 路径集合与指纹那一侧是同一个表达式：「paths += [admission.REGISTRATION_TABLE, f".claude/gate.d/{stage}"]」（`.claude/gate.d/checker-tier-broad-mutation.sh:649`）对「input_paths += [REGISTRATION_TABLE, f".claude/gate.d/{stage}"]」（`research/scripts/admission.py:2219`）。「admission.py stage-marker-check <根> <阶段文件名>」这一句同样是 commit-stages 第 19 行里另四道（qemu-device-streams、lkmm、crates-mutation-replay、formal-verification）开跑前的第一步，改 `stage_fingerprint` 的语义会让它们的旧标记一起对不上。（我先数过 stage-inputs.tsv 路径列直接列了 admission.py 的只有 checker-tier-layer0-replay 与这一道两行，所以不用「登记行里写了几次」当依据。）退出码对得上：「退 1 是没过（或判不了）」（`.claude/skills/crash-test/commit-stages.md:19`）对 「return 3」（`.claude/gate.d/checker-tier-broad-mutation.sh:684`）与「if ((check_exit == 1)); then」（`.claude/gate.d/checker-tier-broad-mutation.sh:682`）、「if marker_check "$ROOT"; then exit 0; else exit "$?"; fi」（`.claude/gate.d/checker-tier-broad-mutation.sh:706`）。样本三份（staged-differs-red、clean-reused、clean-unmarked）全对，开关 marker-check-unguarded 只翻 staged-differs-red（第四节命令输出）。保留：样本经「样本根放了 `.broad-sample/marker-check` 文件」触发（「[[ -f "$SAMPLE_DIRECTORY/marker-check" ]] && marker_check_requested=1」，`.claude/gate.d/checker-tier-broad-mutation.sh:191`），命令行那一支「--marker-check) marker_check_requested=1; shift ;;」（`.claude/gate.d/checker-tier-broad-mutation.sh:140`）没有样本；我删掉标记文件、命令行带 `--marker-check` 在样本根上起，退 0。推翻：真仓上（非样本）`--marker-check` 在工作区等于暂存区时退 1，或 `stage-marker-check` 退 0 而工作区与暂存区不同 |
| 2 | H1-3b 只做了一半 | 站得住；选 (i)「核登记行全部路径」而不选 (ii)「驱动与剪枝规则内容进全局键」是对的，但 (ii) 的「不采」留下无主欠账 | (i) 的路径集合含驱动、剪枝规则、排除表、包装脚本：`.claude/gate.d/stage-inputs.tsv:33`：「research/scripts/admission.py research/scripts/run-with-memory-cap.sh research/scripts/capped.sh」。(ii) 的代价判决自己写「不采：每改一次驱动整版重判，代价没量」（`research/prompts/m3-broad-gate-gov-r2-main-verification.md:23`）；判法改了而不改判法版本时旧结论照沿用，靠手动约定「判法变了就改它」（「JUDGE_RULES_VERSION = "prune-r3-e1-e6-fresh-build"」，`research/scripts/broad-mutation.py:390`）。这一批改了判结局的逻辑（升档），版本号没动，没出事是因为全局键多了一行 escalation、整个键换了写法（推的）。推翻：找到一个只改驱动判结局的代码、不动常量与全局键、`--marker-check` 与 ④ 都绿的提交。欠账见第五节第 3 条 |
| 3 | H2-a 样本与被判驱动共用函数 | 站得住，修补比判决要求的更远。判决要的是字面值样本加驱动自证逐类换输入；今天两边都到位，且字面值样本这一格单独就盯住 16 种驱动写错与 3 种 admission.py 写错 | 字面值样本不起 prepare.py：库目录那一版照字面值写（`.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/library-version-is-staged-tree-literal-global-key-green/setup.sh` 里全局键 16 行是十六进制字面）。我抽出门禁第一格的内嵌 python（`sed -n 202,389p`），对 F 的 `research/scripts` 拷贝在草稿目录里写错 16 种驱动、3 种 admission.py：基线退 0，19 种全退 1（`outputs/cell-mutants.txt`；驱动 16 种是攻方六种加 export-no-cargo、limits-constant、gkl-no-escalation、gkl-no-exclusions、gkl-no-rules、vk-no-global、vk-no-toolchain、fop-content-blind、doc-vars-dropped、build-env-own-copy；admission.py 三种是 `ADMISSION_BREAK=skip-build-environment`、丢 CARGO_HOME 配置段、丢环境变量段）。驱动自证这一边：仓里六个开关各自红 1、1、8、2、2、1 格（第四节）。保留（J1 归攻方）：字面值是人抄进 setup.sh 的，驱动合法改动（加一类行、admission.py 改标签）时这一份必红、要人手改，改的时候照驱动输出重抄就失去独立性（推的）。推翻：写错的驱动或 admission.py 在字面值样本上退 0 |
| 4 | H3 用户定案丙的出路 | 站得住（打中交用户），已被用户 2026-10-03 定案取代：「增加新的登记表，但是不能交给用户判，要能够有足够的证据自动判」（`records/2026-09-30-里程碑三开工第一轮.md:397`）。这一批的做法对着甲（用户判登记）、乙（`exclude_re`）、丙（红着提交）的辩护在第三节；两个新形态（每次提交重烧、手追加行绕证据）也在第三节 | 判决第 4 行两处小处都落了：自证补「加大限时再 `--item --rerun`」一格（驱动里 `selftest_rerun_larger_limit_cells` 在 `research/scripts/broad-mutation.py:4255`、`selftest_escalation_cells` 在 :4280，转述报告「五格」，计入 74 格）；commit-stages 写明「超时、内存撞顶不算定论，照样再选中重跑」（`.claude/skills/crash-test/commit-stages.md:26`）。推翻：见第三节各条 |
| 5 | ② ④ 与 ③ 环境不同 | 站得住。改法选对：驱动加 `--unit-environment`（脚本化），不让主 agent 凭记忆挑变量。替代方案「③ 起单元前人工核变量」同样会漏，且判法与全局键认的集合不是同一份 | 驱动里：「def unit_environment_names(environment, root):」（`research/scripts/broad-mutation.py:3056`）按「拿掉它、构建环境就变」判，不另写一张变量表；排除单元本来带的：「if name in UNIT_CARRIED_VARIABLES:」（`research/scripts/broad-mutation.py:3064`）。commit-stages 对得上：「会话 shell 里设的编译变量没照 `--unit-environment` 带进单元也是 7，带上重起 ③」（`.claude/skills/crash-test/commit-stages.md:28`）。我把第 26 行 ③ 原句（带 `env` 行与不带两种）与 `--unit-environment`、`--marker-check` 各喂重型闸（只判分类、不跑命令）：主 agent 带前缀都退 0，不带前缀后两条退 2，子 agent 带前缀退 2（`outputs/guard-feed.txt`）。推翻：③ 原句在闸里被拒，或会话里设了 RUSTFLAGS 而 `--unit-environment` 不打它 |
| 6 | 全局键环境部分比构建环境窄 | 站得住；改法选对（调 admission.py 那一份，不各写一份）。替代方案「在驱动里把变量表补宽」会与 admission.py 再漂一次 | 「lines = admission.build_environment_lines(root, environment)」（`research/scripts/broad-mutation.py:2553`），文档测试变量另加：「DOC_TEST_VARIABLES = ("RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS")」（`research/scripts/broad-mutation.py:413`）。我在隔离的 HOME 与目录上量 7 种（RUSTFLAGS、CARGO_PROFILE_DEV_OVERFLOW_CHECKS、CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS、RUSTC_WRAPPER、CARGO_BUILD_TARGET、RUSTDOCFLAGS、`$HOME/.cargo/config.toml` 写 overflow-checks）：全局键 7 种全变，删掉配置文件回到不设的那一个；对照 CARGO_BUILD_JOBS 与无关变量不变（`envgap.py` 原样输出：「变了 7 种（共 7 种）」）。推翻：任一种构建环境换了而全局键不变 |
| 7 | 第一轮第 10 行等价变异出路：`exclude_re` | 站得住，但这条出路没有任何样本或自证走过一遍：三处句子都写了，从没有人拿一条 `exclude_re` 真排除过一条变异（没有自证格，我也不能起 `cargo mutants`，闸拒）。选它而不新造排除表是对的：排除表要驱动与门禁各认一份，又得进全局键 | 三处句子对得上：「可写进 .cargo/mutants.toml 的 exclude_re」（`research/scripts/broad-mutation.py:135`）与门禁两处（`.claude/gate.d/checker-tier-broad-mutation.sh:132`、`:458`，都含「确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（整版重判）」）。「整版重判」成立：`.cargo/` 下的文件进全局键「path.startswith(".cargo/")」（`research/scripts/broad-mutation.py:2573`），也在导出范围里进源码指纹。文件位置对：仓里已有 `.cargo/mutants.toml`，「examine_globs = [」（`.cargo/mutants.toml:11`）；cargo-mutants 27.1.0 的 `src/config.rs` 第 60 行 `pub exclude_re: Vec<String>,` 与 `book/src/filter_mutants.md` 第 44 行（转述，在 `~/.cargo/registry/src/` 下，我再读了一遍）。推翻：在导出树的 `.cargo/mutants.toml` 加一条 `exclude_re` 后 `--sync-tree` 生成的清单里那一条还在 |
| 8 | H5 ② 与 ③ 之间换树，记欠账 | 站得住（只费时间，不出假绿）；欠账没有落点，见第五节第 4 条 | 判决原话：「记进下一批治理清单」（`research/prompts/m3-broad-gate-gov-r2-main-verification.md:29`）。commit-stages 第 28 行最后一句只核「git write-tree」，不核 ③ 起的是不是 ① 那个提交：「① 之后再取一次」（`.claude/skills/crash-test/commit-stages.md:28`）。推翻：③ 在指针被别人换走时自己退非 0 |
| 9 | 措辞债三处 | 站得住，三处都改了 | `-h`：「只有 --list-items、--summary、--check-ignored、--help（-h）不算）」（`.claude/hooks/lib_heavy_tests.py:1712`），与分类集合「BROAD_MUTATION_QUERY_ARGUMENTS = frozenset({"--list-items", "--summary", "--check-ignored", "--help", "-h"})」（`.claude/hooks/lib_heavy_tests.py:403`）逐项相同；`resync_command` 的前缀改调「def heavy_prefix_text():」（`research/scripts/broad-mutation.py:555`）；驱动头退出码补 8：「8 --summary 判提交门禁红」（`research/scripts/broad-mutation.py:170`）。推翻：驱动头与代码里退出码常量对不上 |
| 10 | 第一轮第 1–5、7–9 行复核 | 站得住；第 5 行「全局键进版本指纹」选对，但这一批把全局键全文换了写法（env 行没了，多了 build 行与 escalation 行） | 版本指纹含全局键（转述：驱动头「指纹」一段写这一版的指纹含「global <全局键的 sha256>」一行）；今天的字面值样本把 16 行全文写死，写法变了会红。真库现在的两个版本目录的 global-key.txt 只有 rules、toolchain、file、tool、exclusions、limits 六类行（命令输出：`awk` 计数），`tree.source` 里没有 global_key 键，所以它们本来就不被认（驱动头同一段（转述）写全局键进指纹之前建的、tree.source 里没有 global_key 的版本不再认），报告与正文说的「库目录现有各版下一次真跑退 7」不准：它们不是退 7，是更早就不认，第一趟 ② 必须新建一版。推翻：真库 `tree.source` 里有 global_key |
| 11 | 没打中的（H1-4、H1-5、H1-6、H1-7、H2-b、H4） | 一次抽样本来不支撑「没问题」。我对最脆弱的两格各再抽一次（第四节）：H1-7 与 H2-b；两格都没打中 | H1-7：3000 趟随机结果表，另掺升档标签的 caught 行（没有登记行），门禁判定与「每条有结论且没红」逐趟一致，mismatches=0；H2-b：26 个弄坏开关的翻面与文件头逐项相同。H1-5 的「真库 16 行里 15 行相同」今天核不动（见第四节） |

### 第一轮 12 行，今天还成立吗

| 第一轮行 | 今天 | 落点 |
|---|---|---|
| 1 样本指纹按今天驱动算 | 成立；14 份 setup.sh 改调 `build_environment_of`（共用函数那一形，由字面值样本钉住） | 14 份 setup.sh 与 26 份旧样本名全在（`diff -rq` 数出 14 份 setup.sh 不同、旧名 0 个少、新名 10 个） |
| 2 8 个没人盯的判红分支 | 成立；26 个开关逐个翻面与文件头相同 | 第四节开关表 |
| 3 写标记前核 | 成立，且写标记与查标记两支用同一个 `marker_inputs_clean`（第二个参数 write 或 check） | 门禁 :691 `write_marker_if_clean` 与 :669 `marker_check` |
| 4 钉住 tree.source | 成立 | 门禁 :730 一段「钉住版本指针」，开关 tree-source-reread 翻 swapped-between-cells-red |
| 5 全局键进指纹 | 成立（写法变了，见上第 10 行） | 复用登记行补了 run-with-memory-cap.sh、capped.sh 与 environment=（stage-inputs.tsv:33） |
| 6 超时撞顶 | 部分被取代（升档证据齐的判抓到）；证据不齐的仍算没结论，门禁头写了两个日期：「超时、内存撞顶照驱动算没结论（用户 2026-10-02 定）」（`.claude/gate.d/checker-tier-broad-mutation.sh:33`） | 不是悄悄改：门禁头、驱动头、commit-stages 三处都写了 |
| 7 两行 total 互核 | 成立；3000 趟 0 次对不上 | 门禁 :483–493 |
| 8 重型闸两处误拒 | 成立（`--check-ignored` 在放行集合里）；新增 `--unit-environment`、`--marker-check` 照广谱变异算、要前缀（本批报告自报，闸行为我核了） | `outputs/guard-feed.txt` |
| 9 逐句改 | 成立；退出码 8、`-h` 都补了 | 本节第 9 行 |
| 10 等价变异出路 | 被第二轮第 7 行修正，今天句子对得上 | 本节第 7 行 |
| 11 提交没读 staged-green | **去向落空**：拆分提案里没有 | 第五节第 1 条 |
| 12 第二段基线真绿 | 落在 commit-stages 第 28 行「退 3、6、7 照抄」，6 是基线不绿（驱动头 :168） | 第 2b 步真跑才验 |

## 三、替这一批的做法辩护

对着替代方案逐个比，「同样打中」的写在 D。

- **A　升档登记表对「甲：用户判登记」**。用户自己把甲排除了（`records/2026-09-30-里程碑三开工第一轮.md:397` 那句原话）；除此之外甲在门禁这一侧还有一个没解的问题：门禁凭什么认一行是用户判的而不是手写的。第二轮辩方指出过同一个洞（读代码推的，没跑）：「往这一版的 `results.tsv` 手追加一行 caught 或 unviable」（`research/prompts/m3-broad-gate-gov-r2-sonnet-output.md:80`）驱动与门禁都收。升档登记行至少给门禁留了可重算的东西：基线取那一版自己的 baseline.tsv 或峰值表、倍数取此刻驱动里的常量（「multiples = {timeout_label: driver.ESCALATION_TIME_BASELINE_MULTIPLE」，`.claude/gate.d/checker-tier-broad-mutation.sh:513`）、两份日志在 `versions/<那一版>/escalation/` 下且 sha256 对（「没有驱动写的证据（手写的行）」，`.claude/gate.d/checker-tier-broad-mutation.sh:578`）、按版本键只认这一版的行（「if row["version"] != version and "escalated-version-unchecked" not in broken:」，`.claude/gate.d/checker-tier-broad-mutation.sh:604`）；甲的一行只有一句理由。
- **B　对「乙：`.cargo/mutants.toml` 的 `exclude_re`」**。四点：① 语义不对口：用户定案「卡住算抓到」，一条会卡死的变异按定案是抓到了，`exclude_re` 把它从清单里拿走，等于从分母里删掉一条抓到的变异；② 每一条要人挑（正是用户说不要交给人的）；③ 每排除一条都整版重判：`.cargo/` 下的文件进全局键（`research/scripts/broad-mutation.py:2573`），第一轮判决转述一趟「10.6–82.0 小时」（`research/prompts/m3-broad-gate-gov-r1-main-verification.md:33`）；④ 对负载相关的、不稳定的超时不起作用。`exclude_re` 留给真正的等价变异（没有哪条测试杀得了的，`research/scripts/broad-mutation.py:135`），门禁的出路句也是这么分的（`.claude/gate.d/checker-tier-broad-mutation.sh:458`）。
- **C　对「丙：红着提交」**。丙让每次提交都停在同一处，且出路只剩「交用户定」而门禁没有认这个决定的入口（第二轮判决第 4 行「确定性的两种走不出去」，`research/prompts/m3-broad-gate-gov-r2-main-verification.md:25`）；用户已不选。
- **D　同样打中所有臂的两件**。
  1. 假抓到（一条不加限时、不限内存时会跑完且测试全过的变异被判成抓到）：甲的人、乙的人、升档的驱动都会遇到；升档这一臂门槛反而最严。测试步本来就按「超 3 倍基线 CPU 加 30 秒算抓到」判（用户定），驱动头自己写「比它松得多」（`research/scripts/broad-mutation.py:117`）；升档只管有整步限时的步（编译步、文档测试步；测试步经 runner、没有整步限时，不升档，「没有整步限时的步（经 runner 按单条限 CPU 的测试步，time_limit_of 交 None）不升档」，`research/scripts/broad-mutation.py:103`）。算术：升档限时 = max(4 倍原限时, 20 倍基线)，原限时 = max(900, 5 倍基线)，所以默认参数下 4 倍原限时已不小于 20 倍基线，K=20 只在有人把 `--timeout-multiplier`（「--timeout-multiplier <倍> 默认 5」，`research/scripts/broad-mutation.py:33`）调到 5 以下时才咬住（常量见 `research/scripts/broad-mutation.py:397` 与 :398）。撞顶一边：默认上限 24G 乘 1.5 是 36G，在本机 slice 总上限约 40 GiB 之内（「默认上限 24G 乘 1.5 放得下」，`research/scripts/broad-mutation.py:399`；约 40 GiB 是驱动注释与实现员报告的数，我没量）。
  2. 手写行 / 伪造：**证据口径只拦升档那一路**。门禁复核的是带 caught-by-timeout / caught-by-memory 标签的行；往 results.tsv 手追加一行没有标签的 caught，驱动 `--summary` 与门禁第二格都当有结论。我量的：把门禁第二格的内嵌 python（`sed -n 399,624p`）抽出来，喂真驱动 `summarise` 在 3000 趟随机结果表上打的两行：手造的 caught 行（无标签、没有登记行）有 347 趟判绿，手造的带标签行（没有登记行）85 趟全判红（`outputs/combos3000.txt`：「summary_exit=0 gate_exit=0 truth_green=True gate_says_inconsistent=False count=347」「summary_exit=0 gate_exit=1 truth_green=False gate_says_inconsistent=False count=85」）。所以最小的伪造不用造日志与 sha256，直接手追加普通 caught 行；升档登记表的「足够的证据」防的是驱动自己判错与误操作（负载抖动、基线偏小），不防人，这一点今天各臂相同（甲、乙都一样）。要变成防篡改，最小补法是驱动给每行结果行加一列带密钥的 HMAC（密钥在库目录之外）、门禁核，代价是结果表换列、驱动门禁自证各加格、现有库目录整库重建；这一批不建议做，列进第六节「零轮」交用户（推的：同一用户本机写权限的威胁模型项目文件里没写，我没找到条款）。
- **E　`--marker-check` 对「改 admission.py」**：见第二节第 1 行；改 admission.py 要让别的四道阶段的旧标记一起对不上，且 admission.py 自证今天 283 格里已有 1 格红（第四节），在它上面再改不稳。
- **F　升档放不下与包装自己的结局之后，出路还走不走得通**：包装自己的 251、252、254：「升档那趟是内存包装自己的结局（251、252、254）不停整趟，照原结局记、备注写没跑成」（`research/scripts/broad-mutation.py:106`），结果行仍是 timeout / memory-cap，它们在 `--changed-since` 里照旧被选中（「要现判的条里这一版已经是超时、内存撞顶的」，`research/scripts/broad-mutation.py:132`），所以出路（再起 ③ 或加大 `--timeout-floor` 点名重跑）走得通。真走不通的只有一支：`--memory-cap` 大到 1.5 倍放不进 slice 时升档那趟永远 252，那一条永远没结论（推的，约 26G 是报告的数）；默认 24G 不碰它。
- **G　新形态（零轮）：升档判的抓到每次提交重烧**。驱动头：「升档判的抓到不进结论表（conclusions.tsv）、不跨版本沿用，每一版现判」（`research/scripts/broad-mutation.py:125`），代码里：「if runner.pruner and outcome in DECISIVE_OUTCOMES and test_name not in ESCALATED_LABELS:」（`research/scripts/broad-mutation.py:2134`）。所以下一版 `--changed-since` 对这些条找不到沿用的结论，要现判：每条每次提交先跑到原限时（地板 900 秒）、再跑升档限时（不小于 3600 秒），按默认参数每条约 75 分钟（推的）。条数 k 没量：第二轮判决写「频率没量（真库新驱动下结果行只 2 行）」（`research/prompts/m3-broad-gate-gov-r2-main-verification.md:25`），旧库的 7 条超时全是测试步（转述第二轮辩方报告），新驱动下测试步不升档。辩护：不跨版本沿用是保守的，证据绑版本目录、门禁只认这一版的行；k 小时代价可忽略。补法（不在这一批）：升档判的抓到写进结论表并带证据指针，门禁跨版本复核；先量 k。

## 四、J3：前两轮与这一批报出的数，能不能在 F 上核

下面「原样」都是这一轮自己跑出来的，输出在 `research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/`。命令都经 `nice -n 19`、`capped.sh 16`（线程上限 16）；起命令前 `ps` 没有性能测量（qemu-system、vm-bench、e152、fio）在跑，只有别的会话的 `cargo` 在跑（加了 nice）。

| 报出的数 | 出处 | 我核的 | 判 |
|---|---|---|---|
| 驱动 `--selftest`「74 格都对」 | 这一批报告「自证与样本」；正文第二节 | `nice -n 19 bash research/scripts/capped.sh 16 python3 research/scripts/broad-mutation.py --selftest` 退 0，用时约 38 秒：`research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/driver-selftest.log:1`：「74 格都对」 | 对 |
| 门禁样本 36 份全对 | 同上 | 36 个目录（`ls` 数）；我照 `stage-selftest.sh` 的起法（拷临时目录、setup.sh、cd 进样本根、根参数写同一个目录、带 --force）逐份起：36 个 ok、0 个 FLIP（`outputs/samples-36.txt`，`wc -l` 36）。`stage-selftest.sh` 本身被闸拒（10 份新样本未暂存），我这一支是它的复刻，不是它 | 对（起法是复刻） |
| 旧 26 份样本全在、14 份 setup.sh 改过 | 报告「改过的文件」 | 对 F 与第二轮冻结副本的样本目录 `diff -rq`：旧名 0 个少、新名 10 个、`setup.sh` 不同 14 份；第 15 条是目录名里带 differs 的误配 | 对 |
| 新开关 18 个（驱动 12、门禁 6） | 正文第二节 | 驱动 12 个各开一次 `BROAD_MUTATION_BREAK=<开关> … --selftest`，各退 1，格数与报告表逐个相同（`outputs/driver-switches.txt`）：global-key-no-toolchain-line 1、-no-file-lines 1、-no-build-lines 8、-no-tool-lines 2、-no-limits-line 2、fingerprint-path-blind 1、timeout-floor-in-global-key 6、escalation-skipped 5、escalation-same-limit 2、escalation-evidence-unchecked 6、escalated-not-summarised 1、unit-environment-fixed-list 1。门禁 6 个对 36 份样本逐个开：只翻点名那几份（`outputs/gate-new-switches.txt`；例：marker-check-unguarded 只翻 staged-differs-red，escalated-version-unchecked 翻 escalated-green 与 old-version-red） | 对 |
| 门禁旧开关 20 个（第二轮「20 个开关」） | 第二轮判决第 11 行 | 门禁文件头开关表今天 26 行（`sed -n 82,111p` 数出），去掉新 6 个是 20；这 20 个我对 36 份样本逐个开，翻面与文件头逐项相同（`outputs/gate-old-switches.txt`、`outputs/gate-old-switches2.txt`） | 对 |
| env-gap 7 种全变 | 正文第二节 | 见第二节第 6 行：`research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/envgap.txt:13`：「变了 7 种（共 7 种）」，另两个对照不变 | 对（用我自己的值，哈希与报告不同是预期） |
| `admission.py --selftest` 改前改后同为 283 格里 1 格判错 | 报告第 5 条 | 今天跑：`research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/admission-selftest.log:284`：「✗ admission.py 自证没过：1 格判错（共 283 格）」，红的是 `outputs/admission-selftest.log:50` 那一格（environment-outside-stage-marker 弄坏开关）。「改前」的没法核（没有改前的文件可跑） | 今天的对；改前核不动 |
| 包装两句「超过限时 N 秒」「撞了内存上限 X」没经真包装跑 | 报告「没做什么」 | 包装自己的 `--selftest` 是 light：`research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/wrapper-selftest.log:1`：「查了 53 项」，退 0。它钉的就是这两句的写法：「grep -q '撞了内存上限 64M'」（`research/scripts/run-with-memory-cap.sh:1021`）、「'超过限时 1 秒'」（`research/scripts/run-with-memory-cap.sh:1278`）；打的地方：「撞了内存上限 $cap（scope $unit 的 Result=oom-kill」（`research/scripts/run-with-memory-cap.sh:946`）、「超过限时 ${TIME_LIMIT_SECONDS} 秒（scope $unit 的 Result=timeout」（`research/scripts/run-with-memory-cap.sh:951`）。驱动把包装的 stderr 并进步骤日志：「stderr=subprocess.STDOUT」（`research/scripts/broad-mutation.py:1895`），驱动核的句子「超过限时 {limit_text} 秒」「撞了内存上限 {limit_text}」按同一个文本拼（`research/scripts/broad-mutation.py:1674`）；上限写法不被包装改写（`auto` 才改写，驱动不写 auto：「cap="$(cap_judged_against_the_recorded_peak "$cap" "$@")" || exit $?」，`research/scripts/run-with-memory-cap.sh:1613`）。包装一侧的写法有自证钉住，驱动与包装的握手读代码一致。我另用真包装按驱动给的两种写法各起一次（`outputs/wrapper-sentences.txt`）：限时 2 秒的 `sleep 6` 打「超过限时 2 秒（scope」（`research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/wrapper-sentences.txt:2`）、退「exit=253」（`research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/wrapper-sentences.txt:3`）；上限 1536M 里分配 2 GiB 打「撞了内存上限 1536M（scope」（`research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/wrapper-sentences.txt:5`）、退「exit=250」（`research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/wrapper-sentences.txt:6`），限时与上限的写法都原样回显，与驱动拼的句子（超时写整数秒、撞顶写 <MiB>M）一致。没有经真包装把真超时、真撞顶跑成一条升档登记行 | 包装一侧与握手一侧核了；端到端（驱动起真步骤、真落登记行）核不动，要主 agent 首趟 ③ 起来后看第一条升档登记行（第六节命令 3） |
| 第二轮 §5 的数：`samples-base` 判对 26 个、20 个开关翻面、`h1-7-combos` 5000 趟 mismatches=0 | 第二轮判决第五节 | 26 份旧样本是今天 36 份的子集，同判；20 个开关同上；`combos` 我用同一个思路在今天的代码上重做（门禁第二格的内嵌 python 今天在 399–624 行，`h1-7` 脚本里 `awk 'NR>=344 && NR<=443'` 抽出来的是别的段，所以重写了抽取与参数），并在结果表里掺升档标签的行：3000 趟 `research/prompts/m3-broad-gate-gov-r3-sonnet-model/outputs/combos3000.txt:5`：「mismatches=0」 | 子集对；5000 趟那一个数核不动（脚本对今天的代码失效），重做的 3000 趟没打中 |
| 第二轮攻方 `h1-5-real-key`「真库 16 行里 15 行逐字节相同」 | 第二轮判决第 11 行 | 核不动：那份脚本给 `global_key_lines` 的第四个参数是环境字典（`research/prompts/m3-broad-gate-gov-r2-opus-model/h1-5-real-key.py:15` 传 `os.environ`），今天它收的是 `[(名字, 内容)]`；真库两个版本目录的 global-key.txt 都只有 rules、toolchain、file、tool、exclusions、limits 六类行，`tree.source` 没有 global_key，今天的写法（多 build、escalation 行）下必然不同 | 核不动（输入已换） |
| 第二轮「8 种写错版 55 格都对」里的 limits-constant、export-no-cargo | 第二轮判决第五节 | 这两种我在字面值样本的库目录格上量（草稿里写错版驱动，`outputs/cell-mutants.txt`）：都退 1，且分别点出「全局键与那一版记的不同」「导出范围不同」；它们的驱动自证 `--selftest` 没跑（闸拒写错版驱动的拷贝） | 样本一边对；驱动自证一边核不动 |

## 五、J4：没收的线，逐条标今天落在哪

来源三份：第一轮判决（`research/prompts/m3-broad-gate-gov-r1-main-verification.md`）、第二轮判决与两条腿报告、这一批报告「没做什么」；落点查了调度记录、`records/2026-09-16-subagent拆分提案.md`、`.claude/kb/checks-owed.md`（只有 C489 一行提到广谱变异，是 2026-09 的账，与治理无关）、驱动与门禁文件头。「落在」= 一份常驻文件里找得到；只在 `/tmp` 报告或本轮正文里的，算没有落点（正文这一轮之后会被下一次提交归档）。

| # | 未定项 | 出处 | 今天落在哪 | 判 |
|---|---|---|---|---|
| 1 | 提交没有读 `refs/sop/staged-green` 的闸（第一轮第 11 行） | 判决写「记进 `records/2026-09-16-subagent拆分提案.md` 的欠账」（`research/prompts/m3-broad-gate-gov-r1-main-verification.md:35`）；调度记录写「记进 subagent 拆分提案的欠账」（`records/2026-09-30-里程碑三开工第一轮.md:343`） | 拆分提案里没有：`grep -c 'G1-e'` 工作区 0、`git show HEAD:` 那份 0，`pre-commit`、`时间窗`、`跑完才能提交`、`提交没有读` 都无命中；第四十三至四十七节的欠账表是别的事 | **无声消失（打中）**：补一节欠账，或进 `.claude/kb/checks-owed.md` |
| 2 | 第二段基线真绿，真跑时看基线（第一轮第 12 行） | 判决第 36 行 | 落在 commit-stages 第 28 行：「退 3、6、7 照抄」（6 是基线不绿，驱动头 :168） | 落在；真跑才验 |
| 3 | 驱动与剪枝规则内容进全局键，「不采」 | 「不采：每改一次驱动整版重判，代价没量」（`research/prompts/m3-broad-gate-gov-r2-main-verification.md:23`） | 只在判决里；靠手动约定改判法版本号（`research/scripts/broad-mutation.py:390`） | **无主**：写进欠账，附「代价没量」，或量「驱动每改一次的频率」 |
| 4 | ③ 核指针是不是 ① 那个提交（第二轮第 8 行） | 「记进下一批治理清单」（`research/prompts/m3-broad-gate-gov-r2-main-verification.md:29`）；正文「已知要进下一批的：第二轮第 8 行」（`research/prompts/_m3-broad-gate-gov-r3-body.md:51`） | `grep -rn '核指针\|指针是不是'` 在 `records`、`.claude/kb`、`.claude/skills`、`.claude/rules`、`.claude/main-agent.md` 里 0 命中。「下一批治理清单」不是文件：只有「下一批治理清单：heavy-test 闸不能只点名几道轻的样本档」（`records/2026-10-01-门禁注释精简.md:41`）与「误拒这一形进下一批治理清单」（`records/2026-09-30-里程碑三开工第一轮.md:393`）、:403 一句，散在两份记录里 | **没有落点**：本轮正文归档后只剩 git 里找得到。要么立一份常驻的「下一批治理清单」（拆分提案新一节），要么逐条进欠账 |
| 5 | 重型闸误拒 `cargo mutants --help`、`ast.parse` 驱动源码 | 调度记录 :393、:403 | 落在调度记录两条；两条都写「进下一批治理清单」，同上没有清单 | 落在（散） |
| 6 | `admission.py` 自证 environment-outside-stage-marker 一格旧红 | 「admission 自证 environment-outside-stage-marker 一格旧红」（`records/2026-09-30-里程碑三开工第一轮.md:404`） | 拆分提案第四十五节欠账第 4 行：「交下一批工具改动查」（`records/2026-09-16-subagent拆分提案.md:1036`）；今天再跑同一格红（第四节） | 落在；注意它红的那一格正是标记指纹含不含环境，`--marker-check` 靠同一个函数，`admission.py:2222` 实码含环境（「if not break_is_set("environment-outside-stage-marker"):」，`research/scripts/admission.py:2222`），更像自证写法而不是代码缺口（推的） |
| 7 | `--unit-environment`、`--marker-check` 要前缀 | 这一批报告「没做什么」（转述）；正文 :51 | 记录与欠账里没有；闸行为我核了（`outputs/guard-feed.txt:3` 不带前缀退 2） | 没有常驻落点（低）：提交流程本来带前缀，不碍 |
| 8 | `selftest-registry.tsv` 这一道的理由句没补查标记三份与升档六份 | 报告「没做什么」（转述） | 登记行 :107 理由「setup.sh 只起 git 与 python3」，升档样本的 setup.sh 另起 `sha256sum`（`grep -c sha256sum` 得 2）：「samples-light-marker:.broad-mutation-sample」（`.claude/hooks/selftest-registry.tsv:107`） | 没有落点（低）；不影响判定（仍轻） |
| 9 | 第二轮 H2-c：`global-key.txt` 与 tree.source 的 global_key 不是同一次写的那一支没样本 | 「低价值，记一笔」（`research/prompts/m3-broad-gate-gov-r2-opus-output.md:53`） | 第二轮判决没列这一格；今天那一支仍在（「problems.append(f"{global_path} 与 tree.source 记的 global_key 不是同一次写的")」，`.claude/gate.d/checker-tier-broad-mutation.sh:350`），26 个旧样本加 10 个新样本的 `expect` 里 `grep -l 'global_key 不是同一次'` 0 个 | **无声消失（打中，判据 5）**：补一份样本，或写明今天的驱动写不出这种现场 |
| 10 | 第二轮辩方 H4-3 位置债（`main-agent.md` 第 12 条放在「一轮怎么开、怎么收」下） | 「位置债：第 12 条放在」（`research/prompts/m3-broad-gate-gov-r2-sonnet-output.md:93`） | 判决没列，今天没改 | 没有落点（低，辩方自己说不是错） |
| 11 | 第二轮辩方 H3-2 小摩擦：NO_CONCLUSION 行不带上一趟的限时秒数 | 「NO_CONCLUSION 行只打序号、状态、变异名」（`research/prompts/m3-broad-gate-gov-r2-sonnet-output.md:63`） | 判决没列；今天仍是三列（`research/scripts/broad-mutation.py:2366`），升档不齐的备注写了缺哪一样，限时在结果行备注里 | 没有落点（低） |
| 12 | 频率没量：确定性超时（文档测试卡住）与无界分配撞顶有几条 | 「频率没量（真库新驱动下结果行只 2 行）」（`research/prompts/m3-broad-gate-gov-r2-main-verification.md:25`） | 没有落点；今天真库没有 `escalated-verdicts.tsv`、没有 `baseline-peaks.*.tsv`（`ls` 无），首趟 ③ 才产生 | 开着：量法是首趟 ③ 之后数 `--summary` 的 NO_CONCLUSION 与 ESCALATED（第六节命令 2） |
| 13 | 升档判的抓到每次提交重烧（第三节 G） | 新形态，零轮 | 驱动头写了「每一版现判」，成本没写 | 记进零轮表 |
| 14 | `--memory-cap` 大于约 26G 升档放不下 | 报告「没做什么」（转述，推的） | 驱动注释：「默认上限 24G 乘 1.5 放得下」（`research/scripts/broad-mutation.py:399`） | 落在驱动注释，数没量 |
| 15 | 基线峰值写不进 baseline-peaks 时每次重起重跑基线（没造现场） | 报告「没做什么」（转述） | 驱动头：「基线那几步另把 RUN_WITH_MEMORY_CAP_PEAKS 指到这一版的 baseline-peaks.<线程数>.tsv」（`research/scripts/broad-mutation.py:98`）；真库现在一个这样的文件都没有，首趟必重跑一遍基线（推的） | 落在驱动头；一次性成本 |
| 16 | 包装两句端到端没经真驱动跑 | 报告「没做什么」 | 第四节：包装一侧与握手核了，驱动起真步骤那一段核不动 | 开着（第六节命令 3） |
| 17 | 全局键写法变了，「库目录现有各版下一次真跑退 7」 | 报告与正文（转述） | commit-stages 第 28 行有出路：「会话 shell 里设的编译变量没照 `--unit-environment` 带进单元也是 7，带上重起 ③」（`.claude/skills/crash-test/commit-stages.md:28`）；但真库两个版本本来就没有 global_key，不是退 7 而是更早就不认（第二节第 10 行） | 落在；句子要改成「第一趟 ② 新建一版」 |
| 18 | 第一轮第 6 行、第二轮第 4 行交用户的两条 | 调度记录 :397 | 用户 2026-10-03 已定（第 1 条取升档登记表、第 2 条取开终轮） | 已定 |
| 19 | `exclude_re` 出路没有样本也没有自证 | 本报告第二节第 7 行 | 没有落点 | 新欠账（低）：只要有人第一次真排除一条，就在那一趟看清单少一条 |
| 20 | 手追加普通 caught 行绕过升档证据（第三节 D） | 新形态，零轮 | 第二轮辩方指出过同一个洞（读代码推的，`research/prompts/m3-broad-gate-gov-r2-sonnet-output.md:80`），没有落点 | 记进零轮表 |

## 六、交主 agent

### 要主 agent 跑的命令（我不跑：要前缀、要暂存区或起真驱动）

1. 真仓上查标记那一支（样本只走「样本根放了标记文件」那条触发，真仓上没有样本）。仓根，前缀 `SINGLEFS_HEAVY_TESTS=commit`，不设线程与内存（只读）：
   `cd /home/fy5090/code/singlefs && SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/checker-tier-broad-mutation.sh --marker-check; echo exit=$?`
   判：登记路径上工作区等于暂存区时退 0（打 ok 行与「记复用」）或退 3；不等时退 1 并打「✗ 不查全绿标记」。三种之外都算这一支没落对。
2. 首趟 ③ 跑完之后量频率（第五节第 12 行）。仓根，不要前缀：
   `python3 research/scripts/broad-mutation.py --summary > <草稿目录>/summary.txt; grep -c '^NO_CONCLUSION' <草稿目录>/summary.txt; grep -c '^ESCALATED' <草稿目录>/summary.txt; wc -l ~/.local/share/singlefs/broad-mutation/escalated-verdicts.tsv`
   判：NO_CONCLUSION 里 timeout、memory-cap 各几条（结果行备注写步名）、ESCALATED 几条；这个数就是第三节 G 的 k。
3. 首条升档登记行出现后立刻核门禁认不认驱动写的行（端到端，第四节没核到的那一段）。仓根，门禁只点名 gate-verdict 一格，闸放行：
   `bash .claude/gate.d/checker-tier-broad-mutation.sh --check gate-verdict`
   判：「✓ 广谱变异提交门禁判绿：…升档判的抓到 N 条、登记行逐行复核过」。红在「升档判的抓到复核不过」就是驱动写的证据与门禁复核的口径对不上（报告自己写的推翻条件）；先看那一行点名的是日志、基线还是倍数。
4. 暂存之后（10 份新样本入暂存区之后）：`bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item checker-tier-broad-mutation.sh`，判：36 个样本判得对（我的复刻是 36 对 0 错）。
5. （可选）第二轮攻方留的 `limits-constant`、`export-no-cargo` 两种写错版的驱动自证一边没人跑过：在实现员的 `driver-mutant-selftests.sh` 那一套里加这两种，判：每种「74 格里 M 格判错」M 不小于 1，或「74 格都对」时那两种只靠样本钉（样本一边我已量过都红）。

### 判决要采的与要交用户的

- **打中（判据 5）**：第五节第 1 行（G1-e 去向落空）、第 9 行（H2-c 无声消失，仍没样本）；第 3、4 行是未定项无落点（会随正文归档消失），判决里写去向。
- **新形态（零轮，写进判决交用户表，不再开轮，`.claude/rules/three-way-inference.md@多轮：一次打穿不算数，三轮里多数打穿才算`：「第三轮之后停，不开第四轮。」）**：
  1. 升档判的抓到每次提交重烧：先量 k（命令 2），k 小就不改；k 大时补法是升档判的抓到进结论表并带证据指针、门禁跨版本复核。
  2. 升档登记表的证据口径只拦升档那一路：往 results.tsv 手追加普通 caught 行照样判绿。用户要的「足够的证据」若指防篡改，最小补法是结果行加带密钥的校验列（密钥放库目录之外）、门禁核；若指防驱动自己判错（负载抖动、基线偏小），今天的口径够，且 K 在默认参数下由「4 倍原限时」自动满足、不是靠 20 倍在把关（第三节 D）。
  3. 「下一批治理清单」没有常驻载体：要么在拆分提案立一节，要么把第五节第 3、4、7、8、10、11、19 行逐条进欠账。

### 没做什么

- 没跑驱动真跑（`--sync-tree`、`--changed-since`、`--item --rerun`）、没起门禁整道（对真仓）、没跑 `gate.sh`、`stage-selftest.sh`（闸拒：样本未暂存）、`cargo mutants`（闸拒，`exclude_re` 只读了文档与源码）。
- 门禁样本档是我用 `stage-selftest.sh` 的起法复刻的（`outputs/samples-36.txt` 与 `runsamples.sh`），不是它本身。
- 写错版驱动与 admission.py 只在草稿目录的拷贝上、经「抽出门禁第一格内嵌 python」判（`outputs/cell-mutants.txt`），没起拷贝的 `--selftest`（闸拒仓副本）；驱动自证那一边的数是仓里开关打开的数（第四节）。
- 没改任何被判文件；没做 git 写操作；没替主 agent 撤销或维持判决。
- 报告里 `research/prompts/m3-broad-gate-gov-r3-sonnet-model/README.md` 一处我用了 `sed -i` 就地改（我自己建的文件，钩子没拒）；其余仓内文件都经 `replace-once.py` 或排他新建、追加写。
- 没读禁读清单里的文件（第三轮攻方的报告与模型目录）；读了第二轮两条腿的报告（派发提示点名可读）、调度记录 `records/2026-09-30-里程碑三开工第一轮.md`（找未定项落点，不在冻结副本里，主仓里那份；它不是被判文件）。
- 真库 `~/.local/share/singlefs/broad-mutation/`：只读（`ls`、`tree.source`、`global-key.txt` 的类别计数），随本机变，复核时重查。
- 草稿目录 `/tmp/claude-1000/m3-gov-r3-defense/` 里自己建的写错版驱动与 admission.py 拷贝（`drvs/`）、样本现场（`lit/`、`w1/`、`envroot/`、`envhome/`、`s.*`）交回前已删；留着日志与脚本（不是编译目录、工作树或仓副本）。

### 推翻总条件

第二节每行末尾与各句的推翻条件已写；整份报告最容易被推翻的两处：① 第五节第 1 行（G1-e 落空）——在拆分提案或别的常驻记录里找到那一条的字面同义句；② 第三节 D 的 347 趟——证明那些手造 caught 行在真驱动里会被 `read_results` 拒（我造的行段数、变异名、结局名都合法，指纹列取这一版）。
