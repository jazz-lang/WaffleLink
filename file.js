function foo(x, y, z) {
   return x < y;
   return x + y + z;
}

for (let i = 0; i < 100001; i++) {
   try {
      foo(i, i + 2);
   } catch (e) {
      print(e);
   }
}
