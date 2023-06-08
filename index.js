const fs = require("fs");

const rows = 1000000; // 10000
const chars = 500; // 500

fs.writeFile("test.txt", "", () => { });
for (let index = 0; index < rows; index++) {
  let data = "";
  for (let j = 0; j < chars; j++) {
    data = data + "a";
  }
  data = data + "\n";
  fs.appendFile("test.txt", data, () => { });
  console.log("sup");
  console.log('asdf');
}
