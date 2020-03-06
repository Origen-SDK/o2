import origen
from contextlib import contextmanager

@contextmanager
def write_transaction(bit_collection):
    bc = bit_collection._internal_start_write_transaction()
    yield bc
    bc._internal_end_write_transaction()
    write(bit_collection)
        
@contextmanager
def verify_transaction(bit_collection, enable=None):
    bc = bit_collection._internal_start_verify_transaction()
    yield bc
    bc._internal_end_verify_transaction()
    verify(bit_collection, enable=enable, _preset=True)

def verify(bit_collection, enable=None, _preset=False):
    bit_collection._internal_verify(enable, _preset)
    _get_controller(bit_collection).verify_register(bit_collection)

def write(bit_collection):
    bit_collection._internal_write()
    _get_controller(bit_collection).write_register(bit_collection)

def _get_controller(bit_collection):
    obj = origen
    for x in bit_collection.model_path().split("."):
        obj = getattr(obj, x)
    return obj    
