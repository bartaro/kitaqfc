using System;
using System.Collections.Generic;

enum CompilationTargetKind
{
    Nes,
}

// Immutable target capabilities used to gate CLI features and choose output defaults.
sealed class CompilationTargetInfo
{
    public CompilationTargetKind Kind { get; private set; }
    public string CliName { get; private set; }
    public string DisplayName { get; private set; }
    public string DefaultOutputExtension { get; private set; }
    public bool SupportsRomHeaderPatching { get; private set; }
    public bool SupportsDisassembly { get; private set; }
    public bool SupportsAnalysisReports { get; private set; }
    public bool SupportsFunctionBankRelayout { get; private set; }

    // Capture the target identity and supported pipeline features in one configuration record.
    CompilationTargetInfo(CompilationTargetKind kind, string cliName, string displayName, string defaultOutputExtension,
        bool supportsRomHeaderPatching, bool supportsDisassembly, bool supportsAnalysisReports, bool supportsFunctionBankRelayout)
    {
        Kind = kind;
        CliName = cliName;
        DisplayName = displayName;
        DefaultOutputExtension = defaultOutputExtension;
        SupportsRomHeaderPatching = supportsRomHeaderPatching;
        SupportsDisassembly = supportsDisassembly;
        SupportsAnalysisReports = supportsAnalysisReports;
        SupportsFunctionBankRelayout = supportsFunctionBankRelayout;
    }

    public static readonly CompilationTargetInfo Nes = new CompilationTargetInfo(
        CompilationTargetKind.Nes,
        "nes",
        "Nintendo Entertainment System / Famicom",
        ".nes",
        supportsRomHeaderPatching: false,
        supportsDisassembly: false,
        supportsAnalysisReports: true,
        supportsFunctionBankRelayout: true);

    // Accept NES aliases case-insensitively after trimming. Empty input selects NES;
    // GB aliases receive a specific unsupported-target error. The out target remains
    // NES even on failure, so callers must check the Boolean result.
    public static bool TryParse(string raw, out CompilationTargetInfo target, out string error)
    {
        string value = (raw ?? "").Trim().ToLowerInvariant();
        switch (value)
        {
            case "":
            case "nes":
            case "fc":
            case "famicom":
            case "6502":
            case "2a03":
                target = Nes;
                error = null;
                return true;
            case "gb":
            case "dmg":
            case "cgb":
            case "gameboy":
            case "sm83":
                target = Nes;
                error = "error: Game Boy target is no longer included in this KITAQFC build; use --target=nes";
                return false;
            default:
                target = Nes;
                error = "error: --target must be nes|fc|famicom|6502|2a03";
                return false;
        }
    }
}

// Separate common pipeline orchestration from target-specific generation, assembly and reports.
interface ICompilationBackend
{
    CompilationTargetInfo TargetInfo { get; }
    IReadOnlyList<Expr> CompileAll(Expr syntaxTree);
    string Assemble(IReadOnlyList<Expr> assembly, string outputFilename);
    CodegenAnalysisReport CodegenReport { get; }
    AssemblerAnalysisReport AssemblerReport { get; }
}

static class TargetBackendFactory
{
    // Construct the sole backend in this build. The target parameter is currently
    // unused because target parsing has already restricted accepted targets to NES.
    public static ICompilationBackend Create(CompilationTargetInfo target)
    {
        return new NesCompilationBackend();
    }
}

sealed class NesCompilationBackend : ICompilationBackend
{
    public CompilationTargetInfo TargetInfo => CompilationTargetInfo.Nes;
    // Expose the generators' latest shared reports, or fresh empty reports when none exist.
    // These properties are not snapshots owned by this backend instance.
    public CodegenAnalysisReport CodegenReport => CodeGenerator.LastReport ?? new CodegenAnalysisReport();
    public AssemblerAnalysisReport AssemblerReport => Assembler.LastReport ?? new AssemblerAnalysisReport();

    // Delegate syntax-tree code generation to the NES generator and return its assembly expressions.
    public IReadOnlyList<Expr> CompileAll(Expr syntaxTree)
    {
        return CodeGenerator.CompileAll(syntaxTree);
    }

    // Assemble the generated expressions and return the assembler's actual output path.
    public string Assemble(IReadOnlyList<Expr> assembly, string outputFilename)
    {
        return Assembler.Assemble(assembly, outputFilename);
    }
}
