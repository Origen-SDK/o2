from origen.helpers.regressions import cli

class Cmd(cli.cmd.Cmd):
    def assert_args(self, output, *vals):
        ext_args = {}
        args = []
        exp_ext_vals = {}
        cmd_args = []
        if vals[0] is None:
            vals = ()

        for v in vals:
            opts = v[1] if isinstance(v[1], dict) else {}
            opts = v[2] if len(v) > 2 else {}

            if isinstance(v[0], CmdExtOpt):
                ext = ext_args.setdefault(v[0].src_name, {})
                if opts.get("Before", True):
                    before = ext.setdefault("Before Cmd", [])
                    if not (("Before" in opts and opts["Before"] is None) or v[1] is None):
                        before.append(v[0].name)
                if opts.get("After", True):
                    after = ext.setdefault("After Cmd", [])
                    if not (("After" in opts and opts["After"] is None) or v[1] is None):
                        after.append(v[0].name)
                if opts.get("CleanUp", True):
                    clean_up = ext.setdefault("CleanUp Cmd", [])
                    if not (("CleanUp" in opts and opts["CleanUp"] is None) or v[1] is None):
                        clean_up.append(v[0].name)
            else:
                # TODO support args/opts (not extension) with options
                if v[1] is not None:
                    args.append(v[0])
            expected = v[0].to_assert_str(v[1], **opts)
            if isinstance(expected, str):
                expected = [expected]

            if isinstance(v[0], CmdExtOpt):
                vals = exp_ext_vals.setdefault(v[0].src_name, [(v[0], None)])
                if not (v[1] is None and ("Before" not in opts and "After" not in opts and "CleanUp" not in opts)):
                    if vals[0][1] is None:
                        exp_ext_vals[v[0].src_name] = []
                        vals = exp_ext_vals[v[0].src_name]
                    vals.append((v[0], expected))
            else:
                if v[1] is not None:
                    cmd_args.append(expected)
        if len(cmd_args) == 0:
            e = "All Keys: (CMD): []"
            print(f"expecting: {e}")
            assert e in output

            e = "Arg: (CMD): No args or opts given!"
            print(f"expecting: {e}")
            assert e in output
        else:
            for exp in cmd_args:
                for e in exp:
                    print(f"expecting: {e}")
                    assert e in output
        for ns, opt in exp_ext_vals.items():
            if len(opt) == 1 and opt[0][1] is None:
                for e in opt[0][0].to_assert_str(None):
                    print(f"expecting: {e}")
                    assert e in output
            else:
                for exp in opt:
                    for e in exp[1]:
                        print(f"expecting: {e}")
                        assert e in output

        actual = self.parse_arg_keys(output)
        assert len(actual) == len(args)
        actual = self.parse_ext_keys(output)
        print(actual)
        print(ext_args)
        assert actual == ext_args

    @classmethod
    def parse_arg_keys(cls, cmd_output):
        return eval(cmd_output.split("All Keys: (CMD):", 1)[1].split("\n")[0])

    @classmethod
    def parse_ext_keys(cls, cmd_output):
        arg_lines = cmd_output.split("All Keys: (Ext) ")
        retn = {}
        for a in arg_lines[1:]:
            a = a.split("\n")[0]
            n, keys = a.split(":", 1)
            n, phase = n.split(") (")
            retn.setdefault(n[1:], {})[phase[0:-1]] = eval(keys)
        return retn

class CmdArgOpt(cli.cmd.CmdArgOpt):
    def to_assert_str(self, vals, **opts):
        if vals is None:
            return f"Arg: (CMD): {self.name}: No args or opts given!"
        elif self.multi:
            c = list
            if self.use_delimiter:
                vals = [x for v in vals for x in v.split(',')]
        elif isinstance(vals, int):
            c = int
        else:
            c = str
        return f"Arg: (CMD): {self.name} ({c}): {vals}"
    
    def assert_present(self, vals, in_str, **opts):
        for e in self.to_assert_str(vals, **opts):
            assert e in in_str

class CmdArg(cli.cmd.CmdArg, CmdArgOpt):
    pass

class CmdOpt(cli.cmd.CmdOpt, CmdArgOpt):
    pass

class CmdExtOpt(cli.cmd.CmdExtOpt, CmdArgOpt):
    def to_assert_str(self, vals, **opts):
        if isinstance(vals, dict):
            opts = vals 
        preface = f"Arg: (Ext) ({self.src_name})"

        retn = []
        before_val = opts["Before"] if "Before" in opts else vals
        after_val = opts["After"] if "After" in opts else vals
        cleanup_val = opts["CleanUp"] if "CleanUp" in opts else vals
        if not before_val is False:
            if before_val is None:
                retn.append(f"{preface} (Before Cmd):{CmdArgOpt.to_assert_str(self, before_val).split(':', 3)[3]}")
            else:
                retn.append(f"{preface} (Before Cmd):{CmdArgOpt.to_assert_str(self, before_val).split(':', 2)[2]}")
        if not after_val is False:
            if after_val is None:
                retn.append(f"{preface} (After Cmd):{CmdArgOpt.to_assert_str(self, after_val).split(':', 3)[3]}")
            else:
                retn.append(f"{preface} (After Cmd):{CmdArgOpt.to_assert_str(self, after_val).split(':', 2)[2]}")
        if not cleanup_val is False:
            if cleanup_val is None:
                retn.append(f"{preface} (CleanUp Cmd):{CmdArgOpt.to_assert_str(self, cleanup_val).split(':', 3)[3]}")
            else:
                retn.append(f"{preface} (CleanUp Cmd):{CmdArgOpt.to_assert_str(self, cleanup_val).split(':', 2)[2]}")
        return retn