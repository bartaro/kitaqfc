using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

sealed class BankedAssemblerCore
{
    const int HeaderSize = 16;
    const int BankSize = 0x4000;
    const int SwitchableCpuBase = 0x8000;
    const int CommonCpuBase = 0xC000;
    const int FdsSwitchableCpuBase = 0x6000;
    const int FdsCommonCpuBase = 0xA000;
    const int ResetStubReserve = 64;
    const int CommonBankLimitExclusive = 0x10000 - ResetStubReserve;
    const int SwitchableBankLimitExclusive = 0xC000;
    const int FdsCommonBankLimitExclusive = 0xE000 - ResetStubReserve;
    const int FdsSwitchableBankLimitExclusive = 0xA000;

    sealed class PlacedUnit
    {
        public int Index;
        public int RequestedBank;
        public bool HasFixedBank;
        public bool IsFunction;
        public string PrimaryName;
        public List<Expr> Nodes = new List<Expr>();
        public int ActualBank;
        public int StartCpu;
        public int EstimatedSize;
    }

    readonly Dictionary<string, int> _symbols = new Dictionary<string, int>(StringComparer.Ordinal);
    readonly Dictionary<string, FunctionSizeInfo> _functionSizes = new Dictionary<string, FunctionSizeInfo>(StringComparer.Ordinal);
    readonly Dictionary<string, ReadonlyDataInfo> _readonlyDataSizes = new Dictionary<string, ReadonlyDataInfo>(StringComparer.Ordinal);
    readonly Dictionary<int, byte[]> _switchableBankImages = new Dictionary<int, byte[]>();
    readonly byte[] _commonBankImage = new byte[BankSize];
    readonly string _branchLabelPrefix = "__kq_bank_asm_" + global::System.Guid.NewGuid().ToString("N");
    readonly NesCartridgeProfile _profile = Program.NesMapperProfile;

    int _switchableBankCount = 1;
    int _chrRomSize = 0x2000;
    byte[] _resetTail = new byte[0];
    bool _commonReplicaEqual = true;
    string _commonReplicaSha256 = "";

    bool UsesFdsPrgRamLayout
    {
        get { return _profile.HasFds && Program.FdsPrgRamLayoutEnabled; }
    }

    public AssemblerAnalysisReport LastReport { get; private set; } = new AssemblerAnalysisReport();

    List<FdsDiskFileMetadata> BuildFdsAutoOverlayMetadataRecords()
    {
        var list = new List<FdsDiskFileMetadata>();
        if (!UsesFdsPrgRamLayout || !Program.FdsAutoOverlayEnabled || _switchableBankCount <= 1)
            return list;

        int firstId = Program.FdsOverlayStartId;
        int overlayCount = _switchableBankCount - 1;
        if (firstId < 0 || firstId > 254 || firstId + overlayCount - 1 > 254)
        {
            Program.Error("error KQFC2507: FDS auto overlay id range {0}..{1} is outside 0..254. Change --fds-overlay-start-id.", firstId, firstId + overlayCount - 1);
            return list;
        }

        var used = new HashSet<int>((Program.FdsMetadata == null ? FdsDiskMetadata.Empty : Program.FdsMetadata).Files.Select(f => f.Id));
        for (int bank = 2; bank <= _switchableBankCount; bank++)
        {
            int id = firstId + (bank - 2);
            if (!used.Add(id))
            {
                Program.Error("error KQFC2508: FDS auto overlay bank {0} wants file id {1}, but --fds-meta already uses it. Change --fds-overlay-start-id or the manifest id.", bank, id);
                return list;
            }
            list.Add(new FdsDiskFileMetadata
            {
                Id = id,
                Size = BankSize,
                LoadAddress = FdsSwitchableCpuBase,
                FileType = 0,
                Overlay = true,
                Boot = false,
                Name = BuildFdsOverlayFileName(bank),
                Side = 0,
                FileNumber = -1,
            });
        }

        int mergedCount = ((Program.FdsMetadata == null ? FdsDiskMetadata.Empty : Program.FdsMetadata).Files.Count + list.Count);
        if (mergedCount > 42)
            Program.Error("error KQFC2509: FDS runtime metadata table supports up to 42 files; user manifest + auto overlays = {0}.", mergedCount);
        return list;
    }

    bool ApplyFdsRuntimeMetadataTable(List<PlacedUnit> units)
    {
        if (!UsesFdsPrgRamLayout) return false;
        var overlayMeta = BuildFdsAutoOverlayMetadataRecords();
        if (Program.ErrorCount > 0) return false;
        byte[] table = (Program.FdsMetadata ?? FdsDiskMetadata.Empty).BuildRuntimeTable(overlayMeta);
        bool changed = false;

        foreach (var unit in units ?? new List<PlacedUnit>())
        {
            for (int i = 0; i < unit.Nodes.Count; i++)
            {
                string name;
                byte[] oldBytes;
                if (unit.Nodes[i].Match(Tag.ReadonlyData, out name, out oldBytes) && name == "__kq_fds_metadata_table")
                {
                    if (!ByteArrayEquals(oldBytes, table))
                    {
                        unit.Nodes[i] = Expr.Make(Tag.ReadonlyData, name, table).WithSource(unit.Nodes[i].Source);
                        changed = true;
                    }
                }
            }
        }

        return changed;
    }


    bool ApplyFdsOverlayFunctionTable(List<PlacedUnit> units)
    {
        if (!UsesFdsPrgRamLayout || !Program.FdsOverlayFunctionTableEnabled) return false;

        var overlayFunctions = (units ?? new List<PlacedUnit>())
            .Where(u => u != null && u.IsFunction && !string.IsNullOrEmpty(u.PrimaryName) && u.ActualBank >= 2)
            .OrderBy(u => u.ActualBank)
            .ThenBy(u => u.StartCpu)
            .ThenBy(u => u.PrimaryName, StringComparer.Ordinal)
            .ToList();

        if (overlayFunctions.Count > 255)
            Program.Error("error KQFC2511: FDS overlay function table supports up to 255 functions; found {0}.", overlayFunctions.Count);

        var bytes = new List<byte>();
        bytes.Add((byte)Math.Min(overlayFunctions.Count, 255));
        foreach (var u in overlayFunctions.Take(255))
        {
            int fileId = Program.FdsOverlayStartId + (u.ActualBank - 2);
            int hash = StableNameHash16(u.PrimaryName);
            bytes.Add((byte)(u.ActualBank & 0xFF));
            bytes.Add((byte)(fileId & 0xFF));
            bytes.Add((byte)(u.StartCpu & 0xFF));
            bytes.Add((byte)((u.StartCpu >> 8) & 0xFF));
            bytes.Add((byte)(u.EstimatedSize & 0xFF));
            bytes.Add((byte)((u.EstimatedSize >> 8) & 0xFF));
            bytes.Add((byte)(hash & 0xFF));
            bytes.Add((byte)((hash >> 8) & 0xFF));
        }

        bool changed = false;
        byte[] next = bytes.ToArray();
        foreach (var unit in units ?? new List<PlacedUnit>())
        {
            for (int i = 0; i < unit.Nodes.Count; i++)
            {
                string name;
                byte[] oldBytes;
                if (unit.Nodes[i].Match(Tag.ReadonlyData, out name, out oldBytes) && name == "__kq_fds_overlay_function_table")
                {
                    if (!ByteArrayEquals(oldBytes, next))
                    {
                        unit.Nodes[i] = Expr.Make(Tag.ReadonlyData, name, next).WithSource(unit.Nodes[i].Source);
                        changed = true;
                    }
                }
            }
        }
        return changed;
    }

    static int StableNameHash16(string name)
    {
        unchecked
        {
            int h = 0x811C;
            foreach (char ch in name ?? "")
            {
                h ^= (byte)ch;
                h *= 0x0101;
            }
            return h & 0xFFFF;
        }
    }

    List<FdsAutoOverlayFile> BuildFdsAutoOverlayDiskFiles()
    {
        var list = new List<FdsAutoOverlayFile>();
        if (!UsesFdsPrgRamLayout || !Program.FdsAutoOverlayEnabled || _switchableBankCount <= 1)
            return list;

        var overlayMeta = BuildFdsAutoOverlayMetadataRecords();
        if (Program.ErrorCount > 0) return list;

        foreach (var meta in overlayMeta)
        {
            int bank = 2 + (meta.Id - Program.FdsOverlayStartId);
            byte[] image;
            if (!_switchableBankImages.TryGetValue(bank, out image) || image == null)
            {
                Program.Error("error KQFC2510: internal error: missing switchable image for FDS overlay bank {0}.", bank);
                return list;
            }

            byte[] data = CloneAndMaybeTrimFdsOverlay(image);
            list.Add(new FdsAutoOverlayFile
            {
                Id = meta.Id,
                Side = meta.Side,
                Number = meta.FileNumber,
                Name = meta.Name,
                LoadAddress = meta.LoadAddress,
                FileType = meta.FileType,
                Boot = meta.Boot,
                Overlay = true,
                SourceBank = bank,
                Data = data,
            });
        }

        return list;
    }

    byte[] CloneAndMaybeTrimFdsOverlay(byte[] image)
    {
        if (image == null) return new byte[0];
        int len = image.Length;
        if (Program.FdsOverlayTrimTrailingFill)
        {
            while (len > 0 && image[len - 1] == 0xFF)
                len--;
            len = Math.Max(1, len);
        }
        byte[] data = new byte[len];
        Buffer.BlockCopy(image, 0, data, 0, Math.Min(len, image.Length));
        return data;
    }

    string BuildFdsOverlayFileName(int bank)
    {
        string prefix = Program.FdsOverlayNamePrefix ?? "KQFB";
        prefix = new string(prefix.Where(ch => ch >= 0x21 && ch <= 0x7E).ToArray()).ToUpperInvariant();
        if (prefix.Length == 0) prefix = "KQFB";
        string suffix = bank.ToString("D3");
        string name = prefix + suffix;
        if (name.Length > 8) name = name.Substring(0, 8);
        while (name.Length < 8) name += " ";
        return name;
    }

    static bool ByteArrayEquals(byte[] a, byte[] b)
    {
        if (object.ReferenceEquals(a, b)) return true;
        if (a == null || b == null || a.Length != b.Length) return false;
        for (int i = 0; i < a.Length; i++)
            if (a[i] != b[i]) return false;
        return true;
    }

    public string Run(IReadOnlyList<Expr> assembly, string outputFilename)
    {
        var units = ParseUnits(assembly ?? Array.Empty<Expr>());
        if (Program.ErrorCount > 0) return outputFilename;

        bool layoutStable = false;
        for (int pass = 0; pass < 8; pass++)
        {
            layoutStable = AssignActualBanks(units);
            if (Program.ErrorCount > 0) return outputFilename;

            if (ApplyFdsRuntimeMetadataTable(units))
                layoutStable = false;
            if (Program.ErrorCount > 0) return outputFilename;

            var symbols = BuildAddressMap(units);
            if (Program.ErrorCount > 0) return outputFilename;

            if (ApplyFdsOverlayFunctionTable(units))
                layoutStable = false;
            if (Program.ErrorCount > 0) return outputFilename;

            bool expanded = ExpandLongBranches(units, symbols);
            if (!expanded && layoutStable)
                break;
        }

        _symbols.Clear();
        foreach (var pair in BuildAddressMap(units))
            _symbols[pair.Key] = pair.Value;
        if (Program.ErrorCount > 0) return outputFilename;

        InitializeBankImages();
        Pass2(units);
        if (Program.ErrorCount > 0) return outputFilename;

        WriteResetAndVectors();
        if (Program.ErrorCount > 0) return outputFilename;

        byte[] prg = BuildPrgRom();
        if (Program.ErrorCount > 0) return outputFilename;
        byte[] chr = LoadChrRom();

        bool fdsPrimary = Program.IsFdsPrimaryOutput(outputFilename);
        bool fdsSidecar = Program.ShouldEmitFdsSidecar(outputFilename);
        if ((fdsPrimary || fdsSidecar) && !_profile.HasFds)
        {
            Program.Error("error KQFC2500: FDS image output requires --mapper=fds / mapper20; current mapper is {0}.", _profile.CliName);
            return outputFilename;
        }

        byte[] header = BuildHeader(prg.Length, chr.Length);
        byte[] rom = new byte[HeaderSize + prg.Length + chr.Length];
        Buffer.BlockCopy(header, 0, rom, 0, HeaderSize);
        Buffer.BlockCopy(prg, 0, rom, HeaderSize, prg.Length);
        Buffer.BlockCopy(chr, 0, rom, HeaderSize + prg.Length, chr.Length);

        var autoOverlayFiles = BuildFdsAutoOverlayDiskFiles();
        if (Program.ErrorCount > 0) return outputFilename;

        if (fdsPrimary)
        {
            var fds = FdsDiskImageBuilder.BuildFromCompiledImage(prg, chr, Program.FdsMetadata, Program.FdsImageHeaderEnabled, Program.FdsGameCode, Program.FdsLicenseBypassEnabled, UsesFdsPrgRamLayout, autoOverlayFiles);
            if (Program.ErrorCount > 0) return outputFilename;
            File.WriteAllBytes(outputFilename, fds.Image);
            EmitFdsBuildWarnings(outputFilename, fds);
        }
        else
        {
            File.WriteAllBytes(outputFilename, rom);
            if (fdsSidecar)
            {
                string fdsPath = Program.ResolveFdsImageOutputPath(outputFilename);
                var fds = FdsDiskImageBuilder.BuildFromCompiledImage(prg, chr, Program.FdsMetadata, Program.FdsImageHeaderEnabled, Program.FdsGameCode, Program.FdsLicenseBypassEnabled, UsesFdsPrgRamLayout, autoOverlayFiles);
                if (Program.ErrorCount > 0) return outputFilename;
                File.WriteAllBytes(fdsPath, fds.Image);
                EmitFdsBuildWarnings(fdsPath, fds);
            }
        }

        LastReport = BuildReport(prg.Length, chr.Length);
        return outputFilename;
    }

    void EmitFdsBuildWarnings(string path, FdsDiskImageBuildResult result)
    {
        if (result == null) return;
        Console.WriteLine("[fds] image: " + path + " (" + result.SideCount + " side(s), " + result.FileCount + " file(s))");
        foreach (var warning in result.Warnings ?? new List<string>())
            Program.Warning("warning: " + warning);
    }

    List<PlacedUnit> ParseUnits(IReadOnlyList<Expr> assembly)
    {
        var units = new List<PlacedUnit>();
        var pendingPrefix = new List<Expr>();
        PlacedUnit current = null;
        int placementBank = 1;
        bool placementFixed = false;
        int nextIndex = 0;

        Action finalize = () =>
        {
            if (current == null) return;
            current.EstimatedSize = EstimateUnitSize(current, null);
            units.Add(current);
            current = null;
        };

        Func<bool, string, PlacedUnit> createUnit = (isFunction, name) =>
        {
            var unit = new PlacedUnit
            {
                Index = nextIndex++,
                RequestedBank = placementBank,
                HasFixedBank = placementFixed || placementBank == 0,
                IsFunction = isFunction,
                PrimaryName = name,
            };
            if (pendingPrefix.Count > 0)
            {
                unit.Nodes.AddRange(pendingPrefix);
                pendingPrefix.Clear();
            }
            return unit;
        };

        foreach (var e in assembly ?? Array.Empty<Expr>())
        {
            if (e.Match(Tag.PrgBank, out int nextBank, out int fixedFlag))
            {
                finalize();
                placementBank = Math.Max(0, nextBank);
                placementFixed = fixedFlag != 0;
                continue;
            }

            if (e.Match(Tag.Comment, out string pendingComment) && current == null)
            {
                pendingPrefix.Add(e);
                continue;
            }

            if (e.Match(Tag.Function, out string functionName))
            {
                finalize();
                current = createUnit(true, functionName);
                current.Nodes.Add(e);
                continue;
            }

            byte[] ignoredBytes;
            if (e.Match(Tag.ReadonlyData, out string rdName, out ignoredBytes))
            {
                finalize();
                current = createUnit(false, rdName);
                current.Nodes.Add(e);
                finalize();
                continue;
            }

            // Tag.Word can be used either as a standalone pointer word or as inline
            // data after an FDS BIOS JSR direct-pointer call. Keep it in the current
            // unit so direct-pointer return-address semantics remain intact.
            if (e.Match(Tag.Word, out rdName))
            {
                if (current == null)
                    current = createUnit(false, rdName);
                current.Nodes.Add(e);
                continue;
            }

            if (current == null)
                current = createUnit(false, null);
            current.Nodes.Add(e);
        }

        finalize();
        return units;
    }

    bool AssignActualBanks(List<PlacedUnit> units)
    {
        var bankUsage = new Dictionary<int, int>();
        bool stable = true;

        foreach (var unit in units ?? new List<PlacedUnit>())
        {
            unit.EstimatedSize = EstimateUnitSize(unit, _symbols);
            if (unit.RequestedBank == 0)
            {
                unit.ActualBank = 0;
                AddBankUsage(bankUsage, 0, unit.EstimatedSize, unit);
            }
        }

        foreach (var unit in (units ?? new List<PlacedUnit>()).Where(x => x.RequestedBank > 0 && x.HasFixedBank))
        {
            unit.ActualBank = Math.Max(1, unit.RequestedBank);
            if (_profile.IsSurom512 && unit.ActualBank > _profile.LogicalSwitchBankMax)
            {
                Program.Error("error KQFC2603 KQFC-SUROM-BANK-RANGE: logical bank {0} is outside the supported range 1..30.", unit.ActualBank);
                return stable;
            }
            AddBankUsage(bankUsage, unit.ActualBank, unit.EstimatedSize, unit);
        }

        foreach (var unit in (units ?? new List<PlacedUnit>()).Where(x => x.RequestedBank > 0 && !x.HasFixedBank).OrderBy(x => x.Index))
        {
            int bank = Math.Max(1, unit.RequestedBank);
            while (true)
            {
                if (_profile.IsSurom512 && bank > _profile.LogicalSwitchBankMax)
                {
                    Program.Error("error KQFC2602 KQFC-SUROM-PRG-OVERFLOW: automatic placement exhausted logical banks 1..30 while placing {0}.", string.IsNullOrEmpty(unit.PrimaryName) ? "<anonymous>" : unit.PrimaryName);
                    unit.ActualBank = _profile.LogicalSwitchBankMax;
                    break;
                }
                int used = 0;
                bankUsage.TryGetValue(bank, out used);
                if (used + unit.EstimatedSize <= GetBankCapacity(bank))
                {
                    if (unit.ActualBank != 0 && unit.ActualBank != bank) stable = false;
                    unit.ActualBank = bank;
                    bankUsage[bank] = used + unit.EstimatedSize;
                    break;
                }
                bank++;
            }
        }

        int maxSwitchable = Math.Max(1, units.Where(x => x.ActualBank > 0).Select(x => x.ActualBank).DefaultIfEmpty(1).Max());
        if (!_profile.SupportsPrgBanking && maxSwitchable > 1)
        {
            Program.Error("error KQ0000: mapper '{0}' does not support banked PRG code beyond common bank 0 and switchable bank 1.", _profile.CliName);
            return stable;
        }

        if (_switchableBankCount != maxSwitchable) stable = false;
        _switchableBankCount = maxSwitchable;
        return stable;
    }

    void AddBankUsage(Dictionary<int, int> bankUsage, int bank, int amount, PlacedUnit unit)
    {
        int used = 0;
        bankUsage.TryGetValue(bank, out used);
        used += amount;
        if (used > GetBankCapacity(bank))
        {
            Program.Error("error KQ0000: NES assembler bank overflow in bank {0} while placing {1}.", bank, string.IsNullOrEmpty(unit.PrimaryName) ? "<anonymous>" : unit.PrimaryName);
            return;
        }
        bankUsage[bank] = used;
    }

    int EstimateUnitSize(PlacedUnit unit, IReadOnlyDictionary<string, int> symbols)
    {
        int bank = unit == null ? 1 : (unit.ActualBank != 0 ? unit.ActualBank : unit.RequestedBank == 0 ? 0 : 1);
        int pc = GetCpuBase(bank);
        int startPc = pc;
        foreach (var e in unit?.Nodes ?? new List<Expr>())
            pc = AdvancePcForEstimate(bank, pc, e, symbols);
        return Math.Max(0, pc - startPc);
    }

    int AdvancePcForEstimate(int bank, int pc, Expr e, IReadOnlyDictionary<string, int> symbols)
    {
        if (e == null) return pc;
        string ignoredLabel;
        if (e.Match(Tag.Function, out ignoredLabel) || e.Match(Tag.Label, out ignoredLabel) || e.Match(Tag.Comment, out ignoredLabel) || e.Match(Tag.Section, out ignoredLabel))
            return pc;
        if (e.Match(Tag.SkipTo, out int skip))
            return skip == 0 ? Math.Max(pc, GetCpuBase(bank)) : skip;
        if (e.Match(Tag.Align, out int align))
            return (pc + (align - 1)) & ~(align - 1);
        string ignoredName;
        if (e.Match(Tag.ReadonlyData, out ignoredName, out byte[] bytes))
            return pc + (bytes == null ? 0 : bytes.Length);
        if (e.Match(Tag.Word, out ignoredName))
            return pc + 2;
        if (e.Match(Tag.Asm, out string mnemonic, out AsmOperand operand))
            return pc + GetEstimatedInstructionSize(NormalizeMnemonic(mnemonic), operand, symbols, pc);
        return pc;
    }

    Dictionary<string, int> BuildAddressMap(List<PlacedUnit> units)
    {
        var previous = new Dictionary<string, int>(StringComparer.Ordinal);
        for (int pass = 0; pass < 8; pass++)
        {
            var symbols = new Dictionary<string, int>(StringComparer.Ordinal);
            var lookup = new Dictionary<string, int>(previous, StringComparer.Ordinal);

            foreach (var group in OrderedUnitsByBank(units))
            {
                int bank = group.Key;
                int pc = GetCpuBase(bank);
                foreach (var unit in group)
                {
                    unit.StartCpu = pc;
                    foreach (var e in unit.Nodes)
                    {
                        if (e.Match(Tag.Function, out string label) || e.Match(Tag.Label, out label))
                        {
                            if (!string.IsNullOrEmpty(label) && !symbols.ContainsKey(label))
                            {
                                symbols[label] = pc;
                                lookup[label] = pc;
                            }
                            continue;
                        }
                        if (e.Match(Tag.Comment, out label) || e.Match(Tag.Section, out label))
                            continue;
                        if (e.Match(Tag.SkipTo, out int skip))
                        {
                            pc = skip == 0 ? Math.Max(pc, GetCpuBase(bank)) : skip;
                            continue;
                        }
                        if (e.Match(Tag.Align, out int align))
                        {
                            pc = (pc + (align - 1)) & ~(align - 1);
                            continue;
                        }
                        if (e.Match(Tag.ReadonlyData, out label, out byte[] bytes))
                        {
                            if (!string.IsNullOrEmpty(label) && !symbols.ContainsKey(label))
                            {
                                symbols[label] = pc;
                                lookup[label] = pc;
                            }
                            pc += bytes == null ? 0 : bytes.Length;
                            continue;
                        }
                        if (e.Match(Tag.Word, out label))
                        {
                            pc += 2;
                            continue;
                        }
                        if (e.Match(Tag.Asm, out string mnemonic, out AsmOperand operand))
                        {
                            pc += GetEstimatedInstructionSize(NormalizeMnemonic(mnemonic), operand, lookup, pc);
                            continue;
                        }
                    }

                    ValidatePc(bank, pc, FilePosition.Unknown, unit.PrimaryName ?? "layout");
                }
            }

            if (SymbolMapsEqual(previous, symbols))
                return symbols;
            previous = symbols;
        }

        return previous;
    }

    bool ExpandLongBranches(List<PlacedUnit> units, IReadOnlyDictionary<string, int> symbols)
    {
        bool changed = false;
        int branchCounter = 0;

        foreach (var unit in units ?? new List<PlacedUnit>())
        {
            int pc = unit.StartCpu == 0 ? GetCpuBase(unit.ActualBank) : unit.StartCpu;
            var expanded = new List<Expr>(unit.Nodes.Count);
            bool unitChanged = false;
            foreach (var e in unit.Nodes)
            {
                string ignoredLabel;
                if (e.Match(Tag.Function, out ignoredLabel) || e.Match(Tag.Label, out ignoredLabel) || e.Match(Tag.Comment, out ignoredLabel) || e.Match(Tag.Section, out ignoredLabel))
                {
                    expanded.Add(e);
                    continue;
                }
                if (e.Match(Tag.SkipTo, out int skip))
                {
                    expanded.Add(e);
                    pc = skip == 0 ? Math.Max(pc, GetCpuBase(unit.ActualBank)) : skip;
                    continue;
                }
                if (e.Match(Tag.Align, out int align))
                {
                    expanded.Add(e);
                    pc = (pc + (align - 1)) & ~(align - 1);
                    continue;
                }
                string ignoredName;
                if (e.Match(Tag.ReadonlyData, out ignoredName, out byte[] bytes))
                {
                    expanded.Add(e);
                    pc += bytes == null ? 0 : bytes.Length;
                    continue;
                }
                if (e.Match(Tag.Word, out ignoredName))
                {
                    expanded.Add(e);
                    pc += 2;
                    continue;
                }
                if (e.Match(Tag.Asm, out string mnemonic, out AsmOperand operand))
                {
                    if (TryExpandLongBranch(e, mnemonic, operand, symbols, pc, expanded, ref branchCounter))
                    {
                        unitChanged = true;
                        changed = true;
                        pc += 5;
                        continue;
                    }

                    expanded.Add(e);
                    pc += GetEstimatedInstructionSize(NormalizeMnemonic(mnemonic), operand, symbols, pc);
                    continue;
                }

                expanded.Add(e);
            }

            if (unitChanged)
                unit.Nodes = expanded;
        }

        return changed;
    }

    bool TryExpandLongBranch(Expr e, string mnemonic, AsmOperand operand, IReadOnlyDictionary<string, int> symbols, int pc, List<Expr> expanded, ref int branchCounter)
    {
        if (operand == null || operand.Mode != AddressMode.Relative || operand.Modifier != ImmediateModifier.None || !operand.Base.HasValue)
            return false;

        if (!symbols.TryGetValue(operand.Base.Value, out int target))
            return false;
        target += operand.Offset;
        int delta = target - (pc + 2);
        if (delta >= -128 && delta <= 127)
            return false;

        string inverseMnemonic;
        switch (NormalizeMnemonic(mnemonic))
        {
            case "BEQ": inverseMnemonic = "BNE"; break;
            case "BNE": inverseMnemonic = "BEQ"; break;
            case "BCC": inverseMnemonic = "BCS"; break;
            case "BCS": inverseMnemonic = "BCC"; break;
            case "BMI": inverseMnemonic = "BPL"; break;
            case "BPL": inverseMnemonic = "BMI"; break;
            case "BVS": inverseMnemonic = "BVC"; break;
            case "BVC": inverseMnemonic = "BVS"; break;
            default:
                return false;
        }

        string skipLabel = string.Format("{0}_skip_{1}", _branchLabelPrefix, ++branchCounter);
        expanded.Add(Expr.Make(Tag.Asm, inverseMnemonic, new AsmOperand(skipLabel, AddressMode.Relative)).WithSource(e.Source));
        expanded.Add(Expr.Make(Tag.Asm, "JMP", operand.WithMode(AddressMode.Absolute)).WithSource(e.Source));
        expanded.Add(Expr.Make(Tag.Label, skipLabel).WithSource(e.Source));
        return true;
    }

    void InitializeBankImages()
    {
        FillWithFF(_commonBankImage);
        _switchableBankImages.Clear();
        for (int bank = 1; bank <= _switchableBankCount; bank++)
        {
            var image = new byte[BankSize];
            FillWithFF(image);
            _switchableBankImages[bank] = image;
        }
    }

    bool TryResolveWordValue(string label, out int value)
    {
        value = 0;
        if (string.IsNullOrEmpty(label))
            return false;
        string s = label.Trim();
        if (s.StartsWith("$", StringComparison.Ordinal))
        {
            return int.TryParse(s.Substring(1), System.Globalization.NumberStyles.HexNumber, System.Globalization.CultureInfo.InvariantCulture, out value);
        }
        if (s.StartsWith("0x", StringComparison.OrdinalIgnoreCase))
        {
            return int.TryParse(s.Substring(2), System.Globalization.NumberStyles.HexNumber, System.Globalization.CultureInfo.InvariantCulture, out value);
        }
        return _symbols.TryGetValue(label, out value);
    }

    void Pass2(List<PlacedUnit> units)
    {
        _functionSizes.Clear();
        _readonlyDataSizes.Clear();

        foreach (var group in OrderedUnitsByBank(units))
        {
            int bank = group.Key;
            foreach (var unit in group)
            {
                int pc = unit.StartCpu;
                int functionStartPc = pc;
                foreach (var e in unit.Nodes)
                {
                    string ignoredLabel;
                    if (e.Match(Tag.Function, out ignoredLabel) || e.Match(Tag.Label, out ignoredLabel) || e.Match(Tag.Comment, out ignoredLabel) || e.Match(Tag.Section, out ignoredLabel))
                        continue;
                    if (e.Match(Tag.SkipTo, out int skip))
                    {
                        pc = skip == 0 ? Math.Max(pc, GetCpuBase(bank)) : skip;
                        ValidatePc(bank, pc, e.Source, "skip");
                        continue;
                    }
                    if (e.Match(Tag.Align, out int align))
                    {
                        pc = (pc + (align - 1)) & ~(align - 1);
                        ValidatePc(bank, pc, e.Source, "align");
                        continue;
                    }
                    string ignoredName;
                    if (e.Match(Tag.ReadonlyData, out ignoredName, out byte[] bytes))
                    {
                        int dataStartPc = pc;
                        int dataSize = bytes == null ? 0 : bytes.Length;
                        WriteBytes(bank, pc, bytes ?? Array.Empty<byte>(), e.Source);
                        pc += dataSize;
                        if (!string.IsNullOrEmpty(ignoredName))
                        {
                            _readonlyDataSizes[ignoredName] = new ReadonlyDataInfo
                            {
                                Name = ignoredName,
                                Bank = unit.ActualBank,
                                RequestedBank = unit.RequestedBank,
                                HasFixedBank = unit.HasFixedBank,
                                StartFileOffset = GetFileOffset(unit.ActualBank, dataStartPc),
                                EndFileOffset = GetFileOffset(unit.ActualBank, pc),
                                SizeBytes = dataSize,
                                CpuAddress = dataStartPc,
                            };
                        }
                        continue;
                    }
                    if (e.Match(Tag.Word, out string label))
                    {
                        int value;
                        if (!TryResolveWordValue(label, out value))
                        {
                            Program.Error("error KQ0000: NES assembler unresolved symbol in $word: {0}", label);
                            return;
                        }
                        WriteWord(bank, pc, value, e.Source);
                        pc += 2;
                        continue;
                    }
                    if (e.Match(Tag.Asm, out string mnemonic, out AsmOperand operand))
                    {
                        byte[] encoded = EncodeInstruction(NormalizeMnemonic(mnemonic), operand ?? AsmOperand.Implicit, pc, e.Source);
                        if (Program.ErrorCount > 0) return;
                        WriteBytes(bank, pc, encoded, e.Source);
                        pc += encoded.Length;
                        continue;
                    }

                    Program.Error("error KQ0000: NES assembler encountered an unsupported assembly node: {0}", e.Tag);
                    return;
                }

                if (unit.IsFunction && !string.IsNullOrEmpty(unit.PrimaryName))
                {
                    _functionSizes[unit.PrimaryName] = new FunctionSizeInfo
                    {
                        Name = unit.PrimaryName,
                        StartFileOffset = GetFileOffset(unit.ActualBank, functionStartPc),
                        EndFileOffset = GetFileOffset(unit.ActualBank, pc),
                        SizeBytes = pc - functionStartPc,
                        Bank = unit.ActualBank,
                        CpuAddress = functionStartPc,
                    };
                }
            }
        }
    }

    void WriteResetAndVectors()
    {
        int resetTarget = ResolveVectorTarget("__nes_reset", "__kq_reset_stub", "main", "__kq_hang_loop");
        int nmiTarget = ResolveVectorTarget("__nes_nmi", "__kq_nmi_default", "__kq_hang_loop");
        int irqTarget = ResolveVectorTarget("__nes_irq", "__kq_irq_default", "__kq_hang_loop");

        byte[] tail = BuildResetTail(resetTarget, nmiTarget, irqTarget);
        _resetTail = tail;
        int tailOffset = BankSize - ResetStubReserve;
        Buffer.BlockCopy(tail, 0, _commonBankImage, tailOffset, tail.Length);

        if (_profile.RequiresPerBankResetStub)
        {
            foreach (var pair in _switchableBankImages)
            {
                if (RequiresResetTailForLogicalBank(pair.Key))
                    Buffer.BlockCopy(tail, 0, pair.Value, tailOffset, tail.Length);
            }
        }
    }

    byte[] BuildResetTail(int resetTarget, int nmiTarget, int irqTarget)
    {
        if (UsesFdsPrgRamLayout)
            return BuildFdsResetTail(resetTarget, nmiTarget, irqTarget);

        var tail = new byte[ResetStubReserve];
        for (int i = 0; i < tail.Length; i++) tail[i] = 0xEA;

        var code = new List<byte>();
        switch (_profile.BankSwitchKind)
        {
            case NesBankSwitchKind.Mmc1Surom:
                code.Add(0x78); // SEI
                code.Add(0xA9); code.Add(0x80);
                code.Add(0x8D); code.Add(0x00); code.Add(0x80); // reset shift register
                AppendMmc1SerialWrite(code, 0x8000, GetMmc1ControlValue());
                AppendMmc1SerialWrite(code, 0xA000, 0x00); // outer 0
                AppendMmc1SerialWrite(code, 0xE000, 0x00); // inner 0
                WriteJmp(code, resetTarget);
                break;

            case NesBankSwitchKind.Mmc1:
                code.Add(0x78);
                code.Add(0xA9); code.Add(0x80);
                code.Add(0x8D); code.Add(0x00); code.Add(0x80);
                WriteJmp(code, resetTarget);
                break;

            case NesBankSwitchKind.Mmc3:
                code.Add(0x78);
                code.Add(0xA9); code.Add(0x06);
                code.Add(0x8D); code.Add(0x00); code.Add(0x80);
                WriteJmp(code, resetTarget);
                break;

            case NesBankSwitchKind.Mmc5:
                code.Add(0x78);
                code.Add(0xA9); code.Add(0x03);
                code.Add(0x8D); code.Add(0x00); code.Add(0x51);
                code.Add(0xA9); code.Add((byte)(0x80 | (GetCommonSecondLast8kIndex() & 0x7F)));
                code.Add(0x8D); code.Add(0x16); code.Add(0x51);
                code.Add(0xA9); code.Add((byte)(0x80 | (GetCommonLast8kIndex() & 0x7F)));
                code.Add(0x8D); code.Add(0x17); code.Add(0x51);
                WriteJmp(code, resetTarget);
                break;

            case NesBankSwitchKind.Vrc6:
                code.Add(0x78);
                code.Add(0xA9); code.Add((byte)(GetCommonSecondLast8kIndex() & 0xFF));
                code.Add(0x8D); code.Add(0x00); code.Add(0xC0);
                WriteJmp(code, resetTarget);
                break;

            case NesBankSwitchKind.Vrc7:
                code.Add(0x78);
                code.Add(0xA9); code.Add((byte)(GetCommonSecondLast8kIndex() & 0xFF));
                code.Add(0x8D); code.Add(0x00); code.Add(0x90);
                WriteJmp(code, resetTarget);
                break;

            case NesBankSwitchKind.Fme7:
                code.Add(0x78);
                code.Add(0xA9); code.Add(0x0B);
                code.Add(0x8D); code.Add(0x00); code.Add(0x80);
                code.Add(0xA9); code.Add((byte)(GetCommonSecondLast8kIndex() & 0xFF));
                code.Add(0x8D); code.Add(0x00); code.Add(0xA0);
                WriteJmp(code, resetTarget);
                break;

            default:
                WriteJmp(code, resetTarget);
                break;
        }

        if (code.Count > ResetStubReserve - 6)
        {
            Program.Error("error KQ0000: NES boot stub for mapper '{0}' exceeded reserved tail space.", _profile.CliName);
            return tail;
        }

        Buffer.BlockCopy(code.ToArray(), 0, tail, 0, code.Count);
        WriteWordToArray(tail, ResetStubReserve - 6, nmiTarget);
        WriteWordToArray(tail, ResetStubReserve - 4, GetResetStubCpuAddress());
        WriteWordToArray(tail, ResetStubReserve - 2, irqTarget);
        return tail;
    }

    static void AppendMmc1SerialWrite(List<byte> code, int address, int value)
    {
        code.Add(0xA9); code.Add((byte)(value & 0x1F)); // LDA #value
        code.Add(0xA2); code.Add(0x05);                // LDX #5
        code.Add(0x48);                                // loop: PHA
        code.Add(0x29); code.Add(0x01);                // AND #1
        code.Add(0x8D); code.Add((byte)(address & 0xFF)); code.Add((byte)((address >> 8) & 0xFF));
        code.Add(0x68);                                // PLA
        code.Add(0x4A);                                // LSR A
        code.Add(0xCA);                                // DEX
        code.Add(0xD0); code.Add(0xF5);                // BNE loop (-11, back to PHA)
    }

    static int GetMmc1ControlValue()
    {
        int mirroring = Program.NesCartridge.Mirroring == NesMirroringKind.Vertical ? 2 : 3;
        return 0x0C | mirroring;
    }

    byte[] BuildFdsResetTail(int resetTarget, int nmiTarget, int irqTarget)
    {
        var tail = new byte[ResetStubReserve];
        for (int i = 0; i < tail.Length; i++) tail[i] = 0xEA;

        var code = new List<byte>();
        int bypassAddress = GetResetStubCpuAddress();
        int resetEntryAddress = bypassAddress;

        if (Program.FdsLicenseBypassEnabled)
        {
            // NMI #3 approval-screen bypass stub. The FDS BIOS will use ($DFFA)
            // when $0100 is $C0 during boot. The disk builder emits a tiny boot
            // file that writes $80 to PPUCTRL, causing this NMI before the BIOS
            // performs the approval nametable check.
            code.Add(0xA9); code.Add(0x00);              // LDA #$00
            code.Add(0x8D); code.Add(0x00); code.Add(0x20); // STA $2000
            code.Add(0x85); code.Add(0xFF);              // STA $FF (BIOS PPUCTRL mirror)
            code.Add(0xA9); code.Add((byte)(nmiTarget & 0xFF));
            code.Add(0x8D); code.Add(0xFA); code.Add(0xDF); // STA $DFFA
            code.Add(0xA9); code.Add((byte)((nmiTarget >> 8) & 0xFF));
            code.Add(0x8D); code.Add(0xFB); code.Add(0xDF); // STA $DFFB
            code.Add(0xA9); code.Add(0x35);
            code.Add(0x8D); code.Add(0x02); code.Add(0x01); // STA $0102
            code.Add(0xA9); code.Add(0xAC);
            code.Add(0x8D); code.Add(0x03); code.Add(0x01); // STA $0103
            code.Add(0x6C); code.Add(0xFC); code.Add(0xFF); // JMP ($FFFC), BIOS reset
            resetEntryAddress = bypassAddress + code.Count;
        }

        WriteJmp(code, resetTarget);

        if (code.Count > ResetStubReserve - 6)
        {
            Program.Error("error KQFC2504: FDS startup/bypass stub exceeded reserved tail space.");
            return tail;
        }

        Buffer.BlockCopy(code.ToArray(), 0, tail, 0, code.Count);
        WriteWordToArray(tail, ResetStubReserve - 6, Program.FdsLicenseBypassEnabled ? bypassAddress : nmiTarget);
        WriteWordToArray(tail, ResetStubReserve - 4, resetEntryAddress);
        WriteWordToArray(tail, ResetStubReserve - 2, irqTarget);
        return tail;
    }

    byte[] BuildPrgRom()
    {
        if (UsesFdsPrgRamLayout)
        {
            if (_switchableBankCount > 1 && !Program.FdsAutoOverlayEnabled)
                Program.Error("error KQFC2505: FDS native PRG-RAM layout has bank 2+ code/data, but auto overlay export is disabled. Use --fds-auto-overlay or move extra code/data into --fds-meta files.");
            var fdsPrg = new byte[2 * BankSize];
            FillWithFF(fdsPrg);
            Buffer.BlockCopy(_switchableBankImages[1], 0, fdsPrg, 0, BankSize);
            Buffer.BlockCopy(_commonBankImage, 0, fdsPrg, BankSize, BankSize);
            return fdsPrg;
        }

        if (_profile.UsesDuplicatedCommonBank)
        {
            int physicalBanks = Math.Max(1, _switchableBankCount);
            var prg = new byte[physicalBanks * 0x8000];
            FillWithFF(prg);
            for (int bank = 1; bank <= physicalBanks; bank++)
            {
                Buffer.BlockCopy(_switchableBankImages[bank], 0, prg, (bank - 1) * 0x8000, BankSize);
                Buffer.BlockCopy(_commonBankImage, 0, prg, ((bank - 1) * 0x8000) + BankSize, BankSize);
            }
            return prg;
        }

        if (_profile.PrgLayout == NesPrgLayoutKind.SuromOuter256FixedTop16)
        {
            if (_switchableBankCount > _profile.LogicalSwitchBankMax)
            {
                Program.Error("error KQFC2602 KQFC-SUROM-PRG-OVERFLOW: board 'surom512' supports logical switch banks 1..30; found {0}.", _switchableBankCount);
                return new byte[_profile.ExactPrgRomBytes];
            }

            var surom = new byte[_profile.ExactPrgRomBytes];
            FillWithFF(surom);
            foreach (var pair in _switchableBankImages)
            {
                int logical = pair.Key;
                if (logical < 1 || logical > 30) continue;
                int physical = _profile.LogicalToPhysicalBank(logical);
                if (physical == 15 || physical == 31)
                {
                    Program.Error("error KQFC2604 KQFC-SUROM-COMMON-REPLICA: logical bank {0} collided with common physical bank {1}.", logical, physical);
                    continue;
                }
                Buffer.BlockCopy(pair.Value, 0, surom, physical * BankSize, BankSize);
            }

            int tailOffset = BankSize - ResetStubReserve;
            if (_resetTail != null && _resetTail.Length == ResetStubReserve)
            {
                for (int physical = 1; physical < 32; physical += 2)
                    Buffer.BlockCopy(_resetTail, 0, surom, physical * BankSize + tailOffset, ResetStubReserve);
            }

            Buffer.BlockCopy(_commonBankImage, 0, surom, 15 * BankSize, BankSize);
            Buffer.BlockCopy(_commonBankImage, 0, surom, 31 * BankSize, BankSize);
            _commonReplicaEqual = true;
            for (int i = 0; i < BankSize; i++)
                if (surom[15 * BankSize + i] != surom[31 * BankSize + i]) { _commonReplicaEqual = false; break; }
            _commonReplicaSha256 = BuildReportUtil.ComputeSha256Hex(_commonBankImage);
            if (!_commonReplicaEqual)
                Program.Error("error KQFC2604 KQFC-SUROM-COMMON-REPLICA: physical banks 15 and 31 differ.");
            return surom;
        }

        int physicalSwitchable = Math.Max(1, _switchableBankCount);
        var rom = new byte[(physicalSwitchable + 1) * BankSize];
        FillWithFF(rom);
        for (int bank = 1; bank <= physicalSwitchable; bank++)
            Buffer.BlockCopy(_switchableBankImages[bank], 0, rom, (bank - 1) * BankSize, BankSize);
        Buffer.BlockCopy(_commonBankImage, 0, rom, physicalSwitchable * BankSize, BankSize);
        return rom;
    }

    byte[] LoadChrRom()
    {
        string path = Program.NesChrRomPath ?? "";
        if (_profile.RequiresChrRam)
        {
            if (!string.IsNullOrWhiteSpace(path))
                Program.Error("error KQFC2605 KQFC-SUROM-CHR-ROM-FORBIDDEN: board 'surom512' requires 8 KiB CHR-RAM and does not accept CHR-ROM input.");
            _chrRomSize = 0;
            return new byte[0];
        }
        if (string.IsNullOrWhiteSpace(path))
        {
            _chrRomSize = 0x2000;
            return new byte[_chrRomSize];
        }

        try
        {
            string fullPath = Path.GetFullPath(path);
            if (!File.Exists(fullPath))
            {
                Program.Error("error KQ0000: NES CHR file not found: {0}", path);
                _chrRomSize = 0x2000;
                return new byte[_chrRomSize];
            }

            byte[] raw = File.ReadAllBytes(fullPath);
            if (raw.Length == 0 || (raw.Length % 0x2000) != 0)
            {
                Program.Error("error KQ0000: NES CHR file must be a non-zero multiple of 8 KiB: {0} ({1} bytes)", path, raw.Length);
                _chrRomSize = 0x2000;
                return new byte[_chrRomSize];
            }

            _chrRomSize = raw.Length;
            return raw;
        }
        catch (Exception ex)
        {
            Program.Error("error KQ0000: failed to load NES CHR file '{0}': {1}", path, ex.Message);
            _chrRomSize = 0x2000;
            return new byte[_chrRomSize];
        }
    }

    byte[] BuildHeader(int prgRomSize, int chrRomSize)
    {
        byte[] header = new byte[HeaderSize];
        header[0] = 0x4E;
        header[1] = 0x45;
        header[2] = 0x53;
        header[3] = 0x1A;
        header[4] = (byte)(prgRomSize / 0x4000);
        header[5] = (byte)(chrRomSize / 0x2000);

        int flags6 = (_profile.MapperNumber & 0x0F) << 4;
        if (Program.NesCartridge.BatteryBacked) flags6 |= 0x02;
        if (Program.NesCartridge.Mirroring == NesMirroringKind.Vertical) flags6 |= 0x01;
        else if (Program.NesCartridge.Mirroring == NesMirroringKind.FourScreen) flags6 |= 0x08;
        header[6] = (byte)flags6;
        header[7] = (byte)(_profile.MapperNumber & 0xF0);
        if (_profile.IsSurom512)
            header[8] = 1; // one 8 KiB PRG-RAM unit in iNES 1.0
        return header;
    }

    AssemblerAnalysisReport BuildReport(int prgRomSize, int chrRomSize)
    {
        var report = new AssemblerAnalysisReport();
        report.RomSizeBytes = HeaderSize + prgRomSize + chrRomSize;
        report.PrgRomSizeBytes = prgRomSize;
        report.ChrRomSizeBytes = chrRomSize;
        report.UsedBytes = _functionSizes.Values.Select(x => x.EndFileOffset)
            .Concat(_readonlyDataSizes.Values.Select(x => x.EndFileOffset))
            .DefaultIfEmpty(0)
            .Max();
        report.CommonReplicaEqual = _commonReplicaEqual;
        report.CommonReplicaSha256 = _commonReplicaSha256;
        report.BankMaxPc = Enumerable.Range(0, _switchableBankCount + 1)
            .Select(bank => _functionSizes.Values.Where(f => f.Bank == bank).Select(f => f.CpuAddress + f.SizeBytes)
                .Concat(_readonlyDataSizes.Values.Where(d => d.Bank == bank).Select(d => d.CpuAddress + d.SizeBytes))
                .DefaultIfEmpty(GetCpuBase(bank))
                .Max())
            .ToArray();
        foreach (var pair in _functionSizes.OrderBy(x => x.Value.StartFileOffset))
            report.FunctionSizes.Add(pair.Value);
        foreach (var pair in _readonlyDataSizes.OrderBy(x => x.Value.StartFileOffset))
            report.ReadonlyData.Add(pair.Value);
        return report;
    }

    void ValidatePc(int bank, int pc, FilePosition source, string context)
    {
        int baseCpu = GetCpuBase(bank);
        int limitExclusive = GetBankLimitExclusive(bank);
        if (pc < baseCpu)
        {
            Program.Error("error KQ0000: NES assembler PC underflow in {0}.", context);
            return;
        }
        if (pc > limitExclusive)
        {
            Program.Error("error KQ0000: NES assembler overflow in bank {0}: code/data ran past ${1:X4} while processing {2}.", bank, limitExclusive, context);
        }
    }

    void WriteBytes(int bank, int cpuAddress, byte[] bytes, FilePosition source)
    {
        var target = GetBankImage(bank);
        int offset = cpuAddress - GetCpuBase(bank);
        if (offset < 0 || offset + bytes.Length > target.Length)
        {
            Program.Error("error KQ0000: NES assembler write exceeds bank {0} bounds at ${1:X4}.", bank, cpuAddress);
            return;
        }
        Buffer.BlockCopy(bytes, 0, target, offset, bytes.Length);
    }

    void WriteWord(int bank, int cpuAddress, int value, FilePosition source)
    {
        WriteBytes(bank, cpuAddress, new byte[] { (byte)(value & 0xFF), (byte)((value >> 8) & 0xFF) }, source);
    }

    byte[] GetBankImage(int bank)
    {
        return bank == 0 ? _commonBankImage : _switchableBankImages[bank];
    }

    int ResolveVectorTarget(params string[] candidates)
    {
        foreach (var candidate in candidates)
        {
            if (!string.IsNullOrEmpty(candidate) && _symbols.TryGetValue(candidate, out int value))
                return value;
        }
        return GetCpuBase(0);
    }

    int GetBankCapacity(int bank)
    {
        return bank == 0 || RequiresResetTailForLogicalBank(bank) ? (BankSize - ResetStubReserve) : BankSize;
    }

    int GetCpuBase(int bank)
    {
        if (UsesFdsPrgRamLayout) return bank == 0 ? FdsCommonCpuBase : FdsSwitchableCpuBase;
        return bank == 0 ? CommonCpuBase : SwitchableCpuBase;
    }

    int GetBankLimitExclusive(int bank)
    {
        if (UsesFdsPrgRamLayout) return bank == 0 ? FdsCommonBankLimitExclusive : FdsSwitchableBankLimitExclusive;
        return bank == 0 || RequiresResetTailForLogicalBank(bank) ? CommonBankLimitExclusive : SwitchableBankLimitExclusive;
    }

    bool RequiresResetTailForLogicalBank(int bank)
    {
        if (bank <= 0 || !_profile.RequiresPerBankResetStub) return false;
        switch (_profile.ResetVectorReplication)
        {
            case ResetVectorReplicationKind.EveryPhysical16KBank:
                return true;
            case ResetVectorReplicationKind.EveryPhysical32KHighBank:
                if (!_profile.IsSurom512) return (bank & 1) == 0;
                return (_profile.LogicalToPhysicalBank(bank) & 1) != 0;
            default:
                return false;
        }
    }

    int GetFileOffset(int bank, int cpuAddress)
    {
        int offsetWithinBank = cpuAddress - GetCpuBase(bank);
        if (_profile.UsesDuplicatedCommonBank)
        {
            if (bank == 0) return BankSize + offsetWithinBank;
            return ((bank - 1) * 0x8000) + offsetWithinBank;
        }
        if (_profile.IsSurom512)
        {
            if (bank == 0) return (15 * BankSize) + offsetWithinBank;
            return (_profile.LogicalToPhysicalBank(bank) * BankSize) + offsetWithinBank;
        }
        if (bank == 0) return (_switchableBankCount * BankSize) + offsetWithinBank;
        return ((bank - 1) * BankSize) + offsetWithinBank;
    }

    IEnumerable<IGrouping<int, PlacedUnit>> OrderedUnitsByBank(IEnumerable<PlacedUnit> units)
    {
        return (units ?? Enumerable.Empty<PlacedUnit>())
            .OrderBy(x => x.ActualBank)
            .ThenBy(x => x.Index)
            .GroupBy(x => x.ActualBank);
    }

    int GetResetStubCpuAddress()
    {
        return GetBankLimitExclusive(0);
    }

    int GetCommonSecondLast8kIndex()
    {
        return _switchableBankCount * 2;
    }

    int GetCommonLast8kIndex()
    {
        return (_switchableBankCount * 2) + 1;
    }

    static void WriteJmp(List<byte> bytes, int target)
    {
        bytes.Add(0x4C);
        bytes.Add((byte)(target & 0xFF));
        bytes.Add((byte)((target >> 8) & 0xFF));
    }

    static void WriteWordToArray(byte[] data, int offset, int value)
    {
        data[offset] = (byte)(value & 0xFF);
        data[offset + 1] = (byte)((value >> 8) & 0xFF);
    }

    static void FillWithFF(byte[] data)
    {
        for (int i = 0; i < data.Length; i++) data[i] = 0xFF;
    }

    static string NormalizeMnemonic(string mnemonic)
    {
        return (mnemonic ?? "").Trim().ToUpperInvariant();
    }

    byte[] EncodeInstruction(string mnemonic, AsmOperand operand, int pc, FilePosition source)
    {
        var actualMode = NormalizeAddressMode(operand, mnemonic, pc, source);
        if (Program.ErrorCount > 0) return new byte[0];

        if (!TryGetOpcode(mnemonic, actualMode, out byte opcode))
        {
            Program.Error("error KQ0000: NES assembler does not support instruction '{0}' with addressing mode {1}.", mnemonic, actualMode);
            return new byte[0];
        }

        int operandSize = OperandSize(actualMode);
        if (operandSize == 0) return new byte[] { opcode };

        int value = ResolveOperandValue(operand, actualMode, pc, source);
        if (Program.ErrorCount > 0) return new byte[0];

        if (actualMode == AddressMode.Relative)
        {
            int delta = value - (pc + 2);
            if (delta < -128 || delta > 127)
            {
                Program.Error("error KQ0000: NES relative branch out of range for {0} {1} targeting ${2:X4} from ${3:X4}.", mnemonic, operand.Show(), value, pc);
                return new byte[0];
            }
            return new byte[] { opcode, unchecked((byte)(sbyte)delta) };
        }

        if (operandSize == 1)
        {
            if (value < 0 || value > 0xFF)
            {
                Program.Error("error KQ0000: NES assembler operand out of 8-bit range: ${0:X4} for {1}.", value & 0xFFFF, mnemonic);
                return new byte[0];
            }
            return new byte[] { opcode, (byte)(value & 0xFF) };
        }

        return new byte[] { opcode, (byte)(value & 0xFF), (byte)((value >> 8) & 0xFF) };
    }

    AddressMode NormalizeAddressMode(AsmOperand operand, string mnemonic, int pc, FilePosition source)
    {
        operand = operand ?? AsmOperand.Implicit;
        if (operand.Mode == AddressMode.Implicit)
            return AddressMode.Implicit;

        if (operand.Mode == AddressMode.Immediate || operand.Mode == AddressMode.Relative || operand.Mode == AddressMode.Indirect ||
            operand.Mode == AddressMode.IndirectX || operand.Mode == AddressMode.IndirectY)
            return operand.Mode;

        int resolved = ResolveOperandValue(operand, operand.Mode, pc, source, applyModifier: false);
        if (Program.ErrorCount > 0) return operand.Mode;

        if (operand.Mode == AddressMode.Absolute)
        {
            if (TryGetOpcode(mnemonic, AddressMode.HighMem, out byte _) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMem;
            return AddressMode.Absolute;
        }
        if (operand.Mode == AddressMode.AbsoluteX)
        {
            if (TryGetOpcode(mnemonic, AddressMode.HighMemX, out byte _) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMemX;
            return AddressMode.AbsoluteX;
        }
        if (operand.Mode == AddressMode.AbsoluteY)
        {
            if (TryGetOpcode(mnemonic, AddressMode.HighMemY, out byte _) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMemY;
            return AddressMode.AbsoluteY;
        }

        return operand.Mode;
    }

    static bool TryResolveOperandValueForEstimate(AsmOperand operand, IReadOnlyDictionary<string, int> symbols, out int value, bool applyModifier = true)
    {
        operand = operand ?? AsmOperand.Implicit;
        if (operand.Base.HasValue)
        {
            if (symbols == null || !symbols.TryGetValue(operand.Base.Value, out int baseValue))
            {
                value = 0;
                return false;
            }
            value = baseValue + operand.Offset;
        }
        else
        {
            value = operand.Offset;
        }

        if (applyModifier)
        {
            if (operand.Modifier == ImmediateModifier.LowByte) value &= 0xFF;
            else if (operand.Modifier == ImmediateModifier.HighByte) value = (value >> 8) & 0xFF;
            else if (operand.Modifier == ImmediateModifier.Bank) value = 0;
        }

        return true;
    }

    static AddressMode EstimateAddressMode(string mnemonic, AsmOperand operand, IReadOnlyDictionary<string, int> symbols, int pc)
    {
        operand = operand ?? AsmOperand.Implicit;
        if (operand.Mode == AddressMode.Implicit) return AddressMode.Implicit;
        if (operand.Mode == AddressMode.Immediate || operand.Mode == AddressMode.Immediate16 || operand.Mode == AddressMode.Relative ||
            operand.Mode == AddressMode.Indirect || operand.Mode == AddressMode.IndirectX || operand.Mode == AddressMode.IndirectY)
            return operand.Mode;

        if (!TryResolveOperandValueForEstimate(operand, symbols, out int resolved, applyModifier: false))
            return operand.Mode;

        if (operand.Mode == AddressMode.Absolute)
        {
            if (TryGetOpcode(mnemonic, AddressMode.HighMem, out byte _) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMem;
            return AddressMode.Absolute;
        }
        if (operand.Mode == AddressMode.AbsoluteX)
        {
            if (TryGetOpcode(mnemonic, AddressMode.HighMemX, out byte _) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMemX;
            return AddressMode.AbsoluteX;
        }
        if (operand.Mode == AddressMode.AbsoluteY)
        {
            if (TryGetOpcode(mnemonic, AddressMode.HighMemY, out byte _) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMemY;
            return AddressMode.AbsoluteY;
        }

        return operand.Mode;
    }

    static int GetEstimatedInstructionSize(string mnemonic, AsmOperand operand, IReadOnlyDictionary<string, int> symbols, int pc)
    {
        return 1 + OperandSize(EstimateAddressMode(mnemonic, operand, symbols, pc));
    }

    static bool SymbolMapsEqual(IReadOnlyDictionary<string, int> left, IReadOnlyDictionary<string, int> right)
    {
        if (ReferenceEquals(left, right)) return true;
        if (left == null || right == null || left.Count != right.Count) return false;
        foreach (var pair in left)
        {
            if (!right.TryGetValue(pair.Key, out int value) || value != pair.Value)
                return false;
        }
        return true;
    }

    int ResolveOperandValue(AsmOperand operand, AddressMode mode, int pc, FilePosition source, bool applyModifier = true)
    {
        operand = operand ?? AsmOperand.Implicit;
        int value;
        if (operand.Base.HasValue)
        {
            if (!_symbols.TryGetValue(operand.Base.Value, out value))
            {
                Program.Error("error KQ0000: NES assembler unresolved symbol: {0}", operand.Base.Value);
                return 0;
            }
            value += operand.Offset;
        }
        else
        {
            value = operand.Offset;
        }

        if (applyModifier)
        {
            if (operand.Modifier == ImmediateModifier.LowByte) value &= 0xFF;
            else if (operand.Modifier == ImmediateModifier.HighByte) value = (value >> 8) & 0xFF;
            else if (operand.Modifier == ImmediateModifier.Bank) value = 0;
        }

        return value;
    }

    static int OperandSize(AddressMode mode)
    {
        if (mode == AddressMode.Implicit) return 0;
        if (mode == AddressMode.Immediate || mode == AddressMode.Relative || mode == AddressMode.HighMem || mode == AddressMode.HighMemX ||
            mode == AddressMode.HighMemY || mode == AddressMode.IndirectX || mode == AddressMode.IndirectY)
            return 1;
        return 2;
    }

    static readonly Dictionary<string, byte> Opcodes = BuildOpcodeTable();

    static bool TryGetOpcode(string mnemonic, AddressMode mode, out byte opcode)
    {
        return Opcodes.TryGetValue(MakeOpcodeKey(mnemonic, mode), out opcode);
    }

    static string MakeOpcodeKey(string mnemonic, AddressMode mode)
    {
        return mnemonic.ToUpperInvariant() + "|" + mode.ToString();
    }

    static void Def(Dictionary<string, byte> map, string mnemonic, AddressMode mode, byte opcode)
    {
        map[MakeOpcodeKey(mnemonic, mode)] = opcode;
    }

    static Dictionary<string, byte> BuildOpcodeTable()
    {
        var map = new Dictionary<string, byte>(StringComparer.OrdinalIgnoreCase);

        Def(map, "BRK", AddressMode.Implicit, 0x00);
        Def(map, "NOP", AddressMode.Implicit, 0xEA);
        Def(map, "RTS", AddressMode.Implicit, 0x60);
        Def(map, "RTI", AddressMode.Implicit, 0x40);
        Def(map, "TAX", AddressMode.Implicit, 0xAA);
        Def(map, "TXA", AddressMode.Implicit, 0x8A);
        Def(map, "TAY", AddressMode.Implicit, 0xA8);
        Def(map, "TYA", AddressMode.Implicit, 0x98);
        Def(map, "TSX", AddressMode.Implicit, 0xBA);
        Def(map, "TXS", AddressMode.Implicit, 0x9A);
        Def(map, "PHA", AddressMode.Implicit, 0x48);
        Def(map, "PLA", AddressMode.Implicit, 0x68);
        Def(map, "PHP", AddressMode.Implicit, 0x08);
        Def(map, "PLP", AddressMode.Implicit, 0x28);
        Def(map, "INX", AddressMode.Implicit, 0xE8);
        Def(map, "DEX", AddressMode.Implicit, 0xCA);
        Def(map, "INY", AddressMode.Implicit, 0xC8);
        Def(map, "DEY", AddressMode.Implicit, 0x88);
        Def(map, "CLC", AddressMode.Implicit, 0x18);
        Def(map, "SEC", AddressMode.Implicit, 0x38);
        Def(map, "CLI", AddressMode.Implicit, 0x58);
        Def(map, "SEI", AddressMode.Implicit, 0x78);
        Def(map, "CLV", AddressMode.Implicit, 0xB8);
        Def(map, "CLD", AddressMode.Implicit, 0xD8);
        Def(map, "SED", AddressMode.Implicit, 0xF8);
        Def(map, "ASL", AddressMode.Implicit, 0x0A);
        Def(map, "LSR", AddressMode.Implicit, 0x4A);
        Def(map, "ROL", AddressMode.Implicit, 0x2A);
        Def(map, "ROR", AddressMode.Implicit, 0x6A);

        Def(map, "JMP", AddressMode.Absolute, 0x4C);
        Def(map, "JMP", AddressMode.Indirect, 0x6C);
        Def(map, "JSR", AddressMode.Absolute, 0x20);
        Def(map, "BPL", AddressMode.Relative, 0x10);
        Def(map, "BMI", AddressMode.Relative, 0x30);
        Def(map, "BVC", AddressMode.Relative, 0x50);
        Def(map, "BVS", AddressMode.Relative, 0x70);
        Def(map, "BCC", AddressMode.Relative, 0x90);
        Def(map, "BCS", AddressMode.Relative, 0xB0);
        Def(map, "BNE", AddressMode.Relative, 0xD0);
        Def(map, "BEQ", AddressMode.Relative, 0xF0);

        Def(map, "LDA", AddressMode.Immediate, 0xA9);
        Def(map, "LDA", AddressMode.HighMem, 0xA5);
        Def(map, "LDA", AddressMode.HighMemX, 0xB5);
        Def(map, "LDA", AddressMode.Absolute, 0xAD);
        Def(map, "LDA", AddressMode.AbsoluteX, 0xBD);
        Def(map, "LDA", AddressMode.AbsoluteY, 0xB9);
        Def(map, "LDA", AddressMode.IndirectX, 0xA1);
        Def(map, "LDA", AddressMode.IndirectY, 0xB1);

        Def(map, "LDX", AddressMode.Immediate, 0xA2);
        Def(map, "LDX", AddressMode.HighMem, 0xA6);
        Def(map, "LDX", AddressMode.HighMemY, 0xB6);
        Def(map, "LDX", AddressMode.Absolute, 0xAE);
        Def(map, "LDX", AddressMode.AbsoluteY, 0xBE);

        Def(map, "LDY", AddressMode.Immediate, 0xA0);
        Def(map, "LDY", AddressMode.HighMem, 0xA4);
        Def(map, "LDY", AddressMode.HighMemX, 0xB4);
        Def(map, "LDY", AddressMode.Absolute, 0xAC);
        Def(map, "LDY", AddressMode.AbsoluteX, 0xBC);

        Def(map, "STA", AddressMode.HighMem, 0x85);
        Def(map, "STA", AddressMode.HighMemX, 0x95);
        Def(map, "STA", AddressMode.Absolute, 0x8D);
        Def(map, "STA", AddressMode.AbsoluteX, 0x9D);
        Def(map, "STA", AddressMode.AbsoluteY, 0x99);
        Def(map, "STA", AddressMode.IndirectX, 0x81);
        Def(map, "STA", AddressMode.IndirectY, 0x91);

        Def(map, "STX", AddressMode.HighMem, 0x86);
        Def(map, "STX", AddressMode.HighMemY, 0x96);
        Def(map, "STX", AddressMode.Absolute, 0x8E);
        Def(map, "STY", AddressMode.HighMem, 0x84);
        Def(map, "STY", AddressMode.HighMemX, 0x94);
        Def(map, "STY", AddressMode.Absolute, 0x8C);

        Def(map, "ADC", AddressMode.Immediate, 0x69);
        Def(map, "ADC", AddressMode.HighMem, 0x65);
        Def(map, "ADC", AddressMode.HighMemX, 0x75);
        Def(map, "ADC", AddressMode.Absolute, 0x6D);
        Def(map, "ADC", AddressMode.AbsoluteX, 0x7D);
        Def(map, "ADC", AddressMode.AbsoluteY, 0x79);
        Def(map, "ADC", AddressMode.IndirectX, 0x61);
        Def(map, "ADC", AddressMode.IndirectY, 0x71);

        Def(map, "SBC", AddressMode.Immediate, 0xE9);
        Def(map, "SBC", AddressMode.HighMem, 0xE5);
        Def(map, "SBC", AddressMode.HighMemX, 0xF5);
        Def(map, "SBC", AddressMode.Absolute, 0xED);
        Def(map, "SBC", AddressMode.AbsoluteX, 0xFD);
        Def(map, "SBC", AddressMode.AbsoluteY, 0xF9);
        Def(map, "SBC", AddressMode.IndirectX, 0xE1);
        Def(map, "SBC", AddressMode.IndirectY, 0xF1);

        Def(map, "CMP", AddressMode.Immediate, 0xC9);
        Def(map, "CMP", AddressMode.HighMem, 0xC5);
        Def(map, "CMP", AddressMode.HighMemX, 0xD5);
        Def(map, "CMP", AddressMode.Absolute, 0xCD);
        Def(map, "CMP", AddressMode.AbsoluteX, 0xDD);
        Def(map, "CMP", AddressMode.AbsoluteY, 0xD9);
        Def(map, "CMP", AddressMode.IndirectX, 0xC1);
        Def(map, "CMP", AddressMode.IndirectY, 0xD1);
        Def(map, "CPX", AddressMode.Immediate, 0xE0);
        Def(map, "CPX", AddressMode.HighMem, 0xE4);
        Def(map, "CPX", AddressMode.Absolute, 0xEC);
        Def(map, "CPY", AddressMode.Immediate, 0xC0);
        Def(map, "CPY", AddressMode.HighMem, 0xC4);
        Def(map, "CPY", AddressMode.Absolute, 0xCC);

        Def(map, "AND", AddressMode.Immediate, 0x29);
        Def(map, "AND", AddressMode.HighMem, 0x25);
        Def(map, "AND", AddressMode.HighMemX, 0x35);
        Def(map, "AND", AddressMode.Absolute, 0x2D);
        Def(map, "AND", AddressMode.AbsoluteX, 0x3D);
        Def(map, "AND", AddressMode.AbsoluteY, 0x39);
        Def(map, "AND", AddressMode.IndirectX, 0x21);
        Def(map, "AND", AddressMode.IndirectY, 0x31);
        Def(map, "ORA", AddressMode.Immediate, 0x09);
        Def(map, "ORA", AddressMode.HighMem, 0x05);
        Def(map, "ORA", AddressMode.HighMemX, 0x15);
        Def(map, "ORA", AddressMode.Absolute, 0x0D);
        Def(map, "ORA", AddressMode.AbsoluteX, 0x1D);
        Def(map, "ORA", AddressMode.AbsoluteY, 0x19);
        Def(map, "ORA", AddressMode.IndirectX, 0x01);
        Def(map, "ORA", AddressMode.IndirectY, 0x11);
        Def(map, "EOR", AddressMode.Immediate, 0x49);
        Def(map, "EOR", AddressMode.HighMem, 0x45);
        Def(map, "EOR", AddressMode.HighMemX, 0x55);
        Def(map, "EOR", AddressMode.Absolute, 0x4D);
        Def(map, "EOR", AddressMode.AbsoluteX, 0x5D);
        Def(map, "EOR", AddressMode.AbsoluteY, 0x59);
        Def(map, "EOR", AddressMode.IndirectX, 0x41);
        Def(map, "EOR", AddressMode.IndirectY, 0x51);

        Def(map, "BIT", AddressMode.HighMem, 0x24);
        Def(map, "BIT", AddressMode.Absolute, 0x2C);
        Def(map, "INC", AddressMode.HighMem, 0xE6);
        Def(map, "INC", AddressMode.HighMemX, 0xF6);
        Def(map, "INC", AddressMode.Absolute, 0xEE);
        Def(map, "INC", AddressMode.AbsoluteX, 0xFE);
        Def(map, "DEC", AddressMode.HighMem, 0xC6);
        Def(map, "DEC", AddressMode.HighMemX, 0xD6);
        Def(map, "DEC", AddressMode.Absolute, 0xCE);
        Def(map, "DEC", AddressMode.AbsoluteX, 0xDE);
        Def(map, "ASL", AddressMode.HighMem, 0x06);
        Def(map, "ASL", AddressMode.HighMemX, 0x16);
        Def(map, "ASL", AddressMode.Absolute, 0x0E);
        Def(map, "ASL", AddressMode.AbsoluteX, 0x1E);
        Def(map, "LSR", AddressMode.HighMem, 0x46);
        Def(map, "LSR", AddressMode.HighMemX, 0x56);
        Def(map, "LSR", AddressMode.Absolute, 0x4E);
        Def(map, "LSR", AddressMode.AbsoluteX, 0x5E);
        Def(map, "ROL", AddressMode.HighMem, 0x26);
        Def(map, "ROL", AddressMode.HighMemX, 0x36);
        Def(map, "ROL", AddressMode.Absolute, 0x2E);
        Def(map, "ROL", AddressMode.AbsoluteX, 0x3E);
        Def(map, "ROR", AddressMode.HighMem, 0x66);
        Def(map, "ROR", AddressMode.HighMemX, 0x76);
        Def(map, "ROR", AddressMode.Absolute, 0x6E);
        Def(map, "ROR", AddressMode.AbsoluteX, 0x7E);

        return map;
    }
}
