from ...controller import Controller as Parent
import origen


class Controller(Parent):
    def startup(self, **kwargs):
        origen.tester.set_timeset("simple")
        origen.tester.pin_header = self.pin_headers["pins-for-toggle"]
        origen.tester.repeat(100)

    def shutdown(self, **kwargs):
        origen.tester.repeat(10)
