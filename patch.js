const fs = require('fs');

let main = fs.readFileSync('./cache/kcs2/js/main.js').toString();
fs.writeFileSync('./cache/kcs2/js/main.js.bak', main);

const re_raf = /(createjs(?:\[\w+\('\w+'\)\]){2})=(createjs(?:\[\w+\('\w+'\)\]){2})/
console.log(`re_raf = ${re_raf}`);
main = main.replace(re_raf, "$1=createjs.Ticker.RAF");

const mouseup_detect_code = "=!0x1,".concat(new Array(4).fill("this(?:\\[\\w+\\('\\w+'\\)\\]){2}\\((\\w+\\[\\w+\\('\\w+'\\)\\])(\\[\\w+\\('\\w+'\\)\\]),this(\\[\\w+\\('\\w+'\\)\\])\\)").join(","));
let button_mouse_pattern = new RegExp(mouseup_detect_code, "g");
console.log(`button_mouse_pattern = ${button_mouse_pattern}`);
let matcher;
while ((matcher = button_mouse_pattern.exec(main)) !== null) {
    let event_type = matcher[1];
    let prop_mousedown = matcher[8];
    let prop_mouseup = matcher[11];
    let on_mouseup = matcher[12];
    console.log(`on_mouseup = ${on_mouseup}`);
    let regex_event_type = new RegExp(event_type.concat(prop_mouseup).concat(",this").concat(on_mouseup)
        .replace("(", "\\(").replace(")", "\\)").replace("[", "\\["), "g");
    console.log(`regex_event_type = ${regex_event_type}`);
    let replace_event_type = event_type.concat(prop_mousedown).concat(",this").concat(on_mouseup);
    console.log(`replace_event_type = ${replace_event_type}`);
    main = main.replace(regex_event_type, replace_event_type);
}
main += "function patchInteractionManager () {\n" +
    "  var proto = PIXI.interaction.InteractionManager.prototype;\n" +
    "\n" +
    "  function extendMethod (method, extFn) {\n" +
    "    var old = proto[method];\n" +
    "    proto[method] = function () {\n" +
    "      old.call(this, ...arguments);\n" +
    "      extFn.call(this, ...arguments);\n" +
    "    };\n" +
    "  }\n" +
    "  proto.update = mobileUpdate;\n" +
    "\n" +
    "  function mobileUpdate(deltaTime) {\n" +
    "    if (!this.interactionDOMElement) {\n" +
    "      return;\n" +
    "    }\n" +
    // Only trigger "touchout" when there is another object start "touchover", do nothing when "touchend"
    // So that alert bubbles persist after a simple tap, do not disappear when the finger leaves
    "    if (this.eventData.data && (this.eventData.type == 'touchmove' || this.eventData.type == 'touchstart')) {\n" +
    "      window.__eventData = this.eventData;\n" +
    "      this.processInteractive(this.eventData, this.renderer._lastObjectRendered, this.processTouchOverOut, true);\n" +
    "    }\n" +
    "  }\n" +
    "\n" +
    "  extendMethod('processTouchMove', function(displayObject, hit) {\n" +
    "      this.processTouchOverOut('processTouchMove', displayObject, hit);\n" +
    "  });\n" +
    "  extendMethod('processTouchStart', function(displayObject, hit) {\n" +
    "      this.processTouchOverOut('processTouchStart', displayObject, hit);\n" +
    "  });\n" +
    "\n" +
    "  proto.processTouchOverOut = function (interactionEvent, displayObject, hit) {\n" +
    "    if(hit) {\n" +
    "      if(!displayObject.__over && displayObject._events.touchover) {\n" +
    "        if (displayObject.parent._onClickAll2) return;\n" +
    "        displayObject.__over = true;\n" +
    "        proto.dispatchEvent( displayObject, 'touchover', window.__eventData);\n" +
    "      }\n" +
    "    } else {\n" +
    // Only trigger "touchout" when user starts touching another object
    "        if(displayObject.__over && displayObject._events.touchover && interactionEvent.target != displayObject) {\n" +
    "            displayObject.__over = false;\n" +
    "            proto.dispatchEvent( displayObject, 'touchout', window.__eventData);\n" +
    "        }\n" +
    "    }\n" +
    "  };\n" +
    "}\n" +
    "patchInteractionManager();";
main = main.replace(/'over':\w+\[\w+\('\w+'\)\]\?\w+\('\w+'\):\w+\('\w+'\)/g, "'over':'touchover'");
main = main.replace(/'out':\w+\[\w+\('\w+'\)\]\?\w+\('\w+'\):\w+\('\w+'\)/g, "'out':'touchout'");

main = main + `(function(){
    window.addEventListener('DOMContentLoaded', (event) => {
        document.getElementsByTagName('canvas')[0].style.width = "100%";
        document.getElementsByTagName('body')[0].style.overflow = "hidden";
    });
})();`;

fs.writeFileSync('./cache/kcs2/js/main.js', main);
