#pragma once
#include <windows.h>
#include <stddef.h>
#include <cwchar>
#include <cstdint>
// Version 1 layout, identical on x86 and x64. One mapping per target process.
struct SharedSpeed {
    volatile LONG enabled;
    volatile LONG ready;
    volatile LONG factorTenths;
    LONG reserved;
    DWORD owner;
    volatile LONG hooks;
};
static_assert(offsetof(SharedSpeed, factorTenths) == 8, "protocol layout");
inline void mappingName(wchar_t* name, DWORD pid) {
    swprintf_s(name, 80, L"Local\\Xiuxian.Speed.v1.%lu", pid);
}
