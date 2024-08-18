import _origen

REGRESSIONS_WORKFLOW_NAME = "regression_test.yml"

is_gh_regressions = (_origen.utility.revision_control.github.get_current_workflow_name == REGRESSIONS_WORKFLOW_NAME)
