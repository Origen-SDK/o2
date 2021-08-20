import origen_metal


def test_ping_extension():
    assert origen_metal.ping() == "pong"
