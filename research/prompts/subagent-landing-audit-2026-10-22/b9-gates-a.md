# 第九批门禁甲（landing-audit-b9-gates-a）：交回报告（tooling-writer，2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#9。依据：records/2026-09-16-subagent拆分提案.md 第九十节 ⑧、⑫、⑱、㉔ 四行；research/prompts/landing-audit-r3-main-verification.md（N06、K2-8、K2-9）；上一份同类交回 research/prompts/subagent-landing-audit-2026-10-22/r2fix-gates.md。

## 结论（四条各自的出口）

| 条 | 验收 | 状态 |
|---|---|---|
| ⑧ | code-tooling 加一格；读法用 quote-kb.py 的 kb_candidate_roots；覆盖照写范围钩子真实配法；子目录写不到判红；红绿样本、开关；真仓读数 | 做完。新格 kb-directories-write-scope（code-tooling 由二十七格成二十八格）。红绿样本各一份，开关 CODE_TOOLING_BREAK=kb-scope-subdirectory-unchecked 打开后只有红样本判错。真仓读数：红（crates/singlefs-checker/invariants 子目录里的 .md 写回员写不到），写范围表没改，交主 agent |
| ⑫ | 登记表认 .claude/gate.d/ 顶层 lib- 起头的库不是门禁阶段（改认法那一处与它的自证）；加那一行；改了库之后照钩子的起法能跑它的自证；登记表自证加一格 | 做完。经 scripts/gate/hook-swap.py 换上 .claude/hooks/lib_heavy_tests.py 与 .claude/hooks/selftest-registry.tsv；登记行 `.claude/gate.d/lib-lag-baseline.py light`；`python3 .claude/gate.d/lib-lag-baseline.py --selftest` 现在按登记成 light 的自证放行并跑绿；自证加三格，新开关 LIB_HEAVY_TESTS_BREAK=gate-top-library-as-stage 打开后正好那三格红 |
| ⑱ | 两处逐处核用途；「与基准比」的改调 baseline_text、取不到判红、各一份样本；decision-evidence-lag.tsv 文件头补一句 | 做完。两处都是「与基准比」（理由在下面），都改了：checker-independence-and-sync 的 layout-checker-sync 格调 lib-lag-baseline.py 的 baseline_text，doc-decisions 的 history-loss 格调它的 show 命令行（同一份三种结局）；各一份样本，改前判错（一份判绿、一份退 77），改后判红；挂账表文件头补了一句 |
| ㉔ 门禁两处 | replay-comparison-units.tsv 文件头改成四元键；doc-registries 的 inline-sums、invariant-count-elsewhere 两格 python 起不来时判红并加样本 | 做完。新样本 kb-directories-unreadable-red 改前整道退 77、改后两格都红；开关 DOC_REGISTRIES_BREAK=kb-roots-failure-ignored 打开后只有这一份判错 |

推翻条件：主 agent 暂存后跑「要主 agent 跑的命令」第 1 条，四道样本档里有一份判错；或第 2 条三个开关下判错的样本与这里写的不等；或第 3 条 stage-break-switch-replay 报 doc-decisions、doc-registries 有点名与实测不等的条目落在这一批改的几条开关上；或主工作区里 `python3 .claude/hooks/lib_heavy_tests.py --selftest`、`bash .claude/hooks/heavy-test-guard.sh --selftest` 不再全绿。

## 负载与在哪跑

开跑前 `ps -o pid,args -u "$(id -u)"`：有别的会话的 checker 档 `cargo test --release -p singlefs-checker-tier`（pid 2502600）与崩溃放量的几个 python 在跑，没有 qemu-system、vm-bench.sh、e152、fio。这一批不编译、不起 cargo，全在本机主工作区跑，没等锁。

## 草稿里怎么核的样本（阶段本身没在真仓上跑）

样本档的 stage-selftest 要样本与暂存区相同，这一批不暂存，所以在草稿里照 stage-selftest.sh 的喂法逐份核：
- `tools/stage_copy.py <阶段文件> <拷贝>`：把阶段整份拷进草稿（不在 .claude/gate.d/ 这一层路径下），只改两样：去掉 preflight 两行，`STAGE_DIRECTORY` / `STAGE_DIR` 那一行改成主工作区的 .claude/gate.d；其余一字不动，照样 source 主工作区的 lib/stage-cells.sh 与各共用库，所以「样本只跑」「<格>：红（退出 1）」「<阶段>.sh：N 格里…」这类阶段级的 want 也核到了。改前那一版拷自 `before/`（动手前的快照，sha256 在 `before/SHA256SUMS`）。
- `tools/run_sample.sh <拷贝> <样本> <日志> [变量=值…]`：拷样本进临时目录、跑 setup.sh（`env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE`）、拿拷贝判那个目录，比 expect 的 exit= 与每条 want，输出原样进日志。
- `tools/switches.py`：从 code-tooling.sh 里取 BREAK_SWITCH_TABLE_LIBRARY（stage-break-switch-replay 格用的同一份读法）列一道阶段文件头的开关与点名的样本，逐条设上喂新样本，核点名与实测判错是否相等。
- `tools/syntax.py`：bash -n 加每段 python heredoc 的 compile。

真仓上只单跑了一格：⑧ 要的读数（kb-directories-write-scope），经同一份拷贝 `--check kb-directories-write-scope /home/fy5090/code/singlefs` 起。别的格都没在真仓上跑。

## ⑧　code-tooling 新格 kb-directories-write-scope

「覆盖」照钩子真实配法定：写闸 write-guard.sh 与 Bash 检出钩子共用 `.claude/hooks/lib_write_scope.py`，`glob_to_regex` 把 `**` 换成 `.*`、`*` 换成 `[^/]*`、别的字符 re.escape，`path_in_scope` 拿相对仓根的路径去配。所以 `<目录>/*.md` 只盖下一层；`<目录>/**/*.md` 只盖子目录、下一层的 `<目录>/x.md` 配不上（`.*` 之后还要一个 `/`）；`<目录>/**` 两样都盖。新格直接调这两个函数（用这一道所在仓里那一份），不另写配法；`$SINGLEFS_WORK_ROOT/` 起头的模式交 `expanded_scope_patterns(…, None)` 丢掉。

判法（格函数上方注释写全了）：被判的根里 `.claude/kb-directories` 登记的每个目录（这一道所在仓里 `scripts/kb/quote-kb.py` 的 `kb_candidate_roots`，没登记就是 `.claude/kb`），探两条造出来的路径 `<目录>/kb-directories-write-scope-probe.md` 与 `<目录>/kb-directories-write-scope-probe/kb-directories-write-scope-probe.md`，kb-scribe 的模式要各配上一条，不看目录里此刻有没有子目录；别的 agent 的行不算。没有 `.claude/hooks/agent-write-scope.tsv` 退 77；表里没有 kb-scribe 的行每个目录都红。阶段归属表加了一行（tooling-writer）。

新格的样本与开关：
- kb-directories-write-scope-red：登记 .claude/kb（kb-scribe `.claude/kb/**`）、notes/invariants（kb-scribe 只有 `notes/invariants/*.md`）、docs/extra（只有 clerk 的 `docs/**`）。
- kb-directories-write-scope-green：三个目录 kb-scribe 都写 `<目录>/**`。
- CODE_TOOLING_BREAK=kb-scope-subdirectory-unchecked：只探下一层。

读数（`logs/ct-*.log`）：
```
== before（改前的阶段：格还不存在）
✗ kb-directories-write-scope-red 期望退出 1，实测 2
✗ kb-directories-write-scope-green 期望退出 0，实测 2
== after
✓ kb-directories-write-scope-red 退出 1（want 5 条都在）
✓ kb-directories-write-scope-green 退出 0（want 2 条都在）
== after + switch
✗ kb-directories-write-scope-red 退出 1 对，want 2/5 条找不到（CODE_TOOLING_BREAK=kb-scope-subdirectory-unchecked）
    找不到「.claude/kb-directories 登记的 notes/invariants：写回员（kb-scribe）写不到子目录里的 .md（探 notes/invariants/kb-directories-write-scope-probe/kb-directories-write-scope-probe.md）」
    找不到「登记的 3 个 kb 目录里 2 个写回员写不全」
✓ kb-directories-write-scope-green 退出 0（want 2 条都在）（CODE_TOOLING_BREAK=kb-scope-subdirectory-unchecked）
```
（改前退 2 是「样本根的 .gate-cells 里有没登记的格名：kb-directories-write-scope」。样本里的 kb 页补了「## 历史版本」一节之后两份重跑过，读数相同。）

真仓读数（`logs/ct-real-repo.log`，rc=1，原样）：
```
── kb-directories-write-scope：.claude/kb-directories 登记的每个 kb 目录，下一层与子目录里的 .md 都在写回员（kb-scribe）的写范围里
  ✗ .claude/kb-directories 登记的 crates/singlefs-checker/invariants：写回员（kb-scribe）写不到子目录里的 .md（探 crates/singlefs-checker/invariants/kb-directories-write-scope-probe/kb-directories-write-scope-probe.md）
  ✗ 登记的 2 个 kb 目录里 1 个写回员写不全（.claude/hooks/agent-write-scope.tsv 里 kb-scribe 的模式：.claude/kb/**、crates/singlefs-checker/invariants/*.md、$SINGLEFS_WORK_ROOT/**；** 跨目录、* 不跨目录）
```
写范围表与 `.claude/agents/kb-scribe.md`「写范围」没改（派发说那张表不归我）。改法：kb-scribe 那一行改成 `crates/singlefs-checker/invariants/**`，或另加 `crates/singlefs-checker/invariants/**/*.md`，定义同步改（agent-write-scope 格判表与定义逐项一致）。今天那个目录下没有子目录，这一格红的是「载体再长出子目录时写不到」。

## ⑫　登记表认门禁目录顶层 lib- 起头的共用库

改法在 `.claude/hooks/lib_heavy_tests.py`（登记表的读法与判法都在它；写范围检出、看门狗、light-selftest-guard 都经它的 classify_selftest / selftest_registry_key）：
- 新常量 `GATE_TOP_LIBRARY_PREFIX = "lib-"`，`gate_stage_name` 原来写死的 `"lib-"` 改用它；新函数 `gate_top_library_key(key)`：键在 `.claude/gate.d` 这一层、文件名以 lib- 起头。
- `selftest_registry_at`：这类键不当样本档行（`stage_row` 加 `and not gate_top_library_key(key)`），也不报「不在三处目录里」；照脚本收 light / heavy / heavy-if-set。
- `selftest_registry_key`：命令词落在 `.claude/gate.d` 这一层、又是 lib- 起头的，交 `.claude/gate.d/<文件名>`。`classify_selftest` 没改：这类库与脚本一样只认单独一个 `--selftest`（lib/ 底下那几份「直接执行就是自证」的认法不套到它们身上；lib-lag-baseline.py 的 `show`、`replay-rows` 是门禁在用的正事）。
- 文件头「自证」一段补了这一层；开关表照 mutate.sh 写法加 `LIB_HEAVY_TESTS_BREAK=gate-top-library-as-stage`。
- 自证：样本表加 `.claude/gate.d/lib-sample-library.py light` 一行，selftest_registry_cells 加三格（登记成 light 的单独 --selftest 放行、没登记的 lib- 库 --selftest 照拒、带别的参数不当自证）。

`.claude/hooks/selftest-registry.tsv`：表头两处补这一层，加一行 `.claude/gate.d/lib-lag-baseline.py	light	# 临时 git 仓里造基准三种结局（有表、没有表、git 取不到）与复跑表样本，只起 git 与它自己；不起 cargo、编译、虚机`（读过它的自证：只起 git 与 `sys.executable` 起它自己）。

先造的红输入（改前，`logs/registry-probe.tsv` 是登记表加这一行）：`selftest_registry_problems` 报「第 195 行 .claude/gate.d/lib-lag-baseline.py：门禁阶段的判定「light」认不出（只认 samples-light、samples-light-marker:<相对样本根、不带 .. 的路径>、samples-heavy）」。

换法：`scripts/gate/hook-swap.py`，喂法 `edits/hook-feeds.json`（9 条，钩子 heavy-test-guard.sh），规格 `edits/hook-spec.json`（15 处）与 `edits/hook-spec-2.json`（文件头补一句）；两次都是「草稿里验过……换上了……主工作区里再验过」退 0，反向规格在 `hook-swap/reverse-*.json`。

换上之后的读数：
```
python3 .claude/hooks/lib_heavy_tests.py --selftest                       rc=0  ✓ lib_heavy_tests 自检通过（查了 783 种：…样本登记表与仓里那份登记表 41 种…）
LIB_HEAVY_TESTS_BREAK=gate-top-library-as-stage 同上                      rc=1  红三格：
  ✗ lib_heavy_tests 自检：样本表：门禁目录顶层 lib- 起头的共用库没登记的 --selftest 照拒 应当是 commit-check-selftest-unregistered，实际 None
  ✗ lib_heavy_tests 自检：样本登记表里写坏的几行逐条报出来（判定、理由、目录、重复、文件不在、门禁阶段那几行的判定与标记） 应当是 [...]
  ✗ lib_heavy_tests 自检：仓里的登记表（…/.claude/hooks/selftest-registry.tsv）没有写坏的行、指到的文件在 … 里都在 应当是 []，实际 ['第 174 行 .claude/gate.d/lib-lag-baseline.py：门禁阶段的判定「light」认不出…']
bash .claude/hooks/heavy-test-guard.sh --selftest                         rc=0  ✓ 自检通过（查了 1395 种，其中该拒 264 种）…
LIB_HEAVY_TESTS_BREAK=gate-top-library-as-stage 同上                      rc=1  上面三格加「✗ 自检：lib_heavy_tests.py 的自检 应当是 0，实际 1」
python3 scripts/gate/light-selftest-guard.py --selftest                   rc=0  ✓ light-selftest-guard.py 自证 14 格都对
python3 .claude/gate.d/lib-lag-baseline.py --selftest                     rc=0  ✓ lib-lag-baseline.py 自证：9 项都过
```
判法直接核（`classify_selftest` 等，真仓）：键 `.claude/gate.d/lib-lag-baseline.py`；`--selftest` → `(True, None)`（登记成 light 的自证，放行）；`show HEAD x` → `(False, None)`（不是自证）；`selftest_judged_light` 为真；`selftest_registry_problems` 为空。code-tooling 的 research-script-selftests 格那一条从「表外、原样 exec、本次未盯」变成按 light 盯着跑（没在真仓上跑这一格）。

## ⑱　取基准那一版、取不到交空串的两处

两处都是「与基准比」，都改了：

1. `.claude/gate.d/checker-independence-and-sync.sh` layout-checker-sync 格的 `baseline_text`（改前约第 668 行，`git show {base}:{path}`，退非 0 交空串）。用途：取每条格式定义路径在基准那一版的全文，抽出「常量名 → 值」与此刻的比，算这次增、删、改了哪些常量（第 ④ 条）。它不是只缩不涨，是「与基准比出变动」；取不到交空串时与「基准里没有这份（这次新加的）」分不出：常量全算新出现的，同一套布局里另一份没变且同值时按「首次登记」放过去，判绿还报一句不真的「首次登记」。改法：加载 `lib-lag-baseline.py`，`baseline_text` 改调它的 `baseline_text(base, path)`：基准里没有交空串（全算新增，同改前）；git 取不到记进 `baseline_problems`、那一套布局第 ④ 条不往下比，收尾判红、逐份点名。
2. 门禁 doc-decisions 的 history-loss 格（`cell_history_loss`，改前 `git cat-file -e` 失败就 `continue`，接着 `git show … > 副本` 不看退出码）。用途：拿两份变更史基准那一版与此刻的交 history-consolidate.py loss-check 判整理丢没丢，也是与基准比。取不到时跳过，两份都跳过之后报「两份变更史与基准 HEAD 无差异」退 77。改法：改起 `python3 "$LIB_LAG_BASELINE" show "$diff_base" "$hist"`（退 0 有、退 3 基准里没有、别的是 git 取不到，问题句在 stderr；同一份三种结局），退 3 照旧跳过，别的判红、点名那一份与问题句，出路另写一句。

各一份「基准取不到」样本（setup.sh 提交基准之后把那一版的对象从对象库里删掉：ls-tree 认得出、show 读不出）：
- checker-independence-and-sync.sh/layout-checker-sync-base-unreadable-red：.rs 与布局.md 的标记都是 16384、两份都没改，删布局.md 那一版（工作区没改的那一份，改动范围照取得到；删改过的那一份时 gate_changed_paths 先读不出、改前就红了，判不出这一处）。
- doc-decisions.sh/history-loss-base-unreadable-red：现场同 history-loss-green，删基准那一版 decisions-history.md。

读数（`logs/cis-*.log`、`logs/dd-*.log`）：
```
改前 checker-independence：✗ layout-checker-sync-base-unreadable-red 期望退出 1，实测 0
  （输出：「✓ 布局清单 1 套布局、3 条路径都在…另有 1 个是第一次登记标记…」「首次登记：SAMPLE_UNIT_BYTES = 16384（format/布局.md…）」）
改后：✓ layout-checker-sync-base-unreadable-red 退出 1（want 3 条都在）
  「✗ 1 条格式定义路径取不到基准 HEAD 那一版，第 ④ 条对它们没比（不是没变）：」
  「format/布局.md：读不出基准 HEAD 里的 format/布局.md（git show 退 128：fatal: bad object HEAD:./format/布局.md）」
改后 LAG_BASELINE_BREAK=unreadable-as-absent：✗ … 期望退出 1，实测 0
改后这一道 16 份样本：16 份 ✓（logs/cis-after-*.log）

改前 doc-decisions：✗ history-loss-base-unreadable-red 期望退出 1，实测 77（「! 两份变更史与基准 HEAD 无差异，这一格无对象可判」）
改后：✓ history-loss-base-unreadable-red 退出 1（want 4 条都在）
  「✗ .claude/kb/decisions-history.md：读不出基准 HEAD 里的 .claude/kb/decisions-history.md（git show 退 128：fatal: bad object HEAD:./.claude/kb/decisions-history.md），整理丢没丢没比」
改后这一道 53 份样本：53 份 ✓（logs/dd-after-all.txt）
```
开关与文件头：
- checker-independence-and-sync（整道在 stage-break-switch-replay 豁免表里）：文件头样本表加这一份；STAGE_CELLS_BREAK=red-swallowed、REJECTION_BLOCKS_BREAK=summaries-only 两条补点名，加一条 LAG_BASELINE_BREAK=unreadable-as-absent。逐条设上喂新样本的实测：red-swallowed 实测 0、summaries-only 明细出不来、两条 CHECKER_INDEPENDENCE_BREAK 与一条 CPU_JUDGING_SITES_CHECK_BREAK 不碍它，与写的一致。
- doc-decisions：文件头样本表加这一份；头注 58 条开关逐条设上喂新样本，判错的正好五条：STAGE_CELLS_BREAK=red-swallowed、=marker-ignored、=check-ignored、DOC_DECISIONS_BREAK=history-loss、LAG_BASELINE_BREAK=unreadable-as-absent，五条都补了点名（括号里写判错的第一行）；改完复核「点名而没判错」「判错而没点名」两种都是 0（`logs/dd-switch-results.txt`）。marker-ignored、check-ignored 两条在 stage-break-switch-replay-exempt.tsv 里，不复跑，照样补了。
- `.claude/gate.d/decision-evidence-lag.tsv` 文件头「只缩不涨」那一句后补一行：git 取不到基准那一版判红，不当成没有这张表（取法 lib-lag-baseline.py 的 baseline_text）。

## ㉔　门禁两处

1. `.claude/gate.d/replay-comparison-units.tsv` 文件头第 6 列说明里「按登记出处、留存产物名成对」改成「按（实验号，登记出处，单元名，留存产物名）四样一起当键：换实验号、换单元名借同一份登记也算新行」（与 doc-experiments 的 none_identity 一致）。这份文件这段时间里另有会话加了「不登记的比对对象」几行，没碰。
2. 门禁 doc-registries：invariant-count-elsewhere 格开头先 `kb_candidate_roots > …roots`、看退出码，起不来判红 `return 1`（原来在中段那一次调用删掉，复用这一份）；inline-sums 格那一次调用同样看退出码。两格的红行写「<格> 没判：kb 目录登记位读不出（原因在上面）…」，出路另一行。`kb_candidate_roots` 自己那句出路改成「登记位写坏了（不是 UTF-8、读不了）就改它，读法本身坏了修 quote-kb.py」。新开关 `DOC_REGISTRIES_BREAK=kb-roots-failure-ignored`（退回不看退出码）。
   新样本 doc-registries.sh/kb-directories-unreadable-red（.gate-cells 点名这两格；setup.sh 往 .claude/kb-directories 现写一个 0xff 字节，quote-kb.py 读它抛 UnicodeDecodeError，坏字节不进仓）。
```
改前：✗ kb-directories-unreadable-red 期望退出 1，实测 77（两格都「本次未跑」：「! 候选 kb 目录里一个带不变量表行的文件都没扫到」「! 本次无对象可判：扫了 0 份 kb 文件…」）
改后：✓ kb-directories-unreadable-red 退出 1（want 6 条都在）
改后这一道 51 份样本：51 份 ✓（logs/dr-after-all.txt）
改后 DOC_REGISTRIES_BREAK=kb-roots-failure-ignored 喂 51 份：50 份 ✓，只有「✗ kb-directories-unreadable-red 期望退出 1，实测 77」（logs/dr-newswitch-all.txt）
```
   头注 60 条开关逐条设上喂新样本，判错的五条（DOC_REGISTRIES_BREAK=invariant-count-elsewhere、=inline-sums、=kb-roots-failure-ignored、STAGE_CELLS_BREAK=red-swallowed、=check-ignored）都补了点名，复核两种不等都是 0（`logs/dr-switch-results.txt`）。别的样本不碰新分支（kb_candidate_roots 在它们上面都读得出），所以别的开关下别的样本的判错集合不变（推的，没逐条全跑）。

## 改过的文件

改的（都在主工作区，没暂存）：
```
 .claude/gate.d/checker-independence-and-sync.sh | 40 +++++++++++--
 .claude/gate.d/code-tooling.sh                  | 74 ++++++++++++++++++++++++-
 .claude/gate.d/decision-evidence-lag.tsv        |  3 +-
 .claude/gate.d/doc-decisions.sh                 | 37 +++++++++----
 .claude/gate.d/doc-registries.sh                | 29 ++++++++--
 .claude/gate.d/replay-comparison-units.tsv      | 14 ++++-
 .claude/gate.d/stage-owners.tsv                 |  1 +
 .claude/hooks/lib_heavy_tests.py                | 40 +++++++++++--
 .claude/hooks/selftest-registry.tsv             |  8 ++-
 9 files changed, 209 insertions(+), 37 deletions(-)
```
上面是相对暂存区的原样输出，里面混着别的会话没暂存的改动。只算这一批自己的（与动手前的快照 `before/` 比，`logs/mine-*.diff`）：
```
.claude/gate.d/code-tooling.sh	+66 -3
.claude/gate.d/stage-owners.tsv	+1 -0
.claude/hooks/selftest-registry.tsv	+6 -2
.claude/hooks/lib_heavy_tests.py	+32 -6
.claude/gate.d/checker-independence-and-sync.sh	+32 -6
.claude/gate.d/doc-decisions.sh	+27 -10
.claude/gate.d/decision-evidence-lag.tsv	+2 -1
.claude/gate.d/doc-registries.sh	+23 -6
.claude/gate.d/replay-comparison-units.tsv	+11 -3
```
replay-comparison-units.tsv 这一批只改了一行（-1 +1），另外的 +10 -2 是这段时间里别的会话加的「不登记的比对对象」几行。

新建的样本目录（未跟踪，共 5 份）：
- `.claude/gate.d/fixtures/code-tooling.sh/kb-directories-write-scope-red/`（7 份文件）
- `.claude/gate.d/fixtures/code-tooling.sh/kb-directories-write-scope-green/`（7 份文件）
- `.claude/gate.d/fixtures/checker-independence-and-sync.sh/layout-checker-sync-base-unreadable-red/`（7 份文件）
- `.claude/gate.d/fixtures/doc-decisions.sh/history-loss-base-unreadable-red/`（3 份文件）
- `.claude/gate.d/fixtures/doc-registries.sh/kb-directories-unreadable-red/`（4 份文件）

没有新门禁阶段、新钩子：⑧ 是 code-tooling 里加一格，没写 `# gate-similar:` / `# hook-events:` 行。

## 语法检查

`tools/syntax.py`（bash -n 加每段喂给 python 的 heredoc compile；.py 整份 compile，不写 __pycache__）原样：
```
.claude/gate.d/code-tooling.sh：bash -n 过，内嵌 python 21 段
.claude/gate.d/checker-independence-and-sync.sh：bash -n 过，内嵌 python 2 段
.claude/gate.d/doc-decisions.sh：bash -n 过，内嵌 python 17 段
.claude/gate.d/doc-registries.sh：bash -n 过，内嵌 python 28 段
.claude/hooks/lib_heavy_tests.py：整份编译过
.claude/gate.d/fixtures/checker-independence-and-sync.sh/layout-checker-sync-base-unreadable-red/setup.sh：bash -n 过，内嵌 python 0 段
.claude/gate.d/fixtures/doc-decisions.sh/history-loss-base-unreadable-red/setup.sh：bash -n 过，内嵌 python 0 段
.claude/gate.d/fixtures/doc-registries.sh/kb-directories-unreadable-red/setup.sh：bash -n 过，内嵌 python 0 段
编译了 69 段，编译不过 0 段
```
hook-swap.py 换上前后各自另做了一遍：「3 份文件语法检查、内嵌 python 1 段编译」。命名检查没跑：这一批没改 `crates/`、`scripts/` 下的 `.rs`。

## 要主 agent 跑的命令

工作目录都是主工作区仓根 `/home/fy5090/code/singlefs`，不起 cargo，内存与线程没有特别要求。样本档要样本与 HEAD 加暂存区逐字节相同：先暂存这一批（「改过的文件」里九份——replay-comparison-units.tsv 与 code-tooling.sh 等几份里另有别的会话的改动，照共用约束只核这一批自己的那几处——与五个新样本目录）再跑 1、2 两条。

1. 四道的样本档（判：每道末行判得都对、没有一份判错）：
```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh、checker-independence-and-sync.sh、doc-decisions.sh、doc-registries.sh 四道的样本档; 范围=4 道阶段; 为什么=这一批加了 kb-directories-write-scope 一格、两处取基准改调 lib-lag-baseline.py、doc-registries 两格读不出登记位判红，各加了样本' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item code-tooling.sh --item checker-independence-and-sync.sh --item doc-decisions.sh --item doc-registries.sh
```
2. 三个新开关与库的开关红在点名那几份（判：判错的样本正好是括号里那几份）：
```
CODE_TOOLING_BREAK=kb-scope-subdirectory-unchecked SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的样本档，设 kb-scope-subdirectory-unchecked; 范围=1 道阶段; 为什么=核只探下一层时只有 kb-directories-write-scope-red 判错' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item code-tooling.sh
   （判错的只有 kb-directories-write-scope-red）
DOC_REGISTRIES_BREAK=kb-roots-failure-ignored SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-registries.sh 的样本档，设 kb-roots-failure-ignored; 范围=1 道阶段; 为什么=核不看登记位读不出时只有 kb-directories-unreadable-red 判错' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item doc-registries.sh
   （判错的只有 kb-directories-unreadable-red）
LAG_BASELINE_BREAK=unreadable-as-absent SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-independence-and-sync.sh、doc-decisions.sh 的样本档，设 lib-lag-baseline.py 的弄坏开关; 范围=2 道阶段; 为什么=核取不到基准当成没有时判错的正好是两道头注点名的样本' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item checker-independence-and-sync.sh --item doc-decisions.sh
   （判错的正好是 layout-checker-sync-base-unreadable-red、evidence-ledger-base-unreadable-red、history-loss-base-unreadable-red）
```
3. 复跑方与结构格（判：都绿；stage-break-switch-replay 复跑 doc-decisions、doc-registries 头注里这一批补了点名的几条与新加的 kb-roots-failure-ignored；break-switch-headers 判 lib_heavy_tests.py 新开关三样写全；stage-owners、gate-structure、fixture-claims 判新格的归属行、格名表与红绿样本；kb-directories-write-scope 在真仓上照上面的读数是红的，交主 agent 定改写范围表）：
```
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 stage-break-switch-replay、break-switch-replay、break-switch-headers、stage-owners、gate-structure、fixture-claims、kb-directories-write-scope 七格; 范围=7 格; 为什么=两道头注补了开关点名、lib_heavy_tests.py 加开关、code-tooling 加一格与样本' bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay,break-switch-replay,break-switch-headers,stage-owners,gate-structure,fixture-claims,kb-directories-write-scope
```
4. 真仓单跑改过的格（判：都绿或照旧；layout-checker-sync 与 history-loss 的成功行与上一次提交时的读数相同；两格 doc-registries 的读数相同）：
```
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-independence-and-sync.sh 的 layout-checker-sync 格; 范围=1 格; 为什么=取基准改调 lib-lag-baseline.py，取不到判红' bash .claude/gate.d/checker-independence-and-sync.sh --check layout-checker-sync
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 history-loss 格; 范围=1 格; 为什么=取基准改起 lib-lag-baseline.py show，取不到判红' bash .claude/gate.d/doc-decisions.sh --check history-loss
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-registries.sh 的 invariant-count-elsewhere、inline-sums 两格; 范围=2 格; 为什么=读不出 kb 目录登记位改成判红' bash .claude/gate.d/doc-registries.sh --check invariant-count-elsewhere,inline-sums
SINGLEFS_HEAVY_TESTS=commit SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 research-script-selftests、selftest-registry-deviations 两格; 范围=2 格; 为什么=lib-lag-baseline.py 登记成 light 之后那一条自证改由守卫盯着跑，登记表加了一行' bash .claude/gate.d/code-tooling.sh --check research-script-selftests,selftest-registry-deviations
```
5. 提交前那一趟会判到的（gate-lint、shell-lint、preflight-lint 判改过的四道；gate-overlap 判 checker-independence-and-sync 与 doc-decisions 新调库的两段；doc-lint、doc-text 判新样本里的文字）：整轮 `scripts/gate/gate-staged.sh` 里跑，不单列。

## 没核动的（读代码推的，没实跑）

- 四道里别的样本在别的开关下的判错集合：新分支只在「取不到基准」「登记位读不出」时才走，别的样本都取得到、读得出，推的是集合不变；doc-registries 的新开关喂过全部 51 份（只有新样本判错），doc-decisions、doc-registries 的头注开关只拿新样本逐条核过。第 3 条命令的 stage-break-switch-replay 会全量核。
- code-tooling 别的样本没在草稿里重跑（只跑了新格两份与 selftest-registry-deviations、agent-write-scope-green 三份当对照）：新格只在 .gate-cells 点名它的样本里起。第 1 条命令会核。
- 真仓上 layout-checker-sync、history-loss、两格 doc-registries 没跑：真仓取得到基准、读得出登记位，推的是读数与改前相同。第 4 条命令会核。
- gate-structure、stage-owners、fixture-claims 三格对新格的判定：草稿里只核了拷贝的 `--list` 出 28 行、文件头 `# gate-cell:` 28 行、`stage_cell` 28 行、阶段归属表 code-tooling.sh 28 行，格本身没跑。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 broad-mutation.py 起 cargo 的用法、gate.sh、全量 cargo test、check.sh）；没在真仓上跑门禁阶段与各类 lint（真仓上只为 ⑧ 的读数单跑了一格，见上）；没走定义三方（这一批没改定义、共用约束、规则）；没提交，没暂存。
- 写范围表 `.claude/hooks/agent-write-scope.tsv` 与 `.claude/agents/kb-scribe.md`「写范围」没改：⑧ 的新格在真仓上红，改法写在 ⑧ 一节末尾，交主 agent。
- `.claude/gate.d/lib-lag-baseline.py` 文件头「调它的九处」没补这一批新加的两处（checker-independence-and-sync 的 layout-checker-sync、doc-decisions 的 history-loss）：这份不在派发列的文件里。
- `.claude/hooks/heavy-test-guard.sh` 文件头「自证」一段（「scripts/、.claude/hooks/ 下脚本的 --selftest 与 .claude/gate.d/lib/ 下共用库的直接执行」）没补门禁目录顶层 lib- 起头的库：派发只许改登记表与认它的判法（lib_heavy_tests.py 文件头已写）。
- doc-registries 里另外四格（audit-contradictions、declared-counts、invariant-anchors、invariant-prerequisites）也经 kb_candidate_roots 取候选目录，读不出时照旧当空的（上一批报告说它们会因找不到不变量表而红，没核）；派发只点了两格。
- 改了 ⑫ 之后，门禁目录顶层另外几份 lib- 库（lib-forbidden-notations.py 这类，本身不实现 --selftest）带 --selftest 起会按「表外的自证」被拒（改前不认、放行）；hook-swap 的喂法里有这一条（退 2）。
- `.claude/batch-scope` 没登记这一批的文件（不在写范围里）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-b9-gates-a/` 里是：`tools/`（stage_copy.py、run_sample.sh、switches.py、syntax.py、rep.py 小脚本）、`before/`（动手前九份文件的快照与 SHA256SUMS）、`copies/`（改前改后阶段的拷贝，各 4 份）、`edits/`（hook-swap 的规格、喂法与插进 code-tooling 的那段新文本）、`hook-swap/`（喂法输出、检出记录、反向规格）、`logs/`（各次读数）；没有仓副本、工作树与编译目录。
