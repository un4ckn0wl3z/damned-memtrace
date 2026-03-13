// Entity List Test Target
// Demonstrates 3 types of entity lists for testing the Entity List Scanner
// Compile: g++ -o entity_list_test.exe entity_list_test.cpp -std=c++17

#include <iostream>
#include <cstdint>
#include <thread>
#include <chrono>
#include <vector>

// ============================================
// TYPE 1: Array of Structs (Contiguous Array)
// ============================================
struct Entity {
    char padding[0x100];  // Padding to offset 0x100
    int32_t hp;           // Offset 0x100
    int32_t mp;           // Offset 0x104
    int32_t level;        // Offset 0x108
    float posX;           // Offset 0x10C
    float posY;           // Offset 0x110
    float posZ;           // Offset 0x114
};

// Array of entities (contiguous in memory)
Entity g_EntityArray[10];

// ============================================
// TYPE 2: Pointer Table (Array of Pointers)
// ============================================
Entity* g_EntityPointerTable[10];
Entity g_EntityStorage[10];  // Actual entity storage

// ============================================
// TYPE 3: Linked List
// ============================================
struct LinkedEntity {
    LinkedEntity* pNext;  // Offset 0x0 - pointer to next node
    char padding[0xF8];   // Padding to offset 0x100
    int32_t hp;           // Offset 0x100
    int32_t mp;           // Offset 0x104
    int32_t level;        // Offset 0x108
};

LinkedEntity* g_LinkedListHead = nullptr;
std::vector<LinkedEntity*> g_LinkedNodes;  // To prevent memory leaks

void initializeEntities() {
    // Initialize TYPE 1: Array of Structs
    std::cout << "\n=== TYPE 1: Array of Structs ===" << std::endl;
    std::cout << "Base Address: 0x" << std::hex << (uintptr_t)&g_EntityArray << std::dec << std::endl;
    std::cout << "Struct Size: 0x" << std::hex << sizeof(Entity) << std::dec << " (" << sizeof(Entity) << " bytes)" << std::endl;
    std::cout << "Value Offsets: 0x100 (HP), 0x104 (MP), 0x108 (Level)" << std::endl;
    
    for (int i = 0; i < 10; i++) {
        g_EntityArray[i].hp = 100 + i * 10;      // 100, 110, 120...
        g_EntityArray[i].mp = 50 + i * 5;        // 50, 55, 60...
        g_EntityArray[i].level = i + 1;          // 1, 2, 3...
        g_EntityArray[i].posX = 100.0f + i;
        g_EntityArray[i].posY = 200.0f + i;
        g_EntityArray[i].posZ = 0.0f;
    }
    
    // Initialize TYPE 2: Pointer Table
    std::cout << "\n=== TYPE 2: Pointer Table ===" << std::endl;
    std::cout << "Base Address: 0x" << std::hex << (uintptr_t)&g_EntityPointerTable << std::dec << std::endl;
    std::cout << "Pointer Size: 8 bytes (x64)" << std::endl;
    std::cout << "Value Offsets: 0x100 (HP), 0x104 (MP), 0x108 (Level)" << std::endl;
    
    for (int i = 0; i < 10; i++) {
        g_EntityStorage[i].hp = 200 + i * 20;    // 200, 220, 240...
        g_EntityStorage[i].mp = 100 + i * 10;    // 100, 110, 120...
        g_EntityStorage[i].level = (i + 1) * 5;  // 5, 10, 15...
        g_EntityPointerTable[i] = &g_EntityStorage[i];
    }
    // Set some pointers to nullptr to simulate empty slots
    g_EntityPointerTable[3] = nullptr;
    g_EntityPointerTable[7] = nullptr;
    
    // Initialize TYPE 3: Linked List
    std::cout << "\n=== TYPE 3: Linked List ===" << std::endl;
    
    LinkedEntity* prev = nullptr;
    for (int i = 0; i < 5; i++) {
        LinkedEntity* node = new LinkedEntity();
        g_LinkedNodes.push_back(node);  // Track for cleanup
        
        node->hp = 300 + i * 30;         // 300, 330, 360...
        node->mp = 150 + i * 15;         // 150, 165, 180...
        node->level = (i + 1) * 10;      // 10, 20, 30...
        node->pNext = nullptr;
        
        if (prev) {
            prev->pNext = node;
        } else {
            g_LinkedListHead = node;
        }
        prev = node;
    }
    
    std::cout << "Head Address: 0x" << std::hex << (uintptr_t)g_LinkedListHead << std::dec << std::endl;
    std::cout << "Next Ptr Offset: 0x0" << std::endl;
    std::cout << "Value Offsets: 0x100 (HP), 0x104 (MP), 0x108 (Level)" << std::endl;
}

void printCurrentValues() {
    std::cout << "\n--- Current Entity Values ---" << std::endl;
    
    std::cout << "\nArray Entities:" << std::endl;
    for (int i = 0; i < 10; i++) {
        std::cout << "  [" << i << "] HP=" << g_EntityArray[i].hp 
                  << " MP=" << g_EntityArray[i].mp 
                  << " Lv=" << g_EntityArray[i].level << std::endl;
    }
    
    std::cout << "\nPointer Table Entities:" << std::endl;
    for (int i = 0; i < 10; i++) {
        if (g_EntityPointerTable[i]) {
            std::cout << "  [" << i << "] HP=" << g_EntityPointerTable[i]->hp 
                      << " MP=" << g_EntityPointerTable[i]->mp 
                      << " Lv=" << g_EntityPointerTable[i]->level << std::endl;
        } else {
            std::cout << "  [" << i << "] (null)" << std::endl;
        }
    }
    
    std::cout << "\nLinked List Entities:" << std::endl;
    LinkedEntity* node = g_LinkedListHead;
    int idx = 0;
    while (node) {
        std::cout << "  [" << idx << "] HP=" << node->hp 
                  << " MP=" << node->mp 
                  << " Lv=" << node->level << std::endl;
        node = node->pNext;
        idx++;
    }
}

int main() {
    std::cout << "========================================" << std::endl;
    std::cout << "   Entity List Test Target" << std::endl;
    std::cout << "========================================" << std::endl;
    
    initializeEntities();
    
    std::cout << "\n========================================" << std::endl;
    std::cout << "   SCANNER SETTINGS SUMMARY" << std::endl;
    std::cout << "========================================" << std::endl;
    
    std::cout << "\n[Array Scanner]" << std::endl;
    std::cout << "  Mode: Array Scanner" << std::endl;
    std::cout << "  Base: 0x" << std::hex << (uintptr_t)&g_EntityArray << std::dec << std::endl;
    std::cout << "  Struct Size: 0x" << std::hex << sizeof(Entity) << std::dec << std::endl;
    std::cout << "  Max Count: 10" << std::endl;
    std::cout << "  Value Offsets: 0x100,0x104,0x108" << std::endl;
    
    std::cout << "\n[Pointer Table]" << std::endl;
    std::cout << "  Mode: Pointer Table" << std::endl;
    std::cout << "  Base: 0x" << std::hex << (uintptr_t)&g_EntityPointerTable << std::dec << std::endl;
    std::cout << "  Max Count: 10" << std::endl;
    std::cout << "  Value Offsets: 0x100,0x104,0x108" << std::endl;
    
    std::cout << "\n[Linked List]" << std::endl;
    std::cout << "  Mode: Linked List" << std::endl;
    std::cout << "  Base: 0x" << std::hex << (uintptr_t)g_LinkedListHead << std::dec << std::endl;
    std::cout << "  Next Offset: 0x0" << std::endl;
    std::cout << "  Max Depth: 10" << std::endl;
    std::cout << "  Value Offsets: 0x100,0x104,0x108" << std::endl;
    
    printCurrentValues();
    
    std::cout << "\n========================================" << std::endl;
    std::cout << "Press Enter to exit..." << std::endl;
    std::cout << "========================================" << std::endl;
    
    // Keep running so memory can be scanned
    while (true) {
        std::this_thread::sleep_for(std::chrono::seconds(1));
        
        // Simulate value changes
        for (int i = 0; i < 10; i++) {
            g_EntityArray[i].hp = (g_EntityArray[i].hp % 200) + 1;
        }
    }
    
    // Cleanup
    for (auto node : g_LinkedNodes) {
        delete node;
    }
    
    return 0;
}
