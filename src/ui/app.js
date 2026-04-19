"use strict";
(() => {
  // node_modules/preact/dist/preact.module.js
  var n;
  var l;
  var u;
  var t;
  var i;
  var r;
  var o;
  var e;
  var f;
  var c;
  var s;
  var a;
  var h;
  var p;
  var v;
  var y;
  var d = {};
  var w = [];
  var _ = /acit|ex(?:s|g|n|p|$)|rph|grid|ows|mnc|ntw|ine[ch]|zoo|^ord|itera/i;
  var g = Array.isArray;
  function m(n3, l5) {
    for (var u5 in l5) n3[u5] = l5[u5];
    return n3;
  }
  function b(n3) {
    n3 && n3.parentNode && n3.parentNode.removeChild(n3);
  }
  function k(l5, u5, t4) {
    var i4, r4, o4, e4 = {};
    for (o4 in u5) "key" == o4 ? i4 = u5[o4] : "ref" == o4 ? r4 = u5[o4] : e4[o4] = u5[o4];
    if (arguments.length > 2 && (e4.children = arguments.length > 3 ? n.call(arguments, 2) : t4), "function" == typeof l5 && null != l5.defaultProps) for (o4 in l5.defaultProps) void 0 === e4[o4] && (e4[o4] = l5.defaultProps[o4]);
    return x(l5, e4, i4, r4, null);
  }
  function x(n3, t4, i4, r4, o4) {
    var e4 = { type: n3, props: t4, key: i4, ref: r4, __k: null, __: null, __b: 0, __e: null, __c: null, constructor: void 0, __v: null == o4 ? ++u : o4, __i: -1, __u: 0 };
    return null == o4 && null != l.vnode && l.vnode(e4), e4;
  }
  function S(n3) {
    return n3.children;
  }
  function C(n3, l5) {
    this.props = n3, this.context = l5;
  }
  function $(n3, l5) {
    if (null == l5) return n3.__ ? $(n3.__, n3.__i + 1) : null;
    for (var u5; l5 < n3.__k.length; l5++) if (null != (u5 = n3.__k[l5]) && null != u5.__e) return u5.__e;
    return "function" == typeof n3.type ? $(n3) : null;
  }
  function I(n3) {
    if (n3.__P && n3.__d) {
      var u5 = n3.__v, t4 = u5.__e, i4 = [], r4 = [], o4 = m({}, u5);
      o4.__v = u5.__v + 1, l.vnode && l.vnode(o4), q(n3.__P, o4, u5, n3.__n, n3.__P.namespaceURI, 32 & u5.__u ? [t4] : null, i4, null == t4 ? $(u5) : t4, !!(32 & u5.__u), r4), o4.__v = u5.__v, o4.__.__k[o4.__i] = o4, D(i4, o4, r4), u5.__e = u5.__ = null, o4.__e != t4 && P(o4);
    }
  }
  function P(n3) {
    if (null != (n3 = n3.__) && null != n3.__c) return n3.__e = n3.__c.base = null, n3.__k.some(function(l5) {
      if (null != l5 && null != l5.__e) return n3.__e = n3.__c.base = l5.__e;
    }), P(n3);
  }
  function A(n3) {
    (!n3.__d && (n3.__d = true) && i.push(n3) && !H.__r++ || r != l.debounceRendering) && ((r = l.debounceRendering) || o)(H);
  }
  function H() {
    try {
      for (var n3, l5 = 1; i.length; ) i.length > l5 && i.sort(e), n3 = i.shift(), l5 = i.length, I(n3);
    } finally {
      i.length = H.__r = 0;
    }
  }
  function L(n3, l5, u5, t4, i4, r4, o4, e4, f5, c4, s4) {
    var a4, h5, p5, v4, y5, _4, g4, m4 = t4 && t4.__k || w, b4 = l5.length;
    for (f5 = T(u5, l5, m4, f5, b4), a4 = 0; a4 < b4; a4++) null != (p5 = u5.__k[a4]) && (h5 = -1 != p5.__i && m4[p5.__i] || d, p5.__i = a4, _4 = q(n3, p5, h5, i4, r4, o4, e4, f5, c4, s4), v4 = p5.__e, p5.ref && h5.ref != p5.ref && (h5.ref && J(h5.ref, null, p5), s4.push(p5.ref, p5.__c || v4, p5)), null == y5 && null != v4 && (y5 = v4), (g4 = !!(4 & p5.__u)) || h5.__k === p5.__k ? (f5 = j(p5, f5, n3, g4), g4 && h5.__e && (h5.__e = null)) : "function" == typeof p5.type && void 0 !== _4 ? f5 = _4 : v4 && (f5 = v4.nextSibling), p5.__u &= -7);
    return u5.__e = y5, f5;
  }
  function T(n3, l5, u5, t4, i4) {
    var r4, o4, e4, f5, c4, s4 = u5.length, a4 = s4, h5 = 0;
    for (n3.__k = new Array(i4), r4 = 0; r4 < i4; r4++) null != (o4 = l5[r4]) && "boolean" != typeof o4 && "function" != typeof o4 ? ("string" == typeof o4 || "number" == typeof o4 || "bigint" == typeof o4 || o4.constructor == String ? o4 = n3.__k[r4] = x(null, o4, null, null, null) : g(o4) ? o4 = n3.__k[r4] = x(S, { children: o4 }, null, null, null) : void 0 === o4.constructor && o4.__b > 0 ? o4 = n3.__k[r4] = x(o4.type, o4.props, o4.key, o4.ref ? o4.ref : null, o4.__v) : n3.__k[r4] = o4, f5 = r4 + h5, o4.__ = n3, o4.__b = n3.__b + 1, e4 = null, -1 != (c4 = o4.__i = O(o4, u5, f5, a4)) && (a4--, (e4 = u5[c4]) && (e4.__u |= 2)), null == e4 || null == e4.__v ? (-1 == c4 && (i4 > s4 ? h5-- : i4 < s4 && h5++), "function" != typeof o4.type && (o4.__u |= 4)) : c4 != f5 && (c4 == f5 - 1 ? h5-- : c4 == f5 + 1 ? h5++ : (c4 > f5 ? h5-- : h5++, o4.__u |= 4))) : n3.__k[r4] = null;
    if (a4) for (r4 = 0; r4 < s4; r4++) null != (e4 = u5[r4]) && 0 == (2 & e4.__u) && (e4.__e == t4 && (t4 = $(e4)), K(e4, e4));
    return t4;
  }
  function j(n3, l5, u5, t4) {
    var i4, r4;
    if ("function" == typeof n3.type) {
      for (i4 = n3.__k, r4 = 0; i4 && r4 < i4.length; r4++) i4[r4] && (i4[r4].__ = n3, l5 = j(i4[r4], l5, u5, t4));
      return l5;
    }
    n3.__e != l5 && (t4 && (l5 && n3.type && !l5.parentNode && (l5 = $(n3)), u5.insertBefore(n3.__e, l5 || null)), l5 = n3.__e);
    do {
      l5 = l5 && l5.nextSibling;
    } while (null != l5 && 8 == l5.nodeType);
    return l5;
  }
  function O(n3, l5, u5, t4) {
    var i4, r4, o4, e4 = n3.key, f5 = n3.type, c4 = l5[u5], s4 = null != c4 && 0 == (2 & c4.__u);
    if (null === c4 && null == e4 || s4 && e4 == c4.key && f5 == c4.type) return u5;
    if (t4 > (s4 ? 1 : 0)) {
      for (i4 = u5 - 1, r4 = u5 + 1; i4 >= 0 || r4 < l5.length; ) if (null != (c4 = l5[o4 = i4 >= 0 ? i4-- : r4++]) && 0 == (2 & c4.__u) && e4 == c4.key && f5 == c4.type) return o4;
    }
    return -1;
  }
  function z(n3, l5, u5) {
    "-" == l5[0] ? n3.setProperty(l5, null == u5 ? "" : u5) : n3[l5] = null == u5 ? "" : "number" != typeof u5 || _.test(l5) ? u5 : u5 + "px";
  }
  function N(n3, l5, u5, t4, i4) {
    var r4, o4;
    n: if ("style" == l5) if ("string" == typeof u5) n3.style.cssText = u5;
    else {
      if ("string" == typeof t4 && (n3.style.cssText = t4 = ""), t4) for (l5 in t4) u5 && l5 in u5 || z(n3.style, l5, "");
      if (u5) for (l5 in u5) t4 && u5[l5] == t4[l5] || z(n3.style, l5, u5[l5]);
    }
    else if ("o" == l5[0] && "n" == l5[1]) r4 = l5 != (l5 = l5.replace(a, "$1")), o4 = l5.toLowerCase(), l5 = o4 in n3 || "onFocusOut" == l5 || "onFocusIn" == l5 ? o4.slice(2) : l5.slice(2), n3.l || (n3.l = {}), n3.l[l5 + r4] = u5, u5 ? t4 ? u5[s] = t4[s] : (u5[s] = h, n3.addEventListener(l5, r4 ? v : p, r4)) : n3.removeEventListener(l5, r4 ? v : p, r4);
    else {
      if ("http://www.w3.org/2000/svg" == i4) l5 = l5.replace(/xlink(H|:h)/, "h").replace(/sName$/, "s");
      else if ("width" != l5 && "height" != l5 && "href" != l5 && "list" != l5 && "form" != l5 && "tabIndex" != l5 && "download" != l5 && "rowSpan" != l5 && "colSpan" != l5 && "role" != l5 && "popover" != l5 && l5 in n3) try {
        n3[l5] = null == u5 ? "" : u5;
        break n;
      } catch (n4) {
      }
      "function" == typeof u5 || (null == u5 || false === u5 && "-" != l5[4] ? n3.removeAttribute(l5) : n3.setAttribute(l5, "popover" == l5 && 1 == u5 ? "" : u5));
    }
  }
  function V(n3) {
    return function(u5) {
      if (this.l) {
        var t4 = this.l[u5.type + n3];
        if (null == u5[c]) u5[c] = h++;
        else if (u5[c] < t4[s]) return;
        return t4(l.event ? l.event(u5) : u5);
      }
    };
  }
  function q(n3, u5, t4, i4, r4, o4, e4, f5, c4, s4) {
    var a4, h5, p5, v4, y5, d5, _4, k3, x4, M, $3, I2, P2, A3, H2, T4 = u5.type;
    if (void 0 !== u5.constructor) return null;
    128 & t4.__u && (c4 = !!(32 & t4.__u), o4 = [f5 = u5.__e = t4.__e]), (a4 = l.__b) && a4(u5);
    n: if ("function" == typeof T4) try {
      if (k3 = u5.props, x4 = T4.prototype && T4.prototype.render, M = (a4 = T4.contextType) && i4[a4.__c], $3 = a4 ? M ? M.props.value : a4.__ : i4, t4.__c ? _4 = (h5 = u5.__c = t4.__c).__ = h5.__E : (x4 ? u5.__c = h5 = new T4(k3, $3) : (u5.__c = h5 = new C(k3, $3), h5.constructor = T4, h5.render = Q), M && M.sub(h5), h5.state || (h5.state = {}), h5.__n = i4, p5 = h5.__d = true, h5.__h = [], h5._sb = []), x4 && null == h5.__s && (h5.__s = h5.state), x4 && null != T4.getDerivedStateFromProps && (h5.__s == h5.state && (h5.__s = m({}, h5.__s)), m(h5.__s, T4.getDerivedStateFromProps(k3, h5.__s))), v4 = h5.props, y5 = h5.state, h5.__v = u5, p5) x4 && null == T4.getDerivedStateFromProps && null != h5.componentWillMount && h5.componentWillMount(), x4 && null != h5.componentDidMount && h5.__h.push(h5.componentDidMount);
      else {
        if (x4 && null == T4.getDerivedStateFromProps && k3 !== v4 && null != h5.componentWillReceiveProps && h5.componentWillReceiveProps(k3, $3), u5.__v == t4.__v || !h5.__e && null != h5.shouldComponentUpdate && false === h5.shouldComponentUpdate(k3, h5.__s, $3)) {
          u5.__v != t4.__v && (h5.props = k3, h5.state = h5.__s, h5.__d = false), u5.__e = t4.__e, u5.__k = t4.__k, u5.__k.some(function(n4) {
            n4 && (n4.__ = u5);
          }), w.push.apply(h5.__h, h5._sb), h5._sb = [], h5.__h.length && e4.push(h5);
          break n;
        }
        null != h5.componentWillUpdate && h5.componentWillUpdate(k3, h5.__s, $3), x4 && null != h5.componentDidUpdate && h5.__h.push(function() {
          h5.componentDidUpdate(v4, y5, d5);
        });
      }
      if (h5.context = $3, h5.props = k3, h5.__P = n3, h5.__e = false, I2 = l.__r, P2 = 0, x4) h5.state = h5.__s, h5.__d = false, I2 && I2(u5), a4 = h5.render(h5.props, h5.state, h5.context), w.push.apply(h5.__h, h5._sb), h5._sb = [];
      else do {
        h5.__d = false, I2 && I2(u5), a4 = h5.render(h5.props, h5.state, h5.context), h5.state = h5.__s;
      } while (h5.__d && ++P2 < 25);
      h5.state = h5.__s, null != h5.getChildContext && (i4 = m(m({}, i4), h5.getChildContext())), x4 && !p5 && null != h5.getSnapshotBeforeUpdate && (d5 = h5.getSnapshotBeforeUpdate(v4, y5)), A3 = null != a4 && a4.type === S && null == a4.key ? E(a4.props.children) : a4, f5 = L(n3, g(A3) ? A3 : [A3], u5, t4, i4, r4, o4, e4, f5, c4, s4), h5.base = u5.__e, u5.__u &= -161, h5.__h.length && e4.push(h5), _4 && (h5.__E = h5.__ = null);
    } catch (n4) {
      if (u5.__v = null, c4 || null != o4) if (n4.then) {
        for (u5.__u |= c4 ? 160 : 128; f5 && 8 == f5.nodeType && f5.nextSibling; ) f5 = f5.nextSibling;
        o4[o4.indexOf(f5)] = null, u5.__e = f5;
      } else {
        for (H2 = o4.length; H2--; ) b(o4[H2]);
        B(u5);
      }
      else u5.__e = t4.__e, u5.__k = t4.__k, n4.then || B(u5);
      l.__e(n4, u5, t4);
    }
    else null == o4 && u5.__v == t4.__v ? (u5.__k = t4.__k, u5.__e = t4.__e) : f5 = u5.__e = G(t4.__e, u5, t4, i4, r4, o4, e4, c4, s4);
    return (a4 = l.diffed) && a4(u5), 128 & u5.__u ? void 0 : f5;
  }
  function B(n3) {
    n3 && (n3.__c && (n3.__c.__e = true), n3.__k && n3.__k.some(B));
  }
  function D(n3, u5, t4) {
    for (var i4 = 0; i4 < t4.length; i4++) J(t4[i4], t4[++i4], t4[++i4]);
    l.__c && l.__c(u5, n3), n3.some(function(u6) {
      try {
        n3 = u6.__h, u6.__h = [], n3.some(function(n4) {
          n4.call(u6);
        });
      } catch (n4) {
        l.__e(n4, u6.__v);
      }
    });
  }
  function E(n3) {
    return "object" != typeof n3 || null == n3 || n3.__b > 0 ? n3 : g(n3) ? n3.map(E) : m({}, n3);
  }
  function G(u5, t4, i4, r4, o4, e4, f5, c4, s4) {
    var a4, h5, p5, v4, y5, w5, _4, m4 = i4.props || d, k3 = t4.props, x4 = t4.type;
    if ("svg" == x4 ? o4 = "http://www.w3.org/2000/svg" : "math" == x4 ? o4 = "http://www.w3.org/1998/Math/MathML" : o4 || (o4 = "http://www.w3.org/1999/xhtml"), null != e4) {
      for (a4 = 0; a4 < e4.length; a4++) if ((y5 = e4[a4]) && "setAttribute" in y5 == !!x4 && (x4 ? y5.localName == x4 : 3 == y5.nodeType)) {
        u5 = y5, e4[a4] = null;
        break;
      }
    }
    if (null == u5) {
      if (null == x4) return document.createTextNode(k3);
      u5 = document.createElementNS(o4, x4, k3.is && k3), c4 && (l.__m && l.__m(t4, e4), c4 = false), e4 = null;
    }
    if (null == x4) m4 === k3 || c4 && u5.data == k3 || (u5.data = k3);
    else {
      if (e4 = e4 && n.call(u5.childNodes), !c4 && null != e4) for (m4 = {}, a4 = 0; a4 < u5.attributes.length; a4++) m4[(y5 = u5.attributes[a4]).name] = y5.value;
      for (a4 in m4) y5 = m4[a4], "dangerouslySetInnerHTML" == a4 ? p5 = y5 : "children" == a4 || a4 in k3 || "value" == a4 && "defaultValue" in k3 || "checked" == a4 && "defaultChecked" in k3 || N(u5, a4, null, y5, o4);
      for (a4 in k3) y5 = k3[a4], "children" == a4 ? v4 = y5 : "dangerouslySetInnerHTML" == a4 ? h5 = y5 : "value" == a4 ? w5 = y5 : "checked" == a4 ? _4 = y5 : c4 && "function" != typeof y5 || m4[a4] === y5 || N(u5, a4, y5, m4[a4], o4);
      if (h5) c4 || p5 && (h5.__html == p5.__html || h5.__html == u5.innerHTML) || (u5.innerHTML = h5.__html), t4.__k = [];
      else if (p5 && (u5.innerHTML = ""), L("template" == t4.type ? u5.content : u5, g(v4) ? v4 : [v4], t4, i4, r4, "foreignObject" == x4 ? "http://www.w3.org/1999/xhtml" : o4, e4, f5, e4 ? e4[0] : i4.__k && $(i4, 0), c4, s4), null != e4) for (a4 = e4.length; a4--; ) b(e4[a4]);
      c4 || (a4 = "value", "progress" == x4 && null == w5 ? u5.removeAttribute("value") : null != w5 && (w5 !== u5[a4] || "progress" == x4 && !w5 || "option" == x4 && w5 != m4[a4]) && N(u5, a4, w5, m4[a4], o4), a4 = "checked", null != _4 && _4 != u5[a4] && N(u5, a4, _4, m4[a4], o4));
    }
    return u5;
  }
  function J(n3, u5, t4) {
    try {
      if ("function" == typeof n3) {
        var i4 = "function" == typeof n3.__u;
        i4 && n3.__u(), i4 && null == u5 || (n3.__u = n3(u5));
      } else n3.current = u5;
    } catch (n4) {
      l.__e(n4, t4);
    }
  }
  function K(n3, u5, t4) {
    var i4, r4;
    if (l.unmount && l.unmount(n3), (i4 = n3.ref) && (i4.current && i4.current != n3.__e || J(i4, null, u5)), null != (i4 = n3.__c)) {
      if (i4.componentWillUnmount) try {
        i4.componentWillUnmount();
      } catch (n4) {
        l.__e(n4, u5);
      }
      i4.base = i4.__P = null;
    }
    if (i4 = n3.__k) for (r4 = 0; r4 < i4.length; r4++) i4[r4] && K(i4[r4], u5, t4 || "function" != typeof n3.type);
    t4 || b(n3.__e), n3.__c = n3.__ = n3.__e = void 0;
  }
  function Q(n3, l5, u5) {
    return this.constructor(n3, u5);
  }
  function R(u5, t4, i4) {
    var r4, o4, e4, f5;
    t4 == document && (t4 = document.documentElement), l.__ && l.__(u5, t4), o4 = (r4 = "function" == typeof i4) ? null : i4 && i4.__k || t4.__k, e4 = [], f5 = [], q(t4, u5 = (!r4 && i4 || t4).__k = k(S, null, [u5]), o4 || d, d, t4.namespaceURI, !r4 && i4 ? [i4] : o4 ? null : t4.firstChild ? n.call(t4.childNodes) : null, e4, !r4 && i4 ? i4 : o4 ? o4.__e : t4.firstChild, r4, f5), D(e4, u5, f5);
  }
  n = w.slice, l = { __e: function(n3, l5, u5, t4) {
    for (var i4, r4, o4; l5 = l5.__; ) if ((i4 = l5.__c) && !i4.__) try {
      if ((r4 = i4.constructor) && null != r4.getDerivedStateFromError && (i4.setState(r4.getDerivedStateFromError(n3)), o4 = i4.__d), null != i4.componentDidCatch && (i4.componentDidCatch(n3, t4 || {}), o4 = i4.__d), o4) return i4.__E = i4;
    } catch (l6) {
      n3 = l6;
    }
    throw n3;
  } }, u = 0, t = function(n3) {
    return null != n3 && void 0 === n3.constructor;
  }, C.prototype.setState = function(n3, l5) {
    var u5;
    u5 = null != this.__s && this.__s != this.state ? this.__s : this.__s = m({}, this.state), "function" == typeof n3 && (n3 = n3(m({}, u5), this.props)), n3 && m(u5, n3), null != n3 && this.__v && (l5 && this._sb.push(l5), A(this));
  }, C.prototype.forceUpdate = function(n3) {
    this.__v && (this.__e = true, n3 && this.__h.push(n3), A(this));
  }, C.prototype.render = S, i = [], o = "function" == typeof Promise ? Promise.prototype.then.bind(Promise.resolve()) : setTimeout, e = function(n3, l5) {
    return n3.__v.__b - l5.__v.__b;
  }, H.__r = 0, f = Math.random().toString(8), c = "__d" + f, s = "__a" + f, a = /(PointerCapture)$|Capture$/i, h = 0, p = V(false), v = V(true), y = 0;

  // node_modules/preact/jsx-runtime/dist/jsxRuntime.module.js
  var f2 = 0;
  function u2(e4, t4, n3, o4, i4, u5) {
    t4 || (t4 = {});
    var a4, c4, p5 = t4;
    if ("ref" in p5) for (c4 in p5 = {}, t4) "ref" == c4 ? a4 = t4[c4] : p5[c4] = t4[c4];
    var l5 = { type: e4, props: p5, key: n3, ref: a4, __k: null, __: null, __b: 0, __e: null, __c: null, constructor: void 0, __v: --f2, __i: -1, __u: 0, __source: i4, __self: u5 };
    if ("function" == typeof e4 && (a4 = e4.defaultProps)) for (c4 in a4) void 0 === p5[c4] && (p5[c4] = a4[c4]);
    return l.vnode && l.vnode(l5), l5;
  }

  // src/ui/components/Footer.tsx
  function Footer() {
    return /* @__PURE__ */ u2("footer", { children: /* @__PURE__ */ u2("div", { class: "footer-content", children: [
      /* @__PURE__ */ u2("p", { children: [
        "Cost estimates based on Anthropic and OpenAI API pricing (",
        /* @__PURE__ */ u2(
          "a",
          {
            href: "https://docs.anthropic.com/en/docs/about-claude/pricing",
            target: "_blank",
            rel: "noopener noreferrer",
            children: "Anthropic"
          }
        ),
        " ",
        "+",
        " ",
        /* @__PURE__ */ u2(
          "a",
          {
            href: "https://developers.openai.com/api/docs/pricing",
            target: "_blank",
            rel: "noopener noreferrer",
            children: "OpenAI"
          }
        ),
        "). Local dashboard totals are estimates, not subscriber billing statements."
      ] }),
      /* @__PURE__ */ u2("p", { children: [
        "GitHub:",
        " ",
        /* @__PURE__ */ u2(
          "a",
          {
            href: "https://github.com/po4yka/claude-usage-tracker",
            target: "_blank",
            rel: "noopener noreferrer",
            children: "po4yka/claude-usage-tracker"
          }
        ),
        " ",
        "\xB7 License: MIT"
      ] })
    ] }) });
  }

  // node_modules/preact/hooks/dist/hooks.module.js
  var t2;
  var r2;
  var u3;
  var i2;
  var o2 = 0;
  var f3 = [];
  var c2 = l;
  var e2 = c2.__b;
  var a2 = c2.__r;
  var v2 = c2.diffed;
  var l2 = c2.__c;
  var m2 = c2.unmount;
  var s2 = c2.__;
  function p2(n3, t4) {
    c2.__h && c2.__h(r2, n3, o2 || t4), o2 = 0;
    var u5 = r2.__H || (r2.__H = { __: [], __h: [] });
    return n3 >= u5.__.length && u5.__.push({}), u5.__[n3];
  }
  function d2(n3) {
    return o2 = 1, h2(D2, n3);
  }
  function h2(n3, u5, i4) {
    var o4 = p2(t2++, 2);
    if (o4.t = n3, !o4.__c && (o4.__ = [i4 ? i4(u5) : D2(void 0, u5), function(n4) {
      var t4 = o4.__N ? o4.__N[0] : o4.__[0], r4 = o4.t(t4, n4);
      t4 !== r4 && (o4.__N = [r4, o4.__[1]], o4.__c.setState({}));
    }], o4.__c = r2, !r2.__f)) {
      var f5 = function(n4, t4, r4) {
        if (!o4.__c.__H) return true;
        var u6 = o4.__c.__H.__.filter(function(n5) {
          return n5.__c;
        });
        if (u6.every(function(n5) {
          return !n5.__N;
        })) return !c4 || c4.call(this, n4, t4, r4);
        var i5 = o4.__c.props !== n4;
        return u6.some(function(n5) {
          if (n5.__N) {
            var t5 = n5.__[0];
            n5.__ = n5.__N, n5.__N = void 0, t5 !== n5.__[0] && (i5 = true);
          }
        }), c4 && c4.call(this, n4, t4, r4) || i5;
      };
      r2.__f = true;
      var c4 = r2.shouldComponentUpdate, e4 = r2.componentWillUpdate;
      r2.componentWillUpdate = function(n4, t4, r4) {
        if (this.__e) {
          var u6 = c4;
          c4 = void 0, f5(n4, t4, r4), c4 = u6;
        }
        e4 && e4.call(this, n4, t4, r4);
      }, r2.shouldComponentUpdate = f5;
    }
    return o4.__N || o4.__;
  }
  function y2(n3, u5) {
    var i4 = p2(t2++, 3);
    !c2.__s && C2(i4.__H, u5) && (i4.__ = n3, i4.u = u5, r2.__H.__h.push(i4));
  }
  function A2(n3) {
    return o2 = 5, T2(function() {
      return { current: n3 };
    }, []);
  }
  function T2(n3, r4) {
    var u5 = p2(t2++, 7);
    return C2(u5.__H, r4) && (u5.__ = n3(), u5.__H = r4, u5.__h = n3), u5.__;
  }
  function j2() {
    for (var n3; n3 = f3.shift(); ) {
      var t4 = n3.__H;
      if (n3.__P && t4) try {
        t4.__h.some(z2), t4.__h.some(B2), t4.__h = [];
      } catch (r4) {
        t4.__h = [], c2.__e(r4, n3.__v);
      }
    }
  }
  c2.__b = function(n3) {
    r2 = null, e2 && e2(n3);
  }, c2.__ = function(n3, t4) {
    n3 && t4.__k && t4.__k.__m && (n3.__m = t4.__k.__m), s2 && s2(n3, t4);
  }, c2.__r = function(n3) {
    a2 && a2(n3), t2 = 0;
    var i4 = (r2 = n3.__c).__H;
    i4 && (u3 === r2 ? (i4.__h = [], r2.__h = [], i4.__.some(function(n4) {
      n4.__N && (n4.__ = n4.__N), n4.u = n4.__N = void 0;
    })) : (i4.__h.some(z2), i4.__h.some(B2), i4.__h = [], t2 = 0)), u3 = r2;
  }, c2.diffed = function(n3) {
    v2 && v2(n3);
    var t4 = n3.__c;
    t4 && t4.__H && (t4.__H.__h.length && (1 !== f3.push(t4) && i2 === c2.requestAnimationFrame || ((i2 = c2.requestAnimationFrame) || w2)(j2)), t4.__H.__.some(function(n4) {
      n4.u && (n4.__H = n4.u), n4.u = void 0;
    })), u3 = r2 = null;
  }, c2.__c = function(n3, t4) {
    t4.some(function(n4) {
      try {
        n4.__h.some(z2), n4.__h = n4.__h.filter(function(n5) {
          return !n5.__ || B2(n5);
        });
      } catch (r4) {
        t4.some(function(n5) {
          n5.__h && (n5.__h = []);
        }), t4 = [], c2.__e(r4, n4.__v);
      }
    }), l2 && l2(n3, t4);
  }, c2.unmount = function(n3) {
    m2 && m2(n3);
    var t4, r4 = n3.__c;
    r4 && r4.__H && (r4.__H.__.some(function(n4) {
      try {
        z2(n4);
      } catch (n5) {
        t4 = n5;
      }
    }), r4.__H = void 0, t4 && c2.__e(t4, r4.__v));
  };
  var k2 = "function" == typeof requestAnimationFrame;
  function w2(n3) {
    var t4, r4 = function() {
      clearTimeout(u5), k2 && cancelAnimationFrame(t4), setTimeout(n3);
    }, u5 = setTimeout(r4, 35);
    k2 && (t4 = requestAnimationFrame(r4));
  }
  function z2(n3) {
    var t4 = r2, u5 = n3.__c;
    "function" == typeof u5 && (n3.__c = void 0, u5()), r2 = t4;
  }
  function B2(n3) {
    var t4 = r2;
    n3.__c = n3.__(), r2 = t4;
  }
  function C2(n3, t4) {
    return !n3 || n3.length !== t4.length || t4.some(function(t5, r4) {
      return t5 !== n3[r4];
    });
  }
  function D2(n3, t4) {
    return "function" == typeof t4 ? t4(n3) : t4;
  }

  // src/ui/lib/rescan.ts
  function createTriggerRescan({
    button,
    fetchImpl,
    loadData: loadData2,
    showError,
    setTimer,
    logError = () => void 0
  }) {
    return async function triggerRescan() {
      button.disabled = true;
      button.textContent = "\u21BB Scanning...";
      try {
        const resp = await fetchImpl("/api/rescan", { method: "POST" });
        if (!resp.ok) {
          showError(`Rescan failed: HTTP ${resp.status} ${resp.statusText}`);
          button.textContent = "\u21BB Rescan (failed)";
          return;
        }
        const data = await resp.json();
        button.textContent = "\u21BB Rescan (" + data.new + " new, " + data.updated + " updated)";
        await loadData2(true);
      } catch (error) {
        const msg = error instanceof Error ? error.message : String(error);
        showError("Rescan failed: " + msg);
        button.textContent = "\u21BB Rescan (error)";
        logError(error);
      } finally {
        setTimer(() => {
          button.textContent = "\u21BB Rescan";
          button.disabled = false;
        }, 3e3);
      }
    };
  }

  // node_modules/@preact/signals-core/dist/signals-core.module.js
  var i3 = /* @__PURE__ */ Symbol.for("preact-signals");
  function t3() {
    if (!(s3 > 1)) {
      var i4, t4 = false;
      !(function() {
        var i5 = c3;
        c3 = void 0;
        while (void 0 !== i5) {
          if (i5.S.v === i5.v) i5.S.i = i5.i;
          i5 = i5.o;
        }
      })();
      while (void 0 !== h3) {
        var n3 = h3;
        h3 = void 0;
        v3++;
        while (void 0 !== n3) {
          var r4 = n3.u;
          n3.u = void 0;
          n3.f &= -3;
          if (!(8 & n3.f) && w3(n3)) try {
            n3.c();
          } catch (n4) {
            if (!t4) {
              i4 = n4;
              t4 = true;
            }
          }
          n3 = r4;
        }
      }
      v3 = 0;
      s3--;
      if (t4) throw i4;
    } else s3--;
  }
  function n2(i4) {
    if (s3 > 0) return i4();
    e3 = ++u4;
    s3++;
    try {
      return i4();
    } finally {
      t3();
    }
  }
  var r3 = void 0;
  function o3(i4) {
    var t4 = r3;
    r3 = void 0;
    try {
      return i4();
    } finally {
      r3 = t4;
    }
  }
  var f4;
  var h3 = void 0;
  var s3 = 0;
  var v3 = 0;
  var u4 = 0;
  var e3 = 0;
  var c3 = void 0;
  var d3 = 0;
  function a3(i4) {
    if (void 0 !== r3) {
      var t4 = i4.n;
      if (void 0 === t4 || t4.t !== r3) {
        t4 = { i: 0, S: i4, p: r3.s, n: void 0, t: r3, e: void 0, x: void 0, r: t4 };
        if (void 0 !== r3.s) r3.s.n = t4;
        r3.s = t4;
        i4.n = t4;
        if (32 & r3.f) i4.S(t4);
        return t4;
      } else if (-1 === t4.i) {
        t4.i = 0;
        if (void 0 !== t4.n) {
          t4.n.p = t4.p;
          if (void 0 !== t4.p) t4.p.n = t4.n;
          t4.p = r3.s;
          t4.n = void 0;
          r3.s.n = t4;
          r3.s = t4;
        }
        return t4;
      }
    }
  }
  function l3(i4, t4) {
    this.v = i4;
    this.i = 0;
    this.n = void 0;
    this.t = void 0;
    this.l = 0;
    this.W = null == t4 ? void 0 : t4.watched;
    this.Z = null == t4 ? void 0 : t4.unwatched;
    this.name = null == t4 ? void 0 : t4.name;
  }
  l3.prototype.brand = i3;
  l3.prototype.h = function() {
    return true;
  };
  l3.prototype.S = function(i4) {
    var t4 = this, n3 = this.t;
    if (n3 !== i4 && void 0 === i4.e) {
      i4.x = n3;
      this.t = i4;
      if (void 0 !== n3) n3.e = i4;
      else o3(function() {
        var i5;
        null == (i5 = t4.W) || i5.call(t4);
      });
    }
  };
  l3.prototype.U = function(i4) {
    var t4 = this;
    if (void 0 !== this.t) {
      var n3 = i4.e, r4 = i4.x;
      if (void 0 !== n3) {
        n3.x = r4;
        i4.e = void 0;
      }
      if (void 0 !== r4) {
        r4.e = n3;
        i4.x = void 0;
      }
      if (i4 === this.t) {
        this.t = r4;
        if (void 0 === r4) o3(function() {
          var i5;
          null == (i5 = t4.Z) || i5.call(t4);
        });
      }
    }
  };
  l3.prototype.subscribe = function(i4) {
    var t4 = this;
    return j3(function() {
      var n3 = t4.value, o4 = r3;
      r3 = void 0;
      try {
        i4(n3);
      } finally {
        r3 = o4;
      }
    }, { name: "sub" });
  };
  l3.prototype.valueOf = function() {
    return this.value;
  };
  l3.prototype.toString = function() {
    return this.value + "";
  };
  l3.prototype.toJSON = function() {
    return this.value;
  };
  l3.prototype.peek = function() {
    var i4 = r3;
    r3 = void 0;
    try {
      return this.value;
    } finally {
      r3 = i4;
    }
  };
  Object.defineProperty(l3.prototype, "value", { get: function() {
    var i4 = a3(this);
    if (void 0 !== i4) i4.i = this.i;
    return this.v;
  }, set: function(i4) {
    if (i4 !== this.v) {
      if (v3 > 100) throw new Error("Cycle detected");
      !(function(i5) {
        if (0 !== s3 && 0 === v3) {
          if (i5.l !== e3) {
            i5.l = e3;
            c3 = { S: i5, v: i5.v, i: i5.i, o: c3 };
          }
        }
      })(this);
      this.v = i4;
      this.i++;
      d3++;
      s3++;
      try {
        for (var n3 = this.t; void 0 !== n3; n3 = n3.x) n3.t.N();
      } finally {
        t3();
      }
    }
  } });
  function y3(i4, t4) {
    return new l3(i4, t4);
  }
  function w3(i4) {
    for (var t4 = i4.s; void 0 !== t4; t4 = t4.n) if (t4.S.i !== t4.i || !t4.S.h() || t4.S.i !== t4.i) return true;
    return false;
  }
  function _2(i4) {
    for (var t4 = i4.s; void 0 !== t4; t4 = t4.n) {
      var n3 = t4.S.n;
      if (void 0 !== n3) t4.r = n3;
      t4.S.n = t4;
      t4.i = -1;
      if (void 0 === t4.n) {
        i4.s = t4;
        break;
      }
    }
  }
  function b2(i4) {
    var t4 = i4.s, n3 = void 0;
    while (void 0 !== t4) {
      var r4 = t4.p;
      if (-1 === t4.i) {
        t4.S.U(t4);
        if (void 0 !== r4) r4.n = t4.n;
        if (void 0 !== t4.n) t4.n.p = r4;
      } else n3 = t4;
      t4.S.n = t4.r;
      if (void 0 !== t4.r) t4.r = void 0;
      t4 = r4;
    }
    i4.s = n3;
  }
  function p3(i4, t4) {
    l3.call(this, void 0);
    this.x = i4;
    this.s = void 0;
    this.g = d3 - 1;
    this.f = 4;
    this.W = null == t4 ? void 0 : t4.watched;
    this.Z = null == t4 ? void 0 : t4.unwatched;
    this.name = null == t4 ? void 0 : t4.name;
  }
  p3.prototype = new l3();
  p3.prototype.h = function() {
    this.f &= -3;
    if (1 & this.f) return false;
    if (32 == (36 & this.f)) return true;
    this.f &= -5;
    if (this.g === d3) return true;
    this.g = d3;
    this.f |= 1;
    if (this.i > 0 && !w3(this)) {
      this.f &= -2;
      return true;
    }
    var i4 = r3;
    try {
      _2(this);
      r3 = this;
      var t4 = this.x();
      if (16 & this.f || this.v !== t4 || 0 === this.i) {
        this.v = t4;
        this.f &= -17;
        this.i++;
      }
    } catch (i5) {
      this.v = i5;
      this.f |= 16;
      this.i++;
    }
    r3 = i4;
    b2(this);
    this.f &= -2;
    return true;
  };
  p3.prototype.S = function(i4) {
    if (void 0 === this.t) {
      this.f |= 36;
      for (var t4 = this.s; void 0 !== t4; t4 = t4.n) t4.S.S(t4);
    }
    l3.prototype.S.call(this, i4);
  };
  p3.prototype.U = function(i4) {
    if (void 0 !== this.t) {
      l3.prototype.U.call(this, i4);
      if (void 0 === this.t) {
        this.f &= -33;
        for (var t4 = this.s; void 0 !== t4; t4 = t4.n) t4.S.U(t4);
      }
    }
  };
  p3.prototype.N = function() {
    if (!(2 & this.f)) {
      this.f |= 6;
      for (var i4 = this.t; void 0 !== i4; i4 = i4.x) i4.t.N();
    }
  };
  Object.defineProperty(p3.prototype, "value", { get: function() {
    if (1 & this.f) throw new Error("Cycle detected");
    var i4 = a3(this);
    this.h();
    if (void 0 !== i4) i4.i = this.i;
    if (16 & this.f) throw this.v;
    return this.v;
  } });
  function g2(i4, t4) {
    return new p3(i4, t4);
  }
  function S2(i4) {
    var n3 = i4.m;
    i4.m = void 0;
    if ("function" == typeof n3) {
      s3++;
      var o4 = r3;
      r3 = void 0;
      try {
        n3();
      } catch (t4) {
        i4.f &= -2;
        i4.f |= 8;
        m3(i4);
        throw t4;
      } finally {
        r3 = o4;
        t3();
      }
    }
  }
  function m3(i4) {
    for (var t4 = i4.s; void 0 !== t4; t4 = t4.n) t4.S.U(t4);
    i4.x = void 0;
    i4.s = void 0;
    S2(i4);
  }
  function x2(i4) {
    if (r3 !== this) throw new Error("Out-of-order effect");
    b2(this);
    r3 = i4;
    this.f &= -2;
    if (8 & this.f) m3(this);
    t3();
  }
  function E2(i4, t4) {
    this.x = i4;
    this.m = void 0;
    this.s = void 0;
    this.u = void 0;
    this.f = 32;
    this.name = null == t4 ? void 0 : t4.name;
    if (f4) f4.push(this);
  }
  E2.prototype.c = function() {
    var i4 = this.S();
    try {
      if (8 & this.f) return;
      if (void 0 === this.x) return;
      var t4 = this.x();
      if ("function" == typeof t4) this.m = t4;
    } finally {
      i4();
    }
  };
  E2.prototype.S = function() {
    if (1 & this.f) throw new Error("Cycle detected");
    this.f |= 1;
    this.f &= -9;
    S2(this);
    _2(this);
    s3++;
    var i4 = r3;
    r3 = this;
    return x2.bind(this, i4);
  };
  E2.prototype.N = function() {
    if (!(2 & this.f)) {
      this.f |= 2;
      this.u = h3;
      h3 = this;
    }
  };
  E2.prototype.d = function() {
    this.f |= 8;
    if (!(1 & this.f)) m3(this);
  };
  E2.prototype.dispose = function() {
    this.d();
  };
  function j3(i4, t4) {
    var n3 = new E2(i4, t4);
    try {
      n3.c();
    } catch (i5) {
      n3.d();
      throw i5;
    }
    var r4 = n3.d.bind(n3);
    r4[Symbol.dispose] = r4;
    return r4;
  }

  // node_modules/@preact/signals/dist/signals.module.js
  var l4;
  var d4;
  var h4;
  var p4 = "undefined" != typeof window && !!window.__PREACT_SIGNALS_DEVTOOLS__;
  var _3 = [];
  j3(function() {
    l4 = this.N;
  })();
  function g3(i4, r4) {
    l[i4] = r4.bind(null, l[i4] || function() {
    });
  }
  function b3(i4) {
    if (h4) {
      var n3 = h4;
      h4 = void 0;
      n3();
    }
    h4 = i4 && i4.S();
  }
  function y4(i4) {
    var n3 = this, t4 = i4.data, e4 = useSignal(t4);
    e4.value = t4;
    var f5 = T2(function() {
      var i5 = n3, t5 = n3.__v;
      while (t5 = t5.__) if (t5.__c) {
        t5.__c.__$f |= 4;
        break;
      }
      var o4 = g2(function() {
        var i6 = e4.value.value;
        return 0 === i6 ? 0 : true === i6 ? "" : i6 || "";
      }), f6 = g2(function() {
        return !Array.isArray(o4.value) && !t(o4.value);
      }), a5 = j3(function() {
        this.N = F;
        if (f6.value) {
          var n4 = o4.value;
          if (i5.__v && i5.__v.__e && 3 === i5.__v.__e.nodeType) i5.__v.__e.data = n4;
        }
      }), v5 = n3.__$u.d;
      n3.__$u.d = function() {
        a5();
        v5.call(this);
      };
      return [f6, o4];
    }, []), a4 = f5[0], v4 = f5[1];
    return a4.value ? v4.peek() : v4.value;
  }
  y4.displayName = "ReactiveTextNode";
  Object.defineProperties(l3.prototype, { constructor: { configurable: true, value: void 0 }, type: { configurable: true, value: y4 }, props: { configurable: true, get: function() {
    var i4 = this;
    return { data: { get value() {
      return i4.value;
    } } };
  } }, __b: { configurable: true, value: 1 } });
  g3("__b", function(i4, n3) {
    if ("string" == typeof n3.type) {
      var r4, t4 = n3.props;
      for (var o4 in t4) if ("children" !== o4) {
        var e4 = t4[o4];
        if (e4 instanceof l3) {
          if (!r4) n3.__np = r4 = {};
          r4[o4] = e4;
          t4[o4] = e4.peek();
        }
      }
    }
    i4(n3);
  });
  g3("__r", function(i4, n3) {
    i4(n3);
    if (n3.type !== S) {
      b3();
      var r4, o4 = n3.__c;
      if (o4) {
        o4.__$f &= -2;
        if (void 0 === (r4 = o4.__$u)) o4.__$u = r4 = (function(i5, n4) {
          var r5;
          j3(function() {
            r5 = this;
          }, { name: n4 });
          r5.c = i5;
          return r5;
        })(function() {
          var i5;
          if (p4) null == (i5 = r4.y) || i5.call(r4);
          o4.__$f |= 1;
          o4.setState({});
        }, "function" == typeof n3.type ? n3.type.displayName || n3.type.name : "");
      }
      d4 = o4;
      b3(r4);
    }
  });
  g3("__e", function(i4, n3, r4, t4) {
    b3();
    d4 = void 0;
    i4(n3, r4, t4);
  });
  g3("diffed", function(i4, n3) {
    b3();
    d4 = void 0;
    var r4;
    if ("string" == typeof n3.type && (r4 = n3.__e)) {
      var t4 = n3.__np, o4 = n3.props;
      if (t4) {
        var e4 = r4.U;
        if (e4) for (var f5 in e4) {
          var u5 = e4[f5];
          if (void 0 !== u5 && !(f5 in t4)) {
            u5.d();
            e4[f5] = void 0;
          }
        }
        else {
          e4 = {};
          r4.U = e4;
        }
        for (var a4 in t4) {
          var c4 = e4[a4], v4 = t4[a4];
          if (void 0 === c4) {
            c4 = w4(r4, a4, v4);
            e4[a4] = c4;
          } else c4.o(v4, o4);
        }
        for (var s4 in t4) o4[s4] = t4[s4];
      }
    }
    i4(n3);
  });
  function w4(i4, n3, r4, t4) {
    var o4 = n3 in i4 && void 0 === i4.ownerSVGElement, e4 = y3(r4), f5 = r4.peek();
    return { o: function(i5, n4) {
      e4.value = i5;
      f5 = i5.peek();
    }, d: j3(function() {
      this.N = F;
      var r5 = e4.value.value;
      if (f5 !== r5) {
        f5 = void 0;
        if (o4) i4[n3] = r5;
        else if (null != r5 && (false !== r5 || "-" === n3[4])) i4.setAttribute(n3, r5);
        else i4.removeAttribute(n3);
      } else f5 = void 0;
    }) };
  }
  g3("unmount", function(i4, n3) {
    if ("string" == typeof n3.type) {
      var r4 = n3.__e;
      if (r4) {
        var t4 = r4.U;
        if (t4) {
          r4.U = void 0;
          for (var o4 in t4) {
            var e4 = t4[o4];
            if (e4) e4.d();
          }
        }
      }
      n3.__np = void 0;
    } else {
      var f5 = n3.__c;
      if (f5) {
        var u5 = f5.__$u;
        if (u5) {
          f5.__$u = void 0;
          u5.d();
        }
      }
    }
    i4(n3);
  });
  g3("__h", function(i4, n3, r4, t4) {
    if (t4 < 3 || 9 === t4) n3.__$f |= 2;
    i4(n3, r4, t4);
  });
  C.prototype.shouldComponentUpdate = function(i4, n3) {
    if (this.__R) return true;
    var r4 = this.__$u, t4 = r4 && void 0 !== r4.s;
    for (var o4 in n3) return true;
    if (this.__f || "boolean" == typeof this.u && true === this.u) {
      var e4 = 2 & this.__$f;
      if (!(t4 || e4 || 4 & this.__$f)) return true;
      if (1 & this.__$f) return true;
    } else {
      if (!(t4 || 4 & this.__$f)) return true;
      if (3 & this.__$f) return true;
    }
    for (var f5 in i4) if ("__source" !== f5 && i4[f5] !== this.props[f5]) return true;
    for (var u5 in this.props) if (!(u5 in i4)) return true;
    return false;
  };
  function useSignal(i4, n3) {
    return T2(function() {
      return y3(i4, n3);
    }, []);
  }
  var q2 = function(i4) {
    queueMicrotask(function() {
      queueMicrotask(i4);
    });
  };
  function x3() {
    n2(function() {
      var i4;
      while (i4 = _3.shift()) l4.call(i4);
    });
  }
  function F() {
    if (1 === _3.push(this)) (l.requestAnimationFrame || q2)(x3);
  }

  // src/ui/state/store.ts
  var rawData = y3(null);
  var billingBlocksData = y3(null);
  var contextWindowData = y3(null);
  var costReconciliationData = y3(null);
  var selectedModels = y3(/* @__PURE__ */ new Set());
  var selectedRange = y3("30d");
  var selectedProvider = y3("both");
  var projectSearchQuery = y3("");
  function readBucket() {
    const p5 = new URLSearchParams(window.location.search).get("bucket");
    return ["day", "week"].includes(p5) ? p5 : "day";
  }
  var selectedBucket = y3(readBucket());
  var lastFilteredSessions = y3([]);
  var lastByProject = y3([]);
  var metaText = y3("");
  var planBadge = y3("");
  var rescanLabel = y3("\u21BB Rescan");
  var rescanDisabled = y3(false);
  var themeMode = y3("dark");
  var statusByPlacement = y3({
    "global": null,
    "rate-windows": null,
    "rescan": null,
    "header-refresh": null,
    "agent-status": null,
    "community-signal": null
  });
  var SESSIONS_PAGE_SIZE = 25;
  var loadState = y3("idle");
  function readVersionMetric() {
    const p5 = new URLSearchParams(window.location.search).get("version_metric");
    return ["cost", "calls", "tokens"].includes(p5) ? p5 : "cost";
  }
  var versionDonutMetric = y3(readVersionMetric());
  function readAgentStatusExpanded() {
    const p5 = new URLSearchParams(window.location.search).get("agent_status_expanded");
    return p5 === "1" || p5 === "true";
  }
  var agent_status_expanded = y3(readAgentStatusExpanded());

  // src/ui/lib/status.ts
  var timers = {};
  function cancelTimer(placement) {
    const t4 = timers[placement];
    if (t4 != null) {
      clearTimeout(t4);
      delete timers[placement];
    }
  }
  function setStatus(placement, kind, message, autoDismissMs) {
    statusByPlacement.value = { ...statusByPlacement.value, [placement]: { kind, message } };
    cancelTimer(placement);
    if (autoDismissMs && autoDismissMs > 0) {
      timers[placement] = window.setTimeout(() => clearStatus(placement), autoDismissMs);
    }
  }
  function clearStatus(placement) {
    statusByPlacement.value = { ...statusByPlacement.value, [placement]: null };
    cancelTimer(placement);
  }

  // src/ui/components/InlineStatus.tsx
  var LABEL_MAP = {
    success: "OK",
    error: "ERROR",
    loading: "LOADING",
    info: "INFO"
  };
  var COLOR_MAP = {
    success: "var(--success)",
    error: "var(--accent)",
    loading: "var(--text-secondary)",
    info: "var(--text-secondary)"
  };
  function InlineStatus({ placement, inline = false, dismissable = true }) {
    const entry = statusByPlacement.value[placement];
    if (!entry) return null;
    const label = LABEL_MAP[entry.kind];
    const color = COLOR_MAP[entry.kind];
    const content = entry.message ? `[${label}: ${entry.message}]` : `[${label}]`;
    const baseStyle = {
      fontFamily: "var(--font-mono)",
      fontSize: "11px",
      letterSpacing: "0.08em",
      textTransform: "uppercase",
      color,
      animation: "fadeUp 0.15s ease-out",
      display: inline ? "inline-flex" : "flex",
      alignItems: "center",
      gap: "8px",
      padding: inline ? "0" : "8px 16px",
      border: inline ? "none" : `1px solid ${color}`,
      borderRadius: inline ? "0" : "4px",
      background: inline ? "transparent" : "var(--surface)"
    };
    return /* @__PURE__ */ u2("div", { role: entry.kind === "error" ? "alert" : "status", style: baseStyle, children: [
      /* @__PURE__ */ u2("span", { children: content }),
      dismissable && entry.kind !== "loading" && /* @__PURE__ */ u2(
        "button",
        {
          type: "button",
          onClick: () => clearStatus(placement),
          "aria-label": "Dismiss",
          style: {
            background: "transparent",
            border: "none",
            color,
            cursor: "pointer",
            fontFamily: "inherit",
            fontSize: "inherit",
            letterSpacing: "inherit",
            padding: "0 4px",
            opacity: 0.7
          },
          children: "[X]"
        }
      )
    ] });
  }

  // src/ui/components/Header.tsx
  function Header({ onDataReload, onThemeToggle }) {
    const btnRef = A2(null);
    const triggerRef = A2(null);
    y2(() => {
      if (!btnRef.current) return;
      const proxy = {
        get disabled() {
          return rescanDisabled.value;
        },
        set disabled(v4) {
          rescanDisabled.value = v4;
        },
        get textContent() {
          return rescanLabel.value;
        },
        set textContent(v4) {
          rescanLabel.value = v4 ?? "";
        }
      };
      triggerRef.current = createTriggerRescan({
        button: proxy,
        fetchImpl: (input, init) => fetch(input, init),
        loadData: onDataReload,
        showError: (msg) => setStatus("rescan", "error", msg, 6e3),
        setTimer: (cb, ms) => window.setTimeout(cb, ms),
        logError: (e4) => console.error(e4)
      });
    }, [onDataReload]);
    const mode = themeMode.value;
    const icon = mode === "dark" ? /* @__PURE__ */ u2("svg", { width: "14", height: "14", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", children: /* @__PURE__ */ u2("path", { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }) }) : /* @__PURE__ */ u2("svg", { width: "14", height: "14", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", children: [
      /* @__PURE__ */ u2("circle", { cx: "12", cy: "12", r: "5" }),
      /* @__PURE__ */ u2("line", { x1: "12", y1: "1", x2: "12", y2: "3" }),
      /* @__PURE__ */ u2("line", { x1: "12", y1: "21", x2: "12", y2: "23" }),
      /* @__PURE__ */ u2("line", { x1: "4.22", y1: "4.22", x2: "5.64", y2: "5.64" }),
      /* @__PURE__ */ u2("line", { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }),
      /* @__PURE__ */ u2("line", { x1: "1", y1: "12", x2: "3", y2: "12" }),
      /* @__PURE__ */ u2("line", { x1: "21", y1: "12", x2: "23", y2: "12" }),
      /* @__PURE__ */ u2("line", { x1: "4.22", y1: "19.78", x2: "5.64", y2: "18.36" }),
      /* @__PURE__ */ u2("line", { x1: "18.36", y1: "5.64", x2: "19.78", y2: "4.22" })
    ] });
    return /* @__PURE__ */ u2("header", { children: [
      /* @__PURE__ */ u2("h1", { children: [
        /* @__PURE__ */ u2("span", { class: "accent", children: "Code" }),
        " ",
        "Usage",
        planBadge.value && /* @__PURE__ */ u2(
          "span",
          {
            "aria-live": "polite",
            style: {
              fontFamily: "var(--font-mono)",
              fontSize: "10px",
              padding: "1px 8px",
              borderRadius: "999px",
              border: "1px solid var(--border-visible)",
              color: "var(--text-secondary)",
              verticalAlign: "middle",
              marginLeft: "8px",
              letterSpacing: "0.08em",
              textTransform: "uppercase"
            },
            children: planBadge.value
          }
        )
      ] }),
      /* @__PURE__ */ u2("div", { class: "meta", children: metaText.value }),
      /* @__PURE__ */ u2("div", { class: "header-actions", children: [
        /* @__PURE__ */ u2(
          "button",
          {
            class: "theme-toggle",
            type: "button",
            onClick: onThemeToggle,
            "aria-label": "Toggle theme",
            children: icon
          }
        ),
        /* @__PURE__ */ u2(
          "button",
          {
            id: "rescan-btn",
            ref: btnRef,
            type: "button",
            disabled: rescanDisabled.value,
            onClick: () => triggerRef.current?.(),
            "aria-label": "Rescan database",
            children: rescanLabel.value
          }
        ),
        /* @__PURE__ */ u2(InlineStatus, { placement: "rescan", inline: true }),
        /* @__PURE__ */ u2(InlineStatus, { placement: "header-refresh", inline: true, dismissable: false })
      ] })
    ] });
  }

  // src/ui/components/FilterBar.tsx
  var RANGES = ["7d", "30d", "90d", "all"];
  var BUCKETS = ["day", "week"];
  var BUCKET_LABEL = { day: "DAY", week: "WEEK" };
  var PROVIDERS = ["both", "claude", "codex"];
  var PROVIDER_LABEL = {
    both: "Both",
    claude: "Claude",
    codex: "Codex"
  };
  function modelPriority(m4) {
    const ml = m4.toLowerCase();
    if (ml.includes("opus")) return 0;
    if (ml.includes("sonnet")) return 1;
    if (ml.includes("haiku")) return 2;
    return 3;
  }
  function FilterBar({ onFilterChange, onURLUpdate }) {
    const allModels = rawData.value?.all_models ?? [];
    const sortedModels = [...allModels].sort((a4, b4) => {
      const pa = modelPriority(a4);
      const pb = modelPriority(b4);
      return pa !== pb ? pa - pb : a4.localeCompare(b4);
    });
    const toggleModel = (model, checked) => {
      const next = new Set(selectedModels.value);
      if (checked) next.add(model);
      else next.delete(model);
      selectedModels.value = next;
      onURLUpdate();
      onFilterChange();
    };
    const selectAll = () => {
      selectedModels.value = new Set(sortedModels);
      onURLUpdate();
      onFilterChange();
    };
    const clearAll = () => {
      selectedModels.value = /* @__PURE__ */ new Set();
      onURLUpdate();
      onFilterChange();
    };
    const setRange = (range) => {
      selectedRange.value = range;
      onURLUpdate();
      onFilterChange();
    };
    const setBucket = (bucket) => {
      selectedBucket.value = bucket;
      onURLUpdate();
      onFilterChange();
    };
    const setProvider = (provider) => {
      selectedProvider.value = provider;
      onURLUpdate();
      onFilterChange();
    };
    const hasCodexData = rawData.value?.provider_breakdown?.some((p5) => p5.provider === "codex") ?? false;
    const onSearchInput = (e4) => {
      const value = e4.currentTarget.value;
      projectSearchQuery.value = value.toLowerCase().trim();
      onURLUpdate();
      onFilterChange();
    };
    const clearSearch = () => {
      projectSearchQuery.value = "";
      onURLUpdate();
      onFilterChange();
    };
    return /* @__PURE__ */ u2("div", { id: "filter-bar", role: "toolbar", "aria-label": "Filters", children: [
      /* @__PURE__ */ u2("div", { class: "filter-label", children: "Models" }),
      /* @__PURE__ */ u2("div", { id: "model-checkboxes", role: "group", "aria-label": "Model filters", children: sortedModels.map((model) => {
        const checked = selectedModels.value.has(model);
        return /* @__PURE__ */ u2("label", { class: `model-cb-label${checked ? " checked" : ""}`, "data-model": model, children: [
          /* @__PURE__ */ u2(
            "input",
            {
              type: "checkbox",
              value: model,
              checked,
              onChange: (e4) => toggleModel(model, e4.currentTarget.checked),
              "aria-label": model
            }
          ),
          model
        ] }, model);
      }) }),
      /* @__PURE__ */ u2("button", { class: "filter-btn", type: "button", onClick: selectAll, children: "All" }),
      /* @__PURE__ */ u2("button", { class: "filter-btn", type: "button", onClick: clearAll, children: "None" }),
      /* @__PURE__ */ u2("div", { class: "filter-sep" }),
      /* @__PURE__ */ u2("div", { class: "filter-label", children: "Range" }),
      /* @__PURE__ */ u2("div", { class: "range-group", role: "group", "aria-label": "Date range", children: RANGES.map((range) => /* @__PURE__ */ u2(
        "button",
        {
          class: `range-btn${selectedRange.value === range ? " active" : ""}`,
          type: "button",
          "data-range": range,
          onClick: () => setRange(range),
          children: range
        },
        range
      )) }),
      /* @__PURE__ */ u2("div", { class: "filter-sep" }),
      /* @__PURE__ */ u2("div", { class: "filter-label", children: "Bucket" }),
      /* @__PURE__ */ u2("div", { class: "range-group", role: "group", "aria-label": "Chart bucket", children: BUCKETS.map((bucket) => /* @__PURE__ */ u2(
        "button",
        {
          class: `range-btn${selectedBucket.value === bucket ? " active" : ""}`,
          type: "button",
          "data-bucket": bucket,
          onClick: () => setBucket(bucket),
          children: BUCKET_LABEL[bucket]
        },
        bucket
      )) }),
      hasCodexData && /* @__PURE__ */ u2(S, { children: [
        /* @__PURE__ */ u2("div", { class: "filter-sep" }),
        /* @__PURE__ */ u2("div", { class: "filter-label", children: "Provider" }),
        /* @__PURE__ */ u2("div", { class: "range-group", role: "group", "aria-label": "Provider", children: PROVIDERS.map((provider) => /* @__PURE__ */ u2(
          "button",
          {
            class: `range-btn${selectedProvider.value === provider ? " active" : ""}`,
            type: "button",
            "data-provider": provider,
            onClick: () => setProvider(provider),
            children: PROVIDER_LABEL[provider]
          },
          provider
        )) })
      ] }),
      /* @__PURE__ */ u2("div", { class: "filter-sep" }),
      /* @__PURE__ */ u2("label", { for: "project-search", class: "filter-label", children: "Project" }),
      /* @__PURE__ */ u2(
        "input",
        {
          type: "text",
          id: "project-search",
          placeholder: "Search...",
          "aria-label": "Filter by project",
          value: projectSearchQuery.value,
          onInput: onSearchInput,
          style: {
            background: "transparent",
            border: "1px solid var(--border-visible)",
            color: "var(--text-primary)",
            padding: "3px 10px",
            borderRadius: "4px",
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            letterSpacing: "0.04em",
            width: "160px",
            outline: "none"
          }
        }
      ),
      projectSearchQuery.value && /* @__PURE__ */ u2("button", { class: "filter-btn", id: "project-clear-btn", type: "button", onClick: clearSearch, children: "Clear" })
    ] });
  }

  // src/ui/lib/format.ts
  function $2(id) {
    return document.getElementById(id);
  }
  function fmt(n3) {
    if (n3 >= 1e9) return (n3 / 1e9).toFixed(2) + "B";
    if (n3 >= 1e6) return (n3 / 1e6).toFixed(2) + "M";
    if (n3 >= 1e3) return (n3 / 1e3).toFixed(1) + "K";
    return n3.toLocaleString();
  }
  function fmtCost(c4) {
    return "$" + c4.toFixed(4);
  }
  function fmtCostBig(c4) {
    return "$" + c4.toFixed(2);
  }
  function fmtResetTime(minutes) {
    if (minutes == null || minutes <= 0) return "now";
    if (minutes >= 1440) return Math.floor(minutes / 1440) + "d " + Math.floor(minutes % 1440 / 60) + "h";
    if (minutes >= 60) return Math.floor(minutes / 60) + "h " + minutes % 60 + "m";
    return minutes + "m";
  }
  function anyHasCredits(rows) {
    return rows.some((r4) => r4.credits != null);
  }
  function fmtCredits(n3) {
    if (n3 == null) return "\u2014";
    return n3.toFixed(2);
  }

  // src/ui/components/SegmentedProgressBar.tsx
  function resolveStatus(pct, status) {
    if (status === "neutral") return "var(--text-display)";
    if (status === "success") return "var(--success)";
    if (status === "warning") return "var(--warning)";
    if (status === "accent") return "var(--accent)";
    if (pct >= 90) return "var(--accent)";
    if (pct >= 70) return "var(--warning)";
    return "var(--success)";
  }
  function SegmentedProgressBar({
    value,
    max: max2,
    segments = 20,
    size = "standard",
    status = "auto",
    "aria-label": ariaLabel
  }) {
    const safeMax = max2 > 0 ? max2 : 1;
    const ratio = value / safeMax;
    const pct = Math.min(100, Math.max(0, ratio * 100));
    const overflow = ratio > 1;
    const filled = Math.round(pct / 100 * segments);
    const fillColor = overflow ? "var(--accent)" : resolveStatus(pct, status);
    const emptyColor = "var(--border)";
    return /* @__PURE__ */ u2(
      "div",
      {
        class: `segmented-bar segmented-bar--${size}`,
        role: "progressbar",
        "aria-label": ariaLabel,
        "aria-valuenow": Math.round(pct),
        "aria-valuemin": 0,
        "aria-valuemax": 100,
        children: Array.from({ length: segments }).map((_4, i4) => /* @__PURE__ */ u2(
          "div",
          {
            class: "segmented-bar__segment",
            style: { background: i4 < filled ? fillColor : emptyColor }
          },
          i4
        ))
      }
    );
  }

  // src/ui/components/RateWindowCard.tsx
  function RateWindowCard({ label, window: window2 }) {
    const pct = Math.min(100, window2.used_percent);
    const resetText = window2.resets_in_minutes != null ? `Resets in ${fmtResetTime(window2.resets_in_minutes)}` : "";
    return /* @__PURE__ */ u2("div", { class: "card stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
      /* @__PURE__ */ u2("div", { class: "stat-label", children: label }),
      /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "28px" }, children: [
        pct.toFixed(1),
        "%"
      ] }),
      /* @__PURE__ */ u2("div", { style: { marginTop: "12px" }, children: /* @__PURE__ */ u2(
        SegmentedProgressBar,
        {
          value: window2.used_percent,
          max: 100,
          size: "standard",
          "aria-label": `${label} usage`
        }
      ) }),
      resetText && /* @__PURE__ */ u2("div", { class: "stat-sub", children: resetText })
    ] }) });
  }
  function BudgetCard({ used, limit, currency, utilization }) {
    return /* @__PURE__ */ u2("div", { class: "card stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
      /* @__PURE__ */ u2("div", { class: "stat-label", children: "Monthly Budget" }),
      /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "24px" }, children: [
        "$",
        used.toFixed(2),
        " / $",
        limit.toFixed(2)
      ] }),
      /* @__PURE__ */ u2("div", { style: { marginTop: "12px" }, children: /* @__PURE__ */ u2(
        SegmentedProgressBar,
        {
          value: utilization,
          max: 100,
          size: "standard",
          "aria-label": "Monthly budget usage"
        }
      ) }),
      /* @__PURE__ */ u2("div", { class: "stat-sub", children: currency })
    ] }) });
  }
  function RateWindowUnavailable({ error }) {
    return /* @__PURE__ */ u2("div", { class: "card stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
      /* @__PURE__ */ u2("div", { class: "stat-label", children: "Rate Windows" }),
      /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "18px", color: "var(--text-secondary)" }, children: "Unavailable" }),
      /* @__PURE__ */ u2("div", { class: "stat-sub", children: error })
    ] }) });
  }

  // src/ui/components/AgentStatusCard.tsx
  function IndicatorDot({ indicator }) {
    const isAlert = indicator === "major" || indicator === "critical";
    const isMinor = indicator === "minor";
    const color = isAlert ? "var(--accent)" : "var(--text-secondary)";
    const opacity = isAlert ? 1 : isMinor ? 0.6 : 0.3;
    return /* @__PURE__ */ u2(
      "span",
      {
        "aria-label": `Status: ${indicator}`,
        style: {
          display: "inline-block",
          width: "10px",
          height: "10px",
          borderRadius: "50%",
          backgroundColor: color,
          opacity,
          marginRight: "8px",
          flexShrink: 0
        }
      }
    );
  }
  function ProviderRow({ name, status, expanded }) {
    if (!status) {
      return /* @__PURE__ */ u2("div", { style: { display: "flex", alignItems: "center", padding: "8px 0", gap: "8px" }, children: [
        /* @__PURE__ */ u2(IndicatorDot, { indicator: "none" }),
        /* @__PURE__ */ u2("span", { style: { fontFamily: "var(--font-mono)", fontSize: "13px", flex: 1 }, children: name }),
        /* @__PURE__ */ u2("span", { style: { color: "var(--text-secondary)", fontSize: "12px" }, children: "unavailable" })
      ] });
    }
    const incidentCount = status.active_incidents.length;
    return /* @__PURE__ */ u2("div", { children: [
      /* @__PURE__ */ u2("div", { style: { display: "flex", alignItems: "center", padding: "8px 0", gap: "8px" }, children: [
        /* @__PURE__ */ u2(IndicatorDot, { indicator: status.indicator }),
        /* @__PURE__ */ u2("span", { style: { fontFamily: "var(--font-mono)", fontSize: "13px", flex: 1 }, children: name }),
        /* @__PURE__ */ u2("span", { style: { color: "var(--text-secondary)", fontSize: "12px" }, children: status.description }),
        incidentCount > 0 && /* @__PURE__ */ u2(
          "span",
          {
            style: {
              fontFamily: "var(--font-mono)",
              fontSize: "11px",
              color: status.indicator === "major" || status.indicator === "critical" ? "var(--accent)" : "var(--text-secondary)",
              marginLeft: "8px"
            },
            children: [
              "(",
              incidentCount,
              " active)"
            ]
          }
        ),
        /* @__PURE__ */ u2(
          "a",
          {
            href: status.page_url,
            target: "_blank",
            rel: "noopener noreferrer",
            style: { color: "var(--text-secondary)", fontSize: "11px", marginLeft: "4px" },
            "aria-label": `${name} status page`,
            children: "\u2197"
          }
        )
      ] }),
      expanded && /* @__PURE__ */ u2("div", { style: { paddingLeft: "18px", paddingBottom: "8px" }, children: [
        status.components.length > 0 && /* @__PURE__ */ u2("table", { style: { width: "100%", fontSize: "12px", borderCollapse: "collapse", marginBottom: "8px" }, children: [
          /* @__PURE__ */ u2("thead", { children: /* @__PURE__ */ u2("tr", { style: { color: "var(--text-secondary)" }, children: [
            /* @__PURE__ */ u2("th", { style: { textAlign: "left", padding: "2px 8px 2px 0", fontWeight: 500 }, children: "Component" }),
            /* @__PURE__ */ u2("th", { style: { textAlign: "left", padding: "2px 0", fontWeight: 500 }, children: "Status" })
          ] }) }),
          /* @__PURE__ */ u2("tbody", { children: status.components.map((c4, i4) => {
            const fmt2 = (v4) => v4 != null ? `${(v4 * 100).toFixed(2)}%` : "--";
            const has30 = c4.uptime_30d != null;
            const has7 = c4.uptime_7d != null;
            const showUptime = has30 || has7;
            return /* @__PURE__ */ u2(S, { children: [
              /* @__PURE__ */ u2("tr", { children: [
                /* @__PURE__ */ u2("td", { style: { padding: "2px 8px 2px 0", fontFamily: "var(--font-mono)" }, children: c4.name }),
                /* @__PURE__ */ u2("td", { style: { padding: "2px 0", color: "var(--text-secondary)" }, children: c4.status.replace(/_/g, " ") })
              ] }, i4),
              showUptime && /* @__PURE__ */ u2("tr", { children: /* @__PURE__ */ u2("td", { colSpan: 2, style: { padding: "0 0 4px 0" }, children: /* @__PURE__ */ u2("span", { style: {
                fontFamily: "var(--font-mono)",
                fontSize: "11px",
                letterSpacing: "0.04em"
              }, children: [
                /* @__PURE__ */ u2("span", { style: { color: "var(--text-secondary)" }, children: "30D " }),
                /* @__PURE__ */ u2("span", { style: { color: "var(--text-primary)" }, children: fmt2(c4.uptime_30d) }),
                /* @__PURE__ */ u2("span", { style: { color: "var(--text-secondary)" }, children: " \xB7 7D " }),
                /* @__PURE__ */ u2("span", { style: { color: "var(--text-primary)" }, children: fmt2(c4.uptime_7d) })
              ] }) }) }, `${i4}-uptime`)
            ] });
          }) })
        ] }),
        status.active_incidents.map((inc, i4) => /* @__PURE__ */ u2(
          "div",
          {
            style: {
              fontSize: "12px",
              color: "var(--text-secondary)",
              marginBottom: "4px",
              paddingLeft: "4px",
              borderLeft: "2px solid var(--border)"
            },
            children: [
              /* @__PURE__ */ u2("span", { style: { fontFamily: "var(--font-mono)" }, children: inc.shortlink ? /* @__PURE__ */ u2(
                "a",
                {
                  href: inc.shortlink,
                  target: "_blank",
                  rel: "noopener noreferrer",
                  style: { color: "inherit", textDecoration: "underline" },
                  children: inc.name
                }
              ) : inc.name }),
              " ",
              /* @__PURE__ */ u2("span", { style: { opacity: 0.7 }, children: [
                "[",
                inc.impact,
                "] ",
                inc.status,
                " \u2014 ",
                inc.started_at.slice(0, 16).replace("T", " ")
              ] })
            ]
          },
          i4
        ))
      ] })
    ] });
  }
  function signalLevelStyle(level) {
    switch (level) {
      case "spike":
        return { label: "SPIKE", color: "var(--accent)" };
      case "elevated":
        return { label: "ELEVATED", color: "var(--text-primary)" };
      case "normal":
        return { label: "NORMAL", color: "var(--text-secondary)" };
      default:
        return { label: "UNKNOWN", color: "var(--text-secondary)" };
    }
  }
  function CommunitySignalRow({ label, signals }) {
    const first = signals[0];
    if (!first) return null;
    const levelOrder = ["spike", "elevated", "normal", "unknown"];
    const worstLevel = levelOrder.find((l5) => signals.some((s4) => s4.level === l5)) ?? "unknown";
    const { label: levelLabel, color } = signalLevelStyle(worstLevel);
    return /* @__PURE__ */ u2("div", { style: { display: "flex", alignItems: "center", padding: "4px 0", gap: "8px" }, children: [
      /* @__PURE__ */ u2("span", { style: { fontFamily: "var(--font-mono)", fontSize: "13px", flex: 1 }, children: label }),
      /* @__PURE__ */ u2("span", { style: { fontFamily: "var(--font-mono)", fontSize: "12px", color }, children: levelLabel }),
      /* @__PURE__ */ u2(
        "a",
        {
          href: first.source_url,
          target: "_blank",
          rel: "noopener noreferrer",
          style: { color: "var(--text-secondary)", fontSize: "11px" },
          "aria-label": `${label} community signal source`,
          children: "\u2197"
        }
      )
    ] });
  }
  function AgentStatusCard({ snapshot, communitySignal }) {
    const expanded = agent_status_expanded.value;
    const hasData = snapshot.claude != null || snapshot.openai != null;
    return /* @__PURE__ */ u2("div", { class: "card stat-card", style: { minWidth: "300px" }, children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
      /* @__PURE__ */ u2(
        "div",
        {
          style: {
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            marginBottom: "8px"
          },
          children: [
            /* @__PURE__ */ u2("div", { class: "stat-label", children: "Agent Status" }),
            hasData && /* @__PURE__ */ u2(
              "button",
              {
                onClick: () => {
                  agent_status_expanded.value = !expanded;
                },
                style: {
                  background: "none",
                  border: "none",
                  cursor: "pointer",
                  color: "var(--text-secondary)",
                  fontSize: "11px",
                  fontFamily: "var(--font-mono)",
                  padding: "2px 4px"
                },
                "aria-expanded": expanded,
                "aria-label": "Toggle agent status details",
                children: expanded ? "\u25B2 collapse" : "\u25BC expand"
              }
            )
          ]
        }
      ),
      /* @__PURE__ */ u2(ProviderRow, { name: "Claude", status: snapshot.claude, expanded }),
      /* @__PURE__ */ u2(ProviderRow, { name: "OpenAI / Codex", status: snapshot.openai, expanded }),
      communitySignal?.enabled && (communitySignal.claude.length > 0 || communitySignal.openai.length > 0) && /* @__PURE__ */ u2("div", { style: { marginTop: "12px", borderTop: "1px solid var(--border)", paddingTop: "8px" }, children: [
        /* @__PURE__ */ u2("div", { style: { fontSize: "11px", fontFamily: "var(--font-mono)", color: "var(--text-secondary)", marginBottom: "6px", letterSpacing: "0.06em" }, children: "COMMUNITY SIGNAL" }),
        /* @__PURE__ */ u2(CommunitySignalRow, { label: "Claude", signals: communitySignal.claude }),
        /* @__PURE__ */ u2(CommunitySignalRow, { label: "OpenAI", signals: communitySignal.openai }),
        communitySignal.fetched_at && /* @__PURE__ */ u2("div", { style: { fontSize: "10px", color: "var(--text-secondary)", marginTop: "4px", fontFamily: "var(--font-mono)" }, children: [
          "Crowd data ",
          communitySignal.fetched_at.slice(0, 19).replace("T", " "),
          " UTC"
        ] })
      ] }),
      snapshot.fetched_at && /* @__PURE__ */ u2("div", { style: { fontSize: "10px", color: "var(--text-secondary)", marginTop: "8px", fontFamily: "var(--font-mono)" }, children: [
        "Refreshed ",
        snapshot.fetched_at.slice(0, 19).replace("T", " "),
        " UTC"
      ] })
    ] }) });
  }

  // src/ui/components/EstimationMeta.tsx
  function EstimationMeta({
    confidenceBreakdown,
    billingModeBreakdown,
    pricingVersions
  }) {
    return /* @__PURE__ */ u2(S, { children: [
      /* @__PURE__ */ u2("div", { class: "card stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
        /* @__PURE__ */ u2("div", { class: "stat-label", children: "Cost Confidence" }),
        /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "18px" }, children: confidenceBreakdown.length ? confidenceBreakdown.map(([key, value]) => `${key} ${value.sessions}`).join(" / ") : "n/a" }),
        /* @__PURE__ */ u2("div", { class: "stat-sub", children: "Session mix in current filter" })
      ] }) }),
      /* @__PURE__ */ u2("div", { class: "card stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
        /* @__PURE__ */ u2("div", { class: "stat-label", children: "Billing Mode" }),
        /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "18px" }, children: billingModeBreakdown.length ? billingModeBreakdown.map(([key, value]) => `${key} ${value.sessions}`).join(" / ") : "n/a" }),
        /* @__PURE__ */ u2("div", { class: "stat-sub", children: "Local estimate vs subscriber-included sessions" })
      ] }) }),
      /* @__PURE__ */ u2("div", { class: "card stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
        /* @__PURE__ */ u2("div", { class: "stat-label", children: "Pricing Snapshot" }),
        /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "18px" }, children: pricingVersions.length === 0 ? "n/a" : pricingVersions.length === 1 ? pricingVersions[0] : `mixed (${pricingVersions.length})` }),
        /* @__PURE__ */ u2("div", { class: "stat-sub", children: "Stored per-session pricing metadata" })
      ] }) })
    ] });
  }

  // src/ui/components/ReconciliationBlock.tsx
  function ReconciliationBlock({ reconciliation }) {
    const deltaMatch = Math.abs(reconciliation.delta_cost) < 0.01;
    return /* @__PURE__ */ u2("div", { class: "card card-flat bento-full", children: [
      /* @__PURE__ */ u2("h2", { children: "OpenAI Org Usage Reconciliation" }),
      /* @__PURE__ */ u2("div", { class: "muted", style: { marginBottom: "12px" }, children: [
        "Official OpenAI organization usage buckets for Codex-compatible models over the last ",
        reconciliation.lookback_days,
        " days."
      ] }),
      reconciliation.available ? /* @__PURE__ */ u2("div", { style: { display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(200px,1fr))", gap: "16px" }, children: [
        /* @__PURE__ */ u2("div", { class: "stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: "Period" }),
          /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "16px" }, children: [
            reconciliation.start_date,
            " - ",
            reconciliation.end_date
          ] }),
          /* @__PURE__ */ u2("div", { class: "stat-sub", children: "Rolling comparison window" })
        ] }) }),
        /* @__PURE__ */ u2("div", { class: "stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: "Local Estimated Cost" }),
          /* @__PURE__ */ u2("div", { class: "stat-value cost-value", style: { fontSize: "20px" }, children: [
            "$",
            reconciliation.estimated_local_cost.toFixed(4)
          ] }),
          /* @__PURE__ */ u2("div", { class: "stat-sub", children: "Codex local logs" })
        ] }) }),
        /* @__PURE__ */ u2("div", { class: "stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: "Org Usage Cost" }),
          /* @__PURE__ */ u2("div", { class: "stat-value cost-value", style: { fontSize: "20px" }, children: [
            "$",
            reconciliation.api_usage_cost.toFixed(4)
          ] }),
          /* @__PURE__ */ u2("div", { class: "stat-sub", children: "OpenAI organization usage API" })
        ] }) }),
        /* @__PURE__ */ u2("div", { class: "stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: "Delta" }),
          /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "20px", color: deltaMatch ? "var(--text-primary)" : "var(--accent)" }, children: [
            reconciliation.delta_cost >= 0 ? "+" : "",
            "$",
            reconciliation.delta_cost.toFixed(4)
          ] }),
          /* @__PURE__ */ u2("div", { class: "stat-sub", children: "Org usage cost minus local estimate" })
        ] }) }),
        /* @__PURE__ */ u2("div", { class: "stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: "API Tokens" }),
          /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "16px" }, children: [
            reconciliation.api_input_tokens.toLocaleString(),
            " / ",
            reconciliation.api_output_tokens.toLocaleString()
          ] }),
          /* @__PURE__ */ u2("div", { class: "stat-sub", children: "Input / output tokens" })
        ] }) }),
        /* @__PURE__ */ u2("div", { class: "stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: "Cached Input + Requests" }),
          /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "16px" }, children: [
            reconciliation.api_cached_input_tokens.toLocaleString(),
            " / ",
            reconciliation.api_requests.toLocaleString()
          ] }),
          /* @__PURE__ */ u2("div", { class: "stat-sub", children: "Cached input tokens / requests" })
        ] }) })
      ] }) : /* @__PURE__ */ u2("div", { class: "muted", children: reconciliation.error ?? "Unavailable" })
    ] });
  }

  // src/ui/lib/charts.ts
  var RANGE_LABELS = {
    "7d": "Last 7 Days",
    "30d": "Last 30 Days",
    "90d": "Last 90 Days",
    "all": "All Time"
  };
  var RANGE_TICKS = { "7d": 7, "30d": 15, "90d": 13, "all": 12 };
  function apexThemeMode() {
    return document.documentElement.getAttribute("data-theme") === "light" ? "light" : "dark";
  }
  function cssVar(name) {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }
  function hexToRgba(hex, alpha) {
    let h5 = hex.trim();
    if (h5.startsWith("#")) h5 = h5.slice(1);
    if (h5.length === 3) h5 = h5.split("").map((c4) => c4 + c4).join("");
    if (h5.length !== 6) return hex;
    const r4 = parseInt(h5.slice(0, 2), 16);
    const g4 = parseInt(h5.slice(2, 4), 16);
    const b4 = parseInt(h5.slice(4, 6), 16);
    return `rgba(${r4}, ${g4}, ${b4}, ${alpha})`;
  }
  function withAlpha(varName, alpha) {
    return hexToRgba(cssVar(varName), alpha);
  }
  function tokenSeriesColors() {
    return [
      withAlpha("--text-display", 1),
      withAlpha("--text-display", 0.6),
      withAlpha("--text-display", 0.3),
      withAlpha("--text-display", 0.15)
    ];
  }
  function modelSeriesColors(n3) {
    const baseVars = ["--text-display", "--success", "--warning", "--interactive"];
    const out = [];
    for (let i4 = 0; i4 < n3; i4++) {
      const slot = i4 % baseVars.length;
      const cycle = Math.floor(i4 / baseVars.length);
      const alpha = Math.max(0.25, 1 - cycle * 0.25);
      const v4 = baseVars[slot];
      out.push(cycle === 0 ? cssVar(v4) : withAlpha(v4, alpha));
    }
    return out;
  }
  function industrialChartOptions(type) {
    const axisLabelStyle = {
      colors: cssVar("--text-secondary"),
      fontFamily: 'var(--font-mono), "Space Mono", monospace',
      fontSize: "11px",
      letterSpacing: "0.04em"
    };
    const base = {
      chart: {
        type,
        height: "100%",
        background: "transparent",
        toolbar: { show: false },
        fontFamily: 'var(--font-mono), "Space Mono", monospace',
        animations: { enabled: false }
      },
      theme: { mode: apexThemeMode() },
      legend: {
        show: true,
        position: type === "donut" ? "bottom" : "top",
        fontFamily: 'var(--font-mono), "Space Mono", monospace',
        fontSize: "11px",
        labels: { colors: cssVar("--text-secondary") },
        markers: { width: 8, height: 8, radius: 0 },
        itemMargin: { horizontal: 12, vertical: 4 }
      },
      grid: {
        borderColor: cssVar("--border"),
        strokeDashArray: 0,
        xaxis: { lines: { show: false } },
        yaxis: { lines: { show: type !== "donut" } }
      },
      xaxis: {
        labels: { style: axisLabelStyle },
        axisBorder: { color: cssVar("--border-visible") },
        axisTicks: { color: cssVar("--border-visible") }
      },
      yaxis: {
        labels: { style: axisLabelStyle }
      },
      stroke: { width: type === "line" ? 1.5 : 0, curve: "straight" },
      tooltip: {
        theme: apexThemeMode(),
        style: { fontFamily: 'var(--font-mono), "Space Mono", monospace', fontSize: "11px" }
      },
      dataLabels: { enabled: false }
    };
    if (type === "line") {
      base.legend.show = false;
      base.fill = { type: "solid", opacity: 0 };
    }
    return base;
  }

  // src/ui/components/ApexChart.tsx
  function ApexChart({ options, id }) {
    const ref = A2(null);
    const chartRef = A2(null);
    const themeMode2 = options.theme?.mode ?? "";
    const optionsKey = T2(() => {
      const s4 = options.series;
      const type = options.chart?.type ?? "";
      if (Array.isArray(s4)) {
        const parts = s4.map((ss) => {
          const d5 = ss.data;
          if (!d5 || !d5.length) return "0";
          return `${d5.length}:${d5[0]}:${d5[d5.length - 1]}`;
        });
        return `${type}-${themeMode2}-${parts.join(",")}`;
      }
      return `${type}-${themeMode2}-${s4?.length ?? 0}`;
    }, [options, themeMode2]);
    y2(() => {
      if (chartRef.current) chartRef.current.destroy();
      if (!ref.current || !options) {
        return () => {
          chartRef.current?.destroy();
          chartRef.current = null;
        };
      }
      let cancelled = false;
      const raf = requestAnimationFrame(() => {
        if (cancelled || !ref.current) return;
        const parent = ref.current.parentElement;
        let h5 = parent?.clientHeight ?? 0;
        if (h5 <= 0) h5 = parent?.classList.contains("tall") ? 300 : 240;
        const opts = { ...options, chart: { ...options.chart, height: h5 } };
        chartRef.current = new ApexCharts(ref.current, opts);
        chartRef.current.render();
      });
      return () => {
        cancelled = true;
        cancelAnimationFrame(raf);
        chartRef.current?.destroy();
        chartRef.current = null;
      };
    }, [optionsKey]);
    return /* @__PURE__ */ u2("div", { ref, id, style: { width: "100%", height: "100%" } });
  }

  // src/ui/components/Sparkline.tsx
  function Sparkline({ daily }) {
    const last7 = daily.slice(-7);
    if (last7.length < 2) return null;
    const options = {
      chart: {
        type: "line",
        height: 30,
        width: 120,
        sparkline: { enabled: true },
        background: "transparent",
        fontFamily: "inherit"
      },
      series: [{ data: last7.map((d5) => d5.input + d5.output) }],
      stroke: { width: 1.5, curve: "smooth" },
      colors: [cssVar("--accent")],
      tooltip: { enabled: false }
    };
    return /* @__PURE__ */ u2("div", { children: [
      /* @__PURE__ */ u2("div", { class: "sub", style: { marginBottom: "4px" }, children: "7-day trend" }),
      /* @__PURE__ */ u2(ApexChart, { options })
    ] });
  }

  // src/ui/components/CacheEfficiencyCard.tsx
  function CacheEfficiencyCard({
    data,
    inputRatePerMtok,
    cacheReadRatePerMtok
  }) {
    const rate = data.cache_hit_rate;
    const hasRate = rate !== null && rate !== void 0;
    const displayPct = hasRate ? (rate * 100).toFixed(1) + "%" : "--";
    const barFill = hasRate ? Math.max(0, Math.min(1, rate)) : 0;
    const tooltipParts = [];
    if (hasRate) {
      const readM = (data.cache_read_tokens / 1e6).toFixed(2);
      const totalM = ((data.cache_read_tokens + data.input_tokens) / 1e6).toFixed(2);
      tooltipParts.push(`${readM}M tokens cache-read / ${totalM}M total input-addressable tokens`);
      if (inputRatePerMtok !== void 0 && cacheReadRatePerMtok !== void 0 && data.cache_read_tokens > 0) {
        const savedUsd = data.cache_read_tokens / 1e6 * (inputRatePerMtok - cacheReadRatePerMtok);
        tooltipParts.push(
          `saved approx $${savedUsd.toFixed(2)} vs. no-cache`
        );
      }
    } else {
      tooltipParts.push("No cache activity recorded");
    }
    const tooltip = tooltipParts.join(" \xB7 ");
    return /* @__PURE__ */ u2("div", { class: "card stat-card", title: tooltip, children: [
      /* @__PURE__ */ u2("div", { class: "stat-content", children: [
        /* @__PURE__ */ u2("div", { class: "stat-label", children: "Cache Hit Rate" }),
        /* @__PURE__ */ u2(
          "div",
          {
            class: "stat-value",
            style: { fontFamily: "var(--font-mono)", letterSpacing: "-0.02em" },
            children: displayPct
          }
        ),
        /* @__PURE__ */ u2("div", { class: "stat-sub", children: "prompt cache reuse" })
      ] }),
      /* @__PURE__ */ u2(
        "div",
        {
          style: {
            marginTop: "10px",
            height: "4px",
            borderRadius: "2px",
            background: "rgba(var(--text-primary-rgb, 232,232,232), 0.12)",
            overflow: "hidden"
          },
          "aria-label": `Cache hit rate: ${displayPct}`,
          children: /* @__PURE__ */ u2(
            "div",
            {
              style: {
                height: "100%",
                width: `${(barFill * 100).toFixed(2)}%`,
                background: "rgba(var(--text-primary-rgb, 232,232,232), 0.70)",
                borderRadius: "2px",
                transition: "width 300ms cubic-bezier(0.25,0.1,0.25,1)"
              }
            }
          )
        }
      )
    ] });
  }

  // src/ui/components/BillingBlocksCard.tsx
  function severityToStatus(s4) {
    if (s4 === "ok") return "success";
    if (s4 === "warn") return "warning";
    return "accent";
  }
  function severityLabel(s4) {
    return s4 === "ok" ? "[OK]" : s4 === "warn" ? "[WARN]" : "[CRIT]";
  }
  function tierLabel(t4) {
    if (t4 === "normal") return "[NORMAL]";
    if (t4 === "moderate") return "[WARN]";
    return "[CRIT]";
  }
  function tierColor(t4) {
    if (t4 === "normal") return "var(--success)";
    if (t4 === "moderate") return "var(--warning)";
    return "var(--accent)";
  }
  function formatDuration(from, to) {
    const diffMs = new Date(to).getTime() - new Date(from).getTime();
    if (isNaN(diffMs) || diffMs < 0) return "--";
    const totalMin = Math.floor(diffMs / 6e4);
    const h5 = Math.floor(totalMin / 60);
    const m4 = totalMin % 60;
    if (h5 === 0) return `${m4}m`;
    return `${h5}h ${m4}m`;
  }
  function fmtTokens(n3) {
    if (n3 >= 1e6) return (n3 / 1e6).toFixed(1) + "M";
    if (n3 >= 1e3) return (n3 / 1e3).toFixed(1) + "K";
    return n3.toString();
  }
  function fmtUtcTime(iso) {
    try {
      const d5 = new Date(iso);
      return d5.toISOString().slice(11, 16) + " UTC";
    } catch {
      return "--";
    }
  }
  function QuotaSection({ block }) {
    const { quota } = block;
    if (!quota) {
      return /* @__PURE__ */ u2(
        "div",
        {
          class: "stat-sub",
          style: { marginTop: "8px", fontStyle: "italic", opacity: 0.6 },
          children: "Token quota not configured \u2014 set [blocks.token_limit] in config."
        }
      );
    }
    const currentPct = Math.min(100, quota.current_pct).toFixed(0);
    const projectedPct = Math.min(999, quota.projected_pct).toFixed(0);
    return /* @__PURE__ */ u2("div", { style: { marginTop: "10px" }, children: [
      /* @__PURE__ */ u2("div", { style: { marginBottom: "4px" }, children: [
        /* @__PURE__ */ u2(
          "div",
          {
            style: {
              display: "flex",
              justifyContent: "space-between",
              alignItems: "baseline",
              marginBottom: "3px"
            },
            children: [
              /* @__PURE__ */ u2("span", { class: "stat-sub", style: { fontSize: "10px", letterSpacing: "0.08em" }, children: "USED" }),
              /* @__PURE__ */ u2(
                "span",
                {
                  class: "stat-sub",
                  style: { fontFamily: "var(--font-mono)", fontSize: "11px" },
                  children: [
                    fmtTokens(quota.used_tokens),
                    " / ",
                    fmtTokens(quota.limit_tokens),
                    " ",
                    currentPct,
                    "%",
                    " ",
                    /* @__PURE__ */ u2(
                      "span",
                      {
                        style: {
                          color: quota.current_severity === "danger" ? "var(--accent)" : quota.current_severity === "warn" ? "var(--warning)" : void 0
                        },
                        children: severityLabel(quota.current_severity)
                      }
                    )
                  ]
                }
              )
            ]
          }
        ),
        /* @__PURE__ */ u2(
          SegmentedProgressBar,
          {
            value: quota.used_tokens,
            max: quota.limit_tokens,
            status: severityToStatus(quota.current_severity),
            "aria-label": "Token quota used"
          }
        )
      ] }),
      /* @__PURE__ */ u2("div", { style: { marginTop: "1px" }, children: [
        /* @__PURE__ */ u2(
          "div",
          {
            style: {
              display: "flex",
              justifyContent: "space-between",
              alignItems: "baseline",
              marginBottom: "3px"
            },
            children: [
              /* @__PURE__ */ u2("span", { class: "stat-sub", style: { fontSize: "10px", letterSpacing: "0.08em" }, children: "PROJECTED" }),
              /* @__PURE__ */ u2(
                "span",
                {
                  class: "stat-sub",
                  style: { fontFamily: "var(--font-mono)", fontSize: "11px" },
                  children: [
                    fmtTokens(quota.projected_tokens),
                    " / ",
                    fmtTokens(quota.limit_tokens),
                    " ",
                    projectedPct,
                    "%",
                    " ",
                    /* @__PURE__ */ u2(
                      "span",
                      {
                        style: {
                          color: quota.projected_severity === "danger" ? "var(--accent)" : quota.projected_severity === "warn" ? "var(--warning)" : void 0
                        },
                        children: severityLabel(quota.projected_severity)
                      }
                    )
                  ]
                }
              )
            ]
          }
        ),
        /* @__PURE__ */ u2(
          SegmentedProgressBar,
          {
            value: quota.projected_tokens,
            max: quota.limit_tokens,
            status: severityToStatus(quota.projected_severity),
            "aria-label": "Projected token quota"
          }
        )
      ] })
    ] });
  }
  function BillingBlocksCard({ data }) {
    const activeBlock = data.blocks.find((b4) => b4.is_active) ?? null;
    if (!activeBlock) {
      return /* @__PURE__ */ u2("div", { class: "card stat-card", children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
        /* @__PURE__ */ u2("div", { class: "stat-label", style: { letterSpacing: "0.08em", fontSize: "11px" }, children: "BILLING BLOCK" }),
        /* @__PURE__ */ u2("div", { class: "stat-value", style: { opacity: 0.4 }, children: "NO ACTIVE BLOCK" }),
        /* @__PURE__ */ u2("div", { class: "stat-sub", children: [
          "7d historical max:",
          " ",
          /* @__PURE__ */ u2("span", { style: { fontFamily: "var(--font-mono)" }, children: fmtTokens(data.historical_max_tokens) }),
          " ",
          "tokens"
        ] })
      ] }) });
    }
    const totalTokens = activeBlock.tokens.input + activeBlock.tokens.output + activeBlock.tokens.cache_read + activeBlock.tokens.cache_creation + activeBlock.tokens.reasoning_output;
    const elapsed = formatDuration(activeBlock.first_timestamp, activeBlock.last_timestamp);
    const blockEnd = fmtUtcTime(activeBlock.end);
    return /* @__PURE__ */ u2("div", { class: "card stat-card", children: [
      /* @__PURE__ */ u2("div", { class: "stat-content", children: [
        /* @__PURE__ */ u2("div", { class: "stat-label", style: { letterSpacing: "0.08em", fontSize: "11px" }, children: "BILLING BLOCK" }),
        /* @__PURE__ */ u2(
          "div",
          {
            class: "stat-value",
            style: { fontFamily: "var(--font-mono)", letterSpacing: "-0.02em" },
            children: fmtTokens(totalTokens)
          }
        ),
        /* @__PURE__ */ u2("div", { class: "stat-sub", children: [
          elapsed,
          " elapsed \xB7 ends ",
          blockEnd,
          " \xB7 ",
          activeBlock.entry_count,
          " entries"
        ] }),
        activeBlock.burn_rate && /* @__PURE__ */ u2("div", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "12px", marginTop: "4px" }, children: [
          "$",
          (activeBlock.burn_rate.cost_per_hour_nanos / 1e9).toFixed(4),
          "/hr",
          activeBlock.burn_rate.tier && /* @__PURE__ */ u2(
            "span",
            {
              style: {
                marginLeft: "6px",
                color: tierColor(activeBlock.burn_rate.tier),
                fontSize: "11px",
                letterSpacing: "0.04em"
              },
              children: tierLabel(activeBlock.burn_rate.tier)
            }
          )
        ] })
      ] }),
      /* @__PURE__ */ u2(QuotaSection, { block: activeBlock })
    ] });
  }

  // src/ui/components/ContextWindowCard.tsx
  function severityToStatus2(s4) {
    if (s4 === "ok") return "success";
    if (s4 === "warn") return "warning";
    return "accent";
  }
  function severityLabel2(s4) {
    return s4 === "ok" ? "[OK]" : s4 === "warn" ? "[WARN]" : "[CRIT]";
  }
  function ContextWindowCard({ data }) {
    if (!data || data.enabled === false) return null;
    if (data.total_input_tokens == null || data.context_window_size == null) return null;
    if (data.context_window_size <= 0) return null;
    const used = data.total_input_tokens;
    const size = data.context_window_size;
    const pct = Math.max(0, Math.min(999, (data.pct ?? used / size) * 100));
    const severity = data.severity ?? "ok";
    return /* @__PURE__ */ u2("div", { class: "card stat-card", children: [
      /* @__PURE__ */ u2("div", { class: "stat-content", children: [
        /* @__PURE__ */ u2("div", { class: "stat-label", style: { letterSpacing: "0.08em", fontSize: "11px" }, children: "CONTEXT WINDOW" }),
        /* @__PURE__ */ u2(
          "div",
          {
            class: "stat-value",
            style: { fontFamily: "var(--font-mono)", letterSpacing: "-0.02em" },
            children: fmt(used)
          }
        ),
        /* @__PURE__ */ u2("div", { class: "stat-sub", children: [
          "of ",
          fmt(size),
          " \xB7 ",
          pct.toFixed(1),
          "%",
          " ",
          /* @__PURE__ */ u2(
            "span",
            {
              style: {
                color: severity === "danger" ? "var(--accent)" : severity === "warn" ? "var(--warning)" : void 0
              },
              children: severityLabel2(severity)
            }
          )
        ] })
      ] }),
      /* @__PURE__ */ u2(
        SegmentedProgressBar,
        {
          value: used,
          max: size,
          status: severityToStatus2(severity),
          "aria-label": "Context window usage"
        }
      )
    ] });
  }

  // src/ui/components/StatsCards.tsx
  function StatsCards({ totals, daily, activeDays, heatmapTotalNanos, cacheEfficiency, billingBlocks, contextWindow }) {
    const rangeLabel = RANGE_LABELS[selectedRange.value].toLowerCase();
    const avgPerActiveDay = (() => {
      if (activeDays === void 0 || activeDays === null) return "--";
      if (activeDays === 0) return "--";
      const totalUsd = (heatmapTotalNanos ?? 0) / 1e9;
      return fmtCost(totalUsd / activeDays);
    })();
    const activeDayTooltip = activeDays !== void 0 && activeDays !== null && activeDays > 0 ? `Averaged over ${activeDays} day${activeDays === 1 ? "" : "s"} with non-zero spend` : "No spend in selected period";
    const stats = [
      { label: "Sessions", value: totals.sessions.toLocaleString(), sub: rangeLabel },
      { label: "Turns", value: fmt(totals.turns), sub: rangeLabel },
      { label: "Input Tokens", value: fmt(totals.input), sub: rangeLabel },
      { label: "Output Tokens", value: fmt(totals.output), sub: rangeLabel },
      { label: "Cached Input", value: fmt(totals.cache_read), sub: "prompt cache" },
      { label: "Cache Creation", value: fmt(totals.cache_creation), sub: "cache writes" },
      { label: "Reasoning", value: fmt(totals.reasoning_output), sub: "subset of output" },
      { label: "Est. Cost", value: fmtCostBig(totals.cost), sub: "API pricing", isCost: true }
    ];
    return /* @__PURE__ */ u2(S, { children: [
      stats.map((s4) => /* @__PURE__ */ u2("div", { class: "card stat-card", children: [
        /* @__PURE__ */ u2("div", { class: "stat-content", children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: s4.label }),
          /* @__PURE__ */ u2("div", { class: `stat-value${s4.isCost ? " cost-value doto-hero" : ""}`, children: s4.value }),
          s4.sub ? /* @__PURE__ */ u2("div", { class: "stat-sub", children: s4.sub }) : null
        ] }),
        s4.isCost && daily && daily.length >= 2 ? /* @__PURE__ */ u2("div", { class: "stat-sparkline", children: /* @__PURE__ */ u2(Sparkline, { daily }) }) : null
      ] }, s4.label)),
      /* @__PURE__ */ u2("div", { class: "card stat-card", title: activeDayTooltip, children: /* @__PURE__ */ u2("div", { class: "stat-content", children: [
        /* @__PURE__ */ u2("div", { class: "stat-label", children: "Avg / Active Day" }),
        /* @__PURE__ */ u2("div", { class: "stat-value", children: avgPerActiveDay }),
        /* @__PURE__ */ u2("div", { class: "stat-sub", children: activeDays !== void 0 && activeDays !== null && activeDays > 0 ? `${activeDays} active ${activeDays === 1 ? "day" : "days"}` : "no spend" })
      ] }) }),
      billingBlocks && /* @__PURE__ */ u2(BillingBlocksCard, { data: billingBlocks }),
      /* @__PURE__ */ u2(ContextWindowCard, { data: contextWindow ?? null }),
      cacheEfficiency && /* @__PURE__ */ u2(CacheEfficiencyCard, { data: cacheEfficiency })
    ] });
  }

  // src/ui/components/SubagentSummary.tsx
  function SubagentSummary({ summary }) {
    if (summary.subagent_turns === 0) return null;
    const totalInput = summary.parent_input + summary.subagent_input;
    const totalOutput = summary.parent_output + summary.subagent_output;
    const subPctInput = totalInput > 0 ? summary.subagent_input / totalInput * 100 : 0;
    const subPctOutput = totalOutput > 0 ? summary.subagent_output / totalOutput * 100 : 0;
    return /* @__PURE__ */ u2("div", { class: "table-card", children: [
      /* @__PURE__ */ u2("div", { class: "section-header", style: { padding: "20px 20px 0" }, children: /* @__PURE__ */ u2("div", { class: "section-title", style: { padding: "0" }, children: "Subagent Breakdown" }) }),
      /* @__PURE__ */ u2("div", { style: "display:grid;grid-template-columns:1fr 1fr 1fr;gap:16px;padding:12px 20px 20px", children: [
        /* @__PURE__ */ u2("div", { children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: "Turns" }),
          /* @__PURE__ */ u2("div", { style: "font-size:15px", children: [
            "Parent: ",
            /* @__PURE__ */ u2("span", { class: "num", children: fmt(summary.parent_turns) })
          ] }),
          /* @__PURE__ */ u2("div", { style: "font-size:15px", children: [
            "Subagent: ",
            /* @__PURE__ */ u2("span", { class: "num", children: fmt(summary.subagent_turns) })
          ] }),
          /* @__PURE__ */ u2("div", { class: "sub", children: [
            summary.unique_agents,
            " unique agents"
          ] })
        ] }),
        /* @__PURE__ */ u2("div", { children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: "Input Tokens" }),
          /* @__PURE__ */ u2("div", { style: "font-size:15px", children: [
            "Parent: ",
            /* @__PURE__ */ u2("span", { class: "num", children: fmt(summary.parent_input) })
          ] }),
          /* @__PURE__ */ u2("div", { style: "font-size:15px", children: [
            "Subagent: ",
            /* @__PURE__ */ u2("span", { class: "num", children: fmt(summary.subagent_input) }),
            " (",
            subPctInput.toFixed(1),
            "%)"
          ] })
        ] }),
        /* @__PURE__ */ u2("div", { children: [
          /* @__PURE__ */ u2("div", { class: "stat-label", children: "Output Tokens" }),
          /* @__PURE__ */ u2("div", { style: "font-size:15px", children: [
            "Parent: ",
            /* @__PURE__ */ u2("span", { class: "num", children: fmt(summary.parent_output) })
          ] }),
          /* @__PURE__ */ u2("div", { style: "font-size:15px", children: [
            "Subagent: ",
            /* @__PURE__ */ u2("span", { class: "num", children: fmt(summary.subagent_output) }),
            " (",
            subPctOutput.toFixed(1),
            "%)"
          ] })
        ] })
      ] })
    ] });
  }

  // node_modules/@tanstack/table-core/build/lib/index.mjs
  function functionalUpdate(updater, input) {
    return typeof updater === "function" ? updater(input) : updater;
  }
  function makeStateUpdater(key, instance) {
    return (updater) => {
      instance.setState((old) => {
        return {
          ...old,
          [key]: functionalUpdate(updater, old[key])
        };
      });
    };
  }
  function isFunction(d5) {
    return d5 instanceof Function;
  }
  function isNumberArray(d5) {
    return Array.isArray(d5) && d5.every((val) => typeof val === "number");
  }
  function flattenBy(arr, getChildren) {
    const flat = [];
    const recurse = (subArr) => {
      subArr.forEach((item) => {
        flat.push(item);
        const children = getChildren(item);
        if (children != null && children.length) {
          recurse(children);
        }
      });
    };
    recurse(arr);
    return flat;
  }
  function memo(getDeps, fn, opts) {
    let deps = [];
    let result;
    return (depArgs) => {
      let depTime;
      if (opts.key && opts.debug) depTime = Date.now();
      const newDeps = getDeps(depArgs);
      const depsChanged = newDeps.length !== deps.length || newDeps.some((dep, index) => deps[index] !== dep);
      if (!depsChanged) {
        return result;
      }
      deps = newDeps;
      let resultTime;
      if (opts.key && opts.debug) resultTime = Date.now();
      result = fn(...newDeps);
      opts == null || opts.onChange == null || opts.onChange(result);
      if (opts.key && opts.debug) {
        if (opts != null && opts.debug()) {
          const depEndTime = Math.round((Date.now() - depTime) * 100) / 100;
          const resultEndTime = Math.round((Date.now() - resultTime) * 100) / 100;
          const resultFpsPercentage = resultEndTime / 16;
          const pad = (str, num) => {
            str = String(str);
            while (str.length < num) {
              str = " " + str;
            }
            return str;
          };
          console.info(`%c\u23F1 ${pad(resultEndTime, 5)} /${pad(depEndTime, 5)} ms`, `
            font-size: .6rem;
            font-weight: bold;
            color: hsl(${Math.max(0, Math.min(120 - 120 * resultFpsPercentage, 120))}deg 100% 31%);`, opts == null ? void 0 : opts.key);
        }
      }
      return result;
    };
  }
  function getMemoOptions(tableOptions, debugLevel, key, onChange) {
    return {
      debug: () => {
        var _tableOptions$debugAl;
        return (_tableOptions$debugAl = tableOptions == null ? void 0 : tableOptions.debugAll) != null ? _tableOptions$debugAl : tableOptions[debugLevel];
      },
      key,
      onChange
    };
  }
  function createCell(table, row, column, columnId) {
    const getRenderValue = () => {
      var _cell$getValue;
      return (_cell$getValue = cell.getValue()) != null ? _cell$getValue : table.options.renderFallbackValue;
    };
    const cell = {
      id: `${row.id}_${column.id}`,
      row,
      column,
      getValue: () => row.getValue(columnId),
      renderValue: getRenderValue,
      getContext: memo(() => [table, column, row, cell], (table2, column2, row2, cell2) => ({
        table: table2,
        column: column2,
        row: row2,
        cell: cell2,
        getValue: cell2.getValue,
        renderValue: cell2.renderValue
      }), getMemoOptions(table.options, "debugCells", "cell.getContext"))
    };
    table._features.forEach((feature) => {
      feature.createCell == null || feature.createCell(cell, column, row, table);
    }, {});
    return cell;
  }
  function createColumn(table, columnDef, depth, parent) {
    var _ref, _resolvedColumnDef$id;
    const defaultColumn = table._getDefaultColumnDef();
    const resolvedColumnDef = {
      ...defaultColumn,
      ...columnDef
    };
    const accessorKey = resolvedColumnDef.accessorKey;
    let id = (_ref = (_resolvedColumnDef$id = resolvedColumnDef.id) != null ? _resolvedColumnDef$id : accessorKey ? typeof String.prototype.replaceAll === "function" ? accessorKey.replaceAll(".", "_") : accessorKey.replace(/\./g, "_") : void 0) != null ? _ref : typeof resolvedColumnDef.header === "string" ? resolvedColumnDef.header : void 0;
    let accessorFn;
    if (resolvedColumnDef.accessorFn) {
      accessorFn = resolvedColumnDef.accessorFn;
    } else if (accessorKey) {
      if (accessorKey.includes(".")) {
        accessorFn = (originalRow) => {
          let result = originalRow;
          for (const key of accessorKey.split(".")) {
            var _result;
            result = (_result = result) == null ? void 0 : _result[key];
            if (result === void 0) {
              console.warn(`"${key}" in deeply nested key "${accessorKey}" returned undefined.`);
            }
          }
          return result;
        };
      } else {
        accessorFn = (originalRow) => originalRow[resolvedColumnDef.accessorKey];
      }
    }
    if (!id) {
      if (true) {
        throw new Error(resolvedColumnDef.accessorFn ? `Columns require an id when using an accessorFn` : `Columns require an id when using a non-string header`);
      }
      throw new Error();
    }
    let column = {
      id: `${String(id)}`,
      accessorFn,
      parent,
      depth,
      columnDef: resolvedColumnDef,
      columns: [],
      getFlatColumns: memo(() => [true], () => {
        var _column$columns;
        return [column, ...(_column$columns = column.columns) == null ? void 0 : _column$columns.flatMap((d5) => d5.getFlatColumns())];
      }, getMemoOptions(table.options, "debugColumns", "column.getFlatColumns")),
      getLeafColumns: memo(() => [table._getOrderColumnsFn()], (orderColumns2) => {
        var _column$columns2;
        if ((_column$columns2 = column.columns) != null && _column$columns2.length) {
          let leafColumns = column.columns.flatMap((column2) => column2.getLeafColumns());
          return orderColumns2(leafColumns);
        }
        return [column];
      }, getMemoOptions(table.options, "debugColumns", "column.getLeafColumns"))
    };
    for (const feature of table._features) {
      feature.createColumn == null || feature.createColumn(column, table);
    }
    return column;
  }
  var debug = "debugHeaders";
  function createHeader(table, column, options) {
    var _options$id;
    const id = (_options$id = options.id) != null ? _options$id : column.id;
    let header = {
      id,
      column,
      index: options.index,
      isPlaceholder: !!options.isPlaceholder,
      placeholderId: options.placeholderId,
      depth: options.depth,
      subHeaders: [],
      colSpan: 0,
      rowSpan: 0,
      headerGroup: null,
      getLeafHeaders: () => {
        const leafHeaders = [];
        const recurseHeader = (h5) => {
          if (h5.subHeaders && h5.subHeaders.length) {
            h5.subHeaders.map(recurseHeader);
          }
          leafHeaders.push(h5);
        };
        recurseHeader(header);
        return leafHeaders;
      },
      getContext: () => ({
        table,
        header,
        column
      })
    };
    table._features.forEach((feature) => {
      feature.createHeader == null || feature.createHeader(header, table);
    });
    return header;
  }
  var Headers = {
    createTable: (table) => {
      table.getHeaderGroups = memo(() => [table.getAllColumns(), table.getVisibleLeafColumns(), table.getState().columnPinning.left, table.getState().columnPinning.right], (allColumns, leafColumns, left, right) => {
        var _left$map$filter, _right$map$filter;
        const leftColumns = (_left$map$filter = left == null ? void 0 : left.map((columnId) => leafColumns.find((d5) => d5.id === columnId)).filter(Boolean)) != null ? _left$map$filter : [];
        const rightColumns = (_right$map$filter = right == null ? void 0 : right.map((columnId) => leafColumns.find((d5) => d5.id === columnId)).filter(Boolean)) != null ? _right$map$filter : [];
        const centerColumns = leafColumns.filter((column) => !(left != null && left.includes(column.id)) && !(right != null && right.includes(column.id)));
        const headerGroups = buildHeaderGroups(allColumns, [...leftColumns, ...centerColumns, ...rightColumns], table);
        return headerGroups;
      }, getMemoOptions(table.options, debug, "getHeaderGroups"));
      table.getCenterHeaderGroups = memo(() => [table.getAllColumns(), table.getVisibleLeafColumns(), table.getState().columnPinning.left, table.getState().columnPinning.right], (allColumns, leafColumns, left, right) => {
        leafColumns = leafColumns.filter((column) => !(left != null && left.includes(column.id)) && !(right != null && right.includes(column.id)));
        return buildHeaderGroups(allColumns, leafColumns, table, "center");
      }, getMemoOptions(table.options, debug, "getCenterHeaderGroups"));
      table.getLeftHeaderGroups = memo(() => [table.getAllColumns(), table.getVisibleLeafColumns(), table.getState().columnPinning.left], (allColumns, leafColumns, left) => {
        var _left$map$filter2;
        const orderedLeafColumns = (_left$map$filter2 = left == null ? void 0 : left.map((columnId) => leafColumns.find((d5) => d5.id === columnId)).filter(Boolean)) != null ? _left$map$filter2 : [];
        return buildHeaderGroups(allColumns, orderedLeafColumns, table, "left");
      }, getMemoOptions(table.options, debug, "getLeftHeaderGroups"));
      table.getRightHeaderGroups = memo(() => [table.getAllColumns(), table.getVisibleLeafColumns(), table.getState().columnPinning.right], (allColumns, leafColumns, right) => {
        var _right$map$filter2;
        const orderedLeafColumns = (_right$map$filter2 = right == null ? void 0 : right.map((columnId) => leafColumns.find((d5) => d5.id === columnId)).filter(Boolean)) != null ? _right$map$filter2 : [];
        return buildHeaderGroups(allColumns, orderedLeafColumns, table, "right");
      }, getMemoOptions(table.options, debug, "getRightHeaderGroups"));
      table.getFooterGroups = memo(() => [table.getHeaderGroups()], (headerGroups) => {
        return [...headerGroups].reverse();
      }, getMemoOptions(table.options, debug, "getFooterGroups"));
      table.getLeftFooterGroups = memo(() => [table.getLeftHeaderGroups()], (headerGroups) => {
        return [...headerGroups].reverse();
      }, getMemoOptions(table.options, debug, "getLeftFooterGroups"));
      table.getCenterFooterGroups = memo(() => [table.getCenterHeaderGroups()], (headerGroups) => {
        return [...headerGroups].reverse();
      }, getMemoOptions(table.options, debug, "getCenterFooterGroups"));
      table.getRightFooterGroups = memo(() => [table.getRightHeaderGroups()], (headerGroups) => {
        return [...headerGroups].reverse();
      }, getMemoOptions(table.options, debug, "getRightFooterGroups"));
      table.getFlatHeaders = memo(() => [table.getHeaderGroups()], (headerGroups) => {
        return headerGroups.map((headerGroup) => {
          return headerGroup.headers;
        }).flat();
      }, getMemoOptions(table.options, debug, "getFlatHeaders"));
      table.getLeftFlatHeaders = memo(() => [table.getLeftHeaderGroups()], (left) => {
        return left.map((headerGroup) => {
          return headerGroup.headers;
        }).flat();
      }, getMemoOptions(table.options, debug, "getLeftFlatHeaders"));
      table.getCenterFlatHeaders = memo(() => [table.getCenterHeaderGroups()], (left) => {
        return left.map((headerGroup) => {
          return headerGroup.headers;
        }).flat();
      }, getMemoOptions(table.options, debug, "getCenterFlatHeaders"));
      table.getRightFlatHeaders = memo(() => [table.getRightHeaderGroups()], (left) => {
        return left.map((headerGroup) => {
          return headerGroup.headers;
        }).flat();
      }, getMemoOptions(table.options, debug, "getRightFlatHeaders"));
      table.getCenterLeafHeaders = memo(() => [table.getCenterFlatHeaders()], (flatHeaders) => {
        return flatHeaders.filter((header) => {
          var _header$subHeaders;
          return !((_header$subHeaders = header.subHeaders) != null && _header$subHeaders.length);
        });
      }, getMemoOptions(table.options, debug, "getCenterLeafHeaders"));
      table.getLeftLeafHeaders = memo(() => [table.getLeftFlatHeaders()], (flatHeaders) => {
        return flatHeaders.filter((header) => {
          var _header$subHeaders2;
          return !((_header$subHeaders2 = header.subHeaders) != null && _header$subHeaders2.length);
        });
      }, getMemoOptions(table.options, debug, "getLeftLeafHeaders"));
      table.getRightLeafHeaders = memo(() => [table.getRightFlatHeaders()], (flatHeaders) => {
        return flatHeaders.filter((header) => {
          var _header$subHeaders3;
          return !((_header$subHeaders3 = header.subHeaders) != null && _header$subHeaders3.length);
        });
      }, getMemoOptions(table.options, debug, "getRightLeafHeaders"));
      table.getLeafHeaders = memo(() => [table.getLeftHeaderGroups(), table.getCenterHeaderGroups(), table.getRightHeaderGroups()], (left, center, right) => {
        var _left$0$headers, _left$, _center$0$headers, _center$, _right$0$headers, _right$;
        return [...(_left$0$headers = (_left$ = left[0]) == null ? void 0 : _left$.headers) != null ? _left$0$headers : [], ...(_center$0$headers = (_center$ = center[0]) == null ? void 0 : _center$.headers) != null ? _center$0$headers : [], ...(_right$0$headers = (_right$ = right[0]) == null ? void 0 : _right$.headers) != null ? _right$0$headers : []].map((header) => {
          return header.getLeafHeaders();
        }).flat();
      }, getMemoOptions(table.options, debug, "getLeafHeaders"));
    }
  };
  function buildHeaderGroups(allColumns, columnsToGroup, table, headerFamily) {
    var _headerGroups$0$heade, _headerGroups$;
    let maxDepth = 0;
    const findMaxDepth = function(columns4, depth) {
      if (depth === void 0) {
        depth = 1;
      }
      maxDepth = Math.max(maxDepth, depth);
      columns4.filter((column) => column.getIsVisible()).forEach((column) => {
        var _column$columns;
        if ((_column$columns = column.columns) != null && _column$columns.length) {
          findMaxDepth(column.columns, depth + 1);
        }
      }, 0);
    };
    findMaxDepth(allColumns);
    let headerGroups = [];
    const createHeaderGroup = (headersToGroup, depth) => {
      const headerGroup = {
        depth,
        id: [headerFamily, `${depth}`].filter(Boolean).join("_"),
        headers: []
      };
      const pendingParentHeaders = [];
      headersToGroup.forEach((headerToGroup) => {
        const latestPendingParentHeader = [...pendingParentHeaders].reverse()[0];
        const isLeafHeader = headerToGroup.column.depth === headerGroup.depth;
        let column;
        let isPlaceholder = false;
        if (isLeafHeader && headerToGroup.column.parent) {
          column = headerToGroup.column.parent;
        } else {
          column = headerToGroup.column;
          isPlaceholder = true;
        }
        if (latestPendingParentHeader && (latestPendingParentHeader == null ? void 0 : latestPendingParentHeader.column) === column) {
          latestPendingParentHeader.subHeaders.push(headerToGroup);
        } else {
          const header = createHeader(table, column, {
            id: [headerFamily, depth, column.id, headerToGroup == null ? void 0 : headerToGroup.id].filter(Boolean).join("_"),
            isPlaceholder,
            placeholderId: isPlaceholder ? `${pendingParentHeaders.filter((d5) => d5.column === column).length}` : void 0,
            depth,
            index: pendingParentHeaders.length
          });
          header.subHeaders.push(headerToGroup);
          pendingParentHeaders.push(header);
        }
        headerGroup.headers.push(headerToGroup);
        headerToGroup.headerGroup = headerGroup;
      });
      headerGroups.push(headerGroup);
      if (depth > 0) {
        createHeaderGroup(pendingParentHeaders, depth - 1);
      }
    };
    const bottomHeaders = columnsToGroup.map((column, index) => createHeader(table, column, {
      depth: maxDepth,
      index
    }));
    createHeaderGroup(bottomHeaders, maxDepth - 1);
    headerGroups.reverse();
    const recurseHeadersForSpans = (headers) => {
      const filteredHeaders = headers.filter((header) => header.column.getIsVisible());
      return filteredHeaders.map((header) => {
        let colSpan = 0;
        let rowSpan = 0;
        let childRowSpans = [0];
        if (header.subHeaders && header.subHeaders.length) {
          childRowSpans = [];
          recurseHeadersForSpans(header.subHeaders).forEach((_ref) => {
            let {
              colSpan: childColSpan,
              rowSpan: childRowSpan
            } = _ref;
            colSpan += childColSpan;
            childRowSpans.push(childRowSpan);
          });
        } else {
          colSpan = 1;
        }
        const minChildRowSpan = Math.min(...childRowSpans);
        rowSpan = rowSpan + minChildRowSpan;
        header.colSpan = colSpan;
        header.rowSpan = rowSpan;
        return {
          colSpan,
          rowSpan
        };
      });
    };
    recurseHeadersForSpans((_headerGroups$0$heade = (_headerGroups$ = headerGroups[0]) == null ? void 0 : _headerGroups$.headers) != null ? _headerGroups$0$heade : []);
    return headerGroups;
  }
  var createRow = (table, id, original, rowIndex, depth, subRows, parentId) => {
    let row = {
      id,
      index: rowIndex,
      original,
      depth,
      parentId,
      _valuesCache: {},
      _uniqueValuesCache: {},
      getValue: (columnId) => {
        if (row._valuesCache.hasOwnProperty(columnId)) {
          return row._valuesCache[columnId];
        }
        const column = table.getColumn(columnId);
        if (!(column != null && column.accessorFn)) {
          return void 0;
        }
        row._valuesCache[columnId] = column.accessorFn(row.original, rowIndex);
        return row._valuesCache[columnId];
      },
      getUniqueValues: (columnId) => {
        if (row._uniqueValuesCache.hasOwnProperty(columnId)) {
          return row._uniqueValuesCache[columnId];
        }
        const column = table.getColumn(columnId);
        if (!(column != null && column.accessorFn)) {
          return void 0;
        }
        if (!column.columnDef.getUniqueValues) {
          row._uniqueValuesCache[columnId] = [row.getValue(columnId)];
          return row._uniqueValuesCache[columnId];
        }
        row._uniqueValuesCache[columnId] = column.columnDef.getUniqueValues(row.original, rowIndex);
        return row._uniqueValuesCache[columnId];
      },
      renderValue: (columnId) => {
        var _row$getValue;
        return (_row$getValue = row.getValue(columnId)) != null ? _row$getValue : table.options.renderFallbackValue;
      },
      subRows: subRows != null ? subRows : [],
      getLeafRows: () => flattenBy(row.subRows, (d5) => d5.subRows),
      getParentRow: () => row.parentId ? table.getRow(row.parentId, true) : void 0,
      getParentRows: () => {
        let parentRows = [];
        let currentRow = row;
        while (true) {
          const parentRow = currentRow.getParentRow();
          if (!parentRow) break;
          parentRows.push(parentRow);
          currentRow = parentRow;
        }
        return parentRows.reverse();
      },
      getAllCells: memo(() => [table.getAllLeafColumns()], (leafColumns) => {
        return leafColumns.map((column) => {
          return createCell(table, row, column, column.id);
        });
      }, getMemoOptions(table.options, "debugRows", "getAllCells")),
      _getAllCellsByColumnId: memo(() => [row.getAllCells()], (allCells) => {
        return allCells.reduce((acc, cell) => {
          acc[cell.column.id] = cell;
          return acc;
        }, {});
      }, getMemoOptions(table.options, "debugRows", "getAllCellsByColumnId"))
    };
    for (let i4 = 0; i4 < table._features.length; i4++) {
      const feature = table._features[i4];
      feature == null || feature.createRow == null || feature.createRow(row, table);
    }
    return row;
  };
  var ColumnFaceting = {
    createColumn: (column, table) => {
      column._getFacetedRowModel = table.options.getFacetedRowModel && table.options.getFacetedRowModel(table, column.id);
      column.getFacetedRowModel = () => {
        if (!column._getFacetedRowModel) {
          return table.getPreFilteredRowModel();
        }
        return column._getFacetedRowModel();
      };
      column._getFacetedUniqueValues = table.options.getFacetedUniqueValues && table.options.getFacetedUniqueValues(table, column.id);
      column.getFacetedUniqueValues = () => {
        if (!column._getFacetedUniqueValues) {
          return /* @__PURE__ */ new Map();
        }
        return column._getFacetedUniqueValues();
      };
      column._getFacetedMinMaxValues = table.options.getFacetedMinMaxValues && table.options.getFacetedMinMaxValues(table, column.id);
      column.getFacetedMinMaxValues = () => {
        if (!column._getFacetedMinMaxValues) {
          return void 0;
        }
        return column._getFacetedMinMaxValues();
      };
    }
  };
  var includesString = (row, columnId, filterValue) => {
    var _filterValue$toString, _row$getValue;
    const search = filterValue == null || (_filterValue$toString = filterValue.toString()) == null ? void 0 : _filterValue$toString.toLowerCase();
    return Boolean((_row$getValue = row.getValue(columnId)) == null || (_row$getValue = _row$getValue.toString()) == null || (_row$getValue = _row$getValue.toLowerCase()) == null ? void 0 : _row$getValue.includes(search));
  };
  includesString.autoRemove = (val) => testFalsey(val);
  var includesStringSensitive = (row, columnId, filterValue) => {
    var _row$getValue2;
    return Boolean((_row$getValue2 = row.getValue(columnId)) == null || (_row$getValue2 = _row$getValue2.toString()) == null ? void 0 : _row$getValue2.includes(filterValue));
  };
  includesStringSensitive.autoRemove = (val) => testFalsey(val);
  var equalsString = (row, columnId, filterValue) => {
    var _row$getValue3;
    return ((_row$getValue3 = row.getValue(columnId)) == null || (_row$getValue3 = _row$getValue3.toString()) == null ? void 0 : _row$getValue3.toLowerCase()) === (filterValue == null ? void 0 : filterValue.toLowerCase());
  };
  equalsString.autoRemove = (val) => testFalsey(val);
  var arrIncludes = (row, columnId, filterValue) => {
    var _row$getValue4;
    return (_row$getValue4 = row.getValue(columnId)) == null ? void 0 : _row$getValue4.includes(filterValue);
  };
  arrIncludes.autoRemove = (val) => testFalsey(val);
  var arrIncludesAll = (row, columnId, filterValue) => {
    return !filterValue.some((val) => {
      var _row$getValue5;
      return !((_row$getValue5 = row.getValue(columnId)) != null && _row$getValue5.includes(val));
    });
  };
  arrIncludesAll.autoRemove = (val) => testFalsey(val) || !(val != null && val.length);
  var arrIncludesSome = (row, columnId, filterValue) => {
    return filterValue.some((val) => {
      var _row$getValue6;
      return (_row$getValue6 = row.getValue(columnId)) == null ? void 0 : _row$getValue6.includes(val);
    });
  };
  arrIncludesSome.autoRemove = (val) => testFalsey(val) || !(val != null && val.length);
  var equals = (row, columnId, filterValue) => {
    return row.getValue(columnId) === filterValue;
  };
  equals.autoRemove = (val) => testFalsey(val);
  var weakEquals = (row, columnId, filterValue) => {
    return row.getValue(columnId) == filterValue;
  };
  weakEquals.autoRemove = (val) => testFalsey(val);
  var inNumberRange = (row, columnId, filterValue) => {
    let [min2, max2] = filterValue;
    const rowValue = row.getValue(columnId);
    return rowValue >= min2 && rowValue <= max2;
  };
  inNumberRange.resolveFilterValue = (val) => {
    let [unsafeMin, unsafeMax] = val;
    let parsedMin = typeof unsafeMin !== "number" ? parseFloat(unsafeMin) : unsafeMin;
    let parsedMax = typeof unsafeMax !== "number" ? parseFloat(unsafeMax) : unsafeMax;
    let min2 = unsafeMin === null || Number.isNaN(parsedMin) ? -Infinity : parsedMin;
    let max2 = unsafeMax === null || Number.isNaN(parsedMax) ? Infinity : parsedMax;
    if (min2 > max2) {
      const temp = min2;
      min2 = max2;
      max2 = temp;
    }
    return [min2, max2];
  };
  inNumberRange.autoRemove = (val) => testFalsey(val) || testFalsey(val[0]) && testFalsey(val[1]);
  var filterFns = {
    includesString,
    includesStringSensitive,
    equalsString,
    arrIncludes,
    arrIncludesAll,
    arrIncludesSome,
    equals,
    weakEquals,
    inNumberRange
  };
  function testFalsey(val) {
    return val === void 0 || val === null || val === "";
  }
  var ColumnFiltering = {
    getDefaultColumnDef: () => {
      return {
        filterFn: "auto"
      };
    },
    getInitialState: (state) => {
      return {
        columnFilters: [],
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        onColumnFiltersChange: makeStateUpdater("columnFilters", table),
        filterFromLeafRows: false,
        maxLeafRowFilterDepth: 100
      };
    },
    createColumn: (column, table) => {
      column.getAutoFilterFn = () => {
        const firstRow = table.getCoreRowModel().flatRows[0];
        const value = firstRow == null ? void 0 : firstRow.getValue(column.id);
        if (typeof value === "string") {
          return filterFns.includesString;
        }
        if (typeof value === "number") {
          return filterFns.inNumberRange;
        }
        if (typeof value === "boolean") {
          return filterFns.equals;
        }
        if (value !== null && typeof value === "object") {
          return filterFns.equals;
        }
        if (Array.isArray(value)) {
          return filterFns.arrIncludes;
        }
        return filterFns.weakEquals;
      };
      column.getFilterFn = () => {
        var _table$options$filter, _table$options$filter2;
        return isFunction(column.columnDef.filterFn) ? column.columnDef.filterFn : column.columnDef.filterFn === "auto" ? column.getAutoFilterFn() : (
          // @ts-ignore
          (_table$options$filter = (_table$options$filter2 = table.options.filterFns) == null ? void 0 : _table$options$filter2[column.columnDef.filterFn]) != null ? _table$options$filter : filterFns[column.columnDef.filterFn]
        );
      };
      column.getCanFilter = () => {
        var _column$columnDef$ena, _table$options$enable, _table$options$enable2;
        return ((_column$columnDef$ena = column.columnDef.enableColumnFilter) != null ? _column$columnDef$ena : true) && ((_table$options$enable = table.options.enableColumnFilters) != null ? _table$options$enable : true) && ((_table$options$enable2 = table.options.enableFilters) != null ? _table$options$enable2 : true) && !!column.accessorFn;
      };
      column.getIsFiltered = () => column.getFilterIndex() > -1;
      column.getFilterValue = () => {
        var _table$getState$colum;
        return (_table$getState$colum = table.getState().columnFilters) == null || (_table$getState$colum = _table$getState$colum.find((d5) => d5.id === column.id)) == null ? void 0 : _table$getState$colum.value;
      };
      column.getFilterIndex = () => {
        var _table$getState$colum2, _table$getState$colum3;
        return (_table$getState$colum2 = (_table$getState$colum3 = table.getState().columnFilters) == null ? void 0 : _table$getState$colum3.findIndex((d5) => d5.id === column.id)) != null ? _table$getState$colum2 : -1;
      };
      column.setFilterValue = (value) => {
        table.setColumnFilters((old) => {
          const filterFn = column.getFilterFn();
          const previousFilter = old == null ? void 0 : old.find((d5) => d5.id === column.id);
          const newFilter = functionalUpdate(value, previousFilter ? previousFilter.value : void 0);
          if (shouldAutoRemoveFilter(filterFn, newFilter, column)) {
            var _old$filter;
            return (_old$filter = old == null ? void 0 : old.filter((d5) => d5.id !== column.id)) != null ? _old$filter : [];
          }
          const newFilterObj = {
            id: column.id,
            value: newFilter
          };
          if (previousFilter) {
            var _old$map;
            return (_old$map = old == null ? void 0 : old.map((d5) => {
              if (d5.id === column.id) {
                return newFilterObj;
              }
              return d5;
            })) != null ? _old$map : [];
          }
          if (old != null && old.length) {
            return [...old, newFilterObj];
          }
          return [newFilterObj];
        });
      };
    },
    createRow: (row, _table) => {
      row.columnFilters = {};
      row.columnFiltersMeta = {};
    },
    createTable: (table) => {
      table.setColumnFilters = (updater) => {
        const leafColumns = table.getAllLeafColumns();
        const updateFn = (old) => {
          var _functionalUpdate;
          return (_functionalUpdate = functionalUpdate(updater, old)) == null ? void 0 : _functionalUpdate.filter((filter) => {
            const column = leafColumns.find((d5) => d5.id === filter.id);
            if (column) {
              const filterFn = column.getFilterFn();
              if (shouldAutoRemoveFilter(filterFn, filter.value, column)) {
                return false;
              }
            }
            return true;
          });
        };
        table.options.onColumnFiltersChange == null || table.options.onColumnFiltersChange(updateFn);
      };
      table.resetColumnFilters = (defaultState) => {
        var _table$initialState$c, _table$initialState;
        table.setColumnFilters(defaultState ? [] : (_table$initialState$c = (_table$initialState = table.initialState) == null ? void 0 : _table$initialState.columnFilters) != null ? _table$initialState$c : []);
      };
      table.getPreFilteredRowModel = () => table.getCoreRowModel();
      table.getFilteredRowModel = () => {
        if (!table._getFilteredRowModel && table.options.getFilteredRowModel) {
          table._getFilteredRowModel = table.options.getFilteredRowModel(table);
        }
        if (table.options.manualFiltering || !table._getFilteredRowModel) {
          return table.getPreFilteredRowModel();
        }
        return table._getFilteredRowModel();
      };
    }
  };
  function shouldAutoRemoveFilter(filterFn, value, column) {
    return (filterFn && filterFn.autoRemove ? filterFn.autoRemove(value, column) : false) || typeof value === "undefined" || typeof value === "string" && !value;
  }
  var sum = (columnId, _leafRows, childRows) => {
    return childRows.reduce((sum2, next) => {
      const nextValue = next.getValue(columnId);
      return sum2 + (typeof nextValue === "number" ? nextValue : 0);
    }, 0);
  };
  var min = (columnId, _leafRows, childRows) => {
    let min2;
    childRows.forEach((row) => {
      const value = row.getValue(columnId);
      if (value != null && (min2 > value || min2 === void 0 && value >= value)) {
        min2 = value;
      }
    });
    return min2;
  };
  var max = (columnId, _leafRows, childRows) => {
    let max2;
    childRows.forEach((row) => {
      const value = row.getValue(columnId);
      if (value != null && (max2 < value || max2 === void 0 && value >= value)) {
        max2 = value;
      }
    });
    return max2;
  };
  var extent = (columnId, _leafRows, childRows) => {
    let min2;
    let max2;
    childRows.forEach((row) => {
      const value = row.getValue(columnId);
      if (value != null) {
        if (min2 === void 0) {
          if (value >= value) min2 = max2 = value;
        } else {
          if (min2 > value) min2 = value;
          if (max2 < value) max2 = value;
        }
      }
    });
    return [min2, max2];
  };
  var mean = (columnId, leafRows) => {
    let count2 = 0;
    let sum2 = 0;
    leafRows.forEach((row) => {
      let value = row.getValue(columnId);
      if (value != null && (value = +value) >= value) {
        ++count2, sum2 += value;
      }
    });
    if (count2) return sum2 / count2;
    return;
  };
  var median = (columnId, leafRows) => {
    if (!leafRows.length) {
      return;
    }
    const values = leafRows.map((row) => row.getValue(columnId));
    if (!isNumberArray(values)) {
      return;
    }
    if (values.length === 1) {
      return values[0];
    }
    const mid = Math.floor(values.length / 2);
    const nums = values.sort((a4, b4) => a4 - b4);
    return values.length % 2 !== 0 ? nums[mid] : (nums[mid - 1] + nums[mid]) / 2;
  };
  var unique = (columnId, leafRows) => {
    return Array.from(new Set(leafRows.map((d5) => d5.getValue(columnId))).values());
  };
  var uniqueCount = (columnId, leafRows) => {
    return new Set(leafRows.map((d5) => d5.getValue(columnId))).size;
  };
  var count = (_columnId, leafRows) => {
    return leafRows.length;
  };
  var aggregationFns = {
    sum,
    min,
    max,
    extent,
    mean,
    median,
    unique,
    uniqueCount,
    count
  };
  var ColumnGrouping = {
    getDefaultColumnDef: () => {
      return {
        aggregatedCell: (props) => {
          var _toString, _props$getValue;
          return (_toString = (_props$getValue = props.getValue()) == null || _props$getValue.toString == null ? void 0 : _props$getValue.toString()) != null ? _toString : null;
        },
        aggregationFn: "auto"
      };
    },
    getInitialState: (state) => {
      return {
        grouping: [],
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        onGroupingChange: makeStateUpdater("grouping", table),
        groupedColumnMode: "reorder"
      };
    },
    createColumn: (column, table) => {
      column.toggleGrouping = () => {
        table.setGrouping((old) => {
          if (old != null && old.includes(column.id)) {
            return old.filter((d5) => d5 !== column.id);
          }
          return [...old != null ? old : [], column.id];
        });
      };
      column.getCanGroup = () => {
        var _column$columnDef$ena, _table$options$enable;
        return ((_column$columnDef$ena = column.columnDef.enableGrouping) != null ? _column$columnDef$ena : true) && ((_table$options$enable = table.options.enableGrouping) != null ? _table$options$enable : true) && (!!column.accessorFn || !!column.columnDef.getGroupingValue);
      };
      column.getIsGrouped = () => {
        var _table$getState$group;
        return (_table$getState$group = table.getState().grouping) == null ? void 0 : _table$getState$group.includes(column.id);
      };
      column.getGroupedIndex = () => {
        var _table$getState$group2;
        return (_table$getState$group2 = table.getState().grouping) == null ? void 0 : _table$getState$group2.indexOf(column.id);
      };
      column.getToggleGroupingHandler = () => {
        const canGroup = column.getCanGroup();
        return () => {
          if (!canGroup) return;
          column.toggleGrouping();
        };
      };
      column.getAutoAggregationFn = () => {
        const firstRow = table.getCoreRowModel().flatRows[0];
        const value = firstRow == null ? void 0 : firstRow.getValue(column.id);
        if (typeof value === "number") {
          return aggregationFns.sum;
        }
        if (Object.prototype.toString.call(value) === "[object Date]") {
          return aggregationFns.extent;
        }
      };
      column.getAggregationFn = () => {
        var _table$options$aggreg, _table$options$aggreg2;
        if (!column) {
          throw new Error();
        }
        return isFunction(column.columnDef.aggregationFn) ? column.columnDef.aggregationFn : column.columnDef.aggregationFn === "auto" ? column.getAutoAggregationFn() : (_table$options$aggreg = (_table$options$aggreg2 = table.options.aggregationFns) == null ? void 0 : _table$options$aggreg2[column.columnDef.aggregationFn]) != null ? _table$options$aggreg : aggregationFns[column.columnDef.aggregationFn];
      };
    },
    createTable: (table) => {
      table.setGrouping = (updater) => table.options.onGroupingChange == null ? void 0 : table.options.onGroupingChange(updater);
      table.resetGrouping = (defaultState) => {
        var _table$initialState$g, _table$initialState;
        table.setGrouping(defaultState ? [] : (_table$initialState$g = (_table$initialState = table.initialState) == null ? void 0 : _table$initialState.grouping) != null ? _table$initialState$g : []);
      };
      table.getPreGroupedRowModel = () => table.getFilteredRowModel();
      table.getGroupedRowModel = () => {
        if (!table._getGroupedRowModel && table.options.getGroupedRowModel) {
          table._getGroupedRowModel = table.options.getGroupedRowModel(table);
        }
        if (table.options.manualGrouping || !table._getGroupedRowModel) {
          return table.getPreGroupedRowModel();
        }
        return table._getGroupedRowModel();
      };
    },
    createRow: (row, table) => {
      row.getIsGrouped = () => !!row.groupingColumnId;
      row.getGroupingValue = (columnId) => {
        if (row._groupingValuesCache.hasOwnProperty(columnId)) {
          return row._groupingValuesCache[columnId];
        }
        const column = table.getColumn(columnId);
        if (!(column != null && column.columnDef.getGroupingValue)) {
          return row.getValue(columnId);
        }
        row._groupingValuesCache[columnId] = column.columnDef.getGroupingValue(row.original);
        return row._groupingValuesCache[columnId];
      };
      row._groupingValuesCache = {};
    },
    createCell: (cell, column, row, table) => {
      cell.getIsGrouped = () => column.getIsGrouped() && column.id === row.groupingColumnId;
      cell.getIsPlaceholder = () => !cell.getIsGrouped() && column.getIsGrouped();
      cell.getIsAggregated = () => {
        var _row$subRows;
        return !cell.getIsGrouped() && !cell.getIsPlaceholder() && !!((_row$subRows = row.subRows) != null && _row$subRows.length);
      };
    }
  };
  function orderColumns(leafColumns, grouping, groupedColumnMode) {
    if (!(grouping != null && grouping.length) || !groupedColumnMode) {
      return leafColumns;
    }
    const nonGroupingColumns = leafColumns.filter((col) => !grouping.includes(col.id));
    if (groupedColumnMode === "remove") {
      return nonGroupingColumns;
    }
    const groupingColumns = grouping.map((g4) => leafColumns.find((col) => col.id === g4)).filter(Boolean);
    return [...groupingColumns, ...nonGroupingColumns];
  }
  var ColumnOrdering = {
    getInitialState: (state) => {
      return {
        columnOrder: [],
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        onColumnOrderChange: makeStateUpdater("columnOrder", table)
      };
    },
    createColumn: (column, table) => {
      column.getIndex = memo((position) => [_getVisibleLeafColumns(table, position)], (columns4) => columns4.findIndex((d5) => d5.id === column.id), getMemoOptions(table.options, "debugColumns", "getIndex"));
      column.getIsFirstColumn = (position) => {
        var _columns$;
        const columns4 = _getVisibleLeafColumns(table, position);
        return ((_columns$ = columns4[0]) == null ? void 0 : _columns$.id) === column.id;
      };
      column.getIsLastColumn = (position) => {
        var _columns;
        const columns4 = _getVisibleLeafColumns(table, position);
        return ((_columns = columns4[columns4.length - 1]) == null ? void 0 : _columns.id) === column.id;
      };
    },
    createTable: (table) => {
      table.setColumnOrder = (updater) => table.options.onColumnOrderChange == null ? void 0 : table.options.onColumnOrderChange(updater);
      table.resetColumnOrder = (defaultState) => {
        var _table$initialState$c;
        table.setColumnOrder(defaultState ? [] : (_table$initialState$c = table.initialState.columnOrder) != null ? _table$initialState$c : []);
      };
      table._getOrderColumnsFn = memo(() => [table.getState().columnOrder, table.getState().grouping, table.options.groupedColumnMode], (columnOrder, grouping, groupedColumnMode) => (columns4) => {
        let orderedColumns = [];
        if (!(columnOrder != null && columnOrder.length)) {
          orderedColumns = columns4;
        } else {
          const columnOrderCopy = [...columnOrder];
          const columnsCopy = [...columns4];
          while (columnsCopy.length && columnOrderCopy.length) {
            const targetColumnId = columnOrderCopy.shift();
            const foundIndex = columnsCopy.findIndex((d5) => d5.id === targetColumnId);
            if (foundIndex > -1) {
              orderedColumns.push(columnsCopy.splice(foundIndex, 1)[0]);
            }
          }
          orderedColumns = [...orderedColumns, ...columnsCopy];
        }
        return orderColumns(orderedColumns, grouping, groupedColumnMode);
      }, getMemoOptions(table.options, "debugTable", "_getOrderColumnsFn"));
    }
  };
  var getDefaultColumnPinningState = () => ({
    left: [],
    right: []
  });
  var ColumnPinning = {
    getInitialState: (state) => {
      return {
        columnPinning: getDefaultColumnPinningState(),
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        onColumnPinningChange: makeStateUpdater("columnPinning", table)
      };
    },
    createColumn: (column, table) => {
      column.pin = (position) => {
        const columnIds = column.getLeafColumns().map((d5) => d5.id).filter(Boolean);
        table.setColumnPinning((old) => {
          var _old$left3, _old$right3;
          if (position === "right") {
            var _old$left, _old$right;
            return {
              left: ((_old$left = old == null ? void 0 : old.left) != null ? _old$left : []).filter((d5) => !(columnIds != null && columnIds.includes(d5))),
              right: [...((_old$right = old == null ? void 0 : old.right) != null ? _old$right : []).filter((d5) => !(columnIds != null && columnIds.includes(d5))), ...columnIds]
            };
          }
          if (position === "left") {
            var _old$left2, _old$right2;
            return {
              left: [...((_old$left2 = old == null ? void 0 : old.left) != null ? _old$left2 : []).filter((d5) => !(columnIds != null && columnIds.includes(d5))), ...columnIds],
              right: ((_old$right2 = old == null ? void 0 : old.right) != null ? _old$right2 : []).filter((d5) => !(columnIds != null && columnIds.includes(d5)))
            };
          }
          return {
            left: ((_old$left3 = old == null ? void 0 : old.left) != null ? _old$left3 : []).filter((d5) => !(columnIds != null && columnIds.includes(d5))),
            right: ((_old$right3 = old == null ? void 0 : old.right) != null ? _old$right3 : []).filter((d5) => !(columnIds != null && columnIds.includes(d5)))
          };
        });
      };
      column.getCanPin = () => {
        const leafColumns = column.getLeafColumns();
        return leafColumns.some((d5) => {
          var _d$columnDef$enablePi, _ref, _table$options$enable;
          return ((_d$columnDef$enablePi = d5.columnDef.enablePinning) != null ? _d$columnDef$enablePi : true) && ((_ref = (_table$options$enable = table.options.enableColumnPinning) != null ? _table$options$enable : table.options.enablePinning) != null ? _ref : true);
        });
      };
      column.getIsPinned = () => {
        const leafColumnIds = column.getLeafColumns().map((d5) => d5.id);
        const {
          left,
          right
        } = table.getState().columnPinning;
        const isLeft = leafColumnIds.some((d5) => left == null ? void 0 : left.includes(d5));
        const isRight = leafColumnIds.some((d5) => right == null ? void 0 : right.includes(d5));
        return isLeft ? "left" : isRight ? "right" : false;
      };
      column.getPinnedIndex = () => {
        var _table$getState$colum, _table$getState$colum2;
        const position = column.getIsPinned();
        return position ? (_table$getState$colum = (_table$getState$colum2 = table.getState().columnPinning) == null || (_table$getState$colum2 = _table$getState$colum2[position]) == null ? void 0 : _table$getState$colum2.indexOf(column.id)) != null ? _table$getState$colum : -1 : 0;
      };
    },
    createRow: (row, table) => {
      row.getCenterVisibleCells = memo(() => [row._getAllVisibleCells(), table.getState().columnPinning.left, table.getState().columnPinning.right], (allCells, left, right) => {
        const leftAndRight = [...left != null ? left : [], ...right != null ? right : []];
        return allCells.filter((d5) => !leftAndRight.includes(d5.column.id));
      }, getMemoOptions(table.options, "debugRows", "getCenterVisibleCells"));
      row.getLeftVisibleCells = memo(() => [row._getAllVisibleCells(), table.getState().columnPinning.left], (allCells, left) => {
        const cells = (left != null ? left : []).map((columnId) => allCells.find((cell) => cell.column.id === columnId)).filter(Boolean).map((d5) => ({
          ...d5,
          position: "left"
        }));
        return cells;
      }, getMemoOptions(table.options, "debugRows", "getLeftVisibleCells"));
      row.getRightVisibleCells = memo(() => [row._getAllVisibleCells(), table.getState().columnPinning.right], (allCells, right) => {
        const cells = (right != null ? right : []).map((columnId) => allCells.find((cell) => cell.column.id === columnId)).filter(Boolean).map((d5) => ({
          ...d5,
          position: "right"
        }));
        return cells;
      }, getMemoOptions(table.options, "debugRows", "getRightVisibleCells"));
    },
    createTable: (table) => {
      table.setColumnPinning = (updater) => table.options.onColumnPinningChange == null ? void 0 : table.options.onColumnPinningChange(updater);
      table.resetColumnPinning = (defaultState) => {
        var _table$initialState$c, _table$initialState;
        return table.setColumnPinning(defaultState ? getDefaultColumnPinningState() : (_table$initialState$c = (_table$initialState = table.initialState) == null ? void 0 : _table$initialState.columnPinning) != null ? _table$initialState$c : getDefaultColumnPinningState());
      };
      table.getIsSomeColumnsPinned = (position) => {
        var _pinningState$positio;
        const pinningState = table.getState().columnPinning;
        if (!position) {
          var _pinningState$left, _pinningState$right;
          return Boolean(((_pinningState$left = pinningState.left) == null ? void 0 : _pinningState$left.length) || ((_pinningState$right = pinningState.right) == null ? void 0 : _pinningState$right.length));
        }
        return Boolean((_pinningState$positio = pinningState[position]) == null ? void 0 : _pinningState$positio.length);
      };
      table.getLeftLeafColumns = memo(() => [table.getAllLeafColumns(), table.getState().columnPinning.left], (allColumns, left) => {
        return (left != null ? left : []).map((columnId) => allColumns.find((column) => column.id === columnId)).filter(Boolean);
      }, getMemoOptions(table.options, "debugColumns", "getLeftLeafColumns"));
      table.getRightLeafColumns = memo(() => [table.getAllLeafColumns(), table.getState().columnPinning.right], (allColumns, right) => {
        return (right != null ? right : []).map((columnId) => allColumns.find((column) => column.id === columnId)).filter(Boolean);
      }, getMemoOptions(table.options, "debugColumns", "getRightLeafColumns"));
      table.getCenterLeafColumns = memo(() => [table.getAllLeafColumns(), table.getState().columnPinning.left, table.getState().columnPinning.right], (allColumns, left, right) => {
        const leftAndRight = [...left != null ? left : [], ...right != null ? right : []];
        return allColumns.filter((d5) => !leftAndRight.includes(d5.id));
      }, getMemoOptions(table.options, "debugColumns", "getCenterLeafColumns"));
    }
  };
  function safelyAccessDocument(_document) {
    return _document || (typeof document !== "undefined" ? document : null);
  }
  var defaultColumnSizing = {
    size: 150,
    minSize: 20,
    maxSize: Number.MAX_SAFE_INTEGER
  };
  var getDefaultColumnSizingInfoState = () => ({
    startOffset: null,
    startSize: null,
    deltaOffset: null,
    deltaPercentage: null,
    isResizingColumn: false,
    columnSizingStart: []
  });
  var ColumnSizing = {
    getDefaultColumnDef: () => {
      return defaultColumnSizing;
    },
    getInitialState: (state) => {
      return {
        columnSizing: {},
        columnSizingInfo: getDefaultColumnSizingInfoState(),
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        columnResizeMode: "onEnd",
        columnResizeDirection: "ltr",
        onColumnSizingChange: makeStateUpdater("columnSizing", table),
        onColumnSizingInfoChange: makeStateUpdater("columnSizingInfo", table)
      };
    },
    createColumn: (column, table) => {
      column.getSize = () => {
        var _column$columnDef$min, _ref, _column$columnDef$max;
        const columnSize = table.getState().columnSizing[column.id];
        return Math.min(Math.max((_column$columnDef$min = column.columnDef.minSize) != null ? _column$columnDef$min : defaultColumnSizing.minSize, (_ref = columnSize != null ? columnSize : column.columnDef.size) != null ? _ref : defaultColumnSizing.size), (_column$columnDef$max = column.columnDef.maxSize) != null ? _column$columnDef$max : defaultColumnSizing.maxSize);
      };
      column.getStart = memo((position) => [position, _getVisibleLeafColumns(table, position), table.getState().columnSizing], (position, columns4) => columns4.slice(0, column.getIndex(position)).reduce((sum2, column2) => sum2 + column2.getSize(), 0), getMemoOptions(table.options, "debugColumns", "getStart"));
      column.getAfter = memo((position) => [position, _getVisibleLeafColumns(table, position), table.getState().columnSizing], (position, columns4) => columns4.slice(column.getIndex(position) + 1).reduce((sum2, column2) => sum2 + column2.getSize(), 0), getMemoOptions(table.options, "debugColumns", "getAfter"));
      column.resetSize = () => {
        table.setColumnSizing((_ref2) => {
          let {
            [column.id]: _4,
            ...rest
          } = _ref2;
          return rest;
        });
      };
      column.getCanResize = () => {
        var _column$columnDef$ena, _table$options$enable;
        return ((_column$columnDef$ena = column.columnDef.enableResizing) != null ? _column$columnDef$ena : true) && ((_table$options$enable = table.options.enableColumnResizing) != null ? _table$options$enable : true);
      };
      column.getIsResizing = () => {
        return table.getState().columnSizingInfo.isResizingColumn === column.id;
      };
    },
    createHeader: (header, table) => {
      header.getSize = () => {
        let sum2 = 0;
        const recurse = (header2) => {
          if (header2.subHeaders.length) {
            header2.subHeaders.forEach(recurse);
          } else {
            var _header$column$getSiz;
            sum2 += (_header$column$getSiz = header2.column.getSize()) != null ? _header$column$getSiz : 0;
          }
        };
        recurse(header);
        return sum2;
      };
      header.getStart = () => {
        if (header.index > 0) {
          const prevSiblingHeader = header.headerGroup.headers[header.index - 1];
          return prevSiblingHeader.getStart() + prevSiblingHeader.getSize();
        }
        return 0;
      };
      header.getResizeHandler = (_contextDocument) => {
        const column = table.getColumn(header.column.id);
        const canResize = column == null ? void 0 : column.getCanResize();
        return (e4) => {
          if (!column || !canResize) {
            return;
          }
          e4.persist == null || e4.persist();
          if (isTouchStartEvent(e4)) {
            if (e4.touches && e4.touches.length > 1) {
              return;
            }
          }
          const startSize = header.getSize();
          const columnSizingStart = header ? header.getLeafHeaders().map((d5) => [d5.column.id, d5.column.getSize()]) : [[column.id, column.getSize()]];
          const clientX = isTouchStartEvent(e4) ? Math.round(e4.touches[0].clientX) : e4.clientX;
          const newColumnSizing = {};
          const updateOffset = (eventType, clientXPos) => {
            if (typeof clientXPos !== "number") {
              return;
            }
            table.setColumnSizingInfo((old) => {
              var _old$startOffset, _old$startSize;
              const deltaDirection = table.options.columnResizeDirection === "rtl" ? -1 : 1;
              const deltaOffset = (clientXPos - ((_old$startOffset = old == null ? void 0 : old.startOffset) != null ? _old$startOffset : 0)) * deltaDirection;
              const deltaPercentage = Math.max(deltaOffset / ((_old$startSize = old == null ? void 0 : old.startSize) != null ? _old$startSize : 0), -0.999999);
              old.columnSizingStart.forEach((_ref3) => {
                let [columnId, headerSize] = _ref3;
                newColumnSizing[columnId] = Math.round(Math.max(headerSize + headerSize * deltaPercentage, 0) * 100) / 100;
              });
              return {
                ...old,
                deltaOffset,
                deltaPercentage
              };
            });
            if (table.options.columnResizeMode === "onChange" || eventType === "end") {
              table.setColumnSizing((old) => ({
                ...old,
                ...newColumnSizing
              }));
            }
          };
          const onMove = (clientXPos) => updateOffset("move", clientXPos);
          const onEnd = (clientXPos) => {
            updateOffset("end", clientXPos);
            table.setColumnSizingInfo((old) => ({
              ...old,
              isResizingColumn: false,
              startOffset: null,
              startSize: null,
              deltaOffset: null,
              deltaPercentage: null,
              columnSizingStart: []
            }));
          };
          const contextDocument = safelyAccessDocument(_contextDocument);
          const mouseEvents = {
            moveHandler: (e5) => onMove(e5.clientX),
            upHandler: (e5) => {
              contextDocument == null || contextDocument.removeEventListener("mousemove", mouseEvents.moveHandler);
              contextDocument == null || contextDocument.removeEventListener("mouseup", mouseEvents.upHandler);
              onEnd(e5.clientX);
            }
          };
          const touchEvents = {
            moveHandler: (e5) => {
              if (e5.cancelable) {
                e5.preventDefault();
                e5.stopPropagation();
              }
              onMove(e5.touches[0].clientX);
              return false;
            },
            upHandler: (e5) => {
              var _e$touches$;
              contextDocument == null || contextDocument.removeEventListener("touchmove", touchEvents.moveHandler);
              contextDocument == null || contextDocument.removeEventListener("touchend", touchEvents.upHandler);
              if (e5.cancelable) {
                e5.preventDefault();
                e5.stopPropagation();
              }
              onEnd((_e$touches$ = e5.touches[0]) == null ? void 0 : _e$touches$.clientX);
            }
          };
          const passiveIfSupported = passiveEventSupported() ? {
            passive: false
          } : false;
          if (isTouchStartEvent(e4)) {
            contextDocument == null || contextDocument.addEventListener("touchmove", touchEvents.moveHandler, passiveIfSupported);
            contextDocument == null || contextDocument.addEventListener("touchend", touchEvents.upHandler, passiveIfSupported);
          } else {
            contextDocument == null || contextDocument.addEventListener("mousemove", mouseEvents.moveHandler, passiveIfSupported);
            contextDocument == null || contextDocument.addEventListener("mouseup", mouseEvents.upHandler, passiveIfSupported);
          }
          table.setColumnSizingInfo((old) => ({
            ...old,
            startOffset: clientX,
            startSize,
            deltaOffset: 0,
            deltaPercentage: 0,
            columnSizingStart,
            isResizingColumn: column.id
          }));
        };
      };
    },
    createTable: (table) => {
      table.setColumnSizing = (updater) => table.options.onColumnSizingChange == null ? void 0 : table.options.onColumnSizingChange(updater);
      table.setColumnSizingInfo = (updater) => table.options.onColumnSizingInfoChange == null ? void 0 : table.options.onColumnSizingInfoChange(updater);
      table.resetColumnSizing = (defaultState) => {
        var _table$initialState$c;
        table.setColumnSizing(defaultState ? {} : (_table$initialState$c = table.initialState.columnSizing) != null ? _table$initialState$c : {});
      };
      table.resetHeaderSizeInfo = (defaultState) => {
        var _table$initialState$c2;
        table.setColumnSizingInfo(defaultState ? getDefaultColumnSizingInfoState() : (_table$initialState$c2 = table.initialState.columnSizingInfo) != null ? _table$initialState$c2 : getDefaultColumnSizingInfoState());
      };
      table.getTotalSize = () => {
        var _table$getHeaderGroup, _table$getHeaderGroup2;
        return (_table$getHeaderGroup = (_table$getHeaderGroup2 = table.getHeaderGroups()[0]) == null ? void 0 : _table$getHeaderGroup2.headers.reduce((sum2, header) => {
          return sum2 + header.getSize();
        }, 0)) != null ? _table$getHeaderGroup : 0;
      };
      table.getLeftTotalSize = () => {
        var _table$getLeftHeaderG, _table$getLeftHeaderG2;
        return (_table$getLeftHeaderG = (_table$getLeftHeaderG2 = table.getLeftHeaderGroups()[0]) == null ? void 0 : _table$getLeftHeaderG2.headers.reduce((sum2, header) => {
          return sum2 + header.getSize();
        }, 0)) != null ? _table$getLeftHeaderG : 0;
      };
      table.getCenterTotalSize = () => {
        var _table$getCenterHeade, _table$getCenterHeade2;
        return (_table$getCenterHeade = (_table$getCenterHeade2 = table.getCenterHeaderGroups()[0]) == null ? void 0 : _table$getCenterHeade2.headers.reduce((sum2, header) => {
          return sum2 + header.getSize();
        }, 0)) != null ? _table$getCenterHeade : 0;
      };
      table.getRightTotalSize = () => {
        var _table$getRightHeader, _table$getRightHeader2;
        return (_table$getRightHeader = (_table$getRightHeader2 = table.getRightHeaderGroups()[0]) == null ? void 0 : _table$getRightHeader2.headers.reduce((sum2, header) => {
          return sum2 + header.getSize();
        }, 0)) != null ? _table$getRightHeader : 0;
      };
    }
  };
  var passiveSupported = null;
  function passiveEventSupported() {
    if (typeof passiveSupported === "boolean") return passiveSupported;
    let supported = false;
    try {
      const options = {
        get passive() {
          supported = true;
          return false;
        }
      };
      const noop = () => {
      };
      window.addEventListener("test", noop, options);
      window.removeEventListener("test", noop);
    } catch (err) {
      supported = false;
    }
    passiveSupported = supported;
    return passiveSupported;
  }
  function isTouchStartEvent(e4) {
    return e4.type === "touchstart";
  }
  var ColumnVisibility = {
    getInitialState: (state) => {
      return {
        columnVisibility: {},
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        onColumnVisibilityChange: makeStateUpdater("columnVisibility", table)
      };
    },
    createColumn: (column, table) => {
      column.toggleVisibility = (value) => {
        if (column.getCanHide()) {
          table.setColumnVisibility((old) => ({
            ...old,
            [column.id]: value != null ? value : !column.getIsVisible()
          }));
        }
      };
      column.getIsVisible = () => {
        var _ref, _table$getState$colum;
        const childColumns = column.columns;
        return (_ref = childColumns.length ? childColumns.some((c4) => c4.getIsVisible()) : (_table$getState$colum = table.getState().columnVisibility) == null ? void 0 : _table$getState$colum[column.id]) != null ? _ref : true;
      };
      column.getCanHide = () => {
        var _column$columnDef$ena, _table$options$enable;
        return ((_column$columnDef$ena = column.columnDef.enableHiding) != null ? _column$columnDef$ena : true) && ((_table$options$enable = table.options.enableHiding) != null ? _table$options$enable : true);
      };
      column.getToggleVisibilityHandler = () => {
        return (e4) => {
          column.toggleVisibility == null || column.toggleVisibility(e4.target.checked);
        };
      };
    },
    createRow: (row, table) => {
      row._getAllVisibleCells = memo(() => [row.getAllCells(), table.getState().columnVisibility], (cells) => {
        return cells.filter((cell) => cell.column.getIsVisible());
      }, getMemoOptions(table.options, "debugRows", "_getAllVisibleCells"));
      row.getVisibleCells = memo(() => [row.getLeftVisibleCells(), row.getCenterVisibleCells(), row.getRightVisibleCells()], (left, center, right) => [...left, ...center, ...right], getMemoOptions(table.options, "debugRows", "getVisibleCells"));
    },
    createTable: (table) => {
      const makeVisibleColumnsMethod = (key, getColumns) => {
        return memo(() => [getColumns(), getColumns().filter((d5) => d5.getIsVisible()).map((d5) => d5.id).join("_")], (columns4) => {
          return columns4.filter((d5) => d5.getIsVisible == null ? void 0 : d5.getIsVisible());
        }, getMemoOptions(table.options, "debugColumns", key));
      };
      table.getVisibleFlatColumns = makeVisibleColumnsMethod("getVisibleFlatColumns", () => table.getAllFlatColumns());
      table.getVisibleLeafColumns = makeVisibleColumnsMethod("getVisibleLeafColumns", () => table.getAllLeafColumns());
      table.getLeftVisibleLeafColumns = makeVisibleColumnsMethod("getLeftVisibleLeafColumns", () => table.getLeftLeafColumns());
      table.getRightVisibleLeafColumns = makeVisibleColumnsMethod("getRightVisibleLeafColumns", () => table.getRightLeafColumns());
      table.getCenterVisibleLeafColumns = makeVisibleColumnsMethod("getCenterVisibleLeafColumns", () => table.getCenterLeafColumns());
      table.setColumnVisibility = (updater) => table.options.onColumnVisibilityChange == null ? void 0 : table.options.onColumnVisibilityChange(updater);
      table.resetColumnVisibility = (defaultState) => {
        var _table$initialState$c;
        table.setColumnVisibility(defaultState ? {} : (_table$initialState$c = table.initialState.columnVisibility) != null ? _table$initialState$c : {});
      };
      table.toggleAllColumnsVisible = (value) => {
        var _value;
        value = (_value = value) != null ? _value : !table.getIsAllColumnsVisible();
        table.setColumnVisibility(table.getAllLeafColumns().reduce((obj, column) => ({
          ...obj,
          [column.id]: !value ? !(column.getCanHide != null && column.getCanHide()) : value
        }), {}));
      };
      table.getIsAllColumnsVisible = () => !table.getAllLeafColumns().some((column) => !(column.getIsVisible != null && column.getIsVisible()));
      table.getIsSomeColumnsVisible = () => table.getAllLeafColumns().some((column) => column.getIsVisible == null ? void 0 : column.getIsVisible());
      table.getToggleAllColumnsVisibilityHandler = () => {
        return (e4) => {
          var _target;
          table.toggleAllColumnsVisible((_target = e4.target) == null ? void 0 : _target.checked);
        };
      };
    }
  };
  function _getVisibleLeafColumns(table, position) {
    return !position ? table.getVisibleLeafColumns() : position === "center" ? table.getCenterVisibleLeafColumns() : position === "left" ? table.getLeftVisibleLeafColumns() : table.getRightVisibleLeafColumns();
  }
  var GlobalFaceting = {
    createTable: (table) => {
      table._getGlobalFacetedRowModel = table.options.getFacetedRowModel && table.options.getFacetedRowModel(table, "__global__");
      table.getGlobalFacetedRowModel = () => {
        if (table.options.manualFiltering || !table._getGlobalFacetedRowModel) {
          return table.getPreFilteredRowModel();
        }
        return table._getGlobalFacetedRowModel();
      };
      table._getGlobalFacetedUniqueValues = table.options.getFacetedUniqueValues && table.options.getFacetedUniqueValues(table, "__global__");
      table.getGlobalFacetedUniqueValues = () => {
        if (!table._getGlobalFacetedUniqueValues) {
          return /* @__PURE__ */ new Map();
        }
        return table._getGlobalFacetedUniqueValues();
      };
      table._getGlobalFacetedMinMaxValues = table.options.getFacetedMinMaxValues && table.options.getFacetedMinMaxValues(table, "__global__");
      table.getGlobalFacetedMinMaxValues = () => {
        if (!table._getGlobalFacetedMinMaxValues) {
          return;
        }
        return table._getGlobalFacetedMinMaxValues();
      };
    }
  };
  var GlobalFiltering = {
    getInitialState: (state) => {
      return {
        globalFilter: void 0,
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        onGlobalFilterChange: makeStateUpdater("globalFilter", table),
        globalFilterFn: "auto",
        getColumnCanGlobalFilter: (column) => {
          var _table$getCoreRowMode;
          const value = (_table$getCoreRowMode = table.getCoreRowModel().flatRows[0]) == null || (_table$getCoreRowMode = _table$getCoreRowMode._getAllCellsByColumnId()[column.id]) == null ? void 0 : _table$getCoreRowMode.getValue();
          return typeof value === "string" || typeof value === "number";
        }
      };
    },
    createColumn: (column, table) => {
      column.getCanGlobalFilter = () => {
        var _column$columnDef$ena, _table$options$enable, _table$options$enable2, _table$options$getCol;
        return ((_column$columnDef$ena = column.columnDef.enableGlobalFilter) != null ? _column$columnDef$ena : true) && ((_table$options$enable = table.options.enableGlobalFilter) != null ? _table$options$enable : true) && ((_table$options$enable2 = table.options.enableFilters) != null ? _table$options$enable2 : true) && ((_table$options$getCol = table.options.getColumnCanGlobalFilter == null ? void 0 : table.options.getColumnCanGlobalFilter(column)) != null ? _table$options$getCol : true) && !!column.accessorFn;
      };
    },
    createTable: (table) => {
      table.getGlobalAutoFilterFn = () => {
        return filterFns.includesString;
      };
      table.getGlobalFilterFn = () => {
        var _table$options$filter, _table$options$filter2;
        const {
          globalFilterFn
        } = table.options;
        return isFunction(globalFilterFn) ? globalFilterFn : globalFilterFn === "auto" ? table.getGlobalAutoFilterFn() : (_table$options$filter = (_table$options$filter2 = table.options.filterFns) == null ? void 0 : _table$options$filter2[globalFilterFn]) != null ? _table$options$filter : filterFns[globalFilterFn];
      };
      table.setGlobalFilter = (updater) => {
        table.options.onGlobalFilterChange == null || table.options.onGlobalFilterChange(updater);
      };
      table.resetGlobalFilter = (defaultState) => {
        table.setGlobalFilter(defaultState ? void 0 : table.initialState.globalFilter);
      };
    }
  };
  var RowExpanding = {
    getInitialState: (state) => {
      return {
        expanded: {},
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        onExpandedChange: makeStateUpdater("expanded", table),
        paginateExpandedRows: true
      };
    },
    createTable: (table) => {
      let registered = false;
      let queued = false;
      table._autoResetExpanded = () => {
        var _ref, _table$options$autoRe;
        if (!registered) {
          table._queue(() => {
            registered = true;
          });
          return;
        }
        if ((_ref = (_table$options$autoRe = table.options.autoResetAll) != null ? _table$options$autoRe : table.options.autoResetExpanded) != null ? _ref : !table.options.manualExpanding) {
          if (queued) return;
          queued = true;
          table._queue(() => {
            table.resetExpanded();
            queued = false;
          });
        }
      };
      table.setExpanded = (updater) => table.options.onExpandedChange == null ? void 0 : table.options.onExpandedChange(updater);
      table.toggleAllRowsExpanded = (expanded) => {
        if (expanded != null ? expanded : !table.getIsAllRowsExpanded()) {
          table.setExpanded(true);
        } else {
          table.setExpanded({});
        }
      };
      table.resetExpanded = (defaultState) => {
        var _table$initialState$e, _table$initialState;
        table.setExpanded(defaultState ? {} : (_table$initialState$e = (_table$initialState = table.initialState) == null ? void 0 : _table$initialState.expanded) != null ? _table$initialState$e : {});
      };
      table.getCanSomeRowsExpand = () => {
        return table.getPrePaginationRowModel().flatRows.some((row) => row.getCanExpand());
      };
      table.getToggleAllRowsExpandedHandler = () => {
        return (e4) => {
          e4.persist == null || e4.persist();
          table.toggleAllRowsExpanded();
        };
      };
      table.getIsSomeRowsExpanded = () => {
        const expanded = table.getState().expanded;
        return expanded === true || Object.values(expanded).some(Boolean);
      };
      table.getIsAllRowsExpanded = () => {
        const expanded = table.getState().expanded;
        if (typeof expanded === "boolean") {
          return expanded === true;
        }
        if (!Object.keys(expanded).length) {
          return false;
        }
        if (table.getRowModel().flatRows.some((row) => !row.getIsExpanded())) {
          return false;
        }
        return true;
      };
      table.getExpandedDepth = () => {
        let maxDepth = 0;
        const rowIds = table.getState().expanded === true ? Object.keys(table.getRowModel().rowsById) : Object.keys(table.getState().expanded);
        rowIds.forEach((id) => {
          const splitId = id.split(".");
          maxDepth = Math.max(maxDepth, splitId.length);
        });
        return maxDepth;
      };
      table.getPreExpandedRowModel = () => table.getSortedRowModel();
      table.getExpandedRowModel = () => {
        if (!table._getExpandedRowModel && table.options.getExpandedRowModel) {
          table._getExpandedRowModel = table.options.getExpandedRowModel(table);
        }
        if (table.options.manualExpanding || !table._getExpandedRowModel) {
          return table.getPreExpandedRowModel();
        }
        return table._getExpandedRowModel();
      };
    },
    createRow: (row, table) => {
      row.toggleExpanded = (expanded) => {
        table.setExpanded((old) => {
          var _expanded;
          const exists = old === true ? true : !!(old != null && old[row.id]);
          let oldExpanded = {};
          if (old === true) {
            Object.keys(table.getRowModel().rowsById).forEach((rowId) => {
              oldExpanded[rowId] = true;
            });
          } else {
            oldExpanded = old;
          }
          expanded = (_expanded = expanded) != null ? _expanded : !exists;
          if (!exists && expanded) {
            return {
              ...oldExpanded,
              [row.id]: true
            };
          }
          if (exists && !expanded) {
            const {
              [row.id]: _4,
              ...rest
            } = oldExpanded;
            return rest;
          }
          return old;
        });
      };
      row.getIsExpanded = () => {
        var _table$options$getIsR;
        const expanded = table.getState().expanded;
        return !!((_table$options$getIsR = table.options.getIsRowExpanded == null ? void 0 : table.options.getIsRowExpanded(row)) != null ? _table$options$getIsR : expanded === true || (expanded == null ? void 0 : expanded[row.id]));
      };
      row.getCanExpand = () => {
        var _table$options$getRow, _table$options$enable, _row$subRows;
        return (_table$options$getRow = table.options.getRowCanExpand == null ? void 0 : table.options.getRowCanExpand(row)) != null ? _table$options$getRow : ((_table$options$enable = table.options.enableExpanding) != null ? _table$options$enable : true) && !!((_row$subRows = row.subRows) != null && _row$subRows.length);
      };
      row.getIsAllParentsExpanded = () => {
        let isFullyExpanded = true;
        let currentRow = row;
        while (isFullyExpanded && currentRow.parentId) {
          currentRow = table.getRow(currentRow.parentId, true);
          isFullyExpanded = currentRow.getIsExpanded();
        }
        return isFullyExpanded;
      };
      row.getToggleExpandedHandler = () => {
        const canExpand = row.getCanExpand();
        return () => {
          if (!canExpand) return;
          row.toggleExpanded();
        };
      };
    }
  };
  var defaultPageIndex = 0;
  var defaultPageSize = 10;
  var getDefaultPaginationState = () => ({
    pageIndex: defaultPageIndex,
    pageSize: defaultPageSize
  });
  var RowPagination = {
    getInitialState: (state) => {
      return {
        ...state,
        pagination: {
          ...getDefaultPaginationState(),
          ...state == null ? void 0 : state.pagination
        }
      };
    },
    getDefaultOptions: (table) => {
      return {
        onPaginationChange: makeStateUpdater("pagination", table)
      };
    },
    createTable: (table) => {
      let registered = false;
      let queued = false;
      table._autoResetPageIndex = () => {
        var _ref, _table$options$autoRe;
        if (!registered) {
          table._queue(() => {
            registered = true;
          });
          return;
        }
        if ((_ref = (_table$options$autoRe = table.options.autoResetAll) != null ? _table$options$autoRe : table.options.autoResetPageIndex) != null ? _ref : !table.options.manualPagination) {
          if (queued) return;
          queued = true;
          table._queue(() => {
            table.resetPageIndex();
            queued = false;
          });
        }
      };
      table.setPagination = (updater) => {
        const safeUpdater = (old) => {
          let newState = functionalUpdate(updater, old);
          return newState;
        };
        return table.options.onPaginationChange == null ? void 0 : table.options.onPaginationChange(safeUpdater);
      };
      table.resetPagination = (defaultState) => {
        var _table$initialState$p;
        table.setPagination(defaultState ? getDefaultPaginationState() : (_table$initialState$p = table.initialState.pagination) != null ? _table$initialState$p : getDefaultPaginationState());
      };
      table.setPageIndex = (updater) => {
        table.setPagination((old) => {
          let pageIndex = functionalUpdate(updater, old.pageIndex);
          const maxPageIndex = typeof table.options.pageCount === "undefined" || table.options.pageCount === -1 ? Number.MAX_SAFE_INTEGER : table.options.pageCount - 1;
          pageIndex = Math.max(0, Math.min(pageIndex, maxPageIndex));
          return {
            ...old,
            pageIndex
          };
        });
      };
      table.resetPageIndex = (defaultState) => {
        var _table$initialState$p2, _table$initialState;
        table.setPageIndex(defaultState ? defaultPageIndex : (_table$initialState$p2 = (_table$initialState = table.initialState) == null || (_table$initialState = _table$initialState.pagination) == null ? void 0 : _table$initialState.pageIndex) != null ? _table$initialState$p2 : defaultPageIndex);
      };
      table.resetPageSize = (defaultState) => {
        var _table$initialState$p3, _table$initialState2;
        table.setPageSize(defaultState ? defaultPageSize : (_table$initialState$p3 = (_table$initialState2 = table.initialState) == null || (_table$initialState2 = _table$initialState2.pagination) == null ? void 0 : _table$initialState2.pageSize) != null ? _table$initialState$p3 : defaultPageSize);
      };
      table.setPageSize = (updater) => {
        table.setPagination((old) => {
          const pageSize = Math.max(1, functionalUpdate(updater, old.pageSize));
          const topRowIndex = old.pageSize * old.pageIndex;
          const pageIndex = Math.floor(topRowIndex / pageSize);
          return {
            ...old,
            pageIndex,
            pageSize
          };
        });
      };
      table.setPageCount = (updater) => table.setPagination((old) => {
        var _table$options$pageCo;
        let newPageCount = functionalUpdate(updater, (_table$options$pageCo = table.options.pageCount) != null ? _table$options$pageCo : -1);
        if (typeof newPageCount === "number") {
          newPageCount = Math.max(-1, newPageCount);
        }
        return {
          ...old,
          pageCount: newPageCount
        };
      });
      table.getPageOptions = memo(() => [table.getPageCount()], (pageCount) => {
        let pageOptions = [];
        if (pageCount && pageCount > 0) {
          pageOptions = [...new Array(pageCount)].fill(null).map((_4, i4) => i4);
        }
        return pageOptions;
      }, getMemoOptions(table.options, "debugTable", "getPageOptions"));
      table.getCanPreviousPage = () => table.getState().pagination.pageIndex > 0;
      table.getCanNextPage = () => {
        const {
          pageIndex
        } = table.getState().pagination;
        const pageCount = table.getPageCount();
        if (pageCount === -1) {
          return true;
        }
        if (pageCount === 0) {
          return false;
        }
        return pageIndex < pageCount - 1;
      };
      table.previousPage = () => {
        return table.setPageIndex((old) => old - 1);
      };
      table.nextPage = () => {
        return table.setPageIndex((old) => {
          return old + 1;
        });
      };
      table.firstPage = () => {
        return table.setPageIndex(0);
      };
      table.lastPage = () => {
        return table.setPageIndex(table.getPageCount() - 1);
      };
      table.getPrePaginationRowModel = () => table.getExpandedRowModel();
      table.getPaginationRowModel = () => {
        if (!table._getPaginationRowModel && table.options.getPaginationRowModel) {
          table._getPaginationRowModel = table.options.getPaginationRowModel(table);
        }
        if (table.options.manualPagination || !table._getPaginationRowModel) {
          return table.getPrePaginationRowModel();
        }
        return table._getPaginationRowModel();
      };
      table.getPageCount = () => {
        var _table$options$pageCo2;
        return (_table$options$pageCo2 = table.options.pageCount) != null ? _table$options$pageCo2 : Math.ceil(table.getRowCount() / table.getState().pagination.pageSize);
      };
      table.getRowCount = () => {
        var _table$options$rowCou;
        return (_table$options$rowCou = table.options.rowCount) != null ? _table$options$rowCou : table.getPrePaginationRowModel().rows.length;
      };
    }
  };
  var getDefaultRowPinningState = () => ({
    top: [],
    bottom: []
  });
  var RowPinning = {
    getInitialState: (state) => {
      return {
        rowPinning: getDefaultRowPinningState(),
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        onRowPinningChange: makeStateUpdater("rowPinning", table)
      };
    },
    createRow: (row, table) => {
      row.pin = (position, includeLeafRows, includeParentRows) => {
        const leafRowIds = includeLeafRows ? row.getLeafRows().map((_ref) => {
          let {
            id
          } = _ref;
          return id;
        }) : [];
        const parentRowIds = includeParentRows ? row.getParentRows().map((_ref2) => {
          let {
            id
          } = _ref2;
          return id;
        }) : [];
        const rowIds = /* @__PURE__ */ new Set([...parentRowIds, row.id, ...leafRowIds]);
        table.setRowPinning((old) => {
          var _old$top3, _old$bottom3;
          if (position === "bottom") {
            var _old$top, _old$bottom;
            return {
              top: ((_old$top = old == null ? void 0 : old.top) != null ? _old$top : []).filter((d5) => !(rowIds != null && rowIds.has(d5))),
              bottom: [...((_old$bottom = old == null ? void 0 : old.bottom) != null ? _old$bottom : []).filter((d5) => !(rowIds != null && rowIds.has(d5))), ...Array.from(rowIds)]
            };
          }
          if (position === "top") {
            var _old$top2, _old$bottom2;
            return {
              top: [...((_old$top2 = old == null ? void 0 : old.top) != null ? _old$top2 : []).filter((d5) => !(rowIds != null && rowIds.has(d5))), ...Array.from(rowIds)],
              bottom: ((_old$bottom2 = old == null ? void 0 : old.bottom) != null ? _old$bottom2 : []).filter((d5) => !(rowIds != null && rowIds.has(d5)))
            };
          }
          return {
            top: ((_old$top3 = old == null ? void 0 : old.top) != null ? _old$top3 : []).filter((d5) => !(rowIds != null && rowIds.has(d5))),
            bottom: ((_old$bottom3 = old == null ? void 0 : old.bottom) != null ? _old$bottom3 : []).filter((d5) => !(rowIds != null && rowIds.has(d5)))
          };
        });
      };
      row.getCanPin = () => {
        var _ref3;
        const {
          enableRowPinning,
          enablePinning
        } = table.options;
        if (typeof enableRowPinning === "function") {
          return enableRowPinning(row);
        }
        return (_ref3 = enableRowPinning != null ? enableRowPinning : enablePinning) != null ? _ref3 : true;
      };
      row.getIsPinned = () => {
        const rowIds = [row.id];
        const {
          top,
          bottom
        } = table.getState().rowPinning;
        const isTop = rowIds.some((d5) => top == null ? void 0 : top.includes(d5));
        const isBottom = rowIds.some((d5) => bottom == null ? void 0 : bottom.includes(d5));
        return isTop ? "top" : isBottom ? "bottom" : false;
      };
      row.getPinnedIndex = () => {
        var _ref4, _visiblePinnedRowIds$;
        const position = row.getIsPinned();
        if (!position) return -1;
        const visiblePinnedRowIds = (_ref4 = position === "top" ? table.getTopRows() : table.getBottomRows()) == null ? void 0 : _ref4.map((_ref5) => {
          let {
            id
          } = _ref5;
          return id;
        });
        return (_visiblePinnedRowIds$ = visiblePinnedRowIds == null ? void 0 : visiblePinnedRowIds.indexOf(row.id)) != null ? _visiblePinnedRowIds$ : -1;
      };
    },
    createTable: (table) => {
      table.setRowPinning = (updater) => table.options.onRowPinningChange == null ? void 0 : table.options.onRowPinningChange(updater);
      table.resetRowPinning = (defaultState) => {
        var _table$initialState$r, _table$initialState;
        return table.setRowPinning(defaultState ? getDefaultRowPinningState() : (_table$initialState$r = (_table$initialState = table.initialState) == null ? void 0 : _table$initialState.rowPinning) != null ? _table$initialState$r : getDefaultRowPinningState());
      };
      table.getIsSomeRowsPinned = (position) => {
        var _pinningState$positio;
        const pinningState = table.getState().rowPinning;
        if (!position) {
          var _pinningState$top, _pinningState$bottom;
          return Boolean(((_pinningState$top = pinningState.top) == null ? void 0 : _pinningState$top.length) || ((_pinningState$bottom = pinningState.bottom) == null ? void 0 : _pinningState$bottom.length));
        }
        return Boolean((_pinningState$positio = pinningState[position]) == null ? void 0 : _pinningState$positio.length);
      };
      table._getPinnedRows = (visibleRows, pinnedRowIds, position) => {
        var _table$options$keepPi;
        const rows = ((_table$options$keepPi = table.options.keepPinnedRows) != null ? _table$options$keepPi : true) ? (
          //get all rows that are pinned even if they would not be otherwise visible
          //account for expanded parent rows, but not pagination or filtering
          (pinnedRowIds != null ? pinnedRowIds : []).map((rowId) => {
            const row = table.getRow(rowId, true);
            return row.getIsAllParentsExpanded() ? row : null;
          })
        ) : (
          //else get only visible rows that are pinned
          (pinnedRowIds != null ? pinnedRowIds : []).map((rowId) => visibleRows.find((row) => row.id === rowId))
        );
        return rows.filter(Boolean).map((d5) => ({
          ...d5,
          position
        }));
      };
      table.getTopRows = memo(() => [table.getRowModel().rows, table.getState().rowPinning.top], (allRows, topPinnedRowIds) => table._getPinnedRows(allRows, topPinnedRowIds, "top"), getMemoOptions(table.options, "debugRows", "getTopRows"));
      table.getBottomRows = memo(() => [table.getRowModel().rows, table.getState().rowPinning.bottom], (allRows, bottomPinnedRowIds) => table._getPinnedRows(allRows, bottomPinnedRowIds, "bottom"), getMemoOptions(table.options, "debugRows", "getBottomRows"));
      table.getCenterRows = memo(() => [table.getRowModel().rows, table.getState().rowPinning.top, table.getState().rowPinning.bottom], (allRows, top, bottom) => {
        const topAndBottom = /* @__PURE__ */ new Set([...top != null ? top : [], ...bottom != null ? bottom : []]);
        return allRows.filter((d5) => !topAndBottom.has(d5.id));
      }, getMemoOptions(table.options, "debugRows", "getCenterRows"));
    }
  };
  var RowSelection = {
    getInitialState: (state) => {
      return {
        rowSelection: {},
        ...state
      };
    },
    getDefaultOptions: (table) => {
      return {
        onRowSelectionChange: makeStateUpdater("rowSelection", table),
        enableRowSelection: true,
        enableMultiRowSelection: true,
        enableSubRowSelection: true
        // enableGroupingRowSelection: false,
        // isAdditiveSelectEvent: (e: unknown) => !!e.metaKey,
        // isInclusiveSelectEvent: (e: unknown) => !!e.shiftKey,
      };
    },
    createTable: (table) => {
      table.setRowSelection = (updater) => table.options.onRowSelectionChange == null ? void 0 : table.options.onRowSelectionChange(updater);
      table.resetRowSelection = (defaultState) => {
        var _table$initialState$r;
        return table.setRowSelection(defaultState ? {} : (_table$initialState$r = table.initialState.rowSelection) != null ? _table$initialState$r : {});
      };
      table.toggleAllRowsSelected = (value) => {
        table.setRowSelection((old) => {
          value = typeof value !== "undefined" ? value : !table.getIsAllRowsSelected();
          const rowSelection = {
            ...old
          };
          const preGroupedFlatRows = table.getPreGroupedRowModel().flatRows;
          if (value) {
            preGroupedFlatRows.forEach((row) => {
              if (!row.getCanSelect()) {
                return;
              }
              rowSelection[row.id] = true;
            });
          } else {
            preGroupedFlatRows.forEach((row) => {
              delete rowSelection[row.id];
            });
          }
          return rowSelection;
        });
      };
      table.toggleAllPageRowsSelected = (value) => table.setRowSelection((old) => {
        const resolvedValue = typeof value !== "undefined" ? value : !table.getIsAllPageRowsSelected();
        const rowSelection = {
          ...old
        };
        table.getRowModel().rows.forEach((row) => {
          mutateRowIsSelected(rowSelection, row.id, resolvedValue, true, table);
        });
        return rowSelection;
      });
      table.getPreSelectedRowModel = () => table.getCoreRowModel();
      table.getSelectedRowModel = memo(() => [table.getState().rowSelection, table.getCoreRowModel()], (rowSelection, rowModel) => {
        if (!Object.keys(rowSelection).length) {
          return {
            rows: [],
            flatRows: [],
            rowsById: {}
          };
        }
        return selectRowsFn(table, rowModel);
      }, getMemoOptions(table.options, "debugTable", "getSelectedRowModel"));
      table.getFilteredSelectedRowModel = memo(() => [table.getState().rowSelection, table.getFilteredRowModel()], (rowSelection, rowModel) => {
        if (!Object.keys(rowSelection).length) {
          return {
            rows: [],
            flatRows: [],
            rowsById: {}
          };
        }
        return selectRowsFn(table, rowModel);
      }, getMemoOptions(table.options, "debugTable", "getFilteredSelectedRowModel"));
      table.getGroupedSelectedRowModel = memo(() => [table.getState().rowSelection, table.getSortedRowModel()], (rowSelection, rowModel) => {
        if (!Object.keys(rowSelection).length) {
          return {
            rows: [],
            flatRows: [],
            rowsById: {}
          };
        }
        return selectRowsFn(table, rowModel);
      }, getMemoOptions(table.options, "debugTable", "getGroupedSelectedRowModel"));
      table.getIsAllRowsSelected = () => {
        const preGroupedFlatRows = table.getFilteredRowModel().flatRows;
        const {
          rowSelection
        } = table.getState();
        let isAllRowsSelected = Boolean(preGroupedFlatRows.length && Object.keys(rowSelection).length);
        if (isAllRowsSelected) {
          if (preGroupedFlatRows.some((row) => row.getCanSelect() && !rowSelection[row.id])) {
            isAllRowsSelected = false;
          }
        }
        return isAllRowsSelected;
      };
      table.getIsAllPageRowsSelected = () => {
        const paginationFlatRows = table.getPaginationRowModel().flatRows.filter((row) => row.getCanSelect());
        const {
          rowSelection
        } = table.getState();
        let isAllPageRowsSelected = !!paginationFlatRows.length;
        if (isAllPageRowsSelected && paginationFlatRows.some((row) => !rowSelection[row.id])) {
          isAllPageRowsSelected = false;
        }
        return isAllPageRowsSelected;
      };
      table.getIsSomeRowsSelected = () => {
        var _table$getState$rowSe;
        const totalSelected = Object.keys((_table$getState$rowSe = table.getState().rowSelection) != null ? _table$getState$rowSe : {}).length;
        return totalSelected > 0 && totalSelected < table.getFilteredRowModel().flatRows.length;
      };
      table.getIsSomePageRowsSelected = () => {
        const paginationFlatRows = table.getPaginationRowModel().flatRows;
        return table.getIsAllPageRowsSelected() ? false : paginationFlatRows.filter((row) => row.getCanSelect()).some((d5) => d5.getIsSelected() || d5.getIsSomeSelected());
      };
      table.getToggleAllRowsSelectedHandler = () => {
        return (e4) => {
          table.toggleAllRowsSelected(e4.target.checked);
        };
      };
      table.getToggleAllPageRowsSelectedHandler = () => {
        return (e4) => {
          table.toggleAllPageRowsSelected(e4.target.checked);
        };
      };
    },
    createRow: (row, table) => {
      row.toggleSelected = (value, opts) => {
        const isSelected = row.getIsSelected();
        table.setRowSelection((old) => {
          var _opts$selectChildren;
          value = typeof value !== "undefined" ? value : !isSelected;
          if (row.getCanSelect() && isSelected === value) {
            return old;
          }
          const selectedRowIds = {
            ...old
          };
          mutateRowIsSelected(selectedRowIds, row.id, value, (_opts$selectChildren = opts == null ? void 0 : opts.selectChildren) != null ? _opts$selectChildren : true, table);
          return selectedRowIds;
        });
      };
      row.getIsSelected = () => {
        const {
          rowSelection
        } = table.getState();
        return isRowSelected(row, rowSelection);
      };
      row.getIsSomeSelected = () => {
        const {
          rowSelection
        } = table.getState();
        return isSubRowSelected(row, rowSelection) === "some";
      };
      row.getIsAllSubRowsSelected = () => {
        const {
          rowSelection
        } = table.getState();
        return isSubRowSelected(row, rowSelection) === "all";
      };
      row.getCanSelect = () => {
        var _table$options$enable;
        if (typeof table.options.enableRowSelection === "function") {
          return table.options.enableRowSelection(row);
        }
        return (_table$options$enable = table.options.enableRowSelection) != null ? _table$options$enable : true;
      };
      row.getCanSelectSubRows = () => {
        var _table$options$enable2;
        if (typeof table.options.enableSubRowSelection === "function") {
          return table.options.enableSubRowSelection(row);
        }
        return (_table$options$enable2 = table.options.enableSubRowSelection) != null ? _table$options$enable2 : true;
      };
      row.getCanMultiSelect = () => {
        var _table$options$enable3;
        if (typeof table.options.enableMultiRowSelection === "function") {
          return table.options.enableMultiRowSelection(row);
        }
        return (_table$options$enable3 = table.options.enableMultiRowSelection) != null ? _table$options$enable3 : true;
      };
      row.getToggleSelectedHandler = () => {
        const canSelect = row.getCanSelect();
        return (e4) => {
          var _target;
          if (!canSelect) return;
          row.toggleSelected((_target = e4.target) == null ? void 0 : _target.checked);
        };
      };
    }
  };
  var mutateRowIsSelected = (selectedRowIds, id, value, includeChildren, table) => {
    var _row$subRows;
    const row = table.getRow(id, true);
    if (value) {
      if (!row.getCanMultiSelect()) {
        Object.keys(selectedRowIds).forEach((key) => delete selectedRowIds[key]);
      }
      if (row.getCanSelect()) {
        selectedRowIds[id] = true;
      }
    } else {
      delete selectedRowIds[id];
    }
    if (includeChildren && (_row$subRows = row.subRows) != null && _row$subRows.length && row.getCanSelectSubRows()) {
      row.subRows.forEach((row2) => mutateRowIsSelected(selectedRowIds, row2.id, value, includeChildren, table));
    }
  };
  function selectRowsFn(table, rowModel) {
    const rowSelection = table.getState().rowSelection;
    const newSelectedFlatRows = [];
    const newSelectedRowsById = {};
    const recurseRows = function(rows, depth) {
      return rows.map((row) => {
        var _row$subRows2;
        const isSelected = isRowSelected(row, rowSelection);
        if (isSelected) {
          newSelectedFlatRows.push(row);
          newSelectedRowsById[row.id] = row;
        }
        if ((_row$subRows2 = row.subRows) != null && _row$subRows2.length) {
          row = {
            ...row,
            subRows: recurseRows(row.subRows)
          };
        }
        if (isSelected) {
          return row;
        }
      }).filter(Boolean);
    };
    return {
      rows: recurseRows(rowModel.rows),
      flatRows: newSelectedFlatRows,
      rowsById: newSelectedRowsById
    };
  }
  function isRowSelected(row, selection) {
    var _selection$row$id;
    return (_selection$row$id = selection[row.id]) != null ? _selection$row$id : false;
  }
  function isSubRowSelected(row, selection, table) {
    var _row$subRows3;
    if (!((_row$subRows3 = row.subRows) != null && _row$subRows3.length)) return false;
    let allChildrenSelected = true;
    let someSelected = false;
    row.subRows.forEach((subRow) => {
      if (someSelected && !allChildrenSelected) {
        return;
      }
      if (subRow.getCanSelect()) {
        if (isRowSelected(subRow, selection)) {
          someSelected = true;
        } else {
          allChildrenSelected = false;
        }
      }
      if (subRow.subRows && subRow.subRows.length) {
        const subRowChildrenSelected = isSubRowSelected(subRow, selection);
        if (subRowChildrenSelected === "all") {
          someSelected = true;
        } else if (subRowChildrenSelected === "some") {
          someSelected = true;
          allChildrenSelected = false;
        } else {
          allChildrenSelected = false;
        }
      }
    });
    return allChildrenSelected ? "all" : someSelected ? "some" : false;
  }
  var reSplitAlphaNumeric = /([0-9]+)/gm;
  var alphanumeric = (rowA, rowB, columnId) => {
    return compareAlphanumeric(toString(rowA.getValue(columnId)).toLowerCase(), toString(rowB.getValue(columnId)).toLowerCase());
  };
  var alphanumericCaseSensitive = (rowA, rowB, columnId) => {
    return compareAlphanumeric(toString(rowA.getValue(columnId)), toString(rowB.getValue(columnId)));
  };
  var text = (rowA, rowB, columnId) => {
    return compareBasic(toString(rowA.getValue(columnId)).toLowerCase(), toString(rowB.getValue(columnId)).toLowerCase());
  };
  var textCaseSensitive = (rowA, rowB, columnId) => {
    return compareBasic(toString(rowA.getValue(columnId)), toString(rowB.getValue(columnId)));
  };
  var datetime = (rowA, rowB, columnId) => {
    const a4 = rowA.getValue(columnId);
    const b4 = rowB.getValue(columnId);
    return a4 > b4 ? 1 : a4 < b4 ? -1 : 0;
  };
  var basic = (rowA, rowB, columnId) => {
    return compareBasic(rowA.getValue(columnId), rowB.getValue(columnId));
  };
  function compareBasic(a4, b4) {
    return a4 === b4 ? 0 : a4 > b4 ? 1 : -1;
  }
  function toString(a4) {
    if (typeof a4 === "number") {
      if (isNaN(a4) || a4 === Infinity || a4 === -Infinity) {
        return "";
      }
      return String(a4);
    }
    if (typeof a4 === "string") {
      return a4;
    }
    return "";
  }
  function compareAlphanumeric(aStr, bStr) {
    const a4 = aStr.split(reSplitAlphaNumeric).filter(Boolean);
    const b4 = bStr.split(reSplitAlphaNumeric).filter(Boolean);
    while (a4.length && b4.length) {
      const aa = a4.shift();
      const bb = b4.shift();
      const an = parseInt(aa, 10);
      const bn = parseInt(bb, 10);
      const combo = [an, bn].sort();
      if (isNaN(combo[0])) {
        if (aa > bb) {
          return 1;
        }
        if (bb > aa) {
          return -1;
        }
        continue;
      }
      if (isNaN(combo[1])) {
        return isNaN(an) ? -1 : 1;
      }
      if (an > bn) {
        return 1;
      }
      if (bn > an) {
        return -1;
      }
    }
    return a4.length - b4.length;
  }
  var sortingFns = {
    alphanumeric,
    alphanumericCaseSensitive,
    text,
    textCaseSensitive,
    datetime,
    basic
  };
  var RowSorting = {
    getInitialState: (state) => {
      return {
        sorting: [],
        ...state
      };
    },
    getDefaultColumnDef: () => {
      return {
        sortingFn: "auto",
        sortUndefined: 1
      };
    },
    getDefaultOptions: (table) => {
      return {
        onSortingChange: makeStateUpdater("sorting", table),
        isMultiSortEvent: (e4) => {
          return e4.shiftKey;
        }
      };
    },
    createColumn: (column, table) => {
      column.getAutoSortingFn = () => {
        const firstRows = table.getFilteredRowModel().flatRows.slice(10);
        let isString = false;
        for (const row of firstRows) {
          const value = row == null ? void 0 : row.getValue(column.id);
          if (Object.prototype.toString.call(value) === "[object Date]") {
            return sortingFns.datetime;
          }
          if (typeof value === "string") {
            isString = true;
            if (value.split(reSplitAlphaNumeric).length > 1) {
              return sortingFns.alphanumeric;
            }
          }
        }
        if (isString) {
          return sortingFns.text;
        }
        return sortingFns.basic;
      };
      column.getAutoSortDir = () => {
        const firstRow = table.getFilteredRowModel().flatRows[0];
        const value = firstRow == null ? void 0 : firstRow.getValue(column.id);
        if (typeof value === "string") {
          return "asc";
        }
        return "desc";
      };
      column.getSortingFn = () => {
        var _table$options$sortin, _table$options$sortin2;
        if (!column) {
          throw new Error();
        }
        return isFunction(column.columnDef.sortingFn) ? column.columnDef.sortingFn : column.columnDef.sortingFn === "auto" ? column.getAutoSortingFn() : (_table$options$sortin = (_table$options$sortin2 = table.options.sortingFns) == null ? void 0 : _table$options$sortin2[column.columnDef.sortingFn]) != null ? _table$options$sortin : sortingFns[column.columnDef.sortingFn];
      };
      column.toggleSorting = (desc, multi) => {
        const nextSortingOrder = column.getNextSortingOrder();
        const hasManualValue = typeof desc !== "undefined" && desc !== null;
        table.setSorting((old) => {
          const existingSorting = old == null ? void 0 : old.find((d5) => d5.id === column.id);
          const existingIndex = old == null ? void 0 : old.findIndex((d5) => d5.id === column.id);
          let newSorting = [];
          let sortAction;
          let nextDesc = hasManualValue ? desc : nextSortingOrder === "desc";
          if (old != null && old.length && column.getCanMultiSort() && multi) {
            if (existingSorting) {
              sortAction = "toggle";
            } else {
              sortAction = "add";
            }
          } else {
            if (old != null && old.length && existingIndex !== old.length - 1) {
              sortAction = "replace";
            } else if (existingSorting) {
              sortAction = "toggle";
            } else {
              sortAction = "replace";
            }
          }
          if (sortAction === "toggle") {
            if (!hasManualValue) {
              if (!nextSortingOrder) {
                sortAction = "remove";
              }
            }
          }
          if (sortAction === "add") {
            var _table$options$maxMul;
            newSorting = [...old, {
              id: column.id,
              desc: nextDesc
            }];
            newSorting.splice(0, newSorting.length - ((_table$options$maxMul = table.options.maxMultiSortColCount) != null ? _table$options$maxMul : Number.MAX_SAFE_INTEGER));
          } else if (sortAction === "toggle") {
            newSorting = old.map((d5) => {
              if (d5.id === column.id) {
                return {
                  ...d5,
                  desc: nextDesc
                };
              }
              return d5;
            });
          } else if (sortAction === "remove") {
            newSorting = old.filter((d5) => d5.id !== column.id);
          } else {
            newSorting = [{
              id: column.id,
              desc: nextDesc
            }];
          }
          return newSorting;
        });
      };
      column.getFirstSortDir = () => {
        var _ref, _column$columnDef$sor;
        const sortDescFirst = (_ref = (_column$columnDef$sor = column.columnDef.sortDescFirst) != null ? _column$columnDef$sor : table.options.sortDescFirst) != null ? _ref : column.getAutoSortDir() === "desc";
        return sortDescFirst ? "desc" : "asc";
      };
      column.getNextSortingOrder = (multi) => {
        var _table$options$enable, _table$options$enable2;
        const firstSortDirection = column.getFirstSortDir();
        const isSorted = column.getIsSorted();
        if (!isSorted) {
          return firstSortDirection;
        }
        if (isSorted !== firstSortDirection && ((_table$options$enable = table.options.enableSortingRemoval) != null ? _table$options$enable : true) && // If enableSortRemove, enable in general
        (multi ? (_table$options$enable2 = table.options.enableMultiRemove) != null ? _table$options$enable2 : true : true)) {
          return false;
        }
        return isSorted === "desc" ? "asc" : "desc";
      };
      column.getCanSort = () => {
        var _column$columnDef$ena, _table$options$enable3;
        return ((_column$columnDef$ena = column.columnDef.enableSorting) != null ? _column$columnDef$ena : true) && ((_table$options$enable3 = table.options.enableSorting) != null ? _table$options$enable3 : true) && !!column.accessorFn;
      };
      column.getCanMultiSort = () => {
        var _ref2, _column$columnDef$ena2;
        return (_ref2 = (_column$columnDef$ena2 = column.columnDef.enableMultiSort) != null ? _column$columnDef$ena2 : table.options.enableMultiSort) != null ? _ref2 : !!column.accessorFn;
      };
      column.getIsSorted = () => {
        var _table$getState$sorti;
        const columnSort = (_table$getState$sorti = table.getState().sorting) == null ? void 0 : _table$getState$sorti.find((d5) => d5.id === column.id);
        return !columnSort ? false : columnSort.desc ? "desc" : "asc";
      };
      column.getSortIndex = () => {
        var _table$getState$sorti2, _table$getState$sorti3;
        return (_table$getState$sorti2 = (_table$getState$sorti3 = table.getState().sorting) == null ? void 0 : _table$getState$sorti3.findIndex((d5) => d5.id === column.id)) != null ? _table$getState$sorti2 : -1;
      };
      column.clearSorting = () => {
        table.setSorting((old) => old != null && old.length ? old.filter((d5) => d5.id !== column.id) : []);
      };
      column.getToggleSortingHandler = () => {
        const canSort = column.getCanSort();
        return (e4) => {
          if (!canSort) return;
          e4.persist == null || e4.persist();
          column.toggleSorting == null || column.toggleSorting(void 0, column.getCanMultiSort() ? table.options.isMultiSortEvent == null ? void 0 : table.options.isMultiSortEvent(e4) : false);
        };
      };
    },
    createTable: (table) => {
      table.setSorting = (updater) => table.options.onSortingChange == null ? void 0 : table.options.onSortingChange(updater);
      table.resetSorting = (defaultState) => {
        var _table$initialState$s, _table$initialState;
        table.setSorting(defaultState ? [] : (_table$initialState$s = (_table$initialState = table.initialState) == null ? void 0 : _table$initialState.sorting) != null ? _table$initialState$s : []);
      };
      table.getPreSortedRowModel = () => table.getGroupedRowModel();
      table.getSortedRowModel = () => {
        if (!table._getSortedRowModel && table.options.getSortedRowModel) {
          table._getSortedRowModel = table.options.getSortedRowModel(table);
        }
        if (table.options.manualSorting || !table._getSortedRowModel) {
          return table.getPreSortedRowModel();
        }
        return table._getSortedRowModel();
      };
    }
  };
  var builtInFeatures = [
    Headers,
    ColumnVisibility,
    ColumnOrdering,
    ColumnPinning,
    ColumnFaceting,
    ColumnFiltering,
    GlobalFaceting,
    //depends on ColumnFaceting
    GlobalFiltering,
    //depends on ColumnFiltering
    RowSorting,
    ColumnGrouping,
    //depends on RowSorting
    RowExpanding,
    RowPagination,
    RowPinning,
    RowSelection,
    ColumnSizing
  ];
  function createTable(options) {
    var _options$_features, _options$initialState;
    if (options.debugAll || options.debugTable) {
      console.info("Creating Table Instance...");
    }
    const _features = [...builtInFeatures, ...(_options$_features = options._features) != null ? _options$_features : []];
    let table = {
      _features
    };
    const defaultOptions = table._features.reduce((obj, feature) => {
      return Object.assign(obj, feature.getDefaultOptions == null ? void 0 : feature.getDefaultOptions(table));
    }, {});
    const mergeOptions = (options2) => {
      if (table.options.mergeOptions) {
        return table.options.mergeOptions(defaultOptions, options2);
      }
      return {
        ...defaultOptions,
        ...options2
      };
    };
    const coreInitialState = {};
    let initialState = {
      ...coreInitialState,
      ...(_options$initialState = options.initialState) != null ? _options$initialState : {}
    };
    table._features.forEach((feature) => {
      var _feature$getInitialSt;
      initialState = (_feature$getInitialSt = feature.getInitialState == null ? void 0 : feature.getInitialState(initialState)) != null ? _feature$getInitialSt : initialState;
    });
    const queued = [];
    let queuedTimeout = false;
    const coreInstance = {
      _features,
      options: {
        ...defaultOptions,
        ...options
      },
      initialState,
      _queue: (cb) => {
        queued.push(cb);
        if (!queuedTimeout) {
          queuedTimeout = true;
          Promise.resolve().then(() => {
            while (queued.length) {
              queued.shift()();
            }
            queuedTimeout = false;
          }).catch((error) => setTimeout(() => {
            throw error;
          }));
        }
      },
      reset: () => {
        table.setState(table.initialState);
      },
      setOptions: (updater) => {
        const newOptions = functionalUpdate(updater, table.options);
        table.options = mergeOptions(newOptions);
      },
      getState: () => {
        return table.options.state;
      },
      setState: (updater) => {
        table.options.onStateChange == null || table.options.onStateChange(updater);
      },
      _getRowId: (row, index, parent) => {
        var _table$options$getRow;
        return (_table$options$getRow = table.options.getRowId == null ? void 0 : table.options.getRowId(row, index, parent)) != null ? _table$options$getRow : `${parent ? [parent.id, index].join(".") : index}`;
      },
      getCoreRowModel: () => {
        if (!table._getCoreRowModel) {
          table._getCoreRowModel = table.options.getCoreRowModel(table);
        }
        return table._getCoreRowModel();
      },
      // The final calls start at the bottom of the model,
      // expanded rows, which then work their way up
      getRowModel: () => {
        return table.getPaginationRowModel();
      },
      //in next version, we should just pass in the row model as the optional 2nd arg
      getRow: (id, searchAll) => {
        let row = (searchAll ? table.getPrePaginationRowModel() : table.getRowModel()).rowsById[id];
        if (!row) {
          row = table.getCoreRowModel().rowsById[id];
          if (!row) {
            if (true) {
              throw new Error(`getRow could not find row with ID: ${id}`);
            }
            throw new Error();
          }
        }
        return row;
      },
      _getDefaultColumnDef: memo(() => [table.options.defaultColumn], (defaultColumn) => {
        var _defaultColumn;
        defaultColumn = (_defaultColumn = defaultColumn) != null ? _defaultColumn : {};
        return {
          header: (props) => {
            const resolvedColumnDef = props.header.column.columnDef;
            if (resolvedColumnDef.accessorKey) {
              return resolvedColumnDef.accessorKey;
            }
            if (resolvedColumnDef.accessorFn) {
              return resolvedColumnDef.id;
            }
            return null;
          },
          // footer: props => props.header.column.id,
          cell: (props) => {
            var _props$renderValue$to, _props$renderValue;
            return (_props$renderValue$to = (_props$renderValue = props.renderValue()) == null || _props$renderValue.toString == null ? void 0 : _props$renderValue.toString()) != null ? _props$renderValue$to : null;
          },
          ...table._features.reduce((obj, feature) => {
            return Object.assign(obj, feature.getDefaultColumnDef == null ? void 0 : feature.getDefaultColumnDef());
          }, {}),
          ...defaultColumn
        };
      }, getMemoOptions(options, "debugColumns", "_getDefaultColumnDef")),
      _getColumnDefs: () => table.options.columns,
      getAllColumns: memo(() => [table._getColumnDefs()], (columnDefs) => {
        const recurseColumns = function(columnDefs2, parent, depth) {
          if (depth === void 0) {
            depth = 0;
          }
          return columnDefs2.map((columnDef) => {
            const column = createColumn(table, columnDef, depth, parent);
            const groupingColumnDef = columnDef;
            column.columns = groupingColumnDef.columns ? recurseColumns(groupingColumnDef.columns, column, depth + 1) : [];
            return column;
          });
        };
        return recurseColumns(columnDefs);
      }, getMemoOptions(options, "debugColumns", "getAllColumns")),
      getAllFlatColumns: memo(() => [table.getAllColumns()], (allColumns) => {
        return allColumns.flatMap((column) => {
          return column.getFlatColumns();
        });
      }, getMemoOptions(options, "debugColumns", "getAllFlatColumns")),
      _getAllFlatColumnsById: memo(() => [table.getAllFlatColumns()], (flatColumns) => {
        return flatColumns.reduce((acc, column) => {
          acc[column.id] = column;
          return acc;
        }, {});
      }, getMemoOptions(options, "debugColumns", "getAllFlatColumnsById")),
      getAllLeafColumns: memo(() => [table.getAllColumns(), table._getOrderColumnsFn()], (allColumns, orderColumns2) => {
        let leafColumns = allColumns.flatMap((column) => column.getLeafColumns());
        return orderColumns2(leafColumns);
      }, getMemoOptions(options, "debugColumns", "getAllLeafColumns")),
      getColumn: (columnId) => {
        const column = table._getAllFlatColumnsById()[columnId];
        if (!column) {
          console.error(`[Table] Column with id '${columnId}' does not exist.`);
        }
        return column;
      }
    };
    Object.assign(table, coreInstance);
    for (let index = 0; index < table._features.length; index++) {
      const feature = table._features[index];
      feature == null || feature.createTable == null || feature.createTable(table);
    }
    return table;
  }
  function getCoreRowModel() {
    return (table) => memo(() => [table.options.data], (data) => {
      const rowModel = {
        rows: [],
        flatRows: [],
        rowsById: {}
      };
      const accessRows = function(originalRows, depth, parentRow) {
        if (depth === void 0) {
          depth = 0;
        }
        const rows = [];
        for (let i4 = 0; i4 < originalRows.length; i4++) {
          const row = createRow(table, table._getRowId(originalRows[i4], i4, parentRow), originalRows[i4], i4, depth, void 0, parentRow == null ? void 0 : parentRow.id);
          rowModel.flatRows.push(row);
          rowModel.rowsById[row.id] = row;
          rows.push(row);
          if (table.options.getSubRows) {
            var _row$originalSubRows;
            row.originalSubRows = table.options.getSubRows(originalRows[i4], i4);
            if ((_row$originalSubRows = row.originalSubRows) != null && _row$originalSubRows.length) {
              row.subRows = accessRows(row.originalSubRows, depth + 1, row);
            }
          }
        }
        return rows;
      };
      rowModel.rows = accessRows(data);
      return rowModel;
    }, getMemoOptions(table.options, "debugTable", "getRowModel", () => table._autoResetPageIndex()));
  }
  function expandRows(rowModel) {
    const expandedRows = [];
    const handleRow = (row) => {
      var _row$subRows;
      expandedRows.push(row);
      if ((_row$subRows = row.subRows) != null && _row$subRows.length && row.getIsExpanded()) {
        row.subRows.forEach(handleRow);
      }
    };
    rowModel.rows.forEach(handleRow);
    return {
      rows: expandedRows,
      flatRows: rowModel.flatRows,
      rowsById: rowModel.rowsById
    };
  }
  function getPaginationRowModel(opts) {
    return (table) => memo(() => [table.getState().pagination, table.getPrePaginationRowModel(), table.options.paginateExpandedRows ? void 0 : table.getState().expanded], (pagination, rowModel) => {
      if (!rowModel.rows.length) {
        return rowModel;
      }
      const {
        pageSize,
        pageIndex
      } = pagination;
      let {
        rows,
        flatRows,
        rowsById
      } = rowModel;
      const pageStart = pageSize * pageIndex;
      const pageEnd = pageStart + pageSize;
      rows = rows.slice(pageStart, pageEnd);
      let paginatedRowModel;
      if (!table.options.paginateExpandedRows) {
        paginatedRowModel = expandRows({
          rows,
          flatRows,
          rowsById
        });
      } else {
        paginatedRowModel = {
          rows,
          flatRows,
          rowsById
        };
      }
      paginatedRowModel.flatRows = [];
      const handleRow = (row) => {
        paginatedRowModel.flatRows.push(row);
        if (row.subRows.length) {
          row.subRows.forEach(handleRow);
        }
      };
      paginatedRowModel.rows.forEach(handleRow);
      return paginatedRowModel;
    }, getMemoOptions(table.options, "debugTable", "getPaginationRowModel"));
  }
  function getSortedRowModel() {
    return (table) => memo(() => [table.getState().sorting, table.getPreSortedRowModel()], (sorting, rowModel) => {
      if (!rowModel.rows.length || !(sorting != null && sorting.length)) {
        return rowModel;
      }
      const sortingState = table.getState().sorting;
      const sortedFlatRows = [];
      const availableSorting = sortingState.filter((sort) => {
        var _table$getColumn;
        return (_table$getColumn = table.getColumn(sort.id)) == null ? void 0 : _table$getColumn.getCanSort();
      });
      const columnInfoById = {};
      availableSorting.forEach((sortEntry) => {
        const column = table.getColumn(sortEntry.id);
        if (!column) return;
        columnInfoById[sortEntry.id] = {
          sortUndefined: column.columnDef.sortUndefined,
          invertSorting: column.columnDef.invertSorting,
          sortingFn: column.getSortingFn()
        };
      });
      const sortData = (rows) => {
        const sortedData = rows.map((row) => ({
          ...row
        }));
        sortedData.sort((rowA, rowB) => {
          for (let i4 = 0; i4 < availableSorting.length; i4 += 1) {
            var _sortEntry$desc;
            const sortEntry = availableSorting[i4];
            const columnInfo = columnInfoById[sortEntry.id];
            const sortUndefined = columnInfo.sortUndefined;
            const isDesc = (_sortEntry$desc = sortEntry == null ? void 0 : sortEntry.desc) != null ? _sortEntry$desc : false;
            let sortInt = 0;
            if (sortUndefined) {
              const aValue = rowA.getValue(sortEntry.id);
              const bValue = rowB.getValue(sortEntry.id);
              const aUndefined = aValue === void 0;
              const bUndefined = bValue === void 0;
              if (aUndefined || bUndefined) {
                if (sortUndefined === "first") return aUndefined ? -1 : 1;
                if (sortUndefined === "last") return aUndefined ? 1 : -1;
                sortInt = aUndefined && bUndefined ? 0 : aUndefined ? sortUndefined : -sortUndefined;
              }
            }
            if (sortInt === 0) {
              sortInt = columnInfo.sortingFn(rowA, rowB, sortEntry.id);
            }
            if (sortInt !== 0) {
              if (isDesc) {
                sortInt *= -1;
              }
              if (columnInfo.invertSorting) {
                sortInt *= -1;
              }
              return sortInt;
            }
          }
          return rowA.index - rowB.index;
        });
        sortedData.forEach((row) => {
          var _row$subRows;
          sortedFlatRows.push(row);
          if ((_row$subRows = row.subRows) != null && _row$subRows.length) {
            row.subRows = sortData(row.subRows);
          }
        });
        return sortedData;
      };
      return {
        rows: sortData(rowModel.rows),
        flatRows: sortedFlatRows,
        rowsById: rowModel.rowsById
      };
    }, getMemoOptions(table.options, "debugTable", "getSortedRowModel", () => table._autoResetPageIndex()));
  }

  // src/ui/components/DataTable.tsx
  function renderCell(cell) {
    const def = cell.column.columnDef.cell;
    if (typeof def === "function") {
      return def(cell.getContext());
    }
    return cell.getValue();
  }
  function renderHeader(header) {
    const def = header.column.columnDef.header;
    if (typeof def === "function") {
      return def(header.getContext());
    }
    return def;
  }
  function resolveUpdater(updater, prev) {
    return typeof updater === "function" ? updater(prev) : updater;
  }
  function DataTable({
    columns: columns4,
    data,
    title,
    exportFn,
    pageSize,
    defaultSort: defaultSort4,
    enableColumnVisibility,
    costRows
  }) {
    const [sorting, setSorting] = d2(defaultSort4 || []);
    const [pagination, setPagination] = d2({
      pageIndex: 0,
      pageSize: pageSize || data.length || 100
    });
    const [columnVisibility, setColumnVisibility] = d2({});
    const [, rerender] = d2(0);
    y2(() => {
      setPagination((prev) => ({ ...prev, pageIndex: 0 }));
    }, [data]);
    const tableRef = A2(null);
    const stateRef = A2({ sorting, pagination, columnVisibility });
    stateRef.current = { sorting, pagination, columnVisibility };
    if (!tableRef.current) {
      tableRef.current = createTable({
        columns: columns4,
        data,
        state: { sorting, pagination, columnVisibility, columnPinning: { left: [], right: [] } },
        onStateChange: (updater) => {
          const newState = resolveUpdater(updater, tableRef.current.getState());
          if (newState.sorting !== stateRef.current.sorting) setSorting(newState.sorting);
          if (newState.pagination !== stateRef.current.pagination) setPagination(newState.pagination);
          if (newState.columnVisibility !== stateRef.current.columnVisibility) setColumnVisibility(newState.columnVisibility);
          rerender((n3) => n3 + 1);
        },
        onSortingChange: (updater) => setSorting((prev) => resolveUpdater(updater, prev)),
        onPaginationChange: (updater) => setPagination((prev) => resolveUpdater(updater, prev)),
        onColumnVisibilityChange: (updater) => setColumnVisibility((prev) => resolveUpdater(updater, prev)),
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
        ...pageSize ? { getPaginationRowModel: getPaginationRowModel() } : {},
        renderFallbackValue: ""
      });
    }
    tableRef.current.setOptions((prev) => ({
      ...prev,
      columns: columns4,
      data,
      state: { ...tableRef.current.getState(), sorting, pagination, columnVisibility }
    }));
    const table = tableRef.current;
    const headerGroups = table.getHeaderGroups();
    const rows = table.getRowModel().rows;
    return /* @__PURE__ */ u2("div", { class: "table-card", children: [
      (title || exportFn) && /* @__PURE__ */ u2("div", { class: "section-header", children: [
        title && /* @__PURE__ */ u2("div", { class: "section-title", children: title }),
        exportFn && /* @__PURE__ */ u2("button", { class: "export-btn", onClick: exportFn, title: "Export to CSV", children: "\u2913 CSV" })
      ] }),
      enableColumnVisibility && /* @__PURE__ */ u2("div", { class: "column-toggle", children: table.getAllLeafColumns().map((column) => /* @__PURE__ */ u2("label", { children: [
        /* @__PURE__ */ u2(
          "input",
          {
            type: "checkbox",
            checked: column.getIsVisible(),
            onChange: column.getToggleVisibilityHandler()
          }
        ),
        typeof column.columnDef.header === "string" ? column.columnDef.header : column.id
      ] }, column.id)) }),
      /* @__PURE__ */ u2("table", { children: [
        /* @__PURE__ */ u2("thead", { children: headerGroups.map((headerGroup) => /* @__PURE__ */ u2("tr", { children: headerGroup.headers.map((header) => {
          const canSort = header.column.getCanSort();
          const sorted = header.column.getIsSorted();
          return /* @__PURE__ */ u2(
            "th",
            {
              scope: "col",
              class: canSort ? "sortable" : void 0,
              "aria-sort": sorted === "asc" ? "ascending" : sorted === "desc" ? "descending" : void 0,
              style: sorted ? { borderBottom: "2px solid var(--text-display)" } : void 0,
              tabIndex: canSort ? 0 : void 0,
              onClick: canSort ? header.column.getToggleSortingHandler() : void 0,
              onKeyDown: canSort ? (e4) => {
                if (e4.key === "Enter" || e4.key === " ") {
                  e4.preventDefault();
                  header.column.getToggleSortingHandler()?.(e4);
                }
              } : void 0,
              children: [
                renderHeader(header),
                canSort && /* @__PURE__ */ u2("span", { class: "sort-icon", children: sorted === "desc" ? " \u25BC" : sorted === "asc" ? " \u25B2" : "" })
              ]
            },
            header.id
          );
        }) }, headerGroup.id)) }),
        /* @__PURE__ */ u2("tbody", { children: rows.map((row) => /* @__PURE__ */ u2("tr", { class: costRows ? "cost-row" : void 0, children: row.getVisibleCells().map((cell) => /* @__PURE__ */ u2("td", { children: renderCell(cell) }, cell.id)) }, row.id)) })
      ] }),
      pageSize && /* @__PURE__ */ u2("div", { class: "pagination", children: [
        /* @__PURE__ */ u2("span", { children: table.getRowCount() > 0 ? `Showing ${pagination.pageIndex * pagination.pageSize + 1}\u2013${Math.min(
          (pagination.pageIndex + 1) * pagination.pageSize,
          table.getRowCount()
        )} of ${table.getRowCount()}` : "No sessions" }),
        /* @__PURE__ */ u2("div", { style: { display: "flex", gap: "6px" }, children: [
          /* @__PURE__ */ u2(
            "button",
            {
              class: "filter-btn",
              disabled: !table.getCanPreviousPage(),
              onClick: () => table.previousPage(),
              children: "\xAB Prev"
            }
          ),
          /* @__PURE__ */ u2(
            "button",
            {
              class: "filter-btn",
              disabled: !table.getCanNextPage(),
              onClick: () => table.nextPage(),
              children: "Next \xBB"
            }
          )
        ] })
      ] })
    ] });
  }

  // src/ui/components/EntrypointTable.tsx
  var columns = [
    {
      accessorKey: "provider",
      header: "Provider",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
    },
    {
      accessorKey: "entrypoint",
      header: "Entrypoint",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(getValue()) })
    },
    {
      accessorKey: "sessions",
      header: "Sessions",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: getValue() })
    },
    {
      accessorKey: "turns",
      header: "Turns",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
    },
    {
      accessorKey: "input",
      header: "Input",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
    },
    {
      accessorKey: "output",
      header: "Output",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
    }
  ];
  function EntrypointTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u2(DataTable, { columns, data, title: "Usage by Entrypoint" });
  }

  // src/ui/components/ServiceTiers.tsx
  var columns2 = [
    {
      accessorKey: "provider",
      header: "Provider",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
    },
    { accessorKey: "service_tier", header: "Tier" },
    { accessorKey: "inference_geo", header: "Region" },
    {
      accessorKey: "turns",
      header: "Turns",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
    }
  ];
  function ServiceTiersTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u2(DataTable, { columns: columns2, data, title: "Service Tiers" });
  }

  // src/ui/components/ToolUsageTable.tsx
  function RankBar({ value, max: max2, label }) {
    const pct = max2 > 0 ? value / max2 * 100 : 0;
    const tooltip = `${value} (${pct.toFixed(1)}% of max ${max2})`;
    return /* @__PURE__ */ u2(
      "span",
      {
        style: { position: "relative", display: "inline-block", width: "100%" },
        title: tooltip,
        children: [
          /* @__PURE__ */ u2(
            "span",
            {
              "data-testid": "rank-bar",
              style: {
                position: "absolute",
                top: 0,
                left: 0,
                bottom: 0,
                width: `${pct}%`,
                backgroundColor: "var(--color-text-primary)",
                opacity: 0.12,
                pointerEvents: "none"
              }
            }
          ),
          /* @__PURE__ */ u2("span", { class: "num", style: { position: "relative", zIndex: 1 }, children: label })
        ]
      }
    );
  }
  function makeColumns(data) {
    const maxInvocations = data.reduce((m4, r4) => Math.max(m4, r4.invocations), 0);
    return [
      {
        accessorKey: "provider",
        header: "Provider",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
      },
      {
        accessorKey: "tool_name",
        header: "Tool",
        cell: ({ row }) => {
          const cat = row.original.category;
          const badge = cat === "mcp" ? "mcp" : "builtin";
          return /* @__PURE__ */ u2("span", { children: [
            /* @__PURE__ */ u2("span", { class: `model-tag ${badge}`, children: cat }),
            " ",
            row.original.tool_name
          ] });
        }
      },
      {
        accessorKey: "mcp_server",
        header: "MCP Server",
        cell: ({ getValue }) => {
          const v4 = getValue();
          return v4 ? /* @__PURE__ */ u2("span", { class: "muted", children: v4 }) : /* @__PURE__ */ u2("span", { class: "muted", children: "--" });
        }
      },
      {
        accessorKey: "invocations",
        header: "Calls",
        cell: ({ getValue }) => /* @__PURE__ */ u2(
          RankBar,
          {
            value: getValue(),
            max: maxInvocations,
            label: fmt(getValue())
          }
        )
      },
      {
        accessorKey: "turns_used",
        header: "Turns",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "sessions_used",
        header: "Sessions",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "errors",
        header: "Errors",
        cell: ({ row }) => {
          const e4 = row.original.errors;
          if (!e4) return /* @__PURE__ */ u2("span", { class: "dim", children: "0" });
          const pct = row.original.invocations > 0 ? (e4 / row.original.invocations * 100).toFixed(1) : "0";
          return /* @__PURE__ */ u2("span", { class: "num", style: { color: "var(--accent)" }, children: [
            e4,
            " (",
            pct,
            "%)"
          ] });
        }
      }
    ];
  }
  function ToolUsageTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u2(DataTable, { columns: makeColumns(data), data, title: "Tool Usage" });
  }

  // src/ui/components/McpSummaryTable.tsx
  function RankBar2({ value, max: max2, label }) {
    const pct = max2 > 0 ? value / max2 * 100 : 0;
    const tooltip = `${value} (${pct.toFixed(1)}% of max ${max2})`;
    return /* @__PURE__ */ u2(
      "span",
      {
        style: { position: "relative", display: "inline-block", width: "100%" },
        title: tooltip,
        children: [
          /* @__PURE__ */ u2(
            "span",
            {
              "data-testid": "rank-bar",
              style: {
                position: "absolute",
                top: 0,
                left: 0,
                bottom: 0,
                width: `${pct}%`,
                backgroundColor: "var(--color-text-primary)",
                opacity: 0.12,
                pointerEvents: "none"
              }
            }
          ),
          /* @__PURE__ */ u2("span", { class: "num", style: { position: "relative", zIndex: 1 }, children: label })
        ]
      }
    );
  }
  function makeColumns2(data) {
    const maxInvocations = data.reduce((m4, r4) => Math.max(m4, r4.invocations), 0);
    return [
      {
        accessorKey: "provider",
        header: "Provider",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
      },
      {
        accessorKey: "server",
        header: "MCP Server",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag mcp", children: String(getValue()) })
      },
      {
        accessorKey: "tools_used",
        header: "Tools",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: getValue() })
      },
      {
        accessorKey: "invocations",
        header: "Calls",
        cell: ({ getValue }) => /* @__PURE__ */ u2(
          RankBar2,
          {
            value: getValue(),
            max: maxInvocations,
            label: fmt(getValue())
          }
        )
      },
      {
        accessorKey: "sessions_used",
        header: "Sessions",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
      }
    ];
  }
  function McpSummaryTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u2(DataTable, { columns: makeColumns2(data), data, title: "MCP Server Usage" });
  }

  // src/ui/components/BranchTable.tsx
  function RankBar3({ value, max: max2, label }) {
    const pct = max2 > 0 ? value / max2 * 100 : 0;
    const tooltip = `${value} (${pct.toFixed(1)}% of max ${max2})`;
    return /* @__PURE__ */ u2(
      "span",
      {
        style: { position: "relative", display: "inline-block", width: "100%" },
        title: tooltip,
        children: [
          /* @__PURE__ */ u2(
            "span",
            {
              "data-testid": "rank-bar",
              style: {
                position: "absolute",
                top: 0,
                left: 0,
                bottom: 0,
                width: `${pct}%`,
                backgroundColor: "var(--color-text-primary)",
                opacity: 0.12,
                pointerEvents: "none"
              }
            }
          ),
          /* @__PURE__ */ u2("span", { class: "num", style: { position: "relative", zIndex: 1 }, children: label })
        ]
      }
    );
  }
  function makeColumns3(data) {
    const maxSessions = data.reduce((m4, r4) => Math.max(m4, r4.sessions), 0);
    return [
      {
        accessorKey: "provider",
        header: "Provider",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
      },
      {
        accessorKey: "branch",
        header: "Branch",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(getValue()) })
      },
      {
        accessorKey: "sessions",
        header: "Sessions",
        cell: ({ getValue }) => /* @__PURE__ */ u2(
          RankBar3,
          {
            value: getValue(),
            max: maxSessions,
            label: String(getValue())
          }
        )
      },
      {
        accessorKey: "turns",
        header: "Turns",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "input",
        header: "Input",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "output",
        header: "Output",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "cost",
        header: "Est. Cost",
        cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "cost", children: fmtCost(getValue()) })
      }
    ];
  }
  function BranchTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u2(DataTable, { columns: makeColumns3(data), data, title: "Usage by Git Branch" });
  }

  // src/ui/components/VersionTable.tsx
  var columns3 = [
    {
      accessorKey: "provider",
      header: "Provider",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
    },
    {
      accessorKey: "version",
      header: "Version",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(getValue()) })
    },
    {
      accessorKey: "turns",
      header: "Turns",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(getValue()) })
    },
    {
      accessorKey: "sessions",
      header: "Sessions",
      cell: ({ getValue }) => /* @__PURE__ */ u2("span", { class: "num", children: getValue() })
    }
  ];
  function VersionTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u2(DataTable, { columns: columns3, data, title: "CLI Versions" });
  }

  // src/ui/components/VersionDonut.tsx
  var METRIC_LABELS = {
    cost: "Cost",
    calls: "Calls",
    tokens: "Tokens"
  };
  function metricValue(row, metric) {
    switch (metric) {
      case "cost":
        return row.cost;
      case "calls":
        return row.turns;
      case "tokens":
        return row.tokens;
    }
  }
  function formatMetricTotal(total, metric) {
    if (metric === "cost") return fmtCost(total);
    return fmt(total);
  }
  function formatMetricSlice(val, metric) {
    if (metric === "cost") return fmtCost(val);
    return fmt(val);
  }
  function VersionDonut({ rows, metric, onMetricChange }) {
    if (!rows.length) return null;
    const normalized = rows.map((r4) => ({
      ...r4,
      version: r4.version === "" || r4.version === "unknown" ? "(unknown)" : r4.version
    }));
    const series = normalized.map((r4) => metricValue(r4, metric));
    const labels = normalized.map((r4) => r4.version);
    const total = series.reduce((s4, v4) => s4 + v4, 0);
    const base = industrialChartOptions("donut");
    const options = {
      ...base,
      chart: { ...base.chart, type: "donut" },
      series,
      labels,
      colors: modelSeriesColors(normalized.length),
      stroke: { width: 2, colors: [cssVar("--surface")] },
      plotOptions: {
        pie: {
          donut: {
            size: "64%",
            labels: {
              show: true,
              total: {
                show: true,
                label: "TOTAL",
                fontFamily: 'var(--font-mono), "Space Mono", monospace',
                fontSize: "11px",
                color: cssVar("--text-secondary"),
                formatter: () => formatMetricTotal(total, metric)
              },
              value: {
                fontFamily: 'var(--font-mono), "Space Mono", monospace',
                fontSize: "18px",
                color: cssVar("--text-display"),
                formatter: (val) => formatMetricSlice(Number(val), metric)
              },
              name: {
                fontFamily: 'var(--font-mono), "Space Mono", monospace',
                fontSize: "11px",
                color: cssVar("--text-secondary")
              }
            }
          }
        }
      },
      tooltip: {
        ...base.tooltip,
        custom: ({ seriesIndex }) => {
          const r4 = normalized[seriesIndex];
          if (!r4) return "";
          const label = r4.version;
          const cost = fmtCost(r4.cost);
          const calls = fmt(r4.turns);
          const tokens = fmt(r4.tokens);
          return `<div style="padding:8px 12px;font-family:var(--font-mono,'Space Mono',monospace);font-size:11px;line-height:1.6"><strong>${label}</strong><br/>${cost} &nbsp;&bull;&nbsp; ${calls} calls &nbsp;&bull;&nbsp; ${tokens} tokens</div>`;
        }
      }
    };
    return /* @__PURE__ */ u2("div", { style: { display: "flex", flexDirection: "column", gap: "8px", height: "100%" }, children: [
      /* @__PURE__ */ u2("div", { style: { display: "flex", gap: "4px", alignItems: "center" }, children: Object.keys(METRIC_LABELS).map((m4) => /* @__PURE__ */ u2(
        "button",
        {
          type: "button",
          class: `range-btn${metric === m4 ? " active" : ""}`,
          onClick: () => onMetricChange(m4),
          children: METRIC_LABELS[m4]
        },
        m4
      )) }),
      /* @__PURE__ */ u2("div", { style: { flex: 1, minHeight: 0 }, children: /* @__PURE__ */ u2(ApexChart, { options, id: "chart-version-donut" }) })
    ] });
  }

  // src/ui/components/HourlyChart.tsx
  function HourlyChart({ data }) {
    if (!data.length) return null;
    const maxTurns = Math.max(...data.map((d5) => d5.turns), 1);
    const fillColor = cssVar("--text-display");
    const emptyColor = cssVar("--border");
    return /* @__PURE__ */ u2("div", { children: [
      /* @__PURE__ */ u2("div", { class: "section-title", style: { padding: "0", marginBottom: "12px" }, children: "Activity by Hour of Day" }),
      /* @__PURE__ */ u2("div", { style: { display: "flex", alignItems: "flex-end", gap: "2px", height: "80px" }, children: Array.from({ length: 24 }, (_4, h5) => {
        const row = data.find((d5) => d5.hour === h5);
        const turns = row?.turns ?? 0;
        const pct = turns / maxTurns * 100;
        const background = turns > 0 ? withAlpha("--text-display", 0.4 + pct / 100 * 0.6) : emptyColor;
        return /* @__PURE__ */ u2(
          "div",
          {
            title: `${h5}:00 -- ${fmt(turns)} turns`,
            style: {
              flex: 1,
              height: `${Math.max(pct, 2)}%`,
              background,
              borderRadius: 0
            }
          },
          h5
        );
      }) }),
      /* @__PURE__ */ u2("div", { style: { display: "flex", gap: "2px", marginTop: "6px" }, children: Array.from({ length: 24 }, (_4, h5) => /* @__PURE__ */ u2(
        "span",
        {
          class: "muted",
          style: {
            flex: 1,
            fontFamily: "var(--font-mono)",
            fontSize: "9px",
            textAlign: "center",
            letterSpacing: "0.04em",
            color: cssVar("--text-secondary"),
            visibility: [0, 6, 12, 18].includes(h5) ? "visible" : "hidden"
          },
          children: String(h5).padStart(2, "0")
        },
        h5
      )) }),
      /* @__PURE__ */ u2("div", { style: { display: "none" }, "data-fill": fillColor })
    ] });
  }

  // src/ui/components/ActivityHeatmap.tsx
  var DOW_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  function cellOpacity(costNanos, maxCostNanos) {
    if (maxCostNanos <= 0 || costNanos <= 0) return 0.05;
    const ratio = costNanos / maxCostNanos;
    return Math.min(0.05 + 0.85 * ratio, 0.9);
  }
  function ActivityHeatmap({ data }) {
    const { cells, max_cost_nanos, active_days, total_cost_nanos, period } = data;
    const lookup = /* @__PURE__ */ new Map();
    for (const c4 of cells) {
      lookup.set(`${c4.dow},${c4.hour}`, c4);
    }
    const avgPerDay = active_days > 0 ? fmtCost(total_cost_nanos / 1e9 / active_days) : "--";
    return /* @__PURE__ */ u2("div", { children: [
      /* @__PURE__ */ u2(
        "div",
        {
          style: {
            display: "flex",
            alignItems: "baseline",
            gap: "12px",
            marginBottom: "8px",
            flexWrap: "wrap"
          },
          children: [
            /* @__PURE__ */ u2(
              "span",
              {
                class: "section-title",
                style: {
                  padding: 0,
                  fontFamily: "var(--font-mono)",
                  letterSpacing: "0.08em",
                  textTransform: "uppercase"
                },
                children: [
                  "ACTIVITY / 7x24 / ",
                  period.toUpperCase()
                ]
              }
            ),
            /* @__PURE__ */ u2(
              "span",
              {
                style: {
                  fontFamily: "var(--font-mono)",
                  fontSize: "11px",
                  color: "var(--text-secondary)",
                  letterSpacing: "0.04em"
                },
                children: [
                  active_days,
                  " active ",
                  active_days === 1 ? "day" : "days",
                  " \xB7 ",
                  avgPerDay,
                  " per active day"
                ]
              }
            )
          ]
        }
      ),
      /* @__PURE__ */ u2(
        "div",
        {
          style: {
            display: "grid",
            gridTemplateColumns: "28px repeat(24, 1fr)",
            gap: "2px"
          },
          children: [
            /* @__PURE__ */ u2("div", {}),
            Array.from({ length: 24 }, (_4, h5) => /* @__PURE__ */ u2(
              "div",
              {
                style: {
                  fontFamily: "var(--font-mono)",
                  fontSize: "9px",
                  color: "var(--text-secondary)",
                  textAlign: "center",
                  letterSpacing: "0.04em",
                  // Show only 0, 6, 12, 18 to avoid crowding
                  visibility: [0, 6, 12, 18].includes(h5) ? "visible" : "hidden"
                },
                children: String(h5).padStart(2, "0")
              },
              h5
            )),
            Array.from({ length: 7 }, (_4, dow) => /* @__PURE__ */ u2(S, { children: [
              /* @__PURE__ */ u2(
                "div",
                {
                  style: {
                    fontFamily: "var(--font-mono)",
                    fontSize: "9px",
                    color: "var(--text-secondary)",
                    display: "flex",
                    alignItems: "center",
                    letterSpacing: "0.04em"
                  },
                  children: DOW_LABELS[dow]
                },
                `label-${dow}`
              ),
              Array.from({ length: 24 }, (_5, hour) => {
                const cell = lookup.get(`${dow},${hour}`);
                const costNanos = cell?.cost_nanos ?? 0;
                const callCount = cell?.call_count ?? 0;
                const opacity = cellOpacity(costNanos, max_cost_nanos);
                const bg = withAlpha("--text-display", opacity);
                const costUsd = costNanos / 1e9;
                const title = `${DOW_LABELS[dow]} ${String(hour).padStart(2, "0")}:00 \u2014 ${fmtCost(costUsd)} / ${callCount} call${callCount !== 1 ? "s" : ""}`;
                return /* @__PURE__ */ u2(
                  "div",
                  {
                    title,
                    style: {
                      background: bg,
                      borderRadius: "2px",
                      aspectRatio: "1",
                      minHeight: "10px"
                    }
                  },
                  `${dow}-${hour}`
                );
              })
            ] }))
          ]
        }
      )
    ] });
  }

  // src/ui/components/SessionsTable.tsx
  var defaultSort = [{ id: "last", desc: true }];
  function useSessionColumns(showCredits) {
    return T2(
      () => [
        {
          id: "session",
          accessorKey: "session_id",
          header: "Session",
          enableSorting: false,
          cell: (info) => {
            const row = info.row.original;
            const title = row.title;
            return /* @__PURE__ */ u2("span", { class: "muted", style: { fontFamily: "monospace" }, title: title || void 0, children: title || /* @__PURE__ */ u2(S, { children: [
              info.getValue(),
              "\u2026"
            ] }) });
          }
        },
        {
          id: "project",
          accessorKey: "project",
          header: "Project",
          enableSorting: false,
          cell: (info) => {
            const row = info.row.original;
            const label = row.display_name || row.project;
            return /* @__PURE__ */ u2("span", { title: row.project, children: label });
          }
        },
        {
          id: "provider",
          accessorKey: "provider",
          header: "Provider",
          enableSorting: false,
          cell: (info) => /* @__PURE__ */ u2("span", { class: "model-tag", children: String(info.getValue()).toUpperCase() })
        },
        {
          id: "last",
          accessorKey: "last",
          header: "Last Active",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "muted", children: info.getValue() })
        },
        {
          id: "duration_min",
          accessorKey: "duration_min",
          header: "Duration",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "muted", children: [
            info.getValue(),
            "m"
          ] })
        },
        {
          id: "model",
          accessorKey: "model",
          header: "Model",
          enableSorting: false,
          cell: (info) => /* @__PURE__ */ u2("span", { class: "model-tag", children: info.getValue() })
        },
        {
          id: "turns",
          accessorKey: "turns",
          header: "Turns",
          cell: (info) => {
            const row = info.row.original;
            return /* @__PURE__ */ u2("span", { class: "num", children: [
              fmt(info.getValue()),
              row.subagent_count > 0 && /* @__PURE__ */ u2("span", { class: "muted", style: { fontSize: "10px" }, children: [
                " ",
                "(",
                row.subagent_count,
                " agents)"
              ] })
            ] });
          }
        },
        {
          id: "input",
          accessorKey: "input",
          header: "Input",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "output",
          accessorKey: "output",
          header: "Output",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "cost",
          accessorKey: "cost",
          header: "Est. Cost",
          cell: (info) => {
            const row = info.row.original;
            return row.is_billable ? /* @__PURE__ */ u2("span", { class: "cost", children: fmtCost(info.getValue()) }) : /* @__PURE__ */ u2("span", { class: "cost-na", children: "n/a" });
          }
        },
        ...showCredits ? [{
          id: "credits",
          accessorFn: (row) => row.credits ?? null,
          header: "Credits",
          sortUndefined: "last",
          cell: (info) => {
            const v4 = info.getValue();
            return /* @__PURE__ */ u2("span", { class: "num", children: fmtCredits(v4) });
          }
        }] : [],
        {
          id: "cost_meta",
          accessorKey: "cost_confidence",
          header: "Cost Meta",
          enableSorting: false,
          cell: (info) => {
            const row = info.row.original;
            return /* @__PURE__ */ u2("div", { class: "muted", style: { fontSize: "10px", lineHeight: "1.35" }, children: [
              /* @__PURE__ */ u2("div", { children: [
                row.cost_confidence || "low",
                " / ",
                row.billing_mode || "estimated_local"
              ] }),
              /* @__PURE__ */ u2("div", { children: row.pricing_version || "n/a" })
            ] });
          }
        },
        {
          id: "cache_hit_ratio",
          accessorKey: "cache_hit_ratio",
          header: "Cache %",
          cell: (info) => {
            const v4 = info.getValue();
            return /* @__PURE__ */ u2("span", { class: "num", children: [
              (v4 * 100).toFixed(0),
              "%"
            ] });
          }
        },
        {
          id: "tokens_per_min",
          accessorKey: "tokens_per_min",
          header: "Tok/min",
          cell: (info) => {
            const v4 = info.getValue();
            return /* @__PURE__ */ u2("span", { class: "num", children: v4 > 0 ? fmt(Math.round(v4)) : "--" });
          }
        }
      ],
      [showCredits]
    );
  }
  function SessionsTable({ onExportCSV }) {
    const data = lastFilteredSessions.value;
    const showCredits = anyHasCredits(data);
    const columns4 = useSessionColumns(showCredits);
    return /* @__PURE__ */ u2(
      DataTable,
      {
        columns: columns4,
        data,
        title: "Recent Sessions",
        exportFn: onExportCSV,
        pageSize: SESSIONS_PAGE_SIZE,
        defaultSort,
        enableColumnVisibility: true
      }
    );
  }

  // src/ui/components/ModelCostTable.tsx
  var defaultSort2 = [{ id: "cost", desc: true }];
  function CostShareBar({ value, max: max2, label }) {
    if (max2 <= 0 || value <= 0) return /* @__PURE__ */ u2("span", { class: "cost-na", children: "\u2014" });
    const pct = value / max2 * 100;
    return /* @__PURE__ */ u2("div", { style: { display: "flex", alignItems: "center", gap: "6px", minWidth: "100px" }, children: [
      /* @__PURE__ */ u2("span", { class: "num", style: { fontSize: "13px", minWidth: "52px", textAlign: "right" }, children: fmtCost(value) }),
      /* @__PURE__ */ u2(
        "div",
        {
          style: {
            flex: 1,
            height: "4px",
            background: "rgba(var(--text-primary-rgb,232,232,232),0.12)",
            borderRadius: "2px",
            overflow: "hidden"
          },
          "aria-label": label,
          children: /* @__PURE__ */ u2(
            "div",
            {
              style: {
                height: "100%",
                width: `${Math.min(100, pct).toFixed(1)}%`,
                background: "rgba(var(--text-primary-rgb,232,232,232),0.65)",
                borderRadius: "2px"
              }
            }
          )
        }
      )
    ] });
  }
  function useModelColumns(totalCost, totalCacheReadCost, totalCacheWriteCost, showCredits) {
    return T2(
      () => [
        {
          id: "model",
          accessorKey: "model",
          header: "Model",
          enableSorting: false,
          cell: (info) => /* @__PURE__ */ u2("span", { class: "model-tag", children: info.getValue() })
        },
        {
          id: "turns",
          accessorKey: "turns",
          header: "Turns",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "input",
          accessorKey: "input",
          header: "Input",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "output",
          accessorKey: "output",
          header: "Output",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "cache_read",
          accessorKey: "cache_read",
          header: "Cached Input",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "cache_creation",
          accessorKey: "cache_creation",
          header: "Cache Creation",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "cost",
          accessorKey: "cost",
          header: "Est. Cost",
          cell: (info) => {
            const row = info.row.original;
            return row.is_billable ? /* @__PURE__ */ u2("span", { class: "cost", children: fmtCost(info.getValue()) }) : /* @__PURE__ */ u2("span", { class: "cost-na", children: "n/a" });
          }
        },
        {
          id: "share",
          accessorFn: (row) => row.cost,
          header: "Share",
          enableSorting: false,
          cell: (info) => {
            const row = info.row.original;
            if (!row.is_billable || totalCost <= 0) {
              return /* @__PURE__ */ u2("span", { class: "cost-na", children: "\u2014" });
            }
            const pct = row.cost / totalCost * 100;
            return /* @__PURE__ */ u2("div", { style: { minWidth: "120px", display: "flex", alignItems: "center", gap: "8px" }, children: [
              /* @__PURE__ */ u2("div", { style: { flex: 1 }, children: /* @__PURE__ */ u2(
                SegmentedProgressBar,
                {
                  value: row.cost,
                  max: totalCost,
                  segments: 12,
                  size: "compact",
                  status: "neutral",
                  "aria-label": `${row.model} cost share`
                }
              ) }),
              /* @__PURE__ */ u2("span", { class: "num", style: { fontSize: "11px", color: "var(--text-secondary)", minWidth: "36px", textAlign: "right" }, children: [
                pct.toFixed(0),
                "%"
              ] })
            ] });
          }
        },
        // Phase 21: cache-read cost column with inline micro-bar
        {
          id: "cache_read_cost",
          accessorFn: (row) => row.cache_read_cost ?? 0,
          header: "Cache Read",
          cell: (info) => {
            const row = info.row.original;
            if (!row.is_billable) return /* @__PURE__ */ u2("span", { class: "cost-na", children: "\u2014" });
            return /* @__PURE__ */ u2(
              CostShareBar,
              {
                value: row.cache_read_cost ?? 0,
                max: totalCacheReadCost,
                label: `${row.model} cache-read cost share`
              }
            );
          }
        },
        // Phase 21: cache-write cost column with inline micro-bar
        {
          id: "cache_write_cost",
          accessorFn: (row) => row.cache_write_cost ?? 0,
          header: "Cache Write",
          cell: (info) => {
            const row = info.row.original;
            if (!row.is_billable) return /* @__PURE__ */ u2("span", { class: "cost-na", children: "\u2014" });
            return /* @__PURE__ */ u2(
              CostShareBar,
              {
                value: row.cache_write_cost ?? 0,
                max: totalCacheWriteCost,
                label: `${row.model} cache-write cost share`
              }
            );
          }
        },
        // Phase 12: credits column (hidden when no Amp rows in view)
        ...showCredits ? [{
          id: "credits",
          accessorFn: (row) => row.credits ?? null,
          header: "Credits",
          sortUndefined: "last",
          cell: (info) => {
            const v4 = info.getValue();
            return /* @__PURE__ */ u2("span", { class: "num", children: fmtCredits(v4) });
          }
        }] : []
      ],
      [totalCost, totalCacheReadCost, totalCacheWriteCost, showCredits]
    );
  }
  function ModelCostTable({ byModel }) {
    const totalCost = T2(
      () => byModel.reduce((s4, m4) => m4.is_billable ? s4 + m4.cost : s4, 0),
      [byModel]
    );
    const totalCacheReadCost = T2(
      () => byModel.reduce((s4, m4) => s4 + (m4.cache_read_cost ?? 0), 0),
      [byModel]
    );
    const totalCacheWriteCost = T2(
      () => byModel.reduce((s4, m4) => s4 + (m4.cache_write_cost ?? 0), 0),
      [byModel]
    );
    const showCredits = anyHasCredits(byModel);
    const columns4 = useModelColumns(totalCost, totalCacheReadCost, totalCacheWriteCost, showCredits);
    return /* @__PURE__ */ u2(
      DataTable,
      {
        columns: columns4,
        data: byModel,
        title: "Cost by Model",
        defaultSort: defaultSort2,
        costRows: true
      }
    );
  }

  // src/ui/components/ProjectCostTable.tsx
  var defaultSort3 = [{ id: "cost", desc: true }];
  function useProjectColumns(showCredits) {
    return T2(
      () => [
        {
          id: "project",
          accessorKey: "project",
          header: "Project",
          enableSorting: false,
          cell: (info) => {
            const row = info.row.original;
            const label = row.display_name || row.project;
            return /* @__PURE__ */ u2("span", { title: row.project, children: label });
          }
        },
        {
          id: "sessions",
          accessorKey: "sessions",
          header: "Sessions",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: info.getValue() })
        },
        {
          id: "turns",
          accessorKey: "turns",
          header: "Turns",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "input",
          accessorKey: "input",
          header: "Input",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "output",
          accessorKey: "output",
          header: "Output",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "num", children: fmt(info.getValue()) })
        },
        {
          id: "cost",
          accessorKey: "cost",
          header: "Est. Cost",
          cell: (info) => /* @__PURE__ */ u2("span", { class: "cost", children: fmtCost(info.getValue()) })
        },
        ...showCredits ? [{
          id: "credits",
          accessorFn: (row) => row.credits ?? null,
          header: "Credits",
          sortUndefined: "last",
          cell: (info) => {
            const v4 = info.getValue();
            return /* @__PURE__ */ u2("span", { class: "num", children: fmtCredits(v4) });
          }
        }] : []
      ],
      [showCredits]
    );
  }
  function ProjectCostTable({
    byProject,
    onExportCSV
  }) {
    const showCredits = anyHasCredits(byProject);
    const columns4 = useProjectColumns(showCredits);
    return /* @__PURE__ */ u2(
      DataTable,
      {
        columns: columns4,
        data: byProject,
        title: "Cost by Project",
        exportFn: onExportCSV,
        defaultSort: defaultSort3,
        costRows: true
      }
    );
  }

  // src/ui/components/DailyChart.tsx
  function DailyChart({ daily }) {
    const base = industrialChartOptions("bar");
    const options = {
      ...base,
      chart: { ...base.chart, type: "bar", stacked: true },
      series: [
        { name: "Input", data: daily.map((d5) => d5.input) },
        { name: "Output", data: daily.map((d5) => d5.output) },
        { name: "Cached Input", data: daily.map((d5) => d5.cache_read) },
        { name: "Cache Creation", data: daily.map((d5) => d5.cache_creation) }
      ],
      colors: tokenSeriesColors(),
      fill: { type: "solid" },
      plotOptions: { bar: { columnWidth: "70%", borderRadius: 0 } },
      xaxis: {
        ...base.xaxis,
        categories: daily.map((d5) => d5.day),
        labels: { ...base.xaxis.labels, rotate: -45, maxHeight: 60 },
        tickAmount: Math.min(daily.length, RANGE_TICKS[selectedRange.value])
      },
      yaxis: {
        ...base.yaxis,
        labels: { ...base.yaxis.labels, formatter: (v4) => fmt(v4) }
      },
      tooltip: { ...base.tooltip, y: { formatter: (v4) => fmt(v4) + " tokens" } }
    };
    return /* @__PURE__ */ u2(ApexChart, { options, id: "chart-daily" });
  }

  // src/ui/components/WeeklyChart.tsx
  function WeeklyChart({ weekly }) {
    if (!weekly?.length) {
      return /* @__PURE__ */ u2("div", { style: { padding: "24px", color: "var(--text-muted)", fontFamily: "var(--font-mono)", fontSize: "12px" }, children: "No weekly data available." });
    }
    const base = industrialChartOptions("bar");
    const options = {
      ...base,
      chart: { ...base.chart, type: "bar", stacked: true },
      series: [
        { name: "Input", data: weekly.map((w5) => w5.input) },
        { name: "Output", data: weekly.map((w5) => w5.output) },
        { name: "Cached Input", data: weekly.map((w5) => w5.cache_read) },
        { name: "Cache Creation", data: weekly.map((w5) => w5.cache_creation) }
      ],
      colors: tokenSeriesColors(),
      fill: { type: "solid" },
      plotOptions: { bar: { columnWidth: "70%", borderRadius: 0 } },
      xaxis: {
        ...base.xaxis,
        categories: weekly.map((w5) => w5.week),
        labels: { ...base.xaxis.labels, rotate: -45, maxHeight: 60 },
        tickAmount: Math.min(weekly.length, 26)
      },
      yaxis: {
        ...base.yaxis,
        labels: { ...base.yaxis.labels, formatter: (v4) => fmt(v4) }
      },
      tooltip: {
        ...base.tooltip,
        y: { formatter: (v4) => fmt(v4) + " tokens" },
        custom: ({ dataPointIndex }) => {
          const w5 = weekly[dataPointIndex];
          if (!w5) return "";
          const total = w5.input + w5.output + w5.cache_read + w5.cache_creation;
          const costUsd = w5.cost_nanos / 1e9;
          const costStr = costUsd < 1e-4 ? "<$0.0001" : "$" + costUsd.toFixed(4);
          return '<div style="padding:8px 12px;font-family:var(--font-mono);font-size:12px;background:var(--color-bg-secondary);border:1px solid var(--color-border)"><div style="margin-bottom:4px;font-weight:600">' + w5.week + "</div><div>Input: " + fmt(w5.input) + "</div><div>Output: " + fmt(w5.output) + "</div><div>Cached Input: " + fmt(w5.cache_read) + "</div><div>Cache Creation: " + fmt(w5.cache_creation) + '</div><div style="margin-top:4px;border-top:1px solid var(--color-border);padding-top:4px">Total: ' + fmt(total) + " tokens</div><div>Cost: " + costStr + "</div></div>";
        }
      }
    };
    return /* @__PURE__ */ u2(ApexChart, { options, id: "chart-weekly" });
  }

  // src/ui/components/ModelChart.tsx
  function ModelChart({ byModel }) {
    if (!byModel.length) return null;
    const sorted = [...byModel].sort((a4, b4) => b4.input + b4.output - (a4.input + a4.output));
    const TOP_N = 4;
    const top = sorted.slice(0, TOP_N);
    const rest = sorted.slice(TOP_N);
    const series = top.map((m4) => m4.input + m4.output);
    const labels = top.map((m4) => m4.model);
    if (rest.length > 0) {
      const otherTotal = rest.reduce((s4, m4) => s4 + m4.input + m4.output, 0);
      if (otherTotal > 0) {
        series.push(otherTotal);
        labels.push(`Other (${rest.length})`);
      }
    }
    const base = industrialChartOptions("donut");
    const options = {
      ...base,
      chart: { ...base.chart, type: "donut" },
      series,
      labels,
      colors: modelSeriesColors(labels.length),
      stroke: { width: 2, colors: [cssVar("--surface")] },
      plotOptions: {
        pie: {
          donut: {
            size: "64%",
            labels: {
              show: true,
              total: {
                show: true,
                label: "TOTAL",
                fontFamily: 'var(--font-mono), "Space Mono", monospace',
                fontSize: "11px",
                color: cssVar("--text-secondary"),
                formatter: (w5) => fmt(w5.globals.seriesTotals.reduce((a4, b4) => a4 + b4, 0))
              },
              value: {
                fontFamily: 'var(--font-mono), "Space Mono", monospace',
                fontSize: "20px",
                color: cssVar("--text-display"),
                formatter: (val) => fmt(Number(val))
              },
              name: {
                fontFamily: 'var(--font-mono), "Space Mono", monospace',
                fontSize: "11px",
                color: cssVar("--text-secondary")
              }
            }
          }
        }
      },
      tooltip: { ...base.tooltip, y: { formatter: (v4) => fmt(v4) + " tokens" } }
    };
    return /* @__PURE__ */ u2(ApexChart, { options, id: "chart-model" });
  }

  // src/ui/components/ProjectChart.tsx
  function ProjectChart({ byProject }) {
    const top = byProject.slice(0, 10);
    if (!top.length) return null;
    const base = industrialChartOptions("bar");
    const colors = tokenSeriesColors();
    const options = {
      ...base,
      chart: { ...base.chart, type: "bar" },
      series: [
        { name: "Input", data: top.map((p5) => p5.input) },
        { name: "Output", data: top.map((p5) => p5.output) }
      ],
      colors: [colors[0], colors[1]],
      fill: { type: "solid" },
      plotOptions: { bar: { horizontal: true, barHeight: "60%", borderRadius: 0 } },
      xaxis: {
        ...base.xaxis,
        categories: top.map((p5) => {
          const n3 = p5.display_name || p5.project;
          return n3.length > 16 ? "\u2026" + n3.slice(-14) : n3;
        }),
        labels: { ...base.xaxis.labels, formatter: (v4) => fmt(v4) }
      },
      yaxis: {
        ...base.yaxis,
        labels: { ...base.yaxis.labels, maxWidth: 110 }
      },
      tooltip: { ...base.tooltip, y: { formatter: (v4) => fmt(v4) + " tokens" } }
    };
    return /* @__PURE__ */ u2(ApexChart, { options, id: "chart-project" });
  }

  // src/ui/components/CostReconciliationPanel.tsx
  function CostReconciliationPanel({ data }) {
    if (!data || !data.enabled) return null;
    if (data.hook_total_nanos == null || data.local_total_nanos == null) return null;
    const hookUsd = data.hook_total_nanos / 1e9;
    const localUsd = data.local_total_nanos / 1e9;
    const divergence = data.divergence_pct ?? 0;
    const divergencePctSigned = divergence * 100;
    const divergencePctAbs = Math.abs(divergencePctSigned);
    const isWarn = divergencePctAbs > 10;
    return /* @__PURE__ */ u2("div", { class: "card card-flat", style: { gridColumn: "1 / -1" }, children: [
      /* @__PURE__ */ u2("div", { style: { marginBottom: "16px" }, children: [
        /* @__PURE__ */ u2("span", { style: {
          fontFamily: "var(--font-mono)",
          fontSize: "11px",
          fontWeight: 400,
          textTransform: "uppercase",
          letterSpacing: "0.08em",
          color: "var(--text-secondary)"
        }, children: "COST RECONCILIATION" }),
        data.period && /* @__PURE__ */ u2("span", { style: {
          fontFamily: "var(--font-mono)",
          fontSize: "11px",
          color: "var(--text-disabled)",
          marginLeft: "8px"
        }, children: [
          "(",
          data.period,
          ")"
        ] })
      ] }),
      /* @__PURE__ */ u2("div", { style: { display: "flex", gap: "32px", flexWrap: "wrap", marginBottom: "20px" }, children: [
        /* @__PURE__ */ u2("div", { children: [
          /* @__PURE__ */ u2("div", { style: {
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            textTransform: "uppercase",
            letterSpacing: "0.08em",
            color: "var(--text-secondary)",
            marginBottom: "8px"
          }, children: "HOOK-REPORTED" }),
          /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "24px" }, children: fmtCost(hookUsd) })
        ] }),
        /* @__PURE__ */ u2("div", { children: [
          /* @__PURE__ */ u2("div", { style: {
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            textTransform: "uppercase",
            letterSpacing: "0.08em",
            color: "var(--text-secondary)",
            marginBottom: "8px"
          }, children: "LOCAL ESTIMATE" }),
          /* @__PURE__ */ u2("div", { class: "stat-value", style: { fontSize: "24px" }, children: fmtCost(localUsd) })
        ] }),
        /* @__PURE__ */ u2("div", { children: [
          /* @__PURE__ */ u2("div", { style: {
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            textTransform: "uppercase",
            letterSpacing: "0.08em",
            color: "var(--text-secondary)",
            marginBottom: "8px"
          }, children: "DIVERGENCE" }),
          /* @__PURE__ */ u2(
            "div",
            {
              class: "stat-value",
              style: { fontSize: "24px", color: isWarn ? "var(--accent)" : void 0 },
              children: [
                divergencePctSigned >= 0 ? "+" : "",
                divergencePctSigned.toFixed(1),
                "%",
                isWarn && /* @__PURE__ */ u2("span", { style: {
                  display: "inline-block",
                  marginLeft: "8px",
                  fontFamily: "var(--font-mono)",
                  fontSize: "11px",
                  fontWeight: 400,
                  letterSpacing: "0.06em",
                  padding: "2px 8px",
                  border: "1px solid var(--accent)",
                  borderRadius: "4px",
                  color: "var(--accent)",
                  verticalAlign: "middle"
                }, children: "[DRIFT]" })
              ]
            }
          )
        ] })
      ] }),
      data.breakdown && data.breakdown.length > 0 && /* @__PURE__ */ u2("div", { style: { overflowX: "auto" }, children: /* @__PURE__ */ u2("table", { children: [
        /* @__PURE__ */ u2("thead", { children: /* @__PURE__ */ u2("tr", { children: [
          /* @__PURE__ */ u2("th", { children: "DAY" }),
          /* @__PURE__ */ u2("th", { style: { textAlign: "right" }, children: "HOOK" }),
          /* @__PURE__ */ u2("th", { style: { textAlign: "right" }, children: "LOCAL" }),
          /* @__PURE__ */ u2("th", { style: { textAlign: "right" }, children: "\u0394" })
        ] }) }),
        /* @__PURE__ */ u2("tbody", { children: data.breakdown.slice().reverse().slice(0, 30).map((r4) => {
          const h5 = r4.hook_nanos / 1e9;
          const l5 = r4.local_nanos / 1e9;
          const delta = h5 - l5;
          const rowWarn = l5 > 1e-9 && Math.abs(delta) / l5 > 0.1;
          return /* @__PURE__ */ u2("tr", { children: [
            /* @__PURE__ */ u2("td", { class: "num", children: r4.day }),
            /* @__PURE__ */ u2("td", { class: "num", style: { textAlign: "right" }, children: fmtCost(h5) }),
            /* @__PURE__ */ u2("td", { class: "num", style: { textAlign: "right" }, children: fmtCost(l5) }),
            /* @__PURE__ */ u2(
              "td",
              {
                class: "num",
                style: {
                  textAlign: "right",
                  color: rowWarn ? "var(--accent)" : "var(--text-secondary)"
                },
                children: [
                  delta >= 0 ? "+" : "",
                  fmtCost(delta)
                ]
              }
            )
          ] }, r4.day);
        }) })
      ] }) })
    ] });
  }

  // src/ui/lib/csv.ts
  function csvField(val) {
    const s4 = String(val);
    const needsPrefix = /^[=+\-@\t\r]/.test(s4);
    const escaped = needsPrefix ? "'" + s4 : s4;
    if (escaped.includes(",") || escaped.includes('"') || escaped.includes("\n")) {
      return '"' + escaped.replace(/"/g, '""') + '"';
    }
    return escaped;
  }
  function csvTimestamp() {
    const d5 = /* @__PURE__ */ new Date();
    return d5.getFullYear() + "-" + String(d5.getMonth() + 1).padStart(2, "0") + "-" + String(d5.getDate()).padStart(2, "0") + "_" + String(d5.getHours()).padStart(2, "0") + String(d5.getMinutes()).padStart(2, "0");
  }
  function downloadCSV(reportType, header, rows) {
    const lines = [header.map(csvField).join(",")];
    for (const row of rows) lines.push(row.map(csvField).join(","));
    const blob = new Blob([lines.join("\n")], { type: "text/csv;charset=utf-8;" });
    const a4 = document.createElement("a");
    a4.href = URL.createObjectURL(blob);
    a4.download = reportType + "_" + csvTimestamp() + ".csv";
    a4.click();
    setTimeout(() => URL.revokeObjectURL(a4.href), 1e3);
  }

  // src/ui/lib/theme.ts
  function getTheme() {
    const stored = localStorage.getItem("theme");
    if (stored === "light" || stored === "dark") return stored;
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  function applyTheme(theme) {
    if (theme === "light") {
      document.documentElement.setAttribute("data-theme", "light");
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
    themeMode.value = theme;
  }

  // src/ui/app.tsx
  applyTheme(getTheme());
  function toggleTheme() {
    const current = document.documentElement.getAttribute("data-theme") === "light" ? "light" : "dark";
    const next = current === "light" ? "dark" : "light";
    localStorage.setItem("theme", next);
    applyTheme(next);
    if (rawData.value) applyFilter();
  }
  var previousSessionPercent = null;
  var loadDataInFlight = false;
  var loadUsageWindowsInFlight = false;
  var loadHeatmapInFlight = false;
  var lastHeatmapData = null;
  var loadAgentStatusInFlight = false;
  var loadCommunitySignalInFlight = false;
  var lastCommunitySignal = null;
  var loadBillingBlocksInFlight = false;
  var loadContextWindowInFlight = false;
  var loadCostReconciliationInFlight = false;
  function getRangeCutoff(range) {
    if (range === "all") return null;
    const days = range === "7d" ? 7 : range === "30d" ? 30 : 90;
    const d5 = /* @__PURE__ */ new Date();
    d5.setDate(d5.getDate() - days);
    return d5.toISOString().slice(0, 10);
  }
  function readURLRange() {
    const p5 = new URLSearchParams(window.location.search).get("range");
    return ["7d", "30d", "90d", "all"].includes(p5) ? p5 : "30d";
  }
  function readURLProvider() {
    const p5 = new URLSearchParams(window.location.search).get("provider");
    return ["claude", "codex", "both"].includes(p5) ? p5 : "both";
  }
  function readURLModels(allModels) {
    const param = new URLSearchParams(window.location.search).get("models");
    if (!param) return new Set(allModels);
    const fromURL = new Set(param.split(",").map((s4) => s4.trim()).filter(Boolean));
    return new Set(allModels.filter((m4) => fromURL.has(m4)));
  }
  function matchesProvider(row) {
    const p5 = selectedProvider.value;
    if (p5 === "both") return true;
    return row.provider === p5;
  }
  function isDefaultModelSelection(allModels) {
    if (selectedModels.value.size !== allModels.length) return false;
    return allModels.every((m4) => selectedModels.value.has(m4));
  }
  function updateURL() {
    const allModels = rawData.value?.all_models ?? [];
    const params = new URLSearchParams();
    if (selectedRange.value !== "30d") params.set("range", selectedRange.value);
    if (selectedProvider.value !== "both") params.set("provider", selectedProvider.value);
    if (!isDefaultModelSelection(allModels)) params.set("models", Array.from(selectedModels.value).join(","));
    if (projectSearchQuery.value) params.set("project", projectSearchQuery.value);
    if (versionDonutMetric.value !== "cost") params.set("version_metric", versionDonutMetric.value);
    if (selectedBucket.value !== "day") params.set("bucket", selectedBucket.value);
    const search = params.toString() ? "?" + params.toString() : "";
    history.replaceState(null, "", window.location.pathname + search);
  }
  function matchesProjectSearch(project, displayName) {
    if (!projectSearchQuery.value) return true;
    const q3 = projectSearchQuery.value;
    if (project.toLowerCase().includes(q3)) return true;
    if (displayName && displayName.toLowerCase().includes(q3)) return true;
    return false;
  }
  function weekLabelToWeekStart(label) {
    const [yearStr, weekStr] = label.split("-");
    const year = parseInt(yearStr, 10);
    const week = parseInt(weekStr, 10);
    if (!Number.isFinite(year) || !Number.isFinite(week)) return /* @__PURE__ */ new Date(NaN);
    const jan1 = new Date(Date.UTC(year, 0, 1));
    if (week === 0) return jan1;
    const jan1Dow = jan1.getUTCDay();
    const daysToFirstMon = (8 - jan1Dow) % 7 || 7;
    const firstMondayUtc = new Date(Date.UTC(year, 0, 1 + daysToFirstMon));
    return new Date(firstMondayUtc.getTime() + (week - 1) * 7 * 86400 * 1e3);
  }
  function buildWeeklyAgg(range) {
    const rows = rawData.value?.weekly_by_model ?? [];
    if (!rows.length) return [];
    const cutoff = getRangeCutoff(range);
    const weekMap = {};
    for (const r4 of rows) {
      if (cutoff) {
        const weekStart = weekLabelToWeekStart(r4.week);
        if (isNaN(weekStart.getTime())) continue;
        const weekStartStr = weekStart.toISOString().slice(0, 10);
        if (weekStartStr < cutoff) continue;
      }
      const w5 = weekMap[r4.week] ?? (weekMap[r4.week] = {
        week: r4.week,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        cost_nanos: 0
      });
      w5.input += r4.input_tokens;
      w5.output += r4.output_tokens;
      w5.cache_read += r4.cache_read_tokens;
      w5.cache_creation += r4.cache_creation_tokens;
      w5.reasoning_output += r4.reasoning_output_tokens;
      w5.cost_nanos += r4.cost_nanos;
    }
    return Object.values(weekMap).sort((a4, b4) => a4.week.localeCompare(b4.week));
  }
  function buildAggregations(filteredDaily, filteredSessions) {
    const dailyMap = {};
    for (const r4 of filteredDaily) {
      const d5 = dailyMap[r4.day] ?? (dailyMap[r4.day] = {
        day: r4.day,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0
      });
      d5.input += r4.input;
      d5.output += r4.output;
      d5.cache_read += r4.cache_read;
      d5.cache_creation += r4.cache_creation;
      d5.reasoning_output += r4.reasoning_output;
    }
    const daily = Object.values(dailyMap).sort((a4, b4) => a4.day.localeCompare(b4.day));
    const modelMap = {};
    for (const r4 of filteredDaily) {
      const m4 = modelMap[r4.model] ?? (modelMap[r4.model] = {
        model: r4.model,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 0,
        sessions: 0,
        cost: 0,
        is_billable: r4.cost > 0,
        input_cost: 0,
        output_cost: 0,
        cache_read_cost: 0,
        cache_write_cost: 0,
        credits: null
      });
      m4.input += r4.input;
      m4.output += r4.output;
      m4.cache_read += r4.cache_read;
      m4.cache_creation += r4.cache_creation;
      m4.reasoning_output += r4.reasoning_output;
      m4.turns += r4.turns;
      m4.cost += r4.cost;
      if (r4.cost > 0) m4.is_billable = true;
      m4.input_cost = (m4.input_cost ?? 0) + (r4.input_cost ?? 0);
      m4.output_cost = (m4.output_cost ?? 0) + (r4.output_cost ?? 0);
      m4.cache_read_cost = (m4.cache_read_cost ?? 0) + (r4.cache_read_cost ?? 0);
      m4.cache_write_cost = (m4.cache_write_cost ?? 0) + (r4.cache_write_cost ?? 0);
      if (r4.credits != null) {
        m4.credits = (m4.credits ?? 0) + r4.credits;
      }
    }
    for (const s4 of filteredSessions) {
      const m4 = modelMap[s4.model];
      if (m4) m4.sessions++;
    }
    const byModel = Object.values(modelMap).sort((a4, b4) => b4.input + b4.output - (a4.input + a4.output));
    const projMap = {};
    for (const s4 of filteredSessions) {
      const p5 = projMap[s4.project] ?? (projMap[s4.project] = {
        project: s4.project,
        display_name: s4.display_name || s4.project,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 0,
        sessions: 0,
        cost: 0,
        credits: null
      });
      p5.input += s4.input;
      p5.output += s4.output;
      p5.cache_read += s4.cache_read;
      p5.cache_creation += s4.cache_creation;
      p5.reasoning_output += s4.reasoning_output;
      p5.turns += s4.turns;
      p5.sessions++;
      p5.cost += s4.cost;
      if (s4.credits != null) {
        p5.credits = (p5.credits ?? 0) + s4.credits;
      }
    }
    const byProject = Object.values(projMap).sort((a4, b4) => b4.input + b4.output - (a4.input + a4.output));
    const totals = {
      sessions: filteredSessions.length,
      turns: byModel.reduce((s4, m4) => s4 + m4.turns, 0),
      input: byModel.reduce((s4, m4) => s4 + m4.input, 0),
      output: byModel.reduce((s4, m4) => s4 + m4.output, 0),
      cache_read: byModel.reduce((s4, m4) => s4 + m4.cache_read, 0),
      cache_creation: byModel.reduce((s4, m4) => s4 + m4.cache_creation, 0),
      reasoning_output: byModel.reduce((s4, m4) => s4 + m4.reasoning_output, 0),
      cost: filteredSessions.reduce((s4, sess) => s4 + sess.cost, 0)
    };
    const confidenceBreakdown = Object.entries(
      filteredSessions.reduce((acc, session) => {
        const key = session.cost_confidence || "low";
        if (!acc[key]) acc[key] = { sessions: 0, cost: 0 };
        acc[key].sessions += 1;
        acc[key].cost += session.cost;
        return acc;
      }, {})
    ).sort(([a4], [b4]) => confidenceRank(a4) - confidenceRank(b4));
    const billingModeBreakdown = Object.entries(
      filteredSessions.reduce((acc, session) => {
        const key = session.billing_mode || "estimated_local";
        if (!acc[key]) acc[key] = { sessions: 0, cost: 0 };
        acc[key].sessions += 1;
        acc[key].cost += session.cost;
        return acc;
      }, {})
    ).sort((a4, b4) => b4[1].sessions - a4[1].sessions);
    const pricingVersions = Array.from(
      new Set(filteredSessions.map((session) => session.pricing_version).filter(Boolean))
    );
    return { daily, byModel, byProject, totals, confidenceBreakdown, billingModeBreakdown, pricingVersions };
  }
  function confidenceRank(confidence) {
    switch (confidence) {
      case "low":
        return 0;
      case "medium":
        return 1;
      case "high":
        return 2;
      default:
        return 3;
    }
  }
  function renderEstimationMeta(confidenceBreakdown, billingModeBreakdown, pricingVersions) {
    const container = $2("estimation-meta");
    if (!container) return;
    if (!confidenceBreakdown.length && !billingModeBreakdown.length && !pricingVersions.length) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "grid";
    R(
      /* @__PURE__ */ u2(
        EstimationMeta,
        {
          confidenceBreakdown,
          billingModeBreakdown,
          pricingVersions
        }
      ),
      container
    );
  }
  function renderOpenAiReconciliation(reconciliation) {
    const container = $2("openai-reconciliation");
    if (!container) return;
    if (!reconciliation) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(ReconciliationBlock, { reconciliation }), container);
  }
  function applyFilter() {
    if (!rawData.value) return;
    const cutoff = getRangeCutoff(selectedRange.value);
    const filteredDaily = rawData.value.daily_by_model.filter(
      (r4) => selectedModels.value.has(r4.model) && (!cutoff || r4.day >= cutoff) && matchesProvider(r4)
    );
    const filteredSessions = rawData.value.sessions_all.filter(
      (s4) => selectedModels.value.has(s4.model) && (!cutoff || s4.last_date >= cutoff) && matchesProjectSearch(s4.project, s4.display_name) && matchesProvider(s4)
    );
    const { daily, byModel, byProject, totals, confidenceBreakdown, billingModeBreakdown, pricingVersions } = buildAggregations(filteredDaily, filteredSessions);
    const providerLabel = selectedProvider.value === "both" ? "" : ` (${selectedProvider.value})`;
    const bucketIsWeek = selectedBucket.value === "week";
    const chartTitleEl = $2("daily-chart-title");
    if (chartTitleEl) {
      chartTitleEl.textContent = (bucketIsWeek ? "Weekly Token Usage - " : "Daily Token Usage - ") + RANGE_LABELS[selectedRange.value] + providerLabel;
    }
    R(
      /* @__PURE__ */ u2(
        StatsCards,
        {
          totals,
          daily,
          activeDays: lastHeatmapData?.active_days,
          heatmapTotalNanos: lastHeatmapData?.total_cost_nanos,
          cacheEfficiency: rawData.value?.cache_efficiency,
          billingBlocks: billingBlocksData.value,
          contextWindow: contextWindowData.value
        }
      ),
      $2("stats-row")
    );
    renderEstimationMeta(confidenceBreakdown, billingModeBreakdown, pricingVersions);
    renderOpenAiReconciliation(rawData.value.openai_reconciliation);
    if (bucketIsWeek) {
      const weekly = buildWeeklyAgg(selectedRange.value);
      R(/* @__PURE__ */ u2(WeeklyChart, { weekly }), $2("chart-daily"));
    } else {
      R(/* @__PURE__ */ u2(DailyChart, { daily }), $2("chart-daily"));
    }
    R(/* @__PURE__ */ u2(ModelChart, { byModel }), $2("chart-model"));
    R(/* @__PURE__ */ u2(ProjectChart, { byProject }), $2("chart-project"));
    lastFilteredSessions.value = filteredSessions;
    lastByProject.value = byProject;
    R(/* @__PURE__ */ u2(ModelCostTable, { byModel }), $2("model-cost-mount"));
    R(/* @__PURE__ */ u2(SessionsTable, { onExportCSV: exportSessionsCSV }), $2("sessions-mount"));
    R(/* @__PURE__ */ u2(ProjectCostTable, { byProject: lastByProject.value.slice(0, 30), onExportCSV: exportProjectsCSV }), $2("project-cost-mount"));
    if (rawData.value.subagent_summary) renderSubagentSummary(rawData.value.subagent_summary);
    renderEntrypointBreakdown((rawData.value.entrypoint_breakdown ?? []).filter(matchesProvider));
    renderServiceTiers((rawData.value.service_tiers ?? []).filter(matchesProvider));
    renderToolSummary((rawData.value.tool_summary ?? []).filter(matchesProvider));
    renderMcpSummary((rawData.value.mcp_summary ?? []).filter(matchesProvider));
    renderBranchSummary((rawData.value.git_branch_summary ?? []).filter(matchesProvider));
    renderVersionSummary((rawData.value.version_summary ?? []).filter(matchesProvider));
    renderHourlyChart((rawData.value.hourly_distribution ?? []).filter(matchesProvider));
  }
  function exportSessionsCSV() {
    const header = ["Session", "Provider", "Project", "Last Active", "Duration (min)", "Model", "Turns", "Input", "Output", "Cached Input", "Cache Creation", "Reasoning Output", "Est. Cost"];
    const rows = lastFilteredSessions.value.map((s4) => {
      const cost = s4.cost;
      return [s4.session_id, s4.provider, s4.project, s4.last, s4.duration_min, s4.model, s4.turns, s4.input, s4.output, s4.cache_read, s4.cache_creation, s4.reasoning_output, cost.toFixed(4)];
    });
    downloadCSV("sessions", header, rows);
  }
  function exportProjectRowsCSV(filename, rowsData) {
    const header = ["Project", "Sessions", "Turns", "Input", "Output", "Cached Input", "Cache Creation", "Reasoning Output", "Est. Cost"];
    const rows = rowsData.map(
      (p5) => [p5.project, p5.sessions, p5.turns, p5.input, p5.output, p5.cache_read, p5.cache_creation, p5.reasoning_output, p5.cost.toFixed(4)]
    );
    downloadCSV(filename, header, rows);
  }
  function exportProjectsCSV() {
    exportProjectRowsCSV("projects", lastByProject.value);
  }
  function renderUsageWindows(data) {
    const container = $2("usage-windows");
    if (!container) return;
    if (!data.available) {
      planBadge.value = "";
      if (data.error) {
        container.style.display = "grid";
        R(/* @__PURE__ */ u2(RateWindowUnavailable, { error: data.error }), container);
      } else {
        container.style.display = "none";
        R(null, container);
      }
      return;
    }
    container.style.display = "grid";
    R(
      /* @__PURE__ */ u2(S, { children: [
        data.session && /* @__PURE__ */ u2(RateWindowCard, { label: "Session (5h)", window: data.session }),
        data.weekly && /* @__PURE__ */ u2(RateWindowCard, { label: "Weekly", window: data.weekly }),
        data.weekly_opus && /* @__PURE__ */ u2(RateWindowCard, { label: "Weekly Opus", window: data.weekly_opus }),
        data.weekly_sonnet && /* @__PURE__ */ u2(RateWindowCard, { label: "Weekly Sonnet", window: data.weekly_sonnet }),
        data.budget && /* @__PURE__ */ u2(
          BudgetCard,
          {
            used: data.budget.used,
            limit: data.budget.limit,
            currency: data.budget.currency,
            utilization: data.budget.utilization
          }
        ),
        /* @__PURE__ */ u2("div", { style: { gridColumn: "1 / -1" }, children: /* @__PURE__ */ u2(InlineStatus, { placement: "rate-windows" }) })
      ] }),
      container
    );
    if (data.session) {
      const currentPercent = 100 - data.session.used_percent;
      if (previousSessionPercent !== null) {
        if (previousSessionPercent > 0.01 && currentPercent <= 0.01) {
          setStatus(
            "rate-windows",
            "error",
            "Session depleted \u2014 resets in " + (data.session.resets_in_minutes ?? 0) + "m"
          );
        } else if (previousSessionPercent <= 0.01 && currentPercent > 0.01) {
          setStatus("rate-windows", "success", "Session restored", 4e3);
        }
      }
      previousSessionPercent = currentPercent;
    }
    planBadge.value = data.identity?.plan ? data.identity.plan.charAt(0).toUpperCase() + data.identity.plan.slice(1) : "";
  }
  function renderSubagentSummary(summary) {
    const container = $2("subagent-summary");
    if (!container) return;
    if (summary.subagent_turns === 0) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(SubagentSummary, { summary }), container);
  }
  function renderEntrypointBreakdown(data) {
    const container = $2("entrypoint-breakdown");
    if (!container) return;
    if (!data.length) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(EntrypointTable, { data }), container);
  }
  function renderServiceTiers(data) {
    const container = $2("service-tiers");
    if (!container) return;
    if (!data.length) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(ServiceTiersTable, { data }), container);
  }
  function renderToolSummary(data) {
    const container = $2("tool-summary");
    if (!container) return;
    if (!data.length) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(ToolUsageTable, { data }), container);
  }
  function renderMcpSummary(data) {
    const container = $2("mcp-summary");
    if (!container) return;
    if (!data.length) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(McpSummaryTable, { data }), container);
  }
  function renderBranchSummary(data) {
    const container = $2("branch-summary");
    if (!container) return;
    if (!data.length) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(BranchTable, { data }), container);
  }
  function renderVersionSummary(data) {
    const container = $2("version-summary");
    if (!container) return;
    if (!data.length) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    const handleMetricChange = (next) => {
      versionDonutMetric.value = next;
      updateURL();
      renderVersionSummary(data);
    };
    R(
      /* @__PURE__ */ u2("div", { style: { display: "flex", gap: "24px", alignItems: "flex-start", flexWrap: "wrap" }, children: [
        /* @__PURE__ */ u2("div", { style: { flex: "1 1 260px", minWidth: "220px", height: "300px" }, children: /* @__PURE__ */ u2(
          VersionDonut,
          {
            rows: data,
            metric: versionDonutMetric.value,
            onMetricChange: handleMetricChange
          }
        ) }),
        /* @__PURE__ */ u2("div", { style: { flex: "2 1 320px", minWidth: "280px" }, children: /* @__PURE__ */ u2(VersionTable, { data }) })
      ] }),
      container
    );
  }
  function renderHourlyChart(data) {
    const container = $2("hourly-chart");
    if (!container) return;
    if (!data.length) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(HourlyChart, { data }), container);
  }
  function renderActivityHeatmap(data) {
    lastHeatmapData = data;
    const container = $2("activity-heatmap");
    if (!container) return;
    if (!data) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(ActivityHeatmap, { data }), container);
    if (rawData.value) applyFilter();
  }
  async function loadUsageWindows() {
    if (loadUsageWindowsInFlight) return;
    loadUsageWindowsInFlight = true;
    try {
      const resp = await fetch("/api/usage-windows");
      if (!resp.ok) return;
      const data = await resp.json();
      renderUsageWindows(data);
    } catch {
    } finally {
      loadUsageWindowsInFlight = false;
    }
  }
  function renderAgentStatus(snapshot) {
    const container = $2("agent-status");
    if (!container) return;
    container.style.display = "grid";
    R(/* @__PURE__ */ u2(AgentStatusCard, { snapshot, communitySignal: lastCommunitySignal }), container);
  }
  async function loadAgentStatus() {
    if (loadAgentStatusInFlight) return;
    loadAgentStatusInFlight = true;
    try {
      const resp = await fetch("/api/agent-status");
      if (!resp.ok) return;
      const data = await resp.json();
      renderAgentStatus(data);
    } catch {
    } finally {
      loadAgentStatusInFlight = false;
    }
  }
  async function loadCommunitySignal() {
    if (loadCommunitySignalInFlight) return;
    loadCommunitySignalInFlight = true;
    try {
      const resp = await fetch("/api/community-signal");
      if (!resp.ok) return;
      const data = await resp.json();
      lastCommunitySignal = data.enabled ? data : null;
      const container = $2("agent-status");
      if (container && container.style.display !== "none") {
        const existing = container.querySelector("[aria-label]");
        if (existing) {
          loadAgentStatus();
        }
      }
    } catch {
    } finally {
      loadCommunitySignalInFlight = false;
    }
  }
  async function loadBillingBlocks() {
    if (loadBillingBlocksInFlight) return;
    loadBillingBlocksInFlight = true;
    try {
      const resp = await fetch("/api/billing-blocks");
      if (!resp.ok) {
        billingBlocksData.value = null;
        return;
      }
      const data = await resp.json();
      billingBlocksData.value = data;
      if (rawData.value) applyFilter();
    } catch {
      billingBlocksData.value = null;
    } finally {
      loadBillingBlocksInFlight = false;
    }
  }
  async function loadContextWindow() {
    if (loadContextWindowInFlight) return;
    loadContextWindowInFlight = true;
    try {
      const resp = await fetch("/api/context-window");
      if (!resp.ok) {
        contextWindowData.value = null;
        return;
      }
      const data = await resp.json();
      contextWindowData.value = data;
      if (rawData.value) applyFilter();
    } catch {
      contextWindowData.value = null;
    } finally {
      loadContextWindowInFlight = false;
    }
  }
  async function loadCostReconciliation() {
    if (loadCostReconciliationInFlight) return;
    loadCostReconciliationInFlight = true;
    try {
      const resp = await fetch("/api/cost-reconciliation?period=month");
      if (!resp.ok) {
        costReconciliationData.value = null;
        return;
      }
      const data = await resp.json();
      costReconciliationData.value = data;
      renderCostReconciliation();
    } catch {
      costReconciliationData.value = null;
    } finally {
      loadCostReconciliationInFlight = false;
    }
  }
  function renderCostReconciliation() {
    const container = $2("cost-reconciliation");
    if (!container) return;
    const data = costReconciliationData.value;
    if (!data || !data.enabled) {
      container.style.display = "none";
      R(null, container);
      return;
    }
    container.style.display = "";
    R(/* @__PURE__ */ u2(CostReconciliationPanel, { data }), container);
  }
  async function loadHeatmap(period = "month") {
    if (loadHeatmapInFlight) return;
    loadHeatmapInFlight = true;
    try {
      const tzOffset = typeof window !== "undefined" && typeof window.Date !== "undefined" ? (/* @__PURE__ */ new Date()).getTimezoneOffset() * -1 : 0;
      const resp = await fetch(
        `/api/heatmap?period=${encodeURIComponent(period)}&tz_offset_min=${tzOffset}`
      );
      if (!resp.ok) return;
      const data = await resp.json();
      renderActivityHeatmap(data);
    } catch {
    } finally {
      loadHeatmapInFlight = false;
    }
  }
  async function loadData(force = false) {
    if (loadDataInFlight && !force) return;
    loadDataInFlight = true;
    const isSubsequentFetch = rawData.value !== null;
    if (isSubsequentFetch) {
      loadState.value = "refreshing";
      setStatus("header-refresh", "loading", "REFRESHING");
    }
    try {
      const resp = await fetch("/api/data");
      if (!resp.ok) {
        setStatus("global", "error", `Failed to load data: HTTP ${resp.status}`);
        return;
      }
      const d5 = await resp.json();
      if (d5.error) {
        setStatus("global", "error", d5.error);
        return;
      }
      clearStatus("global");
      clearStatus("header-refresh");
      metaText.value = "Updated: " + d5.generated_at + " \xB7 Auto-refresh 30s";
      const isFirstLoad = rawData.value === null;
      rawData.value = d5;
      if (isFirstLoad) {
        selectedRange.value = readURLRange();
        selectedProvider.value = readURLProvider();
        selectedModels.value = readURLModels(d5.all_models);
        const urlProject = new URLSearchParams(window.location.search).get("project");
        if (urlProject) projectSearchQuery.value = urlProject;
      }
      applyFilter();
    } catch (e4) {
      setStatus("global", "error", "Network error loading data");
      clearStatus("header-refresh");
    } finally {
      loadState.value = "idle";
      loadDataInFlight = false;
    }
  }
  var headerMount = document.getElementById("header-mount");
  if (headerMount) {
    R(/* @__PURE__ */ u2(Header, { onDataReload: loadData, onThemeToggle: toggleTheme }), headerMount);
  }
  var filterBarMount = document.getElementById("filter-bar-mount");
  if (filterBarMount) {
    R(/* @__PURE__ */ u2(FilterBar, { onFilterChange: applyFilter, onURLUpdate: updateURL }), filterBarMount);
  }
  var footerEl = document.querySelector("footer");
  if (footerEl && footerEl.parentElement) {
    R(/* @__PURE__ */ u2(Footer, {}), footerEl.parentElement, footerEl);
  }
  var globalStatusMount = document.getElementById("inline-status-global");
  if (globalStatusMount) {
    R(/* @__PURE__ */ u2(InlineStatus, { placement: "global" }), globalStatusMount);
  }
  window.addEventListener("popstate", () => {
    selectedBucket.value = readBucket();
    if (rawData.value) applyFilter();
  });
  loadData();
  setInterval(loadData, 3e4);
  loadUsageWindows();
  loadAgentStatus();
  loadCommunitySignal();
  setInterval(() => {
    loadUsageWindows();
    loadAgentStatus();
    loadCommunitySignal();
  }, 6e4);
  loadHeatmap("all");
  setInterval(() => loadHeatmap("all"), 3e4);
  loadBillingBlocks();
  setInterval(loadBillingBlocks, 3e4);
  loadContextWindow();
  setInterval(loadContextWindow, 3e4);
  loadCostReconciliation();
  setInterval(loadCostReconciliation, 3e4);
})();
/*! Bundled license information:

@tanstack/table-core/build/lib/index.mjs:
  (**
     * table-core
     *
     * Copyright (c) TanStack
     *
     * This source code is licensed under the MIT license found in the
     * LICENSE.md file in the root directory of this source tree.
     *
     * @license MIT
     *)
*/
