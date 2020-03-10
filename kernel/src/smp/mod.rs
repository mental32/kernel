use acpi::Processor;

use crate::KernelResult;

pub fn sipi(application_processor: Processor) -> KernelResult<()> {
    Ok(())
}
