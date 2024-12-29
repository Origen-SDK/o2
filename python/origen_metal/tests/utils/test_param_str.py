from origen_metal.utils.param_str import ParamStr, MultiParamStr
import pytest

class Common:
    @pytest.fixture
    def a1(self):
        return "arg1"

    @pytest.fixture
    def a2(self):
        return "arg2"

    @pytest.fixture
    def a3(self):
        return "arg3"

    @pytest.fixture
    def a4(self):
        return "arg4"

    @pytest.fixture
    def a5(self):
        return "arg5"

    @pytest.fixture
    def a6(self):
        return "arg6"

    @property
    def m(self):
        return "missing"

    @pytest.fixture
    def missing(self):
        return self.m

    @pytest.fixture
    def leading(self):
        return "leading"

    @pytest.fixture
    def reparse_error(self):
        return r"ParamStr has already been parsed. Please use 'param_str.clear\(\)', before parsing new input"

    @pytest.fixture
    def leading_str_not_allowed_msg(self):
        return "Attempted to set leading value but 'allows_leading_str' is not allowed"

    @pytest.fixture
    def set_leading_str_after_parse_error_msg(self):
        return "Attempted to change ParamStr's 'allows_leading_str' setting after parsing, which is not allowed"

    def removing_missing_def_err_msg(self, default):
        return f"No parameter '{default}' to remove from ParamStr's defaults"

    def not_allowed_param_msg(self, param):
        return f"ParamStr encountered parameter '{param}', which is not an expected parameter"

    @pytest.fixture
    def defs_update_after_parse_err_msg(self):
        return "Attempted to update ParamStr's default values after parsing, which is not allowed"

    @pytest.fixture
    def update_allows_non_defaults_after_parse_err_msg(self):
        return "Cannot set ParamStr's allows_non_defaults with no default parameters"

class TestStandaloneParamStr(Common):
    @classmethod
    def to_vals(cls, vals):
        return ':'.join(vals)

    @classmethod
    def to_input_str(cls, args, vals):
        in_str = []
        for i, arg in enumerate(args):
            if vals[i] is None:
                in_str.append(arg)
            else:
                in_str.append(f"{arg}:{cls.to_vals(vals[i])}")
        return '~'.join(in_str)

    def assert_unparsed(self, p):
        not_parsed_error_msg = "ParamStr has not yet been parsed!"
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert p.leading
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert str(p)
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert p.to_str()
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert dict(p)
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert p.keys()
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert p.values()
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert p.items()
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert len(p)
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert p.get(self.m)
        with pytest.raises(RuntimeError, match=not_parsed_error_msg):
            assert p[self.m]

    def test_unparsed_param_str(self, missing):
        p = ParamStr()
        assert p.parsed is None
        assert p.allows_leading_str is False
        assert p.defaults == None
        assert p.allows_non_defaults is None
        self.assert_unparsed(p)

    def test_empty_param_str(self):
        in_str = ""
        p = ParamStr()
        parsed = p.parse(in_str)
        assert isinstance(parsed, ParamStr)

        assert p.to_str() == in_str
        assert str(p) == in_str
        assert p.leading == None

        assert dict(p) == {}
        assert p.keys() == []
        assert p.values() == []
        assert p.items() == []
        assert len(p) == 0

        with pytest.raises(KeyError, match="No key 'missing'"):
            assert p["missing"]
        assert p.get("missing") is None

    def test_empty_setup_after_parse(self):
        in_str = ""
        p = ParamStr()
        p.parse(in_str)

        # These should not change after parsing
        assert p.allows_leading_str is False
        assert p.defaults == None
        assert p.allows_non_defaults is None

    def test_single_param_str(self):
        in_str = "arg1"
        p = ParamStr()
        parsed = p.parse(in_str)

        assert str(p) == in_str
        assert p.leading == None
        assert id(parsed) == id(p)

        assert dict(p) == {in_str: []}
        assert p.keys() == [in_str]
        assert p.values() == [[]]
        assert p.items() == [(in_str, [])]
        assert len(p) == 1
        assert p[in_str] == []
        assert p.get(in_str) == []

    def test_double_param_str(self, a1, a2):
        in_str = f"{a1}~{a2}"
        p = ParamStr()
        p.parse(in_str)

        assert str(p) == in_str
        assert p.leading == None

        assert dict(p) == {a1: [], a2: []}
        assert p.keys() == [a1, a2]
        assert p.values() == [[], []]
        assert p.items() == [(a1, []), (a2, [])]
        assert len(p) == 2
        assert p[a1] == []
        assert p.get(a2) == []

    def test_val(self, a1):
        v1 = "v1"
        in_str = f"{a1}:{v1}"
        p = ParamStr()
        p.parse(in_str)

        assert str(p) == in_str
        assert p.leading == None

        assert dict(p) == {a1: [v1]}
        assert p.keys() == [a1]
        assert p.values() == [[v1]]
        assert p.items() == [(a1, [v1])]
        assert len(p) == 1
        assert p[a1] == [v1]
        assert p.get(a1) == [v1]

    def test_multi_val(self, a1):
        vals = ["v1", "v2", "v3"]
        in_str = f"{a1}:{':'.join(vals)}"
        p = ParamStr()
        p.parse(in_str)

        assert str(p) == in_str
        assert p.leading == None

        assert dict(p) == {a1: vals}
        assert p.keys() == [a1]
        assert p.values() == [vals]
        assert p.items() == [(a1, vals)]
        assert len(p) == 1
        assert p[a1] == vals
        assert p.get(a1) == vals

    def test_multi_arg_multi_vals(self, a1, a2, a3, a4, a5):
        v = ["v1", "v2"]
        w = ["w1"]
        x = ["x1", "x2", "x3"]
        in_str = self.to_input_str([a1, a2, a3, a4, a5], [v, w, None, x, None])
        p = ParamStr()
        p.parse(in_str)

        assert str(p) == in_str
        assert p.leading == None

        assert dict(p) == {
            a1: v,
            a2: w,
            a3: [],
            a4: x,
            a5: []
        }
        assert p.values() == [v, w, [], x, []]
        assert p.items() == [
            (a1, v),
            (a2, w),
            (a3, []),
            (a4, x),
            (a5, [])
        ]
        assert len(p) == 5
        assert p[a1] == v
        assert p.get(a3) == []

    @classmethod
    def assert_param_str(cls, p, as_str, as_dict=None, leading=None, raw=None):
        if as_dict is None:
            as_dict = {}
        assert str(p) == as_str
        assert p.leading == leading
        assert p.raw == raw
        assert p.allows_leading_str == (leading is not None)

        keys = list(as_dict.keys())
        assert dict(p) == as_dict
        assert p.keys() == keys
        assert p.values() == list(as_dict.values())
        assert p.items() == list(as_dict.items())
        assert len(p) == len(as_dict)
        if len(as_dict) > 0:
            k = keys[0]
            assert p[k] == as_dict[k]
            if len(as_dict) > 1:
                k = keys[-1]
            assert p.get(k) == as_dict[k]

        k = "missing"
        with pytest.raises(KeyError, match="No key 'missing'"):
            assert p[k]
        assert p.get(k) is None

    def test_leading_str(self, a1, a2):
        in_str = "blah"
        p = ParamStr(True)
        p.parse(in_str)
        self.assert_param_str(p, in_str, None, leading=in_str, raw=in_str)

        in_str = "blah~"
        p = ParamStr(allows_leading_str=True)
        p.parse(in_str)
        self.assert_param_str(p, "blah", None, "blah", in_str)

        in_str = "blah~arg1"
        p = ParamStr(allows_leading_str=True)
        p.parse(in_str)
        self.assert_param_str(p, in_str, {a1: []}, "blah", in_str)

        in_str = f"blah blah~{a1}:v1:v2~{a2}:w2"
        p = ParamStr(allows_leading_str=True)
        p.parse(in_str)
        self.assert_param_str(p, in_str, {a1: ["v1", "v2"], a2: ["w2"]}, "blah blah", in_str)

        in_str = "~blah"
        p = ParamStr(allows_leading_str=True)
        p.parse(in_str)
        self.assert_param_str(p, "~blah", {"blah": []}, "", in_str)

    def test_key_order(self, a3, a2, a1):
        in_str = f"{a3}~{a2}:val~{a1}:v1:v2"
        p = ParamStr.and_parse(in_str)
        assert p.keys() == [a3, a2, a1]

    def test_error_on_multi_param_str(self):
        p = ParamStr()
        with pytest.raises(RuntimeError, match="ParamStr input contains the multi-param-separate '~:~', which is not allowed. Please use a MultiParamStr instead"):
            p.parse("a1~a2:v1:v2~a3~:~b1:w1~b2~:~c1~a1:x1:x2")
        self.assert_unparsed(p)

    def test_error_on_duplicate_params(self, a1):
        p = ParamStr()
        with pytest.raises(RuntimeError, match=f"ParamStr encountered a duplicate parameter '{a1}', which is not allowed"):
            p.parse(f"{a1}~{a1}:v1")
        self.assert_unparsed(p)

        with pytest.raises(RuntimeError, match=f"ParamStr encountered a duplicate parameter '{a1}', which is not allowed"):
            p.parse(f"{a1}~{a1}")
        self.assert_unparsed(p)

        with pytest.raises(RuntimeError, match=f"ParamStr encountered a duplicate parameter '{a1}', which is not allowed"):
            p.parse(f"{a1}:v0~{a1}:v1")
        self.assert_unparsed(p)

    def test_error_on_invalid_inputs(self):
        p = ParamStr()
        with pytest.raises(RuntimeError, match="ParamStr encountered a parameter with an empty key, which is not allowed"):
            p.parse("~")
        self.assert_unparsed(p)

        with pytest.raises(RuntimeError, match="ParamStr encountered a parameter with an empty key, which is not allowed"):
            p.parse("~~")
        self.assert_unparsed(p)

        with pytest.raises(RuntimeError, match="ParamStr found value separator as first character, which is not allowed"):
            p.parse(":")
        self.assert_unparsed(p)

        with pytest.raises(RuntimeError, match="ParamStr encountered a parameter with an empty key, which is not allowed"):
            p.parse("arg~")
        self.assert_unparsed(p)

        with pytest.raises(RuntimeError, match="ParamStr encountered a parameter with a value separator but no key, which is not allowed"):
            p.parse("arg~:")
        self.assert_unparsed(p)

        with pytest.raises(RuntimeError, match="ParamStr encountered a parameter with an empty key, which is not allowed"):
            p.parse("~arg")
        self.assert_unparsed(p)

        p = ParamStr(allows_leading_str=True)
        with pytest.raises(RuntimeError, match="ParamStr encountered a parameter with an empty key, which is not allowed"):
            p.parse("blah~:")
        self.assert_unparsed(p)

    def test_creating_and_parsing_in_one_go(self, a1, a2, a3, a4, a5):
        p = ParamStr.and_parse(f"{a1}:v1~{a2}")
        assert p.allows_leading_str == False
        assert p.defaults == None
        assert p.allows_non_defaults == None
        assert p.parsed == { a1: ["v1"], a2: []}
        assert p.raw == f"{a1}:v1~{a2}"
        assert p.leading == None

        defs = {a2: None, a3: ['v1', 'v2'], a4: 'v3', a5: 'v4'}
        p = ParamStr.and_parse(f"leading~{a5}:v:4", allows_leading_str=True, defaults=defs)
        assert p.allows_leading_str == True
        assert p.defaults == {a2: None, a3: ['v1', 'v2'], a4: ['v3'], a5: ['v4']}
        assert p.allows_non_defaults == False
        assert p.parsed == { a5: ['v', '4'], a3: ['v1', 'v2'], a4: ['v3']}
        assert p.raw == f"leading~{a5}:v:4"
        assert p.leading == "leading"

        defs = {a2: None, a3: []}
        p = ParamStr.and_parse(f"leading~{a1}", allows_leading_str=True, defaults=defs, allows_non_defaults=True)
        assert p.allows_leading_str == True
        assert p.defaults == defs
        assert p.allows_non_defaults == True
        assert p.parsed == { a1: [], a3: []}
        assert p.raw == f"leading~{a1}"
        assert p.leading == "leading"

    def test_default_keys(self, a1, a2):
        defs = {a1: None, a2: 'default'}
        p = ParamStr(allows_leading_str=False, defaults=defs)
        assert p.defaults == {a1: None, a2: ['default']}
        assert p.parsed is None
        assert p.allows_non_defaults is False

        # Try with empty input
        in_str = ""
        p.parse(in_str)
        assert p.parsed == {a2: ['default']}
        assert str(p) == f"{a2}:default"

        # Try with some values
        p = ParamStr(allows_leading_str=False, defaults=defs)
        in_str = f"{a1}~{a2}:not:default"
        p.parse(in_str)
        assert p.parsed == {a1: [], a2: ['not', 'default']}
        assert str(p) == f"{a1}~{a2}:not:default"

    def test_defaults_with_array(self, a1, a2, a3, a4):
        defs = {a1: ['default_1', 'default_2'], a2: ['default'], a3: [], a4: None}
        p = ParamStr(allows_leading_str=False, defaults=defs)
        assert p.defaults == defs
        assert p.parsed is None
        assert p.allows_non_defaults is False

        in_str = ""
        p = ParamStr(allows_leading_str=False, defaults=defs)
        p.parse(in_str)
        assert p.parsed == {a1: ['default_1', 'default_2'], a2: ['default'], a3: []}
        assert str(p) == f"{a1}:default_1:default_2~{a2}:default~{a3}"

        in_str = f"{a2}~{a3}:a:b:c:d"
        p = ParamStr(allows_leading_str=False, defaults=defs)
        p.parse(in_str)
        assert p.parsed == {a1: ['default_1', 'default_2'], a2: [], a3: ['a', 'b', 'c', 'd']}
        assert str(p) == f"{a2}~{a3}:a:b:c:d~{a1}:default_1:default_2"

    def test_key_ordering_with_defaults(self, a1, a2, a3):
        in_str = f"{a2}:hi"
        p = ParamStr.and_parse(in_str, defaults={a1: 'val', a2: None, a3: ['v1', 'v2']})
        assert p.keys() == [a2, a1, a3]

        in_str = f"{a3}:hi"
        p = ParamStr.and_parse(in_str, defaults={a1: 'val', a2: None, a3: ['v1', 'v2']})
        assert p.keys() == [a3, a1]

    def test_error_on_non_defaults(self, missing, a1, a2):
        defs = {a1: [], a2: []}
        p = ParamStr(allows_leading_str=False, defaults=defs)
        assert p.defaults == defs
        assert p.parsed is None
        assert p.allows_non_defaults is False

        with pytest.raises(RuntimeError, match=f"ParamStr encountered parameter '{missing}', which is not an expected parameter"):
            p.parse(f"{a1}~{missing}")
        self.assert_unparsed(p)

        with pytest.raises(RuntimeError, match=f"ParamStr encountered parameter '{missing}', which is not an expected parameter"):
            p.parse(f"{a1}~{missing}:with:vals")
        self.assert_unparsed(p)

    def test_allowing_non_defaults(self, a1, a2, missing):
        defs = {a1: 'default', a2: []}
        p = ParamStr(allows_leading_str=False, defaults=defs, allows_non_defaults=True)
        assert p.defaults == {a1: ['default'], a2: []}
        assert p.parsed is None
        assert p.allows_non_defaults is True

        p.parse(f"{a2}:val~{missing}")
        assert p.parsed == {a1: ['default'], a2: ['val'], missing: []}
        assert str(p) == f"{a2}:val~{missing}~{a1}:default"

        p = ParamStr(allows_leading_str=False, defaults=defs, allows_non_defaults=True)
        p.parse(f"{a2}:val~{missing}:m:val")
        assert p.parsed == {a1: ['default'], a2: ['val'], missing: ['m', 'val']}
        assert str(p) == f"{a2}:val~{missing}:m:val~{a1}:default"

    def test_try_parse(self, a1, a2, a3):
        p = ParamStr()
        err = p.try_parse("~")
        self.assert_unparsed(p)
        assert isinstance(err, RuntimeError)
        assert str(err) == "ParamStr encountered a parameter with an empty key, which is not allowed"

        p = ParamStr(defaults={a1: None, a2: []})
        err = p.try_parse(f"{a3}:fail")
        self.assert_unparsed(p)
        assert isinstance(err, RuntimeError)
        assert str(err) == self.not_allowed_param_msg(a3)

        p = ParamStr(defaults={a1: None, a2: []})
        parsed = p.try_parse(f"{a1}:should:pass")
        assert isinstance(parsed, ParamStr)
        assert p.parsed == {a1: ['should', 'pass'], a2: []}
        assert str(p) == f"{a1}:should:pass~{a2}"

    def test_clear(self, a1, a2):
        defs = {a1: ['default'], a2: []}
        p = ParamStr(allows_leading_str=True, defaults=defs, allows_non_defaults=True)
        self.assert_unparsed(p)
        assert p.defaults == defs
        assert p.allows_leading_str == True
        assert p.allows_non_defaults == True

        # No change to setup upon clear
        assert isinstance(p.clear(), ParamStr)
        self.assert_unparsed(p)
        assert p.defaults == defs
        assert p.allows_leading_str == True
        assert p.allows_non_defaults == True

        leading = "leading"
        in_str = f"{leading}~{a2}:not:default"
        p.parse(in_str)
        assert p.parsed == {a1: ['default'], a2: ['not', 'default']}
        assert p.raw == in_str
        assert p.leading == leading

        assert isinstance(p.clear(), ParamStr)
        self.assert_unparsed(p)
        assert p.defaults == defs
        assert p.allows_leading_str == True
        assert p.allows_non_defaults == True

    def test_duplicating_unparsed_instance(self, a1, a2):
        defs = {a1: ['default'], a2: []}
        leading = "leading"
        p = ParamStr(allows_leading_str=True, defaults=defs)
        self.assert_unparsed(p)
        assert p.defaults == defs
        assert p.allows_leading_str == True
        assert p.allows_non_defaults == False

        p2 = p.duplicate()
        self.assert_unparsed(p2)
        assert p2.defaults == defs
        assert p2.allows_leading_str == True
        assert p2.allows_non_defaults == False
        in_str_p2 = f"{leading}~{a1}~{a2}:val"
        p2.parse(in_str_p2)
        assert p2.parsed == {a1: [], a2: ['val']}
        assert str(p2) == in_str_p2
        assert p2.raw == in_str_p2
        assert p2.leading == leading

        p3 = p.dup()
        self.assert_unparsed(p3)
        assert p3.defaults == defs
        assert p3.allows_leading_str == True
        assert p3.allows_non_defaults == False

        in_str = f"p3~{a1}:new"
        p3.parse(in_str)
        assert p3.parsed == {a1: ['new'], a2: []}
        assert str(p3) == f"{in_str}~{a2}"
        assert p3.raw == in_str
        assert p3.leading == "p3"

        # p and p2 should remain unchanged
        self.assert_unparsed(p)
        assert p2.parsed == {a1: [], a2: ['val']}
        assert str(p2) == f"{leading}~{a1}~{a2}:val"
        assert p2.raw == in_str_p2
        assert p2.leading == leading

    def test_duplicating_parsed_instance(self, a1, a2, a3):
        defs = {a1: [], a2: []}
        p = ParamStr.and_parse(f"{a3}:v1", defaults=defs, allows_non_defaults=True)
        assert p.parsed == {a1: [], a2: [], a3: ['v1']}

        p2 = p.duplicate()
        assert p2.parsed == {a1: [], a2: [], a3: ['v1']}

        p3 = p2.dup()
        assert p3.parsed == {a1: [], a2: [], a3: ['v1']}

        p.clear()
        self.assert_unparsed(p)
        assert p2.parsed == {a1: [], a2: [], a3: ['v1']}
        assert p3.parsed == {a1: [], a2: [], a3: ['v1']}

        p2.clear()
        self.assert_unparsed(p2)
        assert p3.parsed == {a1: [], a2: [], a3: ['v1']}

    def test_error_on_reparse(self, a1, a2, a3, reparse_error):
        parsed = {a1: [], a2: ['val']}
        leading = 'leading'
        in_str = f"{leading}~{a1}~{a2}:val"
        p = ParamStr(allows_leading_str=True)
        p.parse(in_str)
        assert p.parsed == parsed
        assert p.raw == in_str
        assert p.leading == leading

        with pytest.raises(RuntimeError, match=reparse_error):
            p.parse(f"{a1}~{a3}")
        assert p.parsed == parsed
        assert p.raw == in_str
        assert p.leading == leading

    def test_setting_parameters(self, a1, a2, a3, reparse_error):
        p = ParamStr()
        assert p.set(a1, 'val') == False
        assert p.parsed == {a1: ['val']}
        assert p.raw == None
        assert p.leading == None

        assert p.set(a2, []) == False
        assert p.set(a3, ['v', '3']) == False
        assert p.parsed == {a1: ['val'], a2: [], a3: ['v', '3']}
        assert p.raw == None
        assert p.leading == None

        assert p.set(a2, None) == True
        assert p.parsed == {a1: ['val'], a3: ['v', '3']}

        with pytest.raises(RuntimeError, match=reparse_error):
            p.parse("")

    def test_setting_parameters_after_parse(self, a1, a2, a3):
        p = ParamStr()
        in_str = f"{a1}:v1~{a2}"
        p.parse(in_str)
        assert p.parsed == {a1: ['v1'], a2: []}
        assert p.leading == None
        assert p.raw == in_str

        assert p.set(a3, []) == False
        assert p.parsed == {a1: ['v1'], a2: [], a3: []}
        assert p.raw == in_str

        assert p.set(a2, ['v', '2']) == True
        assert p.set(a1, None) == True
        assert p.parsed == {a2: ['v', '2'], a3: []}
        assert p.raw == in_str

    def test_setting_parameters_with_defaults(self, a1, a2, a3, a4):
        defs = {a1: 'v1', a2: [], a3: ['v', '3']}
        p = ParamStr(defaults=defs)
        assert p.set(a1, 'val') == True
        assert p.parsed == {a1: ['val'], a2: [], a3: ['v', '3']}

        p = ParamStr(defaults=defs)
        with pytest.raises(RuntimeError, match=self.not_allowed_param_msg(a4)):
            p.set(a4, None)
        # This will still initialize the ParamStr
        assert p.parsed == {a1: ['v1'], a2: [], a3: ['v', '3']}

        p = ParamStr(defaults=defs, allows_non_defaults=True)
        assert p.set(a2, None) == True
        assert p.set(a4, []) == False
        assert p.parsed == {a1: ['v1'], a3: ['v', '3'], a4: []}

    @pytest.mark.skip
    def test_setting_with_dict(self):
        fail

    @pytest.mark.skip
    def test_duplicate_param(self):
        fail

    def test_setting_leading_str(self, a1):
        p = ParamStr(allows_leading_str=True)

        lead_str = "test_setting"
        assert p.set_leading(lead_str) == False
        assert p.leading == lead_str
        assert str(p) == lead_str

        lead_str2 = "test setting 2"
        assert p.set_leading(lead_str2) == True
        assert p.leading == lead_str2
        p.set(a1, "val")
        assert str(p) == f"{lead_str2}~{a1}:val"

    def test_setting_leading_str_after_parse(self, a1, a2):
        p = ParamStr(allows_leading_str=True)
        in_str = f"~{a1}~{a2}:v"
        p.parse(in_str)
        assert p.leading == ''
        assert p.raw == in_str

        leading = "leading"
        assert p.set_leading(leading) == True
        assert p.leading == leading
        assert p.raw == in_str
        assert str(p) == f"{leading}{in_str}"

    def test_error_setting_leading_str_if_not_allowed(self, leading_str_not_allowed_msg):
        p = ParamStr()
        with pytest.raises(RuntimeError, match=leading_str_not_allowed_msg):
            p.set_leading("leading")

        p.parse("")
        assert p.parsed == {}
        assert p.leading == None
        with pytest.raises(RuntimeError, match=leading_str_not_allowed_msg):
            p.set_leading("leading")

    def test_setting_allows_leading_str(self, a1, leading, leading_str_not_allowed_msg):
        p = ParamStr()
        assert p.allows_leading_str == False

        assert p.set_allows_leading_str(True) == None
        assert p.allows_leading_str == True
        self.assert_unparsed(p)
        p.set_leading(leading) == True

        p.clear()
        assert p.allows_leading_str == True
        p.parse(f"{leading}~{a1}")
        assert p.leading == leading
        assert p.parsed == {a1: []}

        p.clear()
        assert p.set_allows_leading_str(False) == None
        assert p.allows_leading_str == False
        with pytest.raises(RuntimeError, match=leading_str_not_allowed_msg):
            p.set_leading("leading")

    def test_error_setting_allows_leading_str_after_parse(self, leading_str_not_allowed_msg, set_leading_str_after_parse_error_msg):
        p = ParamStr()
        assert p.allows_leading_str == False
        p.parse("")

        with pytest.raises(RuntimeError, match=set_leading_str_after_parse_error_msg):
            p.set_allows_leading_str(True)

        # Allows leading should still be False
        assert p.allows_leading_str == False
        with pytest.raises(RuntimeError, match=leading_str_not_allowed_msg):
            p.set_leading("leading")

        with pytest.raises(RuntimeError, match=set_leading_str_after_parse_error_msg):
            p.set_allows_leading_str(False)
        assert p.allows_leading_str == False

    def test_setting_defaults(self, a1, a2, a3, a4, a5, a6):
        p = ParamStr()
        assert p.defaults == None
        assert p.allows_non_defaults == None

        assert p.add_default(a1, None) == False
        assert p.add_default(a2, 'v1') == False
        assert p.add_default(a3, ['v', '2']) == False
        assert p.allows_non_defaults == False
        assert p.defaults == {a1: None, a2: ['v1'], a3: ['v', '2']}
        p.parse("")
        assert p.parsed == {a2: ['v1'], a3: ['v', '2']}

        defs = {a1: ['v1'], a2: None}
        p = ParamStr(defaults=defs)
        assert p.defaults == defs
        assert p.add_default(a3, None) == False
        assert p.add_default(a1, None) == True
        assert p.defaults == {a1: None, a2: None, a3: None}

        assert p.add_defaults({a4: None, a5: ['v' , '5']}) == [False, False]
        assert p.defaults == {a1: None, a2: None, a3: None, a4: None, a5: ['v', '5']}

        assert p.add_defaults({a2: None, a3: ['v3'], a6: 'v6'}) == [True, True, False]
        assert p.defaults == {a1: None, a2: None, a3: ['v3'], a4: None, a5: ['v', '5'], a6: ['v6']}

    def test_removing_defaults(self, a1, a2, a3, a4, a5):
        defs = {a1: None, a2: 'val', a3: ['v', '3'], a4: [], a5: 'v5'}
        defs_added = {a1: None, a2: ['val'], a3: ['v', '3'], a4: [], a5: ['v5']}
        p = ParamStr(defaults=defs)
        assert p.defaults == defs_added

        assert p.remove_default(a1) == None
        defs_added.pop(a1)
        assert p.defaults == defs_added

        assert p.remove_default(a2) == ['val']
        assert p.remove_default(a3) == ['v', '3']
        assert p.remove_default(a4) == []
        defs_added.pop(a2)
        defs_added.pop(a3)
        defs_added.pop(a4)
        assert p.defaults == defs_added
        p.parse("")
        assert str(p) == f"{a5}:v5"

        p = ParamStr(defaults=defs)
        assert p.remove_defaults([a1, a3, a4, a5]) == [None, ['v', '3'], [], ['v5']]
        assert p.defaults == {a2: ['val']}

        with pytest.raises(RuntimeError, match=self.not_allowed_param_msg(a1)):
            p.parse(f'{a1}:try')

        p.parse(f"{a2}:override")
        assert str(p) == f"{a2}:override"

    def test_error_removing_missing_default(self, a1, a2, a3):
        defs = {a1: ['v1'], a2: []}
        p = ParamStr(defaults=defs)
        with pytest.raises(RuntimeError, match=self.removing_missing_def_err_msg(a3)):
            p.remove_default(a3)
        with pytest.raises(RuntimeError, match=self.removing_missing_def_err_msg(a3)):
            p.remove_defaults([a1, a3])
        assert p.defaults == defs

    def test_error_changing_defaults_after_parse(self, a1, a2, a3, a4, defs_update_after_parse_err_msg):
        defs = {a1: [], a2: None}
        p = ParamStr(defaults=defs)
        p.parse("")
        with pytest.raises(RuntimeError, match=defs_update_after_parse_err_msg):
            p.add_default(a3, [])
        assert p.defaults == defs

        with pytest.raises(RuntimeError, match=defs_update_after_parse_err_msg):
            p.add_defaults({a3: 'val', a4: 'val'})
        assert p.defaults == defs

        with pytest.raises(RuntimeError, match=defs_update_after_parse_err_msg):
            p.add_defaults({a1: 'v', a3: 'v3'})
        assert p.defaults == defs

        with pytest.raises(RuntimeError, match=defs_update_after_parse_err_msg):
            p.remove_default(a3)
        assert p.defaults == defs

        with pytest.raises(RuntimeError, match=defs_update_after_parse_err_msg):
            p.remove_defaults([a3, a4])
        assert p.defaults == defs

        with pytest.raises(RuntimeError, match=defs_update_after_parse_err_msg):
            p.remove_defaults([a1, a4])
        assert p.defaults == defs

    def test_setting_allows_non_defaults(self, a1, a2, a3, update_allows_non_defaults_after_parse_err_msg):
        p = ParamStr(defaults={a1: [], a2: None})
        assert p.allows_non_defaults == False
        assert p.set_allows_non_defaults(True) == None
        assert p.allows_non_defaults == True
        p.parse(a3)
        assert p.parsed == {a1: [], a3: []}

        with pytest.raises(RuntimeError, match=update_allows_non_defaults_after_parse_err_msg):
            p.set_allows_non_defaults(False)
        assert p.allows_non_defaults == True

        p = ParamStr(defaults={a1: [], a2: None}, allows_non_defaults=True)
        assert p.allows_non_defaults == True
        assert p.set_allows_non_defaults(False) == None
        with pytest.raises(RuntimeError, match=self.not_allowed_param_msg(a3)):
            p.parse(a3)

class TestMultiParamStr(Common):
    @classmethod
    def assert_pre_parsed(cls, mps):
        assert mps.__class__.__name__ == "MultiParamStr"
        assert mps.param_strs == []
        assert mps.leading is None
        assert mps.parsed is None
        assert mps.raw is None

    def test_pre_parsed_no_leading(self):
        mps = MultiParamStr()
        assert mps.allows_leading_str is False
        self.assert_pre_parsed(mps)

    def test_pre_parsed_with_leading(self):
        mps = MultiParamStr(allow_leading_str=True)
        assert mps.allows_leading_str is True
        self.assert_pre_parsed(mps)

    def test_empty_str(self):
        mps = MultiParamStr()
        mps.parse("")
        assert mps.leading is None
        assert mps.parsed == []
        assert mps.raw == ""

    def test_single_param_str(self):
        input_str = "arg1:v1:v2~arg2:x1"
        mps = MultiParamStr()
        mps.parse(input_str)

        assert mps.leading is None
        assert mps.parsed == [ParamStr.and_parse(input_str)]
        assert mps.raw == input_str

    def test_multi_param_str(self):
        in_str1 = "arg1~arg2:v1"
        in_str2 = "arg3:w1:w2~arg4:x1~arg5"
        input_str = f"{in_str1}~:~{in_str2}"

        mps = MultiParamStr()
        mps.parse(input_str)

        assert mps.leading is None
        assert mps.parsed == [ParamStr.and_parse(in_str1), ParamStr.and_parse(in_str2)]
        assert mps.raw == input_str

    def test_first_param_str_is_empty(self):
        mps = MultiParamStr()
        in_str1 = "arg1~arg2:v1"
        in_str2 = "arg3:w1:w2~arg4:x1~arg5"
        input_str = f"~:~{in_str1}~:~{in_str2}"

        mps = MultiParamStr()
        mps.parse(input_str)

        assert mps.leading is None
        assert mps.parsed == [ParamStr.and_parse(""), ParamStr.and_parse(in_str1), ParamStr.and_parse(in_str2)]
        assert mps.raw == input_str

    def test_trailing_empty_param_str(self):
        mps = MultiParamStr()
        in_str1 = "arg1~arg2:v1"
        input_str = f"{in_str1}~:~"

        mps = MultiParamStr()
        mps.parse(input_str)

        assert mps.leading is None
        assert mps.parsed == [ParamStr.and_parse(in_str1), ParamStr.and_parse("")]
        assert mps.raw == input_str

    def test_leading_str_only(self):
        input_str = "leading only"
        mps = MultiParamStr(allow_leading_str=True)
        assert mps.allows_leading_str == True
        mps.parse(input_str)
        assert mps.leading == input_str
        assert mps.parsed == []
        assert mps.raw == input_str

        input_str = "~:~leading only"
        mps = MultiParamStr(allow_leading_str=True)
        mps.parse(input_str)
        assert mps.leading == ""
        assert mps.parsed == [ParamStr.and_parse("leading only")]
        assert mps.raw == input_str

        input_str = "leading only~:~"
        mps = MultiParamStr(allow_leading_str=True)
        mps.parse(input_str)
        assert mps.leading == "leading only"
        assert mps.parsed == []
        assert mps.raw == input_str

        input_str = "leading only~"
        mps = MultiParamStr(allow_leading_str=True)
        mps.parse(input_str)
        assert mps.leading == input_str
        assert mps.parsed == []
        assert mps.raw == input_str

    def test_leading_str_with_params(self, a1, a2, a3):
        p1 = f"{a1}:v1:v2~{a2}:w1~{a3}"
        input_str = f"leading~:~{p1}"
        mps = MultiParamStr(allow_leading_str=True)
        mps.parse(input_str)
        assert mps.leading == "leading"
        assert mps.parsed == [ParamStr.and_parse(p1)]
        assert mps.raw == input_str

    def test_leading_str_with_multi_params(self, a1, a2, a3, a4):
        p1 = f"{a1}:v1:v2"
        p2 = f"{a2}"
        p3 = f"{a3}:w1~{a4}"
        input_str = f"leading~:~{p1}~:~{p2}~:~{p3}"
        mps = MultiParamStr(allow_leading_str=True)
        mps.parse(input_str)
        assert mps.leading == "leading"
        assert mps.parsed == [ParamStr.and_parse(p1), ParamStr.and_parse(p2), ParamStr.and_parse(p3)]
        assert mps.raw == input_str
