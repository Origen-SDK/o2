for i in range(3):
  with produce_pattern(postfix=f"index{i}") as pat:
    tester().cc(f"Producing pattern at index {i}")
    tester().repeat(10)
