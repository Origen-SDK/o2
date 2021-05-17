from ...controller import Controller as Parent
from origen.controller import TopLevel
import origen


class Controller(Parent):
    def __init__(self):
        Parent.__init__(self)
        self.startups_called = 0
        self.shutdowns_called = 0
        self.startup_source = None
        self.shutdown_source = None

    @TopLevel.startup
    def startup(self, **kwargs):
        origen.tester.set_timeset("simple")
        origen.tester.pin_header = self.pin_headers[kwargs.get(
            "pin_header", "pins-for-toggle")]
        origen.tester.repeat(100)
        self.startups_called += 1
        self.startup_source = "eagle"

    @TopLevel.shutdown
    def shutdown(self, **kwargs):
        origen.tester.repeat(10)
        self.shutdowns_called += 1
        self.shutdown_source = "eagle"
