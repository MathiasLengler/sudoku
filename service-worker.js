if(!self.define){const e=e=>{"require"!==e&&(e+=".js");let r=Promise.resolve();return s[e]||(r=new Promise((async r=>{if("document"in self){const s=document.createElement("script");s.src=e,document.head.appendChild(s),s.onload=r}else importScripts(e),r()}))),r.then((()=>{if(!s[e])throw new Error(`Module ${e} didn’t register its module`);return s[e]}))},r=(r,s)=>{Promise.all(r.map(e)).then((e=>s(1===e.length?e[0]:e)))},s={require:Promise.resolve(r)};self.define=(r,i,c)=>{s[r]||(s[r]=Promise.resolve().then((()=>{let s={};const n={uri:location.origin+r.slice(1)};return Promise.all(i.map((r=>{switch(r){case"exports":return s;case"module":return n;default:return e(r)}}))).then((e=>{const r=c(...e);return s.default||(s.default=r),s}))})))}}define("./service-worker.js",["./workbox-0d8277e8"],(function(e){"use strict";self.skipWaiting(),e.clientsClaim(),e.precacheAndRoute([{url:"124.app.js",revision:"64adc6c26eca37b186ec56525d1e860e"},{url:"171.app.js",revision:"41d0dcd2dc1bded160981a0a895d8ea1"},{url:"app.js",revision:"59eb80ff879e21a1c6bb41586f9ae318"},{url:"app.js.LICENSE.txt",revision:"6598e2bcee6e7eca94a661475b0770d1"},{url:"assets/favicon-16x16.png",revision:"f2c36f4584800b150459f4f07037aeed"},{url:"assets/favicon-32x32.png",revision:"8f2cd90d2c4fa577e727a1c5570d5f54"},{url:"assets/favicon-48x48.png",revision:"f1c8001d05c18f981aaf4a508d9b6cfc"},{url:"assets/favicon.ico",revision:"c03186aa57a2a0ca63887fe2b8463f03"},{url:"c1fd91823278735a336e.module.wasm",revision:null},{url:"index.html",revision:"4b603b83330ba0b30be6debb89257f7f"}],{})}));
//# sourceMappingURL=service-worker.js.map
