# defs-m2-closeout-r2 改法 G1–G9 落地报告

写于 2026-09-26。依据：`research/prompts/defs-m2-closeout-r2-main-verification.md` 第三节（G1–G9）与第二节（H1–H9）；原文、探针与 G4 的收严取自 `research/prompts/defs-m2-closeout-r2-opus-output.md` 与 `research/prompts/defs-m2-closeout-r2-opus-model/`（`f13-fix-g1.sh`、`cases-f13.json`、`probe.py`）。
改前备份：`/tmp/claude-1000/defs-closeout-r2-fixes/before/`（开工时 `cp -p`，11 份文件加 74 号样本目录）；改前 / 改后 sha256：同目录 `start-sha256.txt`、`end-sha256.txt`；改后全量 diff：`/tmp/claude-1000/defs-closeout-r2-fixes/my-changes-final.diff`（258 行，11 份文件；74 号样本没改），本报告末尾原样附上。
开工时 `sha256sum -c research/prompts/defs-m2-closeout-r2-snapshot/sha256sums.txt` 24 个全 OK（第二轮快照之后没人动过这批文件）。
只动了放行的文件；没 checkout / restore / reset / clean，没提交。三个脚本（弹窗闸、74 号、15 号）都是先在草稿目录写好、验过，再拷成同目录临时文件、`chmod --reference`、`mv` 换上；装上后 `cmp` 与草稿逐字节相同。
最后一处改动，七道门禁起跑，之后 11 份文件的 sha256 与 `end-sha256.txt` 逐行相同。

## 一、G1–G9 逐条

| G | 文件与小节（改后行号） | 改了什么 |
|---|---|---|
| G1 | `.claude/agents/gate-triage.md`「输入」、第 1b 步（:26）、第 2 步（:27） | 撤回 F9：删「`gate.sh` 整条经内存包装的上限」这一项输入；1b、2 还原成 F9 之前的写法（草稿 `/tmp/claude-1000/defs-closeout-r1-fixes/before/.claude/agents/gate-triage.md`），命令回到 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`。与那份草稿逐行 diff 只差一处：1b 的括注从「87 号经 `research/scripts/replay.sh` 逐条套了」变成「87 号经 `research/scripts/replay.sh` 逐条套了，15、74 号在阶段里面经包装」 |
| G1 | `.claude/gate.d/74-model-differential.sh`（:39–43 注释与变量；cargo 那一支） | `cargo test --release -p singlefs-harness --test "$TEST_BINARY"` 改成经 `research/scripts/run-with-memory-cap.sh` 跑（包装路径按阶段文件自己所在的仓根取，与同一文件取 `stage-must-run.sh` 的写法相同）。上限取 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`，没设取 8G（`research/scripts/replay.sh:24` 的 `REPLAY_MEMORY_CAP` 默认值）。包装退 250–254 单独判红、出路指到包装文件头「退出码」一段；普通判红那一支的单跑命令改用同一个上限变量的值 |
| G1 | `.claude/gate.d/15-research-build.sh`（:25–38） | `cd research && cargo test --release` 改成经同一个包装跑；上限取 `GATE_RESEARCH_BUILD_MEMORY_MAX`，没设取 8G；峰值表的键写死成 `gate 15-research-build: cargo test --release (research)`（不写死的话键是「cargo test --release」，分不出工作区）；包装退 250–254 单独判红 |
| G1 | `.claude/agent-common.md`「不做」一节「跑编译出来的代码经内存包装」那一条（:48） | 「`mutate.sh`、`replay.sh` 与门禁 59 号在里面逐条套了」后面加「门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`）」 |
| G1 | `.claude/agents/crash-verifier.md` 第 1b 步（:26） | 「59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层」 |
| G1 | `.claude/agents/implementation-writer.md` 第 1b 步（:26） | 末尾加「登记给你的 74 号在阶段里面经包装，照跑，外面不再包一层」 |
| G1、G9 | `.claude/main-agent.md`「派发提示怎么写」（:48） | 删「派门禁分诊员时给 `gate.sh` 整条经内存包装的上限」；改成「派崩溃验证员时给 54、55、57 号各自的内存上限，每道一个数，照 crash-verifier「输入」一节给：54、57 号写明是量过的还是推的，55 号的不小于同时起的虚机数乘每台的内存。门禁 15、74 号在阶段里面经内存包装，要换它们的上限就在派发提示里给一个数，没给用 `replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值」 |
| G9 | `.claude/agents/crash-verifier.md`「输入」（:20） | 「每道一个数：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值）还是推的；55 号的不小于……」（55 号照 F8 原样） |
| G2 | `.claude/agents/experiment-runner.md` 第 4 步（:31）、第 4c 步（:33） | 删「逐字节一致就删掉这一次的新文件、不新存」。改成「这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh` 里这个实验的登记行改指新文件，旧的留着；拿新文件与已有的那份比，逐字节一致的在报告里写明，对不上的在实验页两份都点名、写明对不上，交主 agent 定哪份承重」。4c 里「逐字节一致没新存的，读 `research/results/` 里那一份」那半句随之删掉 |
| G3 | `.claude/agents/three-way-attack.md` 第 3b 步第 4 条（:33） | 加「原型里的流只许在原型里自己造，不从名字带 `layer0` 的用例里拷（拷文件、拷代码段、`include!`、`mod` 引进来都算拷）」与「这一轮全部原型跑的全量合起来不超过约 10⁷ 个状态」；结尾改成「一段历史自己就超过约 10⁶ 的，那段不跑；再跑就要越过约 10⁷ 的，剩下的不跑；两样都写进报告交主 agent」 |
| G4 | `.claude/hooks/ask-user-claim-guard.sh`：`FILE_AND_LINE`（:57–59）、`FILE_NAME_CHARACTER` 上的注释、文件头判法、自证 | 第一支 `/` 之后换成攻方 `f13-fix-g1.sh` 的写法 `(?:{ASCII_PATH_CHARACTER}+|{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))`（逐字相同）；自证加 6 格必拒；文件头判法与成功行跟着改（第二节） |
| G5 | `.claude/agents/experiment-runner.md` 第 4c 步（:33） | 点名对象改成表示「没过」的字段：取值是布尔 `false` 的、取值是 `not_run` 的、取值是大于 0 的整数而名字表示违例 / 不匹配 / 歧义 / 失败的计数；取值是布尔 `true` 的、取值是 `zero` 的（例 `control_violations_ok=true`、`layer0_violations=zero`）与名字不表示「没过」的计数（例 `journal_differing_states`）不点名；拿不准的点名，写「拿不准」。「一个都没有」那句改成「……违例、不匹配、歧义、失败类的整数计数都是 0」 |
| G6 | `.claude/agents/three-way-attack.md` 第 3c 步（:38）；`.claude/agent-common.md` ④（:64） | 3c：`.rc` 文件名带批号与件号（`<草稿目录>/b<批号>-<件号>.rc`），每批开跑前 `rm -f <草稿目录>/b<批号>-*.rc`；`wait` 之后数一遍这一批的 `.rc`，与派出去的件数对不上整批作废，对得上再按起的次序读。④：同样三句（文件名带批号与件号、每批开跑前先删这一批的文件、读之前数一遍、对不上整批作废） |
| G7 | `.claude/agents/experiment-runner.md` 第 6 步（:35）、第 7 步（:36） | 第 6 步「84 号除外」改成「读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）」，放到第 7 步写完实验页之后跑；不写实验页的在第 6 步最后跑、红了照写，并加「产物照第 4 步一个不删（86 号出路里『把 research 下那些文件删掉』那一句不照做，写进报告交主 agent）」。第 7 步末尾改成「跑第 6 步留下的那几道，各贴原样末行与退出码」。名单为什么是十道见本节表下 |
| G8 | `.claude/agents/three-way-local-defense.md`「文件名形态」（:19） | 样本不再写死成 `research/prompts/<轮>-local-defense-output-s<n>.md`：「样本照攻方写成 `<前缀>-output-s<n>.md`，前缀取派发提示给的（与攻方「输入」那一项同），不写死」；提示、核对表、运行记录三个名字不变 |

**G7 名单为什么是十道**：判决第三节 G7 写的判据是「登记给它的阶段里凡是读实验页的（84、40、86 号）」。照这个判据现查了登记给 experiment-runner 的 14 道（`awk` 取法同共用约束「门禁」一节）：`27 33 34 40 52 80 84 85 86 88 69 75 96 99`。逐道 `grep -n 'kb/experiments\|experiments\.md\|\.claude/kb\|KB=\|kb_dir'`，读实验页或索引的是 27（`:56` `glob('.claude/kb/**/*.md')`）、34（`:30–31` `IDX=.claude/kb/experiments.md`、`EXP=.claude/kb/experiments`）、40（`:4` 判据点名 `kb/experiments.md`）、69（`:119` `os.path.join(kb_dir, "experiments")`）、75（`:40` `.claude/kb/experiments`）、84、85（`:14` `EXP_DIR=.claude/kb/experiments`）、86、88（`:54` `.claude/kb/**/*.md`）、99（`:33` `EXPERIMENTS=.claude/kb/experiments`）；不读的是 33、52（读 `.claude/kb/layout/01-first-txn.md`，不是实验页）、80、96（读 `.claude/kb/checks-owed.md`）。判决括注只列了会因「页还没写」判红的三道；另外七道在页写出来之前跑是空判（新实验的页不在，判不到它），照「凡是」一起挪。这是按判据字面扩的名单，不是判决括注的原样，第三轮可以攻。

## 二、G4 弹窗闸：自证改前、改后与证红

| 时点 | 自证（`bash <hook> --selftest`，原样） | 退出码 |
|---|---|---|
| 改前（仓里那一份） | `✓ 自检通过（查了 30 种）：…中文文件名没带行号、中文词后面跟冒号与数的照拒；四例走真实的 stdin 入口` | 0 |
| 只加 6 格、判法不改（草稿 `g4/cases-only/`） | 6 格 ✗，原样见下 | 1 |
| 改判法之后（草稿 `g4/final/`，装进仓后再跑一次同样） | `✓ 自检通过（查了 36 种）：带断言词、同一句里没有出处也没写「推的」的句子拒绝，只拒没出处的那一句；出处（反引号里的路径或命令、文件:行号（带扩展名的中文文件名也认）、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、反引号与「」里的断言词、「不一定」放行；中文文件名没带行号、中文词后面跟冒号与数、/ 后面是不带扩展名的汉字跟冒号与数的照拒；四例走真实的 stdin 入口` | 0 |

只加 6 格时的原样（`/tmp/claude-1000/defs-closeout-r2-fixes/g4/selftest-cases-only.txt`）：

```
  ✗ 自检：必拒：/ 后面是不带扩展名的中文，跟冒号与数（O3/O8两格：2） 应当是 ['O3/O8两格：2 格一定该升成打中']，实际 []
  ✗ 自检：必拒：/ 后面是数字跟汉字（54/55号：2） 应当是 ['54/55号：2 道一定要带前缀']，实际 []
  ✗ 自检：必拒：/ 后面是编号跟汉字（F1/F4两条：2） 应当是 ['F1/F4两条：2 处一定都要改']，实际 []
  ✗ 自检：必拒：/ 后面是汉字串（E142/第十六次跑：3） 应当是 ['E142/第十六次跑：3 个字段一定是 0']，实际 []
  ✗ 自检：必拒：ASCII 路径后面紧跟汉字（research/prompts下的判决：3） 应当是 ['research/prompts下的判决：3 条一定都对']，实际 []
  ✗ 自检：必拒：/ 前面是汉字夹数、后面是汉字（抓到40/无效：0） 应当是 ['抓到40/无效：0，这张表一定全抓了']，实际 []
    → 看 sentences_of() / without_code_and_quotations() / assertion_spans() / has_provenance() 的判法与 stdin 入口，改完再跑 --selftest
exit=1
```

6 格就是攻方 `cases-f13.json` 的 Q1–Q6 原句，每一格在旧判法上各自红了一次（上面 6 行 ✗），换判法后全绿。原有 30 格在新判法上照过：其中「必放：多层目录下中文文件名的文件:行号」「必放：不在反引号里、文件名是中文的文件:行号」「必放：不带 / 的中文文件名，全角冒号行号」三格钉住「带扩展名的中文文件名照认」，判法收窄成只认 ASCII 会让它们红。

装进仓之后用攻方的 `probe.py` 原样重喂 `cases-f13.json`（HEAD 那一版与今天这一版并排，只喂 JSON、只看退出码），原样（`probe/f13-after.txt`，末行检出 0 条）：

```
Q1-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q1-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q2-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q2-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q3-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q3-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q4-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q4-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q5-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q5-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q6-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q6-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q7-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q7-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q8-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q8-now	ask	主 agent	fg	exit=0	
Q9-head	ask-head	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
Q9-now	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
```

Q1–Q6 改前放（攻方报告 H4 节 `Q*-now exit=0`）、改后拒；Q8（中文文件名带扩展名与行号，F13 要放的那一形）照放；Q7、Q9 两版都拒。与攻方 `f13-fix-g1.sh` 在副本上的结果（Q1–Q7、Q9 exit=2，Q8 exit=0）一致。

## 三、G1：74、15 号的探针、假根演示与样本

### 三 a、喂 `heavy-test-guard.sh` 的探针（只喂 JSON、只看退出码，一条都没执行）

用攻方的 `probe.py`，用例 `/tmp/claude-1000/defs-closeout-r2-fixes/probe/cases-r2-fixes.json`，输出 `probe/r2-fixes-out.txt`，原样：

```
G1-01	heavy	gate-triage	fg	exit=0	
G1-02	heavy	gate-triage	fg	exit=0	
G1-03	heavy	gate-triage	fg	exit=2	✗ 重型测试被拒：gate.sh --staged（整轮门禁）：gate-triage 跑「整轮门禁」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
G1-04	heavy	gate-triage	fg	exit=0	
G1-05	heavy	gate-triage	fg	exit=0	
G1-06	heavy	implementation-writer	fg	exit=0	
G1-07	heavy	implementation-writer	fg	exit=0	
G1-08	heavy	crash-verifier	fg	exit=0	
G1-09	heavy	crash-verifier	fg	exit=0	
G1-10	heavy	主 agent	fg	exit=0	
G1-11	heavy	implementation-writer	fg	exit=0	
G1-12	heavy	implementation-writer	fg	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（implementation-writer 不经 run-with-memory-cap.sh）
G1-13	heavy	gate-triage	fg	exit=0	
G1-14	heavy	crash-verifier	fg	exit=0	
G1-15	heavy	gate-triage	fg	exit=0	
G1-16	heavy	gate-triage	fg	exit=0	
G6-01	detector	three-way-attack	rib	exit=0	
G6-02	detector	three-way-attack	fg	exit=0	
G6-03	heavy	three-way-attack	rib	exit=0	  ! heavy-test-guard.sh：bash / sh 起的脚本 /tmp/claude-1000/defs-closeout-r2-fixes/probe/w/a.sh 不存在，里面的命令没判；这条命令照常执行，记了一条检出
```

命令（cwd 仓根）：G1-01 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（门禁分诊员还原后的第 2 步）；G1-02 同一条不带 `--staged`；G1-03 对照：不带前缀；G1-04 `nice -n 19 bash .claude/gate.d/15-research-build.sh`；G1-05 前面加 `GATE_RESEARCH_BUILD_MEMORY_MAX=12G`；G1-06、G1-08 `nice -n 19 bash .claude/gate.d/74-model-differential.sh`（实现员、崩溃验证员）；G1-07、G1-09 前面加 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`；G1-10 主 agent 直接跑 74 号；G1-11 74 号普通判红时打印的单跑命令 `bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture`；G1-12 对照：同一条不经包装；G1-13 `bash research/scripts/capped.sh 8 bash .claude/gate.d/15-research-build.sh`；G1-14 崩溃验证员 1b 的 54 号写法（没变，核一次）；G1-15 分诊员单跑 87 号；G1-16 F9 那一版的整条包装写法（hook 照放，定义里已经不叫人这么跑）。G6-01/02/03：3c 新写法（`rm -f …/b2-*.rc; { …; echo "$?" > …/b2-1.rc; } & { …; echo "$?" > …/b2-2.rc; } & wait; n=$(ls …/b2-*.rc | wc -l); if [ "$n" -ne 2 ]; then echo void; else cat …; fi`）在 run_in_background 与前台喂检出 hook、在 run_in_background 喂重型闸，都放行；G6-03 那一行 `!` 是 `a.sh` 在探针里不存在，只记检出。检出记录 3 条（G1-03、G1-12、G6-03 各一条），在 `probe/probe-detections-63sz6yhu.jsonl`。

重型闸按名字判门禁阶段、不读正文（`.claude/rules/implementation-workflow.md:59`），所以 74、15 号阶段里面加了包装不改变它对这两道的判定：两道对各身份照旧放行。

### 三 b、改后的两道阶段在假根上跑（经包装那一支的三种结局）

两道都没在仓上真跑（原因见第五节）。改用两个只有几行 Rust 的假根：`fake74`（工作区成员 `crates/singlefs-harness`，测试目标就叫 `second_transaction_supplement_three_random_history`，打出六段标题与「模型对拍 N 步」）与 `fake15`（`research/` 下一个带两个单测的 crate）；两者的测试在设了 `FAKE_ALLOCATE_MIB` 时先摸满那么多内存。先 `cargo test --offline --release … --no-run` 只编不跑；阶段从一个镜像目录起（`mirror/.claude/gate.d/` 放改后的两份脚本，`mirror/research/scripts` 是指到仓里 `research/scripts` 的符号链接，这样 `$(dirname "$0")/../..` 取到的包装就是仓里那一份），峰值表指到草稿目录里的私有一份（`RUN_WITH_MEMORY_CAP_PEAKS`，不碰主仓的 `memory-peaks.tsv`），`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=300`。假根的全部源码存在 `g1/fake-roots-sources.txt`。

绿（默认上限）：

```
== 74 green (default cap)
  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）：
      随机历史快档：模型对拍 10 步：假根替身
      随机历史：偏向抬 F 之后复用的取样点：模型对拍 11 步：假根替身
      随机历史：偏向抬 F 之后回退的取样点：模型对拍 12 步：假根替身
      随机历史：越过原分配记录墙的取样点：模型对拍 13 步：假根替身
      随机历史：小盘上逼近单元区墙的取样点：模型对拍 14 步：假根替身
      随机历史：小盘上逼近单元区墙的取样点（空间准入判着）：模型对拍 15 步：假根替身
exit=0
== 15 green (default cap)
  ✓ research 构建通过，2 个测试批次、共 2 个单测全绿
exit=0
== peaks
9437184	2026-09-26	8G	cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture
28311552	2026-09-26	8G	gate 15-research-build: cargo test --release (research)
```

私有峰值表里两行的上限列都是 8G、键是设计的那两个：两条 cargo 确实在包装里、按默认上限跑的。

红一：撞上限（`FAKE_ALLOCATE_MIB=400`，上限 200M）：

```
== 74 red (cap 200M, allocates 400 MiB)
    Finished `release` profile [optimized] target(s) in 0.00s
     Running tests/second_transaction_supplement_three_random_history.rs (target/release/deps/second_transaction_supplement_three_random_history-c6b70fa26f432660)

running 1 test
error: test failed, to rerun pass `-p singlefs-harness --test second_transaction_supplement_three_random_history`

Caused by:
  process didn't exit successfully: `/tmp/claude-1000/defs-closeout-r2-fixes/g1/fake74/target/release/deps/second_transaction_supplement_three_random_history-c6b70fa26f432660 --nocapture` (signal: 9, SIGKILL: kill)
run-with-memory-cap: 撞了内存上限 200M（scope singlefs-memory-cap-1468639-87326325 的 Result=oom-kill，命令退出码 101）
  ✗ 随机历史的测试二进制没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 250（上限 200M），这一次的输出不算判定
     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 设大再跑；
                251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。
exit=1
== 15 red (cap 200M, allocates 400 MiB)
         Finished `release` profile [optimized] target(s) in 0.00s
          Running unittests src/lib.rs (target/release/deps/fake_research-073f73e920db56df)
     
     running 2 tests
     test tests::second_test_is_counted ... ok
     run-with-memory-cap: 撞了内存上限 200M（scope singlefs-memory-cap-1468695-1038813893 的 Result=oom-kill，命令退出码 143）
  ✗ research 的构建与单测没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 250（上限 200M），这一次的输出不算判定
     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；
               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。
exit=1
```

红二：测试自己失败（`FAKE_ALLOCATE_MIB=not-a-number`，解析 panic，cargo 退 101，走原来那一支；74 号只贴了末 6 行）：

```
== 74 red (test panics)

error: test failed, to rerun pass `-p singlefs-harness --test second_transaction_supplement_three_random_history`
  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
     → 怎么办：单跑看细节（经内存包装，上限同这一道）：
                bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture
                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。
exit=1
== 15 red (test panics)
  ✗ research 的构建或单测没过（cargo test 退出码 101）
     test tests::allocates_when_asked ... FAILED
     thread 'tests::allocates_when_asked' (1469116) panicked at src/lib.rs:7:54:
     test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     error: test failed, to rerun pass `--lib`
     → 怎么办：先修好——kb 里的实测结论全靠它们背书，编不过就等于那些数字今天没有来源。
exit=1
```

251、252、253、254 这四个码没在假根上造出来（要让 systemd-run 起不来、让 slice 占满或设限时），它们与 250 走同一个分支（74 号 `case 250|251|252|253|254`、15 号 `(( rc >= 250 && rc <= 254 ))`），这一格是按代码推的。

### 三 c、74 号的红绿样本

`stage-selftest.sh` 整道会把 `.claude/gate.d/` 下每个有样本的阶段都跑一遍，没整道跑；照它对单个样本做的几步（拷进临时目录、清掉 `GATE_BASE` 三个变量、以样本目录为根跑阶段、比退出码与 `want=` 行）写了 `g1/run-fixture.sh`，只跑 74 号的两份样本。改后的脚本在镜像目录里跑一次、装进仓后再跑一次，两次原样相同：

```
74-model-differential.sh/green: exit=0 ok
74-model-differential.sh/red: exit=1 ok
```

样本走「没有 Cargo.toml、有录好的输出」那一支，碰不到经包装的 cargo 那一支，所以样本没改；那一支靠第三 b 节的假根演示。15 号没有样本目录、也没有 `--selftest`（`ls .claude/gate.d/fixtures/ | grep '^15'` 空），没有可单跑的自证；假根演示是它唯一跑过的一次。

## 四、门禁判定行（原样，逐道跑，都在最后一处改动之后，`nice -n 19`；日志在 `gates/`）

| 门禁 | 原样判定行 | 退出码 |
|---|---|---|
| 47 | ✓ research 脚本的自证都通过（本阶段跑了 31 条；research/scripts/ 里声称有 --selftest 的 35 份中 34 份有门禁阶段在跑）<br>没跑的 1 份（登记在本阶段的 NOT_RUN_HERE）：research/scripts/vm-bench.sh：自证要连起三次虚机，挂钟太重，不进每轮门禁 | 0 |
| 62 | ✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent） | 0 |
| 63 | ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸与弹窗断言闸注册着、自证通过，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着、自证通过，书记官写入后的核对 hook 注册着、自证通过，表与定义一致（3 个有 Write 或 Edit 的定义、18 条路径模式），共用切词模块的 14 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 12 个文件），共用重型测试判定模块的 15 个函数只在 lib_heavy_tests.py 里定义（查了 .claude/hooks/ 下另外 12 个文件；selftest、load_sibling_module 不算，见 NOT_SHARED_JUDGMENT） | 0 |
| 73 | ✓ 门禁自检通过：84 个脚本（.sh 与 .py）、260 条拒绝都带了出路<br>✓ 查了 72 个脚本：.sh 都可执行，暂存区里的模式与工作区一致<br>✓ 进程安全：查了 168 个脚本（.sh 127 个、.py 41 个），发信号的写法都只打得到点名的一个进程；own-scope 标注放行 1 处（research/scripts/run-with-memory-cap.sh:952）；没判的：.claude/process-safety-pending 里的 2 个文件 research/scripts/agent-watch.py（research/scripts/agent-watch.py:2162）；research/scripts/mutate.sh（research/scripts/mutate.sh:351）<br>✓ shell 纪律检查通过（共 34 个脚本）<br>✓ shell 纪律检查通过（共 8 个脚本）<br>✓ shell 纪律检查通过（共 9 个脚本） | 0 |
| doc-lint（文档铁律，`bash .claude/singlefs-ai-sop/scripts/doc-lint.sh <仓根>`） | ✓ 文档铁律检查通过（检查 485，跳过 0；DOC_LINT_VERBOSE=1 看全部） | 0 |
| 规则纪律（项目本地；照 `gate.sh` 的 `run_rules_lint` 起：`GATE_IN_STAGE=1 RULES_LINT_DIR=<仓根>/.claude/rules RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md .claude/skills/*/SKILL.md"`） | ✓ 规则只写怎么做（扫了 29 份文件 1793 行；没扫 0 个；记录小节 0、论证小节 0、带日期的行 0（另有 9 行的日期只在「」或反引号里）、解释性段落 0、解释性半句 0、没带劝阻句的链接 0；词法说明判了 1670 行（围栏与表格行不判），命中 0；使用者名字这一条无对象可判：被扫的仓没有 I18N 或没登记 consumers=） | 0 |
| gate-overlap（门禁查重） | ✓ 相对 97f5904b44cd：新加的门禁与钩子 1 个（.claude/gate.d/84-verdict-false-named.sh）都写明了比过谁，改过的 15 份脚本对照已有的 132 份没有整段相同<br>没判的：装进来的 SOP 副本 37 份只当对照（.claude/singlefs-ai-sop/scripts） | 0 |

63 号那一行里「弹窗断言闸……自证通过」就是改后弹窗闸的 36 格自证。73 号的 gate-lint 管到改后 74、15 号新加的两处 ✗：每处后面都紧跟 → 行。

## 五、没做的与原因

- **74、15 号没在仓上真跑**。15 号是整个 research 工作区 `cargo test --release`（阶段归属表写它「全工作区十几分钟」，按量级是全量测试），不跑；74 号要 release 编整个 `crates/`，这一轮只验了改动那一支（第三 b 节假根上的绿、撞顶、测试失败三种结局）与样本（第三 c 节）。真跑一次归提交时的崩溃验证员 / 门禁分诊员。
- **默认上限 8G 对 15 号没量过**：峰值表里没有 research 整个工作区 `cargo test --release` 的行（`grep` 键含 `research` 的行都是单个 bin 或 `-p singlefs-harness` 的），推的；判决 G1 定的默认就是 `REPLAY_MEMORY_CAP` 的默认值，照写。74 号整道经包装跑过的峰值 4.41–4.70 GiB（攻方报告 H1 节峰值表三行，含编译），在 8G 以下。
- **G1 里「`check.sh` 的 `cargo test --all` 报给 SOP 会话、记进 `records/2026-09-16-subagent拆分提案.md` 第四十节」与 G3 里「派发闸误拒记进第四节」**：`records/` 与上游 SOP 不在放行文件里，归主 agent。
- **包装文件头 `research/scripts/run-with-memory-cap.sh:72`** 讲嵌套时仍拿「整条 gate.sh 经它跑」当例子；机制描述本身还对，但 G1 之后没有定义再叫人这么跑。那份不在放行文件里，没改。
- **`.claude/gate.d/stage-inputs.tsv` 的 74 号那一行**（`crates/ Cargo.toml Cargo.lock`）没把 `research/scripts/run-with-memory-cap.sh` 列进输入：阶段现在读它，包装改了不会让 74 号的复用判定失效。不在放行文件里，没改（推的影响：包装改坏时 74 号可能按「输入没变」复用旧绿）。
- **攻方 H2 附带的一格**（40 号把 `*.r[0-9].out` 当逐轮中间件跳过，G2 之后新文件常写成 `rN`）不在 G1–G9 里，没动。
- **重型闸对 F9 那种整条包装写法照放**（第三 a 节 G1-16 exit=0）：定义里已经不这么写，闸没拦它；闸不在放行文件里。
- **`bash-command-detector.sh` 自己给 ④ 的拒绝出路**没跟着 G6 改（不在放行文件里）：`bash-command-detector.sh:2395–2397` 的出路只写「几条活要并行就在同一条命令里 `a & b & wait`，用不带参数的 `wait` 等齐」，不提退出码文件、批号件号与数件数。
- **G7 名单**是照判据「凡是读实验页的」现查扩出来的十道，不是判决括注里的三道（第一节表下）。
- 没跑重型测试（54、55、57、59、87、层 0、QEMU、herd7、全量 cargo test、整轮门禁）；没跑 72 号（判形式的是主 agent 写的判决，归主 agent）；`stage-selftest.sh` 整道没跑，只跑了 74 号两份样本。

## 六、草稿与清理

草稿目录 `/tmp/claude-1000/defs-closeout-r2-fixes/`：`before/`（改前备份）、`start-sha256.txt`、`end-sha256.txt`、`defs.diff`（只含八份定义）、`my-changes-final.diff`、`g4/`（改法脚本 `edit_hook.py`、只加格与改判法两份弹窗闸拷贝、三份自证输出）、`g1/`（改法脚本 `edit_stages.py`、两份改后阶段脚本、`run-fixture.sh`、私有峰值表 `peaks.tsv`、假根源码 `fake-roots-sources.txt`、`fake-roots-du.txt`）、`probe/`（探针用例与输出、两份检出记录、`probe.py` 建的 HEAD 版弹窗闸拷贝目录 `probe-ask-head-u0cy62ik/`）、`gates/`（七道门禁的日志、起止时刻与退出码）。
删了的：假根 `/tmp/claude-1000/defs-closeout-r2-fixes/g1/fake74`（1.2M，含它的 `target/`）与 `/tmp/claude-1000/defs-closeout-r2-fixes/g1/fake15`（1.2M，含 `research/target/`），删前 `du -sh` 记在 `g1/fake-roots-du.txt`；镜像目录 `g1/mirror`（两份脚本拷贝加一个指到仓里 `research/scripts` 的符号链接，`rm -rf` 只删链接本身，仓里的目录还在）。
中途有一次交回调用是误发的，被交回闸拒掉（点名的正是这两个假根），没有送达；这一份是唯一送达的交回。

## 附：改后全量 diff（相对开工时的备份，`my-changes-final.diff` 原样）

```diff
--- a/.claude/agents/three-way-attack.md
+++ b/.claude/agents/three-way-attack.md
@@ -30,12 +30,12 @@
    - 盘用内存稀疏盘（`crates/singlefs-harness/src/crash.rs` 的 `SparseBlockDevice`），不在磁盘上建镜像文件；每个起点状态只建一次池（mkfs 与起点历史只跑一次），之后每段历史从内存里那一份拷（`SparseDevice`、`MemoryPool` 都能 `clone`）。
    - 正式跑之前先跑一小段，按它估全量的挂钟；估出来超过 40 分钟，先缩历史、候选与几何的取样，报告里写明缩了什么、缩前缩后各多少、估时怎么算的。
    - 崩溃状态不在缩的范围里（共用约束「不做」一节「崩溃点测试不衡量时间成本」那一条），照层 0 的枚举域取：调同一份文件里每一段都展开的 `enumerate_layer0` / `enumerate_layer0_versions`（录制写流，屏障与 FUA 切段，段内写的整写子集逐个枚举，不是只截前缀），每个崩溃状态恢复之后都跑 checker。缩只缩历史条数与长度、候选与几何的取样；留下的每段历史，它的崩溃状态一个不落。
-   - 在自己的原型里这样调枚举函数跑小流不算重型：测试目标的名字不带 `layer0`（带了 `heavy-test-guard.sh` 按名字拒）；每次跑之前用同一份文件的 `closed_form_state_count` 算出这一次全量的状态数，不超过约 10⁶ 个（几段历史合起来超过的，分几次跑）；一段历史自己就超过的，那段不跑，写进报告交主 agent。
+   - 在自己的原型里这样调枚举函数跑小流不算重型：原型里的流只许在原型里自己造，不从名字带 `layer0` 的用例里拷（拷文件、拷代码段、`include!`、`mod` 引进来都算拷）；测试目标的名字不带 `layer0`（带了 `heavy-test-guard.sh` 按名字拒）；每次跑之前用同一份文件的 `closed_form_state_count` 算出这一次全量的状态数，不超过约 10⁶ 个（几段历史合起来超过的，分几次跑）；这一轮全部原型跑的全量合起来不超过约 10⁷ 个状态。一段历史自己就超过约 10⁶ 的，那段不跑；再跑就要越过约 10⁷ 的，剩下的不跑；两样都写进报告交主 agent。
    - 随机跑批分小批：每批的段数按先跑一小段估出的时长定，不给批设限时（包装的 `RUN_WITH_MEMORY_CAP_TIME_LIMIT`、外面套 `timeout` 都不用）；每批的段数与估时写进报告。
 3c. 内存与进程：
    - 编译与跑都经内存包装：跑的照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条，`cargo build` 也经它（`bash research/scripts/run-with-memory-cap.sh <上限> cargo build …`）。
    - 只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（共用约束「执行前拒绝的写法」那一条的 ⑥）。
-   - 并行起的几件，每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <草稿目录>/<名字>.rc; } &`），最后单独一个不带参数的 `wait` 等齐，再按起的次序逐个读 `.rc` 文件（共用约束「执行前拒绝的写法」那一条的 ④）；不用 `wait "$pid"` 收退出码。
+   - 并行起的几件，每件把退出码写进自己的文件，文件名带批号与件号（`{ <命令>; echo "$?" > <草稿目录>/b<批号>-<件号>.rc; } &`），每批开跑前先删掉这一批的 `.rc`（`rm -f <草稿目录>/b<批号>-*.rc`）；最后单独一个不带参数的 `wait` 等齐，数一遍这一批的 `.rc`，文件数与派出去的件数对不上，整批作废；对得上再按起的次序逐个读（共用约束「执行前拒绝的写法」那一条的 ④）；不用 `wait "$pid"` 收退出码。
    - 等一行字之前先确认那一行真会写进那个文件。
    - 不改正在跑的脚本，要改的写同目录临时文件再 `mv` 换上（共用约束「执行前拒绝的写法」那一条的 ⑦）。
 4. 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
--- a/.claude/agents/three-way-local-defense.md
+++ b/.claude/agents/three-way-local-defense.md
@@ -16,7 +16,7 @@
 - 立场是辩方：替被判出局、被攻、或被计划排除的一方，找它站得住的理由，写出具体机制（哪个文件、哪一步）；站不住就说站不住、为什么。辩护落得成数的（这一方在某段历史上是几、判红没有），同样按攻方第 1 步给事实表、要模型逐格填。提示里先替每一条写一个攻方问题（它被质疑的那一句，整行抄出处），再让本地模型逐条辩护；攻方问题同样要逐句核转述。
 - 输入里的「攻击面」换成「要辩护的一方」（主 agent 给：被复核的判决或被排除的候选，原文路径）。
 - 行号同攻方第 1 步：提示里明令答复不写行号，样本自带的行号在运行记录里标「模型自给、未核」。
-- 文件名形态（攻方定义里出现的 `-local-attack` 文件名，不论在哪一节，一律照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，样本 `research/prompts/<轮>-local-defense-output-s<n>.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。
+- 文件名形态（攻方定义里出现的 `-local-attack` 文件名，不论在哪一节，一律照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。样本照攻方写成 `<前缀>-output-s<n>.md`，前缀取派发提示给的（与攻方「输入」那一项同），不写死。
 
 ## 没做什么（固定会有的）
 
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -28,12 +28,12 @@
 2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
 3. 写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
-4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名（带今天的日期或 `rN`，写之前 `ls` 确认没有同名的），跑完按这个新文件名认本轮才有的完成标记。重跑已有实验时拿新文件与已有的那份比：逐字节一致就删掉这一次的新文件、不新存；对不上就两份都留，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
+4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名（带今天的日期或 `rN`，写之前 `ls` 确认没有同名的），跑完按这个新文件名认本轮才有的完成标记。这一次的新文件也不删：重跑已有实验时，第 5 步把 `research/scripts/replay.sh` 里这个实验的登记行改指新文件，旧的留着；拿新文件与已有的那份比，逐字节一致的在报告里写明，对不上的在实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
 4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
-4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（逐字节一致没新存的，读 `research/results/` 里那一份；输出不截断），任何字段取值是 `false` 或 `not_run`，或字段名表示违例、不匹配、歧义、失败的计数（名字里带 `violation`、`mismatch`、`ambiguous`、`fail` 的，例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）取值不是 0，在报告里逐个点名：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类计数都是 0」。
+4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（输出不截断），在报告里逐个点名其中表示「没过」的字段：取值是布尔 `false` 的；取值是 `not_run` 的；取值是大于 0 的整数、而名字表示违例、不匹配、歧义、失败的计数（例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）。取值是布尔 `true` 的、取值是 `zero` 的（例 `control_violations_ok=true`、`layer0_violations=zero`）与名字不表示「没过」的计数（例 `journal_differing_states`）不点名；拿不准的点名，写「拿不准」。每个点名写：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类的整数计数都是 0」。
 5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
-6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节），84 号除外：它放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
-7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。写完（连同第 7b 步）跑 84 号，贴原样末行与退出码。
+6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节），读实验页或实验索引的那几道除外（27、34、40、69、75、84、85、86、88、99 号）：它们放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写，产物照第 4 步一个不删（86 号出路里「把 research 下那些文件删掉」那一句不照做，写进报告交主 agent）。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
+7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。写完（连同第 7b 步）跑第 6 步留下的那几道，各贴原样末行与退出码。
 7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
 
 ## 写范围
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -17,13 +17,13 @@
 
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
-- 54、55、57 号各自的内存上限（第 1b 步用）；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
+- 54、55、57 号各自的内存上限（第 1b 步用），每道一个数：54、57 号的写明是量过的（峰值表 `research/scripts/memory-peaks.tsv` 里那一道整条经包装跑出的峰值）还是推的；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
-1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，照第 2 步直接跑，外面不再包一层。
+1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，74 号在阶段里面经包装，这两道照第 2 步直接跑，外面不再包一层。
 2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -18,14 +18,13 @@
 - 这一轮的改动已经按 `research/scripts/stage-mine.py` 暂存了没有（没暂存就不派你，或者主 agent 明写「跑全量工作区」）。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - 这一轮暂存区 diff 的范围：`git diff --cached --stat` 原样。主 agent 明写跑全量工作区时，暂存区多半是空的，这时主 agent 另给「这一轮改过的文件清单」，归属按清单判。
-- `gate.sh` 整条经内存包装的上限（第 1b 步用）。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
-1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑。`gate.sh` 整条经内存包装跑，上限取输入给的，退出码 250–254 照那一条办；里面 59 号、87 号逐条再经包装的照常跑（嵌套怎么排队见 `research/scripts/run-with-memory-cap.sh` 文件头「包装里再经包装跑的」那一句）。单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了）。
-2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑同一条、去掉 `--staged`（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
+1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了，15、74 号在阶段里面经包装）。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -23,7 +23,7 @@
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载。
-1b. 第 3 步的证红、第 4 步动到的测试二进制（副本里跑的也算）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`cargo fmt --check`、`cargo clippy`、`cargo build` 不经它。
+1b. 第 3 步的证红、第 4 步动到的测试二进制（副本里跑的也算）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`cargo fmt --check`、`cargo clippy`、`cargo build` 不经它。登记给你的 74 号在阶段里面经包装，照跑，外面不再包一层。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
 4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`、阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内），各贴末尾原样输出。其余门禁阶段、全量 `cargo test --all`、层 0 各流的快档与全量都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -45,7 +45,7 @@
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
 - 重型测试（名字含 `layer0` 的测试二进制与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
-- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
+- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，门禁 15、74 号在阶段里面经包装（派发提示给了上限的，设进阶段文件头写的变量：15 号 `GATE_RESEARCH_BUILD_MEMORY_MAX`，74 号 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX`），都直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
 - 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
 - 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。
@@ -61,7 +61,7 @@
     ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
     ② 前台没有超时的等待循环：`until` / `while` 里有 `sleep`，外面没套 `timeout`。要等就用 run_in_background 起、结束本轮等完成通知，或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。
     ③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。
-    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；不用 `wait "$pid"` 收。
+    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`，文件名带批号与件号，每批开跑前先删掉这一批的文件），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；读之前数一遍，文件数与派出去的件数对不上，整批作废；不用 `wait "$pid"` 收。
     ⑤ 整份覆盖 `research/results/` 下已存在又没进 git 的产物：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 的目标、`dd of=`、`truncate`。追加（`>>`、`tee -a`）与新文件名不拦；要换就按日期另存新文件名，旧的留着。
     ⑥ 终止进程不是点名一个自己起的进程号或任务号：`kill` 的目标带负号、是 0 或 `$PPID`、一次给几个、是命令替换或通配，在循环里逐个发（`proc.py stop` 同样），按名字、按 cgroup 或 `/proc` 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，`systemctl` 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`。
     ⑦ 在同一个 inode 上改已经存在的脚本（`.sh`、`.py` 或带执行位的文件，在仓里或 `/tmp/claude-1000/` 下）：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` 的目标、`dd of=`、`truncate`、python 的 `open(…, 'w')` 这一类。改脚本写到同目录临时文件再 `mv` 换上，或用 `research/scripts/replace-once.py` / `insert-row.py` 定点改；追加与新建不拦。
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -45,7 +45,7 @@
 
 同时在跑的重活（编译、测试、变异、产物、原型扫描）不止一个时，每份派发提示写一行「线程上限：N」，N = 整机核数 ÷ 同时在跑的重活数（向下取整，至少 1）；重活数变了，给在跑的 agent 发消息改 N。
 
-派崩溃验证员时给 54、55、57 号各自的内存上限，55 号的不小于同时起的虚机数乘每台的内存（算法在 `.claude/agents/crash-verifier.md`「输入」一节）；派门禁分诊员时给 `gate.sh` 整条经内存包装的上限。
+派崩溃验证员时给 54、55、57 号各自的内存上限，每道一个数，照 `.claude/agents/crash-verifier.md`「输入」一节给：54、57 号写明是量过的还是推的，55 号的不小于同时起的虚机数乘每台的内存。门禁 15、74 号在阶段里面经内存包装，要换它们的上限就在派发提示里给一个数，没给用 `research/scripts/replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值。
 
 在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。
 
--- a/.claude/hooks/ask-user-claim-guard.sh
+++ b/.claude/hooks/ask-user-claim-guard.sh
@@ -19,8 +19,8 @@
 #   ② 句子里有「推的」「没量过」「推测」「估计」「粗估」之一的放行。
 #   ③ 句子里有出处的放行，出处是下面任一样（在原句上找，引号与反引号里的也算）：
 #      反引号里的路径（带 / 的，或带登记过的扩展名 PATH_EXTENSIONS 的文件名）或命令（全 ASCII、至少两段，第一段是小写命令名、
-#      ./ 开头的路径或 VAR= 赋值）；文件:行号（文件带 / 或带登记过的扩展名，冒号全角半角都认；最后一个 / 之后的文件名里认汉字这类非 ASCII 的字，
-#      / 前面紧挨着的那个字要是 ASCII）；research/results/ 下的文件名；
+#      ./ 开头的路径或 VAR= 赋值）；文件:行号（冒号全角半角都认；文件带 / 的，/ 前面紧挨着的那个字要是 ASCII，最后一个 / 之后要么全是 ASCII，
+#      要么以登记过的扩展名收尾、名字里可以有汉字这类非 ASCII 的字；不带 / 的要以登记过的扩展名收尾）；research/results/ 下的文件名；
 #      name= 开头的产物行；「实测」「量过」「产物」「输出」前后 MEASUREMENT_WINDOW 个字以内带着一个数或路径，
 #      而且那个数或路径与这个词之间没有隔着断言词（「输出一定为 0」里 0 与「输出」之间隔着「一定」，不算）。
 #   ①有、②③都没有的句子逐句列出，退出 2；一句都没有就放行。stdin 读不出 JSON、questions 不是列表的放行。
@@ -46,7 +46,8 @@
 PATH_EXTENSIONS = ("rs", "md", "sh", "py", "tsv", "csv", "out", "txt", "log", "json", "jsonl", "toml", "yaml", "yml",
                    "conf", "litmus", "cat", "diff", "patch", "lock", "img")
 ASCII_PATH_CHARACTER = r"[A-Za-z0-9_.-]"
-# 文件名里的字：ASCII 之外还认汉字这类非 ASCII 的字（\w 在 str 上认 Unicode 字母与数字，不认，。：（）「」这类标点）；目录名只认 ASCII
+# 文件名里的字：ASCII 之外还认汉字这类非 ASCII 的字（\w 在 str 上认 Unicode 字母与数字，不认，。：（）「」这类标点）；目录名只认 ASCII。
+# 带汉字的文件名只在以登记过的扩展名收尾时才算（FILE_AND_LINE）：「O3/O8两格：2」这类「/ 后面是汉字跟冒号与数」的不是文件:行号
 FILE_NAME_CHARACTER = r"[\w.-]"
 EXTENSION_ALTERNATION = "|".join(PATH_EXTENSIONS)
 # 路径：带一个 /，或以登记过的扩展名收尾的文件名
@@ -54,7 +55,7 @@
     rf"{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{ASCII_PATH_CHARACTER}+"
     rf"|(?<![A-Za-z0-9_.-])[A-Za-z0-9_-]{ASCII_PATH_CHARACTER}*\.(?:{EXTENSION_ALTERNATION})(?![A-Za-z0-9])")
 FILE_AND_LINE = re.compile(
-    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{FILE_NAME_CHARACTER}+"
+    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/(?:{ASCII_PATH_CHARACTER}+|{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))"
     rf"|[\w-]{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))[:：]\d+")
 RESULTS_FILE = re.compile(rf"research/results/{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]")
 PRODUCT_LINE = re.compile(r"(?<![A-Za-z0-9_])name=[^\s`'\"]")
@@ -202,6 +203,18 @@
          ["拆分提案第四十节：40 行写着它一定要走三方"]),
         ("必拒：斜杠分开的中文词跟冒号与数不是文件:行号", {"questions": [{"question": "抓到/无效/没红：40/0/0，这张表一定全抓了", "options": []}]},
          ["抓到/无效/没红：40/0/0，这张表一定全抓了"]),
+        ("必拒：/ 后面是不带扩展名的中文，跟冒号与数（O3/O8两格：2）", {"questions": [{"question": "O3/O8两格：2 格一定该升成打中", "options": []}]},
+         ["O3/O8两格：2 格一定该升成打中"]),
+        ("必拒：/ 后面是数字跟汉字（54/55号：2）", {"questions": [{"question": "54/55号：2 道一定要带前缀", "options": []}]},
+         ["54/55号：2 道一定要带前缀"]),
+        ("必拒：/ 后面是编号跟汉字（F1/F4两条：2）", {"questions": [{"question": "F1/F4两条：2 处一定都要改", "options": []}]},
+         ["F1/F4两条：2 处一定都要改"]),
+        ("必拒：/ 后面是汉字串（E142/第十六次跑：3）", {"questions": [{"question": "E142/第十六次跑：3 个字段一定是 0", "options": []}]},
+         ["E142/第十六次跑：3 个字段一定是 0"]),
+        ("必拒：ASCII 路径后面紧跟汉字（research/prompts下的判决：3）", {"questions": [{"question": "research/prompts下的判决：3 条一定都对", "options": []}]},
+         ["research/prompts下的判决：3 条一定都对"]),
+        ("必拒：/ 前面是汉字夹数、后面是汉字（抓到40/无效：0）", {"questions": [{"question": "抓到40/无效：0，这张表一定全抓了", "options": []}]},
+         ["抓到40/无效：0，这张表一定全抓了"]),
         ("必放：没有 questions", {}, []),
     ]
     results = []
@@ -225,8 +238,8 @@
         print("    → 看 sentences_of() / without_code_and_quotations() / assertion_spans() / has_provenance() 的判法与 stdin 入口，改完再跑 --selftest")
         return 1
     print(f"  ✓ 自检通过（查了 {len(results)} 种）：带断言词、同一句里没有出处也没写「推的」的句子拒绝，只拒没出处的那一句；"
-          "出处（反引号里的路径或命令、文件:行号（文件名是中文的也认）、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、"
-          "反引号与「」里的断言词、「不一定」放行；中文文件名没带行号、中文词后面跟冒号与数的照拒；四例走真实的 stdin 入口")
+          "出处（反引号里的路径或命令、文件:行号（带扩展名的中文文件名也认）、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、"
+          "反引号与「」里的断言词、「不一定」放行；中文文件名没带行号、中文词后面跟冒号与数、/ 后面是不带扩展名的汉字跟冒号与数的照拒；四例走真实的 stdin 入口")
     return 0
 
 def main():
--- a/.claude/gate.d/74-model-differential.sh
+++ b/.claude/gate.d/74-model-differential.sh
@@ -36,17 +36,36 @@
 fi
 
 TEST_BINARY="second_transaction_supplement_three_random_history"
+# 这里的 cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
+# 谁跑这一道都一样，外面不再包一层。上限取 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
+# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。包装自己的结局（退出码 250–254）不是测试的判定，单独判红、单独给出路。
+MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
+MODEL_DIFFERENTIAL_MEMORY_MAX="${GATE_MODEL_DIFFERENTIAL_MEMORY_MAX:-8G}"
 SECTIONS=("随机历史快档" "随机历史：偏向抬 F 之后复用的取样点" "随机历史：偏向抬 F 之后回退的取样点" "随机历史：越过原分配记录墙的取样点" "随机历史：小盘上逼近单元区墙的取样点" "随机历史：小盘上逼近单元区墙的取样点（空间准入判着）")
 
 log="$(mktemp)"
 if [[ -f Cargo.toml && -d crates/singlefs-harness ]]; then
-  if ! cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
+  if bash "$MEMORY_CAP_RUNNER" "$MODEL_DIFFERENTIAL_MEMORY_MAX" cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
+    cargo_exit=0
+  else
+    cargo_exit=$?
+  fi
+  if (( cargo_exit != 0 )); then
     tail -40 "$log"
-    echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
-    echo "     → 怎么办：单跑看细节（经内存包装，上限取派发提示给的，没给就用 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G）："
-    echo "                bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
-    echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
     rm -f "$log"
+    case "$cargo_exit" in
+      250|251|252|253|254)
+        echo "  ✗ 随机历史的测试二进制没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $cargo_exit（上限 $MODEL_DIFFERENTIAL_MEMORY_MAX），这一次的输出不算判定"
+        echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX 设大再跑；"
+        echo "                251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。"
+        ;;
+      *)
+        echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
+        echo "     → 怎么办：单跑看细节（经内存包装，上限同这一道）："
+        echo "                bash research/scripts/run-with-memory-cap.sh $MODEL_DIFFERENTIAL_MEMORY_MAX cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
+        echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
+        ;;
+    esac
     exit 1
   fi
 elif [[ -f model-differential-cargo-output.log ]]; then
--- a/.claude/gate.d/15-research-build.sh
+++ b/.claude/gate.d/15-research-build.sh
@@ -22,8 +22,22 @@
   echo "               或把已装的 cargo 放进 PATH。跳过这一步等于 research/ 的数字没人验过。"
   exit 1; }
 
-out="$(cd "$R" && cargo test --release 2>&1)"
+# 这里的 cargo test 跑编译出来的代码，经 research/scripts/run-with-memory-cap.sh 放进内存上限里跑（.claude/agent-common.md「跑编译出来的代码经内存包装」那一条），
+# 谁跑这一道都一样，外面不再包一层。上限取 GATE_RESEARCH_BUILD_MEMORY_MAX（派发提示给了上限的，跑这一道的 agent 设进它），
+# 没设取 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G。峰值表的键写死：光看「cargo test --release」分不出是哪个工作区。
+# 包装自己的结局（退出码 250–254）不是构建与单测的判定，单独判红、单独给出路。
+MEMORY_CAP_RUNNER="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/run-with-memory-cap.sh"
+RESEARCH_BUILD_MEMORY_MAX="${GATE_RESEARCH_BUILD_MEMORY_MAX:-8G}"
+out="$(cd "$R" && RUN_WITH_MEMORY_CAP_KEY="gate 15-research-build: cargo test --release (research)" \
+  bash "$MEMORY_CAP_RUNNER" "$RESEARCH_BUILD_MEMORY_MAX" cargo test --release 2>&1)"
 rc=$?
+if (( rc >= 250 && rc <= 254 )); then
+  tail -8 <<<"$out" | sed 's/^/     /'
+  echo "  ✗ research 的构建与单测没判出结果：内存包装 research/scripts/run-with-memory-cap.sh 退 $rc（上限 $RESEARCH_BUILD_MEMORY_MAX），这一次的输出不算判定"
+  echo "     → 怎么办：照包装文件头「退出码」一段办：250 是这一道要的比上限多，把 GATE_RESEARCH_BUILD_MEMORY_MAX 设大再跑；"
+  echo "               251、252、254 先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再整道重跑；不拿掉包装跑。"
+  exit 1
+fi
 if (( rc != 0 )); then
   echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
   grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
```
