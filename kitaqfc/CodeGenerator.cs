using System;
using System.Collections.Generic;
using System.Linq;

static class CodeGenerator
{
    public static CodegenAnalysisReport LastReport { get; private set; } = new CodegenAnalysisReport();

    // Run a fresh FC backend, publish its report and write the optional aggregate-layout diagnostic text.
    public static IReadOnlyList<Expr> CompileAll(Expr syntaxTree)
    {
        var impl = new CodegenImpl();
        var assembly = impl.Run(syntaxTree);
        LastReport = impl.BuildCodegenAnalysisReport();
        Program.WriteDebugFile("aggregate_layouts.txt", impl.BuildAggregateLayoutText());
        return assembly;
    }

    sealed class CodegenImpl
    {
        // Partition internal CPU RAM into OAM shadow, globals, local zero page, call arguments and default compiler temporaries.
        const int OamRamBase = 0x0200;
        const int OamRamLimitExclusive = 0x0300;
        const int GlobalRamBase = 0x0300;
        const int GlobalRamLimitExclusive = 0x0800;
        const int LocalZpBase = 0x0010;
        const int LocalZpLimitExclusive = 0x00C0;
        const int CallArgBase = 0x00C0;
        const int CallArgLimitExclusive = 0x00D0;
        const int DefaultTempRamBase = 0x00D0;
        const int DefaultTempRamLength = 0x0030;

        // Keep emitted nodes separate from collected function, storage and analysis metadata.
        readonly List<Expr> _assembly = new List<Expr>();
        readonly Dictionary<string, Expr> _functions = new Dictionary<string, Expr>(StringComparer.Ordinal);
        readonly Dictionary<string, CType> _functionReturnTypes = new Dictionary<string, CType>(StringComparer.Ordinal);
        readonly Dictionary<string, FieldInfo[]> _functionParameters = new Dictionary<string, FieldInfo[]>(StringComparer.Ordinal);
        readonly List<string> _orderedFunctionNames = new List<string>();
        readonly Dictionary<string, CFunctionInfo> _functionInfos = new Dictionary<string, CFunctionInfo>(StringComparer.Ordinal);
        readonly Dictionary<string, int> _constants = new Dictionary<string, int>(StringComparer.Ordinal);
        readonly Dictionary<string, StorageSlot> _globals = new Dictionary<string, StorageSlot>(StringComparer.Ordinal);
        readonly Dictionary<string, StorageSlot> _readonlyData = new Dictionary<string, StorageSlot>(StringComparer.Ordinal);
        readonly Dictionary<string, AggregateInfo> _aggregates = new Dictionary<string, AggregateInfo>(StringComparer.Ordinal);
        readonly List<StorageSlot> _orderedReadonlyData = new List<StorageSlot>();
        readonly Dictionary<string, ArithmeticLookupInfo> _arithmeticLookupTables = new Dictionary<string, ArithmeticLookupInfo>(StringComparer.Ordinal);
        readonly Dictionary<string, CallEdgeInfo> _callReportMap = new Dictionary<string, CallEdgeInfo>(StringComparer.Ordinal);
        readonly List<NesActionUseInfo> _nesActionUses = new List<NesActionUseInfo>();
        readonly List<ZpAllocationInfo> _zpAllocations = new List<ZpAllocationInfo>();
        readonly List<StaticFrameSlotInfo> _staticFrameSlots = new List<StaticFrameSlotInfo>();
        readonly List<RamAllocationInfo> _ramAllocations = new List<RamAllocationInfo>();
        readonly List<RamAccessInfo> _ramAccesses = new List<RamAccessInfo>();
        readonly List<InlineDecisionInfo> _inlineDecisions = new List<InlineDecisionInfo>();
        readonly List<LoopLoweringInfo> _loopLowerings = new List<LoopLoweringInfo>();
        readonly List<LtoRemovalInfo> _ltoRemovedFunctions = new List<LtoRemovalInfo>();
        readonly List<BankPlacementInfo> _bankPlacements = new List<BankPlacementInfo>();
        readonly HashSet<string> _nesActionWarningKeys = new HashSet<string>(StringComparer.Ordinal);
        HashSet<string> _reachableFunctions = new HashSet<string>(StringComparer.Ordinal);
        readonly Dictionary<string, BankThunkInfo> _bankThunkInfos = new Dictionary<string, BankThunkInfo>(StringComparer.Ordinal);
        readonly List<string> _orderedThunkNames = new List<string>();
        readonly HashSet<string> _mapperWritingFunctions = new HashSet<string>(StringComparer.Ordinal);
        readonly int[] _tempPool;
        readonly Stack<int> _freeTemps;
        static readonly Dictionary<string, IntrinsicSignature> CompatibilityIntrinsicSignatures = BuildCompatibilityIntrinsicSignatures();
        int _nextOamRam = OamRamBase;
        int _nextGlobalRam = GlobalRamBase;
        int _nextLocalZp = LocalZpBase;
        int _nextDedicatedLocalRam = -1;
        int _labelCounter = 0;
        string _currentFunctionName = "<none>";
        int _currentFunctionBank = 0;
        int _lastPlacementBank = int.MinValue;
        int _lastPlacementFixed = int.MinValue;
        int _runtimeCurrentBankAddress = -1;
        int _runtimeReturnLoAddress = -1;
        int _runtimeReturnHiAddress = -1;
        int _runtimeMapperTempAddress = -1;
        int _runtimeMapperTemp2Address = -1;
        int _runtimeMmc1OuterShadowAddress = -1;
        int _runtimeMmc1InnerShadowAddress = -1;
        int _runtimeMmc1ControlShadowAddress = -1;
        int _runtimeMapperFaultAddress = -1;
        int _runtimePpuCtrlShadowAddress = -1;
        int _runtimePpuMaskShadowAddress = -1;
        int _runtimeScrollXAddress = -1;
        int _runtimeScrollYAddress = -1;
        int _runtimeNmiCounterAddress = -1;
        int _runtimeIntrinsicTmp0Address = -1;
        int _runtimeIntrinsicTmp1Address = -1;
        int _runtimeIntrinsicTmp2Address = -1;
        int _runtimeVramqLenAddress = -1;
        int _runtimeVramqReadyAddress = -1;
        int _runtimeVramqOverflowAddress = -1;
        int _runtimeVramqBufferAddress = -1;
        int _runtimeAxromMirrorShadowAddress = -1;
        int _runtimeFdsLoadListAddress = -1;
        int _runtimeFdsFileHeaderAddress = -1;
        int _runtimeFdsResidentBankAddress = -1;
        int _runtimeRngLoAddress = -1;
        int _runtimeRngHiAddress = -1;
        int _runtimeSretPtrLoAddress = -1;
        int _runtimeSretPtrHiAddress = -1;
        int _structReturnTempCounter = 0;
        const int VramqCapacity = 128;
        const int FdsFileHeaderSize = 17;

        // Build the configured temporary pool so its lowest address is allocated first; initialize an optional dedicated-local cursor.
        public CodegenImpl()
        {
            int tempBase = Program.HasCustomNesTempRamWindow ? Program.NesTempRamBase : DefaultTempRamBase;
            int tempLength = Program.HasCustomNesTempRamWindow ? Program.NesTempRamLength : DefaultTempRamLength;
            _tempPool = Enumerable.Range(tempBase, tempLength).ToArray();
            _freeTemps = new Stack<int>(_tempPool.Reverse());
            if (Program.HasNesLocalRamWindow)
                _nextDedicatedLocalRam = Program.NesLocalRamBase;
        }

        // Carry bank, fixed-bank, ordering and section requests before concrete assembly placement.
        sealed class PlacementInfo
        {
            public int RequestedBank = 1;
            public bool HasFixedBank = false;
            public int PlacementOrder = int.MaxValue;
            public string SectionName = null;
        }

        // Describe a cross-bank call stub and its return width, including whether it needs an FDS overlay load.
        sealed class BankThunkInfo
        {
            public string Name;
            public string TargetName;
            public int TargetBank;
            public bool UseFdsOverlay;
            public int ReturnSize;
        }

        // Represent named RAM storage, constants or readonly bytes together with source and ROM-placement metadata.
        sealed class StorageSlot
        {
            public string Name;
            public CType Type;
            public int Address;
            public int Size;
            public bool IsConstant;
            public int ConstantValue;
            public bool IsReadonlyData;
            public byte[] ReadonlyBytes;
            public FilePosition Source;
            public int RequestedBank = 1;
            public bool HasFixedBank = false;
            public int PlacementOrder = int.MaxValue;
            public string SectionName = null;
        }

        // Pair the enclosing loop's continue and break destinations.
        sealed class LoopLabels
        {
            public readonly string ContinueLabel;
            public readonly string BreakLabel;

            public LoopLabels(string continueLabel, string breakLabel)
            {
                ContinueLabel = continueLabel;
                BreakLabel = breakLabel;
            }
        }

        // Associate the low- and high-byte labels of a generated arithmetic lookup table.
        sealed class ArithmeticLookupInfo
        {
            public string LowLabel;
            public string HighLabel;
        }

        // Describe an intrinsic's result type and argument byte widths; emission is implemented separately.
        sealed class IntrinsicSignature
        {
            public readonly CType ReturnType;
            public readonly int[] ParameterSizes;

            public IntrinsicSignature(CType returnType, params int[] parameterSizes)
            {
                ReturnType = returnType;
                ParameterSizes = parameterSizes ?? Array.Empty<int>();
            }
        }

        // Keep a function's ABI, local bindings, nested-loop destinations and aggregate-return pointer together.
        sealed class FunctionContext
        {
            public readonly string Name;
            public readonly int Bank;
            public readonly string ReturnMnemonic;
            public readonly CType ReturnType;
            public readonly Dictionary<string, StorageSlot> Locals = new Dictionary<string, StorageSlot>(StringComparer.Ordinal);
            public readonly Stack<LoopLabels> LoopStack = new Stack<LoopLabels>();
            public StorageSlot StructReturnPointerSlot;

            public FunctionContext(string name, int bank, string returnMnemonic, CType returnType)
            {
                Name = name;
                Bank = bank;
                ReturnMnemonic = returnMnemonic;
                ReturnType = returnType;
            }
        }

        // Register case-sensitive intrinsic names with their return types and argument widths in bytes.
        static Dictionary<string, IntrinsicSignature> BuildCompatibilityIntrinsicSignatures()
        {
            var map = new Dictionary<string, IntrinsicSignature>(StringComparer.Ordinal);

            // Insert the signature, replacing any existing entry with the same exact name.
            void Add(string name, CType returnType, params int[] parameterSizes)
            {
                map[name] = new IntrinsicSignature(returnType, parameterSizes);
            }

            // KITAQFC public intrinsic set: FC/6502 meaningful names only.
            // CGB/GB compatibility stubs/no-ops are intentionally not registered.
            Add("__memcpy", CType.Void, 2, 2, 2);
            Add("__memset", CType.Void, 2, 1, 2);
            Add("__memcpy_small", CType.Void, 2, 2, 1);
            Add("__memset_small", CType.Void, 2, 1, 1);
            Add("__copy16", CType.Void, 2, 2);
            Add("__copy32", CType.Void, 2, 2);
            Add("__xy_in_rect", CType.UInt8, 1, 1, 1, 1, 1, 1);
            Add("__manhattan", CType.UInt8, 1, 1, 1, 1);
            Add("__map_index", CType.UInt16, 1, 1, 1);
            Add("__bit_test", CType.UInt8, 2, 2);
            Add("__bit_set", CType.Void, 2, 2);
            Add("__bit_clear", CType.Void, 2, 2);
            Add("__bit_toggle", CType.Void, 2, 2);
            Add("__mul8x8_hi", CType.UInt8, 1, 1);
            Add("__mul16x8", CType.UInt16, 2, 1);
            Add("__mac16", CType.UInt16, 2, 2, 1);
            Add("__dot3_q8_8", CType.UInt16, 2, 2, 2, 1, 1, 1);
            Add("__dot2_q8_8", CType.UInt16, 2, 2, 1, 1);
            Add("__smul16x8", CType.UInt16, 2, 1);
            Add("__smul16x8_q1_7", CType.UInt16, 2, 1);
            Add("__smac16", CType.UInt16, 2, 2, 1);
            Add("__smac16_q1_7", CType.UInt16, 2, 2, 1);
            Add("__sdot3_q8_8", CType.UInt16, 2, 2, 2, 1, 1, 1);
            Add("__sdot3_q1_7", CType.UInt16, 2, 2, 2, 1, 1, 1);
            Add("__sdot2_q8_8", CType.UInt16, 2, 2, 1, 1);
            Add("__sdot2_q1_7", CType.UInt16, 2, 2, 1, 1);
            Add("__rng_seed", CType.Void, 2);
            Add("__rng8", CType.UInt8);

            Add("__farcall", CType.UInt8, 1, 2);
            Add("__far_memcpy", CType.Void, 2, 1, 2, 2);
            Add("__farmemcpy", CType.Void, 2, 1, 2, 2);
            Add("__farpeek8", CType.UInt8, 1, 2);
            Add("__farpeek16", CType.UInt16, 1, 2);

            // FC/NES-specific intrinsics.
            Add("__ppu_on", CType.Void);
            Add("__ppu_off", CType.Void);
            Add("__ppu_mask_set", CType.Void, 1);
            Add("__ppu_ctrl_set", CType.Void, 1);
            Add("__ppu_addr", CType.Void, 2);
            Add("__ppu_data", CType.Void, 1);
            Add("__ppu_read_status", CType.UInt8);
            Add("__vram_write", CType.Void, 2, 2, 1);
            Add("__vram_fill", CType.Void, 2, 1, 1);
            Add("__nametable_put", CType.Void, 1, 1, 1);
            Add("__nametable_put_nt", CType.Void, 1, 1, 1, 1);
            Add("__nametable_rect", CType.Void, 1, 1, 1, 1, 1);
            Add("__nametable_rect_nt", CType.Void, 1, 1, 1, 1, 1, 1);
            Add("__attr_set", CType.Void, 1, 1, 1);
            Add("__attr_set_nt", CType.Void, 1, 1, 1, 1);
            Add("__palette_bg_load", CType.Void, 2);
            Add("__palette_sp_load", CType.Void, 2);
            Add("__vramq_clear", CType.Void);
            Add("__vramq_put", CType.Void, 2, 1);
            Add("__vramq_copy", CType.Void, 2, 2, 1);
            Add("__vramq_fill", CType.Void, 2, 1, 1);
            Add("__vramq_commit", CType.Void);
            Add("__vramq_exec", CType.Void);
            Add("__vramq_len", CType.UInt8);
            Add("__vramq_overflow", CType.UInt8);
            Add("__vramq_clear_overflow", CType.Void);
            Add("__vramq_capacity", CType.UInt8);
            Add("__nmi_wait", CType.Void);
            Add("__nmi_ready", CType.UInt8);
            Add("__oam_dma", CType.Void);
            Add("__oam_dma_page", CType.Void, 1);
            Add("__oam_clear", CType.Void);
            Add("__sprite_set", CType.Void, 1, 1, 1, 1, 1);
            Add("__sprite_move", CType.Void, 1, 1, 1);
            Add("__sprite_tile", CType.Void, 1, 1);
            Add("__sprite_attr", CType.Void, 1, 1);
            Add("__sprite_hide", CType.Void, 1);
            Add("__metasprite_draw", CType.UInt8, 1, 1, 1, 2);
            Add("__pad_read1", CType.UInt8);
            Add("__pad_read2", CType.UInt8);
            Add("__pad_read1_safe", CType.UInt8);
            Add("__pad_read2_safe", CType.UInt8);
            Add("__pad_buttons", CType.UInt8, 1);
            Add("__pad_dirs", CType.UInt8, 1);

            // FC/Famicom expansion input devices.
            Add("__pad_read1_d1", CType.UInt8);
            Add("__pad_read2_d1", CType.UInt8);
            Add("__exp_pad_read1", CType.UInt8);
            Add("__exp_pad_read2", CType.UInt8);
            Add("__joypad2p_voice", CType.UInt8);
            Add("__mic_read2p", CType.UInt8);
            Add("__zapper_raw1", CType.UInt8);
            Add("__zapper_raw2", CType.UInt8);
            Add("__zapper_trigger1", CType.UInt8);
            Add("__zapper_trigger2", CType.UInt8);
            Add("__zapper_light1", CType.UInt8);
            Add("__zapper_light2", CType.UInt8);
            Add("__zapper_trigger", CType.UInt8);
            Add("__zapper_light", CType.UInt8);
            Add("__fkb_detect", CType.UInt8);
            Add("__fkb_scan", CType.Void, 2);
            Add("__fkb_read_row_col", CType.UInt8, 1, 1);
            Add("__rob_flash", CType.Void, 1);
            Add("__rob_pulse", CType.Void, 1, 1);
            Add("__rob_send_byte", CType.Void, 1);
            Add("__serial_tx_bit", CType.Void, 1);
            Add("__serial_rx_bit", CType.UInt8);
            Add("__midi_out_byte", CType.Void, 1);
            Add("__midi_in_byte", CType.UInt8);
            Add("__midi_note_on", CType.Void, 1, 1, 1);
            Add("__midi_note_off", CType.Void, 1, 1, 1);
            Add("__midi_control_change", CType.Void, 1, 1, 1);
            Add("__midi_program_change", CType.Void, 1, 1);
            Add("__midi_clock", CType.Void);
            Add("__midi_start", CType.Void);
            Add("__midi_continue", CType.Void);
            Add("__midi_stop", CType.Void);
            Add("__mapper_id", CType.UInt8);
            Add("__prg_bank_set", CType.Void, 1);
            Add("__chr_bank_set", CType.Void, 1);
            Add("__chr_bank_set0", CType.Void, 1);
            Add("__chr_bank_set1", CType.Void, 1);
            Add("__mirroring_set", CType.Void, 1);
            Add("__irq_scanline_set", CType.Void, 1);
            Add("__mapper_irq_set", CType.Void, 1);
            Add("__mapper_irq_enable", CType.Void);
            Add("__mapper_irq_disable", CType.Void);
            Add("__mapper_irq_ack", CType.Void);
            Add("__scroll_set", CType.Void, 1, 1);
            Add("__scroll_x_set", CType.Void, 1);
            Add("__scroll_y_set", CType.Void, 1);
            Add("__scroll_latch_reset", CType.Void);
            Add("__sprite0_wait_hit", CType.Void);
            Add("__split_scroll_sprite0", CType.Void, 1, 1);
            Add("__irq_disable", CType.Void);
            Add("__irq_enable", CType.Void);
            Add("__irq_save", CType.UInt8);
            Add("__irq_restore", CType.Void, 1);
            Add("__nmi_enable", CType.Void);
            Add("__nmi_disable", CType.Void);

            // FDS intrinsic surface. Disk file I/O is backend-gated; sound/status primitives emit concrete FDS register code on --mapper=fds.
            Add("__fds_available", CType.UInt8);
            Add("__fds_disk_ready", CType.UInt8);
            Add("__fds_side", CType.UInt8);
            Add("__fds_error", CType.UInt8);
            Add("__fds_wait_ready", CType.Void);
            Add("__fds_wait_insert", CType.Void);
            Add("__fds_load_file", CType.UInt8, 1, 2);
            Add("__fds_save_file", CType.UInt8, 1, 2, 2);
            Add("__fds_load_overlay", CType.UInt8, 1);
            Add("__fds_load_bank", CType.UInt8, 1);
            Add("__fds_require_bank", CType.UInt8, 1);
            Add("__fds_current_bank", CType.UInt8);
            Add("__fds_is_bank_resident", CType.UInt8, 1);
            Add("__fds_overlay_function_count", CType.UInt8);
            Add("__fds_overlay_farcall", CType.UInt8, 1, 2);
            Add("__fds_farcall", CType.UInt8, 1, 2);
            Add("__fds_file_exists", CType.UInt8, 1);
            Add("__fds_file_size", CType.UInt16, 1);
            Add("__fds_sound_enable", CType.Void);
            Add("__fds_wave_load", CType.Void, 2);
            Add("__fds_mod_load", CType.Void, 2);
            Add("__fds_freq_set", CType.Void, 2);
            Add("__fds_volume_set", CType.Void, 1);
            Add("__fds_env_set", CType.Void, 1, 1, 1);

            return map;
        }

        // Look up the intrinsic spelling exactly in the signature catalog.
        static bool TryGetCompatibilityIntrinsicSignature(string funcName, out IntrinsicSignature signature)
        {
            return CompatibilityIntrinsicSignatures.TryGetValue(funcName, out signature);
        }

        // Create unnamed-ABI parameter descriptors as unsigned bytes or words; field offsets remain zero for later layout.
        static FieldInfo[] BuildSyntheticParameters(IntrinsicSignature signature)
        {
            if (signature == null || signature.ParameterSizes == null || signature.ParameterSizes.Length == 0)
                return Array.Empty<FieldInfo>();

            var fields = new FieldInfo[signature.ParameterSizes.Length];
            for (int i = 0; i < signature.ParameterSizes.Length; i++)
            {
                CType type = signature.ParameterSizes[i] <= 1 ? CType.UInt8 : CType.UInt16;
                fields[i] = new FieldInfo(type, "__arg" + i, 0);
            }
            return fields;
        }

        // Reserve runtime RAM, collect declarations and validate configured windows before selecting reachable functions.
        // Emit entry stubs, user code, helper routines and readonly bytes in that order.
        public IReadOnlyList<Expr> Run(Expr syntaxTree)
        {
            Expr[] items;
            if (!syntaxTree.Match(Tag.Sequence, out items))
            {
                items = new[] { syntaxTree };
            }

            ReserveRuntimeState();
            CollectTopLevel(items);
            ValidateConfiguredRamWindows();
            if (Program.ErrorCount > 0) return _assembly;

            _reachableFunctions = Program.EnableLibraryLtoLite ? ComputeReachableFunctions() : new HashSet<string>(_orderedFunctionNames, StringComparer.Ordinal);
            EmitAutoEntryStubs();
            EmitUserFunctions();
            ValidateSuromInterruptSafety();
            EmitStaticFrameSafetyDiagnostics();
            EmitBankHelpers();
            EmitIntrinsicHelpers();
            EmitReadonlyData();
            return _assembly;
        }

        // Process declarations in input order, registering function ABI/layout, folded constants, aggregates and storage.
        // Extern/opaque/empty/assertion nodes do not emit storage in this pass.
        void CollectTopLevel(Expr[] items)
        {
            foreach (var item in items ?? Array.Empty<Expr>())
            {
                PlacementInfo placement = ExtractPlacement(item, out Expr current);

                Expr[] nested;
                if (current != null && current.MatchAny(Tag.Sequence, out nested))
                {
                    CollectTopLevel(nested);
                    continue;
                }

                CType retType;
                string name;
                FieldInfo[] fields;
                int mustCheck;
                Expr body;
                MemoryRegion region;
                CType type;
                Expr valueExpr;
                Expr rangeExpr;

                if (current.Match<CType, string, FieldInfo[], int, Expr>(Tag.Function, out retType, out name, out fields, out mustCheck, out body) ||
                    current.Match<CType, string, FieldInfo[], int, Expr>(Tag.InlineFunction, out retType, out name, out fields, out mustCheck, out body))
                {
                    if (_functions.ContainsKey(name))
                    {
                        Program.Error("error KQ0000: duplicate function for --target=nes: {0}", name);
                        continue;
                    }

                    if (!ValidateParameterList(name, fields))
                        continue;

                    _functions[name] = current;
                    _functionReturnTypes[name] = retType;
                    _functionParameters[name] = fields ?? Array.Empty<FieldInfo>();
                    _functionInfos[name] = CreateFunctionInfo(name, current.Source, retType, fields, mustCheck, isPrototype: false, isInline: current.Tag == Tag.InlineFunction, placement);
                    _orderedFunctionNames.Add(name);
                    continue;
                }

                // A prototype updates the signature and placement record even when a definition was collected earlier.
                if (current.Match(Tag.FunctionDecl, out retType, out name, out fields, out mustCheck))
                {
                    if (!ValidateParameterList(name, fields))
                        continue;
                    _functionReturnTypes[name] = retType;
                    _functionParameters[name] = fields ?? Array.Empty<FieldInfo>();
                    _functionInfos[name] = CreateFunctionInfo(name, current.Source, retType, fields, mustCheck, isPrototype: true, isInline: false, placement);
                    continue;
                }

                if (current.Match(Tag.Constant, out type, out name, out valueExpr))
                {
                    int value;
                    if (!TryEvaluateConstant(valueExpr, out value))
                    {
                        Program.Error("error KQ0000: --target=nes phase 6 requires top-level constants to be integer-foldable: {0}", name);
                        continue;
                    }
                    if (_constants.ContainsKey(name) || _globals.ContainsKey(name) || _readonlyData.ContainsKey(name))
                    {
                        Program.Error("error KQ0000: duplicate top-level symbol for --target=nes: {0}", name);
                        continue;
                    }
                    // Store the folded value as a masked word; this table does not retain the declared constant type.
                    _constants[name] = value & 0xFFFF;
                    continue;
                }

                int packed;
                int forcedAlign;
                if (current.Match(Tag.Struct, out name, out fields, out packed, out forcedAlign) ||
                    (packed = 0) == 0 && (forcedAlign = 0) == 0 && current.Match(Tag.Struct, out name, out fields))
                {
                    RegisterAggregate(name, AggregateLayout.Struct, fields, packed != 0, forcedAlign);
                    continue;
                }

                if (current.Match(Tag.Union, out name, out fields, out packed, out forcedAlign) ||
                    (packed = 0) == 0 && (forcedAlign = 0) == 0 && current.Match(Tag.Union, out name, out fields))
                {
                    RegisterAggregate(name, AggregateLayout.Union, fields, packed != 0, forcedAlign);
                    continue;
                }

                int[] roInts;
                Expr[] roExprs;
                if (current.Match(Tag.ReadonlyData, out type, out name, out roInts))
                {
                    RegisterReadonlyData(current.Source, type, name, roInts, placement);
                    continue;
                }
                if (current.Match(Tag.ReadonlyData, out type, out name, out roExprs))
                {
                    RegisterReadonlyData(current.Source, type, name, EvaluateReadonlyExprs(name, roExprs), placement);
                    continue;
                }

                if (current.Match(Tag.Variable, out region, out type, out name, out rangeExpr) || current.Match(Tag.Variable, out region, out type, out name))
                {
                    DeclareGlobal(current.Source, region, type, name);
                    continue;
                }

                if (current.Match(Tag.ExternVariable, out region, out type, out name, out rangeExpr) || current.Match(Tag.ExternVariable, out region, out type, out name))
                {
                    continue;
                }

                if (current.MatchAny(Tag.Sequence, out nested))
                {
                    CollectTopLevel(nested);
                    continue;
                }

                if (current.Match(Tag.Empty) || current.Match(Tag.StaticAssert) || current.Match(Tag.OpaqueStruct) || current.Match(Tag.OpaqueUnion))
                    continue;

                Program.Error("error KQ0000: --target=nes phase 6 does not support top-level tag: {0}", current.Tag);
            }
        }

        // Traverse references from entry/interrupt/runtime roots, profiled functions and fixed-bank functions.
        // Record functions outside that reachable set for the optional library-removal report.
        HashSet<string> ComputeReachableFunctions()
        {
            var reachable = new HashSet<string>(StringComparer.Ordinal);
            var work = new Queue<string>();
            // Queue a root only when a body exists, and visit each function name at most once.
            void AddRoot(string name)
            {
                if (string.IsNullOrEmpty(name)) return;
                if (!_functions.ContainsKey(name)) return;
                if (reachable.Add(name)) work.Enqueue(name);
            }

            AddRoot("main");
            AddRoot("__nes_reset");
            AddRoot("__nes_nmi");
            AddRoot("__nes_irq");
            foreach (var name in _orderedFunctionNames)
            {
                if (name.StartsWith("__nes_", StringComparison.Ordinal) || name.StartsWith("__kq_", StringComparison.Ordinal)) AddRoot(name);
                if (Program.GetKurosakiHotness(name) > 0) AddRoot(name);
                CFunctionInfo info;
                if (_functionInfos.TryGetValue(name, out info) && info != null && info.HasFixedBank) AddRoot(name);
            }

            while (work.Count > 0)
            {
                string fn = work.Dequeue();
                Expr decl;
                if (!_functions.TryGetValue(fn, out decl) || decl == null) continue;
                foreach (string callee in EnumerateReferencedFunctions(decl))
                {
                    if (_functions.ContainsKey(callee) && reachable.Add(callee)) work.Enqueue(callee);
                }
            }

            foreach (string name in _orderedFunctionNames)
            {
                if (!reachable.Contains(name))
                {
                    _ltoRemovedFunctions.Add(new LtoRemovalInfo { Name = name, Reason = "not reachable from main/interrupt roots or function-address references" });
                }
            }
            return reachable;
        }

        // Find known symbolic assembly operands, named calls and explicit function-address references recursively.
        // Call arguments are scanned, but a non-name call-target expression is not traversed by the call branch.
        IEnumerable<string> EnumerateReferencedFunctions(Expr expr)
        {
            if (expr == null) yield break;
            string asmMnemonic;
            AsmOperand asmOperand;
            if (expr.Match(Tag.Asm, out asmMnemonic, out asmOperand) &&
                asmOperand != null && asmOperand.Base.HasValue &&
                _functions.ContainsKey(asmOperand.Base.Value))
            {
                // Raw JSR/JMP and function-address operands are references too.
                // Keep their target before LTO emits/removes function bodies.
                yield return asmOperand.Base.Value;
            }
            Expr funcExpr;
            Expr[] args;
            string name;
            if (expr.MatchAny(Tag.Call, out funcExpr, out args))
            {
                if (funcExpr.Match(Tag.Name, out name))
                    yield return name;
                foreach (var a in args ?? Array.Empty<Expr>())
                {
                    if (a != null && a.Match(Tag.Name, out name) && _functions.ContainsKey(name))
                        yield return name;
                    foreach (var nested in EnumerateReferencedFunctions(a)) yield return nested;
                }
                yield break;
            }
            Expr sub;
            if (expr.Match(Tag.AddressOf, out sub) && sub != null && sub.Match(Tag.Name, out name) && _functions.ContainsKey(name))
                yield return name;
            foreach (object arg in expr.GetArgs().Skip(1))
            {
                if (arg is Expr e)
                {
                    foreach (var nested in EnumerateReferencedFunctions(e)) yield return nested;
                }
                else if (arg is Expr[] arr)
                {
                    foreach (var e2 in arr ?? Array.Empty<Expr>())
                        foreach (var nested in EnumerateReferencedFunctions(e2)) yield return nested;
                }
            }
        }

        // Peel up to sixteen supported bank/order/section/static/unsafe wrappers, then clamp a negative requested bank to zero.
        PlacementInfo ExtractPlacement(Expr item, out Expr unwrapped)
        {
            var placement = new PlacementInfo();
            unwrapped = item;

            for (int guard = 0; unwrapped != null && guard < 16; guard++)
            {
                Expr inner;
                int number;
                string sectionName;

                if (unwrapped.Match(Tag.Static, out inner) || unwrapped.Match(Tag.Unsafe, out inner))
                {
                    unwrapped = inner;
                    continue;
                }
                if (unwrapped.Match(Tag.Bank, out number, out inner))
                {
                    placement.RequestedBank = number;
                    unwrapped = inner;
                    continue;
                }
                if (unwrapped.Match(Tag.FixedBank, out number, out inner))
                {
                    placement.RequestedBank = number;
                    placement.HasFixedBank = true;
                    unwrapped = inner;
                    continue;
                }
                if (unwrapped.Match(Tag.FixedOrder, out number, out inner))
                {
                    placement.PlacementOrder = number;
                    unwrapped = inner;
                    continue;
                }
                if (unwrapped.Match(Tag.DeclSection, out sectionName, out inner))
                {
                    placement.SectionName = sectionName;
                    unwrapped = inner;
                    continue;
                }
                break;
            }

            if (placement.RequestedBank < 0)
                placement.RequestedBank = 0;
            return placement;
        }

        // Apply configured bank selection and optional mapper heuristics, then force reset/NMI/IRQ into common bank zero.
        // Derive fixed-placement and fast-call metadata from the resulting bank and ABI eligibility.
        CFunctionInfo CreateFunctionInfo(string functionName, FilePosition source, CType retType, FieldInfo[] fields, int mustCheck, bool isPrototype, bool isInline, PlacementInfo placement)
        {
            fields = fields ?? Array.Empty<FieldInfo>();
            int bank = (placement == null) ? 1 : placement.RequestedBank;
            bool userFixedBank = placement != null && placement.HasFixedBank;
            bool forceCommonBank = string.Equals(functionName, "__nes_reset", StringComparison.Ordinal) ||
                string.Equals(functionName, "__nes_nmi", StringComparison.Ordinal) ||
                string.Equals(functionName, "__nes_irq", StringComparison.Ordinal);

            bank = ResolveFunctionBankOverride(functionName, bank);
            if (!userFixedBank && Program.EnableMapperAwareBankPlacement)
                bank = ApplyMapperAwareBankPlacement(functionName, bank, source);
            if (forceCommonBank) bank = 0;

            bool fastCall = Program.AbiFastCall && IsFastCallEligible(retType, fields);

            return new CFunctionInfo
            {
                Parameters = fields,
                ReturnType = retType,
                MustCheck = (mustCheck != 0),
                RomBank = bank,
                HasFixedBank = forceCommonBank || userFixedBank || (Program.EnableMapperAwareBankPlacement && bank == 0),
                PlacementOrder = placement == null ? int.MaxValue : placement.PlacementOrder,
                HasFixedOrder = placement != null && placement.PlacementOrder != int.MaxValue,
                IsPrototype = isPrototype,
                IsInline = isInline,
                IsFastCall = fastCall,
            };
        }

        // Prefer common code for sufficiently hot or loop/update/draw/scroll-named functions on banked profiles.
        // This selection uses names/profile counts, not a bank-capacity estimate.
        int ApplyMapperAwareBankPlacement(string functionName, int fallbackBank, FilePosition source)
        {
            if (fallbackBank == 0) return 0;
            int hotness = Program.GetKurosakiHotness(functionName);
            bool banked = Program.NesMapperProfile.SupportsPrgBanking || Program.NesMapperProfile.UsesDuplicatedCommonBank || Program.NesMapperProfile.HasFds;
            if (!banked) return fallbackBank;

            string lower = (functionName ?? "").ToLowerInvariant();
            bool commonCandidate = hotness >= 1000 || lower == "main" || lower.Contains("loop") || lower.Contains("update") || lower.Contains("draw") || lower.Contains("scroll");
            if (commonCandidate)
            {
                _bankPlacements.Add(new BankPlacementInfo { Name = functionName, Bank = 0, Hotness = hotness, Reason = hotness >= 1000 ? "KUROSAKI hot function pinned to common bank" : "game-loop style function pinned to common bank" });
                return 0;
            }
            return fallbackBank;
        }

        // Permit only nonaggregate signatures with at most two parameters totaling two bytes and a zero-, one- or two-byte result.
        bool IsFastCallEligible(CType retType, FieldInfo[] fields)
        {
            fields = fields ?? Array.Empty<FieldInfo>();
            if (IsAggregateType(retType)) return false;
            int total = 0;
            foreach (var f in fields)
            {
                if (IsAggregateType(f.Type)) return false;
                int size = GetStorageSize(f.Type);
                if (size != 1 && size != 2) return false;
                total += size;
            }
            if (fields.Length > 2 || total > 2) return false;
            int retSize = TypeStorageSize(retType);
            return retSize == 0 || retSize == 1 || retSize == 2;
        }

        // Prefer an explicit function-bank override when one exists, otherwise keep the supplied fallback.
        int ResolveFunctionBankOverride(string functionName, int fallbackBank)
        {
            if (!string.IsNullOrEmpty(functionName) && Program.TryGetFunctionBankOverride(functionName, out int overrideBank))
                return overrideBank;
            return fallbackBank;
        }

        // Reserve the shared ABI areas and runtime globals/buffers before user storage, including currently unused feature state.
        void ReserveRuntimeState()
        {
            RecordRamAllocation("__kq_call_arg_area", "abi_call_args", CallArgBase, CallArgLimitExclusive - CallArgBase, true);
            RecordRamAllocation("__kq_temp_pool", "compiler_temp_pool", _tempPool[0], _tempPool.Length, true);
            _runtimeCurrentBankAddress = ReserveInternalFastGlobal("__kq_prg_bank_current", CType.UInt8, "mapper current bank hot state");
            _runtimeReturnLoAddress = ReserveInternalFastGlobal("__kq_thunk_ret_lo", CType.UInt8, "farcall return hot state");
            _runtimeReturnHiAddress = ReserveInternalFastGlobal("__kq_thunk_ret_hi", CType.UInt8, "farcall return hot state");
            _runtimeMapperTempAddress = ReserveInternalFastGlobal("__kq_mapper_tmp", CType.UInt8, "mapper switch temporary");
            _runtimeMapperTemp2Address = ReserveInternalFastGlobal("__kq_mapper_tmp2", CType.UInt8, "mapper serial temporary");
            _runtimeMmc1OuterShadowAddress = ReserveInternalFastGlobal("__kq_mmc1_outer_shadow", CType.UInt8, "MMC1 outer PRG bank shadow");
            _runtimeMmc1InnerShadowAddress = ReserveInternalFastGlobal("__kq_mmc1_inner_shadow", CType.UInt8, "MMC1 inner PRG bank shadow");
            _runtimeMmc1ControlShadowAddress = ReserveInternalFastGlobal("__kq_mmc1_control_shadow", CType.UInt8, "MMC1 control register shadow");
            _runtimeMapperFaultAddress = ReserveInternalFastGlobal("__kq_mapper_fault", CType.UInt8, "mapper fail-safe status");
            _runtimePpuCtrlShadowAddress = ReserveInternalFastGlobal("__kq_ppuctrl_shadow", CType.UInt8, "PPUCTRL shadow hot state");
            _runtimePpuMaskShadowAddress = ReserveInternalFastGlobal("__kq_ppumask_shadow", CType.UInt8, "PPUMASK shadow hot state");
            _runtimeScrollXAddress = ReserveInternalFastGlobal("__kq_scroll_x", CType.UInt8, "scroll hot state");
            _runtimeScrollYAddress = ReserveInternalFastGlobal("__kq_scroll_y", CType.UInt8, "scroll hot state");
            _runtimeNmiCounterAddress = ReserveInternalFastGlobal("__kq_nmi_counter", CType.UInt8, "NMI counter hot state");
            _runtimeIntrinsicTmp0Address = ReserveInternalFastGlobal("__kq_intr_tmp0", CType.UInt8, "intrinsic temporary virtual register");
            _runtimeIntrinsicTmp1Address = ReserveInternalFastGlobal("__kq_intr_tmp1", CType.UInt8, "intrinsic temporary virtual register");
            _runtimeIntrinsicTmp2Address = ReserveInternalFastGlobal("__kq_intr_tmp2", CType.UInt8, "intrinsic temporary virtual register");
            _runtimeVramqLenAddress = ReserveInternalFastGlobal("__kq_vramq_len", CType.UInt8, "VRAM queue length hot state");
            _runtimeVramqReadyAddress = ReserveInternalFastGlobal("__kq_vramq_ready", CType.UInt8, "VRAM queue ready flag hot state");
            _runtimeVramqOverflowAddress = ReserveInternalFastGlobal("__kq_vramq_overflow", CType.UInt8, "VRAM queue overflow flag hot state");
            _runtimeVramqBufferAddress = ReserveInternalGlobalBytes("__kq_vramq_buf", VramqCapacity);
            _runtimeAxromMirrorShadowAddress = ReserveInternalFastGlobal("__kq_axrom_mirror_shadow", CType.UInt8, "AxROM mirror shadow hot state");
            _runtimeFdsLoadListAddress = ReserveInternalGlobalBytes("__kq_fds_load_list", 2);
            _runtimeFdsFileHeaderAddress = ReserveInternalGlobalBytes("__kq_fds_file_header", FdsFileHeaderSize);
            _runtimeFdsResidentBankAddress = ReserveInternalFastGlobal("__kq_fds_resident_bank", CType.UInt8, "FDS overlay resident bank hot state");
            _runtimeRngLoAddress = ReserveInternalFastGlobal("__kq_rng_lo", CType.UInt8, "RNG state hot byte");
            _runtimeRngHiAddress = ReserveInternalFastGlobal("__kq_rng_hi", CType.UInt8, "RNG state hot byte");
            _runtimeSretPtrLoAddress = ReserveInternalFastGlobal("__kq_sret_ptr_lo", CType.UInt8, "struct return destination pointer");
            _runtimeSretPtrHiAddress = ReserveInternalFastGlobal("__kq_sret_ptr_hi", CType.UInt8, "struct return destination pointer");
        }

        // Allocate hot runtime storage from local zero page when enabled and space remains; otherwise use ordinary global RAM.
        int ReserveInternalFastGlobal(string name, CType type, string reason)
        {
            int size = GetStorageSize(type);
            if (Program.EnableWholeProgramZpAllocator && size > 0 && _nextLocalZp + size <= LocalZpLimitExclusive)
            {
                int address = _nextLocalZp;
                _nextLocalZp += size;
                _globals[name] = new StorageSlot
                {
                    Name = name,
                    Type = type,
                    Address = address,
                    Size = size,
                    Source = FilePosition.Unknown,
                };
                _zpAllocations.Add(new ZpAllocationInfo { Name = name, Kind = "runtime", Address = address, Size = size, Reason = reason ?? "runtime hot state" });
                RecordRamAllocation(name, "runtime", address, size, false);
                return address;
            }
            return ReserveInternalGlobal(name, type);
        }

        // Reserve a typed runtime slot from $0300-$07FF; on exhaustion report an error and return a placeholder base without advancing.
        int ReserveInternalGlobal(string name, CType type)
        {
            int size = GetStorageSize(type);
            if (_nextGlobalRam + size > GlobalRamLimitExclusive)
            {
                Program.Error("error KQ0000: --target=nes phase 6 ran out of internal RAM while reserving runtime state '{0}'.", name);
                return GlobalRamBase;
            }

            int address = _nextGlobalRam;
            _nextGlobalRam += size;
            _globals[name] = new StorageSlot
            {
                Name = name,
                Type = type,
                Address = address,
                Size = size,
                Source = FilePosition.Unknown,
            };
            RecordRamAllocation(name, "runtime", address, size, false);
            return address;
        }

        // Reserve at least one byte for a runtime buffer, recording its explicit byte size independently of its UInt8 type.
        int ReserveInternalGlobalBytes(string name, int size)
        {
            if (size <= 0) size = 1;
            if (_nextGlobalRam + size > GlobalRamLimitExclusive)
            {
                Program.Error("error KQ0000: --target=nes phase 6 ran out of internal RAM while reserving runtime buffer '{0}'.", name);
                return GlobalRamBase;
            }
            int address = _nextGlobalRam;
            _nextGlobalRam += size;
            _globals[name] = new StorageSlot
            {
                Name = name,
                Type = CType.UInt8,
                Address = address,
                Size = size,
                Source = FilePosition.Unknown,
            };
            RecordRamAllocation(name, "runtime_buffer", address, size, false);
            return address;
        }

        // Append a positive-size allocation and its memory-region classification to the report.
        void RecordRamAllocation(string name, string kind, int address, int size, bool conservative)
        {
            if (size <= 0) return;
            _ramAllocations.Add(new RamAllocationInfo
            {
                Name = name ?? "",
                Kind = kind ?? "storage",
                Address = address,
                Size = size,
                Region = ClassifyCpuMemoryRegion(address, size),
                Conservative = conservative,
            });
        }

        // Classify a span only when its entire half-open interval fits one listed CPU region; otherwise report other.
        static string ClassifyCpuMemoryRegion(int address, int size)
        {
            int endExclusive = address + Math.Max(0, size);
            if (address >= 0x0000 && endExclusive <= 0x0100) return "cpu_ram_zero_page";
            if (address >= 0x0100 && endExclusive <= 0x0200) return "cpu_ram_stack";
            if (address >= 0x0200 && endExclusive <= 0x0300) return "cpu_ram_oam_shadow";
            if (address >= 0x0300 && endExclusive <= 0x0800) return "cpu_ram";
            if (address >= 0x6000 && endExclusive <= 0x8000) return "prg_ram";
            if (address >= 0x2000 && endExclusive <= 0x6000) return "memory_mapped_io";
            return "other";
        }

        // Record selected numeric memory operands inside emitted functions; symbolic operands are skipped.
        // Indirect operands describe the pointer location, while indexed spans use allocation metadata or a 256-byte fallback.
        void RecordRamAccess(string mnemonic, AsmOperand operand)
        {
            if (string.IsNullOrEmpty(_currentFunctionName) || _currentFunctionName == "<none>")
                return;
            if (operand == null || operand.Base.HasValue)
                return;

            string operation;
            switch ((mnemonic ?? "").ToUpperInvariant())
            {
                case "LDA":
                case "LDX":
                case "LDY":
                case "ADC":
                case "SBC":
                case "AND":
                case "ORA":
                case "EOR":
                case "CMP":
                case "CPX":
                case "CPY":
                case "BIT":
                    operation = "read";
                    break;
                case "STA":
                case "STX":
                case "STY":
                    operation = "write";
                    break;
                case "INC":
                case "DEC":
                case "ASL":
                case "LSR":
                case "ROL":
                case "ROR":
                    operation = "read_write";
                    break;
                default:
                    return;
            }

            bool memoryMode =
                operand.Mode == AddressMode.Absolute ||
                operand.Mode == AddressMode.AbsoluteX ||
                operand.Mode == AddressMode.AbsoluteY ||
                operand.Mode == AddressMode.IndirectX ||
                operand.Mode == AddressMode.IndirectY ||
                operand.Mode == AddressMode.HighMem ||
                operand.Mode == AddressMode.HighMemX ||
                operand.Mode == AddressMode.HighMemY;
            if (!memoryMode)
                return;

            int address = operand.Offset & 0xFFFF;
            // CPU reads at $8000-$FFFF fetch immutable PRG-ROM data. They are
            // relevant to the call/data layout reports, but they are not RAM
            // accesses and must not force consumers to approve a writable RAM
            // range. Writes in the same range remain visible because mapper
            // register protocols use them.
            if (operation == "read" && address >= 0x8000)
                return;
            // Mark indirect targets as dynamic; the two-byte span describes pointer storage, not the unknown destination range.
            bool dynamicTarget =
                operand.Mode == AddressMode.IndirectX ||
                operand.Mode == AddressMode.IndirectY;
            int span = dynamicTarget ? 2 : 1;
            if (operand.Mode == AddressMode.AbsoluteX ||
                operand.Mode == AddressMode.AbsoluteY ||
                operand.Mode == AddressMode.HighMemX ||
                operand.Mode == AddressMode.HighMemY)
            {
                // Use the smallest containing allocation to limit an indexed span; this is a report estimate, not runtime bounds enforcement.
                var allocation = _ramAllocations
                    .Where(item =>
                        item.Size > 0 &&
                        item.Address <= address &&
                        address < item.Address + item.Size)
                    .OrderBy(item => item.Size)
                    .FirstOrDefault();
                span = allocation == null
                    ? 256
                    : Math.Min(256, allocation.Address + allocation.Size - address);
            }
            if (address + span > 0x10000)
                span = 0x10000 - address;
            _ramAccesses.Add(new RamAccessInfo
            {
                Function = _currentFunctionName,
                Bank = _currentFunctionBank,
                Operation = operation,
                Address = address,
                Span = Math.Max(1, span),
                Region = ClassifyCpuMemoryRegion(address, Math.Max(1, span)),
                DynamicTarget = dynamicTarget,
            });
        }

        // Lay out a newly named aggregate once; repeated names leave the first registered layout unchanged.
        void RegisterAggregate(string name, AggregateLayout layout, FieldInfo[] fields, bool isPacked, int forcedAlign)
        {
            if (string.IsNullOrEmpty(name)) return;
            if (_aggregates.ContainsKey(name)) return;
            fields = LayoutAggregateFields(fields, layout, isPacked, forcedAlign, out int totalSize, out int finalAlign);
            _aggregates[name] = new AggregateInfo(layout, totalSize, finalAlign, isPacked, fields);
        }

        // Place union fields at zero or assign aligned struct offsets, then compute total extent and aggregate alignment.
        // Packed layouts suppress struct field padding and final tail padding; forced alignment replaces the computed alignment.
        FieldInfo[] LayoutAggregateFields(FieldInfo[] rawFields, AggregateLayout layout, bool isPacked, int forcedAlign, out int totalSize, out int finalAlign)
        {
            rawFields = rawFields ?? Array.Empty<FieldInfo>();
            FieldInfo[] fields = new FieldInfo[rawFields.Length];
            totalSize = 0;
            int naturalAlign = 1;
            if (layout == AggregateLayout.Union)
            {
                for (int i = 0; i < rawFields.Length; i++)
                {
                    var f = rawFields[i];
                    int fs = GetStorageSize(f.Type);
                    totalSize = Math.Max(totalSize, fs);
                    naturalAlign = Math.Max(naturalAlign, GetNaturalAlignment(f.Type));
                    fields[i] = new FieldInfo(f.Type, f.Name, 0);
                }
            }
            else
            {
                int offset = 0;
                for (int i = 0; i < rawFields.Length; i++)
                {
                    var f = rawFields[i];
                    int fieldAlign = isPacked ? 1 : GetNaturalAlignment(f.Type);
                    if (!isPacked && fieldAlign > 1)
                        offset = (offset + (fieldAlign - 1)) & ~(fieldAlign - 1);
                    int fs = GetStorageSize(f.Type);
                    fields[i] = new FieldInfo(f.Type, f.Name, offset);
                    offset += fs;
                    totalSize = Math.Max(totalSize, offset);
                    naturalAlign = Math.Max(naturalAlign, fieldAlign);
                }
            }
            finalAlign = forcedAlign > 0 ? forcedAlign : naturalAlign;
            if (!isPacked && finalAlign > 1)
                totalSize = (totalSize + (finalAlign - 1)) & ~(finalAlign - 1);
            return fields;
        }

        // Fold each readonly initializer, reporting nonconstant expressions and supplying zero placeholders so traversal can continue.
        int[] EvaluateReadonlyExprs(string name, Expr[] exprs)
        {
            exprs = exprs ?? Array.Empty<Expr>();
            int[] values = new int[exprs.Length];
            for (int i = 0; i < exprs.Length; i++)
            {
                int v;
                if (!TryEvaluateConstant(exprs[i], out v))
                {
                    Program.Error("error KQ0000: --target=nes phase 6 requires readonly data initializers to be constant-foldable: {0}", name);
                    v = 0;
                }
                values[i] = v;
            }
            return values;
        }

        // Reject duplicate data symbols, encode their initializer bytes and retain placement metadata for later emission.
        void RegisterReadonlyData(FilePosition source, CType type, string name, int[] values, PlacementInfo placement)
        {
            if (_constants.ContainsKey(name) || _globals.ContainsKey(name) || _readonlyData.ContainsKey(name))
            {
                Program.Error("error KQ0000: duplicate top-level symbol for --target=nes: {0}", name);
                return;
            }
            byte[] bytes = EncodeReadonlyBytes(type, name, values ?? Array.Empty<int>());
            if (Program.ErrorCount > 0) return;
            var slot = new StorageSlot
            {
                Name = name,
                Type = type,
                Address = 0,
                Size = bytes.Length,
                IsReadonlyData = true,
                ReadonlyBytes = bytes,
                Source = source,
                RequestedBank = ResolveReadonlyRequestedBank(source, name, placement),
                HasFixedBank = placement != null && placement.HasFixedBank,
                PlacementOrder = placement == null ? int.MaxValue : placement.PlacementOrder,
                SectionName = placement == null ? null : placement.SectionName,
            };
            _readonlyData[name] = slot;
            _orderedReadonlyData.Add(slot);
        }

        // Honor an explicit fixed bank before data overrides; otherwise place pooled string symbols in common bank zero.
        int ResolveReadonlyRequestedBank(FilePosition source, string name, PlacementInfo placement)
        {
            int requestedBank = placement == null ? 1 : placement.RequestedBank;
            if (placement != null && placement.HasFixedBank)
                return requestedBank;

            if (!string.IsNullOrEmpty(name) && Program.TryGetReadonlyDataBankOverride(name, out int overrideBank))
                return overrideBank;

            // Plain C pointers into PRG ROM only work for the currently visible PRG bank.
            // Cross-bank helpers regularly receive string literals, so keep them in the
            // common fixed bank unless the user explicitly pinned them elsewhere.
            if (!string.IsNullOrEmpty(name) &&
                name.StartsWith("$string", StringComparison.Ordinal))
                return 0;

            return requestedBank;
        }

        // Infer an owner from the nearest preceding function position in the same file, then apply its bank override.
        // No function-end range is checked, so this is a source-position heuristic.
        bool TryResolveOwningFunctionBank(FilePosition source, out int bank)
        {
            bank = 0;
            if (string.IsNullOrEmpty(source.Filename))
                return false;

            string bestFunctionName = null;
            int bestLine = int.MinValue;
            int bestColumn = int.MinValue;

            foreach (var kv in _functions)
            {
                Expr fnExpr = kv.Value;
                if (fnExpr == null) continue;

                FilePosition fnSource = fnExpr.Source;
                if (!string.Equals(fnSource.Filename, source.Filename, StringComparison.OrdinalIgnoreCase))
                    continue;

                if (fnSource.Line > source.Line)
                    continue;

                if (fnSource.Line == source.Line && fnSource.Column > source.Column)
                    continue;

                if (fnSource.Line < bestLine)
                    continue;

                if (fnSource.Line == bestLine && fnSource.Column < bestColumn)
                    continue;

                bestFunctionName = kv.Key;
                bestLine = fnSource.Line;
                bestColumn = fnSource.Column;
            }

            if (string.IsNullOrEmpty(bestFunctionName))
                return false;

            if (_functionInfos.TryGetValue(bestFunctionName, out CFunctionInfo info) && info != null)
            {
                bank = ResolveFunctionBankOverride(bestFunctionName, info.RomBank);
                return true;
            }

            return false;
        }

        // Encode supported scalar initializers little-endian and zero-fill the unused declared extent.
        // Report excess initializers while sizing the placeholder output to contain them.
        byte[] EncodeReadonlyBytes(CType type, string name, int[] values)
        {
            values = values ?? Array.Empty<int>();
            CType elementType = type;
            int declaredCount = values.Length;
            // Expression-sized arrays must reach the constant-evaluation branch below.
            if (type != null && type.Tag == CTypeTag.Array)
            {
                elementType = type.Subtype ?? CType.UInt8;
                declaredCount = type.Dimension > 0 ? type.Dimension : values.Length;
            }
            else if (type != null && type.Tag == CTypeTag.ArrayWithDimensionExpression)
            {
                elementType = type.Subtype ?? CType.UInt8;
                int dim;
                if (type.DimensionExpression != null && TryEvaluateConstant(type.DimensionExpression, out dim) && dim > 0)
                    declaredCount = dim;
            }

            int elemSize = NormalizeScalarSize(GetStorageSize(elementType));
            if (elemSize != 1 && elemSize != 2)
            {
                Program.Error("error KQ0000: --target=nes phase 6 readonly data currently supports only 8-bit/16-bit scalar elements: {0}", name);
                return Array.Empty<byte>();
            }
            if (declaredCount < values.Length)
            {
                Program.Error("error KQ0000: --target=nes phase 6 readonly data initializer overflow for '{0}'.", name);
                declaredCount = values.Length;
            }

            byte[] bytes = new byte[declaredCount * elemSize];
            for (int i = 0; i < values.Length; i++)
            {
                int v = values[i] & 0xFFFF;
                bytes[i * elemSize] = (byte)(v & 0xFF);
                if (elemSize == 2)
                    bytes[i * elemSize + 1] = (byte)((v >> 8) & 0xFF);
            }
            return bytes;
        }

        // Emit readonly blocks in bank/order/name order, fixing bank-zero blocks and retaining source positions.
        void EmitReadonlyData()
        {
            if (_orderedReadonlyData.Count == 0) return;
            _assembly.Add(Expr.Make(Tag.Comment, "KITAQFC phase 6 readonly data"));
            foreach (var slot in _orderedReadonlyData
                .OrderBy(x => x.RequestedBank)
                .ThenBy(x => x.PlacementOrder)
                .ThenBy(x => x.Name, StringComparer.Ordinal))
            {
                EmitPlacement(slot.RequestedBank, slot.HasFixedBank || slot.RequestedBank == 0);
                _assembly.Add(Expr.Make(Tag.ReadonlyData, slot.Name, slot.ReadonlyBytes ?? Array.Empty<byte>()).WithSource(slot.Source));
            }
        }

        PlacementInfo CreateArithmeticLookupPlacement(FunctionContext ctx)
        {
            // Keep lookup tables in the same visible PRG bank as the function that
            // directly indexes them. If the assembler were allowed to spill these
            // tables to another switchable bank, ordinary absolute reads would fetch
            // unrelated bytes from the caller's currently mapped bank.
            if (ctx != null)
            {
                return new PlacementInfo
                {
                    RequestedBank = ctx.Bank,
                    HasFixedBank = true,
                    PlacementOrder = int.MaxValue,
                    SectionName = "__kq_lookup"
                };
            }

            return new PlacementInfo
            {
                RequestedBank = 0,
                HasFixedBank = true,
                PlacementOrder = int.MaxValue,
                SectionName = "__kq_lookup"
            };
        }

        // Deduplicate lookup tables by operation, bank, signedness and constant bits; register low and optional high byte arrays.
        ArithmeticLookupInfo EnsureArithmeticLookupTable(string opName, int constantValue, FunctionContext ctx, bool signedArithmetic, bool runtimeSigned, FilePosition source)
        {
            PlacementInfo placement = CreateArithmeticLookupPlacement(ctx);
            int constantBits = constantValue & 0xFFFF;
            string key = string.Format("{0}|b{1}|s{2}|rs{3}|c{4:X4}",
                opName,
                placement.RequestedBank,
                signedArithmetic ? 1 : 0,
                runtimeSigned ? 1 : 0,
                constantBits);
            if (_arithmeticLookupTables.TryGetValue(key, out ArithmeticLookupInfo cached))
                return cached;

            BuildArithmeticLookupTables(opName, constantValue, signedArithmetic, runtimeSigned, out int[] lowBytes, out int[] highBytes);

            string labelStem = string.Format("__kq_lut_{0}_{1}_{2}_b{3}_c{4:X4}",
                opName,
                signedArithmetic ? "s" : "u",
                runtimeSigned ? "rs" : "ru",
                placement.RequestedBank,
                constantBits);

            string lowLabel = labelStem + "_lo";
            RegisterReadonlyData(source, CType.MakeArray(CType.UInt8, lowBytes.Length), lowLabel, lowBytes, placement);

            string highLabel = null;
            if (highBytes != null)
            {
                highLabel = labelStem + "_hi";
                RegisterReadonlyData(source, CType.MakeArray(CType.UInt8, highBytes.Length), highLabel, highBytes, placement);
            }

            var info = new ArithmeticLookupInfo
            {
                LowLabel = lowLabel,
                HighLabel = highLabel
            };
            _arithmeticLookupTables[key] = info;
            return info;
        }

        // Evaluate all 256 runtime byte values, keeping the low word of each arithmetic result.
        // Division/remainder by zero produce zero here; omit the high-byte table only when every entry is zero.
        void BuildArithmeticLookupTables(string opName, int constantValue, bool signedArithmetic, bool runtimeSigned, out int[] lowBytes, out int[] highBytes)
        {
            lowBytes = new int[256];
            highBytes = new int[256];

            int constantOperand = InterpretArithmeticLookupConstant(constantValue, signedArithmetic);
            bool anyHighNonZero = false;

            for (int raw = 0; raw < 256; raw++)
            {
                int runtimeOperand = runtimeSigned ? SignExtend8(raw) : raw;
                int result;
                switch (opName)
                {
                    case "mul":
                        result = runtimeOperand * constantOperand;
                        break;

                    case "div":
                        if (constantOperand == 0)
                            result = 0;
                        else if (signedArithmetic)
                            result = runtimeOperand / constantOperand;
                        else
                            result = runtimeOperand / (constantOperand & 0xFFFF);
                        break;

                    case "mod":
                        if (constantOperand == 0)
                            result = 0;
                        else if (signedArithmetic)
                            result = runtimeOperand % constantOperand;
                        else
                            result = runtimeOperand % (constantOperand & 0xFFFF);
                        break;

                    default:
                        throw new InvalidOperationException("unknown arithmetic lookup op: " + opName);
                }

                int bits = result & 0xFFFF;
                lowBytes[raw] = bits & 0xFF;
                int hi = (bits >> 8) & 0xFF;
                highBytes[raw] = hi;
                anyHighNonZero |= (hi != 0);
            }

            if (!anyHighNonZero)
                highBytes = null;
        }

        // Replace a byte-sized runtime operand plus constant with indexed ROM lookup, returning the result in A/X.
        bool TryEmitConstantLookupArithmetic(string opName, Expr runtimeExpr, int constantValue, FunctionContext ctx, bool signedArithmetic)
        {
            if (runtimeExpr == null || ctx == null)
                return false;

            if (NormalizeScalarSize(DetermineExprSize(runtimeExpr, ctx)) != 1)
                return false;

            bool runtimeSigned = signedArithmetic && IsSignedArithmeticOperand(runtimeExpr, ctx);
            ArithmeticLookupInfo table = EnsureArithmeticLookupTable(opName, constantValue, ctx, signedArithmetic, runtimeSigned, runtimeExpr.Source);
            EmitLoadA(runtimeExpr, ctx);
            EmitAsm("TAY");
            EmitAsm("LDA", AbsY(table.LowLabel));
            if (!string.IsNullOrEmpty(table.HighLabel))
                EmitAsm("LDX", AbsY(table.HighLabel));
            else
                EmitAsm("LDX", Imm(0));
            return true;
        }

        // Require known positive parameter sizes whose sum fits the sixteen-byte shared call-argument area.
        bool ValidateParameterList(string functionName, FieldInfo[] fields)
        {
            int bytes = 0;
            foreach (var field in fields ?? Array.Empty<FieldInfo>())
            {
                int size = GetStorageSize(field.Type);
                if (size <= 0)
                {
                    Program.Error("error KQ0000: --target=nes could not determine parameter size: {0}::{1}", functionName, field.Name);
                    return false;
                }
                bytes += size;
            }
            if (bytes > (CallArgLimitExclusive - CallArgBase))
            {
                Program.Error("error KQ0000: --target=nes phase 5 parameter area overflow for function '{0}' ({1} bytes > {2} bytes).",
                    functionName, bytes, (CallArgLimitExclusive - CallArgBase));
                return false;
            }
            return true;
        }

        // Accept matching repeated fixed slots, otherwise allocate a new fixed, high-memory, OAM-shadow or ordinary global slot.
        // High-memory declarations use the local-like allocator and may fall back outside zero page.
        void DeclareGlobal(FilePosition source, MemoryRegion region, CType type, string name)
        {
            StorageSlot existing;
            if (_globals.TryGetValue(name, out existing))
            {
                int sizeNow = GetStorageSize(type);
                bool sameFixedSlot = existing.Address == region.FixedAddress && region.Tag == MemoryRegionTag.Fixed && existing.Size == sizeNow;
                if (sameFixedSlot)
                    return;
            }

            if (_constants.ContainsKey(name) || _globals.ContainsKey(name) || _readonlyData.ContainsKey(name))
            {
                Program.Error("error KQ0000: duplicate top-level symbol for --target=nes: {0}", name);
                return;
            }

            int size = GetStorageSize(type);
            if (size <= 0)
            {
                Program.Error("error KQ0000: --target=nes phase 6 could not determine storage size for global '{0}'.", name);
                return;
            }

            int address;
            bool allocatedByLocalLike = false;
            if (region.Tag == MemoryRegionTag.Fixed)
            {
                address = region.FixedAddress;
            }
            else if (region.Tag == MemoryRegionTag.HighMem)
            {
                address = AllocateLocalLike(name, size, source, "global_highmem");
                allocatedByLocalLike = true;
            }
            else if (region.Tag == MemoryRegionTag.Oam)
            {
                if (_nextOamRam + size > OamRamLimitExclusive)
                {
                    Program.Error("error KQ0000: --target=nes phase 6 ran out of OAM shadow RAM while allocating global '{0}'.", name);
                    return;
                }
                address = _nextOamRam;
                _nextOamRam += size;
            }
            else
            {
                if (_nextGlobalRam + size > GlobalRamLimitExclusive)
                {
                    Program.Error("error KQ0000: --target=nes phase 6 ran out of internal RAM while allocating global '{0}'.", name);
                    return;
                }
                address = _nextGlobalRam;
                _nextGlobalRam += size;
            }

            var slot = new StorageSlot
            {
                Name = name,
                Type = type,
                Address = address,
                Size = size,
                Source = source,
            };
            _globals[name] = slot;
            if (!allocatedByLocalLike)
            {
                string kind = region.Tag == MemoryRegionTag.Fixed
                    ? "global_fixed"
                    : (region.Tag == MemoryRegionTag.Oam ? "global_oam" : "global");
                RecordRamAllocation(name, kind, address, size, false);
            }
        }

        // Prefer available zero page, then the configured dedicated local window, then ordinary global RAM.
        // An exhausted configured window reports an error rather than falling back to global RAM.
        int AllocateLocalLike(string name, int size, FilePosition source, string kind)
        {
            if (size <= 0)
            {
                Program.Error("error KQ0000: invalid ZP allocation size for '{0}'.", name);
                return LocalZpBase;
            }
            if (Program.EnableWholeProgramZpAllocator && _nextLocalZp + size <= LocalZpLimitExclusive)
            {
                int zpAddress = _nextLocalZp;
                _nextLocalZp += size;
                _zpAllocations.Add(new ZpAllocationInfo { Name = name, Kind = "storage", Address = zpAddress, Size = size, Reason = GuessZpReason(name, source) });
                RecordRamAllocation(name, kind, zpAddress, size, false);
                return zpAddress;
            }

            if (Program.HasNesLocalRamWindow)
            {
                int limitExclusive = Program.NesLocalRamBase + Program.NesLocalRamLength;
                if (_nextDedicatedLocalRam < Program.NesLocalRamBase ||
                    _nextDedicatedLocalRam + size > limitExclusive)
                {
                    Program.Error("error KQ0000: --target=nes phase 6 ran out of configured local RAM while allocating '{0}'.", name);
                    return Program.NesLocalRamBase;
                }
                int dedicatedAddress = _nextDedicatedLocalRam;
                _nextDedicatedLocalRam += size;
                RecordRamAllocation(name, kind, dedicatedAddress, size, false);
                return dedicatedAddress;
            }

            if (_nextGlobalRam + size > GlobalRamLimitExclusive)
            {
                Program.Error("error KQ0000: --target=nes phase 6 ran out of internal RAM while allocating local '{0}'.", name);
                return GlobalRamBase;
            }

            int ramAddress = _nextGlobalRam;
            _nextGlobalRam += size;
            RecordRamAllocation(name, kind, ramAddress, size, false);
            return ramAddress;
        }

        // Check configured local/temp windows against allocations already recorded after top-level collection, then against each other.
        // Only the temporary pool's own matching allocation is explicitly exempted.
        void ValidateConfiguredRamWindows()
        {
            if (Program.HasNesLocalRamWindow)
            {
                int start = Program.NesLocalRamBase;
                int endExclusive = start + Program.NesLocalRamLength;
                foreach (var allocation in _ramAllocations)
                {
                    if (RangesOverlap(start, endExclusive, allocation.Address, allocation.Address + allocation.Size))
                    {
                        Program.Error(
                            "error KQ0000: configured --nes-local-ram window ${0:X4}-${1:X4} overlaps reserved {2} RAM ${3:X4}-${4:X4}.",
                            start, endExclusive - 1, allocation.Kind, allocation.Address, allocation.Address + allocation.Size - 1);
                        break;
                    }
                }
            }

            if (Program.HasCustomNesTempRamWindow)
            {
                int start = Program.NesTempRamBase;
                int endExclusive = start + Program.NesTempRamLength;
                foreach (var allocation in _ramAllocations)
                {
                    if (allocation.Kind == "compiler_temp_pool" &&
                        allocation.Address == start &&
                        allocation.Size == Program.NesTempRamLength)
                        continue;
                    if (RangesOverlap(start, endExclusive, allocation.Address, allocation.Address + allocation.Size))
                    {
                        Program.Error(
                            "error KQ0000: configured --nes-temp-ram window ${0:X4}-${1:X4} overlaps reserved {2} RAM ${3:X4}-${4:X4}.",
                            start, endExclusive - 1, allocation.Kind, allocation.Address, allocation.Address + allocation.Size - 1);
                        break;
                    }
                }
            }

            if (Program.HasNesLocalRamWindow && Program.HasCustomNesTempRamWindow &&
                RangesOverlap(
                    Program.NesLocalRamBase,
                    Program.NesLocalRamBase + Program.NesLocalRamLength,
                    Program.NesTempRamBase,
                    Program.NesTempRamBase + Program.NesTempRamLength))
            {
                Program.Error("error KQ0000: configured --nes-local-ram and --nes-temp-ram windows overlap.");
            }
        }

        // Test strict overlap of two half-open address intervals; touching endpoints do not overlap.
        static bool RangesOverlap(int leftStart, int leftEndExclusive, int rightStart, int rightEndExclusive)
        {
            return leftStart < rightEndExclusive && rightStart < leftEndExclusive;
        }

        // Choose a descriptive report reason from name substrings; this label does not control the allocation decision.
        string GuessZpReason(string name, FilePosition source)
        {
            string n = (name ?? "").ToLowerInvariant();
            if (n.Contains("x") || n.Contains("y") || n.Contains("i") || n.Contains("idx")) return "small loop/index/coordinate candidate";
            if (n.Contains("ptr") || n.Contains("addr")) return "pointer/address candidate";
            return "whole-program zero-page allocator first-fit candidate";
        }

        // Supply missing reset/NMI/IRQ entry points in common code and always emit the terminal hang loop.
        void EmitAutoEntryStubs()
        {
            if (!_functions.ContainsKey("__nes_reset"))
            {
                EmitPlacement(0, true);
                _assembly.Add(Expr.Make(Tag.Function, "__kq_reset_stub"));
                // Initialize the FDS BIOS control bytes, mapper/PPU shadows and resident-bank state before enabling IRQs.
                if (Program.NesMapperProfile.HasFds)
                {
                    _assembly.Add(Expr.Make(Tag.Comment, "KITAQFC FDS auto-generated reset stub ($6000-$DFFF PRG-RAM, BIOS-safe startup)"));
                    EmitAsm("CLD");
                    EmitAsm("LDX", Imm(0xFF));
                    EmitAsm("TXS");
                    // Keep the FDS BIOS vector-control bytes valid even if user code clears stack RAM.
                    EmitAsm("LDA", Imm(0xC0)); EmitAsm("STA", Mem(0x0100)); // NMI #3 -> ($DFFA)
                    EmitAsm("LDA", Imm(0x80)); EmitAsm("STA", Mem(0x0101)); // BIOS ack+delay IRQ default
                    EmitAsm("LDA", Imm(0x35)); EmitAsm("STA", Mem(0x0102)); // boot files loaded
                    EmitAsm("LDA", Imm(0xAC)); EmitAsm("STA", Mem(0x0103)); // first game boot
                    EmitAsm("LDA", Imm(0x00)); EmitAsm("STA", Mem(0x4022)); // disable FDS timer IRQ
                    int mirrorValue = Program.NesCartridge.Mirroring == NesMirroringKind.Vertical ? 0x26 : 0x2E;
                    EmitAsm("LDA", Imm(mirrorValue)); EmitAsm("STA", Mem(0x4025)); EmitAsm("STA", Mem(0x00FA));
                    EmitAsm("LDA", Imm(0x00));
                    EmitAsm("STA", Mem(0x2000)); EmitAsm("STA", Mem(0x00FF)); // PPUCTRL mirror
                    EmitAsm("STA", Mem(0x2001)); EmitAsm("STA", Mem(0x00FE)); // PPUMASK mirror
                    EmitAsm("LDA", Imm(1));
                    EmitAsm("STA", Mem(_runtimeCurrentBankAddress));
                    EmitAsm("STA", Mem(_runtimeFdsResidentBankAddress));
                    EmitAsm("CLI");
                }
                else
                {
                    _assembly.Add(Expr.Make(Tag.Comment, "KITAQFC phase 6 auto-generated reset stub (NROM-256, arrays/struct fields/readonly data support)"));
                    EmitAsm("SEI");
                    EmitAsm("CLD");
                    EmitAsm("LDX", Imm(0xFF));
                    EmitAsm("TXS");
                    if (Program.NesMapperProfile.IsSurom512)
                    {
                        EmitAsm("LDA", Imm(0));
                        EmitAsm("STA", Mem(_runtimeMmc1OuterShadowAddress));
                        EmitAsm("STA", Mem(_runtimeMmc1InnerShadowAddress));
                        EmitAsm("STA", Mem(_runtimeMapperFaultAddress));
                        EmitAsm("LDA", Imm(GetMmc1ControlValue()));
                        EmitAsm("STA", Mem(_runtimeMmc1ControlShadowAddress));
                    }
                    EmitAsm("LDA", Imm(1));
                    EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
                }
                if (_functions.ContainsKey("main"))
                    EmitResolvedCall("main", 0, FilePosition.Unknown, expectedReturnSize: 0);
                EmitAsm("JMP", Abs("__kq_hang"));
            }

            // The default NMI executes the queued VRAM work and increments the runtime frame counter before RTI.
            if (!_functions.ContainsKey("__nes_nmi"))
            {
                EmitPlacement(0, true);
                _assembly.Add(Expr.Make(Tag.Function, "__kq_nmi_default"));
                EmitAsm("JSR", Abs("__vramq_exec"));
                EmitAsm("INC", Mem(_runtimeNmiCounterAddress));
                EmitAsm("RTI");
            }

            if (!_functions.ContainsKey("__nes_irq"))
            {
                EmitPlacement(0, true);
                _assembly.Add(Expr.Make(Tag.Function, "__kq_irq_default"));
                EmitAsm("RTI");
            }

            EmitPlacement(0, true);
            _assembly.Add(Expr.Make(Tag.Function, "__kq_hang"));
            _assembly.Add(Expr.Make(Tag.Label, "__kq_hang_loop"));
            EmitAsm("JMP", Abs("__kq_hang_loop"));
        }

        // Emit reachable bodies in bank/placement/input order, with RTI for interrupt functions and RTS for ordinary functions.
        void EmitUserFunctions()
        {
            foreach (string functionKey in _orderedFunctionNames
                .OrderBy(name => GetFunctionBank(name))
                .ThenBy(name => GetFunctionPlacementOrder(name))
                .ThenBy(name => _orderedFunctionNames.IndexOf(name)))
            {
                Expr item = _functions[functionKey];
                CType retType;
                FieldInfo[] fields;
                int mustCheck;
                Expr body;
                string emittedName;
                if (!item.Match<CType, string, FieldInfo[], int, Expr>(Tag.Function, out retType, out emittedName, out fields, out mustCheck, out body) &&
                    !item.Match<CType, string, FieldInfo[], int, Expr>(Tag.InlineFunction, out retType, out emittedName, out fields, out mustCheck, out body))
                    continue;

                if (Program.EnableLibraryLtoLite && _reachableFunctions != null && !_reachableFunctions.Contains(emittedName))
                    continue;

                int currentBank = GetFunctionBank(emittedName);
                EmitPlacement(currentBank, IsFunctionFixedBank(emittedName) || currentBank == 0);
                _assembly.Add(Expr.Make(Tag.Function, emittedName).WithSource(item.Source));
                string returnMnemonic = (string.Equals(emittedName, "__nes_nmi", StringComparison.Ordinal) || string.Equals(emittedName, "__nes_irq", StringComparison.Ordinal)) ? "RTI" : "RTS";
                var ctx = new FunctionContext(emittedName, currentBank, returnMnemonic, retType);
                _currentFunctionName = emittedName;
                _currentFunctionBank = currentBank;
                EmitFunctionPrologue(item.Source, fields ?? Array.Empty<FieldInfo>(), ctx);
                EmitStatement(body, ctx);
                EmitAsm(returnMnemonic);
                _currentFunctionName = "<none>";
                _currentFunctionBank = 0;
            }
        }

        // Warn about recorded direct self-calls when static frames are enabled; this check does not find indirect recursion cycles.
        void EmitStaticFrameSafetyDiagnostics()
        {
            if (!Program.EnableStaticFrameAllocator) return;
            foreach (var edge in _callReportMap.Values)
            {
                if (edge != null && string.Equals(edge.Caller, edge.Callee, StringComparison.Ordinal))
                {
                    WarnOnce(ErrorCode.NesBankCrossingCall, "static_frame_recursion:" + edge.Caller, FilePosition.Unknown,
                        "Static frame optimization detected direct recursion in {0}; NES static frames are non-reentrant. Rewrite this as an explicit work stack or disable --static-frame.", edge.Caller);
                }
            }
        }

        // Emit the common bank-switch helper, clamp invalid low bank numbers (and SUROM upper bounds), then emit requested call stubs.
        void EmitBankHelpers()
        {
            EmitPlacement(0, true);
            _assembly.Add(Expr.Make(Tag.Function, "__kq_prg_set_bank_a"));
            string bankOkLabel = NewGeneratedLabel("prg_bank_ok");
            if (Program.NesMapperProfile.IsSurom512)
            {
                string bankFallbackLabel = NewGeneratedLabel("prg_bank_fallback");
                EmitAsm("CMP", Imm(1));
                EmitAsm("BCC", Rel(bankFallbackLabel));
                EmitAsm("CMP", Imm(31));
                EmitAsm("BCC", Rel(bankOkLabel));
                _assembly.Add(Expr.Make(Tag.Label, bankFallbackLabel));
                EmitAsm("LDA", Imm(1));
                EmitAsm("STA", Mem(_runtimeMapperFaultAddress));
            }
            else
            {
                EmitAsm("CMP", Imm(1));
                EmitAsm("BCS", Rel(bankOkLabel));
                EmitAsm("LDA", Imm(1));
            }
            _assembly.Add(Expr.Make(Tag.Label, bankOkLabel));
            EmitAsm("STA", Mem(_runtimeCurrentBankAddress));
            EmitMapperSwitchSequence();
            EmitAsm("RTS");

            foreach (var thunkName in _orderedThunkNames)
            {
                BankThunkInfo info = _bankThunkInfos[thunkName];
                EmitPlacement(0, true);
                _assembly.Add(Expr.Make(Tag.Function, thunkName));
                if (info.UseFdsOverlay)
                    EmitFdsOverlayThunkBody(info);
                else
                    EmitMapperBankThunkBody(info);
            }
        }

        // Traverse recorded call edges from user NMI/IRQ roots and reject reachable mapper writers or functions outside common bank zero.
        void ValidateSuromInterruptSafety()
        {
            if (!Program.NesMapperProfile.IsSurom512) return;

            var reachable = new HashSet<string>(StringComparer.Ordinal);
            var pending = new Queue<string>();
            foreach (string root in new[] { "__nes_nmi", "__nes_irq" })
            {
                if (_functions.ContainsKey(root) && reachable.Add(root)) pending.Enqueue(root);
            }

            while (pending.Count > 0)
            {
                string caller = pending.Dequeue();
                foreach (var edge in _callReportMap.Values.Where(e => string.Equals(e.Caller, caller, StringComparison.Ordinal)))
                    if (!string.IsNullOrEmpty(edge.Callee) && reachable.Add(edge.Callee)) pending.Enqueue(edge.Callee);
            }

            foreach (string name in reachable.OrderBy(x => x, StringComparer.Ordinal))
            {
                if (_mapperWritingFunctions.Contains(name))
                    Program.Error("error KQFC2606 KQFC-SUROM-NMI-MAPPER-WRITE: MMC1 register write is reachable from interrupt function '{0}' on board 'surom512'.", name);
                if (GetFunctionBank(name) != 0)
                    Program.Error("error KQFC2607: interrupt-reachable function '{0}' is in logical switch bank {1}; SUROM interrupt code and its readonly data must remain in common bank 0.", name, GetFunctionBank(name));
            }
        }


        // Save the caller bank on the CPU stack, switch and call, then restore the bank while retaining A/X return bytes in runtime slots.
        void EmitMapperBankThunkBody(BankThunkInfo info)
        {
            EmitAsm("LDA", Mem(_runtimeCurrentBankAddress));
            EmitAsm("PHA");
            EmitAsm("LDA", Imm(info.TargetBank));
            EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
            EmitAsm("JSR", Abs(info.TargetName));
            if (info.ReturnSize > 0)
            {
                EmitAsm("STA", Mem(_runtimeReturnLoAddress));
                if (info.ReturnSize > 1)
                    EmitAsm("STX", Mem(_runtimeReturnHiAddress));
            }
            EmitAsm("PLA");
            EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
            if (info.ReturnSize > 0)
            {
                EmitAsm("LDA", Mem(_runtimeReturnLoAddress));
                if (info.ReturnSize > 1)
                    EmitAsm("LDX", Mem(_runtimeReturnHiAddress));
            }
            EmitAsm("RTS");
        }

        // Load the target overlay when needed, call it, and request restoration of the previous resident bank.
        // Initial load failure returns its status; the later restore call's status is not checked here.
        void EmitFdsOverlayThunkBody(BankThunkInfo info)
        {
            string loaded = NewGeneratedLabel("fds_ovl_loaded");
            string restoreDone = NewGeneratedLabel("fds_ovl_restore_done");
            string loadFailed = NewGeneratedLabel("fds_ovl_load_failed");

            EmitAsm("LDA", Mem(_runtimeFdsResidentBankAddress));
            EmitAsm("PHA");
            if (Program.FdsOverlayResidencyGuardEnabled)
            {
                EmitAsm("CMP", Imm(info.TargetBank));
                EmitAsm("BEQ", Rel(loaded));
            }
            EmitAsm("LDA", Imm(info.TargetBank));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__fds_load_bank"));
            EmitAsm("CMP", Imm(0));
            EmitAsm("BNE", Rel(loadFailed));
            _assembly.Add(Expr.Make(Tag.Label, loaded));
            EmitAsm("JSR", Abs(info.TargetName));
            if (info.ReturnSize > 0)
            {
                EmitAsm("STA", Mem(_runtimeReturnLoAddress));
                if (info.ReturnSize > 1)
                    EmitAsm("STX", Mem(_runtimeReturnHiAddress));
            }
            EmitAsm("PLA");
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            if (Program.FdsOverlayResidencyGuardEnabled)
            {
                EmitAsm("CMP", Mem(_runtimeFdsResidentBankAddress));
                EmitAsm("BEQ", Rel(restoreDone));
            }
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__fds_load_bank"));
            _assembly.Add(Expr.Make(Tag.Label, restoreDone));
            if (info.ReturnSize > 0)
            {
                EmitAsm("LDA", Mem(_runtimeReturnLoAddress));
                if (info.ReturnSize > 1)
                    EmitAsm("LDX", Mem(_runtimeReturnHiAddress));
            }
            EmitAsm("RTS");

            _assembly.Add(Expr.Make(Tag.Label, loadFailed));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("PLA");
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            if (info.ReturnSize > 1)
                EmitAsm("LDX", Imm(0));
            EmitAsm("RTS");
        }

        // Save an aggregate-return destination when needed, then copy incoming fast-call A/X or shared argument bytes into local slots.
        void EmitFunctionPrologue(FilePosition source, FieldInfo[] fields, FunctionContext ctx)
        {
            fields = fields ?? Array.Empty<FieldInfo>();
            bool fastCall = IsFunctionFastCall(ctx == null ? null : ctx.Name);
            int argOffset = 0;
            if (ctx != null && IsAggregateType(ctx.ReturnType))
            {
                ctx.StructReturnPointerSlot = DeclareLocal(source, ctx, CType.MakePointer(ctx.ReturnType), "__kq_sret_dst");
                if (ctx.StructReturnPointerSlot != null)
                {
                    EmitAsm("LDA", Mem(_runtimeSretPtrLoAddress));
                    EmitAsm("STA", Mem(ctx.StructReturnPointerSlot.Address));
                    EmitAsm("LDA", Mem(_runtimeSretPtrHiAddress));
                    EmitAsm("STA", Mem(ctx.StructReturnPointerSlot.Address + 1));
                }
            }
            for (int i = 0; i < fields.Length; i++)
            {
                var field = fields[i];
                StorageSlot slot = DeclareLocal(source, ctx, field.Type, field.Name);
                if (slot == null) return;

                if (fastCall && fields.Length == 1 && slot.Size == 1)
                {
                    EmitAsm("STA", Mem(slot.Address));
                }
                else if (fastCall && fields.Length == 1 && slot.Size == 2)
                {
                    EmitAsm("STA", Mem(slot.Address));
                    EmitAsm("STX", Mem(slot.Address + 1));
                }
                else if (fastCall && fields.Length == 2 && slot.Size == 1)
                {
                    if (i == 0) EmitAsm("STA", Mem(slot.Address));
                    else { EmitAsm("TXA"); EmitAsm("STA", Mem(slot.Address)); }
                }
                else if (slot.Size == 1)
                {
                    EmitAsm("LDA", Mem(CallArgBase + argOffset));
                    EmitAsm("STA", Mem(slot.Address));
                }
                else
                {
                    for (int b = 0; b < slot.Size; b++)
                    {
                        EmitAsm("LDA", Mem(CallArgBase + argOffset + b));
                        EmitAsm("STA", Mem(slot.Address + b));
                    }
                }
                argOffset += slot.Size;
            }
        }

        // Dispatch lowered statement nodes into storage, control flow, calls, returns or assembly; diagnose unsupported tags.
        void EmitStatement(Expr node, FunctionContext ctx)
        {
            if (node == null) return;

            Expr[] seq;
            Expr body;
            Expr init;
            Expr test;
            Expr induct;
            Expr lhs;
            Expr rhs;
            CType varType;
            string varName;
            string label;
            string mnemonic;
            AsmOperand operand;

            if (node.MatchAny(Tag.Sequence, out seq))
            {
                foreach (var part in seq) EmitStatement(part, ctx);
                return;
            }

            if (node.Match(Tag.Empty))
                return;

            if (node.Match(Tag.Unsafe, out body))
            {
                EmitStatement(body, ctx);
                return;
            }

            if (node.Match(Tag.Variable, out varType, out varName, out Expr _range) || node.Match(Tag.Variable, out varType, out varName))
            {
                DeclareLocal(node.Source, ctx, varType, varName);
                return;
            }

            if (node.Match(Tag.Assign, out lhs, out rhs))
            {
                EmitAssignment(lhs, rhs, ctx);
                return;
            }

            string opName;
            // Lower a compound assignment to an assignment whose value reads the same lvalue expression.
            if (node.Match(Tag.AssignModify, out opName, out lhs, out rhs))
            {
                EmitAssignment(lhs, Expr.Make(opName, lhs, rhs).WithSource(node.Source), ctx);
                return;
            }

            if (node.MatchAny(Tag.If, out seq))
            {
                EmitIf(node.Source, seq, ctx);
                return;
            }

            if (node.Match(Tag.For, out init, out test, out induct, out body))
            {
                EmitFor(node.Source, init, test, induct, body, ctx);
                return;
            }

            if (node.Match(Tag.Break))
            {
                if (ctx.LoopStack.Count == 0)
                {
                    Program.Error("error KQ0000: 'break' used outside of a loop in function {0}.", ctx.Name);
                    return;
                }
                EmitAsm("JMP", Abs(ctx.LoopStack.Peek().BreakLabel));
                return;
            }

            if (node.Match(Tag.Continue))
            {
                if (ctx.LoopStack.Count == 0)
                {
                    Program.Error("error KQ0000: 'continue' used outside of a loop in function {0}.", ctx.Name);
                    return;
                }
                EmitAsm("JMP", Abs(ctx.LoopStack.Peek().ContinueLabel));
                return;
            }

            if (node.Match(Tag.Return))
            {
                EmitAsm(ctx.ReturnMnemonic);
                return;
            }

            Expr returnExpr;
            if (node.Match(Tag.Return, out returnExpr))
            {
                if (IsAggregateType(ctx.ReturnType))
                {
                    EmitAggregateReturn(returnExpr, ctx);
                    EmitAsm(ctx.ReturnMnemonic);
                    return;
                }
                EmitLoadValue(returnExpr, ctx, TypeStorageSize(ctx.ReturnType));
                EmitAsm(ctx.ReturnMnemonic);
                return;
            }

            // Normalize a raw instruction, record its numeric memory/call observations and retain its source position.
            if (node.Match(Tag.Asm, out mnemonic, out operand))
            {
                string normalizedMnemonic = NormalizeMnemonic(mnemonic);
                AsmOperand normalizedOperand = NormalizeAsmOperandForEmission(operand, ctx);
                RecordRamAccess(normalizedMnemonic, normalizedOperand);
                RecordRawAssemblyCall(normalizedMnemonic, normalizedOperand);
                _assembly.Add(Expr.Make(Tag.Asm, normalizedMnemonic, normalizedOperand).WithSource(node.Source));
                return;
            }

            if (node.Match(Tag.Label, out label))
            {
                _assembly.Add(Expr.Make(Tag.Label, label).WithSource(node.Source));
                return;
            }

            if (node.Match(Tag.Jump, out label))
            {
                EmitAsm("JMP", Abs(label));
                return;
            }

            Expr funcExpr;
            Expr[] args;
            if (node.MatchAny(Tag.Call, out funcExpr, out args))
            {
                CType callType;
                if (TryGetExprType(node, ctx, out callType) && IsAggregateType(callType))
                {
                    EmitAggregateCallToTemp(node, ctx, callType);
                    return;
                }
                EmitCall(node.Source, funcExpr, args, ctx, expectedReturnSize: 0);
                return;
            }

            if (node.Match(Tag.PreIncrement, out lhs) || node.Match(Tag.PostIncrement, out lhs))
            {
                EmitIncDec(lhs, +1, ctx, false);
                return;
            }

            if (node.Match(Tag.PreDecrement, out lhs) || node.Match(Tag.PostDecrement, out lhs))
            {
                EmitIncDec(lhs, -1, ctx, false);
                return;
            }

            if (IsValueExpression(node))
            {
                EmitLoadValue(node, ctx, DetermineExprSize(node, ctx));
                return;
            }

            Program.Error("error KQ0000: --target=nes phase 5 unsupported statement tag in function {0}: {1}", ctx.Name, node.Tag);
        }

        // Allocate a unique function-local slot, rejecting collisions with local or global data names.
        // Slots use the shared local-like allocator and persist across emitted functions; optional static-frame metadata describes the allocation.
        StorageSlot DeclareLocal(FilePosition source, FunctionContext ctx, CType type, string name)
        {
            if (ctx.Locals.ContainsKey(name) || _constants.ContainsKey(name) || _globals.ContainsKey(name) || _readonlyData.ContainsKey(name))
            {
                Program.Error("error KQ0000: duplicate local symbol for --target=nes phase 5 in function {0}: {1}", ctx.Name, name);
                return null;
            }

            int size = GetStorageSize(type);
            if (size <= 0)
            {
                Program.Error("error KQ0000: --target=nes phase 6 could not determine storage size for local '{0}'.", name);
                return null;
            }

            var slot = new StorageSlot
            {
                Name = name,
                Type = type,
                Size = size,
                Address = AllocateLocalLike(name, size, source, "local"),
                Source = source,
            };
            ctx.Locals[name] = slot;
            if (Program.EnableStaticFrameAllocator)
                _staticFrameSlots.Add(new StaticFrameSlotInfo { Function = ctx == null ? "<none>" : ctx.Name, Name = name, Address = slot.Address, Size = slot.Size, Region = slot.Address < 0x0100 ? "zp_static_frame" : "ram_static_frame" });
            return slot;
        }

        // Emit condition/body pairs with a shared exit; a final nonzero literal condition represents an unconditional else branch.
        void EmitIf(FilePosition source, Expr[] parts, FunctionContext ctx)
        {
            if (parts == null || parts.Length < 2 || (parts.Length % 2) != 0)
            {
                Program.Error("error KQ0000: malformed if node in function {0}.", ctx.Name);
                return;
            }

            string endLabel = NewGeneratedLabel("if_end");
            for (int i = 0; i < parts.Length; i += 2)
            {
                Expr cond = parts[i];
                Expr branch = parts[i + 1];
                bool isElse = false;
                int condValue;
                if (i == parts.Length - 2 && cond.Match(Tag.Integer, out condValue) && condValue != 0)
                    isElse = true;

                if (isElse)
                {
                    EmitStatement(branch, ctx);
                    break;
                }

                string nextLabel = NewGeneratedLabel("if_next");
                EmitBranchIfFalse(cond, nextLabel, ctx);
                EmitStatement(branch, ctx);
                EmitAsm("JMP", Abs(endLabel));
                _assembly.Add(Expr.Make(Tag.Label, nextLabel).WithSource(source));
            }

            _assembly.Add(Expr.Make(Tag.Label, endLabel).WithSource(source));
        }

        // Attempt the optional counted-loop specialization, otherwise emit initialization, condition, body and increment with break/continue labels.
        void EmitFor(FilePosition source, Expr init, Expr test, Expr induct, Expr body, FunctionContext ctx)
        {
            if (Program.EnableLoopLoweringOptimizer && TryEmitCountedFor(source, init, test, induct, body, ctx))
                return;

            string startLabel = NewGeneratedLabel("for_start");
            string continueLabel = NewGeneratedLabel("for_continue");
            string breakLabel = NewGeneratedLabel("for_break");

            EmitStatement(init, ctx);
            _assembly.Add(Expr.Make(Tag.Label, startLabel).WithSource(source));
            if (test != null && !test.Match(Tag.Empty))
                EmitBranchIfFalse(test, breakLabel, ctx);

            ctx.LoopStack.Push(new LoopLabels(continueLabel, breakLabel));
            EmitStatement(body, ctx);
            ctx.LoopStack.Pop();

            _assembly.Add(Expr.Make(Tag.Label, continueLabel).WithSource(source));
            if (induct != null && !induct.Match(Tag.Empty))
                EmitStatement(induct, ctx);
            EmitAsm("JMP", Abs(startLabel));
            _assembly.Add(Expr.Make(Tag.Label, breakLabel).WithSource(source));
        }

        // Recognize a named byte counter with constant start/limit and unit increment, then emit a compact compare/increment loop.
        // The limit-256, start-zero case exits on wrap; signed counters use the generic comparison path.
        bool TryEmitCountedFor(FilePosition source, Expr init, Expr test, Expr induct, Expr body, FunctionContext ctx)
        {
            string var; int start; int limit;
            if (!TryMatchForInit(init, out var, out start)) return false;
            if (!TryMatchForLessThan(test, var, out limit)) return false;
            if (!TryMatchIncrement(induct, var)) return false;
            if (limit < 0 || limit > 256 || start < 0 || start > 255 || start >= limit) return false;
            StorageSlot slot;
            if (!TryResolveStorage(ctx, var, out slot) || slot.Size != 1 || IsSignedIntegerType(slot.Type)) return false;

            string startLabel = NewGeneratedLabel("for_counted_start");
            string breakLabel = NewGeneratedLabel("for_counted_break");
            string continueLabel = NewGeneratedLabel("for_counted_continue");
            EmitAsm("LDA", Imm(start));
            EmitAsm("STA", Mem(slot.Address));
            _assembly.Add(Expr.Make(Tag.Label, startLabel).WithSource(source));
            EmitAsm("LDA", Mem(slot.Address));
            if (limit < 256)
            {
                EmitAsm("CMP", Imm(limit));
                EmitAsm("BCS", Rel(breakLabel));
            }
            else
            {
                // i < 256 is always true for u8.  The INC/BNE below terminates on wrap.
            }

            ctx.LoopStack.Push(new LoopLabels(continueLabel, breakLabel));
            EmitStatement(body, ctx);
            ctx.LoopStack.Pop();
            _assembly.Add(Expr.Make(Tag.Label, continueLabel).WithSource(source));
            EmitAsm("INC", Mem(slot.Address));
            if (limit == 256 && start == 0)
                EmitAsm("BNE", Rel(startLabel));
            else
                EmitAsm("JMP", Abs(startLabel));
            _assembly.Add(Expr.Make(Tag.Label, breakLabel).WithSource(source));
            _loopLowerings.Add(new LoopLoweringInfo { Function = ctx == null ? "<none>" : ctx.Name, Variable = var, Limit = limit, Kind = limit == 256 ? "u8-wrap-counted-loop" : "u8-counted-loop", Source = source.ToString() });
            return true;
        }

        // Accept an assignment of a foldable constant to a plain named loop counter.
        bool TryMatchForInit(Expr init, out string var, out int start)
        {
            var = null; start = 0;
            Expr lhs, rhs;
            if (init == null || !init.Match(Tag.Assign, out lhs, out rhs)) return false;
            if (!lhs.Match(Tag.Name, out var)) return false;
            return TryEvaluateConstant(rhs, out start);
        }

        // Accept a strict less-than comparison of that exact counter name against a foldable constant.
        bool TryMatchForLessThan(Expr test, string var, out int limit)
        {
            limit = 0;
            Expr lhs, rhs; string n;
            if (test == null || !test.Match(Tag.LessThan, out lhs, out rhs)) return false;
            if (!lhs.Match(Tag.Name, out n) || !string.Equals(n, var, StringComparison.Ordinal)) return false;
            return TryEvaluateConstant(rhs, out limit);
        }

        // Accept prefix/postfix increment or += with a constant one for the same named counter.
        bool TryMatchIncrement(Expr induct, string var)
        {
            Expr lhs; string n;
            if (induct == null) return false;
            if ((induct.Match(Tag.PostIncrement, out lhs) || induct.Match(Tag.PreIncrement, out lhs)) && lhs.Match(Tag.Name, out n))
                return string.Equals(n, var, StringComparison.Ordinal);
            string op; Expr rhs; int one;
            if (induct.Match(Tag.AssignModify, out op, out lhs, out rhs) && op == Tag.Add && lhs.Match(Tag.Name, out n) && string.Equals(n, var, StringComparison.Ordinal) && TryEvaluateConstant(rhs, out one))
                return one == 1;
            return false;
        }

        // Handle aggregate copies first; for scalar destinations, load the requested width and store the value through the lvalue.
        void EmitAssignment(Expr lhs, Expr rhs, FunctionContext ctx)
        {
            if (TryEmitAggregateAssignment(Expr.Make(Tag.Assign, lhs, rhs).WithSource(lhs.Source), lhs, rhs, ctx))
                return;

            int size = GetLvalueSize(lhs, ctx, requireWritable: true);
            if (Program.ErrorCount > 0) return;

            EmitLoadValue(rhs, ctx, size);
            EmitStoreLoadedValueToLvalue(lhs, ctx, size, preserveResult: false);
        }

        // Claim assignments involving aggregates, checking compatible complete types and writable destinations.
        // Write a call result directly to the destination when possible; otherwise emit a forward byte copy, releasing temporary pointer slots in finally blocks.
        bool TryEmitAggregateAssignment(Expr assignExpr, Expr lhs, Expr rhs, FunctionContext ctx)
        {
            CType leftType;
            CType rightType;
            bool haveLeft = TryGetExprType(lhs, ctx, out leftType);
            bool haveRight = TryGetExprType(rhs, ctx, out rightType);
            bool leftAgg = haveLeft && IsAggregateType(leftType);
            bool rightAgg = haveRight && IsAggregateType(rightType);
            if (!leftAgg && !rightAgg) return false;

            if (!IsSameCompleteAggregateType(leftType, rightType))
            {
                Program.Error(Maybe.Just(assignExpr.Source), ErrorCode.ParseError,
                    "incompatible struct/union assignment: {0} = {1}",
                    leftType == null ? "<unknown>" : leftType.Show(),
                    rightType == null ? "<unknown>" : rightType.Show());
                return true;
            }
            if (!CanWriteAggregateLvalue(lhs, ctx))
                return true;

            int size = GetStorageSize(leftType);
            if (size <= 0)
            {
                Program.Error(Maybe.Just(assignExpr.Source), ErrorCode.IncompleteType,
                    "incomplete struct/union assignment size for {0}", leftType == null ? "<unknown>" : leftType.Show());
                return true;
            }

            int[] dst = AcquireTemps(2);
            try
            {
                int ignored;
                if (!EmitAddressIntoPair(lhs, ctx, dst[0], dst[1], out ignored))
                    return true;

                CType callRet;
                if (TryGetAggregateReturnCallType(rhs, ctx, out callRet))
                {
                    EmitAggregateCallToAddress(rhs, ctx, dst[0], dst[1], leftType);
                    return true;
                }

                int[] src = AcquireTemps(2);
                try
                {
                    if (!EmitAggregateSourceAddressIntoPair(rhs, ctx, src[0], src[1], leftType))
                        return true;
                    EmitCopyBytesFromPtrToPtr(dst[0], dst[1], src[0], src[1], size, assignExpr.Source);
                }
                finally
                {
                    ReleaseTemps(src);
                }
            }
            finally
            {
                ReleaseTemps(dst);
            }
            return true;
        }

        // Validate the result type, recover the saved return destination and forward an aggregate call or copy an existing value into it.
        void EmitAggregateReturn(Expr returnExpr, FunctionContext ctx)
        {
            CType valueType;
            if (!TryGetExprType(returnExpr, ctx, out valueType) || !IsSameCompleteAggregateType(ctx.ReturnType, valueType))
            {
                Program.Error(Maybe.Just(returnExpr.Source), ErrorCode.ParseError,
                    "incompatible struct/union return: expected {0}, got {1}",
                    ctx.ReturnType == null ? "<unknown>" : ctx.ReturnType.Show(),
                    valueType == null ? "<unknown>" : valueType.Show());
                return;
            }

            int[] dst = AcquireTemps(2);
            try
            {
                if (!EmitSavedStructReturnPointerIntoPair(ctx, dst[0], dst[1], returnExpr.Source))
                    return;

                CType callRet;
                if (TryGetAggregateReturnCallType(returnExpr, ctx, out callRet))
                {
                    EmitAggregateCallToAddress(returnExpr, ctx, dst[0], dst[1], ctx.ReturnType);
                    return;
                }

                int[] src = AcquireTemps(2);
                try
                {
                    if (!EmitAggregateSourceAddressIntoPair(returnExpr, ctx, src[0], src[1], ctx.ReturnType))
                        return;
                    EmitCopyBytesFromPtrToPtr(dst[0], dst[1], src[0], src[1], GetStorageSize(ctx.ReturnType), returnExpr.Source);
                }
                finally
                {
                    ReleaseTemps(src);
                }
            }
            finally
            {
                ReleaseTemps(dst);
            }
        }

        // Reject readonly named storage and recurse through fields; pointer/index forms are accepted here without pointee-const validation.
        bool CanWriteAggregateLvalue(Expr lvalue, FunctionContext ctx)
        {
            if (lvalue == null) return false;
            string name;
            StorageSlot slot;
            Expr sub;
            Expr baseExpr;
            Expr indexExpr;
            if (lvalue.Match(Tag.Name, out name))
            {
                if (!TryResolveStorage(ctx, name, out slot))
                {
                    Program.Error("error KQ0000: unknown aggregate lvalue for --target=nes: {0}", name);
                    return false;
                }
                if (slot.IsConstant || slot.IsReadonlyData)
                {
                    Program.Error("error KQ0000: cannot assign to read-only aggregate '{0}' in --target=nes.", name);
                    return false;
                }
                return true;
            }
            if (lvalue.Match(Tag.Field, out baseExpr, out string _fieldName))
                return CanWriteAggregateLvalue(baseExpr, ctx);
            if (lvalue.Match(Tag.Load, out sub))
                return true;
            if (lvalue.Match(Tag.Index, out baseExpr, out indexExpr))
                return true;
            Program.Error("error KQ0000: unsupported aggregate lvalue in --target=nes.");
            return false;
        }

        // Materialize aggregate call results in newly allocated RAM, or take an existing aggregate source address.
        bool EmitAggregateSourceAddressIntoPair(Expr expr, FunctionContext ctx, int dstLo, int dstHi, CType expectedType)
        {
            CType retType;
            if (TryGetAggregateReturnCallType(expr, ctx, out retType))
            {
                if (!IsSameCompleteAggregateType(expectedType, retType))
                {
                    Program.Error(Maybe.Just(expr.Source), ErrorCode.ParseError,
                        "incompatible struct/union temporary source: expected {0}, got {1}",
                        expectedType == null ? "<unknown>" : expectedType.Show(),
                        retType == null ? "<unknown>" : retType.Show());
                    return false;
                }

                int tempAddr = AllocateAggregateTemp(expr, retType);
                EmitSetGlobalStructReturnPointerToAddress(tempAddr);
                EmitCallExpression(expr, ctx, expectedReturnSize: 0);
                EmitLoadImmediateAddressIntoPair(tempAddr, dstLo, dstHi);
                return true;
            }

            int ignored;
            return EmitAddressIntoPair(expr, ctx, dstLo, dstHi, out ignored);
        }

        // Allocate result storage and invoke an aggregate-returning call even when its value is discarded.
        void EmitAggregateCallToTemp(Expr callExpr, FunctionContext ctx, CType retType)
        {
            int tempAddr = AllocateAggregateTemp(callExpr, retType);
            EmitSetGlobalStructReturnPointerToAddress(tempAddr);
            EmitCallExpression(callExpr, ctx, expectedReturnSize: 0);
        }

        // Check the expected aggregate result type, publish the caller destination pointer and emit the call.
        void EmitAggregateCallToAddress(Expr callExpr, FunctionContext ctx, int dstLo, int dstHi, CType expectedType)
        {
            CType retType;
            if (!TryGetAggregateReturnCallType(callExpr, ctx, out retType))
            {
                Program.Error(Maybe.Just(callExpr.Source), ErrorCode.ParseError, "struct/union return call expected");
                return;
            }
            if (!IsSameCompleteAggregateType(expectedType, retType))
            {
                Program.Error(Maybe.Just(callExpr.Source), ErrorCode.ParseError,
                    "incompatible struct/union call result: expected {0}, got {1}",
                    expectedType == null ? "<unknown>" : expectedType.Show(),
                    retType == null ? "<unknown>" : retType.Show());
                return;
            }

            EmitSetGlobalStructReturnPointerFromPair(dstLo, dstHi);
            EmitCallExpression(callExpr, ctx, expectedReturnSize: 0);
        }

        // Unpack a call expression and delegate argument/result handling, reporting malformed call nodes.
        void EmitCallExpression(Expr callExpr, FunctionContext ctx, int expectedReturnSize)
        {
            Expr funcExpr;
            Expr[] args;
            if (!callExpr.MatchAny(Tag.Call, out funcExpr, out args))
            {
                Program.Error(Maybe.Just(callExpr.Source), ErrorCode.ParseError, "call expression expected");
                return;
            }
            EmitCall(callExpr.Source, funcExpr, args, ctx, expectedReturnSize);
        }

        // Recognize call expressions whose inferred result is an aggregate.
        bool TryGetAggregateReturnCallType(Expr expr, FunctionContext ctx, out CType retType)
        {
            retType = null;
            Expr funcExpr;
            Expr[] args;
            if (!expr.MatchAny(Tag.Call, out funcExpr, out args)) return false;
            return TryGetExprType(expr, ctx, out retType) && IsAggregateType(retType);
        }

        // Copy the function-local saved return destination into the supplied pointer-byte slots, diagnosing a missing destination.
        bool EmitSavedStructReturnPointerIntoPair(FunctionContext ctx, int dstLo, int dstHi, FilePosition source)
        {
            if (ctx == null || ctx.StructReturnPointerSlot == null)
            {
                Program.Error(Maybe.Just(source), ErrorCode.ParseError, "missing struct/union return destination");
                return false;
            }
            EmitAsm("LDA", Mem(ctx.StructReturnPointerSlot.Address));
            EmitAsm("STA", Mem(dstLo));
            EmitAsm("LDA", Mem(ctx.StructReturnPointerSlot.Address + 1));
            EmitAsm("STA", Mem(dstHi));
            return true;
        }

        // Publish an existing low/high pointer pair in the shared aggregate-return ABI slots.
        void EmitSetGlobalStructReturnPointerFromPair(int srcLo, int srcHi)
        {
            EmitAsm("LDA", Mem(srcLo));
            EmitAsm("STA", Mem(_runtimeSretPtrLoAddress));
            EmitAsm("LDA", Mem(srcHi));
            EmitAsm("STA", Mem(_runtimeSretPtrHiAddress));
        }

        // Publish a constant destination address, low byte first, in the shared aggregate-return ABI slots.
        void EmitSetGlobalStructReturnPointerToAddress(int address)
        {
            EmitAsm("LDA", Imm(address & 0xFF));
            EmitAsm("STA", Mem(_runtimeSretPtrLoAddress));
            EmitAsm("LDA", Imm((address >> 8) & 0xFF));
            EmitAsm("STA", Mem(_runtimeSretPtrHiAddress));
        }

        // Store the masked low/high bytes of a constant address in the requested temporary pair.
        void EmitLoadImmediateAddressIntoPair(int address, int dstLo, int dstHi)
        {
            EmitAsm("LDA", Imm(address & 0xFF));
            EmitAsm("STA", Mem(dstLo));
            EmitAsm("LDA", Imm((address >> 8) & 0xFF));
            EmitAsm("STA", Mem(dstHi));
        }

        // Reserve at least one byte of global RAM for a fresh aggregate result; storage is not reused after the expression.
        int AllocateAggregateTemp(Expr origin, CType type)
        {
            int size = GetStorageSize(type);
            if (size <= 0) size = 1;
            string name = "__kq_sret_tmp_" + (++_structReturnTempCounter).ToString();
            if (_nextGlobalRam + size > GlobalRamLimitExclusive)
            {
                Program.Error(Maybe.Just(origin.Source), ErrorCode.ParseError,
                    "not enough RAM for struct/union temporary of {0} bytes", size);
                return GlobalRamBase;
            }
            int address = _nextGlobalRam;
            _nextGlobalRam += size;
            _globals[name] = new StorageSlot
            {
                Name = name,
                Type = type,
                Address = address,
                Size = size,
                Source = origin.Source,
            };
            RecordRamAllocation(name, "aggregate_temp", address, size, false);
            return address;
        }

        // Emit an unrolled forward copy of at most 256 bytes through zero-page indirect pointers.
        // The high-byte arguments are implicit in the adjacent pointer layout; overlapping ranges are not handled as memmove.
        void EmitCopyBytesFromPtrToPtr(int dstLo, int dstHi, int srcLo, int srcHi, int size, FilePosition source)
        {
            if (size <= 0) return;
            if (size > 256)
            {
                Program.Error(Maybe.Just(source), ErrorCode.ParseError,
                    "struct/union copy larger than 256 bytes is not supported in --target=nes yet ({0} bytes)", size);
                return;
            }
            for (int i = 0; i < size; i++)
            {
                EmitAsm("LDY", Imm(i & 0xFF));
                EmitAsm("LDA", IndY(srcLo));
                EmitAsm("STA", IndY(dstLo));
            }
        }

        // Copy at most 256 indexed source bytes into a checked slice of compiler temporary slots.
        void EmitCopyBytesFromPtrToTempBytes(int srcLo, int srcHi, int[] tempBytes, int offset, int size, FilePosition source)
        {
            if (size <= 0) return;
            if (size > 256 || offset < 0 || offset + size > tempBytes.Length)
            {
                Program.Error(Maybe.Just(source), ErrorCode.ParseError,
                    "invalid struct/union argument copy size ({0} bytes)", size);
                return;
            }
            for (int i = 0; i < size; i++)
            {
                EmitAsm("LDY", Imm(i & 0xFF));
                EmitAsm("LDA", IndY(srcLo));
                EmitAsm("STA", Mem(tempBytes[offset + i]));
            }
        }

        // Resolve a scalar destination width, rejecting whole arrays/aggregates and normalizing other storage widths to one or two bytes.
        int GetLvalueSize(Expr lvalue, FunctionContext ctx, bool requireWritable)
        {
            CType type;
            if (!TryGetLvalueType(lvalue, ctx, requireWritable, out type))
                return 1;

            if (type != null && type.IsArray)
            {
                Program.Error("error KQ0000: whole-array lvalue operations are not yet supported in --target=nes.");
                return 1;
            }
            if (type != null && type.IsStructOrUnion)
            {
                Program.Error("error KQ0000: whole-struct lvalue requires aggregate copy handling in --target=nes.");
                return 1;
            }
            int size = GetStorageSize(type);
            size = NormalizeScalarSize(size);
            if (size != 1 && size != 2)
            {
                Program.Error("error KQ0000: --target=nes phase 6 currently supports only 1-byte or 2-byte scalar lvalues.");
                return 1;
            }
            return size;
        }

        // Infer storage/ABI types, preferring unsigned width classifications for foldable constants.
        // Individual operator cases supply this backend's result rules and may not preserve source signedness.
        bool TryGetExprType(Expr expr, FunctionContext ctx, out CType type)
        {
            type = null;
            if (expr == null) return false;

            int constValue;
            if (TryEvaluateConstant(expr, out constValue))
            {
                type = ConstantStorageSize(constValue) >= 2 ? CType.UInt16 : CType.UInt8;
                return true;
            }

            string name;
            StorageSlot slot;
            Expr sub;
            Expr lhs;
            Expr rhs;
            Expr[] args;
            Expr funcExpr;
            CType castType;
            if (expr.Match(Tag.Name, out name))
            {
                if (TryResolveStorage(ctx, name, out slot))
                {
                    type = slot.Type;
                    return true;
                }
                return false;
            }
            if (expr.Match(Tag.Cast, out castType, out sub))
            {
                type = castType;
                return true;
            }
            if (expr.Match(Tag.AddressOf, out sub))
            {
                CType lvType;
                if (TryGetLvalueType(sub, ctx, requireWritable: false, out lvType))
                {
                    type = CType.MakePointer(lvType);
                    return true;
                }
                return false;
            }
            if (expr.Match(Tag.Load, out sub))
            {
                CType ptrType;
                if (TryGetExprType(sub, ctx, out ptrType) && (ptrType.IsPointer || ptrType.IsArray) && ptrType.Subtype != null)
                {
                    type = ptrType.Subtype;
                    return true;
                }
                return false;
            }
            if (expr.Match(Tag.Index, out lhs, out rhs))
            {
                CType baseType;
                if (TryGetExprType(lhs, ctx, out baseType) && (baseType.IsPointer || baseType.IsArray) && baseType.Subtype != null)
                {
                    type = baseType.Subtype;
                    return true;
                }
                return false;
            }
            string fieldName;
            if (expr.Match(Tag.Field, out sub, out fieldName))
            {
                FieldInfo field;
                if (TryResolveField(sub, fieldName, ctx, out field))
                {
                    type = field.Type;
                    return true;
                }
                return false;
            }
            if (expr.Match(Tag.Assign, out lhs, out rhs))
                return TryGetLvalueType(lhs, ctx, requireWritable: false, out type);
            if (expr.Match(Tag.AssignModify, out string _opName, out lhs, out rhs))
                return TryGetLvalueType(lhs, ctx, requireWritable: false, out type);
            if (expr.MatchAny(Tag.Call, out funcExpr, out args))
            {
                string funcName;
                if (funcExpr.Match(Tag.Name, out funcName))
                {
                    if (string.Equals(funcName, "__bankof", StringComparison.Ordinal))
                    {
                        type = CType.UInt8;
                        return true;
                    }
                    if (string.Equals(funcName, "__bankswitch", StringComparison.Ordinal))
                    {
                        type = CType.Void;
                        return true;
                    }
                    if (string.Equals(funcName, "__assert", StringComparison.Ordinal))
                    {
                        type = CType.Void;
                        return true;
                    }
                    if (string.Equals(funcName, "__farcall", StringComparison.Ordinal))
                    {
                        if (args != null && args.Length == 2 && args[1].Match(Tag.Name, out string farTargetName))
                        {
                            if (_functionReturnTypes.TryGetValue(farTargetName, out type))
                                return true;
                            if (TryGetCompatibilityIntrinsicSignature(farTargetName, out IntrinsicSignature farSig))
                            {
                                type = farSig.ReturnType;
                                return true;
                            }
                        }
                        type = CType.UInt8;
                        return true;
                    }
                    if (TryGetCompatibilityIntrinsicSignature(funcName, out IntrinsicSignature signature))
                    {
                        type = signature.ReturnType;
                        return true;
                    }
                    if (_functionReturnTypes.TryGetValue(funcName, out type))
                        return true;
                }
                type = CType.UInt8;
                return true;
            }
            if (expr.Match(Tag.PreIncrement, out sub) || expr.Match(Tag.PostIncrement, out sub) || expr.Match(Tag.PreDecrement, out sub) || expr.Match(Tag.PostDecrement, out sub))
                return TryGetLvalueType(sub, ctx, requireWritable: false, out type);
            if (expr.Match(Tag.Add, out lhs, out rhs) || expr.Match(Tag.Subtract, out lhs, out rhs))
            {
                CType leftType;
                CType rightType;
                if (TryGetExprType(lhs, ctx, out leftType) && leftType != null && leftType.IsPointer)
                {
                    type = leftType;
                    return true;
                }
                if (TryGetExprType(rhs, ctx, out rightType) && rightType != null && rightType.IsPointer)
                {
                    type = rightType;
                    return true;
                }
                type = NormalizeScalarSize(Math.Max(DetermineExprSize(lhs, ctx), DetermineExprSize(rhs, ctx))) >= 2 ? CType.UInt16 : CType.UInt8;
                return true;
            }
            if (expr.Match(Tag.Multiply, out lhs, out rhs) || expr.Match(Tag.Divide, out lhs, out rhs) || expr.Match(Tag.Modulus, out lhs, out rhs))
            {
                CType leftType;
                CType rightType;
                bool haveLeft = TryGetExprType(lhs, ctx, out leftType);
                bool haveRight = TryGetExprType(rhs, ctx, out rightType);
                if ((haveLeft && IsIntegerLike(leftType)) || (haveRight && IsIntegerLike(rightType)))
                {
                    type = PromoteIntegerBinaryType(leftType, rightType);
                    return true;
                }

                type = CType.UInt16;
                return true;
            }
            Expr condExpr;
            Expr trueExpr;
            Expr falseExpr;
            if (expr.Match(Tag.Conditional, out condExpr, out trueExpr, out falseExpr))
            {
                int size = NormalizeScalarSize(Math.Max(DetermineExprSize(trueExpr, ctx), DetermineExprSize(falseExpr, ctx)));
                type = size >= 2 ? CType.UInt16 : CType.UInt8;
                return true;
            }
            if (expr.Match(Tag.BitwiseAnd, out lhs, out rhs) || expr.Match(Tag.BitwiseOr, out lhs, out rhs) || expr.Match(Tag.BitwiseXor, out lhs, out rhs))
            {
                type = NormalizeScalarSize(Math.Max(DetermineExprSize(lhs, ctx), DetermineExprSize(rhs, ctx))) >= 2 ? CType.UInt16 : CType.UInt8;
                return true;
            }
            if (expr.Match(Tag.BitwiseNot, out sub))
            {
                int sz = DetermineExprSize(sub, ctx);
                type = sz >= 2 ? CType.UInt16 : CType.UInt8;
                return true;
            }
            if (expr.Match(Tag.LogicalNot, out sub) || expr.Match(Tag.LogicalAnd, out lhs, out rhs) || expr.Match(Tag.LogicalOr, out lhs, out rhs) || IsComparisonTag(expr.Tag))
            {
                type = CType.UInt8;
                return true;
            }
            return false;
        }

        // Resolve named, dereferenced, indexed or field lvalue types; direct readonly/array names return false even for read-only queries.
        bool TryGetLvalueType(Expr lvalue, FunctionContext ctx, bool requireWritable, out CType type)
        {
            type = null;
            if (lvalue == null) return false;

            string name;
            StorageSlot slot;
            Expr sub;
            Expr baseExpr;
            Expr indexExpr;
            if (lvalue.Match(Tag.Name, out name))
            {
                if (!TryResolveStorage(ctx, name, out slot))
                {
                    Program.Error("error KQ0000: unknown symbol in lvalue for --target=nes: {0}", name);
                    return false;
                }
                if (slot.IsConstant || slot.IsReadonlyData)
                {
                    if (requireWritable)
                        Program.Error("error KQ0000: cannot assign to read-only symbol '{0}' in --target=nes phase 6.", name);
                    return false;
                }
                if (slot.Type != null && slot.Type.IsArray)
                {
                    if (requireWritable)
                        Program.Error("error KQ0000: whole-array assignment is not yet supported in --target=nes: {0}", name);
                    return false;
                }
                type = slot.Type;
                return true;
            }
            if (lvalue.Match(Tag.Load, out sub))
            {
                CType ptrType;
                if (TryGetExprType(sub, ctx, out ptrType) && (ptrType.IsPointer || ptrType.IsArray) && ptrType.Subtype != null)
                {
                    type = ptrType.Subtype;
                    return true;
                }
                Program.Error("error KQ0000: --target=nes phase 5 expected a pointer in unary '*' lvalue.");
                return false;
            }
            if (lvalue.Match(Tag.Index, out baseExpr, out indexExpr))
            {
                CType baseType;
                if (TryGetExprType(baseExpr, ctx, out baseType) && (baseType.IsPointer || baseType.IsArray) && baseType.Subtype != null)
                {
                    type = baseType.Subtype;
                    return true;
                }
                Program.Error("error KQ0000: --target=nes phase 6 expected a pointer/array base in index expression.");
                return false;
            }
            string fieldName2;
            if (lvalue.Match(Tag.Field, out sub, out fieldName2))
            {
                FieldInfo field;
                if (TryResolveField(sub, fieldName2, ctx, out field))
                {
                    type = field.Type;
                    return true;
                }
                return false;
            }

            if (requireWritable)
                Program.Error("error KQ0000: --target=nes phase 6 assignment currently supports scalar names, '*ptr', 'ptr[index]', and field lvalues.");
            return false;
        }

        // Store A/X into named or computed storage, saving value bytes while computing an indirect address.
        // When requested, reload the stored scalar result and release all temporary slots on every exit.
        void EmitStoreLoadedValueToLvalue(Expr lhs, FunctionContext ctx, int size, bool preserveResult)
        {
            string name;
            if (lhs.Match(Tag.Name, out name))
            {
                StorageSlot slot;
                if (!TryResolveStorage(ctx, name, out slot))
                {
                    Program.Error("error KQ0000: unknown symbol in assignment for --target=nes: {0}", name);
                    return;
                }
                if (slot.IsConstant || slot.IsReadonlyData)
                {
                    Program.Error("error KQ0000: cannot assign to read-only symbol '{0}' in --target=nes phase 6.", name);
                    return;
                }
                StoreToSlot(slot);
                if (size == 2 && slot.Size == 1)
                    EmitAsm("LDX", Imm(0));
                return;
            }

            if (size <= 1 && TryEmitDirectByteIndexedStore(lhs, ctx, preserveResult))
                return;

            int valueLoTemp = AcquireTemp();
            int valueHiTemp = size > 1 ? AcquireTemp() : -1;
            int[] ptrTemps = AcquireTemps(2);
            try
            {
                EmitAsm("STA", Mem(valueLoTemp));
                if (size > 1)
                    EmitAsm("STX", Mem(valueHiTemp));

                int valueSize;
                if (!EmitAddressIntoPair(lhs, ctx, ptrTemps[0], ptrTemps[1], out valueSize))
                    return;

                EmitAsm("LDY", Imm(0));
                if (size <= 1)
                {
                    EmitAsm("LDA", Mem(valueLoTemp));
                    EmitAsm("STA", IndY(ptrTemps[0]));
                    if (preserveResult)
                        EmitAsm("LDA", Mem(valueLoTemp));
                    return;
                }

                EmitAsm("LDA", Mem(valueLoTemp));
                EmitAsm("STA", IndY(ptrTemps[0]));
                EmitAsm("INY");
                EmitAsm("LDA", Mem(valueHiTemp));
                EmitAsm("STA", IndY(ptrTemps[0]));
                if (preserveResult)
                {
                    EmitAsm("LDA", Mem(valueLoTemp));
                    EmitAsm("LDX", Mem(valueHiTemp));
                }
            }
            finally
            {
                ReleaseTemps(ptrTemps);
                if (valueHiTemp >= 0) ReleaseTemp(valueHiTemp);
                ReleaseTemp(valueLoTemp);
            }
        }

        // Resolve an addressable expression into caller-owned low/high temporary bytes.
        // The reported size is scalar-normalized except for an aggregate call result.
        bool EmitAddressIntoPair(Expr lvalue, FunctionContext ctx, int dstLo, int dstHi, out int valueSize)
        {
            valueSize = 1;

            string name;
            StorageSlot slot;
            Expr sub;
            Expr baseExpr;
            Expr indexExpr;
            CType callRetType;
            if (TryGetAggregateReturnCallType(lvalue, ctx, out callRetType))
            {
                // Give the callee backing storage for its aggregate result before forming the address.
                int tempAddr = AllocateAggregateTemp(lvalue, callRetType);
                EmitSetGlobalStructReturnPointerToAddress(tempAddr);
                EmitCallExpression(lvalue, ctx, expectedReturnSize: 0);
                EmitLoadImmediateAddressIntoPair(tempAddr, dstLo, dstHi);
                valueSize = GetStorageSize(callRetType);
                return true;
            }
            if (lvalue.Match(Tag.Name, out name))
            {
                if (!TryResolveStorage(ctx, name, out slot) || slot.IsConstant)
                {
                    Program.Error("error KQ0000: cannot take address of '{0}' for --target=nes phase 6.", name);
                    return false;
                }
                // ROM data uses assembler label relocations; writable storage has a fixed CPU address.
                valueSize = NormalizeScalarSize(slot.Size);
                if (slot.IsReadonlyData)
                {
                    EmitAsm("LDA", ImmLo(name));
                    EmitAsm("STA", Mem(dstLo));
                    EmitAsm("LDA", ImmHi(name));
                    EmitAsm("STA", Mem(dstHi));
                }
                else
                {
                    EmitAsm("LDA", Imm(slot.Address & 0xFF));
                    EmitAsm("STA", Mem(dstLo));
                    EmitAsm("LDA", Imm((slot.Address >> 8) & 0xFF));
                    EmitAsm("STA", Mem(dstHi));
                }
                return true;
            }
            if (lvalue.Match(Tag.Load, out sub))
            {
                CType ptrType;
                if (!TryGetExprType(sub, ctx, out ptrType) || (!ptrType.IsPointer && !ptrType.IsArray) || ptrType.Subtype == null)
                {
                    Program.Error("error KQ0000: --target=nes phase 6 expected a pointer in unary '*' expression.");
                    return false;
                }
                // Dereferencing for an address evaluates the pointer itself, without loading its pointee.
                valueSize = NormalizeScalarSize(GetStorageSize(ptrType.Subtype));
                EmitLoadValue(sub, ctx, 2);
                EmitAsm("STA", Mem(dstLo));
                EmitAsm("STX", Mem(dstHi));
                return true;
            }
            if (lvalue.Match(Tag.Index, out baseExpr, out indexExpr))
            {
                CType baseType;
                if (!TryGetExprType(baseExpr, ctx, out baseType) || (!baseType.IsPointer && !baseType.IsArray) || baseType.Subtype == null)
                {
                    Program.Error("error KQ0000: --target=nes phase 6 expected a pointer/array base in index expression.");
                    return false;
                }

                valueSize = NormalizeScalarSize(GetStorageSize(baseType.Subtype));
                // Use full element storage size for indexing, including aggregates larger than a word.
                int elemSize = GetPointerElementStride(baseType);
                EmitLoadValue(baseExpr, ctx, 2);
                EmitAsm("STA", Mem(dstLo));
                EmitAsm("STX", Mem(dstHi));

                int constIndex;
                if (TryEvaluateConstant(indexExpr, out constIndex))
                {
                    // Fold a constant element offset into the 16-bit CPU address.
                    EmitAddConstantToPair(dstLo, dstHi, (constIndex * elemSize) & 0xFFFF);
                    return true;
                }

                // The dynamic path currently zero-extends byte indices before scaling.
                // There is no emitted array-bounds check in this address calculation.
                int indexSize = NormalizeScalarSize(DetermineExprSize(indexExpr, ctx));
                int[] idxTemps = AcquireTemps(2);
                try
                {
                    EmitLoadValue(indexExpr, ctx, indexSize);
                    EmitAsm("STA", Mem(idxTemps[0]));
                    if (indexSize == 2) EmitAsm("STX", Mem(idxTemps[1]));
                    else
                    {
                        EmitAsm("LDA", Imm(0));
                        EmitAsm("STA", Mem(idxTemps[1]));
                    }

                    EmitScalePairByConstant(idxTemps[0], idxTemps[1], elemSize);

                    EmitAddPairToPair(dstLo, dstHi, idxTemps[0], idxTemps[1]);
                    return true;
                }
                finally
                {
                    ReleaseTemps(idxTemps);
                }
            }
            string fieldName;
            if (lvalue.Match(Tag.Field, out baseExpr, out fieldName))
            {
                FieldInfo field;
                if (!TryResolveField(baseExpr, fieldName, ctx, out field))
                    return false;
                // Recursively address the containing object, then add the field layout offset.
                int fieldSize = NormalizeScalarSize(GetStorageSize(field.Type));
                if (!EmitAddressIntoPair(baseExpr, ctx, dstLo, dstHi, out valueSize))
                    return false;
                valueSize = fieldSize;
                if (field.Offset != 0)
                    EmitAddConstantToPair(dstLo, dstHi, field.Offset & 0xFFFF);
                return true;
            }

            Program.Error("error KQ0000: --target=nes phase 6 address computation currently supports names, '*ptr', 'ptr[index]', and field expressions.");
            return false;
        }

        // Load an lvalue into A, or A/X for a word; prefer direct byte-array addressing.
        void EmitLoadIndirectValue(Expr lvalue, FunctionContext ctx, int expectedSize)
        {
            if (TryEmitDirectByteIndexedLoad(lvalue, ctx))
            {
                if (expectedSize > 1)
                    EmitAsm("LDX", Imm(0));
                return;
            }

            int[] ptrTemps = AcquireTemps(2);
            try
            {
                int valueSize;
                if (!EmitAddressIntoPair(lvalue, ctx, ptrTemps[0], ptrTemps[1], out valueSize))
                    return;

                // An explicit requested width determines how many adjacent bytes are loaded here.
                int size = NormalizeScalarSize(expectedSize > 0 ? expectedSize : valueSize);
                if (size <= 1)
                {
                    EmitAsm("LDY", Imm(0));
                    EmitAsm("LDA", IndY(ptrTemps[0]));
                    return;
                }

                int lowTemp = AcquireTemp();
                try
                {
                    EmitAsm("LDY", Imm(0));
                    EmitAsm("LDA", IndY(ptrTemps[0]));
                    EmitAsm("STA", Mem(lowTemp));
                    EmitAsm("INY");
                    EmitAsm("LDA", IndY(ptrTemps[0]));
                    EmitAsm("TAX");
                    EmitAsm("LDA", Mem(lowTemp));
                }
                finally
                {
                    ReleaseTemp(lowTemp);
                }
            }
            finally
            {
                ReleaseTemps(ptrTemps);
            }
        }

        // Return the computed CPU address in A/X and release the temporary pointer pair.
        void EmitAddressOfValue(Expr lvalue, FunctionContext ctx)
        {
            int[] ptrTemps = AcquireTemps(2);
            try
            {
                int ignored;
                if (!EmitAddressIntoPair(lvalue, ctx, ptrTemps[0], ptrTemps[1], out ignored))
                    return;
                EmitAsm("LDA", Mem(ptrTemps[0]));
                EmitAsm("LDX", Mem(ptrTemps[1]));
            }
            finally
            {
                ReleaseTemps(ptrTemps);
            }
        }

        // Recognize named arrays with one-byte elements for absolute or absolute-Y access.
        // Only constant indices are range-checked here; other forms use the general path.
        bool TryResolveDirectByteIndex(
            Expr indexed,
            FunctionContext ctx,
            bool requireWritable,
            out StorageSlot slot,
            out Expr indexExpr,
            out int constantIndex)
        {
            slot = null;
            indexExpr = null;
            constantIndex = -1;
            Expr baseExpr;
            if (indexed == null || !indexed.Match(Tag.Index, out baseExpr, out indexExpr))
                return false;
            string baseName;
            if (baseExpr == null || !baseExpr.Match(Tag.Name, out baseName))
                return false;
            if (!TryResolveStorage(ctx, baseName, out slot) || slot == null || slot.Type == null || !slot.Type.IsArray)
                return false;
            if (slot.Type.Subtype == null || GetStorageSize(slot.Type.Subtype) != 1)
                return false;
            if (requireWritable && (slot.IsConstant || slot.IsReadonlyData))
                return false;
            if (TryEvaluateConstant(indexExpr, out constantIndex))
                return constantIndex >= 0 && constantIndex < slot.Size;
            constantIndex = -1;
            return NormalizeScalarSize(DetermineExprSize(indexExpr, ctx)) == 1;
        }

        // Read a constant array slot directly, or evaluate a byte index once into Y.
        bool TryEmitDirectByteIndexedLoad(Expr indexed, FunctionContext ctx)
        {
            StorageSlot slot;
            Expr indexExpr;
            int constantIndex;
            if (!TryResolveDirectByteIndex(indexed, ctx, false, out slot, out indexExpr, out constantIndex))
                return false;
            if (constantIndex >= 0)
            {
                EmitAsm(
                    "LDA",
                    slot.IsReadonlyData
                        ? new AsmOperand(slot.Name, constantIndex, AddressMode.Absolute, ImmediateModifier.None)
                        : Mem(slot.Address + constantIndex));
                return true;
            }
            EmitLoadValue(indexExpr, ctx, 1);
            EmitAsm("TAY");
            EmitAsm(
                "LDA",
                slot.IsReadonlyData
                    ? new AsmOperand(slot.Name, AddressMode.AbsoluteY)
                    : new AsmOperand(slot.Address, AddressMode.AbsoluteY));
            return true;
        }

        // Preserve the value in A while evaluating a dynamic array index, then store it.
        bool TryEmitDirectByteIndexedStore(Expr indexed, FunctionContext ctx, bool preserveResult)
        {
            StorageSlot slot;
            Expr indexExpr;
            int constantIndex;
            if (!TryResolveDirectByteIndex(indexed, ctx, true, out slot, out indexExpr, out constantIndex))
                return false;
            if (constantIndex >= 0)
            {
                EmitAsm("STA", Mem(slot.Address + constantIndex));
                return true;
            }
            int valueTemp = AcquireTemp();
            try
            {
                EmitAsm("STA", Mem(valueTemp));
                EmitLoadValue(indexExpr, ctx, 1);
                EmitAsm("TAY");
                EmitAsm("LDA", Mem(valueTemp));
                EmitAsm("STA", new AsmOperand(slot.Address, AddressMode.AbsoluteY));
                if (preserveResult)
                    EmitAsm("LDA", Mem(valueTemp));
                return true;
            }
            finally
            {
                ReleaseTemp(valueTemp);
            }
        }

        // Add a constant modulo 65536, propagating low-byte carry when needed.
        // A page-aligned offset can update the high byte alone.
        void EmitAddConstantToPair(int dstLo, int dstHi, int value)
        {
            value &= 0xFFFF;
            int lo = value & 0xFF;
            int hi = (value >> 8) & 0xFF;
            if (lo != 0)
            {
                EmitAsm("CLC");
                EmitAsm("LDA", Mem(dstLo));
                EmitAsm("ADC", Imm(lo));
                EmitAsm("STA", Mem(dstLo));
                EmitAsm("LDA", Mem(dstHi));
                EmitAsm("ADC", Imm(hi));
                EmitAsm("STA", Mem(dstHi));
            }
            else if (hi != 0)
            {
                EmitAsm("CLC");
                EmitAsm("LDA", Mem(dstHi));
                EmitAsm("ADC", Imm(hi));
                EmitAsm("STA", Mem(dstHi));
            }
        }

        // Subtract a word constant with the 6502 carry-as-no-borrow convention.
        void EmitSubConstantFromPair(int dstLo, int dstHi, int value)
        {
            value &= 0xFFFF;
            int lo = value & 0xFF;
            int hi = (value >> 8) & 0xFF;
            EmitAsm("SEC");
            EmitAsm("LDA", Mem(dstLo));
            EmitAsm("SBC", Imm(lo));
            EmitAsm("STA", Mem(dstLo));
            EmitAsm("LDA", Mem(dstHi));
            EmitAsm("SBC", Imm(hi));
            EmitAsm("STA", Mem(dstHi));
        }

        // Add low bytes first, carrying overflow into the high-byte addition.
        void EmitAddPairToPair(int dstLo, int dstHi, int srcLo, int srcHi)
        {
            EmitAsm("CLC");
            EmitAsm("LDA", Mem(dstLo));
            EmitAsm("ADC", Mem(srcLo));
            EmitAsm("STA", Mem(dstLo));
            EmitAsm("LDA", Mem(dstHi));
            EmitAsm("ADC", Mem(srcHi));
            EmitAsm("STA", Mem(dstHi));
        }

        // Subtract low bytes first, propagating their borrow into the high-byte subtraction.
        void EmitSubPairFromPair(int dstLo, int dstHi, int srcLo, int srcHi)
        {
            EmitAsm("SEC");
            EmitAsm("LDA", Mem(dstLo));
            EmitAsm("SBC", Mem(srcLo));
            EmitAsm("STA", Mem(dstLo));
            EmitAsm("LDA", Mem(dstHi));
            EmitAsm("SBC", Mem(srcHi));
            EmitAsm("STA", Mem(dstHi));
        }

        // Derive the byte stride from the complete pointee type; unknown types use one byte.
        int GetPointerElementStride(CType pointerLikeType)
        {
            if (pointerLikeType == null) return 1;
            if (!(pointerLikeType.IsPointer || pointerLikeType.IsArray) || pointerLikeType.Subtype == null) return 1;
            return Math.Max(1, GetStorageSize(pointerLikeType.Subtype));
        }

        // Scale a 16-bit offset by a positive constant, retaining the low 16 result bits.
        void EmitScalePairByConstant(int dstLo, int dstHi, int factor)
        {
            factor = Math.Max(1, factor);
            if (factor == 1) return;

            // Power-of-two strides need only carry-propagating shifts.
            if ((factor & (factor - 1)) == 0)
            {
                while (factor > 1)
                {
                    EmitAsm("ASL", Mem(dstLo));
                    EmitAsm("ROL", Mem(dstHi));
                    factor >>= 1;
                }
                return;
            }

            int[] temps = AcquireTemps(2);
            try
            {
                EmitAsm("LDA", Mem(dstLo));
                EmitAsm("STA", Mem(temps[0]));
                EmitAsm("LDA", Mem(dstHi));
                EmitAsm("STA", Mem(temps[1]));

                EmitAsm("LDA", Imm(0));
                EmitAsm("STA", Mem(dstLo));
                EmitAsm("STA", Mem(dstHi));

                // For other strides, accumulate the selected powers of two from a saved multiplicand.
                int remaining = factor;
                while (remaining != 0)
                {
                    if ((remaining & 1) != 0)
                        EmitAddPairToPair(dstLo, dstHi, temps[0], temps[1]);

                    remaining >>= 1;
                    if (remaining != 0)
                    {
                        EmitAsm("ASL", Mem(temps[0]));
                        EmitAsm("ROL", Mem(temps[1]));
                    }
                }
            }
            finally
            {
                ReleaseTemps(temps);
            }
        }

        // Record recognized NES actions for reports and emit advisory timing/mapper warnings.
        // This analysis does not prove that rendering is disabled or that a call runs in NMI.
        void AnalyzeNesActionCall(FilePosition source, string funcName, Expr[] args, FunctionContext ctx)
        {
            NesActionCatalog.ActionInfo info;
            if (!NesActionCatalog.TryGet(funcName, out info)) return;

            int callerBank = ctx == null ? 0 : ctx.Bank;
            string caller = ctx == null ? _currentFunctionName : ctx.Name;
            string sourceText = source.ToString();
            string timing = info.Timing.ToString();
            string note = "";

            _assembly.Add(Expr.Make(Tag.Comment, string.Format("KUROSAKI_ACTION name={0} category={1} op={2} caller={3} bank={4}",
                info.Name, info.Category, info.Operation, string.IsNullOrEmpty(caller) ? "<none>" : caller, callerBank)).WithSource(source));

            // Timing classification uses the caller name rather than runtime interrupt state.
            bool inNmi = IsNmiLikeFunction(caller);
            if (info.DirectPpuAccess && info.Timing == NesActionCatalog.TimingClass.NmiOrRenderingOff && !inNmi)
            {
                WarnOnce(ErrorCode.NesUnsafePpuAccess,
                    "ppu:" + funcName + ":" + caller + ":" + sourceText,
                    source,
                    "{0} performs direct PPU/VRAM work outside __nes_nmi(); prefer __vramq_* or wrap the write in a rendering-off section.",
                    funcName);
                note = "direct_ppu_outside_nmi";
            }
            if (info.PerformsOamDma && !inNmi)
            {
                WarnOnce(ErrorCode.NesUnsafePpuAccess,
                    "oam_dma:" + funcName + ":" + caller + ":" + sourceText,
                    source,
                    "{0} should normally run during NMI/vblank; doing OAM DMA in mainline code can cause visible stalls.",
                    funcName);
                note = AppendNote(note, "oam_dma_outside_nmi");
            }

            // Compare action requirements with the selected cartridge profile at compile time.
            if (info.RequiresFds && Program.NesMapperProfile.MapperKind != NesMapperKind.Fds)
            {
                WarnOnce(ErrorCode.NesFdsApiMapperMismatch,
                    "fds:" + funcName + ":" + Program.NesMapperProfile.CliName,
                    source,
                    "{0} is FDS-only but current mapper is {1}.",
                    funcName, Program.NesMapperProfile.CliName);
                note = AppendNote(note, "fds_mapper_mismatch");
            }

            if (!string.IsNullOrEmpty(info.MapperRequirement) && !MapperRequirementMatches(info.MapperRequirement))
            {
                WarnOnce(ErrorCode.NesMapperMismatch,
                    "mapper:" + funcName + ":" + Program.NesMapperProfile.CliName,
                    source,
                    "{0} expects mapper {1}; current mapper is {2}.",
                    funcName, info.MapperRequirement, Program.NesMapperProfile.CliName);
                note = AppendNote(note, "mapper_mismatch");
            }

            // Only statically known sprite indices at or above 64 trigger this diagnostic.
            if (info.OamIndexArgument >= 0 && args != null && info.OamIndexArgument < args.Length)
            {
                int spriteIndex;
                if (TryEvaluateConstant(args[info.OamIndexArgument], out spriteIndex) && spriteIndex >= 64)
                {
                    WarnOnce(ErrorCode.NesOamOverflowRisk,
                        "oam_index:" + funcName + ":" + caller + ":" + sourceText,
                        source,
                        "{0} uses sprite index {1}, but NES OAM has indices 0..63.",
                        funcName, spriteIndex);
                    note = AppendNote(note, "oam_index_out_of_range");
                }
            }

            // Keep every recognized use, including its source location and accumulated warning notes.
            _nesActionUses.Add(new NesActionUseInfo
            {
                Name = info.Name,
                Category = info.Category,
                Operation = info.Operation,
                KurosakiKind = info.KurosakiKind,
                Caller = string.IsNullOrEmpty(caller) ? "<none>" : caller,
                CallerBank = callerBank,
                Source = sourceText,
                Timing = timing,
                DirectPpuAccess = info.DirectPpuAccess,
                QueuePpuAccess = info.QueuePpuAccess,
                UsesOamShadow = info.UsesOamShadow,
                PerformsOamDma = info.PerformsOamDma,
                RequiresFds = info.RequiresFds,
                MapperRequirement = info.MapperRequirement ?? "",
                Note = note ?? "",
            });
        }

        // Treat NMI/vblank names as a heuristic; this does not verify a function call graph.
        static bool IsNmiLikeFunction(string name)
        {
            if (string.IsNullOrEmpty(name)) return false;
            return string.Equals(name, "__nes_nmi", StringComparison.Ordinal) ||
                   string.Equals(name, "nmi", StringComparison.OrdinalIgnoreCase) ||
                   name.IndexOf("nmi", StringComparison.OrdinalIgnoreCase) >= 0 ||
                   name.IndexOf("vblank", StringComparison.OrdinalIgnoreCase) >= 0;
        }

        // Join multiple machine-readable diagnostic reasons with semicolons.
        static string AppendNote(string note, string value)
        {
            if (string.IsNullOrEmpty(note)) return value;
            return note + ";" + value;
        }

        // Accept the normalized profile name or supported MMC3/FDS family aliases.
        bool MapperRequirementMatches(string requirement)
        {
            string r = (requirement ?? "").Trim().ToLowerInvariant();
            string m = (Program.NesMapperProfile.CliName ?? "").Trim().ToLowerInvariant();
            if (string.IsNullOrEmpty(r)) return true;
            if (r == m) return true;
            if (r == "mmc3" && Program.NesMapperProfile.MapperKind == NesMapperKind.Mmc3) return true;
            if (r == "fds" && Program.NesMapperProfile.MapperKind == NesMapperKind.Fds) return true;
            return false;
        }

        // Deduplicate action diagnostics by the supplied key within this generator instance.
        void WarnOnce(ErrorCode code, string key, FilePosition source, string format, params object[] args)
        {
            if (!_nesActionWarningKeys.Add(key ?? "")) return;
            Program.Warning(Maybe.Just(source), code, format, args);
        }

        void EmitCall(FilePosition source, Expr funcExpr, Expr[] args, FunctionContext ctx, int expectedReturnSize)
        {
            // Every function uses the same temporary pool. An outer expression
            // can still own bytes in it while a callee evaluates its own body:
            // e.g. f() + array[0], or the first argument of g(a, f()).
            // Save the caller's live bytes before evaluating arguments so nested
            // calls also preserve already evaluated arguments and lvalue pointers.
            int[] liveTemps = _tempPool.Where(t => !_freeTemps.Contains(t)).ToArray();
            if (liveTemps.Length != 0)
            {
                EmitAsm("TAY");
                foreach (int temp in liveTemps)
                {
                    EmitAsm("LDA", Mem(temp));
                    EmitAsm("PHA");
                }
                EmitAsm("TYA");
            }

            EmitCallCore(source, funcExpr, args, ctx, expectedReturnSize);

            if (liveTemps.Length != 0)
            {
                // Return values use A (low) and X (high). PLA only changes A;
                // keep its result in Y while restoring the saved bytes.
                EmitAsm("TAY");
                for (int i = liveTemps.Length - 1; i >= 0; i--)
                {
                    EmitAsm("PLA");
                    EmitAsm("STA", Mem(liveTemps[i]));
                }
                EmitAsm("TYA");
            }
        }

        // Dispatch recognized intrinsics and optimized direct calls before packing the ordinary argument area.
        void EmitCallCore(FilePosition source, Expr funcExpr, Expr[] args, FunctionContext ctx, int expectedReturnSize)
        {
            string funcName;
            if (!funcExpr.Match(Tag.Name, out funcName))
            {
                Program.Error("error KQ0000: --target=nes phase 5 supports only direct calls.");
                return;
            }

            AnalyzeNesActionCall(source, funcName, args, ctx);

            // The direct bank-switch intrinsic evaluates a byte argument and calls the mapper helper.
            if (string.Equals(funcName, "__bankswitch", StringComparison.Ordinal))
            {
                if (args == null || args.Length != 1)
                {
                    Program.Error("error KQ0000: __bankswitch(bank) expects exactly 1 argument.");
                    return;
                }
                EmitLoadValue(args[0], ctx, 1);
                EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
                return;
            }

            if (string.Equals(funcName, "__fds_overlay_farcall", StringComparison.Ordinal) ||
                string.Equals(funcName, "__fds_farcall", StringComparison.Ordinal))
            {
                // FDS overlay dispatch requires the configured FDS RAM layout and a named function target.
                Expr[] farArgs = args ?? Array.Empty<Expr>();
                if (farArgs.Length != 2)
                {
                    Program.Error("error KQFC2512: {0}(bank, func) expects exactly 2 arguments.", funcName);
                    return;
                }
                string targetName;
                if (!farArgs[1].Match(Tag.Name, out targetName))
                {
                    Program.Error("error KQFC2513: {0}(bank, func) requires the 2nd argument to be a function name.", funcName);
                    return;
                }
                if (!Program.NesMapperProfile.HasFds || !Program.FdsPrgRamLayoutEnabled)
                {
                    Program.Error("error KQFC2514: {0} requires --mapper=fds with --fds-layout=fds32.", funcName);
                    return;
                }
                int callerBank = ctx == null ? 0 : ctx.Bank;
                int calleeBank = GetFunctionBank(targetName);
                // Choose an overlay thunk from the target function metadata rather than evaluating the first FDS argument.
                if (ShouldUseFdsOverlayThunk(callerBank, calleeBank))
                {
                    string directFdsThunkName = EnsureFdsOverlayThunk(targetName, calleeBank);
                    EmitAsm("JSR", Abs(directFdsThunkName));
                    RecordCallEdge(targetName, calleeBank, "fds_overlay_thunk", viaThunk: true, viaFarcall: true);
                    return;
                }
                if (calleeBank < 2 || calleeBank == callerBank)
                {
                    EmitAsm("JSR", Abs(targetName));
                    RecordCallEdge(targetName, calleeBank, "direct", viaThunk: false, viaFarcall: true);
                    return;
                }
                string fdsThunkName = EnsureFdsOverlayThunk(targetName, calleeBank);
                EmitAsm("JSR", Abs(fdsThunkName));
                RecordCallEdge(targetName, calleeBank, "fds_overlay_thunk", viaThunk: true, viaFarcall: true);
                return;
            }

            if (string.Equals(funcName, "__farcall", StringComparison.Ordinal))
            {
                Expr[] farArgs = args ?? Array.Empty<Expr>();
                if (farArgs.Length != 2)
                {
                    Program.Error("error KQ0000: __farcall(bank, func) expects exactly 2 arguments.");
                    return;
                }

                string targetName;
                if (!farArgs[1].Match(Tag.Name, out targetName))
                {
                    Program.Error("error KQ0000: __farcall(bank, func) requires the 2nd argument to be a function name.");
                    return;
                }
                if (Program.NesMapperProfile.IsSurom512 &&
                    TryEvaluateConstant(farArgs[0], out int constantFarBank) &&
                    (constantFarBank < 1 || constantFarBank > 30))
                {
                    Program.Error("error KQFC2603 KQFC-SUROM-BANK-RANGE: __farcall constant bank {0} is outside 1..30; common bank 0 is not a switch target.", constantFarBank);
                    return;
                }

                // Evaluate the requested far-call bank byte; the following dispatch uses the target function bank metadata.
                EmitLoadValue(farArgs[0], ctx, 1);

                int callerBank = ctx == null ? 0 : ctx.Bank;
                int calleeBank = GetFunctionBank(targetName);
                if (ShouldUseFdsOverlayThunk(callerBank, calleeBank))
                {
                    string fdsThunkName = EnsureFdsOverlayThunk(targetName, calleeBank);
                    EmitAsm("JSR", Abs(fdsThunkName));
                    RecordCallEdge(targetName, calleeBank, "fds_overlay_thunk", viaThunk: true, viaFarcall: true);
                    return;
                }
                bool bankedPrg = Program.NesMapperProfile.SupportsPrgBanking || Program.NesMapperProfile.UsesDuplicatedCommonBank;
                bool directSafe = !bankedPrg || calleeBank == 0 || calleeBank == callerBank;
                if (directSafe)
                {
                    EmitAsm("JSR", Abs(targetName));
                    RecordCallEdge(targetName, calleeBank, "direct", viaThunk: false, viaFarcall: true);
                    return;
                }

                string thunkName = EnsureBankThunk(targetName, calleeBank);
                EmitAsm("JSR", Abs(thunkName));
                RecordCallEdge(targetName, calleeBank, "bank_thunk", viaThunk: true, viaFarcall: true);
                return;
            }

            if (string.Equals(funcName, "__assert", StringComparison.Ordinal))
            {
                // Test the condition through the byte-value path and evaluate an optional code only on failure.
                Expr[] assertArgs = args ?? Array.Empty<Expr>();
                if (assertArgs.Length != 1 && assertArgs.Length != 2)
                {
                    Program.Error("error KQ0000: __assert(cond[, code]) expects 1 or 2 arguments.");
                    return;
                }

                string okLabel = NewGeneratedLabel("assert_ok");
                EmitLoadValue(assertArgs[0], ctx, 1);
                EmitAsm("CMP", Imm(0));
                EmitAsm("BNE", Rel(okLabel));
                if (assertArgs.Length == 2)
                    EmitLoadValue(assertArgs[1], ctx, 1);
                EmitAsm("JMP", Abs("__kq_hang"));
                _assembly.Add(Expr.Make(Tag.Label, okLabel).WithSource(source));
                return;
            }

            // Try built-in expansion, small-body inlining and register arguments before the general call ABI.
            if (TryEmitKitaqfcIntrinsicCall(funcName, args, ctx))
                return;

            if (TryEmitSmallInlineCall(funcName, args, ctx, expectedReturnSize))
                return;

            if (TryEmitFastCallV2(funcName, args, ctx, source, expectedReturnSize))
                return;

            FieldInfo[] parameters;
            // Prefer a visible prototype; recognized compatibility intrinsics can supply synthetic parameter types.
            bool haveVisiblePrototype = _functionParameters.TryGetValue(funcName, out parameters);
            parameters = parameters ?? Array.Empty<FieldInfo>();
            args = args ?? Array.Empty<Expr>();
            if (!haveVisiblePrototype && TryGetCompatibilityIntrinsicSignature(funcName, out IntrinsicSignature intrinsicSignature))
                parameters = BuildSyntheticParameters(intrinsicSignature);

            if ((haveVisiblePrototype || parameters.Length != 0) && parameters.Length != args.Length)
            {
                Program.Error("error KQ0000: --target=nes phase 5 argument count mismatch for {0}: expected {1}, got {2}.",
                    funcName, parameters.Length, args.Length);
                return;
            }
            if (!haveVisiblePrototype && parameters.Length == 0 && args.Length != 0)
            {
                Program.Error("error KQ0000: --target=nes phase 5 requires a visible prototype/definition before calling '{0}' with arguments.", funcName);
                return;
            }

            // Reserve full storage width for every ordinary argument, including aggregate payloads.
            int totalArgBytes = 0;
            for (int i = 0; i < parameters.Length; i++)
            {
                int size = GetStorageSize(parameters[i].Type);
                if (size <= 0)
                {
                    Program.Error("error KQ0000: --target=nes could not determine parameter size for call: {0}::{1}", funcName, parameters[i].Name);
                    return;
                }
                totalArgBytes += size;
            }
            if (totalArgBytes > _tempPool.Length)
            {
                Program.Error("error KQ0000: --target=nes phase 5 call scratch overflow for '{0}' ({1} bytes > {2} temp bytes).",
                    funcName, totalArgBytes, _tempPool.Length);
                return;
            }

            // Stage argument values away from the shared call area so nested calls cannot overwrite earlier arguments.
            int[] scratch = AcquireTemps(totalArgBytes);
            if (Program.ErrorCount > 0) return;

            try
            {
                int offset = 0;
                for (int i = 0; i < args.Length; i++)
                {
                    // Evaluate ordinary arguments in source order, copying aggregate bytes into the reserved scratch slots.
                    int argSize = GetStorageSize(parameters[i].Type);
                    if (IsAggregateType(parameters[i].Type))
                    {
                        int[] src = AcquireTemps(2);
                        try
                        {
                            if (!EmitAggregateSourceAddressIntoPair(args[i], ctx, src[0], src[1], parameters[i].Type))
                                return;
                            EmitCopyBytesFromPtrToTempBytes(src[0], src[1], scratch, offset, argSize, args[i].Source);
                        }
                        finally
                        {
                            ReleaseTemps(src);
                        }
                    }
                    else
                    {
                        EmitLoadValue(args[i], ctx, argSize);
                        EmitAsm("STA", Mem(scratch[offset]));
                        if (argSize == 2)
                            EmitAsm("STX", Mem(scratch[offset + 1]));
                    }
                    offset += argSize;
                }

                // Publish all staged bytes to the shared call argument area immediately before dispatch.
                for (int i = 0; i < totalArgBytes; i++)
                {
                    EmitAsm("LDA", Mem(scratch[i]));
                    EmitAsm("STA", Mem(CallArgBase + i));
                }

                EmitResolvedCall(funcName, ctx == null ? 0 : ctx.Bank, source, expectedReturnSize > 0 ? expectedReturnSize : GetFunctionReturnSize(funcName));

                int retSize = expectedReturnSize > 0 ? expectedReturnSize : GetFunctionReturnSize(funcName);
                if (retSize <= 1) return;
                // 16-bit return already comes back in A(low)/X(high).
            }
            finally
            {
                ReleaseTemps(scratch);
            }
        }

        // Use A/X arguments only for an enabled fastcall target with at most two scalar-normalized bytes.
        // Cross-bank calls fall back to the ordinary argument-area path.
        bool TryEmitFastCallV2(string funcName, Expr[] args, FunctionContext ctx, FilePosition source, int expectedReturnSize)
        {
            if (!Program.AbiFastCall || !IsFunctionFastCall(funcName)) return false;
            FieldInfo[] parameters;
            if (!_functionParameters.TryGetValue(funcName, out parameters)) return false;
            parameters = parameters ?? Array.Empty<FieldInfo>();
            args = args ?? Array.Empty<Expr>();
            if (parameters.Length != args.Length) return false;
            int total = parameters.Sum(p => NormalizeScalarSize(GetStorageSize(p.Type)));
            if (parameters.Length > 2 || total > 2) return false;

            int calleeBank = GetFunctionBank(funcName);
            int callerBank = ctx == null ? 0 : ctx.Bank;
            bool bankedPrg = Program.NesMapperProfile.SupportsPrgBanking || Program.NesMapperProfile.UsesDuplicatedCommonBank || Program.NesMapperProfile.HasFds;
            bool directSafe = !bankedPrg || calleeBank == 0 || calleeBank == callerBank;
            if (!directSafe) return false;

            if (parameters.Length == 0)
            {
                EmitResolvedCall(funcName, callerBank, source, expectedReturnSize > 0 ? expectedReturnSize : GetFunctionReturnSize(funcName));
            }
            else if (parameters.Length == 1)
            {
                EmitLoadValue(args[0], ctx, NormalizeScalarSize(GetStorageSize(parameters[0].Type)));
                EmitResolvedCall(funcName, callerBank, source, expectedReturnSize > 0 ? expectedReturnSize : GetFunctionReturnSize(funcName));
            }
            else
            {
                // For two byte arguments, evaluate the right argument first and preserve it while loading the left into A.
                int rightTemp = AcquireTemp();
                try
                {
                    EmitLoadValue(args[1], ctx, 1);
                    EmitAsm("STA", Mem(rightTemp));
                    EmitLoadValue(args[0], ctx, 1);
                    EmitAsm("LDX", Mem(rightTemp));
                    EmitResolvedCall(funcName, callerBank, source, expectedReturnSize > 0 ? expectedReturnSize : GetFunctionReturnSize(funcName));
                }
                finally
                {
                    ReleaseTemp(rightTemp);
                }
            }
            RecordInlineDecision(ctx, funcName, "fastcall-v2", "arguments passed in A/X instead of call argument area", true);
            return true;
        }

        // Consider a visible small function body in a compatible bank, excluding direct self-inlining and aggregate returns.
        bool TryEmitSmallInlineCall(string funcName, Expr[] args, FunctionContext ctx, int expectedReturnSize)
        {
            if (!Program.EnableSmallFunctionAutoInline || string.IsNullOrEmpty(funcName)) return false;
            if (ctx != null && string.Equals(ctx.Name, funcName, StringComparison.Ordinal)) return false;
            Expr decl;
            if (!_functions.TryGetValue(funcName, out decl) || decl == null) return false;
            int calleeBank = GetFunctionBank(funcName);
            int callerBank = ctx == null ? 0 : ctx.Bank;
            if (calleeBank != 0 && calleeBank != callerBank) return false;

            CType retType; string emittedName; FieldInfo[] fields; int mustCheck; Expr body;
            if (!decl.Match<CType, string, FieldInfo[], int, Expr>(Tag.Function, out retType, out emittedName, out fields, out mustCheck, out body) &&
                !decl.Match<CType, string, FieldInfo[], int, Expr>(Tag.InlineFunction, out retType, out emittedName, out fields, out mustCheck, out body)) return false;
            if (IsAggregateType(retType)) return false;
            fields = fields ?? Array.Empty<FieldInfo>();
            args = args ?? Array.Empty<Expr>();
            if (fields.Length != args.Length) return false;
            if (!IsSmallInlineCandidate(body, fields.Length)) return false;

            // Substitute actual expression nodes for parameter names; no new parameter storage is created here.
            var subst = new Dictionary<string, Expr>(StringComparer.Ordinal);
            for (int i = 0; i < fields.Length; i++) subst[fields[i].Name] = args[i];

            Expr returnExpr;
            if (body.Match(Tag.Return, out returnExpr))
            {
                EmitLoadValue(SubstituteExpr(returnExpr, subst), ctx, expectedReturnSize > 0 ? expectedReturnSize : NormalizeScalarSize(TypeStorageSize(retType)));
                RecordInlineDecision(ctx, funcName, "auto-inline", "single return expression", true);
                return true;
            }

            Expr[] seq;
            if (body.MatchAny(Tag.Sequence, out seq) && seq != null && seq.Length == 1 && seq[0].Match(Tag.Return, out returnExpr))
            {
                EmitLoadValue(SubstituteExpr(returnExpr, subst), ctx, expectedReturnSize > 0 ? expectedReturnSize : NormalizeScalarSize(TypeStorageSize(retType)));
                RecordInlineDecision(ctx, funcName, "auto-inline", "single return expression in sequence", true);
                return true;
            }

            // An unused return value permits a small statement body only when it contains no return statement.
            if (expectedReturnSize == 0 && !ContainsUnsafeInlineConstruct(body) && !ContainsReturnStatement(body))
            {
                EmitStatement(SubstituteExpr(body, subst), ctx);
                RecordInlineDecision(ctx, funcName, "auto-inline", "small void statement body", true);
                return true;
            }
            return false;
        }

        // Append the applied call-lowering decision and its reason to the compiler report.
        void RecordInlineDecision(FunctionContext ctx, string callee, string kind, string reason, bool applied)
        {
            _inlineDecisions.Add(new InlineDecisionInfo { Caller = ctx == null ? "<none>" : ctx.Name, Callee = callee ?? "", Kind = kind ?? "", Reason = reason ?? "", Applied = applied });
        }

        // Limit expression-tree size using parameter count and the current caller hotness hint.
        bool IsSmallInlineCandidate(Expr body, int parameterCount)
        {
            if (body == null) return false;
            if (ContainsUnsafeInlineConstruct(body)) return false;
            int budget = 10 + parameterCount * 4 + Program.GetKurosakiHotness(_currentFunctionName) / 10000;
            return CountExprNodes(body) <= budget;
        }

        // Search expression children and expression arrays recursively for a return node.
        bool ContainsReturnStatement(Expr expr)
        {
            if (expr == null) return false;
            if (expr.Tag == Tag.Return) return true;
            foreach (object arg in expr.GetArgs().Skip(1))
            {
                if (arg is Expr e && ContainsReturnStatement(e)) return true;
                if (arg is Expr[] arr && (arr ?? Array.Empty<Expr>()).Any(ContainsReturnStatement)) return true;
            }
            return false;
        }

        // Reject the listed local/control-flow/assembly node forms before expression substitution.
        // This structural filter does not analyze actual argument side effects.
        bool ContainsUnsafeInlineConstruct(Expr expr)
        {
            if (expr == null) return false;
            string tag = expr.Tag;
            if (tag == Tag.Variable || tag == Tag.Label || tag == Tag.Jump || tag == Tag.Asm || tag == Tag.For || tag == Tag.DoWhile || tag == Tag.Switch) return true;
            foreach (object arg in expr.GetArgs().Skip(1))
            {
                if (arg is Expr e && ContainsUnsafeInlineConstruct(e)) return true;
                if (arg is Expr[] arr && (arr ?? Array.Empty<Expr>()).Any(ContainsUnsafeInlineConstruct)) return true;
            }
            return false;
        }

        // Count expression nodes recursively; non-expression metadata does not contribute to the budget.
        int CountExprNodes(Expr expr)
        {
            if (expr == null) return 0;
            int count = 1;
            foreach (object arg in expr.GetArgs().Skip(1))
            {
                if (arg is Expr e) count += CountExprNodes(e);
                else if (arg is Expr[] arr) foreach (var e2 in arr ?? Array.Empty<Expr>()) count += CountExprNodes(e2);
            }
            return count;
        }

        // Replace matching name nodes recursively while retaining source locations on rebuilt expressions.
        Expr SubstituteExpr(Expr expr, Dictionary<string, Expr> subst)
        {
            if (expr == null || subst == null || subst.Count == 0) return expr;
            string name;
            if (expr.Match(Tag.Name, out name) && subst.TryGetValue(name, out Expr replacement)) return replacement;
            object[] args = expr.GetArgs();
            object[] next = new object[args.Length];
            next[0] = args[0];
            bool changed = false;
            for (int i = 1; i < args.Length; i++)
            {
                object a = args[i];
                if (a is Expr e)
                {
                    Expr ne = SubstituteExpr(e, subst);
                    next[i] = ne;
                    changed |= !object.ReferenceEquals(e, ne);
                }
                else if (a is Expr[] arr)
                {
                    Expr[] narr = (arr ?? Array.Empty<Expr>()).Select(e2 => SubstituteExpr(e2, subst)).ToArray();
                    next[i] = narr;
                    changed = true;
                }
                else next[i] = a;
            }
            return changed ? Expr.Make(next).WithSource(expr.Source) : expr;
        }

        // Expand recognized public intrinsics into register accesses, inline code or shared helper calls.
        // Return true for a recognized name even when its arguments produce a diagnostic.
        bool TryEmitKitaqfcIntrinsicCall(string funcName, Expr[] args, FunctionContext ctx)
        {
            args = args ?? Array.Empty<Expr>();
            // Check exact arity before indexing an intrinsic argument array.
            bool Need(int n)
            {
                if (args.Length == n) return true;
                Program.Error("error KQ0000: {0} expects {1} argument(s), got {2}.", funcName, n, args.Length);
                return false;
            }
            // Request a byte value for hardware registers and byte-width helper arguments.
            void Load1(Expr e) { EmitLoadValue(e, ctx, 1); }
            // Evaluate once, then store A at the requested CPU address.
            void Store1(Expr e, int addr) { Load1(e); EmitAsm("STA", Mem(addr)); }
            // Stage the address before reading PPUSTATUS to reset the latch, then write high and low bytes to PPUADDR.
            void PpuAddr(Expr e)
            {
                int[] t = AcquireTemps(2);
                try
                {
                    EmitLoadValue(e, ctx, 2);
                    EmitAsm("STA", Mem(t[0]));
                    EmitAsm("STX", Mem(t[1]));
                    EmitAsm("LDA", Mem(0x2002));
                    EmitAsm("LDA", Mem(t[1]));
                    EmitAsm("STA", Mem(0x2006));
                    EmitAsm("LDA", Mem(t[0]));
                    EmitAsm("STA", Mem(0x2006));
                }
                finally { ReleaseTemps(t); }
            }
            // Route the FDS namespace through mapper-specific validation before ordinary intrinsic dispatch.
            if (funcName.StartsWith("__fds_", StringComparison.Ordinal))
                return EmitFdsIntrinsicCall(funcName, args, ctx, Need, Load1);

            switch (funcName)
            {
                case "__memcpy":
                    if (TryEmitMemcpyOrMemsetInline(funcName, args, ctx, isSet: false, lenIndex: 2, lenSize: 2)) return true;
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,2}, "__memcpy");
                case "__memcpy_small":
                    if (TryEmitMemcpyOrMemsetInline(funcName, args, ctx, isSet: false, lenIndex: 2, lenSize: 1)) return true;
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,1}, "__memcpy_small");
                case "__memset":
                    if (TryEmitMemcpyOrMemsetInline(funcName, args, ctx, isSet: true, lenIndex: 2, lenSize: 2)) return true;
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,1,2}, "__memset");
                case "__memset_small":
                    if (TryEmitMemcpyOrMemsetInline(funcName, args, ctx, isSet: true, lenIndex: 2, lenSize: 1)) return true;
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,1,1}, "__memset_small");
                // These fixed-size copies bypass the variable-length helper argument area.
                case "__copy16":
                    if (!Need(2)) return true;
                    EmitFixedMemcpy(args[0], args[1], 16, ctx); return true;
                case "__copy32":
                    if (!Need(2)) return true;
                    EmitFixedMemcpy(args[0], args[1], 32, ctx); return true;
                // Geometry helpers receive byte coordinates; the size array describes ABI packing, not result width.
                case "__xy_in_rect":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1,1,1,1}, "__xy_in_rect");
                case "__manhattan":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1,1}, "__manhattan");
                case "__map_index":
                    if (TryEmitMapIndexInline(args, ctx)) return true;
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1}, "__map_index");
                case "__bit_test":
                    if (TryEmitBitIntrinsicInline(funcName, args, ctx)) return true;
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2}, "__bit_test");
                case "__bit_set":
                    if (TryEmitBitIntrinsicInline(funcName, args, ctx)) return true;
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2}, "__bit_set");
                case "__bit_clear":
                    if (TryEmitBitIntrinsicInline(funcName, args, ctx)) return true;
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2}, "__bit_clear");
                case "__bit_toggle":
                    if (TryEmitBitIntrinsicInline(funcName, args, ctx)) return true;
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2}, "__bit_toggle");
                // Arithmetic variants share argument packing here; signed/fixed-point arithmetic is implemented by their named helpers.
                case "__mul16x8": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,1}, funcName);
                case "__smul16x8": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,1}, funcName);
                case "__mul8x8_hi": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1}, funcName);
                case "__mac16": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,1}, funcName);
                case "__smac16": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,1}, funcName);
                case "__smul16x8_q1_7": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,1}, funcName);
                case "__smac16_q1_7": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,1}, funcName);
                case "__dot2_q8_8": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,1,1}, funcName);
                case "__dot3_q8_8": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,2,1,1,1}, funcName);
                case "__sdot2_q8_8": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,1,1}, funcName);
                case "__sdot3_q8_8": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,2,1,1,1}, funcName);
                case "__sdot2_q1_7": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,1,1}, funcName);
                case "__sdot3_q1_7": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,2,1,1,1}, funcName);
                // Pass a word seed to the RNG helper; the zero-argument generator is called directly below.
                case "__rng_seed":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2}, "__rng_seed");
                case "__rng8":
                    if (!Need(0)) return true;
                    EmitAsm("JSR", Abs("__rng8"));
                    return true;
                // Far-memory helpers pack the address, bank and length arguments according to their explicit byte/word signatures.
                case "__far_memcpy":
                case "__farmemcpy":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,1,2,2}, "__far_memcpy");
                case "__farpeek8":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,2}, "__farpeek8");
                case "__farpeek16":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,2}, "__farpeek16");
                // Rendering controls update the software PPUMASK shadow as well as the hardware register.
                case "__ppu_on":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Imm(0x1E)); EmitAsm("STA", Mem(_runtimePpuMaskShadowAddress)); EmitAsm("STA", Mem(0x2001)); return true;
                case "__ppu_off":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Imm(0)); EmitAsm("STA", Mem(_runtimePpuMaskShadowAddress)); EmitAsm("STA", Mem(0x2001)); return true;
                // Keep the supplied mask available for later runtime code that restores PPU state.
                case "__ppu_mask_set":
                    if (!Need(1)) return true;
                    Store1(args[0], _runtimePpuMaskShadowAddress); EmitAsm("STA", Mem(0x2001)); return true;
                // Update the PPUCTRL shadow before writing the hardware register.
                case "__ppu_ctrl_set":
                    if (!Need(1)) return true;
                    Store1(args[0], _runtimePpuCtrlShadowAddress); EmitAsm("STA", Mem(0x2000)); return true;
                // Direct PPU address/data access does not wait for a safe rendering interval here.
                case "__ppu_addr":
                    if (!Need(1)) return true;
                    PpuAddr(args[0]); return true;
                case "__ppu_data":
                    if (!Need(1)) return true;
                    Load1(args[0]); EmitAsm("STA", Mem(0x2007)); return true;
                // Reading PPUSTATUS also has hardware side effects; this is not a side-effect-free status cache.
                case "__ppu_read_status":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(0x2002)); return true;
                case "__scroll_latch_reset":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(0x2002)); return true;
                // Store both scroll components, reset the write latch, then emit the X/Y pair.
                case "__scroll_set":
                    if (!Need(2)) return true;
                    Store1(args[0], _runtimeScrollXAddress); Store1(args[1], _runtimeScrollYAddress);
                    EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Mem(_runtimeScrollXAddress)); EmitAsm("STA", Mem(0x2005)); EmitAsm("LDA", Mem(_runtimeScrollYAddress)); EmitAsm("STA", Mem(0x2005)); return true;
                // Changing X rewrites both latch bytes, using the retained Y component.
                case "__scroll_x_set":
                    if (!Need(1)) return true;
                    Store1(args[0], _runtimeScrollXAddress);
                    EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Mem(_runtimeScrollXAddress)); EmitAsm("STA", Mem(0x2005)); EmitAsm("LDA", Mem(_runtimeScrollYAddress)); EmitAsm("STA", Mem(0x2005)); return true;
                // Changing Y rewrites both latch bytes, using the retained X component.
                case "__scroll_y_set":
                    if (!Need(1)) return true;
                    Store1(args[0], _runtimeScrollYAddress);
                    EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Mem(_runtimeScrollXAddress)); EmitAsm("STA", Mem(0x2005)); EmitAsm("LDA", Mem(_runtimeScrollYAddress)); EmitAsm("STA", Mem(0x2005)); return true;
                // Modify only the NMI-enable bit in the PPUCTRL shadow and publish the new register value.
                case "__nmi_enable":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(_runtimePpuCtrlShadowAddress)); EmitAsm("ORA", Imm(0x80)); EmitAsm("STA", Mem(_runtimePpuCtrlShadowAddress)); EmitAsm("STA", Mem(0x2000)); return true;
                case "__nmi_disable":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(_runtimePpuCtrlShadowAddress)); EmitAsm("AND", Imm(0x7F)); EmitAsm("STA", Mem(_runtimePpuCtrlShadowAddress)); EmitAsm("STA", Mem(0x2000)); return true;
                case "__irq_disable": if (!Need(0)) return true; EmitAsm("SEI"); return true;
                case "__irq_enable": if (!Need(0)) return true; EmitAsm("CLI"); return true;
                // Return the saved processor status in A after masking IRQs; restore consumes a status byte, not a Boolean.
                case "__irq_save": if (!Need(0)) return true; EmitAsm("PHP"); EmitAsm("SEI"); EmitAsm("PLA"); return true;
                case "__irq_restore": if (!Need(1)) return true; Load1(args[0]); EmitAsm("PHA"); EmitAsm("PLP"); return true;
                case "__nmi_wait": if (!Need(0)) return true; EmitAsm("JSR", Abs("__nmi_wait")); return true;
                // Return the current NMI counter byte; callers decide how to compare it with their previous value.
                case "__nmi_ready": if (!Need(0)) return true; EmitAsm("LDA", Mem(_runtimeNmiCounterAddress)); return true;
                // Reset OAMADDR and transfer page 02; the page-selecting variant evaluates its own byte argument.
                case "__oam_dma": if (!Need(0)) return true; EmitAsm("LDA", Imm(0)); EmitAsm("STA", Mem(0x2003)); EmitAsm("LDA", Imm(0x02)); EmitAsm("STA", Mem(0x4014)); return true;
                case "__oam_dma_page": if (!Need(1)) return true; EmitAsm("LDA", Imm(0)); EmitAsm("STA", Mem(0x2003)); Load1(args[0]); EmitAsm("STA", Mem(0x4014)); return true;
                case "__mapper_id": if (!Need(0)) return true; EmitAsm("LDA", Imm(Program.NesMapperProfile.MapperNumber & 0xFF)); return true;
                // Validate constant SUROM switch-bank numbers on this path and record mapper-writing functions.
                case "__prg_bank_set":
                case "__bankswitch":
                    if (!Need(1)) return true;
                    if (Program.NesMapperProfile.IsSurom512 && TryEvaluateConstant(args[0], out int constantBank) && (constantBank < 1 || constantBank > 30))
                    {
                        Program.Error("error KQFC2603 KQFC-SUROM-BANK-RANGE: {0} constant bank {1} is outside 1..30; common bank 0 is not a switch target.", funcName, constantBank);
                        return true;
                    }
                    _mapperWritingFunctions.Add(_currentFunctionName);
                    Load1(args[0]); EmitAsm("JSR", Abs("__kq_prg_set_bank_a")); return true;
                // Controller sampling is delegated to runtime helpers; mask-only accessors below operate on supplied snapshots.
                case "__pad_read1": if (!Need(0)) return true; EmitAsm("JSR", Abs("__pad_read1")); return true;
                case "__pad_read2": if (!Need(0)) return true; EmitAsm("JSR", Abs("__pad_read2")); return true;
                case "__pad_read1_safe": if (!Need(0)) return true; EmitAsm("JSR", Abs("__pad_read1_safe")); return true;
                case "__pad_read2_safe": if (!Need(0)) return true; EmitAsm("JSR", Abs("__pad_read2_safe")); return true;
                case "__pad_buttons": if (!Need(1)) return true; Load1(args[0]); EmitAsm("AND", Imm(0x0F)); return true;
                case "__pad_dirs": if (!Need(1)) return true; Load1(args[0]); EmitAsm("AND", Imm(0xF0)); return true;
                // Expansion-controller names alias the corresponding D1 sampling helpers.
                case "__pad_read1_d1":
                case "__exp_pad_read1":
                    if (!Need(0)) return true; EmitAsm("JSR", Abs("__pad_read1_d1")); return true;
                case "__pad_read2_d1":
                case "__exp_pad_read2":
                    if (!Need(0)) return true; EmitAsm("JSR", Abs("__pad_read2_d1")); return true;
                // Both microphone aliases call the same second-controller helper.
                case "__joypad2p_voice":
                case "__mic_read2p":
                    if (!Need(0)) return true; EmitAsm("JSR", Abs("__mic_read2p")); return true;
                // Expose the selected raw port bits; trigger/light helpers below perform their own interpretation.
                case "__zapper_raw1": if (!Need(0)) return true; EmitAsm("LDA", Mem(0x4016)); EmitAsm("AND", Imm(0x18)); return true;
                case "__zapper_raw2": if (!Need(0)) return true; EmitAsm("LDA", Mem(0x4017)); EmitAsm("AND", Imm(0x18)); return true;
                case "__zapper_trigger1": if (!Need(0)) return true; EmitAsm("JSR", Abs("__zapper_trigger1")); return true;
                case "__zapper_trigger2":
                case "__zapper_trigger":
                    if (!Need(0)) return true; EmitAsm("JSR", Abs("__zapper_trigger2")); return true;
                case "__zapper_light1": if (!Need(0)) return true; EmitAsm("JSR", Abs("__zapper_light1")); return true;
                case "__zapper_light2":
                case "__zapper_light":
                    if (!Need(0)) return true; EmitAsm("JSR", Abs("__zapper_light2")); return true;
                // Keyboard detection has no arguments; scan destinations are words and row/column selectors are bytes.
                case "__fkb_detect": if (!Need(0)) return true; EmitAsm("JSR", Abs("__fkb_detect")); return true;
                case "__fkb_scan": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2}, funcName);
                case "__fkb_read_row_col": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1}, funcName);
                // Optical and serial operations delegate timing to their shared runtime helpers.
                case "__rob_flash": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, funcName);
                case "__rob_pulse": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1}, funcName);
                case "__rob_send_byte": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, funcName);
                case "__serial_tx_bit": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, funcName);
                case "__serial_rx_bit": if (!Need(0)) return true; EmitAsm("JSR", Abs("__serial_rx_bit")); return true;
                case "__midi_out_byte": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, funcName);
                case "__midi_in_byte": if (!Need(0)) return true; EmitAsm("JSR", Abs("__midi_in_byte")); return true;
                // Pack MIDI message fields as bytes; real-time commands below load their fixed status byte directly.
                case "__midi_note_on": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1}, funcName);
                case "__midi_note_off": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1}, funcName);
                case "__midi_control_change": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1}, funcName);
                case "__midi_program_change": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1}, funcName);
                // Real-time MIDI commands send a single status byte through the common output routine.
                case "__midi_clock": if (!Need(0)) return true; EmitAsm("LDA", Imm(0xF8)); EmitAsm("JSR", Abs("__kq_midi_out_a")); return true;
                case "__midi_start": if (!Need(0)) return true; EmitAsm("LDA", Imm(0xFA)); EmitAsm("JSR", Abs("__kq_midi_out_a")); return true;
                case "__midi_continue": if (!Need(0)) return true; EmitAsm("LDA", Imm(0xFB)); EmitAsm("JSR", Abs("__kq_midi_out_a")); return true;
                case "__midi_stop": if (!Need(0)) return true; EmitAsm("LDA", Imm(0xFC)); EmitAsm("JSR", Abs("__kq_midi_out_a")); return true;
                // Sprite helpers modify the runtime OAM representation; DMA remains a separate operation.
                case "__oam_clear": if (!Need(0)) return true; EmitAsm("JSR", Abs("__oam_clear")); return true;
                case "__sprite_set": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1,1,1}, funcName);
                case "__sprite_move": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1}, funcName);
                case "__sprite_tile": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1}, funcName);
                case "__sprite_attr": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1}, funcName);
                case "__sprite_hide": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, funcName);
                // Metasprites combine three byte arguments with a word-sized data pointer.
                case "__metasprite_draw": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1,2}, funcName);
                // Direct VRAM transfers use word addresses/pointers and byte transfer counts.
                case "__vram_write": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,1}, funcName);
                case "__vram_fill": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,1,1}, funcName);
                // Nametable and attribute helpers receive byte coordinates, values and optional table selectors.
                case "__nametable_put": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1}, funcName);
                case "__nametable_put_nt": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1,1}, funcName);
                case "__nametable_rect": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1,1,1}, funcName);
                case "__nametable_rect_nt": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1,1,1,1}, funcName);
                case "__attr_set": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1}, funcName);
                case "__attr_set_nt": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1,1}, funcName);
                // Palette loaders receive a word pointer; their fixed data layout is handled by the runtime helper.
                case "__palette_bg_load": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2}, funcName);
                case "__palette_sp_load": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2}, funcName);
                // Clear queued length and readiness; the overflow indicator has a separate reset intrinsic.
                case "__vramq_clear":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Imm(0)); EmitAsm("STA", Mem(_runtimeVramqLenAddress)); EmitAsm("STA", Mem(_runtimeVramqReadyAddress)); return true;
                // Publish queue readiness without executing its writes in the calling context.
                case "__vramq_commit":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Imm(1)); EmitAsm("STA", Mem(_runtimeVramqReadyAddress)); return true;
                // Execute the queue through the shared runtime routine; this dispatch adds no timing wait.
                case "__vramq_exec":
                    if (!Need(0)) return true;
                    EmitAsm("JSR", Abs("__vramq_exec")); return true;
                case "__vramq_len":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(_runtimeVramqLenAddress)); return true;
                case "__vramq_overflow":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(_runtimeVramqOverflowAddress)); return true;
                case "__vramq_clear_overflow":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Imm(0)); EmitAsm("STA", Mem(_runtimeVramqOverflowAddress)); return true;
                // Return the fixed queue-storage capacity byte; this is distinct from queued length or remaining space.
                case "__vramq_capacity":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Imm(VramqCapacity)); return true;
                // Queue helpers encode single writes, pointer-based copies or fills using explicit argument widths.
                case "__vramq_put": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,1}, funcName);
                case "__vramq_copy": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,2,1}, funcName);
                case "__vramq_fill": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2,1,1}, funcName);
                // CHR selection is implemented only for CNROM/MMC3 here; SUROM reserves the shared register bit for PRG selection.
                case "__chr_bank_set":
                case "__chr_bank_set0":
                case "__chr_bank_set1":
                    if (!Need(1)) return true;
                    if (Program.NesMapperProfile.IsSurom512)
                    {
                        Program.Error("error KQFC2605 KQFC-SUROM-CHR-ROM-FORBIDDEN: {0} is unavailable for board 'surom512'; CHR bank register bit 4 is reserved for outer PRG selection.", funcName);
                        return true;
                    }
                    Load1(args[0]);
                    if (Program.NesMapperProfile.MapperKind == NesMapperKind.Cnrom) { EmitAsm("STA", Mem(0x8000)); return true; }
                    if (Program.NesMapperProfile.MapperKind == NesMapperKind.Mmc3) { EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("LDA", Imm(funcName == "__chr_bank_set1" ? 1 : 0)); EmitAsm("STA", Mem(0x8000)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("STA", Mem(0x8001)); return true; }
                    Program.Error("error KQFC2406: {0} supports CNROM/MMC3 in this pass; current mapper is {1}.", funcName, Program.NesMapperProfile.CliName); return true;
                case "__mirroring_set":
                    if (!Need(1)) return true;
                    _mapperWritingFunctions.Add(_currentFunctionName);
                    return EmitMirroringSet(args[0], ctx);
                // MMC3 scanline setup writes the latch and reload registers; enabling the IRQ is a separate operation.
                case "__mapper_irq_set":
                case "__irq_scanline_set":
                    if (!Need(1)) return true;
                    if (Program.NesMapperProfile.MapperKind != NesMapperKind.Mmc3) { Program.Error("error KQFC2402: __irq_scanline_set currently supports MMC3 only."); return true; }
                    Load1(args[0]); EmitAsm("STA", Mem(0xC000)); EmitAsm("STA", Mem(0xC001)); return true;
                // For MMC3, the write address controls enable/disable behavior; the current A value is not interpreted here.
                case "__mapper_irq_enable":
                    if (!Need(0)) return true;
                    if (Program.NesMapperProfile.MapperKind == NesMapperKind.Mmc3) EmitAsm("STA", Mem(0xE001)); else Program.Error("error KQFC2403: __mapper_irq_enable currently supports MMC3 only."); return true;
                case "__mapper_irq_disable":
                case "__mapper_irq_ack":
                    if (!Need(0)) return true;
                    if (Program.NesMapperProfile.MapperKind == NesMapperKind.Mmc3) EmitAsm("STA", Mem(0xE000)); else Program.Error("error KQFC2404: {0} currently supports MMC3 only.", funcName); return true;
                // Sprite-zero synchronization and split scrolling remain delegated to their timing-sensitive runtime helpers.
                case "__sprite0_wait_hit": if (!Need(0)) return true; EmitAsm("JSR", Abs("__sprite0_wait_hit")); return true;
                case "__split_scroll_sprite0": return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1}, funcName);
            }
            return false;
        }

        // Inline constant lengths from 0 through 32; memset additionally requires a constant fill byte.
        // Longer or dynamic transfers fall back to the named helper.
        bool TryEmitMemcpyOrMemsetInline(string funcName, Expr[] args, FunctionContext ctx, bool isSet, int lenIndex, int lenSize)
        {
            args = args ?? Array.Empty<Expr>();
            int expected = isSet ? 3 : 3;
            if (args.Length != expected) return false;
            if (!TryEvaluateConstant(args[lenIndex], out int len)) return false;
            if (len < 0 || len > 32) return false;
            int valueConst = 0;
            if (isSet && !TryEvaluateConstant(args[1], out valueConst)) return false;
            // A zero-length expansion emits no address or fill evaluation in this helper.
            if (len == 0) return true;
            if (isSet)
            {
                int[] ptr = AcquireTemps(2);
                try
                {
                    EmitLoadValue(args[0], ctx, 2);
                    EmitAsm("STA", Mem(ptr[0]));
                    EmitAsm("STX", Mem(ptr[1]));
                    for (int i = 0; i < len; i++)
                    {
                        EmitAsm("LDY", Imm(i));
                        EmitAsm("LDA", Imm(valueConst));
                        EmitAsm("STA", IndY(ptr[0]));
                    }
                }
                finally { ReleaseTemps(ptr); }
                return true;
            }
            EmitFixedMemcpy(args[0], args[1], len, ctx);
            return true;
        }

        // Stage destination and source pointers, then emit a forward byte copy with constant Y offsets.
        // This helper does not provide overlap-safe memmove behavior.
        void EmitFixedMemcpy(Expr dstExpr, Expr srcExpr, int len, FunctionContext ctx)
        {
            int[] ptr = AcquireTemps(4);
            try
            {
                EmitLoadValue(dstExpr, ctx, 2);
                EmitAsm("STA", Mem(ptr[0]));
                EmitAsm("STX", Mem(ptr[1]));
                EmitLoadValue(srcExpr, ctx, 2);
                EmitAsm("STA", Mem(ptr[2]));
                EmitAsm("STX", Mem(ptr[3]));
                for (int i = 0; i < len; i++)
                {
                    EmitAsm("LDY", Imm(i));
                    EmitAsm("LDA", IndY(ptr[2]));
                    EmitAsm("STA", IndY(ptr[0]));
                }
            }
            finally { ReleaseTemps(ptr); }
        }

        // Specialize widths 8, 16, 32 and 64 into word shifts and a byte-X addition, returning A/X.
        bool TryEmitMapIndexInline(Expr[] args, FunctionContext ctx)
        {
            args = args ?? Array.Empty<Expr>();
            if (args.Length != 3) return false;
            if (!TryEvaluateConstant(args[2], out int width)) return false;
            if (width != 8 && width != 16 && width != 32 && width != 64) return false;
            int[] tmp = AcquireTemps(2);
            try
            {
                // result = y * width + x.  Width power-of-two cases are direct shifts.
                EmitLoadValue(args[1], ctx, 1);
                EmitAsm("STA", Mem(tmp[0]));
                EmitAsm("LDA", Imm(0));
                EmitAsm("STA", Mem(tmp[1]));
                int shifts = width == 8 ? 3 : width == 16 ? 4 : width == 32 ? 5 : 6;
                for (int i = 0; i < shifts; i++)
                {
                    EmitAsm("ASL", Mem(tmp[0]));
                    EmitAsm("ROL", Mem(tmp[1]));
                }
                EmitLoadValue(args[0], ctx, 1);
                EmitAsm("CLC");
                EmitAsm("ADC", Mem(tmp[0]));
                EmitAsm("STA", Mem(tmp[0]));
                EmitAsm("LDA", Mem(tmp[1]));
                EmitAsm("ADC", Imm(0));
                EmitAsm("TAX");
                EmitAsm("LDA", Mem(tmp[0]));
            }
            finally { ReleaseTemps(tmp); }
            return true;
        }

        // Use a constant bit index within a 256-byte indexed window to emit one byte read/modify/write.
        // Bit tests leave the masked bit value in A, without normalizing it to 1.
        bool TryEmitBitIntrinsicInline(string funcName, Expr[] args, FunctionContext ctx)
        {
            args = args ?? Array.Empty<Expr>();
            if (args.Length != 2) return false;
            if (!TryEvaluateConstant(args[1], out int bit) || bit < 0) return false;
            // Split the bit number into a byte displacement and an in-byte mask.
            int byteOffset = bit >> 3;
            if (byteOffset > 255) return false;
            int mask = 1 << (bit & 7);
            int[] ptr = AcquireTemps(2);
            try
            {
                EmitLoadValue(args[0], ctx, 2);
                EmitAsm("STA", Mem(ptr[0]));
                EmitAsm("STX", Mem(ptr[1]));
                EmitAsm("LDY", Imm(byteOffset));
                EmitAsm("LDA", IndY(ptr[0]));
                switch (funcName)
                {
                    case "__bit_test":
                        EmitAsm("AND", Imm(mask));
                        break;
                    case "__bit_set":
                        EmitAsm("ORA", Imm(mask));
                        EmitAsm("STA", IndY(ptr[0]));
                        break;
                    case "__bit_clear":
                        EmitAsm("AND", Imm((~mask) & 0xFF));
                        EmitAsm("STA", IndY(ptr[0]));
                        break;
                    case "__bit_toggle":
                        EmitAsm("EOR", Imm(mask));
                        EmitAsm("STA", IndY(ptr[0]));
                        break;
                    default:
                        return false;
                }
            }
            finally { ReleaseTemps(ptr); }
            return true;
        }

        // Translate the byte mode into the selected mapper register format; unsupported mappers produce a diagnostic.
        bool EmitMirroringSet(Expr modeExpr, FunctionContext ctx)
        {
            switch (Program.NesMapperProfile.MapperKind)
            {
                // Combine one-screen selection in bit 4 with the current PRG bank, converting the runtime bank from one-based numbering.
                case NesMapperKind.Axrom:
                    EmitLoadValue(modeExpr, ctx, 1);
                    EmitAsm("AND", Imm(1));
                    for (int i = 0; i < 4; i++) EmitAsm("ASL");
                    EmitAsm("STA", Mem(_runtimeAxromMirrorShadowAddress));
                    EmitAsm("LDA", Mem(_runtimeCurrentBankAddress));
                    EmitAsm("SEC");
                    EmitAsm("SBC", Imm(1));
                    EmitAsm("AND", Imm(0x0F));
                    EmitAsm("ORA", Mem(_runtimeAxromMirrorShadowAddress));
                    EmitAsm("STA", Mem(0x8000));
                    return true;

                // Expand the four logical modes into the four two-bit nametable selectors in MMC5 register 5105.
                case NesMapperKind.Mmc5:
                    EmitLoadValue(modeExpr, ctx, 1);
                    EmitAsm("AND", Imm(3));
                    EmitAsm("STA", Mem(_runtimeMapperTempAddress));
                    {
                        string vertical = NewGeneratedLabel("mmc5_mirror_vertical");
                        string one0 = NewGeneratedLabel("mmc5_mirror_one0");
                        string one1 = NewGeneratedLabel("mmc5_mirror_one1");
                        string store = NewGeneratedLabel("mmc5_mirror_store");
                        EmitAsm("CMP", Imm(1));
                        EmitAsm("BEQ", Rel(vertical));
                        EmitAsm("CMP", Imm(2));
                        EmitAsm("BEQ", Rel(one0));
                        EmitAsm("CMP", Imm(3));
                        EmitAsm("BEQ", Rel(one1));
                        // mode 0: horizontal = NT0,NT0,NT1,NT1 -> $50
                        EmitAsm("LDA", Imm(0x50));
                        EmitAsm("JMP", Abs(store));
                        _assembly.Add(Expr.Make(Tag.Label, vertical));
                        // mode 1: vertical = NT0,NT1,NT0,NT1 -> $44
                        EmitAsm("LDA", Imm(0x44));
                        EmitAsm("JMP", Abs(store));
                        _assembly.Add(Expr.Make(Tag.Label, one0));
                        EmitAsm("LDA", Imm(0x00));
                        EmitAsm("JMP", Abs(store));
                        _assembly.Add(Expr.Make(Tag.Label, one1));
                        EmitAsm("LDA", Imm(0x55));
                        _assembly.Add(Expr.Make(Tag.Label, store));
                        EmitAsm("STA", Mem(0x5105));
                    }
                    return true;

                case NesMapperKind.Mmc3:
                    EmitLoadValue(modeExpr, ctx, 1);
                    EmitAsm("AND", Imm(1));
                    EmitAsm("STA", Mem(0xA000));
                    return true;
                // Select FME7 register 0C, then publish the two-bit mirroring value.
                case NesMapperKind.Fme7:
                    EmitLoadValue(modeExpr, ctx, 1);
                    EmitAsm("AND", Imm(3));
                    EmitAsm("STA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("LDA", Imm(0x0C));
                    EmitAsm("STA", Mem(0x8000));
                    EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("STA", Mem(0xA000));
                    return true;
                case NesMapperKind.Mmc1:
                    if (Program.NesMapperProfile.IsSurom512)
                    {
                        EmitLoadValue(modeExpr, ctx, 1);
                        EmitAsm("AND", Imm(3));
                        EmitAsm("ORA", Imm(0x0C));
                        EmitAsm("STA", Mem(_runtimeMmc1ControlShadowAddress));
                        EmitAsm("PHP");
                        EmitAsm("SEI");
                        EmitAsm("LDA", Imm(0x80));
                        EmitAsm("STA", Mem(0x8000));
                        EmitAsm("LDA", Mem(_runtimeMmc1ControlShadowAddress));
                        // The SUROM control write runs under a saved IRQ status and preserves its control-register shadow.
                        EmitMmc1SerialWriteFromA(0x8000, "mmc1_surom_control");
                        EmitAsm("PLP");
                        return true;
                    }
                    EmitLoadValue(modeExpr, ctx, 1);
                    EmitAsm("AND", Imm(3));
                    EmitAsm("ORA", Imm(0x0C)); // keep 16K PRG fixed-at-top style while changing mirroring bits.
                    EmitAsm("STA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("LDA", Imm(0x80));
                    EmitAsm("STA", Mem(0x8000));
                    EmitAsm("LDX", Imm(5));
                    {
                        // Shift the ordinary MMC1 control value out least-significant bit first over five writes.
                        string loop = NewGeneratedLabel("mmc1_mirroring_loop");
                        _assembly.Add(Expr.Make(Tag.Label, loop));
                        EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                        EmitAsm("AND", Imm(1));
                        EmitAsm("STA", Mem(0x8000));
                        EmitAsm("LSR", Mem(_runtimeMapperTempAddress));
                        EmitAsm("DEX");
                        EmitAsm("BNE", Rel(loop));
                    }
                    return true;
                default:
                    Program.Error("error KQFC2410: __mirroring_set requires a mapper with runtime mirroring control (AxROM/MMC1/MMC3/MMC5/FME7); current mapper is {0}.", Program.NesMapperProfile.CliName);
                    return true;
            }
        }

        // Permit the availability query on every mapper, then enforce FDS for all remaining names.
        bool EmitFdsIntrinsicCall(string funcName, Expr[] args, FunctionContext ctx, Func<int, bool> Need, Action<Expr> Load1)
        {
            bool isFds = Program.NesMapperProfile.MapperKind == NesMapperKind.Fds;
            if (funcName == "__fds_available")
            {
                if (!Need(0)) return true;
                EmitAsm("LDA", Imm(isFds ? 1 : 0));
                return true;
            }
            if (!isFds)
            {
                Program.Error("error KQFC2420: {0} requires --mapper=fds / mapper20; current mapper is {1}.", funcName, Program.NesMapperProfile.CliName);
                return true;
            }
            switch (funcName)
            {
                // Return the inverted low status bit from 4032; this operation does not wait.
                case "__fds_disk_ready":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(0x4032));
                    EmitAsm("AND", Imm(0x01));
                    EmitAsm("EOR", Imm(0x01));
                    return true;
                // Expose status bit 2 as the raw mask value 0 or 4, rather than a normalized Boolean.
                case "__fds_side":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(0x4032));
                    EmitAsm("AND", Imm(0x04));
                    return true;
                // Read status register 4030 directly; the caller receives its complete byte value.
                case "__fds_error":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(0x4030));
                    return true;
                case "__fds_wait_ready":
                case "__fds_wait_insert":
                    if (!Need(0)) return true;
                    EmitAsm("JSR", Abs("__fds_wait_ready"));
                    return true;
                // Enable the configured disk/sound register gates and initialize the two sound-control registers.
                case "__fds_sound_enable":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Imm(0x83));
                    EmitAsm("STA", Mem(0x4023));
                    EmitAsm("LDA", Imm(0x00));
                    EmitAsm("STA", Mem(0x4089));
                    EmitAsm("LDA", Imm(0x00));
                    EmitAsm("STA", Mem(0x408A));
                    return true;
                // Wave, modulation and frequency operations pass word arguments to their dedicated helpers.
                case "__fds_wave_load":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2}, "__fds_wave_load");
                case "__fds_mod_load":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2}, "__fds_mod_load");
                case "__fds_freq_set":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {2}, "__fds_freq_set");
                // Clamp the direct volume field to six bits and set the fixed-volume control bit.
                case "__fds_volume_set":
                    if (!Need(1)) return true;
                    Load1(args[0]);
                    EmitAsm("AND", Imm(0x3F));
                    EmitAsm("ORA", Imm(0x80));
                    EmitAsm("STA", Mem(0x4080));
                    return true;
                case "__fds_env_set":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,1,1}, "__fds_env_set");
                // File helpers use byte identifiers and word memory addresses/lengths.
                case "__fds_load_file":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,2}, "__fds_load_file");
                case "__fds_save_file":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1,2,2}, "__fds_save_file");
                // Overlay/bank-loading helpers receive a byte bank selector; residency queries use the runtime state below.
                case "__fds_load_overlay":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, "__fds_load_overlay");
                case "__fds_load_bank":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, "__fds_load_bank");
                case "__fds_require_bank":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, "__fds_require_bank");
                // Read the runtime resident-bank byte without starting a disk transfer.
                case "__fds_current_bank":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Mem(_runtimeFdsResidentBankAddress));
                    return true;
                case "__fds_is_bank_resident":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, "__fds_is_bank_resident");
                // Read the first byte of the emitted overlay-function table as its count.
                case "__fds_overlay_function_count":
                    if (!Need(0)) return true;
                    EmitAsm("LDA", Abs("__kq_fds_overlay_function_table"));
                    return true;
                case "__fds_overlay_farcall":
                case "__fds_farcall":
                    Program.Error("error KQFC2515: {0}(bank, func) must be used as a direct call expression with a function name as the second argument.", funcName);
                    return true;
                case "__fds_file_exists":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, "__fds_file_exists");
                case "__fds_file_size":
                    return EmitKitaqfcHelperCall(funcName, args, ctx, new[] {1}, "__fds_file_size");
                default:
                    Program.Error("error KQFC2422: unsupported FDS intrinsic {0}.", funcName);
                    return true;
            }
        }

        // Evaluate arguments into scratch bytes before copying them into the shared call area and issuing JSR.
        // Each signature entry is normalized to one or two bytes.
        bool EmitKitaqfcHelperCall(string publicName, Expr[] args, FunctionContext ctx, int[] parameterSizes, string helperName)
        {
            args = args ?? Array.Empty<Expr>();
            if (args.Length != parameterSizes.Length)
            {
                Program.Error("error KQ0000: {0} expects {1} argument(s), got {2}.", publicName, parameterSizes.Length, args.Length);
                return true;
            }
            int total = parameterSizes.Sum(x => x <= 1 ? 1 : 2);
            int[] scratch = total > 0 ? AcquireTemps(total) : Array.Empty<int>();
            try
            {
                int off = 0;
                for (int i = 0; i < args.Length; i++)
                {
                    int size = parameterSizes[i] <= 1 ? 1 : 2;
                    EmitLoadValue(args[i], ctx, size);
                    EmitAsm("STA", Mem(scratch[off]));
                    if (size == 2) EmitAsm("STX", Mem(scratch[off + 1]));
                    off += size;
                }
                for (int i = 0; i < total; i++) { EmitAsm("LDA", Mem(scratch[i])); EmitAsm("STA", Mem(CallArgBase + i)); }
                EmitAsm("JSR", Abs(helperName));
            }
            finally { if (scratch.Length != 0) ReleaseTemps(scratch); }
            return true;
        }

        // Update named one- or two-byte storage, scaling pointer steps by the complete pointee size.
        // The sign of delta selects increment versus decrement; the optional result is the updated value.
        void EmitIncDec(Expr target, int delta, FunctionContext ctx, bool wantValue)
        {
            string name;
            if (!target.Match(Tag.Name, out name))
            {
                Program.Error("error KQ0000: --target=nes phase 5 increment/decrement currently supports only simple names.");
                return;
            }
            StorageSlot slot;
            if (!TryResolveStorage(ctx, name, out slot))
            {
                Program.Error("error KQ0000: unknown symbol in increment/decrement for --target=nes: {0}", name);
                return;
            }
            if (slot.IsConstant)
            {
                Program.Error("error KQ0000: cannot modify constant '{0}' in --target=nes phase 5.", name);
                return;
            }
            int step = 1;
            if (slot.Type != null && slot.Type.IsPointer)
                step = GetPointerElementStride(slot.Type);
            if (slot.Size == 1)
            {
                if (delta >= 0)
                {
                    if (step == 1)
                    {
                        EmitAsm("INC", Mem(slot.Address));
                    }
                    else
                    {
                        EmitAsm("CLC");
                        EmitAsm("LDA", Mem(slot.Address));
                        EmitAsm("ADC", Imm(step & 0xFF));
                        EmitAsm("STA", Mem(slot.Address));
                    }
                }
                else
                {
                    if (step == 1)
                    {
                        EmitAsm("DEC", Mem(slot.Address));
                    }
                    else
                    {
                        EmitAsm("SEC");
                        EmitAsm("LDA", Mem(slot.Address));
                        EmitAsm("SBC", Imm(step & 0xFF));
                        EmitAsm("STA", Mem(slot.Address));
                    }
                }
            }
            else if (slot.Size == 2)
            {
                if (delta >= 0)
                {
                    EmitAddConstantToPair(slot.Address, slot.Address + 1, step);
                }
                else
                {
                    EmitSubConstantFromPair(slot.Address, slot.Address + 1, step);
                }
            }
            else
            {
                Program.Error("error KQ0000: --target=nes phase 5 increment/decrement currently supports only 1-byte or 2-byte storage: {0}", name);
                return;
            }

            if (wantValue)
                LoadFromSlot(slot);
        }

        // Request the one-byte expression-loading path.
        void EmitLoadA(Expr expr, FunctionContext ctx)
        {
            EmitLoadValue(expr, ctx, 1);
        }

        // Emit a scalar result in A, or A/X for a word, selecting code by expression shape.
        // Requested width is normalized to one or two bytes; aggregate values use separate copy paths.
        void EmitLoadValue(Expr expr, FunctionContext ctx, int expectedSize)
        {
            expectedSize = NormalizeScalarSize(expectedSize);
            if (expr == null)
            {
                EmitAsm("LDA", Imm(0));
                if (expectedSize == 2) EmitAsm("LDX", Imm(0));
                return;
            }

            int constValue;
            string name;
            Expr lhs;
            Expr rhs;
            Expr[] args;
            Expr funcExpr;
            CType castType;
            Expr castSubexpr;
            Expr condExpr;
            Expr trueExpr;
            Expr falseExpr;
            CType exprType;

            if (TryEvaluateConstant(expr, out constValue))
            {
                EmitAsm("LDA", Imm(constValue & 0xFF));
                if (expectedSize == 2) EmitAsm("LDX", Imm((constValue >> 8) & 0xFF));
                return;
            }

            if (expr.Match(Tag.Name, out name))
            {
                StorageSlot slot;
                if (TryResolveStorage(ctx, name, out slot))
                {
                    if (slot.IsConstant)
                    {
                        EmitAsm("LDA", Imm(slot.ConstantValue & 0xFF));
                        if (expectedSize == 2) EmitAsm("LDX", Imm((slot.ConstantValue >> 8) & 0xFF));
                    }
                    // Named readonly data and arrays produce their base address, not a load of the first element.
                    else if (slot.IsReadonlyData || (slot.Type != null && slot.Type.IsArray))
                    {
                        if (slot.IsReadonlyData)
                        {
                            EmitAsm("LDA", ImmLo(name));
                            if (expectedSize == 2) EmitAsm("LDX", ImmHi(name));
                        }
                        else
                        {
                            EmitAsm("LDA", Imm(slot.Address & 0xFF));
                            if (expectedSize == 2) EmitAsm("LDX", Imm((slot.Address >> 8) & 0xFF));
                        }
                    }
                    else if (slot.Type != null && slot.Type.IsStructOrUnion)
                    {
                        Program.Error("error KQ0000: struct/union object '{0}' cannot be used as a value in --target=nes phase 6.", name);
                    }
                    else
                    {
                        EmitAsm("LDA", Mem(slot.Address));
                        if (expectedSize == 2)
                        {
                            if (slot.Size == 2) EmitAsm("LDX", Mem(slot.Address + 1));
                            else EmitAsm("LDX", Imm(0));
                        }
                    }
                    return;
                }
                Program.Error("error KQ0000: unknown symbol in expression for --target=nes: {0}", name);
                return;
            }

            if (expr.Match(Tag.AddressOf, out lhs))
            {
                EmitAddressOfValue(lhs, ctx);
                if (expectedSize == 1)
                {
                    // Pointer expressions are naturally 16-bit; keep low byte in A.
                }
                return;
            }

            if (TryGetExprType(expr, ctx, out exprType))
            {
                if (exprType != null && exprType.IsArray)
                {
                    EmitAddressOfValue(expr, ctx);
                    return;
                }
                if (exprType != null && exprType.IsStructOrUnion)
                {
                    Program.Error("error KQ0000: struct/union expression cannot be used as a value in --target=nes phase 6.");
                    return;
                }
            }

            // Delegate addressable scalar reads, then clear X when a byte lvalue is requested as a word.
            if (expr.Match(Tag.Load, out lhs) || expr.Match(Tag.Index, out lhs, out rhs) || expr.Match(Tag.Field, out lhs, out name))
            {
                EmitLoadIndirectValue(expr, ctx, expectedSize);
                if (expectedSize == 2 && DetermineExprSize(expr, ctx) == 1)
                    EmitAsm("LDX", Imm(0));
                return;
            }

            if (expr.Match(Tag.Assign, out lhs, out rhs))
            {
                if (TryEmitAggregateAssignment(expr, lhs, rhs, ctx))
                    return;
                // Use the destination width for assignment, retaining its stored result and zero-extending byte assignments when requested.
                int assignSize = GetLvalueSize(lhs, ctx, requireWritable: true);
                if (Program.ErrorCount > 0) return;
                EmitLoadValue(rhs, ctx, assignSize);
                EmitStoreLoadedValueToLvalue(lhs, ctx, assignSize, preserveResult: true);
                if (expectedSize == 2 && assignSize == 1)
                    EmitAsm("LDX", Imm(0));
                return;
            }

            string opName;
            if (expr.Match(Tag.AssignModify, out opName, out lhs, out rhs))
            {
                // Rebuild compound assignment as a load/operation/store expression for the normal assignment path.
                EmitLoadValue(Expr.Make(Tag.Assign, lhs, Expr.Make(opName, lhs, rhs).WithSource(expr.Source)).WithSource(expr.Source), ctx, expectedSize);
                return;
            }

            if (expr.MatchAny(Tag.Call, out funcExpr, out args))
            {
                CType callType;
                if (TryGetExprType(expr, ctx, out callType) && IsAggregateType(callType))
                {
                    Program.Error(Maybe.Just(expr.Source), ErrorCode.ParseError,
                        "struct/union call result cannot be used as a scalar value");
                    return;
                }
                // Call using the declared result width before widening a byte return for its enclosing expression.
                int returnSize = NormalizeScalarSize(DetermineExprSize(expr, ctx));
                EmitCall(expr.Source, funcExpr, args, ctx, returnSize);
                if (expectedSize == 2 && returnSize == 1)
                {
                    // A byte return does not define X. Widen the declared value,
                    // not the stale high register left by the callee.
                    EmitAsm("LDX", Imm(0));
                    if (callType != null && IsSignedIntegerType(callType))
                    {
                        string extended = NewGeneratedLabel("call_sign_extended");
                        EmitAsm("CMP", Imm(0x80));
                        EmitAsm("BCC", Rel(extended));
                        EmitAsm("LDX", Imm(0xFF));
                        _assembly.Add(Expr.Make(Tag.Label, extended).WithSource(expr.Source));
                    }
                }
                return;
            }

            if (expr.Match(Tag.Cast, out castType, out castSubexpr))
            {
                // Load enough bytes for either side of the cast, then clear X for a byte cast used as a word.
                int castSize = NormalizeScalarSize(TypeStorageSize(castType));
                int sourceSize = NormalizeScalarSize(DetermineExprSize(castSubexpr, ctx));
                int loadSize = Math.Max(castSize, sourceSize);
                EmitLoadValue(castSubexpr, ctx, loadSize);
                if (expectedSize == 2 && castSize == 1)
                    EmitAsm("LDX", Imm(0));
                return;
            }
            if (expr.Match(Tag.Conditional, out condExpr, out trueExpr, out falseExpr))
            {
                // Select one conditional arm at runtime and request the same result width from either arm.
                string falseLabel = NewGeneratedLabel("cond_false");
                string doneLabel = NewGeneratedLabel("cond_done");
                EmitBranchIfFalse(condExpr, falseLabel, ctx);
                EmitLoadValue(trueExpr, ctx, expectedSize);
                EmitAsm("JMP", Abs(doneLabel));
                _assembly.Add(Expr.Make(Tag.Label, falseLabel).WithSource(expr.Source));
                EmitLoadValue(falseExpr, ctx, expectedSize);
                _assembly.Add(Expr.Make(Tag.Label, doneLabel).WithSource(expr.Source));
                return;
            }

            if (expr.Match(Tag.PreIncrement, out lhs))
            {
                EmitIncDec(lhs, +1, ctx, true);
                if (expectedSize == 2 && DetermineExprSize(expr, ctx) == 1) EmitAsm("LDX", Imm(0));
                return;
            }
            if (expr.Match(Tag.PostIncrement, out lhs))
            {
                string postIncName;
                if (!lhs.Match(Tag.Name, out postIncName))
                {
                    Program.Error("error KQ0000: --target=nes phase 5 increment/decrement currently supports only simple names.");
                    return;
                }
                StorageSlot postIncSlot;
                if (!TryResolveStorage(ctx, postIncName, out postIncSlot))
                {
                    Program.Error("error KQ0000: unknown symbol in increment/decrement for --target=nes: {0}", postIncName);
                    return;
                }
                if (postIncSlot.IsConstant)
                {
                    Program.Error("error KQ0000: cannot modify constant '{0}' in --target=nes phase 5.", postIncName);
                    return;
                }

                // Load the named pre-update value before emitting the separate storage increment.
                LoadFromSlot(postIncSlot);
                EmitIncDec(lhs, +1, ctx, false);
                if (expectedSize == 2 && DetermineExprSize(expr, ctx) == 1) EmitAsm("LDX", Imm(0));
                return;
            }
            if (expr.Match(Tag.PreDecrement, out lhs))
            {
                EmitIncDec(lhs, -1, ctx, true);
                if (expectedSize == 2 && DetermineExprSize(expr, ctx) == 1) EmitAsm("LDX", Imm(0));
                return;
            }
            if (expr.Match(Tag.PostDecrement, out lhs))
            {
                string postDecName;
                if (!lhs.Match(Tag.Name, out postDecName))
                {
                    Program.Error("error KQ0000: --target=nes phase 5 increment/decrement currently supports only simple names.");
                    return;
                }
                StorageSlot postDecSlot;
                if (!TryResolveStorage(ctx, postDecName, out postDecSlot))
                {
                    Program.Error("error KQ0000: unknown symbol in increment/decrement for --target=nes: {0}", postDecName);
                    return;
                }
                if (postDecSlot.IsConstant)
                {
                    Program.Error("error KQ0000: cannot modify constant '{0}' in --target=nes phase 5.", postDecName);
                    return;
                }

                // Load the named pre-update value before emitting the separate storage decrement.
                LoadFromSlot(postDecSlot);
                EmitIncDec(lhs, -1, ctx, false);
                if (expectedSize == 2 && DetermineExprSize(expr, ctx) == 1) EmitAsm("LDX", Imm(0));
                return;
            }

            if (expr.Match(Tag.Add, out lhs, out rhs))
            {
                EmitBinaryMath(lhs, rhs, ctx, add: true, size: expectedSize);
                return;
            }
            if (expr.Match(Tag.Subtract, out lhs, out rhs))
            {
                EmitBinaryMath(lhs, rhs, ctx, add: false, size: expectedSize);
                return;
            }
            // Delegate multiplication/division/remainder to their arithmetic emitters rather than the requested-width add/subtract path.
            if (expr.Match(Tag.Multiply, out lhs, out rhs))
            {
                EmitMultiplyExpression(lhs, rhs, ctx);
                return;
            }
            if (expr.Match(Tag.Divide, out lhs, out rhs))
            {
                EmitDivideOrModulusExpression(lhs, rhs, ctx, wantRemainder: false);
                return;
            }
            if (expr.Match(Tag.Modulus, out lhs, out rhs))
            {
                EmitDivideOrModulusExpression(lhs, rhs, ctx, wantRemainder: true);
                return;
            }
            // Bitwise operations apply independently to the requested byte or word result.
            if (expr.Match(Tag.BitwiseAnd, out lhs, out rhs))
            {
                EmitBinaryLogical("AND", lhs, rhs, ctx, expectedSize);
                return;
            }
            if (expr.Match(Tag.BitwiseOr, out lhs, out rhs))
            {
                EmitBinaryLogical("ORA", lhs, rhs, ctx, expectedSize);
                return;
            }
            if (expr.Match(Tag.BitwiseXor, out lhs, out rhs))
            {
                EmitBinaryLogical("EOR", lhs, rhs, ctx, expectedSize);
                return;
            }
            if (expr.Match(Tag.LogicalNot, out lhs))
            {
                EmitBooleanToA(expr, ctx);
                if (expectedSize == 2) EmitAsm("LDX", Imm(0));
                return;
            }
            if (expr.Match(Tag.BitwiseNot, out lhs))
            {
                if (expectedSize == 1)
                {
                    EmitLoadA(lhs, ctx);
                    EmitAsm("EOR", Imm(0xFF));
                }
                else
                {
                    int[] temps = AcquireTemps(2);
                    try
                    {
                        EmitLoadValue(lhs, ctx, 2);
                        EmitAsm("STA", Mem(temps[0]));
                        EmitAsm("TXA");
                        EmitAsm("EOR", Imm(0xFF));
                        EmitAsm("TAX");
                        EmitAsm("LDA", Mem(temps[0]));
                        EmitAsm("EOR", Imm(0xFF));
                    }
                    finally
                    {
                        ReleaseTemps(temps);
                    }
                }
                return;
            }
            if (expr.Match(Tag.ShiftLeft, out lhs, out rhs))
            {
                EmitShift(lhs, rhs, ctx, expectedSize, leftShift: true);
                return;
            }
            if (expr.Match(Tag.ShiftRight, out lhs, out rhs))
            {
                EmitShift(lhs, rhs, ctx, expectedSize, leftShift: false);
                return;
            }
            // Materialize comparison/logical results as a Boolean byte, clearing X for word contexts.
            if (IsComparisonTag(expr.Tag) || expr.Tag == Tag.LogicalAnd || expr.Tag == Tag.LogicalOr)
            {
                EmitBooleanToA(expr, ctx);
                if (expectedSize == 2) EmitAsm("LDX", Imm(0));
                return;
            }

            Program.Error("error KQ0000: --target=nes phase 5 unsupported expression tag in function {0}: {1}", ctx.Name, expr.Tag);
        }

        // Scale integer offsets for pointer addition/subtraction; otherwise emit byte or word arithmetic.
        void EmitBinaryMath(Expr lhs, Expr rhs, FunctionContext ctx, bool add, int size)
        {
            CType leftType;
            CType rightType;
            bool haveLeftType = TryGetExprType(lhs, ctx, out leftType);
            bool haveRightType = TryGetExprType(rhs, ctx, out rightType);
            bool leftPointerLike = haveLeftType && leftType != null && (leftType.IsPointer || leftType.IsArray);
            bool rightPointerLike = haveRightType && rightType != null && (rightType.IsPointer || rightType.IsArray);
            if ((leftPointerLike && !rightPointerLike) || (add && rightPointerLike && !leftPointerLike))
            {
                Expr baseExpr = leftPointerLike ? lhs : rhs;
                Expr offsetExpr = leftPointerLike ? rhs : lhs;
                CType pointerType = leftPointerLike ? leftType : rightType;
                int stride = GetPointerElementStride(pointerType);
                int constOffset;
                if (TryEvaluateConstant(offsetExpr, out constOffset))
                {
                    EmitLoadValue(baseExpr, ctx, 2);
                    int[] baseTemps = AcquireTemps(2);
                    try
                    {
                        EmitAsm("STA", Mem(baseTemps[0]));
                        EmitAsm("STX", Mem(baseTemps[1]));
                        // Fold the element displacement into the 16-bit address domain before adding or subtracting it.
                        int scaledOffset = (constOffset * stride) & 0xFFFF;
                        if (add)
                            EmitAddConstantToPair(baseTemps[0], baseTemps[1], scaledOffset);
                        else
                            EmitSubConstantFromPair(baseTemps[0], baseTemps[1], scaledOffset);
                        EmitAsm("LDA", Mem(baseTemps[0]));
                        EmitAsm("LDX", Mem(baseTemps[1]));
                    }
                    finally
                    {
                        ReleaseTemps(baseTemps);
                    }
                    return;
                }

                int[] ptrTemps = AcquireTemps(4);
                try
                {
                    // For dynamic pointer arithmetic, stage the scaled offset before evaluating the base address.
                    EmitLoadValue(offsetExpr, ctx, 2);
                    EmitAsm("STA", Mem(ptrTemps[0]));
                    EmitAsm("STX", Mem(ptrTemps[1]));
                    EmitScalePairByConstant(ptrTemps[0], ptrTemps[1], stride);

                    EmitLoadValue(baseExpr, ctx, 2);
                    EmitAsm("STA", Mem(ptrTemps[2]));
                    EmitAsm("STX", Mem(ptrTemps[3]));

                    if (add)
                        EmitAddPairToPair(ptrTemps[2], ptrTemps[3], ptrTemps[0], ptrTemps[1]);
                    else
                        EmitSubPairFromPair(ptrTemps[2], ptrTemps[3], ptrTemps[0], ptrTemps[1]);

                    EmitAsm("LDA", Mem(ptrTemps[2]));
                    EmitAsm("LDX", Mem(ptrTemps[3]));
                }
                finally
                {
                    ReleaseTemps(ptrTemps);
                }
                return;
            }

            size = NormalizeScalarSize(size);
            if (size == 1)
            {
                int rhsConst;
                if (TryEvaluateConstant(rhs, out rhsConst))
                {
                    EmitLoadA(lhs, ctx);
                    EmitAsm(add ? "CLC" : "SEC");
                    EmitAsm(add ? "ADC" : "SBC", Imm(rhsConst));
                    return;
                }

                int temp = AcquireTemp();
                try
                {
                    EmitLoadA(rhs, ctx);
                    EmitAsm("STA", Mem(temp));
                    EmitLoadA(lhs, ctx);
                    EmitAsm(add ? "CLC" : "SEC");
                    EmitAsm(add ? "ADC" : "SBC", Mem(temp));
                }
                finally
                {
                    ReleaseTemp(temp);
                }
                return;
            }

            int[] temps = AcquireTemps(3);
            try
            {
                EmitLoadValue(rhs, ctx, 2);
                EmitAsm("STA", Mem(temps[0]));
                EmitAsm("STX", Mem(temps[1]));

                EmitLoadValue(lhs, ctx, 2);
                EmitAsm(add ? "CLC" : "SEC");
                EmitAsm(add ? "ADC" : "SBC", Mem(temps[0]));
                EmitAsm("STA", Mem(temps[2]));
                EmitAsm("TXA");
                EmitAsm(add ? "ADC" : "SBC", Mem(temps[1]));
                EmitAsm("TAX");
                EmitAsm("LDA", Mem(temps[2]));
            }
            finally
            {
                ReleaseTemps(temps);
            }
        }

        // Use an immediate RHS when possible; otherwise stage it while evaluating the left operand.
        // Word operations combine low and high bytes separately.
        void EmitBinaryLogical(string mnemonic, Expr lhs, Expr rhs, FunctionContext ctx, int size)
        {
            size = NormalizeScalarSize(size);
            if (size == 1)
            {
                int rhsConst;
                if (TryEvaluateConstant(rhs, out rhsConst))
                {
                    EmitLoadA(lhs, ctx);
                    EmitAsm(mnemonic, Imm(rhsConst));
                    return;
                }

                int temp = AcquireTemp();
                try
                {
                    EmitLoadA(rhs, ctx);
                    EmitAsm("STA", Mem(temp));
                    EmitLoadA(lhs, ctx);
                    EmitAsm(mnemonic, Mem(temp));
                }
                finally
                {
                    ReleaseTemp(temp);
                }
                return;
            }

            int[] temps = AcquireTemps(3);
            try
            {
                EmitLoadValue(rhs, ctx, 2);
                EmitAsm("STA", Mem(temps[0]));
                EmitAsm("STX", Mem(temps[1]));

                EmitLoadValue(lhs, ctx, 2);
                EmitAsm(mnemonic, Mem(temps[0]));
                EmitAsm("STA", Mem(temps[2]));
                EmitAsm("TXA");
                EmitAsm(mnemonic, Mem(temps[1]));
                EmitAsm("TAX");
                EmitAsm("LDA", Mem(temps[2]));
            }
            finally
            {
                ReleaseTemps(temps);
            }
        }

        // Prefer an eligible constant lookup, otherwise emit a sixteen-step word multiply returning the low result word.
        void EmitMultiplyExpression(Expr lhs, Expr rhs, FunctionContext ctx)
        {
            bool signedArithmetic = ShouldUseSignedArithmetic(lhs, rhs, ctx);
            if (TryEvaluateConstant(rhs, out int rhsConst) && TryEmitConstantLookupArithmetic("mul", lhs, rhsConst, ctx, signedArithmetic))
                return;
            if (TryEvaluateConstant(lhs, out int lhsConst) && TryEmitConstantLookupArithmetic("mul", rhs, lhsConst, ctx, signedArithmetic))
                return;

            int[] temps = AcquireTemps(signedArithmetic ? 7 : 6);
            int leftLo = temps[0];
            int leftHi = temps[1];
            int rightLo = temps[2];
            int rightHi = temps[3];
            int resultLo = temps[4];
            int resultHi = temps[5];
            int signTemp = signedArithmetic ? temps[6] : -1;

            try
            {
                EmitLoadArithmeticOperandToTemps(lhs, ctx, leftLo, leftHi, signedArithmetic && IsSignedArithmeticOperand(lhs, ctx));
                EmitLoadArithmeticOperandToTemps(rhs, ctx, rightLo, rightHi, signedArithmetic && IsSignedArithmeticOperand(rhs, ctx));

                if (signedArithmetic)
                {
                    EmitAsm("LDA", Mem(leftHi));
                    EmitAsm("EOR", Mem(rightHi));
                    EmitAsm("AND", Imm(0x80));
                    EmitAsm("STA", Mem(signTemp));

                    // For signed multiplication, save the result sign and convert operands to magnitudes before the unsigned loop.
                    string leftPositive = NewGeneratedLabel("mul_lhs_positive");
                    EmitAsm("LDA", Mem(leftHi));
                    EmitAsm("BPL", Rel(leftPositive));
                    EmitNegateTempPair(leftLo, leftHi);
                    _assembly.Add(Expr.Make(Tag.Label, leftPositive).WithSource(lhs.Source));

                    string rightPositive = NewGeneratedLabel("mul_rhs_positive");
                    EmitAsm("LDA", Mem(rightHi));
                    EmitAsm("BPL", Rel(rightPositive));
                    EmitNegateTempPair(rightLo, rightHi);
                    _assembly.Add(Expr.Make(Tag.Label, rightPositive).WithSource(rhs.Source));
                }

                EmitAsm("LDA", Imm(0));
                EmitAsm("STA", Mem(resultLo));
                EmitAsm("STA", Mem(resultHi));

                // Accumulate selected shifted multiplicands while consuming one multiplier bit per iteration.
                string loopLabel = NewGeneratedLabel("mul_loop");
                string skipAddLabel = NewGeneratedLabel("mul_skip_add");
                EmitAsm("LDY", Imm(16));
                _assembly.Add(Expr.Make(Tag.Label, loopLabel).WithSource(lhs.Source));
                EmitAsm("LDA", Mem(rightLo));
                EmitAsm("AND", Imm(1));
                EmitAsm("BEQ", Rel(skipAddLabel));

                EmitAsm("CLC");
                EmitAsm("LDA", Mem(resultLo));
                EmitAsm("ADC", Mem(leftLo));
                EmitAsm("STA", Mem(resultLo));
                EmitAsm("LDA", Mem(resultHi));
                EmitAsm("ADC", Mem(leftHi));
                EmitAsm("STA", Mem(resultHi));

                _assembly.Add(Expr.Make(Tag.Label, skipAddLabel).WithSource(lhs.Source));
                EmitAsm("ASL", Mem(leftLo));
                EmitAsm("ROL", Mem(leftHi));
                EmitAsm("LSR", Mem(rightHi));
                EmitAsm("ROR", Mem(rightLo));
                EmitAsm("DEY");
                EmitAsm("BNE", Rel(loopLabel));

                if (signedArithmetic)
                {
                    // Restore the product sign after the magnitude loop; arithmetic remains modulo 65536.
                    string signDone = NewGeneratedLabel("mul_sign_done");
                    EmitAsm("LDA", Mem(signTemp));
                    EmitAsm("BEQ", Rel(signDone));
                    EmitNegateTempPair(resultLo, resultHi);
                    _assembly.Add(Expr.Make(Tag.Label, signDone).WithSource(lhs.Source));
                }

                EmitAsm("LDA", Mem(resultLo));
                EmitAsm("LDX", Mem(resultHi));
            }
            finally
            {
                ReleaseTemps(temps);
            }
        }

        // Prefer an eligible constant lookup, otherwise emit word division and select quotient or remainder in A/X.
        void EmitDivideOrModulusExpression(Expr lhs, Expr rhs, FunctionContext ctx, bool wantRemainder)
        {
            bool signedArithmetic = ShouldUseSignedArithmetic(lhs, rhs, ctx);
            if (TryEvaluateConstant(rhs, out int divisorConst) &&
                TryEmitConstantLookupArithmetic(wantRemainder ? "mod" : "div", lhs, divisorConst, ctx, signedArithmetic))
                return;

            int[] temps = AcquireTemps(signedArithmetic ? 10 : 8);
            int dividendLo = temps[0];
            int dividendHi = temps[1];
            int divisorLo = temps[2];
            int divisorHi = temps[3];
            int quotientLo = temps[4];
            int quotientHi = temps[5];
            int remainderLo = temps[6];
            int remainderHi = temps[7];
            int signQTemp = signedArithmetic ? temps[8] : -1;
            int signRTemp = signedArithmetic ? temps[9] : -1;

            try
            {
                EmitLoadArithmeticOperandToTemps(lhs, ctx, dividendLo, dividendHi, signedArithmetic && IsSignedArithmeticOperand(lhs, ctx));
                EmitLoadArithmeticOperandToTemps(rhs, ctx, divisorLo, divisorHi, signedArithmetic && IsSignedArithmeticOperand(rhs, ctx));

                if (signedArithmetic)
                {
                    EmitAsm("LDA", Mem(dividendHi));
                    EmitAsm("AND", Imm(0x80));
                    EmitAsm("STA", Mem(signRTemp));

                    EmitAsm("LDA", Mem(dividendHi));
                    EmitAsm("EOR", Mem(divisorHi));
                    EmitAsm("AND", Imm(0x80));
                    EmitAsm("STA", Mem(signQTemp));

                    // Save separate quotient/remainder signs, then divide operand magnitudes.
                    string dividendPositive = NewGeneratedLabel("div_dividend_positive");
                    EmitAsm("LDA", Mem(dividendHi));
                    EmitAsm("BPL", Rel(dividendPositive));
                    EmitNegateTempPair(dividendLo, dividendHi);
                    _assembly.Add(Expr.Make(Tag.Label, dividendPositive).WithSource(lhs.Source));

                    string divisorPositive = NewGeneratedLabel("div_divisor_positive");
                    EmitAsm("LDA", Mem(divisorHi));
                    EmitAsm("BPL", Rel(divisorPositive));
                    EmitNegateTempPair(divisorLo, divisorHi);
                    _assembly.Add(Expr.Make(Tag.Label, divisorPositive).WithSource(rhs.Source));
                }

                EmitAsm("LDA", Mem(divisorLo));
                EmitAsm("ORA", Mem(divisorHi));
                // A zero divisor takes the explicit zero-quotient/zero-remainder path in this emitter.
                string nonZeroDivisor = NewGeneratedLabel("div_non_zero");
                string doneLabel = NewGeneratedLabel("div_done");
                EmitAsm("BNE", Rel(nonZeroDivisor));

                EmitAsm("LDA", Imm(0));
                EmitAsm("STA", Mem(quotientLo));
                EmitAsm("STA", Mem(quotientHi));
                EmitAsm("STA", Mem(remainderLo));
                EmitAsm("STA", Mem(remainderHi));
                EmitAsm("JMP", Abs(doneLabel));

                _assembly.Add(Expr.Make(Tag.Label, nonZeroDivisor).WithSource(rhs.Source));
                EmitAsm("LDA", Imm(0));
                EmitAsm("STA", Mem(quotientLo));
                EmitAsm("STA", Mem(quotientHi));
                EmitAsm("STA", Mem(remainderLo));
                EmitAsm("STA", Mem(remainderHi));

                // Shift sixteen dividend bits into the remainder, subtracting the divisor when the unsigned remainder is large enough.
                string loopLabel = NewGeneratedLabel("div_loop");
                string skipSubtract = NewGeneratedLabel("div_skip_subtract");
                string doSubtract = NewGeneratedLabel("div_do_subtract");
                string quotientCarryDone = NewGeneratedLabel("div_quotient_carry_done");

                EmitAsm("LDY", Imm(16));
                _assembly.Add(Expr.Make(Tag.Label, loopLabel).WithSource(lhs.Source));
                EmitAsm("ASL", Mem(dividendLo));
                EmitAsm("ROL", Mem(dividendHi));
                EmitAsm("ROL", Mem(remainderLo));
                EmitAsm("ROL", Mem(remainderHi));
                EmitAsm("ASL", Mem(quotientLo));
                EmitAsm("ROL", Mem(quotientHi));

                EmitAsm("LDA", Mem(remainderHi));
                EmitAsm("CMP", Mem(divisorHi));
                EmitAsm("BCC", Rel(skipSubtract));
                EmitAsm("BNE", Rel(doSubtract));
                EmitAsm("LDA", Mem(remainderLo));
                EmitAsm("CMP", Mem(divisorLo));
                EmitAsm("BCC", Rel(skipSubtract));

                _assembly.Add(Expr.Make(Tag.Label, doSubtract).WithSource(lhs.Source));
                EmitAsm("SEC");
                EmitAsm("LDA", Mem(remainderLo));
                EmitAsm("SBC", Mem(divisorLo));
                EmitAsm("STA", Mem(remainderLo));
                EmitAsm("LDA", Mem(remainderHi));
                EmitAsm("SBC", Mem(divisorHi));
                EmitAsm("STA", Mem(remainderHi));
                EmitAsm("INC", Mem(quotientLo));
                EmitAsm("BNE", Rel(quotientCarryDone));
                EmitAsm("INC", Mem(quotientHi));
                _assembly.Add(Expr.Make(Tag.Label, quotientCarryDone).WithSource(lhs.Source));

                _assembly.Add(Expr.Make(Tag.Label, skipSubtract).WithSource(lhs.Source));
                EmitAsm("DEY");
                EmitAsm("BNE", Rel(loopLabel));

                if (signedArithmetic)
                {
                    // Apply the operand-sign XOR to the quotient and the original dividend sign to the remainder.
                    string quotientSignDone = NewGeneratedLabel("div_qsign_done");
                    EmitAsm("LDA", Mem(signQTemp));
                    EmitAsm("BEQ", Rel(quotientSignDone));
                    EmitNegateTempPair(quotientLo, quotientHi);
                    _assembly.Add(Expr.Make(Tag.Label, quotientSignDone).WithSource(lhs.Source));

                    string remainderSignDone = NewGeneratedLabel("div_rsign_done");
                    EmitAsm("LDA", Mem(signRTemp));
                    EmitAsm("BEQ", Rel(remainderSignDone));
                    EmitNegateTempPair(remainderLo, remainderHi);
                    _assembly.Add(Expr.Make(Tag.Label, remainderSignDone).WithSource(lhs.Source));
                }

                _assembly.Add(Expr.Make(Tag.Label, doneLabel).WithSource(lhs.Source));
                EmitAsm("LDA", Mem(wantRemainder ? remainderLo : quotientLo));
                EmitAsm("LDX", Mem(wantRemainder ? remainderHi : quotientHi));
            }
            finally
            {
                ReleaseTemps(temps);
            }
        }

        // Load words unchanged; extend byte operands with either zero or their sign byte into the requested temporary pair.
        void EmitLoadArithmeticOperandToTemps(Expr expr, FunctionContext ctx, int loTemp, int hiTemp, bool signExtend)
        {
            int operandSize = NormalizeScalarSize(DetermineExprSize(expr, ctx));
            if (operandSize >= 2)
            {
                EmitLoadValue(expr, ctx, 2);
                EmitAsm("STA", Mem(loTemp));
                EmitAsm("STX", Mem(hiTemp));
                return;
            }

            EmitLoadA(expr, ctx);
            EmitAsm("STA", Mem(loTemp));
            if (!signExtend)
            {
                EmitAsm("LDA", Imm(0));
                EmitAsm("STA", Mem(hiTemp));
                return;
            }

            string positiveLabel = NewGeneratedLabel("arith_positive");
            string doneLabel = NewGeneratedLabel("arith_done");
            EmitAsm("LDA", Mem(loTemp));
            EmitAsm("BPL", Rel(positiveLabel));
            EmitAsm("LDA", Imm(0xFF));
            EmitAsm("JMP", Abs(doneLabel));
            _assembly.Add(Expr.Make(Tag.Label, positiveLabel).WithSource(expr.Source));
            EmitAsm("LDA", Imm(0));
            _assembly.Add(Expr.Make(Tag.Label, doneLabel).WithSource(expr.Source));
            EmitAsm("STA", Mem(hiTemp));
        }

        // Negate a word in place by complementing both bytes and propagating the low-byte increment carry.
        void EmitNegateTempPair(int loTemp, int hiTemp)
        {
            EmitAsm("LDA", Mem(loTemp));
            EmitAsm("EOR", Imm(0xFF));
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(1));
            EmitAsm("STA", Mem(loTemp));
            EmitAsm("LDA", Mem(hiTemp));
            EmitAsm("EOR", Imm(0xFF));
            EmitAsm("ADC", Imm(0));
            EmitAsm("STA", Mem(hiTemp));
        }

        // Emit constant-count logical shifts; right shifts retain the source width before any requested truncation.
        void EmitShift(Expr lhs, Expr rhs, FunctionContext ctx, int size, bool leftShift)
        {
            size = NormalizeScalarSize(size);
            int operandSize = size;
            if (!leftShift)
                operandSize = NormalizeScalarSize(Math.Max(size, DetermineExprSize(lhs, ctx)));

            int shiftCount;
            if (!TryEvaluateConstant(rhs, out shiftCount))
            {
                Program.Error("error KQ0000: --target=nes phase 5 currently supports only constant shift counts.");
                return;
            }

            // Normalize the shift count modulo 16, including counts outside the native word width.
            shiftCount &= 0x0F;
            EmitLoadValue(lhs, ctx, operandSize);
            if (Program.ErrorCount > 0 || shiftCount == 0)
                return;

            if (operandSize == 1)
            {
                for (int i = 0; i < shiftCount; i++)
                    EmitAsm(leftShift ? "ASL" : "LSR");
                return;
            }

            int[] temps = AcquireTemps(2);
            try
            {
                EmitAsm("STA", Mem(temps[0]));
                EmitAsm("STX", Mem(temps[1]));

                for (int i = 0; i < shiftCount; i++)
                {
                    if (leftShift)
                    {
                        EmitAsm("ASL", Mem(temps[0]));
                        EmitAsm("ROL", Mem(temps[1]));
                    }
                    else
                    {
                        EmitAsm("LSR", Mem(temps[1]));
                        EmitAsm("ROR", Mem(temps[0]));
                    }
                }

                EmitAsm("LDA", Mem(temps[0]));
                if (size == 2)
                    EmitAsm("LDX", Mem(temps[1]));
            }
            finally
            {
                ReleaseTemps(temps);
            }
        }

        // Materialize a branch condition as exactly 0 or 1 in A.
        void EmitBooleanToA(Expr expr, FunctionContext ctx)
        {
            string trueLabel = NewGeneratedLabel("bool_true");
            string endLabel = NewGeneratedLabel("bool_end");
            EmitBranchIfTrue(expr, trueLabel, ctx);
            EmitAsm("LDA", Imm(0));
            EmitAsm("JMP", Abs(endLabel));
            _assembly.Add(Expr.Make(Tag.Label, trueLabel).WithSource(expr.Source));
            EmitAsm("LDA", Imm(1));
            _assembly.Add(Expr.Make(Tag.Label, endLabel).WithSource(expr.Source));
        }

        // Invert a supported short condition around an absolute JMP so the destination need not fit a relative offset.
        void EmitLongBranch(string mnemonic, string targetLabel, FilePosition source)
        {
            string skipLabel = NewGeneratedLabel("branch_skip");
            switch (NormalizeMnemonic(mnemonic))
            {
                case "BEQ":
                    EmitAsm("BNE", Rel(skipLabel));
                    break;
                case "BNE":
                    EmitAsm("BEQ", Rel(skipLabel));
                    break;
                case "BCC":
                    EmitAsm("BCS", Rel(skipLabel));
                    break;
                case "BCS":
                    EmitAsm("BCC", Rel(skipLabel));
                    break;
                default:
                    Program.Error("error KQ0000: unsupported long branch mnemonic for --target=nes phase 5: {0}", mnemonic);
                    return;
            }

            EmitAsm("JMP", Abs(targetLabel));
            _assembly.Add(Expr.Make(Tag.Label, skipLabel).WithSource(source));
        }

        // Lower false branches with short-circuit logic and width-aware zero tests.
        void EmitBranchIfFalse(Expr expr, string falseLabel, FunctionContext ctx)
        {
            if (expr == null || expr.Match(Tag.Empty)) return;

            int value;
            Expr lhs;
            Expr rhs;
            if (TryEvaluateConstant(expr, out value))
            {
                if ((value & 0xFFFF) == 0)
                    EmitAsm("JMP", Abs(falseLabel));
                return;
            }

            if (expr.Match(Tag.LogicalNot, out lhs))
            {
                EmitBranchIfTrue(lhs, falseLabel, ctx);
                return;
            }

            if (expr.Match(Tag.LogicalAnd, out lhs, out rhs))
            {
                EmitBranchIfFalse(lhs, falseLabel, ctx);
                EmitBranchIfFalse(rhs, falseLabel, ctx);
                return;
            }

            if (expr.Match(Tag.LogicalOr, out lhs, out rhs))
            {
                // A true left operand satisfies OR without evaluating the right operand on this emitted path.
                string skipFalse = NewGeneratedLabel("or_true_skip");
                EmitBranchIfTrue(lhs, skipFalse, ctx);
                EmitBranchIfFalse(rhs, falseLabel, ctx);
                _assembly.Add(Expr.Make(Tag.Label, skipFalse).WithSource(expr.Source));
                return;
            }

            if (IsComparisonTag(expr.Tag) && expr.MatchAnyTag<Expr, Expr>(out string cmpTag, out lhs, out rhs))
            {
                EmitComparison(cmpTag, lhs, rhs, falseLabel, branchWhenTrue: false, ctx: ctx, source: expr.Source);
                return;
            }

            if (DetermineExprSize(expr, ctx) == 2)
            {
                EmitLoadValue(expr, ctx, 2);
                // A nonzero high byte is sufficient to avoid the false branch; otherwise test A as the low byte.
                string nonZeroLabel = NewGeneratedLabel("nz16_skip_false");
                EmitAsm("CPX", Imm(0));
                EmitAsm("BNE", Rel(nonZeroLabel));
                EmitAsm("CMP", Imm(0));
                EmitLongBranch("BEQ", falseLabel, expr.Source);
                _assembly.Add(Expr.Make(Tag.Label, nonZeroLabel).WithSource(expr.Source));
                return;
            }

            EmitLoadA(expr, ctx);
            EmitLongBranch("BEQ", falseLabel, expr.Source);
        }

        // Lower true branches with short-circuit logic and a test of both bytes for word-valued conditions.
        void EmitBranchIfTrue(Expr expr, string trueLabel, FunctionContext ctx)
        {
            if (expr == null || expr.Match(Tag.Empty)) return;

            int value;
            Expr lhs;
            Expr rhs;
            if (TryEvaluateConstant(expr, out value))
            {
                if ((value & 0xFFFF) != 0)
                    EmitAsm("JMP", Abs(trueLabel));
                return;
            }

            if (expr.Match(Tag.LogicalNot, out lhs))
            {
                EmitBranchIfFalse(lhs, trueLabel, ctx);
                return;
            }

            if (expr.Match(Tag.LogicalAnd, out lhs, out rhs))
            {
                // A false left operand stops AND before the right operand is evaluated on this emitted path.
                string skipTrue = NewGeneratedLabel("and_false_skip");
                EmitBranchIfFalse(lhs, skipTrue, ctx);
                EmitBranchIfTrue(rhs, trueLabel, ctx);
                _assembly.Add(Expr.Make(Tag.Label, skipTrue).WithSource(expr.Source));
                return;
            }

            if (expr.Match(Tag.LogicalOr, out lhs, out rhs))
            {
                EmitBranchIfTrue(lhs, trueLabel, ctx);
                EmitBranchIfTrue(rhs, trueLabel, ctx);
                return;
            }

            if (IsComparisonTag(expr.Tag) && expr.MatchAnyTag<Expr, Expr>(out string cmpTag, out lhs, out rhs))
            {
                EmitComparison(cmpTag, lhs, rhs, trueLabel, branchWhenTrue: true, ctx: ctx, source: expr.Source);
                return;
            }

            if (DetermineExprSize(expr, ctx) == 2)
            {
                EmitLoadValue(expr, ctx, 2);
                EmitAsm("CPX", Imm(0));
                EmitLongBranch("BNE", trueLabel, expr.Source);
                EmitAsm("CMP", Imm(0));
                EmitLongBranch("BNE", trueLabel, expr.Source);
                return;
            }

            EmitLoadA(expr, ctx);
            EmitLongBranch("BNE", trueLabel, expr.Source);
        }

        // Invert the comparison for a false branch, then choose unsigned-byte or promoted word comparison.
        void EmitComparison(string cmpTag, Expr lhs, Expr rhs, string targetLabel, bool branchWhenTrue, FunctionContext ctx, FilePosition source)
        {
            if (!branchWhenTrue)
            {
                cmpTag = InvertComparisonTag(cmpTag);
                branchWhenTrue = true;
            }

            int size = Math.Max(DetermineExprSize(lhs, ctx), DetermineExprSize(rhs, ctx));
            size = NormalizeScalarSize(size);
            // Signed byte operands require word promotion before comparison.
            bool signedComparison = ShouldUseSignedArithmetic(lhs, rhs, ctx);
            if (size == 2 || signedComparison)
            {
                EmitComparison16(cmpTag, lhs, rhs, targetLabel, ctx, source, signedComparison);
                return;
            }

            EmitCompareOperands8(lhs, rhs, ctx);
            if (Program.ErrorCount > 0) return;

            if (cmpTag == Tag.Equal)
            {
                EmitLongBranch("BEQ", targetLabel, source);
                return;
            }
            if (cmpTag == Tag.NotEqual)
            {
                EmitLongBranch("BNE", targetLabel, source);
                return;
            }
            if (cmpTag == Tag.LessThan)
            {
                EmitLongBranch("BCC", targetLabel, source);
                return;
            }
            if (cmpTag == Tag.GreaterThanOrEqual)
            {
                EmitLongBranch("BCS", targetLabel, source);
                return;
            }
            if (cmpTag == Tag.LessThanOrEqual)
            {
                EmitLongBranch("BCC", targetLabel, source);
                EmitLongBranch("BEQ", targetLabel, source);
                return;
            }
            if (cmpTag == Tag.GreaterThan)
            {
                string skip = NewGeneratedLabel("cmp8_gt_skip");
                EmitAsm("BCC", Rel(skip));
                EmitAsm("BEQ", Rel(skip));
                EmitAsm("JMP", Abs(targetLabel));
                _assembly.Add(Expr.Make(Tag.Label, skip).WithSource(source));
                return;
            }

            Program.Error("error KQ0000: unsupported comparison tag for --target=nes phase 5: {0}", cmpTag);
        }

        // Compare high bytes first and use low bytes to resolve ties; signed order is represented by biased high bytes.
        void EmitComparison16(string cmpTag, Expr lhs, Expr rhs, string targetLabel, FunctionContext ctx, FilePosition source, bool signedComparison)
        {
            int[] rhsTemps = AcquireTemps(signedComparison ? 4 : 2);
            try
            {
                if (signedComparison)
                {
                    // Extend each operand according to its own signedness, then flip
                    // both sign bits so the existing unsigned high/low tests retain signed order.
                    EmitLoadArithmeticOperandToTemps(rhs, ctx, rhsTemps[0], rhsTemps[1], IsSignedArithmeticOperand(rhs, ctx));
                    EmitAsm("LDA", Mem(rhsTemps[1]));
                    EmitAsm("EOR", Imm(0x80));
                    EmitAsm("STA", Mem(rhsTemps[1]));
                    EmitLoadArithmeticOperandToTemps(lhs, ctx, rhsTemps[2], rhsTemps[3], IsSignedArithmeticOperand(lhs, ctx));
                    EmitAsm("LDA", Mem(rhsTemps[3]));
                    EmitAsm("EOR", Imm(0x80));
                    EmitAsm("TAX");
                    EmitAsm("LDA", Mem(rhsTemps[2]));
                }
                else
                {
                    EmitLoadValue(rhs, ctx, 2);
                    EmitAsm("STA", Mem(rhsTemps[0]));
                    EmitAsm("STX", Mem(rhsTemps[1]));
                    EmitLoadValue(lhs, ctx, 2);
                }

                if (cmpTag == Tag.Equal)
                {
                    string done = NewGeneratedLabel("cmp16_eq_done");
                    EmitAsm("CPX", Mem(rhsTemps[1]));
                    EmitAsm("BNE", Rel(done));
                    EmitAsm("CMP", Mem(rhsTemps[0]));
                    EmitLongBranch("BEQ", targetLabel, source);
                    _assembly.Add(Expr.Make(Tag.Label, done).WithSource(source));
                    return;
                }
                if (cmpTag == Tag.NotEqual)
                {
                    EmitAsm("CPX", Mem(rhsTemps[1]));
                    EmitLongBranch("BNE", targetLabel, source);
                    EmitAsm("CMP", Mem(rhsTemps[0]));
                    EmitLongBranch("BNE", targetLabel, source);
                    return;
                }
                if (cmpTag == Tag.LessThan)
                {
                    string highNotLess = NewGeneratedLabel("cmp16_lt_hi_notless");
                    EmitAsm("CPX", Mem(rhsTemps[1]));
                    EmitLongBranch("BCC", targetLabel, source);
                    EmitAsm("BNE", Rel(highNotLess));
                    EmitAsm("CMP", Mem(rhsTemps[0]));
                    EmitLongBranch("BCC", targetLabel, source);
                    _assembly.Add(Expr.Make(Tag.Label, highNotLess).WithSource(source));
                    return;
                }
                if (cmpTag == Tag.GreaterThanOrEqual)
                {
                    string highLess = NewGeneratedLabel("cmp16_ge_hi_less");
                    EmitAsm("CPX", Mem(rhsTemps[1]));
                    EmitAsm("BCC", Rel(highLess));
                    EmitLongBranch("BNE", targetLabel, source);
                    EmitAsm("CMP", Mem(rhsTemps[0]));
                    EmitLongBranch("BCS", targetLabel, source);
                    _assembly.Add(Expr.Make(Tag.Label, highLess).WithSource(source));
                    return;
                }
                if (cmpTag == Tag.LessThanOrEqual)
                {
                    string highGreater = NewGeneratedLabel("cmp16_le_hi_gt");
                    EmitAsm("CPX", Mem(rhsTemps[1]));
                    EmitLongBranch("BCC", targetLabel, source);
                    EmitAsm("BNE", Rel(highGreater));
                    EmitAsm("CMP", Mem(rhsTemps[0]));
                    EmitLongBranch("BCC", targetLabel, source);
                    EmitLongBranch("BEQ", targetLabel, source);
                    _assembly.Add(Expr.Make(Tag.Label, highGreater).WithSource(source));
                    return;
                }
                if (cmpTag == Tag.GreaterThan)
                {
                    string highLessOrEq = NewGeneratedLabel("cmp16_gt_hi_le");
                    EmitAsm("CPX", Mem(rhsTemps[1]));
                    EmitAsm("BCC", Rel(highLessOrEq));
                    EmitLongBranch("BNE", targetLabel, source);
                    EmitAsm("CMP", Mem(rhsTemps[0]));
                    EmitAsm("BCC", Rel(highLessOrEq));
                    EmitAsm("BEQ", Rel(highLessOrEq));
                    EmitAsm("JMP", Abs(targetLabel));
                    _assembly.Add(Expr.Make(Tag.Label, highLessOrEq).WithSource(source));
                    return;
                }

                Program.Error("error KQ0000: unsupported 16-bit comparison tag for --target=nes phase 5: {0}", cmpTag);
            }
            finally
            {
                ReleaseTemps(rhsTemps);
            }
        }

        // Leave byte comparison flags from CMP, staging a dynamic RHS before evaluating the left operand.
        void EmitCompareOperands8(Expr lhs, Expr rhs, FunctionContext ctx)
        {
            int rhsConst;
            if (TryEvaluateConstant(rhs, out rhsConst))
            {
                EmitLoadA(lhs, ctx);
                EmitAsm("CMP", Imm(rhsConst));
                return;
            }

            int temp = AcquireTemp();
            try
            {
                EmitLoadA(rhs, ctx);
                EmitAsm("STA", Mem(temp));
                EmitLoadA(lhs, ctx);
                EmitAsm("CMP", Mem(temp));
            }
            finally
            {
                ReleaseTemp(temp);
            }
        }

        // Resolve locals before globals and readonly data, then expose named constants as synthetic value slots.
        bool TryResolveStorage(FunctionContext ctx, string name, out StorageSlot slot)
        {
            if (ctx != null && ctx.Locals.TryGetValue(name, out slot)) return true;
            if (_globals.TryGetValue(name, out slot)) return true;
            if (_readonlyData.TryGetValue(name, out slot)) return true;
            int value;
            if (_constants.TryGetValue(name, out value))
            {
                int size = ConstantStorageSize(value);
                slot = new StorageSlot
                {
                    Name = name,
                    Type = size == 2 ? CType.UInt16 : CType.UInt8,
                    IsConstant = true,
                    ConstantValue = value,
                    Size = size,
                    Address = 0,
                };
                return true;
            }
            slot = null;
            return false;
        }

        // Resolve one pointer layer and look up the named member in the registered aggregate layout.
        bool TryResolveField(Expr baseExpr, string fieldName, FunctionContext ctx, out FieldInfo field)
        {
            field = null;
            CType baseType;
            if (!TryGetExprType(baseExpr, ctx, out baseType))
            {
                Program.Error("error KQ0000: unable to determine base type for field access '.{0}' in --target=nes phase 6.", fieldName);
                return false;
            }
            if (baseType != null && baseType.IsPointer)
                baseType = baseType.Subtype;
            if (baseType == null || !baseType.IsStructOrUnion)
            {
                Program.Error("error KQ0000: struct/union type required for field access '.{0}' in --target=nes phase 6.", fieldName);
                return false;
            }
            AggregateInfo info;
            if (!_aggregates.TryGetValue(baseType.Name, out info) || info == null)
            {
                Program.Error("error KQ0000: unknown aggregate layout for '{0}' in --target=nes phase 6.", baseType.Name);
                return false;
            }
            field = (info.Fields ?? Array.Empty<FieldInfo>()).FirstOrDefault(f => string.Equals(f.Name, fieldName, StringComparison.Ordinal));
            if (field == null)
            {
                Program.Error("error KQ0000: field '{0}' not found in {1} '{2}' for --target=nes phase 6.", fieldName,
                    info.Layout == AggregateLayout.Union ? "union" : "struct", baseType.Name);
                return false;
            }
            return true;
        }

        // Use aggregate layout alignment when available; other stored values use one- or two-byte alignment.
        int GetNaturalAlignment(CType type)
        {
            int size = GetStorageSize(type);
            if (type != null && type.IsStructOrUnion)
            {
                AggregateInfo agg;
                if (_aggregates.TryGetValue(type.Name, out agg) && agg != null)
                    return Math.Max(1, agg.Alignment);
            }
            return size >= 2 ? 2 : 1;
        }

        static bool IsAggregateType(CType type)
        {
            return type != null && type.WithoutConst().IsStructOrUnion;
        }

        // Require the same unqualified aggregate kind and name with a known nonnegative layout size.
        bool IsSameCompleteAggregateType(CType leftType, CType rightType)
        {
            if (!IsAggregateType(leftType) || !IsAggregateType(rightType)) return false;
            CType left = leftType.WithoutConst();
            CType right = rightType.WithoutConst();
            if (left.Tag != right.Tag || !string.Equals(left.Name, right.Name, StringComparison.Ordinal)) return false;
            AggregateInfo info;
            return _aggregates.TryGetValue(left.Name, out info) && info != null && info.TotalSize >= 0;
        }

        // Calculate object size recursively, resolving expression-based array extents and returning zero for unknown layouts.
        int GetStorageSize(CType type)
        {
            if (type == null) return 0;
            if (type.IsEnum) return 1;
            if (type.IsPointer) return 2;
            if (type.Tag == CTypeTag.Simple)
            {
                if (type.SimpleType == CSimpleType.UInt8 || type.SimpleType == CSimpleType.Int8) return 1;
                if (type.SimpleType == CSimpleType.UInt16 || type.SimpleType == CSimpleType.Int16) return 2;
                if (type.SimpleType == CSimpleType.Void) return 0;
            }
            if (type.Tag == CTypeTag.Array)
            {
                int elem = GetStorageSize(type.Subtype);
                return elem <= 0 ? 0 : elem * Math.Max(0, type.Dimension);
            }
            if (type.Tag == CTypeTag.ArrayWithDimensionExpression)
            {
                int dim;
                if (type.DimensionExpression != null && TryEvaluateConstant(type.DimensionExpression, out dim))
                {
                    int elem = GetStorageSize(type.Subtype);
                    return elem <= 0 ? 0 : elem * Math.Max(0, dim);
                }
                return 0;
            }
            if (type.IsStructOrUnion)
            {
                AggregateInfo agg;
                if (_aggregates.TryGetValue(type.Name, out agg) && agg != null)
                    return Math.Max(0, agg.TotalSize);
                return 0;
            }
            return 0;
        }

        // Lease a scratch byte; exhaustion reports a compile error before returning the diagnostic fallback slot.
        int AcquireTemp()
        {
            if (_freeTemps.Count == 0)
            {
                Program.Error("error KQ0000: --target=nes phase 5 ran out of temporary scratch in function {0}.", _currentFunctionName);
                return _tempPool[0];
            }
            return _freeTemps.Pop();
        }

        // Lease a group only when all requested scratch bytes are available; exhaustion leaves the pool unchanged.
        int[] AcquireTemps(int count)
        {
            if (count <= 0) return Array.Empty<int>();
            if (_freeTemps.Count < count)
            {
                Program.Error("error KQ0000: --target=nes phase 5 ran out of temporary scratch in function {2} (needed {0}, free {1}).", count, _freeTemps.Count, _currentFunctionName);
                return Enumerable.Repeat(_tempPool[0], count).ToArray();
            }
            int[] temps = new int[count];
            for (int i = 0; i < count; i++) temps[i] = _freeTemps.Pop();
            return temps;
        }

        // Return a scratch address once, avoiding duplicate free-pool entries.
        void ReleaseTemp(int temp)
        {
            if (!_freeTemps.Contains(temp))
                _freeTemps.Push(temp);
        }

        // Release grouped scratch bytes in reverse order so later leases can reuse the original ordering.
        void ReleaseTemps(int[] temps)
        {
            if (temps == null) return;
            for (int i = temps.Length - 1; i >= 0; i--)
                ReleaseTemp(temps[i]);
        }

        // Evaluate supported constant forms without emitting instructions; unresolved forms return false.
        bool TryEvaluateConstant(Expr expr, out int value)
        {
            if (expr == null)
            {
                value = 0;
                return false;
            }

            Expr left;
            Expr right;
            Expr sub;
            string name;
            if (expr.Match(Tag.Integer, out value)) return true;
            if (expr.Match(Tag.Name, out name)) return _constants.TryGetValue(name, out value);
            Expr funcExpr;
            Expr[] args;
            if (expr.MatchAny(Tag.Call, out funcExpr, out args) &&
                funcExpr != null &&
                funcExpr.Match(Tag.Name, out name) &&
                string.Equals(name, "__bankof", StringComparison.Ordinal) &&
                args != null &&
                args.Length == 1 &&
                args[0] != null &&
                args[0].Match(Tag.Name, out string bankedSymbol))
            {
                // Resolve __bankof from placement metadata and keep the byte-sized bank value.
                value = ResolveSymbolBank(bankedSymbol) & 0xFF;
                return true;
            }
            CType sizeofType;
            Expr sizeofExpr;
            if (expr.Match(Tag.Sizeof, out sizeofType))
            {
                value = GetStorageSize(sizeofType) & 0xFFFF;
                return true;
            }
            if (expr.Match(Tag.Sizeof, out sizeofExpr))
            {
                CType inferred;
                if (TryGetExprType(sizeofExpr, null, out inferred) && inferred != null)
                {
                    value = GetStorageSize(inferred) & 0xFFFF;
                    return true;
                }
                value = 0;
                return false;
            }
            CType offsetofType;
            string offsetofPath;
            if (expr.Match(Tag.Offsetof, out offsetofType, out offsetofPath))
            {
                return TryEvaluateOffsetof(offsetofType, offsetofPath, out value);
            }
            // This evaluator currently passes casts through; destination-width conversion is not applied here.
            if (expr.Match(Tag.Cast, out CType _ctype, out sub)) return TryEvaluateConstant(sub, out value);
            if (expr.Match(Tag.LogicalNot, out sub))
            {
                int subValue;
                if (TryEvaluateConstant(sub, out subValue))
                {
                    value = (subValue == 0) ? 1 : 0;
                    return true;
                }
            }
            if (expr.Match(Tag.BitwiseNot, out sub))
            {
                int subValue;
                if (TryEvaluateConstant(sub, out subValue))
                {
                    value = (~subValue) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.Add, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = (a + b) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.Subtract, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = (a - b) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.Multiply, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = (a * b) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.Divide, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b) && b != 0)
                {
                    // This constant path divides masked unsigned words, independently of runtime signed arithmetic.
                    value = ((a & 0xFFFF) / (b & 0xFFFF)) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.Modulus, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b) && b != 0)
                {
                    value = ((a & 0xFFFF) % (b & 0xFFFF)) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.BitwiseAnd, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = (a & b) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.BitwiseOr, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = (a | b) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.BitwiseXor, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = (a ^ b) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.ShiftLeft, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = (a << (b & 15)) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.ShiftRight, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    // Apply the host right shift before masking the result; the count is reduced modulo 16.
                    value = (a >> (b & 15)) & 0xFFFF;
                    return true;
                }
            }
            if (expr.Match(Tag.Equal, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = (a == b) ? 1 : 0;
                    return true;
                }
            }
            if (expr.Match(Tag.NotEqual, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = (a != b) ? 1 : 0;
                    return true;
                }
            }
            if (expr.Match(Tag.LessThan, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = ((a & 0xFFFF) < (b & 0xFFFF)) ? 1 : 0;
                    return true;
                }
            }
            if (expr.Match(Tag.LessThanOrEqual, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = ((a & 0xFFFF) <= (b & 0xFFFF)) ? 1 : 0;
                    return true;
                }
            }
            if (expr.Match(Tag.GreaterThan, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = ((a & 0xFFFF) > (b & 0xFFFF)) ? 1 : 0;
                    return true;
                }
            }
            if (expr.Match(Tag.GreaterThanOrEqual, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = ((a & 0xFFFF) >= (b & 0xFFFF)) ? 1 : 0;
                    return true;
                }
            }
            if (expr.Match(Tag.LogicalAnd, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = ((a != 0) && (b != 0)) ? 1 : 0;
                    return true;
                }
            }
            if (expr.Match(Tag.LogicalOr, out left, out right))
            {
                int a, b;
                if (TryEvaluateConstant(left, out a) && TryEvaluateConstant(right, out b))
                {
                    value = ((a != 0) || (b != 0)) ? 1 : 0;
                    return true;
                }
            }

            value = 0;
            return false;
        }

        // Walk a dotted member path through aggregate layouts and accumulate member offsets.
        bool TryEvaluateOffsetof(CType type, string path, out int value)
        {
            value = 0;
            if (type == null || string.IsNullOrEmpty(path))
                return false;

            if (type.IsPointer)
                type = type.Subtype;
            if (type == null || !type.IsStructOrUnion)
                return false;

            AggregateInfo agg;
            if (!_aggregates.TryGetValue(type.Name, out agg) || agg == null)
                return false;

            int offset = 0;
            CType currentType = type;
            string[] parts = path.Split(new[] { '.' }, StringSplitOptions.RemoveEmptyEntries);
            foreach (string part in parts)
            {
                if (currentType == null || !currentType.IsStructOrUnion)
                    return false;
                if (!_aggregates.TryGetValue(currentType.Name, out agg) || agg == null)
                    return false;

                FieldInfo field = null;
                foreach (var f in agg.Fields ?? Array.Empty<FieldInfo>())
                {
                    if (string.Equals(f.Name, part, StringComparison.Ordinal))
                    {
                        field = f;
                        break;
                    }
                }
                if (field == null)
                    return false;

                offset += field.Offset;
                currentType = field.Type;
            }

            value = offset & 0xFFFF;
            return true;
        }

        // Prefer constant value width, then inferred scalar type, then expression-shape fallbacks.
        int DetermineExprSize(Expr expr, FunctionContext ctx)
        {
            if (expr == null) return 1;

            int constValue;
            if (TryEvaluateConstant(expr, out constValue))
                return ConstantStorageSize(constValue);

            CType inferredType;
            if (TryGetExprType(expr, ctx, out inferredType) && inferredType != null)
                return NormalizeScalarSize(GetStorageSize(inferredType));

            string name;
            if (expr.Match(Tag.Name, out name))
            {
                StorageSlot slot;
                return TryResolveStorage(ctx, name, out slot) ? NormalizeScalarSize(slot.Size) : 1;
            }

            Expr lhs;
            Expr rhs;
            Expr[] args;
            Expr funcExpr;
            CType castType;
            Expr castSubexpr;
            Expr condExpr;
            Expr trueExpr;
            Expr falseExpr;
            if (expr.Match(Tag.Assign, out lhs, out rhs))
                return DetermineExprSize(lhs, ctx);
            if (expr.Match(Tag.AssignModify, out string _opName, out lhs, out rhs))
                return DetermineExprSize(lhs, ctx);
            if (expr.MatchAny(Tag.Call, out funcExpr, out args))
            {
                string funcName;
                if (funcExpr.Match(Tag.Name, out funcName)) return GetFunctionReturnSize(funcName);
                return 1;
            }
            if (expr.Match(Tag.Cast, out castType, out castSubexpr))
                return NormalizeScalarSize(TypeStorageSize(castType));
            if (expr.Match(Tag.PreIncrement, out lhs) || expr.Match(Tag.PostIncrement, out lhs) || expr.Match(Tag.PreDecrement, out lhs) || expr.Match(Tag.PostDecrement, out lhs))
                return DetermineExprSize(lhs, ctx);
            if (expr.Match(Tag.Conditional, out condExpr, out trueExpr, out falseExpr))
                return NormalizeScalarSize(Math.Max(DetermineExprSize(trueExpr, ctx), DetermineExprSize(falseExpr, ctx)));
            if (expr.Tag == Tag.Add || expr.Tag == Tag.Subtract || expr.Tag == Tag.Multiply || expr.Tag == Tag.Divide || expr.Tag == Tag.Modulus ||
                expr.Tag == Tag.BitwiseAnd || expr.Tag == Tag.BitwiseOr || expr.Tag == Tag.BitwiseXor ||
                expr.Tag == Tag.ShiftLeft || expr.Tag == Tag.ShiftRight)
            {
                if (expr.MatchAnyTag<Expr, Expr>(out string _, out lhs, out rhs))
                    return NormalizeScalarSize(Math.Max(DetermineExprSize(lhs, ctx), DetermineExprSize(rhs, ctx)));
            }
            if (expr.Tag == Tag.Sizeof || expr.Tag == Tag.Offsetof)
                return 2;
            if (expr.Tag == Tag.LogicalNot || expr.Tag == Tag.LogicalAnd || expr.Tag == Tag.LogicalOr || IsComparisonTag(expr.Tag))
                return 1;
            return 1;
        }

        static bool IsComparisonTag(string tag)
        {
            return tag == Tag.Equal || tag == Tag.NotEqual || tag == Tag.LessThan || tag == Tag.LessThanOrEqual ||
                   tag == Tag.GreaterThan || tag == Tag.GreaterThanOrEqual;
        }

        // Use registered function placement, then a configured override, with bank one as the named-function fallback.
        int GetFunctionBank(string functionName)
        {
            if (string.IsNullOrEmpty(functionName)) return 0;
            CFunctionInfo info;
            if (_functionInfos.TryGetValue(functionName, out info) && info != null)
                return info.RomBank;
            if (Program.TryGetFunctionBankOverride(functionName, out int overrideBank))
                return overrideBank;
            return 1;
        }

        // Unspecified placement sorts after functions with an explicit order.
        int GetFunctionPlacementOrder(string functionName)
        {
            CFunctionInfo info;
            if (_functionInfos.TryGetValue(functionName, out info) && info != null)
                return info.PlacementOrder;
            return int.MaxValue;
        }

        bool IsFunctionFixedBank(string functionName)
        {
            CFunctionInfo info;
            return _functionInfos.TryGetValue(functionName, out info) && info != null && info.HasFixedBank;
        }

        // Resolve function or readonly-data bank metadata, including readonly placement overrides.
        int ResolveSymbolBank(string symbolName)
        {
            if (string.IsNullOrEmpty(symbolName)) return 0;
            if (_functionInfos.TryGetValue(symbolName, out CFunctionInfo functionInfo) && functionInfo != null)
                return functionInfo.RomBank;
            if (_readonlyData.TryGetValue(symbolName, out StorageSlot readonlySlot) && readonlySlot != null)
            {
                if (Program.TryGetReadonlyDataBankOverride(symbolName, out int overrideBank))
                    return overrideBank;
                return readonlySlot.RequestedBank;
            }
            return 0;
        }

        // Emit a placement marker only when the requested bank or fixedness changes.
        void EmitPlacement(int bank, bool isFixed)
        {
            int fixedInt = isFixed ? 1 : 0;
            if (_lastPlacementBank == bank && _lastPlacementFixed == fixedInt)
                return;
            _assembly.Add(Expr.Make(Tag.PrgBank, bank, fixedInt));
            _lastPlacementBank = bank;
            _lastPlacementFixed = fixedInt;
        }

        string EnsureBankThunk(string targetName, int targetBank)
        {
            return EnsureBankThunkCore(targetName, targetBank, useFdsOverlay: false);
        }

        string EnsureFdsOverlayThunk(string targetName, int targetBank)
        {
            return EnsureBankThunkCore(targetName, targetBank, useFdsOverlay: true);
        }

        // Intern a bank/target-specific thunk and preserve first-request order for later emission.
        string EnsureBankThunkCore(string targetName, int targetBank, bool useFdsOverlay)
        {
            string prefix = useFdsOverlay ? "__kq_fds_thunk" : "__kq_thunk";
            string thunkName = string.Format("{0}_b{1}_{2}", prefix, targetBank, targetName);
            if (_bankThunkInfos.ContainsKey(thunkName))
                return thunkName;

            var info = new BankThunkInfo
            {
                Name = thunkName,
                TargetName = targetName,
                TargetBank = targetBank,
                UseFdsOverlay = useFdsOverlay,
                ReturnSize = GetFunctionReturnSize(targetName),
            };
            _bankThunkInfos[thunkName] = info;
            _orderedThunkNames.Add(thunkName);
            return thunkName;
        }

        // Aggregate repeated call sites by caller, callee and dispatch kind for the call report.
        void RecordCallEdge(string callee, int calleeBank, string kind, bool viaThunk, bool viaFarcall)
        {
            string key = string.Format("{0}|{1}|{2}|{3}|{4}", _currentFunctionName, callee, kind, viaThunk ? 1 : 0, viaFarcall ? 1 : 0);
            CallEdgeInfo edge;
            if (!_callReportMap.TryGetValue(key, out edge))
            {
                edge = new CallEdgeInfo
                {
                    Caller = _currentFunctionName,
                    Callee = callee,
                    CallerBank = _currentFunctionBank,
                    CalleeBank = calleeBank,
                    Kind = kind,
                    ViaThunk = viaThunk,
                    ViaFarcall = viaFarcall,
                    Count = 0,
                };
                _callReportMap[key] = edge;
            }
            edge.Count++;
        }

        // Include a raw JSR in call reports only when its symbolic target is a known function.
        void RecordRawAssemblyCall(string mnemonic, AsmOperand operand)
        {
            if (!string.Equals(mnemonic, "JSR", StringComparison.OrdinalIgnoreCase) ||
                operand == null || !operand.Base.HasValue)
                return;

            string target = operand.Base.Value;
            if (string.IsNullOrEmpty(target) || !_functions.ContainsKey(target))
                return;

            RecordCallEdge(
                target,
                GetFunctionBank(target),
                "raw_asm_direct",
                viaThunk: false,
                viaFarcall: false);
        }

        // Require enabled FDS overlay support and a different overlay bank before choosing an overlay thunk.
        bool ShouldUseFdsOverlayThunk(int callerBank, int calleeBank)
        {
            return Program.NesMapperProfile.HasFds &&
                Program.FdsPrgRamLayoutEnabled &&
                Program.FdsOverlayFarcallEnabled &&
                calleeBank >= 2 &&
                calleeBank != callerBank;
        }

        // Select an FDS overlay thunk, safe direct JSR, or implicit mapper-bank thunk and record that choice.
        void EmitResolvedCall(string funcName, int callerBank, FilePosition source, int expectedReturnSize)
        {
            int calleeBank = GetFunctionBank(funcName);
            if (ShouldUseFdsOverlayThunk(callerBank, calleeBank))
            {
                string fdsThunkName = EnsureFdsOverlayThunk(funcName, calleeBank);
                EmitAsm("JSR", Abs(fdsThunkName));
                RecordCallEdge(funcName, calleeBank, "fds_overlay_thunk", viaThunk: true, viaFarcall: false);
                return;
            }

            bool bankedPrg = Program.NesMapperProfile.SupportsPrgBanking || Program.NesMapperProfile.UsesDuplicatedCommonBank;
            bool directSafe = !bankedPrg || calleeBank == 0 || calleeBank == callerBank;
            if (directSafe)
            {
                EmitAsm("JSR", Abs(funcName));
                RecordCallEdge(funcName, calleeBank, "direct", viaThunk: false, viaFarcall: false);
                return;
            }

            if (Program.CheckBankCalls)
            {
                WarnOnce(ErrorCode.NesBankCrossingCall,
                    "bank_call:" + _currentFunctionName + ":" + funcName,
                    source,
                    "Cross-bank call {0}(bank {1}) -> {2}(bank {3}) uses an implicit thunk; make intent explicit with __farcall or a fixed-bank wrapper.",
                    _currentFunctionName, callerBank, funcName, calleeBank);
            }

            string thunkName = EnsureBankThunk(funcName, calleeBank);
            EmitAsm("JSR", Abs(thunkName));
            RecordCallEdge(funcName, calleeBank, "bank_thunk", viaThunk: true, viaFarcall: false);
        }

        // Translate the logical bank in A into mapper-specific register writes using runtime scratch and shadow state.
        void EmitMapperSwitchSequence()
        {
            switch (Program.NesMapperProfile.BankSwitchKind)
            {
                case NesBankSwitchKind.None:
                    return;

                // These profiles select a 16 KiB window using the zero-based physical bank.
                case NesBankSwitchKind.Uxrom:
                case NesBankSwitchKind.Vrc6:
                    EmitAsm("SEC");
                    EmitAsm("SBC", Imm(1));
                    EmitAsm("STA", Mem(0x8000));
                    return;

                // Combine the zero-based PRG selection with the saved single-screen mirroring bits.
                case NesBankSwitchKind.Axrom:
                    EmitAsm("SEC");
                    EmitAsm("SBC", Imm(1));
                    EmitAsm("AND", Imm(0x0F));
                    EmitAsm("ORA", Mem(_runtimeAxromMirrorShadowAddress));
                    EmitAsm("STA", Mem(0x8000));
                    return;

                // Reset the serial latch and write five low-to-high bits of the PRG bank number.
                case NesBankSwitchKind.Mmc1:
                    EmitAsm("SEC");
                    EmitAsm("SBC", Imm(1));
                    EmitAsm("STA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("LDA", Imm(0x80));
                    EmitAsm("STA", Mem(0x8000));
                    EmitAsm("LDX", Imm(5));
                    {
                        string loop = NewGeneratedLabel("mmc1_prg_loop");
                        _assembly.Add(Expr.Make(Tag.Label, loop));
                        EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                        EmitAsm("AND", Imm(1));
                        EmitAsm("STA", Mem(0xE000));
                        EmitAsm("LSR", Mem(_runtimeMapperTempAddress));
                        EmitAsm("DEX");
                        EmitAsm("BNE", Rel(loop));
                    }
                    return;

                // Preserve processor status and mask IRQs while updating the outer and inner SUROM bank fields.
                case NesBankSwitchKind.Mmc1Surom:
                    EmitAsm("PHP");
                    EmitAsm("SEI");
                    EmitAsm("SEC");
                    EmitAsm("SBC", Imm(1));
                    string physicalReady = NewGeneratedLabel("surom_physical_ready");
                    EmitAsm("CMP", Imm(15));
                    EmitAsm("BCC", Rel(physicalReady));
                    EmitAsm("CLC");
                    EmitAsm("ADC", Imm(1));
                    _assembly.Add(Expr.Make(Tag.Label, physicalReady));
                    EmitAsm("STA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("AND", Imm(0x0F));
                    EmitAsm("STA", Mem(_runtimeMmc1InnerShadowAddress));
                    EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("AND", Imm(0x10));
                    EmitAsm("STA", Mem(_runtimeMmc1OuterShadowAddress));
                    EmitAsm("LDA", Imm(0x80));
                    EmitAsm("STA", Mem(0x8000));
                    EmitAsm("LDA", Mem(_runtimeMmc1OuterShadowAddress));
                    EmitMmc1SerialWriteFromA(0xA000, "mmc1_surom_outer");
                    EmitAsm("LDA", Mem(_runtimeMmc1InnerShadowAddress));
                    EmitMmc1SerialWriteFromA(0xE000, "mmc1_surom_inner");
                    EmitAsm("PLP");
                    return;

                // Map a logical 16 KiB bank through consecutive 8 KiB registers six and seven.
                case NesBankSwitchKind.Mmc3:
                    EmitAsm("SEC");
                    EmitAsm("SBC", Imm(1));
                    EmitAsm("ASL");
                    EmitAsm("STA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("LDA", Imm(0x06));
                    EmitAsm("STA", Mem(0x8000));
                    EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("STA", Mem(0x8001));
                    EmitAsm("LDA", Imm(0x07));
                    EmitAsm("STA", Mem(0x8000));
                    EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("CLC");
                    EmitAsm("ADC", Imm(1));
                    EmitAsm("STA", Mem(0x8001));
                    return;

                // Select consecutive 8 KiB ROM banks with the ROM-selection bit set in both register values.
                case NesBankSwitchKind.Mmc5:
                    EmitAsm("SEC");
                    EmitAsm("SBC", Imm(1));
                    EmitAsm("ASL");
                    EmitAsm("ORA", Imm(0x80));
                    EmitAsm("STA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("STA", Mem(0x5114));
                    EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("CLC");
                    EmitAsm("ADC", Imm(1));
                    EmitAsm("STA", Mem(0x5115));
                    return;

                // Write the two consecutive 8 KiB banks comprising the logical 16 KiB window.
                case NesBankSwitchKind.Vrc7:
                    EmitAsm("SEC");
                    EmitAsm("SBC", Imm(1));
                    EmitAsm("ASL");
                    EmitAsm("STA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("STA", Mem(0x8000));
                    EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("CLC");
                    EmitAsm("ADC", Imm(1));
                    EmitAsm("STA", Mem(0x8010));
                    return;

                // Select registers eight and nine in turn to map consecutive 8 KiB PRG banks.
                case NesBankSwitchKind.Fme7:
                    EmitAsm("SEC");
                    EmitAsm("SBC", Imm(1));
                    EmitAsm("ASL");
                    EmitAsm("STA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("LDA", Imm(0x08));
                    EmitAsm("STA", Mem(0x8000));
                    EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("STA", Mem(0xA000));
                    EmitAsm("LDA", Imm(0x09));
                    EmitAsm("STA", Mem(0x8000));
                    EmitAsm("LDA", Mem(_runtimeMapperTempAddress));
                    EmitAsm("CLC");
                    EmitAsm("ADC", Imm(1));
                    EmitAsm("STA", Mem(0xA000));
                    return;

                default:
                    return;
            }
        }

        // Serialize five bits from A to one MMC1 register, using the second mapper scratch byte and X as a counter.
        void EmitMmc1SerialWriteFromA(int address, string labelKind)
        {
            EmitAsm("STA", Mem(_runtimeMapperTemp2Address));
            EmitAsm("LDX", Imm(5));
            string loop = NewGeneratedLabel(labelKind);
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("LDA", Mem(_runtimeMapperTemp2Address));
            EmitAsm("AND", Imm(1));
            EmitAsm("STA", Mem(address));
            EmitAsm("LSR", Mem(_runtimeMapperTemp2Address));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(loop));
        }

        // Combine the configured horizontal/vertical mirroring with fixed-last-bank PRG mode.
        static int GetMmc1ControlValue()
        {
            int mirroring = Program.NesCartridge.Mirroring == NesMirroringKind.Vertical ? 2 : 3;
            return 0x0C | mirroring;
        }

        // Emit intrinsic helper functions in fixed bank zero, including shared implementation aliases.
        void EmitIntrinsicHelpers()
        {
            EmitPlacement(0, true);
            _assembly.Add(Expr.Make(Tag.Comment, "KITAQFC intrinsic helper block"));
            EmitHelperNmiWait();
            EmitHelperMemoryAndBit();
            EmitHelperMath();
            EmitHelperAlias("__smul16x8_q1_7", "__smul16x8");
            EmitHelperAlias("__smac16_q1_7", "__smac16");
            EmitHelperAlias("__sdot2_q1_7", "__sdot2_q8_8");
            EmitHelperAlias("__sdot3_q1_7", "__sdot3_q8_8");
            EmitHelperGeometryAndRng();
            EmitHelperFarMemory();
            EmitHelperVramQueue();
            EmitHelperFds();
            EmitHelperPadRead("__pad_read1", 0x4016);
            EmitHelperPadRead("__pad_read2", 0x4017);
            EmitHelperPadReadMasked("__pad_read1_d1", 0x4016, 0x02);
            EmitHelperPadReadMasked("__pad_read2_d1", 0x4017, 0x02);
            EmitHelperPadSafe("__pad_read1_safe", "__pad_read1");
            EmitHelperPadSafe("__pad_read2_safe", "__pad_read2");
            EmitHelperMicAndZapper();
            EmitHelperFamilyBasicKeyboard();
            EmitHelperRobOptical();
            EmitHelperSerialMidi();
            EmitHelperOamClear();
            EmitHelperVramWrite();
            EmitHelperVramFill();
            EmitHelperNametablePut();
            EmitHelperNametablePutNt();
            EmitHelperNametableRect();
            EmitHelperNametableRectNt();
            EmitHelperAttrSet();
            EmitHelperAttrSetNt();
            EmitHelperSprites();
            EmitHelperPalette("__palette_bg_load", 0x3F00);
            EmitHelperPalette("__palette_sp_load", 0x3F10);
            EmitHelperSprite0WaitHit();
            EmitHelperSplitScrollSprite0();
        }

        // Mark a helper function boundary for assembly, placement and reachability processing.
        void EmitHelperStart(string name)
        {
            _assembly.Add(Expr.Make(Tag.Function, name));
        }

        // Tail-jump from a public alias to its shared implementation without adding a return frame.
        void EmitHelperAlias(string publicName, string targetName)
        {
            EmitHelperStart(publicName);
            EmitAsm("JMP", Abs(targetName));
        }

        // Snapshot the NMI counter and spin until it changes; the runtime NMI handler must be active.
        void EmitHelperNmiWait()
        {
            EmitHelperStart("__nmi_wait");
            EmitAsm("LDA", Mem(_runtimeNmiCounterAddress));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            string loop = NewGeneratedLabel("nmi_wait_loop");
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("LDA", Mem(_runtimeNmiCounterAddress));
            EmitAsm("CMP", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("BEQ", Rel(loop));
            EmitAsm("RTS");
        }

        // Latch controllers, read eight serial data bits and assemble the result from the least significant bit upward.
        void EmitHelperPadRead(string name, int port)
        {
            EmitHelperStart(name);
            EmitAsm("LDA", Imm(1));
            EmitAsm("STA", Mem(0x4016));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(0x4016));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Imm(1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDX", Imm(8));
            string loop = NewGeneratedLabel("pad_loop");
            string skip = NewGeneratedLabel("pad_skip");
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("LDA", Mem(port));
            EmitAsm("AND", Imm(1));
            EmitAsm("BEQ", Rel(skip));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("ORA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            _assembly.Add(Expr.Make(Tag.Label, skip));
            EmitAsm("ASL", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(loop));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");
        }

        // Repeat paired controller reads until both samples agree; the retry loop has no fixed attempt limit.
        void EmitHelperPadSafe(string name, string readHelper)
        {
            EmitHelperStart(name);
            string loop = NewGeneratedLabel("pad_safe_loop");
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("JSR", Abs(readHelper));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("JSR", Abs(readHelper));
            EmitAsm("CMP", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("BNE", Rel(loop));
            EmitAsm("RTS");
        }

        // Build the same eight-button result using the selected input line mask instead of data bit zero.
        void EmitHelperPadReadMasked(string name, int port, int mask)
        {
            EmitHelperStart(name);
            EmitAsm("LDA", Imm(1));
            EmitAsm("STA", Mem(0x4016));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(0x4016));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Imm(1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDX", Imm(8));
            string loop = NewGeneratedLabel("pad_mask_loop");
            string skip = NewGeneratedLabel("pad_mask_skip");
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("LDA", Mem(port));
            EmitAsm("AND", Imm(mask));
            EmitAsm("BEQ", Rel(skip));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("ORA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            _assembly.Add(Expr.Make(Tag.Label, skip));
            EmitAsm("ASL", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(loop));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");
        }

        // Read an input mask and return exactly zero or one, optionally interpreting a cleared bit as active.
        void EmitNormalizeBoolFromMask(int port, int mask, bool invert)
        {
            string zero = NewGeneratedLabel("bool_zero");
            string done = NewGeneratedLabel("bool_done");
            EmitAsm("LDA", Mem(port));
            EmitAsm("AND", Imm(mask));
            if (invert) EmitAsm("BNE", Rel(zero)); else EmitAsm("BEQ", Rel(zero));
            EmitAsm("LDA", Imm(1));
            EmitAsm("JMP", Abs(done));
            _assembly.Add(Expr.Make(Tag.Label, zero));
            EmitAsm("LDA", Imm(0));
            _assembly.Add(Expr.Make(Tag.Label, done));
            EmitAsm("RTS");
        }

        // Expose microphone/trigger signals directly and invert the active-low light-sensor bit.
        void EmitHelperMicAndZapper()
        {
            EmitHelperStart("__mic_read2p");
            EmitNormalizeBoolFromMask(0x4016, 0x04, false);
            EmitHelperStart("__zapper_trigger1");
            EmitNormalizeBoolFromMask(0x4016, 0x10, false);
            EmitHelperStart("__zapper_trigger2");
            EmitNormalizeBoolFromMask(0x4017, 0x10, false);
            EmitHelperStart("__zapper_light1");
            EmitNormalizeBoolFromMask(0x4016, 0x08, true);
            EmitHelperStart("__zapper_light2");
            EmitNormalizeBoolFromMask(0x4017, 0x08, true);
        }

        // Emit six NOPs for a fixed short keyboard settling interval without using loop registers.
        void EmitKeyboardDelayShort()
        {
            for (int i = 0; i < 6; i++) EmitAsm("NOP");
        }

        void EmitKeyboardDelayLong()
        {
            // Keep X/Y intact because keyboard scan uses X for row count and Y for dst offset.
            for (int i = 0; i < 25; i++) EmitAsm("NOP");
        }

        // Emit keyboard scan, row/column read and detection helpers using the controller-port matrix signals.
        void EmitHelperFamilyBasicKeyboard()
        {
            // Reset the matrix and store two masked samples for each of nine rows into the caller buffer.
            EmitHelperStart("__fkb_scan");
            EmitAsm("LDA", Imm(0x05));
            EmitAsm("STA", Mem(0x4016));
            EmitKeyboardDelayShort();
            EmitAsm("LDY", Imm(0));
            EmitAsm("LDX", Imm(9));
            string rowLoop = NewGeneratedLabel("fkb_scan_row");
            _assembly.Add(Expr.Make(Tag.Label, rowLoop));
            EmitAsm("LDA", Imm(0x04));
            EmitAsm("STA", Mem(0x4016));
            EmitKeyboardDelayLong();
            EmitAsm("LDA", Mem(0x4017));
            EmitAsm("AND", Imm(0x1E));
            EmitAsm("STA", IndY(CallArgBase));
            EmitAsm("INY");
            EmitAsm("LDA", Imm(0x06));
            EmitAsm("STA", Mem(0x4016));
            EmitKeyboardDelayLong();
            EmitAsm("LDA", Mem(0x4017));
            EmitAsm("AND", Imm(0x1E));
            EmitAsm("STA", IndY(CallArgBase));
            EmitAsm("INY");
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(rowLoop));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(0x4016));
            EmitAsm("RTS");

            // Advance to the requested row, select its low/high sample and return raw bits 1 through 4.
            EmitHelperStart("__fkb_read_row_col");
            EmitAsm("LDA", Imm(0x05));
            EmitAsm("STA", Mem(0x4016));
            EmitKeyboardDelayShort();
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(1));
            EmitAsm("TAX");
            string adv = NewGeneratedLabel("fkb_adv_row");
            _assembly.Add(Expr.Make(Tag.Label, adv));
            EmitAsm("LDA", Imm(0x04));
            EmitAsm("STA", Mem(0x4016));
            EmitKeyboardDelayLong();
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(adv));
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("AND", Imm(1));
            string readNow = NewGeneratedLabel("fkb_read_now");
            EmitAsm("BEQ", Rel(readNow));
            EmitAsm("LDA", Imm(0x06));
            EmitAsm("STA", Mem(0x4016));
            EmitKeyboardDelayLong();
            _assembly.Add(Expr.Make(Tag.Label, readNow));
            EmitAsm("LDA", Mem(0x4017));
            EmitAsm("AND", Imm(0x1E));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(0x4016));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");

            // Check for the expected selected-row and disabled-matrix bit patterns.
            EmitHelperStart("__fkb_detect");
            EmitAsm("LDA", Imm(0x09));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("JSR", Abs("__fkb_read_row_col"));
            EmitAsm("CMP", Imm(0x1E));
            string no = NewGeneratedLabel("fkb_no");
            EmitAsm("BNE", Rel(no));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(0x4016));
            EmitAsm("LDA", Mem(0x4017));
            EmitAsm("AND", Imm(0x1E));
            EmitAsm("BNE", Rel(no));
            EmitAsm("LDA", Imm(1));
            EmitAsm("RTS");
            _assembly.Add(Expr.Make(Tag.Label, no));
            EmitAsm("LDA", Imm(0));
            EmitAsm("RTS");
        }

        // Emit rendering-mask flashes, frame-counted pulses and a most-significant-bit-first optical byte sequence.
        void EmitHelperRobOptical()
        {
            // Select the rendering mask from the argument and wait for one NMI-counter change.
            EmitHelperStart("__rob_flash");
            EmitAsm("LDA", Mem(CallArgBase));
            string black = NewGeneratedLabel("rob_black");
            EmitAsm("BEQ", Rel(black));
            EmitAsm("LDA", Imm(0x1E));
            EmitAsm("STA", Mem(_runtimePpuMaskShadowAddress));
            EmitAsm("STA", Mem(0x2001));
            EmitAsm("JSR", Abs("__nmi_wait"));
            EmitAsm("RTS");
            _assembly.Add(Expr.Make(Tag.Label, black));
            EmitAsm("LDA", Imm(0x00));
            EmitAsm("STA", Mem(_runtimePpuMaskShadowAddress));
            EmitAsm("STA", Mem(0x2001));
            EmitAsm("JSR", Abs("__nmi_wait"));
            EmitAsm("RTS");

            // Emit the requested number of enabled frames followed by the requested disabled frames.
            EmitHelperStart("__rob_pulse");
            EmitAsm("LDX", Mem(CallArgBase));
            string onLoop = NewGeneratedLabel("rob_on_loop");
            string offStart = NewGeneratedLabel("rob_off_start");
            _assembly.Add(Expr.Make(Tag.Label, onLoop));
            EmitAsm("CPX", Imm(0));
            EmitAsm("BEQ", Rel(offStart));
            EmitAsm("LDA", Imm(1));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__rob_flash"));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(onLoop));
            _assembly.Add(Expr.Make(Tag.Label, offStart));
            EmitAsm("LDX", Mem(CallArgBase + 1));
            string offLoop = NewGeneratedLabel("rob_off_loop");
            string pulseDone = NewGeneratedLabel("rob_pulse_done");
            _assembly.Add(Expr.Make(Tag.Label, offLoop));
            EmitAsm("CPX", Imm(0));
            EmitAsm("BEQ", Rel(pulseDone));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__rob_flash"));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(offLoop));
            _assembly.Add(Expr.Make(Tag.Label, pulseDone));
            EmitAsm("RTS");

            // Choose a four/two or two/four frame pulse pattern for each current high bit.
            EmitHelperStart("__rob_send_byte");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDX", Imm(8));
            string bitLoop = NewGeneratedLabel("rob_bit_loop");
            string bitZero = NewGeneratedLabel("rob_bit_zero");
            string bitNext = NewGeneratedLabel("rob_bit_next");
            _assembly.Add(Expr.Make(Tag.Label, bitLoop));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("AND", Imm(0x80));
            EmitAsm("BEQ", Rel(bitZero));
            EmitAsm("LDA", Imm(4));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", Imm(2));
            EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("JSR", Abs("__rob_pulse"));
            EmitAsm("JMP", Abs(bitNext));
            _assembly.Add(Expr.Make(Tag.Label, bitZero));
            EmitAsm("LDA", Imm(2));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", Imm(4));
            EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("JSR", Abs("__rob_pulse"));
            _assembly.Add(Expr.Make(Tag.Label, bitNext));
            EmitAsm("ASL", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(bitLoop));
            EmitAsm("RTS");
        }

        void EmitMidiBitDelay()
        {
            // Emit 29 NOPs without changing X/Y; serial instruction overhead is additional.
            for (int i = 0; i < 29; i++) EmitAsm("NOP");
        }

        // Emit controller-port serial bit I/O and byte/message helpers using a shared delay sequence.
        void EmitHelperSerialMidi()
        {
            // Write the low argument bit to the controller strobe port.
            EmitHelperStart("__serial_tx_bit");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("AND", Imm(1));
            EmitAsm("STA", Mem(0x4016));
            EmitAsm("RTS");

            // Normalize input data bit four to a zero/one return value.
            EmitHelperStart("__serial_rx_bit");
            EmitAsm("LDA", Mem(0x4017));
            EmitAsm("AND", Imm(0x10));
            string rx0 = NewGeneratedLabel("serial_rx0");
            string rxd = NewGeneratedLabel("serial_rxdone");
            EmitAsm("BEQ", Rel(rx0));
            EmitAsm("LDA", Imm(1));
            EmitAsm("JMP", Abs(rxd));
            _assembly.Add(Expr.Make(Tag.Label, rx0));
            EmitAsm("LDA", Imm(0));
            _assembly.Add(Expr.Make(Tag.Label, rxd));
            EmitAsm("RTS");

            // Send a low start bit, eight least-significant-first data bits and a high stop bit.
            EmitHelperStart("__kq_midi_out_a");
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__serial_tx_bit"));
            EmitMidiBitDelay();
            EmitAsm("LDX", Imm(8));
            string outLoop = NewGeneratedLabel("midi_out_loop");
            _assembly.Add(Expr.Make(Tag.Label, outLoop));
            EmitAsm("LSR", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Imm(0));
            EmitAsm("ADC", Imm(0));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__serial_tx_bit"));
            EmitMidiBitDelay();
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(outLoop));
            EmitAsm("LDA", Imm(1));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__serial_tx_bit"));
            EmitMidiBitDelay();
            EmitAsm("RTS");

            EmitHelperStart("__midi_out_byte");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("RTS");

            // Wait for a low start signal, sample eight bits into a shift register and return the assembled byte.
            EmitHelperStart("__midi_in_byte");
            string waitStart = NewGeneratedLabel("midi_wait_start");
            _assembly.Add(Expr.Make(Tag.Label, waitStart));
            EmitAsm("JSR", Abs("__serial_rx_bit"));
            EmitAsm("BNE", Rel(waitStart));
            EmitMidiBitDelay();
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDX", Imm(8));
            string inLoop = NewGeneratedLabel("midi_in_loop");
            string inZero = NewGeneratedLabel("midi_in_zero");
            string inNext = NewGeneratedLabel("midi_in_next");
            _assembly.Add(Expr.Make(Tag.Label, inLoop));
            EmitMidiBitDelay();
            EmitAsm("JSR", Abs("__serial_rx_bit"));
            EmitAsm("BEQ", Rel(inZero));
            EmitAsm("SEC");
            EmitAsm("ROR", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("JMP", Abs(inNext));
            _assembly.Add(Expr.Make(Tag.Label, inZero));
            EmitAsm("CLC");
            EmitAsm("ROR", Mem(_runtimeIntrinsicTmp0Address));
            _assembly.Add(Expr.Make(Tag.Label, inNext));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(inLoop));
            EmitMidiBitDelay();
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");

            // Compose the channelized note-on status, then send note and velocity arguments.
            EmitHelperStart("__midi_note_on");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("AND", Imm(0x0F));
            EmitAsm("ORA", Imm(0x90));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("RTS");

            // Compose the note-off status and transmit its two supplied data bytes.
            EmitHelperStart("__midi_note_off");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("AND", Imm(0x0F));
            EmitAsm("ORA", Imm(0x80));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("RTS");

            // Compose a control-change status and transmit controller number and value.
            EmitHelperStart("__midi_control_change");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("AND", Imm(0x0F));
            EmitAsm("ORA", Imm(0xB0));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("RTS");

            // Compose a program-change status followed by its single data byte.
            EmitHelperStart("__midi_program_change");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("AND", Imm(0x0F));
            EmitAsm("ORA", Imm(0xC0));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("JSR", Abs("__kq_midi_out_a"));
            EmitAsm("RTS");
        }

        // Hide all 64 shadow-OAM sprites by writing an off-screen Y value every four bytes at page two.
        void EmitHelperOamClear()
        {
            EmitHelperStart("__oam_clear");
            EmitAsm("LDX", Imm(0));
            EmitAsm("LDA", Imm(0xF0));
            string loop = NewGeneratedLabel("oam_clear_loop");
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("STA", new AsmOperand(0x0200, AddressMode.AbsoluteX));
            EmitAsm("INX"); EmitAsm("INX"); EmitAsm("INX"); EmitAsm("INX");
            EmitAsm("BNE", Rel(loop));
            EmitAsm("RTS");
        }

        // Reset the PPU address latch, then write the high and low bytes of the first pointer argument.
        void EmitPpuAddrFromCallArg()
        {
            EmitAsm("LDA", Mem(0x2002));
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("STA", Mem(0x2006));
        }

        // Stream the byte-counted source into PPUDATA; a zero count writes no bytes.
        void EmitHelperVramWrite()
        {
            EmitHelperStart("__vram_write");
            EmitPpuAddrFromCallArg();
            EmitAsm("LDY", Imm(0));
            string loop = NewGeneratedLabel("vram_write_loop");
            string done = NewGeneratedLabel("vram_write_done");
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("CPY", Mem(CallArgBase + 4));
            EmitAsm("BEQ", Rel(done));
            EmitAsm("LDA", IndY(CallArgBase + 2));
            EmitAsm("STA", Mem(0x2007));
            EmitAsm("INY");
            EmitAsm("BNE", Rel(loop));
            _assembly.Add(Expr.Make(Tag.Label, done));
            EmitAsm("RTS");
        }

        // Write the fill value to PPUDATA for a byte-sized count, with zero treated as an empty fill.
        void EmitHelperVramFill()
        {
            EmitHelperStart("__vram_fill");
            EmitPpuAddrFromCallArg();
            EmitAsm("LDX", Mem(CallArgBase + 3));
            string loop = NewGeneratedLabel("vram_fill_loop");
            string done = NewGeneratedLabel("vram_fill_done");
            EmitAsm("BEQ", Rel(done));
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("STA", Mem(0x2007));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(loop));
            _assembly.Add(Expr.Make(Tag.Label, done));
            EmitAsm("RTS");
        }

        // Calculate the first nametable cell address and write one tile through the PPU data port.
        void EmitHelperNametablePut()
        {
            EmitHelperStart("__nametable_put");
            // args: x, y, tile. PPU address = $2000 + y * 32 + x.
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            for (int i = 0; i < 5; i++) { EmitAsm("ASL", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("ROL", Mem(_runtimeIntrinsicTmp1Address)); }
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(CallArgBase));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("ADC", Imm(0x20));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDA", Mem(0x2002));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("STA", Mem(0x2007));
            EmitAsm("RTS");
        }


        // Select one of four nametables, calculate its cell address and write the supplied tile.
        void EmitHelperNametablePutNt()
        {
            EmitHelperStart("__nametable_put_nt");
            // args: nt,x,y,tile. PPU address = $2000 + nt*$400 + y*32 + x.
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("AND", Imm(3));
            EmitAsm("ASL");
            EmitAsm("ASL");
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(0x20));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            for (int i = 0; i < 5; i++) { EmitAsm("ASL", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("ROL", Mem(_runtimeIntrinsicTmp1Address)); }
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(CallArgBase + 1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("ADC", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDA", Mem(0x2002));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Mem(CallArgBase + 3));
            EmitAsm("STA", Mem(0x2007));
            EmitAsm("RTS");
        }


        // Set the supplied palette base and stream sixteen bytes from the caller pointer.
        void EmitHelperPalette(string name, int ppuAddress)
        {
            EmitHelperStart(name);
            EmitAsm("LDA", Mem(0x2002));
            EmitAsm("LDA", Imm((ppuAddress >> 8) & 0xFF));
            EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Imm(ppuAddress & 0xFF));
            EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDY", Imm(0));
            string loop = NewGeneratedLabel("pal_loop");
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("LDA", IndY(CallArgBase));
            EmitAsm("STA", Mem(0x2007));
            EmitAsm("INY");
            EmitAsm("CPY", Imm(16));
            EmitAsm("BNE", Rel(loop));
            EmitAsm("RTS");
        }

        // Wait for the current sprite-zero flag to clear, then wait for the next hit; no timeout is emitted.
        void EmitHelperSprite0WaitHit()
        {
            EmitHelperStart("__sprite0_wait_hit");
            string clear = NewGeneratedLabel("sprite0_clear");
            string hit = NewGeneratedLabel("sprite0_hit");
            _assembly.Add(Expr.Make(Tag.Label, clear));
            EmitAsm("LDA", Mem(0x2002));
            EmitAsm("AND", Imm(0x40));
            EmitAsm("BNE", Rel(clear));
            _assembly.Add(Expr.Make(Tag.Label, hit));
            EmitAsm("LDA", Mem(0x2002));
            EmitAsm("AND", Imm(0x40));
            EmitAsm("BEQ", Rel(hit));
            EmitAsm("RTS");
        }

        // Wait for sprite zero, reset the shared PPU latch and write the two scroll arguments.
        void EmitHelperSplitScrollSprite0()
        {
            EmitHelperStart("__split_scroll_sprite0");
            EmitAsm("JSR", Abs("__sprite0_wait_hit"));
            EmitAsm("LDA", Mem(0x2002));
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("STA", Mem(0x2005));
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("STA", Mem(0x2005));
            EmitAsm("RTS");
        }

        // Emit forward byte-copy/fill loops and their byte-count wrappers, followed by bit-operation helpers.
        void EmitHelperMemoryAndBit()
        {
            // Promote the byte count to a word by clearing its high byte before entering memcpy.
            EmitHelperStart("__memcpy_small");
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(CallArgBase + 5));
            EmitAsm("JMP", Abs("__memcpy"));

            // Advance both argument pointers and decrement the word count in place; overlapping copies are not reversed.
            EmitHelperStart("__memcpy");
            string mloop = NewGeneratedLabel("memcpy_loop");
            string mdone = NewGeneratedLabel("memcpy_done");
            EmitAsm("LDA", Mem(CallArgBase + 4));
            EmitAsm("ORA", Mem(CallArgBase + 5));
            EmitAsm("BEQ", Rel(mdone));
            _assembly.Add(Expr.Make(Tag.Label, mloop));
            EmitAsm("LDY", Imm(0));
            EmitAsm("LDA", IndY(CallArgBase + 2));
            EmitAsm("STA", IndY(CallArgBase));
            EmitAsm("INC", Mem(CallArgBase + 2));
            string ms1 = NewGeneratedLabel("memcpy_src_page_ok");
            EmitAsm("BNE", Rel(ms1));
            EmitAsm("INC", Mem(CallArgBase + 3));
            _assembly.Add(Expr.Make(Tag.Label, ms1));
            EmitAsm("INC", Mem(CallArgBase));
            string md1 = NewGeneratedLabel("memcpy_dst_page_ok");
            EmitAsm("BNE", Rel(md1));
            EmitAsm("INC", Mem(CallArgBase + 1));
            _assembly.Add(Expr.Make(Tag.Label, md1));
            EmitAsm("LDA", Mem(CallArgBase + 4));
            string mlononzero = NewGeneratedLabel("memcpy_lenlo_nonzero");
            EmitAsm("BNE", Rel(mlononzero));
            EmitAsm("DEC", Mem(CallArgBase + 5));
            _assembly.Add(Expr.Make(Tag.Label, mlononzero));
            EmitAsm("DEC", Mem(CallArgBase + 4));
            EmitAsm("LDA", Mem(CallArgBase + 4));
            EmitAsm("ORA", Mem(CallArgBase + 5));
            EmitAsm("BNE", Rel(mloop));
            _assembly.Add(Expr.Make(Tag.Label, mdone));
            EmitAsm("RTS");

            // Clear the high count byte for the small-fill wrapper.
            EmitHelperStart("__memset_small");
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(CallArgBase + 4));
            EmitAsm("JMP", Abs("__memset"));

            // Advance the destination pointer while consuming the word count in the shared argument area.
            EmitHelperStart("__memset");
            string sloop = NewGeneratedLabel("memset_loop");
            string sdone = NewGeneratedLabel("memset_done");
            EmitAsm("LDA", Mem(CallArgBase + 3));
            EmitAsm("ORA", Mem(CallArgBase + 4));
            EmitAsm("BEQ", Rel(sdone));
            _assembly.Add(Expr.Make(Tag.Label, sloop));
            EmitAsm("LDY", Imm(0));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("STA", IndY(CallArgBase));
            EmitAsm("INC", Mem(CallArgBase));
            string sd1 = NewGeneratedLabel("memset_dst_page_ok");
            EmitAsm("BNE", Rel(sd1));
            EmitAsm("INC", Mem(CallArgBase + 1));
            _assembly.Add(Expr.Make(Tag.Label, sd1));
            EmitAsm("LDA", Mem(CallArgBase + 3));
            string slononzero = NewGeneratedLabel("memset_lenlo_nonzero");
            EmitAsm("BNE", Rel(slononzero));
            EmitAsm("DEC", Mem(CallArgBase + 4));
            _assembly.Add(Expr.Make(Tag.Label, slononzero));
            EmitAsm("DEC", Mem(CallArgBase + 3));
            EmitAsm("LDA", Mem(CallArgBase + 3));
            EmitAsm("ORA", Mem(CallArgBase + 4));
            EmitAsm("BNE", Rel(sloop));
            _assembly.Add(Expr.Make(Tag.Label, sdone));
            EmitAsm("RTS");

            EmitBitHelper("__bit_test", "test");
            EmitBitHelper("__bit_set", "set");
            EmitBitHelper("__bit_clear", "clear");
            EmitBitHelper("__bit_toggle", "toggle");
        }

        // Build a low-three-bit mask and an indexed byte offset, then test or update the selected bit.
        void EmitBitHelper(string name, string op)
        {
            EmitHelperStart(name);
            EmitAsm("LDA", Imm(1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("AND", Imm(7));
            EmitAsm("TAX");
            string maskDone = NewGeneratedLabel("bit_mask_done");
            string maskLoop = NewGeneratedLabel("bit_mask_loop");
            EmitAsm("BEQ", Rel(maskDone));
            _assembly.Add(Expr.Make(Tag.Label, maskLoop));
            EmitAsm("ASL", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(maskLoop));
            _assembly.Add(Expr.Make(Tag.Label, maskDone));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDA", Mem(CallArgBase + 3));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            for (int i = 0; i < 3; i++) { EmitAsm("LSR", Mem(_runtimeIntrinsicTmp2Address)); EmitAsm("ROR", Mem(_runtimeIntrinsicTmp1Address)); }
            // Fold the high displacement into the pointer; indirect-Y adds the low byte and its carry.
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("LDY", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDA", IndY(CallArgBase));
            switch (op)
            {
                case "test":
                    EmitAsm("AND", Mem(_runtimeIntrinsicTmp0Address));
                    EmitAsm("RTS");
                    break;
                case "set":
                    EmitAsm("ORA", Mem(_runtimeIntrinsicTmp0Address));
                    EmitAsm("STA", IndY(CallArgBase));
                    EmitAsm("RTS");
                    break;
                case "clear":
                    EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
                    EmitAsm("EOR", Imm(0xFF));
                    EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
                    EmitAsm("LDA", IndY(CallArgBase));
                    EmitAsm("AND", Mem(_runtimeIntrinsicTmp2Address));
                    EmitAsm("STA", IndY(CallArgBase));
                    EmitAsm("RTS");
                    break;
                case "toggle":
                    EmitAsm("EOR", Mem(_runtimeIntrinsicTmp0Address));
                    EmitAsm("STA", IndY(CallArgBase));
                    EmitAsm("RTS");
                    break;
            }
        }

        // Emit word-by-byte multiplication and compose MAC/dot-product helpers from those integer products.
        void EmitHelperMath()
        {
            // Consume eight multiplier bits and accumulate the low product word in shared temporaries, returning A/X.
            EmitHelperStart("__mul16x8");
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDX", Imm(8));
            string loop = NewGeneratedLabel("mul16x8_loop");
            string skip = NewGeneratedLabel("mul16x8_skip_add");
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("LSR", Mem(CallArgBase + 2));
            EmitAsm("BCC", Rel(skip));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(CallArgBase));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("ADC", Mem(CallArgBase + 1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            _assembly.Add(Expr.Make(Tag.Label, skip));
            EmitAsm("ASL", Mem(CallArgBase));
            EmitAsm("ROL", Mem(CallArgBase + 1));
            EmitAsm("DEX");
            EmitAsm("BNE", Rel(loop));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDX", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("RTS");

            // Zero-extend the byte multiplicand, reuse word multiplication and return its high byte.
            EmitHelperStart("__mul8x8_hi");
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("STA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs("__mul16x8"));
            EmitAsm("TXA");
            EmitAsm("RTS");

            // Convert signed operands to magnitudes, multiply, then restore the product sign modulo 65536.
            EmitHelperStart("__smul16x8");
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            string aPos = NewGeneratedLabel("smul_a_positive");
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("AND", Imm(0x80));
            EmitAsm("BEQ", Rel(aPos));
            EmitAsm("LDA", Mem(CallArgBase)); EmitAsm("EOR", Imm(0xFF)); EmitAsm("CLC"); EmitAsm("ADC", Imm(1)); EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("EOR", Imm(0xFF)); EmitAsm("ADC", Imm(0)); EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp2Address)); EmitAsm("EOR", Imm(1)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            _assembly.Add(Expr.Make(Tag.Label, aPos));
            string bPos = NewGeneratedLabel("smul_b_positive");
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("AND", Imm(0x80));
            EmitAsm("BEQ", Rel(bPos));
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("EOR", Imm(0xFF)); EmitAsm("CLC"); EmitAsm("ADC", Imm(1)); EmitAsm("STA", Mem(CallArgBase + 2));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp2Address)); EmitAsm("EOR", Imm(1)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            _assembly.Add(Expr.Make(Tag.Label, bPos));
            EmitAsm("JSR", Abs("__mul16x8"));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("STX", Mem(_runtimeIntrinsicTmp1Address));
            string sdone = NewGeneratedLabel("smul_done");
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("BEQ", Rel(sdone));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("EOR", Imm(0xFF)); EmitAsm("CLC"); EmitAsm("ADC", Imm(1)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address)); EmitAsm("EOR", Imm(0xFF)); EmitAsm("ADC", Imm(0)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            _assembly.Add(Expr.Make(Tag.Label, sdone));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDX", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("RTS");

            EmitMacHelper("__mac16", "__mul16x8");
            EmitMacHelper("__smac16", "__smul16x8");
            EmitDot2Helper("__dot2_q8_8", "__mul16x8");
            EmitDot3Helper("__dot3_q8_8", "__mul16x8");
            EmitDot2Helper("__sdot2_q8_8", "__smul16x8");
            EmitDot3Helper("__sdot3_q8_8", "__smul16x8");
        }

        // Save the accumulator outside the multiply argument bytes, then add it to the selected product.
        void EmitMacHelper(string name, string mulHelper)
        {
            EmitHelperStart(name);
            EmitAsm("LDA", Mem(CallArgBase)); EmitAsm("STA", Mem(CallArgBase + 5));
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("STA", Mem(CallArgBase + 6));
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", Mem(CallArgBase + 3)); EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("LDA", Mem(CallArgBase + 4)); EmitAsm("STA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs(mulHelper));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(CallArgBase + 5));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("TXA");
            EmitAsm("ADC", Mem(CallArgBase + 6));
            EmitAsm("TAX");
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");
        }


        // Stage the second operand pair and first product in spare argument bytes before accumulating the second product.
        void EmitDot2Helper(string name, string mulHelper)
        {
            EmitHelperStart(name);
            // args: a0:u16, a1:u16, b0:u8, b1:u8. return a0*b0 + a1*b1.
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("STA", Mem(CallArgBase + 8));
            EmitAsm("LDA", Mem(CallArgBase + 3)); EmitAsm("STA", Mem(CallArgBase + 9));
            EmitAsm("LDA", Mem(CallArgBase + 5)); EmitAsm("STA", Mem(CallArgBase + 10));
            EmitAsm("LDA", Mem(CallArgBase + 4)); EmitAsm("STA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs(mulHelper));
            EmitAsm("STA", Mem(CallArgBase + 12));
            EmitAsm("STX", Mem(CallArgBase + 13));
            EmitAsm("LDA", Mem(CallArgBase + 8)); EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", Mem(CallArgBase + 9)); EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("LDA", Mem(CallArgBase + 10)); EmitAsm("STA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs(mulHelper));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(CallArgBase + 12));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("TXA");
            EmitAsm("ADC", Mem(CallArgBase + 13));
            EmitAsm("TAX");
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");
        }

        // Preserve all later operands before multiply calls overwrite their input slots; accumulate three low-word products.
        void EmitDot3Helper(string name, string mulHelper)
        {
            EmitHelperStart(name);
            // args: a0:u16, a1:u16, a2:u16, b0:u8, b1:u8, b2:u8.
            // Save operands that will be overwritten by the multiply helper.
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("STA", Mem(CallArgBase + 9));  // a1 lo
            EmitAsm("LDA", Mem(CallArgBase + 3)); EmitAsm("STA", Mem(CallArgBase + 10)); // a1 hi
            EmitAsm("LDA", Mem(CallArgBase + 4)); EmitAsm("STA", Mem(CallArgBase + 11)); // a2 lo
            EmitAsm("LDA", Mem(CallArgBase + 5)); EmitAsm("STA", Mem(CallArgBase + 12)); // a2 hi
            EmitAsm("LDA", Mem(CallArgBase + 7)); EmitAsm("STA", Mem(CallArgBase + 13)); // b1
            EmitAsm("LDA", Mem(CallArgBase + 8)); EmitAsm("STA", Mem(CallArgBase + 14)); // b2

            EmitAsm("LDA", Mem(CallArgBase + 6)); EmitAsm("STA", Mem(CallArgBase + 2));  // b0
            EmitAsm("JSR", Abs(mulHelper));
            EmitAsm("STA", Mem(CallArgBase + 6));  // sum lo
            EmitAsm("STX", Mem(CallArgBase + 7));  // sum hi

            EmitAsm("LDA", Mem(CallArgBase + 9)); EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", Mem(CallArgBase + 10)); EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("LDA", Mem(CallArgBase + 13)); EmitAsm("STA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs(mulHelper));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(CallArgBase + 6));
            EmitAsm("STA", Mem(CallArgBase + 6));
            EmitAsm("TXA");
            EmitAsm("ADC", Mem(CallArgBase + 7));
            EmitAsm("STA", Mem(CallArgBase + 7));

            EmitAsm("LDA", Mem(CallArgBase + 11)); EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", Mem(CallArgBase + 12)); EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("LDA", Mem(CallArgBase + 14)); EmitAsm("STA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs(mulHelper));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(CallArgBase + 6));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("TXA");
            EmitAsm("ADC", Mem(CallArgBase + 7));
            EmitAsm("TAX");
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");
        }


        // Emit rectangle tests, byte distance, word map-index arithmetic and persistent RNG state operations.
        void EmitHelperGeometryAndRng()
        {
            EmitHelperStart("__xy_in_rect");
            string xyFalse = NewGeneratedLabel("xy_in_rect_false");
            string xyDone = NewGeneratedLabel("xy_in_rect_done");
            // return (x-rx < rw) && (y-ry < rh), using unsigned subtract to avoid rx+rw overflow.
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("SEC");
            EmitAsm("SBC", Mem(CallArgBase + 2));
            EmitAsm("BCC", Rel(xyFalse));
            EmitAsm("CMP", Mem(CallArgBase + 4));
            EmitAsm("BCS", Rel(xyFalse));
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("SEC");
            EmitAsm("SBC", Mem(CallArgBase + 3));
            EmitAsm("BCC", Rel(xyFalse));
            EmitAsm("CMP", Mem(CallArgBase + 5));
            EmitAsm("BCS", Rel(xyFalse));
            EmitAsm("LDA", Imm(1));
            EmitAsm("JMP", Abs(xyDone));
            _assembly.Add(Expr.Make(Tag.Label, xyFalse));
            EmitAsm("LDA", Imm(0));
            _assembly.Add(Expr.Make(Tag.Label, xyDone));
            EmitAsm("RTS");

            EmitHelperStart("__manhattan");
            // dx = abs(x1-x2)
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("SEC");
            EmitAsm("SBC", Mem(CallArgBase + 2));
            string mdxOk = NewGeneratedLabel("manhattan_dx_ok");
            EmitAsm("BCS", Rel(mdxOk));
            EmitAsm("EOR", Imm(0xFF));
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(1));
            _assembly.Add(Expr.Make(Tag.Label, mdxOk));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            // dy = abs(y1-y2), then add dx.
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("SEC");
            EmitAsm("SBC", Mem(CallArgBase + 3));
            string mdyOk = NewGeneratedLabel("manhattan_dy_ok");
            EmitAsm("BCS", Rel(mdyOk));
            EmitAsm("EOR", Imm(0xFF));
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(1));
            _assembly.Add(Expr.Make(Tag.Label, mdyOk));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");

            // Save X-coordinate separately while multiplying width by Y, then add X with carry into the high byte.
            EmitHelperStart("__map_index");
            // args: x, y, width. Return y * width + x as u16.
            EmitAsm("LDA", Mem(CallArgBase));      // x
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("LDA", Mem(CallArgBase + 1));  // y
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDA", Mem(CallArgBase + 2));  // width as u16 multiplicand
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("STA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs("__mul16x8"));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("TXA");
            EmitAsm("ADC", Imm(0));
            EmitAsm("TAX");
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");

            // Replace both bytes of the persistent generator state with the supplied seed.
            EmitHelperStart("__rng_seed");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("STA", Mem(_runtimeRngLoAddress));
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("STA", Mem(_runtimeRngHiAddress));
            EmitAsm("RTS");

            EmitHelperStart("__rng8");
            // 16-bit Galois LFSR: if low bit set, shift right then xor $B400.
            EmitAsm("LDA", Mem(_runtimeRngLoAddress));
            EmitAsm("ORA", Mem(_runtimeRngHiAddress));
            // Replace the all-zero LFSR state with a deterministic nonzero seed before advancing.
            string seeded = NewGeneratedLabel("rng_seeded");
            EmitAsm("BNE", Rel(seeded));
            EmitAsm("LDA", Imm(0x5A));
            EmitAsm("STA", Mem(_runtimeRngLoAddress));
            EmitAsm("LDA", Imm(0xA5));
            EmitAsm("STA", Mem(_runtimeRngHiAddress));
            _assembly.Add(Expr.Make(Tag.Label, seeded));
            EmitAsm("LDA", Mem(_runtimeRngLoAddress));
            EmitAsm("AND", Imm(1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LSR", Mem(_runtimeRngHiAddress));
            EmitAsm("ROR", Mem(_runtimeRngLoAddress));
            string rngNoXor = NewGeneratedLabel("rng_no_xor");
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("BEQ", Rel(rngNoXor));
            EmitAsm("LDA", Mem(_runtimeRngHiAddress));
            EmitAsm("EOR", Imm(0xB4));
            EmitAsm("STA", Mem(_runtimeRngHiAddress));
            _assembly.Add(Expr.Make(Tag.Label, rngNoXor));
            EmitAsm("LDA", Mem(_runtimeRngLoAddress));
            EmitAsm("EOR", Mem(_runtimeRngHiAddress));
            EmitAsm("RTS");
        }

        // Save the current PRG bank on the CPU stack, perform the read/copy, then restore that bank.
        void EmitHelperFarMemory()
        {
            // Keep the fetched byte in intrinsic scratch while the bank-switch helper restores the previous mapping.
            EmitHelperStart("__farpeek8");
            // args: bank, addr16. Return A.
            EmitAsm("LDA", Mem(_runtimeCurrentBankAddress));
            EmitAsm("PHA");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
            EmitAsm("LDY", Imm(0));
            EmitAsm("LDA", IndY(CallArgBase + 1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("PLA");
            EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");

            // Retain both fetched bytes across bank restoration and return the word in A/X.
            EmitHelperStart("__farpeek16");
            // args: bank, addr16. Return A=lo, X=hi.
            EmitAsm("LDA", Mem(_runtimeCurrentBankAddress));
            EmitAsm("PHA");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
            EmitAsm("LDY", Imm(0));
            EmitAsm("LDA", IndY(CallArgBase + 1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("INY");
            EmitAsm("LDA", IndY(CallArgBase + 1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("PLA");
            EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDX", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("RTS");

            // Consume the word count while advancing source/destination pointers; zero count still restores the saved bank.
            EmitHelperStart("__far_memcpy");
            // args: dst16, bank, src16, len16.
            EmitAsm("LDA", Mem(_runtimeCurrentBankAddress));
            EmitAsm("PHA");
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
            string fmDone = NewGeneratedLabel("far_memcpy_done");
            string fmLoop = NewGeneratedLabel("far_memcpy_loop");
            EmitAsm("LDA", Mem(CallArgBase + 5));
            EmitAsm("ORA", Mem(CallArgBase + 6));
            EmitAsm("BEQ", Rel(fmDone));
            _assembly.Add(Expr.Make(Tag.Label, fmLoop));
            EmitAsm("LDY", Imm(0));
            EmitAsm("LDA", IndY(CallArgBase + 3));
            EmitAsm("STA", IndY(CallArgBase));
            EmitAsm("INC", Mem(CallArgBase + 3));
            string fmsrcOk = NewGeneratedLabel("far_memcpy_src_page_ok");
            EmitAsm("BNE", Rel(fmsrcOk));
            EmitAsm("INC", Mem(CallArgBase + 4));
            _assembly.Add(Expr.Make(Tag.Label, fmsrcOk));
            EmitAsm("INC", Mem(CallArgBase));
            string fmdstOk = NewGeneratedLabel("far_memcpy_dst_page_ok");
            EmitAsm("BNE", Rel(fmdstOk));
            EmitAsm("INC", Mem(CallArgBase + 1));
            _assembly.Add(Expr.Make(Tag.Label, fmdstOk));
            EmitAsm("LDA", Mem(CallArgBase + 5));
            string fmlenLoNonzero = NewGeneratedLabel("far_memcpy_lenlo_nonzero");
            EmitAsm("BNE", Rel(fmlenLoNonzero));
            EmitAsm("DEC", Mem(CallArgBase + 6));
            _assembly.Add(Expr.Make(Tag.Label, fmlenLoNonzero));
            EmitAsm("DEC", Mem(CallArgBase + 5));
            EmitAsm("LDA", Mem(CallArgBase + 5));
            EmitAsm("ORA", Mem(CallArgBase + 6));
            EmitAsm("BNE", Rel(fmLoop));
            _assembly.Add(Expr.Make(Tag.Label, fmDone));
            EmitAsm("PLA");
            EmitAsm("JSR", Abs("__kq_prg_set_bank_a"));
            EmitAsm("RTS");
        }

        // Encode fixed-size put/fill/copy records and emit the interpreter for a committed queue.
        void EmitHelperVramQueue()
        {
            // Append tag 3, a little-endian PPU address and one tile byte after reserving four queue bytes.
            EmitHelperStart("__vramq_put");
            EmitVramqOverflowGuard(4);
            EmitAsm("LDA", Imm(3)); EmitAsm("STA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX");
            EmitVramqWriteArgByte(0); EmitVramqWriteArgByte(1); EmitVramqWriteArgByte(2);
            EmitAsm("STX", Mem(_runtimeVramqLenAddress));
            EmitAsm("RTS");

            // Append tag 2, address, byte count and fill value in the interpreter record order.
            EmitHelperStart("__vramq_fill");
            EmitVramqOverflowGuard(5);
            EmitAsm("LDA", Imm(2)); EmitAsm("STA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX");
            EmitVramqWriteArgByte(0); EmitVramqWriteArgByte(1); EmitVramqWriteArgByte(3); EmitVramqWriteArgByte(2);
            EmitAsm("STX", Mem(_runtimeVramqLenAddress));
            EmitAsm("RTS");

            // Append tag 1, address, byte count and a source pointer; source data is read when the queue executes.
            EmitHelperStart("__vramq_copy");
            EmitVramqOverflowGuard(6);
            EmitAsm("LDA", Imm(1)); EmitAsm("STA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX");
            EmitVramqWriteArgByte(0); EmitVramqWriteArgByte(1); EmitVramqWriteArgByte(4); EmitVramqWriteArgByte(2); EmitVramqWriteArgByte(3);
            EmitAsm("STX", Mem(_runtimeVramqLenAddress));
            EmitAsm("RTS");

            // Execute only a ready queue, dispatching by record tag until its length or an unknown tag ends processing.
            EmitHelperStart("__vramq_exec");
            string done = NewGeneratedLabel("vramq_exec_done");
            string loopq = NewGeneratedLabel("vramq_exec_loop");
            string cmdCopy = NewGeneratedLabel("vramq_cmd_copy");
            string cmdFill = NewGeneratedLabel("vramq_cmd_fill");
            string cmdPut = NewGeneratedLabel("vramq_cmd_put");
            string finish = NewGeneratedLabel("vramq_finish");
            EmitAsm("LDA", Mem(_runtimeVramqReadyAddress));
            EmitAsm("BEQ", Rel(done));
            EmitAsm("LDX", Imm(0));
            _assembly.Add(Expr.Make(Tag.Label, loopq));
            EmitAsm("CPX", Mem(_runtimeVramqLenAddress));
            EmitAsm("BEQ", Rel(finish));
            EmitAsm("LDA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX));
            EmitAsm("INX");
            EmitAsm("CMP", Imm(1)); EmitAsm("BEQ", Rel(cmdCopy));
            EmitAsm("CMP", Imm(2)); EmitAsm("BEQ", Rel(cmdFill));
            EmitAsm("CMP", Imm(3)); EmitAsm("BEQ", Rel(cmdPut));
            EmitAsm("JMP", Abs(finish));

            _assembly.Add(Expr.Make(Tag.Label, cmdPut));
            EmitVramqReadToTmpAddr();
            EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address)); EmitAsm("STA", Mem(0x2006)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX"); EmitAsm("STA", Mem(0x2007));
            EmitAsm("JMP", Abs(loopq));

            _assembly.Add(Expr.Make(Tag.Label, cmdFill));
            EmitVramqReadToTmpAddr();
            EmitAsm("LDA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX"); EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("LDA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX"); EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("STX", Mem(CallArgBase + 1));
            EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address)); EmitAsm("STA", Mem(0x2006)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDY", Imm(0));
            // Keep the next record position outside X while streaming the fill count through Y.
            string fillLoop = NewGeneratedLabel("vramq_fill_loop");
            string fillDone = NewGeneratedLabel("vramq_fill_done");
            _assembly.Add(Expr.Make(Tag.Label, fillLoop));
            EmitAsm("CPY", Mem(_runtimeIntrinsicTmp2Address)); EmitAsm("BEQ", Rel(fillDone));
            EmitAsm("LDA", Mem(CallArgBase)); EmitAsm("STA", Mem(0x2007)); EmitAsm("INY"); EmitAsm("BNE", Rel(fillLoop));
            _assembly.Add(Expr.Make(Tag.Label, fillDone));
            EmitAsm("LDX", Mem(CallArgBase + 1));
            EmitAsm("JMP", Abs(loopq));

            _assembly.Add(Expr.Make(Tag.Label, cmdCopy));
            EmitVramqReadToTmpAddr();
            EmitAsm("LDA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX"); EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("LDA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX"); EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("LDA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX"); EmitAsm("STA", Mem(CallArgBase + 1));
            EmitAsm("STX", Mem(CallArgBase + 2));
            EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address)); EmitAsm("STA", Mem(0x2006)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDY", Imm(0));
            // Read the queued source through indirect-Y addressing, then restore the next record position.
            string copyLoop = NewGeneratedLabel("vramq_copy_loop");
            string copyDone = NewGeneratedLabel("vramq_copy_done");
            _assembly.Add(Expr.Make(Tag.Label, copyLoop));
            EmitAsm("CPY", Mem(_runtimeIntrinsicTmp2Address)); EmitAsm("BEQ", Rel(copyDone));
            EmitAsm("LDA", IndY(CallArgBase)); EmitAsm("STA", Mem(0x2007)); EmitAsm("INY"); EmitAsm("BNE", Rel(copyLoop));
            _assembly.Add(Expr.Make(Tag.Label, copyDone));
            EmitAsm("LDX", Mem(CallArgBase + 2));
            EmitAsm("JMP", Abs(loopq));

            _assembly.Add(Expr.Make(Tag.Label, finish));
            EmitAsm("LDA", Imm(0)); EmitAsm("STA", Mem(_runtimeVramqReadyAddress)); EmitAsm("STA", Mem(_runtimeVramqLenAddress));
            _assembly.Add(Expr.Make(Tag.Label, done));
            EmitAsm("RTS");
        }

        // Reject an append that cannot fit its complete record, set overflow and return without changing queue length.
        void EmitVramqOverflowGuard(int recordSize)
        {
            string ok = NewGeneratedLabel("vramq_space_ok");
            EmitAsm("LDX", Mem(_runtimeVramqLenAddress));
            EmitAsm("CPX", Imm((VramqCapacity - recordSize + 1) & 0xFF));
            EmitAsm("BCC", Rel(ok));
            EmitAsm("LDA", Imm(1));
            EmitAsm("STA", Mem(_runtimeVramqOverflowAddress));
            EmitAsm("RTS");
            _assembly.Add(Expr.Make(Tag.Label, ok));
        }

        // Append one argument byte and advance X to the next queue slot.
        void EmitVramqWriteArgByte(int argOffset)
        {
            EmitAsm("LDA", Mem(CallArgBase + argOffset));
            EmitAsm("STA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX));
            EmitAsm("INX");
        }

        // Consume the next two queue bytes as a little-endian PPU address in scratch.
        void EmitVramqReadToTmpAddr()
        {
            EmitAsm("LDA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX"); EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", new AsmOperand(_runtimeVramqBufferAddress, AddressMode.AbsoluteX)); EmitAsm("INX"); EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
        }

        // Fill each row from a computed nametable-zero address, advancing the row start by 32 bytes.
        void EmitHelperNametableRect()
        {
            EmitHelperStart("__nametable_rect");
            // args: x,y,w,h,tile; writes tile-filled rectangle to nametable 0.
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            for (int i = 0; i < 5; i++) { EmitAsm("ASL", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("ROL", Mem(_runtimeIntrinsicTmp1Address)); }
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("CLC"); EmitAsm("ADC", Mem(CallArgBase)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address)); EmitAsm("ADC", Imm(0x20)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDA", Mem(CallArgBase + 3)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            string rowLoop = NewGeneratedLabel("nt_rect_row");
            string done = NewGeneratedLabel("nt_rect_done");
            _assembly.Add(Expr.Make(Tag.Label, rowLoop));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp2Address)); EmitAsm("BEQ", Rel(done));
            EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address)); EmitAsm("STA", Mem(0x2006)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDY", Imm(0));
            string colLoop = NewGeneratedLabel("nt_rect_col");
            string rowDone = NewGeneratedLabel("nt_rect_row_done");
            _assembly.Add(Expr.Make(Tag.Label, colLoop));
            EmitAsm("CPY", Mem(CallArgBase + 2)); EmitAsm("BEQ", Rel(rowDone));
            EmitAsm("LDA", Mem(CallArgBase + 4)); EmitAsm("STA", Mem(0x2007)); EmitAsm("INY"); EmitAsm("BNE", Rel(colLoop));
            _assembly.Add(Expr.Make(Tag.Label, rowDone));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("CLC"); EmitAsm("ADC", Imm(32)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("BCC", Rel(rowLoop + "_nohi"));
            EmitAsm("INC", Mem(_runtimeIntrinsicTmp1Address));
            _assembly.Add(Expr.Make(Tag.Label, rowLoop + "_nohi"));
            EmitAsm("DEC", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("JMP", Abs(rowLoop));
            _assembly.Add(Expr.Make(Tag.Label, done));
            EmitAsm("RTS");
        }


        // Use the selected nametable base, then fill rows while preserving the original row width.
        void EmitHelperNametableRectNt()
        {
            EmitHelperStart("__nametable_rect_nt");
            // args: nt,x,y,w,h,tile.
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("AND", Imm(3));
            EmitAsm("ASL");
            EmitAsm("ASL");
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(0x20));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Imm(0));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            for (int i = 0; i < 5; i++) { EmitAsm("ASL", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("ROL", Mem(_runtimeIntrinsicTmp1Address)); }
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("CLC"); EmitAsm("ADC", Mem(CallArgBase + 1)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address)); EmitAsm("ADC", Mem(_runtimeIntrinsicTmp2Address)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDA", Mem(CallArgBase + 4)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            string rowLoop = NewGeneratedLabel("nt_rect_nt_row");
            string done = NewGeneratedLabel("nt_rect_nt_done");
            _assembly.Add(Expr.Make(Tag.Label, rowLoop));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp2Address)); EmitAsm("BEQ", Rel(done));
            EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address)); EmitAsm("STA", Mem(0x2006)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDY", Imm(0));
            string colLoop = NewGeneratedLabel("nt_rect_nt_col");
            string rowDone = NewGeneratedLabel("nt_rect_nt_row_done");
            _assembly.Add(Expr.Make(Tag.Label, colLoop));
            EmitAsm("CPY", Mem(CallArgBase + 3)); EmitAsm("BEQ", Rel(rowDone));
            EmitAsm("LDA", Mem(CallArgBase + 5)); EmitAsm("STA", Mem(0x2007)); EmitAsm("INY"); EmitAsm("BNE", Rel(colLoop));
            _assembly.Add(Expr.Make(Tag.Label, rowDone));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("CLC"); EmitAsm("ADC", Imm(32)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("BCC", Rel(rowLoop + "_nohi"));
            EmitAsm("INC", Mem(_runtimeIntrinsicTmp1Address));
            _assembly.Add(Expr.Make(Tag.Label, rowLoop + "_nohi"));
            EmitAsm("DEC", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("JMP", Abs(rowLoop));
            _assembly.Add(Expr.Make(Tag.Label, done));
            EmitAsm("RTS");
        }


        // Write the full attribute byte covering the requested tile coordinate in nametable zero.
        void EmitHelperAttrSet()
        {
            EmitHelperStart("__attr_set");
            // args: x,y,attr. Attribute address = $23C0 + (y>>2)*8 + (x>>2), nametable 0.
            EmitAsm("LDA", Mem(CallArgBase + 1));
            EmitAsm("LSR"); EmitAsm("LSR"); EmitAsm("ASL"); EmitAsm("ASL"); EmitAsm("ASL");
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(CallArgBase)); EmitAsm("LSR"); EmitAsm("LSR"); EmitAsm("CLC"); EmitAsm("ADC", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Imm(0x23)); EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("CLC"); EmitAsm("ADC", Imm(0xC0)); EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("STA", Mem(0x2007));
            EmitAsm("RTS");
        }


        // Select the nametable and write its complete attribute byte; this does not merge a single palette quadrant.
        void EmitHelperAttrSetNt()
        {
            EmitHelperStart("__attr_set_nt");
            // args: nt,x,y,attr. Attribute address = $23C0 + nt*$400 + (y>>2)*8 + (x>>2).
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("AND", Imm(3));
            EmitAsm("ASL");
            EmitAsm("ASL");
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(0x23));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("LDA", Mem(CallArgBase + 2));
            EmitAsm("LSR"); EmitAsm("LSR"); EmitAsm("ASL"); EmitAsm("ASL"); EmitAsm("ASL");
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("LSR"); EmitAsm("LSR"); EmitAsm("CLC"); EmitAsm("ADC", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("LDA", Mem(0x2002)); EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address)); EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address)); EmitAsm("CLC"); EmitAsm("ADC", Imm(0xC0)); EmitAsm("STA", Mem(0x2006));
            EmitAsm("LDA", Mem(CallArgBase + 3)); EmitAsm("STA", Mem(0x2007));
            EmitAsm("RTS");
        }


        // Emit shadow-OAM field updates and terminated metasprite-stream expansion.
        void EmitHelperSprites()
        {
            // Store Y, tile, attributes and X in hardware OAM byte order for the selected entry.
            EmitHelperStart("__sprite_set");
            EmitSpriteIndexToX();
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("STA", new AsmOperand(OamRamBase + 0, AddressMode.AbsoluteX));
            EmitAsm("LDA", Mem(CallArgBase + 3)); EmitAsm("STA", new AsmOperand(OamRamBase + 1, AddressMode.AbsoluteX));
            EmitAsm("LDA", Mem(CallArgBase + 4)); EmitAsm("STA", new AsmOperand(OamRamBase + 2, AddressMode.AbsoluteX));
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("STA", new AsmOperand(OamRamBase + 3, AddressMode.AbsoluteX));
            EmitAsm("RTS");

            // Replace only the entry coordinates, retaining its tile and attribute bytes.
            EmitHelperStart("__sprite_move");
            EmitSpriteIndexToX();
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("STA", new AsmOperand(OamRamBase + 0, AddressMode.AbsoluteX));
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("STA", new AsmOperand(OamRamBase + 3, AddressMode.AbsoluteX));
            EmitAsm("RTS");

            // Replace one shadow-OAM tile byte.
            EmitHelperStart("__sprite_tile");
            EmitSpriteIndexToX();
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("STA", new AsmOperand(OamRamBase + 1, AddressMode.AbsoluteX));
            EmitAsm("RTS");

            // Replace one shadow-OAM attribute byte.
            EmitHelperStart("__sprite_attr");
            EmitSpriteIndexToX();
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("STA", new AsmOperand(OamRamBase + 2, AddressMode.AbsoluteX));
            EmitAsm("RTS");

            // Move the selected sprite off-screen by writing its Y byte.
            EmitHelperStart("__sprite_hide");
            EmitSpriteIndexToX();
            EmitAsm("LDA", Imm(0xF0)); EmitAsm("STA", new AsmOperand(OamRamBase + 0, AddressMode.AbsoluteX));
            EmitAsm("RTS");

            // Expand four-byte records into successive OAM entries until the reserved DX terminator is encountered.
            EmitHelperStart("__metasprite_draw");
            // args: oam_index, base_x, base_y, metasprite_ptr16.
            // stream: dx, dy, tile, attr ... dx=$FF terminates. Returns next OAM index in A.
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("ASL");
            EmitAsm("ASL");
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address)); // dst byte offset
            string msLoop = NewGeneratedLabel("metasprite_loop");
            string msDone = NewGeneratedLabel("metasprite_done");
            _assembly.Add(Expr.Make(Tag.Label, msLoop));
            EmitAsm("LDY", Imm(0));
            EmitAsm("LDA", IndY(CallArgBase + 3));
            EmitAsm("CMP", Imm(0xFF));
            EmitAsm("BEQ", Rel(msDone));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address)); // dx
            EmitAsm("LDX", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("INY");
            EmitAsm("LDA", IndY(CallArgBase + 3));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(CallArgBase + 2));
            EmitAsm("STA", new AsmOperand(OamRamBase + 0, AddressMode.AbsoluteX));
            EmitAsm("INY");
            EmitAsm("LDA", IndY(CallArgBase + 3));
            EmitAsm("STA", new AsmOperand(OamRamBase + 1, AddressMode.AbsoluteX));
            EmitAsm("INY");
            EmitAsm("LDA", IndY(CallArgBase + 3));
            EmitAsm("STA", new AsmOperand(OamRamBase + 2, AddressMode.AbsoluteX));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("CLC");
            EmitAsm("ADC", Mem(CallArgBase + 1));
            EmitAsm("STA", new AsmOperand(OamRamBase + 3, AddressMode.AbsoluteX));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(4));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("LDA", Mem(CallArgBase + 3));
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(4));
            EmitAsm("STA", Mem(CallArgBase + 3));
            // Propagate stream-pointer page carry after consuming the complete four-byte record.
            string msPtrOk = NewGeneratedLabel("metasprite_ptr_ok");
            EmitAsm("BCC", Rel(msPtrOk));
            EmitAsm("INC", Mem(CallArgBase + 4));
            _assembly.Add(Expr.Make(Tag.Label, msPtrOk));
            EmitAsm("JMP", Abs(msLoop));
            _assembly.Add(Expr.Make(Tag.Label, msDone));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp2Address));
            EmitAsm("LSR");
            EmitAsm("LSR");
            EmitAsm("RTS");
        }

        // Convert a sprite index to its four-byte offset in the OAM page.
        void EmitSpriteIndexToX()
        {
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("ASL");
            EmitAsm("ASL");
            EmitAsm("TAX");
        }

        // Search compiled six-byte file records by ID and return existence or little-endian size; this does not scan the disk.
        void EmitFdsMetadataSearch(bool foundReturnsSize)
        {
            string loop = NewGeneratedLabel("fds_meta_loop");
            string found = NewGeneratedLabel("fds_meta_found");
            string missing = NewGeneratedLabel("fds_meta_missing");
            EmitAsm("LDY", Imm(0));
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("LDA", AbsY("__kq_fds_metadata_table"));
            EmitAsm("CMP", Imm(0xFF));
            EmitAsm("BEQ", Rel(missing));
            EmitAsm("CMP", Mem(CallArgBase));
            EmitAsm("BEQ", Rel(found));
            EmitAsm("TYA");
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(6));
            EmitAsm("TAY");
            EmitAsm("BNE", Rel(loop));
            _assembly.Add(Expr.Make(Tag.Label, missing));
            EmitAsm("LDA", Imm(0));
            if (foundReturnsSize) EmitAsm("LDX", Imm(0));
            EmitAsm("RTS");
            _assembly.Add(Expr.Make(Tag.Label, found));
            if (!foundReturnsSize)
            {
                EmitAsm("LDA", Imm(1));
                EmitAsm("RTS");
                return;
            }
            EmitAsm("INY");
            EmitAsm("LDA", AbsY("__kq_fds_metadata_table"));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("INY");
            EmitAsm("LDA", AbsY("__kq_fds_metadata_table"));
            EmitAsm("TAX");
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");
        }

        // Emit FDS BIOS-call wrappers, overlay residency tracking and sound-register transfer helpers.
        void EmitHelperFds()
        {
            // Wildcard Disk ID for BIOS direct-pointer routines. $FF means "do not compare" for that field.
            _assembly.Add(Expr.Make(Tag.ReadonlyData, "__kq_fds_disk_id_wildcard", new byte[] { 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF }));
            _assembly.Add(Expr.Make(Tag.ReadonlyData, "__kq_fds_metadata_table", (Program.FdsMetadata ?? FdsDiskMetadata.Empty).BuildRuntimeTable()));
            _assembly.Add(Expr.Make(Tag.ReadonlyData, "__kq_fds_overlay_function_table", new byte[] { 0 }));

            // Wait for the disk-inserted status bit to clear; this loop checks presence and has no timeout.
            EmitHelperStart("__fds_wait_ready");
            string loop = NewGeneratedLabel("fds_wait_ready_loop");
            _assembly.Add(Expr.Make(Tag.Label, loop));
            EmitAsm("LDA", Mem(0x4032));
            EmitAsm("AND", Imm(0x01));
            EmitAsm("BNE", Rel(loop));
            EmitAsm("RTS");

            // Build a one-file load list and supply inline wildcard-ID/list pointers to the BIOS LoadFiles entry.
            EmitHelperStart("__fds_load_file");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("STA", Mem(_runtimeFdsLoadListAddress));
            EmitAsm("LDA", Imm(0xFF));
            EmitAsm("STA", Mem(_runtimeFdsLoadListAddress + 1));
            EmitAsm("JSR", Mem(0xE1F8)); // FDS BIOS LoadFiles; A=error, Y=count.
            _assembly.Add(Expr.Make(Tag.Word, "__kq_fds_disk_id_wildcard"));
            _assembly.Add(Expr.Make(Tag.Word, string.Format("${0:X4}", _runtimeFdsLoadListAddress)));
            EmitAsm("RTS");

            // Use the same BIOS loading convention for an overlay file identifier.
            EmitHelperStart("__fds_load_overlay");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("STA", Mem(_runtimeFdsLoadListAddress));
            EmitAsm("LDA", Imm(0xFF));
            EmitAsm("STA", Mem(_runtimeFdsLoadListAddress + 1));
            EmitAsm("JSR", Mem(0xE1F8));
            _assembly.Add(Expr.Make(Tag.Word, "__kq_fds_disk_id_wildcard"));
            _assembly.Add(Expr.Make(Tag.Word, string.Format("${0:X4}", _runtimeFdsLoadListAddress)));
            EmitAsm("RTS");

            // Translate logical banks starting at two to overlay file IDs, optionally reusing the recorded resident bank.
            EmitHelperStart("__fds_load_bank");
            string fdsLoadBankOk = NewGeneratedLabel("fds_load_bank_ok");
            string fdsLoadBankLoad = NewGeneratedLabel("fds_load_bank_load");
            string fdsLoadBankDone = NewGeneratedLabel("fds_load_bank_done");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("CMP", Imm(2));
            EmitAsm("BCS", Rel(fdsLoadBankOk));
            EmitAsm("LDA", Imm(0xFF));
            EmitAsm("RTS");
            _assembly.Add(Expr.Make(Tag.Label, fdsLoadBankOk));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp1Address));
            if (Program.FdsOverlayResidencyGuardEnabled)
            {
                EmitAsm("CMP", Mem(_runtimeFdsResidentBankAddress));
                EmitAsm("BNE", Rel(fdsLoadBankLoad));
                EmitAsm("LDA", Imm(0));
                EmitAsm("RTS");
                _assembly.Add(Expr.Make(Tag.Label, fdsLoadBankLoad));
            }
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address));
            EmitAsm("SEC");
            EmitAsm("SBC", Imm(2));
            EmitAsm("CLC");
            EmitAsm("ADC", Imm(Program.FdsOverlayStartId & 0xFF));
            EmitAsm("STA", Mem(CallArgBase));
            EmitAsm("JSR", Abs("__fds_load_overlay"));
            EmitAsm("STA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("CMP", Imm(0));
            EmitAsm("BNE", Rel(fdsLoadBankDone));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp1Address));
            // Update both bank shadows only after the BIOS load reports success.
            EmitAsm("STA", Mem(_runtimeFdsResidentBankAddress));
            EmitAsm("STA", Mem(_runtimeCurrentBankAddress));
            _assembly.Add(Expr.Make(Tag.Label, fdsLoadBankDone));
            EmitAsm("LDA", Mem(_runtimeIntrinsicTmp0Address));
            EmitAsm("RTS");

            // Return success for the recorded resident bank, otherwise dispatch a bank load.
            EmitHelperStart("__fds_require_bank");
            string fdsRequireLoad = NewGeneratedLabel("fds_require_load");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("CMP", Mem(_runtimeFdsResidentBankAddress));
            EmitAsm("BNE", Rel(fdsRequireLoad));
            EmitAsm("LDA", Imm(0));
            EmitAsm("RTS");
            _assembly.Add(Expr.Make(Tag.Label, fdsRequireLoad));
            EmitAsm("JSR", Abs("__fds_load_bank"));
            EmitAsm("RTS");

            // Compare the requested bank with the software residency record and return zero or one.
            EmitHelperStart("__fds_is_bank_resident");
            string fdsResidentYes = NewGeneratedLabel("fds_resident_yes");
            EmitAsm("LDA", Mem(CallArgBase));
            EmitAsm("CMP", Mem(_runtimeFdsResidentBankAddress));
            EmitAsm("BEQ", Rel(fdsResidentYes));
            EmitAsm("LDA", Imm(0));
            EmitAsm("RTS");
            _assembly.Add(Expr.Make(Tag.Label, fdsResidentYes));
            EmitAsm("LDA", Imm(1));
            EmitAsm("RTS");

            // Query file existence in the compiled metadata table.
            EmitHelperStart("__fds_file_exists");
            EmitFdsMetadataSearch(foundReturnsSize: false);

            // Return the compiled metadata size, using zero for an unmatched ID.
            EmitHelperStart("__fds_file_size");
            EmitFdsMetadataSearch(foundReturnsSize: true);

            // Construct a RAM-source WriteFile header and invoke the BIOS with inline descriptor pointers.
            EmitHelperStart("__fds_save_file");
            // Build a minimal BIOS WriteFile header in RAM.
            // args: id, src, len. File number uses id; file name is "KQFCFILE"; load/source addr = src.
            EmitAsm("LDA", Mem(CallArgBase)); EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 0));
            byte[] nameBytes = System.Text.Encoding.ASCII.GetBytes("KQFCFILE");
            for (int i = 0; i < 8; i++)
            {
                EmitAsm("LDA", Imm(nameBytes[i]));
                EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 1 + i));
            }
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 9));  // load addr lo
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 10)); // load addr hi
            EmitAsm("LDA", Mem(CallArgBase + 3)); EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 11)); // size lo
            EmitAsm("LDA", Mem(CallArgBase + 4)); EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 12)); // size hi
            EmitAsm("LDA", Imm(0)); EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 13)); // file type PRG
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 14)); // source addr lo
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 15)); // source addr hi
            EmitAsm("LDA", Imm(0)); EmitAsm("STA", Mem(_runtimeFdsFileHeaderAddress + 16)); // source address type: RAM
            EmitAsm("LDA", Mem(CallArgBase)); // sequential file number; expected to match id in KITAQFC simple-save convention.
            EmitAsm("JSR", Mem(0xE239)); // FDS BIOS WriteFile
            _assembly.Add(Expr.Make(Tag.Word, "__kq_fds_disk_id_wildcard"));
            _assembly.Add(Expr.Make(Tag.Word, string.Format("${0:X4}", _runtimeFdsFileHeaderAddress)));
            EmitAsm("RTS");

            // Enable wave-RAM writes, copy 64 source bytes, then clear the wave-write control register.
            EmitHelperStart("__fds_wave_load");
            EmitAsm("LDA", Imm(0x80)); EmitAsm("STA", Mem(0x4089));
            EmitAsm("LDY", Imm(0));
            string waveLoop = NewGeneratedLabel("fds_wave_loop");
            _assembly.Add(Expr.Make(Tag.Label, waveLoop));
            EmitAsm("LDA", IndY(CallArgBase));
            EmitAsm("STA", new AsmOperand(0x4040, AddressMode.AbsoluteY));
            EmitAsm("INY"); EmitAsm("CPY", Imm(64)); EmitAsm("BNE", Rel(waveLoop));
            EmitAsm("LDA", Imm(0x00)); EmitAsm("STA", Mem(0x4089));
            EmitAsm("RTS");

            // Stream 32 source bytes to the modulation-table write port; setup of modulation state belongs to the caller.
            EmitHelperStart("__fds_mod_load");
            EmitAsm("LDY", Imm(0));
            string modLoop = NewGeneratedLabel("fds_mod_loop");
            _assembly.Add(Expr.Make(Tag.Label, modLoop));
            EmitAsm("LDA", IndY(CallArgBase));
            EmitAsm("STA", Mem(0x4088));
            EmitAsm("INY"); EmitAsm("CPY", Imm(32)); EmitAsm("BNE", Rel(modLoop));
            EmitAsm("RTS");

            // Write the low frequency byte and only the low four bits of its high byte.
            EmitHelperStart("__fds_freq_set");
            EmitAsm("LDA", Mem(CallArgBase)); EmitAsm("STA", Mem(0x4082));
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("AND", Imm(0x0F)); EmitAsm("STA", Mem(0x4083));
            EmitAsm("RTS");

            // Write the supplied volume-envelope, modulation-envelope and envelope-speed registers.
            EmitHelperStart("__fds_env_set");
            EmitAsm("LDA", Mem(CallArgBase)); EmitAsm("STA", Mem(0x4080));
            EmitAsm("LDA", Mem(CallArgBase + 1)); EmitAsm("STA", Mem(0x4084));
            EmitAsm("LDA", Mem(CallArgBase + 2)); EmitAsm("STA", Mem(0x408A));
            EmitAsm("RTS");
        }

        // Collect compiler analysis records in deterministic order for reports and downstream tooling.
        public CodegenAnalysisReport BuildCodegenAnalysisReport()
        {
            var report = new CodegenAnalysisReport();

            // Describe each known function, including prototypes, with its requested placement and calling convention.
            foreach (var kv in _functionInfos.OrderBy(x => x.Key, StringComparer.Ordinal))
            {
                var fi = kv.Value;
                report.Functions.Add(new FunctionAbiInfo
                {
                    Name = kv.Key,
                    Bank = fi.RomBank,
                    HasFixedBank = fi.HasFixedBank,
                    PlacementOrder = fi.PlacementOrder,
                    HasFixedOrder = fi.HasFixedOrder,
                    IsPrototype = fi.IsPrototype,
                    IsInline = fi.IsInline,
                    IsStackCall = fi.IsStackCall,
                    IsFastCall = fi.IsFastCall,
                    ReturnSize = SizeOfTypeLoose(fi.ReturnType),
                    ParamSizes = (fi.Parameters ?? Array.Empty<FieldInfo>()).Select(p => SizeOfTypeLoose(p.Type)).ToArray(),
                });
            }

            // Report named readonly slots using requested banks; actual placement records are collected separately below.
            foreach (var slot in _orderedReadonlyData
                .Where(x => x != null && !string.IsNullOrEmpty(x.Name))
                .OrderBy(x => x.RequestedBank)
                .ThenBy(x => x.PlacementOrder)
                .ThenBy(x => x.Name, StringComparer.Ordinal))
            {
                report.ReadonlyData.Add(new ReadonlyDataInfo
                {
                    Name = slot.Name,
                    Bank = slot.RequestedBank,
                    RequestedBank = slot.RequestedBank,
                    HasFixedBank = slot.HasFixedBank,
                    PlacementOrder = slot.PlacementOrder,
                    HasFixedOrder = slot.PlacementOrder != int.MaxValue,
                    SizeBytes = slot.Size,
                });
            }

            // Keep caller, callee and call kind ordering stable across dictionary iteration order.
            foreach (var edge in _callReportMap.Values
                .OrderBy(x => x.Caller, StringComparer.Ordinal)
                .ThenBy(x => x.Callee, StringComparer.Ordinal)
                .ThenBy(x => x.Kind, StringComparer.Ordinal))
            {
                report.Calls.Add(edge);
            }

            // Retain source locations for recorded NES actions so diagnostics can identify their origin.
            foreach (var action in _nesActionUses
                .OrderBy(x => x.Caller, StringComparer.Ordinal)
                .ThenBy(x => x.Name, StringComparer.Ordinal)
                .ThenBy(x => x.Source, StringComparer.Ordinal))
            {
                report.NesActions.Add(action);
            }

            // List allocated memory by address and frame slots by owning function.
            report.ZpAllocations.AddRange(_zpAllocations.OrderBy(x => x.Address).ThenBy(x => x.Name, StringComparer.Ordinal));
            report.StaticFrameSlots.AddRange(_staticFrameSlots.OrderBy(x => x.Function, StringComparer.Ordinal).ThenBy(x => x.Address));
            report.RamAllocations.AddRange(_ramAllocations
                .OrderBy(x => x.Address)
                .ThenBy(x => x.Kind, StringComparer.Ordinal)
                .ThenBy(x => x.Name, StringComparer.Ordinal));
            // Collapse accesses with identical function, bank, operation, address, span and dynamic-target status.
            report.RamAccesses.AddRange(_ramAccesses
                .GroupBy(x => string.Format("{0}|{1}|{2}|{3}|{4}|{5}",
                    x.Function, x.Bank, x.Operation, x.Address, x.Span, x.DynamicTarget ? 1 : 0),
                    StringComparer.Ordinal)
                .Select(group => group.First())
                .OrderBy(x => x.Bank)
                .ThenBy(x => x.Function, StringComparer.Ordinal)
                .ThenBy(x => x.Address)
                .ThenBy(x => x.Operation, StringComparer.Ordinal));
            // Append recorded optimization decisions and bank placements without rerunning those passes.
            report.InlineDecisions.AddRange(_inlineDecisions.OrderBy(x => x.Caller, StringComparer.Ordinal).ThenBy(x => x.Callee, StringComparer.Ordinal));
            report.LoopLowerings.AddRange(_loopLowerings.OrderBy(x => x.Function, StringComparer.Ordinal).ThenBy(x => x.Variable, StringComparer.Ordinal));
            report.LtoRemovedFunctions.AddRange(_ltoRemovedFunctions.OrderBy(x => x.Name, StringComparer.Ordinal));
            report.BankPlacements.AddRange(_bankPlacements.OrderBy(x => x.Bank).ThenBy(x => x.Name, StringComparer.Ordinal));

            return report;
        }

        // Render aggregate sizes, alignment and field offsets in a stable human-readable layout listing.
        public string BuildAggregateLayoutText()
        {
            var sb = new System.Text.StringBuilder();

            foreach (var kv in _aggregates.OrderBy(x => x.Key, StringComparer.Ordinal))
            {
                var info = kv.Value;
                sb.Append(kv.Key);
                sb.Append(" ");
                sb.Append(info.Layout == AggregateLayout.Union ? "union" : "struct");
                sb.Append(" size=");
                sb.Append(info.TotalSize);
                sb.Append(" align=");
                sb.Append(info.Alignment);
                if (info.IsPacked)
                    sb.Append(" packed");
                sb.AppendLine();

                foreach (var field in info.Fields ?? Array.Empty<FieldInfo>())
                {
                    sb.Append("  ");
                    sb.Append(field.Name);
                    sb.Append(" @");
                    sb.Append(field.Offset);
                    sb.Append(" size=");
                    sb.Append(GetStorageSize(field.Type));
                    sb.Append(" ");
                    sb.AppendLine(field.Type == null ? "<null>" : field.Type.Show());
                }

                sb.AppendLine();
            }

            return sb.ToString();
        }

        // Use zero bytes for void, two for pointers, exact aggregate storage and normalized scalar widths in ABI reports.
        int SizeOfTypeLoose(CType type)
        {
            if (type == null || type == CType.Void) return 0;
            if (type.IsPointer) return 2;
            if (IsAggregateType(type)) return GetStorageSize(type);
            return NormalizeScalarSize(GetStorageSize(type));
        }

        // Classify AST tags that can produce a value; this is not a purity check and includes calls and assignments.
        static bool IsValueExpression(Expr expr)
        {
            if (expr == null) return false;
            string tag = expr.Tag;
            return tag == Tag.Integer || tag == Tag.Name || tag == Tag.AddressOf || tag == Tag.Load || tag == Tag.Index || tag == Tag.Field ||
                   tag == Tag.Sizeof || tag == Tag.Offsetof ||
                   tag == Tag.Add || tag == Tag.Subtract || tag == Tag.BitwiseAnd || tag == Tag.BitwiseOr ||
                   tag == Tag.BitwiseXor || tag == Tag.ShiftLeft || tag == Tag.ShiftRight ||
                   tag == Tag.LogicalNot || tag == Tag.LogicalAnd || tag == Tag.LogicalOr ||
                   tag == Tag.BitwiseNot || IsComparisonTag(tag) || tag == Tag.Cast || tag == Tag.Call ||
                   tag == Tag.PreIncrement || tag == Tag.PostIncrement || tag == Tag.PreDecrement ||
                   tag == Tag.PostDecrement || tag == Tag.Assign || tag == Tag.AssignModify;
        }

        // Keep aggregate storage sizes while limiting ordinary scalar values to one or two bytes.
        int TypeStorageSize(CType type)
        {
            if (type == null || type == CType.Void) return 0;
            int size = GetStorageSize(type);
            if (IsAggregateType(type)) return size;
            return NormalizeScalarSize(size);
        }

        // Map storage widths to the byte/word scalar register convention.
        static int NormalizeScalarSize(int size)
        {
            return size >= 2 ? 2 : 1;
        }

        // Read the type signedness flag, treating an absent type as unsigned.
        static bool IsSignedIntegerType(CType type)
        {
            return type != null && type.IsSigned;
        }

        // Interpret the low byte as a signed two-complement value in the host integer type.
        static int SignExtend8(int value)
        {
            value &= 0xFF;
            return (value & 0x80) != 0 ? value - 0x100 : value;
        }

        // Interpret the low word as a signed two-complement value in the host integer type.
        static int SignExtend16(int value)
        {
            value &= 0xFFFF;
            return (value & 0x8000) != 0 ? value - 0x10000 : value;
        }

        // Mask lookup constants to a target word before applying the selected signed interpretation.
        static int InterpretArithmeticLookupConstant(int constantValue, bool signedArithmetic)
        {
            int bits = constantValue & 0xFFFF;
            return signedArithmetic ? SignExtend16(bits) : bits;
        }

        // Accept integer and enumeration types for arithmetic promotion.
        static bool IsIntegerLike(CType type)
        {
            return type != null && (type.IsInteger || type.IsEnum);
        }

        // Promote to a word, selecting signed arithmetic when either operand type is signed.
        static CType PromoteIntegerBinaryType(CType leftType, CType rightType)
        {
            bool hasSigned = IsSignedIntegerType(leftType) || IsSignedIntegerType(rightType);
            return hasSigned ? CType.Int16 : CType.UInt16;
        }

        // Resolve an expression in its function context before checking its signedness.
        bool IsSignedArithmeticOperand(Expr expr, FunctionContext ctx)
        {
            if (expr == null) return false;
            CType type;
            return TryGetExprType(expr, ctx, out type) && IsSignedIntegerType(type);
        }

        // Reject known noninteger operands, then apply this compiler's binary signedness promotion rule.
        bool ShouldUseSignedArithmetic(Expr left, Expr right, FunctionContext ctx)
        {
            CType leftType;
            CType rightType;
            bool haveLeft = TryGetExprType(left, ctx, out leftType);
            bool haveRight = TryGetExprType(right, ctx, out rightType);
            if ((haveLeft && !IsIntegerLike(leftType)) || (haveRight && !IsIntegerLike(rightType)))
                return false;
            return IsSignedIntegerType(PromoteIntegerBinaryType(leftType, rightType));
        }

        // Choose a byte only when the masked target word fits without a nonzero high byte.
        static int ConstantStorageSize(int value)
        {
            int v = value & 0xFFFF;
            return v <= 0x00FF ? 1 : 2;
        }

        // Query declared function metadata; missing names do not imply a fastcall convention.
        bool IsFunctionFastCall(string funcName)
        {
            if (string.IsNullOrEmpty(funcName)) return false;
            CFunctionInfo info;
            return _functionInfos.TryGetValue(funcName, out info) && info != null && info.IsFastCall;
        }

        // Resolve intrinsic or declared scalar return widths; aggregates use separate storage and unknown functions default to a byte.
        int GetFunctionReturnSize(string funcName)
        {
            if (string.Equals(funcName, "__bankof", StringComparison.Ordinal)) return 1;
            if (string.Equals(funcName, "__bankswitch", StringComparison.Ordinal)) return 0;
            if (string.Equals(funcName, "__assert", StringComparison.Ordinal)) return 0;
            if (TryGetCompatibilityIntrinsicSignature(funcName, out IntrinsicSignature signature))
                return TypeStorageSize(signature.ReturnType);
            CType retType;
            if (_functionReturnTypes.TryGetValue(funcName, out retType))
            {
                if (IsAggregateType(retType)) return 0;
                return TypeStorageSize(retType);
            }
            return 1;
        }

        // Return the complementary comparison for branch inversion, leaving unrelated tags unchanged.
        static string InvertComparisonTag(string tag)
        {
            if (tag == Tag.Equal) return Tag.NotEqual;
            if (tag == Tag.NotEqual) return Tag.Equal;
            if (tag == Tag.LessThan) return Tag.GreaterThanOrEqual;
            if (tag == Tag.GreaterThanOrEqual) return Tag.LessThan;
            if (tag == Tag.LessThanOrEqual) return Tag.GreaterThan;
            if (tag == Tag.GreaterThan) return Tag.LessThanOrEqual;
            return tag;
        }

        // Store the low byte from A and, for a word slot, the high byte from X.
        void StoreToSlot(StorageSlot slot)
        {
            EmitAsm("STA", Mem(slot.Address));
            if (slot.Size == 2)
                EmitAsm("STX", Mem(slot.Address + 1));
        }

        // Load A and optionally X from a byte/word slot; byte loads leave X unchanged.
        void LoadFromSlot(StorageSlot slot)
        {
            EmitAsm("LDA", Mem(slot.Address));
            if (slot.Size == 2)
                EmitAsm("LDX", Mem(slot.Address + 1));
        }

        // Give emitted internal labels a monotonically increasing suffix within this generator.
        string NewGeneratedLabel(string prefix)
        {
            _labelCounter++;
            return string.Format("__kq_{0}_{1}", prefix, _labelCounter);
        }

        // Append an implicit-operand instruction after canonicalizing its mnemonic.
        void EmitAsm(string mnemonic)
        {
            _assembly.Add(Expr.Make(Tag.Asm, NormalizeMnemonic(mnemonic), AsmOperand.Implicit));
        }

        // Resolve known absolute operands and record RAM access metadata before appending the instruction.
        void EmitAsm(string mnemonic, AsmOperand operand)
        {
            string normalizedMnemonic = NormalizeMnemonic(mnemonic);
            AsmOperand normalizedOperand = NormalizeAsmOperandForEmission(operand, null);
            RecordRamAccess(normalizedMnemonic, normalizedOperand);
            _assembly.Add(Expr.Make(Tag.Asm, normalizedMnemonic, normalizedOperand));
        }

        // Resolve local storage before globals and constants, preserving addressing mode, modifiers and comments; leave other symbols for assembly.
        AsmOperand NormalizeAsmOperandForEmission(AsmOperand operand, FunctionContext ctx)
        {
            operand = operand ?? AsmOperand.Implicit;
            if (!operand.Base.HasValue)
                return operand;

            string name = operand.Base.Value;
            StorageSlot slot;
            if (ctx != null && ctx.Locals.TryGetValue(name, out slot) && slot != null)
            {
                return new AsmOperand(Maybe.Nothing, slot.Address + operand.Offset, operand.Mode, operand.Modifier, operand.Comment);
            }

            if (_globals.TryGetValue(name, out slot) && slot != null)
            {
                return new AsmOperand(Maybe.Nothing, slot.Address + operand.Offset, operand.Mode, operand.Modifier, operand.Comment);
            }

            if (_constants.TryGetValue(operand.Base.Value, out int constantValue))
            {
                return new AsmOperand(Maybe.Nothing, constantValue + operand.Offset, operand.Mode, operand.Modifier, operand.Comment);
            }

            return operand;
        }

        // Trim mnemonic text and canonicalize it without culture-dependent casing.
        static string NormalizeMnemonic(string mnemonic)
        {
            return (mnemonic ?? "").Trim().ToUpperInvariant();
        }

        // Construct byte immediates, word addresses and symbolic operands in the addressing modes used by the emitters.
        static AsmOperand Imm(int value) { return new AsmOperand(value & 0xFF, AddressMode.Immediate); }
        static AsmOperand ImmLo(string label) { return new AsmOperand(label, ImmediateModifier.LowByte); }
        static AsmOperand ImmHi(string label) { return new AsmOperand(label, ImmediateModifier.HighByte); }
        static AsmOperand Mem(int address) { return new AsmOperand(address & 0xFFFF, AddressMode.Absolute); }
        static AsmOperand Abs(string label) { return new AsmOperand(label, AddressMode.Absolute); }
        static AsmOperand AbsY(string label) { return new AsmOperand(label, AddressMode.AbsoluteY); }
        static AsmOperand Rel(string label) { return new AsmOperand(label, AddressMode.Relative); }
        static AsmOperand IndY(int zeroPageAddress) { return new AsmOperand(zeroPageAddress, AddressMode.IndirectY); }
    }
}
