f = open("embeetle_map");
content = f.read().splitlines();

print("const BEETLE_MAP: [(u8, u8); 132] = [")
for x in range(0, 32):
  for y in range(0, 32):
    if content[x][y] == "1":
      print(f"({y}, {x}),")
print("];")