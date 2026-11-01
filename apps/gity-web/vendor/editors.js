import { _ as e, a as t, c as n, d as r, f as i, g as a, h as o, i as s, l as c, m as l, n as u, o as d, p as f, r as p, s as m, t as h, u as g, v as _ } from "./toggleHighContrast-D1SNNEEF.mjs";
//#region node_modules/.pnpm/monaco-editor@0.55.1/node_modules/monaco-editor/esm/vs/editor/internal/initialize.js
function v() {
	return l;
}
globalThis.MonacoEnvironment?.globalAPI && (globalThis.monaco = v());
//#endregion
//#region node_modules/.pnpm/monaco-editor@0.55.1/node_modules/monaco-editor/esm/vs/editor/editor.api.js
var y = /* @__PURE__ */ e({
	CancellationTokenSource: () => h,
	Emitter: () => u,
	KeyCode: () => p,
	KeyMod: () => s,
	MarkerSeverity: () => t,
	MarkerTag: () => d,
	Position: () => m,
	Range: () => n,
	Selection: () => c,
	SelectionDirection: () => g,
	Token: () => r,
	Uri: () => i,
	editor: () => f,
	languages: () => o
}), b = {}, x = {}, ee = class e {
	static getOrCreate(t) {
		return x[t] || (x[t] = new e(t)), x[t];
	}
	constructor(e) {
		this._languageId = e, this._loadingTriggered = !1, this._lazyLoadPromise = new Promise((e, t) => {
			this._lazyLoadPromiseResolve = e, this._lazyLoadPromiseReject = t;
		});
	}
	load() {
		return this._loadingTriggered || (this._loadingTriggered = !0, b[this._languageId].loader().then((e) => this._lazyLoadPromiseResolve(e), (e) => this._lazyLoadPromiseReject(e))), this._lazyLoadPromise;
	}
};
function S(e) {
	let t = e.id;
	b[t] = e, o.register(e);
	let n = ee.getOrCreate(t);
	o.registerTokensProviderFactory(t, { create: async () => (await n.load()).language }), o.onLanguageEncountered(t, async () => {
		let e = await n.load();
		o.setLanguageConfiguration(t, e.conf);
	});
}
//#endregion
//#region node_modules/.pnpm/monaco-editor@0.55.1/node_modules/monaco-editor/esm/vs/basic-languages/markdown/markdown.contribution.js
S({
	id: "rust",
	extensions: [".rs", ".rlib"],
	aliases: ["Rust", "rust"],
	loader: () => import("./rust-CoVfq9Sb.mjs")
}), S({
	id: "yaml",
	extensions: [".yaml", ".yml"],
	aliases: [
		"YAML",
		"yaml",
		"YML",
		"yml"
	],
	mimetypes: ["application/x-yaml", "text/x-yaml"],
	loader: () => import("./yaml-B4Sh2dP6.mjs")
}), S({
	id: "markdown",
	extensions: [
		".md",
		".markdown",
		".mdown",
		".mkdn",
		".mkd",
		".mdwn",
		".mdtxt",
		".mdtext"
	],
	aliases: ["Markdown", "markdown"],
	loader: () => import("./markdown-TGF7r5jx.mjs")
});
//#endregion
//#region node_modules/.pnpm/monaco-editor@0.55.1/node_modules/monaco-editor/esm/vs/editor/editor.worker.js?worker&inline
var te = "const e=new class{constructor(){this.listeners=[],this.unexpectedErrorHandler=function(e){setTimeout(()=>{throw e.stack?a.isErrorNoTelemetry(e)?new a(e.message+`\n\n`+e.stack):Error(e.message+`\n\n`+e.stack):e},0)}}emit(e){this.listeners.forEach(t=>{t(e)})}onUnexpectedError(e){this.unexpectedErrorHandler(e),this.emit(e)}onUnexpectedExternalError(e){this.unexpectedErrorHandler(e)}};function t(t){r(t)||e.onUnexpectedError(t)}function n(e){if(e instanceof Error){let{name:t,message:r,cause:i}=e;return{$isError:!0,name:t,message:r,stack:e.stacktrace||e.stack,noTelemetry:a.isErrorNoTelemetry(e),cause:i?n(i):void 0,code:e.code}}return e}function r(e){return e instanceof i||e instanceof Error&&e.name===`Canceled`&&e.message===`Canceled`}var i=class extends Error{constructor(){super(`Canceled`),this.name=this.message}},a=class e extends Error{constructor(e){super(e),this.name=`CodeExpectedError`}static fromError(t){if(t instanceof e)return t;let n=new e;return n.message=t.message,n.stack=t.stack,n}static isErrorNoTelemetry(e){return e.name===`CodeExpectedError`}},o=class e extends Error{constructor(t){super(t||`An unexpected bug occurred.`),Object.setPrototypeOf(this,e.prototype)}};function s(e,t=`Unreachable`){throw Error(t)}function c(e,t=`unexpected state`){if(!e)throw typeof t==`string`?new o(`Assertion Failed: ${t}`):t}function l(e){e()||(e(),t(new o(`Assertion Failed`)))}function u(e,t){let n=0;for(;n<e.length-1;){let r=e[n],i=e[n+1];if(!t(r,i))return!1;n++}return!0}function d(e){return typeof e==`string`}function f(e){return!!e&&typeof e[Symbol.iterator]==`function`}var p;(function(e){function t(e){return!!e&&typeof e==`object`&&typeof e[Symbol.iterator]==`function`}e.is=t;let n=Object.freeze([]);function r(){return n}e.empty=r;function*i(e){yield e}e.single=i;function a(e){return t(e)?e:i(e)}e.wrap=a;function o(e){return e||n}e.from=o;function*s(e){for(let t=e.length-1;t>=0;t--)yield e[t]}e.reverse=s;function c(e){return!e||e[Symbol.iterator]().next().done===!0}e.isEmpty=c;function l(e){return e[Symbol.iterator]().next().value}e.first=l;function u(e,t){let n=0;for(let r of e)if(t(r,n++))return!0;return!1}e.some=u;function d(e,t){let n=0;for(let r of e)if(!t(r,n++))return!1;return!0}e.every=d;function p(e,t){for(let n of e)if(t(n))return n}e.find=p;function*m(e,t){for(let n of e)t(n)&&(yield n)}e.filter=m;function*h(e,t){let n=0;for(let r of e)yield t(r,n++)}e.map=h;function*g(e,t){let n=0;for(let r of e)yield*t(r,n++)}e.flatMap=g;function*_(...e){for(let t of e)f(t)?yield*t:yield t}e.concat=_;function v(e,t,n){let r=n;for(let n of e)r=t(r,n);return r}e.reduce=v;function y(e){let t=0;for(let n of e)t++;return t}e.length=y;function*b(e,t,n=e.length){for(t<-e.length&&(t=0),t<0&&(t+=e.length),n<0?n+=e.length:n>e.length&&(n=e.length);t<n;t++)yield e[t]}e.slice=b;function x(t,n=1/0){let r=[];if(n===0)return[r,t];let i=t[Symbol.iterator]();for(let t=0;t<n;t++){let t=i.next();if(t.done)return[r,e.empty()];r.push(t.value)}return[r,{[Symbol.iterator](){return i}}]}e.consume=x;async function S(e){let t=[];for await(let n of e)t.push(n);return t}e.asyncToArray=S;async function C(e){let t=[];for await(let n of e)t=t.concat(n);return t}e.asyncToArrayFlat=C})(p||={});function m(e){if(p.is(e)){let t=[];for(let n of e)if(n)try{n.dispose()}catch(e){t.push(e)}if(t.length===1)throw t[0];if(t.length>1)throw AggregateError(t,`Encountered errors while disposing of store`);return Array.isArray(e)?[]:e}if(e)return e.dispose(),e}function h(...e){return _(()=>m(e))}var g=class{constructor(e){this._isDisposed=!1,this._fn=e}dispose(){if(!this._isDisposed){if(!this._fn)throw Error(`Unbound disposable context: Need to use an arrow function to preserve the value of this`);this._isDisposed=!0,this._fn()}}};function _(e){return new g(e)}var v=class e{static{this.DISABLE_DISPOSED_WARNING=!1}constructor(){this._toDispose=/* @__PURE__ */ new Set,this._isDisposed=!1}dispose(){this._isDisposed||(this._isDisposed=!0,this.clear())}get isDisposed(){return this._isDisposed}clear(){if(this._toDispose.size!==0)try{m(this._toDispose)}finally{this._toDispose.clear()}}add(t){if(!t||t===y.None)return t;if(t===this)throw Error(`Cannot register a disposable on itself!`);return this._isDisposed?e.DISABLE_DISPOSED_WARNING||console.warn((/* @__PURE__ */ Error(`Trying to add a disposable to a DisposableStore that has already been disposed of. The added object will be leaked!`)).stack):this._toDispose.add(t),t}delete(e){if(e){if(e===this)throw Error(`Cannot dispose a disposable on itself!`);this._toDispose.delete(e),e.dispose()}}},y=class{static{this.None=Object.freeze({dispose(){}})}constructor(){this._store=new v,this._store}dispose(){this._store.dispose()}_register(e){if(e===this)throw Error(`Cannot register a disposable on itself!`);return this._store.add(e)}},b=class e{static{this.Undefined=new e(void 0)}constructor(t){this.element=t,this.next=e.Undefined,this.prev=e.Undefined}},x=class{constructor(){this._first=b.Undefined,this._last=b.Undefined,this._size=0}get size(){return this._size}isEmpty(){return this._first===b.Undefined}clear(){let e=this._first;for(;e!==b.Undefined;){let t=e.next;e.prev=b.Undefined,e.next=b.Undefined,e=t}this._first=b.Undefined,this._last=b.Undefined,this._size=0}unshift(e){return this._insert(e,!1)}push(e){return this._insert(e,!0)}_insert(e,t){let n=new b(e);if(this._first===b.Undefined)this._first=n,this._last=n;else if(t){let e=this._last;this._last=n,n.prev=e,e.next=n}else{let e=this._first;this._first=n,n.next=e,e.prev=n}this._size+=1;let r=!1;return()=>{r||(r=!0,this._remove(n))}}shift(){if(this._first!==b.Undefined){let e=this._first.element;return this._remove(this._first),e}}pop(){if(this._last!==b.Undefined){let e=this._last.element;return this._remove(this._last),e}}_remove(e){if(e.prev!==b.Undefined&&e.next!==b.Undefined){let t=e.prev;t.next=e.next,e.next.prev=t}else e.prev===b.Undefined&&e.next===b.Undefined?(this._first=b.Undefined,this._last=b.Undefined):e.next===b.Undefined?(this._last=this._last.prev,this._last.next=b.Undefined):e.prev===b.Undefined&&(this._first=this._first.next,this._first.prev=b.Undefined);--this._size}*[Symbol.iterator](){let e=this._first;for(;e!==b.Undefined;)yield e.element,e=e.next}};const S=globalThis.performance.now.bind(globalThis.performance);var C=class e{static create(t){return new e(t)}constructor(e){this._now=e===!1?Date.now:S,this._startTime=this._now(),this._stopTime=-1}stop(){this._stopTime=this._now()}reset(){this._startTime=this._now(),this._stopTime=-1}elapsed(){return this._stopTime===-1?this._now()-this._startTime:this._stopTime-this._startTime}},w;(function(e){e.None=()=>y.None;function t(e,t){return f(e,()=>void 0,0,void 0,!0,void 0,t)}e.defer=t;function n(e){return(t,n=null,r)=>{let i=!1,a;return a=e(e=>{if(!i)return a?a.dispose():i=!0,t.call(n,e)},null,r),i&&a.dispose(),a}}e.once=n;function r(t,n){return e.once(e.filter(t,n))}e.onceIf=r;function i(e,t,n){return u((n,r=null,i)=>e(e=>n.call(r,t(e)),null,i),n)}e.map=i;function a(e,t,n){return u((n,r=null,i)=>e(e=>{t(e),n.call(r,e)},null,i),n)}e.forEach=a;function o(e,t,n){return u((n,r=null,i)=>e(e=>t(e)&&n.call(r,e),null,i),n)}e.filter=o;function s(e){return e}e.signal=s;function c(...e){return(t,n=null,r)=>d(h(...e.map(e=>e(e=>t.call(n,e)))),r)}e.any=c;function l(e,t,n,r){let a=n;return i(e,e=>(a=t(a,e),a),r)}e.reduce=l;function u(e,t){let n,r=new E({onWillAddFirstListener(){n=e(r.fire,r)},onDidRemoveLastListener(){n?.dispose()}});return t?.add(r),r.event}function d(e,t){return t instanceof Array?t.push(e):t&&t.add(e),e}function f(e,t,n=100,r=!1,i=!1,a,o){let s,c,l,u=0,d,f=new E({leakWarningThreshold:a,onWillAddFirstListener(){s=e(e=>{u++,c=t(c,e),r&&!l&&(f.fire(c),c=void 0),d=()=>{let e=c;c=void 0,l=void 0,(!r||u>1)&&f.fire(e),u=0},typeof n==`number`?(l&&clearTimeout(l),l=setTimeout(d,n)):l===void 0&&(l=null,queueMicrotask(d))})},onWillRemoveListener(){i&&u>0&&d?.()},onDidRemoveLastListener(){d=void 0,s.dispose()}});return o?.add(f),f.event}e.debounce=f;function p(t,n=0,r){return e.debounce(t,(e,t)=>e?(e.push(t),e):[t],n,void 0,!0,void 0,r)}e.accumulate=p;function m(e,t=(e,t)=>e===t,n){let r=!0,i;return o(e,e=>{let n=r||!t(e,i);return r=!1,i=e,n},n)}e.latch=m;function g(t,n,r){return[e.filter(t,n,r),e.filter(t,e=>!n(e),r)]}e.split=g;function _(e,t=!1,n=[],r){let i=n.slice(),a=e(e=>{i?i.push(e):s.fire(e)});r&&r.add(a);let o=()=>{i?.forEach(e=>s.fire(e)),i=null},s=new E({onWillAddFirstListener(){a||(a=e(e=>s.fire(e)),r&&r.add(a))},onDidAddFirstListener(){i&&(t?setTimeout(o):o())},onDidRemoveLastListener(){a&&a.dispose(),a=null}});return r&&r.add(s),s.event}e.buffer=_;function b(e,t){return(n,r,i)=>{let a=t(new S);return e(function(e){let t=a.evaluate(e);t!==x&&n.call(r,t)},void 0,i)}}e.chain=b;let x=Symbol(`HaltChainable`);class S{constructor(){this.steps=[]}map(e){return this.steps.push(e),this}forEach(e){return this.steps.push(t=>(e(t),t)),this}filter(e){return this.steps.push(t=>e(t)?t:x),this}reduce(e,t){let n=t;return this.steps.push(t=>(n=e(n,t),n)),this}latch(e=(e,t)=>e===t){let t=!0,n;return this.steps.push(r=>{let i=t||!e(r,n);return t=!1,n=r,i?r:x}),this}evaluate(e){for(let t of this.steps)if(e=t(e),e===x)break;return e}}function C(e,t,n=e=>e){let r=(...e)=>i.fire(n(...e)),i=new E({onWillAddFirstListener:()=>e.on(t,r),onDidRemoveLastListener:()=>e.removeListener(t,r)});return i.event}e.fromNodeEventEmitter=C;function w(e,t,n=e=>e){let r=(...e)=>i.fire(n(...e)),i=new E({onWillAddFirstListener:()=>e.addEventListener(t,r),onDidRemoveLastListener:()=>e.removeEventListener(t,r)});return i.event}e.fromDOMEventEmitter=w;function T(e,t){let r,i=new Promise((i,a)=>{let o=n(e)(i,null,t);r=()=>o.dispose()});return i.cancel=r,i}e.toPromise=T;function ee(e,t){return e(e=>t.fire(e))}e.forward=ee;function te(e,t,n){return t(n),e(e=>t(e))}e.runAndSubscribe=te;class ne{constructor(e,t){this._observable=e,this._counter=0,this._hasChanged=!1;let n={onWillAddFirstListener:()=>{e.addObserver(this),this._observable.reportChanges()},onDidRemoveLastListener:()=>{e.removeObserver(this)}};this.emitter=new E(n),t&&t.add(this.emitter)}beginUpdate(e){this._counter++}handlePossibleChange(e){}handleChange(e,t){this._hasChanged=!0}endUpdate(e){this._counter--,this._counter===0&&(this._observable.reportChanges(),this._hasChanged&&(this._hasChanged=!1,this.emitter.fire(this._observable.get())))}}function re(e,t){return new ne(e,t).emitter.event}e.fromObservable=re;function ie(e){return(t,n,r)=>{let i=0,a=!1,o={beginUpdate(){i++},endUpdate(){i--,i===0&&(e.reportChanges(),a&&(a=!1,t.call(n)))},handlePossibleChange(){},handleChange(){a=!0}};e.addObserver(o),e.reportChanges();let s={dispose(){e.removeObserver(o)}};return r instanceof v?r.add(s):Array.isArray(r)&&r.push(s),s}}e.fromObservableLight=ie})(w||={});var T=class e{static{this.all=/* @__PURE__ */ new Set}static{this._idPool=0}constructor(t){this.listenerCount=0,this.invocationCount=0,this.elapsedOverall=0,this.durations=[],this.name=`${t}_${e._idPool++}`,e.all.add(this)}start(e){this._stopWatch=new C,this.listenerCount=e}stop(){if(this._stopWatch){let e=this._stopWatch.elapsed();this.durations.push(e),this.elapsedOverall+=e,this.invocationCount+=1,this._stopWatch=void 0}}},ee=class e{static{this._idPool=1}constructor(t,n,r=(e._idPool++).toString(16).padStart(3,`0`)){this._errorHandler=t,this.threshold=n,this.name=r,this._warnCountdown=0}dispose(){this._stacks?.clear()}check(e,t){let n=this.threshold;if(n<=0||t<n)return;this._stacks||=/* @__PURE__ */ new Map;let r=this._stacks.get(e.value)||0;if(this._stacks.set(e.value,r+1),--this._warnCountdown,this._warnCountdown<=0){this._warnCountdown=n*.5;let[e,r]=this.getMostFrequentStack(),i=`[${this.name}] potential listener LEAK detected, having ${t} listeners already. MOST frequent listener (${r}):`;console.warn(i),console.warn(e);let a=new ne(i,e);this._errorHandler(a)}return()=>{let t=this._stacks.get(e.value)||0;this._stacks.set(e.value,t-1)}}getMostFrequentStack(){if(!this._stacks)return;let e,t=0;for(let[n,r]of this._stacks)(!e||t<r)&&(e=[n,r],t=r);return e}},te=class e{static create(){return new e((/* @__PURE__ */ Error()).stack??``)}constructor(e){this.value=e}print(){console.warn(this.value.split(`\n`).slice(2).join(`\n`))}},ne=class extends Error{constructor(e,t){super(e),this.name=`ListenerLeakError`,this.stack=t}},re=class extends Error{constructor(e,t){super(e),this.name=`ListenerRefusalError`,this.stack=t}},ie=class{constructor(e){this.value=e}},E=class{constructor(e){this._size=0,this._options=e,this._leakageMon=this._options?.leakWarningThreshold?new ee(e?.onListenerError??t,this._options?.leakWarningThreshold??-1):void 0,this._perfMon=this._options?._profName?new T(this._options._profName):void 0,this._deliveryQueue=this._options?.deliveryQueue}dispose(){this._disposed||(this._disposed=!0,this._deliveryQueue?.current===this&&this._deliveryQueue.reset(),this._listeners&&(this._listeners=void 0,this._size=0),this._options?.onDidRemoveLastListener?.(),this._leakageMon?.dispose())}get event(){return this._event??=(e,n,r)=>{if(this._leakageMon&&this._size>this._leakageMon.threshold**2){let e=`[${this._leakageMon.name}] REFUSES to accept new listeners because it exceeded its threshold by far (${this._size} vs ${this._leakageMon.threshold})`;console.warn(e);let n=this._leakageMon.getMostFrequentStack()??[`UNKNOWN stack`,-1],r=new re(`${e}. HINT: Stack shows most frequent listener (${n[1]}-times)`,n[0]);return(this._options?.onListenerError||t)(r),y.None}if(this._disposed)return y.None;n&&(e=e.bind(n));let i=new ie(e),a;this._leakageMon&&this._size>=Math.ceil(this._leakageMon.threshold*.2)&&(i.stack=te.create(),a=this._leakageMon.check(i.stack,this._size+1)),this._listeners?this._listeners instanceof ie?(this._deliveryQueue??=new ae,this._listeners=[this._listeners,i]):this._listeners.push(i):(this._options?.onWillAddFirstListener?.(this),this._listeners=i,this._options?.onDidAddFirstListener?.(this)),this._options?.onDidAddListener?.(this),this._size++;let o=_(()=>{a?.(),this._removeListener(i)});return r instanceof v?r.add(o):Array.isArray(r)&&r.push(o),o},this._event}_removeListener(e){if(this._options?.onWillRemoveListener?.(this),!this._listeners)return;if(this._size===1){this._listeners=void 0,this._options?.onDidRemoveLastListener?.(this),this._size=0;return}let t=this._listeners,n=t.indexOf(e);if(n===-1)throw console.log(`disposed?`,this._disposed),console.log(`size?`,this._size),console.log(`arr?`,JSON.stringify(this._listeners)),Error(`Attempted to dispose unknown listener`);this._size--,t[n]=void 0;let r=this._deliveryQueue.current===this;if(this._size*2<=t.length){let e=0;for(let n=0;n<t.length;n++)t[n]?t[e++]=t[n]:r&&e<this._deliveryQueue.end&&(this._deliveryQueue.end--,e<this._deliveryQueue.i&&this._deliveryQueue.i--);t.length=e}}_deliver(e,n){if(!e)return;let r=this._options?.onListenerError||t;if(!r)e.value(n);else try{e.value(n)}catch(e){r(e)}}_deliverQueue(e){let t=e.current._listeners;for(;e.i<e.end;)this._deliver(t[e.i++],e.value);e.reset()}fire(e){if(this._deliveryQueue?.current&&(this._deliverQueue(this._deliveryQueue),this._perfMon?.stop()),this._perfMon?.start(this._size),this._listeners){if(this._listeners instanceof ie)this._deliver(this._listeners,e);else{let t=this._deliveryQueue;t.enqueue(this,e,this._listeners.length),this._deliverQueue(t)}}this._perfMon?.stop()}hasListeners(){return this._size>0}},ae=class{constructor(){this.i=-1,this.end=0}enqueue(e,t,n){this.i=0,this.end=n,this.current=e,this.value=t}reset(){this.i=this.end,this.current=void 0,this.value=void 0}};function oe(){return globalThis._VSCODE_NLS_MESSAGES}function se(){return globalThis._VSCODE_NLS_LANGUAGE}const ce=se()===`pseudo`||typeof document<`u`&&document.location&&typeof document.location.hash==`string`&&document.location.hash.indexOf(`pseudo=true`)>=0;function le(e,t){let n;return n=t.length===0?e:e.replace(/\\{(\\d+)\\}/g,(e,n)=>{let r=t[n[0]],i=e;return typeof r==`string`?i=r:(typeof r==`number`||typeof r==`boolean`||r==null)&&(i=String(r)),i}),ce&&(n=`［`+n.replace(/[aouei]/g,`$&$&`)+`］`),n}function D(e,t,...n){return le(typeof e==`number`?ue(e,t):t,n)}function ue(e,t){let n=oe()?.[e];if(typeof n!=`string`){if(typeof t==`string`)return t;throw Error(`!!! NLS MISSING: ${e} !!!`)}return n}let de=!1,fe=!1,pe=!1,me=!1,O;const k=globalThis;let A;k.vscode!==void 0&&k.vscode.process!==void 0?A=k.vscode.process:typeof process<`u`&&typeof process?.versions?.node==`string`&&(A=process);const he=typeof A?.versions?.electron==`string`&&A?.type===`renderer`;if(typeof A==`object`){de=A.platform===`win32`,fe=A.platform===`darwin`,pe=A.platform===`linux`,pe&&A.env.SNAP&&A.env.SNAP_REVISION,A.env.CI||A.env.BUILD_ARTIFACTSTAGINGDIRECTORY||A.env.GITHUB_WORKSPACE;let e=A.env.VSCODE_NLS_CONFIG;if(e)try{let t=JSON.parse(e);t.userLocale,t.osLocale,t.resolvedLanguage,t.languagePack?.translationsConfigFile}catch{}}else typeof navigator==`object`&&!he?(O=navigator.userAgent,de=O.indexOf(`Windows`)>=0,fe=O.indexOf(`Macintosh`)>=0,(O.indexOf(`Macintosh`)>=0||O.indexOf(`iPad`)>=0||O.indexOf(`iPhone`)>=0)&&navigator.maxTouchPoints&&navigator.maxTouchPoints,pe=O.indexOf(`Linux`)>=0,O?.indexOf(`Mobi`),me=!0,se(),navigator.language.toLowerCase()):console.error(`Unable to resolve platform.`);const ge=de,_e=fe;me&&typeof k.importScripts==`function`&&k.origin;const j=O,ve=typeof k.postMessage==`function`&&!k.importScripts;(()=>{if(ve){let e=[];k.addEventListener(`message`,t=>{if(t.data&&t.data.vscodeScheduleAsyncWork)for(let n=0,r=e.length;n<r;n++){let r=e[n];if(r.id===t.data.vscodeScheduleAsyncWork){e.splice(n,1),r.callback();return}}});let t=0;return n=>{let r=++t;e.push({id:r,callback:n}),k.postMessage({vscodeScheduleAsyncWork:r},`*`)}}return e=>setTimeout(e)})();const ye=!!(j&&j.indexOf(`Chrome`)>=0);j&&j.indexOf(`Firefox`),!ye&&j&&j.indexOf(`Safari`),j&&j.indexOf(`Edg/`),j&&j.indexOf(`Android`);function be(e){return e}var xe=class{constructor(e,t){this.lastCache=void 0,this.lastArgKey=void 0,typeof e==`function`?(this._fn=e,this._computeKey=be):(this._fn=t,this._computeKey=e.getCacheKey)}get(e){let t=this._computeKey(e);return this.lastArgKey!==t&&(this.lastArgKey=t,this.lastCache=this._fn(e)),this.lastCache}},M;(function(e){e[e.Uninitialized=0]=`Uninitialized`,e[e.Running=1]=`Running`,e[e.Completed=2]=`Completed`})(M||={});var Se=class{constructor(e){this.executor=e,this._state=M.Uninitialized}get value(){if(this._state===M.Uninitialized){this._state=M.Running;try{this._value=this.executor()}catch(e){this._error=e}finally{this._state=M.Completed}}else if(this._state===M.Running)throw Error(`Cannot read the value of a lazy that is being initialized`);if(this._error)throw this._error;return this._value}get rawValue(){return this._value}};function Ce(e){return e.replace(/[\\\\\\{\\}\\*\\+\\?\\|\\^\\$\\.\\[\\]\\(\\)]/g,`\\\\$&`)}function we(e){return e.source===`^`||e.source===`^$`||e.source===`$`||e.source===`^\\\\s*$`?!1:!!(e.exec(``)&&e.lastIndex===0)}function Te(e){return e.split(/\\r\\n|\\r|\\n/)}function Ee(e){for(let t=0,n=e.length;t<n;t++){let n=e.charCodeAt(t);if(n!==32&&n!==9)return t}return-1}function De(e,t=e.length-1){for(let n=t;n>=0;n--){let t=e.charCodeAt(n);if(t!==32&&t!==9)return n}return-1}function Oe(e){return e>=65&&e<=90}function ke(e,t){let n=Math.min(e.length,t.length),r=0;for(;r<n;r++)if(e.charCodeAt(r)!==t.charCodeAt(r))return r;return n}function Ae(e,t){let n=Math.min(e.length,t.length),r,i=e.length-1,a=t.length-1;for(r=0;r<n;r++)if(e.charCodeAt(i-r)!==t.charCodeAt(a-r))return r;return n}function je(e){return 55296<=e&&e<=56319}function Me(e){return 56320<=e&&e<=57343}function Ne(e,t){return(e-55296<<10)+(t-56320)+65536}function Pe(e,t,n){let r=e.charCodeAt(n);if(je(r)&&n+1<t){let t=e.charCodeAt(n+1);if(Me(t))return Ne(r,t)}return r}const Fe=/^[\\t\\n\\r\\x20-\\x7E]*$/;function Ie(e){return Fe.test(e)}(class e{static{this._INSTANCE=null}static getInstance(){return e._INSTANCE||=new e,e._INSTANCE}constructor(){this._data=Le()}getGraphemeBreakType(e){if(e<32)return e===10?3:e===13?2:4;if(e<127)return 0;let t=this._data,n=t.length/3,r=1;for(;r<=n;)if(e<t[3*r])r=2*r;else if(e>t[3*r+1])r=2*r+1;else return t[3*r+2];return 0}});function Le(){return JSON.parse(`[0,0,0,51229,51255,12,44061,44087,12,127462,127487,6,7083,7085,5,47645,47671,12,54813,54839,12,128678,128678,14,3270,3270,5,9919,9923,14,45853,45879,12,49437,49463,12,53021,53047,12,71216,71218,7,128398,128399,14,129360,129374,14,2519,2519,5,4448,4519,9,9742,9742,14,12336,12336,14,44957,44983,12,46749,46775,12,48541,48567,12,50333,50359,12,52125,52151,12,53917,53943,12,69888,69890,5,73018,73018,5,127990,127990,14,128558,128559,14,128759,128760,14,129653,129655,14,2027,2035,5,2891,2892,7,3761,3761,5,6683,6683,5,8293,8293,4,9825,9826,14,9999,9999,14,43452,43453,5,44509,44535,12,45405,45431,12,46301,46327,12,47197,47223,12,48093,48119,12,48989,49015,12,49885,49911,12,50781,50807,12,51677,51703,12,52573,52599,12,53469,53495,12,54365,54391,12,65279,65279,4,70471,70472,7,72145,72147,7,119173,119179,5,127799,127818,14,128240,128244,14,128512,128512,14,128652,128652,14,128721,128722,14,129292,129292,14,129445,129450,14,129734,129743,14,1476,1477,5,2366,2368,7,2750,2752,7,3076,3076,5,3415,3415,5,4141,4144,5,6109,6109,5,6964,6964,5,7394,7400,5,9197,9198,14,9770,9770,14,9877,9877,14,9968,9969,14,10084,10084,14,43052,43052,5,43713,43713,5,44285,44311,12,44733,44759,12,45181,45207,12,45629,45655,12,46077,46103,12,46525,46551,12,46973,46999,12,47421,47447,12,47869,47895,12,48317,48343,12,48765,48791,12,49213,49239,12,49661,49687,12,50109,50135,12,50557,50583,12,51005,51031,12,51453,51479,12,51901,51927,12,52349,52375,12,52797,52823,12,53245,53271,12,53693,53719,12,54141,54167,12,54589,54615,12,55037,55063,12,69506,69509,5,70191,70193,5,70841,70841,7,71463,71467,5,72330,72342,5,94031,94031,5,123628,123631,5,127763,127765,14,127941,127941,14,128043,128062,14,128302,128317,14,128465,128467,14,128539,128539,14,128640,128640,14,128662,128662,14,128703,128703,14,128745,128745,14,129004,129007,14,129329,129330,14,129402,129402,14,129483,129483,14,129686,129704,14,130048,131069,14,173,173,4,1757,1757,1,2200,2207,5,2434,2435,7,2631,2632,5,2817,2817,5,3008,3008,5,3201,3201,5,3387,3388,5,3542,3542,5,3902,3903,7,4190,4192,5,6002,6003,5,6439,6440,5,6765,6770,7,7019,7027,5,7154,7155,7,8205,8205,13,8505,8505,14,9654,9654,14,9757,9757,14,9792,9792,14,9852,9853,14,9890,9894,14,9937,9937,14,9981,9981,14,10035,10036,14,11035,11036,14,42654,42655,5,43346,43347,7,43587,43587,5,44006,44007,7,44173,44199,12,44397,44423,12,44621,44647,12,44845,44871,12,45069,45095,12,45293,45319,12,45517,45543,12,45741,45767,12,45965,45991,12,46189,46215,12,46413,46439,12,46637,46663,12,46861,46887,12,47085,47111,12,47309,47335,12,47533,47559,12,47757,47783,12,47981,48007,12,48205,48231,12,48429,48455,12,48653,48679,12,48877,48903,12,49101,49127,12,49325,49351,12,49549,49575,12,49773,49799,12,49997,50023,12,50221,50247,12,50445,50471,12,50669,50695,12,50893,50919,12,51117,51143,12,51341,51367,12,51565,51591,12,51789,51815,12,52013,52039,12,52237,52263,12,52461,52487,12,52685,52711,12,52909,52935,12,53133,53159,12,53357,53383,12,53581,53607,12,53805,53831,12,54029,54055,12,54253,54279,12,54477,54503,12,54701,54727,12,54925,54951,12,55149,55175,12,68101,68102,5,69762,69762,7,70067,70069,7,70371,70378,5,70720,70721,7,71087,71087,5,71341,71341,5,71995,71996,5,72249,72249,7,72850,72871,5,73109,73109,5,118576,118598,5,121505,121519,5,127245,127247,14,127568,127569,14,127777,127777,14,127872,127891,14,127956,127967,14,128015,128016,14,128110,128172,14,128259,128259,14,128367,128368,14,128424,128424,14,128488,128488,14,128530,128532,14,128550,128551,14,128566,128566,14,128647,128647,14,128656,128656,14,128667,128673,14,128691,128693,14,128715,128715,14,128728,128732,14,128752,128752,14,128765,128767,14,129096,129103,14,129311,129311,14,129344,129349,14,129394,129394,14,129413,129425,14,129466,129471,14,129511,129535,14,129664,129666,14,129719,129722,14,129760,129767,14,917536,917631,5,13,13,2,1160,1161,5,1564,1564,4,1807,1807,1,2085,2087,5,2307,2307,7,2382,2383,7,2497,2500,5,2563,2563,7,2677,2677,5,2763,2764,7,2879,2879,5,2914,2915,5,3021,3021,5,3142,3144,5,3263,3263,5,3285,3286,5,3398,3400,7,3530,3530,5,3633,3633,5,3864,3865,5,3974,3975,5,4155,4156,7,4229,4230,5,5909,5909,7,6078,6085,7,6277,6278,5,6451,6456,7,6744,6750,5,6846,6846,5,6972,6972,5,7074,7077,5,7146,7148,7,7222,7223,5,7416,7417,5,8234,8238,4,8417,8417,5,9000,9000,14,9203,9203,14,9730,9731,14,9748,9749,14,9762,9763,14,9776,9783,14,9800,9811,14,9831,9831,14,9872,9873,14,9882,9882,14,9900,9903,14,9929,9933,14,9941,9960,14,9974,9974,14,9989,9989,14,10006,10006,14,10062,10062,14,10160,10160,14,11647,11647,5,12953,12953,14,43019,43019,5,43232,43249,5,43443,43443,5,43567,43568,7,43696,43696,5,43765,43765,7,44013,44013,5,44117,44143,12,44229,44255,12,44341,44367,12,44453,44479,12,44565,44591,12,44677,44703,12,44789,44815,12,44901,44927,12,45013,45039,12,45125,45151,12,45237,45263,12,45349,45375,12,45461,45487,12,45573,45599,12,45685,45711,12,45797,45823,12,45909,45935,12,46021,46047,12,46133,46159,12,46245,46271,12,46357,46383,12,46469,46495,12,46581,46607,12,46693,46719,12,46805,46831,12,46917,46943,12,47029,47055,12,47141,47167,12,47253,47279,12,47365,47391,12,47477,47503,12,47589,47615,12,47701,47727,12,47813,47839,12,47925,47951,12,48037,48063,12,48149,48175,12,48261,48287,12,48373,48399,12,48485,48511,12,48597,48623,12,48709,48735,12,48821,48847,12,48933,48959,12,49045,49071,12,49157,49183,12,49269,49295,12,49381,49407,12,49493,49519,12,49605,49631,12,49717,49743,12,49829,49855,12,49941,49967,12,50053,50079,12,50165,50191,12,50277,50303,12,50389,50415,12,50501,50527,12,50613,50639,12,50725,50751,12,50837,50863,12,50949,50975,12,51061,51087,12,51173,51199,12,51285,51311,12,51397,51423,12,51509,51535,12,51621,51647,12,51733,51759,12,51845,51871,12,51957,51983,12,52069,52095,12,52181,52207,12,52293,52319,12,52405,52431,12,52517,52543,12,52629,52655,12,52741,52767,12,52853,52879,12,52965,52991,12,53077,53103,12,53189,53215,12,53301,53327,12,53413,53439,12,53525,53551,12,53637,53663,12,53749,53775,12,53861,53887,12,53973,53999,12,54085,54111,12,54197,54223,12,54309,54335,12,54421,54447,12,54533,54559,12,54645,54671,12,54757,54783,12,54869,54895,12,54981,55007,12,55093,55119,12,55243,55291,10,66045,66045,5,68325,68326,5,69688,69702,5,69817,69818,5,69957,69958,7,70089,70092,5,70198,70199,5,70462,70462,5,70502,70508,5,70750,70750,5,70846,70846,7,71100,71101,5,71230,71230,7,71351,71351,5,71737,71738,5,72000,72000,7,72160,72160,5,72273,72278,5,72752,72758,5,72882,72883,5,73031,73031,5,73461,73462,7,94192,94193,7,119149,119149,7,121403,121452,5,122915,122916,5,126980,126980,14,127358,127359,14,127535,127535,14,127759,127759,14,127771,127771,14,127792,127793,14,127825,127867,14,127897,127899,14,127945,127945,14,127985,127986,14,128000,128007,14,128021,128021,14,128066,128100,14,128184,128235,14,128249,128252,14,128266,128276,14,128335,128335,14,128379,128390,14,128407,128419,14,128444,128444,14,128481,128481,14,128499,128499,14,128526,128526,14,128536,128536,14,128543,128543,14,128556,128556,14,128564,128564,14,128577,128580,14,128643,128645,14,128649,128649,14,128654,128654,14,128660,128660,14,128664,128664,14,128675,128675,14,128686,128689,14,128695,128696,14,128705,128709,14,128717,128719,14,128725,128725,14,128736,128741,14,128747,128748,14,128755,128755,14,128762,128762,14,128981,128991,14,129009,129023,14,129160,129167,14,129296,129304,14,129320,129327,14,129340,129342,14,129356,129356,14,129388,129392,14,129399,129400,14,129404,129407,14,129432,129442,14,129454,129455,14,129473,129474,14,129485,129487,14,129648,129651,14,129659,129660,14,129671,129679,14,129709,129711,14,129728,129730,14,129751,129753,14,129776,129782,14,917505,917505,4,917760,917999,5,10,10,3,127,159,4,768,879,5,1471,1471,5,1536,1541,1,1648,1648,5,1767,1768,5,1840,1866,5,2070,2073,5,2137,2139,5,2274,2274,1,2363,2363,7,2377,2380,7,2402,2403,5,2494,2494,5,2507,2508,7,2558,2558,5,2622,2624,7,2641,2641,5,2691,2691,7,2759,2760,5,2786,2787,5,2876,2876,5,2881,2884,5,2901,2902,5,3006,3006,5,3014,3016,7,3072,3072,5,3134,3136,5,3157,3158,5,3260,3260,5,3266,3266,5,3274,3275,7,3328,3329,5,3391,3392,7,3405,3405,5,3457,3457,5,3536,3537,7,3551,3551,5,3636,3642,5,3764,3772,5,3895,3895,5,3967,3967,7,3993,4028,5,4146,4151,5,4182,4183,7,4226,4226,5,4253,4253,5,4957,4959,5,5940,5940,7,6070,6070,7,6087,6088,7,6158,6158,4,6432,6434,5,6448,6449,7,6679,6680,5,6742,6742,5,6754,6754,5,6783,6783,5,6912,6915,5,6966,6970,5,6978,6978,5,7042,7042,7,7080,7081,5,7143,7143,7,7150,7150,7,7212,7219,5,7380,7392,5,7412,7412,5,8203,8203,4,8232,8232,4,8265,8265,14,8400,8412,5,8421,8432,5,8617,8618,14,9167,9167,14,9200,9200,14,9410,9410,14,9723,9726,14,9733,9733,14,9745,9745,14,9752,9752,14,9760,9760,14,9766,9766,14,9774,9774,14,9786,9786,14,9794,9794,14,9823,9823,14,9828,9828,14,9833,9850,14,9855,9855,14,9875,9875,14,9880,9880,14,9885,9887,14,9896,9897,14,9906,9916,14,9926,9927,14,9935,9935,14,9939,9939,14,9962,9962,14,9972,9972,14,9978,9978,14,9986,9986,14,9997,9997,14,10002,10002,14,10017,10017,14,10055,10055,14,10071,10071,14,10133,10135,14,10548,10549,14,11093,11093,14,12330,12333,5,12441,12442,5,42608,42610,5,43010,43010,5,43045,43046,5,43188,43203,7,43302,43309,5,43392,43394,5,43446,43449,5,43493,43493,5,43571,43572,7,43597,43597,7,43703,43704,5,43756,43757,5,44003,44004,7,44009,44010,7,44033,44059,12,44089,44115,12,44145,44171,12,44201,44227,12,44257,44283,12,44313,44339,12,44369,44395,12,44425,44451,12,44481,44507,12,44537,44563,12,44593,44619,12,44649,44675,12,44705,44731,12,44761,44787,12,44817,44843,12,44873,44899,12,44929,44955,12,44985,45011,12,45041,45067,12,45097,45123,12,45153,45179,12,45209,45235,12,45265,45291,12,45321,45347,12,45377,45403,12,45433,45459,12,45489,45515,12,45545,45571,12,45601,45627,12,45657,45683,12,45713,45739,12,45769,45795,12,45825,45851,12,45881,45907,12,45937,45963,12,45993,46019,12,46049,46075,12,46105,46131,12,46161,46187,12,46217,46243,12,46273,46299,12,46329,46355,12,46385,46411,12,46441,46467,12,46497,46523,12,46553,46579,12,46609,46635,12,46665,46691,12,46721,46747,12,46777,46803,12,46833,46859,12,46889,46915,12,46945,46971,12,47001,47027,12,47057,47083,12,47113,47139,12,47169,47195,12,47225,47251,12,47281,47307,12,47337,47363,12,47393,47419,12,47449,47475,12,47505,47531,12,47561,47587,12,47617,47643,12,47673,47699,12,47729,47755,12,47785,47811,12,47841,47867,12,47897,47923,12,47953,47979,12,48009,48035,12,48065,48091,12,48121,48147,12,48177,48203,12,48233,48259,12,48289,48315,12,48345,48371,12,48401,48427,12,48457,48483,12,48513,48539,12,48569,48595,12,48625,48651,12,48681,48707,12,48737,48763,12,48793,48819,12,48849,48875,12,48905,48931,12,48961,48987,12,49017,49043,12,49073,49099,12,49129,49155,12,49185,49211,12,49241,49267,12,49297,49323,12,49353,49379,12,49409,49435,12,49465,49491,12,49521,49547,12,49577,49603,12,49633,49659,12,49689,49715,12,49745,49771,12,49801,49827,12,49857,49883,12,49913,49939,12,49969,49995,12,50025,50051,12,50081,50107,12,50137,50163,12,50193,50219,12,50249,50275,12,50305,50331,12,50361,50387,12,50417,50443,12,50473,50499,12,50529,50555,12,50585,50611,12,50641,50667,12,50697,50723,12,50753,50779,12,50809,50835,12,50865,50891,12,50921,50947,12,50977,51003,12,51033,51059,12,51089,51115,12,51145,51171,12,51201,51227,12,51257,51283,12,51313,51339,12,51369,51395,12,51425,51451,12,51481,51507,12,51537,51563,12,51593,51619,12,51649,51675,12,51705,51731,12,51761,51787,12,51817,51843,12,51873,51899,12,51929,51955,12,51985,52011,12,52041,52067,12,52097,52123,12,52153,52179,12,52209,52235,12,52265,52291,12,52321,52347,12,52377,52403,12,52433,52459,12,52489,52515,12,52545,52571,12,52601,52627,12,52657,52683,12,52713,52739,12,52769,52795,12,52825,52851,12,52881,52907,12,52937,52963,12,52993,53019,12,53049,53075,12,53105,53131,12,53161,53187,12,53217,53243,12,53273,53299,12,53329,53355,12,53385,53411,12,53441,53467,12,53497,53523,12,53553,53579,12,53609,53635,12,53665,53691,12,53721,53747,12,53777,53803,12,53833,53859,12,53889,53915,12,53945,53971,12,54001,54027,12,54057,54083,12,54113,54139,12,54169,54195,12,54225,54251,12,54281,54307,12,54337,54363,12,54393,54419,12,54449,54475,12,54505,54531,12,54561,54587,12,54617,54643,12,54673,54699,12,54729,54755,12,54785,54811,12,54841,54867,12,54897,54923,12,54953,54979,12,55009,55035,12,55065,55091,12,55121,55147,12,55177,55203,12,65024,65039,5,65520,65528,4,66422,66426,5,68152,68154,5,69291,69292,5,69633,69633,5,69747,69748,5,69811,69814,5,69826,69826,5,69932,69932,7,70016,70017,5,70079,70080,7,70095,70095,5,70196,70196,5,70367,70367,5,70402,70403,7,70464,70464,5,70487,70487,5,70709,70711,7,70725,70725,7,70833,70834,7,70843,70844,7,70849,70849,7,71090,71093,5,71103,71104,5,71227,71228,7,71339,71339,5,71344,71349,5,71458,71461,5,71727,71735,5,71985,71989,7,71998,71998,5,72002,72002,7,72154,72155,5,72193,72202,5,72251,72254,5,72281,72283,5,72344,72345,5,72766,72766,7,72874,72880,5,72885,72886,5,73023,73029,5,73104,73105,5,73111,73111,5,92912,92916,5,94095,94098,5,113824,113827,4,119142,119142,7,119155,119162,4,119362,119364,5,121476,121476,5,122888,122904,5,123184,123190,5,125252,125258,5,127183,127183,14,127340,127343,14,127377,127386,14,127491,127503,14,127548,127551,14,127744,127756,14,127761,127761,14,127769,127769,14,127773,127774,14,127780,127788,14,127796,127797,14,127820,127823,14,127869,127869,14,127894,127895,14,127902,127903,14,127943,127943,14,127947,127950,14,127972,127972,14,127988,127988,14,127992,127994,14,128009,128011,14,128019,128019,14,128023,128041,14,128064,128064,14,128102,128107,14,128174,128181,14,128238,128238,14,128246,128247,14,128254,128254,14,128264,128264,14,128278,128299,14,128329,128330,14,128348,128359,14,128371,128377,14,128392,128393,14,128401,128404,14,128421,128421,14,128433,128434,14,128450,128452,14,128476,128478,14,128483,128483,14,128495,128495,14,128506,128506,14,128519,128520,14,128528,128528,14,128534,128534,14,128538,128538,14,128540,128542,14,128544,128549,14,128552,128555,14,128557,128557,14,128560,128563,14,128565,128565,14,128567,128576,14,128581,128591,14,128641,128642,14,128646,128646,14,128648,128648,14,128650,128651,14,128653,128653,14,128655,128655,14,128657,128659,14,128661,128661,14,128663,128663,14,128665,128666,14,128674,128674,14,128676,128677,14,128679,128685,14,128690,128690,14,128694,128694,14,128697,128702,14,128704,128704,14,128710,128714,14,128716,128716,14,128720,128720,14,128723,128724,14,128726,128727,14,128733,128735,14,128742,128744,14,128746,128746,14,128749,128751,14,128753,128754,14,128756,128758,14,128761,128761,14,128763,128764,14,128884,128895,14,128992,129003,14,129008,129008,14,129036,129039,14,129114,129119,14,129198,129279,14,129293,129295,14,129305,129310,14,129312,129319,14,129328,129328,14,129331,129338,14,129343,129343,14,129351,129355,14,129357,129359,14,129375,129387,14,129393,129393,14,129395,129398,14,129401,129401,14,129403,129403,14,129408,129412,14,129426,129431,14,129443,129444,14,129451,129453,14,129456,129465,14,129472,129472,14,129475,129482,14,129484,129484,14,129488,129510,14,129536,129647,14,129652,129652,14,129656,129658,14,129661,129663,14,129667,129670,14,129680,129685,14,129705,129708,14,129712,129718,14,129723,129727,14,129731,129733,14,129744,129750,14,129754,129759,14,129768,129775,14,129783,129791,14,917504,917504,4,917506,917535,4,917632,917759,4,918000,921599,4,0,9,4,11,12,4,14,31,4,169,169,14,174,174,14,1155,1159,5,1425,1469,5,1473,1474,5,1479,1479,5,1552,1562,5,1611,1631,5,1750,1756,5,1759,1764,5,1770,1773,5,1809,1809,5,1958,1968,5,2045,2045,5,2075,2083,5,2089,2093,5,2192,2193,1,2250,2273,5,2275,2306,5,2362,2362,5,2364,2364,5,2369,2376,5,2381,2381,5,2385,2391,5,2433,2433,5,2492,2492,5,2495,2496,7,2503,2504,7,2509,2509,5,2530,2531,5,2561,2562,5,2620,2620,5,2625,2626,5,2635,2637,5,2672,2673,5,2689,2690,5,2748,2748,5,2753,2757,5,2761,2761,7,2765,2765,5,2810,2815,5,2818,2819,7,2878,2878,5,2880,2880,7,2887,2888,7,2893,2893,5,2903,2903,5,2946,2946,5,3007,3007,7,3009,3010,7,3018,3020,7,3031,3031,5,3073,3075,7,3132,3132,5,3137,3140,7,3146,3149,5,3170,3171,5,3202,3203,7,3262,3262,7,3264,3265,7,3267,3268,7,3271,3272,7,3276,3277,5,3298,3299,5,3330,3331,7,3390,3390,5,3393,3396,5,3402,3404,7,3406,3406,1,3426,3427,5,3458,3459,7,3535,3535,5,3538,3540,5,3544,3550,7,3570,3571,7,3635,3635,7,3655,3662,5,3763,3763,7,3784,3789,5,3893,3893,5,3897,3897,5,3953,3966,5,3968,3972,5,3981,3991,5,4038,4038,5,4145,4145,7,4153,4154,5,4157,4158,5,4184,4185,5,4209,4212,5,4228,4228,7,4237,4237,5,4352,4447,8,4520,4607,10,5906,5908,5,5938,5939,5,5970,5971,5,6068,6069,5,6071,6077,5,6086,6086,5,6089,6099,5,6155,6157,5,6159,6159,5,6313,6313,5,6435,6438,7,6441,6443,7,6450,6450,5,6457,6459,5,6681,6682,7,6741,6741,7,6743,6743,7,6752,6752,5,6757,6764,5,6771,6780,5,6832,6845,5,6847,6862,5,6916,6916,7,6965,6965,5,6971,6971,7,6973,6977,7,6979,6980,7,7040,7041,5,7073,7073,7,7078,7079,7,7082,7082,7,7142,7142,5,7144,7145,5,7149,7149,5,7151,7153,5,7204,7211,7,7220,7221,7,7376,7378,5,7393,7393,7,7405,7405,5,7415,7415,7,7616,7679,5,8204,8204,5,8206,8207,4,8233,8233,4,8252,8252,14,8288,8292,4,8294,8303,4,8413,8416,5,8418,8420,5,8482,8482,14,8596,8601,14,8986,8987,14,9096,9096,14,9193,9196,14,9199,9199,14,9201,9202,14,9208,9210,14,9642,9643,14,9664,9664,14,9728,9729,14,9732,9732,14,9735,9741,14,9743,9744,14,9746,9746,14,9750,9751,14,9753,9756,14,9758,9759,14,9761,9761,14,9764,9765,14,9767,9769,14,9771,9773,14,9775,9775,14,9784,9785,14,9787,9791,14,9793,9793,14,9795,9799,14,9812,9822,14,9824,9824,14,9827,9827,14,9829,9830,14,9832,9832,14,9851,9851,14,9854,9854,14,9856,9861,14,9874,9874,14,9876,9876,14,9878,9879,14,9881,9881,14,9883,9884,14,9888,9889,14,9895,9895,14,9898,9899,14,9904,9905,14,9917,9918,14,9924,9925,14,9928,9928,14,9934,9934,14,9936,9936,14,9938,9938,14,9940,9940,14,9961,9961,14,9963,9967,14,9970,9971,14,9973,9973,14,9975,9977,14,9979,9980,14,9982,9985,14,9987,9988,14,9992,9996,14,9998,9998,14,10000,10001,14,10004,10004,14,10013,10013,14,10024,10024,14,10052,10052,14,10060,10060,14,10067,10069,14,10083,10083,14,10085,10087,14,10145,10145,14,10175,10175,14,11013,11015,14,11088,11088,14,11503,11505,5,11744,11775,5,12334,12335,5,12349,12349,14,12951,12951,14,42607,42607,5,42612,42621,5,42736,42737,5,43014,43014,5,43043,43044,7,43047,43047,7,43136,43137,7,43204,43205,5,43263,43263,5,43335,43345,5,43360,43388,8,43395,43395,7,43444,43445,7,43450,43451,7,43454,43456,7,43561,43566,5,43569,43570,5,43573,43574,5,43596,43596,5,43644,43644,5,43698,43700,5,43710,43711,5,43755,43755,7,43758,43759,7,43766,43766,5,44005,44005,5,44008,44008,5,44012,44012,7,44032,44032,11,44060,44060,11,44088,44088,11,44116,44116,11,44144,44144,11,44172,44172,11,44200,44200,11,44228,44228,11,44256,44256,11,44284,44284,11,44312,44312,11,44340,44340,11,44368,44368,11,44396,44396,11,44424,44424,11,44452,44452,11,44480,44480,11,44508,44508,11,44536,44536,11,44564,44564,11,44592,44592,11,44620,44620,11,44648,44648,11,44676,44676,11,44704,44704,11,44732,44732,11,44760,44760,11,44788,44788,11,44816,44816,11,44844,44844,11,44872,44872,11,44900,44900,11,44928,44928,11,44956,44956,11,44984,44984,11,45012,45012,11,45040,45040,11,45068,45068,11,45096,45096,11,45124,45124,11,45152,45152,11,45180,45180,11,45208,45208,11,45236,45236,11,45264,45264,11,45292,45292,11,45320,45320,11,45348,45348,11,45376,45376,11,45404,45404,11,45432,45432,11,45460,45460,11,45488,45488,11,45516,45516,11,45544,45544,11,45572,45572,11,45600,45600,11,45628,45628,11,45656,45656,11,45684,45684,11,45712,45712,11,45740,45740,11,45768,45768,11,45796,45796,11,45824,45824,11,45852,45852,11,45880,45880,11,45908,45908,11,45936,45936,11,45964,45964,11,45992,45992,11,46020,46020,11,46048,46048,11,46076,46076,11,46104,46104,11,46132,46132,11,46160,46160,11,46188,46188,11,46216,46216,11,46244,46244,11,46272,46272,11,46300,46300,11,46328,46328,11,46356,46356,11,46384,46384,11,46412,46412,11,46440,46440,11,46468,46468,11,46496,46496,11,46524,46524,11,46552,46552,11,46580,46580,11,46608,46608,11,46636,46636,11,46664,46664,11,46692,46692,11,46720,46720,11,46748,46748,11,46776,46776,11,46804,46804,11,46832,46832,11,46860,46860,11,46888,46888,11,46916,46916,11,46944,46944,11,46972,46972,11,47000,47000,11,47028,47028,11,47056,47056,11,47084,47084,11,47112,47112,11,47140,47140,11,47168,47168,11,47196,47196,11,47224,47224,11,47252,47252,11,47280,47280,11,47308,47308,11,47336,47336,11,47364,47364,11,47392,47392,11,47420,47420,11,47448,47448,11,47476,47476,11,47504,47504,11,47532,47532,11,47560,47560,11,47588,47588,11,47616,47616,11,47644,47644,11,47672,47672,11,47700,47700,11,47728,47728,11,47756,47756,11,47784,47784,11,47812,47812,11,47840,47840,11,47868,47868,11,47896,47896,11,47924,47924,11,47952,47952,11,47980,47980,11,48008,48008,11,48036,48036,11,48064,48064,11,48092,48092,11,48120,48120,11,48148,48148,11,48176,48176,11,48204,48204,11,48232,48232,11,48260,48260,11,48288,48288,11,48316,48316,11,48344,48344,11,48372,48372,11,48400,48400,11,48428,48428,11,48456,48456,11,48484,48484,11,48512,48512,11,48540,48540,11,48568,48568,11,48596,48596,11,48624,48624,11,48652,48652,11,48680,48680,11,48708,48708,11,48736,48736,11,48764,48764,11,48792,48792,11,48820,48820,11,48848,48848,11,48876,48876,11,48904,48904,11,48932,48932,11,48960,48960,11,48988,48988,11,49016,49016,11,49044,49044,11,49072,49072,11,49100,49100,11,49128,49128,11,49156,49156,11,49184,49184,11,49212,49212,11,49240,49240,11,49268,49268,11,49296,49296,11,49324,49324,11,49352,49352,11,49380,49380,11,49408,49408,11,49436,49436,11,49464,49464,11,49492,49492,11,49520,49520,11,49548,49548,11,49576,49576,11,49604,49604,11,49632,49632,11,49660,49660,11,49688,49688,11,49716,49716,11,49744,49744,11,49772,49772,11,49800,49800,11,49828,49828,11,49856,49856,11,49884,49884,11,49912,49912,11,49940,49940,11,49968,49968,11,49996,49996,11,50024,50024,11,50052,50052,11,50080,50080,11,50108,50108,11,50136,50136,11,50164,50164,11,50192,50192,11,50220,50220,11,50248,50248,11,50276,50276,11,50304,50304,11,50332,50332,11,50360,50360,11,50388,50388,11,50416,50416,11,50444,50444,11,50472,50472,11,50500,50500,11,50528,50528,11,50556,50556,11,50584,50584,11,50612,50612,11,50640,50640,11,50668,50668,11,50696,50696,11,50724,50724,11,50752,50752,11,50780,50780,11,50808,50808,11,50836,50836,11,50864,50864,11,50892,50892,11,50920,50920,11,50948,50948,11,50976,50976,11,51004,51004,11,51032,51032,11,51060,51060,11,51088,51088,11,51116,51116,11,51144,51144,11,51172,51172,11,51200,51200,11,51228,51228,11,51256,51256,11,51284,51284,11,51312,51312,11,51340,51340,11,51368,51368,11,51396,51396,11,51424,51424,11,51452,51452,11,51480,51480,11,51508,51508,11,51536,51536,11,51564,51564,11,51592,51592,11,51620,51620,11,51648,51648,11,51676,51676,11,51704,51704,11,51732,51732,11,51760,51760,11,51788,51788,11,51816,51816,11,51844,51844,11,51872,51872,11,51900,51900,11,51928,51928,11,51956,51956,11,51984,51984,11,52012,52012,11,52040,52040,11,52068,52068,11,52096,52096,11,52124,52124,11,52152,52152,11,52180,52180,11,52208,52208,11,52236,52236,11,52264,52264,11,52292,52292,11,52320,52320,11,52348,52348,11,52376,52376,11,52404,52404,11,52432,52432,11,52460,52460,11,52488,52488,11,52516,52516,11,52544,52544,11,52572,52572,11,52600,52600,11,52628,52628,11,52656,52656,11,52684,52684,11,52712,52712,11,52740,52740,11,52768,52768,11,52796,52796,11,52824,52824,11,52852,52852,11,52880,52880,11,52908,52908,11,52936,52936,11,52964,52964,11,52992,52992,11,53020,53020,11,53048,53048,11,53076,53076,11,53104,53104,11,53132,53132,11,53160,53160,11,53188,53188,11,53216,53216,11,53244,53244,11,53272,53272,11,53300,53300,11,53328,53328,11,53356,53356,11,53384,53384,11,53412,53412,11,53440,53440,11,53468,53468,11,53496,53496,11,53524,53524,11,53552,53552,11,53580,53580,11,53608,53608,11,53636,53636,11,53664,53664,11,53692,53692,11,53720,53720,11,53748,53748,11,53776,53776,11,53804,53804,11,53832,53832,11,53860,53860,11,53888,53888,11,53916,53916,11,53944,53944,11,53972,53972,11,54000,54000,11,54028,54028,11,54056,54056,11,54084,54084,11,54112,54112,11,54140,54140,11,54168,54168,11,54196,54196,11,54224,54224,11,54252,54252,11,54280,54280,11,54308,54308,11,54336,54336,11,54364,54364,11,54392,54392,11,54420,54420,11,54448,54448,11,54476,54476,11,54504,54504,11,54532,54532,11,54560,54560,11,54588,54588,11,54616,54616,11,54644,54644,11,54672,54672,11,54700,54700,11,54728,54728,11,54756,54756,11,54784,54784,11,54812,54812,11,54840,54840,11,54868,54868,11,54896,54896,11,54924,54924,11,54952,54952,11,54980,54980,11,55008,55008,11,55036,55036,11,55064,55064,11,55092,55092,11,55120,55120,11,55148,55148,11,55176,55176,11,55216,55238,9,64286,64286,5,65056,65071,5,65438,65439,5,65529,65531,4,66272,66272,5,68097,68099,5,68108,68111,5,68159,68159,5,68900,68903,5,69446,69456,5,69632,69632,7,69634,69634,7,69744,69744,5,69759,69761,5,69808,69810,7,69815,69816,7,69821,69821,1,69837,69837,1,69927,69931,5,69933,69940,5,70003,70003,5,70018,70018,7,70070,70078,5,70082,70083,1,70094,70094,7,70188,70190,7,70194,70195,7,70197,70197,7,70206,70206,5,70368,70370,7,70400,70401,5,70459,70460,5,70463,70463,7,70465,70468,7,70475,70477,7,70498,70499,7,70512,70516,5,70712,70719,5,70722,70724,5,70726,70726,5,70832,70832,5,70835,70840,5,70842,70842,5,70845,70845,5,70847,70848,5,70850,70851,5,71088,71089,7,71096,71099,7,71102,71102,7,71132,71133,5,71219,71226,5,71229,71229,5,71231,71232,5,71340,71340,7,71342,71343,7,71350,71350,7,71453,71455,5,71462,71462,7,71724,71726,7,71736,71736,7,71984,71984,5,71991,71992,7,71997,71997,7,71999,71999,1,72001,72001,1,72003,72003,5,72148,72151,5,72156,72159,7,72164,72164,7,72243,72248,5,72250,72250,1,72263,72263,5,72279,72280,7,72324,72329,1,72343,72343,7,72751,72751,7,72760,72765,5,72767,72767,5,72873,72873,7,72881,72881,7,72884,72884,7,73009,73014,5,73020,73021,5,73030,73030,1,73098,73102,7,73107,73108,7,73110,73110,7,73459,73460,5,78896,78904,4,92976,92982,5,94033,94087,7,94180,94180,5,113821,113822,5,118528,118573,5,119141,119141,5,119143,119145,5,119150,119154,5,119163,119170,5,119210,119213,5,121344,121398,5,121461,121461,5,121499,121503,5,122880,122886,5,122907,122913,5,122918,122922,5,123566,123566,5,125136,125142,5,126976,126979,14,126981,127182,14,127184,127231,14,127279,127279,14,127344,127345,14,127374,127374,14,127405,127461,14,127489,127490,14,127514,127514,14,127538,127546,14,127561,127567,14,127570,127743,14,127757,127758,14,127760,127760,14,127762,127762,14,127766,127768,14,127770,127770,14,127772,127772,14,127775,127776,14,127778,127779,14,127789,127791,14,127794,127795,14,127798,127798,14,127819,127819,14,127824,127824,14,127868,127868,14,127870,127871,14,127892,127893,14,127896,127896,14,127900,127901,14,127904,127940,14,127942,127942,14,127944,127944,14,127946,127946,14,127951,127955,14,127968,127971,14,127973,127984,14,127987,127987,14,127989,127989,14,127991,127991,14,127995,127999,5,128008,128008,14,128012,128014,14,128017,128018,14,128020,128020,14,128022,128022,14,128042,128042,14,128063,128063,14,128065,128065,14,128101,128101,14,128108,128109,14,128173,128173,14,128182,128183,14,128236,128237,14,128239,128239,14,128245,128245,14,128248,128248,14,128253,128253,14,128255,128258,14,128260,128263,14,128265,128265,14,128277,128277,14,128300,128301,14,128326,128328,14,128331,128334,14,128336,128347,14,128360,128366,14,128369,128370,14,128378,128378,14,128391,128391,14,128394,128397,14,128400,128400,14,128405,128406,14,128420,128420,14,128422,128423,14,128425,128432,14,128435,128443,14,128445,128449,14,128453,128464,14,128468,128475,14,128479,128480,14,128482,128482,14,128484,128487,14,128489,128494,14,128496,128498,14,128500,128505,14,128507,128511,14,128513,128518,14,128521,128525,14,128527,128527,14,128529,128529,14,128533,128533,14,128535,128535,14,128537,128537,14]`)}var Re=class e{static{this.ambiguousCharacterData=new Se(()=>JSON.parse(`{\"_common\":[8232,32,8233,32,5760,32,8192,32,8193,32,8194,32,8195,32,8196,32,8197,32,8198,32,8200,32,8201,32,8202,32,8287,32,8199,32,8239,32,2042,95,65101,95,65102,95,65103,95,8208,45,8209,45,8210,45,65112,45,1748,45,8259,45,727,45,8722,45,10134,45,11450,45,1549,44,1643,44,184,44,42233,44,894,59,2307,58,2691,58,1417,58,1795,58,1796,58,5868,58,65072,58,6147,58,6153,58,8282,58,1475,58,760,58,42889,58,8758,58,720,58,42237,58,451,33,11601,33,660,63,577,63,2429,63,5038,63,42731,63,119149,46,8228,46,1793,46,1794,46,42510,46,68176,46,1632,46,1776,46,42232,46,1373,96,65287,96,8219,96,1523,96,8242,96,1370,96,8175,96,65344,96,900,96,8189,96,8125,96,8127,96,8190,96,697,96,884,96,712,96,714,96,715,96,756,96,699,96,701,96,700,96,702,96,42892,96,1497,96,2036,96,2037,96,5194,96,5836,96,94033,96,94034,96,65339,91,10088,40,10098,40,12308,40,64830,40,65341,93,10089,41,10099,41,12309,41,64831,41,10100,123,119060,123,10101,125,65342,94,8270,42,1645,42,8727,42,66335,42,5941,47,8257,47,8725,47,8260,47,9585,47,10187,47,10744,47,119354,47,12755,47,12339,47,11462,47,20031,47,12035,47,65340,92,65128,92,8726,92,10189,92,10741,92,10745,92,119311,92,119355,92,12756,92,20022,92,12034,92,42872,38,708,94,710,94,5869,43,10133,43,66203,43,8249,60,10094,60,706,60,119350,60,5176,60,5810,60,5120,61,11840,61,12448,61,42239,61,8250,62,10095,62,707,62,119351,62,5171,62,94015,62,8275,126,732,126,8128,126,8764,126,65372,124,65293,45,118002,50,120784,50,120794,50,120804,50,120814,50,120824,50,130034,50,42842,50,423,50,1000,50,42564,50,5311,50,42735,50,119302,51,118003,51,120785,51,120795,51,120805,51,120815,51,120825,51,130035,51,42923,51,540,51,439,51,42858,51,11468,51,1248,51,94011,51,71882,51,118004,52,120786,52,120796,52,120806,52,120816,52,120826,52,130036,52,5070,52,71855,52,118005,53,120787,53,120797,53,120807,53,120817,53,120827,53,130037,53,444,53,71867,53,118006,54,120788,54,120798,54,120808,54,120818,54,120828,54,130038,54,11474,54,5102,54,71893,54,119314,55,118007,55,120789,55,120799,55,120809,55,120819,55,120829,55,130039,55,66770,55,71878,55,2819,56,2538,56,2666,56,125131,56,118008,56,120790,56,120800,56,120810,56,120820,56,120830,56,130040,56,547,56,546,56,66330,56,2663,57,2920,57,2541,57,3437,57,118009,57,120791,57,120801,57,120811,57,120821,57,120831,57,130041,57,42862,57,11466,57,71884,57,71852,57,71894,57,9082,97,65345,97,119834,97,119886,97,119938,97,119990,97,120042,97,120094,97,120146,97,120198,97,120250,97,120302,97,120354,97,120406,97,120458,97,593,97,945,97,120514,97,120572,97,120630,97,120688,97,120746,97,65313,65,117974,65,119808,65,119860,65,119912,65,119964,65,120016,65,120068,65,120120,65,120172,65,120224,65,120276,65,120328,65,120380,65,120432,65,913,65,120488,65,120546,65,120604,65,120662,65,120720,65,5034,65,5573,65,42222,65,94016,65,66208,65,119835,98,119887,98,119939,98,119991,98,120043,98,120095,98,120147,98,120199,98,120251,98,120303,98,120355,98,120407,98,120459,98,388,98,5071,98,5234,98,5551,98,65314,66,8492,66,117975,66,119809,66,119861,66,119913,66,120017,66,120069,66,120121,66,120173,66,120225,66,120277,66,120329,66,120381,66,120433,66,42932,66,914,66,120489,66,120547,66,120605,66,120663,66,120721,66,5108,66,5623,66,42192,66,66178,66,66209,66,66305,66,65347,99,8573,99,119836,99,119888,99,119940,99,119992,99,120044,99,120096,99,120148,99,120200,99,120252,99,120304,99,120356,99,120408,99,120460,99,7428,99,1010,99,11429,99,43951,99,66621,99,128844,67,71913,67,71922,67,65315,67,8557,67,8450,67,8493,67,117976,67,119810,67,119862,67,119914,67,119966,67,120018,67,120174,67,120226,67,120278,67,120330,67,120382,67,120434,67,1017,67,11428,67,5087,67,42202,67,66210,67,66306,67,66581,67,66844,67,8574,100,8518,100,119837,100,119889,100,119941,100,119993,100,120045,100,120097,100,120149,100,120201,100,120253,100,120305,100,120357,100,120409,100,120461,100,1281,100,5095,100,5231,100,42194,100,8558,68,8517,68,117977,68,119811,68,119863,68,119915,68,119967,68,120019,68,120071,68,120123,68,120175,68,120227,68,120279,68,120331,68,120383,68,120435,68,5024,68,5598,68,5610,68,42195,68,8494,101,65349,101,8495,101,8519,101,119838,101,119890,101,119942,101,120046,101,120098,101,120150,101,120202,101,120254,101,120306,101,120358,101,120410,101,120462,101,43826,101,1213,101,8959,69,65317,69,8496,69,117978,69,119812,69,119864,69,119916,69,120020,69,120072,69,120124,69,120176,69,120228,69,120280,69,120332,69,120384,69,120436,69,917,69,120492,69,120550,69,120608,69,120666,69,120724,69,11577,69,5036,69,42224,69,71846,69,71854,69,66182,69,119839,102,119891,102,119943,102,119995,102,120047,102,120099,102,120151,102,120203,102,120255,102,120307,102,120359,102,120411,102,120463,102,43829,102,42905,102,383,102,7837,102,1412,102,119315,70,8497,70,117979,70,119813,70,119865,70,119917,70,120021,70,120073,70,120125,70,120177,70,120229,70,120281,70,120333,70,120385,70,120437,70,42904,70,988,70,120778,70,5556,70,42205,70,71874,70,71842,70,66183,70,66213,70,66853,70,65351,103,8458,103,119840,103,119892,103,119944,103,120048,103,120100,103,120152,103,120204,103,120256,103,120308,103,120360,103,120412,103,120464,103,609,103,7555,103,397,103,1409,103,117980,71,119814,71,119866,71,119918,71,119970,71,120022,71,120074,71,120126,71,120178,71,120230,71,120282,71,120334,71,120386,71,120438,71,1292,71,5056,71,5107,71,42198,71,65352,104,8462,104,119841,104,119945,104,119997,104,120049,104,120101,104,120153,104,120205,104,120257,104,120309,104,120361,104,120413,104,120465,104,1211,104,1392,104,5058,104,65320,72,8459,72,8460,72,8461,72,117981,72,119815,72,119867,72,119919,72,120023,72,120179,72,120231,72,120283,72,120335,72,120387,72,120439,72,919,72,120494,72,120552,72,120610,72,120668,72,120726,72,11406,72,5051,72,5500,72,42215,72,66255,72,731,105,9075,105,65353,105,8560,105,8505,105,8520,105,119842,105,119894,105,119946,105,119998,105,120050,105,120102,105,120154,105,120206,105,120258,105,120310,105,120362,105,120414,105,120466,105,120484,105,618,105,617,105,953,105,8126,105,890,105,120522,105,120580,105,120638,105,120696,105,120754,105,1110,105,42567,105,1231,105,43893,105,5029,105,71875,105,65354,106,8521,106,119843,106,119895,106,119947,106,119999,106,120051,106,120103,106,120155,106,120207,106,120259,106,120311,106,120363,106,120415,106,120467,106,1011,106,1112,106,65322,74,117983,74,119817,74,119869,74,119921,74,119973,74,120025,74,120077,74,120129,74,120181,74,120233,74,120285,74,120337,74,120389,74,120441,74,42930,74,895,74,1032,74,5035,74,5261,74,42201,74,119844,107,119896,107,119948,107,120000,107,120052,107,120104,107,120156,107,120208,107,120260,107,120312,107,120364,107,120416,107,120468,107,8490,75,65323,75,117984,75,119818,75,119870,75,119922,75,119974,75,120026,75,120078,75,120130,75,120182,75,120234,75,120286,75,120338,75,120390,75,120442,75,922,75,120497,75,120555,75,120613,75,120671,75,120729,75,11412,75,5094,75,5845,75,42199,75,66840,75,1472,108,8739,73,9213,73,65512,73,1633,108,1777,73,66336,108,125127,108,118001,108,120783,73,120793,73,120803,73,120813,73,120823,73,130033,73,65321,73,8544,73,8464,73,8465,73,117982,108,119816,73,119868,73,119920,73,120024,73,120128,73,120180,73,120232,73,120284,73,120336,73,120388,73,120440,73,65356,108,8572,73,8467,108,119845,108,119897,108,119949,108,120001,108,120053,108,120105,73,120157,73,120209,73,120261,73,120313,73,120365,73,120417,73,120469,73,448,73,120496,73,120554,73,120612,73,120670,73,120728,73,11410,73,1030,73,1216,73,1493,108,1503,108,1575,108,126464,108,126592,108,65166,108,65165,108,1994,108,11599,73,5825,73,42226,73,93992,73,66186,124,66313,124,119338,76,8556,76,8466,76,117985,76,119819,76,119871,76,119923,76,120027,76,120079,76,120131,76,120183,76,120235,76,120287,76,120339,76,120391,76,120443,76,11472,76,5086,76,5290,76,42209,76,93974,76,71843,76,71858,76,66587,76,66854,76,65325,77,8559,77,8499,77,117986,77,119820,77,119872,77,119924,77,120028,77,120080,77,120132,77,120184,77,120236,77,120288,77,120340,77,120392,77,120444,77,924,77,120499,77,120557,77,120615,77,120673,77,120731,77,1018,77,11416,77,5047,77,5616,77,5846,77,42207,77,66224,77,66321,77,119847,110,119899,110,119951,110,120003,110,120055,110,120107,110,120159,110,120211,110,120263,110,120315,110,120367,110,120419,110,120471,110,1400,110,1404,110,65326,78,8469,78,117987,78,119821,78,119873,78,119925,78,119977,78,120029,78,120081,78,120185,78,120237,78,120289,78,120341,78,120393,78,120445,78,925,78,120500,78,120558,78,120616,78,120674,78,120732,78,11418,78,42208,78,66835,78,3074,111,3202,111,3330,111,3458,111,2406,111,2662,111,2790,111,3046,111,3174,111,3302,111,3430,111,3664,111,3792,111,4160,111,1637,111,1781,111,65359,111,8500,111,119848,111,119900,111,119952,111,120056,111,120108,111,120160,111,120212,111,120264,111,120316,111,120368,111,120420,111,120472,111,7439,111,7441,111,43837,111,959,111,120528,111,120586,111,120644,111,120702,111,120760,111,963,111,120532,111,120590,111,120648,111,120706,111,120764,111,11423,111,4351,111,1413,111,1505,111,1607,111,126500,111,126564,111,126596,111,65259,111,65260,111,65258,111,65257,111,1726,111,64428,111,64429,111,64427,111,64426,111,1729,111,64424,111,64425,111,64423,111,64422,111,1749,111,3360,111,4125,111,66794,111,71880,111,71895,111,66604,111,1984,79,2534,79,2918,79,12295,79,70864,79,71904,79,118000,79,120782,79,120792,79,120802,79,120812,79,120822,79,130032,79,65327,79,117988,79,119822,79,119874,79,119926,79,119978,79,120030,79,120082,79,120134,79,120186,79,120238,79,120290,79,120342,79,120394,79,120446,79,927,79,120502,79,120560,79,120618,79,120676,79,120734,79,11422,79,1365,79,11604,79,4816,79,2848,79,66754,79,42227,79,71861,79,66194,79,66219,79,66564,79,66838,79,9076,112,65360,112,119849,112,119901,112,119953,112,120005,112,120057,112,120109,112,120161,112,120213,112,120265,112,120317,112,120369,112,120421,112,120473,112,961,112,120530,112,120544,112,120588,112,120602,112,120646,112,120660,112,120704,112,120718,112,120762,112,120776,112,11427,112,65328,80,8473,80,117989,80,119823,80,119875,80,119927,80,119979,80,120031,80,120083,80,120187,80,120239,80,120291,80,120343,80,120395,80,120447,80,929,80,120504,80,120562,80,120620,80,120678,80,120736,80,11426,80,5090,80,5229,80,42193,80,66197,80,119850,113,119902,113,119954,113,120006,113,120058,113,120110,113,120162,113,120214,113,120266,113,120318,113,120370,113,120422,113,120474,113,1307,113,1379,113,1382,113,8474,81,117990,81,119824,81,119876,81,119928,81,119980,81,120032,81,120084,81,120188,81,120240,81,120292,81,120344,81,120396,81,120448,81,11605,81,119851,114,119903,114,119955,114,120007,114,120059,114,120111,114,120163,114,120215,114,120267,114,120319,114,120371,114,120423,114,120475,114,43847,114,43848,114,7462,114,11397,114,43905,114,119318,82,8475,82,8476,82,8477,82,117991,82,119825,82,119877,82,119929,82,120033,82,120189,82,120241,82,120293,82,120345,82,120397,82,120449,82,422,82,5025,82,5074,82,66740,82,5511,82,42211,82,94005,82,65363,115,119852,115,119904,115,119956,115,120008,115,120060,115,120112,115,120164,115,120216,115,120268,115,120320,115,120372,115,120424,115,120476,115,42801,115,445,115,1109,115,43946,115,71873,115,66632,115,65331,83,117992,83,119826,83,119878,83,119930,83,119982,83,120034,83,120086,83,120138,83,120190,83,120242,83,120294,83,120346,83,120398,83,120450,83,1029,83,1359,83,5077,83,5082,83,42210,83,94010,83,66198,83,66592,83,119853,116,119905,116,119957,116,120009,116,120061,116,120113,116,120165,116,120217,116,120269,116,120321,116,120373,116,120425,116,120477,116,8868,84,10201,84,128872,84,65332,84,117993,84,119827,84,119879,84,119931,84,119983,84,120035,84,120087,84,120139,84,120191,84,120243,84,120295,84,120347,84,120399,84,120451,84,932,84,120507,84,120565,84,120623,84,120681,84,120739,84,11430,84,5026,84,42196,84,93962,84,71868,84,66199,84,66225,84,66325,84,119854,117,119906,117,119958,117,120010,117,120062,117,120114,117,120166,117,120218,117,120270,117,120322,117,120374,117,120426,117,120478,117,42911,117,7452,117,43854,117,43858,117,651,117,965,117,120534,117,120592,117,120650,117,120708,117,120766,117,1405,117,66806,117,71896,117,8746,85,8899,85,117994,85,119828,85,119880,85,119932,85,119984,85,120036,85,120088,85,120140,85,120192,85,120244,85,120296,85,120348,85,120400,85,120452,85,1357,85,4608,85,66766,85,5196,85,42228,85,94018,85,71864,85,8744,118,8897,118,65366,118,8564,118,119855,118,119907,118,119959,118,120011,118,120063,118,120115,118,120167,118,120219,118,120271,118,120323,118,120375,118,120427,118,120479,118,7456,118,957,118,120526,118,120584,118,120642,118,120700,118,120758,118,1141,118,1496,118,71430,118,43945,118,71872,118,119309,86,1639,86,1783,86,8548,86,117995,86,119829,86,119881,86,119933,86,119985,86,120037,86,120089,86,120141,86,120193,86,120245,86,120297,86,120349,86,120401,86,120453,86,1140,86,11576,86,5081,86,5167,86,42719,86,42214,86,93960,86,71840,86,66845,86,623,119,119856,119,119908,119,119960,119,120012,119,120064,119,120116,119,120168,119,120220,119,120272,119,120324,119,120376,119,120428,119,120480,119,7457,119,1121,119,1309,119,1377,119,71434,119,71438,119,71439,119,43907,119,71910,87,71919,87,117996,87,119830,87,119882,87,119934,87,119986,87,120038,87,120090,87,120142,87,120194,87,120246,87,120298,87,120350,87,120402,87,120454,87,1308,87,5043,87,5076,87,42218,87,5742,120,10539,120,10540,120,10799,120,65368,120,8569,120,119857,120,119909,120,119961,120,120013,120,120065,120,120117,120,120169,120,120221,120,120273,120,120325,120,120377,120,120429,120,120481,120,5441,120,5501,120,5741,88,9587,88,66338,88,71916,88,65336,88,8553,88,117997,88,119831,88,119883,88,119935,88,119987,88,120039,88,120091,88,120143,88,120195,88,120247,88,120299,88,120351,88,120403,88,120455,88,42931,88,935,88,120510,88,120568,88,120626,88,120684,88,120742,88,11436,88,11613,88,5815,88,42219,88,66192,88,66228,88,66327,88,66855,88,611,121,7564,121,65369,121,119858,121,119910,121,119962,121,120014,121,120066,121,120118,121,120170,121,120222,121,120274,121,120326,121,120378,121,120430,121,120482,121,655,121,7935,121,43866,121,947,121,8509,121,120516,121,120574,121,120632,121,120690,121,120748,121,1199,121,4327,121,71900,121,65337,89,117998,89,119832,89,119884,89,119936,89,119988,89,120040,89,120092,89,120144,89,120196,89,120248,89,120300,89,120352,89,120404,89,120456,89,933,89,978,89,120508,89,120566,89,120624,89,120682,89,120740,89,11432,89,1198,89,5033,89,5053,89,42220,89,94019,89,71844,89,66226,89,119859,122,119911,122,119963,122,120015,122,120067,122,120119,122,120171,122,120223,122,120275,122,120327,122,120379,122,120431,122,120483,122,7458,122,43923,122,71876,122,71909,90,66293,90,65338,90,8484,90,8488,90,117999,90,119833,90,119885,90,119937,90,119989,90,120041,90,120197,90,120249,90,120301,90,120353,90,120405,90,120457,90,918,90,120493,90,120551,90,120609,90,120667,90,120725,90,5059,90,42204,90,71849,90,65282,34,65283,35,65284,36,65285,37,65286,38,65290,42,65291,43,65294,46,65295,47,65296,48,65298,50,65299,51,65300,52,65301,53,65302,54,65303,55,65304,56,65305,57,65308,60,65309,61,65310,62,65312,64,65316,68,65318,70,65319,71,65324,76,65329,81,65330,82,65333,85,65334,86,65335,87,65343,95,65346,98,65348,100,65350,102,65355,107,65357,109,65358,110,65361,113,65362,114,65364,116,65365,117,65367,119,65370,122,65371,123,65373,125,119846,109],\"_default\":[160,32,8211,45,65374,126,8218,44,65306,58,65281,33,8216,96,8217,96,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"cs\":[65374,126,8218,44,65306,58,65281,33,8216,96,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"de\":[65374,126,65306,58,65281,33,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"es\":[8211,45,65374,126,8218,44,65306,58,65281,33,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"fr\":[65374,126,8218,44,65306,58,65281,33,8216,96,8245,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"it\":[160,32,8211,45,65374,126,8218,44,65306,58,65281,33,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"ja\":[8211,45,8218,44,65281,33,8216,96,8245,96,180,96,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65292,44,65297,49,65307,59],\"ko\":[8211,45,65374,126,8218,44,65306,58,65281,33,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"pl\":[65374,126,65306,58,65281,33,8216,96,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"pt-BR\":[65374,126,8218,44,65306,58,65281,33,8216,96,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"qps-ploc\":[160,32,8211,45,65374,126,8218,44,65306,58,65281,33,8216,96,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"ru\":[65374,126,8218,44,65306,58,65281,33,8216,96,8245,96,180,96,12494,47,305,105,921,73,1009,112,215,120,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"tr\":[160,32,8211,45,65374,126,8218,44,65306,58,65281,33,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65288,40,65289,41,65292,44,65297,49,65307,59,65311,63],\"zh-hans\":[160,32,65374,126,8218,44,8245,96,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89,65297,49],\"zh-hant\":[8211,45,65374,126,8218,44,180,96,12494,47,1047,51,1073,54,1072,97,1040,65,1068,98,1042,66,1089,99,1057,67,1077,101,1045,69,1053,72,305,105,1050,75,921,73,1052,77,1086,111,1054,79,1009,112,1088,112,1056,80,1075,114,1058,84,215,120,1093,120,1061,88,1091,121,1059,89]}`))}static{this.cache=new xe({getCacheKey:JSON.stringify},t=>{function n(e){let t=/* @__PURE__ */ new Map;for(let n=0;n<e.length;n+=2)t.set(e[n],e[n+1]);return t}function r(e,t){let n=new Map(e);for(let[e,r]of t)n.set(e,r);return n}function i(e,t){if(!e)return t;let n=/* @__PURE__ */ new Map;for(let[r,i]of e)t.has(r)&&n.set(r,i);return n}let a=this.ambiguousCharacterData.value,o=t.filter(e=>!e.startsWith(`_`)&&Object.hasOwn(a,e));o.length===0&&(o=[`_default`]);let s;for(let e of o){let t=n(a[e]);s=i(s,t)}let c=r(n(a._common),s);return new e(c)})}static getInstance(t){return e.cache.get(Array.from(t))}static{this._locales=new Se(()=>Object.keys(e.ambiguousCharacterData.value).filter(e=>!e.startsWith(`_`)))}static getLocales(){return e._locales.value}constructor(e){this.confusableDictionary=e}isAmbiguous(e){return this.confusableDictionary.has(e)}getPrimaryConfusable(e){return this.confusableDictionary.get(e)}getConfusableCodePoints(){return new Set(this.confusableDictionary.keys())}},ze=class e{static getRawData(){return JSON.parse(`{\"_common\":[11,12,13,127,847,1564,4447,4448,6068,6069,6155,6156,6157,6158,7355,7356,8192,8193,8194,8195,8196,8197,8198,8199,8200,8201,8202,8204,8205,8206,8207,8234,8235,8236,8237,8238,8239,8287,8288,8289,8290,8291,8292,8293,8294,8295,8296,8297,8298,8299,8300,8301,8302,8303,10240,12644,65024,65025,65026,65027,65028,65029,65030,65031,65032,65033,65034,65035,65036,65037,65038,65039,65279,65440,65520,65521,65522,65523,65524,65525,65526,65527,65528,65532,78844,119155,119156,119157,119158,119159,119160,119161,119162,917504,917505,917506,917507,917508,917509,917510,917511,917512,917513,917514,917515,917516,917517,917518,917519,917520,917521,917522,917523,917524,917525,917526,917527,917528,917529,917530,917531,917532,917533,917534,917535,917536,917537,917538,917539,917540,917541,917542,917543,917544,917545,917546,917547,917548,917549,917550,917551,917552,917553,917554,917555,917556,917557,917558,917559,917560,917561,917562,917563,917564,917565,917566,917567,917568,917569,917570,917571,917572,917573,917574,917575,917576,917577,917578,917579,917580,917581,917582,917583,917584,917585,917586,917587,917588,917589,917590,917591,917592,917593,917594,917595,917596,917597,917598,917599,917600,917601,917602,917603,917604,917605,917606,917607,917608,917609,917610,917611,917612,917613,917614,917615,917616,917617,917618,917619,917620,917621,917622,917623,917624,917625,917626,917627,917628,917629,917630,917631,917760,917761,917762,917763,917764,917765,917766,917767,917768,917769,917770,917771,917772,917773,917774,917775,917776,917777,917778,917779,917780,917781,917782,917783,917784,917785,917786,917787,917788,917789,917790,917791,917792,917793,917794,917795,917796,917797,917798,917799,917800,917801,917802,917803,917804,917805,917806,917807,917808,917809,917810,917811,917812,917813,917814,917815,917816,917817,917818,917819,917820,917821,917822,917823,917824,917825,917826,917827,917828,917829,917830,917831,917832,917833,917834,917835,917836,917837,917838,917839,917840,917841,917842,917843,917844,917845,917846,917847,917848,917849,917850,917851,917852,917853,917854,917855,917856,917857,917858,917859,917860,917861,917862,917863,917864,917865,917866,917867,917868,917869,917870,917871,917872,917873,917874,917875,917876,917877,917878,917879,917880,917881,917882,917883,917884,917885,917886,917887,917888,917889,917890,917891,917892,917893,917894,917895,917896,917897,917898,917899,917900,917901,917902,917903,917904,917905,917906,917907,917908,917909,917910,917911,917912,917913,917914,917915,917916,917917,917918,917919,917920,917921,917922,917923,917924,917925,917926,917927,917928,917929,917930,917931,917932,917933,917934,917935,917936,917937,917938,917939,917940,917941,917942,917943,917944,917945,917946,917947,917948,917949,917950,917951,917952,917953,917954,917955,917956,917957,917958,917959,917960,917961,917962,917963,917964,917965,917966,917967,917968,917969,917970,917971,917972,917973,917974,917975,917976,917977,917978,917979,917980,917981,917982,917983,917984,917985,917986,917987,917988,917989,917990,917991,917992,917993,917994,917995,917996,917997,917998,917999],\"cs\":[173,8203,12288],\"de\":[173,8203,12288],\"es\":[8203,12288],\"fr\":[173,8203,12288],\"it\":[160,173,12288],\"ja\":[173],\"ko\":[173,12288],\"pl\":[173,8203,12288],\"pt-BR\":[173,8203,12288],\"qps-ploc\":[160,173,8203,12288],\"ru\":[173,12288],\"tr\":[160,173,8203,12288],\"zh-hans\":[160,173,8203,12288],\"zh-hant\":[173,12288]}`)}static{this._data=void 0}static getData(){return this._data||=new Set([...Object.values(e.getRawData())].flat()),this._data}static isInvisibleCharacter(t){return e.getData().has(t)}static get codePoints(){return e.getData()}};const Be=`default`;var Ve=class{constructor(e,t,n,r,i){this.vsWorker=e,this.req=t,this.channel=n,this.method=r,this.args=i,this.type=0}},He=class{constructor(e,t,n,r){this.vsWorker=e,this.seq=t,this.res=n,this.err=r,this.type=1}},Ue=class{constructor(e,t,n,r,i){this.vsWorker=e,this.req=t,this.channel=n,this.eventName=r,this.arg=i,this.type=2}},We=class{constructor(e,t,n){this.vsWorker=e,this.req=t,this.event=n,this.type=3}},Ge=class{constructor(e,t){this.vsWorker=e,this.req=t,this.type=4}},Ke=class{constructor(e){this._workerId=-1,this._handler=e,this._lastSentReq=0,this._pendingReplies=Object.create(null),this._pendingEmitters=/* @__PURE__ */ new Map,this._pendingEvents=/* @__PURE__ */ new Map}setWorkerId(e){this._workerId=e}async sendMessage(e,t,n){let r=String(++this._lastSentReq);return new Promise((i,a)=>{this._pendingReplies[r]={resolve:i,reject:a},this._send(new Ve(this._workerId,r,e,t,n))})}listen(e,t,n){let r=null,i=new E({onWillAddFirstListener:()=>{r=String(++this._lastSentReq),this._pendingEmitters.set(r,i),this._send(new Ue(this._workerId,r,e,t,n))},onDidRemoveLastListener:()=>{this._pendingEmitters.delete(r),this._send(new Ge(this._workerId,r)),r=null}});return i.event}handleMessage(e){e&&e.vsWorker&&(this._workerId===-1||e.vsWorker===this._workerId)&&this._handleMessage(e)}createProxyToRemoteChannel(e,t){return new Proxy(Object.create(null),{get:(n,r)=>(typeof r==`string`&&!n[r]&&(Je(r)?n[r]=t=>this.listen(e,r,t):qe(r)?n[r]=this.listen(e,r,void 0):r.charCodeAt(0)===36&&(n[r]=async(...n)=>(await t?.(),this.sendMessage(e,r,n)))),n[r])})}_handleMessage(e){switch(e.type){case 1:return this._handleReplyMessage(e);case 0:return this._handleRequestMessage(e);case 2:return this._handleSubscribeEventMessage(e);case 3:return this._handleEventMessage(e);case 4:return this._handleUnsubscribeEventMessage(e)}}_handleReplyMessage(e){if(!this._pendingReplies[e.seq]){console.warn(`Got reply to unknown seq`);return}let t=this._pendingReplies[e.seq];if(delete this._pendingReplies[e.seq],e.err){let n=e.err;if(e.err.$isError){let t=/* @__PURE__ */ Error();t.name=e.err.name,t.message=e.err.message,t.stack=e.err.stack,n=t}t.reject(n)}else t.resolve(e.res)}_handleRequestMessage(e){let t=e.req;this._handler.handleMessage(e.channel,e.method,e.args).then(e=>{this._send(new He(this._workerId,t,e,void 0))},e=>{e.detail instanceof Error&&(e.detail=n(e.detail)),this._send(new He(this._workerId,t,void 0,n(e)))})}_handleSubscribeEventMessage(e){let t=e.req,n=this._handler.handleEvent(e.channel,e.eventName,e.arg)(e=>{this._send(new We(this._workerId,t,e))});this._pendingEvents.set(t,n)}_handleEventMessage(e){this._pendingEmitters.has(e.req)?this._pendingEmitters.get(e.req).fire(e.event):console.warn(`Got event for unknown req`)}_handleUnsubscribeEventMessage(e){this._pendingEvents.has(e.req)?(this._pendingEvents.get(e.req).dispose(),this._pendingEvents.delete(e.req)):console.warn(`Got unsubscribe for unknown req`)}_send(e){let t=[];if(e.type===0)for(let n=0;n<e.args.length;n++){let r=e.args[n];r instanceof ArrayBuffer&&t.push(r)}else e.type===1&&e.res instanceof ArrayBuffer&&t.push(e.res);this._handler.sendMessage(e,t)}};function qe(e){return e[0]===`o`&&e[1]===`n`&&Oe(e.charCodeAt(2))}function Je(e){return/^onDynamic/.test(e)&&Oe(e.charCodeAt(9))}var Ye=class{constructor(e,t){this._localChannels=/* @__PURE__ */ new Map,this._remoteChannels=/* @__PURE__ */ new Map,this._protocol=new Ke({sendMessage:(t,n)=>{e(t,n)},handleMessage:(e,t,n)=>this._handleMessage(e,t,n),handleEvent:(e,t,n)=>this._handleEvent(e,t,n)}),this.requestHandler=t(this)}onmessage(e){this._protocol.handleMessage(e)}_handleMessage(e,t,n){if(e===Be&&t===`$initialize`)return this.initialize(n[0]);let r=e===Be?this.requestHandler:this._localChannels.get(e);if(!r)return Promise.reject(/* @__PURE__ */ Error(`Missing channel ${e} on worker thread`));let i=r[t];if(typeof i!=`function`)return Promise.reject(/* @__PURE__ */ Error(`Missing method ${t} on worker thread channel ${e}`));try{return Promise.resolve(i.apply(r,n))}catch(e){return Promise.reject(e)}}_handleEvent(e,t,n){let r=e===Be?this.requestHandler:this._localChannels.get(e);if(!r)throw Error(`Missing channel ${e} on worker thread`);if(Je(t)){let e=r[t];if(typeof e!=`function`)throw Error(`Missing dynamic event ${t} on request handler.`);let i=e.call(r,n);if(typeof i!=`function`)throw Error(`Missing dynamic event ${t} on request handler.`);return i}if(qe(t)){let e=r[t];if(typeof e!=`function`)throw Error(`Missing event ${t} on request handler.`);return e}throw Error(`Malformed event name ${t}`)}getChannel(e){if(!this._remoteChannels.has(e)){let t=this._protocol.createProxyToRemoteChannel(e);this._remoteChannels.set(e,t)}return this._remoteChannels.get(e)}async initialize(e){this._protocol.setWorkerId(e)}};let Xe=!1;function Ze(e){if(Xe)throw Error(`WebWorker already initialized!`);Xe=!0;let t=new Ye(e=>globalThis.postMessage(e),t=>e(t));return globalThis.onmessage=e=>{t.onmessage(e.data)},t}var N=class{constructor(e,t,n,r){this.originalStart=e,this.originalLength=t,this.modifiedStart=n,this.modifiedLength=r}getOriginalEnd(){return this.originalStart+this.originalLength}getModifiedEnd(){return this.modifiedStart+this.modifiedLength}};const Qe=typeof Buffer<`u`;new Se(()=>/* @__PURE__ */ new Uint8Array(256));let $e;var et=class e{static wrap(t){return Qe&&!Buffer.isBuffer(t)&&(t=Buffer.from(t.buffer,t.byteOffset,t.byteLength)),new e(t)}constructor(e){this.buffer=e,this.byteLength=this.buffer.byteLength}toString(){return Qe?this.buffer.toString():($e||=new TextDecoder,$e.decode(this.buffer))}};const tt=`0123456789abcdef`;function nt({buffer:e}){let t=``;for(let n=0;n<e.length;n++){let r=e[n];t+=tt[r>>>4],t+=tt[r&15]}return t}function rt(e,t){return(t<<5)-t+e|0}function it(e,t){t=rt(149417,t);for(let n=0,r=e.length;n<r;n++)t=rt(e.charCodeAt(n),t);return t}function at(e,t,n=32){let r=n-t,i=~((1<<r)-1);return(e<<t|(i&e)>>>r)>>>0}function ot(e,t=32){return e instanceof ArrayBuffer?nt(et.wrap(new Uint8Array(e))):(e>>>0).toString(16).padStart(t/4,`0`)}(class e{static{this._bigBlock32=/* @__PURE__ */ new DataView(/* @__PURE__ */ new ArrayBuffer(320))}constructor(){this._h0=1732584193,this._h1=4023233417,this._h2=2562383102,this._h3=271733878,this._h4=3285377520,this._buff=/* @__PURE__ */ new Uint8Array(67),this._buffDV=new DataView(this._buff.buffer),this._buffLen=0,this._totalLen=0,this._leftoverHighSurrogate=0,this._finished=!1}update(e){let t=e.length;if(t===0)return;let n=this._buff,r=this._buffLen,i=this._leftoverHighSurrogate,a,o;for(i===0?(a=e.charCodeAt(0),o=0):(a=i,o=-1,i=0);;){let s=a;if(je(a)){if(o+1<t){let t=e.charCodeAt(o+1);Me(t)?(o++,s=Ne(a,t)):s=65533}else{i=a;break}}else Me(a)&&(s=65533);if(r=this._push(n,r,s),o++,o<t)a=e.charCodeAt(o);else break}this._buffLen=r,this._leftoverHighSurrogate=i}_push(e,t,n){return n<128?e[t++]=n:n<2048?(e[t++]=192|(n&1984)>>>6,e[t++]=128|(n&63)>>>0):n<65536?(e[t++]=224|(n&61440)>>>12,e[t++]=128|(n&4032)>>>6,e[t++]=128|(n&63)>>>0):(e[t++]=240|(n&1835008)>>>18,e[t++]=128|(n&258048)>>>12,e[t++]=128|(n&4032)>>>6,e[t++]=128|(n&63)>>>0),t>=64&&(this._step(),t-=64,this._totalLen+=64,e[0]=e[64],e[1]=e[65],e[2]=e[66]),t}digest(){return this._finished||(this._finished=!0,this._leftoverHighSurrogate&&(this._leftoverHighSurrogate=0,this._buffLen=this._push(this._buff,this._buffLen,65533)),this._totalLen+=this._buffLen,this._wrapUp()),ot(this._h0)+ot(this._h1)+ot(this._h2)+ot(this._h3)+ot(this._h4)}_wrapUp(){this._buff[this._buffLen++]=128,this._buff.subarray(this._buffLen).fill(0),this._buffLen>56&&(this._step(),this._buff.fill(0));let e=8*this._totalLen;this._buffDV.setUint32(56,Math.floor(e/4294967296),!1),this._buffDV.setUint32(60,e%4294967296,!1),this._step()}_step(){let t=e._bigBlock32,n=this._buffDV;for(let e=0;e<64;e+=4)t.setUint32(e,n.getUint32(e,!1),!1);for(let e=64;e<320;e+=4)t.setUint32(e,at(t.getUint32(e-12,!1)^t.getUint32(e-32,!1)^t.getUint32(e-56,!1)^t.getUint32(e-64,!1),1),!1);let r=this._h0,i=this._h1,a=this._h2,o=this._h3,s=this._h4,c,l,u;for(let e=0;e<80;e++)e<20?(c=i&a|~i&o,l=1518500249):e<40?(c=i^a^o,l=1859775393):e<60?(c=i&a|i&o|a&o,l=2400959708):(c=i^a^o,l=3395469782),u=at(r,5)+c+s+l+t.getUint32(e*4,!1)|0,s=o,o=a,a=at(i,30),i=r,r=u;this._h0=this._h0+r|0,this._h1=this._h1+i|0,this._h2=this._h2+a|0,this._h3=this._h3+o|0,this._h4=this._h4+s|0}});var st=class{constructor(e){this.source=e}getElements(){let e=this.source,t=new Int32Array(e.length);for(let n=0,r=e.length;n<r;n++)t[n]=e.charCodeAt(n);return t}};function ct(e,t,n){return new ft(new st(e),new st(t)).ComputeDiff(n).changes}var lt=class{static Assert(e,t){if(!e)throw Error(t)}},ut=class{static Copy(e,t,n,r,i){for(let a=0;a<i;a++)n[r+a]=e[t+a]}static Copy2(e,t,n,r,i){for(let a=0;a<i;a++)n[r+a]=e[t+a]}},dt=class{constructor(){this.m_changes=[],this.m_originalStart=1073741824,this.m_modifiedStart=1073741824,this.m_originalCount=0,this.m_modifiedCount=0}MarkNextChange(){(this.m_originalCount>0||this.m_modifiedCount>0)&&this.m_changes.push(new N(this.m_originalStart,this.m_originalCount,this.m_modifiedStart,this.m_modifiedCount)),this.m_originalCount=0,this.m_modifiedCount=0,this.m_originalStart=1073741824,this.m_modifiedStart=1073741824}AddOriginalElement(e,t){this.m_originalStart=Math.min(this.m_originalStart,e),this.m_modifiedStart=Math.min(this.m_modifiedStart,t),this.m_originalCount++}AddModifiedElement(e,t){this.m_originalStart=Math.min(this.m_originalStart,e),this.m_modifiedStart=Math.min(this.m_modifiedStart,t),this.m_modifiedCount++}getChanges(){return(this.m_originalCount>0||this.m_modifiedCount>0)&&this.MarkNextChange(),this.m_changes}getReverseChanges(){return(this.m_originalCount>0||this.m_modifiedCount>0)&&this.MarkNextChange(),this.m_changes.reverse(),this.m_changes}},ft=class e{constructor(t,n,r=null){this.ContinueProcessingPredicate=r,this._originalSequence=t,this._modifiedSequence=n;let[i,a,o]=e._getElements(t),[s,c,l]=e._getElements(n);this._hasStrings=o&&l,this._originalStringElements=i,this._originalElementsOrHash=a,this._modifiedStringElements=s,this._modifiedElementsOrHash=c,this.m_forwardHistory=[],this.m_reverseHistory=[]}static _isStringArray(e){return e.length>0&&typeof e[0]==`string`}static _getElements(t){let n=t.getElements();if(e._isStringArray(n)){let e=new Int32Array(n.length);for(let t=0,r=n.length;t<r;t++)e[t]=it(n[t],0);return[n,e,!0]}return n instanceof Int32Array?[[],n,!1]:[[],new Int32Array(n),!1]}ElementsAreEqual(e,t){return this._originalElementsOrHash[e]===this._modifiedElementsOrHash[t]?!this._hasStrings||this._originalStringElements[e]===this._modifiedStringElements[t]:!1}ElementsAreStrictEqual(t,n){return this.ElementsAreEqual(t,n)?e._getStrictElement(this._originalSequence,t)===e._getStrictElement(this._modifiedSequence,n):!1}static _getStrictElement(e,t){return typeof e.getStrictElement==`function`?e.getStrictElement(t):null}OriginalElementsAreEqual(e,t){return this._originalElementsOrHash[e]===this._originalElementsOrHash[t]?!this._hasStrings||this._originalStringElements[e]===this._originalStringElements[t]:!1}ModifiedElementsAreEqual(e,t){return this._modifiedElementsOrHash[e]===this._modifiedElementsOrHash[t]?!this._hasStrings||this._modifiedStringElements[e]===this._modifiedStringElements[t]:!1}ComputeDiff(e){return this._ComputeDiff(0,this._originalElementsOrHash.length-1,0,this._modifiedElementsOrHash.length-1,e)}_ComputeDiff(e,t,n,r,i){let a=[!1],o=this.ComputeDiffRecursive(e,t,n,r,a);return i&&(o=this.PrettifyChanges(o)),{quitEarly:a[0],changes:o}}ComputeDiffRecursive(e,t,n,r,i){for(i[0]=!1;e<=t&&n<=r&&this.ElementsAreEqual(e,n);)e++,n++;for(;t>=e&&r>=n&&this.ElementsAreEqual(t,r);)t--,r--;if(e>t||n>r){let i;return n<=r?(lt.Assert(e===t+1,`originalStart should only be one more than originalEnd`),i=[new N(e,0,n,r-n+1)]):e<=t?(lt.Assert(n===r+1,`modifiedStart should only be one more than modifiedEnd`),i=[new N(e,t-e+1,n,0)]):(lt.Assert(e===t+1,`originalStart should only be one more than originalEnd`),lt.Assert(n===r+1,`modifiedStart should only be one more than modifiedEnd`),i=[]),i}let a=[0],o=[0],s=this.ComputeRecursionPoint(e,t,n,r,a,o,i),c=a[0],l=o[0];if(s!==null)return s;if(!i[0]){let a=this.ComputeDiffRecursive(e,c,n,l,i),o=[];return o=i[0]?[new N(c+1,t-(c+1)+1,l+1,r-(l+1)+1)]:this.ComputeDiffRecursive(c+1,t,l+1,r,i),this.ConcatenateChanges(a,o)}return[new N(e,t-e+1,n,r-n+1)]}WALKTRACE(e,t,n,r,i,a,o,s,c,l,u,d,f,p,m,h,g,_){let v=null,y=null,b=new dt,x=t,S=n,C=f[0]-h[0]-r,w=-1073741824,T=this.m_forwardHistory.length-1;do{let t=C+e;t===x||t<S&&c[t-1]<c[t+1]?(u=c[t+1],p=u-C-r,u<w&&b.MarkNextChange(),w=u,b.AddModifiedElement(u+1,p),C=t+1-e):(u=c[t-1]+1,p=u-C-r,u<w&&b.MarkNextChange(),w=u-1,b.AddOriginalElement(u,p+1),C=t-1-e),T>=0&&(c=this.m_forwardHistory[T],e=c[0],x=1,S=c.length-1)}while(--T>=-1);if(v=b.getReverseChanges(),_[0]){let e=f[0]+1,t=h[0]+1;if(v!==null&&v.length>0){let n=v[v.length-1];e=Math.max(e,n.getOriginalEnd()),t=Math.max(t,n.getModifiedEnd())}y=[new N(e,d-e+1,t,m-t+1)]}else{b=new dt,x=a,S=o,C=f[0]-h[0]-s,w=1073741824,T=g?this.m_reverseHistory.length-1:this.m_reverseHistory.length-2;do{let e=C+i;e===x||e<S&&l[e-1]>=l[e+1]?(u=l[e+1]-1,p=u-C-s,u>w&&b.MarkNextChange(),w=u+1,b.AddOriginalElement(u+1,p+1),C=e+1-i):(u=l[e-1],p=u-C-s,u>w&&b.MarkNextChange(),w=u,b.AddModifiedElement(u+1,p+1),C=e-1-i),T>=0&&(l=this.m_reverseHistory[T],i=l[0],x=1,S=l.length-1)}while(--T>=-1);y=b.getChanges()}return this.ConcatenateChanges(v,y)}ComputeRecursionPoint(e,t,n,r,i,a,o){let s=0,c=0,l=0,u=0,d=0,f=0;e--,n--,i[0]=0,a[0]=0,this.m_forwardHistory=[],this.m_reverseHistory=[];let p=t-e+(r-n),m=p+1,h=new Int32Array(m),g=new Int32Array(m),_=r-n,v=t-e,y=e-n,b=t-r,x=(v-_)%2==0;h[_]=e,g[v]=t,o[0]=!1;for(let S=1;S<=p/2+1;S++){let p=0,C=0;l=this.ClipDiagonalBound(_-S,S,_,m),u=this.ClipDiagonalBound(_+S,S,_,m);for(let e=l;e<=u;e+=2){s=e===l||e<u&&h[e-1]<h[e+1]?h[e+1]:h[e-1]+1,c=s-(e-_)-y;let n=s;for(;s<t&&c<r&&this.ElementsAreEqual(s+1,c+1);)s++,c++;if(h[e]=s,s+c>p+C&&(p=s,C=c),!x&&Math.abs(e-v)<=S-1&&s>=g[e])return i[0]=s,a[0]=c,n<=g[e]&&S<=1448?this.WALKTRACE(_,l,u,y,v,d,f,b,h,g,s,t,i,c,r,a,x,o):null}let w=(p-e+(C-n)-S)/2;if(this.ContinueProcessingPredicate!==null&&!this.ContinueProcessingPredicate(p,w))return o[0]=!0,i[0]=p,a[0]=C,w>0&&S<=1448?this.WALKTRACE(_,l,u,y,v,d,f,b,h,g,s,t,i,c,r,a,x,o):(e++,n++,[new N(e,t-e+1,n,r-n+1)]);d=this.ClipDiagonalBound(v-S,S,v,m),f=this.ClipDiagonalBound(v+S,S,v,m);for(let p=d;p<=f;p+=2){s=p===d||p<f&&g[p-1]>=g[p+1]?g[p+1]-1:g[p-1],c=s-(p-v)-b;let m=s;for(;s>e&&c>n&&this.ElementsAreEqual(s,c);)s--,c--;if(g[p]=s,x&&Math.abs(p-_)<=S&&s<=h[p])return i[0]=s,a[0]=c,m>=h[p]&&S<=1448?this.WALKTRACE(_,l,u,y,v,d,f,b,h,g,s,t,i,c,r,a,x,o):null}if(S<=1447){let e=new Int32Array(u-l+2);e[0]=_-l+1,ut.Copy2(h,l,e,1,u-l+1),this.m_forwardHistory.push(e),e=new Int32Array(f-d+2),e[0]=v-d+1,ut.Copy2(g,d,e,1,f-d+1),this.m_reverseHistory.push(e)}}return this.WALKTRACE(_,l,u,y,v,d,f,b,h,g,s,t,i,c,r,a,x,o)}PrettifyChanges(e){for(let t=0;t<e.length;t++){let n=e[t],r=t<e.length-1?e[t+1].originalStart:this._originalElementsOrHash.length,i=t<e.length-1?e[t+1].modifiedStart:this._modifiedElementsOrHash.length,a=n.originalLength>0,o=n.modifiedLength>0;for(;n.originalStart+n.originalLength<r&&n.modifiedStart+n.modifiedLength<i&&(!a||this.OriginalElementsAreEqual(n.originalStart,n.originalStart+n.originalLength))&&(!o||this.ModifiedElementsAreEqual(n.modifiedStart,n.modifiedStart+n.modifiedLength));){let e=this.ElementsAreStrictEqual(n.originalStart,n.modifiedStart);if(this.ElementsAreStrictEqual(n.originalStart+n.originalLength,n.modifiedStart+n.modifiedLength)&&!e)break;n.originalStart++,n.modifiedStart++}let s=[null];t<e.length-1&&this.ChangesOverlap(e[t],e[t+1],s)&&(e[t]=s[0],e.splice(t+1,1),t--)}for(let t=e.length-1;t>=0;t--){let n=e[t],r=0,i=0;if(t>0){let n=e[t-1];r=n.originalStart+n.originalLength,i=n.modifiedStart+n.modifiedLength}let a=n.originalLength>0,o=n.modifiedLength>0,s=0,c=this._boundaryScore(n.originalStart,n.originalLength,n.modifiedStart,n.modifiedLength);for(let e=1;;e++){let t=n.originalStart-e,l=n.modifiedStart-e;if(t<r||l<i||a&&!this.OriginalElementsAreEqual(t,t+n.originalLength)||o&&!this.ModifiedElementsAreEqual(l,l+n.modifiedLength))break;let u=(t===r&&l===i?5:0)+this._boundaryScore(t,n.originalLength,l,n.modifiedLength);u>c&&(c=u,s=e)}n.originalStart-=s,n.modifiedStart-=s;let l=[null];t>0&&this.ChangesOverlap(e[t-1],e[t],l)&&(e[t-1]=l[0],e.splice(t,1),t++)}if(this._hasStrings)for(let t=1,n=e.length;t<n;t++){let n=e[t-1],r=e[t],i=r.originalStart-n.originalStart-n.originalLength,a=n.originalStart,o=r.originalStart+r.originalLength,s=o-a,c=n.modifiedStart,l=r.modifiedStart+r.modifiedLength,u=l-c;if(i<5&&s<20&&u<20){let e=this._findBetterContiguousSequence(a,s,c,u,i);if(e){let[t,a]=e;(t!==n.originalStart+n.originalLength||a!==n.modifiedStart+n.modifiedLength)&&(n.originalLength=t-n.originalStart,n.modifiedLength=a-n.modifiedStart,r.originalStart=t+i,r.modifiedStart=a+i,r.originalLength=o-r.originalStart,r.modifiedLength=l-r.modifiedStart)}}}return e}_findBetterContiguousSequence(e,t,n,r,i){if(t<i||r<i)return null;let a=e+t-i+1,o=n+r-i+1,s=0,c=0,l=0;for(let t=e;t<a;t++)for(let e=n;e<o;e++){let n=this._contiguousSequenceScore(t,e,i);n>0&&n>s&&(s=n,c=t,l=e)}return s>0?[c,l]:null}_contiguousSequenceScore(e,t,n){let r=0;for(let i=0;i<n;i++){if(!this.ElementsAreEqual(e+i,t+i))return 0;r+=this._originalStringElements[e+i].length}return r}_OriginalIsBoundary(e){return e<=0||e>=this._originalElementsOrHash.length-1||this._hasStrings&&/^\\s*$/.test(this._originalStringElements[e])}_OriginalRegionIsBoundary(e,t){if(this._OriginalIsBoundary(e)||this._OriginalIsBoundary(e-1))return!0;if(t>0){let n=e+t;if(this._OriginalIsBoundary(n-1)||this._OriginalIsBoundary(n))return!0}return!1}_ModifiedIsBoundary(e){return e<=0||e>=this._modifiedElementsOrHash.length-1||this._hasStrings&&/^\\s*$/.test(this._modifiedStringElements[e])}_ModifiedRegionIsBoundary(e,t){if(this._ModifiedIsBoundary(e)||this._ModifiedIsBoundary(e-1))return!0;if(t>0){let n=e+t;if(this._ModifiedIsBoundary(n-1)||this._ModifiedIsBoundary(n))return!0}return!1}_boundaryScore(e,t,n,r){return+!!this._OriginalRegionIsBoundary(e,t)+ +!!this._ModifiedRegionIsBoundary(n,r)}ConcatenateChanges(e,t){let n=[];if(e.length===0||t.length===0)return t.length>0?t:e;if(this.ChangesOverlap(e[e.length-1],t[0],n)){let r=Array(e.length+t.length-1);return ut.Copy(e,0,r,0,e.length-1),r[e.length-1]=n[0],ut.Copy(t,1,r,e.length,t.length-1),r}{let n=Array(e.length+t.length);return ut.Copy(e,0,n,0,e.length),ut.Copy(t,0,n,e.length,t.length),n}}ChangesOverlap(e,t,n){if(lt.Assert(e.originalStart<=t.originalStart,`Left change is not less than or equal to right change`),lt.Assert(e.modifiedStart<=t.modifiedStart,`Left change is not less than or equal to right change`),e.originalStart+e.originalLength>=t.originalStart||e.modifiedStart+e.modifiedLength>=t.modifiedStart){let r=e.originalStart,i=e.originalLength,a=e.modifiedStart,o=e.modifiedLength;return e.originalStart+e.originalLength>=t.originalStart&&(i=t.originalStart+t.originalLength-e.originalStart),e.modifiedStart+e.modifiedLength>=t.modifiedStart&&(o=t.modifiedStart+t.modifiedLength-e.modifiedStart),n[0]=new N(r,i,a,o),!0}return n[0]=null,!1}ClipDiagonalBound(e,t,n,r){if(e>=0&&e<r)return e;let i=n,a=r-n-1,o=t%2==0;return e<0?o===(i%2==0)?0:1:o===(a%2==0)?r-1:r-2}},P=class e{constructor(e,t){this.lineNumber=e,this.column=t}with(t=this.lineNumber,n=this.column){return t===this.lineNumber&&n===this.column?this:new e(t,n)}delta(e=0,t=0){return this.with(Math.max(1,this.lineNumber+e),Math.max(1,this.column+t))}equals(t){return e.equals(this,t)}static equals(e,t){return!e&&!t||!!e&&!!t&&e.lineNumber===t.lineNumber&&e.column===t.column}isBefore(t){return e.isBefore(this,t)}static isBefore(e,t){return e.lineNumber<t.lineNumber?!0:t.lineNumber<e.lineNumber?!1:e.column<t.column}isBeforeOrEqual(t){return e.isBeforeOrEqual(this,t)}static isBeforeOrEqual(e,t){return e.lineNumber<t.lineNumber?!0:t.lineNumber<e.lineNumber?!1:e.column<=t.column}static compare(e,t){let n=e.lineNumber|0,r=t.lineNumber|0;return n===r?(e.column|0)-(t.column|0):n-r}clone(){return new e(this.lineNumber,this.column)}toString(){return`(`+this.lineNumber+`,`+this.column+`)`}static lift(t){return new e(t.lineNumber,t.column)}static isIPosition(e){return!!e&&typeof e.lineNumber==`number`&&typeof e.column==`number`}toJSON(){return{lineNumber:this.lineNumber,column:this.column}}},F=class e{constructor(e,t,n,r){e>n||e===n&&t>r?(this.startLineNumber=n,this.startColumn=r,this.endLineNumber=e,this.endColumn=t):(this.startLineNumber=e,this.startColumn=t,this.endLineNumber=n,this.endColumn=r)}isEmpty(){return e.isEmpty(this)}static isEmpty(e){return e.startLineNumber===e.endLineNumber&&e.startColumn===e.endColumn}containsPosition(t){return e.containsPosition(this,t)}static containsPosition(e,t){return!(t.lineNumber<e.startLineNumber||t.lineNumber>e.endLineNumber||t.lineNumber===e.startLineNumber&&t.column<e.startColumn||t.lineNumber===e.endLineNumber&&t.column>e.endColumn)}static strictContainsPosition(e,t){return!(t.lineNumber<e.startLineNumber||t.lineNumber>e.endLineNumber||t.lineNumber===e.startLineNumber&&t.column<=e.startColumn||t.lineNumber===e.endLineNumber&&t.column>=e.endColumn)}containsRange(t){return e.containsRange(this,t)}static containsRange(e,t){return!(t.startLineNumber<e.startLineNumber||t.endLineNumber<e.startLineNumber||t.startLineNumber>e.endLineNumber||t.endLineNumber>e.endLineNumber||t.startLineNumber===e.startLineNumber&&t.startColumn<e.startColumn||t.endLineNumber===e.endLineNumber&&t.endColumn>e.endColumn)}strictContainsRange(t){return e.strictContainsRange(this,t)}static strictContainsRange(e,t){return!(t.startLineNumber<e.startLineNumber||t.endLineNumber<e.startLineNumber||t.startLineNumber>e.endLineNumber||t.endLineNumber>e.endLineNumber||t.startLineNumber===e.startLineNumber&&t.startColumn<=e.startColumn||t.endLineNumber===e.endLineNumber&&t.endColumn>=e.endColumn)}plusRange(t){return e.plusRange(this,t)}static plusRange(t,n){let r,i,a,o;return n.startLineNumber<t.startLineNumber?(r=n.startLineNumber,i=n.startColumn):n.startLineNumber===t.startLineNumber?(r=n.startLineNumber,i=Math.min(n.startColumn,t.startColumn)):(r=t.startLineNumber,i=t.startColumn),n.endLineNumber>t.endLineNumber?(a=n.endLineNumber,o=n.endColumn):n.endLineNumber===t.endLineNumber?(a=n.endLineNumber,o=Math.max(n.endColumn,t.endColumn)):(a=t.endLineNumber,o=t.endColumn),new e(r,i,a,o)}intersectRanges(t){return e.intersectRanges(this,t)}static intersectRanges(t,n){let r=t.startLineNumber,i=t.startColumn,a=t.endLineNumber,o=t.endColumn,s=n.startLineNumber,c=n.startColumn,l=n.endLineNumber,u=n.endColumn;return r<s?(r=s,i=c):r===s&&(i=Math.max(i,c)),a>l?(a=l,o=u):a===l&&(o=Math.min(o,u)),r>a||r===a&&i>o?null:new e(r,i,a,o)}equalsRange(t){return e.equalsRange(this,t)}static equalsRange(e,t){return!e&&!t||!!e&&!!t&&e.startLineNumber===t.startLineNumber&&e.startColumn===t.startColumn&&e.endLineNumber===t.endLineNumber&&e.endColumn===t.endColumn}getEndPosition(){return e.getEndPosition(this)}static getEndPosition(e){return new P(e.endLineNumber,e.endColumn)}getStartPosition(){return e.getStartPosition(this)}static getStartPosition(e){return new P(e.startLineNumber,e.startColumn)}toString(){return`[`+this.startLineNumber+`,`+this.startColumn+` -> `+this.endLineNumber+`,`+this.endColumn+`]`}setEndPosition(t,n){return new e(this.startLineNumber,this.startColumn,t,n)}setStartPosition(t,n){return new e(t,n,this.endLineNumber,this.endColumn)}collapseToStart(){return e.collapseToStart(this)}static collapseToStart(t){return new e(t.startLineNumber,t.startColumn,t.startLineNumber,t.startColumn)}collapseToEnd(){return e.collapseToEnd(this)}static collapseToEnd(t){return new e(t.endLineNumber,t.endColumn,t.endLineNumber,t.endColumn)}delta(t){return new e(this.startLineNumber+t,this.startColumn,this.endLineNumber+t,this.endColumn)}isSingleLine(){return this.startLineNumber===this.endLineNumber}static fromPositions(t,n=t){return new e(t.lineNumber,t.column,n.lineNumber,n.column)}static lift(t){return t?new e(t.startLineNumber,t.startColumn,t.endLineNumber,t.endColumn):null}static isIRange(e){return!!e&&typeof e.startLineNumber==`number`&&typeof e.startColumn==`number`&&typeof e.endLineNumber==`number`&&typeof e.endColumn==`number`}static areIntersectingOrTouching(e,t){return!(e.endLineNumber<t.startLineNumber||e.endLineNumber===t.startLineNumber&&e.endColumn<t.startColumn||t.endLineNumber<e.startLineNumber||t.endLineNumber===e.startLineNumber&&t.endColumn<e.startColumn)}static areIntersecting(e,t){return!(e.endLineNumber<t.startLineNumber||e.endLineNumber===t.startLineNumber&&e.endColumn<=t.startColumn||t.endLineNumber<e.startLineNumber||t.endLineNumber===e.startLineNumber&&t.endColumn<=e.startColumn)}static areOnlyIntersecting(e,t){return!(e.endLineNumber<t.startLineNumber-1||e.endLineNumber===t.startLineNumber&&e.endColumn<t.startColumn-1||t.endLineNumber<e.startLineNumber-1||t.endLineNumber===e.startLineNumber&&t.endColumn<e.startColumn-1)}static compareRangesUsingStarts(e,t){if(e&&t){let n=e.startLineNumber|0,r=t.startLineNumber|0;if(n===r){let n=e.startColumn|0,r=t.startColumn|0;if(n===r){let n=e.endLineNumber|0,r=t.endLineNumber|0;return n===r?(e.endColumn|0)-(t.endColumn|0):n-r}return n-r}return n-r}return!!e-+!!t}static compareRangesUsingEnds(e,t){return e.endLineNumber===t.endLineNumber?e.endColumn===t.endColumn?e.startLineNumber===t.startLineNumber?e.startColumn-t.startColumn:e.startLineNumber-t.startLineNumber:e.endColumn-t.endColumn:e.endLineNumber-t.endLineNumber}static spansMultipleLines(e){return e.endLineNumber>e.startLineNumber}toJSON(){return this}};function pt(e){return e<0?0:e>255?255:e|0}function mt(e){return e<0?0:e>4294967295?4294967295:e|0}var ht=class e{constructor(t){let n=pt(t);this._defaultValue=n,this._asciiMap=e._createAsciiMap(n),this._map=/* @__PURE__ */ new Map}static _createAsciiMap(e){let t=/* @__PURE__ */ new Uint8Array(256);return t.fill(e),t}set(e,t){let n=pt(t);e>=0&&e<256?this._asciiMap[e]=n:this._map.set(e,n)}get(e){return e>=0&&e<256?this._asciiMap[e]:this._map.get(e)||this._defaultValue}clear(){this._asciiMap.fill(this._defaultValue),this._map.clear()}},gt=class{constructor(e,t,n){let r=new Uint8Array(e*t);for(let i=0,a=e*t;i<a;i++)r[i]=n;this._data=r,this.rows=e,this.cols=t}get(e,t){return this._data[e*this.cols+t]}set(e,t,n){this._data[e*this.cols+t]=n}},_t=class{constructor(e){let t=0,n=0;for(let r=0,i=e.length;r<i;r++){let[i,a,o]=e[r];a>t&&(t=a),i>n&&(n=i),o>n&&(n=o)}t++,n++;let r=new gt(n,t,0);for(let t=0,n=e.length;t<n;t++){let[n,i,a]=e[t];r.set(n,i,a)}this._states=r,this._maxCharCode=t}nextState(e,t){return t<0||t>=this._maxCharCode?0:this._states.get(e,t)}};let vt=null;function yt(){return vt===null&&(vt=new _t([[1,104,2],[1,72,2],[1,102,6],[1,70,6],[2,116,3],[2,84,3],[3,116,4],[3,84,4],[4,112,5],[4,80,5],[5,115,9],[5,83,9],[5,58,10],[6,105,7],[6,73,7],[7,108,8],[7,76,8],[8,101,9],[8,69,9],[9,58,10],[10,47,11],[11,47,12]])),vt}let bt=null;function xt(){if(bt===null){bt=new ht(0);for(let e=0;e<36;e++)bt.set(` 	<>'\"、。｡､，．：；‘〈「『〔（［｛｢｣｝］）〕』」〉’｀～…|`.charCodeAt(e),1);for(let e=0;e<4;e++)bt.set(`.,;:`.charCodeAt(e),2)}return bt}var St=class e{static _createLink(e,t,n,r,i){let a=i-1;do{let n=t.charCodeAt(a);if(e.get(n)!==2)break;a--}while(a>r);if(r>0){let e=t.charCodeAt(r-1),n=t.charCodeAt(a);(e===40&&n===41||e===91&&n===93||e===123&&n===125)&&a--}return{range:{startLineNumber:n,startColumn:r+1,endLineNumber:n,endColumn:a+2},url:t.substring(r,a+1)}}static computeLinks(t,n=yt()){let r=xt(),i=[];for(let a=1,o=t.getLineCount();a<=o;a++){let o=t.getLineContent(a),s=o.length,c=0,l=0,u=0,d=1,f=!1,p=!1,m=!1,h=!1;for(;c<s;){let t=!1,s=o.charCodeAt(c);if(d===13){let n;switch(s){case 40:f=!0,n=0;break;case 41:n=+!f;break;case 91:m=!0,p=!0,n=0;break;case 93:m=!1,n=+!p;break;case 123:h=!0,n=0;break;case 125:n=+!h;break;case 39:case 34:case 96:n=u===s?1:u===39||u===34||u===96?0:1;break;case 42:n=+(u===42);break;case 32:n=+!m;break;default:n=r.get(s)}n===1&&(i.push(e._createLink(r,o,a,l,c)),t=!0)}else if(d===12){let e;s===91?(p=!0,e=0):e=r.get(s),e===1?t=!0:d=13}else d=n.nextState(d,s),d===0&&(t=!0);t&&(d=1,f=!1,p=!1,h=!1,l=c+1,u=s),c++}d===13&&i.push(e._createLink(r,o,a,l,s))}return i}};function Ct(e){return!e||typeof e.getLineCount!=`function`||typeof e.getLineContent!=`function`?[]:St.computeLinks(e)}var wt=class e{constructor(){this._defaultValueSet=[[`true`,`false`],[`True`,`False`],[`Private`,`Public`,`Friend`,`ReadOnly`,`Partial`,`Protected`,`WriteOnly`],[`public`,`protected`,`private`]]}static{this.INSTANCE=new e}navigateValueSet(e,t,n,r,i){if(e&&t){let n=this.doNavigateValueSet(t,i);if(n)return{range:e,value:n}}if(n&&r){let e=this.doNavigateValueSet(r,i);if(e)return{range:n,value:e}}return null}doNavigateValueSet(e,t){let n=this.numberReplace(e,t);return n===null?this.textReplace(e,t):n}numberReplace(e,t){let n=10**(e.length-(e.lastIndexOf(`.`)+1)),r=Number(e),i=parseFloat(e);return!isNaN(r)&&!isNaN(i)&&r===i?r===0&&!t?null:(r=Math.floor(r*n),r+=t?n:-n,String(r/n)):null}textReplace(e,t){return this.valueSetsReplace(this._defaultValueSet,e,t)}valueSetsReplace(e,t,n){let r=null;for(let i=0,a=e.length;r===null&&i<a;i++)r=this.valueSetReplace(e[i],t,n);return r}valueSetReplace(e,t,n){let r=e.indexOf(t);return r>=0?(r+=n?1:-1,r<0?r=e.length-1:r%=e.length,e[r]):null}};const Tt=Object.freeze(function(e,t){let n=setTimeout(e.bind(t),0);return{dispose(){clearTimeout(n)}}});var Et;(function(e){function t(t){return t===e.None||t===e.Cancelled||t instanceof Dt?!0:!t||typeof t!=`object`?!1:typeof t.isCancellationRequested==`boolean`&&typeof t.onCancellationRequested==`function`}e.isCancellationToken=t,e.None=Object.freeze({isCancellationRequested:!1,onCancellationRequested:w.None}),e.Cancelled=Object.freeze({isCancellationRequested:!0,onCancellationRequested:Tt})})(Et||={});var Dt=class{constructor(){this._isCancelled=!1,this._emitter=null}cancel(){this._isCancelled||(this._isCancelled=!0,this._emitter&&(this._emitter.fire(void 0),this.dispose()))}get isCancellationRequested(){return this._isCancelled}get onCancellationRequested(){return this._isCancelled?Tt:(this._emitter||=new E,this._emitter.event)}dispose(){this._emitter&&=(this._emitter.dispose(),null)}},Ot=class{constructor(e){this._token=void 0,this._parentListener=void 0,this._parentListener=e&&e.onCancellationRequested(this.cancel,this)}get token(){return this._token||=new Dt,this._token}cancel(){this._token?this._token instanceof Dt&&this._token.cancel():this._token=Et.Cancelled}dispose(e=!1){e&&this.cancel(),this._parentListener?.dispose(),this._token?this._token instanceof Dt&&this._token.dispose():this._token=Et.None}},kt=class{constructor(){this._keyCodeToStr=[],this._strToKeyCode=Object.create(null)}define(e,t){this._keyCodeToStr[e]=t,this._strToKeyCode[t.toLowerCase()]=e}keyCodeToStr(e){return this._keyCodeToStr[e]}strToKeyCode(e){return this._strToKeyCode[e.toLowerCase()]||0}};const At=new kt,jt=new kt,Mt=new kt,Nt=Array(230),Pt=Object.create(null),Ft=Object.create(null),It=[];for(let e=0;e<=193;e++)It[e]=-1;(function(){let e=[[1,0,`None`,0,`unknown`,0,`VK_UNKNOWN`,``,``],[1,1,`Hyper`,0,``,0,``,``,``],[1,2,`Super`,0,``,0,``,``,``],[1,3,`Fn`,0,``,0,``,``,``],[1,4,`FnLock`,0,``,0,``,``,``],[1,5,`Suspend`,0,``,0,``,``,``],[1,6,`Resume`,0,``,0,``,``,``],[1,7,`Turbo`,0,``,0,``,``,``],[1,8,`Sleep`,0,``,0,`VK_SLEEP`,``,``],[1,9,`WakeUp`,0,``,0,``,``,``],[0,10,`KeyA`,31,`A`,65,`VK_A`,``,``],[0,11,`KeyB`,32,`B`,66,`VK_B`,``,``],[0,12,`KeyC`,33,`C`,67,`VK_C`,``,``],[0,13,`KeyD`,34,`D`,68,`VK_D`,``,``],[0,14,`KeyE`,35,`E`,69,`VK_E`,``,``],[0,15,`KeyF`,36,`F`,70,`VK_F`,``,``],[0,16,`KeyG`,37,`G`,71,`VK_G`,``,``],[0,17,`KeyH`,38,`H`,72,`VK_H`,``,``],[0,18,`KeyI`,39,`I`,73,`VK_I`,``,``],[0,19,`KeyJ`,40,`J`,74,`VK_J`,``,``],[0,20,`KeyK`,41,`K`,75,`VK_K`,``,``],[0,21,`KeyL`,42,`L`,76,`VK_L`,``,``],[0,22,`KeyM`,43,`M`,77,`VK_M`,``,``],[0,23,`KeyN`,44,`N`,78,`VK_N`,``,``],[0,24,`KeyO`,45,`O`,79,`VK_O`,``,``],[0,25,`KeyP`,46,`P`,80,`VK_P`,``,``],[0,26,`KeyQ`,47,`Q`,81,`VK_Q`,``,``],[0,27,`KeyR`,48,`R`,82,`VK_R`,``,``],[0,28,`KeyS`,49,`S`,83,`VK_S`,``,``],[0,29,`KeyT`,50,`T`,84,`VK_T`,``,``],[0,30,`KeyU`,51,`U`,85,`VK_U`,``,``],[0,31,`KeyV`,52,`V`,86,`VK_V`,``,``],[0,32,`KeyW`,53,`W`,87,`VK_W`,``,``],[0,33,`KeyX`,54,`X`,88,`VK_X`,``,``],[0,34,`KeyY`,55,`Y`,89,`VK_Y`,``,``],[0,35,`KeyZ`,56,`Z`,90,`VK_Z`,``,``],[0,36,`Digit1`,22,`1`,49,`VK_1`,``,``],[0,37,`Digit2`,23,`2`,50,`VK_2`,``,``],[0,38,`Digit3`,24,`3`,51,`VK_3`,``,``],[0,39,`Digit4`,25,`4`,52,`VK_4`,``,``],[0,40,`Digit5`,26,`5`,53,`VK_5`,``,``],[0,41,`Digit6`,27,`6`,54,`VK_6`,``,``],[0,42,`Digit7`,28,`7`,55,`VK_7`,``,``],[0,43,`Digit8`,29,`8`,56,`VK_8`,``,``],[0,44,`Digit9`,30,`9`,57,`VK_9`,``,``],[0,45,`Digit0`,21,`0`,48,`VK_0`,``,``],[1,46,`Enter`,3,`Enter`,13,`VK_RETURN`,``,``],[1,47,`Escape`,9,`Escape`,27,`VK_ESCAPE`,``,``],[1,48,`Backspace`,1,`Backspace`,8,`VK_BACK`,``,``],[1,49,`Tab`,2,`Tab`,9,`VK_TAB`,``,``],[1,50,`Space`,10,`Space`,32,`VK_SPACE`,``,``],[0,51,`Minus`,88,`-`,189,`VK_OEM_MINUS`,`-`,`OEM_MINUS`],[0,52,`Equal`,86,`=`,187,`VK_OEM_PLUS`,`=`,`OEM_PLUS`],[0,53,`BracketLeft`,92,`[`,219,`VK_OEM_4`,`[`,`OEM_4`],[0,54,`BracketRight`,94,`]`,221,`VK_OEM_6`,`]`,`OEM_6`],[0,55,`Backslash`,93,`\\\\`,220,`VK_OEM_5`,`\\\\`,`OEM_5`],[0,56,`IntlHash`,0,``,0,``,``,``],[0,57,`Semicolon`,85,`;`,186,`VK_OEM_1`,`;`,`OEM_1`],[0,58,`Quote`,95,`'`,222,`VK_OEM_7`,`'`,`OEM_7`],[0,59,`Backquote`,91,\"`\",192,`VK_OEM_3`,\"`\",`OEM_3`],[0,60,`Comma`,87,`,`,188,`VK_OEM_COMMA`,`,`,`OEM_COMMA`],[0,61,`Period`,89,`.`,190,`VK_OEM_PERIOD`,`.`,`OEM_PERIOD`],[0,62,`Slash`,90,`/`,191,`VK_OEM_2`,`/`,`OEM_2`],[1,63,`CapsLock`,8,`CapsLock`,20,`VK_CAPITAL`,``,``],[1,64,`F1`,59,`F1`,112,`VK_F1`,``,``],[1,65,`F2`,60,`F2`,113,`VK_F2`,``,``],[1,66,`F3`,61,`F3`,114,`VK_F3`,``,``],[1,67,`F4`,62,`F4`,115,`VK_F4`,``,``],[1,68,`F5`,63,`F5`,116,`VK_F5`,``,``],[1,69,`F6`,64,`F6`,117,`VK_F6`,``,``],[1,70,`F7`,65,`F7`,118,`VK_F7`,``,``],[1,71,`F8`,66,`F8`,119,`VK_F8`,``,``],[1,72,`F9`,67,`F9`,120,`VK_F9`,``,``],[1,73,`F10`,68,`F10`,121,`VK_F10`,``,``],[1,74,`F11`,69,`F11`,122,`VK_F11`,``,``],[1,75,`F12`,70,`F12`,123,`VK_F12`,``,``],[1,76,`PrintScreen`,0,``,0,``,``,``],[1,77,`ScrollLock`,84,`ScrollLock`,145,`VK_SCROLL`,``,``],[1,78,`Pause`,7,`PauseBreak`,19,`VK_PAUSE`,``,``],[1,79,`Insert`,19,`Insert`,45,`VK_INSERT`,``,``],[1,80,`Home`,14,`Home`,36,`VK_HOME`,``,``],[1,81,`PageUp`,11,`PageUp`,33,`VK_PRIOR`,``,``],[1,82,`Delete`,20,`Delete`,46,`VK_DELETE`,``,``],[1,83,`End`,13,`End`,35,`VK_END`,``,``],[1,84,`PageDown`,12,`PageDown`,34,`VK_NEXT`,``,``],[1,85,`ArrowRight`,17,`RightArrow`,39,`VK_RIGHT`,`Right`,``],[1,86,`ArrowLeft`,15,`LeftArrow`,37,`VK_LEFT`,`Left`,``],[1,87,`ArrowDown`,18,`DownArrow`,40,`VK_DOWN`,`Down`,``],[1,88,`ArrowUp`,16,`UpArrow`,38,`VK_UP`,`Up`,``],[1,89,`NumLock`,83,`NumLock`,144,`VK_NUMLOCK`,``,``],[1,90,`NumpadDivide`,113,`NumPad_Divide`,111,`VK_DIVIDE`,``,``],[1,91,`NumpadMultiply`,108,`NumPad_Multiply`,106,`VK_MULTIPLY`,``,``],[1,92,`NumpadSubtract`,111,`NumPad_Subtract`,109,`VK_SUBTRACT`,``,``],[1,93,`NumpadAdd`,109,`NumPad_Add`,107,`VK_ADD`,``,``],[1,94,`NumpadEnter`,3,``,0,``,``,``],[1,95,`Numpad1`,99,`NumPad1`,97,`VK_NUMPAD1`,``,``],[1,96,`Numpad2`,100,`NumPad2`,98,`VK_NUMPAD2`,``,``],[1,97,`Numpad3`,101,`NumPad3`,99,`VK_NUMPAD3`,``,``],[1,98,`Numpad4`,102,`NumPad4`,100,`VK_NUMPAD4`,``,``],[1,99,`Numpad5`,103,`NumPad5`,101,`VK_NUMPAD5`,``,``],[1,100,`Numpad6`,104,`NumPad6`,102,`VK_NUMPAD6`,``,``],[1,101,`Numpad7`,105,`NumPad7`,103,`VK_NUMPAD7`,``,``],[1,102,`Numpad8`,106,`NumPad8`,104,`VK_NUMPAD8`,``,``],[1,103,`Numpad9`,107,`NumPad9`,105,`VK_NUMPAD9`,``,``],[1,104,`Numpad0`,98,`NumPad0`,96,`VK_NUMPAD0`,``,``],[1,105,`NumpadDecimal`,112,`NumPad_Decimal`,110,`VK_DECIMAL`,``,``],[0,106,`IntlBackslash`,97,`OEM_102`,226,`VK_OEM_102`,``,``],[1,107,`ContextMenu`,58,`ContextMenu`,93,``,``,``],[1,108,`Power`,0,``,0,``,``,``],[1,109,`NumpadEqual`,0,``,0,``,``,``],[1,110,`F13`,71,`F13`,124,`VK_F13`,``,``],[1,111,`F14`,72,`F14`,125,`VK_F14`,``,``],[1,112,`F15`,73,`F15`,126,`VK_F15`,``,``],[1,113,`F16`,74,`F16`,127,`VK_F16`,``,``],[1,114,`F17`,75,`F17`,128,`VK_F17`,``,``],[1,115,`F18`,76,`F18`,129,`VK_F18`,``,``],[1,116,`F19`,77,`F19`,130,`VK_F19`,``,``],[1,117,`F20`,78,`F20`,131,`VK_F20`,``,``],[1,118,`F21`,79,`F21`,132,`VK_F21`,``,``],[1,119,`F22`,80,`F22`,133,`VK_F22`,``,``],[1,120,`F23`,81,`F23`,134,`VK_F23`,``,``],[1,121,`F24`,82,`F24`,135,`VK_F24`,``,``],[1,122,`Open`,0,``,0,``,``,``],[1,123,`Help`,0,``,0,``,``,``],[1,124,`Select`,0,``,0,``,``,``],[1,125,`Again`,0,``,0,``,``,``],[1,126,`Undo`,0,``,0,``,``,``],[1,127,`Cut`,0,``,0,``,``,``],[1,128,`Copy`,0,``,0,``,``,``],[1,129,`Paste`,0,``,0,``,``,``],[1,130,`Find`,0,``,0,``,``,``],[1,131,`AudioVolumeMute`,117,`AudioVolumeMute`,173,`VK_VOLUME_MUTE`,``,``],[1,132,`AudioVolumeUp`,118,`AudioVolumeUp`,175,`VK_VOLUME_UP`,``,``],[1,133,`AudioVolumeDown`,119,`AudioVolumeDown`,174,`VK_VOLUME_DOWN`,``,``],[1,134,`NumpadComma`,110,`NumPad_Separator`,108,`VK_SEPARATOR`,``,``],[0,135,`IntlRo`,115,`ABNT_C1`,193,`VK_ABNT_C1`,``,``],[1,136,`KanaMode`,0,``,0,``,``,``],[0,137,`IntlYen`,0,``,0,``,``,``],[1,138,`Convert`,0,``,0,``,``,``],[1,139,`NonConvert`,0,``,0,``,``,``],[1,140,`Lang1`,0,``,0,``,``,``],[1,141,`Lang2`,0,``,0,``,``,``],[1,142,`Lang3`,0,``,0,``,``,``],[1,143,`Lang4`,0,``,0,``,``,``],[1,144,`Lang5`,0,``,0,``,``,``],[1,145,`Abort`,0,``,0,``,``,``],[1,146,`Props`,0,``,0,``,``,``],[1,147,`NumpadParenLeft`,0,``,0,``,``,``],[1,148,`NumpadParenRight`,0,``,0,``,``,``],[1,149,`NumpadBackspace`,0,``,0,``,``,``],[1,150,`NumpadMemoryStore`,0,``,0,``,``,``],[1,151,`NumpadMemoryRecall`,0,``,0,``,``,``],[1,152,`NumpadMemoryClear`,0,``,0,``,``,``],[1,153,`NumpadMemoryAdd`,0,``,0,``,``,``],[1,154,`NumpadMemorySubtract`,0,``,0,``,``,``],[1,155,`NumpadClear`,131,`Clear`,12,`VK_CLEAR`,``,``],[1,156,`NumpadClearEntry`,0,``,0,``,``,``],[1,0,``,5,`Ctrl`,17,`VK_CONTROL`,``,``],[1,0,``,4,`Shift`,16,`VK_SHIFT`,``,``],[1,0,``,6,`Alt`,18,`VK_MENU`,``,``],[1,0,``,57,`Meta`,91,`VK_COMMAND`,``,``],[1,157,`ControlLeft`,5,``,0,`VK_LCONTROL`,``,``],[1,158,`ShiftLeft`,4,``,0,`VK_LSHIFT`,``,``],[1,159,`AltLeft`,6,``,0,`VK_LMENU`,``,``],[1,160,`MetaLeft`,57,``,0,`VK_LWIN`,``,``],[1,161,`ControlRight`,5,``,0,`VK_RCONTROL`,``,``],[1,162,`ShiftRight`,4,``,0,`VK_RSHIFT`,``,``],[1,163,`AltRight`,6,``,0,`VK_RMENU`,``,``],[1,164,`MetaRight`,57,``,0,`VK_RWIN`,``,``],[1,165,`BrightnessUp`,0,``,0,``,``,``],[1,166,`BrightnessDown`,0,``,0,``,``,``],[1,167,`MediaPlay`,0,``,0,``,``,``],[1,168,`MediaRecord`,0,``,0,``,``,``],[1,169,`MediaFastForward`,0,``,0,``,``,``],[1,170,`MediaRewind`,0,``,0,``,``,``],[1,171,`MediaTrackNext`,124,`MediaTrackNext`,176,`VK_MEDIA_NEXT_TRACK`,``,``],[1,172,`MediaTrackPrevious`,125,`MediaTrackPrevious`,177,`VK_MEDIA_PREV_TRACK`,``,``],[1,173,`MediaStop`,126,`MediaStop`,178,`VK_MEDIA_STOP`,``,``],[1,174,`Eject`,0,``,0,``,``,``],[1,175,`MediaPlayPause`,127,`MediaPlayPause`,179,`VK_MEDIA_PLAY_PAUSE`,``,``],[1,176,`MediaSelect`,128,`LaunchMediaPlayer`,181,`VK_MEDIA_LAUNCH_MEDIA_SELECT`,``,``],[1,177,`LaunchMail`,129,`LaunchMail`,180,`VK_MEDIA_LAUNCH_MAIL`,``,``],[1,178,`LaunchApp2`,130,`LaunchApp2`,183,`VK_MEDIA_LAUNCH_APP2`,``,``],[1,179,`LaunchApp1`,0,``,0,`VK_MEDIA_LAUNCH_APP1`,``,``],[1,180,`SelectTask`,0,``,0,``,``,``],[1,181,`LaunchScreenSaver`,0,``,0,``,``,``],[1,182,`BrowserSearch`,120,`BrowserSearch`,170,`VK_BROWSER_SEARCH`,``,``],[1,183,`BrowserHome`,121,`BrowserHome`,172,`VK_BROWSER_HOME`,``,``],[1,184,`BrowserBack`,122,`BrowserBack`,166,`VK_BROWSER_BACK`,``,``],[1,185,`BrowserForward`,123,`BrowserForward`,167,`VK_BROWSER_FORWARD`,``,``],[1,186,`BrowserStop`,0,``,0,`VK_BROWSER_STOP`,``,``],[1,187,`BrowserRefresh`,0,``,0,`VK_BROWSER_REFRESH`,``,``],[1,188,`BrowserFavorites`,0,``,0,`VK_BROWSER_FAVORITES`,``,``],[1,189,`ZoomToggle`,0,``,0,``,``,``],[1,190,`MailReply`,0,``,0,``,``,``],[1,191,`MailForward`,0,``,0,``,``,``],[1,192,`MailSend`,0,``,0,``,``,``],[1,0,``,114,`KeyInComposition`,229,``,``,``],[1,0,``,116,`ABNT_C2`,194,`VK_ABNT_C2`,``,``],[1,0,``,96,`OEM_8`,223,`VK_OEM_8`,``,``],[1,0,``,0,``,0,`VK_KANA`,``,``],[1,0,``,0,``,0,`VK_HANGUL`,``,``],[1,0,``,0,``,0,`VK_JUNJA`,``,``],[1,0,``,0,``,0,`VK_FINAL`,``,``],[1,0,``,0,``,0,`VK_HANJA`,``,``],[1,0,``,0,``,0,`VK_KANJI`,``,``],[1,0,``,0,``,0,`VK_CONVERT`,``,``],[1,0,``,0,``,0,`VK_NONCONVERT`,``,``],[1,0,``,0,``,0,`VK_ACCEPT`,``,``],[1,0,``,0,``,0,`VK_MODECHANGE`,``,``],[1,0,``,0,``,0,`VK_SELECT`,``,``],[1,0,``,0,``,0,`VK_PRINT`,``,``],[1,0,``,0,``,0,`VK_EXECUTE`,``,``],[1,0,``,0,``,0,`VK_SNAPSHOT`,``,``],[1,0,``,0,``,0,`VK_HELP`,``,``],[1,0,``,0,``,0,`VK_APPS`,``,``],[1,0,``,0,``,0,`VK_PROCESSKEY`,``,``],[1,0,``,0,``,0,`VK_PACKET`,``,``],[1,0,``,0,``,0,`VK_DBE_SBCSCHAR`,``,``],[1,0,``,0,``,0,`VK_DBE_DBCSCHAR`,``,``],[1,0,``,0,``,0,`VK_ATTN`,``,``],[1,0,``,0,``,0,`VK_CRSEL`,``,``],[1,0,``,0,``,0,`VK_EXSEL`,``,``],[1,0,``,0,``,0,`VK_EREOF`,``,``],[1,0,``,0,``,0,`VK_PLAY`,``,``],[1,0,``,0,``,0,`VK_ZOOM`,``,``],[1,0,``,0,``,0,`VK_NONAME`,``,``],[1,0,``,0,``,0,`VK_PA1`,``,``],[1,0,``,0,``,0,`VK_OEM_CLEAR`,``,``]],t=[],n=[];for(let r of e){let[e,i,a,o,s,c,l,u,d]=r;if(n[i]||(n[i]=!0,Pt[a]=i,Ft[a.toLowerCase()]=i,e&&(It[i]=o)),!t[o]){if(t[o]=!0,!s)throw Error(`String representation missing for key code ${o} around scan code ${a}`);At.define(o,s),jt.define(o,u||s),Mt.define(o,d||u||s)}c&&(Nt[c]=o)}})();var Lt;(function(e){function t(e){return At.keyCodeToStr(e)}e.toString=t;function n(e){return At.strToKeyCode(e)}e.fromString=n;function r(e){return jt.keyCodeToStr(e)}e.toUserSettingsUS=r;function i(e){return Mt.keyCodeToStr(e)}e.toUserSettingsGeneral=i;function a(e){return jt.strToKeyCode(e)||Mt.strToKeyCode(e)}e.fromUserSettings=a;function o(e){if(e>=98&&e<=113)return null;switch(e){case 16:return`Up`;case 18:return`Down`;case 15:return`Left`;case 17:return`Right`}return At.keyCodeToStr(e)}e.toElectronAccelerator=o})(Lt||={});function Rt(e,t){return(e|(t&65535)<<16>>>0)>>>0}let zt;const Bt=globalThis.vscode;if(Bt!==void 0&&Bt.process!==void 0){let e=Bt.process;zt={get platform(){return e.platform},get arch(){return e.arch},get env(){return e.env},cwd(){return e.cwd()}}}else zt=typeof process<`u`&&typeof process?.versions?.node==`string`?{get platform(){return process.platform},get arch(){return process.arch},get env(){return process.env},cwd(){return process.env.VSCODE_CWD||process.cwd()}}:{get platform(){return ge?`win32`:_e?`darwin`:`linux`},get arch(){},get env(){return{}},cwd(){return`/`}};const Vt=zt.cwd,Ht=zt.env,Ut=zt.platform;var Wt=class extends Error{constructor(e,t,n){let r;typeof t==`string`&&t.indexOf(`not `)===0?(r=`must not be`,t=t.replace(/^not /,``)):r=`must be`;let i=`The \"${e}\" ${e.indexOf(`.`)===-1?`argument`:`property`} ${r} of type ${t}`;i+=`. Received type ${typeof n}`,super(i),this.code=`ERR_INVALID_ARG_TYPE`}};function Gt(e,t){if(typeof e!=`object`||!e)throw new Wt(t,`Object`,e)}function I(e,t){if(typeof e!=`string`)throw new Wt(t,`string`,e)}const L=Ut===`win32`;function R(e){return e===47||e===92}function Kt(e){return e===47}function z(e){return e>=65&&e<=90||e>=97&&e<=122}function qt(e,t,n,r){let i=``,a=0,o=-1,s=0,c=0;for(let l=0;l<=e.length;++l){if(l<e.length)c=e.charCodeAt(l);else if(r(c))break;else c=47;if(r(c)){if(o!==l-1&&s!==1){if(s===2){if(i.length<2||a!==2||i.charCodeAt(i.length-1)!==46||i.charCodeAt(i.length-2)!==46){if(i.length>2){let e=i.lastIndexOf(n);e===-1?(i=``,a=0):(i=i.slice(0,e),a=i.length-1-i.lastIndexOf(n)),o=l,s=0;continue}if(i.length!==0){i=``,a=0,o=l,s=0;continue}}t&&(i+=i.length>0?`${n}..`:`..`,a=2)}else i.length>0?i+=`${n}${e.slice(o+1,l)}`:i=e.slice(o+1,l),a=l-o-1}o=l,s=0}else c===46&&s!==-1?++s:s=-1}return i}function Jt(e){return e?`${e[0]===`.`?``:`.`}${e}`:``}function Yt(e,t){Gt(t,`pathObject`);let n=t.dir||t.root,r=t.base||`${t.name||``}${Jt(t.ext)}`;return n?n===t.root?`${n}${r}`:`${n}${e}${r}`:r}const B={resolve(...e){let t=``,n=``,r=!1;for(let i=e.length-1;i>=-1;i--){let a;if(i>=0){if(a=e[i],I(a,`paths[${i}]`),a.length===0)continue}else t.length===0?a=Vt():(a=Ht[`=${t}`]||Vt(),(a===void 0||a.slice(0,2).toLowerCase()!==t.toLowerCase()&&a.charCodeAt(2)===92)&&(a=`${t}\\\\`));let o=a.length,s=0,c=``,l=!1,u=a.charCodeAt(0);if(o===1)R(u)&&(s=1,l=!0);else if(R(u)){if(l=!0,R(a.charCodeAt(1))){let e=2,t=e;for(;e<o&&!R(a.charCodeAt(e));)e++;if(e<o&&e!==t){let n=a.slice(t,e);for(t=e;e<o&&R(a.charCodeAt(e));)e++;if(e<o&&e!==t){for(t=e;e<o&&!R(a.charCodeAt(e));)e++;(e===o||e!==t)&&(c=`\\\\\\\\${n}\\\\${a.slice(t,e)}`,s=e)}}}else s=1}else z(u)&&a.charCodeAt(1)===58&&(c=a.slice(0,2),s=2,o>2&&R(a.charCodeAt(2))&&(l=!0,s=3));if(c.length>0){if(t.length>0){if(c.toLowerCase()!==t.toLowerCase())continue}else t=c}if(r){if(t.length>0)break}else if(n=`${a.slice(s)}\\\\${n}`,r=l,l&&t.length>0)break}return n=qt(n,!r,`\\\\`,R),r?`${t}\\\\${n}`:`${t}${n}`||`.`},normalize(e){I(e,`path`);let t=e.length;if(t===0)return`.`;let n=0,r,i=!1,a=e.charCodeAt(0);if(t===1)return Kt(a)?`\\\\`:e;if(R(a)){if(i=!0,R(e.charCodeAt(1))){let i=2,a=i;for(;i<t&&!R(e.charCodeAt(i));)i++;if(i<t&&i!==a){let o=e.slice(a,i);for(a=i;i<t&&R(e.charCodeAt(i));)i++;if(i<t&&i!==a){for(a=i;i<t&&!R(e.charCodeAt(i));)i++;if(i===t)return`\\\\\\\\${o}\\\\${e.slice(a)}\\\\`;i!==a&&(r=`\\\\\\\\${o}\\\\${e.slice(a,i)}`,n=i)}}}else n=1}else z(a)&&e.charCodeAt(1)===58&&(r=e.slice(0,2),n=2,t>2&&R(e.charCodeAt(2))&&(i=!0,n=3));let o=n<t?qt(e.slice(n),!i,`\\\\`,R):``;if(o.length===0&&!i&&(o=`.`),o.length>0&&R(e.charCodeAt(t-1))&&(o+=`\\\\`),!i&&r===void 0&&e.includes(`:`)){if(o.length>=2&&z(o.charCodeAt(0))&&o.charCodeAt(1)===58)return`.\\\\${o}`;let n=e.indexOf(`:`);do if(n===t-1||R(e.charCodeAt(n+1)))return`.\\\\${o}`;while((n=e.indexOf(`:`,n+1))!==-1)}return r===void 0?i?`\\\\${o}`:o:i?`${r}\\\\${o}`:`${r}${o}`},isAbsolute(e){I(e,`path`);let t=e.length;if(t===0)return!1;let n=e.charCodeAt(0);return R(n)||t>2&&z(n)&&e.charCodeAt(1)===58&&R(e.charCodeAt(2))},join(...e){if(e.length===0)return`.`;let t,n;for(let r=0;r<e.length;++r){let i=e[r];I(i,`path`),i.length>0&&(t===void 0?t=n=i:t+=`\\\\${i}`)}if(t===void 0)return`.`;let r=!0,i=0;if(typeof n==`string`&&R(n.charCodeAt(0))){++i;let e=n.length;e>1&&R(n.charCodeAt(1))&&(++i,e>2&&(R(n.charCodeAt(2))?++i:r=!1))}if(r){for(;i<t.length&&R(t.charCodeAt(i));)i++;i>=2&&(t=`\\\\${t.slice(i)}`)}return B.normalize(t)},relative(e,t){if(I(e,`from`),I(t,`to`),e===t)return``;let n=B.resolve(e),r=B.resolve(t);if(n===r||(e=n.toLowerCase(),t=r.toLowerCase(),e===t))return``;if(n.length!==e.length||r.length!==t.length){let e=n.split(`\\\\`),t=r.split(`\\\\`);e[e.length-1]===``&&e.pop(),t[t.length-1]===``&&t.pop();let i=e.length,a=t.length,o=i<a?i:a,s=0;for(;s<o&&e[s].toLowerCase()===t[s].toLowerCase();s++);return s===0?r:s===o?a>o?t.slice(s).join(`\\\\`):i>o?`..\\\\`.repeat(i-1-s)+`..`:``:`..\\\\`.repeat(i-s)+t.slice(s).join(`\\\\`)}let i=0;for(;i<e.length&&e.charCodeAt(i)===92;)i++;let a=e.length;for(;a-1>i&&e.charCodeAt(a-1)===92;)a--;let o=a-i,s=0;for(;s<t.length&&t.charCodeAt(s)===92;)s++;let c=t.length;for(;c-1>s&&t.charCodeAt(c-1)===92;)c--;let l=c-s,u=o<l?o:l,d=-1,f=0;for(;f<u;f++){let n=e.charCodeAt(i+f);if(n!==t.charCodeAt(s+f))break;n===92&&(d=f)}if(f!==u){if(d===-1)return r}else{if(l>u){if(t.charCodeAt(s+f)===92)return r.slice(s+f+1);if(f===2)return r.slice(s+f)}o>u&&(e.charCodeAt(i+f)===92?d=f:f===2&&(d=3)),d===-1&&(d=0)}let p=``;for(f=i+d+1;f<=a;++f)(f===a||e.charCodeAt(f)===92)&&(p+=p.length===0?`..`:`\\\\..`);return s+=d,p.length>0?`${p}${r.slice(s,c)}`:(r.charCodeAt(s)===92&&++s,r.slice(s,c))},toNamespacedPath(e){if(typeof e!=`string`||e.length===0)return e;let t=B.resolve(e);if(t.length<=2)return e;if(t.charCodeAt(0)===92){if(t.charCodeAt(1)===92){let e=t.charCodeAt(2);if(e!==63&&e!==46)return`\\\\\\\\?\\\\UNC\\\\${t.slice(2)}`}}else if(z(t.charCodeAt(0))&&t.charCodeAt(1)===58&&t.charCodeAt(2)===92)return`\\\\\\\\?\\\\${t}`;return t},dirname(e){I(e,`path`);let t=e.length;if(t===0)return`.`;let n=-1,r=0,i=e.charCodeAt(0);if(t===1)return R(i)?e:`.`;if(R(i)){if(n=r=1,R(e.charCodeAt(1))){let i=2,a=i;for(;i<t&&!R(e.charCodeAt(i));)i++;if(i<t&&i!==a){for(a=i;i<t&&R(e.charCodeAt(i));)i++;if(i<t&&i!==a){for(a=i;i<t&&!R(e.charCodeAt(i));)i++;if(i===t)return e;i!==a&&(n=r=i+1)}}}}else z(i)&&e.charCodeAt(1)===58&&(n=t>2&&R(e.charCodeAt(2))?3:2,r=n);let a=-1,o=!0;for(let n=t-1;n>=r;--n)if(R(e.charCodeAt(n))){if(!o){a=n;break}}else o=!1;if(a===-1){if(n===-1)return`.`;a=n}return e.slice(0,a)},basename(e,t){t!==void 0&&I(t,`suffix`),I(e,`path`);let n=0,r=-1,i=!0,a;if(e.length>=2&&z(e.charCodeAt(0))&&e.charCodeAt(1)===58&&(n=2),t!==void 0&&t.length>0&&t.length<=e.length){if(t===e)return``;let o=t.length-1,s=-1;for(a=e.length-1;a>=n;--a){let c=e.charCodeAt(a);if(R(c)){if(!i){n=a+1;break}}else s===-1&&(i=!1,s=a+1),o>=0&&(c===t.charCodeAt(o)?--o===-1&&(r=a):(o=-1,r=s))}return n===r?r=s:r===-1&&(r=e.length),e.slice(n,r)}for(a=e.length-1;a>=n;--a)if(R(e.charCodeAt(a))){if(!i){n=a+1;break}}else r===-1&&(i=!1,r=a+1);return r===-1?``:e.slice(n,r)},extname(e){I(e,`path`);let t=0,n=-1,r=0,i=-1,a=!0,o=0;e.length>=2&&e.charCodeAt(1)===58&&z(e.charCodeAt(0))&&(t=r=2);for(let s=e.length-1;s>=t;--s){let t=e.charCodeAt(s);if(R(t)){if(!a){r=s+1;break}}else i===-1&&(a=!1,i=s+1),t===46?n===-1?n=s:o!==1&&(o=1):n!==-1&&(o=-1)}return n===-1||i===-1||o===0||o===1&&n===i-1&&n===r+1?``:e.slice(n,i)},format:Yt.bind(null,`\\\\`),parse(e){I(e,`path`);let t={root:``,dir:``,base:``,ext:``,name:``};if(e.length===0)return t;let n=e.length,r=0,i=e.charCodeAt(0);if(n===1)return R(i)?(t.root=t.dir=e,t):(t.base=t.name=e,t);if(R(i)){if(r=1,R(e.charCodeAt(1))){let t=2,i=t;for(;t<n&&!R(e.charCodeAt(t));)t++;if(t<n&&t!==i){for(i=t;t<n&&R(e.charCodeAt(t));)t++;if(t<n&&t!==i){for(i=t;t<n&&!R(e.charCodeAt(t));)t++;t===n?r=t:t!==i&&(r=t+1)}}}}else if(z(i)&&e.charCodeAt(1)===58){if(n<=2)return t.root=t.dir=e,t;if(r=2,R(e.charCodeAt(2))){if(n===3)return t.root=t.dir=e,t;r=3}}r>0&&(t.root=e.slice(0,r));let a=-1,o=r,s=-1,c=!0,l=e.length-1,u=0;for(;l>=r;--l)if(i=e.charCodeAt(l),R(i)){if(!c){o=l+1;break}}else s===-1&&(c=!1,s=l+1),i===46?a===-1?a=l:u!==1&&(u=1):a!==-1&&(u=-1);return s!==-1&&(a===-1||u===0||u===1&&a===s-1&&a===o+1?t.base=t.name=e.slice(o,s):(t.name=e.slice(o,a),t.base=e.slice(o,s),t.ext=e.slice(a,s))),t.dir=o>0&&o!==r?e.slice(0,o-1):t.root,t},sep:`\\\\`,delimiter:`;`,win32:null,posix:null},Xt=(()=>{if(L){let e=/\\\\/g;return()=>{let t=Vt().replace(e,`/`);return t.slice(t.indexOf(`/`))}}return()=>Vt()})(),V={resolve(...e){let t=``,n=!1;for(let r=e.length-1;r>=0&&!n;r--){let i=e[r];I(i,`paths[${r}]`),i.length!==0&&(t=`${i}/${t}`,n=i.charCodeAt(0)===47)}if(!n){let e=Xt();t=`${e}/${t}`,n=e.charCodeAt(0)===47}return t=qt(t,!n,`/`,Kt),n?`/${t}`:t.length>0?t:`.`},normalize(e){if(I(e,`path`),e.length===0)return`.`;let t=e.charCodeAt(0)===47,n=e.charCodeAt(e.length-1)===47;return e=qt(e,!t,`/`,Kt),e.length===0?t?`/`:n?`./`:`.`:(n&&(e+=`/`),t?`/${e}`:e)},isAbsolute(e){return I(e,`path`),e.length>0&&e.charCodeAt(0)===47},join(...e){if(e.length===0)return`.`;let t=[];for(let n=0;n<e.length;++n){let r=e[n];I(r,`path`),r.length>0&&t.push(r)}return t.length===0?`.`:V.normalize(t.join(`/`))},relative(e,t){if(I(e,`from`),I(t,`to`),e===t||(e=V.resolve(e),t=V.resolve(t),e===t))return``;let n=e.length,r=n-1,i=t.length-1,a=r<i?r:i,o=-1,s=0;for(;s<a;s++){let n=e.charCodeAt(1+s);if(n!==t.charCodeAt(1+s))break;n===47&&(o=s)}if(s===a){if(i>a){if(t.charCodeAt(1+s)===47)return t.slice(1+s+1);if(s===0)return t.slice(1+s)}else r>a&&(e.charCodeAt(1+s)===47?o=s:s===0&&(o=0))}let c=``;for(s=1+o+1;s<=n;++s)(s===n||e.charCodeAt(s)===47)&&(c+=c.length===0?`..`:`/..`);return`${c}${t.slice(1+o)}`},toNamespacedPath(e){return e},dirname(e){if(I(e,`path`),e.length===0)return`.`;let t=e.charCodeAt(0)===47,n=-1,r=!0;for(let t=e.length-1;t>=1;--t)if(e.charCodeAt(t)===47){if(!r){n=t;break}}else r=!1;return n===-1?t?`/`:`.`:t&&n===1?`//`:e.slice(0,n)},basename(e,t){t!==void 0&&I(t,`suffix`),I(e,`path`);let n=0,r=-1,i=!0,a;if(t!==void 0&&t.length>0&&t.length<=e.length){if(t===e)return``;let o=t.length-1,s=-1;for(a=e.length-1;a>=0;--a){let c=e.charCodeAt(a);if(c===47){if(!i){n=a+1;break}}else s===-1&&(i=!1,s=a+1),o>=0&&(c===t.charCodeAt(o)?--o===-1&&(r=a):(o=-1,r=s))}return n===r?r=s:r===-1&&(r=e.length),e.slice(n,r)}for(a=e.length-1;a>=0;--a)if(e.charCodeAt(a)===47){if(!i){n=a+1;break}}else r===-1&&(i=!1,r=a+1);return r===-1?``:e.slice(n,r)},extname(e){I(e,`path`);let t=-1,n=0,r=-1,i=!0,a=0;for(let o=e.length-1;o>=0;--o){let s=e[o];if(s===`/`){if(!i){n=o+1;break}}else r===-1&&(i=!1,r=o+1),s===`.`?t===-1?t=o:a!==1&&(a=1):t!==-1&&(a=-1)}return t===-1||r===-1||a===0||a===1&&t===r-1&&t===n+1?``:e.slice(t,r)},format:Yt.bind(null,`/`),parse(e){I(e,`path`);let t={root:``,dir:``,base:``,ext:``,name:``};if(e.length===0)return t;let n=e.charCodeAt(0)===47,r;n?(t.root=`/`,r=1):r=0;let i=-1,a=0,o=-1,s=!0,c=e.length-1,l=0;for(;c>=r;--c){let t=e.charCodeAt(c);if(t===47){if(!s){a=c+1;break}}else o===-1&&(s=!1,o=c+1),t===46?i===-1?i=c:l!==1&&(l=1):i!==-1&&(l=-1)}if(o!==-1){let r=a===0&&n?1:a;i===-1||l===0||l===1&&i===o-1&&i===a+1?t.base=t.name=e.slice(r,o):(t.name=e.slice(r,i),t.base=e.slice(r,o),t.ext=e.slice(i,o))}return a>0?t.dir=e.slice(0,a-1):n&&(t.dir=`/`),t},sep:`/`,delimiter:`:`,win32:null,posix:null};V.win32=B.win32=B,V.posix=B.posix=V,L?B.normalize:V.normalize,L?B.resolve:V.resolve,L?B.relative:V.relative,L?B.dirname:V.dirname,L?B.basename:V.basename,L?B.extname:V.extname,L?B.sep:V.sep;const Zt=/^\\w[\\w\\d+.-]*$/,Qt=/^\\//,$t=/^\\/\\//;function en(e,t){if(!e.scheme&&t)throw Error(`[UriError]: Scheme is missing: {scheme: \"\", authority: \"${e.authority}\", path: \"${e.path}\", query: \"${e.query}\", fragment: \"${e.fragment}\"}`);if(e.scheme&&!Zt.test(e.scheme))throw Error(`[UriError]: Scheme contains illegal characters.`);if(e.path){if(e.authority){if(!Qt.test(e.path))throw Error(`[UriError]: If a URI contains an authority component, then the path component must either be empty or begin with a slash (\"/\") character`)}else if($t.test(e.path))throw Error(`[UriError]: If a URI does not contain an authority component, then the path cannot begin with two slash characters (\"//\")`)}}function tn(e,t){return!e&&!t?`file`:e}function nn(e,t){switch(e){case`https`:case`http`:case`file`:t?t[0]!==rn&&(t=rn+t):t=rn}return t}const rn=`/`,an=/^(([^:/?#]+?):)?(\\/\\/([^/?#]*))?([^?#]*)(\\?([^#]*))?(#(.*))?/;var on=class e{static isUri(t){return t instanceof e?!0:!t||typeof t!=`object`?!1:typeof t.authority==`string`&&typeof t.fragment==`string`&&typeof t.path==`string`&&typeof t.query==`string`&&typeof t.scheme==`string`&&typeof t.fsPath==`string`&&typeof t.with==`function`&&typeof t.toString==`function`}constructor(e,t,n,r,i,a=!1){typeof e==`object`?(this.scheme=e.scheme||``,this.authority=e.authority||``,this.path=e.path||``,this.query=e.query||``,this.fragment=e.fragment||``):(this.scheme=tn(e,a),this.authority=t||``,this.path=nn(this.scheme,n||``),this.query=r||``,this.fragment=i||``,en(this,a))}get fsPath(){return dn(this,!1)}with(e){if(!e)return this;let{scheme:t,authority:n,path:r,query:i,fragment:a}=e;return t===void 0?t=this.scheme:t===null&&(t=``),n===void 0?n=this.authority:n===null&&(n=``),r===void 0?r=this.path:r===null&&(r=``),i===void 0?i=this.query:i===null&&(i=``),a===void 0?a=this.fragment:a===null&&(a=``),t===this.scheme&&n===this.authority&&r===this.path&&i===this.query&&a===this.fragment?this:new H(t,n,r,i,a)}static parse(e,t=!1){let n=an.exec(e);return n?new H(n[2]||``,hn(n[4]||``),hn(n[5]||``),hn(n[7]||``),hn(n[9]||``),t):new H(``,``,``,``,``)}static file(e){let t=``;if(ge&&(e=e.replace(/\\\\/g,`/`)),e[0]===`/`&&e[1]===`/`){let n=e.indexOf(`/`,2);n===-1?(t=e.substring(2),e=`/`):(t=e.substring(2,n),e=e.substring(n)||`/`)}return new H(`file`,t,e,``,``)}static from(e,t){return new H(e.scheme,e.authority,e.path,e.query,e.fragment,t)}static joinPath(t,...n){if(!t.path)throw Error(`[UriError]: cannot call joinPath on URI without path`);let r;return r=ge&&t.scheme===`file`?e.file(B.join(dn(t,!0),...n)).path:V.join(t.path,...n),t.with({path:r})}toString(e=!1){return fn(this,e)}toJSON(){return this}static revive(t){if(!t||t instanceof e)return t;{let e=new H(t);return e._formatted=t.external??null,e._fsPath=t._sep===sn?t.fsPath??null:null,e}}};const sn=ge?1:void 0;var H=class extends on{constructor(){super(...arguments),this._formatted=null,this._fsPath=null}get fsPath(){return this._fsPath||=dn(this,!1),this._fsPath}toString(e=!1){return e?fn(this,!0):(this._formatted||=fn(this,!1),this._formatted)}toJSON(){let e={$mid:1};return this._fsPath&&(e.fsPath=this._fsPath,e._sep=sn),this._formatted&&(e.external=this._formatted),this.path&&(e.path=this.path),this.scheme&&(e.scheme=this.scheme),this.authority&&(e.authority=this.authority),this.query&&(e.query=this.query),this.fragment&&(e.fragment=this.fragment),e}};const cn={58:`%3A`,47:`%2F`,63:`%3F`,35:`%23`,91:`%5B`,93:`%5D`,64:`%40`,33:`%21`,36:`%24`,38:`%26`,39:`%27`,40:`%28`,41:`%29`,42:`%2A`,43:`%2B`,44:`%2C`,59:`%3B`,61:`%3D`,32:`%20`};function ln(e,t,n){let r,i=-1;for(let a=0;a<e.length;a++){let o=e.charCodeAt(a);if(o>=97&&o<=122||o>=65&&o<=90||o>=48&&o<=57||o===45||o===46||o===95||o===126||t&&o===47||n&&o===91||n&&o===93||n&&o===58)i!==-1&&(r+=encodeURIComponent(e.substring(i,a)),i=-1),r!==void 0&&(r+=e.charAt(a));else{r===void 0&&(r=e.substr(0,a));let t=cn[o];t===void 0?i===-1&&(i=a):(i!==-1&&(r+=encodeURIComponent(e.substring(i,a)),i=-1),r+=t)}}return i!==-1&&(r+=encodeURIComponent(e.substring(i))),r===void 0?e:r}function un(e){let t;for(let n=0;n<e.length;n++){let r=e.charCodeAt(n);r===35||r===63?(t===void 0&&(t=e.substr(0,n)),t+=cn[r]):t!==void 0&&(t+=e[n])}return t===void 0?e:t}function dn(e,t){let n;return n=e.authority&&e.path.length>1&&e.scheme===`file`?`//${e.authority}${e.path}`:e.path.charCodeAt(0)===47&&(e.path.charCodeAt(1)>=65&&e.path.charCodeAt(1)<=90||e.path.charCodeAt(1)>=97&&e.path.charCodeAt(1)<=122)&&e.path.charCodeAt(2)===58?t?e.path.substr(1):e.path[1].toLowerCase()+e.path.substr(2):e.path,ge&&(n=n.replace(/\\//g,`\\\\`)),n}function fn(e,t){let n=t?un:ln,r=``,{scheme:i,authority:a,path:o,query:s,fragment:c}=e;if(i&&(r+=i,r+=`:`),(a||i===`file`)&&(r+=`/`,r+=`/`),a){let e=a.indexOf(`@`);if(e!==-1){let t=a.substr(0,e);a=a.substr(e+1),e=t.lastIndexOf(`:`),e===-1?r+=n(t,!1,!1):(r+=n(t.substr(0,e),!1,!1),r+=`:`,r+=n(t.substr(e+1),!1,!0)),r+=`@`}a=a.toLowerCase(),e=a.lastIndexOf(`:`),e===-1?r+=n(a,!1,!0):(r+=n(a.substr(0,e),!1,!0),r+=a.substr(e))}if(o){if(o.length>=3&&o.charCodeAt(0)===47&&o.charCodeAt(2)===58){let e=o.charCodeAt(1);e>=65&&e<=90&&(o=`/${String.fromCharCode(e+32)}:${o.substr(3)}`)}else if(o.length>=2&&o.charCodeAt(1)===58){let e=o.charCodeAt(0);e>=65&&e<=90&&(o=`${String.fromCharCode(e+32)}:${o.substr(2)}`)}r+=n(o,!0,!1)}return s&&(r+=`?`,r+=n(s,!1,!1)),c&&(r+=`#`,r+=t?c:ln(c,!1,!1)),r}function pn(e){try{return decodeURIComponent(e)}catch{return e.length>3?e.substr(0,3)+pn(e.substr(3)):e}}const mn=/(%[0-9A-Za-z][0-9A-Za-z])+/g;function hn(e){return e.match(mn)?e.replace(mn,e=>pn(e)):e}var gn=class e extends F{constructor(e,t,n,r){super(e,t,n,r),this.selectionStartLineNumber=e,this.selectionStartColumn=t,this.positionLineNumber=n,this.positionColumn=r}toString(){return`[`+this.selectionStartLineNumber+`,`+this.selectionStartColumn+` -> `+this.positionLineNumber+`,`+this.positionColumn+`]`}equalsSelection(t){return e.selectionsEqual(this,t)}static selectionsEqual(e,t){return e.selectionStartLineNumber===t.selectionStartLineNumber&&e.selectionStartColumn===t.selectionStartColumn&&e.positionLineNumber===t.positionLineNumber&&e.positionColumn===t.positionColumn}getDirection(){return this.selectionStartLineNumber===this.startLineNumber&&this.selectionStartColumn===this.startColumn?0:1}setEndPosition(t,n){return this.getDirection()===0?new e(this.startLineNumber,this.startColumn,t,n):new e(t,n,this.startLineNumber,this.startColumn)}getPosition(){return new P(this.positionLineNumber,this.positionColumn)}getSelectionStart(){return new P(this.selectionStartLineNumber,this.selectionStartColumn)}setStartPosition(t,n){return this.getDirection()===0?new e(t,n,this.endLineNumber,this.endColumn):new e(this.endLineNumber,this.endColumn,t,n)}static fromPositions(t,n=t){return new e(t.lineNumber,t.column,n.lineNumber,n.column)}static fromRange(t,n){return n===0?new e(t.startLineNumber,t.startColumn,t.endLineNumber,t.endColumn):new e(t.endLineNumber,t.endColumn,t.startLineNumber,t.startColumn)}static liftSelection(t){return new e(t.selectionStartLineNumber,t.selectionStartColumn,t.positionLineNumber,t.positionColumn)}static selectionsArrEqual(e,t){if(e&&!t||!e&&t)return!1;if(!e&&!t)return!0;if(e.length!==t.length)return!1;for(let n=0,r=e.length;n<r;n++)if(!this.selectionsEqual(e[n],t[n]))return!1;return!0}static isISelection(e){return!!e&&typeof e.selectionStartLineNumber==`number`&&typeof e.selectionStartColumn==`number`&&typeof e.positionLineNumber==`number`&&typeof e.positionColumn==`number`}static createWithDirection(t,n,r,i,a){return a===0?new e(t,n,r,i):new e(r,i,t,n)}};const _n=Object.create(null);function U(e,t){if(d(t)){let n=_n[t];if(n===void 0)throw Error(`${e} references an unknown codicon: ${t}`);t=n}return _n[e]=t,{id:e}}const vn={add:U(`add`,6e4),plus:U(`plus`,6e4),gistNew:U(`gist-new`,6e4),repoCreate:U(`repo-create`,6e4),lightbulb:U(`lightbulb`,60001),lightBulb:U(`light-bulb`,60001),repo:U(`repo`,60002),repoDelete:U(`repo-delete`,60002),gistFork:U(`gist-fork`,60003),repoForked:U(`repo-forked`,60003),gitPullRequest:U(`git-pull-request`,60004),gitPullRequestAbandoned:U(`git-pull-request-abandoned`,60004),recordKeys:U(`record-keys`,60005),keyboard:U(`keyboard`,60005),tag:U(`tag`,60006),gitPullRequestLabel:U(`git-pull-request-label`,60006),tagAdd:U(`tag-add`,60006),tagRemove:U(`tag-remove`,60006),person:U(`person`,60007),personFollow:U(`person-follow`,60007),personOutline:U(`person-outline`,60007),personFilled:U(`person-filled`,60007),sourceControl:U(`source-control`,60008),mirror:U(`mirror`,60009),mirrorPublic:U(`mirror-public`,60009),star:U(`star`,60010),starAdd:U(`star-add`,60010),starDelete:U(`star-delete`,60010),starEmpty:U(`star-empty`,60010),comment:U(`comment`,60011),commentAdd:U(`comment-add`,60011),alert:U(`alert`,60012),warning:U(`warning`,60012),search:U(`search`,60013),searchSave:U(`search-save`,60013),logOut:U(`log-out`,60014),signOut:U(`sign-out`,60014),logIn:U(`log-in`,60015),signIn:U(`sign-in`,60015),eye:U(`eye`,60016),eyeUnwatch:U(`eye-unwatch`,60016),eyeWatch:U(`eye-watch`,60016),circleFilled:U(`circle-filled`,60017),primitiveDot:U(`primitive-dot`,60017),closeDirty:U(`close-dirty`,60017),debugBreakpoint:U(`debug-breakpoint`,60017),debugBreakpointDisabled:U(`debug-breakpoint-disabled`,60017),debugHint:U(`debug-hint`,60017),terminalDecorationSuccess:U(`terminal-decoration-success`,60017),primitiveSquare:U(`primitive-square`,60018),edit:U(`edit`,60019),pencil:U(`pencil`,60019),info:U(`info`,60020),issueOpened:U(`issue-opened`,60020),gistPrivate:U(`gist-private`,60021),gitForkPrivate:U(`git-fork-private`,60021),lock:U(`lock`,60021),mirrorPrivate:U(`mirror-private`,60021),close:U(`close`,60022),removeClose:U(`remove-close`,60022),x:U(`x`,60022),repoSync:U(`repo-sync`,60023),sync:U(`sync`,60023),clone:U(`clone`,60024),desktopDownload:U(`desktop-download`,60024),beaker:U(`beaker`,60025),microscope:U(`microscope`,60025),vm:U(`vm`,60026),deviceDesktop:U(`device-desktop`,60026),file:U(`file`,60027),more:U(`more`,60028),ellipsis:U(`ellipsis`,60028),kebabHorizontal:U(`kebab-horizontal`,60028),mailReply:U(`mail-reply`,60029),reply:U(`reply`,60029),organization:U(`organization`,60030),organizationFilled:U(`organization-filled`,60030),organizationOutline:U(`organization-outline`,60030),newFile:U(`new-file`,60031),fileAdd:U(`file-add`,60031),newFolder:U(`new-folder`,60032),fileDirectoryCreate:U(`file-directory-create`,60032),trash:U(`trash`,60033),trashcan:U(`trashcan`,60033),history:U(`history`,60034),clock:U(`clock`,60034),folder:U(`folder`,60035),fileDirectory:U(`file-directory`,60035),symbolFolder:U(`symbol-folder`,60035),logoGithub:U(`logo-github`,60036),markGithub:U(`mark-github`,60036),github:U(`github`,60036),terminal:U(`terminal`,60037),console:U(`console`,60037),repl:U(`repl`,60037),zap:U(`zap`,60038),symbolEvent:U(`symbol-event`,60038),error:U(`error`,60039),stop:U(`stop`,60039),variable:U(`variable`,60040),symbolVariable:U(`symbol-variable`,60040),array:U(`array`,60042),symbolArray:U(`symbol-array`,60042),symbolModule:U(`symbol-module`,60043),symbolPackage:U(`symbol-package`,60043),symbolNamespace:U(`symbol-namespace`,60043),symbolObject:U(`symbol-object`,60043),symbolMethod:U(`symbol-method`,60044),symbolFunction:U(`symbol-function`,60044),symbolConstructor:U(`symbol-constructor`,60044),symbolBoolean:U(`symbol-boolean`,60047),symbolNull:U(`symbol-null`,60047),symbolNumeric:U(`symbol-numeric`,60048),symbolNumber:U(`symbol-number`,60048),symbolStructure:U(`symbol-structure`,60049),symbolStruct:U(`symbol-struct`,60049),symbolParameter:U(`symbol-parameter`,60050),symbolTypeParameter:U(`symbol-type-parameter`,60050),symbolKey:U(`symbol-key`,60051),symbolText:U(`symbol-text`,60051),symbolReference:U(`symbol-reference`,60052),goToFile:U(`go-to-file`,60052),symbolEnum:U(`symbol-enum`,60053),symbolValue:U(`symbol-value`,60053),symbolRuler:U(`symbol-ruler`,60054),symbolUnit:U(`symbol-unit`,60054),activateBreakpoints:U(`activate-breakpoints`,60055),archive:U(`archive`,60056),arrowBoth:U(`arrow-both`,60057),arrowDown:U(`arrow-down`,60058),arrowLeft:U(`arrow-left`,60059),arrowRight:U(`arrow-right`,60060),arrowSmallDown:U(`arrow-small-down`,60061),arrowSmallLeft:U(`arrow-small-left`,60062),arrowSmallRight:U(`arrow-small-right`,60063),arrowSmallUp:U(`arrow-small-up`,60064),arrowUp:U(`arrow-up`,60065),bell:U(`bell`,60066),bold:U(`bold`,60067),book:U(`book`,60068),bookmark:U(`bookmark`,60069),debugBreakpointConditionalUnverified:U(`debug-breakpoint-conditional-unverified`,60070),debugBreakpointConditional:U(`debug-breakpoint-conditional`,60071),debugBreakpointConditionalDisabled:U(`debug-breakpoint-conditional-disabled`,60071),debugBreakpointDataUnverified:U(`debug-breakpoint-data-unverified`,60072),debugBreakpointData:U(`debug-breakpoint-data`,60073),debugBreakpointDataDisabled:U(`debug-breakpoint-data-disabled`,60073),debugBreakpointLogUnverified:U(`debug-breakpoint-log-unverified`,60074),debugBreakpointLog:U(`debug-breakpoint-log`,60075),debugBreakpointLogDisabled:U(`debug-breakpoint-log-disabled`,60075),briefcase:U(`briefcase`,60076),broadcast:U(`broadcast`,60077),browser:U(`browser`,60078),bug:U(`bug`,60079),calendar:U(`calendar`,60080),caseSensitive:U(`case-sensitive`,60081),check:U(`check`,60082),checklist:U(`checklist`,60083),chevronDown:U(`chevron-down`,60084),chevronLeft:U(`chevron-left`,60085),chevronRight:U(`chevron-right`,60086),chevronUp:U(`chevron-up`,60087),chromeClose:U(`chrome-close`,60088),chromeMaximize:U(`chrome-maximize`,60089),chromeMinimize:U(`chrome-minimize`,60090),chromeRestore:U(`chrome-restore`,60091),circleOutline:U(`circle-outline`,60092),circle:U(`circle`,60092),debugBreakpointUnverified:U(`debug-breakpoint-unverified`,60092),terminalDecorationIncomplete:U(`terminal-decoration-incomplete`,60092),circleSlash:U(`circle-slash`,60093),circuitBoard:U(`circuit-board`,60094),clearAll:U(`clear-all`,60095),clippy:U(`clippy`,60096),closeAll:U(`close-all`,60097),cloudDownload:U(`cloud-download`,60098),cloudUpload:U(`cloud-upload`,60099),code:U(`code`,60100),collapseAll:U(`collapse-all`,60101),colorMode:U(`color-mode`,60102),commentDiscussion:U(`comment-discussion`,60103),creditCard:U(`credit-card`,60105),dash:U(`dash`,60108),dashboard:U(`dashboard`,60109),database:U(`database`,60110),debugContinue:U(`debug-continue`,60111),debugDisconnect:U(`debug-disconnect`,60112),debugPause:U(`debug-pause`,60113),debugRestart:U(`debug-restart`,60114),debugStart:U(`debug-start`,60115),debugStepInto:U(`debug-step-into`,60116),debugStepOut:U(`debug-step-out`,60117),debugStepOver:U(`debug-step-over`,60118),debugStop:U(`debug-stop`,60119),debug:U(`debug`,60120),deviceCameraVideo:U(`device-camera-video`,60121),deviceCamera:U(`device-camera`,60122),deviceMobile:U(`device-mobile`,60123),diffAdded:U(`diff-added`,60124),diffIgnored:U(`diff-ignored`,60125),diffModified:U(`diff-modified`,60126),diffRemoved:U(`diff-removed`,60127),diffRenamed:U(`diff-renamed`,60128),diff:U(`diff`,60129),diffSidebyside:U(`diff-sidebyside`,60129),discard:U(`discard`,60130),editorLayout:U(`editor-layout`,60131),emptyWindow:U(`empty-window`,60132),exclude:U(`exclude`,60133),extensions:U(`extensions`,60134),eyeClosed:U(`eye-closed`,60135),fileBinary:U(`file-binary`,60136),fileCode:U(`file-code`,60137),fileMedia:U(`file-media`,60138),filePdf:U(`file-pdf`,60139),fileSubmodule:U(`file-submodule`,60140),fileSymlinkDirectory:U(`file-symlink-directory`,60141),fileSymlinkFile:U(`file-symlink-file`,60142),fileZip:U(`file-zip`,60143),files:U(`files`,60144),filter:U(`filter`,60145),flame:U(`flame`,60146),foldDown:U(`fold-down`,60147),foldUp:U(`fold-up`,60148),fold:U(`fold`,60149),folderActive:U(`folder-active`,60150),folderOpened:U(`folder-opened`,60151),gear:U(`gear`,60152),gift:U(`gift`,60153),gistSecret:U(`gist-secret`,60154),gist:U(`gist`,60155),gitCommit:U(`git-commit`,60156),gitCompare:U(`git-compare`,60157),compareChanges:U(`compare-changes`,60157),gitMerge:U(`git-merge`,60158),githubAction:U(`github-action`,60159),githubAlt:U(`github-alt`,60160),globe:U(`globe`,60161),grabber:U(`grabber`,60162),graph:U(`graph`,60163),gripper:U(`gripper`,60164),heart:U(`heart`,60165),home:U(`home`,60166),horizontalRule:U(`horizontal-rule`,60167),hubot:U(`hubot`,60168),inbox:U(`inbox`,60169),issueReopened:U(`issue-reopened`,60171),issues:U(`issues`,60172),italic:U(`italic`,60173),jersey:U(`jersey`,60174),json:U(`json`,60175),kebabVertical:U(`kebab-vertical`,60176),key:U(`key`,60177),law:U(`law`,60178),lightbulbAutofix:U(`lightbulb-autofix`,60179),linkExternal:U(`link-external`,60180),link:U(`link`,60181),listOrdered:U(`list-ordered`,60182),listUnordered:U(`list-unordered`,60183),liveShare:U(`live-share`,60184),loading:U(`loading`,60185),location:U(`location`,60186),mailRead:U(`mail-read`,60187),mail:U(`mail`,60188),markdown:U(`markdown`,60189),megaphone:U(`megaphone`,60190),mention:U(`mention`,60191),milestone:U(`milestone`,60192),gitPullRequestMilestone:U(`git-pull-request-milestone`,60192),mortarBoard:U(`mortar-board`,60193),move:U(`move`,60194),multipleWindows:U(`multiple-windows`,60195),mute:U(`mute`,60196),noNewline:U(`no-newline`,60197),note:U(`note`,60198),octoface:U(`octoface`,60199),openPreview:U(`open-preview`,60200),package:U(`package`,60201),paintcan:U(`paintcan`,60202),pin:U(`pin`,60203),play:U(`play`,60204),run:U(`run`,60204),plug:U(`plug`,60205),preserveCase:U(`preserve-case`,60206),preview:U(`preview`,60207),project:U(`project`,60208),pulse:U(`pulse`,60209),question:U(`question`,60210),quote:U(`quote`,60211),radioTower:U(`radio-tower`,60212),reactions:U(`reactions`,60213),references:U(`references`,60214),refresh:U(`refresh`,60215),regex:U(`regex`,60216),remoteExplorer:U(`remote-explorer`,60217),remote:U(`remote`,60218),remove:U(`remove`,60219),replaceAll:U(`replace-all`,60220),replace:U(`replace`,60221),repoClone:U(`repo-clone`,60222),repoForcePush:U(`repo-force-push`,60223),repoPull:U(`repo-pull`,60224),repoPush:U(`repo-push`,60225),report:U(`report`,60226),requestChanges:U(`request-changes`,60227),rocket:U(`rocket`,60228),rootFolderOpened:U(`root-folder-opened`,60229),rootFolder:U(`root-folder`,60230),rss:U(`rss`,60231),ruby:U(`ruby`,60232),saveAll:U(`save-all`,60233),saveAs:U(`save-as`,60234),save:U(`save`,60235),screenFull:U(`screen-full`,60236),screenNormal:U(`screen-normal`,60237),searchStop:U(`search-stop`,60238),server:U(`server`,60240),settingsGear:U(`settings-gear`,60241),settings:U(`settings`,60242),shield:U(`shield`,60243),smiley:U(`smiley`,60244),sortPrecedence:U(`sort-precedence`,60245),splitHorizontal:U(`split-horizontal`,60246),splitVertical:U(`split-vertical`,60247),squirrel:U(`squirrel`,60248),starFull:U(`star-full`,60249),starHalf:U(`star-half`,60250),symbolClass:U(`symbol-class`,60251),symbolColor:U(`symbol-color`,60252),symbolConstant:U(`symbol-constant`,60253),symbolEnumMember:U(`symbol-enum-member`,60254),symbolField:U(`symbol-field`,60255),symbolFile:U(`symbol-file`,60256),symbolInterface:U(`symbol-interface`,60257),symbolKeyword:U(`symbol-keyword`,60258),symbolMisc:U(`symbol-misc`,60259),symbolOperator:U(`symbol-operator`,60260),symbolProperty:U(`symbol-property`,60261),wrench:U(`wrench`,60261),wrenchSubaction:U(`wrench-subaction`,60261),symbolSnippet:U(`symbol-snippet`,60262),tasklist:U(`tasklist`,60263),telescope:U(`telescope`,60264),textSize:U(`text-size`,60265),threeBars:U(`three-bars`,60266),thumbsdown:U(`thumbsdown`,60267),thumbsup:U(`thumbsup`,60268),tools:U(`tools`,60269),triangleDown:U(`triangle-down`,60270),triangleLeft:U(`triangle-left`,60271),triangleRight:U(`triangle-right`,60272),triangleUp:U(`triangle-up`,60273),twitter:U(`twitter`,60274),unfold:U(`unfold`,60275),unlock:U(`unlock`,60276),unmute:U(`unmute`,60277),unverified:U(`unverified`,60278),verified:U(`verified`,60279),versions:U(`versions`,60280),vmActive:U(`vm-active`,60281),vmOutline:U(`vm-outline`,60282),vmRunning:U(`vm-running`,60283),watch:U(`watch`,60284),whitespace:U(`whitespace`,60285),wholeWord:U(`whole-word`,60286),window:U(`window`,60287),wordWrap:U(`word-wrap`,60288),zoomIn:U(`zoom-in`,60289),zoomOut:U(`zoom-out`,60290),listFilter:U(`list-filter`,60291),listFlat:U(`list-flat`,60292),listSelection:U(`list-selection`,60293),selection:U(`selection`,60293),listTree:U(`list-tree`,60294),debugBreakpointFunctionUnverified:U(`debug-breakpoint-function-unverified`,60295),debugBreakpointFunction:U(`debug-breakpoint-function`,60296),debugBreakpointFunctionDisabled:U(`debug-breakpoint-function-disabled`,60296),debugStackframeActive:U(`debug-stackframe-active`,60297),circleSmallFilled:U(`circle-small-filled`,60298),debugStackframeDot:U(`debug-stackframe-dot`,60298),terminalDecorationMark:U(`terminal-decoration-mark`,60298),debugStackframe:U(`debug-stackframe`,60299),debugStackframeFocused:U(`debug-stackframe-focused`,60299),debugBreakpointUnsupported:U(`debug-breakpoint-unsupported`,60300),symbolString:U(`symbol-string`,60301),debugReverseContinue:U(`debug-reverse-continue`,60302),debugStepBack:U(`debug-step-back`,60303),debugRestartFrame:U(`debug-restart-frame`,60304),debugAlt:U(`debug-alt`,60305),callIncoming:U(`call-incoming`,60306),callOutgoing:U(`call-outgoing`,60307),menu:U(`menu`,60308),expandAll:U(`expand-all`,60309),feedback:U(`feedback`,60310),gitPullRequestReviewer:U(`git-pull-request-reviewer`,60310),groupByRefType:U(`group-by-ref-type`,60311),ungroupByRefType:U(`ungroup-by-ref-type`,60312),account:U(`account`,60313),gitPullRequestAssignee:U(`git-pull-request-assignee`,60313),bellDot:U(`bell-dot`,60314),debugConsole:U(`debug-console`,60315),library:U(`library`,60316),output:U(`output`,60317),runAll:U(`run-all`,60318),syncIgnored:U(`sync-ignored`,60319),pinned:U(`pinned`,60320),githubInverted:U(`github-inverted`,60321),serverProcess:U(`server-process`,60322),serverEnvironment:U(`server-environment`,60323),pass:U(`pass`,60324),issueClosed:U(`issue-closed`,60324),stopCircle:U(`stop-circle`,60325),playCircle:U(`play-circle`,60326),record:U(`record`,60327),debugAltSmall:U(`debug-alt-small`,60328),vmConnect:U(`vm-connect`,60329),cloud:U(`cloud`,60330),merge:U(`merge`,60331),export:U(`export`,60332),graphLeft:U(`graph-left`,60333),magnet:U(`magnet`,60334),notebook:U(`notebook`,60335),redo:U(`redo`,60336),checkAll:U(`check-all`,60337),pinnedDirty:U(`pinned-dirty`,60338),passFilled:U(`pass-filled`,60339),circleLargeFilled:U(`circle-large-filled`,60340),circleLarge:U(`circle-large`,60341),circleLargeOutline:U(`circle-large-outline`,60341),combine:U(`combine`,60342),gather:U(`gather`,60342),table:U(`table`,60343),variableGroup:U(`variable-group`,60344),typeHierarchy:U(`type-hierarchy`,60345),typeHierarchySub:U(`type-hierarchy-sub`,60346),typeHierarchySuper:U(`type-hierarchy-super`,60347),gitPullRequestCreate:U(`git-pull-request-create`,60348),runAbove:U(`run-above`,60349),runBelow:U(`run-below`,60350),notebookTemplate:U(`notebook-template`,60351),debugRerun:U(`debug-rerun`,60352),workspaceTrusted:U(`workspace-trusted`,60353),workspaceUntrusted:U(`workspace-untrusted`,60354),workspaceUnknown:U(`workspace-unknown`,60355),terminalCmd:U(`terminal-cmd`,60356),terminalDebian:U(`terminal-debian`,60357),terminalLinux:U(`terminal-linux`,60358),terminalPowershell:U(`terminal-powershell`,60359),terminalTmux:U(`terminal-tmux`,60360),terminalUbuntu:U(`terminal-ubuntu`,60361),terminalBash:U(`terminal-bash`,60362),arrowSwap:U(`arrow-swap`,60363),copy:U(`copy`,60364),personAdd:U(`person-add`,60365),filterFilled:U(`filter-filled`,60366),wand:U(`wand`,60367),debugLineByLine:U(`debug-line-by-line`,60368),inspect:U(`inspect`,60369),layers:U(`layers`,60370),layersDot:U(`layers-dot`,60371),layersActive:U(`layers-active`,60372),compass:U(`compass`,60373),compassDot:U(`compass-dot`,60374),compassActive:U(`compass-active`,60375),azure:U(`azure`,60376),issueDraft:U(`issue-draft`,60377),gitPullRequestClosed:U(`git-pull-request-closed`,60378),gitPullRequestDraft:U(`git-pull-request-draft`,60379),debugAll:U(`debug-all`,60380),debugCoverage:U(`debug-coverage`,60381),runErrors:U(`run-errors`,60382),folderLibrary:U(`folder-library`,60383),debugContinueSmall:U(`debug-continue-small`,60384),beakerStop:U(`beaker-stop`,60385),graphLine:U(`graph-line`,60386),graphScatter:U(`graph-scatter`,60387),pieChart:U(`pie-chart`,60388),bracket:U(`bracket`,60175),bracketDot:U(`bracket-dot`,60389),bracketError:U(`bracket-error`,60390),lockSmall:U(`lock-small`,60391),azureDevops:U(`azure-devops`,60392),verifiedFilled:U(`verified-filled`,60393),newline:U(`newline`,60394),layout:U(`layout`,60395),layoutActivitybarLeft:U(`layout-activitybar-left`,60396),layoutActivitybarRight:U(`layout-activitybar-right`,60397),layoutPanelLeft:U(`layout-panel-left`,60398),layoutPanelCenter:U(`layout-panel-center`,60399),layoutPanelJustify:U(`layout-panel-justify`,60400),layoutPanelRight:U(`layout-panel-right`,60401),layoutPanel:U(`layout-panel`,60402),layoutSidebarLeft:U(`layout-sidebar-left`,60403),layoutSidebarRight:U(`layout-sidebar-right`,60404),layoutStatusbar:U(`layout-statusbar`,60405),layoutMenubar:U(`layout-menubar`,60406),layoutCentered:U(`layout-centered`,60407),target:U(`target`,60408),indent:U(`indent`,60409),recordSmall:U(`record-small`,60410),errorSmall:U(`error-small`,60411),terminalDecorationError:U(`terminal-decoration-error`,60411),arrowCircleDown:U(`arrow-circle-down`,60412),arrowCircleLeft:U(`arrow-circle-left`,60413),arrowCircleRight:U(`arrow-circle-right`,60414),arrowCircleUp:U(`arrow-circle-up`,60415),layoutSidebarRightOff:U(`layout-sidebar-right-off`,60416),layoutPanelOff:U(`layout-panel-off`,60417),layoutSidebarLeftOff:U(`layout-sidebar-left-off`,60418),blank:U(`blank`,60419),heartFilled:U(`heart-filled`,60420),map:U(`map`,60421),mapHorizontal:U(`map-horizontal`,60421),foldHorizontal:U(`fold-horizontal`,60421),mapFilled:U(`map-filled`,60422),mapHorizontalFilled:U(`map-horizontal-filled`,60422),foldHorizontalFilled:U(`fold-horizontal-filled`,60422),circleSmall:U(`circle-small`,60423),bellSlash:U(`bell-slash`,60424),bellSlashDot:U(`bell-slash-dot`,60425),commentUnresolved:U(`comment-unresolved`,60426),gitPullRequestGoToChanges:U(`git-pull-request-go-to-changes`,60427),gitPullRequestNewChanges:U(`git-pull-request-new-changes`,60428),searchFuzzy:U(`search-fuzzy`,60429),commentDraft:U(`comment-draft`,60430),send:U(`send`,60431),sparkle:U(`sparkle`,60432),insert:U(`insert`,60433),mic:U(`mic`,60434),thumbsdownFilled:U(`thumbsdown-filled`,60435),thumbsupFilled:U(`thumbsup-filled`,60436),coffee:U(`coffee`,60437),snake:U(`snake`,60438),game:U(`game`,60439),vr:U(`vr`,60440),chip:U(`chip`,60441),piano:U(`piano`,60442),music:U(`music`,60443),micFilled:U(`mic-filled`,60444),repoFetch:U(`repo-fetch`,60445),copilot:U(`copilot`,60446),lightbulbSparkle:U(`lightbulb-sparkle`,60447),robot:U(`robot`,60448),sparkleFilled:U(`sparkle-filled`,60449),diffSingle:U(`diff-single`,60450),diffMultiple:U(`diff-multiple`,60451),surroundWith:U(`surround-with`,60452),share:U(`share`,60453),gitStash:U(`git-stash`,60454),gitStashApply:U(`git-stash-apply`,60455),gitStashPop:U(`git-stash-pop`,60456),vscode:U(`vscode`,60457),vscodeInsiders:U(`vscode-insiders`,60458),codeOss:U(`code-oss`,60459),runCoverage:U(`run-coverage`,60460),runAllCoverage:U(`run-all-coverage`,60461),coverage:U(`coverage`,60462),githubProject:U(`github-project`,60463),mapVertical:U(`map-vertical`,60464),foldVertical:U(`fold-vertical`,60464),mapVerticalFilled:U(`map-vertical-filled`,60465),foldVerticalFilled:U(`fold-vertical-filled`,60465),goToSearch:U(`go-to-search`,60466),percentage:U(`percentage`,60467),sortPercentage:U(`sort-percentage`,60467),attach:U(`attach`,60468),goToEditingSession:U(`go-to-editing-session`,60469),editSession:U(`edit-session`,60470),codeReview:U(`code-review`,60471),copilotWarning:U(`copilot-warning`,60472),python:U(`python`,60473),copilotLarge:U(`copilot-large`,60474),copilotWarningLarge:U(`copilot-warning-large`,60475),keyboardTab:U(`keyboard-tab`,60476),copilotBlocked:U(`copilot-blocked`,60477),copilotNotConnected:U(`copilot-not-connected`,60478),flag:U(`flag`,60479),lightbulbEmpty:U(`lightbulb-empty`,60480),symbolMethodArrow:U(`symbol-method-arrow`,60481),copilotUnavailable:U(`copilot-unavailable`,60482),repoPinned:U(`repo-pinned`,60483),keyboardTabAbove:U(`keyboard-tab-above`,60484),keyboardTabBelow:U(`keyboard-tab-below`,60485),gitPullRequestDone:U(`git-pull-request-done`,60486),mcp:U(`mcp`,60487),extensionsLarge:U(`extensions-large`,60488),layoutPanelDock:U(`layout-panel-dock`,60489),layoutSidebarLeftDock:U(`layout-sidebar-left-dock`,60490),layoutSidebarRightDock:U(`layout-sidebar-right-dock`,60491),copilotInProgress:U(`copilot-in-progress`,60492),copilotError:U(`copilot-error`,60493),copilotSuccess:U(`copilot-success`,60494),chatSparkle:U(`chat-sparkle`,60495),searchSparkle:U(`search-sparkle`,60496),editSparkle:U(`edit-sparkle`,60497),copilotSnooze:U(`copilot-snooze`,60498),sendToRemoteAgent:U(`send-to-remote-agent`,60499),commentDiscussionSparkle:U(`comment-discussion-sparkle`,60500),chatSparkleWarning:U(`chat-sparkle-warning`,60501),chatSparkleError:U(`chat-sparkle-error`,60502),collection:U(`collection`,60503),newCollection:U(`new-collection`,60504),thinking:U(`thinking`,60505),build:U(`build`,60506),commentDiscussionQuote:U(`comment-discussion-quote`,60507),cursor:U(`cursor`,60508),eraser:U(`eraser`,60509),fileText:U(`file-text`,60510),gitLens:U(`git-lens`,60511),quotes:U(`quotes`,60512),rename:U(`rename`,60513),runWithDeps:U(`run-with-deps`,60514),debugConnected:U(`debug-connected`,60515),strikethrough:U(`strikethrough`,60516),openInProduct:U(`open-in-product`,60517),indexZero:U(`index-zero`,60518),agent:U(`agent`,60519),editCode:U(`edit-code`,60520),repoSelected:U(`repo-selected`,60521),skip:U(`skip`,60522),mergeInto:U(`merge-into`,60523),gitBranchChanges:U(`git-branch-changes`,60524),gitBranchStagedChanges:U(`git-branch-staged-changes`,60525),gitBranchConflicts:U(`git-branch-conflicts`,60526),gitBranch:U(`git-branch`,60527),gitBranchCreate:U(`git-branch-create`,60527),gitBranchDelete:U(`git-branch-delete`,60527),searchLarge:U(`search-large`,60528),terminalGitBash:U(`terminal-git-bash`,60529)},yn={dialogError:U(`dialog-error`,`error`),dialogWarning:U(`dialog-warning`,`warning`),dialogInfo:U(`dialog-info`,`info`),dialogClose:U(`dialog-close`,`close`),treeItemExpanded:U(`tree-item-expanded`,`chevron-down`),treeFilterOnTypeOn:U(`tree-filter-on-type-on`,`list-filter`),treeFilterOnTypeOff:U(`tree-filter-on-type-off`,`list-selection`),treeFilterClear:U(`tree-filter-clear`,`close`),treeItemLoading:U(`tree-item-loading`,`loading`),menuSelection:U(`menu-selection`,`check`),menuSubmenu:U(`menu-submenu`,`chevron-right`),menuBarMore:U(`menubar-more`,`more`),scrollbarButtonLeft:U(`scrollbar-button-left`,`triangle-left`),scrollbarButtonRight:U(`scrollbar-button-right`,`triangle-right`),scrollbarButtonUp:U(`scrollbar-button-up`,`triangle-up`),scrollbarButtonDown:U(`scrollbar-button-down`,`triangle-down`),toolBarMore:U(`toolbar-more`,`more`),quickInputBack:U(`quick-input-back`,`arrow-left`),dropDownButton:U(`drop-down-button`,60084),symbolCustomColor:U(`symbol-customcolor`,60252),exportIcon:U(`export`,60332),workspaceUnspecified:U(`workspace-unspecified`,60355),newLine:U(`newline`,60394),thumbsDownFilled:U(`thumbsdown-filled`,60435),thumbsUpFilled:U(`thumbsup-filled`,60436),gitFetch:U(`git-fetch`,60445),lightbulbSparkleAutofix:U(`lightbulb-sparkle-autofix`,60447),debugBreakpointPending:U(`debug-breakpoint-pending`,60377)},W={...vn,...yn};var bn=class{constructor(){this._tokenizationSupports=/* @__PURE__ */ new Map,this._factories=/* @__PURE__ */ new Map,this._onDidChange=new E,this.onDidChange=this._onDidChange.event,this._colorMap=null}handleChange(e){this._onDidChange.fire({changedLanguages:e,changedColorMap:!1})}register(e,t){return this._tokenizationSupports.set(e,t),this.handleChange([e]),_(()=>{this._tokenizationSupports.get(e)===t&&(this._tokenizationSupports.delete(e),this.handleChange([e]))})}get(e){return this._tokenizationSupports.get(e)||null}registerFactory(e,t){this._factories.get(e)?.dispose();let n=new xn(this,e,t);return this._factories.set(e,n),_(()=>{let t=this._factories.get(e);t&&t===n&&(this._factories.delete(e),t.dispose())})}async getOrCreate(e){let t=this.get(e);if(t)return t;let n=this._factories.get(e);return!n||n.isResolved?null:(await n.resolve(),this.get(e))}isResolved(e){if(this.get(e))return!0;let t=this._factories.get(e);return!(t&&!t.isResolved)}setColorMap(e){this._colorMap=e,this._onDidChange.fire({changedLanguages:Array.from(this._tokenizationSupports.keys()),changedColorMap:!0})}getColorMap(){return this._colorMap}getDefaultBackground(){return this._colorMap&&this._colorMap.length>2?this._colorMap[2]:null}},xn=class extends y{get isResolved(){return this._isResolved}constructor(e,t,n){super(),this._registry=e,this._languageId=t,this._factory=n,this._isDisposed=!1,this._resolvePromise=null,this._isResolved=!1}dispose(){this._isDisposed=!0,super.dispose()}async resolve(){return this._resolvePromise||=this._create(),this._resolvePromise}async _create(){let e=await this._factory.tokenizationSupport;this._isResolved=!0,e&&!this._isDisposed&&this._register(this._registry.register(this._languageId,e))}},Sn=class{constructor(e,t,n){this.offset=e,this.type=t,this.language=n,this._tokenBrand=void 0}toString(){return`(`+this.offset+`, `+this.type+`)`}},Cn;(function(e){e[e.Increase=0]=`Increase`,e[e.Decrease=1]=`Decrease`})(Cn||={});var wn;(function(e){let t=/* @__PURE__ */ new Map;t.set(0,W.symbolMethod),t.set(1,W.symbolFunction),t.set(2,W.symbolConstructor),t.set(3,W.symbolField),t.set(4,W.symbolVariable),t.set(5,W.symbolClass),t.set(6,W.symbolStruct),t.set(7,W.symbolInterface),t.set(8,W.symbolModule),t.set(9,W.symbolProperty),t.set(10,W.symbolEvent),t.set(11,W.symbolOperator),t.set(12,W.symbolUnit),t.set(13,W.symbolValue),t.set(15,W.symbolEnum),t.set(14,W.symbolConstant),t.set(15,W.symbolEnum),t.set(16,W.symbolEnumMember),t.set(17,W.symbolKeyword),t.set(28,W.symbolSnippet),t.set(18,W.symbolText),t.set(19,W.symbolColor),t.set(20,W.symbolFile),t.set(21,W.symbolReference),t.set(22,W.symbolCustomColor),t.set(23,W.symbolFolder),t.set(24,W.symbolTypeParameter),t.set(25,W.account),t.set(26,W.issues),t.set(27,W.tools);function n(e){let n=t.get(e);return n||=(console.info(`No codicon found for CompletionItemKind `+e),W.symbolProperty),n}e.toIcon=n;function r(e){switch(e){case 0:return D(728,`Method`);case 1:return D(729,`Function`);case 2:return D(730,`Constructor`);case 3:return D(731,`Field`);case 4:return D(732,`Variable`);case 5:return D(733,`Class`);case 6:return D(734,`Struct`);case 7:return D(735,`Interface`);case 8:return D(736,`Module`);case 9:return D(737,`Property`);case 10:return D(738,`Event`);case 11:return D(739,`Operator`);case 12:return D(740,`Unit`);case 13:return D(741,`Value`);case 14:return D(742,`Constant`);case 15:return D(743,`Enum`);case 16:return D(744,`Enum Member`);case 17:return D(745,`Keyword`);case 18:return D(746,`Text`);case 19:return D(747,`Color`);case 20:return D(748,`File`);case 21:return D(749,`Reference`);case 22:return D(750,`Custom Color`);case 23:return D(751,`Folder`);case 24:return D(752,`Type Parameter`);case 25:return D(753,`User`);case 26:return D(754,`Issue`);case 27:return D(755,`Tool`);case 28:return D(756,`Snippet`);default:return``}}e.toLabel=r;let i=/* @__PURE__ */ new Map;i.set(`method`,0),i.set(`function`,1),i.set(`constructor`,2),i.set(`field`,3),i.set(`variable`,4),i.set(`class`,5),i.set(`struct`,6),i.set(`interface`,7),i.set(`module`,8),i.set(`property`,9),i.set(`event`,10),i.set(`operator`,11),i.set(`unit`,12),i.set(`value`,13),i.set(`constant`,14),i.set(`enum`,15),i.set(`enum-member`,16),i.set(`enumMember`,16),i.set(`keyword`,17),i.set(`snippet`,28),i.set(`text`,18),i.set(`color`,19),i.set(`file`,20),i.set(`reference`,21),i.set(`customcolor`,22),i.set(`folder`,23),i.set(`type-parameter`,24),i.set(`typeParameter`,24),i.set(`account`,25),i.set(`issue`,26),i.set(`tool`,27);function a(e,t){let n=i.get(e);return n===void 0&&!t&&(n=9),n}e.fromString=a})(wn||={});var Tn;(function(e){e[e.Automatic=0]=`Automatic`,e[e.Explicit=1]=`Explicit`})(Tn||={});var En;(function(e){e[e.Code=1]=`Code`,e[e.Label=2]=`Label`})(En||={});var Dn;(function(e){e[e.Accepted=0]=`Accepted`,e[e.Rejected=1]=`Rejected`,e[e.Ignored=2]=`Ignored`})(Dn||={});var On;(function(e){e[e.Automatic=0]=`Automatic`,e[e.PasteAs=1]=`PasteAs`})(On||={});var kn;(function(e){e[e.Invoke=1]=`Invoke`,e[e.TriggerCharacter=2]=`TriggerCharacter`,e[e.ContentChange=3]=`ContentChange`})(kn||={});var An;(function(e){e[e.Text=0]=`Text`,e[e.Read=1]=`Read`,e[e.Write=2]=`Write`})(An||={}),D(757,`array`),D(758,`boolean`),D(759,`class`),D(760,`constant`),D(761,`constructor`),D(762,`enumeration`),D(763,`enumeration member`),D(764,`event`),D(765,`field`),D(766,`file`),D(767,`function`),D(768,`interface`),D(769,`key`),D(770,`method`),D(771,`module`),D(772,`namespace`),D(773,`null`),D(774,`number`),D(775,`object`),D(776,`operator`),D(777,`package`),D(778,`property`),D(779,`string`),D(780,`struct`),D(781,`type parameter`),D(782,`variable`);var jn;(function(e){let t=/* @__PURE__ */ new Map;t.set(0,W.symbolFile),t.set(1,W.symbolModule),t.set(2,W.symbolNamespace),t.set(3,W.symbolPackage),t.set(4,W.symbolClass),t.set(5,W.symbolMethod),t.set(6,W.symbolProperty),t.set(7,W.symbolField),t.set(8,W.symbolConstructor),t.set(9,W.symbolEnum),t.set(10,W.symbolInterface),t.set(11,W.symbolFunction),t.set(12,W.symbolVariable),t.set(13,W.symbolConstant),t.set(14,W.symbolString),t.set(15,W.symbolNumber),t.set(16,W.symbolBoolean),t.set(17,W.symbolArray),t.set(18,W.symbolObject),t.set(19,W.symbolKey),t.set(20,W.symbolNull),t.set(21,W.symbolEnumMember),t.set(22,W.symbolStruct),t.set(23,W.symbolEvent),t.set(24,W.symbolOperator),t.set(25,W.symbolTypeParameter);function n(e){let n=t.get(e);return n||=(console.info(`No codicon found for SymbolKind `+e),W.symbolProperty),n}e.toIcon=n;let r=/* @__PURE__ */ new Map;r.set(0,20),r.set(1,8),r.set(2,8),r.set(3,8),r.set(4,5),r.set(5,0),r.set(6,9),r.set(7,3),r.set(8,2),r.set(9,15),r.set(10,7),r.set(11,1),r.set(12,4),r.set(13,14),r.set(14,18),r.set(15,13),r.set(16,13),r.set(17,13),r.set(18,13),r.set(19,17),r.set(20,13),r.set(21,16),r.set(22,6),r.set(23,10),r.set(24,11),r.set(25,24);function i(e){let t=r.get(e);return t===void 0&&(console.info(`No completion kind found for SymbolKind `+e),t=20),t}e.toCompletionKind=i})(jn||={}),class e{static{this.Comment=new e(`comment`)}static{this.Imports=new e(`imports`)}static{this.Region=new e(`region`)}static fromValue(t){switch(t){case`comment`:return e.Comment;case`imports`:return e.Imports;case`region`:return e.Region}return new e(t)}constructor(e){this.value=e}};var Mn;(function(e){e[e.AIGenerated=1]=`AIGenerated`})(Mn||={});var Nn;(function(e){e[e.Invoke=0]=`Invoke`,e[e.Automatic=1]=`Automatic`})(Nn||={});var Pn;(function(e){function t(e){return!e||typeof e!=`object`?!1:typeof e.id==`string`&&typeof e.title==`string`}e.is=t})(Pn||={});var Fn;(function(e){e[e.Type=1]=`Type`,e[e.Parameter=2]=`Parameter`})(Fn||={}),new bn;var In;(function(e){e[e.Unknown=0]=`Unknown`,e[e.Disabled=1]=`Disabled`,e[e.Enabled=2]=`Enabled`})(In||={});var Ln;(function(e){e[e.Invoke=1]=`Invoke`,e[e.Auto=2]=`Auto`})(Ln||={});var Rn;(function(e){e[e.None=0]=`None`,e[e.KeepWhitespace=1]=`KeepWhitespace`,e[e.InsertAsSnippet=4]=`InsertAsSnippet`})(Rn||={});var zn;(function(e){e[e.Method=0]=`Method`,e[e.Function=1]=`Function`,e[e.Constructor=2]=`Constructor`,e[e.Field=3]=`Field`,e[e.Variable=4]=`Variable`,e[e.Class=5]=`Class`,e[e.Struct=6]=`Struct`,e[e.Interface=7]=`Interface`,e[e.Module=8]=`Module`,e[e.Property=9]=`Property`,e[e.Event=10]=`Event`,e[e.Operator=11]=`Operator`,e[e.Unit=12]=`Unit`,e[e.Value=13]=`Value`,e[e.Constant=14]=`Constant`,e[e.Enum=15]=`Enum`,e[e.EnumMember=16]=`EnumMember`,e[e.Keyword=17]=`Keyword`,e[e.Text=18]=`Text`,e[e.Color=19]=`Color`,e[e.File=20]=`File`,e[e.Reference=21]=`Reference`,e[e.Customcolor=22]=`Customcolor`,e[e.Folder=23]=`Folder`,e[e.TypeParameter=24]=`TypeParameter`,e[e.User=25]=`User`,e[e.Issue=26]=`Issue`,e[e.Tool=27]=`Tool`,e[e.Snippet=28]=`Snippet`})(zn||={});var Bn;(function(e){e[e.Deprecated=1]=`Deprecated`})(Bn||={});var Vn;(function(e){e[e.Invoke=0]=`Invoke`,e[e.TriggerCharacter=1]=`TriggerCharacter`,e[e.TriggerForIncompleteCompletions=2]=`TriggerForIncompleteCompletions`})(Vn||={});var Hn;(function(e){e[e.EXACT=0]=`EXACT`,e[e.ABOVE=1]=`ABOVE`,e[e.BELOW=2]=`BELOW`})(Hn||={});var Un;(function(e){e[e.NotSet=0]=`NotSet`,e[e.ContentFlush=1]=`ContentFlush`,e[e.RecoverFromMarkers=2]=`RecoverFromMarkers`,e[e.Explicit=3]=`Explicit`,e[e.Paste=4]=`Paste`,e[e.Undo=5]=`Undo`,e[e.Redo=6]=`Redo`})(Un||={});var Wn;(function(e){e[e.LF=1]=`LF`,e[e.CRLF=2]=`CRLF`})(Wn||={});var Gn;(function(e){e[e.Text=0]=`Text`,e[e.Read=1]=`Read`,e[e.Write=2]=`Write`})(Gn||={});var Kn;(function(e){e[e.None=0]=`None`,e[e.Keep=1]=`Keep`,e[e.Brackets=2]=`Brackets`,e[e.Advanced=3]=`Advanced`,e[e.Full=4]=`Full`})(Kn||={});var qn;(function(e){e[e.acceptSuggestionOnCommitCharacter=0]=`acceptSuggestionOnCommitCharacter`,e[e.acceptSuggestionOnEnter=1]=`acceptSuggestionOnEnter`,e[e.accessibilitySupport=2]=`accessibilitySupport`,e[e.accessibilityPageSize=3]=`accessibilityPageSize`,e[e.allowOverflow=4]=`allowOverflow`,e[e.allowVariableLineHeights=5]=`allowVariableLineHeights`,e[e.allowVariableFonts=6]=`allowVariableFonts`,e[e.allowVariableFontsInAccessibilityMode=7]=`allowVariableFontsInAccessibilityMode`,e[e.ariaLabel=8]=`ariaLabel`,e[e.ariaRequired=9]=`ariaRequired`,e[e.autoClosingBrackets=10]=`autoClosingBrackets`,e[e.autoClosingComments=11]=`autoClosingComments`,e[e.screenReaderAnnounceInlineSuggestion=12]=`screenReaderAnnounceInlineSuggestion`,e[e.autoClosingDelete=13]=`autoClosingDelete`,e[e.autoClosingOvertype=14]=`autoClosingOvertype`,e[e.autoClosingQuotes=15]=`autoClosingQuotes`,e[e.autoIndent=16]=`autoIndent`,e[e.autoIndentOnPaste=17]=`autoIndentOnPaste`,e[e.autoIndentOnPasteWithinString=18]=`autoIndentOnPasteWithinString`,e[e.automaticLayout=19]=`automaticLayout`,e[e.autoSurround=20]=`autoSurround`,e[e.bracketPairColorization=21]=`bracketPairColorization`,e[e.guides=22]=`guides`,e[e.codeLens=23]=`codeLens`,e[e.codeLensFontFamily=24]=`codeLensFontFamily`,e[e.codeLensFontSize=25]=`codeLensFontSize`,e[e.colorDecorators=26]=`colorDecorators`,e[e.colorDecoratorsLimit=27]=`colorDecoratorsLimit`,e[e.columnSelection=28]=`columnSelection`,e[e.comments=29]=`comments`,e[e.contextmenu=30]=`contextmenu`,e[e.copyWithSyntaxHighlighting=31]=`copyWithSyntaxHighlighting`,e[e.cursorBlinking=32]=`cursorBlinking`,e[e.cursorSmoothCaretAnimation=33]=`cursorSmoothCaretAnimation`,e[e.cursorStyle=34]=`cursorStyle`,e[e.cursorSurroundingLines=35]=`cursorSurroundingLines`,e[e.cursorSurroundingLinesStyle=36]=`cursorSurroundingLinesStyle`,e[e.cursorWidth=37]=`cursorWidth`,e[e.cursorHeight=38]=`cursorHeight`,e[e.disableLayerHinting=39]=`disableLayerHinting`,e[e.disableMonospaceOptimizations=40]=`disableMonospaceOptimizations`,e[e.domReadOnly=41]=`domReadOnly`,e[e.dragAndDrop=42]=`dragAndDrop`,e[e.dropIntoEditor=43]=`dropIntoEditor`,e[e.editContext=44]=`editContext`,e[e.emptySelectionClipboard=45]=`emptySelectionClipboard`,e[e.experimentalGpuAcceleration=46]=`experimentalGpuAcceleration`,e[e.experimentalWhitespaceRendering=47]=`experimentalWhitespaceRendering`,e[e.extraEditorClassName=48]=`extraEditorClassName`,e[e.fastScrollSensitivity=49]=`fastScrollSensitivity`,e[e.find=50]=`find`,e[e.fixedOverflowWidgets=51]=`fixedOverflowWidgets`,e[e.folding=52]=`folding`,e[e.foldingStrategy=53]=`foldingStrategy`,e[e.foldingHighlight=54]=`foldingHighlight`,e[e.foldingImportsByDefault=55]=`foldingImportsByDefault`,e[e.foldingMaximumRegions=56]=`foldingMaximumRegions`,e[e.unfoldOnClickAfterEndOfLine=57]=`unfoldOnClickAfterEndOfLine`,e[e.fontFamily=58]=`fontFamily`,e[e.fontInfo=59]=`fontInfo`,e[e.fontLigatures=60]=`fontLigatures`,e[e.fontSize=61]=`fontSize`,e[e.fontWeight=62]=`fontWeight`,e[e.fontVariations=63]=`fontVariations`,e[e.formatOnPaste=64]=`formatOnPaste`,e[e.formatOnType=65]=`formatOnType`,e[e.glyphMargin=66]=`glyphMargin`,e[e.gotoLocation=67]=`gotoLocation`,e[e.hideCursorInOverviewRuler=68]=`hideCursorInOverviewRuler`,e[e.hover=69]=`hover`,e[e.inDiffEditor=70]=`inDiffEditor`,e[e.inlineSuggest=71]=`inlineSuggest`,e[e.letterSpacing=72]=`letterSpacing`,e[e.lightbulb=73]=`lightbulb`,e[e.lineDecorationsWidth=74]=`lineDecorationsWidth`,e[e.lineHeight=75]=`lineHeight`,e[e.lineNumbers=76]=`lineNumbers`,e[e.lineNumbersMinChars=77]=`lineNumbersMinChars`,e[e.linkedEditing=78]=`linkedEditing`,e[e.links=79]=`links`,e[e.matchBrackets=80]=`matchBrackets`,e[e.minimap=81]=`minimap`,e[e.mouseStyle=82]=`mouseStyle`,e[e.mouseWheelScrollSensitivity=83]=`mouseWheelScrollSensitivity`,e[e.mouseWheelZoom=84]=`mouseWheelZoom`,e[e.multiCursorMergeOverlapping=85]=`multiCursorMergeOverlapping`,e[e.multiCursorModifier=86]=`multiCursorModifier`,e[e.mouseMiddleClickAction=87]=`mouseMiddleClickAction`,e[e.multiCursorPaste=88]=`multiCursorPaste`,e[e.multiCursorLimit=89]=`multiCursorLimit`,e[e.occurrencesHighlight=90]=`occurrencesHighlight`,e[e.occurrencesHighlightDelay=91]=`occurrencesHighlightDelay`,e[e.overtypeCursorStyle=92]=`overtypeCursorStyle`,e[e.overtypeOnPaste=93]=`overtypeOnPaste`,e[e.overviewRulerBorder=94]=`overviewRulerBorder`,e[e.overviewRulerLanes=95]=`overviewRulerLanes`,e[e.padding=96]=`padding`,e[e.pasteAs=97]=`pasteAs`,e[e.parameterHints=98]=`parameterHints`,e[e.peekWidgetDefaultFocus=99]=`peekWidgetDefaultFocus`,e[e.placeholder=100]=`placeholder`,e[e.definitionLinkOpensInPeek=101]=`definitionLinkOpensInPeek`,e[e.quickSuggestions=102]=`quickSuggestions`,e[e.quickSuggestionsDelay=103]=`quickSuggestionsDelay`,e[e.readOnly=104]=`readOnly`,e[e.readOnlyMessage=105]=`readOnlyMessage`,e[e.renameOnType=106]=`renameOnType`,e[e.renderRichScreenReaderContent=107]=`renderRichScreenReaderContent`,e[e.renderControlCharacters=108]=`renderControlCharacters`,e[e.renderFinalNewline=109]=`renderFinalNewline`,e[e.renderLineHighlight=110]=`renderLineHighlight`,e[e.renderLineHighlightOnlyWhenFocus=111]=`renderLineHighlightOnlyWhenFocus`,e[e.renderValidationDecorations=112]=`renderValidationDecorations`,e[e.renderWhitespace=113]=`renderWhitespace`,e[e.revealHorizontalRightPadding=114]=`revealHorizontalRightPadding`,e[e.roundedSelection=115]=`roundedSelection`,e[e.rulers=116]=`rulers`,e[e.scrollbar=117]=`scrollbar`,e[e.scrollBeyondLastColumn=118]=`scrollBeyondLastColumn`,e[e.scrollBeyondLastLine=119]=`scrollBeyondLastLine`,e[e.scrollPredominantAxis=120]=`scrollPredominantAxis`,e[e.selectionClipboard=121]=`selectionClipboard`,e[e.selectionHighlight=122]=`selectionHighlight`,e[e.selectionHighlightMaxLength=123]=`selectionHighlightMaxLength`,e[e.selectionHighlightMultiline=124]=`selectionHighlightMultiline`,e[e.selectOnLineNumbers=125]=`selectOnLineNumbers`,e[e.showFoldingControls=126]=`showFoldingControls`,e[e.showUnused=127]=`showUnused`,e[e.snippetSuggestions=128]=`snippetSuggestions`,e[e.smartSelect=129]=`smartSelect`,e[e.smoothScrolling=130]=`smoothScrolling`,e[e.stickyScroll=131]=`stickyScroll`,e[e.stickyTabStops=132]=`stickyTabStops`,e[e.stopRenderingLineAfter=133]=`stopRenderingLineAfter`,e[e.suggest=134]=`suggest`,e[e.suggestFontSize=135]=`suggestFontSize`,e[e.suggestLineHeight=136]=`suggestLineHeight`,e[e.suggestOnTriggerCharacters=137]=`suggestOnTriggerCharacters`,e[e.suggestSelection=138]=`suggestSelection`,e[e.tabCompletion=139]=`tabCompletion`,e[e.tabIndex=140]=`tabIndex`,e[e.trimWhitespaceOnDelete=141]=`trimWhitespaceOnDelete`,e[e.unicodeHighlighting=142]=`unicodeHighlighting`,e[e.unusualLineTerminators=143]=`unusualLineTerminators`,e[e.useShadowDOM=144]=`useShadowDOM`,e[e.useTabStops=145]=`useTabStops`,e[e.wordBreak=146]=`wordBreak`,e[e.wordSegmenterLocales=147]=`wordSegmenterLocales`,e[e.wordSeparators=148]=`wordSeparators`,e[e.wordWrap=149]=`wordWrap`,e[e.wordWrapBreakAfterCharacters=150]=`wordWrapBreakAfterCharacters`,e[e.wordWrapBreakBeforeCharacters=151]=`wordWrapBreakBeforeCharacters`,e[e.wordWrapColumn=152]=`wordWrapColumn`,e[e.wordWrapOverride1=153]=`wordWrapOverride1`,e[e.wordWrapOverride2=154]=`wordWrapOverride2`,e[e.wrappingIndent=155]=`wrappingIndent`,e[e.wrappingStrategy=156]=`wrappingStrategy`,e[e.showDeprecated=157]=`showDeprecated`,e[e.inertialScroll=158]=`inertialScroll`,e[e.inlayHints=159]=`inlayHints`,e[e.wrapOnEscapedLineFeeds=160]=`wrapOnEscapedLineFeeds`,e[e.effectiveCursorStyle=161]=`effectiveCursorStyle`,e[e.editorClassName=162]=`editorClassName`,e[e.pixelRatio=163]=`pixelRatio`,e[e.tabFocusMode=164]=`tabFocusMode`,e[e.layoutInfo=165]=`layoutInfo`,e[e.wrappingInfo=166]=`wrappingInfo`,e[e.defaultColorDecorators=167]=`defaultColorDecorators`,e[e.colorDecoratorsActivatedOn=168]=`colorDecoratorsActivatedOn`,e[e.inlineCompletionsAccessibilityVerbose=169]=`inlineCompletionsAccessibilityVerbose`,e[e.effectiveEditContext=170]=`effectiveEditContext`,e[e.scrollOnMiddleClick=171]=`scrollOnMiddleClick`,e[e.effectiveAllowVariableFonts=172]=`effectiveAllowVariableFonts`})(qn||={});var Jn;(function(e){e[e.TextDefined=0]=`TextDefined`,e[e.LF=1]=`LF`,e[e.CRLF=2]=`CRLF`})(Jn||={});var Yn;(function(e){e[e.LF=0]=`LF`,e[e.CRLF=1]=`CRLF`})(Yn||={});var Xn;(function(e){e[e.Left=1]=`Left`,e[e.Center=2]=`Center`,e[e.Right=3]=`Right`})(Xn||={});var Zn;(function(e){e[e.Increase=0]=`Increase`,e[e.Decrease=1]=`Decrease`})(Zn||={});var Qn;(function(e){e[e.None=0]=`None`,e[e.Indent=1]=`Indent`,e[e.IndentOutdent=2]=`IndentOutdent`,e[e.Outdent=3]=`Outdent`})(Qn||={});var $n;(function(e){e[e.Both=0]=`Both`,e[e.Right=1]=`Right`,e[e.Left=2]=`Left`,e[e.None=3]=`None`})($n||={});var er;(function(e){e[e.Type=1]=`Type`,e[e.Parameter=2]=`Parameter`})(er||={});var tr;(function(e){e[e.Accepted=0]=`Accepted`,e[e.Rejected=1]=`Rejected`,e[e.Ignored=2]=`Ignored`})(tr||={});var nr;(function(e){e[e.Code=1]=`Code`,e[e.Label=2]=`Label`})(nr||={});var rr;(function(e){e[e.Automatic=0]=`Automatic`,e[e.Explicit=1]=`Explicit`})(rr||={});var ir;(function(e){e[e.DependsOnKbLayout=-1]=`DependsOnKbLayout`,e[e.Unknown=0]=`Unknown`,e[e.Backspace=1]=`Backspace`,e[e.Tab=2]=`Tab`,e[e.Enter=3]=`Enter`,e[e.Shift=4]=`Shift`,e[e.Ctrl=5]=`Ctrl`,e[e.Alt=6]=`Alt`,e[e.PauseBreak=7]=`PauseBreak`,e[e.CapsLock=8]=`CapsLock`,e[e.Escape=9]=`Escape`,e[e.Space=10]=`Space`,e[e.PageUp=11]=`PageUp`,e[e.PageDown=12]=`PageDown`,e[e.End=13]=`End`,e[e.Home=14]=`Home`,e[e.LeftArrow=15]=`LeftArrow`,e[e.UpArrow=16]=`UpArrow`,e[e.RightArrow=17]=`RightArrow`,e[e.DownArrow=18]=`DownArrow`,e[e.Insert=19]=`Insert`,e[e.Delete=20]=`Delete`,e[e.Digit0=21]=`Digit0`,e[e.Digit1=22]=`Digit1`,e[e.Digit2=23]=`Digit2`,e[e.Digit3=24]=`Digit3`,e[e.Digit4=25]=`Digit4`,e[e.Digit5=26]=`Digit5`,e[e.Digit6=27]=`Digit6`,e[e.Digit7=28]=`Digit7`,e[e.Digit8=29]=`Digit8`,e[e.Digit9=30]=`Digit9`,e[e.KeyA=31]=`KeyA`,e[e.KeyB=32]=`KeyB`,e[e.KeyC=33]=`KeyC`,e[e.KeyD=34]=`KeyD`,e[e.KeyE=35]=`KeyE`,e[e.KeyF=36]=`KeyF`,e[e.KeyG=37]=`KeyG`,e[e.KeyH=38]=`KeyH`,e[e.KeyI=39]=`KeyI`,e[e.KeyJ=40]=`KeyJ`,e[e.KeyK=41]=`KeyK`,e[e.KeyL=42]=`KeyL`,e[e.KeyM=43]=`KeyM`,e[e.KeyN=44]=`KeyN`,e[e.KeyO=45]=`KeyO`,e[e.KeyP=46]=`KeyP`,e[e.KeyQ=47]=`KeyQ`,e[e.KeyR=48]=`KeyR`,e[e.KeyS=49]=`KeyS`,e[e.KeyT=50]=`KeyT`,e[e.KeyU=51]=`KeyU`,e[e.KeyV=52]=`KeyV`,e[e.KeyW=53]=`KeyW`,e[e.KeyX=54]=`KeyX`,e[e.KeyY=55]=`KeyY`,e[e.KeyZ=56]=`KeyZ`,e[e.Meta=57]=`Meta`,e[e.ContextMenu=58]=`ContextMenu`,e[e.F1=59]=`F1`,e[e.F2=60]=`F2`,e[e.F3=61]=`F3`,e[e.F4=62]=`F4`,e[e.F5=63]=`F5`,e[e.F6=64]=`F6`,e[e.F7=65]=`F7`,e[e.F8=66]=`F8`,e[e.F9=67]=`F9`,e[e.F10=68]=`F10`,e[e.F11=69]=`F11`,e[e.F12=70]=`F12`,e[e.F13=71]=`F13`,e[e.F14=72]=`F14`,e[e.F15=73]=`F15`,e[e.F16=74]=`F16`,e[e.F17=75]=`F17`,e[e.F18=76]=`F18`,e[e.F19=77]=`F19`,e[e.F20=78]=`F20`,e[e.F21=79]=`F21`,e[e.F22=80]=`F22`,e[e.F23=81]=`F23`,e[e.F24=82]=`F24`,e[e.NumLock=83]=`NumLock`,e[e.ScrollLock=84]=`ScrollLock`,e[e.Semicolon=85]=`Semicolon`,e[e.Equal=86]=`Equal`,e[e.Comma=87]=`Comma`,e[e.Minus=88]=`Minus`,e[e.Period=89]=`Period`,e[e.Slash=90]=`Slash`,e[e.Backquote=91]=`Backquote`,e[e.BracketLeft=92]=`BracketLeft`,e[e.Backslash=93]=`Backslash`,e[e.BracketRight=94]=`BracketRight`,e[e.Quote=95]=`Quote`,e[e.OEM_8=96]=`OEM_8`,e[e.IntlBackslash=97]=`IntlBackslash`,e[e.Numpad0=98]=`Numpad0`,e[e.Numpad1=99]=`Numpad1`,e[e.Numpad2=100]=`Numpad2`,e[e.Numpad3=101]=`Numpad3`,e[e.Numpad4=102]=`Numpad4`,e[e.Numpad5=103]=`Numpad5`,e[e.Numpad6=104]=`Numpad6`,e[e.Numpad7=105]=`Numpad7`,e[e.Numpad8=106]=`Numpad8`,e[e.Numpad9=107]=`Numpad9`,e[e.NumpadMultiply=108]=`NumpadMultiply`,e[e.NumpadAdd=109]=`NumpadAdd`,e[e.NUMPAD_SEPARATOR=110]=`NUMPAD_SEPARATOR`,e[e.NumpadSubtract=111]=`NumpadSubtract`,e[e.NumpadDecimal=112]=`NumpadDecimal`,e[e.NumpadDivide=113]=`NumpadDivide`,e[e.KEY_IN_COMPOSITION=114]=`KEY_IN_COMPOSITION`,e[e.ABNT_C1=115]=`ABNT_C1`,e[e.ABNT_C2=116]=`ABNT_C2`,e[e.AudioVolumeMute=117]=`AudioVolumeMute`,e[e.AudioVolumeUp=118]=`AudioVolumeUp`,e[e.AudioVolumeDown=119]=`AudioVolumeDown`,e[e.BrowserSearch=120]=`BrowserSearch`,e[e.BrowserHome=121]=`BrowserHome`,e[e.BrowserBack=122]=`BrowserBack`,e[e.BrowserForward=123]=`BrowserForward`,e[e.MediaTrackNext=124]=`MediaTrackNext`,e[e.MediaTrackPrevious=125]=`MediaTrackPrevious`,e[e.MediaStop=126]=`MediaStop`,e[e.MediaPlayPause=127]=`MediaPlayPause`,e[e.LaunchMediaPlayer=128]=`LaunchMediaPlayer`,e[e.LaunchMail=129]=`LaunchMail`,e[e.LaunchApp2=130]=`LaunchApp2`,e[e.Clear=131]=`Clear`,e[e.MAX_VALUE=132]=`MAX_VALUE`})(ir||={});var ar;(function(e){e[e.Hint=1]=`Hint`,e[e.Info=2]=`Info`,e[e.Warning=4]=`Warning`,e[e.Error=8]=`Error`})(ar||={});var or;(function(e){e[e.Unnecessary=1]=`Unnecessary`,e[e.Deprecated=2]=`Deprecated`})(or||={});var sr;(function(e){e[e.Inline=1]=`Inline`,e[e.Gutter=2]=`Gutter`})(sr||={});var cr;(function(e){e[e.Normal=1]=`Normal`,e[e.Underlined=2]=`Underlined`})(cr||={});var lr;(function(e){e[e.UNKNOWN=0]=`UNKNOWN`,e[e.TEXTAREA=1]=`TEXTAREA`,e[e.GUTTER_GLYPH_MARGIN=2]=`GUTTER_GLYPH_MARGIN`,e[e.GUTTER_LINE_NUMBERS=3]=`GUTTER_LINE_NUMBERS`,e[e.GUTTER_LINE_DECORATIONS=4]=`GUTTER_LINE_DECORATIONS`,e[e.GUTTER_VIEW_ZONE=5]=`GUTTER_VIEW_ZONE`,e[e.CONTENT_TEXT=6]=`CONTENT_TEXT`,e[e.CONTENT_EMPTY=7]=`CONTENT_EMPTY`,e[e.CONTENT_VIEW_ZONE=8]=`CONTENT_VIEW_ZONE`,e[e.CONTENT_WIDGET=9]=`CONTENT_WIDGET`,e[e.OVERVIEW_RULER=10]=`OVERVIEW_RULER`,e[e.SCROLLBAR=11]=`SCROLLBAR`,e[e.OVERLAY_WIDGET=12]=`OVERLAY_WIDGET`,e[e.OUTSIDE_EDITOR=13]=`OUTSIDE_EDITOR`})(lr||={});var ur;(function(e){e[e.AIGenerated=1]=`AIGenerated`})(ur||={});var dr;(function(e){e[e.Invoke=0]=`Invoke`,e[e.Automatic=1]=`Automatic`})(dr||={});var fr;(function(e){e[e.TOP_RIGHT_CORNER=0]=`TOP_RIGHT_CORNER`,e[e.BOTTOM_RIGHT_CORNER=1]=`BOTTOM_RIGHT_CORNER`,e[e.TOP_CENTER=2]=`TOP_CENTER`})(fr||={});var pr;(function(e){e[e.Left=1]=`Left`,e[e.Center=2]=`Center`,e[e.Right=4]=`Right`,e[e.Full=7]=`Full`})(pr||={});var mr;(function(e){e[e.Word=0]=`Word`,e[e.Line=1]=`Line`,e[e.Suggest=2]=`Suggest`})(mr||={});var hr;(function(e){e[e.Left=0]=`Left`,e[e.Right=1]=`Right`,e[e.None=2]=`None`,e[e.LeftOfInjectedText=3]=`LeftOfInjectedText`,e[e.RightOfInjectedText=4]=`RightOfInjectedText`})(hr||={});var gr;(function(e){e[e.Off=0]=`Off`,e[e.On=1]=`On`,e[e.Relative=2]=`Relative`,e[e.Interval=3]=`Interval`,e[e.Custom=4]=`Custom`})(gr||={});var _r;(function(e){e[e.None=0]=`None`,e[e.Text=1]=`Text`,e[e.Blocks=2]=`Blocks`})(_r||={});var vr;(function(e){e[e.Smooth=0]=`Smooth`,e[e.Immediate=1]=`Immediate`})(vr||={});var yr;(function(e){e[e.Auto=1]=`Auto`,e[e.Hidden=2]=`Hidden`,e[e.Visible=3]=`Visible`})(yr||={});var br;(function(e){e[e.LTR=0]=`LTR`,e[e.RTL=1]=`RTL`})(br||={});var xr;(function(e){e.Off=`off`,e.OnCode=`onCode`,e.On=`on`})(xr||={});var Sr;(function(e){e[e.Invoke=1]=`Invoke`,e[e.TriggerCharacter=2]=`TriggerCharacter`,e[e.ContentChange=3]=`ContentChange`})(Sr||={});var Cr;(function(e){e[e.File=0]=`File`,e[e.Module=1]=`Module`,e[e.Namespace=2]=`Namespace`,e[e.Package=3]=`Package`,e[e.Class=4]=`Class`,e[e.Method=5]=`Method`,e[e.Property=6]=`Property`,e[e.Field=7]=`Field`,e[e.Constructor=8]=`Constructor`,e[e.Enum=9]=`Enum`,e[e.Interface=10]=`Interface`,e[e.Function=11]=`Function`,e[e.Variable=12]=`Variable`,e[e.Constant=13]=`Constant`,e[e.String=14]=`String`,e[e.Number=15]=`Number`,e[e.Boolean=16]=`Boolean`,e[e.Array=17]=`Array`,e[e.Object=18]=`Object`,e[e.Key=19]=`Key`,e[e.Null=20]=`Null`,e[e.EnumMember=21]=`EnumMember`,e[e.Struct=22]=`Struct`,e[e.Event=23]=`Event`,e[e.Operator=24]=`Operator`,e[e.TypeParameter=25]=`TypeParameter`})(Cr||={});var wr;(function(e){e[e.Deprecated=1]=`Deprecated`})(wr||={});var Tr;(function(e){e[e.LTR=0]=`LTR`,e[e.RTL=1]=`RTL`})(Tr||={});var Er;(function(e){e[e.Hidden=0]=`Hidden`,e[e.Blink=1]=`Blink`,e[e.Smooth=2]=`Smooth`,e[e.Phase=3]=`Phase`,e[e.Expand=4]=`Expand`,e[e.Solid=5]=`Solid`})(Er||={});var Dr;(function(e){e[e.Line=1]=`Line`,e[e.Block=2]=`Block`,e[e.Underline=3]=`Underline`,e[e.LineThin=4]=`LineThin`,e[e.BlockOutline=5]=`BlockOutline`,e[e.UnderlineThin=6]=`UnderlineThin`})(Dr||={});var Or;(function(e){e[e.AlwaysGrowsWhenTypingAtEdges=0]=`AlwaysGrowsWhenTypingAtEdges`,e[e.NeverGrowsWhenTypingAtEdges=1]=`NeverGrowsWhenTypingAtEdges`,e[e.GrowsOnlyWhenTypingBefore=2]=`GrowsOnlyWhenTypingBefore`,e[e.GrowsOnlyWhenTypingAfter=3]=`GrowsOnlyWhenTypingAfter`})(Or||={});var kr;(function(e){e[e.None=0]=`None`,e[e.Same=1]=`Same`,e[e.Indent=2]=`Indent`,e[e.DeepIndent=3]=`DeepIndent`})(kr||={});var Ar=class{static{this.CtrlCmd=2048}static{this.Shift=1024}static{this.Alt=512}static{this.WinCtrl=256}static chord(e,t){return Rt(e,t)}};function jr(){return{editor:void 0,languages:void 0,CancellationTokenSource:Ot,Emitter:E,KeyCode:ir,KeyMod:Ar,Position:P,Range:F,Selection:gn,SelectionDirection:br,MarkerSeverity:ar,MarkerTag:or,Uri:on,Token:Sn}}var Mr,Nr,Pr=class{constructor(e,t){this.uri=e,this.value=t}};function Fr(e){return Array.isArray(e)}(class e{static{this.defaultToKey=e=>e.toString()}constructor(t,n){if(this[Mr]=`ResourceMap`,t instanceof e)this.map=new Map(t.map),this.toKey=n??e.defaultToKey;else if(Fr(t)){this.map=/* @__PURE__ */ new Map,this.toKey=n??e.defaultToKey;for(let[e,n]of t)this.set(e,n)}else this.map=/* @__PURE__ */ new Map,this.toKey=t??e.defaultToKey}set(e,t){return this.map.set(this.toKey(e),new Pr(e,t)),this}get(e){return this.map.get(this.toKey(e))?.value}has(e){return this.map.has(this.toKey(e))}get size(){return this.map.size}clear(){this.map.clear()}delete(e){return this.map.delete(this.toKey(e))}forEach(e,t){t!==void 0&&(e=e.bind(t));for(let[t,n]of this.map)e(n.value,n.uri,this)}*values(){for(let e of this.map.values())yield e.value}*keys(){for(let e of this.map.values())yield e.uri}*entries(){for(let e of this.map.values())yield[e.uri,e.value]}*[(Mr=Symbol.toStringTag,Symbol.iterator)](){for(let[,e]of this.map)yield[e.uri,e.value]}});var Ir=class{constructor(){this[Nr]=`LinkedMap`,this._map=/* @__PURE__ */ new Map,this._head=void 0,this._tail=void 0,this._size=0,this._state=0}clear(){this._map.clear(),this._head=void 0,this._tail=void 0,this._size=0,this._state++}isEmpty(){return!this._head&&!this._tail}get size(){return this._size}get first(){return this._head?.value}get last(){return this._tail?.value}has(e){return this._map.has(e)}get(e,t=0){let n=this._map.get(e);if(n)return t!==0&&this.touch(n,t),n.value}set(e,t,n=0){let r=this._map.get(e);if(r)r.value=t,n!==0&&this.touch(r,n);else{switch(r={key:e,value:t,next:void 0,previous:void 0},n){case 0:this.addItemLast(r);break;case 1:this.addItemFirst(r);break;case 2:this.addItemLast(r);break;default:this.addItemLast(r)}this._map.set(e,r),this._size++}return this}delete(e){return!!this.remove(e)}remove(e){let t=this._map.get(e);if(t)return this._map.delete(e),this.removeItem(t),this._size--,t.value}shift(){if(!this._head&&!this._tail)return;if(!this._head||!this._tail)throw Error(`Invalid list`);let e=this._head;return this._map.delete(e.key),this.removeItem(e),this._size--,e.value}forEach(e,t){let n=this._state,r=this._head;for(;r;){if(t?e.bind(t)(r.value,r.key,this):e(r.value,r.key,this),this._state!==n)throw Error(`LinkedMap got modified during iteration.`);r=r.next}}keys(){let e=this,t=this._state,n=this._head,r={[Symbol.iterator](){return r},next(){if(e._state!==t)throw Error(`LinkedMap got modified during iteration.`);if(n){let e={value:n.key,done:!1};return n=n.next,e}return{value:void 0,done:!0}}};return r}values(){let e=this,t=this._state,n=this._head,r={[Symbol.iterator](){return r},next(){if(e._state!==t)throw Error(`LinkedMap got modified during iteration.`);if(n){let e={value:n.value,done:!1};return n=n.next,e}return{value:void 0,done:!0}}};return r}entries(){let e=this,t=this._state,n=this._head,r={[Symbol.iterator](){return r},next(){if(e._state!==t)throw Error(`LinkedMap got modified during iteration.`);if(n){let e={value:[n.key,n.value],done:!1};return n=n.next,e}return{value:void 0,done:!0}}};return r}[(Nr=Symbol.toStringTag,Symbol.iterator)](){return this.entries()}trimOld(e){if(e>=this.size)return;if(e===0){this.clear();return}let t=this._head,n=this.size;for(;t&&n>e;)this._map.delete(t.key),t=t.next,n--;this._head=t,this._size=n,t&&(t.previous=void 0),this._state++}trimNew(e){if(e>=this.size)return;if(e===0){this.clear();return}let t=this._tail,n=this.size;for(;t&&n>e;)this._map.delete(t.key),t=t.previous,n--;this._tail=t,this._size=n,t&&(t.next=void 0),this._state++}addItemFirst(e){if(!this._head&&!this._tail)this._tail=e;else if(this._head)e.next=this._head,this._head.previous=e;else throw Error(`Invalid list`);this._head=e,this._state++}addItemLast(e){if(!this._head&&!this._tail)this._head=e;else if(this._tail)e.previous=this._tail,this._tail.next=e;else throw Error(`Invalid list`);this._tail=e,this._state++}removeItem(e){if(e===this._head&&e===this._tail)this._head=void 0,this._tail=void 0;else if(e===this._head){if(!e.next)throw Error(`Invalid list`);e.next.previous=void 0,this._head=e.next}else if(e===this._tail){if(!e.previous)throw Error(`Invalid list`);e.previous.next=void 0,this._tail=e.previous}else{let t=e.next,n=e.previous;if(!t||!n)throw Error(`Invalid list`);t.previous=n,n.next=t}e.next=void 0,e.previous=void 0,this._state++}touch(e,t){if(!this._head||!this._tail)throw Error(`Invalid list`);if(t===1||t===2){if(t===1){if(e===this._head)return;let t=e.next,n=e.previous;e===this._tail?(n.next=void 0,this._tail=n):(t.previous=n,n.next=t),e.previous=void 0,e.next=this._head,this._head.previous=e,this._head=e,this._state++}else if(t===2){if(e===this._tail)return;let t=e.next,n=e.previous;e===this._head?(t.previous=void 0,this._head=t):(t.previous=n,n.next=t),e.next=void 0,e.previous=this._tail,this._tail.next=e,this._tail=e,this._state++}}}toJSON(){let e=[];return this.forEach((t,n)=>{e.push([n,t])}),e}fromJSON(e){this.clear();for(let[t,n]of e)this.set(t,n)}},Lr=class extends Ir{constructor(e,t=1){super(),this._limit=e,this._ratio=Math.min(Math.max(0,t),1)}get limit(){return this._limit}set limit(e){this._limit=e,this.checkTrim()}get(e,t=2){return super.get(e,t)}peek(e){return super.get(e,0)}set(e,t){return super.set(e,t,2),this}checkTrim(){this.size>this._limit&&this.trim(Math.round(this._limit*this._ratio))}},Rr=class extends Lr{constructor(e,t=1){super(e,t)}trim(e){this.trimOld(e)}set(e,t){return super.set(e,t),this.checkTrim(),this}},zr=class{constructor(){this.map=/* @__PURE__ */ new Map}add(e,t){let n=this.map.get(e);n||(n=/* @__PURE__ */ new Set,this.map.set(e,n)),n.add(t)}delete(e,t){let n=this.map.get(e);n&&(n.delete(t),n.size===0&&this.map.delete(e))}forEach(e,t){let n=this.map.get(e);n&&n.forEach(t)}};new Rr(10);var Br;(function(e){e[e.Left=1]=`Left`,e[e.Center=2]=`Center`,e[e.Right=4]=`Right`,e[e.Full=7]=`Full`})(Br||={});var Vr;(function(e){e[e.Left=1]=`Left`,e[e.Center=2]=`Center`,e[e.Right=3]=`Right`})(Vr||={});var Hr;(function(e){e[e.LTR=0]=`LTR`,e[e.RTL=1]=`RTL`})(Hr||={});var Ur;(function(e){e[e.Both=0]=`Both`,e[e.Right=1]=`Right`,e[e.Left=2]=`Left`,e[e.None=3]=`None`})(Ur||={});function Wr(e){if(!e||e.length===0)return!1;for(let t=0,n=e.length;t<n;t++){let r=e.charCodeAt(t);if(r===10)return!0;if(r===92){if(t++,t>=n)break;let r=e.charCodeAt(t);if(r===110||r===114||r===87)return!0}}return!1}function Gr(e,t,n,r,i){if(r===0)return!0;let a=t.charCodeAt(r-1);if(e.get(a)!==0||a===13||a===10)return!0;if(i>0){let n=t.charCodeAt(r);if(e.get(n)!==0)return!0}return!1}function Kr(e,t,n,r,i){if(r+i===n)return!0;let a=t.charCodeAt(r+i);if(e.get(a)!==0||a===13||a===10)return!0;if(i>0){let n=t.charCodeAt(r+i-1);if(e.get(n)!==0)return!0}return!1}function qr(e,t,n,r,i){return Gr(e,t,n,r,i)&&Kr(e,t,n,r,i)}var Jr=class{constructor(e,t){this._wordSeparators=e,this._searchRegex=t,this._prevMatchStartIndex=-1,this._prevMatchLength=0}reset(e){this._searchRegex.lastIndex=e,this._prevMatchStartIndex=-1,this._prevMatchLength=0}next(e){let t=e.length,n;do{if(this._prevMatchStartIndex+this._prevMatchLength===t||(n=this._searchRegex.exec(e),!n))return null;let r=n.index,i=n[0].length;if(r===this._prevMatchStartIndex&&i===this._prevMatchLength){if(i===0){Pe(e,t,this._searchRegex.lastIndex)>65535?this._searchRegex.lastIndex+=2:this._searchRegex.lastIndex+=1;continue}return null}if(this._prevMatchStartIndex=r,this._prevMatchLength=i,!this._wordSeparators||qr(this._wordSeparators,e,t,r,i))return n}while(n);return null}};function Yr(e=``){let t=`(-?\\\\d*\\\\.\\\\d\\\\w*)|([^`;for(let n of`\\`~!@#$%^&*()-=+[{]}\\\\|;:'\",.<>/?`)e.indexOf(n)>=0||(t+=`\\\\`+n);return t+=`\\\\s]+)`,new RegExp(t,`g`)}const Xr=Yr();function Zr(e){let t=Xr;if(e&&e instanceof RegExp){if(e.global)t=e;else{let n=`g`;e.ignoreCase&&(n+=`i`),e.multiline&&(n+=`m`),e.unicode&&(n+=`u`),t=new RegExp(e.source,n)}}return t.lastIndex=0,t}const Qr=new x;Qr.unshift({maxLen:1e3,windowSize:15,timeBudget:150});function $r(e,t,n,r,i){if(t=Zr(t),i||=p.first(Qr),n.length>i.maxLen){let a=e-i.maxLen/2;return a<0?a=0:r+=a,n=n.substring(a,e+i.maxLen/2),$r(e,t,n,r,i)}let a=Date.now(),o=e-1-r,s=-1,c=null;for(let e=1;!(Date.now()-a>=i.timeBudget);e++){let r=o-i.windowSize*e;t.lastIndex=Math.max(0,r);let a=ei(t,n,o,s);if(!a&&c||(c=a,r<=0))break;s=r}if(c){let e={word:c[0],startColumn:r+1+c.index,endColumn:r+1+c.index+c[0].length};return t.lastIndex=0,e}return null}function ei(e,t,n,r){let i;for(;i=e.exec(t);){let t=i.index||0;if(t<=n&&e.lastIndex>=n)return i;if(r>0&&t>r)return null}return null}var ti=class{static computeUnicodeHighlights(e,t,n){let r=n?n.startLineNumber:1,i=n?n.endLineNumber:e.getLineCount(),a=new ri(t),o=a.getCandidateCodePoints(),c;c=RegExp(o===`allNonBasicAscii`?`[^\\\\t\\\\n\\\\r\\\\x20-\\\\x7E]`:`${ni(Array.from(o))}`,`g`);let l=new Jr(null,c),u=[],d=!1,f,p=0,m=0,h=0;forLoop:for(let t=r,n=i;t<=n;t++){let n=e.getLineContent(t),r=n.length;l.reset(0);do if(f=l.next(n),f){let e=f.index,i=f.index+f[0].length;e>0&&je(n.charCodeAt(e-1))&&e--,i+1<r&&je(n.charCodeAt(i-1))&&i++;let o=n.substring(e,i),c=$r(e+1,Xr,n,0);c&&c.endColumn<=e+1&&(c=null);let l=a.shouldHighlightNonBasicASCII(o,c?c.word:null);if(l!==0){if(l===3?p++:l===2?m++:l===1?h++:s(),u.length>=1e3){d=!0;break forLoop}u.push(new F(t,e+1,t,i+1))}}while(f)}return{ranges:u,hasMore:d,ambiguousCharacterCount:p,invisibleCharacterCount:m,nonBasicAsciiCharacterCount:h}}static computeUnicodeHighlightReason(e,t){let n=new ri(t);switch(n.shouldHighlightNonBasicASCII(e,null)){case 0:return null;case 2:return{kind:1};case 3:{let r=e.codePointAt(0),i=n.ambiguousCharacters.getPrimaryConfusable(r),a=Re.getLocales().filter(e=>!Re.getInstance(/* @__PURE__ */ new Set([...t.allowedLocales,e])).isAmbiguous(r));return{kind:0,confusableWith:String.fromCodePoint(i),notAmbiguousInLocales:a}}case 1:return{kind:2}}}};function ni(e,t){return`[${Ce(e.map(e=>String.fromCodePoint(e)).join(``))}]`}var ri=class{constructor(e){this.options=e,this.allowedCodePoints=new Set(e.allowedCodePoints),this.ambiguousCharacters=Re.getInstance(new Set(e.allowedLocales))}getCandidateCodePoints(){if(this.options.nonBasicASCII)return`allNonBasicAscii`;let e=/* @__PURE__ */ new Set;if(this.options.invisibleCharacters)for(let t of ze.codePoints)ii(String.fromCodePoint(t))||e.add(t);if(this.options.ambiguousCharacters)for(let t of this.ambiguousCharacters.getConfusableCodePoints())e.add(t);for(let t of this.allowedCodePoints)e.delete(t);return e}shouldHighlightNonBasicASCII(e,t){let n=e.codePointAt(0);if(this.allowedCodePoints.has(n))return 0;if(this.options.nonBasicASCII)return 1;let r=!1,i=!1;if(t)for(let e of t){let t=e.codePointAt(0),n=Ie(e);r||=n,!n&&!this.ambiguousCharacters.isAmbiguous(t)&&!ze.isInvisibleCharacter(t)&&(i=!0)}return!r&&i?0:this.options.invisibleCharacters&&!ii(e)&&ze.isInvisibleCharacter(n)?2:this.options.ambiguousCharacters&&this.ambiguousCharacters.isAmbiguous(n)?3:0}};function ii(e){return e===` `||e===`\n`||e===`	`}var ai=class{constructor(e,t,n){this.changes=e,this.moves=t,this.hitTimeout=n}},oi=class{constructor(e,t){this.lineRangeMapping=e,this.changes=t}};function si(e,t,n=(e,t)=>e===t){if(e===t)return!0;if(!e||!t||e.length!==t.length)return!1;for(let r=0,i=e.length;r<i;r++)if(!n(e[r],t[r]))return!1;return!0}function*ci(e,t){let n,r;for(let i of e)r!==void 0&&t(r,i)?n.push(i):(n&&(yield n),n=[i]),r=i;n&&(yield n)}function li(e,t){for(let n=0;n<=e.length;n++)t(n===0?void 0:e[n-1],n===e.length?void 0:e[n])}function ui(e,t){for(let n=0;n<e.length;n++)t(n===0?void 0:e[n-1],e[n],n+1===e.length?void 0:e[n+1])}function di(e,t){for(let n of t)e.push(n)}var fi;(function(e){function t(e){return e<0}e.isLessThan=t;function n(e){return e<=0}e.isLessThanOrEqual=n;function r(e){return e>0}e.isGreaterThan=r;function i(e){return e===0}e.isNeitherLessOrGreaterThan=i,e.greaterThan=1,e.lessThan=-1,e.neitherLessOrGreaterThan=0})(fi||={});function pi(e,t){return(n,r)=>t(e(n),e(r))}const mi=(e,t)=>e-t;function hi(e){return(t,n)=>-e(t,n)}(class e{static{this.empty=new e(e=>{})}constructor(e){this.iterate=e}toArray(){let e=[];return this.iterate(t=>(e.push(t),!0)),e}filter(t){return new e(e=>this.iterate(n=>!t(n)||e(n)))}map(t){return new e(e=>this.iterate(n=>e(t(n))))}findLast(e){let t;return this.iterate(n=>(e(n)&&(t=n),!0)),t}findLastMaxBy(e){let t,n=!0;return this.iterate(r=>((n||fi.isGreaterThan(e(r,t)))&&(n=!1,t=r),!0)),t}});var G=class e{static fromTo(t,n){return new e(t,n)}static addRange(t,n){let r=0;for(;r<n.length&&n[r].endExclusive<t.start;)r++;let i=r;for(;i<n.length&&n[i].start<=t.endExclusive;)i++;if(r===i)n.splice(r,0,t);else{let a=Math.min(t.start,n[r].start),o=Math.max(t.endExclusive,n[i-1].endExclusive);n.splice(r,i-r,new e(a,o))}}static tryCreate(t,n){if(!(t>n))return new e(t,n)}static ofLength(t){return new e(0,t)}static ofStartAndLength(t,n){return new e(t,t+n)}static emptyAt(t){return new e(t,t)}constructor(e,t){if(this.start=e,this.endExclusive=t,e>t)throw new o(`Invalid range: ${this.toString()}`)}get isEmpty(){return this.start===this.endExclusive}delta(t){return new e(this.start+t,this.endExclusive+t)}deltaStart(t){return new e(this.start+t,this.endExclusive)}deltaEnd(t){return new e(this.start,this.endExclusive+t)}get length(){return this.endExclusive-this.start}toString(){return`[${this.start}, ${this.endExclusive})`}equals(e){return this.start===e.start&&this.endExclusive===e.endExclusive}contains(e){return this.start<=e&&e<this.endExclusive}join(t){return new e(Math.min(this.start,t.start),Math.max(this.endExclusive,t.endExclusive))}intersect(t){let n=Math.max(this.start,t.start),r=Math.min(this.endExclusive,t.endExclusive);if(n<=r)return new e(n,r)}intersectionLength(e){let t=Math.max(this.start,e.start),n=Math.min(this.endExclusive,e.endExclusive);return Math.max(0,n-t)}intersects(e){return Math.max(this.start,e.start)<Math.min(this.endExclusive,e.endExclusive)}intersectsOrTouches(e){return Math.max(this.start,e.start)<=Math.min(this.endExclusive,e.endExclusive)}isBefore(e){return this.endExclusive<=e.start}isAfter(e){return this.start>=e.endExclusive}slice(e){return e.slice(this.start,this.endExclusive)}substring(e){return e.substring(this.start,this.endExclusive)}clip(e){if(this.isEmpty)throw new o(`Invalid clipping range: ${this.toString()}`);return Math.max(this.start,Math.min(this.endExclusive-1,e))}clipCyclic(e){if(this.isEmpty)throw new o(`Invalid clipping range: ${this.toString()}`);return e<this.start?this.endExclusive-(this.start-e)%this.length:e>=this.endExclusive?this.start+(e-this.start)%this.length:e}forEach(e){for(let t=this.start;t<this.endExclusive;t++)e(t)}joinRightTouching(t){if(this.endExclusive!==t.start)throw new o(`Invalid join: ${this.toString()} and ${t.toString()}`);return new e(this.start,t.endExclusive)}};function gi(e,t){let n=_i(e,t);return n===-1?void 0:e[n]}function _i(e,t,n=0,r=e.length){let i=n,a=r;for(;i<a;){let n=Math.floor((i+a)/2);t(e[n])?i=n+1:a=n}return i-1}function vi(e,t){let n=yi(e,t);return n===e.length?void 0:e[n]}function yi(e,t,n=0,r=e.length){let i=n,a=r;for(;i<a;){let n=Math.floor((i+a)/2);t(e[n])?a=n:i=n+1}return i}var bi=class e{static{this.assertInvariants=!1}constructor(e){this._array=e,this._findLastMonotonousLastIdx=0}findLastMonotonous(t){if(e.assertInvariants){if(this._prevFindLastPredicate){for(let e of this._array)if(this._prevFindLastPredicate(e)&&!t(e))throw Error(`MonotonousArray: current predicate must be weaker than (or equal to) the previous predicate.`)}this._prevFindLastPredicate=t}let n=_i(this._array,t,this._findLastMonotonousLastIdx);return this._findLastMonotonousLastIdx=n+1,n===-1?void 0:this._array[n]}},K=class e{static ofLength(t,n){return new e(t,t+n)}static fromRange(t){return new e(t.startLineNumber,t.endLineNumber)}static fromRangeInclusive(t){return new e(t.startLineNumber,t.endLineNumber+1)}static{this.compareByStart=pi(e=>e.startLineNumber,mi)}static joinMany(e){if(e.length===0)return[];let t=new xi(e[0].slice());for(let n=1;n<e.length;n++)t=t.getUnion(new xi(e[n].slice()));return t.ranges}static join(t){if(t.length===0)throw new o(`lineRanges cannot be empty`);let n=t[0].startLineNumber,r=t[0].endLineNumberExclusive;for(let e=1;e<t.length;e++)n=Math.min(n,t[e].startLineNumber),r=Math.max(r,t[e].endLineNumberExclusive);return new e(n,r)}static deserialize(t){return new e(t[0],t[1])}constructor(e,t){if(e>t)throw new o(`startLineNumber ${e} cannot be after endLineNumberExclusive ${t}`);this.startLineNumber=e,this.endLineNumberExclusive=t}contains(e){return this.startLineNumber<=e&&e<this.endLineNumberExclusive}get isEmpty(){return this.startLineNumber===this.endLineNumberExclusive}delta(t){return new e(this.startLineNumber+t,this.endLineNumberExclusive+t)}deltaLength(t){return new e(this.startLineNumber,this.endLineNumberExclusive+t)}get length(){return this.endLineNumberExclusive-this.startLineNumber}join(t){return new e(Math.min(this.startLineNumber,t.startLineNumber),Math.max(this.endLineNumberExclusive,t.endLineNumberExclusive))}toString(){return`[${this.startLineNumber},${this.endLineNumberExclusive})`}intersect(t){let n=Math.max(this.startLineNumber,t.startLineNumber),r=Math.min(this.endLineNumberExclusive,t.endLineNumberExclusive);if(n<=r)return new e(n,r)}intersectsStrict(e){return this.startLineNumber<e.endLineNumberExclusive&&e.startLineNumber<this.endLineNumberExclusive}intersectsOrTouches(e){return this.startLineNumber<=e.endLineNumberExclusive&&e.startLineNumber<=this.endLineNumberExclusive}equals(e){return this.startLineNumber===e.startLineNumber&&this.endLineNumberExclusive===e.endLineNumberExclusive}toInclusiveRange(){return this.isEmpty?null:new F(this.startLineNumber,1,this.endLineNumberExclusive-1,2**53-1)}toExclusiveRange(){return new F(this.startLineNumber,1,this.endLineNumberExclusive,1)}mapToLineArray(e){let t=[];for(let n=this.startLineNumber;n<this.endLineNumberExclusive;n++)t.push(e(n));return t}forEach(e){for(let t=this.startLineNumber;t<this.endLineNumberExclusive;t++)e(t)}serialize(){return[this.startLineNumber,this.endLineNumberExclusive]}toOffsetRange(){return new G(this.startLineNumber-1,this.endLineNumberExclusive-1)}addMargin(t,n){return new e(this.startLineNumber-t,this.endLineNumberExclusive+n)}},xi=class e{constructor(e=[]){this._normalizedRanges=e}get ranges(){return this._normalizedRanges}addRange(e){if(e.length===0)return;let t=yi(this._normalizedRanges,t=>t.endLineNumberExclusive>=e.startLineNumber),n=_i(this._normalizedRanges,t=>t.startLineNumber<=e.endLineNumberExclusive)+1;if(t===n)this._normalizedRanges.splice(t,0,e);else if(t===n-1){let n=this._normalizedRanges[t];this._normalizedRanges[t]=n.join(e)}else{let r=this._normalizedRanges[t].join(this._normalizedRanges[n-1]).join(e);this._normalizedRanges.splice(t,n-t,r)}}contains(e){let t=gi(this._normalizedRanges,t=>t.startLineNumber<=e);return!!t&&t.endLineNumberExclusive>e}intersects(e){let t=gi(this._normalizedRanges,t=>t.startLineNumber<e.endLineNumberExclusive);return!!t&&t.endLineNumberExclusive>e.startLineNumber}getUnion(t){if(this._normalizedRanges.length===0)return t;if(t._normalizedRanges.length===0)return this;let n=[],r=0,i=0,a=null;for(;r<this._normalizedRanges.length||i<t._normalizedRanges.length;){let e=null;if(r<this._normalizedRanges.length&&i<t._normalizedRanges.length){let n=this._normalizedRanges[r],a=t._normalizedRanges[i];n.startLineNumber<a.startLineNumber?(e=n,r++):(e=a,i++)}else r<this._normalizedRanges.length?(e=this._normalizedRanges[r],r++):(e=t._normalizedRanges[i],i++);a===null?a=e:a.endLineNumberExclusive>=e.startLineNumber?a=new K(a.startLineNumber,Math.max(a.endLineNumberExclusive,e.endLineNumberExclusive)):(n.push(a),a=e)}return a!==null&&n.push(a),new e(n)}subtractFrom(t){let n=yi(this._normalizedRanges,e=>e.endLineNumberExclusive>=t.startLineNumber),r=_i(this._normalizedRanges,e=>e.startLineNumber<=t.endLineNumberExclusive)+1;if(n===r)return new e([t]);let i=[],a=t.startLineNumber;for(let e=n;e<r;e++){let t=this._normalizedRanges[e];t.startLineNumber>a&&i.push(new K(a,t.startLineNumber)),a=t.endLineNumberExclusive}return a<t.endLineNumberExclusive&&i.push(new K(a,t.endLineNumberExclusive)),new e(i)}toString(){return this._normalizedRanges.map(e=>e.toString()).join(`, `)}getIntersection(t){let n=[],r=0,i=0;for(;r<this._normalizedRanges.length&&i<t._normalizedRanges.length;){let e=this._normalizedRanges[r],a=t._normalizedRanges[i],o=e.intersect(a);o&&!o.isEmpty&&n.push(o),e.endLineNumberExclusive<a.endLineNumberExclusive?r++:i++}return new e(n)}getWithDelta(t){return new e(this._normalizedRanges.map(e=>e.delta(t)))}},Si=class e{static{this.zero=new e(0,0)}static betweenPositions(t,n){return t.lineNumber===n.lineNumber?new e(0,n.column-t.column):new e(n.lineNumber-t.lineNumber,n.column-1)}static fromPosition(t){return new e(t.lineNumber-1,t.column-1)}static ofRange(t){return e.betweenPositions(t.getStartPosition(),t.getEndPosition())}static ofText(t){let n=0,r=0;for(let e of t)e===`\n`?(n++,r=0):r++;return new e(n,r)}constructor(e,t){this.lineCount=e,this.columnCount=t}isGreaterThanOrEqualTo(e){return this.lineCount===e.lineCount?this.columnCount>=e.columnCount:this.lineCount>e.lineCount}add(t){return t.lineCount===0?new e(this.lineCount,this.columnCount+t.columnCount):new e(this.lineCount+t.lineCount,t.columnCount)}createRange(e){return this.lineCount===0?new F(e.lineNumber,e.column,e.lineNumber,e.column+this.columnCount):new F(e.lineNumber,e.column,e.lineNumber+this.lineCount,this.columnCount+1)}toRange(){return new F(1,1,this.lineCount+1,this.columnCount+1)}toLineRange(){return K.ofLength(1,this.lineCount+1)}addToPosition(e){return this.lineCount===0?new P(e.lineNumber,e.column+this.columnCount):new P(e.lineNumber+this.lineCount,this.columnCount+1)}toString(){return`${this.lineCount},${this.columnCount}`}},Ci=class{getOffsetRange(e){return new G(this.getOffset(e.getStartPosition()),this.getOffset(e.getEndPosition()))}getRange(e){return F.fromPositions(this.getPosition(e.start),this.getPosition(e.endExclusive))}getStringReplacement(e){return new wi.deps.StringReplacement(this.getOffsetRange(e.range),e.text)}getTextReplacement(e){return new wi.deps.TextReplacement(this.getRange(e.replaceRange),e.newText)}getTextEdit(e){let t=e.replacements.map(e=>this.getTextReplacement(e));return new wi.deps.TextEdit(t)}},wi=class{static{this._deps=void 0}static get deps(){if(!this._deps)throw Error(`Dependencies not set. Call _setDependencies first.`);return this._deps}},Ti=class extends Ci{constructor(e){super(),this.text=e,this.lineStartOffsetByLineIdx=[],this.lineEndOffsetByLineIdx=[],this.lineStartOffsetByLineIdx.push(0);for(let t=0;t<e.length;t++)e.charAt(t)===`\n`&&(this.lineStartOffsetByLineIdx.push(t+1),t>0&&e.charAt(t-1)===`\\r`?this.lineEndOffsetByLineIdx.push(t-1):this.lineEndOffsetByLineIdx.push(t));this.lineEndOffsetByLineIdx.push(e.length)}getOffset(e){let t=this._validatePosition(e);return this.lineStartOffsetByLineIdx[t.lineNumber-1]+t.column-1}_validatePosition(e){if(e.lineNumber<1)return new P(1,1);let t=this.textLength.lineCount+1;if(e.lineNumber>t)return new P(t,this.getLineLength(t)+1);if(e.column<1)return new P(e.lineNumber,1);let n=this.getLineLength(e.lineNumber);return e.column-1>n?new P(e.lineNumber,n+1):e}getPosition(e){let t=_i(this.lineStartOffsetByLineIdx,t=>t<=e);return new P(t+1,e-this.lineStartOffsetByLineIdx[t]+1)}get textLength(){let e=this.lineStartOffsetByLineIdx.length-1;return new wi.deps.TextLength(e,this.text.length-this.lineStartOffsetByLineIdx[e])}getLineLength(e){return this.lineEndOffsetByLineIdx[e-1]-this.lineStartOffsetByLineIdx[e-1]}},Ei=class{constructor(){this._transformer=void 0}get endPositionExclusive(){return this.length.addToPosition(new P(1,1))}get lineRange(){return this.length.toLineRange()}getValue(){return this.getValueOfRange(this.length.toRange())}getValueOfOffsetRange(e){return this.getValueOfRange(this.getTransformer().getRange(e))}getLineLength(e){return this.getValueOfRange(new F(e,1,e,2**53-1)).length}getTransformer(){return this._transformer||=new Ti(this.getValue()),this._transformer}getLineAt(e){return this.getValueOfRange(new F(e,1,e,2**53-1))}},Di=class extends Ei{constructor(e,t){c(t>=1),super(),this._getLineContent=e,this._lineCount=t}getValueOfRange(e){if(e.startLineNumber===e.endLineNumber)return this._getLineContent(e.startLineNumber).substring(e.startColumn-1,e.endColumn-1);let t=this._getLineContent(e.startLineNumber).substring(e.startColumn-1);for(let n=e.startLineNumber+1;n<e.endLineNumber;n++)t+=`\n`+this._getLineContent(n);return t+=`\n`+this._getLineContent(e.endLineNumber).substring(0,e.endColumn-1),t}getLineLength(e){return this._getLineContent(e).length}get length(){let e=this._getLineContent(this._lineCount);return new Si(this._lineCount-1,e.length)}},Oi=class extends Di{constructor(e){super(t=>e[t-1],e.length)}},ki=class e{static joinReplacements(t,n){if(t.length===0)throw new o;if(t.length===1)return t[0];let r=t[0].range.getStartPosition(),i=t[t.length-1].range.getEndPosition(),a=``;for(let e=0;e<t.length;e++){let r=t[e];if(a+=r.text,e<t.length-1){let i=t[e+1],o=F.fromPositions(r.range.getEndPosition(),i.range.getStartPosition()),s=n.getValueOfRange(o);a+=s}}return new e(F.fromPositions(r,i),a)}static fromStringReplacement(t,n){return new e(n.getTransformer().getRange(t.replaceRange),t.newText)}static delete(t){return new e(t,``)}constructor(e,t){this.range=e,this.text=t}get isEmpty(){return this.range.isEmpty()&&this.text.length===0}static equals(e,t){return e.range.equalsRange(t.range)&&e.text===t.text}equals(t){return e.equals(this,t)}removeCommonPrefixAndSuffix(e){return this.removeCommonPrefix(e).removeCommonSuffix(e)}removeCommonPrefix(t){let n=t.getValueOfRange(this.range).replaceAll(`\\r\n`,`\n`),r=this.text.replaceAll(`\\r\n`,`\n`),i=ke(n,r),a=Si.ofText(n.substring(0,i)).addToPosition(this.range.getStartPosition()),o=r.substring(i),s=F.fromPositions(a,this.range.getEndPosition());return new e(s,o)}removeCommonSuffix(t){let n=t.getValueOfRange(this.range).replaceAll(`\\r\n`,`\n`),r=this.text.replaceAll(`\\r\n`,`\n`),i=Ae(n,r),a=Si.ofText(n.substring(0,n.length-i)).addToPosition(this.range.getStartPosition()),o=r.substring(0,r.length-i),s=F.fromPositions(this.range.getStartPosition(),a);return new e(s,o)}toString(){let e=this.range.getStartPosition(),t=this.range.getEndPosition();return`(${e.lineNumber},${e.column} -> ${t.lineNumber},${t.column}): \"${this.text}\"`}},Ai=class e{static inverse(t,n,r){let i=[],a=1,o=1;for(let n of t){let t=new e(new K(a,n.original.startLineNumber),new K(o,n.modified.startLineNumber));t.modified.isEmpty||i.push(t),a=n.original.endLineNumberExclusive,o=n.modified.endLineNumberExclusive}let s=new e(new K(a,n+1),new K(o,r+1));return s.modified.isEmpty||i.push(s),i}static clip(t,n,r){let i=[];for(let a of t){let t=a.original.intersect(n),o=a.modified.intersect(r);t&&!t.isEmpty&&o&&!o.isEmpty&&i.push(new e(t,o))}return i}constructor(e,t){this.original=e,this.modified=t}toString(){return`{${this.original.toString()}->${this.modified.toString()}}`}flip(){return new e(this.modified,this.original)}join(t){return new e(this.original.join(t.original),this.modified.join(t.modified))}toRangeMapping(){let e=this.original.toInclusiveRange(),t=this.modified.toInclusiveRange();if(e&&t)return new q(e,t);if(this.original.startLineNumber===1||this.modified.startLineNumber===1){if(this.modified.startLineNumber!==1||this.original.startLineNumber!==1)throw new o(`not a valid diff`);return new q(new F(this.original.startLineNumber,1,this.original.endLineNumberExclusive,1),new F(this.modified.startLineNumber,1,this.modified.endLineNumberExclusive,1))}return new q(new F(this.original.startLineNumber-1,2**53-1,this.original.endLineNumberExclusive-1,2**53-1),new F(this.modified.startLineNumber-1,2**53-1,this.modified.endLineNumberExclusive-1,2**53-1))}toRangeMapping2(e,t){if(Mi(this.original.endLineNumberExclusive,e)&&Mi(this.modified.endLineNumberExclusive,t))return new q(new F(this.original.startLineNumber,1,this.original.endLineNumberExclusive,1),new F(this.modified.startLineNumber,1,this.modified.endLineNumberExclusive,1));if(!this.original.isEmpty&&!this.modified.isEmpty)return new q(F.fromPositions(new P(this.original.startLineNumber,1),ji(new P(this.original.endLineNumberExclusive-1,2**53-1),e)),F.fromPositions(new P(this.modified.startLineNumber,1),ji(new P(this.modified.endLineNumberExclusive-1,2**53-1),t)));if(this.original.startLineNumber>1&&this.modified.startLineNumber>1)return new q(F.fromPositions(ji(new P(this.original.startLineNumber-1,2**53-1),e),ji(new P(this.original.endLineNumberExclusive-1,2**53-1),e)),F.fromPositions(ji(new P(this.modified.startLineNumber-1,2**53-1),t),ji(new P(this.modified.endLineNumberExclusive-1,2**53-1),t)));throw new o}};function ji(e,t){if(e.lineNumber<1)return new P(1,1);if(e.lineNumber>t.length)return new P(t.length,t[t.length-1].length+1);let n=t[e.lineNumber-1];return e.column>n.length+1?new P(e.lineNumber,n.length+1):e}function Mi(e,t){return e>=1&&e<=t.length}var Ni=class e extends Ai{static fromRangeMappings(t){let n=K.join(t.map(e=>K.fromRangeInclusive(e.originalRange))),r=K.join(t.map(e=>K.fromRangeInclusive(e.modifiedRange)));return new e(n,r,t)}constructor(e,t,n){super(e,t),this.innerChanges=n}flip(){return new e(this.modified,this.original,this.innerChanges?.map(e=>e.flip()))}withInnerChangesFromLineRanges(){return new e(this.original,this.modified,[this.toRangeMapping()])}},q=class e{static fromEdit(t){let n=t.getNewRanges();return t.replacements.map((t,r)=>new e(t.range,n[r]))}static assertSorted(e){for(let t=1;t<e.length;t++){let n=e[t-1],r=e[t];if(!(n.originalRange.getEndPosition().isBeforeOrEqual(r.originalRange.getStartPosition())&&n.modifiedRange.getEndPosition().isBeforeOrEqual(r.modifiedRange.getStartPosition())))throw new o(`Range mappings must be sorted`)}}constructor(e,t){this.originalRange=e,this.modifiedRange=t}toString(){return`{${this.originalRange.toString()}->${this.modifiedRange.toString()}}`}flip(){return new e(this.modifiedRange,this.originalRange)}toTextEdit(e){let t=e.getValueOfRange(this.modifiedRange);return new ki(this.originalRange,t)}};function Pi(e,t,n,r=!1){let i=[];for(let r of ci(e.map(e=>Fi(e,t,n)),(e,t)=>e.original.intersectsOrTouches(t.original)||e.modified.intersectsOrTouches(t.modified))){let e=r[0],t=r[r.length-1];i.push(new Ni(e.original.join(t.original),e.modified.join(t.modified),r.map(e=>e.innerChanges[0])))}return l(()=>!r&&i.length>0&&(i[0].modified.startLineNumber!==i[0].original.startLineNumber||n.length.lineCount-i[i.length-1].modified.endLineNumberExclusive!==t.length.lineCount-i[i.length-1].original.endLineNumberExclusive)?!1:u(i,(e,t)=>t.original.startLineNumber-e.original.endLineNumberExclusive===t.modified.startLineNumber-e.modified.endLineNumberExclusive&&e.original.endLineNumberExclusive<t.original.startLineNumber&&e.modified.endLineNumberExclusive<t.modified.startLineNumber)),i}function Fi(e,t,n){let r=0,i=0;return e.modifiedRange.endColumn===1&&e.originalRange.endColumn===1&&e.originalRange.startLineNumber+r<=e.originalRange.endLineNumber&&e.modifiedRange.startLineNumber+r<=e.modifiedRange.endLineNumber&&(i=-1),e.modifiedRange.startColumn-1>=n.getLineLength(e.modifiedRange.startLineNumber)&&e.originalRange.startColumn-1>=t.getLineLength(e.originalRange.startLineNumber)&&e.originalRange.startLineNumber<=e.originalRange.endLineNumber+i&&e.modifiedRange.startLineNumber<=e.modifiedRange.endLineNumber+i&&(r=1),new Ni(new K(e.originalRange.startLineNumber+r,e.originalRange.endLineNumber+1+i),new K(e.modifiedRange.startLineNumber+r,e.modifiedRange.endLineNumber+1+i),[e])}var Ii=class{computeDiff(e,t,n){let r=new Ui(e,t,{maxComputationTime:n.maxComputationTimeMs,shouldIgnoreTrimWhitespace:n.ignoreTrimWhitespace,shouldComputeCharChanges:!0,shouldMakePrettyDiff:!0,shouldPostProcessCharChanges:!0}).computeDiff(),i=[],a=null;for(let e of r.changes){let t;t=e.originalEndLineNumber===0?new K(e.originalStartLineNumber+1,e.originalStartLineNumber+1):new K(e.originalStartLineNumber,e.originalEndLineNumber+1);let n;n=e.modifiedEndLineNumber===0?new K(e.modifiedStartLineNumber+1,e.modifiedStartLineNumber+1):new K(e.modifiedStartLineNumber,e.modifiedEndLineNumber+1);let r=new Ni(t,n,e.charChanges?.map(e=>new q(new F(e.originalStartLineNumber,e.originalStartColumn,e.originalEndLineNumber,e.originalEndColumn),new F(e.modifiedStartLineNumber,e.modifiedStartColumn,e.modifiedEndLineNumber,e.modifiedEndColumn))));a&&(a.modified.endLineNumberExclusive===r.modified.startLineNumber||a.original.endLineNumberExclusive===r.original.startLineNumber)&&(r=new Ni(a.original.join(r.original),a.modified.join(r.modified),a.innerChanges&&r.innerChanges?a.innerChanges.concat(r.innerChanges):void 0),i.pop()),i.push(r),a=r}return l(()=>u(i,(e,t)=>t.original.startLineNumber-e.original.endLineNumberExclusive===t.modified.startLineNumber-e.modified.endLineNumberExclusive&&e.original.endLineNumberExclusive<t.original.startLineNumber&&e.modified.endLineNumberExclusive<t.modified.startLineNumber)),new ai(i,[],r.quitEarly)}};function Li(e,t,n,r){return new ft(e,t,n).ComputeDiff(r)}var Ri=class{constructor(e){let t=[],n=[];for(let r=0,i=e.length;r<i;r++)t[r]=Wi(e[r],1),n[r]=Gi(e[r],1);this.lines=e,this._startColumns=t,this._endColumns=n}getElements(){let e=[];for(let t=0,n=this.lines.length;t<n;t++)e[t]=this.lines[t].substring(this._startColumns[t]-1,this._endColumns[t]-1);return e}getStrictElement(e){return this.lines[e]}getStartLineNumber(e){return e+1}getEndLineNumber(e){return e+1}createCharSequence(e,t,n){let r=[],i=[],a=[],o=0;for(let s=t;s<=n;s++){let t=this.lines[s],c=e?this._startColumns[s]:1,l=e?this._endColumns[s]:t.length+1;for(let e=c;e<l;e++)r[o]=t.charCodeAt(e-1),i[o]=s+1,a[o]=e,o++;!e&&s<n&&(r[o]=10,i[o]=s+1,a[o]=t.length+1,o++)}return new zi(r,i,a)}},zi=class{constructor(e,t,n){this._charCodes=e,this._lineNumbers=t,this._columns=n}toString(){return`[`+this._charCodes.map((e,t)=>(e===10?`\\\\n`:String.fromCharCode(e))+`-(${this._lineNumbers[t]},${this._columns[t]})`).join(`, `)+`]`}_assertIndex(e,t){if(e<0||e>=t.length)throw Error(`Illegal index`)}getElements(){return this._charCodes}getStartLineNumber(e){return e>0&&e===this._lineNumbers.length?this.getEndLineNumber(e-1):(this._assertIndex(e,this._lineNumbers),this._lineNumbers[e])}getEndLineNumber(e){return e===-1?this.getStartLineNumber(e+1):(this._assertIndex(e,this._lineNumbers),this._charCodes[e]===10?this._lineNumbers[e]+1:this._lineNumbers[e])}getStartColumn(e){return e>0&&e===this._columns.length?this.getEndColumn(e-1):(this._assertIndex(e,this._columns),this._columns[e])}getEndColumn(e){return e===-1?this.getStartColumn(e+1):(this._assertIndex(e,this._columns),this._charCodes[e]===10?1:this._columns[e]+1)}},Bi=class e{constructor(e,t,n,r,i,a,o,s){this.originalStartLineNumber=e,this.originalStartColumn=t,this.originalEndLineNumber=n,this.originalEndColumn=r,this.modifiedStartLineNumber=i,this.modifiedStartColumn=a,this.modifiedEndLineNumber=o,this.modifiedEndColumn=s}static createFromDiffChange(t,n,r){let i=n.getStartLineNumber(t.originalStart),a=n.getStartColumn(t.originalStart),o=n.getEndLineNumber(t.originalStart+t.originalLength-1),s=n.getEndColumn(t.originalStart+t.originalLength-1),c=r.getStartLineNumber(t.modifiedStart),l=r.getStartColumn(t.modifiedStart),u=r.getEndLineNumber(t.modifiedStart+t.modifiedLength-1),d=r.getEndColumn(t.modifiedStart+t.modifiedLength-1);return new e(i,a,o,s,c,l,u,d)}};function Vi(e){if(e.length<=1)return e;let t=[e[0]],n=t[0];for(let r=1,i=e.length;r<i;r++){let i=e[r],a=i.originalStart-(n.originalStart+n.originalLength),o=i.modifiedStart-(n.modifiedStart+n.modifiedLength);Math.min(a,o)<3?(n.originalLength=i.originalStart+i.originalLength-n.originalStart,n.modifiedLength=i.modifiedStart+i.modifiedLength-n.modifiedStart):(t.push(i),n=i)}return t}var Hi=class e{constructor(e,t,n,r,i){this.originalStartLineNumber=e,this.originalEndLineNumber=t,this.modifiedStartLineNumber=n,this.modifiedEndLineNumber=r,this.charChanges=i}static createFromDiffResult(t,n,r,i,a,o,s){let c,l,u,d,f;if(n.originalLength===0?(c=r.getStartLineNumber(n.originalStart)-1,l=0):(c=r.getStartLineNumber(n.originalStart),l=r.getEndLineNumber(n.originalStart+n.originalLength-1)),n.modifiedLength===0?(u=i.getStartLineNumber(n.modifiedStart)-1,d=0):(u=i.getStartLineNumber(n.modifiedStart),d=i.getEndLineNumber(n.modifiedStart+n.modifiedLength-1)),o&&n.originalLength>0&&n.originalLength<20&&n.modifiedLength>0&&n.modifiedLength<20&&a()){let e=r.createCharSequence(t,n.originalStart,n.originalStart+n.originalLength-1),o=i.createCharSequence(t,n.modifiedStart,n.modifiedStart+n.modifiedLength-1);if(e.getElements().length>0&&o.getElements().length>0){let t=Li(e,o,a,!0).changes;s&&(t=Vi(t)),f=[];for(let n=0,r=t.length;n<r;n++)f.push(Bi.createFromDiffChange(t[n],e,o))}}return new e(c,l,u,d,f)}},Ui=class{constructor(e,t,n){this.shouldComputeCharChanges=n.shouldComputeCharChanges,this.shouldPostProcessCharChanges=n.shouldPostProcessCharChanges,this.shouldIgnoreTrimWhitespace=n.shouldIgnoreTrimWhitespace,this.shouldMakePrettyDiff=n.shouldMakePrettyDiff,this.originalLines=e,this.modifiedLines=t,this.original=new Ri(e),this.modified=new Ri(t),this.continueLineDiff=Ki(n.maxComputationTime),this.continueCharDiff=Ki(n.maxComputationTime===0?0:Math.min(n.maxComputationTime,5e3))}computeDiff(){if(this.original.lines.length===1&&this.original.lines[0].length===0)return this.modified.lines.length===1&&this.modified.lines[0].length===0?{quitEarly:!1,changes:[]}:{quitEarly:!1,changes:[{originalStartLineNumber:1,originalEndLineNumber:1,modifiedStartLineNumber:1,modifiedEndLineNumber:this.modified.lines.length,charChanges:void 0}]};if(this.modified.lines.length===1&&this.modified.lines[0].length===0)return{quitEarly:!1,changes:[{originalStartLineNumber:1,originalEndLineNumber:this.original.lines.length,modifiedStartLineNumber:1,modifiedEndLineNumber:1,charChanges:void 0}]};let e=Li(this.original,this.modified,this.continueLineDiff,this.shouldMakePrettyDiff),t=e.changes,n=e.quitEarly;if(this.shouldIgnoreTrimWhitespace){let e=[];for(let n=0,r=t.length;n<r;n++)e.push(Hi.createFromDiffResult(this.shouldIgnoreTrimWhitespace,t[n],this.original,this.modified,this.continueCharDiff,this.shouldComputeCharChanges,this.shouldPostProcessCharChanges));return{quitEarly:n,changes:e}}let r=[],i=0,a=0;for(let e=-1,n=t.length;e<n;e++){let o=e+1<n?t[e+1]:null,s=o?o.originalStart:this.originalLines.length,c=o?o.modifiedStart:this.modifiedLines.length;for(;i<s&&a<c;){let e=this.originalLines[i],t=this.modifiedLines[a];if(e!==t){{let n=Wi(e,1),o=Wi(t,1);for(;n>1&&o>1&&e.charCodeAt(n-2)===t.charCodeAt(o-2);)n--,o--;(n>1||o>1)&&this._pushTrimWhitespaceCharChange(r,i+1,1,n,a+1,1,o)}{let n=Gi(e,1),o=Gi(t,1),s=e.length+1,c=t.length+1;for(;n<s&&o<c&&e.charCodeAt(n-1)===e.charCodeAt(o-1);)n++,o++;(n<s||o<c)&&this._pushTrimWhitespaceCharChange(r,i+1,n,s,a+1,o,c)}}i++,a++}o&&(r.push(Hi.createFromDiffResult(this.shouldIgnoreTrimWhitespace,o,this.original,this.modified,this.continueCharDiff,this.shouldComputeCharChanges,this.shouldPostProcessCharChanges)),i+=o.originalLength,a+=o.modifiedLength)}return{quitEarly:n,changes:r}}_pushTrimWhitespaceCharChange(e,t,n,r,i,a,o){if(this._mergeTrimWhitespaceCharChange(e,t,n,r,i,a,o))return;let s;this.shouldComputeCharChanges&&(s=[new Bi(t,n,t,r,i,a,i,o)]),e.push(new Hi(t,t,i,i,s))}_mergeTrimWhitespaceCharChange(e,t,n,r,i,a,o){let s=e.length;if(s===0)return!1;let c=e[s-1];return c.originalEndLineNumber===0||c.modifiedEndLineNumber===0?!1:c.originalEndLineNumber===t&&c.modifiedEndLineNumber===i?(this.shouldComputeCharChanges&&c.charChanges&&c.charChanges.push(new Bi(t,n,t,r,i,a,i,o)),!0):c.originalEndLineNumber+1===t&&c.modifiedEndLineNumber+1===i&&(c.originalEndLineNumber=t,c.modifiedEndLineNumber=i,this.shouldComputeCharChanges&&c.charChanges&&c.charChanges.push(new Bi(t,n,t,r,i,a,i,o)),!0)}};function Wi(e,t){let n=Ee(e);return n===-1?t:n+1}function Gi(e,t){let n=De(e);return n===-1?t:n+2}function Ki(e){if(e===0)return()=>!0;let t=Date.now();return()=>Date.now()-t<e}var J=class e{static trivial(t,n){return new e([new Y(G.ofLength(t.length),G.ofLength(n.length))],!1)}static trivialTimedOut(t,n){return new e([new Y(G.ofLength(t.length),G.ofLength(n.length))],!0)}constructor(e,t){this.diffs=e,this.hitTimeout=t}},Y=class e{static invert(t,n){let r=[];return li(t,(t,i)=>{r.push(e.fromOffsetPairs(t?t.getEndExclusives():X.zero,i?i.getStarts():new X(n,(t?t.seq2Range.endExclusive-t.seq1Range.endExclusive:0)+n)))}),r}static fromOffsetPairs(t,n){return new e(new G(t.offset1,n.offset1),new G(t.offset2,n.offset2))}static assertSorted(e){let t;for(let n of e){if(t&&!(t.seq1Range.endExclusive<=n.seq1Range.start&&t.seq2Range.endExclusive<=n.seq2Range.start))throw new o(`Sequence diffs must be sorted`);t=n}}constructor(e,t){this.seq1Range=e,this.seq2Range=t}swap(){return new e(this.seq2Range,this.seq1Range)}toString(){return`${this.seq1Range} <-> ${this.seq2Range}`}join(t){return new e(this.seq1Range.join(t.seq1Range),this.seq2Range.join(t.seq2Range))}delta(t){return t===0?this:new e(this.seq1Range.delta(t),this.seq2Range.delta(t))}deltaStart(t){return t===0?this:new e(this.seq1Range.deltaStart(t),this.seq2Range.deltaStart(t))}deltaEnd(t){return t===0?this:new e(this.seq1Range.deltaEnd(t),this.seq2Range.deltaEnd(t))}intersect(t){let n=this.seq1Range.intersect(t.seq1Range),r=this.seq2Range.intersect(t.seq2Range);if(n&&r)return new e(n,r)}getStarts(){return new X(this.seq1Range.start,this.seq2Range.start)}getEndExclusives(){return new X(this.seq1Range.endExclusive,this.seq2Range.endExclusive)}},X=class e{static{this.zero=new e(0,0)}static{this.max=new e(2**53-1,2**53-1)}constructor(e,t){this.offset1=e,this.offset2=t}toString(){return`${this.offset1} <-> ${this.offset2}`}delta(t){return t===0?this:new e(this.offset1+t,this.offset2+t)}equals(e){return this.offset1===e.offset1&&this.offset2===e.offset2}},qi=class e{static{this.instance=new e}isValid(){return!0}},Ji=class{constructor(e){if(this.timeout=e,this.startTime=Date.now(),this.valid=!0,e<=0)throw new o(`timeout must be positive`)}isValid(){return!(Date.now()-this.startTime<this.timeout)&&this.valid&&(this.valid=!1),this.valid}},Yi=class{constructor(e,t){this.width=e,this.height=t,this.array=[],this.array=Array(e*t)}get(e,t){return this.array[e+t*this.width]}set(e,t,n){this.array[e+t*this.width]=n}};function Xi(e){return e===32||e===9}var Zi=class e{static{this.chrKeys=/* @__PURE__ */ new Map}static getKey(e){let t=this.chrKeys.get(e);return t===void 0&&(t=this.chrKeys.size,this.chrKeys.set(e,t)),t}constructor(t,n,r){this.range=t,this.lines=n,this.source=r,this.histogram=[];let i=0;for(let r=t.startLineNumber-1;r<t.endLineNumberExclusive-1;r++){let t=n[r];for(let n=0;n<t.length;n++){i++;let r=t[n],a=e.getKey(r);this.histogram[a]=(this.histogram[a]||0)+1}i++;let a=e.getKey(`\n`);this.histogram[a]=(this.histogram[a]||0)+1}this.totalCount=i}computeSimilarity(e){let t=0,n=Math.max(this.histogram.length,e.histogram.length);for(let r=0;r<n;r++)t+=Math.abs((this.histogram[r]??0)-(e.histogram[r]??0));return 1-t/(this.totalCount+e.totalCount)}},Qi=class{compute(e,t,n=qi.instance,r){if(e.length===0||t.length===0)return J.trivial(e,t);let i=new Yi(e.length,t.length),a=new Yi(e.length,t.length),o=new Yi(e.length,t.length);for(let s=0;s<e.length;s++)for(let c=0;c<t.length;c++){if(!n.isValid())return J.trivialTimedOut(e,t);let l=s===0?0:i.get(s-1,c),u=c===0?0:i.get(s,c-1),d;e.getElement(s)===t.getElement(c)?(d=s===0||c===0?0:i.get(s-1,c-1),s>0&&c>0&&a.get(s-1,c-1)===3&&(d+=o.get(s-1,c-1)),d+=r?r(s,c):1):d=-1;let f=Math.max(l,u,d);if(f===d){let e=s>0&&c>0?o.get(s-1,c-1):0;o.set(s,c,e+1),a.set(s,c,3)}else f===l?(o.set(s,c,0),a.set(s,c,1)):f===u&&(o.set(s,c,0),a.set(s,c,2));i.set(s,c,f)}let s=[],c=e.length,l=t.length;function u(e,t){(e+1!==c||t+1!==l)&&s.push(new Y(new G(e+1,c),new G(t+1,l))),c=e,l=t}let d=e.length-1,f=t.length-1;for(;d>=0&&f>=0;)a.get(d,f)===3?(u(d,f),d--,f--):a.get(d,f)===1?d--:f--;return u(-1,-1),s.reverse(),new J(s,!1)}},$i=class{compute(e,t,n=qi.instance){if(e.length===0||t.length===0)return J.trivial(e,t);let r=e,i=t;function a(e,t){for(;e<r.length&&t<i.length&&r.getElement(e)===i.getElement(t);)e++,t++;return e}let o=0,s=new ta;s.set(0,a(0,0));let c=new na;c.set(0,s.get(0)===0?null:new ea(null,0,0,s.get(0)));let l=0;loop:for(;;){if(o++,!n.isValid())return J.trivialTimedOut(r,i);let e=-Math.min(o,i.length+o%2),t=Math.min(o,r.length+o%2);for(l=e;l<=t;l+=2){let n=l===t?-1:s.get(l+1),o=l===e?-1:s.get(l-1)+1,u=Math.min(Math.max(n,o),r.length),d=u-l;if(u>r.length||d>i.length)continue;let f=a(u,d);s.set(l,f);let p=u===n?c.get(l+1):c.get(l-1);if(c.set(l,f===u?p:new ea(p,u,d,f-u)),s.get(l)===r.length&&s.get(l)-l===i.length)break loop}}let u=c.get(l),d=[],f=r.length,p=i.length;for(;;){let e=u?u.x+u.length:0,t=u?u.y+u.length:0;if((e!==f||t!==p)&&d.push(new Y(new G(e,f),new G(t,p))),!u)break;f=u.x,p=u.y,u=u.prev}return d.reverse(),new J(d,!1)}},ea=class{constructor(e,t,n,r){this.prev=e,this.x=t,this.y=n,this.length=r}},ta=class{constructor(){this.positiveArr=/* @__PURE__ */ new Int32Array(10),this.negativeArr=/* @__PURE__ */ new Int32Array(10)}get(e){return e<0?(e=-e-1,this.negativeArr[e]):this.positiveArr[e]}set(e,t){if(e<0){if(e=-e-1,e>=this.negativeArr.length){let e=this.negativeArr;this.negativeArr=new Int32Array(e.length*2),this.negativeArr.set(e)}this.negativeArr[e]=t}else{if(e>=this.positiveArr.length){let e=this.positiveArr;this.positiveArr=new Int32Array(e.length*2),this.positiveArr.set(e)}this.positiveArr[e]=t}}},na=class{constructor(){this.positiveArr=[],this.negativeArr=[]}get(e){return e<0?(e=-e-1,this.negativeArr[e]):this.positiveArr[e]}set(e,t){e<0?(e=-e-1,this.negativeArr[e]=t):this.positiveArr[e]=t}},ra=class{constructor(e,t,n){this.lines=e,this.range=t,this.considerWhitespaceChanges=n,this.elements=[],this.firstElementOffsetByLineIdx=[],this.lineStartOffsets=[],this.trimmedWsLengthsByLineIdx=[],this.firstElementOffsetByLineIdx.push(0);for(let t=this.range.startLineNumber;t<=this.range.endLineNumber;t++){let r=e[t-1],i=0;t===this.range.startLineNumber&&this.range.startColumn>1&&(i=this.range.startColumn-1,r=r.substring(i)),this.lineStartOffsets.push(i);let a=0;if(!n){let e=r.trimStart();a=r.length-e.length,r=e.trimEnd()}this.trimmedWsLengthsByLineIdx.push(a);let o=t===this.range.endLineNumber?Math.min(this.range.endColumn-1-i-a,r.length):r.length;for(let e=0;e<o;e++)this.elements.push(r.charCodeAt(e));t<this.range.endLineNumber&&(this.elements.push(10),this.firstElementOffsetByLineIdx.push(this.elements.length))}}toString(){return`Slice: \"${this.text}\"`}get text(){return this.getText(new G(0,this.length))}getText(e){return this.elements.slice(e.start,e.endExclusive).map(e=>String.fromCharCode(e)).join(``)}getElement(e){return this.elements[e]}get length(){return this.elements.length}getBoundaryScore(e){let t=ca(e>0?this.elements[e-1]:-1),n=ca(e<this.elements.length?this.elements[e]:-1);if(t===7&&n===8)return 0;if(t===8)return 150;let r=0;return t!==n&&(r+=10,t===0&&n===1&&(r+=1)),r+=sa(t),r+=sa(n),r}translateOffset(e,t=`right`){let n=_i(this.firstElementOffsetByLineIdx,t=>t<=e),r=e-this.firstElementOffsetByLineIdx[n];return new P(this.range.startLineNumber+n,1+this.lineStartOffsets[n]+r+(r===0&&t===`left`?0:this.trimmedWsLengthsByLineIdx[n]))}translateRange(e){let t=this.translateOffset(e.start,`right`),n=this.translateOffset(e.endExclusive,`left`);return n.isBefore(t)?F.fromPositions(n,n):F.fromPositions(t,n)}findWordContaining(e){if(e<0||e>=this.elements.length||!ia(this.elements[e]))return;let t=e;for(;t>0&&ia(this.elements[t-1]);)t--;let n=e;for(;n<this.elements.length&&ia(this.elements[n]);)n++;return new G(t,n)}findSubWordContaining(e){if(e<0||e>=this.elements.length||!ia(this.elements[e]))return;let t=e;for(;t>0&&ia(this.elements[t-1])&&!aa(this.elements[t]);)t--;let n=e;for(;n<this.elements.length&&ia(this.elements[n])&&!aa(this.elements[n]);)n++;return new G(t,n)}countLinesIn(e){return this.translateOffset(e.endExclusive).lineNumber-this.translateOffset(e.start).lineNumber}isStronglyEqual(e,t){return this.elements[e]===this.elements[t]}extendToFullLines(e){return new G(gi(this.firstElementOffsetByLineIdx,t=>t<=e.start)??0,vi(this.firstElementOffsetByLineIdx,t=>e.endExclusive<=t)??this.elements.length)}};function ia(e){return e>=97&&e<=122||e>=65&&e<=90||e>=48&&e<=57}function aa(e){return e>=65&&e<=90}const oa={0:0,1:0,2:0,3:10,4:2,5:30,6:3,7:10,8:10};function sa(e){return oa[e]}function ca(e){return e===10?8:e===13?7:Xi(e)?6:e>=97&&e<=122?0:e>=65&&e<=90?1:e>=48&&e<=57?2:e===-1?3:e===44||e===59?5:4}function la(e,t,n,r,i,a){let{moves:o,excludedChanges:s}=da(e,t,n,a);if(!a.isValid())return[];let c=fa(e.filter(e=>!s.has(e)),r,i,t,n,a);return di(o,c),o=ma(o),o=o.filter(e=>{let n=e.original.toOffsetRange().slice(t).map(e=>e.trim());return n.join(`\n`).length>=15&&ua(n,e=>e.length>=2)>=2}),o=ha(e,o),o}function ua(e,t){let n=0;for(let r of e)t(r)&&n++;return n}function da(e,t,n,r){let i=[],a=e.filter(e=>e.modified.isEmpty&&e.original.length>=3).map(e=>new Zi(e.original,t,e)),o=new Set(e.filter(e=>e.original.isEmpty&&e.modified.length>=3).map(e=>new Zi(e.modified,n,e))),s=/* @__PURE__ */ new Set;for(let e of a){let t=-1,n;for(let r of o){let i=e.computeSimilarity(r);i>t&&(t=i,n=r)}if(t>.9&&n&&(o.delete(n),i.push(new Ai(e.range,n.range)),s.add(e.source),s.add(n.source)),!r.isValid())return{moves:i,excludedChanges:s}}return{moves:i,excludedChanges:s}}function fa(e,t,n,r,i,a){let o=[],s=new zr;for(let n of e)for(let e=n.original.startLineNumber;e<n.original.endLineNumberExclusive-2;e++){let n=`${t[e-1]}:${t[e+1-1]}:${t[e+2-1]}`;s.add(n,{range:new K(e,e+3)})}let c=[];e.sort(pi(e=>e.modified.startLineNumber,mi));for(let t of e){let e=[];for(let r=t.modified.startLineNumber;r<t.modified.endLineNumberExclusive-2;r++){let t=`${n[r-1]}:${n[r+1-1]}:${n[r+2-1]}`,i=new K(r,r+3),a=[];s.forEach(t,({range:t})=>{for(let n of e)if(n.originalLineRange.endLineNumberExclusive+1===t.endLineNumberExclusive&&n.modifiedLineRange.endLineNumberExclusive+1===i.endLineNumberExclusive){n.originalLineRange=new K(n.originalLineRange.startLineNumber,t.endLineNumberExclusive),n.modifiedLineRange=new K(n.modifiedLineRange.startLineNumber,i.endLineNumberExclusive),a.push(n);return}let n={modifiedLineRange:i,originalLineRange:t};c.push(n),a.push(n)}),e=a}if(!a.isValid())return[]}c.sort(hi(pi(e=>e.modifiedLineRange.length,mi)));let l=new xi,u=new xi;for(let e of c){let t=e.modifiedLineRange.startLineNumber-e.originalLineRange.startLineNumber,n=l.subtractFrom(e.modifiedLineRange),r=u.subtractFrom(e.originalLineRange).getWithDelta(t),i=n.getIntersection(r);for(let e of i.ranges){if(e.length<3)continue;let n=e,r=e.delta(-t);o.push(new Ai(r,n)),l.addRange(n),u.addRange(r)}}o.sort(pi(e=>e.original.startLineNumber,mi));let d=new bi(e);for(let t=0;t<o.length;t++){let n=o[t],s=d.findLastMonotonous(e=>e.original.startLineNumber<=n.original.startLineNumber),c=gi(e,e=>e.modified.startLineNumber<=n.modified.startLineNumber),f=Math.max(n.original.startLineNumber-s.original.startLineNumber,n.modified.startLineNumber-c.modified.startLineNumber),p=d.findLastMonotonous(e=>e.original.startLineNumber<n.original.endLineNumberExclusive),m=gi(e,e=>e.modified.startLineNumber<n.modified.endLineNumberExclusive),h=Math.max(p.original.endLineNumberExclusive-n.original.endLineNumberExclusive,m.modified.endLineNumberExclusive-n.modified.endLineNumberExclusive),g=0;for(;g<f;g++){let e=n.original.startLineNumber-g-1,t=n.modified.startLineNumber-g-1;if(e>r.length||t>i.length||l.contains(t)||u.contains(e)||!pa(r[e-1],i[t-1],a))break}g>0&&(u.addRange(new K(n.original.startLineNumber-g,n.original.startLineNumber)),l.addRange(new K(n.modified.startLineNumber-g,n.modified.startLineNumber)));let _=0;for(;_<h;_++){let e=n.original.endLineNumberExclusive+_,t=n.modified.endLineNumberExclusive+_;if(e>r.length||t>i.length||l.contains(t)||u.contains(e)||!pa(r[e-1],i[t-1],a))break}_>0&&(u.addRange(new K(n.original.endLineNumberExclusive,n.original.endLineNumberExclusive+_)),l.addRange(new K(n.modified.endLineNumberExclusive,n.modified.endLineNumberExclusive+_))),(g>0||_>0)&&(o[t]=new Ai(new K(n.original.startLineNumber-g,n.original.endLineNumberExclusive+_),new K(n.modified.startLineNumber-g,n.modified.endLineNumberExclusive+_)))}return o}function pa(e,t,n){if(e.trim()===t.trim())return!0;if(e.length>300&&t.length>300)return!1;let r=new $i().compute(new ra([e],new F(1,1,1,e.length),!1),new ra([t],new F(1,1,1,t.length),!1),n),i=0,a=Y.invert(r.diffs,e.length);for(let t of a)t.seq1Range.forEach(t=>{Xi(e.charCodeAt(t))||i++});function o(t){let n=0;for(let r=0;r<e.length;r++)Xi(t.charCodeAt(r))||n++;return n}let s=o(e.length>t.length?e:t);return i/s>.6&&s>10}function ma(e){if(e.length===0)return e;e.sort(pi(e=>e.original.startLineNumber,mi));let t=[e[0]];for(let n=1;n<e.length;n++){let r=t[t.length-1],i=e[n],a=i.original.startLineNumber-r.original.endLineNumberExclusive,o=i.modified.startLineNumber-r.modified.endLineNumberExclusive;a>=0&&o>=0&&a+o<=2?t[t.length-1]=r.join(i):t.push(i)}return t}function ha(e,t){let n=new bi(e);return t=t.filter(t=>(n.findLastMonotonous(e=>e.original.startLineNumber<t.original.endLineNumberExclusive)||new Ai(new K(1,1),new K(1,1)))!==gi(e,e=>e.modified.startLineNumber<t.modified.endLineNumberExclusive)),t}function ga(e,t,n){let r=n;return r=_a(e,t,r),r=_a(e,t,r),r=va(e,t,r),r}function _a(e,t,n){if(n.length===0)return n;let r=[];r.push(n[0]);for(let i=1;i<n.length;i++){let a=r[r.length-1],o=n[i];if(o.seq1Range.isEmpty||o.seq2Range.isEmpty){let n=o.seq1Range.start-a.seq1Range.endExclusive,i=1;for(;i<=n&&e.getElement(o.seq1Range.start-i)===e.getElement(o.seq1Range.endExclusive-i)&&t.getElement(o.seq2Range.start-i)===t.getElement(o.seq2Range.endExclusive-i);i++);if(i--,i===n){r[r.length-1]=new Y(new G(a.seq1Range.start,o.seq1Range.endExclusive-n),new G(a.seq2Range.start,o.seq2Range.endExclusive-n));continue}o=o.delta(-i)}r.push(o)}let i=[];for(let n=0;n<r.length-1;n++){let a=r[n+1],o=r[n];if(o.seq1Range.isEmpty||o.seq2Range.isEmpty){let i=a.seq1Range.start-o.seq1Range.endExclusive,s=0;for(;s<i&&e.isStronglyEqual(o.seq1Range.start+s,o.seq1Range.endExclusive+s)&&t.isStronglyEqual(o.seq2Range.start+s,o.seq2Range.endExclusive+s);s++);if(s===i){r[n+1]=new Y(new G(o.seq1Range.start+i,a.seq1Range.endExclusive),new G(o.seq2Range.start+i,a.seq2Range.endExclusive));continue}s>0&&(o=o.delta(s))}i.push(o)}return r.length>0&&i.push(r[r.length-1]),i}function va(e,t,n){if(!e.getBoundaryScore||!t.getBoundaryScore)return n;for(let r=0;r<n.length;r++){let i=r>0?n[r-1]:void 0,a=n[r],o=r+1<n.length?n[r+1]:void 0,s=new G(i?i.seq1Range.endExclusive+1:0,o?o.seq1Range.start-1:e.length),c=new G(i?i.seq2Range.endExclusive+1:0,o?o.seq2Range.start-1:t.length);a.seq1Range.isEmpty?n[r]=ya(a,e,t,s,c):a.seq2Range.isEmpty&&(n[r]=ya(a.swap(),t,e,c,s).swap())}return n}function ya(e,t,n,r,i){let a=1;for(;e.seq1Range.start-a>=r.start&&e.seq2Range.start-a>=i.start&&n.isStronglyEqual(e.seq2Range.start-a,e.seq2Range.endExclusive-a)&&a<100;)a++;a--;let o=0;for(;e.seq1Range.start+o<r.endExclusive&&e.seq2Range.endExclusive+o<i.endExclusive&&n.isStronglyEqual(e.seq2Range.start+o,e.seq2Range.endExclusive+o)&&o<100;)o++;if(a===0&&o===0)return e;let s=0,c=-1;for(let r=-a;r<=o;r++){let i=e.seq2Range.start+r,a=e.seq2Range.endExclusive+r,o=e.seq1Range.start+r,l=t.getBoundaryScore(o)+n.getBoundaryScore(i)+n.getBoundaryScore(a);l>c&&(c=l,s=r)}return e.delta(s)}function ba(e,t,n){let r=[];for(let e of n){let t=r[r.length-1];t&&(e.seq1Range.start-t.seq1Range.endExclusive<=2||e.seq2Range.start-t.seq2Range.endExclusive<=2)?r[r.length-1]=new Y(t.seq1Range.join(e.seq1Range),t.seq2Range.join(e.seq2Range)):r.push(e)}return r}function xa(e,t,n,r,i=!1){let a=Y.invert(n,e.length),o=[],s=new X(0,0);function c(n,c){if(n.offset1<s.offset1||n.offset2<s.offset2)return;let l=r(e,n.offset1),u=r(t,n.offset2);if(!l||!u)return;let d=new Y(l,u),f=d.intersect(c),p=f.seq1Range.length,m=f.seq2Range.length;for(;a.length>0;){let n=a[0];if(!(n.seq1Range.intersects(d.seq1Range)||n.seq2Range.intersects(d.seq2Range)))break;let i=new Y(r(e,n.seq1Range.start),r(t,n.seq2Range.start)),o=i.intersect(n);if(p+=o.seq1Range.length,m+=o.seq2Range.length,d=d.join(i),d.seq1Range.endExclusive>=n.seq1Range.endExclusive)a.shift();else break}(i&&p+m<d.seq1Range.length+d.seq2Range.length||p+m<(d.seq1Range.length+d.seq2Range.length)*2/3)&&o.push(d),s=d.getEndExclusives()}for(;a.length>0;){let e=a.shift();e.seq1Range.isEmpty||(c(e.getStarts(),e),c(e.getEndExclusives().delta(-1),e))}return Sa(n,o)}function Sa(e,t){let n=[];for(;e.length>0||t.length>0;){let r=e[0],i=t[0],a;a=r&&(!i||r.seq1Range.start<i.seq1Range.start)?e.shift():t.shift(),n.length>0&&n[n.length-1].seq1Range.endExclusive>=a.seq1Range.start?n[n.length-1]=n[n.length-1].join(a):n.push(a)}return n}function Ca(e,t,n){let r=n;if(r.length===0)return r;let i=0,a;do{a=!1;let t=[r[0]];for(let n=1;n<r.length;n++){let i=r[n],o=t[t.length-1];function s(t,n){let r=new G(o.seq1Range.endExclusive,i.seq1Range.start);return e.getText(r).replace(/\\s/g,``).length<=4&&(t.seq1Range.length+t.seq2Range.length>5||n.seq1Range.length+n.seq2Range.length>5)}s(o,i)?(a=!0,t[t.length-1]=t[t.length-1].join(i)):t.push(i)}r=t}while(i++<10&&a);return r}function wa(e,t,n){let r=n;if(r.length===0)return r;let i=0,a;do{a=!1;let n=[r[0]];for(let i=1;i<r.length;i++){let o=r[i],s=n[n.length-1];function c(n,r){let i=new G(s.seq1Range.endExclusive,o.seq1Range.start);if(e.countLinesIn(i)>5||i.length>500)return!1;let a=e.getText(i).trim();if(a.length>20||a.split(/\\r\\n|\\r|\\n/).length>1)return!1;let c=e.countLinesIn(n.seq1Range),l=n.seq1Range.length,u=t.countLinesIn(n.seq2Range),d=n.seq2Range.length,f=e.countLinesIn(r.seq1Range),p=r.seq1Range.length,m=t.countLinesIn(r.seq2Range),h=r.seq2Range.length;function g(e){return Math.min(e,130)}return(g(c*40+l)**1.5+g(u*40+d)**1.5)**1.5+(g(f*40+p)**1.5+g(m*40+h)**1.5)**1.5>(130**1.5)**1.5*1.3}c(s,o)?(a=!0,n[n.length-1]=n[n.length-1].join(o)):n.push(o)}r=n}while(i++<10&&a);let o=[];return ui(r,(t,n,r)=>{let i=n;function a(e){return e.length>0&&e.trim().length<=3&&n.seq1Range.length+n.seq2Range.length>100}let s=e.extendToFullLines(n.seq1Range),c=e.getText(new G(s.start,n.seq1Range.start));a(c)&&(i=i.deltaStart(-c.length));let l=e.getText(new G(n.seq1Range.endExclusive,s.endExclusive));a(l)&&(i=i.deltaEnd(l.length));let u=Y.fromOffsetPairs(t?t.getEndExclusives():X.zero,r?r.getStarts():X.max),d=i.intersect(u);o.length>0&&d.getStarts().equals(o[o.length-1].getEndExclusives())?o[o.length-1]=o[o.length-1].join(d):o.push(d)}),o}var Ta=class{constructor(e,t){this.trimmedHash=e,this.lines=t}getElement(e){return this.trimmedHash[e]}get length(){return this.trimmedHash.length}getBoundaryScore(e){return 1e3-((e===0?0:Ea(this.lines[e-1]))+(e===this.lines.length?0:Ea(this.lines[e])))}getText(e){return this.lines.slice(e.start,e.endExclusive).join(`\n`)}isStronglyEqual(e,t){return this.lines[e]===this.lines[t]}};function Ea(e){let t=0;for(;t<e.length&&(e.charCodeAt(t)===32||e.charCodeAt(t)===9);)t++;return t}var Da=class{constructor(){this.dynamicProgrammingDiffing=new Qi,this.myersDiffingAlgorithm=new $i}computeDiff(e,t,n){if(e.length<=1&&si(e,t,(e,t)=>e===t))return new ai([],[],!1);if(e.length===1&&e[0].length===0||t.length===1&&t[0].length===0)return new ai([new Ni(new K(1,e.length+1),new K(1,t.length+1),[new q(new F(1,1,e.length,e[e.length-1].length+1),new F(1,1,t.length,t[t.length-1].length+1))])],[],!1);let r=n.maxComputationTimeMs===0?qi.instance:new Ji(n.maxComputationTimeMs),i=!n.ignoreTrimWhitespace,a=/* @__PURE__ */ new Map;function o(e){let t=a.get(e);return t===void 0&&(t=a.size,a.set(e,t)),t}let s=e.map(e=>o(e.trim())),c=t.map(e=>o(e.trim())),u=new Ta(s,e),d=new Ta(c,t),f=u.length+d.length<1700?this.dynamicProgrammingDiffing.compute(u,d,r,(n,r)=>e[n]===t[r]?t[r].length===0?.1:1+Math.log(1+t[r].length):.99):this.myersDiffingAlgorithm.compute(u,d,r),p=f.diffs,m=f.hitTimeout;p=ga(u,d,p),p=Ca(u,d,p);let h=[],g=a=>{if(i)for(let o=0;o<a;o++){let a=_+o,s=v+o;if(e[a]!==t[s]){let o=this.refineDiff(e,t,new Y(new G(a,a+1),new G(s,s+1)),r,i,n);for(let e of o.mappings)h.push(e);o.hitTimeout&&(m=!0)}}},_=0,v=0;for(let a of p){l(()=>a.seq1Range.start-_===a.seq2Range.start-v),g(a.seq1Range.start-_),_=a.seq1Range.endExclusive,v=a.seq2Range.endExclusive;let o=this.refineDiff(e,t,a,r,i,n);o.hitTimeout&&(m=!0);for(let e of o.mappings)h.push(e)}g(e.length-_);let y=Pi(h,new Oi(e),new Oi(t)),b=[];return n.computeMoves&&(b=this.computeMoves(y,e,t,s,c,r,i,n)),l(()=>{function n(e,t){if(e.lineNumber<1||e.lineNumber>t.length)return!1;let n=t[e.lineNumber-1];return!(e.column<1||e.column>n.length+1)}function r(e,t){return!(e.startLineNumber<1||e.startLineNumber>t.length+1||e.endLineNumberExclusive<1||e.endLineNumberExclusive>t.length+1)}for(let i of y){if(!i.innerChanges)return!1;for(let r of i.innerChanges)if(!(n(r.modifiedRange.getStartPosition(),t)&&n(r.modifiedRange.getEndPosition(),t)&&n(r.originalRange.getStartPosition(),e)&&n(r.originalRange.getEndPosition(),e)))return!1;if(!r(i.modified,t)||!r(i.original,e))return!1}return!0}),new ai(y,b,m)}computeMoves(e,t,n,r,i,a,o,s){return la(e,t,n,r,i,a).map(e=>new oi(e,Pi(this.refineDiff(t,n,new Y(e.original.toOffsetRange(),e.modified.toOffsetRange()),a,o,s).mappings,new Oi(t),new Oi(n),!0)))}refineDiff(e,t,n,r,i,a){let o=Oa(n).toRangeMapping2(e,t),s=new ra(e,o.originalRange,i),c=new ra(t,o.modifiedRange,i),l=s.length+c.length<500?this.dynamicProgrammingDiffing.compute(s,c,r):this.myersDiffingAlgorithm.compute(s,c,r),u=l.diffs;return u=ga(s,c,u),u=xa(s,c,u,(e,t)=>e.findWordContaining(t)),a.extendToSubwords&&(u=xa(s,c,u,(e,t)=>e.findSubWordContaining(t),!0)),u=ba(s,c,u),u=wa(s,c,u),{mappings:u.map(e=>new q(s.translateRange(e.seq1Range),c.translateRange(e.seq2Range))),hitTimeout:l.hitTimeout}}};function Oa(e){return new Ai(new K(e.seq1Range.start+1,e.seq1Range.endExclusive+1),new K(e.seq2Range.start+1,e.seq2Range.endExclusive+1))}const ka={getLegacy:()=>new Ii,getDefault:()=>new Da};function Z(e,t){let n=10**t;return Math.round(e*n)/n}var Q=class{constructor(e,t,n,r=1){this._rgbaBrand=void 0,this.r=Math.min(255,Math.max(0,e))|0,this.g=Math.min(255,Math.max(0,t))|0,this.b=Math.min(255,Math.max(0,n))|0,this.a=Z(Math.max(Math.min(1,r),0),3)}static equals(e,t){return e.r===t.r&&e.g===t.g&&e.b===t.b&&e.a===t.a}},$=class e{constructor(e,t,n,r){this._hslaBrand=void 0,this.h=Math.max(Math.min(360,e),0)|0,this.s=Z(Math.max(Math.min(1,t),0),3),this.l=Z(Math.max(Math.min(1,n),0),3),this.a=Z(Math.max(Math.min(1,r),0),3)}static equals(e,t){return e.h===t.h&&e.s===t.s&&e.l===t.l&&e.a===t.a}static fromRGBA(t){let n=t.r/255,r=t.g/255,i=t.b/255,a=t.a,o=Math.max(n,r,i),s=Math.min(n,r,i),c=0,l=0,u=(s+o)/2,d=o-s;if(d>0){switch(l=Math.min(u<=.5?d/(2*u):d/(2-2*u),1),o){case n:c=(r-i)/d+(r<i?6:0);break;case r:c=(i-n)/d+2;break;case i:c=(n-r)/d+4}c*=60,c=Math.round(c)}return new e(c,l,u,a)}static _hue2rgb(e,t,n){return n<0&&(n+=1),n>1&&--n,n<1/6?e+(t-e)*6*n:n<1/2?t:n<2/3?e+(t-e)*(2/3-n)*6:e}static toRGBA(t){let n=t.h/360,{s:r,l:i,a}=t,o,s,c;if(r===0)o=s=c=i;else{let t=i<.5?i*(1+r):i+r-i*r,a=2*i-t;o=e._hue2rgb(a,t,n+1/3),s=e._hue2rgb(a,t,n),c=e._hue2rgb(a,t,n-1/3)}return new Q(Math.round(o*255),Math.round(s*255),Math.round(c*255),a)}},Aa=class e{constructor(e,t,n,r){this._hsvaBrand=void 0,this.h=Math.max(Math.min(360,e),0)|0,this.s=Z(Math.max(Math.min(1,t),0),3),this.v=Z(Math.max(Math.min(1,n),0),3),this.a=Z(Math.max(Math.min(1,r),0),3)}static equals(e,t){return e.h===t.h&&e.s===t.s&&e.v===t.v&&e.a===t.a}static fromRGBA(t){let n=t.r/255,r=t.g/255,i=t.b/255,a=Math.max(n,r,i),o=a-Math.min(n,r,i),s=a===0?0:o/a,c;return c=o===0?0:a===n?((r-i)/o%6+6)%6:a===r?(i-n)/o+2:(n-r)/o+4,new e(Math.round(c*60),s,a,t.a)}static toRGBA(e){let{h:t,s:n,v:r,a:i}=e,a=r*n,o=a*(1-Math.abs(t/60%2-1)),s=r-a,[c,l,u]=[0,0,0];return t<60?(c=a,l=o):t<120?(c=o,l=a):t<180?(l=a,u=o):t<240?(l=o,u=a):t<300?(c=o,u=a):t<=360&&(c=a,u=o),c=Math.round((c+s)*255),l=Math.round((l+s)*255),u=Math.round((u+s)*255),new Q(c,l,u,i)}},ja=class e{static fromHex(t){return e.Format.CSS.parseHex(t)||e.red}static equals(e,t){return!e&&!t?!0:!e||!t?!1:e.equals(t)}get hsla(){return this._hsla?this._hsla:$.fromRGBA(this.rgba)}get hsva(){return this._hsva?this._hsva:Aa.fromRGBA(this.rgba)}constructor(e){if(!e)throw Error(`Color needs a value`);if(e instanceof Q)this.rgba=e;else if(e instanceof $)this._hsla=e,this.rgba=$.toRGBA(e);else if(e instanceof Aa)this._hsva=e,this.rgba=Aa.toRGBA(e);else throw Error(`Invalid color ctor argument`)}equals(e){return!!e&&Q.equals(this.rgba,e.rgba)&&$.equals(this.hsla,e.hsla)&&Aa.equals(this.hsva,e.hsva)}getRelativeLuminance(){let t=e._relativeLuminanceForComponent(this.rgba.r),n=e._relativeLuminanceForComponent(this.rgba.g),r=e._relativeLuminanceForComponent(this.rgba.b);return Z(.2126*t+.7152*n+.0722*r,4)}static _relativeLuminanceForComponent(e){let t=e/255;return t<=.03928?t/12.92:((t+.055)/1.055)**2.4}isLighter(){return(this.rgba.r*299+this.rgba.g*587+this.rgba.b*114)/1e3>=128}isLighterThan(e){return this.getRelativeLuminance()>e.getRelativeLuminance()}isDarkerThan(e){return this.getRelativeLuminance()<e.getRelativeLuminance()}lighten(t){return new e(new $(this.hsla.h,this.hsla.s,this.hsla.l+this.hsla.l*t,this.hsla.a))}darken(t){return new e(new $(this.hsla.h,this.hsla.s,this.hsla.l-this.hsla.l*t,this.hsla.a))}transparent(t){let{r:n,g:r,b:i,a}=this.rgba;return new e(new Q(n,r,i,a*t))}isTransparent(){return this.rgba.a===0}isOpaque(){return this.rgba.a===1}opposite(){return new e(new Q(255-this.rgba.r,255-this.rgba.g,255-this.rgba.b,this.rgba.a))}mix(t,n=.5){let r=Math.min(Math.max(n,0),1),i=this.rgba,a=t.rgba,o=i.r+(a.r-i.r)*r,s=i.g+(a.g-i.g)*r,c=i.b+(a.b-i.b)*r,l=i.a+(a.a-i.a)*r;return new e(new Q(o,s,c,l))}makeOpaque(t){if(this.isOpaque()||t.rgba.a!==1)return this;let{r:n,g:r,b:i,a}=this.rgba;return new e(new Q(t.rgba.r-a*(t.rgba.r-n),t.rgba.g-a*(t.rgba.g-r),t.rgba.b-a*(t.rgba.b-i),1))}toString(){return this._toString||=e.Format.CSS.format(this),this._toString}toNumber32Bit(){return this._toNumber32Bit||=(this.rgba.r<<24|this.rgba.g<<16|this.rgba.b<<8|this.rgba.a*255)>>>0,this._toNumber32Bit}static getLighterColor(e,t,n){if(e.isLighterThan(t))return e;n||=.5;let r=e.getRelativeLuminance(),i=t.getRelativeLuminance();return n=n*(i-r)/i,e.lighten(n)}static getDarkerColor(e,t,n){if(e.isDarkerThan(t))return e;n||=.5;let r=e.getRelativeLuminance(),i=t.getRelativeLuminance();return n=n*(r-i)/r,e.darken(n)}static{this.white=new e(new Q(255,255,255,1))}static{this.black=new e(new Q(0,0,0,1))}static{this.red=new e(new Q(255,0,0,1))}static{this.blue=new e(new Q(0,0,255,1))}static{this.green=new e(new Q(0,255,0,1))}static{this.cyan=new e(new Q(0,255,255,1))}static{this.lightgrey=new e(new Q(211,211,211,1))}static{this.transparent=new e(new Q(0,0,0,0))}};(function(e){(function(t){(function(t){function n(t){return t.rgba.a===1?`rgb(${t.rgba.r}, ${t.rgba.g}, ${t.rgba.b})`:e.Format.CSS.formatRGBA(t)}t.formatRGB=n;function r(e){return`rgba(${e.rgba.r}, ${e.rgba.g}, ${e.rgba.b}, ${+e.rgba.a.toFixed(2)})`}t.formatRGBA=r;function i(t){return t.hsla.a===1?`hsl(${t.hsla.h}, ${Math.round(t.hsla.s*100)}%, ${Math.round(t.hsla.l*100)}%)`:e.Format.CSS.formatHSLA(t)}t.formatHSL=i;function a(e){return`hsla(${e.hsla.h}, ${Math.round(e.hsla.s*100)}%, ${Math.round(e.hsla.l*100)}%, ${e.hsla.a.toFixed(2)})`}t.formatHSLA=a;function o(e){let t=e.toString(16);return t.length===2?t:`0`+t}function s(e){return`#${o(e.rgba.r)}${o(e.rgba.g)}${o(e.rgba.b)}`}t.formatHex=s;function c(t,n=!1){return n&&t.rgba.a===1?e.Format.CSS.formatHex(t):`#${o(t.rgba.r)}${o(t.rgba.g)}${o(t.rgba.b)}${o(Math.round(t.rgba.a*255))}`}t.formatHexA=c;function l(t){return t.isOpaque()?e.Format.CSS.formatHex(t):e.Format.CSS.formatRGBA(t)}t.format=l;function u(t){if(t===`transparent`)return e.transparent;if(t.startsWith(`#`))return f(t);if(t.startsWith(`rgba(`)){let n=t.match(/rgba\\((?<r>(?:\\+|-)?\\d+), *(?<g>(?:\\+|-)?\\d+), *(?<b>(?:\\+|-)?\\d+), *(?<a>(?:\\+|-)?\\d+(\\.\\d+)?)\\)/);if(!n)throw Error(`Invalid color format `+t);return new e(new Q(parseInt(n.groups?.r??`0`),parseInt(n.groups?.g??`0`),parseInt(n.groups?.b??`0`),parseFloat(n.groups?.a??`0`)))}if(t.startsWith(`rgb(`)){let n=t.match(/rgb\\((?<r>(?:\\+|-)?\\d+), *(?<g>(?:\\+|-)?\\d+), *(?<b>(?:\\+|-)?\\d+)\\)/);if(!n)throw Error(`Invalid color format `+t);return new e(new Q(parseInt(n.groups?.r??`0`),parseInt(n.groups?.g??`0`),parseInt(n.groups?.b??`0`)))}return d(t)}t.parse=u;function d(t){switch(t){case`aliceblue`:return new e(new Q(240,248,255,1));case`antiquewhite`:return new e(new Q(250,235,215,1));case`aqua`:return new e(new Q(0,255,255,1));case`aquamarine`:return new e(new Q(127,255,212,1));case`azure`:return new e(new Q(240,255,255,1));case`beige`:return new e(new Q(245,245,220,1));case`bisque`:return new e(new Q(255,228,196,1));case`black`:return new e(new Q(0,0,0,1));case`blanchedalmond`:return new e(new Q(255,235,205,1));case`blue`:return new e(new Q(0,0,255,1));case`blueviolet`:return new e(new Q(138,43,226,1));case`brown`:return new e(new Q(165,42,42,1));case`burlywood`:return new e(new Q(222,184,135,1));case`cadetblue`:return new e(new Q(95,158,160,1));case`chartreuse`:return new e(new Q(127,255,0,1));case`chocolate`:return new e(new Q(210,105,30,1));case`coral`:return new e(new Q(255,127,80,1));case`cornflowerblue`:return new e(new Q(100,149,237,1));case`cornsilk`:return new e(new Q(255,248,220,1));case`crimson`:return new e(new Q(220,20,60,1));case`cyan`:return new e(new Q(0,255,255,1));case`darkblue`:return new e(new Q(0,0,139,1));case`darkcyan`:return new e(new Q(0,139,139,1));case`darkgoldenrod`:return new e(new Q(184,134,11,1));case`darkgray`:return new e(new Q(169,169,169,1));case`darkgreen`:return new e(new Q(0,100,0,1));case`darkgrey`:return new e(new Q(169,169,169,1));case`darkkhaki`:return new e(new Q(189,183,107,1));case`darkmagenta`:return new e(new Q(139,0,139,1));case`darkolivegreen`:return new e(new Q(85,107,47,1));case`darkorange`:return new e(new Q(255,140,0,1));case`darkorchid`:return new e(new Q(153,50,204,1));case`darkred`:return new e(new Q(139,0,0,1));case`darksalmon`:return new e(new Q(233,150,122,1));case`darkseagreen`:return new e(new Q(143,188,143,1));case`darkslateblue`:return new e(new Q(72,61,139,1));case`darkslategray`:return new e(new Q(47,79,79,1));case`darkslategrey`:return new e(new Q(47,79,79,1));case`darkturquoise`:return new e(new Q(0,206,209,1));case`darkviolet`:return new e(new Q(148,0,211,1));case`deeppink`:return new e(new Q(255,20,147,1));case`deepskyblue`:return new e(new Q(0,191,255,1));case`dimgray`:return new e(new Q(105,105,105,1));case`dimgrey`:return new e(new Q(105,105,105,1));case`dodgerblue`:return new e(new Q(30,144,255,1));case`firebrick`:return new e(new Q(178,34,34,1));case`floralwhite`:return new e(new Q(255,250,240,1));case`forestgreen`:return new e(new Q(34,139,34,1));case`fuchsia`:return new e(new Q(255,0,255,1));case`gainsboro`:return new e(new Q(220,220,220,1));case`ghostwhite`:return new e(new Q(248,248,255,1));case`gold`:return new e(new Q(255,215,0,1));case`goldenrod`:return new e(new Q(218,165,32,1));case`gray`:return new e(new Q(128,128,128,1));case`green`:return new e(new Q(0,128,0,1));case`greenyellow`:return new e(new Q(173,255,47,1));case`grey`:return new e(new Q(128,128,128,1));case`honeydew`:return new e(new Q(240,255,240,1));case`hotpink`:return new e(new Q(255,105,180,1));case`indianred`:return new e(new Q(205,92,92,1));case`indigo`:return new e(new Q(75,0,130,1));case`ivory`:return new e(new Q(255,255,240,1));case`khaki`:return new e(new Q(240,230,140,1));case`lavender`:return new e(new Q(230,230,250,1));case`lavenderblush`:return new e(new Q(255,240,245,1));case`lawngreen`:return new e(new Q(124,252,0,1));case`lemonchiffon`:return new e(new Q(255,250,205,1));case`lightblue`:return new e(new Q(173,216,230,1));case`lightcoral`:return new e(new Q(240,128,128,1));case`lightcyan`:return new e(new Q(224,255,255,1));case`lightgoldenrodyellow`:return new e(new Q(250,250,210,1));case`lightgray`:return new e(new Q(211,211,211,1));case`lightgreen`:return new e(new Q(144,238,144,1));case`lightgrey`:return new e(new Q(211,211,211,1));case`lightpink`:return new e(new Q(255,182,193,1));case`lightsalmon`:return new e(new Q(255,160,122,1));case`lightseagreen`:return new e(new Q(32,178,170,1));case`lightskyblue`:return new e(new Q(135,206,250,1));case`lightslategray`:return new e(new Q(119,136,153,1));case`lightslategrey`:return new e(new Q(119,136,153,1));case`lightsteelblue`:return new e(new Q(176,196,222,1));case`lightyellow`:return new e(new Q(255,255,224,1));case`lime`:return new e(new Q(0,255,0,1));case`limegreen`:return new e(new Q(50,205,50,1));case`linen`:return new e(new Q(250,240,230,1));case`magenta`:return new e(new Q(255,0,255,1));case`maroon`:return new e(new Q(128,0,0,1));case`mediumaquamarine`:return new e(new Q(102,205,170,1));case`mediumblue`:return new e(new Q(0,0,205,1));case`mediumorchid`:return new e(new Q(186,85,211,1));case`mediumpurple`:return new e(new Q(147,112,219,1));case`mediumseagreen`:return new e(new Q(60,179,113,1));case`mediumslateblue`:return new e(new Q(123,104,238,1));case`mediumspringgreen`:return new e(new Q(0,250,154,1));case`mediumturquoise`:return new e(new Q(72,209,204,1));case`mediumvioletred`:return new e(new Q(199,21,133,1));case`midnightblue`:return new e(new Q(25,25,112,1));case`mintcream`:return new e(new Q(245,255,250,1));case`mistyrose`:return new e(new Q(255,228,225,1));case`moccasin`:return new e(new Q(255,228,181,1));case`navajowhite`:return new e(new Q(255,222,173,1));case`navy`:return new e(new Q(0,0,128,1));case`oldlace`:return new e(new Q(253,245,230,1));case`olive`:return new e(new Q(128,128,0,1));case`olivedrab`:return new e(new Q(107,142,35,1));case`orange`:return new e(new Q(255,165,0,1));case`orangered`:return new e(new Q(255,69,0,1));case`orchid`:return new e(new Q(218,112,214,1));case`palegoldenrod`:return new e(new Q(238,232,170,1));case`palegreen`:return new e(new Q(152,251,152,1));case`paleturquoise`:return new e(new Q(175,238,238,1));case`palevioletred`:return new e(new Q(219,112,147,1));case`papayawhip`:return new e(new Q(255,239,213,1));case`peachpuff`:return new e(new Q(255,218,185,1));case`peru`:return new e(new Q(205,133,63,1));case`pink`:return new e(new Q(255,192,203,1));case`plum`:return new e(new Q(221,160,221,1));case`powderblue`:return new e(new Q(176,224,230,1));case`purple`:return new e(new Q(128,0,128,1));case`rebeccapurple`:return new e(new Q(102,51,153,1));case`red`:return new e(new Q(255,0,0,1));case`rosybrown`:return new e(new Q(188,143,143,1));case`royalblue`:return new e(new Q(65,105,225,1));case`saddlebrown`:return new e(new Q(139,69,19,1));case`salmon`:return new e(new Q(250,128,114,1));case`sandybrown`:return new e(new Q(244,164,96,1));case`seagreen`:return new e(new Q(46,139,87,1));case`seashell`:return new e(new Q(255,245,238,1));case`sienna`:return new e(new Q(160,82,45,1));case`silver`:return new e(new Q(192,192,192,1));case`skyblue`:return new e(new Q(135,206,235,1));case`slateblue`:return new e(new Q(106,90,205,1));case`slategray`:return new e(new Q(112,128,144,1));case`slategrey`:return new e(new Q(112,128,144,1));case`snow`:return new e(new Q(255,250,250,1));case`springgreen`:return new e(new Q(0,255,127,1));case`steelblue`:return new e(new Q(70,130,180,1));case`tan`:return new e(new Q(210,180,140,1));case`teal`:return new e(new Q(0,128,128,1));case`thistle`:return new e(new Q(216,191,216,1));case`tomato`:return new e(new Q(255,99,71,1));case`turquoise`:return new e(new Q(64,224,208,1));case`violet`:return new e(new Q(238,130,238,1));case`wheat`:return new e(new Q(245,222,179,1));case`white`:return new e(new Q(255,255,255,1));case`whitesmoke`:return new e(new Q(245,245,245,1));case`yellow`:return new e(new Q(255,255,0,1));case`yellowgreen`:return new e(new Q(154,205,50,1));default:return null}}function f(t){let n=t.length;if(n===0||t.charCodeAt(0)!==35)return null;if(n===7)return new e(new Q(16*p(t.charCodeAt(1))+p(t.charCodeAt(2)),16*p(t.charCodeAt(3))+p(t.charCodeAt(4)),16*p(t.charCodeAt(5))+p(t.charCodeAt(6)),1));if(n===9)return new e(new Q(16*p(t.charCodeAt(1))+p(t.charCodeAt(2)),16*p(t.charCodeAt(3))+p(t.charCodeAt(4)),16*p(t.charCodeAt(5))+p(t.charCodeAt(6)),(16*p(t.charCodeAt(7))+p(t.charCodeAt(8)))/255));if(n===4){let n=p(t.charCodeAt(1)),r=p(t.charCodeAt(2)),i=p(t.charCodeAt(3));return new e(new Q(16*n+n,16*r+r,16*i+i))}if(n===5){let n=p(t.charCodeAt(1)),r=p(t.charCodeAt(2)),i=p(t.charCodeAt(3)),a=p(t.charCodeAt(4));return new e(new Q(16*n+n,16*r+r,16*i+i,(16*a+a)/255))}return null}t.parseHex=f;function p(e){switch(e){case 48:return 0;case 49:return 1;case 50:return 2;case 51:return 3;case 52:return 4;case 53:return 5;case 54:return 6;case 55:return 7;case 56:return 8;case 57:return 9;case 97:return 10;case 65:return 10;case 98:return 11;case 66:return 11;case 99:return 12;case 67:return 12;case 100:return 13;case 68:return 13;case 101:return 14;case 69:return 14;case 102:return 15;case 70:return 15}return 0}})(t.CSS||={})})(e.Format||={})})(ja||={});function Ma(e){let t=[];for(let n of e){let e=Number(n);(e||e===0&&n.replace(/\\s/g,``)!==``)&&t.push(e)}return t}function Na(e,t,n,r){return{red:e/255,blue:n/255,green:t/255,alpha:r}}function Pa(e,t){let n=t.index,r=t[0].length;if(n===void 0)return;let i=e.positionAt(n);return{startLineNumber:i.lineNumber,startColumn:i.column,endLineNumber:i.lineNumber,endColumn:i.column+r}}function Fa(e,t){if(!e)return;let n=ja.Format.CSS.parseHex(t);if(n)return{range:e,color:Na(n.rgba.r,n.rgba.g,n.rgba.b,n.rgba.a)}}function Ia(e,t,n){if(!e||t.length!==1)return;let r=Ma(t[0].values());return{range:e,color:Na(r[0],r[1],r[2],n?r[3]:1)}}function La(e,t,n){if(!e||t.length!==1)return;let r=Ma(t[0].values()),i=new ja(new $(r[0],r[1]/100,r[2]/100,n?r[3]:1));return{range:e,color:Na(i.rgba.r,i.rgba.g,i.rgba.b,i.rgba.a)}}function Ra(e,t){return typeof e==`string`?[...e.matchAll(t)]:e.findMatches(t)}function za(e){let t=[],n=Ra(e,/\\b(rgb|rgba|hsl|hsla)(\\([0-9\\s,.\\%]*\\))|^(#)([A-Fa-f0-9]{3})\\b|^(#)([A-Fa-f0-9]{4})\\b|^(#)([A-Fa-f0-9]{6})\\b|^(#)([A-Fa-f0-9]{8})\\b|(?<=['\"\\s])(#)([A-Fa-f0-9]{3})\\b|(?<=['\"\\s])(#)([A-Fa-f0-9]{4})\\b|(?<=['\"\\s])(#)([A-Fa-f0-9]{6})\\b|(?<=['\"\\s])(#)([A-Fa-f0-9]{8})\\b/gm);if(n.length>0)for(let r of n){let n=r.filter(e=>e!==void 0),i=n[1],a=n[2];if(!a)continue;let o;i===`rgb`?o=Ia(Pa(e,r),Ra(a,/^\\(\\s*(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\\s*,\\s*(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\\s*,\\s*(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\\s*\\)$/gm),!1):i===`rgba`?o=Ia(Pa(e,r),Ra(a,/^\\(\\s*(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\\s*,\\s*(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\\s*,\\s*(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9][0-9]|[0-9])\\s*,\\s*(0[.][0-9]+|[.][0-9]+|[01][.]|[01])\\s*\\)$/gm),!0):i===`hsl`?o=La(Pa(e,r),Ra(a,/^\\(\\s*((?:360(?:\\.0+)?|(?:36[0]|3[0-5][0-9]|[12][0-9][0-9]|[1-9]?[0-9])(?:\\.\\d+)?))\\s*[\\s,]\\s*(100|\\d{1,2}[.]\\d*|\\d{1,2})%\\s*[\\s,]\\s*(100|\\d{1,2}[.]\\d*|\\d{1,2})%\\s*\\)$/gm),!1):i===`hsla`?o=La(Pa(e,r),Ra(a,/^\\(\\s*((?:360(?:\\.0+)?|(?:36[0]|3[0-5][0-9]|[12][0-9][0-9]|[1-9]?[0-9])(?:\\.\\d+)?))\\s*[\\s,]\\s*(100|\\d{1,2}[.]\\d*|\\d{1,2})%\\s*[\\s,]\\s*(100|\\d{1,2}[.]\\d*|\\d{1,2})%\\s*[\\s,]\\s*(0[.][0-9]+|[.][0-9]+|[01][.]0*|[01])\\s*\\)$/gm),!0):i===`#`&&(o=Fa(Pa(e,r),i+a)),o&&t.push(o)}return t}function Ba(e){return!e||typeof e.getValue!=`function`||typeof e.positionAt!=`function`?[]:za(e)}const Va=/^-+|-+$/g;function Ha(e,t){let n=[];if(t.findRegionSectionHeaders&&t.foldingRules?.markers){let r=Ua(e,t);n=n.concat(r)}if(t.findMarkSectionHeaders){let r=Wa(e,t);n=n.concat(r)}return n}function Ua(e,t){let n=[],r=e.getLineCount();for(let i=1;i<=r;i++){let r=e.getLineContent(i),a=r.match(t.foldingRules.markers.start);if(a){let e={startLineNumber:i,startColumn:a[0].length+1,endLineNumber:i,endColumn:r.length+1};if(e.endColumn>e.startColumn){let t={range:e,...Ga(r.substring(a[0].length)),shouldBeInComments:!1};(t.text||t.hasSeparatorLine)&&n.push(t)}}}return n}function Wa(e,t){let n=[],r=e.getLineCount();if(!t.markSectionHeaderRegex||t.markSectionHeaderRegex.trim()===``)return n;let i=Wr(t.markSectionHeaderRegex),a=new RegExp(t.markSectionHeaderRegex,`gdm${i?`s`:``}`);if(we(a))return n;for(let t=1;t<=r;t+=95){let i=Math.min(t+100-1,r),o=[];for(let n=t;n<=i;n++)o.push(e.getLineContent(n));let s=o.join(`\n`);a.lastIndex=0;let c;for(;(c=a.exec(s))!==null;){let e=s.substring(0,c.index),r=(e.match(/\\n/g)||[]).length,i=t+r,o=c[0].split(`\n`),l=o.length,u=i+l-1,d=e.lastIndexOf(`\n`)+1,f=c.index-d+1,p=o[o.length-1],m={range:{startLineNumber:i,startColumn:f,endLineNumber:u,endColumn:l===1?f+c[0].length:p.length+1},text:(c.groups??{}).label??``,hasSeparatorLine:((c.groups??{}).separator??``)!==``,shouldBeInComments:!0};(m.text||m.hasSeparatorLine)&&(n.length===0||n[n.length-1].range.endLineNumber<m.range.startLineNumber)&&n.push(m),a.lastIndex=c.index+c[0].length}}return n}function Ga(e){e=e.trim();let t=e.startsWith(`-`);return e=e.replace(Va,``),{text:e,hasSeparatorLine:t}}(function(){let e=globalThis;typeof e.requestIdleCallback!=`function`||e.cancelIdleCallback})();var Ka=class{get isRejected(){return this.outcome?.outcome===1}get isSettled(){return!!this.outcome}constructor(){this.p=new Promise((e,t)=>{this.completeCallback=e,this.errorCallback=t})}complete(e){return this.isSettled?Promise.resolve():new Promise(t=>{this.completeCallback(e),this.outcome={outcome:0,value:e},t()})}error(e){return this.isSettled?Promise.resolve():new Promise(t=>{this.errorCallback(e),this.outcome={outcome:1,value:e},t()})}cancel(){return this.error(new i)}},qa;(function(e){async function t(e){let t,n=await Promise.all(e.map(e=>e.then(e=>e,e=>{t||=e})));if(t!==void 0)throw t;return n}e.settled=t;function n(e){return new Promise(async(t,n)=>{try{await e(t,n)}catch(e){n(e)}})}e.withAsyncBody=n})(qa||={});var Ja=class{constructor(){this._unsatisfiedConsumers=[],this._unconsumedValues=[]}get hasFinalValue(){return!!this._finalValue}produce(e){if(this._ensureNoFinalValue(),this._unsatisfiedConsumers.length>0){let t=this._unsatisfiedConsumers.shift();this._resolveOrRejectDeferred(t,e)}else this._unconsumedValues.push(e)}produceFinal(e){this._ensureNoFinalValue(),this._finalValue=e;for(let t of this._unsatisfiedConsumers)this._resolveOrRejectDeferred(t,e);this._unsatisfiedConsumers.length=0}_ensureNoFinalValue(){if(this._finalValue)throw new o(`ProducerConsumer: cannot produce after final value has been set`)}_resolveOrRejectDeferred(e,t){t.ok?e.complete(t.value):e.error(t.error)}consume(){if(this._unconsumedValues.length>0||this._finalValue){let e=this._unconsumedValues.length>0?this._unconsumedValues.shift():this._finalValue;return e.ok?Promise.resolve(e.value):Promise.reject(e.error)}{let e=new Ka;return this._unsatisfiedConsumers.push(e),e.p}}};(class e{constructor(e,t){this._onReturn=t,this._producerConsumer=new Ja,this._iterator={next:()=>this._producerConsumer.consume(),return:()=>(this._onReturn?.(),Promise.resolve({done:!0,value:void 0})),throw:async e=>(this._finishError(e),{done:!0,value:void 0})},queueMicrotask(async()=>{let t=e({emitOne:e=>this._producerConsumer.produce({ok:!0,value:{done:!1,value:e}}),emitMany:e=>{for(let t of e)this._producerConsumer.produce({ok:!0,value:{done:!1,value:t}})},reject:e=>this._finishError(e)});if(!this._producerConsumer.hasFinalValue)try{await t,this._finishOk()}catch(e){this._finishError(e)}})}static fromArray(t){return new e(e=>{e.emitMany(t)})}static fromPromise(t){return new e(async e=>{e.emitMany(await t)})}static fromPromisesResolveOrder(t){return new e(async e=>{await Promise.all(t.map(async t=>e.emitOne(await t)))})}static merge(t){return new e(async e=>{await Promise.all(t.map(async t=>{for await(let n of t)e.emitOne(n)}))})}static{this.EMPTY=e.fromArray([])}static map(t,n){return new e(async e=>{for await(let r of t)e.emitOne(n(r))})}map(t){return e.map(this,t)}static coalesce(t){return e.filter(t,e=>!!e)}coalesce(){return e.coalesce(this)}static filter(t,n){return new e(async e=>{for await(let r of t)n(r)&&e.emitOne(r)})}filter(t){return e.filter(this,t)}_finishOk(){this._producerConsumer.hasFinalValue||this._producerConsumer.produceFinal({ok:!0,value:{done:!0,value:void 0}})}_finishError(e){this._producerConsumer.hasFinalValue||this._producerConsumer.produceFinal({ok:!1,error:e})}[Symbol.asyncIterator](){return this._iterator}});var Ya=class{constructor(e){this.values=e,this.prefixSum=new Uint32Array(e.length),this.prefixSumValidIndex=/* @__PURE__ */ new Int32Array(1),this.prefixSumValidIndex[0]=-1}insertValues(e,t){e=mt(e);let n=this.values,r=this.prefixSum,i=t.length;return i!==0&&(this.values=new Uint32Array(n.length+i),this.values.set(n.subarray(0,e),0),this.values.set(n.subarray(e),e+i),this.values.set(t,e),e-1<this.prefixSumValidIndex[0]&&(this.prefixSumValidIndex[0]=e-1),this.prefixSum=new Uint32Array(this.values.length),this.prefixSumValidIndex[0]>=0&&this.prefixSum.set(r.subarray(0,this.prefixSumValidIndex[0]+1)),!0)}setValue(e,t){return e=mt(e),t=mt(t),this.values[e]!==t&&(this.values[e]=t,e-1<this.prefixSumValidIndex[0]&&(this.prefixSumValidIndex[0]=e-1),!0)}removeValues(e,t){e=mt(e),t=mt(t);let n=this.values,r=this.prefixSum;if(e>=n.length)return!1;let i=n.length-e;return t>=i&&(t=i),t!==0&&(this.values=new Uint32Array(n.length-t),this.values.set(n.subarray(0,e),0),this.values.set(n.subarray(e+t),e),this.prefixSum=new Uint32Array(this.values.length),e-1<this.prefixSumValidIndex[0]&&(this.prefixSumValidIndex[0]=e-1),this.prefixSumValidIndex[0]>=0&&this.prefixSum.set(r.subarray(0,this.prefixSumValidIndex[0]+1)),!0)}getTotalSum(){return this.values.length===0?0:this._getPrefixSum(this.values.length-1)}getPrefixSum(e){return e<0?0:(e=mt(e),this._getPrefixSum(e))}_getPrefixSum(e){if(e<=this.prefixSumValidIndex[0])return this.prefixSum[e];let t=this.prefixSumValidIndex[0]+1;t===0&&(this.prefixSum[0]=this.values[0],t++),e>=this.values.length&&(e=this.values.length-1);for(let n=t;n<=e;n++)this.prefixSum[n]=this.prefixSum[n-1]+this.values[n];return this.prefixSumValidIndex[0]=Math.max(this.prefixSumValidIndex[0],e),this.prefixSum[e]}getIndexOf(e){e=Math.floor(e),this.getTotalSum();let t=0,n=this.values.length-1,r=0,i=0,a=0;for(;t<=n;)if(r=t+(n-t)/2|0,i=this.prefixSum[r],a=i-this.values[r],e<a)n=r-1;else if(e>=i)t=r+1;else break;return new Xa(r,e-a)}},Xa=class{constructor(e,t){this.index=e,this.remainder=t,this._prefixSumIndexOfResultBrand=void 0,this.index=e,this.remainder=t}},Za=class{constructor(e,t,n,r){this._uri=e,this._lines=t,this._eol=n,this._versionId=r,this._lineStarts=null,this._cachedTextValue=null}dispose(){this._lines.length=0}get version(){return this._versionId}getText(){return this._cachedTextValue===null&&(this._cachedTextValue=this._lines.join(this._eol)),this._cachedTextValue}onEvents(e){e.eol&&e.eol!==this._eol&&(this._eol=e.eol,this._lineStarts=null);let t=e.changes;for(let e of t)this._acceptDeleteRange(e.range),this._acceptInsertText(new P(e.range.startLineNumber,e.range.startColumn),e.text);this._versionId=e.versionId,this._cachedTextValue=null}_ensureLineStarts(){if(!this._lineStarts){let e=this._eol.length,t=this._lines.length,n=new Uint32Array(t);for(let r=0;r<t;r++)n[r]=this._lines[r].length+e;this._lineStarts=new Ya(n)}}_setLineText(e,t){this._lines[e]=t,this._lineStarts&&this._lineStarts.setValue(e,this._lines[e].length+this._eol.length)}_acceptDeleteRange(e){if(e.startLineNumber===e.endLineNumber){if(e.startColumn===e.endColumn)return;this._setLineText(e.startLineNumber-1,this._lines[e.startLineNumber-1].substring(0,e.startColumn-1)+this._lines[e.startLineNumber-1].substring(e.endColumn-1))}else this._setLineText(e.startLineNumber-1,this._lines[e.startLineNumber-1].substring(0,e.startColumn-1)+this._lines[e.endLineNumber-1].substring(e.endColumn-1)),this._lines.splice(e.startLineNumber,e.endLineNumber-e.startLineNumber),this._lineStarts&&this._lineStarts.removeValues(e.startLineNumber,e.endLineNumber-e.startLineNumber)}_acceptInsertText(e,t){if(t.length===0)return;let n=Te(t);if(n.length===1){this._setLineText(e.lineNumber-1,this._lines[e.lineNumber-1].substring(0,e.column-1)+n[0]+this._lines[e.lineNumber-1].substring(e.column-1));return}n[n.length-1]+=this._lines[e.lineNumber-1].substring(e.column-1),this._setLineText(e.lineNumber-1,this._lines[e.lineNumber-1].substring(0,e.column-1)+n[0]);let r=new Uint32Array(n.length-1);for(let t=1;t<n.length;t++)this._lines.splice(e.lineNumber+t-1,0,n[t]),r[t-1]=n[t].length+this._eol.length;this._lineStarts&&this._lineStarts.insertValues(e.lineNumber,r)}},Qa=class{constructor(){this._models=Object.create(null)}getModel(e){return this._models[e]}getModels(){let e=[];return Object.keys(this._models).forEach(t=>e.push(this._models[t])),e}$acceptNewModel(e){this._models[e.url]=new $a(on.parse(e.url),e.lines,e.EOL,e.versionId)}$acceptModelChanged(e,t){this._models[e]&&this._models[e].onEvents(t)}$acceptRemovedModel(e){this._models[e]&&delete this._models[e]}},$a=class extends Za{get uri(){return this._uri}get eol(){return this._eol}getValue(){return this.getText()}findMatches(e){let t=[];for(let n=0;n<this._lines.length;n++){let r=this._lines[n],i=this.offsetAt(new P(n+1,1)),a=r.matchAll(e);for(let e of a)(e.index||e.index===0)&&(e.index+=i),t.push(e)}return t}getLinesContent(){return this._lines.slice(0)}getLineCount(){return this._lines.length}getLineContent(e){return this._lines[e-1]}getWordAtPosition(e,t){let n=$r(e.column,Zr(t),this._lines[e.lineNumber-1],0);return n?new F(e.lineNumber,n.startColumn,e.lineNumber,n.endColumn):null}words(e){let t=this._lines,n=this._wordenize.bind(this),r=0,i=``,a=0,o=[];return{*[Symbol.iterator](){for(;;)if(a<o.length){let e=i.substring(o[a].start,o[a].end);a+=1,yield e}else if(r<t.length)i=t[r],o=n(i,e),a=0,r+=1;else break}}}getLineWords(e,t){let n=this._lines[e-1],r=this._wordenize(n,t),i=[];for(let e of r)i.push({word:n.substring(e.start,e.end),startColumn:e.start+1,endColumn:e.end+1});return i}_wordenize(e,t){let n=[],r;for(t.lastIndex=0;(r=t.exec(e))&&r[0].length!==0;)n.push({start:r.index,end:r.index+r[0].length});return n}getValueInRange(e){if(e=this._validateRange(e),e.startLineNumber===e.endLineNumber)return this._lines[e.startLineNumber-1].substring(e.startColumn-1,e.endColumn-1);let t=this._eol,n=e.startLineNumber-1,r=e.endLineNumber-1,i=[];i.push(this._lines[n].substring(e.startColumn-1));for(let e=n+1;e<r;e++)i.push(this._lines[e]);return i.push(this._lines[r].substring(0,e.endColumn-1)),i.join(t)}offsetAt(e){return e=this._validatePosition(e),this._ensureLineStarts(),this._lineStarts.getPrefixSum(e.lineNumber-2)+(e.column-1)}positionAt(e){e=Math.floor(e),e=Math.max(0,e),this._ensureLineStarts();let t=this._lineStarts.getIndexOf(e),n=this._lines[t.index].length;return{lineNumber:1+t.index,column:1+Math.min(t.remainder,n)}}_validateRange(e){let t=this._validatePosition({lineNumber:e.startLineNumber,column:e.startColumn}),n=this._validatePosition({lineNumber:e.endLineNumber,column:e.endColumn});return t.lineNumber!==e.startLineNumber||t.column!==e.startColumn||n.lineNumber!==e.endLineNumber||n.column!==e.endColumn?{startLineNumber:t.lineNumber,startColumn:t.column,endLineNumber:n.lineNumber,endColumn:n.column}:e}_validatePosition(e){if(!P.isIPosition(e))throw Error(`bad position`);let{lineNumber:t,column:n}=e,r=!1;if(t<1)t=1,n=1,r=!0;else if(t>this._lines.length)t=this._lines.length,n=this._lines[t-1].length+1,r=!0;else{let e=this._lines[t-1].length+1;n<1?(n=1,r=!0):n>e&&(n=e,r=!0)}return r?{lineNumber:t,column:n}:e}},eo=class e{constructor(e=null){this._foreignModule=e,this._requestHandlerBrand=void 0,this._workerTextModelSyncServer=new Qa}dispose(){}async $ping(){return`pong`}_getModel(e){return this._workerTextModelSyncServer.getModel(e)}getModels(){return this._workerTextModelSyncServer.getModels()}$acceptNewModel(e){this._workerTextModelSyncServer.$acceptNewModel(e)}$acceptModelChanged(e,t){this._workerTextModelSyncServer.$acceptModelChanged(e,t)}$acceptRemovedModel(e){this._workerTextModelSyncServer.$acceptRemovedModel(e)}async $computeUnicodeHighlights(e,t,n){let r=this._getModel(e);return r?ti.computeUnicodeHighlights(r,t,n):{ranges:[],hasMore:!1,ambiguousCharacterCount:0,invisibleCharacterCount:0,nonBasicAsciiCharacterCount:0}}async $findSectionHeaders(e,t){let n=this._getModel(e);return n?Ha(n,t):[]}async $computeDiff(t,n,r,i){let a=this._getModel(t),o=this._getModel(n);return!a||!o?null:e.computeDiff(a,o,r,i)}static computeDiff(e,t,n,r){let i=r===`advanced`?ka.getDefault():ka.getLegacy(),a=e.getLinesContent(),o=t.getLinesContent(),s=i.computeDiff(a,o,n),c=s.changes.length>0?!1:this._modelsAreIdentical(e,t);function l(e){return e.map(e=>[e.original.startLineNumber,e.original.endLineNumberExclusive,e.modified.startLineNumber,e.modified.endLineNumberExclusive,e.innerChanges?.map(e=>[e.originalRange.startLineNumber,e.originalRange.startColumn,e.originalRange.endLineNumber,e.originalRange.endColumn,e.modifiedRange.startLineNumber,e.modifiedRange.startColumn,e.modifiedRange.endLineNumber,e.modifiedRange.endColumn])])}return{identical:c,quitEarly:s.hitTimeout,changes:l(s.changes),moves:s.moves.map(e=>[e.lineRangeMapping.original.startLineNumber,e.lineRangeMapping.original.endLineNumberExclusive,e.lineRangeMapping.modified.startLineNumber,e.lineRangeMapping.modified.endLineNumberExclusive,l(e.changes)])}}static _modelsAreIdentical(e,t){let n=e.getLineCount();if(n!==t.getLineCount())return!1;for(let r=1;r<=n;r++)if(e.getLineContent(r)!==t.getLineContent(r))return!1;return!0}static{this._diffLimit=1e5}async $computeMoreMinimalEdits(t,n,r){let i=this._getModel(t);if(!i)return n;let a=[],o;n=n.slice(0).sort((e,t)=>e.range&&t.range?F.compareRangesUsingStarts(e.range,t.range):+!e.range-!t.range);let s=0;for(let e=1;e<n.length;e++)F.getEndPosition(n[s].range).equals(F.getStartPosition(n[e].range))?(n[s].range=F.fromPositions(F.getStartPosition(n[s].range),F.getEndPosition(n[e].range)),n[s].text+=n[e].text):(s++,n[s]=n[e]);n.length=s+1;for(let{range:t,text:s,eol:c}of n){if(typeof c==`number`&&(o=c),F.isEmpty(t)&&!s)continue;let n=i.getValueInRange(t);if(s=s.replace(/\\r\\n|\\n|\\r/g,i.eol),n===s)continue;if(Math.max(s.length,n.length)>e._diffLimit){a.push({range:t,text:s});continue}let l=ct(n,s,r),u=i.offsetAt(F.lift(t).getStartPosition());for(let e of l){let t=i.positionAt(u+e.originalStart),n=i.positionAt(u+e.originalStart+e.originalLength),r={text:s.substr(e.modifiedStart,e.modifiedLength),range:{startLineNumber:t.lineNumber,startColumn:t.column,endLineNumber:n.lineNumber,endColumn:n.column}};i.getValueInRange(r.range)!==r.text&&a.push(r)}}return typeof o==`number`&&a.push({eol:o,text:``,range:{startLineNumber:0,startColumn:0,endLineNumber:0,endColumn:0}}),a}async $computeLinks(e){let t=this._getModel(e);return t?Ct(t):null}async $computeDefaultDocumentColors(e){let t=this._getModel(e);return t?Ba(t):null}static{this._suggestionsLimit=1e4}async $textualSuggest(t,n,r,i){let a=new C,o=new RegExp(r,i),s=/* @__PURE__ */ new Set;outer:for(let r of t){let t=this._getModel(r);if(t){for(let r of t.words(o))if(r!==n&&isNaN(Number(r))&&(s.add(r),s.size>e._suggestionsLimit))break outer}}return{words:Array.from(s),duration:a.elapsed()}}async $computeWordRanges(e,t,n,r){let i=this._getModel(e);if(!i)return Object.create(null);let a=new RegExp(n,r),o=Object.create(null);for(let e=t.startLineNumber;e<t.endLineNumber;e++){let t=i.getLineWords(e,a);for(let n of t){if(!isNaN(Number(n.word)))continue;let t=o[n.word];t||(t=[],o[n.word]=t),t.push({startLineNumber:e,startColumn:n.startColumn,endLineNumber:e,endColumn:n.endColumn})}}return o}async $navigateValueSet(e,t,n,r,i){let a=this._getModel(e);if(!a)return null;let o=new RegExp(r,i);t.startColumn===t.endColumn&&(t={startLineNumber:t.startLineNumber,startColumn:t.startColumn,endLineNumber:t.endLineNumber,endColumn:t.endColumn+1});let s=a.getValueInRange(t),c=a.getWordAtPosition({lineNumber:t.startLineNumber,column:t.startColumn},o);if(!c)return null;let l=a.getValueInRange(c);return wt.INSTANCE.navigateValueSet(t,s,c,l,n)}$fmr(e,t){if(!this._foreignModule||typeof this._foreignModule[e]!=`function`)return Promise.reject(/* @__PURE__ */ Error(`Missing requestHandler or method: `+e));try{return Promise.resolve(this._foreignModule[e].apply(this._foreignModule,t))}catch(e){return Promise.reject(e)}}};typeof importScripts==`function`&&(globalThis.monaco=jr());var to=class e{static{this.CHANNEL_NAME=`editorWorkerHost`}static getChannel(t){return t.getChannel(e.CHANNEL_NAME)}static setChannel(t,n){t.setChannel(e.CHANNEL_NAME,n)}};function no(e){let t,n=Ze(r=>{let i=to.getChannel(r);return t=e({host:new Proxy({},{get(e,t,n){if(t!==`then`){if(typeof t!=`string`)throw Error(`Not supported`);return(...e)=>i.$fhr(t,e)}}}),getMirrorModels:()=>n.requestHandler.getModels()}),new eo(t)});return t}function ro(){return!1}self.onmessage=()=>{ro()||no(()=>({}))};", ne = typeof self < "u" && self.Blob && new Blob(["URL.revokeObjectURL(import.meta.url);", te], { type: "text/javascript;charset=utf-8" });
function C(e) {
	let t;
	try {
		if (t = ne && (self.URL || self.webkitURL).createObjectURL(ne), !t) throw "";
		let n = new Worker(t, {
			type: "module",
			name: e?.name
		});
		return n.addEventListener("error", () => {
			(self.URL || self.webkitURL).revokeObjectURL(t);
		}), n;
	} catch {
		return new Worker("data:text/javascript;charset=utf-8," + encodeURIComponent(te), {
			type: "module",
			name: e?.name
		});
	}
}
//#endregion
//#region node_modules/.pnpm/@milkdown+exception@7.22.2/node_modules/@milkdown/exception/lib/index.js
var w = /* @__PURE__ */ function(e) {
	return e.docTypeError = "docTypeError", e.contextNotFound = "contextNotFound", e.timerNotFound = "timerNotFound", e.ctxCallOutOfScope = "ctxCallOutOfScope", e.createNodeInParserFail = "createNodeInParserFail", e.stackOverFlow = "stackOverFlow", e.parserMatchError = "parserMatchError", e.serializerMatchError = "serializerMatchError", e.getAtomFromSchemaFail = "getAtomFromSchemaFail", e.expectDomTypeError = "expectDomTypeError", e.callCommandBeforeEditorView = "callCommandBeforeEditorView", e.missingRootElement = "missingRootElement", e.missingNodeInSchema = "missingNodeInSchema", e.missingMarkInSchema = "missingMarkInSchema", e.ctxNotBind = "ctxNotBind", e.missingYjsDoc = "missingYjsDoc", e.aiProviderError = "aiProviderError", e.aiBuildContextError = "aiBuildContextError", e;
}({}), T = class extends Error {
	constructor(e, t, n) {
		super(t, n), this.name = "MilkdownError", this.code = e, n?.cause !== void 0 && (this.cause = n.cause);
	}
}, re = (e, t) => typeof t == "function" ? "[Function]" : t, ie = (e) => JSON.stringify(e, re);
function ae(e) {
	return new T(w.docTypeError, `Doc type error, unsupported type: ${ie(e)}`);
}
function oe(e) {
	return new T(w.contextNotFound, `Context "${e}" not found, do you forget to inject it?`);
}
function se(e) {
	return new T(w.timerNotFound, `Timer "${e}" not found, do you forget to record it?`);
}
function ce() {
	return new T(w.ctxCallOutOfScope, "Should not call a context out of the plugin.");
}
function le(e, t, n) {
	let r = `Cannot create node for ${"name" in e ? e.name : e}`, i = (e) => {
		if (e == null) return "null";
		if (Array.isArray(e)) return `[${e.map(i).join(", ")}]`;
		if (typeof e == "object") return "toJSON" in e && typeof e.toJSON == "function" ? JSON.stringify(e.toJSON()) : "spec" in e ? JSON.stringify(e.spec) : JSON.stringify(e);
		if (typeof e == "string" || typeof e == "number" || typeof e == "boolean") return JSON.stringify(e);
		if (typeof e == "function") return `[Function: ${e.name || "anonymous"}]`;
		try {
			return String(e);
		} catch {
			return "[Unserializable]";
		}
	}, a = [
		["[Description]", r],
		["[Attributes]", t],
		["[Content]", (n ?? []).map((e) => e ? typeof e == "object" && "type" in e ? `${e}` : i(e) : "null")]
	].reduce((e, [t, n]) => {
		let r = `${t}: ${i(n)}.`;
		return e.concat(r);
	}, []);
	return new T(w.createNodeInParserFail, a.join("\n"));
}
function ue() {
	return new T(w.stackOverFlow, "Stack over flow, cannot pop on an empty stack.");
}
function de(e) {
	return new T(w.parserMatchError, `Cannot match target parser for node: ${ie(e)}.`);
}
function fe(e) {
	return new T(w.serializerMatchError, `Cannot match target serializer for node: ${ie(e)}.`);
}
function pe(e) {
	return new T(w.expectDomTypeError, `Expect to be a dom, but get: ${ie(e)}.`);
}
function me() {
	return new T(w.callCommandBeforeEditorView, "You're trying to call a command before editor view initialized, make sure to get commandManager from ctx after editor view has been initialized");
}
function he(e) {
	return new T(w.missingNodeInSchema, `Missing node in schema, milkdown cannot find "${e}" in schema.`);
}
function ge(e) {
	return new T(w.missingMarkInSchema, `Missing mark in schema, milkdown cannot find "${e}" in schema.`);
}
//#endregion
//#region node_modules/.pnpm/@milkdown+ctx@7.22.2/node_modules/@milkdown/ctx/lib/index.js
var _e = class {
	constructor() {
		this.sliceMap = /* @__PURE__ */ new Map(), this.get = (e) => {
			let t = typeof e == "string" ? [...this.sliceMap.values()].find((t) => t.type.name === e) : this.sliceMap.get(e.id);
			if (!t) throw oe(typeof e == "string" ? e : e.name);
			return t;
		}, this.remove = (e) => {
			let t = typeof e == "string" ? [...this.sliceMap.values()].find((t) => t.type.name === e) : this.sliceMap.get(e.id);
			t && this.sliceMap.delete(t.type.id);
		}, this.has = (e) => typeof e == "string" ? [...this.sliceMap.values()].some((t) => t.type.name === e) : this.sliceMap.has(e.id);
	}
}, ve = class {
	#e;
	#t;
	#n;
	constructor(e, t, n) {
		this.#e = [], this.#n = () => {
			this.#e.forEach((e) => e(this.#t));
		}, this.set = (e) => {
			this.#t = e, this.#n();
		}, this.get = () => this.#t, this.update = (e) => {
			this.#t = e(this.#t), this.#n();
		}, this.type = n, this.#t = t, e.set(n.id, this);
	}
	on(e) {
		return this.#e.push(e), () => {
			this.#e = this.#e.filter((t) => t !== e);
		};
	}
	once(e) {
		let t = this.on((n) => {
			e(n), t();
		});
		return t;
	}
	off(e) {
		this.#e = this.#e.filter((t) => t !== e);
	}
	offAll() {
		this.#e = [];
	}
}, ye = class {
	constructor(e, t) {
		this.id = Symbol(`Context-${t}`), this.name = t, this._defaultValue = e, this._typeInfo = () => {
			throw ce();
		};
	}
	create(e, t = this._defaultValue) {
		return new ve(e, t, this);
	}
}, E = (e, t) => new ye(e, t), be = class {
	#e;
	#t;
	#n;
	#r;
	#i;
	#a;
	#o;
	constructor(e, t, n) {
		this.#r = /* @__PURE__ */ new Set(), this.#i = /* @__PURE__ */ new Set(), this.#a = /* @__PURE__ */ new Map(), this.#o = /* @__PURE__ */ new Map(), this.read = () => ({
			metadata: this.#e,
			injectedSlices: [...this.#r].map((e) => ({
				name: typeof e == "string" ? e : e.name,
				value: this.#s(e)
			})),
			consumedSlices: [...this.#i].map((e) => ({
				name: typeof e == "string" ? e : e.name,
				value: this.#s(e)
			})),
			recordedTimers: [...this.#a].map(([e, { duration: t }]) => ({
				name: e.name,
				duration: t,
				status: this.#c(e)
			})),
			waitTimers: [...this.#o].map(([e, { duration: t }]) => ({
				name: e.name,
				duration: t,
				status: this.#c(e)
			}))
		}), this.onRecord = (e) => {
			this.#a.set(e, {
				start: Date.now(),
				duration: 0
			});
		}, this.onClear = (e) => {
			this.#a.delete(e);
		}, this.onDone = (e) => {
			let t = this.#a.get(e);
			t && (t.duration = Date.now() - t.start);
		}, this.onWait = (e, t) => {
			let n = Date.now();
			t.finally(() => {
				this.#o.set(e, { duration: Date.now() - n });
			}).catch(console.error);
		}, this.onInject = (e) => {
			this.#r.add(e);
		}, this.onRemove = (e) => {
			this.#r.delete(e);
		}, this.onUse = (e) => {
			this.#i.add(e);
		}, this.#s = (e) => this.#t.get(e).get(), this.#c = (e) => this.#n.get(e).status, this.#t = e, this.#n = t, this.#e = n;
	}
	#s;
	#c;
}, xe = class e {
	#e;
	#t;
	#n;
	#r;
	constructor(t, n, r) {
		this.produce = (t) => t && Object.keys(t).length ? new e(this.#e, this.#t, { ...t }) : this, this.inject = (e, t) => {
			let n = e.create(this.#e.sliceMap);
			return t != null && n.set(t), this.#r?.onInject(e), this;
		}, this.remove = (e) => (this.#e.remove(e), this.#r?.onRemove(e), this), this.record = (e) => (e.create(this.#t.store), this.#r?.onRecord(e), this), this.clearTimer = (e) => (this.#t.remove(e), this.#r?.onClear(e), this), this.isInjected = (e) => this.#e.has(e), this.isRecorded = (e) => this.#t.has(e), this.use = (e) => (this.#r?.onUse(e), this.#e.get(e)), this.get = (e) => this.use(e).get(), this.set = (e, t) => this.use(e).set(t), this.update = (e, t) => this.use(e).update(t), this.timer = (e) => this.#t.get(e), this.done = (e) => {
			this.timer(e).done(), this.#r?.onDone(e);
		}, this.wait = (e) => {
			let t = this.timer(e).start();
			return this.#r?.onWait(e, t), t;
		}, this.waitTimers = async (e) => {
			await Promise.all(this.get(e).map((e) => this.wait(e)));
		}, this.#e = t, this.#t = n, this.#n = r, r && (this.#r = new be(t, n, r));
	}
	get meta() {
		return this.#n;
	}
	get inspector() {
		return this.#r;
	}
}, Se = class {
	constructor() {
		this.store = /* @__PURE__ */ new Map(), this.get = (e) => {
			let t = this.store.get(e.id);
			if (!t) throw se(e.name);
			return t;
		}, this.remove = (e) => {
			this.store.delete(e.id);
		}, this.has = (e) => this.store.has(e.id);
	}
}, Ce = class {
	#e;
	#t;
	#n;
	#r;
	#i;
	constructor(e, t) {
		this.#e = null, this.#t = null, this.#r = "pending", this.#i = null, this.start = () => (this.#e ??= new Promise((e, t) => {
			this.#t = (t) => {
				t instanceof CustomEvent && t.detail.id === this.#n && (this.#r = "resolved", this.#a(), t.stopImmediatePropagation(), e());
			}, this.#o(() => {
				this.#r === "pending" && (this.#r = "rejected"), this.#a(), t(/* @__PURE__ */ Error(`Timing ${this.type.name} timeout.`));
			}), this.#r = "pending", addEventListener(this.type.name, this.#t);
		}), this.#e), this.done = () => {
			let e = new CustomEvent(this.type.name, { detail: { id: this.#n } });
			dispatchEvent(e);
		}, this.#a = () => {
			this.#i !== null && (clearTimeout(this.#i), this.#i = null), this.#t && removeEventListener(this.type.name, this.#t);
		}, this.#o = (e) => {
			this.#i = setTimeout(() => {
				this.#i = null, e();
			}, this.type.timeout);
		}, this.#n = Symbol(t.name), this.type = t, e.set(t.id, this);
	}
	get status() {
		return this.#r;
	}
	#a;
	#o;
}, we = class {
	constructor(e, t = 3e3) {
		this.create = (e) => new Ce(e, this), this.id = Symbol(`Timer-${e}`), this.name = e, this.timeout = t;
	}
}, Te = (e, t = 3e3) => new we(e, t), Ee = {};
function De(e, t) {
	let n = t || Ee;
	return Oe(e, typeof n.includeImageAlt != "boolean" || n.includeImageAlt, typeof n.includeHtml != "boolean" || n.includeHtml);
}
function Oe(e, t, n) {
	if (Ae(e)) {
		if ("value" in e) return e.type === "html" && !n ? "" : e.value;
		if (t && "alt" in e && e.alt) return e.alt;
		if ("children" in e) return ke(e.children, t, n);
	}
	return Array.isArray(e) ? ke(e, t, n) : "";
}
function ke(e, t, n) {
	let r = [], i = -1;
	for (; ++i < e.length;) r[i] = Oe(e[i], t, n);
	return r.join("");
}
function Ae(e) {
	return !!(e && typeof e == "object");
}
//#endregion
//#region node_modules/.pnpm/decode-named-character-reference@1.3.0/node_modules/decode-named-character-reference/index.dom.js
var je = document.createElement("i");
function Me(e) {
	let t = "&" + e + ";";
	je.innerHTML = t;
	let n = je.textContent;
	return n.charCodeAt(n.length - 1) === 59 && e !== "semi" ? !1 : n !== t && n;
}
//#endregion
//#region node_modules/.pnpm/micromark-util-chunked@2.0.1/node_modules/micromark-util-chunked/index.js
function Ne(e, t, n, r) {
	let i = e.length, a = 0, o;
	if (t = t < 0 ? -t > i ? 0 : i + t : t > i ? i : t, n = n > 0 ? n : 0, r.length < 1e4) o = Array.from(r), o.unshift(t, n), e.splice(...o);
	else for (n && e.splice(t, n); a < r.length;) o = r.slice(a, a + 1e4), o.unshift(t, 0), e.splice(...o), a += 1e4, t += 1e4;
}
function Pe(e, t) {
	return e.length > 0 ? (Ne(e, e.length, 0, t), e) : t;
}
//#endregion
//#region node_modules/.pnpm/micromark-util-combine-extensions@2.0.1/node_modules/micromark-util-combine-extensions/index.js
var Fe = {}.hasOwnProperty;
function Ie(e) {
	let t = {}, n = -1;
	for (; ++n < e.length;) Le(t, e[n]);
	return t;
}
function Le(e, t) {
	let n;
	for (n in t) {
		let r = (Fe.call(e, n) ? e[n] : void 0) || (e[n] = {}), i = t[n], a;
		if (i) for (a in i) {
			Fe.call(r, a) || (r[a] = []);
			let e = i[a];
			Re(r[a], Array.isArray(e) ? e : e ? [e] : []);
		}
	}
}
function Re(e, t) {
	let n = -1, r = [];
	for (; ++n < t.length;) (t[n].add === "after" ? e : r).push(t[n]);
	Ne(e, 0, 0, r);
}
//#endregion
//#region node_modules/.pnpm/micromark-util-decode-numeric-character-reference@2.0.2/node_modules/micromark-util-decode-numeric-character-reference/index.js
function ze(e, t) {
	let n = Number.parseInt(e, t);
	return n < 9 || n === 11 || n > 13 && n < 32 || n > 126 && n < 160 || n > 55295 && n < 57344 || n > 64975 && n < 65008 || (n & 65535) == 65535 || (n & 65535) == 65534 || n > 1114111 ? "�" : String.fromCodePoint(n);
}
//#endregion
//#region node_modules/.pnpm/micromark-util-normalize-identifier@2.0.1/node_modules/micromark-util-normalize-identifier/index.js
function Be(e) {
	return e.replace(/[\t\n\r ]+/g, " ").replace(/^ | $/g, "").toLowerCase().toUpperCase();
}
//#endregion
//#region node_modules/.pnpm/micromark-util-character@2.1.1/node_modules/micromark-util-character/index.js
var D = Ye(/[A-Za-z]/), Ve = Ye(/[\dA-Za-z]/), He = Ye(/[#-'*+\--9=?A-Z^-~]/);
function Ue(e) {
	return e !== null && (e < 32 || e === 127);
}
var We = Ye(/\d/), Ge = Ye(/[\dA-Fa-f]/), Ke = Ye(/[!-/:-@[-`{-~]/);
function O(e) {
	return e !== null && e < -2;
}
function k(e) {
	return e !== null && (e < 0 || e === 32);
}
function A(e) {
	return e === -2 || e === -1 || e === 32;
}
var qe = Ye(/\p{P}|\p{S}/u), Je = Ye(/\s/);
function Ye(e) {
	return t;
	function t(t) {
		return t !== null && t > -1 && e.test(String.fromCharCode(t));
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-factory-space@2.1.0/node_modules/micromark-factory-space/index.js
function j(e, t, n, r) {
	let i = r ? r - 1 : Infinity, a = 0;
	return o;
	function o(r) {
		return A(r) ? (e.enter(n), s(r)) : t(r);
	}
	function s(r) {
		return A(r) && a++ < i ? (e.consume(r), s) : (e.exit(n), t(r));
	}
}
function Xe(e, t, n, r, i, a) {
	let o = 0;
	return s;
	function s(t) {
		return a > 0 && A(t) ? (e.enter(r), c(t)) : l(t);
	}
	function c(t) {
		return A(t) && o < a ? (e.consume(t), o++, c) : (e.exit(r), l(t));
	}
	function l(e) {
		return o >= i ? t(e) : n(e);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark@4.0.3/node_modules/micromark/lib/initialize/content.js
var Ze = { tokenize: Qe };
function Qe(e) {
	let t = e.attempt(this.parser.constructs.contentInitial, r, i), n;
	return t;
	function r(n) {
		if (n === null) e.consume(n);
		else return e.enter("lineEnding"), e.consume(n), e.exit("lineEnding"), j(e, t, "linePrefix");
	}
	function i(t) {
		return e.enter("paragraph"), a(t);
	}
	function a(t) {
		let r = e.enter("chunkText", {
			contentType: "text",
			previous: n
		});
		return n && (n.next = r), n = r, o(t);
	}
	function o(t) {
		if (t === null) e.exit("chunkText"), e.exit("paragraph"), e.consume(t);
		else return O(t) ? (e.consume(t), e.exit("chunkText"), a) : (e.consume(t), o);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-util-edit-map@1.0.0/node_modules/micromark-util-edit-map/index.js
var $e = class {
	constructor() {
		this.index = /* @__PURE__ */ new Map(), this.map = [];
	}
	add(e, t, n) {
		et(this, e, t, n, !1);
	}
	addBefore(e, t, n) {
		et(this, e, t, n, !0);
	}
	consume(e) {
		if (this.map.sort(function(e, t) {
			return e[0] - t[0];
		}), this.map.length === 0) return;
		let t = this.map.length, n = [];
		for (; t > 0;) --t, n.push(e.slice(this.map[t][0] + this.map[t][1]), this.map[t][2]), e.length = this.map[t][0];
		n.push(e.slice()), e.length = 0;
		let r = n.pop();
		for (; r;) {
			for (let t of r) e.push(t);
			r = n.pop();
		}
		this.map.length = 0, this.index.clear();
	}
};
function et(e, t, n, r, i) {
	if (n === 0 && r.length === 0) return;
	let a = e.index.get(t);
	if (a) {
		a[1] += n, i ? (r.push(...a[2]), a[2] = r) : a[2].push(...r);
		return;
	}
	let o = [
		t,
		n,
		r
	];
	e.map.push(o), e.index.set(t, o);
}
//#endregion
//#region node_modules/.pnpm/micromark@4.0.3/node_modules/micromark/lib/initialize/document.js
var tt = { tokenize: rt }, nt = { tokenize: it };
function rt(e) {
	let t = this, n = [], r = 0, i, a, o;
	return s;
	function s(i) {
		if (r < n.length) {
			let a = n[r];
			return t.containerState = a[1], e.attempt(a[0].continuation, c, l)(i);
		}
		return l(i);
	}
	function c(e) {
		if (r++, t.containerState._closeFlow) {
			t.containerState._closeFlow = void 0, i && v();
			let n = t.events.length, a = n, o;
			for (; a--;) if (t.events[a][0] === "exit" && t.events[a][1].type === "chunkFlow") {
				o = t.events[a][1].end;
				break;
			}
			_(r);
			let s = n;
			for (; s < t.events.length;) t.events[s][1].end = { ...o }, s++;
			let c = new $e();
			return c.add(a + 1, 0, t.events.slice(n)), c.add(n, s - n, []), c.consume(t.events), l(e);
		}
		return s(e);
	}
	function l(a) {
		if (r === n.length) {
			if (!i) return f(a);
			if (i.currentConstruct && i.currentConstruct.concrete) return m(a);
			t.interrupt = !(!i.currentConstruct || i._gfmTableDynamicInterruptHack);
		}
		return t.containerState = {}, e.check(nt, u, d)(a);
	}
	function u(e) {
		return i && v(), _(r), f(e);
	}
	function d(e) {
		return t.parser.lazy[t.now().line] = r !== n.length, o = t.now().offset, m(e);
	}
	function f(n) {
		return t.containerState = {}, e.attempt(nt, p, m)(n);
	}
	function p(e) {
		return r++, n.push([t.currentConstruct, t.containerState]), f(e);
	}
	function m(n) {
		if (n === null) i && v(), _(0), e.consume(n);
		else return i ||= t.parser.flow(t.now()), e.enter("chunkFlow", {
			_tokenizer: i,
			contentType: "flow",
			previous: a
		}), h(n);
	}
	function h(n) {
		if (n === null) g(e.exit("chunkFlow"), !0), _(0), e.consume(n);
		else return O(n) ? (e.consume(n), g(e.exit("chunkFlow")), r = 0, t.interrupt = void 0, s) : (e.consume(n), h);
	}
	function g(e, n) {
		let s = t.sliceStream(e);
		if (n && s.push(null), e.previous = a, a && (a.next = e), a = e, i.defineSkip(e.start), i.write(s), t.parser.lazy[e.start.line]) {
			let e = i.events.length;
			for (; e--;) if (i.events[e][1].start.offset < o && (!i.events[e][1].end || i.events[e][1].end.offset > o)) return;
			let n = t.events.length, a = n, s, c;
			for (; a--;) if (t.events[a][0] === "exit" && t.events[a][1].type === "chunkFlow") {
				if (s) {
					c = t.events[a][1].end;
					break;
				}
				s = !0;
			}
			for (_(r), e = n; e < t.events.length;) t.events[e][1].end = { ...c }, e++;
			let l = new $e();
			l.add(a + 1, 0, t.events.slice(n)), l.add(n, e - n, []), l.consume(t.events);
		}
	}
	function _(r) {
		let i = n.length;
		for (; i-- > r;) {
			let r = n[i];
			t.containerState = r[1], r[0].exit.call(t, e);
		}
		n.length = r;
	}
	function v() {
		i.write([null]), a = void 0, i = void 0, t.containerState._closeFlow = void 0;
	}
}
function it(e, t, n) {
	return j(e, e.attempt(this.parser.constructs.document, t, n), "linePrefix", this.parser.constructs.disable.null.includes("codeIndented") ? void 0 : 4);
}
//#endregion
//#region node_modules/.pnpm/micromark-util-classify-character@2.0.1/node_modules/micromark-util-classify-character/index.js
function at(e) {
	if (e === null || k(e) || Je(e)) return 1;
	if (qe(e)) return 2;
}
//#endregion
//#region node_modules/.pnpm/micromark-util-resolve-all@2.0.1/node_modules/micromark-util-resolve-all/index.js
function ot(e, t, n) {
	let r = [], i = -1;
	for (; ++i < e.length;) {
		let a = e[i].resolveAll;
		a && !r.includes(a) && (t = a(t, n), r.push(a));
	}
	return t;
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/attention.js
var st = {
	name: "attention",
	resolveAll: ct,
	tokenize: lt
};
function ct(e, t) {
	let n = -1, r;
	for (; ++n < e.length;) if (e[n][0] === "enter" && e[n][1].type === "attentionSequence" && e[n][1]._close) {
		let i = n;
		for (; i--;) if (e[i][0] === "exit" && e[i][1].type === "attentionSequence" && e[i][1]._open && t.sliceSerialize(e[i][1]).charCodeAt(0) === t.sliceSerialize(e[n][1]).charCodeAt(0)) {
			if ((e[i][1]._close || e[n][1]._open) && (e[n][1].end.offset - e[n][1].start.offset) % 3 && !((e[i][1].end.offset - e[i][1].start.offset + e[n][1].end.offset - e[n][1].start.offset) % 3)) continue;
			let a = e[i][1].end.offset - e[i][1].start.offset > 1 && e[n][1].end.offset - e[n][1].start.offset > 1 ? 2 : 1, o = { ...e[i][1].end }, s = { ...e[n][1].start };
			ut(o, -a), ut(s, a);
			let c = {
				type: a > 1 ? "strongSequence" : "emphasisSequence",
				start: o,
				end: { ...e[i][1].end }
			}, l = {
				type: a > 1 ? "strongSequence" : "emphasisSequence",
				start: { ...e[n][1].start },
				end: s
			}, u = {
				type: a > 1 ? "strongText" : "emphasisText",
				start: { ...e[i][1].end },
				end: { ...e[n][1].start }
			}, d = {
				type: a > 1 ? "strong" : "emphasis",
				start: { ...c.start },
				end: { ...l.end }
			};
			e[i][1].end = { ...c.start }, e[n][1].start = { ...l.end }, r = [], e[i][1].end.offset - e[i][1].start.offset && (r = Pe(r, [[
				"enter",
				e[i][1],
				t
			], [
				"exit",
				e[i][1],
				t
			]])), r = Pe(r, [
				[
					"enter",
					d,
					t
				],
				[
					"enter",
					c,
					t
				],
				[
					"exit",
					c,
					t
				],
				[
					"enter",
					u,
					t
				]
			]), r = Pe(r, ot(t.parser.constructs.insideSpan.null, e.slice(i + 1, n), t)), r = Pe(r, [
				[
					"exit",
					u,
					t
				],
				[
					"enter",
					l,
					t
				],
				[
					"exit",
					l,
					t
				],
				[
					"exit",
					d,
					t
				]
			]);
			let f = 0;
			e[n][1].end.offset - e[n][1].start.offset && (f = 2, r = Pe(r, [[
				"enter",
				e[n][1],
				t
			], [
				"exit",
				e[n][1],
				t
			]])), Ne(e, i - 1, n - i + 3, r), n = i + r.length - f - 2;
			break;
		}
	}
	for (n = -1; ++n < e.length;) e[n][1].type === "attentionSequence" && (e[n][1].type = "data");
	return e;
}
function lt(e, t) {
	let n = this.parser.constructs.attentionMarkers.null, r = this.previous, i = at(r), a;
	return o;
	function o(t) {
		return a = t, e.enter("attentionSequence"), s(t);
	}
	function s(o) {
		if (o === a) return e.consume(o), s;
		let c = e.exit("attentionSequence"), l = at(o), u = !l || l === 2 && i || n.includes(o) && o !== 42 && o !== 95, d = !i || i === 2 && l || n.includes(r) && r !== 42 && r !== 95;
		return c._open = !!(a === 42 ? u : u && (i || !d)), c._close = !!(a === 42 ? d : d && (l || !u)), t(o);
	}
}
function ut(e, t) {
	e.column += t, e.offset += t, e._bufferIndex += t;
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/autolink.js
var dt = {
	name: "autolink",
	tokenize: ft
};
function ft(e, t, n) {
	let r = 0;
	return i;
	function i(t) {
		return e.enter("autolink"), e.enter("autolinkMarker"), e.consume(t), e.exit("autolinkMarker"), e.enter("autolinkProtocol"), a;
	}
	function a(t) {
		return D(t) ? (e.consume(t), o) : t === 64 ? n(t) : l(t);
	}
	function o(e) {
		return e === 43 || e === 45 || e === 46 || Ve(e) ? (r = 1, s(e)) : l(e);
	}
	function s(t) {
		return t === 58 ? (e.consume(t), r = 0, c) : (t === 43 || t === 45 || t === 46 || Ve(t)) && r++ < 32 ? (e.consume(t), s) : (r = 0, l(t));
	}
	function c(r) {
		return r === 62 ? (e.exit("autolinkProtocol"), e.enter("autolinkMarker"), e.consume(r), e.exit("autolinkMarker"), e.exit("autolink"), t) : r === null || r === 32 || r === 60 || Ue(r) ? n(r) : (e.consume(r), c);
	}
	function l(t) {
		return t === 64 ? (e.consume(t), u) : He(t) ? (e.consume(t), l) : n(t);
	}
	function u(e) {
		return Ve(e) ? d(e) : n(e);
	}
	function d(n) {
		return n === 46 ? (e.consume(n), r = 0, u) : n === 62 ? (e.exit("autolinkProtocol").type = "autolinkEmail", e.enter("autolinkMarker"), e.consume(n), e.exit("autolinkMarker"), e.exit("autolink"), t) : f(n);
	}
	function f(t) {
		if ((t === 45 || Ve(t)) && r++ < 63) {
			let n = t === 45 ? f : d;
			return e.consume(t), n;
		}
		return n(t);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/blank-line.js
var pt = {
	partial: !0,
	tokenize: mt
};
function mt(e, t, n) {
	return r;
	function r(t) {
		return A(t) ? j(e, i, "linePrefix")(t) : i(t);
	}
	function i(e) {
		return e === null || O(e) ? t(e) : n(e);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/block-quote.js
var ht = {
	continuation: { tokenize: _t },
	exit: vt,
	name: "blockQuote",
	tokenize: gt
};
function gt(e, t, n) {
	let r = this;
	return i;
	function i(t) {
		if (t === 62) {
			let n = r.containerState;
			return n.open ||= (e.enter("blockQuote", { _container: !0 }), !0), e.enter("blockQuotePrefix"), e.enter("blockQuoteMarker"), e.consume(t), e.exit("blockQuoteMarker"), a;
		}
		return n(t);
	}
	function a(n) {
		return A(n) ? (e.enter("blockQuotePrefixWhitespace"), e.consume(n), e.exit("blockQuotePrefixWhitespace"), e.exit("blockQuotePrefix"), t) : (e.exit("blockQuotePrefix"), t(n));
	}
}
function _t(e, t, n) {
	let r = this;
	return i;
	function i(t) {
		return A(t) ? j(e, a, "linePrefix", r.parser.constructs.disable.null.includes("codeIndented") ? void 0 : 4)(t) : a(t);
	}
	function a(r) {
		return e.attempt(ht, t, n)(r);
	}
}
function vt(e) {
	e.exit("blockQuote");
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/character-escape.js
var yt = {
	name: "characterEscape",
	tokenize: bt
};
function bt(e, t, n) {
	return r;
	function r(t) {
		return e.enter("characterEscape"), e.enter("escapeMarker"), e.consume(t), e.exit("escapeMarker"), i;
	}
	function i(r) {
		return Ke(r) ? (e.enter("characterEscapeValue"), e.consume(r), e.exit("characterEscapeValue"), e.exit("characterEscape"), t) : n(r);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/character-reference.js
var xt = {
	name: "characterReference",
	tokenize: St
};
function St(e, t, n) {
	let r = this, i = 0, a, o;
	return s;
	function s(t) {
		return e.enter("characterReference"), e.enter("characterReferenceMarker"), e.consume(t), e.exit("characterReferenceMarker"), c;
	}
	function c(t) {
		return t === 35 ? (e.enter("characterReferenceMarkerNumeric"), e.consume(t), e.exit("characterReferenceMarkerNumeric"), l) : (e.enter("characterReferenceValue"), a = 31, o = Ve, u(t));
	}
	function l(t) {
		return t === 88 || t === 120 ? (e.enter("characterReferenceMarkerHexadecimal"), e.consume(t), e.exit("characterReferenceMarkerHexadecimal"), e.enter("characterReferenceValue"), a = 6, o = Ge, u) : (e.enter("characterReferenceValue"), a = 7, o = We, u(t));
	}
	function u(s) {
		if (s === 59 && i) {
			let i = e.exit("characterReferenceValue");
			return o === Ve && !Me(r.sliceSerialize(i)) ? n(s) : (e.enter("characterReferenceMarker"), e.consume(s), e.exit("characterReferenceMarker"), e.exit("characterReference"), t);
		}
		return o(s) && i++ < a ? (e.consume(s), u) : n(s);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/partial-non-lazy-continuation.js
var Ct = {
	partial: !0,
	tokenize: wt
};
function wt(e, t, n) {
	let r = this;
	return i;
	function i(t) {
		return t === null ? n(t) : (e.enter("lineEnding"), e.consume(t), e.exit("lineEnding"), a);
	}
	function a(e) {
		return r.parser.lazy[r.now().line] ? n(e) : t(e);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/code-fenced.js
var Tt = {
	concrete: !0,
	name: "codeFenced",
	tokenize: Et
};
function Et(e, t, n) {
	let r = this, i = {
		partial: !0,
		tokenize: x
	}, a = 0, o = 0, s;
	return c;
	function c(e) {
		return l(e);
	}
	function l(t) {
		let n = r.events[r.events.length - 1];
		return a = n && n[1].type === "linePrefix" ? n[2].sliceSerialize(n[1], !0).length : 0, s = t, e.enter("codeFenced"), e.enter("codeFencedFence"), e.enter("codeFencedFenceSequence"), u(t);
	}
	function u(t) {
		return t === s ? (o++, e.consume(t), u) : o < 3 ? n(t) : (e.exit("codeFencedFenceSequence"), A(t) ? j(e, d, "whitespace")(t) : d(t));
	}
	function d(n) {
		return n === null || O(n) ? (e.exit("codeFencedFence"), r.interrupt ? t(n) : e.check(Ct, h, b)(n)) : (e.enter("codeFencedFenceInfo"), e.enter("chunkString", { contentType: "string" }), f(n));
	}
	function f(t) {
		return t === null || O(t) ? (e.exit("chunkString"), e.exit("codeFencedFenceInfo"), d(t)) : A(t) ? (e.exit("chunkString"), e.exit("codeFencedFenceInfo"), j(e, p, "whitespace")(t)) : t === 96 && t === s ? n(t) : (e.consume(t), f);
	}
	function p(t) {
		return t === null || O(t) ? d(t) : (e.enter("codeFencedFenceMeta"), e.enter("chunkString", { contentType: "string" }), m(t));
	}
	function m(t) {
		return t === null || O(t) ? (e.exit("chunkString"), e.exit("codeFencedFenceMeta"), d(t)) : t === 96 && t === s ? n(t) : (e.consume(t), m);
	}
	function h(t) {
		return e.attempt(i, b, g)(t);
	}
	function g(t) {
		return e.enter("lineEnding"), e.consume(t), e.exit("lineEnding"), _;
	}
	function _(t) {
		return a > 0 && A(t) ? j(e, v, "linePrefix", a + 1)(t) : v(t);
	}
	function v(t) {
		return t === null || O(t) ? e.check(Ct, h, b)(t) : (e.enter("codeFlowValue"), y(t));
	}
	function y(t) {
		return t === null || O(t) ? (e.exit("codeFlowValue"), v(t)) : (e.consume(t), y);
	}
	function b(n) {
		return e.exit("codeFenced"), t(n);
	}
	function x(e, t, n) {
		let i = 0;
		return a;
		function a(t) {
			return e.enter("lineEnding"), e.consume(t), e.exit("lineEnding"), c;
		}
		function c(t) {
			return e.enter("codeFencedFence"), A(t) ? j(e, l, "linePrefix", r.parser.constructs.disable.null.includes("codeIndented") ? void 0 : 4)(t) : l(t);
		}
		function l(t) {
			return t === s ? (e.enter("codeFencedFenceSequence"), u(t)) : n(t);
		}
		function u(t) {
			return t === s ? (i++, e.consume(t), u) : i >= o ? (e.exit("codeFencedFenceSequence"), A(t) ? j(e, d, "whitespace")(t) : d(t)) : n(t);
		}
		function d(r) {
			return r === null || O(r) ? (e.exit("codeFencedFence"), t(r)) : n(r);
		}
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/code-indented.js
var Dt = {
	name: "codeIndented",
	tokenize: kt
}, Ot = {
	partial: !0,
	tokenize: At
};
function kt(e, t, n) {
	return r;
	function r(t) {
		return e.enter("codeIndented"), Xe(e, i, n, "linePrefix", 4, 4)(t);
	}
	function i(t) {
		return t === null ? o(t) : O(t) ? e.attempt(Ot, i, o)(t) : (e.enter("codeFlowValue"), a(t));
	}
	function a(t) {
		return t === null || O(t) ? (e.exit("codeFlowValue"), i(t)) : (e.consume(t), a);
	}
	function o(n) {
		return e.exit("codeIndented"), t(n);
	}
}
function At(e, t, n) {
	let r = this;
	return i;
	function i(o) {
		return r.parser.lazy[r.now().line] ? n(o) : O(o) ? (e.enter("lineEnding"), e.consume(o), e.exit("lineEnding"), i) : Xe(e, t, a, "linePrefix", 4, 4)(o);
	}
	function a(e) {
		return O(e) ? i(e) : n(e);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/code-text.js
var jt = {
	name: "codeText",
	previous: Nt,
	resolve: Mt,
	tokenize: Pt
};
function Mt(e) {
	let t = e.length - 4, n = 3, r, i;
	if ((e[n][1].type === "lineEnding" || e[n][1].type === "space") && (e[t][1].type === "lineEnding" || e[t][1].type === "space")) {
		for (r = n; ++r < t;) if (e[r][1].type === "codeTextData") {
			e[n][1].type = "codeTextPadding", e[t][1].type = "codeTextPadding", n += 2, t -= 2;
			break;
		}
	}
	for (r = n - 1, t++; ++r <= t;) i === void 0 ? r !== t && e[r][1].type !== "lineEnding" && (i = r) : (r === t || e[r][1].type === "lineEnding") && (e[i][1].type = "codeTextData", r !== i + 2 && (e[i][1].end = e[r - 1][1].end, e.splice(i + 2, r - i - 2), t -= r - i - 2, r = i + 2), i = void 0);
	return e;
}
function Nt(e) {
	return e !== 96 || this.events[this.events.length - 1][1].type === "characterEscape";
}
function Pt(e, t, n) {
	let r = 0, i, a;
	return o;
	function o(t) {
		return e.enter("codeText"), e.enter("codeTextSequence"), s(t);
	}
	function s(t) {
		return t === 96 ? (e.consume(t), r++, s) : (e.exit("codeTextSequence"), c(t));
	}
	function c(t) {
		return t === null ? n(t) : t === 32 ? (e.enter("space"), e.consume(t), e.exit("space"), c) : t === 96 ? (a = e.enter("codeTextSequence"), i = 0, u(t)) : O(t) ? (e.enter("lineEnding"), e.consume(t), e.exit("lineEnding"), c) : (e.enter("codeTextData"), l(t));
	}
	function l(t) {
		return t === null || t === 32 || t === 96 || O(t) ? (e.exit("codeTextData"), c(t)) : (e.consume(t), l);
	}
	function u(n) {
		return n === 96 ? (e.consume(n), i++, u) : i === r ? (e.exit("codeTextSequence"), e.exit("codeText"), t(n)) : (a.type = "codeTextData", l(n));
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-util-subtokenize@2.1.0/node_modules/micromark-util-subtokenize/lib/splice-buffer.js
var Ft = class {
	constructor(e) {
		this.left = e ? [...e] : [], this.right = [];
	}
	get(e) {
		if (e < 0 || e >= this.left.length + this.right.length) throw RangeError("Cannot access index `" + e + "` in a splice buffer of size `" + (this.left.length + this.right.length) + "`");
		return e < this.left.length ? this.left[e] : this.right[this.right.length - e + this.left.length - 1];
	}
	get length() {
		return this.left.length + this.right.length;
	}
	shift() {
		return this.setCursor(0), this.right.pop();
	}
	slice(e, t) {
		let n = t ?? Infinity;
		return n < this.left.length ? this.left.slice(e, n) : e > this.left.length ? this.right.slice(this.right.length - n + this.left.length, this.right.length - e + this.left.length).reverse() : this.left.slice(e).concat(this.right.slice(this.right.length - n + this.left.length).reverse());
	}
	splice(e, t, n) {
		let r = t || 0;
		this.setCursor(Math.trunc(e));
		let i = this.right.splice(this.right.length - r, Infinity);
		return n && It(this.left, n), i.reverse();
	}
	pop() {
		return this.setCursor(Infinity), this.left.pop();
	}
	push(e) {
		this.setCursor(Infinity), this.left.push(e);
	}
	pushMany(e) {
		this.setCursor(Infinity), It(this.left, e);
	}
	unshift(e) {
		this.setCursor(0), this.right.push(e);
	}
	unshiftMany(e) {
		this.setCursor(0), It(this.right, e.reverse());
	}
	setCursor(e) {
		if (!(e === this.left.length || e > this.left.length && this.right.length === 0 || e < 0 && this.left.length === 0)) {
			if (e < this.left.length) {
				let t = this.left.splice(e, Infinity);
				It(this.right, t.reverse());
			} else {
				let t = this.right.splice(this.left.length + this.right.length - e, Infinity);
				It(this.left, t.reverse());
			}
		}
	}
};
function It(e, t) {
	let n = 0;
	if (t.length < 1e4) e.push(...t);
	else for (; n < t.length;) e.push(...t.slice(n, n + 1e4)), n += 1e4;
}
//#endregion
//#region node_modules/.pnpm/micromark-util-subtokenize@2.1.0/node_modules/micromark-util-subtokenize/index.js
function Lt(e) {
	let t = {}, n = -1, r, i, a, o, s, c, l, u = new Ft(e);
	for (; ++n < u.length;) {
		for (; n in t;) n = t[n];
		if (r = u.get(n), n && r[1].type === "chunkFlow" && u.get(n - 1)[1].type === "listItemPrefix" && (c = r[1]._tokenizer.events, a = 0, a < c.length && c[a][1].type === "lineEndingBlank" && (a += 2), a < c.length && c[a][1].type === "content")) for (; ++a < c.length && c[a][1].type !== "content";) c[a][1].type === "chunkText" && (c[a][1]._isInFirstContentOfListItem = !0, a++);
		if (r[0] === "enter") r[1].contentType && (Object.assign(t, Rt(u, n)), n = t[n], l = !0);
		else if (r[1]._container) {
			for (a = n, i = void 0; a--;) if (o = u.get(a), o[1].type === "lineEnding" || o[1].type === "lineEndingBlank") o[0] === "enter" && (i && (u.get(i)[1].type = "lineEndingBlank"), o[1].type = "lineEnding", i = a);
			else if (o[1].type !== "linePrefix" && o[1].type !== "listItemIndent") break;
			i && (r[1].end = { ...u.get(i)[1].start }, s = u.slice(i, n), s.unshift(r), u.splice(i, n - i + 1, s));
		}
	}
	return Ne(e, 0, Infinity, u.slice(0)), !l;
}
function Rt(e, t) {
	let n = e.get(t)[1], r = e.get(t)[2], i = t - 1, a = [], o = n._tokenizer;
	o || (o = r.parser[n.contentType](n.start), n._contentTypeTextTrailing && (o._contentTypeTextTrailing = !0));
	let s = o.events, c = [], l = {}, u, d, f = -1, p = n, m = 0, h = 0, g = [h];
	for (; p;) {
		for (; e.get(++i)[1] !== p;);
		a.push(i), p._tokenizer || (u = r.sliceStream(p), p.next || u.push(null), d && o.defineSkip(p.start), p._isInFirstContentOfListItem && (o._gfmTasklistFirstContentOfListItem = !0), o.write(u), p._isInFirstContentOfListItem && (o._gfmTasklistFirstContentOfListItem = void 0)), d = p, p = p.next;
	}
	for (p = n; ++f < s.length;) s[f][0] === "exit" && s[f - 1][0] === "enter" && s[f][1].type === s[f - 1][1].type && s[f][1].start.line !== s[f][1].end.line && (h = f + 1, g.push(h), p._tokenizer = void 0, p.previous = void 0, p = p.next);
	for (o.events = [], p ? (p._tokenizer = void 0, p.previous = void 0) : g.pop(), f = g.length; f--;) {
		let t = s.slice(g[f], g[f + 1]), n = a.pop();
		c.push([n, n + t.length - 1]), e.splice(n, 2, t);
	}
	for (c.reverse(), f = -1; ++f < c.length;) l[m + c[f][0]] = m + c[f][1], m += c[f][1] - c[f][0] - 1;
	return l;
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/content.js
var zt = {
	resolve: Vt,
	tokenize: Ht
}, Bt = {
	partial: !0,
	tokenize: Ut
};
function Vt(e) {
	return Lt(e), e;
}
function Ht(e, t) {
	let n;
	return r;
	function r(t) {
		return e.enter("content"), n = e.enter("chunkContent", { contentType: "content" }), i(t);
	}
	function i(t) {
		return t === null ? a(t) : O(t) ? e.check(Bt, o, a)(t) : (e.consume(t), i);
	}
	function a(n) {
		return e.exit("chunkContent"), e.exit("content"), t(n);
	}
	function o(t) {
		return e.consume(t), e.exit("chunkContent"), n.next = e.enter("chunkContent", {
			contentType: "content",
			previous: n
		}), n = n.next, i;
	}
}
function Ut(e, t, n) {
	let r = this;
	return i;
	function i(t) {
		return e.exit("chunkContent"), e.enter("lineEnding"), e.consume(t), e.exit("lineEnding"), j(e, a, "linePrefix");
	}
	function a(i) {
		if (i === null || O(i)) return n(i);
		let a = r.events[r.events.length - 1];
		return !r.parser.constructs.disable.null.includes("codeIndented") && a && a[1].type === "linePrefix" && a[2].sliceSerialize(a[1], !0).length >= 4 ? t(i) : e.interrupt(r.parser.constructs.flow, n, t)(i);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-factory-destination@2.0.1/node_modules/micromark-factory-destination/index.js
function Wt(e, t, n, r, i, a, o, s, c) {
	let l = c || Infinity, u = 0;
	return d;
	function d(t) {
		return t === 60 ? (e.enter(r), e.enter(i), e.enter(a), e.consume(t), e.exit(a), f) : t === null || t === 32 || t === 41 || Ue(t) ? n(t) : (e.enter(r), e.enter(o), e.enter(s), e.enter("chunkString", { contentType: "string" }), h(t));
	}
	function f(n) {
		return n === 62 ? (e.enter(a), e.consume(n), e.exit(a), e.exit(i), e.exit(r), t) : (e.enter(s), e.enter("chunkString", { contentType: "string" }), p(n));
	}
	function p(t) {
		return t === 62 ? (e.exit("chunkString"), e.exit(s), f(t)) : t === null || t === 60 || O(t) ? n(t) : (e.consume(t), t === 92 ? m : p);
	}
	function m(t) {
		return t === 60 || t === 62 || t === 92 ? (e.consume(t), p) : p(t);
	}
	function h(i) {
		return !u && (i === null || i === 41 || k(i)) ? (e.exit("chunkString"), e.exit(s), e.exit(o), e.exit(r), t(i)) : u < l && i === 40 ? (e.consume(i), u++, h) : i === 41 ? (e.consume(i), u--, h) : i === null || i === 32 || i === 40 || Ue(i) ? n(i) : (e.consume(i), i === 92 ? g : h);
	}
	function g(t) {
		return t === 40 || t === 41 || t === 92 ? (e.consume(t), h) : h(t);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-factory-label@2.0.1/node_modules/micromark-factory-label/index.js
function Gt(e, t, n, r, i, a) {
	let o = this, s = 0, c;
	return l;
	function l(t) {
		return e.enter(r), e.enter(i), e.consume(t), e.exit(i), e.enter(a), u;
	}
	function u(l) {
		return s > 999 || l === null || l === 91 || l === 93 && !c || 
		/* c8 ignore next 3 */
		l === 94 && !s && "_hiddenFootnoteSupport" in o.parser.constructs ? n(l) : l === 93 ? (e.exit(a), e.enter(i), e.consume(l), e.exit(i), e.exit(r), t) : O(l) ? (e.enter("lineEnding"), e.consume(l), e.exit("lineEnding"), u) : (e.enter("chunkString", { contentType: "string" }), d(l));
	}
	function d(t) {
		return t === null || t === 91 || t === 93 || O(t) || s++ > 999 ? (e.exit("chunkString"), u(t)) : (e.consume(t), c ||= !A(t), t === 92 ? f : d);
	}
	function f(t) {
		return t === 91 || t === 92 || t === 93 ? (e.consume(t), s++, d) : d(t);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-factory-title@2.0.1/node_modules/micromark-factory-title/index.js
function Kt(e, t, n, r, i, a) {
	let o;
	return s;
	function s(t) {
		return t === 34 || t === 39 || t === 40 ? (e.enter(r), e.enter(i), e.consume(t), e.exit(i), o = t === 40 ? 41 : t, c) : n(t);
	}
	function c(n) {
		return n === o ? (e.enter(i), e.consume(n), e.exit(i), e.exit(r), t) : (e.enter(a), l(n));
	}
	function l(t) {
		return t === o ? (e.exit(a), c(o)) : t === null ? n(t) : O(t) ? (e.enter("lineEnding"), e.consume(t), e.exit("lineEnding"), j(e, l, "linePrefix")) : (e.enter("chunkString", { contentType: "string" }), u(t));
	}
	function u(t) {
		return t === o || t === null || O(t) ? (e.exit("chunkString"), l(t)) : (e.consume(t), t === 92 ? d : u);
	}
	function d(t) {
		return t === o || t === 92 ? (e.consume(t), u) : u(t);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-factory-whitespace@2.0.1/node_modules/micromark-factory-whitespace/index.js
function qt(e, t) {
	let n;
	return r;
	function r(i) {
		return O(i) ? (e.enter("lineEnding"), e.consume(i), e.exit("lineEnding"), n = !0, r) : A(i) ? j(e, r, n ? "linePrefix" : "lineSuffix")(i) : t(i);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/definition.js
var Jt = {
	name: "definition",
	tokenize: Xt
}, Yt = {
	partial: !0,
	tokenize: Zt
};
function Xt(e, t, n) {
	let r = this, i;
	return a;
	function a(t) {
		return e.enter("definition"), o(t);
	}
	function o(t) {
		return Gt.call(r, e, s, n, "definitionLabel", "definitionLabelMarker", "definitionLabelString")(t);
	}
	function s(t) {
		return i = Be(r.sliceSerialize(r.events[r.events.length - 1][1]).slice(1, -1)), t === 58 ? (e.enter("definitionMarker"), e.consume(t), e.exit("definitionMarker"), c) : n(t);
	}
	function c(t) {
		return k(t) ? qt(e, l)(t) : l(t);
	}
	function l(t) {
		return Wt(e, u, n, "definitionDestination", "definitionDestinationLiteral", "definitionDestinationLiteralMarker", "definitionDestinationRaw", "definitionDestinationString")(t);
	}
	function u(t) {
		return e.attempt(Yt, d, d)(t);
	}
	function d(t) {
		return A(t) ? j(e, f, "whitespace")(t) : f(t);
	}
	function f(a) {
		return a === null || O(a) ? (e.exit("definition"), r.parser.defined.push(i), t(a)) : n(a);
	}
}
function Zt(e, t, n) {
	return r;
	function r(t) {
		return k(t) ? qt(e, i)(t) : n(t);
	}
	function i(t) {
		return Kt(e, a, n, "definitionTitle", "definitionTitleMarker", "definitionTitleString")(t);
	}
	function a(t) {
		return A(t) ? j(e, o, "whitespace")(t) : o(t);
	}
	function o(e) {
		return e === null || O(e) ? t(e) : n(e);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/hard-break-escape.js
var Qt = {
	name: "hardBreakEscape",
	tokenize: $t
};
function $t(e, t, n) {
	return r;
	function r(t) {
		return e.enter("hardBreakEscape"), e.consume(t), i;
	}
	function i(r) {
		return O(r) ? (e.exit("hardBreakEscape"), t(r)) : n(r);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/heading-atx.js
var en = {
	name: "headingAtx",
	resolve: tn,
	tokenize: nn
};
function tn(e, t) {
	let n = e.length - 2, r = 3;
	if (e[r][1].type === "whitespace" && (r += 2), n - 2 > r && e[n][1].type === "whitespace" && (n -= 2), e[n][1].type === "atxHeadingSequence" && (r === n - 1 || n - 4 > r && e[n - 2][1].type === "whitespace") && (n -= r + 1 === n ? 2 : 4), n > r) {
		let i = {
			type: "atxHeadingText",
			start: e[r][1].start,
			end: e[n][1].end
		}, a = {
			type: "chunkText",
			start: e[r][1].start,
			end: e[n][1].end,
			contentType: "text"
		};
		Ne(e, r, n - r + 1, [
			[
				"enter",
				i,
				t
			],
			[
				"enter",
				a,
				t
			],
			[
				"exit",
				a,
				t
			],
			[
				"exit",
				i,
				t
			]
		]);
	}
	return e;
}
function nn(e, t, n) {
	let r = 0;
	return i;
	function i(t) {
		return e.enter("atxHeading"), a(t);
	}
	function a(t) {
		return e.enter("atxHeadingSequence"), o(t);
	}
	function o(t) {
		return t === 35 && r++ < 6 ? (e.consume(t), o) : t === null || k(t) ? (e.exit("atxHeadingSequence"), s(t)) : n(t);
	}
	function s(n) {
		return n === 35 ? (e.enter("atxHeadingSequence"), c(n)) : n === null || O(n) ? (e.exit("atxHeading"), t(n)) : A(n) ? j(e, s, "whitespace")(n) : (e.enter("atxHeadingText"), l(n));
	}
	function c(t) {
		return t === 35 ? (e.consume(t), c) : (e.exit("atxHeadingSequence"), s(t));
	}
	function l(t) {
		return t === null || t === 35 || k(t) ? (e.exit("atxHeadingText"), s(t)) : (e.consume(t), l);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-util-html-tag-name@2.0.1/node_modules/micromark-util-html-tag-name/index.js
var rn = /* @__PURE__ */ "address.article.aside.base.basefont.blockquote.body.caption.center.col.colgroup.dd.details.dialog.dir.div.dl.dt.fieldset.figcaption.figure.footer.form.frame.frameset.h1.h2.h3.h4.h5.h6.head.header.hr.html.iframe.legend.li.link.main.menu.menuitem.nav.noframes.ol.optgroup.option.p.param.search.section.summary.table.tbody.td.tfoot.th.thead.title.tr.track.ul".split("."), an = [
	"pre",
	"script",
	"style",
	"textarea"
], on = {
	concrete: !0,
	name: "htmlFlow",
	resolveTo: cn,
	tokenize: ln
}, sn = {
	partial: !0,
	tokenize: un
};
function cn(e) {
	let t = e.length;
	for (; t-- && (e[t][0] !== "enter" || e[t][1].type !== "htmlFlow"););
	return t > 1 && e[t - 2][1].type === "linePrefix" && (e[t][1].start = e[t - 2][1].start, e[t + 1][1].start = e[t - 2][1].start, e.splice(t - 2, 2)), e;
}
function ln(e, t, n) {
	let r = this, i, a, o, s, c;
	return l;
	function l(e) {
		return u(e);
	}
	function u(t) {
		return e.enter("htmlFlow"), e.enter("htmlFlowData"), e.consume(t), d;
	}
	function d(s) {
		return s === 33 ? (e.consume(s), f) : s === 47 ? (e.consume(s), a = !0, h) : s === 63 ? (e.consume(s), i = 3, r.interrupt ? t : ue) : D(s) ? (e.consume(s), o = String.fromCharCode(s), g) : n(s);
	}
	function f(a) {
		return a === 45 ? (e.consume(a), i = 2, p) : a === 91 ? (e.consume(a), i = 5, s = 0, m) : D(a) ? (e.consume(a), i = 4, r.interrupt ? t : ue) : n(a);
	}
	function p(i) {
		return i === 45 ? (e.consume(i), r.interrupt ? t : ue) : n(i);
	}
	function m(i) {
		return i === "CDATA[".charCodeAt(s++) ? (e.consume(i), s === 6 ? r.interrupt ? t : T : m) : n(i);
	}
	function h(t) {
		return D(t) ? (e.consume(t), o = String.fromCharCode(t), g) : n(t);
	}
	function g(s) {
		if (s === null || s === 47 || s === 62 || k(s)) {
			let c = s === 47, l = o.toLowerCase();
			return !c && !a && an.includes(l) ? (i = 1, r.interrupt ? t(s) : T(s)) : rn.includes(o.toLowerCase()) ? (i = 6, c ? (e.consume(s), _) : r.interrupt ? t(s) : T(s)) : (i = 7, r.interrupt && !r.parser.lazy[r.now().line] ? n(s) : a ? v(s) : y(s));
		}
		return s === 45 || Ve(s) ? (e.consume(s), o += String.fromCharCode(s), g) : n(s);
	}
	function _(i) {
		return i === 62 ? (e.consume(i), r.interrupt ? t : T) : n(i);
	}
	function v(t) {
		return A(t) ? (e.consume(t), v) : C(t);
	}
	function y(t) {
		return t === 47 ? (e.consume(t), C) : t === 58 || t === 95 || D(t) ? (e.consume(t), b) : A(t) ? (e.consume(t), y) : C(t);
	}
	function b(t) {
		return t === 45 || t === 46 || t === 58 || t === 95 || Ve(t) ? (e.consume(t), b) : x(t);
	}
	function x(t) {
		return t === 61 ? (e.consume(t), ee) : A(t) ? (e.consume(t), x) : y(t);
	}
	function ee(t) {
		return t === null || t === 60 || t === 61 || t === 62 || t === 96 ? n(t) : t === 34 || t === 39 ? (e.consume(t), c = t, S) : A(t) ? (e.consume(t), ee) : te(t);
	}
	function S(t) {
		return t === c ? (e.consume(t), c = null, ne) : t === null || O(t) ? n(t) : (e.consume(t), S);
	}
	function te(t) {
		return t === null || t === 34 || t === 39 || t === 47 || t === 60 || t === 61 || t === 62 || t === 96 || k(t) ? x(t) : (e.consume(t), te);
	}
	function ne(e) {
		return e === 47 || e === 62 || A(e) ? y(e) : n(e);
	}
	function C(t) {
		return t === 62 ? (e.consume(t), w) : n(t);
	}
	function w(t) {
		return t === null || O(t) ? T(t) : A(t) ? (e.consume(t), w) : n(t);
	}
	function T(t) {
		return t === 45 && i === 2 ? (e.consume(t), oe) : t === 60 && i === 1 ? (e.consume(t), se) : t === 62 && i === 4 ? (e.consume(t), de) : t === 63 && i === 3 ? (e.consume(t), ue) : t === 93 && i === 5 ? (e.consume(t), le) : O(t) && (i === 6 || i === 7) ? (e.exit("htmlFlowData"), e.check(sn, fe, re)(t)) : t === null || O(t) ? (e.exit("htmlFlowData"), re(t)) : (e.consume(t), T);
	}
	function re(t) {
		return e.check(Ct, ie, fe)(t);
	}
	function ie(t) {
		return e.enter("lineEnding"), e.consume(t), e.exit("lineEnding"), ae;
	}
	function ae(t) {
		return t === null || O(t) ? re(t) : (e.enter("htmlFlowData"), T(t));
	}
	function oe(t) {
		return t === 45 ? (e.consume(t), ue) : T(t);
	}
	function se(t) {
		return t === 47 ? (e.consume(t), o = "", ce) : T(t);
	}
	function ce(t) {
		if (t === 62) {
			let n = o.toLowerCase();
			return an.includes(n) ? (e.consume(t), de) : T(t);
		}
		return D(t) && o.length < 8 ? (e.consume(t), o += String.fromCharCode(t), ce) : T(t);
	}
	function le(t) {
		return t === 93 ? (e.consume(t), ue) : T(t);
	}
	function ue(t) {
		return t === 62 ? (e.consume(t), de) : t === 45 && i === 2 ? (e.consume(t), ue) : T(t);
	}
	function de(t) {
		return t === null || O(t) ? (e.exit("htmlFlowData"), fe(t)) : (e.consume(t), de);
	}
	function fe(n) {
		return e.exit("htmlFlow"), t(n);
	}
}
function un(e, t, n) {
	return r;
	function r(r) {
		return e.enter("lineEnding"), e.consume(r), e.exit("lineEnding"), e.attempt(pt, t, n);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/html-text.js
var dn = {
	name: "htmlText",
	tokenize: fn
};
function fn(e, t, n) {
	let r = this, i, a, o;
	return s;
	function s(t) {
		return e.enter("htmlText"), e.enter("htmlTextData"), e.consume(t), c;
	}
	function c(t) {
		return t === 33 ? (e.consume(t), l) : t === 47 ? (e.consume(t), x) : t === 63 ? (e.consume(t), y) : D(t) ? (e.consume(t), te) : n(t);
	}
	function l(t) {
		return t === 45 ? (e.consume(t), u) : t === 91 ? (e.consume(t), a = 0, m) : D(t) ? (e.consume(t), v) : n(t);
	}
	function u(t) {
		return t === 45 ? (e.consume(t), p) : n(t);
	}
	function d(t) {
		return t === null ? n(t) : t === 45 ? (e.consume(t), f) : O(t) ? (o = d, se(t)) : (e.consume(t), d);
	}
	function f(t) {
		return t === 45 ? (e.consume(t), p) : d(t);
	}
	function p(e) {
		return e === 62 ? oe(e) : e === 45 ? f(e) : d(e);
	}
	function m(t) {
		return t === "CDATA[".charCodeAt(a++) ? (e.consume(t), a === 6 ? h : m) : n(t);
	}
	function h(t) {
		return t === null ? n(t) : t === 93 ? (e.consume(t), g) : O(t) ? (o = h, se(t)) : (e.consume(t), h);
	}
	function g(t) {
		return t === 93 ? (e.consume(t), _) : h(t);
	}
	function _(t) {
		return t === 62 ? oe(t) : t === 93 ? (e.consume(t), _) : h(t);
	}
	function v(t) {
		return t === null || t === 62 ? oe(t) : O(t) ? (o = v, se(t)) : (e.consume(t), v);
	}
	function y(t) {
		return t === null ? n(t) : t === 63 ? (e.consume(t), b) : O(t) ? (o = y, se(t)) : (e.consume(t), y);
	}
	function b(e) {
		return e === 62 ? oe(e) : y(e);
	}
	function x(t) {
		return D(t) ? (e.consume(t), ee) : n(t);
	}
	function ee(t) {
		return t === 45 || Ve(t) ? (e.consume(t), ee) : S(t);
	}
	function S(t) {
		return O(t) ? (o = S, se(t)) : A(t) ? (e.consume(t), S) : oe(t);
	}
	function te(t) {
		return t === 45 || Ve(t) ? (e.consume(t), te) : t === 47 || t === 62 || k(t) ? ne(t) : n(t);
	}
	function ne(t) {
		return t === 47 ? (e.consume(t), oe) : t === 58 || t === 95 || D(t) ? (e.consume(t), C) : O(t) ? (o = ne, se(t)) : A(t) ? (e.consume(t), ne) : oe(t);
	}
	function C(t) {
		return t === 45 || t === 46 || t === 58 || t === 95 || Ve(t) ? (e.consume(t), C) : w(t);
	}
	function w(t) {
		return t === 61 ? (e.consume(t), T) : O(t) ? (o = w, se(t)) : A(t) ? (e.consume(t), w) : ne(t);
	}
	function T(t) {
		return t === null || t === 60 || t === 61 || t === 62 || t === 96 ? n(t) : t === 34 || t === 39 ? (e.consume(t), i = t, re) : O(t) ? (o = T, se(t)) : A(t) ? (e.consume(t), T) : (e.consume(t), ie);
	}
	function re(t) {
		return t === i ? (e.consume(t), i = void 0, ae) : t === null ? n(t) : O(t) ? (o = re, se(t)) : (e.consume(t), re);
	}
	function ie(t) {
		return t === null || t === 34 || t === 39 || t === 60 || t === 61 || t === 96 ? n(t) : t === 47 || t === 62 || k(t) ? ne(t) : (e.consume(t), ie);
	}
	function ae(e) {
		return e === 47 || e === 62 || k(e) ? ne(e) : n(e);
	}
	function oe(r) {
		return r === 62 ? (e.consume(r), e.exit("htmlTextData"), e.exit("htmlText"), t) : n(r);
	}
	function se(t) {
		return e.exit("htmlTextData"), e.enter("lineEnding"), e.consume(t), e.exit("lineEnding"), ce;
	}
	function ce(t) {
		return A(t) ? j(e, le, "linePrefix", r.parser.constructs.disable.null.includes("codeIndented") ? void 0 : 4)(t) : le(t);
	}
	function le(t) {
		return e.enter("htmlTextData"), o(t);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/label-end.js
var pn = {
	name: "labelEnd",
	resolveAll: _n,
	resolveTo: vn,
	tokenize: yn
}, mn = { tokenize: bn }, hn = { tokenize: xn }, gn = { tokenize: Sn };
function _n(e) {
	let t = -1, n = [];
	for (; ++t < e.length;) {
		let r = e[t][1];
		if (n.push(e[t]), r.type === "labelImage" || r.type === "labelLink" || r.type === "labelEnd") {
			let e = r.type === "labelImage" ? 4 : 2;
			r.type = "data", t += e;
		}
	}
	return e.length !== n.length && Ne(e, 0, e.length, n), e;
}
function vn(e, t) {
	let n = e.length, r = 0, i, a, o;
	for (; n--;) {
		let t = e[n][1];
		if (i) {
			if (t.type === "link" || t.type === "labelLink" && t._inactive) break;
			e[n][0] === "enter" && t.type === "labelLink" && (t._inactive = !0);
		} else if (a) {
			if (e[n][0] === "enter" && (t.type === "labelImage" || t.type === "labelLink") && !t._balanced && (i = n, t.type !== "labelLink")) {
				r = 2;
				break;
			}
		} else t.type === "labelEnd" && (a = n);
	}
	let s = {
		type: e[i][1].type === "labelLink" ? "link" : "image",
		start: { ...e[i][1].start },
		end: { ...e[e.length - 1][1].end }
	}, c = {
		type: "label",
		start: { ...e[i][1].start },
		end: { ...e[a][1].end }
	}, l = {
		type: "labelText",
		start: { ...e[i + r + 2][1].end },
		end: { ...e[a - 2][1].start }
	};
	return o = [[
		"enter",
		s,
		t
	], [
		"enter",
		c,
		t
	]], o = Pe(o, e.slice(i + 1, i + r + 3)), o = Pe(o, [[
		"enter",
		l,
		t
	]]), o = Pe(o, ot(t.parser.constructs.insideSpan.null, e.slice(i + r + 4, a - 3), t)), o = Pe(o, [
		[
			"exit",
			l,
			t
		],
		e[a - 2],
		e[a - 1],
		[
			"exit",
			c,
			t
		]
	]), o = Pe(o, e.slice(a + 1)), o = Pe(o, [[
		"exit",
		s,
		t
	]]), Ne(e, i, e.length, o), e;
}
function yn(e, t, n) {
	let r = this, i = r._labelStarts, a, o;
	if (i) {
		for (; i.length > 0 && i[i.length - 1]._balanced;) i.pop();
		a = i[i.length - 1];
	}
	return s;
	function s(t) {
		return a ? a._inactive ? d(t) : (o = r.parser.defined.includes(Be(r.sliceSerialize({
			start: a.end,
			end: r.now()
		}))), e.enter("labelEnd"), e.enter("labelMarker"), e.consume(t), e.exit("labelMarker"), e.exit("labelEnd"), c) : n(t);
	}
	function c(t) {
		return t === 40 ? e.attempt(mn, u, o ? u : d)(t) : t === 91 ? e.attempt(hn, u, o ? l : d)(t) : o ? u(t) : d(t);
	}
	function l(t) {
		return e.attempt(gn, u, d)(t);
	}
	function u(e) {
		return i.pop(), t(e);
	}
	function d(e) {
		return a._balanced = !0, n(e);
	}
}
function bn(e, t, n) {
	return r;
	function r(t) {
		return e.enter("resource"), e.enter("resourceMarker"), e.consume(t), e.exit("resourceMarker"), i;
	}
	function i(t) {
		return k(t) ? qt(e, a)(t) : a(t);
	}
	function a(t) {
		return t === 41 ? u(t) : Wt(e, o, s, "resourceDestination", "resourceDestinationLiteral", "resourceDestinationLiteralMarker", "resourceDestinationRaw", "resourceDestinationString", 32)(t);
	}
	function o(t) {
		return k(t) ? qt(e, c)(t) : u(t);
	}
	function s(e) {
		return n(e);
	}
	function c(t) {
		return t === 34 || t === 39 || t === 40 ? Kt(e, l, n, "resourceTitle", "resourceTitleMarker", "resourceTitleString")(t) : u(t);
	}
	function l(t) {
		return k(t) ? qt(e, u)(t) : u(t);
	}
	function u(r) {
		return r === 41 ? (e.enter("resourceMarker"), e.consume(r), e.exit("resourceMarker"), e.exit("resource"), t) : n(r);
	}
}
function xn(e, t, n) {
	let r = this;
	return i;
	function i(t) {
		return Gt.call(r, e, a, o, "reference", "referenceMarker", "referenceString")(t);
	}
	function a(e) {
		return r.parser.defined.includes(Be(r.sliceSerialize(r.events[r.events.length - 1][1]).slice(1, -1))) ? t(e) : n(e);
	}
	function o(e) {
		return n(e);
	}
}
function Sn(e, t, n) {
	return r;
	function r(t) {
		return e.enter("reference"), e.enter("referenceMarker"), e.consume(t), e.exit("referenceMarker"), i;
	}
	function i(r) {
		return r === 93 ? (e.enter("referenceMarker"), e.consume(r), e.exit("referenceMarker"), e.exit("reference"), t) : n(r);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/label-start-image.js
var Cn = {
	name: "labelStartImage",
	resolveAll: pn.resolveAll,
	tokenize: wn
};
function wn(e, t, n) {
	let r = this, i;
	return a;
	function a(t) {
		return e.enter("labelImage"), e.enter("labelImageMarker"), e.consume(t), e.exit("labelImageMarker"), o;
	}
	function o(t) {
		return t === 91 ? (e.enter("labelMarker"), e.consume(t), e.exit("labelMarker"), i = e.exit("labelImage"), s) : n(t);
	}
	function s(e) {
		return e === 94 && "_hiddenFootnoteSupport" in r.parser.constructs ? n(e) : (r._labelStarts = r._labelStarts || [], r._labelStarts.push(i), t(e));
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/label-start-link.js
var Tn = {
	name: "labelStartLink",
	resolveAll: pn.resolveAll,
	tokenize: En
};
function En(e, t, n) {
	let r = this, i;
	return a;
	function a(t) {
		return e.enter("labelLink"), e.enter("labelMarker"), e.consume(t), e.exit("labelMarker"), i = e.exit("labelLink"), o;
	}
	function o(e) {
		return e === 94 && "_hiddenFootnoteSupport" in r.parser.constructs ? n(e) : (r._labelStarts = r._labelStarts || [], r._labelStarts.push(i), t(e));
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/line-ending.js
var Dn = {
	name: "lineEnding",
	tokenize: On
};
function On(e, t) {
	return n;
	function n(n) {
		return e.enter("lineEnding"), e.consume(n), e.exit("lineEnding"), j(e, t, "linePrefix");
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/thematic-break.js
var kn = {
	name: "thematicBreak",
	tokenize: An
};
function An(e, t, n) {
	let r = 0, i;
	return a;
	function a(t) {
		return e.enter("thematicBreak"), o(t);
	}
	function o(e) {
		return i = e, s(e);
	}
	function s(a) {
		return a === i ? (e.enter("thematicBreakSequence"), c(a)) : r >= 3 && (a === null || O(a)) ? (e.exit("thematicBreak"), t(a)) : n(a);
	}
	function c(t) {
		return t === i ? (e.consume(t), r++, c) : (e.exit("thematicBreakSequence"), A(t) ? j(e, s, "whitespace")(t) : s(t));
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/list.js
var jn = {
	continuation: { tokenize: Fn },
	exit: Ln,
	name: "list",
	tokenize: Pn
}, Mn = {
	partial: !0,
	tokenize: Rn
}, Nn = {
	partial: !0,
	tokenize: In
};
function Pn(e, t, n) {
	let r = this, i = r.events[r.events.length - 1], a = i && i[1].type === "linePrefix" ? i[2].sliceSerialize(i[1], !0).length : 0, o = 0;
	return s;
	function s(t) {
		let i = r.containerState.type || (t === 42 || t === 43 || t === 45 ? "listUnordered" : "listOrdered");
		if (i === "listUnordered" ? !r.containerState.marker || t === r.containerState.marker : We(t)) {
			if (r.containerState.type || (r.containerState.type = i, e.enter(i, { _container: !0 })), i === "listUnordered") return e.enter("listItemPrefix"), t === 42 || t === 45 ? e.check(kn, n, l)(t) : l(t);
			if (!r.interrupt || t === 49) return e.enter("listItemPrefix"), e.enter("listItemValue"), c(t);
		}
		return n(t);
	}
	function c(t) {
		return We(t) && ++o < 10 ? (e.consume(t), c) : (!r.interrupt || o < 2) && (r.containerState.marker ? t === r.containerState.marker : t === 41 || t === 46) ? (e.exit("listItemValue"), l(t)) : n(t);
	}
	function l(t) {
		return e.enter("listItemMarker"), e.consume(t), e.exit("listItemMarker"), r.containerState.marker = r.containerState.marker || t, e.check(pt, r.interrupt ? n : u, e.attempt(Mn, f, d));
	}
	function u(e) {
		return r.containerState.initialBlankLine = !0, a++, f(e);
	}
	function d(t) {
		return A(t) ? (e.enter("listItemPrefixWhitespace"), e.consume(t), e.exit("listItemPrefixWhitespace"), f) : n(t);
	}
	function f(n) {
		return r.containerState.size = a + r.sliceSerialize(e.exit("listItemPrefix"), !0).length, t(n);
	}
}
function Fn(e, t, n) {
	let r = this;
	return r.containerState._closeFlow = void 0, e.check(pt, i, a);
	function i(n) {
		return r.containerState.furtherBlankLines = r.containerState.furtherBlankLines || r.containerState.initialBlankLine, j(e, t, "listItemIndent", r.containerState.size + 1)(n);
	}
	function a(n) {
		return r.containerState.furtherBlankLines || !A(n) ? (r.containerState.furtherBlankLines = void 0, r.containerState.initialBlankLine = void 0, o(n)) : (r.containerState.furtherBlankLines = void 0, r.containerState.initialBlankLine = void 0, e.attempt(Nn, t, o)(n));
	}
	function o(i) {
		return r.containerState._closeFlow = !0, r.interrupt = void 0, j(e, e.attempt(jn, t, n), "linePrefix", r.parser.constructs.disable.null.includes("codeIndented") ? void 0 : 4)(i);
	}
}
function In(e, t, n) {
	let r = this;
	return j(e, i, "listItemIndent", r.containerState.size + 1);
	function i(e) {
		let i = r.events[r.events.length - 1];
		return i && i[1].type === "listItemIndent" && i[2].sliceSerialize(i[1], !0).length === r.containerState.size ? t(e) : n(e);
	}
}
function Ln(e) {
	e.exit(this.containerState.type);
}
function Rn(e, t, n) {
	let r = this;
	return j(e, i, "listItemPrefixWhitespace", r.parser.constructs.disable.null.includes("codeIndented") ? void 0 : 5);
	function i(e) {
		let i = r.events[r.events.length - 1];
		return !A(e) && i && i[1].type === "listItemPrefixWhitespace" ? t(e) : n(e);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-core-commonmark@2.0.4/node_modules/micromark-core-commonmark/lib/setext-underline.js
var zn = {
	name: "setextUnderline",
	resolveTo: Bn,
	tokenize: Vn
};
function Bn(e, t) {
	let n = new $e(), r = e.length, i, a, o;
	for (; r--;) if (e[r][0] === "enter") {
		if (e[r][1].type === "content") {
			i = r;
			break;
		}
		e[r][1].type === "paragraph" && (a = r);
	} else e[r][1].type === "content" && n.add(r, 1, []), !o && e[r][1].type === "definition" && (o = r);
	let s = {
		type: "setextHeading",
		start: { ...e[i][1].start },
		end: { ...e[e.length - 1][1].end }
	};
	return e[a][1].type = "setextHeadingText", o ? (n.add(a, 0, [[
		"enter",
		s,
		t
	]]), n.add(o + 1, 0, [[
		"exit",
		e[i][1],
		t
	]]), e[i][1].end = { ...e[o][1].end }) : e[i][1] = s, n.add(e.length, 0, [[
		"exit",
		s,
		t
	]]), n.consume(e), e;
}
function Vn(e, t, n) {
	let r = this, i;
	return a;
	function a(t) {
		let a = r.events.length, s;
		for (; a--;) if (r.events[a][1].type !== "lineEnding" && r.events[a][1].type !== "linePrefix" && r.events[a][1].type !== "content") {
			s = r.events[a][1].type === "paragraph";
			break;
		}
		return !r.parser.lazy[r.now().line] && (r.interrupt || s) ? (e.enter("setextHeadingLine"), i = t, o(t)) : n(t);
	}
	function o(t) {
		return e.enter("setextHeadingLineSequence"), s(t);
	}
	function s(t) {
		return t === i ? (e.consume(t), s) : (e.exit("setextHeadingLineSequence"), A(t) ? j(e, c, "lineSuffix")(t) : c(t));
	}
	function c(r) {
		return r === null || O(r) ? (e.exit("setextHeadingLine"), t(r)) : n(r);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark@4.0.3/node_modules/micromark/lib/initialize/flow.js
var Hn = { tokenize: Un };
function Un(e) {
	let t = this, n = e.attempt(pt, r, e.attempt(this.parser.constructs.flowInitial, i, j(e, e.attempt(this.parser.constructs.flow, i, e.attempt(zt, i)), "linePrefix")));
	return n;
	function r(r) {
		if (r === null) e.consume(r);
		else return e.enter("lineEndingBlank"), e.consume(r), e.exit("lineEndingBlank"), t.currentConstruct = void 0, n;
	}
	function i(r) {
		if (r === null) e.consume(r);
		else return e.enter("lineEnding"), e.consume(r), e.exit("lineEnding"), t.currentConstruct = void 0, n;
	}
}
//#endregion
//#region node_modules/.pnpm/micromark@4.0.3/node_modules/micromark/lib/initialize/text.js
var Wn = { resolveAll: Jn() }, Gn = qn("string"), Kn = qn("text");
function qn(e) {
	return {
		resolveAll: Jn(e === "text" ? Yn : void 0),
		tokenize: t
	};
	function t(t) {
		let n = this, r = this.parser.constructs[e], i = t.attempt(r, a, o);
		return a;
		function a(e) {
			return c(e) ? i(e) : o(e);
		}
		function o(e) {
			if (e === null) t.consume(e);
			else return t.enter("data"), t.consume(e), s;
		}
		function s(e) {
			return c(e) ? (t.exit("data"), i(e)) : (t.consume(e), s);
		}
		function c(e) {
			if (e === null) return !0;
			let t = r[e], i = -1;
			if (t) for (; ++i < t.length;) {
				let e = t[i];
				if (!e.previous || e.previous.call(n, n.previous)) return !0;
			}
			return !1;
		}
	}
}
function Jn(e) {
	return t;
	function t(t, n) {
		let r = -1, i;
		for (; ++r <= t.length;) i === void 0 ? t[r] && t[r][1].type === "data" && (i = r, r++) : (!t[r] || t[r][1].type !== "data") && (r !== i + 2 && (t[i][1].end = t[r - 1][1].end, t.splice(i + 2, r - i - 2), r = i + 2), i = void 0);
		return e ? e(t, n) : t;
	}
}
function Yn(e, t) {
	let n = new $e(), r = 0;
	for (; ++r <= e.length;) if ((r === e.length || e[r][1].type === "lineEnding") && e[r - 1][1].type === "data") {
		let i = e[r - 1][1], a = t.sliceStream(i), o = a.length, s = -1, c = 0, l;
		for (; o--;) {
			let e = a[o];
			if (typeof e == "string") {
				for (s = e.length; e.charCodeAt(s - 1) === 32;) c++, s--;
				if (s) break;
				s = -1;
			} else if (e === -2) l = !0, c++;
			else if (e !== -1) {
				o++;
				break;
			}
		}
		if (t._contentTypeTextTrailing && r === e.length && (c = 0), c) {
			let a = {
				type: r === e.length || l || c < 2 ? "lineSuffix" : "hardBreakTrailing",
				start: {
					_bufferIndex: o ? s : i.start._bufferIndex + s,
					_index: i.start._index + o,
					line: i.end.line,
					column: i.end.column - c,
					offset: i.end.offset - c
				},
				end: { ...i.end }
			};
			i.end = { ...a.start }, i.start.offset === i.end.offset ? Object.assign(i, a) : n.add(r, 0, [[
				"enter",
				a,
				t
			], [
				"exit",
				a,
				t
			]]);
		}
		r++;
	}
	return n.consume(e), e;
}
//#endregion
//#region node_modules/.pnpm/micromark@4.0.3/node_modules/micromark/lib/constructs.js
var Xn = /* @__PURE__ */ e({
	attentionMarkers: () => ir,
	contentInitial: () => Qn,
	disable: () => ar,
	document: () => Zn,
	flow: () => er,
	flowInitial: () => $n,
	insideSpan: () => rr,
	string: () => tr,
	text: () => nr
}), Zn = {
	42: jn,
	43: jn,
	45: jn,
	48: jn,
	49: jn,
	50: jn,
	51: jn,
	52: jn,
	53: jn,
	54: jn,
	55: jn,
	56: jn,
	57: jn,
	62: ht
}, Qn = { 91: Jt }, $n = {
	[-2]: Dt,
	[-1]: Dt,
	32: Dt
}, er = {
	35: en,
	42: kn,
	45: [zn, kn],
	60: on,
	61: zn,
	95: kn,
	96: Tt,
	126: Tt
}, tr = {
	38: xt,
	92: yt
}, nr = {
	[-5]: Dn,
	[-4]: Dn,
	[-3]: Dn,
	33: Cn,
	38: xt,
	42: st,
	60: [dt, dn],
	91: Tn,
	92: [Qt, yt],
	93: pn,
	95: st,
	96: jt
}, rr = { null: [st, Wn] }, ir = { null: [42, 95] }, ar = { null: [] };
//#endregion
//#region node_modules/.pnpm/micromark@4.0.3/node_modules/micromark/lib/create-tokenizer.js
function or(e, t, n) {
	let r = {
		_bufferIndex: -1,
		_index: 0,
		line: n && n.line || 1,
		column: n && n.column || 1,
		offset: n && n.offset || 0
	}, i = {}, a = [], o = [], s = [], c = {
		attempt: S(x),
		check: S(ee),
		consume: v,
		enter: y,
		exit: b,
		interrupt: S(ee, { interrupt: !0 })
	}, l = {
		code: null,
		containerState: {},
		defineSkip: h,
		events: [],
		now: m,
		parser: e,
		previous: null,
		sliceSerialize: f,
		sliceStream: p,
		write: d
	}, u = t.tokenize.call(l, c);
	return t.resolveAll && a.push(t), l;
	function d(e) {
		return o = Pe(o, e), g(), o[o.length - 1] === null ? (te(t, 0), l.events = ot(a, l.events, l), l.events) : [];
	}
	function f(e, t) {
		return cr(p(e), t);
	}
	function p(e) {
		return sr(o, e);
	}
	function m() {
		let { _bufferIndex: e, _index: t, line: n, column: i, offset: a } = r;
		return {
			_bufferIndex: e,
			_index: t,
			line: n,
			column: i,
			offset: a
		};
	}
	function h(e) {
		i[e.line] = e.column, C();
	}
	function g() {
		for (; r._index < o.length;) {
			let e = o[r._index];
			if (typeof e == "string") {
				let t = r._index;
				for (r._bufferIndex < 0 && (r._bufferIndex = 0); r._index === t && r._bufferIndex < e.length;) _(e.charCodeAt(r._bufferIndex));
			} else _(e);
		}
	}
	function _(e) {
		u = u(e);
	}
	function v(e) {
		O(e) ? (r.line++, r.column = 1, r.offset += e === -3 ? 2 : 1, C()) : e !== -1 && (r.column++, r.offset++), r._bufferIndex < 0 ? r._index++ : (r._bufferIndex++, r._bufferIndex === o[r._index].length && (r._bufferIndex = -1, r._index++)), l.previous = e;
	}
	function y(e, t) {
		let n = t || {};
		return n.type = e, n.start = m(), l.events.push([
			"enter",
			n,
			l
		]), s.push(n), n;
	}
	function b(e) {
		let t = s.pop();
		return t.end = m(), l.events.push([
			"exit",
			t,
			l
		]), t;
	}
	function x(e, t) {
		te(e, t.from);
	}
	function ee(e, t) {
		t.restore();
	}
	function S(e, t) {
		return n;
		function n(n, r, i) {
			let a, o, s, u;
			return Array.isArray(n) ? f(n) : "tokenize" in n ? f([n]) : d(n);
			function d(e) {
				return t;
				function t(t) {
					let n = t !== null && e[t], r = t !== null && e.null;
					return f([...Array.isArray(n) ? n : n ? [n] : [], ...Array.isArray(r) ? r : r ? [r] : []])(t);
				}
			}
			function f(e) {
				return a = e, o = 0, e.length === 0 ? i : p(e[o]);
			}
			function p(e) {
				return n;
				function n(n) {
					return u = ne(), s = e, e.partial || (l.currentConstruct = e), e.name && l.parser.constructs.disable.null.includes(e.name) ? h(n) : e.tokenize.call(t ? Object.assign(Object.create(l), t) : l, c, m, h)(n);
				}
			}
			function m(t) {
				return e(s, u), r;
			}
			function h(e) {
				return u.restore(), ++o < a.length ? p(a[o]) : i;
			}
		}
	}
	function te(e, t) {
		e.resolveAll && !a.includes(e) && a.push(e), e.resolve && Ne(l.events, t, l.events.length - t, e.resolve(l.events.slice(t), l)), e.resolveTo && (l.events = e.resolveTo(l.events, l));
	}
	function ne() {
		let e = m(), t = l.previous, n = l.currentConstruct, i = l.events.length, a = Array.from(s);
		return {
			from: i,
			restore: o
		};
		function o() {
			r = e, l.previous = t, l.currentConstruct = n, l.events.length = i, s = a, C();
		}
	}
	function C() {
		r.line in i && r.column < 2 && (r.column = i[r.line], r.offset += i[r.line] - 1);
	}
}
function sr(e, t) {
	let n = t.start._index, r = t.start._bufferIndex, i = t.end._index, a = t.end._bufferIndex, o;
	if (n === i) o = [e[n].slice(r, a)];
	else {
		if (o = e.slice(n, i), r > -1) {
			let e = o[0];
			typeof e == "string" ? o[0] = e.slice(r) : o.shift();
		}
		a > 0 && o.push(e[i].slice(0, a));
	}
	return o;
}
function cr(e, t) {
	let n = -1, r = [], i;
	for (; ++n < e.length;) {
		let a = e[n], o;
		if (typeof a == "string") o = a;
		else switch (a) {
			case -5:
				o = "\r";
				break;
			case -4:
				o = "\n";
				break;
			case -3:
				o = "\r\n";
				break;
			case -2:
				o = t ? " " : "	";
				break;
			case -1:
				if (!t && i) continue;
				o = " ";
				break;
			default: o = String.fromCharCode(a);
		}
		i = a === -2, r.push(o);
	}
	return r.join("");
}
//#endregion
//#region node_modules/.pnpm/micromark@4.0.3/node_modules/micromark/lib/parse.js
function lr(e) {
	let t = {
		constructs: Ie([Xn, ...(e || {}).extensions || []]),
		content: n(Ze),
		defined: [],
		document: n(tt),
		flow: n(Hn),
		lazy: {},
		string: n(Gn),
		text: n(Kn)
	};
	return t;
	function n(e) {
		return n;
		function n(n) {
			return or(t, e, n);
		}
	}
}
//#endregion
//#region node_modules/.pnpm/micromark@4.0.3/node_modules/micromark/lib/postprocess.js
function ur(e) {
	for (; !Lt(e););
	return e;
}
//#endregion
//#region node_modules/.pnpm/micromark@4.0.3/node_modules/micromark/lib/preprocess.js
var dr = /[\0\t\n\r]/g;
function fr() {
	let e = 1, t = "", n = !0, r;
	return i;
	function i(i, a, o) {
		i = t + (typeof i == "string" ? i.toString() : new TextDecoder(a || void 0).decode(i));
		let s = [], c = 0;
		for (t = "", n &&= (i.charCodeAt(0) === 65279 && c++, void 0); c < i.length;) {
			dr.lastIndex = c;
			let n = dr.exec(i), a = n && n.index !== void 0 ? n.index : i.length, o = i.charCodeAt(a);
			if (!n) {
				t = i.slice(c);
				break;
			}
			if (o === 10 && c === a && r) s.push(-3), r = void 0;
			else switch (r &&= (s.push(-5), void 0), c < a && (s.push(i.slice(c, a)), e += a - c), o) {
				case 0:
					s.push(65533), e++;
					break;
				case 9: {
					let t = Math.ceil(e / 4) * 4;
					for (s.push(-2); e++ < t;) s.push(-1);
					break;
				}
				case 10:
					s.push(-4), e = 1;
					break;
				default: r = !0, e = 1;
			}
			c = a + 1;
		}
		return o && (r && s.push(-5), t && s.push(t), s.push(null)), s;
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-util-decode-string@2.0.1/node_modules/micromark-util-decode-string/index.js
var pr = /\\([!-/:-@[-`{-~])|&(#(?:\d{1,7}|x[\da-f]{1,6})|[\da-z]{1,31});/gi;
function mr(e) {
	return e.replace(pr, hr);
}
function hr(e, t, n) {
	if (t) return t;
	if (n.charCodeAt(0) === 35) {
		let e = n.charCodeAt(1), t = e === 120 || e === 88;
		return ze(n.slice(t ? 2 : 1), t ? 16 : 10);
	}
	return Me(n) || e;
}
//#endregion
//#region node_modules/.pnpm/unist-util-stringify-position@4.0.0/node_modules/unist-util-stringify-position/lib/index.js
function gr(e) {
	return !e || typeof e != "object" ? "" : "position" in e || "type" in e ? vr(e.position) : "start" in e || "end" in e ? vr(e) : "line" in e || "column" in e ? _r(e) : "";
}
function _r(e) {
	return yr(e && e.line) + ":" + yr(e && e.column);
}
function vr(e) {
	return _r(e && e.start) + "-" + _r(e && e.end);
}
function yr(e) {
	return e && typeof e == "number" ? e : 1;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-from-markdown@2.1.0/node_modules/mdast-util-from-markdown/lib/index.js
var br = {}.hasOwnProperty;
function xr(e, t, n) {
	return t && typeof t == "object" && (n = t, t = void 0), Sr(n)(ur(lr(n).document().write(fr()(e, t, !0))));
}
function Sr(e) {
	let t = {
		afterExit: [],
		beforeEnter: [],
		transforms: [],
		canContainEols: [
			"emphasis",
			"fragment",
			"heading",
			"paragraph",
			"strong"
		],
		enter: {
			autolink: o(Ae),
			autolinkProtocol: C,
			autolinkEmail: C,
			atxHeading: o(Te),
			blockQuote: o(be),
			characterEscape: C,
			characterReference: C,
			codeFenced: o(xe),
			codeFencedFenceInfo: s,
			codeFencedFenceMeta: s,
			codeIndented: o(xe, s),
			codeText: o(Se, s),
			codeTextData: C,
			data: C,
			codeFlowValue: C,
			definition: o(Ce),
			definitionDestinationString: s,
			definitionLabelString: s,
			definitionTitleString: s,
			emphasis: o(we),
			hardBreakEscape: o(Ee),
			hardBreakTrailing: o(Ee),
			htmlFlow: o(Oe, s),
			htmlFlowData: C,
			htmlText: o(Oe, s),
			htmlTextData: C,
			image: o(ke),
			label: s,
			link: o(Ae),
			listItem: o(Ne),
			listItemValue: p,
			listOrdered: o(je, f),
			listUnordered: o(je),
			paragraph: o(Pe),
			reference: me,
			referenceString: s,
			resourceDestinationString: s,
			resourceTitleString: s,
			setextHeading: o(Te),
			strong: o(Fe),
			thematicBreak: o(Le)
		},
		exit: {
			atxHeading: l(),
			atxHeadingSequence: ee,
			autolink: l(),
			autolinkEmail: E,
			autolinkProtocol: ye,
			blockQuote: l(),
			characterEscapeValue: w,
			characterReferenceMarkerHexadecimal: ge,
			characterReferenceMarkerNumeric: ge,
			characterReferenceValue: _e,
			characterReference: ve,
			codeFenced: l(_),
			codeFencedFence: g,
			codeFencedFenceInfo: m,
			codeFencedFenceMeta: h,
			codeFlowValue: w,
			codeIndented: l(v),
			codeText: l(oe),
			codeTextData: w,
			data: w,
			definition: l(),
			definitionDestinationString: x,
			definitionLabelString: y,
			definitionTitleString: b,
			emphasis: l(),
			hardBreakEscape: l(re),
			hardBreakTrailing: l(re),
			htmlFlow: l(ie),
			htmlFlowData: w,
			htmlText: l(ae),
			htmlTextData: w,
			image: l(ce),
			label: ue,
			labelText: le,
			lineEnding: T,
			link: l(se),
			listItem: l(),
			listOrdered: l(),
			listUnordered: l(),
			paragraph: l(),
			referenceString: he,
			resourceDestinationString: de,
			resourceTitleString: fe,
			resource: pe,
			setextHeading: l(ne),
			setextHeadingLineSequence: te,
			setextHeadingText: S,
			strong: l(),
			thematicBreak: l()
		}
	};
	wr(t, (e || {}).mdastExtensions || []);
	let n = {};
	return r;
	function r(e) {
		let r = {
			type: "root",
			children: [],
			position: void 0
		}, o = {
			stack: [r],
			tokenStack: [],
			config: t,
			enter: c,
			exit: u,
			buffer: s,
			resume: d,
			data: n
		}, l = [], f = [], p = -1;
		for (; ++p < e.length;) f.push(e[p]), (e[p][1].type === "listOrdered" || e[p][1].type === "listUnordered") && (e[p][0] === "enter" ? l.push(f.length - 1) : a(f, l.pop()));
		for (e = f, p = -1; ++p < e.length;) {
			let n = t[e[p][0]];
			e[p][0] === "enter" && t.beforeEnter.length > 0 && i(t.beforeEnter, {
				...o,
				sliceSerialize: e[p][2].sliceSerialize
			}, e[p][1]), br.call(n, e[p][1].type) && n[e[p][1].type].call({
				...o,
				sliceSerialize: e[p][2].sliceSerialize
			}, e[p][1]), e[p][0] === "exit" && t.afterExit.length > 0 && i(t.afterExit, {
				...o,
				sliceSerialize: e[p][2].sliceSerialize
			}, e[p][1]);
		}
		if (o.tokenStack.length > 0) {
			let e = o.tokenStack[o.tokenStack.length - 1];
			(e[1] || Er).call(o, void 0, e[0]);
		}
		for (r.position = {
			start: Cr(e.length > 0 ? e[0][1].start : {
				line: 1,
				column: 1,
				offset: 0
			}),
			end: Cr(e.length > 0 ? e[e.length - 2][1].end : {
				line: 1,
				column: 1,
				offset: 0
			})
		}, p = -1; ++p < t.transforms.length;) r = t.transforms[p](r) || r;
		return r;
	}
	function i(e, t, n) {
		let r = -1;
		for (; ++r < e.length;) e[r].call(t, n);
	}
	function a(e, t) {
		let n = e.length - 1, r = t - 1, i = -1, a = !1, o, s, c, l, u = [];
		for (; ++r <= n;) {
			let t = e[r];
			switch (t[1].type) {
				case "listUnordered":
				case "listOrdered":
				case "blockQuote":
					t[0] === "enter" ? i++ : i--, l = void 0;
					break;
				case "lineEndingBlank":
					t[0] === "enter" && (o && !l && !i && !c && (c = r), l = void 0);
					break;
				case "linePrefix":
				case "listItemValue":
				case "listItemMarker":
				case "listItemPrefix":
				case "listItemPrefixWhitespace": break;
				default: l = void 0;
			}
			if (!i && t[0] === "enter" && t[1].type === "listItemPrefix" || i === -1 && t[0] === "exit" && (t[1].type === "listUnordered" || t[1].type === "listOrdered")) {
				if (o) {
					let n = r;
					for (s = void 0; n--;) {
						let t = e[n];
						if (t[1].type === "lineEnding" || t[1].type === "lineEndingBlank") {
							if (t[0] === "exit") continue;
							s && (e[s][1].type = "lineEndingBlank", a = !0), t[1].type = "lineEnding", s = n;
						} else if (t[1].type !== "linePrefix" && t[1].type !== "blockQuotePrefix" && t[1].type !== "blockQuotePrefixWhitespace" && t[1].type !== "blockQuoteMarker" && t[1].type !== "listItemIndent") break;
					}
					c && (!s || c < s) && (o._spread = !0), o.end = Object.assign({}, s ? e[s][1].start : t[1].end), u.push({
						at: s || r,
						event: [
							"exit",
							o,
							t[2]
						]
					});
				}
				if (t[1].type === "listItemPrefix") {
					let e = {
						type: "listItem",
						_spread: !1,
						start: Object.assign({}, t[1].start),
						end: void 0
					};
					o = e, u.push({
						at: r,
						event: [
							"enter",
							e,
							t[2]
						]
					}), c = void 0, l = !0;
				}
			}
		}
		let d = e.splice(t), f = 0;
		for (r = -1; ++r < d.length;) {
			for (; f < u.length && u[f].at === t + r;) e.push(u[f++].event);
			e.push(d[r]);
		}
		e[t][1]._spread = a;
	}
	function o(e, t) {
		return n;
		function n(n) {
			c.call(this, e(n), n), t && t.call(this, n);
		}
	}
	function s() {
		this.stack.push({
			type: "fragment",
			children: []
		});
	}
	function c(e, t, n) {
		this.stack[this.stack.length - 1].children.push(e), this.stack.push(e), this.tokenStack.push([t, n || void 0]), e.position = {
			start: Cr(t.start),
			end: void 0
		};
	}
	function l(e) {
		return t;
		function t(t) {
			e && e.call(this, t), u.call(this, t);
		}
	}
	function u(e, t) {
		let n = this.stack.pop(), r = this.tokenStack.pop();
		if (!r) throw Error("Cannot close `" + e.type + "` (" + gr({
			start: e.start,
			end: e.end
		}) + "): it’s not open");
		r[0].type !== e.type && (t ? t.call(this, e, r[0]) : (r[1] || Er).call(this, e, r[0])), n.position.end = Cr(e.end);
	}
	function d() {
		return De(this.stack.pop());
	}
	function f() {
		this.data.expectingFirstListItemValue = !0;
	}
	function p(e) {
		if (this.data.expectingFirstListItemValue) {
			let t = this.stack[this.stack.length - 2];
			t.start = Number.parseInt(this.sliceSerialize(e), 10), this.data.expectingFirstListItemValue = void 0;
		}
	}
	function m() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.lang = e;
	}
	function h() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.meta = e;
	}
	function g() {
		this.data.flowCodeInside || (this.buffer(), this.data.flowCodeInside = !0);
	}
	function _() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.value = e.replace(/^(\r?\n|\r)|(\r?\n|\r)$/g, ""), this.data.flowCodeInside = void 0;
	}
	function v() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.value = e.replace(/(\r?\n|\r)$/g, "");
	}
	function y(e) {
		let t = this.resume(), n = this.stack[this.stack.length - 1];
		n.label = t, n.identifier = Be(this.sliceSerialize(e)).toLowerCase();
	}
	function b() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.title = e;
	}
	function x() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.url = e;
	}
	function ee(e) {
		let t = this.stack[this.stack.length - 1];
		t.depth ||= this.sliceSerialize(e).length;
	}
	function S() {
		this.data.setextHeadingSlurpLineEnding = !0;
	}
	function te(e) {
		let t = this.stack[this.stack.length - 1];
		t.depth = this.sliceSerialize(e).codePointAt(0) === 61 ? 1 : 2;
	}
	function ne() {
		this.data.setextHeadingSlurpLineEnding = void 0;
	}
	function C(e) {
		let t = this.stack[this.stack.length - 1].children, n = t[t.length - 1];
		(!n || n.type !== "text") && (n = Ie(), n.position = {
			start: Cr(e.start),
			end: void 0
		}, t.push(n)), this.stack.push(n);
	}
	function w(e) {
		let t = this.stack.pop();
		t.value += this.sliceSerialize(e), t.position.end = Cr(e.end);
	}
	function T(e) {
		let n = this.stack[this.stack.length - 1];
		if (this.data.atHardBreak) {
			let t = n.children[n.children.length - 1];
			t.position.end = Cr(e.end), this.data.atHardBreak = void 0;
		} else !this.data.setextHeadingSlurpLineEnding && t.canContainEols.includes(n.type) && (C.call(this, e), w.call(this, e));
	}
	function re() {
		this.data.atHardBreak = !0;
	}
	function ie() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.value = e;
	}
	function ae() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.value = e;
	}
	function oe() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.value = e;
	}
	function se() {
		let e = this.stack[this.stack.length - 1];
		if (this.data.inReference) {
			let t = this.data.referenceType || "shortcut";
			e.type += "Reference", e.referenceType = t, delete e.url, delete e.title;
		} else delete e.identifier, delete e.label;
		this.data.referenceType = void 0;
	}
	function ce() {
		let e = this.stack[this.stack.length - 1];
		if (this.data.inReference) {
			let t = this.data.referenceType || "shortcut";
			e.type += "Reference", e.referenceType = t, delete e.url, delete e.title;
		} else delete e.identifier, delete e.label;
		this.data.referenceType = void 0;
	}
	function le(e) {
		let t = this.sliceSerialize(e), n = this.stack[this.stack.length - 2];
		n.label = mr(t), n.identifier = Be(t).toLowerCase();
	}
	function ue() {
		let e = this.stack[this.stack.length - 1], t = this.resume(), n = this.stack[this.stack.length - 1];
		this.data.inReference = !0, n.type === "link" ? n.children = e.children : n.alt = t;
	}
	function de() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.url = e;
	}
	function fe() {
		let e = this.resume(), t = this.stack[this.stack.length - 1];
		t.title = e;
	}
	function pe() {
		this.data.inReference = void 0;
	}
	function me() {
		this.data.referenceType = "collapsed";
	}
	function he(e) {
		let t = this.resume(), n = this.stack[this.stack.length - 1];
		n.label = t, n.identifier = Be(this.sliceSerialize(e)).toLowerCase(), this.data.referenceType = "full";
	}
	function ge(e) {
		this.data.characterReferenceType = e.type;
	}
	function _e(e) {
		let t = this.sliceSerialize(e), n = this.data.characterReferenceType, r;
		n ? (r = ze(t, n === "characterReferenceMarkerNumeric" ? 10 : 16), this.data.characterReferenceType = void 0) : r = Me(t);
		let i = this.stack[this.stack.length - 1];
		i.value += r;
	}
	function ve(e) {
		let t = this.stack.pop();
		t.position.end = Cr(e.end);
	}
	function ye(e) {
		w.call(this, e);
		let t = this.stack[this.stack.length - 1];
		t.url = this.sliceSerialize(e);
	}
	function E(e) {
		w.call(this, e);
		let t = this.stack[this.stack.length - 1];
		t.url = "mailto:" + this.sliceSerialize(e);
	}
	function be() {
		return {
			type: "blockquote",
			children: [],
			position: void 0
		};
	}
	function xe() {
		return {
			type: "code",
			lang: null,
			meta: null,
			value: "",
			position: void 0
		};
	}
	function Se() {
		return {
			type: "inlineCode",
			value: "",
			position: void 0
		};
	}
	function Ce() {
		return {
			type: "definition",
			identifier: "",
			label: null,
			title: null,
			url: "",
			position: void 0
		};
	}
	function we() {
		return {
			type: "emphasis",
			children: [],
			position: void 0
		};
	}
	function Te() {
		return {
			type: "heading",
			depth: 0,
			children: [],
			position: void 0
		};
	}
	function Ee() {
		return {
			type: "break",
			position: void 0
		};
	}
	function Oe() {
		return {
			type: "html",
			value: "",
			position: void 0
		};
	}
	function ke() {
		return {
			type: "image",
			title: null,
			url: "",
			alt: null,
			position: void 0
		};
	}
	function Ae() {
		return {
			type: "link",
			title: null,
			url: "",
			children: [],
			position: void 0
		};
	}
	function je(e) {
		return {
			type: "list",
			ordered: e.type === "listOrdered",
			start: null,
			spread: e._spread,
			children: [],
			position: void 0
		};
	}
	function Ne(e) {
		return {
			type: "listItem",
			spread: e._spread,
			checked: null,
			children: [],
			position: void 0
		};
	}
	function Pe() {
		return {
			type: "paragraph",
			children: [],
			position: void 0
		};
	}
	function Fe() {
		return {
			type: "strong",
			children: [],
			position: void 0
		};
	}
	function Ie() {
		return {
			type: "text",
			value: "",
			position: void 0
		};
	}
	function Le() {
		return {
			type: "thematicBreak",
			position: void 0
		};
	}
}
function Cr(e) {
	return {
		line: e.line,
		column: e.column,
		offset: e.offset
	};
}
function wr(e, t) {
	let n = -1;
	for (; ++n < t.length;) {
		let r = t[n];
		Array.isArray(r) ? wr(e, r) : Tr(e, r);
	}
}
function Tr(e, t) {
	let n;
	for (n in t) if (br.call(t, n)) switch (n) {
		case "canContainEols": {
			let r = t[n];
			r && e[n].push(...r);
			break;
		}
		case "transforms": {
			let r = t[n];
			r && e[n].push(...r);
			break;
		}
		case "afterExit":
		case "beforeEnter": {
			let r = t[n];
			r && e[n].push(r);
			break;
		}
		case "enter":
		case "exit": {
			let r = t[n];
			r && Object.assign(e[n], r);
			break;
		}
	}
}
function Er(e, t) {
	throw Error(e ? "Cannot close `" + e.type + "` (" + gr({
		start: e.start,
		end: e.end
	}) + "): a different token (`" + t.type + "`, " + gr({
		start: t.start,
		end: t.end
	}) + ") is open" : "Cannot close document, a token (`" + t.type + "`, " + gr({
		start: t.start,
		end: t.end
	}) + ") is still open");
}
//#endregion
//#region node_modules/.pnpm/remark-parse@11.0.0/node_modules/remark-parse/lib/index.js
function Dr(e) {
	let t = this;
	t.parser = n;
	function n(n) {
		return xr(n, {
			...t.data("settings"),
			...e,
			extensions: t.data("micromarkExtensions") || [],
			mdastExtensions: t.data("fromMarkdownExtensions") || []
		});
	}
}
//#endregion
//#region node_modules/.pnpm/unist-util-is@6.0.1/node_modules/unist-util-is/lib/index.js
var Or = (function(e) {
	if (e == null) return Nr;
	if (typeof e == "function") return Mr(e);
	if (typeof e == "object") return Array.isArray(e) ? kr(e) : Ar(e);
	if (typeof e == "string") return jr(e);
	throw Error("Expected function, string, or object as test");
});
function kr(e) {
	let t = [], n = -1;
	for (; ++n < e.length;) t[n] = Or(e[n]);
	return Mr(r);
	function r(...e) {
		let n = -1;
		for (; ++n < t.length;) if (t[n].apply(this, e)) return !0;
		return !1;
	}
}
function Ar(e) {
	let t = e;
	return Mr(n);
	function n(n) {
		let r = n, i;
		for (i in e) if (r[i] !== t[i]) return !1;
		return !0;
	}
}
function jr(e) {
	return Mr(t);
	function t(t) {
		return t && t.type === e;
	}
}
function Mr(e) {
	return t;
	function t(t, n, r) {
		return !!(Pr(t) && e.call(this, t, typeof n == "number" ? n : void 0, r || void 0));
	}
}
function Nr() {
	return !0;
}
function Pr(e) {
	return typeof e == "object" && !!e && "type" in e;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-phrasing@4.1.0/node_modules/mdast-util-phrasing/lib/index.js
var Fr = Or([
	"break",
	"delete",
	"emphasis",
	"footnote",
	"footnoteReference",
	"image",
	"imageReference",
	"inlineCode",
	"inlineMath",
	"link",
	"linkReference",
	"mdxJsxTextElement",
	"mdxTextExpression",
	"strong",
	"text",
	"textDirective"
]), Ir = {}.hasOwnProperty;
function Lr(e, t) {
	let n = t || {};
	function r(t, ...n) {
		let i = r.invalid, a = r.handlers;
		if (t && Ir.call(t, e)) {
			let n = String(t[e]);
			i = Ir.call(a, n) ? a[n] : r.unknown;
		}
		if (i) return i.call(this, t, ...n);
	}
	return r.handlers = n.handlers || {}, r.invalid = n.invalid, r.unknown = n.unknown, r;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/configure.js
var Rr = {}.hasOwnProperty;
function zr(e, t) {
	let n = -1, r;
	if (t.extensions) for (; ++n < t.extensions.length;) zr(e, t.extensions[n]);
	for (r in t) if (Rr.call(t, r)) switch (r) {
		case "extensions": break;
		/* c8 ignore next 4 */
		case "unsafe":
			Br(e[r], t[r]);
			break;
		case "join":
			Br(e[r], t[r]);
			break;
		case "handlers":
			Vr(e[r], t[r]);
			break;
		default: e.options[r] = t[r];
	}
	return e;
}
function Br(e, t) {
	t && e.push(...t);
}
function Vr(e, t) {
	t && Object.assign(e, t);
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/blockquote.js
function Hr(e, t, n, r) {
	let i = n.enter("blockquote"), a = n.createTracker(r);
	a.move("> "), a.shift(2);
	let o = n.indentLines(n.containerFlow(e, a.current()), Ur);
	return i(), o;
}
function Ur(e, t, n) {
	return ">" + (n ? "" : " ") + e;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/pattern-in-scope.js
function Wr(e, t) {
	return Gr(e, t.inConstruct, !0) && !Gr(e, t.notInConstruct, !1);
}
function Gr(e, t, n) {
	if (typeof t == "string" && (t = [t]), !t || t.length === 0) return n;
	let r = -1;
	for (; ++r < t.length;) if (e.includes(t[r])) return !0;
	return !1;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/break.js
function Kr(e, t, n, r) {
	let i = -1;
	for (; ++i < n.unsafe.length;) {
		let e = n.unsafe[i];
		if (e.character === "\n" && !e.before && !e.after && Wr(n.stack, e)) return /[\t ]/.test(r.before) ? "" : " ";
	}
	return "\\\n";
}
//#endregion
//#region node_modules/.pnpm/longest-streak@3.1.0/node_modules/longest-streak/index.js
function qr(e, t) {
	let n = String(e), r = n.indexOf(t), i = r, a = 0, o = 0;
	if (typeof t != "string") throw TypeError("Expected substring");
	for (; r !== -1;) r === i ? ++a > o && (o = a) : a = 1, i = r + t.length, r = n.indexOf(t, i);
	return o;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/format-code-as-indented.js
function Jr(e, t) {
	return !(t.options.fences !== !1 || !e.value || e.lang || !/[^\n\r ]/.test(e.value) || /^[\t ]*(?:[\n\r]|$)|(?:^|[\n\r])[\t ]*$/.test(e.value));
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-fence.js
function Yr(e) {
	let t = e.options.fence || "`";
	if (t !== "`" && t !== "~") throw Error("Cannot serialize code with `" + t + "` for `options.fence`, expected `` ` `` or `~`");
	return t;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/code.js
function Xr(e, t, n, r) {
	let i = Yr(n), a = e.value || "";
	if (Jr(e, n)) {
		let e = n.enter("codeIndented"), t = n.indentLines(a, Zr);
		return e(), t;
	}
	let o = n.createTracker(r), s = i.repeat(Math.max(qr(a, i) + 1, 3)), c = n.enter("codeFenced"), l = i === "`" ? "GraveAccent" : "Tilde", u = o.move(s);
	if (e.lang) {
		let t = n.enter(`codeFencedLang${l}`);
		u += o.move(n.safe(e.lang, {
			before: u,
			after: " ",
			encode: ["`"],
			...o.current()
		})), t();
	}
	if (e.lang && e.meta) {
		let t = n.enter(`codeFencedMeta${l}`);
		u += o.move(" "), u += o.move(n.safe(e.meta, {
			before: u,
			after: "\n",
			encode: ["`"],
			...o.current()
		})), t();
	}
	return u += o.move("\n"), a && (u += o.move(a + "\n")), u += o.move(s), c(), u;
}
function Zr(e, t, n) {
	return (n ? "" : "    ") + e;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-quote.js
function Qr(e) {
	let t = e.options.quote || "\"";
	if (t !== "\"" && t !== "'") throw Error("Cannot serialize title with `" + t + "` for `options.quote`, expected `\"`, or `'`");
	return t;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/definition.js
function $r(e, t, n, r) {
	let i = Qr(n), a = i === "\"" ? "Quote" : "Apostrophe", o = n.enter("definition"), s = n.enter("label"), c = n.createTracker(r), l = c.move("[");
	return l += c.move(n.safe(n.associationId(e), {
		before: l,
		after: "]",
		...c.current()
	})), l += c.move("]: "), s(), !e.url || /[\0- \u007F]/.test(e.url) ? (s = n.enter("destinationLiteral"), l += c.move("<"), l += c.move(n.safe(e.url, {
		before: l,
		after: ">",
		...c.current()
	})), l += c.move(">")) : (s = n.enter("destinationRaw"), l += c.move(n.safe(e.url, {
		before: l,
		after: e.title ? " " : "\n",
		...c.current()
	}))), s(), e.title && (s = n.enter(`title${a}`), l += c.move(" " + i), l += c.move(n.safe(e.title, {
		before: l,
		after: i,
		...c.current()
	})), l += c.move(i), s()), o(), l;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-emphasis.js
function ei(e) {
	let t = e.options.emphasis || "*";
	if (t !== "*" && t !== "_") throw Error("Cannot serialize emphasis with `" + t + "` for `options.emphasis`, expected `*`, or `_`");
	return t;
}
ti.attention = ni, ti.peek = ri;
function ti(e, t, n, r) {
	let i = n.enter("phrasing"), a = n.containerPhrasing({
		type: "root",
		children: [e]
	}, r);
	return i(), a;
}
function ni(e, t) {
	return {
		construct: "emphasis",
		markers: ei(t) === "*" ? ["*", "_"] : ["_", "*"],
		sizes: [1]
	};
}
function ri(e, t, n) {
	return n.options.emphasis || "*";
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/encode-character-reference.js
function ii(e) {
	return "&#x" + e.toString(16).toUpperCase() + ";";
}
//#endregion
//#region node_modules/.pnpm/unist-util-visit-parents@6.0.2/node_modules/unist-util-visit-parents/lib/color.js
function ai(e) {
	return e;
}
//#endregion
//#region node_modules/.pnpm/unist-util-visit-parents@6.0.2/node_modules/unist-util-visit-parents/lib/index.js
var oi = [], si = "skip";
function ci(e, t, n, r) {
	let i;
	typeof t == "function" && typeof n != "function" ? (r = n, n = t) : i = t;
	let a = Or(i), o = r ? -1 : 1;
	s(e, void 0, [])();
	function s(e, i, c) {
		let l = e && typeof e == "object" ? e : {};
		if (typeof l.type == "string") {
			let t = typeof l.tagName == "string" ? l.tagName : typeof l.name == "string" ? l.name : void 0;
			Object.defineProperty(u, "name", { value: "node (" + ai(e.type + (t ? "<" + t + ">" : "")) + ")" });
		}
		return u;
		function u() {
			let l = oi, u, d, f;
			if ((!t || a(e, i, c[c.length - 1] || void 0)) && (l = li(n(e, c)), l[0] === !1)) return l;
			if ("children" in e && e.children) {
				let t = e;
				if (t.children && l[0] !== "skip") for (d = (r ? t.children.length : -1) + o, f = c.concat(t); d > -1 && d < t.children.length;) {
					let e = t.children[d];
					if (u = s(e, d, f)(), u[0] === !1) return u;
					d = typeof u[1] == "number" ? u[1] : d + o;
				}
			}
			return l;
		}
	}
}
function li(e) {
	return Array.isArray(e) ? e : typeof e == "number" ? [!0, e] : e == null ? oi : [e];
}
//#endregion
//#region node_modules/.pnpm/unist-util-visit@5.1.0/node_modules/unist-util-visit/lib/index.js
function ui(e, t, n, r) {
	let i, a, o;
	typeof t == "function" && typeof n != "function" ? (a = void 0, o = t, i = n) : (a = t, o = n, i = r), ci(e, a, s, i);
	function s(e, t) {
		let n = t[t.length - 1], r = n ? n.children.indexOf(e) : void 0;
		return o(e, r, n);
	}
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/format-heading-as-setext.js
function di(e, t) {
	let n = !1;
	return ui(e, function(e) {
		if ("value" in e && /\r?\n|\r/.test(e.value) || e.type === "break") return n = !0, !1;
	}), !!((!e.depth || e.depth < 3) && De(e) && (t.options.setext || n));
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/heading.js
function fi(e, t, n, r) {
	let i = Math.max(Math.min(6, e.depth || 1), 1), a = n.createTracker(r);
	if (di(e, n)) {
		let t = n.enter("headingSetext"), r = n.enter("phrasing"), o = n.containerPhrasing(e, {
			...a.current(),
			before: "\n",
			after: "\n"
		});
		return r(), t(), o + "\n" + (i === 1 ? "=" : "-").repeat(o.length - (Math.max(o.lastIndexOf("\r"), o.lastIndexOf("\n")) + 1));
	}
	let o = "#".repeat(i), s = n.enter("headingAtx"), c = n.enter("phrasing");
	a.move(o + " ");
	let l = n.containerPhrasing(e, {
		before: "# ",
		after: "\n",
		...a.current()
	}), u = l.charCodeAt(0);
	return (u === 9 || u === 32) && (l = ii(u) + l.slice(1)), l = l ? o + " " + l : o, n.options.closeAtx && (l += " " + o), c(), s(), l;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/html.js
pi.peek = mi;
function pi(e) {
	return e.value || "";
}
function mi() {
	return "<";
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/image.js
hi.peek = gi;
function hi(e, t, n, r) {
	let i = Qr(n), a = i === "\"" ? "Quote" : "Apostrophe", o = n.enter("image"), s = n.enter("label"), c = n.createTracker(r), l = c.move("![");
	return l += c.move(n.safe(e.alt, {
		before: l,
		after: "]",
		...c.current()
	})), l += c.move("]("), s(), !e.url && e.title || /[\0- \u007F]/.test(e.url) ? (s = n.enter("destinationLiteral"), l += c.move("<"), l += c.move(n.safe(e.url, {
		before: l,
		after: ">",
		...c.current()
	})), l += c.move(">")) : (s = n.enter("destinationRaw"), l += c.move(n.safe(e.url, {
		before: l,
		after: e.title ? " " : ")",
		...c.current()
	}))), s(), e.title && (s = n.enter(`title${a}`), l += c.move(" " + i), l += c.move(n.safe(e.title, {
		before: l,
		after: i,
		...c.current()
	})), l += c.move(i), s()), l += c.move(")"), o(), l;
}
function gi() {
	return "!";
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/image-reference.js
_i.peek = vi;
function _i(e, t, n, r) {
	let i = e.referenceType, a = n.enter("imageReference"), o = n.enter("label"), s = n.createTracker(r), c = s.move("!["), l = n.safe(e.alt, {
		before: c,
		after: "]",
		...s.current()
	});
	c += s.move(l + "]["), o();
	let u = n.stack;
	n.stack = u.filter(function(e) {
		return e !== "phrasing";
	}), o = n.enter("reference");
	let d = n.safe(n.associationId(e), {
		before: c,
		after: "]",
		...s.current()
	});
	return o(), n.stack = u, a(), i === "full" || !l || l !== d ? c += s.move(d + "]") : i === "shortcut" ? c = c.slice(0, -1) : c += s.move("]"), c;
}
function vi() {
	return "!";
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/inline-code.js
yi.peek = bi;
function yi(e, t, n) {
	let r = e.value || "", i = "`", a = -1;
	for (; RegExp("(^|[^`])" + i + "([^`]|$)").test(r);) i += "`";
	for (/[^\n\r ]/.test(r) && (/^[\n\r ]/.test(r) && /[\n\r ]$/.test(r) || /^`|`$/.test(r)) && (r = " " + r + " "); ++a < n.unsafe.length;) {
		let e = n.unsafe[a], t = n.compilePattern(e);
		if (!e.atBreak) continue;
		let i;
		for (; i = t.exec(r);) {
			let e = i.index;
			r.charCodeAt(e) === 10 && r.charCodeAt(e - 1) === 13 && e--, r = r.slice(0, e) + " " + r.slice(i.index + 1);
		}
	}
	return i + r + i;
}
function bi() {
	return "`";
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/format-link-as-autolink.js
function xi(e, t) {
	let n = De(e);
	return !(t.options.resourceLink || !e.url || e.title || !e.children || e.children.length !== 1 || e.children[0].type !== "text" || n !== e.url && "mailto:" + n !== e.url || !/^[a-z][+\-.a-z]+:/i.test(e.url) || /[\0- <>\u007F]/.test(e.url));
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/link.js
Si.peek = Ci;
function Si(e, t, n, r) {
	let i = Qr(n), a = i === "\"" ? "Quote" : "Apostrophe", o = n.createTracker(r);
	if (xi(e, n)) {
		let t = n.stack;
		n.stack = t.filter(function(e) {
			return e !== "phrasing";
		});
		let r = n.enter("autolink"), i = o.move("<");
		return i += o.move(n.containerPhrasing(e, {
			before: i,
			after: ">",
			...o.current()
		})), i += o.move(">"), r(), n.stack = t, i;
	}
	let s = n.enter("link"), c = n.enter("label"), l = o.move("[");
	return l += o.move(n.containerPhrasing(e, {
		before: l,
		after: "](",
		...o.current()
	})), l += o.move("]("), c(), !e.url && e.title || /[\0- \u007F]/.test(e.url) ? (c = n.enter("destinationLiteral"), l += o.move("<"), l += o.move(n.safe(e.url, {
		before: l,
		after: ">",
		...o.current()
	})), l += o.move(">")) : (c = n.enter("destinationRaw"), l += o.move(n.safe(e.url, {
		before: l,
		after: e.title ? " " : ")",
		...o.current()
	}))), c(), e.title && (c = n.enter(`title${a}`), l += o.move(" " + i), l += o.move(n.safe(e.title, {
		before: l,
		after: i,
		...o.current()
	})), l += o.move(i), c()), l += o.move(")"), s(), l;
}
function Ci(e, t, n) {
	return xi(e, n) ? "<" : "[";
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/link-reference.js
wi.peek = Ti;
function wi(e, t, n, r) {
	let i = e.referenceType, a = n.enter("linkReference"), o = n.enter("label"), s = n.createTracker(r), c = s.move("["), l = n.containerPhrasing(e, {
		before: c,
		after: "]",
		...s.current()
	});
	c += s.move(l + "]["), o();
	let u = n.stack;
	n.stack = u.filter(function(e) {
		return e !== "phrasing";
	}), o = n.enter("reference");
	let d = n.safe(n.associationId(e), {
		before: c,
		after: "]",
		...s.current()
	});
	return o(), n.stack = u, a(), i === "full" || !l || l !== d ? c += s.move(d + "]") : i === "shortcut" ? c = c.slice(0, -1) : c += s.move("]"), c;
}
function Ti() {
	return "[";
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-bullet.js
function Ei(e) {
	let t = e.options.bullet || "*";
	if (t !== "*" && t !== "+" && t !== "-") throw Error("Cannot serialize items with `" + t + "` for `options.bullet`, expected `*`, `+`, or `-`");
	return t;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-bullet-other.js
function Di(e) {
	let t = Ei(e), n = e.options.bulletOther;
	if (!n) return t === "*" ? "-" : "*";
	if (n !== "*" && n !== "+" && n !== "-") throw Error("Cannot serialize items with `" + n + "` for `options.bulletOther`, expected `*`, `+`, or `-`");
	if (n === t) throw Error("Expected `bullet` (`" + t + "`) and `bulletOther` (`" + n + "`) to be different");
	return n;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-bullet-ordered.js
function Oi(e) {
	let t = e.options.bulletOrdered || ".";
	if (t !== "." && t !== ")") throw Error("Cannot serialize items with `" + t + "` for `options.bulletOrdered`, expected `.` or `)`");
	return t;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-rule.js
function ki(e) {
	let t = e.options.rule || "*";
	if (t !== "*" && t !== "-" && t !== "_") throw Error("Cannot serialize rules with `" + t + "` for `options.rule`, expected `*`, `-`, or `_`");
	return t;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/list.js
function Ai(e, t, n, r) {
	let i = n.enter("list"), a = n.bulletCurrent, o = e.ordered ? Oi(n) : Ei(n), s = e.ordered ? o === "." ? ")" : "." : Di(n), c = t && n.bulletLastUsed ? o === n.bulletLastUsed : !1;
	if (!e.ordered) {
		let t = e.children ? e.children[0] : void 0;
		if ((o === "*" || o === "-") && t && (!t.children || !t.children[0]) && n.stack[n.stack.length - 1] === "list" && n.stack[n.stack.length - 2] === "listItem" && n.stack[n.stack.length - 3] === "list" && n.stack[n.stack.length - 4] === "listItem" && n.indexStack[n.indexStack.length - 1] === 0 && n.indexStack[n.indexStack.length - 2] === 0 && n.indexStack[n.indexStack.length - 3] === 0 && (c = !0), ki(n) === o && t) {
			let t = -1;
			for (; ++t < e.children.length;) {
				let n = e.children[t];
				if (n && n.type === "listItem" && n.children && n.children[0] && n.children[0].type === "thematicBreak") {
					c = !0;
					break;
				}
			}
		}
	}
	c && (o = s), n.bulletCurrent = o;
	let l = n.containerFlow(e, r);
	return n.bulletLastUsed = o, n.bulletCurrent = a, i(), l;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-list-item-indent.js
function ji(e) {
	let t = e.options.listItemIndent || "one";
	if (t !== "tab" && t !== "one" && t !== "mixed") throw Error("Cannot serialize items with `" + t + "` for `options.listItemIndent`, expected `tab`, `one`, or `mixed`");
	return t;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/list-item.js
function Mi(e, t, n, r) {
	let i = ji(n), a = n.bulletCurrent || Ei(n);
	t && t.type === "list" && t.ordered && (a = (typeof t.start == "number" && t.start > -1 ? t.start : 1) + (n.options.incrementListMarker === !1 ? 0 : t.children.indexOf(e)) + a);
	let o = a.length + 1;
	(i === "tab" || i === "mixed" && (t && t.type === "list" && t.spread || e.spread)) && (o = Math.ceil(o / 4) * 4);
	let s = n.createTracker(r);
	s.move(a + " ".repeat(o - a.length)), s.shift(o);
	let c = n.enter("listItem"), l = n.indentLines(n.containerFlow(e, s.current()), u);
	return c(), l;
	function u(e, t, n) {
		return t ? (n ? "" : " ".repeat(o)) + e : (n ? a : a + " ".repeat(o - a.length)) + e;
	}
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/paragraph.js
function Ni(e, t, n, r) {
	let i = n.enter("paragraph"), a = n.enter("phrasing"), o = n.containerPhrasing(e, r);
	return a(), i(), o;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/root.js
function Pi(e, t, n, r) {
	if (!e.children.some(function(e) {
		return Fr(e);
	})) return n.containerFlow(e, r);
	let i = n.enter("phrasing"), a = n.containerPhrasing(e, r);
	return i(), a;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-strong.js
function Fi(e) {
	let t = e.options.strong || "*";
	if (t !== "*" && t !== "_") throw Error("Cannot serialize strong with `" + t + "` for `options.strong`, expected `*`, or `_`");
	return t;
}
Ii.attention = Li, Ii.peek = Ri;
function Ii(e, t, n, r) {
	let i = n.enter("phrasing"), a = n.containerPhrasing({
		type: "root",
		children: [e]
	}, r);
	return i(), a;
}
function Li(e, t) {
	return {
		construct: "strong",
		markers: Fi(t) === "*" ? ["*", "_"] : ["_", "*"],
		sizes: [2]
	};
}
function Ri(e, t, n) {
	return n.options.strong || "*";
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/text.js
function zi(e, t, n, r) {
	/* c8 ignore next -- parent always passed */
	let i = t ? t.children : [], a = i.indexOf(e);
	return n.safe(e.value, {
		...r,
		afterNode: i[a + 1],
		beforeNode: i[a - 1]
	});
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/check-rule-repetition.js
function Bi(e) {
	let t = e.options.ruleRepetition || 3;
	if (t < 3) throw Error("Cannot serialize rules with repetition `" + t + "` for `options.ruleRepetition`, expected `3` or more");
	return t;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/thematic-break.js
function Vi(e, t, n) {
	let r = (ki(n) + (n.options.ruleSpaces ? " " : "")).repeat(Bi(n));
	return n.options.ruleSpaces ? r.slice(0, -1) : r;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/handle/index.js
var Hi = {
	blockquote: Hr,
	break: Kr,
	code: Xr,
	definition: $r,
	emphasis: ti,
	hardBreak: Kr,
	heading: fi,
	html: pi,
	image: hi,
	imageReference: _i,
	inlineCode: yi,
	link: Si,
	linkReference: wi,
	list: Ai,
	listItem: Mi,
	paragraph: Ni,
	root: Pi,
	strong: Ii,
	text: zi,
	thematicBreak: Vi
};
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/html-kind.js
function Ui(e) {
	if (e.charCodeAt(0) !== 60) return;
	let t = e.charCodeAt(1);
	if (t === 33) {
		let t = e.charCodeAt(2);
		return t === 45 ? 2 : t === 91 ? 5 : D(t) ? 4 : void 0;
	}
	if (t === 63) return 3;
	let n = t === 47, r = n ? 2 : 1, i = r, a = e.charCodeAt(i);
	if (!D(a)) return;
	for (; a === 45 || Ve(a);) a = e.charCodeAt(++i);
	let o = e.slice(r, i).toLowerCase();
	return !n && a !== 47 && an.includes(o) ? 1 : rn.includes(o) ? 6 : 7;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/join.js
var Wi = [Gi];
function Gi(e, t, n, r) {
	if (t.type === "code" && Jr(t, r) && (e.type === "list" || e.type === t.type && Jr(e, r))) return !1;
	if ("spread" in n && typeof n.spread == "boolean") {
		if (e.type === "paragraph" && (e.type === t.type || t.type === "definition" || t.type === "heading" && di(t, r))) return;
		if (e.type === "html") {
			let t = Ui(e.value);
			if (t === void 0 || t === 6 || t === 7) return 1;
		}
		if (e.type === "paragraph" && t.type === "html") {
			let e = Ui(t.value);
			if (e === void 0 || e === 7) return 1;
		}
		return +!!n.spread;
	}
}
for (var Ki = [
	"autolink",
	"destinationLiteral",
	"destinationRaw",
	"reference",
	"titleQuote",
	"titleApostrophe"
], qi = [
	{
		character: "	",
		after: "[\\r\\n]",
		inConstruct: "phrasing"
	},
	{
		character: "	",
		before: "[\\r\\n]",
		inConstruct: "phrasing"
	},
	{
		character: "	",
		inConstruct: ["codeFencedLangGraveAccent", "codeFencedLangTilde"]
	},
	{
		character: "\r",
		inConstruct: [
			"codeFencedLangGraveAccent",
			"codeFencedLangTilde",
			"codeFencedMetaGraveAccent",
			"codeFencedMetaTilde",
			"destinationLiteral",
			"headingAtx"
		]
	},
	{
		character: "\n",
		inConstruct: [
			"codeFencedLangGraveAccent",
			"codeFencedLangTilde",
			"codeFencedMetaGraveAccent",
			"codeFencedMetaTilde",
			"destinationLiteral",
			"headingAtx"
		]
	},
	{
		character: "\r",
		before: "[\\r\\n]",
		inConstruct: "phrasing"
	},
	{
		character: "\r",
		after: "\\r",
		inConstruct: "phrasing"
	},
	{
		character: "\n",
		before: "\\n",
		inConstruct: "phrasing"
	},
	{
		character: "\n",
		after: "[\\r\\n]",
		inConstruct: "phrasing"
	},
	{
		character: " ",
		after: "[\\r\\n]",
		inConstruct: "phrasing"
	},
	{
		character: " ",
		before: "[\\r\\n]",
		inConstruct: "phrasing"
	},
	{
		character: " ",
		inConstruct: ["codeFencedLangGraveAccent", "codeFencedLangTilde"]
	},
	{
		character: "!",
		after: "\\[",
		inConstruct: "phrasing",
		notInConstruct: Ki
	},
	{
		character: "\"",
		inConstruct: "titleQuote"
	},
	{
		atBreak: !0,
		character: "#"
	},
	{
		character: "#",
		inConstruct: "headingAtx",
		after: "(?:[\r\n]|$)"
	},
	{
		character: "&",
		after: "[#A-Za-z]",
		inConstruct: "phrasing"
	},
	{
		character: "'",
		inConstruct: "titleApostrophe"
	},
	{
		character: "(",
		inConstruct: "destinationRaw"
	},
	{
		before: "\\]",
		character: "(",
		inConstruct: "phrasing",
		notInConstruct: Ki
	},
	{
		atBreak: !0,
		before: "\\d+",
		character: ")"
	},
	{
		character: ")",
		inConstruct: "destinationRaw"
	},
	{
		atBreak: !0,
		character: "*",
		after: "(?:[ 	\r\n*])"
	},
	{
		character: "*",
		inConstruct: "phrasing",
		notInConstruct: Ki
	},
	{
		atBreak: !0,
		character: "+",
		after: "(?:[ 	\r\n])"
	},
	{
		atBreak: !0,
		character: "-",
		after: "(?:[ 	\r\n-])"
	},
	{
		atBreak: !0,
		before: "\\d+",
		character: ".",
		after: "(?:[ 	\r\n]|$)"
	},
	{
		atBreak: !0,
		character: "<",
		after: "[!/?A-Za-z]"
	},
	{
		character: "<",
		after: "[!/?A-Za-z]",
		inConstruct: "phrasing",
		notInConstruct: Ki
	},
	{
		character: "<",
		inConstruct: "destinationLiteral"
	},
	{
		atBreak: !0,
		character: "="
	},
	{
		atBreak: !0,
		character: ">"
	},
	{
		character: ">",
		inConstruct: "destinationLiteral"
	},
	{
		atBreak: !0,
		character: "["
	},
	{
		character: "[",
		inConstruct: "phrasing",
		notInConstruct: Ki
	},
	{
		character: "[",
		inConstruct: ["label", "reference"]
	},
	{
		character: "\\",
		after: "[\\r\\n]",
		inConstruct: "phrasing"
	},
	{
		character: "]",
		inConstruct: ["label", "reference"]
	},
	{
		atBreak: !0,
		character: "_"
	},
	{
		character: "_",
		inConstruct: "phrasing",
		notInConstruct: Ki
	},
	{
		atBreak: !0,
		character: "`"
	},
	{
		character: "`",
		inConstruct: ["codeFencedLangGraveAccent", "codeFencedMetaGraveAccent"]
	},
	{
		character: "`",
		inConstruct: "phrasing",
		notInConstruct: Ki
	},
	{
		atBreak: !0,
		character: "~"
	},
	{
		character: "<",
		inConstruct: "autolink"
	},
	{
		character: ">",
		inConstruct: "autolink"
	},
	{
		character: "",
		inConstruct: "autolink"
	}
], Ji = -1; ++Ji < 33;) qi.push({
	character: String.fromCharCode(Ji),
	inConstruct: "autolink"
});
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/association.js
function Yi(e) {
	return e.label || !e.identifier ? e.label || "" : mr(e.identifier);
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/compile-pattern.js
function Xi(e) {
	if (!e._compiled) {
		let t = (e.atBreak ? "[\\r\\n][\\t ]*" : "") + (e.before ? "(?:" + e.before + ")" : "");
		e._compiled = RegExp((e.beforeNode ? "^" : t ? "(" + t + ")" : "") + (/[$()*+.?[\\\]^{|}]/.test(e.character) ? "\\" : "") + e.character + (e.afterNode ? "$" : e.after ? "(?:" + e.after + ")" : ""), e.unicode ? "gu" : "g");
	}
	return e._compiled;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/character.js
function Zi(e) {
	let t = e.codePointAt(0);
	return t === void 0 ? "" : String.fromCodePoint(t);
}
function Qi(e) {
	let t = e.codePointAt(e.length - 2);
	return t !== void 0 && t > 65535 ? e.slice(-2) : e.slice(-1);
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/encode-info.js
function $i(e, t, n) {
	let r = at(e), i = at(t);
	return r === void 0 ? i === void 0 ? n === "_" ? {
		inside: !0,
		outside: !0
	} : {
		inside: !1,
		outside: !1
	} : i === 1 ? {
		inside: !0,
		outside: !0
	} : {
		inside: !1,
		outside: !0
	} : r === 1 ? i === void 0 ? {
		inside: !1,
		outside: !1
	} : i === 1 ? {
		inside: !0,
		outside: !0
	} : {
		inside: !1,
		outside: !1
	} : i === void 0 ? {
		inside: !1,
		outside: !1
	} : i === 1 ? {
		inside: !0,
		outside: !1
	} : {
		inside: !1,
		outside: !1
	};
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/container-phrasing.js
var ea = /* @__PURE__ */ new Map([["*", !1], ["_", !0]]);
function ta(e, t, n) {
	let r = /* @__PURE__ */ new Set();
	for (let i of e) {
		let e = n.get(i);
		if (e && i.attention && t.some((e) => e.attention === i.attention)) for (let t of e.tokens) r.add(t.attention);
	}
	let i = [];
	for (let t of e) t.attention && r.delete(t.attention) && i.push(t.attention);
	return i.reverse(), i;
}
function na(e, t, n) {
	let r = [], i = /* @__PURE__ */ new Map(), a = /* @__PURE__ */ new Map(), o = 0;
	for (; o < e.length;) {
		if (!e[o].attention) {
			o++;
			continue;
		}
		let s = e[o].value.charAt(0), c = ea.get(s), l = o + 1;
		for (; l < e.length && e[l].attention && e[l].value.charAt(0) === s;) l++;
		let u = o ? e[o - 1].value : t, d = at(u.charCodeAt(u.length - 1)), f = at((l < e.length ? e[l].value : n).charCodeAt(0)), p = !f || f === 2 && !!d, m = !d || d === 2 && !!f, h = {
			tokens: e.slice(o, l),
			markers: [],
			start: 0,
			end: 0,
			open: c ? p && (!!d || !m) : p,
			close: c ? m && (!!f || !p) : m,
			split: c !== void 0
		};
		for (let e of h.tokens) {
			i.set(e, [...h.tokens]), a.set(e, h);
			let t = e.value.length;
			for (; t--;) h.markers.push(e);
		}
		h.end = h.markers.length, r.push(h), o = l;
	}
	for (oa(r, i), o = -1; ++o < e.length;) {
		let t = i.get(e[o]);
		if (t) return {
			attention: ta(e, t, a),
			index: o
		};
	}
}
function ra(e, t, n) {
	return ua(ca(e, t, n), n.before, n.after);
}
function ia(e, t) {
	let n = e[t ? 0 : e.length - 1];
	if (!n || n.attention) return;
	let r = t ? Zi(n.value) : Qi(n.value), i = ii(r.codePointAt(0));
	n.value = t ? i + n.value.slice(r.length) : n.value.slice(0, n.value.length - r.length) + i;
}
function aa(e, t, n, r, i) {
	for (let a of n.attention) {
		let o = t.get(a) || a.sequences[0];
		for (let s of a.sequences) {
			if (s === o) continue;
			let c = new Map(t);
			c.set(a, s);
			let l = la(e, c, r, i), u = na(l, r, i);
			if (!u || u.index > n.index) return {
				chosen: c,
				mistake: u,
				tokens: l
			};
		}
	}
}
function oa(e, t) {
	let n = 0;
	for (; n < e.length;) {
		let r = e[n], i = r.tokens[0].value.charAt(0), a = r.end - r.start, o = r.close && a ? n : 0;
		for (; o--;) {
			let n = e[o], s = n.end - n.start;
			if (n.tokens[0].value.charAt(0) === i && n.open && s && (r.split ? !((n.close || r.open) && a % 3) || (s + a) % 3 : s === n.markers.length && s === a)) {
				let e = r.split ? s > 1 && a > 1 ? 2 : 1 : a, i = n.markers.slice(n.end - e, n.end), o = r.markers.slice(r.start, r.start + e), c = i[0], l = o[0];
				if (c === i[e - 1] && c.value.length === e && l === o[e - 1] && l.value.length === e && c.attention === l.attention) t.delete(c), t.delete(l);
				else {
					let e = [.../* @__PURE__ */ new Set([...i, ...o])];
					for (let n of e) {
						let r = t.get(n);
						r && r.push(...e);
					}
				}
				n.end -= e, r.start += e;
				break;
			}
		}
		(o === -1 || r.start === r.end) && n++;
	}
}
function sa(e, t, n) {
	if (t.length === 0) throw Error("Cannot serialize `" + e + "` as attention without markers, expected one or more markers");
	if (n.length === 0) throw Error("Cannot serialize `" + e + "` as attention without sizes, expected one or more sizes");
	for (let t of n) if (!Number.isSafeInteger(t) || t < 1) throw Error("Cannot serialize `" + e + "` as attention with `" + t + "` as size, expected positive integer");
	let r = [];
	for (let i of t) {
		if (i.length !== 1) throw Error("Cannot serialize `" + e + "` as attention with `" + i + "` as marker, expected a single ascii character");
		if (!Ke(i.charCodeAt(0))) throw Error("Cannot serialize `" + e + "` as attention with `" + i + "` as marker, expected ascii punctuation");
		for (let e of n) r.push(i.repeat(e));
	}
	return r;
}
function ca(e, t, n) {
	let r = t.indexStack, i = e.children || [], a = [], o = -1, s = n.before;
	r.push(-1);
	let c = t.createTracker(n);
	for (; ++o < i.length;) {
		let l = i[o], u;
		if (r[r.length - 1] = o, o + 1 < i.length) {
			let n = t.handle.handlers[i[o + 1].type];
			n && n.peek && (n = n.peek), u = n ? Zi(n(i[o + 1], e, t, {
				before: "",
				after: "",
				...c.current()
			})) : "";
		} else u = n.after;
		let d = a[a.length - 1];
		typeof d == "string" && (s === "\r" || s === "\n") && l.type === "html" && Ui(l.value) !== 7 && (a[a.length - 1] = d.replace(/(\r?\n|\r)$/, " "), s = " ", c = t.createTracker(n), c.move(ua(a, n.before, n.after)));
		let f = t.handle.handlers[l.type];
		if (f && f.attention) {
			let { construct: e, markers: n, sizes: r } = f.attention(l, t), i = sa(l.type, n, r), o = i[0], u = t.enter(e);
			c.move(o);
			let d = ca(l, t, {
				...c.current(),
				before: o,
				after: o
			});
			c.move(ua(d, o, o)), c.move(o), u(), a.push({
				children: d,
				sequences: i
			}), s = o.charAt(o.length - 1);
		} else {
			let n = t.handle(l, e, t, {
				...c.current(),
				after: u,
				before: s
			});
			if (!n) continue;
			c.move(n), a.push(n), s = Qi(n);
		}
	}
	return r.pop(), a;
}
function la(e, t, n, r) {
	let i = [], a, o = Qi(n), s = -1;
	for (; ++s < e.length;) {
		let n = e[s];
		if (typeof n == "string") {
			let e = {
				value: n,
				attention: void 0
			};
			a && a === Zi(n) && ia([e], !0), a = void 0, i.push(e), e.value && (o = Qi(e.value));
			continue;
		}
		let c = t.get(n) || n.sequences[0], l = c.charAt(0), u = e[s + 1], d = o, f = Zi(u === void 0 ? r : typeof u == "string" ? u : t.get(u) || u.sequences[0]), p = la(n.children, t, c, c), m = p.length > 0 ? p[0].value : "", h = $i(d.charCodeAt(d.length - 1), m.charCodeAt(0), l);
		h.inside && ia(p, !0);
		let g = p.length > 0 ? Qi(p[p.length - 1].value) : "", _ = $i(f.charCodeAt(0), g.charCodeAt(g.length - 1), l);
		_.inside && ia(p, !1), h.outside && d !== "\n" && d !== "\r" && ia(i, !1), _.outside && (a = f), i.push({
			value: c,
			attention: n
		}, ...p, {
			value: c,
			attention: n
		}), o = c.charAt(c.length - 1);
	}
	return i;
}
function ua(e, t, n) {
	let r = /* @__PURE__ */ new Map(), i = la(e, r, t, n), a = i, o = na(a, t, n);
	for (; o;) {
		let s = aa(e, r, o, t, n);
		if (!s) {
			a = i;
			break;
		}
		r = s.chosen, a = s.tokens, o = s.mistake;
	}
	let s = "";
	for (let e of a) s += e.value;
	return s;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/container-flow.js
function da(e, t, n) {
	let r = t.indexStack, i = e.children || [], a = t.createTracker(n), o = [], s = -1;
	for (r.push(-1); ++s < i.length;) {
		let n = i[s];
		r[r.length - 1] = s, o.push(a.move(t.handle(n, e, t, {
			before: "\n",
			after: "\n",
			...a.current()
		}))), n.type !== "list" && (t.bulletLastUsed = void 0), s < i.length - 1 && o.push(a.move(fa(n, i[s + 1], e, t)));
	}
	return r.pop(), o.join("");
}
function fa(e, t, n, r) {
	let i = r.join.length;
	for (; i--;) {
		let a = r.join[i](e, t, n, r);
		if (a === !0 || a === 1) break;
		if (typeof a == "number") return "\n".repeat(1 + a);
		if (a === !1) return "\n\n<!---->\n\n";
	}
	return "\n\n";
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/indent-lines.js
var pa = /\r?\n|\r/g;
function ma(e, t) {
	let n = [], r = 0, i = 0, a;
	for (; a = pa.exec(e);) o(e.slice(r, a.index)), n.push(a[0]), r = a.index + a[0].length, i++;
	return o(e.slice(r)), n.join("");
	function o(e) {
		n.push(t(e, i, !e));
	}
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/safe.js
var ha = {}.hasOwnProperty;
function ga(e, t, n) {
	let r = (n.before || "") + (t || "") + (n.after || ""), i = [], a = [], o = {}, s = e.stack.includes("autolink"), c = n.before ? n.before.length : 0, l = r.length - (n.after ? n.after.length : 0), u = -1;
	for (; ++u < e.unsafe.length;) {
		let t = e.unsafe[u];
		if (!Wr(e.stack, t) || !_a(t.beforeNode, n.beforeNode) || !_a(t.afterNode, n.afterNode)) continue;
		let a = e.compilePattern(t), s = t.beforeNode ? c : 0, d = r.slice(s, t.afterNode ? l : void 0), f;
		for (; f = a.exec(d);) {
			let e = "before" in t || !!t.atBreak, n = "after" in t, r = s + f.index + (e ? f[1].length : 0);
			ha.call(o, r) ? (o[r].before && !e && (o[r].before = !1), o[r].after && !n && (o[r].after = !1)) : (i.push(r), o[r] = {
				before: e,
				after: n
			});
		}
	}
	i.sort(va);
	let d = c;
	for (u = -1; ++u < i.length;) {
		let e = i[u];
		if (!(e < d || e >= l)) {
			if (r.charAt(e) === "_" && e > 0 && at(r.charCodeAt(e - 1)) === void 0 && !ha.call(o, e - 1) && !(e - 1 === c && /[*_]/.test(r.charAt(c - 1)))) {
				let t = e + 1;
				for (; t < l && r.charAt(t) === "_";) t++;
				let n = t - e - 1;
				if (i[u + n] === t - 1 && t < r.length && at(r.charCodeAt(t)) === void 0 && !ha.call(o, t) && !(t === l - 1 && /[*_]/.test(r.charAt(l)))) {
					u += n;
					continue;
				}
			}
			if (!(e + 1 < l && i[u + 1] === e + 1 && o[e].after && !o[e + 1].before && !o[e + 1].after || i[u - 1] === e - 1 && o[e].before && !o[e - 1].before && !o[e - 1].after)) {
				if (d !== e) {
					let t = r.slice(d, e);
					a.push(s ? t : ya(t, "\\"));
				}
				d = e, s ? (a.push("%" + r.charCodeAt(e).toString(16).toUpperCase().padStart(2, "0")), d++) : /[!-/:-@[-`{-~]/.test(r.charAt(e)) && (!n.encode || !n.encode.includes(r.charAt(e))) ? a.push("\\") : (a.push(ii(r.charCodeAt(e))), d++);
			}
		}
	}
	let f = r.slice(d, l);
	return a.push(s ? f : ya(f, n.after)), a.join("");
}
function _a(e, t) {
	return e ? t ? typeof e == "string" ? e === t.type : e.includes(t.type) : !1 : !0;
}
function va(e, t) {
	return e - t;
}
function ya(e, t) {
	let n = /\\(?=[!-/:-@[-`{-~])/g, r = [], i = [], a = e + t, o = -1, s = 0, c;
	for (; c = n.exec(a);) r.push(c.index);
	for (; ++o < r.length;) s !== r[o] && i.push(e.slice(s, r[o])), i.push("\\"), s = r[o];
	return i.push(e.slice(s)), i.join("");
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/util/track.js
function ba(e) {
	/* c8 ignore next 5 */
	let t = e || {}, n = t.now || {}, r = t.lineShift || 0, i = n.line || 1, a = n.column || 1;
	return {
		move: c,
		current: o,
		shift: s
	};
	function o() {
		return {
			now: {
				line: i,
				column: a
			},
			lineShift: r
		};
	}
	function s(e) {
		r += e;
	}
	function c(e) {
		let t = e || "", n = t.split(/\r?\n|\r/g), o = n[n.length - 1];
		return i += n.length - 1, a = n.length === 1 ? a + o.length : 1 + o.length + r, t;
	}
}
//#endregion
//#region node_modules/.pnpm/mdast-util-to-markdown@2.2.0/node_modules/mdast-util-to-markdown/lib/index.js
function xa(e, t) {
	let n = t || {}, r = {
		associationId: Yi,
		bulletCurrent: void 0,
		bulletLastUsed: void 0,
		containerPhrasing: Ta,
		containerFlow: Ea,
		createTracker: ba,
		compilePattern: Xi,
		enter: a,
		handlers: { ...Hi },
		handle: void 0,
		indentLines: ma,
		indexStack: [],
		join: [...Wi],
		options: {},
		safe: Da,
		stack: [],
		unsafe: [...qi]
	};
	zr(r, n), r.options.tightDefinitions && r.join.push(wa), r.handle = Lr("type", {
		invalid: Sa,
		unknown: Ca,
		handlers: r.handlers
	});
	let i = r.handle(Fr(e) ? {
		type: "root",
		children: [e]
	} : e, void 0, r, {
		before: "\n",
		after: "\n",
		now: {
			line: 1,
			column: 1
		},
		lineShift: 0
	});
	return i && i.charCodeAt(i.length - 1) !== 10 && i.charCodeAt(i.length - 1) !== 13 && (i += "\n"), i;
	function a(e) {
		return r.stack.push(e), t;
		function t() {
			r.stack.pop();
		}
	}
}
function Sa(e) {
	throw Error("Cannot handle value `" + e + "`, expected node");
}
function Ca(e) {
	throw Error("Cannot handle unknown node `" + e.type + "`");
}
function wa(e, t) {
	if (e.type === "definition" && e.type === t.type) return 0;
}
function Ta(e, t) {
	return ra(e, this, t);
}
function Ea(e, t) {
	return da(e, this, t);
}
function Da(e, t) {
	return ga(this, e, t);
}
//#endregion
//#region node_modules/.pnpm/remark-stringify@11.0.0/node_modules/remark-stringify/lib/index.js
function Oa(e) {
	let t = this;
	t.compiler = n;
	function n(n) {
		return xa(n, {
			...t.data("settings"),
			...e,
			extensions: t.data("toMarkdownExtensions") || []
		});
	}
}
//#endregion
//#region node_modules/.pnpm/bail@2.0.2/node_modules/bail/index.js
function ka(e) {
	if (e) throw e;
}
//#endregion
//#region node_modules/.pnpm/extend@3.0.2/node_modules/extend/index.js
var Aa = /* @__PURE__ */ a(((e, t) => {
	var n = Object.prototype.hasOwnProperty, r = Object.prototype.toString, i = Object.defineProperty, a = Object.getOwnPropertyDescriptor, o = function(e) {
		return typeof Array.isArray == "function" ? Array.isArray(e) : r.call(e) === "[object Array]";
	}, s = function(e) {
		if (!e || r.call(e) !== "[object Object]") return !1;
		var t = n.call(e, "constructor"), i = e.constructor && e.constructor.prototype && n.call(e.constructor.prototype, "isPrototypeOf");
		if (e.constructor && !t && !i) return !1;
		for (var a in e);
		return a === void 0 || n.call(e, a);
	}, c = function(e, t) {
		i && t.name === "__proto__" ? i(e, t.name, {
			enumerable: !0,
			configurable: !0,
			value: t.newValue,
			writable: !0
		}) : e[t.name] = t.newValue;
	}, l = function(e, t) {
		if (t === "__proto__") {
			if (!n.call(e, t)) return;
			if (a) return a(e, t).value;
		}
		return e[t];
	};
	t.exports = function e() {
		var t, n, r, i, a, u, d = arguments[0], f = 1, p = arguments.length, m = !1;
		for (typeof d == "boolean" && (m = d, d = arguments[1] || {}, f = 2), (d == null || typeof d != "object" && typeof d != "function") && (d = {}); f < p; ++f) if (t = arguments[f], t != null) for (n in t) r = l(d, n), i = l(t, n), d !== i && (m && i && (s(i) || (a = o(i))) ? (a ? (a = !1, u = r && o(r) ? r : []) : u = r && s(r) ? r : {}, c(d, {
			name: n,
			newValue: e(m, u, i)
		})) : i !== void 0 && c(d, {
			name: n,
			newValue: i
		}));
		return d;
	};
}));
//#endregion
//#region node_modules/.pnpm/is-plain-obj@4.1.0/node_modules/is-plain-obj/index.js
function ja(e) {
	if (typeof e != "object" || !e) return !1;
	let t = Object.getPrototypeOf(e);
	return (t === null || t === Object.prototype || Object.getPrototypeOf(t) === null) && !(Symbol.toStringTag in e) && !(Symbol.iterator in e);
}
//#endregion
//#region node_modules/.pnpm/trough@2.2.0/node_modules/trough/lib/index.js
function Ma() {
	let e = [], t = {
		run: n,
		use: r
	};
	return t;
	function n(...t) {
		let n = -1, r = t.pop();
		if (typeof r != "function") throw TypeError("Expected function as last argument, not " + r);
		i(null, ...t);
		function i(a, ...o) {
			let s = e[++n], c = -1;
			if (a) r(a);
			else {
				for (; ++c < t.length;) (o[c] === null || o[c] === void 0) && (o[c] = t[c]);
				t = o, s ? Na(s, i)(...o) : r(null, ...o);
			}
		}
	}
	function r(n) {
		if (typeof n != "function") throw TypeError("Expected `middelware` to be a function, not " + n);
		return e.push(n), t;
	}
}
function Na(e, t) {
	let n;
	return r;
	function r(...t) {
		let r = e.length > t.length, o;
		r && t.push(i);
		try {
			o = e.apply(this, t);
		} catch (e) {
			let t = e;
			if (r && n) throw t;
			return i(t);
		}
		r || (o && o.then && typeof o.then == "function" ? o.then(a, i) : o instanceof Error ? i(o) : a(o));
	}
	function i(e, ...r) {
		n || (n = !0, t(e, ...r));
	}
	function a(e) {
		i(null, e);
	}
}
//#endregion
//#region node_modules/.pnpm/vfile-message@4.0.3/node_modules/vfile-message/lib/index.js
var Pa = class extends Error {
	constructor(e, t, n) {
		super(), typeof t == "string" && (n = t, t = void 0);
		let r = "", i = {}, a = !1;
		if (t && (i = "line" in t && "column" in t || "start" in t && "end" in t ? { place: t } : "type" in t ? {
			ancestors: [t],
			place: t.position
		} : { ...t }), typeof e == "string" ? r = e : !i.cause && e && (a = !0, r = e.message, i.cause = e), !i.ruleId && !i.source && typeof n == "string") {
			let e = n.indexOf(":");
			e === -1 ? i.ruleId = n : (i.source = n.slice(0, e), i.ruleId = n.slice(e + 1));
		}
		if (!i.place && i.ancestors && i.ancestors) {
			let e = i.ancestors[i.ancestors.length - 1];
			e && (i.place = e.position);
		}
		let o = i.place && "start" in i.place ? i.place.start : i.place;
		this.ancestors = i.ancestors || void 0, this.cause = i.cause || void 0, this.column = o ? o.column : void 0, this.fatal = void 0, this.file = "", this.message = r, this.line = o ? o.line : void 0, this.name = gr(i.place) || "1:1", this.place = i.place || void 0, this.reason = this.message, this.ruleId = i.ruleId || void 0, this.source = i.source || void 0, this.stack = a && i.cause && typeof i.cause.stack == "string" ? i.cause.stack : "", this.actual = void 0, this.expected = void 0, this.note = void 0, this.url = void 0;
	}
};
Pa.prototype.file = "", Pa.prototype.name = "", Pa.prototype.reason = "", Pa.prototype.message = "", Pa.prototype.stack = "", Pa.prototype.column = void 0, Pa.prototype.line = void 0, Pa.prototype.ancestors = void 0, Pa.prototype.cause = void 0, Pa.prototype.fatal = void 0, Pa.prototype.place = void 0, Pa.prototype.ruleId = void 0, Pa.prototype.source = void 0;
//#endregion
//#region node_modules/.pnpm/vfile@6.0.3/node_modules/vfile/lib/minpath.browser.js
var Fa = {
	basename: Ia,
	dirname: La,
	extname: Ra,
	join: za,
	sep: "/"
};
function Ia(e, t) {
	if (t !== void 0 && typeof t != "string") throw TypeError("\"ext\" argument must be a string");
	Ha(e);
	let n = 0, r = -1, i = e.length, a;
	if (t === void 0 || t.length === 0 || t.length > e.length) {
		for (; i--;) if (e.codePointAt(i) === 47) {
			if (a) {
				n = i + 1;
				break;
			}
		} else r < 0 && (a = !0, r = i + 1);
		return r < 0 ? "" : e.slice(n, r);
	}
	if (t === e) return "";
	let o = -1, s = t.length - 1;
	for (; i--;) if (e.codePointAt(i) === 47) {
		if (a) {
			n = i + 1;
			break;
		}
	} else o < 0 && (a = !0, o = i + 1), s > -1 && (e.codePointAt(i) === t.codePointAt(s--) ? s < 0 && (r = i) : (s = -1, r = o));
	return n === r ? r = o : r < 0 && (r = e.length), e.slice(n, r);
}
function La(e) {
	if (Ha(e), e.length === 0) return ".";
	let t = -1, n = e.length, r;
	for (; --n;) if (e.codePointAt(n) === 47) {
		if (r) {
			t = n;
			break;
		}
	} else r ||= !0;
	return t < 0 ? e.codePointAt(0) === 47 ? "/" : "." : t === 1 && e.codePointAt(0) === 47 ? "//" : e.slice(0, t);
}
function Ra(e) {
	Ha(e);
	let t = e.length, n = -1, r = 0, i = -1, a = 0, o;
	for (; t--;) {
		let s = e.codePointAt(t);
		if (s === 47) {
			if (o) {
				r = t + 1;
				break;
			}
		} else n < 0 && (o = !0, n = t + 1), s === 46 ? i < 0 ? i = t : a !== 1 && (a = 1) : i > -1 && (a = -1);
	}
	return i < 0 || n < 0 || a === 0 || a === 1 && i === n - 1 && i === r + 1 ? "" : e.slice(i, n);
}
function za(...e) {
	let t = -1, n;
	for (; ++t < e.length;) Ha(e[t]), e[t] && (n = n === void 0 ? e[t] : n + "/" + e[t]);
	return n === void 0 ? "." : Ba(n);
}
function Ba(e) {
	Ha(e);
	let t = e.codePointAt(0) === 47, n = Va(e, !t);
	return n.length === 0 && !t && (n = "."), n.length > 0 && e.codePointAt(e.length - 1) === 47 && (n += "/"), t ? "/" + n : n;
}
function Va(e, t) {
	let n = "", r = 0, i = -1, a = 0, o = -1, s, c;
	for (; ++o <= e.length;) {
		if (o < e.length) s = e.codePointAt(o);
		else if (s === 47) break;
		else s = 47;
		if (s === 47) {
			if (i !== o - 1 && a !== 1) {
				if (i !== o - 1 && a === 2) {
					if (n.length < 2 || r !== 2 || n.codePointAt(n.length - 1) !== 46 || n.codePointAt(n.length - 2) !== 46) {
						if (n.length > 2) {
							if (c = n.lastIndexOf("/"), c !== n.length - 1) {
								c < 0 ? (n = "", r = 0) : (n = n.slice(0, c), r = n.length - 1 - n.lastIndexOf("/")), i = o, a = 0;
								continue;
							}
						} else if (n.length > 0) {
							n = "", r = 0, i = o, a = 0;
							continue;
						}
					}
					t && (n = n.length > 0 ? n + "/.." : "..", r = 2);
				} else n.length > 0 ? n += "/" + e.slice(i + 1, o) : n = e.slice(i + 1, o), r = o - i - 1;
			}
			i = o, a = 0;
		} else s === 46 && a > -1 ? a++ : a = -1;
	}
	return n;
}
function Ha(e) {
	if (typeof e != "string") throw TypeError("Path must be a string. Received " + JSON.stringify(e));
}
//#endregion
//#region node_modules/.pnpm/vfile@6.0.3/node_modules/vfile/lib/minproc.browser.js
var Ua = { cwd: Wa };
function Wa() {
	return "/";
}
//#endregion
//#region node_modules/.pnpm/vfile@6.0.3/node_modules/vfile/lib/minurl.shared.js
function Ga(e) {
	return !!(typeof e == "object" && e && "href" in e && e.href && "protocol" in e && e.protocol && e.auth === void 0);
}
//#endregion
//#region node_modules/.pnpm/vfile@6.0.3/node_modules/vfile/lib/minurl.browser.js
function Ka(e) {
	if (typeof e == "string") e = new URL(e);
	else if (!Ga(e)) {
		let t = /* @__PURE__ */ TypeError("The \"path\" argument must be of type string or an instance of URL. Received `" + e + "`");
		throw t.code = "ERR_INVALID_ARG_TYPE", t;
	}
	if (e.protocol !== "file:") {
		let e = /* @__PURE__ */ TypeError("The URL must be of scheme file");
		throw e.code = "ERR_INVALID_URL_SCHEME", e;
	}
	return qa(e);
}
function qa(e) {
	if (e.hostname !== "") {
		let e = /* @__PURE__ */ TypeError("File URL host must be \"localhost\" or empty on darwin");
		throw e.code = "ERR_INVALID_FILE_URL_HOST", e;
	}
	let t = e.pathname, n = -1;
	for (; ++n < t.length;) if (t.codePointAt(n) === 37 && t.codePointAt(n + 1) === 50) {
		let e = t.codePointAt(n + 2);
		if (e === 70 || e === 102) {
			let e = /* @__PURE__ */ TypeError("File URL path must not include encoded / characters");
			throw e.code = "ERR_INVALID_FILE_URL_PATH", e;
		}
	}
	return decodeURIComponent(t);
}
//#endregion
//#region node_modules/.pnpm/vfile@6.0.3/node_modules/vfile/lib/index.js
var Ja = [
	"history",
	"path",
	"basename",
	"stem",
	"extname",
	"dirname"
], Ya = class {
	constructor(e) {
		let t;
		t = e ? Ga(e) ? { path: e } : typeof e == "string" || $a(e) ? { value: e } : e : {}, this.cwd = "cwd" in t ? "" : Ua.cwd(), this.data = {}, this.history = [], this.messages = [], this.value, this.map, this.result, this.stored;
		let n = -1;
		for (; ++n < Ja.length;) {
			let e = Ja[n];
			e in t && t[e] !== void 0 && t[e] !== null && (this[e] = e === "history" ? [...t[e]] : t[e]);
		}
		let r;
		for (r in t) Ja.includes(r) || (this[r] = t[r]);
	}
	get basename() {
		return typeof this.path == "string" ? Fa.basename(this.path) : void 0;
	}
	set basename(e) {
		Za(e, "basename"), Xa(e, "basename"), this.path = Fa.join(this.dirname || "", e);
	}
	get dirname() {
		return typeof this.path == "string" ? Fa.dirname(this.path) : void 0;
	}
	set dirname(e) {
		Qa(this.basename, "dirname"), this.path = Fa.join(e || "", this.basename);
	}
	get extname() {
		return typeof this.path == "string" ? Fa.extname(this.path) : void 0;
	}
	set extname(e) {
		if (Xa(e, "extname"), Qa(this.dirname, "extname"), e) {
			if (e.codePointAt(0) !== 46) throw Error("`extname` must start with `.`");
			if (e.includes(".", 1)) throw Error("`extname` cannot contain multiple dots");
		}
		this.path = Fa.join(this.dirname, this.stem + (e || ""));
	}
	get path() {
		return this.history[this.history.length - 1];
	}
	set path(e) {
		Ga(e) && (e = Ka(e)), Za(e, "path"), this.path !== e && this.history.push(e);
	}
	get stem() {
		return typeof this.path == "string" ? Fa.basename(this.path, this.extname) : void 0;
	}
	set stem(e) {
		Za(e, "stem"), Xa(e, "stem"), this.path = Fa.join(this.dirname || "", e + (this.extname || ""));
	}
	fail(e, t, n) {
		let r = this.message(e, t, n);
		throw r.fatal = !0, r;
	}
	info(e, t, n) {
		let r = this.message(e, t, n);
		return r.fatal = void 0, r;
	}
	message(e, t, n) {
		let r = new Pa(e, t, n);
		return this.path && (r.name = this.path + ":" + r.name, r.file = this.path), r.fatal = !1, this.messages.push(r), r;
	}
	toString(e) {
		return this.value === void 0 ? "" : typeof this.value == "string" ? this.value : new TextDecoder(e || void 0).decode(this.value);
	}
};
function Xa(e, t) {
	if (e && e.includes(Fa.sep)) throw Error("`" + t + "` cannot be a path: did not expect `" + Fa.sep + "`");
}
function Za(e, t) {
	if (!e) throw Error("`" + t + "` cannot be empty");
}
function Qa(e, t) {
	if (!e) throw Error("Setting `" + t + "` requires `path` to be set too");
}
function $a(e) {
	return !!(e && typeof e == "object" && "byteLength" in e && "byteOffset" in e);
}
//#endregion
//#region node_modules/.pnpm/unified@11.0.5/node_modules/unified/lib/callable-instance.js
var eo = (function(e) {
	let t = this.constructor.prototype, n = t[e], r = function() {
		return n.apply(r, arguments);
	};
	return Object.setPrototypeOf(r, t), r;
}), to = /* @__PURE__ */ _(Aa(), 1), no = {}.hasOwnProperty, ro = new class e extends eo {
	constructor() {
		super("copy"), this.Compiler = void 0, this.Parser = void 0, this.attachers = [], this.compiler = void 0, this.freezeIndex = -1, this.frozen = void 0, this.namespace = {}, this.parser = void 0, this.transformers = Ma();
	}
	copy() {
		let t = new e(), n = -1;
		for (; ++n < this.attachers.length;) {
			let e = this.attachers[n];
			t.use(...e);
		}
		return t.data((0, to.default)(!0, {}, this.namespace)), t;
	}
	data(e, t) {
		return typeof e == "string" ? arguments.length === 2 ? (oo("data", this.frozen), this.namespace[e] = t, this) : no.call(this.namespace, e) && this.namespace[e] || void 0 : e ? (oo("data", this.frozen), this.namespace = e, this) : this.namespace;
	}
	freeze() {
		if (this.frozen) return this;
		let e = this;
		for (; ++this.freezeIndex < this.attachers.length;) {
			let [t, ...n] = this.attachers[this.freezeIndex];
			if (n[0] === !1) continue;
			n[0] === !0 && (n[0] = void 0);
			let r = t.call(e, ...n);
			typeof r == "function" && this.transformers.use(r);
		}
		return this.frozen = !0, this.freezeIndex = Infinity, this;
	}
	parse(e) {
		this.freeze();
		let t = lo(e), n = this.parser || this.Parser;
		return io("parse", n), n(String(t), t);
	}
	process(e, t) {
		let n = this;
		return this.freeze(), io("process", this.parser || this.Parser), ao("process", this.compiler || this.Compiler), t ? r(void 0, t) : new Promise(r);
		function r(r, i) {
			let a = lo(e), o = n.parse(a);
			n.run(o, a, function(e, t, r) {
				if (e || !t || !r) return s(e);
				let i = t, a = n.stringify(i, r);
				fo(a) ? r.value = a : r.result = a, s(e, r);
			});
			function s(e, n) {
				e || !n ? i(e) : r ? r(n) : t(void 0, n);
			}
		}
	}
	processSync(e) {
		let t = !1, n;
		return this.freeze(), io("processSync", this.parser || this.Parser), ao("processSync", this.compiler || this.Compiler), this.process(e, r), co("processSync", "process", t), n;
		function r(e, r) {
			t = !0, ka(e), n = r;
		}
	}
	run(e, t, n) {
		so(e), this.freeze();
		let r = this.transformers;
		return !n && typeof t == "function" && (n = t, t = void 0), n ? i(void 0, n) : new Promise(i);
		function i(i, a) {
			let o = lo(t);
			r.run(e, o, s);
			function s(t, r, o) {
				let s = r || e;
				t ? a(t) : i ? i(s) : n(void 0, s, o);
			}
		}
	}
	runSync(e, t) {
		let n = !1, r;
		return this.run(e, t, i), co("runSync", "run", n), r;
		function i(e, t) {
			ka(e), r = t, n = !0;
		}
	}
	stringify(e, t) {
		this.freeze();
		let n = lo(t), r = this.compiler || this.Compiler;
		return ao("stringify", r), so(e), r(e, n);
	}
	use(e, ...t) {
		let n = this.attachers, r = this.namespace;
		if (oo("use", this.frozen), e != null) {
			if (typeof e == "function") s(e, t);
			else if (typeof e == "object") Array.isArray(e) ? o(e) : a(e);
			else throw TypeError("Expected usable value, not `" + e + "`");
		}
		return this;
		function i(e) {
			if (typeof e == "function") s(e, []);
			else if (typeof e == "object") {
				if (Array.isArray(e)) {
					let [t, ...n] = e;
					s(t, n);
				} else a(e);
			} else throw TypeError("Expected usable value, not `" + e + "`");
		}
		function a(e) {
			if (!("plugins" in e) && !("settings" in e)) throw Error("Expected usable value but received an empty preset, which is probably a mistake: presets typically come with `plugins` and sometimes with `settings`, but this has neither");
			o(e.plugins), e.settings && (r.settings = (0, to.default)(!0, r.settings, e.settings));
		}
		function o(e) {
			let t = -1;
			if (e != null) {
				if (Array.isArray(e)) for (; ++t < e.length;) {
					let n = e[t];
					i(n);
				}
				else throw TypeError("Expected a list of plugins, not `" + e + "`");
			}
		}
		function s(e, t) {
			let r = -1, i = -1;
			for (; ++r < n.length;) if (n[r][0] === e) {
				i = r;
				break;
			}
			if (i === -1) n.push([e, ...t]);
			else if (t.length > 0) {
				let [r, ...a] = t, o = n[i][1];
				ja(o) && ja(r) && (r = (0, to.default)(!0, o, r)), n[i] = [
					e,
					r,
					...a
				];
			}
		}
	}
}().freeze();
function io(e, t) {
	if (typeof t != "function") throw TypeError("Cannot `" + e + "` without `parser`");
}
function ao(e, t) {
	if (typeof t != "function") throw TypeError("Cannot `" + e + "` without `compiler`");
}
function oo(e, t) {
	if (t) throw Error("Cannot call `" + e + "` on a frozen processor.\nCreate a new processor first, by calling it: use `processor()` instead of `processor`.");
}
function so(e) {
	if (!ja(e) || typeof e.type != "string") throw TypeError("Expected node, got `" + e + "`");
}
function co(e, t, n) {
	if (!n) throw Error("`" + e + "` finished async. Use `" + t + "` instead");
}
function lo(e) {
	return uo(e) ? e : new Ya(e);
}
function uo(e) {
	return !!(e && typeof e == "object" && "message" in e && "messages" in e);
}
function fo(e) {
	return typeof e == "string" || po(e);
}
function po(e) {
	return !!(e && typeof e == "object" && "byteLength" in e && "byteOffset" in e);
}
//#endregion
//#region node_modules/.pnpm/orderedmap@2.1.1/node_modules/orderedmap/dist/index.js
function M(e) {
	this.content = e;
}
M.prototype = {
	constructor: M,
	find: function(e) {
		for (var t = 0; t < this.content.length; t += 2) if (this.content[t] === e) return t;
		return -1;
	},
	get: function(e) {
		var t = this.find(e);
		return t == -1 ? void 0 : this.content[t + 1];
	},
	update: function(e, t, n) {
		var r = n && n != e ? this.remove(n) : this, i = r.find(e), a = r.content.slice();
		return i == -1 ? a.push(n || e, t) : (a[i + 1] = t, n && (a[i] = n)), new M(a);
	},
	remove: function(e) {
		var t = this.find(e);
		if (t == -1) return this;
		var n = this.content.slice();
		return n.splice(t, 2), new M(n);
	},
	addToStart: function(e, t) {
		return new M([e, t].concat(this.remove(e).content));
	},
	addToEnd: function(e, t) {
		var n = this.remove(e).content.slice();
		return n.push(e, t), new M(n);
	},
	addBefore: function(e, t, n) {
		var r = this.remove(t), i = r.content.slice(), a = r.find(e);
		return i.splice(a == -1 ? i.length : a, 0, t, n), new M(i);
	},
	forEach: function(e) {
		for (var t = 0; t < this.content.length; t += 2) e(this.content[t], this.content[t + 1]);
	},
	prepend: function(e) {
		return e = M.from(e), e.size ? new M(e.content.concat(this.subtract(e).content)) : this;
	},
	append: function(e) {
		return e = M.from(e), e.size ? new M(this.subtract(e).content.concat(e.content)) : this;
	},
	subtract: function(e) {
		var t = this;
		e = M.from(e);
		for (var n = 0; n < e.content.length; n += 2) t = t.remove(e.content[n]);
		return t;
	},
	toObject: function() {
		var e = {};
		return this.forEach(function(t, n) {
			e[t] = n;
		}), e;
	},
	get size() {
		return this.content.length >> 1;
	}
}, M.from = function(e) {
	if (e instanceof M) return e;
	var t = [];
	if (e) for (var n in e) t.push(n, e[n]);
	return new M(t);
};
//#endregion
//#region node_modules/.pnpm/prosemirror-model@1.25.12/node_modules/prosemirror-model/dist/index.js
function mo(e, t, n) {
	for (let r = 0;; r++) {
		if (r == e.childCount || r == t.childCount) return e.childCount == t.childCount ? null : n;
		let i = e.child(r), a = t.child(r);
		if (i == a) n += i.nodeSize;
		else {
			if (!i.sameMarkup(a)) return n;
			if (i.isText && i.text != a.text) {
				let e = i.text, t = a.text, r = 0;
				for (; e[r] == t[r]; r++) n++;
				return r && r < e.length && r < t.length && _o(e.charCodeAt(r - 1)) && go(e.charCodeAt(r)) && n--, n;
			}
			if (i.content.size || a.content.size) {
				let e = mo(i.content, a.content, n + 1);
				if (e != null) return e;
			}
			n += i.nodeSize;
		}
	}
}
function ho(e, t, n, r) {
	for (let i = e.childCount, a = t.childCount;;) {
		if (i == 0 || a == 0) return i == a ? null : {
			a: n,
			b: r
		};
		let o = e.child(--i), s = t.child(--a), c = o.nodeSize;
		if (o == s) n -= c, r -= c;
		else {
			if (!o.sameMarkup(s)) return {
				a: n,
				b: r
			};
			if (o.isText && o.text != s.text) {
				let e = o.text, t = s.text, i = e.length, a = t.length;
				for (; i > 0 && a > 0 && e[i - 1] == t[a - 1];) i--, a--, n--, r--;
				return i && a && i < e.length && _o(e.charCodeAt(i - 1)) && go(e.charCodeAt(i)) && (n++, r++), {
					a: n,
					b: r
				};
			}
			if (o.content.size || s.content.size) {
				let e = ho(o.content, s.content, n - 1, r - 1);
				if (e) return e;
			}
			n -= c, r -= c;
		}
	}
}
function go(e) {
	return e >= 56320 && e < 57344;
}
function _o(e) {
	return e >= 55296 && e < 56320;
}
var N = class e {
	constructor(e, t) {
		if (this.content = e, this.size = t || 0, t == null) for (let t = 0; t < e.length; t++) this.size += e[t].nodeSize;
	}
	nodesBetween(e, t, n, r = 0, i) {
		for (let a = 0, o = 0; o < t; a++) {
			let s = this.content[a], c = o + s.nodeSize;
			if (c > e && n(s, r + o, i || null, a) !== !1 && s.content.size) {
				let i = o + 1;
				s.nodesBetween(Math.max(0, e - i), Math.min(s.content.size, t - i), n, r + i);
			}
			o = c;
		}
	}
	descendants(e) {
		this.nodesBetween(0, this.size, e);
	}
	textBetween(e, t, n, r) {
		let i = "", a = !0;
		return this.nodesBetween(e, t, (o, s) => {
			let c = o.isText ? o.text.slice(Math.max(e, s) - s, t - s) : o.isLeaf ? r ? typeof r == "function" ? r(o) : r : o.type.spec.leafText ? o.type.spec.leafText(o) : "" : "";
			o.isBlock && (o.isLeaf && c || o.isTextblock) && n && (a ? a = !1 : i += n), i += c;
		}, 0), i;
	}
	append(t) {
		if (!t.size) return this;
		if (!this.size) return t;
		let n = this.lastChild, r = t.firstChild, i = this.content.slice(), a = 0;
		for (n.isText && n.sameMarkup(r) && (i[i.length - 1] = n.withText(n.text + r.text), a = 1); a < t.content.length; a++) i.push(t.content[a]);
		return new e(i, this.size + t.size);
	}
	cut(t, n = this.size) {
		if (t == 0 && n == this.size) return this;
		let r = [], i = 0;
		if (n > t) for (let e = 0, a = 0; a < n; e++) {
			let o = this.content[e], s = a + o.nodeSize;
			s > t && ((a < t || s > n) && (o = o.isText ? o.cut(Math.max(0, t - a), Math.min(o.text.length, n - a)) : o.cut(Math.max(0, t - a - 1), Math.min(o.content.size, n - a - 1))), r.push(o), i += o.nodeSize), a = s;
		}
		return new e(r, i);
	}
	cutByIndex(t, n) {
		return t == n ? e.empty : t == 0 && n == this.content.length ? this : new e(this.content.slice(t, n));
	}
	replaceChild(t, n) {
		let r = this.content[t];
		if (r == n) return this;
		let i = this.content.slice(), a = this.size + n.nodeSize - r.nodeSize;
		return i[t] = n, new e(i, a);
	}
	addToStart(t) {
		return new e([t].concat(this.content), this.size + t.nodeSize);
	}
	addToEnd(t) {
		return new e(this.content.concat(t), this.size + t.nodeSize);
	}
	eq(e) {
		if (this.content.length != e.content.length) return !1;
		for (let t = 0; t < this.content.length; t++) if (!this.content[t].eq(e.content[t])) return !1;
		return !0;
	}
	get firstChild() {
		return this.content.length ? this.content[0] : null;
	}
	get lastChild() {
		return this.content.length ? this.content[this.content.length - 1] : null;
	}
	get childCount() {
		return this.content.length;
	}
	child(e) {
		let t = this.content[e];
		if (!t) throw RangeError("Index " + e + " out of range for " + this);
		return t;
	}
	maybeChild(e) {
		return this.content[e] || null;
	}
	forEach(e) {
		for (let t = 0, n = 0; t < this.content.length; t++) {
			let r = this.content[t];
			e(r, n, t), n += r.nodeSize;
		}
	}
	findDiffStart(e, t = 0) {
		return mo(this, e, t);
	}
	findDiffEnd(e, t = this.size, n = e.size) {
		return ho(this, e, t, n);
	}
	findIndex(e) {
		if (e == 0) return yo(0, e);
		if (e == this.size) return yo(this.content.length, e);
		if (e > this.size || e < 0) throw RangeError(`Position ${e} outside of fragment (${this})`);
		for (let t = 0, n = 0;; t++) {
			let r = this.child(t), i = n + r.nodeSize;
			if (i >= e) return i == e ? yo(t + 1, i) : yo(t, n);
			n = i;
		}
	}
	toString() {
		return "<" + this.toStringInner() + ">";
	}
	toStringInner() {
		return this.content.join(", ");
	}
	toJSON() {
		return this.content.length ? this.content.map((e) => e.toJSON()) : null;
	}
	static fromJSON(t, n) {
		if (!n) return e.empty;
		if (!Array.isArray(n)) throw RangeError("Invalid input for Fragment.fromJSON");
		return e.fromArray(n.map(t.nodeFromJSON));
	}
	static fromArray(t) {
		if (!t.length) return e.empty;
		let n, r = 0;
		for (let e = 0; e < t.length; e++) {
			let i = t[e];
			r += i.nodeSize, e && i.isText && t[e - 1].sameMarkup(i) ? (n ||= t.slice(0, e), n[n.length - 1] = i.withText(n[n.length - 1].text + i.text)) : n && n.push(i);
		}
		return new e(n || t, r);
	}
	static from(t) {
		if (!t) return e.empty;
		if (t instanceof e) return t;
		if (Array.isArray(t)) return this.fromArray(t);
		if (t.attrs) return new e([t], t.nodeSize);
		throw RangeError("Can not convert " + t + " to a Fragment" + (t.nodesBetween ? " (looks like multiple versions of prosemirror-model were loaded)" : ""));
	}
};
N.empty = new N([], 0);
var vo = {
	index: 0,
	offset: 0
};
function yo(e, t) {
	return vo.index = e, vo.offset = t, vo;
}
function bo(e, t) {
	if (e === t) return !0;
	if (!(e && typeof e == "object") || !(t && typeof t == "object")) return !1;
	let n = Array.isArray(e);
	if (Array.isArray(t) != n) return !1;
	if (n) {
		if (e.length != t.length) return !1;
		for (let n = 0; n < e.length; n++) if (!bo(e[n], t[n])) return !1;
	} else {
		for (let n in e) if (!(n in t) || !bo(e[n], t[n])) return !1;
		for (let n in t) if (!(n in e)) return !1;
	}
	return !0;
}
var P = class e {
	constructor(e, t) {
		this.type = e, this.attrs = t;
	}
	addToSet(e) {
		let t, n = !1;
		for (let r = 0; r < e.length; r++) {
			let i = e[r];
			if (this.eq(i)) return e;
			if (this.type.excludes(i.type)) t ||= e.slice(0, r);
			else if (i.type.excludes(this.type)) return e;
			else !n && i.type.rank > this.type.rank && (t ||= e.slice(0, r), t.push(this), n = !0), t && t.push(i);
		}
		return t ||= e.slice(), n || t.push(this), t;
	}
	removeFromSet(e) {
		for (let t = 0; t < e.length; t++) if (this.eq(e[t])) return e.slice(0, t).concat(e.slice(t + 1));
		return e;
	}
	isInSet(e) {
		for (let t = 0; t < e.length; t++) if (this.eq(e[t])) return !0;
		return !1;
	}
	eq(e) {
		return this == e || this.type == e.type && bo(this.attrs, e.attrs);
	}
	toJSON() {
		let e = { type: this.type.name };
		for (let t in this.attrs) {
			e.attrs = this.attrs;
			break;
		}
		return e;
	}
	static fromJSON(e, t) {
		if (!t) throw RangeError("Invalid input for Mark.fromJSON");
		let n = e.marks[t.type];
		if (!n) throw RangeError(`There is no mark type ${t.type} in this schema`);
		return n.create(t.attrs);
	}
	static sameSet(e, t) {
		if (e == t) return !0;
		if (e.length != t.length) return !1;
		for (let n = 0; n < e.length; n++) if (!e[n].eq(t[n])) return !1;
		return !0;
	}
	static setFrom(t) {
		if (!t || Array.isArray(t) && t.length == 0) return e.none;
		if (t instanceof e) return [t];
		let n = t.slice();
		return n.sort((e, t) => e.type.rank - t.type.rank), n;
	}
};
P.none = [];
var xo = class extends Error {}, F = class e {
	constructor(e, t, n) {
		this.content = e, this.openStart = t, this.openEnd = n;
	}
	get size() {
		return this.content.size - this.openStart - this.openEnd;
	}
	insertAt(t, n) {
		let r = Co(this.content, t + this.openStart, n, this.openStart + 1, this.openEnd + 1);
		return r && new e(r, this.openStart, this.openEnd);
	}
	removeBetween(t, n) {
		return new e(So(this.content, t + this.openStart, n + this.openStart), this.openStart, this.openEnd);
	}
	eq(e) {
		return this.content.eq(e.content) && this.openStart == e.openStart && this.openEnd == e.openEnd;
	}
	toString() {
		return this.content + "(" + this.openStart + "," + this.openEnd + ")";
	}
	toJSON() {
		if (!this.content.size) return null;
		let e = { content: this.content.toJSON() };
		return this.openStart > 0 && (e.openStart = this.openStart), this.openEnd > 0 && (e.openEnd = this.openEnd), e;
	}
	static fromJSON(t, n) {
		if (!n) return e.empty;
		let r = n.openStart || 0, i = n.openEnd || 0;
		if (typeof r != "number" || typeof i != "number") throw RangeError("Invalid input for Slice.fromJSON");
		return new e(N.fromJSON(t, n.content), r, i);
	}
	static maxOpen(t, n = !0) {
		let r = 0, i = 0;
		for (let e = t.firstChild; e && !e.isLeaf && (n || !e.type.spec.isolating); e = e.firstChild) r++;
		for (let e = t.lastChild; e && !e.isLeaf && (n || !e.type.spec.isolating); e = e.lastChild) i++;
		return new e(t, r, i);
	}
};
F.empty = new F(N.empty, 0, 0);
function So(e, t, n) {
	let { index: r, offset: i } = e.findIndex(t), a = e.maybeChild(r), { index: o, offset: s } = e.findIndex(n);
	if (i == t || a.isText) {
		if (s != n && !e.child(o).isText) throw RangeError("Removing non-flat range");
		return e.cut(0, t).append(e.cut(n));
	}
	if (r != o) throw RangeError("Removing non-flat range");
	return e.replaceChild(r, a.copy(So(a.content, t - i - 1, n - i - 1)));
}
function Co(e, t, n, r, i, a) {
	let { index: o, offset: s } = e.findIndex(t), c = e.maybeChild(o);
	if (s == t || c.isText) return a && r <= 0 && i <= 0 && !a.canReplace(o, o, n) ? null : e.cut(0, t).append(n).append(e.cut(t));
	let l = Co(c.content, t - s - 1, n, o == 0 ? r - 1 : 0, o == e.childCount - 1 ? i - 1 : 0, c);
	return l && e.replaceChild(o, c.copy(l));
}
function wo(e, t, n) {
	if (n.openStart > e.depth) throw new xo("Inserted content deeper than insertion position");
	if (e.depth - n.openStart != t.depth - n.openEnd) throw new xo("Inconsistent open depths");
	return To(e, t, n, 0);
}
function To(e, t, n, r) {
	let i = e.index(r), a = e.node(r);
	if (i == t.index(r) && r < e.depth - n.openStart) {
		let o = To(e, t, n, r + 1);
		return a.copy(a.content.replaceChild(i, o));
	}
	if (!n.content.size) return Ao(a, Mo(e, t, r));
	if (!n.openStart && !n.openEnd && e.depth == r && t.depth == r) {
		let r = e.parent, i = r.content;
		return Ao(r, i.cut(0, e.parentOffset).append(n.content).append(i.cut(t.parentOffset)));
	}
	{
		let { start: i, end: o } = No(n, e);
		return Ao(a, jo(e, i, o, t, r));
	}
}
function Eo(e, t) {
	if (!t.type.compatibleContent(e.type)) throw new xo("Cannot join " + t.type.name + " onto " + e.type.name);
}
function Do(e, t, n) {
	let r = e.node(n);
	return Eo(r, t.node(n)), r;
}
function Oo(e, t) {
	let n = t.length - 1;
	n >= 0 && e.isText && e.sameMarkup(t[n]) ? t[n] = e.withText(t[n].text + e.text) : t.push(e);
}
function ko(e, t, n, r) {
	let i = (t || e).node(n), a = 0, o = t ? t.index(n) : i.childCount;
	e && (a = e.index(n), e.depth > n ? a++ : e.textOffset && (Oo(e.nodeAfter, r), a++));
	for (let e = a; e < o; e++) Oo(i.child(e), r);
	t && t.depth == n && t.textOffset && Oo(t.nodeBefore, r);
}
function Ao(e, t) {
	if (!e.type.validContent(t)) throw new xo("Invalid content for node " + e.type.name);
	return e.copy(t);
}
function jo(e, t, n, r, i) {
	let a = e.depth > i && Do(e, t, i + 1), o = r.depth > i && Do(n, r, i + 1), s = [];
	return ko(null, e, i, s), a && o && t.index(i) == n.index(i) ? (Eo(a, o), Oo(Ao(a, jo(e, t, n, r, i + 1)), s)) : (a && Oo(Ao(a, Mo(e, t, i + 1)), s), ko(t, n, i, s), o && Oo(Ao(o, Mo(n, r, i + 1)), s)), ko(r, null, i, s), new N(s);
}
function Mo(e, t, n) {
	let r = [];
	return ko(null, e, n, r), e.depth > n && Oo(Ao(Do(e, t, n + 1), Mo(e, t, n + 1)), r), ko(t, null, n, r), new N(r);
}
function No(e, t) {
	let n = t.depth - e.openStart, r = t.node(n).copy(e.content);
	for (let e = n - 1; e >= 0; e--) r = t.node(e).copy(N.from(r));
	return {
		start: r.resolveNoCache(e.openStart + n),
		end: r.resolveNoCache(r.content.size - e.openEnd - n)
	};
}
var Po = class e {
	constructor(e, t, n) {
		this.pos = e, this.path = t, this.parentOffset = n, this.depth = t.length / 3 - 1;
	}
	resolveDepth(e) {
		return e == null ? this.depth : e < 0 ? this.depth + e : e;
	}
	get parent() {
		return this.node(this.depth);
	}
	get doc() {
		return this.node(0);
	}
	node(e) {
		return this.path[this.resolveDepth(e) * 3];
	}
	index(e) {
		return this.path[this.resolveDepth(e) * 3 + 1];
	}
	indexAfter(e) {
		return e = this.resolveDepth(e), this.index(e) + (e == this.depth && !this.textOffset ? 0 : 1);
	}
	start(e) {
		return e = this.resolveDepth(e), e == 0 ? 0 : this.path[e * 3 - 1] + 1;
	}
	end(e) {
		return e = this.resolveDepth(e), this.start(e) + this.node(e).content.size;
	}
	before(e) {
		if (e = this.resolveDepth(e), !e) throw RangeError("There is no position before the top-level node");
		return e == this.depth + 1 ? this.pos : this.path[e * 3 - 1];
	}
	after(e) {
		if (e = this.resolveDepth(e), !e) throw RangeError("There is no position after the top-level node");
		return e == this.depth + 1 ? this.pos : this.path[e * 3 - 1] + this.path[e * 3].nodeSize;
	}
	get textOffset() {
		return this.pos - this.path[this.path.length - 1];
	}
	get nodeAfter() {
		let e = this.parent, t = this.index(this.depth);
		if (t == e.childCount) return null;
		let n = this.pos - this.path[this.path.length - 1], r = e.child(t);
		return n ? e.child(t).cut(n) : r;
	}
	get nodeBefore() {
		let e = this.index(this.depth), t = this.pos - this.path[this.path.length - 1];
		return t ? this.parent.child(e).cut(0, t) : e == 0 ? null : this.parent.child(e - 1);
	}
	posAtIndex(e, t) {
		t = this.resolveDepth(t);
		let n = this.path[t * 3], r = t == 0 ? 0 : this.path[t * 3 - 1] + 1;
		for (let t = 0; t < e; t++) r += n.child(t).nodeSize;
		return r;
	}
	marks() {
		let e = this.parent, t = this.index();
		if (e.content.size == 0) return P.none;
		if (this.textOffset) return e.child(t).marks;
		let n = e.maybeChild(t - 1), r = e.maybeChild(t);
		if (!n) {
			let e = n;
			n = r, r = e;
		}
		let i = n.marks;
		for (var a = 0; a < i.length; a++) i[a].type.spec.inclusive === !1 && (!r || !i[a].isInSet(r.marks)) && (i = i[a--].removeFromSet(i));
		return i;
	}
	marksAcross(e) {
		let t = this.parent.maybeChild(this.index());
		if (!t || !t.isInline) return null;
		let n = t.marks, r = e.parent.maybeChild(e.index());
		for (var i = 0; i < n.length; i++) n[i].type.spec.inclusive === !1 && (!r || !n[i].isInSet(r.marks)) && (n = n[i--].removeFromSet(n));
		return n;
	}
	sharedDepth(e) {
		for (let t = this.depth; t > 0; t--) if (this.start(t) <= e && this.end(t) >= e) return t;
		return 0;
	}
	blockRange(e = this, t) {
		if (e.pos < this.pos) return e.blockRange(this);
		for (let n = this.depth - (this.parent.inlineContent || this.pos == e.pos ? 1 : 0); n >= 0; n--) if (e.pos <= this.end(n) && (!t || t(this.node(n)))) return new Ro(this, e, n);
		return null;
	}
	sameParent(e) {
		return this.pos - this.parentOffset == e.pos - e.parentOffset;
	}
	max(e) {
		return e.pos > this.pos ? e : this;
	}
	min(e) {
		return e.pos < this.pos ? e : this;
	}
	toString() {
		let e = "";
		for (let t = 1; t <= this.depth; t++) e += (e ? "/" : "") + this.node(t).type.name + "_" + this.index(t - 1);
		return e + ":" + this.parentOffset;
	}
	static resolve(t, n) {
		if (!(n >= 0 && n <= t.content.size)) throw RangeError("Position " + n + " out of range");
		let r = [], i = 0, a = n;
		for (let e = t;;) {
			let { index: t, offset: n } = e.content.findIndex(a), o = a - n;
			if (r.push(e, t, i + n), !o || (e = e.child(t), e.isText)) break;
			a = o - 1, i += n + 1;
		}
		return new e(n, r, a);
	}
	static resolveCached(t, n) {
		let r = Lo.get(t);
		if (r) for (let e = 0; e < r.elts.length; e++) {
			let t = r.elts[e];
			if (t.pos == n) return t;
		}
		else Lo.set(t, r = new Fo());
		let i = r.elts[r.i] = e.resolve(t, n);
		return r.i = (r.i + 1) % Io, i;
	}
}, Fo = class {
	constructor() {
		this.elts = [], this.i = 0;
	}
}, Io = 12, Lo = /* @__PURE__ */ new WeakMap(), Ro = class {
	constructor(e, t, n) {
		this.$from = e, this.$to = t, this.depth = n;
	}
	get start() {
		return this.$from.before(this.depth + 1);
	}
	get end() {
		return this.$to.after(this.depth + 1);
	}
	get parent() {
		return this.$from.node(this.depth);
	}
	get startIndex() {
		return this.$from.index(this.depth);
	}
	get endIndex() {
		return this.$to.indexAfter(this.depth);
	}
}, zo = Object.create(null), Bo = class e {
	constructor(e, t, n, r = P.none) {
		this.type = e, this.attrs = t, this.marks = r, this.content = n || N.empty;
	}
	get children() {
		return this.content.content;
	}
	get nodeSize() {
		return this.isLeaf ? 1 : 2 + this.content.size;
	}
	get childCount() {
		return this.content.childCount;
	}
	child(e) {
		return this.content.child(e);
	}
	maybeChild(e) {
		return this.content.maybeChild(e);
	}
	forEach(e) {
		this.content.forEach(e);
	}
	nodesBetween(e, t, n, r = 0) {
		this.content.nodesBetween(e, t, n, r, this);
	}
	descendants(e) {
		this.nodesBetween(0, this.content.size, e);
	}
	get textContent() {
		return this.isLeaf && this.type.spec.leafText ? this.type.spec.leafText(this) : this.textBetween(0, this.content.size, "");
	}
	textBetween(e, t, n, r) {
		return this.content.textBetween(e, t, n, r);
	}
	get firstChild() {
		return this.content.firstChild;
	}
	get lastChild() {
		return this.content.lastChild;
	}
	eq(e) {
		return this == e || this.sameMarkup(e) && this.content.eq(e.content);
	}
	sameMarkup(e) {
		return this.hasMarkup(e.type, e.attrs, e.marks);
	}
	hasMarkup(e, t, n) {
		return this.type == e && bo(this.attrs, t || e.defaultAttrs || zo) && P.sameSet(this.marks, n || P.none);
	}
	copy(t = null) {
		return t == this.content ? this : new e(this.type, this.attrs, t, this.marks);
	}
	mark(t) {
		return t == this.marks ? this : new e(this.type, this.attrs, this.content, t);
	}
	cut(e, t = this.content.size) {
		return e == 0 && t == this.content.size ? this : this.copy(this.content.cut(e, t));
	}
	slice(e, t = this.content.size, n = !1) {
		if (e == t) return F.empty;
		let r = this.resolve(e), i = this.resolve(t), a = n ? 0 : r.sharedDepth(t), o = r.start(a);
		return new F(r.node(a).content.cut(r.pos - o, i.pos - o), r.depth - a, i.depth - a);
	}
	replace(e, t, n) {
		return wo(this.resolve(e), this.resolve(t), n);
	}
	nodeAt(e) {
		for (let t = this;;) {
			let { index: n, offset: r } = t.content.findIndex(e);
			if (t = t.maybeChild(n), !t) return null;
			if (r == e || t.isText) return t;
			e -= r + 1;
		}
	}
	childAfter(e) {
		let { index: t, offset: n } = this.content.findIndex(e);
		return {
			node: this.content.maybeChild(t),
			index: t,
			offset: n
		};
	}
	childBefore(e) {
		if (e == 0) return {
			node: null,
			index: 0,
			offset: 0
		};
		let { index: t, offset: n } = this.content.findIndex(e);
		if (n < e) return {
			node: this.content.child(t),
			index: t,
			offset: n
		};
		let r = this.content.child(t - 1);
		return {
			node: r,
			index: t - 1,
			offset: n - r.nodeSize
		};
	}
	resolve(e) {
		return Po.resolveCached(this, e);
	}
	resolveNoCache(e) {
		return Po.resolve(this, e);
	}
	rangeHasMark(e, t, n) {
		let r = !1;
		return t > e && this.nodesBetween(e, t, (e) => (n.isInSet(e.marks) && (r = !0), !r)), r;
	}
	get isBlock() {
		return this.type.isBlock;
	}
	get isTextblock() {
		return this.type.isTextblock;
	}
	get inlineContent() {
		return this.type.inlineContent;
	}
	get isInline() {
		return this.type.isInline;
	}
	get isText() {
		return this.type.isText;
	}
	get isLeaf() {
		return this.type.isLeaf;
	}
	get isAtom() {
		return this.type.isAtom;
	}
	toString() {
		if (this.type.spec.toDebugString) return this.type.spec.toDebugString(this);
		let e = this.type.name;
		return this.content.size && (e += "(" + this.content.toStringInner() + ")"), Ho(this.marks, e);
	}
	contentMatchAt(e) {
		let t = this.type.contentMatch.matchFragment(this.content, 0, e);
		if (!t) throw Error("Called contentMatchAt on a node with invalid content");
		return t;
	}
	canReplace(e, t, n = N.empty, r = 0, i = n.childCount) {
		let a = this.contentMatchAt(e).matchFragment(n, r, i), o = a && a.matchFragment(this.content, t);
		if (!o || !o.validEnd) return !1;
		for (let e = r; e < i; e++) if (!this.type.allowsMarks(n.child(e).marks)) return !1;
		return !0;
	}
	canReplaceWith(e, t, n, r) {
		if (r && !this.type.allowsMarks(r)) return !1;
		let i = this.contentMatchAt(e).matchType(n), a = i && i.matchFragment(this.content, t);
		return a ? a.validEnd : !1;
	}
	canAppend(e) {
		return e.content.size ? this.canReplace(this.childCount, this.childCount, e.content) : this.type.compatibleContent(e.type);
	}
	check() {
		this.type.checkContent(this.content), Uo(this.type.attrs, this.attrs, "node", this.type.name);
		let e = P.none;
		for (let t = 0; t < this.marks.length; t++) {
			let n = this.marks[t];
			Uo(n.type.attrs, n.attrs, "mark", n.type.name), e = n.addToSet(e);
		}
		if (!P.sameSet(e, this.marks)) throw RangeError(`Invalid collection of marks for node ${this.type.name}: ${this.marks.map((e) => e.type.name)}`);
		this.content.forEach((e) => e.check());
	}
	toJSON() {
		let e = { type: this.type.name };
		for (let t in this.attrs) {
			e.attrs = this.attrs;
			break;
		}
		return this.content.size && (e.content = this.content.toJSON()), this.marks.length && (e.marks = this.marks.map((e) => e.toJSON())), e;
	}
	static fromJSON(e, t) {
		if (!t) throw RangeError("Invalid input for Node.fromJSON");
		let n;
		if (t.marks) {
			if (!Array.isArray(t.marks)) throw RangeError("Invalid mark data for Node.fromJSON");
			n = t.marks.map(e.markFromJSON);
		}
		if (t.type == "text") {
			if (typeof t.text != "string") throw RangeError("Invalid text node in JSON");
			return e.text(t.text, n);
		}
		let r = N.fromJSON(e, t.content);
		return e.nodeType(t.type).create(t.attrs, r, n);
	}
};
Bo.prototype.text = void 0;
var Vo = class e extends Bo {
	constructor(e, t, n, r) {
		if (super(e, t, null, r), !n) throw RangeError("Empty text nodes are not allowed");
		this.text = n;
	}
	toString() {
		return this.type.spec.toDebugString ? this.type.spec.toDebugString(this) : Ho(this.marks, JSON.stringify(this.text));
	}
	get textContent() {
		return this.text;
	}
	textBetween(e, t) {
		return this.text.slice(e, t);
	}
	get nodeSize() {
		return this.text.length;
	}
	mark(t) {
		return t == this.marks ? this : new e(this.type, this.attrs, this.text, t);
	}
	withText(t) {
		return t == this.text ? this : new e(this.type, this.attrs, t, this.marks);
	}
	cut(e = 0, t = this.text.length) {
		return e == 0 && t == this.text.length ? this : this.withText(this.text.slice(e, t));
	}
	eq(e) {
		return this.sameMarkup(e) && this.text == e.text;
	}
	toJSON() {
		let e = super.toJSON();
		return e.text = this.text, e;
	}
};
function Ho(e, t) {
	for (let n = e.length - 1; n >= 0; n--) t = e[n].type.name + "(" + t + ")";
	return t;
}
function Uo(e, t, n, r) {
	for (let i in t) if (!(i in e)) throw RangeError(`Unsupported attribute ${i} for ${n} of type ${r}`);
	for (let n in e) e[n].validate && e[n].validate(t[n]);
}
var Wo = class e {
	constructor(e) {
		this.validEnd = e, this.next = [], this.wrapCache = [];
	}
	static parse(t, n) {
		let r = new Go(t, n);
		if (r.next == null) return e.empty;
		let i = Ko(r);
		r.next && r.err("Unexpected trailing text");
		let a = ns($o(i));
		return rs(a, r), a;
	}
	matchType(e) {
		for (let t = 0; t < this.next.length; t++) if (this.next[t].type == e) return this.next[t].next;
		return null;
	}
	matchFragment(e, t = 0, n = e.childCount) {
		let r = this;
		for (let i = t; r && i < n; i++) r = r.matchType(e.child(i).type);
		return r;
	}
	get inlineContent() {
		return this.next.length != 0 && this.next[0].type.isInline;
	}
	get defaultType() {
		for (let e = 0; e < this.next.length; e++) {
			let { type: t } = this.next[e];
			if (!(t.isText || t.hasRequiredAttrs())) return t;
		}
		return null;
	}
	compatible(e) {
		for (let t = 0; t < this.next.length; t++) for (let n = 0; n < e.next.length; n++) if (this.next[t].type == e.next[n].type) return !0;
		return !1;
	}
	fillBefore(e, t = !1, n = 0) {
		let r = [this];
		function i(a, o) {
			let s = a.matchFragment(e, n);
			if (s && (!t || s.validEnd)) return N.from(o.map((e) => e.createAndFill()));
			for (let e = 0; e < a.next.length; e++) {
				let { type: t, next: n } = a.next[e];
				if (!(t.isText || t.hasRequiredAttrs()) && r.indexOf(n) == -1) {
					r.push(n);
					let e = i(n, o.concat(t));
					if (e) return e;
				}
			}
			return null;
		}
		return i(this, []);
	}
	findWrapping(e) {
		for (let t = 0; t < this.wrapCache.length; t += 2) if (this.wrapCache[t] == e) return this.wrapCache[t + 1];
		let t = this.computeWrapping(e);
		return this.wrapCache.push(e, t), t;
	}
	computeWrapping(e) {
		let t = Object.create(null), n = [{
			match: this,
			type: null,
			via: null
		}];
		for (; n.length;) {
			let r = n.shift(), i = r.match;
			if (i.matchType(e)) {
				let e = [];
				for (let t = r; t.type; t = t.via) e.push(t.type);
				return e.reverse();
			}
			for (let e = 0; e < i.next.length; e++) {
				let { type: a, next: o } = i.next[e];
				!a.isLeaf && !a.hasRequiredAttrs() && !(a.name in t) && (!r.type || o.validEnd) && (n.push({
					match: a.contentMatch,
					type: a,
					via: r
				}), t[a.name] = !0);
			}
		}
		return null;
	}
	get edgeCount() {
		return this.next.length;
	}
	edge(e) {
		if (e >= this.next.length) throw RangeError(`There's no ${e}th edge in this content match`);
		return this.next[e];
	}
	toString() {
		let e = [];
		function t(n) {
			e.push(n);
			for (let r = 0; r < n.next.length; r++) e.indexOf(n.next[r].next) == -1 && t(n.next[r].next);
		}
		return t(this), e.map((t, n) => {
			let r = n + (t.validEnd ? "*" : " ") + " ";
			for (let n = 0; n < t.next.length; n++) r += (n ? ", " : "") + t.next[n].type.name + "->" + e.indexOf(t.next[n].next);
			return r;
		}).join("\n");
	}
};
Wo.empty = new Wo(!0);
var Go = class {
	constructor(e, t) {
		this.string = e, this.nodeTypes = t, this.inline = null, this.pos = 0, this.tokens = e.split(/\s*(?=\b|\W|$)/), this.tokens[this.tokens.length - 1] == "" && this.tokens.pop(), this.tokens[0] == "" && this.tokens.shift();
	}
	get next() {
		return this.tokens[this.pos];
	}
	eat(e) {
		return this.next == e && (this.pos++ || !0);
	}
	err(e) {
		throw SyntaxError(e + " (in content expression '" + this.string + "')");
	}
};
function Ko(e) {
	let t = [];
	do
		t.push(qo(e));
	while (e.eat("|"));
	return t.length == 1 ? t[0] : {
		type: "choice",
		exprs: t
	};
}
function qo(e) {
	let t = [];
	do
		t.push(Jo(e));
	while (e.next && e.next != ")" && e.next != "|");
	return t.length == 1 ? t[0] : {
		type: "seq",
		exprs: t
	};
}
function Jo(e) {
	let t = Qo(e);
	for (;;) if (e.eat("+")) t = {
		type: "plus",
		expr: t
	};
	else if (e.eat("*")) t = {
		type: "star",
		expr: t
	};
	else if (e.eat("?")) t = {
		type: "opt",
		expr: t
	};
	else if (e.eat("{")) t = Xo(e, t);
	else break;
	return t;
}
function Yo(e) {
	/\D/.test(e.next) && e.err("Expected number, got '" + e.next + "'");
	let t = Number(e.next);
	return e.pos++, t;
}
function Xo(e, t) {
	let n = Yo(e), r = n;
	return e.eat(",") && (r = e.next == "}" ? -1 : Yo(e)), e.eat("}") || e.err("Unclosed braced range"), {
		type: "range",
		min: n,
		max: r,
		expr: t
	};
}
function Zo(e, t) {
	let n = e.nodeTypes, r = n[t];
	if (r) return [r];
	let i = [];
	for (let e in n) {
		let r = n[e];
		r.isInGroup(t) && i.push(r);
	}
	return i.length == 0 && e.err("No node type or group '" + t + "' found"), i;
}
function Qo(e) {
	if (e.eat("(")) {
		let t = Ko(e);
		return e.eat(")") || e.err("Missing closing paren"), t;
	}
	if (/\W/.test(e.next)) e.err("Unexpected token '" + e.next + "'");
	else {
		let t = Zo(e, e.next).map((t) => (e.inline == null ? e.inline = t.isInline : e.inline != t.isInline && e.err("Mixing inline and block content"), {
			type: "name",
			value: t
		}));
		return e.pos++, t.length == 1 ? t[0] : {
			type: "choice",
			exprs: t
		};
	}
}
function $o(e) {
	let t = [[]];
	return i(a(e, 0), n()), t;
	function n() {
		return t.push([]) - 1;
	}
	function r(e, n, r) {
		let i = {
			term: r,
			to: n
		};
		return t[e].push(i), i;
	}
	function i(e, t) {
		e.forEach((e) => e.to = t);
	}
	function a(e, t) {
		if (e.type == "choice") return e.exprs.reduce((e, n) => e.concat(a(n, t)), []);
		if (e.type == "seq") for (let r = 0;; r++) {
			let o = a(e.exprs[r], t);
			if (r == e.exprs.length - 1) return o;
			i(o, t = n());
		}
		else if (e.type == "star") {
			let o = n();
			return r(t, o), i(a(e.expr, o), o), [r(o)];
		} else if (e.type == "plus") {
			let o = n();
			return i(a(e.expr, t), o), i(a(e.expr, o), o), [r(o)];
		} else if (e.type == "opt") return [r(t)].concat(a(e.expr, t));
		else if (e.type == "range") {
			let o = t;
			for (let t = 0; t < e.min; t++) {
				let t = n();
				i(a(e.expr, o), t), o = t;
			}
			if (e.max == -1) i(a(e.expr, o), o);
			else for (let t = e.min; t < e.max; t++) {
				let t = n();
				r(o, t), i(a(e.expr, o), t), o = t;
			}
			return [r(o)];
		} else if (e.type == "name") return [r(t, void 0, e.value)];
		else throw Error("Unknown expr type");
	}
}
function es(e, t) {
	return t - e;
}
function ts(e, t) {
	let n = [];
	return r(t), n.sort(es);
	function r(t) {
		let i = e[t];
		if (i.length == 1 && !i[0].term) return r(i[0].to);
		n.push(t);
		for (let e = 0; e < i.length; e++) {
			let { term: t, to: a } = i[e];
			!t && n.indexOf(a) == -1 && r(a);
		}
	}
}
function ns(e) {
	let t = Object.create(null);
	return n(ts(e, 0));
	function n(r) {
		let i = [];
		r.forEach((t) => {
			e[t].forEach(({ term: t, to: n }) => {
				if (!t) return;
				let r;
				for (let e = 0; e < i.length; e++) i[e][0] == t && (r = i[e][1]);
				ts(e, n).forEach((e) => {
					r || i.push([t, r = []]), r.indexOf(e) == -1 && r.push(e);
				});
			});
		});
		let a = t[r.join(",")] = new Wo(r.indexOf(e.length - 1) > -1);
		for (let e = 0; e < i.length; e++) {
			let r = i[e][1].sort(es);
			a.next.push({
				type: i[e][0],
				next: t[r.join(",")] || n(r)
			});
		}
		return a;
	}
}
function rs(e, t) {
	for (let n = 0, r = [e]; n < r.length; n++) {
		let e = r[n], i = !e.validEnd, a = [];
		for (let t = 0; t < e.next.length; t++) {
			let { type: n, next: o } = e.next[t];
			a.push(n.name), i && !(n.isText || n.hasRequiredAttrs()) && (i = !1), r.indexOf(o) == -1 && r.push(o);
		}
		i && t.err("Only non-generatable nodes (" + a.join(", ") + ") in a required position (see https://prosemirror.net/docs/guide/#generatable)");
	}
}
function is(e) {
	let t = Object.create(null);
	for (let n in e) {
		let r = e[n];
		if (!r.hasDefault) return null;
		t[n] = r.default;
	}
	return t;
}
function as(e, t) {
	let n = Object.create(null);
	for (let r in e) {
		let i = e[r], a = t && t[r];
		if (a === void 0) {
			if (i.hasDefault) a = i.default;
			else throw RangeError("No value supplied for attribute " + r);
		} else i.validate && i.validate(a);
		n[r] = a;
	}
	return n;
}
function ss(e, t) {
	let n = Object.create(null);
	if (t) for (let r in t) n[r] = new us(e, r, t[r]);
	return n;
}
var cs = class e {
	constructor(e, t, n) {
		this.name = e, this.schema = t, this.spec = n, this.markSet = null, this.groups = n.group ? n.group.split(" ") : [], this.attrs = ss(e, n.attrs), this.defaultAttrs = is(this.attrs), this.contentMatch = null, this.inlineContent = null, this.isBlock = !(n.inline || e == "text"), this.isText = e == "text";
	}
	get isInline() {
		return !this.isBlock;
	}
	get isTextblock() {
		return this.isBlock && this.inlineContent;
	}
	get isLeaf() {
		return this.contentMatch == Wo.empty;
	}
	get isAtom() {
		return this.isLeaf || !!this.spec.atom;
	}
	isInGroup(e) {
		return this.groups.indexOf(e) > -1;
	}
	get whitespace() {
		return this.spec.whitespace || (this.spec.code ? "pre" : "normal");
	}
	hasRequiredAttrs() {
		for (let e in this.attrs) if (this.attrs[e].isRequired) return !0;
		return !1;
	}
	compatibleContent(e) {
		return this == e || this.contentMatch.compatible(e.contentMatch);
	}
	computeAttrs(e) {
		return !e && this.defaultAttrs ? this.defaultAttrs : as(this.attrs, e);
	}
	create(e = null, t, n) {
		if (this.isText) throw Error("NodeType.create can't construct text nodes");
		return new Bo(this, this.computeAttrs(e), N.from(t), P.setFrom(n));
	}
	createChecked(e = null, t, n) {
		return t = N.from(t), this.checkContent(t), new Bo(this, this.computeAttrs(e), t, P.setFrom(n));
	}
	createAndFill(e = null, t, n) {
		if (e = this.computeAttrs(e), t = N.from(t), t.size) {
			let e = this.contentMatch.fillBefore(t);
			if (!e) return null;
			t = e.append(t);
		}
		let r = this.contentMatch.matchFragment(t), i = r && r.fillBefore(N.empty, !0);
		return i ? new Bo(this, e, t.append(i), P.setFrom(n)) : null;
	}
	validContent(e) {
		let t = this.contentMatch.matchFragment(e);
		if (!t || !t.validEnd) return !1;
		for (let t = 0; t < e.childCount; t++) if (!this.allowsMarks(e.child(t).marks)) return !1;
		return !0;
	}
	checkContent(e) {
		if (!this.validContent(e)) throw RangeError(`Invalid content for node ${this.name}: ${e.toString().slice(0, 50)}`);
	}
	checkAttrs(e) {}
	allowsMarkType(e) {
		return this.markSet == null || this.markSet.indexOf(e) > -1;
	}
	allowsMarks(e) {
		if (this.markSet == null) return !0;
		for (let t = 0; t < e.length; t++) if (!this.allowsMarkType(e[t].type)) return !1;
		return !0;
	}
	allowedMarks(e) {
		if (this.markSet == null) return e;
		let t;
		for (let n = 0; n < e.length; n++) this.allowsMarkType(e[n].type) ? t && t.push(e[n]) : t ||= e.slice(0, n);
		return t ? t.length ? t : P.none : e;
	}
	static compile(t, n) {
		let r = Object.create(null);
		t.forEach((t, i) => r[t] = new e(t, n, i));
		let i = n.spec.topNode || "doc";
		if (!r[i]) throw RangeError("Schema is missing its top node type ('" + i + "')");
		if (!r.text) throw RangeError("Every schema needs a 'text' type");
		for (let e in r.text.attrs) throw RangeError("The text node type should not have attributes");
		return r;
	}
};
function ls(e, t, n) {
	let r = n.split("|");
	return (n) => {
		let i = n === null ? "null" : typeof n;
		if (r.indexOf(i) < 0) throw RangeError(`Expected value of type ${r} for attribute ${t} on type ${e}, got ${i}`);
	};
}
var us = class {
	constructor(e, t, n) {
		this.hasDefault = Object.prototype.hasOwnProperty.call(n, "default"), this.default = n.default, this.validate = typeof n.validate == "string" ? ls(e, t, n.validate) : n.validate;
	}
	get isRequired() {
		return !this.hasDefault;
	}
}, ds = class e {
	constructor(e, t, n, r) {
		this.name = e, this.rank = t, this.schema = n, this.spec = r, this.attrs = ss(e, r.attrs), this.excluded = null;
		let i = is(this.attrs);
		this.instance = i ? new P(this, i) : null;
	}
	create(e = null) {
		return !e && this.instance ? this.instance : new P(this, as(this.attrs, e));
	}
	static compile(t, n) {
		let r = Object.create(null), i = 0;
		return t.forEach((t, a) => r[t] = new e(t, i++, n, a)), r;
	}
	removeFromSet(e) {
		for (var t = 0; t < e.length; t++) e[t].type == this && (e = e.slice(0, t).concat(e.slice(t + 1)), t--);
		return e;
	}
	isInSet(e) {
		for (let t = 0; t < e.length; t++) if (e[t].type == this) return e[t];
	}
	excludes(e) {
		return this.excluded.indexOf(e) > -1;
	}
}, fs = class {
	constructor(e) {
		this.linebreakReplacement = null, this.cached = Object.create(null);
		let t = this.spec = {};
		for (let n in e) t[n] = e[n];
		t.nodes = M.from(e.nodes), t.marks = M.from(e.marks || {}), this.nodes = cs.compile(this.spec.nodes, this), this.marks = ds.compile(this.spec.marks, this);
		let n = Object.create(null);
		for (let e in this.nodes) {
			if (e in this.marks) throw RangeError(e + " can not be both a node and a mark");
			let t = this.nodes[e], r = t.spec.content || "", i = t.spec.marks;
			if (t.contentMatch = n[r] || (n[r] = Wo.parse(r, this.nodes)), t.inlineContent = t.contentMatch.inlineContent, t.spec.linebreakReplacement) {
				if (this.linebreakReplacement) throw RangeError("Multiple linebreak nodes defined");
				if (!t.isInline || !t.isLeaf) throw RangeError("Linebreak replacement nodes must be inline leaf nodes");
				this.linebreakReplacement = t;
			}
			t.markSet = i == "_" ? null : i ? ps(this, i.split(" ")) : i == "" || !t.inlineContent ? [] : null;
		}
		for (let e in this.marks) {
			let t = this.marks[e], n = t.spec.excludes;
			t.excluded = n == null ? [t] : n == "" ? [] : ps(this, n.split(" "));
		}
		this.nodeFromJSON = (e) => Bo.fromJSON(this, e), this.markFromJSON = (e) => P.fromJSON(this, e), this.topNodeType = this.nodes[this.spec.topNode || "doc"], this.cached.wrappings = Object.create(null);
	}
	node(e, t = null, n, r) {
		if (typeof e == "string") e = this.nodeType(e);
		else if (!(e instanceof cs)) throw RangeError("Invalid node type: " + e);
		else if (e.schema != this) throw RangeError("Node type from different schema used (" + e.name + ")");
		return e.createChecked(t, n, r);
	}
	text(e, t) {
		let n = this.nodes.text;
		return new Vo(n, n.defaultAttrs, e, P.setFrom(t));
	}
	mark(e, t) {
		return typeof e == "string" && (e = this.marks[e]), e.create(t);
	}
	nodeType(e) {
		let t = this.nodes[e];
		if (!t) throw RangeError("Unknown node type: " + e);
		return t;
	}
};
function ps(e, t) {
	let n = [];
	for (let r = 0; r < t.length; r++) {
		let i = t[r], a = e.marks[i], o = a;
		if (a) n.push(a);
		else for (let t in e.marks) {
			let r = e.marks[t];
			(i == "_" || r.spec.group && r.spec.group.split(" ").indexOf(i) > -1) && n.push(o = r);
		}
		if (!o) throw SyntaxError("Unknown mark type: '" + t[r] + "'");
	}
	return n;
}
function ms(e) {
	return e.tag != null;
}
function hs(e) {
	return e.style != null;
}
var gs = class e {
	constructor(e, t) {
		this.schema = e, this.rules = t, this.tags = [], this.styles = [];
		let n = this.matchedStyles = [];
		t.forEach((e) => {
			if (ms(e)) this.tags.push(e);
			else if (hs(e)) {
				let t = /[^=]*/.exec(e.style)[0];
				n.indexOf(t) < 0 && n.push(t), this.styles.push(e);
			}
		}), this.normalizeLists = !this.tags.some((t) => {
			if (!/^(ul|ol)\b/.test(t.tag) || !t.node) return !1;
			let n = e.nodes[t.node];
			return n.contentMatch.matchType(n);
		});
	}
	parse(e, t = {}) {
		let n = new Ts(this, t, !1);
		return n.addAll(e, P.none, t.from, t.to), n.finish();
	}
	parseSlice(e, t = {}) {
		let n = new Ts(this, t, !0);
		return n.addAll(e, P.none, t.from, t.to), F.maxOpen(n.finish());
	}
	matchTag(e, t, n) {
		for (let r = n ? this.tags.indexOf(n) + 1 : 0; r < this.tags.length; r++) {
			let n = this.tags[r];
			if (Ds(e, n.tag) && (n.namespace === void 0 || e.namespaceURI == n.namespace) && (!n.context || t.matchesContext(n.context))) {
				if (n.getAttrs) {
					let t = n.getAttrs(e);
					if (t === !1) continue;
					n.attrs = t || void 0;
				}
				return n;
			}
		}
	}
	matchStyle(e, t, n, r) {
		for (let i = r ? this.styles.indexOf(r) + 1 : 0; i < this.styles.length; i++) {
			let r = this.styles[i], a = r.style;
			if (!(a.indexOf(e) != 0 || r.context && !n.matchesContext(r.context) || a.length > e.length && (a.charCodeAt(e.length) != 61 || a.slice(e.length + 1) != t))) {
				if (r.getAttrs) {
					let e = r.getAttrs(t);
					if (e === !1) continue;
					r.attrs = e || void 0;
				}
				return r;
			}
		}
	}
	static schemaRules(e) {
		let t = [];
		function n(e) {
			let n = e.priority == null ? 50 : e.priority, r = 0;
			for (; r < t.length; r++) {
				let e = t[r];
				if ((e.priority == null ? 50 : e.priority) < n) break;
			}
			t.splice(r, 0, e);
		}
		for (let t in e.marks) {
			let r = e.marks[t].spec.parseDOM;
			r && r.forEach((e) => {
				n(e = Os(e)), e.mark || e.ignore || e.clearMark || (e.mark = t);
			});
		}
		for (let t in e.nodes) {
			let r = e.nodes[t].spec.parseDOM;
			r && r.forEach((e) => {
				n(e = Os(e)), e.node || e.ignore || e.mark || (e.node = t);
			});
		}
		return t;
	}
	static fromSchema(t) {
		return t.cached.domParser || (t.cached.domParser = new e(t, e.schemaRules(t)));
	}
}, _s = {
	address: !0,
	article: !0,
	aside: !0,
	blockquote: !0,
	body: !0,
	canvas: !0,
	dd: !0,
	div: !0,
	dl: !0,
	fieldset: !0,
	figcaption: !0,
	figure: !0,
	footer: !0,
	form: !0,
	h1: !0,
	h2: !0,
	h3: !0,
	h4: !0,
	h5: !0,
	h6: !0,
	header: !0,
	hgroup: !0,
	hr: !0,
	li: !0,
	noscript: !0,
	ol: !0,
	output: !0,
	p: !0,
	pre: !0,
	section: !0,
	table: !0,
	tfoot: !0,
	ul: !0
}, vs = {
	head: !0,
	noscript: !0,
	object: !0,
	script: !0,
	style: !0,
	title: !0
}, ys = {
	ol: !0,
	ul: !0
}, bs = 1, xs = 2, Ss = 4;
function Cs(e, t, n) {
	return t == null ? e && e.whitespace == "pre" ? 3 : n & -5 : (t ? bs : 0) | (t === "full" ? xs : 0);
}
var ws = class {
	constructor(e, t, n, r, i, a) {
		this.type = e, this.attrs = t, this.marks = n, this.solid = r, this.options = a, this.content = [], this.activeMarks = P.none, this.match = i || (a & Ss ? null : e.contentMatch);
	}
	findWrapping(e) {
		if (!this.match) {
			if (!this.type) return [];
			let t = this.type.contentMatch.fillBefore(N.from(e));
			if (t) this.match = this.type.contentMatch.matchFragment(t);
			else {
				let t = this.type.contentMatch, n;
				return (n = t.findWrapping(e.type)) ? (this.match = t, n) : null;
			}
		}
		return this.match.findWrapping(e.type);
	}
	finish(e) {
		if (!(this.options & bs)) {
			let e = this.content[this.content.length - 1], t;
			if (e && e.isText && (t = /[ \t\r\n\u000c]+$/.exec(e.text))) {
				let n = e;
				e.text.length == t[0].length ? this.content.pop() : this.content[this.content.length - 1] = n.withText(n.text.slice(0, n.text.length - t[0].length));
			}
		}
		let t = N.from(this.content);
		return !e && this.match && (t = t.append(this.match.fillBefore(N.empty, !0))), this.type ? this.type.create(this.attrs, t, this.marks) : t;
	}
	inlineContext(e) {
		return this.type ? this.type.inlineContent : this.content.length ? this.content[0].isInline : e.parentNode && !_s.hasOwnProperty(e.parentNode.nodeName.toLowerCase());
	}
}, Ts = class {
	constructor(e, t, n) {
		this.parser = e, this.options = t, this.isOpen = n, this.open = 0, this.localPreserveWS = !1;
		let r = t.topNode, i, a = Cs(null, t.preserveWhitespace, 0) | (n ? Ss : 0);
		i = r ? new ws(r.type, r.attrs, P.none, !0, t.topMatch || r.type.contentMatch, a) : n ? new ws(null, null, P.none, !0, null, a) : new ws(e.schema.topNodeType, null, P.none, !0, null, a), this.nodes = [i], this.find = t.findPositions, this.needsBlock = !1;
	}
	get top() {
		return this.nodes[this.open];
	}
	addDOM(e, t) {
		e.nodeType == 3 ? this.addTextNode(e, t) : e.nodeType == 1 && this.addElement(e, t);
	}
	addTextNode(e, t) {
		let n = e.nodeValue, r = this.top, i = r.options & xs ? "full" : this.localPreserveWS || (r.options & bs) > 0, { schema: a } = this.parser;
		if (i === "full" || r.inlineContext(e) || /[^ \t\r\n\u000c]/.test(n)) {
			if (!i) {
				if (n = n.replace(/[ \t\r\n\u000c]+/g, " "), /^[ \t\r\n\u000c]/.test(n) && this.open == this.nodes.length - 1) {
					let t = r.content[r.content.length - 1], i = e.previousSibling;
					(!t || i && i.nodeName == "BR" || t.isText && /[ \t\r\n\u000c]$/.test(t.text)) && (n = n.slice(1));
				}
			} else if (i === "full") n = n.replace(/\r\n?/g, "\n");
			else if (a.linebreakReplacement && /[\r\n]/.test(n) && this.top.findWrapping(a.linebreakReplacement.create())) {
				let e = n.split(/\r?\n|\r/);
				for (let n = 0; n < e.length; n++) n && this.insertNode(a.linebreakReplacement.create(), t, !0), e[n] && this.insertNode(a.text(e[n]), t, !/\S/.test(e[n]));
				n = "";
			} else n = n.replace(/\r?\n|\r/g, " ");
			n && this.insertNode(a.text(n), t, !/\S/.test(n)), this.findInText(e);
		} else this.findInside(e);
	}
	addElement(e, t, n) {
		let r = this.localPreserveWS, i = this.top;
		(e.tagName == "PRE" || /pre/.test(e.style && e.style.whiteSpace)) && (this.localPreserveWS = !0);
		let a = e.nodeName.toLowerCase(), o;
		ys.hasOwnProperty(a) && this.parser.normalizeLists && Es(e);
		let s = this.options.ruleFromNode && this.options.ruleFromNode(e) || (o = this.parser.matchTag(e, this, n));
		out: if (s ? s.ignore : vs.hasOwnProperty(a)) this.findInside(e), this.ignoreFallback(e, t);
		else if (!s || s.skip || s.closeParent) {
			s && s.closeParent ? this.open = Math.max(0, this.open - 1) : s && s.skip.nodeType && (e = s.skip);
			let n, r = this.needsBlock;
			if (_s.hasOwnProperty(a)) i.content.length && i.content[0].isInline && this.open && (this.open--, i = this.top), n = !0, i.type || (this.needsBlock = !0);
			else if (!e.firstChild) {
				this.leafFallback(e, t);
				break out;
			}
			let o = s && s.skip ? t : this.readStyles(e, t);
			o && this.addAll(e, o), n && this.sync(i), this.needsBlock = r;
		} else {
			let n = this.readStyles(e, t);
			n && this.addElementByRule(e, s, n, s.consuming === !1 ? o : void 0);
		}
		this.localPreserveWS = r;
	}
	leafFallback(e, t) {
		e.nodeName == "BR" && this.top.type && this.top.type.inlineContent && this.addTextNode(e.ownerDocument.createTextNode("\n"), t);
	}
	ignoreFallback(e, t) {
		e.nodeName == "BR" && (!this.top.type || !this.top.type.inlineContent) && this.findPlace(this.parser.schema.text("-"), t, !0);
	}
	readStyles(e, t) {
		let n = e.style;
		if (n && n.length) for (let e = 0; e < this.parser.matchedStyles.length; e++) {
			let r = this.parser.matchedStyles[e], i = n.getPropertyValue(r);
			if (i) for (let e;;) {
				let n = this.parser.matchStyle(r, i, this, e);
				if (!n) break;
				if (n.ignore) return null;
				if (t = n.clearMark ? t.filter((e) => !n.clearMark(e)) : t.concat(this.parser.schema.marks[n.mark].create(n.attrs)), n.consuming === !1) e = n;
				else break;
			}
		}
		return t;
	}
	addElementByRule(e, t, n, r) {
		let i, a;
		if (t.node) {
			if (a = this.parser.schema.nodes[t.node], a.isLeaf) this.insertNode(a.create(t.attrs), n, e.nodeName == "BR") || this.leafFallback(e, n);
			else {
				let e = this.enter(a, t.attrs || null, n, t.preserveWhitespace);
				e && (i = !0, n = e);
			}
		} else {
			let e = this.parser.schema.marks[t.mark];
			n = n.concat(e.create(t.attrs));
		}
		let o = this.top;
		if (a && a.isLeaf) this.findInside(e);
		else if (r) this.addElement(e, n, r);
		else if (t.getContent) this.findInside(e), t.getContent(e, this.parser.schema).forEach((e) => this.insertNode(e, n, !1));
		else {
			let r = e;
			typeof t.contentElement == "string" ? r = e.querySelector(t.contentElement) : typeof t.contentElement == "function" ? r = t.contentElement(e) : t.contentElement && (r = t.contentElement), this.findAround(e, r, !0), this.addAll(r, n), this.findAround(e, r, !1);
		}
		i && this.sync(o) && this.open--;
	}
	addAll(e, t, n, r) {
		let i = n || 0;
		for (let a = n ? e.childNodes[n] : e.firstChild, o = r == null ? null : e.childNodes[r]; a != o; a = a.nextSibling, ++i) this.findAtPoint(e, i), this.addDOM(a, t);
		this.findAtPoint(e, i);
	}
	findPlace(e, t, n) {
		let r, i;
		for (let t = this.open, a = 0; t >= 0; t--) {
			let o = this.nodes[t], s = o.findWrapping(e);
			if (s && (!r || r.length > s.length + a) && (r = s, i = o, !s.length)) break;
			if (o.solid) {
				if (n) break;
				a += 2;
			}
		}
		if (!r) return null;
		this.sync(i);
		for (let e = 0; e < r.length; e++) t = this.enterInner(r[e], null, t, !1);
		return t;
	}
	insertNode(e, t, n) {
		if (e.isInline && this.needsBlock && !this.top.type) {
			let e = this.textblockFromContext();
			e && (t = this.enterInner(e, null, t));
		}
		let r = this.findPlace(e, t, n);
		if (r) {
			this.closeExtra();
			let t = this.top;
			t.match &&= t.match.matchType(e.type);
			let n = P.none;
			for (let i of r.concat(e.marks)) (t.type ? t.type.allowsMarkType(i.type) : ks(i.type, e.type)) && (n = i.addToSet(n));
			return t.content.push(e.mark(n)), !0;
		}
		return !1;
	}
	enter(e, t, n, r) {
		let i = this.findPlace(e.create(t), n, !1);
		return i &&= this.enterInner(e, t, n, !0, r), i;
	}
	enterInner(e, t, n, r = !1, i) {
		this.closeExtra();
		let a = this.top;
		a.match = a.match && a.match.matchType(e);
		let o = Cs(e, i, a.options);
		a.options & Ss && a.content.length == 0 && (o |= Ss);
		let s = P.none;
		return n = n.filter((t) => !(a.type ? a.type.allowsMarkType(t.type) : ks(t.type, e)) || (s = t.addToSet(s), !1)), this.nodes.push(new ws(e, t, s, r, null, o)), this.open++, n;
	}
	closeExtra(e = !1) {
		let t = this.nodes.length - 1;
		if (t > this.open) {
			for (; t > this.open; t--) this.nodes[t - 1].content.push(this.nodes[t].finish(e));
			this.nodes.length = this.open + 1;
		}
	}
	finish() {
		return this.open = 0, this.closeExtra(this.isOpen), this.nodes[0].finish(!!(this.isOpen || this.options.topOpen));
	}
	sync(e) {
		for (let t = this.open; t >= 0; t--) if (this.nodes[t] == e) return this.open = t, !0;
		else this.localPreserveWS && (this.nodes[t].options |= bs);
		return !1;
	}
	get currentPos() {
		this.closeExtra();
		let e = 0;
		for (let t = this.open; t >= 0; t--) {
			let n = this.nodes[t].content;
			for (let t = n.length - 1; t >= 0; t--) e += n[t].nodeSize;
			t && e++;
		}
		return e;
	}
	findAtPoint(e, t) {
		if (this.find) for (let n = 0; n < this.find.length; n++) this.find[n].node == e && this.find[n].offset == t && (this.find[n].pos = this.currentPos);
	}
	findInside(e) {
		if (this.find) for (let t = 0; t < this.find.length; t++) this.find[t].pos == null && e.nodeType == 1 && e.contains(this.find[t].node) && (this.find[t].pos = this.currentPos);
	}
	findAround(e, t, n) {
		if (e != t && this.find) for (let r = 0; r < this.find.length; r++) this.find[r].pos == null && e.nodeType == 1 && e.contains(this.find[r].node) && t.compareDocumentPosition(this.find[r].node) & (n ? 2 : 4) && (this.find[r].pos = this.currentPos);
	}
	findInText(e) {
		if (this.find) for (let t = 0; t < this.find.length; t++) this.find[t].node == e && (this.find[t].pos = this.currentPos - (e.nodeValue.length - this.find[t].offset));
	}
	matchesContext(e) {
		if (e.indexOf("|") > -1) return e.split(/\s*\|\s*/).some(this.matchesContext, this);
		let t = e.split("/"), n = this.options.context, r = !this.isOpen && (!n || n.parent.type == this.nodes[0].type), i = -(n ? n.depth + 1 : 0) + +!r, a = (e, o) => {
			for (; e >= 0; e--) {
				let s = t[e];
				if (s == "") {
					if (e == t.length - 1 || e == 0) continue;
					for (; o >= i; o--) if (a(e - 1, o)) return !0;
					return !1;
				}
				{
					let e = o > 0 || o == 0 && r ? this.nodes[o].type : n && o >= i ? n.node(o - i).type : null;
					if (!e || e.name != s && !e.isInGroup(s)) return !1;
					o--;
				}
			}
			return !0;
		};
		return a(t.length - 1, this.open);
	}
	textblockFromContext() {
		let e = this.options.context;
		if (e) for (let t = e.depth; t >= 0; t--) {
			let n = e.node(t).contentMatchAt(e.indexAfter(t)).defaultType;
			if (n && n.isTextblock && n.defaultAttrs) return n;
		}
		for (let e in this.parser.schema.nodes) {
			let t = this.parser.schema.nodes[e];
			if (t.isTextblock && t.defaultAttrs) return t;
		}
	}
};
function Es(e) {
	for (let t = e.firstChild, n = null; t; t = t.nextSibling) {
		let e = t.nodeType == 1 ? t.nodeName.toLowerCase() : null;
		e && ys.hasOwnProperty(e) && n ? (n.appendChild(t), t = n) : e == "li" ? n = t : e && (n = null);
	}
}
function Ds(e, t) {
	return (e.matches || e.msMatchesSelector || e.webkitMatchesSelector || e.mozMatchesSelector).call(e, t);
}
function Os(e) {
	let t = {};
	for (let n in e) t[n] = e[n];
	return t;
}
function ks(e, t) {
	let n = t.schema.nodes;
	for (let r in n) {
		let i = n[r];
		if (!i.allowsMarkType(e)) continue;
		let a = [], o = (e) => {
			a.push(e);
			for (let n = 0; n < e.edgeCount; n++) {
				let { type: r, next: i } = e.edge(n);
				if (r == t || a.indexOf(i) < 0 && o(i)) return !0;
			}
		};
		if (o(i.contentMatch)) return !0;
	}
}
var As = class e {
	constructor(e, t) {
		this.nodes = e, this.marks = t;
	}
	serializeFragment(e, t = {}, n) {
		n ||= Ms(t).createDocumentFragment();
		let r = n, i = [];
		return e.forEach((e) => {
			if (i.length || e.marks.length) {
				let n = 0, a = 0;
				for (; n < i.length && a < e.marks.length;) {
					let t = e.marks[a];
					if (!this.marks[t.type.name]) a++;
					else {
						if (!t.eq(i[n][0]) || t.type.spec.spanning === !1) break;
						n++, a++;
					}
				}
				for (; n < i.length;) r = i.pop()[1];
				for (; a < e.marks.length;) {
					let n = e.marks[a++], o = this.serializeMark(n, e.isInline, t);
					o && (i.push([n, r]), r.appendChild(o.dom), r = o.contentDOM || o.dom);
				}
			}
			r.appendChild(this.serializeNodeInner(e, t));
		}), n;
	}
	serializeNodeInner(e, t) {
		if (e.isText) return Ms(t).createTextNode(e.text);
		let { dom: n, contentDOM: r } = Is(Ms(t), this.nodes[e.type.name](e), null, e.attrs);
		if (r) {
			if (e.isLeaf) throw RangeError("Content hole not allowed in a leaf node spec");
			this.serializeFragment(e.content, t, r);
		}
		return n;
	}
	serializeNode(e, t = {}) {
		let n = this.serializeNodeInner(e, t);
		for (let r = e.marks.length - 1; r >= 0; r--) {
			let i = this.serializeMark(e.marks[r], e.isInline, t);
			i && ((i.contentDOM || i.dom).appendChild(n), n = i.dom);
		}
		return n;
	}
	serializeMark(e, t, n = {}) {
		let r = this.marks[e.type.name];
		return r && Is(Ms(n), r(e, t), null, e.attrs);
	}
	static renderSpec(e, t, n = null, r) {
		return typeof t == "string" ? { dom: e.createTextNode(t) } : Is(e, t, n, r);
	}
	static fromSchema(t) {
		return t.cached.domSerializer || (t.cached.domSerializer = new e(this.nodesFromSchema(t), this.marksFromSchema(t)));
	}
	static nodesFromSchema(e) {
		let t = js(e.nodes);
		return t.text ||= (e) => e.text, t;
	}
	static marksFromSchema(e) {
		return js(e.marks);
	}
};
function js(e) {
	let t = {};
	for (let n in e) {
		let r = e[n].spec.toDOM;
		r && (t[n] = r);
	}
	return t;
}
function Ms(e) {
	return e.document || window.document;
}
var Ns = /* @__PURE__ */ new WeakMap();
function Ps(e) {
	let t = Ns.get(e);
	return t === void 0 && Ns.set(e, t = Fs(e)), t;
}
function Fs(e) {
	let t = null;
	function n(e) {
		if (e && typeof e == "object") {
			if (Array.isArray(e)) {
				if (typeof e[0] == "string") t ||= [], t.push(e);
				else for (let t = 0; t < e.length; t++) n(e[t]);
			} else for (let t in e) n(e[t]);
		}
	}
	return n(e), t;
}
function Is(e, t, n, r) {
	if (t.nodeType == 1) return { dom: t };
	if (t.dom && t.dom.nodeType == 1) return t;
	let i = t[0], a;
	if (typeof i != "string") throw RangeError("Invalid array passed to renderSpec");
	if (r && (a = Ps(r)) && a.indexOf(t) > -1) throw RangeError("Using an array from an attribute object as a DOM spec. This may be an attempted cross site scripting attack.");
	let o = i.indexOf(" ");
	o > 0 && (n = i.slice(0, o), i = i.slice(o + 1));
	let s, c = n ? e.createElementNS(n, i) : e.createElement(i), l = t[1], u = 1;
	if (l && typeof l == "object" && l.nodeType == null && !Array.isArray(l)) {
		u = 2;
		for (let e in l) if (l[e] != null) {
			let t = e.indexOf(" ");
			t > 0 ? c.setAttributeNS(e.slice(0, t), e.slice(t + 1), l[e]) : e == "style" && c.style ? c.style.cssText = l[e] : c.setAttribute(e, l[e]);
		}
	}
	for (let i = u; i < t.length; i++) {
		let a = t[i];
		if (a === 0) {
			if (i < t.length - 1 || i > u) throw RangeError("Content hole must be the only child of its parent node");
			return {
				dom: c,
				contentDOM: c
			};
		}
		if (typeof a == "string") c.appendChild(e.createTextNode(a));
		else {
			let { dom: t, contentDOM: i } = Is(e, a, n, r);
			if (c.appendChild(t), i) {
				if (s) throw RangeError("Multiple content holes");
				s = i;
			}
		}
	}
	return {
		dom: c,
		contentDOM: s
	};
}
//#endregion
//#region node_modules/.pnpm/prosemirror-transform@1.12.2/node_modules/prosemirror-transform/dist/index.js
var Ls = 65535, Rs = 2 ** 16;
function zs(e, t) {
	return e + t * Rs;
}
function Bs(e) {
	return e & Ls;
}
function Vs(e) {
	return (e - (e & Ls)) / Rs;
}
var Hs = 1, Us = 2, Ws = 4, Gs = 8, Ks = class {
	constructor(e, t, n) {
		this.pos = e, this.delInfo = t, this.recover = n;
	}
	get deleted() {
		return (this.delInfo & Gs) > 0;
	}
	get deletedBefore() {
		return (this.delInfo & 5) > 0;
	}
	get deletedAfter() {
		return (this.delInfo & 6) > 0;
	}
	get deletedAcross() {
		return (this.delInfo & Ws) > 0;
	}
}, qs = class e {
	constructor(t, n = !1) {
		if (this.ranges = t, this.inverted = n, !t.length && e.empty) return e.empty;
	}
	recover(e) {
		let t = 0, n = Bs(e);
		if (!this.inverted) for (let e = 0; e < n; e++) t += this.ranges[e * 3 + 2] - this.ranges[e * 3 + 1];
		return this.ranges[n * 3] + t + Vs(e);
	}
	mapResult(e, t = 1) {
		return this._map(e, t, !1);
	}
	map(e, t = 1) {
		return this._map(e, t, !0);
	}
	_map(e, t, n) {
		let r = 0, i = this.inverted ? 2 : 1, a = this.inverted ? 1 : 2;
		for (let o = 0; o < this.ranges.length; o += 3) {
			let s = this.ranges[o] - (this.inverted ? r : 0);
			if (s > e) break;
			let c = this.ranges[o + i], l = this.ranges[o + a], u = s + c;
			if (e <= u) {
				let i = c ? e == s ? -1 : e == u ? 1 : t : t, a = s + r + (i < 0 ? 0 : l);
				if (n) return a;
				let d = e == (t < 0 ? s : u) ? null : zs(o / 3, e - s), f = 0;
				return c && (f |= e == s ? Us : e == u ? Hs : Ws, (t < 0 ? e != s : e != u) && (f |= Gs)), new Ks(a, f, d);
			}
			r += l - c;
		}
		return n ? e + r : new Ks(e + r, 0, null);
	}
	touches(e, t) {
		let n = 0, r = Bs(t), i = this.inverted ? 2 : 1, a = this.inverted ? 1 : 2;
		for (let t = 0; t < this.ranges.length; t += 3) {
			let o = this.ranges[t] - (this.inverted ? n : 0);
			if (o > e) break;
			let s = this.ranges[t + i];
			if (e <= o + s && t == r * 3) return !0;
			n += this.ranges[t + a] - s;
		}
		return !1;
	}
	forEach(e) {
		let t = this.inverted ? 2 : 1, n = this.inverted ? 1 : 2;
		for (let r = 0, i = 0; r < this.ranges.length; r += 3) {
			let a = this.ranges[r], o = a - (this.inverted ? i : 0), s = a + (this.inverted ? 0 : i), c = this.ranges[r + t], l = this.ranges[r + n];
			e(o, o + c, s, s + l), i += l - c;
		}
	}
	invert() {
		return new e(this.ranges, !this.inverted);
	}
	toString() {
		return (this.inverted ? "-" : "") + JSON.stringify(this.ranges);
	}
	static offset(t) {
		return t == 0 ? e.empty : new e(t < 0 ? [
			0,
			-t,
			0
		] : [
			0,
			0,
			t
		]);
	}
};
qs.empty = new qs([]);
var Js = class e {
	constructor(e, t, n = 0, r = e ? e.length : 0) {
		this.mirror = t, this.from = n, this.to = r, this._maps = e || [], this.ownData = !(e || t);
	}
	get maps() {
		return this._maps;
	}
	slice(t = 0, n = this.maps.length) {
		return new e(this._maps, this.mirror, t, n);
	}
	appendMap(e, t) {
		this.ownData ||= (this._maps = this._maps.slice(), this.mirror = this.mirror && this.mirror.slice(), !0), this.to = this._maps.push(e), t != null && this.setMirror(this._maps.length - 1, t);
	}
	appendMapping(e) {
		for (let t = 0, n = this._maps.length; t < e._maps.length; t++) {
			let r = e.getMirror(t);
			this.appendMap(e._maps[t], r != null && r < t ? n + r : void 0);
		}
	}
	getMirror(e) {
		if (this.mirror) {
			for (let t = 0; t < this.mirror.length; t++) if (this.mirror[t] == e) return this.mirror[t + (t % 2 ? -1 : 1)];
		}
	}
	setMirror(e, t) {
		this.mirror ||= [], this.mirror.push(e, t);
	}
	appendMappingInverted(e) {
		for (let t = e.maps.length - 1, n = this._maps.length + e._maps.length; t >= 0; t--) {
			let r = e.getMirror(t);
			this.appendMap(e._maps[t].invert(), r != null && r > t ? n - r - 1 : void 0);
		}
	}
	invert() {
		let t = new e();
		return t.appendMappingInverted(this), t;
	}
	map(e, t = 1) {
		if (this.mirror) return this._map(e, t, !0);
		for (let n = this.from; n < this.to; n++) e = this._maps[n].map(e, t);
		return e;
	}
	mapResult(e, t = 1) {
		return this._map(e, t, !1);
	}
	_map(e, t, n) {
		let r = 0;
		for (let n = this.from; n < this.to; n++) {
			let i = this._maps[n].mapResult(e, t);
			if (i.recover != null) {
				let t = this.getMirror(n);
				if (t != null && t > n && t < this.to) {
					n = t, e = this._maps[t].recover(i.recover);
					continue;
				}
			}
			r |= i.delInfo, e = i.pos;
		}
		return n ? e : new Ks(e, r, null);
	}
}, Ys = Object.create(null), I = class {
	getMap() {
		return qs.empty;
	}
	merge(e) {
		return null;
	}
	static fromJSON(e, t) {
		if (!t || !t.stepType) throw RangeError("Invalid input for Step.fromJSON");
		let n = Ys[t.stepType];
		if (!n) throw RangeError(`No step type ${t.stepType} defined`);
		return n.fromJSON(e, t);
	}
	static jsonID(e, t) {
		if (e in Ys) throw RangeError("Duplicate use of step JSON ID " + e);
		return Ys[e] = t, t.prototype.jsonID = e, t;
	}
}, Xs = class e {
	constructor(e, t) {
		this.doc = e, this.failed = t;
	}
	static ok(t) {
		return new e(t, null);
	}
	static fail(t) {
		return new e(null, t);
	}
	static fromReplace(t, n, r, i) {
		try {
			return e.ok(t.replace(n, r, i));
		} catch (t) {
			if (t instanceof xo) return e.fail(t.message);
			throw t;
		}
	}
};
function Zs(e, t, n) {
	let r = [];
	for (let i = 0; i < e.childCount; i++) {
		let a = e.child(i);
		a.content.size && (a = a.copy(Zs(a.content, t, a))), a.isInline && (a = t(a, n, i)), r.push(a);
	}
	return N.fromArray(r);
}
var Qs = class e extends I {
	constructor(e, t, n) {
		super(), this.from = e, this.to = t, this.mark = n;
	}
	apply(e) {
		let t = e.slice(this.from, this.to), n = e.resolve(this.from), r = n.node(n.sharedDepth(this.to)), i = new F(Zs(t.content, (e, t) => !e.isAtom || !t.type.allowsMarkType(this.mark.type) ? e : e.mark(this.mark.addToSet(e.marks)), r), t.openStart, t.openEnd);
		return Xs.fromReplace(e, this.from, this.to, i);
	}
	invert() {
		return new $s(this.from, this.to, this.mark);
	}
	map(t) {
		let n = t.mapResult(this.from, 1), r = t.mapResult(this.to, -1);
		return n.deleted && r.deleted || n.pos >= r.pos ? null : new e(n.pos, r.pos, this.mark);
	}
	merge(t) {
		return t instanceof e && t.mark.eq(this.mark) && this.from <= t.to && this.to >= t.from ? new e(Math.min(this.from, t.from), Math.max(this.to, t.to), this.mark) : null;
	}
	toJSON() {
		return {
			stepType: "addMark",
			mark: this.mark.toJSON(),
			from: this.from,
			to: this.to
		};
	}
	static fromJSON(t, n) {
		if (typeof n.from != "number" || typeof n.to != "number") throw RangeError("Invalid input for AddMarkStep.fromJSON");
		return new e(n.from, n.to, t.markFromJSON(n.mark));
	}
};
I.jsonID("addMark", Qs);
var $s = class e extends I {
	constructor(e, t, n) {
		super(), this.from = e, this.to = t, this.mark = n;
	}
	apply(e) {
		let t = e.slice(this.from, this.to), n = new F(Zs(t.content, (e) => e.mark(this.mark.removeFromSet(e.marks)), e), t.openStart, t.openEnd);
		return Xs.fromReplace(e, this.from, this.to, n);
	}
	invert() {
		return new Qs(this.from, this.to, this.mark);
	}
	map(t) {
		let n = t.mapResult(this.from, 1), r = t.mapResult(this.to, -1);
		return n.deleted && r.deleted || n.pos >= r.pos ? null : new e(n.pos, r.pos, this.mark);
	}
	merge(t) {
		return t instanceof e && t.mark.eq(this.mark) && this.from <= t.to && this.to >= t.from ? new e(Math.min(this.from, t.from), Math.max(this.to, t.to), this.mark) : null;
	}
	toJSON() {
		return {
			stepType: "removeMark",
			mark: this.mark.toJSON(),
			from: this.from,
			to: this.to
		};
	}
	static fromJSON(t, n) {
		if (typeof n.from != "number" || typeof n.to != "number") throw RangeError("Invalid input for RemoveMarkStep.fromJSON");
		return new e(n.from, n.to, t.markFromJSON(n.mark));
	}
};
I.jsonID("removeMark", $s);
var ec = class e extends I {
	constructor(e, t) {
		super(), this.pos = e, this.mark = t;
	}
	apply(e) {
		let t = e.nodeAt(this.pos);
		if (!t) return Xs.fail("No node at mark step's position");
		let n = t.type.create(t.attrs, null, this.mark.addToSet(t.marks));
		return Xs.fromReplace(e, this.pos, this.pos + 1, new F(N.from(n), 0, +!t.isLeaf));
	}
	invert(t) {
		let n = t.nodeAt(this.pos);
		if (n) {
			let t = this.mark.addToSet(n.marks);
			if (t.length == n.marks.length) {
				for (let r = 0; r < n.marks.length; r++) if (!n.marks[r].isInSet(t)) return new e(this.pos, n.marks[r]);
				return new e(this.pos, this.mark);
			}
		}
		return new tc(this.pos, this.mark);
	}
	map(t) {
		let n = t.mapResult(this.pos, 1);
		return n.deletedAfter ? null : new e(n.pos, this.mark);
	}
	toJSON() {
		return {
			stepType: "addNodeMark",
			pos: this.pos,
			mark: this.mark.toJSON()
		};
	}
	static fromJSON(t, n) {
		if (typeof n.pos != "number") throw RangeError("Invalid input for AddNodeMarkStep.fromJSON");
		return new e(n.pos, t.markFromJSON(n.mark));
	}
};
I.jsonID("addNodeMark", ec);
var tc = class e extends I {
	constructor(e, t) {
		super(), this.pos = e, this.mark = t;
	}
	apply(e) {
		let t = e.nodeAt(this.pos);
		if (!t) return Xs.fail("No node at mark step's position");
		let n = t.type.create(t.attrs, null, this.mark.removeFromSet(t.marks));
		return Xs.fromReplace(e, this.pos, this.pos + 1, new F(N.from(n), 0, +!t.isLeaf));
	}
	invert(e) {
		let t = e.nodeAt(this.pos);
		return !t || !this.mark.isInSet(t.marks) ? this : new ec(this.pos, this.mark);
	}
	map(t) {
		let n = t.mapResult(this.pos, 1);
		return n.deletedAfter ? null : new e(n.pos, this.mark);
	}
	toJSON() {
		return {
			stepType: "removeNodeMark",
			pos: this.pos,
			mark: this.mark.toJSON()
		};
	}
	static fromJSON(t, n) {
		if (typeof n.pos != "number") throw RangeError("Invalid input for RemoveNodeMarkStep.fromJSON");
		return new e(n.pos, t.markFromJSON(n.mark));
	}
};
I.jsonID("removeNodeMark", tc);
var nc = class e extends I {
	constructor(e, t, n, r = !1) {
		super(), this.from = e, this.to = t, this.slice = n, this.structure = r;
	}
	apply(e) {
		return this.structure && ic(e, this.from, this.to) ? Xs.fail("Structure replace would overwrite content") : Xs.fromReplace(e, this.from, this.to, this.slice);
	}
	getMap() {
		return new qs([
			this.from,
			this.to - this.from,
			this.slice.size
		]);
	}
	invert(t) {
		return new e(this.from, this.from + this.slice.size, t.slice(this.from, this.to));
	}
	map(t) {
		let n = t.mapResult(this.to, -1), r = this.from == this.to && e.MAP_BIAS < 0 ? n : t.mapResult(this.from, 1);
		return r.deletedAcross && n.deletedAcross ? null : new e(r.pos, Math.max(r.pos, n.pos), this.slice, this.structure);
	}
	merge(t) {
		if (!(t instanceof e) || t.structure || this.structure) return null;
		if (this.from + this.slice.size == t.from && !this.slice.openEnd && !t.slice.openStart) {
			let n = this.slice.size + t.slice.size == 0 ? F.empty : new F(this.slice.content.append(t.slice.content), this.slice.openStart, t.slice.openEnd);
			return new e(this.from, this.to + (t.to - t.from), n, this.structure);
		}
		if (t.to == this.from && !this.slice.openStart && !t.slice.openEnd) {
			let n = this.slice.size + t.slice.size == 0 ? F.empty : new F(t.slice.content.append(this.slice.content), t.slice.openStart, this.slice.openEnd);
			return new e(t.from, this.to, n, this.structure);
		}
		return null;
	}
	toJSON() {
		let e = {
			stepType: "replace",
			from: this.from,
			to: this.to
		};
		return this.slice.size && (e.slice = this.slice.toJSON()), this.structure && (e.structure = !0), e;
	}
	static fromJSON(t, n) {
		if (typeof n.from != "number" || typeof n.to != "number") throw RangeError("Invalid input for ReplaceStep.fromJSON");
		return new e(n.from, n.to, F.fromJSON(t, n.slice), !!n.structure);
	}
};
nc.MAP_BIAS = 1, I.jsonID("replace", nc);
var rc = class e extends I {
	constructor(e, t, n, r, i, a, o = !1) {
		super(), this.from = e, this.to = t, this.gapFrom = n, this.gapTo = r, this.slice = i, this.insert = a, this.structure = o;
	}
	apply(e) {
		if (this.structure && (ic(e, this.from, this.gapFrom) || ic(e, this.gapTo, this.to))) return Xs.fail("Structure gap-replace would overwrite content");
		let t = e.slice(this.gapFrom, this.gapTo);
		if (t.openStart || t.openEnd) return Xs.fail("Gap is not a flat range");
		let n = this.slice.insertAt(this.insert, t.content);
		return n ? Xs.fromReplace(e, this.from, this.to, n) : Xs.fail("Content does not fit in gap");
	}
	getMap() {
		return new qs([
			this.from,
			this.gapFrom - this.from,
			this.insert,
			this.gapTo,
			this.to - this.gapTo,
			this.slice.size - this.insert
		]);
	}
	invert(t) {
		let n = this.gapTo - this.gapFrom;
		return new e(this.from, this.from + this.slice.size + n, this.from + this.insert, this.from + this.insert + n, t.slice(this.from, this.to).removeBetween(this.gapFrom - this.from, this.gapTo - this.from), this.gapFrom - this.from, this.structure);
	}
	map(t) {
		let n = t.mapResult(this.from, 1), r = t.mapResult(this.to, -1), i = this.from == this.gapFrom ? n.pos : t.map(this.gapFrom, -1), a = this.to == this.gapTo ? r.pos : t.map(this.gapTo, 1);
		return n.deletedAcross && r.deletedAcross || i < n.pos || a > r.pos ? null : new e(n.pos, r.pos, i, a, this.slice, this.insert, this.structure);
	}
	toJSON() {
		let e = {
			stepType: "replaceAround",
			from: this.from,
			to: this.to,
			gapFrom: this.gapFrom,
			gapTo: this.gapTo,
			insert: this.insert
		};
		return this.slice.size && (e.slice = this.slice.toJSON()), this.structure && (e.structure = !0), e;
	}
	static fromJSON(t, n) {
		if (typeof n.from != "number" || typeof n.to != "number" || typeof n.gapFrom != "number" || typeof n.gapTo != "number" || typeof n.insert != "number") throw RangeError("Invalid input for ReplaceAroundStep.fromJSON");
		return new e(n.from, n.to, n.gapFrom, n.gapTo, F.fromJSON(t, n.slice), n.insert, !!n.structure);
	}
};
I.jsonID("replaceAround", rc);
function ic(e, t, n) {
	let r = e.resolve(t), i = n - t, a = r.depth;
	for (; i > 0 && a > 0 && r.indexAfter(a) == r.node(a).childCount;) a--, i--;
	if (i > 0) {
		let e = r.node(a).maybeChild(r.indexAfter(a));
		for (; i > 0;) {
			if (!e || e.isLeaf) return !0;
			e = e.firstChild, i--;
		}
	}
	return !1;
}
function ac(e, t, n, r) {
	let i = [], a = [], o, s;
	e.doc.nodesBetween(t, n, (e, c, l) => {
		if (!e.isInline) return;
		let u = e.marks;
		if (!r.isInSet(u) && l.type.allowsMarkType(r.type)) {
			let l = Math.max(c, t), d = Math.min(c + e.nodeSize, n), f = r.addToSet(u);
			for (let e = 0; e < u.length; e++) u[e].isInSet(f) || (o && o.to == l && o.mark.eq(u[e]) ? o.to = d : i.push(o = new $s(l, d, u[e])));
			s && s.to == l ? s.to = d : a.push(s = new Qs(l, d, r));
		}
	}), i.forEach((t) => e.step(t)), a.forEach((t) => e.step(t));
}
function oc(e, t, n, r) {
	let i = [], a = 0;
	e.doc.nodesBetween(t, n, (e, o) => {
		if (!e.isInline) return;
		a++;
		let s = null;
		if (r instanceof ds) {
			let t = e.marks, n;
			for (; n = r.isInSet(t);) (s ||= []).push(n), t = n.removeFromSet(t);
		} else r ? r.isInSet(e.marks) && (s = [r]) : s = e.marks;
		if (s && s.length) {
			let r = Math.min(o + e.nodeSize, n);
			for (let e = 0; e < s.length; e++) {
				let n = s[e], c;
				for (let e = 0; e < i.length; e++) {
					let t = i[e];
					t.step == a - 1 && n.eq(i[e].style) && (c = t);
				}
				c ? (c.to = r, c.step = a) : i.push({
					style: n,
					from: Math.max(o, t),
					to: r,
					step: a
				});
			}
		}
	}), i.forEach((t) => e.step(new $s(t.from, t.to, t.style)));
}
function sc(e, t, n, r = n.contentMatch, i = !0) {
	let a = e.doc.nodeAt(t), o = [], s = t + 1;
	for (let t = 0; t < a.childCount; t++) {
		let c = a.child(t), l = s + c.nodeSize, u = r.matchType(c.type);
		if (!u) o.push(new nc(s, l, F.empty));
		else {
			r = u;
			for (let t = 0; t < c.marks.length; t++) n.allowsMarkType(c.marks[t].type) || e.step(new $s(s, l, c.marks[t]));
			if (i && c.isText && n.whitespace != "pre") {
				let e, t = /\r?\n|\r/g, r;
				for (; e = t.exec(c.text);) r ||= new F(N.from(n.schema.text(" ", n.allowedMarks(c.marks))), 0, 0), o.push(new nc(s + e.index, s + e.index + e[0].length, r));
			}
		}
		s = l;
	}
	if (!r.validEnd) {
		let t = r.fillBefore(N.empty, !0);
		e.replace(s, s, new F(t, 0, 0));
	}
	for (let t = o.length - 1; t >= 0; t--) e.step(o[t]);
}
function cc(e, t, n) {
	return (t == 0 || e.canReplace(t, e.childCount)) && (n == e.childCount || e.canReplace(0, n));
}
function lc(e) {
	let t = e.parent.content.cutByIndex(e.startIndex, e.endIndex);
	for (let n = e.depth, r = 0, i = 0;; --n) {
		let a = e.$from.node(n), o = e.$from.index(n) + r, s = e.$to.indexAfter(n) - i;
		if (n < e.depth && a.canReplace(o, s, t)) return n;
		if (n == 0 || a.type.spec.isolating || !cc(a, o, s)) break;
		o && (r = 1), s < a.childCount && (i = 1);
	}
	return null;
}
function uc(e, t, n) {
	let { $from: r, $to: i, depth: a } = t, o = r.before(a + 1), s = i.after(a + 1), c = o, l = s, u = N.empty, d = 0;
	for (let e = a, t = !1; e > n; e--) t || r.index(e) > 0 ? (t = !0, u = N.from(r.node(e).copy(u)), d++) : c--;
	let f = N.empty, p = 0;
	for (let e = a, t = !1; e > n; e--) t || i.after(e + 1) < i.end(e) ? (t = !0, f = N.from(i.node(e).copy(f)), p++) : l++;
	e.step(new rc(c, l, o, s, new F(u.append(f), d, p), u.size - d, !0));
}
function dc(e, t, n = null, r = e) {
	let i = pc(e, t), a = i && mc(r, t);
	return a ? i.map(fc).concat({
		type: t,
		attrs: n
	}).concat(a.map(fc)) : null;
}
function fc(e) {
	return {
		type: e,
		attrs: null
	};
}
function pc(e, t) {
	let { parent: n, startIndex: r, endIndex: i } = e, a = n.contentMatchAt(r).findWrapping(t);
	if (!a) return null;
	let o = a.length ? a[0] : t;
	return n.canReplaceWith(r, i, o) ? a : null;
}
function mc(e, t) {
	let { parent: n, startIndex: r, endIndex: i } = e, a = n.child(r), o = t.contentMatch.findWrapping(a.type);
	if (!o) return null;
	let s = (o.length ? o[o.length - 1] : t).contentMatch;
	for (let e = r; s && e < i; e++) s = s.matchType(n.child(e).type);
	return !s || !s.validEnd ? null : o;
}
function hc(e, t, n) {
	let r = N.empty;
	for (let e = n.length - 1; e >= 0; e--) {
		if (r.size) {
			let t = n[e].type.contentMatch.matchFragment(r);
			if (!t || !t.validEnd) throw RangeError("Wrapper type given to Transform.wrap does not form valid content of its parent wrapper");
		}
		r = N.from(n[e].type.create(n[e].attrs, r));
	}
	let i = t.start, a = t.end;
	e.step(new rc(i, a, i, a, new F(r, 0, 0), n.length, !0));
}
function gc(e, t, n, r, i) {
	if (!r.isTextblock) throw RangeError("Type given to setBlockType should be a textblock");
	let a = e.steps.length;
	e.doc.nodesBetween(t, n, (t, n) => {
		let o = typeof i == "function" ? i(t) : i;
		if (t.isTextblock && !t.hasMarkup(r, o) && yc(e.doc, e.mapping.slice(a).map(n), r)) {
			let i = null;
			if (r.schema.linebreakReplacement) {
				let e = r.whitespace == "pre", t = !!r.contentMatch.matchType(r.schema.linebreakReplacement);
				e && !t ? i = !1 : !e && t && (i = !0);
			}
			i === !1 && vc(e, t, n, a), sc(e, e.mapping.slice(a).map(n, 1), r, void 0, i === null);
			let s = e.mapping.slice(a), c = s.map(n, 1), l = s.map(n + t.nodeSize, 1);
			return e.step(new rc(c, l, c + 1, l - 1, new F(N.from(r.create(o, null, t.marks)), 0, 0), 1, !0)), i === !0 && _c(e, t, n, a), !1;
		}
	});
}
function _c(e, t, n, r) {
	t.forEach((i, a) => {
		if (i.isText) {
			let o, s = /\r?\n|\r/g;
			for (; o = s.exec(i.text);) {
				let i = e.mapping.slice(r).map(n + 1 + a + o.index);
				e.replaceWith(i, i + 1, t.type.schema.linebreakReplacement.create());
			}
		}
	});
}
function vc(e, t, n, r) {
	t.forEach((i, a) => {
		if (i.type == i.type.schema.linebreakReplacement) {
			let i = e.mapping.slice(r).map(n + 1 + a);
			e.replaceWith(i, i + 1, t.type.schema.text("\n"));
		}
	});
}
function yc(e, t, n) {
	let r = e.resolve(t), i = r.index();
	return r.parent.canReplaceWith(i, i + 1, n);
}
function bc(e, t, n, r, i) {
	let a = e.doc.nodeAt(t);
	if (!a) throw RangeError("No node at given position");
	n ||= a.type;
	let o = n.create(r, null, i || a.marks);
	if (a.isLeaf) return e.replaceWith(t, t + a.nodeSize, o);
	if (!n.validContent(a.content)) throw RangeError("Invalid content for node type " + n.name);
	e.step(new rc(t, t + a.nodeSize, t + 1, t + a.nodeSize - 1, new F(N.from(o), 0, 0), 1, !0));
}
function xc(e, t, n = 1, r) {
	let i = e.resolve(t), a = i.depth - n, o = r && r[r.length - 1] || i.parent;
	if (a < 0 || i.parent.type.spec.isolating || !i.parent.canReplace(i.index(), i.parent.childCount) || !o.type.validContent(i.parent.content.cutByIndex(i.index(), i.parent.childCount))) return !1;
	for (let e = i.depth - 1, t = n - 2; e > a; e--, t--) {
		let n = i.node(e), a = i.index(e);
		if (n.type.spec.isolating) return !1;
		let o = n.content.cutByIndex(a, n.childCount), s = r && r[t + 1];
		s && (o = o.replaceChild(0, s.type.create(s.attrs)));
		let c = r && r[t] || n;
		if (!n.canReplace(a + 1, n.childCount) || !c.type.validContent(o)) return !1;
	}
	let s = i.indexAfter(a), c = r && r[0];
	return i.node(a).canReplaceWith(s, s, c ? c.type : i.node(a + 1).type);
}
function Sc(e, t, n = 1, r) {
	let i = e.doc.resolve(t), a = N.empty, o = N.empty;
	for (let e = i.depth, t = i.depth - n, s = n - 1; e > t; e--, s--) {
		a = N.from(i.node(e).copy(a));
		let t = r && r[s];
		o = N.from(t ? t.type.create(t.attrs, o) : i.node(e).copy(o));
	}
	e.step(new nc(t, t, new F(a.append(o), n, n), !0));
}
function Cc(e, t) {
	let n = e.resolve(t), r = n.index();
	return Tc(n.nodeBefore, n.nodeAfter) && n.parent.canReplace(r, r + 1);
}
function wc(e, t) {
	t.content.size || e.type.compatibleContent(t.type);
	let n = e.contentMatchAt(e.childCount), { linebreakReplacement: r } = e.type.schema;
	for (let i = 0; i < t.childCount; i++) {
		let a = t.child(i), o = a.type == r ? e.type.schema.nodes.text : a.type;
		if (n = n.matchType(o), !n || !e.type.allowsMarks(a.marks)) return !1;
	}
	return n.validEnd;
}
function Tc(e, t) {
	return !!(e && t && !e.isLeaf && wc(e, t));
}
function Ec(e, t, n) {
	let r = null, { linebreakReplacement: i } = e.doc.type.schema, a = e.doc.resolve(t - n), o = a.node().type;
	if (i && o.inlineContent) {
		let e = o.whitespace == "pre", t = !!o.contentMatch.matchType(i);
		e && !t ? r = !1 : !e && t && (r = !0);
	}
	let s = e.steps.length;
	if (r === !1) {
		let r = e.doc.resolve(t + n);
		vc(e, r.node(), r.before(), s);
	}
	o.inlineContent && sc(e, t + n - 1, o, a.node().contentMatchAt(a.index()), r == null);
	let c = e.mapping.slice(s), l = c.map(t - n);
	if (e.step(new nc(l, c.map(t + n, -1), F.empty, !0)), r === !0) {
		let t = e.doc.resolve(l);
		_c(e, t.node(), t.before(), e.steps.length);
	}
	return e;
}
function Dc(e, t, n) {
	let r = e.resolve(t);
	if (r.parent.canReplaceWith(r.index(), r.index(), n)) return t;
	if (r.parentOffset == 0) for (let e = r.depth - 1; e >= 0; e--) {
		let t = r.index(e);
		if (r.node(e).canReplaceWith(t, t, n)) return r.before(e + 1);
		if (t > 0) return null;
	}
	if (r.parentOffset == r.parent.content.size) for (let e = r.depth - 1; e >= 0; e--) {
		let t = r.indexAfter(e);
		if (r.node(e).canReplaceWith(t, t, n)) return r.after(e + 1);
		if (t < r.node(e).childCount) return null;
	}
	return null;
}
function Oc(e, t, n) {
	let r = e.resolve(t);
	if (!n.content.size) return t;
	let i = n.content;
	for (let e = 0; e < n.openStart; e++) i = i.firstChild.content;
	for (let e = 1; e <= (n.openStart == 0 && n.size ? 2 : 1); e++) for (let t = r.depth; t >= 0; t--) {
		let n = t == r.depth ? 0 : r.pos <= (r.start(t + 1) + r.end(t + 1)) / 2 ? -1 : 1, a = r.index(t) + +(n > 0), o = r.node(t), s = !1;
		if (e == 1) s = o.canReplace(a, a, i);
		else {
			let e = o.contentMatchAt(a).findWrapping(i.firstChild.type);
			s = e && o.canReplaceWith(a, a, e[0]);
		}
		if (s) return n == 0 ? r.pos : n < 0 ? r.before(t + 1) : r.after(t + 1);
	}
	return null;
}
function kc(e, t, n = t, r = F.empty) {
	if (t == n && !r.size) return null;
	let i = e.resolve(t), a = e.resolve(n);
	return Ac(i, a, r) ? new nc(t, n, r) : new jc(i, a, r).fit();
}
function Ac(e, t, n) {
	return !n.openStart && !n.openEnd && e.start() == t.start() && e.parent.canReplace(e.index(), t.index(), n.content);
}
var jc = class {
	constructor(e, t, n) {
		this.$from = e, this.$to = t, this.unplaced = n, this.frontier = [], this.placed = N.empty;
		for (let t = 0; t <= e.depth; t++) {
			let n = e.node(t);
			this.frontier.push({
				type: n.type,
				match: n.contentMatchAt(e.indexAfter(t))
			});
		}
		for (let t = e.depth; t > 0; t--) this.placed = N.from(e.node(t).copy(this.placed));
	}
	get depth() {
		return this.frontier.length - 1;
	}
	fit() {
		for (; this.unplaced.size;) {
			let e = this.findFittable();
			e ? this.placeNodes(e) : this.openMore() || this.dropNode();
		}
		let e = this.mustMoveInline(), t = this.placed.size - this.depth - this.$from.depth, n = this.$from, r = this.close(e < 0 ? this.$to : n.doc.resolve(e));
		if (!r) return null;
		let i = this.placed, a = n.depth, o = r.depth;
		for (; a && o && i.childCount == 1;) i = i.firstChild.content, a--, o--;
		let s = new F(i, a, o);
		return e > -1 ? new rc(n.pos, e, this.$to.pos, this.$to.end(), s, t) : s.size || n.pos != this.$to.pos ? new nc(n.pos, r.pos, s) : null;
	}
	findFittable() {
		let e = this.unplaced.openStart;
		for (let t = this.unplaced.content, n = 0, r = this.unplaced.openEnd; n < e; n++) {
			let i = t.firstChild;
			if (t.childCount > 1 && (r = 0), i.type.spec.isolating && r <= n) {
				e = n;
				break;
			}
			t = i.content;
		}
		for (let t = 1; t <= 2; t++) for (let n = t == 1 ? e : this.unplaced.openStart; n >= 0; n--) {
			let e, r = null;
			n ? (r = Pc(this.unplaced.content, n - 1).firstChild, e = r.content) : e = this.unplaced.content;
			let i = e.firstChild;
			for (let e = this.depth; e >= 0; e--) {
				let { type: a, match: o } = this.frontier[e], s, c = null;
				if (t == 1 && (i ? o.matchType(i.type) || (c = o.fillBefore(N.from(i), !1)) : r && a.compatibleContent(r.type))) return {
					sliceDepth: n,
					frontierDepth: e,
					parent: r,
					inject: c
				};
				if (t == 2 && i && (s = o.findWrapping(i.type))) return {
					sliceDepth: n,
					frontierDepth: e,
					parent: r,
					wrap: s
				};
				if (r && o.matchType(r.type)) break;
			}
		}
	}
	openMore() {
		let { content: e, openStart: t, openEnd: n } = this.unplaced;
		return zc(e, -1) <= t ? !1 : (this.unplaced.size > 1 && zc(e, 1) > n && n++, this.unplaced = new F(e, t + 1, n), !0);
	}
	dropNode() {
		let { content: e, openStart: t, openEnd: n } = this.unplaced, r = Pc(e, t);
		if (r.childCount <= 1 && t > 0) {
			let i = e.size - t <= t + r.size;
			this.unplaced = new F(Mc(e, t - 1, 1), t - 1, i ? t - 1 : n);
		} else this.unplaced = new F(Mc(e, t, 1), t, n);
	}
	placeNodes({ sliceDepth: e, frontierDepth: t, parent: n, inject: r, wrap: i }) {
		for (; this.depth > t;) this.closeFrontierNode();
		if (i) for (let e = 0; e < i.length; e++) this.openFrontierNode(i[e]);
		let a = this.unplaced, o = n ? n.content : a.content, s = a.openStart - e, c = 0, l = [], { match: u, type: d } = this.frontier[t];
		if (r) {
			for (let e = 0; e < r.childCount; e++) l.push(r.child(e));
			u = u.matchFragment(r);
		}
		let f = o.size + e - (a.content.size - a.openEnd);
		for (; c < o.childCount;) {
			let e = o.child(c), t = u.matchType(e.type);
			if (!t) break;
			c++, (c > 1 || s == 0 || e.content.size) && (u = t, l.push(Fc(e.mark(d.allowedMarks(e.marks)), c == 1 ? s : 0, c == o.childCount ? f : -1)));
		}
		let p = c == o.childCount;
		p || (f = -1), this.placed = Nc(this.placed, t, N.from(l)), this.frontier[t].match = u, p && f < 0 && n && n.type == this.frontier[this.depth].type && this.frontier.length > 1 && this.closeFrontierNode();
		for (let e = 0, t = o; e < f; e++) {
			let e = t.lastChild;
			this.frontier.push({
				type: e.type,
				match: e.contentMatchAt(e.childCount)
			}), t = e.content;
		}
		this.unplaced = p ? e == 0 ? F.empty : new F(Mc(a.content, e - 1, 1), e - 1, f < 0 ? a.openEnd : e - 1) : new F(Mc(a.content, e, c), a.openStart, a.openEnd);
	}
	mustMoveInline() {
		if (!this.$to.parent.isTextblock) return -1;
		let e = this.frontier[this.depth], t;
		if (!e.type.isTextblock || !Ic(this.$to, this.$to.depth, e.type, e.match, !1) || this.$to.depth == this.depth && (t = this.findCloseLevel(this.$to)) && t.depth == this.depth) return -1;
		let { depth: n } = this.$to, r = this.$to.after(n);
		for (; n > 1 && r == this.$to.end(--n);) ++r;
		return r;
	}
	findCloseLevel(e) {
		scan: for (let t = Math.min(this.depth, e.depth); t >= 0; t--) {
			let { match: n, type: r } = this.frontier[t], i = t < e.depth && e.end(t + 1) == e.pos + (e.depth - (t + 1)), a = Ic(e, t, r, n, i);
			if (a) {
				for (let n = t - 1; n >= 0; n--) {
					let { match: t, type: r } = this.frontier[n], i = Ic(e, n, r, t, !0);
					if (!i || i.childCount) continue scan;
				}
				return {
					depth: t,
					fit: a,
					move: i ? e.doc.resolve(e.after(t + 1)) : e
				};
			}
		}
	}
	close(e) {
		let t = this.findCloseLevel(e);
		if (!t) return null;
		for (; this.depth > t.depth;) this.closeFrontierNode();
		t.fit.childCount && (this.placed = Nc(this.placed, t.depth, t.fit)), e = t.move;
		for (let n = t.depth + 1; n <= e.depth; n++) {
			let t = e.node(n), r = t.type.contentMatch.fillBefore(t.content, !0, e.index(n));
			this.openFrontierNode(t.type, t.attrs, r);
		}
		return e;
	}
	openFrontierNode(e, t = null, n) {
		let r = this.frontier[this.depth];
		r.match = r.match.matchType(e), this.placed = Nc(this.placed, this.depth, N.from(e.create(t, n))), this.frontier.push({
			type: e,
			match: e.contentMatch
		});
	}
	closeFrontierNode() {
		let e = this.frontier.pop().match.fillBefore(N.empty, !0);
		e.childCount && (this.placed = Nc(this.placed, this.frontier.length, e));
	}
};
function Mc(e, t, n) {
	return t == 0 ? e.cutByIndex(n, e.childCount) : e.replaceChild(0, e.firstChild.copy(Mc(e.firstChild.content, t - 1, n)));
}
function Nc(e, t, n) {
	return t == 0 ? e.append(n) : e.replaceChild(e.childCount - 1, e.lastChild.copy(Nc(e.lastChild.content, t - 1, n)));
}
function Pc(e, t) {
	for (let n = 0; n < t; n++) e = e.firstChild.content;
	return e;
}
function Fc(e, t, n) {
	if (t <= 0) return e;
	let r = e.content;
	return t > 1 && (r = r.replaceChild(0, Fc(r.firstChild, t - 1, r.childCount == 1 ? n - 1 : 0))), t > 0 && (r = e.type.contentMatch.fillBefore(r).append(r), n <= 0 && (r = r.append(e.type.contentMatch.matchFragment(r).fillBefore(N.empty, !0)))), e.copy(r);
}
function Ic(e, t, n, r, i) {
	let a = e.node(t), o = i ? e.indexAfter(t) : e.index(t);
	if (o == a.childCount && !n.compatibleContent(a.type)) return null;
	let s = r.fillBefore(a.content, !0, o);
	return s && !Lc(n, a.content, o) ? s : null;
}
function Lc(e, t, n) {
	for (let r = n; r < t.childCount; r++) if (!e.allowsMarks(t.child(r).marks)) return !0;
	return !1;
}
function Rc(e) {
	return e.spec.defining || e.spec.definingForContent;
}
function zc(e, t) {
	for (let n = 0;; n++) {
		let r = t < 0 ? e.firstChild : e.lastChild;
		if (!r || r.isAtom) return n;
		e = r.content;
	}
}
function Bc(e, t, n, r) {
	if (!r.size) return e.deleteRange(t, n);
	let i = e.doc.resolve(t), a = e.doc.resolve(n);
	if (Ac(i, a, r)) return e.step(new nc(t, n, r));
	let o = Wc(i, a);
	o[o.length - 1] == 0 && o.pop();
	let s = -(i.depth + 1);
	o.unshift(s);
	for (let e = i.depth, t = i.pos - 1; e > 0; e--, t--) {
		let n = i.node(e).type.spec;
		if (n.defining || n.definingAsContext || n.isolating) break;
		o.indexOf(e) > -1 ? s = e : i.before(e) == t && o.splice(1, 0, -e);
	}
	let c = o.indexOf(s), l = [], u = r.openStart;
	for (let e = r.content, t = 0;; t++) {
		let n = e.firstChild;
		if (l.push(n), t == r.openStart) break;
		e = n.content;
	}
	for (let e = u - 1; e >= 0; e--) {
		let t = l[e], n = Rc(t.type);
		if (n && !t.sameMarkup(i.node(Math.abs(s) - 1))) u = e;
		else if (n || !t.type.isTextblock) break;
	}
	for (let t = r.openStart; t >= 0; t--) {
		let s = (t + u + 1) % (r.openStart + 1), d = l[s];
		if (d) for (let t = 0; t < o.length; t++) {
			let l = o[(t + c) % o.length], u = !0;
			l < 0 && (u = !1, l = -l);
			let f = i.node(l - 1), p = i.index(l - 1);
			if (f.canReplaceWith(p, p, d.type, d.marks)) return e.replace(i.before(l), u ? a.after(l) : n, new F(Vc(r.content, 0, r.openStart, s), s, r.openEnd));
		}
	}
	let d = e.steps.length;
	for (let s = o.length - 1; s >= 0 && (e.replace(t, n, r), !(e.steps.length > d)); s--) {
		let e = o[s];
		e < 0 || (t = i.before(e), n = a.after(e));
	}
}
function Vc(e, t, n, r, i) {
	if (t < n) {
		let i = e.firstChild;
		e = e.replaceChild(0, i.copy(Vc(i.content, t + 1, n, r, i)));
	}
	if (t > r) {
		let t = i.contentMatchAt(0), n = t.fillBefore(e).append(e);
		e = n.append(t.matchFragment(n).fillBefore(N.empty, !0));
	}
	return e;
}
function Hc(e, t, n, r) {
	if (!r.isInline && t == n && e.doc.resolve(t).parent.content.size) {
		let i = Dc(e.doc, t, r.type);
		i != null && (t = n = i);
	}
	e.replaceRange(t, n, new F(N.from(r), 0, 0));
}
function Uc(e, t, n) {
	let r = e.doc.resolve(t), i = e.doc.resolve(n);
	if (r.parent.isTextblock && i.parent.isTextblock && r.start() != i.start() && r.parentOffset == 0 && i.parentOffset == 0) {
		let a = r.sharedDepth(n), o = !1;
		for (let e = r.depth; e > a; e--) r.node(e).type.spec.isolating && (o = !0);
		for (let e = i.depth; e > a; e--) i.node(e).type.spec.isolating && (o = !0);
		if (!o) {
			for (let e = r.depth; e > 0 && t == r.start(e); e--) t = r.before(e);
			for (let e = i.depth; e > 0 && n == i.start(e); e--) n = i.before(e);
			r = e.doc.resolve(t), i = e.doc.resolve(n);
		}
	}
	let a = Wc(r, i);
	for (let t = 0; t < a.length; t++) {
		let n = a[t], o = t == a.length - 1;
		if (o && n == 0 || r.node(n).type.contentMatch.validEnd) return e.delete(r.start(n), i.end(n));
		if (n > 0 && (o || r.node(n - 1).canReplace(r.index(n - 1), i.indexAfter(n - 1)))) return e.delete(r.before(n), i.after(n));
	}
	for (let a = 1; a <= r.depth && a <= i.depth; a++) if (t - r.start(a) == r.depth - a && n > r.end(a) && i.end(a) - n != i.depth - a && r.start(a - 1) == i.start(a - 1) && r.node(a - 1).canReplace(r.index(a - 1), i.index(a - 1))) return e.delete(r.before(a), n);
	e.delete(t, n);
}
function Wc(e, t) {
	let n = [], r = Math.min(e.depth, t.depth);
	for (let i = r; i >= 0; i--) {
		let r = e.start(i);
		if (r < e.pos - (e.depth - i) || t.end(i) > t.pos + (t.depth - i) || e.node(i).type.spec.isolating || t.node(i).type.spec.isolating) break;
		(r == t.start(i) || i == e.depth && i == t.depth && e.parent.inlineContent && t.parent.inlineContent && i && t.start(i - 1) == r - 1) && n.push(i);
	}
	return n;
}
var Gc = class e extends I {
	constructor(e, t, n) {
		super(), this.pos = e, this.attr = t, this.value = n;
	}
	apply(e) {
		let t = e.nodeAt(this.pos);
		if (!t) return Xs.fail("No node at attribute step's position");
		let n = Object.create(null);
		for (let e in t.attrs) n[e] = t.attrs[e];
		n[this.attr] = this.value;
		let r = t.type.create(n, null, t.marks);
		return Xs.fromReplace(e, this.pos, this.pos + 1, new F(N.from(r), 0, +!t.isLeaf));
	}
	getMap() {
		return qs.empty;
	}
	invert(t) {
		return new e(this.pos, this.attr, t.nodeAt(this.pos).attrs[this.attr]);
	}
	map(t) {
		let n = t.mapResult(this.pos, 1);
		return n.deletedAfter ? null : new e(n.pos, this.attr, this.value);
	}
	toJSON() {
		return {
			stepType: "attr",
			pos: this.pos,
			attr: this.attr,
			value: this.value
		};
	}
	static fromJSON(t, n) {
		if (typeof n.pos != "number" || typeof n.attr != "string") throw RangeError("Invalid input for AttrStep.fromJSON");
		return new e(n.pos, n.attr, n.value);
	}
};
I.jsonID("attr", Gc);
var Kc = class e extends I {
	constructor(e, t) {
		super(), this.attr = e, this.value = t;
	}
	apply(e) {
		let t = Object.create(null);
		for (let n in e.attrs) t[n] = e.attrs[n];
		t[this.attr] = this.value;
		let n = e.type.create(t, e.content, e.marks);
		return Xs.ok(n);
	}
	getMap() {
		return qs.empty;
	}
	invert(t) {
		return new e(this.attr, t.attrs[this.attr]);
	}
	map(e) {
		return this;
	}
	toJSON() {
		return {
			stepType: "docAttr",
			attr: this.attr,
			value: this.value
		};
	}
	static fromJSON(t, n) {
		if (typeof n.attr != "string") throw RangeError("Invalid input for DocAttrStep.fromJSON");
		return new e(n.attr, n.value);
	}
};
I.jsonID("docAttr", Kc);
var qc = class extends Error {};
qc = function e(t) {
	let n = Error.call(this, t);
	return n.__proto__ = e.prototype, n;
}, qc.prototype = Object.create(Error.prototype), qc.prototype.constructor = qc, qc.prototype.name = "TransformError";
var Jc = class {
	constructor(e) {
		this.doc = e, this.steps = [], this.docs = [], this.mapping = new Js();
	}
	get before() {
		return this.docs.length ? this.docs[0] : this.doc;
	}
	step(e) {
		let t = this.maybeStep(e);
		if (t.failed) throw new qc(t.failed);
		return this;
	}
	maybeStep(e) {
		let t = e.apply(this.doc);
		return t.failed || this.addStep(e, t.doc), t;
	}
	get docChanged() {
		return this.steps.length > 0;
	}
	changedRange() {
		let e = 1e9, t = -1e9;
		for (let n = 0; n < this.mapping.maps.length; n++) {
			let r = this.mapping.maps[n];
			n && (e = r.map(e, 1), t = r.map(t, -1)), r.forEach((n, r, i, a) => {
				e = Math.min(e, i), t = Math.max(t, a);
			});
		}
		return e == 1e9 ? null : {
			from: e,
			to: t
		};
	}
	addStep(e, t) {
		this.docs.push(this.doc), this.steps.push(e), this.mapping.appendMap(e.getMap()), this.doc = t;
	}
	replace(e, t = e, n = F.empty) {
		let r = kc(this.doc, e, t, n);
		return r && this.step(r), this;
	}
	replaceWith(e, t, n) {
		return this.replace(e, t, new F(N.from(n), 0, 0));
	}
	delete(e, t) {
		return this.replace(e, t, F.empty);
	}
	insert(e, t) {
		return this.replaceWith(e, e, t);
	}
	replaceRange(e, t, n) {
		return Bc(this, e, t, n), this;
	}
	replaceRangeWith(e, t, n) {
		return Hc(this, e, t, n), this;
	}
	deleteRange(e, t) {
		return Uc(this, e, t), this;
	}
	lift(e, t) {
		return uc(this, e, t), this;
	}
	join(e, t = 1) {
		return Ec(this, e, t), this;
	}
	wrap(e, t) {
		return hc(this, e, t), this;
	}
	setBlockType(e, t = e, n, r = null) {
		return gc(this, e, t, n, r), this;
	}
	setNodeMarkup(e, t, n = null, r) {
		return bc(this, e, t, n, r), this;
	}
	setNodeAttribute(e, t, n) {
		return this.step(new Gc(e, t, n)), this;
	}
	setDocAttribute(e, t) {
		return this.step(new Kc(e, t)), this;
	}
	addNodeMark(e, t) {
		return this.step(new ec(e, t)), this;
	}
	removeNodeMark(e, t) {
		let n = this.doc.nodeAt(e);
		if (!n) throw RangeError("No node at position " + e);
		if (t instanceof P) t.isInSet(n.marks) && this.step(new tc(e, t));
		else {
			let r = n.marks, i, a = [];
			for (; i = t.isInSet(r);) a.push(new tc(e, i)), r = i.removeFromSet(r);
			for (let e = a.length - 1; e >= 0; e--) this.step(a[e]);
		}
		return this;
	}
	split(e, t = 1, n) {
		return Sc(this, e, t, n), this;
	}
	addMark(e, t, n) {
		return ac(this, e, t, n), this;
	}
	removeMark(e, t, n) {
		return oc(this, e, t, n), this;
	}
	clearIncompatible(e, t, n) {
		return sc(this, e, t, n), this;
	}
}, Yc = Object.create(null), L = class {
	constructor(e, t, n) {
		this.$anchor = e, this.$head = t, this.ranges = n || [new Xc(e.min(t), e.max(t))];
	}
	get anchor() {
		return this.$anchor.pos;
	}
	get head() {
		return this.$head.pos;
	}
	get from() {
		return this.$from.pos;
	}
	get to() {
		return this.$to.pos;
	}
	get $from() {
		return this.ranges[0].$from;
	}
	get $to() {
		return this.ranges[0].$to;
	}
	get empty() {
		let e = this.ranges;
		for (let t = 0; t < e.length; t++) if (e[t].$from.pos != e[t].$to.pos) return !1;
		return !0;
	}
	content() {
		return this.$from.doc.slice(this.from, this.to, !0);
	}
	replace(e, t = F.empty) {
		let n = t.content.lastChild, r = null;
		for (let e = 0; e < t.openEnd; e++) r = n, n = n.lastChild;
		let i = e.steps.length, a = this.ranges;
		for (let o = 0; o < a.length; o++) {
			let { $from: s, $to: c } = a[o], l = e.mapping.slice(i);
			e.replaceRange(l.map(s.pos), l.map(c.pos), o ? F.empty : t), o == 0 && il(e, i, (n ? n.isInline : r && r.isTextblock) ? -1 : 1);
		}
	}
	replaceWith(e, t) {
		let n = e.steps.length, r = this.ranges;
		for (let i = 0; i < r.length; i++) {
			let { $from: a, $to: o } = r[i], s = e.mapping.slice(n), c = s.map(a.pos), l = s.map(o.pos);
			i ? e.deleteRange(c, l) : (e.replaceRangeWith(c, l, t), il(e, n, t.isInline ? -1 : 1));
		}
	}
	static findFrom(e, t, n = !1) {
		let r = e.parent.inlineContent ? new R(e) : rl(e.node(0), e.parent, e.pos, e.index(), t, n);
		if (r) return r;
		for (let r = e.depth - 1; r >= 0; r--) {
			let i = t < 0 ? rl(e.node(0), e.node(r), e.before(r + 1), e.index(r), t, n) : rl(e.node(0), e.node(r), e.after(r + 1), e.index(r) + 1, t, n);
			if (i) return i;
		}
		return null;
	}
	static near(e, t = 1) {
		return this.findFrom(e, t) || this.findFrom(e, -t) || new tl(e.node(0));
	}
	static atStart(e) {
		return rl(e, e, 0, 0, 1) || new tl(e);
	}
	static atEnd(e) {
		return rl(e, e, e.content.size, e.childCount, -1) || new tl(e);
	}
	static fromJSON(e, t) {
		if (!t || !t.type) throw RangeError("Invalid input for Selection.fromJSON");
		let n = Yc[t.type];
		if (!n) throw RangeError(`No selection type ${t.type} defined`);
		return n.fromJSON(e, t);
	}
	static jsonID(e, t) {
		if (e in Yc) throw RangeError("Duplicate use of selection JSON ID " + e);
		return Yc[e] = t, t.prototype.jsonID = e, t;
	}
	getBookmark() {
		return R.between(this.$anchor, this.$head).getBookmark();
	}
};
L.prototype.visible = !0;
var Xc = class {
	constructor(e, t) {
		this.$from = e, this.$to = t;
	}
}, Zc = !1;
function Qc(e) {
	!Zc && !e.parent.inlineContent && (Zc = !0, console.warn("TextSelection endpoint not pointing into a node with inline content (" + e.parent.type.name + ")"));
}
var R = class e extends L {
	constructor(e, t = e) {
		Qc(e), Qc(t), super(e, t);
	}
	get $cursor() {
		return this.$anchor.pos == this.$head.pos ? this.$head : null;
	}
	map(t, n) {
		let r = t.resolve(n.map(this.head));
		if (!r.parent.inlineContent) return L.near(r);
		let i = t.resolve(n.map(this.anchor));
		return new e(i.parent.inlineContent ? i : r, r);
	}
	replace(e, t = F.empty) {
		if (super.replace(e, t), t == F.empty) {
			let t = this.$from.marksAcross(this.$to);
			t && e.ensureMarks(t);
		}
	}
	eq(t) {
		return t instanceof e && t.anchor == this.anchor && t.head == this.head;
	}
	getBookmark() {
		return new $c(this.anchor, this.head);
	}
	toJSON() {
		return {
			type: "text",
			anchor: this.anchor,
			head: this.head
		};
	}
	static fromJSON(t, n) {
		if (typeof n.anchor != "number" || typeof n.head != "number") throw RangeError("Invalid input for TextSelection.fromJSON");
		return new e(t.resolve(n.anchor), t.resolve(n.head));
	}
	static create(e, t, n = t) {
		let r = e.resolve(t);
		return new this(r, n == t ? r : e.resolve(n));
	}
	static between(t, n, r) {
		let i = t.pos - n.pos;
		if ((!r || i) && (r = i >= 0 ? 1 : -1), !n.parent.inlineContent) {
			let e = L.findFrom(n, r, !0) || L.findFrom(n, -r, !0);
			if (e) n = e.$head;
			else return L.near(n, r);
		}
		return t.parent.inlineContent || (i == 0 ? t = n : (t = (L.findFrom(t, -r, !0) || L.findFrom(t, r, !0)).$anchor, t.pos < n.pos != i < 0 && (t = n))), new e(t, n);
	}
};
L.jsonID("text", R);
var $c = class e {
	constructor(e, t) {
		this.anchor = e, this.head = t;
	}
	map(t) {
		return new e(t.map(this.anchor), t.map(this.head));
	}
	resolve(e) {
		return R.between(e.resolve(this.anchor), e.resolve(this.head));
	}
}, z = class e extends L {
	constructor(e) {
		let t = e.nodeAfter, n = e.node(0).resolve(e.pos + t.nodeSize);
		super(e, n), this.node = t;
	}
	map(t, n) {
		let { deleted: r, pos: i } = n.mapResult(this.anchor), a = t.resolve(i);
		return r ? L.near(a) : new e(a);
	}
	content() {
		return new F(N.from(this.node), 0, 0);
	}
	eq(t) {
		return t instanceof e && t.anchor == this.anchor;
	}
	toJSON() {
		return {
			type: "node",
			anchor: this.anchor
		};
	}
	getBookmark() {
		return new el(this.anchor);
	}
	static fromJSON(t, n) {
		if (typeof n.anchor != "number") throw RangeError("Invalid input for NodeSelection.fromJSON");
		return new e(t.resolve(n.anchor));
	}
	static create(t, n) {
		return new e(t.resolve(n));
	}
	static isSelectable(e) {
		return !e.isText && e.type.spec.selectable !== !1;
	}
};
z.prototype.visible = !1, L.jsonID("node", z);
var el = class e {
	constructor(e) {
		this.anchor = e;
	}
	map(t) {
		let { deleted: n, pos: r } = t.mapResult(this.anchor);
		return n ? new $c(r, r) : new e(r);
	}
	resolve(e) {
		let t = e.resolve(this.anchor), n = t.nodeAfter;
		return n && z.isSelectable(n) ? new z(t) : L.near(t);
	}
}, tl = class e extends L {
	constructor(e) {
		super(e.resolve(0), e.resolve(e.content.size));
	}
	replace(e, t = F.empty) {
		if (t == F.empty) {
			e.delete(0, e.doc.content.size);
			let t = L.atStart(e.doc);
			t.eq(e.selection) || e.setSelection(t);
		} else super.replace(e, t);
	}
	toJSON() {
		return { type: "all" };
	}
	static fromJSON(t) {
		return new e(t);
	}
	map(t) {
		return new e(t);
	}
	eq(t) {
		return t instanceof e;
	}
	getBookmark() {
		return nl;
	}
};
L.jsonID("all", tl);
var nl = {
	map() {
		return this;
	},
	resolve(e) {
		return new tl(e);
	}
};
function rl(e, t, n, r, i, a = !1) {
	if (t.inlineContent) return R.create(e, n);
	for (let o = r - (i > 0 ? 0 : 1); i > 0 ? o < t.childCount : o >= 0; o += i) {
		let r = t.child(o);
		if (!r.isAtom) {
			let t = rl(e, r, n + i, i < 0 ? r.childCount : 0, i, a);
			if (t) return t;
		} else if (!a && z.isSelectable(r)) return z.create(e, n - (i < 0 ? r.nodeSize : 0));
		n += r.nodeSize * i;
	}
	return null;
}
function il(e, t, n) {
	let r = e.steps.length - 1;
	if (r < t) return;
	let i = e.steps[r];
	if (!(i instanceof nc || i instanceof rc)) return;
	let a = e.mapping.maps[r], o;
	a.forEach((e, t, n, r) => {
		o ??= r;
	}), e.setSelection(L.near(e.doc.resolve(o), n));
}
var al = 1, ol = 2, sl = 4, cl = class extends Jc {
	constructor(e) {
		super(e.doc), this.curSelectionFor = 0, this.updated = 0, this.meta = Object.create(null), this.time = Date.now(), this.curSelection = e.selection, this.storedMarks = e.storedMarks;
	}
	get selection() {
		return this.curSelectionFor < this.steps.length && (this.curSelection = this.curSelection.map(this.doc, this.mapping.slice(this.curSelectionFor)), this.curSelectionFor = this.steps.length), this.curSelection;
	}
	setSelection(e) {
		if (e.$from.doc != this.doc) throw RangeError("Selection passed to setSelection must point at the current document");
		return this.curSelection = e, this.curSelectionFor = this.steps.length, this.updated = (this.updated | al) & -3, this.storedMarks = null, this;
	}
	get selectionSet() {
		return (this.updated & al) > 0;
	}
	setStoredMarks(e) {
		return this.storedMarks = e, this.updated |= ol, this;
	}
	ensureMarks(e) {
		return P.sameSet(this.storedMarks || this.selection.$from.marks(), e) || this.setStoredMarks(e), this;
	}
	addStoredMark(e) {
		return this.ensureMarks(e.addToSet(this.storedMarks || this.selection.$head.marks()));
	}
	removeStoredMark(e) {
		return this.ensureMarks(e.removeFromSet(this.storedMarks || this.selection.$head.marks()));
	}
	get storedMarksSet() {
		return (this.updated & ol) > 0;
	}
	addStep(e, t) {
		super.addStep(e, t), this.updated &= -3, this.storedMarks = null;
	}
	setTime(e) {
		return this.time = e, this;
	}
	replaceSelection(e) {
		return this.selection.replace(this, e), this;
	}
	replaceSelectionWith(e, t = !0) {
		let n = this.selection;
		return t && (e = e.mark(this.storedMarks || (n.empty ? n.$from.marks() : n.$from.marksAcross(n.$to) || P.none))), n.replaceWith(this, e), this;
	}
	deleteSelection() {
		return this.selection.replace(this), this;
	}
	insertText(e, t, n) {
		let r = this.doc.type.schema;
		if (t == null) return e ? this.replaceSelectionWith(r.text(e), !0) : this.deleteSelection();
		{
			if (n ??= t, !e) return this.deleteRange(t, n);
			let i = this.storedMarks;
			if (!i) {
				let e = this.doc.resolve(t);
				i = n == t ? e.marks() : e.marksAcross(this.doc.resolve(n));
			}
			return this.replaceRangeWith(t, n, r.text(e, i)), !this.selection.empty && this.selection.to == t + e.length && this.setSelection(L.near(this.selection.$to)), this;
		}
	}
	setMeta(e, t) {
		return this.meta[typeof e == "string" ? e : e.key] = t, this;
	}
	getMeta(e) {
		return this.meta[typeof e == "string" ? e : e.key];
	}
	get isGeneric() {
		for (let e in this.meta) return !1;
		return !0;
	}
	scrollIntoView() {
		return this.updated |= sl, this;
	}
	get scrolledIntoView() {
		return (this.updated & sl) > 0;
	}
};
function ll(e, t) {
	return !t || !e ? e : e.bind(t);
}
var ul = class {
	constructor(e, t, n) {
		this.name = e, this.init = ll(t.init, n), this.apply = ll(t.apply, n);
	}
}, dl = [
	new ul("doc", {
		init(e) {
			return e.doc || e.schema.topNodeType.createAndFill();
		},
		apply(e) {
			return e.doc;
		}
	}),
	new ul("selection", {
		init(e, t) {
			return e.selection || L.atStart(t.doc);
		},
		apply(e) {
			return e.selection;
		}
	}),
	new ul("storedMarks", {
		init(e) {
			return e.storedMarks || null;
		},
		apply(e, t, n, r) {
			return r.selection.$cursor ? e.storedMarks : null;
		}
	}),
	new ul("scrollToSelection", {
		init() {
			return 0;
		},
		apply(e, t) {
			return e.scrolledIntoView ? t + 1 : t;
		}
	})
], fl = class {
	constructor(e, t) {
		this.schema = e, this.plugins = [], this.pluginsByKey = Object.create(null), this.fields = dl.slice(), t && t.forEach((e) => {
			if (this.pluginsByKey[e.key]) throw RangeError("Adding different instances of a keyed plugin (" + e.key + ")");
			this.plugins.push(e), this.pluginsByKey[e.key] = e, e.spec.state && this.fields.push(new ul(e.key, e.spec.state, e));
		});
	}
}, pl = class e {
	constructor(e) {
		this.config = e;
	}
	get schema() {
		return this.config.schema;
	}
	get plugins() {
		return this.config.plugins;
	}
	apply(e) {
		return this.applyTransaction(e).state;
	}
	filterTransaction(e, t = -1) {
		for (let n = 0; n < this.config.plugins.length; n++) if (n != t) {
			let t = this.config.plugins[n];
			if (t.spec.filterTransaction && !t.spec.filterTransaction.call(t, e, this)) return !1;
		}
		return !0;
	}
	applyTransaction(e) {
		if (!this.filterTransaction(e)) return {
			state: this,
			transactions: []
		};
		let t = [e], n = this.applyInner(e), r = null;
		for (;;) {
			let i = !1;
			for (let a = 0; a < this.config.plugins.length; a++) {
				let o = this.config.plugins[a];
				if (o.spec.appendTransaction) {
					let s = r ? r[a].n : 0, c = r ? r[a].state : this, l = s < t.length && o.spec.appendTransaction.call(o, s ? t.slice(s) : t, c, n);
					if (l && n.filterTransaction(l, a)) {
						if (l.setMeta("appendedTransaction", e), !r) {
							r = [];
							for (let e = 0; e < this.config.plugins.length; e++) r.push(e < a ? {
								state: n,
								n: t.length
							} : {
								state: this,
								n: 0
							});
						}
						t.push(l), n = n.applyInner(l), i = !0;
					}
					r && (r[a] = {
						state: n,
						n: t.length
					});
				}
			}
			if (!i) return {
				state: n,
				transactions: t
			};
		}
	}
	applyInner(t) {
		if (!t.before.eq(this.doc)) throw RangeError("Applying a mismatched transaction");
		let n = new e(this.config), r = this.config.fields;
		for (let e = 0; e < r.length; e++) {
			let i = r[e];
			n[i.name] = i.apply(t, this[i.name], this, n);
		}
		return n;
	}
	get tr() {
		return new cl(this);
	}
	static create(t) {
		let n = new fl(t.doc ? t.doc.type.schema : t.schema, t.plugins), r = new e(n);
		for (let e = 0; e < n.fields.length; e++) r[n.fields[e].name] = n.fields[e].init(t, r);
		return r;
	}
	reconfigure(t) {
		let n = new fl(this.schema, t.plugins), r = n.fields, i = new e(n);
		for (let e = 0; e < r.length; e++) {
			let n = r[e].name;
			i[n] = this.hasOwnProperty(n) ? this[n] : r[e].init(t, i);
		}
		return i;
	}
	toJSON(e) {
		let t = {
			doc: this.doc.toJSON(),
			selection: this.selection.toJSON()
		};
		if (this.storedMarks && (t.storedMarks = this.storedMarks.map((e) => e.toJSON())), e && typeof e == "object") for (let n in e) {
			if (n == "doc" || n == "selection") throw RangeError("The JSON fields `doc` and `selection` are reserved");
			let r = e[n], i = r.spec.state;
			i && i.toJSON && (t[n] = i.toJSON.call(r, this[r.key]));
		}
		return t;
	}
	static fromJSON(t, n, r) {
		if (!n) throw RangeError("Invalid input for EditorState.fromJSON");
		if (!t.schema) throw RangeError("Required config field 'schema' missing");
		let i = new fl(t.schema, t.plugins), a = new e(i);
		return i.fields.forEach((e) => {
			if (e.name == "doc") a.doc = Bo.fromJSON(t.schema, n.doc);
			else if (e.name == "selection") a.selection = L.fromJSON(a.doc, n.selection);
			else if (e.name == "storedMarks") n.storedMarks && (a.storedMarks = n.storedMarks.map(t.schema.markFromJSON));
			else {
				if (r) for (let i in r) {
					let o = r[i], s = o.spec.state;
					if (o.key == e.name && s && s.fromJSON && Object.prototype.hasOwnProperty.call(n, i)) {
						a[e.name] = s.fromJSON.call(o, t, n[i], a);
						return;
					}
				}
				a[e.name] = e.init(t, a);
			}
		}), a;
	}
};
function ml(e, t, n) {
	for (let r in e) {
		let i = e[r];
		i instanceof Function ? i = i.bind(t) : r == "handleDOMEvents" && (i = ml(i, t, {})), n[r] = i;
	}
	return n;
}
var hl = class {
	constructor(e) {
		this.spec = e, this.props = {}, e.props && ml(e.props, this, this.props), this.key = e.key ? e.key.key : _l("plugin");
	}
	getState(e) {
		return e[this.key];
	}
}, gl = Object.create(null);
function _l(e) {
	return e in gl ? e + "$" + ++gl[e] : (gl[e] = 0, e + "$");
}
var vl = class {
	constructor(e = "key") {
		this.key = _l(e);
	}
	get(e) {
		return e.config.pluginsByKey[this.key];
	}
	getState(e) {
		return e[this.key];
	}
}, yl = (e, t) => !e.selection.empty && (t && t(e.tr.deleteSelection().scrollIntoView()), !0);
function bl(e, t) {
	let { $cursor: n } = e.selection;
	return !n || (t ? !t.endOfTextblock("backward", e) : n.parentOffset > 0) ? null : n;
}
var xl = (e, t, n) => {
	let r = bl(e, n);
	if (!r) return !1;
	let i = El(r);
	if (!i) {
		let n = r.blockRange(), i = n && lc(n);
		return i != null && (t && t(e.tr.lift(n, i).scrollIntoView()), !0);
	}
	let a = i.nodeBefore;
	if (Bl(e, i, t, -1)) return !0;
	if (r.parent.content.size == 0 && (wl(a, "end") || z.isSelectable(a))) for (let n = r.depth;; n--) {
		let o = kc(e.doc, r.before(n), r.after(n), F.empty);
		if (o && o.slice.size < o.to - o.from) {
			if (t) {
				let n = e.tr.step(o);
				n.setSelection(wl(a, "end") ? L.findFrom(n.doc.resolve(n.mapping.map(i.pos, -1)), -1) : z.create(n.doc, i.pos - a.nodeSize)), t(n.scrollIntoView());
			}
			return !0;
		}
		if (n == 1 || r.node(n - 1).childCount > 1) break;
	}
	return a.isAtom && i.depth == r.depth - 1 ? (t && t(e.tr.delete(i.pos - a.nodeSize, i.pos).scrollIntoView()), !0) : !1;
}, Sl = (e, t, n) => {
	let r = bl(e, n);
	if (!r) return !1;
	let i = El(r);
	return i ? Cl(e, i, t) : !1;
};
function Cl(e, t, n) {
	let r = t.nodeBefore, i = t.pos - 1;
	for (; !r.isTextblock; i--) {
		if (r.type.spec.isolating) return !1;
		let e = r.lastChild;
		if (!e) return !1;
		r = e;
	}
	let a = t.nodeAfter, o = t.pos + 1;
	for (; !a.isTextblock; o++) {
		if (a.type.spec.isolating) return !1;
		let e = a.firstChild;
		if (!e) return !1;
		a = e;
	}
	let s = kc(e.doc, i, o, F.empty);
	if (!s || s.from != i || s instanceof nc && s.slice.size >= o - i) return !1;
	if (n) {
		let t = e.tr.step(s);
		t.setSelection(R.create(t.doc, i)), n(t.scrollIntoView());
	}
	return !0;
}
function wl(e, t, n = !1) {
	for (let r = e; r; r = t == "start" ? r.firstChild : r.lastChild) {
		if (r.isTextblock) return !0;
		if (n && r.childCount != 1) return !1;
	}
	return !1;
}
var Tl = (e, t, n) => {
	let { $head: r, empty: i } = e.selection, a = r;
	if (!i) return !1;
	if (r.parent.isTextblock) {
		if (n ? !n.endOfTextblock("backward", e) : r.parentOffset > 0) return !1;
		a = El(r);
	}
	let o = a && a.nodeBefore;
	return !o || !z.isSelectable(o) ? !1 : (t && t(e.tr.setSelection(z.create(e.doc, a.pos - o.nodeSize)).scrollIntoView()), !0);
};
function El(e) {
	if (!e.parent.type.spec.isolating) for (let t = e.depth - 1; t >= 0; t--) {
		if (e.index(t) > 0) return e.doc.resolve(e.before(t + 1));
		if (e.node(t).type.spec.isolating) break;
	}
	return null;
}
function Dl(e, t) {
	let { $cursor: n } = e.selection;
	return !n || (t ? !t.endOfTextblock("forward", e) : n.parentOffset < n.parent.content.size) ? null : n;
}
var Ol = (e, t, n) => {
	let r = Dl(e, n);
	if (!r) return !1;
	let i = Al(r);
	if (!i) return !1;
	let a = i.nodeAfter;
	if (Bl(e, i, t, 1)) return !0;
	if (r.parent.content.size == 0 && (wl(a, "start") || z.isSelectable(a))) {
		let n = kc(e.doc, r.before(), r.after(), F.empty);
		if (n && n.slice.size < n.to - n.from) {
			if (t) {
				let r = e.tr.step(n);
				r.setSelection(wl(a, "start") ? L.findFrom(r.doc.resolve(r.mapping.map(i.pos)), 1) : z.create(r.doc, r.mapping.map(i.pos))), t(r.scrollIntoView());
			}
			return !0;
		}
	}
	return a.isAtom && i.depth == r.depth - 1 ? (t && t(e.tr.delete(i.pos, i.pos + a.nodeSize).scrollIntoView()), !0) : !1;
}, kl = (e, t, n) => {
	let { $head: r, empty: i } = e.selection, a = r;
	if (!i) return !1;
	if (r.parent.isTextblock) {
		if (n ? !n.endOfTextblock("forward", e) : r.parentOffset < r.parent.content.size) return !1;
		a = Al(r);
	}
	let o = a && a.nodeAfter;
	return !o || !z.isSelectable(o) ? !1 : (t && t(e.tr.setSelection(z.create(e.doc, a.pos)).scrollIntoView()), !0);
};
function Al(e) {
	if (!e.parent.type.spec.isolating) for (let t = e.depth - 1; t >= 0; t--) {
		let n = e.node(t);
		if (e.index(t) + 1 < n.childCount) return e.doc.resolve(e.after(t + 1));
		if (n.type.spec.isolating) break;
	}
	return null;
}
var jl = (e, t) => {
	let { $head: n, $anchor: r } = e.selection;
	return !n.parent.type.spec.code || !n.sameParent(r) ? !1 : (t && t(e.tr.insertText("\n").scrollIntoView()), !0);
};
function Ml(e) {
	for (let t = 0; t < e.edgeCount; t++) {
		let { type: n } = e.edge(t);
		if (n.isTextblock && !n.hasRequiredAttrs()) return n;
	}
	return null;
}
var Nl = (e, t) => {
	let { $head: n, $anchor: r } = e.selection;
	if (!n.parent.type.spec.code || !n.sameParent(r)) return !1;
	let i = n.node(-1), a = n.indexAfter(-1), o = Ml(i.contentMatchAt(a));
	if (!o || !i.canReplaceWith(a, a, o)) return !1;
	if (t) {
		let r = n.after(), i = e.tr.replaceWith(r, r, o.createAndFill());
		i.setSelection(L.near(i.doc.resolve(r), 1)), t(i.scrollIntoView());
	}
	return !0;
}, Pl = (e, t) => {
	let n = e.selection, { $from: r, $to: i } = n;
	if (n instanceof tl || r.parent.inlineContent || i.parent.inlineContent) return !1;
	let a = Ml(i.parent.contentMatchAt(i.indexAfter()));
	if (!a || !a.isTextblock) return !1;
	if (t) {
		let n = (!r.parentOffset && i.index() < i.parent.childCount ? r : i).pos, o = e.tr.insert(n, a.createAndFill());
		o.setSelection(R.create(o.doc, n + 1)), t(o.scrollIntoView());
	}
	return !0;
}, Fl = (e, t) => {
	let { $cursor: n } = e.selection;
	if (!n || n.parent.content.size) return !1;
	if (n.depth > 1 && n.after() != n.end(-1)) {
		let r = n.before();
		if (xc(e.doc, r)) return t && t(e.tr.split(r).scrollIntoView()), !0;
	}
	let r = n.blockRange(), i = r && lc(r);
	return i != null && (t && t(e.tr.lift(r, i).scrollIntoView()), !0);
};
function Il(e) {
	return (t, n) => {
		if (t.selection instanceof z && t.selection.node.isBlock) {
			let { $from: e } = t.selection;
			return !e.parentOffset || !xc(t.doc, e.pos) ? !1 : (n && n(t.tr.split(e.pos).scrollIntoView()), !0);
		}
		if (!t.selection.$from.depth) return !1;
		let r = t.tr;
		!t.selection.empty && (t.selection instanceof R || t.selection instanceof tl) && r.deleteSelection();
		let { $from: i } = r.selection, a = r.steps.length, o = [], s, c, l = !1, u = !1;
		for (let t = i.depth;; t--) if (i.node(t).isBlock) {
			l = i.end(t) == i.pos + (i.depth - t), u = i.start(t) == i.pos - (i.depth - t), c = Ml(i.node(t - 1).contentMatchAt(i.indexAfter(t - 1)));
			let n = e && e(i.parent, l, i);
			o.unshift(n || (l && c ? { type: c } : null)), s = t;
			break;
		} else {
			if (t == 1) return !1;
			o.unshift(null);
		}
		let d = i.pos, f = xc(r.doc, d, o.length, o);
		if (f ||= (o[0] = c ? { type: c } : null, xc(r.doc, d, o.length, o)), !f) return !1;
		if (r.split(d, o.length, o), !l && u && i.node(s).type != c) {
			let e = r.mapping.slice(a), t = e.map(i.before(s)), n = r.doc.resolve(t);
			c && i.node(s - 1).canReplaceWith(n.index(), n.index() + 1, c) && r.setNodeMarkup(e.map(i.before(s)), c);
		}
		return n && n(r.scrollIntoView()), !0;
	};
}
var Ll = Il(), Rl = (e, t) => (t && t(e.tr.setSelection(new tl(e.doc))), !0);
function zl(e, t, n) {
	let r = t.nodeBefore, i = t.nodeAfter, a = t.index();
	return !r || !i || !r.type.compatibleContent(i.type) ? !1 : !r.content.size && t.parent.canReplace(a - 1, a) ? (n && n(e.tr.delete(t.pos - r.nodeSize, t.pos).scrollIntoView()), !0) : !t.parent.canReplace(a, a + 1) || !(i.isTextblock || Cc(e.doc, t.pos)) ? !1 : (n && n(e.tr.join(t.pos).scrollIntoView()), !0);
}
function Bl(e, t, n, r) {
	let i = t.nodeBefore, a = t.nodeAfter, o, s, c = i.type.spec.isolating || a.type.spec.isolating;
	if (!c && zl(e, t, n)) return !0;
	let l = !c && t.parent.canReplace(t.index(), t.index() + 1);
	if (l && (o = (s = i.contentMatchAt(i.childCount)).findWrapping(a.type)) && s.matchType(o[0] || a.type).validEnd) {
		if (n) {
			let r = t.pos + a.nodeSize, s = N.empty;
			for (let e = o.length - 1; e >= 0; e--) s = N.from(o[e].create(null, s));
			s = N.from(i.copy(s));
			let c = e.tr.step(new rc(t.pos - 1, r, t.pos, r, new F(s, 1, 0), o.length, !0)), l = c.doc.resolve(r + 2 * o.length);
			l.nodeAfter && l.nodeAfter.type == i.type && Cc(c.doc, l.pos) && c.join(l.pos), n(c.scrollIntoView());
		}
		return !0;
	}
	let u = a.type.spec.isolating || r > 0 && c ? null : L.findFrom(t, 1), d = u && u.$from.blockRange(u.$to), f = d && lc(d);
	if (f != null && f >= t.depth) return n && n(e.tr.lift(d, f).scrollIntoView()), !0;
	if (l && wl(a, "start", !0) && wl(i, "end")) {
		let r = i, o = [];
		for (; o.push(r), !r.isTextblock;) r = r.lastChild;
		let s = a, c = 1;
		for (; !s.isTextblock; s = s.firstChild) c++;
		if (r.canReplace(r.childCount, r.childCount, s.content)) {
			if (n) {
				let r = N.empty;
				for (let e = o.length - 1; e >= 0; e--) r = N.from(o[e].copy(r));
				n(e.tr.step(new rc(t.pos - o.length, t.pos + a.nodeSize, t.pos + c, t.pos + a.nodeSize - c, new F(r, o.length, 0), 0, !0)).scrollIntoView());
			}
			return !0;
		}
	}
	return !1;
}
function Vl(e) {
	return function(t, n) {
		let r = t.selection, i = e < 0 ? r.$from : r.$to, a = i.depth;
		for (; i.node(a).isInline;) {
			if (!a) return !1;
			a--;
		}
		return i.node(a).isTextblock ? (n && n(t.tr.setSelection(R.create(t.doc, e < 0 ? i.start(a) : i.end(a)))), !0) : !1;
	};
}
var Hl = Vl(-1), Ul = Vl(1);
function Wl(e, t = null) {
	return function(n, r) {
		let { $from: i, $to: a } = n.selection, o = i.blockRange(a), s = o && dc(o, e, t);
		return s ? (r && r(n.tr.wrap(o, s).scrollIntoView()), !0) : !1;
	};
}
function Gl(e, t = null) {
	return function(n, r) {
		let i = !1;
		for (let r = 0; r < n.selection.ranges.length && !i; r++) {
			let { $from: { pos: a }, $to: { pos: o } } = n.selection.ranges[r];
			n.doc.nodesBetween(a, o, (r, a) => {
				if (i) return !1;
				if (r.isTextblock && !r.hasMarkup(e, t)) {
					if (r.type == e) i = !0;
					else {
						let t = n.doc.resolve(a), r = t.index();
						i = t.parent.canReplaceWith(r, r + 1, e);
					}
				}
			});
		}
		if (!i) return !1;
		if (r) {
			let i = n.tr;
			for (let r = 0; r < n.selection.ranges.length; r++) {
				let { $from: { pos: a }, $to: { pos: o } } = n.selection.ranges[r];
				i.setBlockType(a, o, e, t);
			}
			r(i.scrollIntoView());
		}
		return !0;
	};
}
function Kl(e, t, n, r) {
	for (let i = 0; i < t.length; i++) {
		let { $from: a, $to: o } = t[i], s = a.depth == 0 && e.inlineContent && e.type.allowsMarkType(n);
		if (e.nodesBetween(a.pos, o.pos, (e, t) => {
			if (s || !r && e.isAtom && e.isInline && t >= a.pos && t + e.nodeSize <= o.pos) return !1;
			s = e.inlineContent && e.type.allowsMarkType(n);
		}), s) return !0;
	}
	return !1;
}
function ql(e) {
	let t = [];
	for (let n = 0; n < e.length; n++) {
		let { $from: r, $to: i } = e[n];
		r.doc.nodesBetween(r.pos, i.pos, (e, n) => {
			if (e.isAtom && e.content.size && e.isInline && n >= r.pos && n + e.nodeSize <= i.pos) return n + 1 > r.pos && t.push(new Xc(r, r.doc.resolve(n + 1))), r = r.doc.resolve(n + 1 + e.content.size), !1;
		}), r.pos < i.pos && t.push(new Xc(r, i));
	}
	return t;
}
function Jl(e, t = null, n) {
	let r = (n && n.removeWhenPresent) !== !1, i = (n && n.enterInlineAtoms) !== !1, a = !(n && n.includeWhitespace);
	return function(n, o) {
		let { empty: s, $cursor: c, ranges: l } = n.selection;
		if (s && !c || !Kl(n.doc, l, e, i)) return !1;
		if (o) {
			if (c) e.isInSet(n.storedMarks || c.marks()) ? o(n.tr.removeStoredMark(e)) : o(n.tr.addStoredMark(e.create(t)));
			else {
				let s, c = n.tr;
				i || (l = ql(l)), s = r ? !l.some((t) => n.doc.rangeHasMark(t.$from.pos, t.$to.pos, e)) : !l.every((t) => {
					let n = !1;
					return c.doc.nodesBetween(t.$from.pos, t.$to.pos, (r, i, a) => {
						if (n) return !1;
						n = !e.isInSet(r.marks) && !!a && a.type.allowsMarkType(e) && !(r.isText && /^\s*$/.test(r.textBetween(Math.max(0, t.$from.pos - i), Math.min(r.nodeSize, t.$to.pos - i))));
					}), !n;
				});
				for (let n = 0; n < l.length; n++) {
					let { $from: r, $to: i } = l[n];
					if (!s) c.removeMark(r.pos, i.pos, e);
					else {
						let n = r.pos, o = i.pos, s = r.nodeAfter, l = i.nodeBefore, u = a && s && s.isText ? /^\s*/.exec(s.text)[0].length : 0, d = a && l && l.isText ? /\s*$/.exec(l.text)[0].length : 0;
						n + u < o && (n += u, o -= d), c.addMark(n, o, e.create(t));
					}
				}
				o(c.scrollIntoView());
			}
		}
		return !0;
	};
}
function Yl(...e) {
	return function(t, n, r) {
		for (let i = 0; i < e.length; i++) if (e[i](t, n, r)) return !0;
		return !1;
	};
}
var Xl = Yl(yl, xl, Tl), Zl = Yl(yl, Ol, kl), Ql = {
	Enter: Yl(jl, Pl, Fl, Ll),
	"Mod-Enter": Nl,
	Backspace: Xl,
	"Mod-Backspace": Xl,
	"Shift-Backspace": Xl,
	Delete: Zl,
	"Mod-Delete": Zl,
	"Mod-a": Rl
}, $l = {
	"Ctrl-h": Ql.Backspace,
	"Alt-Backspace": Ql["Mod-Backspace"],
	"Ctrl-d": Ql.Delete,
	"Ctrl-Alt-Backspace": Ql["Mod-Delete"],
	"Alt-Delete": Ql["Mod-Delete"],
	"Alt-d": Ql["Mod-Delete"],
	"Ctrl-a": Hl,
	"Ctrl-e": Ul
};
for (let e in Ql) $l[e] = Ql[e];
var eu = (typeof navigator < "u" ? /Mac|iP(hone|[oa]d)/.test(navigator.platform) : typeof os < "u" && os.platform && os.platform() == "darwin") ? $l : Ql, tu = class {
	constructor(e, t, n = {}) {
		this.match = e, this.match = e, this.handler = typeof t == "string" ? nu(t) : t, this.undoable = n.undoable !== !1, this.inCode = n.inCode || !1, this.inCodeMark = n.inCodeMark !== !1;
	}
};
function nu(e) {
	return function(t, n, r, i) {
		let a = e;
		if (n[1]) {
			let e = n[0].lastIndexOf(n[1]);
			a += n[0].slice(e + n[1].length), r += e;
			let t = r - i;
			t > 0 && (a = n[0].slice(e - t, e) + a, r = i);
		}
		return t.tr.insertText(a, r, i);
	};
}
var ru = (e, t) => {
	let n = e.plugins;
	for (let r = 0; r < n.length; r++) {
		let i = n[r], a;
		if (i.spec.isInputRules && (a = i.getState(e))) {
			if (t) {
				let n = e.tr, r = a.transform;
				for (let e = r.steps.length - 1; e >= 0; e--) n.step(r.steps[e].invert(r.docs[e]));
				if (a.text) {
					let t = n.doc.resolve(a.from).marks();
					n.replaceWith(a.from, a.to, e.schema.text(a.text, t));
				} else n.delete(a.from, a.to);
				t(n);
			}
			return !0;
		}
	}
	return !1;
};
new tu(/--$/, "—", { inCodeMark: !1 }), new tu(/\.\.\.$/, "…", { inCodeMark: !1 }), new tu(/(?:^|[\s\{\[\(\<'"\u2018\u201C])(")$/, "“", { inCodeMark: !1 }), new tu(/"$/, "”", { inCodeMark: !1 }), new tu(/(?:^|[\s\{\[\(\<'"\u2018\u201C])(')$/, "‘", { inCodeMark: !1 }), new tu(/'$/, "’", { inCodeMark: !1 });
function iu(e, t, n = null, r) {
	return new tu(e, (e, i, a, o) => {
		let s = n instanceof Function ? n(i) : n, c = e.tr.delete(a, o), l = c.doc.resolve(a).blockRange(), u = l && dc(l, t, s);
		if (!u) return null;
		c.wrap(l, u);
		let d = c.doc.resolve(a - 1).nodeBefore;
		return d && d.type == t && Cc(c.doc, a - 1) && (!r || r(i, d)) && c.join(a - 1), c;
	});
}
function au(e, t, n = null) {
	return new tu(e, (e, r, i, a) => {
		let o = e.doc.resolve(i), s = n instanceof Function ? n(r) : n;
		return o.node(-1).canReplaceWith(o.index(-1), o.indexAfter(-1), t) ? e.tr.delete(i, a).setBlockType(i, i, t, s) : null;
	});
}
//#endregion
//#region node_modules/.pnpm/@milkdown+prose@7.22.2/node_modules/@milkdown/prose/lib/index.js
var ou = typeof navigator < "u" ? navigator : null, su = typeof document < "u" ? document : null, cu = ou && ou.userAgent || "", lu = /Edge\/(\d+)/.exec(cu), uu = /MSIE \d/.exec(cu), du = /Trident\/(?:[7-9]|\d{2,})\..*rv:(\d+)/.exec(cu), fu = !!(uu || du || lu);
uu ? document.documentMode : du ? +du[1] : lu && +lu[1], !fu && /gecko\/(\d+)/i.test(cu) && +(/Firefox\/(\d+)/.exec(cu) || [0, 0])[1];
var pu = !fu && /Chrome\/(\d+)/.exec(cu), mu = !!pu;
pu && +pu[1], !fu && ou && /Apple Computer/.test(ou.vendor) && (/Mobile\/\w+/.test(cu) || ou && ou.maxTouchPoints > 2) || ou && /Mac/.test(ou.platform);
var hu = /Android \d/.test(cu);
su && "webkitFontSmoothing" in su.documentElement.style && +(/\bAppleWebKit\/(\d+)/.exec(navigator.userAgent) || [0, 0])[1];
function gu(e, t, n, r, i, a) {
	if (e.composing) return !1;
	let o = e.state, s = o.doc.resolve(t);
	if (s.parent.type.spec.code) return !1;
	let c = s.parent.textBetween(Math.max(0, s.parentOffset - 500), s.parentOffset, void 0, "￼") + r;
	for (let s of i) {
		let i = s, l = i.match.exec(c), u = l && l[0] && i.handler(o, l, t - (l[0].length - r.length), n);
		if (u) return i.undoable !== !1 && u.setMeta(a, {
			transform: u,
			from: t,
			to: n,
			text: r
		}), e.dispatch(u), !0;
	}
	return !1;
}
var _u = new vl("MILKDOWN_CUSTOM_INPUTRULES");
function vu({ rules: e }) {
	let t = new hl({
		key: _u,
		isInputRules: !0,
		state: {
			init() {
				return null;
			},
			apply(e, t) {
				return e.getMeta(this) || (e.selectionSet || e.docChanged ? null : t);
			}
		},
		props: {
			handleTextInput(n, r, i, a) {
				return gu(n, r, i, a, e, t);
			},
			handleDOMEvents: {
				compositionend: (n) => (setTimeout(() => {
					let { $cursor: r } = n.state.selection;
					r && gu(n, r.pos, r.pos, "", e, t);
				}), !1),
				keydown: (e, t) => !(hu && mu && t.key === "Enter") || e.composing ? !1 : e.someProp("handleKeyDown", (n) => n(e, t)) ? (t.preventDefault(), !0) : !1
			},
			handleKeyDown(n, r) {
				if (r.key !== "Enter") return !1;
				let { $cursor: i } = n.state.selection;
				return i ? gu(n, i.pos, i.pos, "\n", e, t) : !1;
			}
		}
	});
	return t;
}
function yu(e) {
	return e.some((e) => e.type.spec.code);
}
function bu(e, t, n) {
	if (t >= n) return !1;
	let r = !1;
	return e.doc.nodesBetween(t, n, (e) => (e.isInline && yu(e.marks) && (r = !0), !r)), r;
}
function xu(e, t) {
	return yu(e.storedMarks ?? e.doc.resolve(t).marks());
}
function Su(e, t, n = {}) {
	return new tu(e, (e, r, i, a) => {
		var o;
		let { tr: s } = e, c = r[r.length - 1], l = r[0], u = [], d, f = {
			group: c,
			fullMatch: l,
			start: i,
			end: a
		}, p = n.updateCaptured?.call(n, f);
		if (Object.assign(f, p), {group: c, fullMatch: l, start: i, end: a} = f, l === null || c?.trim() === "") return null;
		if (c) {
			let f = l.search(/\S/), p = i + l.indexOf(c), m = p + c.length, h = bu(e, i + f, p), g = m < a && bu(e, m, a);
			if (h || g || xu(e, a)) return null;
			u = s.storedMarks ?? [], m < a && s.delete(m, a), p > i && s.delete(i + f, p), d = i + f + c.length;
			let _ = n.getAttr?.call(n, r);
			s.addMark(i, d, t.create(_)), s.setStoredMarks(u), (o = n.beforeDispatch) == null || o.call(n, {
				match: r,
				start: i,
				end: a,
				tr: s
			});
		}
		return s;
	});
}
function Cu(e) {
	return Object.assign(Object.create(e), e).setTime(Date.now());
}
function wu(e, t) {
	return Array.isArray(e) && e.includes(t.type) || t.type === e;
}
function Tu(e) {
	return (t) => {
		for (let n = t.depth; n > 0; --n) {
			let r = t.node(n);
			if (e(r)) return {
				from: t.before(n),
				to: t.after(n),
				node: r
			};
		}
	};
}
function Eu(e, t) {
	return Tu((e) => e.type === t)(e);
}
function Du(e) {
	return (t) => {
		for (let n = t.depth; n > 0; n--) {
			let r = t.node(n);
			if (e(r)) return {
				pos: t.before(n),
				start: t.start(n),
				depth: n,
				node: r
			};
		}
	};
}
function Ou(e, t) {
	if (!(e instanceof z)) return;
	let { node: n, $from: r } = e;
	if (wu(t, n)) return {
		node: n,
		pos: r.pos,
		start: r.start(r.depth),
		depth: r.depth
	};
}
for (var ku = (e, t) => {
	let { selection: n, doc: r } = e;
	if (n instanceof z) return {
		hasNode: n.node.type === t,
		pos: n.from,
		target: n.node
	};
	let { from: i, to: a } = n, o = !1, s = -1, c = null;
	return r.nodesBetween(i, a, (e, n) => c ? !1 : e.type !== t || (o = !0, s = n, c = e, !1)), {
		hasNode: o,
		pos: s,
		target: c
	};
}, Au = {
	8: "Backspace",
	9: "Tab",
	10: "Enter",
	12: "NumLock",
	13: "Enter",
	16: "Shift",
	17: "Control",
	18: "Alt",
	20: "CapsLock",
	27: "Escape",
	32: " ",
	33: "PageUp",
	34: "PageDown",
	35: "End",
	36: "Home",
	37: "ArrowLeft",
	38: "ArrowUp",
	39: "ArrowRight",
	40: "ArrowDown",
	44: "PrintScreen",
	45: "Insert",
	46: "Delete",
	59: ";",
	61: "=",
	91: "Meta",
	92: "Meta",
	106: "*",
	107: "+",
	108: ",",
	109: "-",
	110: ".",
	111: "/",
	144: "NumLock",
	145: "ScrollLock",
	160: "Shift",
	161: "Shift",
	162: "Control",
	163: "Control",
	164: "Alt",
	165: "Alt",
	173: "-",
	186: ";",
	187: "=",
	188: ",",
	189: "-",
	190: ".",
	191: "/",
	192: "`",
	219: "[",
	220: "\\",
	221: "]",
	222: "'"
}, ju = {
	48: ")",
	49: "!",
	50: "@",
	51: "#",
	52: "$",
	53: "%",
	54: "^",
	55: "&",
	56: "*",
	57: "(",
	59: ":",
	61: "+",
	173: "_",
	186: ":",
	187: "+",
	188: "<",
	189: "_",
	190: ">",
	191: "?",
	192: "~",
	219: "{",
	220: "|",
	221: "}",
	222: "\""
}, Mu = typeof navigator < "u" && /Mac/.test(navigator.platform), Nu = typeof navigator < "u" && /MSIE \d|Trident\/(?:[7-9]|\d{2,})\..*rv:(\d+)/.exec(navigator.userAgent), Pu = 0; Pu < 10; Pu++) Au[48 + Pu] = Au[96 + Pu] = String(Pu);
for (var Pu = 1; Pu <= 24; Pu++) Au[Pu + 111] = "F" + Pu;
for (var Pu = 65; Pu <= 90; Pu++) Au[Pu] = String.fromCharCode(Pu + 32), ju[Pu] = String.fromCharCode(Pu);
for (var Fu in Au) ju.hasOwnProperty(Fu) || (ju[Fu] = Au[Fu]);
function Iu(e) {
	var t = !(Mu && e.metaKey && e.shiftKey && !e.ctrlKey && !e.altKey || Nu && e.shiftKey && e.key && e.key.length == 1 || e.key == "Unidentified") && e.key || (e.shiftKey ? ju : Au)[e.keyCode] || e.key || "Unidentified";
	return t == "Esc" && (t = "Escape"), t == "Del" && (t = "Delete"), t == "Left" && (t = "ArrowLeft"), t == "Up" && (t = "ArrowUp"), t == "Right" && (t = "ArrowRight"), t == "Down" && (t = "ArrowDown"), t;
}
//#endregion
//#region node_modules/.pnpm/prosemirror-keymap@1.2.3/node_modules/prosemirror-keymap/dist/index.js
var Lu = typeof navigator < "u" && /Mac|iP(hone|[oa]d)/.test(navigator.platform), Ru = typeof navigator < "u" && /Win/.test(navigator.platform);
function zu(e) {
	let t = e.split(/-(?!$)/), n = t[t.length - 1];
	n == "Space" && (n = " ");
	let r, i, a, o;
	for (let e = 0; e < t.length - 1; e++) {
		let n = t[e];
		if (/^(cmd|meta|m)$/i.test(n)) o = !0;
		else if (/^a(lt)?$/i.test(n)) r = !0;
		else if (/^(c|ctrl|control)$/i.test(n)) i = !0;
		else if (/^s(hift)?$/i.test(n)) a = !0;
		else if (/^mod$/i.test(n)) Lu ? o = !0 : i = !0;
		else throw Error("Unrecognized modifier name: " + n);
	}
	return r && (n = "Alt-" + n), i && (n = "Ctrl-" + n), o && (n = "Meta-" + n), a && (n = "Shift-" + n), n;
}
function Bu(e) {
	let t = Object.create(null);
	for (let n in e) t[zu(n)] = e[n];
	return t;
}
function Vu(e, t, n = !0) {
	return t.altKey && (e = "Alt-" + e), t.ctrlKey && (e = "Ctrl-" + e), t.metaKey && (e = "Meta-" + e), n && t.shiftKey && (e = "Shift-" + e), e;
}
function Hu(e) {
	return new hl({ props: { handleKeyDown: Uu(e) } });
}
function Uu(e) {
	let t = Bu(e);
	return function(e, n) {
		let r = Iu(n), i, a = t[Vu(r, n)];
		if (a && a(e.state, e.dispatch, e)) return !0;
		if (r.length == 1 && r != " ") {
			if (n.shiftKey) {
				let i = t[Vu(r, n, !1)];
				if (i && i(e.state, e.dispatch, e)) return !0;
			}
			if ((n.altKey || n.metaKey || n.ctrlKey) && !(Ru && n.ctrlKey && n.altKey) && (i = Au[n.keyCode]) && i != r) {
				let r = t[Vu(i, n)];
				if (r && r(e.state, e.dispatch, e)) return !0;
			}
		}
		return !1;
	};
}
//#endregion
//#region node_modules/.pnpm/@milkdown+transformer@7.22.2/node_modules/@milkdown/transformer/lib/index.js
var Wu = class {}, Gu = class {
	constructor() {
		this.elements = [], this.size = () => this.elements.length, this.top = () => this.elements.at(-1), this.push = (e) => {
			this.top()?.push(e);
		}, this.open = (e) => {
			this.elements.push(e);
		}, this.close = () => {
			let e = this.elements.pop();
			if (!e) throw ue();
			return e;
		};
	}
}, Ku = class e extends Wu {
	constructor(e, t, n) {
		super(), this.type = e, this.content = t, this.attrs = n;
	}
	push(e, ...t) {
		this.content.push(e, ...t);
	}
	pop() {
		return this.content.pop();
	}
	static create(t, n, r) {
		return new e(t, n, r);
	}
}, qu = class extends Gu {
	#e;
	static {
		this.create = (e, t) => {
			let n = new this(e);
			return (e) => (n.run(t, e), n.toDoc());
		};
	}
	constructor(e) {
		super(), this.#e = P.none, this.#t = (e) => e.isText, this.#n = (e, t) => {
			if (this.#t(e) && this.#t(t) && P.sameSet(e.marks, t.marks)) return this.schema.text(e.text + t.text, e.marks);
		}, this.#r = (e) => {
			let t = Object.values({
				...this.schema.nodes,
				...this.schema.marks
			}).find((t) => t.spec.parseMarkdown.match(e));
			if (!t) throw de(e);
			return t;
		}, this.#i = (e) => {
			let t = this.#r(e);
			t.spec.parseMarkdown.runner(this, e, t);
		}, this.injectRoot = (e, t, n) => (this.openNode(t, n), this.next(e.children), this), this.openNode = (e, t) => (this.open(Ku.create(e, [], t)), this), this.#a = () => {
			this.#e = P.none;
			let e = this.close();
			return this.#o(e.type, e.attrs, e.content);
		}, this.closeNode = () => {
			try {
				this.#a();
			} catch (e) {
				console.error(e);
			}
			return this;
		}, this.#o = (e, t, n) => {
			let r = e.createAndFill(t, n, this.#e);
			if (!r) throw le(e, t, n);
			return this.push(r), r;
		}, this.addNode = (e, t, n) => {
			try {
				this.#o(e, t, n);
			} catch (e) {
				console.error(e);
			}
			return this;
		}, this.openMark = (e, t) => {
			let n = e.create(t);
			return this.#e = n.addToSet(this.#e), this;
		}, this.closeMark = (e) => (this.#e = e.removeFromSet(this.#e), this), this.addText = (e) => {
			try {
				let t = this.top();
				if (!t) throw ue();
				let n = t.pop(), r = this.schema.text(e, this.#e);
				if (!n) return t.push(r), this;
				let i = this.#n(n, r);
				return i ? (t.push(i), this) : (t.push(n, r), this);
			} catch (e) {
				return console.error(e), this;
			}
		}, this.build = () => {
			let e;
			do
				e = this.#a();
			while (this.size());
			return e;
		}, this.next = (e = []) => ([e].flat().forEach((e) => this.#i(e)), this), this.toDoc = () => this.build(), this.run = (e, t) => {
			let n = e.runSync(e.parse(t), t);
			return this.next(n), this;
		}, this.schema = e;
	}
	#t;
	#n;
	#r;
	#i;
	#a;
	#o;
}, Ju = class e extends Wu {
	constructor(e, t, n, r = {}) {
		super(), this.type = e, this.children = t, this.value = n, this.props = r, this.push = (e, ...t) => {
			this.children ||= [], this.children.push(e, ...t);
		}, this.pop = () => this.children?.pop();
	}
	static {
		this.create = (t, n, r, i = {}) => new e(t, n, r, i);
	}
}, Yu = (e) => Object.prototype.hasOwnProperty.call(e, "size"), Xu = class extends Gu {
	#e;
	static {
		this.create = (e, t) => {
			let n = new this(e);
			return (e) => (n.run(e), n.toString(t));
		};
	}
	constructor(e) {
		super(), this.#e = [], this.#t = (e) => {
			let t = Object.values({
				...this.schema.nodes,
				...this.schema.marks
			}).find((t) => t.spec.toMarkdown.match(e));
			if (!t) throw fe(e.type);
			return t;
		}, this.#n = (e) => this.#t(e).spec.toMarkdown.runner(this, e), this.#r = (e, t) => this.#t(e).spec.toMarkdown.runner(this, e, t), this.#i = (e) => {
			let t = (e) => e.type.spec.priority ?? 50, n = [...e].sort((e, n) => t(e) - t(n)), r = [];
			return this.#e.forEach(({ mark: e }) => {
				let t = n.findIndex((t) => t.eq(e));
				t >= 0 && r.push(...n.splice(t, 1));
			}), r.concat(n);
		}, this.#a = (e) => {
			let t = e?.marks, n = 0;
			for (; n < this.#e.length;) {
				let { mark: e, spanning: r } = this.#e[n];
				if (r && t?.some((t) => t.eq(e))) n++;
				else break;
			}
			for (let e = this.#e.length - 1; e >= n; e--) this.#p(this.#e[e].mark);
		}, this.#o = (e, t) => {
			this.#i(e.marks).every((t) => !this.#r(t, e)) && this.#n(e), this.#a(t);
		}, this.#s = (e) => {
			let { children: t } = e;
			return t && (e.children = t.reduce((e, t, n) => {
				if (n === 0) return [t];
				let r = e.at(-1);
				if (r && r.isMark && t.isMark) {
					let { children: n, ...i } = t, { children: a, ...o } = r;
					if (t.type === r.type && n && a && JSON.stringify(i) === JSON.stringify(o)) {
						let t = {
							...o,
							children: [...a, ...n]
						};
						return e.slice(0, -1).concat(this.#s(t));
					}
				}
				return e.concat(t);
			}, [])), e;
		}, this.#c = (e) => {
			let t = {
				...e.props,
				type: e.type
			};
			return e.children && (t.children = e.children), e.value && (t.value = e.value), t;
		}, this.openNode = (e, t, n) => (this.open(Ju.create(e, void 0, t, n)), this), this.#l = (e, t) => {
			let n = "", r = "", i = e.children;
			if (i) {
				let e = i[0], t = i.at(-1);
				if (t && t.type === "text" && t.value.endsWith(" ")) {
					let e = t.value, n = e.trimEnd();
					r = e.slice(n.length), t.value = n;
				}
				if (e && e.type === "text" && e.value.startsWith(" ")) {
					let t = e.value, r = t.trimStart();
					n = t.slice(0, t.length - r.length), e.value = r;
				}
			}
			n.length && this.#d("text", void 0, n);
			let a = t();
			return r.length && this.#d("text", void 0, r), a;
		}, this.#u = (e = !1) => {
			let t = this.close(), n = () => this.#d(t.type, t.children, t.value, t.props);
			return e ? this.#l(t, n) : n();
		}, this.closeNode = () => (this.#u(), this), this.#d = (e, t, n, r) => {
			let i = Ju.create(e, t, n, r), a = this.#s(this.#c(i));
			return this.push(a), a;
		}, this.addNode = (e, t, n, r) => (this.#d(e, t, n, r), this), this.#f = (e, t, n, r) => this.#e.some((t) => t.mark.eq(e)) ? this : (this.#e.push({
			mark: e,
			spanning: n == null
		}), this.openNode(t, n, {
			...r,
			isMark: !0
		})), this.#p = (e) => {
			let t = -1;
			for (let n = this.#e.length - 1; n >= 0; n--) if (this.#e[n].mark.eq(e)) {
				t = n;
				break;
			}
			t < 0 || (this.#e.splice(t, 1), this.#u(!0));
		}, this.withMark = (e, t, n, r) => (this.#f(e, t, n, r), this), this.closeMark = (e) => (this.#p(e), this), this.build = () => {
			let e = null;
			do
				e = this.#u();
			while (this.size());
			return e.children || (e.children = []), e;
		}, this.next = (e) => Yu(e) ? (e.forEach((t, n, r) => {
			this.#o(t, e.maybeChild(r + 1) ?? void 0);
		}), this) : (this.#o(e), this), this.toString = (e) => e.stringify(this.build()), this.run = (e) => (this.#e = [], this.next(e), this), this.schema = e;
	}
	#t;
	#n;
	#r;
	#i;
	#a;
	#o;
	#s;
	#c;
	#l;
	#u;
	#d;
	#f;
	#p;
}, B = function(e) {
	for (var t = 0;; t++) if (e = e.previousSibling, !e) return t;
}, Zu = function(e) {
	let t = e.assignedSlot || e.parentNode;
	return t && t.nodeType == 11 ? t.host : t;
}, Qu = null, $u = function(e, t, n) {
	let r = Qu ||= document.createRange();
	return r.setEnd(e, n ?? e.nodeValue.length), r.setStart(e, t || 0), r;
}, ed = function() {
	Qu = null;
}, td = function(e, t, n, r) {
	return n && (rd(e, t, n, r, -1) || rd(e, t, n, r, 1));
}, nd = /^(img|br|input|textarea|hr)$/i;
function rd(e, t, n, r, i) {
	for (;;) {
		if (e == n && t == r) return !0;
		if (t == (i < 0 ? 0 : V(e))) {
			let n = e.parentNode;
			if (!n || !H(n) || cd(e) || nd.test(e.nodeName) || e.contentEditable == "false") return !1;
			t = B(e) + (i < 0 ? 0 : 1), e = n;
		} else if (H(e)) {
			let n = e.childNodes[t + (i < 0 ? -1 : 0)];
			if (H(n) && n.contentEditable == "false") {
				if (n.pmViewDesc?.ignoreForSelection) t += i;
				else return !1;
			} else e = n, t = i < 0 ? V(e) : 0;
		} else return !1;
	}
}
function V(e) {
	return e.nodeType == 3 ? e.nodeValue.length : e.childNodes.length;
}
function H(e) {
	return e.nodeType == 1;
}
function U(e) {
	return e.nodeType == 3;
}
function id(e, t) {
	return t ? e.contains(H(t) ? t : t.parentNode) : !1;
}
function ad(e, t) {
	for (;;) {
		if (U(e) && t) return e;
		if (H(e) && t > 0) {
			if (e.contentEditable == "false") return null;
			e = e.childNodes[t - 1], t = V(e);
		} else if (e.parentNode && !cd(e)) t = B(e), e = e.parentNode;
		else return null;
	}
}
function od(e, t) {
	for (;;) {
		if (U(e) && t < e.nodeValue.length) return e;
		if (H(e) && t < e.childNodes.length) {
			if (e.contentEditable == "false") return null;
			e = e.childNodes[t], t = 0;
		} else if (e.parentNode && !cd(e)) t = B(e) + 1, e = e.parentNode;
		else return null;
	}
}
function sd(e, t, n) {
	for (let r = t == 0, i = t == V(e); r || i;) {
		if (e == n) return !0;
		let t = B(e);
		if (e = e.parentNode, !e) return !1;
		r &&= t == 0, i &&= t == V(e);
	}
}
function cd(e) {
	let t;
	for (let n = e; n && !(t = n.pmViewDesc); n = n.parentNode);
	return t && t.node && t.node.isBlock && (t.dom == e || t.contentDOM == e);
}
var ld = function(e) {
	return e.focusNode && td(e.focusNode, e.focusOffset, e.anchorNode, e.anchorOffset);
};
function ud(e, t) {
	let n = document.createEvent("Event");
	return n.initEvent("keydown", !0, !0), n.keyCode = e, n.key = n.code = t, n;
}
function dd(e) {
	let t = e.activeElement;
	for (; t && t.shadowRoot;) t = t.shadowRoot.activeElement;
	return t;
}
function fd(e, t, n) {
	if (e.caretPositionFromPoint) try {
		let r = e.caretPositionFromPoint(t, n);
		if (r) return {
			node: r.offsetNode,
			offset: Math.min(V(r.offsetNode), r.offset)
		};
	} catch {}
	if (e.caretRangeFromPoint) {
		let r = e.caretRangeFromPoint(t, n);
		if (r) return {
			node: r.startContainer,
			offset: Math.min(V(r.startContainer), r.startOffset)
		};
	}
}
var pd = typeof navigator < "u" ? navigator : null, md = typeof document < "u" ? document : null, hd = pd && pd.userAgent || "", gd = /Edge\/(\d+)/.exec(hd), _d = /MSIE \d/.exec(hd), vd = /Trident\/(?:[7-9]|\d{2,})\..*rv:(\d+)/.exec(hd), yd = !!(_d || vd || gd), bd = _d ? document.documentMode : vd ? +vd[1] : gd ? +gd[1] : 0, xd = !yd && /gecko\/(\d+)/i.test(hd);
xd && +(/Firefox\/(\d+)/.exec(hd) || [0, 0])[1];
var Sd = !yd && /Chrome\/(\d+)/.exec(hd), W = !!Sd, Cd = Sd ? +Sd[1] : 0, G = !yd && !!pd && /Apple Computer/.test(pd.vendor), wd = G && (/Mobile\/\w+/.test(hd) || !!pd && pd.maxTouchPoints > 2), Td = wd || (pd ? /Mac/.test(pd.platform) : !1), Ed = pd ? /Win/.test(pd.platform) : !1, Dd = /Android \d/.test(hd), Od = !!md && "webkitFontSmoothing" in md.documentElement.style, kd = Od ? +(/\bAppleWebKit\/(\d+)/.exec(navigator.userAgent) || [0, 0])[1] : 0;
function Ad(e) {
	let t = e.defaultView && e.defaultView.visualViewport;
	return t ? {
		left: 0,
		right: t.width,
		top: 0,
		bottom: t.height
	} : {
		left: 0,
		right: e.documentElement.clientWidth,
		top: 0,
		bottom: e.documentElement.clientHeight
	};
}
function jd(e, t) {
	return typeof e == "number" ? e : e[t];
}
function Md(e) {
	let t = e.getBoundingClientRect(), n = t.width / e.offsetWidth || 1, r = t.height / e.offsetHeight || 1;
	return {
		left: t.left,
		right: t.left + e.clientWidth * n,
		top: t.top,
		bottom: t.top + e.clientHeight * r
	};
}
function Nd(e, t, n) {
	if (!Jd(t) && t.left == 0) return;
	let r = e.someProp("scrollThreshold") || 0, i = e.someProp("scrollMargin") || 5, a = e.dom.ownerDocument;
	for (let o = n || e.dom; o;) {
		if (!H(o)) {
			o = Zu(o);
			continue;
		}
		let e = o == a.body, n = e ? Ad(a) : Md(o), s = 0, c = 0;
		if (t.top < n.top + jd(r, "top") ? c = -(n.top - t.top + jd(i, "top")) : t.bottom > n.bottom - jd(r, "bottom") && (c = t.bottom - t.top > n.bottom - n.top ? t.top + jd(i, "top") - n.top : t.bottom - n.bottom + jd(i, "bottom")), t.left < n.left + jd(r, "left") ? s = -(n.left - t.left + jd(i, "left")) : t.right > n.right - jd(r, "right") && (s = t.right - n.right + jd(i, "right")), s || c) {
			if (e) a.defaultView.scrollBy(s, c);
			else {
				let e = o.scrollLeft, n = o.scrollTop;
				c && (o.scrollTop += c), s && (o.scrollLeft += s);
				let r = o.scrollLeft - e, i = o.scrollTop - n;
				t = {
					left: t.left - r,
					top: t.top - i,
					right: t.right - r,
					bottom: t.bottom - i
				};
			}
		}
		let l = e ? "fixed" : getComputedStyle(o).position;
		if (/^(fixed|sticky)$/.test(l)) break;
		o = l == "absolute" ? o.offsetParent : Zu(o);
	}
}
function Pd(e) {
	let t = e.dom.getBoundingClientRect(), n = Math.max(0, t.top), r, i;
	for (let a = (t.left + t.right) / 2, o = n + 1; o < Math.min(innerHeight, t.bottom); o += 5) {
		let t = e.root.elementFromPoint(a, o);
		if (!t || t == e.dom || !e.dom.contains(t)) continue;
		let s = t.getBoundingClientRect();
		if (s.top >= n - 20) {
			r = t, i = s.top;
			break;
		}
	}
	return {
		refDOM: r,
		refTop: i,
		stack: Fd(e.dom)
	};
}
function Fd(e) {
	let t = [], n = e.ownerDocument;
	for (let r = e; r && (t.push({
		dom: r,
		top: r.scrollTop,
		left: r.scrollLeft
	}), e != n); r = Zu(r));
	return t;
}
function Id({ refDOM: e, refTop: t, stack: n }) {
	let r = e ? e.getBoundingClientRect().top : 0;
	Ld(n, r == 0 ? 0 : r - t);
}
function Ld(e, t) {
	for (let n = 0; n < e.length; n++) {
		let { dom: r, top: i, left: a } = e[n];
		r.scrollTop != i + t && (r.scrollTop = i + t), r.scrollLeft != a && (r.scrollLeft = a);
	}
}
var Rd = null;
function zd(e) {
	if (e.setActive) return e.setActive();
	if (Rd) return e.focus(Rd);
	let t = Fd(e);
	e.focus(Rd == null ? { get preventScroll() {
		return Rd = { preventScroll: !0 }, !0;
	} } : void 0), Rd || (Rd = !1, Ld(t, 0));
}
function Bd(e, t) {
	let n, r = 2e8, i, a = 0, o = t.top, s = t.top, c, l;
	for (let u = e.firstChild, d = 0; u; u = u.nextSibling, d++) {
		let e;
		if (H(u)) e = u.getClientRects();
		else if (U(u)) e = $u(u).getClientRects();
		else continue;
		for (let f = 0; f < e.length; f++) {
			let p = e[f];
			if (p.top <= o && p.bottom >= s) {
				o = Math.max(p.bottom, o), s = Math.min(p.top, s);
				let e = p.left > t.left ? p.left - t.left : p.right < t.left ? t.left - p.right : 0;
				if (e < r) {
					n = u, r = e, i = e && U(n) ? {
						left: p.right < t.left ? p.right : p.left,
						top: t.top
					} : t, H(u) && e && (a = d + +(t.left >= (p.left + p.right) / 2));
					continue;
				}
			} else p.top > t.top && !c && p.left <= t.left && p.right >= t.left && (c = u, l = {
				left: Math.max(p.left, Math.min(p.right, t.left)),
				top: p.top
			});
			!n && (t.left >= p.right && t.top >= p.top || t.left >= p.left && t.top >= p.bottom) && (a = d + 1);
		}
	}
	return !n && c && (n = c, i = l, r = 0), n && U(n) ? Vd(n, i) : !n || r ? {
		node: e,
		offset: a
	} : Bd(n, i);
}
function Vd(e, t) {
	let n = e.nodeValue.length, r = document.createRange(), i;
	for (let a = 0; a < n; a++) {
		r.setEnd(e, a + 1), r.setStart(e, a);
		let n = Yd(r, 1);
		if (n.top != n.bottom && Hd(t, n)) {
			i = {
				node: e,
				offset: a + +(t.left >= (n.left + n.right) / 2)
			};
			break;
		}
	}
	return r.detach(), i || {
		node: e,
		offset: 0
	};
}
function Hd(e, t) {
	return e.left >= t.left - 1 && e.left <= t.right + 1 && e.top >= t.top - 1 && e.top <= t.bottom + 1;
}
function Ud(e, t) {
	let n = e.parentNode;
	return n && /^li$/i.test(n.nodeName) && t.left < e.getBoundingClientRect().left ? n : e;
}
function Wd(e, t, n) {
	let { node: r, offset: i } = Bd(t, n), a = -1;
	if (H(r) && !r.firstChild) {
		let e = r.getBoundingClientRect();
		a = e.left != e.right && n.left > (e.left + e.right) / 2 ? 1 : -1;
	}
	return e.docView.posFromDOM(r, i, a);
}
function Gd(e, t, n, r) {
	let i = -1;
	for (let n = t, a = !1; n != e.dom;) {
		let t = e.docView.nearestDesc(n, !0), o;
		if (!t) return null;
		if (H(t.dom) && (t.node.isBlock && t.parent || !t.contentDOM) && ((o = t.dom.getBoundingClientRect()).width || o.height) && (t.node.isBlock && t.parent && !/^T(R|BODY|HEAD|FOOT)$/.test(t.dom.nodeName) && (!a && o.left > r.left || o.top > r.top ? i = t.posBefore : (!a && o.right < r.left || o.bottom < r.top) && (i = t.posAfter), a = !0), !t.contentDOM && i < 0 && !t.node.isText)) return (t.node.isBlock ? r.top < (o.top + o.bottom) / 2 : r.left < (o.left + o.right) / 2) ? t.posBefore : t.posAfter;
		n = t.dom.parentNode;
	}
	return i > -1 ? i : e.docView.posFromDOM(t, n, -1);
}
function Kd(e, t, n) {
	let r = e.childNodes.length;
	if (r && n.top < n.bottom) for (let i = Math.max(0, Math.min(r - 1, Math.floor(r * (t.top - n.top) / (n.bottom - n.top)) - 2)), a = i;;) {
		let n = e.childNodes[a];
		if (H(n)) {
			let e = n.getClientRects();
			for (let r = 0; r < e.length; r++) {
				let i = e[r];
				if (Hd(t, i)) return Kd(n, t, i);
			}
		}
		if ((a = (a + 1) % r) == i) break;
	}
	return e;
}
function qd(e, t) {
	let n = e.dom.ownerDocument, r, i = 0, a = fd(n, t.left, t.top);
	a && ({node: r, offset: i} = a);
	let o = (e.root.elementFromPoint ? e.root : n).elementFromPoint(t.left, t.top), s;
	if (!o || !e.dom.contains(o)) {
		let n = e.dom.getBoundingClientRect();
		if (!Hd(t, n) || (o = Kd(e.dom, t, n), !o)) return null;
	}
	if (G) for (let e = o; r && e; e = Zu(e)) e.draggable && (r = void 0);
	if (o = Ud(o, t), r) {
		if (xd && H(r) && (i = Math.min(i, r.childNodes.length), i < r.childNodes.length)) {
			let e = r.childNodes[i], n;
			e.nodeName == "IMG" && (n = e.getBoundingClientRect()).right <= t.left && n.bottom > t.top && i++;
		}
		let n;
		Od && i && H(r) && H(n = r.childNodes[i - 1]) && n.contentEditable == "false" && n.getBoundingClientRect().top >= t.top && i--, r == e.dom && i == r.childNodes.length - 1 && H(r.lastChild) && t.top > r.lastChild.getBoundingClientRect().bottom ? s = e.state.doc.content.size : (i == 0 || !H(r) || r.childNodes[i - 1].nodeName != "BR") && (s = Gd(e, r, i, t));
	}
	s ??= Wd(e, o, t);
	let c = e.docView.nearestDesc(o, !0);
	return {
		pos: s,
		inside: c ? c.posAtStart - c.border : -1
	};
}
function Jd(e) {
	return e.top < e.bottom || e.left < e.right;
}
function Yd(e, t) {
	let n = e.getClientRects();
	if (n.length) {
		let e = n[t < 0 ? 0 : n.length - 1];
		if (Jd(e)) return e;
	}
	return Array.prototype.find.call(n, Jd) || e.getBoundingClientRect();
}
var Xd = /[\u0590-\u05f4\u0600-\u06ff\u0700-\u08ac]/;
function Zd(e, t, n) {
	let { node: r, offset: i, atom: a } = e.docView.domFromPos(t, n < 0 ? -1 : 1), o = Od || xd;
	if (U(r)) {
		if (o && (Xd.test(r.nodeValue) || (n < 0 ? !i : i == r.nodeValue.length))) {
			let e = Yd($u(r, i, i), n);
			if (xd && i && /\s/.test(r.nodeValue[i - 1]) && i < r.nodeValue.length) {
				let t = Yd($u(r, i - 1, i - 1), -1);
				if (t.top == e.top) {
					let n = Yd($u(r, i, i + 1), -1);
					if (n.top != e.top) return Qd(n, n.left < t.left);
				}
			}
			return e;
		}
		{
			let e = i, t = i, a = n < 0 ? 1 : -1;
			return n < 0 && !i ? (t++, a = -1) : n >= 0 && i == r.nodeValue.length ? (e--, a = 1) : n < 0 ? e-- : t++, Qd(Yd($u(r, e, t), a), a < 0);
		}
	}
	if (!e.state.doc.resolve(t - (a || 0)).parent.inlineContent) {
		if (a == null && i && (n < 0 || i == V(r))) {
			let e = r.childNodes[i - 1];
			if (H(e)) return $d(e.getBoundingClientRect(), !1);
		}
		if (a == null && i < V(r)) {
			let e = r.childNodes[i];
			if (H(e)) return $d(e.getBoundingClientRect(), !0);
		}
		return $d(r.getBoundingClientRect(), n >= 0);
	}
	if (a == null && i && (n < 0 || i == V(r))) {
		let e = r.childNodes[i - 1], t = U(e) ? $u(e, V(e) - +!o) : H(e) && (e.nodeName != "BR" || !e.nextSibling) ? e : null;
		if (t) return Qd(Yd(t, 1), !1);
	}
	if (a == null && i < V(r)) {
		let e = r.childNodes[i];
		for (; e.pmViewDesc && e.pmViewDesc.ignoreForCoords;) e = e.nextSibling;
		let t = e ? U(e) ? $u(e, 0, +!o) : H(e) ? e : null : null;
		if (t) return Qd(Yd(t, -1), !0);
	}
	return Qd(Yd(U(r) ? $u(r) : r, -n), n >= 0);
}
function Qd(e, t) {
	if (e.width == 0) return e;
	let n = t ? e.left : e.right;
	return {
		top: e.top,
		bottom: e.bottom,
		left: n,
		right: n
	};
}
function $d(e, t) {
	if (e.height == 0) return e;
	let n = t ? e.top : e.bottom;
	return {
		top: n,
		bottom: n,
		left: e.left,
		right: e.right
	};
}
function ef(e, t, n) {
	let r = e.state, i = e.root.activeElement;
	r != t && e.updateState(t), i != e.dom && e.focus();
	try {
		return n();
	} finally {
		r != t && e.updateState(r), i != e.dom && i && i.focus();
	}
}
function tf(e, t, n) {
	let r = t.selection, i = n == "up" ? r.$from : r.$to;
	return ef(e, t, () => {
		let { node: t } = e.docView.domFromPos(i.pos, n == "up" ? -1 : 1);
		for (;;) {
			let n = e.docView.nearestDesc(t, !0);
			if (!n) break;
			if (n.node.isBlock) {
				t = n.contentDOM || n.dom;
				break;
			}
			t = n.dom.parentNode;
		}
		let r = Zd(e, i.pos, 1);
		for (let e = t.firstChild; e; e = e.nextSibling) {
			let t;
			if (H(e)) t = e.getClientRects();
			else if (U(e)) t = $u(e, 0, e.nodeValue.length).getClientRects();
			else continue;
			for (let e = 0; e < t.length; e++) {
				let i = t[e];
				if (i.bottom > i.top + 1 && (n == "up" ? r.top - i.top > (i.bottom - r.top) * 2 : i.bottom - r.bottom > (r.bottom - i.top) * 2)) return !1;
			}
		}
		return !0;
	});
}
var nf = /[\u0590-\u08ac]/;
function rf(e, t, n) {
	let { $head: r } = t.selection;
	if (!r.parent.isTextblock) return !1;
	let i = r.parentOffset, a = !i, o = i == r.parent.content.size, s = e.domSelection();
	return s ? !nf.test(r.parent.textContent) || !s.modify ? n == "left" || n == "backward" ? a : o : ef(e, t, () => {
		let { focusNode: t, focusOffset: i, anchorNode: a, anchorOffset: o } = e.domSelectionRange(), c = s.caretBidiLevel;
		s.modify("move", n, "character");
		let l = r.depth ? e.docView.domAfterPos(r.before()) : e.dom, { focusNode: u, focusOffset: d } = e.domSelectionRange(), f = u && !id(l, u) || t == u && i == d;
		try {
			s.collapse(a, o), t && (t != a || i != o) && s.extend && s.extend(t, i);
		} catch {}
		return c != null && (s.caretBidiLevel = c), f;
	}) : r.pos == r.start() || r.pos == r.end();
}
var af = null, of = null, sf = !1;
function cf(e, t, n) {
	return af == t && of == n ? sf : (af = t, of = n, sf = n == "up" || n == "down" ? tf(e, t, n) : rf(e, t, n));
}
var lf = 0, uf = 1, df = 2, ff = 3, pf = class {
	constructor(e, t, n, r) {
		this.parent = e, this.children = t, this.dom = n, this.contentDOM = r, this.dirty = lf, n.pmViewDesc = this;
	}
	matchesWidget(e) {
		return !1;
	}
	matchesMark(e) {
		return !1;
	}
	matchesNode(e, t, n) {
		return !1;
	}
	matchesHack(e) {
		return !1;
	}
	parseRule(e) {
		return null;
	}
	stopEvent(e) {
		return !1;
	}
	get size() {
		let e = 0;
		for (let t = 0; t < this.children.length; t++) e += this.children[t].size;
		return e;
	}
	get border() {
		return 0;
	}
	destroy() {
		this.parent = void 0, this.dom.pmViewDesc == this && (this.dom.pmViewDesc = void 0);
		for (let e = 0; e < this.children.length; e++) this.children[e].destroy();
	}
	posBeforeChild(e) {
		for (let t = 0, n = this.posAtStart;; t++) {
			let r = this.children[t];
			if (r == e) return n;
			n += r.size;
		}
	}
	get posBefore() {
		return this.parent.posBeforeChild(this);
	}
	get posAtStart() {
		return this.parent ? this.parent.posBeforeChild(this) + this.border : 0;
	}
	get posAfter() {
		return this.posBefore + this.size;
	}
	get posAtEnd() {
		return this.posAtStart + this.size - 2 * this.border;
	}
	localPosFromDOM(e, t, n) {
		if (this.contentDOM && id(this.contentDOM, e)) {
			if (n < 0) {
				let n, r;
				if (e == this.contentDOM) n = e.childNodes[t - 1];
				else {
					for (; e.parentNode != this.contentDOM;) e = e.parentNode;
					n = e.previousSibling;
				}
				for (; n && !((r = n.pmViewDesc) && r.parent == this);) n = n.previousSibling;
				return n ? this.posBeforeChild(r) + r.size : this.posAtStart;
			}
			{
				let n, r;
				if (e == this.contentDOM) n = e.childNodes[t];
				else {
					for (; e.parentNode != this.contentDOM;) e = e.parentNode;
					n = e.nextSibling;
				}
				for (; n && !((r = n.pmViewDesc) && r.parent == this);) n = n.nextSibling;
				return n ? this.posBeforeChild(r) : this.posAtEnd;
			}
		}
		let r;
		if (e == this.dom && this.contentDOM) r = t > B(this.contentDOM);
		else if (this.contentDOM && this.contentDOM != this.dom && this.dom.contains(this.contentDOM)) r = !!(e.compareDocumentPosition(this.contentDOM) & 4);
		else if (this.dom.firstChild) {
			if (t == 0) for (let t = e;; t = t.parentNode) {
				if (t == this.dom) {
					r = !1;
					break;
				}
				if (t.previousSibling) break;
			}
			if (r == null && t == e.childNodes.length) for (let t = e;; t = t.parentNode) {
				if (t == this.dom) {
					r = !0;
					break;
				}
				if (t.nextSibling) break;
			}
		}
		return r ?? n > 0 ? this.posAtEnd : this.posAtStart;
	}
	nearestDesc(e, t = !1) {
		for (let n = !0, r = e; r; r = r.parentNode) {
			let i = this.getDesc(r), a;
			if (i && (!t || i.node)) {
				if (n && (a = i.nodeDOM) && !(H(a) ? id(a, e) : a == e)) n = !1;
				else return i;
			}
		}
	}
	getDesc(e) {
		let t = e.pmViewDesc;
		for (let e = t; e; e = e.parent) if (e == this) return t;
	}
	posFromDOM(e, t, n) {
		for (let r = e; r; r = r.parentNode) {
			let i = this.getDesc(r);
			if (i) return i.localPosFromDOM(e, t, n);
		}
		return -1;
	}
	descAt(e) {
		for (let t = 0, n = 0; t < this.children.length; t++) {
			let r = this.children[t], i = n + r.size;
			if (n == e && i != n) {
				for (; !r.border && r.children.length;) for (let e = 0; e < r.children.length; e++) {
					let t = r.children[e];
					if (t.size) {
						r = t;
						break;
					}
				}
				return r;
			}
			if (e < i) return r.descAt(e - n - r.border);
			n = i;
		}
	}
	domFromPos(e, t) {
		if (!this.contentDOM) return {
			node: this.dom,
			offset: 0,
			atom: e + 1
		};
		let n = 0, r = 0;
		for (let t = 0; n < this.children.length; n++) {
			let i = this.children[n], a = t + i.size;
			if (a > e) {
				r = e - t;
				break;
			}
			t = a;
		}
		if (r) return this.children[n].domFromPos(r - this.children[n].border, t);
		for (; n > 0; n--) {
			let e = this.children[n - 1];
			if (e.size || !(e instanceof bf || e instanceof mf && e.side >= 0)) break;
		}
		if (t <= 0) {
			let e, r = !0;
			for (; e = n ? this.children[n - 1] : null, e && e.dom.parentNode != this.contentDOM; n--, r = !1);
			return e && t && r && !e.border && !e.domAtom ? e.domFromPos(e.size, t) : {
				node: this.contentDOM,
				offset: e ? B(e.dom) + 1 : 0
			};
		}
		{
			let e, r = !0;
			for (; e = n < this.children.length ? this.children[n] : null, e && e.dom.parentNode != this.contentDOM; n++, r = !1);
			return e && r && !e.border && !e.domAtom ? e.domFromPos(0, t) : {
				node: this.contentDOM,
				offset: e ? B(e.dom) : this.contentDOM.childNodes.length
			};
		}
	}
	parseRange(e, t, n = 0) {
		if (this.children.length == 0) return {
			node: this.contentDOM,
			from: e,
			to: t,
			fromOffset: 0,
			toOffset: this.contentDOM.childNodes.length
		};
		let r = -1, i = -1;
		for (let a = n, o = 0;; o++) {
			let n = this.children[o], s = a + n.size;
			if (r == -1 && e <= s) {
				let i = a + n.border;
				if (e >= i && t <= s - n.border && n.node && n.contentDOM && this.contentDOM.contains(n.contentDOM)) return n.parseRange(e, t, i);
				e = a;
				for (let t = o; t > 0; t--) {
					let n = this.children[t - 1];
					if (n.size && n.dom.parentNode == this.contentDOM && !n.emptyChildAt(1)) {
						r = B(n.dom) + 1;
						break;
					}
					e -= n.size;
				}
				r == -1 && (r = 0);
			}
			if (r > -1 && (s > t || o == this.children.length - 1)) {
				t = s;
				for (let e = o + 1; e < this.children.length; e++) {
					let n = this.children[e];
					if (n.size && n.dom.parentNode == this.contentDOM && !n.emptyChildAt(-1)) {
						i = B(n.dom);
						break;
					}
					t += n.size;
				}
				i == -1 && (i = this.contentDOM.childNodes.length);
				break;
			}
			a = s;
		}
		return {
			node: this.contentDOM,
			from: e,
			to: t,
			fromOffset: r,
			toOffset: i
		};
	}
	emptyChildAt(e) {
		if (this.border || !this.contentDOM || !this.children.length) return !1;
		let t = this.children[e < 0 ? 0 : this.children.length - 1];
		return t.size == 0 || t.emptyChildAt(e);
	}
	domAfterPos(e) {
		let { node: t, offset: n } = this.domFromPos(e, 0);
		if (!H(t) || n == t.childNodes.length) throw RangeError("No node after pos " + e);
		return t.childNodes[n];
	}
	setSelection(e, t, n, r = !1) {
		let i = Math.min(e, t), a = Math.max(e, t);
		for (let o = 0, s = 0; o < this.children.length; o++) {
			let c = this.children[o], l = s + c.size;
			if (i > s && a < l) return c.setSelection(e - s - c.border, t - s - c.border, n, r);
			s = l;
		}
		let o = this.domFromPos(e, e ? -1 : 1), s = t == e ? o : this.domFromPos(t, t ? -1 : 1), c = n.root.getSelection(), l = n.domSelectionRange(), u = !1;
		if ((xd || G) && e == t) {
			let { node: e, offset: t } = o;
			if (U(e)) {
				if (u = !!(t && e.nodeValue[t - 1] == "\n"), u && t == e.nodeValue.length) for (let t = e, n; t; t = t.parentNode) {
					if (n = t.nextSibling) {
						n.nodeName == "BR" && (o = s = {
							node: n.parentNode,
							offset: B(n) + 1
						});
						break;
					}
					let e = t.pmViewDesc;
					if (e && e.node && e.node.isBlock) break;
				}
			} else {
				let n = e.childNodes[t - 1];
				u = n && (n.nodeName == "BR" || n.contentEditable == "false");
			}
		}
		if (xd && l.focusNode && l.focusNode != s.node && H(l.focusNode)) {
			let e = l.focusNode.childNodes[l.focusOffset];
			e && e.contentEditable == "false" && (r = !0);
		}
		if (!(r || u && G) && td(o.node, o.offset, l.anchorNode, l.anchorOffset) && td(s.node, s.offset, l.focusNode, l.focusOffset)) return;
		let d = !1;
		if ((c.extend || e == t) && !(u && xd)) {
			c.collapse(o.node, o.offset);
			try {
				e != t && c.extend(s.node, s.offset), d = !0;
			} catch {}
		}
		if (!d) {
			if (e > t) {
				let e = o;
				o = s, s = e;
			}
			let n = document.createRange();
			n.setEnd(s.node, s.offset), n.setStart(o.node, o.offset), c.removeAllRanges(), c.addRange(n);
		}
	}
	ignoreMutation(e) {
		return !this.contentDOM && e.type != "selection";
	}
	get contentLost() {
		return this.contentDOM && this.contentDOM != this.dom && !this.dom.contains(this.contentDOM);
	}
	markDirty(e, t) {
		for (let n = 0, r = 0; r < this.children.length; r++) {
			let i = this.children[r], a = n + i.size;
			if (n == a ? e <= a && t >= n : e < a && t > n) {
				let r = n + i.border, o = a - i.border;
				if (e >= r && t <= o) {
					this.dirty = e == n || t == a ? df : uf, e == r && t == o && (i.contentLost || i.dom.parentNode != this.contentDOM) ? i.dirty = ff : i.markDirty(e - r, t - r);
					return;
				}
				i.dirty = i.dom == i.contentDOM && i.dom.parentNode == this.contentDOM && !i.children.length ? df : ff;
			}
			n = a;
		}
		this.dirty = df;
	}
	markParentsDirty() {
		let e = 1;
		for (let t = this.parent; t; t = t.parent, e++) {
			let n = e == 1 ? df : uf;
			t.dirty < n && (t.dirty = n);
		}
	}
	get domAtom() {
		return !1;
	}
	get ignoreForCoords() {
		return !1;
	}
	get ignoreForSelection() {
		return !1;
	}
	isText(e) {
		return !1;
	}
}, mf = class extends pf {
	constructor(e, t, n, r) {
		let i, a = t.type.toDOM;
		if (typeof a == "function" && (a = a(n, () => {
			if (!i) return r;
			if (i.parent) return i.parent.posBeforeChild(i);
		})), !t.type.spec.raw) {
			let e = H(a) ? a : document.createElement("span");
			e != a && e.appendChild(a), e.hasAttribute("contenteditable") || (e.contentEditable = "false"), e.classList.add("ProseMirror-widget");
		}
		super(e, [], a, null), this.widget = t, this.widget = t, i = this;
	}
	matchesWidget(e) {
		return this.dirty == lf && e.type.eq(this.widget.type);
	}
	parseRule() {
		return { ignore: !0 };
	}
	stopEvent(e) {
		let t = this.widget.spec.stopEvent;
		return t ? t(e) : !1;
	}
	ignoreMutation(e) {
		return e.type != "selection" || this.widget.spec.ignoreSelection;
	}
	destroy() {
		this.widget.type.destroy(this.dom), super.destroy();
	}
	get domAtom() {
		return !0;
	}
	get ignoreForSelection() {
		return !!this.widget.type.spec.relaxedSide;
	}
	get side() {
		return this.widget.type.side;
	}
}, hf = class extends pf {
	constructor(e, t, n, r) {
		super(e, [], t, null), this.textDOM = n, this.text = r;
	}
	get size() {
		return this.text.length;
	}
	localPosFromDOM(e, t) {
		return e == this.textDOM ? this.posAtStart + t : this.posAtStart + (t ? this.size : 0);
	}
	domFromPos(e) {
		return {
			node: this.textDOM,
			offset: e
		};
	}
	ignoreMutation(e) {
		return e.type === "characterData" && e.target.nodeValue == e.oldValue;
	}
}, gf = class e extends pf {
	constructor(e, t, n, r, i) {
		super(e, [], n, r), this.mark = t, this.spec = i;
	}
	static create(t, n, r, i) {
		let a = i.nodeViews[n.type.name], o = a && a(n, i, r);
		return (!o || !o.dom) && (o = As.renderSpec(document, n.type.spec.toDOM(n, r), null, n.attrs)), new e(t, n, o.dom, o.contentDOM || o.dom, o);
	}
	parseRule() {
		return this.dirty & ff || this.mark.type.spec.reparseInView ? null : {
			mark: this.mark.type.name,
			attrs: this.mark.attrs,
			contentElement: this.contentDOM
		};
	}
	matchesMark(e) {
		return this.dirty != ff && this.mark.eq(e);
	}
	markDirty(e, t) {
		if (super.markDirty(e, t), this.dirty != lf) {
			let e = this.parent;
			for (; !e.node;) e = e.parent;
			e.dirty < this.dirty && (e.dirty = this.dirty), this.dirty = lf;
		}
	}
	slice(t, n, r) {
		let i = e.create(this.parent, this.mark, !0, r), a = this.children, o = this.size;
		n < o && (a = Lf(a, n, o, r)), t > 0 && (a = Lf(a, 0, t, r));
		for (let e = 0; e < a.length; e++) a[e].parent = i;
		return i.children = a, i;
	}
	ignoreMutation(e) {
		return this.spec.ignoreMutation ? this.spec.ignoreMutation(e) : super.ignoreMutation(e);
	}
	destroy() {
		this.spec.destroy && this.spec.destroy(), super.destroy();
	}
}, _f = class e extends pf {
	constructor(e, t, n, r, i, a, o) {
		super(e, [], i, a), this.node = t, this.outerDeco = n, this.innerDeco = r, this.nodeDOM = o;
	}
	static create(t, n, r, i, a, o) {
		let s = a.nodeViews[n.type.name], c, l = s && s(n, a, () => {
			if (!c) return o;
			if (c.parent) return c.parent.posBeforeChild(c);
		}, r, i), u = l && l.dom, d = l && l.contentDOM;
		if (n.isText) {
			if (!u) u = document.createTextNode(n.text);
			else if (!U(u)) throw RangeError("Text must be rendered as a DOM text node");
		} else if (!u) {
			let e = As.renderSpec(document, n.type.spec.toDOM(n), null, n.attrs);
			({dom: u, contentDOM: d} = e);
		}
		!d && !n.isText && u.nodeName != "BR" && (u.hasAttribute("contenteditable") || (u.contentEditable = "false"), n.type.spec.draggable && (u.draggable = !0));
		let f = u;
		return u = Of(u, r, n), l ? c = new xf(t, n, r, i, u, d || null, f, l) : n.isText ? new yf(t, n, r, i, u, f) : new e(t, n, r, i, u, d || null, f);
	}
	parseRule(e) {
		if (this.node.type.spec.reparseInView) return null;
		let t = {
			node: this.node.type.name,
			attrs: this.node.attrs
		};
		if (this.node.type.whitespace == "pre" && (t.preserveWhitespace = "full"), !this.contentDOM) t.getContent = () => this.node.content;
		else if (!this.contentLost) t.contentElement = this.contentDOM;
		else {
			for (let e = this.children.length - 1; e >= 0; e--) {
				let n = this.children[e];
				if (this.dom.contains(n.dom.parentNode)) {
					t.contentElement = n.dom.parentNode;
					break;
				}
			}
			if (!t.contentElement) {
				let n = e && e.find((t) => H(t) && e.indexOf(t.parentNode) < 0 && this.dom.contains(t));
				n ? t.contentElement = n : t.getContent = () => N.empty;
			}
		}
		return t;
	}
	matchesNode(e, t, n) {
		return this.dirty == lf && e.eq(this.node) && kf(t, this.outerDeco) && n.eq(this.innerDeco);
	}
	get size() {
		return this.node.nodeSize;
	}
	get border() {
		return +!this.node.isLeaf;
	}
	updateChildren(e, t) {
		let n = this.node.inlineContent, r = t, i = e.composing ? this.localCompositionInfo(e, t) : null, a = i && i.pos > -1 ? i : null, o = i && i.pos < 0, s = new jf(this, a && a.node, e), c = this.node.inlineContent, l = c && this.node.isBlock;
		Pf(this.node, this.innerDeco, (t, i, a) => {
			t.spec.marks ? s.syncToMarks(t.spec.marks, n, e, i) : t.type.side >= 0 && !a && s.syncToMarks(i == this.node.childCount ? P.none : this.node.child(i).marks, n, e, i), s.placeWidget(t, e, r);
		}, (t, a, u, d) => {
			s.syncToMarks(t.marks, n, e, d), l && t.isAtom && !t.isText && s.addHackNode("IMG", s.top);
			let f;
			s.findNodeMatch(t, a, u, d) || o && e.state.selection.from > r && e.state.selection.to < r + t.nodeSize && (f = s.findIndexWithChild(i.node)) > -1 && s.updateNodeAt(t, a, u, f, e) || s.updateNextNode(t, a, u, e, d, r) || s.addNode(t, a, u, e, r), r += t.nodeSize, c && (l = t.isAtom && !t.isText);
		}), s.syncToMarks([], n, e, 0), this.node.isTextblock && s.addTextblockHacks(), s.destroyRest(), (s.changed || this.dirty == df) && (a && this.protectLocalComposition(e, a), Sf(this.contentDOM, this.children, e), wd && Ff(this.dom));
	}
	localCompositionInfo(e, t) {
		let { from: n, to: r } = e.state.selection;
		if (!(e.state.selection instanceof R) || n < t || r > t + this.node.content.size) return null;
		let i = e.input.compositionNode;
		if (!i || !this.dom.contains(i.parentNode)) return null;
		if (this.node.inlineContent) {
			let e = i.nodeValue, a = If(this.node.content, e, n - t, r - t);
			return a < 0 ? null : {
				node: i,
				pos: a,
				text: e
			};
		}
		return {
			node: i,
			pos: -1,
			text: ""
		};
	}
	protectLocalComposition(e, { node: t, pos: n, text: r }) {
		if (this.getDesc(t)) return;
		let i = t;
		for (; i.parentNode != this.contentDOM; i = i.parentNode) {
			for (; i.previousSibling;) i.parentNode.removeChild(i.previousSibling);
			for (; i.nextSibling;) i.parentNode.removeChild(i.nextSibling);
			i.pmViewDesc && (i.pmViewDesc = void 0);
		}
		let a = new hf(this, i, t, r);
		e.input.compositionNodes.push(a), this.children = Lf(this.children, n, n + r.length, e, a);
	}
	update(e, t, n, r) {
		return this.dirty == ff || !e.sameMarkup(this.node) ? !1 : (this.updateInner(e, t, n, r), !0);
	}
	updateInner(e, t, n, r) {
		this.updateOuterDeco(t), this.node = e, this.innerDeco = n, this.contentDOM && this.updateChildren(r, this.posAtStart), this.dirty = lf;
	}
	updateOuterDeco(e) {
		if (kf(e, this.outerDeco)) return;
		let t = !H(this.nodeDOM), n = this.dom;
		this.dom = Ef(this.dom, this.nodeDOM, Tf(this.outerDeco, this.node, t), Tf(e, this.node, t)), this.dom != n && (n.pmViewDesc = void 0, this.dom.pmViewDesc = this), this.outerDeco = e;
	}
	selectNode() {
		H(this.nodeDOM) && (this.nodeDOM.classList.add("ProseMirror-selectednode"), (this.contentDOM || !this.node.type.spec.draggable) && (this.nodeDOM.draggable = !0));
	}
	deselectNode() {
		H(this.nodeDOM) && (this.nodeDOM.classList.remove("ProseMirror-selectednode"), (this.contentDOM || !this.node.type.spec.draggable) && this.nodeDOM.removeAttribute("draggable"));
	}
	get domAtom() {
		return this.node.isAtom;
	}
};
function vf(e, t, n, r, i) {
	Of(r, t, e);
	let a = new _f(void 0, e, t, n, r, r, r);
	return a.contentDOM && a.updateChildren(i, 0), a;
}
var yf = class e extends _f {
	constructor(e, t, n, r, i, a) {
		super(e, t, n, r, i, null, a);
	}
	parseRule() {
		let e = this.nodeDOM.parentNode;
		for (; e && e != this.dom && !e.pmIsDeco;) e = e.parentNode;
		return { skip: e || !0 };
	}
	update(e, t, n, r) {
		return this.dirty == ff || this.dirty != lf && !this.inParent() || !e.sameMarkup(this.node) ? !1 : (this.updateOuterDeco(t), (this.dirty != lf || e.text != this.node.text) && e.text != this.nodeDOM.nodeValue && (this.nodeDOM.nodeValue = e.text, r.trackWrites == this.nodeDOM && (r.trackWrites = null)), this.node = e, this.dirty = lf, !0);
	}
	inParent() {
		let e = this.parent.contentDOM;
		for (let t = this.nodeDOM; t; t = t.parentNode) if (t == e) return !0;
		return !1;
	}
	domFromPos(e) {
		return {
			node: this.nodeDOM,
			offset: e
		};
	}
	localPosFromDOM(e, t, n) {
		return e == this.nodeDOM ? this.posAtStart + Math.min(t, this.node.text.length) : super.localPosFromDOM(e, t, n);
	}
	ignoreMutation(e) {
		return e.type != "characterData" && e.type != "selection";
	}
	slice(t, n, r) {
		let i = this.node.cut(t, n), a = document.createTextNode(i.text);
		return new e(this.parent, i, this.outerDeco, this.innerDeco, a, a);
	}
	markDirty(e, t) {
		super.markDirty(e, t), this.dom != this.nodeDOM && (e == 0 || t == this.nodeDOM.nodeValue.length) && (this.dirty = ff);
	}
	get domAtom() {
		return !1;
	}
	isText(e) {
		return this.node.text == e;
	}
}, bf = class extends pf {
	parseRule() {
		return { ignore: !0 };
	}
	matchesHack(e) {
		return this.dirty == lf && this.dom.nodeName == e;
	}
	get domAtom() {
		return !0;
	}
	get ignoreForCoords() {
		return this.dom.nodeName == "IMG";
	}
}, xf = class extends _f {
	constructor(e, t, n, r, i, a, o, s) {
		super(e, t, n, r, i, a, o), this.spec = s;
	}
	update(e, t, n, r) {
		if (this.dirty == ff) return !1;
		if (this.spec.update && (this.node.type == e.type || this.spec.multiType)) {
			let i = this.spec.update(e, t, n);
			return i && this.updateInner(e, t, n, r), i;
		}
		return !this.contentDOM && !e.isLeaf ? !1 : super.update(e, t, n, r);
	}
	selectNode() {
		this.spec.selectNode ? this.spec.selectNode() : super.selectNode();
	}
	deselectNode() {
		this.spec.deselectNode ? this.spec.deselectNode() : super.deselectNode();
	}
	setSelection(e, t, n, r) {
		this.spec.setSelection ? this.spec.setSelection(e, t, n.root) : super.setSelection(e, t, n, r);
	}
	destroy() {
		this.spec.destroy && this.spec.destroy(), super.destroy();
	}
	stopEvent(e) {
		return this.spec.stopEvent ? this.spec.stopEvent(e) : !1;
	}
	ignoreMutation(e) {
		return this.spec.ignoreMutation ? this.spec.ignoreMutation(e) : super.ignoreMutation(e);
	}
};
function Sf(e, t, n) {
	let r = e.firstChild, i = !1;
	for (let a = 0; a < t.length; a++) {
		let o = t[a], s = o.dom;
		if (s.parentNode == e) {
			for (; s != r;) r = Af(r), i = !0;
			r = r.nextSibling;
		} else i = !0, e.insertBefore(s, r);
		if (o instanceof gf) {
			let t = r ? r.previousSibling : e.lastChild;
			Sf(o.contentDOM, o.children, n), r = t ? t.nextSibling : e.firstChild;
		}
	}
	for (; r;) r = Af(r), i = !0;
	i && n.trackWrites == e && (n.trackWrites = null);
}
var Cf = function(e) {
	e && (this.nodeName = e);
};
Cf.prototype = Object.create(null);
var wf = [new Cf()];
function Tf(e, t, n) {
	if (e.length == 0) return wf;
	let r = n ? wf[0] : new Cf(), i = [r];
	for (let a = 0; a < e.length; a++) {
		let o = e[a].type.attrs;
		if (o) {
			o.nodeName && i.push(r = new Cf(o.nodeName));
			for (let e in o) {
				let a = o[e];
				a != null && (n && i.length == 1 && i.push(r = new Cf(t.isInline ? "span" : "div")), e == "class" ? r.class = (r.class ? r.class + " " : "") + a : e == "style" ? r.style = (r.style ? r.style + ";" : "") + a : e != "nodeName" && (r[e] = a));
			}
		}
	}
	return i;
}
function Ef(e, t, n, r) {
	if (n == wf && r == wf) return t;
	let i = t;
	for (let t = 0; t < r.length; t++) {
		let a = r[t], o = n[t];
		if (t) {
			let t;
			o && o.nodeName == a.nodeName && i != e && (t = i.parentNode) && t.nodeName.toLowerCase() == a.nodeName || (t = document.createElement(a.nodeName), t.pmIsDeco = !0, t.appendChild(i), o = wf[0]), i = t;
		}
		Df(i, o || wf[0], a);
	}
	return i;
}
function Df(e, t, n) {
	for (let r in t) r != "class" && r != "style" && r != "nodeName" && !(r in n) && e.removeAttribute(r);
	for (let r in n) r != "class" && r != "style" && r != "nodeName" && n[r] != t[r] && e.setAttribute(r, n[r]);
	if (t.class != n.class) {
		let r = t.class ? t.class.split(" ").filter(Boolean) : [], i = n.class ? n.class.split(" ").filter(Boolean) : [];
		for (let t = 0; t < r.length; t++) i.indexOf(r[t]) == -1 && e.classList.remove(r[t]);
		for (let t = 0; t < i.length; t++) r.indexOf(i[t]) == -1 && e.classList.add(i[t]);
		e.classList.length == 0 && e.removeAttribute("class");
	}
	if (t.style != n.style) {
		if (t.style) {
			let n = /\s*([\w\-\xa1-\uffff]+)\s*:(?:"(?:\\.|[^"])*"|'(?:\\.|[^'])*'|\(.*?\)|[^;])*/g, r;
			for (; r = n.exec(t.style);) e.style.removeProperty(r[1]);
		}
		n.style && (e.style.cssText += n.style);
	}
}
function Of(e, t, n) {
	return Ef(e, e, wf, Tf(t, n, !H(e)));
}
function kf(e, t) {
	if (e.length != t.length) return !1;
	for (let n = 0; n < e.length; n++) if (!e[n].type.eq(t[n].type)) return !1;
	return !0;
}
function Af(e) {
	let t = e.nextSibling;
	return e.parentNode.removeChild(e), t;
}
var jf = class {
	constructor(e, t, n) {
		this.lock = t, this.view = n, this.index = 0, this.stack = [], this.changed = !1, this.top = e, this.preMatch = Mf(e.node.content, e);
	}
	destroyBetween(e, t) {
		if (e != t) {
			for (let n = e; n < t; n++) this.top.children[n].destroy();
			this.top.children.splice(e, t - e), this.changed = !0;
		}
	}
	destroyRest() {
		this.destroyBetween(this.index, this.top.children.length);
	}
	syncToMarks(e, t, n, r) {
		let i = 0, a = this.stack.length >> 1, o = Math.min(a, e.length);
		for (; i < o && (i == a - 1 ? this.top : this.stack[i + 1 << 1]).matchesMark(e[i]) && e[i].type.spec.spanning !== !1;) i++;
		for (; i < a;) this.destroyRest(), this.top.dirty = lf, this.index = this.stack.pop(), this.top = this.stack.pop(), a--;
		for (; a < e.length;) {
			this.stack.push(this.top, this.index + 1);
			let i = -1, o = this.top.children.length;
			r < this.preMatch.index && (o = Math.min(this.index + 3, o));
			for (let t = this.index; t < o; t++) {
				let n = this.top.children[t];
				if (n.matchesMark(e[a]) && !this.isLocked(n.dom)) {
					i = t;
					break;
				}
			}
			if (i < 0 && this.index < this.top.children.length) {
				let t = this.top.children[this.index];
				t instanceof gf && t.dirty != ff && t.mark.type == e[a].type && t.spec.update && !this.isLocked(t.dom) && t.spec.update(e[a]) && (t.mark = e[a], i = this.index, this.changed = !0);
			}
			if (i > -1) i > this.index && (this.changed = !0, this.destroyBetween(this.index, i)), this.top = this.top.children[this.index];
			else {
				let r = gf.create(this.top, e[a], t, n);
				this.top.children.splice(this.index, 0, r), this.top = r, this.changed = !0;
			}
			this.index = 0, a++;
		}
	}
	findNodeMatch(e, t, n, r) {
		let i = -1, a;
		if (r >= this.preMatch.index && (a = this.preMatch.matches[r - this.preMatch.index]).parent == this.top && a.matchesNode(e, t, n)) i = this.top.children.indexOf(a, this.index);
		else for (let r = this.index, a = Math.min(this.top.children.length, r + 5); r < a; r++) {
			let a = this.top.children[r];
			if (a.matchesNode(e, t, n) && !this.preMatch.matched.has(a)) {
				i = r;
				break;
			}
		}
		return i < 0 ? !1 : (this.destroyBetween(this.index, i), this.index++, !0);
	}
	updateNodeAt(e, t, n, r, i) {
		let a = this.top.children[r];
		return a.dirty == ff && a.dom == a.contentDOM && (a.dirty = df), a.update(e, t, n, i) ? (this.destroyBetween(this.index, r), this.index++, !0) : !1;
	}
	findIndexWithChild(e) {
		for (;;) {
			let t = e.parentNode;
			if (!t) return -1;
			if (t == this.top.contentDOM) {
				let t = e.pmViewDesc;
				if (t) {
					for (let e = this.index; e < this.top.children.length; e++) if (this.top.children[e] == t) return e;
				}
				return -1;
			}
			e = t;
		}
	}
	updateNextNode(e, t, n, r, i, a) {
		for (let o = this.index; o < this.top.children.length; o++) {
			let s = this.top.children[o];
			if (s instanceof _f) {
				let c = this.preMatch.matched.get(s);
				if (c != null && c != i) return !1;
				let l = s.dom, u, d = this.isLocked(l) && !(e.isText && s.node && s.node.isText && s.nodeDOM.nodeValue == e.text && s.dirty != ff && kf(t, s.outerDeco));
				if (!d && s.update(e, t, n, r)) return this.destroyBetween(this.index, o), s.dom != l && (this.changed = !0), this.index++, !0;
				if (!d && (u = this.recreateWrapper(s, e, t, n, r, a))) return this.destroyBetween(this.index, o), this.top.children[this.index] = u, u.contentDOM && (u.dirty = df, u.updateChildren(r, a + 1), u.dirty = lf), this.changed = !0, this.index++, !0;
				break;
			}
		}
		return !1;
	}
	recreateWrapper(e, t, n, r, i, a) {
		if (e.dirty || t.isAtom || !e.children.length || !e.node.content.eq(t.content) || !kf(n, e.outerDeco) || !r.eq(e.innerDeco)) return null;
		let o = _f.create(this.top, t, n, r, i, a);
		if (o.contentDOM) {
			o.children = e.children, e.children = [];
			for (let e of o.children) e.parent = o;
		}
		return e.destroy(), o;
	}
	addNode(e, t, n, r, i) {
		let a = _f.create(this.top, e, t, n, r, i);
		a.contentDOM && a.updateChildren(r, i + 1), this.top.children.splice(this.index++, 0, a), this.changed = !0;
	}
	placeWidget(e, t, n) {
		let r = this.index < this.top.children.length ? this.top.children[this.index] : null;
		if (r && r.matchesWidget(e) && (e == r.widget || !r.widget.type.toDOM.parentNode)) this.index++;
		else {
			let r = new mf(this.top, e, t, n);
			this.top.children.splice(this.index++, 0, r), this.changed = !0;
		}
	}
	addTextblockHacks() {
		let e = this.top.children[this.index - 1], t = this.top;
		for (; e instanceof gf;) t = e, e = t.children[t.children.length - 1];
		(!e || !(e instanceof yf) || /\n$/.test(e.node.text) || this.view.requiresGeckoHackNode && /\s$/.test(e.node.text)) && ((G || W) && e && e.dom.contentEditable == "false" && this.addHackNode("IMG", t), this.addHackNode("BR", this.top));
	}
	addHackNode(e, t) {
		if (t == this.top && this.index < t.children.length && t.children[this.index].matchesHack(e)) this.index++;
		else {
			let n = document.createElement(e);
			e == "IMG" && (n.className = "ProseMirror-separator", n.alt = ""), e == "BR" && (n.className = "ProseMirror-trailingBreak");
			let r = new bf(this.top, [], n, null);
			t == this.top ? t.children.splice(this.index++, 0, r) : t.children.push(r), this.changed = !0;
		}
	}
	isLocked(e) {
		return this.lock && (e == this.lock || H(e) && e.contains(this.lock.parentNode));
	}
};
function Mf(e, t) {
	let n = t, r = n.children.length, i = e.childCount, a = /* @__PURE__ */ new Map(), o = [];
	outer: for (; i > 0;) {
		let s;
		for (;;) if (r) {
			let e = n.children[r - 1];
			if (e instanceof gf) n = e, r = e.children.length;
			else {
				s = e, r--;
				break;
			}
		} else if (n == t) break outer;
		else r = n.parent.children.indexOf(n), n = n.parent;
		let c = s.node;
		if (c) {
			if (c != e.child(i - 1)) break;
			--i, a.set(s, i), o.push(s);
		}
	}
	return {
		index: i,
		matched: a,
		matches: o.reverse()
	};
}
function Nf(e, t) {
	return e.type.side - t.type.side;
}
function Pf(e, t, n, r) {
	let i = t.locals(e), a = 0;
	if (i.length == 0) {
		for (let n = 0; n < e.childCount; n++) {
			let o = e.child(n);
			r(o, i, t.forChild(a, o), n), a += o.nodeSize;
		}
		return;
	}
	let o = 0, s = [], c = null;
	for (let l = 0;;) {
		let u, d;
		for (; o < i.length && i[o].to == a;) {
			let e = i[o++];
			e.widget && (u ? (d ||= [u]).push(e) : u = e);
		}
		if (u) {
			if (d) {
				d.sort(Nf);
				for (let e = 0; e < d.length; e++) n(d[e], l, !!c);
			} else n(u, l, !!c);
		}
		let f, p;
		if (c) p = -1, f = c, c = null;
		else if (l < e.childCount) p = l, f = e.child(l++);
		else break;
		for (let e = 0; e < s.length; e++) s[e].to <= a && s.splice(e--, 1);
		for (; o < i.length && i[o].from <= a && i[o].to > a;) s.push(i[o++]);
		let m = a + f.nodeSize;
		if (f.isText) {
			let e = m;
			o < i.length && i[o].from < e && (e = i[o].from);
			for (let t = 0; t < s.length; t++) s[t].to < e && (e = s[t].to);
			e < m && (c = f.cut(e - a), f = f.cut(0, e - a), m = e, p = -1);
		} else for (; o < i.length && i[o].to < m;) o++;
		let h = f.isInline && !f.isLeaf ? s.filter((e) => !e.inline) : s.slice();
		r(f, h, t.forChild(a, f), p), a = m;
	}
}
function Ff(e) {
	if (e.nodeName == "UL" || e.nodeName == "OL") {
		let t = e.style.cssText;
		e.style.cssText = t + "; list-style: square !important", window.getComputedStyle(e).listStyle, e.style.cssText = t;
	}
}
function If(e, t, n, r) {
	for (let i = 0, a = 0; i < e.childCount && a <= r;) {
		let o = e.child(i++), s = a;
		if (a += o.nodeSize, !o.isText) continue;
		let c = o.text;
		for (; i < e.childCount;) {
			let t = e.child(i++);
			if (a += t.nodeSize, !t.isText) break;
			c += t.text;
		}
		if (a >= n) {
			if (a >= r && c.slice(r - t.length - s, r - s) == t) return r - t.length;
			let e = s < r ? c.lastIndexOf(t, r - s - 1) : -1;
			if (e >= 0 && e + t.length + s >= n) return s + e;
			if (n == r && c.length >= r + t.length - s && c.slice(r - s, r - s + t.length) == t) return r;
		}
	}
	return -1;
}
function Lf(e, t, n, r, i) {
	let a = [];
	for (let o = 0, s = 0; o < e.length; o++) {
		let c = e[o], l = s, u = s += c.size;
		l >= n || u <= t ? a.push(c) : (l < t && a.push(c.slice(0, t - l, r)), i &&= (a.push(i), void 0), u > n && a.push(c.slice(n - l, c.size, r)));
	}
	return a;
}
function Rf(e, t = null) {
	let n = e.domSelectionRange(), r = e.state.doc;
	if (!n.focusNode) return null;
	let i = e.docView.nearestDesc(n.focusNode), a = i && i.size == 0, o = e.docView.posFromDOM(n.focusNode, n.focusOffset, 1);
	if (o < 0) return null;
	let s = r.resolve(o), c, l;
	if (ld(n)) {
		for (c = o; i && !i.node;) i = i.parent;
		if (i && i.node.isAtom && z.isSelectable(i.node) && i.parent && !(i.node.isInline && sd(n.focusNode, n.focusOffset, i.dom))) {
			let e = i.posBefore;
			l = new z(o == e ? s : r.resolve(e));
		} else if (xd && t == "key" && o > e.state.selection.to && H(n.focusNode) && n.focusOffset == n.focusNode.childNodes.length && i && i.posAtStart > e.state.selection.to) for (let e = n.focusNode;; e = e.parentNode) {
			if (getComputedStyle(e).overflow != "visible") {
				l = L.findFrom(r.resolve(i.posAtStart), 1);
				break;
			}
			if (e == i.dom || e.nextSibling || !e.parentNode || !H(e.parentNode)) break;
		}
	} else {
		if (n instanceof e.dom.ownerDocument.defaultView.Selection && n.rangeCount > 1) {
			let t = o, i = o;
			for (let r = 0; r < n.rangeCount; r++) {
				let a = n.getRangeAt(r);
				t = Math.min(t, e.docView.posFromDOM(a.startContainer, a.startOffset, 1)), i = Math.max(i, e.docView.posFromDOM(a.endContainer, a.endOffset, -1));
			}
			if (t < 0) return null;
			[c, o] = i == e.state.selection.anchor ? [i, t] : [t, i], s = r.resolve(o);
		} else c = e.docView.posFromDOM(n.anchorNode, n.anchorOffset, 1);
		if (c < 0) return null;
	}
	let u = r.resolve(c);
	if (!l) {
		let n = t == "pointer" || e.state.selection.head < s.pos && !a ? 1 : -1;
		l = Yf(e, u, s, n);
	}
	return l;
}
function zf(e) {
	return e.editable ? e.hasFocus() : Zf(e) && document.activeElement && document.activeElement.contains(e.dom);
}
function Bf(e, t = !1) {
	let n = e.state.selection;
	if (qf(e, n), !zf(e)) return;
	let r = e.input.mouseDown;
	if (!t && W && r) {
		let t = e.domSelectionRange(), n = e.domObserver.currentSelection;
		if (t.anchorNode && n.anchorNode && td(t.anchorNode, t.anchorOffset, n.anchorNode, n.anchorOffset) && r.delaySelUpdate()) {
			e.domObserver.setCurSelection();
			return;
		}
	}
	if (e.domObserver.disconnectSelection(), e.cursorWrapper) Kf(e);
	else {
		let { anchor: r, head: i } = n, a, o;
		Vf && !(n instanceof R) && (n.$from.parent.inlineContent || (a = Hf(e, n.from)), !n.empty && !n.$from.parent.inlineContent && (o = Hf(e, n.to))), e.docView.setSelection(r, i, e, t), Vf && (a && Wf(a), o && Wf(o)), n.visible ? e.dom.classList.remove("ProseMirror-hideselection") : (e.dom.classList.add("ProseMirror-hideselection"), "onselectionchange" in document && Gf(e));
	}
	e.domObserver.setCurSelection(), e.domObserver.connectSelection();
}
var Vf = G || W && Cd < 63;
function Hf(e, t) {
	let { node: n, offset: r } = e.docView.domFromPos(t, 0), i = r < n.childNodes.length ? n.childNodes[r] : null, a = r ? n.childNodes[r - 1] : null;
	if (G && i && i.contentEditable == "false") return Uf(i);
	if ((!i || i.contentEditable == "false") && (!a || a.contentEditable == "false")) {
		if (i) return Uf(i);
		if (a) return Uf(a);
	}
}
function Uf(e) {
	return e.contentEditable = "true", G && e.draggable && (e.draggable = !1, e.wasDraggable = !0), e;
}
function Wf(e) {
	e.contentEditable = "false", e.wasDraggable &&= (e.draggable = !0, null);
}
function Gf(e) {
	let t = e.dom.ownerDocument;
	t.removeEventListener("selectionchange", e.input.hideSelectionGuard);
	let n = e.domSelectionRange(), r = n.anchorNode, i = n.anchorOffset;
	t.addEventListener("selectionchange", e.input.hideSelectionGuard = () => {
		(n.anchorNode != r || n.anchorOffset != i) && (t.removeEventListener("selectionchange", e.input.hideSelectionGuard), setTimeout(() => {
			(!zf(e) || e.state.selection.visible) && e.dom.classList.remove("ProseMirror-hideselection");
		}, 20));
	});
}
function Kf(e) {
	let t = e.domSelection();
	if (!t) return;
	let n = e.cursorWrapper.dom, r = n.nodeName == "IMG";
	r ? t.collapse(n.parentNode, B(n) + 1) : t.collapse(n, 0), !r && !e.state.selection.visible && yd && bd <= 11 && (n.disabled = !0, n.disabled = !1);
}
function qf(e, t) {
	if (t instanceof z) {
		let n = e.docView.descAt(t.from);
		n != e.lastSelectedViewDesc && (Jf(e), n && n.selectNode(), e.lastSelectedViewDesc = n);
	} else Jf(e);
}
function Jf(e) {
	e.lastSelectedViewDesc &&= (e.lastSelectedViewDesc.parent && e.lastSelectedViewDesc.deselectNode(), void 0);
}
function Yf(e, t, n, r) {
	return e.someProp("createSelectionBetween", (r) => r(e, t, n)) || R.between(t, n, r);
}
function Xf(e) {
	return e.editable && !e.hasFocus() ? !1 : Zf(e);
}
function Zf(e) {
	let t = e.domSelectionRange();
	if (!t.anchorNode) return !1;
	try {
		return id(e.dom, t.anchorNode) && (e.editable || id(e.dom, t.focusNode));
	} catch {
		return !1;
	}
}
function Qf(e) {
	let t = e.docView.domFromPos(e.state.selection.anchor, 0), n = e.domSelectionRange();
	return td(t.node, t.offset, n.anchorNode, n.anchorOffset);
}
function $f(e, t) {
	let { $anchor: n, $head: r } = e.selection, i = t > 0 ? n.max(r) : n.min(r), a = i.parent.inlineContent ? i.depth ? e.doc.resolve(t > 0 ? i.after() : i.before()) : null : i;
	return a && L.findFrom(a, t);
}
function ep(e, t) {
	return e.dispatch(e.state.tr.setSelection(t).scrollIntoView()), !0;
}
function tp(e, t, n) {
	let r = e.state.selection;
	if (r instanceof R) {
		if (n.indexOf("s") > -1) {
			let { $head: n } = r, i = n.textOffset ? null : t < 0 ? n.nodeBefore : n.nodeAfter;
			if (!i || i.isText || !i.isLeaf) return !1;
			let a = e.state.doc.resolve(n.pos + i.nodeSize * (t < 0 ? -1 : 1));
			return ep(e, new R(r.$anchor, a));
		}
		if (!r.empty) return !1;
		if (e.endOfTextblock(t > 0 ? "forward" : "backward")) {
			let n = $f(e.state, t);
			return n && n instanceof z ? ep(e, n) : !1;
		}
		if (!(Td && n.indexOf("m") > -1)) {
			let n = r.$head, i = n.textOffset ? null : t < 0 ? n.nodeBefore : n.nodeAfter, a;
			if (!i || i.isText) return !1;
			let o = t < 0 ? n.pos - i.nodeSize : n.pos;
			return i.isAtom || (a = e.docView.descAt(o)) && !a.contentDOM ? z.isSelectable(i) ? ep(e, new z(t < 0 ? e.state.doc.resolve(n.pos - i.nodeSize) : n)) : ep(e, new R(e.state.doc.resolve(t < 0 ? o : o + i.nodeSize))) : !1;
		}
	} else if (r instanceof z && r.node.isInline) return ep(e, new R(t > 0 ? r.$to : r.$from));
	else {
		let n = $f(e.state, t);
		return n ? ep(e, n) : !1;
	}
}
function np(e, t) {
	let n = e.pmViewDesc;
	return n ? n.size == 0 && (t < 0 || e.nextSibling || e.nodeName != "BR") : H(e) && e.contentEditable == "false";
}
function rp(e, t) {
	return t < 0 ? ip(e) : ap(e);
}
function ip(e) {
	let t = e.domSelectionRange(), n = t.focusNode, r = t.focusOffset;
	if (!n) return;
	let i, a, o = !1;
	for (xd && H(n) && r < V(n) && np(n.childNodes[r], -1) && (o = !0);;) if (r > 0) {
		if (!H(n)) break;
		let e = n.childNodes[r - 1];
		if (np(e, -1)) i = n, a = --r;
		else if (U(e)) n = e, r = n.nodeValue.length;
		else break;
	} else if (op(n)) break;
	else {
		let t = n.previousSibling;
		for (; t && np(t, -1);) i = n.parentNode, a = B(t), t = t.previousSibling;
		if (t) n = t, r = V(n);
		else {
			if (n = n.parentNode, n == e.dom) break;
			r = 0;
		}
	}
	o ? lp(e, n, r) : i && lp(e, i, a);
}
function ap(e) {
	let t = e.domSelectionRange(), n = t.focusNode, r = t.focusOffset;
	if (!n) return;
	let i = V(n), a, o;
	for (;;) if (r < i) {
		if (!H(n)) break;
		let e = n.childNodes[r];
		if (np(e, 1)) a = n, o = ++r;
		else break;
	} else if (op(n)) break;
	else {
		let t = n.nextSibling;
		for (; t && np(t, 1);) a = t.parentNode, o = B(t) + 1, t = t.nextSibling;
		if (t) n = t, r = 0, i = V(n);
		else {
			if (n = n.parentNode, n == e.dom) break;
			r = i = 0;
		}
	}
	a && lp(e, a, o);
}
function op(e) {
	let t = e.pmViewDesc;
	return t && t.node && t.node.isBlock;
}
function sp(e, t) {
	for (; e && t == e.childNodes.length && !cd(e);) t = B(e) + 1, e = e.parentNode;
	for (; e && t < e.childNodes.length;) {
		let n = e.childNodes[t];
		if (U(n)) return n;
		if (H(n) && n.contentEditable == "false") break;
		e = n, t = 0;
	}
}
function cp(e, t) {
	for (; e && !t && !cd(e);) t = B(e), e = e.parentNode;
	for (; e && t;) {
		let n = e.childNodes[t - 1];
		if (U(n)) return n;
		if (H(n) && n.contentEditable == "false") break;
		e = n, t = e.childNodes.length;
	}
}
function lp(e, t, n) {
	if (!U(t)) {
		let e, r;
		(r = sp(t, n)) ? (t = r, n = 0) : (e = cp(t, n)) && (t = e, n = e.nodeValue.length);
	}
	let r = e.domSelection();
	if (!r) return;
	if (ld(r)) {
		let e = document.createRange();
		e.setEnd(t, n), e.setStart(t, n), r.removeAllRanges(), r.addRange(e);
	} else r.extend && r.extend(t, n);
	e.domObserver.setCurSelection();
	let { state: i } = e;
	setTimeout(() => {
		e.state == i && Bf(e);
	}, 50);
}
function up(e, t) {
	let n = e.state.doc.resolve(t);
	if (!(W || Ed) && n.parent.inlineContent) {
		let r = e.coordsAtPos(t);
		if (t > n.start()) {
			let n = e.coordsAtPos(t - 1), i = (n.top + n.bottom) / 2;
			if (i > r.top && i < r.bottom && Math.abs(n.left - r.left) > 1) return n.left < r.left ? "ltr" : "rtl";
		}
		if (t < n.end()) {
			let n = e.coordsAtPos(t + 1), i = (n.top + n.bottom) / 2;
			if (i > r.top && i < r.bottom && Math.abs(n.left - r.left) > 1) return n.left > r.left ? "ltr" : "rtl";
		}
	}
	return getComputedStyle(e.dom).direction == "rtl" ? "rtl" : "ltr";
}
function dp(e, t, n) {
	let r = e.state.selection;
	if (r instanceof R && !r.empty || n.indexOf("s") > -1 || Td && n.indexOf("m") > -1) return !1;
	let { $from: i, $to: a } = r;
	if (!i.parent.inlineContent || e.endOfTextblock(t < 0 ? "up" : "down")) {
		let n = $f(e.state, t);
		if (n && n instanceof z) return ep(e, n);
	}
	if (!i.parent.inlineContent) {
		let n = t < 0 ? i : a, o = r instanceof tl ? L.near(n, t) : L.findFrom(n, t);
		return o ? ep(e, o) : !1;
	}
	return !1;
}
function fp(e, t) {
	if (!(e.state.selection instanceof R)) return !0;
	let { $head: n, $anchor: r, empty: i } = e.state.selection;
	if (!n.sameParent(r)) return !0;
	if (!i) return !1;
	if (e.endOfTextblock(t > 0 ? "forward" : "backward")) return !0;
	let a = !n.textOffset && (t < 0 ? n.nodeBefore : n.nodeAfter);
	if (a && !a.isText) {
		let r = e.state.tr;
		return t < 0 ? r.delete(n.pos - a.nodeSize, n.pos) : r.delete(n.pos, n.pos + a.nodeSize), e.dispatch(r), !0;
	}
	return !1;
}
function pp(e, t, n) {
	e.domObserver.stop(), t.contentEditable = n, e.domObserver.start();
}
function mp(e) {
	if (!G || e.state.selection.$head.parentOffset > 0) return !1;
	let { focusNode: t, focusOffset: n } = e.domSelectionRange();
	if (t && H(t) && n == 0 && t.firstChild && H(t.firstChild) && t.firstChild.contentEditable == "false") {
		let n = t.firstChild;
		pp(e, n, "true"), setTimeout(() => pp(e, n, "false"), 20);
	}
	return !1;
}
function hp(e) {
	let t = "";
	return e.ctrlKey && (t += "c"), e.metaKey && (t += "m"), e.altKey && (t += "a"), e.shiftKey && (t += "s"), t;
}
function gp(e, t) {
	let n = t.keyCode, r = hp(t);
	if (n == 8 || Td && n == 72 && r == "c") return fp(e, -1) || rp(e, -1);
	if (n == 46 && !t.shiftKey || Td && n == 68 && r == "c") return fp(e, 1) || rp(e, 1);
	if (n == 13 || n == 27) return !0;
	if (n == 37 || Td && n == 66 && r == "c") {
		let t = n == 37 ? up(e, e.state.selection.from) == "ltr" ? -1 : 1 : -1;
		return tp(e, t, r) || rp(e, t);
	}
	if (n == 39 || Td && n == 70 && r == "c") {
		let t = n == 39 ? up(e, e.state.selection.from) == "ltr" ? 1 : -1 : 1;
		return tp(e, t, r) || rp(e, t);
	}
	return n == 38 || Td && n == 80 && r == "c" ? dp(e, -1, r) || rp(e, -1) : n == 40 || Td && n == 78 && r == "c" ? mp(e) || dp(e, 1, r) || rp(e, 1) : !(r != (Td ? "m" : "c") || n != 66 && n != 73 && n != 89 && n != 90);
}
function _p(e, t) {
	e.someProp("transformCopied", (n) => {
		t = n(t, e);
	});
	let n = [], { content: r, openStart: i, openEnd: a } = t;
	for (; i > 1 && a > 1 && r.childCount == 1 && r.firstChild.childCount == 1;) {
		i--, a--;
		let e = r.firstChild;
		n.push(e.type.name, e.attrs == e.type.defaultAttrs ? null : e.attrs), r = e.content;
	}
	let o = e.someProp("clipboardSerializer") || As.fromSchema(e.state.schema), s = Dp(), c = s.createElement("div");
	c.appendChild(o.serializeFragment(r, { document: s }));
	let l = c.firstChild, u, d = 0;
	for (; l && H(l) && (u = Ep[l.nodeName.toLowerCase()]);) {
		for (let e = u.length - 1; e >= 0; e--) {
			let t = s.createElement(u[e]);
			for (; c.firstChild;) t.appendChild(c.firstChild);
			c.appendChild(t), d++;
		}
		l = c.firstChild;
	}
	return l && H(l) && l.setAttribute("data-pm-slice", `${i} ${a}${d ? ` -${d}` : ""} ${JSON.stringify(n)}`), {
		dom: c,
		text: e.someProp("clipboardTextSerializer", (n) => n(t, e)) || t.content.textBetween(0, t.content.size, "\n\n"),
		slice: t
	};
}
function vp(e, t, n, r, i) {
	let a = i.parent.type.spec.code, o, s;
	if (!n && !t) return null;
	let c = !!t && (r || a || !n);
	if (c) {
		if (e.someProp("transformPastedText", (n) => {
			t = n(t, a || r, e);
		}), a) return s = new F(N.from(e.state.schema.text(t.replace(/\r\n?/g, "\n"))), 0, 0), e.someProp("transformPasted", (t) => {
			s = t(s, e, !0);
		}), s;
		let n = e.someProp("clipboardTextParser", (n) => n(t, i, r, e));
		if (n) s = n;
		else {
			let n = i.marks(), { schema: r } = e.state, a = As.fromSchema(r);
			o = document.createElement("div"), t.split(/(?:\r\n?|\n)+/).forEach((e) => {
				let t = o.appendChild(document.createElement("p"));
				e && t.appendChild(a.serializeNode(r.text(e, n)));
			});
		}
	} else e.someProp("transformPastedHTML", (t) => {
		n = t(n, e);
	}), o = Ap(n), Od && jp(o);
	let l = o && o.querySelector("[data-pm-slice]"), u = l && /^(\d+) (\d+)(?: -(\d+))? (.*)/.exec(l.getAttribute("data-pm-slice") || "");
	if (u && u[3]) for (let e = +u[3]; e > 0; e--) {
		let e = o.firstChild;
		for (; e && !H(e);) e = e.nextSibling;
		if (!e) break;
		o = e;
	}
	if (s ||= (e.someProp("clipboardParser") || e.someProp("domParser") || gs.fromSchema(e.state.schema)).parseSlice(o, {
		preserveWhitespace: !!(c || u),
		context: i,
		ruleFromNode(e) {
			return e.nodeName == "BR" && !e.nextSibling && e.parentNode && !yp.test(e.parentNode.nodeName) ? { ignore: !0 } : null;
		}
	}), u) s = Mp(Tp(s, +u[1], +u[2]), u[4]);
	else if (s = F.maxOpen(bp(s.content, i), !0), s.openStart || s.openEnd) {
		let e = 0, t = 0;
		for (let t = s.content.firstChild; e < s.openStart && !t.type.spec.isolating; e++, t = t.firstChild);
		for (let e = s.content.lastChild; t < s.openEnd && !e.type.spec.isolating; t++, e = e.lastChild);
		s = Tp(s, e, t);
	}
	return e.someProp("transformPasted", (t) => {
		s = t(s, e, c);
	}), s;
}
var yp = /^(a|abbr|acronym|b|cite|code|del|em|i|ins|kbd|label|output|q|ruby|s|samp|span|strong|sub|sup|time|u|tt|var)$/i;
function bp(e, t) {
	if (e.childCount < 2) return e;
	for (let n = t.depth; n >= 0; n--) {
		let r = t.node(n).contentMatchAt(t.index(n)), i, a = [];
		if (e.forEach((e) => {
			if (!a) return;
			let t = r.findWrapping(e.type), n;
			if (!t) return a = null;
			if (n = a.length && i.length && Sp(t, i, e, a[a.length - 1], 0)) a[a.length - 1] = n;
			else {
				a.length && (a[a.length - 1] = Cp(a[a.length - 1], i.length));
				let n = xp(e, t);
				a.push(n), r = r.matchType(n.type), i = t;
			}
		}), a) return N.from(a);
	}
	return e;
}
function xp(e, t, n = 0) {
	for (let r = t.length - 1; r >= n; r--) e = t[r].create(null, N.from(e));
	return e;
}
function Sp(e, t, n, r, i) {
	if (i < e.length && i < t.length && e[i] == t[i]) {
		let a = Sp(e, t, n, r.lastChild, i + 1);
		if (a) return r.copy(r.content.replaceChild(r.childCount - 1, a));
		if (r.contentMatchAt(r.childCount).matchType(i == e.length - 1 ? n.type : e[i + 1])) return r.copy(r.content.append(N.from(xp(n, e, i + 1))));
	}
}
function Cp(e, t) {
	if (t == 0) return e;
	let n = e.content.replaceChild(e.childCount - 1, Cp(e.lastChild, t - 1)), r = e.contentMatchAt(e.childCount).fillBefore(N.empty, !0);
	return e.copy(n.append(r));
}
function wp(e, t, n, r, i, a) {
	let o = t < 0 ? e.firstChild : e.lastChild, s = o.content;
	return e.childCount > 1 && (a = 0), i < r - 1 && (s = wp(s, t, n, r, i + 1, a)), i >= n && (s = t < 0 ? o.contentMatchAt(0).fillBefore(s, a <= i).append(s) : s.append(o.contentMatchAt(o.childCount).fillBefore(N.empty, !0))), e.replaceChild(t < 0 ? 0 : e.childCount - 1, o.copy(s));
}
function Tp(e, t, n) {
	return t < e.openStart && (e = new F(wp(e.content, -1, t, e.openStart, 0, e.openEnd), t, e.openEnd)), n < e.openEnd && (e = new F(wp(e.content, 1, n, e.openEnd, 0, 0), e.openStart, n)), e;
}
var Ep = {
	thead: ["table"],
	tbody: ["table"],
	tfoot: ["table"],
	caption: ["table"],
	colgroup: ["table"],
	col: ["table", "colgroup"],
	tr: ["table", "tbody"],
	td: [
		"table",
		"tbody",
		"tr"
	],
	th: [
		"table",
		"tbody",
		"tr"
	]
};
function Dp() {
	return document.implementation.createHTMLDocument("title");
}
var Op = null;
function kp(e) {
	let t = window.trustedTypes;
	if (!t) return e;
	if (!Op) {
		if (Op = t.defaultPolicy) try {
			return Op.createHTML(e);
		} catch {}
		Op = t.createPolicy("ProseMirrorClipboard", { createHTML: (e) => e });
	}
	return Op.createHTML(e);
}
function Ap(e) {
	let t = /^(\s*<meta [^>]*>)*/.exec(e);
	t && (e = e.slice(t[0].length));
	let n = Dp(), r = n.body, i = /<([a-z][^>\s]+)/i.exec(e), a;
	if ((a = i && Ep[i[1].toLowerCase()]) && (e = a.map((e) => "<" + e + ">").join("") + e + a.map((e) => "</" + e + ">").reverse().join("")), r.innerHTML = kp(e), a) for (let e = 0; e < a.length; e++) r = r.querySelector(a[e]) || r;
	for (let e = 0; e < n.styleSheets.length; e++) {
		let t = n.styleSheets[e];
		for (let e = 0; e < t.rules.length; e++) {
			let n = t.rules[e];
			if (n instanceof CSSStyleRule) {
				let e = r.querySelectorAll(n.selectorText);
				for (let t = 0; t < e.length; t++) e[t].style.cssText += n.style.cssText;
			}
		}
	}
	return r;
}
function jp(e) {
	let t = e.querySelectorAll(W ? "span:not([class]):not([style])" : "span.Apple-converted-space");
	for (let n = 0; n < t.length; n++) {
		let r = t[n];
		r.childNodes.length == 1 && r.textContent == "\xA0" && r.parentNode && r.parentNode.replaceChild(e.ownerDocument.createTextNode(" "), r);
	}
}
function Mp(e, t) {
	if (!e.size) return e;
	let n = e.content.firstChild.type.schema, r;
	try {
		r = JSON.parse(t);
	} catch {
		return e;
	}
	let { content: i, openStart: a, openEnd: o } = e;
	for (let e = r.length - 2; e >= 0; e -= 2) {
		let t = n.nodes[r[e]];
		if (!t || t.hasRequiredAttrs()) break;
		try {
			i = N.from(t.create(r[e + 1], i));
		} catch {
			break;
		}
		a++, o++;
	}
	return new F(i, a, o);
}
var Np = {}, Pp = {}, Fp = {
	touchstart: !0,
	touchmove: !0
}, Ip = class {
	constructor() {
		this.shiftKey = !1, this.mouseDown = null, this.lastKeyCode = null, this.lastKeyCodeTime = 0, this.lastClick = {
			time: 0,
			x: 0,
			y: 0,
			type: "",
			button: 0
		}, this.lastSelectionOrigin = null, this.lastSelectionTime = 0, this.lastIOSEnter = 0, this.lastIOSEnterFallbackTimeout = -1, this.lastFocus = 0, this.lastTouch = 0, this.lastChromeDelete = 0, this.composing = !1, this.compositionNode = null, this.composingTimeout = -1, this.compositionNodes = [], this.compositionEndedAt = -2e8, this.compositionID = 1, this.badSafariComposition = !1, this.compositionPendingChanges = 0, this.domChangeCount = 0, this.eventHandlers = Object.create(null), this.hideSelectionGuard = null;
	}
};
function Lp(e) {
	for (let t in Np) {
		let n = Np[t];
		e.dom.addEventListener(t, e.input.eventHandlers[t] = (t) => {
			Hp(e, t) && !Vp(e, t) && (e.editable || !(t.type in Pp)) && n(e, t);
		}, Fp[t] ? { passive: !0 } : void 0);
	}
	G && e.dom.addEventListener("input", () => null), Bp(e);
}
function Rp(e, t) {
	e.input.lastSelectionOrigin = t, e.input.lastSelectionTime = Date.now();
}
function zp(e) {
	e.input.mouseDown && e.input.mouseDown.done(), e.domObserver.stop();
	for (let t in e.input.eventHandlers) e.dom.removeEventListener(t, e.input.eventHandlers[t]);
	clearTimeout(e.input.composingTimeout), clearTimeout(e.input.lastIOSEnterFallbackTimeout);
}
function Bp(e) {
	e.someProp("handleDOMEvents", (t) => {
		for (let n in t) e.input.eventHandlers[n] || e.dom.addEventListener(n, e.input.eventHandlers[n] = (t) => Vp(e, t));
	});
}
function Vp(e, t) {
	return e.someProp("handleDOMEvents", (n) => {
		let r = n[t.type];
		return r ? r(e, t) || t.defaultPrevented : !1;
	});
}
function Hp(e, t) {
	if (!t.bubbles) return !0;
	if (t.defaultPrevented) return !1;
	for (let n = t.target; n != e.dom; n = n.parentNode) if (!n || n.nodeType == 11 || n.pmViewDesc && n.pmViewDesc.stopEvent(t)) return !1;
	return !0;
}
function Up(e, t) {
	!Vp(e, t) && Np[t.type] && (e.editable || !(t.type in Pp)) && Np[t.type](e, t);
}
Pp.keydown = (e, t) => {
	let n = t;
	if (e.input.shiftKey = n.keyCode == 16 || n.shiftKey, !om(e) && (e.input.lastKeyCode = n.keyCode, e.input.lastKeyCodeTime = Date.now(), !(Dd && W && n.keyCode == 13))) {
		if (n.keyCode != 229 && e.domObserver.forceFlush(), wd && n.keyCode == 13 && !n.ctrlKey && !n.altKey && !n.metaKey) {
			let t = Date.now();
			e.input.lastIOSEnter = t, e.input.lastIOSEnterFallbackTimeout = setTimeout(() => {
				e.input.lastIOSEnter == t && (e.someProp("handleKeyDown", (t) => t(e, ud(13, "Enter"))), e.input.lastIOSEnter = 0);
			}, 200);
		} else e.someProp("handleKeyDown", (t) => t(e, n)) || gp(e, n) ? n.preventDefault() : Rp(e, "key");
	}
}, Pp.keyup = (e, t) => {
	t.keyCode == 16 && (e.input.shiftKey = !1);
}, Pp.keypress = (e, t) => {
	let n = t;
	if (om(e) || !n.charCode || n.ctrlKey && !n.altKey || Td && n.metaKey) return;
	if (e.someProp("handleKeyPress", (t) => t(e, n))) {
		n.preventDefault();
		return;
	}
	let r = e.state.selection;
	if (!(r instanceof R) || !r.$from.sameParent(r.$to)) {
		let t = String.fromCharCode(n.charCode), i = () => e.state.tr.insertText(t).scrollIntoView();
		!/[\r\n]/.test(t) && !e.someProp("handleTextInput", (n) => n(e, r.$from.pos, r.$to.pos, t, i)) && e.dispatch(i()), n.preventDefault();
	}
};
function Wp(e) {
	return {
		left: e.clientX,
		top: e.clientY
	};
}
function Gp(e, t) {
	let n = t.x - e.clientX, r = t.y - e.clientY;
	return n * n + r * r < 100;
}
function Kp(e, t, n, r, i) {
	if (r == -1) return !1;
	let a = e.state.doc.resolve(r);
	for (let r = a.depth + 1; r > 0; r--) if (e.someProp(t, (t) => r > a.depth ? t(e, n, a.nodeAfter, a.before(r), i, !0) : t(e, n, a.node(r), a.before(r), i, !1))) return !0;
	return !1;
}
function qp(e, t, n) {
	if (e.focused || e.focus(), e.state.selection.eq(t)) return;
	let r = e.state.tr.setSelection(t);
	r.setMeta("pointer", !0), e.dispatch(r);
}
function Jp(e, t) {
	if (t == -1) return !1;
	let n = e.state.doc.resolve(t), r = n.nodeAfter;
	return r && r.isAtom && z.isSelectable(r) ? (qp(e, new z(n)), !0) : !1;
}
function Yp(e, t) {
	if (t == -1) return !1;
	let n = e.state.selection, r, i;
	n instanceof z && (r = n.node);
	let a = e.state.doc.resolve(t);
	for (let e = a.depth + 1; e > 0; e--) {
		let t = e > a.depth ? a.nodeAfter : a.node(e);
		if (z.isSelectable(t)) {
			i = r && n.$from.depth > 0 && e >= n.$from.depth && a.before(n.$from.depth + 1) == n.$from.pos ? a.before(n.$from.depth) : a.before(e);
			break;
		}
	}
	return i != null && (qp(e, z.create(e.state.doc, i)), !0);
}
function Xp(e, t, n, r, i) {
	return Kp(e, "handleClickOn", t, n, r) || e.someProp("handleClick", (n) => n(e, t, r)) || (i ? Yp(e, n) : Jp(e, n));
}
function Zp(e, t, n, r) {
	return Kp(e, "handleDoubleClickOn", t, n, r) || e.someProp("handleDoubleClick", (n) => n(e, t, r));
}
function Qp(e, t, n, r) {
	return Kp(e, "handleTripleClickOn", t, n, r) || e.someProp("handleTripleClick", (n) => n(e, t, r)) || $p(e, n, r);
}
function $p(e, t, n) {
	if (n.button != 0) return !1;
	let r = em(e, t, !0), i = e.state.doc;
	return r ? (qp(e, r), r instanceof R && i.eq(e.state.doc) && (e.input.mouseDown = new am(e, r)), !0) : !1;
}
function em(e, t, n) {
	let r = e.state.doc;
	if (t == -1) return r.inlineContent ? R.create(r, 0, r.content.size) : null;
	let i = r.resolve(t);
	for (let e = i.depth + 1; e > 0; e--) {
		let t = e > i.depth ? i.nodeAfter : i.node(e), a = i.before(e);
		if (t.inlineContent) return R.create(r, a + 1, a + 1 + t.content.size);
		if (n && z.isSelectable(t)) return z.create(r, a);
	}
	return null;
}
function tm(e) {
	return fm(e);
}
var nm = Td ? "metaKey" : "ctrlKey";
Np.mousedown = (e, t) => {
	let n = t;
	e.input.shiftKey = n.shiftKey;
	let r = tm(e), i = Date.now(), a = "singleClick";
	i - e.input.lastClick.time < 500 && Gp(n, e.input.lastClick) && !n[nm] && e.input.lastClick.button == n.button && (e.input.lastClick.type == "singleClick" ? a = "doubleClick" : e.input.lastClick.type == "doubleClick" && (a = "tripleClick")), e.input.lastClick = {
		time: i,
		x: n.clientX,
		y: n.clientY,
		type: a,
		button: n.button
	}, e.input.mouseDown && e.input.mouseDown.done();
	let o = e.posAtCoords(Wp(n));
	o && (a == "singleClick" ? e.input.mouseDown = new im(e, o, n, !!r) : (a == "doubleClick" ? Zp : Qp)(e, o.pos, o.inside, n) ? n.preventDefault() : Rp(e, "pointer"));
};
var rm = class {
	constructor(e) {
		this.view = e, this.mightDrag = null, e.root.addEventListener("mouseup", this.up = this.up.bind(this)), e.root.addEventListener("mousemove", this.move = this.move.bind(this));
	}
	up(e) {
		this.done();
	}
	move(e) {
		e.buttons == 0 && this.done();
	}
	done() {
		this.view.root.removeEventListener("mouseup", this.up), this.view.root.removeEventListener("mousemove", this.move), this.view.input.mouseDown == this && (this.view.input.mouseDown = null);
	}
	delaySelUpdate() {
		return !1;
	}
}, im = class extends rm {
	constructor(e, t, n, r) {
		super(e), this.pos = t, this.event = n, this.flushed = r, this.delayedSelectionSync = !1, this.startDoc = e.state.doc, this.selectNode = !!n[nm], this.allowDefault = n.shiftKey;
		let i, a;
		if (t.inside > -1) i = e.state.doc.nodeAt(t.inside), a = t.inside;
		else {
			let n = e.state.doc.resolve(t.pos);
			i = n.parent, a = n.depth ? n.before() : 0;
		}
		let o = r ? null : n.target, s = o ? e.docView.nearestDesc(o, !0) : null;
		this.target = s && H(s.nodeDOM) ? s.nodeDOM : null;
		let { selection: c } = e.state;
		n.button == 0 && (i.type.spec.draggable && i.type.spec.selectable !== !1 || c instanceof z && c.from <= a && c.to > a) && (this.mightDrag = {
			node: i,
			pos: a,
			addAttr: !(!this.target || this.target.draggable),
			setUneditable: !!(this.target && xd && !this.target.hasAttribute("contentEditable"))
		}), this.target && this.mightDrag && (this.mightDrag.addAttr || this.mightDrag.setUneditable) && (this.view.domObserver.stop(), this.mightDrag.addAttr && (this.target.draggable = !0), this.mightDrag.setUneditable && setTimeout(() => {
			this.view.input.mouseDown == this && this.target.setAttribute("contentEditable", "false");
		}, 20), this.view.domObserver.start()), Rp(e, "pointer");
	}
	done() {
		super.done(), this.mightDrag && this.target && (this.view.domObserver.stop(), this.mightDrag.addAttr && this.target.removeAttribute("draggable"), this.mightDrag.setUneditable && this.target.removeAttribute("contentEditable"), this.view.domObserver.start()), this.delayedSelectionSync && setTimeout(() => {
			this.view.isDestroyed || Bf(this.view);
		});
	}
	up(e) {
		if (this.done(), !this.view.dom.contains(e.target)) return;
		let t = this.pos;
		this.view.state.doc != this.startDoc && (t = this.view.posAtCoords(Wp(e))), this.updateAllowDefault(e), this.allowDefault || !t ? Rp(this.view, "pointer") : Xp(this.view, t.pos, t.inside, e, this.selectNode) ? e.preventDefault() : e.button == 0 && (this.flushed || G && this.mightDrag && !this.mightDrag.node.isAtom || W && !this.view.state.selection.visible && Math.min(Math.abs(t.pos - this.view.state.selection.from), Math.abs(t.pos - this.view.state.selection.to)) <= 2) ? (qp(this.view, L.near(this.view.state.doc.resolve(t.pos))), e.preventDefault()) : Rp(this.view, "pointer");
	}
	move(e) {
		this.updateAllowDefault(e), Rp(this.view, "pointer"), super.move(e);
	}
	updateAllowDefault(e) {
		!this.allowDefault && (Math.abs(this.event.x - e.clientX) > 4 || Math.abs(this.event.y - e.clientY) > 4) && (this.allowDefault = !0);
	}
	delaySelUpdate() {
		return this.allowDefault ? (this.delayedSelectionSync = !0, !0) : !1;
	}
}, am = class extends rm {
	constructor(e, t) {
		super(e), this.startSelection = t, this.startDoc = e.state.doc;
	}
	move(e) {
		if (e.buttons == 0 || this.view.isDestroyed || !this.view.state.doc.eq(this.startDoc)) {
			this.done();
			return;
		}
		e.preventDefault(), Rp(this.view, "pointer");
		let t = this.view.posAtCoords(Wp(e)), n = t && em(this.view, t.inside, !1);
		if (!n) return;
		let { doc: r } = this.view.state, i = this.startSelection, [a, o] = n.from < i.from ? [i.to, n.from] : [i.from, n.to];
		qp(this.view, R.create(r, a, o));
	}
};
Np.touchstart = (e) => {
	e.input.lastTouch = Date.now(), tm(e), Rp(e, "pointer");
}, Np.touchmove = (e) => {
	e.input.lastTouch = Date.now(), Rp(e, "pointer");
}, Np.contextmenu = (e) => tm(e);
function om(e, t) {
	return e.composing ? !0 : G && Math.abs(Date.now() - e.input.compositionEndedAt) < 500 ? (e.input.compositionEndedAt = -2e8, !0) : !1;
}
var sm = Dd ? 5e3 : -1;
Pp.compositionstart = Pp.compositionupdate = (e) => {
	if (!e.composing) {
		e.domObserver.flush();
		let { state: t } = e, n = t.selection.$to;
		if (t.selection instanceof R && t.selection.empty && (t.storedMarks || !n.textOffset && n.parentOffset && n.nodeBefore.marks.some((e) => e.type.spec.inclusive === !1) || W && Ed && cm(e))) e.markCursor = e.state.storedMarks || n.marks(), fm(e, !0), e.markCursor = null;
		else if (fm(e, !t.selection.empty), xd && t.selection.empty && n.parentOffset && !n.textOffset && n.nodeBefore.marks.length) {
			let t = e.domSelectionRange();
			for (let n = t.focusNode, r = t.focusOffset; n && H(n) && r != 0;) {
				let t = r < 0 ? n.lastChild : n.childNodes[r - 1];
				if (!t) break;
				if (U(t)) {
					let n = e.domSelection();
					n && n.collapse(t, t.nodeValue.length);
					break;
				}
				n = t, r = -1;
			}
		}
		e.input.composing = !0;
	}
	lm(e, sm);
};
function cm(e) {
	let { focusNode: t, focusOffset: n } = e.domSelectionRange();
	if (!t || !H(t) || n >= t.childNodes.length) return !1;
	let r = t.childNodes[n];
	return H(r) && r.contentEditable == "false";
}
Pp.compositionend = (e, t) => {
	e.composing && (e.input.composing = !1, e.input.compositionEndedAt = Date.now(), e.input.compositionPendingChanges = e.domObserver.pendingRecords().length ? e.input.compositionID : 0, e.input.compositionNode = null, e.input.badSafariComposition ? e.domObserver.forceFlush() : e.input.compositionPendingChanges && Promise.resolve().then(() => e.domObserver.flush()), e.input.compositionID++, lm(e, 20));
};
function lm(e, t) {
	clearTimeout(e.input.composingTimeout), t > -1 && (e.input.composingTimeout = setTimeout(() => fm(e), t));
}
function um(e) {
	for (e.composing && (e.input.composing = !1, e.input.compositionEndedAt = Date.now()); e.input.compositionNodes.length > 0;) e.input.compositionNodes.pop().markParentsDirty();
}
function dm(e) {
	let t = e.domSelectionRange();
	if (!t.focusNode) return null;
	let n = ad(t.focusNode, t.focusOffset), r = od(t.focusNode, t.focusOffset);
	if (n && r && n != r) {
		let t = r.pmViewDesc, i = e.domObserver.lastChangedTextNode;
		if (n == i || r == i) return i;
		if (!t || !t.isText(r.nodeValue)) return r;
		if (e.input.compositionNode == r) {
			let e = n.pmViewDesc;
			if (e && e.isText(n.nodeValue)) return r;
		}
	}
	return n || r;
}
function fm(e, t = !1) {
	if (!(Dd && e.domObserver.flushingSoon >= 0)) {
		if (e.domObserver.forceFlush(), um(e), t || e.docView && e.docView.dirty) {
			let n = Rf(e), r = e.state.selection;
			return n && !n.eq(r) ? e.dispatch(e.state.tr.setSelection(n)) : (e.markCursor || t) && !r.$from.node(r.$from.sharedDepth(r.to)).inlineContent ? e.dispatch(e.state.tr.deleteSelection()) : e.updateState(e.state), !0;
		}
		return !1;
	}
}
function pm(e, t) {
	if (!e.dom.parentNode) return;
	let n = e.dom.parentNode.appendChild(document.createElement("div"));
	n.appendChild(t), n.style.cssText = "position: fixed; left: -10000px; top: 10px";
	let r = getSelection(), i = document.createRange();
	i.selectNodeContents(t), e.dom.blur(), r.removeAllRanges(), r.addRange(i), setTimeout(() => {
		n.parentNode && n.parentNode.removeChild(n), e.focus();
	}, 50);
}
var mm = yd && bd < 15 || wd && kd < 604;
Np.copy = Pp.cut = (e, t) => {
	let n = t, r = e.state.selection, i = n.type == "cut";
	if (r.empty) return;
	let a = mm ? null : n.clipboardData, { dom: o, text: s } = _p(e, r.content());
	a ? (n.preventDefault(), a.clearData(), a.setData("text/html", o.innerHTML), a.setData("text/plain", s)) : pm(e, o), i && e.dispatch(e.state.tr.deleteSelection().scrollIntoView().setMeta("uiEvent", "cut"));
};
function hm(e) {
	return e.openStart == 0 && e.openEnd == 0 && e.content.childCount == 1 ? e.content.firstChild : null;
}
function gm(e, t) {
	if (!e.dom.parentNode) return;
	let n = e.input.shiftKey || e.state.selection.$from.parent.type.spec.code, r = e.dom.parentNode.appendChild(document.createElement(n ? "textarea" : "div"));
	n || (r.contentEditable = "true"), r.style.cssText = "position: fixed; left: -10000px; top: 10px", r.focus();
	let i = e.input.shiftKey && e.input.lastKeyCode != 45;
	setTimeout(() => {
		e.focus(), r.parentNode && r.parentNode.removeChild(r), n ? _m(e, r.value, null, i, t) : _m(e, r.textContent, r.innerHTML, i, t);
	}, 50);
}
function _m(e, t, n, r, i) {
	let a = vp(e, t, n, r, e.state.selection.$from);
	if (e.someProp("handlePaste", (t) => t(e, i, a || F.empty))) return !0;
	if (!a) return !1;
	let o = hm(a), s = o ? e.state.tr.replaceSelectionWith(o, r) : e.state.tr.replaceSelection(a);
	return e.dispatch(s.scrollIntoView().setMeta("paste", !0).setMeta("uiEvent", "paste")), !0;
}
function vm(e) {
	let t = e.getData("text/plain") || e.getData("Text");
	if (t) return t;
	let n = e.getData("text/uri-list");
	return n ? n.replace(/\r?\n/g, " ") : "";
}
Pp.paste = (e, t) => {
	let n = t;
	if (e.composing && !Dd) return;
	let r = mm ? null : n.clipboardData, i = e.input.shiftKey && e.input.lastKeyCode != 45;
	r && _m(e, vm(r), r.getData("text/html"), i, n) ? n.preventDefault() : gm(e, n);
};
var ym = class {
	constructor(e, t, n) {
		this.slice = e, this.move = t, this.node = n;
	}
}, bm = Td ? "altKey" : "ctrlKey";
function xm(e, t) {
	let n;
	return e.someProp("dragCopies", (e) => {
		n ||= e(t);
	}), n == null ? !t[bm] : !n;
}
Np.dragstart = (e, t) => {
	let n = t, r = e.input.mouseDown;
	if (r && r.done(), !n.dataTransfer) return;
	let i = e.state.selection, a = i.empty ? null : e.posAtCoords(Wp(n)), o;
	if (!(a && a.pos >= i.from && a.pos <= (i instanceof z ? i.to - 1 : i.to))) {
		if (r && r.mightDrag) o = z.create(e.state.doc, r.mightDrag.pos);
		else if (n.target && H(n.target)) {
			let t = e.docView.nearestDesc(n.target, !0);
			t && t.node.type.spec.draggable && t != e.docView && (o = z.create(e.state.doc, t.posBefore));
		}
	}
	let { dom: s, text: c, slice: l } = _p(e, (o || e.state.selection).content());
	(!n.dataTransfer.files.length || !W || Cd > 120) && n.dataTransfer.clearData(), n.dataTransfer.setData(mm ? "Text" : "text/html", s.innerHTML), n.dataTransfer.effectAllowed = "copyMove", mm || n.dataTransfer.setData("text/plain", c), e.dragging = new ym(l, xm(e, n), o);
}, Np.dragend = (e) => {
	let t = e.dragging;
	window.setTimeout(() => {
		e.dragging == t && (e.dragging = null);
	}, 50);
}, Pp.dragover = Pp.dragenter = (e, t) => t.preventDefault(), Pp.drop = (e, t) => {
	try {
		Sm(e, t, e.dragging);
	} finally {
		e.dragging = null;
	}
};
function Sm(e, t, n) {
	if (!t.dataTransfer) return;
	let r = e.posAtCoords(Wp(t));
	if (!r) return;
	let i = e.state.doc.resolve(r.pos), a = n && n.slice;
	a ? e.someProp("transformPasted", (t) => {
		a = t(a, e, !1);
	}) : a = vp(e, vm(t.dataTransfer), mm ? null : t.dataTransfer.getData("text/html"), !1, i);
	let o = !!(n && xm(e, t));
	if (e.someProp("handleDrop", (n) => n(e, t, a || F.empty, o))) {
		t.preventDefault();
		return;
	}
	if (!a) return;
	t.preventDefault();
	let s = a ? Oc(e.state.doc, i.pos, a) : i.pos;
	s ??= i.pos;
	let c = e.state.tr;
	if (o) {
		let { node: e } = n;
		e ? e.replace(c) : c.deleteSelection();
	}
	let l = c.mapping.map(s), u = a.openStart == 0 && a.openEnd == 0 && a.content.childCount == 1, d = c.doc;
	if (u ? c.replaceRangeWith(l, l, a.content.firstChild) : c.replaceRange(l, l, a), c.doc.eq(d)) return;
	let f = c.doc.resolve(l);
	if (u && z.isSelectable(a.content.firstChild) && f.nodeAfter && f.nodeAfter.sameMarkup(a.content.firstChild)) c.setSelection(new z(f));
	else {
		let t = c.mapping.map(s);
		c.mapping.maps[c.mapping.maps.length - 1].forEach((e, n, r, i) => t = i), c.setSelection(Yf(e, f, c.doc.resolve(t)));
	}
	e.focus(), e.dispatch(c.setMeta("uiEvent", "drop"));
}
Np.focus = (e) => {
	e.input.lastFocus = Date.now(), e.focused || (e.domObserver.stop(), e.dom.classList.add("ProseMirror-focused"), e.domObserver.start(), e.focused = !0, setTimeout(() => {
		e.docView && e.hasFocus() && !e.domObserver.currentSelection.eq(e.domSelectionRange()) && Bf(e);
	}, 20));
}, Np.blur = (e, t) => {
	let n = t;
	e.focused &&= (e.domObserver.stop(), e.dom.classList.remove("ProseMirror-focused"), e.domObserver.start(), n.relatedTarget && e.dom.contains(n.relatedTarget) && e.domObserver.currentSelection.clear(), !1);
}, Np.beforeinput = (e, t) => {
	if (Dd && t.inputType == "deleteContentBackward") {
		e.domObserver.flushSoon();
		let { domChangeCount: t } = e.input;
		setTimeout(() => {
			if (e.input.domChangeCount != t || (e.dom.blur(), e.focus(), e.someProp("handleKeyDown", (t) => t(e, ud(8, "Backspace"))))) return;
			let { $cursor: n } = e.state.selection;
			n && n.pos > 0 && e.dispatch(e.state.tr.delete(n.pos - 1, n.pos).scrollIntoView());
		}, 50);
	}
};
for (let e in Pp) Np[e] = Pp[e];
function Cm(e, t) {
	if (e == t) return !0;
	for (let n in e) if (e[n] !== t[n]) return !1;
	for (let n in t) if (!(n in e)) return !1;
	return !0;
}
var wm = class e {
	constructor(e, t) {
		this.toDOM = e, this.spec = t || km, this.side = this.spec.side || 0;
	}
	map(e, t, n, r) {
		let { pos: i, deleted: a } = e.mapResult(t.from + r, this.side < 0 ? -1 : 1);
		return a ? null : new Dm(i - n, i - n, this);
	}
	valid() {
		return !0;
	}
	eq(t) {
		return this == t || t instanceof e && (this.spec.key && this.spec.key == t.spec.key || this.toDOM == t.toDOM && Cm(this.spec, t.spec));
	}
	destroy(e) {
		this.spec.destroy && this.spec.destroy(e);
	}
}, Tm = class e {
	constructor(e, t) {
		this.attrs = e, this.spec = t || km;
	}
	map(e, t, n, r) {
		let i = e.map(t.from + r, this.spec.inclusiveStart ? -1 : 1) - n, a = e.map(t.to + r, this.spec.inclusiveEnd ? 1 : -1) - n;
		return i >= a ? null : new Dm(i, a, this);
	}
	valid(e, t) {
		return t.from < t.to;
	}
	eq(t) {
		return this == t || t instanceof e && Cm(this.attrs, t.attrs) && Cm(this.spec, t.spec);
	}
	static is(t) {
		return t.type instanceof e;
	}
	destroy() {}
}, Em = class e {
	constructor(e, t) {
		this.attrs = e, this.spec = t || km;
	}
	map(e, t, n, r) {
		let i = e.mapResult(t.from + r, 1);
		if (i.deleted) return null;
		let a = e.mapResult(t.to + r, -1);
		return a.deleted || a.pos <= i.pos ? null : new Dm(i.pos - n, a.pos - n, this);
	}
	valid(e, t) {
		let { index: n, offset: r } = e.content.findIndex(t.from), i;
		return r == t.from && !(i = e.child(n)).isText && r + i.nodeSize == t.to;
	}
	eq(t) {
		return this == t || t instanceof e && Cm(this.attrs, t.attrs) && Cm(this.spec, t.spec);
	}
	destroy() {}
}, Dm = class e {
	constructor(e, t, n) {
		this.from = e, this.to = t, this.type = n;
	}
	copy(t, n) {
		return new e(t, n, this.type);
	}
	eq(e, t = 0) {
		return this.type.eq(e.type) && this.from + t == e.from && this.to + t == e.to;
	}
	map(e, t, n) {
		return this.type.map(e, this, t, n);
	}
	static widget(t, n, r) {
		return new e(t, t, new wm(n, r));
	}
	static inline(t, n, r, i) {
		return new e(t, n, new Tm(r, i));
	}
	static node(t, n, r, i) {
		return new e(t, n, new Em(r, i));
	}
	get spec() {
		return this.type.spec;
	}
	get inline() {
		return this.type instanceof Tm;
	}
	get widget() {
		return this.type instanceof wm;
	}
}, Om = [], km = {}, K = class e {
	constructor(e, t) {
		this.local = e.length ? e : Om, this.children = t.length ? t : Om;
	}
	static create(e, t) {
		return t.length ? Lm(t, e, 0, km) : Am;
	}
	find(e, t, n) {
		let r = [];
		return this.findInner(e ?? 0, t ?? 1e9, r, 0, n), r;
	}
	findInner(e, t, n, r, i) {
		for (let a = 0; a < this.local.length; a++) {
			let o = this.local[a];
			o.from <= t && o.to >= e && (!i || i(o.spec)) && n.push(o.copy(o.from + r, o.to + r));
		}
		for (let a = 0; a < this.children.length; a += 3) if (this.children[a] < t && this.children[a + 1] > e) {
			let o = this.children[a] + 1;
			this.children[a + 2].findInner(e - o, t - o, n, r + o, i);
		}
	}
	map(e, t, n) {
		return this == Am || e.maps.length == 0 ? this : this.mapInner(e, t, 0, 0, n || km);
	}
	mapInner(t, n, r, i, a) {
		let o;
		for (let e = 0; e < this.local.length; e++) {
			let s = this.local[e].map(t, r, i);
			s && s.type.valid(n, s) ? (o ||= []).push(s) : a.onRemove && a.onRemove(this.local[e].spec);
		}
		return this.children.length ? Mm(this.children, o || [], t, n, r, i, a) : o ? new e(o.sort(Rm), Om) : Am;
	}
	add(t, n) {
		return n.length ? this == Am ? e.create(t, n) : this.addInner(t, n, 0) : this;
	}
	addInner(t, n, r) {
		let i, a = 0;
		t.forEach((e, t) => {
			let o = t + r, s;
			if (s = Fm(n, e, o)) {
				for (i ||= this.children.slice(); a < i.length && i[a] < t;) a += 3;
				i[a] == t ? i[a + 2] = i[a + 2].addInner(e, s, o + 1) : i.splice(a, 0, t, t + e.nodeSize, Lm(s, e, o + 1, km)), a += 3;
			}
		});
		let o = Nm(a ? Im(n) : n, -r);
		for (let e = 0; e < o.length; e++) o[e].type.valid(t, o[e]) || o.splice(e--, 1);
		return new e(o.length ? this.local.concat(o).sort(Rm) : this.local, i || this.children);
	}
	remove(e) {
		return e.length == 0 || this == Am ? this : this.removeInner(e, 0);
	}
	removeInner(t, n) {
		let r = this.children, i = this.local;
		for (let e = 0; e < r.length; e += 3) {
			let i, a = r[e] + n, o = r[e + 1] + n;
			for (let e = 0, n; e < t.length; e++) (n = t[e]) && n.from > a && n.to < o && (t[e] = null, (i ||= []).push(n));
			if (!i) continue;
			r == this.children && (r = this.children.slice());
			let s = r[e + 2].removeInner(i, a + 1);
			s == Am ? (r.splice(e, 3), e -= 3) : r[e + 2] = s;
		}
		if (i.length) {
			for (let e = 0, r; e < t.length; e++) if (r = t[e]) for (let e = 0; e < i.length; e++) i[e].eq(r, n) && (i == this.local && (i = this.local.slice()), i.splice(e--, 1));
		}
		return r == this.children && i == this.local ? this : i.length || r.length ? new e(i, r) : Am;
	}
	forChild(t, n) {
		if (this == Am) return this;
		if (n.isLeaf) return e.empty;
		let r, i;
		for (let e = 0; e < this.children.length; e += 3) if (this.children[e] >= t) {
			this.children[e] == t && (r = this.children[e + 2]);
			break;
		}
		let a = t + 1, o = a + n.content.size;
		for (let e = 0; e < this.local.length; e++) {
			let t = this.local[e];
			if (t.from < o && t.to > a && t.type instanceof Tm) {
				let e = Math.max(a, t.from) - a, n = Math.min(o, t.to) - a;
				e < n && (i ||= []).push(t.copy(e, n));
			}
		}
		if (i) {
			let t = new e(i.sort(Rm), Om);
			return r ? new jm([t, r]) : t;
		}
		return r || Am;
	}
	eq(t) {
		if (this == t) return !0;
		if (!(t instanceof e) || this.local.length != t.local.length || this.children.length != t.children.length) return !1;
		for (let e = 0; e < this.local.length; e++) if (!this.local[e].eq(t.local[e])) return !1;
		for (let e = 0; e < this.children.length; e += 3) if (this.children[e] != t.children[e] || this.children[e + 1] != t.children[e + 1] || !this.children[e + 2].eq(t.children[e + 2])) return !1;
		return !0;
	}
	locals(e) {
		return zm(this.localsInner(e));
	}
	localsInner(e) {
		if (this == Am) return Om;
		if (e.inlineContent || !this.local.some(Tm.is)) return this.local;
		let t = [];
		for (let e = 0; e < this.local.length; e++) this.local[e].type instanceof Tm || t.push(this.local[e]);
		return t;
	}
	forEachSet(e) {
		e(this);
	}
};
K.empty = new K([], []), K.removeOverlap = zm;
var Am = K.empty, jm = class e {
	constructor(e) {
		this.members = e;
	}
	map(t, n) {
		let r = this.members.map((e) => e.map(t, n, km));
		return e.from(r);
	}
	forChild(t, n) {
		if (n.isLeaf) return K.empty;
		let r = [];
		for (let i = 0; i < this.members.length; i++) {
			let a = this.members[i].forChild(t, n);
			a != Am && (a instanceof e ? r = r.concat(a.members) : r.push(a));
		}
		return e.from(r);
	}
	eq(t) {
		if (!(t instanceof e) || t.members.length != this.members.length) return !1;
		for (let e = 0; e < this.members.length; e++) if (!this.members[e].eq(t.members[e])) return !1;
		return !0;
	}
	locals(e) {
		let t, n = !0;
		for (let r = 0; r < this.members.length; r++) {
			let i = this.members[r].localsInner(e);
			if (i.length) {
				if (!t) t = i;
				else {
					n &&= (t = t.slice(), !1);
					for (let e = 0; e < i.length; e++) t.push(i[e]);
				}
			}
		}
		return t ? zm(n ? t : t.sort(Rm)) : Om;
	}
	static from(t) {
		switch (t.length) {
			case 0: return Am;
			case 1: return t[0];
			default: return new e(t.every((e) => e instanceof K) ? t : t.reduce((e, t) => e.concat(t instanceof K ? t : t.members), []));
		}
	}
	forEachSet(e) {
		for (let t = 0; t < this.members.length; t++) this.members[t].forEachSet(e);
	}
};
function Mm(e, t, n, r, i, a, o) {
	let s = e.slice();
	for (let e = 0, t = a; e < n.maps.length; e++) {
		let r = 0;
		n.maps[e].forEach((e, n, i, a) => {
			let o = a - i - (n - e);
			for (let i = 0; i < s.length; i += 3) {
				let a = s[i + 1];
				if (a < 0 || e > a + t - r) continue;
				let c = s[i] + t - r;
				n >= c ? s[i + 1] = e <= c ? -2 : -1 : e >= t && o && (s[i] += o, s[i + 1] += o);
			}
			r += o;
		}), t = n.maps[e].map(t, -1);
	}
	let c = !1;
	for (let t = 0; t < s.length; t += 3) if (s[t + 1] < 0) {
		if (s[t + 1] == -2) {
			c = !0, s[t + 1] = -1;
			continue;
		}
		let l = n.map(e[t] + a), u = l - i;
		if (u < 0 || u >= r.content.size) {
			c = !0;
			continue;
		}
		let d = n.map(e[t + 1] + a, -1) - i, { index: f, offset: p } = r.content.findIndex(u), m = r.maybeChild(f);
		if (m && p == u && p + m.nodeSize == d) {
			let r = s[t + 2].mapInner(n, m, l + 1, e[t] + a + 1, o);
			r == Am ? (s[t + 1] = -2, c = !0) : (s[t] = u, s[t + 1] = d, s[t + 2] = r);
		} else c = !0;
	}
	if (c) {
		let c = Lm(Pm(s, e, t, n, i, a, o), r, 0, o);
		t = c.local;
		for (let e = 0; e < s.length; e += 3) s[e + 1] < 0 && (s.splice(e, 3), e -= 3);
		for (let e = 0, t = 0; e < c.children.length; e += 3) {
			let n = c.children[e];
			for (; t < s.length && s[t] < n;) t += 3;
			s.splice(t, 0, c.children[e], c.children[e + 1], c.children[e + 2]);
		}
	}
	return new K(t.sort(Rm), s);
}
function Nm(e, t) {
	if (!t || !e.length) return e;
	let n = [];
	for (let r = 0; r < e.length; r++) {
		let i = e[r];
		n.push(new Dm(i.from + t, i.to + t, i.type));
	}
	return n;
}
function Pm(e, t, n, r, i, a, o) {
	function s(e, t) {
		for (let a = 0; a < e.local.length; a++) {
			let s = e.local[a].map(r, i, t);
			s ? n.push(s) : o.onRemove && o.onRemove(e.local[a].spec);
		}
		for (let n = 0; n < e.children.length; n += 3) s(e.children[n + 2], e.children[n] + t + 1);
	}
	for (let n = 0; n < e.length; n += 3) e[n + 1] == -1 && s(e[n + 2], t[n] + a + 1);
	return n;
}
function Fm(e, t, n) {
	if (t.isLeaf) return null;
	let r = n + t.nodeSize, i = null;
	for (let t = 0, a; t < e.length; t++) (a = e[t]) && a.from > n && a.to < r && ((i ||= []).push(a), e[t] = null);
	return i;
}
function Im(e) {
	let t = [];
	for (let n = 0; n < e.length; n++) e[n] != null && t.push(e[n]);
	return t;
}
function Lm(e, t, n, r) {
	let i = [], a = !1;
	t.forEach((t, o) => {
		let s = Fm(e, t, o + n);
		if (s) {
			a = !0;
			let e = Lm(s, t, n + o + 1, r);
			e != Am && i.push(o, o + t.nodeSize, e);
		}
	});
	let o = Nm(a ? Im(e) : e, -n).sort(Rm);
	for (let e = 0; e < o.length; e++) o[e].type.valid(t, o[e]) || (r.onRemove && r.onRemove(o[e].spec), o.splice(e--, 1));
	return o.length || i.length ? new K(o, i) : Am;
}
function Rm(e, t) {
	return e.from - t.from || e.to - t.to;
}
function zm(e) {
	let t = e;
	for (let n = 0; n < t.length - 1; n++) {
		let r = t[n];
		if (r.from != r.to) for (let i = n + 1; i < t.length; i++) {
			let a = t[i];
			if (a.from == r.from) a.to != r.to && (t == e && (t = e.slice()), t[i] = a.copy(a.from, r.to), Bm(t, i + 1, a.copy(r.to, a.to)));
			else {
				a.from < r.to && (t == e && (t = e.slice()), t[n] = r.copy(r.from, a.from), Bm(t, i, r.copy(a.from, r.to)));
				break;
			}
		}
	}
	return t;
}
function Bm(e, t, n) {
	for (; t < e.length && Rm(n, e[t]) > 0;) t++;
	e.splice(t, 0, n);
}
function Vm(e) {
	let t = [];
	return e.someProp("decorations", (n) => {
		let r = n(e.state);
		r && r != Am && t.push(r);
	}), e.cursorWrapper && t.push(K.create(e.state.doc, [e.cursorWrapper.deco])), jm.from(t);
}
var Hm = {
	childList: !0,
	characterData: !0,
	characterDataOldValue: !0,
	attributes: !0,
	attributeOldValue: !0,
	subtree: !0
}, Um = yd && bd <= 11, Wm = class {
	constructor() {
		this.anchorNode = null, this.anchorOffset = 0, this.focusNode = null, this.focusOffset = 0;
	}
	set(e) {
		this.anchorNode = e.anchorNode, this.anchorOffset = e.anchorOffset, this.focusNode = e.focusNode, this.focusOffset = e.focusOffset;
	}
	clear() {
		this.anchorNode = this.focusNode = null;
	}
	eq(e) {
		return e.anchorNode == this.anchorNode && e.anchorOffset == this.anchorOffset && e.focusNode == this.focusNode && e.focusOffset == this.focusOffset;
	}
}, Gm = class {
	constructor(e, t) {
		this.view = e, this.handleDOMChange = t, this.queue = [], this.flushingSoon = -1, this.observer = null, this.currentSelection = new Wm(), this.onCharData = null, this.suppressingSelectionUpdates = !1, this.lastChangedTextNode = null, this.observer = window.MutationObserver && new window.MutationObserver((t) => {
			for (let e = 0; e < t.length; e++) this.queue.push(t[e]);
			yd && bd <= 11 && t.some((e) => e.type == "childList" && e.removedNodes.length || e.type == "characterData" && e.oldValue.length > e.target.nodeValue.length) ? this.flushSoon() : G && e.composing && t.some((e) => e.type == "childList" && e.target.nodeName == "TR") ? (e.input.badSafariComposition = !0, this.flushSoon()) : this.flush();
		}), Um && (this.onCharData = (e) => {
			this.queue.push({
				target: e.target,
				type: "characterData",
				oldValue: e.prevValue
			}), this.flushSoon();
		}), this.onSelectionChange = this.onSelectionChange.bind(this);
	}
	flushSoon() {
		this.flushingSoon < 0 && (this.flushingSoon = window.setTimeout(() => {
			this.flushingSoon = -1, this.flush();
		}, 20));
	}
	forceFlush() {
		this.flushingSoon > -1 && (window.clearTimeout(this.flushingSoon), this.flushingSoon = -1, this.flush());
	}
	start() {
		this.observer && (this.observer.takeRecords(), this.observer.observe(this.view.dom, Hm)), this.onCharData && this.view.dom.addEventListener("DOMCharacterDataModified", this.onCharData), this.connectSelection();
	}
	stop() {
		if (this.observer) {
			let e = this.observer.takeRecords();
			if (e.length) {
				for (let t = 0; t < e.length; t++) this.queue.push(e[t]);
				window.setTimeout(() => this.flush(), 20);
			}
			this.observer.disconnect();
		}
		this.onCharData && this.view.dom.removeEventListener("DOMCharacterDataModified", this.onCharData), this.disconnectSelection();
	}
	connectSelection() {
		this.view.dom.ownerDocument.addEventListener("selectionchange", this.onSelectionChange);
	}
	disconnectSelection() {
		this.view.dom.ownerDocument.removeEventListener("selectionchange", this.onSelectionChange);
	}
	suppressSelectionUpdates() {
		this.suppressingSelectionUpdates = !0, setTimeout(() => this.suppressingSelectionUpdates = !1, 50);
	}
	onSelectionChange() {
		if (Xf(this.view)) {
			if (this.suppressingSelectionUpdates) return Bf(this.view);
			if (yd && bd <= 11 && !this.view.state.selection.empty) {
				let e = this.view.domSelectionRange();
				if (e.focusNode && td(e.focusNode, e.focusOffset, e.anchorNode, e.anchorOffset)) return this.flushSoon();
			}
			this.flush();
		}
	}
	setCurSelection() {
		this.currentSelection.set(this.view.domSelectionRange());
	}
	ignoreSelectionChange(e) {
		if (!e.focusNode) return !0;
		let t = /* @__PURE__ */ new Set(), n;
		for (let n = e.focusNode; n; n = Zu(n)) t.add(n);
		for (let r = e.anchorNode; r; r = Zu(r)) if (t.has(r)) {
			n = r;
			break;
		}
		let r = n && this.view.docView.nearestDesc(n);
		if (r && r.ignoreMutation({
			type: "selection",
			target: U(n) ? n.parentNode : n
		})) return this.setCurSelection(), !0;
	}
	pendingRecords() {
		if (this.observer) for (let e of this.observer.takeRecords()) this.queue.push(e);
		return this.queue;
	}
	flush() {
		let { view: e } = this;
		if (!e.docView || this.flushingSoon > -1) return;
		let t = this.pendingRecords();
		t.length && (this.queue = []);
		let n = e.domSelectionRange(), r = !this.suppressingSelectionUpdates && !this.currentSelection.eq(n) && Xf(e) && !this.ignoreSelectionChange(n), i = -1, a = -1, o = !1, s = [];
		if (e.editable) for (let e = 0; e < t.length; e++) {
			let n = this.registerMutation(t[e], s);
			n && (i = i < 0 ? n.from : Math.min(n.from, i), a = a < 0 ? n.to : Math.max(n.to, a), n.typeOver && (o = !0));
		}
		if (s.some((e) => e.nodeName == "BR") && (e.input.lastKeyCode == 8 || e.input.lastKeyCode == 46 || W && (e.composing || e.input.compositionEndedAt > Date.now() - 50) && t.some((e) => e.type == "childList" && e.removedNodes.length))) {
			for (let e of s) if (e.nodeName == "BR" && e.parentNode) {
				let t = e.nextSibling;
				for (; t && H(t);) {
					if (t.contentEditable == "false") {
						e.parentNode.removeChild(e);
						break;
					}
					t = t.firstChild;
				}
			}
		} else if (xd && s.length) {
			let t = s.filter((e) => e.nodeName == "BR");
			if (t.length == 2) {
				let [e, n] = t;
				e.parentNode && e.parentNode.parentNode == n.parentNode ? n.remove() : e.remove();
			} else {
				let { focusNode: n } = this.currentSelection;
				for (let r of t) {
					let t = r.parentNode;
					t && t.nodeName == "LI" && (!n || Zm(e, n) != t) && r.remove();
				}
			}
		}
		let c = null;
		i < 0 && r && e.input.lastFocus > Date.now() - 200 && Math.max(e.input.lastTouch, e.input.lastClick.time) < Date.now() - 300 && ld(n) && (c = Rf(e)) && c.eq(L.near(e.state.doc.resolve(0), 1)) ? (e.input.lastFocus = 0, Bf(e), this.currentSelection.set(n), e.scrollToSelection()) : (i > -1 || r) && (i > -1 && (e.docView.markDirty(i, a), Jm(e)), e.input.badSafariComposition && (e.input.badSafariComposition = !1, Qm(e, s)), this.handleDOMChange(i, a, o, s), e.docView && e.docView.dirty ? e.updateState(e.state) : this.currentSelection.eq(n) || Bf(e), this.currentSelection.set(n));
	}
	registerMutation(e, t) {
		if (t.indexOf(e.target) > -1) return null;
		let n = this.view.docView.nearestDesc(e.target);
		if (e.type == "attributes" && (n == this.view.docView || e.attributeName == "contenteditable" || e.attributeName == "style" && !e.oldValue && !e.target.getAttribute("style")) || !n || n.ignoreMutation(e)) return null;
		if (e.type == "childList") {
			for (let n = 0; n < e.addedNodes.length; n++) {
				let r = e.addedNodes[n];
				t.push(r), U(r) && (this.lastChangedTextNode = r);
			}
			if (n.contentDOM && n.contentDOM != n.dom && !n.contentDOM.contains(e.target)) return {
				from: n.posBefore,
				to: n.posAfter
			};
			let r = e.previousSibling, i = e.nextSibling;
			if (yd && bd <= 11 && e.addedNodes.length) for (let t = 0; t < e.addedNodes.length; t++) {
				let { previousSibling: n, nextSibling: a } = e.addedNodes[t];
				(!n || Array.prototype.indexOf.call(e.addedNodes, n) < 0) && (r = n), (!a || Array.prototype.indexOf.call(e.addedNodes, a) < 0) && (i = a);
			}
			let a = r && r.parentNode == e.target ? B(r) + 1 : 0, o = n.localPosFromDOM(e.target, a, -1), s = i && i.parentNode == e.target ? B(i) : e.target.childNodes.length;
			return {
				from: o,
				to: n.localPosFromDOM(e.target, s, 1)
			};
		}
		return e.type == "attributes" ? {
			from: n.posAtStart - n.border,
			to: n.posAtEnd + n.border
		} : (this.lastChangedTextNode = e.target, {
			from: n.posAtStart,
			to: n.posAtEnd,
			typeOver: e.target.nodeValue == e.oldValue
		});
	}
}, Km = /* @__PURE__ */ new WeakMap(), qm = !1;
function Jm(e) {
	if (!Km.has(e) && (Km.set(e, null), [
		"normal",
		"nowrap",
		"pre-line"
	].indexOf(getComputedStyle(e.dom).whiteSpace) !== -1)) {
		if (e.requiresGeckoHackNode = xd, qm) return;
		console.warn("ProseMirror expects the CSS white-space property to be set, preferably to 'pre-wrap'. It is recommended to load style/prosemirror.css from the prosemirror-view package."), qm = !0;
	}
}
function Ym(e, t) {
	let n = t.startContainer, r = t.startOffset, i = t.endContainer, a = t.endOffset, o = e.domAtPos(e.state.selection.anchor);
	return td(o.node, o.offset, i, a) && ([n, r, i, a] = [
		i,
		a,
		n,
		r
	]), {
		anchorNode: n,
		anchorOffset: r,
		focusNode: i,
		focusOffset: a
	};
}
function Xm(e, t) {
	if (t.getComposedRanges) {
		let n = t.getComposedRanges(e.root)[0];
		if (n) return Ym(e, n);
	}
	let n;
	function r(e) {
		e.preventDefault(), e.stopImmediatePropagation(), n = e.getTargetRanges()[0];
	}
	return e.dom.addEventListener("beforeinput", r, !0), document.execCommand("indent"), e.dom.removeEventListener("beforeinput", r, !0), n ? Ym(e, n) : null;
}
function Zm(e, t) {
	for (let n = t.parentNode; n && n != e.dom; n = n.parentNode) {
		let t = e.docView.nearestDesc(n, !0);
		if (t && t.node.isBlock) return n;
	}
	return null;
}
function Qm(e, t) {
	let { focusNode: n, focusOffset: r } = e.domSelectionRange();
	for (let i of t) if (i.parentNode?.nodeName == "TR") {
		let t = i.nextSibling;
		for (; t && t.nodeName != "TD" && t.nodeName != "TH";) t = t.nextSibling;
		if (t) {
			let a = t;
			for (;;) {
				let e = a.firstChild;
				if (!e || !H(e) || e.contentEditable == "false" || /^(BR|IMG)$/.test(e.nodeName)) break;
				a = e;
			}
			a.insertBefore(i, a.firstChild), n == i && e.domSelection().collapse(i, r);
		} else i.parentNode.removeChild(i);
	}
}
function $m(e, t, n, r) {
	let { node: i, fromOffset: a, toOffset: o, from: s, to: c } = e.docView.parseRange(t, n), l = e.domSelectionRange(), u, d = l.anchorNode;
	if (d && id(e.dom, d) && (u = [{
		node: d,
		offset: l.anchorOffset
	}], ld(l) || u.push({
		node: l.focusNode,
		offset: l.focusOffset
	})), W && e.input.lastKeyCode === 8) for (let e = o; e > a; e--) {
		let t = i.childNodes[e - 1], n = t.pmViewDesc;
		if (t.nodeName == "BR" && !n) {
			o = e;
			break;
		}
		if (!n || n.size) break;
	}
	let f = e.state.doc, p = e.someProp("domParser") || gs.fromSchema(e.state.schema), m = f.resolve(s), h = null, g = p.parse(i, {
		topNode: m.parent,
		topMatch: m.parent.contentMatchAt(m.index()),
		topOpen: !0,
		from: a,
		to: o,
		preserveWhitespace: m.parent.type.whitespace != "pre" || "full",
		findPositions: u,
		ruleFromNode: eh(r),
		context: m
	});
	if (u && u[0].pos != null) {
		let e = u[0].pos, t = u[1] && u[1].pos;
		t ??= e, h = {
			anchor: e + s,
			head: t + s
		};
	}
	return {
		doc: g,
		sel: h,
		from: s,
		to: c
	};
}
var eh = (e) => (t) => {
	let n = t.pmViewDesc;
	if (n) return n.parseRule(e);
	if (t.nodeName == "BR" && t.parentNode) {
		if (G && /^(ul|ol)$/i.test(t.parentNode.nodeName)) {
			let e = document.createElement("div");
			return e.appendChild(document.createElement("li")), { skip: e };
		}
		if (t.parentNode.lastChild == t || G && /^(tr|table)$/i.test(t.parentNode.nodeName)) return { ignore: !0 };
	} else if (t.nodeName == "IMG" && t.getAttribute("mark-placeholder")) return { ignore: !0 };
	return null;
}, th = /^(a|abbr|acronym|b|bd[io]|big|br|button|cite|code|data(list)?|del|dfn|em|i|img|ins|kbd|label|map|mark|meter|output|q|ruby|s|samp|small|span|strong|su[bp]|time|u|tt|var)$/i;
function nh(e, t, n, r, i) {
	let a = e.input.compositionPendingChanges || (e.composing ? e.input.compositionID : 0);
	if (e.input.compositionPendingChanges = 0, t < 0) {
		let t = e.input.lastSelectionTime > Date.now() - 50 ? e.input.lastSelectionOrigin : null, n = Rf(e, t);
		if (n && !e.state.selection.eq(n)) {
			if (W && Dd && e.input.lastKeyCode === 13 && Date.now() - 100 < e.input.lastKeyCodeTime && e.someProp("handleKeyDown", (t) => t(e, ud(13, "Enter")))) return;
			let r = e.state.tr.setSelection(n);
			t == "pointer" ? r.setMeta("pointer", !0) : t == "key" && r.scrollIntoView(), a && r.setMeta("composition", a), e.dispatch(r);
		}
		return;
	}
	let o = e.state.doc.resolve(t), s = o.sharedDepth(n);
	t = o.before(s + 1), n = e.state.doc.resolve(n).after(s + 1);
	let c = e.state.selection, l = $m(e, t, n, i), u = e.state.doc, d = u.slice(l.from, l.to), f, p;
	e.input.lastKeyCode === 8 && Date.now() - 100 < e.input.lastKeyCodeTime ? (f = e.state.selection.to, p = "end") : (f = e.state.selection.from, p = "start"), e.input.lastKeyCode = null;
	let m = ch(d.content, l.doc.content, l.from, f, p);
	if (m && e.input.domChangeCount++, (wd && e.input.lastIOSEnter > Date.now() - 225 || Dd) && i.some((e) => H(e) && !th.test(e.nodeName)) && (!m || m.endA >= m.endB) && e.someProp("handleKeyDown", (t) => t(e, ud(13, "Enter")))) {
		e.input.lastIOSEnter = 0;
		return;
	}
	if (!m) {
		if (r && c instanceof R && !c.empty && c.$head.sameParent(c.$anchor) && !e.composing && !(l.sel && l.sel.anchor != l.sel.head)) m = {
			start: c.from,
			endA: c.to,
			endB: c.to
		};
		else {
			if (l.sel) {
				let t = rh(e, e.state.doc, l.sel);
				if (t && !t.eq(e.state.selection)) {
					let n = e.state.tr.setSelection(t);
					a && n.setMeta("composition", a), e.dispatch(n);
				}
			}
			return;
		}
	}
	e.state.selection.from < e.state.selection.to && m.start == m.endB && e.state.selection instanceof R && (m.start > e.state.selection.from && m.start <= e.state.selection.from + 2 && e.state.selection.from >= l.from ? m.start = e.state.selection.from : m.endA < e.state.selection.to && m.endA >= e.state.selection.to - 2 && e.state.selection.to <= l.to && (m.endB += e.state.selection.to - m.endA, m.endA = e.state.selection.to)), yd && bd <= 11 && m.endB == m.start + 1 && m.endA == m.start && m.start > l.from && l.doc.textBetween(m.start - l.from - 1, m.start - l.from + 1) == " \xA0" && (m.start--, m.endA--, m.endB--);
	let h = l.doc.resolveNoCache(m.start - l.from), g = l.doc.resolveNoCache(m.endB - l.from), _ = u.resolve(m.start), v = h.sameParent(g) && h.parent.inlineContent && _.end() >= m.endA;
	if ((wd && e.input.lastIOSEnter > Date.now() - 225 && (!v || i.some((e) => e.nodeName == "DIV" || e.nodeName == "P")) || !v && oh(l.doc, h.pos, g.pos)) && e.someProp("handleKeyDown", (t) => t(e, ud(13, "Enter")))) {
		e.input.lastIOSEnter = 0;
		return;
	}
	if (e.state.selection.anchor > m.start && ah(u, m.start, m.endA, h, g) && e.someProp("handleKeyDown", (t) => t(e, ud(8, "Backspace")))) {
		Dd && W && e.domObserver.suppressSelectionUpdates();
		return;
	}
	W && m.endB == m.start && (e.input.lastChromeDelete = Date.now()), Dd && !v && h.start() != g.start() && g.parentOffset == 0 && h.depth == g.depth && l.sel && l.sel.anchor == l.sel.head && l.sel.head == m.endA && (m.endB -= 2, g = l.doc.resolveNoCache(m.endB - l.from), setTimeout(() => {
		e.someProp("handleKeyDown", function(t) {
			return t(e, ud(13, "Enter"));
		});
	}, 20));
	let y = m.start, b = m.endA, x = (t) => {
		let n = t || e.state.tr.replace(y, b, l.doc.slice(m.start - l.from, m.endB - l.from));
		if (l.sel) {
			let t = rh(e, n.doc, l.sel);
			t && !(W && e.composing && t.empty && (m.start != m.endB || e.input.lastChromeDelete < Date.now() - 100) && (t.head == y || t.head == n.mapping.map(b) - 1) || yd && t.empty && t.head == y) && n.setSelection(t);
		}
		return a && n.setMeta("composition", a), n.scrollIntoView();
	}, ee;
	if (v) {
		if (h.pos == g.pos) {
			yd && bd <= 11 && h.parentOffset == 0 && (e.domObserver.suppressSelectionUpdates(), setTimeout(() => Bf(e), 20));
			let t = x(e.state.tr.delete(y, b)), n = u.resolve(m.start).marksAcross(u.resolve(m.endA));
			n && t.ensureMarks(n), e.dispatch(t);
		} else if (m.endA == m.endB && (ee = ih(h.parent.content.cut(h.parentOffset, g.parentOffset), _.parent.content.cut(_.parentOffset, m.endA - _.start())))) {
			let t = x(e.state.tr);
			ee.type == "add" ? t.addMark(y, b, ee.mark) : t.removeMark(y, b, ee.mark), e.dispatch(t);
		} else if (h.parent.child(h.index()).isText && h.index() == g.index() - +!g.textOffset) {
			let t = h.parent.textBetween(h.parentOffset, g.parentOffset), n = () => x(e.state.tr.insertText(t, y, b));
			e.someProp("handleTextInput", (r) => r(e, y, b, t, n)) || e.dispatch(n());
		} else e.dispatch(x());
	} else e.dispatch(x());
}
function rh(e, t, n) {
	return Math.max(n.anchor, n.head) > t.content.size ? null : Yf(e, t.resolve(n.anchor), t.resolve(n.head));
}
function ih(e, t) {
	let n = e.firstChild.marks, r = t.firstChild.marks, i = n, a = r, o, s, c;
	for (let e = 0; e < r.length; e++) i = r[e].removeFromSet(i);
	for (let e = 0; e < n.length; e++) a = n[e].removeFromSet(a);
	if (i.length == 1 && a.length == 0) s = i[0], o = "add", c = (e) => e.mark(s.addToSet(e.marks));
	else if (i.length == 0 && a.length == 1) s = a[0], o = "remove", c = (e) => e.mark(s.removeFromSet(e.marks));
	else return null;
	let l = [];
	for (let e = 0; e < t.childCount; e++) l.push(c(t.child(e)));
	if (N.from(l).eq(e)) return {
		mark: s,
		type: o
	};
}
function ah(e, t, n, r, i) {
	if (n - t <= i.pos - r.pos || sh(r, !0, !1) < i.pos) return !1;
	let a = e.resolve(t);
	if (!r.parent.isTextblock) {
		let e = a.nodeAfter;
		return e != null && n == t + e.nodeSize;
	}
	if (a.parentOffset < a.parent.content.size || !a.parent.isTextblock) return !1;
	let o = e.resolve(sh(a, !0, !0));
	return !o.parent.isTextblock || o.pos > n || sh(o, !0, !1) < n ? !1 : r.parent.content.cut(r.parentOffset).eq(o.parent.content);
}
function oh(e, t, n) {
	let r = 0, i = 0;
	return e.nodesBetween(t, n, (e, a) => {
		e.isTextblock && a >= t && a < n ? r++ : (e.isText ? /\S/.test(e.text.slice(Math.max(0, t - a), Math.min(e.nodeSize, n - a))) : e.isLeaf) && i++;
	}), i == 0 && r == 1;
}
function sh(e, t, n) {
	let r = e.depth, i = t ? e.end() : e.pos;
	for (; r > 0 && (t || e.indexAfter(r) == e.node(r).childCount);) r--, i++, t = !1;
	if (n) {
		let t = e.node(r).maybeChild(e.indexAfter(r));
		for (; t && !t.isLeaf;) t = t.firstChild, i++;
	}
	return i;
}
function ch(e, t, n, r, i) {
	let a = e.findDiffStart(t, n), o = n + e.size, s = n + t.size;
	if (a == null) return null;
	let { a: c, b: l } = e.findDiffEnd(t, o, s);
	if (i == "end") {
		let e = Math.max(0, a - Math.min(c, l));
		r -= c + e - a;
	}
	if (c < a && o < s) {
		let e = r <= a && r >= c ? a - r : 0;
		a -= e, l = a + (l - c), c = a;
	} else if (l < a) {
		let e = r <= a && r >= l ? a - r : 0;
		a -= e, c = a + (c - l), l = a;
	}
	return {
		start: a,
		endA: c,
		endB: l
	};
}
var lh = class {
	constructor(e, t) {
		this._root = null, this.focused = !1, this.trackWrites = null, this.mounted = !1, this.markCursor = null, this.cursorWrapper = null, this.lastSelectedViewDesc = void 0, this.input = new Ip(), this.prevDirectPlugins = [], this.pluginViews = [], this.requiresGeckoHackNode = !1, this.dragging = null, this._props = t, this.state = t.state, this.directPlugins = t.plugins || [], this.directPlugins.forEach(gh), this.dispatch = this.dispatch.bind(this), this.dom = e && e.mount || document.createElement("div"), e && (e.appendChild ? e.appendChild(this.dom) : typeof e == "function" ? e(this.dom) : e.mount && (this.mounted = !0)), this.editable = fh(this), dh(this), this.nodeViews = mh(this), this.docView = vf(this.state.doc, uh(this), Vm(this), this.dom, this), this.domObserver = new Gm(this, (e, t, n, r) => nh(this, e, t, n, r)), this.domObserver.start(), Lp(this), this.updatePluginViews();
	}
	get composing() {
		return this.input.composing;
	}
	get props() {
		if (this._props.state != this.state) {
			let e = this._props;
			this._props = {};
			for (let t in e) this._props[t] = e[t];
			this._props.state = this.state;
		}
		return this._props;
	}
	update(e) {
		e.handleDOMEvents != this._props.handleDOMEvents && Bp(this);
		let t = this._props;
		this._props = e, e.plugins && (e.plugins.forEach(gh), this.directPlugins = e.plugins), this.updateStateInner(e.state, t);
	}
	setProps(e) {
		let t = {};
		for (let e in this._props) t[e] = this._props[e];
		t.state = this.state;
		for (let n in e) t[n] = e[n];
		this.update(t);
	}
	updateState(e) {
		this.updateStateInner(e, this._props);
	}
	updateStateInner(e, t) {
		let n = this.state, r = !1, i = !1;
		e.storedMarks && this.composing && (um(this), i = !0), this.state = e;
		let a = n.plugins != e.plugins || this._props.plugins != t.plugins;
		if (a || this._props.plugins != t.plugins || this._props.nodeViews != t.nodeViews) {
			let e = mh(this);
			hh(e, this.nodeViews) && (this.nodeViews = e, r = !0);
		}
		(a || t.handleDOMEvents != this._props.handleDOMEvents) && Bp(this), this.editable = fh(this), dh(this);
		let o = Vm(this), s = uh(this), c = n.plugins != e.plugins && !n.doc.eq(e.doc) ? "reset" : e.scrollToSelection > n.scrollToSelection ? "to selection" : "preserve", l = r || !this.docView.matchesNode(e.doc, s, o);
		(l || !e.selection.eq(n.selection)) && (i = !0);
		let u = c == "preserve" && i && this.dom.style.overflowAnchor == null && Pd(this);
		if (i) {
			this.domObserver.stop();
			let t = l && (yd || W) && !this.composing && !n.selection.empty && !e.selection.empty && ph(n.selection, e.selection);
			if (l) {
				let n = W ? this.trackWrites = this.domSelectionRange().focusNode : null;
				this.composing && (this.input.compositionNode = dm(this)), (r || !this.docView.update(e.doc, s, o, this)) && (this.docView.updateOuterDeco(s), this.docView.destroy(), this.docView = vf(e.doc, s, o, this.dom, this)), n && (!this.trackWrites || !this.dom.contains(this.trackWrites)) && (t = !0);
			}
			let i = this.input.mouseDown;
			t || !(i && this.domObserver.currentSelection.eq(this.domSelectionRange()) && Qf(this) && i.delaySelUpdate()) ? Bf(this, t) : (qf(this, e.selection), this.domObserver.setCurSelection()), this.domObserver.start();
		}
		this.updatePluginViews(n), this.dragging?.node && !n.doc.eq(e.doc) && this.updateDraggedNode(this.dragging, n), c == "reset" ? this.dom.scrollTop = 0 : c == "to selection" ? this.scrollToSelection() : u && Id(u);
	}
	scrollToSelection() {
		let e = this.domSelectionRange().focusNode;
		if (e && id(this.dom, e) && !this.someProp("handleScrollToSelection", (e) => e(this))) {
			if (this.state.selection instanceof z) {
				let t = this.docView.domAfterPos(this.state.selection.from);
				H(t) && Nd(this, t.getBoundingClientRect(), e);
			} else Nd(this, this.coordsAtPos(this.state.selection.head, 1), e);
		}
	}
	destroyPluginViews() {
		let e;
		for (; e = this.pluginViews.pop();) e.destroy && e.destroy();
	}
	updatePluginViews(e) {
		if (!e || e.plugins != this.state.plugins || this.directPlugins != this.prevDirectPlugins) {
			this.prevDirectPlugins = this.directPlugins, this.destroyPluginViews();
			for (let e = 0; e < this.directPlugins.length; e++) {
				let t = this.directPlugins[e];
				t.spec.view && this.pluginViews.push(t.spec.view(this));
			}
			for (let e = 0; e < this.state.plugins.length; e++) {
				let t = this.state.plugins[e];
				t.spec.view && this.pluginViews.push(t.spec.view(this));
			}
		} else for (let t = 0; t < this.pluginViews.length; t++) {
			let n = this.pluginViews[t];
			n.update && n.update(this, e);
		}
	}
	updateDraggedNode(e, t) {
		let n = e.node, r = -1;
		if (n.from < this.state.doc.content.size && this.state.doc.nodeAt(n.from) == n.node) r = n.from;
		else {
			let e = n.from + (this.state.doc.content.size - t.doc.content.size);
			(e > 0 && e < this.state.doc.content.size && this.state.doc.nodeAt(e)) == n.node && (r = e);
		}
		this.dragging = new ym(e.slice, e.move, r < 0 ? void 0 : z.create(this.state.doc, r));
	}
	someProp(e, t) {
		let n = this._props && this._props[e], r;
		if (n != null && (r = t ? t(n) : n)) return r;
		for (let n = 0; n < this.directPlugins.length; n++) {
			let i = this.directPlugins[n].props[e];
			if (i != null && (r = t ? t(i) : i)) return r;
		}
		let i = this.state.plugins;
		if (i) for (let n = 0; n < i.length; n++) {
			let a = i[n].props[e];
			if (a != null && (r = t ? t(a) : a)) return r;
		}
	}
	hasFocus() {
		if (yd) {
			let e = this.root.activeElement;
			if (e == this.dom) return !0;
			if (!e || !this.dom.contains(e)) return !1;
			for (; e && this.dom != e && this.dom.contains(e);) {
				if (e.contentEditable == "false") return !1;
				e = e.parentElement;
			}
			return !0;
		}
		return this.root.activeElement == this.dom;
	}
	focus() {
		this.domObserver.stop(), this.editable && zd(this.dom), Bf(this), this.domObserver.start();
	}
	get root() {
		let e = this._root;
		if (e == null) {
			for (let e = this.dom.parentNode; e; e = e.parentNode) if (e.nodeType == 9 || e.nodeType == 11 && e.host) return e.getSelection || (Object.getPrototypeOf(e).getSelection = () => e.ownerDocument.getSelection()), this._root = e;
		}
		return e || document;
	}
	updateRoot() {
		this._root = null;
	}
	posAtCoords(e) {
		return qd(this, e);
	}
	coordsAtPos(e, t = 1) {
		return Zd(this, e, t);
	}
	domAtPos(e, t = 0) {
		return this.docView.domFromPos(e, t);
	}
	nodeDOM(e) {
		let t = this.docView.descAt(e);
		return t ? t.nodeDOM : null;
	}
	posAtDOM(e, t, n = -1) {
		let r = this.docView.posFromDOM(e, t, n);
		if (r == null) throw RangeError("DOM position not inside the editor");
		return r;
	}
	endOfTextblock(e, t) {
		return cf(this, t || this.state, e);
	}
	pasteHTML(e, t) {
		return _m(this, "", e, !1, t || new ClipboardEvent("paste"));
	}
	pasteText(e, t) {
		return _m(this, e, null, !0, t || new ClipboardEvent("paste"));
	}
	serializeForClipboard(e) {
		return _p(this, e);
	}
	destroy() {
		this.docView && (zp(this), this.destroyPluginViews(), this.mounted ? (this.docView.update(this.state.doc, [], Vm(this), this), this.dom.textContent = "") : this.dom.parentNode && this.dom.parentNode.removeChild(this.dom), this.docView.destroy(), this.docView = null, ed());
	}
	get isDestroyed() {
		return this.docView == null;
	}
	dispatchEvent(e) {
		return Up(this, e);
	}
	domSelectionRange() {
		let e = this.dom.isConnected !== !1 && this.domSelection();
		return e ? G && this.root.nodeType === 11 && dd(this.dom.ownerDocument) == this.dom && Xm(this, e) || e : {
			focusNode: null,
			focusOffset: 0,
			anchorNode: null,
			anchorOffset: 0
		};
	}
	domSelection() {
		return this.root.getSelection();
	}
};
lh.prototype.dispatch = function(e) {
	let t = this._props.dispatchTransaction;
	t ? t.call(this, e) : this.updateState(this.state.apply(e));
};
function uh(e) {
	let t = Object.create(null);
	return t.class = "ProseMirror", t.contenteditable = String(e.editable), e.someProp("attributes", (n) => {
		if (typeof n == "function" && (n = n(e.state)), n) for (let e in n) e == "class" ? t.class += " " + n[e] : e == "style" ? t.style = (t.style ? t.style + ";" : "") + n[e] : !t[e] && e != "contenteditable" && e != "nodeName" && (t[e] = String(n[e]));
	}), t.translate ||= "no", [Dm.node(0, e.state.doc.content.size, t)];
}
function dh(e) {
	if (e.markCursor) {
		let t = document.createElement("img");
		t.className = "ProseMirror-separator", t.setAttribute("mark-placeholder", "true"), t.setAttribute("alt", ""), e.cursorWrapper = {
			dom: t,
			deco: Dm.widget(e.state.selection.from, t, {
				raw: !0,
				marks: e.markCursor
			})
		};
	} else e.cursorWrapper = null;
}
function fh(e) {
	return !e.someProp("editable", (t) => t(e.state) === !1);
}
function ph(e, t) {
	let n = Math.min(e.$anchor.sharedDepth(e.head), t.$anchor.sharedDepth(t.head));
	return e.$anchor.start(n) != t.$anchor.start(n);
}
function mh(e) {
	let t = Object.create(null);
	function n(e) {
		for (let n in e) Object.prototype.hasOwnProperty.call(t, n) || (t[n] = e[n]);
	}
	return e.someProp("nodeViews", n), e.someProp("markViews", n), t;
}
function hh(e, t) {
	let n = 0, r = 0;
	for (let r in e) {
		if (e[r] != t[r]) return !0;
		n++;
	}
	for (let e in t) r++;
	return n != r;
}
function gh(e) {
	if (e.spec.state || e.spec.filterTransaction || e.spec.appendTransaction) throw RangeError("Plugins passed directly to the view must not have a state component");
}
//#endregion
//#region node_modules/.pnpm/@milkdown+core@7.22.2/node_modules/@milkdown/core/lib/index.js
function _h(e, t) {
	return e.meta = {
		package: "@milkdown/core",
		group: "System",
		...t
	}, e;
}
var vh = {
	text: (e, t, n, r) => {
		let i = e.value;
		return /^[^*_\\]*\s+$/.test(i) ? i : n.safe(i, {
			...r,
			encode: []
		});
	},
	strong: (e, t, n, r) => {
		let i = e.marker || n.options.strong || "*", a = n.enter("strong"), o = n.createTracker(r), s = o.move(i + i);
		return s += o.move(n.containerPhrasing(e, {
			before: s,
			after: i,
			...o.current()
		})), s += o.move(i + i), a(), s;
	},
	emphasis: (e, t, n, r) => {
		let i = e.marker || n.options.emphasis || "*", a = n.enter("emphasis"), o = n.createTracker(r), s = o.move(i);
		return s += o.move(n.containerPhrasing(e, {
			before: s,
			after: i,
			...o.current()
		})), s += o.move(i), a(), s;
	}
}, yh = E({}, "editorView"), bh = E({}, "editorState"), xh = E([], "initTimer"), Sh = E({}, "editor"), Ch = E([], "inputRules"), wh = E([], "prosePlugins"), Th = E([], "remarkPlugins"), Eh = E([], "nodeView"), Dh = E([], "markView"), Oh = E(ro().use(Dr).use(Oa), "remark"), kh = E({
	handlers: vh,
	encode: []
}, "remarkStringifyOptions"), Ah = Te("ConfigReady");
function jh(e) {
	let t = (t) => (t.record(Ah), async () => (await e(t), t.done(Ah), () => {
		t.clearTimer(Ah);
	}));
	return _h(t, { displayName: "Config" }), t;
}
var Mh = Te("InitReady");
function Nh(e) {
	let t = (t) => (t.inject(Sh, e).inject(wh, []).inject(Th, []).inject(Ch, []).inject(Eh, []).inject(Dh, []).inject(kh, {
		handlers: vh,
		encode: []
	}).inject(Oh, ro().use(Dr).use(Oa)).inject(xh, [Ah]).record(Mh), async () => {
		await t.waitTimers(xh);
		let e = t.get(kh);
		return t.set(Oh, ro().use(Dr).use(Oa, e)), t.done(Mh), () => {
			t.remove(Sh).remove(wh).remove(Th).remove(Ch).remove(Eh).remove(Dh).remove(kh).remove(Oh).remove(xh).clearTimer(Mh);
		};
	});
	return _h(t, { displayName: "Init" }), t;
}
var Ph = Te("SchemaReady"), Fh = E([], "schemaTimer"), Ih = E({}, "schema"), Lh = E([], "nodes"), Rh = E([], "marks");
function zh(e) {
	return {
		...e,
		parseDOM: e.parseDOM?.map((t) => ({
			priority: e.priority,
			...t
		}))
	};
}
var Bh = (e) => (e.inject(Ih, {}).inject(Lh, []).inject(Rh, []).inject(Fh, [Mh]).record(Ph), async () => {
	await e.waitTimers(Fh);
	let t = e.get(Oh), n = e.get(Th).reduce((e, t) => e.use(t.plugin, t.options), t);
	e.set(Oh, n);
	let r = new fs({
		nodes: Object.fromEntries(e.get(Lh).map(([e, t]) => [e, zh(t)])),
		marks: Object.fromEntries(e.get(Rh).map(([e, t]) => [e, zh(t)]))
	});
	return e.set(Ih, r), e.done(Ph), () => {
		e.remove(Ih).remove(Lh).remove(Rh).remove(Fh).clearTimer(Ph);
	};
});
_h(Bh, { displayName: "Schema" });
var Vh = class {
	constructor() {
		this.#e = new _e(), this.#t = null, this.setCtx = (e) => {
			this.#t = e;
		}, this.chain = () => {
			if (this.#t == null) throw me();
			let e = this.#t, t = [], n = this.get.bind(this), r = {
				run: () => {
					let n = Yl(...t), r = e.get(yh);
					return n(r.state, r.dispatch, r);
				},
				inline: (e) => (t.push(e), r),
				pipe: i.bind(this)
			};
			function i(e, i) {
				let a = n(e);
				return t.push(a(i)), r;
			}
			return r;
		};
	}
	#e;
	#t;
	get ctx() {
		return this.#t;
	}
	create(e, t) {
		let n = e.create(this.#e.sliceMap);
		return n.set(t), n;
	}
	get(e) {
		return this.#e.get(e).get();
	}
	remove(e) {
		return this.#e.remove(e);
	}
	call(e, t) {
		if (this.#t == null) throw me();
		let n = this.get(e)(t), r = this.#t.get(yh);
		return n(r.state, r.dispatch, r);
	}
	inline(e) {
		if (this.#t == null) throw me();
		let t = this.#t.get(yh);
		return e(t.state, t.dispatch, t);
	}
};
function Hh(e = "cmdKey") {
	return E((() => () => !1), e);
}
var q = E(new Vh(), "commands"), Uh = E([Ph], "commandsTimer"), Wh = Te("CommandsReady"), Gh = (e) => {
	let t = new Vh();
	return t.setCtx(e), e.inject(q, t).inject(Uh, [Ph]).record(Wh), async () => (await e.waitTimers(Uh), e.done(Wh), () => {
		e.remove(q).remove(Uh).clearTimer(Wh);
	});
};
_h(Gh, { displayName: "Commands" });
function Kh(e) {
	return e.Backspace = Yl(ru, yl, Sl, Tl), e;
}
var qh = class {
	constructor() {
		this.#e = null, this.#t = [], this.setCtx = (e) => {
			this.#e = e;
		}, this.add = (e) => (this.#t.push(e), () => {
			this.#t = this.#t.filter((t) => t !== e);
		}), this.addObjectKeymap = (e) => {
			let t = [];
			return Object.entries(e).forEach(([e, n]) => {
				if (typeof n == "function") {
					let r = {
						key: e,
						onRun: () => n
					};
					this.#t.push(r), t.push(() => {
						this.#t = this.#t.filter((e) => e !== r);
					});
				} else this.#t.push(n), t.push(() => {
					this.#t = this.#t.filter((e) => e !== n);
				});
			}), () => {
				t.forEach((e) => e());
			};
		}, this.addBaseKeymap = () => {
			let e = Kh(eu);
			return this.addObjectKeymap(e);
		}, this.build = () => {
			let e = {};
			return this.#t.forEach((t) => {
				e[t.key] = [...e[t.key] || [], t];
			}), Object.fromEntries(Object.entries(e).map(([e, t]) => {
				let n = t.sort((e, t) => (t.priority ?? 50) - (e.priority ?? 50));
				return [e, (e, t, r) => {
					let i = this.#e;
					if (i == null) throw ce();
					return Yl(...n.map((e) => e.onRun(i)))(e, t, r);
				}];
			}));
		};
	}
	#e;
	#t;
	get ctx() {
		return this.#e;
	}
}, Jh = E(new qh(), "keymap"), Yh = E([Ph], "keymapTimer"), Xh = Te("KeymapReady"), Zh = (e) => {
	let t = new qh();
	return t.setCtx(e), e.inject(Jh, t).inject(Yh, [Ph]).record(Xh), async () => (await e.waitTimers(Yh), e.done(Xh), () => {
		e.remove(Jh).remove(Yh).clearTimer(Xh);
	});
}, Qh = Te("ParserReady"), $h = (() => {
	throw ce();
}), eg = E($h, "parser"), tg = E([], "parserTimer"), ng = (e) => (e.inject(eg, $h).inject(tg, [Ph]).record(Qh), async () => {
	await e.waitTimers(tg);
	let t = e.get(Oh), n = e.get(Ih);
	return e.set(eg, qu.create(n, t)), e.done(Qh), () => {
		e.remove(eg).remove(tg).clearTimer(Qh);
	};
});
_h(ng, { displayName: "Parser" });
var rg = Te("SerializerReady"), ig = E([], "serializerTimer"), ag = (() => {
	throw ce();
}), og = E(ag, "serializer"), sg = (e) => (e.inject(og, ag).inject(ig, [Ph]).record(rg), async () => {
	await e.waitTimers(ig);
	let t = e.get(Oh), n = e.get(Ih);
	return e.set(og, Xu.create(n, t)), e.done(rg), () => {
		e.remove(og).remove(ig).clearTimer(rg);
	};
});
_h(sg, { displayName: "Serializer" });
var cg = E("", "defaultValue"), lg = E((e) => e, "stateOptions"), ug = E([], "editorStateTimer"), dg = Te("EditorStateReady");
function fg(e, t, n) {
	if (typeof e == "string") return t(e);
	if (e.type === "html") return gs.fromSchema(n).parse(e.dom);
	if (e.type === "json") return Bo.fromJSON(n, e.value);
	throw ae(e);
}
var pg = new vl("MILKDOWN_STATE_TRACKER"), mg = (e) => (e.inject(cg, "").inject(bh, {}).inject(lg, (e) => e).inject(ug, [
	Qh,
	rg,
	Wh,
	Xh
]).record(dg), async () => {
	await e.waitTimers(ug);
	let t = e.get(Ih), n = e.get(eg), r = e.get(Ch), i = e.get(lg), a = e.get(wh), o = fg(e.get(cg), n, t), s = e.get(Jh), c = s.addBaseKeymap(), l = [
		...a,
		new hl({
			key: pg,
			state: {
				init: () => {},
				apply: (t, n, r, i) => {
					e.set(bh, i);
				}
			}
		}),
		vu({ rules: r }),
		Hu(s.build())
	];
	e.set(wh, l);
	let u = i({
		schema: t,
		doc: o,
		plugins: l
	}), d = pl.create(u);
	return e.set(bh, d), e.done(dg), () => {
		c(), e.remove(cg).remove(bh).remove(lg).remove(ug).clearTimer(dg);
	};
});
_h(mg, { displayName: "EditorState" });
var hg = E([], "pasteRule"), gg = E([Ph], "pasteRuleTimer"), _g = Te("PasteRuleReady"), vg = (e) => (e.inject(hg, []).inject(gg, [Ph]).record(_g), async () => (await e.waitTimers(gg), e.done(_g), () => {
	e.remove(hg).remove(gg).clearTimer(_g);
}));
_h(vg, { displayName: "PasteRule" });
var yg = Te("EditorViewReady"), bg = E([], "editorViewTimer"), xg = E({}, "editorViewOptions"), Sg = E(null, "root"), Cg = E(null, "rootDOM"), wg = E({}, "rootAttrs");
function Tg(e, t) {
	let n = document.createElement("div");
	n.className = "milkdown", e.appendChild(n), t.set(Cg, n);
	let r = t.get(wg);
	return Object.entries(r).forEach(([e, t]) => n.setAttribute(e, t)), n;
}
function Eg(e) {
	e.classList.add("editor"), e.setAttribute("role", "textbox");
}
var Dg = new vl("MILKDOWN_VIEW_CLEAR"), Og = (e) => (e.inject(Sg, document.body).inject(yh, {}).inject(xg, {}).inject(Cg, null).inject(wg, {}).inject(bg, [dg, _g]).record(yg), async () => {
	await e.wait(Mh);
	let t = e.get(Sg) || document.body, n = typeof t == "string" ? document.querySelector(t) : t;
	e.update(wh, (t) => [new hl({
		key: Dg,
		view: (t) => {
			let r = n ? Tg(n, e) : void 0;
			return (() => {
				if (r && n) {
					let e = t.dom;
					n.replaceChild(r, e), r.appendChild(e);
				}
			})(), { destroy: () => {
				r?.parentNode && r?.parentNode.replaceChild(t.dom, r), r?.remove();
			} };
		}
	}), ...t]), await e.waitTimers(bg);
	let r = e.get(bh), i = e.get(xg), a = new lh(n, {
		state: r,
		nodeViews: Object.fromEntries(e.get(Eh)),
		markViews: Object.fromEntries(e.get(Dh)),
		transformPasted: (t, n, r) => (e.get(hg).sort((e, t) => (t.priority ?? 50) - (e.priority ?? 50)).map((e) => e.run).forEach((e) => {
			t = e(t, n, r);
		}), t),
		...i
	});
	return Eg(a.dom), e.set(yh, a), e.done(yg), () => {
		a?.destroy(), e.remove(Sg).remove(yh).remove(xg).remove(Cg).remove(wg).remove(bg).clearTimer(yg);
	};
});
_h(Og, { displayName: "EditorView" });
var kg = class e {
	constructor() {
		this.#e = !1, this.#t = "Idle", this.#n = [], this.#r = () => void 0, this.#i = new _e(), this.#a = new Se(), this.#o = /* @__PURE__ */ new Map(), this.#s = /* @__PURE__ */ new Map(), this.#c = new xe(this.#i, this.#a), this.#l = () => {
			let e = jh(async (e) => {
				await Promise.all(this.#n.map((t) => Promise.resolve(t(e))));
			}), t = [
				Bh,
				ng,
				sg,
				Gh,
				Zh,
				vg,
				mg,
				Og,
				Nh(this),
				e
			];
			this.#u(t, this.#s);
		}, this.#u = (e, t) => {
			e.forEach((e) => {
				let n = this.#c.produce(this.#e ? e.meta : void 0), r = e(n);
				t.set(e, {
					ctx: n,
					handler: r,
					cleanup: void 0
				});
			});
		}, this.#d = (e, t = !1) => Promise.all([e].flat().map(async (e) => {
			let n = this.#o.get(e)?.cleanup;
			return t ? this.#o.delete(e) : this.#o.set(e, {
				ctx: void 0,
				handler: void 0,
				cleanup: void 0
			}), typeof n == "function" ? n() : n;
		})), this.#f = async () => {
			await Promise.all([...this.#s.entries()].map(async ([e, { cleanup: t }]) => typeof t == "function" ? t() : t)), this.#s.clear();
		}, this.#p = (e) => {
			this.#t = e, this.#r(e);
		}, this.#m = (e) => [...e.entries()].map(async ([t, n]) => {
			let { ctx: r, handler: i } = n;
			if (!i) return;
			let a = await i();
			e.set(t, {
				ctx: r,
				handler: i,
				cleanup: a
			});
		}), this.enableInspector = (e = !0) => (this.#e = e, this), this.onStatusChange = (e) => (this.#r = e, this), this.config = (e) => (this.#n.push(e), this), this.removeConfig = (e) => (this.#n = this.#n.filter((t) => t !== e), this), this.use = (e) => {
			let t = [e].flat();
			return t.flat().forEach((e) => {
				this.#o.set(e, {
					ctx: void 0,
					handler: void 0,
					cleanup: void 0
				});
			}), this.#t === "Created" && this.#u(t, this.#o), this;
		}, this.remove = async (e) => this.#t === "OnCreate" ? (console.warn("[Milkdown]: You are trying to remove plugins when the editor is creating, this is not recommended, please check your code."), new Promise((t) => {
			setTimeout(() => {
				t(this.remove(e));
			}, 50);
		})) : (await this.#d([e].flat(), !0), this), this.create = async () => (this.#t === "OnCreate" || (this.#t === "Created" && await this.destroy(), this.#p("OnCreate"), this.#l(), this.#u([...this.#o.keys()], this.#o), await Promise.all([this.#m(this.#s), this.#m(this.#o)].flat()), this.#p("Created")), this), this.destroy = async (e = !1) => this.#t === "Destroyed" || this.#t === "OnDestroy" ? this : this.#t === "OnCreate" ? new Promise((t) => {
			setTimeout(() => {
				t(this.destroy(e));
			}, 50);
		}) : (e && (this.#n = []), this.#p("OnDestroy"), await this.#d([...this.#o.keys()], e), await this.#f(), this.#p("Destroyed"), this), this.action = (e) => e(this.#c), this.inspect = () => this.#e ? [...this.#s.values(), ...this.#o.values()].map(({ ctx: e }) => e?.inspector?.read()).filter((e) => !!e) : (console.warn("[Milkdown]: You are trying to collect inspection when inspector is disabled, please enable inspector by `editor.enableInspector()` first."), []);
	}
	static make() {
		return new e();
	}
	#e;
	#t;
	#n;
	#r;
	#i;
	#a;
	#o;
	#s;
	#c;
	#l;
	#u;
	#d;
	#f;
	#p;
	#m;
	get ctx() {
		return this.#c;
	}
	get status() {
		return this.#t;
	}
}, Ag = (e) => crypto.getRandomValues(new Uint8Array(e)), jg = (e, t, n) => {
	let r = 256 - 256 % e.length;
	if (r === 256) {
		let r = e.length - 1;
		return (i = t) => {
			if (!i) return "";
			let a = "";
			for (;;) {
				let t = n(i), o = i |= 0;
				for (; o--;) if (a += e[t[o] & r], a.length >= i) return a;
			}
		};
	}
	return (i = t) => {
		if (!i) return "";
		let a = "", o = Math.ceil(409.6 * i / r);
		for (;;) {
			let t = n(o), s = o;
			for (; s--;) if (t[s] < r && (a += e[t[s] % e.length], a.length >= i)) return a;
		}
	};
};
((e, t = 21) => jg(e, t | 0, Ag))("abcedfghicklmn", 10);
function Mg(e, t, n) {
	let r = e.findIndex(([e]) => e === t);
	if (r === -1) return [...e, n];
	let i = e.slice();
	return i[r] = n, i;
}
function J(e, t) {
	let n = Hh(e), r = (i) => async () => {
		r.key = n, await i.wait(Wh);
		let a = t(i);
		return i.get(q).create(n, a), r.run = (t) => i.get(q).call(e, t), () => {
			i.get(q).remove(n);
		};
	};
	return r;
}
function Ng(e) {
	let t = (n) => async () => {
		await n.wait(Ph);
		let r = e(n);
		return n.update(Ch, (e) => [...e, r]), t.inputRule = r, () => {
			n.update(Ch, (e) => e.filter((e) => e !== r));
		};
	};
	return t;
}
function Pg(e) {
	let t = (n) => async () => {
		await n.wait(Ph);
		let r = e(n);
		return n.update(hg, (e) => [...e, r]), t.pasteRule = r, () => {
			n.update(hg, (e) => e.filter((e) => e !== r));
		};
	};
	return t;
}
function Fg(e, t) {
	let n = (r) => async () => {
		let i = t(r);
		return r.update(Rh, (t) => Mg(t, e, [e, i])), n.id = e, n.schema = i, () => {
			r.update(Rh, (t) => t.filter(([t]) => t !== e));
		};
	};
	return n.type = (t) => {
		let n = t.get(Ih).marks[e];
		if (!n) throw ge(e);
		return n;
	}, n;
}
function Ig(e, t) {
	let n = (r) => async () => {
		let i = t(r);
		return r.update(Lh, (t) => Mg(t, e, [e, i])), n.id = e, n.schema = i, () => {
			r.update(Lh, (t) => t.filter(([t]) => t !== e));
		};
	};
	return n.type = (t) => {
		let n = t.get(Ih).nodes[e];
		if (!n) throw he(e);
		return n;
	}, n;
}
function Lg(e) {
	let t, n = (n) => async () => (await n.wait(Ph), t = e(n), n.update(wh, (e) => [...e, t]), () => {
		n.update(wh, (e) => e.filter((e) => e !== t));
	});
	return n.plugin = () => t, n.key = () => t.spec.key, n;
}
function Rg(e) {
	let t = (n) => async () => {
		await n.wait(Xh);
		let r = n.get(Jh), i = e(n), a = r.addObjectKeymap(i);
		return t.keymap = i, () => {
			a();
		};
	};
	return t;
}
function zg(e, t) {
	let n = E(e, t), r = (e) => (e.inject(n), () => () => {
		e.remove(n);
	});
	return r.key = n, r;
}
function Y(e, t) {
	let n = zg(t, e), r = Ig(e, (e) => e.get(n.key)(e)), i = [n, r];
	return i.id = r.id, i.node = r, i.type = (e) => r.type(e), i.ctx = n, i.key = n.key, i.extendSchema = (n) => Y(e, n(t)), i;
}
function Bg(e, t) {
	let n = zg(t, e), r = Fg(e, (e) => e.get(n.key)(e)), i = [n, r];
	return i.id = r.id, i.mark = r, i.type = (e) => r.type(e), i.ctx = n, i.key = n.key, i.extendSchema = (n) => Bg(e, n(t)), i;
}
function Vg(e, t) {
	let n = zg(Object.fromEntries(Object.entries(t).map(([e, { shortcuts: t, priority: n }]) => [e, {
		shortcuts: t,
		priority: n
	}])), `${e}Keymap`), r = Rg((e) => {
		let r = e.get(n.key), i = Object.entries(t).flatMap(([e, { command: t }]) => {
			let n = r[e], i = [n.shortcuts].flat(), a = n.priority;
			return i.map((e) => [e, {
				key: e,
				onRun: t,
				priority: a
			}]);
		});
		return Object.fromEntries(i);
	}), i = [n, r];
	return i.ctx = n, i.shortcuts = r, i.key = n.key, i.keymap = r.keymap, i;
}
var Hg = (e, t = () => ({})) => zg(t, `${e}Attr`), Ug = (e, t = () => ({})) => zg(t, `${e}Attr`);
function Wg(e, t, n) {
	let r = zg(n ?? {}, e), i = (e) => async () => {
		await e.wait(Mh);
		let n = {
			plugin: t(e),
			options: e.get(r.key)
		};
		return e.update(Th, (e) => [...e, n]), () => {
			e.update(Th, (e) => e.filter((e) => e !== n));
		};
	}, a = [r, i];
	return a.id = e, a.plugin = i, a.options = r, a;
}
//#endregion
//#region node_modules/.pnpm/prosemirror-schema-list@1.5.1/node_modules/prosemirror-schema-list/dist/index.js
function Gg(e, t) {
	return function(n, r) {
		let { $from: i, $to: a, node: o } = n.selection;
		if (o && o.isBlock || i.depth < 2 || !i.sameParent(a)) return !1;
		let s = i.node(-1);
		if (s.type != e) return !1;
		if (i.parent.content.size == 0 && i.node(-1).childCount == i.indexAfter(-1)) {
			if (i.depth == 3 || i.node(-3).type != e || i.index(-2) != i.node(-2).childCount - 1) return !1;
			if (r) {
				let t = N.empty, a = i.index(-1) ? 1 : i.index(-2) ? 2 : 3;
				for (let e = i.depth - a; e >= i.depth - 3; e--) t = N.from(i.node(e).copy(t));
				let o = i.indexAfter(-1) < i.node(-2).childCount ? 1 : i.indexAfter(-2) < i.node(-3).childCount ? 2 : 3;
				t = t.append(N.from(e.createAndFill()));
				let s = i.before(i.depth - (a - 1)), c = n.tr.replace(s, i.after(-o), new F(t, 4 - a, 0)), l = -1;
				c.doc.nodesBetween(s, c.doc.content.size, (e, t) => {
					if (l > -1) return !1;
					e.isTextblock && e.content.size == 0 && (l = t + 1);
				}), l > -1 && c.setSelection(L.near(c.doc.resolve(l))), r(c.scrollIntoView());
			}
			return !0;
		}
		let c = a.pos == i.end() ? s.contentMatchAt(0).defaultType : null, l = n.tr.delete(i.pos, a.pos), u = c ? [t ? {
			type: e,
			attrs: t
		} : null, { type: c }] : void 0;
		return xc(l.doc, i.pos, 2, u) ? (r && r(l.split(i.pos, 2, u).scrollIntoView()), !0) : !1;
	};
}
function Kg(e) {
	return function(t, n) {
		let { $from: r, $to: i } = t.selection, a = r.blockRange(i, (t) => t.childCount > 0 && t.firstChild.type == e);
		return a ? n ? r.node(a.depth - 1).type == e ? qg(t, n, e, a) : Jg(t, n, a) : !0 : !1;
	};
}
function qg(e, t, n, r) {
	let i = e.tr, a = r.end, o = r.$to.end(r.depth);
	a < o && (i.step(new rc(a - 1, o, a, o, new F(N.from(n.create(null, r.parent.copy())), 1, 0), 1, !0)), r = new Ro(i.doc.resolve(r.$from.pos), i.doc.resolve(o), r.depth));
	let s = lc(r);
	if (s == null) return !1;
	i.lift(r, s);
	let c = i.doc.resolve(i.mapping.map(a, -1) - 1);
	return Cc(i.doc, c.pos) && c.nodeBefore.type == c.nodeAfter.type && i.join(c.pos), t(i.scrollIntoView()), !0;
}
function Jg(e, t, n) {
	let r = e.tr, i = n.parent;
	for (let e = n.end, t = n.endIndex - 1, a = n.startIndex; t > a; t--) e -= i.child(t).nodeSize, r.delete(e - 1, e + 1);
	let a = r.doc.resolve(n.start), o = a.nodeAfter;
	if (r.mapping.map(n.end) != n.start + a.nodeAfter.nodeSize) return !1;
	let s = n.startIndex == 0, c = n.endIndex == i.childCount, l = a.node(-1), u = a.index(-1);
	if (!l.canReplace(u + +!s, u + 1, o.content.append(c ? N.empty : N.from(i)))) return !1;
	let d = a.pos, f = d + o.nodeSize;
	return r.step(new rc(d - +!!s, f + +!!c, d + 1, f - 1, new F((s ? N.empty : N.from(i.copy(N.empty))).append(c ? N.empty : N.from(i.copy(N.empty))), +!s, +!c), +!s)), t(r.scrollIntoView()), !0;
}
function Yg(e) {
	return function(t, n) {
		let { $from: r, $to: i } = t.selection, a = r.blockRange(i, (t) => t.childCount > 0 && t.firstChild.type == e);
		if (!a) return !1;
		let o = a.startIndex;
		if (o == 0) return !1;
		let s = a.parent, c = s.child(o - 1);
		if (c.type != e) return !1;
		if (n) {
			let r = c.lastChild && c.lastChild.type == s.type, i = N.from(r ? e.create() : null), o = new F(N.from(e.create(null, N.from(s.type.create(null, i)))), r ? 3 : 1, 0), l = a.start, u = a.end;
			n(t.tr.step(new rc(l - (r ? 3 : 1), u, l, u, o, 1, !0)).scrollIntoView());
		}
		return !0;
	};
}
//#endregion
//#region node_modules/.pnpm/mdast-util-definitions@6.0.0/node_modules/mdast-util-definitions/lib/index.js
function Xg(e) {
	let t = /* @__PURE__ */ new Map();
	if (!e || !e.type) throw Error("mdast-util-definitions expected node");
	return ui(e, "definition", function(e) {
		let n = Zg(e.identifier);
		n && !t.get(n) && t.set(n, e);
	}), n;
	function n(e) {
		let n = Zg(e);
		return t.get(n);
	}
}
function Zg(e) {
	return String(e || "").toUpperCase();
}
//#endregion
//#region node_modules/.pnpm/remark-inline-links@7.0.0/node_modules/remark-inline-links/lib/index.js
function Qg() {
	return function(e) {
		let t = Xg(e);
		ui(e, function(e, n, r) {
			if (e.type === "definition" && r !== void 0 && typeof n == "number") return r.children.splice(n, 1), [si, n];
			if (e.type === "imageReference" || e.type === "linkReference") {
				let i = t(e.identifier);
				if (i && r && typeof n == "number") return r.children[n] = e.type === "imageReference" ? {
					type: "image",
					url: i.url,
					title: i.title,
					alt: e.alt
				} : {
					type: "link",
					url: i.url,
					title: i.title,
					children: e.children
				}, [si, n];
			}
		});
	};
}
//#endregion
//#region node_modules/.pnpm/@milkdown+preset-commonmark@7.22.2/node_modules/@milkdown/preset-commonmark/lib/index.js
function $g(e) {
	let t = { ...e };
	return t.position &&= {
		start: { ...t.position.start },
		end: { ...t.position.end }
	}, delete t.children, t;
}
var e_ = "<br />", t_ = /^<br\s*\/?\s*>$/i, n_ = (e) => typeof e == "string" && t_.test(e.trim());
function r_(e, t) {
	if (!(t.childCount >= 1 && t.lastChild?.type.name === "hardbreak")) {
		e.next(t.content);
		return;
	}
	let n = [];
	t.content.forEach((e, r, i) => {
		i !== t.childCount - 1 && n.push(e);
	}), e.next(N.fromArray(n));
}
function X(e, t) {
	return Object.assign(e, { meta: {
		package: "@milkdown/preset-commonmark",
		...t
	} }), e;
}
var i_ = Ug("emphasis");
X(i_, {
	displayName: "Attr<emphasis>",
	group: "Emphasis"
});
var a_ = Bg("emphasis", (e) => ({
	attrs: { marker: {
		default: e.get(kh).emphasis || "*",
		validate: "string"
	} },
	parseDOM: [
		{ tag: "i" },
		{ tag: "em" },
		{
			style: "font-style",
			getAttrs: (e) => e === "italic"
		}
	],
	toDOM: (t) => ["em", e.get(i_.key)(t)],
	parseMarkdown: {
		match: (e) => e.type === "emphasis",
		runner: (e, t, n) => {
			e.openMark(n, { marker: t.marker }), e.next(t.children), e.closeMark(n);
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "emphasis",
		runner: (e, t) => {
			e.withMark(t, "emphasis", void 0, { marker: t.attrs.marker });
		}
	}
}));
X(a_.mark, {
	displayName: "MarkSchema<emphasis>",
	group: "Emphasis"
}), X(a_.ctx, {
	displayName: "MarkSchemaCtx<emphasis>",
	group: "Emphasis"
});
var o_ = J("ToggleEmphasis", (e) => () => Jl(a_.type(e)));
X(o_, {
	displayName: "Command<toggleEmphasisCommand>",
	group: "Emphasis"
});
var s_ = Ng((e) => Su(/(?:^|[^*])\*([^*]+)\*$/, a_.type(e), {
	getAttr: () => ({ marker: "*" }),
	updateCaptured: ({ fullMatch: e, start: t }) => e.startsWith("*") ? {} : {
		fullMatch: e.slice(1),
		start: t + 1
	}
}));
X(s_, {
	displayName: "InputRule<emphasis>|Star",
	group: "Emphasis"
});
var c_ = Ng((e) => Su(/\b_(?![_\s])(.*?[^_\s])_$/, a_.type(e), {
	getAttr: () => ({ marker: "_" }),
	updateCaptured: ({ fullMatch: e, start: t }) => e.startsWith("_") ? {} : {
		fullMatch: e.slice(1),
		start: t + 1
	}
}));
X(c_, {
	displayName: "InputRule<emphasis>|Underscore",
	group: "Emphasis"
});
var l_ = Vg("emphasisKeymap", { ToggleEmphasis: {
	shortcuts: "Mod-i",
	command: (e) => {
		let t = e.get(q);
		return () => t.call(o_.key);
	}
} });
X(l_.ctx, {
	displayName: "KeymapCtx<emphasis>",
	group: "Emphasis"
}), X(l_.shortcuts, {
	displayName: "Keymap<emphasis>",
	group: "Emphasis"
});
var u_ = Ug("strong");
X(u_, {
	displayName: "Attr<strong>",
	group: "Strong"
});
var d_ = Bg("strong", (e) => ({
	attrs: { marker: {
		default: e.get(kh).strong || "*",
		validate: "string"
	} },
	parseDOM: [
		{
			tag: "b",
			getAttrs: (e) => e.style.fontWeight != "normal" && null
		},
		{ tag: "strong" },
		{
			style: "font-style",
			getAttrs: (e) => e === "bold"
		},
		{
			style: "font-weight=400",
			clearMark: (e) => e.type.name == "strong"
		},
		{
			style: "font-weight",
			getAttrs: (e) => /^(bold(er)?|[5-9]\d{2,})$/.test(e) && null
		}
	],
	toDOM: (t) => ["strong", e.get(u_.key)(t)],
	parseMarkdown: {
		match: (e) => e.type === "strong",
		runner: (e, t, n) => {
			e.openMark(n, { marker: t.marker }), e.next(t.children), e.closeMark(n);
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "strong",
		runner: (e, t) => {
			e.withMark(t, "strong", void 0, { marker: t.attrs.marker });
		}
	}
}));
X(d_.mark, {
	displayName: "MarkSchema<strong>",
	group: "Strong"
}), X(d_.ctx, {
	displayName: "MarkSchemaCtx<strong>",
	group: "Strong"
});
var f_ = J("ToggleStrong", (e) => () => Jl(d_.type(e)));
X(f_, {
	displayName: "Command<toggleStrongCommand>",
	group: "Strong"
});
var p_ = Ng((e) => Su(/(?<![\w:/])(?:\*\*|__)([^*_]+?)(?:\*\*|__)(?![\w/])$/, d_.type(e), { getAttr: (e) => ({ marker: e[0].startsWith("*") ? "*" : "_" }) }));
X(p_, {
	displayName: "InputRule<strong>",
	group: "Strong"
});
var m_ = Vg("strongKeymap", { ToggleBold: {
	shortcuts: ["Mod-b"],
	command: (e) => {
		let t = e.get(q);
		return () => t.call(f_.key);
	}
} });
X(m_.ctx, {
	displayName: "KeymapCtx<strong>",
	group: "Strong"
}), X(m_.shortcuts, {
	displayName: "Keymap<strong>",
	group: "Strong"
});
var h_ = Ug("inlineCode");
X(h_, {
	displayName: "Attr<inlineCode>",
	group: "InlineCode"
});
var g_ = Bg("inlineCode", (e) => ({
	priority: 100,
	code: !0,
	inclusive: !1,
	parseDOM: [{ tag: "code" }],
	toDOM: (t) => ["code", e.get(h_.key)(t)],
	parseMarkdown: {
		match: (e) => e.type === "inlineCode",
		runner: (e, t, n) => {
			e.openMark(n), e.addText(t.value), e.closeMark(n);
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "inlineCode",
		runner: (e, t, n) => (e.withMark(t, "inlineCode", n.text || ""), !0)
	}
}));
X(g_.mark, {
	displayName: "MarkSchema<inlineCode>",
	group: "InlineCode"
}), X(g_.ctx, {
	displayName: "MarkSchemaCtx<inlineCode>",
	group: "InlineCode"
});
var __ = J("ToggleInlineCode", (e) => () => (t, n) => {
	let { selection: r, tr: i } = t;
	if (r.empty) return !1;
	let { from: a, to: o } = r;
	return t.doc.rangeHasMark(a, o, g_.type(e)) ? (n?.(i.removeMark(a, o, g_.type(e))), !0) : (Object.keys(t.schema.marks).filter((e) => e !== g_.type.name).map((e) => t.schema.marks[e]).forEach((e) => {
		i.removeMark(a, o, e);
	}), n?.(i.addMark(a, o, g_.type(e).create())), !0);
});
X(__, {
	displayName: "Command<toggleInlineCodeCommand>",
	group: "InlineCode"
});
var v_ = Ng((e) => Su(/(?:`)([^`]+)(?:`)$/, g_.type(e)));
X(v_, {
	displayName: "InputRule<inlineCodeInputRule>",
	group: "InlineCode"
});
var y_ = Vg("inlineCodeKeymap", { ToggleInlineCode: {
	shortcuts: "Mod-e",
	command: (e) => {
		let t = e.get(q);
		return () => t.call(__.key);
	}
} });
X(y_.ctx, {
	displayName: "KeymapCtx<inlineCode>",
	group: "InlineCode"
}), X(y_.shortcuts, {
	displayName: "Keymap<inlineCode>",
	group: "InlineCode"
});
var b_ = /* @__PURE__ */ new Set([
	"http:",
	"https:",
	"mailto:",
	"tel:",
	"ftp:"
]);
function x_(e) {
	return e <= 32 || e >= 127 && e <= 160 || e >= 8203 && e <= 8205 || e === 8232 || e === 8233 || e === 65279;
}
function S_(e) {
	let t = "";
	for (let n of e) x_(n.codePointAt(0) ?? 0) || (t += n);
	return t;
}
function C_(e) {
	if (typeof e != "string") return "";
	let t = e.trim();
	if (!t) return "";
	let n = S_(t), r = /^([a-z][a-z0-9+.-]*):/i.exec(n)?.[1];
	return !r || b_.has(`${r.toLowerCase()}:`) ? t : "";
}
var w_ = Ug("link");
X(w_, {
	displayName: "Attr<link>",
	group: "Link"
});
var T_ = Bg("link", (e) => ({
	attrs: {
		href: { validate: "string" },
		title: {
			default: null,
			validate: "string|null"
		}
	},
	parseDOM: [{
		tag: "a[href]",
		getAttrs: (e) => {
			if (!(e instanceof HTMLElement)) throw pe(e);
			return {
				href: e.getAttribute("href"),
				title: e.getAttribute("title")
			};
		}
	}],
	toDOM: (t) => ["a", {
		...e.get(w_.key)(t),
		...t.attrs,
		href: C_(t.attrs.href)
	}],
	parseMarkdown: {
		match: (e) => e.type === "link",
		runner: (e, t, n) => {
			let r = t.url, i = t.title;
			e.openMark(n, {
				href: r,
				title: i
			}), e.next(t.children), e.closeMark(n);
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "link",
		runner: (e, t) => {
			e.withMark(t, "link", void 0, {
				title: t.attrs.title,
				url: t.attrs.href
			});
		}
	}
}));
X(T_.mark, {
	displayName: "MarkSchema<link>",
	group: "Link"
});
var E_ = J("ToggleLink", (e) => (t = {}) => Jl(T_.type(e), t));
X(E_, {
	displayName: "Command<toggleLinkCommand>",
	group: "Link"
});
var D_ = J("UpdateLink", (e) => (t = {}) => (n, r) => {
	if (!r) return !1;
	let i, a = -1, { selection: o } = n, { from: s, to: c } = o;
	if (n.doc.nodesBetween(s, s === c ? c + 1 : c, (t, n) => {
		if (T_.type(e).isInSet(t.marks)) return i = t, a = n, !1;
	}), !i) return !1;
	let l = i.marks.find(({ type: t }) => t === T_.type(e));
	if (!l) return !1;
	let u = a, d = a + i.nodeSize, { tr: f } = n, p = T_.type(e).create({
		...l.attrs,
		...t
	});
	return p ? (r(f.removeMark(u, d, l).addMark(u, d, p).setSelection(new R(f.selection.$anchor)).scrollIntoView()), !0) : !1;
});
X(D_, {
	displayName: "Command<updateLinkCommand>",
	group: "Link"
});
var O_ = Ig("doc", () => ({
	content: "block+",
	parseMarkdown: {
		match: ({ type: e }) => e === "root",
		runner: (e, t, n) => {
			e.injectRoot(t, n);
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "doc",
		runner: (e, t) => {
			e.openNode("root");
			let n = t.childCount > 0 ? t.child(t.childCount - 1) : null, r = n?.type.name === "paragraph" && n.content.size === 0 ? t.content.cut(0, t.content.size - n.nodeSize) : t.content;
			e.next(r);
		}
	}
}));
X(O_, {
	displayName: "NodeSchema<doc>",
	group: "Doc"
});
var k_ = Object.freeze([
	"root",
	"blockquote",
	"listItem",
	"footnoteDefinition"
]), A_ = /* @__PURE__ */ new Set([
	"paragraph",
	"heading",
	"tableCell",
	"link",
	"linkReference",
	"emphasis",
	"strong",
	"delete",
	"footnote"
]), j_ = zg(k_, "blockContainerTypes");
X(j_, {
	displayName: "Ctx<blockContainerTypes>",
	group: "Remark"
});
var M_ = /* @__PURE__ */ new Set();
function N_(e) {
	let t = e.isInjected(j_.key) ? e.get(j_.key) : k_, n = /* @__PURE__ */ new Set();
	for (let e of t) A_.has(e) ? M_.has(e) || (M_.add(e), console.warn(`[milkdown/preset-commonmark] "${e}" holds phrasing content and cannot be a block container; it is ignored in blockContainerTypes.`)) : n.add(e);
	return n;
}
var P_ = /* @__PURE__ */ new Set(["paragraph", "tableCell"]);
function F_(e, t) {
	return ci(e, (e) => e.type === "html" && n_(e.value), (e, n) => {
		let r = n[n.length - 1];
		if (r) {
			if (P_.has(r.type)) r.children.length === 1 && (r.children = []);
			else if (t.has(r.type)) {
				let t = e;
				return t.children = [], delete t.value, t.type = "paragraph", si;
			}
		}
	});
}
var I_ = Wg("remark-preserve-empty-line", (e) => () => (t) => F_(t, N_(e)));
X(I_.plugin, {
	displayName: "Remark<remarkPreserveEmptyLine>",
	group: "Remark"
}), X(I_.options, {
	displayName: "RemarkConfig<remarkPreserveEmptyLine>",
	group: "Remark"
});
var L_ = Hg("paragraph");
X(L_, {
	displayName: "Attr<paragraph>",
	group: "Paragraph"
});
var R_ = Y("paragraph", (e) => ({
	content: "inline*",
	group: "block",
	parseDOM: [{ tag: "p" }],
	toDOM: (t) => [
		"p",
		e.get(L_.key)(t),
		0
	],
	parseMarkdown: {
		match: (e) => e.type === "paragraph",
		runner: (e, t, n) => {
			e.openNode(n), t.children ? e.next(t.children) : e.addText(t.value || ""), e.closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "paragraph",
		runner: (t, n) => {
			t.openNode("paragraph"), (!n.content || n.content.size === 0) && z_(e) ? t.addNode("html", void 0, e_) : r_(t, n), t.closeNode();
		}
	}
}));
function z_(e) {
	return e.isInjected(I_.options.key);
}
X(R_.node, {
	displayName: "NodeSchema<paragraph>",
	group: "Paragraph"
}), X(R_.ctx, {
	displayName: "NodeSchemaCtx<paragraph>",
	group: "Paragraph"
});
var B_ = J("TurnIntoText", (e) => () => Gl(R_.type(e)));
X(B_, {
	displayName: "Command<turnIntoTextCommand>",
	group: "Paragraph"
});
var V_ = Vg("paragraphKeymap", { TurnIntoText: {
	shortcuts: "Mod-Alt-0",
	command: (e) => {
		let t = e.get(q);
		return () => t.call(B_.key);
	}
} });
X(V_.ctx, {
	displayName: "KeymapCtx<paragraph>",
	group: "Paragraph"
}), X(V_.shortcuts, {
	displayName: "Keymap<paragraph>",
	group: "Paragraph"
});
var H_ = [
	,
	,
	,
	,
	,
	,
].fill(0).map((e, t) => t + 1);
function U_(e) {
	return e.textContent.toLowerCase().trim().replace(/\s+/g, "-");
}
var W_ = zg(U_, "headingIdGenerator");
X(W_, {
	displayName: "Ctx<HeadingIdGenerator>",
	group: "Heading"
});
var G_ = Hg("heading");
X(G_, {
	displayName: "Attr<heading>",
	group: "Heading"
});
var K_ = Y("heading", (e) => {
	let t = e.get(W_.key);
	return {
		content: "inline*",
		group: "block",
		defining: !0,
		attrs: {
			id: {
				default: "",
				validate: "string"
			},
			level: {
				default: 1,
				validate: "number"
			}
		},
		parseDOM: H_.map((e) => ({
			tag: `h${e}`,
			getAttrs: (t) => {
				if (!(t instanceof HTMLElement)) throw pe(t);
				return {
					level: e,
					id: t.id
				};
			}
		})),
		toDOM: (n) => [
			`h${n.attrs.level}`,
			{
				...e.get(G_.key)(n),
				id: n.attrs.id || t(n)
			},
			0
		],
		parseMarkdown: {
			match: ({ type: e }) => e === "heading",
			runner: (e, t, n) => {
				let r = t.depth;
				e.openNode(n, { level: r }), e.next(t.children), e.closeNode();
			}
		},
		toMarkdown: {
			match: (e) => e.type.name === "heading",
			runner: (e, t) => {
				e.openNode("heading", void 0, { depth: t.attrs.level }), r_(e, t), e.closeNode();
			}
		}
	};
});
X(K_.node, {
	displayName: "NodeSchema<heading>",
	group: "Heading"
}), X(K_.ctx, {
	displayName: "NodeSchemaCtx<heading>",
	group: "Heading"
});
var q_ = Ng((e) => au(/^(?<hashes>#+)\s$/, K_.type(e), (t) => {
	let n = t.groups?.hashes?.length || 0, { $from: r } = e.get(yh).state.selection, i = r.node();
	if (i.type.name === "heading") {
		let e = Number(i.attrs.level) + Number(n);
		return e > 6 && (e = 6), { level: e };
	}
	return { level: n };
}));
X(q_, {
	displayName: "InputRule<wrapInHeadingInputRule>",
	group: "Heading"
});
var J_ = J("WrapInHeading", (e) => (t) => (t ??= 1, t < 1 ? Gl(R_.type(e)) : Gl(K_.type(e), { level: t })));
X(J_, {
	displayName: "Command<wrapInHeadingCommand>",
	group: "Heading"
});
var Y_ = J("DowngradeHeading", (e) => () => (t, n, r) => {
	let { $from: i } = t.selection, a = i.node();
	if (a.type !== K_.type(e) || !t.selection.empty || i.parentOffset !== 0) return !1;
	let o = a.attrs.level - 1;
	return o ? (n?.(t.tr.setNodeMarkup(t.selection.$from.before(), void 0, {
		...a.attrs,
		level: o
	})), !0) : Gl(R_.type(e))(t, n, r);
});
X(Y_, {
	displayName: "Command<downgradeHeadingCommand>",
	group: "Heading"
});
var X_ = Vg("headingKeymap", {
	TurnIntoH1: {
		shortcuts: "Mod-Alt-1",
		command: (e) => {
			let t = e.get(q);
			return () => t.call(J_.key, 1);
		}
	},
	TurnIntoH2: {
		shortcuts: "Mod-Alt-2",
		command: (e) => {
			let t = e.get(q);
			return () => t.call(J_.key, 2);
		}
	},
	TurnIntoH3: {
		shortcuts: "Mod-Alt-3",
		command: (e) => {
			let t = e.get(q);
			return () => t.call(J_.key, 3);
		}
	},
	TurnIntoH4: {
		shortcuts: "Mod-Alt-4",
		command: (e) => {
			let t = e.get(q);
			return () => t.call(J_.key, 4);
		}
	},
	TurnIntoH5: {
		shortcuts: "Mod-Alt-5",
		command: (e) => {
			let t = e.get(q);
			return () => t.call(J_.key, 5);
		}
	},
	TurnIntoH6: {
		shortcuts: "Mod-Alt-6",
		command: (e) => {
			let t = e.get(q);
			return () => t.call(J_.key, 6);
		}
	},
	DowngradeHeading: {
		shortcuts: ["Delete", "Backspace"],
		command: (e) => {
			let t = e.get(q);
			return () => t.call(Y_.key);
		}
	}
});
X(X_.ctx, {
	displayName: "KeymapCtx<heading>",
	group: "Heading"
}), X(X_.shortcuts, {
	displayName: "Keymap<heading>",
	group: "Heading"
});
var Z_ = Hg("blockquote");
X(Z_, {
	displayName: "Attr<blockquote>",
	group: "Blockquote"
});
var Q_ = Y("blockquote", (e) => ({
	content: "block+",
	group: "block",
	defining: !0,
	parseDOM: [{ tag: "blockquote" }],
	toDOM: (t) => [
		"blockquote",
		e.get(Z_.key)(t),
		0
	],
	parseMarkdown: {
		match: ({ type: e }) => e === "blockquote",
		runner: (e, t, n) => {
			e.openNode(n).next(t.children).closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "blockquote",
		runner: (e, t) => {
			e.openNode("blockquote").next(t.content).closeNode();
		}
	}
}));
X(Q_.node, {
	displayName: "NodeSchema<blockquote>",
	group: "Blockquote"
}), X(Q_.ctx, {
	displayName: "NodeSchemaCtx<blockquote>",
	group: "Blockquote"
});
var $_ = Ng((e) => iu(/^\s*>\s$/, Q_.type(e)));
X($_, {
	displayName: "InputRule<wrapInBlockquoteInputRule>",
	group: "Blockquote"
});
var ev = J("WrapInBlockquote", (e) => () => Wl(Q_.type(e)));
X(ev, {
	displayName: "Command<wrapInBlockquoteCommand>",
	group: "Blockquote"
});
var tv = Vg("blockquoteKeymap", { WrapInBlockquote: {
	shortcuts: "Mod-Shift-b",
	command: (e) => {
		let t = e.get(q);
		return () => t.call(ev.key);
	}
} });
X(tv.ctx, {
	displayName: "KeymapCtx<blockquote>",
	group: "Blockquote"
}), X(tv.shortcuts, {
	displayName: "Keymap<blockquote>",
	group: "Blockquote"
});
var nv = Hg("codeBlock", () => ({
	pre: {},
	code: {}
}));
X(nv, {
	displayName: "Attr<codeBlock>",
	group: "CodeBlock"
});
var rv = Y("code_block", (e) => ({
	content: "text*",
	group: "block",
	marks: "",
	defining: !0,
	code: !0,
	attrs: { language: {
		default: "",
		validate: "string"
	} },
	parseDOM: [{
		tag: "pre",
		preserveWhitespace: "full",
		getAttrs: (e) => {
			if (!(e instanceof HTMLElement)) throw pe(e);
			return { language: e.dataset.language };
		}
	}],
	toDOM: (t) => {
		let n = e.get(nv.key)(t), r = t.attrs.language, i = r && r.length > 0 ? { "data-language": r } : void 0;
		return [
			"pre",
			{
				...n.pre,
				...i
			},
			[
				"code",
				n.code,
				0
			]
		];
	},
	parseMarkdown: {
		match: ({ type: e }) => e === "code",
		runner: (e, t, n) => {
			let r = t.lang ?? "", i = t.value;
			e.openNode(n, { language: r }), i && e.addText(i), e.closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "code_block",
		runner: (e, t) => {
			e.addNode("code", void 0, t.content.firstChild?.text || "", { lang: t.attrs.language });
		}
	}
}));
X(rv.node, {
	displayName: "NodeSchema<codeBlock>",
	group: "CodeBlock"
}), X(rv.ctx, {
	displayName: "NodeSchemaCtx<codeBlock>",
	group: "CodeBlock"
});
var iv = Ng((e) => au(/^```(?<language>[a-z]*)?[\s\n]$/, rv.type(e), (e) => ({ language: e.groups?.language ?? "" })));
X(iv, {
	displayName: "InputRule<createCodeBlockInputRule>",
	group: "CodeBlock"
});
var av = J("CreateCodeBlock", (e) => (t = "") => Gl(rv.type(e), { language: t }));
X(av, {
	displayName: "Command<createCodeBlockCommand>",
	group: "CodeBlock"
}), X(J("UpdateCodeBlockLanguage", () => ({ pos: e, language: t } = {
	pos: -1,
	language: ""
}) => (n, r) => e >= 0 && (r?.(n.tr.setNodeAttribute(e, "language", t)), !0)), {
	displayName: "Command<updateCodeBlockLanguageCommand>",
	group: "CodeBlock"
});
var ov = Vg("codeBlockKeymap", { CreateCodeBlock: {
	shortcuts: "Mod-Alt-c",
	command: (e) => {
		let t = e.get(q);
		return () => t.call(av.key);
	}
} });
X(ov.ctx, {
	displayName: "KeymapCtx<codeBlock>",
	group: "CodeBlock"
}), X(ov.shortcuts, {
	displayName: "Keymap<codeBlock>",
	group: "CodeBlock"
});
var sv = Hg("image");
X(sv, {
	displayName: "Attr<image>",
	group: "Image"
});
var cv = Y("image", (e) => ({
	inline: !0,
	group: "inline",
	selectable: !0,
	draggable: !0,
	marks: "",
	atom: !0,
	defining: !0,
	isolating: !0,
	attrs: {
		src: {
			default: "",
			validate: "string"
		},
		alt: {
			default: "",
			validate: "string"
		},
		title: {
			default: "",
			validate: "string"
		}
	},
	parseDOM: [{
		tag: "img[src]",
		getAttrs: (e) => {
			if (!(e instanceof HTMLElement)) throw pe(e);
			return {
				src: e.getAttribute("src") || "",
				alt: e.getAttribute("alt") || "",
				title: e.getAttribute("title") || e.getAttribute("alt") || ""
			};
		}
	}],
	toDOM: (t) => ["img", {
		...e.get(sv.key)(t),
		...t.attrs
	}],
	parseMarkdown: {
		match: ({ type: e }) => e === "image",
		runner: (e, t, n) => {
			let r = t.url, i = t.alt, a = t.title;
			e.addNode(n, {
				src: r,
				alt: i,
				title: a
			});
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "image",
		runner: (e, t) => {
			e.addNode("image", void 0, void 0, {
				title: t.attrs.title,
				url: t.attrs.src,
				alt: t.attrs.alt
			});
		}
	}
}));
X(cv.node, {
	displayName: "NodeSchema<image>",
	group: "Image"
}), X(cv.ctx, {
	displayName: "NodeSchemaCtx<image>",
	group: "Image"
});
var lv = J("InsertImage", (e) => (t = {}) => (n, r) => {
	if (!r) return !0;
	let { src: i = "", alt: a = "", title: o = "" } = t, s = cv.type(e).create({
		src: i,
		alt: a,
		title: o
	});
	return s && r(n.tr.replaceSelectionWith(s).scrollIntoView()), !0;
});
X(lv, {
	displayName: "Command<insertImageCommand>",
	group: "Image"
});
var uv = J("UpdateImage", (e) => (t = {}) => (n, r) => {
	let i = Ou(n.selection, cv.type(e));
	if (!i) return !1;
	let { node: a, pos: o } = i, s = { ...a.attrs }, { src: c, alt: l, title: u } = t;
	return c !== void 0 && (s.src = c), l !== void 0 && (s.alt = l), u !== void 0 && (s.title = u), r?.(n.tr.setNodeMarkup(o, void 0, s).scrollIntoView()), !0;
});
X(uv, {
	displayName: "Command<updateImageCommand>",
	group: "Image"
}), X(Ng((e) => new tu(/!\[(?<alt>.*?)]\((?<filename>.*?)\s*(?="|\))"?(?<title>[^"]+)?"?\)/, (t, n, r, i) => {
	let [a, o, s = "", c] = n;
	return a ? t.tr.replaceWith(r, i, cv.type(e).create({
		src: s,
		alt: o,
		title: c
	})) : null;
})), {
	displayName: "InputRule<insertImageInputRule>",
	group: "Image"
});
var dv = Hg("hardbreak", (e) => ({
	"data-type": "hardbreak",
	"data-is-inline": e.attrs.isInline
}));
X(dv, {
	displayName: "Attr<hardbreak>",
	group: "Hardbreak"
});
var fv = Y("hardbreak", (e) => ({
	inline: !0,
	group: "inline",
	attrs: { isInline: {
		default: !1,
		validate: "boolean"
	} },
	selectable: !1,
	parseDOM: [{ tag: "br" }, {
		tag: "span[data-type=\"hardbreak\"]",
		getAttrs: () => ({ isInline: !0 })
	}],
	toDOM: (t) => t.attrs.isInline ? [
		"span",
		e.get(dv.key)(t),
		" "
	] : ["br", e.get(dv.key)(t)],
	parseMarkdown: {
		match: ({ type: e }) => e === "break",
		runner: (e, t, n) => {
			e.addNode(n, { isInline: !!t.data?.isInline });
		}
	},
	leafText: () => "\n",
	toMarkdown: {
		match: (e) => e.type.name === "hardbreak",
		runner: (e, t) => {
			t.attrs.isInline ? e.addNode("text", void 0, "\n") : e.addNode("break");
		}
	}
}));
X(fv.node, {
	displayName: "NodeSchema<hardbreak>",
	group: "Hardbreak"
}), X(fv.ctx, {
	displayName: "NodeSchemaCtx<hardbreak>",
	group: "Hardbreak"
});
var pv = J("InsertHardbreak", (e) => () => (t, n) => {
	let { selection: r, tr: i } = t;
	if (!(r instanceof R)) return !1;
	if (r.empty) {
		let e = r.$from.node();
		if (e.childCount > 0 && e.lastChild?.type.name === "hardbreak") return n?.(i.replaceRangeWith(r.to - 1, r.to, t.schema.node("paragraph")).setSelection(L.near(i.doc.resolve(r.to))).scrollIntoView()), !0;
	}
	return n?.(i.setMeta("hardbreak", !0).replaceSelectionWith(fv.type(e).create()).scrollIntoView()), !0;
});
X(pv, {
	displayName: "Command<insertHardbreakCommand>",
	group: "Hardbreak"
});
var mv = Vg("hardbreakKeymap", { InsertHardbreak: {
	shortcuts: "Shift-Enter",
	command: (e) => {
		let t = e.get(q);
		return () => t.call(pv.key);
	}
} });
X(mv.ctx, {
	displayName: "KeymapCtx<hardbreak>",
	group: "Hardbreak"
}), X(mv.shortcuts, {
	displayName: "Keymap<hardbreak>",
	group: "Hardbreak"
});
var hv = Hg("hr");
X(hv, {
	displayName: "Attr<hr>",
	group: "Hr"
});
var gv = Y("hr", (e) => ({
	group: "block",
	parseDOM: [{ tag: "hr" }],
	toDOM: (t) => ["hr", e.get(hv.key)(t)],
	parseMarkdown: {
		match: ({ type: e }) => e === "thematicBreak",
		runner: (e, t, n) => {
			e.addNode(n);
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "hr",
		runner: (e) => {
			e.addNode("thematicBreak");
		}
	}
}));
X(gv.node, {
	displayName: "NodeSchema<hr>",
	group: "Hr"
}), X(gv.ctx, {
	displayName: "NodeSchemaCtx<hr>",
	group: "Hr"
});
var _v = Ng((e) => new tu(/^(?:---|___\s|\*\*\*\s)$/, (t, n, r, i) => {
	let { tr: a } = t;
	return n[0] && a.replaceWith(r - 1, i, gv.type(e).create()), a;
}));
X(_v, {
	displayName: "InputRule<insertHrInputRule>",
	group: "Hr"
});
var vv = J("InsertHr", (e) => () => (t, n) => {
	if (!n) return !0;
	let r = R_.node.type(e).create(), { tr: i, selection: a } = t, { from: o } = a, s = gv.type(e).create();
	if (!s) return !0;
	let c = i.replaceSelectionWith(s).insert(o, r), l = L.findFrom(c.doc.resolve(o), 1, !0);
	return l && n(c.setSelection(l).scrollIntoView()), !0;
});
X(vv, {
	displayName: "Command<insertHrCommand>",
	group: "Hr"
});
var yv = Hg("bulletList");
X(yv, {
	displayName: "Attr<bulletList>",
	group: "BulletList"
});
var bv = Y("bullet_list", (e) => ({
	content: "listItem+",
	group: "block",
	attrs: { spread: {
		default: !1,
		validate: "boolean"
	} },
	parseDOM: [{
		tag: "ul",
		getAttrs: (e) => {
			if (!(e instanceof HTMLElement)) throw pe(e);
			return { spread: e.dataset.spread === "true" };
		}
	}],
	toDOM: (t) => [
		"ul",
		{
			...e.get(yv.key)(t),
			"data-spread": t.attrs.spread
		},
		0
	],
	parseMarkdown: {
		match: ({ type: e, ordered: t }) => e === "list" && !t,
		runner: (e, t, n) => {
			let r = t.spread ?? !1;
			e.openNode(n, { spread: r }).next(t.children).closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "bullet_list",
		runner: (e, t) => {
			e.openNode("list", void 0, {
				ordered: !1,
				spread: t.attrs.spread
			}).next(t.content).closeNode();
		}
	}
}));
X(bv.node, {
	displayName: "NodeSchema<bulletList>",
	group: "BulletList"
}), X(bv.ctx, {
	displayName: "NodeSchemaCtx<bulletList>",
	group: "BulletList"
});
var xv = Ng((e) => iu(/^\s*([-+*])\s$/, bv.type(e)));
X(xv, {
	displayName: "InputRule<wrapInBulletListInputRule>",
	group: "BulletList"
});
var Sv = J("WrapInBulletList", (e) => () => Wl(bv.type(e)));
X(Sv, {
	displayName: "Command<wrapInBulletListCommand>",
	group: "BulletList"
});
var Cv = Vg("bulletListKeymap", { WrapInBulletList: {
	shortcuts: "Mod-Alt-8",
	command: (e) => {
		let t = e.get(q);
		return () => t.call(Sv.key);
	}
} });
X(Cv.ctx, {
	displayName: "KeymapCtx<bulletListKeymap>",
	group: "BulletList"
}), X(Cv.shortcuts, {
	displayName: "Keymap<bulletListKeymap>",
	group: "BulletList"
});
var wv = Hg("orderedList");
X(wv, {
	displayName: "Attr<orderedList>",
	group: "OrderedList"
});
var Tv = Y("ordered_list", (e) => ({
	content: "listItem+",
	group: "block",
	attrs: {
		order: {
			default: 1,
			validate: "number"
		},
		spread: {
			default: !1,
			validate: "boolean"
		}
	},
	parseDOM: [{
		tag: "ol",
		getAttrs: (e) => {
			if (!(e instanceof HTMLElement)) throw pe(e);
			return {
				spread: e.dataset.spread === "true",
				order: e.hasAttribute("start") ? Number(e.getAttribute("start")) : 1
			};
		}
	}],
	toDOM: (t) => [
		"ol",
		{
			...e.get(wv.key)(t),
			...t.attrs.order === 1 ? {} : { start: t.attrs.order },
			"data-spread": t.attrs.spread
		},
		0
	],
	parseMarkdown: {
		match: ({ type: e, ordered: t }) => e === "list" && !!t,
		runner: (e, t, n) => {
			let r = t.spread ?? !0;
			e.openNode(n, {
				spread: r,
				order: t.start ?? 1
			}).next(t.children).closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "ordered_list",
		runner: (e, t) => {
			e.openNode("list", void 0, {
				ordered: !0,
				start: t.attrs.order ?? 1,
				spread: t.attrs.spread
			}), e.next(t.content), e.closeNode();
		}
	}
}));
X(Tv.node, {
	displayName: "NodeSchema<orderedList>",
	group: "OrderedList"
}), X(Tv.ctx, {
	displayName: "NodeSchemaCtx<orderedList>",
	group: "OrderedList"
});
var Ev = Ng((e) => iu(/^\s*(\d+)\.\s$/, Tv.type(e), (e) => ({ order: Number(e[1]) }), (e, t) => t.childCount + t.attrs.order === Number(e[1])));
X(Ev, {
	displayName: "InputRule<wrapInOrderedListInputRule>",
	group: "OrderedList"
});
var Dv = J("WrapInOrderedList", (e) => () => Wl(Tv.type(e)));
X(Dv, {
	displayName: "Command<wrapInOrderedListCommand>",
	group: "OrderedList"
});
var Ov = Vg("orderedListKeymap", { WrapInOrderedList: {
	shortcuts: "Mod-Alt-7",
	command: (e) => {
		let t = e.get(q);
		return () => t.call(Dv.key);
	}
} });
X(Ov.ctx, {
	displayName: "KeymapCtx<orderedList>",
	group: "OrderedList"
}), X(Ov.shortcuts, {
	displayName: "Keymap<orderedList>",
	group: "OrderedList"
});
var kv = Hg("listItem");
X(kv, {
	displayName: "Attr<listItem>",
	group: "ListItem"
});
var Av = Y("list_item", (e) => ({
	group: "listItem",
	content: "paragraph block*",
	attrs: {
		label: {
			default: "•",
			validate: "string"
		},
		listType: {
			default: "bullet",
			validate: "string"
		},
		spread: {
			default: !0,
			validate: "boolean"
		}
	},
	defining: !0,
	parseDOM: [{
		tag: "li",
		getAttrs: (e) => {
			if (!(e instanceof HTMLElement)) throw pe(e);
			return {
				label: e.dataset.label,
				listType: e.dataset.listType,
				spread: e.dataset.spread === "true"
			};
		}
	}],
	toDOM: (t) => [
		"li",
		{
			...e.get(kv.key)(t),
			"data-label": t.attrs.label,
			"data-list-type": t.attrs.listType,
			"data-spread": t.attrs.spread
		},
		0
	],
	parseMarkdown: {
		match: ({ type: e }) => e === "listItem",
		runner: (e, t, n) => {
			let r = t.label == null ? "•" : `${t.label}.`, i = t.label == null ? "bullet" : "ordered", a = t.spread ?? !0;
			e.openNode(n, {
				label: r,
				listType: i,
				spread: a
			}), e.next(t.children), e.closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "list_item",
		runner: (e, t) => {
			e.openNode("listItem", void 0, { spread: t.attrs.spread }), e.next(t.content), e.closeNode();
		}
	}
}));
X(Av.node, {
	displayName: "NodeSchema<listItem>",
	group: "ListItem"
}), X(Av.ctx, {
	displayName: "NodeSchemaCtx<listItem>",
	group: "ListItem"
});
var jv = J("SinkListItem", (e) => () => Yg(Av.type(e)));
X(jv, {
	displayName: "Command<sinkListItemCommand>",
	group: "ListItem"
});
var Mv = J("LiftListItem", (e) => () => Kg(Av.type(e)));
X(Mv, {
	displayName: "Command<liftListItemCommand>",
	group: "ListItem"
});
var Nv = J("SplitListItem", (e) => () => Gg(Av.type(e)));
X(Nv, {
	displayName: "Command<splitListItemCommand>",
	group: "ListItem"
});
function Pv(e) {
	return (t, n, r) => {
		let { selection: i } = t;
		if (!(i instanceof R)) return !1;
		let { empty: a, $from: o } = i;
		return !a || o.parentOffset !== 0 || o.node(-1).type !== Av.type(e) ? !1 : xl(t, n, r);
	};
}
var Fv = J("LiftFirstListItem", (e) => () => Pv(e));
X(Fv, {
	displayName: "Command<liftFirstListItemCommand>",
	group: "ListItem"
});
var Iv = Vg("listItemKeymap", {
	NextListItem: {
		shortcuts: "Enter",
		command: (e) => {
			let t = e.get(q);
			return () => t.call(Nv.key);
		}
	},
	SinkListItem: {
		shortcuts: ["Tab", "Mod-]"],
		command: (e) => {
			let t = e.get(q);
			return () => t.call(jv.key);
		}
	},
	LiftListItem: {
		shortcuts: ["Shift-Tab", "Mod-["],
		command: (e) => {
			let t = e.get(q);
			return () => t.call(Mv.key);
		}
	},
	LiftFirstListItem: {
		shortcuts: ["Backspace", "Delete"],
		command: (e) => {
			let t = e.get(q);
			return () => t.call(Fv.key);
		}
	}
});
X(Iv.ctx, {
	displayName: "KeymapCtx<listItem>",
	group: "ListItem"
}), X(Iv.shortcuts, {
	displayName: "Keymap<listItem>",
	group: "ListItem"
});
var Lv = Ig("text", () => ({
	group: "inline",
	parseMarkdown: {
		match: ({ type: e }) => e === "text",
		runner: (e, t) => {
			e.addText(t.value);
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "text",
		runner: (e, t) => {
			e.addNode("text", void 0, t.text);
		}
	}
}));
X(Lv, {
	displayName: "NodeSchema<text>",
	group: "Text"
});
var Rv = Hg("html");
X(Rv, {
	displayName: "Attr<html>",
	group: "Html"
});
var zv = Y("html", (e) => ({
	atom: !0,
	group: "inline",
	inline: !0,
	attrs: { value: {
		default: "",
		validate: "string"
	} },
	toDOM: (t) => {
		let n = document.createElement("span"), r = {
			...e.get(Rv.key)(t),
			"data-value": t.attrs.value,
			"data-type": "html"
		};
		return n.textContent = t.attrs.value, [
			"span",
			r,
			t.attrs.value
		];
	},
	parseDOM: [{
		tag: "span[data-type=\"html\"]",
		getAttrs: (e) => ({ value: e.dataset.value ?? "" })
	}],
	parseMarkdown: {
		match: ({ type: e }) => e === "html",
		runner: (e, t, n) => {
			e.addNode(n, { value: t.value });
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "html",
		runner: (e, t) => {
			e.addNode("html", void 0, t.attrs.value);
		}
	}
}));
X(zv.node, {
	displayName: "NodeSchema<html>",
	group: "Html"
}), X(zv.ctx, {
	displayName: "NodeSchemaCtx<html>",
	group: "Html"
});
var Bv = [
	O_,
	L_,
	R_,
	W_,
	G_,
	K_,
	dv,
	fv,
	Z_,
	Q_,
	nv,
	rv,
	hv,
	gv,
	sv,
	cv,
	yv,
	bv,
	wv,
	Tv,
	kv,
	Av,
	i_,
	a_,
	u_,
	d_,
	h_,
	g_,
	w_,
	T_,
	Rv,
	zv,
	Lv
].flat(), Vv = [
	$_,
	xv,
	Ev,
	iv,
	_v,
	q_
].flat(), Hv = [
	s_,
	c_,
	v_,
	p_
], Uv = [
	B_,
	ev,
	J_,
	Y_,
	av,
	pv,
	vv,
	lv,
	uv,
	Dv,
	Sv,
	jv,
	Nv,
	Mv,
	Fv,
	o_,
	__,
	f_,
	E_,
	D_,
	J("IsMarkSelected", () => (e) => (t) => {
		if (!e) return !1;
		let { doc: n, selection: r } = t;
		return n.rangeHasMark(r.from, r.to, e);
	}),
	J("IsNoteSelected", () => (e) => (t) => e ? ku(t, e).hasNode : !1),
	J("ClearTextInCurrentBlock", () => () => (e, t) => {
		let n = e.tr, { $from: r, $to: i } = n.selection, { pos: a } = r, { pos: o } = i, s = a - r.node().content.size;
		return s < 0 ? !1 : (n = n.deleteRange(s, o), t?.(n), !0);
	}),
	J("SetBlockType", () => (e) => (t, n) => {
		let { nodeType: r, attrs: i = null } = e ?? {};
		if (!r) return !1;
		let a = t.tr, { from: o, to: s } = a.selection;
		try {
			a.setBlockType(o, s, r, i);
		} catch {
			return !1;
		}
		return n?.(a), !0;
	}),
	J("WrapInBlockType", () => (e) => (t, n) => {
		let { nodeType: r, attrs: i = null } = e ?? {};
		if (!r) return !1;
		let a = t.tr;
		try {
			let { $from: e, $to: t } = a.selection, n = e.blockRange(t), o = n && dc(n, r, i);
			if (!o) return !1;
			a = a.wrap(n, o);
		} catch {
			return !1;
		}
		return n?.(a), !0;
	}),
	J("AddBlockType", () => (e) => (t, n) => {
		let { nodeType: r, attrs: i = null } = e ?? {};
		if (!r) return !1;
		let a = t.tr;
		try {
			let e = r instanceof Bo ? r : r.createAndFill(i);
			if (!e) return !1;
			a.replaceSelectionWith(e);
		} catch {
			return !1;
		}
		return n?.(a), !0;
	}),
	J("SelectTextNearPos", () => (e) => (t, n) => {
		let { pos: r } = e ?? {};
		if (r == null) return !1;
		let i = (e, t, n) => Math.min(Math.max(e, t), n), a = t.tr;
		try {
			let e = t.doc.resolve(i(r, 0, t.doc.content.size));
			a.setSelection(R.near(e));
		} catch {
			return !1;
		}
		return n?.(a.scrollIntoView()), !0;
	})
], Wv = [
	tv,
	ov,
	mv,
	X_,
	Iv,
	Ov,
	Cv,
	V_,
	l_,
	y_,
	m_
].flat(), Gv = Wg("remarkAddOrderInList", () => () => (e) => {
	ui(e, "list", (e) => {
		if (e.ordered) {
			let t = e.start ?? 1;
			e.children.forEach((e, n) => {
				e.label = n + t;
			});
		}
	});
});
X(Gv.plugin, {
	displayName: "Remark<remarkAddOrderInListPlugin>",
	group: "Remark"
}), X(Gv.options, {
	displayName: "RemarkConfig<remarkAddOrderInListPlugin>",
	group: "Remark"
});
var Kv = Wg("remarkLineBreak", () => () => (e) => {
	let t = /[\t ]*(?:\r?\n|\r)/g;
	ui(e, "text", (e, n, r) => {
		if (!e.value || typeof e.value != "string") return;
		let i = [], a = 0;
		t.lastIndex = 0;
		let o = t.exec(e.value);
		for (; o;) {
			let n = o.index;
			a !== n && i.push({
				type: "text",
				value: e.value.slice(a, n)
			}), i.push({
				type: "break",
				data: { isInline: !0 }
			}), a = n + o[0].length, o = t.exec(e.value);
		}
		if (i.length > 0 && r && typeof n == "number") return a < e.value.length && i.push({
			type: "text",
			value: e.value.slice(a)
		}), r.children.splice(n, 1, ...i), n + i.length;
	});
});
X(Kv.plugin, {
	displayName: "Remark<remarkLineBreak>",
	group: "Remark"
}), X(Kv.options, {
	displayName: "RemarkConfig<remarkLineBreak>",
	group: "Remark"
});
var qv = Wg("remarkInlineLink", () => Qg);
X(qv.plugin, {
	displayName: "Remark<remarkInlineLinkPlugin>",
	group: "Remark"
}), X(qv.options, {
	displayName: "RemarkConfig<remarkInlineLinkPlugin>",
	group: "Remark"
});
var Jv = (e) => !!e.children, Yv = (e) => e.type === "html";
function Xv(e, t) {
	return n(e, 0, null)[0];
	function n(e, r, i) {
		if (Jv(e)) {
			let t = [];
			for (let r = 0, i = e.children.length; r < i; r++) {
				let i = e.children[r];
				if (i) {
					let a = n(i, r, e);
					if (a) for (let e = 0, n = a.length; e < n; e++) {
						let n = a[e];
						n && t.push(n);
					}
				}
			}
			e.children = t;
		}
		return t(e, r, i);
	}
}
var Zv = Wg("remarkHTMLTransformer", (e) => () => (t) => {
	let n = N_(e);
	Xv(t, (e, t, r) => (Yv(e) && r && n.has(r.type) && (e.children = [$g(e)], delete e.value, e.type = "paragraph"), [e]));
});
X(Zv.plugin, {
	displayName: "Remark<remarkHtmlTransformer>",
	group: "Remark"
}), X(Zv.options, {
	displayName: "RemarkConfig<remarkHtmlTransformer>",
	group: "Remark"
});
var Qv = Wg("remarkMarker", () => () => (e, t) => {
	let n = (e) => t.value.charAt(e.position.start.offset);
	ui(e, (e) => ["strong", "emphasis"].includes(e.type), (e) => {
		e.marker = n(e);
	});
});
X(Qv.plugin, {
	displayName: "Remark<remarkMarker>",
	group: "Remark"
}), X(Qv.options, {
	displayName: "RemarkConfig<remarkMarker>",
	group: "Remark"
});
var $v = Lg(() => {
	let e = !1, t = new hl({
		key: new vl("MILKDOWN_INLINE_NODES_CURSOR"),
		state: {
			init() {
				return !1;
			},
			apply(e) {
				if (!e.selection.empty) return !1;
				let t = e.selection.$from, n = t.nodeBefore, r = t.nodeAfter;
				return !!(n && r && n.isInline && !n.isText && r.isInline && !r.isText);
			}
		},
		props: {
			handleDOMEvents: {
				compositionend: (n, r) => e ? (e = !1, requestAnimationFrame(() => {
					if (t.getState(n.state)) {
						let e = n.state.selection.from;
						r.preventDefault(), n.dispatch(n.state.tr.insertText(r.data || "", e));
					}
				}), !0) : !1,
				compositionstart: (n) => (t.getState(n.state) && (e = !0), !1),
				beforeinput: (n, r) => {
					if (t.getState(n.state) && r instanceof InputEvent && r.data && !e) {
						let e = n.state.selection.from;
						return r.preventDefault(), n.dispatch(n.state.tr.insertText(r.data || "", e)), !0;
					}
					return !1;
				}
			},
			decorations(e) {
				if (t.getState(e)) {
					let t = e.selection.$from.pos, n = document.createElement("span"), r = Dm.widget(t, n, { side: -1 }), i = document.createElement("span"), a = Dm.widget(t, i);
					return setTimeout(() => {
						n.contentEditable = "true", i.contentEditable = "true";
					}), K.create(e.doc, [r, a]);
				}
				return K.empty;
			}
		}
	});
	return t;
});
X($v, {
	displayName: "Prose<inlineNodesCursorPlugin>",
	group: "Prose"
});
var ey = Lg((e) => new hl({
	key: new vl("MILKDOWN_HARDBREAK_MARKS"),
	appendTransaction: (t, n, r) => {
		if (!t.length) return;
		let [i] = t;
		if (!i) return;
		let [a] = i.steps;
		if (i.getMeta("hardbreak")) {
			if (!(a instanceof nc)) return;
			let { from: t } = a;
			return r.tr.setNodeMarkup(t, fv.type(e), void 0, []);
		}
		if (a instanceof Qs) {
			let t = r.tr, { from: n, to: i } = a;
			return r.doc.nodesBetween(n, i, (n, r) => {
				n.type === fv.type(e) && (t = t.setNodeMarkup(r, fv.type(e), void 0, []));
			}), t;
		}
	}
}));
X(ey, {
	displayName: "Prose<hardbreakClearMarkPlugin>",
	group: "Prose"
});
var ty = zg(["table", "code_block"], "hardbreakFilterNodes");
X(ty, {
	displayName: "Ctx<hardbreakFilterNodes>",
	group: "Prose"
});
var ny = Lg((e) => {
	let t = e.get(ty.key);
	return new hl({
		key: new vl("MILKDOWN_HARDBREAK_FILTER"),
		filterTransaction: (e, n) => {
			let r = e.getMeta("hardbreak"), [i] = e.steps;
			if (r && i) {
				let { from: e } = i, r = n.doc.resolve(e), a = r.depth, o = !0;
				for (; a > 0;) t.includes(r.node(a).type.name) && (o = !1), a--;
				return o;
			}
			return !0;
		}
	});
});
X(ny, {
	displayName: "Prose<hardbreakFilterPlugin>",
	group: "Prose"
});
var ry = Lg((e) => {
	let t = new vl("MILKDOWN_HEADING_ID"), n = (n) => {
		if (n.composing) return;
		let r = e.get(W_.key), i = n.state.tr.setMeta("addToHistory", !1), a = !1, o = {};
		n.state.doc.descendants((n, s) => {
			if (n.type === K_.type(e)) {
				if (n.textContent.trim().length === 0) return;
				let e = n.attrs, c = r(n);
				o[c] ? (o[c] += 1, c += `-#${o[c]}`) : o[c] = 1, e.id !== c && (a = !0, i.setMeta(t, !0).setNodeMarkup(s, void 0, {
					...e,
					id: c
				}));
			}
		}), a && n.dispatch(i);
	};
	return new hl({
		key: t,
		view: (e) => (n(e), { update: (e, t) => {
			e.state.doc.eq(t.doc) || n(e);
		} })
	});
});
X(ry, {
	displayName: "Prose<syncHeadingIdPlugin>",
	group: "Prose"
});
var iy = Lg((e) => new hl({
	key: new vl("MILKDOWN_KEEP_LIST_ORDER"),
	appendTransaction: (t, n, r) => {
		if (!r.selection || t.some((e) => e.getMeta("addToHistory") === !1 || !e.isGeneric)) return null;
		let i = Tv.type(e), a = bv.type(e), o = Av.type(e), s = (e, t, n = 1) => {
			let r = !1, i = `${t + n}.`;
			return e.label !== i && (e.label = i, r = !0), r;
		}, c = r.tr, l = !1;
		return r.doc.descendants((e, t, n, r) => {
			if (e.type === a) {
				let n = e.maybeChild(0);
				n?.type === o && n.attrs.listType === "ordered" && (l = !0, c.setNodeMarkup(t, i, { spread: !0 }), e.descendants((e, t, n, r) => {
					if (e.type === o) {
						let n = { ...e.attrs };
						s(n, r) && (c = c.setNodeMarkup(t, void 0, n));
					}
					return !1;
				}));
			} else if (e.type === o && n?.type === i) {
				let i = { ...e.attrs }, a = !1;
				i.listType !== "ordered" && (i.listType = "ordered", a = !0), n?.maybeChild(0) && (a = s(i, r, n?.attrs.order ?? 1)), a && (c = c.setNodeMarkup(t, void 0, i), l = !0);
			}
		}), l ? c.setMeta("addToHistory", !1) : null;
	}
}));
X(iy, {
	displayName: "Prose<syncListOrderPlugin>",
	group: "Prose"
});
var ay = [
	Bv,
	Vv,
	Hv,
	Uv,
	Wv,
	[
		ey,
		ty,
		ny,
		$v,
		j_,
		Gv,
		qv,
		Kv,
		Zv,
		Qv,
		I_,
		ry,
		iy
	].flat()
].flat(), oy, sy;
if (typeof WeakMap < "u") {
	let e = /* @__PURE__ */ new WeakMap();
	oy = (t) => e.get(t), sy = (t, n) => (e.set(t, n), n);
} else {
	let e = [], t = 0;
	oy = (t) => {
		for (let n = 0; n < e.length; n += 2) if (e[n] == t) return e[n + 1];
	}, sy = (n, r) => (t == 10 && (t = 0), e[t++] = n, e[t++] = r);
}
var Z = class {
	constructor(e, t, n, r) {
		this.width = e, this.height = t, this.map = n, this.problems = r;
	}
	findCell(e) {
		for (let t = 0; t < this.map.length; t++) {
			let n = this.map[t];
			if (n != e) continue;
			let r = t % this.width, i = t / this.width | 0, a = r + 1, o = i + 1;
			for (let e = 1; a < this.width && this.map[t + e] == n; e++) a++;
			for (let e = 1; o < this.height && this.map[t + this.width * e] == n; e++) o++;
			return {
				left: r,
				top: i,
				right: a,
				bottom: o
			};
		}
		throw RangeError(`No cell with offset ${e} found`);
	}
	colCount(e) {
		for (let t = 0; t < this.map.length; t++) if (this.map[t] == e) return t % this.width;
		throw RangeError(`No cell with offset ${e} found`);
	}
	nextCell(e, t, n) {
		let { left: r, right: i, top: a, bottom: o } = this.findCell(e);
		return t == "horiz" ? (n < 0 ? r == 0 : i == this.width) ? null : this.map[a * this.width + (n < 0 ? r - 1 : i)] : (n < 0 ? a == 0 : o == this.height) ? null : this.map[r + this.width * (n < 0 ? a - 1 : o)];
	}
	rectBetween(e, t) {
		let { left: n, right: r, top: i, bottom: a } = this.findCell(e), { left: o, right: s, top: c, bottom: l } = this.findCell(t);
		return {
			left: Math.min(n, o),
			top: Math.min(i, c),
			right: Math.max(r, s),
			bottom: Math.max(a, l)
		};
	}
	cellsInRect(e) {
		let t = [], n = {};
		for (let r = e.top; r < e.bottom; r++) for (let i = e.left; i < e.right; i++) {
			let a = r * this.width + i, o = this.map[a];
			n[o] || (n[o] = !0, !(i == e.left && i && this.map[a - 1] == o || r == e.top && r && this.map[a - this.width] == o) && t.push(o));
		}
		return t;
	}
	positionAt(e, t, n) {
		for (let r = 0, i = 0;; r++) {
			let a = i + n.child(r).nodeSize;
			if (r == e) {
				let n = t + e * this.width, r = (e + 1) * this.width;
				for (; n < r && this.map[n] < i;) n++;
				return n == r ? a - 1 : this.map[n];
			}
			i = a;
		}
	}
	static get(e) {
		return oy(e) || sy(e, cy(e));
	}
};
function cy(e) {
	if (e.type.spec.tableRole != "table") throw RangeError("Not a table node: " + e.type.name);
	let t = ly(e), n = e.childCount, r = [], i = 0, a = null, o = [];
	for (let e = 0, i = t * n; e < i; e++) r[e] = 0;
	for (let s = 0, c = 0; s < n; s++) {
		let l = e.child(s);
		c++;
		for (let e = 0;; e++) {
			for (; i < r.length && r[i] != 0;) i++;
			if (e == l.childCount) break;
			let u = l.child(e), { colspan: d, rowspan: f, colwidth: p } = u.attrs;
			for (let e = 0; e < f; e++) {
				if (e + s >= n) {
					(a ||= []).push({
						type: "overlong_rowspan",
						pos: c,
						n: f - e
					});
					break;
				}
				let l = i + e * t;
				for (let e = 0; e < d; e++) {
					r[l + e] == 0 ? r[l + e] = c : (a ||= []).push({
						type: "collision",
						row: s,
						pos: c,
						n: d - e
					});
					let n = p && p[e];
					if (n) {
						let r = (l + e) % t * 2, i = o[r];
						i == null || i != n && o[r + 1] == 1 ? (o[r] = n, o[r + 1] = 1) : i == n && o[r + 1]++;
					}
				}
			}
			i += d, c += u.nodeSize;
		}
		let u = (s + 1) * t, d = 0;
		for (; i < u;) r[i++] == 0 && d++;
		d && (a ||= []).push({
			type: "missing",
			row: s,
			n: d
		}), c++;
	}
	(t === 0 || n === 0) && (a ||= []).push({ type: "zero_sized" });
	let s = new Z(t, n, r, a), c = !1;
	for (let e = 0; !c && e < o.length; e += 2) o[e] != null && o[e + 1] < n && (c = !0);
	return c && uy(s, o, e), s;
}
function ly(e) {
	let t = -1, n = !1;
	for (let r = 0; r < e.childCount; r++) {
		let i = e.child(r), a = 0;
		if (n) for (let t = 0; t < r; t++) {
			let n = e.child(t);
			for (let e = 0; e < n.childCount; e++) {
				let i = n.child(e);
				t + i.attrs.rowspan > r && (a += i.attrs.colspan);
			}
		}
		for (let e = 0; e < i.childCount; e++) {
			let t = i.child(e);
			a += t.attrs.colspan, t.attrs.rowspan > 1 && (n = !0);
		}
		t == -1 ? t = a : t != a && (t = Math.max(t, a));
	}
	return t;
}
function uy(e, t, n) {
	e.problems ||= [];
	let r = {};
	for (let i = 0; i < e.map.length; i++) {
		let a = e.map[i];
		if (r[a]) continue;
		r[a] = !0;
		let o = n.nodeAt(a);
		if (!o) throw RangeError(`No cell with offset ${a} found`);
		let s = null, c = o.attrs;
		for (let n = 0; n < c.colspan; n++) {
			let r = t[(i + n) % e.width * 2];
			r != null && (!c.colwidth || c.colwidth[n] != r) && ((s ||= dy(c))[n] = r);
		}
		s && e.problems.unshift({
			type: "colwidth mismatch",
			pos: a,
			colwidth: s
		});
	}
}
function dy(e) {
	if (e.colwidth) return e.colwidth.slice();
	let t = [];
	for (let n = 0; n < e.colspan; n++) t.push(0);
	return t;
}
function fy(e, t) {
	if (typeof e == "string") return {};
	let n = e.getAttribute("data-colwidth"), r = n && /^\d+(,\d+)*$/.test(n) ? n.split(",").map((e) => Number(e)) : null, i = Number(e.getAttribute("colspan") || 1), a = {
		colspan: i,
		rowspan: Number(e.getAttribute("rowspan") || 1),
		colwidth: r && r.length == i ? r : null
	};
	for (let n in t) {
		let r = t[n].getFromDOM, i = r && r(e);
		i != null && (a[n] = i);
	}
	return a;
}
function py(e, t) {
	let n = {};
	e.attrs.colspan != 1 && (n.colspan = e.attrs.colspan), e.attrs.rowspan != 1 && (n.rowspan = e.attrs.rowspan), e.attrs.colwidth && (n["data-colwidth"] = e.attrs.colwidth.join(","));
	for (let r in t) {
		let i = t[r].setDOMAttr;
		i && i(e.attrs[r], n);
	}
	return n;
}
function my(e) {
	if (e !== null) {
		if (!Array.isArray(e)) throw TypeError("colwidth must be null or an array");
		for (let t of e) if (typeof t != "number") throw TypeError("colwidth must be null or an array of numbers");
	}
}
function hy(e) {
	let t = e.cellAttributes || {}, n = {
		colspan: {
			default: 1,
			validate: "number"
		},
		rowspan: {
			default: 1,
			validate: "number"
		},
		colwidth: {
			default: null,
			validate: my
		}
	};
	for (let e in t) n[e] = {
		default: t[e].default,
		validate: t[e].validate
	};
	return {
		table: {
			content: "table_row+",
			tableRole: "table",
			isolating: !0,
			group: e.tableGroup,
			parseDOM: [{ tag: "table" }],
			toDOM() {
				return ["table", ["tbody", 0]];
			}
		},
		table_row: {
			content: "(table_cell | table_header)*",
			tableRole: "row",
			parseDOM: [{ tag: "tr" }],
			toDOM() {
				return ["tr", 0];
			}
		},
		table_cell: {
			content: e.cellContent,
			attrs: n,
			tableRole: "cell",
			isolating: !0,
			parseDOM: [{
				tag: "td",
				getAttrs: (e) => fy(e, t)
			}],
			toDOM(e) {
				return [
					"td",
					py(e, t),
					0
				];
			}
		},
		table_header: {
			content: e.cellContent,
			attrs: n,
			tableRole: "header_cell",
			isolating: !0,
			parseDOM: [{
				tag: "th",
				getAttrs: (e) => fy(e, t)
			}],
			toDOM(e) {
				return [
					"th",
					py(e, t),
					0
				];
			}
		}
	};
}
function gy(e) {
	let t = e.cached.tableNodeTypes;
	if (!t) {
		t = e.cached.tableNodeTypes = {};
		for (let n in e.nodes) {
			let r = e.nodes[n], i = r.spec.tableRole;
			i && (t[i] = r);
		}
	}
	return t;
}
var _y = new vl("selectingCells");
function vy(e) {
	for (let t = e.depth - 1; t > 0; t--) if (e.node(t).type.spec.tableRole == "row") return e.node(0).resolve(e.before(t + 1));
	return null;
}
function yy(e) {
	let t = e.selection.$head;
	for (let e = t.depth; e > 0; e--) if (t.node(e).type.spec.tableRole == "row") return !0;
	return !1;
}
function by(e) {
	let t = e.selection;
	if ("$anchorCell" in t && t.$anchorCell) return t.$anchorCell.pos > t.$headCell.pos ? t.$anchorCell : t.$headCell;
	if ("node" in t && t.node && t.node.type.spec.tableRole == "cell") return t.$anchor;
	let n = vy(t.$head) || xy(t.$head);
	if (n) return n;
	throw RangeError(`No cell found around position ${t.head}`);
}
function xy(e) {
	for (let t = e.nodeAfter, n = e.pos; t; t = t.firstChild, n++) {
		let r = t.type.spec.tableRole;
		if (r == "cell" || r == "header_cell") return e.doc.resolve(n);
	}
	for (let t = e.nodeBefore, n = e.pos; t; t = t.lastChild, n--) {
		let r = t.type.spec.tableRole;
		if (r == "cell" || r == "header_cell") return e.doc.resolve(n - t.nodeSize);
	}
}
function Sy(e) {
	return e.parent.type.spec.tableRole == "row" && !!e.nodeAfter;
}
function Cy(e) {
	return e.node(0).resolve(e.pos + e.nodeAfter.nodeSize);
}
function wy(e, t) {
	return e.depth == t.depth && e.pos >= t.start(-1) && e.pos <= t.end(-1);
}
function Ty(e, t, n) {
	let r = e.node(-1), i = Z.get(r), a = e.start(-1), o = i.nextCell(e.pos - a, t, n);
	return o == null ? null : e.node(0).resolve(a + o);
}
function Ey(e, t, n = 1) {
	let r = {
		...e,
		colspan: e.colspan - n
	};
	return r.colwidth && (r.colwidth = r.colwidth.slice(), r.colwidth.splice(t, n), r.colwidth.some((e) => e > 0) || (r.colwidth = null)), r;
}
function Dy(e, t, n = 1) {
	let r = {
		...e,
		colspan: e.colspan + n
	};
	if (r.colwidth) {
		r.colwidth = r.colwidth.slice();
		for (let e = 0; e < n; e++) r.colwidth.splice(t, 0, 0);
	}
	return r;
}
function Oy(e, t, n) {
	let r = gy(t.type.schema).header_cell;
	for (let i = 0; i < e.height; i++) if (t.nodeAt(e.map[n + i * e.width]).type != r) return !1;
	return !0;
}
var Q = class e extends L {
	constructor(e, t = e) {
		let n = e.node(-1), r = Z.get(n), i = e.start(-1), a = r.rectBetween(e.pos - i, t.pos - i), o = e.node(0), s = r.cellsInRect(a).filter((e) => e != t.pos - i);
		s.unshift(t.pos - i);
		let c = s.map((e) => {
			let t = n.nodeAt(e);
			if (!t) throw RangeError(`No cell with offset ${e} found`);
			let r = i + e + 1;
			return new Xc(o.resolve(r), o.resolve(r + t.content.size));
		});
		super(c[0].$from, c[0].$to, c), this.$anchorCell = e, this.$headCell = t;
	}
	map(t, n) {
		let r = t.resolve(n.map(this.$anchorCell.pos)), i = t.resolve(n.map(this.$headCell.pos));
		if (Sy(r) && Sy(i) && wy(r, i)) {
			let t = this.$anchorCell.node(-1) != r.node(-1);
			return t && this.isRowSelection() ? e.rowSelection(r, i) : t && this.isColSelection() ? e.colSelection(r, i) : new e(r, i);
		}
		return R.between(r, i);
	}
	content() {
		let e = this.$anchorCell.node(-1), t = Z.get(e), n = this.$anchorCell.start(-1), r = t.rectBetween(this.$anchorCell.pos - n, this.$headCell.pos - n), i = {}, a = [];
		for (let n = r.top; n < r.bottom; n++) {
			let o = [];
			for (let a = n * t.width + r.left, s = r.left; s < r.right; s++, a++) {
				let n = t.map[a];
				if (i[n]) continue;
				i[n] = !0;
				let s = t.findCell(n), c = e.nodeAt(n);
				if (!c) throw RangeError(`No cell with offset ${n} found`);
				let l = r.left - s.left, u = s.right - r.right;
				if (l > 0 || u > 0) {
					let e = c.attrs;
					if (l > 0 && (e = Ey(e, 0, l)), u > 0 && (e = Ey(e, e.colspan - u, u)), s.left < r.left) {
						if (c = c.type.createAndFill(e), !c) throw RangeError(`Could not create cell with attrs ${JSON.stringify(e)}`);
					} else c = c.type.create(e, c.content);
				}
				if (s.top < r.top || s.bottom > r.bottom) {
					let e = {
						...c.attrs,
						rowspan: Math.min(s.bottom, r.bottom) - Math.max(s.top, r.top)
					};
					c = s.top < r.top ? c.type.createAndFill(e) : c.type.create(e, c.content);
				}
				o.push(c);
			}
			a.push(e.child(n).copy(N.from(o)));
		}
		let o = this.isColSelection() && this.isRowSelection() ? e : a;
		return new F(N.from(o), 1, 1);
	}
	replace(e, t = F.empty) {
		let n = e.steps.length, r = this.ranges;
		for (let i = 0; i < r.length; i++) {
			let { $from: a, $to: o } = r[i], s = e.mapping.slice(n);
			e.replace(s.map(a.pos), s.map(o.pos), i ? F.empty : t);
		}
		let i = L.findFrom(e.doc.resolve(e.mapping.slice(n).map(this.to)), -1);
		i && e.setSelection(i);
	}
	replaceWith(e, t) {
		this.replace(e, new F(N.from(t), 0, 0));
	}
	forEachCell(e) {
		let t = this.$anchorCell.node(-1), n = Z.get(t), r = this.$anchorCell.start(-1), i = n.cellsInRect(n.rectBetween(this.$anchorCell.pos - r, this.$headCell.pos - r));
		for (let n = 0; n < i.length; n++) e(t.nodeAt(i[n]), r + i[n]);
	}
	isColSelection() {
		let e = this.$anchorCell.index(-1), t = this.$headCell.index(-1);
		if (Math.min(e, t) > 0) return !1;
		let n = e + this.$anchorCell.nodeAfter.attrs.rowspan, r = t + this.$headCell.nodeAfter.attrs.rowspan;
		return Math.max(n, r) == this.$headCell.node(-1).childCount;
	}
	static colSelection(t, n = t) {
		let r = t.node(-1), i = Z.get(r), a = t.start(-1), o = i.findCell(t.pos - a), s = i.findCell(n.pos - a), c = t.node(0);
		return o.top <= s.top ? (o.top > 0 && (t = c.resolve(a + i.map[o.left])), s.bottom < i.height && (n = c.resolve(a + i.map[i.width * (i.height - 1) + s.right - 1]))) : (s.top > 0 && (n = c.resolve(a + i.map[s.left])), o.bottom < i.height && (t = c.resolve(a + i.map[i.width * (i.height - 1) + o.right - 1]))), new e(t, n);
	}
	isRowSelection() {
		let e = this.$anchorCell.node(-1), t = Z.get(e), n = this.$anchorCell.start(-1), r = t.colCount(this.$anchorCell.pos - n), i = t.colCount(this.$headCell.pos - n);
		if (Math.min(r, i) > 0) return !1;
		let a = r + this.$anchorCell.nodeAfter.attrs.colspan, o = i + this.$headCell.nodeAfter.attrs.colspan;
		return Math.max(a, o) == t.width;
	}
	eq(t) {
		return t instanceof e && t.$anchorCell.pos == this.$anchorCell.pos && t.$headCell.pos == this.$headCell.pos;
	}
	static rowSelection(t, n = t) {
		let r = t.node(-1), i = Z.get(r), a = t.start(-1), o = i.findCell(t.pos - a), s = i.findCell(n.pos - a), c = t.node(0);
		return o.left <= s.left ? (o.left > 0 && (t = c.resolve(a + i.map[o.top * i.width])), s.right < i.width && (n = c.resolve(a + i.map[i.width * (s.top + 1) - 1]))) : (s.left > 0 && (n = c.resolve(a + i.map[s.top * i.width])), o.right < i.width && (t = c.resolve(a + i.map[i.width * (o.top + 1) - 1]))), new e(t, n);
	}
	toJSON() {
		return {
			type: "cell",
			anchor: this.$anchorCell.pos,
			head: this.$headCell.pos
		};
	}
	static fromJSON(t, n) {
		return new e(t.resolve(n.anchor), t.resolve(n.head));
	}
	static create(t, n, r = n) {
		return new e(t.resolve(n), t.resolve(r));
	}
	getBookmark() {
		return new ky(this.$anchorCell.pos, this.$headCell.pos);
	}
};
Q.prototype.visible = !1, L.jsonID("cell", Q);
var ky = class e {
	constructor(e, t) {
		this.anchor = e, this.head = t;
	}
	map(t) {
		return new e(t.map(this.anchor), t.map(this.head));
	}
	resolve(e) {
		let t = e.resolve(this.anchor), n = e.resolve(this.head);
		return t.parent.type.spec.tableRole == "row" && n.parent.type.spec.tableRole == "row" && t.index() < t.parent.childCount && n.index() < n.parent.childCount && wy(t, n) ? new Q(t, n) : L.near(n, 1);
	}
};
function Ay(e) {
	if (!(e.selection instanceof Q)) return null;
	let t = [];
	return e.selection.forEachCell((e, n) => {
		t.push(Dm.node(n, n + e.nodeSize, { class: "selectedCell" }));
	}), K.create(e.doc, t);
}
function jy({ $from: e, $to: t }) {
	if (e.pos == t.pos || e.pos < t.pos - 6) return !1;
	let n = e.pos, r = t.pos, i = e.depth;
	for (; i >= 0 && !(e.after(i + 1) < e.end(i)); i--, n++);
	for (let e = t.depth; e >= 0 && !(t.before(e + 1) > t.start(e)); e--, r--);
	return n == r && /row|table/.test(e.node(i).type.spec.tableRole);
}
function My({ $from: e, $to: t }) {
	let n, r;
	for (let t = e.depth; t > 0; t--) {
		let r = e.node(t);
		if (r.type.spec.tableRole === "cell" || r.type.spec.tableRole === "header_cell") {
			n = r;
			break;
		}
	}
	for (let e = t.depth; e > 0; e--) {
		let n = t.node(e);
		if (n.type.spec.tableRole === "cell" || n.type.spec.tableRole === "header_cell") {
			r = n;
			break;
		}
	}
	return n !== r && t.parentOffset === 0;
}
function Ny(e, t, n) {
	let r = (t || e).selection, i = (t || e).doc, a, o;
	if (r instanceof z && (o = r.node.type.spec.tableRole)) {
		if (o == "cell" || o == "header_cell") a = Q.create(i, r.from);
		else if (o == "row") {
			let e = i.resolve(r.from + 1);
			a = Q.rowSelection(e, e);
		} else if (!n) {
			let e = Z.get(r.node), t = r.from + 1, n = t + e.map[e.width * e.height - 1];
			a = Q.create(i, t + 1, n);
		}
	} else r instanceof R && jy(r) ? a = R.create(i, r.from) : r instanceof R && My(r) && (a = R.create(i, r.$from.start(), r.$from.end()));
	return a && (t ||= e.tr).setSelection(a), t;
}
var Py = new vl("fix-tables");
function Fy(e, t, n, r) {
	let i = e.childCount, a = t.childCount;
	outer: for (let o = 0, s = 0; o < a; o++) {
		let a = t.child(o);
		for (let t = s, r = Math.min(i, o + 3); t < r; t++) if (e.child(t) == a) {
			s = t + 1, n += a.nodeSize;
			continue outer;
		}
		r(a, n), s < i && e.child(s).sameMarkup(a) ? Fy(e.child(s), a, n + 1, r) : a.nodesBetween(0, a.content.size, r, n + 1), n += a.nodeSize;
	}
}
function Iy(e, t) {
	let n, r = (t, r) => {
		t.type.spec.tableRole == "table" && (n = Ly(e, t, r, n));
	};
	return t ? t.doc != e.doc && Fy(t.doc, e.doc, 0, r) : e.doc.descendants(r), n;
}
function Ly(e, t, n, r) {
	let i = Z.get(t);
	if (!i.problems) return r;
	r ||= e.tr;
	let a = [];
	for (let e = 0; e < i.height; e++) a.push(0);
	for (let e = 0; e < i.problems.length; e++) {
		let o = i.problems[e];
		if (o.type == "collision") {
			let e = t.nodeAt(o.pos);
			if (!e) continue;
			let i = e.attrs;
			for (let e = 0; e < i.rowspan; e++) a[o.row + e] += o.n;
			r.setNodeMarkup(r.mapping.map(n + 1 + o.pos), null, Ey(i, i.colspan - o.n, o.n));
		} else if (o.type == "missing") a[o.row] += o.n;
		else if (o.type == "overlong_rowspan") {
			let e = t.nodeAt(o.pos);
			if (!e) continue;
			r.setNodeMarkup(r.mapping.map(n + 1 + o.pos), null, {
				...e.attrs,
				rowspan: e.attrs.rowspan - o.n
			});
		} else if (o.type == "colwidth mismatch") {
			let e = t.nodeAt(o.pos);
			if (!e) continue;
			r.setNodeMarkup(r.mapping.map(n + 1 + o.pos), null, {
				...e.attrs,
				colwidth: o.colwidth
			});
		} else if (o.type == "zero_sized") {
			let e = r.mapping.map(n);
			r.delete(e, e + t.nodeSize);
		}
	}
	let o, s;
	for (let e = 0; e < a.length; e++) a[e] && (o ??= e, s = e);
	for (let c = 0, l = n + 1; c < i.height; c++) {
		let n = t.child(c), i = l + n.nodeSize, u = a[c];
		if (u > 0) {
			let t = "cell";
			n.firstChild && (t = n.firstChild.type.spec.tableRole);
			let a = [];
			for (let n = 0; n < u; n++) {
				let n = gy(e.schema)[t].createAndFill();
				n && a.push(n);
			}
			let d = (c == 0 || o == c - 1) && s == c ? l + 1 : i - 1;
			r.insert(r.mapping.map(d), a);
		}
		l = i;
	}
	return r.setMeta(Py, { fixTables: !0 });
}
function Ry(e) {
	let t = Z.get(e), n = [], r = t.height, i = t.width;
	for (let a = 0; a < r; a++) {
		let r = [];
		for (let n = 0; n < i; n++) {
			let o = a * i + n, s = t.map[o];
			if (a > 0) {
				let e = o - i;
				if (s === t.map[e]) {
					r.push(null);
					continue;
				}
			}
			if (n > 0) {
				let e = o - 1;
				if (s === t.map[e]) {
					r.push(null);
					continue;
				}
			}
			r.push(e.nodeAt(s));
		}
		n.push(r);
	}
	return n;
}
function zy(e, t) {
	let n = [], r = Z.get(e), i = r.height, a = r.width;
	for (let o = 0; o < i; o++) {
		let i = e.child(o), s = [];
		for (let n = 0; n < a; n++) {
			let i = t[o][n];
			if (!i) continue;
			let a = r.map[o * r.width + n], c = e.nodeAt(a);
			if (!c) continue;
			let l = c.type.createChecked(i.attrs, i.content, i.marks);
			s.push(l);
		}
		let c = i.type.createChecked(i.attrs, s, i.marks);
		n.push(c);
	}
	return e.type.createChecked(e.attrs, n, e.marks);
}
function By(e, t, n, r) {
	let i = t[0] > n[0] ? -1 : 1, a = e.splice(t[0], t.length), o = +(a.length % 2 == 0), s;
	return s = r === -1 && i === 1 ? n[0] - 1 : r === 1 && i === -1 ? n[n.length - 1] - o + 1 : i === -1 ? n[0] : n[n.length - 1] - o, e.splice(s, 0, ...a), e;
}
function Vy(e) {
	return Hy((e) => e.type.spec.tableRole === "table", e);
}
function Hy(e, t) {
	for (let n = t.depth; n >= 0; --n) {
		let r = t.node(n);
		if (e(r)) return {
			node: r,
			pos: n === 0 ? 0 : t.before(n),
			start: t.start(n),
			depth: n
		};
	}
	return null;
}
function Uy(e, t) {
	let n = Vy(t.$from);
	if (!n) return;
	let r = Z.get(n.node);
	if (!(e < 0 || e > r.width - 1)) return r.cellsInRect({
		left: e,
		right: e + 1,
		top: 0,
		bottom: r.height
	}).map((e) => {
		let t = n.node.nodeAt(e), r = e + n.start;
		return {
			pos: r,
			start: r + 1,
			node: t,
			depth: n.depth + 2
		};
	});
}
function Wy(e, t) {
	let n = Vy(t.$from);
	if (!n) return;
	let r = Z.get(n.node);
	if (!(e < 0 || e > r.height - 1)) return r.cellsInRect({
		left: 0,
		right: r.width,
		top: e,
		bottom: e + 1
	}).map((e) => {
		let t = n.node.nodeAt(e), r = e + n.start;
		return {
			pos: r,
			start: r + 1,
			node: t,
			depth: n.depth + 2
		};
	});
}
function Gy(e, t, n = t) {
	let r = t, i = n;
	for (let n = t; n >= 0; n--) {
		let t = Uy(n, e.selection);
		t && t.forEach((e) => {
			let t = e.node.attrs.colspan + n - 1;
			t >= r && (r = n), t > i && (i = t);
		});
	}
	for (let n = t; n <= i; n++) {
		let t = Uy(n, e.selection);
		t && t.forEach((e) => {
			let t = e.node.attrs.colspan + n - 1;
			e.node.attrs.colspan > 1 && t > i && (i = t);
		});
	}
	let a = [];
	for (let t = r; t <= i; t++) {
		let n = Uy(t, e.selection);
		n && n.length > 0 && a.push(t);
	}
	r = a[0], i = a[a.length - 1];
	let o = Uy(r, e.selection), s = Wy(0, e.selection);
	if (!o || !s) return;
	let c = e.doc.resolve(o[o.length - 1].pos), l;
	for (let t = i; t >= r; t--) {
		let n = Uy(t, e.selection);
		if (n && n.length > 0) {
			for (let e = s.length - 1; e >= 0; e--) if (s[e].pos === n[0].pos) {
				l = n[0];
				break;
			}
			if (l) break;
		}
	}
	if (l) return {
		$anchor: c,
		$head: e.doc.resolve(l.pos),
		indexes: a
	};
}
function Ky(e, t, n = t) {
	let r = t, i = n;
	for (let n = t; n >= 0; n--) {
		let t = Wy(n, e.selection);
		t && t.forEach((e) => {
			let t = e.node.attrs.rowspan + n - 1;
			t >= r && (r = n), t > i && (i = t);
		});
	}
	for (let n = t; n <= i; n++) {
		let t = Wy(n, e.selection);
		t && t.forEach((e) => {
			let t = e.node.attrs.rowspan + n - 1;
			e.node.attrs.rowspan > 1 && t > i && (i = t);
		});
	}
	let a = [];
	for (let t = r; t <= i; t++) {
		let n = Wy(t, e.selection);
		n && n.length > 0 && a.push(t);
	}
	r = a[0], i = a[a.length - 1];
	let o = Wy(r, e.selection), s = Uy(0, e.selection);
	if (!o || !s) return;
	let c = e.doc.resolve(o[o.length - 1].pos), l;
	for (let t = i; t >= r; t--) {
		let n = Wy(t, e.selection);
		if (n && n.length > 0) {
			for (let e = s.length - 1; e >= 0; e--) if (s[e].pos === n[0].pos) {
				l = n[0];
				break;
			}
			if (l) break;
		}
	}
	if (l) return {
		$anchor: c,
		$head: e.doc.resolve(l.pos),
		indexes: a
	};
}
function qy(e) {
	return e[0].map((t, n) => e.map((e) => e[n]));
}
function Jy(e) {
	let { tr: t, originIndex: n, targetIndex: r, select: i, pos: a } = e, o = Vy(t.doc.resolve(a));
	if (!o) return !1;
	let s = Gy(t, n)?.indexes, c = Gy(t, r)?.indexes;
	if (!s || !c || s.includes(r)) return !1;
	let l = Yy(o.node, s, c, 0);
	if (t.replaceWith(o.pos, o.pos + o.node.nodeSize, l), !i) return !0;
	let u = Z.get(l), d = o.start, f = r, p = u.positionAt(u.height - 1, f, l), m = t.doc.resolve(d + p), h = u.positionAt(0, f, l), g = t.doc.resolve(d + h);
	return t.setSelection(Q.colSelection(m, g)), !0;
}
function Yy(e, t, n, r) {
	let i = qy(Ry(e));
	return i = By(i, t, n, r), i = qy(i), zy(e, i);
}
function Xy(e) {
	let { tr: t, originIndex: n, targetIndex: r, select: i, pos: a } = e, o = Vy(t.doc.resolve(a));
	if (!o) return !1;
	let s = Ky(t, n)?.indexes, c = Ky(t, r)?.indexes;
	if (!s || !c || s.includes(r)) return !1;
	let l = Zy(o.node, s, c, 0);
	if (t.replaceWith(o.pos, o.pos + o.node.nodeSize, l), !i) return !0;
	let u = Z.get(l), d = o.start, f = r, p = u.positionAt(f, u.width - 1, l), m = t.doc.resolve(d + p), h = u.positionAt(f, 0, l), g = t.doc.resolve(d + h);
	return t.setSelection(Q.rowSelection(m, g)), !0;
}
function Zy(e, t, n, r) {
	let i = Ry(e);
	return i = By(i, t, n, r), zy(e, i);
}
function Qy(e) {
	let t = e.selection, n = by(e), r = n.node(-1), i = n.start(-1), a = Z.get(r);
	return {
		...t instanceof Q ? a.rectBetween(t.$anchorCell.pos - i, t.$headCell.pos - i) : a.findCell(n.pos - i),
		tableStart: i,
		map: a,
		table: r
	};
}
function $y(e, { map: t, tableStart: n, table: r }, i) {
	let a = i > 0 ? -1 : 0;
	Oy(t, r, i + a) && (a = i == 0 || i == t.width ? null : 0);
	for (let o = 0; o < t.height; o++) {
		let s = o * t.width + i;
		if (i > 0 && i < t.width && t.map[s - 1] == t.map[s]) {
			let a = t.map[s], c = r.nodeAt(a);
			e.setNodeMarkup(e.mapping.map(n + a), null, Dy(c.attrs, i - t.colCount(a))), o += c.attrs.rowspan - 1;
		} else {
			let c = a == null ? gy(r.type.schema).cell : r.nodeAt(t.map[s + a]).type, l = t.positionAt(o, i, r);
			e.insert(e.mapping.map(n + l), c.createAndFill());
		}
	}
	return e;
}
function eb(e, t) {
	if (!yy(e)) return !1;
	if (t) {
		let n = Qy(e);
		t($y(e.tr, n, n.left));
	}
	return !0;
}
function tb(e, t) {
	if (!yy(e)) return !1;
	if (t) {
		let n = Qy(e);
		t($y(e.tr, n, n.right));
	}
	return !0;
}
function nb(e, { map: t, table: n, tableStart: r }, i) {
	let a = e.mapping.maps.length;
	for (let o = 0; o < t.height;) {
		let s = o * t.width + i, c = t.map[s], l = n.nodeAt(c), u = l.attrs;
		if (i > 0 && t.map[s - 1] == c || i < t.width - 1 && t.map[s + 1] == c) e.setNodeMarkup(e.mapping.slice(a).map(r + c), null, Ey(u, i - t.colCount(c)));
		else {
			let t = e.mapping.slice(a).map(r + c);
			e.delete(t, t + l.nodeSize);
		}
		o += u.rowspan;
	}
}
function rb(e, t) {
	if (!yy(e)) return !1;
	if (t) {
		let n = Qy(e), r = e.tr;
		if (n.left == 0 && n.right == n.map.width) return !1;
		for (let e = n.right - 1; nb(r, n, e), e != n.left; e--) {
			let e = n.tableStart ? r.doc.nodeAt(n.tableStart - 1) : r.doc;
			if (!e) throw RangeError("No table found");
			n.table = e, n.map = Z.get(e);
		}
		t(r);
	}
	return !0;
}
function ib(e, { map: t, table: n, tableStart: r }, i) {
	let a = 0;
	for (let e = 0; e < i; e++) a += n.child(e).nodeSize;
	let o = a + n.child(i).nodeSize, s = e.mapping.maps.length;
	e.delete(a + r, o + r);
	let c = /* @__PURE__ */ new Set();
	for (let a = 0, o = i * t.width; a < t.width; a++, o++) {
		let l = t.map[o];
		if (!c.has(l)) {
			if (c.add(l), i > 0 && l == t.map[o - t.width]) {
				let t = n.nodeAt(l).attrs;
				e.setNodeMarkup(e.mapping.slice(s).map(l + r), null, {
					...t,
					rowspan: t.rowspan - 1
				}), a += t.colspan - 1;
			} else if (i < t.height && l == t.map[o + t.width]) {
				let o = n.nodeAt(l), c = o.attrs, u = o.type.create({
					...c,
					rowspan: o.attrs.rowspan - 1
				}, o.content), d = t.positionAt(i + 1, a, n);
				e.insert(e.mapping.slice(s).map(r + d), u), a += c.colspan - 1;
			}
		}
	}
}
function ab(e, t) {
	if (!yy(e)) return !1;
	if (t) {
		let n = Qy(e), r = e.tr;
		if (n.top == 0 && n.bottom == n.map.height) return !1;
		for (let e = n.bottom - 1; ib(r, n, e), e != n.top; e--) {
			let e = n.tableStart ? r.doc.nodeAt(n.tableStart - 1) : r.doc;
			if (!e) throw RangeError("No table found");
			n.table = e, n.map = Z.get(n.table);
		}
		t(r);
	}
	return !0;
}
function ob(e, t) {
	return function(n, r) {
		if (!yy(n)) return !1;
		let i = by(n);
		if (i.nodeAfter.attrs[e] === t) return !1;
		if (r) {
			let a = n.tr;
			n.selection instanceof Q ? n.selection.forEachCell((n, r) => {
				n.attrs[e] !== t && a.setNodeMarkup(r, null, {
					...n.attrs,
					[e]: t
				});
			}) : a.setNodeMarkup(i.pos, null, {
				...i.nodeAfter.attrs,
				[e]: t
			}), r(a);
		}
		return !0;
	};
}
function sb(e) {
	return function(t, n) {
		if (!yy(t)) return !1;
		if (n) {
			let r = gy(t.schema), i = Qy(t), a = t.tr, o = i.map.cellsInRect(e == "column" ? {
				left: i.left,
				top: 0,
				right: i.right,
				bottom: i.map.height
			} : e == "row" ? {
				left: 0,
				top: i.top,
				right: i.map.width,
				bottom: i.bottom
			} : i), s = o.map((e) => i.table.nodeAt(e));
			for (let e = 0; e < o.length; e++) s[e].type == r.header_cell && a.setNodeMarkup(i.tableStart + o[e], r.cell, s[e].attrs);
			if (a.steps.length === 0) for (let e = 0; e < o.length; e++) a.setNodeMarkup(i.tableStart + o[e], r.header_cell, s[e].attrs);
			n(a);
		}
		return !0;
	};
}
function cb(e, t, n) {
	let r = t.map.cellsInRect({
		left: 0,
		top: 0,
		right: e == "row" ? t.map.width : 1,
		bottom: e == "column" ? t.map.height : 1
	});
	for (let e = 0; e < r.length; e++) {
		let i = t.table.nodeAt(r[e]);
		if (i && i.type !== n.header_cell) return !1;
	}
	return !0;
}
function lb(e, t) {
	return t ||= { useDeprecatedLogic: !1 }, t.useDeprecatedLogic ? sb(e) : function(t, n) {
		if (!yy(t)) return !1;
		if (n) {
			let r = gy(t.schema), i = Qy(t), a = t.tr, o = cb("row", i, r), s = cb("column", i, r), c = (e === "column" ? o : e === "row" && s) ? 1 : 0, l = e == "column" ? {
				left: 0,
				top: c,
				right: 1,
				bottom: i.map.height
			} : e == "row" ? {
				left: c,
				top: 0,
				right: i.map.width,
				bottom: 1
			} : i, u = e == "column" ? s ? r.cell : r.header_cell : e == "row" ? o ? r.cell : r.header_cell : r.cell;
			i.map.cellsInRect(l).forEach((e) => {
				let t = e + i.tableStart, n = a.doc.nodeAt(t);
				n && a.setNodeMarkup(t, u, n.attrs);
			}), n(a);
		}
		return !0;
	};
}
lb("row", { useDeprecatedLogic: !0 }), lb("column", { useDeprecatedLogic: !0 }), lb("cell", { useDeprecatedLogic: !0 });
function ub(e, t) {
	if (t < 0) {
		let t = e.nodeBefore;
		if (t) return e.pos - t.nodeSize;
		for (let t = e.index(-1) - 1, n = e.before(); t >= 0; t--) {
			let r = e.node(-1).child(t), i = r.lastChild;
			if (i) return n - 1 - i.nodeSize;
			n -= r.nodeSize;
		}
	} else {
		if (e.index() < e.parent.childCount - 1) return e.pos + e.nodeAfter.nodeSize;
		let t = e.node(-1);
		for (let n = e.indexAfter(-1), r = e.after(); n < t.childCount; n++) {
			let e = t.child(n);
			if (e.childCount) return r + 1;
			r += e.nodeSize;
		}
	}
	return null;
}
function db(e) {
	return function(t, n) {
		if (!yy(t)) return !1;
		let r = ub(by(t), e);
		if (r == null) return !1;
		if (n) {
			let e = t.doc.resolve(r);
			n(t.tr.setSelection(R.between(e, Cy(e))).scrollIntoView());
		}
		return !0;
	};
}
function fb(e, t) {
	let n = e.selection.$anchor;
	for (let r = n.depth; r > 0; r--) if (n.node(r).type.spec.tableRole == "table") return t && t(e.tr.delete(n.before(r), n.after(r)).scrollIntoView()), !0;
	return !1;
}
function pb(e, t) {
	let n = e.selection;
	if (!(n instanceof Q)) return !1;
	if (t) {
		let r = e.tr, i = gy(e.schema).cell.createAndFill().content;
		n.forEachCell((e, t) => {
			e.content.eq(i) || r.replace(r.mapping.map(t + 1), r.mapping.map(t + e.nodeSize - 1), new F(i, 0, 0));
		}), r.docChanged && t(r);
	}
	return !0;
}
function mb(e) {
	return (t, n) => {
		let { from: r, to: i, select: a = !0, pos: o = t.selection.from } = e, s = t.tr;
		return Xy({
			tr: s,
			originIndex: r,
			targetIndex: i,
			select: a,
			pos: o
		}) ? (n?.(s), !0) : !1;
	};
}
function hb(e) {
	return (t, n) => {
		let { from: r, to: i, select: a = !0, pos: o = t.selection.from } = e, s = t.tr;
		return Jy({
			tr: s,
			originIndex: r,
			targetIndex: i,
			select: a,
			pos: o
		}) ? (n?.(s), !0) : !1;
	};
}
function gb(e) {
	if (e.size === 0) return null;
	let { content: t, openStart: n, openEnd: r } = e;
	for (; t.childCount == 1 && (n > 0 && r > 0 || t.child(0).type.spec.tableRole == "table");) n--, r--, t = t.child(0).content;
	let i = t.child(0), a = i.type.spec.tableRole, o = i.type.schema, s = [];
	if (a == "row") for (let e = 0; e < t.childCount; e++) {
		let i = t.child(e).content, a = e ? 0 : Math.max(0, n - 1), c = e < t.childCount - 1 ? 0 : Math.max(0, r - 1);
		(a || c) && (i = vb(gy(o).row, new F(i, a, c)).content), s.push(i);
	}
	else if (a == "cell" || a == "header_cell") s.push(n || r ? vb(gy(o).row, new F(t, n, r)).content : t);
	else return null;
	return _b(o, s);
}
function _b(e, t) {
	let n = [];
	for (let e = 0; e < t.length; e++) {
		let r = t[e];
		for (let t = r.childCount - 1; t >= 0; t--) {
			let { rowspan: i, colspan: a } = r.child(t).attrs;
			for (let t = e; t < e + i; t++) n[t] = (n[t] || 0) + a;
		}
	}
	let r = 0;
	for (let e = 0; e < n.length; e++) r = Math.max(r, n[e]);
	for (let i = 0; i < n.length; i++) if (i >= t.length && t.push(N.empty), n[i] < r) {
		let a = gy(e).cell.createAndFill(), o = [];
		for (let e = n[i]; e < r; e++) o.push(a);
		t[i] = t[i].append(N.from(o));
	}
	return {
		height: t.length,
		width: r,
		rows: t
	};
}
function vb(e, t) {
	let n = e.createAndFill();
	return new Jc(n).replace(0, n.content.size, t).doc;
}
function yb({ width: e, height: t, rows: n }, r, i) {
	if (e != r) {
		let t = [], i = [];
		for (let e = 0; e < n.length; e++) {
			let a = n[e], o = [];
			for (let n = t[e] || 0, i = 0; n < r; i++) {
				let s = a.child(i % a.childCount);
				n + s.attrs.colspan > r && (s = s.type.createChecked(Ey(s.attrs, s.attrs.colspan, n + s.attrs.colspan - r), s.content)), o.push(s), n += s.attrs.colspan;
				for (let n = 1; n < s.attrs.rowspan; n++) t[e + n] = (t[e + n] || 0) + s.attrs.colspan;
			}
			i.push(N.from(o));
		}
		n = i, e = r;
	}
	if (t != i) {
		let e = [];
		for (let r = 0, a = 0; r < i; r++, a++) {
			let o = [], s = n[a % t];
			for (let e = 0; e < s.childCount; e++) {
				let t = s.child(e);
				r + t.attrs.rowspan > i && (t = t.type.create({
					...t.attrs,
					rowspan: Math.max(1, i - t.attrs.rowspan)
				}, t.content)), o.push(t);
			}
			e.push(N.from(o));
		}
		n = e, t = i;
	}
	return {
		width: e,
		height: t,
		rows: n
	};
}
function bb(e, t, n, r, i, a, o) {
	let s = e.doc.type.schema, c = gy(s), l, u;
	if (i > t.width) for (let a = 0, s = 0; a < t.height; a++) {
		let d = n.child(a);
		s += d.nodeSize;
		let f = [], p;
		p = d.lastChild == null || d.lastChild.type == c.cell ? l ||= c.cell.createAndFill() : u ||= c.header_cell.createAndFill();
		for (let e = t.width; e < i; e++) f.push(p);
		e.insert(e.mapping.slice(o).map(s - 1 + r), f);
	}
	if (a > t.height) {
		let s = [];
		for (let e = 0, r = (t.height - 1) * t.width; e < Math.max(t.width, i); e++) {
			let i = e >= t.width ? !1 : n.nodeAt(t.map[r + e]).type == c.header_cell;
			s.push(i ? u ||= c.header_cell.createAndFill() : l ||= c.cell.createAndFill());
		}
		let d = c.row.create(null, N.from(s)), f = [];
		for (let e = t.height; e < a; e++) f.push(d);
		e.insert(e.mapping.slice(o).map(r + n.nodeSize - 2), f);
	}
	return !!(l || u);
}
function xb(e, t, n, r, i, a, o, s) {
	if (o == 0 || o == t.height) return !1;
	let c = !1;
	for (let l = i; l < a; l++) {
		let i = o * t.width + l, a = t.map[i];
		if (t.map[i - t.width] == a) {
			c = !0;
			let i = n.nodeAt(a), { top: u, left: d } = t.findCell(a);
			e.setNodeMarkup(e.mapping.slice(s).map(a + r), null, {
				...i.attrs,
				rowspan: o - u
			}), e.insert(e.mapping.slice(s).map(t.positionAt(o, d, n)), i.type.createAndFill({
				...i.attrs,
				rowspan: u + i.attrs.rowspan - o
			})), l += i.attrs.colspan - 1;
		}
	}
	return c;
}
function Sb(e, t, n, r, i, a, o, s) {
	if (o == 0 || o == t.width) return !1;
	let c = !1;
	for (let l = i; l < a; l++) {
		let i = l * t.width + o, a = t.map[i];
		if (t.map[i - 1] == a) {
			c = !0;
			let i = n.nodeAt(a), u = t.colCount(a), d = e.mapping.slice(s).map(a + r);
			e.setNodeMarkup(d, null, Ey(i.attrs, o - u, i.attrs.colspan - (o - u))), e.insert(d + i.nodeSize, i.type.createAndFill(Ey(i.attrs, 0, o - u))), l += i.attrs.rowspan - 1;
		}
	}
	return c;
}
function Cb(e, t, n, r, i) {
	let a = n ? e.doc.nodeAt(n - 1) : e.doc;
	if (!a) throw Error("No table found");
	let o = Z.get(a), { top: s, left: c } = r, l = c + i.width, u = s + i.height, d = e.tr, f = 0;
	function p() {
		if (a = n ? d.doc.nodeAt(n - 1) : d.doc, !a) throw Error("No table found");
		o = Z.get(a), f = d.mapping.maps.length;
	}
	bb(d, o, a, n, l, u, f) && p(), xb(d, o, a, n, c, l, s, f) && p(), xb(d, o, a, n, c, l, u, f) && p(), Sb(d, o, a, n, s, u, c, f) && p(), Sb(d, o, a, n, s, u, l, f) && p();
	for (let e = s; e < u; e++) {
		let t = o.positionAt(e, c, a), r = o.positionAt(e, l, a);
		d.replace(d.mapping.slice(f).map(t + n), d.mapping.slice(f).map(r + n), new F(i.rows[e - s], 0, 0));
	}
	p(), d.setSelection(new Q(d.doc.resolve(n + o.positionAt(s, c, a)), d.doc.resolve(n + o.positionAt(u - 1, l - 1, a)))), t(d);
}
var wb = Uu({
	ArrowLeft: Eb("horiz", -1),
	ArrowRight: Eb("horiz", 1),
	ArrowUp: Eb("vert", -1),
	ArrowDown: Eb("vert", 1),
	"Shift-ArrowLeft": Db("horiz", -1),
	"Shift-ArrowRight": Db("horiz", 1),
	"Shift-ArrowUp": Db("vert", -1),
	"Shift-ArrowDown": Db("vert", 1),
	Backspace: pb,
	"Mod-Backspace": pb,
	Delete: pb,
	"Mod-Delete": pb
});
function Tb(e, t, n) {
	return !n.eq(e.selection) && (t && t(e.tr.setSelection(n).scrollIntoView()), !0);
}
function Eb(e, t) {
	return (n, r, i) => {
		if (!i) return !1;
		let a = n.selection;
		if (a instanceof Q) return Tb(n, r, L.near(a.$headCell, t));
		if (e != "horiz" && !a.empty) return !1;
		let o = jb(i, e, t);
		if (o == null) return !1;
		if (e == "horiz") return Tb(n, r, L.near(n.doc.resolve(a.head + t), t));
		{
			let i = n.doc.resolve(o), a = Ty(i, e, t), s;
			return s = a ? L.near(a, 1) : t < 0 ? L.near(n.doc.resolve(i.before(-1)), -1) : L.near(n.doc.resolve(i.after(-1)), 1), Tb(n, r, s);
		}
	};
}
function Db(e, t) {
	return (n, r, i) => {
		if (!i) return !1;
		let a = n.selection, o;
		if (a instanceof Q) o = a;
		else {
			let r = jb(i, e, t);
			if (r == null) return !1;
			o = new Q(n.doc.resolve(r));
		}
		let s = Ty(o.$headCell, e, t);
		return s ? Tb(n, r, new Q(o.$anchorCell, s)) : !1;
	};
}
function Ob(e, t) {
	let n = e.state.doc, r = vy(n.resolve(t));
	return r ? (e.dispatch(e.state.tr.setSelection(new Q(r))), !0) : !1;
}
function kb(e, t, n) {
	if (!yy(e.state)) return !1;
	let r = gb(n), i = e.state.selection;
	if (i instanceof Q) {
		r ||= {
			width: 1,
			height: 1,
			rows: [N.from(vb(gy(e.state.schema).cell, n))]
		};
		let t = i.$anchorCell.node(-1), a = i.$anchorCell.start(-1), o = Z.get(t).rectBetween(i.$anchorCell.pos - a, i.$headCell.pos - a);
		return r = yb(r, o.right - o.left, o.bottom - o.top), Cb(e.state, e.dispatch, a, o, r), !0;
	}
	if (r) {
		let t = by(e.state), n = t.start(-1);
		return Cb(e.state, e.dispatch, n, Z.get(t.node(-1)).findCell(t.pos - n), r), !0;
	}
	return !1;
}
function Ab(e, t) {
	if (t.button != 0 || t.ctrlKey || t.metaKey) return;
	let n = Mb(e, t.target), r;
	if (t.shiftKey && e.state.selection instanceof Q) i(e.state.selection.$anchorCell, t), t.preventDefault();
	else if (t.shiftKey && n && (r = vy(e.state.selection.$anchor)) != null && Nb(e, t)?.pos != r.pos) i(r, t), t.preventDefault();
	else if (!n) return;
	function i(t, n) {
		let r = Nb(e, n), i = _y.getState(e.state) == null;
		if (!r || !wy(t, r)) {
			if (i) r = t;
			else return;
		}
		let a = new Q(t, r);
		if (i || !e.state.selection.eq(a)) {
			let n = e.state.tr.setSelection(a);
			i && n.setMeta(_y, t.pos), e.dispatch(n);
		}
	}
	function a() {
		e.root.removeEventListener("mouseup", a), e.root.removeEventListener("dragstart", a), e.root.removeEventListener("mousemove", o), _y.getState(e.state) != null && e.dispatch(e.state.tr.setMeta(_y, -1));
	}
	function o(r) {
		let o = r, s = _y.getState(e.state), c;
		if (s != null) c = e.state.doc.resolve(s);
		else if (Mb(e, o.target) != n && (c = Nb(e, t), !c)) return a();
		c && i(c, o);
	}
	e.root.addEventListener("mouseup", a), e.root.addEventListener("dragstart", a), e.root.addEventListener("mousemove", o);
}
function jb(e, t, n) {
	if (!(e.state.selection instanceof R)) return null;
	let { $head: r } = e.state.selection;
	for (let i = r.depth - 1; i >= 0; i--) {
		let a = r.node(i);
		if ((n < 0 ? r.index(i) : r.indexAfter(i)) != (n < 0 ? 0 : a.childCount)) return null;
		if (a.type.spec.tableRole == "cell" || a.type.spec.tableRole == "header_cell") {
			let a = r.before(i), o = t == "vert" ? n > 0 ? "down" : "up" : n > 0 ? "right" : "left";
			return e.endOfTextblock(o) ? a : null;
		}
	}
	return null;
}
function Mb(e, t) {
	for (; t && t != e.dom; t = t.parentNode) if (t.nodeName == "TD" || t.nodeName == "TH") return t;
	return null;
}
function Nb(e, t) {
	let n = e.posAtCoords({
		left: t.clientX,
		top: t.clientY
	});
	if (!n) return null;
	let { inside: r, pos: i } = n;
	return r >= 0 && vy(e.state.doc.resolve(r)) || vy(e.state.doc.resolve(i));
}
var Pb = class {
	constructor(e, t) {
		this.node = e, this.defaultCellMinWidth = t, this.dom = document.createElement("div"), this.dom.className = "tableWrapper", this.table = this.dom.appendChild(document.createElement("table")), this.table.style.setProperty("--default-cell-min-width", `${t}px`), this.colgroup = this.table.appendChild(document.createElement("colgroup")), Fb(e, this.colgroup, this.table, t), this.contentDOM = this.table.appendChild(document.createElement("tbody"));
	}
	update(e) {
		return e.type == this.node.type && (this.node = e, Fb(e, this.colgroup, this.table, this.defaultCellMinWidth), !0);
	}
	ignoreMutation(e) {
		return e.type == "attributes" && (e.target == this.table || this.colgroup.contains(e.target));
	}
};
function Fb(e, t, n, r, i, a) {
	let o = 0, s = !0, c = t.firstChild, l = e.firstChild;
	if (l) {
		for (let e = 0, n = 0; e < l.childCount; e++) {
			let { colspan: u, colwidth: d } = l.child(e).attrs;
			for (let e = 0; e < u; e++, n++) {
				let l = i == n ? a : d && d[e], u = l ? l + "px" : "";
				if (o += l || r, l || (s = !1), c) c.style.width != u && (c.style.width = u), c = c.nextSibling;
				else {
					let e = document.createElement("col");
					e.style.width = u, t.appendChild(e);
				}
			}
		}
		for (; c;) {
			var u;
			let e = c.nextSibling;
			(u = c.parentNode) == null || u.removeChild(c), c = e;
		}
		s ? (n.style.width = o + "px", n.style.minWidth = "") : (n.style.width = "", n.style.minWidth = o + "px");
	}
}
var Ib = new vl("tableColumnResizing");
function Lb({ handleWidth: e = 5, cellMinWidth: t = 25, defaultCellMinWidth: n = 100, View: r = Pb, lastColumnResizable: i = !0 } = {}) {
	let a = new hl({
		key: Ib,
		state: {
			init(e, t) {
				var i;
				let o = (i = a.spec) == null || (i = i.props) == null ? void 0 : i.nodeViews, s = gy(t.schema).table.name;
				return r && o && (o[s] = (e, t) => new r(e, n, t)), new Rb(-1, !1);
			},
			apply(e, t) {
				return t.apply(e);
			}
		},
		props: {
			attributes: (e) => {
				let t = Ib.getState(e);
				return t && t.activeHandle > -1 ? { class: "resize-cursor" } : {};
			},
			handleDOMEvents: {
				mousemove: (t, n) => {
					zb(t, n, e, i);
				},
				mouseleave: (e) => {
					Bb(e);
				},
				mousedown: (e, r) => {
					Vb(e, r, t, n);
				}
			},
			decorations: (e) => {
				let t = Ib.getState(e);
				if (t && t.activeHandle > -1) return Xb(e, t.activeHandle);
			},
			nodeViews: {}
		}
	});
	return a;
}
var Rb = class e {
	constructor(e, t) {
		this.activeHandle = e, this.dragging = t;
	}
	apply(t) {
		let n = this, r = t.getMeta(Ib);
		if (r && r.setHandle != null) return new e(r.setHandle, !1);
		if (r && r.setDragging !== void 0) return new e(n.activeHandle, r.setDragging);
		if (n.activeHandle > -1 && t.docChanged) {
			let r = t.mapping.map(n.activeHandle, -1);
			return Sy(t.doc.resolve(r)) || (r = -1), new e(r, n.dragging);
		}
		return n;
	}
};
function zb(e, t, n, r) {
	if (!e.editable) return;
	let i = Ib.getState(e.state);
	if (i && !i.dragging) {
		let a = Ub(t.target), o = -1;
		if (a) {
			let { left: r, right: i } = a.getBoundingClientRect();
			t.clientX - r <= n ? o = Wb(e, t, "left", n) : i - t.clientX <= n && (o = Wb(e, t, "right", n));
		}
		if (o != i.activeHandle) {
			if (!r && o !== -1) {
				let t = e.state.doc.resolve(o), n = t.node(-1), r = Z.get(n), i = t.start(-1);
				if (r.colCount(t.pos - i) + t.nodeAfter.attrs.colspan - 1 == r.width - 1) return;
			}
			Kb(e, o);
		}
	}
}
function Bb(e) {
	if (!e.editable) return;
	let t = Ib.getState(e.state);
	t && t.activeHandle > -1 && !t.dragging && Kb(e, -1);
}
function Vb(e, t, n, r) {
	if (!e.editable) return !1;
	let i = e.dom.ownerDocument.defaultView ?? window, a = Ib.getState(e.state);
	if (!a || a.activeHandle == -1 || a.dragging) return !1;
	let o = e.state.doc.nodeAt(a.activeHandle), s = Hb(e, a.activeHandle, o.attrs);
	e.dispatch(e.state.tr.setMeta(Ib, { setDragging: {
		startX: t.clientX,
		startWidth: s
	} }));
	function c(t) {
		i.removeEventListener("mouseup", c), i.removeEventListener("mousemove", l);
		let r = Ib.getState(e.state);
		r?.dragging && (qb(e, r.activeHandle, Gb(r.dragging, t, n)), e.dispatch(e.state.tr.setMeta(Ib, { setDragging: null })));
	}
	function l(t) {
		if (!t.which) return c(t);
		let i = Ib.getState(e.state);
		if (i && i.dragging) {
			let a = Gb(i.dragging, t, n);
			Jb(e, i.activeHandle, a, r);
		}
	}
	return Jb(e, a.activeHandle, s, r), i.addEventListener("mouseup", c), i.addEventListener("mousemove", l), t.preventDefault(), !0;
}
function Hb(e, t, { colspan: n, colwidth: r }) {
	let i = r && r[r.length - 1];
	if (i) return i;
	let a = e.domAtPos(t), o = a.node.childNodes[a.offset].offsetWidth, s = n;
	if (r) for (let e = 0; e < n; e++) r[e] && (o -= r[e], s--);
	return o / s;
}
function Ub(e) {
	for (; e && e.nodeName != "TD" && e.nodeName != "TH";) e = e.classList && e.classList.contains("ProseMirror") ? null : e.parentNode;
	return e;
}
function Wb(e, t, n, r) {
	let i = n == "right" ? -r : r, a = e.posAtCoords({
		left: t.clientX + i,
		top: t.clientY
	});
	if (!a) return -1;
	let { pos: o } = a, s = vy(e.state.doc.resolve(o));
	if (!s) return -1;
	if (n == "right") return s.pos;
	let c = Z.get(s.node(-1)), l = s.start(-1), u = c.map.indexOf(s.pos - l);
	return u % c.width == 0 ? -1 : l + c.map[u - 1];
}
function Gb(e, t, n) {
	let r = t.clientX - e.startX;
	return Math.max(n, e.startWidth + r);
}
function Kb(e, t) {
	e.dispatch(e.state.tr.setMeta(Ib, { setHandle: t }));
}
function qb(e, t, n) {
	let r = e.state.doc.resolve(t), i = r.node(-1), a = Z.get(i), o = r.start(-1), s = a.colCount(r.pos - o) + r.nodeAfter.attrs.colspan - 1, c = e.state.tr;
	for (let e = 0; e < a.height; e++) {
		let t = e * a.width + s;
		if (e && a.map[t] == a.map[t - a.width]) continue;
		let r = a.map[t], l = i.nodeAt(r).attrs, u = l.colspan == 1 ? 0 : s - a.colCount(r);
		if (l.colwidth && l.colwidth[u] == n) continue;
		let d = l.colwidth ? l.colwidth.slice() : Yb(l.colspan);
		d[u] = n, c.setNodeMarkup(o + r, null, {
			...l,
			colwidth: d
		});
	}
	c.docChanged && e.dispatch(c);
}
function Jb(e, t, n, r) {
	let i = e.state.doc.resolve(t), a = i.node(-1), o = i.start(-1), s = Z.get(a).colCount(i.pos - o) + i.nodeAfter.attrs.colspan - 1, c = e.domAtPos(i.start(-1)).node;
	for (; c && c.nodeName != "TABLE";) c = c.parentNode;
	c && Fb(a, c.firstChild, c, r, s, n);
}
function Yb(e) {
	return Array(e).fill(0);
}
function Xb(e, t) {
	let n = [], r = e.doc.resolve(t), i = r.node(-1);
	if (!i) return K.empty;
	let a = Z.get(i), o = r.start(-1), s = a.colCount(r.pos - o) + r.nodeAfter.attrs.colspan - 1;
	for (let t = 0; t < a.height; t++) {
		let r = s + t * a.width;
		if ((s == a.width - 1 || a.map[r] != a.map[r + 1]) && (t == 0 || a.map[r] != a.map[r - a.width])) {
			let t = a.map[r], s = o + t + i.nodeAt(t).nodeSize - 1, c = document.createElement("div");
			c.className = "column-resize-handle", Ib.getState(e)?.dragging && n.push(Dm.node(o + t, o + t + i.nodeAt(t).nodeSize, { class: "column-resize-dragging" })), n.push(Dm.widget(s, c));
		}
	}
	return K.create(e.doc, n);
}
function Zb({ allowTableNodeSelection: e = !1 } = {}) {
	return new hl({
		key: _y,
		state: {
			init() {
				return null;
			},
			apply(e, t) {
				let n = e.getMeta(_y);
				if (n != null) return n == -1 ? null : n;
				if (t == null || !e.docChanged) return t;
				let { deleted: r, pos: i } = e.mapping.mapResult(t);
				return r ? null : i;
			}
		},
		props: {
			decorations: Ay,
			handleDOMEvents: { mousedown: Ab },
			createSelectionBetween(e) {
				return _y.getState(e.state) == null ? null : e.state.selection;
			},
			handleTripleClick: Ob,
			handleKeyDown: wb,
			handlePaste: kb
		},
		appendTransaction(t, n, r) {
			return Ny(r, Iy(r, n), e);
		}
	});
}
//#endregion
//#region node_modules/.pnpm/prosemirror-safari-ime-span@1.0.2/node_modules/prosemirror-safari-ime-span/dist/index.js
var Qb = typeof navigator < "u" ? navigator : null, $b = Qb && Qb.userAgent || "", ex = /Edge\/(\d+)/.exec($b), tx = /MSIE \d/.exec($b), nx = /Trident\/(?:[7-9]|\d{2,})\..*rv:(\d+)/.exec($b), rx = !(tx || nx || ex) && !!Qb && /Apple Computer/.test(Qb.vendor), ix = new vl("safari-ime-span"), ax = !1, ox = {
	key: ix,
	props: {
		decorations: sx,
		handleDOMEvents: {
			compositionstart: () => {
				ax = !0;
			},
			compositionend: () => {
				ax = !1;
			}
		}
	}
};
function sx(e) {
	let { $from: t, $to: n, to: r } = e.selection;
	if (ax && t.sameParent(n)) {
		let t = Dm.widget(r, cx, {
			ignoreSelection: !0,
			key: "safari-ime-span"
		});
		return K.create(e.doc, [t]);
	}
}
function cx(e) {
	let t = e.dom.ownerDocument.createElement("span");
	return t.className = "ProseMirror-safari-ime-span", t;
}
var lx = new hl(rx ? ox : { key: ix });
//#endregion
//#region node_modules/.pnpm/ccount@2.0.1/node_modules/ccount/index.js
function ux(e, t) {
	let n = String(e);
	if (typeof t != "string") throw TypeError("Expected character");
	let r = 0, i = n.indexOf(t);
	for (; i !== -1;) r++, i = n.indexOf(t, i + t.length);
	return r;
}
//#endregion
//#region node_modules/.pnpm/escape-string-regexp@5.0.0/node_modules/escape-string-regexp/index.js
function dx(e) {
	if (typeof e != "string") throw TypeError("Expected a string");
	return e.replace(/[|\\{}()[\]^$+*?.]/g, "\\$&").replace(/-/g, "\\x2d");
}
//#endregion
//#region node_modules/.pnpm/mdast-util-find-and-replace@3.0.3/node_modules/mdast-util-find-and-replace/lib/index.js
function fx(e, t, n) {
	let r = Or((n || {}).ignore || []), i = px(t), a = -1;
	for (; ++a < i.length;) ci(e, "text", o);
	function o(e, t) {
		let n = -1, i;
		for (; ++n < t.length;) {
			let e = t[n], a = i ? i.children : void 0;
			if (r(e, a ? a.indexOf(e) : void 0, i)) return;
			i = e;
		}
		if (i) return s(e, t);
	}
	function s(e, t) {
		let n = t[t.length - 1], r = i[a][0], o = i[a][1], s = 0, c = n.children.indexOf(e), l = !1, u = [];
		r.lastIndex = 0;
		let d = r.exec(e.value);
		for (; d;) {
			let n = d.index, i = {
				index: d.index,
				input: d.input,
				stack: [...t, e]
			}, a = o(...d, i);
			if (typeof a == "string" && (a = a.length > 0 ? {
				type: "text",
				value: a
			} : void 0), a === !1) r.lastIndex = n + 1;
			else {
				if (s !== n && f({
					type: "text",
					value: e.value.slice(s, n)
				}), Array.isArray(a)) for (let e of a) f(e);
				else a && f(a);
				s = n + d[0].length, l = !0;
			}
			if (!r.global) break;
			d = r.exec(e.value);
		}
		return l ? (s < e.value.length && f({
			type: "text",
			value: e.value.slice(s)
		}), n.children.splice(c, 1, ...u)) : u = [e], c + u.length;
		function f(e) {
			let t = u[u.length - 1];
			t && t.type === "text" && e.type === "text" ? t.value += e.value : u.push(e);
		}
	}
}
function px(e) {
	if (!Array.isArray(e)) throw TypeError("Expected find and replace tuple or list of tuples");
	let t = [], n = !e[0] || Array.isArray(e[0]) ? e : [e], r = -1;
	for (; ++r < n.length;) {
		let e = n[r];
		t.push([mx(e[0]), hx(e[1])]);
	}
	return t;
}
function mx(e) {
	return typeof e == "string" ? new RegExp(dx(e), "g") : e;
}
function hx(e) {
	return typeof e == "function" ? e : function() {
		return e;
	};
}
//#endregion
//#region node_modules/.pnpm/mdast-util-gfm-autolink-literal@2.0.1/node_modules/mdast-util-gfm-autolink-literal/lib/index.js
var gx = "phrasing", _x = [
	"autolink",
	"link",
	"image",
	"label"
];
function vx() {
	return {
		transforms: [Ex],
		enter: {
			literalAutolink: bx,
			literalAutolinkEmail: xx,
			literalAutolinkHttp: xx,
			literalAutolinkWww: xx
		},
		exit: {
			literalAutolink: Tx,
			literalAutolinkEmail: wx,
			literalAutolinkHttp: Sx,
			literalAutolinkWww: Cx
		}
	};
}
function yx() {
	return { unsafe: [
		{
			character: "@",
			before: "[+\\-.\\w]",
			after: "[\\-.\\w]",
			inConstruct: gx,
			notInConstruct: _x
		},
		{
			character: ".",
			before: "[Ww]",
			after: "[\\-.\\w]",
			inConstruct: gx,
			notInConstruct: _x
		},
		{
			character: ":",
			before: "[ps]",
			after: "\\/",
			inConstruct: gx,
			notInConstruct: _x
		}
	] };
}
function bx(e) {
	this.enter({
		type: "link",
		title: null,
		url: "",
		children: []
	}, e);
}
function xx(e) {
	this.config.enter.autolinkProtocol.call(this, e);
}
function Sx(e) {
	this.config.exit.autolinkProtocol.call(this, e);
}
function Cx(e) {
	this.config.exit.data.call(this, e);
	let t = this.stack[this.stack.length - 1];
	t.type, t.url = "http://" + this.sliceSerialize(e);
}
function wx(e) {
	this.config.exit.autolinkEmail.call(this, e);
}
function Tx(e) {
	this.exit(e);
}
function Ex(e) {
	fx(e, [[/(https?:\/\/|www(?=\.))([-.\w]+)([^ \t\r\n]*)/gi, Dx], [/(?<=^|\s|\p{P}|\p{S})([-.\w+]+)@([-\w]+(?:\.[-\w]+)+)/gu, Ox]], { ignore: ["link", "linkReference"] });
}
function Dx(e, t, n, r, i) {
	let a = "";
	if (!jx(i) || (/^w/i.test(t) && (n = t + n, t = "", a = "http://"), !kx(n))) return !1;
	let o = Ax(n + r);
	if (!o[0]) return !1;
	let s = {
		type: "link",
		title: null,
		url: a + t + o[0],
		children: [{
			type: "text",
			value: t + o[0]
		}]
	};
	return o[1] ? [s, {
		type: "text",
		value: o[1]
	}] : s;
}
function Ox(e, t, n, r) {
	return !jx(r, !0) || /[-\d_]$/.test(n) ? !1 : {
		type: "link",
		title: null,
		url: "mailto:" + t + "@" + n,
		children: [{
			type: "text",
			value: t + "@" + n
		}]
	};
}
function kx(e) {
	let t = e.split(".");
	return !(t.length < 2 || t[t.length - 1] && (/_/.test(t[t.length - 1]) || !/[a-zA-Z\d]/.test(t[t.length - 1])) || t[t.length - 2] && (/_/.test(t[t.length - 2]) || !/[a-zA-Z\d]/.test(t[t.length - 2])));
}
function Ax(e) {
	let t = /[!"&'),.:;<>?\]}]+$/.exec(e);
	if (!t) return [e, void 0];
	e = e.slice(0, t.index);
	let n = t[0], r = n.indexOf(")"), i = ux(e, "("), a = ux(e, ")");
	for (; r !== -1 && i > a;) e += n.slice(0, r + 1), n = n.slice(r + 1), r = n.indexOf(")"), a++;
	return [e, n];
}
function jx(e, t) {
	let n = e.input.charCodeAt(e.index - 1);
	return (e.index === 0 || Je(n) || qe(n)) && (!t || n !== 47);
}
//#endregion
//#region node_modules/.pnpm/mdast-util-gfm-footnote@2.1.0/node_modules/mdast-util-gfm-footnote/lib/index.js
Vx.peek = Bx;
function Mx() {
	this.buffer();
}
function Nx(e) {
	this.enter({
		type: "footnoteReference",
		identifier: "",
		label: ""
	}, e);
}
function Px() {
	this.buffer();
}
function Fx(e) {
	this.enter({
		type: "footnoteDefinition",
		identifier: "",
		label: "",
		children: []
	}, e);
}
function Ix(e) {
	let t = this.resume(), n = this.stack[this.stack.length - 1];
	n.type, n.identifier = Be(this.sliceSerialize(e)).toLowerCase(), n.label = t;
}
function Lx(e) {
	this.exit(e);
}
function Rx(e) {
	let t = this.resume(), n = this.stack[this.stack.length - 1];
	n.type, n.identifier = Be(this.sliceSerialize(e)).toLowerCase(), n.label = t;
}
function zx(e) {
	this.exit(e);
}
function Bx() {
	return "[";
}
function Vx(e, t, n, r) {
	let i = n.createTracker(r), a = i.move("[^"), o = n.enter("footnoteReference"), s = n.enter("reference");
	return a += i.move(n.safe(n.associationId(e), {
		after: "]",
		before: a
	})), s(), o(), a += i.move("]"), a;
}
function Hx() {
	return {
		enter: {
			gfmFootnoteCallString: Mx,
			gfmFootnoteCall: Nx,
			gfmFootnoteDefinitionLabelString: Px,
			gfmFootnoteDefinition: Fx
		},
		exit: {
			gfmFootnoteCallString: Ix,
			gfmFootnoteCall: Lx,
			gfmFootnoteDefinitionLabelString: Rx,
			gfmFootnoteDefinition: zx
		}
	};
}
function Ux(e) {
	let t = !1;
	return e && e.firstLineBlank && (t = !0), {
		handlers: {
			footnoteDefinition: n,
			footnoteReference: Vx
		},
		unsafe: [{
			character: "[",
			inConstruct: [
				"label",
				"phrasing",
				"reference"
			]
		}]
	};
	function n(e, n, r, i) {
		let a = r.createTracker(i), o = a.move("[^"), s = r.enter("footnoteDefinition"), c = r.enter("label");
		return o += a.move(r.safe(r.associationId(e), {
			before: o,
			after: "]"
		})), c(), o += a.move("]:"), e.children && e.children.length > 0 && (a.shift(4), o += a.move((t ? "\n" : " ") + r.indentLines(r.containerFlow(e, a.current()), t ? Gx : Wx))), s(), o;
	}
}
function Wx(e, t, n) {
	return t === 0 ? e : Gx(e, t, n);
}
function Gx(e, t, n) {
	return (n ? "" : "    ") + e;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-gfm-strikethrough@2.0.1/node_modules/mdast-util-gfm-strikethrough/lib/index.js
var Kx = [
	"autolink",
	"destinationLiteral",
	"destinationRaw",
	"reference",
	"titleQuote",
	"titleApostrophe"
];
Zx.attention = Qx, Zx.peek = $x;
function qx() {
	return {
		canContainEols: ["delete"],
		enter: { strikethrough: Yx },
		exit: { strikethrough: Xx }
	};
}
function Jx() {
	return {
		handlers: { delete: Zx },
		unsafe: [{
			character: "~",
			inConstruct: "phrasing",
			notInConstruct: Kx
		}]
	};
}
function Yx(e) {
	this.enter({
		type: "delete",
		children: [],
		position: void 0
	}, e);
}
function Xx(e) {
	this.exit(e);
}
function Zx(e, t, n, r) {
	let i = n.createTracker(r), a = n.stack.includes("strikethrough") ? "~" : "~~", o = n.enter("strikethrough"), s = i.move(a);
	return s += n.containerPhrasing(e, {
		...i.current(),
		before: s,
		after: "~"
	}), s += i.move(a), o(), s;
}
function Qx() {
	return {
		construct: "strikethrough",
		markers: ["~"],
		sizes: [2, 1]
	};
}
function $x() {
	return "~";
}
//#endregion
//#region node_modules/.pnpm/markdown-table@3.0.4/node_modules/markdown-table/index.js
function eS(e) {
	return e.length;
}
function tS(e, t) {
	let n = t || {}, r = (n.align || []).concat(), i = n.stringLength || eS, a = [], o = [], s = [], c = [], l = 0, u = -1;
	for (; ++u < e.length;) {
		let t = [], r = [], a = -1;
		for (e[u].length > l && (l = e[u].length); ++a < e[u].length;) {
			let o = nS(e[u][a]);
			if (n.alignDelimiters !== !1) {
				let e = i(o);
				r[a] = e, (c[a] === void 0 || e > c[a]) && (c[a] = e);
			}
			t.push(o);
		}
		o[u] = t, s[u] = r;
	}
	let d = -1;
	if (typeof r == "object" && "length" in r) for (; ++d < l;) a[d] = rS(r[d]);
	else {
		let e = rS(r);
		for (; ++d < l;) a[d] = e;
	}
	d = -1;
	let f = [], p = [];
	for (; ++d < l;) {
		let e = a[d], t = "", r = "";
		e === 99 ? (t = ":", r = ":") : e === 108 ? t = ":" : e === 114 && (r = ":");
		let i = n.alignDelimiters === !1 ? 1 : Math.max(1, c[d] - t.length - r.length), o = t + "-".repeat(i) + r;
		n.alignDelimiters !== !1 && (i = t.length + i + r.length, i > c[d] && (c[d] = i), p[d] = i), f[d] = o;
	}
	o.splice(1, 0, f), s.splice(1, 0, p), u = -1;
	let m = [];
	for (; ++u < o.length;) {
		let e = o[u], t = s[u];
		d = -1;
		let r = [];
		for (; ++d < l;) {
			let i = e[d] || "", o = "", s = "";
			if (n.alignDelimiters !== !1) {
				let e = c[d] - (t[d] || 0), n = a[d];
				n === 114 ? o = " ".repeat(e) : n === 99 ? e % 2 ? (o = " ".repeat(e / 2 + .5), s = " ".repeat(e / 2 - .5)) : (o = " ".repeat(e / 2), s = o) : s = " ".repeat(e);
			}
			n.delimiterStart !== !1 && !d && r.push("|"), n.padding !== !1 && (n.alignDelimiters !== !1 || i !== "") && (n.delimiterStart !== !1 || d) && r.push(" "), n.alignDelimiters !== !1 && r.push(o), r.push(i), n.alignDelimiters !== !1 && r.push(s), n.padding !== !1 && r.push(" "), (n.delimiterEnd !== !1 || d !== l - 1) && r.push("|");
		}
		m.push(n.delimiterEnd === !1 ? r.join("").replace(/ +$/, "") : r.join(""));
	}
	return m.join("\n");
}
function nS(e) {
	return e == null ? "" : String(e);
}
function rS(e) {
	let t = typeof e == "string" ? e.codePointAt(0) : 0;
	return t === 67 || t === 99 ? 99 : t === 76 || t === 108 ? 108 : t === 82 || t === 114 ? 114 : 0;
}
//#endregion
//#region node_modules/.pnpm/mdast-util-gfm-table@2.0.0/node_modules/mdast-util-gfm-table/lib/index.js
function iS() {
	return {
		enter: {
			table: aS,
			tableData: lS,
			tableHeader: lS,
			tableRow: sS
		},
		exit: {
			codeText: uS,
			table: oS,
			tableData: cS,
			tableHeader: cS,
			tableRow: cS
		}
	};
}
function aS(e) {
	let t = e._align;
	this.enter({
		type: "table",
		align: t.map(function(e) {
			return e === "none" ? null : e;
		}),
		children: []
	}, e), this.data.inTable = !0;
}
function oS(e) {
	this.exit(e), this.data.inTable = void 0;
}
function sS(e) {
	this.enter({
		type: "tableRow",
		children: []
	}, e);
}
function cS(e) {
	this.exit(e);
}
function lS(e) {
	this.enter({
		type: "tableCell",
		children: []
	}, e);
}
function uS(e) {
	let t = this.resume();
	this.data.inTable && (t = t.replace(/\\([\\|])/g, dS));
	let n = this.stack[this.stack.length - 1];
	n.type, n.value = t, this.exit(e);
}
function dS(e, t) {
	return t === "|" ? t : e;
}
function fS(e) {
	let t = e || {}, n = t.tableCellPadding, r = t.tablePipeAlign, i = t.stringLength, a = n ? " " : "|";
	return {
		unsafe: [
			{
				character: "\r",
				inConstruct: "tableCell"
			},
			{
				character: "\n",
				inConstruct: "tableCell"
			},
			{
				atBreak: !0,
				character: "|",
				after: "[	 :-]"
			},
			{
				character: "|",
				inConstruct: "tableCell"
			},
			{
				atBreak: !0,
				character: ":",
				after: "-"
			},
			{
				atBreak: !0,
				character: "-",
				after: "[:|-]"
			}
		],
		handlers: {
			inlineCode: f,
			table: o,
			tableCell: c,
			tableRow: s
		}
	};
	function o(e, t, n, r) {
		return l(u(e, n, r), e.align);
	}
	function s(e, t, n, r) {
		let i = l([d(e, n, r)]);
		return i.slice(0, i.indexOf("\n"));
	}
	function c(e, t, n, r) {
		let i = n.enter("tableCell"), o = n.enter("phrasing"), s = n.containerPhrasing(e, {
			...r,
			before: a,
			after: a
		});
		return o(), i(), s;
	}
	function l(e, t) {
		return tS(e, {
			align: t,
			alignDelimiters: r,
			padding: n,
			stringLength: i
		});
	}
	function u(e, t, n) {
		let r = e.children, i = -1, a = [], o = t.enter("table");
		for (; ++i < r.length;) a[i] = d(r[i], t, n);
		return o(), a;
	}
	function d(e, t, n) {
		let r = e.children, i = -1, a = [], o = t.enter("tableRow");
		for (; ++i < r.length;) a[i] = c(r[i], e, t, n);
		return o(), a;
	}
	function f(e, t, n) {
		let r = Hi.inlineCode(e, t, n);
		return n.stack.includes("tableCell") && (r = r.replace(/\|/g, "\\$&")), r;
	}
}
//#endregion
//#region node_modules/.pnpm/mdast-util-gfm-task-list-item@2.0.0/node_modules/mdast-util-gfm-task-list-item/lib/index.js
function pS() {
	return { exit: {
		taskListCheckValueChecked: hS,
		taskListCheckValueUnchecked: hS,
		paragraph: gS
	} };
}
function mS() {
	return {
		unsafe: [{
			atBreak: !0,
			character: "-",
			after: "[:|-]"
		}],
		handlers: { listItem: _S }
	};
}
function hS(e) {
	let t = this.stack[this.stack.length - 2];
	t.type, t.checked = e.type === "taskListCheckValueChecked";
}
function gS(e) {
	let t = this.stack[this.stack.length - 2];
	if (t && t.type === "listItem" && typeof t.checked == "boolean") {
		let e = this.stack[this.stack.length - 1];
		e.type;
		let n = e.children[0];
		if (n && n.type === "text") {
			let r = t.children, i = -1, a;
			for (; ++i < r.length;) {
				let e = r[i];
				if (e.type === "paragraph") {
					a = e;
					break;
				}
			}
			a === e && (n.value = n.value.slice(1), n.value.length === 0 ? e.children.shift() : e.position && n.position && typeof n.position.start.offset == "number" && (n.position.start.column++, n.position.start.offset++, e.position.start = Object.assign({}, n.position.start)));
		}
	}
	this.exit(e);
}
function _S(e, t, n, r) {
	let i = e.children[0], a = typeof e.checked == "boolean" && i && i.type === "paragraph", o = "[" + (e.checked ? "x" : " ") + "] ", s = n.createTracker(r);
	a && s.move(o);
	let c = Hi.listItem(e, t, n, {
		...r,
		...s.current()
	});
	return a && (c = c.replace(/^(?:[*+-]|\d+\.)([\r\n]| {1,3})/, l)), c;
	function l(e) {
		return e + o;
	}
}
//#endregion
//#region node_modules/.pnpm/mdast-util-gfm@3.1.0/node_modules/mdast-util-gfm/lib/index.js
function vS() {
	return [
		vx(),
		Hx(),
		qx(),
		iS(),
		pS()
	];
}
function yS(e) {
	return { extensions: [
		yx(),
		Ux(e),
		Jx(),
		fS(e),
		mS()
	] };
}
//#endregion
//#region node_modules/.pnpm/micromark-extension-gfm-autolink-literal@2.1.0/node_modules/micromark-extension-gfm-autolink-literal/lib/syntax.js
var bS = {
	tokenize: PS,
	partial: !0
}, xS = {
	tokenize: FS,
	partial: !0
}, SS = {
	tokenize: IS,
	partial: !0
}, CS = {
	tokenize: LS,
	partial: !0
}, wS = {
	tokenize: RS,
	partial: !0
}, TS = {
	name: "wwwAutolink",
	tokenize: MS,
	previous: zS
}, ES = {
	name: "protocolAutolink",
	tokenize: NS,
	previous: BS
}, DS = {
	name: "emailAutolink",
	tokenize: jS,
	previous: VS
}, OS = {};
function kS() {
	return { text: OS };
}
for (var AS = 48; AS < 123;) OS[AS] = DS, AS++, AS === 58 ? AS = 65 : AS === 91 && (AS = 97);
OS[43] = DS, OS[45] = DS, OS[46] = DS, OS[95] = DS, OS[72] = [DS, ES], OS[104] = [DS, ES], OS[87] = [DS, TS], OS[119] = [DS, TS];
function jS(e, t, n) {
	let r = this, i, a;
	return o;
	function o(t) {
		return !HS(t) || !VS.call(r, r.previous) || US(r.events) ? n(t) : (e.enter("literalAutolink"), e.enter("literalAutolinkEmail"), s(t));
	}
	function s(t) {
		return HS(t) ? (e.consume(t), s) : t === 64 ? (e.consume(t), c) : n(t);
	}
	function c(t) {
		return t === 46 ? e.check(wS, u, l)(t) : t === 45 || t === 95 || Ve(t) ? (a = !0, e.consume(t), c) : u(t);
	}
	function l(t) {
		return e.consume(t), i = !0, c;
	}
	function u(o) {
		return a && i && D(r.previous) ? (e.exit("literalAutolinkEmail"), e.exit("literalAutolink"), t(o)) : n(o);
	}
}
function MS(e, t, n) {
	let r = this;
	return i;
	function i(t) {
		return t !== 87 && t !== 119 || !zS.call(r, r.previous) || US(r.events) ? n(t) : (e.enter("literalAutolink"), e.enter("literalAutolinkWww"), e.check(bS, e.attempt(xS, e.attempt(SS, a), n), n)(t));
	}
	function a(n) {
		return e.exit("literalAutolinkWww"), e.exit("literalAutolink"), t(n);
	}
}
function NS(e, t, n) {
	let r = this, i = "", a = !1;
	return o;
	function o(t) {
		return (t === 72 || t === 104) && BS.call(r, r.previous) && !US(r.events) ? (e.enter("literalAutolink"), e.enter("literalAutolinkHttp"), i += String.fromCodePoint(t), e.consume(t), s) : n(t);
	}
	function s(t) {
		if (D(t) && i.length < 5) return i += String.fromCodePoint(t), e.consume(t), s;
		if (t === 58) {
			let n = i.toLowerCase();
			if (n === "http" || n === "https") return e.consume(t), c;
		}
		return n(t);
	}
	function c(t) {
		return t === 47 ? (e.consume(t), a ? l : (a = !0, c)) : n(t);
	}
	function l(t) {
		return t === null || Ue(t) || k(t) || Je(t) || qe(t) ? n(t) : e.attempt(xS, e.attempt(SS, u), n)(t);
	}
	function u(n) {
		return e.exit("literalAutolinkHttp"), e.exit("literalAutolink"), t(n);
	}
}
function PS(e, t, n) {
	let r = 0;
	return i;
	function i(t) {
		return (t === 87 || t === 119) && r < 3 ? (r++, e.consume(t), i) : t === 46 && r === 3 ? (e.consume(t), a) : n(t);
	}
	function a(e) {
		return e === null ? n(e) : t(e);
	}
}
function FS(e, t, n) {
	let r, i, a;
	return o;
	function o(t) {
		return t === 46 || t === 95 ? e.check(CS, c, s)(t) : t === null || k(t) || Je(t) || t !== 45 && qe(t) ? c(t) : (a = !0, e.consume(t), o);
	}
	function s(t) {
		return t === 95 ? r = !0 : (i = r, r = void 0), e.consume(t), o;
	}
	function c(e) {
		return i || r || !a ? n(e) : t(e);
	}
}
function IS(e, t) {
	let n = 0, r = 0;
	return i;
	function i(o) {
		return o === 40 ? (n++, e.consume(o), i) : o === 41 && r < n ? a(o) : o === 33 || o === 34 || o === 38 || o === 39 || o === 41 || o === 42 || o === 44 || o === 46 || o === 58 || o === 59 || o === 60 || o === 63 || o === 93 || o === 95 || o === 126 ? e.check(CS, t, a)(o) : o === null || k(o) || Je(o) ? t(o) : (e.consume(o), i);
	}
	function a(t) {
		return t === 41 && r++, e.consume(t), i;
	}
}
function LS(e, t, n) {
	return r;
	function r(o) {
		return o === 33 || o === 34 || o === 39 || o === 41 || o === 42 || o === 44 || o === 46 || o === 58 || o === 59 || o === 63 || o === 95 || o === 126 ? (e.consume(o), r) : o === 38 ? (e.consume(o), a) : o === 93 ? (e.consume(o), i) : o === 60 || o === null || k(o) || Je(o) ? t(o) : n(o);
	}
	function i(e) {
		return e === null || e === 40 || e === 91 || k(e) || Je(e) ? t(e) : r(e);
	}
	function a(e) {
		return D(e) ? o(e) : n(e);
	}
	function o(t) {
		return t === 59 ? (e.consume(t), r) : D(t) ? (e.consume(t), o) : n(t);
	}
}
function RS(e, t, n) {
	return r;
	function r(t) {
		return e.consume(t), i;
	}
	function i(e) {
		return Ve(e) ? n(e) : t(e);
	}
}
function zS(e) {
	return e === null || e === 40 || e === 42 || e === 95 || e === 91 || e === 93 || e === 126 || k(e);
}
function BS(e) {
	return !D(e);
}
function VS(e) {
	return !(e === 47 || HS(e));
}
function HS(e) {
	return e === 43 || e === 45 || e === 46 || e === 95 || Ve(e);
}
function US(e) {
	let t = e.length, n = !1;
	for (; t--;) {
		let r = e[t][1];
		if ((r.type === "labelLink" || r.type === "labelImage") && !r._balanced) {
			n = !0;
			break;
		}
		if (r._gfmAutolinkLiteralWalkedInto) {
			n = !1;
			break;
		}
	}
	return e.length > 0 && !n && (e[e.length - 1][1]._gfmAutolinkLiteralWalkedInto = !0), n;
}
//#endregion
//#region node_modules/.pnpm/micromark-extension-gfm-footnote@2.1.0/node_modules/micromark-extension-gfm-footnote/lib/syntax.js
var WS = {
	tokenize: QS,
	partial: !0
};
function GS() {
	return {
		document: { 91: {
			name: "gfmFootnoteDefinition",
			tokenize: YS,
			continuation: { tokenize: XS },
			exit: ZS
		} },
		text: {
			91: {
				name: "gfmFootnoteCall",
				tokenize: JS
			},
			93: {
				name: "gfmPotentialFootnoteCall",
				add: "after",
				tokenize: KS,
				resolveTo: qS
			}
		}
	};
}
function KS(e, t, n) {
	let r = this, i = r.events.length, a = r.parser.gfmFootnotes || (r.parser.gfmFootnotes = []), o;
	for (; i--;) {
		let e = r.events[i][1];
		if (e.type === "labelImage") {
			o = e;
			break;
		}
		if (e.type === "gfmFootnoteCall" || e.type === "labelLink" || e.type === "label" || e.type === "image" || e.type === "link") break;
	}
	return s;
	function s(i) {
		if (!o || !o._balanced) return n(i);
		let s = Be(r.sliceSerialize({
			start: o.end,
			end: r.now()
		}));
		return s.codePointAt(0) !== 94 || !a.includes(s.slice(1)) ? n(i) : (e.enter("gfmFootnoteCallLabelMarker"), e.consume(i), e.exit("gfmFootnoteCallLabelMarker"), t(i));
	}
}
function qS(e, t) {
	let n = e.length;
	for (; n--;) if (e[n][1].type === "labelImage" && e[n][0] === "enter") {
		e[n][1];
		break;
	}
	e[n + 1][1].type = "data", e[n + 3][1].type = "gfmFootnoteCallLabelMarker";
	let r = {
		type: "gfmFootnoteCall",
		start: Object.assign({}, e[n + 3][1].start),
		end: Object.assign({}, e[e.length - 1][1].end)
	}, i = {
		type: "gfmFootnoteCallMarker",
		start: Object.assign({}, e[n + 3][1].end),
		end: Object.assign({}, e[n + 3][1].end)
	};
	i.end.column++, i.end.offset++, i.end._bufferIndex++;
	let a = {
		type: "gfmFootnoteCallString",
		start: Object.assign({}, i.end),
		end: Object.assign({}, e[e.length - 1][1].start)
	}, o = {
		type: "chunkString",
		contentType: "string",
		start: Object.assign({}, a.start),
		end: Object.assign({}, a.end)
	}, s = [
		e[n + 1],
		e[n + 2],
		[
			"enter",
			r,
			t
		],
		e[n + 3],
		e[n + 4],
		[
			"enter",
			i,
			t
		],
		[
			"exit",
			i,
			t
		],
		[
			"enter",
			a,
			t
		],
		[
			"enter",
			o,
			t
		],
		[
			"exit",
			o,
			t
		],
		[
			"exit",
			a,
			t
		],
		e[e.length - 2],
		e[e.length - 1],
		[
			"exit",
			r,
			t
		]
	];
	return e.splice(n, e.length - n + 1, ...s), e;
}
function JS(e, t, n) {
	let r = this, i = r.parser.gfmFootnotes || (r.parser.gfmFootnotes = []), a = 0, o;
	return s;
	function s(t) {
		return e.enter("gfmFootnoteCall"), e.enter("gfmFootnoteCallLabelMarker"), e.consume(t), e.exit("gfmFootnoteCallLabelMarker"), c;
	}
	function c(t) {
		return t === 94 ? (e.enter("gfmFootnoteCallMarker"), e.consume(t), e.exit("gfmFootnoteCallMarker"), e.enter("gfmFootnoteCallString"), e.enter("chunkString").contentType = "string", l) : n(t);
	}
	function l(s) {
		if (a > 999 || s === 93 && !o || s === null || s === 91 || k(s)) return n(s);
		if (s === 93) {
			e.exit("chunkString");
			let a = e.exit("gfmFootnoteCallString");
			return i.includes(Be(r.sliceSerialize(a))) ? (e.enter("gfmFootnoteCallLabelMarker"), e.consume(s), e.exit("gfmFootnoteCallLabelMarker"), e.exit("gfmFootnoteCall"), t) : n(s);
		}
		return k(s) || (o = !0), a++, e.consume(s), s === 92 ? u : l;
	}
	function u(t) {
		return t === 91 || t === 92 || t === 93 ? (e.consume(t), a++, l) : l(t);
	}
}
function YS(e, t, n) {
	let r = this, i = r.parser.gfmFootnotes || (r.parser.gfmFootnotes = []), a, o = 0, s;
	return c;
	function c(t) {
		return e.enter("gfmFootnoteDefinition")._container = !0, e.enter("gfmFootnoteDefinitionLabel"), e.enter("gfmFootnoteDefinitionLabelMarker"), e.consume(t), e.exit("gfmFootnoteDefinitionLabelMarker"), l;
	}
	function l(t) {
		return t === 94 ? (e.enter("gfmFootnoteDefinitionMarker"), e.consume(t), e.exit("gfmFootnoteDefinitionMarker"), e.enter("gfmFootnoteDefinitionLabelString"), e.enter("chunkString").contentType = "string", u) : n(t);
	}
	function u(t) {
		if (o > 999 || t === 93 && !s || t === null || t === 91 || k(t)) return n(t);
		if (t === 93) {
			e.exit("chunkString");
			let n = e.exit("gfmFootnoteDefinitionLabelString");
			return a = Be(r.sliceSerialize(n)), e.enter("gfmFootnoteDefinitionLabelMarker"), e.consume(t), e.exit("gfmFootnoteDefinitionLabelMarker"), e.exit("gfmFootnoteDefinitionLabel"), f;
		}
		return k(t) || (s = !0), o++, e.consume(t), t === 92 ? d : u;
	}
	function d(t) {
		return t === 91 || t === 92 || t === 93 ? (e.consume(t), o++, u) : u(t);
	}
	function f(t) {
		return t === 58 ? (e.enter("definitionMarker"), e.consume(t), e.exit("definitionMarker"), i.includes(a) || i.push(a), j(e, p, "gfmFootnoteDefinitionWhitespace")) : n(t);
	}
	function p(e) {
		return t(e);
	}
}
function XS(e, t, n) {
	return e.check(pt, t, e.attempt(WS, t, n));
}
function ZS(e) {
	e.exit("gfmFootnoteDefinition");
}
function QS(e, t, n) {
	let r = this;
	return j(e, i, "gfmFootnoteDefinitionIndent", 5);
	function i(e) {
		let i = r.events[r.events.length - 1];
		return i && i[1].type === "gfmFootnoteDefinitionIndent" && i[2].sliceSerialize(i[1], !0).length === 4 ? t(e) : n(e);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-extension-gfm-strikethrough@2.1.0/node_modules/micromark-extension-gfm-strikethrough/lib/syntax.js
function $S(e) {
	let t = (e || {}).singleTilde, n = {
		name: "strikethrough",
		tokenize: i,
		resolveAll: r
	};
	return t ??= !0, {
		text: { 126: n },
		insideSpan: { null: [n] },
		attentionMarkers: { null: [126] }
	};
	function r(e, t) {
		let n = -1;
		for (; ++n < e.length;) if (e[n][0] === "enter" && e[n][1].type === "strikethroughSequenceTemporary" && e[n][1]._close) {
			let r = n;
			for (; r--;) if (e[r][0] === "exit" && e[r][1].type === "strikethroughSequenceTemporary" && e[r][1]._open && e[n][1].end.offset - e[n][1].start.offset === e[r][1].end.offset - e[r][1].start.offset) {
				e[n][1].type = "strikethroughSequence", e[r][1].type = "strikethroughSequence";
				let i = {
					type: "strikethrough",
					start: Object.assign({}, e[r][1].start),
					end: Object.assign({}, e[n][1].end)
				}, a = {
					type: "strikethroughText",
					start: Object.assign({}, e[r][1].end),
					end: Object.assign({}, e[n][1].start)
				}, o = [
					[
						"enter",
						i,
						t
					],
					[
						"enter",
						e[r][1],
						t
					],
					[
						"exit",
						e[r][1],
						t
					],
					[
						"enter",
						a,
						t
					]
				], s = t.parser.constructs.insideSpan.null;
				s && Ne(o, o.length, 0, ot(s, e.slice(r + 1, n), t)), Ne(o, o.length, 0, [
					[
						"exit",
						a,
						t
					],
					[
						"enter",
						e[n][1],
						t
					],
					[
						"exit",
						e[n][1],
						t
					],
					[
						"exit",
						i,
						t
					]
				]), Ne(e, r - 1, n - r + 3, o), n = r + o.length - 2;
				break;
			}
		}
		for (n = -1; ++n < e.length;) e[n][1].type === "strikethroughSequenceTemporary" && (e[n][1].type = "data");
		return e;
	}
	function i(e, n, r) {
		let i = this.previous, a = this.events, o = 0;
		return s;
		function s(t) {
			return i === 126 && a[a.length - 1][1].type !== "characterEscape" ? r(t) : (e.enter("strikethroughSequenceTemporary"), c(t));
		}
		function c(a) {
			let s = at(i);
			if (a === 126) return o > 1 ? r(a) : (e.consume(a), o++, c);
			if (o < 2 && !t) return r(a);
			let l = e.exit("strikethroughSequenceTemporary"), u = at(a);
			return l._open = !u || u === 2 && !!s, l._close = !s || s === 2 && !!u, n(a);
		}
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-extension-gfm-table@2.1.2/node_modules/micromark-extension-gfm-table/lib/edit-map.js
var eC = class {
	constructor() {
		this.map = [], this.index = /* @__PURE__ */ new Map();
	}
	add(e, t, n) {
		tC(this, e, t, n);
	}
	consume(e) {
		/* c8 ignore next 3 -- `resolve` is never called without tables, so without edits. */
		if (this.map.sort(function(e, t) {
			return e[0] - t[0];
		}), this.map.length === 0) return;
		let t = this.map.length, n = [];
		for (; t > 0;) --t, n.push(e.slice(this.map[t][0] + this.map[t][1]), this.map[t][2]), e.length = this.map[t][0];
		n.push(e.slice()), e.length = 0;
		let r = n.pop();
		for (; r;) {
			for (let t of r) e.push(t);
			r = n.pop();
		}
		this.map.length = 0, this.index.clear();
	}
};
function tC(e, t, n, r) {
	/* c8 ignore next 3 -- `resolve` is never called without tables, so without edits. */
	if (n === 0 && r.length === 0) return;
	let i = e.index.get(t);
	if (i) {
		i[1] += n, i[2].push(...r);
		return;
	}
	let a = [
		t,
		n,
		r
	];
	e.map.push(a), e.index.set(t, a);
}
//#endregion
//#region node_modules/.pnpm/micromark-extension-gfm-table@2.1.2/node_modules/micromark-extension-gfm-table/lib/infer.js
function nC(e, t) {
	let n = !1, r = [];
	for (; t < e.length;) {
		let i = e[t];
		if (n) {
			if (i[0] === "enter") i[1].type === "tableContent" && r.push(e[t + 1][1].type === "tableDelimiterMarker" ? "left" : "none");
			else if (i[1].type === "tableContent") {
				if (e[t - 1][1].type === "tableDelimiterMarker") {
					let e = r.length - 1;
					r[e] = r[e] === "left" ? "center" : "right";
				}
			} else if (i[1].type === "tableDelimiterRow") break;
		} else i[0] === "enter" && i[1].type === "tableDelimiterRow" && (n = !0);
		t += 1;
	}
	return r;
}
//#endregion
//#region node_modules/.pnpm/micromark-extension-gfm-table@2.1.2/node_modules/micromark-extension-gfm-table/lib/syntax.js
function rC() {
	return { flow: { null: {
		name: "table",
		tokenize: iC,
		resolveAll: aC
	} } };
}
function iC(e, t, n) {
	let r = this, i = 0, a = 0, o;
	return s;
	function s(e) {
		let t = r.events.length - 1;
		for (; t > -1;) {
			let { type: e } = r.events[t][1];
			if (e === "lineEnding" || e === "linePrefix") t--;
			else break;
		}
		let i = t > -1 ? r.events[t][1].type : null, a = i === "tableHead" || i === "tableRow" ? ee : c;
		return a === ee && r.parser.lazy[r.now().line] ? n(e) : a(e);
	}
	function c(t) {
		return e.enter("tableHead"), e.enter("tableRow"), l(t);
	}
	function l(e) {
		return e === 124 || (o = !0, a += 1), u(e);
	}
	function u(t) {
		return t === null ? n(t) : O(t) ? a > 1 ? (a = 0, r.interrupt = !0, e.exit("tableRow"), e.enter("lineEnding"), e.consume(t), e.exit("lineEnding"), p) : n(t) : A(t) ? j(e, u, "whitespace")(t) : (a += 1, o && (o = !1, i += 1), t === 124 ? (e.enter("tableCellDivider"), e.consume(t), e.exit("tableCellDivider"), o = !0, u) : (e.enter("data"), d(t)));
	}
	function d(t) {
		return t === null || t === 124 || k(t) ? (e.exit("data"), u(t)) : (e.consume(t), t === 92 ? f : d);
	}
	function f(t) {
		return t === 92 || t === 124 ? (e.consume(t), d) : d(t);
	}
	function p(t) {
		return r.interrupt = !1, r.parser.lazy[r.now().line] ? n(t) : (e.enter("tableDelimiterRow"), o = !1, A(t) ? j(e, m, "linePrefix", r.parser.constructs.disable.null.includes("codeIndented") ? void 0 : 4)(t) : m(t));
	}
	function m(t) {
		return t === 45 || t === 58 ? g(t) : t === 124 ? (o = !0, e.enter("tableCellDivider"), e.consume(t), e.exit("tableCellDivider"), h) : x(t);
	}
	function h(t) {
		return A(t) ? j(e, g, "whitespace")(t) : g(t);
	}
	function g(t) {
		return t === 58 ? (a += 1, o = !0, e.enter("tableDelimiterMarker"), e.consume(t), e.exit("tableDelimiterMarker"), _) : t === 45 ? (a += 1, _(t)) : t === null || O(t) ? b(t) : x(t);
	}
	function _(t) {
		return t === 45 ? (e.enter("tableDelimiterFiller"), v(t)) : x(t);
	}
	function v(t) {
		return t === 45 ? (e.consume(t), v) : t === 58 ? (o = !0, e.exit("tableDelimiterFiller"), e.enter("tableDelimiterMarker"), e.consume(t), e.exit("tableDelimiterMarker"), y) : (e.exit("tableDelimiterFiller"), y(t));
	}
	function y(t) {
		return A(t) ? j(e, b, "whitespace")(t) : b(t);
	}
	function b(n) {
		return n === 124 ? m(n) : n === null || O(n) ? !o || i !== a ? x(n) : (e.exit("tableDelimiterRow"), e.exit("tableHead"), t(n)) : x(n);
	}
	function x(e) {
		return n(e);
	}
	function ee(t) {
		return e.enter("tableRow"), S(t);
	}
	function S(n) {
		return n === 124 ? (e.enter("tableCellDivider"), e.consume(n), e.exit("tableCellDivider"), S) : n === null || O(n) ? (e.exit("tableRow"), t(n)) : A(n) ? j(e, S, "whitespace")(n) : (e.enter("data"), te(n));
	}
	function te(t) {
		return t === null || t === 124 || k(t) ? (e.exit("data"), S(t)) : (e.consume(t), t === 92 ? ne : te);
	}
	function ne(t) {
		return t === 92 || t === 124 ? (e.consume(t), te) : te(t);
	}
}
function aC(e, t) {
	let n = -1, r = !0, i = 0, a = [
		0,
		0,
		0,
		0
	], o = [
		0,
		0,
		0,
		0
	], s = !1, c = 0, l, u, d, f = new eC();
	for (; ++n < e.length;) {
		let p = e[n], m = p[1];
		p[0] === "enter" ? m.type === "tableHead" ? (s = !1, c !== 0 && (sC(f, t, c, l, u), u = void 0, c = 0), l = {
			type: "table",
			start: Object.assign({}, m.start),
			end: Object.assign({}, m.end)
		}, f.add(n, 0, [[
			"enter",
			l,
			t
		]])) : m.type === "tableRow" || m.type === "tableDelimiterRow" ? (r = !0, d = void 0, a = [
			0,
			0,
			0,
			0
		], o = [
			0,
			n + 1,
			0,
			0
		], s && (s = !1, u = {
			type: "tableBody",
			start: Object.assign({}, m.start),
			end: Object.assign({}, m.end)
		}, f.add(n, 0, [[
			"enter",
			u,
			t
		]])), i = m.type === "tableDelimiterRow" ? 2 : u ? 3 : 1) : i && (m.type === "data" || m.type === "tableDelimiterMarker" || m.type === "tableDelimiterFiller") ? (r = !1, o[2] === 0 && (a[1] !== 0 && (o[0] = o[1], d = oC(f, t, a, i, void 0, d), a = [
			0,
			0,
			0,
			0
		]), o[2] = n)) : m.type === "tableCellDivider" && (r ? r = !1 : (a[1] !== 0 && (o[0] = o[1], d = oC(f, t, a, i, void 0, d)), a = o, o = [
			a[1],
			n,
			0,
			0
		])) : m.type === "tableHead" ? (s = !0, c = n) : m.type === "tableRow" || m.type === "tableDelimiterRow" ? (c = n, a[1] === 0 ? o[1] !== 0 && (d = oC(f, t, o, i, n, d)) : (o[0] = o[1], d = oC(f, t, a, i, n, d)), i = 0) : i && (m.type === "data" || m.type === "tableDelimiterMarker" || m.type === "tableDelimiterFiller") && (o[3] = n);
	}
	for (c !== 0 && sC(f, t, c, l, u), f.consume(t.events), n = -1; ++n < t.events.length;) {
		let e = t.events[n];
		e[0] === "enter" && e[1].type === "table" && (e[1]._align = nC(t.events, n));
	}
	return e;
}
function oC(e, t, n, r, i, a) {
	let o = r === 1 ? "tableHeader" : r === 2 ? "tableDelimiter" : "tableData";
	n[0] !== 0 && (a.end = Object.assign({}, cC(t.events, n[0])), e.add(n[0], 0, [[
		"exit",
		a,
		t
	]]));
	let s = cC(t.events, n[1]);
	if (a = {
		type: o,
		start: Object.assign({}, s),
		end: Object.assign({}, s)
	}, e.add(n[1], 0, [[
		"enter",
		a,
		t
	]]), n[2] !== 0) {
		let i = cC(t.events, n[2]), a = cC(t.events, n[3]), o = {
			type: "tableContent",
			start: Object.assign({}, i),
			end: Object.assign({}, a)
		};
		if (e.add(n[2], 0, [[
			"enter",
			o,
			t
		]]), r !== 2) {
			let r = t.events[n[2]], i = t.events[n[3]];
			if (r[1].end = Object.assign({}, i[1].end), r[1].type = "chunkText", r[1].contentType = "text", n[3] > n[2] + 1) {
				let t = n[2] + 1, r = n[3] - n[2] - 1;
				e.add(t, r, []);
			}
		}
		e.add(n[3] + 1, 0, [[
			"exit",
			o,
			t
		]]);
	}
	return i !== void 0 && (a.end = Object.assign({}, cC(t.events, i)), e.add(i, 0, [[
		"exit",
		a,
		t
	]]), a = void 0), a;
}
function sC(e, t, n, r, i) {
	let a = [], o = cC(t.events, n);
	i && (i.end = Object.assign({}, o), a.push([
		"exit",
		i,
		t
	])), r.end = Object.assign({}, o), a.push([
		"exit",
		r,
		t
	]), e.add(n + 1, 0, a);
}
function cC(e, t) {
	let n = e[t], r = n[0] === "enter" ? "start" : "end";
	return n[1][r];
}
//#endregion
//#region node_modules/.pnpm/micromark-extension-gfm-task-list-item@2.1.0/node_modules/micromark-extension-gfm-task-list-item/lib/syntax.js
var lC = {
	name: "tasklistCheck",
	tokenize: dC
};
function uC() {
	return { text: { 91: lC } };
}
function dC(e, t, n) {
	let r = this;
	return i;
	function i(t) {
		return r.previous !== null || !r._gfmTasklistFirstContentOfListItem ? n(t) : (e.enter("taskListCheck"), e.enter("taskListCheckMarker"), e.consume(t), e.exit("taskListCheckMarker"), a);
	}
	function a(t) {
		return k(t) ? (e.enter("taskListCheckValueUnchecked"), e.consume(t), e.exit("taskListCheckValueUnchecked"), o) : t === 88 || t === 120 ? (e.enter("taskListCheckValueChecked"), e.consume(t), e.exit("taskListCheckValueChecked"), o) : n(t);
	}
	function o(t) {
		return t === 93 ? (e.enter("taskListCheckMarker"), e.consume(t), e.exit("taskListCheckMarker"), e.exit("taskListCheck"), s) : n(t);
	}
	function s(r) {
		return O(r) ? t(r) : A(r) ? e.check({ tokenize: fC }, t, n)(r) : n(r);
	}
}
function fC(e, t, n) {
	return j(e, r, "whitespace");
	function r(e) {
		return e === null ? n(e) : t(e);
	}
}
//#endregion
//#region node_modules/.pnpm/micromark-extension-gfm@3.0.0/node_modules/micromark-extension-gfm/index.js
function pC(e) {
	return Ie([
		kS(),
		GS(),
		$S(e),
		rC(),
		uC()
	]);
}
//#endregion
//#region node_modules/.pnpm/remark-gfm@4.0.1/node_modules/remark-gfm/lib/index.js
var mC = {};
function hC(e) {
	let t = this, n = e || mC, r = t.data(), i = r.micromarkExtensions ||= [], a = r.fromMarkdownExtensions ||= [], o = r.toMarkdownExtensions ||= [];
	i.push(pC(n)), a.push(vS()), o.push(yS(n));
}
//#endregion
//#region node_modules/.pnpm/@milkdown+preset-gfm@7.22.2/node_modules/@milkdown/preset-gfm/lib/index.js
function $(e, t) {
	return Object.assign(e, { meta: {
		package: "@milkdown/preset-gfm",
		...t
	} }), e;
}
var gC = Ug("strike_through");
$(gC, {
	displayName: "Attr<strikethrough>",
	group: "Strikethrough"
});
var _C = Bg("strike_through", (e) => ({
	parseDOM: [{ tag: "del" }, {
		style: "text-decoration",
		getAttrs: (e) => e === "line-through"
	}],
	toDOM: (t) => ["del", e.get(gC.key)(t)],
	parseMarkdown: {
		match: (e) => e.type === "delete",
		runner: (e, t, n) => {
			e.openMark(n), e.next(t.children), e.closeMark(n);
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "strike_through",
		runner: (e, t) => {
			e.withMark(t, "delete");
		}
	}
}));
$(_C.mark, {
	displayName: "MarkSchema<strikethrough>",
	group: "Strikethrough"
}), $(_C.ctx, {
	displayName: "MarkSchemaCtx<strikethrough>",
	group: "Strikethrough"
});
var vC = J("ToggleStrikeThrough", (e) => () => Jl(_C.type(e)));
$(vC, {
	displayName: "Command<ToggleStrikethrough>",
	group: "Strikethrough"
});
var yC = Ng((e) => Su(/(?<![\w:/])(~{1,2})(.+?)\1(?!\w|\/)$/, _C.type(e)));
$(yC, {
	displayName: "InputRule<strikethrough>",
	group: "Strikethrough"
});
var bC = Vg("strikeThroughKeymap", { ToggleStrikethrough: {
	shortcuts: "Mod-Alt-x",
	command: (e) => {
		let t = e.get(q);
		return () => t.call(vC.key);
	}
} });
$(bC.ctx, {
	displayName: "KeymapCtx<strikethrough>",
	group: "Strikethrough"
}), $(bC.shortcuts, {
	displayName: "Keymap<strikethrough>",
	group: "Strikethrough"
});
function xC(e, t) {
	let n = t.content.firstChild;
	if (t.childCount !== 1 || n?.type.name !== "paragraph") {
		e.next(t.content);
		return;
	}
	let r = n.lastChild, i = r?.type.name === "hardbreak" ? n.content.cut(0, n.content.size - r.nodeSize) : n.content;
	e.next(i);
}
var SC = hy({
	tableGroup: "block",
	cellContent: "paragraph",
	cellAttributes: { alignment: {
		default: "left",
		getFromDOM: (e) => e.style.textAlign || "left",
		setDOMAttr: (e, t) => {
			t.style = `text-align: ${e || "left"}`;
		}
	} }
}), CC = Y("table", () => ({
	...SC.table,
	content: "table_header_row table_row+",
	disableDropCursor: !0,
	parseMarkdown: {
		match: (e) => e.type === "table",
		runner: (e, t, n) => {
			let r = t.align, i = t.children.map((e, t) => ({
				...e,
				align: r,
				isHeader: t === 0
			}));
			e.openNode(n), e.next(i), e.closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "table",
		runner: (e, t) => {
			let n = t.content.firstChild?.content;
			if (!n) return;
			let r = [];
			n.forEach((e) => {
				r.push(e.attrs.alignment);
			}), e.openNode("table", void 0, { align: r }), e.next(t.content), e.closeNode();
		}
	}
}));
$(CC.node, {
	displayName: "NodeSchema<table>",
	group: "Table"
}), $(CC.ctx, {
	displayName: "NodeSchemaCtx<table>",
	group: "Table"
});
var wC = Y("table_header_row", () => ({
	...SC.table_row,
	disableDropCursor: !0,
	content: "(table_header)*",
	parseDOM: [{ tag: "tr[data-is-header]" }, {
		tag: "tr",
		getAttrs: (e) => e instanceof HTMLElement && e.querySelector("th") ? {} : !1
	}],
	toDOM() {
		return [
			"tr",
			{ "data-is-header": !0 },
			0
		];
	},
	parseMarkdown: {
		match: (e) => !!(e.type === "tableRow" && e.isHeader),
		runner: (e, t, n) => {
			let r = t.align, i = t.children.map((e, n) => ({
				...e,
				align: r[n],
				isHeader: t.isHeader
			}));
			e.openNode(n), e.next(i), e.closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "table_header_row",
		runner: (e, t) => {
			t.content.size !== 0 && (e.openNode("tableRow", void 0, { isHeader: !0 }), e.next(t.content), e.closeNode());
		}
	}
}));
$(wC.node, {
	displayName: "NodeSchema<tableHeaderRow>",
	group: "Table"
}), $(wC.ctx, {
	displayName: "NodeSchemaCtx<tableHeaderRow>",
	group: "Table"
});
var TC = Y("table_row", () => ({
	...SC.table_row,
	disableDropCursor: !0,
	content: "(table_cell)*",
	parseMarkdown: {
		match: (e) => e.type === "tableRow",
		runner: (e, t, n) => {
			let r = t.align, i = t.children.map((e, t) => ({
				...e,
				align: r[t]
			}));
			e.openNode(n), e.next(i), e.closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "table_row",
		runner: (e, t) => {
			t.content.size !== 0 && (e.openNode("tableRow"), e.next(t.content), e.closeNode());
		}
	}
}));
$(TC.node, {
	displayName: "NodeSchema<tableRow>",
	group: "Table"
}), $(TC.ctx, {
	displayName: "NodeSchemaCtx<tableRow>",
	group: "Table"
});
var EC = Y("table_cell", () => ({
	...SC.table_cell,
	disableDropCursor: !0,
	parseMarkdown: {
		match: (e) => e.type === "tableCell" && !e.isHeader,
		runner: (e, t, n) => {
			let r = t.align;
			e.openNode(n, { alignment: r }).openNode(e.schema.nodes.paragraph).next(t.children).closeNode().closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "table_cell",
		runner: (e, t) => {
			e.openNode("tableCell"), xC(e, t), e.closeNode();
		}
	}
}));
$(EC.node, {
	displayName: "NodeSchema<tableCell>",
	group: "Table"
}), $(EC.ctx, {
	displayName: "NodeSchemaCtx<tableCell>",
	group: "Table"
});
var DC = Y("table_header", () => ({
	...SC.table_header,
	disableDropCursor: !0,
	parseMarkdown: {
		match: (e) => e.type === "tableCell" && !!e.isHeader,
		runner: (e, t, n) => {
			let r = t.align;
			e.openNode(n, { alignment: r }), e.openNode(e.schema.nodes.paragraph), e.next(t.children), e.closeNode(), e.closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === "table_header",
		runner: (e, t) => {
			e.openNode("tableCell"), xC(e, t), e.closeNode();
		}
	}
}));
$(DC.node, {
	displayName: "NodeSchema<tableHeader>",
	group: "Table"
}), $(DC.ctx, {
	displayName: "NodeSchemaCtx<tableHeader>",
	group: "Table"
});
function OC(e, t = 3, n = 3) {
	let r = Array(n).fill(0).map(() => EC.type(e).createAndFill()), i = Array(n).fill(0).map(() => DC.type(e).createAndFill()), a = Array(t).fill(0).map((t, n) => n === 0 ? wC.type(e).create(null, i) : TC.type(e).create(null, r));
	return CC.type(e).create(null, a);
}
function kC(e) {
	return (t, n) => (r) => {
		n ??= r.selection.from;
		let i = r.doc.resolve(n), a = Du((e) => e.type.name === "table")(i), o = a ? {
			node: a.node,
			from: a.start
		} : void 0, s = e === "row";
		if (o) {
			let e = Z.get(o.node);
			if (t >= 0 && t < (s ? e.height : e.width)) {
				let n = e.positionAt(s ? t : e.height - 1, s ? e.width - 1 : t, o.node), i = r.doc.resolve(o.from + n), a = s ? Q.rowSelection : Q.colSelection, c = e.positionAt(s ? t : 0, s ? 0 : t, o.node), l = r.doc.resolve(o.from + c);
				return Cu(r.setSelection(a(i, l)));
			}
		}
		return r;
	};
}
var AC = kC("row"), jC = kC("col");
function MC(e, t, { map: n, tableStart: r, table: i }, a) {
	let o = Array(a).fill(0).reduce((e, t, n) => e + i.child(n).nodeSize, r), s = Array(n.width).fill(0).map((t, r) => {
		let a = i.nodeAt(n.map[r]);
		return EC.type(e).createAndFill({ alignment: a?.attrs.alignment });
	});
	return t.insert(o, TC.type(e).create(null, s)), t;
}
function NC(e) {
	let t = Vy(e.$from);
	if (!t) return;
	let n = Z.get(t.node);
	return n.cellsInRect({
		left: 0,
		right: n.width,
		top: 0,
		bottom: n.height
	}).map((e) => {
		let n = t.node.nodeAt(e), r = e + t.start;
		return {
			pos: r,
			start: r + 1,
			node: n
		};
	});
}
function PC(e) {
	let t = NC(e.selection);
	if (t && t[0]) {
		let n = e.doc.resolve(t[0].pos), r = t[t.length - 1];
		if (r) {
			let t = e.doc.resolve(r.pos);
			return Cu(e.setSelection(new Q(t, n)));
		}
	}
	return e;
}
var FC = J("GoToPrevTableCell", () => () => db(-1));
$(FC, {
	displayName: "Command<goToPrevTableCellCommand>",
	group: "Table"
});
var IC = J("GoToNextTableCell", () => () => db(1));
$(IC, {
	displayName: "Command<goToNextTableCellCommand>",
	group: "Table"
});
var LC = J("ExitTable", (e) => () => (t, n) => {
	if (!yy(t)) return !1;
	let { $head: r } = t.selection, i = Eu(r, CC.type(e));
	if (!i) return !1;
	let { to: a } = i, o = t.tr.replaceWith(a, a, R_.type(e).createAndFill());
	return o.setSelection(L.near(o.doc.resolve(a), 1)).scrollIntoView(), n?.(o), !0;
});
$(LC, {
	displayName: "Command<breakTableCommand>",
	group: "Table"
});
var RC = J("InsertTable", (e) => ({ row: t, col: n } = {}) => (r, i) => {
	let { selection: a, tr: o } = r, { from: s } = a, c = OC(e, t, n), l = o.replaceSelectionWith(c), u = L.findFrom(l.doc.resolve(s), 1, !0);
	return u && l.setSelection(u), i?.(l), !0;
});
$(RC, {
	displayName: "Command<insertTableCommand>",
	group: "Table"
});
var zC = J("MoveRow", () => ({ from: e, to: t, pos: n } = {}) => mb({
	from: e ?? 0,
	to: t ?? 0,
	pos: n
}));
$(zC, {
	displayName: "Command<moveRowCommand>",
	group: "Table"
});
var BC = J("MoveCol", () => ({ from: e, to: t, pos: n } = {}) => hb({
	from: e ?? 0,
	to: t ?? 0,
	pos: n
}));
$(BC, {
	displayName: "Command<moveColCommand>",
	group: "Table"
});
var VC = J("SelectRow", () => (e = { index: 0 }) => (t, n) => {
	let { tr: r } = t;
	return !!n?.(AC(e.index, e.pos)(r));
});
$(VC, {
	displayName: "Command<selectRowCommand>",
	group: "Table"
});
var HC = J("SelectCol", () => (e = { index: 0 }) => (t, n) => {
	let { tr: r } = t;
	return !!n?.(jC(e.index, e.pos)(r));
});
$(HC, {
	displayName: "Command<selectColCommand>",
	group: "Table"
});
var UC = J("SelectTable", () => () => (e, t) => {
	let { tr: n } = e;
	return !!t?.(PC(n));
});
$(UC, {
	displayName: "Command<selectTableCommand>",
	group: "Table"
});
var WC = J("DeleteSelectedCells", () => () => (e, t) => {
	let { selection: n } = e;
	if (!(n instanceof Q)) return !1;
	let r = n.isRowSelection(), i = n.isColSelection();
	return r && i ? fb(e, t) : i ? rb(e, t) : ab(e, t);
});
$(WC, {
	displayName: "Command<deleteSelectedCellsCommand>",
	group: "Table"
});
var GC = J("AddColBefore", () => () => eb);
$(GC, {
	displayName: "Command<addColBeforeCommand>",
	group: "Table"
});
var KC = J("AddColAfter", () => () => tb);
$(KC, {
	displayName: "Command<addColAfterCommand>",
	group: "Table"
});
var qC = J("AddRowBefore", (e) => () => (t, n) => {
	if (!yy(t)) return !1;
	if (n) {
		let r = Qy(t);
		n(MC(e, t.tr, r, r.top));
	}
	return !0;
});
$(qC, {
	displayName: "Command<addRowBeforeCommand>",
	group: "Table"
});
var JC = J("AddRowAfter", (e) => () => (t, n) => {
	if (!yy(t)) return !1;
	if (n) {
		let r = Qy(t);
		n(MC(e, t.tr, r, r.bottom));
	}
	return !0;
});
$(JC, {
	displayName: "Command<addRowAfterCommand>",
	group: "Table"
});
var YC = J("SetAlign", () => (e = "left") => ob("alignment", e));
$(YC, {
	displayName: "Command<setAlignCommand>",
	group: "Table"
});
var XC = Ng((e) => new tu(/^\|(?<col>\d+)[xX](?<row>\d+)\|\s$/, (t, n, r, i) => {
	let a = t.doc.resolve(r);
	if (!a.node(-1).canReplaceWith(a.index(-1), a.indexAfter(-1), CC.type(e))) return null;
	let o = OC(e, Math.max(Number(n.groups?.row ?? 0), 2), Number(n.groups?.col)), s = t.tr.replaceRangeWith(r, i, o);
	return s.setSelection(R.create(s.doc, r + 3)).scrollIntoView();
}));
$(XC, {
	displayName: "InputRule<insertTableInputRule>",
	group: "Table"
});
var ZC = Pg((e) => ({ run: (t, n, r) => {
	if (r) return t;
	function i(t) {
		let n = t.childCount, r = t.lastChild?.childCount ?? 0;
		if (n === 0 || r === 0) return R_.type(e).create();
		let i = t.firstChild;
		if (!(r > 0 && i && i.childCount === 0)) return t;
		if (n >= 3) {
			let r = t.child(1), a = [];
			for (let t = 0; t < r.childCount; t++) {
				let n = r.child(t);
				a.push(DC.type(e).create(n.attrs, n.content, n.marks));
			}
			let o = i.type.create(i.attrs, a), s = [];
			for (let e = 2; e < n; e++) s.push(t.child(e));
			return t.type.create(t.attrs, [o, ...s]);
		}
		let a = Array(r).fill(0).map(() => DC.type(e).createAndFill()), o = new F(N.from(a), 0, 0), s = i.replace(0, 0, o);
		return t.replace(0, i.nodeSize, new F(N.from(s), 0, 0));
	}
	function a(t) {
		let n = TC.type(e), r = [], a = [], o = !1;
		function s() {
			if (a.length === 0) return;
			let t = wC.type(e).createAndFill(), n = CC.type(e).create(null, [t, ...a]);
			r.push(i(n)), a = [];
		}
		return t.forEach((e) => {
			e.type === n ? (o = !0, a.push(e)) : (s(), r.push(e));
		}), s(), o ? N.from(r) : t;
	}
	function o(t) {
		let n = a(t), r = n !== t, s = [];
		return n.forEach((t) => {
			if (t.type === CC.type(e)) {
				let e = i(t);
				e !== t && (r = !0), s.push(e);
			} else if (t.childCount > 0) {
				let e = o(t.content);
				e === t.content ? s.push(t) : (r = !0, s.push(t.copy(e)));
			} else s.push(t);
		}), r ? N.from(s) : t;
	}
	function s(t) {
		let n = [], r = [];
		t.forEach((e) => r.push(e));
		for (let t = 0; t < r.length; t++) {
			let i = r[t], a = r[t + 1];
			i.type === R_.type(e) && i.content.size === 0 && a && a.type === CC.type(e) || n.push(i);
		}
		return n.length < r.length ? N.from(n) : t;
	}
	let c = o(t.content);
	return c = s(c), new F(N.from(c), t.openStart, t.openEnd);
} }));
$(ZC, {
	displayName: "PasteRule<table>",
	group: "Table"
});
var QC = Vg("tableKeymap", {
	NextCell: {
		priority: 100,
		shortcuts: ["Mod-]", "Tab"],
		command: (e) => {
			let t = e.get(q);
			return () => t.call(IC.key);
		}
	},
	PrevCell: {
		shortcuts: ["Mod-[", "Shift-Tab"],
		command: (e) => {
			let t = e.get(q);
			return () => t.call(FC.key);
		}
	},
	ExitTable: {
		shortcuts: ["Mod-Enter", "Enter"],
		command: (e) => {
			let t = e.get(q);
			return () => t.call(LC.key);
		}
	}
});
$(QC.ctx, {
	displayName: "KeymapCtx<table>",
	group: "Table"
}), $(QC.shortcuts, {
	displayName: "Keymap<table>",
	group: "Table"
});
var $C = "footnote_definition", ew = "footnoteDefinition", tw = Y("footnote_definition", () => ({
	group: "block",
	content: "block+",
	defining: !0,
	attrs: { label: {
		default: "",
		validate: "string"
	} },
	parseDOM: [{
		tag: `dl[data-type="${$C}"]`,
		getAttrs: (e) => {
			if (!(e instanceof HTMLElement)) throw pe(e);
			return { label: e.dataset.label };
		},
		contentElement: "dd"
	}],
	toDOM: (e) => {
		let t = e.attrs.label;
		return [
			"dl",
			{
				"data-label": t,
				"data-type": $C
			},
			["dt", t],
			["dd", 0]
		];
	},
	parseMarkdown: {
		match: ({ type: e }) => e === ew,
		runner: (e, t, n) => {
			e.openNode(n, { label: t.label }).next(t.children).closeNode();
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === $C,
		runner: (e, t) => {
			e.openNode(ew, void 0, {
				label: t.attrs.label,
				identifier: t.attrs.label
			}).next(t.content).closeNode();
		}
	}
}));
$(tw.ctx, {
	displayName: "NodeSchemaCtx<footnodeDef>",
	group: "footnote"
}), $(tw.node, {
	displayName: "NodeSchema<footnodeDef>",
	group: "footnote"
});
var nw = "footnote_reference", rw = Y("footnote_reference", () => ({
	group: "inline",
	inline: !0,
	atom: !0,
	attrs: { label: {
		default: "",
		validate: "string"
	} },
	parseDOM: [{
		tag: `sup[data-type="${nw}"]`,
		getAttrs: (e) => {
			if (!(e instanceof HTMLElement)) throw pe(e);
			return { label: e.dataset.label };
		}
	}],
	toDOM: (e) => {
		let t = e.attrs.label;
		return [
			"sup",
			{
				"data-label": t,
				"data-type": nw
			},
			t
		];
	},
	parseMarkdown: {
		match: ({ type: e }) => e === "footnoteReference",
		runner: (e, t, n) => {
			e.addNode(n, { label: t.label });
		}
	},
	toMarkdown: {
		match: (e) => e.type.name === nw,
		runner: (e, t) => {
			e.addNode("footnoteReference", void 0, void 0, {
				label: t.attrs.label,
				identifier: t.attrs.label
			});
		}
	}
}));
$(rw.ctx, {
	displayName: "NodeSchemaCtx<footnodeRef>",
	group: "footnote"
}), $(rw.node, {
	displayName: "NodeSchema<footnodeRef>",
	group: "footnote"
});
var iw = Av.extendSchema((e) => (t) => {
	let n = e(t);
	return {
		...n,
		attrs: {
			...n.attrs,
			checked: {
				default: null,
				validate: "boolean|null"
			}
		},
		parseDOM: [{
			tag: "li[data-item-type=\"task\"]",
			getAttrs: (e) => {
				if (!(e instanceof HTMLElement)) throw pe(e);
				return {
					label: e.dataset.label,
					listType: e.dataset.listType,
					spread: e.dataset.spread === "true",
					checked: e.dataset.checked ? e.dataset.checked === "true" : null
				};
			}
		}, ...n?.parseDOM || []],
		toDOM: (e) => n.toDOM && e.attrs.checked == null ? n.toDOM(e) : [
			"li",
			{
				"data-item-type": "task",
				"data-label": e.attrs.label,
				"data-list-type": e.attrs.listType,
				"data-spread": e.attrs.spread,
				"data-checked": e.attrs.checked
			},
			0
		],
		parseMarkdown: {
			match: ({ type: e }) => e === "listItem",
			runner: (e, t, r) => {
				if (t.checked == null) {
					n.parseMarkdown.runner(e, t, r);
					return;
				}
				let i = t.label == null ? "•" : `${t.label}.`, a = t.checked == null ? null : !!t.checked, o = t.label == null ? "bullet" : "ordered", s = t.spread ?? !0;
				e.openNode(r, {
					label: i,
					listType: o,
					spread: s,
					checked: a
				}), e.next(t.children), e.closeNode();
			}
		},
		toMarkdown: {
			match: (e) => e.type.name === "list_item",
			runner: (e, t) => {
				if (t.attrs.checked == null) {
					n.toMarkdown.runner(e, t);
					return;
				}
				let r = t.attrs.label, i = t.attrs.listType, a = t.attrs.spread, o = t.attrs.checked;
				e.openNode("listItem", void 0, {
					label: r,
					listType: i,
					spread: a,
					checked: o
				}), e.next(t.content), e.closeNode();
			}
		}
	};
});
$(iw.node, {
	displayName: "NodeSchema<taskListItem>",
	group: "ListItem"
}), $(iw.ctx, {
	displayName: "NodeSchemaCtx<taskListItem>",
	group: "ListItem"
});
var aw = Ng(() => new tu(/^\[(?<checked>\s|x)\]\s$/, (e, t, n, r) => {
	let i = e.doc.resolve(n), a = 0, o = i.node(a);
	for (; o && o.type.name !== "list_item";) a--, o = i.node(a);
	if (!o || o.attrs.checked != null) return null;
	let s = t.groups?.checked === "x", c = i.before(a), l = e.tr;
	return l.deleteRange(n, r).setNodeMarkup(c, void 0, {
		...o.attrs,
		checked: s
	}), l;
}));
$(aw, {
	displayName: "InputRule<wrapInTaskListInputRule>",
	group: "ListItem"
});
var ow = [bC, QC].flat(), sw = [XC, aw], cw = [yC], lw = [ZC], uw = Lg(() => lx);
$(uw, {
	displayName: "Prose<autoInsertSpanPlugin>",
	group: "Prose"
}), $(Lg(() => Lb({})), {
	displayName: "Prose<columnResizingPlugin>",
	group: "Prose"
});
var dw = Lg(() => Zb({ allowTableNodeSelection: !0 }));
$(dw, {
	displayName: "Prose<tableEditingPlugin>",
	group: "Prose"
});
var fw = Wg("remarkGFM", () => hC);
$(fw.plugin, {
	displayName: "Remark<remarkGFMPlugin>",
	group: "Remark"
}), $(fw.options, {
	displayName: "RemarkConfig<remarkGFMPlugin>",
	group: "Remark"
});
var pw = new vl("MILKDOWN_KEEP_TABLE_ALIGN_PLUGIN");
function mw(e, t) {
	let n = 0;
	return t.forEach((t, r, i) => {
		t === e && (n = i);
	}), n;
}
function hw(e, t) {
	let n = e.content.findDiffStart(t.content);
	if (n == null) return null;
	let r = e.content.findDiffEnd(t.content);
	return r ? {
		from: n,
		to: Math.max(r.b, n)
	} : null;
}
var gw = Lg(() => new hl({
	key: pw,
	appendTransaction: (e, t, n) => {
		if (t.doc === n.doc) return;
		let r = hw(t.doc, n.doc);
		if (!r) return;
		let i, a = (e, t) => {
			if (e.type.name !== "table_cell") return;
			let r = n.doc.resolve(t), a = r.node(r.depth), o = r.node(r.depth - 1).firstChild;
			if (!o) return;
			let s = mw(e, a), c = o.maybeChild(s);
			if (!c) return;
			let l = c.attrs.alignment;
			l !== e.attrs.alignment && (i ||= n.tr, i.setNodeMarkup(t, void 0, {
				...e.attrs,
				alignment: l
			}));
		};
		return n.doc.nodesBetween(r.from, r.to, (e, t) => e.type.name !== "table" || (n.doc.nodesBetween(t, t + e.nodeSize, a), !1)), i;
	}
}));
$(gw, {
	displayName: "Prose<keepTableAlignPlugin>",
	group: "Prose"
});
var _w = [
	gw,
	uw,
	fw,
	dw
].flat(), vw = [
	[
		iw,
		CC,
		wC,
		TC,
		DC,
		EC,
		tw,
		rw,
		gC,
		_C
	].flat(),
	sw,
	lw,
	cw,
	ow,
	[
		IC,
		FC,
		LC,
		RC,
		zC,
		BC,
		VC,
		HC,
		UC,
		WC,
		qC,
		JC,
		GC,
		KC,
		YC,
		vC
	],
	_w
].flat(), yw = typeof global == "object" && global && global.Object === Object && global, bw = typeof self == "object" && self && self.Object === Object && self, xw = yw || bw || Function("return this")(), Sw = xw.Symbol, Cw = Object.prototype, ww = Cw.hasOwnProperty, Tw = Cw.toString, Ew = Sw ? Sw.toStringTag : void 0;
function Dw(e) {
	var t = ww.call(e, Ew), n = e[Ew];
	try {
		e[Ew] = void 0;
		var r = !0;
	} catch {}
	var i = Tw.call(e);
	return r && (t ? e[Ew] = n : delete e[Ew]), i;
}
//#endregion
//#region node_modules/.pnpm/lodash-es@4.18.1/node_modules/lodash-es/_objectToString.js
var Ow = Object.prototype.toString;
function kw(e) {
	return Ow.call(e);
}
//#endregion
//#region node_modules/.pnpm/lodash-es@4.18.1/node_modules/lodash-es/_baseGetTag.js
var Aw = "[object Null]", jw = "[object Undefined]", Mw = Sw ? Sw.toStringTag : void 0;
function Nw(e) {
	return e == null ? e === void 0 ? jw : Aw : Mw && Mw in Object(e) ? Dw(e) : kw(e);
}
//#endregion
//#region node_modules/.pnpm/lodash-es@4.18.1/node_modules/lodash-es/isObjectLike.js
function Pw(e) {
	return typeof e == "object" && !!e;
}
//#endregion
//#region node_modules/.pnpm/lodash-es@4.18.1/node_modules/lodash-es/isSymbol.js
var Fw = "[object Symbol]";
function Iw(e) {
	return typeof e == "symbol" || Pw(e) && Nw(e) == Fw;
}
//#endregion
//#region node_modules/.pnpm/lodash-es@4.18.1/node_modules/lodash-es/_trimmedEndIndex.js
var Lw = /\s/;
function Rw(e) {
	for (var t = e.length; t-- && Lw.test(e.charAt(t)););
	return t;
}
//#endregion
//#region node_modules/.pnpm/lodash-es@4.18.1/node_modules/lodash-es/_baseTrim.js
var zw = /^\s+/;
function Bw(e) {
	return e && e.slice(0, Rw(e) + 1).replace(zw, "");
}
//#endregion
//#region node_modules/.pnpm/lodash-es@4.18.1/node_modules/lodash-es/isObject.js
function Vw(e) {
	var t = typeof e;
	return e != null && (t == "object" || t == "function");
}
//#endregion
//#region node_modules/.pnpm/lodash-es@4.18.1/node_modules/lodash-es/toNumber.js
var Hw = NaN, Uw = /^[-+]0x[0-9a-f]+$/i, Ww = /^0b[01]+$/i, Gw = /^0o[0-7]+$/i, Kw = parseInt;
function qw(e) {
	if (typeof e == "number") return e;
	if (Iw(e)) return Hw;
	if (Vw(e)) {
		var t = typeof e.valueOf == "function" ? e.valueOf() : e;
		e = Vw(t) ? t + "" : t;
	}
	if (typeof e != "string") return e === 0 ? e : +e;
	e = Bw(e);
	var n = Ww.test(e);
	return n || Gw.test(e) ? Kw(e.slice(2), n ? 2 : 8) : Uw.test(e) ? Hw : +e;
}
//#endregion
//#region node_modules/.pnpm/lodash-es@4.18.1/node_modules/lodash-es/now.js
var Jw = function() {
	return xw.Date.now();
}, Yw = "Expected a function", Xw = Math.max, Zw = Math.min;
function Qw(e, t, n) {
	var r, i, a, o, s, c, l = 0, u = !1, d = !1, f = !0;
	if (typeof e != "function") throw TypeError(Yw);
	t = qw(t) || 0, Vw(n) && (u = !!n.leading, d = "maxWait" in n, a = d ? Xw(qw(n.maxWait) || 0, t) : a, f = "trailing" in n ? !!n.trailing : f);
	function p(t) {
		var n = r, a = i;
		return r = i = void 0, l = t, o = e.apply(a, n), o;
	}
	function m(e) {
		return l = e, s = setTimeout(_, t), u ? p(e) : o;
	}
	function h(e) {
		var n = e - c, r = e - l, i = t - n;
		return d ? Zw(i, a - r) : i;
	}
	function g(e) {
		var n = e - c, r = e - l;
		return c === void 0 || n >= t || n < 0 || d && r >= a;
	}
	function _() {
		var e = Jw();
		if (g(e)) return v(e);
		s = setTimeout(_, h(e));
	}
	function v(e) {
		return s = void 0, f && r ? p(e) : (r = i = void 0, o);
	}
	function y() {
		s !== void 0 && clearTimeout(s), l = 0, r = c = i = s = void 0;
	}
	function b() {
		return s === void 0 ? o : v(Jw());
	}
	function x() {
		var e = Jw(), n = g(e);
		if (r = arguments, i = this, c = e, n) {
			if (s === void 0) return m(c);
			if (d) return clearTimeout(s), s = setTimeout(_, t), p(c);
		}
		return s === void 0 && (s = setTimeout(_, t)), o;
	}
	return x.cancel = y, x.flush = b, x;
}
//#endregion
//#region node_modules/.pnpm/@milkdown+plugin-listener@7.22.2/node_modules/@milkdown/plugin-listener/lib/index.js
var $w = class {
	constructor() {
		this.beforeMountedListeners = [], this.mountedListeners = [], this.updatedListeners = [], this.selectionUpdatedListeners = [], this.markdownUpdatedListeners = [], this.blurListeners = [], this.focusListeners = [], this.destroyListeners = [], this.beforeMount = (e) => (this.beforeMountedListeners.push(e), this), this.mounted = (e) => (this.mountedListeners.push(e), this), this.updated = (e) => (this.updatedListeners.push(e), this);
	}
	get listeners() {
		return {
			beforeMount: this.beforeMountedListeners,
			mounted: this.mountedListeners,
			updated: this.updatedListeners,
			markdownUpdated: this.markdownUpdatedListeners,
			blur: this.blurListeners,
			focus: this.focusListeners,
			destroy: this.destroyListeners,
			selectionUpdated: this.selectionUpdatedListeners
		};
	}
	markdownUpdated(e) {
		return this.markdownUpdatedListeners.push(e), this;
	}
	blur(e) {
		return this.blurListeners.push(e), this;
	}
	focus(e) {
		return this.focusListeners.push(e), this;
	}
	destroy(e) {
		return this.destroyListeners.push(e), this;
	}
	selectionUpdated(e) {
		return this.selectionUpdatedListeners.push(e), this;
	}
}, eT = E(new $w(), "listener"), tT = new vl("MILKDOWN_LISTENER"), nT = (e) => (e.inject(eT, new $w()), async () => {
	await e.wait(Mh);
	let { listeners: t } = e.get(eT);
	t.beforeMount.forEach((t) => t(e)), await e.wait(rg);
	let n = e.get(og), r = null, i = null, a = null, o = null, s = Qw(() => {
		if (!o) return;
		let { doc: a } = o;
		if (t.updated.length > 0 && r && !r.eq(a) && t.updated.forEach((t) => {
			t(e, a, r);
		}), t.markdownUpdated.length > 0 && r && !r.eq(a)) {
			let r = n(a);
			t.markdownUpdated.forEach((t) => {
				t(e, r, i);
			}), i = r;
		}
		r = a, o = null;
	}, 200), c = new hl({
		key: tT,
		view: () => ({ destroy: () => {
			s.cancel(), t.destroy.forEach((t) => t(e));
		} }),
		props: { handleDOMEvents: {
			focus: () => (t.focus.forEach((t) => t(e)), !1),
			blur: () => (t.blur.forEach((t) => t(e)), !1)
		} },
		state: {
			init: (e, t) => {
				r = t.doc, i = n(t.doc);
			},
			apply: (n) => {
				let r = n.selection;
				(!a && r || a && !r.eq(a)) && (t.selectionUpdated.forEach((t) => {
					t(e, r, a);
				}), a = r), (n.docChanged || n.storedMarksSet) && n.getMeta("addToHistory") !== !1 && (o = n, s());
			}
		}
	});
	e.update(wh, (e) => e.concat(c)), await e.wait(yg), t.mounted.forEach((t) => t(e));
});
nT.meta = {
	package: "@milkdown/plugin-listener",
	displayName: "Listener"
};
//#endregion
//#region node_modules/.pnpm/clsx@2.1.1/node_modules/clsx/dist/clsx.mjs
function rT(e) {
	var t, n, r = "";
	if (typeof e == "string" || typeof e == "number") r += e;
	else if (typeof e == "object") {
		if (Array.isArray(e)) {
			var i = e.length;
			for (t = 0; t < i; t++) e[t] && (n = rT(e[t])) && (r && (r += " "), r += n);
		} else for (n in e) e[n] && (r && (r += " "), r += n);
	}
	return r;
}
function iT() {
	for (var e, t, n = 0, r = "", i = arguments.length; n < i; n++) (e = arguments[n]) && (t = rT(e)) && (r && (r += " "), r += t);
	return r;
}
//#endregion
//#region node_modules/.pnpm/@milkdown+theme-nord@7.22.2/node_modules/@milkdown/theme-nord/lib/index.js
function aT(e) {
	e.update(xg, (e) => {
		let t = e.attributes;
		return {
			...e,
			attributes: (e) => {
				let n = typeof t == "function" ? t(e) : t;
				return {
					...n,
					class: iT("prose dark:prose-invert", n?.class || "", "milkdown-theme-nord")
				};
			}
		};
	});
}
//#endregion
export { kg as Editor, C as EditorWorker, ay as commonmark, cg as defaultValueCtx, yh as editorViewCtx, vw as gfm, nT as listener, eT as listenerCtx, y as monaco, aT as nord, Sg as rootCtx };
