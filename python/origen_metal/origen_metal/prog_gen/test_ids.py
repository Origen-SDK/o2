from typing import Union, Optional, Tuple, Dict, Any
from pathlib import Path
from origen_metal import _origen_metal
_TestIDs = _origen_metal.prog_gen.test_ids.TestIDs
_AllocationOptions = _origen_metal.prog_gen.test_ids.AllocationOptions
_Pool = _origen_metal.prog_gen.test_ids.Pool

class TestIDs:
    """
    Provides an API to automatically assign bin, softbin and test numbers for a given test ID/name and then
    remember them for future use
    
    The database of test IDs can be saved to a file and reloaded later to pick up from where it left off
    
    ```python
    from origen_metal.prog_gen.test_ids import TestIDs
    
    # Create a new instance of TestIDs
    tids = TestIDs()
    
    # Restore a previously saved TestIDs instance
    tids = TestIDs("/path/to/my_test_ids.json")
    
    tids.include("bin", (100, 200))                  # Make a range of bin numbers available

    tids.include("number", (10000, 20000))           # Make a range of test numbers available
    
    tids.allocate("test1")    # => {"bin": 100, "number": 10000}
    tids.allocate("test1")    # => {"bin": 100, "number": 10000}   returns the same values

    tids.allocate("test2")    # => {"bin": 101, "number": 10001}

    # If manually assigning a bin number, let the allocator know so that it can remove it from the
    # pool of available bins
    tids.allocate("test3", bin=105)                  # => {"bin": 105, "number": 10002}
    
    # Assign a test number only
    tids.allocate("test4", bin=None, softbin=None)   # => {"number": 10003}
    ```
    """
    def __init__(self, file: Optional[Union[Path, str]] = None):
        if file is not None:
            self._backend = _TestIDs.from_file(str(file))
        else:
            self._backend = _TestIDs()
        
    def save(self, file: Optional[Union[Path, str]]):
        """
        Save the current state of the test ID database to a file, if this test ID instance was not originally
        created from a file then the file argument must be provided, otherwise it is optional and the original
        file will be used
        """
        self._backend.save(str(file))
        
    def _pool(self, kind: str, exclude: bool = False) -> _Pool:
        if kind == "bin" or kind == "bins":
            if exclude:
                return _Pool.BinExclude
            else:
                return _Pool.BinInclude
        elif kind == "softbin" or kind == "softbins":
            if exclude:
                return _Pool.SoftBinExclude
            else:
                return _Pool.SoftBinInclude
        elif kind == "number" or kind == "numbers" or kind == "test_number" or kind == "test_numbers":
            if exclude:
                return _Pool.NumberExclude
            else:
                return _Pool.NumberInclude
        else:
            raise ValueError(f"Unsupported kind: {kind}, must be 'bin', 'softbin' or 'number'")

    def increment(self, kind: str, int):
        """
        Set the increment size for the given bin, softbin or test number pool, defaults to 1
        
        ```python
        tids = TestIDs()

        tids.include("number", (10000, 20000))
        tids.size("number", 10)                  # Increments of 10 will be used for test numbers
        ```
        """
        pool = self._pool(kind, False)
        self._backend.set_increment(pool, int)
        
    def include(self, kind: str, *number_or_ranges: Union[int, Tuple[int, int]]):
        """
        Add a number (or range of numbers) to the available bin, softbin or test number pool

        ```python
        tids = TestIDs()

        tids.include("bin", 1, 2, 3, 4, 5)                  # Include individual numbers
        tids.include("softbin", (1, 10), (20, 30))          # Include ranges
        tids.include("test_number", 1000, (2000, 3000))
        ```
        """
        pool = self._pool(kind, False)
        for n in number_or_ranges:
            if isinstance(n, int):
                self._backend.push(pool, n)
            elif isinstance(n, Tuple):
                self._backend.push_range(pool, n[0], n[1])
            else:
                raise ValueError(f"Unsupported type: {n} ({type(n)}), must be an int or a range tuple like (1, 10)")

    def exclude(self, kind: str, *number_or_ranges: Union[int, range]):
        """
        Exclude a number (or range of numbers) from the available bin, softbin or test number pool
        
        ```python
        tids = TestIDs()

        tids.include("softbin", (10000, 20000))

        tids.exclude("softbin", 15000)           # Exclude a single number
        tids.exclude("softbin", (16000, 17000))  # Exclude a range
        ```
        """
        pool = self._pool(kind, True)
        for n in number_or_ranges:
            if isinstance(n, int):
                self._backend.push(pool, n)
            elif isinstance(n, Tuple):
                self._backend.push_range(pool, n[0], n[1])
            else:
                raise ValueError(f"Unsupported type: {n} ({type(n)}), must be an int or a range tuple like (1, 10)")

    def allocate(self, id: str, size: Optional[int] = None, **kwargs) -> Dict[str, Any]:
        """
        Allocate a bin, softbin and test number for the given test ID/name
        
        Passing `bin=None`, `softbin=None`, and/or `number=None` will prevent the allocator from assigning
        a number for that attribute
        
        Providing a number for either one, e.g. `bin=100`, will force the allocator to use that number and it will
        make it unavailable for future allocations (if it is within the range of available numbers)
        """
        opts = _AllocationOptions()
        if "bin" in kwargs:
            if kwargs["bin"] is not None:
                opts.bin = kwargs["bin"]
            else:
                opts.no_bin = True

        if "softbin" in kwargs:
            if kwargs["softbin"] is not None:
                opts.softbin = kwargs["softbin"]
            else:
                opts.no_softbin = True

        if "number" in kwargs:
            if kwargs["number"] is not None:
                opts.number = kwargs["number"]
            else:
                opts.no_number = True
                
        if size is not None:
            opts.size = size
            
        allocation = self._backend.allocate_with_options(id, opts)
        return allocation.to_hashmap()