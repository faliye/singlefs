"""样本的钩子库：只有拒绝说明给整条写法的那一个函数（真库是 .claude/hooks/lib_project_code_runs.py）。"""


def temporary_service_command(key):
    bare = f"{'python3' if key.endswith('.py') else 'bash'} {key} --selftest"
    return bare
