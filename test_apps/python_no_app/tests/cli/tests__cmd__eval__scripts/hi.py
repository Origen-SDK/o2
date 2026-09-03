preface = locals().get("preface", None)
hi = "hi!"
if not preface:
    preface = "eval_script__say_hi"
print(f"{preface}: {hi}")
