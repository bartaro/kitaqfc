//! NES mapper and board constraints used by native emission and image assembly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mapper {
    #[default]
    Nrom,
    Uxrom,
    Cnrom,
    Axrom,
    Mmc1,
    Mmc3,
    Mmc5,
    Vrc6,
    Vrc7,
    Fme7,
    Fds,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Board {
    #[default]
    Auto,
    Generic,
    Surom512,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mirroring {
    #[default]
    Horizontal,
    Vertical,
    FourScreen,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    FixedTop16,
    DuplicatedCommonTop16,
    SuromOuter256FixedTop16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BankSwitch {
    None,
    Uxrom,
    Axrom,
    Mmc1,
    Mmc1Surom,
    Mmc3,
    Mmc5,
    Vrc6,
    Vrc7,
    Fme7,
    Fds,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetReplication {
    CommonOnly,
    EveryPhysical16KBank,
    EveryPhysical32KHighBank,
}
#[derive(Clone, Debug)]
pub struct Profile {
    pub mapper: Mapper,
    pub board: Board,
    pub name: &'static str,
    pub mapper_number: u8,
    pub layout: Layout,
    pub bank_switch: BankSwitch,
    pub supports_prg_banking: bool,
    pub requires_power_on_stub: bool,
    pub requires_per_bank_reset: bool,
    pub reset_replication: ResetReplication,
    pub exact_prg_bytes: usize,
    pub requires_chr_ram: bool,
    pub chr_ram_bytes: usize,
    pub prg_ram_bytes: usize,
    pub default_battery: bool,
    pub max_logical_bank: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Options {
    pub mapper: Mapper,
    pub board: Board,
    pub mirroring: Mirroring,
    pub battery: Option<bool>,
}
impl Mapper {
    pub fn parse(raw: &str) -> Result<(Self, bool), String> {
        let s = raw.trim().to_ascii_lowercase();
        let value = match s.as_str() {
            "nrom" | "mapper0" => Self::Nrom,
            "uxrom" | "unrom" | "uorom" | "mapper2" => Self::Uxrom,
            "cnrom" | "mapper3" => Self::Cnrom,
            "axrom" | "anrom" | "mapper7" => Self::Axrom,
            "mmc1" | "sxrom" | "mapper1" | "surom512" => Self::Mmc1,
            "mmc3" | "txrom" | "mapper4" => Self::Mmc3,
            "mmc5" | "mapper5" => Self::Mmc5,
            "vrc6" | "mapper24" | "mapper26" => Self::Vrc6,
            "vrc7" | "mapper85" => Self::Vrc7,
            "fme7" | "sunsoft5b" | "sunsoft-5b" | "mapper69" => Self::Fme7,
            "fds" | "fds20" | "mapper20" | "famicom-disk-system" => Self::Fds,
            _ => {
                return Err(format!(
                    "error: --mapper unsupported value: {raw} (try nrom|uxrom|cnrom|axrom|mmc1|mmc3|mmc5|vrc6|vrc7|fme7|fds)"
                ));
            }
        };
        Ok((value, s == "surom512"))
    }
}
impl Board {
    pub fn parse(raw: &str) -> Result<Self, String> {
        Ok(match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Self::Auto,
            "generic" => Self::Generic,
            "surom" | "surom512" | "mmc1-surom512" => Self::Surom512,
            _ => {
                return Err(format!(
                    "error: --board unsupported value: {raw} (try auto|generic|surom512)"
                ));
            }
        })
    }
}
impl Mirroring {
    pub fn parse(raw: &str) -> Result<Self, String> {
        Ok(match raw.trim().to_ascii_lowercase().as_str() {
            "h" | "horz" | "horizontal" => Self::Horizontal,
            "v" | "vert" | "vertical" => Self::Vertical,
            "4" | "4screen" | "four_screen" | "four-screen" => Self::FourScreen,
            _ => return Err(format!("error: --mirroring unsupported value: {raw}")),
        })
    }
}
impl Options {
    pub fn set_mapper(&mut self, raw: &str) -> Result<(), String> {
        let (mapper, surom) = Mapper::parse(raw)?;
        self.mapper = mapper;
        if surom {
            self.board = Board::Surom512
        }
        Ok(())
    }
    pub fn profile(&self) -> Result<Profile, String> {
        if self.board == Board::Surom512 && self.mapper != Mapper::Mmc1 {
            return Err("error KQFC2601 KQFC-SUROM-BOARD-CONFLICT: board 'surom512' requires mapper 'mmc1' (mapper 1).".into());
        }
        Ok(Profile::new(self.mapper, self.board))
    }
    pub fn battery(&self) -> bool {
        self.battery.unwrap_or(self.board == Board::Surom512)
    }
}
impl Profile {
    pub fn new(mapper: Mapper, board: Board) -> Self {
        use Mapper::*;
        let (name, number, switch, banking, power, per_bank) = match mapper {
            Nrom => ("nrom", 0, BankSwitch::None, false, false, false),
            Uxrom => ("uxrom", 2, BankSwitch::Uxrom, true, false, false),
            Cnrom => ("cnrom", 3, BankSwitch::None, false, false, false),
            Axrom => ("axrom", 7, BankSwitch::Axrom, true, false, false),
            Mmc1 => ("mmc1", 1, BankSwitch::Mmc1, true, false, true),
            Mmc3 => ("mmc3", 4, BankSwitch::Mmc3, true, true, false),
            Mmc5 => ("mmc5", 5, BankSwitch::Mmc5, true, true, false),
            Vrc6 => ("vrc6", 24, BankSwitch::Vrc6, true, true, false),
            Vrc7 => ("vrc7", 85, BankSwitch::Vrc7, true, true, false),
            Fme7 => ("fme7", 69, BankSwitch::Fme7, true, true, false),
            Fds => ("fds", 20, BankSwitch::Fds, false, true, false),
        };
        let surom = board == Board::Surom512;
        Self {
            mapper: if surom { Mmc1 } else { mapper },
            board: if board == Board::Auto {
                Board::Generic
            } else {
                board
            },
            name: if surom { "mmc1" } else { name },
            mapper_number: if surom { 1 } else { number },
            bank_switch: if surom { BankSwitch::Mmc1Surom } else { switch },
            layout: if surom {
                Layout::SuromOuter256FixedTop16
            } else if mapper == Axrom {
                Layout::DuplicatedCommonTop16
            } else {
                Layout::FixedTop16
            },
            supports_prg_banking: banking || surom,
            requires_power_on_stub: power && !surom,
            requires_per_bank_reset: per_bank || surom,
            reset_replication: if surom {
                ResetReplication::EveryPhysical32KHighBank
            } else if mapper == Mmc1 {
                ResetReplication::EveryPhysical16KBank
            } else {
                ResetReplication::CommonOnly
            },
            exact_prg_bytes: if surom { 0x80000 } else { 0 },
            requires_chr_ram: surom,
            chr_ram_bytes: if surom { 0x2000 } else { 0 },
            prg_ram_bytes: 0x2000,
            default_battery: surom,
            max_logical_bank: if surom { 30 } else { i32::MAX },
        }
    }
    pub fn is_surom(&self) -> bool {
        self.board == Board::Surom512
    }
    pub fn has_fds(&self) -> bool {
        self.mapper == Mapper::Fds
    }
    pub fn logical_to_physical(&self, bank: i32) -> Result<i32, String> {
        if !self.is_surom() {
            return Ok(bank - 1);
        }
        if !(1..=30).contains(&bank) {
            return Err(format!("logical bank {bank} is outside 1..30"));
        }
        let index = bank - 1;
        Ok(if index < 15 { index } else { index + 1 })
    }
    pub fn needs_reset_tail(&self, bank: i32) -> bool {
        if bank <= 0 || !self.requires_per_bank_reset {
            return false;
        }
        match self.reset_replication {
            ResetReplication::EveryPhysical16KBank => true,
            ResetReplication::EveryPhysical32KHighBank => {
                if self.is_surom() {
                    self.logical_to_physical(bank).is_ok_and(|n| n & 1 != 0)
                } else {
                    bank & 1 == 0
                }
            }
            ResetReplication::CommonOnly => false,
        }
    }
}
