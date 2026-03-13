// Example Game Trainer
// Generated from Damned Memory Traversal Tool export
// Compile: g++ -o trainer.exe trainer.cpp -std=c++17

#include <iostream>
#include <string>
#include <Windows.h>
#include <TlHelp32.h>
#include "GameStruct.h"

// Find process ID by name
DWORD GetProcessIdByName(const wchar_t* processName) {
    DWORD pid = 0;
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snapshot != INVALID_HANDLE_VALUE) {
        PROCESSENTRY32W pe;
        pe.dwSize = sizeof(pe);
        if (Process32FirstW(snapshot, &pe)) {
            do {
                if (_wcsicmp(pe.szExeFile, processName) == 0) {
                    pid = pe.th32ProcessID;
                    break;
                }
            } while (Process32NextW(snapshot, &pe));
        }
        CloseHandle(snapshot);
    }
    return pid;
}

// Get module base address
uintptr_t GetModuleBaseAddress(DWORD pid, const wchar_t* moduleName) {
    uintptr_t baseAddr = 0;
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
    if (snapshot != INVALID_HANDLE_VALUE) {
        MODULEENTRY32W me;
        me.dwSize = sizeof(me);
        if (Module32FirstW(snapshot, &me)) {
            do {
                if (_wcsicmp(me.szModule, moduleName) == 0) {
                    baseAddr = (uintptr_t)me.modBaseAddr;
                    break;
                }
            } while (Module32NextW(snapshot, &me));
        }
        CloseHandle(snapshot);
    }
    return baseAddr;
}

void PrintMenu() {
    std::cout << "\n========== GAME TRAINER ==========\n";
    std::cout << "[1] Show current stats\n";
    std::cout << "[2] Set Health to Max\n";
    std::cout << "[3] Set MP to Max\n";
    std::cout << "[4] God Mode (Max Health + MP)\n";
    std::cout << "[5] Set custom Health\n";
    std::cout << "[6] Set custom MP\n";
    std::cout << "[0] Exit\n";
    std::cout << "===================================\n";
    std::cout << "Choice: ";
}

int main() {
    std::cout << "=== Example Game Trainer ===\n";
    std::cout << "Looking for target process...\n";
    
    // Change this to your target process name
    const wchar_t* targetProcess = L"test_target.exe";
    
    DWORD pid = GetProcessIdByName(targetProcess);
    if (pid == 0) {
        std::cout << "Process not found! Make sure the game is running.\n";
        std::cout << "Press Enter to exit...";
        std::cin.get();
        return 1;
    }
    
    std::cout << "Found process: PID = " << pid << "\n";
    
    // Open process with read/write access
    HANDLE hProcess = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
    if (hProcess == NULL) {
        std::cout << "Failed to open process! Run as Administrator.\n";
        std::cout << "Press Enter to exit...";
        std::cin.get();
        return 1;
    }
    
    // Get module base address
    uintptr_t moduleBase = GetModuleBaseAddress(pid, targetProcess);
    if (moduleBase == 0) {
        std::cout << "Failed to get module base address!\n";
        CloseHandle(hProcess);
        std::cout << "Press Enter to exit...";
        std::cin.get();
        return 1;
    }
    
    std::cout << "Module base: 0x" << std::hex << moduleBase << std::dec << "\n";
    std::cout << "Base offset: 0x" << std::hex << BASE_OFFSET << std::dec << "\n";
    
    // Create the game struct reader
    GameStructReader game(hProcess, moduleBase);
    
    int choice;
    bool running = true;
    
    while (running) {
        PrintMenu();
        std::cin >> choice;
        
        switch (choice) {
            case 1: {
                std::cout << "\n--- Current Stats ---\n";
                std::cout << "Health:     " << game.getHealth() << " / " << game.getMaxHealth() << "\n";
                std::cout << "MP:         " << game.getMp() << " / " << game.getMaxMp() << "\n";
                break;
            }
            case 2: {
                int maxHp = game.getMaxHealth();
                if (game.setHealth(maxHp)) {
                    std::cout << "Health set to " << maxHp << "!\n";
                } else {
                    std::cout << "Failed to write health!\n";
                }
                break;
            }
            case 3: {
                int maxMp = game.getMaxMp();
                if (game.setMp(maxMp)) {
                    std::cout << "MP set to " << maxMp << "!\n";
                } else {
                    std::cout << "Failed to write MP!\n";
                }
                break;
            }
            case 4: {
                int maxHp = game.getMaxHealth();
                int maxMp = game.getMaxMp();
                game.setHealth(maxHp);
                game.setMp(maxMp);
                std::cout << "God Mode activated! HP=" << maxHp << ", MP=" << maxMp << "\n";
                break;
            }
            case 5: {
                int newHealth;
                std::cout << "Enter new Health value: ";
                std::cin >> newHealth;
                if (game.setHealth(newHealth)) {
                    std::cout << "Health set to " << newHealth << "!\n";
                } else {
                    std::cout << "Failed to write health!\n";
                }
                break;
            }
            case 6: {
                int newMp;
                std::cout << "Enter new MP value: ";
                std::cin >> newMp;
                if (game.setMp(newMp)) {
                    std::cout << "MP set to " << newMp << "!\n";
                } else {
                    std::cout << "Failed to write MP!\n";
                }
                break;
            }
            case 0:
                running = false;
                std::cout << "Exiting trainer...\n";
                break;
            default:
                std::cout << "Invalid choice!\n";
                break;
        }
    }
    
    CloseHandle(hProcess);
    return 0;
}
