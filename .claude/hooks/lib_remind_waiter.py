"""循环提醒「还有进程在等」怎么认：只在这一份里写，两处都调它，不各写一份。

两处用这一份：
  scripts/agents/remind.sh                    start 写登记时取自己的启动时刻（start-time）；同名去重、stop、list 判在不在等（waiting-pid）；
                                              到点与被停时只删登记里仍是自己的那一份进程号文件（remove-own）
  .claude/hooks/remind-rearm-stop-guard.sh    Stop 时判登记着的循环提醒有没有进程在等，没有才交出重起命令（waiting_pid）

登记：<登记目录>/<名字>.pid 一行「<进程号> <启动时刻>」。启动时刻是 /proc/<进程号>/stat 第 22 段 starttime（开机以来的时钟滴答数），
  由等着的那个 remind.sh 进程起计时之前自己写。
认「在等」（waiting_pid），三样都对才算：
  进程活着：/proc/<进程号>/stat 读得到；
  启动时刻与登记的相同：进程号被别的进程复用了就对不上；读命令行前后各读一次，两次都要相同；
  命令真是 remind.sh（is_remind_command）：/proc/<进程号>/cmdline 的命令词文件名是 bash 或 sh，跳过解释器选项（-o、+o、-O、+O、--rcfile、--init-file
    各带一个值；单独的 - 与 -- 收尾），之后的脚本参数是 remind.sh 或以 /remind.sh 结尾；带 c 或 s 的短选项（bash -c、bash -s）跑的是命令串或标准输入，不算。
对不上任何一样按没人在等。旧登记只有进程号、没有启动时刻的，也按没人在等（会给出重起命令，重起之后登记里就有了）。
删自己的登记（remove_own_registration）：等着的 remind.sh 到点或被停时，<名字>.pid 里仍是「<自己的进程号> <自己的启动时刻>」才删；
  已被换成别的进程的（新旧登记交替时，重起的那一份已经写了自己的）、不在的、读不了的都不删。读完之后另核一次 inode，
  读与删之间被 mv 换上的新登记不删；剩下的窗口只有核 inode 与删之间那一步。
legacy_string_match=True 是调用方的弄坏开关退回去的旧判法（进程号活着、cmdline 任意位置含 remind.sh 就算在等），只给两份的自证证红。
这份库不读任何环境变量；调用方各按自己的弄坏开关决定传不传 legacy_string_match。

命令行用法（remind.sh 调）：
  python3 -B lib_remind_waiter.py start-time <进程号>                                  打出它的启动时刻，读不到退 1
  python3 -B lib_remind_waiter.py waiting-pid <登记目录> <名字> [--legacy-string-match]  在等就打出进程号，不在等什么都不打；都退 0
  python3 -B lib_remind_waiter.py remove-own <登记目录> <名字> <进程号> <启动时刻>         登记里仍是这个进程才删 <名字>.pid、打出 removed，没删打出 kept；都退 0
参数不对退 2，stderr 写用法。
"""
import os
import re
import sys

SCRIPT_NAME = "remind.sh"
INTERPRETERS = ("bash", "sh")
INTERPRETER_OPTIONS_WITH_VALUE = ("-o", "+o", "-O", "+O", "--rcfile", "--init-file")
START_TIME_FIELD = 22
PROCESS_NUMBER_PATTERN = re.compile(r"^[0-9]+$")


def process_start_time(pid):
    """/proc/<pid>/stat 第 22 段 starttime（文字）；进程不在或读不出交 None。"""
    try:
        with open(f"/proc/{pid}/stat", "rb") as handle:
            text = handle.read()
    except OSError:
        return None
    # 进程名在括号里、可能带空格与括号：取最后一个右括号之后的段，那里从第 3 段（state）起
    fields = text[text.rfind(b")") + 1:].split()
    index = START_TIME_FIELD - 3
    if len(fields) <= index:
        return None
    return fields[index].decode("ascii", "replace")


def process_arguments(pid):
    """/proc/<pid>/cmdline 切成的词；读不到交 None。"""
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as handle:
            raw = handle.read()
    except OSError:
        return None
    return [word.decode("utf-8", "replace") for word in raw.split(b"\0") if word]


def is_remind_command(arguments):
    """命令行是不是「bash [解释器选项] …/remind.sh …」。"""
    if not arguments or os.path.basename(arguments[0]) not in INTERPRETERS:
        return False
    index = 1
    while index < len(arguments):
        word = arguments[index]
        if word in ("-", "--"):
            index += 1
            break
        if word in INTERPRETER_OPTIONS_WITH_VALUE:
            index += 2
            continue
        if word.startswith("--"):
            index += 1
            continue
        if len(word) > 1 and word[0] in "-+":
            if word[0] == "-" and ("c" in word[1:] or "s" in word[1:]):
                return False
            index += 1
            continue
        break
    if index >= len(arguments):
        return False
    script = arguments[index]
    return script == SCRIPT_NAME or script.endswith("/" + SCRIPT_NAME)


def registration_words(directory, name):
    try:
        with open(os.path.join(directory, name + ".pid"), encoding="utf-8", errors="replace") as handle:
            return handle.read().split()
    except OSError:
        return []


def waiting_pid(directory, name, legacy_string_match=False):
    """这一条提醒等着的进程号（int）；没人在等交 None。"""
    words = registration_words(directory, name)
    if not words or not PROCESS_NUMBER_PATTERN.match(words[0]):
        return None
    pid = int(words[0])
    if legacy_string_match:
        try:
            os.kill(pid, 0)
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                return pid if SCRIPT_NAME.encode() in handle.read() else None
        except OSError:
            return None
    if len(words) != 2:
        return None
    recorded = words[1]
    if process_start_time(pid) != recorded:
        return None
    arguments = process_arguments(pid)
    if process_start_time(pid) != recorded:
        return None
    return pid if is_remind_command(arguments) else None


def remove_own_registration(directory, name, pid, start_time):
    """<名字>.pid 里仍是「<pid> <start_time>」才删，交 True；已被换成别的进程的、不在的、读不了的不删，交 False。
    读完之后另核一次 inode：读与删之间被 mv 换上的新登记不删。"""
    path = os.path.join(directory, name + ".pid")
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            read_identity = os.fstat(handle.fileno())
            words = handle.read().split()
    except OSError:
        return False
    if words != [str(pid), str(start_time)]:
        return False
    try:
        current = os.stat(path)
        if (current.st_dev, current.st_ino) != (read_identity.st_dev, read_identity.st_ino):
            return False
        os.remove(path)
    except OSError:
        return False
    return True


def usage(problem):
    print(f"✗ lib_remind_waiter.py：{problem}", file=sys.stderr)
    print("→ 怎么办：用法 start-time <进程号> | waiting-pid <登记目录> <名字> [--legacy-string-match] | remove-own <登记目录> <名字> <进程号> <启动时刻>"
          "（写法在本文件开头）", file=sys.stderr)
    return 2


def main(arguments):
    if arguments[:1] == ["start-time"] and len(arguments) == 2 and PROCESS_NUMBER_PATTERN.match(arguments[1]):
        start_time = process_start_time(int(arguments[1]))
        if start_time is None:
            print(f"✗ 读不到进程 {arguments[1]} 的启动时刻（/proc/{arguments[1]}/stat）", file=sys.stderr)
            print("→ 怎么办：在有 /proc 的 Linux 上起 remind.sh；进程已经退出的不用再登记", file=sys.stderr)
            return 1
        print(start_time)
        return 0
    if arguments[:1] == ["waiting-pid"] and len(arguments) in (3, 4):
        legacy = arguments[3:] == ["--legacy-string-match"]
        if len(arguments) == 4 and not legacy:
            return usage(f"认不出的参数：{arguments[3]}")
        pid = waiting_pid(arguments[1], arguments[2], legacy_string_match=legacy)
        if pid is not None:
            print(pid)
        return 0
    if (arguments[:1] == ["remove-own"] and len(arguments) == 5
            and PROCESS_NUMBER_PATTERN.match(arguments[3]) and PROCESS_NUMBER_PATTERN.match(arguments[4])):
        print("removed" if remove_own_registration(arguments[1], arguments[2], int(arguments[3]), arguments[4]) else "kept")
        return 0
    return usage(f"认不出的调用：{' '.join(arguments) or '（没给参数）'}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
