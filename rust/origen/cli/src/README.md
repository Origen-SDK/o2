## Command Line Interpreter

### commands

This folder contains the implementators of the available commands, "generate", "interactive", etc.

### bins.rs

The main entry point, calls commands::launch() to run the requested command

### python.rs

Code specific to launching the python environment, called from command::launch() located in commands/mod.rs

## Launch process

The Python app launch process follows this flow:

1) Rust CLI builds the command to lauch Python and passes interpreted command line arguments to Python through Poetry.

2) Poetry launches the Python run environment, boot.py (in the Python source area) interprets the command line 
arguments passed to Poetry on it's commandline

3) Run arguments are then passed from boot.py to the Rust run-time library