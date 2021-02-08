from origen.web.origen_sphinx_extension.shorthand_defs import defs

# TODO(coreyeng) Try to pull the version and branch dynamically
#   so it always up-to-date with the workspace - instead of hardcoded.
pyo3_version = '0.8.5'
python_doc_version = '3'  # Points to latest, stable Python3 version.
links = defs['abslinks']
core_github_branch = "mailer_ldap_users_session_and_more"
github_root__python_app = f"https://github.com/Origen-SDK/o2/blob/{core_github_branch}/test_apps/python_app"

extlinks = {
    # Sphinx basics or built-in (non-extension) features
    'sphinx_homepage': ('https://www.sphinx-doc.org/en/master/index.html%s',
                        ''),
    'sphinx_app':
    ('https://www.sphinx-doc.org/en/master/usage/quickstart.html#getting-started%s',
     ''),
    'sphinx_docs': ('https://www.sphinx-doc.org/en/master/contents.html%s',
                    ''),
    'sphinx_extensions':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/index.html%s', ''),
    'sphinx_themes':
    ('https://www.sphinx-doc.org/en/master/usage/theming.html#themes%s', ''),
    'sphinx_using_a_theme':
    ('https://www.sphinx-doc.org/en/master/usage/theming.html#using-a-theme%s',
     ''),
    'sphinx_builtin_themes':
    ('https://www.sphinx-doc.org/en/master/usage/theming.html#builtin-themes%s',
     ''),
    'sphinx_add_theme':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.add_html_theme%s',
     ''),
    'sphinx_creating_themes':
    ('https://www.sphinx-doc.org/en/3.x/development/theming.html#creating-themes%s',
     ''),
    'sphinx_available_themes': ('https://sphinx-themes.org/%s', ''),
    'sphinx_project_examples':
    ('https://www.sphinx-doc.org/en/master/examples.html#projects-using-sphinx%s',
     ''),
    'sphinx_conf':
    ('https://www.sphinx-doc.org/en/master/usage/configuration.html#module-conf%s',
     ''),
    'sphinx_add_config_var':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.add_config_value%s',
     ''),
    'sphinx_confval_html_logo':
    ('https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_logo%s',
     ''),
    'sphinx_confval_html_favicon':
    ('https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_favicon%s',
     ''),
    'sphinx_confval_html_theme_options':
    ('https://www.sphinx-doc.org/en/3.x/usage/configuration.html#confval-html_theme_options%s',
     ''),
    'sphinx_rst':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/index.html%s',
     ''),
    'sphinx_rst_primer':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/basics.html%s',
     ''),
    'sphinx_rst_directives':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html%s',
     ''),
    'sphinx_templating':
    ('https://www.sphinx-doc.org/en/master/templating.html#templating%s', ''),
    'sphinx_manpages':
    ('https://www.sphinx-doc.org/en/master/man/index.html%s', ''),
    'sphinx_build_cmd':
    ('https://www.sphinx-doc.org/en/master/man/sphinx-build.html%s', ''),
    'sphinx_build_phases':
    ('https://www.sphinx-doc.org/en/master/extdev/index.html#build-phases%s',
     ''),
    'sphinx_connect':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx.application.Sphinx.connect%s',
     ''),
    'sphinx_core_events':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#sphinx-core-events%s',
     ''),
    'sphinx_event_config_inited':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#event-config-inited%s',
     ''),
    'sphinx_event_builder_inited':
    ('https://www.sphinx-doc.org/en/master/extdev/appapi.html#event-builder-inited%s',
     ''),
    'sphinx_alabaster_theme': ('https://alabaster.readthedocs.io/en/latest/%s',
                               ''),
    'sphinx_nitpicky':
    ('https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-nitpicky%s',
     ''),
    'sphinx_xrefing':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/roles.html#cross-referencing-arbitrary-locations%s',
     ''),
    'sphinx_ref_role':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/roles.html#role-ref%s',
     ''),
    'sphinx_doc_role':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/roles.html#role-doc%s',
     ''),
    'sphinx_python_domain':
    ('https://www.sphinx-doc.org/en/master/usage/restructuredtext/domains.html#the-python-domain%s',
     ''),

    # RST tutorials
    'rst_quickstart':
    ('https://docutils.sourceforge.io/docs/user/rst/quickstart.html%s', ''),
    'rst_cheatsheet':
    ('https://docutils.sourceforge.io/docs/user/rst/cheatsheet.txt%s', ''),
    'rst_docs': ('https://docutils.sourceforge.io/rst.html%s', ''),
    'rst_spec':
    ('https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html%s',
     ''),
    'rst_cokelaer_cheatsheet':
    ('https://thomas-cokelaer.info/tutorials/sphinx/rest_syntax.html#contents-directives%s',
     ''),
    'rst_guide_zephyr':
    ('https://docs.zephyrproject.org/latest/guides/documentation/index.html%s',
     ''),
    'rst_substitutions':
    ('https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#substitution-definitions%s',
     ''),
    'rst_include_directive':
    ('https://docutils.sourceforge.io/docs/ref/rst/directives.html#include%s',
     ''),

    # Jinja tutorials
    'jinja_home': ('https://palletsprojects.com/p/jinja/%s', ''),
    'jinja_docs': ('https://jinja.palletsprojects.com/en/master/%s', ''),

    # Extension homepages, tutorials, & wrapped libraries
    'recommonmark_home': ('https://recommonmark.readthedocs.io/en/latest/%s',
                          ''),
    'recommonmark_embedded_rst':
    ('https://recommonmark.readthedocs.io/en/latest/auto_structify.html#embed-restructuredtext%s',
     ''),
    'markdown_home': ('https://www.markdownguide.org/%s', ''),
    'autoapi_home': ('https://autoapi.readthedocs.io/%s', ''),
    'autoapi_usage': ('https://autoapi.readthedocs.io/#usage%s', ''),
    'autodoc_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/autodoc.html%s',
     ''),
    'bootstrap4':
    ('https://getbootstrap.com/docs/4.5/getting-started/introduction/%s', ''),
    'bootstrap4_widgets':
    ('https://getbootstrap.com/docs/4.0/components/alerts/%s', ''),
    'bootstrap4_sphinx_theme':
    ('http://myyasuda.github.io/sphinxbootstrap4theme/%s', ''),
    'bootstrap4_sphinx_theme_options':
    ('http://myyasuda.github.io/sphinxbootstrap4theme/setup.html#html-theme-options%s',
     ''),
    'bootstrap4_sphinx_theme_templates':
    ('https://github.com/myyasuda/sphinxbootstrap4theme/tree/master/themes/sphinxbootstrap4theme%s',
     ''),
    'autosectionlabel_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/autosectionlabel.html%s',
     ''),
    'autosectionlabel_prefix_document_config':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/autosectionlabel.html#confval-autosectionlabel_prefix_document%s',
     ''),
    'napoleon_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/napoleon.html%s',
     ''),
    'google_docstring_spec':
    ('https://google.github.io/styleguide/pyguide.html%s', ''),
    'numpy_docstring_spec':
    ('https://numpydoc.readthedocs.io/en/latest/format.html#docstring-standard%s',
     ''),
    'inheritance_diagram_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/inheritance.html%s',
     ''),
    'inheritance_diagram_example':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/inheritance.html#examples%s',
     ''),
    'graphviz_ext_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/graphviz.html#module-sphinx.ext.graphviz%s',
     ''),
    'graphviz_home': ('https://graphviz.org/%s', ''),
    'graphviz_download': ('https://www.graphviz.org/download/%s', ''),
    'extlinks_home':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/extlinks.html%s',
     ''),
    'extlinks_config_var':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/extlinks.html#confval-extlinks%s',
     ''),

    # Other webpage generation related links
    'darkly': ('https://bootswatch.com/darkly/%s', ''),
    'dracula_pygments': ('https://draculatheme.com/pygments%s', ''),
    'o2_github_root': ('https://github.com/Origen-SDK/o2%s', ''),
    'static_website': ('https://en.wikipedia.org/wiki/Static_web_page%s', ''),

    # General Python stuff
    'python_docs': ('https://docs.python.org/3.8/index.html%s', ''),
    'python_docs_list':
    (f'https://docs.python.org/{python_doc_version}/library/stdtypes.html#lists%s',
     ''),
    'python_docs_tuple':
    (f'https://docs.python.org/{python_doc_version}/library/stdtypes.html#tuples%s',
     ''),
    'python_docs_dict':
    (f'https://docs.python.org/{python_doc_version}/library/stdtypes.html#mapping-types-dict%s',
     ''),
    'python_exception_hierarchy':
    (f'https://docs.python.org/{python_doc_version}/library/exceptions.html#exception-hierarchy%s',
     ''),
    'python_docs_pathlib': (
        f'https://docs.python.org/{python_doc_version}/library/pathlib.html%s',
        ''),
    'ticket_mako_multiple_newlines':
    ('https://stackoverflow.com/questions/22558067/how-to-convert-multiple-newlines-in-mako-template-to-one-newline%s',
     ''),
    'docstrings_spec': ('https://www.python.org/dev/peps/pep-0257/%s', ''),
    'docstrings_intro': (
        'https://www.programiz.com/python-programming/docstrings%s', ''),
    'docstring_sig_override_so':
    ('https://stackoverflow.com/questions/12082570/override-function-declaration-in-autodoc-for-sphinx/12087750#12087750%s',
     ''),
    'docstring_sig_override_cv':
    ('https://www.sphinx-doc.org/en/master/usage/extensions/autodoc.html#confval-autodoc_docstring_signature%s',
     ''),
    'docstrings_guide_tc':
    ('https://thomas-cokelaer.info/tutorials/sphinx/docstring_python.html%s',
     ''),
    'python_docs_pickle': (f'https://docs.python.org/{python_doc_version}/library/pickle.html%s', ''),
    'python_docs_bytes': (f'https://docs.python.org/{python_doc_version}/library/stdtypes.html#binary-sequence-types-bytes-bytearray-memoryview%s', ''),
    'python_docs_marshal': (f'https://docs.python.org/{python_doc_version}/library/marshal.html%s', ''),

    # PyO3 Stuff
    'pyo3_crate_home': (f'https://docs.rs/crate/pyo3/{pyo3_version}%s', ''),
    'pyo3_dev_api_home': (f'https://docs.rs/pyo3/{pyo3_version}/pyo3/%s', ''),
    'pyo3_user_guide': (f'https://pyo3.rs/v{pyo3_version}/%s', ''),
    'pyo3_github': ('https://github.com/pyo3/pyo3%s', ''),
    'pyo3_pyclass': (f'https://pyo3.rs/v{pyo3_version}/class.html%s', ''),
    'pyo3_pyfunction': (f'https://pyo3.rs/v{pyo3_version}/function.html%s',
                        ''),
    'pyo3_pymodule': (f'https://pyo3.rs/v{pyo3_version}/module.html%s', ''),

    # Rust Stuff
    'rust_homepage': ('https://www.rust-lang.org%s', ''),
    'rust_cargo_doc': (
        'https://doc.rust-lang.org/cargo/commands/cargo-doc.html%s', ''),
    'rust_docstrings':
    ('https://doc.rust-lang.org/stable/rust-by-example/meta/doc.html#doc-comments%s',
     ''),

    # Origen Github links
    'origen_sdk_home': (f'{links["home"]}%s', ''),
    'origen_github_home': (f'{links["core"]["github_home"][1]}%s', ''),
    'origen_issues_home': (f'{links["core"]["issues"][1]}%s', ''),
    'origen_issues_bugs':
    ('https://github.com/Origen-SDK/o2/issues?q=is:open+is:issue+label:bug%s',
     ''),
    'origen_core_team': (f'{links["core"]["core_team"][1]}%s', ''),
    'origen_project_tracker': (f'{links["core"]["core_team"][1]}%s', ''),
    'origen_so_home': (f'{links["so_tag"][1]}%s', ''),
    'origen_core_init_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/__init__.py%s',
     ''),
    'origen_core_ose_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/web/origen_sphinx_extension/__init__.py%s',
     ''),
    'origen_core_pytester_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/rust/pyapi/src/tester.rs%s',
     ''),
    'origen_core_guides_root_src':
    (f'https://github.com/Origen-SDK/o2/tree/{core_github_branch}/python/web/source/guides%s',
     ''),
    'origen_core_guides__conf_dir_src':
    (f'https://github.com/Origen-SDK/o2/tree/{core_github_branch}/python/web/source/_conf%s',
     ''),
    'origen_core_guides_conf_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/web/source/conf.py%s',
     ''),
    'origen_core_shorthand_init_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/web/shorthand/__init__.py%s',
     ''),
    'origen_core_web_init_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/web/__init__.py%s',
     ''),
    'origen_app_shorthand_defs_src':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/web/source/_conf/shorthand.py%s',
     ''),
    'origen_core_dev_guides_root_src':
    (f'https://github.com/Origen-SDK/o2/tree/{core_github_branch}/python/web/source/guides/developers%s',
     ''),
    'origen_src_origen.application':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/python/origen/application.py%s',
     ''),
    'origen_src_example_commands':
    (f'https://github.com/Origen-SDK/o2/blob/{core_github_branch}/test_apps/python_app/example/commands/examples.py%s',
     ''),
    'origen_example_app_config':
    (f'{github_root__python_app}/config/application.toml%s',
     ''),
    'origen_example_config':
    (f'{github_root__python_app}/config/origen.toml%s',
     ''),
    'origen_specs_users':
    (f'{github_root__python_app}/tests/origen_utilities/test_users.py%s',
     ''),
    'origen_specs_ldap':
    (f'{github_root__python_app}/tests/origen_utilities/test_ldap.py%s',
     ''),
    'origen_specs_session_store':
    (f'{github_root__python_app}/tests/origen_utilities/test_session_store.py%s',
     ''),

    # LDAP
    'ldap_wiki': ('https://ldapwiki.com/wiki/LDAP%s', ''),
    'ldap_invalid_credentials': ('https://ldapwiki.com/wiki/LDAP_INVALID_CREDENTIALS%s', ''),
    'ldap_filters': ('https://confluence.atlassian.com/kb/how-to-write-ldap-search-filters-792496933.html%s', ''),
    'ldap_test_server': ('https://www.forumsys.com/tutorials/integration-how-to/ldap/online-ldap-test-server/%s', ''),

    # Other
    'mvc_dp_wiki': ('https://en.wikipedia.org/wiki/Model–view–controller%s',
                    ''),
    'git_pull_requests':
    ('https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/about-pull-requests%s',
     ''),
    'svg_to_png_converter': ('https://svgtopng.com/%s', ''),

    # Python Package Servers
    'sonatype_nexus': ('https://www.sonatype.com/nexus/repository-oss%s', ''),
    'pypi_server': ('https://github.com/pypiserver/pypiserver%s', ''),
    'jfrog_artifactory': (
        'https://www.jfrog.com/confluence/display/JFROG/JFrog+Artifactory%s',
        ''),
}
