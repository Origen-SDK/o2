class HelpMsg:
    not_extendable_msg = "This command does not support extensions."

    def __init__(self, help_str):
        self.text = help_str
        sections = [section.strip("\n") for section in help_str.split("\n\n") if section.strip()]
        if "Origen, The Semiconductor Developer's Kit" in sections[0]:
            self.version_str = sections.pop(1).strip()
            self.root_cmd = True
        else:
            self.version_str = None
            self.root_cmd = False

        # Clap 3's custom template began with a command-name/help block, while
        # Clap 4 renders the command description as its own block immediately
        # before Usage (and omits that block when no description is set).
        if sections[0].startswith("Usage:"):
            self.help = None
            usage_index = 0
        elif len(sections) > 1 and sections[1].startswith("Usage:"):
            self.help = sections[0]
            usage_index = 1
        else:
            header = sections[0].split("\n")
            self.cmd = header[0]
            self.help = "\n".join(header[1:]) if len(header) > 1 else None
            usage_index = 1

        usage = sections[usage_index].split("\n")
        if usage[0] == "USAGE:":
            self.usage = usage[1]
        elif usage[0].startswith("Usage:"):
            self.usage = usage[0].split("Usage:", 1)[1].strip()
        else:
            raise AssertionError(f"Unrecognized usage section: {usage[0]}")
        if not hasattr(self, "cmd"):
            self.cmd = self.usage.split()[0]
        self.after_help_msg = None

        sects = {}
        for sect in sections[usage_index + 1:]:
            subsects = sect.split("\n")
            for (i, s) in enumerate(subsects):
                if s in ["ARGS:", "Arguments:"]:
                    current = "args"
                    sects[current] = []
                elif s in ["OPTIONS:", "Options:"]:
                    current = "opts"
                    sects[current] = []
                elif s in ["SUBCOMMANDS:", "Commands:"]:
                    current = "subcmds"
                    sects[current] = []
                elif s == "APP COMMAND SHORTCUTS:":
                    current = "app_cmd_shortcuts"
                    sects[current] = []
                elif s == "PLUGIN COMMAND SHORTCUTS:":
                    current = "pl_cmd_shortcuts"
                    sects[current] = []
                elif s == "AUX COMMAND SHORTCUTS:":
                    current = "aux_cmd_shortcuts"
                    sects[current] = []
                elif s == "This command is extended from:":
                    current = "extensions"
                    sects["extensions"] = []
                else:
                    if (i == len(subsects) - 1) and s == "":
                        next
                    elif current == "after_help_msg":
                        self.after_help_msg.append(s)
                    elif not s.startswith(" "):
                        if current not in ["app_cmd_shortcuts", "pl_cmd_shortcuts", "aux_cmd_shortcuts"]:
                            current = "after_help_msg"
                            self.after_help_msg = [s]
                    else:
                        sects[current].append(s)

        self.args = []
        if "args" in sects:
            arg = None
            for line in sects["args"]:
                stripped = line.strip()
                if stripped.startswith(('<', '[')):
                    arg = {}
                    closing = '>' if stripped.startswith('<') else ']'
                    s = stripped.split(closing, 1)
                    n = s[0][1:]
                    arg["value_name"] = n
                    if s[1].startswith("..."):
                        arg['multiple_values'] = True
                        s[1] = s[1][3:]
                    else:
                        arg['multiple_values'] = False
                    s[1] = s[1].strip()
                    arg['help'] = s[1] if len(s[1]) > 0 else None
                    self.args.append(arg)
                elif arg is not None:
                    if arg['help'] is None:
                        arg['help'] = stripped
                    else:
                        arg['help'] += f" {stripped}"

        self.opts = []
        n = None
        if "opts" in sects:
            for line in sects["opts"]:
                l = line.strip()
                if l[0] == "-":
                    opt = {}
                    opt['long_aliases'] = None
                    opt['short_aliases'] = None
                    opt['extended_from'] = None
                    opt['ext_type'] = None

                    s = l.split("    ", 1)
                    if len(s) > 1:
                        opt["help"] = s[1].strip()
                    else:
                        opt["help"] = None

                    s = s[0].split(" <")
                    if len(s) > 1:
                        opt["value_name"] = s[1].split(">")[0]
                        opt["multiple_values"] = True if ">..." in s[1] else False
                    else:
                        opt["value_name"] = None
                        opt["multiple_values"] = False

                    if s[0].endswith("..."):
                        s[0] = s[0][:-3]

                    if l[1] == "-":
                        # Long name only
                        opt["short_name"] = None
                        opt["long_name"] = s[0][2:]
                    else:
                        names = s[0].split(", ")
                        if len(names) > 1:
                            # long name and short name
                            opt["short_name"] = names[0][1:]
                            opt["long_name"] = names[1][2:]
                        else:
                            # short name only
                            opt["short_name"] = names[0][1:]
                            opt["long_name"] = None
                    self.opts.append(opt)
                else:
                    opt = self.opts[-1]
                    if "help" in opt:
                        if opt["help"] is None:
                            opt["help"] = l
                        else:
                            opt["help"] += f" {l}"
                    else:
                        opt["help"] = l.strip()

                opt = self.opts[-1]
                if ('help' in opt) and (opt['help'] is not None):
                    import re
                    app_ext_substring = "[Extended from the app]"

                    if re.search(r"\[Extended from aux namespace: .*\]", opt['help']):
                        split = opt['help'].split("[Extended from aux namespace: '", 1)
                        if len(split) == 2:
                            if ']' in split[1]:
                                inner_split = split[1].split("']", 1)
                                opt['extended_from'] = inner_split[0]
                                opt['help'] = (split[0] + inner_split[1]).strip()
                                from .command import SrcTypes
                                opt['ext_type'] = SrcTypes.AUX
                    elif re.search(r"\[Extended from plugin: .*\]", opt['help']):
                        split = opt['help'].split("[Extended from plugin: '", 1)
                        if len(split) == 2:
                            if ']' in split[1]:
                                inner_split = split[1].split("']", 1)
                                opt['extended_from'] = inner_split[0]
                                opt['help'] = (split[0] + inner_split[1]).strip()
                                from .command import SrcTypes
                                opt['ext_type'] = SrcTypes.PLUGIN
                    elif app_ext_substring in opt["help"]:
                        opt["help"] = opt["help"].replace(app_ext_substring, '').strip()
                        from .command import SrcTypes
                        opt['extended_from'] = SrcTypes.APP
                        opt['ext_type'] = SrcTypes.APP

                    if re.search(r"\[alias(?:es)?: .*\]", opt['help']):
                        marker = "[aliases: " if "[aliases: " in opt['help'] else "[alias: "
                        split = opt['help'].split(marker, 1)
                        if len(split) == 2:
                            if ']' in split[1]:
                                aliases = [a.strip().lstrip('-') for a in split[1].split(']', 1)[0].split(',')]
                                opt['short_aliases'] = [a for a in aliases if len(a) == 1] or None
                                opt['long_aliases'] = [a for a in aliases if len(a) > 1] or None
                                opt['help'] = split[0] + split[1].split(']', 1)[1]
                                opt['help'] = opt['help'].strip()

                    if re.search(r"\[short aliases: .*\]", opt['help']):
                        split = opt['help'].split("[short aliases: ", 1)
                        if len(split) == 2:
                            if ']' in split[1]:
                                opt['short_aliases'] = [a.strip().lstrip('-') for a in split[1].split(']', 1)[0].split(',')]
                                opt['help'] = split[0].strip()

        self.subcmds = []
        n = None
        if "subcmds" in sects:
            for line in sects["subcmds"]:
                if line.startswith("     "):
                    self.subcmds[-1]["help"] += f" {line.strip()}"
                else:
                    import re
                    s = re.split(r"\s{2,}", line.strip(), maxsplit=1)
                    n = s[0]
                    self.subcmds.append({
                        "name": n,
                        "help": (s[1].strip() if len(s) > 1 else None)
                    })
            for subc in self.subcmds:
                if subc["help"]:
                    if re.search(r"\[alias(?:es)?: .*\]", subc['help']):
                        marker = "[aliases: " if "[aliases: " in subc['help'] else "[alias: "
                        split = subc['help'].split(marker, 1)
                        subc['aliases'] = [a.strip() for a in split[1].split(']', 1)[0].split(',')]
                        subc['help'] = (split[0] + split[1].split(']', 1)[1]).strip()
                        continue
                subc["aliases"] = None

        if "app_cmd_shortcuts" in sects:
            self.app_cmd_shortcuts = {}
            for l in sects["app_cmd_shortcuts"]:
                cmd = l.split("=>")
                for c in cmd[0].strip().split(", "):
                    self.app_cmd_shortcuts[c] = cmd[1].strip()
        else:
            self.app_cmd_shortcuts = None

        if "pl_cmd_shortcuts" in sects:
            self.pl_cmd_shortcuts = {}
            for l in sects["pl_cmd_shortcuts"]:
                cmd = l.split("=>")
                pln, subc = cmd[1].strip().split(" ")
                for c in cmd[0].strip().split(", "):
                    self.pl_cmd_shortcuts[c] = (pln, subc)
        else:
            self.plugin_cmd_shortcuts = None

        if "aux_cmd_shortcuts" in sects:
            self.aux_cmd_shortcuts = {}
            for l in sects["aux_cmd_shortcuts"]:
                cmd = l.split("=>")
                ns, subc = cmd[1].strip().split(" ")
                for c in cmd[0].strip().split(", "):
                    self.aux_cmd_shortcuts[c] = (ns, subc)
        else:
            self.aux_cmd_shortcuts = None
        self.app_exts = False
        self.aux_exts = None
        self.pl_exts = None
        if "extensions" in sects:
            for l in sects["extensions"]:
                if l.strip() == "- the App":
                    self.app_exts = True
                    next

                split = l.split("- Aux Namespaces: ", 1)
                if len(split) == 2:
                    self.aux_exts = [s[1:-1] for s in split[1].split(", ")]
                    next
                
                split = l.split("- Plugins: ", 1)
                if len(split) == 2:
                    self.pl_exts = [s[1:-1] for s in split[1].split(", ")]
                    next

        if self.after_help_msg is not None:
            self.after_help_msg = ("\n").join(self.after_help_msg)

    @property
    def subcmd_names(self):
        if self.subcmds:
            return [subc["name"] for subc in self.subcmds]

    def assert_num_args(self, expected):
        assert len(self.args) == expected
        return True

    def assert_num_opts(self, expected):
        assert len(self.opts) == expected
        return True
    
    def assert_arg_at(self, expected_index, arg):
        a = self.args[expected_index]
        if arg.value_name is not False:
            if arg.value_name is None:
                assert a["value_name"] == arg.name.upper()
            else:
                assert a["value_name"] == arg.value_name
        if arg.multi is not False:
            assert a["multiple_values"] == arg.multi
        if arg.help is not False:
            assert a["help"] == arg.help
        return True

    def assert_args(self, *expected):
        if expected == (None,):
            expected = []
        elif len(expected) == 1 and isinstance(expected[0], dict):
            expected = list(expected[0].values())
        self.assert_num_args(len(expected))
        for i, a in enumerate(expected):
            self.assert_arg_at(i, a)
        return True

    def _assert_opt_params_(self, o, opt):
        if opt.sn is not False:
            assert o["short_name"] == opt.sn
        if opt.ln is not False:
            if opt.sn is not None:
                assert o["long_name"] == opt.ln
            else:
                assert o["long_name"] == opt.to_ln()
        if opt.value_name is not False:
            if opt.takes_value:
                assert o["value_name"] == opt.to_vn()
            # Clap 4 does not annotate an Append option that consumes one
            # value per occurrence with an ellipsis. Repeatability remains a
            # parser behavior and is covered by invocation tests.
            if not opt.multi:
                assert o["multiple_values"] is False
        if opt.help is not False:
            assert o["help"] ==opt.help
        if opt.sn_aliases is not False:
            assert o['short_aliases'] == opt.sn_aliases
        if opt.ln_aliases is not False:
            assert o['long_aliases'] == opt.ln_aliases

    def assert_opt_at(self, expected_index, opt):
        o = self.opts[expected_index]
        self._assert_opt_params_(o, opt)
        assert o["extended_from"] is None
        assert o["ext_type"] is None
        return True

    def assert_bare_opts(self):
        return self.assert_opts("help", "v", 'vk')

    def assert_bare_app_opts(self):
        return self.assert_opts("help", "mode", "no_targets", "targets", "v", 'vk')

    def assert_ext_at(self, expected_index, ext):
        o = self.opts[expected_index]
        self._assert_opt_params_(o, ext)
        if ext.src_name is not False:
            from .command import SrcTypes
            if ext.src_type == SrcTypes.APP:
                assert o["extended_from"] == SrcTypes.APP
            else:
                assert o["extended_from"] == ext.src_name
        if ext.src_type is not False:
            assert o["ext_type"] == ext.src_type
        return True

    def assert_opts(self, *expected_opts):
        assert self.assert_num_opts(len(expected_opts))
        unmatched = list(range(len(self.opts)))
        for o in expected_opts:
            from .command import CmdExtOpt
            keyword = o if isinstance(o, str) else None
            if keyword in ["help", "h"]:
                from .origen import CoreOpts
                expected = CoreOpts.help
            elif keyword == "vk":
                from .origen import CoreOpts
                expected = CoreOpts.vk
            elif keyword in ["mode", "m"]:
                from .origen import InAppOpts
                expected = InAppOpts.mode
            elif keyword in ["no_targets", "nt"]:
                from .origen import InAppOpts
                expected = InAppOpts.no_targets
            elif keyword in ["targets", "t"]:
                from .origen import InAppOpts
                expected = InAppOpts.targets
            elif keyword == "v":
                from .origen import CoreOpts
                expected = CoreOpts.verbosity
            elif keyword is not None:
                raise RuntimeError(f"Unknown keyword opt: {o}")
            else:
                expected = o

            matched = None
            for i in unmatched:
                try:
                    if isinstance(o, CmdExtOpt):
                        self.assert_ext_at(i, o)
                    else:
                        self.assert_opt_at(i, expected)
                    matched = i
                    break
                except AssertionError:
                    continue
            if matched is None:
                raise AssertionError(
                    f"Could not find expected option {expected.name!r} in {self.opts!r}"
                )
            unmatched.remove(matched)
        return True

    def assert_subcmd_at(self, expected_index, subc):
        s = self.subcmds[expected_index]
        assert s['name'] == subc.name
        if subc.help is not False:
            assert s['help'] == subc.help
        if subc.aliases is not False:
            assert s['aliases'] == subc.aliases
        return True

    def assert_subcmds(self, *expected_subcmds, help=None):
        if expected_subcmds == (None,):
            expected_subcmds = []
        elif len(expected_subcmds) == 1 and isinstance(expected_subcmds[0], dict):
            expected_subcmds = expected_subcmds[0].values()
        expected_subcmds = list(expected_subcmds)
        if help is not None:
            expected_subcmds.insert(help, "h")
        assert len(expected_subcmds) == len(self.subcmds)
        unmatched = list(range(len(self.subcmds)))
        for o in expected_subcmds:
            if isinstance(o, tuple):
                expected = o[1]
            elif isinstance(o, str):
                if o not in ["help", "h"]:
                    raise RuntimeError(f"Unknown subcmd keyword: {o}")
                from .origen import help_subcmd
                expected = help_subcmd()
            else:
                expected = o
            try:
                i = next(i for i in unmatched if self.subcmds[i]["name"] == expected.name)
            except StopIteration:
                raise AssertionError(
                    f"Could not find expected subcommand {expected.name!r} in {self.subcmds!r}"
                )
            unmatched.remove(i)
            self.assert_subcmd_at(i, expected)
        return True

    def assert_help_subcmd_at(self, expected_index):
        from .origen import help_subcmd
        return self.assert_subcmd_at(expected_index, help_subcmd())

    def assert_help_opt_at(self, expected_index):
        from .origen import CoreOpts
        return self.assert_opt_at(expected_index, CoreOpts.help)

    def assert_v_opt_at(self, expected_index):
        from .origen import CoreOpts
        return self.assert_opt_at(expected_index, CoreOpts.verbosity)

    def assert_vk_opt_at(self, expected_index):
        from .origen import CoreOpts
        return self.assert_opt_at(expected_index, CoreOpts.vk)

    def assert_mode_opt_at(self, expected_index):
        from .origen import InAppOpts
        return self.assert_opt_at(expected_index, InAppOpts.mode)

    def assert_no_targets_opt_at(self, expected_index):
        from .origen import InAppOpts
        return self.assert_opt_at(expected_index, InAppOpts.no_targets)

    def assert_targets_opt_at(self, expected_index):
        from .origen import InAppOpts
        return self.assert_opt_at(expected_index, InAppOpts.targets)

    def assert_summary(self, msg):
        assert self.help == msg
        return True

    def assert_not_extendable(self):
        assert self.after_help_msg is not None
        assert self.not_extendable_msg == self.after_help_msg.split("\n")[-1]

    def assert_cmd(self, cmd):
        self.assert_args(*(cmd.args.values() or [None]))
        if cmd.opts:
            l = list(cmd.opts.values())
            l.insert(cmd.h_opt_idx, "h")
            l.insert(cmd.v_opt_idx, "v")
            l.insert(cmd.vk_opt_idx, "vk")
            self.assert_opts(*l)
        else:
            self.assert_bare_opts()
        
        if cmd.subcmds:
            subcs = list(cmd.subcmds.values())
            subcs.insert(cmd.help_subc_idx, "help")
            self.assert_subcmds(*subcs)
        else:
            self.assert_subcmds(None)

        if not cmd.extendable:
            self.assert_not_extendable()
        self.assert_summary(cmd.help)

    @property
    def logged_errors(self):
        from . import CLI
        return CLI.extract_logged_errors(self.text)
