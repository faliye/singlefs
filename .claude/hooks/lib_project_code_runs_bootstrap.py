# lib_project_code_runs.py 的部分模块：引导要的名字（弄坏开关变量名与读法、部分文件名表）。
# 入口 lib_project_code_runs.py 在 import 别的部分模块之前先 import 这一份（别的部分模块读不到时的处置要用这里的名字），这一份不 import 同目录别的部分模块；
# 这一份读不到时没法点名是哪一份，入口的导入直接抛原来的错。
import os

# 弄坏开关的环境变量（写法见文件头）
BREAK_VARIABLE = "LIB_PROJECT_CODE_RUNS_BREAK"
# 入口 lib_project_code_runs.py 按这张表的次序 import 的部分模块（后一份 import 前一份）：自证拷一份目录逐份写坏、截空的是这几份；路径按这份文件自己所在的目录取
PROJECT_CODE_RUNS_PART_NAMES = ("lib_project_code_runs_base.py", "lib_project_code_runs_launchers.py", "lib_project_code_runs_selftest.py")
PROJECT_CODE_RUNS_BOOTSTRAP_FILE_NAME = os.path.basename(__file__)   # 这一份自己的文件名：不在上面那张表里（它读不到时入口没法点名），自证拷一份目录时连它一起拷


def break_is_set(switch_name):
    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")
