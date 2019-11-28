from origen.application import Base

# This class represents this application and is automatically instantiated as origen.app
# It is required by Origen and should not be renamed or removed under any circumstances
class Application(Base):
    def yo(self):
        print("hello from the app")
