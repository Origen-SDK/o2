def no_index_clashes(app, node):
    ''' Adds the :noindex: directive to matching nodes '''
    if node.name in app.config.origen_api_module_data_clashes:
        clashes = app.config.origen_api_module_data_clashes[node.name]
        for clash in clashes:
            node.variables[clash.split('.')[-1]][1]['directives'].append(
                'noindex')
