(window.webpackJsonp=window.webpackJsonp||[]).push([[1],{160:function(n,t,r){"use strict";r.r(t),r.d(t,"run",function(){return u}),r.d(t,"get_wasm_sudoku",function(){return o}),r.d(t,"__widl_f_debug_1_",function(){return l}),r.d(t,"__widl_f_error_1_",function(){return s}),r.d(t,"__widl_f_info_1_",function(){return a}),r.d(t,"__widl_f_log_1_",function(){return b}),r.d(t,"__widl_f_warn_1_",function(){return w}),r.d(t,"__wbg_error_4bb6c2a97407129a",function(){return k}),r.d(t,"__wbg_new_59cb74e423758ede",function(){return m}),r.d(t,"__wbg_stack_558ba5917b466edd",function(){return S}),r.d(t,"__wbg_new_c6fb156f56e56d5f",function(){return x}),r.d(t,"__wbg_call_f0cc87553c1e3b2a",function(){return F}),r.d(t,"__wbg_self_4bcc93945365c19a",function(){return J}),r.d(t,"__wbg_crypto_67744919473fd26f",function(){return N}),r.d(t,"__wbg_getRandomValues_cfbff5328b3ac59b",function(){return R}),r.d(t,"__wbg_getRandomValues_542f44b2df1d9c36",function(){return V}),r.d(t,"__wbg_require_10ff47d62350bb71",function(){return C}),r.d(t,"__wbg_randomFillSync_7c8b02669ad0870b",function(){return D}),r.d(t,"__wbindgen_string_new",function(){return T}),r.d(t,"__wbindgen_is_undefined",function(){return q}),r.d(t,"__wbindgen_json_parse",function(){return I}),r.d(t,"__wbindgen_json_serialize",function(){return z}),r.d(t,"__wbindgen_jsval_eq",function(){return L}),r.d(t,"__wbindgen_throw",function(){return M}),r.d(t,"WasmSudoku",function(){return W}),r.d(t,"__wbindgen_object_drop_ref",function(){return B});var e=r(161);function u(){return e.g()}function o(){return W.__wrap(e.e())}const c=new Array(32);function i(n){return c[n]}c.fill(void 0),c.push(void 0,null,!0,!1);let f=c.length;function _(n){n<36||(c[n]=f,f=n)}function d(n){f===c.length&&c.push(c.length+1);const t=f;return f=c[t],c[t]=n,t}function l(n){console.debug(i(n))}function s(n){console.error(i(n))}function a(n){console.info(i(n))}function b(n){console.log(i(n))}function w(n){console.warn(i(n))}let g=new TextDecoder("utf-8"),h=null;function p(){return null!==h&&h.buffer===e.f.buffer||(h=new Uint8Array(e.f.buffer)),h}function y(n,t){return g.decode(p().subarray(n,n+t))}function k(n,t){let r=y(n,t);r=r.slice(),e.b(n,1*t),console.error(r)}function m(){return d(new Error)}let v,j=0,O=new TextEncoder("utf-8");v="function"==typeof O.encodeInto?function(n){let t=n.length,r=e.c(t),u=0;{const t=p();for(;u<n.length;u++){const e=n.charCodeAt(u);if(e>127)break;t[r+u]=e}}if(u!==n.length){n=n.slice(u),r=e.d(r,t,t=u+3*n.length);const o=p().subarray(r+u,r+t);u+=O.encodeInto(n,o).written}return j=u,r}:function(n){let t=n.length,r=e.c(t),u=0;{const t=p();for(;u<n.length;u++){const e=n.charCodeAt(u);if(e>127)break;t[r+u]=e}}if(u!==n.length){const o=O.encode(n.slice(u));r=e.d(r,t,t=u+o.length),p().set(o,r+u),u+=o.length}return j=u,r};let A=null;function E(){return null!==A&&A.buffer===e.f.buffer||(A=new Uint32Array(e.f.buffer)),A}function S(n,t){const r=v(i(t).stack),e=j,u=E();u[n/4]=r,u[n/4+1]=e}function x(n,t){let r=y(n,t);return d(new Function(r))}function F(n,t){return d(i(n).call(i(t)))}function J(n){return d(i(n).self)}function N(n){return d(i(n).crypto)}function R(n){return d(i(n).getRandomValues)}function U(n,t){return p().subarray(n/1,n/1+t)}function V(n,t,r){let e=U(t,r);i(n).getRandomValues(e)}function C(n,t){let e=y(n,t);return d(r(162)(e))}function D(n,t,r){let e=U(t,r);i(n).randomFillSync(e)}function T(n,t){return d(y(n,t))}function q(n){return void 0===i(n)?1:0}function I(n,t){return d(JSON.parse(y(n,t)))}function z(n,t){const r=v(JSON.stringify(i(n)));return E()[t/4]=r,j}function L(n,t){return i(n)===i(t)?1:0}function M(n,t){throw new Error(y(n,t))}class W{static __wrap(n){const t=Object.create(W.prototype);return t.ptr=n,t}free(){const n=this.ptr;this.ptr=0,function(n){e.a(n)}(n)}say_hello(){return e.j(this.ptr)}get_sudoku(){return function(n){const t=i(n);return _(n),t}(e.i(this.ptr))}set_value(n,t){return e.n(this.ptr,d(n),t)}set_or_toggle_value(n,t){return e.m(this.ptr,d(n),t)}set_candidates(n,t){return e.l(this.ptr,d(n),d(t))}toggle_candidate(n,t){return e.o(this.ptr,d(n),t)}delete(n){return e.h(this.ptr,d(n))}set_all_direct_candidates(){return e.k(this.ptr)}}function B(n){_(n)}},161:function(n,t,r){"use strict";var e=r.w[n.i];n.exports=e;r(160);e.p()},162:function(n,t){function r(n){var t=new Error("Cannot find module '"+n+"'");throw t.code="MODULE_NOT_FOUND",t}r.keys=function(){return[]},r.resolve=r,n.exports=r,r.id=162}}]);