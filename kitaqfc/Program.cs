using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

// Global compile-time configuration options.
public static class Config
{
    public static readonly bool ShowSourcePositionInDebugOutput = true;
}

static partial class Program
{
    // CLI settings live in static process state and start with the defaults declared here.
    public static bool EnableDebugOutput { get; private set; } = false;
    public static bool DisableDisasm { get; private set; } = false;
    public static bool EnableIncrementalCache { get; private set; } = true;
    public static bool EmitDiagJson { get; private set; } = false;
    public static string DiagJsonPath { get; private set; } = "kitaqfc.diag.json";
    public static bool EmitDependenciesList { get; private set; } = false;
    public static string DependenciesListPath { get; private set; } = "";
    public enum DiagnosticMode { Permissive, Strict }
    public static DiagnosticMode DiagnosticsMode { get; private set; } = DiagnosticMode.Permissive;
    public static bool StrictDiagnostics => DiagnosticsMode == DiagnosticMode.Strict;
    public static readonly List<string> IncludeDirectories = new List<string>();
    public static IReadOnlyList<string> LastCompilationDependencies => _lastCompilationDependencies;
    static readonly List<string> _lastCompilationDependencies = new List<string>();
    static string _cacheKey = "";
    static string[] _originalArgs = new string[0];
    static bool _autoMinimizeOnFail = false;
    static bool _autoMinimizeTriggered = false;
    static bool _reproPackageWritten = false;
    static string _outputFilenameForDiag = "out.nes";
    // Keep the selected backend, cartridge options and optional external CHR input separate from GB header compatibility fields.
    public static CompilationTargetInfo TargetInfo { get; private set; } = CompilationTargetInfo.Nes;
    public static bool IsNesTarget => TargetInfo.Kind == CompilationTargetKind.Nes;
    public static string NesChrRomPath { get; private set; } = "";
    public static NesCartridgeOptions NesCartridge { get; private set; } = new NesCartridgeOptions();
    // FDS packaging combines optional input metadata with output-container and overlay settings below.
    public static FdsDiskMetadata FdsMetadata { get; private set; } = FdsDiskMetadata.Empty;
    public static string FdsMetadataPath { get; private set; } = "";
    public static string FdsMetadataOutPath { get; private set; } = "";
    public enum NesOutputContainerFormat { Auto, INes, Fds }
    public static NesOutputContainerFormat OutputContainerFormat { get; private set; } = NesOutputContainerFormat.Auto;
    public static bool EmitFdsSidecar { get; private set; } = false;
    public static string FdsImageOutputPath { get; private set; } = "";
    public static bool FdsImageHeaderEnabled { get; private set; } = true;
    public static string FdsGameCode { get; private set; } = "KQF";
    public static bool FdsLicenseBypassEnabled { get; private set; } = true;
    public static bool FdsPrgRamLayoutEnabled { get; private set; } = true;
    public static bool FdsAutoOverlayEnabled { get; private set; } = true;
    public static int FdsOverlayStartId { get; private set; } = 32;
    public static string FdsOverlayNamePrefix { get; private set; } = "KQFB";
    public static bool FdsOverlayTrimTrailingFill { get; private set; } = false;
    public static bool FdsOverlayFarcallEnabled { get; private set; } = true;
    public static bool FdsOverlayResidencyGuardEnabled { get; private set; } = true;
    public static bool FdsOverlayFunctionTableEnabled { get; private set; } = true;
    public static bool EmitKurosakiMetadata { get; private set; } = false;
    public static string KurosakiMetadataPath { get; private set; } = "";
    public static NesCartridgeProfile NesMapperProfile => (NesCartridge ?? new NesCartridgeOptions()).Profile;
    public static bool RstUse38 { get; private set; } = false;
    // RST hot-call compression is OFF by default.
    // Enable explicitly with --rst-enable / --rst-on.
    public static bool RstDisable { get; private set; } = true;
    public static bool ConstScalarInRom { get; private set; } = false;
    // RST hot-call compression tuning
    public static int RstMaxCalls { get; private set; } = int.MaxValue;
    // 0 means "use all available vectors" (7 or 8 depending on --rst-use-38).
    public static int RstMaxVectors { get; private set; } = 0;
    public static readonly System.Collections.Generic.HashSet<string> RstExcludeLabels = new System.Collections.Generic.HashSet<string>(StringComparer.Ordinal);
    public static bool RstSpeedSafe { get; private set; } = false;
    // Conservative mode is ON by default. It avoids RST mapping for timing/IO-sensitive routines.
    // Use --rst-unsafe to force aggressive historical behavior.
    public static bool RstUnsafe { get; private set; } = false;
    private static bool AttachDebuggerOnError = false;
    static readonly List<DiagnosticEntry> Diagnostics = new List<DiagnosticEntry>();
    static bool SuppressConsoleDiagnostics = false;

    // Capture diagnostic counters and list length so speculative work can restore its prior diagnostic state.
    public struct DiagnosticSnapshot
    {
        public int ErrorCount;
        public int WarningCount;
        public int DiagnosticCount;
    }

    // --- Multiple-error collection ---
    // We keep compiling until we hit MaxErrors, then abort.
    public static int ErrorCount { get; private set; } = 0;
    public static int WarningCount { get; private set; } = 0;
    public static int MaxErrors { get; private set; } = 20;

    // Cache of source files for richer diagnostics.
    // Key: normalized filename as passed to the compiler.
    static readonly Dictionary<string, string[]> SourceCache = new Dictionary<string, string[]>(StringComparer.Ordinal);

    public static string DebugOutputPath = "debug_output";

    // --- Trace (compile pipeline dump) ---
    // --trace[=tokens,ast,ir,asm] dumps intermediate forms to TraceOutputPath.
    public static bool TraceEnabled { get; private set; } = false;
    public static string TraceOutputPath { get; private set; } = "trace_output";
    static readonly HashSet<string> TraceStages = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
    // Optional: disassemble functions selected from source files changed against a Git base.
    static bool EmitChangedFunctionDisasm { get; set; } = false;
    static string ChangedFunctionDisasmBaseRef { get; set; } = "";
    static string ChangedFunctionDisasmOutPath { get; set; } = "";
    static readonly Dictionary<string, int> FunctionBankOverrides = new Dictionary<string, int>(StringComparer.Ordinal);
    static readonly Dictionary<string, int> ReadonlyDataBankOverrides = new Dictionary<string, int>(StringComparer.Ordinal);

    // --- Variable list output (vlist) ---
    // vlist / --vlist emits a variable table after assembly.
    public static bool EmitVarList { get; private set; } = false;
    public static string VarListOutputPath { get; private set; } = ""; // if empty, use <out>.vlist.txt

    // --- Lightweight optimization level ---
    // -O0: off (baseline)
    // -O1: cheap but effective passes (peepholes, constant-control-flow pruning, etc.)
    public static int OptLevel { get; private set; } = 0;

    // --- Optional safety checks controlled by command-line settings ---
    // -Zcheck : enable all safety checks
    // -Zcheck-bounds : enable only fixed-array bounds checks for a[i]
    public static bool CheckBounds { get; private set; } = false;
    public static bool CheckMemCopy { get; private set; } = false;
    public static bool CheckStack { get; private set; } = false;
    public static bool CheckBankCalls { get; private set; } = false;
    // Slice bounds checks are enabled only by -Zcheck (not by -Zcheck-bounds).
    public static bool CheckSliceBounds { get; private set; } = false;

    // --- Calling convention / ABI ---
    public enum AbiMode { Legacy, Stack, FastCall }
    public static AbiMode Abi { get; private set; } = AbiMode.Legacy;
    public static bool AbiStack => Abi == AbiMode.Stack;
    // The register-convention flag also enables this predicate, independently of the enum value.
    public static bool AbiFastCall => Abi == AbiMode.FastCall || EnableRegisterCallingConventionV2;

    // --- Analysis / report options ---
    public static bool EmitBankSimReport { get; private set; } = false;
    public static string BankSimReportPath { get; private set; } = "";
    public static bool EmitFarcallSuggestionReport { get; private set; } = false;
    public static string FarcallSuggestionReportPath { get; private set; } = "";
    public static bool EmitCrossBankCallReport { get; private set; } = false;
    public static string CrossBankCallReportPath { get; private set; } = "";
    public static bool EmitAbiVerifyReport { get; private set; } = false;
    public static string AbiVerifyReportPath { get; private set; } = "";
    public static bool EmitAbiDiffReport { get; private set; } = false;
    public static string AbiDiffReportPath { get; private set; } = "";
    public static bool EmitRstApplyReport { get; private set; } = false;
    public static string RstApplyReportPath { get; private set; } = "";
    public static bool EmitOptDiffReport { get; private set; } = false;
    public static string OptDiffReportPath { get; private set; } = "";
    public static bool EmitFunctionSizeReport { get; private set; } = false;
    public static string FunctionSizeReportPath { get; private set; } = "";
    public static bool EmitHotspotReport { get; private set; } = false;
    public static string HotspotReportPath { get; private set; } = "";

    // NES performance optimization set.  These are intentionally split so that
    // individual compiler experiments can be disabled when debugging output.
    public static bool EnableWholeProgramZpAllocator { get; private set; } = false;
    public static bool EnableRegisterCallingConventionV2 { get; private set; } = false;
    public static bool EnableStaticFrameAllocator { get; private set; } = false;
    public static bool EnableSmallFunctionAutoInline { get; private set; } = false;
    public static bool EnableLoopLoweringOptimizer { get; private set; } = false;
    public static bool EnableLibraryLtoLite { get; private set; } = false;
    public static bool EnableKurosakiProfileFeedback { get; private set; } = false;
    public static bool EnableMapperAwareBankPlacement { get; private set; } = false;
    public static bool EmitNesOptimizationReport { get; private set; } = false;
    public static string NesOptimizationReportPath { get; private set; } = "";
    public static string KurosakiProfilePath { get; private set; } = "";
    // Track whether a RAM override was supplied separately from its address and byte count.
    public static bool HasNesLocalRamWindow { get; private set; } = false;
    public static int NesLocalRamBase { get; private set; } = 0;
    public static int NesLocalRamLength { get; private set; } = 0;
    public static bool HasCustomNesTempRamWindow { get; private set; } = false;
    public static int NesTempRamBase { get; private set; } = 0x00D0;
    public static int NesTempRamLength { get; private set; } = 0x0030;
    static readonly Dictionary<string, int> KurosakiHotness = new Dictionary<string, int>(StringComparer.Ordinal);

    public static bool EnableReproCheck { get; private set; } = false;
    public static string ReproCheckReportPath { get; private set; } = "";
    public static bool EmitCgbConsistencyReport { get; private set; } = false;
    public static string CgbConsistencyReportPath { get; private set; } = "";
    public static bool EmitCgbSymbolVerifyReport { get; private set; } = false;
    public static string CgbSymbolVerifyReportPath { get; private set; } = "";
    // Build-failure repro package (--repro-pack[=<dir>])
    static bool EmitReproPackageOnFail { get; set; } = false;
    static string ReproPackagePath { get; set; } = "";

    // --- ROM header patching options (fix31_rom_header_patch) ---
    // Priority: CLI (incl. --rom-header JSON template) > #pragma rom_* > defaults.
    static RomHeaderOptions RomHeader = new RomHeaderOptions(); // CLI/JSON-specified
    static readonly RomHeaderOptions RomHeaderPragma = new RomHeaderOptions(); // #pragma rom_* specified
    static readonly List<CgbPaletteDefinition> CgbPalettePragmas = new List<CgbPaletteDefinition>();

    // Merge into a new options object so effective-header queries do not mutate pragma or CLI settings.
    static RomHeaderOptions BuildRequestedRomHeader()
    {
        var effective = new RomHeaderOptions();
        effective.MergeFrom(RomHeaderPragma, overwrite: true);
        effective.MergeFrom(RomHeader, overwrite: true);
        return effective;
    }

    // Fold the hardware query only for explicit DMG-only or CGB-only headers.
    // Dual-mode or unspecified headers need a runtime decision; value is valid only when this returns true.
    public static bool TryGetKnownCgbRuntimeValue(out int value)
    {
        value = 0;

        var effective = BuildRequestedRomHeader();
        if (!effective.CgbFlag.HasValue) return false;

        if (effective.CgbFlag.Value == 0xC0)
        {
            value = 1;
            return true;
        }

        if (effective.CgbFlag.Value == 0x00)
        {
            value = 0;
            return true;
        }

        return false;
    }

    // Require a known nonzero hardware value rather than treating every CGB-compatible header as CGB-only.
    public static bool IsCgbOnlyTargetRequested()
    {
        return TryGetKnownCgbRuntimeValue(out int value) && value != 0;
    }

    // Retain legacy palette values and their diagnostic source positions.
    sealed class CgbPaletteDefinition
    {
        public string Name;
        public int[] Colors;
        public FilePosition Position;
    }

    // Carry one pipeline timing and its display detail into the final trace summary.
    sealed class TraceStageStat
    {
        public string Name;
        public long ElapsedMs;
        public string Detail;
    }

    // Dispatch utility commands first; otherwise parse options and run the compilation pipeline.
    static void Main(string[] argsArray)
    {
        Console.OutputEncoding = IoUtil.Utf8NoBom;
        _originalArgs = argsArray ?? new string[0];
        CgbPalettePragmas.Clear();
        ClearBankOverrides();
        // Record the invocation before dispatch; history records an attempted command, not proof of a successful build.
        TryAppendCommandHistory(argsArray);

        // Each recognized subcommand owns its arguments and completion, bypassing ordinary source compilation.
        if (TryRunKqHelpCommand(argsArray)) return;
        if (TryRunSymbolFindCommand(argsArray)) return;
        if (TryRunSourceToAsmCommand(argsArray)) return;
        if (TryRunRomDiffCommand(argsArray)) return;
        if (TryRunVibeTemplateCommand(argsArray)) return;
        if (TryRunFixHintCommand(argsArray)) return;
        if (TryRunAiIrSummaryCommand(argsArray)) return;
        if (TryRunConventionsCommand(argsArray)) return;
        if (TryRunSnippetLibraryCommand(argsArray)) return;
        if (TryRunDevServerCommand(argsArray)) return;
        if (TryRunRecipeCommand(argsArray)) return;

        if (TryRunAttrVizCommand(argsArray)) return;

        if (TryRunTestCommand(argsArray)) return;
        // The watch driver starts child compilations; do not treat its control flag as a compiler option here.
        if (HasWatchFlag(argsArray))
        {
            RunWatchDriver(argsArray);
            return;
        }

        // --minimize: delta-debugging helper to auto-generate a minimal reproducer.
        // Runs as a driver (spawns child compiler processes) and exits.
        if (Minimizer.IsMinimizeRequested(argsArray))
        {
            Minimizer.Run(argsArray);
            return;
        }

        // Process options in encounter order. Presets change several settings, and later options can change them again.
        // Only branches that explicitly dequeue a value consume the next argument.
        Queue<string> args = new Queue<string>(argsArray);

        List<string> sourceFilenames = new List<string>();
        string outputFilename = null;
        bool outputFilenameExplicit = false;
        bool help = (args.Count == 0);

        while (args.Count > 0)
        {
            string arg = args.Dequeue();
            if (arg == "-?" || arg == "-h" || arg == "--help")
            {
                help = true;
            }
            else if (arg == "--no-disasm")
            {
                DisableDisasm = true;
            }
            else if (arg == "--disasm")
            {
                DisableDisasm = false;
            }
            // This convenience switch changes diagnostic work only; it does not select an optimization level.
            else if (arg == "--fast-build" || arg == "--fast")
            {
                DisableDisasm = true;
                TraceEnabled = false;
            }
            else if (arg == "--cache")
            {
                EnableIncrementalCache = true;
            }
            else if (arg == "--no-cache")
            {
                EnableIncrementalCache = false;
            }
            else if (arg == "--diag-json")
            {
                EmitDiagJson = true;
                DiagJsonPath = "kitaqfc.diag.json";
            }
            else if (arg == "--strict")
            {
                DiagnosticsMode = DiagnosticMode.Strict;
            }
            else if (arg == "--permissive")
            {
                DiagnosticsMode = DiagnosticMode.Permissive;
            }
            else if (arg.StartsWith("--diag-json="))
            {
                EmitDiagJson = true;
                DiagJsonPath = ValueAfterEquals(arg);
                if (string.IsNullOrWhiteSpace(DiagJsonPath)) DiagJsonPath = "kitaqfc.diag.json";
            }
            // Request default metadata naming without consuming a following source argument as its path.
            else if (arg == "--kurosaki-metadata" || arg == "--kurosaki-meta")
            {
                EmitKurosakiMetadata = true;
                KurosakiMetadataPath = "";
            }
            else if (arg.StartsWith("--kurosaki-metadata=") || arg.StartsWith("--kurosaki-meta=") || arg.StartsWith("--kurosaki-out="))
            {
                EmitKurosakiMetadata = true;
                KurosakiMetadataPath = ValueAfterEquals(arg);
            }
            else if (arg == "--emit-ai-metadata")
            {
                EmitKurosakiMetadata = true;
                if (args.Count > 0) KurosakiMetadataPath = args.Dequeue();
                else Error("error: --emit-ai-metadata requires a file path");
            }
            else if (arg.StartsWith("--emit-ai-metadata="))
            {
                EmitKurosakiMetadata = true;
                KurosakiMetadataPath = ValueAfterEquals(arg);
                if (string.IsNullOrWhiteSpace(KurosakiMetadataPath)) Error("error: --emit-ai-metadata requires a file path");
            }
            // A bare dependency switch requests the output-derived default; only the equals form sets its path.
            else if (arg == "--deps-out")
            {
                EmitDependenciesList = true;
                DependenciesListPath = "";
            }
            else if (arg.StartsWith("--deps-out="))
            {
                EmitDependenciesList = true;
                DependenciesListPath = ValueAfterEquals(arg);
            }
            else if (arg == "--minimize-on-fail" || arg == "--auto-minimize")
            {
                _autoMinimizeOnFail = true;
            }
            // Apply the preset at this point in the argument stream, preserving the meaning of subsequent overrides.
            else if (arg.StartsWith("--profile="))
            {
                ApplyProfilePreset(ValueAfterEquals(arg));
            }
            // Accept a separate include path here; the following branches support attached and long-option spellings.
            else if (arg == "-I")
            {
                if (args.Count > 0) AddIncludeDirectory(args.Dequeue());
                else Error("error: -I option requires a directory path");
            }
            else if (arg.StartsWith("-I") && arg.Length > 2)
            {
                AddIncludeDirectory(arg.Substring(2));
            }
            else if (arg.StartsWith("--include-dir="))
            {
                AddIncludeDirectory(ValueAfterEquals(arg));
            }
            else if (arg.StartsWith("--target="))
            {
                CompilationTargetInfo parsedTarget;
                string err;
                if (!CompilationTargetInfo.TryParse(ValueAfterEquals(arg), out parsedTarget, out err)) Error(err);
                else TargetInfo = parsedTarget;
            }
            else if (arg.StartsWith("--nes-chr="))
            {
                NesChrRomPath = ValueAfterEquals(arg) ?? "";
            }
            else if (arg.StartsWith("--chr-rom="))
            {
                NesChrRomPath = ValueAfterEquals(arg) ?? "";
            }
            else if (arg == "-O0")
            {
                OptLevel = 0;
            }
            else if (arg == "-O1")
            {
                OptLevel = 1;
            }
            // Enable the NES option bundle and raise ordinary optimization to at least level 1.
            else if (arg == "-O2" || arg == "--nes-opt" || arg == "--nes-opt=all")
            {
                OptLevel = Math.Max(OptLevel, 1);
                EnableNesOptimizationSet();
            }
            else if (arg.StartsWith("--nes-opt="))
            {
                string v = ValueAfterEquals(arg).Trim().ToLowerInvariant();
                if (v == "all" || v == "on" || v == "true" || v == "1") EnableNesOptimizationSet();
                else if (v == "off" || v == "none" || v == "false" || v == "0") DisableNesOptimizationSet();
                else Error("error: --nes-opt must be all or off");
            }
            else if (arg == "--zp-alloc" || arg == "--whole-program-zp")
            {
                EnableWholeProgramZpAllocator = true;
            }
            else if (arg == "--no-zp-alloc" || arg == "--no-whole-program-zp")
            {
                EnableWholeProgramZpAllocator = false;
            }
            // Restrict explicit local storage to the physical 2 KiB internal CPU RAM range.
            else if (arg.StartsWith("--nes-local-ram="))
            {
                int start;
                int length;
                string error;
                if (!TryParseNesRamWindow(ValueAfterEquals(arg), out start, out length, out error))
                    Error("error: --nes-local-ram " + error);
                else
                {
                    HasNesLocalRamWindow = true;
                    NesLocalRamBase = start;
                    NesLocalRamLength = length;
                }
            }
            // Temporary indirect pointers must remain entirely in zero page, even though local storage may use more RAM.
            else if (arg.StartsWith("--nes-temp-ram="))
            {
                int start;
                int length;
                string error;
                if (!TryParseNesRamWindow(ValueAfterEquals(arg), out start, out length, out error))
                    Error("error: --nes-temp-ram " + error);
                else if (length > 256)
                    Error("error: --nes-temp-ram length must be at most 256 bytes");
                else if (start + length > 0x100)
                    Error("error: --nes-temp-ram must be entirely in zero page ($0000-$00FF); 6502 indirect pointer temporaries require zero-page addressing");
                else
                {
                    HasCustomNesTempRamWindow = true;
                    NesTempRamBase = start;
                    NesTempRamLength = length;
                }
            }
            else if (arg == "--reg-cc-v2" || arg == "--fastcall-v2")
            {
                EnableRegisterCallingConventionV2 = true;
                Abi = AbiMode.FastCall;
            }
            // Clear the register flag and restore legacy mode only if the enum currently selects fastcall.
            else if (arg == "--no-reg-cc-v2" || arg == "--no-fastcall-v2")
            {
                EnableRegisterCallingConventionV2 = false;
                if (Abi == AbiMode.FastCall) Abi = AbiMode.Legacy;
            }
            else if (arg == "--static-frame")
            {
                EnableStaticFrameAllocator = true;
            }
            else if (arg == "--no-static-frame")
            {
                EnableStaticFrameAllocator = false;
            }
            else if (arg == "--small-inline" || arg == "--auto-inline-small")
            {
                EnableSmallFunctionAutoInline = true;
            }
            else if (arg == "--no-small-inline" || arg == "--no-auto-inline-small")
            {
                EnableSmallFunctionAutoInline = false;
            }
            else if (arg == "--loop-lowering")
            {
                EnableLoopLoweringOptimizer = true;
            }
            else if (arg == "--no-loop-lowering")
            {
                EnableLoopLoweringOptimizer = false;
            }
            else if (arg == "--library-lto-lite" || arg == "--lto-lite")
            {
                EnableLibraryLtoLite = true;
            }
            else if (arg == "--no-library-lto-lite" || arg == "--no-lto-lite")
            {
                EnableLibraryLtoLite = false;
            }
            else if (arg == "--mapper-aware-placement")
            {
                EnableMapperAwareBankPlacement = true;
            }
            else if (arg == "--no-mapper-aware-placement")
            {
                EnableMapperAwareBankPlacement = false;
            }
            else if (arg == "--nes-opt-report")
            {
                EmitNesOptimizationReport = true;
                NesOptimizationReportPath = "";
            }
            else if (arg.StartsWith("--nes-opt-report="))
            {
                EmitNesOptimizationReport = true;
                NesOptimizationReportPath = ValueAfterEquals(arg);
            }
            // Load profile data immediately; later profile options replace the collected hotness table.
            else if (arg.StartsWith("--kurosaki-profile="))
            {
                EnableKurosakiProfileFeedback = true;
                KurosakiProfilePath = ValueAfterEquals(arg);
                LoadKurosakiProfile(KurosakiProfilePath);
            }
            else if (arg.StartsWith("--abi="))
            {
                string v = arg.Substring("--abi=".Length).Trim().ToLowerInvariant();
                if (v == "stack") Abi = AbiMode.Stack;
                else if (v == "fastcall" || v == "reg" || v == "register") { Abi = AbiMode.FastCall; EnableRegisterCallingConventionV2 = true; }
                else if  (v == "legacy" || v == "default") Abi = AbiMode.Legacy;
                else Error("error: --abi must be legacy, stack, or fastcall");
            }
            // Report switches pair an enable flag with a path; an empty path requests the report-specific default.
            else if (arg == "--bank-sim")
            {
                EmitBankSimReport = true;
                BankSimReportPath = "";
            }
            else if (arg.StartsWith("--bank-sim="))
            {
                EmitBankSimReport = true;
                BankSimReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--farcall-suggest")
            {
                EmitFarcallSuggestionReport = true;
                FarcallSuggestionReportPath = "";
            }
            else if (arg.StartsWith("--farcall-suggest="))
            {
                EmitFarcallSuggestionReport = true;
                FarcallSuggestionReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--cross-bank-report")
            {
                EmitCrossBankCallReport = true;
                CrossBankCallReportPath = "";
            }
            else if (arg.StartsWith("--cross-bank-report="))
            {
                EmitCrossBankCallReport = true;
                CrossBankCallReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--abi-verify")
            {
                EmitAbiVerifyReport = true;
                AbiVerifyReportPath = "";
            }
            else if (arg.StartsWith("--abi-verify="))
            {
                EmitAbiVerifyReport = true;
                AbiVerifyReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--abi-diff-report")
            {
                EmitAbiDiffReport = true;
                AbiDiffReportPath = "";
            }
            else if (arg.StartsWith("--abi-diff-report="))
            {
                EmitAbiDiffReport = true;
                AbiDiffReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--rst-report")
            {
                EmitRstApplyReport = true;
                RstApplyReportPath = "";
            }
            else if (arg.StartsWith("--rst-report="))
            {
                EmitRstApplyReport = true;
                RstApplyReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--opt-diff")
            {
                EmitOptDiffReport = true;
                OptDiffReportPath = "";
            }
            else if (arg.StartsWith("--opt-diff="))
            {
                EmitOptDiffReport = true;
                OptDiffReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--func-size-report")
            {
                EmitFunctionSizeReport = true;
                FunctionSizeReportPath = "";
            }
            else if (arg.StartsWith("--func-size-report="))
            {
                EmitFunctionSizeReport = true;
                FunctionSizeReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--hotspot-report")
            {
                EmitHotspotReport = true;
                HotspotReportPath = "";
            }
            else if (arg.StartsWith("--hotspot-report="))
            {
                EmitHotspotReport = true;
                HotspotReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--repro-check")
            {
                EnableReproCheck = true;
                ReproCheckReportPath = "";
            }
            else if (arg.StartsWith("--repro-check="))
            {
                EnableReproCheck = true;
                ReproCheckReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--cgb-consistency")
            {
                EmitCgbConsistencyReport = true;
                CgbConsistencyReportPath = "";
            }
            else if (arg.StartsWith("--cgb-consistency="))
            {
                EmitCgbConsistencyReport = true;
                CgbConsistencyReportPath = ValueAfterEquals(arg);
            }
            else if (arg == "--verify-cgb-symbols")
            {
                EmitCgbSymbolVerifyReport = true;
                CgbSymbolVerifyReportPath = "";
            }
            else if (arg.StartsWith("--verify-cgb-symbols="))
            {
                EmitCgbSymbolVerifyReport = true;
                CgbSymbolVerifyReportPath = ValueAfterEquals(arg);
            }
// Enable the complete check set together; the bounds-only switch below leaves other checks unchanged.
else if (arg == "-Zcheck")
            {
                CheckBounds = true;
                CheckMemCopy = true;
                CheckStack = true;
                CheckBankCalls = true;
                CheckSliceBounds = true;
            }
            else if (arg == "-Zcheck-bounds" || arg == "-Zcheck_bounds")
            {
                CheckBounds = true;
            }
            else if (arg == "-Zconst-scalar-in-rom" || arg == "-Zconst_scalar_in_rom")
            {
                ConstScalarInRom = true;
            }
            else if (arg == "--debug-output" || arg == "--debug-out")
            {
                EnableDebugOutput = true;
            }
            else if (arg.StartsWith("--debug-output=") || arg.StartsWith("--debug-out="))
            {
                EnableDebugOutput = true;
                int eq = arg.IndexOf('=');
                if (eq >= 0 && eq + 1 < arg.Length)
                {
                    DebugOutputPath = arg.Substring(eq + 1);
                }
            }
            else if (arg == "vlist" || arg == "--vlist")
            {
                EmitVarList = true;
            }
            else if (arg.StartsWith("--vlist=", StringComparison.Ordinal) || arg.StartsWith("--vlist-out=", StringComparison.Ordinal))
            {
                EmitVarList = true;
                int eq = arg.IndexOf('=');
                if (eq >= 0 && eq + 1 < arg.Length)
                {
                    VarListOutputPath = arg.Substring(eq + 1);
                }
            }
            else if (arg == "--rst-disable" || arg == "--no-rst" || arg == "--rst-off")
            {
                RstDisable = true;
            }
            else if (arg == "--rst-enable" || arg == "--rst" || arg == "--rst-on")
            {
                RstDisable = false;
            }
            else if (arg == "--rst-use-38")
            {
                // Any RST tuning option implies the user intends to use RST mapping.
                RstDisable = false;
                RstUse38 = true;
            }
            else if (arg == "--rst-speed-safe")
            {
                RstDisable = false;
                RstSpeedSafe = true;
                // Speed-safe preset: by default, avoid RST mapping on very hot targets.
                // You can override with --rst-max-calls.
                if (RstMaxCalls == int.MaxValue) RstMaxCalls = 12;
                else RstMaxCalls = System.Math.Min(RstMaxCalls, 12);
            }
            else if (arg == "--rst-unsafe")
            {
                RstDisable = false;
                RstUnsafe = true;
            }
            else if (arg == "--rst-safe")
            {
                RstDisable = false;
                RstUnsafe = false;
            }
            else if (arg.StartsWith("--rst-max-calls="))
            {
                RstDisable = false;
                int eq = arg.IndexOf('=');
                if (eq >= 0 && eq + 1 < arg.Length && int.TryParse(arg.Substring(eq + 1), out int v) && v > 0)
                    RstMaxCalls = v;
                else
                    Error("error: --rst-max-calls requires a positive integer");
            }
            else if (arg.StartsWith("--rst-max-vectors="))
            {
                RstDisable = false;
                int eq = arg.IndexOf('=');
                if (eq >= 0 && eq + 1 < arg.Length && int.TryParse(arg.Substring(eq + 1), out int v) && v >= 0)
                    RstMaxVectors = v;
                else
                    Error("error: --rst-max-vectors requires a non-negative integer");
            }
            // Accumulate trimmed, nonempty exclusions across repeated switches rather than replacing the set.
            else if (arg.StartsWith("--rst-exclude="))
            {
                RstDisable = false;
                int eq = arg.IndexOf('=');
                string s = (eq >= 0 && eq + 1 < arg.Length) ? arg.Substring(eq + 1) : "";
                foreach (var part in s.Split(new char[] { ',', ';' }, System.StringSplitOptions.RemoveEmptyEntries))
                {
                    string name = part.Trim();
                    if (name.Length > 0) RstExcludeLabels.Add(name);
                }
            }
            else if (arg == "--attach")
            {
                AttachDebuggerOnError = true;
            }
            else if (arg == "--trace")
            {
                TraceEnabled = true;
                // Default: dump all stages.
                TraceStages.Clear();
                TraceStages.Add("tokens");
                TraceStages.Add("ast");
                TraceStages.Add("ir");
                TraceStages.Add("asm");
            }
            else if (arg.StartsWith("--trace="))
            {
                TraceEnabled = true;
                TraceStages.Clear();
                int eq = arg.IndexOf('=');
                string list = (eq >= 0 && eq + 1 < arg.Length) ? arg.Substring(eq + 1) : "";
                foreach (var part in list.Split(new char[] { ',', ';' }, StringSplitOptions.RemoveEmptyEntries))
                {
                    string s = part.Trim();
                    if (s.Length > 0) TraceStages.Add(s);
                }
                // If user passed an empty list, treat as "all".
                if (TraceStages.Count == 0)
                {
                    TraceStages.Add("tokens");
                    TraceStages.Add("ast");
                    TraceStages.Add("ir");
                    TraceStages.Add("asm");
                }
            }
            else if (arg.StartsWith("--trace-out="))
            {
                TraceEnabled = true;
                int eq = arg.IndexOf('=');
                if (eq >= 0 && eq + 1 < arg.Length)
                {
                    TraceOutputPath = arg.Substring(eq + 1);
                }
            }
            else if (arg == "--disasm-changed")
            {
                EmitChangedFunctionDisasm = true;
                ChangedFunctionDisasmBaseRef = "";
            }
            else if (arg.StartsWith("--disasm-changed="))
            {
                EmitChangedFunctionDisasm = true;
                ChangedFunctionDisasmBaseRef = ValueAfterEquals(arg);
            }
            else if (arg.StartsWith("--disasm-changed-out="))
            {
                EmitChangedFunctionDisasm = true;
                ChangedFunctionDisasmOutPath = ValueAfterEquals(arg);
            }
            else if (arg == "--repro-pack")
            {
                EmitReproPackageOnFail = true;
                ReproPackagePath = "";
            }
            else if (arg.StartsWith("--repro-pack="))
            {
                EmitReproPackageOnFail = true;
                ReproPackagePath = ValueAfterEquals(arg);
            }
            // --- ROM header patching (fix31_rom_header_patch) ---
            // Load and merge this JSON immediately, so the relative order of JSON templates and explicit flags matters.
            else if (arg.StartsWith("--rom-header="))
            {
                string path = ValueAfterEquals(arg);
                try
                {
                    var fromJson = RomHeaderOptions.LoadFromJson(path);
                    // JSON acts like a CLI template: later explicit CLI flags override it.
                    RomHeader.MergeFrom(fromJson, overwrite: true);
                }
                catch (Exception ex)
                {
                    Error("error: --rom-header failed to load '{0}': {1}", path, ex.Message);
                }
            }
            else if (arg.StartsWith("--rom-title="))
            {
                RomHeader.Title = ValueAfterEquals(arg);
            }
            else if (arg.StartsWith("--cgb="))
            {
                if (!RomHeader.TrySetCgb(ValueAfterEquals(arg), out string err)) Error(err);
            }
            else if (arg.StartsWith("--cart="))
            {
                if (!RomHeader.TrySetCart(ValueAfterEquals(arg), out string err)) Error(err);
            }
            else if (arg.StartsWith("--mapper="))
            {
                if (!NesCartridge.TrySetMapper(ValueAfterEquals(arg), out string err)) Error(err);
            }
            else if (arg.StartsWith("--board="))
            {
                if (!NesCartridge.TrySetBoard(ValueAfterEquals(arg), out string err)) Error(err);
            }
            // Resolve and load external disk metadata during option parsing so later packaging has a concrete input.
            else if (arg.StartsWith("--fds-meta=") || arg.StartsWith("--fds-metadata=") || arg.StartsWith("--fds-manifest=") || arg.StartsWith("--fds-builder-manifest="))
            {
                string path = ValueAfterEquals(arg);
                try
                {
                    FdsMetadataPath = Path.GetFullPath(path);
                    FdsMetadata = FdsDiskMetadata.LoadFromFile(FdsMetadataPath);
                }
                catch (Exception ex)
                {
                    Error("error: failed to load FDS metadata manifest {0}: {1}", path, ex.Message);
                }
            }
            // Use a sentinel to distinguish default output naming from disabled metadata output.
            else if (arg == "--fds-meta-out" || arg == "--fds-metadata-out")
            {
                FdsMetadataOutPath = "__default__";
            }
            else if (arg.StartsWith("--fds-meta-out=") || arg.StartsWith("--fds-metadata-out="))
            {
                FdsMetadataOutPath = ValueAfterEquals(arg);
            }
            // Select container policy independently of the filename and optional FDS sidecar request.
            else if (arg.StartsWith("--output-format=") || arg.StartsWith("--container="))
            {
                string v = ValueAfterEquals(arg).Trim().ToLowerInvariant();
                if (v == "auto" || v == "") OutputContainerFormat = NesOutputContainerFormat.Auto;
                else if (v == "nes" || v == "ines" || v == "i-nes") OutputContainerFormat = NesOutputContainerFormat.INes;
                else if (v == "fds" || v == "famicom-disk-system") OutputContainerFormat = NesOutputContainerFormat.Fds;
                else Error("error: --output-format must be auto|nes|fds");
            }
            // Request an additional FDS image using default naming; the equals form supplies its destination.
            else if (arg == "--fds-out" || arg == "--fds-image")
            {
                EmitFdsSidecar = true;
                FdsImageOutputPath = "__default__";
            }
            else if (arg.StartsWith("--fds-out=") || arg.StartsWith("--fds-image="))
            {
                EmitFdsSidecar = true;
                FdsImageOutputPath = ValueAfterEquals(arg);
            }
            else if (arg == "--fds-header")
            {
                FdsImageHeaderEnabled = true;
            }
            else if (arg == "--fds-no-header" || arg == "--fds-raw")
            {
                FdsImageHeaderEnabled = false;
            }
            else if (arg.StartsWith("--fds-game-code="))
            {
                FdsGameCode = ValueAfterEquals(arg);
            }
            else if (arg == "--fds-license-bypass" || arg == "--fds-bypass-license" || arg == "--fds-approval-bypass")
            {
                FdsLicenseBypassEnabled = true;
            }
            else if (arg == "--fds-no-license-bypass" || arg == "--fds-no-approval-bypass")
            {
                FdsLicenseBypassEnabled = false;
            }
            else if (arg.StartsWith("--fds-layout="))
            {
                string v = ValueAfterEquals(arg).Trim().ToLowerInvariant();
                if (v == "fds32" || v == "fds" || v == "prgram" || v == "6000-dfff" || v == "6000") FdsPrgRamLayoutEnabled = true;
                else if (v == "legacy" || v == "ines" || v == "nrom") FdsPrgRamLayoutEnabled = false;
                else Error("error: --fds-layout must be fds32|legacy");
            }
            else if (arg == "--fds-auto-overlay" || arg == "--fds-auto-overlays")
            {
                FdsAutoOverlayEnabled = true;
            }
            else if (arg == "--fds-no-auto-overlay" || arg == "--fds-no-auto-overlays")
            {
                FdsAutoOverlayEnabled = false;
            }
            // Reserve 255 by accepting overlay start IDs only through 254.
            else if (arg.StartsWith("--fds-overlay-start-id=") || arg.StartsWith("--fds-overlay-id-base="))
            {
                string v = ValueAfterEquals(arg).Trim();
                int id;
                if (!int.TryParse(v, out id) || id < 0 || id > 254)
                    Error("error: --fds-overlay-start-id must be 0..254");
                else
                    FdsOverlayStartId = id;
            }
            else if (arg.StartsWith("--fds-overlay-prefix="))
            {
                FdsOverlayNamePrefix = ValueAfterEquals(arg);
            }
            else if (arg == "--fds-overlay-trim")
            {
                FdsOverlayTrimTrailingFill = true;
            }
            else if (arg == "--fds-overlay-no-trim")
            {
                FdsOverlayTrimTrailingFill = false;
            }
            else if (arg == "--fds-overlay-farcall")
            {
                FdsOverlayFarcallEnabled = true;
            }
            else if (arg == "--fds-no-overlay-farcall")
            {
                FdsOverlayFarcallEnabled = false;
            }
            else if (arg == "--fds-overlay-guard" || arg == "--fds-residency-guard")
            {
                FdsOverlayResidencyGuardEnabled = true;
            }
            else if (arg == "--fds-no-overlay-guard" || arg == "--fds-no-residency-guard")
            {
                FdsOverlayResidencyGuardEnabled = false;
            }
            else if (arg == "--fds-overlay-table")
            {
                FdsOverlayFunctionTableEnabled = true;
            }
            else if (arg == "--fds-no-overlay-table")
            {
                FdsOverlayFunctionTableEnabled = false;
            }
            else if (arg.StartsWith("--mirroring="))
            {
                if (!NesCartridge.TrySetMirroring(ValueAfterEquals(arg), out string err)) Error(err);
            }
            else if (arg == "--battery")
            {
                NesCartridge.BatteryBacked = true;
            }
            else if (arg == "--no-battery")
            {
                NesCartridge.BatteryBacked = false;
            }
            else if (arg.StartsWith("--romsize="))
            {
                if (!RomHeader.TrySetRomSize(ValueAfterEquals(arg), out string err)) Error(err);
            }
            else if (arg.StartsWith("--ramsize="))
            {
                if (!RomHeader.TrySetRamSize(ValueAfterEquals(arg), out string err)) Error(err);
            }
            else if (arg.StartsWith("--sgb="))
            {
                if (!RomHeader.TrySetSgb(ValueAfterEquals(arg), out string err)) Error(err);
            }
            else if (arg.StartsWith("--dest="))
            {
                if (!RomHeader.TrySetDest(ValueAfterEquals(arg), out string err)) Error(err);
            }
            else if (arg.StartsWith("--version="))
            {
                string v = ValueAfterEquals(arg);
                if (!byte.TryParse(v, out byte bv)) Error("error: --version must be 0..255");
                RomHeader.Version = bv;
            }
            else if (arg.StartsWith("--max-errors="))
            {
                int eq = arg.IndexOf('=');
                if (eq >= 0 && eq + 1 < arg.Length && int.TryParse(arg.Substring(eq + 1), out int v) && v > 0)
                    MaxErrors = v;
                else
                    Error("error: --max-errors requires a positive integer");
            }
            else if (arg == "-o")
            {
                if (args.Count > 0)
                {
                    outputFilename = args.Dequeue();
                    outputFilenameExplicit = true;
                }
                else Error("error: -o option requires a filename");
            }
            // Reject unknown options rather than interpreting them as source paths.
            else if (arg.StartsWith("-"))
            {
                Error("error: unknown option: " + arg);
            }
            else
            {
                sourceFilenames.Add(arg);
            }
        }

        // Help and an empty invocation take the usage path before attempting a build.
        if (help)
        {
            Console.Error.WriteLine("usage: kitaqfc first.c second.c ... [--target=nes] [--mapper=nrom|uxrom|cnrom|axrom|mmc1|mmc3|mmc5|vrc6|vrc7|fme7|fds] [--board=auto|generic|surom512] [--mirroring=horizontal|vertical|four-screen] [--battery|--no-battery] [--nes-chr=path/to/chr.bin] [--kurosaki-metadata[=<file>]] [-o out.nes]");
            Console.Error.WriteLine("NES RAM placement: [--no-whole-program-zp] [--nes-local-ram=START:LENGTH] [--nes-temp-ram=START:LENGTH]");
            Console.Error.WriteLine("subcommands: kitaqfc test | attrviz | src2asm | symfind | romdiff | kqhelp | template | fixhint | irsum | conventions | snippet | devserver | recipe");
            Exit(1);
        }

        // Check mapper/board compatibility after all individual cartridge options have been applied.
        if (!NesCartridge.ValidateCombination(out string cartridgeError))
            Error(cartridgeError);

        if (sourceFilenames.Count == 0)
        {
            Error("error: no source files provided");
        }

        // Choose the target default extension only when the user did not supply an output filename.
        if (!outputFilenameExplicit)
        {
            outputFilename = "out" + TargetInfo.DefaultOutputExtension;
        }

        _outputFilenameForDiag = outputFilename;

        if (EnableIncrementalCache && CanUseBuildCacheForThisRun())
        {
            // A successful restore bypasses the compilation stages and exits through the normal completion path.
            if (TryRestoreBuildCache(sourceFilenames, outputFilename, out _cacheKey))
            {
                Console.WriteLine("[cache] hit: " + _cacheKey);
                if (TraceEnabled)
                {
                    var cacheStats = new List<TraceStageStat>();
                    cacheStats.Add(new TraceStageStat { Name = "cache_restore", ElapsedMs = 0, Detail = _cacheKey });
                    EmitTraceShortSummary(cacheStats, 0, sourceFilenames, outputFilename);
                }
                Exit(0);
            }
        }

        // Measure executed pipeline stages separately from a successful cache-restore shortcut.
        var traceStageStats = new List<TraceStageStat>();
        var compileTotalSw = Stopwatch.StartNew();

        try
        {
            // 0. Tokenize (optional trace)
            // The token dump is an extra pass over top-level inputs; parsing still performs its own tokenization.
            if (TraceEnabled && (TraceStages.Contains("tokens") || TraceStages.Contains("all")))
            {
                var swTokens = Stopwatch.StartNew();
                Directory.CreateDirectory(TraceOutputPath);
                StringBuilder all = new StringBuilder();
                foreach (string fn in sourceFilenames)
                {
                    var toks = Tokenizer.TokenizeFile(fn);
                    all.AppendLine("==== TOKENS: " + fn + " ====");
                    all.AppendLine(ShowTokens(toks));
                    all.AppendLine();
                }
                WriteTraceFile("tokens.txt", all.ToString());
                swTokens.Stop();
                traceStageStats.Add(new TraceStageStat { Name = "tokens", ElapsedMs = swTokens.ElapsedMilliseconds, Detail = sourceFilenames.Count + " file(s)" });
            }

            // 1. Parse
            var swParse = Stopwatch.StartNew();
            Expr syntaxTree = Parser.ParseFiles(sourceFilenames);
            if (ErrorCount > 0) Exit(1);
            // Add accepted palette pragmas as readonly declarations before lowering consumes the tree.
            syntaxTree = InjectCgbPaletteDeclarations(syntaxTree);
            if (EnableDebugOutput && CgbPalettePragmas.Count > 0)
            {
                WriteDebugFile("cgb_palettes.txt", BuildCgbPaletteText());
            }
            if (EnableDebugOutput) WritePassOutputToFile("syntax_tree", syntaxTree.ShowMultiline());
            if (TraceEnabled && (TraceStages.Contains("ast") || TraceStages.Contains("all")))
            {
                WriteTraceFile("ast_simple.txt", ShowAstSimple(syntaxTree));
                WriteTraceFile("ast_full.txt", syntaxTree.ShowMultiline());
            }
            swParse.Stop();
            traceStageStats.Add(new TraceStageStat { Name = "parse", ElapsedMs = swParse.ElapsedMilliseconds, Detail = "ast built" });

            // 1.5 Lowering (introduce explicit temporaries for a few key patterns)
            var swLower = Stopwatch.StartNew();
            syntaxTree = Lowerer.Lower(syntaxTree);
            if (ErrorCount > 0) Exit(1);
            if (EnableDebugOutput) WritePassOutputToFile("syntax_tree_lowered", syntaxTree.ShowMultiline());
            if (TraceEnabled && (TraceStages.Contains("ir") || TraceStages.Contains("all")))
            {
                WriteTraceFile("ir_lowered.txt", syntaxTree.ShowMultiline());
            }
            swLower.Stop();
            traceStageStats.Add(new TraceStageStat { Name = "lower", ElapsedMs = swLower.ElapsedMilliseconds, Detail = "ir lowered" });

            // 2. CodeGen + 3. Assemble
            long totalCodegenMs = 0;
            long totalAssembleMs = 0;
            int bankRelayoutPasses = 0;
            IReadOnlyList<Expr> assembly = null;
            string actualOutputFilename = outputFilename;
            ClearBankOverrides();
            // Select code generation and assembly through the target backend after clearing previous placement overrides.
            var backend = TargetBackendFactory.Create(TargetInfo);

            while (true)
            {
                var swCodegen = Stopwatch.StartNew();
                assembly = backend.CompileAll(syntaxTree);
                if (ErrorCount > 0) Exit(1);
                // Apply assembly optimization to the backend output before recording or assembling that pass.
                assembly = Optimizer.Optimize((assembly ?? Array.Empty<Expr>()).ToList(), OptLevel);
                if (ErrorCount > 0) Exit(1);
                if (EnableDebugOutput) WritePassOutputToFile("assembly_code", ShowAssembly(assembly));
                if (TraceEnabled && (TraceStages.Contains("asm") || TraceStages.Contains("all")))
                {
                    WriteTraceFile("asm.txt", ShowAssembly(assembly));
                }
                swCodegen.Stop();
                totalCodegenMs += swCodegen.ElapsedMilliseconds;

                var swAssemble = Stopwatch.StartNew();
                actualOutputFilename = backend.Assemble(assembly, outputFilename);
                outputFilename = actualOutputFilename;
                _outputFilenameForDiag = outputFilename;
                if (ErrorCount > 0) Exit(1);
                swAssemble.Stop();
                totalAssembleMs += swAssemble.ElapsedMilliseconds;

                // Run placement feedback only when the selected backend advertises support for it.
                if (!backend.TargetInfo.SupportsFunctionBankRelayout)
                    break;

                var relayout = DetectFunctionBankRelocations(backend.CodegenReport, backend.AssemblerReport);
                if (relayout.FixedBankConflicts.Count > 0)
                {
                    Error("error: fixed-bank functions spilled to a different bank: {0}",
                        string.Join(", ", relayout.FixedBankConflicts
                            .OrderBy(x => x.Name, StringComparer.Ordinal)
                            .Select(x => x.Name + " requested b" + x.RequestedBank + " actual b" + x.ActualBank)));
                    Exit(1);
                }
                // Readonly data with an explicit fixed bank is a placement constraint, just like a fixed function.
                if (relayout.FixedReadonlyDataConflicts.Count > 0)
                {
                    Error("error: fixed-bank readonly data spilled to a different bank: {0}",
                        string.Join(", ", relayout.FixedReadonlyDataConflicts
                            .OrderBy(x => x.Name, StringComparer.Ordinal)
                            .Select(x => x.Name + " requested b" + x.RequestedBank + " actual b" + x.ActualBank)));
                    Exit(1);
                }

                var relocations = relayout.Relocations;
                var readonlyRelocations = relayout.ReadonlyDataRelocations;
                // Both function and readonly-data assignments must stabilize before the output is accepted.
                if (relocations.Count == 0 && readonlyRelocations.Count == 0)
                    break;

                bankRelayoutPasses++;
                // Stop after four applied relocation retries instead of allowing an unstable layout to loop indefinitely.
                if (bankRelayoutPasses > 4)
                {
                    var unstableNotes = relocations.OrderBy(x => x.Key, StringComparer.Ordinal)
                        .Select(x => x.Key + "->b" + x.Value)
                        .Concat(readonlyRelocations.OrderBy(x => x.Key, StringComparer.Ordinal)
                            .Select(x => x.Key + "->b" + x.Value));
                    Error("error: symbol bank layout did not stabilize after {0} passes: {1}",
                        bankRelayoutPasses,
                        string.Join(", ", unstableNotes));
                    Exit(1);
                }

                // Feed both symbol classes back into parsing and code generation for the next attempt.
                ApplyBankOverrides(relocations, readonlyRelocations);
                var layoutNotes = relocations.OrderBy(x => x.Key, StringComparer.Ordinal)
                    .Select(x => x.Key + "->b" + x.Value)
                    .Concat(readonlyRelocations.OrderBy(x => x.Key, StringComparer.Ordinal)
                        .Select(x => x.Key + "->b" + x.Value));
                Console.Error.WriteLine("[bank-layout] recompiling with actual symbol banks: " +
                    string.Join(", ", layoutNotes));

                // Rebuild the AST so parser-side bank-aware constructs (e.g. pooled string literals,
                // function-local static placement) can observe the updated bank overrides.
                var swReparse = Stopwatch.StartNew();
                syntaxTree = Parser.ParseFiles(sourceFilenames);
                if (ErrorCount > 0) Exit(1);
                syntaxTree = Lowerer.Lower(syntaxTree);
                if (ErrorCount > 0) Exit(1);
                swReparse.Stop();
                totalCodegenMs += swReparse.ElapsedMilliseconds;
            }

            string codegenDetail = (assembly == null ? 0 : assembly.Count).ToString() + " asm node(s)";
            // Accumulate timings across placement retries rather than reporting only the final code-generation pass.
            if (bankRelayoutPasses > 0) codegenDetail += ", relayout_passes=" + bankRelayoutPasses;
            traceStageStats.Add(new TraceStageStat { Name = "codegen", ElapsedMs = totalCodegenMs, Detail = codegenDetail });
            traceStageStats.Add(new TraceStageStat { Name = "assemble", ElapsedMs = totalAssembleMs, Detail = Path.GetFileName(outputFilename) });

            // 3.5 Patch ROM header & checksums (GB target only in phase 1)
            var swHeader = Stopwatch.StartNew();
            var effectiveRomHeader = new RomHeaderOptions();
            // Keep the GB header-patching path behind the backend capability gate; it is not an iNES/FDS header writer.
            if (backend.TargetInfo.SupportsRomHeaderPatching)
            {
                // defaults -> pragma -> cli/json
                effectiveRomHeader.MergeFrom(RomHeaderPragma, overwrite: true);
                effectiveRomHeader.MergeFrom(RomHeader, overwrite: true);

                // If the ROM is larger than 32KB, it *must* use an MBC; otherwise banked code/data is unreachable.
                // Some projects rely on #pragma rom_* in a dedicated romcfg file; however, to avoid producing
                // non-bankable ROMs by accident, we auto-fill the minimum-required fields when absent.
                bool autoFilled = false;
                try
                {
                    var romBytes = File.ReadAllBytes(outputFilename);
                    int romLen = romBytes.Length;

                    if (!effectiveRomHeader.RomSizeCode.HasValue)
                    {
                        byte? code = GuessRomSizeCodeFromLength(romLen);
                        if (code.HasValue)
                        {
                            effectiveRomHeader.RomSizeCode = code.Value;
                            autoFilled = true;
                        }
                    }

                    if (!effectiveRomHeader.CartType.HasValue)
                    {
                        if (romLen > 32 * 1024)
                        {
                            // Safe default: MBC5 ROM-only. Projects can override via #pragma rom_cart / CLI.
                            effectiveRomHeader.CartType = 0x19;
                            autoFilled = true;
                        }
                    }
                }
                catch
                {
                    // Size-based default inference is best effort; PatchFile below can still report errors.
                }

                if (effectiveRomHeader.HasAny || autoFilled)
                {
                    RomHeaderPatcher.PatchFile(outputFilename, effectiveRomHeader);
                    if (ErrorCount > 0) Exit(1);
                }
                swHeader.Stop();
                traceStageStats.Add(new TraceStageStat { Name = "header", ElapsedMs = swHeader.ElapsedMilliseconds, Detail = (effectiveRomHeader.HasAny || autoFilled) ? "patched" : "unchanged" });
            }
            else
            {
                swHeader.Stop();
                traceStageStats.Add(new TraceStageStat { Name = "header", ElapsedMs = swHeader.ElapsedMilliseconds, Detail = "skipped (target)" });
            }

            // 4. Disassemble
            if (EnableDebugOutput && !DisableDisasm && backend.TargetInfo.SupportsDisassembly)
            {
                var swDisasm = Stopwatch.StartNew();
                Disassembler.Disassemble(outputFilename);
                swDisasm.Stop();
                traceStageStats.Add(new TraceStageStat { Name = "disasm", ElapsedMs = swDisasm.ElapsedMilliseconds, Detail = "debug_output/dis.s" });
            }
            else if (EnableDebugOutput && DisableDisasm)
            {
                traceStageStats.Add(new TraceStageStat { Name = "disasm", ElapsedMs = 0, Detail = "skipped (--no-disasm)" });
            }
            else if (EnableDebugOutput && !backend.TargetInfo.SupportsDisassembly)
            {
                traceStageStats.Add(new TraceStageStat { Name = "disasm", ElapsedMs = 0, Detail = "skipped (target)" });
            }

            if (EnableDebugOutput && EmitChangedFunctionDisasm && !DisableDisasm && backend.TargetInfo.SupportsDisassembly)
            {
                var swChangedDis = Stopwatch.StartNew();
                TryWriteChangedFunctionDisasm(outputFilename, sourceFilenames, ChangedFunctionDisasmBaseRef, ChangedFunctionDisasmOutPath);
                swChangedDis.Stop();
                traceStageStats.Add(new TraceStageStat { Name = "disasm_changed", ElapsedMs = swChangedDis.ElapsedMilliseconds, Detail = "filtered" });
            }
            else if (EnableDebugOutput && EmitChangedFunctionDisasm && DisableDisasm)
            {
                Warning("warning: --disasm-changed was requested but disassembly is disabled (--no-disasm)");
            }
            else if (EnableDebugOutput && EmitChangedFunctionDisasm && !backend.TargetInfo.SupportsDisassembly)
            {
                Warning("warning: --disasm-changed is not supported for target '" + backend.TargetInfo.CliName + "' in phase 1");
            }

            // Run requested analysis after the final assembled/header-patched output; reported errors still fail the build.
            var swReports = Stopwatch.StartNew();
            RunAnalysisReports(sourceFilenames, assembly, outputFilename, effectiveRomHeader, backend);
            if (ErrorCount > 0) Exit(1);
            swReports.Stop();
            traceStageStats.Add(new TraceStageStat { Name = "reports", ElapsedMs = swReports.ElapsedMilliseconds, Detail = backend.TargetInfo.SupportsAnalysisReports ? "analysis" : "skipped (target)" });

            // Write requested disk metadata against the actual assembled output name before dependency/cache bookkeeping.
            TryWriteFdsMetadataOut(outputFilename);
            TryWriteDependenciesList(sourceFilenames, outputFilename);

            if (EnableIncrementalCache && CanUseBuildCacheForThisRun())
            {
                // Save only after compilation and reports reach this point; compute a cache key if restore did not supply one.
                var swCacheSave = Stopwatch.StartNew();
                if (string.IsNullOrEmpty(_cacheKey))
                {
                    _cacheKey = ComputeBuildCacheKey(sourceFilenames);
                }
                TrySaveBuildCache(_cacheKey, outputFilename);
                swCacheSave.Stop();
                traceStageStats.Add(new TraceStageStat { Name = "cache_save", ElapsedMs = swCacheSave.ElapsedMilliseconds, Detail = _cacheKey });
            }
        }
        catch (Exception ex)
        {
            Panic(ex.Message + "\n" + ex.StackTrace);
        }

        compileTotalSw.Stop();
        if (TraceEnabled)
        {
            EmitTraceShortSummary(traceStageStats, compileTotalSw.ElapsedMilliseconds, sourceFilenames, outputFilename);
        }

        Exit(0);
    }

    // Create the debug directory on demand and delegate UTF-8 writing and alternate-path handling to IoUtil.
    public static void WriteDebugFile(string filename, string text)
    {
        if (EnableDebugOutput)
        {
            Directory.CreateDirectory(DebugOutputPath);
            IoUtil.WriteAllTextUtf8Robust(Path.Combine(DebugOutputPath, filename), text, allowAlternatePath: true);
        }
    }

    // Trace output has its own enable flag and directory, independent of ordinary debug dumps.
    static void WriteTraceFile(string filename, string text)
    {
        if (!TraceEnabled) return;
        Directory.CreateDirectory(TraceOutputPath);
        IoUtil.WriteAllTextUtf8Robust(Path.Combine(TraceOutputPath, filename), text, allowAlternatePath: true);
    }

    static byte? GuessRomSizeCodeFromLength(int romLenBytes)
    {
        // GB header ROM size codes (most common sizes). We round up to the next supported size.
        int[] sizes = new int[]
        {
            32 * 1024,
            64 * 1024,
            128 * 1024,
            256 * 1024,
            512 * 1024,
            1024 * 1024,
            2 * 1024 * 1024,
            4 * 1024 * 1024,
            8 * 1024 * 1024,
        };

        byte[] codes = new byte[] { 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08 };

        for (int i = 0; i < sizes.Length; i++)
        {
            if (romLenBytes <= sizes[i]) return codes[i];
        }
        return null;
    }

    // Keep everything after the first equals sign, including further equals signs; a missing value is empty.
    static string ValueAfterEquals(string arg)
    {
        int eq = arg.IndexOf('=');
        if (eq >= 0 && eq + 1 < arg.Length) return arg.Substring(eq + 1);
        return "";
    }

    // Parse START:LENGTH and require a positive half-open interval within physical internal CPU RAM.
    static bool TryParseNesRamWindow(string value, out int start, out int length, out string error)
    {
        start = 0;
        length = 0;
        error = "";
        string[] parts = (value ?? "").Split(':');
        if (parts.Length != 2 ||
            !TryParseCliInteger(parts[0], out start) ||
            !TryParseCliInteger(parts[1], out length))
        {
            error = "must use START:LENGTH with decimal or 0x-prefixed values";
            return false;
        }
        if (start < 0 || start >= 0x0800 || length <= 0)
        {
            error = "must name a positive range within internal CPU RAM ($0000-$07FF)";
            return false;
        }
        // Widen before addition so a large length cannot overflow into an apparently valid RAM endpoint.
        long endExclusive = (long)start + length;
        if (endExclusive > 0x0800)
        {
            error = "range exceeds internal CPU RAM ($0000-$07FF)";
            return false;
        }
        return true;
    }

    // Use invariant decimal or 0x-prefixed hexadecimal syntax for RAM window components.
    static bool TryParseCliInteger(string value, out int result)
    {
        result = 0;
        string text = (value ?? "").Trim();
        if (text.StartsWith("0x", StringComparison.OrdinalIgnoreCase))
            return int.TryParse(text.Substring(2), NumberStyles.AllowHexSpecifier, CultureInfo.InvariantCulture, out result);
        return int.TryParse(text, NumberStyles.Integer, CultureInfo.InvariantCulture, out result);
    }

    // Enable the coordinated NES optimization flags and their report; promote a legacy ABI to fastcall.
    static void EnableNesOptimizationSet()
    {
        EnableWholeProgramZpAllocator = true;
        EnableRegisterCallingConventionV2 = true;
        EnableStaticFrameAllocator = true;
        EnableSmallFunctionAutoInline = true;
        EnableLoopLoweringOptimizer = true;
        EnableLibraryLtoLite = true;
        EnableKurosakiProfileFeedback = true;
        EnableMapperAwareBankPlacement = true;
        EmitNesOptimizationReport = true;
        if (Abi == AbiMode.Legacy) Abi = AbiMode.FastCall;
    }

    // Disable the coordinated optimization flags without clearing loaded profile data or the report request.
    static void DisableNesOptimizationSet()
    {
        EnableWholeProgramZpAllocator = false;
        EnableRegisterCallingConventionV2 = false;
        EnableStaticFrameAllocator = false;
        EnableSmallFunctionAutoInline = false;
        EnableLoopLoweringOptimizer = false;
        EnableLibraryLtoLite = false;
        EnableKurosakiProfileFeedback = false;
        EnableMapperAwareBankPlacement = false;
        if (Abi == AbiMode.FastCall) Abi = AbiMode.Legacy;
    }

    // Treat an absent or empty function name as having no recorded profile weight.
    public static int GetKurosakiHotness(string functionName)
    {
        if (string.IsNullOrEmpty(functionName)) return 0;
        int v;
        if (KurosakiHotness.TryGetValue(functionName, out v)) return v;
        return 0;
    }

    // Replace prior weights using permissive text-pattern matching rather than a full JSON parser.
    // For repeated names, keep the greatest accepted nonnegative count.
    static void LoadKurosakiProfile(string path)
    {
        KurosakiHotness.Clear();
        if (string.IsNullOrWhiteSpace(path)) return;
        if (!File.Exists(path))
        {
            Warning("warning: KUROSAKI profile not found: " + path);
            return;
        }
        try
        {
            string text = File.ReadAllText(path);
            // Accept both compact {"function":123} profiles and richer KUROSAKI artifacts
            // that contain {"name":"foo","count":123} / {"function":"foo","cycles":...}.
            foreach (System.Text.RegularExpressions.Match m in System.Text.RegularExpressions.Regex.Matches(text, @"""(?<name>[A-Za-z_.$][A-Za-z0-9_.$]*)""\s*:\s*(?<n>[0-9]+)"))
            {
                string name = m.Groups["name"].Value;
                int n;
                if (int.TryParse(m.Groups["n"].Value, out n)) KurosakiHotness[name] = Math.Max(GetKurosakiHotness(name), n);
            }
            foreach (System.Text.RegularExpressions.Match m in System.Text.RegularExpressions.Regex.Matches(text, @"""(?:name|function)""\s*:\s*""(?<name>[A-Za-z_.$][A-Za-z0-9_.$]*)""(?:(?!\}).)*?""(?:count|calls|cycles|samples)""\s*:\s*(?<n>[0-9]+)", System.Text.RegularExpressions.RegexOptions.Singleline))
            {
                string name = m.Groups["name"].Value;
                int n;
                if (int.TryParse(m.Groups["n"].Value, out n)) KurosakiHotness[name] = Math.Max(GetKurosakiHotness(name), n);
            }
        }
        catch (Exception ex)
        {
            Warning("warning: failed to read KUROSAKI profile '{0}': {1}", path, ex.Message);
        }
    }

    // Called from Tokenizer when it sees #pragma rom_* directives.
    // Priority rule: CLI/JSON (RomHeader) overrides pragma.
    public static void ApplyRomHeaderPragma(FilePosition pos, string key, string value)
    {
        // Normalize legacy key names.
        string k = (key ?? "").Trim();
        if (k.Equals("rom_title", StringComparison.OrdinalIgnoreCase)) k = "rom_title";
        if (k.Equals("rom-title", StringComparison.OrdinalIgnoreCase)) k = "rom_title";

        // Option B: #pragma rom_header "header.json"
        // Loads a JSON template and applies it as pragma-level defaults.
        // Priority rule (already enforced here): CLI/JSON > pragma > defaults.
        // Within pragma level: rom_header provides defaults, and later explicit rom_* pragmas can override.
        if (k.Equals("rom_header", StringComparison.OrdinalIgnoreCase) ||
            k.Equals("romheader", StringComparison.OrdinalIgnoreCase) ||
            k.Equals("rom-header", StringComparison.OrdinalIgnoreCase))
        {
            // If the CLI already specified a rom-header template, we still allow pragma rom_header
            // to fill any remaining *unset* fields. However, any field already set by CLI is never overridden.

            string path = (value ?? "").Trim();
            if (string.IsNullOrEmpty(path))
            {
                Warning(pos, "warning: #pragma rom_header requires a path");
                return;
            }

            // Resolve relative to the current source file.
            try
            {
                if (!Path.IsPathRooted(path))
                {
                    string baseDir = "";
                    try { baseDir = Path.GetDirectoryName(pos.Filename) ?? ""; } catch { baseDir = ""; }
                    if (!string.IsNullOrEmpty(baseDir)) path = Path.Combine(baseDir, path);
                }

                var tpl = RomHeaderOptions.LoadFromJson(path);

                // Apply each field as pragma-level defaults:
                // - never override CLI (RomHeader)
                // - never override already-set pragma fields (so explicit #pragma rom_title can override)
                if (tpl.Title != null && !RomHeader.IsFieldSetByKey("title") && RomHeaderPragma.Title == null)
                    RomHeaderPragma.Title = tpl.Title;

                if (tpl.CgbFlag.HasValue && !RomHeader.IsFieldSetByKey("cgb") && !RomHeaderPragma.CgbFlag.HasValue)
                    RomHeaderPragma.CgbFlag = tpl.CgbFlag;

                if (tpl.SgbFlag.HasValue && !RomHeader.IsFieldSetByKey("sgb") && !RomHeaderPragma.SgbFlag.HasValue)
                    RomHeaderPragma.SgbFlag = tpl.SgbFlag;

                if (tpl.CartType.HasValue && !RomHeader.IsFieldSetByKey("cart") && !RomHeaderPragma.CartType.HasValue)
                    RomHeaderPragma.CartType = tpl.CartType;

                if ((tpl.RomSizeCode.HasValue || tpl.RomSizeBytes.HasValue) &&
                    !RomHeader.IsFieldSetByKey("romsize") &&
                    !(RomHeaderPragma.RomSizeCode.HasValue || RomHeaderPragma.RomSizeBytes.HasValue))
                {
                    RomHeaderPragma.RomSizeCode = tpl.RomSizeCode;
                    RomHeaderPragma.RomSizeBytes = tpl.RomSizeBytes;
                }

                if (tpl.RamSizeCode.HasValue && !RomHeader.IsFieldSetByKey("ramsize") && !RomHeaderPragma.RamSizeCode.HasValue)
                    RomHeaderPragma.RamSizeCode = tpl.RamSizeCode;

                if (tpl.DestinationCode.HasValue && !RomHeader.IsFieldSetByKey("dest") && !RomHeaderPragma.DestinationCode.HasValue)
                    RomHeaderPragma.DestinationCode = tpl.DestinationCode;

                if (tpl.Version.HasValue && !RomHeader.IsFieldSetByKey("version") && !RomHeaderPragma.Version.HasValue)
                    RomHeaderPragma.Version = tpl.Version;
            }
            catch (Exception ex)
            {
                Warning(pos, "warning: #pragma rom_header failed to load '" + path + "': " + ex.Message);
            }
            return;
        }

        // If CLI already specified this field, ignore the pragma.
        if (RomHeader.IsFieldSetByKey(k)) return;

        // Apply to pragma options.
        if (!RomHeaderPragma.TrySetByKey(k, value, out string err))
        {
            if (!string.IsNullOrEmpty(err)) Warning(pos, err);
            return;
        }
    }

    // Accept a named group of exactly four 15-bit colors, retaining the pragma location for later diagnostics.
    public static void ApplyCgbPalettePragma(FilePosition pos, string name, int[] colors)
    {
        string n = (name ?? "").Trim();
        if (string.IsNullOrEmpty(n))
        {
            Warning(pos, "warning: #pragma cgb_palette requires a name");
            return;
        }

        if (colors == null || colors.Length != 4)
        {
            Warning(pos, "warning: #pragma cgb_palette requires exactly 4 colors");
            return;
        }

        for (int i = 0; i < colors.Length; i++)
        {
            if (colors[i] < 0 || colors[i] > 0x7FFF)
            {
                Warning(pos, "warning: #pragma cgb_palette color out of range (0..0x7FFF): " + colors[i]);
                return;
            }
        }

        // A later valid definition replaces earlier entries with the same case-sensitive name.
        for (int i = CgbPalettePragmas.Count - 1; i >= 0; i--)
        {
            if (string.Equals(CgbPalettePragmas[i].Name, n, StringComparison.Ordinal))
            {
                CgbPalettePragmas.RemoveAt(i);
            }
        }

        // Copy the color array so later caller mutations cannot alter the stored definition.
        CgbPalettePragmas.Add(new CgbPaletteDefinition
        {
            Name = n,
            Colors = colors.ToArray(),
            Position = pos
        });
    }

    // Append const u16[4] data for palette names that do not collide with an existing top-level declaration.
    static Expr InjectCgbPaletteDeclarations(Expr syntaxTree)
    {
        if (CgbPalettePragmas.Count == 0) return syntaxTree;

        Expr[] decls;
        if (!syntaxTree.MatchAny(Tag.Sequence, out decls)) return syntaxTree;

        var merged = new List<Expr>(decls ?? new Expr[0]);
        // Collect names before insertion so a palette cannot silently replace a user declaration.
        var usedNames = new HashSet<string>(StringComparer.Ordinal);

        foreach (var d in merged)
        {
            if (TryGetTopLevelDeclName(d, out string dn) && !string.IsNullOrEmpty(dn))
                usedNames.Add(dn);
        }

        foreach (var p in CgbPalettePragmas)
        {
            if (usedNames.Contains(p.Name))
            {
                Warning(p.Position, "warning: cgb palette name collides with existing symbol: " + p.Name);
                continue;
            }

            // Retain the pragma source location on generated data for meaningful compiler diagnostics.
            var type = CType.MakeArray(CType.UInt16, 4);
            type.IsConst = true;
            Expr rd = Expr.Make(Tag.ReadonlyData, type, p.Name, p.Colors.ToArray()).WithSource(p.Position);
            merged.Add(rd);
            usedNames.Add(p.Name);
        }

        // Rebuild the sequence with its original source location while retaining declaration order.
        var args = new object[merged.Count + 1];
        args[0] = Tag.Sequence;
        for (int i = 0; i < merged.Count; i++) args[i + 1] = merged[i];
        return Expr.Make(args).WithSource(syntaxTree.Source);
    }

    // Unwrap recognized placement/calling-convention tags before checking declaration shapes for a name.
    static bool TryGetTopLevelDeclName(Expr decl, out string name)
    {
        name = null;
        if (decl == null) return false;

        Expr d = decl;
        int guard = 0;
        // Bound wrapper traversal; unsupported shapes or deeper nesting fall through as unrecognized.
        while (guard++ < 16)
        {
            Expr inner;
            int i;
            string s;
            if (d.Match(Tag.Bank, out i, out inner) ||
                d.Match(Tag.FixedBank, out i, out inner) ||
                d.Match(Tag.FixedOrder, out i, out inner) ||
                d.Match(Tag.DeclAlign, out i, out inner) ||
                d.Match(Tag.DeclSection, out s, out inner) ||
                d.Match(Tag.Unsafe, out inner) ||
                d.Match(Tag.StackCall, out inner))
            {
                d = inner;
                continue;
            }
            break;
        }

        CType t; FieldInfo[] f; Expr body; Expr range;
        if (d.Match(Tag.ReadonlyData, out t, out name, out int[] _rd)) return true;
        if (d.Match(Tag.ReadonlyData, out t, out name, out Expr[] _rdExprs)) return true;
        if (d.Match(Tag.Constant, out t, out name, out Expr _cv)) return true;
        if (d.Match(Tag.Function, out t, out name, out f, out int _mc, out body) ||
            d.Match(Tag.Function, out t, out name, out f, out body)) return true;
        if (d.Match(Tag.InlineFunction, out t, out name, out f, out int _imc, out body) ||
            d.Match(Tag.InlineFunction, out t, out name, out f, out body)) return true;
        if (d.Match(Tag.FunctionDecl, out t, out name, out f, out int _pmc) ||
            d.Match(Tag.FunctionDecl, out t, out name, out f)) return true;
        if (d.Match(Tag.Variable, out MemoryRegion _r, out t, out name, out range) ||
            d.Match(Tag.Variable, out _r, out t, out name)) return true;
        if (d.Match(Tag.ExternVariable, out MemoryRegion _er, out t, out name, out range) ||
            d.Match(Tag.ExternVariable, out _er, out t, out name)) return true;
        return false;
    }

    // Format the accepted palette definitions as four hexadecimal color values per name.
    static string BuildCgbPaletteText()
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB CGB palette DSL");
        sb.AppendLine("# name, c0, c1, c2, c3");
        foreach (var p in CgbPalettePragmas)
        {
            sb.AppendFormat("{0}, 0x{1:X4}, 0x{2:X4}, 0x{3:X4}, 0x{4:X4}\n",
                p.Name, p.Colors[0], p.Colors[1], p.Colors[2], p.Colors[3]);
        }
        return sb.ToString();
    }

    // Route pass dumps through the ordinary debug-output gate with a predictable .txt filename.
    public static void WritePassOutputToFile(string passName, string output)
    {
        WriteDebugFile(string.Format("{0}.txt", passName), output);
    }

    // Expose the same assembly formatting used by internal diagnostic dumps.
    public static string ShowAssemblyPublic(IReadOnlyList<Expr> assembly)
    {
        return ShowAssembly(assembly);
    }

    // Render assembly IR for inspection, preserving labels, comments and optional source positions.
    static string ShowAssembly(IReadOnlyList<Expr> assembly)
    {
        StringBuilder sb = new StringBuilder();
        foreach (Expr e in assembly)
        {
            string line = "";
            string mnemonic, text;
            AsmOperand operand;

            bool isTopLevel = e.MatchTag(Tag.Function);
            if (isTopLevel) sb.AppendLine();
            if (!isTopLevel && !e.MatchTag(Tag.Label)) line = "\t";

            // Emit source annotations as assembly comments so they remain distinct from instructions.
            if (Config.ShowSourcePositionInDebugOutput)
            {
                sb.AppendLine(line + "; <" + e.Source + ">");
            }

            if (e.Match(Tag.Comment, out text)) line += "; " + text;
            else if (e.Match(Tag.Label, out text)) line += text + ":";
            else if (e.Match(Tag.Function, out text)) line += string.Format("; function {0}:", text);
            else if (e.MatchTag(Tag.Function)) line += e.Show();
            else if (e.Match(Tag.Asm, out mnemonic, out operand)) line += FormatAssembly(mnemonic, operand);
            else line += e.Show();

            sb.AppendLine(line);
        }
        return sb.ToString();
    }

    // Show each token with its location and selected payload fields; this is a diagnostic view, not a lexer input format.
    static string ShowTokens(List<Token> tokens)
    {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < tokens.Count; i++)
        {
            Token t = tokens[i];
            // Position can be noisy; keep it compact but present.
            sb.Append(t.Position.ToString());
            sb.Append("\t");
            sb.Append(t.Tag.ToString());
            if (t.Tag == TokenType.INT) sb.Append("\t" + t.Int);
            else if (t.Tag == TokenType.NAME) sb.Append("\t" + t.Name);
            else if (t.Tag == TokenType.STRING) sb.Append("\t\"" + t.Name + "\"");
            else if (t.Tag == TokenType.PRAGMA_BANK) sb.Append("\tbank=" + t.Int);
            else if (t.Tag == TokenType.PRAGMA_FIXED_BANK) sb.Append("\tfixed_bank=" + t.Int);
            else if (t.Tag == TokenType.PRAGMA_FIXED_ORDER) sb.Append("\tfixed_order=" + t.Int);
            else if (t.Tag == TokenType.PRAGMA_WRAMX_BANK) sb.Append("\twramx_bank=" + t.Int);
            sb.AppendLine();
        }
        return sb.ToString();
    }

    // A compact, vibecoding-friendly AST summary.
    // Prints one line per top-level item.
    // Summarize recognized top-level shapes and show unmatched nodes in full.
    // Function statement counts cover immediate sequence children only, not nested statements.
    static string ShowAstSimple(Expr tree)
    {
        StringBuilder sb = new StringBuilder();
        if (tree.Match(Tag.Sequence, out Expr[] items))
        {
            sb.AppendLine("Top-level items: " + items.Length);
            foreach (var e in items)
            {
                string tag = e.GetTag();
                if (tag == Tag.Function || tag == Tag.InlineFunction)
                {
                    if (e.Match<CType, string, FieldInfo[], int, Expr>(tag, out var rt, out var name, out var fields, out var mustCheck, out var body))
                    {
                        int stmtCount = 0;
                        if (body.Match(Tag.Sequence, out Expr[] stmts)) stmtCount = stmts.Length;
                        sb.AppendLine($"{tag}\t{name}({fields.Length} args) -> {rt.Show()}\tstmts={stmtCount}\tmust_check={(mustCheck != 0)}\t@{e.Source}");
                        continue;
                    }
                }
                if (tag == Tag.FunctionDecl)
                {
                    if (e.Match<CType, string, FieldInfo[], int>(tag, out var rt, out var name, out var fields, out var mustCheck))
                    {
                        sb.AppendLine($"{tag}\t{name}({fields.Length} args) -> {rt.Show()}\tmust_check={(mustCheck != 0)}\t@{e.Source}");
                        continue;
                    }
                }
                if (tag == Tag.Variable)
                {
                    if (e.Match<CType, string, int, Expr>(tag, out var vt, out var name, out var regionTag, out var init))
                    {
                        sb.AppendLine($"{tag}\t{name}: {vt.Show()}\tregion={regionTag}\t@{e.Source}");
                        continue;
                    }
                }
                if (tag == Tag.Constant)
                {
                    if (e.Match<CType, string, Expr>(tag, out var ct, out var name, out var value))
                    {
                        sb.AppendLine($"{tag}\t{name}: {ct.Show()} = {value.Show()}\t@{e.Source}");
                        continue;
                    }
                }
                if (tag == Tag.ReadonlyData)
                {
                    if (e.Match<CType, string, int, int[]>(tag, out var dt, out var name, out var bank, out var values))
                    {
                        sb.AppendLine($"{tag}\t{name}: {dt.Show()}\tbank={bank}\tbytes={values.Length}\t@{e.Source}");
                        continue;
                    }
                }

                // Fallback
                sb.AppendLine(tag + "\t" + e.Show() + "\t@" + e.Source);
            }
        }
        else
        {
            sb.AppendLine(tree.Show());
        }
        return sb.ToString();
    }

    // Implicit operands need no trailing operand text; other modes use the operand formatter.
    static string FormatAssembly(string mnemonic, AsmOperand operand)
    {
        string format = (operand.Mode == AddressMode.Implicit) ? "{0}" : "{0} {1}";
        return string.Format(format, mnemonic, operand.Show());
    }

    // Use decimal below 256, including negative values; larger values use the dollar-prefixed hexadecimal form.
    public static string FormatAssemblyInteger(int n)
    {
        if (n < 256) return n.ToString();
        else return string.Format("${0:X}", n);
    }

    // Route uncoded, positionless diagnostics through the same counting and reporting path as coded diagnostics.
    public static void GeneralError(Severity severity, string format, params object[] args)
    {
        GeneralError(severity, Maybe.Nothing, ErrorCode.None, format, args);
    }

    [DebuggerStepThrough]
    public static void Warning(string format, params object[] args) => GeneralError(Severity.Warning, Maybe.Nothing, ErrorCode.None, format, args);
    [DebuggerStepThrough]
    public static void Warning(Maybe<FilePosition> position, string format, params object[] args) => GeneralError(Severity.Warning, position, ErrorCode.None, format, args);
    [DebuggerStepThrough]
    public static void Error(string format, params object[] args) => GeneralError(Severity.Error, Maybe.Nothing, ErrorCode.None, format, args);
    [DebuggerStepThrough]
    public static void Error(Maybe<FilePosition> position, string format, params object[] args) => GeneralError(Severity.Error, position, ErrorCode.None, format, args);
    [DebuggerStepThrough]
    public static void Panic(string format, params object[] args) => GeneralError(Severity.InternalError, Maybe.Nothing, ErrorCode.Internal, format, args);
    [DebuggerStepThrough]
    public static void Panic(Maybe<FilePosition> position, string format, params object[] args) => GeneralError(Severity.InternalError, position, ErrorCode.Internal, format, args);

    // --- New: Diagnostic codes ---
    [DebuggerStepThrough]
    public static void Warning(ErrorCode code, string format, params object[] args) => GeneralError(Severity.Warning, Maybe.Nothing, code, format, args);
    [DebuggerStepThrough]
    public static void Warning(Maybe<FilePosition> position, ErrorCode code, string format, params object[] args) => GeneralError(Severity.Warning, position, code, format, args);
    [DebuggerStepThrough]
    public static void Error(ErrorCode code, string format, params object[] args) => GeneralError(Severity.Error, Maybe.Nothing, code, format, args);
    [DebuggerStepThrough]
    public static void Error(Maybe<FilePosition> position, ErrorCode code, string format, params object[] args) => GeneralError(Severity.Error, position, code, format, args);
    [DebuggerStepThrough]
    public static void Panic(ErrorCode code, string format, params object[] args) => GeneralError(Severity.InternalError, Maybe.Nothing, code, format, args);
    [DebuggerStepThrough]
    public static void Panic(Maybe<FilePosition> position, ErrorCode code, string format, params object[] args) => GeneralError(Severity.InternalError, position, code, format, args);

    [DebuggerStepThrough]
    // Apply severity policy, retain a structured diagnostic, optionally print context, and enforce termination limits.
    public static void GeneralError(Severity severity, Maybe<FilePosition> position, ErrorCode code, string format, params object[] args)
    {
        // Promoted lint warnings count and serialize as errors, while the original severity remains available for the hint.
        Severity effectiveSeverity = severity;
        if (severity == Severity.Warning && ShouldPromoteWarningToError(code))
        {
            effectiveSeverity = Severity.Error;
        }

        // Always show a diagnostic code (KQ0000 for uncoded diagnostics).
        string codeText = ErrorCodeText(code);

        string prefix;
        if (position.HasValue) prefix = string.Format("{0} {1} {2}: ", position.Value.ToString(), SeverityText[effectiveSeverity], codeText);
        else prefix = string.Format("{0} {1}: ", SeverityText[effectiveSeverity], codeText);

        string message = string.Format(format, args);
        string suggestion = GetFixSuggestion(code, message);
        // Record diagnostics even when console output is suppressed; stored source coordinates are one-based, or zero when absent.
        Diagnostics.Add(new DiagnosticEntry
        {
            Severity = SeverityText[effectiveSeverity],
            Code = codeText,
            Message = message,
            Suggestion = suggestion,
            Filename = position.HasValue ? position.Value.Filename : "",
            Line = position.HasValue ? (position.Value.Line + 1) : 0,
            Column = position.HasValue ? (position.Value.Column + 1) : 0,
        });
        if (!SuppressConsoleDiagnostics)
        {
            Console.Error.WriteLine(prefix + message);
            if (severity == Severity.Warning && effectiveSeverity == Severity.Error)
            {
                Console.Error.WriteLine("  hint: strict mode treats this lint warning as an error (use --permissive to keep warnings)");
            }

            // Rich diagnostics: show the source line and caret when a file position is available.
            if (position.HasValue)
            {
                TryWriteSourceContext(position.Value);
            }
            if (!string.IsNullOrEmpty(suggestion))
            {
                Console.Error.WriteLine("  hint: " + suggestion);
            }
        }

        // Update counters after retention/output; internal errors terminate immediately with status 2.
        if (effectiveSeverity == Severity.Warning)
        {
            WarningCount++;
        }
        else if (effectiveSeverity == Severity.Error)
        {
            ErrorCount++;
            // Keep going to collect multiple diagnostics.
            // If errors explode, abort to avoid infinite cascades.
            if (ErrorCount >= MaxErrors)
            {
                Console.Error.WriteLine(string.Format("fatal KQ0000: too many errors (>{0})", MaxErrors));
                Exit(1);
            }
        }
        else if (severity == Severity.InternalError)
        {
            Exit(2);
        }
    }

    // Clamp the numeric code into the four-digit diagnostic namespace, including uncoded KQ0000.
    static string ErrorCodeText(ErrorCode code)
    {
        int n = (int)code;
        if (n < 0) n = 0;
        // KQ0000 ... KQ9999
        if (n > 9999) n = 9999;
        return string.Format("KQ{0:0000}", n);
    }

    // Strict mode promotes only lint codes 2400..2499, not every warning emitted by the compiler.
    static bool ShouldPromoteWarningToError(ErrorCode code)
    {
        if (!StrictDiagnostics) return false;
        int n = (int)code;
        return n >= 2400 && n < 2500;
    }

    // Prefer an explanation tied to the diagnostic code; a small message-based fallback handles uncoded failures.
    static string GetFixSuggestion(ErrorCode code, string message)
    {
        switch (code)
        {
            case ErrorCode.ExpectedToken:
                return "Check matching tokens (; ) ] }) and ensure the previous expression/declarator is properly closed.";
            case ErrorCode.ExpectedType:
                return "A type is required here. Add a valid type specifier such as u8/u16/s8/s16/struct/enum.";
            case ErrorCode.ParseError:
                return "Check the previous line for missing brackets, commas, or semicolons.";
            case ErrorCode.ConstAssign:
            case ErrorCode.ConstModify:
                return "You are modifying a const-qualified object. Write to a non-const variable or revise qualifiers.";
            case ErrorCode.StaticAssertFailed:
                return "Revisit the static_assert condition and the constants it depends on.";
            case ErrorCode.RangeViolation:
                return "Adjust the assigned value or the __range declaration so the value stays in range.";
            case ErrorCode.RangeIndexOob:
                return "Align array length and possible index range; clamp the index if needed.";
            case ErrorCode.SwitchImplicitFallthrough:
                return "If fallthrough is intentional, add 'fallthrough;' explicitly.";
            case ErrorCode.SwitchFallthroughUsage:
                return "Use 'fallthrough;' only as the last statement in a case block.";
            case ErrorCode.UnreachableCode:
                return "Remove statements after return/goto/break/continue or split control flow.";
            case ErrorCode.UnusedSymbol:
                return "The symbol is unused. Remove it or add a real use-site.";
            case ErrorCode.ImplicitNarrowing:
                return "If narrowing is intentional, add an explicit cast.";
            case ErrorCode.PointerArithmeticDanger:
                return "Review pointer-offset type/range and add explicit casts if intentional.";
            case ErrorCode.ExternUndefined:
                return "Add a matching definition for this extern declaration.";
            case ErrorCode.ExternTypeMismatch:
                return "Make the extern declaration type match the actual definition type.";
            case ErrorCode.ExternNotSupported:
                return "For const-scalar address-taking, consider enabling -Zconst-scalar-in-rom.";
            case ErrorCode.InvalidWramXBank:
                return "Use WRAMX bank numbers in the range 1..7.";
            case ErrorCode.BankedWramRequiresCgbOnly:
                return "Build with #pragma rom_cgb cgb_only or --cgb=cgb_only before using banked WRAM/SVBK.";
            case ErrorCode.BankedWramLocalNotAllowed:
                return "Move this declaration to file scope; MVP banked WRAM is only for globals/file-statics.";
            case ErrorCode.WramXBankOverflow:
                return "Reduce the allocation size or move data into a different WRAMX bank.";
            case ErrorCode.ManualSvbkRequired:
                return "Save SVBK, switch to the symbol's bank, use the data, then restore the previous SVBK.";
            default:
                break;
        }

        if (!string.IsNullOrEmpty(message))
        {
            if (message.IndexOf("unknown option", StringComparison.OrdinalIgnoreCase) >= 0)
                return "Use --help to check valid option names.";
            if (message.IndexOf("division by zero", StringComparison.OrdinalIgnoreCase) >= 0)
                return "Fix the constant expression so it does not divide by zero.";
        }

        return "";
    }

    // Show a bounded source preview when available; missing files or display failures must not hide the main diagnostic.
    static void TryWriteSourceContext(FilePosition pos)
    {
        try
        {
            if (string.IsNullOrEmpty(pos.Filename) || pos.Filename == "<unknown>") return;

            // Read each source into the diagnostic cache on first use rather than reopening it for every error.
            if (!SourceCache.TryGetValue(pos.Filename, out var lines))
            {
                if (!File.Exists(pos.Filename)) return;
                lines = IoUtil.ReadAllLinesUtf8(pos.Filename);
                SourceCache[pos.Filename] = lines;
            }

            if (pos.Line < 0 || pos.Line >= lines.Length) return;

            string line = lines[pos.Line] ?? "";
            // Avoid huge output for long lines.
            const int MaxPreview = 200;
            string shown = (line.Length > MaxPreview) ? line.Substring(0, MaxPreview) + "…" : line;

            Console.Error.WriteLine("  " + shown);

            // Clamp the marker to the displayed prefix when the source position falls outside the preview.
            int col = pos.Column;
            if (col < 0) col = 0;
            if (col > shown.Length) col = shown.Length;

            // Estimate marker indentation as four columns per tab; the displayed source line remains unchanged.
            string prefixText = shown.Substring(0, col);
            int visualCols = 0;
            foreach (char c in prefixText)
            {
                if (c == '\t') visualCols += 4;
                else visualCols += 1;
            }
            Console.Error.WriteLine("  " + new string(' ', visualCols) + "^");
        }
        catch
        {
            // Ignore context printing failures; main error is already printed.
        }
    }


    // Token context printing: show surrounding tokens to make parser errors easier to understand.
    // Callers provide up to a few previous-consumed tokens and the remaining input tokens.
    public static void TryWriteTokenContext(IEnumerable<Token> prevTokens, IReadOnlyList<Token> remainingTokens, int prevCount = 3, int nextCount = 3)
    {
        try
        {
            // Console suppression leaves structured diagnostics intact; token-context output is optional.
            if (SuppressConsoleDiagnostics) return;
            if (prevTokens == null || remainingTokens == null) return;

            // Take last prevCount tokens.
            List<Token> prev = prevTokens as List<Token>;
            if (prev == null) prev = new List<Token>(prevTokens);
            if (prev.Count > prevCount) prev = prev.GetRange(prev.Count - prevCount, prevCount);

            // Take nextCount tokens from remaining.
            // Bound lookahead by the available tokens; the current token is bracketed separately below.
            int take = Math.Min(nextCount, remainingTokens.Count);
            List<Token> next = new List<Token>();
            for (int i = 0; i < take; i++) next.Add(remainingTokens[i]);

            // Build a compact preview.
            StringBuilder sb = new StringBuilder();
            sb.Append("  tokens: ");

            for (int i = 0; i < prev.Count; i++)
            {
                sb.Append(prev[i].Show());
                sb.Append(' ');
            }

            // Current token is remainingTokens[0] if present.
            if (remainingTokens.Count > 0)
            {
                sb.Append('[');
                sb.Append(remainingTokens[0].Show());
                sb.Append(']');
                sb.Append(' ');
            }

            for (int i = 1; i < next.Count; i++)
            {
                sb.Append(next[i].Show());
                sb.Append(' ');
            }

            Console.Error.WriteLine(sb.ToString().TrimEnd());
        }
        catch
        {
            // Best-effort only
        }
    }


    [DebuggerStepThrough]
    // Finish diagnostic/failure bookkeeping before ending this CLI process with the supplied status.
    static void Exit(int code)
    {
        // Attempt requested diagnostics before failure-only helpers or process termination.
        TryWriteDiagnosticJson(code);

        // Run automatic reduction once per invocation and avoid starting it recursively from an explicit minimizer run.
        if (code != 0 && _autoMinimizeOnFail && !_autoMinimizeTriggered && !Minimizer.IsMinimizeRequested(_originalArgs))
        {
            _autoMinimizeTriggered = true;
            TryRunAutoMinimize();
        }

        // Collect a requested failure package before terminating, after any automatic minimization attempt.
        if (code != 0 && EmitReproPackageOnFail)
        {
            TryWriteFailureReproPackage(code);
        }

        if (code != 0)
        {
            if (AttachDebuggerOnError) Debugger.Launch();
            if (Debugger.IsAttached) Debugger.Break();
        }
        Environment.Exit(code);
    }

    // Replace the dependency snapshot, dropping blank entries but retaining supplied order and duplicates.
    public static void SetLastCompilationDependencies(IEnumerable<string> deps)
    {
        _lastCompilationDependencies.Clear();
        if (deps == null) return;
        foreach (var d in deps)
        {
            if (string.IsNullOrWhiteSpace(d)) continue;
            _lastCompilationDependencies.Add(d);
        }
    }

    // Normalize include directories relative to the current process directory and deduplicate case-insensitively.
    // This accepts paths without checking that the directory already exists.
    static void AddIncludeDirectory(string path)
    {
        if (string.IsNullOrWhiteSpace(path)) return;
        try
        {
            string full = Path.GetFullPath(path.Trim());
            if (!IncludeDirectories.Any(x => string.Equals(x, full, StringComparison.OrdinalIgnoreCase)))
                IncludeDirectories.Add(full);
        }
        catch
        {
            Warning("warning: invalid include directory ignored: " + path);
        }
    }

    // Change only the listed debug, disassembly, optimization, RST and cache settings; other explicit options remain in force.
    static void ApplyProfilePreset(string profileName)
    {
        string p = (profileName ?? "").Trim().ToLowerInvariant();
        if (p == "dev")
        {
            EnableDebugOutput = true;
            DisableDisasm = false;
            OptLevel = 0;
            RstDisable = true;
            EnableIncrementalCache = true;
            return;
        }
        if (p == "release")
        {
            EnableDebugOutput = false;
            DisableDisasm = true;
            OptLevel = 1;
            RstDisable = false;
            EnableIncrementalCache = true;
            return;
        }
        if (p == "test")
        {
            EnableDebugOutput = true;
            DisableDisasm = true;
            OptLevel = 0;
            RstDisable = true;
            EnableIncrementalCache = false;
            return;
        }
        Error("error: --profile must be dev|release|test");
    }

    // Disable cache shortcuts for the listed outputs that require fresh pipeline work.
    // This is a feature-eligibility check, not validation of source inputs or diagnostic status.
    static bool CanUseBuildCacheForThisRun()
    {
        if (TraceEnabled) return false;
        if (EmitVarList) return false;
        if (EnableDebugOutput) return false;
        if (EmitBankSimReport || EmitFarcallSuggestionReport || EmitCrossBankCallReport ||
            EmitAbiVerifyReport || EmitAbiDiffReport || EmitRstApplyReport || EmitOptDiffReport ||
            EmitFunctionSizeReport || EmitHotspotReport || EnableReproCheck ||
            EmitCgbConsistencyReport || EmitCgbSymbolVerifyReport || EmitKurosakiMetadata)
            return false;
        return true;
    }

    // Restore a ROM and required sidecars from this working directory when the computed key has a complete entry.
    static bool TryRestoreBuildCache(List<string> sourceFilenames, string outputFilename, out string cacheKey)
    {
        cacheKey = ComputeBuildCacheKey(sourceFilenames);
        // An empty key disables the shortcut and lets normal compilation handle the input.
        if (string.IsNullOrEmpty(cacheKey)) return false;

        string cacheDir = Path.Combine(Environment.CurrentDirectory, ".kitaqfc_cache", cacheKey);
        string cachedPrimary = Path.Combine(cacheDir, GetCachePrimaryArtifactName(outputFilename));
        if (!File.Exists(cachedPrimary)) return false;

        string cachedMap = Path.Combine(cacheDir, "out.map");
        string cachedDbg = Path.Combine(cacheDir, "out.dbg");
        string cachedDbc = Path.Combine(cacheDir, "out.dbc");
        string cachedBanks = Path.Combine(cacheDir, "out.banks.txt");
        string cachedFuncSizes = Path.Combine(cacheDir, "out.funcsizes.txt");
        // Require all five core sidecars before copying any cached output.
        if (!File.Exists(cachedMap) || !File.Exists(cachedDbg) || !File.Exists(cachedDbc) || !File.Exists(cachedBanks) || !File.Exists(cachedFuncSizes)) return false;

        try
        {
            IoUtil.CopyFileRobust(cachedPrimary, outputFilename, true);
            IoUtil.CopyFileRobust(cachedMap, Path.ChangeExtension(outputFilename, ".map"), true);
            IoUtil.CopyFileRobust(cachedDbg, Path.ChangeExtension(outputFilename, ".dbg"), true);
            IoUtil.CopyFileRobust(cachedDbc, Path.ChangeExtension(outputFilename, ".dbc"), true);
            IoUtil.CopyFileRobust(cachedBanks, Path.ChangeExtension(outputFilename, ".banks.txt"), true);
            IoUtil.CopyFileRobust(cachedFuncSizes, Path.ChangeExtension(outputFilename, ".funcsizes.txt"), true);
            if (EmitDependenciesList)
            {
                // Dependency output is optional in the entry; its absence does not currently turn a restore into a miss.
                string cachedDeps = Path.Combine(cacheDir, "out.deps.txt");
                if (File.Exists(cachedDeps))
                    IoUtil.CopyFileRobust(cachedDeps, ResolveDepsOutputPath(outputFilename), true);
            }
            return true;
        }
        catch (Exception ex)
        {
            // Fall back to compilation on copy failure; earlier successful copies are not rolled back.
            Warning("warning: cache restore skipped: " + ex.Message);
            return false;
        }
    }

    // Copy the completed output set into the key directory; this writes individual files, not an atomic cache transaction.
    static void TrySaveBuildCache(string cacheKey, string outputFilename)
    {
        if (string.IsNullOrEmpty(cacheKey)) return;
        string cacheDir = Path.Combine(Environment.CurrentDirectory, ".kitaqfc_cache", cacheKey);
        // Directory creation precedes the copy-error handler and can propagate a failure to the compilation caller.
        Directory.CreateDirectory(cacheDir);
        try
        {
            IoUtil.CopyFileRobust(outputFilename, Path.Combine(cacheDir, GetCachePrimaryArtifactName(outputFilename)), true);
            IoUtil.CopyFileRobust(Path.ChangeExtension(outputFilename, ".map"), Path.Combine(cacheDir, "out.map"), true);
            IoUtil.CopyFileRobust(Path.ChangeExtension(outputFilename, ".dbg"), Path.Combine(cacheDir, "out.dbg"), true);
            IoUtil.CopyFileRobust(Path.ChangeExtension(outputFilename, ".dbc"), Path.Combine(cacheDir, "out.dbc"), true);
            IoUtil.CopyFileRobust(Path.ChangeExtension(outputFilename, ".banks.txt"), Path.Combine(cacheDir, "out.banks.txt"), true);
            IoUtil.CopyFileRobust(Path.ChangeExtension(outputFilename, ".funcsizes.txt"), Path.Combine(cacheDir, "out.funcsizes.txt"), true);
            // Copy an existing dependency list if present, including one produced by an earlier invocation.
            string depsPath = ResolveDepsOutputPath(outputFilename);
            if (File.Exists(depsPath))
                IoUtil.CopyFileRobust(depsPath, Path.Combine(cacheDir, "out.deps.txt"), true);
        }
        catch (Exception ex)
        {
            Warning("warning: cache save skipped: " + ex.Message);
        }
    }

    // Prefer an explicit dependency path; otherwise replace the ROM extension with .deps.txt.
    static string ResolveDepsOutputPath(string outputFilename)
    {
        if (!string.IsNullOrWhiteSpace(DependenciesListPath)) return DependenciesListPath;
        return Path.ChangeExtension(outputFilename, ".deps.txt");
    }

    // Name the cached primary artifact by the requested extension, using the target default when none is present.
    static string GetCachePrimaryArtifactName(string outputFilename)
    {
        string ext = Path.GetExtension(outputFilename);
        if (string.IsNullOrWhiteSpace(ext)) ext = TargetInfo.DefaultOutputExtension;
        return "out" + ext;
    }

    // An explicit container selection overrides the extension; auto mode recognizes .fds case-insensitively.
    public static bool IsFdsPrimaryOutput(string outputFilename)
    {
        if (OutputContainerFormat == NesOutputContainerFormat.Fds) return true;
        if (OutputContainerFormat == NesOutputContainerFormat.INes) return false;
        return string.Equals(Path.GetExtension(outputFilename ?? ""), ".fds", StringComparison.OrdinalIgnoreCase);
    }

    // Avoid requesting a second disk image when the primary output already uses the FDS container.
    public static bool ShouldEmitFdsSidecar(string outputFilename)
    {
        if (!EmitFdsSidecar) return false;
        return !IsFdsPrimaryOutput(outputFilename);
    }

    // Resolve the default sentinel to the primary output basename with an .fds extension.
    public static string ResolveFdsImageOutputPath(string outputFilename)
    {
        string p = FdsImageOutputPath ?? "";
        if (string.IsNullOrWhiteSpace(p) || p == "__default__")
            return Path.ChangeExtension(outputFilename, ".fds");
        return p;
    }

    // Serialize normalized disk metadata only when its output option is enabled; failures are warnings.
    static void TryWriteFdsMetadataOut(string outputFilename)
    {
        if (string.IsNullOrWhiteSpace(FdsMetadataOutPath)) return;
        try
        {
            string path = FdsMetadataOutPath;
            if (string.IsNullOrWhiteSpace(path) || path == "__default__") path = Path.ChangeExtension(outputFilename, ".fdsmeta.json");
            IoUtil.WriteAllTextUtf8Robust(path, (FdsMetadata ?? FdsDiskMetadata.Empty).ToNormalizedJson(), allowAlternatePath: true);
            Console.WriteLine("[fds] metadata: " + path);
        }
        catch (Exception ex)
        {
            Warning("warning: failed to write FDS metadata: " + ex.Message);
        }
    }

    // Write normalized source/dependency paths when requested; listing failures produce a warning.
    static void TryWriteDependenciesList(List<string> sourceFilenames, string outputFilename)
    {
        if (!EmitDependenciesList) return;
        try
        {
            string path = ResolveDepsOutputPath(outputFilename);
            // Deduplicate full paths case-insensitively before sorting them for stable listing order.
            var deps = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (var d in LastCompilationDependencies ?? Array.Empty<string>())
            {
                if (!string.IsNullOrWhiteSpace(d)) deps.Add(Path.GetFullPath(d));
            }
            if (!string.IsNullOrWhiteSpace(FdsMetadataPath) && File.Exists(FdsMetadataPath))
                deps.Add(Path.GetFullPath(FdsMetadataPath));
            foreach (var f in (FdsMetadata == null ? FdsDiskMetadata.Empty : FdsMetadata).Files)
            {
                string src = f.SourcePath ?? "";
                if (string.IsNullOrWhiteSpace(src)) continue;
                string baseDir = string.IsNullOrWhiteSpace(FdsMetadata.SourcePath) ? "" : (Path.GetDirectoryName(FdsMetadata.SourcePath) ?? "");
                string full = Path.IsPathRooted(src) ? src : Path.Combine(baseDir, src);
                if (File.Exists(full)) deps.Add(Path.GetFullPath(full));
            }

            foreach (var s in sourceFilenames ?? new List<string>())
            {
                if (!string.IsNullOrWhiteSpace(s)) deps.Add(Path.GetFullPath(s));
            }

            var lines = new List<string>();
            lines.Add("# KITAQGB dependency list");
            // The generation timestamp is informational; subsequent lines are dependency paths.
            lines.Add("# generated_utc=" + DateTime.UtcNow.ToString("O"));
            foreach (var d in deps.OrderBy(x => x, StringComparer.OrdinalIgnoreCase))
                lines.Add(d);
            IoUtil.WriteAllLinesUtf8Robust(path, lines, allowAlternatePath: true);
        }
        catch (Exception ex)
        {
            Warning("warning: failed to write deps list: " + ex.Message);
        }
    }

    // Hash the selected settings, executable stamp and discovered file metadata.
    // This implementation fingerprints source size/time, not source contents or the complete parsed dependency graph.
    static string ComputeBuildCacheKey(List<string> sourceFilenames)
    {
        try
        {
            var files = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            if (!string.IsNullOrWhiteSpace(FdsMetadataPath) && File.Exists(FdsMetadataPath))
                files.Add(Path.GetFullPath(FdsMetadataPath));
            foreach (var f in (FdsMetadata == null ? FdsDiskMetadata.Empty : FdsMetadata).Files)
            {
                string src = f.SourcePath ?? "";
                if (string.IsNullOrWhiteSpace(src)) continue;
                string baseDir = string.IsNullOrWhiteSpace(FdsMetadata.SourcePath) ? "" : (Path.GetDirectoryName(FdsMetadata.SourcePath) ?? "");
                string full = Path.IsPathRooted(src) ? src : Path.Combine(baseDir, src);
                if (File.Exists(full)) files.Add(Path.GetFullPath(full));
            }

            foreach (var s in sourceFilenames ?? new List<string>())
            {
                string full = Path.GetFullPath(s);
                if (File.Exists(full)) files.Add(full);

                string dir = Path.GetDirectoryName(full) ?? "";
                if (!string.IsNullOrWhiteSpace(dir) && Directory.Exists(dir))
                {
                    foreach (var h in Directory.GetFiles(dir, "*.h", SearchOption.AllDirectories))
                        files.Add(h);
                }
            }

            // Recursively include headers beneath configured include directories, even when they are not used by this build.
            foreach (var d in IncludeDirectories)
            {
                if (!Directory.Exists(d)) continue;
                foreach (var h in Directory.GetFiles(d, "*.h", SearchOption.AllDirectories))
                    files.Add(h);
            }

            // Fingerprint the running host executable; an embedded API host is not necessarily the compiler assembly.
            string exePath = Process.GetCurrentProcess().MainModule.FileName;
            // Leave the executable stamp empty if its metadata or content hash cannot be read.
            string exeStamp = "";
            try
            {
                var fi = new FileInfo(exePath);
                using (var sha = SHA256.Create())
                using (var fs = File.OpenRead(exePath))
                {
                    var hash = sha.ComputeHash(fs);
                    exeStamp = fi.Length + "|" + fi.LastWriteTimeUtc.Ticks + "|" +
                               string.Concat(hash.Select(b => b.ToString("x2")));
                }
            }
            catch { }

            var sb = new StringBuilder();
            sb.AppendLine("kitaqfc_cache_v2");
            sb.AppendLine(exeStamp);
            sb.AppendLine("opt=" + OptLevel);
            sb.AppendLine("abi=" + Abi);
            sb.AppendLine("diag_mode=" + DiagnosticsMode);
            sb.AppendLine("rst_disable=" + RstDisable);
            sb.AppendLine("rst_use_38=" + RstUse38);
            sb.AppendLine("rst_unsafe=" + RstUnsafe);
            sb.AppendLine("rst_speed_safe=" + RstSpeedSafe);
            sb.AppendLine("rom_title=" + (RomHeader.Title ?? ""));
            sb.AppendLine("cgb=" + (RomHeader.CgbFlag.HasValue ? RomHeader.CgbFlag.Value.ToString() : ""));
            sb.AppendLine("cart=" + (RomHeader.CartType.HasValue ? RomHeader.CartType.Value.ToString() : ""));
            sb.AppendLine("romsize=" + (RomHeader.RomSizeCode.HasValue ? RomHeader.RomSizeCode.Value.ToString() : ""));
            sb.AppendLine("ramsize=" + (RomHeader.RamSizeCode.HasValue ? RomHeader.RamSizeCode.Value.ToString() : ""));
            sb.AppendLine("target=" + TargetInfo.CliName);
            sb.AppendLine("target_ext=" + TargetInfo.DefaultOutputExtension);
            sb.AppendLine("nes_mapper=" + NesMapperProfile.CliName);
            sb.AppendLine("nes_board=" + NesMapperProfile.BoardCliName);
            sb.AppendLine("fds_meta=" + (FdsMetadataPath ?? ""));
            sb.AppendLine("output_container=" + OutputContainerFormat);
            sb.AppendLine("fds_sidecar=" + EmitFdsSidecar);
            sb.AppendLine("fds_header=" + FdsImageHeaderEnabled);
            sb.AppendLine("fds_game_code=" + (FdsGameCode ?? ""));
            sb.AppendLine("fds_license_bypass=" + FdsLicenseBypassEnabled);
            sb.AppendLine("fds_prgram_layout=" + FdsPrgRamLayoutEnabled);
            sb.AppendLine("fds_auto_overlay=" + FdsAutoOverlayEnabled);
            sb.AppendLine("fds_overlay_start_id=" + FdsOverlayStartId);
            sb.AppendLine("fds_overlay_prefix=" + (FdsOverlayNamePrefix ?? ""));
            sb.AppendLine("fds_overlay_trim=" + FdsOverlayTrimTrailingFill);
            sb.AppendLine("fds_overlay_farcall=" + FdsOverlayFarcallEnabled);
            sb.AppendLine("fds_overlay_guard=" + FdsOverlayResidencyGuardEnabled);
            sb.AppendLine("fds_overlay_table=" + FdsOverlayFunctionTableEnabled);

            // Sort the discovered set for deterministic metadata ordering; this does not preserve source or include-search order.
            foreach (string f in files.OrderBy(x => x, StringComparer.OrdinalIgnoreCase))
            {
                try
                {
                    var fi = new FileInfo(f);
                    // Metadata-only fingerprints cannot distinguish equal-size edits that preserve the file timestamp.
                    sb.AppendLine(f + "|" + fi.Length + "|" + fi.LastWriteTimeUtc.Ticks);
                }
                catch
                {
                    sb.AppendLine(f + "|missing");
                }
            }

            using (var sha = SHA256.Create())
            {
                // Encode the assembled fingerprint text as UTF-8 before producing its SHA-256 directory key.
                var data = IoUtil.Utf8NoBom.GetBytes(sb.ToString());
                var hash = sha.ComputeHash(data);
                return string.Concat(hash.Select(b => b.ToString("x2")));
            }
        }
        catch
        {
            return "";
        }
    }

    // Hold estimated file-offset high-water marks and function extents, separate from final assembler reports.
    sealed class BankSimulationResult
    {
        public int[] BankMaxPc = new int[0];
        public int RomSizeBytes;
        public int UsedBytes;
        public List<FunctionSizeInfo> FunctionSizes = new List<FunctionSizeInfo>();
    }

    // Dispatch requested structural reports after assembly; verification reports can also add compiler errors.
    static void RunAnalysisReports(List<string> sourceFilenames, IReadOnlyList<Expr> assembly, string outputFilename, RomHeaderOptions effectiveRomHeader, ICompilationBackend backend)
    {
        // Skip the whole dispatcher when the selected backend does not advertise these reports.
        if (backend == null || !backend.TargetInfo.SupportsAnalysisReports) return;

        var codegen = backend.CodegenReport ?? new CodegenAnalysisReport();
        var opt = Optimizer.LastReport ?? new OptimizerAnalysisReport();
        var asm = backend.AssemblerReport ?? new AssemblerAnalysisReport();

        if (EmitBankSimReport)
        {
            string path = ResolveReportPath(BankSimReportPath, outputFilename, ".bank_sim.txt");
            var sim = SimulateBankLayout(assembly);
            IoUtil.WriteAllTextUtf8Robust(path, BuildBankSimReportText(sim), allowAlternatePath: true);
            Console.WriteLine("[report] bank simulation: " + path);
        }

        if (EmitFarcallSuggestionReport)
        {
            string path = ResolveReportPath(FarcallSuggestionReportPath, outputFilename, ".farcall_suggestions.txt");
            IoUtil.WriteAllTextUtf8Robust(path, BuildFarcallSuggestionText(codegen), allowAlternatePath: true);
            Console.WriteLine("[report] farcall suggestions: " + path);
        }

        if (EmitCrossBankCallReport)
        {
            string path = ResolveReportPath(CrossBankCallReportPath, outputFilename, ".cross_bank_calls.txt");
            IoUtil.WriteAllTextUtf8Robust(path, BuildCrossBankCallText(codegen), allowAlternatePath: true);
            Console.WriteLine("[report] cross-bank calls: " + path);
        }

        // Write the recorded ABI issues, then fail compilation when that collected issue list is nonempty.
        if (EmitAbiVerifyReport)
        {
            string path = ResolveReportPath(AbiVerifyReportPath, outputFilename, ".abi_verify.txt");
            var issues = new List<string>(codegen.AbiIssues ?? new List<string>());
            string text = BuildAbiVerifyText(codegen, issues);
            IoUtil.WriteAllTextUtf8Robust(path, text, allowAlternatePath: true);
            Console.WriteLine("[report] abi verify: " + path);
            if (issues.Count > 0)
            {
                Error("ABI verification failed ({0} issue(s)). See: {1}", issues.Count, path);
            }
        }

        if (EmitRstApplyReport)
        {
            string path = ResolveReportPath(RstApplyReportPath, outputFilename, ".rst_report.txt");
            IoUtil.WriteAllTextUtf8Robust(path, BuildRstApplyText(codegen, opt), allowAlternatePath: true);
            Console.WriteLine("[report] rst apply: " + path);
        }

        if (EmitOptDiffReport)
        {
            string path = ResolveReportPath(OptDiffReportPath, outputFilename, ".opt_diff.txt");
            IoUtil.WriteAllTextUtf8Robust(path, BuildOptimizerDiffText(opt), allowAlternatePath: true);
            Console.WriteLine("[report] optimizer diff: " + path);
        }

        if (EmitFunctionSizeReport)
        {
            string path = ResolveReportPath(FunctionSizeReportPath, outputFilename, ".funcsizes.txt");
            IoUtil.WriteAllTextUtf8Robust(path, BuildFunctionSizeReportText(asm), allowAlternatePath: true);
            Console.WriteLine("[report] function sizes: " + path);
        }

        if (EmitHotspotReport)
        {
            string path = ResolveReportPath(HotspotReportPath, outputFilename, ".hotspots.txt");
            IoUtil.WriteAllTextUtf8Robust(path, BuildHotspotReportText(codegen, asm), allowAlternatePath: true);
            Console.WriteLine("[report] hotspots: " + path);
        }

        // Emit recorded NES optimization decisions separately from generic optimizer pass diffs.
        if (EmitNesOptimizationReport)
        {
            string path = ResolveReportPath(NesOptimizationReportPath, outputFilename, ".nesopt.txt");
            IoUtil.WriteAllTextUtf8Robust(path, BuildNesOptimizationReportText(codegen), allowAlternatePath: true);
            Console.WriteLine("[report] NES optimizations: " + path);
        }

        // Emit the consistency details before reporting a failing result to the compiler diagnostic path.
        if (EmitCgbConsistencyReport)
        {
            string path = ResolveReportPath(CgbConsistencyReportPath, outputFilename, ".cgb_consistency.txt");
            var result = AnalyzeCgbConsistency(assembly, codegen, outputFilename, effectiveRomHeader);
            IoUtil.WriteAllTextUtf8Robust(path, BuildCgbConsistencyText(result), allowAlternatePath: true);
            Console.WriteLine("[report] cgb consistency: " + path);
            if (!result.Pass)
            {
                Error("CGB consistency check failed ({0} issue(s)). See: {1}", result.Issues.Count, path);
            }
        }

        // Compare expected CGB symbols with the output map and retain the missing-symbol report on failure.
        if (EmitCgbSymbolVerifyReport)
        {
            string path = ResolveReportPath(CgbSymbolVerifyReportPath, outputFilename, ".cgb_symbols.txt");
            var result = AnalyzeCgbSymbolVerification(assembly, Path.ChangeExtension(outputFilename, ".map"));
            IoUtil.WriteAllTextUtf8Robust(path, BuildCgbSymbolVerifyText(result), allowAlternatePath: true);
            Console.WriteLine("[report] cgb symbols: " + path);
            if (!result.Pass)
            {
                Error("CGB symbol verification failed ({0} missing symbol(s)). See: {1}", result.Missing.Count, path);
            }
        }

        // Package target-specific compiler metadata for KUROSAKI using the selected cartridge profile.
        if (EmitKurosakiMetadata)
        {
            string path = ResolveReportPath(KurosakiMetadataPath, outputFilename, ".kurosaki.json");
            IoUtil.WriteAllTextUtf8Robust(path, BuildKurosakiMetadataJson(codegen, asm, outputFilename), allowAlternatePath: true);
            Console.WriteLine("[report] KUROSAKI metadata: " + path);
        }

        if (EmitAbiDiffReport)
        {
            string path = ResolveReportPath(AbiDiffReportPath, outputFilename, ".abi_diff.txt");
            TryGenerateAbiDiffReport(path);
        }

        if (EnableReproCheck)
        {
            string path = ResolveReportPath(ReproCheckReportPath, outputFilename, ".repro_check.txt");
            TryRunReproCheck(outputFilename, path);
        }
    }

    // Keep an explicit path unchanged; otherwise place a suffixed report beside the output ROM.
    static string ResolveReportPath(string requestedPath, string outputFilename, string suffix)
    {
        if (!string.IsNullOrWhiteSpace(requestedPath)) return requestedPath;
        string dir = Path.GetDirectoryName(outputFilename);
        if (string.IsNullOrWhiteSpace(dir)) dir = ".";
        string stem = Path.GetFileNameWithoutExtension(outputFilename);
        return Path.Combine(dir, stem + suffix);
    }

    // Estimate layout from selected IR nodes using the legacy 16 KiB bank model.
    // This is not a substitute for target-specific assembler placement or exact encoded instruction sizes.
    static BankSimulationResult SimulateBankLayout(IReadOnlyList<Expr> assembly)
    {
        const int bankSize = 0x4000;
        const int codeStart = 0x0160;
        const int minRomSize = 0x8000;
        const int maxRomSize = 0x800000;

        int pc = codeStart;
        int maxPc = pc;
        var bankMax = new Dictionary<int, int>();
        var functionSizes = new List<FunctionSizeInfo>();
        string currentFunctionName = null;
        int currentFunctionStart = 0;
        int currentFunctionBytes = 0;

        // Track the supplied cursor in its containing bank and retain the greatest cursor seen overall.
        Action<int> updateBankMax = (value) =>
        {
            if (value < 0) return;
            int bank = value / bankSize;
            int cur = 0;
            bankMax.TryGetValue(bank, out cur);
            if (value > cur) bankMax[bank] = value;
            if (value > maxPc) maxPc = value;
        };

        // Close a function using counted instruction/data bytes; alignment and explicit cursor jumps are not included in that byte count.
        Action finalizeFunction = () =>
        {
            if (string.IsNullOrEmpty(currentFunctionName)) return;
            functionSizes.Add(new FunctionSizeInfo
            {
                Name = currentFunctionName,
                StartFileOffset = currentFunctionStart,
                EndFileOffset = currentFunctionStart + currentFunctionBytes,
                SizeBytes = currentFunctionBytes,
                Bank = currentFunctionStart >> 14,
                CpuAddress = BuildReportUtil.CpuAddrFromFileOffset(currentFunctionStart)
            });
            currentFunctionName = null;
            currentFunctionStart = 0;
            currentFunctionBytes = 0;
        };

        updateBankMax(pc);

        foreach (Expr e in assembly ?? new List<Expr>())
        {
            // A new function marker closes the prior record and starts counting bytes at the current cursor.
            if (e.Match(Tag.Function, out string fn))
            {
                finalizeFunction();
                currentFunctionName = fn;
                currentFunctionStart = pc;
                currentFunctionBytes = 0;
                continue;
            }

            // Treat SkipTo(0) as a minimum code-start constraint; other values replace the cursor directly.
            if (e.Match(Tag.SkipTo, out int skip))
            {
                if (skip == 0)
                {
                    if (pc < codeStart) pc = codeStart;
                }
                else
                {
                    pc = skip;
                }
                updateBankMax(pc);
                continue;
            }

            // Round the cursor with a bit mask, assuming positive alignments are powers of two.
            if (e.Match(Tag.Align, out int align))
            {
                if (align > 0)
                {
                    int mask = align - 1;
                    pc = (pc + mask) & ~mask;
                    updateBankMax(pc);
                }
                continue;
            }

            // Count inline byte arrays in both total layout and the active function estimate.
            if (e.Match(Tag.ReadonlyData, out string rdName, out byte[] rdBytes))
            {
                int n = rdBytes == null ? 0 : rdBytes.Length;
                pc += n;
                if (!string.IsNullOrEmpty(currentFunctionName)) currentFunctionBytes += n;
                updateBankMax(pc);
                continue;
            }

            // A symbolic word contributes two bytes even though its value is resolved later.
            if (e.Match(Tag.Word, out string wordLabel))
            {
                pc += 2;
                if (!string.IsNullOrEmpty(currentFunctionName)) currentFunctionBytes += 2;
                updateBankMax(pc);
                continue;
            }

            if (e.Match(Tag.Asm, out string mnemonic, out AsmOperand operand))
            {
                // Use one opcode byte plus an addressing-mode estimate; this does not consult the opcode encoding table.
                int n = 1 + OperandBytes(operand == null ? AddressMode.Implicit : operand.Mode);
                pc += n;
                if (!string.IsNullOrEmpty(currentFunctionName)) currentFunctionBytes += n;
                updateBankMax(pc);
                continue;
            }
        }
        finalizeFunction();

        // Round the greatest cursor up to a bank boundary, then clamp the estimate to the supported size limits.
        int romSize = Math.Max(minRomSize, RoundUpInt(maxPc, bankSize));
        if (romSize > maxRomSize) romSize = maxRomSize;
        int bankCount = Math.Max(2, romSize / bankSize);
        int[] bankMaxPc = new int[bankCount];
        for (int i = 0; i < bankCount; i++)
        {
            int v = 0;
            bankMax.TryGetValue(i, out v);
            bankMaxPc[i] = v;
        }

        return new BankSimulationResult
        {
            BankMaxPc = bankMaxPc,
            RomSizeBytes = romSize,
            UsedBytes = Math.Max(codeStart, pc),
            FunctionSizes = functionSizes
        };
    }

    // Estimate operand length by addressing mode for the lightweight simulator.
    static int OperandBytes(AddressMode mode)
    {
        if (mode == AddressMode.Implicit) return 0;
        if (mode == AddressMode.Immediate || mode == AddressMode.HighMem || mode == AddressMode.HighMemX || mode == AddressMode.HighMemY || mode == AddressMode.Relative) return 1;
        return 2;
    }

    // Round up using a power-of-two alignment mask; nonpositive alignment leaves the value unchanged.
    static int RoundUpInt(int value, int align)
    {
        if (align <= 0) return value;
        int mask = align - 1;
        return (value + mask) & ~mask;
    }

    // Report estimated bank headroom and byte-counted functions, explicitly labeled as pre-assembly estimates.
    static string BuildBankSimReportText(BankSimulationResult sim)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB bank allocation simulator");
        sb.AppendLine("# estimated pre-assembly layout");
        sb.AppendLine("rom_size=" + sim.RomSizeBytes + " used=" + sim.UsedBytes);
        sb.AppendLine("# bank, used, free");
        const int bankSize = 0x4000;
        for (int b = 0; b < sim.BankMaxPc.Length; b++)
        {
            int start = b * bankSize;
            int end = (b + 1) * bankSize;
            int maxPc = sim.BankMaxPc[b];
            int used = Math.Max(0, Math.Min(maxPc, end) - start);
            int free = Math.Max(0, bankSize - used);
            sb.AppendFormat("{0,2}, {1,5}, {2,5}\n", b, used, free);
        }

        sb.AppendLine();
        sb.AppendLine("# estimated function sizes");
        sb.AppendLine("# name, bank, start, end, size");
        foreach (var f in sim.FunctionSizes.OrderByDescending(x => x.SizeBytes).ThenBy(x => x.Name, StringComparer.Ordinal))
        {
            sb.AppendFormat("{0}, {1}, 0x{2:X5}, 0x{3:X5}, {4}\n", f.Name, f.Bank, f.StartFileOffset, f.EndFileOffset, f.SizeBytes);
        }
        return sb.ToString();
    }

    // List known cross-bank call edges that are not already marked as explicit farcalls.
    static string BuildFarcallSuggestionText(CodegenAnalysisReport report)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB farcall suggestion report");
        var rows = (report.Calls ?? new List<CallEdgeInfo>())
            .Where(c => c.CallerBank >= 0 && c.CalleeBank >= 0 && c.CallerBank != c.CalleeBank && !c.ViaFarcall)
            .OrderByDescending(c => c.Count)
            .ThenBy(c => c.Caller, StringComparer.Ordinal)
            .ThenBy(c => c.Callee, StringComparer.Ordinal)
            .ToList();

        if (rows.Count == 0)
        {
            sb.AppendLine("no farcall candidates");
            return sb.ToString();
        }

        sb.AppendLine("# caller -> callee, count, caller_bank, callee_bank, current_kind, recommendation");
        foreach (var c in rows)
        {
            // Suggest explicit intent for thunked calls; other cross-bank candidates require a bank-safety review.
            string rec = c.ViaThunk ? "consider __farcall(bank, func) for explicit cross-bank intent" : "check bank safety";
            sb.AppendFormat("{0} -> {1}, {2}, {3}, {4}, {5}, {6}\n",
                c.Caller, c.Callee, c.Count, c.CallerBank, c.CalleeBank, c.Kind, rec);
        }
        return sb.ToString();
    }

    // List recorded cross-bank edges in descending structural call-count order, including thunk/farcall flags.
    static string BuildCrossBankCallText(CodegenAnalysisReport report)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB cross-bank call report");
        sb.AppendLine("# caller, caller_bank, callee, callee_bank, count, kind, via_thunk, via_farcall");
        var rows = (report.Calls ?? new List<CallEdgeInfo>())
            .Where(c => c.CallerBank >= 0 && c.CalleeBank >= 0 && c.CallerBank != c.CalleeBank)
            .OrderByDescending(c => c.Count)
            .ThenBy(c => c.Caller, StringComparer.Ordinal)
            .ThenBy(c => c.Callee, StringComparer.Ordinal);

        foreach (var c in rows)
        {
            sb.AppendFormat("{0}, {1}, {2}, {3}, {4}, {5}, {6}, {7}\n",
                c.Caller, c.CallerBank, c.Callee, c.CalleeBank, c.Count, c.Kind, c.ViaThunk ? 1 : 0, c.ViaFarcall ? 1 : 0);
        }
        return sb.ToString();
    }

    // Serialize cartridge layout, allocator/access records, ABI/call information and recorded NES actions for tool consumption.
    static string BuildKurosakiMetadataJson(CodegenAnalysisReport codegen, AssemblerAnalysisReport asm, string outputFilename)
    {
        var sb = new StringBuilder();
        // Share one name-to-alias map across metadata sections so repeated identifiers receive consistent aliases.
        var redactedIdentifiers = new Dictionary<string, string>(StringComparer.Ordinal);
        Func<string, string> metadataIdentifier = raw =>
        {
            string value = raw ?? "";
            // Identifier aliasing is conditional on the SUROM profile in this format; other profiles retain original names.
            if (!NesMapperProfile.IsSurom512) return value;
            string replacement;
            if (!redactedIdentifiers.TryGetValue(value, out replacement))
            {
                // Assign aliases in first-encounter order, not by a stable hash of the original name.
                replacement = "function_" + (redactedIdentifiers.Count + 1).ToString("D4");
                redactedIdentifiers[value] = replacement;
            }
            return replacement;
        };
        sb.Append("{");
        sb.Append("\"schema\":\"kitaqfc-nes-build-metadata-v2\",");
        sb.Append("\"producer\":\"KITAQFC\",");
        sb.Append("\"target\":\"nes\",");
        sb.Append("\"output\":\"").Append(JsonEscape(NesMapperProfile.IsSurom512 ? "output_0001.nes" : (outputFilename ?? ""))).Append("\",");
        sb.Append("\"mapper\":\"").Append(JsonEscape(NesMapperProfile.CliName)).Append("\",");
        sb.Append("\"board\":\"").Append(JsonEscape(NesMapperProfile.BoardCliName)).Append("\",");
        sb.Append("\"cartridge\":{");
        sb.Append("\"mapper_number\":").Append(NesMapperProfile.MapperNumber).Append(",");
        sb.Append("\"mapper\":\"").Append(JsonEscape(NesMapperProfile.CliName)).Append("\",");
        sb.Append("\"board\":\"").Append(JsonEscape(NesMapperProfile.BoardCliName)).Append("\",");
        sb.Append("\"header\":\"").Append(JsonEscape(NesMapperProfile.HeaderKind)).Append("\",");
        sb.Append("\"prg_rom_bytes\":").Append(asm.PrgRomSizeBytes >= 0 ? asm.PrgRomSizeBytes : (NesMapperProfile.IsSurom512 ? 0x80000 : Math.Max(0, asm.RomSizeBytes - 16))).Append(",");
        sb.Append("\"chr_rom_bytes\":").Append(asm.ChrRomSizeBytes >= 0 ? asm.ChrRomSizeBytes : (NesMapperProfile.IsSurom512 ? 0 : -1)).Append(",");
        sb.Append("\"chr_ram_bytes\":").Append(NesMapperProfile.ChrRamBytes).Append(",");
        sb.Append("\"prg_ram_bytes\":").Append(NesMapperProfile.PrgRamBytes).Append(",");
        sb.Append("\"battery\":").Append(NesCartridge.BatteryBacked ? "true" : "false").Append(",");
        sb.Append("\"prg_mode\":3,\"chr_mode\":0},");
        if (NesMapperProfile.IsSurom512)
        {
            sb.Append("\"bank_layout\":{");
            sb.Append("\"kind\":\"surom_outer256_fixed_top16\",");
            sb.Append("\"logical_common_bank\":0,\"logical_switchable_min\":1,\"logical_switchable_max\":30,");
            sb.Append("\"common_physical_banks\":[15,31],");
            sb.Append("\"common_replica_equal\":").Append(asm.CommonReplicaEqual ? "true" : "false").Append(",");
            sb.Append("\"common_replica_sha256\":\"").Append(JsonEscape(asm.CommonReplicaSha256 ?? "")).Append("\",");
            sb.Append("\"mappings\":[");
            bool firstMapping = true;
            // Export logical switchable banks and their outer/inner physical-bank coordinates; common replicas follow separately.
            for (int logical = 1; logical <= 30; logical++)
            {
                int physical = NesMapperProfile.LogicalToPhysicalBank(logical);
                if (!firstMapping) sb.Append(",");
                firstMapping = false;
                sb.Append("{\"logical_bank\":").Append(logical).Append(",\"physical_bank\":").Append(physical)
                    .Append(",\"outer_256k\":").Append((physical >> 4) & 1)
                    .Append(",\"inner_16k\":").Append(physical & 15)
                    .Append(",\"is_common_replica\":false}");
            }
            sb.Append(", {\"logical_bank\":0,\"physical_bank\":15,\"outer_256k\":0,\"inner_16k\":15,\"is_common_replica\":true}");
            sb.Append(", {\"logical_bank\":0,\"physical_bank\":31,\"outer_256k\":1,\"inner_16k\":15,\"is_common_replica\":true}");
            sb.Append("]},");
            sb.Append("\"reset\":{\"tail_reserve_bytes\":64,\"replication\":\"every_physical_32k_high_bank\"},");
            sb.Append("\"redaction\":{\"source_paths_redacted\":true,\"project_labels_redacted\":true,\"identifiers_redacted\":true},");
        }
        sb.Append("\"optimizations\":{");
        sb.Append("\"whole_program_zp\":").Append(EnableWholeProgramZpAllocator ? "true" : "false").Append(",");
        sb.Append("\"fastcall_v2\":").Append(EnableRegisterCallingConventionV2 ? "true" : "false").Append(",");
        sb.Append("\"static_frame\":").Append(EnableStaticFrameAllocator ? "true" : "false").Append(",");
        sb.Append("\"small_inline\":").Append(EnableSmallFunctionAutoInline ? "true" : "false").Append(",");
        sb.Append("\"loop_lowering\":").Append(EnableLoopLoweringOptimizer ? "true" : "false").Append(",");
        sb.Append("\"lto_lite\":").Append(EnableLibraryLtoLite ? "true" : "false").Append(",");
        sb.Append("\"profile_feedback\":").Append(EnableKurosakiProfileFeedback ? "true" : "false").Append(",");
        sb.Append("\"mapper_aware_placement\":").Append(EnableMapperAwareBankPlacement ? "true" : "false");
        sb.Append("},");
        sb.Append("\"ram_access_contract\":{");
        sb.Append("\"schema\":\"kitaqfc-ram-access-v1\",");
        sb.Append("\"allocator_complete\":true,");
        sb.Append("\"implicit_stack\":{\"start\":256,\"end_exclusive\":512,\"conservative\":true},");
        sb.Append("\"configured_local_window\":");
        if (HasNesLocalRamWindow)
            sb.Append("{\"start\":").Append(NesLocalRamBase).Append(",\"end_exclusive\":").Append(NesLocalRamBase + NesLocalRamLength).Append("},");
        else
            sb.Append("null,");
        sb.Append("\"compiler_temp_window\":{\"start\":").Append(NesTempRamBase)
            .Append(",\"end_exclusive\":").Append(NesTempRamBase + NesTempRamLength)
            .Append(",\"custom\":").Append(HasCustomNesTempRamWindow ? "true" : "false").Append("},");
        sb.Append("\"allocations\":[");
        // Sort allocation records before assigning sequential IDs; retain conservative flags for downstream interpretation.
        var ramAllocations = (codegen.RamAllocations ?? new List<RamAllocationInfo>())
            .OrderBy(a => a.Address)
            .ThenBy(a => a.Kind, StringComparer.Ordinal)
            .ThenBy(a => a.Name, StringComparer.Ordinal)
            .ToList();
        for (int i = 0; i < ramAllocations.Count; i++)
        {
            var allocation = ramAllocations[i];
            if (i != 0) sb.Append(",");
            sb.Append("{");
            sb.Append("\"id\":\"allocation_").Append((i + 1).ToString("D4")).Append("\",");
            sb.Append("\"kind\":\"").Append(JsonEscape(allocation.Kind)).Append("\",");
            sb.Append("\"address\":").Append(allocation.Address).Append(",");
            sb.Append("\"size\":").Append(allocation.Size).Append(",");
            sb.Append("\"end_exclusive\":").Append(allocation.Address + allocation.Size).Append(",");
            sb.Append("\"region\":\"").Append(JsonEscape(allocation.Region)).Append("\",");
            sb.Append("\"conservative\":").Append(allocation.Conservative ? "true" : "false");
            sb.Append("}");
        }
        sb.Append("],\"function_accesses\":[");
        // Export recorded access spans and dynamic-target flags rather than treating every pointer target as statically known.
        var ramAccesses = (codegen.RamAccesses ?? new List<RamAccessInfo>())
            .OrderBy(a => a.Bank)
            .ThenBy(a => a.Function, StringComparer.Ordinal)
            .ThenBy(a => a.Address)
            .ThenBy(a => a.Operation, StringComparer.Ordinal)
            .ToList();
        for (int i = 0; i < ramAccesses.Count; i++)
        {
            var access = ramAccesses[i];
            if (i != 0) sb.Append(",");
            sb.Append("{");
            sb.Append("\"id\":\"access_").Append((i + 1).ToString("D4")).Append("\",");
            sb.Append("\"function\":\"").Append(JsonEscape(metadataIdentifier(access.Function))).Append("\",");
            sb.Append("\"bank\":").Append(access.Bank).Append(",");
            sb.Append("\"operation\":\"").Append(JsonEscape(access.Operation)).Append("\",");
            sb.Append("\"address\":").Append(access.Address).Append(",");
            sb.Append("\"span\":").Append(access.Span).Append(",");
            sb.Append("\"end_exclusive\":").Append(access.Address + access.Span).Append(",");
            sb.Append("\"region\":\"").Append(JsonEscape(access.Region)).Append("\",");
            sb.Append("\"dynamic_target\":").Append(access.DynamicTarget ? "true" : "false");
            sb.Append("}");
        }
        sb.Append("]},");
        sb.Append("\"fds\":{");
        sb.Append("\"enabled\":").Append(NesMapperProfile.HasFds ? "true" : "false").Append(",");
        sb.Append("\"layout\":\"").Append(FdsPrgRamLayoutEnabled ? "fds32" : "legacy").Append("\",");
        sb.Append("\"auto_overlay\":").Append(FdsAutoOverlayEnabled ? "true" : "false").Append(",");
        sb.Append("\"overlay_farcall\":").Append(FdsOverlayFarcallEnabled ? "true" : "false");
        sb.Append("},");

        sb.Append("\"functions\":[");
        // Keep prototype/inline/fastcall flags and parameter sizes on bank-ordered function records.
        var funcs = (codegen.Functions ?? new List<FunctionAbiInfo>()).OrderBy(f => f.Bank).ThenBy(f => f.Name, StringComparer.Ordinal).ToList();
        for (int i = 0; i < funcs.Count; i++)
        {
            var f = funcs[i];
            if (i != 0) sb.Append(",");
            sb.Append("{");
            sb.Append("\"name\":\"").Append(JsonEscape(metadataIdentifier(f.Name))).Append("\",");
            sb.Append("\"bank\":").Append(f.Bank).Append(",");
            sb.Append("\"fixed_bank\":").Append(f.HasFixedBank ? "true" : "false").Append(",");
            sb.Append("\"prototype\":").Append(f.IsPrototype ? "true" : "false").Append(",");
            sb.Append("\"inline\":").Append(f.IsInline ? "true" : "false").Append(",");
            sb.Append("\"fastcall\":").Append(f.IsFastCall ? "true" : "false").Append(",");
            sb.Append("\"return_size\":").Append(f.ReturnSize).Append(",");
            sb.Append("\"param_sizes\":[");
            for (int j = 0; j < (f.ParamSizes ?? new int[0]).Length; j++) { if (j != 0) sb.Append(","); sb.Append(f.ParamSizes[j]); }
            sb.Append("]}");
        }
        sb.Append("],");

        sb.Append("\"calls\":[");
        // Serialize compiler-recorded call edges; these counts are not runtime profile samples.
        var calls = (codegen.Calls ?? new List<CallEdgeInfo>()).OrderBy(c => c.Caller, StringComparer.Ordinal).ThenBy(c => c.Callee, StringComparer.Ordinal).ToList();
        for (int i = 0; i < calls.Count; i++)
        {
            var c = calls[i];
            if (i != 0) sb.Append(",");
            sb.Append("{");
            sb.Append("\"caller\":\"").Append(JsonEscape(metadataIdentifier(c.Caller))).Append("\",");
            sb.Append("\"caller_bank\":").Append(c.CallerBank).Append(",");
            sb.Append("\"callee\":\"").Append(JsonEscape(metadataIdentifier(c.Callee))).Append("\",");
            sb.Append("\"callee_bank\":").Append(c.CalleeBank).Append(",");
            sb.Append("\"kind\":\"").Append(JsonEscape(c.Kind)).Append("\",");
            sb.Append("\"via_thunk\":").Append(c.ViaThunk ? "true" : "false").Append(",");
            sb.Append("\"via_farcall\":").Append(c.ViaFarcall ? "true" : "false").Append(",");
            sb.Append("\"count\":").Append(c.Count);
            sb.Append("}");
        }
        sb.Append("],");

        sb.Append("\"optimization_events\":{");
        sb.Append("\"zp_allocations\":").Append((codegen.ZpAllocations ?? new List<ZpAllocationInfo>()).Count).Append(",");
        sb.Append("\"static_frame_slots\":").Append((codegen.StaticFrameSlots ?? new List<StaticFrameSlotInfo>()).Count).Append(",");
        sb.Append("\"ram_allocations\":").Append((codegen.RamAllocations ?? new List<RamAllocationInfo>()).Count).Append(",");
        sb.Append("\"ram_accesses\":").Append((codegen.RamAccesses ?? new List<RamAccessInfo>()).Count).Append(",");
        sb.Append("\"inline_decisions\":").Append((codegen.InlineDecisions ?? new List<InlineDecisionInfo>()).Count).Append(",");
        sb.Append("\"loop_lowerings\":").Append((codegen.LoopLowerings ?? new List<LoopLoweringInfo>()).Count).Append(",");
        sb.Append("\"lto_removed_functions\":").Append((codegen.LtoRemovedFunctions ?? new List<LtoRemovalInfo>()).Count).Append(",");
        sb.Append("\"bank_placements\":").Append((codegen.BankPlacements ?? new List<BankPlacementInfo>()).Count);
        sb.Append("},");

        sb.Append("\"nes_actions\":[");
        // Export action contracts such as timing and mapper requirements from the compiler records, not observed emulator events.
        var actions = (codegen.NesActions ?? new List<NesActionUseInfo>()).OrderBy(a => a.Caller, StringComparer.Ordinal).ThenBy(a => a.Name, StringComparer.Ordinal).ToList();
        for (int i = 0; i < actions.Count; i++)
        {
            var a = actions[i];
            if (i != 0) sb.Append(",");
            sb.Append("{");
            sb.Append("\"name\":\"").Append(JsonEscape(a.Name)).Append("\",");
            sb.Append("\"category\":\"").Append(JsonEscape(a.Category)).Append("\",");
            sb.Append("\"operation\":\"").Append(JsonEscape(a.Operation)).Append("\",");
            sb.Append("\"kurosaki_kind\":\"").Append(JsonEscape(a.KurosakiKind)).Append("\",");
            sb.Append("\"caller\":\"").Append(JsonEscape(metadataIdentifier(a.Caller))).Append("\",");
            sb.Append("\"caller_bank\":").Append(a.CallerBank).Append(",");
            sb.Append("\"source\":\"").Append(JsonEscape(NesMapperProfile.IsSurom512 ? "redacted" : a.Source)).Append("\",");
            sb.Append("\"timing\":\"").Append(JsonEscape(a.Timing)).Append("\",");
            sb.Append("\"direct_ppu\":").Append(a.DirectPpuAccess ? "true" : "false").Append(",");
            sb.Append("\"queue_ppu\":").Append(a.QueuePpuAccess ? "true" : "false").Append(",");
            sb.Append("\"oam_shadow\":").Append(a.UsesOamShadow ? "true" : "false").Append(",");
            sb.Append("\"oam_dma\":").Append(a.PerformsOamDma ? "true" : "false").Append(",");
            sb.Append("\"requires_fds\":").Append(a.RequiresFds ? "true" : "false").Append(",");
            sb.Append("\"mapper_requirement\":\"").Append(JsonEscape(a.MapperRequirement)).Append("\",");
            sb.Append("\"note\":\"").Append(JsonEscape(a.Note)).Append("\"");
            sb.Append("}");
        }
        sb.Append("],");

        sb.Append("\"function_sizes\":[");
        // Keep final function placement distinct from ABI declarations; addresses and sizes come from assembly.
        var sizes = (asm.FunctionSizes ?? new List<FunctionSizeInfo>()).OrderBy(f => f.Bank).ThenBy(f => f.CpuAddress).ToList();
        for (int i = 0; i < sizes.Count; i++)
        {
            var f = sizes[i];
            if (i != 0) sb.Append(",");
            sb.Append("{");
            sb.Append("\"name\":\"").Append(JsonEscape(metadataIdentifier(f.Name))).Append("\",");
            sb.Append("\"bank\":").Append(f.Bank).Append(",");
            sb.Append("\"cpu_address\":").Append(f.CpuAddress).Append(",");
            sb.Append("\"size\":").Append(f.SizeBytes);
            sb.Append("}");
        }
        sb.Append("]");
        sb.Append("}");
        return sb.ToString();
    }

    // Format the supplied ABI issue list; PASS means that this list is empty, not that runtime calling behavior was tested.
    static string BuildAbiVerifyText(CodegenAnalysisReport report, List<string> issues)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB ABI verification report");
        sb.AppendLine("abi_mode=" + Abi.ToString().ToLowerInvariant());
        sb.AppendLine("functions=" + (report.Functions == null ? 0 : report.Functions.Count));
        sb.AppendLine("calls=" + (report.Calls == null ? 0 : report.Calls.Count));
        sb.AppendLine("issues=" + (issues == null ? 0 : issues.Count));
        sb.AppendLine();
        if (issues == null || issues.Count == 0)
        {
            sb.AppendLine("PASS");
            return sb.ToString();
        }
        sb.AppendLine("FAIL");
        foreach (var issue in issues) sb.AppendLine("- " + issue);
        return sb.ToString();
    }

    // Show selected RST targets separately from optimizer rewrite counts so selection and application are distinguishable.
    static string BuildRstApplyText(CodegenAnalysisReport codegen, OptimizerAnalysisReport opt)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB RST apply report");
        sb.AppendLine("rst_enabled=" + (!RstDisable ? "1" : "0"));
        sb.AppendLine("rst_use_38=" + (RstUse38 ? "1" : "0"));
        sb.AppendLine();
        sb.AppendLine("# selection");
        foreach (var r in (codegen.RstSelections ?? new List<RstSelectionInfo>()).OrderBy(x => x.Vector))
        {
            sb.AppendFormat("RST_{0:X2} -> {1} (calls={2}, net_bytes={3})\n", r.Vector, r.TargetLabel, r.Calls, r.NetBytes);
        }

        sb.AppendLine();
        sb.AppendLine("# optimizer rewrites");
        sb.AppendLine("total=" + opt.TotalRstRewrites);
        foreach (var kv in opt.RstRewriteCountsByVector.OrderBy(x => x.Key))
        {
            sb.AppendFormat("RST_{0:X2}: {1}\n", kv.Key, kv.Value);
        }
        return sb.ToString();
    }

    // Preserve each optimizer pass report and its captured textual diff in pass order.
    static string BuildOptimizerDiffText(OptimizerAnalysisReport report)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB optimizer pass diff report");
        foreach (var p in report.Passes ?? new List<OptimizerPassReport>())
        {
            sb.AppendLine();
            sb.AppendLine("## " + p.Name);
            sb.AppendLine("before=" + p.BeforeLines + " after=" + p.AfterLines + " changed=" + p.ChangedLines + " added=" + p.AddedLines + " removed=" + p.RemovedLines);
            sb.AppendLine(p.DiffText ?? "");
        }
        return sb.ToString();
    }

    // Report final assembler extents using both banked CPU addresses and ROM file offsets, sorted by decreasing size.
    static string BuildFunctionSizeReportText(AssemblerAnalysisReport report)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB function size report");
        sb.AppendLine("# name, bank, cpu_addr, start_file, end_file, size");
        foreach (var f in (report.FunctionSizes ?? new List<FunctionSizeInfo>()).OrderByDescending(x => x.SizeBytes).ThenBy(x => x.Name, StringComparer.Ordinal))
        {
            sb.AppendFormat("{0}, {1}, 0x{2:X4}, 0x{3:X5}, 0x{4:X5}, {5}\n",
                f.Name, f.Bank, f.CpuAddress, f.StartFileOffset, f.EndFileOffset, f.SizeBytes);
        }
        return sb.ToString();
    }

    // Snapshot counters/list length and suppress console diagnostics while speculative work runs.
    public static DiagnosticSnapshot BeginSuppressedDiagnostics()
    {
        DiagnosticSnapshot snap = new DiagnosticSnapshot
        {
            ErrorCount = ErrorCount,
            WarningCount = WarningCount,
            DiagnosticCount = Diagnostics.Count,
        };
        SuppressConsoleDiagnostics = true;
        return snap;
    }

    // Restore counters and discard later records, then enable console diagnostics.
    // The prior suppression state is not stored, so this helper is not a nested suppression stack.
    public static void RestoreDiagnostics(DiagnosticSnapshot snap)
    {
        ErrorCount = snap.ErrorCount;
        WarningCount = snap.WarningCount;
        while (Diagnostics.Count > snap.DiagnosticCount) Diagnostics.RemoveAt(Diagnostics.Count - 1);
        SuppressConsoleDiagnostics = false;
    }

    // Clear both function and readonly-data placement feedback before a new layout starts.
    static void ClearBankOverrides()
    {
        FunctionBankOverrides.Clear();
        ReadonlyDataBankOverrides.Clear();
    }

    // Look up a case-sensitive relocation override; a false result does not establish bank zero.
    public static bool TryGetFunctionBankOverride(string name, out int bank)
    {
        if (string.IsNullOrEmpty(name))
        {
            bank = 0;
            return false;
        }
        return FunctionBankOverrides.TryGetValue(name, out bank);
    }

    // Look up data-placement feedback without treating a missing name as a bank-zero assignment.
    public static bool TryGetReadonlyDataBankOverride(string name, out int bank)
    {
        if (string.IsNullOrEmpty(name))
        {
            bank = 0;
            return false;
        }
        return ReadonlyDataBankOverrides.TryGetValue(name, out bank);
    }

    // Merge new assignments while retaining earlier overrides needed by already stabilized symbols.
    static void ApplyBankOverrides(Dictionary<string, int> functionOverrides, Dictionary<string, int> readonlyDataOverrides)
    {
        foreach (var kv in functionOverrides ?? new Dictionary<string, int>())
            FunctionBankOverrides[kv.Key] = kv.Value;
        foreach (var kv in readonlyDataOverrides ?? new Dictionary<string, int>())
            ReadonlyDataBankOverrides[kv.Key] = kv.Value;
    }

    // Retain both requested and actual banks so fixed-placement failures can identify the mismatch.
    sealed class FunctionBankConflictInfo
    {
        public string Name;
        public int RequestedBank;
        public int ActualBank;
    }

    // Separate relocations that can be fed into another pass from conflicts with explicit fixed placement.
    sealed class FunctionBankRelayoutResult
    {
        public readonly Dictionary<string, int> Relocations = new Dictionary<string, int>(StringComparer.Ordinal);
        public readonly Dictionary<string, int> ReadonlyDataRelocations = new Dictionary<string, int>(StringComparer.Ordinal);
        public readonly List<FunctionBankConflictInfo> FixedBankConflicts = new List<FunctionBankConflictInfo>();
        public readonly List<FunctionBankConflictInfo> FixedReadonlyDataConflicts = new List<FunctionBankConflictInfo>();
    }

    // Compare code-generation bank assumptions with assembler placement, matching records by symbol name.
    static FunctionBankRelayoutResult DetectFunctionBankRelocations(CodegenAnalysisReport codegen, AssemblerAnalysisReport asm)
    {
        // For duplicate function names, use the earliest file-offset record as the placement source.
        var actualByName = (asm?.FunctionSizes ?? new List<FunctionSizeInfo>())
            .Where(f => f != null && !string.IsNullOrEmpty(f.Name))
            .GroupBy(f => f.Name, StringComparer.Ordinal)
            .ToDictionary(
                g => g.Key,
                g => g.OrderBy(f => f.StartFileOffset).First().Bank,
                StringComparer.Ordinal);

        var result = new FunctionBankRelayoutResult();
        foreach (var f in codegen?.Functions ?? new List<FunctionAbiInfo>())
        {
            if (f == null || string.IsNullOrEmpty(f.Name)) continue;
            // Only emitted ordinary functions participate in relocation feedback; declarations and inline definitions have no independent body placement.
            if (f.IsPrototype || f.IsInline) continue;
            if (!actualByName.TryGetValue(f.Name, out int actualBank)) continue;
            if (actualBank == f.Bank) continue;

            if (f.HasFixedBank)
            {
                result.FixedBankConflicts.Add(new FunctionBankConflictInfo
                {
                    Name = f.Name,
                    RequestedBank = f.Bank,
                    ActualBank = actualBank
                });
                continue;
            }

            // Request a new bank assumption only after excluding explicit fixed-bank conflicts.
            result.Relocations[f.Name] = actualBank;
        }

        // Choose the earliest emitted extent per data name when comparing assembler and compiler bank assignments.
        var actualReadonlyByName = (asm?.ReadonlyData ?? new List<ReadonlyDataInfo>())
            .Where(d => d != null && !string.IsNullOrEmpty(d.Name))
            .GroupBy(d => d.Name, StringComparer.Ordinal)
            .ToDictionary(
                g => g.Key,
                g => g.OrderBy(d => d.StartFileOffset).First().Bank,
                StringComparer.Ordinal);

        foreach (var d in codegen?.ReadonlyData ?? new List<ReadonlyDataInfo>())
        {
            if (d == null || string.IsNullOrEmpty(d.Name)) continue;
            if (!actualReadonlyByName.TryGetValue(d.Name, out int actualBank)) continue;
            if (actualBank == d.Bank) continue;

            if (d.HasFixedBank)
            {
                result.FixedReadonlyDataConflicts.Add(new FunctionBankConflictInfo
                {
                    Name = d.Name,
                    RequestedBank = d.Bank,
                    ActualBank = actualBank
                });
                continue;
            }

            // Only movable readonly data receives feedback; explicit fixed-bank conflicts were collected above.
            result.ReadonlyDataRelocations[d.Name] = actualBank;
        }
        return result;
    }

    // Show enabled options and recorded allocation, inlining, lowering, removal and placement decisions.
    // These records describe compiler actions, not measured game performance.
    static string BuildNesOptimizationReportText(CodegenAnalysisReport codegen)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQFC NES optimization report");
        sb.AppendLine("# enabled: zp=" + EnableWholeProgramZpAllocator + ", fastcall=" + EnableRegisterCallingConventionV2 + ", static_frame=" + EnableStaticFrameAllocator + ", inline=" + EnableSmallFunctionAutoInline + ", loop=" + EnableLoopLoweringOptimizer + ", lto_lite=" + EnableLibraryLtoLite + ", kurosaki_profile=" + EnableKurosakiProfileFeedback + ", mapper_placement=" + EnableMapperAwareBankPlacement);
        if (!string.IsNullOrEmpty(KurosakiProfilePath)) sb.AppendLine("# kurosaki_profile_path=" + KurosakiProfilePath);

        sb.AppendLine();
        sb.AppendLine("## fastcall functions");
        foreach (var f in (codegen.Functions ?? new List<FunctionAbiInfo>()).Where(f => f.IsFastCall).OrderBy(f => f.Bank).ThenBy(f => f.Name, StringComparer.Ordinal))
            sb.AppendFormat("{0}, bank={1}, params={2}, ret={3}\n", f.Name, f.Bank, BuildReportUtil.JoinInts(f.ParamSizes), f.ReturnSize);

        sb.AppendLine();
        sb.AppendLine("## zero-page allocations");
        foreach (var z in codegen.ZpAllocations ?? new List<ZpAllocationInfo>())
            sb.AppendFormat("${0:X2}, size={1}, {2}, {3}, {4}\n", z.Address & 0xFF, z.Size, z.Kind, z.Name, z.Reason);

        sb.AppendLine();
        sb.AppendLine("## static frame slots");
        foreach (var sf in codegen.StaticFrameSlots ?? new List<StaticFrameSlotInfo>())
            sb.AppendFormat("{0}::{1}, ${2:X4}, size={3}, {4}\n", sf.Function, sf.Name, sf.Address, sf.Size, sf.Region);

        sb.AppendLine();
        sb.AppendLine("## RAM access contract allocations");
        foreach (var allocation in codegen.RamAllocations ?? new List<RamAllocationInfo>())
            sb.AppendFormat("${0:X4}-${1:X4}, size={2}, {3}, {4}, conservative={5}\n",
                allocation.Address,
                allocation.Address + Math.Max(0, allocation.Size) - 1,
                allocation.Size,
                allocation.Kind,
                allocation.Region,
                allocation.Conservative);

        sb.AppendLine();
        sb.AppendLine("## inline decisions");
        foreach (var i in codegen.InlineDecisions ?? new List<InlineDecisionInfo>())
            sb.AppendFormat("{0}->{1}, {2}, applied={3}, {4}\n", i.Caller, i.Callee, i.Kind, i.Applied, i.Reason);

        sb.AppendLine();
        sb.AppendLine("## loop lowerings");
        foreach (var l in codegen.LoopLowerings ?? new List<LoopLoweringInfo>())
            sb.AppendFormat("{0}, var={1}, limit={2}, {3}, {4}\n", l.Function, l.Variable, l.Limit, l.Kind, l.Source);

        sb.AppendLine();
        sb.AppendLine("## LTO-lite removed functions");
        foreach (var r in codegen.LtoRemovedFunctions ?? new List<LtoRemovalInfo>())
            sb.AppendFormat("{0}, {1}\n", r.Name, r.Reason);

        sb.AppendLine();
        sb.AppendLine("## mapper-aware bank placements");
        foreach (var b in codegen.BankPlacements ?? new List<BankPlacementInfo>())
            sb.AppendFormat("{0}, bank={1}, hotness={2}, {3}\n", b.Name, b.Bank, b.Hotness, b.Reason);
        return sb.ToString();
    }

    // Rank all known function names by incoming structural calls times maximum reported size; this is not a runtime profile.
    static string BuildHotspotReportText(CodegenAnalysisReport codegen, AssemblerAnalysisReport asm)
    {
        var sizeByFunc = (asm.FunctionSizes ?? new List<FunctionSizeInfo>())
            .GroupBy(x => x.Name, StringComparer.Ordinal)
            .ToDictionary(g => g.Key, g => g.Max(x => x.SizeBytes), StringComparer.Ordinal);

        var incomingCalls = new Dictionary<string, int>(StringComparer.Ordinal);
        foreach (var c in codegen.Calls ?? new List<CallEdgeInfo>())
        {
            if (string.IsNullOrEmpty(c.Callee) || c.Callee.StartsWith("<", StringComparison.Ordinal)) continue;
            int cur = 0;
            incomingCalls.TryGetValue(c.Callee, out cur);
            incomingCalls[c.Callee] = cur + c.Count;
        }

        var allFuncs = new HashSet<string>(sizeByFunc.Keys, StringComparer.Ordinal);
        foreach (var k in incomingCalls.Keys) allFuncs.Add(k);

        var rows = allFuncs
            .Select(name =>
            {
                int size = 0; sizeByFunc.TryGetValue(name, out size);
                int calls = 0; incomingCalls.TryGetValue(name, out calls);
                long score = (long)size * (long)calls;
                return new { Name = name, Size = size, Calls = calls, Score = score };
            })
            .OrderByDescending(x => x.Score)
            .ThenByDescending(x => x.Calls)
            .ThenByDescending(x => x.Size)
            .ThenBy(x => x.Name, StringComparer.Ordinal)
            .ToList();

        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB hotspot estimate");
        sb.AppendLine("# score = incoming_call_count * function_size");
        sb.AppendLine("# name, calls, size, score");
        foreach (var r in rows)
        {
            sb.AppendFormat("{0}, {1}, {2}, {3}\n", r.Name, r.Calls, r.Size, r.Score);
        }
        return sb.ToString();
    }

    // Store recognized writes and compiler guard counters alongside header-derived consistency issues.
    sealed class CgbConsistencyResult
    {
        public bool Pass;
        public byte HeaderFlag;
        public string HeaderMode;
        public int TotalCgbWrites;
        public int GuardedWrites;
        public int UnguardedWrites;
        public int RuntimeChecks;
        public readonly List<string> Registers = new List<string>();
        public readonly List<string> Issues = new List<string>();
    }

    // Keep the required namespace, referenced subset and missing names separate for the symbol report.
    sealed class CgbSymbolVerifyResult
    {
        public bool Pass;
        public readonly List<string> Required = new List<string>();
        public readonly List<string> Referenced = new List<string>();
        public readonly List<string> Missing = new List<string>();
        public int MapSymbolCount;
    }

    // Recognize selected CGB registers by their low FF00-page address byte.
    static readonly Dictionary<int, string> CgbIoOffsetToName = new Dictionary<int, string>
    {
        { 0x4D, "KEY1" },
        { 0x4F, "VBK" },
        { 0x51, "HDMA1" },
        { 0x52, "HDMA2" },
        { 0x53, "HDMA3" },
        { 0x54, "HDMA4" },
        { 0x55, "HDMA5" },
        { 0x68, "BCPS" },
        { 0x69, "BCPD" },
        { 0x6A, "OCPS" },
        { 0x6B, "OCPD" },
        { 0x70, "SVBK" },
    };

    // The verification contract requires this entire register-symbol list, even when only some names are referenced.
    static readonly string[] CgbRequiredSymbols = new[]
    {
        "KEY1", "VBK", "SVBK", "BCPS", "BCPD", "OCPS", "OCPD", "HDMA1", "HDMA2", "HDMA3", "HDMA4", "HDMA5"
    };

    // Combine recognized direct register writes with aggregate compiler guard/check counters.
    // This is a structural consistency check, not control-flow proof that every hardware access is guarded.
    static CgbConsistencyResult AnalyzeCgbConsistency(IReadOnlyList<Expr> assembly, CodegenAnalysisReport codegen, string outputFilename, RomHeaderOptions effectiveRomHeader)
    {
        var result = new CgbConsistencyResult();
        result.HeaderFlag = ReadRomHeaderByteSafe(outputFilename, 0x143, effectiveRomHeader != null && effectiveRomHeader.CgbFlag.HasValue ? effectiveRomHeader.CgbFlag.Value : (byte)0x00);
        result.HeaderMode = DescribeCgbHeaderFlag(result.HeaderFlag);

        var regs = new HashSet<string>(StringComparer.Ordinal);
        int writes = 0;
        string currentFunction = null;

        foreach (var e in assembly ?? new List<Expr>())
        {
            if (e.Match(Tag.Function, out string fn))
            {
                currentFunction = fn;
                continue;
            }

            string mnemonic;
            AsmOperand operand;
            if (!e.Match(Tag.Asm, out mnemonic, out operand)) continue;

            // __kq_is_cgb uses VBK toggle/readback internally for runtime detection.
            // Do not count those helper writes as payload CGB register writes.
            if (string.Equals(currentFunction, "__kq_is_cgb", StringComparison.Ordinal)) continue;

            if (TryGetCgbWriteRegisterName(mnemonic, operand, out string regName))
            {
                writes++;
                regs.Add(regName);
            }
        }

        result.TotalCgbWrites = writes;
        result.GuardedWrites = Math.Max(0, codegen == null ? 0 : codegen.CgbGuardedWriteCount);
        // Infer the unguarded total by subtraction; this does not pair individual writes with individual guards.
        result.UnguardedWrites = Math.Max(0, result.TotalCgbWrites - result.GuardedWrites);
        result.RuntimeChecks = Math.Max(0, codegen == null ? 0 : codegen.CgbRuntimeCheckCount);
        foreach (var r in regs.OrderBy(x => x, StringComparer.Ordinal)) result.Registers.Add(r);

        if (result.HeaderFlag == 0x00 && result.UnguardedWrites > 0)
        {
            result.Issues.Add("header is DMG-only (0x00) but unguarded CGB register writes were detected");
        }
        if (result.HeaderFlag == 0x80 && result.UnguardedWrites > 0)
        {
            result.Issues.Add("header is CGB-compatible (0x80) but unguarded CGB register writes were detected");
        }
        // A dual-mode build reporting guards also needs an emitted runtime hardware-check count.
        if (result.HeaderFlag == 0x80 && result.GuardedWrites > 0 && result.RuntimeChecks == 0)
        {
            result.Issues.Add("CGB-compatible build uses guarded writes but no runtime CGB check calls were emitted");
        }

        result.Pass = result.Issues.Count == 0;
        return result;
    }

    // Check required symbol names against the map and list recognized symbolic references in assembly IR.
    static CgbSymbolVerifyResult AnalyzeCgbSymbolVerification(IReadOnlyList<Expr> assembly, string mapPath)
    {
        var result = new CgbSymbolVerifyResult();
        var mapSymbols = ParseMapSymbolNames(mapPath);
        result.MapSymbolCount = mapSymbols.Count;

        foreach (var s in CgbRequiredSymbols)
        {
            result.Required.Add(s);
        }

        var referenced = new HashSet<string>(StringComparer.Ordinal);
        foreach (var e in assembly ?? new List<Expr>())
        {
            string mnemonic;
            AsmOperand operand;
            if (!e.Match(Tag.Asm, out mnemonic, out operand)) continue;
            if (operand == null || !operand.Base.HasValue) continue;
            string baseName = operand.Base.Value;
            if (CgbRequiredSymbols.Contains(baseName))
            {
                referenced.Add(baseName);
            }
        }

        foreach (var r in referenced.OrderBy(x => x, StringComparer.Ordinal))
        {
            result.Referenced.Add(r);
        }

        var missing = new HashSet<string>(StringComparer.Ordinal);
        foreach (var req in CgbRequiredSymbols)
        {
            if (!mapSymbols.Contains(req)) missing.Add(req);
        }
        foreach (var rf in referenced)
        {
            if (!mapSymbols.Contains(rf)) missing.Add(rf);
        }

        foreach (var m in missing.OrderBy(x => x, StringComparer.Ordinal))
        {
            result.Missing.Add(m);
        }

        result.Pass = result.Missing.Count == 0;
        return result;
    }

    // Read the final whitespace field of noncomment rows with at least five fields; missing maps yield an empty set.
    static HashSet<string> ParseMapSymbolNames(string mapPath)
    {
        var set = new HashSet<string>(StringComparer.Ordinal);
        if (string.IsNullOrWhiteSpace(mapPath) || !File.Exists(mapPath)) return set;

        foreach (var line in IoUtil.ReadAllLinesUtf8(mapPath))
        {
            if (string.IsNullOrWhiteSpace(line)) continue;
            string t = line.Trim();
            if (t.StartsWith(";") || t.StartsWith("#")) continue;

            var parts = t.Split(new[] { ' ', '\t' }, StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length < 5) continue;
            string name = parts[parts.Length - 1];
            if (!string.IsNullOrEmpty(name)) set.Add(name);
        }
        return set;
    }

    // Render the collected counters and issues without adding further analysis.
    static string BuildCgbConsistencyText(CgbConsistencyResult result)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB CGB consistency report");
        sb.AppendLine("header_flag=0x" + result.HeaderFlag.ToString("X2"));
        sb.AppendLine("header_mode=" + result.HeaderMode);
        sb.AppendLine("total_cgb_writes=" + result.TotalCgbWrites);
        sb.AppendLine("guarded_writes=" + result.GuardedWrites);
        sb.AppendLine("unguarded_writes=" + result.UnguardedWrites);
        sb.AppendLine("runtime_checks=" + result.RuntimeChecks);
        sb.AppendLine("result=" + (result.Pass ? "PASS" : "FAIL"));
        sb.AppendLine();
        sb.AppendLine("# registers");
        foreach (var r in result.Registers) sb.AppendLine("- " + r);
        sb.AppendLine();
        sb.AppendLine("# issues");
        if (result.Issues.Count == 0) sb.AppendLine("none");
        else foreach (var i in result.Issues) sb.AppendLine("- " + i);
        return sb.ToString();
    }

    // Display required, referenced and missing names so an empty reference set is distinguishable from a complete map.
    static string BuildCgbSymbolVerifyText(CgbSymbolVerifyResult result)
    {
        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB CGB symbol verification report");
        sb.AppendLine("map_symbol_count=" + result.MapSymbolCount);
        sb.AppendLine("required_symbol_count=" + result.Required.Count);
        sb.AppendLine("referenced_symbol_count=" + result.Referenced.Count);
        sb.AppendLine("missing_symbol_count=" + result.Missing.Count);
        sb.AppendLine("result=" + (result.Pass ? "PASS" : "FAIL"));
        sb.AppendLine();
        sb.AppendLine("# required");
        foreach (var x in result.Required) sb.AppendLine("- " + x);
        sb.AppendLine();
        sb.AppendLine("# referenced");
        if (result.Referenced.Count == 0) sb.AppendLine("none");
        else foreach (var x in result.Referenced) sb.AppendLine("- " + x);
        sb.AppendLine();
        sb.AppendLine("# missing");
        if (result.Missing.Count == 0) sb.AppendLine("none");
        else foreach (var x in result.Missing) sb.AppendLine("- " + x);
        return sb.ToString();
    }

    // Recognize only direct LDH_MEM_A/LD_MEM_A stores to known symbolic or literal CGB registers.
    // Indirect stores and other instruction shapes are outside this recognizer.
    static bool TryGetCgbWriteRegisterName(string mnemonic, AsmOperand operand, out string regName)
    {
        regName = null;
        if (operand == null || string.IsNullOrEmpty(mnemonic)) return false;
        if (mnemonic != "LDH_MEM_A" && mnemonic != "LD_MEM_A") return false;

        if (operand.Base.HasValue)
        {
            string b = operand.Base.Value;
            if (CgbRequiredSymbols.Contains(b))
            {
                regName = b;
                return true;
            }
        }

        if (!operand.Base.HasValue)
        {
            if (mnemonic == "LDH_MEM_A")
            {
                int off = operand.Offset & 0xFF;
                if (CgbIoOffsetToName.TryGetValue(off, out string n))
                {
                    regName = n;
                    return true;
                }
            }
            else if (mnemonic == "LD_MEM_A")
            {
                int addr = operand.Offset & 0xFFFF;
                if ((addr & 0xFF00) == 0xFF00)
                {
                    int off = addr & 0xFF;
                    if (CgbIoOffsetToName.TryGetValue(off, out string n))
                    {
                        regName = n;
                        return true;
                    }
                }
            }
        }

        return false;
    }

    // Read the actual ROM byte when available; missing, unreadable or short files retain the supplied fallback.
    static byte ReadRomHeaderByteSafe(string romPath, int offset, byte fallback)
    {
        try
        {
            if (File.Exists(romPath))
            {
                var bytes = File.ReadAllBytes(romPath);
                if (offset >= 0 && offset < bytes.Length) return bytes[offset];
            }
        }
        catch { }
        return fallback;
    }

    // Name the three recognized exact header values and retain unknown for all other bytes.
    static string DescribeCgbHeaderFlag(byte flag)
    {
        if (flag == 0xC0) return "cgb_only";
        if (flag == 0x80) return "cgb_compatible";
        if (flag == 0x00) return "dmg_only";
        return "unknown";
    }

    // Identify the listed report switches for child-build filtering; this is not a complete CLI option parser.
    static bool IsAnalysisFlag(string arg)
    {
        if (string.IsNullOrEmpty(arg)) return false;
        return
            arg == "--bank-sim" || arg.StartsWith("--bank-sim=") ||
            arg == "--farcall-suggest" || arg.StartsWith("--farcall-suggest=") ||
            arg == "--cross-bank-report" || arg.StartsWith("--cross-bank-report=") ||
            arg == "--abi-verify" || arg.StartsWith("--abi-verify=") ||
            arg == "--abi-diff-report" || arg.StartsWith("--abi-diff-report=") ||
            arg == "--rst-report" || arg.StartsWith("--rst-report=") ||
            arg == "--opt-diff" || arg.StartsWith("--opt-diff=") ||
            arg == "--func-size-report" || arg.StartsWith("--func-size-report=") ||
            arg == "--hotspot-report" || arg.StartsWith("--hotspot-report=") ||
            arg == "--repro-check" || arg.StartsWith("--repro-check=") ||
            arg == "--cgb-consistency" || arg.StartsWith("--cgb-consistency=") ||
            arg == "--verify-cgb-symbols" || arg.StartsWith("--verify-cgb-symbols=");
    }

    // Derive child compilation arguments by removing selected driver/report options, then append no-cache/output/ABI overrides.
    static string[] BuildChildCompileArgs(string childOutput, string abiOverride)
    {
        var list = new List<string>();
        var args = _originalArgs ?? new string[0];
        for (int i = 0; i < args.Length; i++)
        {
            string a = args[i];

            // Remove the original output flag and its following value so the child owns a separate ROM path.
            if (a == "-o")
            {
                i++;
                continue;
            }
            if (a.StartsWith("--abi=")) continue;
            if (a == "--cache" || a == "--no-cache") continue;
            if (a == "--watch") continue;
            if (a == "--minimize" || a.StartsWith("--minimize")) continue;
            if (a == "--disasm-changed" || a.StartsWith("--disasm-changed=") || a.StartsWith("--disasm-changed-out=")) continue;
            if (a == "--repro-pack" || a.StartsWith("--repro-pack=")) continue;
            if (IsAnalysisFlag(a)) continue;
            list.Add(a);
        }

        list.Add("--no-cache");
        list.Add("--no-disasm");
        if (!string.IsNullOrWhiteSpace(abiOverride))
            list.Add("--abi=" + abiOverride);
        list.Add("-o");
        list.Add(childOutput);
        return list.ToArray();
    }

    // Read name/size from six-column function-size rows; later duplicate names replace earlier entries.
    static Dictionary<string, int> ParseFunctionSizesFile(string path)
    {
        var map = new Dictionary<string, int>(StringComparer.Ordinal);
        if (!File.Exists(path)) return map;
        foreach (var line in IoUtil.ReadAllLinesUtf8(path))
        {
            if (string.IsNullOrWhiteSpace(line)) continue;
            if (line.StartsWith("#")) continue;
            var parts = line.Split(',');
            if (parts.Length < 6) continue;
            string name = parts[0].Trim();
            if (name.Length == 0) continue;
            int size;
            if (int.TryParse(parts[5].Trim(), out size))
                map[name] = size;
        }
        return map;
    }

    // Compile legacy and stack variants, compare their reported function sizes, and report either child failure.
    static void TryGenerateAbiDiffReport(string reportPath)
    {
        string exe = Process.GetCurrentProcess().MainModule.FileName;
        string tempDir = Path.Combine(Path.GetDirectoryName(reportPath) ?? ".", "kitaqfc_abidiff_" + DateTime.UtcNow.ToString("yyyyMMdd_HHmmssfff"));
        Directory.CreateDirectory(tempDir);
        string legacyOut = Path.Combine(tempDir, "legacy.nes");
        string stackOut = Path.Combine(tempDir, "stack.nes");

        int codeLegacy = RunChildCompiler(exe, BuildChildCompileArgs(legacyOut, "legacy"), Environment.CurrentDirectory);
        int codeStack = RunChildCompiler(exe, BuildChildCompileArgs(stackOut, "stack"), Environment.CurrentDirectory);

        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB ABI diff report");
        sb.AppendLine("legacy_exit=" + codeLegacy);
        sb.AppendLine("stack_exit=" + codeStack);

        if (codeLegacy == 0 && File.Exists(legacyOut))
        {
            sb.AppendLine("legacy_size=" + new FileInfo(legacyOut).Length);
            sb.AppendLine("legacy_sha256=" + BuildReportUtil.ComputeSha256HexOfFile(legacyOut));
        }
        if (codeStack == 0 && File.Exists(stackOut))
        {
            sb.AppendLine("stack_size=" + new FileInfo(stackOut).Length);
            sb.AppendLine("stack_sha256=" + BuildReportUtil.ComputeSha256HexOfFile(stackOut));
        }

        string legacyFuncPath = Path.ChangeExtension(legacyOut, ".funcsizes.txt");
        string stackFuncPath = Path.ChangeExtension(stackOut, ".funcsizes.txt");
        var legacySizes = ParseFunctionSizesFile(legacyFuncPath);
        var stackSizes = ParseFunctionSizesFile(stackFuncPath);
        var all = new HashSet<string>(legacySizes.Keys, StringComparer.Ordinal);
        foreach (var k in stackSizes.Keys) all.Add(k);

        // Use zero for a name missing from either side; this makes additions and removals visible as size deltas.
        var deltas = new List<(string Name, int Legacy, int Stack, int Delta)>();
        foreach (var name in all)
        {
            int l = 0, s = 0;
            legacySizes.TryGetValue(name, out l);
            stackSizes.TryGetValue(name, out s);
            deltas.Add((name, l, s, s - l));
        }

        sb.AppendLine();
        sb.AppendLine("# function size delta (stack - legacy)");
        sb.AppendLine("# name, legacy, stack, delta");
        foreach (var d in deltas.OrderByDescending(x => Math.Abs(x.Delta)).ThenBy(x => x.Name, StringComparer.Ordinal))
        {
            sb.AppendFormat("{0}, {1}, {2}, {3}\n", d.Name, d.Legacy, d.Stack, d.Delta);
        }

        IoUtil.WriteAllTextUtf8Robust(reportPath, sb.ToString(), allowAlternatePath: true);
        Console.WriteLine("[report] abi diff: " + reportPath);

        if (codeLegacy != 0 || codeStack != 0)
        {
            Error("ABI diff report generation failed (legacy={0}, stack={1}). See: {2}", codeLegacy, codeStack, reportPath);
        }

        try { Directory.Delete(tempDir, true); } catch { }
    }

    // Rebuild once without cache and require both nonempty ROM and map hashes to match the original output.
    static void TryRunReproCheck(string outputFilename, string reportPath)
    {
        string exe = Process.GetCurrentProcess().MainModule.FileName;
        string abiName = (Abi == AbiMode.Stack) ? "stack" : "legacy";
        string tempDir = Path.Combine(Path.GetDirectoryName(reportPath) ?? ".", "kitaqfc_repro_" + DateTime.UtcNow.ToString("yyyyMMdd_HHmmssfff"));
        Directory.CreateDirectory(tempDir);
        string reproOut = Path.Combine(tempDir, "repro.nes");

        int code = RunChildCompiler(exe, BuildChildCompileArgs(reproOut, abiName), Environment.CurrentDirectory);

        string currentGbHash = BuildReportUtil.ComputeSha256HexOfFile(outputFilename);
        string reproGbHash = BuildReportUtil.ComputeSha256HexOfFile(reproOut);
        string currentMapHash = BuildReportUtil.ComputeSha256HexOfFile(Path.ChangeExtension(outputFilename, ".map"));
        string reproMapHash = BuildReportUtil.ComputeSha256HexOfFile(Path.ChangeExtension(reproOut, ".map"));

        bool sameGb = !string.IsNullOrEmpty(currentGbHash) && currentGbHash == reproGbHash;
        bool sameMap = !string.IsNullOrEmpty(currentMapHash) && currentMapHash == reproMapHash;
        // Matching file hashes alone are insufficient if the child compiler reported failure.
        bool ok = (code == 0) && sameGb && sameMap;

        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB reproducibility check");
        sb.AppendLine("exit=" + code);
        sb.AppendLine("same_gb=" + (sameGb ? "1" : "0"));
        sb.AppendLine("same_map=" + (sameMap ? "1" : "0"));
        sb.AppendLine("current_gb_sha256=" + currentGbHash);
        sb.AppendLine("repro_gb_sha256=" + reproGbHash);
        sb.AppendLine("current_map_sha256=" + currentMapHash);
        sb.AppendLine("repro_map_sha256=" + reproMapHash);
        sb.AppendLine("result=" + (ok ? "PASS" : "FAIL"));

        IoUtil.WriteAllTextUtf8Robust(reportPath, sb.ToString(), allowAlternatePath: true);
        Console.WriteLine("[report] repro check: " + reportPath);

        if (!ok)
        {
            Error("Reproducibility check failed. See: {0}", reportPath);
        }

        try { Directory.Delete(tempDir, true); } catch { }
    }

    // Handle the raw attribute-byte visualizer, validating dimensions before reading input and selecting an output destination.
    static bool TryRunAttrVizCommand(string[] argsArray)
    {
        if (argsArray == null || argsArray.Length == 0) return false;
        if (!string.Equals(argsArray[0], "attrviz", StringComparison.OrdinalIgnoreCase)) return false;

        if (argsArray.Length < 2)
        {
            Console.Error.WriteLine("error KQ0000: attrviz requires an input file");
            Console.Error.WriteLine("usage: kitaqfc attrviz <attr.bin> [--width=N] [--height=N] [--out=<file>|-]");
            Environment.Exit(1);
            return true;
        }

        string inputPath = argsArray[1];
        int width = 32;
        int height = 0;
        string outPath = "";

        for (int i = 2; i < argsArray.Length; i++)
        {
            string a = argsArray[i] ?? "";
            if (a.StartsWith("--width=", StringComparison.Ordinal))
            {
                if (!int.TryParse(ValueAfterEquals(a), out width) || width <= 0 || width > 1024)
                {
                    Console.Error.WriteLine("error KQ0000: --width must be 1..1024");
                    Environment.Exit(1);
                    return true;
                }
            }
            else if (a.StartsWith("--height=", StringComparison.Ordinal))
            {
                if (!int.TryParse(ValueAfterEquals(a), out height) || height < 0 || height > 1024)
                {
                    Console.Error.WriteLine("error KQ0000: --height must be 0..1024");
                    Environment.Exit(1);
                    return true;
                }
            }
            else if (a.StartsWith("--out=", StringComparison.Ordinal))
            {
                outPath = ValueAfterEquals(a);
            }
            else
            {
                Console.Error.WriteLine("error KQ0000: unknown attrviz option: " + a);
                Environment.Exit(1);
                return true;
            }
        }

        try
        {
            if (!File.Exists(inputPath))
            {
                Console.Error.WriteLine("error KQ0000: attrviz input not found: " + inputPath);
                Environment.Exit(1);
                return true;
            }

            byte[] bytes = File.ReadAllBytes(inputPath);
            if (height <= 0)
            {
                // Infer enough rows for all bytes, including a partially filled final row.
                height = (bytes.Length + width - 1) / width;
            }

            string text = BuildAttrVizText(inputPath, bytes, width, height);
            if (string.IsNullOrWhiteSpace(outPath))
            {
                string stem = Path.GetFileNameWithoutExtension(inputPath);
                if (string.IsNullOrWhiteSpace(stem)) stem = "attr";
                outPath = Path.Combine(Path.GetDirectoryName(Path.GetFullPath(inputPath)) ?? ".", stem + ".attrviz.txt");
            }

            // The visualizer uses a dash to select stdout; ordinary paths are sent to the robust file writer.
            if (outPath == "-")
            {
                Console.Write(text);
            }
            else
            {
                IoUtil.WriteAllTextUtf8Robust(outPath, text, allowAlternatePath: true);
                Console.WriteLine("[attrviz] " + outPath);
            }

            Environment.Exit(0);
            return true;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine("error KQ0000: attrviz failed: " + ex.Message);
            Environment.Exit(1);
            return true;
        }
    }

    // Show raw bytes and the legacy GB/CGB-style bit interpretation, padding missing grid cells with placeholders.
    static string BuildAttrVizText(string inputPath, byte[] data, int width, int height)
    {
        if (data == null) data = new byte[0];
        if (width <= 0) width = 1;
        if (height < 0) height = 0;

        int n = data.Length;
        int rows = height;
        int cells = width * rows;

        // Summary counters cover the entire input, even when explicit dimensions show only a prefix in the grids.
        int[] palCount = new int[8];
        int bank1 = 0, dmgPal1 = 0, xflip = 0, yflip = 0, pri = 0;

        for (int i = 0; i < n; i++)
        {
            byte b = data[i];
            palCount[b & 0x07]++;
            if ((b & 0x08) != 0) bank1++;
            if ((b & 0x10) != 0) dmgPal1++;
            if ((b & 0x20) != 0) xflip++;
            if ((b & 0x40) != 0) yflip++;
            if ((b & 0x80) != 0) pri++;
        }

        var sb = new StringBuilder();
        sb.AppendLine("# KITAQGB tile attribute map visualization");
        sb.AppendLine("source=" + inputPath);
        sb.AppendLine("bytes=" + n);
        sb.AppendLine("width=" + width);
        sb.AppendLine("height=" + rows);
        sb.AppendLine("cells=" + cells);
        sb.AppendLine();
        sb.AppendLine("# bit layout: b0-2=palette b3=vram_bank b4=dmg_palette b5=xflip b6=yflip b7=priority");
        sb.AppendLine();
        sb.AppendLine("# summary");
        for (int p = 0; p < palCount.Length; p++) sb.AppendLine("palette_" + p + "=" + palCount[p]);
        sb.AppendLine("vram_bank_1=" + bank1);
        sb.AppendLine("dmg_palette_1=" + dmgPal1);
        sb.AppendLine("xflip_1=" + xflip);
        sb.AppendLine("yflip_1=" + yflip);
        sb.AppendLine("priority_1=" + pri);
        sb.AppendLine();

        sb.AppendLine("# grid raw(hex)");
        for (int y = 0; y < rows; y++)
        {
            sb.Append(y.ToString("D3")).Append(": ");
            for (int x = 0; x < width; x++)
            {
                int idx = y * width + x;
                if (idx < n) sb.Append(data[idx].ToString("X2"));
                else sb.Append("--");
                if (x + 1 < width) sb.Append(' ');
            }
            sb.AppendLine();
        }

        sb.AppendLine();
        sb.AppendLine("# grid decoded");
        sb.AppendLine("# token format: p<pal><B|.><D|.><X|.><Y|.><P|.>");
        for (int y = 0; y < rows; y++)
        {
            sb.Append(y.ToString("D3")).Append(": ");
            for (int x = 0; x < width; x++)
            {
                int idx = y * width + x;
                if (idx < n)
                {
                    byte b = data[idx];
                    int pal = b & 0x07;
                    sb.Append('p').Append((char)('0' + pal));
                    sb.Append((b & 0x08) != 0 ? 'B' : '.');
                    sb.Append((b & 0x10) != 0 ? 'D' : '.');
                    sb.Append((b & 0x20) != 0 ? 'X' : '.');
                    sb.Append((b & 0x40) != 0 ? 'Y' : '.');
                    sb.Append((b & 0x80) != 0 ? 'P' : '.');
                }
                else
                {
                    sb.Append("------");
                }

                if (x + 1 < width) sb.Append(' ');
            }
            sb.AppendLine();
        }

        return sb.ToString();
    }

    // Locate the repository integration script, forward its extra arguments and propagate the script process status.
    static bool TryRunTestCommand(string[] argsArray)
    {
        if (argsArray == null || argsArray.Length == 0) return false;
        if (!string.Equals(argsArray[0], "test", StringComparison.OrdinalIgnoreCase)) return false;

        string root = FindRepoRoot(Environment.CurrentDirectory);
        if (string.IsNullOrEmpty(root))
        {
            Console.Error.WriteLine("error KQ0000: integration_test/scripts/run_all.ps1 not found");
            Environment.Exit(1);
            return true;
        }

        string script = Path.Combine(root, "integration_test", "scripts", "run_all.ps1");
        if (!File.Exists(script))
        {
            Console.Error.WriteLine("error KQ0000: test script not found: " + script);
            Environment.Exit(1);
            return true;
        }

        var p = new ProcessStartInfo("powershell")
        {
            UseShellExecute = false,
            WorkingDirectory = root,
            CreateNoWindow = false,
            Arguments = "-NoProfile -ExecutionPolicy Bypass -File " + QuoteArg(script),
        };
        if (argsArray.Length > 1)
        {
            for (int i = 1; i < argsArray.Length; i++)
            {
                p.Arguments += " " + QuoteArg(argsArray[i]);
            }
        }

        try
        {
            using (var proc = Process.Start(p))
            {
                proc.WaitForExit();
                Environment.Exit(proc.ExitCode);
                return true;
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine("error KQ0000: failed to run test command: " + ex.Message);
            Environment.Exit(1);
            return true;
        }
    }

    // Recognize the bare watch flag case-insensitively before ordinary compilation dispatch.
    static bool HasWatchFlag(string[] args)
    {
        if (args == null) return false;
        return args.Any(a => string.Equals(a, "--watch", StringComparison.OrdinalIgnoreCase));
    }

    // Build in a child process, then wait for relevant file events beneath source/include directories before rebuilding.
    static void RunWatchDriver(string[] argsArray)
    {
        string exe = Process.GetCurrentProcess().MainModule.FileName;
        var childArgs = argsArray.Where(a => !string.Equals(a, "--watch", StringComparison.OrdinalIgnoreCase)).ToList();

        var sourceFiles = ExtractSourceFiles(childArgs);
        var includeDirs = ExtractIncludeDirs(childArgs);
        if (sourceFiles.Count == 0)
        {
            Console.Error.WriteLine("error KQ0000: --watch requires at least one source file");
            Environment.Exit(1);
            return;
        }

        // Collect existing parent/include directories once; this does not use the compiler-resolved dependency graph.
        var watchDirs = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        foreach (var s in sourceFiles)
        {
            string d = Path.GetDirectoryName(Path.GetFullPath(s));
            if (!string.IsNullOrWhiteSpace(d) && Directory.Exists(d)) watchDirs.Add(d);
        }
        foreach (var d in includeDirs)
        {
            try
            {
                string full = Path.GetFullPath(d);
                if (!string.IsNullOrWhiteSpace(full) && Directory.Exists(full)) watchDirs.Add(full);
            }
            catch { }
        }

        bool shouldStop = false;
        // Convert Ctrl+C into an orderly stop request instead of immediate process termination.
        Console.CancelKeyPress += (sender, e) =>
        {
            e.Cancel = true;
            shouldStop = true;
        };

        while (!shouldStop)
        {
            int code = RunChildCompiler(exe, childArgs.ToArray(), Environment.CurrentDirectory);
            Console.WriteLine("[watch] build exit code: " + code);
            if (shouldStop) break;

            // Coalesce file notifications while waiting; watchers are installed after the preceding child build finishes.
            using (var changed = new AutoResetEvent(false))
            {
                var watchers = new List<FileSystemWatcher>();
                foreach (var d in watchDirs)
                {
                    var w = new FileSystemWatcher(d)
                    {
                        IncludeSubdirectories = true,
                        NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.CreationTime | NotifyFilters.Size,
                        Filter = "*.*",
                        EnableRaisingEvents = true
                    };
                    FileSystemEventHandler onChanged = (s, e) =>
                    {
                        string ext = Path.GetExtension(e.FullPath).ToLowerInvariant();
                        if (ext == ".c" || ext == ".h" || ext == ".inc")
                        {
                            changed.Set();
                        }
                    };
                    // Filter rename events by the new path extension, matching the other event handlers.
                    RenamedEventHandler onRenamed = (s, e) =>
                    {
                        string ext = Path.GetExtension(e.FullPath).ToLowerInvariant();
                        if (ext == ".c" || ext == ".h" || ext == ".inc")
                        {
                            changed.Set();
                        }
                    };
                    w.Changed += onChanged;
                    w.Created += onChanged;
                    w.Deleted += onChanged;
                    w.Renamed += onRenamed;
                    watchers.Add(w);
                }

                Console.WriteLine("[watch] waiting for file changes...");
                while (!shouldStop)
                {
                    if (changed.WaitOne(200))
                    {
                        // Wait a fixed debounce interval after the first event before starting another build.
                        Thread.Sleep(180);
                        break;
                    }
                }
                foreach (var w in watchers) w.Dispose();
            }
        }

        Environment.Exit(0);
    }

    // Launch the executable directly with Windows-quoted arguments, inherit console streams and wait for its exit status.
    static int RunChildCompiler(string exePath, string[] args, string workingDir)
    {
        var psi = new ProcessStartInfo(exePath)
        {
            UseShellExecute = false,
            WorkingDirectory = workingDir,
            CreateNoWindow = false,
            Arguments = string.Join(" ", args.Select(QuoteArg))
        };
        using (var p = Process.Start(psi))
        {
            p.WaitForExit();
            return p.ExitCode;
        }
    }

    // Heuristically collect non-option arguments, skipping values for -o and -I only.
    // Other options with separate values require care when using this list for watch setup.
    static List<string> ExtractSourceFiles(List<string> args)
    {
        var files = new List<string>();
        for (int i = 0; i < args.Count; i++)
        {
            string a = args[i];
            if (a == "-o" || a == "-I")
            {
                i++;
                continue;
            }
            if (a.StartsWith("-")) continue;
            files.Add(a);
        }
        return files;
    }

    // Collect separate, attached and long-form include-directory arguments without resolving them here.
    static List<string> ExtractIncludeDirs(List<string> args)
    {
        var dirs = new List<string>();
        for (int i = 0; i < args.Count; i++)
        {
            string a = args[i];
            if (a == "-I")
            {
                if (i + 1 < args.Count) dirs.Add(args[i + 1]);
                i++;
                continue;
            }
            if (a.StartsWith("-I") && a.Length > 2)
            {
                dirs.Add(a.Substring(2));
                continue;
            }
            if (a.StartsWith("--include-dir="))
            {
                dirs.Add(ValueAfterEquals(a));
                continue;
            }
        }
        return dirs;
    }

    // Walk upward for the product solution file; the caller separately checks whether its integration script exists.
    static string FindRepoRoot(string startDir)
    {
        string dir = Path.GetFullPath(startDir);
        while (true)
        {
            if (File.Exists(Path.Combine(dir, "kitaqfc.sln"))) return dir;
            string parent = Path.GetDirectoryName(dir);
            if (string.IsNullOrEmpty(parent) || string.Equals(parent, dir, StringComparison.OrdinalIgnoreCase)) return null;
            dir = parent;
        }
    }

    // Encode one Windows process argument, including quotes and trailing backslashes.
    // Always quote the value; each slash run before a quote or the closing delimiter is doubled.
    static string QuoteArg(string a)
    {
        var sb = new StringBuilder("\"");
        int slashes = 0;
        foreach (char c in a ?? "")
        {
            if (c == '\\') { slashes++; continue; }
            if (c == '"')
            {
                sb.Append('\\', slashes * 2 + 1);
                sb.Append('"');
            }
            else
            {
                sb.Append('\\', slashes);
                sb.Append(c);
            }
            slashes = 0;
        }
        sb.Append('\\', slashes * 2);
        sb.Append('"');
        return sb.ToString();
    }

    // Remove recursive auto-minimize triggers, add the reduction driver and default trace mode, then wait for the child.
    static void TryRunAutoMinimize()
    {
        try
        {
            string exe = Process.GetCurrentProcess().MainModule.FileName;
            var args = new List<string>();
            foreach (var a in _originalArgs ?? new string[0])
            {
                if (string.Equals(a, "--minimize-on-fail", StringComparison.OrdinalIgnoreCase)) continue;
                if (string.Equals(a, "--auto-minimize", StringComparison.OrdinalIgnoreCase)) continue;
                args.Add(a);
            }
            if (!args.Any(a => a.StartsWith("--minimize", StringComparison.Ordinal)))
            {
                args.Add("--minimize");
            }
            if (!args.Any(a => a.StartsWith("--minimize-trace=", StringComparison.Ordinal)))
            {
                args.Add("--minimize-trace=final");
            }

            var psi = new ProcessStartInfo(exe)
            {
                UseShellExecute = false,
                CreateNoWindow = true,
                Arguments = string.Join(" ", args.Select(QuoteArg))
            };
            using (var p = Process.Start(psi))
            {
                p.WaitForExit();
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine("warning KQ0000: auto-minimize failed: " + ex.Message);
        }
    }

    // Serialize the current diagnostic snapshot and requested exit code before final termination.
    static void TryWriteDiagnosticJson(int exitCode)
    {
        if (!EmitDiagJson) return;

        var sb = new StringBuilder();
        sb.Append("{");
        sb.Append("\"exit_code\":").Append(exitCode).Append(",");
        sb.Append("\"errors\":").Append(ErrorCount).Append(",");
        sb.Append("\"warnings\":").Append(WarningCount).Append(",");
        sb.Append("\"output\":\"").Append(JsonEscape(_outputFilenameForDiag ?? "")).Append("\",");
        sb.Append("\"diagnostics\":[");
        for (int i = 0; i < Diagnostics.Count; i++)
        {
            var d = Diagnostics[i];
            if (i != 0) sb.Append(",");
            sb.Append("{");
            sb.Append("\"severity\":\"").Append(JsonEscape(d.Severity)).Append("\",");
            sb.Append("\"code\":\"").Append(JsonEscape(d.Code)).Append("\",");
            sb.Append("\"message\":\"").Append(JsonEscape(d.Message)).Append("\",");
            sb.Append("\"suggestion\":\"").Append(JsonEscape(d.Suggestion ?? "")).Append("\",");
            sb.Append("\"file\":\"").Append(JsonEscape(d.Filename)).Append("\",");
            sb.Append("\"line\":").Append(d.Line).Append(",");
            sb.Append("\"column\":").Append(d.Column);
            sb.Append("}");
        }
        sb.Append("]}");

        try
        {
            // Diagnostic JSON uses stderr for a dash destination, unlike attrviz stdout.
            if (DiagJsonPath == "-")
            {
                Console.Error.WriteLine(sb.ToString());
            }
            else
            {
                IoUtil.WriteAllTextUtf8Robust(DiagJsonPath, sb.ToString(), allowAlternatePath: true);
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine("warning KQ0000: failed to write diag json: " + ex.Message);
        }
    }

    // Escape quotes, backslashes and control characters for a JSON string body; callers provide the surrounding quotes.
    static string JsonEscape(string s)
    {
        if (string.IsNullOrEmpty(s)) return "";
        var sb = new StringBuilder(s.Length + 8);
        foreach (char c in s)
        {
            switch (c)
            {
                case '\\': sb.Append("\\\\"); break;
                case '"': sb.Append("\\\""); break;
                case '\n': sb.Append("\\n"); break;
                case '\r': sb.Append("\\r"); break;
                case '\t': sb.Append("\\t"); break;
                default:
                    if (c < 32) sb.Append("\\u").Append(((int)c).ToString("x4"));
                    else sb.Append(c);
                    break;
            }
        }
        return sb.ToString();
    }

    [DebuggerStepThrough]
    // Report an unsupported internal implementation path with its stable internal-error code.
    public static void NYI() => GeneralError(Severity.InternalError, Maybe.Nothing, ErrorCode.InternalNYI, "not yet implemented");
    [DebuggerStepThrough]
    // Report a compiler invariant case that reached no supported handler.
    public static void UnhandledCase() => GeneralError(Severity.InternalError, Maybe.Nothing, ErrorCode.InternalUnhandledCase, "unhandled case");

    // Use the same severity spellings in console diagnostics and structured output.
    static readonly Dictionary<Severity, string> SeverityText = new Dictionary<Severity, string>
    {
        { Severity.Warning, "warning" },
        { Severity.Error, "error" },
        { Severity.InternalError, "internal error" },
    };

    // Retain structured diagnostic fields independently of console visibility.
    sealed class DiagnosticEntry
    {
        public string Severity;
        public string Code;
        public string Message;
        public string Suggestion;
        public string Filename;
        public int Line;
        public int Column;
    }
}

// Symbolic constants used in Exprs.
// Centralize the string tags matched across parsing, lowering, code generation and assembly IR.
static class Tag
{
    // Top-level declarations:
    public static readonly string Function = "$function";
    public static readonly string InlineFunction = "$inline_function";
    public static readonly string FunctionDecl = "$function_decl";
    public static readonly string Constant = "$constant";
    public static readonly string Variable = "$variable";
    public static readonly string ExternVariable = "$extern_variable";
    public static readonly string ReadonlyData = "$readonly_data";
    public static readonly string Bank = "$bank";
    public static readonly string FixedBank = "$fixed_bank";
    public static readonly string FixedOrder = "$fixed_order";
    public static readonly string Static = "$static";
    public static readonly string Struct = "$struct";
    public static readonly string Union = "$union";
    public static readonly string OpaqueStruct = "$opaque_struct";
    public static readonly string OpaqueUnion = "$opaque_union";

    // Compile-time assertions:
    public static readonly string StaticAssert = "$static_assert";

    // Attributes:
    public static readonly string Range = "$range";
    // Declaration placement wrappers (produced by #pragma shortcuts)
    public static readonly string DeclAlign = "$decl_align";
    public static readonly string DeclSection = "$decl_section";

    // Safety boundary markers (zero runtime cost)
    public static readonly string Unsafe = "$unsafe";

    // Calling convention marker
    public static readonly string StackCall = "$stackcall";

    // Statements:
    public static readonly string Sequence = "$sequence";
    public static readonly string If = "$if";
    public static readonly string For = "$for";
    public static readonly string DoWhile = "$do_while";
    public static readonly string Continue = "$continue";
    public static readonly string Break = "$break";
    public static readonly string Fallthrough = "$fallthrough";
    public static readonly string Return = "$return";

    // Expressions:
    public static readonly string AddressOf = "$address_of";
    public static readonly string Field = "$field";
    public static readonly string Index = "$index";
    // Slice helper: __slice(ptr,len) lowered to $slice(ptr,len)
    public static readonly string Slice = "$slice";
    public static readonly string Call = "$call";
    public static readonly string Assign = "$assign";
    public static readonly string AssignModify = "$assign_modify";
    public static readonly string Conditional = "$conditional";
    public static readonly string Add = "$add";
    public static readonly string Subtract = "$sub";
    public static readonly string Multiply = "$mul";
    public static readonly string Divide = "$div";
    public static readonly string Modulus = "$mod";
    public static readonly string Load = "$load";
    public static readonly string Store = "$store";
    public static readonly string Cast = "$cast";
    public static readonly string RawOffset = "$raw_offset";

    public static readonly string Sizeof = "$sizeof";
    public static readonly string Offsetof = "$offsetof";

    public static readonly string Equal = "$equal";
    public static readonly string NotEqual = "$not_equal";
    public static readonly string LessThan = "$less_than";
    public static readonly string LessThanOrEqual = "$less_than_or_equal";
    public static readonly string GreaterThan = "$greater_than";
    public static readonly string GreaterThanOrEqual = "$greater_than_or_equal";

    public static readonly string BitwiseNot = "$bitwise_not";
    public static readonly string BitwiseAnd = "$bitwise_and";
    public static readonly string BitwiseOr = "$bitwise_or";
    public static readonly string BitwiseXor = "$bitwise_xor";
    public static readonly string ShiftLeft = "$shift_left";
    public static readonly string ShiftRight = "$shift_right";

    public static readonly string LogicalNot = "$logical_not";
    public static readonly string LogicalOr = "$logical_or";
    public static readonly string LogicalAnd = "$logical_and";

    public static readonly string PreIncrement = "$pre_increment";
    public static readonly string PostIncrement = "$post_increment";
    public static readonly string PreDecrement = "$pre_decrement";
    public static readonly string PostDecrement = "$post_decrement";

    // Assembly directives:
    public static readonly string Asm = "$asm";
    public static readonly string Label = "$label";
    public static readonly string Jump = "$jump";
    public static readonly string Comment = "$comment";
    public static readonly string SkipTo = "$skip_to";
    public static readonly string Align = "$align";
    public static readonly string Section = "$section";
    // Mark a target PRG-bank directive in the assembly IR.
    public static readonly string PrgBank = "$prg_bank";
    public static readonly string Word = "$word";

    // Assembler-side directives:
    // - $rst_map <vector:int> <targetLabel:string>
    // Instructs the assembler to place a JP stub at the given RST vector (0x00..0x38)
    // so that codegen/optimizer can safely emit RST_xx instead of CALL target.
    public static readonly string RstMap = "$rst_map";

    // Leaf expressions:
    public static readonly string Empty = "$empty";
    public static readonly string Integer = "$integer";
    public static readonly string Name = "$name";

    public static readonly string Switch = "$switch";
    public static readonly string Case = "$case";
}

// Distinguish recoverable warnings, compilation errors and fatal compiler-internal failures.
enum Severity
{
    Warning,
    Error,
    InternalError,
}

// Diagnostic codes (KQ0000..KQ9999). Keep the numbers stable once published.
enum ErrorCode
{
    None = 0,

    // Parser / front-end (1xxx)
    ParseError = 1000,
    ExpectedToken = 1001,
    ExpectedType = 1002,
    PreprocessorWarning = 1003,
    PreprocessorError = 1004,

    // Type / semantic (2xxx)
    AggregateNotDefined = 2100,
    IncompleteType = 2101,
    InvalidWramXBank = 2102,
    BankedWramRequiresCgbOnly = 2103,
    BankedWramLocalNotAllowed = 2104,
    WramXBankOverflow = 2105,

    // Const / qualifiers (22xx)
    ConstAssign = 2201,
    ConstModify = 2202,

    // Compile-time assertions (23xx)
    StaticAssertFailed = 2301,

    // Lints / warnings (24xx)
    MustCheckUnused = 2401,

    NonNullArgument = 2402,
    RangeViolation = 2403,

    // Lint: __range(min,max) index used with array length check
    RangeIndexOob = 2404,

    // Lint: switch(enum) case label type checks
    SwitchCaseEnumMismatch = 2405,
    SwitchCaseNonEnumOnEnumSwitch = 2406,

    // Lint: __enum_strict enum/integer mixing
    EnumStrictMix = 2407,

    // Lint: __bitflags misuse / mixing
    BitFlagsOp = 2408,
    BitFlagsMix = 2409,
    
    // Lint: __safe_index index/mixing
    SafeIndexIndex = 2410,
    SafeIndexMix = 2411,

    // Lint: __restrict aliasing (lightweight)
    RestrictAlias = 2412,

    // Lint: discarding const on pointer conversions (assignment / argument passing)
    ConstDiscard = 2413,

    // Lint: switch fallthrough annotation
    SwitchImplicitFallthrough = 2414,
    SwitchFallthroughUsage = 2415,

    // Lint: unreachable code
    UnreachableCode = 2416,

    // Lint: unused variable / function
    UnusedSymbol = 2417,

    // Lint: implicit narrowing conversion
    ImplicitNarrowing = 2418,

    // Lint: potentially dangerous pointer arithmetic
    PointerArithmeticDanger = 2419,
    ManualSvbkRequired = 2420,
    NesUnsafePpuAccess = 2421,
    NesMapperMismatch = 2422,
    NesFdsApiMapperMismatch = 2423,
    NesOamOverflowRisk = 2424,
    NesBankCrossingCall = 2425,
    LargeStructCopy = 2426,

    // Extern (25xx)
    ExternUndefined = 2501,
    ExternTypeMismatch = 2502,
    ExternNotSupported = 2503,

    // Internal errors (9xxx)
    Internal = 9000,
    InternalNYI = 9001,
    InternalUnhandledCase = 9002,
}
