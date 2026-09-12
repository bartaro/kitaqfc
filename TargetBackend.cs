using System;
using System.Collections.Generic;

enum CompilationTargetKind
{
    Nes,
}

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
    public static ICompilationBackend Create(CompilationTargetInfo target)
    {
        return new NesCompilationBackend();
    }
}

sealed class NesCompilationBackend : ICompilationBackend
{
    public CompilationTargetInfo TargetInfo => CompilationTargetInfo.Nes;
    public CodegenAnalysisReport CodegenReport => CodeGenerator.LastReport ?? new CodegenAnalysisReport();
    public AssemblerAnalysisReport AssemblerReport => Assembler.LastReport ?? new AssemblerAnalysisReport();

    public IReadOnlyList<Expr> CompileAll(Expr syntaxTree)
    {
        return CodeGenerator.CompileAll(syntaxTree);
    }

    public string Assemble(IReadOnlyList<Expr> assembly, string outputFilename)
    {
        return Assembler.Assemble(assembly, outputFilename);
    }
}
