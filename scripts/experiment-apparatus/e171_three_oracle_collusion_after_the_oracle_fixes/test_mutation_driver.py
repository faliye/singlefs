"""研究一侧变异驱动 mutation_driver.py 的副本怎么搭（第 2 次重跑登记 research/prompts/e171-r2-prereg.md 修订 5 第 5.2 条）：
副本里的研究目录往上三层要像仓根，模块按那条相对路径取的 .claude/hooks/lib_work_root.py 才取得到，--scratch 指哪里都一样。
每条用例自己建、自己删临时目录；不起装置、不编。"""
from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import unittest

import mutation_driver


class FreshCopyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.mkdtemp(prefix="e171-mutation-driver-test-")

    def tearDown(self) -> None:
        shutil.rmtree(self.scratch, ignore_errors=True)

    def test_the_copy_reaches_the_work_root_library_three_levels_up(self) -> None:
        copy = mutation_driver.fresh_copy(self.scratch, "probe")
        library = os.path.join(copy, "..", "..", "..", ".claude", "hooks", "lib_work_root.py")
        self.assertTrue(os.path.isfile(library), f"副本 {copy} 往上三层取不到 .claude/hooks/lib_work_root.py")

    def test_the_research_directory_in_the_copy_is_a_real_copy_not_a_link(self) -> None:
        copy = mutation_driver.fresh_copy(self.scratch, "probe")
        self.assertFalse(os.path.islink(copy))
        self.assertNotEqual(os.path.realpath(copy), os.path.realpath(mutation_driver.DIRECTORY))
        self.assertFalse(os.path.islink(os.path.join(copy, "mutation_driver.py")))
        self.assertTrue(copy.startswith(mutation_driver.copy_root(self.scratch, "probe") + os.sep))

    def test_the_copy_root_does_not_carry_the_git_directory(self) -> None:
        mutation_driver.fresh_copy(self.scratch, "probe")
        self.assertNotIn(".git", os.listdir(mutation_driver.copy_root(self.scratch, "probe")))

    def test_a_module_that_loads_the_work_root_library_by_relative_path_imports_inside_the_copy(self) -> None:
        copy = mutation_driver.fresh_copy(self.scratch, "probe")
        completed = subprocess.run([sys.executable, "-B", "-c", "import revision_8_placeholders"], cwd=copy,
                                   env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"), capture_output=True, text=True, timeout=120)
        self.assertEqual(completed.returncode, 0, completed.stderr)

    def test_removing_the_copy_root_leaves_the_real_repository_alone(self) -> None:
        mutation_driver.fresh_copy(self.scratch, "probe")
        shutil.rmtree(mutation_driver.copy_root(self.scratch, "probe"))
        self.assertTrue(os.path.isfile(os.path.join(mutation_driver.REPOSITORY_ROOT, ".claude", "hooks", "lib_work_root.py")))
        self.assertTrue(os.path.isfile(os.path.join(mutation_driver.DIRECTORY, "mutation_driver.py")))


if __name__ == "__main__":
    unittest.main()
