import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import side_effect_lib as unreferenced

def used():
    return 1

def other():
    return unreferenced.thing()
