using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;

sealed class FdsDiskFileMetadata
{
    public int Id;
    public int Size;
    public int LoadAddress;
    public int FileType;
    public bool Overlay;
    public bool Boot = true;
    public string Name;
    public string SourcePath;
    public int Side = 0;
    public int FileNumber = -1;

    public byte[] ToRuntimeRecord()
    {
        return new byte[]
        {
            (byte)(Id & 0xFF),
            (byte)(Size & 0xFF),
            (byte)((Size >> 8) & 0xFF),
            (byte)(LoadAddress & 0xFF),
            (byte)((LoadAddress >> 8) & 0xFF),
            (byte)((FileType & 0x7F) | (Overlay ? 0x80 : 0x00)),
        };
    }
}

sealed class FdsDiskMetadata
{
    public static readonly FdsDiskMetadata Empty = new FdsDiskMetadata("", new List<FdsDiskFileMetadata>());

    public string SourcePath { get; private set; }
    public IReadOnlyList<FdsDiskFileMetadata> Files { get; private set; }
    public bool HasFiles { get { return Files != null && Files.Count > 0; } }

    public FdsDiskMetadata(string sourcePath, List<FdsDiskFileMetadata> files)
    {
        SourcePath = sourcePath ?? "";
        Files = (files ?? new List<FdsDiskFileMetadata>()).OrderBy(f => f.Id).ToList();
    }

    public byte[] BuildRuntimeTable()
    {
        return BuildRuntimeTable(null);
    }

    public byte[] BuildRuntimeTable(IEnumerable<FdsDiskFileMetadata> additionalFiles)
    {
        var bytes = new List<byte>();
        foreach (var f in GetMergedFiles(additionalFiles))
            bytes.AddRange(f.ToRuntimeRecord());
        bytes.Add(0xFF);
        return bytes.ToArray();
    }

    public IReadOnlyList<FdsDiskFileMetadata> GetMergedFiles(IEnumerable<FdsDiskFileMetadata> additionalFiles)
    {
        var merged = new List<FdsDiskFileMetadata>();
        merged.AddRange(Files ?? Array.Empty<FdsDiskFileMetadata>());
        if (additionalFiles != null)
            merged.AddRange(additionalFiles.Where(f => f != null));
        return merged.OrderBy(f => f.Id).ToList();
    }

    public string ToNormalizedJson()
    {
        var sb = new StringBuilder();
        sb.AppendLine("{");
        sb.AppendLine("  \"format\": \"kitaqfc-fds-metadata-v1\",");
        sb.AppendLine("  \"files\": [");
        for (int i = 0; i < Files.Count; i++)
        {
            var f = Files[i];
            sb.Append("    { ");
            sb.Append("\"id\": ").Append(f.Id).Append(", ");
            sb.Append("\"size\": ").Append(f.Size).Append(", ");
            sb.Append("\"load\": \"0x").Append(f.LoadAddress.ToString("X4")).Append("\", ");
            sb.Append("\"type\": ").Append(f.FileType).Append(", ");
            sb.Append("\"overlay\": ").Append(f.Overlay ? "true" : "false");
            if (!string.IsNullOrEmpty(f.Name))
                sb.Append(", \"name\": \"").Append(EscapeJson(f.Name)).Append("\"");
            if (!string.IsNullOrEmpty(f.SourcePath))
                sb.Append(", \"source\": \"").Append(EscapeJson(f.SourcePath)).Append("\"");
            if (!f.Boot)
                sb.Append(", \"boot\": false");
            if (f.Side != 0)
                sb.Append(", \"side\": ").Append(f.Side);
            if (f.FileNumber >= 0)
                sb.Append(", \"number\": ").Append(f.FileNumber);
            sb.Append(" }");
            if (i + 1 < Files.Count) sb.Append(",");
            sb.AppendLine();
        }
        sb.AppendLine("  ]");
        sb.AppendLine("}");
        return sb.ToString();
    }

    static string EscapeJson(string value)
    {
        return (value ?? "").Replace("\\", "\\\\").Replace("\"", "\\\"");
    }

    public static FdsDiskMetadata LoadFromFile(string path)
    {
        if (string.IsNullOrWhiteSpace(path)) return Empty;
        string full = Path.GetFullPath(path);
        string text = File.ReadAllText(full, Encoding.UTF8);
        var files = LooksLikeJson(text) ? ParseJsonLike(text) : ParseDelimited(text);
        ValidateFiles(files, full);
        return new FdsDiskMetadata(full, files);
    }

    static bool LooksLikeJson(string text)
    {
        string t = (text ?? "").TrimStart();
        return t.StartsWith("{") || t.StartsWith("[");
    }

    static List<FdsDiskFileMetadata> ParseJsonLike(string text)
    {
        var files = new List<FdsDiskFileMetadata>();
        foreach (Match m in Regex.Matches(text ?? "", "\\{[^{}]*\\}", RegexOptions.Singleline))
        {
            string obj = m.Value;
            if (!TryReadInt(obj, "id|file_id|fileId|number|file", out int id))
                continue;
            TryReadInt(obj, "size|length|len|bytes", out int size);
            TryReadInt(obj, "load|load_addr|loadAddress|addr|address|dst", out int load);
            TryReadInt(obj, "type|file_type|fileType", out int type);
            bool overlay = TryReadBool(obj, "overlay|is_overlay|isOverlay", out bool ov) && ov;
            bool boot = !(TryReadBool(obj, "boot|boot_file|bootFile|load_on_boot|loadOnBoot", out bool bt) && !bt);
            string name = TryReadString(obj, "name|filename|file_name|label", out string n) ? n : "";
            string source = TryReadString(obj, "source|src|path|data|binary|asset", out string sp) ? sp : "";
            int side = TryReadInt(obj, "side|disk_side", out int sd) ? sd : 0;
            int number = TryReadInt(obj, "number|file_number|fileNumber|index", out int no) ? no : -1;
            files.Add(new FdsDiskFileMetadata { Id = id, Size = size, LoadAddress = load, FileType = type, Overlay = overlay, Boot = boot, Name = name, SourcePath = source, Side = side, FileNumber = number });
        }
        return files;
    }

    static List<FdsDiskFileMetadata> ParseDelimited(string text)
    {
        var files = new List<FdsDiskFileMetadata>();
        int lineNo = 0;
        foreach (string raw in (text ?? "").Replace("\r", "").Split('\n'))
        {
            lineNo++;
            string line = raw.Trim();
            if (line.Length == 0 || line.StartsWith("#") || line.StartsWith(";")) continue;
            string[] parts = line.Split(new[] { ',', '\t', ' ' }, StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length < 3) continue;
            if (parts[0].Equals("id", StringComparison.OrdinalIgnoreCase)) continue;
            int id = ParseNumber(parts[0], "id", lineNo);
            int size = ParseNumber(parts[1], "size", lineNo);
            int load = ParseNumber(parts[2], "load", lineNo);
            int type = parts.Length >= 4 ? ParseNumber(parts[3], "type", lineNo) : 0;
            bool overlay = parts.Length >= 5 && ParseBoolToken(parts[4]);
            string name = parts.Length >= 6 ? parts[5] : "";
            string source = parts.Length >= 7 ? parts[6] : "";
            int side = parts.Length >= 8 ? ParseNumber(parts[7], "side", lineNo) : 0;
            int number = parts.Length >= 9 ? ParseNumber(parts[8], "number", lineNo) : -1;
            bool boot = parts.Length >= 10 ? ParseBoolToken(parts[9]) : true;
            files.Add(new FdsDiskFileMetadata { Id = id, Size = size, LoadAddress = load, FileType = type, Overlay = overlay, Boot = boot, Name = name, SourcePath = source, Side = side, FileNumber = number });
        }
        return files;
    }

    static bool TryReadInt(string obj, string keyAlternatives, out int value)
    {
        foreach (string key in keyAlternatives.Split('|'))
        {
            var m = Regex.Match(obj, "[\\\"']" + Regex.Escape(key) + "[\\\"']\\s*:\\s*([\\\"']?)([-+]?0x[0-9A-Fa-f]+|[-+]?[0-9]+)\\1", RegexOptions.IgnoreCase);
            if (m.Success)
            {
                value = ParseNumberToken(m.Groups[2].Value);
                return true;
            }
        }
        value = 0;
        return false;
    }

    static bool TryReadBool(string obj, string keyAlternatives, out bool value)
    {
        foreach (string key in keyAlternatives.Split('|'))
        {
            var m = Regex.Match(obj, "[\\\"']" + Regex.Escape(key) + "[\\\"']\\s*:\\s*([\\\"']?)(true|false|yes|no|1|0)\\1", RegexOptions.IgnoreCase);
            if (m.Success)
            {
                value = ParseBoolToken(m.Groups[2].Value);
                return true;
            }
        }
        value = false;
        return false;
    }

    static bool TryReadString(string obj, string keyAlternatives, out string value)
    {
        foreach (string key in keyAlternatives.Split('|'))
        {
            var m = Regex.Match(obj, "[\\\"']" + Regex.Escape(key) + "[\\\"']\\s*:\\s*[\\\"']([^\\\"']*)[\\\"']", RegexOptions.IgnoreCase);
            if (m.Success)
            {
                value = m.Groups[1].Value;
                return true;
            }
        }
        value = "";
        return false;
    }

    static int ParseNumber(string token, string field, int lineNo)
    {
        try { return ParseNumberToken(token); }
        catch { throw new FormatException("invalid FDS metadata " + field + " at line " + lineNo + ": " + token); }
    }

    static int ParseNumberToken(string token)
    {
        string t = (token ?? "").Trim();
        if (t.StartsWith("$")) return int.Parse(t.Substring(1), NumberStyles.HexNumber, CultureInfo.InvariantCulture);
        if (t.StartsWith("0x", StringComparison.OrdinalIgnoreCase)) return int.Parse(t.Substring(2), NumberStyles.HexNumber, CultureInfo.InvariantCulture);
        return int.Parse(t, CultureInfo.InvariantCulture);
    }

    static bool ParseBoolToken(string token)
    {
        string t = (token ?? "").Trim().ToLowerInvariant();
        return t == "1" || t == "true" || t == "yes" || t == "y" || t == "overlay";
    }

    static void ValidateFiles(List<FdsDiskFileMetadata> files, string path)
    {
        if (files == null || files.Count == 0)
            throw new FormatException("FDS metadata has no file records: " + path);
        if (files.Count > 42)
            throw new FormatException("FDS metadata supports up to 42 files in the current 6-byte runtime table: " + files.Count);
        var ids = new HashSet<int>();
        foreach (var f in files)
        {
            if (f.Id < 0 || f.Id > 254) throw new FormatException("FDS file id must be 0..254: " + f.Id);
            if (!ids.Add(f.Id)) throw new FormatException("duplicate FDS file id: " + f.Id);
            if (f.Size < 0 || f.Size > 0xFFFF) throw new FormatException("FDS file size must be 0..65535 for id " + f.Id);
            if (f.LoadAddress < 0 || f.LoadAddress > 0xFFFF) throw new FormatException("FDS load address must be 0..65535 for id " + f.Id);
            if (f.FileType < 0 || f.FileType > 0x7F) throw new FormatException("FDS file type must be 0..127 for id " + f.Id);
            if (f.Side < 0 || f.Side > 254) throw new FormatException("FDS side must be 0..254 for id " + f.Id);
            if (f.FileNumber < -1 || f.FileNumber > 254) throw new FormatException("FDS file number must be -1..254 for id " + f.Id);
        }
    }
}
