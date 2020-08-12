from origen.application import Base


# This class represents this application and is automatically instantiated as origen.app
# It is required by Origen and should not be renamed or removed under any circumstances
class Application(Base):
    ''' Origen Application '''
    def yo(self):
        ''' Say hello '''
        print("hello from the app")

    def pattern_header(self, pattern):
        ''' Say hello from a generated pattern '''
        return ["Hello pattern from the application!"]
