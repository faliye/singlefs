# m3-close-gov-b17-r3 攻方腿（Opus）：批 17 治理第三轮，攻前两轮站住的形态

日期：2026-10-24。攻击面：正文第三节 K1–K5 全部格。被判对象是主工作区现文：开工与写报告前两次 `sha256sum -c research/prompts/m3-close-gov-b17-r3-snapshot/judged-sha256.txt` 都是 12 行 OK、`kb-sha256.txt` 1 行 OK。不重复前两轮的角度：第二轮 H3-b 只量了「改 `lib_function_units.py` 键不变」、没造出判定翻面的行，这一轮专造判定翻面；第二轮 H2-a 量的是终端符号链接，这一轮只判两种改法各自的射程，不再造放行现场；第二轮 H4 量的是旧行使导入退 1，这一轮判出路收不收敛与那句依据的出处。

## 复跑

模型目录 `research/prompts/m3-close-gov-b17-r3-opus-model/`。复跑（工作目录随意，`<新的草稿目录>` 给一个新的；一趟约 5 分钟，超过子 agent 前台时限，我没整条跑过，分步各跑过，见「这条腿自己的限度」）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-gov-b17-r3-opus-model/tools/rerun.sh --repo <仓根> --frozen <仓根>/research/prompts/m3-close-gov-b17-r3-snapshot --work <新的草稿目录>
```

末尾两行 `name=rerun_compare file=k1-runs.txt result=identical`、`… k1-runs-long.txt result=identical`，再打真 crates/ 那一份（随 crates/ 现文变，只打不比）与 `name=rerun_summary identical=2 differs=0`。全是 python：假 cargo、假 llvm 工具、假内存包装取自 `scripts/mutation/crates-mutation-rows.py` 自证（`FAKE_CARGO`、`FAKE_LLVM_TOOL`、`Selftest`），不起真 cargo、不编译、不碰任何真判定库（库放在 `GATE_MUTATION_RECORDS_HOME` 指的草稿目录里）。文件 sha256（写报告时现算）：

```
39fc702d0d606cd0d4608f4edc541180f13455ee5eee974b23dbd3e60defee9a  k1_real_tree.py
0ab55d90be43dba30905655b547576e2e1577ff0e28b7ba154f014675264b3fc  k1_runs_long.sh
d8e2b5809c4d5c4e8adae403592efba8a4920cb674e9a61139b0a1e2b597b16b  k1_runs.sh
160e5f8a1790e93e5375130fc690ff7a08815207a81b6893730a95c53f898f7d  k1_timeline.py
404791c39b1c2419eca0501488df0154ab5f8ca3373affc7112a3aec47ee5145  make_copies.py
738b2cb3845332b80b650d41f578841633d87695a8df188de3f442532d362df9  out/k1-coverage-id-fix.diff
9fff0fe6a9b53776d9681f26a8c0d1e417f4ef437c3e92ab2a39e019a52a5f03  out/k1-lfu-edit.diff
e74e91877eb7ea36a6330330d894a9f9a378dbbb7c0fa484febef9d5d4dcf25c  out/k1-real-tree.txt
32d268ad36a98207ca90e2d12e269290c02f2a661921f1746e11efb608f1f5a9  out/k1-runs-long.txt
b60d7b6371437107cf5f755ac803998a7d9d1cb396c330c0df770d919ac5344d  out/k1-runs.txt
dda080c8025d8147cb8e4e0033c8140e62d785319e957d6cb3b203c035e5e1a9  tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 量的 / 推的 | 什么现象会推翻 |
|---|---|---|---|
| K1 文件级排除在 crates 变异判法上 | **打中，不成立**：`lib_function_units.py` 改一处切单元（Rust 单元末行多算一行，E1），判法单元指纹、行键、基线键、覆盖号四把键逐字不变，同一行变异第二趟的结局从「基线不绿、退 1」翻成「抓到、退 0」（假抓到） | 量的（副本、假 cargo，时间线 A/B/C）；真 crates/ 上 38 189 个单元指纹逐字不变、19 946 个 fn 单元里 878 个被落进别的单元（量的） | E1 那一类改动让任何一把键变；或覆盖记录不参与基线复用 |
| K1 附带：把两份放回判法单元指纹（改前的样子）也不够 | 打中不分辨臂的那一半：放回之后只修第一次换代，下一次源码改动又假抓到；要同时让覆盖号带判法单元指纹才在四趟里都对 | 量的（时间线 E/F/G）；改法零轮 | 覆盖号本来就带判法单元指纹 |
| K2 两种零轮改法 | 打中（推的，静态读）：「认符号链接时连旁边 `preflight.sh` 一起核」核到的只是入口按链接所在那棵树 source 的至少 6 份里的 1 份，cd 进那棵树之后执行的一概不核；够的层级等价于「命令词所在目录一级都不经符号链接」。「不认符号链接」若写成只看命令词本身是不是链接，目录符号链接照样放过；写成整条路径不经符号链接，误拒只在仓本身挂在符号链接路径下时出现，本机 `realpath` 等于仓路径 | 推的（只读源码、只核了一个路径事实） | 入口不再用 `dirname "${BASH_SOURCE[0]}"` 与 `cd "$(dirname "$0")/../.."` 定根；或改法实现成比整棵树 |
| K3 GPU 导入退 1 | 打中两处：① 出路第一支「重跑那一片、再重导」在持久库上不收敛（导入逐行核全部旧行，不看有没有被新行顶掉）；② 正文说的「CPU 那一侧旧行不退 1 的依据（第一轮 G3 判决那一句）」在第一轮判决里找不到，那句在驱动自证的一句提示里；那句理由放到 GPU 一侧同样成立。commit-stages 第 2b 步写了退 1、别的行已并进来、之后照起 ③，照它做不会误判成失败 | ① 推的（读 `plan_gpu_import`）；② 量的（`grep`） | `plan_gpu_import` 只核每把键最新的一行；或第一轮判决里有那句 |
| K4 修补二批 | 打中一处（小）：③ 补进去的实验复跑入口那一句（三处）写「`--list-items` 与登记为 light 的单独 `--selftest` 不算」，「单独」只挂在 `--selftest` 上、没挂在 `--list-items` 上，而 `classify` 对 `--list-items E165` 判重型——规则比 classify 松。没打中：补丁 3 份文件的每一处都对得上判决要的、没找到改出判决没要的东西；① 快照的补丁只记 3 份，`heavy-test-guard.sh` 文件头、`verification.md:35`、`lib_mutation_units.py:36` 在被判的树里（题面点名、judged-sha256 都 OK）而不在补丁文件里，是补丁作部分记录、不是缺改 | 量的 | `--list-items` 那一句改成「单独 `--list-items`」或补「后面带别的词算重型」 |
| K5 第二轮的数 | 能核的：961/496/465、1442、801、cite-check 引文数 53 现跑对上；159 现跑打出「159 格都对」但我的 235 秒时限先到、退 124。核不到的：13/14 倒推今天是 9/14（修补二批改了 5 份）、cite-check「对上 27、没判 81」今天是 19、89；辩方「八行」今天第一步就退 1（它 grep 的「两种排除理由不同」已被修补二批删掉）；攻方 identical=6 在 230 秒里没跑完 | 量的 | 在第二轮那一刻的树上（倒推不出的 5 份补上）重跑 |

## K1 文件级排除在 crates 变异判法上成不成立

结论：**不成立**。`lib_function_units.py` 不只「管怎么存、怎么算键」：它切出的单元边界决定覆盖落进哪几个单元，覆盖记录决定基线复不复用，基线复不复用决定一行是「抓到」还是「基线不绿」。而它的单元被排出判法单元指纹之后，这条路上没有任何一把键随它变。翻面的观测：同一份源码、同一份库、同一把行键，只换 `lib_function_units.py`，结局不同（下面时间线 A 对 B）。

三条路互不共享前提：第一条只读代码；第二条在真 crates/ 源码上调真 `executed_units`，不用假 cargo、不跑 `run_stage`；第三条在自造小仓上用假 cargo 跑整道 `run_stage`，不碰真 crates/。

### 第一条路：读代码定位（推的）

- 行键：`scripts/mutation/crates-mutation-rows.py:1284` 起算基线键「baseline_key = mutation_units.key_of_lines([("form", [BASELINE_KEY_FORM]), ("judging", [self.judging]), ("test-unit", [test.unit.fingerprint]),」——基线键带判法单元指纹与点名测试自己的单元指纹，不带闭包。
- 覆盖号：`scripts/mutation/crates-mutation-rows.py:1287`「coverage_id = mutation_units.key_of_lines([("form", [COVERAGE_KEY_FORM]), ("test-unit", [test.unit.fingerprint]),」——覆盖号连判法单元指纹都不带。
- 基线复用：`scripts/mutation/crates-mutation-rows.py:1378`「if executed_set_is_current(keys, coverage):」成立就复用，不成立再看 `:1380`「if record.get("test_closure") and record.get("test_closure") == components.test_closure:」。
- 覆盖落单元：`scripts/mutation/lib_mutation_units.py:553`「if not unit.data and unit.kind != "doctest" and unit.first_line <= line <= unit.last_line]」，`:559`「innermost = min(units, key=lambda unit: unit.last_line - unit.first_line)」——用的是 `lib_function_units.py` 给的行号区间。
- 覆盖不重记：`scripts/mutation/crates-mutation-rows.py:3821`「if executed_set_is_current(self.keys, coverage):」成立就不排记覆盖的活；`:2984` 库里有这把覆盖号又不是过期的就沿用。
- 闭包键只看（种类、指纹）：`scripts/gate/lib_function_units.py:7603`「lines += sorted(f"unit\t{unit.kind}\t{unit.fingerprint}" for unit in closure.order if not unit.data)」，不看行号与名字。
- 排除：`scripts/mutation/lib_mutation_units.py:774`「excluded = () if break_is("judging-files-included") else not_judging_files()」。

推出来的：改 `lib_function_units.py` 里只动行号区间、不动指纹的一处，全部单元指纹不变，于是闭包键、行键、基线键、覆盖号都不变；但覆盖记录里「执行过的单元」变了，基线复用跟着变。这一类改动在判法单元指纹里完全看不见，覆盖号里也没有判法单元指纹。

### 第二条路：真 crates/ 上，改一处切单元的逻辑（量的，副本）

E1：`out/k1-lfu-edit.diff`，`lib_function_units.py` 第 1085 行 Rust 单元的末行 `self.lines[min(end, self.count) - 1]` 后面加 `+ 1`（一处、只动行号）。`k1_real_tree.py` 给真 crates/ 下每个 fn 单元造一条「执行过」的函数（区域起在 `fn` 那个词的行），交给真 `UnitKeys.executed_units` 落单元，原样与 E1 各跑一次（`out/k1-real-tree.txt` 原样）：

```
name=real_tree scripts=copy-lfu0 fn_units=19946 units=38189 placed_elsewhere=0
name=real_tree scripts=copy-lfu1 fn_units=19946 units=38189 placed_elsewhere=878
name=real_tree_fingerprints identical=1
name=real_tree_placement_diff lines=878
```

38 189 个单元的（文件、种类、限定名、指纹）逐字相同（`cmp` 退 0），878 个 fn 单元落进了紧挨在它前面的单元（其中 `crates/singlefs-core/src` 124 个），例如 `crates/singlefs-checker/src/image.rs:22 <trait ImageReader>::device_bytes -> fn:<trait ImageReader>::devices`。两份副本 `crates-mutation-rows.py judging-units .` 都打 `c63379ff285eba00ca251e038844715f9057189fb3fca205c2985172c9d0dccc 465`。

### 第三条路：自造小仓上造一条变异行，跑整道 `run_stage`（量的，副本、假 cargo）

小仓（`k1_timeline.py` 的 `SOURCE`）：`OFFSET` 常量一行、紧接着一行的 `add`、空一行、`helper`；点名测试 `helps` 调 `add` 与 `helper`，假 cargo 的 `// TEST helps NEEDS value + 7` 与 `NEEDS left + right` 两句定它红不红、`RUNS helps,add,helper` 定它执行了哪几个函数。变异行一条：`helper-becomes-eight`（`{ value + 7 }` → `{ value + 8 }`，点名 `helps`）。第一趟 S0 原样；第二趟 S1 把 `add` 的函数体 `{ left + right }` 改成 `{ right + left }`——点名测试在没改坏的源码上就红，正确的判定是「基线不绿」。命令：`bash k1_runs.sh <模型目录> <原样副本> <E1 副本> <工作目录>`（rerun.sh 里那一步）。`out/k1-runs.txt` 原样摘第二趟与覆盖记录（`<W>` 是工作目录）：

```
A.1 name=coverage_record id=b962b2fa0b6f9993 complete=True executed=fn:add,fn:helper,fn:tests::helps,mod_head:tests
A.2 name=run state=S1 break=- exit=1
A.2 name=verdict_line   计数：抓到 0 条、没红 0 条、无效 0 条（共 1 条）
A.2 name=keys judging=c63379ff285eba00 row_key=2866562c9f23fcfe baseline_key=7c09233338be8f3b coverage_id=b962b2fa0b6f9993 test_closure=7a775197c34742c5 mutated=fn:helper
B.1 name=coverage_record id=b962b2fa0b6f9993 complete=True executed=const:OFFSET,fn:helper,fn:tests::helps,mod_head:tests
B.2 name=run state=S1 break=- exit=0
B.2 name=verdict_line   计数：抓到 1 条、没红 0 条、无效 0 条（共 1 条）
B.2 name=keys judging=c63379ff285eba00 row_key=2866562c9f23fcfe baseline_key=7c09233338be8f3b coverage_id=b962b2fa0b6f9993 test_closure=7a775197c34742c5 mutated=fn:helper
C.2 name=run state=S1 break=- exit=0
C.2 name=verdict_line   计数：抓到 1 条、没红 0 条、无效 0 条（共 1 条）
D.2 name=run state=S1 break=judging-files-included exit=1
D.2 name=keys judging=c7bc2303eee3e0dd row_key=f764e8a0501391e3 baseline_key=b4be201abe1260c5 coverage_id=b962b2fa0b6f9993 test_closure=7a775197c34742c5 mutated=fn:helper
```

| 时间线 | 第一趟（S0）用的 | 第二趟（S1）用的 | 开关 | 第二趟结局 | 四把键 |
|---|---|---|---|---|---|
| A | 原样 | 原样 | — | 基线不绿，退 1（对） | 判法 `c63379ff…`、行键 `2866562c…`、基线键 `7c092333…`、覆盖号 `b962b2fa…` |
| B | E1 | E1 | — | 抓到，退 0（假抓到） | 与 A 逐字相同 |
| C | E1 | 原样（「修好」） | — | 抓到，退 0（修好之后仍假抓到） | 与 A 逐字相同 |
| D | E1 | 原样 | `judging-files-included`（两份进指纹，改前） | 基线不绿，退 1（对） | 判法、行键、基线键都变了，覆盖号没变 |

A 与 B 只差 `lib_function_units.py` 那一处，四把键在两趟里逐字相同，覆盖记录的「执行过的单元」从 `fn:add` 变成 `const:OFFSET`，第二趟 `add` 改坏了而 `executed_set_is_current` 照旧成立，基线复用、点名测试照红、记成抓到。C 是「改坏之后又改回来」：改回来不作废，第一趟按 E1 记下的覆盖照旧用。D 用改前的判法（两份进指纹），基线键随换代变了、基线重跑，判对。

### 附：放回指纹只修第一次换代（量的，副本；改法零轮）

`k1_runs_long.sh`：四趟 S0（建）、S1（`add` 改坏）、S2（改回）、S3（再改坏），第一趟 E1、之后原样。`out/k1-runs-long.txt` 原样摘第四趟：

```
E.4 name=run state=S3 break=judging-files-included exit=0
E.4 name=verdict_line   计数：抓到 1 条、没红 0 条、无效 0 条（共 1 条）
F.3 name=coverage_record id=b5d98ea585322e26 complete=True executed=fn:add,fn:helper,fn:tests::helps,mod_head:tests
F.4 name=run state=S3 break=judging-files-included exit=1
G.4 name=run state=S3 break=- exit=0
```

| 时间线 | 判法单元指纹带不带那两份 | 覆盖号带不带判法单元指纹 | S1 | S3 |
|---|---|---|---|---|
| E | 带（改前） | 不带（现文） | 基线不绿（对） | 抓到（假） |
| F | 带 | 带（`out/k1-coverage-id-fix.diff`，只在我的副本上改） | 基线不绿（对） | 基线不绿（对） |
| G | 不带（现文） | 带 | 抓到（假） | 抓到（假） |

E 的病根是覆盖号不带判法单元指纹：E1 时记下的覆盖在 S2 那一趟被判作「执行过的单元都没变」，不重记，S3 又拿它复用基线。F 两样都加，S2 那一趟覆盖号换了、重记出 `fn:add`，S3 判对。G 只加覆盖号一样，判法单元指纹不随 `lib_function_units.py` 变，覆盖号也不变，照样假。

### 四句

1. 分不分辨臂：分辨「现文（排除）」与「改前（进指纹）」两条臂，只在换代那一趟（C 对 D）；同一份坏的 `lib_function_units.py` 一直用着（B）两条臂一起错，那一半不分辨臂，病根是 E1 本身写错，不归键管。另有一半两条臂共有：覆盖号不带判法单元指纹（E 对 F），改前的样子也只修一次。
2. 系统看不看得到：看得到，`lib_function_units.py` 就在判法闭包里（`judging-files-included` 下 496 个单元，K5 现数），是定案按角色把它排掉的；覆盖记录里也存着落进的单元清单。
3. 满足的是哪一句：`.claude/kb/decisions/13-验证路线.md` 第 435 行文件级那一句的「管怎么存、怎么算键，不管判对错」分句——`lib_function_units.py` 经 `executed_units` 管了「覆盖落进哪几个单元」，这在 crates 变异判法里是判对错的一环（正文 K1 格点名的四件里的两件：覆盖落单元、基线复用；结局「抓到 / 基线不绿」跟着翻）。不是「它们的单元不进任何一份判定的键」那一句写错，那一句照做了。
4. 跑前给的改法：第二轮判决没给改法（只挂起）。我量过的三样：进指纹（D、E）在 C 那一格修、在 S3 那一格不修；覆盖号带指纹单独做（G）两格都不修；两样都做（F）两格都修。都只在我的模型上量过、被攻过零轮。

| 改法 | 修 C/D 那一格（换代当趟） | 修 E 第四趟那一格（换代之后再改源码） | 量过 / 推的 |
|---|---|---|---|
| 判法单元指纹放回那两份（改前） | 修 | 不修 | 量过（D、E） |
| 覆盖号带判法单元指纹 | 不修 | 不修 | 量过（G） |
| 两样都做 | 修 | 修 | 量过（F） |
| 只把 `lib_function_units.py` 里 `executed_units` 用到的那几个函数（行号区间那一路）挑出来进判法单元指纹，再加覆盖号带指纹 | 修 | 修 | 推的（没实现：要先证 `RustFile.row` 这类只打提示的函数不在那一路上） |

## K2 入口符号链接旁的 preflight：两种零轮改法各自放过与误拒什么

入口 source 的不止 `preflight.sh`。全是按 `${BASH_SOURCE[0]}` 或 `$0`（链接的路径，`dirname` 取链接所在那棵树）取的（都是量的，`grep -n`）：

| 入口 | 行 | source 的 |
|---|---|---|
| `scripts/experiment-apparatus/replay.sh` | 99 | `$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh` |
| 同上 | 101–102 | `cd "$(dirname "$0")/../.."` 之后 `source scripts/hosts/heavy-threads.sh` |
| 同上 | 1400 | `source scripts/hosts/run-build-directory.sh` |
| `scripts/crash/crash-point-replay.sh` | 271 | `preflight.sh` |
| 同上 | 287–288 | `crash-amplification-lib.sh`、`../hosts/run-build-directory.sh`（按 `${BASH_SOURCE[0]}` 定目录） |
| `scripts/crash/crash-amplification.sh` | 346 | `preflight.sh` |
| 同上 | 355、1790 | `crash-amplification-lib.sh`、`../hosts/run-build-directory.sh` |

`preflight.sh` 自己（第 11 行）又 source `../singlefs-ai-sop/scripts/preflight.sh`；`crash-amplification-lib.sh:119` 再 source `heavy-threads.sh`。所以一份指进本仓的入口符号链接，cd 进链接那棵树之后，`heavy-threads.sh`、`crash-amplification-lib.sh`、`run-build-directory.sh`、`.claude/singlefs-ai-sop/scripts/preflight.sh` 都从那棵树里取——至少 6 份，比 `preflight.sh` 一份多得多。

- **改法二（认符号链接时连旁边 `preflight.sh` 一起核）**：只核了 6 份里的 1 份。攻击者把 `preflight.sh` 留成与本仓一致、改 `heavy-threads.sh` 或 `crash-amplification-lib.sh`（里面照样 source 得到、照样能起 cargo），这条改法放过。要核到够的层级，等价于「命令词所在目录这一级都不经符号链接」——因为 cd 之后所有相对 source 都挂在那棵树上，核某几份永远漏下一份。推翻条件：入口不再用 `dirname "${BASH_SOURCE[0]}"`/`$0` 定根，改从 `git rev-parse` 之类定根，旁边放什么都不被 source。
- **改法一（不认符号链接）**：看它写成哪一种。写成「命令词这一个文件不是符号链接」——目录符号链接（`ln -s 别处/scripts/experiment-apparatus 本仓名`、再起里面的 `replay.sh`）照样放过，因为命令词文件本身不是链接。写成「命令词的真实路径整条不经任何符号链接」（`realpath` 等于字面路径）才堵得住，这一条的误拒只在「本仓本身挂在一条带符号链接的路径下」时出现。本机的事实：`realpath /home/fy5090/code/singlefs` 等于 `/home/fy5090/code/singlefs`（量的，仓根不经符号链接），这台机器上改法一不误拒本仓的轻用法。

判定：两条改法现在的写法都够不着，改法二结构上永远差一层、改法一要写成「整条真实路径」那一读法才成立。都是推的（只读源码、只核了一个 `realpath` 事实，没造放行现场——造现场是第二轮 H2-a 的角度，不重复）。四句：① 分辨「核一份」「核整棵树」「命令词这一文件」「整条真实路径」几读法；② 钩子看得到链接与它旁边的几份库；③ 满足的是 `own_repository_script`（`lib_heavy_tests.py:3375`）的 `realpath(resolved) == realpath(本仓那一份)` 这一句，它只比命令词那一个文件、不管 cd 之后 source 的；④ 第二轮没给跑前条款，这里只判射程。改法：改法一写成「整条真实路径不经符号链接」（与 `--selftest`、`stage_selftest_is_own_copy` 比 sha256 并列的另一条，`lib_heavy_tests.py:3370` 写了「判法与 stage_selftest_is_own_copy 同一种」），只在我的读法上推过、被攻过零轮。

## K3 GPU 导入旧行被拒整条退 1 的语义

commit-stages 第 2b 步第 26 行写明：`IMPORT_GPU_REJECTED` 各行、「导入退 1、别的行已并进来」、「之后本机的 GPU 趟照现判」、再之后「导入之后再在有卡的机器上起一趟 ③（不带 `--shard`，沿用得了的不重跑）」。退 1 的含义（量的，`grep`）：`scripts/mutation/broad-mutation.py:631`「EXIT_INCONSISTENT, … = 1, …」，`:4453`「if conflicts or gpu_plan.rejected or gpu_plan.conflicts:」后 `return EXIT_INCONSISTENT`。照出路第二支「不重导也行，被拒的这几条本机之后补跑 GPU 趟时照现判」做，不会误判成失败——退 1 当的是「有旧行没并，去起 ③」，不是「本趟判错了」。这一半站得住。

两处打中：

1. **出路第一支在持久库上不收敛（推的，读 `plan_gpu_import`）**：`broad-mutation.py:4322`「if row["key"] not in current_keys:」与 `:4324`「elif row["gpu_key"] != pass_key:」对 `gpu-verdicts.tsv` 里**每一行**核键，不看这把键有没有被更新的行顶掉。`gpu-verdicts.tsv` 只追加（文件头、`--rerun` 那一行「追加新行，旧行留着」，量的 `broad-mutation.py:37`），commit-stages 第 2b 步 ③ 指的是固定持久库目录。出路第一支「在拿卡的那台上重跑它那一片、再重导」：重跑只追加这一趟的新行，旧的被拒行还在库里，下一次 `--import-results` 照样核到它、照样退 1。换句话说出路第一支治不了退 1；真能让它退 0 的只有出路第二支（不重导、之后补跑）或让第二台每次导入前的 `gpu-verdicts.tsv` 只有这一趟的行。推翻条件：`plan_gpu_import` 只核每把键最新的一行、或旧行被顶掉后不再核（现在是逐行全核）。
2. **正文 K3 说的「第一轮 G3 判决那一句」指不到（量的，`grep`）**：正文说「CPU 那一侧旧行不退 1 的依据（第一轮 G3 判决那一句）放在 GPU 一侧成不成立」。第一轮判决 `research/prompts/m3-close-gov-b17-r1-main-verification.md` 的 G3 那一格里没有「旧行不退 1」「只多一行」这类句子（`grep -F '多一行'` 在那份判决 0 命中）。那句理由在驱动自证的提示里：`scripts/mutation/broad-mutation.py:6659`「…别的源码的键与本机的不同，并进来只是多一行」。放到 GPU 一侧成不成立：CPU 侧 `import_results` 对 `verdicts.tsv` 的旧行确实不核「键在不在本机此刻清单」，只核「同一把键两边定论不同」才退 1；GPU 侧多加了两把键的核（`:4322`、`:4324`），所以同一张持久库的旧行在 GPU 侧退 1、CPU 侧不退。两侧对「键已不在本机清单的旧行」处置不一样——这是导入语义的取舍，不是 bug 的证据。

判定：退 1 的语义（有旧行没并、去起 ③）对、照 commit-stages 做不会误判成失败；但出路第一支不收敛、正文引的那句出处写错（在驱动自证不在第一轮判决）。都是小的。四句：① 分辨「GPU 侧逐行核两把键、旧行退 1」与「像 CPU 侧那样旧行只多一行」；② 系统看得到旧行的 key 与 gpu_key；③ 满足的是 `:4453` 的 `gpu_plan.rejected` 非空就 `return EXIT_INCONSISTENT`；④ 没有跑前条款。改法（推的，没实现）：出路第一支补一句「重导前把被顶掉的旧行从第二台库里删掉，或导入只核每把键最新的一行」，被攻过零轮。

## K4 修补二批逐项对第二轮判决

快照里的 `governance-fix-2.patch` 反向打得干净（`git apply --reverse --check` 退 0），它改 3 份：`implementation-workflow.md`、`verification.md`、`crates-mutation-rows.py`（`grep -c '^+++'` = 3）。逐处对第二轮判决第三节「修补二批」那一行（量的）：

- `implementation-workflow.md:31`（现文含「解析出的闭包真引到的依赖版本」「算键时有两种排除。」）：补丁把依赖版本那一段加「闭包真引到的」、把「两种排除」后面的「，理由不同、分开记」删掉——判决要的两样都做了；补丁里「模块级语句一起进闭包」的 `-` 与 `+` 行都写「进闭包」（`grep -oE '一起(收进来|进闭包)'` 全是「进闭包」，没有悄悄多改一处）。
- `verification.md:50`（重型那一行）：同两样（依赖版本、删「理由不同、分开记」）。
- `crates-mutation-rows.py`：H3-c 自证那一半——删掉改 `lib_verdict_library.py` 的 `digest_text` 那一处（空测），改成只改 `lib_function_units.py` 的 `RustFile.row`，断言里写明「`lib_verdict_library.py` 对 crates 判法闭包贡献 0，没有函数可改」，与判决 H3-c「写明或换函数」对得上。

题面还把 `heavy-test-guard.sh` 文件头补实验复跑入口、`verification.md:35`（快档那一行的依赖版本）、`lib_mutation_units.py:36` 引文同步算进修补二批。这三处在被判的树里（`judged-sha256.txt` 12 行 OK），但不在 `governance-fix-2.patch` 文件里（`verification.md:35` 的改动 `git diff -U0` 看得到、补丁的 `verification.md` 只有重型那一行那一个 hunk）。这是补丁作部分记录（钩子那一份经 `hook-swap.py`，不进这份补丁），不是缺改——这三处都是题面点名的，不是「改出判决没要的东西」。

打中（小）：③ 补进去的实验复跑入口那一句，三处逐字相同（量的，`grep`）：

- `implementation-workflow.md:49`、`heavy-test-guard.sh:49`、`agent-common.md:58` 都写「`--list-items` 与登记为 light 的单独 `--selftest` 不算」。

「单独」这个限定只挂在 `--selftest` 上，没挂在 `--list-items` 上。而 `classify`（`lib_heavy_tests.py:623`「不读正文的只有解到本仓那一份的」那一段，判法在 `lib_heavy_tests.py:3289` 的 replay-all-stage 分支）只对「摘掉 `--force` 之后恰好一个 `--list-items`」交 None，别的都算重型。我在主工作区现跑 `classify`（只算 python、不起子进程，登记为 light），原样：

```
name=classify args=--list-items kind=None
name=classify args=--list-items E165 kind=replay-all-stage
name=classify args=--force --list-items kind=None
name=classify args=--list-items --list-items kind=replay-all-stage
name=classify args=E165 kind=replay-all-stage
name=classify args=--admission-only kind=replay-all-stage
```

`--list-items E165` 与 `--list-items --list-items` 都判重型（replay-all-stage），规则那一句「`--list-items` …不算」字面比 classify 松——正文 K4 点名的「`--list-items` 后面还带别的词算重型那一例」在规则那一句里没写出来。四句：① 不分辨臂（规则与代码的字面差，不是臂）；② 看得到；③ 满足的是规则那一句缺「单独」限定词；④ 没有跑前条款。改法（推的）：规则写「单独 `--list-items`」或补「后面带别的词算重型」，与 `--selftest` 那一侧一个口径，被攻过零轮。`heavy-test-guard.sh:49` 的「按名字判、不读正文」是这份文件头的既有写法（该段 40–150 行里「不读正文」用了 9 次），own-copy 的例外在 145–149 行补着（`grep` 量的），不是这一批新写破的，不算打中。

## K5 第二轮报出的数逐个现跑

能核到的（主工作区现文、现跑，量的）：

- 961 / 496 / 465：`GATE_MUTATION_BREAK=judging-files-included python3 scripts/mutation/crates-mutation-rows.py judging-units . --units` 首行 `c7bc2303…b47da40 961`，`grep -c 'scripts/gate/lib_function_units.py:'` = 496、`grep -c 'scripts/crash/lib_verdict_library.py:'` = 0；不带开关 `judging-units .` 首行 `c63379ff…c9d0dccc 465`。三个数都对上。
- 1442（`heavy-test-guard.sh --selftest`）：现跑末行「查了 1442 种，其中该拒 274 种」，退 0。
- 801（`lib_heavy_tests.py --selftest`）：现跑「查了 801 种」，退 0。
- 159（`crates-mutation-rows.py --selftest`）：现跑打「crates-mutation-rows.py 自证 159 格都对」，但我 235 秒时限先到、整条退 124（自证真跑 cargo 要更久）——数对上，退码没等到。
- cite-check 的「53 处引文」：`cite-check.py <两份腿报告> --root <倒推树> --background …` 末行「核了 53 处引文、55 处裸行号」，53、55 对上。

核不到的（树变了、现跑与第二轮报的不同）：

- 13/14 倒推：`snapshot-tree.py m3-close-gov-b17-r2` 现跑「清单点名 14 份，倒推出 9 份、对上 9 份、倒推不出 5 份」，退 1——第二轮报 13/14。差的 4 份（`implementation-workflow.md`、`verification.md`、`heavy-test-guard.sh`、`lib_mutation_units.py`、`crates-mutation-rows.py` 里的几份）是修补二批之后主工作区又改过，倒推的基底对不上。这不是第二轮报错，是今天的树已不是第二轮那一刻的树（`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 第 31 行「冻结只在这一轮内有效，出了轮归档」）。
- cite-check「对上 27、没判 81」：现跑「对上 19、没判 89」——倒推树今天少 4 份，引到它们的行从「对上」掉进「没判」。
- 辩方「八行」(`result=identical`)：拷它的模型进副本现跑，第一步就退 1，`rerun_steps` 卡在「`.claude/rules/verification.md` 里找不到『两种排除理由不同』」——那句正是修补二批删掉的（H1 够不着那一处），辩方的 rerun 写死 grep 它，树一改就红。这证明辩方那八行是第二轮那一刻的树上的数，出了轮核不动。
- 攻方「identical=6」：拷模型进副本现跑，230 秒里没跑完（它造三棵仓副本、各起钩子），退 124。核不动（要更长挂钟），不当核过。

cite-check 的倒推树、`snapshot-tree` 与两腿模型复跑的命令、退码都在「要主 agent 跑的命令」一节；主 agent 要核第二轮的 13/14、27/81、八行、identical=6，得把倒推不出的 5 份按第二轮快照补回去再跑（`.claude/rules/three-way-inference.md` 第 125 行那一步）。

## 没打中的形状

- K1 时间线 A/B（同一份坏的 `lib_function_units.py` 两趟都用）两条臂一起错，不分辨臂——那一半的病根是 E1 本身写错、不归键管，不拿它判「排除成不成立」；判「排除成不成立」只看换代那一趟（C 对 D）与换代之后（E 对 F），这两处分辨得出。
- K1 只造了 E1（动行号区间）一种改动翻面；别种改切单元逻辑而连指纹一起变的那一类不翻面（键变、照作废），没单独造。
- K2 没造符号链接旁改 `heavy-threads.sh` 的放行现场（第二轮 H2-a 造过符号链接旁改 `preflight.sh`，不重复）：只读源码确认入口 cd 之后从链接那棵树 source 至少 6 份，推出改法二差一层。
- K3 没跑真两台工作流、没往第二台派活：出路第一支不收敛由 `plan_gpu_import` 逐行核的源码推，库持久由 commit-stages 的固定库目录推。
- K4 没找到补丁 3 份文件里「改出判决没要的东西」；`heavy-test-guard.sh:49` 的「不读正文」是既有写法 + 145–149 行的 own-copy 例外，不算打中。
- K5 的 13/14、27/81、八行、identical=6 核不到（树出了第二轮那一刻），不当第二轮报错，也不当核过。

## 这条腿自己的限度

- 全在本机跑，全是 python 与钩子：假 cargo / 假 llvm / 假内存包装取自 `crates-mutation-rows.py` 自证，不起真 cargo、不编译、不碰真判定库（库在草稿目录）；`judging-units`、`classify`、`snapshot-tree`、`cite-check` 只算 python。
- K2、K3 的结论是推的（只读源码、只核了一个 `realpath` 事实与 `plan_gpu_import` 的逐行核），没造放行现场、没跑真两台。
- K1 的覆盖号带判法单元指纹、两份放回指纹、`executed_units` 那几个函数单独进指纹几条改法只在我的副本上量过（E/F/G）或只推过，被攻过零轮；一览表里标「推的」的没实现、没跑。
- rerun.sh 一趟约 5 分钟（造 4 份脚本副本、跑假 cargo 时间线、走真 crates/），超过子 agent 前台时限，我分步各跑过、没整条跑一遍；主 agent 复跑按「复跑」那条命令整条跑。
- 没判正推、辩方那几格；没替主 agent 采纳；副本上量出的数不算入库装置上的数。

## 要主 agent 跑的命令

判决要核第二轮的数时（K5 核不到的那几个），在第二轮那一刻的树上跑（倒推不出的 5 份按 `research/prompts/m3-close-gov-b17-r2-snapshot` 补回去）：

- `python3 scripts/agents/snapshot-tree.py m3-close-gov-b17-r2 --out <草稿>/r2-tree`（我现跑退 1，9/14）
- `python3 scripts/agents/cite-check.py research/prompts/m3-close-gov-b17-r2-opus-output.md research/prompts/m3-close-gov-b17-r2-sonnet-output.md --root <草稿>/r2-tree --background research/prompts/_m3-close-gov-b17-r2-background.md`（我现跑「核了 53、对上 19、没判 89」）
- 两腿模型复跑：`bash scripts/hosts/run-with-memory-cap.sh 2G bash <副本>/{opus,sonnet}-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-gov-b17-r2-snapshot --work <副本>/{opus,sonnet}-work`（辩方现跑退 1、卡在 grep 已被删的那句；攻方 230 秒没跑完、退 124）
- 本腿模型复跑（整条约 5 分钟，前台跑得完或经 `detached-run.sh`）：`bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-gov-b17-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-gov-b17-r3-snapshot --work <新的草稿目录>`，末行 `name=rerun_summary identical=2 differs=0`。

## 没做什么

- 没跑门禁阶段、没编译、没跑重型；`judging-units`、`classify`、`snapshot-tree`、`cite-check` 与三个 `--selftest` 都登记为 light（`.claude/hooks/selftest-registry.tsv`）、只算 python 或起 git。
- 没读禁读清单里的文件（本轮另一条腿的报告与模型）。
- 草稿清理：交回前 `du -sh` —— 仓副本 `copy-lfu0`/`copy-lfu1`/`copy-lfu0-fix`/`copy-lfu1-fix`（各约 26M）、K1 的 `runs/`、K5 的两腿模型副本与 work、仓副本都删；模型已拷进 `research/prompts/m3-close-gov-b17-r3-opus-model/`，草稿目录交回前删到只剩认领标记。

## cite-check

`python3 scripts/agents/cite-check.py research/prompts/m3-close-gov-b17-r3-opus-output.md` 末行：

```
  ✓ 核了 10 处引文、18 处裸行号，对上 9 处，没判 19 处（其中裸行号没有引文可比 13 处；逐处列在上面）
```

没判的多是代码引文只写了文件名（仓里有同名文件，cite-check 认不出是哪一份）与裸行号没有引文可比；逐处的全路径在正文里写明了（`scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/lib_mutation_units.py`、`scripts/gate/lib_function_units.py`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`scripts/mutation/broad-mutation.py`、`scripts/crash/crash-amplification-lib.sh`、`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`.claude/agent-common.md`），主 agent 判决时逐处现查（`grep -nF`）。
