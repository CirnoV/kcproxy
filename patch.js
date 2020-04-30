const fs = require('fs');

const main = fs.readFileSync('./cache/kcs2/js/main.js').toString();
fs.writeFileSync('./cache/kcs2/js/main.js.bak', main);

const re_60fps = /(createjs(?:\[\w+\('\w+'\)\]){2})=(createjs(?:\[\w+\('\w+'\)\]){2})/

let patched = main.replace(re_60fps, "$1=createjs.Ticker.RAF");

fs.writeFileSync('./cache/kcs2/js/main.js', patched);
