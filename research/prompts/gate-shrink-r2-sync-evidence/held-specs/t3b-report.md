# gate-t3b 写回规格报告（门禁去编号称呼，checks-owed.md 批 + 简称全表扫）

判决：`research/prompts/gate-shrink-r1-main-verification.md`「采纳的改法」第 5 条 + 用户定案（2026-09-28，见
`records/2026-09-28-门禁59号提速与双机分片.md` 第 73、88 行引用）；对照表：同文件「旧编号与现在的门禁」一节。
前一份规格/报告写法：`/tmp/claude-1000/gate-t3/spec.md`、`/tmp/claude-1000/gate-t3/report.md`。

## 规格文件

- `/tmp/claude-1000/gate-t3b/spec.json`
- `/tmp/claude-1000/gate-t3b/spec.md`

```
7a410069687defd7bef47ea5a293ebbf30e23bce5043e2b9e5fd613c6b359ca7  spec.json
bc78264b867df0076157f2a7d9adce14d0b9a772997324eb17483b65ca62bf5c  spec.md
```

134 条，覆盖 3 个文件：`.claude/kb/checks-owed.md`（132 条）、`.claude/kb/layout/01-first-txn.md`（1 条）、
`.claude/kb/experiments/142-新池新建文件的干跑.md`（1 条，后两条是把 C485 简称本身换成新登记位一致的写法，
gate-t3 那一轮已把这两处旁边的门禁引用换过、留了 C485 原样，这一轮补上）。

## kb-spec-check.py 末行

针对真仓跑（原样，末尾几行）：

```
  ✗ 第 89 条（.claude/kb/checks-owed.md）：编号引用不对：C114（declared-counts 格只扫索引页）→ 登记位写的是「C114（10 号阶段第 3 项只扫索引页）」
  ✗ 第 105 条（.claude/kb/checks-owed.md）：编号引用不对：C520（decision-links 格只认状态段恰好是已跑）→ 登记位写的是「C520（75 号只认状态段恰好是已跑）」
  ✗ 第 129 条（.claude/kb/checks-owed.md）：编号引用不对：C485（segment-registry 格不读活代码，段序列变了照样绿）→ 登记位写的是「C485（门禁 52 号不读活代码，段序列变了它照样绿）」
  ✗ 第 130 条（.claude/kb/checks-owed.md）：编号引用不对：C487（device-streams 格拿活代码与不跟踪它的模型比）→ 登记位写的是「C487（门禁 55 号拿活代码与不跟踪它的模型比）」
  ✗ 第 131 条（.claude/kb/checks-owed.md）：编号引用不对：C520（decision-links 格只认状态段恰好是已跑）→ 登记位写的是「C520（75 号只认状态段恰好是已跑）」
  ✗ 第 132 条（.claude/kb/checks-owed.md）：编号引用不对：C114（declared-counts 格只扫索引页）→ 登记位写的是「C114（10 号阶段第 3 项只扫索引页）」
  ✗ 第 133 条（.claude/kb/layout/01-first-txn.md）：编号引用不对：C485（segment-registry 格不读活代码，段序列变了照样绿）→ 登记位写的是「C485（门禁 52 号不读活代码，段序列变了它照样绿）」
  ✗ 第 134 条（.claude/kb/experiments/142-新池新建文件的干跑.md）：编号引用不对：C485（segment-registry 格不读活代码，段序列变了照样绿）→ 登记位写的是「C485（门禁 52 号不读活代码，段序列变了它照样绿）」
  ✗ 规格 134 条里有 8 处会让门禁红（逐处列在上面）
  → 怎么办：按每处改规格原文再跑一次这个脚本；改不动的（要判断的）交主 agent，不派书记员
```

**这 8 处不是规格写错，是检查②（`reference_problems`）的判据本身与「重命名一个已登记简称」这件事结构性撞车**：
`load_registry()` 只在跑之前读一次真仓磁盘，取的是**改之前**的登记位；这 8 条引用的都是**这份规格自己同批改掉**的
C114 / C520 / C485 / C487 的登记行（它们的登记位分别在 `checks-owed.md` 第 543、415、509、402 行，规格里也在改这四行本身，
在 132/134 条清单里），因此在「改之前」的快照上必然判「登记位写的是旧名」。这四个简称的**注册行**本身没有被判红
（`reference_problems` 对表格首列 `token == first_cell` 的行豁免，工具头部注释①也写了「表格首列是编号不算登记」）。

**旁证（不是硬判据）**：把这份规格整批套到一份仅含 `.claude/kb/` 的草稿副本上现改一遍（`/tmp/claude-1000/gate-t3b/scratch-root2`，
已在交回前删掉），134 条全部命中且恰好 1 次，套完之后再扫这 11 个简称，`.claude/kb/` 下再也找不到任何一处还写着旧号
（除下面「不改」列的 `decisions-history.md`、`experiments-history.md`、`milestone/02-second-txn.md` 三处不在这份规格射程内），
说明改完之后是内部自洽的：C114/C520/C485/C487 的登记行与全部引用行同时落地就没有中间不一致的状态。

## 覆盖范围（checks-owed.md 132 条）

按判决对照表把「旧编号／`.claude/gate.d/NN-*.sh` 路径」换成「门禁 <文件名去掉 .sh>」或「门禁 <文件名> 的 <格名> 格」；
`54-layer0-replay` 按判决单独写全名，不进「门禁 <文件名去掉 .sh>」这个通式（它本身文件名就带号）。
格名逐个现查各门禁文件头 `# gate-cell:` 行、`git show HEAD:.claude/gate.d/<旧文件名>.sh` 的 `# gate-stage:` 行与
`records/2026-09-28-门禁59号提速与双机分片.md`「旧编号与现在的门禁」表交叉核对，不是照抄前一份规格。

现查覆盖：`.claude/kb/checks-owed.md` 正文（1–610 行，「## 历史版本」之前）里带旧号/旧路径的候选行 129 条，
132 条最终规格里含 128 条落在这个区间（1 条——第 364 行 C436 那处「23 号」——判定超出这一轮对照表射程，见「要主 agent 判的点」）；
另外 4 条是「## 历史版本」节里 C114 / C520 / C485 / C487 简称本身的引用（第 617、628、651、652 行），按定义第 2 条允许改动。

**其中一处依赖直接的文字证据而不是我自己的语义匹配**：`.claude/kb/checks-owed.md` 第 468 行（C589 那一行，2026-09-28
当天写的）原文已经写着「30 号 `status-sync` 格只比对变更史每节顶上的现状行……」——把旧号与新格名并列写在一起，
与 `cell_status_sync_check()` 的实际判法（`.claude/gate.d/doc-decisions.sh` 第 2809 行起，比对的正是「变更史各节顶上的现状与索引表」）
逐字对得上，这条比对照旧`gate-stage`docstring 猜的更可靠，所以规格按「30 → status-sync」写，不是「30 → entry-added」
（`30-decision-history.sh`自己的 `gate-stage` 行写的是「决策变更留痕」，读起来更像 `entry-added`；这份规格没有再核实
`entry-added`/`shape`两格到底对应旧 30/48/49 里的哪个——因为除了第 468、469 行，`48号`、`49号`在整份文件里一次都没被引用，
不影响这份规格要改的任何一行）。

## 不改（逐处列理由）

- `.claude/kb/checks-owed.md`「## 历史版本」节（第 611–1261 行）**正文散文**：kb-discipline 第 8 条、这份判决第 2 条
  「不改」明写除简称外不动；本规格只动了这一节里 4 处「Cnnn（简称）」token 本身（617/628/651/652 行），周围叙事一个字没碰。
- `.claude/kb/decisions-history.md`、`.claude/kb/experiments-history.md`：判决明确排除（除简称外）。这两份文件本身还有一个
  与本轮无关但必须报的异常，见下「要主 agent 判的点」第 1 条——按这个异常，这两份文件里 C112（11 处）、C487（2 处）、
  C485（2 处）的简称这一轮**没有**动，列进「要主 agent 判的点」而不是硬塞进规格。
- `.claude/kb/milestone/02-second-txn.md`：判决/派发提示明写「不碰的文件（另一份规格在做）」；里面有 C487（4 处）、
  C485（2 处）的旧简称，交那份规格处理，这份没有触碰。
- `.claude/kb/checks-owed.md` 第 415/416 行「三方判决 `research/prompts/gate-fix-forks-r3-main-verification.md`
  「75 号 ⑤ 后一半、10 号并进 ⑨」一节」与第 420 行「三方判决 `research/prompts/gate-fix-forks-r2-main-verification.md`
  「52 号找不到脚本、31 号找不到生成器退 1」一节」：`「」`内是那两份判决文件里小节标题的逐字引用，不是这份文件自己的现状陈述；
  按 `.claude/rules/path-moves.md`「说搬迁这件事本身的那一行保留旧路径」同一判据——把旧号换了，这句话就不再是那份判决
  文件里真实存在的小节标题，成假引用——两处 `「」` 内的旧号原样留着，`「」`外同一行别的旧号（例如 420 行「31、52 号的写法」）照改。
- `.claude/kb/checks-owed.md` 第 397 行「`.claude/batch-scope` 里写的理由是「同 54 号」」：`「」`内是逐字引用
  `.claude/batch-scope` 那个登记文件里**当时真实写着的理由字符串**，同一判据——`.claude/batch-scope` 本身不在这份
  规格的写范围里，这句引用换了就不是那个文件当时写的那句话，原样留着；`「」`外同一行的「55 号那一行」「`.claude/gate.d/…`」
  照改。
- `.claude/kb/checks-owed.md` 第 558 行「门禁 46、47、63 号」里的 **46**：现查 `git log --all --diff-filter=D --name-only`，
  `.claude/gate.d/46-write-hook.sh` 在 HEAD 之前更早一轮就已经被删/改名，不在这份判决的对照表（对照表只覆盖 HEAD 上
  10–99 现存的那批），我没有依据替它定新名字；这一行只改了 47、63（在对照表里），46 原样保留并加了「（未在这轮改名
  范围内）」的括注防止读的人误以为它也已经改名，列进「要主 agent 判的点」。
- 同一行第 364 行（C436）：「门禁 23 号只判 `[文字](路径)` 形态的链接」——现查 `.claude/gate.d/` 在 HEAD 上没有
  `23-*.sh`（`old_files.txt` 现查确认），`git-similar` 线索指向 `.claude/singlefs-ai-sop/scripts/link-targets.py`
  （10-kb-rot.sh 旧文件头写过 `gate-similar: link-targets.py`），但那是**共享 SOP 脚本、不是项目 `.claude/gate.d/`
  下的编号阶段**，不在这份判决要去编号的射程里，也没有依据替它定新名字；一个字没动，列进「要主 agent 判的点」。

## kb 外的引用（给主 agent 改，这份没有动）

C112 / C487 / C520 / C522 / C523 / C524 / C526 / C530 / C590 / C485 / C114 这 11 个简称，`.claude/kb/` 之外
（`records/`、`research/prompts/`）现查还有这些文件里用着旧简称（文件、命中次数，`grep -c "Cxxx（" <文件>` 现数）：

| 编号 | 文件 | 处数 |
|---|---|---|
| C112 | （`.claude/kb/` 外未命中） | — |
| C487 | `records/2026-09-28-里程碑三第十项清单.md` | 2 |
| C487 | `research/prompts/m3-item10-owed-verify-b-verdicts.md` | 1 |
| C487 | `research/prompts/m3-item10-owed-verify-b-report.md` | 1 |
| C487 | `research/prompts/_m2-treesplit-r1-background.md` | 1 |
| C487 | `research/prompts/_m2-treesplit-r1-appendix.md` | 1 |
| C487 | `research/prompts/_m2-lines234-code-r1-body.md` | 1 |
| C487 | `research/prompts/_changelog-format-r1-background.md` | 1 |
| C487 | `research/prompts/_changelog-format-r1-appendix.md` | 1 |
| C487 | `research/prompts/_abandoned-floor-r1-background.md` | 1 |
| C487 | `research/prompts/_abandoned-floor-r1-appendix.md` | 1 |
| C487 | `research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/checks-owed.md` | 1（**冻结快照，见下**） |
| C520 | `research/prompts/gate-audit-judge-all.md` | 1 |
| C520 | `research/prompts/_changelog-format-r1-background.md` | 1 |
| C520 | `research/prompts/_changelog-format-r1-appendix.md` | 1 |
| C520 | `research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/checks-owed.md` | 2（**冻结快照**） |
| C590 | `research/prompts/changelog-format-r1-main-verification.md` | 1 |
| C485 | `records/2026-09-22-并行线二三四与捶打.md` | 1 |
| C485 | `records/2026-09-28-里程碑三第十项清单.md` | 2 |
| C485 | `research/prompts/m3-item10-owed-verify-b-verdicts.md` | 1 |
| C485 | `research/prompts/_gate-shrink-r1-appendix.md` | 1 |
| C485 | `research/prompts/_gate-shrink-r1-background.md` | 1 |
| C485 | `research/prompts/_m2-lines234-code-r1-body.md` | 1 |
| C485 | `research/prompts/_c245-r3-appendix.md` | 1 |
| C485 | `research/prompts/_c245-r3-background.md` | 1 |
| C485 | `research/prompts/_changelog-format-r1-background.md` | 2 |
| C485 | `research/prompts/_changelog-format-r1-appendix.md` | 2 |
| C485 | `research/prompts/_abandoned-floor-r1-background.md` | 1 |
| C485 | `research/prompts/_abandoned-floor-r1-appendix.md` | 1 |
| C485 | `research/prompts/e142-r19-prereg.md` | 1 |
| C485 | `research/prompts/e142-r19-runner-evidence/step0/section2.md` | 1 |
| C485 | `research/prompts/e142-r19-runner-evidence/step0/baseline-earlier-reconstructed.md` | 1 |
| C485 | `research/prompts/e142-r19-runner-evidence/step0/baseline-clauses-step0.md` | 1 |
| C485 | `research/prompts/e142-r19-runner-evidence/step0/base-earlier.md` | 1 |
| C485 | `research/prompts/e142-r19-runner-evidence/step0/baseline-clauses-step2.md` | 1 |
| C485 | `research/prompts/gate-shrink-r1-inventory/report.md` | 1 |
| C114 | `records/2026-09-05-SOP剥离轮.md` | 1 |
| C114 | `research/prompts/gate-audit-judge-all.md` | 1 |
| C114 | `research/prompts/_changelog-format-r1-background.md` | 1 |
| C114 | `research/prompts/_changelog-format-r1-appendix.md` | 1 |
| C114 | `research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/checks-owed.md` | 2（**冻结快照**） |
| C522/C523/C524/C526/C530 | （`.claude/kb/` 外未命中） | — |

**`research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/checks-owed.md` 不建议改**：路径里的
`kb-at-start` 说明这是某一轮三方论证开工时冻结下来的 `checks-owed.md` 快照，属于 `evidence-discipline.md`
「原样保存的证据不许事后改」管的那一类，是不是要改由主 agent 按那一轮是不是已经出轮判断，这份规格没有替它定。

`_*-background.md` / `_*-appendix.md` 这一批（下划线开头，`_m2-treesplit-r1-*`、`_changelog-format-r1-*`、
`_abandoned-floor-r1-*`、`_gate-shrink-r1-*`、`_c245-r3-*`）看着像是已归档三方轮次的材料（命名与
`.claude/agent-common.md`「找不到历史实验的数据……去 git log 里看」一节描述的「归档只写文件名」现象类似），
是不是也算冻结证据、要不要改，这份规格没有判断，一并列给主 agent。

`records/*.md` 与 `research/prompts/gate-audit-judge-all.md`、`m3-item10-owed-verify-b-*.md`、
`changelog-format-r1-main-verification.md`、`gate-shrink-r1-inventory/report.md`、`e142-r19-*` 这几份看起来是
仍在参考的记录/判决/实验材料，不像冻结证据，本次没有改是因为超出「只写 checks-owed.md 与两处 kb 引用」的写范围。

## 要主 agent 判的点

1. **`.claude/kb/decisions-history.md` 与 `.claude/kb/experiments-history.md` 工作区里异常膨胀，怀疑是别的会话
   未提交的坏改动，建议先核实再决定要不要动它们里的 C112/C487/C485 简称**：现查
   `git show HEAD:.claude/kb/decisions-history.md | wc -l` = 383 行，工作区 `wc -l` = 173769 行（约 454 倍）；
   `experiments-history.md` 现查 `git show HEAD:.claude/kb/experiments-history.md | wc -l` = 3244 行，
   工作区 `wc -l` = 10345 行（约 3.2 倍）。抽查发现同一段文字（含 C112 的那一行）在 `decisions-history.md` 里逐字重复了
   11 次，行号相隔均匀（20119、35557、52260、62110、89011、95349、115084、129711、136978、146523、166842），
   像是某种批处理脚本重复追加同一段。**这份规格没有碰这两个文件**：按同一份旧串在文件里重复出现多次，做不到
   kb-spec-check.py 要求的「恰好 1 次」，若真要改需要先弄清膨胀的成因（是不是该整份回退到 HEAD 或某个更早的干净提交）。
2. **8 处 kb-spec-check.py 判红**（第 89、105、129、130、131、132、133、134 条）：结构性地由「同一份规格里既改一个
   C 编号的登记简称、又改它自己的引用」造成，见「kb-spec-check.py 末行」一节的解释与草稿仓副本套用验证；建议主 agent
   要么接受整份规格一次性交给 `kb-scribe`（`replace-batch.py` 是按「旧串→新串」逐条找换、不依时间顺序，套完就是
   自洽状态，我已经在草稿副本上验证过），要么拆成两次派发（先只派登记行那 4 条改名，跑一次门禁/doc-lint 确认 C114/
   C520/C485/C487 的登记位已经翻新，再派剩下 130 条），由主 agent 定要哪一种。
3. **第 558 行「46 号」**、**第 364 行「23 号」**：均确认是这份判决对照表覆盖不到的旧号（46 在 HEAD 之前已经不存在于
   `.claude/gate.d/`；23 更早，疑似对应共享脚本 `link-targets.py` 而不是项目本地编号阶段），这份规格没有替它们定新
   名字，原样留着（46 那处加了括注说明未改），要不要另查它们历史上到底是什么、现在该怎么称呼，交主 agent。
4. **`.claude/kb/checks-owed.md` 第 468 行（C589）的「30 号」到底对不对应今天的 `status-sync` 格**：这份规格采信
   了第 468 行原文已经把旧号与新格名并列写在一起、且与 `cell_status_sync_check()` 实际判法逐字吻合这条证据；但
   `30-decision-history.sh` 自己旧文件头的 `# gate-stage:` 描述（「决策变更留痕」）语义上更像新格 `entry-added`
   而不是 `status-sync`。两条证据不一致，这份规格按「文字直接证据」定了 30→status-sync（只影响第 468/469 两行，
   `48号`、`49号`本身整份文件一次都没被引用，不影响别处），如果这个判断错了，请主 agent 复核。
5. **`kb 外的引用」一节列出的全部行**，是不是要跟着这一轮一起改、哪些算冻结证据不能碰，交主 agent 判断并派发（这份
   定义规定我只能写 `.claude/kb/**` 与草稿目录，够不到那些文件）。

## 没做什么

- 没写 kb 本身，没跑 kb 门禁阶段（doc-decisions.sh、doc-registries.sh 等），按定义规格与写回分离。
- 没判定案对不对，没做规格外的判断；判决点名的 54-layer0-replay 命名照判决原文写全名，其余按对照表与各门禁文件头
  `# gate-cell:` 行逐个核对，不是照抄前一份规格拍脑袋。
- 没有替「要主 agent 判的点」列的 5 条自己选答案，也没有为了让 kb-spec-check.py 变绿而扭曲规格原意（8 处红照原样
  交回，附结构性原因与旁证）。
- 没有动 `.claude/kb/decisions-history.md`、`.claude/kb/experiments-history.md`、`.claude/kb/milestone/02-second-txn.md`
  三份文件（按判决与派发提示排除，也因前两份文件的膨胀异常不敢碰）。
- 没有动 `.claude/kb/` 之外的任何文件；`kb 外的引用」一节的清单只是现查列出，没有替它们写规格、没有派生判断。
- 删了草稿目录里两份临时仓副本（`scratch-root`、`scratch-root2`，各约几十 KB，只含 `.claude/kb/`）用于验证内部一致性，
  交回前已 `rm -rf`。
