# Example Game Trainer

A simple C++ game trainer example generated from Damned Memory Traversal Tool export.

## Files

- `GameStruct.h` - Header file exported from the tool with struct layout and pointer chain helper
- `trainer.cpp` - Example trainer application with menu interface

## Building

### Using g++ (MinGW)
```bash
g++ -o trainer.exe trainer.cpp -std=c++17
```

### Using MSVC
```bash
cl /EHsc trainer.cpp
```

## Usage

1. Start your target game/application first
2. Run `trainer.exe` as Administrator
3. Use the menu to:
   - View current stats (Health, MP)
   - Set Health to Max
   - Set MP to Max
   - Enable God Mode (Max HP + MP)
   - Set custom values

## Customization

1. Change `targetProcess` in `main()` to match your game's executable name
2. Modify `GameStruct.h` with your own exported offsets from the tool
3. Add more trainer features as needed

## Notes

- Run as Administrator for memory access
- The trainer uses the pointer chain from the exported header
- BASE_OFFSET is automatically calculated from your base address
- Values are read/written in real-time
