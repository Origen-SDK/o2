for i in range(3):
    with Pattern(postfix=f"index{i}") as pat:
        tester().cc(f"Producing pattern at index {i}")
        tester().repeat(10)
