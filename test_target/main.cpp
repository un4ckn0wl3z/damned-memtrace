// Test Target Program for Memory Traversal Tool
// This program creates a game-like structure with pointers for testing

#include <windows.h>
#include <iostream>
#include <cstdint>
#include <thread>
#include <chrono>

// Game structures with pointer chains
struct Vector3 {
    float x;
    float y;
    float z;
};

struct Stats {
    int32_t health;      // offset 0x00
    int32_t maxHealth;   // offset 0x04
    int32_t mana;        // offset 0x08
    int32_t maxMana;     // offset 0x0C
    int32_t stamina;     // offset 0x10
    int32_t level;       // offset 0x14
    int64_t experience;  // offset 0x18
    float attackPower;   // offset 0x20
    float defense;       // offset 0x24
};

struct Inventory {
    int32_t gold;        // offset 0x00
    int32_t itemCount;   // offset 0x04
    int32_t slots[10];   // offset 0x08 - item IDs
};

struct Player {
    char name[32];       // offset 0x00
    Stats* stats;        // offset 0x20 (pointer)
    Vector3* position;   // offset 0x28 (pointer)
    Inventory* inventory;// offset 0x30 (pointer)
    int32_t playerId;    // offset 0x38
    bool isAlive;        // offset 0x3C
};

struct GameWorld {
    char mapName[64];    // offset 0x00
    Player* localPlayer; // offset 0x40 (pointer)
    int32_t playerCount; // offset 0x48
    float gameTime;      // offset 0x4C
};

// Global game instance (this will be our base)
GameWorld* g_pGameWorld = nullptr;

void PrintAddresses() {
    std::cout << "\n========== MEMORY ADDRESSES FOR TESTING ==========\n\n";
    
    std::cout << "Base Address (g_pGameWorld pointer location):\n";
    std::cout << "  &g_pGameWorld = 0x" << std::hex << (uintptr_t)&g_pGameWorld << std::dec << "\n\n";
    
    std::cout << "GameWorld Structure:\n";
    std::cout << "  g_pGameWorld = 0x" << std::hex << (uintptr_t)g_pGameWorld << std::dec << "\n";
    std::cout << "  ->mapName = \"" << g_pGameWorld->mapName << "\"\n";
    std::cout << "  ->localPlayer (offset 0x40) = 0x" << std::hex << (uintptr_t)g_pGameWorld->localPlayer << std::dec << "\n";
    std::cout << "  ->playerCount = " << g_pGameWorld->playerCount << "\n";
    std::cout << "  ->gameTime = " << g_pGameWorld->gameTime << "\n\n";
    
    Player* player = g_pGameWorld->localPlayer;
    std::cout << "Player Structure:\n";
    std::cout << "  player->name = \"" << player->name << "\"\n";
    std::cout << "  player->stats (offset 0x20) = 0x" << std::hex << (uintptr_t)player->stats << std::dec << "\n";
    std::cout << "  player->position (offset 0x28) = 0x" << std::hex << (uintptr_t)player->position << std::dec << "\n";
    std::cout << "  player->inventory (offset 0x30) = 0x" << std::hex << (uintptr_t)player->inventory << std::dec << "\n";
    std::cout << "  player->playerId = " << player->playerId << "\n\n";
    
    Stats* stats = player->stats;
    std::cout << "Stats Structure (at 0x" << std::hex << (uintptr_t)stats << std::dec << "):\n";
    std::cout << "  ->health (offset 0x00) = " << stats->health << "\n";
    std::cout << "  ->maxHealth (offset 0x04) = " << stats->maxHealth << "\n";
    std::cout << "  ->mana (offset 0x08) = " << stats->mana << "\n";
    std::cout << "  ->maxMana (offset 0x0C) = " << stats->maxMana << "\n";
    std::cout << "  ->stamina (offset 0x10) = " << stats->stamina << "\n";
    std::cout << "  ->level (offset 0x14) = " << stats->level << "\n";
    std::cout << "  ->experience (offset 0x18) = " << stats->experience << "\n";
    std::cout << "  ->attackPower (offset 0x20) = " << stats->attackPower << "\n";
    std::cout << "  ->defense (offset 0x24) = " << stats->defense << "\n\n";
    
    Vector3* pos = player->position;
    std::cout << "Position Structure (at 0x" << std::hex << (uintptr_t)pos << std::dec << "):\n";
    std::cout << "  ->x (offset 0x00) = " << pos->x << "\n";
    std::cout << "  ->y (offset 0x04) = " << pos->y << "\n";
    std::cout << "  ->z (offset 0x08) = " << pos->z << "\n\n";
    
    Inventory* inv = player->inventory;
    std::cout << "Inventory Structure (at 0x" << std::hex << (uintptr_t)inv << std::dec << "):\n";
    std::cout << "  ->gold (offset 0x00) = " << inv->gold << "\n";
    std::cout << "  ->itemCount (offset 0x04) = " << inv->itemCount << "\n\n";
    
    std::cout << "========== POINTER CHAINS FOR TESTING ==========\n\n";
    std::cout << "To find Player Health:\n";
    std::cout << "  Base: 0x" << std::hex << (uintptr_t)&g_pGameWorld << std::dec << "\n";
    std::cout << "  Offsets: 0x0 + 0x40 + 0x20 + 0x0\n";
    std::cout << "  Chain: [[[base]+0x0]+0x40]+0x20]+0x0 = health\n\n";
    
    std::cout << "To find Player Mana:\n";
    std::cout << "  Base: 0x" << std::hex << (uintptr_t)&g_pGameWorld << std::dec << "\n";
    std::cout << "  Offsets: 0x0 + 0x40 + 0x20 + 0x8\n";
    std::cout << "  Chain: [[[base]+0x0]+0x40]+0x20]+0x8 = mana\n\n";
    
    std::cout << "To find Player Position X:\n";
    std::cout << "  Base: 0x" << std::hex << (uintptr_t)&g_pGameWorld << std::dec << "\n";
    std::cout << "  Offsets: 0x0 + 0x40 + 0x28 + 0x0\n";
    std::cout << "  Chain: [[[base]+0x0]+0x40]+0x28]+0x0 = position.x\n\n";
    
    std::cout << "To find Gold:\n";
    std::cout << "  Base: 0x" << std::hex << (uintptr_t)&g_pGameWorld << std::dec << "\n";
    std::cout << "  Offsets: 0x0 + 0x40 + 0x30 + 0x0\n";
    std::cout << "  Chain: [[[base]+0x0]+0x40]+0x30]+0x0 = gold\n\n";
    
    std::cout << "===================================================\n";
}

void SimulateGame() {
    int tick = 0;
    while (true) {
        // Simulate game updates
        Stats* stats = g_pGameWorld->localPlayer->stats;
        Vector3* pos = g_pGameWorld->localPlayer->position;
        
        // Health regeneration
        if (stats->health < stats->maxHealth) {
            stats->health += 1;
        }
        
        // Mana regeneration
        if (stats->mana < stats->maxMana) {
            stats->mana += 2;
        }
        
        // Movement simulation
        pos->x += 0.1f;
        pos->y += 0.05f;
        
        // Game time
        g_pGameWorld->gameTime += 0.016f;
        
        // Experience gain
        if (tick % 100 == 0) {
            stats->experience += 10;
        }
        
        tick++;
        std::this_thread::sleep_for(std::chrono::milliseconds(16)); // ~60 FPS
    }
}

int main() {
    std::cout << "=== Test Target Program for Memory Traversal Tool ===\n";
    std::cout << "Process ID: " << GetCurrentProcessId() << "\n\n";
    
    // Allocate game structures
    g_pGameWorld = new GameWorld();
    strcpy_s(g_pGameWorld->mapName, "DarkForest_01");
    g_pGameWorld->playerCount = 1;
    g_pGameWorld->gameTime = 0.0f;
    
    // Create player
    Player* player = new Player();
    strcpy_s(player->name, "TestHero");
    player->playerId = 12345;
    player->isAlive = true;
    g_pGameWorld->localPlayer = player;
    
    // Create stats
    Stats* stats = new Stats();
    stats->health = 85;
    stats->maxHealth = 100;
    stats->mana = 50;
    stats->maxMana = 100;
    stats->stamina = 75;
    stats->level = 42;
    stats->experience = 123456789;
    stats->attackPower = 156.5f;
    stats->defense = 89.3f;
    player->stats = stats;
    
    // Create position
    Vector3* position = new Vector3();
    position->x = 100.5f;
    position->y = 200.25f;
    position->z = 50.0f;
    player->position = position;
    
    // Create inventory
    Inventory* inventory = new Inventory();
    inventory->gold = 99999;
    inventory->itemCount = 5;
    inventory->slots[0] = 1001; // Sword
    inventory->slots[1] = 2001; // Shield
    inventory->slots[2] = 3001; // Potion
    inventory->slots[3] = 4001; // Ring
    inventory->slots[4] = 5001; // Amulet
    player->inventory = inventory;
    
    // Print addresses for testing
    PrintAddresses();
    
    std::cout << "\nProgram is running. Values will update every frame.\n";
    std::cout << "Press Ctrl+C to exit.\n\n";
    
    // Start game simulation in background
    std::thread gameThread(SimulateGame);
    gameThread.detach();
    
    // Keep program running and periodically print values
    while (true) {
        std::cout << "\r[HP: " << stats->health << "/" << stats->maxHealth 
                  << " | MP: " << stats->mana << "/" << stats->maxMana
                  << " | Gold: " << inventory->gold
                  << " | Pos: (" << position->x << ", " << position->y << ", " << position->z << ")"
                  << " | EXP: " << stats->experience << "]     " << std::flush;
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
    }
    
    // Cleanup (never reached)
    delete inventory;
    delete position;
    delete stats;
    delete player;
    delete g_pGameWorld;
    
    return 0;
}
