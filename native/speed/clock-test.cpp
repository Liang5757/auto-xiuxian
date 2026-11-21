#include <windows.h>
#include <mmsystem.h>
#include <iostream>
int main() {
    LARGE_INTEGER rate;
    QueryPerformanceFrequency(&rate);
    std::cout << GetCurrentProcessId() << std::endl;
    for (;;) {
        LARGE_INTEGER start, end;
        QueryPerformanceCounter(&start);
        DWORD tick = GetTickCount(), time = timeGetTime();
        ULONGLONG tick64 = GetTickCount64();
        Sleep(400);
        QueryPerformanceCounter(&end);
        std::cout << double(end.QuadPart - start.QuadPart) * 1000 / rate.QuadPart << " "
            << GetTickCount() - tick << " " << timeGetTime() - time << " "
            << GetTickCount64() - tick64 << std::endl;
    }
}
