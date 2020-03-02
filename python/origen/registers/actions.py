from contextlib import contextmanager

@contextmanager
def write_transaction(bit_collection):
    bc = bit_collection.start_write_transaction()
    yield bc
    bc.end_write_transaction()
        
@contextmanager
def verify_transaction(bit_collection):
    bc = bit_collection.start_verify_transaction()
    yield bc
    bc.end_verify_transaction()