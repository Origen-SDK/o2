def run(input, repeat=False):
    s = f"Echoing '" + ','.join(input) + "' from python_plugin"
    print(s)
    if repeat:
        print(f"(repeat) {s}")
