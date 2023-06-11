from .command import CmdOpt

def to_std_opt(opt):
    if opt == "m":
        return CmdOpt(
                "mode",
                help="Override the default mode currently set by the workspace for this command",
                takes_value=True,
                multi=False,
                ln="mode",
            )
    elif opt == "nt":
        return CmdOpt(
            "no_targets",
            help="Clear any targets currently set by the workspace for this command",
            takes_value=False,
            ln_aliases=["no_target"],
        )
    elif opt == "o":
        return CmdOpt(
            "output_dir",
            help="Override the default output directory (<APP ROOT>/output)",
            sn="o",
            ln="output-dir",
            takes_value=True,
            ln_aliases=["output_dir"],
        )
    elif opt == "r":
        return CmdOpt(
            "reference_dir",
            help="Override the default reference directory (<APP ROOT>/.ref)",
            sn="r",
            ln="ref-dir",
            takes_value=True,
            ln_aliases=["reference_dir", "ref_dir", "reference-dir"],
            value_name="REFERENCE_DIR",
        )
    elif opt == "t":
        return CmdOpt(
            "targets",
            help="Override the targets currently set by the workspace for this command",
            takes_value=True,
            multi=True,
            use_delimiter=True,
            ln="targets",
            ln_aliases=["target"],
            sn="t",
        )
    else:
        raise RuntimeError(f"Unknown std opt '{opt}'")
