/*
 * Windows loads DirectML and Direct3D 12 on first use, not at program start
 * (/DELAYLOAD in build.rs), so Mash runs on a Windows without them: Windows
 * Server 2019 and Windows 10 before version 1903 have no directml.dll, and
 * the one in versions 1903 and 1909 lacks DMLCreateDevice1.
 *
 * A delayed load that fails raises a structured exception, which would end
 * the process. This hook is the linker's documented alternative: it is asked
 * what to call in place of a function that could not be found, and it
 * answers with one that reports failure the way the real one would. ONNX
 * Runtime then says the GPU is unavailable, and Mash uses the CPU as it does
 * on a machine without a usable graphics card.
 *
 * Only the functions ONNX Runtime imports are answered. Any other failure
 * keeps the default behaviour, and .github/scripts/check-windows-delay-load.py
 * fails the build when the imports change, so a new one is looked at here.
 */
#include <windows.h>

#include <delayimp.h>
#include <string.h>

#define UNAVAILABLE HRESULT_FROM_WIN32(ERROR_MOD_NOT_FOUND)

/* D3D12CreateDevice */
static HRESULT WINAPI create_d3d12_device(void *adapter, int feature_level, const GUID *interface_id,
                                          void **device) {
    (void)adapter;
    (void)feature_level;
    (void)interface_id;
    if (device != NULL) {
        *device = NULL;
    }
    return UNAVAILABLE;
}

/* D3D12SerializeVersionedRootSignature */
static HRESULT WINAPI serialize_root_signature(const void *signature, void **blob, void **error_blob) {
    (void)signature;
    if (blob != NULL) {
        *blob = NULL;
    }
    if (error_blob != NULL) {
        *error_blob = NULL;
    }
    return UNAVAILABLE;
}

/* DMLCreateDevice1 */
static HRESULT WINAPI create_dml_device(void *d3d12_device, int flags, int feature_level,
                                        const GUID *interface_id, void **device) {
    (void)d3d12_device;
    (void)flags;
    (void)feature_level;
    (void)interface_id;
    if (device != NULL) {
        *device = NULL;
    }
    return UNAVAILABLE;
}

/*
 * Whether the failed import is this function. The Windows SDK's import
 * libraries name some functions and give others by the number their library
 * fixes for them (101 and 2 below); zero is for one that is always named.
 */
static int imports(const DelayLoadProc *import, const char *name, DWORD ordinal) {
    if (import->fImportByName) {
        return strcmp(import->szProcName, name) == 0;
    }
    return ordinal != 0 && import->dwOrdinal == ordinal;
}

static FARPROC WINAPI delay_load_failed(unsigned notification, PDelayLoadInfo info) {
    int direct3d = _stricmp(info->szDll, "d3d12.dll") == 0;
    int directml = _stricmp(info->szDll, "directml.dll") == 0;
    if (!direct3d && !directml) {
        return NULL;
    }
    if (notification == dliFailLoadLib) {
        /*
         * The library is missing or cannot be loaded. The loader accepts
         * another module in its place and looks the function up there; this
         * program exports none of them, so each lookup comes back here as
         * dliFailGetProc below, as it does for an old library without it.
         */
        return (FARPROC)GetModuleHandleW(NULL);
    }
    if (notification != dliFailGetProc) {
        return NULL;
    }
    if (direct3d && imports(&info->dlp, "D3D12CreateDevice", 101)) {
        return (FARPROC)create_d3d12_device;
    }
    if (direct3d && imports(&info->dlp, "D3D12SerializeVersionedRootSignature", 0)) {
        return (FARPROC)serialize_root_signature;
    }
    if (directml && imports(&info->dlp, "DMLCreateDevice1", 2)) {
        return (FARPROC)create_dml_device;
    }
    return NULL;
}

const PfnDliHook __pfnDliFailureHook2 = delay_load_failed;
