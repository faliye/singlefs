"""样本的客户端：文件头「命令行」里写整条的那几行（真客户端是 scripts/prune-server/prune_server_client.py）。

  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <…>] -- <命令> [<参数>…]
      没有 CARGO_TARGET_DIR 不现编，出路是整条经 task-server 交（服务给编译目录、包装现编一遍 prune-server，不带 --server-binary）：
      task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest；
      自己编了、两台都读得到的二进制才另加 --server-binary，写在整条之外。
"""
