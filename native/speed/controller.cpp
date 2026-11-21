#include "shared.h"
#include <tlhelp32.h>
#include <iostream>
#include <string>
#include <cmath>

static SharedSpeed* config;
static HANDLE mapping;
static HANDLE game;

static void detach() {
    if (config) { InterlockedExchange(&config->enabled, 0); UnmapViewOfFile(config); }
    if (mapping) CloseHandle(mapping);
    if (game) CloseHandle(game);
    config = nullptr; mapping = nullptr; game = nullptr;
}
static uintptr_t remoteLoader(DWORD pid) {
    auto loader = (uintptr_t)GetProcAddress(GetModuleHandleW(L"kernel32.dll"), "LoadLibraryW");
    MEMORY_BASIC_INFORMATION memory;
    if (!VirtualQuery((void*)loader, &memory, sizeof(memory))) return 0;
    wchar_t localPath[MAX_PATH];
    GetModuleFileNameW((HMODULE)memory.AllocationBase, localPath, MAX_PATH);
    const wchar_t* baseName = wcsrchr(localPath, L'\\');
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
    MODULEENTRY32W entry = { sizeof(entry) };
    uintptr_t result = 0;
    if (Module32FirstW(snapshot, &entry)) do {
        if (_wcsicmp(entry.szModule, baseName ? baseName + 1 : localPath) == 0) {
            result = (uintptr_t)entry.modBaseAddr + loader - (uintptr_t)memory.AllocationBase;
            break;
        }
    } while (Module32NextW(snapshot, &entry));
    CloseHandle(snapshot);
    return result;
}
static bool inject(DWORD pid) {
    detach();
    game = OpenProcess(PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION |
        PROCESS_VM_WRITE | PROCESS_VM_READ | SYNCHRONIZE, FALSE, pid);
    if (!game) return false;
    wchar_t name[80];
    mappingName(name, pid);
    mapping = CreateFileMappingW(INVALID_HANDLE_VALUE, nullptr, PAGE_READWRITE, 0, sizeof(SharedSpeed), name);
    if (!mapping) return false;
    config = (SharedSpeed*)MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, sizeof(SharedSpeed));
    if (!config) return false;
    config->owner = GetCurrentProcessId();
    if (config->ready == 1) return true;
    wchar_t dll[MAX_PATH];
    GetModuleFileNameW(nullptr, dll, MAX_PATH);
    wchar_t* file = wcsrchr(dll, L'\\');
#ifdef _WIN64
    wcscpy_s(file + 1, MAX_PATH - (file + 1 - dll), L"xiuxian-speed64.dll");
#else
    wcscpy_s(file + 1, MAX_PATH - (file + 1 - dll), L"xiuxian-speed32.dll");
#endif
    SIZE_T size = (wcslen(dll) + 1) * sizeof(wchar_t);
    void* remote = VirtualAllocEx(game, nullptr, size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    uintptr_t loader = remoteLoader(pid);
    if (!remote || !loader) { if (remote) VirtualFreeEx(game, remote, 0, MEM_RELEASE); return false; }
    SIZE_T written = 0;
    HANDLE thread = nullptr;
    if (WriteProcessMemory(game, remote, dll, size, &written) && written == size)
        thread = CreateRemoteThread(game, nullptr, 0, (LPTHREAD_START_ROUTINE)loader, remote, 0, nullptr);
    if (!thread) { VirtualFreeEx(game, remote, 0, MEM_RELEASE); return false; }
    DWORD wait = WaitForSingleObject(thread, 5000);
    CloseHandle(thread);
    // On timeout, leave the small remote allocation intact: the loader may still read it.
    if (wait != WAIT_OBJECT_0) return false;
    VirtualFreeEx(game, remote, 0, MEM_RELEASE);
    for (int i = 0; i < 100 && config->ready == 0; ++i) Sleep(20);
    return config->ready == 1;
}
int main() {
    std::string line;
    while (std::getline(std::cin, line)) {
        try {
            if (line == "exit") break;
            if (line.rfind("inject ", 0) == 0) {
                if (!inject(std::stoul(line.substr(7)))) { std::cerr << "native injection failed\n"; detach(); return 2; }
            } else if (line.rfind("change ", 0) == 0) {
                double factor = std::stod(line.substr(7));
                if (!config || !std::isfinite(factor) || factor < 0.1 || factor > 10) return 3;
                InterlockedExchange(&config->factorTenths, (LONG)std::lround(factor * 10));
                InterlockedExchange(&config->enabled, 1);
            } else if (line.rfind("assistant_marker_", 0) == 0) {
                std::cerr << line << std::endl;
            } else return 4;
        } catch (...) { detach(); return 5; }
    }
    detach();
    return 0;
}
