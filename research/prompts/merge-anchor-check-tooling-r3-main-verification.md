# merge-anchor-check-tooling-r3：判决（2026-11-21）

正文 `research/prompts/_merge-anchor-check-tooling-r3-body.md`，开工快照 `research/prompts/merge-anchor-check-tooling-r3-snapshot/`。这一题治理文件轮第三轮（最后一轮），攻第二轮判决 `research/prompts/merge-anchor-check-tooling-r2-main-verification.md`「三、改法」六条的修补；前两轮判决 `research/prompts/merge-anchor-check-tooling-r1-main-verification.md`、`research/prompts/merge-anchor-check-tooling-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/merge-anchor-check-tooling-r3-sonnet-output.md` | 六条修补五条做到、改法 4 不成立（`scripts/gate/apply_writer_patch_plan.py:49` 仍写单个数）；第二节第 1 条打中：合并补丁带拷贝范围外的文件段（拒算表、角色表）时合入前 `--dry-run` 退 2「没做成」，出路做不成；真合不跑静态核、`--dry-run` 的出路与合入那一段相反（推的）；其余四条成立 |
| 攻方（Opus） | `research/prompts/merge-anchor-check-tooling-r3-opus-output.md` | 打中：快核排在 `--dry-run` 之前、`--dry-run` 之后别的会话追加的行看不见、两棵树不是同一刻取的（改了又改回被放行、补丁前就红的被修好反被判新红）、真合退 2 之后的出路不含 `--dry-run`、`--dry-run` 几种结局没出路、技能文件没写 `--dry-run`、自证分不出两处改坏；15 GiB 内存、`--skip-apply` 三格、`--dry-run` 七格、开关四条与改了红格的六条、样本登记表判成立 |

核查：
- 开工快照七份在仓根 `sha256sum -c research/prompts/merge-anchor-check-tooling-r3-snapshot/judged-sha256.txt` 都 OK（腿跑着时没被改），引文直接在仓根核：`python3 scripts/agents/cite-check.py research/prompts/merge-anchor-check-tooling-r3-sonnet-output.md research/prompts/merge-anchor-check-tooling-r3-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_merge-anchor-check-tooling-r3-body.md`：「核了 31 处引文、77 处裸行号，对上 31 处，没判 77 处（其中裸行号没有引文可比 77 处）」。判决用到的裸行号主 agent 现读过：`scripts/gate/merge-and-verify-items-lib.sh` 第 840–880 行（合入那一段：核主工作区没变、核变异行两条排在 `--dry-run` 之前，真合退 2 之后写「重做核的那几条、再交同一条」）、`scripts/gate/apply-writer-patch.py` 第 84–88 行与第 1010–1018 行（真打不跑静态核，只在 `dry_run` 那一支调 `static_check_report`）、`scripts/gate/apply_writer_patch_plan.py` 第 44–50 行（`STATIC_CHECK_DECLARED` 只另声明 `litmus` 与 `.claude/gate.d/stage-inputs.tsv`；第 49 行注释写「共约 423 秒、内存峰值 15.20 GiB」）与第 1095–1130 行（补丁之后那一棵从拷贝范围拷出再 `git apply`，没打补丁那一趟直接对 `root` 跑），`scripts/gate/merge-and-verify.sh` 第 2177–2192 行（合入后验证自己的锚点核把拷贝范围外的补丁段滤掉）。
- 正推腿第三节 B：`task-server/task submit -- python3 scripts/hosts/lib_repository_copy.py entries /home/fy5090/code/singlefs --test-inputs --with litmus=变异行可以锚在litmus下 --with .claude/gate.d/stage-inputs.tsv=static-check读登记表` `type=succeeded exit=0`「REPOSITORY_COPY entries=32」，32 条里没有 `.claude/gate.d/refused-tests.tsv` 与 `.claude/gate.d/crate-roles.tsv`。
- 正推腿第三节 C：探针补丁（`probe-refused/crates.patch`，sha256 与腿报告相同，往拒算表末尾加一行）拷到主 agent 草稿目录交 `task-server/task submit -- python3 scripts/gate/apply-writer-patch.py --dry-run <主 agent 草稿目录>/probe-refused` `type=command-failed exit=2`，用时 4.089 秒：「✗ 变异锚点静态核没做成：补丁在临时树上套不上：error: .claude/gate.d/refused-tests.tsv: No such file or directory」，出路「改好再跑一次 --dry-run」。拒算补丁目录照 `.claude/main-agent.md`「合入后验证之后判新出现的拒算」接在一批补丁的最后，这一形每一批带拒算行的合入都会撞上。
- 攻方腿的样本补丁（`attack3.patch`，sha256 f3802ef9… 与腿报告相同）拷到主 agent 草稿目录，在主 agent 用 `scripts/hosts/minimal-copy.py` 建的副本上 `git apply -p2` 退 0，三份 sha256 与腿报告 D/new 三行相同；串行交四趟 `task-server/task submit --cwd <副本> -- env ATTACK3_CELLS=1 [ATTACK3_MUTATION=<名>] python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`：
  - R0（不改坏）`type=command-failed exit=1`：「✗ 自检：」0 行；红的六格：window-fingerprint（「--dry-run 之后、合的那一条之前没有一条核退非 0（核主工作区没变的那一条排在 --dry-run 之前，此刻重跑退 1）」）、redo-loop、dry-run-way-out-outcomes、window-table-append（「合进去之后那一行原文命中 0 次」）、transient-mask（「--dry-run 照补丁前就红的放行（退 0），合进去之后那一行命中 0 次」）、baseline-skew（「把它算成这一批新弄断的、退 2」）；不该红的 dry-run-way-out-cost、skip-none-not-red 两格是「·」行。
  - R1（`skip-none-records-red`）`exit=1`、「✗ 自检：」0 行，比 R0 多红 skip-none-not-red（「实际两趟退 3 与 1」）；R2（`skip-none-reason-dropped`）`exit=1`、「✗ 自检：」0 行，多红 skip-none-not-red（「实际两趟退 3 与 3」，原因那一句没了）；R3（`dry-run-way-out-dropped`）`exit=1`、「✗ 自检：」0 行，多红 dry-run-way-out-cost。三处改坏现有自证都分不出。副本复跑完当场删了。
- `merge-and-verify.sh` 的开关复跑（`bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh`，176 条）写判决之后出了结局 `type=command-failed exit=1`：「✗ 脚本与钩子的弄坏开关复跑：3 处不对（复跑了 176 条、1 份）」，现判 177、复用 0、拒算 0。三处（明细拷在主 agent 草稿目录 `break-switch-replay-25197-mismatches.txt`）：`verify-in-main-tree` 多红了合入前 `--dry-run` 七格里的六格（「没点名」，如「✗ 自检：合入前的 --dry-run：别的会话开跑之后追加的、锚在这一批改到的文件上的变异行应当拦住：…实际退 0」）；`registration-tables-refused` 多红了「合入前的 --dry-run：这一批登记崩溃枚举用例、开跑之后别的会话追加了点名它的变异行应当拦住」那一格（「没点名」，实际退 3）；`anchor-created-files-kept` 点名的「变异锚点核过」没红（另一项「变异锚点核：补丁只新建一份文件、不碰变异行，应当过」红了）。三条的开工前后键都变过（「组成清单有 3 处不同」，变的是 `.claude/gate.d`、`scripts/experiment-apparatus` 两个数据目录与 `scripts/mutation/` 下一处，别的会话跑着时改的），不是判红的原因。

## 二、判决

| # | 腿报的 | 判 | 依据 |
|---|---|---|---|
| 1 | 合并补丁带拷贝范围外的文件段（拒算表、角色表）时合入前 `--dry-run` 退 2「没做成」，出路做不成（正推 1b） | **采，量过** | 第三节 B、C 两条；拒算补丁目录是合入流程的一部分 |
| 2 | 两棵树不是同一刻取的：补丁之后那一棵先拷先跑，没打补丁那一趟后在活的主工作区上跑；改了又改回的被当成补丁前就红而放行，补丁前就红的被修好反被判新红（攻方 transient-mask、baseline-skew） | **采，量过** | R0 两格；`scripts/gate/apply_writer_patch_plan.py` 第 1119–1122 行 |
| 3 | 快核排在 `--dry-run` 之前，`--dry-run` 跑着的几百秒里别的会话改了这一批改到的文件、或往表里追加锚在这一批改到的文件上的行，合之前没有一条核看它（攻方 window-fingerprint、window-table-append；正推 1a） | **采，量过** | R0 两格；真打不跑静态核（`scripts/gate/apply-writer-patch.py` 第 86 行） |
| 4 | 真合退 2 等完之后的出路只写「重做核的那几条」，不含 `--dry-run`；`--dry-run` 判不了、没做成、被停、撞上限几种结局没有出路，落到「整批重交」；`--dry-run` 自带「改好再跑一次」与合入那一段的「整批重交」相反（攻方 redo-loop、dry-run-way-out-outcomes；正推 1c） | **采，量过** | R0 两格 |
| 5 | 自证分不出「本次未核」改成记红、不写原因，与 `--dry-run` 那一条出路（连代价）整行删掉（攻方） | **采，量过** | R1–R3 |
| 6 | `.claude/skills/merge-verify/SKILL.md` 第 3 条没写合入前的 `--dry-run`（攻方） | **采** | 读文件，grep 零命中 |
| 7 | `scripts/gate/apply_writer_patch_plan.py` 第 49 行注释仍写单个数（正推改法 4） | **采** | 第二轮判决改法第 4 条要三处写成范围，这一处不在被换的五份里 |
| 8 | 15 GiB 撞上限、`--skip-apply` 三格、`--dry-run` 七格走到了锚点静态核、样本登记表一行够 | **成立**（两条腿都判） | 腿报告各自第二节；内存由服务撞上限乘 1.5 重排 |
| 9 | 新开关四条与改了红格的六条红在点名那一格（两条腿读码推成立） | **采，量过**：三处不对 | 开关复跑「3 处不对」：`verify-in-main-tree`、`registration-tables-refused` 让第二轮新加的合入前 `--dry-run` 格跟着红而开关表没点名，`anchor-created-files-kept` 点名的一项没红 |

## 三、改法

都由工具员一次改完；这一轮是最后一轮，下面每一条都**零轮**（没被攻过），交用户时写明。这一题只改协作工具、不碰产品代码，`.claude/rules/three-way-inference.md`「腿提的、判决里标「零轮」的改法，写进产品代码之前先在探针里对它做 A/B」只管产品代码，不适用；代替它的是攻方腿 R0 样本格：改之前六格红，改完照第 4 条收进自证、该放行的放行、该拦的拦。

1. 判决第 1、2 条（`scripts/gate/apply_writer_patch_plan.py` 的 `build_patched_tree` 与 `static_check_report`，零轮）：补丁前后两棵树从同一刻拷出——先拷一棵补丁之前的树（拷贝范围 + `STATIC_CHECK_DECLARED` + 补丁改到的每个文件在主工作区里此刻那一份，补丁新建的不拷），再把它整棵拷成补丁之后那一棵、套补丁；两趟 static-check 都对拷出来的树跑，不对活的 `root` 跑。拷贝范围外的文件段因此套得上，判不判仍只看 static-check 输出。
2. 判决第 3 条（`scripts/gate/merge-and-verify-items-lib.sh` 合入那一段，零轮）：次序改成 ① 记「`crates/mutations.tsv` 里锚在 `crates.patch` 改到的文件上的变异行」（只读、秒级，写进日志目录）→ ② 合入前 `--dry-run` → ③ 照 ① 同一条命令重算、与 ① diff → ④ 核主工作区没变、核要换掉删掉的变异行没被改（两条快核挪到 `--dry-run` 之后）→ ⑤ 真合。③ 对不上不整批重交：从 ① 重做（表变了，再核一遍就够）；④ 退非 0 照旧整批重交。
3. 判决第 4 条（同一份，零轮）：`--dry-run` 各种结局分开写出路——退 2 点名「变异表有 N 处静态核红」的整批重交；退 2 写「没做成」「判不了」的照它点名的原因办（工具的毛病当场报给管它的会话），办完从 ① 重做；被停、撞内存上限这类不是 2 的照原样再交同一条 `--dry-run`；真合退 2 等完（开工快照、同仓在换）与三方合并冲突、核与换之间被改了的，从 ① 重做到 ⑤。`--dry-run` 自己那句「改好再跑一次」只在合入那一段之外适用，合入那一段打的出路以这一段为准。
4. 判决第 5 条（`scripts/gate/merge-and-verify-selftest-lib.sh`，零轮）：攻方腿 `attack3.patch` 的六格照改法第 1–3 条改成平常就跑、该拦的拦（window-fingerprint、window-table-append 由 ③④ 拦；redo-loop、dry-run-way-out-outcomes 判出路写全）、该放行的放行（transient-mask、baseline-skew 两棵树同一刻取之后照实判）；skip-none-not-red 那一格改成平常就跑、判退出码与原因那一句都在；dry-run-way-out-cost 一格平常就跑。再加一格：合并补丁带拷贝范围外的文件段（拒算表一行）时 `--dry-run` 照常判、不退「没做成」。三处改坏与改法第 1–3 条各登弄坏开关、各红在点名那一格。`scripts/gate/apply-writer-patch.py` 自证同样加两格：补丁改拷贝范围外的文件时照常判；没打补丁那一棵不是此刻的活树（造一个跑着时改了又改回的假 static-check），各配开关。
5. 判决第 7 条（零轮）：`scripts/gate/apply_writer_patch_plan.py` 第 49 行注释写成范围（真仓一趟两次 static-check 约 400–740 秒、峰值约 15 GiB）。
6. 判决第 6 条（零轮）：`.claude/skills/merge-verify/SKILL.md` 第 3 条照改法第 2、3 条写出 ① 到 ⑤ 的次序与各结局的出路（这一份另一个会话今天改过，只做定点替换）。
7. 判决第 9 条（`scripts/gate/merge-and-verify.sh` 文件头开关表，零轮）：`verify-in-main-tree`、`registration-tables-refused` 两条的点名照实补上跟着红的合入前 `--dry-run` 那几格（照改法第 2–4 条改完之后的格名与红法重核）；`anchor-created-files-kept` 点名的「变异锚点核过」照它此刻真红的那一句改，或改开关让它真弄坏点名那一处。改完与改法第 4 条新加的开关一起重跑开关复跑，「点名的格与实际红的格逐格相同」。

改完：两份自证与两份开关复跑由主 agent 交；改法第 2–4 条换上之前告诉改槽位那一件的会话改了哪几行。

## 交用户的

这一轮是最后一轮，改法六条都是零轮（被攻过零轮），收进自证样本格当会红的量；不另开第四轮。

## 这一轮被判过的文件

- scripts/gate/merge-and-verify.sh：判了合入后验证自己的锚点核滤掉拷贝范围外的补丁段、`--dry-run` 跟不上（判决第 1 条），`--skip-apply` 不给补丁目录那一行判成立
- scripts/gate/merge-and-verify-steps-lib.sh：判了「本次未核」那一行改坏分不出（判决第 5 条）与耗时注释写成范围
- scripts/gate/merge-and-verify-items-lib.sh：判了合入那一段快核与 `--dry-run` 的次序（判决第 3 条）与各结局的出路（判决第 4 条）
- scripts/gate/merge-and-verify-selftest-lib.sh：判了 `--dry-run` 七格走到锚点静态核（成立）与三处改坏分不出（判决第 5 条）
- scripts/gate/apply-writer-patch.py：判了真打不跑静态核（判决第 3 条）与自证样本树的登记表一行够（成立）
- research/prompts/_merge-anchor-check-tooling-r3-body.md：判了题面与第二节五条，两条腿逐条答过
- research/prompts/merge-anchor-check-tooling-r2-main-verification.md：判了改法六条五条做到、第 4 条漏 `scripts/gate/apply_writer_patch_plan.py` 第 49 行（判决第 7 条）

## 回看决策

不涉及决策：这一题只改合入后验证这一道协作工具，不碰文件系统的设计条款。
