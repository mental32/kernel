pub enum PCIExpressError {
    Uncorrectable(PCIUncorrectable),
    Correctable(PCICorrectable),
}

/// Uncorrectable errors are those error conditions that impact functionality of the interface. There is
/// no mechanism defined in this specification to correct these errors. Reporting an uncorrectable error
/// is analogous to asserting SERR# in PCI/PCI-X. For more robust error handling by the system, this
/// specification further classifies uncorrectable errors as Fatal and Non-fatal.
pub enum PCIUncorrectable {}

/// Correctable errors include those error conditions where hardware can recover without any loss of
/// information. Hardware corrects these errors and software intervention is not required. For
/// example, an LCRC error in a TLP that might be corrected by Data Link Level Retry is considered a
/// correctable error. Measuring the frequency of Link-level correctable errors may be helpful for
/// profiling the integrity of a Link.
pub enum PCICorrectable {}
