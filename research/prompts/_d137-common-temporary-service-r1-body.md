# d137-common-temporary-service-r1：共用约束里「要临时服务的自证怎么写命令」那一句——第一轮（2026-11-05）

## 一、题面

被判的是 `.claude/agent-common.md` 第 64 行「子 agent 不交任务」那一条里新加的一句（账本 `records/2026-10-29-subagent拆分提案.md` 一百八十三节，调度记录 `records/2026-10-29-一百三十七节那一轮调度记录.md` 批次 #23 件 37）。原句：

> 登在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本，写进报告的自证命令要套成 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <解释器> <脚本> --selftest`

用户对这一句定了走三方（不豁免）。同一批里与它配套、已换上或已改进主工作区的东西（这一轮不判它们本身对不对，只拿来核那一句与它们是否一致）：

- 登记表 `.claude/hooks/selftest-temporary-service.tsv`（26 行：24 行 required、2 行 own-server，格式在它文件头）；
- `.claude/hooks/lib_heavy_tests.py` 读这张表的判法、`.claude/hooks/heavy-test-guard.sh` 对 required 脚本的裸 `--selftest` 执行前拒（hook-swap 任务 6569 换上）；
- `scripts/agents/dispatch-prompt.py` 骨架多一行「要临时服务的自证：…」；
- `.claude/gate.d/code-tooling.sh` 新格 temporary-service-runners，认要临时服务改读这张表。

另一个会话正在把 `.claude/hooks/heavy-test-guard.sh` 重写、改名为 `project-code-guard.sh`（用户定不再分轻重快慢）；这一轮判的是那一句写在共用约束里站不站得住，钩子改名之后它还成不成立也在射程里。

被判的文件（快照 `research/prompts/d137-common-temporary-service-r1-snapshot/files-sha256.txt`，7 份）：`.claude/agent-common.md`、`.claude/hooks/selftest-temporary-service.tsv`、`scripts/agents/dispatch-prompt.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`.claude/gate.d/code-tooling.sh`、`scripts/prune-server/prune_server_client_selftest.py`。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

- 这一批不改 `crates/` 的源码；`grep -rn selftest-temporary-service crates/` 零命中。
- 换上之后：`lib_heavy_tests.py --selftest` 任务 6572、`heavy-test-guard.sh --selftest` 6573、`dispatch-prompt.py --selftest` 6574 都退 0；十七条弄坏开关 6575–6591 红的格都在各自文件头的点名里。
- code-tooling 新格、样本与开关交了服务（任务 6703–6710），交这份材料时还没出结局。
- 要临时服务的 required 脚本，裸起 `--selftest` 时自己退 2（例：任务 2731，`crates-mutation-rows.py` 的原话「没有临时 prune-server（PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET 没设）：自证不写真服务」）。

## 三、格

### F　正推：那一句做到了它要做的没有（正推腿）
从定义的读者（子 agent 写报告「要主 agent 跑的命令」一节）出发：照这一句写出的命令，主 agent 照原样交服务，能不能过 `heavy-test-guard.sh`（执行前拒）、`task-server/task submit` 的提交时拒、`scripts/prune-server/prune_server_client_selftest.py` 自己的参数解析，并且真在临时服务底下起自证。逐个读这几处的源码核，不推。另核：这一句与共用约束同一条前后的句子（「写法照下一条」那一条、范围声明、白名单环境变量、不带包装前缀那一串）有没有互相顶；与 `.claude/main-agent.md` 和 `.claude/agents/` 下各定义里给自证命令的写法是否一致。

### A　攻方：哪一格会错（攻方腿）
找这一句写错、写漏、会被照错的地方。至少看这几面，不限于此：
- 「交给服务的命令不带包装前缀」那一串点名的包装（同一条第 64 行）与 `prune_server_client_selftest.py --with-temporary-service --` 这一层是不是同一类；照字面读两句会不会互相否定。
- `<解释器>` 怎么取：`.sh` 的、`.py` 的、带执行位直接起的；登记表里有没有一份脚本按这一句拼出来起不来。
- own-server 那两份、登记表外的脚本、表里一份脚本有几种起法的，照这一句会不会被拼错。
- 钩子改名、不分轻重之后，这一句点名的表与写法还对不对；共用约束写到具体表名与命令，表或命令改了谁来同步、会不会腐烂。
- 要不要写进共用约束：骨架那一行已经拼整条命令，共用约束再写一遍是不是重复的第二处登记位（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条「矛盾比空白更糟」那一面）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿 | F | 照源码核这一句写出的命令走得通 |
| 攻方腿 | A | 找写错、写漏、互相顶、会腐烂的地方 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引定义、规则、脚本、钩子写主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 子 agent 不往统一任务服务交任务：要跑的命令（只读的 `task-server/task log` 除外）拼成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节，由主 agent 交。
- 不造新的绕过钩子的写法。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。草稿与临时文件放 `~/.local/share/singlefs/work/<腿名>/`。
