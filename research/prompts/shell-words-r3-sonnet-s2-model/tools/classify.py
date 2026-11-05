#!/usr/bin/env python3
"""把判错的样本归类。类的名字照正文「二、实现今天的样子」列的三组：
  A 第二轮判决第 11 条：A1 拷出写法（>>、&>、>&、tee）  A2 bash -s < 文件  A3 xargs（含 xargs env）  A4 名单外启动器
  B 修补报告新偏严三类：B1 cat 不看源文件在不在  B2 重定向目标花括号展开成几个词  B3 函数体之外引号里的花括号被当认不准的调用
  C C12c：仓副本里经脚本自己的函数起重活（名单外启动器）
  Z 不在清单里；Z1…是子类。第三项是"钩子文件头是否已写明这是看不见的/接受的"：是/否/相反（文件头写了认，实际不认）。
返回 (类, 说明, 文件头)。顺序即优先级。"""
import re


def classify(r):
    cmd, want, hook, fid = r["cmd"], r["want_reject"], r["hook"] == "拒", r["id"]
    over, miss = hook and not want, want and not hook
    runs_script = bool(re.search(r"\b(bash|sh|source)\s+(-\w+\s+)*\S*\.sh\b|(^|[;&|\n]\s*)(\./|\S*/)\S+\.sh\b", cmd))
    if miss and re.search(r"\bxargs\b", cmd):
        return "A3", "xargs 起的命令/脚本（含 xargs env、xargs bash）", "是（heavy-test-guard.sh:197「xargs 起的命令」）"
    if miss and re.search(r"\bfind\b.*-exec(dir)?\b|\bssh\b|\bdoas\b|\bscript\s+-|\bunshare\b|\bsetpriv\b|\bnumactl\b", cmd) and not cmd.startswith("sed"):
        return "A4", "名单外启动器：" + re.search(r"find .*-exec(dir)?|ssh|doas|script|unshare|setpriv|numactl", cmd).group(0).split()[0], "否（名单见 lib_heavy_tests.py LAUNCHER_OPTIONS_WITH_VALUE，文件头只点了其中几个）"
    if miss and re.search(r"(\b(bash|sh)\b( -s| -)? *<(?!<)\s*\S+|\bcat\s+\S+\s*\|\s*(bash|sh)\b)", cmd) and not re.search(r"<<<|<\(", cmd):
        return "A2", "标准输入从文件来的 shell（`bash -s < 文件`、`bash < 文件`、`cat 文件` 经管道进 bash）", "是（heavy-test-guard.sh:182「标准输入从文件来）判不了：照常执行，stderr 报一句、记一条检出」）"
    if miss and "watch.sh" in cmd:
        return "Z7", "① 看门狗输出经 `| tee /dev/null` 丢掉", "否"
    if miss and runs_script and re.search(r"&>|>&\s|>>|\btee\b", cmd) and not re.search(r"\bdd\b|\bsed\b|\bhead\b|\bprintf\b|\becho\b|python|install", cmd.split("EOF")[0] if "EOF" in cmd else cmd):
        return "A1", "脚本拷出用 >> / &> / >& / tee 再执行", "是（heavy-test-guard.sh:176「`>>` / `&>` 写出的 cat 拷贝」「管道 `cat <<EOF | tee x.sh`」）"
    if miss and re.search(r"run_logged|\w+\(\) *\{.*\"\$@\"|function \w+ \{.*\"\$@\"", cmd):
        return "C", "函数当启动器（run_logged 一类）", "否"
    if over and re.search(r"nosuch|missing", cmd):
        return "B1", "cat 不看源文件在不在", "否"
    if over and re.search(r">\s*'?\S*\{[^}]*(,|\.\.)[^}]*\}", cmd):
        return "B2", "重定向目标花括号展开成几个词", "否"
    if over and re.search(r"echo ['\"][^'\"]*\{", cmd) and re.search(r"\(\) *\{|function ", cmd):
        return "B3", "函数体之外引号里的花括号", "否"
    # —— 不在清单里 ——
    if fid.startswith("D-"):
        if miss and "two-amp-wait-one" in fid:
            return "Z11", "两个 & 只等其中一个（wait $p1），另一个留下", "是（detector 文件头 ② 末「之后的 wait \"$pid\" 只等了其中一个」只记不拒）"
        if over and "amp-wait-n" in fid:
            return "Z12", "bg 里 `& … wait -n`（单个作业，真等到了）按没等算", "是（detector:29「`wait -n` 不算」）"
        if over and re.search(r"tail --pid|kill -0", cmd) and "bg1" in fid:
            return "Z13", "bg 里 `tail --pid` / `kill -0` 轮询等它，文件头写了算等，实际仍按没等拒", "相反（detector:29「或之后有 `tail --pid` / `while|until … kill -0` 轮询」）"
        if over and "amp-then-wait-in-fn" in fid:
            return "Z14", "wait 写在函数里、函数被调用，认不出", "否"
        if over and "fn-loop-notcalled" in fid:
            return "Z5", "没被调用的函数体按字面判（循环）", "否"
        if over and fid.startswith(("D-loop0", "D-bg0")):
            return "Z10", "真会结束的前台等待循环按字面拒（计数上限、文件已在、read 循环、break、kill -0 轮询）", "是（detector:109「带计数上限的 while 循环机器分不出上限，照拒」；其余同一原理）"
    if miss and re.search(r"\beval\b", cmd):
        return "Z1", "eval 拼出命令", "是（heavy-test-guard.sh:195「eval」；detector:67「eval」）"
    if miss and re.search(r"bash <\(|source <\(|\. <\(|bash -c \"\$\(|\$\(cat|\$\(printf", cmd):
        return "Z1", "进程替换 / 命令替换里拼出命令或脚本", "是（heavy-test-guard.sh:195/182「喂给 shell 的不是字面」）"
    if miss and "claude" in cmd and re.search(r"<<<|env -S", cmd):
        return "Z6", "⑬ claude：here-string / env -S 里写 claude 认不出（①与重型闸对同一写法认得）", "否（detector ① 写了 here-string 字面照命令判；⑬ 那一条只写了 heredoc）"
    if miss and "claude" in cmd and re.search(r"\$c\b", cmd):
        return "Z2", "变量里拼出命令词", "是（detector:67「变量里拼出来的命令」）"
    if miss and re.search(r"bash -c '[^']*(\$0|\"\$1\"|\"\$@\")|sh -c '[^']*\"\$@\"", cmd):
        return "Z16", "bash -c / sh -c 的脚本里用位置参数 $0 $1 \"$@\" 拼出命令", "否"
    if miss and re.search(r"(^|;|&&)\s*cargo test[^;]*\$\w+|(^|;)\s*cargo \$\(", cmd):
        return "Z15", "cargo 的旗标 / 包名 / 子命令由变量或命令替换拼出", "否（文件头只写命令词与脚本路径）"
    if miss and re.search(r"\"\$\(command -v|\$\(which|^set -- ", cmd):
        return "Z2", "命令词由命令替换 / set -- 算出", "是（heavy-test-guard.sh:195「命令词没展开（$b、$(ls …)）」）"
    if miss and re.search(r"PATH=\S+:\$PATH \w+\.sh", cmd):
        return "Z3", "命令词不带斜杠、经 PATH 找到的脚本", "是（heavy-test-guard.sh:199「命令词不带斜杠、从 PATH 里找到的脚本」）"
    if miss and re.search(r"(^|;)\s*\w+=\(|(^|;)\s*\w+='[^']*'; *(\$\w+|eval)|(^|;)\s*t=|^for f in|d=\{|\$\(echo ", cmd):
        return "Z2", "变量里拼出命令词 / 脚本路径（含 for 循环变量、命令替换算出路径）", "是（heavy-test-guard.sh:195「变量里拼出来的命令与脚本路径」）"
    if miss and re.search(r"\*s\.sh|\*ok\.sh|s\.s\?", cmd):
        return "Z9", "bash 的脚本路径参数里带通配，没按 bash 展开去读", "是（heavy-test-guard.sh:195「通配与花括号写出来的脚本里的正文」）"
    if miss and re.search(r"\bdd\b|\bsed\b .*>|\bhead\b .*>|\bprintf\b.*>|\becho\b.*>|\binstall\b|python3? ", cmd) and runs_script and "rsync" not in cmd:
        return "Z4", "脚本用 dd / sed / head / printf / echo / install / python 写出再执行", "是（heavy-test-guard.sh:176「认不出的写法（printf、echo、python 写文件」；dd/sed/head/install 没点名，同一原理）"
    if over and re.search(r"\bif (false|true)|&&|\bfor i in;|\bwhile false|\buntil true|\bcase a in b|\(\) *\{.*\}; *echo done", cmd):
        return "Z5", "永远不执行的分支 / 没被调用的函数体按字面拒", "否（文件头只写了 `function 名字 { }` 不判，圆括号写法、if/while/case 没写）"
    if miss and "rsync" in cmd:
        return "Z8", "⑦ 没点名 rsync --inplace（同 inode 改脚本）", "否"
    if miss and "tee /dev/null" in cmd:
        return "Z7", "① 看门狗输出经 `| tee /dev/null` 丢掉", "否"
    if miss and "git" in cmd and "$x" in cmd:
        return "Z2", "变量里拼出命令词（git $x）", "是"
    return "Z0", "（待逐条看）", "?"
