import origen

doc_root = "guides/documenting"
testers_root = 'guides/testers'
pattern_api_root = f'{testers_root}/pattern_generation/pattern_api'
program_api_root = f'{testers_root}/program_generation/program_api'
device_modeling_root = 'guides/device_modeling'
utility_root = 'guides/runtime/utilities'
autoapi_root = 'interbuild/autoapi'
getting_started_root = 'guides/getting_started'

origen_shorthand_defs = {
  'extlinks': {
    'core_team': (
      'Origen core team',
      'origen_core_team'
    ),
    'issues_page': (
      'Origen issues page',
      'origen_issues_home'
    ),
    'github_home': (
      'Origen Github project',
      'origen_github_home'
    ),
    'project_tracker': (
      "Origen's project tracker",
      'origen_project_tracker'
    ),
    'so_tag': (
      'Origen stack overflow',
      'origen_so_home'
    ),
    'jinja': 'jinja_home',

    'sphinx_app': ('Sphinx app', 'sphinx_app'),
    'sphinx_ext': ('sphinx extension', 'sphinx_extensions'),
    'sphinx_exts': ('sphinx extensions', 'sphinx_extensions'),
    'sphinx_ref': ('sphinx :ref:', 'sphinx_ref_role'),
    'sphinx_refs': ('sphinx references', 'sphinx_ref_role'),
    'sphinx_doc': ('sphinx :doc:', 'sphinx_doc_role'),
    'sphinx_docs': ('sphinx docs', 'sphinx_doc_role'),
    'sphinx_doc_role': ('sphinx :doc: role', 'sphinx_doc_role'),
    'sphinx_config_var': ('Sphinx configuration variable', 'sphinx_conf'),
    'sphinx_conf_var': ('Sphinx configuration variable', 'sphinx_conf'),
    'sphinx_config_vars': ('Sphinx configuration variables', 'sphinx_conf'),
    'sphinx_conf_vars': ('Sphinx configuration variables', 'sphinx_conf'),
    'sphinx_build_cmd': ('sphinx-build command', 'sphinx_build_cmd'),

    'autosectionlabel': 'autosectionlabel_home',
    'autosectionlabel_prefix_document': 'autosectionlabel_prefix_document_config',
    'inheritance_diagram': ('inheritance diagram', 'inheritance_diagram_home'),
    'napoleon': 'napoleon_home',
    'extlinks': 'extlinks_home',
    'recommonmark': 'recommonmark_home',

    'markdown': 'markdown_home',
    'rst_subs': ('RST substitutions', 'rst_substitutions'),
    'rst_include_directive': ('RST include directive', 'rst_include_directive'),
    'docstrings': 'docstrings_intro',
    'docstring': 'docstrings_intro',
    'google_docstring_spec': ('Google Docstring Spec', 'google_docstring_spec'),
    'numpy_docstring_spec': ('Numpy Docstring Spec', 'numpy_docstring_spec'),
    'autodoc': ('Autodoc', 'autodoc_home'),
    'autoapi': ('AutoAPI', 'autoapi_home'),
    'Autodoc': ('Autodoc', 'autodoc_home'),
    'Autoapi': ('AutoAPI', 'autoapi_home'),
    'py_domain': (
      'Sphinx Python Domain',
      'sphinx_python_domain'
    ),

    # Python data structures. Link to the API instead of
    # placing in quotes, italics, or inline code blocks.
    'dict': 'python_docs_dict',
    'dicts': 'python_docs_dict',
    'tuple': 'python_docs_tuple',
    'tuples': 'python_docs_tuple',
    'list': 'python_docs_list',
    'lists': 'python_docs_list',
    'pathlib.Path': 'python_docs_pathlib',

    'src_code': {
      'origen_init': 'origen_core_init_src',
      'ose_init': 'origen_core_ose_src',
      'guides_root': 'origen_core_guides_root_src',
      'origen_app_shorthand_defs': 'origen_app_shorthand_defs_src',
      'pytester': 'origen_core_pytester_src',
      'dev_guides_root': 'origen_core_dev_guides_root_src',
      'origen.application': 'origen_src_origen.application',
      'example_commands': 'origen_src_example_commands',
      'core_conf': ('Origen core conf.py', 'origen_core_guides__conf'),
      '_conf_dir': 'origen_core_guides__conf_dir_src',
    },

    # Rust stuff
    'rust': 'rust_homepage',
    'Rust': 'rust_homepage',
    'cargo_doc': ('cargo doc', 'rust_cargo_doc'),
  },

  'substitutions': {
    'conf.py': '``conf.py``',
    'inline_ose': '``origen sphinx extensions``',
  },

  'statics': {
    'rustdoc_origen': (
      'Rustdoc Origen',
      '_static/build/rustdoc/origen/doc/origen/index',
    ),
    'rustdoc_pyapi': (
      'pyapi',
      '_static/build/rustdoc/pyapi/doc/_origen/index'
    ),
    'rustdoc_cli': (
      'Rustdoc CLI',
      '_static/build/rustdoc/cli/doc/origen/index',
    ),
    'example_application_docs': (
      'example application',
      '_static/build/origen_sphinx_extension/example/sphinx_build/index'
    )
  },

  'shorthand_defs': {
    'shorthand_referencing': ('Shorthand referencing', 'shorthand~basic_usage'),
  },

  'docs': {
    'guides': (
      'Guides',
      'index'
    ),
    'dev_install': (
      'development installation guide',
      'guides/developers/installation'
    ),
    'origen_api': (
      'Origen API',
      f'{autoapi_root}/origen/origen'
    ),
    '_origen_api': (
      '_origen API',
      f"{autoapi_root}/_origen/_origen"
    ),
    'ose_api': (
      'Origen Sphinx Extension API',
      f'{autoapi_root}/origen/origen.web.origen_sphinx_extension'
    ),
    'origen_web_api': (
      'origen.web API',
      f'{autoapi_root}/origen/origen.web'
    ),
    'documenting': {
      'rustdoc': (
        # Currently RustDoc API and user guides are the same.
        'RustDoc',
        f'{autoapi_root}/origen/origen.web.rustdoc'
      ),
    },
    'rustdoc_api': (
      'RustDoc API',
      f'{autoapi_root}/origen/origen.web.rustdoc'
    ),
  },

  'refs': {
    'origen_app': (
      'Origen app',
      f'{getting_started_root}/the_origen_app:The Origen App'
    ),
    'origen_cli': (
      'Origen CLI',
      f'{getting_started_root}/core_concepts:The Origen CLI'
    ),
    'timesets': (
      'Timesets',
      f'{device_modeling_root}/timesets:Timesets'
    ),
    'pins': (
      'Pins',
      f'{device_modeling_root}/pins:Pins'
    ),
    'bits': (
      'Bits',
      f'{device_modeling_root}/bits_and_registers:Bits'
    ),
    'registers': (
      'Registers',
      f'{device_modeling_root}/bits_and_registers:Registers'
    ),
    'logger': (
      'logger',
      f'{utility_root}/logger:Logger'
    ),
    'community_contributions': (
      'community contributions',
      'community:Contributing'
    ),
    'web_output_dir': (
      'web output directory',
      f"{doc_root}/further_customizing_your_sphinx_app:Application Customizations"
    ),
    'web_cmd': (
      'origen web',
      f"{doc_root}/building_your_webpages:Origen Web"
    ),
    'ose': (
      'origen sphinx extension',
      f"{doc_root}/the_origen_sphinx_extension:The Origen Sphinx Extension"
    ),
    'origen-s_sphinx_app': (
      'Origen\'s sphinx app',
      f"{doc_root}/your_sphinx_app:Origen's Sphinx App"
    ),
    'ose_theme': (
      'origen theme',
      f"{doc_root}/the_origen_sphinx_extension:The Origen Theme"
    ),
    'ose_theme_opts': (
      'origen sphinx extension theme options',
      f"{doc_root}/the_origen_sphinx_extension:Origen Theme Options"
    ),
    'ose_config_vars': (
      'origen sphinx extension config variables',
      f"{doc_root}/the_origen_sphinx_extension:Configuration Variables"
    ),
    'ose_favicon': (
      'OSE favicon',
      f"{doc_root}/the_origen_sphinx_extension:Favicon"
    ),
    'ose_logos': (
      'OSE logos',
      f"{doc_root}/the_origen_sphinx_extension:Logos"
    ),
    'ose_subprojects': (
      'OSE SubProjects',
      f"{doc_root}/the_origen_sphinx_extension:SubProjects"
    ),
    'shorthand': (
      'Shorthand',
      f'{doc_root}/shorthand_ext:The Shorthand Extension'
    ),

    'prog-gen': {
      'comments': 'guides/testers/program_generation/program_api:Comments',
    },

    'pat-gen': {
      'comments': 'guides/testers/pattern_generation/pattern_api:Comments',
    },

    'developers': {
      'home': (
        'developers',
        'guides/developers:Developers'
      ),
      'doc_gen_arch': (
        'documentation generation architecture',
        'guides/developers/doc_gen_arch:Documentation Generation Architecture',
      ),
      'documenting_the_core': (
        'documenting the core',
        'guides/developers/documenting_the_core:Documenting The Core'
      )
    },

    'documenting': {
      'introduction': (
        'Documenting: Introduction',
        f"{doc_root}/introduction:Introduction"
      ),
      'core_concepts': (
        'Documenting: Core Concept',
        f"{doc_root}/core_concepts:Core Concepts"
      ),
      'block_diagram': (
        'block diagram',
        f"{doc_root}/core_concepts:Doc System Block Diagram"
      ),
      'api_generation': (
        'API generation',
        f"{doc_root}/your_sphinx_app:Automatic API Generation"
      ),
      'origen_included_extensions': (
        'Origen included extensions',
        f"{doc_root}/your_sphinx_app:Extensions"
      ),
      'shorthand': (
        'Shorthand',
        f'{doc_root}/shorthand_ext:The Shorthand Extension'
      ),
      'origen_shorthand_defs': (
        "Origen's shorthand defs",
        f"{doc_root}/reference:Origen's Shorthand Defs"
      )
    }
  }
}

# Couldn't decided whether or not to include the hyphen, so just added them again,
# essentially aliasing them.
origen_shorthand_defs['refs']['patgen'] = origen_shorthand_defs['refs']['pat-gen']
origen_shorthand_defs['refs']['proggen'] = origen_shorthand_defs['refs']['prog-gen']
