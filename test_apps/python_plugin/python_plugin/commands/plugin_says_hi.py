def run(times=1, loudly=False, to=None):
    t = int(times)
    print(f"Saying hi {t} time(s)...")
    for _ in range(t):
        s = f"Hi{(' ' + ','.join(to)) if to else ''} from the python plugin!"
        print(s if not loudly else s.upper())