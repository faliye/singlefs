# d137-common-temporary-service-r2：共用约束里「要临时服务的自证怎么写命令」那一句——第二轮（2026-11-05）

## 一、题面

第一轮判决：`research/prompts/d137-common-temporary-service-r1-main-verification.md`。照它把 `.claude/agent-common.md` 第 64 行那一句改成了：

> 登在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本，写进报告的自证命令照那张表文件头 required 一条给的整条写法套临时服务（`<解释器>`：`.py` 用 `python3`，别的用 `bash`；这一层不算「交给服务的命令不带包装前缀」那一串里的包装）

第一轮判「采一半」的 H1（不带 `--server-binary` 时包装回落现编），根交 prune-server 那一路（拆分提案一百九十六节 ①）；H3、H6 交改钩子名的那一路；H4 不采、记账（一百九十六节 ②）。

这一轮按 `.claude/rules/three-way-inference.md`「多轮」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。不重复第一轮攻过的角度（第一轮攻方 H1–H7、正推缺口 1–4 不再原样提）。

被判的文件（快照 `research/prompts/d137-common-temporary-service-r2-snapshot/files-sha256.txt`，8 份）：`.claude/agent-common.md`、`.claude/hooks/selftest-temporary-service.tsv`、`scripts/agents/dispatch-prompt.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py`、`.claude/gate.d/code-tooling.sh`、`scripts/prune-server/prune_server_client_selftest.py`、第一轮判决。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

- 这一批不改 `crates/` 的源码；`grep -rn selftest-temporary-service crates/` 零命中。
- 登记表文件头 required 那一条此刻写着整条写法 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <二进制>] -- <解释器> <脚本> --selftest`（第 5 行）。
- 另一个会话正在把 `.claude/hooks/heavy-test-guard.sh` 重写、改名为 `project-code-guard.sh`（用户定不再分轻重快慢），`.claude/gate.d/code-tooling.sh` 已改读 `.claude/hooks/lib_project_code_runs.py`。
- 第一轮主 agent 复跑的读数都在第一轮判决「核查怎么做的」一节（任务 6777–6782）。

## 三、格

### R　第一轮判决够不够得着（辩方腿）
逐条复核第一轮判决：每一条「采 / 不采 / 采一半」的依据是不是那几条任务与源码真撑得住（例：H1 只在本机那一侧成立这个判断，6778 的「编译目录 无」与二进制修改时刻能不能撑；H4 判「不归这一句」站不站得住）；被判句改成「照表头那一条」之后，是不是同样打中了别的写法（例：把写法交给表头，表头错了谁来发现；子 agent 读不读得到表头）。替被判「不采」的一方辩护：正推缺口 3、H4、H6 是不是该采。

### A　第一轮修补本身与没看过的地方（攻方腿）
只攻改过之后的那一句与它点到的东西：「照那张表文件头 required 一条给的整条写法」读者照着能不能唯一地拼出命令（`[--server-binary <二进制>]` 这种可选段，读者写不写、写什么）；「这一层不算……那一串里的包装」这个例外写法会不会让人把别的包装也当例外；第一轮没看过的角度（例：表头改名、表行增删时这一句怎么跟；三方腿、回扫员这些骨架不给那一行的类型照这一句写出来的命令）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 辩方腿 | R | 复核第一轮判决，替被判不采的一方辩护 |
| 攻方腿 | A | 只攻第一轮修补本身与没看过的地方 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引定义、规则、脚本、钩子写主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 子 agent 不往统一任务服务交任务：要跑的命令（只读的 `task-server/task log` 除外）拼成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节，由主 agent 交。
- 不造新的绕过钩子的写法。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。草稿与临时文件放 `~/.local/share/singlefs/work/<腿名>/`。
