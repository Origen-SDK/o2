use origen::LOGGER;
use std::process::Command;
//use origen::core::term::*;

pub fn run(_command: Option<&str>, _remote: Option<&str>) {
//    println!("COMMAND: {:?}", _command);
    if _command.is_none() {
//           function to start web server
//           pass documentation location to serve function
        is_compile();
    } else {
        let name = _command.unwrap();
        let remote = _remote.unwrap();
        if name == "compile" {
            if remote == "remote" {
//            push to remote server here                
              is_remote();
            } else {
                LOGGER.info(&format!("2: running is compile, currently should never get here"));
            }
        }
    }


pub fn is_compile() {
    LOGGER.info(&format!("running is_compile"));
    let status = Command::new("origen")
                         .arg("compile")
                         .arg("../python/templates/dut_info.txt.mako")
                         .status()
                         .expect("ls command failed to start");
    LOGGER.info(&format!("is_compile command process exited with: {}", status));
    assert!(status.success());
}    
pub fn is_remote() {
    LOGGER.info(&format!("running is_remote"));
    let status = Command::new("ls")
                         .arg("-l")
                         .arg("-a")
                         .status()
                         .expect("ls command failed to start");
    LOGGER.info(&format!("is_remote command process exited with: {}", status));
    assert!(status.success());
}
    std::process::exit(0)

}
