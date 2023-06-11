from origen.helpers.regressions.cli import CoreErrorMessages

class ErrorCases(CoreErrorMessages):
    ''' Error cases, messages, and generators too esoteric to be relevant to global origen package'''

    @classmethod
    def to_conflict_msg(cls, cmd, conflict):
        if not isinstance(conflict[0], str):
            cmd = conflict[0]
            conflict = conflict[1:]

        type = conflict[0]
        def tname(t, cap=False):
            if t in ["lna", "repeated_lna"]:
                n = "long name alias"
            elif t == "ln":
                n = "long name"
            elif t == "iln":
                n = "inferred long name"
            elif t in ["sna", "repeated_sna"]:
                n = "short name alias"
            elif t == "sn":
                n = "short name"
            else:
                raise RuntimeError(f"Cannot get conflict name from conflict type {t}")
            if cap:
                n = n.capitalize()
            return n

        prefix = f"When processing command '{cmd.full_name}':"
        if type in ["lna", "ln", "sna", "sn", "iln"]:
            with_type = conflict[1]
            offender_opt = conflict[2]
            with_opt = conflict[3]
            if type == "iln":
                if not isinstance(offender_opt, str):
                    c = offender_opt.name
                else:
                    c = offender_opt
            else:
                c = conflict[4]
            if with_opt is None:
                with_opt = offender_opt

            if (not isinstance(offender_opt, str)) and offender_opt.is_ext:
                if with_opt.is_ext:
                    msg = f"{tname(type, True)} '{c}' for extension option '{offender_opt.name}', from {offender_opt.displayed}, conflicts with {tname(with_type, False)} for extension '{with_opt.name}' provided by {with_opt.displayed}"
                else:
                    msg = f"{tname(type, True)} '{c}' for extension option '{offender_opt.name}', from {offender_opt.displayed}, conflicts with {tname(with_type, False)} from command option '{with_opt.name}'"
            else:
                if not isinstance(offender_opt, str):
                    offender_opt = offender_opt.name
                msg = f"{tname(type, True)} '{c}' for command option '{offender_opt}' conflicts with {tname(with_type, False)} from option '{with_opt.name}'"
        elif type in ["inter_ext_sna_sn", "inter_ext_lna_ln", "inter_ext_lna_iln"]:
            offending_opt = conflict[1]
            if type == "inter_ext_sna_sn":
                type = "sna"
                with_type = "sn"
                name = conflict[2]
            elif type == "inter_ext_lna_ln":
                type = "lna"
                with_type = "ln"
                name = conflict[2]
            elif "inter_ext_lna_iln":
                type = "lna"
                with_type = "iln"
                name = offending_opt.name
            if offending_opt.is_ext:
                msg = f"Option '{offending_opt.name}' extended from {offending_opt.displayed} specifies {tname(type, False)} '{name}' but it conflicts with the option's {tname(with_type, False)}"
            else:
                msg = f"Option '{offending_opt.name}' specifies {tname(type, False)} '{name}' but it conflicts with the option's {tname(with_type, False)}"
        elif type in ["repeated_sna", "repeated_lna"]:
            offending_opt = conflict[1]
            if offending_opt.is_ext:
                offending_src = f"extended from {conflict[1].displayed} "
            else:
                offending_src = ''
            name = conflict[2]
            index = conflict[3]
            msg = f"Option '{offending_opt.name}' {offending_src}repeats {tname(type, False)} '{name}' (first occurrence at index {index})"
        elif type == "reserved_prefix_arg_name":
            offending_arg = conflict[1]
            msg = f"Argument '{offending_arg}' uses reserved prefix 'ext_opt'. This option will not be available"
        elif type == "reserved_prefix_opt_name":
            offending_opt = conflict[1]
            offending_src = conflict[2]
            if offending_src is None:
                msg = f"Option '{offending_opt}' uses reserved prefix 'ext_opt'. This option will not be available"
            else:
                msg = f"Option '{offending_opt}' extended from {offending_src} uses reserved prefix 'ext_opt'. This option will not be available"
        elif type in ["reserved_prefix_ln", "reserved_prefix_lna"]:
            offending_opt = conflict[1]
            name = conflict[2]
            if type == "reserved_prefix_ln":
                type = "ln"
            elif type == "reserved_prefix_lna":
                type = "lna"
            if offending_opt.is_ext:
                msg = f"Option '{offending_opt.name}' extended from {offending_opt.displayed} uses reserved prefix 'ext_opt' in {tname(type, False)} '{name}' and will not be available as '--{name}'"
            else:
                msg = f"Option '{offending_opt.name}' uses reserved prefix 'ext_opt' in {tname(type, False)} '{name}' and will not be available as '--{name}'"
        elif type == "self_lna_iln":
            offending_opt = conflict[1]
            msg = f"Option '{offending_opt.name}' extended from {offending_opt.displayed} specifies long name alias '{offending_opt.name}' but it conflicts with the option's inferred long name. If this is intentional, please set this as the option's long name"
        elif type == "duplicate":
            offending_opt = conflict[1]
            index = conflict[2]
            if offending_opt.is_ext:
                msg = f"Option '{offending_opt.name}' extended from {offending_opt.displayed} is already present. Subsequent occurrences will be skipped (first occurrence at index {index})"
            elif offending_opt.is_arg:
                msg = f"Argument '{offending_opt.name}' is already present. Subsequent occurrences will be skipped (first occurrence at index {index})"
            else:
                msg = f"Option '{offending_opt.name}' is already present. Subsequent occurrences will be skipped (first occurrence at index {index})"
        elif type == "intra_cmd_not_placed":
            msg = f"Unable to place unique long name, short name, or inferred long name for command option '{conflict[1]}'. Please resolve any previous conflicts regarding this option or add/update this option's name, long name, or short name"
        elif type == "arg_opt_name_conflict":
            msg = f"Option '{conflict[1].name}' conflicts with Arg of the same name (Arg #{conflict[2]})"
        else:
            raise RuntimeError(f"Unrecognized conflict type {conflict[0]}")
        msg = f"{prefix} {msg}"
        return msg