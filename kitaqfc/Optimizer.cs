using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

// Rewrite assembly IR using local patterns and record textual pass summaries.
// Pattern matching is not a whole-program register/flag liveness proof.
class Optimizer
{
    // Keep the latest report in shared static state; a new Optimize call replaces it.
    public static OptimizerAnalysisReport LastReport { get; private set; } = new OptimizerAnalysisReport();

    // Copy the input list, treating null as empty. Level zero skips all rewrites;
    // higher levels run two peephole sweeps, register/load reuse and fallthrough cleanup.
    public static List<Expr> Optimize(List<Expr> sourceLines, int optLevel)
    {
        var report = new OptimizerAnalysisReport();
        var lines = new List<Expr>(sourceLines ?? new List<Expr>());

        if (optLevel <= 0)
        {
            LastReport = report;
            return lines;
        }

        lines = RunWithDiff("o1_peepholes_pass1", lines, x => RemoveUnreachable(ApplyO1Peepholes(x)), report);
        lines = RunWithDiff("o1_peepholes_pass2", lines, x => RemoveUnreachable(ApplyO1Peepholes(x)), report);
        lines = RunWithDiff("mini_cse", lines, ApplyMiniCsePass, report);
        lines = RunWithDiff("cleanup", lines, RemoveUnreachable, report);

        LastReport = report;
        return lines;
    }

    // Run one pass and compare diagnostic lines by position. Added/removed counts are net length differences,
    // not an edit-distance diff or measurements of encoded instruction bytes.
    static List<Expr> RunWithDiff(string passName, List<Expr> before, Func<List<Expr>, List<Expr>> pass, OptimizerAnalysisReport report)
    {
        var beforeLines = before.Select(ShowExprForDiff).ToList();
        var after = pass(before);
        var afterLines = after.Select(ShowExprForDiff).ToList();

        var pr = new OptimizerPassReport
        {
            Name = passName,
            BeforeLines = beforeLines.Count,
            AfterLines = afterLines.Count
        };

        int min = Math.Min(beforeLines.Count, afterLines.Count);
        int changed = 0;
        for (int i = 0; i < min; i++)
        {
            if (!string.Equals(beforeLines[i], afterLines[i], StringComparison.Ordinal))
                changed++;
        }
        pr.ChangedLines = changed;
        pr.AddedLines = Math.Max(0, afterLines.Count - beforeLines.Count);
        pr.RemovedLines = Math.Max(0, beforeLines.Count - afterLines.Count);
        pr.DiffText = BuildSimpleDiff(beforeLines, afterLines);

        report.Passes.Add(pr);
        return after;
    }

    // Render assembly operands or general IR nodes for reports, omitting implicit operand text.
    static string ShowExprForDiff(Expr e)
    {
        string m;
        AsmOperand o;
        if (e.Match(Tag.Asm, out m, out o))
        {
            if (o == null || o.Mode == AddressMode.Implicit) return m;
            return m + " " + o.Show();
        }
        return e.Show();
    }

    // Show at most 120 differing positions or tail entries, with no insertion alignment.
    // One changed position can emit several text lines; the limit counts entries, not output lines.
    static string BuildSimpleDiff(List<string> before, List<string> after)
    {
        const int MaxLines = 120;
        var sb = new StringBuilder();
        int min = Math.Min(before.Count, after.Count);
        int emitted = 0;

        for (int i = 0; i < min && emitted < MaxLines; i++)
        {
            if (!string.Equals(before[i], after[i], StringComparison.Ordinal))
            {
                sb.AppendLine("@@ line " + (i + 1) + " @@");
                sb.AppendLine("- " + before[i]);
                sb.AppendLine("+ " + after[i]);
                emitted++;
            }
        }

        if (emitted < MaxLines && after.Count > before.Count)
        {
            for (int i = before.Count; i < after.Count && emitted < MaxLines; i++)
            {
                sb.AppendLine("+ " + after[i]);
                emitted++;
            }
        }
        else if (emitted < MaxLines && before.Count > after.Count)
        {
            for (int i = after.Count; i < before.Count && emitted < MaxLines; i++)
            {
                sb.AppendLine("- " + before[i]);
                emitted++;
            }
        }

        if (sb.Length == 0)
        {
            sb.AppendLine("(no textual changes)");
        }
        else if (emitted >= MaxLines)
        {
            sb.AppendLine("... (truncated)");
        }
        return sb.ToString();
    }

    // Apply adjacent 6502 patterns in one forward sweep, preserving source coordinates for a rewritten tail call.
    static List<Expr> ApplyO1Peepholes(List<Expr> lines)
    {
        var outLines = new List<Expr>(lines.Count);

        for (int i = 0; i < lines.Count; i++)
        {
            var cur = lines[i];
            string mnemonic;
            AsmOperand operand;
            if (!cur.Match(Tag.Asm, out mnemonic, out operand))
            {
                outLines.Add(cur);
                continue;
            }

            string nextMnemonic = null;
            AsmOperand nextOperand = null;
            bool hasImmediateNextAsm = i + 1 < lines.Count && lines[i + 1].Match(Tag.Asm, out nextMnemonic, out nextOperand);

            if (TargetsImmediatelyFollowingLabel(lines, i, mnemonic, operand))
                continue;

            // Replace a symbolic call immediately followed by RTS with a tail jump.
            if (hasImmediateNextAsm && mnemonic == "JSR" && IsAbsoluteLabelOperand(operand) && nextMnemonic == "RTS")
            {
                outLines.Add(Expr.MakeAsm("JMP", operand.WithMode(AddressMode.Absolute)).WithSource(cur.Source));
                i++;
                continue;
            }

            // Keep the first of two inverse transfers; it already leaves both registers with the same value.
            if (hasImmediateNextAsm && AreMirrorTransfers(mnemonic, nextMnemonic))
            {
                outLines.Add(cur);
                i++;
                continue;
            }

            // Keep one copy of selected idempotent instructions; instruction timing and interrupt sampling are not preserved.
            if (hasImmediateNextAsm && mnemonic == nextMnemonic && IsDuplicateSafeInstruction(mnemonic))
            {
                outLines.Add(cur);
                i++;
                continue;
            }

            // Remove compare-zero when the preceding recognized instruction sets the tested register's zero flag.
            // Only the adjacent Z branch is checked; this rule does not analyze carry use beyond that branch.
            if (hasImmediateNextAsm && IsImmediateZero(operand) && (nextMnemonic == "BEQ" || nextMnemonic == "BNE"))
            {
                if (mnemonic == "CMP" && PreviousInstructionSetsZeroFromA(lines, i))
                {
                    outLines.Add(lines[i + 1]);
                    i++;
                    continue;
                }
                if (mnemonic == "CPX" && PreviousInstructionSetsZeroFromX(lines, i))
                {
                    outLines.Add(lines[i + 1]);
                    i++;
                    continue;
                }
                if (mnemonic == "CPY" && PreviousInstructionSetsZeroFromY(lines, i))
                {
                    outLines.Add(lines[i + 1]);
                    i++;
                    continue;
                }
            }

            outLines.Add(cur);
        }

        return outLines;
    }

    // Reuse tracked A/X/Y values within uninterrupted IR regions, preserving comments and resetting at boundaries.
    // Load elimination also requires the corresponding zero/negative flags to be current.
    static List<Expr> ApplyMiniCsePass(List<Expr> lines)
    {
        var outLines = new List<Expr>(lines.Count);
        var state = new TrackedRegisterState();

        foreach (var e in lines)
        {
            if (e.MatchTag(Tag.Comment))
            {
                outLines.Add(e);
                continue;
            }

            if (e.MatchTag(Tag.Label) || e.MatchTag(Tag.Function) || e.MatchTag(Tag.Section) ||
                e.MatchTag(Tag.SkipTo) || e.MatchTag(Tag.Align) || e.MatchTag(Tag.ReadonlyData) || e.MatchTag(Tag.Word))
            {
                outLines.Add(e);
                state.Reset();
                continue;
            }

            string mnemonic;
            AsmOperand operand;
            if (!e.Match(Tag.Asm, out mnemonic, out operand))
            {
                outLines.Add(e);
                state.Reset();
                continue;
            }

            if (IsControlFlowOrCall(mnemonic))
            {
                outLines.Add(e);
                state.Reset();
                continue;
            }

            string targetReg;
            string valueKey;
            if (TryGetTrackedLoad(mnemonic, operand, out targetReg, out valueKey))
            {
                if (state.CanElideLoad(targetReg, valueKey))
                    continue;

                string transferMnemonic = state.TryGetTransferMnemonic(targetReg, valueKey);
                if (transferMnemonic != null)
                {
                    outLines.Add(Expr.MakeAsm(transferMnemonic).WithSource(e.Source));
                    state.Observe(transferMnemonic, AsmOperand.Implicit);
                    continue;
                }

                outLines.Add(e);
                state.ApplyTrackedLoad(targetReg, valueKey);
                continue;
            }

            outLines.Add(e);
            state.Observe(mnemonic, operand);
        }

        return outLines;
    }

    // Track value identities for A/X/Y and which register supplied the current zero/negative flags.
    sealed class TrackedRegisterState
    {
        string _aKey;
        string _xKey;
        string _yKey;
        string _flagRegister;

        // Discard all value and flag-source knowledge.
        public void Reset()
        {
            _aKey = null;
            _xKey = null;
            _yKey = null;
            _flagRegister = null;
        }

        // Require both the same value identity and flags already describing the destination register.
        public bool CanElideLoad(string registerName, string valueKey)
        {
            return string.Equals(GetKey(registerName), valueKey, StringComparison.Ordinal) &&
                   string.Equals(_flagRegister, registerName, StringComparison.Ordinal);
        }

        // Choose an available A-to-X/Y or X/Y-to-A transfer for a known value; no direct X/Y transfer exists here.
        public string TryGetTransferMnemonic(string registerName, string valueKey)
        {
            if (registerName == "A")
            {
                if (string.Equals(_xKey, valueKey, StringComparison.Ordinal)) return "TXA";
                if (string.Equals(_yKey, valueKey, StringComparison.Ordinal)) return "TYA";
            }
            else if (registerName == "X")
            {
                if (string.Equals(_aKey, valueKey, StringComparison.Ordinal)) return "TAX";
            }
            else if (registerName == "Y")
            {
                if (string.Equals(_aKey, valueKey, StringComparison.Ordinal)) return "TAY";
            }
            return null;
        }

        // Remember the loaded value and mark its destination as the source of zero/negative flags.
        public void ApplyTrackedLoad(string registerName, string valueKey)
        {
            SetKey(registerName, valueKey);
            _flagRegister = registerName;
        }

        // Invalidate memory-derived identities on recognized writes, then model register/flag effects.
        // Unknown instructions clear all state; known flag-neutral instructions retain it.
        public void Observe(string mnemonic, AsmOperand operand)
        {
            if (mnemonic == null)
            {
                Reset();
                return;
            }

            bool memoryWrite = IsMemoryWrite(mnemonic, operand);
            if (memoryWrite)
                InvalidateMemoryBackedValues();

            switch (mnemonic)
            {
                case "TAX":
                    _xKey = _aKey;
                    _flagRegister = "X";
                    return;
                case "TXA":
                    _aKey = _xKey;
                    _flagRegister = "A";
                    return;
                case "TAY":
                    _yKey = _aKey;
                    _flagRegister = "Y";
                    return;
                case "TYA":
                    _aKey = _yKey;
                    _flagRegister = "A";
                    return;
                case "LDA":
                    _aKey = null;
                    _flagRegister = "A";
                    return;
                case "LDX":
                    _xKey = null;
                    _flagRegister = "X";
                    return;
                case "LDY":
                    _yKey = null;
                    _flagRegister = "Y";
                    return;
                case "INX":
                case "DEX":
                    _xKey = null;
                    _flagRegister = "X";
                    return;
                case "INY":
                case "DEY":
                    _yKey = null;
                    _flagRegister = "Y";
                    return;
                case "PLA":
                    _aKey = null;
                    _flagRegister = "A";
                    return;
                case "TSX":
                    _xKey = null;
                    _flagRegister = "X";
                    return;
                case "ADC":
                case "SBC":
                case "AND":
                case "ORA":
                case "EOR":
                    _aKey = null;
                    _flagRegister = "A";
                    return;
                case "ASL":
                case "LSR":
                case "ROL":
                case "ROR":
                    if (operand == null || operand.Mode == AddressMode.Implicit)
                    {
                        _aKey = null;
                        _flagRegister = "A";
                    }
                    else
                    {
                        _flagRegister = null;
                    }
                    return;
                case "CMP":
                case "CPX":
                case "CPY":
                case "BIT":
                case "INC":
                case "DEC":
                    _flagRegister = null;
                    return;
                case "STA":
                case "STX":
                case "STY":
                case "PHA":
                case "PHP":
                case "TXS":
                case "NOP":
                case "CLC":
                case "SEC":
                case "CLI":
                case "SEI":
                case "CLD":
                case "SED":
                case "CLV":
                    return;
                default:
                    Reset();
                    return;
            }
        }

        // Drop cached memory identities while retaining immediate constants and the flag-source marker.
        void InvalidateMemoryBackedValues()
        {
            if (IsMemoryKey(_aKey)) _aKey = null;
            if (IsMemoryKey(_xKey)) _xKey = null;
            if (IsMemoryKey(_yKey)) _yKey = null;
        }

        // Read a supported register identity; callers supply A/X/Y, with other strings falling back to Y.
        string GetKey(string registerName)
        {
            if (registerName == "A") return _aKey;
            if (registerName == "X") return _xKey;
            return _yKey;
        }

        // Assign a supported register identity; callers supply A/X/Y, with other strings selecting Y.
        void SetKey(string registerName, string valueKey)
        {
            if (registerName == "A") _aKey = valueKey;
            else if (registerName == "X") _xKey = valueKey;
            else _yKey = valueKey;
        }

        // Identify keys produced for concrete memory addresses, excluding immediate constants.
        static bool IsMemoryKey(string key)
        {
            return key != null && key.StartsWith("MEM:", StringComparison.Ordinal);
        }
    }

    // Map LDA/LDX/LDY to a destination register and attempt to derive a reusable operand identity.
    static bool TryGetTrackedLoad(string mnemonic, AsmOperand operand, out string targetReg, out string valueKey)
    {
        targetReg = null;
        valueKey = null;

        if (mnemonic == "LDA") targetReg = "A";
        else if (mnemonic == "LDX") targetReg = "X";
        else if (mnemonic == "LDY") targetReg = "Y";
        else return false;

        return TryGetTrackedValueKey(operand, out valueKey);
    }

    // Track unmodified numeric immediates or numeric absolute/zero-page reads outside the excluded I/O range.
    // Symbolic, indexed and indirect operands are deliberately untracked.
    static bool TryGetTrackedValueKey(AsmOperand operand, out string valueKey)
    {
        valueKey = null;
        if (operand == null || operand.Modifier != ImmediateModifier.None)
            return false;

        if (operand.Mode == AddressMode.Immediate && !operand.Base.HasValue)
        {
            valueKey = "IMM:" + (operand.Offset & 0xFF).ToString();
            return true;
        }

        if ((operand.Mode == AddressMode.Absolute || operand.Mode == AddressMode.HighMem) && !operand.Base.HasValue)
        {
            int addr = operand.Offset & 0xFFFF;
            if (IsVolatileAbsoluteAddress(addr))
                return false;
            valueKey = "MEM:" + addr.ToString();
            return true;
        }

        return false;
    }

    // Exclude PPU mirrors and the base APU/controller/I/O range 2000-401F.
    // Mapper-specific expansion registers outside this range are not classified here.
    static bool IsVolatileAbsoluteAddress(int address)
    {
        return address >= 0x2000 && address < 0x4020;
    }

    // Look backward past trivia for a known A-result instruction; memory shifts do not qualify.
    static bool PreviousInstructionSetsZeroFromA(List<Expr> lines, int currentIndex)
    {
        int prev = FindPreviousAsmIndex(lines, currentIndex - 1);
        if (prev < 0) return false;

        string mnemonic;
        AsmOperand operand;
        if (!lines[prev].Match(Tag.Asm, out mnemonic, out operand))
            return false;

        switch (mnemonic)
        {
            case "LDA":
            case "TXA":
            case "TYA":
            case "PLA":
            case "ADC":
            case "SBC":
            case "AND":
            case "ORA":
            case "EOR":
                return true;
            case "ASL":
            case "LSR":
            case "ROL":
            case "ROR":
                return operand == null || operand.Mode == AddressMode.Implicit;
            default:
                return false;
        }
    }

    // Look backward past trivia for a listed X load, transfer or increment/decrement.
    static bool PreviousInstructionSetsZeroFromX(List<Expr> lines, int currentIndex)
    {
        int prev = FindPreviousAsmIndex(lines, currentIndex - 1);
        if (prev < 0) return false;

        string mnemonic;
        AsmOperand operand;
        if (!lines[prev].Match(Tag.Asm, out mnemonic, out operand))
            return false;

        return mnemonic == "LDX" || mnemonic == "TAX" || mnemonic == "TSX" || mnemonic == "INX" || mnemonic == "DEX";
    }

    // Look backward past trivia for a listed Y load, transfer or increment/decrement.
    static bool PreviousInstructionSetsZeroFromY(List<Expr> lines, int currentIndex)
    {
        int prev = FindPreviousAsmIndex(lines, currentIndex - 1);
        if (prev < 0) return false;

        string mnemonic;
        AsmOperand operand;
        if (!lines[prev].Match(Tag.Asm, out mnemonic, out operand))
            return false;

        return mnemonic == "LDY" || mnemonic == "TAY" || mnemonic == "INY" || mnemonic == "DEY";
    }

    // Skip comments/sections backward and return the nearest assembly node; stop at any other node.
    static int FindPreviousAsmIndex(List<Expr> lines, int startIndex)
    {
        for (int i = startIndex; i >= 0; i--)
        {
            if (IsTriviaNode(lines[i])) continue;
            if (lines[i].MatchTag(Tag.Asm)) return i;
            return -1;
        }
        return -1;
    }

    // Match a symbolic jump/branch destination against the next non-trivia label or function marker.
    static bool TargetsImmediatelyFollowingLabel(List<Expr> lines, int currentIndex, string mnemonic, AsmOperand operand)
    {
        if (!IsJumpOrBranch(mnemonic))
            return false;

        string label;
        if (!TryGetTargetLabel(mnemonic, operand, out label))
            return false;

        int nextIndex = FindNextSignificantIndex(lines, currentIndex + 1);
        if (nextIndex < 0) return false;

        string nextLabel;
        if (lines[nextIndex].Match(Tag.Label, out nextLabel) && nextLabel == label)
            return true;
        if (lines[nextIndex].Match(Tag.Function, out nextLabel) && nextLabel == label)
            return true;

        return false;
    }

    // Find the next non-comment/non-section node, or -1 at the end.
    static int FindNextSignificantIndex(List<Expr> lines, int startIndex)
    {
        for (int i = startIndex; i < lines.Count; i++)
        {
            if (!IsTriviaNode(lines[i]))
                return i;
        }
        return -1;
    }

    // Treat comments and section markers as skippable for local instruction/label searches.
    static bool IsTriviaNode(Expr expr)
    {
        return expr != null && (expr.MatchTag(Tag.Comment) || expr.MatchTag(Tag.Section));
    }

    // Recognize JMP and the eight ordinary conditional branches.
    static bool IsJumpOrBranch(string mnemonic)
    {
        if (mnemonic == "JMP") return true;
        return mnemonic == "BEQ" || mnemonic == "BNE" || mnemonic == "BCC" || mnemonic == "BCS" ||
               mnemonic == "BMI" || mnemonic == "BPL" || mnemonic == "BVC" || mnemonic == "BVS";
    }

    // Extract a zero-offset symbolic target from an absolute JMP or relative conditional branch.
    static bool TryGetTargetLabel(string mnemonic, AsmOperand operand, out string label)
    {
        label = null;
        if (operand == null || !operand.Base.HasValue || operand.Offset != 0)
            return false;

        if (mnemonic == "JMP" && operand.Mode == AddressMode.Absolute)
        {
            label = operand.Base.Value;
            return true;
        }

        if ((mnemonic == "BEQ" || mnemonic == "BNE" || mnemonic == "BCC" || mnemonic == "BCS" ||
             mnemonic == "BMI" || mnemonic == "BPL" || mnemonic == "BVC" || mnemonic == "BVS") &&
            operand.Mode == AddressMode.Relative)
        {
            label = operand.Base.Value;
            return true;
        }

        return false;
    }

    // Recognize adjacent opposite-direction transfers through A.
    static bool AreMirrorTransfers(string first, string second)
    {
        return (first == "TAX" && second == "TXA") ||
               (first == "TXA" && second == "TAX") ||
               (first == "TAY" && second == "TYA") ||
               (first == "TYA" && second == "TAY");
    }

    // Select repeatable flag setters, NOP and register transfers for adjacent duplicate removal.
    static bool IsDuplicateSafeInstruction(string mnemonic)
    {
        return mnemonic == "CLC" || mnemonic == "SEC" || mnemonic == "CLI" || mnemonic == "SEI" ||
               mnemonic == "CLD" || mnemonic == "SED" || mnemonic == "CLV" || mnemonic == "NOP" ||
               mnemonic == "TAX" || mnemonic == "TXA" || mnemonic == "TAY" || mnemonic == "TYA";
    }

    // Accept a symbolic absolute operand, including a nonzero displacement; no label resolution occurs here.
    static bool IsAbsoluteLabelOperand(AsmOperand operand)
    {
        return operand != null && operand.Mode == AddressMode.Absolute && operand.Base.HasValue;
    }

    // Test an unmodified numeric immediate after masking to its emitted byte width.
    static bool IsImmediateZero(AsmOperand operand)
    {
        return operand != null &&
               operand.Mode == AddressMode.Immediate &&
               operand.Modifier == ImmediateModifier.None &&
               !operand.Base.HasValue &&
               ((operand.Offset & 0xFF) == 0);
    }

    // Reset tracked state at ordinary jumps, calls, returns and conditional branches.
    static bool IsControlFlowOrCall(string mnemonic)
    {
        return mnemonic == "JMP" || mnemonic == "JSR" ||
               mnemonic == "RTS" || mnemonic == "RTI" ||
               mnemonic == "BEQ" || mnemonic == "BNE" || mnemonic == "BCC" || mnemonic == "BCS" ||
               mnemonic == "BMI" || mnemonic == "BPL" || mnemonic == "BVC" || mnemonic == "BVS";
    }

    // Recognize stores, memory increments/decrements and non-accumulator shifts/rotates.
    // Stack pushes are not classified as memory writes by this helper.
    static bool IsMemoryWrite(string mnemonic, AsmOperand operand)
    {
        if (mnemonic == "STA" || mnemonic == "STX" || mnemonic == "STY" || mnemonic == "INC" || mnemonic == "DEC")
            return true;

        if ((mnemonic == "ASL" || mnemonic == "LSR" || mnemonic == "ROL" || mnemonic == "ROR") &&
            operand != null && operand.Mode != AddressMode.Implicit)
            return true;

        return false;
    }

    // Recognize unconditional JMP/RTS/RTI fallthrough terminators.
    static bool IsTerminator(string mnemonic)
    {
        return mnemonic == "JMP" || mnemonic == "RTS" || mnemonic == "RTI";
    }

    // Drop fallthrough assembly and intervening comments/sections after a terminator until a label/function
    // or another non-assembly node resumes output; this is not whole-program reachability.
    static List<Expr> RemoveUnreachable(List<Expr> lines)
    {
        var outLines = new List<Expr>(lines.Count);
        bool skipping = false;

        foreach (var e in lines)
        {
            if (skipping)
            {
                if (e.MatchTag(Tag.Label) || e.MatchTag(Tag.Function))
                {
                    skipping = false;
                    outLines.Add(e);
                    continue;
                }

                if (e.MatchTag(Tag.Comment) || e.MatchTag(Tag.Section))
                    continue;

                if (!e.MatchTag(Tag.Asm))
                {
                    skipping = false;
                    outLines.Add(e);
                    continue;
                }

                continue;
            }

            outLines.Add(e);

            string mnemonic;
            AsmOperand operand;
            if (e.Match(Tag.Asm, out mnemonic, out operand) && IsTerminator(mnemonic))
                skipping = true;
        }

        return outLines;
    }
}
