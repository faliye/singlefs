# m3-close-gov-b17-r2 攻方腿（Opus）：批 17 治理第一轮判决之后四份修补的反例

日期：2026-10-24。攻击面：正文第三节 H1–H5。被判对象是主工作区现文（快照 `research/prompts/m3-close-gov-b17-r2-snapshot/judged-sha256.txt` 记的 14 份，开工时 `sha256sum -c` 15 行里 13 行 OK、`scripts/mutation/crates-mutation-rows.py` 一行 FAILED——那一份已被另一会话在快照之后又换过一版，与背景材料二节「另一会话又换过一版」一致，下面引它的行号按主工作区现文现查）。被判的命令一条都没执行：只调判法函数、把命令包成钩子 JSON 从 stdin 喂钩子、只算 python 闭包，不起 cargo、不碰任何判定库、不跑重型。不重复第一轮攻过的角度（G4-a 路径尾部、G1-g 判法指纹多收、G2 复跑副本、G3-a 第二台 GPU 不导回）。

## 复跑

模型目录 `research/prompts/m3-close-gov-b17-r2-opus-model/`，复跑命令（工作目录随意、`<新的草稿目录>` 给一个新的）：

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-gov-b17-r2-opus-model/tools/rerun.sh --repo <仓根> --frozen <仓根>/research/prompts/m3-close-gov-b17-r2-snapshot --work <新的草稿目录>
```

末尾打 `name=rerun_summary identical=6 differs=0`（六份产物 `out/*.txt` 逐行比）。全是 python 与钩子，不跑编译出来的代码；我这一趟经内存包装与直接起各跑一次，结果相同（主工作区此刻有别的会话的 crash-amplification 两台在跑，`ps` 见 `qemu`/`cargo`，没抢它们的卡与重型）。模型文件 sha256（写报告时现算）：

```
27c91d2b1fd0e6182b5403b3c1c7f6be91dcd8fe1863cb3c55bb0b74561d88f0  tools/rerun.sh
fd7607fa954b9d276c74eb7e468c9bfb64b8db8972188627db971ea5be6e645f  closure_by_file.py
e37599b8d12227801c1f032168945d6b9878bbe4196ee3feeb38f3d7a778d46e  closure_members.py
160b3801506b997c1deaf096513213fb16685c7d096a05bbc5eca4d509d2f3b0  crates_entries_by_file.py
297c4a03d168c13d4486cfb58a7a28c30342080edb7559c73137e2d590b13e94  refusal_probe.py
2135234cc6ed54ce492f41fe7958c3ecad782a443f79090583abc68e03a9185a  gpu_import_model.py
70ca615054920e58cd4b39e200362a164f3b81be988bcbe7ce39fd26bd70fe8c  probe_own_copy.py
a4a4d5d87cb3011fa130d3a7fba83fbc396a4dbb05ea513641ec0110d2e3764f  probe_copy_light.py
42a605f1ac31f0c1320746bcc8faa4f9d1ad017a9a25f1748ac6ba52cd16667c  out/closure-base.txt
e19b373f5767ce02cfb1c205363a6d474123088889e352e24fce1b43e6746984  out/crates-entries.txt
37348b6e17f4c4a99cb4fafa6b2d6d2d0d0624fdd79158403ad1f3a5dde80c57  out/h3-refusal.txt
910a5a578bf72a95d45c85d72f877002856f467e49a0b995a2c3737be3a864d1  out/h4-run.txt
4aa88169214c7a5108ece90194b116c7fe0d9ad84f725e2d9643585105a7dc6b  out/h2-run.txt
2cb2e899f64e8a109cc062bb6a39d85e15285b0be6eb785548e68bc47848204f  out/h2-light-run.txt
```

## 各格判定一览

| 格 | 判定 | 量的 / 推的 | 依据（详见各节） | 什么现象会推翻 |
|---|---|---|---|---|
| H1 文本修补 | 没打中：修补 1 五份逐处照判决改，与定案、第 11 条、收工闸、现文都对得上，没补出新口子 | 量的（字面比对） | implementation-workflow.md:31、verification.md:50、commit-stages.md、agent-common.md | 判决某一格要的改动在现文里没有、或改出判决没要的东西 |
| H2-a 符号链接旁的 preflight | 打中：指进本仓的入口符号链接被判作本仓那一份、不读正文，而 bash 按链接所在目录 source 旁边一份改过的 `preflight.sh`（起 checker 档 cargo），钩子退 0 | 量的（钩子退 0 + marker） | lib_heavy_tests.py:3368、3375、3387；replay.sh:99–101 | `own_repository_script` 对符号链接也核旁边的 `preflight.sh` 与本仓一致 |
| H2-b 逐字相同副本的轻用法被拒 | 打中（误拒一侧，是修补 2 自证里写明的 intent）：sha256 与本仓那一份相同的副本里起 `--list-items` / `--quick`，读正文照拒 | 量的 | h2-light-run.txt；lib_heavy_tests.py:3387 | 修补改成比 sha256（照 `--selftest` 那一条），不只比真路径 |
| H2-c 别的 KNOWN 名字的路径尾部口子 | 打中（老口子，非本次引入）：门禁阶段、`mutate.sh`、内存包装/capped 的路径尾部拷贝照 `judged_by_name=True` 不读正文 | 量的 | lib_heavy_tests.py:3382、3364；h2-run.txt | 这几类也改成比真路径 |
| H3-a 排除本身 | 没打中：排掉 `NOT_JUDGING_FILES` 两份后 465 个单元、0 个出自那两份，与崩溃判定库同一条 | 量的 | out/closure-base.txt、out/crates-entries.txt | — |
| H3-b 少作废 | 打中：`lib_function_units.py` 的切单元逻辑经 `executed_units` / `resolve_test` 进判法闭包、影响复用判定，排掉它的单元后改它不作废；修补把第一轮的「多作废」换成「少作废」 | key 不变量的，「改它改判对错」推的 | out/h3-refusal.txt；`.claude/kb/decisions/13-验证路线.md`@已定项 17 | 改 `lib_function_units` 的切单元不会改任何 crates 变异的结局或复用判定 |
| H3-c 自证分辨力 | 打中：自证新格对 `lib_verdict_library.py` 那一半是空测（`digest_text` 本来就不在 crates 判法闭包里，0 个单元），排不排都一样，分辨不出「排掉了」与「本来就没进闭包」 | 量的 | out/crates-entries.txt | `digest_text` 真进 crates 判法闭包 |
| H4 GPU 判定导回 | 打中（小）：第二台库是固定持久库（只追加、不压实）时，旧 `gpu_key` 与已不在清单的键的旧行使每次 `--import-results` 退 1（CPU 那一侧对旧行只是多一行、不退 1） | plan 量的；库持久推的 | out/h4-run.txt；broad-mutation.py:4322、4324、4454 | 第二台每次导入前的 `gpu-verdicts.tsv` 只有这一趟的行 |
| H5 第一轮方法 | 没打中：四格方法站得住；两点旁注（见 H5） | 量的 / 推的 | r1-main-verification、out/closure-base.txt | — |

## H1 修补 1 逐处对判决与现文

现文与快照 `judged-sha256.txt` 对得上（`implementation-workflow.md`、`verification.md`、`three-way-inference.md`、`commit-stages.md`、`agent-common.md` 五份 `sha256sum -c` 都 OK）。逐处核（都是量的字面比对，没打中）：

- G1 文件级：`.claude/rules/implementation-workflow.md:31`「文件级：`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份（算键的库自己与 `scripts/gate/lib_function_units.py`）按角色排，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键。」——补上了判决要的「管怎么存、怎么算键，不管判对错」，与定案 `.claude/kb/decisions/13-验证路线.md`@已定项 17 那一句逐字同。
- G1 边级：同一行「边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；登记在 `scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES`。」——「单元不进任何一份判定的键」不再挂到边级（第一轮 G1-a 打中的多写已去掉），「现在只有那一条」换成指登记位（第一轮 G1-f）。
- G1 自证只写一处：`.claude/rules/verification.md:50`「边级排除的三格自证在 `.claude/rules/implementation-workflow.md`「复用重型门禁的判定：按每条判定的单元键判」一节」——三格自证只在 implementation-workflow 写、verification 指过去（判决 G1「自证要求只写一处、另一处指过去」）；`grep -nF '边级排除要配三格自证' .claude/rules/verification.md` 0 命中，`… implementation-workflow.md` 1 命中（第 31 行）。
- G1 模块级四样：verification.md:50 现在也列了「import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头」（第一轮正推 2.1 报 verification 少这四样，已补）。verification.md:50 的括号此刻配平（`grep -o （ / ）` 各 4 个），没把这一处写破。
- G2：`.claude/rules/three-way-inference.md`@判决由主 agent 做，不由投票做 那一节现写「复跑那一步做完，副本里的仓副本与编译目录当场删；复跑的输出与模型目录拷贝留到这一题的下一轮判决写完」。副本当步删，只留输出与纯模型目录拷贝；纯模型目录（python + out/，无 `.git`、无 `Cargo.toml`+`crates`）收工闸不列（第一轮 G2-a 实测 `kind model-copy-only = None`，`.claude/hooks/completion-cleanup-guard.sh` 文件头「仓副本」「库拷贝」只认那三样）——与 `.claude/main-agent.md` 第 11 条「用完就删」不再顶牛。没打中。
- G3：commit-stages 第 2b 步 `gpu-driver-error` 出路改成「查日志、修环境再 `--item <序号> --rerun`」（第一轮 G3-b），补了哨兵退 1（G3-c：「退 1 里有一种是 GPU 那一趟的复用哨兵判红」）、清场退 78 的几种原因、「在有卡的机器上补 GPU 趟」。与 `scripts/mutation/broad-mutation.py` 现文（见 H4 引的几处）对得上。
- 定义级：`.claude/agent-common.md`「不做」一节那一行补了「与入口 `scripts/experiment-apparatus/replay.sh` 的不带参数、点名实验号、`--admission-only`；`--list-items` 与登记为 light 的单独 `--selftest` 不算」，与 `.claude/rules/implementation-workflow.md`@checker 档只在提交时跑，harness 随时跑 一节、`lib_heavy_tests.py` 的 `classify`（replay.sh 判成 replay-all-stage、`--list-items` 交 None）一致，不矛盾。

## H2 修补 2 `own_repository_script` 的认法

修补 2 把放行靠参数的三份入口（`OWN_COPY_ONLY_SCRIPTS`）改成只认解到本仓那一份（`.claude/hooks/lib_heavy_tests.py:3387`「if name in OWN_COPY_ONLY_SCRIPTS and not break_is_set("judged-by-name-path-tail"):」后调 `own_repository_script`，`:3375`「return resolved is not None and os.path.realpath(resolved) == os.path.realpath(os.path.join(HOOK_REPOSITORY, location))」）。喂法：`probe_own_copy.py` 把每条命令包成钩子 JSON（`agent_type` 写 implementation-writer）喂 `.claude/hooks/heavy-test-guard.sh`，另调 `judged_by_name`；被判命令一条都没执行，造的树都在 `--work` 下。

### H2-a（打中，量的）：指进本仓的入口符号链接，旁边一份改过的 `preflight.sh`

`own_repository_script` 用 `os.path.realpath` 比，符号链接「按指到的算」判成本仓那一份（修补 2 自证里专门有一格「指到本仓那一份的符号链接按指到的算（replay.sh）」要它 True）。out/h2-run.txt 原样：`case=linked-replay-list-items judged_by_name=True guard_exit=0`、`case=linked-crash-point-replay-quick judged_by_name=True guard_exit=0`、`case=linked-crash-amplification-list-items judged_by_name=True guard_exit=0`——三份入口的符号链接都判作本仓那一份、不读正文。

但 `scripts/experiment-apparatus/replay.sh:99`「source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"」（`crash-point-replay.sh:271`、`crash-amplification.sh:346` 同形）里的 `${BASH_SOURCE[0]}` 是链接的路径，`dirname` 取的是链接所在那棵树，`replay.sh:101`「cd "$(dirname "$0")/../.."」也按链接所在目录 cd——于是 bash 跑之前 source 的是链接旁边那一份 `preflight.sh`，不是本仓的。造一棵树：三份入口是指向本仓那一份的符号链接，`<树>/.claude/scripts/preflight.sh` 换成一份 `preflight() { … }` 里起 checker 档 cargo 的。`research/prompts/m3-close-gov-b17-r2-opus-model/out/` 没留这一格的 marker（marker 路径随 `--work` 变、不进比对），我另造了一棵树实跑两条 `--list-items` 证实（marker 两行各写 `sourced-by=<work>/…/replay.sh preflight=<work>/…/../../.claude/scripts/preflight.sh`），确认被 source 的是链接旁边那一份。

照字面做会做错：攻击者（或照共用约束 rsync 进草稿目录又把入口替成符号链接省空间的实现员）在一棵树里放三份指向本仓入口的符号链接、改掉旁边的 `.claude/scripts/preflight.sh`，起 `bash <树>/scripts/experiment-apparatus/replay.sh --list-items`——`judged_by_name` 认作本仓那一份、`heavy-test-guard.sh:818`「if heavy_tests.judged_by_name(command.words[0], command.directory):」后 `continue` 不读正文，钩子退 0，而 bash 真正 source 的是改过的 `preflight.sh`。

四句：分辨「副本按内容判」与「副本按真实路径判」两读法；钩子看得到链接与旁边的 `preflight.sh`（它们都在盘上）；满足的分句是 `own_repository_script` 的 `realpath(resolved) == realpath(本仓那一份)`；改法（只在我的读法与草稿上推过、被攻过零轮）：(1) `judged_by_name` 对 `OWN_COPY_ONLY_SCRIPTS` 的符号链接，除真实路径外再核链接所在目录往上两层的 `.claude/scripts/preflight.sh` 与本仓那一份 `sha256` 相同——修这一格；(2) 干脆不认符号链接（只认命令词真落在本仓那一份路径下），与 `--selftest` 那条「副本里那份脚本与主工作区那一份 sha256 相同」一个口径——修这一格，但把符号链接的本仓轻用法也拒了。推翻条件：`own_repository_script` 对符号链接也核旁边 `preflight.sh` 一致，或入口不再 source `dirname(BASH_SOURCE)` 的 `preflight.sh`。

### H2-b（打中，误拒一侧，量的）：sha256 与本仓那一份相同的副本里起轻用法照拒

修补 2 用真路径认、不比 sha256。`probe_copy_light.py` 先打三份入口在本仓与副本里逐字节相同（out/h2-light-run.txt：`name=same_bytes file=scripts/experiment-apparatus/replay.sh repo=ab8007103c090ba8 copy=ab8007103c090ba8`，另两份同），再喂实现员身份的轻用法。原样：`role=implementation-writer case=copy-replay-list-items judged_by_name=False guard_exit=2`（读正文撞上 replay.sh 里起 cargo 的那一行）、`case=copy-crash-point-replay-list-items … guard_exit=2`（`crash-point-replay.sh:1678`「cargo run …」）。本仓那一份照放：`case=repo-crash-point-replay-quick … guard_exit=0`。

就是说：实现员照共用约束把仓 rsync 进草稿目录（`crates`、`scripts`、`.claude`），在那份逐字节相同的副本里起 `bash scripts/crash/crash-point-replay.sh --quick` 这条轻型三条之一，钩子读正文、撞上 `cargo run` 拒掉。第一轮判决 G4「改成只认本仓那一份（与 `--selftest` 那条一样比 sha256 或比真路径）」给了两条路，修补 2 选了比真路径；而同一道闸对副本里的 `--selftest` 是比 sha256（`heavy-test-guard.sh` 文件头「副本里被起的那份脚本与主仓那一份 sha256 相同」）——同一份 sha256 相同的副本，`--selftest` 放行、`--quick` / `--list-items` 照拒，两条判据口径不一。这不是放过，是误拒本仓的轻用法，且与同闸的 `--selftest` 判据不一致。四句：分辨「比 sha256」与「比真路径」两改法；钩子看得到副本内容；满足的分句是 `own_repository_script` 真路径不等于本仓；改法：改成比 sha256（与 `--selftest` 同口径）——只在我的读法上推过、被攻过零轮。推翻条件：修补改成比 sha256，逐字节相同的副本放行。

### H2-c（打中，老口子，非本次引入）：别的 KNOWN 名字仍按路径尾部认

`OWN_COPY_ONLY_SCRIPTS` 只收那三份（`lib_heavy_tests.py:780`），别的 KNOWN 名字走 `judged_by_name` → `known_script_at_location`（`lib_heavy_tests.py:3382` 门禁阶段、`:3364`「return any(candidate == location or candidate.endswith("/" + location) …」只比路径尾部）。out/h2-run.txt：`case=copied-formal-quick judged_by_name=True guard_exit=0`、`copied-crates-mutation-quick … True guard_exit=0`、`copied-doc-text-list … True 0`、`copied-lkmm-version … True 0`、`copied-mutate-research-table … True 0`、`copied-memory-cap-wrapper … True 0`、`copied-capped-wrapper … True 0`——一棵别处树里路径尾部相同、正文起 checker 档 cargo 的门禁阶段拷贝，带 `--quick`（轻型 `classify_gate_stage` 交 None）或 `--list` / `--herd7-version`（靠参数放行），`judged_by_name` 照 True、不读正文、钩子退 0。这与第一轮 G4-a 讲的 replay.sh 是同一种「路径尾部相同就不读正文」，第一轮没延到门禁阶段，修补 2 也没延到——照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`@别的项目怎么做，是线索不是证据 这一节（打中之后先判是哪一种）：这是老口子，列在这里只为说明 `KNOWN_SCRIPT_LOCATIONS 里别的靠参数放行的名字有没有同一个口子`（H2 问句）——有，且修补 2 没碰它们。推翻条件：门禁阶段与包装脚本也改成比真路径。

## H3 修补 3 变异判法排掉算键库

`closure_by_file.py` 调驱动自己的 `judging_units_key`、`gpu_judging_key` 与 `crates-mutation-rows.py judging-units`（只算 python 闭包，不起 cargo）。

### H3-a（没打中）：排除正确实现

out/closure-base.txt：`name=crates_default exit=0 head=c63379ff…c9d0dccc 465`，按文件数单元里没有 `scripts/gate/lib_function_units.py` 与 `scripts/crash/lib_verdict_library.py` 的行；`name=crates_judging_files_included exit=0 head=c7bc2303…b47da40 961`，`…_units file=scripts/gate/lib_function_units.py units=496`。与第一轮判决「961 个单元里 496 个出自 lib_function_units.py」「排掉后 465」对得上（主 agent 现跑的 465 也对得上）。排法 `scripts/mutation/lib_mutation_units.py:774`「excluded = () if break_is("judging-files-included") else not_judging_files()」，名单 `:759 def not_judging_files()` 从 `scripts/crash/lib_verdict_library.py:88`「NOT_JUDGING_FILES = (…)」按路径载入，与崩溃判定库 `judging_unit_key` 同一条。

### H3-b（打中）：排掉之后「少作废」，与定案「不管判对错」的字面有缝

定案 `.claude/kb/decisions/13-验证路线.md`@已定项 17 把这两份算「管怎么存、怎么算键，不管判对错」。但 `scripts/mutation/lib_mutation_units.py` 的 `UnitKeys.mutated_units`、`resolve_test`、`executed_units` 是 crates 变异判法的入口（`scripts/mutation/crates-mutation-rows.py:514`–516 的 `JUDGING_ENTRIES`），它们经 `self.rust_file()`、`lfu.rust_closure_of` 调 `lib_function_units.py` 切单元、算闭包——`executed_units` 把覆盖 JSON 落进哪几个单元、`resolve_test` 点名测试是哪个单元，直接喂给 `baseline_reuse_problem`（`crates-mutation-rows.py:1378`「if executed_set_is_current(keys, coverage):」）定「基线复不复用」。切单元逻辑一变，复用判定就可能变，而它的单元被排掉、键不变 → 库里旧判定照沿用 → 少作废。

`refusal_probe.py` 把 `lib_function_units.py` 的 `RustFile.row`（闭包里有它，out/crates-entries.txt 里 `UnitKeys.mutated_units` 一个入口就 `lib_function_units=495`）改一处函数体，out/h3-refusal.txt：`case=lfu-dynamic default exit=0 out=c63379ff…c9d0dccc 465`——key 与没改时逐字相同（排掉了，改它不作废）；`case=lfu-dynamic judging-files-included exit=2`（照老样子进键就作废）。

第一轮 G1-g 把老代码（lfu 进键、961 单元）判成「多作废、偏保守」，判决要它照定案改；修补 3 改成排除之后，翻成了「少作废」——第一轮的方法只核了现状是多作废，没推到「修了会变少作废」。这一格是否算 bug 取决于「改 lfu 的切单元会不会真改某条 crates 变异的结局或复用判定」：改切单元确实改 `executed_set_is_current` 的答案（量的，key 不变而行为变），但真跑的 caught/missed 来自 cargo 运行时、与 lfu 无关，受影响的只有复用判定（推的，我没造出一条真翻面的变异）。定案「不管判对错」字面上只在「运行时结局」成立，对「复用判定」不完全成立。四句：分辨「lfu 进键（多作废）」与「排掉（少作废）」；系统看得到 lfu 的改动（同闭包），只是键里排掉了；满足的分句是定案「它们的单元不进任何一份判定的键」；改法——两处都是照定案字面做的，要么定案补一句「切单元逻辑变时手动重跑整表」（现规则无此句），要么把 `executed_units` / `resolve_test` 这几个真影响复用的入口从「判对错」里分出来另立键，这一条只在我的读法上推过、被攻过零轮。推翻条件：改 `lib_function_units` 的切单元不改任何 crates 变异的结局或复用判定。

### H3-c（打中）：自证新格对 `lib_verdict_library.py` 那一半是空测

修补 3 的自证新格（`crates-mutation-rows.py:6887`「selftest.check("改 NOT_JUDGING_FILES 那两份里任一份的一个函数：判法单元指纹不变，闭包里也没有那两份的单元",」）改 `lib_function_units.py` 的 `RustFile.row` 与 `scripts/crash/lib_verdict_library.py` 的 `digest_text` 各一处。但 out/crates-entries.txt 六个入口每一行都是 `lib_verdict_library=0`——`lib_verdict_library.py` 的 `digest_text` 本来就不在 crates 判法闭包里（它归崩溃判定库，不是 crates 变异判法调的）。所以自证改 `digest_text` 那一半：排不排、改不改，key 都不动，这一格对它恒真——分辨不出「排掉了」与「本来就没进闭包」。只有改 `RustFile.row`（495 单元、真在闭包里）那一半是有效的分辨。H3 问句「自证新格与开关能不能分辨『排掉了』与『本来就没进闭包』」——对 `lib_function_units` 能（out/h3-refusal.txt 的 `judging-files-included exit=2` 证了它进键会变），对 `lib_verdict_library` 不能。四句：分辨「测到排除」与「测了个本不在闭包的文件」；系统看得到闭包成员；满足的分句是自证断言「闭包里也没有那两份的单元」（对 `lib_verdict_library` 恒真、无信息）；改法——自证改一处真在 crates 闭包里、又出自 `lib_verdict_library` 的函数（若没有这样的函数，就写明 `lib_verdict_library` 对 crates 判法本就无贡献、这一半不是排除测试），只在我的读法上推过、被攻过零轮。推翻条件：`lib_verdict_library.py` 有函数进 crates 判法闭包。

## H4 修补 4 GPU 判定导回

`gpu_import_model.py` 调驱动自己的 `plan_gpu_import`（`scripts/mutation/broad-mutation.py:4278`，导入前的核，一行都不写），本机清单与 GPU 趟自己的键用替身（`read_only_judgement`、`GpuJudging` 换桩），不起 cargo、不碰任何库目录。两把键的核在 `broad-mutation.py:4322`「if row["key"] not in current_keys:」与 `:4324`「elif row["gpu_key"] != pass_key:」，`current_keys` 由 `:4312` 从本机此刻清单算、`pass_key` 由 `GpuJudging.pass_key`（`:3643`，含 `gpu_judging_key`）算。

### H4（打中，小）：持久的第二台库里旧行使每次导入退 1

`gpu-verdicts.tsv` 是只追加的库（`broad-mutation.py` 文件头「gpu-verdicts.tsv …只追加」、`--rerun` 那一行「追加新行，旧行留着」），没有压实。commit-stages 第 2b 步 ③ 的库目录是固定持久的 `~/.local/share/singlefs/broad-mutation-units/`，跨里程碑累积。out/h4-run.txt 两个情形：`scenario=fresh-peer attempt=first-import rows_read=2 fresh=2 rejected=0 … import_exit_would_be=0`（这一趟新造的库，干净，退 0）；`scenario=peer-with-history attempt=first-import rows_read=4 fresh=2 rejected=2 reasons=gpu-pass-changed,key-not-in-list conflicts=0 import_exit_would_be=1`——库里留着上一版驱动判的行（旧 `gpu_key`）与一条变异改了源码、键已不在清单的行，这两条触发 `key-not-in-list` / `gpu-pass-changed`，`broad-mutation.py:4454`「if conflicts or gpu_plan.rejected or gpu_plan.conflicts:」使整条导入退 1，尽管这一趟该导的两行导进去了。

对比 CPU 那一侧（`import_results` 原有逻辑）：它不核「键在不在本机此刻清单」，旧行只是多一行、不退 1（第一轮 G3 判决「别的源码的键与本机的不同，并进来只是多一行」）。修补 4 给 GPU 判定行加的这两把键核，使同一张持久库里的旧行从「多一行」变成「退 1」。退 1 是修补 4 自己的 intent（自证 ② 要 `bad.returncode == EXIT_INCONSISTENT`），但代价是：第二台的 GPU 库一旦跨趟复用（commit-stages 指的就是固定库），任何一次改过判法或源码之后的 `--import-results` 都退 1，逼操作者当 inconsistent 处置（`broad-mutation.py:4454` 之后那一句出路写「对齐之后在拿卡的那台上重跑它那一片、再重导；不重导也行，被拒的这几条本机之后补跑 GPU 趟时照现判」）。被拒行的处置本身对（照现判），但「退 1」让一条正常的复用库导入看起来像失败。量的：plan 的 rejected/conflicts 与退码由模型算出；推的：第二台库是持久/复用的（我没往第二台派活、没跑真两台工作流，`GpuJudging` 与清单用了桩，所以 `fresh-peer` 的 `already_present` 恒 0 是桩的产物，不是真幂等——真幂等由修补 4 自证 ① 盖，见下）。四句：分辨「GPU 侧核两把键、旧行退 1」与「GPU 侧像 CPU 侧那样旧行只多一行」；系统看得到旧行的键与 gpu_key；满足的分句是 `:4454` 的 `gpu_plan.rejected` 非空就退 1；改法——被拒行照列、照之后现判，但整条导入是否因「旧行对不上键」退非 0 另判（本趟该导的都导进去了就退 0、把被拒行当提示不当错），只在我的读法与模型上推过、被攻过零轮。推翻条件：commit-stages 指的第二台库每次导入前是只含这一趟行的全新库。

另核：我另跑了一次修补 4 自己的自证组（`BROAD_MUTATION_SELFTEST_CELLS=selftest_import_cells python3 scripts/mutation/broad-mutation.py --selftest`，登记为 light、不起 cargo），退 0、打一行 `✓ --selftest：…合并导入…`，`selftest_gpu_import_cells` 的四格（① 幂等再导不加、② 键对不上被拒照现判、③ 不成形整表拒、④ 两边定论不同不导）都过——H4 问句里「被拒的行之后由本机 GPU 趟照现判」在自证 ② 里成立，「自证四格的前提格是不是真的造出了两台分片的现场」成立（自证 `peer-run --shard 1/2`、`local-run --shard 0/2`，前提格核了 `peer_sequences == ["1","5"]`、`local_sequences == ["2","4"]`）。我的 H4 打中只在「持久库的旧行」这一个自证没覆盖的现场。

## H5 第一轮判决的方法

逐条核第一轮判决第一节「核查」与第五节「复跑」的方法（没打中，两点旁注）：

- 两腿在新旧两版钩子上喂 replay.sh 相关命令结果相同（判决第一节第 1 条）：站得住。那两版钩子的改动是「标准输入喂 shell 那一判」「没判全放行也记检出」，与 replay.sh 的 `--list-items` 放行无关；两腿与我这一轮在现版上喂同一批命令，结论不变。
- 正推一次复跑 `hook_feed` 的 diff 计数多 1、记「未稳」不采（判决第四节）：方法对——照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「单次观测不算数」，一次不稳不当证据，主 agent 自己复跑一趟逐字相同才落「未稳」，没拿它支撑任何结论。
- G1-g 用 961 个单元数推「多作废、不少作废」（判决第二节 G1 格）：数本身对（out/closure-base.txt 的 `judging-files-included` 961、496 出自 lfu；排除后 465）。但这个方法只核了「现状是多作废（偏保守）」，没推到「照定案排除之后会翻成少作废」——见 H3-b。即第一轮的方法对现状的描述站得住，对修补后果的预判缺一步。这不是判决的错（判决只判现状跟不跟定案），是 H3-b 要补的一笔。
- 旁注（方法的覆盖）：第一轮判决第一节第 3 条用的是 `cite-check.py <两份腿报告> --root . --background …`，不是 `.claude/rules/three-way-inference.md`@多轮：一次打穿不算数，三轮里多数打穿才算 现写的「先跑 `snapshot-tree.py` 倒推开工快照那棵树再逐份核 sha256」那一步——`scripts/agents/snapshot-tree.py` 的落盘时刻（`stat` 2026-10-23 10:40）晚于第一轮判决文件（07:16），那一步当时没有工具可用，判决用 `--root .`（现树）加对两份被改钩子按新版现查顶上。kb 与 judged 文件的 sha256 当时 OK，没因此漏判；只是现规则要求的倒推步在第一轮没走，本轮判决写之前主 agent 该走一遍（现在有工具了）。

## 没打中的形状

- H1：修补 1 五份逐处比，没找到「改出判决没要的东西」或「判决要的没改」；verification.md:50 的括号配平没写破。
- H2：本仓那一份经 `$(git rev-parse --show-toplevel)`、变量根、`bash -c cd`、子 shell、`pushd` 起 replay.sh `--list-items`（out/h2-run.txt 的 `own-replay-*` 六格）`judged_by_name` 多半解不出（`-`），但钩子都退 0——因为起的仍是本仓那一份、读正文只列项不起 cargo，没误拒；本仓那一份经只供自证的变量换根（`own-crash-point-replay-quick-root-override`）退 0。都不是放过也不是误拒。
- H2：没找到「本仓那一份被 `own_repository_script` 判成 False 而放过真重型」的例子——真路径对不上时退回读正文，本仓那三份入口的正文对真重型是拦得住的。
- H3：没造出一条「改 lfu 切单元后真翻面」的 crates 变异（H3-b 的后果是推的）；`lib_verdict_library` 对 crates 判法闭包贡献 0（H3-c），没找到它有函数进 crates 闭包。
- H4：没往第二台派活、没跑真两台工作流；`fresh-peer` 的幂等用桩测不出（靠修补 4 自证 ①）。GPU 判定行的 `--source` 不同、部分写入尾行（`read_table` 的 `partial_tail`）、同一份导两次——这几样都在修补 4 自证里核过，我没另造出偏差。

## 这条腿自己的限度

- 全在本机跑、全是 python 与钩子，没起 cargo、没跑重型、没编译、没碰任何判定库；H2 的符号链接 marker 另造一棵树实跑两条 `--list-items` 证实（只 source、不起 cargo）。
- H3-b「改 lfu 改复用判定」、H4「第二台库持久」是推的（库持久由 commit-stages 的固定库目录推，没跑真两台）。
- 所有改法只在我的读法或草稿上推过、被攻过零轮；一览表里标「推的」的没有实现、没有跑。
- 没判正推、辩方那几格；没替主 agent 采纳；副本上量出的数不算入库装置上的数。

## 没做什么

- 没跑门禁阶段、没编译、没跑重型；`crates-mutation-rows.py judging-units`、`closure_by_file.py` 只算 python 闭包，`broad-mutation.py --selftest` 是登记为 light 的单独 `--selftest`（不起 cargo）。
- 没读禁读清单里的文件（本轮另一条腿的报告与模型）。
- 草稿清理：交回前 `du -sh` —— 仓副本（`copy-base`、`copy-lfu-edit`、`copy-hooks-reverted` 各 71M）、`h3-refusal` 141M、几份小运行目录都已删（删前合计约 0.5G）；`/home/fy5090/.local/share/singlefs/work/m3-close-gov-b17-r2-opus` 只剩 `model/`（已拷进 `research/prompts/m3-close-gov-b17-r2-opus-model/`）与 `scratch/` 的日志，交回前一并删到只剩认领标记。

## cite-check

`python3 scripts/agents/cite-check.py research/prompts/m3-close-gov-b17-r2-opus-output.md` 末行：

```
  ✓ 核了 13 处引文、16 处裸行号，对上 10 处，没判 19 处（其中裸行号没有引文可比 10 处；逐处列在上面）
```

没判的多是代码引文只写了文件名（仓里有同名文件，cite-check 认不出是哪一份）与裸行号没有引文可比；逐处的全路径在正文里写明了（`.claude/hooks/lib_heavy_tests.py`、`scripts/experiment-apparatus/replay.sh`、`scripts/crash/crash-point-replay.sh`、`scripts/mutation/broad-mutation.py`、`scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/lib_mutation_units.py`），主 agent 判决时逐处现查（`grep -nF`）。
