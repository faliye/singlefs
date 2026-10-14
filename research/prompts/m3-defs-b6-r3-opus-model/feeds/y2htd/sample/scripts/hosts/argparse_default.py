# 攻方样本（Y2）：argparse 的并行度默认值写死 8
import argparse
parser = argparse.ArgumentParser()
parser.add_argument("--jobs", type=int, default=8)
