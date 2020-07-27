var startTime, endTime;

function start() {
  startTime = new Date();
};

function end() {
  endTime = new Date();
  var timeDiff = endTime - startTime; //in ms

  print(timeDiff + " ms");
}
start();
var i = 0
while (i < 100000) {
    i = i + 1
}
print("Result is: ",i)
end();