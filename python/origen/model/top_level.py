import _origen
class TopLevel:

    db = None

    def __init__(self):
        self.db = _origen.model.ModelDB("tbd")
