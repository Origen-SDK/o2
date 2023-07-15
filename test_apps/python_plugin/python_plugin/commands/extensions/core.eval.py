import origen
from origen.boot import before_cmd, after_cmd

@before_cmd
def before_cmd(**ext_kwargs):
    if "say_hi_before_eval" in ext_kwargs:
        print("Hi from python-plugin during 'eval'!")

@after_cmd
def after_cmd(**ext_kwargs):
    if "say_hi_after_eval" in ext_kwargs:
        print("Hi again from python-plugin during 'eval'!")
