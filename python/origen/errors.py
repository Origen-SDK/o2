class DuplicateDetectedError(Exception):
    def __init__(self, obj):
        klass = str(obj.__class__).split("'")[1]
        Exception.__init__(self, f"Cannot create instance of class '{klass}' named '{obj.name}', it already exists!") 