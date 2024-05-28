import origen

def run(**args):
    print(f"verbosity: {origen.logger.verbosity}")
    print(f"keywords: {origen.logger.keywords}")
    print(f"Args: {args}")
