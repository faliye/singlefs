# 辩方腿报告：广谱变异接进提交门禁的治理改动，第二轮（Sonnet，复核第一轮判决；H3、H4）

日期 2026-10-03。冻结副本根 `/tmp/claude-1000/m3-gov-r2-freeze/tree/`（下文写 F）。下面所有「原文」（`文件:行号`）都在 F 里的那份文件上用 `grep -nF` 现查过，行号是那份文件自己的。
引第一轮判决写「判决第 N 行」，是转述，不带引号。腿自己看的外部状态（真库、cargo-mutants 源码）标「现查」并写命令。

## 一、总判（先给结论）

- 第一轮判决 12 行：第 1、2、3、4、7、8、9 行站得住（改法够得着，对替代方案同样成立）；第 5 行选「全局键进版本指纹」站得住，「门禁另核」单独做有两条代码上的输入打不中（见三）；第 10 行够不着一半：驱动与门禁没有排除单条变异的开关是真的，但一层之下 cargo-mutants 的 `.cargo/mutants.toml` 有 `exclude_re`，判决「改成没有」把一条走得通的路写没了（见二第 10 行）；第 6 行用户定案丙：出路在「慢」与「撞内存但加大就过」的结局上走得通，在「真卡住的文档测试」与「无界分配的内存撞顶」两种结局上走不出去，「交用户定」之后门禁没有任何机制认（H3，见四）。第 11、12 行判决本来就没改，不复核。
- H4：五份文件这一批写下或挪动的句子里，没找到与今天做法不符的整句；`SINGLEFS_HEAVY_TESTS` 进脱离单元（题面「只带 PATH、HOME」不对，它带五个变量）；三处重型例外清单与 `BROAD_MUTATION_QUERY_ARGUMENTS` 逐项相同；`main-agent.md` 第 12 条两处指针都指得到。找到几处措辞债，不是整句错，见五。
- 一处判决没点名的新输入：第 9 条把 ③ 改走 `detached-run.sh` 之后，全局键读的编译配置环境变量（`RUSTFLAGS` 这一类）在 ②④（会话 shell）与 ③（脱离单元只带五个变量）之间可以不同，不同时 ③ 退 7、会响、不静默；今天两处都没设（现查），是潜在的，不是现在的红（见五 H4-6）。

## 二、判决条目复核表

| 判决行 | 辩方判定 | 证据 |
|---|---|---|
| 1 样本指纹按今天的驱动算 | 站得住 | 病根是样本 `setup.sh` 抄了旧公式；门禁自己也不抄公式，是载入驱动调同一批函数：「current_global = driver.global_key_lines(driver.JUDGE_RULES_VERSION, current_toolchain, staged, os.environ, tools,」（`.claude/gate.d/checker-tier-broad-mutation.sh:319`）。样本改成同样调驱动，是把「两份各抄一份会漂」收成一份；这一族函数写错时谁盯：驱动 `--selftest` 的 `expected_version` 自己拼了一遍公式（`research/scripts/broad-mutation.py:3408`，读代码，没跑），不靠样本 |
| 2 8 个没人盯的判红分支补样本与开关 | 站得住 | 报告第 2 条的 8 行表与门禁文件头「弄坏开关」一一对应；报告原样输出里 8 个新开关各只红 1 份（`/tmp/claude-1000/m3-r1/batch5-broad-gate-fix/report.md` 「弄坏开关逐个核」一节），读报告，没重跑（样本档在暂存前被重型闸拒） |
| 3 写标记前核工作区等于暂存区 | 站得住，且比「别的五道」更严 | 别的五道的 `git diff --quiet` 写在 `commit-stages.md` 第 19 行、由主 agent 手跑；这一道由阶段自己核，不等就不写：「echo "  ! 本次未写全绿标记：${reason:-判不了输入路径上工作区与暂存区同不同}"」（`.claude/gate.d/checker-tier-broad-mutation.sh:487`）。路径集合取登记行路径加登记表加阶段脚本：「paths += [admission.REGISTRATION_TABLE, f".claude/gate.d/{stage}"]」（`.claude/gate.d/checker-tier-broad-mutation.sh:467`），与指纹那一侧的 `listed_input_files` 同用 `--exclude-standard`（`research/scripts/admission.py:477`）；两侧路径集合逐项是否相同归攻方 H1 |
| 4 开格前钉住 tree.source、跑完再核 | 站得住，范围到 tree.source 为止 | 钉住的是指针文件；驱动 `--summary` 读库目录里活的指针，门禁拿汇总行的 fingerprint 与钉住的那一版比：「print(f"  ✗ 汇总行的 fingerprint={summary.get('fingerprint')} 不是库目录 tree.source 那一版（{version[:16]}）：判的不是同一版")」（`.claude/gate.d/checker-tier-broad-mutation.sh:380`），所以两格之间换版本两头都有。库目录里别的文件（结论表）在两格之间被换：第一格读 `global-key.txt` 时核了它的 sha256 等于 tree.source 的 global_key：「if hashlib.sha256(recorded_global.encode()).hexdigest() != global_digest:」（`.claude/gate.d/checker-tier-broad-mutation.sh:310`），第二格不再读它；这一支归攻方 H5，我不下判 |
| 5 全局键进版本指纹，复用登记补三样 | 站得住；「门禁另核」单独做够不着两条输入 | 见三。复用登记那一行补 `run-with-memory-cap.sh`、`capped.sh` 与 `environment=` 的 `--tool-versions`（`.claude/gate.d/stage-inputs.tsv` 的 `checker-tier-broad-mutation.sh` 那一行，`awk -F'\t' '$1=="checker-tier-broad-mutation.sh"'` 现查含这三样） |
| 6 丙：重跑出路 | 部分站得住，部分够不着 | 见四 |
| 7 两行各数互核 | 站得住（读代码） | 门禁各等式与 `summarise` 里 `without` 的算法逐项对：`without` 取「状态不在 DECISIVE_OUTCOMES」的条，恰是汇总行的超时加内存撞顶加只过了第一段加驱动错加没跑；判定行 `carried + fresh == concluded` 因 `fresh` 是余数，恒成立，只防打印写错，不是独立证据（`research/scripts/broad-mutation.py:2078` 一带） |
| 8 重型闸两处误拒、清单对齐、旧名 | 站得住 | 我在 F 里用 `heavy-test-guard.sh` 喂了命令，只判分类，不跑被判命令：驱动 `--check-ignored` 主 agent 放行（退 0），门禁 `--check gate-verdict` 放行（退 0），门禁 `--check library-version-is-staged-tree` 照拒（退 2），子 agent 带前缀跑 `--coverage-step` 照拒（退 2）。「带任一只读参数就放行」没有洞：「mode = parser.add_mutually_exclusive_group()」（`research/scripts/broad-mutation.py:2559`）把 `--summary` 与 `--coverage-step`（`:2566`）互斥，同给被 argparse 拒 |
| 9 逐句改 | 站得住，几处措辞债 | 见五 |
| 10 等价变异排除的出路改成「没有」 | 够不着一半 | 驱动与门禁的新句子字面对：「等价变异没有排除表、驱动没有排除单条变异的开关，判不出的没红照旧判红、交用户定」（`research/scripts/broad-mutation.py:2095`），`EXCLUSIONS_FILE` 管的是忽略用例（`:300`）。但清单是 `cargo mutants --list` 在导出树里生成的：「as_json = subprocess.run(base + ["--json", "--diff"], capture_output=True, text=True, cwd=library.tree, env=environment, stdin=subprocess.DEVNULL)」（`research/scripts/broad-mutation.py:1222`），树里带 `.cargo/`（导出范围，`:54`），cargo-mutants 27.1.0 的配置有 `exclude_re`（现查：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cargo-mutants-27.1.0/src/config.rs` 第 60 行 `pub exclude_re: Vec<String>,`，`book/src/filter_mutants.md` 第 44 行起）。所以等价变异有一条走得通的排除路：在 `.cargo/mutants.toml` 加 `exclude_re`、重做 `--sync-tree`。代价是整版重判：`.cargo/` 下的文件进全局键（「if os.path.basename(path) in ("Cargo.toml", "Cargo.lock") or path.startswith(".cargo/"):」`research/scripts/broad-mutation.py:2251`），键变了结论一条不沿用（「if record["global_key"] != global_key and not broken("global-key-ignored"):」`research/scripts/mutation-prune.py:644`）。第一轮 G1-d 的「grep 零命中」只 grep 了驱动与门禁，没往 cargo-mutants 配置里看。是否要写进出路、要不要这条路（它让「没红」可以被人从清单里拿走），交主 agent / 用户 |

## 三、第 5 行：「全局键进版本指纹」对「门禁另核」（替被判出局的一方也找一遍理由）

判决第 5 行写的是「全局键进版本指纹（或门禁另核）」，工具实现员两样都做了。问：只做「门禁另核」行不行。

事实（F 里现查）：
- 结果行没有全局键这一段：「"finished_at", "fingerprint", "note")」（`research/scripts/broad-mutation.py:308`）；全局键只在结论表那边：「CONCLUSION_COLUMNS = ("key", "outcome", "caught_by", "test_fingerprint", "fingerprint", "dependency", "global_key", "version", "commit", "sequence", "finished_at")」（`research/scripts/broad-mutation.py:311`）。所以 `results.tsv` 里一行是在哪个全局键下判的，从行上看不出，只能靠它住在哪个版本目录。
- 每一趟真跑都把 `global-key.txt` 覆盖成这一趟的：「write_atomically(library.version_file("global-key.txt"), pruner.global_text.encode())」（`research/scripts/broad-mutation.py:2664`）。
- 同一份源码、同一工具链、同一判法版本下重做 `--sync-tree`，目录名不变，已有的清单与结果表照用：「if os.path.isfile(library.version_file("list.json")):」（`research/scripts/broad-mutation.py:1141`）；`sync_tree` 里没有任何一行去动 `results.tsv`（现查：`grep -n results.tsv` 在 `sync_tree` 函数体里无命中）。

两条「只门禁另核」打不中的输入：
1. 排除表改了，主 agent 不重做 ② 直接再跑一趟 `--changed-since`：若驱动没有 `check_global_key`，上面 2664 行把 `global-key.txt` 换成新键，门禁现算的与它相同，判绿；这一版里旧键下判的行一条不重判。报告第 5 条已讲这一条。
2. 补上驱动的 `check_global_key`（退 7）堵住 1：改键之后必须重做 ②。但重做 ② 在同一提交、同一源码下落回同一个版本目录（1141），旧键行还在 `results.tsv` 里照算。要堵它，就得「换键时另起目录」或「换键时清结果表」；前者就是把全局键放进目录名（版本指纹），后者要删用户的结果行，更糟。

所以进指纹是堵 1 与 2 的最小改法，且与 E4（全局键变了整版重判）同字面。门禁现算这一支仍留着，它堵的是第三种：最后一趟真跑之后、④ 之前环境变了（工具升级、排除表被改）——驱动没再被调，库目录仍是旧键，`--summary` 照绿，只有门禁现算会红。这一支是边际收益，成本低（调的是同一个函数，`.claude/gate.d/checker-tier-broad-mutation.sh:319`）。

替出局一方找的理由是「别让旧版本目录作废」。今天不成立（现查，命令在末尾）：真库当前版本 `e1ba…` 的 `results.tsv` 2 行（caught 1、unviable 1），`conclusions.tsv` 2 行结论，清单 5091 条；作废它的代价约等于零，第一趟 ③ 本来就要现判约 5089 条。往后结论多了，换键整版重判是 E4 本来的要求，不是进指纹新添的：「if record["global_key"] != global_key and not broken("global-key-ignored"):」（`research/scripts/mutation-prune.py:644`）早就让换键之后的结论一条不沿用。

进指纹新添的约束：限时参数与 `--llvm-bin` 在 ② 与 ③ 要一致。`commit-stages.md` 第 25、26 行的 ② ③ 都不带这几个选项，默认值一致；`limits_line_of` 只取 cpu 与 wall 四个：「return f"limits cpu={arguments.cpu_multiplier}x+{arguments.cpu_floor} wall={arguments.wall_multiplier}x+{arguments.wall_floor}"」（`research/scripts/broad-mutation.py:2267`），不含 `--timeout-floor`、`--memory-cap`、`--threads`，所以出路里「加大 --timeout-floor 或 --memory-cap」不改键、不退 7、结果行追加在同一张表里（H3 用到）。

什么现象会推翻第 5 行的这个选择：找到一个不动目录名、只靠「驱动退 7 加门禁现算」就同时挡住输入 1 与 2 的改法；或驱动 `--selftest` 在 `BROAD_MUTATION_BREAK=global-key-outside-version` 下不红（报告第 5 条说红，我没跑）。

## 四、H3：用户定案丙的出路走不走得通

出路的原句（驱动文件头）：「重跑还是这两种的，加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun 点名重跑，仍没结论的门禁照旧红、交用户定」（`research/scripts/broad-mutation.py:95`）。

### H3-1　机制走得通的那几步（读代码，没跑）

- 第一步，`--changed-since` 也重跑这两类：「elif (plans is not None and state in ENVIRONMENTAL_OUTCOMES and not arguments.stage_two_only」（`research/scripts/broad-mutation.py:2017`）。只在给了 `--changed-since`（plans）时选；条还得在 plans 里是「要现判」，否则被 2009 行的「if plans is not None and not plans[item["sequence"]][0]:」滤掉。timeout、memory-cap 不进结论表：「if runner.pruner and outcome in DECISIVE_OUTCOMES:」（`research/scripts/broad-mutation.py:1875`），所以它们没有可沿用的同键 caught 记录，plans 必给现判（推的，由 `carry_decision` 与 `record_conclusion` 推出，没跑）。
- 第二步，加大限时或内存：限时是 `max(floor, 倍数 × 基线)`：「return int(max(self.arguments.timeout_floor, self.arguments.timeout_multiplier * self.baseline_seconds.get(step_name, 0)))」（`research/scripts/broad-mutation.py:1639`）。真库基线表里编译步与文档测试步最大 95.7 秒，倍数 5 也不过 480 秒，低于默认 floor 900，所以 floor 说了算，加大 floor 有效（现查 `versions/19c45e02…/baseline.tsv`）。加大 `--timeout-floor`、`--memory-cap` 都不在全局键里（见三），不退 7、不换版本。
- `--item <序号> --rerun` 可多次给，追加新行、旧行留着；状态取最后一条定论，再抓到、没红、无效就解开：`state_of_outcomes`（`research/scripts/broad-mutation.py:771`）。通用路径有自证格：「点名加 --rerun 没有重跑并追加新行」（`research/scripts/broad-mutation.py:3216`）；第一步有专格 `selftest_environmental_cells`（`:3774`）；**第二步「加大限时再 --item --rerun」没有专格**（`grep -n timeout-floor` 只在文件头、解析与两处别的自证里命中）。
- 出路不需要重做 ① ②：版本目录不换，③ 之后直接接着做。

### H3-2　真库的规模下，一次提交里走不走得完

- 现查：真库两个版本目录的清单都是 5091 条；旧版本 `19c45e02…` 的 `results.tsv` 303 行，build 加 test 秒数合计 37818.7，其中 7 行 timeout，全是 `one-test` 整步超时（`awk -F'\t' '$4=="timeout"' …/results.tsv`，备注「one-test 超过 1392 秒」）。新驱动下测试步卡住算抓到（「RUNNER_TEST_STEPS = ("one-test", "two-test-slow-harness", "two-test-checker-tier")」`research/scripts/broad-mutation.py:317` 这三步不设整步限时），这 7 条在新驱动下会变「抓到」，所以旧数不能估新驱动下没结论的条数 k；k 没量过。
- 出路的成本与「一版几千条」无关，与 k 线性：每条每趟至多 `max(floor, 5×基线)` 秒（默认 floor 900），出路至少两趟（第一步同参数重跑、第二步加大后再来）。k 为几条到几十条、且重跑能出结论的结局里，一次提交走得完；第一趟 ③ 本身（约 5089 条现判，判决第 9 行转述的 10.6–82.0 小时不是我算的）才是一次提交里的大头。
- 小摩擦：`NO_CONCLUSION` 行只打序号、状态、变异名：「say(f"NO_CONCLUSION\t{item['sequence']}\t{(states.get(str(item['sequence'])) or ItemState(NOT_RUN, False)).state}\t{item['name']}")」（`research/scripts/broad-mutation.py:2081`），上一趟用的限时秒数在结果行备注里（「return record("timeout", stage, note_text=f"{step_name} 超过 {runner.time_limit_of(step_name)} 秒"」`research/scripts/broad-mutation.py:1906`）；floor 不加到比那个秒数大，加大无效，出路文字没写这一句。

### H3-3　走不出去的结局组合（判据第 4 条）与「交用户定」之后

出路的两步（重跑、加大再重跑）对下面三种结局给不出结论，最后一步只剩「交用户定」：

| 组合 | 为什么走不出去 | 证据 | 概率 |
|---|---|---|---|
| C1 文档测试真卡住：变异把被测代码弄成死循环，第一段没抓到，只有 `cargo test --doc` 跑得到它 | `two-test-doc` 不在三个经 runner 限 CPU 的步里，按整步限时记 timeout；真卡住的加大 floor 还是撞，每趟白烧一个 floor | 「RUNNER_TEST_STEPS = ("one-test", "two-test-slow-harness", "two-test-checker-tier")」（`research/scripts/broad-mutation.py:317`）；「if exit_code == TIME_LIMIT_HIT_EXIT:」（`research/scripts/broad-mutation.py:1905`）；真库旧基线 `two-test-doc` 1.5 秒（现查），限时就是 floor | 没量过；要第一段漏掉、只有文档测试跑得到、还卡住，推的：低 |
| C2 无界分配（尺寸算术的变异）撞内存 | 加大 `--memory-cap` 还是撞，直到包装退 252；测试步里撞顶也记 memory-cap，不记抓到：撞顶的分支排在「测试没过」之前 | 「if exit_code == MEMORY_CAP_HIT_EXIT:」（`research/scripts/broad-mutation.py:1907`）在「if kind == "test" and exit_code == test_failed_exit:」（`research/scripts/broad-mutation.py:1914`）之前；驱动自己写过这种变异存在：「整条包一层的话一条无界分配的变异会连驱动一起杀掉，续跑又撞同一条、永远过不去。」（`research/scripts/broad-mutation.py:86`） | 没量过；真库至今 0 行 memory-cap（现查），新驱动下结果行才 2 行，不能外推 |
| C3 重起带来的重复开销（不是死路，是费时） | ③ 被停重起，plans 在，本趟已落的 timeout / memory-cap 行又被 2017 行选一遍；`commit-stages.md` 第 26 行的「已有定论的条不重跑，被打断的那一条重跑」只对 caught / missed / unviable 成立；驱动注释还写着「也算定论、不再自动重跑」（`research/scripts/broad-mutation.py:321`，已过时） | 「ENVIRONMENTAL_OUTCOMES = ("timeout", "memory-cap")            # 也算定论、不再自动重跑，但不盖掉前面的 caught / missed / unviable」（`research/scripts/broad-mutation.py:321`） | 每次重起重烧本趟已落的 k0 条乘限时 |

按跑前写死的判据（丙的出路在某种结局组合下走不出去 ⇒ 打中，交用户）：C1、C2 是打中，C3 不是。这一点我不替它辩：用户 2026-10-02 定丙要的是「补一条走得通的出路」，对这两类，出路的终点是「交用户定」。

「交用户定」之后门禁认什么：
- 绿只认驱动 `--summary` 的判定行：「gate_red = bool(without or counts["missed"])」（`research/scripts/broad-mutation.py:2077`），门禁文件头写它判的就是这一行与汇总行（「超时、内存撞顶照驱动算没结论（用户 2026-10-02 定）：出路是 --changed-since 重跑它们，仍没结论的照旧红、交用户定。」`.claude/gate.d/checker-tier-broad-mutation.sh:32`）。
- 现查：`grep -n '人工\|豁免\|waive\|--decide'` 在 F 里的驱动与门禁文件各命中 0 处（门禁 0 处里不含上面第 32 行那句，那句写的是「交用户定」，不是机制）。**没有让门禁认用户判决的机制。**
- 用户判完之后，今天能让门禁变绿的只有：改代码或测试（新版本整版判）；改 `.cargo/mutants.toml` 的 `exclude_re`（见二第 10 行，整版重判）；往这一版的 `results.tsv` 手追加一行 caught 或 unviable——`read_results` 只核段数、变异名与结局名（「if outcome not in ROW_OUTCOMES:」`research/scripts/broad-mutation.py:882` 一带），不核来源，驱动与门禁都会收（推的，没跑；它不进结论表，下一版不沿用，也没有任何文档写过这条）。所以丙括号里的「人工逐条判」今天没有落点；要不要立一个「用户判」的登记、什么形态，是要用户另定的一件事，我不设计。
- 反过来，对「慢」与「撞内存但加大就过」的结局，出路走得通：不重做 ①②、不换版本、不整版重判，只追加行。

什么现象会推翻 H3 的这几条：C1、C2 在真库首趟 ③ 之后一条都没出现（`--summary` 的 NO_CONCLUSION 为 0，或剩下的全是加大后能过的）；F 里找到让门禁认用户判决的机制（我 grep 的词没覆盖到的写法）；第二步「加大限时再 `--item --rerun`」在驱动 `--selftest` 里有专格（我只看到 `--timeout-floor` 在 3001、3650 两处别的格里出现）。

（上一段「门禁 0 处」指 `人工`、`豁免`、`waive`、`--decide` 四个词在驱动与门禁里都命中 0 处；`交用户定` 四个字在门禁与驱动里命中，那是出路文字，不是机制。）

## 五、H4：五份文件里这一批写下或挪动的句子

| 句子或位置 | 判定 | 证据 |
|---|---|---|
| H4-1 `commit-stages.md` 第 26 行 ③ 的前缀 `SINGLEFS_HEAVY_TESTS=commit` 进不进脱离单元 | 站得住；题面「只带 PATH、HOME」不对 | 「带进单元的环境：调用方的 SINGLEFS_HEAVY_TESTS、PATH、HOME、CARGO_HOME、RUSTUP_HOME」（`research/scripts/detached-run.sh:17`），「DETACHED_RUN_CARRIED_VARIABLES=(SINGLEFS_HEAVY_TESTS PATH HOME CARGO_HOME RUSTUP_HOME)」（`research/scripts/detached-run.sh:209`）。承重点有两处：驱动在单元里先用它过自己的前缀核「if os.environ.get(HEAVY_PREFIX_VARIABLE) not in HEAVY_PREFIX_VALUES:」（`research/scripts/broad-mutation.py:2619`），没带进去就退 2、一步不起；带进去之后，内存包装对带前缀的不判「上限不超过实测峰值 2 倍」：「带 SINGLEFS_HEAVY_TESTS（=commit / =user-request，提交时与用户要求的重阶段，主 agent 跑的）的不判这一条」（`research/scripts/run-with-memory-cap.sh:79`）。后一处保证出路的第二步不被这条挡：峰值表 `research/scripts/memory-peaks.tsv` 第 2916 行 `broad-mutation two-test-doc` 的实测峰值 90177536 字节（约 86 MiB），默认 24G 与任何加大后的 `--memory-cap` 都远超它的 2 倍，带前缀都不判（没带前缀时驱动先退 2，到不了包装）；读代码，没跑。重型闸把 `start … --` 当包装看穿：「research/scripts/detached-run.sh 的 `start … -- <命令>`（把命令挂到用户级 systemd 下脱离会话跑，调用方的 SINGLEFS_HEAVY_TESTS 带进去）同样剥掉」（`.claude/hooks/heavy-test-guard.sh:98`）。我在 F 里把 ③ 原句喂闸（只判分类）：带前缀主 agent 退 0，不带前缀退 2，退 2 的原因写「主 agent 跑「广谱变异」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带」 |
| H4-2 第 28 行退出码与驱动文件头逐个对 | 站得住；一处遗漏在驱动头 | 第 28 行点名 4、3、6、7、143：「退 3、6、7 照抄（7 是工具链或全局键与这一版记的不同，出路照驱动打的另起一版）」（`.claude/skills/crash-test/commit-stages.md:28`）对驱动头：「3 内存包装自己的结局（251 起不来、252 排不上、254 被总上限挤掉）：那一条不落行，修好重跑同一条命令；」（`research/scripts/broad-mutation.py:125`）、「4 锁被占着；5 树脏了恢复不了、树里多出文件、上一次 --sync-tree 没做完、编译目录里可能留着换树之前编的工作区产物；6 基线不绿、或疑似环境坏了；」（`research/scripts/broad-mutation.py:126`）、「143 被 TERM / INT 停（在途那一条不落行）。」（`research/scripts/broad-mutation.py:128`）；代码里常量表也是这些：`research/scripts/broad-mutation.py:341`。1、2、5 没单列，落在「原样抄驱动的「✗」「→」与退出码」那一句里。驱动头的「退出码」一段没有 8（`--summary` 判红：「EXIT_GATE_RED = 8」`research/scripts/broad-mutation.py:318`），8 是 ④ 里门禁调 `--summary` 得到的、不是 ②③ 的退码，不影响第 28 行，是驱动文件的一处缺 |
| H4-3 `main-agent.md` 第 12 条挪过去之后两处指针 | 站得住 | 第 12 条在：「12. **提交时的六道重阶段，上限由主 agent 自己定**」（`.claude/main-agent.md:30`）；「暂存之后、提交之前跑门禁」那一行的指针：「内存上限照「一轮怎么开、怎么收」第 12 条各给」（`.claude/main-agent.md:68`）指得到。`.claude/main-agent.md@派发提示怎么写` 那一节仍留着给 research-unit-tests 与 harness-model-differential-and-scenarios 的那一句（`.claude/main-agent.md:54`）。全 `.claude` 下对「派发提示怎么写」的引用只剩 `.claude/main-agent.md:38` 与那节标题，都指得到。位置债：第 12 条放在「一轮怎么开、怎么收」里，与那一节其余各条（怎么开一轮）主题不同，不是错 |
| H4-4 三处重型例外清单与 `BROAD_MUTATION_QUERY_ARGUMENTS` 逐项相同 | 站得住 | 集合：「BROAD_MUTATION_QUERY_ARGUMENTS = frozenset({"--list-items", "--summary", "--check-ignored", "--help", "-h"})」（`.claude/hooks/lib_heavy_tests.py:403`）。三处同一句：「`--list-items`、`--summary`、`--check-ignored`、`--help`（`-h`）与登记为 light 的单独 `--selftest` 不算）与它的提交门禁 checker-tier-broad-mutation（只点名 gate-verdict 一格的 `--check gate-verdict` 不算）」（`.claude/agent-common.md:52`）、「`--list-items`、`--summary`、`--check-ignored`、`--help`（`-h`）与登记为 light 的单独 `--selftest` 不算）与门禁 checker-tier-broad-mutation（只点名 gate-verdict 一格的 `--check gate-verdict` 不算）」（`.claude/rules/verification.md:108`）、同句（`.claude/rules/implementation-workflow.md:57`）；闸文件头：「只有带 --list-items / --summary / --check-ignored / --help（-h）的不算（仓内那一份单独一个 --selftest 照自证登记表判）」（`.claude/hooks/heavy-test-guard.sh:45`）。措辞债：闸打给人看的拒绝说明少 `-h`：「只有 --list-items、--summary、--check-ignored、--help 不算）")」（`.claude/hooks/lib_heavy_tests.py:1712`），不影响放行 |
| H4-5 第 26 行 ③ 其余句子 | 站得住；一句半对 | 标签 `broad-mutation-changed-since` 合 `detached-run.sh` 的写法：「DETACHED_RUN_LABEL_PATTERN='^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$'」（`research/scripts/detached-run.sh:210`）；「驱动每条命令自己经内存包装，外面不再包一层」（`.claude/skills/crash-test/commit-stages.md:26`）与「广谱变异驱动 `research/scripts/broad-mutation.py`（每一步各经包装，上限是它的 `--memory-cap`）」（`.claude/agent-common.md:54`）同；被停退 143、不落行与驱动头 128 行同；脱离入口的默认停机时限 180 秒也写了驱动先停在途那一步再退：「（broad-mutation.py 先停在途那一步的子进程再退；」（`research/scripts/detached-run.sh:37`）。半对的一句：「已有定论的条不重跑，被打断的那一条重跑」（`.claude/skills/crash-test/commit-stages.md:26`）对 caught / missed / unviable 成立，对 timeout / memory-cap 不成立（C3，见四） |
| H4-6 ② ④ 与 ③ 的环境不同（判决没点名） | 句子没错；潜在的重复撞 7 | 全局键读这几个环境变量：「if name in ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS") or name.startswith("CARGO_PROFILE_"):」（`research/scripts/broad-mutation.py:2254`）。② 与 ④ 在会话 shell 里起，③ 经脱离入口只带五个变量：「别的变量不带：单元里是用户级 systemd 给的那一份环境（systemctl --user show-environment）加上这几样；要别的，写成 -- env 名=值 <命令>。」（`research/scripts/detached-run.sh:18`）。会话 shell 里要是设了上面任一个，② 把它写进全局键，③ 在单元里算出没有它的键，真跑之前核：「    check_global_key(library, arguments)」（`research/scripts/broad-mutation.py:2800`）退 7；第 28 行的出路「照驱动打的另起一版」是重做 ②，而 ② 仍在会话 shell 里、仍带那个变量，③ 再退 7，循环到主 agent 在 ③ 的 `--` 后面加 `env 名=值` 为止。响，不静默。今天两处都没设（现查，命令在末尾），所以是潜在的 |
| H4-7 `verification.md`、`implementation-workflow.md`、`agent-common.md` 里别的广谱句子 | 站得住 | 「gate-verdict 格判驱动 `--summary` 的 `BROAD_MUTATION_GATE` 行与汇总行」（`.claude/rules/verification.md:100`）与门禁两格的现状对得上（第一格多判全局键，第二格多核汇总行，见门禁文件头 20–32 行）；`commit-stages.md` 第 12 行与 `main-agent.md` 第 12 条对广谱驱动给「内存上限与线程数」的写法一致：「广谱变异驱动（第 2b 步）的两个上限」（`.claude/skills/crash-test/commit-stages.md:12`） |

五份文件里没有找到「一句话与今天做法不符」的整句（跑前判据第 2 条没被打中）。上面的措辞债与 H4-6 都是主 agent 要不要改、怎么改的事，我没改任何文件。

什么现象会推翻 H4：重型闸对 ③ 原句（带前缀的主 agent）不退 0；单元里读到的 `SINGLEFS_HEAVY_TESTS` 不是 commit（`detached-run.sh` 自证 Ⓓ 格红：`research/scripts/detached-run.sh` 的 `--selftest`，我没跑，要主 agent 跑）；`main-agent.md` 第 12 条之外还有哪处引用「派发提示怎么写」里的六道上限句（我 grep 全 `.claude` 与 `research/scripts` 只找到上面两处）；会话 shell 或用户级 systemd 的环境里今天就有上面那几个变量。

## 六、要主 agent 跑的命令（我没跑；都在主仓根目录、不起 cargo 编译）

1. ② 做完之后量 ③ 的选法与耗在计划上的时间（只读，驱动头写「库目录一个字节不写」；不要前缀）：
   `cd /home/fy5090/code/singlefs && time nice -n 19 python3 research/scripts/broad-mutation.py --list-items --changed-since HEAD > <草稿目录>/changed-since-plan.tsv`，再 `cut -f4 <草稿目录>/changed-since-plan.tsv | sed 's/:.*//' | sort | uniq -c`。看「carry」与「judge」各几条；`time` 的 real 是每趟驱动起来在选条上的固定开销，我没量过。
2. 首趟 ③ 跑完之后量 C1、C2 的真实频率：
   `python3 research/scripts/broad-mutation.py --summary > <草稿目录>/summary.txt; grep -c '^NO_CONCLUSION' <草稿目录>/summary.txt; grep '^NO_CONCLUSION' <草稿目录>/summary.txt | cut -f3 | sort | uniq -c`，
   再 `awk -F'\t' '$4=="timeout"||$4=="memory-cap" {print $1"\t"$4"\t"$5"\t"$11}' ~/.local/share/singlefs/broad-mutation/versions/<tree.source 里的 fingerprint>/results.tsv`，看备注里的步名：`two-test-doc` 或 memory-cap 出现就是 C1 或 C2 在真库里发生了；全是 `one-build`、`two-build-*` 的编译步超时，加大 floor 多半能过。
3. 起 ② 之前确认 H4-6 的前提（两条都应当没有输出；我这一趟两条的退出码都是 1，无命中）：
   `env | grep -E '^(RUSTFLAGS|RUSTDOCFLAGS|CARGO_BUILD_RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|CARGO_PROFILE_)'`；
   `systemctl --user show-environment | grep -E '^(RUSTFLAGS|RUSTDOCFLAGS|CARGO_BUILD_RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|CARGO_PROFILE_)'`。
4. 推翻第 5 行与 H4-1 的两条自证（登记表里都是 light，`.claude/hooks/selftest-registry.tsv` 第 27、47 行）：
   `BROAD_MUTATION_BREAK=global-key-outside-version nice -n 19 python3 research/scripts/broad-mutation.py --selftest`（报告第 5 条说退 1、两格红：「工具链没进指纹」与「排除表改了之后这一版的旧结论照算」）；
   `nice -n 19 bash research/scripts/detached-run.sh --selftest`（看 Ⓓ 格：SINGLEFS_HEAVY_TESTS 带进单元）。
5. 我对重型闸喂的分类（只判分类，不跑被判命令；在冻结副本根目录里）：`printf '%s' '{"tool_name":"Bash","tool_input":{"command":"<命令>"},"cwd":"."}' | bash .claude/hooks/heavy-test-guard.sh; echo rc=$?`，命令取 ③ 原句（带前缀与不带前缀）、`bash .claude/gate.d/checker-tier-broad-mutation.sh --check gate-verdict`、`python3 research/scripts/broad-mutation.py --check-ignored`；子 agent 的在 JSON 里加 `"agent_type":"implementation-writer"`。

## 没做什么

- 没跑驱动的任何真用法（`--sync-tree`、跑条、`--changed-since`、`--item --rerun`）、没跑 `--selftest`（驱动、钩子、`detached-run.sh` 的都没跑）、没起门禁整道、没跑 `stage-selftest.sh`；H3 的「出路走得通」都是读代码，没有一趟真跑支撑，上面标「推的」「读代码」处同。
- 没替主 agent 撤销或维持第一轮判决：第 10 行的「`.cargo/mutants.toml` 的 `exclude_re`」与 H3 的 C1、C2、「用户判决没有落点」交主 agent 与用户定；没提改法的实现，也没设计「用户判」登记的形态。
- 没复核判决第 11、12 行（本来没改）；没攻 H1、H2、H5（归攻方）：第 3、4 行里「路径集合逐项相同」「库目录里别的文件被换」只写了我看到的那一半，注明了归攻方。
- 没读禁读清单里的文件（`research/prompts/m3-broad-gate-gov-r2-opus-output.md`、`research/prompts/m3-broad-gate-gov-r2-opus-model/`、`records/` 下的调度记录）。
- 没改任何仓内文件：报告之外只在 `/tmp/claude-1000/gov-r2-defense/` 写了几份草稿（报告的分段底稿、重型闸喂法的输出 `g.out`），没有编译目录、worktree、仓副本，不需要删。
- 现查的外部状态：真库 `~/.local/share/singlefs/broad-mutation/`（`ls`、`cat tree.source`、对 `list.json` 与 `results.tsv` 的 python 只读统计）、`research/scripts/memory-peaks.tsv` 的两行、`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cargo-mutants-27.1.0/` 的两处源码。这几处不在冻结副本里，会随本机变，复核时重查。我曾想用 `cargo mutants --help` 看 `exclude_re`，被重型闸拒（闸把 `cargo mutants` 算进 cargo test 一类），没绕，改读了源码。
