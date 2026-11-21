#include "shared.h"
#include "MinHook.h"
#include <mmsystem.h>

static SharedSpeed* config;
static BOOL (WINAPI* realQpc)(LARGE_INTEGER*) = QueryPerformanceCounter;
static DWORD (WINAPI* realTick)() = GetTickCount;
static ULONGLONG (WINAPI* realTick64)() = GetTickCount64;
static DWORD (WINAPI* realTime)() = timeGetTime;
static double frequency;
struct Clock {
    SRWLOCK lock = SRWLOCK_INIT;
    LONGLONG last = 0;
    double value = 0;
    double factor = 1;
    double units;
};
static Clock qpcClock, tickClock, tick64Clock, timeClock;

static double advance(Clock& clock, double initial) {
    AcquireSRWLockExclusive(&clock.lock);
    LARGE_INTEGER now;
    realQpc(&now);
    if (!clock.last) clock.value = initial;
    else clock.value += double(now.QuadPart - clock.last) * clock.units * clock.factor;
    clock.last = now.QuadPart;
    clock.factor = InterlockedCompareExchange(&config->enabled, 0, 0) ? InterlockedCompareExchange(&config->factorTenths, 0, 0) / 10.0 : 1;
    double result = clock.value;
    ReleaseSRWLockExclusive(&clock.lock);
    return result;
}
static BOOL WINAPI scaledQpc(LARGE_INTEGER* value) {
    LARGE_INTEGER raw;
    realQpc(&raw);
    value->QuadPart = (LONGLONG)advance(qpcClock, double(raw.QuadPart));
    return TRUE;
}
static DWORD WINAPI scaledTick() { return (DWORD)(ULONGLONG)advance(tickClock, realTick()); }
static ULONGLONG WINAPI scaledTick64() { return (ULONGLONG)advance(tick64Clock, double(realTick64())); }
static DWORD WINAPI scaledTime() { return (DWORD)(ULONGLONG)advance(timeClock, realTime()); }

static DWORD WINAPI initialize(void*) {
    wchar_t name[80];
    mappingName(name, GetCurrentProcessId());
    HANDLE mapping = OpenFileMappingW(FILE_MAP_ALL_ACCESS, FALSE, name);
    if (!mapping) return 1;
    config = (SharedSpeed*)MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, sizeof(SharedSpeed));
    if (!config) { CloseHandle(mapping); return 2; }
    LARGE_INTEGER rate;
    QueryPerformanceFrequency(&rate);
    frequency = double(rate.QuadPart);
    qpcClock.units = 1;
    tickClock.units = tick64Clock.units = timeClock.units = 1000 / frequency;
    if (MH_Initialize() != MH_OK ||
        MH_CreateHook((LPVOID)QueryPerformanceCounter, (LPVOID)scaledQpc, (LPVOID*)&realQpc) != MH_OK ||
        MH_CreateHook((LPVOID)GetTickCount, (LPVOID)scaledTick, (LPVOID*)&realTick) != MH_OK ||
        MH_CreateHook((LPVOID)GetTickCount64, (LPVOID)scaledTick64, (LPVOID*)&realTick64) != MH_OK ||
        MH_CreateHook((LPVOID)timeGetTime, (LPVOID)scaledTime, (LPVOID*)&realTime) != MH_OK ||
        MH_EnableHook(MH_ALL_HOOKS) != MH_OK) {
        MH_Uninitialize();
        InterlockedExchange(&config->ready, -1);
        return 3;
    }
    InterlockedExchange(&config->hooks, 4);
    InterlockedExchange(&config->ready, 1);
    // Keep hooks and virtual clock offsets for monotonic time, even after disabling.
    // If the controller disappears (including crashes), restore the rate to 1.
    DWORD observed = 0;
    HANDLE owner = nullptr;
    for (;;) {
        DWORD pid = config->owner;
        if (pid != observed) {
            if (owner) CloseHandle(owner);
            owner = OpenProcess(SYNCHRONIZE, FALSE, pid);
            observed = pid;
        }
        if (!owner || WaitForSingleObject(owner, 50) != WAIT_TIMEOUT) {
            InterlockedExchange(&config->enabled, 0);
            Sleep(50);
        }
        // Advance idle clocks too, so a rate change cannot apply an old rate
        // to an arbitrarily long interval on the next call from the game.
        LARGE_INTEGER now;
        realQpc(&now);
        advance(qpcClock, double(now.QuadPart));
        advance(tickClock, realTick());
        advance(tick64Clock, double(realTick64()));
        advance(timeClock, realTime());
    }
}
BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        DisableThreadLibraryCalls(instance);
        HANDLE thread = CreateThread(nullptr, 0, initialize, nullptr, 0, nullptr);
        if (thread) CloseHandle(thread);
    }
    return TRUE;
}
