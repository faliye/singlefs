"""样本：python 驱动在 argv 列表里起第二台。"""
def peer_command(context, inner):
    memory = []
    return ["bash", context.multi_host_run, "--peer"] + memory + ["--"] + inner
