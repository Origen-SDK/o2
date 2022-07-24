import pytest, abc, inspect


# Test Dict-Like API
class Fixture_DictLikeAPI(abc.ABC):
    class Expected:
        def __init__(self, parameters):
            self.keys = parameters["keys"]
            if inspect.isclass(parameters["klass"]):
                self.klass = [parameters["klass"]] * self.len
            else:
                self.klass = parameters["klass"]
            self.not_in_dut = parameters.get("not_in_dut", "not_in_dut")

        @property
        def len(self):
            return len(self.keys)

        @property
        def in_dut(self):
            return self.keys[0]

    @abc.abstractmethod
    def boot_dict_under_test(self):
        pass

    @abc.abstractmethod
    def parameterize(self):
        pass

    def init_dict_under_test(self):
        pass

    # Pytest skips classes which have an __init__ constructor (http://doc.pytest.org/en/latest/goodpractices.html#conventions-for-python-test-discovery)
    # get around this by using a fixture on the first test to boot the object.
    @pytest.fixture
    def boot(self):
        if not getattr(self.__class__, "_init_dict_under_test_run", False):
            self.init_dict_under_test()
            setattr(self.__class__, '_init_dict_under_test_run', True)
        self.expected = self.Expected(self.parameterize())
        self.dut = self.boot_dict_under_test()

    def test_keys(self, boot):
        assert self.dut.keys
        assert isinstance(self.dut.keys(), list)
        assert self.dut.keys() == self.expected.keys

    def test_values(self, boot):
        assert self.dut.values
        assert isinstance(self.dut.values(), list)
        assert len(self.dut.values()) == self.expected.len
        assert isinstance(self.dut.values()[0], self.expected.klass[0])

    def test_items(self, boot):
        items = self.dut.items()
        assert items
        assert isinstance(items, list)
        assert isinstance(items[0], tuple)
        assert len(items) == self.expected.len
        assert list(i[0] for i in items) == self.expected.keys

    def test_contains(self, boot):
        assert self.expected.in_dut in self.dut
        assert self.expected.not_in_dut not in self.dut

    def test_len(self, boot):
        assert self.dut.__len__
        assert len(self.dut) == self.expected.len

    def test_getitem(self, boot):
        assert self.dut.get
        assert self.dut.get(self.expected.in_dut) is not None
        assert self.dut.get(self.expected.not_in_dut) is None

    def test_index_notation(self, boot):
        assert self.dut[self.expected.in_dut] is not None
        with pytest.raises(KeyError):
            self.dut[self.expected.not_in_dut]

    def test_iterating(self, boot):
        i = 0
        for _i, k in enumerate(self.dut):
            assert k == self.expected.keys[_i]
            i += 1
        assert i == self.expected.len

    def test_iterating_through_items(self, boot):
        i = 0
        for _i, (k, v) in enumerate(self.dut.items()):
            assert k == self.expected.keys[_i]
            assert isinstance(v, self.expected.klass[_i])
            i += 1
        assert i == self.expected.len

    def test_dict_conversion(self, boot):
        d = dict(self.dut)
        assert isinstance(d, dict)
        assert list(d.keys()) == self.expected.keys

    def test_list_converstion(self, boot):
        l = list(self.dut)
        assert isinstance(l, list)
        assert l == self.expected.keys


# Test List-Like API
class Fixture_ListLikeAPI(abc.ABC):
    class Expected:
        def __init__(self, parent, parameters):
            self.parent = parent
            self.slice_klass = parameters.get("slice_klass", list)

        @property
        def len(self):
            return 3

        def verify_i0(self, i):
            return self.parent.verify_i0(i)

        def verify_i1(self, i):
            return self.parent.verify_i1(i)

        def verify_i2(self, i):
            return self.parent.verify_i2(i)

    def parameterize(self):
        return {}

    @abc.abstractmethod
    def boot_list_under_test(self):
        ...

    @abc.abstractmethod
    def verify_i0(self, i):
        ...

    @abc.abstractmethod
    def verify_i1(self, i):
        ...

    @abc.abstractmethod
    def verify_i2(self, i):
        ...

    @pytest.fixture
    def boot(self):
        self.expected = self.Expected(self, self.parameterize())
        self.lut = self.boot_list_under_test()

    def test_len(self, boot):
        assert self.lut.__len__
        assert len(self.lut) == self.expected.len

    def test_indexing(self, boot):
        i0 = self.lut[0]
        i1 = self.lut[1]
        i2 = self.lut[2]
        self.expected.verify_i0(i0)
        self.expected.verify_i1(i1)
        self.expected.verify_i2(i2)

    def test_negative_indexing(self, boot):
        i0 = self.lut[-3]
        i1 = self.lut[-2]
        i2 = self.lut[-1]
        self.expected.verify_i0(i0)
        self.expected.verify_i1(i1)
        self.expected.verify_i2(i2)

    def test_range_a(self, boot):
        items = self.lut[:2]
        assert isinstance(items, self.expected.slice_klass)
        assert len(items) == 2
        self.expected.verify_i0(items[0])
        self.expected.verify_i1(items[1])

    def test_range_b(self, boot):
        items = self.lut[1:]
        assert isinstance(items, self.expected.slice_klass)
        assert len(items) == 2
        self.expected.verify_i1(items[0])
        self.expected.verify_i2(items[1])

    def test_range_c(self, boot):
        items = self.lut[:]
        assert isinstance(items, self.expected.slice_klass)
        assert len(items) == 3
        self.expected.verify_i0(items[0])
        self.expected.verify_i1(items[1])
        self.expected.verify_i2(items[2])

    def test_range_d(self, boot):
        items = self.lut[::2]
        assert isinstance(items, self.expected.slice_klass)
        assert len(items) == 2
        self.expected.verify_i0(items[0])
        self.expected.verify_i2(items[1])

    def test_range_e(self, boot):
        items = self.lut[0:2]
        assert isinstance(items, self.expected.slice_klass)
        assert len(items) == 2
        self.expected.verify_i0(items[0])
        self.expected.verify_i1(items[1])

    def test_negative_range_a(self, boot):
        ritems = self.lut[-2:]
        assert isinstance(ritems, self.expected.slice_klass)
        assert len(ritems) == 2
        self.expected.verify_i1(ritems[0])
        self.expected.verify_i2(ritems[1])

    def test_negative_range_b(self, boot):
        ritems = self.lut[:-1]
        assert isinstance(ritems, self.expected.slice_klass)
        assert len(ritems) == 2
        self.expected.verify_i0(ritems[0])
        self.expected.verify_i1(ritems[1])

    def test_range_reversal_a(self, boot):
        ritems = self.lut[::-1]
        assert isinstance(ritems, self.expected.slice_klass)
        assert len(ritems) == 3
        self.expected.verify_i2(ritems[0])
        self.expected.verify_i1(ritems[1])
        self.expected.verify_i0(ritems[2])

    def test_range_reversal_b(self, boot):
        ritems = self.lut[2:0:-1]
        assert isinstance(ritems, self.expected.slice_klass)
        assert len(ritems) == 2
        self.expected.verify_i2(ritems[0])
        self.expected.verify_i1(ritems[1])

    def test_range_reversal_c(self, boot):
        ritems = self.lut[::-2]
        assert isinstance(ritems, self.expected.slice_klass)
        assert len(ritems) == 2
        self.expected.verify_i2(ritems[0])
        self.expected.verify_i0(ritems[1])

    def test_iterating(self, boot):
        for i, x in enumerate(self.lut):
            getattr(self.expected, f"verify_i{i}")(x)

    # TODO need to revisit contains. Maybe make an option
    #def test_contains(self, boot):
    #  assert self.expected.in_list in self.lut
    #  assert self.expected.not_in_list not in self.lut

    def test_list(self, boot):
        l = list(self.lut)
        assert isinstance(l, list)
        assert len(l) == 3
        self.expected.verify_i0(l[0])
        self.expected.verify_i1(l[1])
        self.expected.verify_i2(l[2])

    # Note: the contains is a bit tricky since we're going back to the Origen DB
    # and getting a different object each time. So, there's really now way to
    # implicitly compare.
    # Plan is to make implementors to provide an 'eqls' for backend objects.
    # However, a quick test here just makes sure it doesn't crash.
    def test_contains_kinda(self, boot):
        assert None not in self.lut

    def test_exception_on_step_0(self, boot):
        with pytest.raises(ValueError):
            self.lut[::0]

    def test_over_indexing(self, boot):
        l = self.lut[0:4]
        assert isinstance(l, self.expected.slice_klass)
        assert len(l) == 3
        l = self.lut[-3:-4]
        assert isinstance(l, self.expected.slice_klass)
        assert len(l) == 0

    def test_under_indexing(self, boot):
        l = self.lut[4:]
        assert isinstance(l, self.expected.slice_klass)
        assert len(l) == 0
        l = self.lut[4:1]
        assert isinstance(l, self.expected.slice_klass)
        assert len(l) == 0
        l = self.lut[4:1:-1]
        assert isinstance(l, self.expected.slice_klass)
        assert len(l) == 1

    def test_exception_on_index_out_of_bounds(self, boot):
        with pytest.raises(IndexError):
            self.lut[3]
        with pytest.raises(IndexError):
            self.lut[-4]

    # Noticed during debug that an incorrect type would cause a panic on the Rust
    # side. Case here is to catch that.
    # (Side Note: Was caused by unwrapping a <PyAny>.extract instead of checking for an error)
    def test_exception_on_slicing_with_other_types(self, boot):
        with pytest.raises(TypeError):
            self.lut[1, 2]
