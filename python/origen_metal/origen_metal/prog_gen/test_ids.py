from typing import Union, Optional, Tuple, Dict, Any
from pathlib import Path
from origen_metal import _origen_metal
_TestIDInterface = _origen_metal.prog_gen.test_ids.TestIDInterface

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
        self._backend = _TestIDInterface()
        
    def save(self, file: Optional[Union[Path, str]]):
        """
        Save the current state of the test ID database to a file, if this test ID instance was not originally
        created from a file then the file argument must be provided, otherwise it is optional and the original
        file will be used
        """
        self._backend.save(file)
        
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
        if kind != "bin" and kind != "softbin" and kind != "number":
            raise ValueError(f"Unsupported kind: {kind}, must be 'bin', 'softbin' or 'number'")

        for n in number_or_ranges:
            if isinstance(n, int):
                self._backend.configure(kind, False, n, None)
            elif isinstance(n, Tuple):
                self._backend.configure(kind, False, n[0], n[1])
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
        for n in number_or_ranges:
            if isinstance(n, int):
                self._backend.configure(kind, True, n, None)
            elif isinstance(n, Tuple):
                self._backend.configure(kind, True, n[0], n[1])
            else:
                raise ValueError(f"Unsupported type: {n} ({type(n)}), must be an int or a range tuple like (1, 10)")

    def allocate(self, id: str, **kwargs) -> Dict[str, Any]:
        """
        Allocate a bin, softbin and test number for the given test ID/name
        
        Passing `bin=None`, `softbin=None`, and/or `number=None` will prevent the allocator from assigning
        a number for that attribute
        
        Providing a number for either one, e.g. `bin=100`, will force the allocator to use that number and it will
        make it unavailable for future allocations (if it is within the range of available numbers)
        """
        if "bin" in kwargs:
            if kwargs["bin"] is not None:
                bin = kwargs["bin"]
                no_bin = False
            else:
                bin = None
                no_bin = True
        else:
            bin = None
            no_bin = False

        if "softbin" in kwargs:
            if kwargs["softbin"] is not None:
                softbin = kwargs["softbin"]
                no_softbin = False
            else:
                softbin = None
                no_softbin = True
        else:
            softbin = None
            no_softbin = False

        if "number" in kwargs:
            if kwargs["number"] is not None:
                number = kwargs["number"]
                no_number = False
            else:
                number = None
                no_number = True
        else:
            number = None
            no_number = False
            
        return self._backend.allocate(id, no_bin, no_softbin, no_number, bin, softbin, number)