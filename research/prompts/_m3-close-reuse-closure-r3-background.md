# m3-close-reuse-closure-r3：判法单元闭包不走内存包装查机器那条提示路径，第二轮修补之后的形态站不站得住（推论轮第三轮，最终轮）正文（2026-10-22）

## 一、题面

前两轮判决：`research/prompts/m3-close-reuse-closure-r1-main-verification.md`、`research/prompts/m3-close-reuse-closure-r2-main-verification.md`。被判对象是用户 2026-10-22 定的「丙加甲」（原话在调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`）：形式化门禁三个判法入口与广谱变异 `Runner.escalate_step` 的判法单元闭包，今天经内存包装 `scripts/hosts/run-with-memory-cap.sh` 的 `HostLookup` → `scripts/hosts/host-resources.py` → `scripts/hosts/peer-host-lib.sh` 的 `peer_host_run` 拒算。

第二轮挂起：try 只包三处不够（`run` 读完输出后格式化抛 TypeError 仍让包装 252 变 251）；按 `run-with-memory-cap:` 前缀去行，去多（升档证据两句）也去少（不带前缀的提示行与 traceback）；闭包不收模块级 import 与赋值（今天就有的盲区，`lib_long_run.py`、`memory_peaks_lib.py` 改坏而键不变）；甲靠同步那一写法打穿（`source-stamp.py sync` 是本机到本机、只拷 `scripts/hosts/` 起不来）。

这一轮攻第二轮判决第三节「第三轮攻的形态」（下面第三节照抄）。这是最后一轮：分工照 `.claude/rules/three-way-inference.md` 第三轮——「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。这一轮新冒出来的改法算零轮，写进交用户表，不开第四轮。

## 二、实现今天的样子（主 agent 的观测，2026-10-22，行号按主工作区）

- `scripts/hosts/run-with-memory-cap.sh`：内嵌 python 的 `class HostLookup`（第 920 行附近），`begin` 第 939–940 行起线程、`finish` 与 `query` 第 1007 行前台建临时文件，都不在 try 里；排队锁与 join 没有时限。
- `scripts/hosts/host-resources.py`：`remote_probe_program`（第 227 行附近）、`run_on_machine`（第 312–329 行附近，program 经 base64 送 `peer_host_run`）；它在仓内 import 的文件 10 份、7 份不在 `scripts/hosts/`（第二轮攻方量的）。
- `scripts/hosts/host-dispatch.py` 第 315–316 行同样经 `peer_host_run` 送 base64 程序；模块体 import `lib_long_run`。
- `scripts/gate/lib_function_units.py`：脚本一侧的判法单元算法，文件头写「import 时的副作用照算」，第二轮攻方量出 check_proof 缺 164 个、escalate_step 缺 290 个模块级单元。
- `scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES`：文件级排除，两份经 import 与 load 进 check_proof 闭包（776 个单元）。
- 广谱变异 `scripts/mutation/broad-mutation.py`：升档证据读包装的「撞了内存上限」「超过限时」两句；`import_results` 只核键的形状。
- 前两轮的模型可以拿来改臂再量：`research/prompts/m3-close-reuse-closure-r1-*-model/`、`research/prompts/m3-close-reuse-closure-r2-*-model/`（只读，要改拷进自己这一轮的模型目录）。

## 三、被判的形态（照抄第二轮判决第三节）

丙：

1. **HostLookup 整体兜底**：`begin`、`finish`、`run`、`query`、`cancel` 的函数体整段包在 try 里，任何异常只记一行带标记的告警、不改包装退出码；排队锁取 `LOCK_NB`、等线程的 join 加上限。自证：六个注入点（起线程、建临时文件、读输出、格式化、取锁、join）各注入一次，包装退出码与关掉 HostLookup 的那一臂逐个相同；弄坏开关拿掉 try，红在点名那一格。
2. **源头打标记，判法只去标记行**：HostLookup 打的每一行（提示、告警、它线程里的 traceback）以同一个固定标记起头；判法去掉标记行再读，包装别的行（升档证据要读的「撞了内存上限」「超过限时」）照读。自证：同一份 Verus 输出、标记行内容不同，存进库的判定逐字相同；升档证据两句照旧判得出；弄坏开关不去标记行，红。
3. **闭包收模块级语句，再只排一条边**：判法单元闭包把被载入、被 import 的文件的模块级 import 与赋值收进来（不收 class 头）；然后不走 `HostLookup.query` 起 `host-resources.py` 那一条子进程边。自证：改 `lib_long_run.py` 一个模块级 import，形式化入口的键变；改 `memory_peaks_lib.py` 的 `import subprocess`，包装入口的键变；切边前后少掉的单元只经那条边可达。

甲：第二台跑的是本机同一份 `host-resources.py`（连同它在仓内 import 的文件，按值打包送过去），由参数选机器；不依赖第二台有仓、也不依赖同步，送过去的就是本机那几份文件的原样。键里不算第二台（送过去的文件已在本机闭包里）。

条文：D13（验证路线） 已定项 17 与 `.claude/rules/implementation-workflow.md` 写两种排除并列——文件级（`NOT_JUDGING_FILES`，算键的库自己）与边级（只打提示、不改结局的一条子进程边，配上面三件自证）；自证另加一格「键不变而退出码变」必红。

## 四、格

| 格 | 问什么 |
|---|---|
| X1 | 「已定」项撑不撑得住：被判形态与 D13 已定项 17（逐句读原文，尤其「键过窄」「不静默少收」「拒算的照跑、不复用」）、`.claude/rules/implementation-workflow.md`「复用重型门禁的判定」一节、`.claude/rules/verification.md` 判定库那几句是否字面或语义打架；「收模块级 import 与赋值」之后，已登记的 35 个入口里哪些键变、会不会冒出新的拒算 |
| X2 | 未定项清单的完整性：被判形态与第二轮「另登欠账」漏了哪些要随这一改一起动的地方（判法读包装输出的全部调用点、按值打包要带的文件、`host-dispatch.py` 那一路、`selftest-registry.tsv` 与门禁 code-tooling 的 reuse-selftests 格、文件头说明）；甲「按值打包送同一份文件」在第二台缺 python 包或版本不同时会怎样 |
| X3 | 前两轮报出的数能不能核：第一轮「单元 1579 → 1232」「792 → 436」「35 个入口只 4 个变」；第二轮「check_proof 缺 164、escalate_step 缺 290 个模块级单元」「1232 → 1655」「776 个单元」「25 个入口的键变、多 18 到 45 个单元」「仓内 10 份、7 份在外」。逐个用独立于原腿的办法复核 |

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | X1–X3 | 复核两份判决：替被判出局或改掉的写法辩护（按前缀去行、「认 `exec_module` 字面路径」、甲靠同步），判打中够不够得着、是不是同样打中所有替代写法；X3 的数逐个复核 |
| 云端攻方（Opus） | X1–X3 | 找反例：X1 被判形态与已定条款打架、收模块级语句之后冒出的新拒算或错键；X2 漏掉的落点、按值打包在第二台环境不同时的失败；X3 前两轮数字核不上的。不重复前两轮攻过的角度（TMPDIR 被删退 251、stderr 末行进句子、`lib_long_run` 经那条边进闭包、第二台没有常驻仓、ssh 串不连边、try 只包三处、按前缀去行、闭包不收模块级语句这件事本身、同步写法） |

方案按 `crates/` 与 `scripts/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。

### 小节清单：`.claude/kb/decisions/13-verification-route.en.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D13 Verification Route —— Settled | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D13 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### Settled Items | 抄 | 默认全抄（脚本生成） |
| #### Settled Item 1: Crash Refinement First Verifies "One Publish + One Crash + One Recovery" | 抄 | 随父节带到 |
| #### Settled Item 2: Serial Commit First | 抄 | 随父节带到 |
| #### Settled Item 3: RefFS Taken to the Extreme, Handling Persistence and Crash Semantics Itself | 抄 | 随父节带到 |
| #### Settled Item 4: Crash-Point Replay Enumerates Full Write Subsets Only | 抄 | 随父节带到 |
| #### Settled Item 5: Sharing Only a Single Constants Module Generated from kb | 抄 | 随父节带到 |
| #### Settled Item 6: Three Mutually Independent Oracles | 抄 | 随父节带到 |
| #### Settled Item 7: O2 (Independent Parser + Checker) Judges a Single Image | 抄 | 随父节带到 |
| #### Settled Item 8: Two Custom-Developed Mechanisms: Merkle Root Chain Diff and Generational Incremental Scrub | 抄 | 随父节带到 |
| #### Settled Item 9: Crash-Point Replay Must Be Tiered to Avoid Wall-Clock Explosion | 抄 | 随父节带到 |
| #### Settled Item 10: Truly Required Exact Coverage N Is "Number of Commit Protocol Equivalence Classes", Not "Layout Schemes" | 抄 | 随父节带到 |
| #### Settled Item 11: When Ignored Nodes Are Enabled: Rejecting Ledger Rotating by Owed Checks, Retaining Three Rules | 抄 | 随父节带到 |
| #### Settled Item 12: Conflict 1: `Checker as Specification` vs `Verification Paths Sharing No Tools` | 抄 | 随父节带到 |
| #### Settled Item 13: Conflict 2: Refinement Spec vs Differential Model | 抄 | 随父节带到 |
| #### Settled Item 14: Interlocks with Other Decisions | 抄 | 随父节带到 |
| #### Settled Item 15: Verification Code in Two Tiers —— Harness Tier Runs Anytime, Checker Tier Defaults to Commit | 抄 | 随父节带到 |
| #### Settled Item 16: Segments Exceeding u64 Judged in Slices, State Count Uses 1280-Bit Fixed-Width Integer | 抄 | 随父节带到 |
| #### Settled Item 17: Unit of Verdict Reuse Is Function, Key Composed Solely of Function Units, Toolchain, Dependency Versions | 抄 | 随父节带到 |
| ## Historical Versions | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/13-検証ルート.ja.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D13 検証ルート —— 確定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D13 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 確定項目 | 抄 | 默认全抄（脚本生成） |
| #### 確定項目 1：crash refinement はまず「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」を検証する | 抄 | 随父节带到 |
| #### 確定項目 2：まず直列コミットとする | 抄 | 随父节带到 |
| #### 確定項目 3：RefFS を極限まで突き詰め、自前で永続化とクラッシュセマンティクスを処理する | 抄 | 随父节带到 |
| #### 確定項目 4：クラッシュポイント再生は完全書き込みの部分集合のみを列挙する | 抄 | 随父节带到 |
| #### 確定項目 5：kb から生成された定数モジュール1つのみを共有する | 抄 | 随父节带到 |
| #### 確定項目 6：互いに独立な3つの oracle | 抄 | 随父节带到 |
| #### 確定項目 7：O2 は何を判定するか：単一イメージとレコード照合器 | 抄 | 随父节带到 |
| #### 確定項目 8：独自開発の2点：Merkle ルートチェーン差分と世代間増分 scrub | 抄 | 随父节带到 |
| #### 確定項目 9：クラッシュポイント再生は3層に分割する。さもなければ実時間が爆発する | 抄 | 随父节带到 |
| #### 確定項目 10：真に正確に網羅すべき N は「レイアウトのセット数」ではなく「コミットプロトコル構造の同値類数」である | 抄 | 随父节带到 |
| #### 確定項目 11：ignore ノードをいつ開放するか：未済の検査によるローテーション台帳を否決し、3条を残す | 抄 | 随父节带到 |
| #### 確定項目 12：衝突 1：checker 即仕様 と 検証の2経路間でツール共有を禁止する の衝突 | 抄 | 随父节带到 |
| #### 確定項目 13：衝突 2：crash refinement の spec とモデル突き合わせの model | 抄 | 随父节带到 |
| #### 確定項目 14：他の決定との連鎖 | 抄 | 随父节带到 |
| #### 確定項目 15：検証コードを2層に分割 —— harness 層は常時実行、checker 層は既定でコミット時のみ実行 | 抄 | 随父节带到 |
| #### 確定項目 16：セグメント単位で登録されたセグメントの全領域が u64 に収まらなければスライス判定し、状態数には 1280 ビット固定幅整数を用いる | 抄 | 随父节带到 |
| #### 確定項目 17：判定再利用のユニットは関数であり、キーは関数ユニット、ツールチェーン、依存バージョンのみで構成する | 抄 | 随父节带到 |
| ## 履歴バージョン | 抄 | 顶上标题，与开篇正文合成行区间 |

### 小节清单：`.claude/kb/decisions/13-验证路线.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## D13 验证路线 —— 已定 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （D13 标题之下、第一个下级标题之前的正文：第 2-6 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 已定项 | 不抄 | 分项索引表，用 --extra 按行区间取 |
| #### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 2：先串行提交 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 4：崩溃点重放只枚举整写子集 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 5：只共享一份从 kb 生成的常量 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 6：三个互相独立的 oracle | 抄 | 默认全抄（脚本生成） |
| #### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 8：自研的两样：Merkle 根链差分与代际增量 scrub | 抄 | 默认全抄（脚本生成） |
| #### 已定项 9：崩溃点重放要分三层，否则挂钟必炸 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` | 抄 | 默认全抄（脚本生成） |
| #### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model | 抄 | 默认全抄（脚本生成） |
| #### 已定项 14：与其他决策的连锁 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数 | 抄 | 默认全抄（脚本生成） |
| #### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |

**出处 `.claude/kb/decisions/13-verification-route.en.md:1-6`（整段抄，未转述）**

```markdown
## D13 Verification Route —— Settled

[中文](13-验证路线.md) · **English** · [日本語](13-検証ルート.ja.md)

D13 (Verification Route) governs how this project verifies correctness: employing all three off-the-shelf approaches—writing a custom reference implementation for differential testing, bounded exhaustive crash testing, and crash refinement formal verification (survey in [prior-art.md](../prior-art.md) Section 7.4)—without slavishly copying them, designing bespoke verification methods tailored to four characteristics unique to this project (COW issuing a new root per transaction, Merkle self-verification, nonce uniqueness for full-volume AEAD, and lack of reference implementation). It settles how oracles are divided, which two items are custom-developed, how crash-point replay is tiered and sampled, how the crash state set is defined, and what is shared between checker and implementation. Out of scope: what each invariant specifies in [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md); how the three verification approaches land and what blocks what in [verification-build.md](../verification-build.md); where commit steps are registered, how structural equivalence classes are partitioned, and how many exist belong to D17 (Implementation Layering and Third-Party Pipelines) settled item 2.

```

**出处 `.claude/kb/decisions/13-verification-route.en.md:7-446`（整段抄，未转述）**

```markdown
### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Which part crash refinement verifies first** | Verifies the closed unit of "one publish + one crash + one recovery" first; first milestone is that any `jsn` continuous prefix does not split a publish; mount transparency is an independent fourth property; former candidate I-6.5 (Monotonic Nonce Watermark) withdrawn **Status: Settled.** **Progress: Unimplemented.** |
| 2 | **Whether transaction layer commits to serial commit** | Serial first, opening concurrency only past the intersection; future concurrent commits append records in transaction number order **Status: Settled.** **Progress: Implemented.** |
| 3 | **To what extent custom reference implementation (RefFS form) is pursued** | Taken to the extreme at the semantic level: operates in memory strictly via POSIX operations, defining persisted state and permissible post-crash states without on-disk format; written from scratch per kb, isolated crate, accepts whitelisted inputs only **Status: Settled.** **Progress: Implemented.** |
| 4 | **How crash state set of crash-point replay is defined** | Write stream segmented by device; segment closes only when every disk with writes in segment is released by its own barrier or FUA; each full write in segment takes states combined arbitrarily; in-place overwrites take third state "neither new nor old readable"; non-persisted takes old bytes generating real images; mirrored writes counted per device **Status: Settled.** **Progress: Implemented.** |
| 5 | **What is permitted to be shared between checker and tested implementation** | Shares only a constants module generated from kb, all else implemented separately; decision width probed by checker itself then compared against system configuration, reporting "declared value, unprobed" if unprobed **Status: Settled.** **Progress: Implemented.** |
| 6 | **Three mutually independent oracles** | In-memory model differential testing, independent parser + checker, and independent spec executor each verify distinct aspects; independence verified by "three-oracle collusion test", never self-proclaimed **Status: Settled.** **Progress: Unimplemented.** |
| 7 | **What O2 (Independent Parser + Checker) verifies** | Domain of O2 (Independent Parser + Checker) is a single image, judging set membership only; checks requiring a second input belong to record checker (inputs include persisted set from enumerator), unnumbered by design **Status: Settled.** **Progress: Implemented.** |
| 8 | **Two custom-developed mechanisms** | Merkle root chain diff (recursive comparison of adjacent generations of roots, skipping unchanged subtree pointers) and generational incremental scrub (traversing only blocks with birth generation above watermark) **Status: Settled.** **Progress: Unimplemented.** |
| 9 | **How crash-point replay is tiered** | Three tiers all exact full run; incomplete enumeration handled by exact pruning, temporarily unrunnable nodes marked ignore and opened gradually; milestone closeout and judge unit changes require main agent popup asking user to run full run, commits allowed on failure or skip with owed check; failure reproducibility prioritized **Status: Settled.** **Progress: Implemented.** |
| 10 | **What N truly must be covered exactly** | Number of equivalence classes of commit protocol structure, not number of layout schemes; implemented via closed enum plus exhaustive match **Status: Settled.** **Progress: Implemented.** |
| 11 | **When ignored nodes are enabled** | Ledger rotating by owed checks rejected; three rules retained for enabling: deterministic coverage records, `T1` (Regular Tier Wall-Clock Budget) as input, and separating discriminative power from coverage checks **Status: Settled.** **Progress: No code object.** |
| 12 | **Conflict 1: Checker-as-specification and verification paths sharing no tools** | Checker implements two directions: traversal and scan, independently judging the same invariant, sharing only format parsing **Status: Settled.** **Progress: Implemented.** |
| 13 | **Conflict 2: Refinement spec and differential model** | Spec and model written differently at representation level; common-origin observed: ≥ 5 inconsistencies with 0 spec errors triggers merge and budget transfer to exhaustive testing **Status: Settled.** **Progress: No code object.** |
| 14 | **Interlocks with other decisions** | Hash and AEAD treated as uninterpreted functions in formal verification; log structure three tiers burden symbolic execution; checker does not parse message semantics **Status: Settled.** **Progress: No code object.** |
| 15 | **Two tiers of verification code** | Harness tier (`singlefs-harness`) for unit and integration tests run anytime; checker tier (`singlefs-checker-tier`) runs quick tier by default at commit, full run upon user request **Status: Settled.** **Progress: Implemented.** |
| 16 | **Slice sizing and state count width for registered segments** | Segments whose domain exceeds u64 judged in slices, defaulting to 2^32 states per slice starting from library resume point; state count uniformly uses 1280-bit fixed-width integer with checked arithmetic; registry read only via environment variable **Status: Settled.** **Progress: Unimplemented.** |
| 17 | **Unit of verdict reuse is function** | Pruning unit is function: verdict reuse depends solely on de-commented fingerprints of dependent function units, toolchain, and resolved dependency versions; commits, tree diffs, full file text, file/crate fingerprints, manual versions, paths, and whole-stage marks forbidden **Status: Settled.** **Progress: Unimplemented.** |

#### Settled Item 1: Crash Refinement First Verifies "One Publish + One Crash + One Recovery"

**Settlement**: The first verification target of crash refinement is the relational property of recovery function `Recover`, whose minimal closed unit is "one publish + one crash + one recovery". The subject `Recover` across the four milestones refers to three call sites invoking `replay_journal` with `verify_named_units = true` in `crates/singlefs-core` (`Consult` strategy of `recover`, writable mount read phase, read-only mount), all three taking `records` from the output of `scan_journal`; `Ignore` and `ConsultWithoutNamedVerification` are arms for testing and control, placed in trusted computing base; `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` is not a production caller. The four milestones in sequence are:

| Milestone | Obligation | Placement |
|---|---|---|
| **M1 Totality** | `Recover` is defined, terminates, and outputs an image satisfying I-7.1 (Non-Empty Valid Root Slot Set) and I-8.3 (Strictly Continuous Replay Prefix) on **every** crash state enumerated in D13 (Verification Route) settled item 4 | Scaffold. **Completion criterion is not "proven", but "M2 can be stated on top of it"**—crash-point replay already enumerates full run ([verification-build.md](../verification-build.md) "crash-point replay does not sample, full run enumeration"), recovery hangs are immediately exposed, proving totality alone yields no marginal value. Prefix extraction, root selection, and ring scan are in Verus in `crates/singlefs-replay-prefix/src/lib.rs`: prefix core `replay_core` (termination and non-overflow proven inside core, loops have `decreases`; two `requires`—record table sorted ascending by (instance, counter) and counter has successor—guarded by assertions in glue `replay_journal`), root selection core `choose_root_slot` (takes max (`checkpoint_txg`, instance) among self-verified slots, tie-broken by earliest slot order), ring scan core `first_copy_of_each_record_key` (first come first served for same (instance, counter)), joint verdict written as corollary `corollary_recovery_replays_from_the_newest_self_verified_root`; gate checker-tier-formal-verification checks proof, pinned at 56 verified (`.claude/gate.d/verus-pins.tsv`). E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) serves as lightweight tool, its rerun 3 part 2 root-selection joint-verdict family calling real `choose_root`, `scan_journal`, `replay_journal` on ramdisk. Four buggy root-selection variants on legal history K family show only "take first valid" red, other three (ignoring instance, taking lower txg on tie, instance-primary) covered by Verus must-red worlds (`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` root selection section) and adversarial root ring KA (rerun 3 part 3). Outside core: system configuration selection (`choose_system_configuration`) and two glue sections (root selection slot reading, self-verification checksum, fsid, region-to-disk mapping, slot traversal order; ring scan offset enumeration, hint, parsing and CRC, disk traversal order) are outside Verus, covered by crash-point replay and crash amplification CPU judges invoking `recover` on every crash state (GPU judge rewrites `replay_journal` in WGSL, cannot verify core) |
| **M2 Prefix Does Not Split Publish** | **For any `jsn` continuous prefix, its replay result equals the result of an integer number of published checkpoints—no publish is split by prefix; when prefix stops mid-publish, that publish is wholly unapplied** (D16 (Publish Semantics) settled item 4 "a publish is wholly applied or wholly unapplied"). Two layers. **Syntactic layer** (for arbitrary self-verified record table, unconditional): applied records are integral publishes, in-flight bound and commit markers verified entry by entry, applied counters strictly consecutive from chain head (starting from anchor + 1 if readable; if unreadable, first entry `checkpoint_txg` = chosen root + 1, seq 1); back-pointer chain and named units are results of disk-reading callbacks, phrased as "for every applied record, callback returned true". **Semantic layer** (premise: every on-disk record originates from legal write side of this version, i.e. legal history family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound); same (instance, counter) corresponds to unique content across all timelines): "result" is projection of root plus disk, not whole root—returned root's (instance, `checkpoint_txg`) and four new-root fields equal version j on chosen root's timeline (j not earlier than chosen root), remaining fields equal chosen root except unmount marker (reconstructed root marked "not written by unmount sequence"); on disk every version in (chosen root, j] has readable records, named units verified on both copies, sum of records not exceeding in-flight bound | Placement is I-8.3 (Strictly Continuous Replay Prefix)—its supplementary verdict clause is verbatim this rule ("prefix must not stop mid-publish—a publish is wholly applied or wholly unapplied"), do not create duplicate in [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md). Syntactic layer in Verus core `replay_core` `ensures` (`replay_follows_the_prefix_rules`), must-red worlds in `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` where "modify verdict" lines alter executable verdict only without touching spec; semantic layer judged within bound by legal history family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound), Verus does not prove. Outside core: glue (flattening tables, wrapping callbacks, assembling reports and new root) covered by E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) calling real `replay_journal`, harness tests, and mutation lines in `crates/mutations.tsv`; non-projected fields in root assembly (unmount marker, instance table pointer, allocation record tree root) lack mutation lines, generator of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) writing fixed values; two disk-reading callbacks (back-pointer chain, named unit closure) covered by real callbacks in E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound), ramdisk read counts, mutation lines in `crates/mutations.tsv`, and read-only mount read-count cases |
| **M3 Idempotence and Monotonicity** | **Idempotence**: `Recover(Recover(c)) = Recover(c)`, proven in two readings—record-level (applying same record twice vs once yields identical root, verbatim I-8.4 (Replay Idempotence)) and fixed-point (replaying with reconstructed root as chosen root, same record table, same reader, same switches returns identical root; premise that records above chosen root do not exceed in-flight bound, and record table satisfies I-8.12 (Non-Decreasing Journal Record TXG for Same Instance): journal records for same instance have non-decreasing txg with counter); second run returning `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` with 0 records applied when chosen root `checkpoint_txg` saturates counts as equal. "Is just I-8.4 (Replay Idempotence)" does not hold: I-8.4 (Replay Idempotence) is record-level, fixed-point is distinct property. **Monotonicity** (corollary of M2, not proven separately): under serial commit (settled item 2), persistence order of D16 (Publish Semantics) settled item 7, and chosen root being newest valid root in root ring, for two fault-free crash states `c ⊆ c'` in same run, `Recover(c)` lands on version not later than `Recover(c')`, compared by (`checkpoint_txg`, instance), identical order to D22 (How Unit Atomicity Is Synthesized) settled item 7; reopened if concurrent commit enabled or domain widened to include faults. "Applied transaction set is subset of" reading withdrawn: false even for ideal recovery | Verus core `replay_core` `ensures` proves both readings: record-level and fixed-point (fixed-point premise includes I-8.12 (Non-Decreasing Journal Record TXG for Same Instance)); monotonicity written as corollary `corollary_the_landed_version_does_not_go_back`, root selection and write-side premises in `requires`. Adversarial family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) judges fixed-point within bound, violations strictly on tables violating I-8.12 (Non-Decreasing Journal Record TXG for Same Instance) (rerun 3 part 1). Checker side (I-8.4 (Replay Idempotence)) implemented, judged on secondary crash in crash injection outside pool-level checker list: `REPLAY_IDEMPOTENCE_INVARIANT` and `judge_the_remount_after_a_second_crash` in `crates/singlefs-checker-tier/src/crash_injection.rs`. O2 (Independent Parser + Checker) cannot judge idempotence: I-4.5 (Deprecated: Requires Two Images) retirement reason verbatim "idempotence requires comparing two images, cannot be judged on single image" |
| **M4 Not Under-Applying** | Premise and target identical to M2 semantic layer. Root returned by `Recover` is precisely j\*: farthest version from chosen root where every version is fully on disk (record has at least one self-verified copy, named units verified on both copies), and sum of records does not exceed in-flight bound. Without this, proving M1–M3 cannot exclude "confirmed complete publish discarded by recovery": a recovery applying nothing passes all literal clauses of M1–M3. "Not under-applying" read as "spurious violations": variant must yield spurious violation to be blocked | Farthest-version criterion of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) judged within bound; Verus must-red world line "stopping at first record above watermark and applying zero" goes red; M4 not proven separately |

**Mount Transparency** (independent fourth property, not third reading of M3): recovering read-only once on crash image yields R1; executing an action on same image—mount only (acquire number, write row, warm-up, mount continuation, and mount-initiated floor raising), or mount plus one publish—then recovering read-only again yields R2, comparing R1 and R2 field by field. Version compared: mount only compares to version recovered to, mount plus one publish compares to version immediately before post-mount publish. Also judged on secondary crash images img2 across three phases during mount: C(recover(img2)) = C(v1) (v1 is version landed by R1, C takes POSIX content and existence), judging both content and fields; in field half, falling back to v1 has fields identical to R1, landing on a version written by this mount judged per field-by-field table, scan report unjudged, floor raising F unjudged from first write onward, F prior to that (including warm-up) judged as "mount did not raise floor"; position of first floor raising write determined by location of floor-raising sequences in mount write table, unlocated (`NotLocatedInTheMountWrites`) marks whole-segment F unjudged. Field-by-field table (R2 against R1):

| Field | Judgment |
|---|---|
| POSIX content and existence | Must be identical |
| Pool identity and immutable config in system configuration (except local device number) | Must be identical; 37 immutable rows compared byte-for-byte except local device number |
| Node size in system configuration | Must be identical |
| Runtime config 36 bytes in system configuration (T_time, T_dirty, three watermarks) | R2 equals built-in default (`to_slot` writes defaults, lacking mount option channel) |
| Unmount marker | R2 constantly "not written by unmount", R1 marker excluded from comparison |
| F and effective F | Mount did not raise floor: R2 effective F and root F equal R1 effective F; raised floor: R2 effective F equals last raised floor from mount |
| Instance code | max(self-verified system config instances on both slots across disks, readable root instances in root ring) + 1, read from crash image |
| Instance table row | Written per D18 (Information Carried in a Block) settled item 11 recovery rows |
| `checkpoint_txg` | Lower bound only: ≥ max(root ring roots txg, ring self-verified records `checkpoint_txg`) + 1 (D23 (Role and Format of Journal) settled item 14 lower bound, allowing skipped numbers) |
| Root slot position | Not judged separately, determined by `checkpoint_txg` |
| Tree ID watermark | Identical when mount does not create first file |
| Root pointers of inode tree, extent tree, tombstones, lineage, deleted_inodes in tree table (latter three are tree table entries, kinds 9, 10, 11, not root record fields) | Identical; if mount resumed, judged by post-resumption version: equals closest step root pointers where txg ≤ R2; tree table unit landing point not compared |
| Root pointers, birth txg, head ID, tree ID, kind for tree table kinds 6, 7, 8, 12 | Must be identical (roots for kinds 6, 7, 8, 12 constantly all-zero); birth txg across roots judged by I-9.14 (Tree Table Entry Birth TXG Invariant Across Roots), head ID by I-9.27 (Tree Table Entry Head ID Value) on single image |
| Instance table and tree table unit pointers | Byte comparison omitted, same as tree table unit landing point |
| Scan report | 0 applied, 0 above watermark |
| Reclamation threshold t and pinned slot p (root slot offsets 428..437) | Not compared: each publish carrying units recomputes them from this process's root ring and pins, and a zero-unit publish copies the previous version, so publishes made by the mount change them (same class as F and the unmount marker); single-image property, judged by pool-level checker I-7.18 (the root record's reclamation threshold equals the currently computed threshold) and I-7.19 (a pinned slot does not hold a root written by this instance) |
| Root slot offsets 437..466 (algorithm, nonce, MAC) | Must be 0; single-image property, judged by pool-level checker I-2.4 (Header Checksum Coverage), mount transparency does not compare |
| magic, flags remaining bits, self-verification checksum, whole-slot checksum | Not compared (structural, judged at slot read) |
| System configuration slot generation | Not compared, judged by I-7.16 (System Configuration Slot Generation and Slot Number Relation) on R2 image |
| System configuration journal_tail | R1 ≤ R2 ≤ max counter of self-verified records in R2 ring, equality holds when mount finishes |
| System configuration journal instance code | Equals R2 root instance code when mount finishes; secondary crash takes either R1 instance or new instance number |
| Allocation records (including root record `mapping_root`, `allocation_record_tree_root`, and tree table kinds 3, 4 roots) | Whole table comparison omitted: POSIX content identical and pool-level checker green on R2 image; `mapping_root`: R2 all-zero ⇔ R1 all-zero; `allocation_record_tree_root`: R1 has tree ⇒ R2 all-zero, R1 tree table 0 entries ⇒ (R2 non-zero ⇔ new instance number ≥ 2) |

System configuration runtime variables (slot generation, journal_tail, journal instance code) falling back to v1 are judged by their own row rules, not identical to v1.

**Scope**: Marginal value of refinement exists only in dimensions unattainable by crash-point replay enumeration—payload bytes, record counts / segment counts / txg counts (including ring wraparound), remnants of previous timeline, slot counts and device counts; crash-point dimension is already fully enumerated by crash-point replay. "Checksum passed" in model is boolean flag, **must not be read as refinement verifying integrity**—isomorphic to D4 (Checksum Placement) line in settled item 14 "must not read as verifying Merkle", and same informed acceptance regarding CRC32C collision probability as D13 (Verification Route) settled item 4. D13 (Verification Route) settled item 4 is `Crash` on left of refinement (enumeration domain of model layer), not `Spec` on right; treating it as spec makes refinement trivially true. Formalization of I-6.5 (Monotonic Nonce Watermark) withdrawn; reopened once D18 (Information Carried in a Block) settled item 14 settles and nonce allocation rules (monotonic counter + pre-allocated watermark, persisted before use) are established as settled item—when reopened, its sole true advantage is that nonces are invisible to POSIX, its spec structurally incapable of sharing origin with model of O1 (In-Memory Model Differential Testing (HashMap Ideal File System)), immune to Conflict 2 (D13 (Verification Route) settled item 13).

Premises and domains for M1–M4: semantic layer hinges solely on "same (instance, counter) corresponds to unique content across all timelines", guaranteed by verbatim re-issue in D23 (Role and Format of Journal) settled item 14 "failure handling of this version"; re-checked when root slot write failure advancing `checkpoint_txg` lands; premise domain takes legal history family of E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) (arbitrary landed subset of history written by legal writer, newer root slots may be unreadable). In M2 "remaining fields equal chosen root", root reconstructed from records has unmount marker `NotWrittenByTheUnmountSequence` (D22 (How Unit Atomicity Is Synthesized) settled item 7 "other roots write 0"), which may differ from chosen root marker, hence "except unmount marker". M3 monotonicity corollary holds only under serial commit, fault-free crash states. M4 together with root selection eliminates confirmed complete publishes discarded by recovery under complete premises: fault-free crash states (enumeration domain of settled item 4), chosen root per D22 (How Unit Atomicity Is Synthesized) settled item 7, chain not straddling instance boundaries (D23 (Role and Format of Journal) settled item 14 note 1), new instance roots written across two disks before any fsync returns (warm-up of D16 (Publish Semantics) settled item 8); not eliminated in faulty domain—in-flight bound truncates complete publishes, missing copy on dead disk prevents application (C519 (Losing Whole Disk Discards Just-Confirmed Version)).

Scope of mount transparency: applies only to fault-free crash states resulting in successful writable mount. Rejected writable mounts judged not for transparency, but legality of refusal: refusal legal ⇔ reason in D23 (Role and Format of Journal) settled item 14 list, recalculation holds on crash image (the refusal reason "system configuration unreadable again after the read phase has judged" carries the rewrite exception: on the D23 (Role and Format of Journal) settled item 14 list this refusal holds only when some disk has not a single self-verified system configuration and also fails the rewrite gate of D22 (How Unit Atomicity Is Synthesized) settled item 8; the judge recomputes only the "not a single one" half and does not recompute the five conditions of the rewrite gate; a disk state the rewrite gate lets through is not refused when it mounts and is judged by the crash-point replay observer (settled item 6); the slot self-verification judgment reads the pool-level checker's per-disk counts, not the core layer's parsing, implemented in the implementation batch), on-disk bytes identical, and read-only read in allowed set; unlisted reasons reported red per crash injection phase 2. Faulty states out of scope: unreadable slot at acquire time and N-config reported as same reason, separated before read-fault tier (C664 (Mount Refusal Legal Recalculation Conflates Two Refusal Reasons)). Tree ID watermark, two tree root pointers, `checkpoint_txg`, and scan report in field-by-field table are inferred; floor raising "raised" half unmeasured; `mapping_root`, `allocation_record_tree_root`, tombstones, lineage, deleted_inodes pointers, algorithm, nonce, MAC, slot generation, and journal_tail classified per C655 (Mount Transparency Field Table Has Unclassified Fields) and C687 (Placement Unjudged for Initial Tree Creation Version) verdicts Y1, Y2 in table.

**Basis**:
- E42 (How Many Records per Transaction): Payload pinned to make "split ⇒ illegal" an identity, structurally blind to "split yet coincidentally legal"—M2 symbolization of payload covers this gap.
- Three-party review: background `_d13-item1-background.md`, outputs `d13-item1-opus-forward-output.md`, `d13-item1-sonnet-reverse-output.md`, `d13-item1-local-attack-v2-output.md`; all three legs ranked former candidate last, cloud legs converging on "prefix judgment and commit atomicity", main agent combining root selection and publish atomicity into single unit (verdict and threshold in history).
- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound): Judges M2 semantic layer, M4, and M3 readings on real `replay_journal` within bound, measuring mutation kill bound; user ordered it as lightweight tool on 2026-10-01; C-domain crash state count divided into two cuts (cut A is barrier position of D16 (Publish Semantics) settled item 7); fixed-point violations in adversarial family strictly on record tables violating I-8.12 (Non-Decreasing Journal Record TXG for Same Instance), 0 violations when predicate holds (rerun 3 part 1); root-selection joint-verdict family calling real `choose_root`, `scan_journal`, `replay_journal` retains confirmed complete publishes within bound, legally and fault-free (rerun 3 part 2); four buggy root-selection variants on legal histories K, K9, KW show only "take first valid" red, other three red on adversarial root ring KA (rerun 3 part 3).
- Three-party verdicts (M1–M4 phrasing and placement): `research/prompts/m3-formal-r1-main-verification.md`, `research/prompts/m3-formal-r2-main-verification.md` Section 2, `research/prompts/m3-formal-r3-main-verification.md` Sections 2, 3; M3 monotonicity 4-step derivation in `m3-formal-r2-opus-output.md`.
- Three-party verdicts (M3 fixed-point premise adding I-8.12 (Non-Decreasing Journal Record TXG for Same Instance)): `research/prompts/m3-formal-code-r1-main-verification.md` Section 2 fixed-point precondition and Section 3 item 5; harness premise wider than Verus, reconciliation in scheduling record `records/2026-10-01-里程碑三形式化接入.md` batch 10.
- User decision (2026-10-01 popup, all per recommendation): unmount marker excluded, M4 scope premises stated fully, monotonic version order takes (`checkpoint_txg`, instance), txg saturation takes "report no_successor with zero applied counts as equal"; write-back deferred until K10 verdict, original wording in history.
- Mount transparency: no experiment: judged in crash injection (`crates/singlefs-harness/src/mount_transparency.rs`, called by checker tier crash injection), not experiment; attack numbers measured on replica harness across three verdict rounds, round 2 measured 147,806 initial crash states and 234,773 secondary crash states with 0 red on merged code without main agent rerun.
- Three-party verdicts (mount transparency): `research/prompts/m3-formal-r4-main-verification.md` Section 2 and Section 4 items 2, 3; `research/prompts/m3-formal-r5-main-verification.md` Section 2 U5, U6 and Section 4 items 5, 6; `research/prompts/m3-formal-r6-main-verification.md` Section 2 U5, U6 and untouched fields line; `research/prompts/m3-formal-code-r2-main-verification.md` Section 2 K2 lines.
- User decision (mount transparency plus secondary crash, 2026-10-02 popup, original wording in history): `research/prompts/m3-formal-r4-main-verification.md` user decision line.
- Root selection variant sentence: scheduling record `records/2026-10-01-里程碑三形式化接入.md` batch 12 E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) rerun 3 part 3 Q2 handback (main agent added this sentence).
- Mount resumption counted as part of mount, F bound located at end of write table: user decision 2026-10-03 "resumption is part of mount" (original wording in history); code round 3 verdict `research/prompts/m3-formal-code-r3-main-verification.md` Section 3 item 3; implementation `crates/singlefs-harness/src/mount_transparency.rs`.
- In the refusal-legal sentence, the recomputation of the refusal reason "system configuration unreadable again after the read phase has judged" carries the rewrite exception, and the slot self-verification judgment reads the pool-level checker's per-disk counts: three-party verdict `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` Section 2 rows Y1, Y3 and the rows "clause" and "slot self-verification judgment of the G4 refusal arm" of the user table in Section 3; user decision 2026-10-22 "switch to reading the pool-level checker", zero rounds, original wording in history.

**Owed**: C77 (Replay start undefined): at seam of root selection and prefix judgment; C649 (Two callback results unverified by Verus): paid off (M2 semantic layer back-pointers and named units remain postconditions unjudged by Verus, covered by mutation lines in `crates/mutations.tsv`, see settled owed table); C650 (Root selection and replay lack joint verdict): M4 scope sentence requires joint verdict to hold; C653 (Ring scan glue memory scales with disks); C654 (Must-red world expected lines drift with solver environment); C655 (Mount transparency field table has unclassified fields): paid off (table completed with padding per verdicts Y1, Y2, see settled owed table).

#### Settled Item 2: Serial Commit First

**Settlement**: Transaction layer commits to serial commit first—implementing `.claude/rules/fs-design.md` rule "when criteria cannot be written, implement only one path, and **implement the slow but correct path first**"; serialization is the explicit default of this project, not an arbitrary pick. Concurrency opens only past the intersection, never promised before it. Future concurrent commit constraint: records appended in transaction number order, fsync for transaction N must wait before return until transactions < N are all appended ∨ invalidated ∨ confirmed instance switch or read-only transition (invalidation is in-memory state of commit pipeline, visible to concurrent successors); this rule idles under serial commit.

**Scope**: Serial ceiling is ~11,000 entries per checkpoint (intersection where single-thread merge time catches up to one device sync, formula in C58 (Serial Commit Hitting Ceiling Lacks Alarm)): below intersection device sync is bottleneck and serialization suffers no penalty; above intersection single-thread merge becomes ceiling with zero safety margin. Settled item 2 does not decide record header field list or alter bytes written by new-pool file creation; it dictates performance ceiling and scope of crash refinement. Concurrent commit will affect crash semantics: "prefix" is total order concept; under concurrency, a prefix may contain complete transaction B alongside truncated transaction A; record header already carries transaction boundary fields (D23 (Role and Format of Journal) settled item 7: transaction number 8 + commit marker 1).

**Basis**:
- E17 (Write Buffer Merge Upper Bound): Single-thread sort-merge ceiling is half the intersection; lock-free parallelism per leaf partition is escape hatch past intersection, but assuming it sufficient is forbidden.
- E44 (Local Measurement and Cost of Sequence Width): Local fsync rate is other half of intersection.
- E42 (How Many Records per Transaction): Under serialization, multi-record transactions can still be split by prefix—record header field list dictated by whether transaction produces exactly one journal record, not serial vs concurrent.
- User decision, recorded in history; future concurrent constraint derived from C113 (Current Version Input Missing for Rebuild Multi-Version Units) settlement P1. Unreviewed by three-party: confirms explicit default; review required when switching to concurrency.

**Owed**: C58 (Serial commit hitting ceiling lacks alarm).

#### Settled Item 3: RefFS Taken to the Extreme, Handling Persistence and Crash Semantics Itself

**Settlement**: Refuses functional-only memory model, building a reference implementation (RefFS) handling persistence and crash semantics itself, elevating RefFS from "reserve candidate" to **active commitment**. "Taken to the extreme" means semantic level: RefFS operates in memory strictly via POSIX operations, defining what counts as persisted after each fsync and which POSIX states are permissible after crash, having no on-disk format; disk-writing and disk-recovering tier (effort approximating building another file system) is rejected. RefFS and implementation forbidden from sharing crash-point enumeration code: "three-oracle collusion test" (settled item 6) guards against shared misunderstanding, most critically "whether RefFS and implementation share identical flawed understanding of crash semantics".

Provenance and boundaries of RefFS:

| Item | Settlement |
|---|---|
| Provenance | Written from scratch per kb text only: does not copy code of O1 (In-Memory Model Differential Testing (HashMap Ideal File System)), does not read recording stream, does not invoke segmentation, state planner, or crash enumeration of implementation |
| Placement | Isolated crate `crates/singlefs-reference-filesystem`: dependency closure (including dev-dependencies) ⊆ {itself, `singlefs-format`}, preferring std where possible; source code does not reference `singlefs_core`, `singlefs_checker`, `singlefs_harness`, `singlefs_checker_tier` (including renamed aliases); neither `singlefs-core` nor `singlefs-checker` include it in dependency closure. Judged by gate checker-independence-and-sync cell checker-implementation-disjoint |
| Input whitelist | Provided by caller, accepts five items only: outcome of each operation (success, rejected before write, errored after write, re-issued as-is); operation index where crash occurs (hi) and its attribution rule; rollback target m; content read on each recovery; explicit publish points |
| First-version operations | First file, overwrite, publish call, fsync, rollback |
| Call sites | At call sites of crash injection and fault injection, RefFS verdict conjoins with all four criteria of O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge; any failure flags red |

Crash semantics (serial commit, fault-free crash states):

1. **fsync**: Upon successful return of fsync, all operations carried by the publish awaited by this fsync must be present (under serialization, all operations with acquired numbers before call began, counted in units); read by state index, raising lower bound lo of allowed set to end of that publish. Errored fsync does not raise lo, operations of errored publish optionally present; subsequent operations judged against state read on next recovery.
2. **Drop rule: prefix by publish boundary**. When crash occurs during operation hi, recovery permits reading {S_j : j = lo, or j is end of an explicit publish; lo ≤ j ≤ hi}, where S_j is state resulting from applying first j operations after previous recovery, and S_0 is content read on previous recovery. Dropping arbitrary subsets is excluded under serial commit (incompatible with D16 (Publish Semantics) settled item 12), reopened under concurrency.
3. **Raising lo**: Successful publish calls, fsync, and rollback all raise lo (D16 (Publish Semantics) settled item 7: rollback and fsync share return condition); rollback counts as confirmed publish, its index marking publish end.
4. **Rollback**: Rollback is an operation in RefFS; post-rollback content equals content following m-th confirmed publish across entire history; m passed in by caller from root reported by implementation, RefFS does not judge whether target selection is correct. m covers every root-writing entry point (publish call, rollback, write row and warm-up, raise F, unmount), mapping to active version when written, active version including that read on recovery and restarted.
5. **Attribution of crash moment**: Operation index hi where crash occurs is attributed by "which step issued the barrier cutting current segment", not by first intercepted write; used only by RefFS, O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge untouched.
6. **Post-recovery**: Following each recovery, RefFS first verifies that read state lies within allowed set, then advances from that state as starting point.

**Scope**: Settles ambition level and form of RefFS, not scheduling; must not be read as "building RefFS before implementation"—RefFS is verification vehicle, not milestone (`.claude/rules/format-evolution.md` "start from writing, not reading"). Explicit publish points take only publish calls and fsync in history: until automated batch accumulation lands, RefFS interfaces only with explicit publish points; automated publishes outside RefFS scope, belonging to settled item 1 M2 and record checker; recoveries discarding newest root cause RefFS to restart from read content without judging that step. Faulty crash states (lying devices, whole disk loss C519 (Losing Whole Disk Discards Just-Confirmed Version)) inapplicable. Independence: RefFS reads no recording streams, calls no segmentation or state planning, but its (lo, hi) shares origin with inputs to O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge disk-level lower bound criterion, logged under C191 (Common-Origin Counter Covers Only Single Pair). D17 (Implementation Layering and Third-Party Pipelines) settled item 4 delegates executable definition of "entry" to RefFS operation surface, fsync persistence defined by RefFS crash semantics; O3 (Independent Spec Executor) remains independent executor of crash sets with unchanged role. First version of RefFS in `crates/singlefs-reference-filesystem`, connected to crash injection phases and fault injection: harness tier depends on it via translation layer `crates/singlefs-harness/src/reference_filesystem_translation.rs`, call sites taking votes from both RefFS and O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge, flagging red if either fails.

**Basis**:
- No experiment: Ambition level, provenance, and inputs of RefFS are trade-offs between verification investment and boundaries; attack numbers from three review rounds measured on replica harness, unrepeated on main repo (`research/prompts/m3-formal-r5-main-verification.md` Section 5 item 3), uncited here.
- User decision, original wording in history: "take to the extreme" (original settlement); semantic level (2026-10-02, fork ticket `m3-formal-7-forks.md` RefFS level line, reinterpreting original settlement); prefix by publish boundary, call site conjunction, scratch rewrite, isolated crate (2026-10-02, `research/prompts/m3-formal-r4-main-verification.md`); conjunction tightened to all four criteria, rollback target as input (2026-10-02, `research/prompts/m3-formal-r5-main-verification.md`); rollback raises lo, crash moment attributed by barrier, m covers all root entries, whitelist adds re-issue (2026-10-02, `research/prompts/m3-formal-r6-main-verification.md`).
- Three-party verdicts: `research/prompts/m3-formal-r4-main-verification.md` (Round 1: fsync, drop rules, independence, six forks), `research/prompts/m3-formal-r5-main-verification.md` (Round 2: fsync synthesis sentence, publish boundary supplements, conjunction tightening, isolated crate, arbitrary subset rejected under serial), `research/prompts/m3-formal-r6-main-verification.md` (Round 3 final: rollback and lo, scope sentences, whitelist, fsync read by state index).
- D17 (Implementation Layering and Third-Party Pipelines) settled item 4: O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) explicitly states "ignores crash", incapable of defining fsync persistence promise, whereas first runnable target is "correctly committing one transaction"—extreme RefFS fills this exact half.

**Owed**: C447 (Spec executor not connected to crash-point replay to compare against RefFS); C191 (Common-origin counter covers only single pair); C658 (Crash injection third phase permits resurrection of dropped step).

#### Settled Item 4: Crash-Point Replay Enumerates Full Write Subsets Only

**Settlement**: The crash state set to be enumerated by crash-point replay is defined as: recorded write request stream is segmented **by device**—a write is ordered before subsequent writes only by subsequent barriers on its own disk (or FUA writes on its disk); current segment closes only when every disk with writes in the segment is released by its own barrier or FUA; if any disk has unreleased writes, that barrier (or FUA) does not close segment, and writes before and after belong to same segment. A crash state is "preceding segments all persisted, each write in current segment takes one of its states in arbitrary combination, subsequent segments all non-persisted"; enumeration unit is **one full write**. FUA writes release only their own disk: grouped in same segment with preceding unseparated writes on that disk, writes on other disks not persisted thereby. A write takes two states (persisted / non-persisted); **in-place overwrites** (non-unit writes longer than a sector covering pre-existing content, such as system configuration slot writes and journal writes overwriting old records) take a third state "neither new nor old readable", enumerated in both full run and quick tier; third-state images cover entire write range, differing bytes split with first half new bytes and second half old bytes (torn by byte, not sector); torn states of other writes collapsed into "write non-persisted". In "non-persisted" positions, images retain **pre-crash old bytes at that position**, not zeroes, and these images (including third-state) are concretely generated and fed to checker and record checker, never treated as mere counters. Mirrored writes (w ≥ 2 of D2 (RAID Striping Strategy) settled item 6, journal mirroring of D22 (How Unit Atomicity Is Synthesized) settled item 8) counted as distinct writes per **device**, harness state counts computed via closed form per write stream. **Adjacent barriers on same disk collapsed into one**: when recording stream contains consecutive barriers on a disk without intervening writes (barriers on other disks do not count as writes), second barrier omitted (`push` in `crates/singlefs-harness/src/lib.rs`), independent model recording identically; lacking intervening writes, segment produces zero crash states, merging affecting only operation count and kind string without altering crash state set.

**Scope**: Defines enumeration domain at model layer. Whether FLUSH / FUA take effect on real devices as advertised belongs to C6 (Block Layer Semantic Assumptions Flawed), verified in QEMU with real devices. Segmenting by device yields states unreachable in practice (writes released on other disks remain combinable in segment when one disk unreleased), without missing reachable states. Tearing falls into two categories: writes outside in-place overwrites (unit writes, sub-sector writes, writes over empty ranges) differ from "non-persisted" solely by CRC32C collision probability (32-bit, D19 (Block Pointer Structure and Width Budget) settled item 2, D23 (Role and Format of Journal) settled item 11), an **informed acceptance**; tear granularity excluded from model, consistent with D20 (Bearing Surface: Unit Atomicity and Self-Containment) "units with parent pointers depend on no width"; in-place overwrite tears lose old content as well, whereas "non-persisted" retains complete old content, representing distinct states, hence enumerated separately. Crash-point replay sectors take 512 bytes; system configuration slot writes are sole source of third-state on both crash replay streams. Known corner case: third-state old bytes take version where "writes prior to this write fully landed", combinations where earlier overlapping writes in same segment failed to land contain extra bytes (such combination occurs zero times across both streams); multi-sector root slot writes over old content do not support third state in first version (harness assertion; pools built with 512-byte physical blocks, unreachable). Device tearing writes to match checksum collisions excluded from model. D13 (Verification Route) settled item 5 orthogonal: one defines enumeration domain, other defines crate boundaries.

**Basis**:
- E77 (Publish Persistence Order): State count of segment model across four barrier arrangements matches closed form per arm—enumeration domain counted correctly.
- E142 (File Creation Dry Run on New Pool): Rerun 17 part 1 geometry sampling point G7 (positive control on same stream treating only FUA as boundary) measured 4097 states, 2046 violations—lacking barriers, units, journal records, and root slots merge into single segment `[12]`; harness segmentation aligns with `crates/singlefs-harness/src/segments.rs:91` "FUA write closes segment containing it (FUA does not persist preceding writes, grouped together in arbitrary subset)", predicting 2050 / 0 contradicts "FUA forms own segment"—FUA text written per implementation.
- Three-party adversarial review and verdict: [verification-build.md](../verification-build.md) "Three-Party Adversarial Review (2026-09-03)" verdict item 3—every unit class has whole-unit validation (parent-pointed rely on parent checksum, D4 (Checksum Placement) settled item 1, D16 (Publish Semantics) settled item 7, D22 (How Unit Atomicity Is Synthesized) settled item 19; self-verified rely on whole-unit checksum plus checked generation, D22 (How Unit Atomicity Is Synthesized) settled item 21; journal records rely on header checksum and payload checksum, D23 (Role and Format of Journal) settled item 13); torn and non-persisted are identical to checksum.
- User decision (settled per E77 (Publish Persistence Order) and three-party review; FUA segment boundary sentence settled at C313 (Whether FUA Write Counts as Crash Segment Boundary) closeout, updated 2026-09-26 to match implementation "closes containing segment, grouped with preceding unseparated writes"), recorded in history.
- Device segmentation (unreleased disk keeps segment open), FUA releasing local disk only, in-place overwrites taking third state "neither new nor old readable": user decision (code review items 1, 3, 4; 2026-09-27 popup selecting "A merge, track barriers per device", "backfill, run full run", and "in-place overwrite adds third state"); third-state byte-tearing settled by main agent same day, 0 rounds attacked, recorded in history; implementation and discriminative power in `m2-rev-b3a2-implementer-report.md` Sections 1, 3.
- Collapsing adjacent barriers on same disk: user settled 2026-09-28 "collapse, follow today's recorder" (E142 (File Creation Dry Run on New Pool) rerun 19 hit registered stop S19-crates: independent model recorded both, recorder collapsed second, warm-up and full stream differing in op count and kind string but identical in state count, report `e142-r19-runner3-report.md`, prereg `e142-r19-prereg.md` R40), original wording in history.

**Owed**: C6 (Block layer semantic assumptions flawed): reconstructing crash states from VM device logs, where crash tests must turn red.

#### Settled Item 5: Sharing Only a Single Constants Module Generated from kb

**Settlement**: Checker and implementation share exactly one artifact: a constants module generated from kb field tables, where generator reads kb and both sides consume without manual modification (`.claude/singlefs-ai-sop/rules/machine-first.md`: generate repetition, do not copy manually). **All else strictly unshared**: address space newtypes declared separately, format parsing implemented independently, checksums use independent implementations, traversal and accounting code share zero intersection (criterion for C12 (Incremental Semantics Shared) defined by gate checker-independence-and-sync cell checker-implementation-disjoint, enforced as crate-level dependency closure disjointness rather than symbol level; exception "except format parsing and constants" exercised by this item).

- **Decision width must not be obtained elsewhere**: Criteria for I-7.5 (Root Slot Aligned to Decision Width) and I-8.2 (Record Header Does Not Straddle Atomic Unit) depend on `physical_block_size` probed at mount, which is not in image. Checker probes independently, comparing against value recorded in system configuration at mkfs (checker-side form of D22 (How Unit Atomicity Is Synthesized) settled item 2 "mkfs bounds by pool maximum, mount recomputes and compares per device"). **When unprobed, report "declared value, unprobed", never treating as probed**—images backed by files fall into this class, comprising majority of crash-point replay images, hence non-trivial. In heterogeneous pools decision width is a set rather than scalar (C17 (Device Geometry as Global Parameter)), checker recomputing mkfs maximum aggregation to avoid false reds on healthy heterogeneous pools.
- **Generator refuses to emit constants lacking kb placement**, emitting scalar values only: if it emits "offset functions computed from field tables", it becomes format parsing explicitly forbidden from sharing by D13 (Verification Route); criterion is presence of branches or arithmetic in emitted code.
- **Independent CRC32C implementations require pinned parameters** (polynomial, initial value, input/output reflection, XOR output): unpinned parameters yield divergent results on healthy images, producing false reds rather than independence.

**Scope**: Governs crate boundary between checker and implementation; cannot catch erroneous values written in kb itself—shared generated constants and separate manual copying both miss that class, requiring orthogonal gate (C94 (Registered Format Constants Conflict with Later Settlement)). Implementation gap: `crates/singlefs-format` is handwritten rather than generated (values bound by gate code-source-discipline format-constants cell per `format-const` markers in kb, binding marked ones only), generator absent; `crates/singlefs-checker` depends solely on `singlefs-format`, rewriting bitwise CRC-32C; checker reads `physical_block_size` and `io_min` declared in system configuration (`geometry_of` in `crates/singlefs-checker/src/image.rs`), "probed / declared" types defined as `DecisionWidth` in `crates/singlefs-checker/src/lib.rs`: declared form (`DeclaredOnly`) connected to I-7.5 (Root Slot Aligned to Decision Width) judgment (`decision_width_declared_by_the_chosen_system_configurations` taking max(`physical_block_size`, `io_min`) across chosen configs), probed form (`Probed`) exists only as type definition without constructors or connected judgments; I-8.2 (Record Header Does Not Straddle Atomic Unit) unimplemented. `physical_block_size` field at mkfs resides in geometry segment of D22 (How Unit Atomicity Is Synthesized) settled item 9 (4 bytes).

**Basis**:
- No experiment: Sharing boundary is verification independence policy, lacking measurable quantities; discriminative power demonstrated by separate implementations flagging red individually.
- Three-party review: background `_checker-sharing-vehicle-background.md`, outputs `research/prompts/checker-sharing-vehicle-*`, detailed in [verification-build.md](../verification-build.md)—forward leg derived "constants within exception, rest forbidden" from D13 (Verification Route) and C12 (Incremental Semantics Shared); alternate reverse leg and local leg independently constructed finding: errors written in kb missed by both sharing and separate copying.
- User decision, recorded in history.

**Owed**: C17 (Device geometry as global parameter); C387 (Constants module handwritten, not generated from kb); C388 (Checker decision width not self-probed).

#### Settled Item 6: Three Mutually Independent Oracles

**Settlement**: Three oracles each judge distinct aspects, their independence deriving from different sources:

<!-- doc-lint:registry name-col=2 -->

| # | Oracle | What It Judges | Source of Independence | Blind Spots |
|---|---|---|---|---|
| O1 | In-memory model differential testing (HashMap ideal file system) | POSIX visible semantics: content, size, directory structure; content expectation without crash belongs here, permissible post-crash states belong to RefFS (settled item 3) | Touches no disk, parses no disk format | On-disk structural self-consistency; crash semantics; **shared misunderstanding of same POSIX semantics between two implementations** |
| O2 | Independent parser + checker | On-disk bytes satisfy I-1 (Block Self-Description) ~ I-6 (Encryption Invariants) | **Checker calls no deserialization/traversal code of tested crate**, implementing independent parsing directly reading raw device | Pure structural self-consistency only: overwriting block with alternate **valid** content leaves Merkle chain and accounting intact |
| O3 | Independent spec executor | Truncated disk state at each crash point falls within specification-allowed set | **Written in another language** (non-Rust), derived solely from decisions and invariants text; first version in Python (E170 (First Version of O3 Spec Executor)) | Spec itself may be flawed—**meaningful only with positive control**; projected values and unprojected root fields, implementation's own recovery/reader code, system config witness values (journal_tail), write order within publish |

**Independence must be verified, never self-proclaimed**: Design a "three-oracle collusion test"—feeding an intentionally flawed implementation (e.g. constant zero nonce) through all three; if all pass, a shared blind spot is revealed, requiring oracle enhancement. Functional correctness does not rely solely on these three: O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) delegates permissible post-crash states to RefFS (settled item 3), recovery function M2–M4 rests on Verus and E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) (settled item 1), and checks requiring second inputs belong to record checker (settled item 7); shared blind spots identified via "three-oracle collusion test".

**Scope**: Blind spots of each oracle tabulated in final column; domain of O2 (Independent Parser + Checker) belongs to settled item 7, dual-input checks to record checker. **Current implementation status**: O2 (Independent Parser + Checker) in `crates/singlefs-checker` (dependent only on format constants crate); O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) implemented as ideal model in `crates/singlefs-harness/src/model.rs` (memory-only, uses `singlefs_format` constants, models no crashes), gate harness-model-differential-and-scenarios running differential testing on random history; O3 (Independent Spec Executor) is Python executor written per kb text, active version being `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/` used in E171 (Three-Oracle Collusion Test) rerun 1 (E170 (First Version of O3 Spec Executor) rerun 2 copied `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` to `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` adding predicates W, T, then updated 487-byte journal header; rerun 1 archive and original `scripts/experiment-apparatus/e170_o3_spec_executor/` retained): strict execution judges successfully (0 states hit "spec missing clause" on batches 1, 2), state-for-state consistent with implementation (all pass, not evidence of correctness; "selected root not max txg among self-verified roots" and "③ received ≥ 2 slots" paths have 0 states, C661 (Spec Executor Lacks States on Two Paths)), bad state table has discriminative power under corrected expectations (positive control and blind test correct), not yet connected to crash-point replay; judging a crash state: independently recovers version from image per spec recovery rules (system config selection, root selection, prefix per D23 (Role and Format of Journal) settled item 14 five rules), judging not in set if unrecoverable (pool rejection or no self-verified root); projection of recovered version (instance, checkpoint_txg, tree table pointer, central mapping root pointer, tree ID watermark, F) must match projection of a version in version table, with index in [confirmed, started]. Version table consists of active version recovered on base image plus each root slot write in recording stream, projections matching bytes written by implementation; confirmed takes latest version returned before latest landed write; started takes latest version of landed write. System config slots used only to choose slot, determine geometries, and reject out-of-bounds pool fields. Recovery rules written per kb text, allowed set drawn from implementation's own written roots; RefFS (settled item 3) in `crates/singlefs-reference-filesystem`, connected to crash injection and fault injection; "three-oracle collusion test" is E171 (Three-Oracle Collusion Test), read from frozen tree (snapshot summary on experiment page, differs from active `crates/`): O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) column uses model differential plus model crash judge without RefFS (post-crash set half not using RefFS in E171 (Three-Oracle Collusion Test)); O3 (Independent Spec Executor) column uses E170 (First Version of O3 Spec Executor) rerun 1 frozen version; handcrafted bad images fed to O2 (Independent Parser + Checker) and O3 (Independent Spec Executor) only, expectations from E170 (First Version of O3 Spec Executor) designer, Q2 judging "present" per registration (all 4 are P8), judged per F3 as flawed expectation in E170 (First Version of O3 Spec Executor) rather than bad images, zero genuine bad images passed by both O2 (Independent Parser + Checker) and O3 (Independent Spec Executor); independence of origins (table col 4) measured only in Q2 between O2 (Independent Parser + Checker) and O3 (Independent Spec Executor), where patch and expectation derived from E170 (First Version of O3 Spec Executor) code, not treated as independent; O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) excluded from Q2; unmeasured aspects not claimed verified.

**Two aspects unjudged by all three oracles** (the first aspect is judged on two sides, crash injection and crash-point replay; the observer on the crash-point replay side is in the implementation batch): first, on fault-free crash states, D23 (Role and Format of Journal) settled item 14 writable mount witness criterion holds (witness in system config slot precedes readable records on disk): crash injection phase 2 verifies refusal legality by recalculating criterion on crash image, judging legal if true (mount transparency applies to fault-free states, settled item 1); judge required to ask "crash state lacks injected read fault, why does criterion hold": crash injection phases 2, 3 recalculate criterion on fault-free crash states, judging illegal refusal if true (`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults` in `crates/singlefs-checker-tier/src/crash_injection.rs`, C670 (No Judge for Writable Mount Witness Criterion True on Fault-Free Crash State) paid off, see settled table). Property "every disk has at least one self-verified system config on fault-free crash state" judged by the observer on each stream of crash-point replay after mkfs returns: the observer is registered per stream, not attached to the stream recording mkfs, and records only disks that newly appear relative to the base image with not a single self-verified system config, torn domains included; the slot self-verification judgment reads the pool-level checker's per-disk counts (`system_configuration_redundancy_report` in `crates/singlefs-checker/src/image.rs`), and one registration completeness test case judges that every stream containing a system config slot write segment has it attached (implemented in the implementation batch). In a random history, the shape that fails the rewrite gate (D22 (How Unit Atomicity Is Synthesized) settled item 8) is judged by the G4 refusal arm in crash injection phases 2, 3; the shape that passes the rewrite gate is judged by nobody in a random history, and a correct implementation cannot leave such a disk state (neither of two samplings hit it). Crash states before mkfs returns are outside the scope of the observer and the G4 refusal arm: mkfs writing two slots per disk in single loop before disk barrier, crash injection candidate segments and E171 (Three-Oracle Collusion Test) base images chosen post-mkfs; the G4 refusal arm differs in one more shape from C670 (No Judge for Writable Mount Witness Criterion True on Fault-Free Crash State): when no system config selectable, recalculation yields `None`, G4 treats as true, read phase having rejected before reaching cell. Second, write order within publish: I-7.12 (System Configuration Floor Not Lower Than Root on Same Disk) of O2 (Independent Parser + Checker) judges step of writing system config first when raising floor; mutation cases test line 270 of E171 (Three-Oracle Collusion Test) (witness value at crash points post-publish return), line 242 case testing failure path freeze; `layout/01-first-txn.md` Section 8 registers root slot FUA and system config rotation as two segments; other write sequences unjudged by the three.

**Basis**:
- E170 (First Version of O3 Spec Executor): O3 (Independent Spec Executor) successfully implemented per "different language, derived solely from kb text, paired with positive control", judging single crash states fast enough for quick tier; rerun 1 (spec executor updated per 2026-10-03 kb) judged strictly, matching implementation on sample states with discriminative positive controls (unrunnable states and weak discriminative power of run 1 detailed on experiment page).
- G4 (every disk has at least one self-verified system config on fault-free crash state): Code round verdict `research/prompts/m3-formal-code-b26-r1-main-verification.md` Section 2 j2-b established, batch 29 implemented, scope and mkfs segment in `research/prompts/m3-formal-code-b29-r1-main-verification.md` Section 2 L3 and Section 3 item 3.
- Disk states with both slots dead are now judged by the observer on each stream of crash-point replay after mkfs returns, and the G4 refusal arm stays: three-party verdict `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` Section 2 Y2 rows and Section 3 (earlier two rounds `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`, `research/prompts/m3-close-g4-rewrite-r2-main-verification.md`); the observer's wiring (registered per stream, relative to the base image, registration completeness test case) is user decision 2026-10-22 "register per stream + relative to base image", zero rounds, original wording in history. Numbers measured on the three-party legs' models and replica prototypes; no experiment number assigned.
- Division of labor and independence sources of three oracles are architectural design of verification system; independence observed via "three-oracle collusion test" (E171 (Three-Oracle Collusion Test)).
- E171 (Three-Oracle Collusion Test): On frozen tree, flawed implementations table contains rows passed by all three oracles and rows caught by one oracle only, O3 (Independent Spec Executor) catching zero rows in flawed table; handcrafted forged roots and continuation images include cases passed by O2 (Independent Parser + Checker) but rejected by O3 (Independent Spec Executor) (readings on experiment page Q1, Q2, Q3).
- Three-party argumentation: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.
- Division of labor between O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) and RefFS: three-party verdict `research/prompts/m3-formal-r5-main-verification.md` Section 2 isolated crate cell (both legs aligned).

**Owed**: C445 (Model differential sampling points do not scale with write paths); C446 (Unimplemented invariants lack precondition registry and expiry); C447 (Spec executor not connected to crash-point replay to compare against RefFS); C407 (Three-oracle collusion test judge not in gate): E171 (Three-Oracle Collusion Test) measured once, zero gate cells read output; C672 (Erroneous return value or memory state with unchanged write stream); C679 (Unmount marker excluded from model differential); C874 (The rewrite branch cannot enter crash tests and is invisible at run time): neither the observer for disk states with both slots dead nor the G4 refusal arm's switch to reading the pool-level checker is implemented yet.

#### Settled Item 7: O2 (Independent Parser + Checker) Judges a Single Image

**Settlement**:

> **Settlement: The domain of O2 (Independent Parser + Checker) is a single image**—a unary predicate `P(image) → satisfied/unsatisfied`,
> judging set membership only, never identity. **Any check requiring a second input (another image or a recording stream) does not belong to it.**

**Checks requiring a second input belong to record checker**, with arguments `(pre-crash image, recording stream, post-crash image, persisted set)`—persisted set denotes writes landed in this crash state provided by enumerator (segment full write subset in crash-point replay); on-disk bytes alone cannot distinguish "write not landed, in-place content coincidentally identical" from "write landed", judging reuse and absence requiring persisted set; belongs to binary checks alongside mechanism 1 "Merkle root chain diff" (settled item 8). ⚠️ **Unnumbered by design**—confusion surrounding O2 (Independent Parser + Checker) was caused by using numbers as names; adding O4 merely provides another anchor for future drift. ⚠️ **"Post-crash image" consists of two copies, not one**: harness sequence executes implementation's recovery first, then record checker, recovery itself modifying disk (instance switch writing rows, re-issuing in-flight checkpoints, D23 (Role and Format of Journal) settled item 14) ⇒ root selection and prefix judgment inspect **crash-state image** ("recording stream" being journal records in its ring), compared against **implementation post-recovery image**.

**Scope**: In pointer layer target state, checking a record requires only format parsing + checksum + root ring selection; logical intent records name no on-disk objects, requiring checker to implement tree semantics + accounting + allocation validity—D23 (Role and Format of Journal) rejection of logical intent logs rewritten as guard on first reason in D23 (Role and Format of Journal) settled item 5. Record checker code in `crates/singlefs-checker-tier/src/crash.rs`, connected to each crash state in crash-point replay (gate checker-tier-crash-point-replay). Record checker judgment is thus not pure function of image: two states with identical post-crash bytes but different persisted sets may be judged differently, invalidating image-deduplication speedups (O2 (Independent Parser + Checker) and oracle inspect image only). Crash-point replay excludes writable mount: runs read-only recovery only (with and without journal) on each crash state; acquire number, write row, warm-up never run in crash-point replay. Writable mount covered by crash injection (`crates/singlefs-checker-tier/src/crash_injection.rs`): on each sampled crash state, following read-only recovery and judgment, mounts writable on same post-crash image, publishes once, runs pool-level checker; crashes again during mount, placing secondary crash state in acquire number, write row, warm-up phases, executing read-only recovery, querying model, pool-level checker, record checker: phase 3 write table passed to record checker switches from "prefix from mount segment to current segment" to full history stream + full mount stream + post-mount publish, persisted set corresponding segment by segment (history takes phase 1 persisted set, mount takes secondary crash subset, post-mount publish unlanded). "Compared against implementation post-recovery image": record checker accepts two images, crash-state image judging root presence and record presence (root selection and prefix), post-recovery image judging unit presence and reading landed instance table; phase 2 post-recovery image is pool after mount and publish, claimed version taking newest root written by mount and publish, falling back to version landed in phase 1 if mount wrote zero roots.

**Basis**:
- No experiment: **Basis is type, not cost**: "byte-by-byte comparison" is binary relation `R(image₁, image₂)`; **no amount of investment turns unary predicate into binary relation**; I-4.5 (Deprecated: Requires Two Images) retired on identical threshold.
- "Post-crash image" is two copies: settled by user during D16 (Publish Semantics) settled item 4 scope narrowing review (background `_d16-items134-background.md`), recorded in history.
- Persisted set as input: user decision, original wording in history; discriminator unobservability evidence in `research/prompts/m2-layer0-scale-r3-main-verification.md` Section 2 sector exemption line.
- Crash-point replay excludes writable mount, covered by crash injection: user decision (code review item 2, 2026-09-27 popup selecting "crash injection layer covers writable mount"), recorded in history; crash injection implementation in `m2-rev-b3b-implementer-report.md`.

**Owed**: None.

#### Settled Item 8: Two Custom-Developed Mechanisms: Merkle Root Chain Diff and Generational Incremental Scrub

**Settlement**: Two mechanisms unavailable off-the-shelf and custom-developed for this project:

**1. Merkle root chain diff —— Leverage unique to COW + new root per transaction.**
Historical roots physically exist prior to reclamation, forming replayable chain. Recursively comparing "subtree pointer + checksum/MAC" across adjacent root generations,
**directly skipping identical pointers without descending**, with complexity proportional strictly to actual modifications between generations.
Detects error class undetectable by traversal checker: **in-place writes self-consistent after synchronous tampering**—
a write bypassing COW modifying block in-place while synchronously updating checksum appears self-consistent in isolation (I-2.1 (Checksum Matches Content) all green),
whereas root chain diff exposes contradiction: **lacking COW, no new allocation should occur; parent pointer should remain unchanged, yet checksum altered**.
⚠️ **Must pair with negative control**: running same check post-normal COW commit must be all green, avoiding flagging normal updates as tampering.

**2. Generational incremental scrub.** Leveraging I-1.2 (Block Header Write Order Published) and D5 (Snapshot / Space Accounting Mechanism) birth txg
to maintain "generation watermark scanned in previous scrub", traversing only blocks with `birth generation > watermark`—
checksum chains of older subtrees remain self-consistent if unaltered. **Enables "verifying current change completely on every commit" within wall-clock constraints without sampling.**

**Scope**: Re-located replicas retain original `birth`, permanently falling below watermark, invisible to generational incremental scrub. Neither root chain diff (`grep -rn 根链 crates` yields 0 hits) nor generational scrub (`grep -rn scrub crates` yields 0 hits) exists in `crates/`.

**Basis**:
- No experiment: Both rely on structural properties of COW new root per transaction and birth txg; neither implemented, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.

**Owed**: C389 (Merkle root chain diff has zero code); C194 (Generational incremental scrub misses relocated replicas); C408 (Generational incremental scrub has zero code).

#### Settled Item 9: Crash-Point Replay Must Be Tiered to Avoid Wall-Clock Explosion

**Settlement**:

| Tier | Scope | Trigger |
|---|---|---|
| Smoke tier | Fixed seed, fixed script workloads and registered crash enumeration cases, unconstrained by write request count (scope below), exact full run | Checker tier full run (settled item 15): run upon user request; commit runs quick tier by default |
| Regular tier | Moderate workloads (thousands of write requests), exact full run, incomplete enumeration handled by exact pruning, unrunnable nodes marked ignore, coverage report and gate listing "not run this time" entry by entry | Every commit |
| Heavy tier | Regular tier workloads + large workloads + multi-device cross matrix, N≥5 rounds, exact full run | User request / pre-release |

**What can be claimed after pruning plus ignore**: can claim "enabled nodes passed across all crash states";
**cannot claim** "ignored nodes passed".

**Full run is not run by default at commit, unconstrained by time, run upon user request; pre-commit = quick tier all green (necessary condition).**
**At milestone closeouts or batches modifying crash-point replay or judges, main agent displays popup asking user to run full run; commits permitted on failure or skip, logging owed check for red or "milestone closeout skipped full run" until resolved.**
"Full run" refers to two items: checker-tier-crash-point-replay `--full` (gate checker-tier-crash-point-replay, settled item 15 full run) and crash amplification full run (`scripts/crash/crash-amplification.sh` without arguments).
"Modifying crash-point replay or judges" calculated dynamically from crash verdict library: triggered if changes cause mismatch in library verdict block unit keys, with mismatched keys containing judge units (function units in closure of judge entry fn); gate reports invalidated block count and changed judge unit list, main agent prompting popup accordingly, avoiding path-based lists.
"Previous full run" recorded by main agent in milestone log: date, conclusions of both runs.
**Full run tier must prioritize "failure reproducibility" over "maximal coverage"**—fixing a full run bug takes wall-clock ≈ `log2(N) × single full run duration`; an irreproducible full run failure has near-zero or negative value, training developers to ignore red flags.

**Scope**: Only smoke tier exists (gate checker-tier-crash-point-replay); smoke tier divided into two modes: regular quick tier takes arbitrary subset for in-place writes and all-or-nothing for COW writes (settled 2026-09-26, designated "variant 2", used after fixing C561 (Record Checker Reuse Exemption False Red on Partial COW)), unable to detect bugs manifesting only under partial COW landing, incapable of replacing full run; full run with resumable checkpoints (gate checker-tier-crash-point-replay `--full`), registered crash enumeration cases in `.claude/gate.d/stage-inputs.tsv` growing to 12,230,590,578 states on parallel line 1 stream, far exceeding smoke tier scale of "two-digit write requests", hence commit runs quick tier by default, full run executed upon user request per settled item 15; regular tier and heavy tier lack gate stages. Defining crash states belongs to settled item 4; exact pruning and enabling ignored nodes belong to settled items 10, 11.

**Basis**:
- No experiment: Tiering is design constraint of gate wall-clock; workloads of regular tier and heavy tier in the gate are not yet settled; E184 (Cost Counts of Regular Layer and Large Workload Layer) segment 1 measured scale and state-count interval overlap of candidate workloads, preparation only; segments 2–4 (invalidation surface, cold throughput, fixed cost of the two landing ways) unrun.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.
- Regular quick tier uses variant 2: user decision, original wording in history; crash replay scale review `research/prompts/m2-layer0-scale-r3-main-verification.md`.
- User decision: "we do not sample, we prune plus initial ignore" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); "enumeration can set ignore flag and open slowly" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); "1. design complete enumeration, 2. incomplete enumeration is purpose of pruning" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); Round 1 verdict `research/prompts/m3-prune-gpu-r1-main-verification.md`: settled item 9 overturned.
- User decision (2026-10-01): Full run unconstrained by time, runnable anytime upon user request, removing "nightly", original wording in history.
- User decision (2026-10-01): Timing for full run all-green is milestone closeout and pre-commit of batches modifying crash replay or judges, prompted via main agent popup, original wording in history; delegated to user per three-party verdict `research/prompts/test-script-align-r1-main-verification.md` Section 2 t1-3.
- Three-party verdict `research/prompts/test-script-align-r2-main-verification.md` Section 2 U3: two meanings of "full run", path judgment for "modifying crash replay or judges", logging "previous full run", synthesizing "admission condition" and "async" into single rule.

**Owed**: C409 (Regular tier and heavy tier lack stages in gate); C635 (Previous full run lacks machine-readable marker).

#### Settled Item 10: Truly Required Exact Coverage N Is "Number of Commit Protocol Equivalence Classes", Not "Layout Schemes"

**Settlement**: Regular tier exact coverage reduces the problem before addressing pruning. Domain of crash points is the **transaction layer state machine**, governed by two top-level rules in `.claude/rules/fs-design.md`:
"One transaction layer, shared by all structures" and "The only layer forbidden from diversifying is the transaction layer". Thus layout can alter only two categories:

| Difference | What It Changes | Judgment |
|---|---|---|
| Node size, stripe width | Bytes written per step, repetition of step | **Parameter**, introduces no new step kinds |
| Rotational vs SSD | Seek cost, IO scheduling | Resides at device layer, **outside transaction layer protocol** |
| **Zoned (ZNS)** | **Ability to overwrite root pointer in-place** | **Structural**: append-only, drifting root, introduces zone finish/reset |

**Implemented via closed enum + exhaustive match**: `CommitStep` in `crates/singlefs-core/src/transaction.rs`, step kinds registered in segment sequence registry of [layout/01-first-txn.md](../layout/01-first-txn.md) Section 8.
Any layout introducing new structural steps **must modify this shared definition ⇒ compiler enforces branches at all call sites ⇒
diff is grepped by gate**, automatically tagging "this commit introduced new structural equivalence class, requiring independent heavy tier replay",
**without relying on human memory** (diff-based gate scope owed under C8 (Gate Scope Indeterminable)). Adding newtype isolation (`RotationalPtr` / `ZonedPtr`),
turning "forbidden sharing" bucket in D12 (Target Media) settled item 2 (accounting fact judgment paths) into compile-time constraint.
⚠️ **Second duty (added by D17 (Implementation Layering and Third-Party Pipelines)): serves as integration point for third-party pipelines.**
Protocol owned by this project; external pipeline introducing new structural commit steps must submit diff to this repository.

**Scope**: Rotational and SSD are isomorphic at commit protocol level, established by D12 (Target Media) three-party review; zoned line indicates it introduces at least one additional equivalence class. Equivalence relation isomorphic to recording stream segment sequence, count belonging to D17 (Implementation Layering and Third-Party Pipelines) settled item 2. Whether kind string identifies nodes adequately hit in Round 1 verdict (suspended, awaiting Rounds 2, 3): `research/prompts/m3-prune-gpu-r1-main-verification.md`.
⚠️ **Pins "which steps exist", but cannot pin "failure atomicity width of each step"**—
persistent memory and object storage alter failure atomicity width of each step, unhandled by enum (gap and remediation in D17 (Implementation Layering and Third-Party Pipelines)).
`RotationalPtr` / `ZonedPtr` newtype isolation absent in `crates/` (`grep -rn "RotationalPtr\|ZonedPtr" crates` yields 0 hits).

**Basis**:
- No experiment: Identity of N derived from `.claude/rules/fs-design.md` top-level rules, lacking measurable quantities; criteria for equivalence relation and counts in D17 (Implementation Layering and Third-Party Pipelines) settled item 2.
- Three-party review: Regular tier sampling round ("three-party review + empirical verification"), background in history.

**Owed**: C8 (Gate scope indeterminable).

#### Settled Item 11: When Ignored Nodes Are Enabled: Rejecting Ledger Rotating by Owed Checks, Retaining Three Rules

**Settlement**:

**"Ledger rotating by owed checks" scheduling: Rejected.** Scheduling based on cells = `(layout, substage_bucket)`, taking head of queue in descending order of owed checks, claiming any cell covered within `K_max = ⌈N×M/C⌉` commits is rejected: fails to guarantee precisely the slow, rare branches it aims to protect (empirical calculations and four rejection reasons in history).

**Three retained rules**:

1. **Correct direction**: Requires **deterministic coverage records capable of turning gates red**, rather than probabilistic arguments—
   probabilistic guarantees equal no guarantee in long-term maintenance.

<!-- doc-lint:registry name-col=2 -->

| Symbol | Short Name | Meaning |
|---|---|---|
| T1 | Regular tier wall-clock budget | Local wall-clock budget for regular tier, an input rather than output |

2. **`T1` (Regular Tier Wall-Clock Budget) is input, not output** (`.claude/rules/fs-design.md`: gate runtime is itself a design constraint).
   Correct sequence is "declare budget first → constrain how many ignored nodes single commit can enable", not "measure runtime → infer how many can open".
   **This rule requires no measurement, settled immediately.**
3. **Discriminative power must be checked separately from coverage**: Commit enabling an ignored node must run its checker against a "known corrupted image",
   flagging red if no error reported—direct reuse of existing positive control discipline in D13 (Verification Route). See C13 (Checker Judgment Ineffective).

Gate lacks "regular tier crash-point replay" stage, unable to measure `T1` (Regular Tier Wall-Clock Budget) / `C` / per-cell runtime.
⚠️ **Therefore `C`, `K_max`, `T1` (Regular Tier Wall-Clock Budget) must not be written into kb at this time.**

**Scope**: Governs when ignored nodes are opened; smoke tier full run enumeration does not require `T1` (Regular Tier Wall-Clock Budget). Rule 2 "settled immediately" conflicts with ending sentence "must not be written into kb", recorded in [verification-build.md](../verification-build.md) "Pending Decisions" question 6, unadjudicated. Node registry carries status field (open / ignore / awaiting implementation), plus three rules—each ignore row names mutating object, turning red if object turns green while node remains ignored, coverage report lists ignored state count and ratio: attacked 0 rounds (`research/prompts/m3-prune-gpu-r1-main-verification.md`), unwritten as settlement pending Rounds 2, 3.

**Basis**:
- No experiment: Rejecting ledger based on simulation of scheduling rules (200 rounds, steady-state maximum of second half, strict prefix greedy, numbers in history), unnumbered as experiment; retained three rules are gate design principles, lacking measurable quantities.
- Three-party review: Regular tier sampling round ("three-party review + empirical verification"), background in history.
- User decision: "enumeration can set ignore flag and open slowly" (`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` Section 3); Round 1 verdict `research/prompts/m3-prune-gpu-r1-main-verification.md` overturned settled item 11, providing status field form and three supplementary conditions.

**Owed**: C8 (Gate scope indeterminable); C13 (Checker judgment ineffective).

#### Settled Item 12: Conflict 1: `Checker as Specification` vs `Verification Paths Sharing No Tools`

**Settlement**: **Conflict 1: Direct contradiction between `checker as specification` and `verification paths sharing no tools`.**
If active invariants (authoritative count in [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) header) serve as both checker source and formal verification spec, they become single point of failure.
**Viable compromise, paid for in this project**: Checker implements two directions—**traversal** (descending from system configuration)
and **scan** (full disk scan, reconstructing from block headers under "metadata blocks must be self-describing"), both judging the same invariant independently,
sharing only format parsing.

**Scope**: Governs sharing between the two internal directions of checker; sharing between checker and implementation governed by settled item 5. Pool-level checker implements both directions (`crates/singlefs-checker/src/walk.rs` header).

**Basis**:
- No experiment: Trade-off between "checker as spec" and verification path independence, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.

**Owed**: C410 (Checker two directions sharing lacks check): rule that two directions "share only format parsing" lacks checks turning red (no stage in `.claude/gate.d/`).

#### Settled Item 13: Conflict 2: Refinement Spec vs Differential Model

**Settlement**: **Conflict 2: Refinement spec and differential model are two expressions of the same entity.**
Merging into one → halves effort, but **directly violates independence requirements**; writing separately → nominal independence,
yet typically authored by same person in same week. Mitigation (reduces correlation without eliminating): ensure two versions **differ at representation level**—
spec phrased as "state predicates over images", model phrased as "op → state transition", facilitating mutual localization upon divergence.
**Common-origin must be observed**: Track counts of "spec and model diverge" events and which side was judged in error;
accumulating ≥ 5 divergences with 0 "spec error" verdicts indicates effective common origin, requiring merging paths and transferring budget to exhaustive testing.
Verdict has three categories: spec error / model error / clause missing sentence (each following valid interpretation where clause left ambiguity). Inconsistency re-evaluated within valid premise domain, recorded only if persisting; "clause missing sentence" recorded separately, excluded from "accumulating ≥ 5" and "spec error", re-evaluated and assigned to first two categories after clause updated.
Model here refers to O1 (In-Memory Model Differential Testing (HashMap Ideal File System)). Until O3 (Independent Spec Executor) delivers verdicts for same (history, crash point) (first version Python executor E170 (First Version of O3 Spec Executor) exists, unconnected to crash replay, C447 (Spec Executor Not Connected to Crash-Point Replay to Compare Against RefFS)), pair O3 (Independent Spec Executor) × RefFS registered as blank, logging zero events; O1 (In-Memory Model Differential Testing (HashMap Ideal File System)) crash judge does not substitute; unestablished pairs report "unestablished", never reporting green or entering counts.

**Scope**: Mitigation reduces correlation without eliminating it. Former candidate withdrawn in settled item 1 unaffected, reasons in settled item 1 scope. Crash refinement code consists of Verus core and spec in `crates/singlefs-replay-prefix/src/lib.rs` (prefix extraction, root selection, ring scan, and joint corollary, covering settled item 1 M1, M2 syntactic layer, M3 readings and monotonicity; M2 semantic layer and M4 judged within bound by E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound)); spec cannot evaluate concrete states, provenance of divergence events undefined, counter lacks code (C191 (Common-Origin Counter Covers Only Single Pair)).

**Basis**:
- No experiment: Common origin observed via future divergence counts; spec and counter absent, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.
- Third category "clause missing sentence": three-party verdicts `research/prompts/m3-formal-r1-main-verification.md` Section 2 "F2 common origin", `research/prompts/m3-formal-r2-main-verification.md` Section 2 line 9, `research/prompts/m3-formal-r3-main-verification.md` Section 2 common-origin counter line; no experiment: counter lacks event source.
- "Model refers to O1 (In-Memory Model Differential Testing (HashMap Ideal File System))" and unestablished pairs registered as blank: three-party verdict `research/prompts/m3-formal-r5-main-verification.md` Section 2 common-origin cell; `research/prompts/m3-formal-r4-main-verification.md` user decision.

**Owed**: C191 (Common-origin counter covers only single pair).

#### Settled Item 14: Interlocks with Other Decisions

**Settlement**:

| Decision | Interlock Point |
|---|---|
| D4 (Checksum Placement) | Root chain diff flips Merkle from "necessary overhead" to verification lever; in formal verification hashes are uninterpreted functions, **collision resistance completely outside verification scope, must not be read as "verifying Merkle"** |
| D8 (Core Index Structure) | Log-structured nodes, write buffer, key cache burden symbolic execution (loop bounds, lag windows, cache coherence), rendering "Yggdrasil within one minute" measurement basis invalid in this project |
| D9 (Encryption) | AEAD intractable in SMT, modeled as uninterpreted function; **consequence "nonce reuse leaks plaintext" unprovable**, leaving pure combinatorial property "nonces never repeat"—which is precisely the most valuable output of this leg |
| D11 (Whether to Reserve Message Buffer in Index Nodes) | Settled item 1 selects "reserve"; once D8 (Core Index Structure) settled item 1 adopted idempotent full values, checker I-3.1 (Allocated Statistics Match) need not parse message semantics (D11 (Whether to Reserve Message Buffer in Index Nodes) judging Cost 1 invalid in this project), preventing Conflict 1 from becoming intractable |

**Scope**: Interlocks define boundaries of formal verification and checker when touching these decisions, leaving clauses of those decisions unaltered; root chain diff and Conflict 1 belong to settled items 8, 12.

**Basis**:
- No experiment: Interlocks are boundaries deduced from settled clauses of those decisions, lacking measurable quantities.
- Three-party review: New round establishing D13 (Verification Route) (local LLM / Sonnet / Opus, non-overlapping stances), background in history.

**Owed**: None.

#### Settled Item 15: Verification Code in Two Tiers —— Harness Tier Runs Anytime, Checker Tier Defaults to Commit

**Settlement**: Verification code under `crates/` divided into two tiers by "when it runs", each inhabiting a package, comprising three names alongside pool-level checker: **Harness tier** is `crates/singlefs-harness`, housing unit and integration tests plus scaffolding libraries (recorder, ramdisk, ideal model, random history, fault injection device, scenarios), run anytime code changes; internally split into light and heavy: debug runs taking ≥ 60s marked heavy with `#[ignore]`, measured during routine runs rather than isolated benchmarks, runnable anytime, uniformly wrapped in memory; one case per scenario, test files named by target rather than milestone. **Checker tier** is `crates/singlefs-checker-tier`, housing crash state enumeration engine, record checker, crash enumeration cases via verdict library across two phases and counter, discovery logs and toolchain identity, long-term verdict library (block keys determined by content, test types indexing needed blocks) and two-phase protocol, crash injection and bad-disk campaigns, real device log comparisons, experiment and real-device binaries, all corresponding cases, along with QEMU (gate checker-tier-qemu-device-streams), herd7 (checker-tier-lkmm), crates mutation table (checker-tier-crates-mutation-replay), and full experiment replays (checker-tier-research-build-and-replay experiment-replay cell): runs by default at commit only, runnable standalone (command carrying `SINGLEFS_HEAVY_TESTS=user-request`). Crash-point replay within checker tier (gate checker-tier-crash-point-replay) divided into two modes (others undivided): **Quick tier** consists of two passes: `cargo test --release -p singlefs-checker-tier --lib --tests` without `--ignored` (`--tests` including all `test = true` targets, including harness inline unit tests), plus pass with features touched by `cfg` in `tests/` (`--features <features> --lib --test <feature-gated-test>…`, features with `gpu` run via `scripts/crash/gpu-quiesce-run.sh`) (gate checker-tier-crash-point-replay runs both passes during full gate and commit, second pass being feature-gated-tests cell); **Full run** executes registered cases in `.claude/gate.d/stage-inputs.tsv` under key `crash-case:` entry by entry via `--include-ignored --exact` (checker-tier-crash-point-replay `--full`), skipped by default at commit, run upon user request per settled item 9; crash amplification judges migrated entirely to GPU, covering all pool-level checker invariants (`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`). **Pool-level checker** is `crates/singlefs-checker`, the verdict library called by both tiers. Dependencies permitted in single direction only: checker tier depends on harness tier and pool-level checker, harness tier never depends on checker tier (gate checker-independence-and-sync cell checker-implementation-disjoint item ④, dev-dependencies included); shared snippets duplicated separately. Crash enumeration cases written strictly in `crates/singlefs-checker-tier/tests/` (judged by `scripts/crash/crash-case-check.py`); heavy test gate identified by package: tests reaching `singlefs-checker-tier` belong to checker tier.

**Scope**: Governs package placement, execution timing, gate recognition, and naming of code and tests under `crates/` (`.claude/rules/verification.md` "Definitions and Names"); does not govern what each invariant judges ([invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)), enumeration domain definition (settled item 4), or sharing between pool-level checker and implementation (settled item 5: pool-level checker depends solely on `singlefs-format`). Upstream SOP no longer documents project verification methods: crash-point replay, model differential, checker-as-spec, and filesystem-specific deduction gaps consolidated into repository (`.claude/rules/verification.md`, `.claude/gate-not-implemented.tsv`), synced to replica without release. Tier splitting recorded in `records/2026-09-27-验证两档拆分.md`.

**Basis**:
- No experiment: Execution timing and package placement are process policies, lacking measurable quantities; discriminative power demonstrated by test cases and mutation tables.
- User decision (2026-09-27), original wording in history; three-party review `verification-split-r1` (`research/prompts/verification-split-r1-main-verification.md`) reviewed version prior to extracting `singlefs-checker-tier`, subsequent changes unattacked.
- User decision (2026-10-01): Full run unconstrained by time, runnable anytime upon user request, removing "nightly", original wording in history.

**Owed**: None.

#### Settled Item 16: Segments Exceeding u64 Judged in Slices, State Count Uses 1280-Bit Fixed-Width Integer

**Settlement**:
- Slicing: Segments registered by segment in crash amplification (`.claude/gate.d/crash-point-replay-segments.tsv`, five columns per row: item name, flow name, segment index, segment write count, segment write table digest) whose full domain state count exceeds u64 are judged sequentially in fixed-size slices; slice size determined by environment variable `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES`, defaulting to 2^32 states; slice start is resumption point in verdict library (indexed by write table digest and judge unit key; item name, flow name, segment index for display only). Segments fitting in u64 judged unsliced in single pass. Code locations: `crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`, `crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`.
- Bit width: Total state count, slice start, slice end, and progress for registered segments uniformly use 1280-bit (10 × u128) fixed-width unsigned integer `StateCount1280Bits` (`crates/singlefs-checker-tier/src/crash.rs:1640`, limbs defined by `STATE_COUNT_LIMBS` in same file), arithmetic fully checked, non-saturating, non-truncating; driver lines `registered_segment_progress=` report judged states and total states in exact decimal, never powers.
- Passing registry to engine: Engine reads segment registry strictly via environment variable `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` (`crates/singlefs-checker-tier/src/crash_amplification.rs:14764` in `fn registered_segments_of_the_flow_from_the_environment`), treating unconfigured as zero segments registered without fallback. Stream extending 144-unit file to 145 units growing lower extent segment to two levels (`position_addressed_ExtentLowerSegmentGrowsToTwoLevels`) registers both unit write segment (322 unit writes) and record segment (290 in-place journal writes).

**Scope**: Governs slicing, integer representations, and registry reading in crash amplification engine; does not govern which segments should be registered (approved by user, recorded in registry header), how segment domain expands (verdict J10 of `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md`), or how individual crash states are defined (settled item 4). Unregistered segments and whole-flow state sums must still fit in u64, rejecting execution before library write if exceeding; registered segments exceeding 1280 bits likewise rejected before library write. Slice size restricted to range 1 to 2^32: two GPU kernels add intra-slice offset as u32 to slice start (`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`); light tier callers (`scripts/crash/crash-point-replay.sh --quick`) default slice size to `LIGHT_REGISTERED_SEGMENT_SLICE_STATES`. Segments exceeding expansion limit without registration fail amplification entry `scripts/crash/crash-amplification.sh`; crash-point replay, amplification, and item drivers verify table before passing to engine.

**Basis**:
- E161 (Deduplication and Segmentation Elapsed Time in Crash Amplification) "Rerun 2: Sampling Domain Reset to 11-Segment Stream (2026-10-05)": Full state counts match closed form, harness values, and crates plan segment by segment, growing exponentially with write counts, supporting premise that "long segment full domain exceeds u64"; slice size, bit width, and registry reading are engineering parameters settled by user, unsupported by E161.
- User decision, original wording in history: slicing and 2^32 states per slice (`records/2026-10-11-崩溃放量两段与脚本审计.md` line 96), 1280-bit width (lines 98, 99), registering both segments (line 92).
- Slicing and width supplement verdict J10 of `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md`, unattacked by three-party review.

**Owed**: None.

#### Settled Item 17: Unit of Verdict Reuse Is Function, Key Composed Solely of Function Units, Toolchain, Dependency Versions

**Settlement**:
- Verdict reuse across heavy gates, crash verdict library, and experiment admission (crash state blocks, quick tier and model differential test functions, crates mutation lines, broad mutations, formal verification, QEMU tiers, litmus suites, research reruns, experiment mutation tables, experiment admission) depends strictly on its unit key: de-commented fingerprints of dependent function units, plus `#[cfg]`, `#[cfg_attr]`, and `cfg!` atoms genuinely referenced within closures (including mod headers) paired with boolean values under current target (unreferenced atoms excluded: builds with/without `gpu` feature share identical key and library if `gpu` unreferenced in closure), toolchain identity (including cargo build environment and external tool versions), dependency versions resolved from `Cargo.lock` genuinely referenced by closure, and content inputs of the verdict itself (segment write tables, base image digests, litmus content, mutation originals and replacements). Function units collected from entry point along call and reference paths, including referenced constants, types, statics, and macro expansions; WGSL segmented by fn.
- Strictly excluded from keys and reuse criteria: diff ranges computed by commit, HEAD, or tree; entire file text; file-level, crate-level, or table-level fingerprints/closures; manual version constants; paths (including repository root), names, seed strings, line numbers, resource caps, timeouts; whole-stage or whole-cell flags lacking per-verdict libraries. Keys too narrow (missing referenced constants, types, macros), placing libraries in ephemeral paths, losing libraries on rename, or duplicate judging across two libraries constitute violations.
- Incomplete collections (dynamic dispatch, macros, unparsed `include_str!(concat!…)`) reject key computation and run normally, never silently omitting dependencies; each key persists alongside its component unit list, reporting which units changed upon invalidation.
- One library per verdict class, fixed in persistent directory, never changing directory across runs or evaluation methods; repository root is not input: crash verdict library directory accepts absolute paths only via `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`, rejecting relative paths; full run switch writes without reading, updating newly judged results into same library; second host results merged into local copy.
- Invalidating old libraries upon key changes incurs no migration cost: no compatibility migration or backfilled signatures, rerunning after changes.

**Scope**: Governs reuse keys for heavy gates, crash verdict library, and experiment admission; does not govern diff ranges for static gates checking added lines only (`scripts/gate/changed-paths.sh`) or preflight `admission: inputs-changed`; does not govern enumeration domain (settled item 4), tiering, or user prompts for full runs (settled item 9). Image verdicts and given-state verdicts keyed by verdict definition group (closures of CPU reference judges `crash::judge_crash_image`, `crash_given_states::judge_the_given_states_on_the_cpu`), regardless of CPU or GPU evaluation: GPU judge is accelerated path, admitted via differential testing against definition group (quick tier GPU vs CPU differential cases); its units enter block attributes as provenance rather than keys, undivided by kind; GPU card drivers, card models, and multi-card partitioning remain block attributes or run parameters, excluded from keys.

**Basis**:
- User decision (2026-10-15), original wording in history and `records/2026-10-13-复用键与全仓改名.md` Section "2026-10-15 User Settles: Commits Unrelated to Reruns, Reruns Follow Code Changes Only", "2026-10-15 Batch 3".
- User decision (2026-10-17): Verdicts unbound to hardware, CPU vs GPU; reuse and pruning ignore devices; original wording in history and `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 halting full run.
- Main agent decision (2026-10-15): Repository root excluded from keys and libraries, library path accepts absolute only, cfg components include genuinely referenced atoms only, recorded in same document decisions 1, 2.
- Counter-evidence: E164 (Small-Scope Functional Testing of New Crash Amplification Design) F4 demonstrated fingerprinting functions alone without referenced constants missed V5 and V11 (V11 modifying constant `DATA_POINTER_BYTES` only): function units must include referenced constants, types, statics, and macros.
- On-disk measurements: `ledger.md` Section 1.2—blocks differing only by name stored and judged separately across 4100 and 9679 blocks; altering single whole-library method digest rejudged 3,935,242,934 states, where 24 of 29 changed files in closure were verbatim identical after de-commenting: keys with names or file-level fingerprints trigger spurious re-judgments.
- Unattacked by three-party review; user settled opening three review rounds post-code changes, attacking strictly "whether pruning is atomic, whether keys comprise function units only".

**Owed**: C8 (Gate scope indeterminable), C489 (Broad mutation of singlefs-core lacks conclusion for current code), C528 (Two gate paths judge different change windows), C564 (Remaining 6 crash-point replay full runs unrun), C605 (Crash amplification reuse and pruning lack disabled control), C630 (Commit-time formal verification step 1 halt frequency unmeasured), C635 (Previous full run lacks machine-readable marker), C645 (Crash amplification full run lacks executor), C777 (Five experiment keys lack registered lines in stage-inputs.tsv), C836 (Tree attached to staged-green not necessarily judged tree).

```

**出处 `.claude/kb/decisions/13-verification-route.en.md:447-450`（整段抄，未转述）**

```markdown
## Historical Versions

Historical items for D13 (Verification Route) are consolidated in [decisions-history.md](../decisions-history.md).

```

**出处 `.claude/kb/decisions/13-検証ルート.ja.md:1-6`（整段抄，未转述）**

```markdown
## D13 検証ルート —— 確定

[中文](13-验证路线.md) · [English](13-verification-route.en.md) · **日本語**

D13 (検証ルート) は本プロジェクトが正当性を何によって検証するかを司る：3種の既存手法をすべて実施し —— 独自参照実装の作成による差分突き合わせ、有界全数探索クラッシュテスト、crash refinement 形式検証（調査は [prior-art.md](../prior-art.md) 7.4 参照） —— その上で単なる模倣に留まらず、本プロジェクト固有の4点（トランザクションごとの新ルート発行を伴う COW、Merkle 自己検証、ボリューム全体の AEAD における nonce の一意性、参照実装の欠如）に応じた独自の検証手段を設計する。oracle をどう分割するか、独自開発する2点は何か、クラッシュポイント再生をどのように階層化およびサンプリングするか、クラッシュ状態集合をどう定義するか、checker と実装が何を共有するかを決定する。管轄外：各不変条件の規定内容は [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) に委ねる；3種の検証手段の具現化手順と依存関係は [verification-build.md](../verification-build.md) に委ねる；コミット手順の登録場所、構造同値類の分割方針とクラス数は D17 (実装階層とサードパーティパイプライン) 確定項目 2 に帰属する。

```

**出处 `.claude/kb/decisions/13-検証ルート.ja.md:7-447`（整段抄，未转述）**

```markdown
### 確定項目

| # | 項目 | 確定事項 |
|---|---|---|
| 1 | **crash refinement はどの部分を先に検証するか** | まず「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」という閉じたユニットを検証する。第1マイルストーンは任意の `jsn` 連続プレフィックスがパブリッシュを分断しないこと；マウント透明性は第4の独立した性質として確立；旧候補 I-6.5 (nonce 水位の単調性) は撤回 **状態：確定。** **進捗：未実装。** |
| 2 | **トランザクション層が直列コミットを公約するか** | まず直列とし、交差点を越えた後に並行を開く；将来の並行コミット時はレコードをトランザクション番号順に追加する **状態：確定。** **進捗：実装済み。** |
| 3 | **独自参照実装（RefFS 形態）をどの程度まで突き詰めるか** | セマンティクスレベルを極限まで突き詰める：メモリ内でのみ POSIX 操作に従って動作し、永続化およびクラッシュ後に許容される状態を独自に定義し、ディスク上フォーマットは持たない；kb に基づいてゼロから独立パッケージとして作成し、ホワイトリストの入力のみを受理する **状態：確定。** **進捗：実装済み。** |
| 4 | **クラッシュポイント再生のクラッシュ状態集合をどう定義するか** | 書き込みストリームをデバイス単位でセグメントに分割し、セグメント内で書き込みのあった各ディスクが自らのバリアまたは FUA で解放されて初めてセグメントを閉じる；セグメント内の各完全書き込みは状態を任意に組み合わせ、インプレース上書きには「新旧いずれも読み出せない」という第3の状態を追加；未永続化部分には旧バイトを配置し実際にイメージを生成する；ミラー書き込みはデバイスごとに個別に計算する **状態：確定。** **進捗：実装済み。** |
| 5 | **checker とテスト対象実装の間で共有を許容するもの** | kb から生成された定数モジュール1つのみを共有し、それ以外は各々独自に実装する；判定幅は checker が独自に検出しシステム構成と照合し、検出不能時は「宣言値、未検出」と報告する **状態：確定。** **進捗：実装済み。** |
| 6 | **互いに独立な3つの oracle** | インメモリモデル突き合わせ、独立パーサ + checker、独立規約実行器の3つの oracle が各々異なる側面を判定する；独立性は「3 oracle 結託テスト」によって検証し、自己申告を禁ずる **状態：確定。** **進捗：未実装。** |
| 7 | **O2 (独立パーサ + checker) は何を判定するか** | O2 (独立パーサ + checker) の定義域は単一イメージであり、集合の要素性のみを判定する；第2の入力を必要とする検査はレコード照合器（入力パラメータに列挙器が提供する永続化集合を含む）に帰属させ、意図的に番号を付与しない **状態：確定。** **進捗：実装済み。** |
| 8 | **独自開発の2点** | Merkle ルートチェーン差分（隣接する2世代のルートを再帰的に比較し、ポインタ不変箇所には掘り下げない）および世代間増分 scrub（birth generation が水位より高いブロックのみ走査） **状態：確定。** **進捗：未実装。** |
| 9 | **クラッシュポイント再生をどう階層化するか** | 3層とも正確なフル実行とし、列挙しきれない場合は正確な枝刈りに依拠し、直ちに実行できないノードには ignore を付与して順次開放する；マイルストーン完了および判定器ユニット変更を伴うコミット前にはポップアップでユーザーにフル実行層の実行を促し、失敗または未実行でもそのままコミット可能とし未済として記録；まず失敗再現性を確保する **状態：確定。** **進捗：実装済み。** |
| 10 | **真に正確に網羅すべき N は何か** | コミットプロトコル構造の同値類数であり、レイアウトのセット数ではない；具現化手段は閉じたコミット手順列挙と網羅的な match である **状態：確定。** **進捗：実装済み。** |
| 11 | **ignore ノードをいつ開放するか** | 未済の検査のローテーション台帳を否決；開放の契機を規定する3条を残す：決定論的なカバレッジ記録、`T1` (通常層 実時間予算) を入力とする、判別力とカバレッジを分離して検査する **状態：確定。** **進捗：対象なし。** |
| 12 | **衝突 1：checker 即仕様 と 検証の2経路間でツール共有を禁止する** | checker はトラバーサルとスキャンの2方向を実装し、同一の不変条件に対して各々独立に判定し、共有するのはフォーマット解析のみとする **状態：確定。** **進捗：実装済み。** |
| 13 | **衝突 2：refinement の spec と突き合わせの model** | spec と model は表現形式のレベルから異なって記述する；同源性を観測し、不一致が5回以上かつ spec の誤りが0回の場合は統合して予算を全数探索へ移管する **状態：確定。** **進捗：対象なし。** |
| 14 | **他の決定との連鎖** | ハッシュと AEAD は形式検証において uninterpreted function としてのみ扱える；ログ構造の3層がシンボリック実行を拘束する；checker はメッセージセマンティクスを解析する必要はない **状態：確定。** **進捗：対象なし。** |
| 15 | **検証コードを2層に分割** | harness 層（`singlefs-harness`）は単体および結合テストであり常時実行；checker 層（`singlefs-checker-tier`）は既定でコミット時にクイック層のみ実行し、フル実行はユーザー要求時に実行する **状態：確定。** **進捗：実装済み。** |
| 16 | **セグメント単位で登録されたセグメントの分割方法と状態数のビット幅** | 登録セグメントの全領域が u64 に収まらない場合はスライスごとに順次判定し、スライスサイズは既定で 2^32 状態、スライス起点はライブラリ内の再開点とする；状態数は一律に 1280 ビット固定幅整数を用い、演算はすべて checked とする；登録表は環境変数経由でのみ読み込む **状態：確定。** **進捗：未実装。** |
| 17 | **判定再利用のユニットは何か** | 枝刈りのユニットは関数である：ある判定を踏襲するか否かは、それが依存する関数ユニットの各コメント除去後指紋、ツールチェーン、解決された依存バージョンのみを参照する；コミットやツリーによる変更計算、ファイル全体の原文、ファイル単位やパッケージ単位の指紋、手書きバージョン番号、パス名のキーへの導入、工程全体のマークは一律に禁止する **状態：確定。** **進捗：未実装。** |

#### 確定項目 1：crash refinement はまず「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」を検証する

**確定事項**：crash refinement の最初の検証対象はリカバリ関数 `Recover` の関係的性質であり、最小の閉じたユニットは「1回のパブリッシュ + 1回のクラッシュ + 1回のリカバリ」である。4つのマイルストーンの主語 `Recover` は、`crates/singlefs-core` 内で `verify_named_units = true` を指定して `replay_journal` を呼び出している3箇所（`recover` の `Consult` 戦略、書き込み可能マウントの読み出しフェーズ、読み取り専用マウント）を指し、3箇所の `records` はすべて `scan_journal` の出力を取得する；`Ignore` および `ConsultWithoutNamedVerification` はテストおよび対照用のアームであり、トラステッドコンピューティングベースに配置する；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` はプロダクションの呼び出し元ではない。4つのマイルストーンは順に以下の通りである：

| マイルストーン | 義務 | 配置先 |
|---|---|---|
| **M1 全域性** | `Recover` が D13 (検証ルート) 確定項目 4 で列挙された**すべての**クラッシュ状態において定義され、停止し、I-7.1 (ルートスロット有効集合の非空性) および I-8.3 (再生プレフィックスの厳密な連続性) を満たすイメージを出力すること | スキャフォールド。**完了の判定基準は「証明されたこと」ではなく「M2 をその上に記述できること」である** —— クラッシュポイント再生は元よりフル実行で列挙しており（[verification-build.md](../verification-build.md)「クラッシュポイント再生はサンプリングせず、フル実行で列挙」）、リカバリのハングはその場で露見するため、全域性単独を証明することに限界価値はない。プレフィックス取得、ルート選択、リングスキャンの3区間はすでに Verus に導入され、すべて `crates/singlefs-replay-prefix/src/lib.rs` にある：プレフィックス取得コア `replay_core`（停止性と非オーバーフローはコア内で証明済み、ループはすべて `decreases` を伴う；2つの `requires` —— レコード表が (インスタンス番号, カウンタ) 昇順、カウンタに後続が存在 —— はグルーコード `replay_journal` 内のアサーションで担保）、ルート選択コア `choose_root_slot`（自己検証済みのスロットのうち (`checkpoint_txg`, インスタンス番号) 最大のものを取得、同値時はスロット順最前を取得）、リングスキャンコア `first_copy_of_each_record_key`（同一 (インスタンス番号, カウンタ) は先着順）、ルート選択とプレフィックス取得の結合判定は系 `corollary_recovery_replays_from_the_newest_self_verified_root` として記述；ゲート checker-tier-formal-verification が証明を判定し、固定数は 56 verified（`.claude/gate.d/verus-pins.tsv`）。E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) を軽量ツールとして用い、その第3回再実行第2区間のルート選択結合判定ファミリが ramdisk 上で本物の `choose_root`、`scan_journal`、`replay_journal` を呼び出す。ルート選択順序の4つの改悪変種のうち、正当な履歴 K ファミリで赤となるのは「最初の有効なものを選択」のみであり、他の3種（比較時にインスタンス番号を無視、txg 同値時に低位を選択、インスタンス番号主導）は Verus 必赤世界（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` のルート選択の節）および敵対的ルートリング KA でカバーされる（第3回再実行第3区間）。コア外：システム構成選択（`choose_system_configuration`）および2つのグルーコード（ルート選択のスロット読み出し、自己検証チェックサム、fsid、領域からディスクへのマッピング、スロットの走査順序；リングスキャンのオフセット列挙、ヒント、パースと CRC、ディスクの走査順序）は Verus の対象外であり、クラッシュポイント再生およびクラッシュ増幅の CPU 判定器が各クラッシュ状態上で `recover` を呼び出してこれらをカバーする（GPU 判定器は WGSL で `replay_journal` を別途記述しており、core は検証できない） |
| **M2 プレフィックスがパブリッシュを分断しない** | **任意の `jsn` 連続プレフィックスについて、その再生結果はある整数個のパブリッシュ済み checkpoint の結果と等しくなる —— プレフィックスによって分断されるパブリッシュは存在しない；プレフィックスがあるパブリッシュの途中で停止した場合、そのパブリッシュは全体として適用されない**（D16 (パブリッシュセマンティクス) 確定項目 4「1回のパブリッシュは全体として適用されるか全体として適用されないかのいずれかである」）。2層に分かれる。**構文層**（任意の自己検証済みレコード表に対して、前提なし）：適用されるレコードはいくつかの完全なパブリッシュであり、インフライト上限とコミットマーカーが1条ずつ判定され、適用されるカウンタはチェーン先頭から順に連続する（アンカーが読み出せる場合はアンカー + 1 から開始；読み出せない場合、先頭の `checkpoint_txg` = 選択されたルート + 1、シーケンス番号 1）；逆方向チェーンおよび指名ユニットはディスク読み出しコールバックの結果であり、「適用された各レコードについて、コールバックが真を返した」と記述される。**セマンティクス層**（前提：ディスク上の各レコードはこのバージョンの正当な書き込み側に由来する、すなわち E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の正当な履歴ファミリに属する；同一の (インスタンス番号, カウンタ) はすべてのタイムラインを通じて唯一の内容に対応する）：「結果」はルートの射影 + ディスクであり、ルート全体ではない —— 返却されるルートの (インスタンス番号, `checkpoint_txg`) および新ルートセグメントの4フィールドは、選択されたルートのタイムライン上の第 j 版（j は選択されたルートより前ではない）と等しく、その他のフィールドは選択されたルートと等しい（アンマウント記号を除く；レコードから再構築されたルートは「アンマウントシーケンスで書き込まれたものではない」とする）；ディスク上の (選択されたルート, j] の各バージョンのレコードが読み出せ、指名ユニットの両方のコピーが検証済みであり、レコード件数の合計がインフライト上限を超えない | 配置先は I-8.3 (再生プレフィックスの厳密な連続性) —— その判定補足文は逐語的に本ルールそのもの（「プレフィックスがパブリッシュの途中で停止してはならない —— 1回のパブリッシュは全体として適用されるか全体として適用されないかのいずれかである」）であり、[invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) 内に別途新設してはならない。構文層は Verus コア `replay_core` の `ensures`（`replay_follows_the_prefix_rules`）内にあり、必赤世界は `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` に登録され、そのうち「判定変更」行は実行可能判定のみを変更し spec には触れない；セマンティクス層は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の正当な履歴ファミリによって有界内で判定され、Verus では証明しない。コア外：グルーコード（テーブル展開、コールバックのラッピング、レポートと新ルートの組み立て）は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) が直接本物の `replay_journal` を呼び出すこと、harness テストケース、および `crates/mutations.tsv` のミューテーション行でカバーする。新ルート組み立て時の射影外フィールド（アンマウント記号、インスタンス表ポインタ、割り当てレコードツリーのルート）にはミューテーション行がなく、E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) のジェネレータがこれらを固定値として記述する；2つのディスク読み出しコールバック（逆方向チェーン、指名ユニットクロージャ）は、E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) 内の実コールバック、ramdisk の読み出し回数、`crates/mutations.tsv` のミューテーション行、および読み取り専用マウントの幾つかのパターンの読み出し回数テストケースでカバーする |
| **M3 冪等性と単調性** | **冪等性**：`Recover(Recover(c)) = Recover(c)`、両方の解釈を証明する —— レコードレベル（同一レコードを2回適用した結果と1回適用した結果で得られるルートが同一であること、I-8.4 (再生の冪等性) の文面）および不動点（再構築されたルートを選択されたルートとし、同一のレコード表、同一のリーダー、同一のスイッチで再度再生した際、返却されるルートが不変であること；前提として選択されたルート以上のレコードがインフライト上限を超えず、かつレコード表が I-8.12 (同一インスタンスレコードの txg 非減少) を満たす：同一インスタンス内の journal レコードの txg はカウンタに伴って減少しない）；2回目で選択されたルートの `checkpoint_txg` が上限に達して `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` を報告し、1件も適用されなかった場合は同等とみなす。「すなわち I-8.4 (再生の冪等性) である」は成立しない：I-8.4 (再生の冪等性) はレコードレベルの1解釈であり、不動点は別の独立した性質である。**単調性**（M2 の系、個別には証明しない）：直列コミット（確定項目 2）、D16 (パブリッシュセマンティクス) 確定項目 7 の永続化順序、選択されたルートがルートリング内で最新の有効なルートであるという条件下で、同一実行内の D13 (検証ルート) 確定項目 4 の故障を含まない2つのクラッシュ状態 `c ⊆ c'` について、`Recover(c)` が到達するバージョンは `Recover(c')` が到達するバージョンより遅くならず、バージョンは (`checkpoint_txg`, インスタンス番号) の順で比較され、D22 (ユニットの原子性をどう合成するか) 確定項目 7 の最新選択順序と同一順序である；並行コミットを導入する場合、またはドメインを故障を含む状態に緩和する場合は再審議する。「適用されたトランザクション集合が包含される」という解釈は撤回：理想的なリカバリに対しても偽となるためである | Verus コア `replay_core` の ensures で両方の解釈を証明：レコードレベルおよび不動点（不動点の前提条件には I-8.12 (同一インスタンスレコードの txg 非減少) を含む）；単調性は系 `corollary_the_landed_version_does_not_go_back` として記述し、ルート選択と書き込み側の2つの前提はその requires に記述する。E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の敵対的ファミリが有界内で不動点を判定し、違反はすべて I-8.12 (同一インスタンスレコードの txg 非減少) が成立しないレコード表においてのみ発生する（第3回再実行第1区間）。checker 側（I-8.4 (再生の冪等性)）は実装済みであり、クラッシュ注入の二次クラッシュ上で判定され、プールレベル checker のリストには含めない：`crates/singlefs-checker-tier/src/crash_injection.rs` の `REPLAY_IDEMPOTENCE_INVARIANT` および `judge_the_remount_after_a_second_crash`。O2 (独立パーサ + checker) は冪等性を判定できない：I-4.5 (廃止済み・2イメージ必要) の廃止理由は逐語的に「冪等性は2つのイメージを比較する必要があり、単一イメージでは判定できない」である |
| **M4 不足適用の防止** | 前提と対象は M2 のセマンティクス層と同一。`Recover` が返却するルートはまさに j\* である：選択されたルート以降の各バージョンがディスク上に完全に存在し（レコードのいずれかのコピーが自己検証済み、指名ユニットの両コピーが検証済み）、かつレコード件数の合計がインフライト上限を超えない最も遠いバージョンである。この条項を付加しない場合、M1–M3 をすべて証明したとしても「確認済みの完全なパブリッシュがリカバリによって破棄される」ことを排除できない：1件も適用しないリカバリであっても M1–M3 の文面をすべて通過してしまう。「不足適用の防止」は「追加の違反の発生」として解釈する：変種が追加の違反を生じさせて初めて、本条項によって阻止されたとみなす | E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の最遠バージョン判定基準によって有界内で判定する；Verus の必赤世界において「水位を超える最初のレコードで停止し、1件も適用しない」行が赤となる；M4 自体は単独では証明しない |

**マウント透明性**（M3 の第3の解釈ではなく、独立した第4の性質）：クラッシュイメージ上で読み取り専用リカバリを1回実行して R1 を得る；同一のイメージ上でアクションを1つ実行 —— マウントのみ（番号取得、行書き込み、ウォームアップ、マウント継続実行、およびマウント自体の押し上げによる F 引き上げ）、またはマウント + 1回のパブリッシュ —— その後、再度読み取り専用リカバリを実行して R2 を得て、R1 と R2 をフィールドごとに比較する。比較対象のバージョン：マウントのみの場合はリカバリが到達したバージョンと比較し、マウント + 1回のパブリッシュの場合はマウント後のパブリッシュの直前のバージョンと比較する。マウント途中の3フェーズにおける二次クラッシュイメージ img2 上でも判定する：C(recover(img2)) = C(v1)（v1 は R1 が到達したバージョン、C は POSIX の内容および存在性を取得）、内容とフィールドの両半面を判定する；フィールド半面において、v1 のバージョンにフォールバックした場合は各フィールドが R1 と同一であり、今回のマウントによって書き込まれたいずれかのバージョンに到達した場合はフィールド別表に従って判定し、スキャンレポートは判定せず、F 引き上げの最初の書き込み以降は F を判定せず、それ以前（ウォームアップを含む）は「マウントが F 引き上げを押し上げなかった」セルに従って判定する；F 引き上げの最初の書き込み位置は、マウントが返却した F 引き上げシーケンス（受入で押し上げられたものおよび継続実行で空間のために押し上げられたものの双方）のマウント書き込み表末尾における位置に基づいて取得し、取得不能時（`NotLocatedInTheMountWrites`）は今回のマウントの二次クラッシュ区間全体の F を未判定とする。フィールド別表（R2 対 R1）：

| フィールド | 判定法 |
|---|---|
| POSIX 内容および存在性 | 一致必須 |
| プール識別情報およびシステム構成の不変構成（自ディスクデバイス番号を除く） | 一致必須；不変階層の37行は自ディスクデバイス番号を除き一律に生バイトで比較 |
| システム構成のノードサイズ | 一致必須 |
| システム構成の実行時構成 36 バイト（T_time、T_dirty、3水位） | R2 は組み込み既定値に等しい（マウントオプションチャネルがなく、`to_slot` は常に既定値を書き込む） |
| アンマウント記号 | R2 は常に「アンマウントによる書き込みではない」であり、R1 の記号は比較対象外 |
| F および F 有効値 | マウントが F 引き上げを押し上げなかった場合：R2 の F 有効値および R2 ルート内の F はいずれも R1 の F 有効値に等しい；押し上げた場合（受入によるものと継続実行で空間のために押し上げたものの双方を含む）：R2 の F 有効値はマウントが返却した最後のシーケンスが引き上げた F に等しい |
| インスタンス番号 | max(各ディスクの2スロットの自己検証済みシステム構成のインスタンス番号, ルートリングの全可読ルートのインスタンス番号) + 1、双方の量をクラッシュイメージから読み出す |
| インスタンス表の行 | D18 (ブロックに含める情報) 確定項目 11 に従いリカバリが書き込んだ行 |
| `checkpoint_txg` | 下限のみ判定：≥ max(ルートリングの全ルートの txg, リング内の全自己検証済みレコードの `checkpoint_txg`) + 1（D23 (journal の役割とフォーマット) 確定項目 14 の下限、番号飛びを許容） |
| ルートスロット位置 | 個別には判定せず、`checkpoint_txg` によって決定 |
| ツリー ID 水位 | マウントが最初のファイルを発行しない場合は同一 |
| ツリーテーブル内の inode ツリー、extent ツリー、墓碑、系譜、deleted_inodes のルートポインタ（後者3つはツリーテーブルエントリであり種別 9, 10, 11、ルートレコードのフィールドではない） | 同一；マウントが継続実行された場合は、継続実行後のバージョンに従って判定：マウントが返却した継続実行の各ステップのうち txg が R2 のバージョン以下の最新ステップの各ルートポインタに等しい；ツリーテーブルユニット自体の配置先は比較しない |
| ツリーテーブル種別 6, 7, 8, 12 のルート、誕生 txg、ヘッド ID、ツリー ID、種別 | 一致必須（種別 6, 7, 8, 12 のルートは常にオールゼロ）；誕生 txg のルート間比較は別途 I-9.14 (ツリーテーブルエントリの誕生 txg のルート間不変)、ヘッド ID は別途 I-9.27 (ツリーテーブルエントリのヘッド ID の値) が単一イメージ上で判定 |
| インスタンス表およびツリーテーブルのユニットポインタ | バイト比較は行わず、「ツリーテーブルユニット自体の配置先を比較しない」と同様とする |
| スキャンレポート | 適用 0 件、水位超過 0 件 |
| 回収閾値 t と固定スロット p（ルートスロットオフセット 428..437） | 比較しない：ユニットを伴うパブリッシュのたびにこのプロセスのルートリングと固定から改めて算出し、ゼロユニットのパブリッシュは前の版をそのまま写すので、マウントが行うパブリッシュはこれらを変える（F やアンマウント記号と同類）；単一イメージの性質であり、プールレベル checker の I-7.18 (ルートレコードの回収閾値は現に算出した閾値に等しい) と I-7.19 (固定されたスロットには本インスタンスが書いたルートが住まない) が判定する |
| ルートスロットオフセット 437..466（アルゴリズム、nonce、MAC） | 0 であることが必須；単一イメージの性質であり、プールレベル checker の I-2.4 (ヘッダチェックサム対象範囲) が判定し、マウント透明性では比較しない |
| magic、flags の残りビット、自己検証チェックサム、スロット全体チェックサム | 比較しない（構造的性質であり、スロット読み出し時に判定） |
| システム構成スロット世代番号 | 比較せず、I-7.16 (システム構成スロット世代番号とスロット番号の関係) に委ねて R2 のイメージ上で判定 |
| システム構成の journal_tail | R1 ≤ R2 ≤ R2 リング内の自己検証済みレコードの最大カウンタ、マウント完了時は等号成立 |
| システム構成の journal インスタンス番号 | マウント完了時は R2 ルートのインスタンス番号と等しい；二次クラッシュ時は R1 のインスタンス番号または新インスタンス番号のいずれかを取る |
| 割り当てレコード（ルートレコードの `mapping_root`、`allocation_record_tree_root` およびツリーテーブル種別 3, 4 のルートを含む） | テーブル全体は比較しない：POSIX 内容が同一であり、かつプールレベル checker が R2 のイメージに対して緑と判定すること；`mapping_root`：R2 オールゼロ ⇔ R1 オールゼロ；`allocation_record_tree_root`：R1 がツリーを保持 ⇒ R2 オールゼロ、R1 のツリーテーブルが 0 件 ⇒（R2 非ゼロ ⇔ 新インスタンス番号 ≥ 2） |

システム構成の実行時変数（スロット世代番号、journal_tail、journal インスタンス番号）が v1 にフォールバックした場合、それら自身の行の規則に従って判定し、v1 と同一であることとしては判定しない。

**射程**：refinement の限界価値は、クラッシュポイント再生の列挙では網羅できない幾つかの次元 —— ペイロードバイト、レコード数 / セグメント数 / txg 数（リングラップアラウンドを含む）、直前のタイムラインの残留物、スロット数とデバイス数 —— にのみ存在する；クラッシュポイントの次元自体はクラッシュポイント再生によりすでにフル実行で列挙されている。「チェックサムを通過した」ことはモデル内ではブールフラグに過ぎず、**refinement が完全性を検証したと解釈してはならない** —— 確定項目 14 における D4 (チェックサムの位置) の「『Merkle を検証した』と解釈してはならない」と同形であり、D13 (検証ルート) 確定項目 4 の CRC32C 衝突確率に対する「認識した上での受容」と同一の判断である。D13 (検証ルート) 確定項目 4 は refinement の左側の `Crash`（モデル層の列挙ドメイン）であり、右側の `Spec` ではない；これを spec とみなすと refinement は自明に真となってしまう。I-6.5 (nonce 水位の単調性) の形式化は撤回された；D18 (ブロックに含める情報) 確定項目 14 が確定し、かつ nonce 割り当て規則（単調カウンタ + 前借り水位、水位の使用前永続化）が独立項目として確立された後にこのセルを再開する —— 再開時、その唯一の真の利点は nonce が POSIX から不可視である点であり、その spec は O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の model と構造的に同源になり得ず、衝突 2 (D13 (検証ルート) 確定項目 13) の影響を受けない。

M1–M4 の前提とドメイン：セマンティクス層は「同一の (インスタンス番号, カウンタ) はすべてのタイムラインを通じて唯一の内容に対応する」ことのみに依存し、D23 (journal の役割とフォーマット) 確定項目 14「このバージョンの失敗処理」の同一再送によって担保される。ルートスロットの書き込み失敗により `checkpoint_txg` を進めて再送するパスが実装された際に再検証する；前提ドメインは E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) の正当な履歴ファミリ（正当な書き込み側が書き出した履歴の任意のディスク永続化部分集合であり、新しいルートスロットは読み出せない場合がある）を取る。M2 の「その他のフィールドは選択されたルートと等しい」において、レコードから再構築されたルートのアンマウント記号は `NotWrittenByTheUnmountSequence`（D22 (ユニットの原子性をどう合成するか) 確定項目 7「他のルートは 0 を書き込む」）であり、選択されたルートの記号と異なる可能性があるため、「アンマウント記号を除く」と明記している。M3 の単調性の系は、直列コミットかつ故障を含まないクラッシュ状態においてのみ成立する。M4 はルート選択と連動して初めて「確認済みの」完全なパブリッシュがリカバリによって破棄されることを排除する。前提条件が揃っている必要がある：故障を含まないクラッシュ状態（確定項目 4 の列挙ドメイン）、D22 (ユニットの原子性をどう合成するか) 確定項目 7 のルート選択に従って選択されたルート、インスタンス境界を跨がないチェーン（D23 (journal の役割とフォーマット) 確定項目 14 注 1）、新インスタンスにおいて自インスタンスで書き込まれたルートが2枚のディスクをカバーするまでいかなる fsync も復帰させないこと（D16 (パブリッシュセマンティクス) 確定項目 8 のウォームアップ）；故障を含むドメインにおいては排除されない —— インフライト上限によりレコードが完全なパブリッシュが切り捨てられる場合があり、ディスク1台が丸ごと喪失した際に指名ユニットが1部欠損したバージョンは適用されない（C519 (ディスク丸ごと喪失時に直前に確認されたバージョンがリカバリで破棄される)）。

マウント透明性の射程：故障を含まず、書き込み可能マウントが成功したクラッシュ状態のみを管轄する。書き込み可能マウントが拒否された状態については透明性を判定せず、拒絶自体が正当であるか否かを判定する：拒絶が正当 ⇔ 拒絶理由が D23 (journal の役割とフォーマット) 確定項目 14 のリストに含まれ、かつその条項の判定基準に基づいてクラッシュイメージ上で自前で再計算して成立し（拒絶理由「読み出しフェーズの判定後にシステム構成がまた読み出せない」は補い書き例外を伴う：D23 (journal の役割とフォーマット) 確定項目 14 のリストではこの拒絶は、あるディスクに自己検証済みのシステム構成が1部もなく、かつ D22 (ユニットの原子性をどう合成するか) 確定項目 8 の補い書きゲートを通らないときにだけ成立する；判定器は「1部もない」の半分だけを再計算し、補い書きゲートの五条件は再計算せず、補い書きゲートが通したディスク面はマウントできても拒絶せず、クラッシュポイント再生のオブザーバーが判定する（確定項目 6）；スロットの自己検証の判定はプールレベル checker のディスクごとの数を読み、コア層の解析を使わない。実装は実装バッチで行う）、かつディスク上の生バイトが不変であり、かつ読み取り専用読み出しが許容集合内にあること；リストにない拒絶理由についてはクラッシュ注入第2区間の記述に従って赤を報告する。故障を含む状態は射程外である：番号取得の瞬間に1スロットが読み出せないことと N-構成が同一の拒絶理由として報告されるため、読み出し故障を含む階層に接続する前に分離する必要がある（C664 (マウント拒否の正当性再計算が2種類の拒絶理由を1つに統合して報告する)）。フィールド別表におけるツリー ID 水位、2つのツリーのルートポインタ、`checkpoint_txg`、スキャンレポートの4セルは推論されたものである（三者判定は弁護側の計算式を採用し、未測定）、F のセルの「押し上げた」半面は未測定である；ルートレコードの `mapping_root`、`allocation_record_tree_root`、墓碑、系譜、deleted_inodes の3つのルートポインタ、アルゴリズム、nonce、MAC、システム構成のスロット世代番号および journal_tail が各々どのカテゴリに帰属するかは、C655 (マウント透明性のフィールド別表に未分類フィールドが存在する)、C687 (mkfs 水位からツリー構築したバージョンの配置先で判定不能な数セル) の判定 Y1, Y2 に従ってフィールド別表内にパディングで補完して記述している。

**根拠**：
- E42 (1トランザクションあたりのレコード数)：「分断された ⇒ 不正」を恒等式とするためにペイロードを固定した結果、「分断されたが偶然に正当」であるクラスが構造的に不可視となった —— M2 がペイロードを記号化して補完したのはまさにこの点である。
- 三者レビュー：資料 `_d13-item1-background.md`、3つの経路の出力 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；3経路一致して旧候補を最下位とし、2つのクラウド経路はいずれも「プレフィックス判定とコミット原子性」の接合部に収束し、メイン agent は閉鎖性に基づいて最新選択とパブリッシュ原子性を同一ユニットに統合した（判定および旧候補撤回の3つの閾値の原文は変更履歴にある）。
- E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界)：有界内において本物の `replay_journal` に対し M2 のセマンティクス層、M4 および M3 の両解釈を判定し、ミューテーション殺傷限界を測定；ユーザーは 2026-10-01 にこれを軽量ツールと決定；C ドメインの正当なクラッシュ状態数は2種の分割法で記述（甲分割法は D16 (パブリッシュセマンティクス) 確定項目 7 のバリア位置）；敵対的ファミリにおける不動点の違反はすべて I-8.12 (同一インスタンスレコードの txg 非減少) が成立しないレコード表上にあり、述語が成立する側では違反 0（第3回再実行第1区間）；ルート選択結合判定ファミリが本物の `choose_root`、`scan_journal`、`replay_journal` を呼び出し、有界内・正当・故障なしの条件下で確認済みの完全なパブリッシュが失われない（第3回再実行第2区間）；ルート選択順序の4つの改悪変種のうち、正当な履歴 K, K9, KW ファミリの有界内において最初の有効なものを選択のみが赤となり、他の3種は敵対的ルートリング KA 上で赤となる（第3回再実行第3区間）。
- 三者判定（M1–M4 の文面と配置先）：`research/prompts/m3-formal-r1-main-verification.md`、`research/prompts/m3-formal-r2-main-verification.md` 第2節、`research/prompts/m3-formal-r3-main-verification.md` 第2, 3節；M3 単調性の系の4段階の導出は `m3-formal-r2-opus-output.md`「3.2 推論：直列コミット下で、j1 と j2 から到達バージョンが単調であることが導かれる」の節にある。
- 三者判定（M3 不動点の前提条件への I-8.12 (同一インスタンスレコードの txg 非減少) の追加と配置先の現状）：`research/prompts/m3-formal-code-r1-main-verification.md` 第2節不動点前提条件セル（条項に命中、コードには命中せず）および第3節第5項；ハーネス側の前提が Verus の前提条件より広く、違反がすべて当該述語不成立側に位置することの照合は、スケジューリング記録 `records/2026-10-01-里程碑三形式化接入.md` 第10バッチ「10-5 返却」にある。
- ユーザー決定（2026-10-01 ポップアップ、すべて推奨通り）：アンマウント記号の除外、M4 射程文の前提条件の完全記述、単調性のバージョン順序に (`checkpoint_txg`, インスタンス番号) を採用、txg 上限到達時は「no_successor を報告し1件も適用されなかった場合は同等とみなす」を採用；書き戻しは K10 判定完了後にまとめて実施、原文は変更履歴にある。
- マウント透明性：実験なし：判定手法はクラッシュ注入内（`crates/singlefs-harness/src/mount_transparency.rs`、checker 層のクラッシュ注入がこれを呼び出す）にあり、実験ではない；3回の判定において攻撃側の数値はレプリカハーネス上で測定され、コードラウンド第2回で攻撃側がマージ済みコード上で測定した第1次クラッシュ 147 806 状態、二次クラッシュ 234 773 状態 0 赤について、メイン agent は再実行していない。
- 三者判定（マウント透明性）：`research/prompts/m3-formal-r4-main-verification.md` 第2節「マウント後に再リカバリ：何を法とするか」「比較対象と射程」の2行および第4節第2, 3項（独立した第4条、比較対象のバージョン、射程）；`research/prompts/m3-formal-r5-main-verification.md` 第2節 U5, U6 の2行および第4節第5, 6項（フィールド別表、二次クラッシュの v1 比較、正当な拒絶の条件）；`research/prompts/m3-formal-r6-main-verification.md` 第2節 U5, U6 の2行（txg は下限のみ判定、拒絶の判定基準はクラッシュイメージ上で自前で再計算）および「フィールド別表で触れていないフィールド」の行；`research/prompts/m3-formal-code-r2-main-verification.md` 第2節 K2 の各行（実装は判定通り、成立；ウォームアップとマウントによる F 引き上げのセグメント分割）。
- ユーザー決定（マウント透明性に二次クラッシュを追加、2026-10-02 ポップアップ、原文は変更履歴にある）：`research/prompts/m3-formal-r4-main-verification.md`「ユーザー決定（2026-10-02）」二次クラッシュの行。
- ルート選択順序変種の一文：スケジューリング記録 `records/2026-10-01-里程碑三形式化接入.md` 第12バッチ E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) 第3回再実行第3区間判定 Q2 返却の項目（メイン agent がこの文を補完すると判定）。
- マウント継続実行をマウントの一部とみなす、F の境界はマウントが返却した F 引き上げの書き込み表末尾における位置とする：ユーザー決定 2026-10-03「継続実行はマウントの一部とみなす」（原文は変更履歴にある）；コードラウンド第3回判定 `research/prompts/m3-formal-code-r3-main-verification.md` 第3節第3項（境界は書き込みストリームの位置に基づき取得、見つからない場合も panic しない）；実装 `crates/singlefs-harness/src/mount_transparency.rs`。
- 拒絶が正当である一文における拒絶理由「読み出しフェーズの判定後にシステム構成がまた読み出せない」の再計算は補い書き例外を伴い、スロットの自己検証の判定はプールレベル checker のディスクごとの数を読む：三者判定 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第2節 Y1、Y3 の2行と第3節のユーザーへ渡す表の「条文」「G4 拒絶アームのスロット自己検証」の2行；ユーザー決定 2026-10-22「プールレベル checker を読むように変える」、ゼロラウンド、原文は変更履歴に記載。

**未済の検査**：C77 (再生起点の未定義)：最新選択とプレフィックス判定の接合部にある；C649 (Verus で判定できない2種のコールバック結果)：弁済済み（M2 セマンティクス層の逆方向チェーンと指名ユニットは依然として後置条件としてのみ記述でき、Verus 側では判定せず、2つのディスク読み出しコールバックは `crates/mutations.tsv` のミューテーション行でカバーする方針に変更、未済の検査弁済済み一覧を参照）；C650 (ルート選択と再生の結合判定がない)：M4 の射程文はルート選択と再生が結合して初めて成立する；C653 (リングスキャンのグルーコードのメモリがディスク数に応じて増加する)；C654 (必赤世界の期待行数がソルバ環境によって変動する)；C655 (マウント透明性のフィールド別表に未分類フィールドが存在する)：弁済済み（フィールド別表は判定 Y1, Y2 に従ってパディングで補完、未済の検査弁済済み一覧を参照）。

#### 確定項目 2：まず直列コミットとする

**確定事項**：トランザクション層はまず直列コミットを公約する —— `.claude/rules/fs-design.md` の「判定基準が書けない場合は、まず1つの経路のみを作成し、かつ**低速であっても正しい経路を先に作成する**」を具現化するものであり、直列化は本プロジェクトが明示する既定値であって、2つの選択肢から選んだ結果ではない。交差点を越えた後に並行を開くのであり、交差点の前に並行を公約するのではない。将来の並行コミットに対する事前の制約事項：レコードはトランザクション番号順に追加され、トランザクション番号 N の fsync は復帰前に < N のトランザクションがすべて追加済み ∨ 無効化済み ∨ インスタンス切り替え確定または読み取り専用遷移済みであることを待機しなければならない（無効化はコミットパイプラインのメモリ状態であり、並行する後続から可視）；直列コミット下ではこの規則は空転する。

**射程**：直列化の上限は1回の checkpoint あたり約 1.1 万エントリである（シングルスレッドのマージ所要時間が1回のデバイス同期に追いつく交差点、計算式は C58 (直列コミットの上限到達時の警報不在) にある）：交差点以下ではデバイス同期がボトルネックであり直列化に不利はないが、交差点を超えるとシングルスレッドのマージが上限となり安全マージンが消滅する。確定項目 2 はレコードヘッダのフィールド一覧を決定せず、新規プールでのファイル作成時に書き出されるバイト列を変更しない。性能上限と crash refinement の適用範囲を決定する。並行コミットは依然としてクラッシュセマンティクスに影響を与える：「プレフィックス」は全順序上の概念であり、並行下ではプレフィックスが完全なトランザクション B と不完全なトランザクション A を同時に含む可能性がある；レコードヘッダにはすでにトランザクション境界フィールドが含まれている（D23 (journal の役割とフォーマット) 確定項目 7：トランザクション番号 8 + コミットマーカー 1）。

**根拠**：
- E17 (write buffer マージ上限)：シングルスレッドの sort-merge の上限は交差点の半分である；リーフ分割によるロックフリー並行処理は交差点を越えた後の退避口であるが、それが十分であると既定で仮定することは許されない。
- E44 (シーケンス番号ビット幅の実測とコスト)：ローカル fsync レートは交差点のもう半分である。
- E42 (1トランザクションあたりのレコード数)：直列下であっても1つのトランザクションが複数レコードに跨がる場合は同様にプレフィックスによって分断され得る —— レコードヘッダのフィールド一覧を決定するのは「1つのトランザクションがちょうど1件の journal レコードを生成するか否か」であり、直列 / 並行ではない。
- ユーザー決定、変更履歴に記録；将来の並行コミットに関する事前制約は C113 (スキャン再構築時に複数バージョンユニットの現行バージョン判定に入力がない) 確定事項 P1 に由来する。三者検証は経ていない：これはすでに明記されている既定値を確認するものであり、真に三者検証を経るべきはいつ並行に変更するかという点であり、そのステップは実施されていない。

**未済の検査**：C58 (直列コミットの上限到達時の警報不在)。

#### 確定項目 3：RefFS を極限まで突き詰め、自前で永続化とクラッシュセマンティクスを処理する

**確定事項**：機能のみを扱うインメモリモデルにとどまらず、自前で永続化とクラッシュセマンティクスを処理する参照実装（RefFS）を作成し、RefFS を「予備候補」から**作成すべき対象**へと引き上げる。「極限まで突き詰める」はセマンティクスレベルを意味する：RefFS はメモリ内でのみ POSIX 操作に従って動作し、各 fsync 後に何が永続化されたとみなされるか、クラッシュ後にどのような POSIX 状態の観測が許容されるかを自前で定義し、ディスク上フォーマットは持たない；独自のディスク上フォーマットを持ち、自前でディスク書き込みとリカバリを行う層（工数はもう1つのファイルシステムを開発することに匹敵）は作成しない。RefFS と実装の間でクラッシュポイント列挙のコードを共用することを禁ずる：極限まで突き詰めた後、「3 oracle 結託テスト」（確定項目 6）が防ぐ共通の誤解の最も危険な落とし穴は、「RefFS と実装が同一のクラッシュセマンティクスに関する誤った理解を共有していないか」である。

RefFS の由来と境界：

| 項目 | 確定事項 |
|---|---|
| 由来 | kb の条文のみを参照して独自に作成：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のコードをコピーせず、記録ストリームを読み込まず、セグメント分割・状態計画・実装のクラッシュポイント列挙を呼び出さない |
| 配置先 | 独立したパッケージ `crates/singlefs-reference-filesystem`：依存関係クロージャ（dev-dependencies を含む）⊆ {それ自身, `singlefs-format`}、std のみで済む場合は std のみを使用；ソースコードは `singlefs_core`、`singlefs_checker`、`singlefs_harness`、`singlefs_checker_tier` を参照しない（リネームされたエイリアスも同様）；`singlefs-core` と `singlefs-checker` の依存関係クロージャにこれは含まれない。3条はゲート checker-independence-and-sync の checker-implementation-disjoint セルが判定する |
| 入力ホワイトリスト | すべて呼び出し側から渡され、5点のみを受理：各操作の結末（成功、いかなる書き込みの前での拒絶、書き込み後のエラー、同一再送）；クラッシュが何番目の操作で発生したか（hi）およびその帰属ルール；ロールバック先 m；各リカバリで読み出された内容；明示的なパブリッシュポイント |
| 第1版の操作 | 最初のファイル、上書き、パブリッシュ呼び出し、fsync、ロールバック |
| 呼び出し箇所 | クラッシュ注入および故障注入の呼び出し箇所において、RefFS の判定と O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官の全4条の判定基準の論理積を取り、いずれか1つでも赤であれば赤とする |

クラッシュセマンティクス（直列コミット、故障を含まないクラッシュ状態）：

1. **fsync**：fsync が成功復帰した後、今回の fsync が待機したパブリッシュが引き連れたすべての操作が存在しなければならない（直列下では呼び出し開始前に番号取得に成功した全操作であり、ユニット単位で計算）；状態インデックスとして読み替えると、許容集合の下限 lo がそのパブリッシュの末尾まで引き上げられる。fsync がエラーを返した場合は lo を引き上げず、エラーとなったパブリッシュが引き連れた操作は存在しても存在しなくてもよい；以降の操作は次回のリカバリで読み出された状態に接続して判定する。
2. **欠落方法：パブリッシュ境界によるプレフィックス**。クラッシュが第 hi 番目の操作内で発生した際、リカバリで読み出しが許容されるのは {S_j : j = lo、または j がある明示的パブリッシュの末尾；lo ≤ j ≤ hi} であり、S_j は前回のリカバリ以降の先頭 j 個の操作を適用して得られる状態、S_0 は前回のリカバリで読み出された内容である。任意の部分集合を欠落させるような欠落方法は直列コミット下では除外され（D16 (パブリッシュセマンティクス) 確定項目 12 と非互換）、並行コミットを導入する際に再審議する。
3. **lo の引き上げ**：成功復帰したパブリッシュ呼び出し、fsync、ロールバックはいずれも lo を引き上げる（D16 (パブリッシュセマンティクス) 確定項目 7：ロールバックと fsync は同一の復帰条件）；ロールバックは1回の確認済みパブリッシュとみなされ、そのインデックスはパブリッシュ末尾として計算される。
4. **ロールバック**：ロールバックは RefFS の1つの操作であり、ロールバック後の内容は履歴全体における第 m 回の確認済みパブリッシュの後の内容と等しい；m は呼び出し側が実装の報告したロールバックルートから逆算して渡し、RefFS は対象の選定が正しいか否かを判定しない。m の番号付けはルート書き込みのすべての入口（パブリッシュ呼び出し、ロールバック、行書き込みとウォームアップ、F 引き上げ、アンマウント）をカバーし、それを書き込んだ時点の現行バージョンにマッピングされ、現行バージョンには各リカバリで読み出され新たに開始されたバージョンを含む。
5. **クラッシュ時点の帰属**：クラッシュが何番目の操作で発生したかは、「現在のセグメントを区切ったバリアがどのステップから発行されたか」に基づいて帰属させ、最初に差し押さえられた書き込みには基づかない；このルールは RefFS のみに適用され、O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官は変更しない。
6. **リカバリ後**：各リカバリの後、RefFS はまず読み出された状態が許容集合内にあるかを判定し、その状態を起点として処理を続行する。

**射程**：どの程度まで突き詰めるかと RefFS の形態のみを決定し、スケジュールは決定しない；「実装の前にまず RefFS を作成する」と解釈してはならない —— RefFS は検証手段でありマイルストーンではない（`.claude/rules/format-evolution.md`「書き込みから開始し、読み出しから開始しない」）。明示的なパブリッシュポイントは履歴内のパブリッシュ呼び出しと fsync のみを取得する：自動トリガーによるバッチ処理が具現化するまで、RefFS は明示的パブリッシュポイントのみを受け付け、自動トリガーのパブリッシュは RefFS の射程外であり、確定項目 1 の M2 およびレコード照合器に帰属する；クラッシュリカバリで最新ルートを破棄した際、RefFS は読み出された内容から新たに開始し、そのステップ自体は判定しない。故障を含むクラッシュ状態（虚偽のデバイス、ディスク丸ごと喪失 C519 (ディスク丸ごと喪失時に直前に確認されたバージョンがリカバリで破棄される)）には適用されない。独立性：RefFS は記録ストリームを読み込まず、セグメント分割や状態計画を呼び出さないが、その (lo, hi) と O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官のディスクレベル下限の入力パラメータは同源であり、C191 (同源性カウンタが1ペアのみ対象) の同源会計として記録される。D17 (実装階層とサードパーティパイプライン) 確定項目 4 はこれに基づき「入口」の実行可能定義を RefFS の操作面に委ね、fsync の永続性公約は RefFS のクラッシュセマンティクスによって定義される；O3 (独立規約実行器) は依然としてクラッシュポイント集合の独立実行器であり、責務は不変である。RefFS の第1版は `crates/singlefs-reference-filesystem` にあり、クラッシュ注入の2区間および故障注入再開の呼び出し箇所に接続済みである：harness 層は変換層 `crates/singlefs-harness/src/reference_filesystem_translation.rs` を介してこれに依存し、呼び出し箇所では RefFS と O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官の2票を判定し、いずれかが赤であれば赤とする。

**根拠**：
- 実験なし：どの程度まで突き詰めるか、RefFS の由来と入力の選定は検証投資と境界のトレードオフである；3回の判定における攻撃側の数値はレプリカ装置上で測定されたものであり、マージ先コード上では未再実行であるため（`research/prompts/m3-formal-r5-main-verification.md` 第5節第3項）、ここでは引用しない。
- ユーザー決定、原文は変更履歴にある：「極限まで突き詰める」（当初確定）；セマンティクスレベルとする（2026-10-02、分岐表 `m3-formal-7-forks.md` 内の RefFS をどの階層まで作成するかの行、当初確定事項の解釈変更）；パブリッシュ境界によるプレフィックス、呼び出し箇所の論理積、新規作成、独立パッケージ（2026-10-02、`research/prompts/m3-formal-r4-main-verification.md`「ユーザー決定」）；論理積を全4条へ厳格化、ロールバック先を入力とする（2026-10-02、`research/prompts/m3-formal-r5-main-verification.md`「ユーザー決定」）；ロールバックによる lo 引き上げ、クラッシュ時点のバリア帰属、m が全ルート書き込み入口をカバー、ホワイトリストに同一再送を追加（2026-10-02、`research/prompts/m3-formal-r6-main-verification.md`「ユーザー決定」）。
- 三者判定：`research/prompts/m3-formal-r4-main-verification.md`（第1ラウンド：fsync、欠落方法、独立性の3セルと6つの分岐）、`research/prompts/m3-formal-r5-main-verification.md`（第2ラウンド：fsync 合成文、パブリッシュ境界による補足3文、論理積厳格化、新規作成と独立パッケージ、任意部分集合除外の直列限定）、`research/prompts/m3-formal-r6-main-verification.md`（第3ラウンド最終ラウンド：ロールバックと lo、射程2文、ホワイトリスト、状態インデックスによる fsync 読み替え）。
- D17 (実装階層とサードパーティパイプライン) 確定項目 4：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) は「クラッシュは関知しない」と明記しており、fsync の永続性公約を定義できないが、最初の実行可能目標はまさに「1つのトランザクションを正しくコミットすること」である —— 極限まで突き詰めた RefFS が補完するのはまさにこの半分である。

**未済の検査**：C447 (規約実行器がクラッシュポイント再生および RefFS 比較に未接続)；C191 (同源性カウンタが1ペアのみ対象)；C658 (クラッシュ注入第3段階が失われたステップの復活を見逃す)。

#### 確定項目 4：クラッシュポイント再生は完全書き込みの部分集合のみを列挙する

**確定事項**：クラッシュポイント再生が列挙すべきクラッシュ状態集合は以下のように定義される：記録された書き込みリクエストストリームを**デバイス単位で**セグメントに分割する —— ある書き込みは、同一ディスク上の後続のバリア（またはそのディスク上の FUA 書き込み）によってのみ後続の書き込みより前に順序付けられる；現在のセグメント内で書き込みのあった各ディスクが自らのバリアまたは FUA で解放されたときに初めてそのセグメントが閉じられ、1枚でも解放されていないディスクがある場合、そのバリア（または FUA）はセグメントを閉じず、前後の書き込みは同一セグメントとなる。1つのクラッシュ状態は「先行するいくつかのセグメントがすべて永続化され、現在のセグメント内の各書き込みが各自の状態を任意に組み合わせて取得し、後続の各セグメントはすべて未永続化である」状態である；列挙の単位は**1回の完全書き込み**である。FUA を伴う書き込みは自ディスクのみを解放する：それはそのディスク上で先行するバリアで隔てられていない書き込みと同一セグメントとなり、他ディスク上の書き込みがそれによって永続化されることはない。1回の書き込みは2つの状態（永続化 / 未永続化）を取る；**インプレース上書き**（ユニット書き込みではなく、1セクタより長く、対象範囲に元々データが存在していた書き込みであり、システム構成スロットの書き込みや旧レコードを上書きする journal 書き込みがこれに該当）は第3の状態「新旧いずれも読み出せない」を追加で取り、フル実行およびクイック層の双方が列挙する；第3の状態のイメージはその書き込みの全範囲を覆い、新旧で異なる区間の前半は新しいバイト、後半は古いバイトとする（セクタ単位ではなくバイト単位で断裂）；その他の書き込みの断裂状態は一律に「今回の書き込みが未永続化」に統合する。「未永続化」の位置には、イメージ内に**クラッシュ前のその位置の古いバイト**を配置し、ゼロ埋めは行わない。また、これらのイメージ（第3の状態を含む）は実際に生成して checker およびレコード照合器に渡す必要があり、単なるカウンタとして扱うことは許されない。イメージのミラー書き込み（D2 (RAID ストライピング戦略) 確定項目 6 の w ≥ 2、D22 (ユニットの原子性をどう合成するか) 確定項目 8 の journal ミラー）は**デバイス単位で**各1回の書き込みとして計算し、harness の状態数は自らの書き込みストリームに従って別途閉じた式を計算する。**同一ディスク上で連続するバリアは1つに統合する**：記録ストリーム内で、あるディスク上のバリアと同一ディスク上の次のバリアとの間に書き込みが一切存在しない場合（他ディスクのバリアは書き込みとみなさない）、このバリアは記録せず（`crates/singlefs-harness/src/lib.rs` の `push`）、独立モデルも同一規則に従って記録する；中間に書き込みがなければそのセグメントはクラッシュ状態を生成せず、統合の有無はクラッシュ状態集合を変更せず、操作数と種類文字列のみを変更する。

**射程**：モデル層の列挙ドメインを定義する。実デバイスの FLUSH / FUA が公約通りに効力を発揮するか否かは C6 (ブロック層セマンティクス仮定の記述誤り) に帰属し、QEMU 内で実デバイスを用いて検証する必要がある。デバイス単位のセグメント分割により、実際には到達し得ない状態が余分に生成される（1台のディスクが解放されていないとき、他ディスクの解放済み書き込みもセグメント内に残って任意の組み合わせを取る）が、到達可能な状態の取りこぼしはない。断裂は2種類に分かれる：インプレース上書き以外の書き込み（ユニット書き込み、1セクタ以下の書き込み、対象範囲に元々データが存在しなかった書き込み）の断裂状態と「未永続化」の差異は CRC32C の衝突確率（32 ビット、D19 (ブロックポインタの構造と幅の予算) 確定項目 2、D23 (journal の役割とフォーマット) 確定項目 11）のみであり、**認識した上で受容**し、断裂粒度はモデルに含めない。これは D20 (荷重面：ユニットの原子性と自己完結性) の「親ポインタを持つユニットはいかなる幅にも依存しない」と整合する；インプレース上書きが断裂した場合、その箇所の旧データも失われるが、「未永続化」では完全な旧データが保持されるため、両者は同一の状態ではなく、個別に列挙する。クラッシュポイント再生のセクタは 512 バイトとする；2本のクラッシュポイント再生ストリームにおいて第3の状態を取るのはシステム構成スロットの書き込みのみである。既知のエッジケース：第3の状態の旧バイトは「今回の書き込み以前の書き込みがすべて永続化した」バージョンを採用し、同一セグメント内のより早期の重複する書き込みが永続化しなかった組み合わせでは、古い半分にその書き込みのバイトが多く含まれる（2本のクラッシュポイント再生ストリーム上ではこのような組み合わせは一度も発生していない）；ルートスロットの書き込みが1セクタを超えかつ旧データを上書きする場合、第1版では第3の状態をサポートしない（harness アサーション；クラッシュポイント再生のプールはすべて 512 バイトの物理ブロックで構築され、到達しない）。デバイスが1回の書き込みをチェックサム衝突にちょうど重なる形状に断裂させるケースはモデル外である。D13 (検証ルート) 確定項目 5 はこれと直交する：一方は列挙ドメインを決定し、他方は crate の境界を決定する。

**根拠**：
- E77 (パブリッシュの永続化順序)：セグメントモデルの4種のバリア配置における状態数は、アームごとに閉じた式と厳密に等しい —— この列挙ドメインの数え上げは正しい。
- E142 (新規プールでのファイル作成のドライラン)：第17回実行第1区間の幾何学的感度サンプリングポイント G7（同一書き込みストリームに対する陽性対照、FUA のみを境界とする）の実測で 4097 状態、2046 違反 —— バリアが存在しない場合、ユニット、journal レコード、ルートスロットが1つのセグメント `[12]` に統合される；装置の分割法は `crates/singlefs-harness/src/segments.rs:91` の「FUA 書き込みは自身が属するセグメントを閉じる（FUA は前方の通常書き込みを永続化しないため、それらは同一セグメントとなり任意の部分集合を取る）」と一致し、「FUA が単独でセグメントを形成する」という解釈が予言した 2050 / 0 と一致しない —— FUA の一文の字面は実装の分割法に従って記述されている。
- 三者対抗および判定：[verification-build.md](../verification-build.md)「三者対抗（2026-09-03）」判定第3項 —— すべての種類のユニットがユニット全体の検証を持つ（親ポインタを持つものは親ポインタ内のチェックサムに依拠、D4 (チェックサムの位置) 確定項目 1、D16 (パブリッシュセマンティクス) 確定項目 7、D22 (ユニットの原子性をどう合成するか) 確定項目 19；自己検証ユニットはユニット全体のチェックサムと実際に検査されるコードに依拠、D22 (ユニットの原子性をどう合成するか) 確定項目 21；journal レコードはヘッダチェックサムとヘッダ内のペイロードチェックサムに依拠、D23 (journal の役割とフォーマット) 確定項目 13）、断裂状態と「未永続化」はチェックサムの観点からは同一の事象である。
- ユーザー決定（E77 (パブリッシュの永続化順序) および三者対抗に従って決定；FUA 書き込みをセグメント境界とみなす一文は C313 (FUA 書き込みをクラッシュセグメント境界とみなすか否か) 完了時のユーザー決定であり、2026-09-26 にユーザーが実装に合わせて「自身が属するセグメントを閉じ、前方のバリアで隔てられていない書き込みと同一セグメントとする」に変更）、変更履歴に記録。
- デバイス単位のセグメント分割（1台でも解放されていないディスクがあればセグメントを閉じない）、FUA は自ディスクのみを解放、インプレース上書きに「新旧いずれも読み出せない」を追加：ユーザー決定（コードレビュー第1, 3, 4項；2026-09-27 ポップアップで「A 統合、デバイス単位でバリアを記録」「同様に補完、フル実行も走らせる」「インプレース上書きに第3の状態を追加」を選択）、第3の状態をバイト単位で断裂させることはメイン agent が同日に決定し、ゼロラウンド攻撃された、変更履歴に記録；実装と判別力（ディスク1のバリア1つのみを吸収、セグメント末尾 `26,2,1,2` → `26,5`、クラッシュポイント再生小ストリーム内でレコード照合器が報告）は `m2-rev-b3a2-implementer-report.md` 第1, 3節にある。
- 同一ディスク上で連続するバリアを1つに統合：ユーザーが 2026-09-28 に「統合し、本日のレコーダーに追従する」と決定（E142 (新規プールでのファイル作成のドライラン) 第19回実行で登録された停止 S19-crates に遭遇：独立モデルは双方を記録し、レコーダーは2つ目を統合、ウォームアップと全ストリームの4セルで操作数と種類文字列が異なるがクラッシュ状態数は同一、報告 `e142-r19-runner3-report.md` 結論段、登録 `e142-r19-prereg.md` の R40）、原文は変更履歴にある。

**未済の検査**：C6 (ブロック層セマンティクス仮定の記述誤り)：仮想マシンのデバイス側ログを用いてクラッシュ状態を再構築し、クラッシュテストが必ず赤になる側の半分。

#### 確定項目 5：kb から生成された定数モジュール1つのみを共有する

**確定事項**：checker と実装の間で共有するものは唯一つ：kb のフィールド表から生成された定数モジュールであり、ジェネレータが kb から読み取り、双方が消費するのみで、何人も手作業で改変してはならない（`.claude/singlefs-ai-sop/rules/machine-first.md`：重複は生成すべきであり、手作業で書き写してはならない）。**それ以外は一律に共有しない**：アドレス空間の newtype は各自で宣言し、フォーマット解析は各々独自に実装し、チェックサムは各々独立した実装を用い、トラバーサルおよび会計コードの積集合は空とする（C12 (増分セマンティクスの共用) はゲート checker-independence-and-sync の checker-implementation-disjoint セルが定めた判定基準であり、形態は crate 粒度の依存関係クロージャの積集合でありシンボルレベルではない、「フォーマット解析と定数を除く」という例外規定はまさに本項目が利用している）。

- **判定幅を他所から取得することを禁ずる**：I-7.5 (ルートスロットの判定幅アライメント) および I-8.2 (レコードヘッダが原子ユニットを跨がない) の判定基準はマウント時に検出された `physical_block_size` であり、イメージ内には存在しない。checker は自ら1回検出し、システム構成に記録された mkfs 時の値と照合する（D22 (ユニットの原子性をどう合成するか) 確定項目 2「mkfs 時にプール内最大値に基づいて区画を割り当て、マウント時にデバイスごとに再計算して照合する」の checker 側の形態）。**検出不能時は「宣言値、未検出」と報告し、検出されたものとみなすことを禁ずる** —— ファイルをデバイスとするイメージがこれに該当し、クラッシュポイント再生の大多数のイメージはファイルであるため、この規定はエッジケースではない。異種混在プールにおいて判定幅は単一の数値ではなく集合であり（C17 (デバイスジオメトリをグローバルパラメータとみなす誤り)）、checker は mkfs 時の最大値取得の集約を再計算しなければならない。さもなければ正常な異種混在プール上で誤って赤を報告することになる。
- **ジェネレータは kb に配置先のない定数の出力を拒否する**、またスカラー値のみを出力する：仮に「フィールド表に基づいて計算されたオフセット関数」を出力し始めた場合、それは D13 (検証ルート) が明示的に共有を禁止したフォーマット解析となってしまい、もはや定数ではない；判定基準は出力物に分岐と演算が含まれているか否かである。
- **CRC32C を各自で実装する前提はパラメータの固定である**（多項式、初期値、入力/出力の反転、排他的論理和出力）：パラメータを固定しなければ、2つの独立した実装が健全なイメージ上で異なる結果をもたらし、それは独立性ではなく誤判定の赤となる。

**射程**：checker と実装の crate 境界を管轄する；kb 内でそもそも誤って記述された値は管轄できない —— 生成された定数を1つ共有することと双方で個別に書き写すことのいずれもそのクラスの誤りを捕捉できず、直交するゲートを必要とする（C94 (登録されたフォーマット定数と後続の確定事項の不一致)）。実装と確定事項の乖離：`crates/singlefs-format` は手書きであり生成されたものではない（値はゲート code-source-discipline の format-constants セルが kb 内の `format-const` マークに従って固定しており、マークされたもののみを固定）、ジェネレータは存在しない；`crates/singlefs-checker` は `singlefs-format` のみに依存し、CRC-32C はビット単位のものを別途実装している；checker はシステム構成が宣言した `physical_block_size` と `io_min`（`crates/singlefs-checker/src/image.rs` の `geometry_of`）を取得し、「検出 / 宣言」の2種類の出所の型は `crates/singlefs-checker/src/lib.rs` の `DecisionWidth` である：宣言値の形式（`DeclaredOnly`）はすでに I-7.5 (ルートスロットの判定幅アライメント) の判定に接続されており（実装済み、`decision_width_declared_by_the_chosen_system_configurations` は各ディスクで選択されたシステム構成が宣言する max(`physical_block_size`, `io_min`) の最大値を取る）、検出値の形式（`Probed`）は型定義のみが存在し（`crates/singlefs-checker/src/lib.rs`）、リポジトリ全体にそれを構築するコードはなくいかなる判定にも接続されていない；I-8.2 (レコードヘッダが原子ユニットを跨がない) は未実装である。システム構成における mkfs 時の `physical_block_size` フィールドは D22 (ユニットの原子性をどう合成するか) 確定項目 9 のジオメトリセグメント（4バイト）にある。

**根拠**：
- 実験なし：共有の境界は検証の独立性を担保する規律であり、測定可能な量はない；その判別力は双方が個別に実装した検査が1条ずつ赤を報告して示すことに依拠する。
- 三者対抗：資料 `_checker-sharing-vehicle-background.md`、各経路の出力 `research/prompts/checker-sharing-vehicle-*`、状況は [verification-build.md](../verification-build.md) にある —— 順方向経路は D13 (検証ルート)、C12 (増分セマンティクスの共用) から「定数は例外内、他は禁止」を導出；代替逆方向経路とローカル経路は各々同一の結論を構築：kb 内で誤って記述された値は共有と個別記述のいずれでも捕捉できない。
- ユーザー決定、変更履歴に記録。

**未済の検査**：C17 (デバイスジオメトリをグローバルパラメータとみなす誤り)；C387 (定数モジュールが手書きで kb から生成されていない)；C388 (checker の判定幅を自前で検出しない)。

#### 確定項目 6：互いに独立な3つの oracle

**確定事項**：3つの oracle が各々異なる対象を判定し、独立性の根拠は各々異なる：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判定対象 | 独立性の根拠 | 検証できないもの |
|---|---|---|---|---|
| O1 | インメモリモデル突き合わせ (HashMap 理想ファイルシステム) | POSIX 可視セマンティクス：内容、サイズ、ディレクトリ構造；クラッシュのない箇所での内容の期待値はこれに帰属し、クラッシュ後に許容される観測集合は RefFS（確定項目 3）に帰属する | ディスクに触れず、ディスクフォーマットを解析しない | ディスク構造が自己整合しているか；クラッシュセマンティクス；**同一の POSIX セマンティクスに対する2つの実装の共通の誤解** |
| O2 | 独立パーサ + checker | ディスクバイト列が I-1 (ブロックの自己記述) ～ I-6 (暗号化不変条件) を満たすか | **checker はテスト対象 crate のいかなるデシリアライズ / トラバーサルコードも呼び出してはならず**、解析処理を独自に再実装し、raw デバイスを直接読み出す必要がある | 構造の自己整合性のみを判定：あるブロックが別の**正当な**内容で上書きされた場合、Merkle チェーンは依然として自己整合し、会計も一致してしまう |
| O3 | 独立規約実行器 | クラッシュポイントで切断された後の各ディスク状態が規約の許容集合に含まれるか | **別の言語で実装**（Rust を使用しない）、decisions と invariants の記述のみから独自に再導出；第1版は Python で単独実装（E170 (O3仕様実行器第1版)） | 規約自体が誤って記述されている可能性 —— **陽性対照と組み合わせることで初めて意味を持つ**；射影される6種の要素の値および6種以外のルートレコードフィールド、実装自身のリカバリおよびリーダーのコード、システム構成スロットの証拠値（journal_tail）、1回のパブリッシュ内での各書き込みの前後関係 |

**独立性は検証されるべきであり、自己申告を禁ずる**：「3 oracle 結託テスト」を設計する —— 精密に構築された誤った実装（例えば nonce が常に 0）を用いて3つの関門を通し、3者すべてがパスさせた場合は共通の盲点が存在することを示しており、oracle の補完に立ち戻る。機能の正当性はこれら3つの oracle のみに依存するわけではない：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の行はすでにクラッシュ後に許容される観測集合を RefFS（確定項目 3）に委ねており、リカバリ関数の M2–M4 は Verus および E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界)（確定項目 1）に配置され、第2の入力を必要とする照合はレコード照合器（確定項目 7）に帰属する；3つの oracle 間の共通の盲点は「3 oracle 結託テスト」によって発見する。

**射程**：3つの oracle が各々何を検証できないかは表の最終列に記載されている；O2 (独立パーサ + checker) の定義域は確定項目 7 に帰属し、第2の入力を必要とする照合はレコード照合器に帰属する。**実装の現状**：O2 (独立パーサ + checker) は `crates/singlefs-checker`（フォーマット定数 crate にのみ依存）にある；O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の形態は `crates/singlefs-harness/src/model.rs` の理想モデル（メモリ上にのみ存在し、`singlefs_format` の定数のみを使用し、クラッシュは構築しない）であり、ゲート harness-model-differential-and-scenarios がランダム履歴を用いて段階的に突き合わせを行う；O3 (独立規約実行器) は kb の記述のみに基づいて書かれた Python 実行器であり、現行のものは E171 (3 oracle 結託テスト) 第1回再実行で使用された `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170 (O3仕様実行器第1版) 第2回再実行が第1回再実行で凍結された `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` から `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` をコピーし述語 W, T を追加、さらに 487 バイトの journal レコードヘッダに追従；第1回再実行で凍結された版と初回の版 `scripts/experiment-apparatus/e170_o3_spec_executor/` はアーカイブ済み）である：厳密な実行方法により判定可能であり（初回にエクスポートされたバッチ1、バッチ2において「規約の記述不足」に遭遇した状態は 0）、実装と状態ごとに一致し（両者とも完全にパス、これを正しく判定した証拠とはみなさない；「選択されたルートが自己検証済みルートの中で txg 最大のものではない」「③ 2スロット以上を受信」の2つのパスはバッチ内で 0 状態、C661 (規約実行器バッチ内で2つのパスに状態が存在しない)）、不正状態表を修正後の期待値に従って再判定すると判別力を有し（陽性対照およびブラインドテストの双方が正しく判定）、クラッシュポイント再生にはまだ接続されていない；あるクラッシュ状態を判定する手順：規約のリカバリ規則（システム構成選択、ルート選択、プレフィックスは D23 (journal の役割とフォーマット) 確定項目 14 の5条の計測条件に基づく）に従ってイメージから自前で1バージョンをリカバリし、リカバリ不能（プール全体の拒否または自己検証済みルート不在）の場合は集合外と判定；リカバリされたこのバージョンの射影（インスタンス番号、checkpoint_txg、ツリーテーブルユニットポインタ、中央マッピングツリールートポインタ、ツリー ID 水位、F）がバージョン表内のいずれかのバージョンの射影と等しく、かつそのバージョンのインデックスが [確認済み, 開始済み] の間にあること。バージョン表はベースイメージ上で自前でリカバリされた現行バージョンに加え、記録ストリーム内の各ルートスロット書き込みをストリーム順に各1バージョンとし、各バージョンの射影は実装がルートスロットに書き込んだバイト列を取得する；確認済みはクラッシュ状態で永続化した最新の書き込みより前に復帰済みの最新バージョンを取り（永続化した書き込みからのみ推論）、開始済みは永続化した書き込みが属する最新バージョンを取る。システム構成スロットはスロット選択、ルートリングおよび journal リングのジオメトリ取得、プールレベルフィールドの範囲外によるプール全体拒否の判定にのみ用いられる。したがってリカバリ規則の半分は kb の記述に基づいて別途実装されたものであり、許容集合の半分は実装自身が書き出したルートから取得される；RefFS（確定項目 3）は `crates/singlefs-reference-filesystem` にあり、クラッシュ注入および故障注入の呼び出し箇所に接続済みである；「3 oracle 結託テスト」は E171 (3 oracle 結託テスト) であり、測定値は凍結されたツリーから取得される（スナップショットダイジェストは実験ページ参照、現行の `crates/` とは同一ではない）：O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の列はモデル突き合わせ + モデルのクラッシュ判定官であり、RefFS には接続されておらず、表の O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) の行とは異なる（クラッシュ後の集合の半分において E171 (3 oracle 結託テスト) は RefFS を使用していない）；O3 (独立規約実行器) の列は E170 (O3仕様実行器第1版) 第1回再実行で凍結された版を使用；手動構築された不正イメージファミリは O2 (独立パーサ + checker)、O3 (独立規約実行器) のみに供給され、期待値は E170 (O3仕様実行器第1版) の設計者によるものであり、Q2 は登録字面通りに「存在する」（4枚とも P8）と判定し、登録 F3 に基づいて E170 (O3仕様実行器第1版) の期待値推論の誤りでありこれら4枚は不正イメージではないと判定され、真の不正イメージの中で O2 (独立パーサ + checker)、O3 (独立規約実行器) の両者が完全にパスしたものは 0 枚である；3者の出所が独立であるか（表第4列）について E171 (3 oracle 結託テスト) は Q2 において O2 (独立パーサ + checker)、O3 (独立規約実行器) のペアのみを測定しており、そのペアのパッチと期待値は E170 (O3仕様実行器第1版) の改修コードに由来するため、独立した測定値とはみなさず、O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) は Q2 に含まれていない；測定されていない項目を「互いに独立」が検証されたと記述してはならない。

**3つの oracle がいずれも判定しない2点**（第1点はクラッシュ注入とクラッシュポイント再生の両側で判定し、クラッシュポイント再生側のオブザーバーは実装バッチで行う）：第1に、読み出し故障のないクラッシュ状態において、D23 (journal の役割とフォーマット) 確定項目 14 の書き込み可能マウントの証拠判定基準が真であること（システム構成スロット内の証拠値がディスク上から読み出せるレコードより先行している）：クラッシュ注入第2段階は拒絶が正当か否かを判定する際にクラッシュイメージ上でこの判定基準を再計算するのみであり、真であれば正当と判定する（マウント透明性の射程は故障を含まない状態のみを管轄する、確定項目 1）；「このクラッシュ状態には読み出し故障が注入されていないのに、判定基準が真である根拠は何か」を問う判定器が必要である：クラッシュ注入第2, 3段階は読み出し故障が注入されていないクラッシュ状態上でこの判定基準を再計算し、真であれば拒絶が不正であると判定する（`crates/singlefs-checker-tier/src/crash_injection.rs` の `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`、C670 (読み出し故障のないクラッシュ状態での書き込み可能マウント証拠判定が真であることの判定器不在) は弁済済み、未済の検査弁済済み一覧を参照）。「読み出し故障のないクラッシュ状態において各ディスクに少なくとも1部の自己検証済みシステム構成が存在する」という性質は、クラッシュポイント再生で mkfs 復帰後の各ストリームに付けるオブザーバーが判定する：オブザーバーはストリームごとに登録し、mkfs を記録するストリームには付けず、ベースイメージに対して新たに現れた、自己検証済みシステム構成が1部もないディスクだけを記録し（断裂域を含む）、スロットの自己検証の判定はプールレベル checker のディスクごとの数を読む（`crates/singlefs-checker/src/image.rs` の `system_configuration_redundancy_report`）。さらに登録の完全性のテストケースを1つ置き、システム構成スロット書き込みセグメントを含むストリームすべてにそれが付いていることを判定する（実装バッチで実装）。ランダム履歴のうち補い書きゲート（D22 (ユニットの原子性をどう合成するか) 確定項目 8）を通らない形は、クラッシュ注入第2, 3段階の G4 拒絶アームが判定する；補い書きゲートを通る形はランダム履歴では誰も判定せず、正しい実装はこのようなディスク面を残さない（2回の抽出ともに当たらなかった）。mkfs 復帰前のクラッシュ状態はオブザーバーと G4 拒絶アームの射程外であり、mkfs はディスクごとに2スロットを同一ループ内で書き込みその後に初めてディスクごとにバリアを発行するため、クラッシュ注入の候補セグメントと E171 (3 oracle 結託テスト) のベースイメージはいずれも mkfs の後に取得される；G4 拒絶アームと C670 (読み出し故障のないクラッシュ状態での書き込み可能マウント証拠判定が真であることの判定器不在) の条項にはもう1点形状の相違がある：システム構成を選択できない場合、その条項の再計算は `None` を返すが G4 は真とする。この時読み出しフェーズ自体がすでに拒絶しており、このセルには到達しない。第2に、1回のパブリッシュ内での各書き込みの前後関係：O2 (独立パーサ + checker) の I-7.12 (システム構成の F が同一ディスクルート上の F 以上) は、F 引き上げシーケンスにおいてまずシステム構成を書き込むステップを判定する；ミューテーションで指名されたテストケースは E171 (3 oracle 結託テスト) 270行目の種類（パブリッシュ復帰後の各クラッシュポイントにおける証拠値）のみを判定し、242行目で指名されたテストケースは失敗パスの凍結を判定する；`layout/01-first-txn.md` 第8節はルートスロットの FUA とシステム構成スロットのローテーションを2セグメントとして登録しているが、登録簿上に存在するに過ぎない；その他の書き込み順序は3者には含まれない。

**根拠**：
- E170 (O3仕様実行器第1版)：O3 (独立規約実行器) は「別の言語、kb の記述のみから再導出、陽性対照を配備」に従って第1版を記述可能であり、単一のクラッシュ状態を判定でき、1状態あたりの所要時間はクイック層に組み込み可能である；第1回再実行（2026-10-03 の kb に従って1回改修された実行器）は厳密な実行方法で判定可能であり、サンプリングされたバッチ状態上で実装と一致し、陽性対照は判別力を有する（初回実行で判定不能であった点および判別力不足であった2点は実験ページに記録）。
- G4（読み出し故障のないクラッシュ状態において各ディスクに少なくとも1部の自己検証済みシステム構成が存在する）：コードラウンド判定 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第2節 j2-b 成立、第29バッチで実装、射程と mkfs に関する部分は `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第2節 L3 および第3節第3項を参照。
- 両スロットとも死んだディスク面は、クラッシュポイント再生で mkfs 復帰後の各ストリームに付けるオブザーバーが判定するように変え、G4 拒絶アームは残す：三者判定 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第2節 Y2 の各行と第3節（前の2ラウンド `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`、`research/prompts/m3-close-g4-rewrite-r2-main-verification.md`）；オブザーバーの接続方法（ストリームごとの登録、ベースイメージ相対、登録の完全性のテストケース）はユーザー決定 2026-10-22「ストリームごとの登録＋ベースイメージ相対」、ゼロラウンド、原文は変更履歴に記載。数値は三者の各経路のモデルとレプリカの原型上で計測し、実験番号は立てていない。
- 3つの oracle の分担とそれぞれの独立性の由来は検証体系の構造設計である；独立性は「3 oracle 結託テスト」によって観測される（E171 (3 oracle 結託テスト)）。
- E171 (3 oracle 結託テスト)：凍結されたツリー上において、不正実装表の中に3つの oracle すべてがパスさせた行があり、1つのみによって捉えられた行もあるが、O3 (独立規約実行器) は不正実装表上で1行も赤を出さなかった；手動作成の偽造ルート、偽造継続イメージの中に O2 (独立パーサ + checker) がパスさせ O3 (独立規約実行器) が集合外と判定したものがある（測定値と限定条件は実験ページ Q1, Q2, Q3 の3行にある）。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。
- O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) と RefFS の分担文：三者判定 `research/prompts/m3-formal-r5-main-verification.md` 第2節の新規作成および独立パッケージのセル（両経路が同一方向）。

**未済の検査**：C445 (モデル突き合わせのサンプリング点が書き込みパスに応じて増加しない)；C446 (未実装の不変条件に事前登録がなく期日もない)；C447 (規約実行器がクラッシュポイント再生および RefFS 比較に未接続)；C407 (3 oracle 結託テストの判定器がゲート未導入)：E171 (3 oracle 結託テスト) は1回測定されたが、その成果物を読み込むゲートセルは存在しない；C672 (戻り値やメモリ状態が誤っているが書き込みストリームが不変であるクラス)；C679 (アンマウント記号がモデル突き合わせに含まれていない)；C874 (補い書きの分岐がクラッシュテストに入れず、実行時に見えない)：両スロットとも死んだディスク面のオブザーバーも、G4 拒絶アームのプールレベル checker を読む切り替えも、まだ実装されていない。

#### 確定項目 7：O2 は何を判定するか：単一イメージとレコード照合器

**確定事項**：

> **確定事項：O2 (独立パーサ + checker) の定義域は単一イメージである** —— 単項述語 `P(イメージ) → 成立/不成立` であり、
> 集合の要素性のみを判定し、決して identity を判定しない。**第2の入力（別のイメージ、または一連のレコードストリーム）を必要とする検査はいずれもこれに属さない。**

**第2の入力を必要とする照合はレコード照合器に帰属する**。入力パラメータは `(クラッシュ前イメージ, レコードストリーム, クラッシュ後イメージ, 永続化集合)` である —— 永続化集合は列挙器が提供する、このクラッシュ状態においてどの書き込みがディスクに永続化したかを示す情報である（クラッシュポイント再生のセグメント内完全書き込み部分集合）；ディスク上の生バイトを参照するだけでは「書き込みが未永続化であり、元の位置の内容が偶然同一であった」ことと「書き込みが永続化した」ことを区別できないため、再利用と欠損の判定には永続化集合を用いて判定しなければならない；これは独自開発第1項の「Merkle ルートチェーン差分」（確定項目 8）と同様に二項検査に属する。⚠️ **意図的にこれに番号を付与しない** —— O2 (独立パーサ + checker) をめぐる混乱は番号を呼称として用いたことに起因しており、さらに O4 を追加することは次なるドリフトの温床を提供するに過ぎない。⚠️ **「クラッシュ後イメージ」は1つではなく2つである**：harness の実行順序はまず実装自身のリカバリを実行し、その後にレコード照合器を実行するが、リカバリ自体がディスクを変更する（インスタンス切り替えによる行書き込み、インフライトな checkpoint の再送、D23 (journal の役割とフォーマット) 確定項目 14）⇒ ルート選択とプレフィックス判定は**クラッシュ状態イメージ**を参照し（「レコードストリーム」はそのリング内の journal レコードである）、比較対象は**実装がリカバリした後のイメージ**となる。

**射程**：ポインタ層の目標状態下において、1件のレコードを照合するにはフォーマット解析 + チェックサム + ルートリングの最新選択のみで足りる；論理意図レコードはいかなるディスク上オブジェクトも指名しないため、照合側はツリーセマンティクス + 会計 + 割り当て正当性を自前で実装しなければならない —— D23 (journal の役割とフォーマット) が論理意図ログを否決した第2の理由はこれに基づき第1の理由の1つの封じ込めとして書き直され、D23 (journal の役割とフォーマット) 確定項目 5 に記述されている。レコード照合器のコードは `crates/singlefs-checker-tier/src/crash.rs` にあり、クラッシュポイント再生の各クラッシュ状態に接続されている（ゲート checker-tier-crash-point-replay）。したがってレコード照合器の判定は単なるイメージの関数ではない：クラッシュ後イメージがバイト単位で同一であっても永続化集合が異なる2つの状態に対して、異なる判定を下すことが可能であり、イメージに基づく重複排除による高速化はこれには適用されない（O2 (独立パーサ + checker) および oracle は従来通りイメージのみを参照する）。クラッシュポイント再生は書き込み可能マウントをカバーしない：クラッシュポイント再生は各クラッシュ状態において読み取り専用リカバリのみを実行し（journal を参照する場合と参照しない場合を各1回）、番号取得、行書き込み、ウォームアップはクラッシュポイント再生のいかなるクラッシュ状態上でも実行しない。書き込み可能マウントはクラッシュ注入によってカバーされる（`crates/singlefs-checker-tier/src/crash_injection.rs`）：抽出された各クラッシュ状態において、読み取り専用リカバリと判定の後に同一のクラッシュ後イメージ上で書き込み可能マウントを1回起動し、再度パブリッシュを行い、プールレベル checker を実行する；マウント途中で再度クラッシュさせ、番号取得、行書き込み、ウォームアップの3区間に各1つの二次クラッシュ状態を配置し、各々において読み取り専用リカバリ、モデルへの問い合わせ、プールレベル checker、レコード照合器を実行する：第3段階においてレコード照合器に渡される書き込み表は「マウント区間から現在のセグメントまでのプレフィックス」から履歴全記録ストリーム + 全マウントストリーム + マウント後のパブリッシュへと置き換えられ、永続化集合はセグメントごとに対応する（履歴区間は第1次クラッシュの永続化集合を取り、マウント区間は二次クラッシュに従って部分集合を取り、その後のパブリッシュはすべて未永続化とする）。「比較対象は実装がリカバリした後のイメージである」の半分：レコード照合器は2つのイメージを受理するように改修され、クラッシュ状態イメージはルートが存在しレコードが存在するか否かを判定し（ルート選択とプレフィックス）、リカバリ後イメージはユニットが存在するかを判定し到達バージョンのインスタンス表を読み出す；第2段階で接続されるリカバリ後イメージはマウントとそのパブリッシュ後のプールであり、リカバリが自称するバージョンはマウントとそのパブリッシュが書き出した最新のルートを取得し、マウントが1つのルートも書き出さなかった場合は第1次クラッシュが到達したバージョンにフォールバックする。

**根拠**：
- 実験なし：**根拠は型であり、コストではない**：「バイト単位の比較」は二項関係 `R(イメージ₁, イメージ₂)` であり、**いかなる投資を行っても単項述語を二項関係に変換することはできない**；I-4.5 (廃止済み・2イメージ必要) は同一の基準によって廃止された。
- 「クラッシュ後イメージ」は2つである：D16 (パブリッシュセマンティクス) 確定項目 4 に伴い質問の焦点を絞り込んだラウンドの三者論証（資料 `_d16-items134-background.md`）においてユーザーが決定、変更履歴に記録。
- 永続化集合という入力パラメータ：ユーザー決定、原文は変更履歴にある；判定子が観測できない実証根拠は `research/prompts/m2-layer0-scale-r3-main-verification.md` 第2節のセクタ単位免除の行にある。
- クラッシュポイント再生は書き込み可能マウントをカバーせず、書き込み可能マウントと二次クラッシュはクラッシュ注入がカバーする：ユーザー決定（コードレビュー第2項、2026-09-27 ポップアップで「クラッシュ注入層で書き込み可能マウントを補完」を選択）、変更履歴に記録；クラッシュ注入側の実装は `m2-rev-b3b-implementer-report.md` にある。

**未済の検査**：なし。

#### 確定項目 8：独自開発の2点：Merkle ルートチェーン差分と世代間増分 scrub

**確定事項**：既存のツールには存在せず、本プロジェクトが独自に開発する2点：

**1. Merkle ルートチェーン差分 —— COW + トランザクションごとの新ルート発行に特有のてこ。**
履歴ルートは回収されるまで物理的に存在し、再生可能なチェーンを構成する。隣接する2世代のルートに対して再帰的に「部分ツリーポインタ + チェックサム / MAC」を比較し、
**ポインタが不変の箇所は直接スキップして掘り下げない**。計算量は2世代間で実際に変更された部分にのみ比例する。
これは従来の走査型 checker では検知できない種類の誤りを検出できる：**同期改ざん後に自己整合したインプレース書き込み** ——
ある書き込みが COW を迂回してインプレースでブロック内容を書き換え、同時にチェックサムを更新した場合、単一点で見ればそれは自己整合している（I-2.1 (チェックサムと内容の一致) は全緑）が、
ルートチェーン差分は矛盾を発見する：**COW を経ていない以上は新たな割り当てが存在するはずがなく、親ポインタは不変であるべきなのに、チェックサムが変化している**。
⚠️ **必ず陰性対照を設けること**：通常の COW コミット後に同一の検査を実行した際に完全に緑でなければならず、さもなければ通常更新のすべてを改ざんとみなしてしまう。

**2. 世代間増分 scrub。** I-1.2 (ブロックヘッダ書き込み順序パブリッシュ済み) および D5 (スナップショット / 空間会計メカニズム) の birth txg に依拠し、
「前回の scrub が走査した generation 水位」を維持し、`birth generation > 水位` のブロックのみを走査する ——
古い部分ツリーのチェックサムチェーンが変化していなければ、必然的に依然として自己整合している。**これにより、「毎コミット時に今回の変更をフル実行で検証する」ことが実時間上で実現可能となり、サンプリングが不要となる。**

**射程**：移動されたレプリカは `birth` が変化せず、永遠に水位の下に位置するため、世代間増分 scrub はそれらを検知できない。`crates/` 内にルートチェーン差分は存在せず（`grep -rn 根链 crates` でヒット 0）、世代間増分 scrub も存在しない（`grep -rn scrub crates` でヒット 0）。

**根拠**：
- 実験なし：2点とも COW のトランザクションごとの新ルート発行と birth txg の構造的性質に基づいて成立しており、いずれもまだ実装されておらず、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：C389 (Merkle ルートチェーン差分のコード不在)；C194 (世代間増分 scrub が再配置されたレプリカを検知できない)；C408 (世代間増分 scrub のコード不在)。

#### 確定項目 9：クラッシュポイント再生は3層に分割する。さもなければ実時間が爆発する

**確定事項**：

| 層 | 範囲 | トリガー |
|---|---|---|
| スモーク層 | 固定シード、固定スクリプトの負荷と登録されたクラッシュ列挙ケース、書き込みリクエスト数で規模を制限しない（規模は射程参照）、正確なフル実行 | checker 層のフル実行（確定項目 15）：ユーザー要求時に実行；コミット時は既定でクイック層のみ実行 |
| 通常層 | 中規模負荷（書き込みリクエスト数千件以上）、正確なフル実行、列挙しきれない場合は正確な枝刈りに依拠し、直ちに実行できないノードには ignore を付与、カバレッジレポートおよびゲートで個別に「今回未実行」と列挙 | 毎コミット |
| 大負荷層 | 通常層の負荷 + 大規模負荷 + 複数デバイス階層の交差、N≥5 ラウンド、正確なフル実行 | ユーザー要求時 / リリース前 |

**枝刈りと ignore を導入した後に断言できること**：有効になっているノードがすべてのクラッシュ状態において通過したと断言できる；
**断言できないこと**：ignore されたノードが通過したとは断言できない。

**フル実行層はコミット時には既定で実行せず、時刻を制限せず、ユーザーが要求した際に実行する；コミット前 = クイック層が全緑（必要条件）。**
**マイルストーン完了時、クラッシュポイント再生または判定器を変更した一連のコミットの前には、メイン agent がポップアップでユーザーにフル実行層の実行を促す；失敗した場合やユーザーが実行しない場合でもコミット自体は受け入れ可能とし、修正または追試が完了するまで未結の赤または「この段階完了時にフル実行未実行」として記録する。**
「フル実行」は2点を指す：checker-tier-crash-point-replay `--full`（ゲート checker-tier-crash-point-replay、確定項目 15 のフル実行）およびクラッシュ増幅のフル実行（`scripts/crash/crash-amplification.sh` 引数なし）。
「クラッシュポイント再生または判定器を変更した」の判定はクラッシュ判定ライブラリに基づいて動的に計算する：今回の変更によりライブラリ内に判定ブロックを持つユニットキーが一致しなくなり、一致しないキーの中に判定器ユニット（判定器エントリ fn のユニットクロージャ内の関数ユニット）の変更が含まれている場合に該当とみなす；ステージは無効化されたブロック数と変更された判定器ユニット一覧を出力し、メイン agent はそれに基づいてポップアップを表示し、パス一覧に基づく判定は行わない。
「前回のフル実行」はメイン agent が現在のマイルストーン記録に1行で記述する：日付、2点それぞれの結論。
**フル実行層は「カバレッジの最大化」ではなく「失敗の再現性」を最優先で保証しなければならない** —— フル実行層のバグを1つ修正するための実時間 ≈ `log2(N) × フル実行1回の実時間` であり、再現不可能なフル実行層の失敗の価値はほぼゼロ、
あるいはマイナスであり、赤信号を無視する習慣を助長してしまう。

**射程**：スモーク層（ゲート checker-tier-crash-point-replay）のみが存在する；スモーク層は2層に分かれる：通常のクイック層ではセグメントごとのインプレース書き込みは任意の部分集合を取り、COW 書き込みは完全永続化または完全未永続化のみを取る（ユーザーが 2026-09-26 に決定、「甲二」と呼称、C561 (レコード照合器の再利用免除が再利用の半分のみ永続化された際に誤判定で赤となる) の修正後に使用）、これは COW 部分のみがディスクに永続化した際に初めて現れるクラスの誤りを検知できず、フル実行の代替にはならない；フル実行は中断からの再開をサポートし（ゲート checker-tier-crash-point-replay `--full`）、`.claude/gate.d/stage-inputs.tsv` に登録されたクラッシュ列挙ケースは並行線1のストリームで 12230590578 状態に達しており、もはや「書き込みリクエスト数2桁」のスモーク規模ではないため、コミット時には既定でクイック層のみを実行し、フル実行は確定項目 15 に従いユーザー要求時に実行する；通常層、大負荷層はゲート内にステージを持たない。各クラッシュ状態の定義方法は確定項目 4 に帰属する；ノードの正確な枝刈り方法、ignore ノードをいつ開放するかは確定項目 10、11 に帰属する。

**根拠**：
- 実験なし：階層化はゲートの実時間に関する設計制約である；通常層、大負荷層のゲート内の負荷はまだ決まっておらず、E184 (通常層と大負荷層のコストカウント) 第1段階は候補負荷の規模と状態数区間の重なりを測定済みだが準備のみ；第2〜4段階（無効化面、コールドスループット、2通りの着地方式の固定オーバーヘッド）は未実行。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。
- 通常のクイック層に甲二を使用：ユーザー決定、原文は変更履歴にある；クラッシュポイント再生の規模に関する3ラウンドの判定 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- ユーザー決定：「我々はサンプリングではなく、枝刈り + 初期 ignore である」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節「ユーザー決定：サンプリングせず、枝刈り＋初期 ignore」の行）、「列挙には ignore マークを設定し、徐々に開放できる」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節「ユーザー決定：ignore マークを徐々に開放」の行）、「1 列挙すべきものは設計し尽くす、全網羅で設計する」「2 列挙しきれないことこそが枝刈りの出番である」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節で資料担当がゲート doc-process-records の implementation-premise セルに阻止され、ユーザーが『全網羅で設計、列挙しきれない分は枝刈り、適切な粒度』と定めた行）；第1ラウンド判定 `research/prompts/m3-prune-gpu-r1-main-verification.md`「決定の振り返り」の節：確定項目 9 を破棄。
- ユーザー決定（2026-10-01）：フル実行層は時刻を制限せず、ユーザー要求時に随時実行、「夜間」を削除、原文は変更履歴にある。
- ユーザー決定（2026-10-01）：フル実行層が全緑であるべきタイミングは、マイルストーン完了時およびクラッシュポイント再生や判定器を変更した一連のコミットの前であり、メイン agent がポップアップでユーザーに実行を促す、原文は変更履歴にある；ユーザーに委ねるセルは三者判定 `research/prompts/test-script-align-r1-main-verification.md` 第2節 t1-3。
- 三者判定 `research/prompts/test-script-align-r2-main-verification.md` 第2節 U3（「U3 t1-3 の書き戻しは字面通りには実現不能」の行）：「フル実行」が指す2つの対象、「クラッシュポイント再生または判定器を変更した」のパス判定方法、「前回のフル実行」の記録方法、「受入条件」と「非同期」を1文に統合。

**未済の検査**：C409 (通常層・大負荷層がゲート内にステージを持たない)；C635 (前回のフル実行に機械可読なマーカーがない)。

#### 確定項目 10：真に正確に網羅すべき N は「レイアウトのセット数」ではなく「コミットプロトコル構造の同値類数」である

**確定事項**：通常層の正確な網羅は、まず問題を縮小してから枝刈りを議論する。クラッシュポイントの定義域は**トランザクション層ステートマシン**であり、`.claude/rules/fs-design.md` には2つのトップレベルルールがある：
「1つのトランザクション層をすべての構造が共用する」および「唯一多様化を許さないのがトランザクション層である」。したがってレイアウトが変更し得るのは2種類のみである：

| 差異 | 変更されるもの | 判定 |
|---|---|---|
| ノードサイズ、ストライプ幅 | 1ステップで書き込むバイト数、特定ステップの繰り返し回数 | **パラメータ**、ステップ種類の新規追加なし |
| 回転 vs SSD | seek コスト、IO スケジューリング | デバイス層に位置し、**トランザクション層プロトコルには位置しない** |
| **zoned（ZNS）** | **ルートポインタをインプレース上書きできるか否か** | **構造的**：追記のみ、ルート位置のドリフト、zone finish/reset の追加 |

**具現化手段は閉じた列挙 + 網羅的な match である**：`crates/singlefs-core/src/transaction.rs` の `CommitStep`、ステップ種類の登録場所は [layout/01-first-txn.md](../layout/01-first-txn.md) 8のセグメントシーケンス登録表である。
いかなるレイアウトであっても新たな構造的ステップを導入する場合、**この共有定義を変更しなければならない ⇒ コンパイラがすべての参照箇所で分岐の補完を強制する ⇒
この diff はゲートで grep 可能である**。これにより「今回新たな構造同値類が導入されたため、これに対して独立した大負荷層の再生を新たに開始する必要がある」と自動的にマークでき、
**人間が申告を覚えていることに依存しない**（diff によるゲート範囲の計算は C8 (ゲート適用範囲の判定不能) で未済）。さらに newtype による隔離（`RotationalPtr` / `ZonedPtr`）を追加し、
D12 (対象メディア) 確定項目 2 における「共用を禁ずる」バケット（会計事実の判定パス）をコンパイル時の制約とする。
⚠️ **これには第2の責務がある（D17 (実装階層とサードパーティパイプライン) で追加）：サードパーティパイプラインの接続ポイントである。**
プロトコルは本プロジェクトに帰属し、あるパイプラインが新たな構造的コミットステップを導入する場合、本リポジトリにこの diff を送信しなければならない。

**射程**：回転ディスクと SSD はコミットプロトコルのレベルで同型であり、D12 (対象メディア) の三者論証ラウンドがこれを担保する；zoned の行はそれが少なくとも1つの同値類を追加することを示すのみである。同値関係を記録ストリームのセグメントシーケンスの同型性に従って判定すること、および同値類が幾つ存在するかは D17 (実装階層とサードパーティパイプライン) 確定項目 2 に帰属する。種類文字列によってノードを識別することで十分か否かは、第1ラウンド判定で指摘された（保留、第2, 3ラウンドの攻撃待ち）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「セルごとの判定」の節。
⚠️ **これが固定するのは「どのようなステップが存在するか」のみであり、「各ステップの失敗原子性の幅」を固定することはできない** ——
永続性メモリやオブジェクトストレージは各ステップの失敗原子性の幅を変更するが、列挙では対応できない（ギャップと補完策は D17 (実装階層とサードパーティパイプライン) 参照）。
`RotationalPtr` / `ZonedPtr` という newtype による隔離は `crates/` 内に存在しない（`grep -rn "RotationalPtr\|ZonedPtr" crates` でヒット 0）。

**根拠**：
- 実験なし：N が何であるかは `.claude/rules/fs-design.md` の2つのトップレベルルールから導出され、測定可能な量はない；同値関係をどう判定するか、幾つ存在するかに関する根拠は D17 (実装階層とサードパーティパイプライン) 確定項目 2 にある。
- 三者論証：通常層サンプリングのラウンド（「三者論証 + 実測検証」）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：C8 (ゲート適用範囲の判定不能)。

#### 確定項目 11：ignore ノードをいつ開放するか：未済の検査によるローテーション台帳を否決し、3条を残す

**確定事項**：

**「未済の検査によるローテーション台帳」スケジューリング：否決。** スロット = `(レイアウト, サブフェーズバケット)` とし、未済の検査の降順でキューの先頭を取得し、任意のスロットが高々 `K_max = ⌈N×M/C⌉` 回のコミットで必ずカバーされると主張したスケジューリングは否決された：それが保証できないものこそがまさに防ぐべき対象 —— 低速で稀な分岐 —— であったためである（実測検証および他の4つの否決理由は変更履歴にある）。

**残された3条**：

1. **方向性の正しさ**：確率的な論証ではなく、**決定論的であり、ゲートで赤と判定可能なカバレッジ記録**を求める ——
   長期的な保守において、確率的な保証は保証が存在しないことに等しい。

<!-- doc-lint:registry name-col=2 -->

| 記号 | 略称 | 意味 |
|---|---|---|
| T1 | 通常層 実時間予算 | 通常層のローカル実時間予算、出力ではなく入力 |

2. **`T1` (通常層 実時間予算) は入力であり、出力ではない**（`.claude/rules/fs-design.md`：ゲートの実行時間自体が設計制約である）。
   正しい順序は「まず予算を宣言する → それを用いて1回のコミットで開放可能な ignore ノード数を制約する」であり、「所要時間を測定する → 開放可能な数を導出する」ではない。
   **この条項はいかなる測定も必要とせず、今すぐ確定すべきである。**
3. **判別力はカバレッジと分離して検査しなければならない**：各 ignore ノードが開放されたコミットにおいて、「既知の壊れたイメージ」に対してそのノード自身の checker を追加で実行し、
   エラーを報告しなければ赤と判定する —— これは D13 (検証ルート) に既存の陽性対照規律の直接の再利用である。C13 (checker 判定の無効化) 参照。

ゲート内には「通常層 クラッシュポイント再生」というステージが存在せず、`T1` (通常層 実時間予算) / `C` / スロットごとの所要時間は測定できない。
⚠️ **したがって `C`、`K_max`、`T1` (通常層 実時間予算) の3つの数値はいずれも現時点で kb に書き込むことを禁ずる。**

**射程**：ignore ノードをいつ開放するかのみを管轄する；スモーク層の全数列挙には `T1` (通常層 実時間予算) は不要である。第2条の「今すぐ確定すべきである」と末尾の「現時点で kb に書き込むことを禁ずる」は矛盾しており、[verification-build.md](../verification-build.md)「未決確定事項」第6問に記録されているが未判定である。ノード登録表には状態フィールド（開放 / ignore / 実装待ち）を持たせ、さらに3条を追加する —— 各 ignore 行は変更対象となるオブジェクトを1つ指名し、そのオブジェクトが緑になったにもかかわらずノードが依然として ignore である場合に赤と判定、カバレッジレポートに ignore の状態数と割合を列挙：これら3条はゼロラウンド攻撃されており（`research/prompts/m3-prune-gpu-r1-main-verification.md`「セルごとの判定」の節）、確定事項とはせず第2, 3ラウンドの攻撃を待つ。

**根拠**：
- 実験なし：台帳の否決はスケジューリング規則のシミュレーション検証（200ラウンド、後半の定常状態最大値を取得、厳密なプレフィックス貪欲法、3行の数値は変更履歴にある）に基づき、実験としては登録されていない；残された3条はゲートの設計原則であり、測定可能な量はない。
- 三者論証：通常層サンプリングのラウンド（「三者論証 + 実測検証」）、資料は保存されていない、経過は変更履歴にある。
- ユーザー決定：「列挙には ignore マークを設定し、徐々に開放できる」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第3節「ユーザー決定：ignore マークを徐々に開放」の行）；第1ラウンド判定 `research/prompts/m3-prune-gpu-r1-main-verification.md`「決定の振り返り」の節：確定項目 11 を破棄、「セルごとの判定」の節でノード登録表に状態フィールドを持たせる形態と3条の攻撃待ち補足条件を提示。

**未済の検査**：C8 (ゲート適用範囲の判定不能)；C13 (checker 判定の無効化)。

#### 確定項目 12：衝突 1：checker 即仕様 と 検証の2経路間でツール共有を禁止する の衝突

**確定事項**：**衝突 1：`checker 即仕様` と `検証の2つの経路間でツール共有を禁止する` が直接衝突する。**
使用中の不変条件（件数の公式な登録場所は [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) のファイル先頭にある）が checker のソースコードであり形式検証の spec でもある場合、単一障害点となってしまう。
**実行可能な妥協点であり、本プロジェクトがすでに投資済みの解決策**：checker は2つの方向を実装する —— **トラバーサル型**（システム構成から順に辿る）
と**スキャン型**（ディスク全体を走査し、「メタデータブロックは自己記述的でなければならない」というブロックヘッダに基づいて再構築する）。双方は同一の不変条件に対して独立に判定を行い、
共有するのはフォーマット解析のみとする。

**射程**：checker 内部の2つの方向の間で何を共有するかを管轄する；checker と実装の間で何を共有するかは確定項目 5 に帰属する。プールレベル checker は双方の方向を備えている（`crates/singlefs-checker/src/walk.rs` のファイル先頭）。

**根拠**：
- 実験なし：これは「checker 即仕様」と検証経路の独立性の間のトレードオフであり、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：C410 (checker の2方向での共有事項に関する検査不在)：2つの方向が「共有するのはフォーマット解析のみ」という条項に対して、赤になり得る検査が存在しない（`.claude/gate.d/` 内にこれを判定するステージがない）。

#### 確定項目 13：衝突 2：crash refinement の spec とモデル突き合わせの model

**確定事項**：**衝突 2：crash refinement の spec とモデル突き合わせの model は同一のものの2通りの記述である。**
1つに統合する → 工数を半減できるが、**独立性の要件に直接違反する**；各自で記述する → 名目上は独立だが、
通常は同一人物が同一週に書くことになる。緩和策（相関を低減するのみで排除はしない）：両者を**表現形式のレベルから異ならせる** ——
spec は「イメージに対する状態述語」として記述し、model は「op → 状態遷移」として記述する。不一致が生じた際に少なくとも相互の位置特定が可能となる。
**同源性は観測されなければならない**：「spec と model が不一致」となったイベント数および各判定でどちらが誤っていたかを統計する；
不一致が累計 ≥ 5 回で「spec の誤り」と判定された回数が 0 回であった場合、両者が実質的に同源であることを示しており、2本の経路を統合して予算を全数探索へ移管すべきである。
判定は3つの枠に分かれる：spec の誤り / model の誤り / 条項の記述不足（双方が条項のそれぞれ異なる解釈に従っており、条項にどちらが正しいかが書かれていない）。不一致が発生した場合、まず前提が成立するドメインで再判定し、依然として存在する不一致のみを記録する；「条項の記述不足」イベントはまず未済として記録し、「累計 ≥ 5 回」にも「spec の誤り」にも算入しない。条項に該当する記述が補完された後、補完された文に従って再度判定し、先の2つの枠のいずれかに帰属させる。
ここでの model は O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) を指す。O3 (独立規約実行器) が同一の (履歴, クラッシュポイント) に対して判定行を出力できるようになるまで（Python 製実行器の第1版 E170 (O3仕様実行器第1版) は存在するがクラッシュポイント再生に未接続、C447 (規約実行器がクラッシュポイント再生および RefFS 比較に未接続)）、O3 (独立規約実行器) × RefFS のペアは空欄として登録し、イベントとしてカウントしない；O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) のクラッシュ判定官を身代わりとしない；未確立のペアは「未確立」と報告し、緑とは報告せず、いかなる枠にも算入しない。

**射程**：緩和策は相関を低減するのみで排除はしない。確定項目 1 で撤回された旧候補はこの条項の影響を受けない、理由は確定項目 1 の射程にある。crash refinement のコードは `crates/singlefs-replay-prefix/src/lib.rs` 内の Verus コアとその spec（プレフィックス取得、ルート選択、リングスキャンの3区間と、ルート選択およびプレフィックス取得の結合判定の系であり、確定項目 1 の M1、M2 構文層および M3 の両解釈と単調性の系をカバーする；M2 セマンティクス層および M4 は E165 (リカバリ再生の有界全数探索とミューテーション殺傷限界) に依拠して有界内で判定する）；spec は具体的な状態に対して評価を行うことができず、「spec と model の不一致」イベントがどこから生じるかは未定であり、カウンタにはコードが存在しない（C191 (同源性カウンタが1ペアのみ対象)）。

**根拠**：
- 実験なし：同源性は将来の spec と model の不一致イベントのカウントによって観測されるべきであるが、spec もカウンタもまだ存在せず、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。
- 第3の枠「条項の記述不足」：三者判定 `research/prompts/m3-formal-r1-main-verification.md` 第2節「F2 同源性」の行（攻撃側が指摘、推論）、`research/prompts/m3-formal-r2-main-verification.md` 第2節第9行、`research/prompts/m3-formal-r3-main-verification.md` 第2節同源性カウンタの行、3ラウンド耐えた；実験なし：カウンタには依然としてイベントソースがない。
- 「model は O1 (インメモリモデル突き合わせ (HashMap 理想ファイルシステム)) を指す」および未確立のペアを空欄として登録：三者判定 `research/prompts/m3-formal-r5-main-verification.md` 第2節同源性カウンタのセル（両経路が同一方向を向いた部分）；`research/prompts/m3-formal-r4-main-verification.md`「ユーザー決定」内の身代わりペアの行（身代わりペアとはみなさない）。

**未済の検査**：C191 (同源性カウンタが1ペアのみ対象)。

#### 確定項目 14：他の決定との連鎖

**確定事項**：

| 決定 | 連鎖点 |
|---|---|
| D4 (チェックサムの位置) | ルートチェーン差分が Merkle を「支払うべき代償」から検証のてこへと反転させる；ただし形式検証においてハッシュは uninterpreted function であり、**衝突耐性は検証範囲の完全に外側にあり、「Merkle を検証した」と解釈してはならない** |
| D8 (コアインデックス構造) | ログ構造ノード、write buffer、key cache の3層はいずれもシンボリック実行を拘束し（ループ上限、遅延ウィンドウ、キャッシュ一貫性）、「Yggdrasil は1分以内」という計測条件を本プロジェクトにおいて成立させない |
| D9 (暗号化) | AEAD は SMT で解決不能であり、uninterpreted function としてのみモデル化できる；**「nonce の再利用による平文漏洩」という結果は検証できず**、残るのは「nonce が重複しない」という純粋な組合せ的性質のみである —— そしてこの純粋な組合せ的性質こそが、この検証経路の最も価値ある成果物である |
| D11 (インデックスノードにメッセージバッファを残すか) | 確定項目 1 で「残す」を選択；D8 (コアインデックス構造) 確定項目 1 が冪等完全値を選択した後は、checker の I-3.1 (割り当て済み統計の整合) はメッセージセマンティクスを解析する必要がなくなり（D11 (インデックスノードにメッセージバッファを残すか) はこれに基づきコスト 1 が本プロジェクトで成立しないと判定）、衝突 1 がこれによって「解決不能」になることはない |

**射程**：連鎖するのは形式検証と checker がこれら幾つかの決定に直面した際の境界であり、それらの決定の条項自体を変更するものではない；ルートチェーン差分および衝突 1 自体は確定項目 8、12 に帰属する。

**根拠**：
- 実験なし：連鎖はそれら幾つかの決定の確定条項から演繹された境界であり、測定可能な量はない。
- 三者論証：D13 (検証ルート) 新設時のラウンド（ローカル LLM / Sonnet / Opus が各1経路、立場重複なし）、資料は保存されていない、経過は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 15：検証コードを2層に分割 —— harness 層は常時実行、checker 層は既定でコミット時のみ実行

**確定事項**：`crates/` 以下の検証コードは「いつ実行するか」に基づいて2層に分割し、各々1つのパッケージに配置する。プールレベル checker と合わせて計3つの名称となる：**harness 層**は `crates/singlefs-harness` であり、単体テストおよび結合テストに加えてそれらが使用するスキャフォールドライブラリ（レコーダー、インメモリプール、理想モデル、ランダム履歴、故障注入デバイス、シナリオ）を含み、コード変更時に随時実行する；パッケージ内はさらに軽量と重量に分かれる：debug ビルドで単一テストの実行が 60 秒以上のものは重量とし、`#[ignore]` を付与する。単一テストの所要時間は通常の実行時に測定されたものを参照し、別途個別に測定はしない。実行したいときは随時実行可能であり、一律にメモリラッパー経由とする；1テストケース1シナリオとし、テストファイル名はテスト対象に基づいて命名しマイルストーン名には基づかない。**checker 層**は `crates/singlefs-checker-tier` であり、クラッシュ状態列挙エンジンとレコード照合器、クラッシュ列挙ケースの判定ライブラリ経由の2段階とそのカウント、検出ログとツールチェーン識別情報、1つの長期判定ライブラリ（ブロックキーは内容のみによって決定され、各種テストが各々どのブロックを必要とするかはインデックスとしてのみ機能）と2段階プロトコル、クラッシュ注入と破損ディスク入力の2つの戦役、実デバイス側でのデバイスログ比較、実験および実デバイスのハーネスバイナリ、ならびにそれらの全テストケースを含み、QEMU（ゲート checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates ミューテーション全表（checker-tier-crates-mutation-replay）、全実験再実行（checker-tier-research-build-and-replay の experiment-replay セル）を伴う：既定ではコミット時のみ実行し、単独での実行も可能（コマンドに `SINGLEFS_HEAVY_TESTS=user-request` を付与）。checker 層内のクラッシュポイント再生（ゲート checker-tier-crash-point-replay）は2層に分かれ、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay は分割しない：**クイック層**は2回実行する：`cargo test --release -p singlefs-checker-tier --lib --tests`（`--ignored` なし、`--tests` はすべての `test = true` のターゲットを含み、ハーネスバイナリのインライン単体テストもこれに含まれる）、およびもう1回は `tests/` 内の `cfg` で指定されたフィーチャを有効化して実行（`--features <フィーチャ> --lib --test <フィーチャ付きテストファイル>…`、フィーチャに `gpu` を含む実行は `scripts/crash/gpu-quiesce-run.sh` 経由で起動）（ゲート checker-tier-crash-point-replay はゲート全体の実行時およびコミット時の双方でこの2回を実行し、2回目はその feature-gated-tests セルである）、**フル実行**は `.claude/gate.d/stage-inputs.tsv` にキー `crash-case:` として登録されたテストケースを1件ずつ `--include-ignored --exact` で実行（checker-tier-crash-point-replay `--full`）、コミット時には既定で実行せずユーザー要求時に実行し、実行方法は確定項目 9 の階層化に従う；クラッシュ増幅判定器はシステム全体が GPU に移行しており、すべてのプールレベル checker 不変条件をカバーする（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`）。**プールレベル checker** は `crates/singlefs-checker` であり、判定器ライブラリとして機能し、上記2層から呼び出される。依存関係は一方向のみ許容される：checker 層は harness 層とプールレベル checker に依存し、harness 層は checker 層に依存しない（ゲート checker-independence-and-sync の checker-implementation-disjoint セル第 ④ 項が判定、dev-dependencies も含む）；双方の層が必要とする短いコードは各々個別に保持することを厭わない。クラッシュ列挙ケースは一律に `crates/singlefs-checker-tier/tests/` 内に記述する（`scripts/crash/crash-case-check.py` が判定）。ヘビー層テストの関門はパッケージ単位で判定する：`singlefs-checker-tier` に到達するテストが checker 層である。

**射程**：`crates/` 以下のコードとテストがどのパッケージに属するか、いつ実行するか、ゲートが何に基づいて識別するか、3つのパッケージをどう呼称するか（`.claude/rules/verification.md`「定義と名前」）を管轄する；各不変条件が何を判定するか（[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)）、列挙ドメインをどう定義するか（確定項目 4）、プールレベル checker と実装の間で何を共有するか（確定項目 5：プールレベル checker は依然として `singlefs-format` にのみ依存する）は関知しない。アップストリームの SOP には本プロジェクトの検証手段をこれ以上記載しない：クラッシュポイント再生、モデル突き合わせ、checker 即仕様、ファイルシステム特有の逆推論ギャップの4節および `gate.sh` の未実装リスト内のプロジェクトキーはすべて本リポジトリに集約された（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`）。アップストリーム側の変更はレプリカに同期済みであり、未リリースである。分割ラウンドの記録は `records/2026-09-27-验证两档拆分.md` にある。

**根拠**：
- 実験なし：いつ実行するか、どのパッケージに属するかはプロセスポリシーであり、測定可能な量はない；2層それぞれの判別力はそれぞれのテストケースおよび変異表によって証明される。
- ユーザー決定（2026-09-27）、原文は変更履歴にある；三者 `verification-split-r1`（判定 `research/prompts/verification-split-r1-main-verification.md`）は `singlefs-checker-tier` 分割前のバージョンを攻撃したものであり、パッケージ分割後の変更はゼロラウンド攻撃されている。
- ユーザー決定（2026-10-01）：フル実行は時刻を制限せず、ユーザー要求時に随時実行、「夜間」を削除、原文は変更履歴にある。

**未済の検査**：なし。

#### 確定項目 16：セグメント単位で登録されたセグメントの全領域が u64 に収まらなければスライス判定し、状態数には 1280 ビット固定幅整数を用いる

**確定事項**：
- スライス分割：クラッシュ増幅においてセグメント単位で登録されたセグメント（`.claude/gate.d/crash-point-replay-segments.tsv`、1行1セグメント5列：項目名、ストリーム名、セグメント番号、セグメント書き込み数、セグメント書き込み表ダイジェスト）について、全領域の状態数が u64 に収まらない場合は固定サイズのスライスごとに順次判定する；スライスサイズは環境変数 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` で定義され、未設定時は 2^32 状態とする；スライスの起点は判定ライブラリ内の再開点とする（再開点はセグメント書き込み表ダイジェストと判定器ユニットキーによってアドレス指定され、項目名、ストリーム名、セグメント番号は表示用に過ぎない）。全領域が u64 に収まる登録セグメントはスライス分割せず、全領域を一度に判定する。コード配置先：`crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- ビット幅：登録セグメントの総状態数、スライス起点、スライス末尾、および進捗には一律に 1280 ビット（10 個の u128）固定幅符号なし整数 `StateCount1280Bits`（`crates/singlefs-checker-tier/src/crash.rs:1640`、手足の数は同ファイルの `STATE_COUNT_LIMBS`）を用い、演算はすべて checked とし、飽和や切り捨ては行わない；ドライバ行 `registered_segment_progress=` の判定済み状態数および総状態数はすべて正確な10進数で記述し、指数の形式では記述しない。
- 登録表をエンジンに渡す方法：エンジンは環境変数 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 経由でのみセグメント登録表を読み込み（`crates/singlefs-checker-tier/src/crash_amplification.rs:14764`、`fn registered_segments_of_the_flow_from_the_environment` 内）、未設定時は1セグメントも登録されていないものとして扱い、他の逃げ道は設けない。144 ユニットのファイル順次書き込みが 145 個となり extent 下段が2層に成長したストリーム（エンジンストリーム名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）のユニット書き込みセグメント（322 回のユニット書き込み）およびレコードセグメント（290 回の journal レコードインプレース書き込み）の双方が登録されている。

**射程**：クラッシュ増幅エンジンがセグメント登録されたセグメントをどうスライス分割するか、状態数をどの整数で記録するか、登録表をどこから読み込むかを管轄する；どのセグメントを登録すべきか（ユーザーが承認し登録表のファイル先頭に記述）、登録セグメントのドメイン自体をどう展開するか（判定 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` の J10）、各クラッシュ状態をどう定義するか（確定項目 4）は関知しない。未登録のセグメントの状態数、およびストリーム全体を合算した状態数は依然として u64 に収まる必要があり、収まらない場合はライブラリ書き込み前に実行を拒絶する；登録セグメントの全領域が 1280 ビットを超えた場合も同様にライブラリ書き込み前に実行を拒絶する。スライスサイズは 1 から 2^32 までのみ許容される：2つの GPU カーネルがスライス内インデックスを u32 としてスライス起点に加算するためである（`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）；ライト層（`scripts/crash/crash-point-replay.sh --quick`）の呼び出し元がスライスサイズを設定していない場合は `LIGHT_REGISTERED_SEGMENT_SLICE_STATES` に設定する。展開上限でカバーできず未登録のセグメントがある場合、増幅エントリ `scripts/crash/crash-amplification.sh` は赤と判定する；クラッシュポイント再生エントリ、増幅エントリ、および項目ドライバは実行開始前に本テーブルを照合した上でエンジンに渡す。

**根拠**：
- E161 (クラッシュ増幅の重複排除とセグメント分割所要時間)「第2回再実行：サンプリングドメインを11セグメントのストリームに従って再定義（2026-10-05）」の節：セグメントのフル実行状態数と閉じた式、ハーネスによる個別の数え上げ、crates の計画がセグメントごとに等しく、状態数はセグメント書き込み数に伴い指数関数的に増加するが、これは「長いセグメントの全領域は u64 に収まらない」という前提を支持するに過ぎない；スライスサイズ、ビット幅の値、登録表の読み出し方法はエンジニアリングパラメータでありユーザーが決定したものであって、本実験が支持するものではない。
- ユーザー決定、原文は変更履歴にある：スライス分割と1スライスあたり 2^32 状態（`records/2026-10-11-崩溃放量两段与脚本审计.md` 96行目）、ビット幅 1280 ビット（同記録 98, 99行目）、両セグメントの登録（同記録 92行目）。
- スライス分割とビット幅は判定 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` の J10（セグメントごとの登録）に対する補足であり、三者による攻撃は経ていない。

**未済の検査**：なし。

#### 確定項目 17：判定再利用のユニットは関数であり、キーは関数ユニット、ツールチェーン、依存バージョンのみで構成する

**確定事項**：
- 重型ゲート、クラッシュ判定ライブラリ、および実験受入における各判定（クラッシュ状態ブロック、クイック層およびモデル突き合わせの各テスト関数、crates ミューテーションの各行、広域ミューテーションの各行、形式検証の各条、QEMU の各階層、litmus の各項目、研究再実行の各行、実験ミューテーション表の各条、実験受入）を踏襲するか否かは、そのユニットキーのみを参照する：それが依存する関数ユニットの各コメント除去後指紋、クロージャ内の各ユニット（mod ヘッダを含む）の `#[cfg]`、`#[cfg_attr]`、`cfg!` が実際に参照する cfg アトムに今回のターゲット下での真理値を付加したもの（クロージャが参照しないアトムはキーに含まれない：`gpu` フィーチャの有無でコンパイルされた2回の実行において、クロージャが `gpu` に言及しない判定キーは同一となり、ライブラリを共有する）、ツールチェーン（cargo 解決後の有効ビルド環境および外部ツールバージョンを含む）、`Cargo.lock` から解決されたクロージャが実際に参照する依存バージョン、ならびにその判定自体の内容入力（セグメント書き込み表と開始イメージのダイジェスト、litmus の内容、ミューテーション原文と置換文など）を加える。関数ユニットはその判定のエントリから呼び出しと参照を辿って収集し、参照される定数、型、静的項目、マクロ展開を合わせて計算する；wgsl は fn 単位で区切る。
- 一律にキーに含めず、再利用の根拠とすることも禁ずるもの：コミット、HEAD、ツリーに基づく変更範囲；ファイル全体の原文；ファイル単位、パッケージ単位、テーブル全体の指紋またはクロージャ；手書きのバージョン定数；パス（リポジトリルートも含む）、名前、シード文字列、行番号、リソース上限、タイムアウト；全工程またはセル全体のマークのみで判定ごとのライブラリを持たないもの。キーが狭すぎる（参照される定数、型、マクロの収集漏れ）、ライブラリが失われやすい場所に置かれている、改名によりライブラリが消失する、2つのライブラリで同一バッチを重複判定することも同様に違反とみなす。
- 完全に収集できないもの（動的ディスパッチ、マクロ、`include_str!(concat!…)` などで認識不能なもの）はその判定のキー計算を拒否してそのまま実行し、暗黙のうちに収集を省略することはしない；各キーはそれを構成するユニット一覧とともにディスクに保存され、無効化された際にはどのユニットが変更されたかを報告できるようにする。
- 判定の種類ごとに1つのライブラリを永続ディレクトリに固定し、1回の実行ごとや判定手法ごとにディレクトリを変更しない；リポジトリルートもライブラリの入力ではない：クラッシュ判定ライブラリのディレクトリが環境変数 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` で指定される際は絶対パスのみを受理し、相対パスは拒絶する；フル実行スイッチは書き込みのみで読み出さず、オンデマンドで判定されたものはキーに従って同一のライブラリに書き戻す；二台目で判定された結果はローカルのものにマージされる。
- キーの変更により古いライブラリが無効化されることはコストとはみなさない：互換マイグレーションは行わず、古い行への追記署名も行わず、変更後は再実行する。

**射程**：重型ゲート、クラッシュ判定ライブラリ、および実験受入の再利用キーを管轄する；静的ゲートの「新規追加行のみ判定」の変更範囲（`scripts/gate/changed-paths.sh`、これは判定基準のセマンティクスである）や preflight の `admission: inputs-changed` は関知しない；列挙ドメイン（確定項目 4）、階層化、およびいつユーザーにフル実行を促すか（確定項目 9）は関知しない。イメージ判定および特定状態判定のバージョンは、判定定義セット（CPU 参照判定器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` のクロージャ）のキーであり、そのブロックが CPU 上で判定されたか GPU 上で判定されたかを問わない：GPU 判定器は加速経路であり、その受入は定義セットとの突き合わせ（クイック層の GPU と CPU の突き合わせテストケース）に依拠し、そのユニットはブロック属性に出典としてのみ入り、キーには入らず、種別も分けない；GPU カードのドライバ、カード型、複数枚のカードの分担もブロック属性や実行時パラメータとして残し、キーには含めない。

**根拠**：
- ユーザー決定（2026-10-15）、原文は変更履歴および `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 ユーザー決定：コミットは再実行と無関係、再実行はコード変更のみに従う」の節、「2026-10-15 バッチ3」の各段にある。
- ユーザー決定（2026-10-17）：判定はハードウェアや CPU / GPU とバインドせず、再利用と枝刈りはデバイスを参照しない；原文は変更履歴および `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 のフル実行停止に関する各段にある。
- メイン agent 決定（2026-10-15）：リポジトリルートはキーやライブラリに含めず、ライブラリディレクトリは絶対パスのみを受理、cfg 成分はクロージャが実際に参照するアトムのみを取得、出所は同記録「2026-10-15 検証フェーズ第1ステップ赤判定、2条の決定、4-3 4バッチ派遣」の決定第1, 2項。
- 反証：E164 (クラッシュ増幅新設計の小規模機能テスト) F4 において、関数のみに基づいて指紋を取得し参照される定数を収集しなかった結果、V5 と V11（V11 は定数 `DATA_POINTER_BYTES` のみを変更）の判定を漏洩したことが測定された：関数ユニットは参照される定数、型、静的項目、マクロとともに計算しなければならない。
- ディスク上の数値：`ledger.md`「1の2、ディスク上で測定されたもの」の節 —— 名前のみが異なるブロックが個別に保存・判定されたものが 4100 ブロックおよび 9679 ブロック；ライブラリ全体で一括の判定法ダイジェストを変更した際に累積で 3 935 242 934 状態の追加判定が発生したが、そのクロージャ内で変更された 29 ファイルのうち 24 ファイルはコメント除去後に逐語的に同一であった：キーに名前を含めたり、ファイル単位やバージョン全体で指紋を取得すると、同一の判定が重複して再実行されてしまう。
- 経路状況記録なし：三者による攻撃は経ていない；ユーザーはコード改修後に3ラウンドの攻撃側を開始し、「枝刈りがアトミックであるか、キーが関数ユニットのみで構成されているか」のみを攻撃対象と決定した。

**未済の検査**：C8 (ゲート適用範囲の判定不能)、C489 (singlefs-core の広域ミューテーションが今日のコードに対して結論を持たない)、C528 (2つのゲート経路で判定される変更ウィンドウが異なる)、C564 (残る6本のクラッシュポイント再生フル実行が誰も走らせていない)、C605 (クラッシュ増幅の再利用と枝刈りで無効化して再判定する対照がない)、C630 (コミット時の形式検証ステップ1で停止する頻度が未計測)、C635 (前回のフル実行に機械可読なマーカーがない)、C645 (クラッシュ増幅フル実行を誰もディスパッチできない)、C777 (5つの実験キーが stage-inputs.tsv に登録行を持たない)、C836 (staged-green に紐づくツリーが判定されたツリーとは限らない)。

```

**出处 `.claude/kb/decisions/13-検証ルート.ja.md:448-451`（整段抄，未转述）**

```markdown
## 履歴バージョン

D13 (検証ルート) の履歴エントリは [decisions-history.md](../decisions-history.md) に集約されている。

```

**出处 `.claude/kb/decisions/13-验证路线.md:1-6`（整段抄，未转述）**

```markdown
## D13 验证路线 —— 已定

**中文** · [English](13-verification-route.en.md) · [日本語](13-検証ルート.ja.md)

D13（验证路线） 管本工程拿什么验正确性：三类现成办法全做——自写参照实现做差分对拍、有界穷举崩溃测试、crash refinement 形式验证（调查见 [prior-art.md](../prior-art.md) 7.4）——并且不照抄，针对本工程特有的四样（COW 每事务发新根、Merkle 自验证、整卷 AEAD 的 nonce 唯一性、没有参照实现）自己设计验证手段。它定 oracle 怎么分、自研哪两样、崩溃点重放怎么分层与抽样、崩溃状态集合怎么定义、checker 与实现共享什么。不管的：每条不变量写什么在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)；三样验证手段怎么落地、谁挡着谁在 [verification-build.md](../verification-build.md)；提交步骤登记在哪、结构等价类怎么分、有几个，归 D17（实现分层与第三方管道） 已定项 2。

```

**出处 `.claude/kb/decisions/13-验证路线.md:29-90`（整段抄，未转述）**

```markdown
#### 已定项 1：crash refinement 先验「一次发布 + 一次崩溃 + 一次恢复」

**定案**：crash refinement 的第一个验证对象是恢复函数 `Recover` 的关系性质，最小封闭单元是「一次发布 + 一次崩溃 + 一次恢复」。四个里程碑的主语 `Recover` 指 `crates/singlefs-core` 里以 `verify_named_units = true` 调 `replay_journal` 的三处（`recover` 的 `Consult` 策略、可写挂载读阶段、只读挂载），三处的 `records` 都取 `scan_journal` 的输出；`Ignore` 与 `ConsultWithoutNamedVerification` 是测试与对照用的臂，写进可信部分；`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` 不是产品调用方。四个里程碑依次是：

| 里程碑 | 义务 | 落点 |
|---|---|---|
| **M1 全性** | `Recover` 在 D13（验证路线） 已定项 4 枚举的**每一个**崩溃状态上有定义、终止、输出一个能过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像 | 脚手架。**完成判据不是「证出来了」，是「M2 能在它上面写出来」**——崩溃点重放本就全量枚举（[verification-build.md](../verification-build.md)「崩溃点重放不抽样、全量枚举」），恢复卡住当场暴露，单独证它没有边际价值。取前缀、择根、扫环三截已进 Verus，都在 `crates/singlefs-replay-prefix/src/lib.rs`：取前缀核 `replay_core`（终止与不溢出在核内证到，循环都带 `decreases`；`requires` 两条——记录表按 (实例代号, 计数器) 升序、计数器有后继——由胶水 `replay_journal` 里的断言兜），择根核 `choose_root_slot`（自证过的槽里取 (`checkpoint_txg`, 实例代号) 最大、键相同取槽序最前），扫环核 `first_copy_of_each_record_key`（同一 (实例代号, 计数器) 先到先得），择根与取前缀的合判写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`；门禁 checker-tier-formal-verification 判证明，登记数 56 verified（`.claude/gate.d/verus-pins.tsv`）。E165（恢复重放有界穷举与变异杀伤界） 当轻量工具，它第 3 次重跑第二段的择根合判族在内存盘上调真 `choose_root`、`scan_journal`、`replay_journal`。择根序的四个改坏变体在合法历史 K 族里只有「取第一个有效者」红，另三种（比较不看实例代号、txg 平局取低、实例代号为主）由 Verus 必红世界（`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 择根那一节）与对抗根环 KA 罩（第 3 次重跑第三段）。核之外：择系统配置（`choose_system_configuration`）与两截胶水（择根的读槽、自证校验和、fsid、区域到盘的映射、槽的遍历次序；扫环的偏移枚举、提示、解析与 CRC、盘的遍历次序）不在 Verus 里，崩溃点重放与崩溃放量的 CPU 判器在每个崩溃状态上调 `recover` 罩它们（GPU 判器用 WGSL 另写一份 `replay_journal`，验不到 core） |
| **M2 前缀不切发布** | **任一 `jsn` 连续前缀，其重放结果等于某个整数个已发布的 checkpoint 的结果——不存在被前缀切开的发布；前缀停在一次发布中间时，那次发布整体不施加**（D16（发布语义） 已定项 4「一次发布整体施加或整体不施加」）。分两层。**句法层**（对任意自证过的记录表，无前提）：施加的记录是若干整次发布，在飞上限与提交标记逐条判过，施加的计数器从链首起逐一连续（锚点读得出时从锚点加一起；读不出时首条 `checkpoint_txg` = 所选根 + 1、序号 1）；反向链与点名单元是读盘回调的结果，写成「被施加的每条记录，回调交回了真」。**语义层**（前提：盘上每条记录出自这一版的合法写侧，即 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族；同一 (实例代号, 计数器) 在全部时间线上只对应一份内容）：「结果」是根的投影加盘，不是整条根——返回的根的 (实例代号, `checkpoint_txg`) 与新根段四个字段等于所选根那条时间线上的第 j 版（j 不早于所选根），其余字段等于所选根，卸载记号除外（由记录重建的根取「不是卸载那一串写下的」）；盘上 (所选根, j] 每一版的记录读得出、点名单元两份都验过、记录条数之和不超在飞上限 | 落点是 I-8.3（重放前缀严格连续）——它的判据补句逐字就是这一条（「前缀不得停在一次发布的中间——一次发布整体施加或整体不施加」），不要在 [invariants/03-journal.md](../../../crates/singlefs-checker/invariants/03-journal.md) 里另立一条。句法层在 Verus 核 `replay_core` 的 `ensures`（`replay_follows_the_prefix_rules`）里，必红世界登记在 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`，其中「改判定」行只改可执行判定、不碰 spec；语义层靠 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族在界内判，Verus 不证。核之外：胶水（摊表、包回调、拼报告与新根）由 E165（恢复重放有界穷举与变异杀伤界） 直接调真 `replay_journal`、harness 用例与 `crates/mutations.tsv` 的变异行罩，拼根里投影之外的字段（卸载记号、实例表指针、分配记录树根）没有变异行、E165（恢复重放有界穷举与变异杀伤界） 的生成器把它们写成定值；两个读盘回调（反向链、点名单元闭包）由 E165（恢复重放有界穷举与变异杀伤界） 里真回调、内存盘的读数、`crates/mutations.tsv` 的变异行与只读挂载几种形状的读盘次数用例罩 |
| **M3 幂等与单调** | **幂等**：`Recover(Recover(c)) = Recover(c)`，两读都证——记录级（同一条记录施加两次与施加一次，得到的根相同，I-8.4（重放幂等） 的字面）与不动点（把重建出的根当所选根、同一张记录表、同一个读者、同一个开关再重放一次，返回的根不变；前提是所选根之上的记录不超在飞上限，且记录表满足 I-8.12（同实例记录的 txg 不减）：同一实例里 journal 记录的 txg 随计数器不减）；第二次因所选根 `checkpoint_txg` 顶格报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` 且一条没施加的，算相等。「就是 I-8.4（重放幂等）」不成立：I-8.4（重放幂等） 是记录级一读，不动点是另一条性质。**单调**（M2 的推论，不单独证）：串行提交（已定项 2）、D16（发布语义） 已定项 7 的持久顺序、所选根取根环里最新的有效根之下，同一次运行里 D13（验证路线） 已定项 4 的两个不带故障的崩溃状态 `c ⊆ c'`，`Recover(c)` 落到的版本不晚于 `Recover(c')` 落到的版本，版本按 (`checkpoint_txg`, 实例代号) 比，与 D22（单元原子性怎么合成） 已定项 7 的择新序同一个序；开并行提交或把域放宽到带故障时重开。「施加的事务集包含于」那一读法撤：它对理想恢复也为假 | Verus 核 `replay_core` 的 ensures 证两读：记录级与不动点（不动点的前件含 I-8.12（同实例记录的 txg 不减））；单调写成推论 `corollary_the_landed_version_does_not_go_back`，择根与写侧那两条前提写在它的 requires 里。E165（恢复重放有界穷举与变异杀伤界） 的对抗一族在界内判不动点，违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上（第 3 次重跑第一段）。checker 那一半（I-8.4（重放幂等））已实现，取在崩溃注入的二次崩溃上判、不进池级 checker 的清单：`crates/singlefs-checker-tier/src/crash_injection.rs` 的 `REPLAY_IDEMPOTENCE_INVARIANT` 与 `judge_the_remount_after_a_second_crash`。O2（独立解析器 + checker） 判不了幂等：I-4.5（已停用·需两镜像） 退役理由逐字「幂等要比对两个镜像，单镜像判不了」 |
| **M4 不少施加** | 前提与对象同 M2 的语义层。`Recover` 返回的根恰是 j\*：从所选根起每一版都完整在盘上（记录任一份自证过、点名单元两份都验过）、记录条数之和不超在飞上限的最远一版。不加这一条，M1–M3 全部证出来也排除不了「已确认的完整发布被恢复丢掉」：一条都不施加的恢复过 M1–M3 的全部字面。「不少施加」按「多出违例」读：变体多出违例才算被它挡住 | E165（恢复重放有界穷举与变异杀伤界） 的最远一版判据在界内判；Verus 的必红世界里「第一条水位之上的记录就停、一条都不施加」那一行红；M4 自己不单独证 |

**挂载透明**（独立的第四条性质，不是 M3 的第三读）：在崩溃镜像上只读恢复一次得 R1；在同一份镜像上做一个动作——只挂载（取号、写行、暖机、挂载续做与挂载自己推的抬 F），或挂载加一次发布——之后再只读恢复一次得 R2，逐字段比 R1 与 R2。比的那一版：只挂载比恢复落到的那一版，挂载加一次发布比挂载后那次发布之前的那一版。挂载途中三个相位的二次崩溃镜像 img2 上也判：C(recover(img2)) = C(v1)（v1 是 R1 落到的那一版，C 取 POSIX 内容与存在性），内容与字段两半都判；字段一半里，落回 v1 那一版自己的各字段与 R1 相同，落到这次挂载写的某一版的照逐字段表判，扫描报告不判，抬 F 第一个写起 F 不判、在它之前（含暖机）F 照「挂载没推抬 F」那一格判；抬 F 第一个写的位置按挂载交回的抬 F 那几串（准入推的与续做里推的）在挂载写表末尾的位置取，取不出（`NotLocatedInTheMountWrites`）时这一次挂载的二次崩溃整段 F 记不判。逐字段表（R2 对 R1）：

| 字段 | 判法 |
|---|---|
| POSIX 内容与存在性 | 必须相同 |
| 池身份与系统配置的不可变配置（本盘设备号除外） | 必须相同；不可变档 37 行除本盘设备号一律按原字节比 |
| 系统配置的节点大小 | 必须相同 |
| 系统配置的运行配置 36 字节（T_time、T_dirty、三水位） | R2 等于内置默认值（没有挂载选项通道，`to_slot` 总写默认） |
| 卸载记号 | R2 恒为「不是卸载写的」，R1 的记号不进比较 |
| F 与 F 生效值 | 挂载没推抬 F：R2 的 F 生效值与 R2 根里的 F 都等于 R1 的 F 生效值；推了（准入推的与续做里为空间推的都算）：R2 的 F 生效值等于挂载交回的最后一串抬到的 F |
| 实例代号 | max(各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1，两个量都从崩溃镜像读 |
| 实例表行 | 照 D18（块里携带什么信息） 已定项 11 恢复写的行 |
| `checkpoint_txg` | 只判下界：≥ max(根环全部根的 txg, 环里全部自证记录的 `checkpoint_txg`) + 1（D23（journal 的角色与格式） 已定项 14 的下界，允许跳号） |
| 根槽位置 | 不另判，由 `checkpoint_txg` 定 |
| 树 ID 水位 | 挂载不发第一个文件时相同 |
| 树表里 inode 树、extent 树、墓碑、谱系、deleted_inodes 的根指针（后三样是树表条目，种类 9、10、11，不是根记录字段） | 相同；挂载续做过时，按续做之后那一版判：等于挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步的各根指针；树表单元自己的落点不比 |
| 树表种类 6、7、8、12 的根，诞生 txg、头 ID、树 ID、种类 | 必须相同（种类 6、7、8、12 的根恒全零）；诞生 txg 跨根另有 I-9.14（树表条目的诞生 txg 跨根不变），头 ID 另有 I-9.27（树表条目的头 ID 取值），单镜像判 |
| 实例表与树表的单元指针 | 不比字节，与「树表单元自己的落点不比」同 |
| 扫描报告 | 施加 0 条、水位之上 0 条 |
| 回收门槛 t 与钉槽 p（根槽偏移 428..437） | 不比：每次带单元的发布按这个进程的根环与钉现算、零单元发布照抄上一版，挂载做的发布会改它们（与 F、卸载记号同类）；单镜像性质，归池级 checker 的 I-7.18（根记录的回收门槛等于现算的门槛） 与 I-7.19（钉着的槽不住本实例写的根） 判 |
| 根槽偏移 437..466（算法、nonce、MAC） | 必须为 0；单镜像性质，归池级 checker 的 I-2.4（头校验和覆盖范围）判，挂载透明不比 |
| magic、flags 其余位、自证校验和、整槽校验和 | 不比（结构性，读槽时判） |
| 系统配置槽世代号 | 不比，交 I-7.16（系统配置槽世代号与槽号的关系） 在 R2 的镜像上判 |
| 系统配置的 journal_tail | R1 ≤ R2 ≤ R2 环里自证记录的最大计数器，挂载做完取等 |
| 系统配置的 journal 实例代号 | 挂载做完等于 R2 根的实例代号；二次崩溃时取 R1 的实例代号或新实例号两者之一 |
| 分配记录（含根记录的 `mapping_root`、`allocation_record_tree_root` 与树表种类 3、4 的根） | 不比整表：POSIX 内容相同，且池级 checker 对 R2 的镜像判绿；`mapping_root`：R2 全零 ⇔ R1 全零；`allocation_record_tree_root`：R1 带树 ⇒ R2 全零，R1 树表 0 条 ⇒ （R2 非零 ⇔ 新实例号 ≥ 2） |

系统配置的运行量（槽世代号、journal_tail、journal 实例代号）落回 v1 时照它们自己那一行判，不按 v1 相同判。

**射程**：refinement 的边际价值只在崩溃点重放枚举取不到的那几维——载荷字节、记录条数 / 段数 / txg 数（含环回绕）、上一条时间线的残留、槽数与设备数；崩溃点那一维崩溃点重放已全量枚举。「校验和过了」在模型里是一个布尔标志，**不许读成 refinement 验了完整性**——与已定项 14 里 D4（校验和位置） 那一行「不许读成『验了 Merkle』」同形，也与 D13（验证路线） 已定项 4 对 CRC32C 碰撞概率的「知情接受」是同一笔账。D13（验证路线） 已定项 4 是 refinement 左边的 `Crash`（模型层的枚举域），不是右边的 `Spec`，拿它当 spec 会让 refinement 恒真。I-6.5（nonce 水位单调） 的形式化撤下；D18（块里携带什么信息） 已定项 14 定了、且 nonce 分配规则（单调计数器 + 预支水位、水位在使用前持久）被立成一条分项之后，这一格重开——重开时它唯一的真优势是 nonce 对 POSIX 不可见，它的 spec 与 O1（内存模型对拍（HashMap 理想文件系统）） 的 model 结构上不可能同源，不受冲突 2（D13（验证路线） 已定项 13） 影响。

M1–M4 的前提与域：语义层只挂「同一 (实例代号, 计数器) 在全部时间线上只对应一份内容」，由 D23（journal 的角色与格式） 已定项 14「这一版的失败处置」的原样重发担保，根槽写失败推进 `checkpoint_txg` 再发那条路落地时重核；前提域取 E165（恢复重放有界穷举与变异杀伤界） 的合法历史一族（合法写侧写出的历史的任一落盘子集，较新的根槽可以读不出）。M2 的「其余字段等于所选根」里，由记录重建的根的卸载记号是 `NotWrittenByTheUnmountSequence`（D22（单元原子性怎么合成） 已定项 7「其余根写 0」），与所选根的记号可以不同，所以写「卸载记号除外」。M3 的单调推论只在串行提交、不带故障的崩溃状态里成立。M4 连同择根才排除「已确认的」完整发布被恢复丢掉，前提要全：不带故障的崩溃状态（已定项 4 的枚举域）、所选根按 D22（单元原子性怎么合成） 已定项 7 的择根取、链不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1）、新实例本实例写成的根覆盖两块盘之前不让任何 fsync 返回（D16（发布语义） 已定项 8 的暖机）；带故障的域里不排除——在飞上限会截掉记录完整的发布，一块盘整块掉了时点名单元少一份的那一版不施加（C519（丢一整块盘时恢复丢掉刚确认的那一版））。

挂载透明的射程：只管不带故障、可写挂载做成的崩溃状态。可写挂载被拒的状态不判透明，判拒绝本身合不合法：拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立（「读阶段判完之后系统配置又读不出」这一拒因带补写例外：D23（journal 的角色与格式） 已定项 14 的单子上这一拒只在某块盘一份自证过的系统配置都没有、又过不了 D22（单元原子性怎么合成） 已定项 8 的补写闸时成立；判官只复算「一份都没有」那一半，补写闸五件不复算，补写闸放过的盘面挂上了不拒，归崩溃点重放的观察者判（已定项 6）；判槽自证读池级 checker 的逐盘数，不用核心层的解析，实现在实现批），且盘上逐字节不变，且只读读出在允许集合里；拒因不在单子上的照崩溃注入第二截的写法报红。带故障的状态不在射程：取号那一刻有一槽读不出与 N-配置报成同一个拒因，接进带读故障的档之前要分开（C664（挂载拒绝合法复算把两种拒因报成一个））。逐字段表里树 ID 水位、两棵树的根指针、`checkpoint_txg`、扫描报告四格是推的（三方判决取辩方的式子，没量），F 那一格「推了」的一半没量；根记录的 `mapping_root`、`allocation_record_tree_root`，墓碑、谱系、deleted_inodes 三根指针，算法、nonce、MAC，系统配置的槽世代号与 journal_tail 各归哪一类，照 C655（挂载透明逐字段表有字段没归类）、C687（从 mkfs 水位建树那一版落点判不了的几格） 判决 Y1、Y2 写在逐字段表里。

**依据**：
- E42（一事务几条记录）：为了让「被切开 ⇒ 非法」成为恒等式而钉死了载荷，「被切开却碰巧合法」这一类它结构上看不见——M2 把载荷符号化补的正是这一格。
- 三方：材料 `_d13-item1-background.md`，三条腿的产出 `d13-item1-opus-forward-output.md`、`d13-item1-sonnet-reverse-output.md`、`d13-item1-local-attack-v2-output.md`；三腿一致把原候选排最后，两条云端腿各自收敛到「前缀判定与提交原子性」这道接缝，主 agent 按封闭性把择新与发布原子性收进同一个单元（判决与撤下原候选的三条门槛原文在变更史）。
- E165（恢复重放有界穷举与变异杀伤界）：在界内对真 `replay_journal` 判 M2 的语义层、M4 与 M3 两读，并量变异杀伤界；用户 2026-10-01 定它当轻量工具；C 域合法崩溃状态数按两种切法分写（甲切法是 D16（发布语义） 已定项 7 的屏障位置）；对抗一族里不动点的违例全在 I-8.12（同实例记录的 txg 不减） 不成立的记录表上，谓词成立那一边 0 违例（第 3 次重跑第一段）；择根合判族调真 `choose_root`、`scan_journal`、`replay_journal`，界内、合法、不带故障时已确认的完整发布不丢（第 3 次重跑第二段）；择根序的四个改坏变体在合法历史 K、K9、KW 三族界内只有取第一个有效者红，另三种在对抗根环 KA 上红（第 3 次重跑第三段）。
- 三方判决（M1–M4 的措辞与落点）：`research/prompts/m3-formal-r1-main-verification.md`、`research/prompts/m3-formal-r2-main-verification.md` 第二节、`research/prompts/m3-formal-r3-main-verification.md` 第二、三节；M3 单调推论的四步推导在 `m3-formal-r2-opus-output.md`「3.2 推的：串行提交下，j1 与 j2 推得出落到的版本单调」一节。
- 三方判决（M3 不动点前提补 I-8.12（同实例记录的 txg 不减） 与落点现状）：`research/prompts/m3-formal-code-r1-main-verification.md` 第二节不动点前件那一格（打中条款、不打代码）与第三节第 5 条；装置的前提比 Verus 的前件宽、违例全落在该谓词不成立一边的对账在调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 10 批「10-5 交回」那一条。
- 用户定案（2026-10-01 弹窗，都照推荐）：卸载记号除外、M4 射程句的前提写全、单调的版本序取 (`checkpoint_txg`, 实例代号)、txg 顶格取「报 no_successor 且一条没施加算相等」；写回等 K10 判完一起做，原话在变更史。
- 挂载透明：无实验：判法在崩溃注入里（`crates/singlefs-harness/src/mount_transparency.rs`，checker 档的崩溃注入调它），不是实验；三轮判决里攻方的数在副本装置上量，代码轮第二轮攻方在入库代码上量的第一次崩溃 147 806 态、二次崩溃 234 773 态 0 红主 agent 没复跑。
- 三方判决（挂载透明）：`research/prompts/m3-formal-r4-main-verification.md` 第二节「挂载后再恢复：模掉什么」「比的对象与射程」两行与第四节第 2、3 条（独立的第四条、比的那一版、射程）；`research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条（逐字段表、二次崩溃比 v1、拒绝合法的条件）；`research/prompts/m3-formal-r6-main-verification.md` 第二节 U5、U6 两行（txg 只判下界、拒的判据在崩溃镜像上自己复算）与「逐字段表没碰的字段」一行；`research/prompts/m3-formal-code-r2-main-verification.md` 第二节 K2 各行（实现照判、站住；暖机与挂载推的抬 F 拆段）。
- 用户定案（挂载透明加二次崩溃，2026-10-02 弹窗，原话在变更史）：`research/prompts/m3-formal-r4-main-verification.md`「用户定案（2026-10-02）」二次崩溃那一行。
- 择根序变体那一句：调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 12 批 E165（恢复重放有界穷举与变异杀伤界） 第 3 次重跑第三段判 Q2 交回那一条（主 agent 判补这一句）。
- 挂载续做算挂载的一部分、F 的界按挂载交回的抬 F 在写表末尾的位置取：用户定案 2026-10-03「续做算挂载的一部分」（原话在变更史）；代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 3 条（界按写流位置取，找不到不 panic）；实现 `crates/singlefs-harness/src/mount_transparency.rs`。
- 拒绝合法那一句里「读阶段判完之后系统配置又读不出」这一拒因的复算带补写例外、判槽自证读池级 checker 的逐盘数：三方判决 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第二节 Y1、Y3 两行与第三节交用户表「条文」「G4 拒绝那一臂的判槽自证」两行；用户定案 2026-10-22「改读池级 checker」，零轮，原话在变更史。

**欠**：C77（重放起点未定义）：在择新与前缀判定那道接缝上；C649（Verus 判不到的两样回调结果）：已还清（M2 语义层的反向链与点名单元仍只能写成后置条件，Verus 一侧不判，两个读盘回调改由 `crates/mutations.tsv` 的变异行罩着，见欠账表已还清表）；C650（择根与重放没有合判）：M4 的射程句要择根与重放合起来才成立；C653（扫环胶水内存随盘数涨）；C654（必红世界期望条数随求解环境漂）；C655（挂载透明逐字段表有字段没归类）：已还清（逐字段表照判决 Y1、Y2 补齐，见欠账表已还清表）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:91-104`（整段抄，未转述）**

```markdown
#### 已定项 2：先串行提交

**定案**：事务层先承诺串行提交——`.claude/rules/fs-design.md`「判据写不出来时，先只做一条，而且**先做慢而正确的那条**」落到实处，串行是本工程明令的默认值，不是在两条路里挑了一条。越过交点再开并行，而不是在交点之前就承诺并行。对将来并行提交的一条前置约束：记录按事务号顺序追加，事务号 N 的 fsync 在返回前必须等 < N 的事务全部已追加 ∨ 已作废 ∨ 已确定实例切换或转只读（作废是提交管道的内存状态，对并发后继可见）；串行提交下这条规则空转。

**射程**：串行的天花板在一次 checkpoint 约 1.1 万条条目（单线程合并耗时追上一次设备同步的交点，算式在 C58（串行提交撞天花板没人报警））：低于交点设备同步是瓶颈、串行不吃亏，高于交点单线程合并变成天花板，没有安全余量。已定项 2 不决定记录头的字段清单、不改变新池新建文件写出的字节，它决定性能天花板与 crash refinement 的范围。并行提交仍然会动崩溃语义：「前缀」是全序上的概念，并行下一个前缀可能同时含有完整的事务 B 与残缺的事务 A；记录头已带事务边界字段（D23（journal 的角色与格式） 已定项 7：事务号 8 + 提交标记 1）。

**依据**：
- E17（write buffer 合并上限）：单线程 sort-merge 的上限是交点的一半；按叶分区的无锁并行是越过交点之后的逃生口，但不许默认它够。
- E44（序号位宽的本机实测与代价）：本机 fsync 率是交点的另一半。
- E42（一事务几条记录）：串行下一个事务跨多条记录同样会被前缀切开——决定记录头字段清单的是「一个事务恰不恰好产生一条 journal 记录」，不是串行 / 并行。
- 用户定案，记在变更史；将来并行提交那条前置约束来自 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1。没走三方论证：这是确认一条已经写明的默认值，真正要走三方的是何时改成并行，那一步没做。

**欠**：C58（串行提交撞天花板没人报警）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:105-137`（整段抄，未转述）**

```markdown
#### 已定项 3：RefFS 做到极致，自己处理持久化与崩溃语义

**定案**：不做只管功能的内存模型，做一个自己处理持久化与崩溃语义的参照实现（RefFS），把 RefFS 从「储备候选」升为**要做的东西**。「做到极致」取语义级：RefFS 只在内存里按 POSIX 操作走，自己定义每次 fsync 之后什么算持久、崩溃之后允许看到哪几种 POSIX 状态，没有盘上格式；带自己盘上格式、自己写盘与恢复的那一档（工程量接近再写一个文件系统）不做。RefFS 与实现不许共用崩溃点枚举的那段代码：做到极致之后「三 oracle 合谋测试」（已定项 6）防的那个共同误解，最危险的落点是「RefFS 与实现是否共享同一个崩溃语义的错误理解」。

RefFS 的来处与边界：

| 项 | 定案 |
|---|---|
| 来处 | 只照 kb 条文另写：不拷 O1（内存模型对拍（HashMap 理想文件系统）） 的代码，不读录制流，不调切段、状态计划与实现的崩溃点枚举 |
| 放哪 | 单独一个包 `crates/singlefs-reference-filesystem`：依赖闭包（dev-dependencies 也算）⊆ {它自己, `singlefs-format`}，能只用 std 就只用 std；源码不引 `singlefs_core`、`singlefs_checker`、`singlefs_harness`、`singlefs_checker_tier`（经改名的别名同样算）；`singlefs-core` 与 `singlefs-checker` 的依赖闭包里没有它。三条由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格判 |
| 输入白名单 | 都由调用方交入，只收五样：每个操作的结局（成、在任何写之前拒、写了之后报错、原样重发）；崩溃落在第几个操作里（hi）与它的归属规则；回退目标 m；每次恢复读出的内容；显式发布点 |
| 第一版的操作 | 第一个文件、覆盖写、发布调用、fsync、回退 |
| 调用点 | 崩溃注入与故障注入的调用点上，RefFS 的判定与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官的全部四条判据合取，任一红即红 |

崩溃语义（串行提交、不带故障的崩溃状态）：

1. **fsync**：fsync 返回成功之后，这次 fsync 等到的那次发布带走的全部操作必须在（串行下就是调用开始之前已取号成功的全部操作，按单元计）；按状态下标读，即允许集合的下界 lo 抬到那次发布的末尾。fsync 报错时不抬 lo，报错那次发布带的操作可在可不在；之后的操作接在下一次恢复读出的状态上判。
2. **丢法：按发布边界的前缀**。崩溃落在第 hi 个操作里时，恢复允许读出的是 {S_j : j = lo，或 j 恰是某次显式发布的末尾；lo ≤ j ≤ hi}，S_j 是上一次恢复之后前 j 个操作施加出来的状态，S_0 是上一次恢复读出的内容。丢掉任意子集的那种丢法在串行提交下出局（与 D16（发布语义） 已定项 12 不相容），开并行提交时重开。
3. **抬 lo**：返回成功的发布调用、fsync、回退都抬 lo（D16（发布语义） 已定项 7：回退与 fsync 同一个返回条件）；回退算一次已确认发布，它的下标算发布末尾。
4. **回退**：回退是 RefFS 的一个操作，回退之后的内容等于整段历史里第 m 次已确认发布之后的内容；m 由调用方从实现报的回退根翻出来交入，RefFS 不判目标选得对不对。m 的编号覆盖每一个写根的入口（发布调用、回退、写行与暖机、抬 F、卸载），映到写它时的现行版本，现行版本含每次恢复读出、重新起头的那一份。
5. **崩溃时刻的归属**：崩溃落在第几个操作里，按「切开当前段的那道屏障由哪一步发出」归属，不按第一个被扣下的写；这条只给 RefFS 用，O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不动。
6. **恢复之后**：每次恢复之后，RefFS 先判读出的状态在允许集合里，再以它为起点接着走。

**射程**：只定做到什么程度与 RefFS 的形态，不定排期；不许读成「先做 RefFS 再做实现」——RefFS 是验证手段不是里程碑（`.claude/rules/format-evolution.md`「从写开始，不从读开始」）。显式发布点只取历史里的发布调用与 fsync：自动触发的攒批落地之前 RefFS 只接显式发布点，自动触发的发布不在 RefFS 射程，归已定项 1 的 M2 与记录核对器；崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步。带故障的崩溃状态（说谎的设备、丢整块盘 C519（丢一整块盘时恢复丢掉刚确认的那一版））不适用。独立性：RefFS 不读录制流、不调切段与状态计划，但它的 (lo, hi) 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官盘级下界那一条判据的入参输入同源，记在 C191（同源性计数器只盖了一对） 那一类同源账上。D17（实现分层与第三方管道） 已定项 4 据此把「入口」的可执行定义交给 RefFS 的操作面，fsync 的持久性承诺由 RefFS 的崩溃语义定义；O3（独立规约执行器） 仍是崩溃点集合的独立执行器，职责不变。RefFS 的第一版在 `crates/singlefs-reference-filesystem`，崩溃注入两截与故障注入重开的调用点已接：harness 档经翻译层 `crates/singlefs-harness/src/reference_filesystem_translation.rs` 依赖它，调用点判 RefFS 与 O1（内存模型对拍（HashMap 理想文件系统）） 崩溃判官两票、任一红即红。

**依据**：
- 无实验：做到什么程度、RefFS 怎么来与收什么是验证投入与边界的取舍；三轮判决里攻方的数都是副本装置上量的，入库装置上还没重做（`research/prompts/m3-formal-r5-main-verification.md` 第五节第 3 条），这里不引。
- 用户定案，原话在变更史：「做到极致」（原定案）；做到语义级（2026-10-02，岔路单 `m3-formal-7-forks.md` 里 RefFS 做到哪一档那一行，是对原定案换了读法）；按发布边界的前缀、调用点合取、另起、单独一个包（2026-10-02，`research/prompts/m3-formal-r4-main-verification.md`「用户定案」）；合取收严成全四条、回退目标作输入（2026-10-02，`research/prompts/m3-formal-r5-main-verification.md`「用户定案」）；回退抬 lo、崩溃时刻按屏障归属、m 覆盖每一个写根的入口、白名单加原样重发（2026-10-02，`research/prompts/m3-formal-r6-main-verification.md`「用户定案」）。
- 三方判决：`research/prompts/m3-formal-r4-main-verification.md`（第一轮：fsync、丢法、独立性三格与六条岔路）、`research/prompts/m3-formal-r5-main-verification.md`（第二轮：fsync 合成句、按发布边界补三句、合取收严、另起与单独一个包、任意子集出局限串行）、`research/prompts/m3-formal-r6-main-verification.md`（第三轮最终轮：回退与 lo、射程两句、白名单、fsync 按状态下标读）。
- D17（实现分层与第三方管道） 已定项 4：O1（内存模型对拍（HashMap 理想文件系统））明写「不管崩溃」，定义不了 fsync 的持久性承诺，而第一个可运行目标恰恰是「正确提交一个事务」——做到极致的 RefFS 补的正是这一半。

**欠**：C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C191（同源性计数器只盖了一对）；C658（崩溃注入第三截放过丢掉的一步复活）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:138-153`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。**同一块盘上连着的屏障并成一道**：录制流里一道屏障与这块盘上一道屏障之间没有任何一次写（别的盘的屏障不算写）时，这一道不记（`crates/singlefs-harness/src/lib.rs` 的 `push`），独立模型照同一条规矩记；中间没有写，这一段不产生崩溃状态，合并与否不改变崩溃状态集合，只改变操作数与种类串。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。崩溃点重放的扇区取 512 字节；两条崩溃点重放流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条崩溃点重放流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；崩溃点重放的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，崩溃点重放小流里记录核对器报出来）在 `m2-rev-b3a2-implementer-report.md` 第一、三节。
- 同一块盘上连着的屏障并成一道：用户 2026-09-28 定「并，跟今天的录制器」（E142（新池新建文件的干跑） 第十九次跑撞上登记的停机 S19-crates：独立模型两道都记、录制器并掉第二道，暖机与整条流四格的操作数与种类串不同、崩溃状态数相同，报告 `e142-r19-runner3-report.md` 结论段，登记 `e142-r19-prereg.md` 的 R40），原话在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:154-170`（整段抄，未转述）**

```markdown
#### 已定项 5：只共享一份从 kb 生成的常量

**定案**：checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（`.claude/singlefs-ai-sop/rules/machine-first.md`：重复要生成，不能手抄）。**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用） 由门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格定的判据，形态是 crate 粒度的依赖闭包交集、不是符号级，「格式解析与常量除外」那条例外就是这一项在用）。

- **判定宽度不许从别处拿**：I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 的判据是挂载时探测的 `physical_block_size`，它不在镜像里。checker 自己探一次，再与系统配置记录的 mkfs 时取值比对（D22（单元原子性怎么合成） 已定项 2「mkfs 按池内最大值划、挂载时逐设备复算比对」的 checker 侧形态）。**探不到时报「声明值，未探测」，不许当成探到的**——文件当设备的镜像属于这一类，崩溃点重放的绝大多数镜像都是文件，所以这条不是边角。异构池里判定宽度是一个集合不是一个数（C17（设备几何当全局参数）），checker 要复算 mkfs 那步取最大值的聚合，不然健康的异构池上会假红。
- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
- **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 code-source-discipline 的 format-constants 格按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size` 与 `io_min`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源的类型是 `crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`：声明值那一种（`DeclaredOnly`）已接到 I-7.5（根槽按判定宽度对齐） 的判定上（已实现，`decision_width_declared_by_the_chosen_system_configurations` 取各盘所选系统配置声明的 max(`physical_block_size`, `io_min`) 里最大的），探到那一种（`Probed`）只有类型定义（`crates/singlefs-checker/src/lib.rs`），全仓没有构造它的代码、没有接到任何判定上；I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。

**依据**：
- 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
- 三方对抗：材料 `_checker-sharing-vehicle-background.md`，腿的产出 `research/prompts/checker-sharing-vehicle-*`，腿况在 [verification-build.md](../verification-build.md)——正推腿从 D13（验证路线）、C12（增量语义共用） 推出「常量在例外内、其余禁止」；替代反推腿与本地腿各自构造出同一条：kb 里写错的值共享与各抄都抓不住。
- 用户定案，记在变更史。

**欠**：C17（设备几何当全局参数）；C387（常量模块手写，不是从 kb 生成）；C388（checker 的判定宽度不自己探测）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:171-201`（整段抄，未转述）**

```markdown
#### 已定项 6：三个互相独立的 oracle

**定案**：三个 oracle 各判一样东西，独立性各有来处：

<!-- doc-lint:registry name-col=2 -->

| # | oracle | 判什么 | 独立性来自哪里 | 验不到什么 |
|---|---|---|---|---|
| O1 | 内存模型对拍（HashMap 理想文件系统） | POSIX 可见语义：内容、大小、目录结构；无崩溃处的内容期望归它，崩溃之后允许看到的集合归 RefFS（已定项 3） | 不碰盘、不解析磁盘格式 | 磁盘结构是否自洽；崩溃语义；**两份实现对同一条 POSIX 语义的共同误解** |
| O2 | 独立解析器 + checker | 磁盘字节是否满足 I-1（块自描述）~I-6（加密不变量） | **checker 不得调用被测 crate 的任何反序列化/遍历代码**，必须重新实现一份解析，直接读裸设备 | 只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对 |
| O3 | 独立规约执行器 | 崩溃点截断后的每个磁盘状态是否落在规约允许的集合里 | **用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导；第一版用 Python 单写（E170（O3规约执行器第一版）） | 规约本身可能写错——**必须配阳性对照才有意义**；被投影的六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值（journal_tail）、一次发布内各写的先后 |

**独立性要被验证，不许自称**：设计一个「三 oracle 合谋测试」——拿一个精心构造的坏实现
（例如 nonce 永远为 0）去过三关，若三者全部放行，说明存在共同盲区，回头补 oracle。
功能正确性不只押在这三个 oracle 上：O1（内存模型对拍（HashMap 理想文件系统）） 那一行已把崩溃之后允许看到的集合交给 RefFS（已定项 3），恢复函数的 M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）（已定项 1），需要第二个输入的核对归记录核对器（已定项 7）；三个 oracle 之间的共同盲区靠「三 oracle 合谋测试」找。

**射程**：三个 oracle 各自验不到什么写在表的最后一列；O2（独立解析器 + checker） 的定义域归已定项 7，需要第二个输入的核对归记录核对器。**实现今天的样子**：O2（独立解析器 + checker） 在 `crates/singlefs-checker`（只依赖格式常量 crate）；O1（内存模型对拍（HashMap 理想文件系统）） 的形态是 `crates/singlefs-harness/src/model.rs` 的理想模型（只住内存、只用 `singlefs_format` 的常量、不建崩溃），门禁 harness-model-differential-and-scenarios 拿随机历史逐步对拍；O3（独立规约执行器） 是只照 kb 文字写的 Python 执行器，现行那一份是 E171（三oracle合谋测试） 第 1 次重跑用的 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`（E170（O3规约执行器第一版） 第 2 次重跑从第 1 次重跑冻结的 `scripts/experiment-apparatus/e170_second_run_o3_spec_executor/` 拷出 `scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/` 加谓词 W、T，再跟上 487 字节 journal 记录头；第 1 次重跑冻结的那一份与第一次那一份 `scripts/experiment-apparatus/e170_o3_spec_executor/` 留档）：严格跑法判得出（第一次导出的批 1、批 2 上 0 态撞「规约欠一句」），与实现逐态一致（两边全放行，不当它判得对的证据；「择到的根不是自证根里 txg 最大的那条」「③ 收到不少于 2 槽」两条路径批里 0 态，C661（规约执行器批里两条路径没有态）），坏状态表照改正的期望重判有判别力（阳性对照与盲测都判对），还没接进崩溃点重放；判一个崩溃态的做法：照规约的恢复规则（择系统配置、择根、前缀按 D23（journal 的角色与格式） 已定项 14 的五条口径）从镜像自己恢复出一版，恢复不出（整池拒或没有自证根）判不在；恢复出的这一版的投影（实例代号、checkpoint_txg、树表单元指针、中央映射树根指针、树 ID 水位、F）要等于版本表里某一版的投影，且这一版的下标在 [已确认, 已开始] 之间。版本表是基镜像上自己恢复出的现行那一版，加录制流里每一次根槽写按流序各一版，每版的投影取实现写进根槽的字节；已确认取崩溃态里落了的最晚一条写之前已返回的最晚一版（只从落了的写推），已开始取落了的写所属的最晚一版。系统配置槽只用来择槽、取根环与 journal 环的几何、判池级字段越界整池拒。所以恢复规则那一半是照 kb 文字另写的，允许集合那一半取自实现自己写出的根；RefFS（已定项 3）在 `crates/singlefs-reference-filesystem`，崩溃注入与故障注入的调用点已接；「三 oracle 合谋测试」是 E171（三oracle合谋测试），读数取自一棵冻住的树（快照摘要见实验页，与现行 `crates/` 不是同一份）：O1（内存模型对拍（HashMap 理想文件系统）） 一列是模型对拍加模型的崩溃判官，不接 RefFS，与表里 O1（内存模型对拍（HashMap 理想文件系统）） 那一行不同（崩溃之后的集合那一半 E171（三oracle合谋测试） 没用 RefFS）；O3（独立规约执行器） 一列用 E170（O3规约执行器第一版） 第 1 次重跑冻结的那一份；手造坏镜像那一族只喂 O2（独立解析器 + checker）、O3（独立规约执行器），期望出自 E170（O3规约执行器第一版） 的设计员，Q2 照登记字面判「有」（4 张全是 P8），按登记 F3 判成 E170（O3规约执行器第一版） 推期望推错、这四张不是坏镜像，真坏镜像里 O2（独立解析器 + checker）、O3（独立规约执行器） 两条全放行的 0 张；三条的源头是否独立（表第 4 列）E171（三oracle合谋测试） 只在 Q2 量过 O2（独立解析器 + checker）、O3（独立规约执行器） 一对，那一对的补丁与期望同出 E170（O3规约执行器第一版） 的改法代码，不当独立读数，O1（内存模型对拍（HashMap 理想文件系统）） 不在 Q2 里；没有量到的不写成「互相独立」被验证过。

**三条 oracle 都不判的两样**（第一样由崩溃注入与崩溃点重放两侧判，崩溃点重放那一侧的观察者在实现批）：一是无读故障的崩溃态上，D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真（系统配置槽里的见证值超前于盘上读得出的记录）：崩溃注入第二截判拒绝合不合法时只在崩溃镜像上复算这条判据，为真就判合法（挂载透明的射程只管不带故障的状态，已定项 1）；要有判官问「这个崩溃态没注入读故障，判据凭什么为真」：崩溃注入第二、三截在没注入读故障的崩溃态上复算这条判据，为真即判拒得不合法（`crates/singlefs-checker-tier/src/crash_injection.rs` 的 `RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`，C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 已还清，见欠账表已还清表）。「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由崩溃点重放 mkfs 返回之后各条流上的观察者判：观察者按流登记，记录 mkfs 的流不挂，只记相对基镜像新出现的、一份自证过的系统配置都没有的盘，撕裂域在内，判槽自证读池级 checker 的逐盘数（`crates/singlefs-checker/src/image.rs` 的 `system_configuration_redundancy_report`），另有一条登记完整性用例判含系统配置槽写段的流都挂了它（实现在实现批）。随机历史里补写闸（D22（单元原子性怎么合成） 已定项 8）不过的那一形，由崩溃注入第二、三截的 G4 拒绝那一臂判；补写闸过了的那一形，随机历史里没人判，正确实现留不下这种盘面（两次抽样都没打中）。mkfs 返回之前的崩溃态不在观察者与 G4 拒绝那一臂的射程：mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后；G4 拒绝那一臂与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条还有一处不同形：择不到系统配置时那条的复算交 `None`、G4 按真，这时读阶段本身已拒、到不了这一格。二是一次发布内各写的先后：O2（独立解析器 + checker） 的 I-7.12（系统配置 F 不低于同盘根上的 F） 判抬 F 那一串先写系统配置这一步；变异点名的用例只判 E171（三oracle合谋测试） 第 270 行那一种（发布返回之后每个崩溃点的见证值），第 242 行点名的用例判的是失败路径冻结；`layout/01-first-txn.md` 第八节把根槽 FUA 与系统配置槽轮换登记成两段，只是登记在册；其余写序不在三条之内。

**依据**：
- E170（O3规约执行器第一版）：O3（独立规约执行器） 照「另一种语言、只照 kb 文字重新推导、配阳性对照」写得出第一版，判得动单个崩溃状态、一态够进快档；第 1 次重跑（照 2026-10-03 的 kb 改过一次的执行器）严格跑法判得出、与实现在取样的批态上一致、阳性对照有判别力（第一次跑判不出与判别力不足那两笔留在实验页）。
- G4（无读故障的崩溃态上每块盘至少一份自证过的系统配置）：代码轮判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 j2-b 立、第 29 批实现，射程与 mkfs 那一段见 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L3 与第三节第 3 条。
- 两槽全废盘面改由崩溃点重放 mkfs 返回之后各条流上的观察者判、G4 拒绝那一臂留着：三方判决 `research/prompts/m3-close-g4-rewrite-r3-main-verification.md` 第二节 Y2 各行与第三节（前两轮 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`、`research/prompts/m3-close-g4-rewrite-r2-main-verification.md`）；观察者的接法（按流登记、相对基镜像、登记完整性用例）是用户定案 2026-10-22「按流登记+相对基镜像」，零轮，原话在变更史。数在三方腿的模型与副本原型上量，没立实验号。
- 三个 oracle 的分工与各自独立性的来处是验证体系的结构设计；独立性靠「三 oracle 合谋测试」观测（E171（三oracle合谋测试））。
- E171（三oracle合谋测试）：在一棵冻住的树上，坏实现表里有三条 oracle 全部放行的行、有只靠一条才抓到的行，O3（独立规约执行器） 在坏实现表上一行没红；手造的伪造根、伪造续接镜像里有 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的（读数与限定在实验页 Q1、Q2、Q3 三行）。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- O1（内存模型对拍（HashMap 理想文件系统）） 与 RefFS 的分工句：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节另起与单独一个包那一格（两腿同向）。

**欠**：C445（模型对拍的取样点不随写路径增长）；C446（未实现的不变量没有前置登记，也没有到期）；C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C672（交回值或内存状态错而写流不变的一类）；C679（卸载记号没进模型对拍）；C874（补写那一支进不了崩溃测试、运行时看不出）：两槽全废盘面的观察者与 G4 拒绝那一臂改读池级 checker 都还没实现

```

**出处 `.claude/kb/decisions/13-验证路线.md:202-220`（整段抄，未转述）**

```markdown
#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像

**定案**：

> **定案：O2（独立解析器 + checker） 的定义域是单个镜像**——一个一元谓词 `P(镜像) → 成立/不成立`，
> 只判集合成员，永不判 identity。**任何需要第二个输入（另一个镜像、或一段记录流）的检查都不属于它。**

**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`——持久集合是枚举器给出的这个崩溃状态里哪些写落了盘（崩溃点重放的段内整写子集）；只看盘上字节分不开「写没落、而原地内容恰好相同」与「写落了」，判复用与缺席要拿持久集合判；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。

**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-checker-tier/src/crash.rs`，接在崩溃点重放的每个崩溃状态上（门禁 checker-tier-crash-point-replay）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 崩溃点重放不覆盖可写挂载：崩溃点重放在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在崩溃点重放的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-checker-tier/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。

**依据**：
- 无实验：**依据是类型，不是成本**：「逐字节比对」是二元关系 `R(镜像₁, 镜像₂)`，**没有任何投入能把一元谓词变成二元关系**；I-4.5（已停用·需两镜像） 因同一条门槛退役。
- 「崩溃后镜像」是两份：随 D16（发布语义） 已定项 4 收窄问法那一轮三方论证（材料 `_d16-items134-background.md`）由用户定案，记在变更史。
- 持久集合那一样入参：用户定案，原话在变更史；判别子观测不到的实据在 `research/prompts/m2-layer0-scale-r3-main-verification.md` 第二节按扇区豁免那一行。
- 崩溃点重放不覆盖可写挂载、可写挂载与二次崩溃由崩溃注入覆盖：用户定案（代码审阅第 2 条，2026-09-27 弹窗选「崩溃注入层补可写挂载」），记在变更史；崩溃注入那一侧的实现在 `m2-rev-b3b-implementer-report.md`。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:221-244`（整段抄，未转述）**

```markdown
#### 已定项 8：自研的两样：Merkle 根链差分与代际增量 scrub

**定案**：现成工具没有、本工程自研的两样：

**1. Merkle 根链差分 —— COW + 每事务发新根独有的杠杆。**
历史根在被回收前物理存在过，构成一条可回放的链。对相邻两代根递归比较「子树指针 + 校验和/MAC」，
**指针没变的地方直接跳过不下钻**，复杂度只正比于两代之间实际改动的部分。
它能检出常规遍历式 checker 检不出的一类错误：**同步篡改后自洽的原地写**——
某次写绕过 COW、原地改了块内容并同步更新了校验和，单点看它是自洽的（I-2.1（校验和与内容匹配） 全绿），
而根链差分会发现矛盾：**没走 COW 就不该有新分配，父指针理应没变，校验和却变了**。
⚠️ **必须配阴性对照**：正常 COW 提交后跑同一检查必须全绿，否则它会把每次正常更新都当成篡改。

**2. 代际增量 scrub。** 靠 I-1.2（块头写序已发布） 与 D5（快照 / 空间记账机制） 的 birth txg
维护一个「上次 scrub 扫到的 generation 水位」，只遍历 `birth generation > 水位` 的块——
老子树的校验和链没变就必然还自洽。**它让「每次提交都全量验证本次改动」在挂钟上可行，不必抽样。**

**射程**：被搬过的副本 `birth` 不变、会永远落在水位之下，代际增量 scrub 看不见它们。`crates/` 里没有根链差分（`grep -rn 根链 crates` 零命中），也没有代际增量 scrub（`grep -rn scrub crates` 零命中）。

**依据**：
- 无实验：两样都靠 COW 每事务发新根与 birth txg 的结构性质成立，都还没有实现，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C389（Merkle 根链差分零代码）；C194（代际增量 scrub 看不见被搬过的副本）；C408（代际增量 scrub 零代码）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:245-278`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 触发 |
|---|---|---|
| 冒烟层 | 固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程），精确全量 | checker 档的全量（已定项 15）：用户要求时跑；提交时默认只跑快档 |
| 常规层 | 中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列「本次未跑」 | 每次提交 |
| 大负载层 | 常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量 | 用户要求时 / 发布前 |

**剪枝加 ignore 之后敢说什么**：敢说「开着的节点在全部崩溃状态上通过」；
**不敢说**「ignore 的节点通过」。

**全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快档全绿（必要条件）。**
**里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑。**
「全量」指两样：checker-tier-crash-point-replay `--full`（门禁 checker-tier-crash-point-replay，已定项 15 的全量）与崩溃放量全量（`scripts/crash/crash-amplification.sh` 不带参数）。
「动了崩溃点重放或判器」按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上、对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算；阶段把作废块数与变了的判器单元清单打出来，主 agent 照它弹窗，不按路径清单判。
「上一次全量」由主 agent 写进当前里程碑记录一行：日期、两样各自的结论。
**全量档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个全量档 bug 的挂钟 ≈ `log2(N) × 一趟全量的挂钟`，一个不可复现的全量档失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：只有冒烟层（门禁 checker-tier-crash-point-replay）；冒烟层分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 checker-tier-crash-point-replay `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求时跑；常规层、大负载层在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；节点怎么精确剪枝、ignore 节点什么时候开归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束；常规层、大负载层在门禁里的负载还没有定，E184（常规层与大负载层的代价计数） 第一段已量候选负载的规模与态数区间重叠，只备料；第二到四段（作废面、冷吞吐、两种落法的固定开销）未跑。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 平时快档用甲二：用户定案，原话在变更史；崩溃点重放规模三轮判决 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- 用户定案：「我们不是抽样 我们是剪枝 加 初期ignore」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定案：不抽样，剪枝加初期 ignore」那一行）、「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）、「1 该枚举该设计的设计完 设计全」「2 枚举不完 正是剪枝的用处」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节里材料员被门禁 doc-process-records 的 implementation-premise 格挡下、用户定『设计全、枚举不完靠剪枝、粒度合适』的那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 9 推翻。
- 用户定案（2026-10-01）：全量档不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。
- 用户定案（2026-10-01）：全量档全绿的时点是里程碑收尾与动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑，原话在变更史；交用户那一格是三方判决 `research/prompts/test-script-align-r1-main-verification.md` 第二节 t1-3。
- 三方判决 `research/prompts/test-script-align-r2-main-verification.md` 第二节 U3（「U3 t1-3 写回照字面做不了」那一行）：「全量」的两个所指、「动了崩溃点重放或判器」的路径判法、「上一次全量」的记法，「准入条件」与「异步」合成一句。

**欠**：C409（常规层、大负载层在门禁里没有阶段）；C635（上一次全量没有机器可读的标记）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:279-308`（整段抄，未转述）**

```markdown
#### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」

**定案**：常规层 精确覆盖先缩问题，再谈剪枝。崩溃点的定义域是**事务层状态机**，而 `.claude/rules/fs-design.md` 有两句顶层规则：
「一个事务层，所有结构共用」与「唯一不许多样化的是事务层」。所以布局能改的只有两类：

| 差异 | 改变的是 | 判定 |
|---|---|---|
| 节点大小、条带宽度 | 一步写多少字节、某步重复几次 | **参数**，不新增步骤种类 |
| 旋转 vs SSD | seek 代价、IO 调度 | 落在设备层，**不落在事务层协议里** |
| **zoned（ZNS）** | **能不能原地覆盖根指针** | **结构性**：只能追加、根位置漂移，且多出 zone finish/reset |

**落地手段是一个封闭枚举 + 穷尽 match**：`crates/singlefs-core/src/transaction.rs` 的 `CommitStep`，步骤种类的登记位是 [layout/01-first-txn.md](../layout/01-first-txn.md) 八的段序列登记表。
任何布局要引入新的结构性步骤**必须改这个共享定义 ⇒ 编译器强制所有引用处补分支 ⇒
这个 diff 门禁 grep 得到**，可以自动打标「本次引入了新的结构等价类，需为它新开一轮独立大负载层 重放」，
**不靠人记得申报**（按 diff 算门禁范围欠在 C8（门禁范围判不出来））。再加 newtype 隔离（`RotationalPtr` / `ZonedPtr`），
把 D12（目标介质） 已定项 2 里「不许共用」那一桶（记账事实的判定路径）变成编译期约束。
⚠️ **它有第二个职责（D17（实现分层与第三方管道） 加的）：它是第三方管道的接入点。**
协议归本工程所有，一条管道要引入新的结构性提交步骤必须往本仓发这个 diff。

**射程**：旋转与 SSD 在提交协议层面同构，由 D12（目标介质） 那一轮三方论证承担；zoned 那一行只说明它至少多出一个等价类。等价关系按录制流的段序列同构、有几个，归 D17（实现分层与第三方管道） 已定项 2。按种类串认节点够不够，第一轮判决打中（挂起，待第二、三轮攻）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节。
⚠️ **它钉住的只是「有哪些步骤」，钉不住「每一步的失败原子性宽度」**——
持久内存与对象存储改变的是每一步的失败原子性宽度，枚举接不住（缺口与补法见 D17（实现分层与第三方管道））。
`RotationalPtr` / `ZonedPtr` 这层 newtype 隔离 `crates/` 里没有（`grep -rn "RotationalPtr\|ZonedPtr" crates` 零命中）。

**依据**：
- 无实验：N 是什么由 `.claude/rules/fs-design.md` 那两句顶层规则推出，没有可量的量；等价关系怎么判、有几个的依据在 D17（实现分层与第三方管道） 已定项 2。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。

**欠**：C8（门禁范围判不出来）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:309-343`（整段抄，未转述）**

```markdown
#### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条

**定案**：

**「按欠账轮转的账本」调度：否决。** 格子 = `(布局, 子阶段桶)`、按欠账降序取队首、声称任一格子最多 `K_max = ⌈N×M/C⌉` 次提交必被覆盖的那个调度被否决：它保证不了的恰恰是它要防的那一类——慢的、罕见的分支（实测验算与另外四条否决理由在变更史）。

**留下的三条**：

1. **方向对**：要一个**确定性的、能被门禁判红的覆盖记录**，而不是概率论证——
   概率保证在长期维护里等于没有保证。

<!-- doc-lint:registry name-col=2 -->

| 符号 | 简称 | 含义 |
|---|---|---|
| T1 | 常规层 挂钟预算 | 常规层 本地挂钟预算，是输入不是输出 |

2. **`T1`（常规层 挂钟预算） 是输入，不是输出**（`.claude/rules/fs-design.md`：门禁运行时长本身是设计约束）。
   正确顺序是「先声明预算 → 用它约束一次提交能开几个 ignore 节点」，不是「量出耗时 → 推出能开多少」。
   **这一条不需要任何测量，现在就该定。**
3. **判别力必须与覆盖分开检查**：每个 ignore 节点被打开的那次提交里，额外对一份「已知坏的镜像」跑它自己的 checker，
   不报错即判红——这是 D13（验证路线）已有的阳性对照纪律的直接复用。见 C13（checker 判定失效）。

门禁里没有「常规层 崩溃点重放」这个阶段，量不出 `T1`（常规层 挂钟预算） / `C` / 每格耗时。
⚠️ **因此 `C`、`K_max`、`T1`（常规层 挂钟预算） 这三个数一个都不许现在写进 kb。**

**射程**：只管 ignore 的节点什么时候开；冒烟层全量枚举不需要 `T1`（常规层 挂钟预算）。第 2 条的「现在就该定」与末句「一个都不许现在写进 kb」说反话，记在 [verification-build.md](../verification-build.md)「待定案」第 6 问，没判。节点登记表带状态字段（开着 / ignore / 等实现），另加三条——每个 ignore 行点名一个会变的对象、那个对象变绿而节点仍 ignore 时判红、覆盖报告列 ignore 的状态数与占比：这三条被攻过零轮（`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节），不写成定案，等第二、三轮再攻。

**依据**：
- 无实验：否决账本靠的是一次调度规则的模拟验算（200 轮、取后半程稳态最大值、严格前缀贪心，三行数在变更史），没有登记成实验；留下的三条是门禁设计原则，没有可量的量。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。
- 用户定案：「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 11 推翻，「逐格判决」一节给节点登记表带状态字段的形态与三条待攻的补充条件。

**欠**：C8（门禁范围判不出来）；C13（checker 判定失效）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:344-359`（整段抄，未转述）**

```markdown
#### 已定项 12：冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具`

**定案**：**冲突 1：`checker 即规范` 与 `校验的两条路径不许共享工具` 直接打架。**
在用的不变量（条数的权威登记位在 [invariants/README.md](../../../crates/singlefs-checker/invariants/README.md) 文件头）若既是 checker 源码又是形式验证的 spec，就成了单点。
**可行的折中，而且本工程已经付过这笔钱**：checker 做两个方向——**遍历式**（从系统配置走下去）
与**扫描式**（全盘扫，靠「元数据块必须自描述」的块头重建），两者对同一条不变量独立判断，
共享的只有格式解析。

**射程**：管 checker 内部两个方向之间共享什么；checker 与实现之间共享什么归已定项 5。池级 checker 两个方向都有（`crates/singlefs-checker/src/walk.rs` 文件头）。

**依据**：
- 无实验：这是「checker 即规范」与校验路径独立之间的取舍，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：C410（checker 两个方向共享什么没有检查）：两个方向「共享的只有格式解析」这一条没有会红的检查（`.claude/gate.d/` 里没有判它的阶段）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:360-380`（整段抄，未转述）**

```markdown
#### 已定项 13：冲突 2：crash refinement 的 spec 与模型对拍的 model

**定案**：**冲突 2：crash refinement 的 spec 与模型对拍的 model 是同一个东西的两种写法。**
合并成一份 → 省一半工作量，但**直接违反独立性要求**；各写一份 → 名义独立，
但通常是同一个人同一周写的。缓解（只降低相关性、不消除）：让两份**在表述层面就不同**——
spec 写成「对镜像的状态谓词」，model 写成「op → 状态转移」，不一致时至少能互相定位。
**同源性要被观测**：统计「spec 与 model 不一致」的事件数及每次判定哪一方错；
若累计 ≥ 5 次不一致而判定「spec 错」的次数为 0，说明两者实际同源，应把两条腿合并、预算移给穷举。
判定分三格：spec 错 / model 错 / 条款欠一句（两边各照条款的一种读法，条款没写谁对）。一次不一致先在前提成立的域里重判，还在的才记；「条款欠一句」的事件先记账，不计入「累计 ≥ 5 次」，也不计入「spec 错」，条款补了那一句之后按补的句子再判一次、归到前两格。
这里的 model 指 O1（内存模型对拍（HashMap 理想文件系统））。O3（独立规约执行器） 能对同一（历史、崩溃点）交出判定行之前（第一版 Python 执行器 E170（O3规约执行器第一版） 已有，没接进崩溃点重放，C447（规约执行器没接进崩溃点重放与 RefFS 比对）），O3（独立规约执行器） × RefFS 这一对登记为空格、不计事件；O1（内存模型对拍（HashMap 理想文件系统）） 的崩溃判官不当替身；没立的对报「未立」，不报绿、不计入任何一格。

**射程**：缓解只降低相关性、不消除。已定项 1 撤下的那条原候选不受这条影响，理由在已定项 1 的射程。crash refinement 的代码是 `crates/singlefs-replay-prefix/src/lib.rs` 里的 Verus 核与它的 spec（取前缀、择根、扫环三截与择根和取前缀的合判推论，覆盖已定项 1 的 M1、M2 句法层与 M3 的两读和单调推论；M2 语义层与 M4 靠 E165（恢复重放有界穷举与变异杀伤界） 在界内判）；spec 不能对具体状态求值，「spec 与 model 不一致」的事件从哪来没定，计数器没有代码（C191（同源性计数器只盖了一对））。

**依据**：
- 无实验：同源性要靠将来 spec 与 model 的不一致事件计数观测，spec 与计数器都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 第三格「条款欠一句」：三方判决 `research/prompts/m3-formal-r1-main-verification.md` 第二节「F2 同源性」一行（攻方打中，推的）、`research/prompts/m3-formal-r2-main-verification.md` 第二节第 9 行、`research/prompts/m3-formal-r3-main-verification.md` 第二节同源性计数器一行，三轮站住；无实验：计数器仍没有事件来源。
- 「model 指 O1（内存模型对拍（HashMap 理想文件系统））」与没立的对登记为空格：三方判决 `research/prompts/m3-formal-r5-main-verification.md` 第二节同源性计数器那一格（两腿同向的部分）；`research/prompts/m3-formal-r4-main-verification.md`「用户定案」里替身对那一行（不算替身对）。

**欠**：C191（同源性计数器只盖了一对）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:381-399`（整段抄，未转述）**

```markdown
#### 已定项 14：与其他决策的连锁

**定案**：

| 决策 | 连锁点 |
|---|---|
| D4（校验和位置） | 根链差分把 Merkle 从「要付的代价」翻转成验证杠杆；但形式验证里哈希是 uninterpreted function，**碰撞抗性完全在验证范围之外，不许读成「验了 Merkle」** |
| D8（核心索引结构） | 日志结构节点、write buffer、key cache 三层都会绑架符号执行（循环上界、滞后窗口、缓存一致性），使「Yggdrasil 一分钟内」的口径在本工程不成立 |
| D9（加密） | AEAD 在 SMT 里不可解，只能建模成 uninterpreted function，**「nonce 重用泄漏明文」这个后果验不出来**，只剩「nonce 不重复」这条纯组合性质——而这条纯组合性质恰是这条腿最值钱的产出 |
| D11（索引节点要不要留消息缓冲区） | 已定项 1 取「留」；D8（核心索引结构） 已定项 1 取幂等完整值之后 checker 的 I-3.1（已分配统计对得上） 不必解析消息语义（D11（索引节点要不要留消息缓冲区） 据此判它的代价 1 在本工程不成立），冲突 1 不因此变成「无解」 |

**射程**：连锁的是形式验证与 checker 碰到这几条决策时的边界，不改那几条决策的条款；根链差分与冲突 1 本身归已定项 8、12。

**依据**：
- 无实验：连锁是从那几条决策的已定条款推出来的边界，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:400-412`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码按「什么时候跑」分两档，各住一个包，与池级 checker 一共三个名字：**harness 档**是 `crates/singlefs-harness`，单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），改了代码随时跑；包内再分轻重：debug 下单条跑到 60 秒及以上的是重，标 `#[ignore]`，单条用时看平常跑的时候量到的、不另外单独量，要跑随时跑，一律经内存包装；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。**checker 档**是 `crates/singlefs-checker-tier`，崩溃态枚举引擎与记录核对器、崩溃枚举用例经判定库的两段与它的计数、发现日志与工具链身份、一份长期判定库（块键只由内容定，几类测试各自要哪些块只当索引）与两段协议、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例，连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档里的崩溃点重放（门禁 checker-tier-crash-point-replay）分两档，checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 不分：**快档**是两趟：`cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（`--tests` 带上全部 `test = true` 的目标，装置二进制的内联单测也在里面），再一趟带 `tests/` 里 `cfg` 点到的特性（`--features <特性> --lib --test <挂特性的测试文件>…`，特性里有 `gpu` 的这一趟经 `scripts/crash/gpu-quiesce-run.sh` 起）（门禁 checker-tier-crash-point-replay 在整轮门禁与提交时都跑这两趟，第二趟是它的 feature-gated-tests 格），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（checker-tier-crash-point-replay `--full`），提交时不默认跑，由用户要求时跑，跑法按已定项 9 分层；崩溃放量判器整套上了 GPU，罩全部池级 checker 不变量（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被两档调用。依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ④ 条判，dev-dependencies 也算）；两档都要的一小段代码宁可各留一份。崩溃枚举用例一律写在 `crates/singlefs-checker-tier/tests/` 里（`scripts/crash/crash-case-check.py` 判），重型测试闸按包判：跑到 `singlefs-checker-tier` 的测试就是 checker 档。

**射程**：管 `crates/` 下代码与测试住哪个包、什么时候跑、闸按什么认、三个包怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑、住哪个包是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；三方 `verification-split-r1`（判决 `research/prompts/verification-split-r1-main-verification.md`）攻的是拆出 `singlefs-checker-tier` 之前的那一版，拆包之后的改动被攻过零轮。
- 用户定案（2026-10-01）：全量不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:413-428`（整段抄，未转述）**

```markdown
#### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数

**定案**：
- 分片：崩溃放量里按段登记的段（`.claude/gate.d/crash-point-replay-segments.tsv`，一行一段五列：项名、流名、段号、段写数、段写表摘要），整域态数装不进 u64 时按固定大小的片依次判；片大小由环境变量 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` 定，没设取 2^32 态；片起点是判定库里的续点（续点按段写表摘要与判器单元键寻址，项名、流名、段号只作显示）。整域装得进 u64 的登记段不分片，整域一次判。代码落点：`crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- 位宽：登记段的总态数、片起点、片尾与进度一律用 1280 位（10 个 u128）定宽无符号整数 `StateCount1280Bits`（`crates/singlefs-checker-tier/src/crash.rs:1640`，肢数是同一文件的 `STATE_COUNT_LIMBS`），算术全部 checked，不饱和、不截断；驱动行 `registered_segment_progress=` 的已判态数与总态数都写精确十进制，不写幂的形态。
- 登记表怎么交给引擎：引擎只经环境变量 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 读按段登记表（`crates/singlefs-checker-tier/src/crash_amplification.rs:14764`，在 `fn registered_segments_of_the_flow_from_the_environment` 里），没设就当一段都没登，没有别的退路。144 个单元的文件顺序写成 145 个、extent 下段长成两层的那条流（引擎流名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）的单元写段（322 次单元写）与记录段（290 次 journal 记录原地写）两段都登。

**射程**：管崩溃放量引擎对按段登记的段怎么分片、态数用什么整数记、登记表从哪里读；不管哪几段该登（由用户点头，写在登记表文件头）、不管登记段的域本身怎么展开（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10）、不管每个崩溃状态怎么定义（已定项 4）。没登记的段的态数、与整条流合起来的态数仍要装得进 u64，装不下在写库之前拒跑；登记段整域越过 1280 位同样在写库之前拒跑。片大小只许 1 到 2^32：两个 GPU 内核把片内序号当 u32 加到片起点上（`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）；轻型（`scripts/crash/crash-point-replay.sh --quick`）调用方没设片大小时设成 `LIGHT_REGISTERED_SEGMENT_SLICE_STATES`。展开上限盖不住又没登记的段，放量入口 `scripts/crash/crash-amplification.sh` 判红；崩溃点重放入口、放量入口与分项驱动开跑前核这份表再送给引擎。

**依据**：
- E161（崩溃放量的去重与分段耗时）「第 2 次重跑：取样域按 11 段的流重定（2026-10-05）」一节：段的全量态数与闭式、装置逐个数、crates 计划逐段相等，态数随段写数按幂长，撑的只是「长段整域装不进 u64」这个前提；片大小、位宽取值与登记表的读法是工程参数，用户定，它不撑。
- 用户定案，原话在变更史：分片与每片 2^32 态（`records/2026-10-11-崩溃放量两段与脚本审计.md` 第 96 行）、位宽 1280 位（同一份记录第 98、99 行）、两段都登（同一份记录第 92 行）。
- 分片与位宽是对判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10（按段登记）的补充，没经三方攻过。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:429-449`（整段抄，未转述）**

```markdown
#### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成

**定案**：
- 重型门禁、崩溃判定库与实验准入的每一份判定（崩溃状态块、快档与模型对拍的每条测试函数、crates 变异每行、广谱变异每条、形式化每条、QEMU 每档、litmus 每份、研究复跑每行、实验变异表每条、实验准入）沿不沿用，只看它的单元键：它依赖的函数单元各自去注释后的指纹，加闭包里各单元（含 mod 头）的 `#[cfg]`、`#[cfg_attr]`、`cfg!` 真引到的 cfg 原子、各带这一次目标下的真值（闭包没引到的原子不进键：带不带 `gpu` 特性编的两趟，闭包不提 `gpu` 的判定键相同、共用一份库），加工具链（含 cargo 解析后的生效构建环境与外部工具版本）、从 `Cargo.lock` 解析出的闭包真引到的依赖版本，加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）。函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。
- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。
- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。
- 每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。
- 改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。

**射程**：管重型门禁、崩溃判定库与实验准入的复用键；不管静态门禁「只判新加行」的改动范围（`scripts/gate/changed-paths.sh`，那是判据语义）与 preflight 的 `admission: inputs-changed`；不管枚举域（已定项 4）、分层与什么时候请用户跑全量（已定项 9）。镜像判定与给定状态判定的版本是判定定义那一组（CPU 参照判器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` 的闭包）的键，不论这一块在 CPU 还是 GPU 上判的：GPU 判器是加速路径，它的准入靠与定义那一组的对拍（快档的 GPU 与 CPU 对拍用例），它的单元只进块属性当出处，不进键、不分种类；显卡驱动、卡型、几张卡怎么分活同样留作块的属性或跑的参数，不进键。

**依据**：
- 用户定案（2026-10-15），原话在变更史与 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 用户定：提交与重跑无关，重跑只跟代码变化走」一节、「2026-10-15 批3」那几段。
- 用户定案（2026-10-17）：判定不与硬件、不与 CPU 还是 GPU 绑定，复用与剪枝不看设备；原话在变更史与 `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 叫停全量那几段。
- 主 agent 定案（2026-10-15）：仓根不进键与库、库目录只收绝对路径、cfg 成分只取闭包真引到的原子，出处同一份记录「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」定案第 1、2 条。
- 反证：E164（崩溃放量新设计的小范围功能测试） F4 量到只按函数取指纹、不连引用的常量一起收，漏判 V5 与 V11（V11 只改常量 `DATA_POINTER_BYTES`）：函数单元要连同引用的常量、类型、静态项与宏一起算。
- 盘上的数：`ledger.md`「一之二、盘上量到的」一节——只差名字的块各存各判 4100 块与 9679 块；整库一把的判法摘要一换累计多判 3 935 242 934 态，那次闭包里改动的 29 份文件有 24 份去注释后逐字相同：键带名字、按文件或整版取指纹，同一份判定会重判。
- 没有腿况记录：未经三方攻过；用户定改完代码开三轮攻方，只攻「剪枝是不是原子化的、键是不是只由函数单元组成」。

**欠**：C8（门禁范围判不出来）、C489（singlefs-core 的广谱变异对今天的代码没有结论）、C528（两条门禁路判的改动窗口不同）、C564（另外 6 条崩溃点重放全量没人跑）、C605（崩溃放量的复用与剪枝没有关掉再判的对照）、C630（提交时形式化那一道第 1 步停下的频率没量）、C635（上一次全量没有机器可读的标记）、C645（崩溃放量全量没有人能派）、C777（五个实验键在 stage-inputs.tsv 没有登记行）、C836（staged-green 挂的树不一定是被判的树）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:8-28`（整段抄，未转述）**

```markdown

| # | 分项 | 定案 |
|---|---|---|
| 1 | **crash refinement 先验哪一块** | 先验「一次发布 + 一次崩溃 + 一次恢复」这个封闭单元，第一个里程碑是任一 `jsn` 连续前缀不切开发布；挂载透明另立第四条性质；原候选 I-6.5（nonce 水位单调） 撤下 **状态：已定。** **进度：未实现。** |
| 2 | **事务层承不承诺串行提交** | 先串行，越过交点再开并行；将来并行提交时记录按事务号顺序追加 **状态：已定。** **进度：已实现。** |
| 3 | **自写参照实现（RefFS 形态）做到什么程度** | 做到极致取语义级：只在内存里按 POSIX 操作走，自己定义持久与崩溃后允许的状态，没有盘上格式；照 kb 另起、单独一个包、只收白名单输入 **状态：已定。** **进度：已实现。** |
| 4 | **崩溃点重放的崩溃状态集合怎么定义** | 写流按设备切段，段里每块有写的盘都被自己的屏障或 FUA 放行才关段；段内每次整写各取几态、任意组合，原地覆写多一态「新旧都读不出」；没持久放旧字节且真生成镜像；双写按设备各算 **状态：已定。** **进度：已实现。** |
| 5 | **checker 与被测实现允许共享什么** | 只共享一份从 kb 生成的常量模块，其余各写一份；判定宽度由 checker 自己探测再与系统配置比对，探不到报「声明值，未探测」 **状态：已定。** **进度：已实现。** |
| 6 | **三个互相独立的 oracle** | 内存模型对拍、独立解析器 + checker、独立规约执行器三个 oracle 各判一样；独立性要靠「三 oracle 合谋测试」验证，不许自称 **状态：已定。** **进度：未实现。** |
| 7 | **O2（独立解析器 + checker） 判什么** | O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员；需要第二个输入的检查归记录核对器（入参含枚举器给的持久集合），故意不给编号 **状态：已定。** **进度：已实现。** |
| 8 | **自研的两样** | Merkle 根链差分（相邻两代根递归比较，指针没变处不下钻）与代际增量 scrub（只遍历 birth generation 高于水位的块） **状态：已定。** **进度：未实现。** |
| 9 | **崩溃点重放怎么分层** | 三层都精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore 慢慢开；里程碑收尾与判器单元有变的那批提交前弹窗请用户跑全量档，红或不跑照样提交、记未结；先保失败可复现 **状态：已定。** **进度：已实现。** |
| 10 | **真正必须精确覆盖的 N 是什么** | 提交协议结构的等价类数，不是布局套数；落地是一个封闭的提交步骤枚举加穷尽 match **状态：已定。** **进度：已实现。** |
| 11 | **ignore 的节点什么时候开** | 否决按欠账轮转的账本；留下三条改管开的时机：确定性的覆盖记录、`T1`（常规层 挂钟预算） 是输入、判别力与覆盖分开检查 **状态：已定。** **进度：无对象。** |
| 12 | **冲突 1：checker 即规范与校验路径不许共享工具** | checker 做遍历与扫描两个方向，对同一条不变量各自独立判，共享的只有格式解析 **状态：已定。** **进度：已实现。** |
| 13 | **冲突 2：refinement 的 spec 与对拍的 model** | spec 与 model 在表述层面就写得不同；同源性要观测，≥ 5 次不一致而判 spec 错 0 次就合并、预算移给穷举 **状态：已定。** **进度：无对象。** |
| 14 | **与其他决策的连锁** | 哈希与 AEAD 在形式验证里只能当 uninterpreted function；日志结构三层绑架符号执行；checker 不必解析消息语义 **状态：已定。** **进度：无对象。** |
| 15 | **验证代码分两档** | harness 档（`singlefs-harness`）是单元与集成测试、随时跑；checker 档（`singlefs-checker-tier`）默认只在提交时跑快档，全量用户要求时跑 **状态：已定。** **进度：已实现。** |
| 16 | **按段登记的段怎么分片、态数用多宽** | 登记段整域装不进 u64 就按片依次判，片大小默认 2^32 态、片起点是库里的续点；态数一律用 1280 位定宽整数、算术全 checked；登记表只经环境变量读 **状态：已定。** **进度：未实现。** |
| 17 | **判定复用的单元是什么** | 剪枝的单元是函数：一份判定沿不沿用只看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本；按提交或树算改动、整文件原文、文件级与包级指纹、手写版本号、路径名字进键、整道标记一律不许 **状态：已定。** **进度：未实现。** |

```
