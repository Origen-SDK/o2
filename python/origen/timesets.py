import origen


class Proxy:
    def __init__(self, controller):
        self.controller = controller

    @property
    def timesets(self):
        return origen.dut.db.timesets(self.controller.model_id)

    def add_timeset(self, name, period=None, **kwargs):
        return origen.dut.db.add_timeset(self.controller.model_id, name,
                                         period, **kwargs)

    def timeset(self, name):
        return origen.dut.db.timeset(self.controller.model_id, name)

    @classmethod
    def api(cls):
        return [
            'timesets',
            'add_timeset',
            'timeset',
        ]


class Loader:
    def __init__(self, controller):
        self.controller = controller

    def Timeset(self, name, **kwargs):
        return self.controller.add_timeset(name, **kwargs)

    def api(self):
        return {
            "Timeset": self.Timeset,
        }
