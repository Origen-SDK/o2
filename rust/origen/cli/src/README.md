## Command Line Interface (CLI)

The Origen CLI (the 'origen' command) has two primary modes of execution:

* As a fully self contained and standalone tool
* Within the context of an Origen application written in Python

When running as a standalone tool, everything is done in Rust, this has the benefit of making Origen extremely
easy to deploy as single binary, while realizing all of the speed and correctness guarantees that come from
using a compiled language like Rust.

When running within an application, as much as possible is implemented in Rust to avoid duplication
and to maintain the speed/correctness guarantees from Rust. So while on the face of it Origen may seem like
a Python tool to the average user, it is still primarily a Rust tool under the hood.

Origen's functionality is mainly implemented within the 'origen' crate (in `rust/origen/src`) and this is
included by the CLI crate (`rust/origen/cli/src`) which is mainly responsible for defining the commands that
are available to the user.
When a user invokes Origen in standalone mode, the CLI version and the Origen version are always the same.

When the user invokes Origen within an application, then the CLI and Origen version can be different.
The high level flow is that the CLI boots up standalone, it realizes that it is within an application and
so then it boots up Python and then the Python runtime determines and loads the version of Origen that
is required by the application (distributed as a Python package).

It is envisaged that once Origen 2 stabilizes, it will be common for organizations to update their CLI
version much less frequently than the Origen Python package.

~~~text
         ________________
        |                |
user -> |  CLI ver X     |
        |________________|
                ^
         _______|________
        |                |
        |  Origen ver X  |
        |________________|
 
        

         ________________        ________________
        |                |      |                |
user -> |  CLI ver X     |  ->  |  Python App    | 
        |________________|      |________________|
                                        ^
                                 _______|________
                                |                |
                                |  Origen ver Y  |
                                |________________|

~~~


### commands

This folder contains the implementors of the available commands, "generate", "interactive", etc.

### bins.rs

This is the main entry point which uses [a popular Rust crate called Clap](https://docs.rs/clap) to define
the available Origen commands, this is split into commands which are available globally (when running standalone)
and those which are only made available when executing inside an application workspace.

### python.rs

Code specific to launching the python environment, called from `commands::launch()` located in `commands/mod.rs`.

### Booting Standalone

Standalone operation is straightforward and is (currently) stateless.
The user invokes the Origen command and the Clap code in `origen/cli/src/bin.rs` processes the user input.
The requested command is then either handled locally within the CLI crate, or else calls are made to functions
from the Origen crate, or in many cases a command will be implemented as a combination of the two.

### Booting Within an Application

Unsurprisingly, the boot process within an application setting is more complicated.
The Python application environment is currently managed by a tool called Poetry, which is analogous to Bundler
for Ruby or Cargo for Rust.
Origen aims to abstract the detail of that from the user and the command `origen setup` should completely
set up a working Python environment for a given application workspace.
This will bring in the Origen package and any other Python package dependencies specified within the application's
`pyproject.toml` file.

Note that the CLI version is still the same CLI that was always there, this will be loaded via the user's PATH
from outside of the application workspace, normally from a company's central tool repository.

Even within an application context, some commands may be fully handled by the CLI, in that case the boot process
is identical to the standalone case.

However, when it is determined that the requested command will involve Python then the following sequence
is performed:

1) The command and arguments are processed by the Clap code in the CLI.

2) Python is invoked within a new process by the `run` function within `origen/cli/src/python.rs`. This generates
   a command which will invoke a short Origen boot script within the Poetry environment, the command it runs will
   be something like: 

   ~~~
   ~/.poetry/bin/poetry run python3 -c "from origen.boot import __origen__; __origen__('generate', files=['my_pat.py'])"
   ~~~

3) The CLI's work is now done and it simply sits and waits for the above process to finish and when it does it
   will exit and return the exit code from the Python process.

4) The Python process is kicked off by the `__origen__` function in `python/origen/boot.py`. This will load the
   Origen extension which creates a new instance of Origen's Rust runtime. Many of the arguments originally given
   to the CLI are then provided to this new Rust instance, for example the files argument in the example above.
 
5) The Python code will then continue to execute the requested command, offloading to Rust code whenever possible.
   For example, the de-composing of any lists or directories in file arguments is done in Rust so that the same code
   can be used by standalone and application commands. It is for this reason that the file argument is passed from
   the CLI into Python and then immediately handed over to the new Rust process.

