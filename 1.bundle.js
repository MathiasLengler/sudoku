(window.webpackJsonp=window.webpackJsonp||[]).push([[1],{279:function(n,t,r){"use strict";r.r(t),r.d(t,"run",(function(){return o})),r.d(t,"get_wasm_sudoku",(function(){return u})),r.d(t,"WasmSudoku",(function(){return k})),r.d(t,"__wbindgen_string_new",(function(){return O})),r.d(t,"__wbindgen_json_serialize",(function(){return j})),r.d(t,"__wbindgen_object_drop_ref",(function(){return E})),r.d(t,"__wbindgen_json_parse",(function(){return S})),r.d(t,"__widl_f_debug_1_",(function(){return x})),r.d(t,"__widl_f_error_1_",(function(){return A})),r.d(t,"__widl_f_info_1_",(function(){return J})),r.d(t,"__widl_f_log_1_",(function(){return N})),r.d(t,"__widl_f_warn_1_",(function(){return R})),r.d(t,"__wbindgen_is_undefined",(function(){return V})),r.d(t,"__wbg_new_59cb74e423758ede",(function(){return D})),r.d(t,"__wbg_stack_558ba5917b466edd",(function(){return F})),r.d(t,"__wbg_error_4bb6c2a97407129a",(function(){return I})),r.d(t,"__wbg_randomFillSync_eae3007264ffc138",(function(){return T})),r.d(t,"__wbg_getRandomValues_f724b5822126eff7",(function(){return U})),r.d(t,"__wbg_self_1801c027cb0e6124",(function(){return q})),r.d(t,"__wbg_require_e89d842e759f0a4c",(function(){return C})),r.d(t,"__wbg_crypto_3e91f24788b1203d",(function(){return M})),r.d(t,"__wbg_getRandomValues_7ecea3ecacbb2f9e",(function(){return z})),r.d(t,"__wbindgen_throw",(function(){return B})),r.d(t,"__wbindgen_rethrow",(function(){return L}));var e=r(280);function o(){e.h()}function u(){const n=e.f();return k.__wrap(n)}const i=new Array(32);function c(n){return i[n]}i.fill(void 0),i.push(void 0,null,!0,!1);let _=i.length;function f(n){const t=c(n);return function(n){n<36||(i[n]=_,_=n)}(n),t}function s(n){_===i.length&&i.push(i.length+1);const t=_;return _=i[t],i[t]=n,t}let d=0,l=new TextEncoder("utf-8");const a="function"==typeof l.encodeInto?function(n,t){return l.encodeInto(n,t)}:function(n,t){const r=l.encode(n);return t.set(r),{read:n.length,written:r.length}};let w=null;function g(){return null!==w&&w.buffer===e.g.buffer||(w=new Uint8Array(e.g.buffer)),w}function b(n){let t=n.length,r=e.d(t);const o=g();let u=0;for(;u<t;u++){const t=n.charCodeAt(u);if(t>127)break;o[r+u]=t}if(u!==t){0!==u&&(n=n.slice(u)),r=e.e(r,t,t=u+3*n.length);const o=g().subarray(r+u,r+t);u+=a(n,o).written}return d=u,r}let p=new TextDecoder("utf-8",{ignoreBOM:!0,fatal:!0});function h(n,t){return p.decode(g().subarray(n,n+t))}let y=null;function m(){return null!==y&&y.buffer===e.g.buffer||(y=new Int32Array(e.g.buffer)),y}function v(n,t){return g().subarray(n/1,n/1+t)}class k{static __wrap(n){const t=Object.create(k.prototype);return t.ptr=n,t}free(){const n=this.ptr;this.ptr=0,e.a(n)}get_sudoku(){return f(e.k(this.ptr))}set_value(n,t){e.q(this.ptr,s(n),t)}set_or_toggle_value(n,t){e.p(this.ptr,s(n),t)}set_candidates(n,t){e.o(this.ptr,s(n),s(t))}toggle_candidate(n,t){e.s(this.ptr,s(n),t)}delete(n){e.i(this.ptr,s(n))}set_all_direct_candidates(){e.n(this.ptr)}undo(){e.t(this.ptr)}generate(n){e.j(this.ptr,s(n))}import(n){e.m(this.ptr,b(n),d)}solve_single_candidates(){e.r(this.ptr)}group_reduction(){e.l(this.ptr)}}const O=function(n,t){return s(h(n,t))},j=function(n,t){const r=c(t),e=b(JSON.stringify(void 0===r?null:r)),o=d;m()[n/4+0]=e,m()[n/4+1]=o},E=function(n){f(n)},S=function(n,t){return s(JSON.parse(h(n,t)))},x=function(n){console.debug(c(n))},A=function(n){console.error(c(n))},J=function(n){console.info(c(n))},N=function(n){console.log(c(n))},R=function(n){console.warn(c(n))},V=function(n){return void 0===c(n)},D=function(){return s(new Error)},F=function(n,t){const r=b(c(t).stack),e=d;m()[n/4+0]=r,m()[n/4+1]=e},I=function(n,t){const r=h(n,t).slice();e.c(n,1*t),console.error(r)},T=function(n,t,r){c(n).randomFillSync(v(t,r))},U=function(n,t,r){c(n).getRandomValues(v(t,r))},q=function(){try{return s(self.self)}catch(n){!function(n){e.b(s(n))}(n)}},C=function(n,t){return s(r(281)(h(n,t)))},M=function(n){return s(c(n).crypto)},z=function(n){return s(c(n).getRandomValues)},B=function(n,t){throw new Error(h(n,t))},L=function(n){throw f(n)}},280:function(n,t,r){"use strict";var e=r.w[n.i];n.exports=e;r(279);e.u()},281:function(n,t){function r(n){var t=new Error("Cannot find module '"+n+"'");throw t.code="MODULE_NOT_FOUND",t}r.keys=function(){return[]},r.resolve=r,n.exports=r,r.id=281}}]);