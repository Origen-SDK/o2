import _origen;
# The base class of all application instances
class Base:
    config = _origen.app_config()
    current_target = _origen.app.current_target()

    def instantiate_block(self, path):
        print(f"Call to instatiate: {path}!")
