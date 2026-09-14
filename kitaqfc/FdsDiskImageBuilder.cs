using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;

// Carry generated bank-overlay bytes and their disk placement from code generation to the image builder.
sealed class FdsAutoOverlayFile
{
    public int Id;
    public int Side;
    public int Number = -1;
    public string Name;
    public int LoadAddress;
    public int FileType;
    public bool Boot;
    public bool Overlay;
    public int SourceBank;
    public byte[] Data;
}

// Return assembled image bytes and counts with nonfatal conversion warnings; errors are reported through Program.Error.
sealed class FdsDiskImageBuildResult
{
    public byte[] Image = new byte[0];
    public int SideCount;
    public int FileCount;
    public List<string> Warnings = new List<string>();
}

// Build fixed-size raw FDS side images from compiled PRG/CHR and optional metadata, with an optional 16-byte wrapper.
sealed class FdsDiskImageBuilder
{
    const int FdsHeaderSize = 16;
    const int SideSize = 65500;
    const int LegacyPrgRamLoadBase = 0x8000;
    const int LegacyPrgRamBootSize = 0x6000; // $8000-$DFFF when converting old NROM-like images.
    const int FdsPrgRamLoadBase = 0x6000;
    const int FdsPrgRamBootSize = 0x8000; // $6000-$DFFF. $E000-$FFFF is the fixed FDS BIOS.
    const int NesBankSize = 0x4000;
    const int FdsNmiTriggerAddress = 0x2000; // PPUCTRL. Used by the approval/license bypass boot file.
    const int FdsBypassStallBytes = 8192;

    // Internal resolved file record containing the payload bytes and physical disk ordering information.
    sealed class DiskFile
    {
        public int Side;
        public int Number;
        public int Id;
        public int LoadAddress;
        public int FileType;
        public bool Boot;
        public string Name;
        public byte[] Data;
    }

    // Resolve boot data and optional overlays, assign file numbers, then concatenate all side images.
    // Side gaps produce empty sides, and the optional wrapper records the side count in one byte.
    public static FdsDiskImageBuildResult BuildFromCompiledImage(
        byte[] prgRom,
        byte[] chrRom,
        FdsDiskMetadata metadata,
        bool includeHeader,
        string gameCode,
        bool licenseBypass,
        bool fdsPrgRamLayout,
        IReadOnlyList<FdsAutoOverlayFile> autoOverlayFiles)
    {
        var result = new FdsDiskImageBuildResult();
        var warnings = result.Warnings;

        byte[] autoPrg = BuildFdsPrgBootImage(prgRom ?? new byte[0], fdsPrgRamLayout, warnings);
        int autoPrgLoadBase = fdsPrgRamLayout ? FdsPrgRamLoadBase : LegacyPrgRamLoadBase;
        byte[] autoChr = NormalizeChr(chrRom);

        var files = BuildDiskFiles(autoPrg, autoChr, metadata, warnings, autoPrgLoadBase, autoOverlayFiles);
        if (licenseBypass)
            AddLicenseBypassBootFiles(files, warnings);

        if (files.Count == 0)
        {
            Program.Error("error KQFC2501: FDS image builder has no files to write.");
            return result;
        }

        AssignFileNumbers(files);
        int sideCount = Math.Max(1, files.Select(f => f.Side).DefaultIfEmpty(0).Max() + 1);
        var sideImages = new List<byte[]>();
        for (int side = 0; side < sideCount; side++)
            sideImages.Add(BuildSide(side, files.Where(f => f.Side == side).OrderBy(f => f.Number).ThenBy(f => f.Id).ToList(), gameCode, warnings));

        int total = (includeHeader ? FdsHeaderSize : 0) + (SideSize * sideCount);
        byte[] image = new byte[total];
        int pos = 0;
        if (includeHeader)
        {
            image[0] = 0x46; // F
            image[1] = 0x44; // D
            image[2] = 0x53; // S
            image[3] = 0x1A;
            image[4] = (byte)Math.Min(255, sideCount);
            pos = FdsHeaderSize;
        }

        foreach (var sideImage in sideImages)
        {
            Buffer.BlockCopy(sideImage, 0, image, pos, SideSize);
            pos += SideSize;
        }

        result.Image = image;
        result.SideCount = sideCount;
        result.FileCount = files.Count;
        return result;
    }

    // Construct a 0xFF-padded PRG boot file for native $6000-$DFFF or legacy $8000-$DFFF placement.
    // The legacy path combines the first bank, lower common-bank bytes and relocated vectors, warning about omitted BIOS-region data.
    static byte[] BuildFdsPrgBootImage(byte[] prgRom, bool fdsPrgRamLayout, List<string> warnings)
    {
        int bootSize = fdsPrgRamLayout ? FdsPrgRamBootSize : LegacyPrgRamBootSize;
        byte[] dst = new byte[bootSize];
        for (int i = 0; i < dst.Length; i++) dst[i] = 0xFF;

        if (prgRom.Length == 0)
        {
            warnings.Add("compiled PRG was empty; FDS PRG file was filled with $FF.");
            return dst;
        }

        if (fdsPrgRamLayout)
        {
            int len = Math.Min(prgRom.Length, dst.Length);
            Buffer.BlockCopy(prgRom, 0, dst, 0, len);
            if (prgRom.Length < dst.Length)
                warnings.Add("compiled FDS PRG is smaller than 32 KiB; FDS PRG-RAM boot file was padded with $FF.");
            if (prgRom.Length > dst.Length && ContainsNonFill(prgRom, dst.Length, prgRom.Length, 0xFF))
                warnings.Add("compiled FDS PRG exceeds $6000-$DFFF; bytes after the first 32 KiB were not included in the boot file.");
            return dst;
        }

        if (prgRom.Length < NesBankSize)
        {
            warnings.Add("compiled PRG is smaller than 16 KiB; legacy FDS PRG file was padded with $FF.");
            Buffer.BlockCopy(prgRom, 0, dst, 0, Math.Min(prgRom.Length, dst.Length));
            return dst;
        }

        int switchableLen = Math.Min(NesBankSize, prgRom.Length);
        Buffer.BlockCopy(prgRom, 0, dst, 0, Math.Min(switchableLen, dst.Length));

        int commonStart = Math.Max(0, prgRom.Length - NesBankSize);
        int commonLowLen = Math.Min(0x2000, prgRom.Length - commonStart);
        if (commonLowLen > 0)
            Buffer.BlockCopy(prgRom, commonStart, dst, 0x4000, Math.Min(commonLowLen, 0x2000));

        int inesVectorOffset = commonStart + 0x3FFA;
        int fdsVectorOffset = 0x5FFA; // $DFFA in a legacy file loaded at $8000.
        if (inesVectorOffset + 6 <= prgRom.Length)
            Buffer.BlockCopy(prgRom, inesVectorOffset, dst, fdsVectorOffset, 6);
        else
            warnings.Add("compiled PRG did not contain an iNES vector tail; FDS vectors remain as padded bytes.");

        int upperCommonStart = commonStart + 0x2000;
        int upperCommonEnd = Math.Min(commonStart + 0x3FC0, prgRom.Length);
        if (upperCommonStart < upperCommonEnd && ContainsNonFill(prgRom, upperCommonStart, upperCommonEnd, 0xFF))
            warnings.Add("compiled common-bank bytes in $E000-$FFBF cannot be loaded on FDS because BIOS occupies $E000-$FFFF; those bytes were not included in the .fds PRG file. Use --fds-layout=fds32 for native $6000-$DFFF placement.");

        return dst;
    }

    // Produce exactly 8 KiB of CHR data: clone an exact input, otherwise truncate or pad with zeros.
    static byte[] NormalizeChr(byte[] chrRom)
    {
        if (chrRom == null || chrRom.Length == 0)
            return new byte[0x2000];

        if (chrRom.Length == 0x2000)
            return (byte[])chrRom.Clone();

        int len = Math.Min(chrRom.Length, 0x2000);
        byte[] dst = new byte[0x2000];
        Buffer.BlockCopy(chrRom, 0, dst, 0, len);
        return dst;
    }

    // Use default PRG/CHR boot files when metadata is empty; otherwise resolve the listed files and append automatic overlays.
    // Check duplicate ids after combining both sources.
    static List<DiskFile> BuildDiskFiles(byte[] autoPrg, byte[] autoChr, FdsDiskMetadata metadata, List<string> warnings, int autoPrgLoadBase, IReadOnlyList<FdsAutoOverlayFile> autoOverlayFiles)
    {
        var files = new List<DiskFile>();
        var metaFiles = (metadata == null ? null : metadata.Files) ?? new List<FdsDiskFileMetadata>();
        if (metaFiles.Count == 0)
        {
            files.Add(new DiskFile { Side = 0, Number = 0, Id = 0, Name = "KQFPRG", LoadAddress = autoPrgLoadBase, FileType = 0, Boot = true, Data = autoPrg });
            files.Add(new DiskFile { Side = 0, Number = 1, Id = 1, Name = "KQFCHR", LoadAddress = 0x0000, FileType = 1, Boot = true, Data = autoChr });
        }
        else
        {
            string baseDir = "";
            if (!string.IsNullOrWhiteSpace(metadata.SourcePath))
                baseDir = Path.GetDirectoryName(metadata.SourcePath) ?? "";

            foreach (var f in metaFiles)
            {
                byte[] data = ResolveFileData(f, autoPrg, autoChr, baseDir, warnings, autoPrgLoadBase);
                files.Add(new DiskFile
                {
                    Side = Math.Max(0, f.Side),
                    Number = f.FileNumber,
                    Id = f.Id,
                    Name = string.IsNullOrWhiteSpace(f.Name) ? ("FILE" + f.Id.ToString("D3")) : f.Name,
                    LoadAddress = f.LoadAddress,
                    FileType = f.FileType,
                    Boot = f.Boot,
                    Data = data,
                });
            }
        }

        foreach (var f in autoOverlayFiles ?? Array.Empty<FdsAutoOverlayFile>())
        {
            if (f == null) continue;
            files.Add(new DiskFile
            {
                Side = Math.Max(0, f.Side),
                Number = f.Number,
                Id = f.Id,
                Name = string.IsNullOrWhiteSpace(f.Name) ? ("OVL" + f.Id.ToString("D3")) : f.Name,
                LoadAddress = f.LoadAddress,
                FileType = f.FileType,
                Boot = f.Boot,
                Data = f.Data ?? new byte[0],
            });
        }

        if (autoOverlayFiles != null && autoOverlayFiles.Count > 0)
            warnings.Add("FDS auto overlay export enabled: " + autoOverlayFiles.Count + " PRG bank overlay file(s) were added for bank 2+.");

        ValidateNoDuplicateFileIds(files);
        return files;
    }

    // Report the first duplicate id across all sides. This helper reports an error but does not stop its caller directly.
    static void ValidateNoDuplicateFileIds(List<DiskFile> files)
    {
        var seen = new HashSet<int>();
        foreach (var f in files ?? new List<DiskFile>())
        {
            if (!seen.Add(f.Id))
            {
                Program.Error("error KQFC2506: duplicate FDS file id {0}. Change --fds-overlay-start-id or the --fds-meta id values.", f.Id);
                return;
            }
        }
    }

    // Append a side-zero boot write that enables PPU NMI and an 8 KiB non-boot delay payload.
    // Choose unused ids above the side-zero maximum; byte-range validation is not performed in this helper.
    static void AddLicenseBypassBootFiles(List<DiskFile> files, List<string> warnings)
    {
        if (files == null) return;
        var side0 = files.Where(f => f.Side == 0).ToList();
        int nextId = side0.Select(f => f.Id).DefaultIfEmpty(-1).Max() + 1;
        while (files.Any(f => f.Id == nextId)) nextId++;
        int triggerId = nextId;
        nextId++;
        while (files.Any(f => f.Id == nextId)) nextId++;
        int stallId = nextId;

        files.Add(new DiskFile
        {
            Side = 0,
            Number = -1,
            Id = triggerId,
            Name = "KQFNMI",
            LoadAddress = FdsNmiTriggerAddress,
            FileType = 0,
            Boot = true,
            Data = new byte[] { 0x80 }, // PPUCTRL.NMI = 1, intentionally triggers NMI during boot loading.
        });

        byte[] stall = new byte[FdsBypassStallBytes];
        for (int i = 0; i < stall.Length; i++) stall[i] = 0xFF;
        files.Add(new DiskFile
        {
            Side = 0,
            Number = -1,
            Id = stallId,
            Name = "KQFWAIT",
            LoadAddress = FdsPrgRamLoadBase,
            FileType = 0,
            Boot = false,
            Data = stall,
        });

        warnings.Add("FDS approval/license bypass enabled: added KQFNMI boot file and KQFWAIT non-boot stall file. Use --fds-no-license-bypass to emit a traditional approval-screen disk layout.");
    }

    // Prefer an explicit source path relative to the metadata file; otherwise select compiled PRG or CHR data by file type.
    // Unfilled tails and unsupported source-less file types use zero bytes, with warnings where indicated.
    static byte[] ResolveFileData(FdsDiskFileMetadata f, byte[] autoPrg, byte[] autoChr, string baseDir, List<string> warnings, int autoPrgLoadBase)
    {
        if (!string.IsNullOrWhiteSpace(f.SourcePath))
        {
            string path = f.SourcePath;
            if (!Path.IsPathRooted(path) && !string.IsNullOrWhiteSpace(baseDir))
                path = Path.Combine(baseDir, path);
            try
            {
                byte[] raw = File.ReadAllBytes(path);
                return ApplyDeclaredSize(raw, f.Size, warnings, "FDS file id " + f.Id);
            }
            catch (Exception ex)
            {
                Program.Error("error KQFC2502: failed to read FDS file source '{0}' for id {1}: {2}", f.SourcePath, f.Id, ex.Message);
                return new byte[0];
            }
        }

        if (f.FileType == 0)
        {
            // Clamp load addresses below the automatic PRG base to its first byte before selecting the payload slice.
            int offset = Math.Max(0, f.LoadAddress - autoPrgLoadBase);
            int size = f.Size > 0 ? f.Size : Math.Max(0, autoPrg.Length - offset);
            if (offset < 0 || offset >= autoPrg.Length)
            {
                warnings.Add("FDS metadata id " + f.Id + " has PRG load address outside the auto PRG image; zero-filled data was emitted.");
                return new byte[Math.Max(0, size)];
            }
            int len = Math.Min(size, autoPrg.Length - offset);
            byte[] data = new byte[size];
            Buffer.BlockCopy(autoPrg, offset, data, 0, len);
            return data;
        }

        if (f.FileType == 1 || f.FileType == 2)
        {
            int size = f.Size > 0 ? f.Size : autoChr.Length;
            return ApplyDeclaredSize(autoChr, size, warnings, "FDS CHR/VRAM file id " + f.Id);
        }

        if (f.Size > 0)
        {
            warnings.Add("FDS metadata id " + f.Id + " has no source; zero-filled file data was emitted.");
            return new byte[f.Size];
        }

        warnings.Add("FDS metadata id " + f.Id + " has no source and no size; emitted an empty data block.");
        return new byte[0];
    }

    // Clone unchanged input when size is unspecified or exact; otherwise resize with zero padding or truncation and warn.
    static byte[] ApplyDeclaredSize(byte[] raw, int declaredSize, List<string> warnings, string label)
    {
        raw = raw ?? new byte[0];
        if (declaredSize <= 0 || declaredSize == raw.Length)
            return (byte[])raw.Clone();

        byte[] data = new byte[declaredSize];
        int len = Math.Min(raw.Length, data.Length);
        Buffer.BlockCopy(raw, 0, data, 0, len);
        if (raw.Length > declaredSize)
            warnings.Add(label + " was truncated from " + raw.Length + " to " + declaredSize + " bytes.");
        else
            warnings.Add(label + " was padded from " + raw.Length + " to " + declaredSize + " bytes.");
        return data;
    }

    // Within each side, keep explicit numbers and allocate missing numbers after them in id order.
    // Existing duplicate explicit numbers are not rejected here.
    static void AssignFileNumbers(List<DiskFile> files)
    {
        foreach (var group in files.GroupBy(f => f.Side))
        {
            int n = 0;
            foreach (var f in group.OrderBy(f => f.Number < 0 ? int.MaxValue : f.Number).ThenBy(f => f.Id))
            {
                if (f.Number < 0) f.Number = n;
                n = Math.Max(n, f.Number + 1);
            }
        }
    }

    // Write disk-info and count blocks followed by each file header/data block into a zero-filled fixed-size side.
    // This byte-stream format omits physical gap and CRC encoding.
    static byte[] BuildSide(int side, List<DiskFile> files, string gameCode, List<string> warnings)
    {
        byte[] data = new byte[SideSize];
        int pos = 0;

        byte[] block1 = BuildDiskInfoBlock(side, files, gameCode);
        CopyBlock(data, ref pos, block1, side, warnings);

        byte[] block2 = new byte[] { 0x02, (byte)Math.Min(255, files.Count) };
        CopyBlock(data, ref pos, block2, side, warnings);

        foreach (var f in files)
        {
            CopyBlock(data, ref pos, BuildFileHeaderBlock(f), side, warnings);

            byte[] block4 = new byte[1 + (f.Data == null ? 0 : f.Data.Length)];
            block4[0] = 0x04;
            if (f.Data != null && f.Data.Length > 0)
                Buffer.BlockCopy(f.Data, 0, block4, 1, f.Data.Length);
            CopyBlock(data, ref pos, block4, side, warnings);
        }

        return data;
    }

    // Emit the fixed disk-info template with format identifier, game code, side/disk numbers and highest boot file id.
    // Boot eligibility is represented as an id threshold, not an independent flag in each on-disk file header.
    static byte[] BuildDiskInfoBlock(int side, List<DiskFile> files, string gameCode)
    {
        byte[] b = new byte[56];
        b[0] = 0x01;
        WriteAscii(b, 1, 14, "*NINTENDO-HVC*");
        b[0x0F] = 0x00;
        WriteAscii(b, 0x10, 3, NormalizeGameCode(gameCode));
        b[0x13] = 0x20;
        b[0x14] = 0x00;
        b[0x15] = (byte)(side & 1);
        b[0x16] = (byte)(side / 2);
        b[0x17] = 0x00;
        b[0x18] = 0x00;
        b[0x19] = (byte)Math.Min(254, files.Where(f => f.Boot).Select(f => f.Id).DefaultIfEmpty(0).Max());
        for (int i = 0; i < 5; i++) b[0x1A + i] = 0xFF;
        b[0x1F] = 0x26; b[0x20] = 0x04; b[0x21] = 0x25;
        b[0x22] = 0x49;
        b[0x23] = 0x61;
        b[0x24] = 0x00;
        b[0x25] = 0x00; b[0x26] = 0x02;
        for (int i = 0; i < 5; i++) b[0x27 + i] = 0x00;
        b[0x2C] = 0x26; b[0x2D] = 0x04; b[0x2E] = 0x25;
        b[0x2F] = 0x00;
        b[0x30] = 0x80;
        b[0x31] = 0x00; b[0x32] = 0x00;
        b[0x33] = 0x07;
        b[0x34] = 0x00;
        b[0x35] = (byte)(side & 1);
        b[0x36] = 0x00;
        b[0x37] = 0x00;
        return b;
    }

    // Serialize file number/id, an eight-byte name, little-endian load address/size and the seven-bit file type.
    // Values are masked to the on-disk widths; validation belongs to earlier stages.
    static byte[] BuildFileHeaderBlock(DiskFile f)
    {
        byte[] b = new byte[16];
        b[0] = 0x03;
        b[1] = (byte)(f.Number & 0xFF);
        b[2] = (byte)(f.Id & 0xFF);
        WriteAscii(b, 3, 8, f.Name);
        int addr = f.LoadAddress & 0xFFFF;
        int size = (f.Data == null ? 0 : f.Data.Length) & 0xFFFF;
        b[0x0B] = (byte)(addr & 0xFF);
        b[0x0C] = (byte)((addr >> 8) & 0xFF);
        b[0x0D] = (byte)(size & 0xFF);
        b[0x0E] = (byte)((size >> 8) & 0xFF);
        b[0x0F] = (byte)(f.FileType & 0x7F);
        return b;
    }

    // Append a block only when it fits; otherwise report an error and leave the write position unchanged.
    static void CopyBlock(byte[] side, ref int pos, byte[] block, int sideIndex, List<string> warnings)
    {
        if (block == null) block = new byte[0];
        if (pos + block.Length > side.Length)
        {
            Program.Error("error KQFC2503: FDS side {0} exceeds {1} bytes while writing block {2}.", sideIndex, SideSize, block.Length);
            return;
        }
        Buffer.BlockCopy(block, 0, side, pos, block.Length);
        pos += block.Length;
    }

    // Uppercase and trim to three characters, defaulting empty input to KQF and padding short codes with spaces.
    static string NormalizeGameCode(string gameCode)
    {
        string s = (gameCode ?? "KQF").Trim().ToUpperInvariant();
        if (s.Length == 0) s = "KQF";
        if (s.Length > 3) s = s.Substring(0, 3);
        while (s.Length < 3) s += " ";
        return s;
    }

    // Write a fixed-width ASCII field, truncating long text and space-padding short text.
    static void WriteAscii(byte[] dst, int offset, int length, string text)
    {
        byte[] raw = Encoding.ASCII.GetBytes(text ?? "");
        for (int i = 0; i < length; i++)
            dst[offset + i] = i < raw.Length ? raw[i] : (byte)0x20;
    }

    // Test a bounds-clamped half-open byte range for data that would be lost rather than harmless padding.
    static bool ContainsNonFill(byte[] data, int start, int endExclusive, byte fill)
    {
        for (int i = Math.Max(0, start); i < Math.Min(data.Length, endExclusive); i++)
            if (data[i] != fill) return true;
        return false;
    }
}
