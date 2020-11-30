from origen.controller import TopLevel


class Controller(TopLevel):
    def write_register(self, reg_or_val, size=None, address=None, **kwargs):
        pass

    def verify_register(self, reg_or_val, size=None, address=None, **kwargs):
        pass
