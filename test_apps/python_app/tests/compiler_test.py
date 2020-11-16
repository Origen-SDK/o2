import origen  # pylint: disable=import-error
import pytest, pathlib, os, stat, abc
from os import access, W_OK, X_OK, R_OK
from tests.shared import clean_falcon, clean_compiler, tmp_dir


def user_compiler():
    ''' End users should access the compiler via ``origen.app.compiler``. '''
    return origen.app.compiler


MakoRenderer = origen.compiler.MakoRenderer
# JinjaRenderer = origen.compiler.JinjaRenderer


def test_compiler_inits(clean_falcon):
    assert isinstance(user_compiler(), origen.compiler.Compiler) == True
    assert user_compiler().stack == []
    assert user_compiler().renders == []
    assert user_compiler().output_files == []
    assert 'mako' in user_compiler().renderers
    assert user_compiler().renderers['mako'] is MakoRenderer


def test_copmiler_selects_appropriate_syntax(clean_falcon):
    test = "myfile.txt.mako"
    assert user_compiler().select_syntax(test) == 'mako'
    assert user_compiler().select_syntax(pathlib.Path(test)) == 'mako'

    test = "myfile.txt.jinja"
    assert user_compiler().select_syntax(test) == 'jinja'
    assert user_compiler().select_syntax(pathlib.Path(test)) == 'jinja'

    test = "myfile.txt"
    assert user_compiler().select_syntax(test) is None
    assert user_compiler().select_syntax(pathlib.Path(test)) is None


def test_compiler_text_render_requires_syntax(clean_falcon):
    with pytest.raises(origen.compiler.ExplicitSyntaxRequiredError):
        user_compiler().render("Test...", direct_src=True)


class FixtureCompilerTest(abc.ABC):
    ''' Fixture conformance testing the child renderer
    '''
    @property
    @abc.abstractclassmethod
    def extension(cls):
        raise NotImplementedError

    @property
    @abc.abstractclassmethod
    def syntax(cls):
        raise NotImplementedError

    @property
    def str_render(self):
        return "Hello " + self.templatify('"Origen"') + "!"

    @property
    def str_render_with_standard_context(self):
        return f"Hello from Origen version {self.templatify('origen.version')}!"

    @property
    def str_render_with_additional_context(self):
        return f"Hello from template compiler \"{self.templatify('test_renderer_name')}\"!"

    @property
    def expected_str_render(self):
        return "Hello Origen!"

    @property
    def expected_str_render_with_standard_context(self):
        # Make sure origen.version isn't woefully broken
        assert isinstance(origen.version, str)
        assert len(origen.version) > 0
        return f"Hello from Origen version {origen.version}!"

    @property
    def expected_str_render_with_additional_context(self):
        return f"Hello from template compiler \"{self.syntax}\"!"

    @property
    def dummy_input_filename(self):
        return pathlib.Path(
            str(self.expected_output_filename) + f'.{self.extension}')

    @property
    def expected_output_filename(self):
        return tmp_dir().joinpath(f'test_file.txt')

    @property
    def expected_default_output_filename(self):
        s = user_compiler().renderers[self.syntax]
        return origen.app.output_dir.joinpath(f'compiled/test_file.txt')

    @property
    def input_filename(self):
        return origen.root.joinpath('templates/dut_info.txt' +
                                    f'.{self.extension}')

    @property
    def output_filename(self):
        return tmp_dir().joinpath('dut_info.txt')

    @property
    def expected_dut_info_output(self):
        return "\n".join([
            self.expected_str_render_with_standard_context,
            self.expected_str_render_with_additional_context,
            'The application name is "example"'
        ])

    def test_compiler_resolves_default_filenames(self):
        # Test as string
        f = str(self.dummy_input_filename)
        r = user_compiler().resolve_filename(f)
        assert r == self.expected_default_output_filename

        # Test as pathlib.Path
        assert user_compiler().resolve_filename(
            self.dummy_input_filename) == self.expected_default_output_filename

    def test_compiler_resolves_filenames(self):
        # Test as string
        assert user_compiler().resolve_filename(
            str(self.dummy_input_filename),
            output_dir=tmp_dir()) == self.expected_output_filename

        # Test as pathlib.Path
        assert user_compiler().resolve_filename(
            self.dummy_input_filename,
            output_dir=tmp_dir()) == self.expected_output_filename

    @property
    def additional_context(self):
        return {'test_renderer_name': self.syntax}

    def test_render_file(self):
        ''' Test that the renderer can render a given file '''
        rendered = user_compiler().render(self.input_filename,
                                          syntax=self.syntax,
                                          direct_src=False,
                                          output_dir=tmp_dir(),
                                          context=self.additional_context)
        assert isinstance(rendered, pathlib.Path)
        assert rendered == self.output_filename
        assert rendered.exists
        assert open(rendered, 'r').read() == self.expected_dut_info_output

    def test_render_str(self):
        ''' Test that the renderer can render a given string '''
        rendered = user_compiler().render(self.str_render,
                                          syntax=self.syntax,
                                          direct_src=True)
        assert rendered == self.expected_str_render

    def test_render_with_standard_context(self):
        ''' Renders output using the standard context '''
        rendered = user_compiler().render(
            self.str_render_with_standard_context,
            syntax=self.syntax,
            direct_src=True)
        assert rendered == self.expected_str_render_with_standard_context

    def test_render_with_additional_context(self):
        ''' Renders output using additional context given as an option
            -> Test that the renderer supports the 'additional_context' option
        '''
        rendered = user_compiler().render(
            self.str_render_with_additional_context,
            syntax=self.syntax,
            direct_src=True,
            context={'test_renderer_name': self.syntax})
        assert rendered == self.expected_str_render_with_additional_context

    @abc.abstractclassmethod
    def templatify(self, input):
        raise NotImplementedError


class TestMakoCompiler(FixtureCompilerTest):
    extension = 'mako'
    syntax = 'mako'

    def templatify(self, input):
        return "${" + input + "}"


# class TestJinjaCompiler:
#     pass


class TestCompilerStack():
    ''' Tests the compiler's stack-like interface '''

    test_cases = TestMakoCompiler()
    ''' Borrow the Mako test cases for use here '''
    def test_compiler_can_accept_requests(self, clean_falcon, clean_compiler):
        ''' Push can accept either a straight pathlib.Path or str object (interpreted as a file)
            or a tuple consisting of a 'src' and 'options'
        '''
        assert len(user_compiler().stack) == 0
        user_compiler().push('test.mako')
        assert len(user_compiler().stack) == 1
        assert isinstance(user_compiler().stack[0], tuple)
        assert isinstance(user_compiler().stack[0][0], list)
        assert isinstance(user_compiler().stack[0][0][0], pathlib.Path)
        assert user_compiler().stack[0][1] == {}

    def test_compiler_can_clear_itself(self):
        assert len(user_compiler().stack) > 0
        user_compiler().clear()
        assert user_compiler().stack == []
        assert user_compiler().renders == []
        assert user_compiler().output_files == []

    def test_compiler_renders_text(self, clean_falcon, clean_compiler):
        origen.app.compile(self.test_cases.str_render,
                           direct_src=True,
                           syntax='mako')
        assert len(user_compiler().renders) == 1
        assert len(user_compiler().stack) == 0
        assert user_compiler(
        ).renders[0] == self.test_cases.expected_str_render

        origen.app.compile(self.test_cases.str_render_with_additional_context,
                           context=self.test_cases.additional_context,
                           direct_src=True,
                           syntax='mako')
        assert len(user_compiler().renders) == 2
        assert len(user_compiler().stack) == 0
        assert user_compiler().renders[
            1] == self.test_cases.expected_str_render_with_additional_context
        assert user_compiler().renders[-1] == user_compiler().last_render

    def test_compiler_text_render_requires_syntax(self, clean_falcon,
                                                  clean_compiler):
        assert len(user_compiler().stack) == 0
        with pytest.raises(origen.compiler.ExplicitSyntaxRequiredError):
            origen.app.compile(self.test_cases.str_render, direct_src=True)

    def test_compiler_returns_templates_dir(self):
        assert user_compiler().templates_dir == origen.app.root.joinpath(
            'templates')

    def test_compiler_renders_files(self, clean_falcon, clean_compiler):
        origen.app.compile('dut_info.txt.mako',
                           output_dir=tmp_dir(),
                           context=self.test_cases.additional_context,
                           templates_dir=user_compiler().templates_dir)
        assert len(user_compiler().stack) == 0
        assert len(user_compiler().output_files) == 1
        compiled_file = user_compiler().output_files[0]
        compiled_file_status = os.stat(compiled_file)
        assert isinstance(compiled_file, pathlib.PurePath) == True
        assert compiled_file.exists() == True
        assert access(compiled_file, R_OK) == True
        # Check file permissions
        assert bool(compiled_file_status.st_mode & stat.S_IRUSR) == True
        assert bool(compiled_file_status.st_mode & stat.S_IWUSR) == True
        assert bool(compiled_file_status.st_mode & stat.S_IWUSR) == True
