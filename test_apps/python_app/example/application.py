from origen.application import Base #, on_app_init
import origen

# This class represents this application and is automatically instantiated as origen.app
# It is required by Origen and should not be renamed or removed under any circumstances
class Application(Base):
    tester_resets = 0

    ''' Origen Application '''
    def yo(self):
        ''' Say hello '''
        print("hello from the app")

    def pattern_header(self, pattern):
        ''' Say hello from a generated pattern '''
        return ["Hello pattern from the application!"]

    # @on_app_init
    # def set_app_init(self):
    #     self.callback_app_init = True

    @staticmethod
    @origen.callbacks.listen_for("before_tester_reset")
    def count_tester_resets():
        Application.tester_resets += 1