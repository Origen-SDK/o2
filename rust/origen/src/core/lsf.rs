//! This module is responsible for abstracting and managing job submissions to LSF

use std::env;

pub fn is_running_remotely() -> bool {
    // See here for info about LSB env vars:
    //   https://www.ibm.com/support/knowledgecenter/SSWRJV_10.1.0/lsf_config_ref/lsf_envars_job_exec.html
    env::var("LSB_JOBID").is_ok()
}
