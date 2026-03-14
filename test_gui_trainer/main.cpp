// AssaultCube ImGui External Trainer
// Uses GameStructAC.h exported from Damned Memory Traversal Tool

#include <Windows.h>
#include <TlHelp32.h>
#include <d3d11.h>
#include <dwmapi.h>
#include <string>
#include <thread>
#include <chrono>

#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dwmapi.lib")

#include "imgui/imgui.h"
#include "imgui/imgui_impl_win32.h"
#include "imgui/imgui_impl_dx11.h"
#include "GameStructAC.h"

// Forward declare message handler from imgui_impl_win32.cpp
extern IMGUI_IMPL_API LRESULT ImGui_ImplWin32_WndProcHandler(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam);

// Globals
ID3D11Device* g_pd3dDevice = nullptr;
ID3D11DeviceContext* g_pd3dDeviceContext = nullptr;
IDXGISwapChain* g_pSwapChain = nullptr;
ID3D11RenderTargetView* g_mainRenderTargetView = nullptr;
HWND g_hwnd = nullptr;
HWND g_gameHwnd = nullptr;

// Trainer state
bool g_running = true;
bool g_attached = false;
HANDLE g_hProcess = nullptr;
uintptr_t g_moduleBase = 0;
GameStructReader* g_game = nullptr;

// Cheat toggles
bool g_godMode = false;
bool g_infiniteArmor = false;
bool g_infiniteAmmo = false;
bool g_infiniteGrenades = false;

// Cached values
float g_x = 0, g_y = 0, g_z = 0;
int g_health = 0;
int g_armor = 0;
int g_pistolMag = 0;
int g_pistolAmmo = 0;
int g_rifleMag = 0;
int g_rifleAmmo = 0;
int g_bomb = 0;
int g_rapidBomb = 0;

// Process helpers
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

bool AttachToGame() {
    DWORD pid = GetProcessIdByName(L"ac_client.exe");
    if (pid == 0) return false;
    
    g_hProcess = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
    if (!g_hProcess) return false;
    
    g_moduleBase = GetModuleBaseAddress(pid, L"ac_client.exe");
    if (g_moduleBase == 0) {
        CloseHandle(g_hProcess);
        g_hProcess = nullptr;
        return false;
    }
    
    g_game = new GameStructReader(g_hProcess, g_moduleBase);
    g_attached = true;
    
    // Find game window
    g_gameHwnd = FindWindowA(nullptr, "AssaultCube");
    
    return true;
}

void DetachFromGame() {
    if (g_game) {
        delete g_game;
        g_game = nullptr;
    }
    if (g_hProcess) {
        CloseHandle(g_hProcess);
        g_hProcess = nullptr;
    }
    g_attached = false;
    g_moduleBase = 0;
}

void UpdateValues() {
    if (!g_attached || !g_game) return;
    
    g_x = g_game->getX();
    g_y = g_game->getY();
    g_z = g_game->getZ();
    g_health = g_game->getHealth();
    g_armor = g_game->getArmor();
    g_pistolMag = g_game->getPistal_mag();
    g_pistolAmmo = g_game->getPistal_ammo();
    g_rifleMag = g_game->getRifle_mag();
    g_rifleAmmo = g_game->getRifle_ammo();
    g_bomb = g_game->getBomb();
    g_rapidBomb = g_game->getRapid_bomb();
}

void ApplyCheats() {
    if (!g_attached || !g_game) return;
    
    if (g_godMode) {
        g_game->setHealth(1000);
    }
    if (g_infiniteArmor) {
        g_game->setArmor(1000);
    }
    if (g_infiniteAmmo) {
        g_game->setPistal_mag(999);
        g_game->setPistal_ammo(999);
        g_game->setRifle_mag(999);
        g_game->setRifle_ammo(999);
    }
    if (g_infiniteGrenades) {
        g_game->setBomb(999);
        g_game->setRapid_bomb(999);
    }
}

// DirectX setup
bool CreateDeviceD3D(HWND hWnd) {
    DXGI_SWAP_CHAIN_DESC sd;
    ZeroMemory(&sd, sizeof(sd));
    sd.BufferCount = 2;
    sd.BufferDesc.Width = 0;
    sd.BufferDesc.Height = 0;
    sd.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    sd.BufferDesc.RefreshRate.Numerator = 60;
    sd.BufferDesc.RefreshRate.Denominator = 1;
    sd.Flags = DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH;
    sd.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    sd.OutputWindow = hWnd;
    sd.SampleDesc.Count = 1;
    sd.SampleDesc.Quality = 0;
    sd.Windowed = TRUE;
    sd.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;

    UINT createDeviceFlags = 0;
    D3D_FEATURE_LEVEL featureLevel;
    const D3D_FEATURE_LEVEL featureLevelArray[2] = { D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_10_0 };
    HRESULT res = D3D11CreateDeviceAndSwapChain(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, createDeviceFlags, 
        featureLevelArray, 2, D3D11_SDK_VERSION, &sd, &g_pSwapChain, &g_pd3dDevice, &featureLevel, &g_pd3dDeviceContext);
    if (res != S_OK) return false;

    ID3D11Texture2D* pBackBuffer;
    g_pSwapChain->GetBuffer(0, IID_PPV_ARGS(&pBackBuffer));
    g_pd3dDevice->CreateRenderTargetView(pBackBuffer, nullptr, &g_mainRenderTargetView);
    pBackBuffer->Release();

    return true;
}

void CleanupDeviceD3D() {
    if (g_mainRenderTargetView) { g_mainRenderTargetView->Release(); g_mainRenderTargetView = nullptr; }
    if (g_pSwapChain) { g_pSwapChain->Release(); g_pSwapChain = nullptr; }
    if (g_pd3dDeviceContext) { g_pd3dDeviceContext->Release(); g_pd3dDeviceContext = nullptr; }
    if (g_pd3dDevice) { g_pd3dDevice->Release(); g_pd3dDevice = nullptr; }
}

LRESULT WINAPI WndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    if (ImGui_ImplWin32_WndProcHandler(hWnd, msg, wParam, lParam))
        return true;

    switch (msg) {
    case WM_SIZE:
        if (g_pd3dDevice != nullptr && wParam != SIZE_MINIMIZED) {
            if (g_mainRenderTargetView) { g_mainRenderTargetView->Release(); g_mainRenderTargetView = nullptr; }
            g_pSwapChain->ResizeBuffers(0, (UINT)LOWORD(lParam), (UINT)HIWORD(lParam), DXGI_FORMAT_UNKNOWN, 0);
            ID3D11Texture2D* pBackBuffer;
            g_pSwapChain->GetBuffer(0, IID_PPV_ARGS(&pBackBuffer));
            g_pd3dDevice->CreateRenderTargetView(pBackBuffer, nullptr, &g_mainRenderTargetView);
            pBackBuffer->Release();
        }
        return 0;
    case WM_DESTROY:
        g_running = false;
        PostQuitMessage(0);
        return 0;
    }
    return DefWindowProc(hWnd, msg, wParam, lParam);
}

void RenderTrainerUI() {
    // Fill entire client area, no title bar (Windows has the title)
    ImGui::SetNextWindowPos(ImVec2(0, 0));
    ImGui::SetNextWindowSize(ImGui::GetIO().DisplaySize);
    ImGui::Begin("##Trainer", &g_running, 
        ImGuiWindowFlags_NoTitleBar | ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoCollapse);
    
    // Connection status
    if (g_attached) {
        ImGui::TextColored(ImVec4(0.0f, 1.0f, 0.0f, 1.0f), "Status: Connected");
        ImGui::Text("Module Base: 0x%X", (unsigned int)g_moduleBase);
        
        if (ImGui::Button("Detach")) {
            DetachFromGame();
        }
    } else {
        ImGui::TextColored(ImVec4(1.0f, 0.0f, 0.0f, 1.0f), "Status: Not Connected");
        ImGui::Text("Make sure AssaultCube is running");
        
        if (ImGui::Button("Attach to Game")) {
            AttachToGame();
        }
    }
    
    ImGui::Separator();
    
    if (g_attached) {
        // Cheats section
        ImGui::Text("Cheats");
        ImGui::Checkbox("God Mode (Infinite Health)", &g_godMode);
        ImGui::Checkbox("Infinite Armor", &g_infiniteArmor);
        ImGui::Checkbox("Infinite Ammo", &g_infiniteAmmo);
        ImGui::Checkbox("Infinite Grenades", &g_infiniteGrenades);
        
        ImGui::Separator();
        
        // Current values
        ImGui::Text("Current Values");
        ImGui::Text("Position: X=%.2f Y=%.2f Z=%.2f", g_x, g_y, g_z);
        ImGui::Text("Health: %d", g_health);
        ImGui::Text("Armor: %d", g_armor);
        
        ImGui::Separator();
        
        // Manual editors
        ImGui::Text("Manual Editors");
        
        static int newHealth = 100;
        ImGui::SliderInt("Health##edit", &newHealth, 0, 1000);
        ImGui::SameLine();
        if (ImGui::Button("Set##health")) {
            g_game->setHealth(newHealth);
        }
        
        static int newArmor = 100;
        ImGui::SliderInt("Armor##edit", &newArmor, 0, 1000);
        ImGui::SameLine();
        if (ImGui::Button("Set##armor")) {
            g_game->setArmor(newArmor);
        }
        
        ImGui::Separator();
        
        // Ammo section
        ImGui::Text("Ammo");
        ImGui::Text("Pistol: %d / %d", g_pistolMag, g_pistolAmmo);
        ImGui::Text("Rifle: %d / %d", g_rifleMag, g_rifleAmmo);
        ImGui::Text("Grenades: %d / Rapid: %d", g_bomb, g_rapidBomb);
        
        if (ImGui::Button("Refill All Ammo")) {
            g_game->setPistal_mag(999);
            g_game->setPistal_ammo(999);
            g_game->setRifle_mag(999);
            g_game->setRifle_ammo(999);
            g_game->setBomb(999);
            g_game->setRapid_bomb(999);
        }
    }
    
    ImGui::Separator();
    ImGui::TextColored(ImVec4(0.5f, 0.5f, 0.5f, 1.0f), "Generated by Damned Memory Traversal Tool");
    
    ImGui::End();
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    // Create normal window with title bar
    WNDCLASSEXW wc = { sizeof(wc), CS_CLASSDC, WndProc, 0L, 0L, GetModuleHandle(nullptr), nullptr, nullptr, nullptr, nullptr, L"ACTrainer", nullptr };
    RegisterClassExW(&wc);
    g_hwnd = CreateWindowW(wc.lpszClassName, L"AssaultCube Trainer", 
        WS_OVERLAPPEDWINDOW & ~WS_MAXIMIZEBOX & ~WS_THICKFRAME, 
        100, 100, 420, 520, nullptr, nullptr, wc.hInstance, nullptr);

    // Initialize Direct3D
    if (!CreateDeviceD3D(g_hwnd)) {
        CleanupDeviceD3D();
        UnregisterClassW(wc.lpszClassName, wc.hInstance);
        return 1;
    }

    ShowWindow(g_hwnd, SW_SHOWDEFAULT);
    UpdateWindow(g_hwnd);

    // Setup ImGui
    IMGUI_CHECKVERSION();
    ImGui::CreateContext();
    ImGuiIO& io = ImGui::GetIO();
    io.ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard;

    // Setup style
    ImGui::StyleColorsDark();
    ImGuiStyle& style = ImGui::GetStyle();
    style.WindowRounding = 5.0f;
    style.FrameRounding = 3.0f;
    style.Colors[ImGuiCol_WindowBg] = ImVec4(0.1f, 0.1f, 0.1f, 0.95f);
    style.Colors[ImGuiCol_TitleBg] = ImVec4(0.2f, 0.4f, 0.6f, 1.0f);
    style.Colors[ImGuiCol_TitleBgActive] = ImVec4(0.3f, 0.5f, 0.7f, 1.0f);
    style.Colors[ImGuiCol_Button] = ImVec4(0.2f, 0.4f, 0.6f, 1.0f);
    style.Colors[ImGuiCol_ButtonHovered] = ImVec4(0.3f, 0.5f, 0.7f, 1.0f);
    style.Colors[ImGuiCol_CheckMark] = ImVec4(0.3f, 0.8f, 0.3f, 1.0f);

    // Setup Platform/Renderer backends
    ImGui_ImplWin32_Init(g_hwnd);
    ImGui_ImplDX11_Init(g_pd3dDevice, g_pd3dDeviceContext);

    // Try to attach on startup
    AttachToGame();

    // Main loop
    MSG msg;
    ZeroMemory(&msg, sizeof(msg));
    while (g_running) {
        while (PeekMessage(&msg, nullptr, 0U, 0U, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessage(&msg);
            if (msg.message == WM_QUIT)
                g_running = false;
        }
        if (!g_running) break;

        // Update game values and apply cheats
        UpdateValues();
        ApplyCheats();

        // Start ImGui frame
        ImGui_ImplDX11_NewFrame();
        ImGui_ImplWin32_NewFrame();
        ImGui::NewFrame();

        // Render trainer UI
        RenderTrainerUI();

        // Rendering
        ImGui::Render();
        const float clear_color[4] = { 0.0f, 0.0f, 0.0f, 0.0f }; // Transparent background
        g_pd3dDeviceContext->OMSetRenderTargets(1, &g_mainRenderTargetView, nullptr);
        g_pd3dDeviceContext->ClearRenderTargetView(g_mainRenderTargetView, clear_color);
        ImGui_ImplDX11_RenderDrawData(ImGui::GetDrawData());

        g_pSwapChain->Present(1, 0); // VSync

        // Small delay to reduce CPU usage
        std::this_thread::sleep_for(std::chrono::milliseconds(16));
    }

    // Cleanup
    DetachFromGame();
    ImGui_ImplDX11_Shutdown();
    ImGui_ImplWin32_Shutdown();
    ImGui::DestroyContext();
    CleanupDeviceD3D();
    DestroyWindow(g_hwnd);
    UnregisterClassW(wc.lpszClassName, wc.hInstance);

    return 0;
}
