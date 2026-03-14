// AssaultCube Internal DLL Cheat
// Proves GameStruct works with direct memory access

#include <Windows.h>
#include <thread>
#include <atomic>
#include "GameStruct.h"

std::atomic<bool> g_running = true;

// Cheat toggles
bool g_godMode = false;
bool g_infiniteArmor = false;
bool g_infiniteAmmo = false;
bool g_infiniteGrenades = false;

void CheatThread() {
    // Wait for game to fully load
    Sleep(1000);
    
    // Allocate console for debug output
    AllocConsole();
    FILE* f;
    freopen_s(&f, "CONOUT$", "w", stdout);
    
    printf("[ACCheat] Internal DLL Loaded!\n");
    printf("[ACCheat] Press INSERT to toggle God Mode\n");
    printf("[ACCheat] Press HOME to toggle Infinite Ammo\n");
    printf("[ACCheat] Press DELETE to toggle Infinite Armor\n");
    printf("[ACCheat] Press END to toggle Infinite Grenades\n");
    printf("[ACCheat] Press F12 to Unload\n");
    printf("-----------------------------------\n");
    
    while (g_running) {
        // Get player struct
        GameStruct* player = GetLocalPlayer();
        
        if (player) {
            // Toggle keys
            if (GetAsyncKeyState(VK_INSERT) & 1) {
                g_godMode = !g_godMode;
                printf("[ACCheat] God Mode: %s\n", g_godMode ? "ON" : "OFF");
            }
            if (GetAsyncKeyState(VK_HOME) & 1) {
                g_infiniteAmmo = !g_infiniteAmmo;
                printf("[ACCheat] Infinite Ammo: %s\n", g_infiniteAmmo ? "ON" : "OFF");
            }
            if (GetAsyncKeyState(VK_DELETE) & 1) {
                g_infiniteArmor = !g_infiniteArmor;
                printf("[ACCheat] Infinite Armor: %s\n", g_infiniteArmor ? "ON" : "OFF");
            }
            if (GetAsyncKeyState(VK_END) & 1) {
                g_infiniteGrenades = !g_infiniteGrenades;
                printf("[ACCheat] Infinite Grenades: %s\n", g_infiniteGrenades ? "ON" : "OFF");
            }
            
            // Apply cheats using GameStruct directly
            if (g_godMode) {
                player->health = 1000;
            }
            if (g_infiniteArmor) {
                player->armor = 1000;
            }
            if (g_infiniteAmmo) {
                player->pistal_mag = 999;
                player->pistal_ammo = 999;
                player->rifle_mag = 999;
                player->rifle_ammo = 999;
            }
            if (g_infiniteGrenades) {
                player->bomb = 999;
                player->rapid_bomb = 999;
            }
        }
        
        // Unload key
        if (GetAsyncKeyState(VK_F12) & 1) {
            printf("[ACCheat] Unloading...\n");
            g_running = false;
        }
        
        Sleep(10);
    }
    
    // Cleanup
    fclose(f);
    FreeConsole();
}

BOOL APIENTRY DllMain(HMODULE hModule, DWORD reason, LPVOID lpReserved) {
    switch (reason) {
    case DLL_PROCESS_ATTACH:
        DisableThreadLibraryCalls(hModule);
        CreateThread(nullptr, 0, (LPTHREAD_START_ROUTINE)[](LPVOID param) -> DWORD {
            CheatThread();
            FreeLibraryAndExitThread((HMODULE)param, 0);
            return 0;
        }, hModule, 0, nullptr);
        break;
    case DLL_PROCESS_DETACH:
        g_running = false;
        break;
    }
    return TRUE;
}
