(window.webpackJsonp=window.webpackJsonp||[]).push([[1],{178:function(n,t,r){"use strict";r.r(t),r.d(t,"run",function(){return u}),r.d(t,"get_rust_sudoku",function(){return o}),r.d(t,"__widl_f_debug_1_",function(){return d}),r.d(t,"__widl_f_error_1_",function(){return l}),r.d(t,"__widl_f_info_1_",function(){return a}),r.d(t,"__widl_f_log_1_",function(){return w}),r.d(t,"__widl_f_warn_1_",function(){return b}),r.d(t,"__wbg_error_4bb6c2a97407129a",function(){return y}),r.d(t,"__wbg_new_59cb74e423758ede",function(){return v}),r.d(t,"__wbg_stack_558ba5917b466edd",function(){return O}),r.d(t,"__wbindgen_string_new",function(){return S}),r.d(t,"__wbindgen_json_parse",function(){return m}),r.d(t,"__wbindgen_json_serialize",function(){return I}),r.d(t,"__wbindgen_throw",function(){return N}),r.d(t,"WasmSudoku",function(){return T}),r.d(t,"__wbindgen_object_drop_ref",function(){return U});var e=r(179);function u(){return e.g()}function o(){return T.__wrap(e.e())}const i=new Array(32);function c(n){return i[n]}i.fill(void 0),i.push(void 0,null,!0,!1);let _=i.length;function f(n){n<36||(i[n]=_,_=n)}function s(n){_===i.length&&i.push(i.length+1);const t=_;return _=i[t],i[t]=n,t}function d(n){console.debug(c(n))}function l(n){console.error(c(n))}function a(n){console.info(c(n))}function w(n){console.log(c(n))}function b(n){console.warn(c(n))}let g=new TextDecoder("utf-8"),p=null;function h(){return null!==p&&p.buffer===e.f.buffer||(p=new Uint8Array(e.f.buffer)),p}function k(n,t){return g.decode(h().subarray(n,n+t))}function y(n,t){let r=k(n,t);r=r.slice(),e.b(n,1*t),console.error(r)}function v(){return s(new Error)}let j,J=0,x=new TextEncoder("utf-8");j="function"==typeof x.encodeInto?function(n){let t=n.length,r=e.c(t),u=0;for(;;){const o=h().subarray(r+u,r+t),{read:i,written:c}=x.encodeInto(n,o);if(u+=c,i===n.length)break;n=n.substring(i),r=e.d(r,t,t+=3*n.length)}return J=u,r}:function(n){const t=x.encode(n),r=e.c(t.length);return h().set(t,r),J=t.length,r};let A=null;function E(){return null!==A&&A.buffer===e.f.buffer||(A=new Uint32Array(e.f.buffer)),A}function O(n,t){const r=j(c(t).stack),e=J,u=E();u[n/4]=r,u[n/4+1]=e}function S(n,t){return s(k(n,t))}function m(n,t){return s(JSON.parse(k(n,t)))}function I(n,t){const r=j(JSON.stringify(c(n)));return E()[t/4]=r,J}function N(n,t){throw new Error(k(n,t))}class T{static __wrap(n){const t=Object.create(T.prototype);return t.ptr=n,t}free(){const n=this.ptr;this.ptr=0,function(n){e.a(n)}(n)}say_hello(){return e.j(this.ptr)}get_sudoku(){return function(n){const t=c(n);return f(n),t}(e.i(this.ptr))}set_value(n,t){return e.n(this.ptr,s(n),t)}set_or_toggle_value(n,t){return e.m(this.ptr,s(n),t)}set_candidates(n,t){return e.l(this.ptr,s(n),s(t))}toggle_candidate(n,t){return e.o(this.ptr,s(n),t)}delete(n){return e.h(this.ptr,s(n))}set_all_direct_candidates(){return e.k(this.ptr)}}function U(n){f(n)}},179:function(n,t,r){"use strict";var e=r.w[n.i];n.exports=e;r(178);e.p()}}]);