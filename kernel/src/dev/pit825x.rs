use spin::Mutex;

use pit825x::ProgrammableIntervalTimer;

/// Global static refrence to the PIT.
pub static PIT: Mutex<ProgrammableIntervalTimer> = Mutex::new(ProgrammableIntervalTimer::new(0));
