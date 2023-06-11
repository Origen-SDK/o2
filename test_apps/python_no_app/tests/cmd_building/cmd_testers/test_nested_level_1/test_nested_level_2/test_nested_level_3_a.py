import sys, pathlib
sys.path.append(str(pathlib.Path(__file__).parent.parent.parent))
from nested_common import say_hi

def run(**args):
    say_hi("3 (A)")
