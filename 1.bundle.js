(window.webpackJsonp=window.webpackJsonp||[]).push([[1],{181:function(n,t,r){"use strict";r.r(t),r.d(t,"run",function(){return u}),r.d(t,"get_wasm_sudoku",function(){return o}),r.d(t,"__widl_f_debug_1_",function(){return s}),r.d(t,"__widl_f_error_1_",function(){return l}),r.d(t,"__widl_f_info_1_",function(){return a}),r.d(t,"__widl_f_log_1_",function(){return w}),r.d(t,"__widl_f_warn_1_",function(){return b}),r.d(t,"__wbg_error_4bb6c2a97407129a",function(){return k}),r.d(t,"__wbg_new_59cb74e423758ede",function(){return m}),r.d(t,"__wbg_stack_558ba5917b466edd",function(){return x}),r.d(t,"__wbg_new_3a746f2619705add",function(){return F}),r.d(t,"__wbg_call_f54d3a6dadb199ca",function(){return J}),r.d(t,"__wbg_self_ac379e780a0d8b94",function(){return N}),r.d(t,"__wbg_crypto_1e4302b85d4f64a2",function(){return R}),r.d(t,"__wbg_getRandomValues_1b4ba144162a5c9e",function(){return U}),r.d(t,"__wbg_getRandomValues_1ef11e888e5228e9",function(){return A}),r.d(t,"__wbg_require_6461b1e9a0d7c34a",function(){return D}),r.d(t,"__wbg_randomFillSync_1b52c8482374c55b",function(){return T}),r.d(t,"__wbindgen_string_new",function(){return q}),r.d(t,"__wbindgen_is_undefined",function(){return I}),r.d(t,"__wbindgen_json_parse",function(){return z}),r.d(t,"__wbindgen_json_serialize",function(){return C}),r.d(t,"__wbindgen_jsval_eq",function(){return L}),r.d(t,"__wbindgen_throw",function(){return M}),r.d(t,"WasmSudoku",function(){return W}),r.d(t,"__wbindgen_object_drop_ref",function(){return B});var e=r(182);function u(){return e.g()}function o(){return W.__wrap(e.e())}const i=new Array(32);function c(n){return i[n]}i.fill(void 0),i.push(void 0,null,!0,!1);let _=i.length;function f(n){n<36||(i[n]=_,_=n)}function d(n){_===i.length&&i.push(i.length+1);const t=_;return _=i[t],i[t]=n,t}function s(n){console.debug(c(n))}function l(n){console.error(c(n))}function a(n){console.info(c(n))}function w(n){console.log(c(n))}function b(n){console.warn(c(n))}let g=new TextDecoder("utf-8"),p=null;function h(){return null!==p&&p.buffer===e.f.buffer||(p=new Uint8Array(e.f.buffer)),p}function y(n,t){return g.decode(h().subarray(n,n+t))}function k(n,t){let r=y(n,t);r=r.slice(),e.b(n,1*t),console.error(r)}function m(){return d(new Error)}let v,j=0,O=new TextEncoder("utf-8");v="function"==typeof O.encodeInto?function(n){let t=n.length,r=e.c(t),u=0;for(;;){const o=h().subarray(r+u,r+t),{read:i,written:c}=O.encodeInto(n,o);if(u+=c,i===n.length)break;n=n.substring(i),r=e.d(r,t,t+=3*n.length)}return j=u,r}:function(n){const t=O.encode(n),r=e.c(t.length);return h().set(t,r),j=t.length,r};let E=null;function S(){return null!==E&&E.buffer===e.f.buffer||(E=new Uint32Array(e.f.buffer)),E}function x(n,t){const r=v(c(t).stack),e=j,u=S();u[n/4]=r,u[n/4+1]=e}function F(n,t){let r=y(n,t);return d(new Function(r))}function J(n,t){return d(c(n).call(c(t)))}function N(n){return d(c(n).self)}function R(n){return d(c(n).crypto)}function U(n){return d(c(n).getRandomValues)}function V(n,t){return h().subarray(n/1,n/1+t)}function A(n,t,r){let e=V(t,r);c(n).getRandomValues(e)}function D(n,t){let e=y(n,t);return d(r(183)(e))}function T(n,t,r){let e=V(t,r);c(n).randomFillSync(e)}function q(n,t){return d(y(n,t))}function I(n){return void 0===c(n)?1:0}function z(n,t){return d(JSON.parse(y(n,t)))}function C(n,t){const r=v(JSON.stringify(c(n)));return S()[t/4]=r,j}function L(n,t){return c(n)===c(t)?1:0}function M(n,t){throw new Error(y(n,t))}class W{static __wrap(n){const t=Object.create(W.prototype);return t.ptr=n,t}free(){const n=this.ptr;this.ptr=0,function(n){e.a(n)}(n)}say_hello(){return e.j(this.ptr)}get_sudoku(){return function(n){const t=c(n);return f(n),t}(e.i(this.ptr))}set_value(n,t){return e.n(this.ptr,d(n),t)}set_or_toggle_value(n,t){return e.m(this.ptr,d(n),t)}set_candidates(n,t){return e.l(this.ptr,d(n),d(t))}toggle_candidate(n,t){return e.o(this.ptr,d(n),t)}delete(n){return e.h(this.ptr,d(n))}set_all_direct_candidates(){return e.k(this.ptr)}}function B(n){f(n)}},182:function(n,t,r){"use strict";var e=r.w[n.i];n.exports=e;r(181);e.p()},183:function(n,t){function r(n){var t=new Error("Cannot find module '"+n+"'");throw t.code="MODULE_NOT_FOUND",t}r.keys=function(){return[]},r.resolve=r,n.exports=r,r.id=183}}]);