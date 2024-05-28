import origen, _origen

def test_app_is_none():
    print(origen.app)
    assert isinstance(origen.app, origen.application.Base)

def test_is_app_present():
    assert origen.is_app_present is True
    assert _origen.is_app_present() is True
    assert origen.status["is_app_present"] is True