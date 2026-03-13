# Test Target Program for Memory Traversal Tool

A C++ program that simulates a game with pointer chains for testing the Damned Memory Traversal Tool.

## Build

```bash
cd test_target
cmake -B build -G "Visual Studio 17 2022" -A x64
cmake --build build --config Release
```

## Run

```bash
.\build\Release\test_target.exe
```

## Structure

```
GameWorld (g_pGameWorld)
├── mapName[64]           @ offset 0x00
├── localPlayer*          @ offset 0x40  ──► Player
│                                           ├── name[32]      @ offset 0x00
│                                           ├── stats*        @ offset 0x20  ──► Stats
│                                           │                                    ├── health     @ 0x00 (int32)
│                                           │                                    ├── maxHealth  @ 0x04 (int32)
│                                           │                                    ├── mana       @ 0x08 (int32)
│                                           │                                    ├── maxMana    @ 0x0C (int32)
│                                           │                                    ├── stamina    @ 0x10 (int32)
│                                           │                                    ├── level      @ 0x14 (int32)
│                                           │                                    ├── experience @ 0x18 (int64)
│                                           │                                    ├── attackPower@ 0x20 (float)
│                                           │                                    └── defense    @ 0x24 (float)
│                                           ├── position*     @ offset 0x28  ──► Vector3
│                                           │                                    ├── x @ 0x00 (float)
│                                           │                                    ├── y @ 0x04 (float)
│                                           │                                    └── z @ 0x08 (float)
│                                           ├── inventory*    @ offset 0x30  ──► Inventory
│                                           │                                    ├── gold      @ 0x00 (int32)
│                                           │                                    ├── itemCount @ 0x04 (int32)
│                                           │                                    └── slots[10] @ 0x08 (int32[])
│                                           ├── playerId      @ offset 0x38
│                                           └── isAlive       @ offset 0x3C
├── playerCount           @ offset 0x48
└── gameTime              @ offset 0x4C
```

## Testing with Memory Traversal Tool

1. Run `test_target.exe` - it will print the base address
2. Open Damned Memory Traversal Tool
3. Select `test_target.exe` from the process list
4. Use the base address shown in the console

### Example Pointer Chains

The program prints the exact base address when it starts. Use that address.

| Target Value | Offsets to Enter | Expected Value |
|--------------|------------------|----------------|
| Health | `0x0+0x40+0x20+0x0` | 85-100 (regenerates) |
| Max Health | `0x0+0x40+0x20+0x4` | 100 |
| Mana | `0x0+0x40+0x20+0x8` | 50-100 (regenerates) |
| Max Mana | `0x0+0x40+0x20+0xC` | 100 |
| Level | `0x0+0x40+0x20+0x14` | 42 |
| Experience | `0x0+0x40+0x20+0x18` | 123456789+ (increases) |
| Position X | `0x0+0x40+0x28+0x0` | ~100.5 (float, changes) |
| Position Y | `0x0+0x40+0x28+0x4` | ~200.25 (float, changes) |
| Gold | `0x0+0x40+0x30+0x0` | 99999 |
| Item Count | `0x0+0x40+0x30+0x4` | 5 |

### How to Use

1. Copy the base address from test_target console (e.g., `0x7FF6A1B2C3D0`)
2. In the tool, paste it in "Base Address" field
3. Enter offsets: `0x0+0x40+0x20+0x0` (for health)
4. Set Loop Count: 1 (for single read) or higher to scan range
5. Click "Start Scan"
6. Check the 4-Byte column for health value (85-100)

### Scanning a Range

To find all Stats values at once:
- Base: (from console)
- Offsets: `0x0+0x40+0x20`
- Loop Count: 10
- Step Size: 4 bytes

This will show health, maxHealth, mana, maxMana, stamina, level at consecutive addresses.
