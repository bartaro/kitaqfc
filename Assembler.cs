using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;

sealed class Assembler
{
    const int HeaderSize = 16;
    const int PrgRomSize = 0x8000; // NROM-256 only in phase 3
    const int ChrRomSize = 0x2000; // CHR-ROM 8 KiB placeholder
    const int CpuBase = 0x8000;
    const int VectorNmi = 0xFFFA;
    const int VectorReset = 0xFFFC;
    const int VectorIrq = 0xFFFE;
    const int VectorStart = VectorNmi;

    public static AssemblerAnalysisReport LastReport { get; private set; } = new AssemblerAnalysisReport();

    readonly Dictionary<string, int> _symbols = new Dictionary<string, int>(StringComparer.Ordinal);
    readonly Dictionary<string, FunctionSizeInfo> _functionSizes = new Dictionary<string, FunctionSizeInfo>(StringComparer.Ordinal);
    readonly byte[] _prg = new byte[PrgRomSize];
    readonly string _branchLabelPrefix = "__kq_asm_branch_" + global::System.Guid.NewGuid().ToString("N");

    Assembler()
    {
        for (int i = 0; i < _prg.Length; i++) _prg[i] = 0xFF;
    }

    public static string Assemble(IReadOnlyList<Expr> assembly, string outputFilename)
    {
        var core = new BankedAssemblerCore();
        string result = core.Run(assembly, outputFilename);
        LastReport = core.LastReport ?? new AssemblerAnalysisReport();
        return result;
    }

    string Run(IReadOnlyList<Expr> assembly, string outputFilename)
    {
        assembly = ExpandLongBranches(assembly ?? new List<Expr>());
        WriteExpandedAssemblyDebugDump(assembly);

        Pass1(assembly);
        if (Program.ErrorCount > 0) return outputFilename;

        Pass2(assembly);
        if (Program.ErrorCount > 0) return outputFilename;

        int resetTarget = ResolveVectorTarget("__nes_reset", "__kq_reset_stub", "main", "__kq_hang_loop");
        int nmiTarget = ResolveVectorTarget("__nes_nmi", "__kq_nmi_default", "__kq_hang_loop");
        int irqTarget = ResolveVectorTarget("__nes_irq", "__kq_irq_default", "__kq_hang_loop");
        WriteVector(VectorNmi, nmiTarget);
        WriteVector(VectorReset, resetTarget);
        WriteVector(VectorIrq, irqTarget);

        var header = BuildHeader();
        var chr = LoadChrRom();
        byte[] rom = new byte[HeaderSize + PrgRomSize + ChrRomSize];
        Buffer.BlockCopy(header, 0, rom, 0, HeaderSize);
        Buffer.BlockCopy(_prg, 0, rom, HeaderSize, PrgRomSize);
        Buffer.BlockCopy(chr, 0, rom, HeaderSize + PrgRomSize, ChrRomSize);
        File.WriteAllBytes(outputFilename, rom);

        LastReport = BuildReport();
        return outputFilename;
    }

    IReadOnlyList<Expr> ExpandLongBranches(IReadOnlyList<Expr> assembly)
    {
        var current = (assembly ?? Array.Empty<Expr>()).ToList();
        int branchCounter = 0;
        for (int pass = 0; pass < 8; pass++)
        {
            var symbols = BuildAddressMap(current);
            var expanded = new List<Expr>(current.Count);
            bool changed = false;
            int pc = CpuBase;

            foreach (var e in current)
            {
                string label;
                string mnemonic;
                AsmOperand operand;
                byte[] bytes;
                int skip;
                int align;

                if (e.Match(Tag.Function, out label) || e.Match(Tag.Label, out label) || e.Match(Tag.Comment, out label) || e.Match(Tag.Section, out label))
                {
                    expanded.Add(e);
                    continue;
                }
                if (e.Match(Tag.SkipTo, out skip))
                {
                    expanded.Add(e);
                    pc = (skip == 0) ? Math.Max(pc, CpuBase) : skip;
                    continue;
                }
                if (e.Match(Tag.Align, out align))
                {
                    expanded.Add(e);
                    pc = (pc + (align - 1)) & ~(align - 1);
                    continue;
                }
                if (e.Match(Tag.ReadonlyData, out label, out bytes))
                {
                    expanded.Add(e);
                    pc += bytes == null ? 0 : bytes.Length;
                    continue;
                }
                if (e.Match(Tag.Word, out label))
                {
                    expanded.Add(e);
                    pc += 2;
                    continue;
                }
                if (e.Match(Tag.Asm, out mnemonic, out operand))
                {
                    if (TryExpandLongBranch(e, mnemonic, operand, symbols, pc, expanded, ref branchCounter))
                    {
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

            if (!changed) return current;
            current = expanded;
        }

        return current;
    }

    void WriteExpandedAssemblyDebugDump(IReadOnlyList<Expr> assembly)
    {
        if (!Program.EnableDebugOutput) return;
        try
        {
            Directory.CreateDirectory(Program.DebugOutputPath);
            File.WriteAllText(Path.Combine(Program.DebugOutputPath, "nes_expanded_assembly_dump.txt"), BuildExpandedAssemblyDebugDump(assembly));
        }
        catch (Exception ex)
        {
            Program.Warning("warning: failed to write NES expanded assembly dump: {0}", ex.Message);
        }
    }

    string BuildExpandedAssemblyDebugDump(IReadOnlyList<Expr> assembly)
    {
        var sb = new StringBuilder();
        var symbols = BuildAddressMap(assembly);
        int pc = CpuBase;
        int index = 0;
        foreach (var e in assembly ?? Array.Empty<Expr>())
        {
            index++;
            string label;
            string mnemonic;
            AsmOperand operand;
            byte[] bytes;
            int skip;
            int align;

            if (e.Match(Tag.Function, out label))
            {
                sb.AppendFormat("{0:D5} {1:X4} FUNC  {2}", index, pc, label).AppendLine();
                continue;
            }
            if (e.Match(Tag.Label, out label))
            {
                sb.AppendFormat("{0:D5} {1:X4} LABEL {2}", index, pc, label).AppendLine();
                continue;
            }
            if (e.Match(Tag.Comment, out label))
            {
                sb.AppendFormat("{0:D5} {1:X4} CMT   {2}", index, pc, label).AppendLine();
                continue;
            }
            if (e.Match(Tag.Section, out label))
            {
                sb.AppendFormat("{0:D5} {1:X4} SECT  {2}", index, pc, label).AppendLine();
                continue;
            }
            if (e.Match(Tag.SkipTo, out skip))
            {
                sb.AppendFormat("{0:D5} {1:X4} SKIP  {2:X4}", index, pc, skip == 0 ? Math.Max(pc, CpuBase) : skip).AppendLine();
                pc = (skip == 0) ? Math.Max(pc, CpuBase) : skip;
                continue;
            }
            if (e.Match(Tag.Align, out align))
            {
                int newPc = (pc + (align - 1)) & ~(align - 1);
                sb.AppendFormat("{0:D5} {1:X4} ALIGN {2} -> {3:X4}", index, pc, align, newPc).AppendLine();
                pc = newPc;
                continue;
            }
            if (e.Match(Tag.ReadonlyData, out label, out bytes))
            {
                sb.AppendFormat("{0:D5} {1:X4} RODAT {2} len={3}", index, pc, label ?? "", bytes == null ? 0 : bytes.Length).AppendLine();
                pc += bytes == null ? 0 : bytes.Length;
                continue;
            }
            if (e.Match(Tag.Word, out label))
            {
                sb.AppendFormat("{0:D5} {1:X4} WORD  {2}", index, pc, label).AppendLine();
                pc += 2;
                continue;
            }
            if (e.Match(Tag.Asm, out mnemonic, out operand))
            {
                int size = GetEstimatedInstructionSize(NormalizeMnemonic(mnemonic), operand, symbols, pc);
                sb.AppendFormat("{0:D5} {1:X4} ASM   {2} {3} size={4}", index, pc, mnemonic, operand == null ? "" : operand.Show(), size).AppendLine();
                pc += size;
                continue;
            }

            sb.AppendFormat("{0:D5} {1:X4} NODE  {2}", index, pc, e.Tag).AppendLine();
        }
        return sb.ToString();
    }

    bool TryExpandLongBranch(Expr e, string mnemonic, AsmOperand operand, Dictionary<string, int> symbols, int pc, List<Expr> expanded, ref int branchCounter)
    {
        if (operand == null || operand.Mode != AddressMode.Relative || operand.Modifier != ImmediateModifier.None || !operand.Base.HasValue)
            return false;

        int target;
        if (!symbols.TryGetValue(operand.Base.Value, out target))
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

    Dictionary<string, int> BuildAddressMap(IReadOnlyList<Expr> assembly)
    {
        var previous = new Dictionary<string, int>(StringComparer.Ordinal);
        for (int pass = 0; pass < 8; pass++)
        {
            var symbols = new Dictionary<string, int>(StringComparer.Ordinal);
            var lookup = new Dictionary<string, int>(previous, StringComparer.Ordinal);
            int pc = CpuBase;

            foreach (var e in assembly ?? Array.Empty<Expr>())
            {
                string label;
                string mnemonic;
                AsmOperand operand;
                byte[] bytes;
                int skip;
                int align;

                if (e.Match(Tag.Function, out label) || e.Match(Tag.Label, out label))
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
                if (e.Match(Tag.SkipTo, out skip))
                {
                    pc = (skip == 0) ? Math.Max(pc, CpuBase) : skip;
                    continue;
                }
                if (e.Match(Tag.Align, out align))
                {
                    pc = (pc + (align - 1)) & ~(align - 1);
                    continue;
                }
                if (e.Match(Tag.ReadonlyData, out label, out bytes))
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
                if (e.Match(Tag.Asm, out mnemonic, out operand))
                {
                    pc += GetEstimatedInstructionSize(NormalizeMnemonic(mnemonic), operand, lookup, pc);
                }
            }

            if (SymbolMapsEqual(previous, symbols))
                return symbols;
            previous = symbols;
        }

        return previous;
    }

    void Pass1(IReadOnlyList<Expr> assembly)
    {
        var symbols = BuildAddressMap(assembly);
        int pc = CpuBase;
        string currentFunction = null;
        int currentFunctionStart = 0;

        Action finalizeFunction = () =>
        {
            if (string.IsNullOrEmpty(currentFunction)) return;
            var info = new FunctionSizeInfo();
            info.Name = currentFunction;
            info.StartFileOffset = currentFunctionStart - CpuBase;
            info.EndFileOffset = pc - CpuBase;
            info.SizeBytes = pc - currentFunctionStart;
            info.Bank = 0;
            info.CpuAddress = currentFunctionStart;
            _functionSizes[currentFunction] = info;
            currentFunction = null;
        };

        foreach (var e in assembly)
        {
            string label;
            string mnemonic;
            AsmOperand operand;
            byte[] bytes;
            int skip;
            int align;
            if (e.Match(Tag.Function, out label))
            {
                finalizeFunction();
                currentFunction = label;
                currentFunctionStart = pc;
                DefineSymbol(label, pc, e.Source);
                continue;
            }
            if (e.Match(Tag.Label, out label))
            {
                DefineSymbol(label, pc, e.Source);
                continue;
            }
            if (e.Match(Tag.Comment, out label) || e.Match(Tag.Section, out label))
            {
                continue;
            }
            if (e.Match(Tag.SkipTo, out skip))
            {
                if (skip == 0) pc = Math.Max(pc, CpuBase);
                else pc = skip;
                ValidatePc(pc, e.Source, "skip");
                continue;
            }
            if (e.Match(Tag.Align, out align))
            {
                if (align <= 0 || (align & (align - 1)) != 0)
                {
                    Program.Error("error KQ0000: NES assembler $align expects a positive power of two.");
                    return;
                }
                pc = (pc + (align - 1)) & ~(align - 1);
                ValidatePc(pc, e.Source, "align");
                continue;
            }
            if (e.Match(Tag.ReadonlyData, out label, out bytes))
            {
                if (!string.IsNullOrEmpty(label))
                    DefineSymbol(label, pc, e.Source);
                pc += bytes == null ? 0 : bytes.Length;
                ValidatePc(pc, e.Source, "readonly data");
                continue;
            }
            if (e.Match(Tag.Word, out label))
            {
                pc += 2;
                ValidatePc(pc, e.Source, "$word");
                continue;
            }
            if (e.Match(Tag.Asm, out mnemonic, out operand))
            {
                pc += GetEstimatedInstructionSize(NormalizeMnemonic(mnemonic), operand, symbols, pc);
                ValidatePc(pc, e.Source, mnemonic);
                continue;
            }

            Program.Error("error KQ0000: NES assembler encountered an unsupported assembly node: {0}", e.Tag);
            return;
        }

        finalizeFunction();
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

    void Pass2(IReadOnlyList<Expr> assembly)
    {
        int pc = CpuBase;
        foreach (var e in assembly)
        {
            string label;
            string mnemonic;
            AsmOperand operand;
            byte[] bytes;
            int skip;
            int align;
            if (e.Match(Tag.Function, out label) || e.Match(Tag.Label, out label) || e.Match(Tag.Comment, out label) || e.Match(Tag.Section, out label))
            {
                continue;
            }
            if (e.Match(Tag.SkipTo, out skip))
            {
                pc = (skip == 0) ? Math.Max(pc, CpuBase) : skip;
                ValidatePc(pc, e.Source, "skip");
                continue;
            }
            if (e.Match(Tag.Align, out align))
            {
                pc = (pc + (align - 1)) & ~(align - 1);
                ValidatePc(pc, e.Source, "align");
                continue;
            }
            if (e.Match(Tag.ReadonlyData, out label, out bytes))
            {
                WriteBytes(pc, bytes ?? new byte[0], e.Source);
                pc += bytes == null ? 0 : bytes.Length;
                continue;
            }
            if (e.Match(Tag.Word, out label))
            {
                int value;
                if (!TryResolveWordValue(label, out value))
                {
                    Program.Error("error KQ0000: NES assembler unresolved symbol in $word: {0}", label);
                    return;
                }
                WriteWord(pc, value, e.Source);
                pc += 2;
                continue;
            }
            if (e.Match(Tag.Asm, out mnemonic, out operand))
            {
                byte[] encoded = EncodeInstruction(NormalizeMnemonic(mnemonic), operand ?? AsmOperand.Implicit, pc, e.Source);
                if (Program.ErrorCount > 0) return;
                WriteBytes(pc, encoded, e.Source);
                pc += encoded.Length;
                continue;
            }

            Program.Error("error KQ0000: NES assembler encountered an unsupported assembly node: {0}", e.Tag);
            return;
        }
    }

    void DefineSymbol(string label, int value, FilePosition source)
    {
        if (string.IsNullOrEmpty(label))
        {
            Program.Error("error KQ0000: NES assembler received an empty label.");
            return;
        }
        int old;
        if (_symbols.TryGetValue(label, out old))
        {
            Program.Error("error KQ0000: NES assembler duplicate label/function: {0}", label);
            return;
        }
        _symbols[label] = value;
    }

    static string NormalizeMnemonic(string mnemonic)
    {
        return (mnemonic ?? "").Trim().ToUpperInvariant();
    }

    byte[] LoadChrRom()
    {
        var chr = new byte[ChrRomSize];
        string path = Program.NesChrRomPath ?? "";
        if (string.IsNullOrWhiteSpace(path)) return chr;

        try
        {
            string fullPath = Path.GetFullPath(path);
            if (!File.Exists(fullPath))
            {
                Program.Error("error KQ0000: NES CHR file not found: {0}", path);
                return chr;
            }

            byte[] raw = File.ReadAllBytes(fullPath);
            if (raw.Length > ChrRomSize)
            {
                Program.Error("error KQ0000: NES CHR file exceeds 8 KiB NROM CHR-ROM limit: {0} ({1} bytes)", path, raw.Length);
                return chr;
            }

            Buffer.BlockCopy(raw, 0, chr, 0, raw.Length);
            return chr;
        }
        catch (Exception ex)
        {
            Program.Error("error KQ0000: failed to load NES CHR file '{0}': {1}", path, ex.Message);
            return chr;
        }
    }

    void ValidatePc(int pc, FilePosition source, string context)
    {
        if (pc < CpuBase)
        {
            Program.Error("error KQ0000: NES assembler PC underflow in {0}.", context);
            return;
        }
        if (pc > VectorStart)
        {
            Program.Error("error KQ0000: NES assembler overflow: code/data ran past vector table (${0:X4}) while processing {1}.", VectorStart, context);
        }
    }

    void WriteBytes(int cpuAddress, byte[] bytes, FilePosition source)
    {
        int offset = cpuAddress - CpuBase;
        if (offset < 0 || offset + bytes.Length > _prg.Length)
        {
            Program.Error("error KQ0000: NES assembler write exceeds PRG-ROM bounds at ${0:X4}.", cpuAddress);
            return;
        }
        Buffer.BlockCopy(bytes, 0, _prg, offset, bytes.Length);
    }

    void WriteWord(int cpuAddress, int value, FilePosition source)
    {
        int offset = cpuAddress - CpuBase;
        if (offset < 0 || offset + 2 > _prg.Length)
        {
            Program.Error("error KQ0000: NES assembler word write exceeds PRG-ROM bounds at ${0:X4}.", cpuAddress);
            return;
        }
        _prg[offset] = (byte)(value & 0xFF);
        _prg[offset + 1] = (byte)((value >> 8) & 0xFF);
    }

    void WriteVector(int vectorCpuAddress, int targetCpuAddress)
    {
        WriteWord(vectorCpuAddress, targetCpuAddress, FilePosition.Unknown);
    }

    int ResolveVectorTarget(params string[] candidates)
    {
        foreach (var candidate in candidates)
        {
            int value;
            if (!string.IsNullOrEmpty(candidate) && _symbols.TryGetValue(candidate, out value)) return value;
        }
        return CpuBase;
    }

    static byte[] BuildHeader()
    {
        byte[] header = new byte[HeaderSize];
        header[0] = 0x4E; // N
        header[1] = 0x45; // E
        header[2] = 0x53; // S
        header[3] = 0x1A;
        header[4] = 2; // 2 x 16 KiB PRG-ROM = 32 KiB (NROM-256)
        header[5] = 1; // 1 x 8 KiB CHR-ROM
        header[6] = 0x00; // mapper 0, horizontal mirroring, no battery/trainer
        header[7] = 0x00;
        return header;
    }

    AssemblerAnalysisReport BuildReport()
    {
        var report = new AssemblerAnalysisReport();
        report.RomSizeBytes = HeaderSize + PrgRomSize + ChrRomSize;
        report.PrgRomSizeBytes = PrgRomSize;
        report.ChrRomSizeBytes = ChrRomSize;
        int used = 0;
        for (int i = 0; i < VectorStart - CpuBase; i++)
        {
            if (_prg[i] != 0xFF) used = i + 1;
        }
        report.UsedBytes = used;
        report.BankMaxPc = new int[] { used };
        foreach (var pair in _functionSizes.OrderBy(x => x.Value.StartFileOffset))
        {
            report.FunctionSizes.Add(pair.Value);
        }
        return report;
    }

    byte[] EncodeInstruction(string mnemonic, AsmOperand operand, int pc, FilePosition source)
    {
        var actualMode = NormalizeAddressMode(operand, mnemonic, pc, source);
        if (Program.ErrorCount > 0) return new byte[0];

        byte opcode;
        if (!TryGetOpcode(mnemonic, actualMode, out opcode))
        {
            Program.Error("error KQ0000: NES assembler does not support instruction '{0}' with addressing mode {1} in phase 3.", mnemonic, actualMode);
            return new byte[0];
        }

        int operandSize = OperandSize(actualMode);
        if (operandSize == 0) return new byte[] { opcode };

        int value = ResolveOperandValue(operand, actualMode, pc, source);
        if (Program.ErrorCount > 0) return new byte[0];

        if (actualMode == AddressMode.Relative)
        {
            int target = value;
            int nextPc = pc + 2;
            int delta = target - nextPc;
            if (delta < -128 || delta > 127)
            {
                Program.Error("error KQ0000: NES relative branch out of range for {0} {1} targeting ${2:X4} from ${3:X4}.", mnemonic, operand.Show(), target, pc);
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
        {
            byte impliedOpcode;
            if (IsAccumulatorMnemonic(mnemonic) && TryGetOpcode(mnemonic, AddressMode.Implicit, out impliedOpcode))
                return AddressMode.Implicit;
            return AddressMode.Implicit;
        }

        if (operand.Mode == AddressMode.Immediate || operand.Mode == AddressMode.Relative || operand.Mode == AddressMode.Indirect ||
            operand.Mode == AddressMode.IndirectX || operand.Mode == AddressMode.IndirectY)
            return operand.Mode;

        int resolved = ResolveOperandValue(operand, operand.Mode, pc, source, applyModifier: false);
        if (Program.ErrorCount > 0) return operand.Mode;

        if (operand.Mode == AddressMode.Absolute)
        {
            byte zpOpcode;
            if (TryGetOpcode(mnemonic, AddressMode.HighMem, out zpOpcode) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMem;
            return AddressMode.Absolute;
        }
        if (operand.Mode == AddressMode.AbsoluteX)
        {
            byte zpxOpcode;
            if (TryGetOpcode(mnemonic, AddressMode.HighMemX, out zpxOpcode) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMemX;
            return AddressMode.AbsoluteX;
        }
        if (operand.Mode == AddressMode.AbsoluteY)
        {
            byte zpyOpcode;
            if (TryGetOpcode(mnemonic, AddressMode.HighMemY, out zpyOpcode) && resolved >= 0 && resolved <= 0xFF)
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
            int baseValue;
            if (symbols == null || !symbols.TryGetValue(operand.Base.Value, out baseValue))
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
        if (operand.Mode == AddressMode.Implicit)
            return AddressMode.Implicit;

        if (operand.Mode == AddressMode.Immediate || operand.Mode == AddressMode.Immediate16 || operand.Mode == AddressMode.Relative ||
            operand.Mode == AddressMode.Indirect || operand.Mode == AddressMode.IndirectX || operand.Mode == AddressMode.IndirectY)
            return operand.Mode;

        int resolved;
        if (!TryResolveOperandValueForEstimate(operand, symbols, out resolved, applyModifier: false))
            return operand.Mode;

        if (operand.Mode == AddressMode.Absolute)
        {
            byte zpOpcode;
            if (TryGetOpcode(mnemonic, AddressMode.HighMem, out zpOpcode) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMem;
            return AddressMode.Absolute;
        }
        if (operand.Mode == AddressMode.AbsoluteX)
        {
            byte zpxOpcode;
            if (TryGetOpcode(mnemonic, AddressMode.HighMemX, out zpxOpcode) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMemX;
            return AddressMode.AbsoluteX;
        }
        if (operand.Mode == AddressMode.AbsoluteY)
        {
            byte zpyOpcode;
            if (TryGetOpcode(mnemonic, AddressMode.HighMemY, out zpyOpcode) && resolved >= 0 && resolved <= 0xFF)
                return AddressMode.HighMemY;
            return AddressMode.AbsoluteY;
        }

        return operand.Mode;
    }

    static int GetEstimatedInstructionSize(string mnemonic, AsmOperand operand, IReadOnlyDictionary<string, int> symbols, int pc)
    {
        AddressMode mode = EstimateAddressMode(mnemonic, operand, symbols, pc);
        return 1 + OperandSize(mode);
    }

    static bool SymbolMapsEqual(IReadOnlyDictionary<string, int> left, IReadOnlyDictionary<string, int> right)
    {
        if (ReferenceEquals(left, right)) return true;
        if (left == null || right == null || left.Count != right.Count) return false;
        foreach (var pair in left)
        {
            int value;
            if (!right.TryGetValue(pair.Key, out value) || value != pair.Value)
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
            string name = operand.Base.Value;
            if (!_symbols.TryGetValue(name, out value))
            {
                Program.Error("error KQ0000: NES assembler unresolved symbol: {0}", name);
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

    static bool IsAccumulatorMnemonic(string mnemonic)
    {
        return string.Equals(mnemonic, "ASL", StringComparison.OrdinalIgnoreCase) ||
               string.Equals(mnemonic, "LSR", StringComparison.OrdinalIgnoreCase) ||
               string.Equals(mnemonic, "ROL", StringComparison.OrdinalIgnoreCase) ||
               string.Equals(mnemonic, "ROR", StringComparison.OrdinalIgnoreCase);
    }

    static int GetInstructionSize(string mnemonic, AsmOperand operand)
    {
        AddressMode mode = operand == null ? AddressMode.Implicit : operand.Mode;
        return 1 + OperandSize(mode);
    }

    static int OperandSize(AddressMode mode)
    {
        if (mode == AddressMode.Implicit) return 0;
        if (mode == AddressMode.Immediate || mode == AddressMode.Relative || mode == AddressMode.HighMem || mode == AddressMode.HighMemX || mode == AddressMode.HighMemY || mode == AddressMode.IndirectX || mode == AddressMode.IndirectY) return 1;
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

        // Implied / accumulator
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

        // Control flow
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

        // Loads/stores
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

        // Arithmetic / logic
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
