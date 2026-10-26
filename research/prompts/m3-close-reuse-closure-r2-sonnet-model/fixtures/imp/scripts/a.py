import importlib.util, os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import unused_dep  # 模块级 import，名字没被引用

def run():
    spec = importlib.util.spec_from_file_location("loaded", os.path.join(HERE, "loaded.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.used()
