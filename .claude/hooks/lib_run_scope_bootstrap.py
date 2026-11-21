# lib_run_scope.py 的部分模块：引导要的名字（弄坏开关变量名、自证标记与它的认法、守卫脚本名、钩子目录、部分文件名表、读不到部分文件时退的码）。
# 入口 lib_run_scope.py 在 import 别的部分模块之前先 import 这一份（别的部分模块读不到时的处置要用这里的名字），这一份不 import 同目录别的部分模块；
# 这一份读不到时没法点名是哪一份，入口的导入直接抛原来的错。
import os

HOOK_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
GUARD_SCRIPT_NAME = "run-scope-guard.sh"
BREAK_VARIABLE_NAME = "RUN_SCOPE_GUARD_BREAK"
# 弄坏开关只在自证里认：自证起的入口靠这个变量指到自证建的标记文件、内容逐字是 SELFTEST_MARK_CONTENT 才认开关（同一句写在 run-scope-guard.sh 那一段守卫里）
SELFTEST_MARK_VARIABLE = "RUN_SCOPE_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "run-scope-guard.sh --selftest 起的子进程"
# 入口 lib_run_scope.py 按这张表的次序 import 的部分模块（后一份 import 前一份）：第一份是判法本体，自证拷一份目录写坏、截空的是第一份；路径按这份文件自己所在的目录取
RUN_SCOPE_PART_NAMES = ("lib_run_scope_core.py", "lib_run_scope_selftest.py")
RUN_SCOPE_BOOTSTRAP_FILE_NAME = os.path.basename(__file__)   # 这一份自己的文件名：不在上面那张表里（它读不到时入口没法点名），自证拷一份目录时连它一起拷
PART_UNREADABLE_STATUS = 70   # 当入口脚本起、读不到、编不过或缺入口要的名字（编得过而被截空）的部分文件时退的码（不是 0、2：run-scope-guard.sh 照「判法没跑完」放行并记检出）


def selftest_marked():
    """自证起的子进程：SELFTEST_MARK_VARIABLE 指到的文件在、内容逐字是 SELFTEST_MARK_CONTENT。"""
    mark = os.environ.get(SELFTEST_MARK_VARIABLE, "")
    if not mark or not os.path.isfile(mark):
        return False
    with open(mark, encoding="utf-8", errors="replace") as handle:
        return handle.read().rstrip("\n") == SELFTEST_MARK_CONTENT
