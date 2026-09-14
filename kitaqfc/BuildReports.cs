using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;

// Capture a function's placement constraints and argument/return widths for ABI reports.
sealed class FunctionAbiInfo
{
    public string Name;
    public int Bank;
    public bool HasFixedBank;
    public int PlacementOrder;
    public bool HasFixedOrder;
    public bool IsPrototype;
    public bool IsInline;
    public bool IsStackCall;
    public bool IsFastCall;
    public int ReturnSize;
    public int[] ParamSizes = new int[0];
}

// Track requested versus actual ROM placement and final byte ranges for read-only data.
sealed class ReadonlyDataInfo
{
    public string Name;
    public int Bank;
    public int RequestedBank;
    public bool HasFixedBank;
    public int PlacementOrder;
    public bool HasFixedOrder;
    public int StartFileOffset;
    public int EndFileOffset;
    public int SizeBytes;
    public int CpuAddress;
}

// Aggregate one caller/callee relationship, including bank-crossing strategy and
// the latest observed argument-size/source details used by report generation.
sealed class CallEdgeInfo
{
    public string Caller;
    public string Callee;
    public int CallerBank;
    public int CalleeBank;
    public string Kind;
    public bool ViaThunk;
    public bool ViaFarcall;
    public int Count;
    public int[] LastActualArgSizes = new int[0];
    public int[] LastExpectedArgSizes = new int[0];
    public string LastSource;
}

// Describe a generated NES operation and its timing, PPU/OAM and mapper/FDS
// requirements so tools can connect compiler intent to runtime observations.
sealed class NesActionUseInfo
{
    public string Name;
    public string Category;
    public string Operation;
    public string KurosakiKind;
    public string Caller;
    public int CallerBank;
    public string Source;
    public string Timing;
    public bool DirectPpuAccess;
    public bool QueuePpuAccess;
    public bool UsesOamShadow;
    public bool PerformsOamDma;
    public bool RequiresFds;
    public string MapperRequirement;
    public string Note;
}

// Describe a selected restart vector, its target and estimated net code-size saving.
sealed class RstSelectionInfo
{
    public int Vector;
    public string TargetLabel;
    public int Calls;
    public int NetBytes;
}

// Record a zero-page allocation and the allocator's stated reason for assigning it.
sealed class ZpAllocationInfo
{
    public string Name;
    public string Kind;
    public int Address;
    public int Size;
    public string Reason;
}

// Identify the RAM slot assigned to a function-local static frame value.
sealed class StaticFrameSlotInfo
{
    public string Function;
    public string Name;
    public int Address;
    public int Size;
    public string Region;
}

// Describe allocated RAM and whether the reported allocation is conservative.
sealed class RamAllocationInfo
{
    public string Name;
    public string Kind;
    public int Address;
    public int Size;
    public string Region;
    public bool Conservative;
}

// Record an emitted memory operation, its known span/region and whether its target is dynamic.
sealed class RamAccessInfo
{
    public string Function;
    public int Bank;
    public string Operation;
    public int Address;
    public int Span;
    public string Region;
    public bool DynamicTarget;
}

// Retain both applied and rejected inline decisions together with their reasons.
sealed class InlineDecisionInfo
{
    public string Caller;
    public string Callee;
    public string Kind;
    public string Reason;
    public bool Applied;
}

// Record the loop variable, limit and selected lowering strategy with source context.
sealed class LoopLoweringInfo
{
    public string Function;
    public string Variable;
    public int Limit;
    public string Kind;
    public string Source;
}

// Explain why a function was removed by whole-program reachability/optimization work.
sealed class LtoRemovalInfo
{
    public string Name;
    public string Reason;
}

// Record a bank assignment, its rationale and the hotness value considered during placement.
sealed class BankPlacementInfo
{
    public string Name;
    public int Bank;
    public string Reason;
    public int Hotness;
}

// Collect facts and decisions emitted by code generation; this object stores
// report data and does not itself validate or execute the generated ROM.
sealed class CodegenAnalysisReport
{
    public readonly List<FunctionAbiInfo> Functions = new List<FunctionAbiInfo>();
    public readonly List<ReadonlyDataInfo> ReadonlyData = new List<ReadonlyDataInfo>();
    public readonly List<CallEdgeInfo> Calls = new List<CallEdgeInfo>();
    public readonly List<NesActionUseInfo> NesActions = new List<NesActionUseInfo>();
    public readonly List<RstSelectionInfo> RstSelections = new List<RstSelectionInfo>();
    public readonly List<ZpAllocationInfo> ZpAllocations = new List<ZpAllocationInfo>();
    public readonly List<StaticFrameSlotInfo> StaticFrameSlots = new List<StaticFrameSlotInfo>();
    public readonly List<RamAllocationInfo> RamAllocations = new List<RamAllocationInfo>();
    public readonly List<RamAccessInfo> RamAccesses = new List<RamAccessInfo>();
    public readonly List<InlineDecisionInfo> InlineDecisions = new List<InlineDecisionInfo>();
    public readonly List<LoopLoweringInfo> LoopLowerings = new List<LoopLoweringInfo>();
    public readonly List<LtoRemovalInfo> LtoRemovedFunctions = new List<LtoRemovalInfo>();
    public readonly List<BankPlacementInfo> BankPlacements = new List<BankPlacementInfo>();
    public readonly List<string> AbiIssues = new List<string>();
    public int CgbRuntimeCheckCount;
    public int CgbGuardedWriteCount;
    public readonly List<string> CgbGuardedRegisters = new List<string>();
}

// Record before/after line counts and the textual diff for one optimizer pass.
sealed class OptimizerPassReport
{
    public string Name;
    public int BeforeLines;
    public int AfterLines;
    public int ChangedLines;
    public int AddedLines;
    public int RemovedLines;
    public string DiffText;
}

// Collect pass results and per-vector restart rewrite counts from an optimization run.
sealed class OptimizerAnalysisReport
{
    public readonly List<OptimizerPassReport> Passes = new List<OptimizerPassReport>();
    public readonly Dictionary<int, int> RstRewriteCountsByVector = new Dictionary<int, int>();
    public int TotalRstRewrites;
}

// Keep file offsets distinct from CPU addresses when reporting placed function sizes.
sealed class FunctionSizeInfo
{
    public string Name;
    public int StartFileOffset;
    public int EndFileOffset;
    public int SizeBytes;
    public int Bank;
    public int CpuAddress;
}

// Store final layout measurements and metadata produced by assembly for downstream reports.
sealed class AssemblerAnalysisReport
{
    public readonly List<FunctionSizeInfo> FunctionSizes = new List<FunctionSizeInfo>();
    public readonly List<ReadonlyDataInfo> ReadonlyData = new List<ReadonlyDataInfo>();
    public int[] BankMaxPc = new int[0];
    public int RomSizeBytes;
    // Use -1 when separate PRG/CHR size measurements have not been supplied.
    public int PrgRomSizeBytes = -1;
    public int ChrRomSizeBytes = -1;
    public int UsedBytes;
    public bool CommonReplicaEqual;
    public string CommonReplicaSha256 = "";
}

static class BuildReportUtil
{
    // Hash the supplied bytes as lowercase hexadecimal; null is treated as an empty byte sequence.
    public static string ComputeSha256Hex(byte[] data)
    {
        if (data == null) data = new byte[0];
        using (var sha = SHA256.Create())
        {
            var hash = sha.ComputeHash(data);
            return string.Concat(hash.Select(b => b.ToString("x2")));
        }
    }

    // Return an empty string for a missing/empty path; otherwise read and hash the
    // file. Existing-file read failures propagate rather than becoming a fake digest.
    public static string ComputeSha256HexOfFile(string path)
    {
        if (string.IsNullOrEmpty(path) || !File.Exists(path)) return "";
        return ComputeSha256Hex(File.ReadAllBytes(path));
    }

    // Apply the legacy 16 KiB fixed/switchable-window address convention: bank zero
    // is unchanged and later offsets map into 0x4000-0x7FFF. This helper is not a
    // general mapper-aware NES address translator; negative sentinel values pass through.
    public static int CpuAddrFromFileOffset(int fileOffset)
    {
        const int bankSize = 0x4000;
        if (fileOffset < 0) return fileOffset;
        if (fileOffset < bankSize) return fileOffset;
        return bankSize + (fileOffset & (bankSize - 1));
    }

    // Format an optional integer array as a comma-separated report field; empty input yields an empty string.
    public static string JoinInts(int[] values)
    {
        if (values == null || values.Length == 0) return "";
        return string.Join(",", values.Select(x => x.ToString()));
    }
}
