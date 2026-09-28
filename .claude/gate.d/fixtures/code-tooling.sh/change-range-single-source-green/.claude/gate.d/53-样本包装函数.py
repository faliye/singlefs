# 绿样本：同一个文件里的包装函数带着选项
import subprocess
def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], capture_output=True, text=True)
tracked = git('ls-files', '--error-unmatch', 'a.md').returncode == 0
