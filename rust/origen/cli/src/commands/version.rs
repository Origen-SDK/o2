use origen::LOGGER;
use origen::STATUS;

pub fn run() {
    LOGGER.info(&format!("{}", STATUS.origen_version));
}
