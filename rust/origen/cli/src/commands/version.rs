use origen::STATUS;
use origen::LOGGER;

pub fn run() {
    LOGGER.info(&format!("{}", STATUS.origen_version));
}
