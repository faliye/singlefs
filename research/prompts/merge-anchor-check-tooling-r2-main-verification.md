# merge-anchor-check-tooling-r2：判决（2026-11-21）

正文 `research/prompts/_merge-anchor-check-tooling-r2-body.md`，开工快照 `research/prompts/merge-anchor-check-tooling-r2-snapshot/`。这一题治理文件轮第二轮，攻第一轮判决 `research/prompts/merge-anchor-check-tooling-r1-main-verification.md`「三、改法」六条的修补，并由辩方腿复核第一轮判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（Sonnet） | `research/prompts/merge-anchor-check-tooling-r2-sonnet-output.md` | 第一轮八条大多站得住；第 1 条补得不全：只追加变异行、没有 crates.patch 的那一批合入那一段一条核都不打（镜像方向），同文件追加一律误拒、出路整批重交按小时计；第 4 条不给补丁目录的那种照旧不核；第 7 条耗时偏低（量到 406、740 秒，第一轮攻方腿量到 625 秒）；六条修补都对得上 |
| 攻方（Opus） | `research/prompts/merge-anchor-check-tooling-r2-opus-output.md` | 打中 1a（这一批自己追加的行锚在补丁没改到的文件上漏过）、1b（RELEASE 要看的登记表不进指纹起点）、1c（同文件追加误拒）、3a（带 `--skip-apply` 不给补丁目录时一句都不打），自证分不出三处改坏；没打中 RELEASE 去行号互相顶掉、日志全数与出路、十条新开关、两棵临时树里的登记表 |

核查：
- 开工快照八份 `sha256sum -c` 都 OK（腿跑着时没被改），引文直接在仓根上核：`python3 scripts/agents/cite-check.py research/prompts/merge-anchor-check-tooling-r2-sonnet-output.md research/prompts/merge-anchor-check-tooling-r2-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_merge-anchor-check-tooling-r2-body.md`：「核了 42 处引文、46 处裸行号，对上 42 处，没判 46 处（其中裸行号没有引文可比 46 处）」。
- 辩方腿的镜像样本（`mirror-cell.patch`，sha256 与腿报告相同）打在主 agent 建的副本上交合入后验证自证，`type=command-failed exit=1`：「✗ 自检：」0 行，只红「✗ 镜像样本命中：开跑之后别的会话改了这一批追加的变异行锚着的文件，合进主工作区那一段打出的 0 条核一条都没退非 0，合进去之后 mirror-own-row 命中 0 次」。
- 攻方腿的样本补丁（`attack2.patch`，sha256 与腿报告相同，副本三份与腿草稿 `base/` 逐份相同）在主 agent 建的副本上串行交五趟：
  - R0（样本格开、不改坏）`exit=1`：「✗ 自检：」0 行；红的五格：own-row-concurrent-edit（mutonly、with-code 两格，「合进主工作区那一段的核全退 0，合进去之后那一行原文命中 0 次」）、own-row-concurrent-registration 与 registration-concurrent-row（「合进去之后那一行 RELEASE 红」）、skip-apply-no-directories（「变异锚点核一句都没打（没核，也没写本次未核）」）；该绿的 anchored-rows-unchanged、skip-apply-replacement、skip-apply-irreversible 三格是「·」。
  - R1、R2（锚定行记法改坏）`exit=1`、「✗ 自检：」0 行，各红 anchored-rows-unchanged（「主工作区没动……那一条退 1」）；R3（`--skip-apply` 替换不倒回）红 skip-apply-replacement；R4（倒不回照放行）红 skip-apply-irreversible。三处改坏现有自证都分不出。
- 两份脚本的开关复跑：`apply-writer-patch.py` 那一条 `type=command-failed exit=1`「不设开关的 --selftest 就没过」，单跑 `apply-writer-patch.py --selftest` `exit=1`，锚点静态核几格全红在「ERROR 被判的树里没有登记表 …/.claude/gate.d/stage-inputs.tsv」——管变异键那一件换上之后 static-check 读被判那棵树的登记表，打补丁脚本自证自己造的样本树里没有这张表；`merge-and-verify.sh` 那一条（改槽位那一件的会话交的）「开跑前算键，174 条里拒算 174 条」，拒算点 `scripts/gate/merge-and-verify-selftest-lib.sh:707` 的 `bash -c "$anchored_check_line"`。

## 二、判决

| # | 腿报的 | 判 | 依据 |
|---|---|---|---|
| 1 | 只追加变异行（或自己追加的行锚在补丁没改到的文件上）时，开跑之后别的会话改了那份文件，合进去才断（辩方镜像、攻方 1a） | **采，量过** | 辩方镜像样本与攻方 R0 两格都红；记的路径只从 `crates.patch` 取 |
| 2 | RELEASE 要看的登记表不进指纹起点：开跑之后别的会话改了登记表、或这一批改登记表而别的会话追加点名同一目标的行，合进去 RELEASE 红（攻方 1b） | **采，量过** | 攻方 R0 两格 |
| 3 | 同文件追加一律误拒、整批重交（辩方、攻方 1c） | **采** | 改法第 1 条按文件比，不看锚点受不受这一批影响；整批重交要再编再测 |
| 4 | 带 `--skip-apply` 不给补丁目录时一句都不打（辩方、攻方 3a） | **采，量过** | 攻方 R0 skip-apply-no-directories |
| 5 | 自证分不出三处改坏（锚定行记法、`--skip-apply` 替换不倒回、倒不回照放行） | **采，量过** | R1–R4 |
| 6 | 耗时注释取一个样本偏低（量到 406、625、740 秒） | **采** | 各趟挂钟受同机任务影响，写成范围 |
| 7 | `apply-writer-patch.py --selftest` 样本树缺登记表 | **采，量过** | 单跑 `exit=1` 那一句；管变异键那一件换上带出的，打补丁脚本在这一轮快照里，由这边改 |
| 8 | `merge-and-verify-selftest-lib.sh:707` 用 `bash -c` 起拼出来的一串，开关复跑全数拒算 | **采，量过** | 开关复跑「174 条里拒算 174 条」 |
| 9 | RELEASE 去行号互相顶掉、日志全数与出路、十条新开关、两棵临时树里的登记表、第一轮第 3 条的归属 | **没打中**（两条腿都判） | 腿报告各自第二节；第一轮第 3 条已由管变异键那一件落地 |

## 三、改法

都在 `scripts/gate/` 下改，交工具员一次改完；这一轮的改法在第三轮攻。

1. 判决第 1、2、3 条：合进主工作区那一段，真合之前多一条 `apply-writer-patch.py --dry-run <merge-patch>`（攻方腿 R2-2）：在合入那一刻的主工作区上照补丁前后比锚点（static-check 读被判那棵树的登记表），只拦这一批新弄断的，罩住镜像方向、自己追加的行、登记表被并发改三形，也不误拒同文件里不受影响的追加；第一轮改法第 1 条那一条按文件比锚定行的核删掉。代价写进出路：多一趟约 400–740 秒、峰值约 15 GiB。
2. 判决第 4 条：带 `--skip-apply` 不给补丁目录的那一趟打一行「变异锚点核本次未核」并点名为什么。
3. 判决第 5 条：自证收攻方腿 `attack2.patch` 里 skip-apply-replacement、skip-apply-irreversible、skip-apply-no-directories 三格与辩方腿镜像那一格，改成平常就跑；照改法第 1 条把 own-row 与 registration 那几格改成合入前那一条 `--dry-run` 拦住；改坏登成弄坏开关、各红在点名那一格。
4. 判决第 6 条：三处耗时注释写成范围（真仓一趟两次 static-check 约 400–740 秒、峰值约 15 GiB）。
5. 判决第 7 条：`apply-writer-patch.py` 自证造的样本树补上 `.claude/gate.d/stage-inputs.tsv`，几格锚点静态核照旧判。
6. 判决第 8 条：`merge-and-verify-selftest-lib.sh:707` 一带改成闭包折得出的写法（照 `scripts/gate/lib_function_units.py` 文件头认得的那几种，不经 `bash -c` 起拼出来的一串）；照改法第 1 条那一格删掉时一起处理。

## 交用户的

无。

## 这一轮被判过的文件

- scripts/gate/merge-and-verify.sh：判了指纹起点只取 crates.patch 的路径（判决第 1、2 条）、`--skip-apply` 不给补丁目录不打（判决第 4 条）
- scripts/gate/merge-and-verify-steps-lib.sh：判了锚点核的出路与耗时注释（判决第 6 条），日志全数没打中
- scripts/gate/merge-and-verify-items-lib.sh：判了合入那一段的核按文件比误拒（判决第 3 条）
- scripts/gate/merge-and-verify-selftest-lib.sh：判了三处改坏分不出（判决第 5 条）与第 707 行 `bash -c` 拒算（判决第 8 条）
- scripts/gate/apply-writer-patch.py：判了自证样本树缺登记表（判决第 7 条），RELEASE 去行号没打中
- scripts/gate/apply_writer_patch_plan.py：判了两棵临时树里的登记表（没打中）与 `STATIC_CHECK_DECLARED` 那一处
- research/prompts/_merge-anchor-check-tooling-r2-body.md：判了题面与第二节五条，两条腿逐条答过
- research/prompts/merge-anchor-check-tooling-r1-main-verification.md：辩方复核：第 1 条补得不全、第 4 条漏一种、第 7 条数偏低，其余站得住

## 回看决策

不涉及决策：这一题只改合入后验证这一道协作工具，不碰文件系统的设计条款。
