This directory contains workspaces for the project defined by bom.toml (the project
BOM), only the project leaders should modify this file.

To create a new workspace run:

~~~
origen proj create <MY_WORKSPACE_NAME>
~~~

You can then edit the bom.toml file within your personal workspace if you want to
override any of the package sources inherited from the project BOM.

To apply any changes made via your workspace BOM, run the following command from
within your workspace (this can also be used to update open-ended package versions,
such as any references to 'Trunk' or another branch name):

~~~
origen proj update
~~~

To see the additional options for the update command run:

~~~
origen proj update -h
~~~

To see what other project commands are available run:

~~~
origen proj
~~~

