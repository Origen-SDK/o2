import origen

defs = {
    #'output_dir': origen.web.interbuild_dir.joinpath('shorthand'),
    'namespace': 'origen',
    'abslinks': {
        'home':
        origen.web.ORIGEN_CORE_HOMEPAGE,
        'so_tag': ('Origen stack overflow',
                   'https://stackoverflow.com/questions/tagged/origen-sdk'),
        'core': {
            'core_team': ('Origen core team',
                          'https://github.com/orgs/Origen-SDK/teams/core'),
            'issues':
            ('Origen issues page', 'https://github.com/Origen-SDK/o2/issues'),
            'github_home':
            ('Origen Github project', 'https://github.com/Origen-SDK/o2'),
            'project_tracker': ("Origen's project tracker",
                                'https://github.com/Origen-SDK/o2/projects'),
        },
        'guides': {
            'api':
            ('Origen API',
             f'{origen.web.ORIGEN_CORE_HOMEPAGE}/interbuild/autoapi/origen/origen'
             ),
            'documenting':
            f'{origen.web.ORIGEN_CORE_HOMEPAGE}/guides/documenting',
            'pattern_api':
            f'{origen.web.ORIGEN_CORE_HOMEPAGE}/guides/testers/pattern_generation/pattern_api',
            'program_api':
            f'{origen.web.ORIGEN_CORE_HOMEPAGE}/guides/testers/program_generation/program_api',
            'logger':
            f'{origen.web.ORIGEN_CORE_HOMEPAGE}/guides/runtime/utilities/logger',
        }
    },
}
