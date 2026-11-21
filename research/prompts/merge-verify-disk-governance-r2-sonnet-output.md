# merge-verify-disk-governance-r2 辩方腿报告（sonnet，2026-11-19）

被复核的判决：`research/prompts/merge-verify-disk-governance-r1-main-verification.md`；本腿只答背景材料「问题」第 1 条。被判文件与开工快照 `research/prompts/merge-verify-disk-governance-r2-snapshot/judged-sha256.txt` 比：开工时 `sha256sum -c` 前六份 OK，第七份 `.claude/gate.d/break-switch-replay-files.tsv` FAILED（别的会话在改，不在本腿问题里，没追）。

## 复跑命令与模型文件

模型目录 `research/prompts/merge-verify-disk-governance-r2-sonnet-model/`，两份文件：

| 文件 | sha256 | 行数 |
|---|---|---|
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps` 那一段与它用的函数） | `761a15c1c0a0d1d12c78b5175576666fec48babda67e3559e95b6f7b1f308dad` | 427 |
| `tools/rerun-from-a-clean-work-directory.sh`（先删运行目录再调同一份 `rerun.sh`，拷自第一轮攻方腿的同名文件） | `0fa66b1880e7c2e072b667e09db029ae34fc8f49ad06f2c7aa063f38dbf4aad4` | 24 |

- 复跑命令写在文末「要主 agent 跑的命令」第 1 条；模型读被判脚本（sha256 写死 `75c1df5d5d8dc859a68241c9d4fae9adb3b23aa92530e75dfc7b81602fdf8d5c`，此刻主工作区那一份）里的挑目标程序与「build 包集合」程序，原样取出来跑，不改它们。
- 起头 `rerun.sh` 核被判脚本的 sha256，不等就退 2、什么都不跑。
- 探针的内存与时间：没量（子 agent 不跑），推的：七棵两包或一包的玩具树，没有外部依赖，最重的是 `cargo clippy` 与几趟 `cargo build`，合计推几分钟、内存推不到 2 GB；运行目录里每棵树各有一个编译目录，看完由主 agent 删。
- 预期读数（跑前写死，每格 `expected=` 里也有）：`name=summary matched=20 mismatched=0`；`observe` 的格只量不判，数在第三节。
- 第一轮攻方腿的模型（`research/prompts/merge-verify-disk-governance-r1-opus-model/`）在此刻的脚本上跑不了（它写死第一轮判的那一份 sha256 `c6badd7b…`）；那一份此刻在 `~/.local/share/singlefs/work/merge-verify-drop-build-step/batch7/base/scripts/gate/merge-and-verify.sh`，`sha256sum` 读出就是 `c6badd7bdfb5505053ed6cb89a8f4f2db577539f46753a95aff0bca9f8b00d2a`。那份 `batch7/base` 只有 `scripts/gate/merge-and-verify.sh` 一个文件（`ls` 读数），而第一轮模型还要从冻结副本里读 `rerun-failed-tests.py`、`lib_rebuilt_packages.py`，所以它也不能直接指到 `batch7/base` 去跑；判决 2、3 引的挑目标两格是第一轮那一版脚本的读数，现在的脚本已改，重放不出，只能引主 agent 当时的复跑。判决 1 的 cargo 格不依赖脚本版本，本腿模型的 `poly_*` 格在现在的脚本上重量同一批。

## 一、判定一览

「站得住」=我顺着依据现查，成立；「打穿」=找到反例走法；「推不出」=依据不够得着结论。读数没有主 agent 复跑的一律「推的，待主 agent 跑」。

| 判决条目 | 辩方判定 | 证据（要点） |
|---|---|---|
| 1 去掉 build 打穿 | 站得住（机制）；带三条限定 | 单态化那一臂的格名与判决引的相符；限定：判决引的原样输出行没有落进任何入库文件；链接那一臂的 clippy 格是玩具树自己的 lint；「退 0、合进主工作区」是三格拼出来的推的（模型没整条跑被判脚本）；本仓里没有现成的造出这种错的写法 |
| 2 只改 `[[bin]]` 的 path 漏挑集成测试 | 站得住；同样打中别的臂 | 判决引的两格在模型里是 `items_check alpha_items_binpath` 与 `alpha_binpath_cli_test`；同一形状在改前与改后对「只改 bin 源码」「只改库源码」同样成立（脚本只落到那一个目标），判决与改法 2 没写这一层 |
| 3 新加 `[[test]]` 同名顶掉 | 站得住 | `alpha_shadow_list`、`alpha_shadow_picked_list` 是真 cargo 的 `--list`；改前整份退 3 不中；没找到别的入口造出同一个结果 |
| 4 残留说法（正推腿的几处） | 站得住 | 恢复 build 之后两处「链接留给合入后验证」又成立；`Cargo.toml` 第 30 行已是现状句 |
| 5 广谱变异键整张变 | 站得住（只核了那一行还在） | `scripts/mutation/mutation-prune.py:1049` 原句在；没重算两个哈希 |
| 6 收尾守卫 | 推不出「两条腿都没打中」 | 只有攻方腿试过打它；正推腿只核文件头与代码一致、没跑复刻；我读代码也没找到反例，按规则要再一次独立抽样 |
| 6 删上一次尝试的快照 | 站得住；判决写的「核不动」那一格我核动了 | 服务落结局、定重排之前先确认第二台与本机的清理；会重排的结局只有四种 |
| 6 Checking 行当重编证据 | 站得住（推的）；只有一条腿试过 | 与判决 1 同一个「检查不等于编译」形状，但只在编译目录里有比快照新的产物时才通，快照每趟新拷，不可达 |
| 改法 1 恢复 build、关调试信息 | 站得住（拦得住判决 1 那种错） | 判决 1 的单态化与链接两格，`*_build_without_debuginfo` 就是改法 1 的那一臂；包集合算法对依赖方是传递闭包；留下的同形缺口不是调试信息引起的，且改前同样有 |

打穿 0 处。推不出 1 处（收尾守卫的「两条腿」）。判决 2 与改法 2 的差（第二节）、判决 1 的三条限定、改法 1 的两条同形缺口（第三节）交主 agent 定要不要写进判决。

## 二、逐条展开

### 判决 1 去掉 build 打穿：站得住（机制）

- 判决引的格都在第一轮攻方腿的模型里：单态化那一臂 clippy 格是「run_in "$base/$variant" "poly_${variant}_clippy" zero cargo clippy --offline --all-targets -- -D warnings」（`research/prompts/merge-verify-disk-governance-r1-opus-model/tools/rerun.sh:213`），整个工作区带 `--all-targets` 跑，依赖方 `poly-user` 的 bin 也被它类型检查到，所以「clippy 过」对这一格是最宽的读法，不是只检了改到的包。build 那两格是「run_in "$base/$variant" "poly_${variant}_build_without_debuginfo" nonzero env CARGO_PROFILE_DEV_DEBUG=0 cargo build --offline --all-targets -p poly-core -p poly-user」（同文件 `:221`）。
- 判决写的复跑结局：「`name=summary matched=21 mismatched=1`」（`research/prompts/merge-verify-disk-governance-r1-main-verification.md:16`）。这一行读数我没法重现：那一趟的运行目录已删、输出没有落进任何入库文件（`grep -rln 'poly_after-postmono_build' research ~/.local/share/singlefs/work/merge-verify-disk-governance-r1-main` 只命中判决与攻方腿报告，两份里只有格名与预期、没有输出行）。`.claude/rules/evidence-discipline.md@引产物就整行抄` 要带文件名；这里读数只能写「推的，待主 agent 跑」，本腿模型 `poly_after_clippy_whole_workspace`、`poly_after_picked_test`、`poly_after_build_without_debuginfo` 在现在的脚本上重量同一批 cargo 格，复跑命令是文末第 1 条（推的，待主 agent 跑）。
- 限定一：链接那一臂。判决自己写「链接那一臂的 clippy 格没对上」（`research/prompts/merge-verify-disk-governance-r1-main-verification.md:21`）。病根在玩具树：不导出符号时 `$attribute` 是空的，函数前留一个空行，贴在文档注释后面，触发 clippy 的「文档注释后空一行」lint（模型里 `[[ "$2" == 1 ]] && attribute='#[no_mangle]'`，`research/prompts/merge-verify-disk-governance-r1-opus-model/tools/rerun.sh:148`；文档注释后紧跟 `$attribute` 一行，`:159`）。所以链接那一臂的 `clippy exit=101` 不是「clippy 见到链接错」，是玩具造的无关红；这一臂只有 build 那两格（链接失败）是真信号。判决已降级处理，我同意。
- 限定二：「合入后验证退 0、合进主工作区」是几格拼出来的推：模型没有整条起被判脚本，没跑验证包那一步与重编核对。本仓里验证包那一步只在挑出的目标是验证包目标时才起（脚本文件头 `scripts/gate/merge-and-verify.sh:71` 一带的测试那一步只编挑出的目标），所以这条推在「依赖方的坏代码不在被挑目标的依赖闭包里」时成立；闭包里的依赖库会被测试那一步一并编到、单态化错照样报（推的，按依赖库照编译模式出代码的规矩，没单独量）。模型的单态化格把坏实例放在依赖方的 bin（`poly-user/src/main.rs`）里，正是被挑目标闭包之外的那一类。
- 限定三：本仓里造得出但没出现过：攻方腿 grep `const {`、`#[no_mangle]` 没找到现成的写法（`research/prompts/merge-verify-disk-governance-r1-opus-output.md:59`、`:65`）。判决写「坐实」坐实的是机制，不是「漏过一次」。
- 站得住的理由：改后合入后验证（恢复 build 之前）对依赖方的 bin、example、别的测试目标，既不链接也不单态化；check.sh 的 `cargo build --all-targets` 在提交时才兜（判决 1 自己写了这一点）；主工作区在这期间是编不过的（`.claude/skills/merge-verify/SKILL.md` 验证段第 2 步列的几条是检查脚本、门禁与崩溃放量带 `--item`、变异重放、命名检查、缩域重判，没有 build，`.claude/skills/merge-verify/SKILL.md:32-37`；第 3 步才合进主工作区）。
- 为被判出局的一方（去掉 build）找到的、站得住的理由，如实列：本仓里没有现成的造出这种错的写法；提交时有兜底；省盘。三条里前两条判决已经写了（后移到提交、本仓没出现），第三条（省盘）改法 1 用关调试信息拿回了大半，没量。推翻判决 1 的现象：`poly_after-postmono_clippy` 在主 agent 复跑里退非 0（clippy 在这一版里求了那个常量），或 `poly_after-postmono_build` 退 0。

### 判决 2 只改 `[[bin]]` 的 path 漏挑集成测试：站得住；同样打中别的臂

- 判决引的两格：`items_check alpha_items_binpath …`（模型 `:291`）与「run_in "$base/binpath" alpha_binpath_cli_test nonzero cargo test --offline --no-fail-fast -p alpha --test cli」（模型 `:296`）。第一格只用被判脚本里原样取出的挑目标程序，不起 cargo；第二格是真 cargo。两格合起来的结论（挑了 `--bin tool`、没挑起它的 `tests/cli.rs`、而 `tests/cli.rs` 红）成立。
- 同样打中别的臂：同一个形状不靠清单段也成立。被判脚本落「src/bin 下的根文件」时只落到那个 bin 目标：「if relative in targets and relative != rerun.DOC_TARGET:」（`scripts/gate/merge-and-verify.sh:2056`）下面直接 `return [relative]`；库源码一路同理，「if parts[0] == "src" and len(parts) >= 2:」（`scripts/gate/merge-and-verify.sh:2064`）只落到库目标一个根。文件头也写了：「src/ 下别的（不论是不是 .rs）→ 库目标」（`scripts/gate/merge-and-verify.sh:93`）。改前整份基线（`~/.local/share/singlefs/work/merge-verify-drop-build-step/base/scripts/gate/merge-and-verify.sh`，sha256 前 16 位 `8b580ee426327c0d`）对 bin 源码、库源码是同一段落法（那一份 `CARGO_BIN_EXE` 命中 0 次，`grep -c` 读数）。所以「起 bin 的集成测试没挑」「读库的集成测试没挑」是挑目标这一整套的设计，不是清单段新引入的；清单段新引入的只是：改前包的 Cargo.toml 一改就退 3，改后放成了挑目标。
- 改法 2（`[[bin]]` 段有差别时把同包集成测试一并挑上，现在是 `scripts/gate/merge-and-verify.sh:110` 与 `:2145`）只补了清单段这一个入口，bin 源码与库源码两个入口仍然只挑自己。两条路对同一类错（集成测试经 `CARGO_BIN_EXE_<名>` 起的是改了的 bin）给的覆盖不一致。这不推翻判决 2，是判决没写的一层，交主 agent 定要不要写进来。
- 复跑：模型里 `alpha_items_srcedit`、`alpha_srcedit_cli_test_goes_red`、`alpha_items_libedit`、`alpha_libedit_cli_test_goes_red`、`alpha_libedit_beta_test_goes_red` 五格；预期（推的，待主 agent 跑）：只改 bin 源码挑 `alpha:src/bin/tool.rs|--bin tool`、`--test cli` 红；只改库源码挑 `alpha:src/lib.rs|--lib`、`--test cli` 与 `--test beta` 都红。推翻「同一形状」：这五格里挑出的项多了集成测试，或集成测试不红。
- 顺带一个数（量的，`ls`、`grep -c` 现跑）：`crates/singlefs-verification/tests` 下 481 个 `.rs`、`Cargo.toml` 里 `required-features` 28 行（26 段 `[[bin]]`）、这个包里没有一个文件读 `CARGO_BIN_EXE`。改法 2 的「同包集成测试一并挑上」落到这个包就是挑上几百个与 bin 无关的测试目标；这一条属攻方腿「挑多了」那一问，我只数了数，没判。

### 判决 3 新加 `[[test]]` 同名顶掉自动认的目标：站得住

- 判决引的格是真 cargo 的 `--list`：「list_check "$base/shadow" alpha_shadow_list absent beta_guards_greeting -p alpha」（模型 `:298`）；对照格 `alpha_before_list` 在 `:297`，`alpha_shadow_picked_list`（挑出的 `--test beta` 里只有 `other_check`）在 `:299`。三格都是真 cargo，不是照 cargo 的规矩自己推。
- 同样打中别的臂？改前整份退 3，不中；改后新加了 `shadowed_automatic_targets` 比两侧目标集合（「shadowed = shadowed_automatic_targets(」，`scripts/gate/merge-and-verify.sh:2124`，开关 `manifest-shadowed-targets-unchecked` 在 `:2123`）。顶掉自动目标的入口只有清单里加同名段这一条（别的写法 cargo 直接报重复目标、clippy 就红），没有挑目标这一套的别的入口造得出同一个结果。
- 推翻它的现象：`alpha_shadow_list` 的 `hits` 不是 0，或 `alpha_before_list` 的 `hits` 不是 1。

### 判决 4 正推腿的几处残留说法：站得住

- 恢复 build 之后，「链接留给合入后验证」又成立：`.claude/agents/implementation-writer.md:32` 与 「clippy（点名改到的包；不写 build，链接留给合入后验证）」（`.claude/rules/implementation-workflow.md:60`）不用改；脚本现在跑 build，成功句与红句写「编了 N 个包」，build 按依赖方传递闭包点包（第三节）。
- 文件头「同一套判据」那一句现在带着限定：「与上游 .claude/singlefs-ai-sop/scripts/check.sh 同一套判据（test 与 build 的范围另照下面两段收窄）」（`scripts/gate/merge-and-verify.sh:69`）。
- `Cargo.toml` 第 30 行注释现在是「调试构建只留行号表：完整调试信息占每个测试二进制的 73%–94%，几百个测试二进制一起链时盘写不过来。」（`Cargo.toml:30`），没有「合入后验证一趟要链几百个」那半句。

### 判决 5 广谱变异键整张变：站得住（只核了那一行还在）

- 「lines.append(f"profile {canonical(parsed.get('profile', {}))}")」（`scripts/mutation/mutation-prune.py:1049`）原句仍在，整张 `[profile]` 进键；仓根 `Cargo.toml:31` 仍是 `[profile.dev]`。我没重算两个哈希，正推腿的 `657baa2d…` 对 `957ed70c…` 当线索。
- 判决「只让这一次整表现判一遍、不改结局」：我顺着看的是 `scripts/gate/admission.py:879` 的「PROFILE_SETTINGS_LEFT_OUT = frozenset(("debug", "incremental"))」与 `scripts/mutation/lib_mutation_units.py:110` 的「PROFILE_SETTINGS_NOT_AFFECTING_OUTPUT = frozenset(("debug", "incremental"))」，两处原句都在；不进键的口径不变，所以只有广谱变异这一条路会整张变。

### 判决 6 三块「两条腿都没打中」

先看「两条腿」这个数。判决写「两条腿都没打中」（`research/prompts/merge-verify-disk-governance-r1-main-verification.md:26`）。规则的判法是：「只有一条腿覆盖到的范围」要拿它的「没打中」当依据，得再派一条同立场的腿（`.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`）。这三块里正推腿做的是「文件头与代码对不对得上」，不是构造反例：它自己写「没有构造反例」（`research/prompts/merge-verify-disk-governance-r1-sonnet-output.md:221`）；攻方腿自己写「这条腿只有一次抽样，没打中的三块」（`research/prompts/merge-verify-disk-governance-r1-opus-output.md:186`）。所以三块都是一条腿的「没打中」，「两条腿」的写法不成立。下面逐块给我自己的读代码结果（也只是一次独立抽样，不替它们凑成「两次」）。

**6a 收尾守卫：推不出「两条腿」；我读代码没找到反例**

- 守卫原句：「if [[ "$BASHPID" != "$$" && "${MERGE_AND_VERIFY_BREAK:-}" != "finish-in-forked-shell" ]]; then trap - EXIT; exit "$rc"; fi」（`scripts/gate/merge-and-verify.sh:895`），在 `finish` 里 `rc=$?` 之后第一条语句（`:892`），EXIT、INT、TERM 的 trap 在主 shell 顶层装一次（`trap 'exit 143' TERM` 在 `:5231`）。主 shell 里 `BASHPID` 恒等于 `$$`，守卫只在 fork 窗口里的子 shell 起作用。我顺着想了三种让「该删的不删」的路：脚本被 `source` 进子 shell 跑（入口脚本照文件头「起法」是 `bash 脚本` 起的，没找到 source 的用法，推的）；`finish` 被 `$( … )` 或 `( … )` 调（全文件只有 `trap finish EXIT` 一处接线，`:5229`，另一处是只给自证演的直接调用 `:848`）；主 shell 在 `finish` 里被第二个 TERM 打断（`finish` 开头 `trap - EXIT`，TERM 的 trap 仍是 `exit 143`，会在删到一半时退出——改前同一句，攻方腿已记「不是这一轮改的」，`research/prompts/merge-verify-disk-governance-r1-opus-output.md:147`）。三种里只有最后一种是真的，而它与守卫无关、改前就有。
- 没做的：正推腿自己写的「复刻脚本我没跑」（`research/prompts/merge-verify-disk-governance-r1-sonnet-output.md:125`）这一格，到现在还是只有主 agent 的工具员跑过（2000 次里 862 次，背景材料里的说法，没有独立复跑）。我读了那两份复刻脚本（`~/.local/share/singlefs/work/merge-verify-drop-build-step/batch5/repro/race.sh` 与 `race-guarded-b.sh`）：后者的守卫是 `[[ "$BASHPID" != "$$" ]]`，与被判脚本 `:895` 的守卫少一个 `finish-in-forked-shell` 开关分句，判法同一个。复跑命令在文末第 3 条（读数推的，待主 agent 跑）。推翻 6a 的现象：复跑里带守卫的一版出现 `FINISH-IN <子 shell 号>` 行。

**6b 删上一次尝试的快照：站得住；「跨台核不动」我核动了**

- 判决写的「跨台那一格核不动」是因为攻方腿没读清「服务重排与第二台清理的先后」。我读了服务：落结局之前先做清理确认，没确认就不落结局、不重排：`finish_attempt` 里「peer_clean, peer_note = self.confirm_peer_cleanup(attempt)」（`task-server/task_server.py:5995`）与本机的 `stop_leftovers` 之后，没收干净就 `attempt.update(cleanup_pending=note, cleanup_retry_at=time.time() + CLEANUP_RETRY_SECONDS)`（`:6004`）并 `return`，收干净了才 `attempt.pop("cleanup_pending", None)`（`:6012`），其后才 `attempt.update(state="ended", …outcome=outcome…)`（`:6041`）、再 `settle_attempt` 定重排。文件头也写了：「状态机不变：清理没确认的照旧保持 starting/running、记 cleanup_pending。」（`task-server/task_server.py:140`）与「新的远端尝试另有 peer-cleanup.json：起远端命令之前先落 pending，确认进程停下并核目录没有持有者后才写 confirmed」（`:133`）。所以第二台上一次尝试还在跑时，这条任务还是 running、不会有下一次尝试去删它的快照。
- 会重排的结局只有四种：「RETRY_LIMITS = {"memory-cap-hit": MEMORY_CAP_RETRIES, "squeezed-out": SQUEEZE_RETRIES, "peer-run-failed": PEER_RUN_RETRIES, EXTERNAL_KILL_OUTCOME: EXTERNAL_KILL_RETRIES}」（`task-server/task_server_lib.py:2283`）。这也答了正推腿没核的那句「正常以退出码 1、3 收尾的一趟绝不会被重排」（`research/prompts/merge-verify-disk-governance-r1-sonnet-output.md:163`）：退 1、退 3 是命令自己的退出码，不在这四种里，`retry_decision` 对不在表里的结局直接 `RetryDecision(False, …)`，不重排；带 `KEEP_COPY=1` 留给验证段第 2 步的快照不会被删。
- 限度：我读的是服务代码的次序，没起服务；「升级前无此回执的尝试按旧协议处理」（`task-server/task_server.py:134` 同一段）这一支我没追。推翻 6b：主 agent 在服务里造出「第二台那一趟还活着、本机已起下一次尝试」的局面（交一条会被 `peer-run-failed` 的命令、断第二台的连接再看下一次尝试起没起），那一次尝试的日志里出现「删了上一次尝试留下的快照」而第二台那一趟仍有进程。

**6c Checking 行当重编证据：站得住（推的）；一条腿**

- 机制：clippy 日志里行首的「Checking」被换写成「Compiling」再交重编核对（「sed -E 's/^([[:space:]]*)Checking ([^[:space:]]+ v)/\1Compiling \2/' "$@" > "$output"」，`scripts/gate/merge-and-verify.sh:2735`；挂进核对日志的一行 `:2983`）。它与判决 1 是同一个形状：「检查过」不等于「编译过」。能放过的走法：测试那一步的产物被 cargo 判新鲜（没打 Compiling）、clippy 那一趟的检查产物旧而重查（打了 Checking），核对拿 clippy 的行替测试作保，测试其实跑在旧产物上。
- 够不着：旧产物要比快照里的源码「新」才被判新鲜；快照每趟新拷（文件头「快照」一段：不保留修改时刻，`scripts/gate/merge-and-verify.sh:29` 起），而编译目录里的产物都是这一趟之前写的，所以走不通；攻方腿同一个结论（`research/prompts/merge-verify-disk-governance-r1-opus-output.md:88`）。恢复 build 之后这条证据多了第三路（build 日志），只放宽、不收紧。
- 我没找到打中的走法，仍记一条腿。推翻 6c 的现象：编译目录里有一份比快照修改时刻新的、补丁改到的包的产物（例如调用方给的 `CARGO_TARGET_DIR` 被另一棵同路径的快照在这一趟拷完之后编过），而核对仍绿。

## 三、改法 1（恢复 build、build 那一条设 `CARGO_PROFILE_DEV_DEBUG=0`）：站得住

判决写的量过：「量过：模型里关掉调试信息的 build 两臂都红」（`research/prompts/merge-verify-disk-governance-r1-main-verification.md:30`）；它量的是「关掉调试信息还拦不拦得住」，没量盘，也没量脚本自己算出来的包集合。我补两处。

1. 拦不拦得住判决 1 那种错：
   - 代码这一条：「build_command=(env CARGO_PROFILE_DEV_DEBUG=0 "${CARGO_BUILT_COMMAND[@]}")」（`scripts/gate/merge-and-verify.sh:2849`），文件头写「只设在 build 这一条上，test、fmt、clippy 不设」（`scripts/gate/merge-and-verify.sh:137`）。
   - 关调试信息不改单态化与链接的结局：判决 1 两格的 `*_build_without_debuginfo` 与不关的 `*_build` 同退 101（模型 `:220`、`:221`）。模型 `poly_after_build_without_debuginfo`、`poly_after_build_default_debuginfo` 两格在带 `[profile.dev] debug = "line-tables-only"` 的玩具根上各跑一遍（预期都退非 0，推的，待主 agent 跑）；推翻：两格退码不同。
   - 两个模型没量过的缝：一是 `CARGO_PROFILE_DEV_DEBUG=0` 在仓根写了 `[profile.dev]` 的工作区里是不是真覆盖了清单（第一轮攻方模型的玩具根没有 `[profile.dev]`，脚本自证只记假 cargo 收到的环境变量）。模型 `debuginfo_flag_model` 起两棵同源码的树、各 `cargo build -v`，数 rustc 命令行里的 `debuginfo=`：带环境变量那棵应当没有 `debuginfo=1`，不带的应当有（预期，推的，待主 agent 跑）。二是下面第三点。
2. 脚本自己算出来的包集合是不是依赖方：
   - 包集合：「names = sorted({package_name(directory) for directory in dependents_of(packages) if directory in members})」（`scripts/gate/merge-and-verify.sh:3493`），`dependents_of` 取改到的包连同传递依赖它们的包（`scripts/gate/merge-and-verify.sh:3483`）。模型 `poly_build_packages_path` 把被判脚本里的这一段原样取出，喂第一轮的单态化补丁，预期 `poly-core;poly-user`（推的，待主 agent 跑）；再用这个集合起 `env CARGO_PROFILE_DEV_DEBUG=0 cargo build --offline --all-targets -p …`，改前绿、改后红（`poly_before_build_without_debuginfo`、`poly_after_build_without_debuginfo`）。
   - 同形缺口，不是调试信息引起的：依赖边只认写了 `path` 的：「if isinstance(entry, dict) and isinstance(entry.get("path"), str):」（`scripts/gate/merge-and-verify.sh:3401`）。依赖写成 `poly-core = { workspace = true }`（根里 `[workspace.dependencies]` 给路径）时，依赖方不在包集合里，判决 1 的单态化错同样漏过。模型 `poly_build_packages_workspace_inherit`（预期只有 `poly-core`）与 `poly_inherit_dependent_build_would_fail`（把依赖方补进 `-p` 之后红）演这一格。本仓今天没有这种写法（`grep -n 'workspace = true\|workspace.dependencies\|\.workspace'` 在根 `Cargo.toml`、`crates/*/Cargo.toml`、`gpu-verification-server/Cargo.toml` 三处 0 命中，量的），改前整份基线的包集合同一段算法，所以是潜在的、不分臂的缺口，不推翻改法 1；以后有人往仓里加 `[workspace.dependencies]` 时它会静默放过。
   - 同形缺口二：build 不带特性（`cargo build --offline --all-targets`，文件头 `:130`），挂在特性后面的目标不编；提交时 check.sh 同一条也不带，两头一样，不分臂。
3. build 与测试用不同的 profile 的后果（只量）：`CARGO_PROFILE_DEV_DEBUG=0` 与仓根 `line-tables-only` 是两套编译参数。模型 `profile_split_model` 在同一个编译目录里先起关调试信息的 build、再起默认的 `cargo test -p poly-core --lib`，数 `Compiling poly-core` 行：`split_compiling_poly_core_in_test` 读出 `count=0` 是共用、`count>=1` 是各编各的；`split_compiling_poly_core_in_second_build` 同理量再起 build 是否又重编（读数推的，待主 agent 跑）。这一条答攻方腿的「build 与测试那一步的产物、指纹、重编核对会不会打架」前半，我只给数，不判。

结论：就背景材料问的那一句（关掉调试信息的 build 拦不拦得住判决 1 那种错）站得住；省盘这一半没量，判决「交用户的」第二条已写明。推翻：`poly_after_build_without_debuginfo` 退 0，或 `debuginfo_flag_model` 里带环境变量那棵仍有 `debuginfo=1`（那时关调试信息这件事本身没生效，盘的账要重算）。

## 四、留给主 agent 定的

- 判决 1：要不要把「判决引的原样输出行」落进一份入库产物（第一轮那一趟的输出随运行目录删了、重放不出；以后主 agent 复跑判决要引的模型时，输出存进 `research/results/`），并把三条限定写进判决；我不替主 agent 改。
- 判决 2 与改法 2：改法 2 只补清单段入口，bin 源码、库源码两个入口仍只挑自己（第二节）；改法 2 落到 `singlefs-verification` 会挑上几百个与 bin 无关的集成测试（481 个 `.rs`，量的）。要不要把「挑哪些集成测试」改成只挑真读 `CARGO_BIN_EXE_<名>` 的，或写明这一层是设计、不修，是主 agent 的判断。
- 判决 6：「两条腿」改写成「攻方腿一条腿」，并按规则对收尾守卫与 Checking 行再取一次独立抽样（本轮攻方腿的四问里只有跨台一格，守卫与 Checking 行不在其中）。

## 要主 agent 跑的命令

1. 本腿模型整份复跑（第三节、第二节判决 1、2 的同形读数、第三节改法 1 的几格都从这一趟取）：

   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-sonnet -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-sonnet/model-run /home/fy5090/code/singlefs/research/prompts/merge-verify-disk-governance-r2-sonnet-model`

   - 在哪个目录起：`/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-sonnet`（submit 的 `--cwd`）。
   - 跑完看：末行 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；`task-server/task log <号> --all` 里 `name=summary matched=20 mismatched=0`；`name=split_compiling_poly_core_in_test count=`、`name=split_compiling_poly_core_in_second_build count=`、`name=flags_debuginfo_with_env values=`、`name=flags_debuginfo_default values=` 四行是只量不判的读数，原样抄进判决。`mismatched` 不是 0 的看 `verdict=mismatched` 那几行：`poly_build_packages_path` 的 `packages=` 不是 `poly-core;poly-user`，推翻第三节第 2 点；`poly_after_build_without_debuginfo` 退 0，推翻改法 1；`alpha_items_srcedit` 或 `alpha_items_libedit` 的 `items=` 里多出集成测试，推翻第二节判决 2「同样打中别的臂」；`exit=2` 且日志点名 sha256 的，是被判脚本换过了，不是模型的结论。
   - 跑完删运行目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-sonnet/model-run`（里面每棵玩具树各一个编译目录，在 `target` 底下，或在服务给的 `CARGO_TARGET_DIR` 底下）。

2. 收尾守卫的竞争复刻，带守卫的一版（第二节 6a；复刻脚本是前一批工具员写的，我读了没改）：

   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-sonnet -- bash /home/fy5090/.local/share/singlefs/work/merge-verify-drop-build-step/batch5/repro/race-guarded-b.sh /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-sonnet/guarded-hits.txt 2000`

   - 在哪个目录起：同上。
   - 跑完看：`TASK_SERVER_RESULT` 的 `exit=0`；再 `grep -c FINISH-IN <上面的 guarded-hits.txt>` 读数应当恰好 1（主 shell 收尾那一次）；多于 1 推翻 6a。里面每轮起一个 `sleep 300` 后台，脚本自己收，跑完 `ps` 看有没有留下 `sleep 300`。

3. 同上，不带守卫的对照（证明复刻装置确实造得出子 shell 跑 EXIT trap）：

   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-sonnet -- bash /home/fy5090/.local/share/singlefs/work/merge-verify-drop-build-step/batch5/repro/race.sh /home/fy5090/.local/share/singlefs/work/merge-verify-disk-governance-r2-sonnet/unguarded-hits.txt 2000`

   - 跑完看：`exit=0`；`grep -c FINISH-IN <unguarded-hits.txt>` 读数应当大于 1（背景材料的说法是 2000 次里 862 次）；等于 1 说明装置造不出竞争，第 2 条的「恰好 1」不构成证据。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没起过 cargo、没起服务：第二节与第三节里所有「预期」都是推的，待上面第 1 条出读数；第二节判决 1 的「退 101」等读数是判决转述的、我没法重现（运行目录已删、输出没入库）。
- 没重放第一轮攻方腿的模型：它写死第一轮那一版脚本的 sha256，而此刻的脚本是改后的；判决 2、3 引的两格没法在现在的脚本上重放。
- 没复核判决 5 的两个哈希；没复核 `.claude/gate.d/break-switch-replay-files.tsv`（开工 sha256 已与快照不符，别的会话在改）；没看 `.claude/hooks/lib_run_scope.py` 与 `.claude/skills/merge-verify/SKILL.md` 第 22 行的写回（不在问题第 1 条里）。
- 没攻修补本身（挑多了、`autotests = false`、`[lib]` 改名、目录形目标、重编核对读 build 的 Compiling 行认错包）：那是攻方腿的问题第 2 条，我只数了 `singlefs-verification` 的集成测试个数与 `[[bin]]` 段个数，顺手写在第二节。
- 草稿目录里没有仓副本、编译目录或库拷贝（`du -sh` 约 12K），交回前不用删；`model-run` 由主 agent 那一趟建、主 agent 删。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/merge-verify-disk-governance-r2-sonnet-output.md` 退 0，末行原样：

```
  ✓ 核了 25 处引文、16 处裸行号，对上 24 处，没判 17 处（其中裸行号没有引文可比 15 处；逐处列在上面）
```

没判的 17 处是裸行号（没有引文可比）与 `Cargo.toml`、`SKILL.md` 只写文件名认不出的几处，原文我用 `grep -nF` 与 `sed -n` 逐一现查过。
