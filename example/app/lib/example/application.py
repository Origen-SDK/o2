from origen.application import Base
# This class represents this application and is automatically instantiated as origen.app
# It is required by Origen and should not be renamed or removed under any circumstances
class Application(Base):
    def yo(self):
        print("hello from the app")

# Also see:
#https://docs.python.org/2/library/imp.html#imp.load_source
#
#with open("/home/stephen/Code/github/reboot/example/app/lib/p2.py") as f:
#    code = compile(f.read(), "/home/stephen/Code/github/reboot/example/app/lib/p2.py", 'exec')
#    exec(code, global_vars, local_vars)
