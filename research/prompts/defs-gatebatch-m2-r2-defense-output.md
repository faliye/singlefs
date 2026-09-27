# defs-gatebatch-m2-r2 云端辩方（Sonnet）报告：K5（复核 r1 判决）兼 K2（红证办法能证什么）

范围：只做 K5（复核第一轮判决 J1-a、J1-b、J4-b、J2-a、J2-b、J3-a 各自打中是否真、Y1–Y8 对不对症、判决略过了什么、替被判出局的写法辩护）与 K2（改法的证红办法证得了什么、证不了什么；O2/O4/B6/B7/R2、54 号自证 4 格假 cargo）。不碰 K1/K3/K4（另两条腿）。

方法：对判决里每一条引文，在被引文件里 `grep -nF` 现查一次；命中的行号就是要写的行号。凡是判决没给具体行号的（J2-a/J2-b/J3-a 那三行「依据」列只写「攻方」，没有行内引文），不编行号去核，直接核它引的攻方报告与探针产物是否支持判决的措辞。

## 一、K5：逐条打中是否站得住

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| J1-a（只改登记表或 54 号本身，快档退 77） | **站得住** | 判决引 `54-layer0-replay.sh:114`。改法前备份（`/tmp/claude-1000/gate-batch-m2-r1-fixes/backup/.claude/gate.d/54-layer0-replay.sh:114`）原文恰是 `scope_reason="$(bash "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/change-touches-crates.sh" "$ROOT" "${layer0_registered_input_paths[@]}")"`（`grep -nF` 命中 1 次，行号 114，逐字节相同）；`layer0_registered_input_paths` 取自 `stage-inputs.tsv` 里 54 号自己那一行的第二列，该行原文 `crates/ Cargo.toml Cargo.lock`（备份 `stage-inputs.tsv:24`，`grep -nF` 命中）。这一行是「阶段」登记，不是「用例」登记，只改 `crash-case:` 那几行或 54 号本身确实碰不到这三个前缀，range 问必答「没碰」。opus 报告 `defs-gatebatch-m2-r1-opus-output.md:81` 给的正是同一行同一引文，核查员对它判 ✓（`defs-gatebatch-m2-r1-verifier-output.md`）。Y1 修法已现查落地：现仓 `54-layer0-replay.sh:118` `layer0_judge_paths=(.claude/gate.d/stage-inputs.tsv "$(realpath ... layer0_stage_script_path)" "$(realpath ... layer0_admission_module)")`，`stage-inputs.tsv:26` 第二列已加 `research/scripts/admission.py`；range 问现在把登记表本身、54 号自己、准入模块一起算进「碰没碰」。**推翻条件**：找到一条不在这三个新增路径里、也不在原三前缀里、但确实改变某条用例判定结果的输入（K1 的射程，不是这里）。 |
| J1-b（判法搬进 admission.py，不进指纹） | **结论站得住，判决自己的行内引文是错的** | 判决第 16 行写「`admission.py:168`」。`grep -nF` 现查：现仓与备份 `admission.py:168` 都是 `"""进指纹的只有键与路径：准入条件判的是「能不能跑」，不是「算什么」，改它不该让没变的输入放行。"""`——这是 `AdmissionCondition.fingerprint_text` 的文档字符串，判的是「准入条件」（cargo/rustc 版本这类前提），不是崩溃枚举用例的日志判法，字面上够不着「判日志的代码…它不进每条用例的指纹」这句话。真正支持这句话的引文在别处，且都能现查命中：`judge_crash_case_log` 定义在 `admission.py:1265`（备份同）；改动前判法写在 54 号正文，`--extra-file` 只把 54 号自己算进指纹，原文 `54-layer0-replay.sh:168`（备份）`      --extra-file "<判它的 54 号：$layer0_stage_file_name>" "$layer0_stage_script_path" --toolchain --build-environment)"; then`（`grep -nF` 命中）；核查员对 `54-layer0-replay.sh:168`、`admission.py:1265`、`admission.py:1321` 三条都判 ✓（`defs-gatebatch-m2-r1-verifier-output.md:147,150,151`）。判决第 16 行的「admission.py:168」这四个字在核查员核过的引文清单里一次都没出现——这是主 agent 压缩摘要时把文件名或行号错配了，不是攻方或核查员的错。**结论本身**（判法不进指纹）不受影响：探针 `probe-j1-new.log` 的 A4 系列（旧判法判绿写标记 → 修好判法后指纹不变、复用、快档判绿 → 这是 bug；Y1 修完后 A4「提交了修好的判法」这一格指纹改变，`旧 e34004646b8a489b… 新 c850971e1cbe77e6…`）在改后仓复现为 holds，说明 Y1 确实堵上了这一条。**推翻条件**：判决原文若能在别处找到「admission.py:168」被正确引用的上下文，此项撤销；未找到。 |
| J4-b（用例函数 `#[ignore]` 没人查） | **站得住** | 判决未给行内引文，但攻方报告给了可核的历史（`defs-gatebatch-m2-r1-opus-output.md:190-198`，I1/I2）。改后仓复跑同一形（`probe-j4-new.log` I1/I2 两行）：I1「副本里去掉那一行 `#[ignore]` 之后，点名 c561 目标、不带 `--ignored`」现仓判 `holds`、退 2（Y2 已堵住这一条：闸现在读源码核标没标 `#[ignore]`）；I2「同一个副本上 `admission.py crash-cases`」现仓判 `holds`，`stdout` 首行原样「`.claude/gate.d/stage-inputs.tsv 第 37 行：崩溃枚举用例 crash-case:c561-sigma-full 的用例函数 every_crash_state_of_sigma_leaves_every_u`」（`probe-j4-new.log` 原样，未截断的那一行在同文件）。这两格是「Y2 修完之后」的复跑，证明 Y2 对症；J4-b 在 r1（Y2 之前）打中的判定本身没有变化余地——它问的是 r1 时刻的旧代码，旧代码那时确实没有这一处检查，这一点判决与攻方一致，我没找到反例。**推翻条件**：找到 r1 时刻旧代码里已经存在的等价检查（未找到）。 |
| J2-a（排除法漏认，`include!(concat!(…))` 与按目录读的 `build.rs`） | **站得住，但判决自己标注的「今天的仓里没有这种写法」要看紧——这是限定语，不是反驳** | 判决措辞已经自带限定：「打中（量过），今天的仓里没有这种写法」。核对攻方报告 `defs-gatebatch-m2-r1-opus-output.md:117-127`（U1、U2）：两个探针都是「形状是构造的」（攻方自己写的四句「跑前改法还中不中」也写明「形状是构造的」）。改后仓复跑 `probe-j2-new.log`：U1「用例在编译期 `include!(concat!("other_", "target.rs"))` 把别的测试目标编进自己」holds（改前 `f567306e09a303bb…`，改后 `acb01dafe2579759…`，指纹确实变了——即 Y5 已经堵住这个构造形状）；U2「另一个包的 `build.rs` 按目录读 `crates/pkg/tests/`」holds，同样变了。**这与判决「打中」不矛盾**：Y5 已经把这两种写法接进「认不出的」清单（`admission.py` 文件头，判决 Y5 原文「另把这两种写进 admission.py 文件头认不出的那一段」）。J2-a 在 r1 判决时点的是「旧代码认不出」，不是「新代码也认不出」——判决没有把这两件事混，站得住。**推翻条件**：改后仓上 U1/U2 若仍 BROKEN 才会推翻 Y5 对症；探针显示 holds，未推翻。 |
| J2-b（排除法减得太少，`mutations.tsv`／`src/bin/*.rs`／共用测试模块） | **站得住，且这一条恰是判决自己承认「代价写进文件头、这一批不做」的那部分——不是被略过，是被明写成不做** | 攻方报告 O1–O4 四格（`defs-gatebatch-m2-r1-opus-output.md:129-144`）；改后仓复跑 `probe-j2-new.log`：O1（`mutations.tsv` 加一行）holds（无用例读到，指纹不变，符合期望）、O3（harness 的 `src/bin/*.rs` 加函数）holds；**O2、O4 仍 BROKEN**（O2「`tests/common_tree_split/mod.rs` 加一个没人调的函数」四条全变；O4「`tests/common_admission/mod.rs` 加一个函数」三条变）。这与 Y6 的措辞完全对得上：Y6 原文「共用测试模块（`tests/common_*/`）要顺着模块图才减得准，这一批不做，代价写进文件头」——现仓 `admission.py:100` 原文「减得少的（改了照样让用例重跑）：几个测试目标共用的测试模块（`tests/common_*/` 这一类）要顺着模块图才减得准，这里不走模块图」（`grep -nF` 命中）。**判决没有略过它**：判决第三节 Y6 原句已经点名这两个探针对应的形状并写明代价。 |
| J3-a（构建输入按路径不按内容进指纹） | **站得住** | 攻方 E1–E3（`defs-gatebatch-m2-r1-opus-output.md:148-161`）；改后仓复跑 `probe-j3-new.log`：E1（`CARGO_TARGET_..._RUNNER` 指的脚本换内容）holds、指纹变（`6c4a6bb63717cc15… → f3f88c3651b3abe0…`）；E2（`RUSTC_WRAPPER` 换内容）holds；E3（`.cargo/config.toml` 的 `build.rustc` 换版本）holds。三格全部由 BROKEN（r1 判决时）→ holds（Y7 之后），Y7 对症。判决同一行还写「今天走到的可能性低」——这是限定语不是弱化判定，攻方与判决口径一致（`defs-gatebatch-m2-r1-opus-output.md:146` 同样写「今天能不能走到：低」）。 |

## 二、替被判出局的写法辩护

### J1-c「两趟并发只会假红」不改：辩方没找到反例，站得住

判决第 38 行「J1-c 不改，写进 54 号文件头『两趟并发只会假红』」。现仓 `54-layer0-replay.sh` 头部已加原文（`grep -nF` 命中）：「两趟 --full 同时跑同一条用例、同一个指纹时这里不加锁：后跑完的那一趟判红会删掉先跑完的那一趟刚写的绿标记，只会假红、不会假绿（崩溃验证员的定义里『另有 --full 在跑时不起』挡着这一种）。」

辩方现查了这句话能不能被推翻：
- 写标记的路径是原子的：`admission.py` 的 `write_crash_case_marker`（现仓 `research/scripts/admission.py:1613` 起）用 `tempfile.mkstemp` 建同目录临时文件、写完 `os.replace` 换上（`grep -nF` 命中该函数定义与 `os.replace(temporary_path, marker_path)` 两行），不存在「写一半」被另一趟读到的路径——没有部分写入导致误判绿的机会。
- 删除路径是按「用例键 + 起跑时的指纹」删（`54-layer0-replay.sh:187` `delete_crash_case_marker()`，`grep -nF` 命中定义行），删掉的只可能是同一条用例、同一批输入的标记；不会跨用例、跨指纹误删出一个「本不该绿」的标记变绿——它只删，不写，删除动作本身不能把红判成绿。
- 唯一站得住的顾虑不是「假绿」而是「非确定性」：若同一份代码在同一批输入上跑两次结果不同（flaky），并发的两趟一趟绿一趟红，标记最终留哪个纯看时序；但这时两趟的判定都是「真的」（各自那一次真实跑出来的结果），不是标记机制伪造出来的假绿——这已经超出 J1-c 问的「假复用」范畴，是被测代码本身的确定性问题，判决没有必要在这里去接。

**辩方结论**：J1-c 的「只会假红」在标记读写这一层站得住，没找到能把它推回「会假绿」的路径。**推翻条件**：找到一处标记写入不是走 `write_crash_case_marker`（例如某处直接 `open(..., "w")` 覆盖标记文件）；`grep -n 'marker_path.*"w"' research/scripts/admission.py` 现查为空，未推翻。

### J4-c「危害小」的误拒被判「打中、危害小」而不改：站得住，Y4 只吃掉了 R1 那一半

判决 J4-c 行「打中、危害小：`-- --ignored --list`、`--include-ignored --exact <一条快用例>` 被拒」。这一条在改法里被 Y4 部分吃掉：Y4 原文「带 `--list` 的不算重型；`--include-ignored --exact <快用例>` 那一种接受误拒，写进闸的文件头」——即 Y4 自己承认只修 R1（`--list`），R2（`--include-ignored --exact` 单条快用例）**明确选择不修、只接受**。改后仓复跑 `probe-j4-new.log`：R1 holds（`--ignored --list` 现在退 0，放行）；**R2 仍 BROKEN**（期望「应当放行（退 0）」，实测「退 2」，仍被误拒）。这与 Y4 的字面完全一致——判决没有把「打中」误判成「已修」，Y4 本来就只打算修一半，另一半的代价（继续误拒）写进了闸文件头。辩方现查确实做到了：`lib_heavy_tests.py:40` 与 `heavy-test-guard.sh:83` 原文都是「接受的误拒：`--include-ignored --exact <同一目标里的快用例>` 照拒（过滤之后剩下哪几条，执行前判不出）；别名展开之后不是 test 的照拒。」（`grep -nF` 两处都命中）——与 Y4 的承诺逐字对得上，不是「少写」。**J4-c 站得住，Y4 对症，没有文字缺口**（本节此前一版稿曾误判「没找到文档」，现查后撤销该说法）。

## 三、判决略过了什么

- **没有略过任何一格「打中」**：攻方报告第 54–69 行的判定一览表共 11 行（J1-a/b/c、J1 续跑、J2-a/b、J3-a/其余/worktree、J4-a/b/c），除「J1 续跑目录串用……没打中」与「J3 CARGO_BUILD_JOBS……没打中」两行外，其余每一行都能在判决第二节或第三节找到对应格与对应 Y；两条「没打中」的行本就不该有 Y，判决也确实没给。
- **略过的是引文的可核性，不是结论**：J1-b 那一处「admission.py:168」的错配（见上）是判决压缩时留下的唯一一处查不实的行内引文；J2-a/J2-b/J3-a 三行判决本身没给行内引文（只写「攻方」），审到攻方报告与探针即止，没有第二层可查的错配。
- **没有略过「代价」**：J2-b（Y6）、J4-a 里 B6/B7（Y3）两处遗留缺口，判决与改法都各自写明「这一批不做」「接受」，不是无声消失。

## 四、K2：改法的证红办法证得了什么、证不了什么

**证红办法**：改法交回报告说的是「新自证格在旧代码上跑必须红」+「攻方探针在改后仓复跑」。这两句各自证到的东西不同：

1. **「新自证格在旧代码上跑必须红」**证的是：新写的检查代码语法正确、逻辑上确实能识别出旧代码的错误行为（show-me-test 的最低要求，参见 `.claude/rules/implementation-workflow.md`「三步」表「1 写代码」一行）。它**证不了**新检查在真实场景（真编译、真 cargo、真崩溃枚举用例）下的行为——因为这一格测的是旧代码，不是新代码在新场景下的正确性。
2. **「攻方探针在改后仓复跑」**证的是：r1 攻方造出的每一种具体形状（A、O、E、B、I、R 系列），在改后仓上现在判什么。辩方现查逐条复跑结果：
   - J1（A1–A4）：`probe-j1-new.log` 全 holds，**但跑在一个合成的临时小仓**（`.claude/stage-under-test/` 拷贝、假 cargo，与 `admission.py --selftest` 内嵌的 `build_selftest_layer0_stage_repository`/`FAKE_CARGO_FOR_STAGE` 同一套机制）——这里的 A1–A4 与真实的 `layer0-first-stream` 等四条用例、真实的 `cargo test --release --include-ignored` 编译执行完全无关，只验证 admission.py 的**判定逻辑**在合成日志上算得对。
   - J2（U1/U2/O1–O4）：`probe-j2-new.log` 跑在**真仓拷贝**上，用真实的 `git ls-files` 与文件改动，O2、O4 **仍 BROKEN**（`admission.py:100` 已把这两种写进「减得少的」清单，是文档化的已知缺口，不是「没修好」）。
   - J3（E1–E8）：`probe-j3-new.log` 跑在真仓拷贝上，全部 holds（Y7 对症）。
   - J4（B1–B13、I0–I2、R1/R2）：`probe-j4-new.log`／`probe-j4-repo.log` 里的命令是**真实的 cargo/nextest/mutants 等命令行**，但每条命令在被闸拦截之后就没有继续跑——闸是 hook 里的 JSON 判定（喂 JSON 看退出码），凡是「✗ 重型测试被拒」的格子，cargo 从未真正被执行；只有少数 holds/BROKEN 是靠命令真的跑起来才能测出来的（B6、B7 让命令真的跑到底才知道退 0）。B6、B7 仍 BROKEN——`lib_heavy_tests.py:41` 已把「拷走改名、又不点名用例函数的测试二进制」列为「看不见的」，交给看门狗在进程层兜底，但看门狗那一半「推的，没量」（`lib_heavy_tests.py:42-43` 原文），这个兜底本身没有一次被真的量过。B6/B7 的探针命令本身没有点名用例函数（`probe_j4_hook.py:63-64` 原文 `f"cp {BIN} {X}/rc && {MC} {X}/rc --ignored"`、`f"{MC} find target/release/deps -name '{C561}-*' -type f -executable -exec {{}} --ignored ';'"`，都只带 `--ignored`，不带任何函数名参数）——这恰好是 Y3 规则要求的「参数里有登记的用例函数名」永远碰不到的那一类，Y3 本身在 r1 攻方报告里就写明「只点名测试目标、不点名函数的写法照样漏」（`defs-gatebatch-m2-r1-opus-output.md:216` 原文 F8 行）。**结论**：B6、B7 是「规格明写不做」（文档化、且是 F8 提出时自己就承认的已知局限），不是「本该修好而没修好」；但它覆盖的恰恰是最省事的绕过写法（直接拷走跑，不用带任何参数），文档化不等于低危。
   - R2：BROKEN 但方向是「多拒」，与 Y4「接受误拒」一致（第二节已核实）。
3. **54 号自己的场景自证（`admission.py --selftest` 里嵌的 4 个新格）**：辩方现查代码确认，这 4 格（范围问「不带 SINGLEFS_GATE_FULL=1」在改登记表/改54号/改准入模块三种改动下的行为，加「一个改动都没有」的对照格，见 `admission.py:3113` 起、`scope_cells` 定义在 `admission.py:3128`）全部经 `quick_tier_without_forcing()`（`admission.py:3120`）调用，这个函数只跑「快档」（`layer0_tier == quick` 那条路径），快档在真代码路径上根本不起 `cargo test`——它答的是「碰没碰」，不是「跑没跑对」；即使起 `--full`，这条自证函数用的工具链也是 `FAKE_CARGO_FOR_STAGE`（`admission.py:2913` 定义，`admission.py:2979` `write_executable(os.path.join(tools, "cargo"), FAKE_CARGO_FOR_STAGE)` 装进 PATH）。**这 4 格全程没有一次调用真 cargo，也没有一次调用真 `.claude/gate.d/54-layer0-replay.sh --full` 在真仓上跑完一条崩溃枚举用例**——K2 问句里「真 --full 一次没跑」在这 4 格上现查为真。
4. **推翻条件**：若能在 `/tmp/claude-1000/gate-batch-m2-r1-fixes/logs/` 之外找到一份用真 cargo、真 `--full`、对着 `crates/singlefs-harness` 的四条真实崩溃枚举用例跑过的日志（`test result:` 与计数行落在真实文件里，不是 `FAKE_CARGO_FOR_STAGE` 打的合成行），此条撤销；辩方在给定的日志目录与探针目录里 `grep -rl "FAKE_CARGO\|synthetic\|合成"` 与人工核对文件名，没找到这样一份。

## 五、没做什么

- 不碰 K1（修补本身的语义正确性）、K3（`stage-inputs.tsv` 第 36/37 行判法字段）、K4（指纹变化的处置代价）——这三格归另两条腿。
- 不跑重型测试（54/55/57/59/87 号、`gate.sh`、名字带 layer0 的目标、全量 `cargo test`）；本报告全部结论基于现查代码、现查引文行号、与已落盘的探针日志（`/tmp/claude-1000/gate-batch-m2-r1-fixes/logs/`，只读）复读，没有自己起新的重跑。
- 没有对 J4-c 遗留缺口另开探针目录——现查已在仓里现有的文件头与探针日志里核实完，不需要新造探针；本轮探针目录 `research/prompts/defs-gatebatch-m2-r2-defense-model/` 因此空着，没有新产物。
- 没有对「看门狗兜 B6/B7」这条兜底做实测（`lib_heavy_tests.py:42-43` 自己写的是「推的，没量」）——这是它自己承认的未验证项，不属于本轮 K5/K2 的射程（K1 的射程），辩方只如实转述。
- 没有重新计算 J2-b 的「2.3 天」「2.8 天」代价数字，判决与攻方都已标「推的」，K4 腿会重估，这里不重复。

## 六、产出与自证

- 探针目录 `research/prompts/defs-gatebatch-m2-r2-defense-model/`：本轮没有新造探针（全部结论基于现查与已有探针日志），目录留空，无 `SHA256SUMS`／`rerun.sh`。
- 本报告 `research/prompts/defs-gatebatch-m2-r2-defense-output.md`，`sha256sum` 见交回消息。
