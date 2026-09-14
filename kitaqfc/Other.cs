using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;


// Store computed aggregate layout and a borrowed field array; the model does not recalculate or validate offsets.
class AggregateInfo
{
    public readonly AggregateLayout Layout;
    public readonly int TotalSize;
    public readonly int Alignment;
    public readonly bool IsPacked;
    public readonly FieldInfo[] Fields;

    // Retain the provided layout result and field references as supplied.
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

// Describe a field or parameter by type/name/offset without owning its type object.
class FieldInfo
{
    public readonly CType Type;
    public readonly string Name;
    public readonly int Offset;

    // Store a field descriptor; offset meaning and validation belong to the consuming compiler phase.
    public FieldInfo(CType type, string name, int offset)
    {
        Type = type;
        Name = name;
        Offset = offset;
    }

    // Show the offset, diagnostic type text and field name for inspection.
    public override string ToString() => string.Format("Field: {0}, {1}, {2}", Offset, Type.Show(), Name);
}

[DebuggerDisplay("{Show(),nq}")]
// Collect mutable function signature, calling convention, placement and optional body metadata.
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

    // Format parameter types and selected calling-convention attributes; placement and body details are omitted.
    public string Show()
    {
        var paramTypes = Parameters.Select(x => string.Format("{0} {1}", x.Type.Show(), x.Name));
        string attr = (IsStackCall ? " [stackcall]" : "") + (IsFastCall ? " [fastcall]" : "");
        if (MustCheck) attr += " [must_check]";
        return string.Format("function({0}) {1}{2}", string.Join(", ", paramTypes), ReturnType.Show(), attr);
    }
}

// Describe requested placement, optionally with a fixed address or bank. These descriptors do not allocate memory.
struct MemoryRegion
{
    public readonly MemoryRegionTag Tag;
    public readonly int FixedAddress;
    public readonly int WramBank;

    // Store the requested region and optional address/bank without range validation.
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
    // Build a fixed-address request without checking address-space availability.
    public static MemoryRegion Fixed(int address) => new MemoryRegion(MemoryRegionTag.Fixed, address);
    // Build a banked-memory request; the target allocator interprets and validates the bank.
    public static MemoryRegion WramXBank(int bank) => new MemoryRegion(MemoryRegionTag.WramX, 0, bank);

    // Display an explicit address for Fixed or a positive bank for WramX, otherwise the tag name.
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

// Store a tagged constant or storage reference; Value interpretation depends on the symbol tag.
class Symbol
{
    public readonly SymbolTag Tag;
    public readonly int Value;
    public readonly CType Type;
    public readonly string Name;
    public readonly int WramBank;

    // Retain symbol metadata and the shared type object without assigning storage.
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

// Link nested loops to their break/continue assembly targets.
class LoopScope
{
    public LoopScope Outer;
    public AsmOperand ContinueLabel;
    public AsmOperand BreakLabel;
}

// Keep local symbol bindings, the outer scope and allocator checkpoints for scope exit.
class LexicalScope
{
    public readonly LexicalScope Outer;
    public readonly Dictionary<string, Symbol> Symbols = new Dictionary<string, Symbol>();
    public int SavedHramNext;
    public int SavedWram0Next;
    public int SavedWram1Next;
    public bool PreserveAllocations = false;
    // Create an empty binding scope linked to its supplied parent.
    public LexicalScope(LexicalScope outer) { Outer = outer; }
}

// Track a named allocation interval and its next free cursor; allocation policy is implemented elsewhere.
class AllocationRegion
{
    public readonly string Name;
    public readonly int Bottom;
    public readonly int Top;
    public int Next;

    // Initialize the next cursor at the region bottom without validating the interval.
    public AllocationRegion(string name, int bottom, int top)
    {
        Name = name;
        Bottom = bottom;
        Next = bottom;
        Top = top;
    }
}

// Buffer speculative expression output together with its failure reason and reserved-register mask.
class OutputTransaction
{
    public List<Expr> Lines = new List<Expr>();
    public bool SpeculationError = false;
    public string AbortReason = "";
    public Register Reserved = Register.None;
}

[Flags]
enum Register { None = 0, A = 1, B = 2, C = 4, D = 8, E = 16, H = 32, L = 64 }

// Pair low/high assembly operands for a wider value; the referenced operands are not cloned.
class WideOperand
{
    public AsmOperand Low, High;
    // Retain the low and high operand references in byte order.
    public WideOperand(AsmOperand low, AsmOperand high)
    {
        Low = low;
        High = high;
    }
}

[DebuggerDisplay("{Show(),nq}")]
// Represent an optional symbol plus integer offset, addressing mode and extraction modifier.
// The display syntax is internal and is not necessarily valid external 6502 assembler syntax.
class AsmOperand
{
    public readonly Maybe<string> Base = Maybe.Nothing;
    public readonly int Offset = 0;
    public readonly AddressMode Mode;
    public readonly ImmediateModifier Modifier;
    public readonly string Comment;
    public static readonly AsmOperand Implicit = new AsmOperand(0, AddressMode.Implicit);

    // Construct a symbolic operand with zero displacement and no extraction modifier.
    public AsmOperand(string actualBase, AddressMode mode) : this(actualBase, 0, mode, ImmediateModifier.None) { }
    // Construct an immediate symbolic operand with the selected extraction modifier.
    public AsmOperand(string actualBase, ImmediateModifier modifier) : this(actualBase, 0, AddressMode.Immediate, modifier) { }
    // Construct a literal operand in the requested addressing mode.
    public AsmOperand(int value, AddressMode mode) : this(Maybe.Nothing, value, mode, ImmediateModifier.None) { }
    // Construct a modified immediate literal without a base symbol.
    public AsmOperand(int value, ImmediateModifier modifier) : this(Maybe.Nothing, value, AddressMode.Immediate, modifier) { }
    // Promote a non-null symbol name to an optional base and retain the displacement.
    public AsmOperand(string actualBase, int offset, AddressMode mode, ImmediateModifier modifier) : this(Maybe.Just(actualBase), offset, mode, modifier) { }

    // Store all operand fields directly; range and instruction-mode validation occur during assembly.
    public AsmOperand(Maybe<string> optionalBase, int offset, AddressMode mode, ImmediateModifier modifier, string comment = null)
    {
        Base = optionalBase;
        Offset = offset;
        Mode = mode;
        Modifier = modifier;
        Comment = comment;
    }

    // Copy the operand with a new addressing mode and preserve other fields.
    public AsmOperand WithMode(AddressMode newMode) => new AsmOperand(Base, Offset, newMode, Modifier, Comment);
    // Copy the operand with a new extraction modifier.
    public AsmOperand WithModifier(ImmediateModifier newModifier) => new AsmOperand(Base, Offset, Mode, newModifier, Comment);
    // Copy the operand with a replacement optional comment.
    public AsmOperand WithComment(string newComment) => new AsmOperand(Base, Offset, Mode, Modifier, newComment);
    // Format a comment and delegate to the copying overload.
    public AsmOperand WithComment(string newCommentFormat, params object[] args) => WithComment(string.Format(newCommentFormat, args));

    // Require a symbolic base, add its resolved value to the displacement and return a literal operand.
    public AsmOperand ReplaceBase(int baseValue)
    {
        if (!Base.HasValue) throw new Exception("this operand has no base symbol");
        return new AsmOperand(Maybe.Nothing, baseValue + Offset, Mode, Modifier, Comment);
    }

    // Render the base/displacement, then extraction wrapper and address-mode brackets.
    // Indirect currently displays [HL]; the text is a legacy diagnostic representation.
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
