## Development Environment Setup

1) [Install Rust](https://www.rust-lang.org/tools/install)

2) Enable Rust nightly version (this must be done for every o2 workspace):

~~~
rustup install nightly or rustup default nightly (this will make rust nightly the default version being run)
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
cargo build --workspace
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


8) Now that you have the Origen CLI available and Python, you can try booting the example app:

~~~
cd o2/example
origen setup
origen i
~~~

8a) If you are running this on the **Windows Sub-System Linux (WSL)** like I am, you might run into the following issues:
    **I was running Ubuntu 18.04 LTS as my WSL environment.**

A quick solution to this is running the following commands in your WSL environment
- **sudo pip install pyfs**
- **sudo pip install --upgrade keyrings.alt**

9) You should now be able to access the Origen interactive environment without issues


All being well, you now have a booted Origen console and an app instance available. e.g. `origen.app.config` should return a DICT from the values defined in `config/application.toml`.




### Screenshots of the above process

![image](https://user-images.githubusercontent.com/3895377/69558867-95aa3300-0f6e-11ea-9b80-ae9cd7fb8e81.png)
![image](https://user-images.githubusercontent.com/3895377/69559347-6647f600-0f6f-11ea-97d6-0414de2339d5.png)
![image](https://user-images.githubusercontent.com/3895377/69559358-6c3dd700-0f6f-11ea-8ee3-af2fe0bd318d.png)
![image](https://user-images.githubusercontent.com/3895377/69559433-90011d00-0f6f-11ea-9704-baf195b97daa.png)
![image](https://user-images.githubusercontent.com/3895377/69559492-a6a77400-0f6f-11ea-8b7d-46a95d3b6573.png)
![image](https://user-images.githubusercontent.com/3895377/69559551-c8a0f680-0f6f-11ea-9e40-d971b0a36afb.png)
![image](https://user-images.githubusercontent.com/3895377/69559574-cfc80480-0f6f-11ea-9053-7a037b67cbb6.png)
![image](https://user-images.githubusercontent.com/3895377/69559585-d5bde580-0f6f-11ea-9392-ea9622b03551.png)
![image](https://user-images.githubusercontent.com/3895377/69559611-dd7d8a00-0f6f-11ea-96c1-56a3ec4b9c23.png)
![image](https://user-images.githubusercontent.com/3895377/69559629-e9694c00-0f6f-11ea-9dde-d6158a51bf8d.png)
![image](https://user-images.githubusercontent.com/3895377/69559691-0867de00-0f70-11ea-96bf-e2cd680d87d5.png)
![image](https://user-images.githubusercontent.com/3895377/69559712-13bb0980-0f70-11ea-867f-c8012268ab06.png)
![image](https://user-images.githubusercontent.com/3895377/69559744-23d2e900-0f70-11ea-9da5-3e7bd7320104.png)
![image](https://user-images.githubusercontent.com/3895377/69559758-2c2b2400-0f70-11ea-829a-1268404321b0.png)
![image](https://user-images.githubusercontent.com/3895377/69559897-6dbbcf00-0f70-11ea-9b7d-f6a1e8b3de10.png)
![image](https://user-images.githubusercontent.com/3895377/69559928-79a79100-0f70-11ea-9641-1728ec7ea31f.png)
![image](https://user-images.githubusercontent.com/3895377/69560272-1c600f80-0f71-11ea-9951-d326591847b0.png)
