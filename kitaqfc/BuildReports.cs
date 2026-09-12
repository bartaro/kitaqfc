using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;

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

sealed class RstSelectionInfo
{
    public int Vector;
    public string TargetLabel;
    public int Calls;
    public int NetBytes;
}

sealed class ZpAllocationInfo
{
    public string Name;
    public string Kind;
    public int Address;
    public int Size;
    public string Reason;
}

sealed class StaticFrameSlotInfo
{
    public string Function;
    public string Name;
    public int Address;
    public int Size;
    public string Region;
}

sealed class RamAllocationInfo
{
    public string Name;
    public string Kind;
    public int Address;
    public int Size;
    public string Region;
    public bool Conservative;
}

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

sealed class InlineDecisionInfo
{
    public string Caller;
    public string Callee;
    public string Kind;
    public string Reason;
    public bool Applied;
}

sealed class LoopLoweringInfo
{
    public string Function;
    public string Variable;
    public int Limit;
    public string Kind;
    public string Source;
}

sealed class LtoRemovalInfo
{
    public string Name;
    public string Reason;
}

sealed class BankPlacementInfo
{
    public string Name;
    public int Bank;
    public string Reason;
    public int Hotness;
}

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

sealed class OptimizerAnalysisReport
{
    public readonly List<OptimizerPassReport> Passes = new List<OptimizerPassReport>();
    public readonly Dictionary<int, int> RstRewriteCountsByVector = new Dictionary<int, int>();
    public int TotalRstRewrites;
}

sealed class FunctionSizeInfo
{
    public string Name;
    public int StartFileOffset;
    public int EndFileOffset;
    public int SizeBytes;
    public int Bank;
    public int CpuAddress;
}

sealed class AssemblerAnalysisReport
{
    public readonly List<FunctionSizeInfo> FunctionSizes = new List<FunctionSizeInfo>();
    public readonly List<ReadonlyDataInfo> ReadonlyData = new List<ReadonlyDataInfo>();
    public int[] BankMaxPc = new int[0];
    public int RomSizeBytes;
    public int PrgRomSizeBytes = -1;
    public int ChrRomSizeBytes = -1;
    public int UsedBytes;
    public bool CommonReplicaEqual;
    public string CommonReplicaSha256 = "";
}

static class BuildReportUtil
{
    public static string ComputeSha256Hex(byte[] data)
    {
        if (data == null) data = new byte[0];
        using (var sha = SHA256.Create())
        {
            var hash = sha.ComputeHash(data);
            return string.Concat(hash.Select(b => b.ToString("x2")));
        }
    }

    public static string ComputeSha256HexOfFile(string path)
    {
        if (string.IsNullOrEmpty(path) || !File.Exists(path)) return "";
        return ComputeSha256Hex(File.ReadAllBytes(path));
    }

    public static int CpuAddrFromFileOffset(int fileOffset)
    {
        const int bankSize = 0x4000;
        if (fileOffset < 0) return fileOffset;
        if (fileOffset < bankSize) return fileOffset;
        return bankSize + (fileOffset & (bankSize - 1));
    }

    public static string JoinInts(int[] values)
    {
        if (values == null || values.Length == 0) return "";
        return string.Join(",", values.Select(x => x.ToString()));
    }
}
