from origen_metal.utils.differ import has_diffs
import tempfile
import os


def test_basic_operation():
    fa, file_a = tempfile.mkstemp()
    fb, file_b = tempfile.mkstemp()
    os.write(fa, b"This part is the same // But this is different\n")
    os.write(fb, b"This part is the same // But this is not\n")
    os.write(fb, b"\n")  # An extra blank line in file B
    os.close(fa)
    os.close(fb)

    assert (has_diffs(file_a, file_b))
    assert (not has_diffs(file_a, file_b, ignore_comments="//"))
    assert (has_diffs(file_a,
                      file_b,
                      ignore_comments="//",
                      ignore_blank_lines=False))


def test_basic_suspend_works():
    fa, file_a = tempfile.mkstemp()
    fb, file_b = tempfile.mkstemp()
    os.write(
        fa, b"""This part is the same
         until here<START> dfodisgsg
         soemghow dosghsg
         iewg<STOP>
         and now we are back""")
    os.write(
        fb, b"""This part is the same
         until here<START>
         some diff
         <STOP>
         and now we are back""")
    os.close(fa)
    os.close(fb)

    assert (has_diffs(file_a, file_b))
    assert (not has_diffs(
        file_a, file_b, suspend_on="<START>", resume_on="<STOP>"))


def test_pre_suspend_works():
    fa, file_a = tempfile.mkstemp()
    fb, file_b = tempfile.mkstemp()
    os.write(
        fa, b"""This part is the same
         until here is a pre diff<START> dfodisgsg
         soemghow dosghsg
         iewg<STOP>
         and now we are back""")
    os.write(
        fb, b"""This part is the same
         until here<START>
         some diff
         <STOP>
         and now we are back""")
    os.close(fa)
    os.close(fb)

    assert (has_diffs(file_a, file_b))
    assert (has_diffs(file_a, file_b, suspend_on="<START>",
                      resume_on="<STOP>"))


def test_post_suspend_works():
    fa, file_a = tempfile.mkstemp()
    fb, file_b = tempfile.mkstemp()
    os.write(
        fa, b"""This part is the same
         until here is a pre diff<START> dfodisgsg
         soemghow dosghsg
         iewg<STOP>
         and now we are back""")
    os.write(
        fb, b"""This part is the same
         until here<START>
         some diff
         <STOP> here is a diff!
         and now we are back""")
    os.close(fa)
    os.close(fb)

    assert (has_diffs(file_a, file_b))
    assert (has_diffs(file_a, file_b, suspend_on="<START>",
                      resume_on="<STOP>"))


def test_blank_lines_works():
    fa, file_a = tempfile.mkstemp()
    fb, file_b = tempfile.mkstemp()
    os.write(
        fa, b"""This part is the same
         This part is the same
         This part is the same""")
    os.write(
        fb, b"""This part is the same
         This part is the same

         This part is the same""")

    assert (not has_diffs(file_a, file_b))
    assert (has_diffs(file_a, file_b, ignore_blank_lines=False))


def test_c_style_comments():
    fa, file_a = tempfile.mkstemp()
    fb, file_b = tempfile.mkstemp()
    os.write(
        fa, b"""This part is the same
         about to change /* jdhkjdghsg
         dfdfsfsf ioihjsdgs sdfsdf */ and we're back
         This part is the same
         This part is the same""")
    os.write(
        fb, b"""This part is the same
         about to change /*
         dflslkj ebiuhw sdogih
         dflslkj ebiuhw sdogih */ and we're back
         This part is the same
         This part is the same""")

    assert (has_diffs(file_a, file_b))
    assert (not has_diffs(file_a, file_b, suspend_on="/*", resume_on="*/"))
