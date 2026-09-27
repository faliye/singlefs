# defs-gatebatch-m2-r2 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

取辩方报告第 11 行的引用（`54-layer0-replay.sh:118`，原文
`layer0_judge_paths=(.claude/gate.d/stage-inputs.tsv "$(realpath ... layer0_stage_script_path)" "$(realpath ... layer0_admission_module)")`），
在草稿目录 `/tmp/claude-1000/defs-gatebatch-m2-r2-verifier/selftest/54-layer0-replay.sh` 副本上把行号人为改成 119（原文第 119 行是
`scope_reason="$(bash ...)"`，与被核引文不同）。核法：`awk 'NR==119' 54-layer0-replay.sh | grep -qF 'layer0_judge_paths='`。

```
✗（引文与该行号实际内容不符——判别力自证通过：核对方法确实能分辨行号错配）
```

自证通过（正确判 ✗），下面按同一套方法核三条腿。

## 一、输入与快照

- 三条腿报告：`defs-gatebatch-m2-r2-defense-output.md`（66 行）、`defs-gatebatch-m2-r2-opus-output.md`（308 行）、
  `defs-gatebatch-m2-r2-local-attack-output-s1.md`（15 行）、`-s2.md`（15 行），提示 `defs-gatebatch-m2-r2-local-attack.md`（61 行）、
  核对表 `-translation-audit.md`（31 行）、运行记录 `-runlog.md`（11 行）。
- 快照：`research/prompts/defs-gatebatch-m2-r2-snapshot/sha256sums.txt`，11 个文件（`54-layer0-replay.sh`、`admission.py`、
  `stage-inputs.tsv`、`stage-owners.tsv`、`lib_heavy_tests.py`、`heavy-test-guard.sh`、`crash-verifier.md`、
  `implementation-workflow.md`、`main-agent.md`、`gate-triage.md`、`check-segment-registry.py`）。现跑 `sha256sum -c` 11 个全 OK
  （与主 agent 交回时一致），故这 11 个文件的引用直接对主树核，等价于对快照核。
- **快照没覆盖 `crates/`**（这一轮不是给 `crates/` 与 `.claude/kb/` 的快照，只给了这 11 个具体文件）。三条腿都引用了
  `crates/singlefs-harness/` 下的测试文件与 `src/crash.rs`；`git status --short` 现查，此刻仓里有 116 处 `crates/` 下的改动
  （其中 19 处是未跟踪的新文件，含被引的 `second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs` 本身），
  说明有别的会话在并发改 `crates/`。凡是 `crates/` 下的引用，行号对不上主树的，按「分不清：文件可能在腿交回之后被改过」记，
  不记 ✗；这与攻方报告自己写的「floor-raise 那份测试文件的行号在我读的前后挪了 5 行」是同一种现象（我核到的实际是挪了 5 行：
  该文件当年攻方读到 213 行，我现读到 218 行）。

## 二、辩方（`defs-gatebatch-m2-r2-defense-output.md`）核对表

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `54-layer0-replay.sh:118`（Y1，`layer0_judge_paths=(...)`） | ✓ | `awk 'NR==118' .claude/gate.d/54-layer0-replay.sh` |
| `admission.py:168`（第 12 行「现仓与备份…都是」docstring） | **✗ 现仓部分错** | 现仓第 168 行实为 `BUILD_ENVIRONMENT_VARIABLE_FORMS = (...)`，与所引 docstring 无关；docstring 现查在 **admission.py:198**（`grep -n '进指纹的只有键与路径' research/scripts/admission.py` → `198:        """进指纹的只有键与路径…`）。经 `_defs-gatebatch-m2-r2-diff.md` 的 hunk 偏移重算（`@@ -146,6 +169,13 @@` 之后、下一个 hunk 在 `@@ -425,7 +455,9 @@` 之前，这一段旧行到新行恒定 +30），倒推备份（r1 时刻）里这句 docstring 确在旧行 168（198−30=168），故「备份」那半准确，「现仓」那半错配 |
| `admission.py:1265`（`judge_crash_case_log` 定义，「备份同」） | **✗ 现仓错** | 现仓第 1265 行是空行（`attributes_before_function` 函数体内），`judge_crash_case_log` 现查定义在 **admission.py:1495**（`grep -n '^def judge_crash_case_log' research/scripts/admission.py`） |
| `admission.py:1321`（转述 r1 核查员 ✓ 判定，未逐字引文） | ✓（转述准确） | 对照 `defs-gatebatch-m2-r1-verifier-output.md:147,150,151`：分别是 `54-layer0-replay.sh:168 ✓`、`admission.py:1265 ✓`、`admission.py:1321 ✓`，与辩方转述一致 |
| `54-layer0-replay.sh` 头部「两趟 --full 同时跑同一条用例…」（未给行号） | ✓ | `grep -n '两趟 --full 同时跑同一条用例' .claude/gate.d/54-layer0-replay.sh` 命中第 17 行，逐字节相同 |
| `admission.py:1613`（`write_crash_case_marker` 定义） | ✓ | `awk 'NR==1613' research/scripts/admission.py` |
| `54-layer0-replay.sh:187`（`delete_crash_case_marker`） | ✓ | `awk 'NR==187' .claude/gate.d/54-layer0-replay.sh` |
| `admission.py:100`（「减得少的（改了照样让用例重跑）…」） | ✓ | `awk 'NR==100' research/scripts/admission.py`，逐字节相同 |
| `lib_heavy_tests.py:40`（「接受的误拒…」） | ✓ | `awk 'NR==40' .claude/hooks/lib_heavy_tests.py` |
| `heavy-test-guard.sh:83`（同一句） | ✓ | `awk 'NR==83' .claude/hooks/heavy-test-guard.sh` |
| `lib_heavy_tests.py:41-43`（看门狗兜底、「推的，没量」） | ✓ | `sed -n '41,43p' .claude/hooks/lib_heavy_tests.py` |
| `probe_j4_hook.py:63-64`（r1 攻方模型目录，B6/B7 命令原文） | ✓ | `sed -n '63,64p' research/prompts/defs-gatebatch-m2-r1-opus-model/probe_j4_hook.py`，与所引 f-string 逐字节相同 |
| `admission.py:3113/3120/3128/2913/2979`（4 格自证、`FAKE_CARGO_FOR_STAGE`） | ✓（5/5） | 逐行 `awk 'NR==<N>' research/scripts/admission.py` |

计数：核 18 处，✓ 16，✗ 2，分不清 0，核不动 0。

### 重点问题：R1 判决第 16 行 `admission.py:168` 引文错配是否属实

**属实。** 三条独立证据：

1. 全仓搜索 `admission.py:168` 这个字面串，只出现在 `defs-gatebatch-m2-r1-main-verification.md:16`（判决本身），在
   `defs-gatebatch-m2-r1-opus-output.md`、`defs-gatebatch-m2-r1-verifier-output.md`、`_defs-gatebatch-m2-r1-diff.md`
   三份腿产的材料里一次都没出现（`grep -rn "admission.py:168\b" 那三个文件` 无命中）——这个行号不是从任何一条腿的报告里抄来的。
2. 按 `_defs-gatebatch-m2-r2-diff.md` 的 hunk 位移重算（见上表第 2 行），r1 时刻（备份）`admission.py:168` 确实是
   `AdmissionCondition.fingerprint_text` 的文档字符串「进指纹的只有键与路径：准入条件判的是『能不能跑』，不是『算什么』…」——
   这句话讲的是**实验准入条件**（experiment admission）的指纹口径，字面上不是「判日志的代码…它不进每条用例的指纹」
   （崩溃枚举用例的日志判法）这件事。
3. 该句真正的支持性引文在别处且都能现核命中：`judge_crash_case_log` 定义处（现仓 admission.py:1495，r1 时刻按同一偏移换算约
   1265）、`--extra-file` 只把 54 号自己算进指纹处（`54-layer0-replay.sh:168`，r1 与现仓这一行本身没变）。

结论：**这是主 agent 写判决时的一处行内引文错配**（辩方原话），不是攻方或 r1 核查员的错——三方材料里都没有这个具体行号。
但辩方自己复核这一处时，报告里把「现仓」与「备份」的行号混成同一个数（都写 168），这本身在现仓一侧也是错的
（现仓该内容已挪到 198 行），是辩方报告自己的一处引文瑕疵，已计入上表的两条 ✗。

## 三、攻方（Opus，`defs-gatebatch-m2-r2-opus-output.md`）核对表（第一段）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `54-layer0-replay.sh:173`（`--extra-file "<判它的 54 号：…>"`） | ✓ | `sed -n '173p'` |
| `54-layer0-replay.sh:174`（`--extra-file "<判它的准入模块…>"`） | ✓ | `sed -n '174p'` |
| `admission.py:1270`（`found = re.search(...)`） | ✓ | `awk 'NR==1270'` |
| `admission.py:153`（`IGNORE_ATTRIBUTE_FORM = re.compile(...)`） | ✓ | `awk 'NR==153'` |
| `lib_heavy_tests.py:181`（说明句「标了、找不到那个目标…」） | ✓ | `awk 'NR>=178&&NR<=186{print NR": "$0}'` |
| `lib_heavy_tests.py:185`（`return module.test_function_is_marked_ignored(...)`） | ✓ | 同上 |
| `.claude/rules/implementation-workflow.md:89`（「门禁管哪一半」整段） | ✓ | `awk 'NR==89'`，逐字节相同 |
| `admission.py:1476`（`if resumed_slices + freshly_run_slices != slices:`） | ✓ | `awk 'NR==1476'` |
| `admission.py:1911`（`("worker_threads", values["--threads-text"])`） | ✓ | `awk 'NR==1911'` |
| `54-layer0-replay.sh:384`（`threads_text=...`） | ✓ | `awk 'NR==384'` |
| `admission.py:97/98/99`（文件头「认不出的」「按宽处理」两段） | ✓（3/3） | `sed -n '97,99p'` |
| `admission.py:481/501/539`（runner/wrapper 按内容进指纹三处） | ✓（3/3） | 逐行 `awk 'NR==<N>'` |
| `admission.py:158`（`COMPILE_TIME_CONCATENATED_INCLUDE`） | ✓ | `awk 'NR==158'` |
| `admission.py:1381/1386/1404/1405/1415/1379`（排除法六处代码） | ✓（6/6） | 逐行 `awk 'NR==<N>'` |
| `records/2026-09-24-里程碑二收尾调度.md:190`（用户原话「改了只跑改了的部分」） | ✓ | `awk 'NR==190' | grep -o` |

计数：核 24 处，✓ 24，✗ 0。攻方对这 11 个快照内文件与一份记录文件的行内引用，抽样到的全部逐字节命中，**没有找到反例**。

## 四、攻方复跑：Y2–Y7「打中，量过」与 R5/B3 对照

（第二段见下）

**方法**：不在腿的原目录里跑。`rsync -a --exclude target /home/fy5090/code/singlefs/ /tmp/claude-1000/defs-gatebatch-m2-r2-verifier/repocopy/`
（含 `.git`，因为 `admission.py crash-case-manifest` 要调 `git ls-files`），把 `research/prompts/defs-gatebatch-m2-r2-opus-model/` 随副本
一起带过去，在副本里 `cd` 进去跑 `nice -n 19 bash research/scripts/capped.sh 4 bash research/prompts/defs-gatebatch-m2-r2-opus-model/rerun.sh
<草稿输出目录> <草稿 scratch 目录>`（线程上限 4，按派发提示），产物落进草稿目录，不落回仓里的 `outputs/`。跑了两次（第一次误排除
`.git` 导致 `crash-case-manifest` 全部因「不是 git 仓」报错，发现后重来一次带 `.git` 的副本；两次的非 git 依赖部分结果一致，见下）。

### 4.1 各 SUMMARY 行

```
grep -h "^SUMMARY" 复跑产出/*.log | sort > 复跑.txt
grep -h "^SUMMARY" 仓里的 outputs/*.log | sort > 入库.txt
diff 复跑.txt 入库.txt   # exit 0（两次复跑都是 exit 0）
```

6 条 SUMMARY（`k1-hook`、`k1-manifest`、`k3-judge`、`k4-cost`、`fix-k1-hook`、`fix-k1-manifest` 各一条）**两次复跑都与入库版逐字节相同**：
`K1 闸与属性解析 cells=44 attack_cells_broken=18`、`K1 排除规则与 runner 指纹 cells=12 attack_cells_broken=8`、
`K3 两条登记行的判法 cells=12 attack_cells_broken=0`、`K4 窄的判法摘要 cells=8 attack_cells_broken=0`、
改法副本上两条 `attack_cells_broken=2`。

### 4.2 逐格明细（Y2–Y7 覆盖到的格）

`k1-hook.log`、`k3-judge.log`、`k4-cost.log` 三份与入库版 `diff` **逐字节完全相同**（`exit 0`）——Y2（V3–V5、V7）、K3（floor-raise、c561）、
K4（判法闭包 37/365/3194 行、21 个 hunk 全落闭包外）的每一行判定与命令输出都原样复现。

`k1-manifest.log`（Y5/Y6/Y7：M1–M5、G1–G3）与入库版逐字节 `diff` 只有两类差异，都是允许的形态（复跑命令带临时路径、且草稿仓的绝对路径
本身进了指纹的哈希输入）：

- 嵌入临时路径的字段（草稿目录路径 vs 原路径）——按字段比不按整份哈希，一致。
- runner 指向的 `tools/r.sh` 内容里带着这次跑的临时仓绝对路径，导致改前/改后的 sha256 数值本身与入库版不同，但**关系不变**：
  G1、G2、G3 三格都是「改前=改后」（同一份改动前后指纹没变，即打中），C4 对照都是「改前≠改后」——与入库版结论完全一致，只是具体哈希值因路径不同而不同，不算 ✗。

### 4.3 R5（`--config` 别名）与 B3（r1）对照

独立于 `rerun.sh`，直接在本机验证：

```
cargo --version                                             → cargo 1.98.0 (797e8a9bc 2026-08-05)
bash research/scripts/run-with-memory-cap.sh 1G cargo --config 'alias.k1zz="version"' k1zz   （在 /tmp，非仓内跑）
  → error: no such command: `k1zz`
CARGO_ALIAS_K1ZZ=version bash research/scripts/run-with-memory-cap.sh 1G cargo k1zz
  → cargo 1.98.0 (797e8a9bc 2026-08-05)（退 0）
```

与 `rerun.sh` 产出的 `semantics.log:40-43` 逐字节相同（两次复跑都一样）：`--config alias.k1zz="version" k1zz` 报
`error: no such command: 'k1zz'`，`CARGO_ALIAS_K1ZZ=version` 认。**攻方 R5 的核心事实独立复现为真。**

第一轮 B3（`cargo --config 'alias.xt="test"' xt -p singlefs-harness --test c561 -- --ignored`）与 R5 用的是**同一种**
`--config 'alias.<名>="<命令>"'` 语法；r1 攻方报告自己写明 B3「只喂了 JSON」（`defs-gatebatch-m2-r1-opus-output.md:163`
「怎么喂的：… 把 JSON 喂给仓里的 `.claude/hooks/heavy-test-guard.sh`，只看退出码」），从未在真 cargo 上跑过 B3 这条命令。
既然同一语法在本机真 cargo 1.98.0 上报「no such command」，B3 在本机同样走不到——**攻方这一句核实为真**，且它没有改判 r1 的 J4-a
（J4-a 打中的是「闸只喂 JSON 就放行」这件事本身，与命令在真 cargo 上跑不跑得通是两个问题，攻方原文也只说「交辩方核，我不下判」）。

### 4.4 唯一的分歧：`real-repo-manifests.log`（61/77 对 61/82）

两次复跑（相隔约 15 分钟）里，四条真崩溃枚举用例的清单文件数都是 `61`，但「减去数」两次都是 `82`（入库版是 `77`），四条各自的哈希也随之
不同。核对方法：`diff 复跑/real-repo-manifests.log outputs/real-repo-manifests.log`。**记「分不清：文件可能在腿交回之后被改过」，
不记 ✗**——这个量直接依赖 `crates/` 的当前文件集（被排除的文件数），而 `crates/` 不在这一轮给的快照里；现查 `git status --short`
此刻有 116 处 `crates/` 改动（19 处未跟踪新文件），足以解释 77→82 这 5 个文件的差异，量级与攻方自己记录的「floor-raise 测试文件行号
挪了 5 行」一致。


## 五、本地攻方（`-output-s1.md`、`-s2.md`、提示、核对表、运行记录）

### 5.1 提示里 Table 1 逐字抄录

| 引用 | 核的结果 | 命令 |
|---|---|---|
| Table 1 Row A 第三列（`test=...floor-raise... count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED`） | ✓ | `awk -F'\t' 'NR==36' .claude/gate.d/stage-inputs.tsv`，第三列逐字节相同 |
| Table 1 Row B 第三列（`test=...c561... count-line=C561_SIGMA_FULL`） | ✓ | `awk -F'\t' 'NR==37'`，逐字节相同 |

### 5.2 翻译核对表（`-translation-audit.md`）逐行核

| 行 | 原文文件:行 | 核的结果 | 命令 |
|---|---|---|---|
| Row 1 | `stage-inputs.tsv:36` 第四列注释（长句，含「不留进度文件，计数由用例自己断言」） | ✓ | `awk -F'\t' 'NR==36'`，第四列逐字节相同，含被核对表点名要求补的那句 |
| Row 2 | `stage-inputs.tsv:37` 第四列注释 | ✓ | `awk -F'\t' 'NR==37'`，逐字节相同 |
| Row 4 | `.claude/rules/implementation-workflow.md` 第 82–90 行，标题「测试与崩溃检测优先多线程」整节 | **✗ 行区间多算一行** | `wc -l .claude/rules/implementation-workflow.md` → 全文件恰好 89 行，`sed -n '90p'` 空（没有第 90 行）；实际这一节内容是第 82–89 行（标题 82、四条要点 84–87、「门禁管哪一半」段落 89）。核对表写「82 through 90」，多算了一行；引文本身（整段文字）逐字节核对与仓里一致，只是行区间上界错 |
| Row 5 | `implementation-workflow.md:84`（`std::thread::scope`、`std::thread::available_parallelism`、`SharedStream`） | ✓ | `awk 'NR==84'`，三个标识符逐字节都在 |
| Row 5 | `crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs:132`（`&Layer0Resume::NoProgressFile`） | **分不清：文件可能在腿交回之后被改过** | `awk 'NR==132'` 现在是 `&base,`，与所引不符；`grep -n "Layer0Resume::NoProgressFile"` 现查实际在第 140 行；`git status --short` 该文件是未跟踪新文件（`??`），当前总行数 218，核对表写「whole file, 213 lines」——213→218 的 5 行差与攻方腿自己记的「floor-raise 测试文件行号挪了 5 行」量级一致，判定为并发编辑造成的漂移，不判 ✗ |

### 5.3 运行记录（`-runlog.md`）里能核的事实

| 陈述 | 核的结果 | 命令 |
|---|---|---|
| `-output-s1.md` 273 词、`-output-s2.md` 453 词、`-output-void1.md` 478 词 | ✓（3/3） | `wc -w` 三个文件，三个数逐一对上 |
| `oov-check.py` 对 s1、s2 两份都判「生词=0 拼接=0」、退出码 0 | ✓（2/2，现跑复现） | `python3 research/scripts/oov-check.py research/prompts/defs-gatebatch-m2-r2-local-attack-output-s{1,2}.md research/prompts/defs-gatebatch-m2-r2-local-attack.md`，两次都输出「绿 … 生词=0 拼接=0」、`exit 0` |
| `corruption-check.py` 的「粘连」正则是 `(?<![A-Za-z0-9])[:;,]\w`，会把 `::` 的第二个冒号判成粘连 | ✓（机制核实） | `grep -n '\[:;,\]\\\\w' research/scripts/corruption-check.py` → 第 90 行逐字节命中同一条正则；正则语义（否定环视排除字母数字、后跟词字符）确实会命中 `thread::scope` 里第二个冒号后的 `s`。「原提示触发 5 处命中」这一具体计数**核不动**：原始（带 `::`、已作废的）提示版本没有保留副本（`-output-void1.md` 只留了模型答案，不是提示本身），且 `corruption-check.py` 对孤立小样本有语料量门槛（现测「语料太小，判不了」），无法用短样本直接复现计数 |

### 5.4 提示里 Table 2/3 陈述的事实性抽查（对主树核，`crates/` 不在快照射程，仅作旁证）

- Row R1「该测试文件全文零处 `println!`/`eprintln!`」：现查 `grep -c 'println!\|eprintln!' <文件>` = `0`，**实质结论仍成立**（尽管全文行数已从 213 变 218，见 5.2）。
- Row R3 字段列表（`states=`、`slices=`、`worker_threads=`、`configured_worker_threads=`、`worker_threads_source=`、`resumed_slices=`、
  `freshly_run_slices=`、`progress_file_after_completion=`、`elapsed_seconds=`，不含 `exhaustive`）：`grep -n 'LAYER0_PARALLEL_FINISHED states='
  crates/singlefs-harness/src/crash.rs` 命中两处（2714、2833 行，NoProgressFile 与 KeepProgressFile 两条路径），字段集合与核对表所写一致 ✓。
- Row R4 字段列表（`states=`、`record_claimed_state_missing_unit=`，不含 `exhaustive`/`worker_threads`/`threads`）：
  `grep -n 'C561_SIGMA_FULL states=' crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs`
  命中第 586 行，字段与核对表所写一致 ✓。


## 六、计数汇总

| 腿 | 核了几处 | ✓ | ✗ | 分不清 / 核不动 |
|---|---|---|---|---|
| 判别力自证 | 1 | — | 1（预期，证明方法有效） | 0 |
| 辩方 | 18 | 16 | 2 | 0 |
| 攻方（Opus）行内引文 | 24 | 24 | 0 | 0 |
| 攻方（Opus）复跑（SUMMARY 6 条 ×2 次、semantics 关键行 ×2 次、real-repo-manifests 1 项、B3/R5 对照 1 项） | 10 | 9 | 0 | 1（`real-repo-manifests.log` 61/77 对 61/82） |
| 本地攻方（提示 2 处 + 核对表 5 行 + 运行记录 3 项 + 事实性抽查 3 处） | 13 | 10 | 1 | 2（`::rs:132` 行号漂移、原提示「5 处粘连」计数核不动） |
| **合计** | **66** | **59** | **4** | **3** |

单独结论（不计入上表某一格，是这一轮点名要核的问题本身）：**辩方指出的「R1 判决第 16 行 `admission.py:168` 引文错配」属实**
（见第二节「重点问题」，三条独立证据：字面串全仓只见于判决本身、diff 位移重算证明该行 r1 时刻确是另一件事的文档字符串、
真正的支持性引文在别处且可核）。

## 七、没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（例如 J1-a/J1-b/J2/J3/J4 各条打中的判断是否正确、K4 三种更窄办法哪个该选），
  只核引用、产物与复跑。
- 没有逐条核完 opus 报告全部约 90 处行内引文与「四句」小节里的每一句判断，只抽样到 24 处文件行引用 + 全部 6 条 SUMMARY 聚合数 +
  R5/B3 关键语义点；抽样未发现反例。
- 没有重新核 K4 的「2.3–2.8 天」这类推的代价数字（辩方、攻方自己都标「推的」，判决与 K4 腿的估算不重复核）。
- 没有独立复现 `-output-void1.md` 那一次「5 处粘连命中被 `ask-local.sh` 判红、退出码 5」的具体计数：原始（带 `::`）提示版本
  未保留副本，`corruption-check.py` 对孤立短样本有语料量门槛，无法用改造过的小样本精确复现命中数；只核实了正则本身与所述机制一致
  （见 5.3）。
- 没有核本地攻方 s1/s2 两份答案里对 8 个问题的**推理**是否正确（例如「floor-raise 判绿是否真的不要求覆盖全状态空间」这类结论），
  按核查员职责只核引文、产物与复跑，不核结论。
- 没有编译 Rust、没有跑门禁阶段整轮、没有跑任何重型测试（54/55/57/59/87 号、`gate.sh`、`cargo test --workspace` 等）。
- 复跑用的仓副本 `/tmp/claude-1000/defs-gatebatch-m2-r2-verifier/repocopy/`（含 `.git`，359M）、草稿产出目录
  `rerun-out/`、`rerun-scratch/`（含 opus 探针自己建的 `fixcopy/`、小 crate 等）已在交回前删除（见下）。

## 八、产出

- 报告：`research/prompts/defs-gatebatch-m2-r2-verifier-output.md`（本文件）。
- 草稿目录：`/tmp/claude-1000/defs-gatebatch-m2-r2-verifier/`（含 `selftest/`、`rerun-out/`、`rerun-scratch/`、`repocopy/`、
  两份 summary 排序文件）。
