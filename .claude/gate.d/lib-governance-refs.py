#!/usr/bin/env python3
"""治理文档里的指向还指不指得到：门禁 10 号第 4 段调它，不单独成阶段。

扫的文件（治理文档）：CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、.claude/agents/*.md、
.claude/rules/*.md、.claude/skills/*/SKILL.md。三类指向：

  ① 门禁号：「门禁 65 号」「54、55、57、59 号」这类两位数加「号」，门禁目录里要有 <号>-*.sh。
     前面紧挨「第」「月」「日」（可隔空白）的不算门禁号（「第 13 号」「9 月 23 号」）。
     门禁目录取本文件所在的目录，不取被扫的仓根：样本目录里没有门禁脚本。
  ② 反引号里的仓内路径：反引号里按空白切开的每个词（命令里的脚本路径也算），ASCII 写成、中间带 `/`、
     而且第一段在解析基准下现存、或末段带常见的文件扩展名（`origin/master`、`text/plain` 两样都不满足，不算仓内路径）。
     解析基准：仓根、所在文件的目录、`.claude/`、`.claude/kb/`，外加同一个反引号里 `cd 目录` 之后的那个目录；一处在就算指得到。
     跳过：被 .gitignore 挡着的（规范副本、构建目录）、`../` `/` `~` 起头的、`refs/` 起头的 git 引用、带占位（`<` `*` `{`）的。
     已归档的写裸文件名（`.claude/agent-common.md`「找不到历史实验的数据」那一条），不带目录，不在射程里。
  ③ `文件「小节」`：文件在仓里、而「小节」里的字（去掉空白、反引号、星号、「」之后）在那份文件的全文里一处都找不到。
     判的是「全文任意位置出现」，不只看标题：定义常点正文里的一句；小节改了名而原名的字还散在正文里时抓不到。

判不到、逐条列进「没判的」（不算红）：③ 里文件解析不到的（多半是只写了文件名、住在四个基准之外）；
② 里带非 ASCII 字符、中间带 `/`、又指不到的记号（悬空的中文文件名路径与 `NN-简称.md` 这类占位分不开；指得到的照常算判过）。

输出：每处红一行「文件:行: 说明」；「没判的」逐条一行；末行「扫 N 份治理文档：门禁号 A 处、路径 B 处、小节 C 处；没判 D 处」。
退出码：0 全指得到；1 有指不到的；2 一份治理文档都没扫到（没有对象，不许当通过）。
"""
import glob
import os
import re
import subprocess
import sys

GATE_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
CARRIER_PATTERNS = ["CLAUDE.md", ".claude/main-agent.md", ".claude/agent-common.md",
                    ".claude/agents/*.md", ".claude/rules/*.md", ".claude/skills/*/SKILL.md"]
GATE_NUMBER = re.compile(r"(?<![0-9A-Za-z.])((?:[0-9]{2}\s*、\s*)*[0-9]{2})\s*号")
NOT_A_GATE_BEFORE = re.compile(r"(第|月|日)\s*$")
BACKTICK = re.compile(r"`([^`\n]+)`")
REPO_PATH = re.compile(r"^[A-Za-z0-9_.\-/]+$")
FILE_WITH_SECTIONS = re.compile(r"`?([^\s`「」]+\.(?:md|py|sh))`?\s*(?:的)?\s*((?:「[^」]+」(?:、|与|和)?)+)")
DECORATION = re.compile(r"[\s`*「」]")
BASE_DIRECTORIES = ("", ".claude", ".claude/kb")


def existing_gate_numbers():
    numbers = set()
    for name in os.listdir(GATE_DIRECTORY):
        match = re.match(r"([0-9]+)-.*\.sh$", name)
        if match:
            numbers.add(int(match.group(1)))
    return numbers


def ignored_by_git(path):
    result = subprocess.run(["git", "check-ignore", "-q", "--no-index", path],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return result.returncode == 0


def bases_for(carrier, changed_directories):
    return list(BASE_DIRECTORIES) + [os.path.dirname(carrier)] + list(changed_directories)


def resolve(path, carrier, changed_directories=()):
    bare = path.rstrip("/")
    for base in bases_for(carrier, changed_directories):
        candidate = os.path.normpath(os.path.join(base, bare))
        if os.path.exists(candidate):
            return candidate
    return None


FILE_EXTENSION = re.compile(r"\.(md|sh|py|rs|tsv|toml|json|txt|out|lock)$")


def looks_like_repo_path(path, carrier, changed_directories):
    """第一段在解析基准下现存，或末段带常见的文件扩展名，才当仓内路径：`origin/master`、`text/plain` 两样都不满足。"""
    first = path.split("/", 1)[0]
    if any(os.path.exists(os.path.join(base, first)) for base in bases_for(carrier, changed_directories)):
        return True
    return bool(FILE_EXTENSION.search(path.rstrip("/")))


def path_token(token):
    """反引号里的一个词若形如一条仓内路径，返回去掉 `:行号`、`@标题`、`~正则` 之后的路径，否则 None。"""
    path = re.split(r"[:@~]", token, maxsplit=1)[0]
    if "/" not in path.rstrip("/"):
        return None
    if path.startswith(("../", "/", "~", "refs/", "http")):
        return None
    if any(mark in path for mark in "<*{"):
        return None
    return path


def main():
    carriers = sorted({found for pattern in CARRIER_PATTERNS for found in glob.glob(pattern)})
    if not carriers:
        print("  ✗ 一份治理文档都没扫到（CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、agents、rules、skills）")
        print("     → 怎么办：在仓库根上跑；治理文档搬了家的话，改本文件的 CARRIER_PATTERNS。")
        return 2
    gates = existing_gate_numbers()
    problems, unjudged = [], []
    counts = {"gate": 0, "path": 0, "section": 0}
    for carrier in carriers:
        with open(carrier, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        for number, line in enumerate(lines, 1):
            for match in GATE_NUMBER.finditer(line):
                if NOT_A_GATE_BEFORE.search(line[:match.start()]):
                    continue
                for gate in re.split(r"\s*、\s*", match.group(1)):
                    counts["gate"] += 1
                    if int(gate) not in gates:
                        problems.append(f"{carrier}:{number}: 「{gate} 号」在门禁目录里没有 {gate}-*.sh")
            for match in BACKTICK.finditer(line):
                words = match.group(1).split()
                # 命令里 `cd 目录 && …` 之后的相对路径按那个目录解析
                changed_directories = [words[i + 1] for i in range(len(words) - 1) if words[i] == "cd"]
                for word in words:
                    path = path_token(word)
                    if path is None:
                        continue
                    if not REPO_PATH.match(path):
                        # 非 ASCII 的记号：指得到就算判过；指不到的与 `NN-简称.md` 这类占位分不开，列进没判的
                        if resolve(path, carrier, changed_directories) is not None:
                            counts["path"] += 1
                        else:
                            unjudged.append(f"{carrier}:{number}: `{word}` 带非 ASCII 字符、指不到，分不清是占位还是悬空，路径没判")
                        continue
                    if not looks_like_repo_path(path, carrier, changed_directories) or ignored_by_git(path):
                        continue
                    counts["path"] += 1
                    if resolve(path, carrier, changed_directories) is None:
                        problems.append(f"{carrier}:{number}: `{match.group(1)}` 里的 {path} 在仓里不存在")
            for match in FILE_WITH_SECTIONS.finditer(line):
                target = resolve(match.group(1), carrier)
                if target is None or not os.path.isfile(target):
                    unjudged.append(f"{carrier}:{number}: {match.group(1)}{match.group(2)} 的文件在四个解析基准下都找不到，小节没判")
                    continue
                with open(target, encoding="utf-8") as handle:
                    haystack = DECORATION.sub("", handle.read())
                for section in re.findall(r"「([^」]+)」", match.group(2)):
                    counts["section"] += 1
                    if DECORATION.sub("", section) not in haystack:
                        problems.append(f"{carrier}:{number}: {match.group(1)}「{section}」在那份文件里一处都找不到")
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    if problems:
        print("     → 怎么办：门禁号改成现存的号或共享 gate.sh 里那一道的名字；路径改成现存的，已归档的写裸文件名；小节按那份文件今天的标题改。")
    for item in unjudged:
        print(f"  没判的：{item}")
    print(f"  扫 {len(carriers)} 份治理文档：门禁号 {counts['gate']} 处、路径 {counts['path']} 处、小节 {counts['section']} 处；没判 {len(unjudged)} 处")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
