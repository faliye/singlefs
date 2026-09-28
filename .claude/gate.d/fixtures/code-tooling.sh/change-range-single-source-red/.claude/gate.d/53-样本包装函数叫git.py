# 红样本：包装函数叫 git，另一行直接跑 ['git', …] 列表而没调它；参数列表换了行
import subprocess
def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], capture_output=True, text=True)
bare = subprocess.run(['git', 'ls-files', '--others', '--exclude-standard'], capture_output=True, text=True)
split = subprocess.run(['git', 'diff',
                        '--name-only', 'HEAD', '--'], capture_output=True, text=True)
