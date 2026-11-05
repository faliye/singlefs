"""样本的包装：文件头用法行与认选项的那一句（真包装是 scripts/prune-server/prune_server_client_selftest.py）。

  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <命令> [<参数>…]
"""


def with_temporary_service(arguments):
    separator = arguments.index("--")
    options, command = arguments[:separator], arguments[separator + 1:]
    unknown = [word for position, word in enumerate(options) if word != "--server-binary" and (position == 0 or options[position - 1] != "--server-binary")]
    return unknown, command
