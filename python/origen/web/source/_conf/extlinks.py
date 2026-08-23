from origen.web.origen_sphinx_extension.shorthand_defs import defs

pyo3_version = '0.21.2'
python_doc_version = '3'  # Points to latest, stable Python3 version.
links = defs['abslinks']
core_github_branch = "master"
github_root__python_app = f"https://github.com/Origen-SDK/o2/blob/{core_github_branch}/test_apps/python_app"

extlinks = {
    # Sphinx basics or built-in (non-extension) features
    'sphinx_homepage': ('https://www.sphinx-doc.org/en/master/index.html%s', '%s'),
    'sphinx_app':
    ('https://www.sphinx-doc.org/en/master/usage/quickstart.html#getting-started%s', '%s'),
    'sphinx_docs': ('https://www.sphinx-doc.org/en/master/contents.html%s', '%s'),
    'sphinx_extensions':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/index.html%s', '%s'),
    'sphinx_themes':
    ('https://www.sphinx-doc.org/en/master/usage/theming.html#themes%s', '%s'),
    'sphinx_using_a_theme':
    ('https://www.sphinx-doc.org/en/master/usage/theming.html#using-a-theme%s', '%s'),
    'sphinx_builtin_themes':
    ('https://www.sphinx-doc.org/en/master/usage/theming.html#builtin-themes%s', '%s'),
    'sphinx_add_theme':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.add_html_theme%s', '%s'),
    'sphinx_creating_themes':
    ('https://www.sphinx-doc.org/en/master/development/html_themes/index.html%s', '%s'),
    'sphinx_available_themes': ('https://sphinx-themes.org/%s', '%s'),
    'sphinx_project_examples':
    ('https://www.sphinx-doc.org/en/master/examples.html#projects-using-sphinx%s', '%s'),
    'sphinx_conf':
    ('https://www.sphinx-doc.org/en/master/usage/configuration.html#module-conf%s', '%s'),
    'sphinx_add_config_var':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.add_config_value%s', '%s'),
    'sphinx_confval_html_logo':
    ('https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_logo%s', '%s'),
    'sphinx_confval_html_favicon':
    ('https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_favicon%s', '%s'),
    'sphinx_confval_html_theme_options':
    ('https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_theme_options%s', '%s'),
    'sphinx_rst':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/index.html%s', '%s'),
    'sphinx_rst_primer':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html%s', '%s'),
    'sphinx_rst_directives':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html%s', '%s'),
    'sphinx_templating':
    ('https://www.sphinx-doc.org/en/master/development/html_themes/templating.html%s', '%s'),
    'sphinx_manpages':
    ('https://www.sphinx-doc.org/en/master/man/index.html%s', '%s'),
    'sphinx_build_cmd':
    ('https://www.sphinx-doc.org/en/master/man/sphinx-build.html%s', '%s'),
    'sphinx_build_phases':
    ('https://www.sphinx-doc.org/en/master/extdev/index.html#build-phases%s', '%s'),
    'sphinx_connect':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.connect%s', '%s'),
    'sphinx_core_events':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx-core-events%s', '%s'),
    'sphinx_event_config_inited':
    ('https://www.sphinx-doc.org/en/master/extdev/event_callbacks.html#event-config-inited%s', '%s'),
    'sphinx_event_builder_inited':
    ('https://www.sphinx-doc.org/en/master/extdev/event_callbacks.html#event-builder-inited%s', '%s'),
    'sphinx_alabaster_theme': ('https://alabaster.readthedocs.io/en/latest/%s', '%s'),
    'sphinx_nitpicky':
    ('https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-nitpicky%s', '%s'),
    'sphinx_xrefing':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/roles.html#cross-referencing-arbitrary-locations%s', '%s'),
    'sphinx_ref_role':
    ('https://www.sphinx-doc.org/en/master/usage/referencing.html#role-ref%s', '%s'),
    'sphinx_doc_role':
    ('https://www.sphinx-doc.org/en/master/usage/referencing.html#role-doc%s', '%s'),
    'sphinx_python_domain':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/domains.html#the-python-domain%s', '%s'),

    # RST tutorials
    'rst_quickstart':
    ('https://docutils.sourceforge.io/docs/user/rst/quickstart.html%s', '%s'),
    'rst_cheatsheet':
    ('https://docutils.sourceforge.io/docs/user/rst/cheatsheet.txt%s', '%s'),
    'rst_docs': ('https://docutils.sourceforge.io/rst.html%s', '%s'),
    'rst_spec':
    ('https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html%s', '%s'),
    'rst_cokelaer_cheatsheet':
    ('https://thomas-cokelaer.info/tutorials/sphinx/rest_syntax.html#contents-directives%s', '%s'),
    'rst_guide_zephyr':
    ('https://docs.zephyrproject.org/latest/guides/documentation/index.html%s', '%s'),
    'rst_substitutions':
    ('https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#substitution-definitions%s', '%s'),
    'rst_include_directive':
    ('https://docutils.sourceforge.io/docs/ref/rst/directives.html#include%s', '%s'),

    # Jinja tutorials
    'jinja_home': ('https://palletsprojects.com/p/jinja/%s', '%s'),
    'jinja_docs': ('https://jinja.palletsprojects.com/en/master/%s', '%s'),

    # Extension homepages, tutorials, & wrapped libraries
    'markdown_home': ('https://www.markdownguide.org/%s', '%s'),
    'autoapi_home': ('https://autoapi.readthedocs.io/%s', '%s'),
    'autoapi_usage': ('https://autoapi.readthedocs.io/#usage%s', '%s'),
    'autodoc_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/autodoc.html%s', '%s'),
    'bootstrap4':
    ('https://getbootstrap.com/docs/4.5/getting-started/introduction/%s', '%s'),
    'bootstrap4_widgets':
    ('https://getbootstrap.com/docs/4.0/components/alerts/%s', '%s'),
    'bootstrap4_sphinx_theme':
    ('http://myyasuda.github.io/sphinxbootstrap4theme/%s', '%s'),
    'bootstrap4_sphinx_theme_options':
    ('http://myyasuda.github.io/sphinxbootstrap4theme/setup.html#html-theme-options%s', '%s'),
    'bootstrap4_sphinx_theme_templates':
    ('https://github.com/myyasuda/sphinxbootstrap4theme/tree/master/themes/sphinxbootstrap4theme%s', '%s'),
    'autosectionlabel_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/autosectionlabel.html%s', '%s'),
    'autosectionlabel_prefix_document_config':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/autosectionlabel.html#confval-autosectionlabel_prefix_document%s', '%s'),
    'napoleon_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/napoleon.html%s', '%s'),
    'google_docstring_spec':
    ('https://google.github.io/styleguide/pyguide.html%s', '%s'),
    'numpy_docstring_spec':
    ('https://numpydoc.readthedocs.io/en/latest/format.html#docstring-standard%s', '%s'),
    'inheritance_diagram_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/inheritance.html%s', '%s'),
    'inheritance_diagram_example':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/inheritance.html#examples%s', '%s'),
    'graphviz_ext_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/graphviz.html#module-sphinx.ext.graphviz%s', '%s'),
    'graphviz_home': ('https://graphviz.org/%s', '%s'),
    'graphviz_download': ('https://www.graphviz.org/download/%s', '%s'),
    'extlinks_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/extlinks.html%s', '%s'),
    'extlinks_config_var':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/extlinks.html#confval-extlinks%s', '%s'),

    # Other webpage generation related links
    'darkly': ('https://bootswatch.com/darkly/%s', '%s'),
    'dracula_pygments': ('https://draculatheme.com/pygments%s', '%s'),
    'o2_github_root': ('https://github.com/Origen-SDK/o2%s', '%s'),
    'static_website': ('https://en.wikipedia.org/wiki/Static_web_page%s', '%s'),

    # General Python stuff
    'python_docs': ('https://docs.python.org/3/index.html%s', '%s'),
    'python_docs_list':
    (f'https://docs.python.org/{python_doc_version}/library/stdtypes.html#lists%s', '%s'),
    'python_docs_tuple':
    (f'https://docs.python.org/{python_doc_version}/library/stdtypes.html#tuples%s', '%s'),
    'python_docs_dict':
    (f'https://docs.python.org/{python_doc_version}/library/stdtypes.html#mapping-types-dict%s', '%s'),
    'python_exception_hierarchy':
    (f'https://docs.python.org/{python_doc_version}/library/exceptions.html#exception-hierarchy%s', '%s'),
    'python_docs_pathlib': (
        f'https://docs.python.org/{python_doc_version}/library/pathlib.html%s', '%s'),
    'ticket_mako_multiple_newlines':
    ('https://stackoverflow.com/questions/22558067/how-to-convert-multiple-newlines-in-mako-template-to-one-newline%s', '%s'),
    'docstrings_spec': ('https://www.python.org/dev/peps/pep-0257/%s', '%s'),
    'docstrings_intro': (
        'https://www.programiz.com/python-programming/docstrings%s', '%s'),
    'docstring_sig_override_so':
    ('https://stackoverflow.com/questions/12082570/override-function-declaration-in-autodoc-for-sphinx/12087750#12087750%s', '%s'),
    'docstring_sig_override_cv':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/autodoc.html#confval-autodoc_docstring_signature%s', '%s'),
    'docstrings_guide_tc':
    ('https://thomas-cokelaer.info/tutorials/sphinx/docstring_python.html%s', '%s'),
    'python_docs_pickle': (
        f'https://docs.python.org/{python_doc_version}/library/pickle.html%s', '%s'),
    'python_docs_bytes':
    (f'https://docs.python.org/{python_doc_version}/library/stdtypes.html#binary-sequence-types-bytes-bytearray-memoryview%s', '%s'),
    'python_docs_marshal': (
        f'https://docs.python.org/{python_doc_version}/library/marshal.html%s', '%s'),

    # PyO3 Stuff
    'pyo3_crate_home': (f'https://docs.rs/crate/pyo3/{pyo3_version}%s', '%s'),
    'pyo3_dev_api_home': (f'https://docs.rs/pyo3/{pyo3_version}/pyo3/%s', '%s'),
    'pyo3_user_guide': (f'https://pyo3.rs/v{pyo3_version}/%s', '%s'),
    'pyo3_github': ('https://github.com/pyo3/pyo3%s', '%s'),
    'pyo3_pyclass': (f'https://pyo3.rs/v{pyo3_version}/class.html%s', '%s'),
    'pyo3_pyfunction': (f'https://pyo3.rs/v{pyo3_version}/function.html%s', '%s'),
    'pyo3_pymodule': (f'https://pyo3.rs/v{pyo3_version}/module.html%s', '%s'),

    # Rust Stuff
    'rust_homepage': ('https://www.rust-lang.org%s', '%s'),
    'rust_cargo_doc': (
        'https://doc.rust-lang.org/cargo/commands/cargo-doc.html%s', '%s'),
    'rust_docstrings':
    ('https://doc.rust-lang.org/stable/rust-by-example/meta/doc.html#doc-comments%s', '%s'),

    # TOML
    'toml_homepage': ('https://toml.io/en/%s', '%s'),

    # Origen Github links
    'origen_sdk_home': (f'{links["home"]}%s', '%s'),
    'origen_github_home': (f'{links["core"]["github_home"][1]}%s', '%s'),
    'origen_issues_home': (f'{links["core"]["issues"][1]}%s', '%s'),
    'origen_issues_bugs':
    ('https://github.com/Origen-SDK/o2/issues?q=is:open+is:issue+label:bug%s', '%s'),
    'origen_core_team': (f'{links["core"]["core_team"][1]}%s', '%s'),
    'origen_project_tracker': (f'{links["core"]["core_team"][1]}%s', '%s'),
    'origen_so_home': (f'{links["so_tag"][1]}%s', '%s'),
    'origen_core_init_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/origen/__init__.py%s', '%s'),
    'origen_core_ose_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/origen/web/origen_sphinx_extension/__init__.py%s', '%s'),
    'origen_core_pytester_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/rust/pyapi/src/tester.rs%s', '%s'),
    'origen_core_guides_root_src':
    (f'https://github.com/Origen-SDK/o2/tree/{core_github_branch}/python/origen/web/source/guides%s', '%s'),
    'origen_core_guides__conf_dir_src':
    (f'https://github.com/Origen-SDK/o2/tree/{core_github_branch}/python/origen/web/source/_conf%s', '%s'),
    'origen_core_guides_conf_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/web/source/conf.py%s', '%s'),
    'origen_core_shorthand_init_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/origen/web/shorthand/__init__.py%s', '%s'),
    'origen_core_web_init_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/origen/web/__init__.py%s', '%s'),
    'origen_app_shorthand_defs_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/web/source/_conf/shorthand.py%s', '%s'),
    'origen_core_dev_guides_root_src':
    (f'https://github.com/Origen-SDK/o2/tree/{core_github_branch}/python/origen/web/source/guides/developers%s', '%s'),
    'origen_src_origen.application':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/origen/application.py%s', '%s'),
    'origen_src_example_commands':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/test_apps/python_app/example/commands/examples.py%s', '%s'),
    'origen_example_app_config': (
        f'{github_root__python_app}/config/application.toml%s', '%s'),
    'origen_example_config': (
        f'{github_root__python_app}/config/origen.toml%s', '%s'),
    'origen_specs_users': (
        f'{github_root__python_app}/tests/origen_utilities/test_users.py%s', '%s'),
    'origen_specs_ldap': (
        f'{github_root__python_app}/tests/origen_utilities/test_ldap.py%s', '%s'),
    'origen_specs_session_store':
    (f'{github_root__python_app}/tests/origen_utilities/test_session_store.py%s', '%s'),

    # LDAP
    'ldap_wiki': ('https://ldapwiki.com/wiki/LDAP%s', '%s'),
    'ldap_invalid_credentials': (
        'https://ldapwiki.com/wiki/LDAP_INVALID_CREDENTIALS%s', '%s'),
    'ldap_filters':
    ('https://confluence.atlassian.com/kb/how-to-write-ldap-search-filters-792496933.html%s', '%s'),
    'ldap_test_server':
    ('https://www.forumsys.com/tutorials/integration-how-to/ldap/online-ldap-test-server/%s', '%s'),

    # Git
    'git': ('https://git-scm.com/%s', '%s'),
    'git_configuration': (
        'https://git-scm.com/book/en/v2/Customizing-Git-Git-Configuration%s', '%s'),
    'git_pull_requests':
    ('https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/about-pull-requests%s', '%s'),

    # Other
    'mvc_dp_wiki': ('https://en.wikipedia.org/wiki/Model–view–controller%s', '%s'),
    'svg_to_png_converter': ('https://svgtopng.com/%s', '%s'),
    'linux_keyring': ('https://en.wikipedia.org/wiki/GNOME_Keyring%s', '%s'),
    'windows_credential_manager':
    ('https://support.microsoft.com/en-us/windows/accessing-credential-manager-1b5c916a-6a16-889f-8581-fc16e8165ac0%s', '%s'),

    # Python Package Servers
    'sonatype_nexus': ('https://www.sonatype.com/nexus/repository-oss%s', '%s'),
    'pypi_server': ('https://github.com/pypiserver/pypiserver%s', '%s'),
    'jfrog_artifactory': (
        'https://www.jfrog.com/confluence/display/JFROG/JFrog+Artifactory%s', '%s'),
}
