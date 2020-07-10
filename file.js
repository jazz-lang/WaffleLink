function foo(x,y,z) {
   if (x + y == 182200) {
      throw "Heeey!"
   }
   print(x + y);
   return x + y + z;
}

for (let i = 0;i<100001;i++) {
   foo(i,"x" + i,[] + i);
}
