!function(e){self.webpackChunk=function(t,r){for(var o in r)e[o]=r[o];for(;t.length;)n[t.pop()]=1};var t={},n={0:1},r={};var o={2:function(){return{"./index_bg.js":{__wbindgen_json_parse:function(e,n){return t[1].exports.r(e,n)},__wbindgen_json_serialize:function(e,n){return t[1].exports.s(e,n)},__wbindgen_object_drop_ref:function(e){return t[1].exports.t(e)},__wbindgen_string_new:function(e,n){return t[1].exports.v(e,n)},__wbg_debug_b1dc198008810ba0:function(e){return t[1].exports.c(e)},__wbg_error_44d97cfce214d7c7:function(e){return t[1].exports.d(e)},__wbg_info_0e6ed2caec2b6177:function(e){return t[1].exports.h(e)},__wbg_log_d85e484a8ba03c98:function(e){return t[1].exports.i(e)},__wbg_warn_381332f6edb6676a:function(e){return t[1].exports.p(e)},__wbg_new_59cb74e423758ede:function(){return t[1].exports.j()},__wbg_stack_558ba5917b466edd:function(e,n){return t[1].exports.o(e,n)},__wbg_error_4bb6c2a97407129a:function(e,n){return t[1].exports.e(e,n)},__wbg_new_f6f27499ae4ea5b4:function(e,n){return t[1].exports.k(e,n)},__wbindgen_is_undefined:function(e){return t[1].exports.q(e)},__wbg_getRandomValues_f5e14ab7ac8e995d:function(e,n,r){return t[1].exports.g(e,n,r)},__wbg_randomFillSync_d5bd2d655fdf256a:function(e,n,r){return t[1].exports.l(e,n,r)},__wbg_self_1b7a39e3a92c949c:function(){return t[1].exports.n()},__wbg_require_604837428532a733:function(e,n){return t[1].exports.m(e,n)},__wbg_crypto_968f1772287e2df0:function(e){return t[1].exports.b(e)},__wbg_getRandomValues_a3d34b4fee3c2869:function(e){return t[1].exports.f(e)},__wbindgen_throw:function(e,n){return t[1].exports.w(e,n)},__wbindgen_rethrow:function(e){return t[1].exports.u(e)}}}}};function a(n){if(t[n])return t[n].exports;var r=t[n]={i:n,l:!1,exports:{}};return e[n].call(r.exports,r,r.exports,a),r.l=!0,r.exports}a.e=function(e){var t=[];return t.push(Promise.resolve().then((function(){n[e]||importScripts(a.p+""+e+".app.worker.js")}))),({1:[2]}[e]||[]).forEach((function(e){var n=r[e];if(n)t.push(n);else{var s,u=o[e](),i=fetch(a.p+""+{2:"d815f224d4218a8bab6b"}[e]+".module.wasm");if(u instanceof Promise&&"function"==typeof WebAssembly.compileStreaming)s=Promise.all([WebAssembly.compileStreaming(i),u]).then((function(e){return WebAssembly.instantiate(e[0],e[1])}));else if("function"==typeof WebAssembly.instantiateStreaming)s=WebAssembly.instantiateStreaming(i,u);else{s=i.then((function(e){return e.arrayBuffer()})).then((function(e){return WebAssembly.instantiate(e,u)}))}t.push(r[e]=s.then((function(t){return a.w[e]=(t.instance||t).exports})))}})),Promise.all(t)},a.m=e,a.c=t,a.d=function(e,t,n){a.o(e,t)||Object.defineProperty(e,t,{enumerable:!0,get:n})},a.r=function(e){"undefined"!=typeof Symbol&&Symbol.toStringTag&&Object.defineProperty(e,Symbol.toStringTag,{value:"Module"}),Object.defineProperty(e,"__esModule",{value:!0})},a.t=function(e,t){if(1&t&&(e=a(e)),8&t)return e;if(4&t&&"object"==typeof e&&e&&e.__esModule)return e;var n=Object.create(null);if(a.r(n),Object.defineProperty(n,"default",{enumerable:!0,value:e}),2&t&&"string"!=typeof e)for(var r in e)a.d(n,r,function(t){return e[t]}.bind(null,r));return n},a.n=function(e){var t=e&&e.__esModule?function(){return e.default}:function(){return e};return a.d(t,"a",t),t},a.o=function(e,t){return Object.prototype.hasOwnProperty.call(e,t)},a.p="",a.w={},a(a.s=0)}([function(e,t,n){"use strict";n.r(t);const r=Symbol("Comlink.proxy"),o=Symbol("Comlink.endpoint"),a=Symbol("Comlink.releaseProxy"),s=Symbol("Comlink.thrown"),u=e=>"object"==typeof e&&null!==e||"function"==typeof e,i=new Map([["proxy",{canHandle:e=>u(e)&&e[r],serialize(e){const{port1:t,port2:n}=new MessageChannel;return c(e,t),[n,[n]]},deserialize(e){return e.start(),function e(t,n=[],r=function(){}){let s=!1;const u=new Proxy(r,{get(r,o){if(l(s),o===a)return()=>g(t,{type:5,path:n.map(e=>e.toString())}).then(()=>{d(t),s=!0});if("then"===o){if(0===n.length)return{then:()=>u};const e=g(t,{type:0,path:n.map(e=>e.toString())}).then(b);return e.then.bind(e)}return e(t,[...n,o])},set(e,r,o){l(s);const[a,u]=_(o);return g(t,{type:1,path:[...n,r].map(e=>e.toString()),value:a},u).then(b)},apply(r,a,u){l(s);const i=n[n.length-1];if(i===o)return g(t,{type:4}).then(b);if("bind"===i)return e(t,n.slice(0,-1));const[c,d]=f(u);return g(t,{type:2,path:n.map(e=>e.toString()),argumentList:c},d).then(b)},construct(e,r){l(s);const[o,a]=f(r);return g(t,{type:3,path:n.map(e=>e.toString()),argumentList:o},a).then(b)}});return u}(e,[],t);var t}}],["throw",{canHandle:e=>u(e)&&s in e,serialize({value:e}){let t;return t=e instanceof Error?{isError:!0,value:{message:e.message,name:e.name,stack:e.stack}}:{isError:!1,value:e},[t,[]]},deserialize(e){if(e.isError)throw Object.assign(new Error(e.value.message),e.value);throw e.value}}]]);function c(e,t=self){t.addEventListener("message",(function n(o){if(!o||!o.data)return;const{id:a,type:u,path:i}=Object.assign({path:[]},o.data),l=(o.data.argumentList||[]).map(b);let f;try{const t=i.slice(0,-1).reduce((e,t)=>e[t],e),n=i.reduce((e,t)=>e[t],e);switch(u){case 0:f=n;break;case 1:t[i.slice(-1)[0]]=b(o.data.value),f=!0;break;case 2:f=n.apply(t,l);break;case 3:f=function(e){return Object.assign(e,{[r]:!0})}(new n(...l));break;case 4:{const{port1:t,port2:n}=new MessageChannel;c(e,n),f=function(e,t){return p.set(e,t),e}(t,[t])}break;case 5:f=void 0}}catch(e){f={value:e,[s]:0}}Promise.resolve(f).catch(e=>({value:e,[s]:0})).then(e=>{const[r,o]=_(e);t.postMessage(Object.assign(Object.assign({},r),{id:a}),o),5===u&&(t.removeEventListener("message",n),d(t))})})),t.start&&t.start()}function d(e){(function(e){return"MessagePort"===e.constructor.name})(e)&&e.close()}function l(e){if(e)throw new Error("Proxy has been released and is not useable")}function f(e){const t=e.map(_);return[t.map(e=>e[0]),(n=t.map(e=>e[1]),Array.prototype.concat.apply([],n))];var n}const p=new WeakMap;function _(e){for(const[t,n]of i)if(n.canHandle(e)){const[r,o]=n.serialize(e);return[{type:3,name:t,value:r},o]}return[{type:0,value:e},p.get(e)||[]]}function b(e){switch(e.type){case 3:return i.get(e.name).deserialize(e.value);case 0:return e.value}}function g(e,t,n){return new Promise(r=>{const o=new Array(4).fill(0).map(()=>Math.floor(Math.random()*Number.MAX_SAFE_INTEGER).toString(16)).join("-");e.addEventListener("message",(function t(n){n.data&&n.data.id&&n.data.id===o&&(e.removeEventListener("message",t),r(n.data))})),e.start&&e.start(),e.postMessage(Object.assign({id:o},t),n)})}class m{constructor(e){this.wasmSudoku=e}getSudoku(){return this.wasmSudoku.get_sudoku()}setValue(e,t){return this.wasmSudoku.set_value(e,t)}setOrToggleValue(e,t){return this.wasmSudoku.set_or_toggle_value(e,t)}setCandidates(e,t){return this.wasmSudoku.set_candidates(e,t)}toggleCandidate(e,t){return this.wasmSudoku.toggle_candidate(e,t)}delete(e){return this.wasmSudoku.delete(e)}setAllDirectCandidates(){return this.wasmSudoku.set_all_direct_candidates()}undo(){return this.wasmSudoku.undo()}generate(e){return this.wasmSudoku.generate(e)}import(e){return this.wasmSudoku.import(e)}solveSingleCandidates(){return this.wasmSudoku.solve_single_candidates()}groupReduction(){return this.wasmSudoku.group_reduction()}}const w={init:async function(){return w.typedWasmSudoku=await h(),"Worker initialized"},typedWasmSudoku:void 0};async function h(){const e=await n.e(1).then(n.bind(null,5));return e.run(),new m(e.get_wasm_sudoku())}c(w)}]);
//# sourceMappingURL=0.app.worker.js.map