using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;


class AggregateInfo
{
    public readonly AggregateLayout Layout;
    public readonly int TotalSize;
    public readonly int Alignment;
    public readonly bool IsPacked;
    public readonly FieldInfo[] Fields;

    public AggregateInfo(AggregateLayout layout, int totalSize, int alignment, bool isPacked, FieldInfo[] fields)
    {
        Layout = layout;
        TotalSize = totalSize;
        Alignment = alignment;
        IsPacked = isPacked;
        Fields = fields;
    }
}

enum AggregateLayout { Struct, Union }

class FieldInfo
{
    public readonly CType Type;
    public readonly string Name;
    public readonly int Offset;

    public FieldInfo(CType type, string name, int offset)
    {
        Type = type;
        Name = name;
        Offset = offset;
    }

    public override string ToString() => string.Format("Field: {0}, {1}, {2}", Offset, Type.Show(), Name);
}

[DebuggerDisplay("{Show(),nq}")]
class CFunctionInfo
{
    public FieldInfo[] Parameters;
    public Symbol[] ParameterSymbols;
    public CType ReturnType;
    public bool IsFastCall;
    public bool IsStackCall;
    public bool IsPrototype;

    public bool MustCheck;

    public int RomBank = 1;
    public bool HasFixedBank;
    public int PlacementOrder = int.MaxValue;
    public bool HasFixedOrder;

    public bool IsInline;
    public Expr Body;

    public string Show()
    {
        var paramTypes = Parameters.Select(x => string.Format("{0} {1}", x.Type.Show(), x.Name));
        string attr = (IsStackCall ? " [stackcall]" : "") + (IsFastCall ? " [fastcall]" : "");
        if (MustCheck) attr += " [must_check]";
        return string.Format("function({0}) {1}{2}", string.Join(", ", paramTypes), ReturnType.Show(), attr);
    }
}

struct MemoryRegion
{
    public readonly MemoryRegionTag Tag;
    public readonly int FixedAddress;
    public readonly int WramBank;

    MemoryRegion(MemoryRegionTag tag, int address, int wramBank = 0)
    {
        Tag = tag;
        FixedAddress = address;
        WramBank = wramBank;
    }

    public static readonly MemoryRegion HighMem = new MemoryRegion(MemoryRegionTag.HighMem, 0);
    public static readonly MemoryRegion Oam = new MemoryRegion(MemoryRegionTag.Oam, 0);
    public static readonly MemoryRegion Ram = new MemoryRegion(MemoryRegionTag.Ram, 0);
    public static readonly MemoryRegion Wram0 = new MemoryRegion(MemoryRegionTag.Wram0, 0);
    public static readonly MemoryRegion WramX = new MemoryRegion(MemoryRegionTag.WramX, 0);
    public static readonly MemoryRegion ProgramRom = new MemoryRegion(MemoryRegionTag.ProgramRom, 0);
    public static MemoryRegion Fixed(int address) => new MemoryRegion(MemoryRegionTag.Fixed, address);
    public static MemoryRegion WramXBank(int bank) => new MemoryRegion(MemoryRegionTag.WramX, 0, bank);

    public override string ToString()
    {
        if (Tag == MemoryRegionTag.Fixed) return string.Format("{0}=${1:X4}", Tag, FixedAddress);
        if (Tag == MemoryRegionTag.WramX && WramBank > 0) return string.Format("{0}[{1}]", Tag, WramBank);
        return Tag.ToString();
    }
}

// Memory placement tags.
// Ram = "default WRAM (prefers WRAM0, may fall back to WRAMX depending on allocator policy)"
// Wram0 = force fixed WRAM0 (0xC000-0xCFFF)
// WramX = force bankable WRAMX (0xD000-0xDFFF)
enum MemoryRegionTag { HighMem, Oam, Ram, Wram0, WramX, ProgramRom, Fixed }

class Symbol
{
    public readonly SymbolTag Tag;
    public readonly int Value;
    public readonly CType Type;
    public readonly string Name;
    public readonly int WramBank;

    public Symbol(SymbolTag tag, int value, CType type, string name, int wramBank = 0)
    {
        Tag = tag;
        Value = value;
        Type = type;
        Name = name;
        WramBank = wramBank;
    }
}

enum SymbolTag { Constant, ReadonlyData, Global, Local, StackParam }

class LoopScope
{
    public LoopScope Outer;
    public AsmOperand ContinueLabel;
    public AsmOperand BreakLabel;
}

class LexicalScope
{
    public readonly LexicalScope Outer;
    public readonly Dictionary<string, Symbol> Symbols = new Dictionary<string, Symbol>();
    public int SavedHramNext;
    public int SavedWram0Next;
    public int SavedWram1Next;
    public bool PreserveAllocations = false;
    public LexicalScope(LexicalScope outer) { Outer = outer; }
}

class AllocationRegion
{
    public readonly string Name;
    public readonly int Bottom;
    public readonly int Top;
    public int Next;

    public AllocationRegion(string name, int bottom, int top)
    {
        Name = name;
        Bottom = bottom;
        Next = bottom;
        Top = top;
    }
}

class OutputTransaction
{
    public List<Expr> Lines = new List<Expr>();
    public bool SpeculationError = false;
    public string AbortReason = "";
    public Register Reserved = Register.None;
}

[Flags]
enum Register { None = 0, A = 1, B = 2, C = 4, D = 8, E = 16, H = 32, L = 64 }

class WideOperand
{
    public AsmOperand Low, High;
    public WideOperand(AsmOperand low, AsmOperand high)
    {
        Low = low;
        High = high;
    }
}

[DebuggerDisplay("{Show(),nq}")]
class AsmOperand
{
    public readonly Maybe<string> Base = Maybe.Nothing;
    public readonly int Offset = 0;
    public readonly AddressMode Mode;
    public readonly ImmediateModifier Modifier;
    public readonly string Comment;
    public static readonly AsmOperand Implicit = new AsmOperand(0, AddressMode.Implicit);

    public AsmOperand(string actualBase, AddressMode mode) : this(actualBase, 0, mode, ImmediateModifier.None) { }
    public AsmOperand(string actualBase, ImmediateModifier modifier) : this(actualBase, 0, AddressMode.Immediate, modifier) { }
    public AsmOperand(int value, AddressMode mode) : this(Maybe.Nothing, value, mode, ImmediateModifier.None) { }
    public AsmOperand(int value, ImmediateModifier modifier) : this(Maybe.Nothing, value, AddressMode.Immediate, modifier) { }
    public AsmOperand(string actualBase, int offset, AddressMode mode, ImmediateModifier modifier) : this(Maybe.Just(actualBase), offset, mode, modifier) { }

    public AsmOperand(Maybe<string> optionalBase, int offset, AddressMode mode, ImmediateModifier modifier, string comment = null)
    {
        Base = optionalBase;
        Offset = offset;
        Mode = mode;
        Modifier = modifier;
        Comment = comment;
    }

    public AsmOperand WithMode(AddressMode newMode) => new AsmOperand(Base, Offset, newMode, Modifier, Comment);
    public AsmOperand WithModifier(ImmediateModifier newModifier) => new AsmOperand(Base, Offset, Mode, newModifier, Comment);
    public AsmOperand WithComment(string newComment) => new AsmOperand(Base, Offset, Mode, Modifier, newComment);
    public AsmOperand WithComment(string newCommentFormat, params object[] args) => WithComment(string.Format(newCommentFormat, args));

    public AsmOperand ReplaceBase(int baseValue)
    {
        if (!Base.HasValue) throw new Exception("this operand has no base symbol");
        return new AsmOperand(Maybe.Nothing, baseValue + Offset, Mode, Modifier, Comment);
    }

    public string Show()
    {
        string s;
        if (!Base.HasValue) s = Program.FormatAssemblyInteger(Offset);
        else if (Offset == 0) s = Base.Value;
        else s = string.Format("{0}+{1}", Base.Value, Program.FormatAssemblyInteger(Offset));

        if (Modifier == ImmediateModifier.LowByte) s = "LOW(" + s + ")";
        else if (Modifier == ImmediateModifier.HighByte) s = "HIGH(" + s + ")";
        else if (Modifier == ImmediateModifier.Bank) s = "BANK(" + s + ")";

        if (Mode == AddressMode.Immediate || Mode == AddressMode.Immediate16) return s;
        if (Mode == AddressMode.Absolute) return "[" + s + "]";
        if (Mode == AddressMode.AbsoluteX) return "[" + s + "+X]";
        if (Mode == AddressMode.AbsoluteY) return "[" + s + "+Y]";
        if (Mode == AddressMode.Indirect) return "[HL]";
        if (Mode == AddressMode.IndirectX) return "[" + s + "+X]";
        if (Mode == AddressMode.IndirectY) return "[" + s + "+Y]";
        if (Mode == AddressMode.HighMemX) return "[" + s + "+X]";
        if (Mode == AddressMode.HighMemY) return "[" + s + "+Y]";
        return s;
    }
}

enum AddressMode
{
    Implicit,
    Immediate,
    Immediate16,
    HighMem,
    HighMemX,
    HighMemY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
}

enum ImmediateModifier
{
    None,
    LowByte,
    HighByte,
    Bank,
}
