import origen

def before_cmd():
    print("before cmd!!")
    print(origen.current_command.args)
    if "say_hi_before_eval" in origen.current_command.args:
        print("Hi from python-plugin during 'eval'!")

def after_cmd():
    if "say_hi_before_eval" in origen.current_command.args:
        print("Hi again from python-plugin during 'eval'!")
