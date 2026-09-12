using System;

enum NesMapperKind
{
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

enum NesMirroringKind
{
    Horizontal,
    Vertical,
    FourScreen,
}

enum NesBoardKind
{
    Auto,
    Generic,
    Surom512,
}

enum NesPrgLayoutKind
{
    FixedTop16,
    DuplicatedCommonTop16,
    SuromOuter256FixedTop16,
}

enum NesBankSwitchKind
{
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

enum ResetVectorReplicationKind
{
    CommonOnly,
    EveryPhysical32KHighBank,
    EveryPhysical16KBank,
}

sealed class NesCartridgeProfile
{
    public NesMapperKind MapperKind { get; private set; }
    public NesBoardKind BoardKind { get; private set; }
    public string CliName { get; private set; }
    public string BoardCliName { get; private set; }
    public int MapperNumber { get; private set; }
    public NesPrgLayoutKind PrgLayout { get; private set; }
    public NesBankSwitchKind BankSwitchKind { get; private set; }
    public bool SupportsPrgBanking { get; private set; }
    public bool RequiresPowerOnBootStub { get; private set; }
    public bool RequiresPerBankResetStub { get; private set; }
    public bool HasVrc6Audio { get; private set; }
    public bool HasVrc7Audio { get; private set; }
    public bool HasFds { get; private set; }
    public int ExactPrgRomBytes { get; private set; }
    public int ExactChrRomBytes { get; private set; }
    public int ChrRamBytes { get; private set; }
    public int PrgRamBytes { get; private set; }
    public bool DefaultBatteryBacked { get; private set; }
    public int LogicalSwitchBankMin { get; private set; }
    public int LogicalSwitchBankMax { get; private set; }
    public int[] CommonPhysicalBanks { get; private set; }
    public bool UsesOuterPrgBank { get; private set; }
    public bool RequiresChrRam { get; private set; }
    public string HeaderKind { get; private set; }
    public ResetVectorReplicationKind ResetVectorReplication { get; private set; }

    NesCartridgeProfile(
        NesMapperKind mapperKind,
        NesBoardKind boardKind,
        string cliName,
        string boardCliName,
        int mapperNumber,
        NesPrgLayoutKind prgLayout,
        NesBankSwitchKind bankSwitchKind,
        bool supportsPrgBanking,
        bool requiresPowerOnBootStub,
        bool requiresPerBankResetStub,
        bool hasVrc6Audio,
        bool hasVrc7Audio,
        bool hasFds = false,
        int exactPrgRomBytes = 0,
        int exactChrRomBytes = -1,
        int chrRamBytes = 0,
        int prgRamBytes = 0x2000,
        bool defaultBatteryBacked = false,
        int logicalSwitchBankMin = 1,
        int logicalSwitchBankMax = int.MaxValue,
        int[] commonPhysicalBanks = null,
        bool usesOuterPrgBank = false,
        bool requiresChrRam = false,
        string headerKind = "ines1",
        ResetVectorReplicationKind resetVectorReplication = ResetVectorReplicationKind.CommonOnly)
    {
        MapperKind = mapperKind;
        BoardKind = boardKind;
        CliName = cliName;
        BoardCliName = boardCliName;
        MapperNumber = mapperNumber;
        PrgLayout = prgLayout;
        BankSwitchKind = bankSwitchKind;
        SupportsPrgBanking = supportsPrgBanking;
        RequiresPowerOnBootStub = requiresPowerOnBootStub;
        RequiresPerBankResetStub = requiresPerBankResetStub;
        HasVrc6Audio = hasVrc6Audio;
        HasVrc7Audio = hasVrc7Audio;
        HasFds = hasFds;
        ExactPrgRomBytes = exactPrgRomBytes;
        ExactChrRomBytes = exactChrRomBytes;
        ChrRamBytes = chrRamBytes;
        PrgRamBytes = prgRamBytes;
        DefaultBatteryBacked = defaultBatteryBacked;
        LogicalSwitchBankMin = logicalSwitchBankMin;
        LogicalSwitchBankMax = logicalSwitchBankMax;
        CommonPhysicalBanks = commonPhysicalBanks ?? new int[0];
        UsesOuterPrgBank = usesOuterPrgBank;
        RequiresChrRam = requiresChrRam;
        HeaderKind = headerKind;
        ResetVectorReplication = resetVectorReplication;
    }

    public bool UsesDuplicatedCommonBank => PrgLayout == NesPrgLayoutKind.DuplicatedCommonTop16;
    public bool SupportsBankedCode => SupportsPrgBanking || UsesDuplicatedCommonBank;
    public bool IsSurom512 => BoardKind == NesBoardKind.Surom512;

    public int LogicalToPhysicalBank(int logicalBank)
    {
        if (!IsSurom512) return logicalBank - 1;
        if (logicalBank < LogicalSwitchBankMin || logicalBank > LogicalSwitchBankMax)
            throw new ArgumentOutOfRangeException(nameof(logicalBank));
        int index = logicalBank - 1;
        return index < 15 ? index : index + 1;
    }

    public static NesCartridgeProfile ForMapper(NesMapperKind mapper)
    {
        return For(mapper, NesBoardKind.Auto);
    }

    public static NesCartridgeProfile For(NesMapperKind mapper, NesBoardKind board)
    {
        if (board == NesBoardKind.Surom512)
        {
            return new NesCartridgeProfile(
                NesMapperKind.Mmc1, NesBoardKind.Surom512, "mmc1", "surom512", 1,
                NesPrgLayoutKind.SuromOuter256FixedTop16, NesBankSwitchKind.Mmc1Surom,
                true, false, true, false, false, false,
                0x80000, 0, 0x2000, 0x2000, true, 1, 30,
                new[] { 15, 31 }, true, true, "ines1",
                ResetVectorReplicationKind.EveryPhysical32KHighBank);
        }

        NesBoardKind normalizedBoard = board == NesBoardKind.Auto ? NesBoardKind.Generic : board;
        switch (mapper)
        {
            case NesMapperKind.Uxrom:
                return new NesCartridgeProfile(NesMapperKind.Uxrom, normalizedBoard, "uxrom", "generic", 2, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.Uxrom, true, false, false, false, false);
            case NesMapperKind.Cnrom:
                return new NesCartridgeProfile(NesMapperKind.Cnrom, normalizedBoard, "cnrom", "generic", 3, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.None, false, false, false, false, false);
            case NesMapperKind.Axrom:
                return new NesCartridgeProfile(NesMapperKind.Axrom, normalizedBoard, "axrom", "generic", 7, NesPrgLayoutKind.DuplicatedCommonTop16, NesBankSwitchKind.Axrom, true, false, false, false, false);
            case NesMapperKind.Mmc1:
                return new NesCartridgeProfile(NesMapperKind.Mmc1, normalizedBoard, "mmc1", "generic", 1, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.Mmc1, true, false, true, false, false, false, 0, -1, 0, 0x2000, false, 1, int.MaxValue, null, false, false, "ines1", ResetVectorReplicationKind.EveryPhysical16KBank);
            case NesMapperKind.Mmc3:
                return new NesCartridgeProfile(NesMapperKind.Mmc3, normalizedBoard, "mmc3", "generic", 4, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.Mmc3, true, true, false, false, false);
            case NesMapperKind.Mmc5:
                return new NesCartridgeProfile(NesMapperKind.Mmc5, normalizedBoard, "mmc5", "generic", 5, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.Mmc5, true, true, false, false, false);
            case NesMapperKind.Vrc6:
                return new NesCartridgeProfile(NesMapperKind.Vrc6, normalizedBoard, "vrc6", "generic", 24, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.Vrc6, true, true, false, true, false);
            case NesMapperKind.Vrc7:
                return new NesCartridgeProfile(NesMapperKind.Vrc7, normalizedBoard, "vrc7", "generic", 85, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.Vrc7, true, true, false, false, true);
            case NesMapperKind.Fme7:
                return new NesCartridgeProfile(NesMapperKind.Fme7, normalizedBoard, "fme7", "generic", 69, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.Fme7, true, true, false, false, false);
            case NesMapperKind.Fds:
                return new NesCartridgeProfile(NesMapperKind.Fds, normalizedBoard, "fds", "generic", 20, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.Fds, false, true, false, false, false, true);
            case NesMapperKind.Nrom:
            default:
                return new NesCartridgeProfile(NesMapperKind.Nrom, normalizedBoard, "nrom", "generic", 0, NesPrgLayoutKind.FixedTop16, NesBankSwitchKind.None, false, false, false, false, false);
        }
    }
}

sealed class NesCartridgeOptions
{
    public NesMapperKind MapperKind { get; set; } = NesMapperKind.Nrom;
    public NesBoardKind BoardKind { get; set; } = NesBoardKind.Auto;
    public NesMirroringKind Mirroring { get; set; } = NesMirroringKind.Horizontal;
    bool? _batteryBacked;
    public bool BatteryBacked { get { return _batteryBacked ?? Profile.DefaultBatteryBacked; } set { _batteryBacked = value; } }
    public bool BatteryWasExplicitlySet => _batteryBacked.HasValue;

    public NesCartridgeProfile Profile => NesCartridgeProfile.For(MapperKind, BoardKind);

    public bool TrySetMapper(string raw, out string error)
    {
        string value = (raw ?? "").Trim().ToLowerInvariant();
        switch (value)
        {
            case "nrom":
            case "mapper0":
                MapperKind = NesMapperKind.Nrom;
                error = null;
                return true;
            case "uxrom":
            case "unrom":
            case "uorom":
            case "mapper2":
                MapperKind = NesMapperKind.Uxrom;
                error = null;
                return true;
            case "cnrom":
            case "mapper3":
                MapperKind = NesMapperKind.Cnrom;
                error = null;
                return true;
            case "axrom":
            case "anrom":
            case "mapper7":
                MapperKind = NesMapperKind.Axrom;
                error = null;
                return true;
            case "mmc1":
            case "sxrom":
            case "mapper1":
                MapperKind = NesMapperKind.Mmc1;
                error = null;
                return true;
            case "surom512":
                MapperKind = NesMapperKind.Mmc1;
                BoardKind = NesBoardKind.Surom512;
                error = null;
                return true;
            case "mmc3":
            case "txrom":
            case "mapper4":
                MapperKind = NesMapperKind.Mmc3;
                error = null;
                return true;
            case "mmc5":
            case "mapper5":
                MapperKind = NesMapperKind.Mmc5;
                error = null;
                return true;
            case "vrc6":
            case "mapper24":
            case "mapper26":
                MapperKind = NesMapperKind.Vrc6;
                error = null;
                return true;
            case "vrc7":
            case "mapper85":
                MapperKind = NesMapperKind.Vrc7;
                error = null;
                return true;
            case "fme7":
            case "sunsoft5b":
            case "sunsoft-5b":
            case "mapper69":
                MapperKind = NesMapperKind.Fme7;
                error = null;
                return true;
            case "fds":
            case "fds20":
            case "mapper20":
            case "famicom-disk-system":
                MapperKind = NesMapperKind.Fds;
                error = null;
                return true;
            default:
                error = "error: --mapper unsupported value: " + raw + " (try nrom|uxrom|cnrom|axrom|mmc1|mmc3|mmc5|vrc6|vrc7|fme7|fds)";
                return false;
        }
    }

    public bool TrySetBoard(string raw, out string error)
    {
        string value = (raw ?? "").Trim().ToLowerInvariant();
        switch (value)
        {
            case "auto": BoardKind = NesBoardKind.Auto; error = null; return true;
            case "generic": BoardKind = NesBoardKind.Generic; error = null; return true;
            case "surom":
            case "surom512":
            case "mmc1-surom512": BoardKind = NesBoardKind.Surom512; error = null; return true;
            default:
                error = "error: --board unsupported value: " + raw + " (try auto|generic|surom512)";
                return false;
        }
    }

    public bool ValidateCombination(out string error)
    {
        if (BoardKind == NesBoardKind.Surom512 && MapperKind != NesMapperKind.Mmc1)
        {
            error = "error KQFC2601 KQFC-SUROM-BOARD-CONFLICT: board 'surom512' requires mapper 'mmc1' (mapper 1).";
            return false;
        }
        error = null;
        return true;
    }

    public bool TrySetMirroring(string raw, out string error)
    {
        string value = (raw ?? "").Trim().ToLowerInvariant();
        switch (value)
        {
            case "h":
            case "horz":
            case "horizontal":
                Mirroring = NesMirroringKind.Horizontal;
                error = null;
                return true;
            case "v":
            case "vert":
            case "vertical":
                Mirroring = NesMirroringKind.Vertical;
                error = null;
                return true;
            case "4":
            case "4screen":
            case "four_screen":
            case "four-screen":
                Mirroring = NesMirroringKind.FourScreen;
                error = null;
                return true;
            default:
                error = "error: --mirroring unsupported value: " + raw + " (try horizontal|vertical|four-screen)";
                return false;
        }
    }
}
