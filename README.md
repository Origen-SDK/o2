[![Gitter](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/Origen-SDK/users?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Build Status](https://travis-ci.org/Origen-SDK/origen.svg)](https://travis-ci.org/Origen-SDK/origen)
[![Coverage Status](https://coveralls.io/repos/Origen-SDK/origen/badge.svg?branch=master&service=github)](https://coveralls.io/github/Origen-SDK/origen?branch=master)
[![Inline docs](http://inch-ci.org/github/Origen-SDK/origen.svg)](http://inch-ci.org/github/Origen-SDK/origen)


# Origen

The Origen Semiconductor Developer's Kit

NOTE - These instructions are not complete and not working yet, will be available asap



## Development Environment Setup

1) [Install Rust](https://www.rust-lang.org/tools/install) 
2) Enable Rust nightly version (this must be done for every o2 workspace):

~~~
rustup install nightly
cd path/to/o2
rustup override set nightly
~~~

3) By this point make sure your $PATH contains the following to make the `cargo` command available:

~~~
export PATH="$HOME/.cargo/bin:$PATH"
~~~

4) Compile the Rust code (you will repeat this step everytime you change it):
~~~
cd o2/rust/origen
cargo build
~~~

On Ubuntu, the following packages may need to be installed if you get errors:

~~~
sudo apt install libssl-dev
sudo apt install pkg-config
~~~

5) Add this dir to your $PATH, ahead of any other dir that provides an `origen` command:
~~~
export PATH="</path/to/your>/o2/rust/origen/target/debug:$PATH"
~~~

6) Verify that you now have the new `origen` command available:

~~~
$ origen -v
Origen: 2.0.0-pre0
~~~

7) Make sure your system has at least Python 3.5 available



4) [Install Poetry](https://poetry.eustace.io/docs/) (equivalent to Ruby's Bundler), this way:

  * Download [this file](https://raw.githubusercontent.com/sdispater/poetry/master/get-poetry.py)
  


5) Make sure your $PATH contains the following directories:

~~~
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.poetry/bin:$PATH"
~~~

6) Currently we need a pre-release version of Poetry, to install it:
~~~
poetry self:update --preview
~~~

