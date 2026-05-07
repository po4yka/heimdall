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
    var a4, h5, p5, v4, y5, _4, g4, m5 = t4 && t4.__k || w, b4 = l5.length;
    for (f5 = T(u5, l5, m5, f5, b4), a4 = 0; a4 < b4; a4++) null != (p5 = u5.__k[a4]) && (h5 = -1 != p5.__i && m5[p5.__i] || d, p5.__i = a4, _4 = q(n3, p5, h5, i4, r4, o4, e4, f5, c4, s4), v4 = p5.__e, p5.ref && h5.ref != p5.ref && (h5.ref && J(h5.ref, null, p5), s4.push(p5.ref, p5.__c || v4, p5)), null == y5 && null != v4 && (y5 = v4), (g4 = !!(4 & p5.__u)) || h5.__k === p5.__k ? (f5 = j(p5, f5, n3, g4), g4 && h5.__e && (h5.__e = null)) : "function" == typeof p5.type && void 0 !== _4 ? f5 = _4 : v4 && (f5 = v4.nextSibling), p5.__u &= -7);
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
    var a4, h5, p5, v4, y5, d5, _4, k4, x4, M, $3, I2, P2, A4, H2, T5 = u5.type;
    if (void 0 !== u5.constructor) return null;
    128 & t4.__u && (c4 = !!(32 & t4.__u), o4 = [f5 = u5.__e = t4.__e]), (a4 = l.__b) && a4(u5);
    n: if ("function" == typeof T5) try {
      if (k4 = u5.props, x4 = T5.prototype && T5.prototype.render, M = (a4 = T5.contextType) && i4[a4.__c], $3 = a4 ? M ? M.props.value : a4.__ : i4, t4.__c ? _4 = (h5 = u5.__c = t4.__c).__ = h5.__E : (x4 ? u5.__c = h5 = new T5(k4, $3) : (u5.__c = h5 = new C(k4, $3), h5.constructor = T5, h5.render = Q), M && M.sub(h5), h5.state || (h5.state = {}), h5.__n = i4, p5 = h5.__d = true, h5.__h = [], h5._sb = []), x4 && null == h5.__s && (h5.__s = h5.state), x4 && null != T5.getDerivedStateFromProps && (h5.__s == h5.state && (h5.__s = m({}, h5.__s)), m(h5.__s, T5.getDerivedStateFromProps(k4, h5.__s))), v4 = h5.props, y5 = h5.state, h5.__v = u5, p5) x4 && null == T5.getDerivedStateFromProps && null != h5.componentWillMount && h5.componentWillMount(), x4 && null != h5.componentDidMount && h5.__h.push(h5.componentDidMount);
      else {
        if (x4 && null == T5.getDerivedStateFromProps && k4 !== v4 && null != h5.componentWillReceiveProps && h5.componentWillReceiveProps(k4, $3), u5.__v == t4.__v || !h5.__e && null != h5.shouldComponentUpdate && false === h5.shouldComponentUpdate(k4, h5.__s, $3)) {
          u5.__v != t4.__v && (h5.props = k4, h5.state = h5.__s, h5.__d = false), u5.__e = t4.__e, u5.__k = t4.__k, u5.__k.some(function(n4) {
            n4 && (n4.__ = u5);
          }), w.push.apply(h5.__h, h5._sb), h5._sb = [], h5.__h.length && e4.push(h5);
          break n;
        }
        null != h5.componentWillUpdate && h5.componentWillUpdate(k4, h5.__s, $3), x4 && null != h5.componentDidUpdate && h5.__h.push(function() {
          h5.componentDidUpdate(v4, y5, d5);
        });
      }
      if (h5.context = $3, h5.props = k4, h5.__P = n3, h5.__e = false, I2 = l.__r, P2 = 0, x4) h5.state = h5.__s, h5.__d = false, I2 && I2(u5), a4 = h5.render(h5.props, h5.state, h5.context), w.push.apply(h5.__h, h5._sb), h5._sb = [];
      else do {
        h5.__d = false, I2 && I2(u5), a4 = h5.render(h5.props, h5.state, h5.context), h5.state = h5.__s;
      } while (h5.__d && ++P2 < 25);
      h5.state = h5.__s, null != h5.getChildContext && (i4 = m(m({}, i4), h5.getChildContext())), x4 && !p5 && null != h5.getSnapshotBeforeUpdate && (d5 = h5.getSnapshotBeforeUpdate(v4, y5)), A4 = null != a4 && a4.type === S && null == a4.key ? E(a4.props.children) : a4, f5 = L(n3, g(A4) ? A4 : [A4], u5, t4, i4, r4, o4, e4, f5, c4, s4), h5.base = u5.__e, u5.__u &= -161, h5.__h.length && e4.push(h5), _4 && (h5.__E = h5.__ = null);
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
    var a4, h5, p5, v4, y5, w5, _4, m5 = i4.props || d, k4 = t4.props, x4 = t4.type;
    if ("svg" == x4 ? o4 = "http://www.w3.org/2000/svg" : "math" == x4 ? o4 = "http://www.w3.org/1998/Math/MathML" : o4 || (o4 = "http://www.w3.org/1999/xhtml"), null != e4) {
      for (a4 = 0; a4 < e4.length; a4++) if ((y5 = e4[a4]) && "setAttribute" in y5 == !!x4 && (x4 ? y5.localName == x4 : 3 == y5.nodeType)) {
        u5 = y5, e4[a4] = null;
        break;
      }
    }
    if (null == u5) {
      if (null == x4) return document.createTextNode(k4);
      u5 = document.createElementNS(o4, x4, k4.is && k4), c4 && (l.__m && l.__m(t4, e4), c4 = false), e4 = null;
    }
    if (null == x4) m5 === k4 || c4 && u5.data == k4 || (u5.data = k4);
    else {
      if (e4 = e4 && n.call(u5.childNodes), !c4 && null != e4) for (m5 = {}, a4 = 0; a4 < u5.attributes.length; a4++) m5[(y5 = u5.attributes[a4]).name] = y5.value;
      for (a4 in m5) y5 = m5[a4], "dangerouslySetInnerHTML" == a4 ? p5 = y5 : "children" == a4 || a4 in k4 || "value" == a4 && "defaultValue" in k4 || "checked" == a4 && "defaultChecked" in k4 || N(u5, a4, null, y5, o4);
      for (a4 in k4) y5 = k4[a4], "children" == a4 ? v4 = y5 : "dangerouslySetInnerHTML" == a4 ? h5 = y5 : "value" == a4 ? w5 = y5 : "checked" == a4 ? _4 = y5 : c4 && "function" != typeof y5 || m5[a4] === y5 || N(u5, a4, y5, m5[a4], o4);
      if (h5) c4 || p5 && (h5.__html == p5.__html || h5.__html == u5.innerHTML) || (u5.innerHTML = h5.__html), t4.__k = [];
      else if (p5 && (u5.innerHTML = ""), L("template" == t4.type ? u5.content : u5, g(v4) ? v4 : [v4], t4, i4, r4, "foreignObject" == x4 ? "http://www.w3.org/1999/xhtml" : o4, e4, f5, e4 ? e4[0] : i4.__k && $(i4, 0), c4, s4), null != e4) for (a4 = e4.length; a4--; ) b(e4[a4]);
      c4 || (a4 = "value", "progress" == x4 && null == w5 ? u5.removeAttribute("value") : null != w5 && (w5 !== u5[a4] || "progress" == x4 && !w5 || "option" == x4 && w5 != m5[a4]) && N(u5, a4, w5, m5[a4], o4), a4 = "checked", null != _4 && _4 != u5[a4] && N(u5, a4, _4, m5[a4], o4));
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

  // node_modules/preact/hooks/dist/hooks.module.js
  var t2;
  var r2;
  var u2;
  var i2;
  var o2 = 0;
  var f2 = [];
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
  function q2(n3, t4) {
    return o2 = 8, T2(function() {
      return n3;
    }, t4);
  }
  function j2() {
    for (var n3; n3 = f2.shift(); ) {
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
    i4 && (u2 === r2 ? (i4.__h = [], r2.__h = [], i4.__.some(function(n4) {
      n4.__N && (n4.__ = n4.__N), n4.u = n4.__N = void 0;
    })) : (i4.__h.some(z2), i4.__h.some(B2), i4.__h = [], t2 = 0)), u2 = r2;
  }, c2.diffed = function(n3) {
    v2 && v2(n3);
    var t4 = n3.__c;
    t4 && t4.__H && (t4.__H.__h.length && (1 !== f2.push(t4) && i2 === c2.requestAnimationFrame || ((i2 = c2.requestAnimationFrame) || w2)(j2)), t4.__H.__.some(function(n4) {
      n4.u && (n4.__H = n4.u), n4.u = void 0;
    })), u2 = r2 = null;
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
    e3 = ++u3;
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
  var f3;
  var h3 = void 0;
  var s3 = 0;
  var v3 = 0;
  var u3 = 0;
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
    if (f3) f3.push(this);
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
  var m4 = [];
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
  var k3 = "undefined" == typeof requestAnimationFrame ? setTimeout : function(i4) {
    var n3 = function() {
      clearTimeout(r4);
      cancelAnimationFrame(t4);
      i4();
    }, r4 = setTimeout(n3, 35), t4 = requestAnimationFrame(n3);
  };
  var q3 = function(i4) {
    queueMicrotask(function() {
      queueMicrotask(i4);
    });
  };
  function A3() {
    n2(function() {
      var i4;
      while (i4 = m4.shift()) l4.call(i4);
    });
  }
  function T4() {
    if (1 === m4.push(this)) (l.requestAnimationFrame || k3)(A3);
  }
  function x3() {
    n2(function() {
      var i4;
      while (i4 = _3.shift()) l4.call(i4);
    });
  }
  function F() {
    if (1 === _3.push(this)) (l.requestAnimationFrame || q3)(x3);
  }
  function useSignalEffect(i4, n3) {
    var r4 = A2(i4);
    r4.current = i4;
    y2(function() {
      return j3(function() {
        this.N = T4;
        return r4.current();
      }, n3);
    }, []);
  }

  // src/ui/state/store.ts
  var rawData = y3(null);
  var billingBlocksData = y3(null);
  var contextWindowData = y3(null);
  var costReconciliationData = y3(null);
  var versionInfo = y3(null);
  var versionChecking = y3(false);
  var backupSnapshots = y3([]);
  var backupLoadState = y3("idle");
  var webConversations = y3([]);
  var companionHeartbeat = y3(null);
  var archiveImports = y3([]);
  var selectedDate = y3(null);
  var todayData = y3(null);
  var todayLoading = y3(false);
  var SESSIONS_PAGE_PARAM = "sessions_page";
  var SESSIONS_HIDDEN_COLUMNS_PARAM = "sessions_hidden";
  var FILTERS_EXPANDED_PARAM = "filters_expanded";
  var DASHBOARD_TAB_PARAM = "tab";
  var COLLAPSED_SECTIONS_PARAM = "collapsed_sections";
  var TODAY_DATE_PARAM = "today_date";
  var SESSIONS_TABLE_COLUMN_IDS = /* @__PURE__ */ new Set([
    "session",
    "project",
    "provider",
    "last",
    "duration_min",
    "model",
    "turns",
    "input",
    "output",
    "cost",
    "credits",
    "cost_meta",
    "cache_hit_ratio",
    "tokens_per_min"
  ]);
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
    "community-signal": null,
    "snapshot": null,
    "agent-registry": null,
    "layout-save": null,
    "project-registry": null,
    "settings": null
  });
  var registryModalOpen = y3(null);
  var backupModalOpen = y3(false);
  var setupBannerDismissed = y3(false);
  var settingsModalOpen = y3(false);
  var settingsServer = y3(null);
  var settingsDraft = y3(null);
  var settingsInFlight = y3(false);
  var settingsActiveSection = y3("display");
  var commandPaletteOpen = y3(false);
  var selectedProjectUuid = y3(null);
  var projectsRegistry = y3([]);
  var registryByUuid = g2(
    () => new Map(projectsRegistry.value.map((r4) => [r4.project_uuid, r4]))
  );
  var registryBySlug = g2(
    () => new Map(projectsRegistry.value.map((r4) => [r4.slug, r4]))
  );
  function setProjectHash(uuid) {
    const next = uuid ? `${window.location.pathname}${window.location.search}#/project/${encodeURIComponent(uuid)}` : window.location.pathname + window.location.search;
    history.replaceState(null, "", next);
  }
  var editMode = y3(false);
  var SESSIONS_PAGE_SIZE = 25;
  function readSearchParam(name) {
    return new URLSearchParams(window.location.search).get(name);
  }
  function readPositiveIntParam(name) {
    const raw = readSearchParam(name);
    if (!raw) return null;
    const parsed = Number.parseInt(raw, 10);
    return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
  }
  function readRangeFromUrl() {
    const p5 = readSearchParam("range");
    return ["7d", "30d", "90d", "all"].includes(p5) ? p5 : "30d";
  }
  function readDashboardTab() {
    const p5 = readSearchParam(DASHBOARD_TAB_PARAM);
    if (p5 === "backup") {
      if (typeof window !== "undefined" && !/^#\/backup\b/.test(window.location.hash)) {
        history.replaceState(null, "", window.location.pathname + window.location.search + "#/backup");
      }
      return "overview";
    }
    return ["overview", "activity", "breakdowns", "tables", "projects", "today", "agents"].includes(p5) ? p5 : "overview";
  }
  function readProviderFromUrl() {
    const p5 = readSearchParam("provider");
    return ["claude", "codex", "both"].includes(p5) ? p5 : "both";
  }
  function readModelsFromUrl(allModels) {
    const param = readSearchParam("models");
    if (!param) return new Set(allModels);
    const fromUrl = new Set(param.split(",").map((s4) => s4.trim()).filter(Boolean));
    return new Set(allModels.filter((model) => fromUrl.has(model)));
  }
  function readSessionsTablePagination() {
    return {
      pageIndex: Math.max((readPositiveIntParam(SESSIONS_PAGE_PARAM) ?? 1) - 1, 0),
      pageSize: SESSIONS_PAGE_SIZE
    };
  }
  function readSessionsTableColumnVisibility() {
    const hiddenColumns = readSearchParam(SESSIONS_HIDDEN_COLUMNS_PARAM);
    if (!hiddenColumns) return {};
    const visibility = /* @__PURE__ */ Object.create(null);
    for (const columnId of hiddenColumns.split(",").map((value) => value.trim()).filter(Boolean)) {
      if (SESSIONS_TABLE_COLUMN_IDS.has(columnId)) {
        visibility[columnId] = false;
      }
    }
    return visibility;
  }
  function isDefaultModelSelection(allModels) {
    if (selectedModels.value.size !== allModels.length) return false;
    return allModels.every((model) => selectedModels.value.has(model));
  }
  var loadState = y3("refreshing");
  function readVersionMetric() {
    const p5 = new URLSearchParams(window.location.search).get("version_metric");
    return ["cost", "calls", "tokens"].includes(p5) ? p5 : "cost";
  }
  var versionDonutMetric = y3(readVersionMetric());
  function readHeatmapMetric() {
    const p5 = readSearchParam("hm_metric");
    return p5 === "calls" ? "calls" : "cost";
  }
  var heatmapMetric = y3(readHeatmapMetric());
  function readAgentStatusExpanded() {
    const p5 = readSearchParam("agent_status_expanded");
    return p5 === "1" || p5 === "true";
  }
  function readOfficialSyncExpanded() {
    const p5 = readSearchParam("official_sync_expanded");
    return p5 === "1" || p5 === "true";
  }
  function readCollapsedSections() {
    const p5 = readSearchParam(COLLAPSED_SECTIONS_PARAM);
    if (!p5) return /* @__PURE__ */ new Set();
    return new Set(p5.split(",").map((value) => value.trim()).filter(Boolean));
  }
  function readFiltersExpanded() {
    const p5 = readSearchParam(FILTERS_EXPANDED_PARAM);
    return p5 === "1" || p5 === "true";
  }
  var activeDashboardTab = y3(readDashboardTab());
  function tabToScreen(tab) {
    if (tab === "today") return "activity";
    if (tab === "agents") return "breakdowns";
    return tab;
  }
  function readSidebarCollapsed() {
    try {
      return localStorage.getItem("heimdall:sidebarCollapsed") === "true";
    } catch {
      return false;
    }
  }
  var sidebarCollapsed = y3(readSidebarCollapsed());
  var agent_status_expanded = y3(readAgentStatusExpanded());
  var official_sync_expanded = y3(readOfficialSyncExpanded());
  var mobile_filters_expanded = y3(readFiltersExpanded());
  var collapsedSectionKeys = y3(readCollapsedSections());
  var sessionsTablePagination = y3(readSessionsTablePagination());
  var sessionsTableColumnVisibility = y3(readSessionsTableColumnVisibility());
  function isSectionCollapsed(sectionKey) {
    return collapsedSectionKeys.value.has(sectionKey);
  }
  function setSectionCollapsed(sectionKey, collapsed) {
    const next = new Set(collapsedSectionKeys.value);
    if (collapsed) next.add(sectionKey);
    else next.delete(sectionKey);
    collapsedSectionKeys.value = next;
  }
  function restoreDashboardStateFromUrl(allModels) {
    activeDashboardTab.value = readDashboardTab();
    selectedRange.value = readRangeFromUrl();
    selectedProvider.value = readProviderFromUrl();
    selectedModels.value = readModelsFromUrl(allModels);
    projectSearchQuery.value = readSearchParam("project") ?? "";
    selectedBucket.value = readBucket();
    versionDonutMetric.value = readVersionMetric();
    heatmapMetric.value = readHeatmapMetric();
    agent_status_expanded.value = readAgentStatusExpanded();
    official_sync_expanded.value = readOfficialSyncExpanded();
    mobile_filters_expanded.value = readFiltersExpanded();
    collapsedSectionKeys.value = readCollapsedSections();
    sessionsTablePagination.value = readSessionsTablePagination();
    sessionsTableColumnVisibility.value = readSessionsTableColumnVisibility();
    selectedDate.value = readSearchParam(TODAY_DATE_PARAM);
  }
  function syncDashboardUrl() {
    const allModels = rawData.value?.all_models ?? [];
    const params = new URLSearchParams();
    if (activeDashboardTab.value !== "overview") params.set(DASHBOARD_TAB_PARAM, activeDashboardTab.value);
    if ((activeDashboardTab.value === "today" || activeDashboardTab.value === "activity") && selectedDate.value) {
      params.set(TODAY_DATE_PARAM, selectedDate.value);
    }
    if (selectedRange.value !== "30d") params.set("range", selectedRange.value);
    if (selectedProvider.value !== "both") params.set("provider", selectedProvider.value);
    if (!isDefaultModelSelection(allModels)) {
      params.set("models", Array.from(selectedModels.value).join(","));
    }
    if (projectSearchQuery.value) params.set("project", projectSearchQuery.value);
    if (versionDonutMetric.value !== "cost") params.set("version_metric", versionDonutMetric.value);
    if (heatmapMetric.value !== "cost") params.set("hm_metric", heatmapMetric.value);
    if (selectedBucket.value !== "day") params.set("bucket", selectedBucket.value);
    if (agent_status_expanded.value) params.set("agent_status_expanded", "1");
    if (official_sync_expanded.value) params.set("official_sync_expanded", "1");
    if (mobile_filters_expanded.value) params.set(FILTERS_EXPANDED_PARAM, "1");
    const collapsedSections = Array.from(collapsedSectionKeys.value).sort();
    if (collapsedSections.length) {
      params.set(COLLAPSED_SECTIONS_PARAM, collapsedSections.join(","));
    }
    const pageNumber = sessionsTablePagination.value.pageIndex + 1;
    if (pageNumber > 1) params.set(SESSIONS_PAGE_PARAM, String(pageNumber));
    const hiddenColumns = Object.entries(sessionsTableColumnVisibility.value).filter(([, isVisible]) => isVisible === false).map(([columnId]) => columnId).sort();
    if (hiddenColumns.length) {
      params.set(SESSIONS_HIDDEN_COLUMNS_PARAM, hiddenColumns.join(","));
    }
    const search = params.toString();
    const nextUrl = search ? `${window.location.pathname}?${search}` : window.location.pathname;
    history.replaceState(null, "", nextUrl);
  }

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
    return "$" + c4.toLocaleString("en-US", { minimumFractionDigits: 4, maximumFractionDigits: 4 });
  }
  function fmtCostBig(c4) {
    return "$" + c4.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }
  function fmtCostCompact(c4) {
    const abs = Math.abs(c4);
    if (abs >= 1e9) return "$" + (c4 / 1e9).toFixed(2) + "B";
    if (abs >= 1e6) return "$" + (c4 / 1e6).toFixed(2) + "M";
    if (abs >= 1e3) return "$" + (c4 / 1e3).toFixed(1) + "K";
    if (abs >= 1) return "$" + c4.toFixed(2);
    return "$" + c4.toFixed(4);
  }
  function fmtTzOffset(minutes) {
    if (minutes === 0) return "UTC";
    const sign = minutes < 0 ? "-" : "+";
    const abs = Math.abs(minutes);
    const hh = Math.floor(abs / 60);
    const mm = abs % 60;
    return `UTC${sign}${String(hh).padStart(2, "0")}:${String(mm).padStart(2, "0")}`;
  }
  function fmtResetTime(minutes) {
    if (minutes == null || minutes <= 0) return "now";
    if (minutes >= 1440) return Math.floor(minutes / 1440) + "d " + Math.floor(minutes % 1440 / 60) + "h";
    if (minutes >= 60) return Math.floor(minutes / 60) + "h " + minutes % 60 + "m";
    return minutes + "m";
  }
  function fmtRelativeTime(iso) {
    if (!iso) return "never";
    const ts = Date.parse(iso);
    if (Number.isNaN(ts)) return iso;
    const diffMs = Date.now() - ts;
    if (diffMs <= 0) return "just now";
    const minutes = Math.floor(diffMs / 6e4);
    if (minutes < 1) return "just now";
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  }
  function anyHasCredits(rows2) {
    return rows2.some((r4) => r4.credits != null);
  }
  function fmtCredits(n3) {
    if (n3 == null) return "\u2014";
    return n3.toFixed(2);
  }
  function esc(s4) {
    return s4.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }
  function fmtLabel(s4) {
    if (!s4 || s4 === "not_available" || s4 === "unknown") return "\u2014";
    return s4.split(/[_\s-]+/).filter(Boolean).map((w5) => w5.charAt(0).toUpperCase() + w5.slice(1).toLowerCase()).join(" ");
  }
  function truncateMid(s4, max2, tailChars = 8) {
    const codepoints = Array.from(s4);
    if (codepoints.length <= max2) return s4;
    const safeTail = Math.min(tailChars, Math.max(0, max2 - 2));
    const head = Math.max(0, max2 - safeTail - 1);
    return codepoints.slice(0, head).join("") + "\u2026" + codepoints.slice(-safeTail).join("");
  }

  // node_modules/preact/jsx-runtime/dist/jsxRuntime.module.js
  var f4 = 0;
  function u4(e4, t4, n3, o4, i4, u5) {
    t4 || (t4 = {});
    var a4, c4, p5 = t4;
    if ("ref" in p5) for (c4 in p5 = {}, t4) "ref" == c4 ? a4 = t4[c4] : p5[c4] = t4[c4];
    var l5 = { type: e4, props: p5, key: n3, ref: a4, __k: null, __: null, __b: 0, __e: null, __c: null, constructor: void 0, __v: --f4, __i: -1, __u: 0, __source: i4, __self: u5 };
    if ("function" == typeof e4 && (a4 = e4.defaultProps)) for (c4 in a4) void 0 === p5[c4] && (p5[c4] = a4[c4]);
    return l.vnode && l.vnode(l5), l5;
  }

  // src/ui/components/_primitives/Skeleton.tsx
  function Skeleton({
    width = "100%",
    height = "var(--font-size-body)",
    radius = "var(--radius-1)",
    className,
    ariaLabel
  }) {
    const cls = ["skeleton", className].filter(Boolean).join(" ");
    return /* @__PURE__ */ u4(
      "span",
      {
        class: cls,
        role: ariaLabel ? "status" : void 0,
        "aria-label": ariaLabel,
        "aria-busy": ariaLabel ? "true" : void 0,
        style: { width, height, borderRadius: radius, display: "block" }
      }
    );
  }
  function KpiSkeleton({
    size = "standard",
    withBar = false,
    withSub = true
  }) {
    const valueHeight = size === "hero" ? "var(--font-size-display)" : size === "compact" ? "var(--font-size-value)" : "var(--font-size-display-sm)";
    return /* @__PURE__ */ u4("div", { class: `card stat-card kpi-card kpi-card--${size}`, "aria-busy": "true", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
      /* @__PURE__ */ u4(Skeleton, { width: "40%", height: "var(--font-size-tertiary)" }),
      /* @__PURE__ */ u4(Skeleton, { width: "60%", height: valueHeight }),
      withBar && /* @__PURE__ */ u4(Skeleton, { width: "100%", height: "6px" }),
      withSub && /* @__PURE__ */ u4(Skeleton, { width: "50%", height: "var(--font-size-tertiary)" })
    ] }) });
  }
  function ChartSkeleton({ tall = false, bars = 12 }) {
    const heights = ["55%", "80%", "40%", "95%", "65%", "50%", "75%", "35%", "90%", "60%", "45%", "85%"];
    const height = tall ? "var(--chart-h-lg)" : "var(--chart-h-md)";
    return /* @__PURE__ */ u4(
      "div",
      {
        class: "chart-wrap",
        "aria-busy": "true",
        style: {
          height,
          display: "flex",
          alignItems: "flex-end",
          gap: "var(--space-1)",
          padding: "var(--space-3)"
        },
        children: Array.from({ length: bars }).map((_4, i4) => /* @__PURE__ */ u4(
          "span",
          {
            class: "skeleton",
            style: {
              flex: 1,
              height: heights[i4 % heights.length],
              borderRadius: "var(--radius-1) var(--radius-1) 0 0",
              display: "block"
            }
          },
          i4
        ))
      }
    );
  }
  function TableSkeleton({ rows: rows2 = 6, columns: columns7 = 4 }) {
    return /* @__PURE__ */ u4("table", { "aria-busy": "true", style: { width: "100%" }, children: [
      /* @__PURE__ */ u4("thead", { children: /* @__PURE__ */ u4("tr", { children: Array.from({ length: columns7 }).map((_4, i4) => /* @__PURE__ */ u4("th", { children: /* @__PURE__ */ u4(Skeleton, { width: "60%", height: "var(--font-size-tertiary)" }) }, i4)) }) }),
      /* @__PURE__ */ u4("tbody", { children: Array.from({ length: rows2 }).map((_4, ri) => /* @__PURE__ */ u4("tr", { children: Array.from({ length: columns7 }).map((_5, ci) => /* @__PURE__ */ u4("td", { children: /* @__PURE__ */ u4(Skeleton, { width: ci === 0 ? "70%" : "50%" }) }, ci)) }, ri)) })
    ] });
  }
  function SkeletonGroup({ children }) {
    return /* @__PURE__ */ u4(
      "div",
      {
        "aria-busy": "true",
        style: { display: "flex", flexDirection: "column", gap: "var(--space-2)" },
        children
      }
    );
  }

  // src/ui/components/BackupPanel.tsx
  function BackupPanel({ onSnapshot, onReload }) {
    const snapshots = backupSnapshots.value;
    const state = backupLoadState.value;
    return /* @__PURE__ */ u4("section", { class: "backup-panel", children: [
      /* @__PURE__ */ u4("header", { class: "backup-panel-header", children: [
        /* @__PURE__ */ u4("h2", { children: "Snapshots" }),
        /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: "primary",
            disabled: state === "loading",
            onClick: async () => {
              setStatus("snapshot", "loading", "snapshotting...");
              try {
                await onSnapshot();
                await onReload();
                setStatus("snapshot", "success", "done", 3e3);
              } catch (err) {
                setStatus("snapshot", "error", `error: ${err instanceof Error ? err.message : String(err)}`);
              }
            },
            children: "Snapshot now"
          }
        )
      ] }),
      state === "error" && /* @__PURE__ */ u4("p", { class: "backup-panel-error", children: "Failed to load snapshots." }),
      state === "loading" && snapshots.length === 0 && /* @__PURE__ */ u4(TableSkeleton, { rows: 5, columns: 4 }),
      snapshots.length === 0 && state === "idle" && /* @__PURE__ */ u4("p", { class: "backup-panel-empty", children: 'No snapshots yet \u2014 click "Snapshot now" to create one.' }),
      snapshots.length > 0 && /* @__PURE__ */ u4("table", { class: "data-table", children: [
        /* @__PURE__ */ u4("thead", { children: /* @__PURE__ */ u4("tr", { children: [
          /* @__PURE__ */ u4("th", { children: "SNAPSHOT" }),
          /* @__PURE__ */ u4("th", { children: "CREATED" }),
          /* @__PURE__ */ u4("th", { children: "FILES" }),
          /* @__PURE__ */ u4("th", { children: "BYTES" })
        ] }) }),
        /* @__PURE__ */ u4("tbody", { children: snapshots.map((s4) => /* @__PURE__ */ u4("tr", { children: [
          /* @__PURE__ */ u4("td", { children: esc(s4.snapshot_id) }),
          /* @__PURE__ */ u4("td", { children: esc(s4.created_at) }),
          /* @__PURE__ */ u4("td", { children: s4.total_files }),
          /* @__PURE__ */ u4("td", { children: s4.total_bytes })
        ] }, s4.snapshot_id)) })
      ] })
    ] });
  }

  // src/ui/components/BackupModal.tsx
  function closeModal() {
    backupModalOpen.value = false;
    if (/^#\/backup\b/.test(window.location.hash)) {
      history.replaceState(null, "", window.location.pathname + window.location.search);
    }
  }
  function BackupModal({ onSnapshot, onReload }) {
    y2(() => {
      const handler = (e4) => {
        if (e4.key === "Escape") closeModal();
      };
      window.addEventListener("keydown", handler);
      return () => window.removeEventListener("keydown", handler);
    }, []);
    y2(() => {
      void onReload();
    }, [onReload]);
    return /* @__PURE__ */ u4("div", { class: "agent-registry-overlay", onClick: closeModal, children: /* @__PURE__ */ u4(
      "div",
      {
        class: "agent-registry-modal",
        onClick: (e4) => e4.stopPropagation(),
        role: "dialog",
        "aria-modal": "true",
        "aria-label": "Backup and snapshots",
        children: [
          /* @__PURE__ */ u4("div", { class: "agent-registry-header", children: [
            /* @__PURE__ */ u4("h2", { class: "agent-registry-title", children: "Backup & snapshots" }),
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "agent-registry-close",
                "aria-label": "Close",
                onClick: closeModal,
                children: "[X]"
              }
            )
          ] }),
          /* @__PURE__ */ u4("div", { style: { padding: "0 20px 20px" }, children: /* @__PURE__ */ u4(BackupPanel, { onSnapshot, onReload }) })
        ]
      }
    ) });
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
    return /* @__PURE__ */ u4("div", { role: entry.kind === "error" ? "alert" : "status", style: baseStyle, children: [
      /* @__PURE__ */ u4("span", { children: content }),
      dismissable && entry.kind !== "loading" && /* @__PURE__ */ u4(
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
          children: "Dismiss"
        }
      )
    ] });
  }

  // src/ui/components/settings/DisplaySection.tsx
  var CURRENCY_OPTIONS = ["USD", "EUR", "GBP", "JPY", "KRW", "CNY"];
  function patchDisplay(patch2) {
    const draft = settingsDraft.value;
    if (!draft) return;
    settingsDraft.value = { ...draft, display: { ...draft.display, ...patch2 } };
  }
  function DisplaySection() {
    const draft = settingsDraft.value;
    if (!draft) return null;
    const { currency, locale, compact } = draft.display;
    return /* @__PURE__ */ u4("div", { class: "settings-section", children: [
      /* @__PURE__ */ u4("div", { class: "settings-row", children: [
        /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-display-currency", children: "Currency" }),
        /* @__PURE__ */ u4(
          "select",
          {
            id: "settings-display-currency",
            class: "settings-input num",
            value: currency ?? "USD",
            onChange: (e4) => patchDisplay({ currency: e4.target.value }),
            children: CURRENCY_OPTIONS.map((c4) => /* @__PURE__ */ u4("option", { value: c4, children: c4 }, c4))
          }
        )
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-row", children: [
        /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-display-locale", children: "Locale" }),
        /* @__PURE__ */ u4(
          "input",
          {
            id: "settings-display-locale",
            type: "text",
            class: "settings-input",
            value: locale ?? "",
            placeholder: "auto",
            onInput: (e4) => {
              const v4 = e4.target.value.trim();
              patchDisplay({ locale: v4.length === 0 ? null : v4 });
            }
          }
        )
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-row settings-row--toggle", children: [
        /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-display-compact", children: "Compact mode" }),
        /* @__PURE__ */ u4(
          "input",
          {
            id: "settings-display-compact",
            type: "checkbox",
            checked: compact ?? false,
            onChange: (e4) => patchDisplay({ compact: e4.target.checked })
          }
        )
      ] })
    ] });
  }

  // src/ui/components/settings/PollingSection.tsx
  var ROWS = [
    { key: "oauth", label: "OAuth (Claude usage windows)", minInterval: 30, hasLookback: false },
    { key: "claude_admin", label: "Claude admin", minInterval: 30, hasLookback: true },
    { key: "openai", label: "OpenAI", minInterval: 30, hasLookback: true },
    { key: "agent_status", label: "Agent status", minInterval: 30, hasLookback: false },
    { key: "aggregator", label: "Aggregator", minInterval: 60, hasLookback: false }
  ];
  var SEVERITY_OPTIONS = [
    "minor",
    "major",
    "critical"
  ];
  var MAX_INTERVAL = 86400;
  var MIN_LOOKBACK = 1;
  var MAX_LOOKBACK = 365;
  function patch(key, p5) {
    const draft = settingsDraft.value;
    if (!draft) return;
    const next = { ...draft[key], ...p5 };
    settingsDraft.value = { ...draft, [key]: next };
  }
  function clampInterval(raw, min2) {
    if (!Number.isFinite(raw)) return min2;
    return Math.max(min2, Math.min(MAX_INTERVAL, Math.round(raw)));
  }
  function clampLookback(raw) {
    if (!Number.isFinite(raw)) return MIN_LOOKBACK;
    return Math.max(MIN_LOOKBACK, Math.min(MAX_LOOKBACK, Math.round(raw)));
  }
  function intervalLabel(seconds) {
    if (seconds >= 3600 && seconds % 3600 === 0) {
      const h5 = seconds / 3600;
      return `polled every ${h5} hour${h5 === 1 ? "" : "s"}`;
    }
    if (seconds >= 60 && seconds % 60 === 0) {
      const m5 = seconds / 60;
      return `polled every ${m5} minute${m5 === 1 ? "" : "s"}`;
    }
    return `polled every ${seconds} seconds`;
  }
  function IntervalStepper({ value, min: min2, onChange, ariaLabel }) {
    return /* @__PURE__ */ u4("div", { class: "settings-stepper", children: [
      /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: "settings-stepper-btn",
          "aria-label": `${ariaLabel} decrease`,
          onClick: () => onChange(clampInterval(value - 30, min2)),
          children: "[-]"
        }
      ),
      /* @__PURE__ */ u4(
        "input",
        {
          type: "number",
          class: "settings-input num settings-input--narrow",
          value,
          min: min2,
          max: MAX_INTERVAL,
          step: 30,
          "aria-label": ariaLabel,
          onInput: (e4) => {
            const raw = Number.parseFloat(e4.target.value);
            onChange(clampInterval(raw, min2));
          }
        }
      ),
      /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: "settings-stepper-btn",
          "aria-label": `${ariaLabel} increase`,
          onClick: () => onChange(clampInterval(value + 30, min2)),
          children: "[+]"
        }
      )
    ] });
  }
  function PollingSection() {
    const draft = settingsDraft.value;
    if (!draft) return null;
    return /* @__PURE__ */ u4("div", { class: "settings-section", children: ROWS.map((row) => {
      const groupAny = draft[row.key];
      const enabled = groupAny.enabled;
      const interval = groupAny.refresh_interval;
      return /* @__PURE__ */ u4("div", { class: "settings-card", children: [
        /* @__PURE__ */ u4("div", { class: "settings-row settings-row--toggle", children: [
          /* @__PURE__ */ u4("label", { class: "settings-label", for: `settings-polling-${row.key}-enabled`, children: row.label }),
          /* @__PURE__ */ u4(
            "input",
            {
              id: `settings-polling-${row.key}-enabled`,
              type: "checkbox",
              checked: enabled,
              onChange: (e4) => patch(row.key, { enabled: e4.target.checked })
            }
          )
        ] }),
        enabled && /* @__PURE__ */ u4(S, { children: [
          /* @__PURE__ */ u4("div", { class: "settings-row", children: [
            /* @__PURE__ */ u4("label", { class: "settings-label", children: "Refresh interval" }),
            /* @__PURE__ */ u4(
              IntervalStepper,
              {
                value: interval,
                min: row.minInterval,
                ariaLabel: `${row.label} refresh interval seconds`,
                onChange: (next) => patch(row.key, { refresh_interval: next })
              }
            )
          ] }),
          /* @__PURE__ */ u4("div", { class: "settings-helper", children: intervalLabel(interval) }),
          row.hasLookback && /* @__PURE__ */ u4("div", { class: "settings-row", children: [
            /* @__PURE__ */ u4("label", { class: "settings-label", for: `settings-polling-${row.key}-lookback`, children: "Lookback days" }),
            /* @__PURE__ */ u4(
              "input",
              {
                id: `settings-polling-${row.key}-lookback`,
                type: "number",
                class: "settings-input num settings-input--narrow",
                value: groupAny.lookback_days,
                min: MIN_LOOKBACK,
                max: MAX_LOOKBACK,
                step: 1,
                onInput: (e4) => {
                  const raw = Number.parseFloat(e4.target.value);
                  patch(row.key, { lookback_days: clampLookback(raw) });
                }
              }
            )
          ] }),
          row.key === "agent_status" && /* @__PURE__ */ u4(S, { children: [
            /* @__PURE__ */ u4("div", { class: "settings-row settings-row--toggle", children: [
              /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-polling-agent-status-claude", children: "Claude provider" }),
              /* @__PURE__ */ u4(
                "input",
                {
                  id: "settings-polling-agent-status-claude",
                  type: "checkbox",
                  checked: draft.agent_status.claude_enabled,
                  onChange: (e4) => patch("agent_status", {
                    claude_enabled: e4.target.checked
                  })
                }
              )
            ] }),
            /* @__PURE__ */ u4("div", { class: "settings-row settings-row--toggle", children: [
              /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-polling-agent-status-openai", children: "OpenAI provider" }),
              /* @__PURE__ */ u4(
                "input",
                {
                  id: "settings-polling-agent-status-openai",
                  type: "checkbox",
                  checked: draft.agent_status.openai_enabled,
                  onChange: (e4) => patch("agent_status", {
                    openai_enabled: e4.target.checked
                  })
                }
              )
            ] }),
            /* @__PURE__ */ u4("div", { class: "settings-row", children: [
              /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-polling-agent-status-severity", children: "Alert min severity" }),
              /* @__PURE__ */ u4(
                "select",
                {
                  id: "settings-polling-agent-status-severity",
                  class: "settings-input",
                  value: draft.agent_status.alert_min_severity,
                  onChange: (e4) => patch("agent_status", {
                    alert_min_severity: e4.target.value
                  }),
                  children: SEVERITY_OPTIONS.map((s4) => /* @__PURE__ */ u4("option", { value: s4, children: s4 }, s4))
                }
              )
            ] })
          ] }),
          row.key === "aggregator" && /* @__PURE__ */ u4("div", { class: "settings-row settings-row--toggle", children: [
            /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-polling-aggregator-spike", children: "Spike webhook" }),
            /* @__PURE__ */ u4(
              "input",
              {
                id: "settings-polling-aggregator-spike",
                type: "checkbox",
                checked: draft.aggregator.spike_webhook,
                onChange: (e4) => patch("aggregator", {
                  spike_webhook: e4.target.checked
                })
              }
            )
          ] })
        ] })
      ] }, row.key);
    }) });
  }

  // src/ui/components/settings/StatuslineBlocksSection.tsx
  function patchStatusline(p5) {
    const draft = settingsDraft.value;
    if (!draft) return;
    settingsDraft.value = { ...draft, statusline: { ...draft.statusline, ...p5 } };
  }
  function patchBlocks(p5) {
    const draft = settingsDraft.value;
    if (!draft) return;
    settingsDraft.value = { ...draft, blocks: { ...draft.blocks, ...p5 } };
  }
  function thresholdHint(value, kind) {
    if (kind === "fraction") {
      if (value < 0) return "must be >= 0";
      if (value > 1) return "must be <= 1.0";
    } else {
      if (value <= 0) return "must be > 0";
    }
    return null;
  }
  function ThresholdInput({ id, label, value, kind, step, onChange }) {
    const hint = thresholdHint(value, kind);
    return /* @__PURE__ */ u4("div", { class: "settings-threshold", children: [
      /* @__PURE__ */ u4("label", { class: "settings-label", for: id, children: label }),
      /* @__PURE__ */ u4(
        "input",
        {
          id,
          type: "number",
          class: "settings-input num",
          value,
          step,
          onInput: (e4) => {
            const raw = Number.parseFloat(e4.target.value);
            if (Number.isFinite(raw)) onChange(raw);
          }
        }
      ),
      hint && /* @__PURE__ */ u4("div", { class: "settings-hint settings-hint--error", children: hint })
    ] });
  }
  function StatuslineBlocksSection() {
    const draft = settingsDraft.value;
    if (!draft) return null;
    const sl = draft.statusline;
    const bl = draft.blocks;
    return /* @__PURE__ */ u4("div", { class: "settings-section", children: [
      /* @__PURE__ */ u4("div", { class: "settings-card", children: [
        /* @__PURE__ */ u4("h3", { class: "settings-subtitle", children: "Statusline thresholds" }),
        /* @__PURE__ */ u4("div", { class: "settings-grid-2x2", children: [
          /* @__PURE__ */ u4(
            ThresholdInput,
            {
              id: "settings-statusline-context-low",
              label: "Context low",
              value: sl.context_low_threshold,
              kind: "fraction",
              step: 0.01,
              onChange: (v4) => patchStatusline({ context_low_threshold: v4 })
            }
          ),
          /* @__PURE__ */ u4(
            ThresholdInput,
            {
              id: "settings-statusline-context-medium",
              label: "Context medium",
              value: sl.context_medium_threshold,
              kind: "fraction",
              step: 0.01,
              onChange: (v4) => patchStatusline({ context_medium_threshold: v4 })
            }
          ),
          /* @__PURE__ */ u4(
            ThresholdInput,
            {
              id: "settings-statusline-burn-normal",
              label: "Burn-rate normal max",
              value: sl.burn_rate_normal_max,
              kind: "positive",
              step: 0.1,
              onChange: (v4) => patchStatusline({ burn_rate_normal_max: v4 })
            }
          ),
          /* @__PURE__ */ u4(
            ThresholdInput,
            {
              id: "settings-statusline-burn-moderate",
              label: "Burn-rate moderate max",
              value: sl.burn_rate_moderate_max,
              kind: "positive",
              step: 0.1,
              onChange: (v4) => patchStatusline({ burn_rate_moderate_max: v4 })
            }
          )
        ] })
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-card", children: [
        /* @__PURE__ */ u4("h3", { class: "settings-subtitle", children: "Blocks" }),
        /* @__PURE__ */ u4("div", { class: "settings-row", children: [
          /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-blocks-token-limit", children: "Token limit" }),
          /* @__PURE__ */ u4("div", { class: "settings-input-group", children: [
            /* @__PURE__ */ u4(
              "input",
              {
                id: "settings-blocks-token-limit",
                type: "number",
                class: "settings-input num",
                value: bl.token_limit ?? "",
                placeholder: "auto",
                min: 0,
                step: 1,
                onInput: (e4) => {
                  const v4 = e4.target.value.trim();
                  if (v4 === "") {
                    patchBlocks({ token_limit: null });
                    return;
                  }
                  const parsed = Number.parseInt(v4, 10);
                  if (Number.isFinite(parsed) && parsed >= 0) {
                    patchBlocks({ token_limit: parsed });
                  }
                }
              }
            ),
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "settings-clear-btn",
                "aria-label": "Clear token limit",
                disabled: bl.token_limit == null,
                onClick: () => patchBlocks({ token_limit: null }),
                children: "[CLEAR]"
              }
            )
          ] })
        ] }),
        /* @__PURE__ */ u4("div", { class: "settings-row", children: [
          /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-blocks-session-length", children: "Session length (hours)" }),
          /* @__PURE__ */ u4("div", { class: "settings-input-group", children: [
            /* @__PURE__ */ u4(
              "input",
              {
                id: "settings-blocks-session-length",
                type: "number",
                class: "settings-input num",
                value: bl.session_length_hours ?? "",
                placeholder: "auto",
                min: 0,
                step: 0.5,
                onInput: (e4) => {
                  const v4 = e4.target.value.trim();
                  if (v4 === "") {
                    patchBlocks({ session_length_hours: null });
                    return;
                  }
                  const parsed = Number.parseFloat(v4);
                  if (Number.isFinite(parsed) && parsed >= 0) {
                    patchBlocks({ session_length_hours: parsed });
                  }
                }
              }
            ),
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "settings-clear-btn",
                "aria-label": "Clear session length",
                disabled: bl.session_length_hours == null,
                onClick: () => patchBlocks({ session_length_hours: null }),
                children: "[CLEAR]"
              }
            )
          ] })
        ] })
      ] })
    ] });
  }

  // src/ui/components/settings/WebhooksSection.tsx
  function patchWebhooks(p5) {
    const draft = settingsDraft.value;
    if (!draft) return;
    settingsDraft.value = {
      ...draft,
      webhooks: { ...draft.webhooks, ...p5 }
    };
  }
  function EventToggle({ id, label, help, checked, onChange, disabled }) {
    return /* @__PURE__ */ u4("div", { class: `settings-card settings-toggle-stack-item${disabled ? " settings-toggle-stack-item--disabled" : ""}`, children: [
      /* @__PURE__ */ u4("div", { class: "settings-row settings-row--toggle", children: [
        /* @__PURE__ */ u4("label", { class: "settings-label", for: id, children: label }),
        /* @__PURE__ */ u4(
          "input",
          {
            id,
            type: "checkbox",
            checked,
            disabled,
            onChange: (e4) => onChange(e4.target.checked)
          }
        )
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-helper", children: help })
    ] });
  }
  function WebhooksSection({ onUrlIntentChange }) {
    const draft = settingsDraft.value;
    if (!draft) return null;
    const { webhooks } = draft;
    const urlPresent = webhooks.url_present;
    const [editing, setEditing] = d2(false);
    const [urlInput, setUrlInput] = d2("");
    function handleSetUrl() {
      setUrlInput("");
      setEditing(true);
    }
    function handleSaveUrl() {
      const trimmed = urlInput.trim();
      if (!trimmed) return;
      onUrlIntentChange({ kind: "set", value: trimmed });
      const current = settingsDraft.value;
      if (current) {
        settingsDraft.value = {
          ...current,
          webhooks: { ...current.webhooks, url_present: true }
        };
      }
      setEditing(false);
      setUrlInput("");
    }
    function handleCancelUrl() {
      setEditing(false);
      setUrlInput("");
    }
    function handleClearUrl() {
      onUrlIntentChange({ kind: "clear" });
      const current = settingsDraft.value;
      if (current) {
        settingsDraft.value = {
          ...current,
          webhooks: { ...current.webhooks, url_present: false }
        };
      }
    }
    const costThreshold = webhooks.cost_threshold;
    const costInputVal = costThreshold !== null && costThreshold !== void 0 ? String(costThreshold) : "";
    function handleCostInput(raw) {
      const trimmed = raw.trim();
      if (trimmed === "") {
        patchWebhooks({ cost_threshold: null });
        return;
      }
      const n3 = parseFloat(trimmed);
      if (Number.isFinite(n3) && n3 > 0) {
        patchWebhooks({ cost_threshold: n3 });
      }
    }
    const filterArr = webhooks.agent_stop_reason_filter ?? [];
    const filterStr = filterArr.join(", ");
    const agentStopEnabled = webhooks.agent_stop_reason;
    function handleFilterInput(raw) {
      const parts = raw.split(",").map((s4) => s4.trim()).filter(Boolean);
      patchWebhooks({ agent_stop_reason_filter: parts.length === 0 ? null : parts });
    }
    return /* @__PURE__ */ u4("div", { class: "settings-section", children: [
      /* @__PURE__ */ u4("div", { class: "settings-card", children: [
        /* @__PURE__ */ u4("div", { class: "settings-row", children: [
          /* @__PURE__ */ u4("label", { class: "settings-label", children: "Webhook URL" }),
          /* @__PURE__ */ u4("div", { class: "settings-input-group", children: [
            /* @__PURE__ */ u4("span", { class: "settings-helper", style: "margin-top:0", children: [
              "URL configured: ",
              urlPresent ? "yes" : "no"
            ] }),
            !editing && /* @__PURE__ */ u4(S, { children: [
              /* @__PURE__ */ u4(
                "button",
                {
                  type: "button",
                  class: "settings-clear-btn",
                  onClick: handleSetUrl,
                  children: "[Set URL]"
                }
              ),
              /* @__PURE__ */ u4(
                "button",
                {
                  type: "button",
                  class: "settings-clear-btn settings-clear-btn--destructive",
                  disabled: !urlPresent,
                  onClick: handleClearUrl,
                  children: "[Clear URL]"
                }
              )
            ] })
          ] })
        ] }),
        editing && /* @__PURE__ */ u4("div", { class: "settings-webhook-url-editor", children: [
          /* @__PURE__ */ u4(
            "input",
            {
              type: "url",
              class: "settings-input",
              placeholder: "https://hooks.example.com/...",
              value: urlInput,
              onInput: (e4) => {
                setUrlInput(e4.target.value);
              },
              onKeyDown: (e4) => {
                if (e4.key === "Enter") handleSaveUrl();
                if (e4.key === "Escape") handleCancelUrl();
              },
              ref: (el) => {
                if (el) el.focus();
              }
            }
          ),
          /* @__PURE__ */ u4("button", { type: "button", class: "settings-btn settings-btn--primary", onClick: handleSaveUrl, children: "[Save]" }),
          /* @__PURE__ */ u4("button", { type: "button", class: "settings-btn", onClick: handleCancelUrl, children: "[Cancel]" })
        ] }),
        /* @__PURE__ */ u4("div", { class: "settings-helper", children: esc("Webhook events POST a JSON body to this URL. Heimdall never sends the configured URL back to the browser; only \u201CURL configured: yes/no\u201D is exposed. Set or clear it from this Mac to update.") })
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-card", children: [
        /* @__PURE__ */ u4("div", { class: "settings-row", children: [
          /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-webhook-cost-threshold", children: "Cost threshold (USD)" }),
          /* @__PURE__ */ u4("div", { class: "settings-input-group", children: [
            /* @__PURE__ */ u4(
              "input",
              {
                id: "settings-webhook-cost-threshold",
                type: "number",
                class: "settings-input num settings-input--narrow",
                value: costInputVal,
                min: "0.01",
                step: "0.01",
                placeholder: "\u2014",
                onInput: (e4) => handleCostInput(e4.target.value)
              }
            ),
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "settings-clear-btn",
                disabled: costThreshold === null || costThreshold === void 0,
                onClick: () => patchWebhooks({ cost_threshold: null }),
                children: "[CLEAR]"
              }
            )
          ] })
        ] }),
        /* @__PURE__ */ u4("div", { class: "settings-helper", children: "Fire a webhook when daily cost crosses this amount." })
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-toggle-stack", children: [
        /* @__PURE__ */ u4(
          EventToggle,
          {
            id: "settings-webhook-session-depleted",
            label: "Session depleted",
            help: "Fire when an active session crosses the rate-limit ceiling.",
            checked: webhooks.session_depleted,
            onChange: (v4) => patchWebhooks({ session_depleted: v4 })
          }
        ),
        /* @__PURE__ */ u4(
          EventToggle,
          {
            id: "settings-webhook-agent-status",
            label: "Agent status",
            help: "Fire on Claude / OpenAI status transitions.",
            checked: webhooks.agent_status,
            onChange: (v4) => patchWebhooks({ agent_status: v4 })
          }
        ),
        /* @__PURE__ */ u4(
          EventToggle,
          {
            id: "settings-webhook-spike",
            label: "Community spike",
            help: "Fire when a third-party aggregator reports a Claude or OpenAI outage spike (only when official status is below Major).",
            checked: webhooks.spike_webhook,
            onChange: (v4) => patchWebhooks({ spike_webhook: v4 })
          }
        ),
        /* @__PURE__ */ u4(
          EventToggle,
          {
            id: "settings-webhook-cap-changes",
            label: "Subscription cap changes",
            help: "Fire when estimated Claude Code or Codex caps shift materially.",
            checked: webhooks.cap_changes,
            onChange: (v4) => patchWebhooks({ cap_changes: v4 })
          }
        ),
        /* @__PURE__ */ u4(
          EventToggle,
          {
            id: "settings-webhook-agent-stop-reason",
            label: "Subagent stop reason",
            help: "Fire when a subagent exits with a stop_reason on the allowlist below.",
            checked: webhooks.agent_stop_reason,
            onChange: (v4) => patchWebhooks({ agent_stop_reason: v4 })
          }
        )
      ] }),
      /* @__PURE__ */ u4("div", { class: `settings-card${!agentStopEnabled ? " settings-card--muted" : ""}`, children: [
        /* @__PURE__ */ u4("div", { class: "settings-row", children: [
          /* @__PURE__ */ u4("label", { class: "settings-label", for: "settings-webhook-stop-reason-filter", children: "Stop-reason allowlist" }),
          /* @__PURE__ */ u4(
            "input",
            {
              id: "settings-webhook-stop-reason-filter",
              type: "text",
              class: "settings-input",
              value: filterStr,
              placeholder: "max_tokens, refusal",
              disabled: !agentStopEnabled,
              onInput: (e4) => handleFilterInput(e4.target.value)
            }
          )
        ] }),
        /* @__PURE__ */ u4("div", { class: "settings-helper", children: [
          "Comma-separated. Empty = use default (",
          /* @__PURE__ */ u4("span", { class: "settings-mono", children: "max_tokens, refusal" }),
          ")."
        ] })
      ] })
    ] });
  }

  // src/ui/components/settings/ProjectAliasesSection.tsx
  function patchAliases(entries) {
    const draft = settingsDraft.value;
    if (!draft) return;
    settingsDraft.value = {
      ...draft,
      project_aliases: { entries }
    };
  }
  function AutocompleteDropdown({ query, onSelect, onClose }) {
    const ref = A2(null);
    y2(() => {
      function handler(e4) {
        if (ref.current && !ref.current.contains(e4.target)) {
          onClose();
        }
      }
      document.addEventListener("mousedown", handler);
      return () => document.removeEventListener("mousedown", handler);
    }, [onClose]);
    const allSlugs = projectsRegistry.value.map((r4) => r4.slug);
    const lower = query.toLowerCase();
    const matches = allSlugs.filter((s4) => !lower || s4.toLowerCase().includes(lower)).slice(0, 8);
    return /* @__PURE__ */ u4("div", { class: "settings-aliases-autocomplete", ref, children: matches.length === 0 ? /* @__PURE__ */ u4("div", { class: "settings-aliases-autocomplete-empty", children: "No matching projects" }) : matches.map((slug) => /* @__PURE__ */ u4(
      "button",
      {
        type: "button",
        class: "settings-aliases-autocomplete-item",
        onMouseDown: (e4) => {
          e4.preventDefault();
          onSelect(slug);
        },
        children: esc(slug)
      },
      slug
    )) });
  }
  function AliasRow({ entry, index, isFocusTarget, isDuplicateSlug, onChange, onDelete }) {
    const slugRef = A2(null);
    const [showAc, setShowAc] = d2(false);
    y2(() => {
      if (isFocusTarget && slugRef.current) {
        slugRef.current.focus();
        slugRef.current.scrollIntoView({ block: "nearest" });
      }
    }, [isFocusTarget]);
    const isEmpty = !entry.slug.trim() || !entry.display_name.trim();
    const showHint = isEmpty || isDuplicateSlug;
    function handleSlugChange(val) {
      onChange(index, { ...entry, slug: val });
    }
    function handleDisplayNameChange(val) {
      onChange(index, { ...entry, display_name: val });
    }
    function handleAcSelect(slug) {
      onChange(index, { ...entry, slug });
      setShowAc(false);
    }
    return /* @__PURE__ */ u4("div", { class: "settings-aliases-row-wrapper", children: [
      /* @__PURE__ */ u4("div", { class: "settings-aliases-row", children: [
        /* @__PURE__ */ u4("div", { class: "settings-aliases-cell settings-aliases-cell--slug", children: [
          /* @__PURE__ */ u4("div", { class: "settings-aliases-slug-wrap", children: [
            /* @__PURE__ */ u4(
              "input",
              {
                ref: slugRef,
                type: "text",
                class: "settings-input num settings-aliases-input",
                value: entry.slug,
                placeholder: "project-slug",
                onInput: (e4) => handleSlugChange(e4.target.value),
                onKeyDown: (e4) => {
                  if (e4.key === "Escape") setShowAc(false);
                  if (e4.key === "/" && !entry.slug) setShowAc(true);
                },
                onFocus: () => {
                }
              }
            ),
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "settings-aliases-ac-trigger",
                title: "Pick from recent projects",
                onClick: () => setShowAc((v4) => !v4),
                tabIndex: -1,
                children: "\u21A7"
              }
            )
          ] }),
          showAc && /* @__PURE__ */ u4(
            AutocompleteDropdown,
            {
              query: entry.slug,
              onSelect: handleAcSelect,
              onClose: () => setShowAc(false)
            }
          )
        ] }),
        /* @__PURE__ */ u4("div", { class: "settings-aliases-cell settings-aliases-cell--name", children: /* @__PURE__ */ u4(
          "input",
          {
            type: "text",
            class: "settings-input settings-aliases-input",
            value: entry.display_name,
            placeholder: "Friendly name",
            onInput: (e4) => handleDisplayNameChange(e4.target.value)
          }
        ) }),
        /* @__PURE__ */ u4("div", { class: "settings-aliases-cell settings-aliases-cell--action", children: /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: "settings-aliases-delete-btn",
            "aria-label": `Delete alias for ${esc(entry.slug)}`,
            onClick: () => onDelete(index),
            children: "[X]"
          }
        ) })
      ] }),
      showHint && /* @__PURE__ */ u4("div", { class: "settings-aliases-validation-hint", children: isDuplicateSlug ? "Duplicate slug \u2014 only the first will save." : "Both fields are required." })
    ] });
  }
  function ProjectAliasesSection() {
    const draft = settingsDraft.value;
    if (!draft) return null;
    const entries = draft.project_aliases.entries;
    const [filter, setFilter] = d2("");
    const [focusIndex, setFocusIndex] = d2(null);
    const slugCounts = /* @__PURE__ */ new Map();
    for (const e4 of entries) {
      const s4 = e4.slug.trim().toLowerCase();
      if (s4) slugCounts.set(s4, (slugCounts.get(s4) ?? 0) + 1);
    }
    const firstOccurrence = /* @__PURE__ */ new Map();
    for (let i4 = 0; i4 < entries.length; i4++) {
      const s4 = entries[i4].slug.trim().toLowerCase();
      if (s4 && !firstOccurrence.has(s4)) firstOccurrence.set(s4, i4);
    }
    const hasEmptySlugRow = entries.some((e4) => e4.slug.trim() === "");
    function handleAdd() {
      if (hasEmptySlugRow) return;
      const newEntries = [...entries, { slug: "", display_name: "" }];
      patchAliases(newEntries);
      setFocusIndex(newEntries.length - 1);
    }
    function handleChange(index, updated) {
      const newEntries = entries.map((e4, i4) => i4 === index ? updated : e4);
      patchAliases(newEntries);
    }
    function handleDelete(index) {
      patchAliases(entries.filter((_4, i4) => i4 !== index));
      setFocusIndex(null);
    }
    const lowerFilter = filter.toLowerCase();
    const visibleIndices = entries.map((e4, i4) => ({ e: e4, i: i4 })).filter(
      ({ e: e4 }) => !lowerFilter || e4.slug.toLowerCase().includes(lowerFilter) || e4.display_name.toLowerCase().includes(lowerFilter)
    );
    const count2 = entries.length;
    return /* @__PURE__ */ u4("div", { class: "settings-section", children: [
      /* @__PURE__ */ u4("div", { class: "settings-aliases-header", children: [
        /* @__PURE__ */ u4(
          "input",
          {
            type: "text",
            class: "settings-input settings-aliases-filter",
            placeholder: "Filter aliases...",
            value: filter,
            onInput: (e4) => setFilter(e4.target.value)
          }
        ),
        /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: "settings-btn settings-btn--primary",
            disabled: hasEmptySlugRow,
            onClick: handleAdd,
            title: hasEmptySlugRow ? "Fill in the empty row first" : void 0,
            children: "[+ Add alias]"
          }
        )
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-aliases-table", children: [
        /* @__PURE__ */ u4("div", { class: "settings-aliases-thead", children: [
          /* @__PURE__ */ u4("div", { class: "settings-aliases-th settings-aliases-th--slug", children: "SLUG" }),
          /* @__PURE__ */ u4("div", { class: "settings-aliases-th settings-aliases-th--name", children: "DISPLAY NAME" }),
          /* @__PURE__ */ u4("div", { class: "settings-aliases-th settings-aliases-th--action" })
        ] }),
        entries.length === 0 ? /* @__PURE__ */ u4("div", { class: "settings-aliases-empty", children: "No project aliases configured yet. Click [+ Add alias] to create one." }) : visibleIndices.length === 0 ? /* @__PURE__ */ u4("div", { class: "settings-aliases-empty", children: [
          "No aliases match \u201C",
          esc(filter),
          "\u201D."
        ] }) : visibleIndices.map(({ e: e4, i: i4 }) => {
          const slugKey = e4.slug.trim().toLowerCase();
          const isDup = slugKey !== "" && (slugCounts.get(slugKey) ?? 0) > 1 && firstOccurrence.get(slugKey) !== i4;
          return /* @__PURE__ */ u4(
            AliasRow,
            {
              entry: e4,
              index: i4,
              isFocusTarget: focusIndex === i4,
              isDuplicateSlug: isDup,
              onChange: handleChange,
              onDelete: handleDelete
            },
            i4
          );
        })
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-aliases-footer", children: [
        /* @__PURE__ */ u4("span", { class: "settings-hint", children: "Aliases override the slug shown in the dashboard. The raw slug is always preserved in storage." }),
        /* @__PURE__ */ u4("span", { class: "settings-aliases-counter", children: [
          count2,
          " ",
          count2 === 1 ? "alias" : "aliases"
        ] })
      ] })
    ] });
  }

  // src/ui/components/settings/PricingSection.tsx
  function patchOverrides(overrides) {
    const draft = settingsDraft.value;
    if (!draft) return;
    settingsDraft.value = {
      ...draft,
      pricing: { overrides }
    };
  }
  function fmtRate(v4) {
    if (v4 === null) return "?";
    return `$${v4.toFixed(2)}`;
  }
  function ModelPicker({ models, existingModels, onSelect, onCustom, onClose }) {
    const ref = A2(null);
    const inputRef = A2(null);
    const [query, setQuery] = d2("");
    y2(() => {
      if (inputRef.current) inputRef.current.focus();
    }, []);
    y2(() => {
      function handler(e4) {
        if (ref.current && !ref.current.contains(e4.target)) {
          onClose();
        }
      }
      document.addEventListener("mousedown", handler);
      return () => document.removeEventListener("mousedown", handler);
    }, [onClose]);
    y2(() => {
      function handler(e4) {
        if (e4.key === "Escape") onClose();
      }
      document.addEventListener("keydown", handler);
      return () => document.removeEventListener("keydown", handler);
    }, [onClose]);
    const lower = query.toLowerCase();
    const filtered = models.filter((m5) => !lower || m5.model.toLowerCase().includes(lower) || m5.family.toLowerCase().includes(lower)).slice(0, 50);
    const grouped = /* @__PURE__ */ new Map();
    for (const m5 of filtered) {
      const group = grouped.get(m5.family) ?? [];
      group.push(m5);
      grouped.set(m5.family, group);
    }
    const typedNotInList = query.trim().length > 0 && !models.some((m5) => m5.model.toLowerCase() === query.trim().toLowerCase());
    return /* @__PURE__ */ u4("div", { class: "settings-pricing-picker", ref, children: [
      /* @__PURE__ */ u4(
        "input",
        {
          ref: inputRef,
          type: "text",
          class: "settings-input settings-pricing-picker-search",
          placeholder: "Search models...",
          value: query,
          onInput: (e4) => setQuery(e4.target.value)
        }
      ),
      /* @__PURE__ */ u4("div", { class: "settings-pricing-picker-list", children: [
        filtered.length === 0 && !typedNotInList && /* @__PURE__ */ u4("div", { class: "settings-pricing-picker-empty", children: models.length === 0 ? "No model catalog available. Type a model name below." : "No models match." }),
        Array.from(grouped.entries()).map(([family, rows2]) => /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "settings-pricing-picker-group", children: esc(family) }),
          rows2.map((m5) => {
            const alreadyAdded = existingModels.has(m5.model);
            return /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: `settings-pricing-picker-row${alreadyAdded ? " settings-pricing-picker-row--disabled" : ""}`,
                disabled: alreadyAdded,
                onMouseDown: (e4) => {
                  e4.preventDefault();
                  if (!alreadyAdded) onSelect(m5);
                },
                children: [
                  /* @__PURE__ */ u4("span", { class: "num", children: esc(m5.model) }),
                  /* @__PURE__ */ u4("span", { class: "settings-pricing-picker-rates", children: alreadyAdded ? /* @__PURE__ */ u4("em", { children: "Already overridden" }) : `${fmtRate(m5.default_input)} / ${fmtRate(m5.default_output)}` })
                ]
              },
              m5.model
            );
          })
        ] }, family)),
        typedNotInList && /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: "settings-pricing-picker-row settings-pricing-picker-row--custom",
            onMouseDown: (e4) => {
              e4.preventDefault();
              onCustom(query.trim());
            },
            children: [
              /* @__PURE__ */ u4("span", { class: "num", children: [
                "Use custom: ",
                esc(query.trim())
              ] }),
              /* @__PURE__ */ u4("span", { class: "settings-pricing-picker-rates", children: "rates: 0 / 0" })
            ]
          }
        )
      ] })
    ] });
  }
  function OverrideRow({ entry, index, defaultRates, onChange, onDelete }) {
    function setField(k4, v4) {
      onChange(index, { ...entry, [k4]: v4 });
    }
    function parseNum(raw) {
      const v4 = parseFloat(raw);
      return isFinite(v4) && v4 >= 0 ? v4 : 0;
    }
    function parseOptNum(raw) {
      if (raw.trim() === "") return null;
      const v4 = parseFloat(raw);
      return isFinite(v4) && v4 >= 0 ? v4 : null;
    }
    return /* @__PURE__ */ u4("div", { class: "settings-pricing-row", children: [
      /* @__PURE__ */ u4("div", { class: "settings-pricing-cell settings-pricing-cell--model", children: /* @__PURE__ */ u4("span", { class: "num", children: esc(entry.model) }) }),
      /* @__PURE__ */ u4("div", { class: "settings-pricing-cell settings-pricing-cell--rate", children: [
        /* @__PURE__ */ u4("div", { class: "settings-pricing-input-wrap", children: [
          /* @__PURE__ */ u4(
            "input",
            {
              type: "number",
              class: "settings-input num settings-pricing-num-input",
              value: entry.input,
              min: "0",
              step: "0.01",
              onInput: (e4) => setField("input", parseNum(e4.target.value))
            }
          ),
          /* @__PURE__ */ u4("span", { class: "settings-pricing-suffix", children: "/ 1M" })
        ] }),
        /* @__PURE__ */ u4("small", { class: "settings-pricing-default", children: defaultRates !== null ? `default: ${fmtRate(defaultRates.default_input)}` : "default: ?" })
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-pricing-cell settings-pricing-cell--rate", children: [
        /* @__PURE__ */ u4("div", { class: "settings-pricing-input-wrap", children: [
          /* @__PURE__ */ u4(
            "input",
            {
              type: "number",
              class: "settings-input num settings-pricing-num-input",
              value: entry.output,
              min: "0",
              step: "0.01",
              onInput: (e4) => setField("output", parseNum(e4.target.value))
            }
          ),
          /* @__PURE__ */ u4("span", { class: "settings-pricing-suffix", children: "/ 1M" })
        ] }),
        /* @__PURE__ */ u4("small", { class: "settings-pricing-default", children: defaultRates !== null ? `default: ${fmtRate(defaultRates.default_output)}` : "default: ?" })
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-pricing-cell settings-pricing-cell--rate", children: [
        /* @__PURE__ */ u4("div", { class: "settings-pricing-input-wrap", children: [
          /* @__PURE__ */ u4(
            "input",
            {
              type: "number",
              class: "settings-input num settings-pricing-num-input",
              value: entry.cache_write ?? "",
              placeholder: "\u2014",
              min: "0",
              step: "0.01",
              onInput: (e4) => setField("cache_write", parseOptNum(e4.target.value))
            }
          ),
          entry.cache_write !== null && /* @__PURE__ */ u4(
            "button",
            {
              type: "button",
              class: "settings-pricing-clear-btn",
              onClick: () => setField("cache_write", null),
              children: "[CLEAR]"
            }
          )
        ] }),
        /* @__PURE__ */ u4("small", { class: "settings-pricing-default", children: defaultRates !== null ? `default: ${fmtRate(defaultRates.default_cache_write)}` : "default: ?" })
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-pricing-cell settings-pricing-cell--rate", children: [
        /* @__PURE__ */ u4("div", { class: "settings-pricing-input-wrap", children: [
          /* @__PURE__ */ u4(
            "input",
            {
              type: "number",
              class: "settings-input num settings-pricing-num-input",
              value: entry.cache_read ?? "",
              placeholder: "\u2014",
              min: "0",
              step: "0.01",
              onInput: (e4) => setField("cache_read", parseOptNum(e4.target.value))
            }
          ),
          entry.cache_read !== null && /* @__PURE__ */ u4(
            "button",
            {
              type: "button",
              class: "settings-pricing-clear-btn",
              onClick: () => setField("cache_read", null),
              children: "[CLEAR]"
            }
          )
        ] }),
        /* @__PURE__ */ u4("small", { class: "settings-pricing-default", children: defaultRates !== null ? `default: ${fmtRate(defaultRates.default_cache_read)}` : "default: ?" })
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-pricing-cell settings-pricing-cell--action", children: /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: "settings-aliases-delete-btn",
          "aria-label": `Delete override for ${esc(entry.model)}`,
          onClick: () => onDelete(index),
          children: "[X]"
        }
      ) })
    ] });
  }
  function PricingSection() {
    const draft = settingsDraft.value;
    if (!draft) return null;
    const overrides = draft.pricing.overrides;
    const [filter, setFilter] = d2("");
    const [showPicker, setShowPicker] = d2(false);
    const [pricingModels, setPricingModels] = d2([]);
    const [modelsFetched, setModelsFetched] = d2(false);
    const [defaultsCache, setDefaultsCache] = d2(/* @__PURE__ */ new Map());
    const addBtnRef = A2(null);
    async function ensureModelsFetched() {
      if (modelsFetched) return;
      try {
        const r4 = await fetch("/api/pricing-models");
        if (!r4.ok) throw new Error(`HTTP ${r4.status}`);
        const body = await r4.json();
        setPricingModels(body.models ?? []);
        setDefaultsCache((prev) => {
          const next = new Map(prev);
          for (const m5 of body.models ?? []) next.set(m5.model, m5);
          return next;
        });
      } catch {
        setPricingModels([]);
      }
      setModelsFetched(true);
    }
    function handleAddClick() {
      void ensureModelsFetched().then(() => setShowPicker(true));
      if (modelsFetched) setShowPicker(true);
    }
    function handlePickerSelect(m5) {
      const newEntry = {
        model: m5.model,
        input: m5.default_input,
        output: m5.default_output,
        cache_write: m5.default_cache_write,
        cache_read: m5.default_cache_read
      };
      setDefaultsCache((prev) => new Map(prev).set(m5.model, m5));
      patchOverrides([...overrides, newEntry]);
      setShowPicker(false);
    }
    function handlePickerCustom(name) {
      const newEntry = {
        model: name,
        input: 0,
        output: 0,
        cache_write: null,
        cache_read: null
      };
      patchOverrides([...overrides, newEntry]);
      setShowPicker(false);
    }
    function handleChange(index, updated) {
      patchOverrides(overrides.map((e4, i4) => i4 === index ? updated : e4));
    }
    function handleDelete(index) {
      patchOverrides(overrides.filter((_4, i4) => i4 !== index));
    }
    const lowerFilter = filter.toLowerCase();
    const visibleIndices = overrides.map((e4, i4) => ({ e: e4, i: i4 })).filter(({ e: e4 }) => !lowerFilter || e4.model.toLowerCase().includes(lowerFilter));
    const existingModels = new Set(overrides.map((o4) => o4.model));
    const count2 = overrides.length;
    return /* @__PURE__ */ u4("div", { class: "settings-section", children: [
      /* @__PURE__ */ u4("div", { class: "settings-aliases-header", children: [
        /* @__PURE__ */ u4(
          "input",
          {
            type: "text",
            class: "settings-input settings-aliases-filter",
            placeholder: "Filter overrides...",
            value: filter,
            onInput: (e4) => setFilter(e4.target.value)
          }
        ),
        /* @__PURE__ */ u4("div", { class: "settings-pricing-add-wrap", children: [
          /* @__PURE__ */ u4(
            "button",
            {
              ref: addBtnRef,
              type: "button",
              class: "settings-btn settings-btn--primary",
              onClick: handleAddClick,
              children: "[+ Add override]"
            }
          ),
          showPicker && /* @__PURE__ */ u4(
            ModelPicker,
            {
              models: pricingModels,
              existingModels,
              onSelect: handlePickerSelect,
              onCustom: handlePickerCustom,
              onClose: () => setShowPicker(false)
            }
          )
        ] })
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-pricing-table", children: [
        /* @__PURE__ */ u4("div", { class: "settings-pricing-thead", children: [
          /* @__PURE__ */ u4("div", { class: "settings-pricing-th settings-pricing-th--model", children: "MODEL" }),
          /* @__PURE__ */ u4("div", { class: "settings-pricing-th settings-pricing-th--rate", children: "INPUT" }),
          /* @__PURE__ */ u4("div", { class: "settings-pricing-th settings-pricing-th--rate", children: "OUTPUT" }),
          /* @__PURE__ */ u4("div", { class: "settings-pricing-th settings-pricing-th--rate", children: "CACHE WRITE" }),
          /* @__PURE__ */ u4("div", { class: "settings-pricing-th settings-pricing-th--rate", children: "CACHE READ" }),
          /* @__PURE__ */ u4("div", { class: "settings-pricing-th settings-pricing-th--action" })
        ] }),
        overrides.length === 0 ? /* @__PURE__ */ u4("div", { class: "settings-pricing-empty", children: "No pricing overrides configured yet. Click [+ Add override] to override a model's price." }) : visibleIndices.length === 0 ? /* @__PURE__ */ u4("div", { class: "settings-pricing-empty", children: [
          "No overrides match \u201C",
          esc(filter),
          "\u201D."
        ] }) : visibleIndices.map(({ e: e4, i: i4 }) => /* @__PURE__ */ u4(
          OverrideRow,
          {
            entry: e4,
            index: i4,
            defaultRates: defaultsCache.get(e4.model) ?? null,
            onChange: handleChange,
            onDelete: handleDelete
          },
          i4
        ))
      ] }),
      /* @__PURE__ */ u4("div", { class: "settings-aliases-footer", children: [
        /* @__PURE__ */ u4("span", { class: "settings-hint", children: "Overrides replace Heimdall's built-in pricing for matching model names. Values are USD per million tokens." }),
        /* @__PURE__ */ u4("span", { class: "settings-aliases-counter", children: [
          count2,
          " ",
          count2 === 1 ? "override" : "overrides"
        ] })
      ] })
    ] });
  }

  // src/ui/components/settings/SettingsModal.tsx
  var SECTIONS = [
    { key: "display", label: "Display", description: "Currency, locale, and number compaction.", comingSoon: false },
    { key: "polling", label: "Polling", description: "How often live data sources are refreshed.", comingSoon: false },
    { key: "statusline_blocks", label: "Statusline & blocks", description: "Threshold tuning and block sizing.", comingSoon: false },
    { key: "webhooks", label: "Webhooks", description: "Notify external systems on events.", comingSoon: false },
    { key: "aliases", label: "Project aliases", description: "Map project slugs to display names.", comingSoon: false },
    { key: "pricing", label: "Pricing overrides", description: "Custom rates for specific models.", comingSoon: false }
  ];
  function isDirty(server, draft) {
    if (!server || !draft) return false;
    return JSON.stringify(server) !== JSON.stringify(draft);
  }
  function diffPatch(server, draft, urlIntent) {
    const patch2 = {};
    const keys = [
      "display",
      "oauth",
      "claude_admin",
      "openai",
      "agent_status",
      "aggregator",
      "blocks",
      "statusline",
      "webhooks",
      "project_aliases",
      "pricing"
    ];
    for (const key of keys) {
      if (JSON.stringify(server[key]) !== JSON.stringify(draft[key])) {
        if (key === "webhooks") {
          const { url_present: _drop, ...rest } = draft.webhooks;
          void _drop;
          const webhookPatch = { ...rest };
          if (urlIntent.kind === "set") {
            webhookPatch.url = urlIntent.value;
          } else if (urlIntent.kind === "clear") {
            webhookPatch.url = null;
          }
          patch2.webhooks = webhookPatch;
        } else {
          patch2[key] = draft[key];
        }
      }
    }
    if (!patch2.webhooks && urlIntent.kind !== "unchanged") {
      const { url_present: _drop, ...rest } = draft.webhooks;
      void _drop;
      const webhookPatch = { ...rest };
      if (urlIntent.kind === "set") {
        webhookPatch.url = urlIntent.value;
      } else if (urlIntent.kind === "clear") {
        webhookPatch.url = null;
      }
      patch2.webhooks = webhookPatch;
    }
    return patch2;
  }
  function closeModal2(force = false) {
    const dirty = isDirty(settingsServer.value, settingsDraft.value);
    if (dirty && !force) {
      const ok = window.confirm("Discard unsaved changes?");
      if (!ok) return;
    }
    settingsModalOpen.value = false;
    settingsDraft.value = settingsServer.value;
    if (/^#\/settings\b/.test(window.location.hash)) {
      history.replaceState(null, "", window.location.pathname + window.location.search);
    }
  }
  function SettingsModal({ onDataReload }) {
    const [loadError, setLoadError] = d2(null);
    const [loading, setLoading] = d2(false);
    const [urlIntent, setUrlIntent] = d2({ kind: "unchanged" });
    const fetchSettings = q2(async () => {
      setLoading(true);
      setLoadError(null);
      try {
        const r4 = await fetch("/api/settings");
        if (!r4.ok) throw new Error(`HTTP ${r4.status}`);
        const body = await r4.json();
        settingsServer.value = body;
        settingsDraft.value = body;
        setUrlIntent({ kind: "unchanged" });
      } catch (err) {
        setLoadError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    }, []);
    y2(() => {
      void fetchSettings();
    }, [fetchSettings]);
    y2(() => {
      const handler = (e4) => {
        if (e4.key === "Escape") closeModal2();
      };
      window.addEventListener("keydown", handler);
      return () => window.removeEventListener("keydown", handler);
    }, []);
    async function handleSave() {
      const server = settingsServer.value;
      const draft = settingsDraft.value;
      if (!server || !draft) return;
      const patch2 = diffPatch(server, draft, urlIntent);
      if (Object.keys(patch2).length === 0) return;
      settingsInFlight.value = true;
      try {
        const r4 = await fetch("/api/settings", {
          method: "PATCH",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(patch2)
        });
        if (!r4.ok) {
          let msg = `HTTP ${r4.status}`;
          try {
            const body = await r4.json();
            if (body.error) msg = body.error;
          } catch {
          }
          setStatus("settings", "error", msg, 6e3);
          return;
        }
        const updated = await r4.json();
        settingsServer.value = updated;
        settingsDraft.value = updated;
        setUrlIntent({ kind: "unchanged" });
        setStatus("settings", "success", "SAVED", 2500);
        void onDataReload(true);
      } catch (err) {
        setStatus("settings", "error", err instanceof Error ? err.message : String(err), 6e3);
      } finally {
        settingsInFlight.value = false;
      }
    }
    const dirty = isDirty(settingsServer.value, settingsDraft.value) || urlIntent.kind !== "unchanged";
    const inFlight2 = settingsInFlight.value;
    const activeKey = settingsActiveSection.value;
    const activeMeta = SECTIONS.find((s4) => s4.key === activeKey) ?? SECTIONS[0];
    function renderSection2() {
      if (loading) {
        return /* @__PURE__ */ u4(SkeletonGroup, { children: [
          /* @__PURE__ */ u4(Skeleton, { width: "40%", height: "var(--font-size-display-sm)" }),
          Array.from({ length: 4 }).map((_4, i4) => /* @__PURE__ */ u4(SkeletonGroup, { children: [
            /* @__PURE__ */ u4(Skeleton, { width: "30%", height: "var(--font-size-tertiary)" }),
            /* @__PURE__ */ u4(Skeleton, { width: "100%", height: "32px", radius: "var(--radius-1)" })
          ] }, i4))
        ] });
      }
      if (loadError) {
        return /* @__PURE__ */ u4("div", { class: "settings-error-panel", children: [
          /* @__PURE__ */ u4("div", { children: [
            "[ERROR: ",
            loadError,
            "]"
          ] }),
          /* @__PURE__ */ u4("button", { type: "button", class: "settings-btn", onClick: () => void fetchSettings(), children: "[Retry]" })
        ] });
      }
      if (!settingsDraft.value) return null;
      switch (activeKey) {
        case "display":
          return /* @__PURE__ */ u4(DisplaySection, {});
        case "polling":
          return /* @__PURE__ */ u4(PollingSection, {});
        case "statusline_blocks":
          return /* @__PURE__ */ u4(StatuslineBlocksSection, {});
        case "webhooks":
          return /* @__PURE__ */ u4(
            WebhooksSection,
            {
              onUrlIntentChange: setUrlIntent
            }
          );
        case "aliases":
          return /* @__PURE__ */ u4(ProjectAliasesSection, {});
        case "pricing":
          return /* @__PURE__ */ u4(PricingSection, {});
        default:
          return /* @__PURE__ */ u4("div", { class: "settings-loading", children: "Coming soon." });
      }
    }
    return /* @__PURE__ */ u4("div", { class: "settings-overlay", onClick: () => closeModal2(), children: /* @__PURE__ */ u4(
      "div",
      {
        class: "settings-modal",
        onClick: (e4) => e4.stopPropagation(),
        role: "dialog",
        "aria-modal": "true",
        "aria-label": "Settings",
        children: [
          /* @__PURE__ */ u4("nav", { class: "settings-rail", "aria-label": "Settings sections", children: [
            /* @__PURE__ */ u4("h2", { class: "settings-rail-title", children: "Settings" }),
            /* @__PURE__ */ u4("ul", { class: "settings-rail-list", children: SECTIONS.map((s4) => {
              const isActive = s4.key === activeKey && !s4.comingSoon;
              return /* @__PURE__ */ u4("li", { children: /* @__PURE__ */ u4(
                "button",
                {
                  type: "button",
                  class: `settings-rail-item${isActive ? " settings-rail-item--active" : ""}`,
                  disabled: s4.comingSoon,
                  "aria-current": isActive ? "page" : void 0,
                  onClick: () => {
                    if (!s4.comingSoon) settingsActiveSection.value = s4.key;
                  },
                  children: [
                    /* @__PURE__ */ u4("span", { children: s4.label }),
                    s4.comingSoon && /* @__PURE__ */ u4("span", { class: "settings-rail-suffix", children: "[Coming soon]" })
                  ]
                }
              ) }, s4.key);
            }) })
          ] }),
          /* @__PURE__ */ u4("div", { class: "settings-pane", children: [
            /* @__PURE__ */ u4("header", { class: "settings-pane-header", children: [
              /* @__PURE__ */ u4("div", { children: [
                /* @__PURE__ */ u4("h3", { class: "settings-pane-title", children: activeMeta.label }),
                /* @__PURE__ */ u4("p", { class: "settings-pane-desc", children: activeMeta.description })
              ] }),
              /* @__PURE__ */ u4(
                "button",
                {
                  type: "button",
                  class: "settings-close",
                  "aria-label": "Close",
                  onClick: () => closeModal2(),
                  children: "[X]"
                }
              )
            ] }),
            /* @__PURE__ */ u4("div", { class: "settings-pane-body", children: renderSection2() }),
            /* @__PURE__ */ u4("footer", { class: "settings-pane-footer", children: [
              /* @__PURE__ */ u4(
                "button",
                {
                  type: "button",
                  class: "settings-btn",
                  onClick: () => closeModal2(),
                  children: "[Cancel]"
                }
              ),
              /* @__PURE__ */ u4("div", { class: "settings-footer-status", children: /* @__PURE__ */ u4(InlineStatus, { placement: "settings", inline: true }) }),
              /* @__PURE__ */ u4(
                "button",
                {
                  type: "button",
                  class: "settings-btn settings-btn--primary",
                  disabled: !dirty || inFlight2,
                  onClick: () => void handleSave(),
                  children: inFlight2 ? "[Saving\u2026]" : "[Save]"
                }
              )
            ] })
          ] })
        ]
      }
    ) });
  }

  // src/ui/components/ImportsPanel.tsx
  function ImportsPanel({ onReload }) {
    const imports = archiveImports.value;
    return /* @__PURE__ */ u4("section", { class: "imports-panel", children: [
      /* @__PURE__ */ u4("header", { class: "imports-panel-header", children: [
        /* @__PURE__ */ u4("h2", { children: "Imports" }),
        /* @__PURE__ */ u4("button", { type: "button", onClick: () => void onReload(), children: "Refresh" })
      ] }),
      imports.length === 0 && /* @__PURE__ */ u4("p", { class: "imports-panel-empty", children: [
        "No imports yet. To bring in your web-chat history, request a data export from claude.ai or chatgpt.com (Settings \u2192 Export data) and drop the resulting ZIP onto Heimdall via",
        " ",
        /* @__PURE__ */ u4("code", { children: "heimdall import-export <zip>" }),
        " or run",
        " ",
        /* @__PURE__ */ u4("code", { children: "heimdall import-export --watch ~/Downloads" }),
        "."
      ] }),
      imports.length > 0 && /* @__PURE__ */ u4("table", { class: "data-table", children: [
        /* @__PURE__ */ u4("thead", { children: /* @__PURE__ */ u4("tr", { children: [
          /* @__PURE__ */ u4("th", { children: "VENDOR" }),
          /* @__PURE__ */ u4("th", { children: "IMPORTED" }),
          /* @__PURE__ */ u4("th", { children: "CONVERSATIONS" }),
          /* @__PURE__ */ u4("th", { children: "SCHEMA FINGERPRINT" })
        ] }) }),
        /* @__PURE__ */ u4("tbody", { children: imports.map((m5) => /* @__PURE__ */ u4("tr", { children: [
          /* @__PURE__ */ u4("td", { children: esc(m5.vendor) }),
          /* @__PURE__ */ u4("td", { children: esc(m5.created_at) }),
          /* @__PURE__ */ u4("td", { children: m5.conversation_count }),
          /* @__PURE__ */ u4("td", { children: /* @__PURE__ */ u4("code", { children: esc((m5.schema_fingerprint || "\u2014").slice(0, 12)) }) })
        ] }, m5.import_id)) })
      ] })
    ] });
  }

  // src/ui/components/WebCapturesPanel.tsx
  function vendorCounts(rows2) {
    const out = {};
    for (const r4 of rows2) out[r4.vendor] = (out[r4.vendor] ?? 0) + 1;
    return out;
  }
  function relativeMinutes(iso) {
    const ts = Date.parse(iso);
    if (Number.isNaN(ts)) return iso;
    const mins = Math.max(0, Math.round((Date.now() - ts) / 6e4));
    if (mins < 1) return "just now";
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.round(mins / 60);
    if (hrs < 48) return `${hrs}h ago`;
    return `${Math.round(hrs / 24)}d ago`;
  }
  function WebCapturesPanel({ onReload }) {
    const rows2 = webConversations.value;
    const heartbeat = companionHeartbeat.value;
    const counts = vendorCounts(rows2);
    return /* @__PURE__ */ u4("section", { class: "web-captures-panel", children: [
      /* @__PURE__ */ u4("header", { class: "web-captures-panel-header", children: [
        /* @__PURE__ */ u4("h2", { children: "Web captures" }),
        /* @__PURE__ */ u4("button", { type: "button", onClick: () => void onReload(), children: "Refresh" })
      ] }),
      heartbeat && /* @__PURE__ */ u4("p", { class: "web-captures-panel-heartbeat", children: [
        "Companion: connected",
        heartbeat.vendors_seen.length > 0 && /* @__PURE__ */ u4(S, { children: [
          " (",
          esc(heartbeat.vendors_seen.join(" + ")),
          ")"
        ] }),
        " \xB7 last seen ",
        esc(relativeMinutes(heartbeat.last_seen_at))
      ] }),
      !heartbeat && rows2.length === 0 && /* @__PURE__ */ u4("p", { class: "web-captures-panel-empty", children: [
        "No web captures yet. Install the Heimdall companion browser extension at ",
        /* @__PURE__ */ u4("code", { children: "extensions/heimdall-companion/" }),
        ", pair it with the token from ",
        /* @__PURE__ */ u4("code", { children: "heimdall companion-token show" }),
        ", and your claude.ai + chatgpt.com chats will appear here on the next sync."
      ] }),
      rows2.length > 0 && /* @__PURE__ */ u4(S, { children: [
        /* @__PURE__ */ u4("p", { class: "web-captures-panel-counts", children: Object.entries(counts).map(([vendor, n3]) => `${vendor}: ${n3}`).join(" \xB7 ") }),
        /* @__PURE__ */ u4("table", { class: "data-table", children: [
          /* @__PURE__ */ u4("thead", { children: /* @__PURE__ */ u4("tr", { children: [
            /* @__PURE__ */ u4("th", { children: "VENDOR" }),
            /* @__PURE__ */ u4("th", { children: "CONVERSATION" }),
            /* @__PURE__ */ u4("th", { children: "CAPTURED" }),
            /* @__PURE__ */ u4("th", { children: "HISTORY" })
          ] }) }),
          /* @__PURE__ */ u4("tbody", { children: rows2.map((r4) => /* @__PURE__ */ u4("tr", { children: [
            /* @__PURE__ */ u4("td", { children: esc(r4.vendor) }),
            /* @__PURE__ */ u4("td", { children: /* @__PURE__ */ u4("code", { children: esc(r4.conversation_id) }) }),
            /* @__PURE__ */ u4("td", { children: esc(relativeMinutes(r4.captured_at)) }),
            /* @__PURE__ */ u4("td", { children: r4.history_count })
          ] }, `${r4.vendor}/${r4.conversation_id}`)) })
        ] })
      ] })
    ] });
  }

  // src/ui/lib/agents.ts
  var ENC = encodeURIComponent;
  async function jsonOrThrow(res) {
    if (!res.ok) {
      const text2 = await res.text().catch(() => "");
      throw new Error(`${res.status} ${res.statusText}${text2 ? ": " + text2 : ""}`);
    }
    return res.json();
  }
  async function fetchRegistry(projectId) {
    const res = await fetch(`/api/agents/${ENC(projectId)}/registry`);
    return jsonOrThrow(res);
  }
  async function upsertRole(projectId, rawRole, body) {
    const res = await fetch(`/api/agents/${ENC(projectId)}/registry/${ENC(rawRole)}`, {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body)
    });
    return jsonOrThrow(res);
  }
  async function deleteRole(projectId, rawRole) {
    const res = await fetch(`/api/agents/${ENC(projectId)}/registry/${ENC(rawRole)}`, {
      method: "DELETE"
    });
    return jsonOrThrow(res);
  }
  function unclassifiedDetectedRolesGlobal(telemetry) {
    return telemetry.detected.filter(
      (d5) => d5.raw_role !== "unknown" && !d5.registered
    );
  }

  // src/ui/components/agents/AgentRegistryModal.tsx
  function initialRowState(row) {
    return {
      display_name: row?.display_name ?? "",
      description: row?.description ?? "",
      enabled: row?.enabled ?? true,
      merged_into: row?.merged_into ?? ""
    };
  }
  function AgentRegistryModal({ project, telemetry, onReload }) {
    const [registryRows, setRegistryRows] = d2([]);
    const [loading, setLoading] = d2(true);
    const [rowStates, setRowStates] = d2({});
    const detectedForProject = telemetry.detected.filter((d5) => d5.project === project);
    const allRoles = [
      .../* @__PURE__ */ new Set([
        ...detectedForProject.map((d5) => d5.raw_role),
        ...registryRows.map((r4) => r4.raw_role)
      ])
    ].filter((r4) => r4 !== "unknown");
    const load = q2(async () => {
      setLoading(true);
      try {
        const resp = await fetchRegistry(project);
        setRegistryRows(resp.registry);
        const states = {};
        for (const rawRole of allRoles) {
          const existing = resp.registry.find((r4) => r4.raw_role === rawRole);
          states[rawRole] = initialRowState(existing);
        }
        setRowStates(states);
      } catch {
      } finally {
        setLoading(false);
      }
    }, [project]);
    y2(() => {
      void load();
    }, [load]);
    y2(() => {
      const handler = (e4) => {
        if (e4.key === "Escape") registryModalOpen.value = null;
      };
      window.addEventListener("keydown", handler);
      return () => window.removeEventListener("keydown", handler);
    }, []);
    function updateRow(rawRole, patch2) {
      setRowStates((prev) => ({
        ...prev,
        [rawRole]: { ...prev[rawRole], ...patch2 }
      }));
    }
    async function handleSave(rawRole) {
      const state = rowStates[rawRole];
      if (!state) return;
      const body = {
        display_name: state.display_name || null,
        description: state.description || null,
        enabled: state.enabled,
        merged_into: state.merged_into || null
      };
      if (state.merged_into && state.merged_into === rawRole) {
        setStatus("agent-registry", "error", "cannot merge a role into itself", 2e3);
        return;
      }
      clearStatus("agent-registry");
      try {
        await upsertRole(project, rawRole, body);
        setStatus("agent-registry", "success", "SAVED", 1500);
        await onReload();
        await load();
      } catch (err) {
        setStatus("agent-registry", "error", err instanceof Error ? err.message : String(err), 3e3);
      }
    }
    async function handleDelete(rawRole) {
      if (!window.confirm(`Delete registry entry for "${rawRole}"?`)) return;
      clearStatus("agent-registry");
      try {
        await deleteRole(project, rawRole);
        setStatus("agent-registry", "success", "DELETED", 1500);
        await onReload();
        await load();
      } catch (err) {
        setStatus("agent-registry", "error", err instanceof Error ? err.message : String(err), 3e3);
      }
    }
    const mergeOptions = allRoles.filter((r4) => r4 !== "unknown");
    return /* @__PURE__ */ u4("div", { class: "agent-registry-overlay", onClick: () => registryModalOpen.value = null, children: /* @__PURE__ */ u4(
      "div",
      {
        class: "agent-registry-modal",
        onClick: (e4) => e4.stopPropagation(),
        role: "dialog",
        "aria-modal": "true",
        "aria-label": `Agent registry \u2014 ${project}`,
        children: [
          /* @__PURE__ */ u4("div", { class: "agent-registry-header", children: [
            /* @__PURE__ */ u4("h2", { class: "agent-registry-title", children: [
              "Agent registry \u2014 ",
              esc(project)
            ] }),
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "agent-registry-close",
                "aria-label": "Close",
                onClick: () => registryModalOpen.value = null,
                children: "[X]"
              }
            )
          ] }),
          /* @__PURE__ */ u4("div", { style: { padding: "0 20px 8px" }, children: /* @__PURE__ */ u4(InlineStatus, { placement: "agent-registry", inline: true }) }),
          loading ? /* @__PURE__ */ u4("div", { style: { padding: "var(--space-4)" }, children: /* @__PURE__ */ u4(TableSkeleton, { rows: 4, columns: 3 }) }) : allRoles.length === 0 ? /* @__PURE__ */ u4("div", { class: "empty-state", style: { margin: "20px" }, children: "No agent roles detected for this project" }) : /* @__PURE__ */ u4("div", { class: "agent-registry-table-wrap", children: /* @__PURE__ */ u4("table", { class: "agent-registry-table", children: [
            /* @__PURE__ */ u4("thead", { children: /* @__PURE__ */ u4("tr", { children: [
              /* @__PURE__ */ u4("th", { children: "ROLE" }),
              /* @__PURE__ */ u4("th", { children: "DISPLAY NAME" }),
              /* @__PURE__ */ u4("th", { children: "DESCRIPTION" }),
              /* @__PURE__ */ u4("th", { children: "ENABLED" }),
              /* @__PURE__ */ u4("th", { children: "MERGED INTO" }),
              /* @__PURE__ */ u4("th", { children: "CONFIDENCE" }),
              /* @__PURE__ */ u4("th", { children: "ACTIONS" })
            ] }) }),
            /* @__PURE__ */ u4("tbody", { children: allRoles.map((rawRole) => {
              const state = rowStates[rawRole] ?? initialRowState(void 0);
              const registered = registryRows.find((r4) => r4.raw_role === rawRole);
              const detected = detectedForProject.find((d5) => d5.raw_role === rawRole);
              const confidence = detected?.confidence ?? "unknown";
              const sessionCount = detected?.count ?? 0;
              return /* @__PURE__ */ u4("tr", { class: !registered ? "agent-row-unclassified" : "", children: [
                /* @__PURE__ */ u4("td", { children: /* @__PURE__ */ u4("span", { class: "model-tag", title: rawRole, children: esc(rawRole) }) }),
                /* @__PURE__ */ u4("td", { children: /* @__PURE__ */ u4(
                  "input",
                  {
                    type: "text",
                    class: "agent-registry-input",
                    value: state.display_name,
                    placeholder: "Display name",
                    onInput: (e4) => updateRow(rawRole, { display_name: e4.target.value })
                  }
                ) }),
                /* @__PURE__ */ u4("td", { children: /* @__PURE__ */ u4(
                  "input",
                  {
                    type: "text",
                    class: "agent-registry-input",
                    value: state.description,
                    placeholder: "Description",
                    onInput: (e4) => updateRow(rawRole, { description: e4.target.value })
                  }
                ) }),
                /* @__PURE__ */ u4("td", { style: { textAlign: "center" }, children: /* @__PURE__ */ u4(
                  "input",
                  {
                    type: "checkbox",
                    checked: state.enabled,
                    onChange: (e4) => updateRow(rawRole, { enabled: e4.target.checked })
                  }
                ) }),
                /* @__PURE__ */ u4("td", { children: /* @__PURE__ */ u4(
                  "select",
                  {
                    class: "agent-registry-select",
                    value: state.merged_into,
                    onChange: (e4) => updateRow(rawRole, { merged_into: e4.target.value }),
                    children: [
                      /* @__PURE__ */ u4("option", { value: "", children: "(none)" }),
                      mergeOptions.filter((r4) => r4 !== rawRole).map((r4) => /* @__PURE__ */ u4("option", { value: r4, children: esc(r4) }, r4))
                    ]
                  }
                ) }),
                /* @__PURE__ */ u4("td", { children: [
                  /* @__PURE__ */ u4("span", { class: `confidence-badge ${confidence}`, children: [
                    "[",
                    confidence.toUpperCase(),
                    "]"
                  ] }),
                  sessionCount > 0 && /* @__PURE__ */ u4("span", { style: { color: "var(--text-secondary)", fontFamily: "var(--font-mono)", fontSize: "10px", marginLeft: "6px" }, children: [
                    "(",
                    sessionCount,
                    " ",
                    sessionCount === 1 ? "session" : "sessions",
                    ")"
                  ] })
                ] }),
                /* @__PURE__ */ u4("td", { children: /* @__PURE__ */ u4("div", { style: { display: "flex", gap: "6px" }, children: [
                  /* @__PURE__ */ u4(
                    "button",
                    {
                      type: "button",
                      class: "filter-btn",
                      style: { fontSize: "10px", padding: "2px 8px" },
                      onClick: () => void handleSave(rawRole),
                      children: "Save"
                    }
                  ),
                  registered && /* @__PURE__ */ u4(
                    "button",
                    {
                      type: "button",
                      class: "filter-btn",
                      style: { fontSize: "10px", padding: "2px 8px", color: "var(--accent)", borderColor: "var(--accent)" },
                      onClick: () => void handleDelete(rawRole),
                      children: "Delete"
                    }
                  )
                ] }) })
              ] }, rawRole);
            }) })
          ] }) })
        ]
      }
    ) });
  }

  // src/ui/lib/projects.ts
  async function fetchProjectsRegistry() {
    const r4 = await fetch("/api/projects");
    if (!r4.ok) throw new Error(`HTTP ${r4.status}`);
    const body = await r4.json();
    return body.projects;
  }
  async function patchProject(uuid, body) {
    const r4 = await fetch(`/api/projects/${encodeURIComponent(uuid)}`, {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body)
    });
    if (!r4.ok) throw new Error(`HTTP ${r4.status}`);
    return await r4.json();
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
    const findMaxDepth = function(columns7, depth) {
      if (depth === void 0) {
        depth = 1;
      }
      maxDepth = Math.max(maxDepth, depth);
      columns7.filter((column) => column.getIsVisible()).forEach((column) => {
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
      column.getIndex = memo((position) => [_getVisibleLeafColumns(table, position)], (columns7) => columns7.findIndex((d5) => d5.id === column.id), getMemoOptions(table.options, "debugColumns", "getIndex"));
      column.getIsFirstColumn = (position) => {
        var _columns$;
        const columns7 = _getVisibleLeafColumns(table, position);
        return ((_columns$ = columns7[0]) == null ? void 0 : _columns$.id) === column.id;
      };
      column.getIsLastColumn = (position) => {
        var _columns;
        const columns7 = _getVisibleLeafColumns(table, position);
        return ((_columns = columns7[columns7.length - 1]) == null ? void 0 : _columns.id) === column.id;
      };
    },
    createTable: (table) => {
      table.setColumnOrder = (updater) => table.options.onColumnOrderChange == null ? void 0 : table.options.onColumnOrderChange(updater);
      table.resetColumnOrder = (defaultState) => {
        var _table$initialState$c;
        table.setColumnOrder(defaultState ? [] : (_table$initialState$c = table.initialState.columnOrder) != null ? _table$initialState$c : []);
      };
      table._getOrderColumnsFn = memo(() => [table.getState().columnOrder, table.getState().grouping, table.options.groupedColumnMode], (columnOrder, grouping, groupedColumnMode) => (columns7) => {
        let orderedColumns = [];
        if (!(columnOrder != null && columnOrder.length)) {
          orderedColumns = columns7;
        } else {
          const columnOrderCopy = [...columnOrder];
          const columnsCopy = [...columns7];
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
      column.getStart = memo((position) => [position, _getVisibleLeafColumns(table, position), table.getState().columnSizing], (position, columns7) => columns7.slice(0, column.getIndex(position)).reduce((sum2, column2) => sum2 + column2.getSize(), 0), getMemoOptions(table.options, "debugColumns", "getStart"));
      column.getAfter = memo((position) => [position, _getVisibleLeafColumns(table, position), table.getState().columnSizing], (position, columns7) => columns7.slice(column.getIndex(position) + 1).reduce((sum2, column2) => sum2 + column2.getSize(), 0), getMemoOptions(table.options, "debugColumns", "getAfter"));
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
        return memo(() => [getColumns(), getColumns().filter((d5) => d5.getIsVisible()).map((d5) => d5.id).join("_")], (columns7) => {
          return columns7.filter((d5) => d5.getIsVisible == null ? void 0 : d5.getIsVisible());
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
        const rows2 = ((_table$options$keepPi = table.options.keepPinnedRows) != null ? _table$options$keepPi : true) ? (
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
        return rows2.filter(Boolean).map((d5) => ({
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
    const recurseRows = function(rows2, depth) {
      return rows2.map((row) => {
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
        const rows2 = [];
        for (let i4 = 0; i4 < originalRows.length; i4++) {
          const row = createRow(table, table._getRowId(originalRows[i4], i4, parentRow), originalRows[i4], i4, depth, void 0, parentRow == null ? void 0 : parentRow.id);
          rowModel.flatRows.push(row);
          rowModel.rowsById[row.id] = row;
          rows2.push(row);
          if (table.options.getSubRows) {
            var _row$originalSubRows;
            row.originalSubRows = table.options.getSubRows(originalRows[i4], i4);
            if ((_row$originalSubRows = row.originalSubRows) != null && _row$originalSubRows.length) {
              row.subRows = accessRows(row.originalSubRows, depth + 1, row);
            }
          }
        }
        return rows2;
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
        rows: rows2,
        flatRows,
        rowsById
      } = rowModel;
      const pageStart = pageSize * pageIndex;
      const pageEnd = pageStart + pageSize;
      rows2 = rows2.slice(pageStart, pageEnd);
      let paginatedRowModel;
      if (!table.options.paginateExpandedRows) {
        paginatedRowModel = expandRows({
          rows: rows2,
          flatRows,
          rowsById
        });
      } else {
        paginatedRowModel = {
          rows: rows2,
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
      const sortData = (rows2) => {
        const sortedData = rows2.map((row) => ({
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

  // src/ui/components/tables/DataTable.tsx
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
    return def ?? header.column.id;
  }
  function resolveUpdater(updater, prev) {
    return typeof updater === "function" ? updater(prev) : updater;
  }
  function DataTable({
    columns: columns7,
    data,
    title,
    sectionKey,
    exportFn,
    pageSize,
    defaultSort: defaultSort5,
    enableColumnVisibility,
    costRows,
    paginationState,
    onPaginationChange,
    columnVisibilityState,
    onColumnVisibilityChange
  }) {
    const [sorting, setSorting] = d2(defaultSort5 || []);
    const [localPagination, setLocalPagination] = d2({
      pageIndex: 0,
      pageSize: pageSize || data.length || 100
    });
    const [localColumnVisibility, setLocalColumnVisibility] = d2({});
    const [, rerender] = d2(0);
    const pagination = paginationState ?? localPagination;
    const columnVisibility = columnVisibilityState ?? localColumnVisibility;
    y2(() => {
      if (!pageSize) return;
      const rowsPerPage = pagination.pageSize || pageSize;
      const maxPageIndex = Math.max(Math.ceil(data.length / rowsPerPage) - 1, 0);
      if (pagination.pageIndex <= maxPageIndex) return;
      const nextPagination = { ...pagination, pageIndex: maxPageIndex };
      if (paginationState) {
        onPaginationChange?.(nextPagination);
      } else {
        setLocalPagination(nextPagination);
      }
    }, [data.length, pageSize, pagination, paginationState, onPaginationChange]);
    const tableRef = A2(null);
    const stateRef = A2({ sorting, pagination, columnVisibility });
    stateRef.current = { sorting, pagination, columnVisibility };
    const handlePaginationChange = (updater) => {
      const nextPagination = resolveUpdater(updater, stateRef.current.pagination);
      if (paginationState) {
        onPaginationChange?.(nextPagination);
      } else {
        setLocalPagination(nextPagination);
      }
      rerender((n3) => n3 + 1);
    };
    const handleColumnVisibilityChange = (updater) => {
      const nextVisibility = resolveUpdater(updater, stateRef.current.columnVisibility);
      if (columnVisibilityState) {
        onColumnVisibilityChange?.(nextVisibility);
      } else {
        setLocalColumnVisibility(nextVisibility);
      }
      rerender((n3) => n3 + 1);
    };
    if (!tableRef.current) {
      const tableState = {
        sorting,
        pagination,
        columnVisibility,
        columnPinning: { left: [], right: [] }
      };
      tableRef.current = createTable({
        columns: columns7,
        data,
        state: tableState,
        onStateChange: () => {
        },
        onSortingChange: (updater) => {
          setSorting((prev) => resolveUpdater(updater, prev));
          rerender((n3) => n3 + 1);
        },
        onPaginationChange: handlePaginationChange,
        onColumnVisibilityChange: handleColumnVisibilityChange,
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
        ...pageSize ? { getPaginationRowModel: getPaginationRowModel() } : {},
        renderFallbackValue: ""
      });
    }
    tableRef.current.setOptions((prev) => ({
      ...prev,
      columns: columns7,
      data,
      state: { ...tableRef.current.getState(), sorting, pagination, columnVisibility }
    }));
    const table = tableRef.current;
    const headerGroups = table.getHeaderGroups();
    const rows2 = table.getRowModel().rows;
    const headingId = title ? `table-heading-${title.toLowerCase().replace(/[^a-z0-9]+/g, "-")}` : void 0;
    const sectionContentId = sectionKey ? `section-content-${sectionKey}` : void 0;
    const collapsed = sectionKey ? isSectionCollapsed(sectionKey) : false;
    const handleToggleCollapse = () => {
      if (!sectionKey) return;
      setSectionCollapsed(sectionKey, !collapsed);
      syncDashboardUrl();
      rerender((n3) => n3 + 1);
    };
    return /* @__PURE__ */ u4("div", { class: "table-card", children: [
      (title || exportFn) && /* @__PURE__ */ u4("div", { class: "section-header", children: [
        title && /* @__PURE__ */ u4("h2", { id: headingId, class: "section-title", style: { margin: 0 }, children: title }),
        /* @__PURE__ */ u4("div", { class: "section-actions", children: [
          sectionKey && /* @__PURE__ */ u4(
            "button",
            {
              class: "section-toggle",
              type: "button",
              "aria-expanded": !collapsed,
              "aria-controls": sectionContentId,
              onClick: handleToggleCollapse,
              children: collapsed ? "Show" : "Hide"
            }
          ),
          exportFn && /* @__PURE__ */ u4("button", { class: "export-btn", type: "button", onClick: exportFn, title: "Export to CSV", children: "\u2913 CSV" })
        ] })
      ] }),
      /* @__PURE__ */ u4("div", { id: sectionContentId, style: collapsed ? { display: "none" } : void 0, children: [
        enableColumnVisibility && /* @__PURE__ */ u4("div", { class: "column-toggle", children: table.getAllLeafColumns().map((column) => {
          const colLabel = typeof column.columnDef.header === "string" ? column.columnDef.header : column.id;
          const inputId = `col-toggle-${column.id}`;
          return /* @__PURE__ */ u4("label", { htmlFor: inputId, children: [
            /* @__PURE__ */ u4(
              "input",
              {
                id: inputId,
                name: inputId,
                type: "checkbox",
                checked: column.getIsVisible(),
                onChange: column.getToggleVisibilityHandler(),
                "aria-label": `Toggle column: ${colLabel}`
              }
            ),
            colLabel
          ] }, column.id);
        }) }),
        /* @__PURE__ */ u4("table", { "aria-labelledby": headingId, children: [
          /* @__PURE__ */ u4("thead", { children: headerGroups.map((headerGroup) => /* @__PURE__ */ u4("tr", { children: headerGroup.headers.map((header) => {
            const canSort = header.column.getCanSort();
            const sorted = header.column.getIsSorted();
            return /* @__PURE__ */ u4(
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
                  canSort && /* @__PURE__ */ u4("span", { class: "sort-icon", children: sorted === "desc" ? " \u25BC" : sorted === "asc" ? " \u25B2" : "" })
                ]
              },
              header.id
            );
          }) }, headerGroup.id)) }),
          /* @__PURE__ */ u4("tbody", { children: rows2.map((row) => /* @__PURE__ */ u4("tr", { class: costRows ? "cost-row" : void 0, children: row.getVisibleCells().map((cell) => /* @__PURE__ */ u4("td", { children: renderCell(cell) }, cell.id)) }, row.id)) })
        ] }),
        pageSize && /* @__PURE__ */ u4("div", { class: "pagination", children: [
          /* @__PURE__ */ u4("span", { children: table.getRowCount() > 0 ? `Showing ${pagination.pageIndex * pagination.pageSize + 1}\u2013${Math.min(
            (pagination.pageIndex + 1) * pagination.pageSize,
            table.getRowCount()
          )} of ${table.getRowCount()}` : "No sessions" }),
          /* @__PURE__ */ u4("div", { style: { display: "flex", gap: "6px" }, children: [
            /* @__PURE__ */ u4(
              "button",
              {
                class: "filter-btn",
                disabled: !table.getCanPreviousPage(),
                onClick: () => table.previousPage(),
                children: "\xAB Prev"
              }
            ),
            /* @__PURE__ */ u4(
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
      ] })
    ] });
  }

  // src/ui/components/projects/PinStar.tsx
  function PinStar({ projectUuid, pinned, onChange, label }) {
    const [busy, setBusy] = d2(false);
    const [optimistic, setOptimistic] = d2(null);
    const current = optimistic ?? pinned;
    const ariaLabel = `${current ? "Unpin" : "Pin"} ${label ?? "project"}`;
    async function toggle2() {
      if (busy) return;
      const next = !current;
      setBusy(true);
      setOptimistic(next);
      try {
        await patchProject(projectUuid, { pinned: next });
        onChange?.();
      } catch (err) {
        setOptimistic(null);
        setStatus(
          "project-registry",
          "error",
          `Pin failed: ${err instanceof Error ? err.message : String(err)}`,
          3e3
        );
      } finally {
        setBusy(false);
      }
    }
    return /* @__PURE__ */ u4(
      "button",
      {
        type: "button",
        class: `pin-star ${current ? "is-pinned" : "is-unpinned"}`,
        "aria-label": ariaLabel,
        "aria-pressed": current,
        title: ariaLabel,
        onClick: toggle2,
        disabled: busy,
        style: busy ? { cursor: "wait" } : void 0,
        children: current ? "\u2605" : "\u2606"
      }
    );
  }

  // src/ui/components/projects/ProjectsRegistry.tsx
  var defaultSort = [
    { id: "pinned", desc: true },
    { id: "last_active", desc: true }
  ];
  async function copyText(text2, label) {
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(text2);
        setStatus("project-registry", "success", `[COPIED ${label}]`, 1500);
      }
    } catch {
      setStatus("project-registry", "error", `[COPY FAILED]`, 2e3);
    }
  }
  function ProjectsRegistry({ onReload }) {
    const [rows2, setRows] = d2(projectsRegistry.value);
    const [loading, setLoading] = d2(rows2.length === 0);
    const [query, setQuery] = d2("");
    const [labelEdits, setLabelEdits] = d2({});
    async function load() {
      setLoading(true);
      try {
        const fresh = await fetchProjectsRegistry();
        projectsRegistry.value = fresh;
        setRows(fresh);
      } catch (err) {
        setStatus(
          "project-registry",
          "error",
          `[ERROR: ${err instanceof Error ? err.message : String(err)}]`,
          4e3
        );
      } finally {
        setLoading(false);
      }
    }
    y2(() => {
      void load();
    }, []);
    useSignalEffect(() => {
      setRows(projectsRegistry.value);
    });
    async function handleLabelSave(row, raw) {
      const trimmed = raw.trim();
      const next = trimmed.length > 0 ? trimmed : null;
      const current = row.custom_label ?? "";
      if ((next ?? "") === current) return;
      clearStatus("project-registry");
      try {
        await patchProject(row.project_uuid, { label: next });
        setStatus("project-registry", "success", "[SAVED]", 1500);
        await load();
        onReload?.();
      } catch (err) {
        setStatus(
          "project-registry",
          "error",
          `[ERROR: ${err instanceof Error ? err.message : String(err)}]`,
          3e3
        );
      }
    }
    async function handleClearLabel(row) {
      if ((row.custom_label ?? null) === null) return;
      clearStatus("project-registry");
      try {
        await patchProject(row.project_uuid, { label: null });
        setLabelEdits((prev) => {
          const next = { ...prev };
          delete next[row.project_uuid];
          return next;
        });
        setStatus("project-registry", "success", "[CLEARED]", 1500);
        await load();
        onReload?.();
      } catch (err) {
        setStatus(
          "project-registry",
          "error",
          `[ERROR: ${err instanceof Error ? err.message : String(err)}]`,
          3e3
        );
      }
    }
    function openProject(row) {
      setProjectHash(row.project_uuid);
      window.dispatchEvent(new HashChangeEvent("hashchange"));
    }
    const filtered = T2(() => {
      const q4 = query.trim().toLowerCase();
      if (!q4) return rows2;
      return rows2.filter((r4) => {
        return r4.slug.toLowerCase().includes(q4) || r4.raw_name.toLowerCase().includes(q4) || (r4.custom_label ?? "").toLowerCase().includes(q4) || r4.project_uuid.toLowerCase().includes(q4) || r4.display_name.toLowerCase().includes(q4);
      });
    }, [rows2, query]);
    const columns7 = T2(
      () => [
        {
          id: "pinned",
          accessorFn: (row) => row.pinned ? 1 : 0,
          header: "Pin",
          sortingFn: (a4, b4) => a4.original.pinned === b4.original.pinned ? 0 : a4.original.pinned ? -1 : 1,
          cell: (info) => {
            const row = info.row.original;
            return /* @__PURE__ */ u4(
              PinStar,
              {
                projectUuid: row.project_uuid,
                pinned: row.pinned,
                label: row.display_name || row.slug,
                onChange: () => {
                  void load();
                  onReload?.();
                }
              }
            );
          }
        },
        {
          id: "label",
          accessorFn: (row) => row.custom_label ?? row.display_name ?? row.slug,
          header: "Label",
          cell: (info) => {
            const row = info.row.original;
            const editValue = labelEdits[row.project_uuid] ?? (row.custom_label ?? "");
            return /* @__PURE__ */ u4(
              "input",
              {
                type: "text",
                class: "agent-registry-input",
                value: editValue,
                placeholder: row.raw_name || row.slug,
                onInput: (e4) => {
                  const v4 = e4.target.value;
                  setLabelEdits((prev) => ({ ...prev, [row.project_uuid]: v4 }));
                },
                onBlur: (e4) => {
                  const v4 = e4.target.value;
                  void handleLabelSave(row, v4);
                },
                onKeyDown: (e4) => {
                  if (e4.key === "Enter") {
                    e4.preventDefault();
                    e4.target.blur();
                  }
                }
              }
            );
          }
        },
        {
          id: "slug",
          accessorKey: "slug",
          header: "Slug",
          cell: (info) => {
            const row = info.row.original;
            return /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "table-action-btn",
                title: "Click to copy slug",
                style: { fontFamily: "var(--font-mono)", color: "var(--text-secondary)" },
                onClick: () => void copyText(row.slug, "SLUG"),
                children: [
                  row.slug,
                  row.is_cowork && /* @__PURE__ */ u4("span", { style: { marginLeft: 6, opacity: 0.6 }, title: "Cowork session", children: "[cowork]" })
                ]
              }
            );
          }
        },
        {
          id: "uuid",
          accessorKey: "project_uuid",
          header: "UUID",
          cell: (info) => {
            const row = info.row.original;
            const link = `#/project/${row.project_uuid}`;
            return /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "table-action-btn",
                title: "Click to copy deep link",
                style: { fontFamily: "var(--font-mono)", color: "var(--text-secondary)", fontSize: "0.85em" },
                onClick: () => void copyText(link, "LINK"),
                children: [
                  row.project_uuid.slice(0, 8),
                  "\u2026"
                ]
              }
            );
          }
        },
        {
          id: "sessions",
          accessorKey: "sessions",
          header: "Sessions",
          cell: (info) => /* @__PURE__ */ u4("span", { class: "num", children: String(Number(info.getValue() ?? 0)) })
        },
        {
          id: "calls",
          accessorKey: "calls",
          header: "Calls",
          cell: (info) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(Number(info.getValue() ?? 0)) })
        },
        {
          id: "cost",
          accessorKey: "cost",
          header: "Est. Cost",
          cell: (info) => /* @__PURE__ */ u4("span", { class: "cost", children: fmtCost(Number(info.getValue() ?? 0)) })
        },
        {
          id: "last_active",
          accessorFn: (row) => row.last_active ?? "",
          header: "Last active",
          cell: (info) => {
            const row = info.row.original;
            return /* @__PURE__ */ u4("span", { style: { color: "var(--text-secondary)" }, children: fmtRelativeTime(row.last_active) });
          }
        },
        {
          id: "actions",
          header: "Actions",
          enableSorting: false,
          cell: (info) => {
            const row = info.row.original;
            return /* @__PURE__ */ u4("div", { style: { display: "flex", gap: "6px" }, children: [
              /* @__PURE__ */ u4(
                "button",
                {
                  type: "button",
                  class: "filter-btn",
                  style: { fontSize: "10px", padding: "2px 8px" },
                  title: "Open in dashboard with this project pre-filtered",
                  onClick: () => openProject(row),
                  children: "[Open]"
                }
              ),
              row.custom_label != null && /* @__PURE__ */ u4(
                "button",
                {
                  type: "button",
                  class: "filter-btn",
                  style: { fontSize: "10px", padding: "2px 8px" },
                  title: "Clear custom label",
                  onClick: () => void handleClearLabel(row),
                  children: "[Clear label]"
                }
              )
            ] });
          }
        }
      ],
      // eslint-disable-next-line react-hooks/exhaustive-deps
      [labelEdits, onReload]
    );
    return /* @__PURE__ */ u4("div", { class: "table-card", children: [
      /* @__PURE__ */ u4("div", { class: "section-header", style: { padding: "20px 20px 12px" }, children: [
        /* @__PURE__ */ u4("h2", { class: "section-title", style: { margin: 0 }, children: "Projects" }),
        /* @__PURE__ */ u4("div", { class: "section-actions", style: { display: "flex", gap: "8px", alignItems: "center" }, children: [
          /* @__PURE__ */ u4(
            "input",
            {
              type: "search",
              placeholder: "Search slug, label, UUID\u2026",
              value: query,
              onInput: (e4) => setQuery(e4.target.value),
              class: "agent-registry-input",
              style: { minWidth: 220 }
            }
          ),
          /* @__PURE__ */ u4(
            "button",
            {
              type: "button",
              class: "filter-btn",
              onClick: () => void load(),
              disabled: loading,
              title: "Refresh registry",
              children: loading ? "[REFRESHING]" : "Refresh"
            }
          ),
          /* @__PURE__ */ u4(InlineStatus, { placement: "project-registry", inline: true })
        ] })
      ] }),
      /* @__PURE__ */ u4("div", { style: { padding: "0 20px 20px" }, children: [
        loading && rows2.length === 0 ? /* @__PURE__ */ u4(TableSkeleton, { rows: 6, columns: 5 }) : /* @__PURE__ */ u4(
          DataTable,
          {
            columns: columns7,
            data: filtered,
            defaultSort
          }
        ),
        !loading && filtered.length === 0 && /* @__PURE__ */ u4("div", { class: "empty-state", style: { marginTop: 12 }, children: query ? "No projects match the search." : "No projects detected yet." })
      ] })
    ] });
  }

  // src/ui/components/Sidebar.tsx
  var NAV_ITEMS = [
    { key: "overview", label: "Overview", abbr: "OV" },
    { key: "today", label: "Today", abbr: "TD" },
    { key: "activity", label: "Activity", abbr: "AC" },
    { key: "agents", label: "Agents", abbr: "AG" },
    { key: "breakdowns", label: "Cost & Models", abbr: "C$" },
    { key: "tables", label: "Sessions", abbr: "SS" },
    { key: "projects", label: "Projects", abbr: "PR" }
  ];
  function Sidebar() {
    const collapsed = sidebarCollapsed.value;
    const activeTab = activeDashboardTab.value;
    y2(() => {
      if (collapsed) {
        document.documentElement.dataset["sidebarCollapsed"] = "";
      } else {
        delete document.documentElement.dataset["sidebarCollapsed"];
      }
    }, [collapsed]);
    const handleNavClick = (tab) => {
      activeDashboardTab.value = tab;
      syncDashboardUrl();
    };
    const toggleCollapsed = () => {
      sidebarCollapsed.value = !collapsed;
      try {
        localStorage.setItem("heimdall:sidebarCollapsed", String(!collapsed));
      } catch {
      }
    };
    return /* @__PURE__ */ u4(
      "nav",
      {
        class: `sidebar${collapsed ? " sidebar--collapsed" : ""}`,
        "aria-label": "Dashboard navigation",
        children: [
          /* @__PURE__ */ u4("ul", { class: "sidebar__nav", role: "list", children: NAV_ITEMS.map((item) => {
            const active = activeTab === item.key;
            return /* @__PURE__ */ u4("li", { children: /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: `sidebar__item${active ? " sidebar__item--active" : ""}`,
                "aria-current": active ? "page" : void 0,
                title: collapsed ? item.label : void 0,
                onClick: () => handleNavClick(item.key),
                children: [
                  /* @__PURE__ */ u4("span", { class: "sidebar__abbr", "aria-hidden": "true", children: item.abbr }),
                  !collapsed && /* @__PURE__ */ u4("span", { class: "sidebar__label", children: item.label })
                ]
              }
            ) }, item.key);
          }) }),
          /* @__PURE__ */ u4("div", { class: "sidebar__footer", children: [
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "sidebar__icon-btn",
                "aria-label": "Settings",
                title: "Settings",
                onClick: () => {
                  settingsModalOpen.value = true;
                },
                children: [
                  /* @__PURE__ */ u4("span", { "aria-hidden": "true", children: "\u2699" }),
                  !collapsed && /* @__PURE__ */ u4("span", { class: "sidebar__label", children: "Settings" })
                ]
              }
            ),
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "sidebar__icon-btn",
                "aria-label": "Backup",
                title: "Backup",
                onClick: () => {
                  backupModalOpen.value = true;
                },
                children: [
                  /* @__PURE__ */ u4("span", { "aria-hidden": "true", children: "\u2299" }),
                  !collapsed && /* @__PURE__ */ u4("span", { class: "sidebar__label", children: "Backup" })
                ]
              }
            ),
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "sidebar__collapse-btn",
                "aria-label": collapsed ? "Expand sidebar" : "Collapse sidebar",
                title: collapsed ? "Expand sidebar" : "Collapse sidebar",
                onClick: toggleCollapsed,
                children: /* @__PURE__ */ u4("span", { "aria-hidden": "true", children: collapsed ? "\xBB" : "\xAB" })
              }
            )
          ] })
        ]
      }
    );
  }

  // src/ui/widgets/apply-layout.ts
  var pendingLayoutApply = y3(null);
  var currentLayoutByScreen = y3({});
  function publishCurrentLayout(screen, layout) {
    currentLayoutByScreen.value = { ...currentLayoutByScreen.value, [screen]: layout };
  }

  // src/ui/widgets/mount-registry.ts
  var callbacks = /* @__PURE__ */ new Map();
  function registerMountCallback(mountId, cb) {
    callbacks.set(mountId, cb);
  }
  function invokeMountCallback(mountId, el) {
    const cb = callbacks.get(mountId);
    if (cb) {
      cb(el);
      delete el.dataset["loading"];
    }
  }

  // src/ui/widgets/registry.ts
  function mount(id) {
    return (el) => {
      el.id = id;
    };
  }
  var WIDGET_CATALOG = [
    // ── Overview tab ─────────────────────────────────────────────────────────
    {
      id: "usage-windows",
      title: "Rate windows",
      description: "Session and weekly rate-limit progress bars",
      category: "kpi",
      screens: ["overview"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 1,
      render: mount("usage-windows"),
      hideWhenEmpty: true
    },
    {
      id: "subscription-quota",
      title: "Subscription quota",
      description: "Provider subscription utilization and history chart",
      category: "kpi",
      screens: ["overview"],
      // Renders six rate-window sub-cards (Session/Weekly/Weekly Sonnet/
      // Weekly Opus/Claude/Codex). Natural content height ≈ 1300 px which at
      // the 132 px GridStack cellHeight is exactly 10 rows.
      defaultSize: { w: 4, h: 10 },
      minW: 2,
      minH: 8,
      render: mount("subscription-quota")
    },
    {
      id: "claude-usage",
      title: "Claude usage",
      description: "Claude API usage details from the credentials file",
      category: "kpi",
      screens: ["overview"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 1,
      render: mount("claude-usage")
    },
    {
      id: "agent-status",
      title: "Agent status",
      description: "Upstream provider health (Claude, OpenAI)",
      category: "system",
      screens: ["overview"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 1,
      render: mount("agent-status")
    },
    {
      id: "estimation-meta",
      title: "Estimation metadata",
      description: "Confidence, billing mode, and pricing version breakdown",
      category: "system",
      screens: ["overview"],
      defaultSize: { w: 4, h: 1 },
      minW: 2,
      minH: 1,
      render: mount("estimation-meta")
    },
    {
      id: "official-sync",
      title: "Official pricing sync",
      description: "Status of official pricing data synchronization",
      category: "system",
      screens: ["overview"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 1,
      render: mount("official-sync"),
      hideWhenEmpty: true
    },
    {
      id: "openai-reconciliation",
      title: "OpenAI reconciliation",
      description: "OpenAI organization usage reconciliation",
      category: "system",
      screens: ["overview"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 1,
      render: mount("openai-reconciliation"),
      hideWhenEmpty: true
    },
    {
      id: "subagent-reconciliation",
      title: "Subagent reconciliation",
      description: "agent_sessions vs turns(is_subagent=1) cost diff",
      category: "system",
      screens: ["overview"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 1,
      render: mount("subagent-reconciliation"),
      hideWhenEmpty: true
    },
    {
      id: "codex-plan-kpi-mount",
      title: "Codex plan",
      description: "Codex plan utilization KPI tile",
      category: "codex",
      screens: ["overview"],
      defaultSize: { w: 1, h: 1 },
      minW: 1,
      minH: 1,
      render: mount("codex-plan-kpi-mount")
    },
    {
      id: "stats-row",
      title: "Summary stats",
      description: "Token counts, cost, cache efficiency, and active-day averages",
      category: "kpi",
      screens: ["overview"],
      defaultSize: { w: 4, h: 1 },
      minW: 2,
      minH: 1,
      render: mount("stats-row")
    },
    // ── Activity tab ──────────────────────────────────────────────────────────
    {
      id: "codex-plan-history-mount",
      title: "Codex plan history",
      description: "30-day stacked bar chart of Codex plan utilization",
      category: "codex",
      screens: ["activity"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: mount("codex-plan-history-mount")
    },
    {
      id: "daily-chart-card",
      title: "Daily token usage",
      description: "Daily or weekly token usage bar chart",
      category: "chart",
      screens: ["activity"],
      defaultSize: { w: 2, h: 3 },
      minW: 1,
      minH: 2,
      render: (el) => {
        el.id = "daily-chart-card";
        el.className = "card bento-2 chart-card";
        if (!el.querySelector("#daily-chart-title")) {
          el.innerHTML = '<h2 id="daily-chart-title">Daily Token Usage</h2><div class="chart-wrap tall"><div id="chart-daily"></div></div>';
        }
      }
    },
    {
      id: "model-chart-card",
      title: "Model distribution",
      description: "Token usage donut chart broken down by model",
      category: "chart",
      screens: ["activity"],
      defaultSize: { w: 1, h: 3 },
      minW: 1,
      minH: 2,
      render: (el) => {
        el.id = "model-chart-card";
        el.className = "card chart-card";
        if (!el.querySelector("#chart-model")) {
          el.innerHTML = '<h2>By Model</h2><div class="chart-wrap model-chart-wrap"><div id="chart-model"></div></div>';
        }
      }
    },
    {
      id: "project-chart-card",
      title: "Top projects",
      description: "Horizontal bar chart of top projects by cost",
      category: "chart",
      screens: ["activity"],
      defaultSize: { w: 1, h: 3 },
      minW: 1,
      minH: 2,
      render: (el) => {
        el.id = "project-chart-card";
        el.className = "card chart-card";
        if (!el.querySelector("#chart-project")) {
          el.innerHTML = '<h2>Top Projects</h2><div class="chart-wrap"><div id="chart-project"></div></div>';
        }
      }
    },
    {
      id: "hourly-chart",
      title: "Activity by hour",
      description: "Token usage broken down by hour of day",
      category: "chart",
      screens: ["activity"],
      defaultSize: { w: 2, h: 3 },
      minW: 1,
      minH: 2,
      render: (el) => {
        el.id = "hourly-chart";
        el.className = "card card-flat bento-2";
      }
    },
    {
      id: "activity-heatmap",
      title: "Activity heatmap",
      description: "7\xD724 heatmap of token usage or cost",
      category: "heatmap",
      screens: ["activity"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "activity-heatmap";
        el.className = "card card-flat bento-full table-card";
      }
    },
    // ── Breakdowns tab ────────────────────────────────────────────────────────
    {
      id: "subagent-summary",
      title: "Subagent summary",
      description: "Breakdown of subagent vs orchestrator turns and costs",
      category: "agent",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 1,
      render: (el) => {
        el.id = "subagent-summary";
        el.className = "card card-flat bento-full table-card";
      }
    },
    {
      id: "agent-setup-banner",
      title: "Agent setup banner",
      description: "Setup guidance for agent telemetry",
      category: "agent",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 1 },
      minW: 2,
      minH: 1,
      render: mount("agent-setup-banner"),
      hideWhenEmpty: true
    },
    {
      id: "agent-kpis-row",
      title: "Agent KPIs",
      description: "Key metrics for agent telemetry (sessions, cost, tokens)",
      category: "agent",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 1 },
      minW: 2,
      minH: 1,
      render: (el) => {
        el.id = "agent-kpis-row";
        el.style.display = "none";
        el.style.gridTemplateColumns = "repeat(3,1fr)";
        el.style.gap = "16px";
      }
    },
    {
      id: "agent-timeline",
      title: "Agent timeline",
      description: "Cost timeline broken down by agent role",
      category: "agent",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "agent-timeline";
        el.className = "card card-flat bento-full table-card";
      }
    },
    {
      id: "agent-distribution",
      title: "Agent distribution",
      description: "Breakdown of sessions and cost by agent role",
      category: "agent",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "agent-distribution";
        el.className = "card card-flat bento-full table-card";
      }
    },
    {
      id: "agent-top-sessions",
      title: "Top agent sessions",
      description: "Highest-cost agent sessions",
      category: "agent",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "agent-top-sessions";
        el.className = "card card-flat bento-full table-card";
      }
    },
    {
      id: "agent-spawn-batches",
      title: "Agent spawn batches",
      description: "Batches of agent spawns grouped by session",
      category: "agent",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "agent-spawn-batches";
        el.className = "card card-flat bento-full table-card";
      }
    },
    {
      id: "agent-tool-spectrum",
      title: "Agent tool spectrum",
      description: "Tool usage breakdown across agent roles",
      category: "agent",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "agent-tool-spectrum";
        el.className = "card card-flat bento-full table-card";
      }
    },
    {
      id: "entrypoint-breakdown",
      title: "Entrypoint breakdown",
      description: "Usage broken down by CLI entrypoint",
      category: "table",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "entrypoint-breakdown";
        el.className = "card card-flat bento-full table-card";
      }
    },
    {
      id: "service-tiers",
      title: "Service tiers",
      description: "Usage and cost split by service tier",
      category: "table",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "service-tiers";
        el.className = "card card-flat bento-full table-card";
      },
      hideWhenEmpty: true
    },
    {
      id: "tool-summary",
      title: "Tool usage",
      description: "Tool invocation counts with cost attribution",
      category: "table",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "tool-summary";
        el.className = "card card-flat bento-full table-card";
      }
    },
    {
      id: "mcp-summary",
      title: "MCP server usage",
      description: "MCP server invocation counts with cost attribution",
      category: "table",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "mcp-summary";
        el.className = "card card-flat bento-full table-card";
      }
    },
    {
      id: "branch-summary",
      title: "Git branch summary",
      description: "Usage broken down by git branch",
      category: "table",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "branch-summary";
        el.className = "card card-flat bento-full table-card";
      },
      hideWhenEmpty: true
    },
    {
      id: "version-summary",
      title: "CLI versions",
      description: "Usage breakdown by Claude CLI version with donut chart",
      category: "table",
      screens: ["breakdowns"],
      defaultSize: { w: 2, h: 3 },
      minW: 1,
      minH: 2,
      render: (el) => {
        el.id = "version-summary";
        el.className = "card card-flat bento-2";
      },
      hideWhenEmpty: true
    },
    {
      id: "cost-reconciliation",
      title: "Cost reconciliation",
      description: "Hook-measured vs estimated cost comparison",
      category: "system",
      screens: ["breakdowns"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 1,
      render: mount("cost-reconciliation")
    },
    // ── Tables tab ────────────────────────────────────────────────────────────
    {
      id: "model-cost-mount",
      title: "Cost by model",
      description: "Per-model cost table with cache breakdown columns",
      category: "table",
      screens: ["tables"],
      defaultSize: { w: 4, h: 4 },
      minW: 2,
      minH: 2,
      render: mount("model-cost-mount")
    },
    {
      id: "sessions-mount",
      title: "Sessions",
      description: "All sessions with sorting, pagination, and CSV export",
      category: "table",
      screens: ["tables"],
      defaultSize: { w: 4, h: 5 },
      minW: 2,
      minH: 3,
      render: mount("sessions-mount")
    },
    {
      id: "project-cost-mount",
      title: "Cost by project",
      description: "Per-project cost table with CSV export",
      category: "table",
      screens: ["tables"],
      defaultSize: { w: 4, h: 4 },
      minW: 2,
      minH: 2,
      render: mount("project-cost-mount")
    },
    // ── Today tab ─────────────────────────────────────────────────────────────
    {
      id: "today-date-picker-mount",
      title: "Date picker",
      description: "Select a specific date to view",
      category: "today",
      screens: ["activity"],
      defaultSize: { w: 4, h: 1 },
      minW: 2,
      minH: 1,
      render: (el) => {
        el.id = "today-date-picker-mount";
        invokeMountCallback("today-date-picker-mount", el);
      }
    },
    {
      id: "today-kpis-mount",
      title: "Today KPIs",
      description: "Key metrics for the selected day",
      category: "today",
      screens: ["activity"],
      defaultSize: { w: 4, h: 1 },
      minW: 2,
      minH: 1,
      render: (el) => {
        el.id = "today-kpis-mount";
        el.style.gridTemplateColumns = "repeat(auto-fit,minmax(180px,1fr))";
        el.style.gap = "16px";
      }
    },
    {
      id: "today-hour-timeline-mount",
      title: "Hour timeline",
      description: "Token usage timeline for each hour of the selected day",
      category: "today",
      screens: ["activity"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "today-hour-timeline-mount";
        el.className = "card card-flat bento-full";
      }
    },
    {
      id: "today-hour-heatstrip-mount",
      title: "Hour heatstrip",
      description: "Single-row heat strip showing hourly intensity",
      category: "today",
      screens: ["activity"],
      defaultSize: { w: 4, h: 2 },
      minW: 2,
      minH: 1,
      render: (el) => {
        el.id = "today-hour-heatstrip-mount";
        el.className = "card card-flat bento-full";
      }
    },
    {
      id: "today-days-hours-30-mount",
      title: "30-day heat grid",
      description: "30 days \xD7 24 hours usage grid",
      category: "today",
      screens: ["activity"],
      defaultSize: { w: 4, h: 4 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "today-days-hours-30-mount";
        el.className = "card card-flat bento-full";
      }
    },
    {
      id: "today-days-hours-7-mount",
      title: "7-day heat grid",
      description: "7 days \xD7 24 hours usage grid",
      category: "today",
      screens: ["activity"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "today-days-hours-7-mount";
        el.className = "card card-flat bento-full";
      }
    },
    {
      id: "today-weekday-hour-mount",
      title: "Weekday \xD7 hour pattern",
      description: "7\xD724 behavioral heatmap over a 90-day window",
      category: "today",
      screens: ["activity"],
      defaultSize: { w: 4, h: 3 },
      minW: 2,
      minH: 2,
      render: (el) => {
        el.id = "today-weekday-hour-mount";
        el.className = "card card-flat bento-full";
      }
    },
    // ── Projects tab ──────────────────────────────────────────────────────────
    {
      id: "projects-registry",
      title: "Projects",
      description: "Searchable project registry with pinning, custom labels, and deep links",
      category: "table",
      screens: ["projects"],
      defaultSize: { w: 4, h: 12 },
      minW: 2,
      minH: 4,
      render: (el) => {
        el.id = "projects-registry";
        invokeMountCallback("projects-registry", el);
      }
    }
  ];
  function widgetById(id) {
    return WIDGET_CATALOG.find((w5) => w5.id === id);
  }
  function widgetsForScreen(screen) {
    return WIDGET_CATALOG.filter((w5) => w5.screens.includes(screen));
  }

  // src/ui/widgets/default-layouts.ts
  function stack(defs) {
    let y5 = 0;
    const result = [];
    for (const d5 of defs) {
      const w5 = d5.w ?? 4;
      const x4 = d5.x ?? 0;
      const p5 = { i: d5.id, x: x4, y: y5, w: w5, h: d5.h };
      if (d5.minW !== void 0) p5.minW = d5.minW;
      if (d5.minH !== void 0) p5.minH = d5.minH;
      result.push(p5);
      y5 += d5.h;
    }
    return result;
  }
  var OVERVIEW_WIDGETS = stack([
    { id: "usage-windows", h: 2 },
    { id: "subscription-quota", h: 10 },
    { id: "claude-usage", h: 2 },
    { id: "agent-status", h: 2 },
    { id: "estimation-meta", h: 1 },
    { id: "official-sync", h: 2 },
    { id: "openai-reconciliation", h: 2 },
    { id: "subagent-reconciliation", h: 2 },
    { id: "codex-plan-kpi-mount", h: 1 },
    { id: "stats-row", h: 1 }
  ]);
  function makeActivityWidgets() {
    const widgets = [
      // Today block — drilldown for the selected date.
      { i: "today-date-picker-mount", x: 0, y: 0, w: 4, h: 1 },
      { i: "today-kpis-mount", x: 0, y: 1, w: 4, h: 1 },
      { i: "today-hour-timeline-mount", x: 0, y: 2, w: 4, h: 2 },
      { i: "today-hour-heatstrip-mount", x: 0, y: 4, w: 4, h: 1 },
      { i: "today-days-hours-30-mount", x: 0, y: 5, w: 4, h: 4 },
      { i: "today-days-hours-7-mount", x: 0, y: 9, w: 4, h: 2 },
      { i: "today-weekday-hour-mount", x: 0, y: 11, w: 4, h: 2 },
      // Range block — applies the dashboard filter strip.
      // Codex plan history — full width
      { i: "codex-plan-history-mount", x: 0, y: 13, w: 4, h: 3 },
      // Charts row: daily (2 wide) | model (1) | project (1)
      { i: "daily-chart-card", x: 0, y: 16, w: 2, h: 3, minW: 1, minH: 2 },
      { i: "model-chart-card", x: 2, y: 16, w: 1, h: 3, minW: 1, minH: 2 },
      { i: "project-chart-card", x: 3, y: 16, w: 1, h: 3, minW: 1, minH: 2 },
      // Hourly chart (2 wide) then activity heatmap full width
      { i: "hourly-chart", x: 0, y: 19, w: 2, h: 2, minW: 1, minH: 2 },
      { i: "activity-heatmap", x: 0, y: 21, w: 4, h: 2, minW: 2, minH: 2 }
    ];
    return widgets;
  }
  var BREAKDOWNS_WIDGETS = stack([
    { id: "subagent-summary", h: 2 },
    { id: "agent-setup-banner", h: 1 },
    { id: "agent-kpis-row", h: 1 },
    { id: "agent-timeline", h: 3 },
    { id: "agent-distribution", h: 3 },
    { id: "agent-top-sessions", h: 3 },
    { id: "agent-spawn-batches", h: 3 },
    { id: "agent-tool-spectrum", h: 3 },
    { id: "entrypoint-breakdown", h: 3 },
    { id: "service-tiers", h: 3 },
    { id: "tool-summary", h: 3 },
    { id: "mcp-summary", h: 3 },
    { id: "branch-summary", h: 3 },
    { id: "version-summary", h: 3, w: 2 },
    { id: "cost-reconciliation", h: 2 }
  ]);
  var TABLES_WIDGETS = stack([
    { id: "model-cost-mount", h: 4 },
    { id: "sessions-mount", h: 5 },
    { id: "project-cost-mount", h: 4 }
  ]);
  var PROJECTS_WIDGETS = stack([
    { id: "projects-registry", h: 8 }
  ]);
  var DEFAULT_LAYOUTS = {
    overview: { widgets: OVERVIEW_WIDGETS, hidden: [] },
    activity: { widgets: makeActivityWidgets(), hidden: [] },
    breakdowns: { widgets: BREAKDOWNS_WIDGETS, hidden: [] },
    tables: { widgets: TABLES_WIDGETS, hidden: [] },
    projects: { widgets: PROJECTS_WIDGETS, hidden: [] }
  };

  // src/ui/lib/saved-views.ts
  var STORAGE_KEY_PREFIX = "heimdall.saved-views.";
  var ACTIVE_KEY_PREFIX = "heimdall.active-view.";
  var TRIAGE_WIDGET_IDS = {
    overview: ["usage-windows", "subscription-quota", "agent-status", "claude-usage", "stats-row"],
    activity: ["today-date-picker-mount", "today-kpis-mount", "today-hour-heatstrip-mount"],
    breakdowns: ["subagent-summary", "cost-reconciliation"],
    tables: ["sessions-mount"],
    projects: ["projects-registry"]
  };
  function compactLayout(screen) {
    const widgets = widgetsForScreen(screen);
    let y5 = 0;
    const placed = widgets.map((def) => {
      const w5 = def.minW ?? def.defaultSize.w;
      const h5 = def.minH ?? def.defaultSize.h;
      const item = { i: def.id, x: 0, y: y5, w: w5, h: h5 };
      if (def.minW !== void 0) item.minW = def.minW;
      if (def.minH !== void 0) item.minH = def.minH;
      y5 += h5;
      return item;
    });
    return { widgets: placed, hidden: [] };
  }
  function triageLayout(screen) {
    const want = new Set(TRIAGE_WIDGET_IDS[screen]);
    const allDefs = WIDGET_CATALOG.filter((d5) => d5.screens.includes(screen));
    const visible = allDefs.filter((d5) => want.has(d5.id));
    const hidden = allDefs.filter((d5) => !want.has(d5.id)).map((d5) => d5.id);
    let y5 = 0;
    const widgets = visible.map((def) => {
      const item = {
        i: def.id,
        x: 0,
        y: y5,
        w: def.defaultSize.w,
        h: def.defaultSize.h
      };
      if (def.minW !== void 0) item.minW = def.minW;
      if (def.minH !== void 0) item.minH = def.minH;
      y5 += def.defaultSize.h;
      return item;
    });
    return { widgets, hidden };
  }
  function presetsFor(screen) {
    return [
      {
        id: "preset-default",
        name: "Default",
        screen,
        layout: DEFAULT_LAYOUTS[screen],
        isPreset: true
      },
      {
        id: "preset-compact",
        name: "Compact",
        screen,
        layout: compactLayout(screen),
        isPreset: true
      },
      {
        id: "preset-triage",
        name: "Triage",
        screen,
        layout: triageLayout(screen),
        isPreset: true
      }
    ];
  }
  function readCustom(screen) {
    try {
      const raw = localStorage.getItem(`${STORAGE_KEY_PREFIX}${screen}`);
      if (!raw) return [];
      const parsed = JSON.parse(raw);
      if (!Array.isArray(parsed)) return [];
      return parsed.filter(
        (v4) => typeof v4 === "object" && v4 !== null && typeof v4.id === "string" && typeof v4.name === "string" && typeof v4.screen === "string" && typeof v4.layout === "object"
      );
    } catch {
      return [];
    }
  }
  function writeCustom(screen, views) {
    try {
      localStorage.setItem(`${STORAGE_KEY_PREFIX}${screen}`, JSON.stringify(views));
    } catch {
    }
  }
  function listViews(screen) {
    return [...presetsFor(screen), ...readCustom(screen)];
  }
  function saveView(screen, name, layout) {
    const id = `view-${Date.now().toString(36)}`;
    const view = { id, name, screen, layout, isPreset: false };
    const next = [...readCustom(screen), view];
    writeCustom(screen, next);
    savedViewsToken.value++;
    return view;
  }
  function deleteView(screen, viewId) {
    const next = readCustom(screen).filter((v4) => v4.id !== viewId);
    writeCustom(screen, next);
    if (getActiveViewId(screen) === viewId) {
      setActiveViewId(screen, "preset-default");
    }
    savedViewsToken.value++;
  }
  function getActiveViewId(screen) {
    try {
      return localStorage.getItem(`${ACTIVE_KEY_PREFIX}${screen}`) ?? "preset-default";
    } catch {
      return "preset-default";
    }
  }
  function setActiveViewId(screen, viewId) {
    try {
      localStorage.setItem(`${ACTIVE_KEY_PREFIX}${screen}`, viewId);
    } catch {
    }
    activeViewToken.value++;
  }
  var savedViewsToken = y3(0);
  var activeViewToken = y3(0);

  // src/ui/components/SavedViewsBar.tsx
  function SavedViewsBar({ getCurrentLayout }) {
    void savedViewsToken.value;
    void activeViewToken.value;
    const screen = tabToScreen(activeDashboardTab.value);
    const views = listViews(screen);
    const activeId = getActiveViewId(screen);
    const [savingName, setSavingName] = d2(null);
    y2(() => {
      if (savingName === null) return;
      const handler = (e4) => {
        if (e4.key === "Escape") setSavingName(null);
      };
      window.addEventListener("keydown", handler);
      return () => window.removeEventListener("keydown", handler);
    }, [savingName]);
    const applyView = (view) => {
      setActiveViewId(screen, view.id);
      pendingLayoutApply.value = { screen: view.screen, layout: view.layout };
    };
    const onSaveCurrent = () => {
      const layout = getCurrentLayout();
      if (!layout) return;
      setSavingName("My view");
    };
    const commitSave = (name) => {
      const layout = getCurrentLayout();
      if (!layout) {
        setSavingName(null);
        return;
      }
      const trimmed = name.trim();
      if (!trimmed) {
        setSavingName(null);
        return;
      }
      const view = saveView(screen, trimmed, layout);
      setActiveViewId(screen, view.id);
      setSavingName(null);
    };
    return /* @__PURE__ */ u4("nav", { class: "saved-views-bar", "aria-label": "Saved views", children: /* @__PURE__ */ u4("ul", { class: "saved-views-bar__list", children: [
      views.map((view) => {
        const isActive = view.id === activeId;
        return /* @__PURE__ */ u4("li", { class: "saved-views-bar__item", children: /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: `saved-views-bar__chip${isActive ? " is-active" : ""}${view.isPreset ? " is-preset" : ""}`,
            "aria-pressed": isActive,
            onClick: () => applyView(view),
            title: view.isPreset ? "Built-in preset" : "Custom view \u2014 click \xD7 to delete",
            children: [
              view.name,
              !view.isPreset && /* @__PURE__ */ u4(
                "span",
                {
                  class: "saved-views-bar__delete",
                  role: "button",
                  "aria-label": `Delete ${view.name}`,
                  onClick: (e4) => {
                    e4.stopPropagation();
                    if (confirm(`Delete view "${view.name}"?`)) {
                      deleteView(screen, view.id);
                    }
                  },
                  children: "\xD7"
                }
              )
            ]
          }
        ) }, view.id);
      }),
      /* @__PURE__ */ u4("li", { class: "saved-views-bar__item", children: savingName === null ? /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: "saved-views-bar__chip saved-views-bar__chip--ghost",
          onClick: onSaveCurrent,
          "aria-label": "Save current view",
          children: "+ Save view"
        }
      ) : /* @__PURE__ */ u4(
        "input",
        {
          autoFocus: true,
          class: "saved-views-bar__input",
          type: "text",
          value: savingName,
          placeholder: "View name",
          onInput: (e4) => setSavingName(e4.currentTarget.value),
          onKeyDown: (e4) => {
            if (e4.key === "Enter") commitSave(savingName ?? "");
            if (e4.key === "Escape") setSavingName(null);
          },
          onBlur: () => commitSave(savingName ?? "")
        }
      ) })
    ] }) });
  }

  // src/ui/lib/commands.ts
  var TAB_LABELS = {
    overview: "Overview",
    today: "Today",
    activity: "Activity",
    agents: "Agents",
    breakdowns: "Breakdowns",
    tables: "Sessions",
    projects: "Projects"
  };
  function navigateToTab(tab) {
    activeDashboardTab.value = tab;
  }
  function scrollToWidget(widgetId) {
    const el = document.querySelector(`.grid-stack-item[gs-id="${widgetId}"]`);
    if (!el) return;
    el.scrollIntoView({ behavior: "smooth", block: "center" });
    el.classList.add("widget-flash");
    window.setTimeout(() => el.classList.remove("widget-flash"), 1200);
  }
  function widgetCommands() {
    const screen = tabToScreen(activeDashboardTab.value);
    const defs = widgetsForScreen(screen);
    return defs.map((def) => ({
      id: `widget:${def.id}`,
      group: "widget",
      label: def.title,
      hint: def.description,
      searchTerms: `${def.title} ${def.description ?? ""} ${def.id}`.toLowerCase(),
      run: () => scrollToWidget(def.id)
    }));
  }
  function sessionCommands() {
    const sessions = rawData.value?.sessions_all ?? [];
    return sessions.slice(0, 50).map((s4) => {
      const label = s4.title || s4.display_name || s4.session_id;
      const subtitle = `${s4.model} \xB7 ${s4.project || "\u2014"}`;
      return {
        id: `session:${s4.session_id}`,
        group: "session",
        label,
        hint: subtitle,
        searchTerms: `${label} ${subtitle} ${s4.session_id}`.toLowerCase(),
        run: () => {
          navigateToTab("tables");
          window.setTimeout(() => {
            const row = document.querySelector(
              `tr[data-session-id="${s4.session_id}"]`
            );
            if (row) {
              row.scrollIntoView({ behavior: "smooth", block: "center" });
              row.classList.add("widget-flash");
              window.setTimeout(() => row.classList.remove("widget-flash"), 1200);
            }
          }, 80);
        }
      };
    });
  }
  function projectCommands() {
    const sessions = rawData.value?.sessions_all ?? [];
    const seen = /* @__PURE__ */ new Map();
    for (const s4 of sessions) {
      const existing = seen.get(s4.project);
      if (existing) {
        existing.sessions += 1;
        existing.cost += s4.cost;
      } else {
        seen.set(s4.project, {
          project: s4.project,
          display: s4.custom_label || s4.display_name || s4.project,
          sessions: 1,
          cost: s4.cost
        });
      }
    }
    return [...seen.values()].sort((a4, b4) => b4.cost - a4.cost).slice(0, 50).map((p5) => ({
      id: `project:${p5.project}`,
      group: "project",
      label: p5.display,
      hint: `${p5.sessions.toLocaleString()} sessions \xB7 $${p5.cost.toFixed(2)}`,
      searchTerms: `${p5.project} ${p5.display}`.toLowerCase(),
      run: () => {
        navigateToTab("projects");
      }
    }));
  }
  function modelCommands() {
    const models = rawData.value?.all_models ?? [];
    return models.map((m5) => ({
      id: `model:${m5}`,
      group: "model",
      label: m5,
      hint: "Toggle model filter",
      searchTerms: m5.toLowerCase(),
      run: () => {
        const next = new Set(selectedModels.value);
        if (next.has(m5)) next.delete(m5);
        else next.add(m5);
        selectedModels.value = next;
      }
    }));
  }
  function actionCommands(ctx) {
    return [
      {
        id: "action:rescan",
        group: "action",
        label: "Rescan",
        hint: "Re-scan transcripts and refresh dashboard",
        searchTerms: "rescan refresh sync reload",
        run: () => {
          void ctx.triggerRescan();
        }
      },
      {
        id: "action:settings",
        group: "action",
        label: "Open settings",
        searchTerms: "settings preferences config",
        run: () => {
          settingsModalOpen.value = true;
        }
      },
      {
        id: "action:backup",
        group: "action",
        label: "Open backup and snapshots",
        searchTerms: "backup snapshot export",
        run: () => {
          backupModalOpen.value = true;
        }
      },
      {
        id: "action:edit-layout",
        group: "action",
        label: editMode.value ? "Exit edit layout" : "Edit layout",
        searchTerms: "edit layout customize widgets rearrange",
        run: () => {
          editMode.value = !editMode.value;
        }
      },
      {
        id: "action:theme",
        group: "action",
        label: themeMode.value === "dark" ? "Switch to light theme" : "Switch to dark theme",
        searchTerms: "theme dark light mode",
        run: () => ctx.toggleTheme()
      },
      {
        id: "action:open-monitor",
        group: "action",
        label: "Open live monitor",
        hint: "Real-time provider lanes",
        searchTerms: "live monitor real-time provider",
        run: () => {
          window.location.href = "/monitor";
        }
      }
    ];
  }
  function navigationCommands() {
    return Object.keys(TAB_LABELS).map((tab) => ({
      id: `nav:${tab}`,
      group: "navigate",
      label: `Go to ${TAB_LABELS[tab]}`,
      searchTerms: `${TAB_LABELS[tab]} tab navigate go to`.toLowerCase(),
      run: () => navigateToTab(tab)
    }));
  }
  function buildCommands(ctx) {
    return [
      ...navigationCommands(),
      ...actionCommands(ctx),
      ...widgetCommands(),
      ...sessionCommands(),
      ...projectCommands(),
      ...modelCommands()
    ];
  }
  function filterCommands(commands, query) {
    const q4 = query.trim().toLowerCase();
    if (!q4) return commands;
    const tokens = q4.split(/\s+/).filter(Boolean);
    return commands.filter((c4) => {
      const hay = `${c4.label} ${c4.searchTerms}`.toLowerCase();
      return tokens.every((t4) => hay.includes(t4));
    });
  }

  // src/ui/components/CommandPalette.tsx
  var GROUP_LABEL = {
    navigate: "Navigate",
    widget: "Widgets",
    session: "Sessions",
    project: "Projects",
    model: "Models",
    action: "Actions"
  };
  var GROUP_ORDER = [
    "navigate",
    "action",
    "widget",
    "session",
    "project",
    "model"
  ];
  function CommandPalette({ triggerRescan, toggleTheme: toggleTheme2 }) {
    const open = commandPaletteOpen.value;
    const [query, setQuery] = d2("");
    const [highlight, setHighlight] = d2(0);
    const inputRef = A2(null);
    const listRef = A2(null);
    const commands = T2(
      () => open ? buildCommands({ triggerRescan, toggleTheme: toggleTheme2 }) : [],
      [open, triggerRescan, toggleTheme2]
    );
    const filtered = T2(
      () => filterCommands(commands, query),
      [commands, query]
    );
    y2(() => {
      if (!open) return;
      setQuery("");
      setHighlight(0);
      window.setTimeout(() => inputRef.current?.focus(), 0);
    }, [open]);
    y2(() => {
      if (highlight >= filtered.length) {
        setHighlight(Math.max(0, filtered.length - 1));
      }
    }, [filtered.length, highlight]);
    y2(() => {
      if (!open) return;
      const el = listRef.current?.querySelector(
        `[data-cmd-index="${highlight}"]`
      );
      el?.scrollIntoView({ block: "nearest" });
    }, [highlight, open]);
    if (!open) return null;
    const close = () => {
      commandPaletteOpen.value = false;
    };
    const onKeyDown = (e4) => {
      if (e4.key === "Escape") {
        e4.preventDefault();
        close();
        return;
      }
      if (e4.key === "ArrowDown") {
        e4.preventDefault();
        setHighlight((h5) => Math.min(filtered.length - 1, h5 + 1));
        return;
      }
      if (e4.key === "ArrowUp") {
        e4.preventDefault();
        setHighlight((h5) => Math.max(0, h5 - 1));
        return;
      }
      if (e4.key === "Enter") {
        e4.preventDefault();
        const cmd = filtered[highlight];
        if (cmd) {
          cmd.run();
          close();
        }
      }
    };
    const grouped = /* @__PURE__ */ new Map();
    for (const cmd of filtered) {
      const list = grouped.get(cmd.group) ?? [];
      list.push(cmd);
      grouped.set(cmd.group, list);
    }
    let runningIndex = -1;
    const flatIndex = /* @__PURE__ */ new Map();
    for (const cmd of filtered) {
      runningIndex++;
      flatIndex.set(cmd.id, runningIndex);
    }
    return /* @__PURE__ */ u4(
      "div",
      {
        class: "cmd-palette-overlay",
        role: "dialog",
        "aria-modal": "true",
        "aria-label": "Command palette",
        onClick: (e4) => {
          if (e4.target === e4.currentTarget) close();
        },
        children: /* @__PURE__ */ u4("div", { class: "cmd-palette", onKeyDown, children: [
          /* @__PURE__ */ u4("div", { class: "cmd-palette__input-row", children: [
            /* @__PURE__ */ u4("span", { class: "cmd-palette__prompt", children: "[>" }),
            /* @__PURE__ */ u4(
              "input",
              {
                ref: inputRef,
                class: "cmd-palette__input",
                type: "text",
                placeholder: "Search\u2026",
                value: query,
                onInput: (e4) => setQuery(e4.currentTarget.value),
                autoComplete: "off",
                spellcheck: false,
                enterKeyHint: "go"
              }
            ),
            /* @__PURE__ */ u4("span", { class: "cmd-palette__hint", children: "[esc]" })
          ] }),
          /* @__PURE__ */ u4("div", { class: "cmd-palette__list", ref: listRef, children: [
            filtered.length === 0 && /* @__PURE__ */ u4("div", { class: "cmd-palette__empty", children: "No results" }),
            GROUP_ORDER.map((group) => {
              const items = grouped.get(group);
              if (!items || items.length === 0) return null;
              return /* @__PURE__ */ u4("div", { class: "cmd-palette__group", children: [
                /* @__PURE__ */ u4("div", { class: "cmd-palette__group-label", children: GROUP_LABEL[group] }),
                items.map((cmd) => {
                  const idx = flatIndex.get(cmd.id) ?? 0;
                  const isActive = idx === highlight;
                  return /* @__PURE__ */ u4(
                    "button",
                    {
                      type: "button",
                      "data-cmd-index": idx,
                      class: `cmd-palette__row${isActive ? " is-active" : ""}`,
                      onMouseEnter: () => setHighlight(idx),
                      onClick: () => {
                        cmd.run();
                        close();
                      },
                      children: [
                        /* @__PURE__ */ u4("span", { class: "cmd-palette__row-label", children: cmd.label }),
                        cmd.hint && /* @__PURE__ */ u4("span", { class: "cmd-palette__row-hint", children: cmd.hint })
                      ]
                    },
                    cmd.id
                  );
                })
              ] }, group);
            })
          ] }),
          /* @__PURE__ */ u4("div", { class: "cmd-palette__footer", children: [
            /* @__PURE__ */ u4("span", { children: "\u2191\u2193 to move" }),
            /* @__PURE__ */ u4("span", { children: "\u21B5 to run" }),
            /* @__PURE__ */ u4("span", { children: "esc to close" })
          ] })
        ] })
      }
    );
  }

  // src/ui/components/FilterBar.tsx
  function shortModelName(full) {
    return full.replace(/^claude-/, "").replace(/-\d{8}$/, "");
  }
  var SECTION_FILTER_GROUPS = {
    overview: ["range", "bucket"],
    today: [],
    activity: ["range", "bucket", "provider", "models"],
    agents: ["range", "provider"],
    breakdowns: ["range", "bucket", "provider", "models"],
    tables: ["range", "provider", "models", "project-search"],
    projects: ["project-search"]
  };
  var RANGES = ["7d", "30d", "90d", "all"];
  var RANGE_LABEL = {
    "7d": "7d",
    "30d": "30d",
    "90d": "90d",
    all: "All"
  };
  var BUCKETS = ["day", "week"];
  var BUCKET_LABEL = { day: "Day", week: "Week" };
  var PROVIDERS = ["both", "claude", "codex"];
  var PROVIDER_LABEL = {
    both: "Both",
    claude: "Claude",
    codex: "Codex"
  };
  function modelPriority(m5) {
    const ml = m5.toLowerCase();
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
    const [modelsOpen, setModelsOpen] = d2(false);
    const popoverRef = A2(null);
    const chipRef = A2(null);
    y2(() => {
      if (!modelsOpen) return;
      const onDocClick = (e4) => {
        const t4 = e4.target;
        if (popoverRef.current?.contains(t4)) return;
        if (chipRef.current?.contains(t4)) return;
        setModelsOpen(false);
      };
      const onKey = (e4) => {
        if (e4.key === "Escape") setModelsOpen(false);
      };
      document.addEventListener("mousedown", onDocClick);
      document.addEventListener("keydown", onKey);
      return () => {
        document.removeEventListener("mousedown", onDocClick);
        document.removeEventListener("keydown", onKey);
      };
    }, [modelsOpen]);
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
    const activeGroups = SECTION_FILTER_GROUPS[activeDashboardTab.value] ?? Object.values(SECTION_FILTER_GROUPS).flat();
    const show = (group) => activeGroups.includes(group);
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
    const toggleMobileFilters = () => {
      mobile_filters_expanded.value = !mobile_filters_expanded.value;
      onURLUpdate();
    };
    const selectedModelCount = selectedModels.value.size;
    const totalModels = sortedModels.length;
    const allSelected = selectedModelCount === totalModels;
    const modelChipLabel = totalModels === 0 ? "Models" : allSelected ? `Models \xB7 all ${totalModels}` : `Models \xB7 ${selectedModelCount}/${totalModels}`;
    const providerSummary = hasCodexData ? PROVIDER_LABEL[selectedProvider.value] : null;
    const filterSummary = [
      RANGE_LABEL[selectedRange.value],
      BUCKET_LABEL[selectedBucket.value],
      providerSummary,
      allSelected ? `All ${totalModels} models` : `${selectedModelCount}/${totalModels} models`,
      projectSearchQuery.value ? `Project: ${projectSearchQuery.value}` : null
    ].filter(Boolean).join(" \xB7 ");
    return /* @__PURE__ */ u4(
      "div",
      {
        id: "filter-bar",
        role: "toolbar",
        "aria-label": "Filters",
        class: `filter-dock${mobile_filters_expanded.value ? " expanded" : " collapsed"}`,
        children: [
          /* @__PURE__ */ u4("div", { class: "mobile-filter-header", children: [
            /* @__PURE__ */ u4("div", { class: "mobile-filter-summary", "aria-live": "polite", children: [
              /* @__PURE__ */ u4("span", { class: "mobile-filter-summary-label", children: "Filters" }),
              /* @__PURE__ */ u4("span", { class: "mobile-filter-summary-text", children: filterSummary })
            ] }),
            /* @__PURE__ */ u4(
              "button",
              {
                class: "mobile-filter-toggle",
                type: "button",
                "aria-expanded": mobile_filters_expanded.value,
                "aria-controls": "filter-sections",
                onClick: toggleMobileFilters,
                children: mobile_filters_expanded.value ? "Hide" : "Show"
              }
            )
          ] }),
          /* @__PURE__ */ u4("div", { id: "filter-sections", class: "filter-sections", children: [
            show("range") && /* @__PURE__ */ u4("div", { class: "filter-group", children: [
              /* @__PURE__ */ u4("span", { class: "filter-group__label", children: "Range" }),
              /* @__PURE__ */ u4("div", { class: "segmented", role: "group", "aria-label": "Date range", children: RANGES.map((range) => /* @__PURE__ */ u4(
                "button",
                {
                  class: `segmented__item${selectedRange.value === range ? " is-active" : ""}`,
                  type: "button",
                  "data-range": range,
                  onClick: () => setRange(range),
                  children: RANGE_LABEL[range]
                },
                range
              )) })
            ] }),
            show("bucket") && /* @__PURE__ */ u4("div", { class: "filter-group", children: [
              /* @__PURE__ */ u4("span", { class: "filter-group__label", children: "Bucket" }),
              /* @__PURE__ */ u4("div", { class: "segmented", role: "group", "aria-label": "Chart bucket", children: BUCKETS.map((bucket) => /* @__PURE__ */ u4(
                "button",
                {
                  class: `segmented__item${selectedBucket.value === bucket ? " is-active" : ""}`,
                  type: "button",
                  "data-bucket": bucket,
                  onClick: () => setBucket(bucket),
                  children: BUCKET_LABEL[bucket]
                },
                bucket
              )) })
            ] }),
            show("provider") && hasCodexData && /* @__PURE__ */ u4("div", { class: "filter-group", children: [
              /* @__PURE__ */ u4("span", { class: "filter-group__label", children: "Provider" }),
              /* @__PURE__ */ u4("div", { class: "segmented", role: "group", "aria-label": "Provider", children: PROVIDERS.map((provider) => /* @__PURE__ */ u4(
                "button",
                {
                  class: `segmented__item${selectedProvider.value === provider ? " is-active" : ""}`,
                  type: "button",
                  "data-provider": provider,
                  onClick: () => setProvider(provider),
                  children: PROVIDER_LABEL[provider]
                },
                provider
              )) })
            ] }),
            show("models") && /* @__PURE__ */ u4("div", { class: "filter-group filter-group--chip", children: [
              /* @__PURE__ */ u4(
                "button",
                {
                  ref: chipRef,
                  type: "button",
                  class: `filter-chip${modelsOpen ? " is-open" : ""}`,
                  "aria-expanded": modelsOpen,
                  "aria-controls": "filter-models-popover",
                  onClick: () => setModelsOpen((o4) => !o4),
                  children: [
                    modelChipLabel,
                    /* @__PURE__ */ u4("span", { class: "filter-chip__caret", "aria-hidden": "true", children: "\u25BE" })
                  ]
                }
              ),
              modelsOpen && /* @__PURE__ */ u4(
                "div",
                {
                  ref: popoverRef,
                  id: "filter-models-popover",
                  class: "filter-popover",
                  role: "dialog",
                  "aria-label": "Select models",
                  children: [
                    /* @__PURE__ */ u4("div", { class: "filter-popover__header", children: [
                      /* @__PURE__ */ u4("span", { children: [
                        selectedModelCount,
                        "/",
                        totalModels,
                        " selected"
                      ] }),
                      /* @__PURE__ */ u4("div", { class: "filter-popover__actions", children: [
                        /* @__PURE__ */ u4("button", { class: "filter-link", type: "button", onClick: selectAll, children: "All" }),
                        /* @__PURE__ */ u4("button", { class: "filter-link", type: "button", onClick: clearAll, children: "None" })
                      ] })
                    ] }),
                    /* @__PURE__ */ u4("div", { class: "filter-popover__body", role: "group", "aria-label": "Model filters", children: sortedModels.map((model) => {
                      const checked = selectedModels.value.has(model);
                      return /* @__PURE__ */ u4(
                        "label",
                        {
                          class: `filter-popover__row${checked ? " is-checked" : ""}`,
                          "data-model": model,
                          children: [
                            /* @__PURE__ */ u4(
                              "input",
                              {
                                type: "checkbox",
                                value: model,
                                checked,
                                onChange: (e4) => toggleModel(model, e4.currentTarget.checked),
                                "aria-label": model
                              }
                            ),
                            /* @__PURE__ */ u4("span", { class: "filter-popover__row-text", children: shortModelName(model) })
                          ]
                        },
                        model
                      );
                    }) })
                  ]
                }
              )
            ] }),
            show("project-search") && /* @__PURE__ */ u4("div", { class: "filter-group filter-group--search", children: [
              /* @__PURE__ */ u4("label", { for: "project-search", class: "filter-group__label", children: "Project" }),
              /* @__PURE__ */ u4(
                "input",
                {
                  type: "text",
                  id: "project-search",
                  name: "project-search",
                  placeholder: "Search projects\u2026",
                  "aria-label": "Filter by project name",
                  autoComplete: "off",
                  spellcheck: false,
                  enterKeyHint: "search",
                  value: projectSearchQuery.value,
                  onInput: onSearchInput,
                  class: "project-search-input"
                }
              ),
              projectSearchQuery.value && /* @__PURE__ */ u4("button", { class: "filter-link", id: "project-clear-btn", type: "button", onClick: clearSearch, children: "Clear" })
            ] })
          ] })
        ]
      }
    );
  }

  // src/ui/components/Footer.tsx
  function Footer() {
    return /* @__PURE__ */ u4("footer", { children: /* @__PURE__ */ u4("div", { class: "footer-content", children: [
      /* @__PURE__ */ u4("p", { children: [
        "Cost estimates based on Anthropic and OpenAI API pricing (",
        /* @__PURE__ */ u4(
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
        /* @__PURE__ */ u4(
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
      /* @__PURE__ */ u4("p", { children: [
        "GitHub:",
        " ",
        /* @__PURE__ */ u4(
          "a",
          {
            href: "https://github.com/po4yka/heimdall",
            target: "_blank",
            rel: "noopener noreferrer",
            children: "po4yka/heimdall"
          }
        ),
        " ",
        "\xB7 License: MIT"
      ] })
    ] }) });
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
      button.textContent = "\u21BB Scanning\u2026";
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

  // src/ui/components/Header.tsx
  function Header({
    onDataReload,
    onThemeToggle,
    navigationHref,
    navigationLabel
  }) {
    const headerRef = A2(null);
    const btnRef = A2(null);
    const triggerRef = A2(null);
    y2(() => {
      const themeColorMeta = document.querySelector('meta[name="theme-color"]');
      if (!themeColorMeta) return;
      themeColorMeta.setAttribute("content", themeMode.value === "light" ? "#F5F5F5" : "#000000");
    }, [themeMode.value]);
    y2(() => {
      if (!headerRef.current) return;
      const root = document.documentElement;
      const updateOffset = () => {
        if (!headerRef.current) return;
        root.style.setProperty("--header-offset", `${Math.ceil(headerRef.current.getBoundingClientRect().height)}px`);
      };
      updateOffset();
      const observer = new ResizeObserver(() => updateOffset());
      observer.observe(headerRef.current);
      window.addEventListener("resize", updateOffset);
      return () => {
        observer.disconnect();
        window.removeEventListener("resize", updateOffset);
      };
    }, []);
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
    const isEditing = editMode.value;
    const isMobile = typeof window !== "undefined" && window.innerWidth < 720;
    const mode = themeMode.value;
    const icon = mode === "dark" ? /* @__PURE__ */ u4("svg", { "aria-hidden": "true", focusable: "false", width: "14", height: "14", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", children: /* @__PURE__ */ u4("path", { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }) }) : /* @__PURE__ */ u4("svg", { "aria-hidden": "true", focusable: "false", width: "14", height: "14", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", children: [
      /* @__PURE__ */ u4("circle", { cx: "12", cy: "12", r: "5" }),
      /* @__PURE__ */ u4("line", { x1: "12", y1: "1", x2: "12", y2: "3" }),
      /* @__PURE__ */ u4("line", { x1: "12", y1: "21", x2: "12", y2: "23" }),
      /* @__PURE__ */ u4("line", { x1: "4.22", y1: "4.22", x2: "5.64", y2: "5.64" }),
      /* @__PURE__ */ u4("line", { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }),
      /* @__PURE__ */ u4("line", { x1: "1", y1: "12", x2: "3", y2: "12" }),
      /* @__PURE__ */ u4("line", { x1: "21", y1: "12", x2: "23", y2: "12" }),
      /* @__PURE__ */ u4("line", { x1: "4.22", y1: "19.78", x2: "5.64", y2: "18.36" }),
      /* @__PURE__ */ u4("line", { x1: "18.36", y1: "5.64", x2: "19.78", y2: "4.22" })
    ] });
    const info = versionInfo.value;
    const versionTitle = info ? `Heimdall \xB7 v${info.current}` : "Heimdall";
    const updateUrl = info?.update_available ? info.latest_url : null;
    const planLabel2 = planBadge.value;
    return /* @__PURE__ */ u4("header", { ref: headerRef, children: [
      /* @__PURE__ */ u4("h1", { title: versionTitle, "aria-label": "Heimdall", children: [
        /* @__PURE__ */ u4("span", { class: "header-logo-mark", "aria-hidden": "true", children: [
          /* @__PURE__ */ u4("span", { class: "header-logo-mark__pre", children: "Code" }),
          " ",
          /* @__PURE__ */ u4("span", { class: "header-logo-mark__post", children: "Usage" })
        ] }),
        planLabel2 && /* @__PURE__ */ u4("span", { class: "header-plan-badge", "aria-hidden": "true", children: planLabel2 })
      ] }),
      /* @__PURE__ */ u4("div", { class: "meta", children: metaText.value }),
      /* @__PURE__ */ u4("div", { class: "header-actions", children: [
        navigationHref && navigationLabel && /* @__PURE__ */ u4("a", { class: "header-button header-button--link", href: navigationHref, children: [
          "[",
          navigationLabel,
          "]"
        ] }),
        updateUrl && /* @__PURE__ */ u4(
          "a",
          {
            class: "header-button header-button--link header-button--update",
            href: updateUrl,
            target: "_blank",
            rel: "noopener noreferrer",
            title: `Latest: v${info?.latest} (current: v${info?.current})`,
            children: [
              "[Update v",
              info?.latest,
              " \u2192]"
            ]
          }
        ),
        /* @__PURE__ */ u4(
          "button",
          {
            class: "theme-toggle",
            type: "button",
            onClick: onThemeToggle,
            "aria-label": "Toggle theme",
            children: icon
          }
        ),
        !isMobile && /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: `header-button${isEditing ? " header-button--active" : ""}`,
            onClick: () => {
              editMode.value = !editMode.value;
            },
            "aria-pressed": isEditing,
            "aria-label": isEditing ? "Done editing layout" : "Edit layout",
            children: isEditing ? "[Done]" : "[Edit layout]"
          }
        ),
        !isMobile && /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: "header-button",
            onClick: () => {
              backupModalOpen.value = true;
              if (!/^#\/backup\b/.test(window.location.hash)) {
                history.replaceState(null, "", `${window.location.pathname}${window.location.search}#/backup`);
              }
            },
            "aria-label": "Open backup and snapshots",
            children: "[Backup]"
          }
        ),
        !isMobile && /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: "header-button",
            onClick: () => {
              settingsModalOpen.value = true;
              if (!/^#\/settings\b/.test(window.location.hash)) {
                history.replaceState(null, "", `${window.location.pathname}${window.location.search}#/settings`);
              }
            },
            "aria-label": "Open settings",
            children: "[Settings]"
          }
        ),
        /* @__PURE__ */ u4(
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
        /* @__PURE__ */ u4(InlineStatus, { placement: "rescan", inline: true }),
        /* @__PURE__ */ u4(InlineStatus, { placement: "header-refresh", inline: true, dismissable: false })
      ] })
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
    return d5.getUTCFullYear() + "-" + String(d5.getUTCMonth() + 1).padStart(2, "0") + "-" + String(d5.getUTCDate()).padStart(2, "0") + "_" + String(d5.getUTCHours()).padStart(2, "0") + String(d5.getUTCMinutes()).padStart(2, "0");
  }
  function downloadCSV(reportType, header, rows2) {
    const lines = [header.map(csvField).join(",")];
    for (const row of rows2) lines.push(row.map(csvField).join(","));
    const blob = new Blob([lines.join("\n")], { type: "text/csv;charset=utf-8;" });
    const a4 = document.createElement("a");
    a4.href = URL.createObjectURL(blob);
    a4.download = reportType + "_" + csvTimestamp() + ".csv";
    a4.click();
    setTimeout(() => URL.revokeObjectURL(a4.href), 1e3);
  }

  // src/ui/components/today/DatePicker.tsx
  function addDays(dateStr, delta) {
    const d5 = /* @__PURE__ */ new Date(`${dateStr}T00:00:00`);
    d5.setDate(d5.getDate() + delta);
    return d5.toISOString().slice(0, 10);
  }
  function localToday() {
    const now = /* @__PURE__ */ new Date();
    const y5 = now.getFullYear();
    const m5 = String(now.getMonth() + 1).padStart(2, "0");
    const d5 = String(now.getDate()).padStart(2, "0");
    return `${y5}-${m5}-${d5}`;
  }
  function DatePicker({ onDateChange }) {
    const today = localToday();
    const resolvedDate = selectedDate.value ?? todayData.value?.day ?? today;
    const isToday = resolvedDate === today || selectedDate.value === null;
    function previousDay() {
      const next = addDays(resolvedDate, -1);
      selectedDate.value = next;
      onDateChange(next);
    }
    function nextDay() {
      if (resolvedDate >= today) return;
      const next = addDays(resolvedDate, 1);
      selectedDate.value = next === today ? null : next;
      onDateChange(next === today ? null : next);
    }
    function goToday() {
      selectedDate.value = null;
      onDateChange(null);
    }
    function onPick(e4) {
      const val = e4.target.value;
      if (!val) return;
      const next = val === today ? null : val;
      selectedDate.value = next;
      onDateChange(next);
    }
    return /* @__PURE__ */ u4("div", { class: "date-picker", children: [
      /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: "date-picker-btn",
          onClick: previousDay,
          "aria-label": "Previous day",
          children: "\u25C0"
        }
      ),
      /* @__PURE__ */ u4(
        "input",
        {
          type: "date",
          class: "date-picker-input",
          value: resolvedDate,
          max: today,
          onChange: onPick,
          "aria-label": "Select date"
        }
      ),
      /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: "date-picker-btn",
          onClick: nextDay,
          disabled: resolvedDate >= today,
          "aria-label": "Next day",
          children: "\u25B6"
        }
      ),
      /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: `date-picker-btn date-picker-today-btn${isToday ? " date-picker-btn--active" : ""}`,
          onClick: goToday,
          disabled: isToday,
          "aria-label": "Go to today",
          children: "Today"
        }
      )
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
  function dashboardChartOptions(type) {
    const monoStack = 'var(--font-mono), "Geist Mono", ui-monospace, monospace';
    const axisLabelStyle = {
      colors: cssVar("--text-secondary"),
      fontFamily: monoStack,
      fontSize: "11px",
      letterSpacing: "0.04em"
    };
    const base = {
      chart: {
        type,
        height: "100%",
        background: "transparent",
        toolbar: { show: false },
        fontFamily: monoStack,
        animations: { enabled: false }
      },
      theme: { mode: apexThemeMode() },
      legend: {
        show: true,
        position: type === "donut" ? "bottom" : "top",
        fontFamily: monoStack,
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
        style: { fontFamily: monoStack, fontSize: "11px" }
      },
      dataLabels: { enabled: false }
    };
    if (type === "line") {
      if (base.legend) base.legend.show = false;
      base.fill = { type: "solid", opacity: 0 };
    }
    return base;
  }

  // src/ui/components/today/DaysHoursHeatmap.tsx
  function cellOpacity(value, max2) {
    if (max2 <= 0 || value <= 0) return 0.05;
    return Math.min(0.05 + 0.85 * (value / max2), 0.9);
  }
  function DaysHoursHeatmap({ cells, daysCount, title, onDayClick }) {
    const maxCost = Math.max(...cells.map((c4) => c4.cost_nanos), 1);
    const daysSet = /* @__PURE__ */ new Set();
    for (const c4 of cells) daysSet.add(c4.day);
    const days = Array.from(daysSet).sort((a4, b4) => b4.localeCompare(a4));
    const lookup = /* @__PURE__ */ new Map();
    for (const c4 of cells) lookup.set(`${c4.day},${c4.hour}`, c4);
    const HOUR_LABELS = [0, 6, 12, 18, 23];
    return /* @__PURE__ */ u4("div", { class: "days-hours-heatmap-wrap", children: [
      /* @__PURE__ */ u4("div", { class: "days-hours-heatmap-title", children: title }),
      /* @__PURE__ */ u4(
        "div",
        {
          class: "days-hours-heatmap",
          style: {
            display: "grid",
            gridTemplateColumns: `60px repeat(24, 1fr)`,
            gap: "1px"
          },
          role: "figure",
          "aria-label": `${daysCount} days by 24 hours heatmap`,
          children: [
            /* @__PURE__ */ u4("div", {}),
            Array.from({ length: 24 }, (_4, h5) => /* @__PURE__ */ u4(
              "div",
              {
                style: {
                  fontFamily: "var(--font-mono)",
                  fontSize: "8px",
                  textAlign: "center",
                  color: "var(--text-secondary)",
                  paddingBottom: "2px",
                  visibility: HOUR_LABELS.includes(h5) ? "visible" : "hidden"
                },
                children: String(h5).padStart(2, "0")
              },
              h5
            )),
            days.map((day) => [
              // Day label
              /* @__PURE__ */ u4(
                "div",
                {
                  style: {
                    fontFamily: "var(--font-mono)",
                    fontSize: "9px",
                    color: "var(--text-secondary)",
                    display: "flex",
                    alignItems: "center",
                    paddingRight: "4px",
                    whiteSpace: "nowrap"
                  },
                  children: [
                    day.slice(5),
                    " "
                  ]
                },
                `label-${day}`
              ),
              // Hour cells
              ...Array.from({ length: 24 }, (_4, hour) => {
                const cell = lookup.get(`${day},${hour}`);
                const cost = cell?.cost_nanos ?? 0;
                const turns = cell?.turns ?? 0;
                const opacity = cellOpacity(cost, maxCost);
                const bg = withAlpha("--text-primary", opacity);
                const costUsd = cost / 1e9;
                const title_ = `${day} ${String(hour).padStart(2, "0")}:00 \u2014 ${fmtCost(costUsd)} / ${fmt(turns)} turn${turns !== 1 ? "s" : ""}`;
                const clickable = onDayClick && cost > 0;
                return /* @__PURE__ */ u4(
                  "div",
                  {
                    class: `days-hours-heatmap-cell${clickable ? " days-hours-heatmap-cell--clickable" : ""}`,
                    title: title_,
                    role: "img",
                    "aria-label": title_,
                    style: { background: bg },
                    onClick: clickable ? () => onDayClick(day) : void 0
                  },
                  `${day}-${hour}`
                );
              })
            ])
          ]
        }
      )
    ] });
  }

  // src/ui/components/today/HourHeatstrip.tsx
  function cellOpacity2(value, max2) {
    if (max2 <= 0 || value <= 0) return 0.05;
    return Math.min(0.05 + 0.85 * (value / max2), 0.9);
  }
  function HourHeatstrip({ hours }) {
    const maxCost = Math.max(...hours.map((h5) => h5.cost_nanos), 1);
    return /* @__PURE__ */ u4("div", { class: "hour-heatstrip", role: "figure", "aria-label": "Hour-by-hour cost heatstrip", children: hours.map((h5) => {
      const opacity = cellOpacity2(h5.cost_nanos, maxCost);
      const bg = withAlpha("--text-primary", opacity);
      const costUsd = h5.cost_nanos / 1e9;
      const title = `${String(h5.hour).padStart(2, "0")}:00 \u2014 ${fmtCost(costUsd)} / ${fmt(h5.turns)} turn${h5.turns !== 1 ? "s" : ""}`;
      return /* @__PURE__ */ u4(
        "div",
        {
          class: "hour-heatstrip-cell",
          title,
          role: "img",
          "aria-label": title,
          style: { background: bg }
        },
        h5.hour
      );
    }) });
  }

  // src/ui/components/today/HourTimeline.tsx
  function HourTimeline({ hours }) {
    const maxCost = Math.max(...hours.map((h5) => h5.cost_nanos), 1);
    const totalCost = hours.reduce((s4, h5) => s4 + h5.cost_nanos, 0);
    if (totalCost === 0) {
      return /* @__PURE__ */ u4("div", { class: "today-empty-state", style: { flex: 1 }, children: /* @__PURE__ */ u4("span", { children: "No activity for this day" }) });
    }
    return /* @__PURE__ */ u4("div", { style: { flex: 1, display: "flex", flexDirection: "column" }, children: [
      /* @__PURE__ */ u4(
        "div",
        {
          style: {
            display: "flex",
            alignItems: "flex-end",
            gap: "2px",
            flex: 1,
            minHeight: "60px"
          },
          children: hours.map((h5) => {
            const pct = h5.cost_nanos / maxCost * 100;
            const background = h5.cost_nanos > 0 ? withAlpha("--text-display", 0.35 + pct / 100 * 0.55) : cssVar("--border");
            const costUsd = h5.cost_nanos / 1e9;
            const totalTokens2 = h5.input_tokens + h5.output_tokens + h5.cache_read_tokens + h5.cache_creation_tokens;
            const title = `${String(h5.hour).padStart(2, "0")}:00 \u2014 ${fmtCost(costUsd)} / ${fmt(h5.turns)} turn${h5.turns !== 1 ? "s" : ""} / ${fmt(totalTokens2)} tokens`;
            return /* @__PURE__ */ u4(
              "div",
              {
                title,
                style: {
                  flex: 1,
                  height: `${Math.max(pct, 2)}%`,
                  background,
                  borderRadius: 0
                }
              },
              h5.hour
            );
          })
        }
      ),
      /* @__PURE__ */ u4("div", { style: { display: "flex", gap: "2px", marginTop: "6px" }, children: hours.map((h5) => /* @__PURE__ */ u4(
        "span",
        {
          style: {
            flex: 1,
            fontFamily: "var(--font-mono)",
            fontSize: "9px",
            textAlign: "center",
            letterSpacing: "0.04em",
            color: cssVar("--text-secondary"),
            visibility: [0, 6, 12, 18].includes(h5.hour) ? "visible" : "hidden"
          },
          children: String(h5.hour).padStart(2, "0")
        },
        h5.hour
      )) })
    ] });
  }

  // src/ui/components/today/TodayKpis.tsx
  function TodayKpis({ totals, day }) {
    const costUsd = totals.cost_nanos / 1e9;
    const peakLabel = totals.peak_hour !== null ? `${String(totals.peak_hour).padStart(2, "0")}:00` : "--";
    const cards = [
      { label: "Cost", value: fmtCostBig(costUsd), sub: day },
      { label: "Tokens", value: fmt(totals.total_tokens), sub: "input + output + cache" },
      { label: "Turns", value: fmt(totals.turns), sub: "API calls" },
      { label: "Peak hour", value: peakLabel, sub: "highest cost hour" }
    ];
    return /* @__PURE__ */ u4("div", { class: "today-kpi-grid", children: cards.map((card) => /* @__PURE__ */ u4("div", { class: "stat-card", children: [
      /* @__PURE__ */ u4("div", { class: "stat-label", children: card.label }),
      /* @__PURE__ */ u4("div", { class: "stat-value", children: card.value }),
      card.sub && /* @__PURE__ */ u4("div", { class: "stat-sub", children: card.sub })
    ] }, card.label)) });
  }

  // src/ui/components/today/WeekdayHourHeatmap.tsx
  var DOW_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  function cellOpacity3(value, max2) {
    if (max2 <= 0 || value <= 0) return 0.05;
    return Math.min(0.05 + 0.85 * (value / max2), 0.9);
  }
  function WeekdayHourHeatmap({ cells }) {
    const maxCost = Math.max(...cells.map((c4) => c4.cost_nanos), 1);
    const lookup = /* @__PURE__ */ new Map();
    for (const c4 of cells) lookup.set(`${c4.dow},${c4.hour}`, c4);
    const HOUR_LABELS = [0, 6, 12, 18, 23];
    return /* @__PURE__ */ u4(
      "div",
      {
        class: "days-hours-heatmap",
        style: {
          display: "grid",
          gridTemplateColumns: `40px repeat(24, 1fr)`,
          gap: "1px"
        },
        role: "figure",
        "aria-label": "7\xD724 weekday by hour pattern heatmap (90-day window)",
        children: [
          /* @__PURE__ */ u4("div", {}),
          Array.from({ length: 24 }, (_4, h5) => /* @__PURE__ */ u4(
            "div",
            {
              style: {
                fontFamily: "var(--font-mono)",
                fontSize: "8px",
                textAlign: "center",
                color: "var(--text-secondary)",
                paddingBottom: "2px",
                visibility: HOUR_LABELS.includes(h5) ? "visible" : "hidden"
              },
              children: String(h5).padStart(2, "0")
            },
            h5
          )),
          Array.from({ length: 7 }, (_4, dow) => [
            /* @__PURE__ */ u4(
              "div",
              {
                style: {
                  fontFamily: "var(--font-mono)",
                  fontSize: "9px",
                  color: "var(--text-secondary)",
                  display: "flex",
                  alignItems: "center"
                },
                children: DOW_LABELS[dow]
              },
              `label-${dow}`
            ),
            ...Array.from({ length: 24 }, (_5, hour) => {
              const cell = lookup.get(`${dow},${hour}`);
              const cost = cell?.cost_nanos ?? 0;
              const turns = cell?.turns ?? 0;
              const opacity = cellOpacity3(cost, maxCost);
              const bg = withAlpha("--text-primary", opacity);
              const costUsd = cost / 1e9;
              const title = `${DOW_LABELS[dow]} ${String(hour).padStart(2, "0")}:00 \u2014 ${fmtCost(costUsd)} / ${fmt(turns)} turn${turns !== 1 ? "s" : ""} (90d avg)`;
              return /* @__PURE__ */ u4(
                "div",
                {
                  class: "days-hours-heatmap-cell",
                  title,
                  role: "img",
                  "aria-label": title,
                  style: { background: bg }
                },
                `${dow}-${hour}`
              );
            })
          ])
        ]
      }
    );
  }

  // src/ui/components/charts/ActivityHeatmap.tsx
  var DOW_LABELS2 = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  var METRIC_LABELS = {
    cost: "Cost",
    calls: "Calls"
  };
  var LEGEND_STEPS = [0.05, 0.2, 0.4, 0.6, 0.9];
  function cellOpacity4(value, max2) {
    if (max2 <= 0 || value <= 0) return 0.05;
    const ratio = value / max2;
    return Math.min(0.05 + 0.85 * ratio, 0.9);
  }
  function formatPeak(value, metric) {
    if (metric === "cost") {
      if (value >= 1e3) return fmtCostCompact(value);
      return fmtCostBig(value);
    }
    return fmt(value);
  }
  function ActivityHeatmap({ data, metric, onMetricChange }) {
    const {
      cells,
      max_cost_nanos,
      max_call_count,
      active_days,
      total_cost_nanos,
      period,
      tz_offset_min
    } = data;
    const lookup = /* @__PURE__ */ new Map();
    for (const c4 of cells) lookup.set(`${c4.dow},${c4.hour}`, c4);
    const avgPerDayUsd = active_days > 0 ? total_cost_nanos / 1e9 / active_days : 0;
    const avgPerDay = active_days > 0 ? fmtCostBig(avgPerDayUsd) : "\u2014";
    const metricMaxRaw = metric === "cost" ? max_cost_nanos : max_call_count;
    const metricMaxDisplay = metric === "cost" ? metricMaxRaw / 1e9 : metricMaxRaw;
    let peakKey = null;
    let peakVal = 0;
    for (const c4 of cells) {
      const v4 = metric === "cost" ? c4.cost_nanos : c4.call_count;
      if (v4 > peakVal) {
        peakVal = v4;
        peakKey = `${c4.dow},${c4.hour}`;
      }
    }
    return /* @__PURE__ */ u4("div", { class: "heatmap-panel", children: [
      /* @__PURE__ */ u4("div", { class: "heatmap-header", children: [
        /* @__PURE__ */ u4("span", { class: "heatmap-title", children: [
          "ACTIVITY / 7x24 / ",
          period.toUpperCase()
        ] }),
        /* @__PURE__ */ u4("span", { class: "heatmap-subtitle", children: [
          active_days,
          " active ",
          active_days === 1 ? "day" : "days",
          " \xB7 ",
          avgPerDay,
          " per active day",
          " \xB7 ",
          fmtTzOffset(tz_offset_min)
        ] }),
        /* @__PURE__ */ u4("div", { class: "range-group heatmap-metric", "aria-label": "Heatmap metric", children: Object.keys(METRIC_LABELS).map((m5) => /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: `range-btn${metric === m5 ? " active" : ""}`,
            "aria-pressed": metric === m5,
            onClick: () => onMetricChange(m5),
            children: METRIC_LABELS[m5]
          },
          m5
        )) })
      ] }),
      /* @__PURE__ */ u4(
        "div",
        {
          class: "heatmap-grid",
          role: "figure",
          "aria-label": "Activity heatmap: 7 days by 24 hours",
          children: [
            /* @__PURE__ */ u4("div", {}),
            /* @__PURE__ */ u4("div", { class: "heatmap-hour-labels", "aria-hidden": "true", children: [
              /* @__PURE__ */ u4("span", { children: "00" }),
              /* @__PURE__ */ u4("span", { children: "06" }),
              /* @__PURE__ */ u4("span", { children: "12" }),
              /* @__PURE__ */ u4("span", { children: "18" })
            ] }),
            Array.from({ length: 7 }, (_4, dow) => {
              const isWeekend = dow === 0 || dow === 6;
              return [
                /* @__PURE__ */ u4("div", { class: "heatmap-dow-label", children: DOW_LABELS2[dow] }, `label-${dow}`),
                ...Array.from({ length: 24 }, (_5, hour) => {
                  const key = `${dow},${hour}`;
                  const cell = lookup.get(key);
                  const costNanos = cell?.cost_nanos ?? 0;
                  const callCount = cell?.call_count ?? 0;
                  const raw = metric === "cost" ? costNanos : callCount;
                  const opacity = cellOpacity4(raw, metricMaxRaw);
                  const bg = withAlpha("--text-display", opacity);
                  const costUsd = costNanos / 1e9;
                  const title = `${DOW_LABELS2[dow]} ${String(hour).padStart(2, "0")}:00 \u2014 ${fmtCost(costUsd)} / ${callCount} call${callCount !== 1 ? "s" : ""}`;
                  const isPeak = key === peakKey && peakVal > 0;
                  const classes = [
                    "heatmap-cell",
                    isWeekend ? "heatmap-cell--weekend" : "",
                    isPeak ? "heatmap-cell--peak" : ""
                  ].filter(Boolean).join(" ");
                  return /* @__PURE__ */ u4(
                    "div",
                    {
                      role: "img",
                      "aria-label": title,
                      title,
                      class: classes,
                      style: { background: bg }
                    },
                    key
                  );
                })
              ];
            })
          ]
        }
      ),
      /* @__PURE__ */ u4("div", { class: "heatmap-legend", "aria-hidden": "true", children: [
        /* @__PURE__ */ u4("span", { children: "Less" }),
        /* @__PURE__ */ u4("div", { class: "heatmap-legend-track", children: LEGEND_STEPS.map((op, i4) => /* @__PURE__ */ u4(
          "div",
          {
            class: "heatmap-legend-step",
            style: { background: withAlpha("--text-display", op) }
          },
          i4
        )) }),
        /* @__PURE__ */ u4("span", { children: [
          "More",
          peakVal > 0 ? ` (peak ${formatPeak(metricMaxDisplay, metric)})` : ""
        ] })
      ] })
    ] });
  }

  // src/ui/components/agents/AgentDistribution.tsx
  var columns = [
    {
      accessorKey: "role",
      header: "ROLE",
      cell: ({ row }) => {
        const agg = row.original;
        const display = agg.display_name ?? agg.role;
        return /* @__PURE__ */ u4("span", { title: agg.role, children: esc(display) });
      }
    },
    {
      accessorKey: "sessions",
      header: "SESSIONS",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: Number(getValue() ?? 0).toLocaleString() })
    },
    {
      accessorKey: "total_tokens",
      header: "TOKENS",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
    },
    {
      accessorKey: "cost_usd",
      header: "COST",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmtCostBig(getValue()) })
    },
    {
      accessorKey: "tool_uses",
      header: "TOOL USES",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: Number(getValue() ?? 0).toLocaleString() })
    }
  ];
  function AgentDistribution({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u4(
      DataTable,
      {
        columns,
        data,
        title: "Role distribution",
        sectionKey: "agent-distribution",
        defaultSort: [{ id: "cost_usd", desc: true }]
      }
    );
  }

  // src/ui/components/agents/AgentKpis.tsx
  function fmtDurationTotal(seconds) {
    if (seconds <= 0) return "0s";
    if (seconds < 60) return `${Math.round(seconds)}s`;
    if (seconds < 3600) {
      const m6 = Math.floor(seconds / 60);
      const s4 = Math.round(seconds % 60);
      return s4 ? `${m6}m ${s4}s` : `${m6}m`;
    }
    const h5 = Math.floor(seconds / 3600);
    const m5 = Math.round(seconds % 3600 / 60);
    return m5 ? `${h5}h ${m5}m` : `${h5}h`;
  }
  function AgentKpis({ telemetry, totalCostUsd }) {
    const { totals } = telemetry;
    if (totals.sessions === 0) {
      return /* @__PURE__ */ u4("div", { class: "table-card", style: { padding: "20px" }, children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", style: { marginBottom: 0 }, children: "Agent delegation" }),
        /* @__PURE__ */ u4("div", { class: "empty-state", children: "No agent activity yet" })
      ] });
    }
    const delegationPct = totalCostUsd > 0 ? (totals.cost_usd / totalCostUsd * 100).toFixed(1) : "0.0";
    const tokensPerSession = totals.sessions > 0 ? Math.round(totals.total_tokens / totals.sessions) : 0;
    const costPerSession = totals.sessions > 0 ? totals.cost_usd / totals.sessions : 0;
    const cards = [
      {
        label: "Agent delegation",
        value: `${delegationPct}%`,
        sub: `${fmtCostBig(totals.cost_usd)} agent cost`
      },
      {
        label: "Agent sessions",
        value: totals.sessions.toLocaleString(),
        sub: `${fmt(totals.total_tokens)} total tokens`
      },
      {
        label: "Tokens / session",
        value: totals.sessions > 0 ? fmt(tokensPerSession) : "\u2014",
        sub: totals.sessions > 0 ? `${fmtCostBig(costPerSession)} avg cost` : "no sessions"
      },
      {
        label: "Tool calls",
        value: fmt(totals.tool_uses),
        sub: `${fmtDurationTotal(totals.duration_s)} total runtime`
      }
    ];
    return /* @__PURE__ */ u4(S, { children: cards.map((c4) => /* @__PURE__ */ u4("div", { class: "card stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
      /* @__PURE__ */ u4("div", { class: "stat-label", children: c4.label }),
      /* @__PURE__ */ u4("div", { class: "stat-value", children: c4.value }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", children: c4.sub })
    ] }) }, c4.label)) });
  }

  // src/ui/components/agents/AgentSetupBanner.tsx
  var unclassifiedGlobal = y3(null);
  var inFlight = false;
  async function fetchUnclassifiedGlobal() {
    if (inFlight || unclassifiedGlobal.value !== null) return;
    inFlight = true;
    try {
      const res = await fetch("/api/agents/unclassified-global");
      if (!res.ok) return;
      unclassifiedGlobal.value = await res.json();
    } finally {
      inFlight = false;
    }
  }
  function AgentSetupBanner({ telemetry }) {
    y2(() => {
      void fetchUnclassifiedGlobal();
    }, []);
    if (setupBannerDismissed.value) return null;
    const server = unclassifiedGlobal.value;
    if (!server || server.count <= 0) return null;
    const roleCount = server.count;
    const projects = [
      ...new Set(unclassifiedDetectedRolesGlobal(telemetry).map((d5) => d5.project))
    ];
    const projectCount = projects.length || 1;
    function openRegistry() {
      const firstProject = projects[0];
      if (firstProject) {
        registryModalOpen.value = { project: firstProject };
      }
    }
    function dismiss() {
      setupBannerDismissed.value = true;
    }
    return /* @__PURE__ */ u4("div", { class: "agent-setup-banner", children: [
      /* @__PURE__ */ u4("div", { class: "agent-setup-banner-body", children: [
        /* @__PURE__ */ u4("div", { class: "agent-setup-banner-line1", children: [
          roleCount,
          " unclassified agent role",
          roleCount !== 1 ? "s" : "",
          " detected",
          " ",
          "across ",
          projectCount,
          " project",
          projectCount !== 1 ? "s" : ""
        ] }),
        /* @__PURE__ */ u4("div", { class: "agent-setup-banner-line2", children: [
          "Classify them in the registry to see them in distribution and timeline.",
          " ",
          /* @__PURE__ */ u4(
            "button",
            {
              type: "button",
              class: "agent-setup-banner-action",
              onClick: openRegistry,
              children: "[Open registry]"
            }
          )
        ] })
      ] }),
      /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: "agent-setup-banner-dismiss",
          "aria-label": "Dismiss",
          onClick: dismiss,
          children: "[X]"
        }
      )
    ] });
  }

  // src/ui/components/agents/AgentSpawnBatches.tsx
  var columns2 = [
    {
      accessorKey: "spawned_at",
      header: "SPAWNED",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", title: String(getValue() ?? ""), children: fmtRelativeTime(String(getValue() ?? "")) })
    },
    {
      accessorKey: "project",
      header: "PROJECT",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { children: esc(String(getValue() ?? "")) })
    },
    {
      accessorKey: "size",
      header: "SIZE",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: Number(getValue() ?? 0) })
    },
    {
      accessorKey: "roles",
      header: "ROLES",
      cell: ({ getValue }) => {
        const roles = getValue();
        const sorted = [...roles].sort();
        const joined = sorted.join(", ");
        const truncated = joined.length > 60 ? joined.slice(0, 60) + "\u2026" : joined;
        return /* @__PURE__ */ u4("span", { title: joined, children: esc(truncated) });
      }
    },
    {
      accessorKey: "total_tokens",
      header: "TOKENS",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
    },
    {
      accessorKey: "cost_usd",
      header: "COST",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmtCostBig(getValue()) })
    }
  ];
  function AgentSpawnBatches({ data, summary }) {
    if (!data.length) return null;
    const avg = summary.avg_size > 0 ? summary.avg_size.toFixed(1) : "0";
    return /* @__PURE__ */ u4(S, { children: [
      /* @__PURE__ */ u4(
        "div",
        {
          class: "num",
          style: {
            color: "var(--text-secondary)",
            fontSize: "12px",
            padding: "0 4px 8px"
          },
          children: [
            summary.batch_count,
            " batches \xB7 avg ",
            avg,
            " \xB7 max ",
            summary.max_size,
            " \xB7",
            " ",
            summary.batched_agents,
            " agents"
          ]
        }
      ),
      /* @__PURE__ */ u4(
        DataTable,
        {
          columns: columns2,
          data,
          title: "Parallel spawn batches",
          sectionKey: "agent-spawn-batches",
          defaultSort: [{ id: "spawned_at", desc: true }]
        }
      )
    ] });
  }

  // src/ui/components/charts/ApexChart.tsx
  function ApexChart({ options, id }) {
    const ref = A2(null);
    const chartRef = A2(null);
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
        const apexCharts = window.ApexCharts;
        if (!apexCharts) return;
        const chartCfg = options.chart;
        const isSparkline = chartCfg?.sparkline?.enabled === true;
        let opts;
        if (isSparkline) {
          opts = options;
        } else {
          const parent = ref.current.parentElement;
          let h5 = parent?.clientHeight ?? 0;
          if (h5 <= 0) {
            const tokenName = parent?.classList.contains("tall") ? "--chart-h-lg" : "--chart-h-md";
            const resolved = getComputedStyle(document.documentElement).getPropertyValue(tokenName).trim();
            const parsed = parseFloat(resolved);
            h5 = Number.isFinite(parsed) && parsed > 0 ? parsed : 240;
          }
          opts = { ...options, chart: { ...options.chart, height: h5 } };
        }
        chartRef.current = new apexCharts(ref.current, opts);
        void chartRef.current.render();
      });
      return () => {
        cancelled = true;
        cancelAnimationFrame(raf);
        chartRef.current?.destroy();
        chartRef.current = null;
      };
    }, [id, options]);
    return /* @__PURE__ */ u4("div", { ref, id, style: { width: "100%", height: "100%" } });
  }

  // src/ui/components/agents/AgentTimeline.tsx
  var MAX_ROLES = 8;
  function AgentTimeline({ timeline }) {
    const options = T2(() => {
      if (!timeline.length) return null;
      const roleCost = /* @__PURE__ */ new Map();
      for (const pt of timeline) {
        roleCost.set(pt.role, (roleCost.get(pt.role) ?? 0) + pt.cost_usd);
      }
      const sortedRoles = [...roleCost.entries()].sort((a4, b4) => b4[1] - a4[1]).map(([r4]) => r4);
      const topRoles = sortedRoles.slice(0, MAX_ROLES);
      const hasOther = sortedRoles.length > MAX_ROLES;
      const buckets = [...new Set(timeline.map((pt) => pt.bucket))].sort();
      const seriesRoles = hasOther ? [...topRoles, "Other"] : topRoles;
      const seriesData = {};
      for (const role of seriesRoles) seriesData[role] = new Array(buckets.length).fill(0);
      for (const pt of timeline) {
        const bucketIdx = buckets.indexOf(pt.bucket);
        if (bucketIdx < 0) continue;
        if (topRoles.includes(pt.role)) {
          const arr = seriesData[pt.role];
          if (arr) arr[bucketIdx] = (arr[bucketIdx] ?? 0) + pt.cost_usd;
        } else if (hasOther) {
          const arr = seriesData["Other"];
          if (arr) arr[bucketIdx] = (arr[bucketIdx] ?? 0) + pt.cost_usd;
        }
      }
      const opacityLadder = [1, 0.7, 0.5, 0.35, 0.25, 0.18, 0.12, 0.08, 0.06];
      const colors = seriesRoles.map(
        (_4, i4) => withAlpha("--text-display", opacityLadder[Math.min(i4, opacityLadder.length - 1)] ?? 0.06)
      );
      const series = seriesRoles.map((role) => ({
        name: role,
        data: seriesData[role]
      }));
      const base = dashboardChartOptions("bar");
      return {
        ...base,
        chart: {
          ...base.chart,
          type: "bar",
          stacked: true
        },
        series,
        colors,
        xaxis: {
          ...base.xaxis,
          categories: buckets,
          tickAmount: Math.min(buckets.length, 10),
          labels: {
            ...base.xaxis?.labels,
            rotate: -30,
            style: {
              ...base.xaxis?.labels?.style,
              fontSize: "9px"
            }
          }
        },
        yaxis: {
          ...base.yaxis,
          labels: {
            ...base.yaxis?.labels,
            formatter: (v4) => fmtCostBig(v4)
          }
        },
        tooltip: {
          ...base.tooltip,
          y: {
            formatter: (v4) => fmtCostBig(v4)
          }
        },
        plotOptions: {
          bar: { horizontal: false, columnWidth: "60%" }
        }
      };
    }, [timeline]);
    if (!timeline.length) {
      return /* @__PURE__ */ u4("div", { class: "table-card agent-timeline-wrap", style: { padding: "20px" }, children: [
        /* @__PURE__ */ u4("div", { class: "section-title", children: "Agent activity" }),
        /* @__PURE__ */ u4("div", { class: "empty-state", children: "No timeline data" })
      ] });
    }
    return /* @__PURE__ */ u4("div", { class: "table-card agent-timeline-wrap", children: [
      /* @__PURE__ */ u4("div", { class: "section-header", style: { padding: "20px 20px 0" }, children: /* @__PURE__ */ u4("div", { class: "section-title", style: { padding: 0 }, children: "Agent activity" }) }),
      /* @__PURE__ */ u4("div", { class: "chart-wrap tall", style: { padding: "0 12px 12px" }, children: options && /* @__PURE__ */ u4(ApexChart, { options, id: "agent-timeline-chart" }) })
    ] });
  }

  // src/ui/components/agents/AgentToolSpectrum.tsx
  var MAX_TOOLS = 12;
  function AgentToolSpectrum({ data }) {
    if (!data.length) {
      return /* @__PURE__ */ u4("div", { class: "table-card", style: { padding: "20px" }, children: [
        /* @__PURE__ */ u4("div", { class: "section-title", children: "Tool spectrum" }),
        /* @__PURE__ */ u4("div", { class: "empty-state", children: "No tool data" })
      ] });
    }
    const toolTotals = /* @__PURE__ */ new Map();
    for (const c4 of data) {
      toolTotals.set(c4.tool, (toolTotals.get(c4.tool) ?? 0) + c4.count);
    }
    const sortedTools = [...toolTotals.entries()].sort((a4, b4) => b4[1] - a4[1]).map(([t4]) => t4);
    const topTools = sortedTools.slice(0, MAX_TOOLS);
    const hasOtherTool = sortedTools.length > MAX_TOOLS;
    const displayTools = hasOtherTool ? [...topTools, "Other"] : topTools;
    const roleTotals = /* @__PURE__ */ new Map();
    for (const c4 of data) roleTotals.set(c4.role, (roleTotals.get(c4.role) ?? 0) + c4.count);
    const roles = [...roleTotals.entries()].sort((a4, b4) => b4[1] - a4[1]).map(([r4]) => r4);
    const cellMap = /* @__PURE__ */ new Map();
    for (const c4 of data) {
      if (!cellMap.has(c4.role)) cellMap.set(c4.role, /* @__PURE__ */ new Map());
      const toolKey = topTools.includes(c4.tool) ? c4.tool : hasOtherTool ? "Other" : null;
      if (!toolKey) continue;
      const row = cellMap.get(c4.role);
      row.set(toolKey, (row.get(toolKey) ?? 0) + c4.count);
    }
    function rowMax(role) {
      const row = cellMap.get(role);
      if (!row) return 0;
      return Math.max(...row.values(), 0);
    }
    const gridCols = displayTools.length;
    return /* @__PURE__ */ u4("div", { class: "table-card", style: { padding: "20px" }, children: [
      /* @__PURE__ */ u4("div", { class: "section-title", style: { marginBottom: "16px" }, children: "Tool spectrum" }),
      /* @__PURE__ */ u4(
        "div",
        {
          class: "agent-tool-spectrum",
          style: {
            gridTemplateColumns: `minmax(80px, 160px) repeat(${gridCols}, minmax(0, 1fr))`
          },
          children: [
            /* @__PURE__ */ u4("div", { class: "spectrum-cell spectrum-header-corner" }),
            displayTools.map((tool) => /* @__PURE__ */ u4("div", { class: "spectrum-cell spectrum-col-header", title: tool, children: esc(tool.length > 14 ? tool.slice(0, 12) + "\u2026" : tool) }, tool)),
            roles.map((role) => {
              const maxVal = rowMax(role);
              const row = cellMap.get(role);
              return [
                /* @__PURE__ */ u4("div", { class: "spectrum-cell spectrum-row-label", title: role, children: esc(role.length > 18 ? role.slice(0, 16) + "\u2026" : role) }, `label-${role}`),
                ...displayTools.map((tool) => {
                  const count2 = row?.get(tool) ?? 0;
                  const opacity = maxVal > 0 && count2 > 0 ? Math.min(0.08 + 0.82 * (count2 / maxVal), 0.9) : 0;
                  const bg = opacity > 0 ? withAlpha("--text-primary", opacity) : "transparent";
                  const textColor = opacity > 0.5 ? "var(--black)" : opacity > 0 ? "var(--text-primary)" : "var(--text-disabled)";
                  return /* @__PURE__ */ u4(
                    "div",
                    {
                      class: "spectrum-cell spectrum-data-cell",
                      title: `${role} / ${tool}: ${count2}`,
                      style: { background: bg },
                      children: /* @__PURE__ */ u4("span", { style: { color: textColor }, children: count2 > 0 ? count2 : "" })
                    },
                    `${role}-${tool}`
                  );
                })
              ];
            })
          ]
        }
      )
    ] });
  }

  // src/ui/components/agents/AgentTopSessions.tsx
  function fmtDuration(seconds) {
    if (seconds < 60) return `${Math.round(seconds)}s`;
    const m5 = Math.floor(seconds / 60);
    const s4 = Math.round(seconds % 60);
    return `${m5}m ${s4}s`;
  }
  function StopReasonBadge({ reason }) {
    if (!reason) return /* @__PURE__ */ u4("span", { class: "num", style: { color: "var(--text-disabled)" }, children: "\u2014" });
    let cls = "agent-stop-reason-badge";
    if (reason === "end_turn") cls += " agent-stop-reason--success";
    else if (reason === "max_tokens") cls += " agent-stop-reason--warning";
    else cls += " agent-stop-reason--error";
    return /* @__PURE__ */ u4("span", { class: cls, children: esc(reason) });
  }
  var columns3 = [
    {
      accessorKey: "ts_start",
      header: "STARTED",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", title: String(getValue() ?? ""), children: fmtRelativeTime(String(getValue() ?? "")) })
    },
    {
      accessorKey: "role",
      header: "ROLE",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { children: esc(String(getValue() ?? "")) })
    },
    {
      accessorKey: "description",
      header: "DESCRIPTION",
      cell: ({ getValue }) => {
        const raw = String(getValue() ?? "");
        const truncated = raw.length > 60 ? raw.slice(0, 60) + "\u2026" : raw;
        return /* @__PURE__ */ u4("span", { title: raw, children: esc(truncated) });
      }
    },
    {
      accessorKey: "model",
      header: "MODEL",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: esc(String(getValue() ?? "")) })
    },
    {
      accessorKey: "duration_s",
      header: "DURATION",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmtDuration(getValue()) })
    },
    {
      accessorKey: "total_tokens",
      header: "TOKENS",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
    },
    {
      accessorKey: "cost_usd",
      header: "COST",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmtCostBig(getValue()) })
    },
    {
      accessorKey: "stop_reason",
      header: "STOP",
      cell: ({ getValue }) => /* @__PURE__ */ u4(StopReasonBadge, { reason: getValue() })
    }
  ];
  function AgentTopSessions({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u4(
      DataTable,
      {
        columns: columns3,
        data: data.slice(0, 25),
        title: "Top agent sessions",
        sectionKey: "agent-top-sessions",
        defaultSort: [{ id: "cost_usd", desc: true }]
      }
    );
  }

  // src/ui/components/AgentStatusCard.tsx
  var fmtUtc = (ts) => ts.slice(0, 19).replace("T", " ");
  function IndicatorDot({ indicator }) {
    const isAlert = indicator === "major" || indicator === "critical";
    const isMinor = indicator === "minor";
    return /* @__PURE__ */ u4(
      "span",
      {
        "aria-hidden": "true",
        style: {
          display: "inline-block",
          width: "8px",
          height: "8px",
          borderRadius: "50%",
          flexShrink: 0,
          backgroundColor: isAlert ? "var(--accent)" : "var(--text-secondary)",
          opacity: isAlert ? 1 : isMinor ? 0.5 : 0.25
        }
      }
    );
  }
  function ProviderRow({ name, status, expanded, isLast }) {
    const indicator = status?.indicator ?? "none";
    const isAlert = indicator === "major" || indicator === "critical";
    return /* @__PURE__ */ u4(
      "div",
      {
        style: {
          borderBottom: isLast ? "none" : "1px solid var(--border)",
          paddingBottom: isLast ? 0 : "10px",
          marginBottom: isLast ? 0 : "10px"
        },
        children: [
          /* @__PURE__ */ u4("div", { style: { display: "flex", alignItems: "center", gap: "10px" }, children: [
            /* @__PURE__ */ u4(IndicatorDot, { indicator }),
            /* @__PURE__ */ u4(
              "span",
              {
                style: {
                  fontFamily: "var(--font-mono)",
                  fontSize: "13px",
                  fontWeight: 500,
                  flex: 1,
                  color: "var(--text-primary)"
                },
                children: name
              }
            ),
            !status ? /* @__PURE__ */ u4(
              "span",
              {
                style: {
                  fontFamily: "var(--font-mono)",
                  fontSize: "11px",
                  color: "var(--text-secondary)",
                  opacity: 0.6
                },
                title: "Status API unreachable",
                children: "unavailable"
              }
            ) : /* @__PURE__ */ u4(S, { children: [
              /* @__PURE__ */ u4(
                "span",
                {
                  style: {
                    fontFamily: "var(--font-mono)",
                    fontSize: "11px",
                    color: isAlert ? "var(--accent)" : "var(--text-secondary)"
                  },
                  children: [
                    status.description,
                    status.active_incidents.length > 0 && /* @__PURE__ */ u4("span", { style: { opacity: 0.7 }, children: [
                      " ",
                      "(",
                      status.active_incidents.length,
                      ")"
                    ] })
                  ]
                }
              ),
              /* @__PURE__ */ u4(
                "a",
                {
                  href: status.page_url,
                  target: "_blank",
                  rel: "noopener noreferrer",
                  style: {
                    color: "var(--text-secondary)",
                    fontSize: "11px",
                    opacity: 0.5,
                    display: "inline-flex",
                    alignItems: "center",
                    lineHeight: 1,
                    flexShrink: 0,
                    textDecoration: "none"
                  },
                  "aria-label": `${name} status page`,
                  children: "\u2197"
                }
              )
            ] })
          ] }),
          expanded && status && /* @__PURE__ */ u4("div", { style: { paddingLeft: "18px", paddingTop: "8px" }, children: [
            status.components.length > 0 && /* @__PURE__ */ u4("table", { style: { width: "100%", fontSize: "12px", borderCollapse: "collapse", marginBottom: "8px" }, children: [
              /* @__PURE__ */ u4("thead", { children: /* @__PURE__ */ u4("tr", { style: { color: "var(--text-secondary)" }, children: [
                /* @__PURE__ */ u4("th", { style: { textAlign: "left", padding: "2px 8px 2px 0", fontWeight: 500 }, children: "Component" }),
                /* @__PURE__ */ u4("th", { style: { textAlign: "left", padding: "2px 0", fontWeight: 500 }, children: "Status" })
              ] }) }),
              /* @__PURE__ */ u4("tbody", { children: status.components.map((c4, i4) => {
                const fmt2 = (v4) => v4 != null ? `${(v4 * 100).toFixed(2)}%` : "--";
                const showUptime = c4.uptime_30d != null || c4.uptime_7d != null;
                return /* @__PURE__ */ u4(S, { children: [
                  /* @__PURE__ */ u4("tr", { children: [
                    /* @__PURE__ */ u4("td", { style: { padding: "2px 8px 2px 0", fontFamily: "var(--font-mono)" }, children: c4.name }),
                    /* @__PURE__ */ u4("td", { style: { padding: "2px 0", color: "var(--text-secondary)" }, children: c4.status.replace(/_/g, " ") })
                  ] }, i4),
                  showUptime && /* @__PURE__ */ u4("tr", { children: /* @__PURE__ */ u4("td", { colSpan: 2, style: { padding: "0 0 4px 0" }, children: /* @__PURE__ */ u4("span", { style: { fontFamily: "var(--font-mono)", fontSize: "11px", letterSpacing: "0.04em" }, children: [
                    /* @__PURE__ */ u4("span", { style: { color: "var(--text-secondary)" }, children: "30D " }),
                    /* @__PURE__ */ u4("span", { style: { color: "var(--text-primary)" }, children: fmt2(c4.uptime_30d) }),
                    /* @__PURE__ */ u4("span", { style: { color: "var(--text-secondary)" }, children: " \xB7 7D " }),
                    /* @__PURE__ */ u4("span", { style: { color: "var(--text-primary)" }, children: fmt2(c4.uptime_7d) })
                  ] }) }) }, `${i4}-uptime`)
                ] });
              }) })
            ] }),
            status.active_incidents.map((inc, i4) => /* @__PURE__ */ u4(
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
                  /* @__PURE__ */ u4("span", { style: { fontFamily: "var(--font-mono)" }, children: inc.shortlink ? /* @__PURE__ */ u4(
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
                  /* @__PURE__ */ u4("span", { style: { opacity: 0.7 }, children: [
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
        ]
      }
    );
  }
  function signalLevelStyle(level) {
    switch (level) {
      case "spike":
        return { label: "[Spike]", color: "var(--accent)" };
      case "elevated":
        return { label: "[Elevated]", color: "var(--text-primary)" };
      case "normal":
        return { label: "[Normal]", color: "var(--text-secondary)" };
      default:
        return { label: "[Unknown]", color: "var(--text-secondary)" };
    }
  }
  function CommunitySignalRow({ label, signals }) {
    const first = signals[0];
    if (!first) return null;
    const levelOrder = ["spike", "elevated", "normal", "unknown"];
    const worstLevel = levelOrder.find((l5) => signals.some((s4) => s4.level === l5)) ?? "unknown";
    const { label: levelLabel, color } = signalLevelStyle(worstLevel);
    return /* @__PURE__ */ u4("div", { style: { display: "flex", alignItems: "center", padding: "4px 0", gap: "8px" }, children: [
      /* @__PURE__ */ u4("span", { style: { fontFamily: "var(--font-mono)", fontSize: "13px", flex: 1 }, children: label }),
      /* @__PURE__ */ u4("span", { style: { fontFamily: "var(--font-mono)", fontSize: "12px", color }, children: levelLabel }),
      /* @__PURE__ */ u4(
        "a",
        {
          href: first.source_url,
          target: "_blank",
          rel: "noopener noreferrer",
          style: { color: "var(--text-secondary)", fontSize: "11px", opacity: 0.5 },
          "aria-label": `${label} community signal source`,
          children: "\u2197"
        }
      )
    ] });
  }
  function AgentStatusCard({ snapshot, communitySignal }) {
    const expanded = agent_status_expanded.value;
    const hasData = snapshot.claude != null || snapshot.openai != null;
    const hasCommunity = !!(communitySignal?.enabled && (communitySignal.claude.length > 0 || communitySignal.openai.length > 0));
    return /* @__PURE__ */ u4(
      "div",
      {
        class: "card",
        style: {
          minWidth: "300px",
          display: "flex",
          flexDirection: "column"
        },
        children: [
          /* @__PURE__ */ u4(
            "div",
            {
              style: {
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                marginBottom: "12px"
              },
              children: [
                /* @__PURE__ */ u4("div", { class: "stat-label", style: { margin: 0 }, children: "Agent status" }),
                hasData && /* @__PURE__ */ u4(
                  "button",
                  {
                    type: "button",
                    onClick: () => {
                      agent_status_expanded.value = !expanded;
                      syncDashboardUrl();
                    },
                    style: {
                      display: "inline-flex",
                      alignItems: "center",
                      gap: "4px",
                      background: "none",
                      border: "none",
                      cursor: "pointer",
                      color: "var(--text-secondary)",
                      fontSize: "11px",
                      fontFamily: "var(--font-mono)",
                      padding: "4px 0 4px 8px",
                      opacity: 0.7
                    },
                    "aria-expanded": expanded,
                    children: [
                      /* @__PURE__ */ u4("span", { "aria-hidden": "true", style: { fontSize: "9px" }, children: expanded ? "\u25B2" : "\u25BC" }),
                      /* @__PURE__ */ u4("span", { children: expanded ? "Collapse" : "Expand" })
                    ]
                  }
                )
              ]
            }
          ),
          /* @__PURE__ */ u4(ProviderRow, { name: "Claude", status: snapshot.claude, expanded, isLast: false }),
          /* @__PURE__ */ u4(ProviderRow, { name: "OpenAI / Codex", status: snapshot.openai, expanded, isLast: true }),
          hasCommunity && communitySignal && /* @__PURE__ */ u4("div", { style: { marginTop: "12px", borderTop: "1px solid var(--border)", paddingTop: "8px" }, children: [
            /* @__PURE__ */ u4(
              "div",
              {
                style: {
                  fontSize: "var(--font-size-tertiary)",
                  fontFamily: "var(--font-sans)",
                  color: "var(--text-secondary)",
                  marginBottom: "6px",
                  letterSpacing: 0
                },
                children: "Community signal"
              }
            ),
            /* @__PURE__ */ u4(CommunitySignalRow, { label: "Claude", signals: communitySignal.claude }),
            /* @__PURE__ */ u4(CommunitySignalRow, { label: "OpenAI", signals: communitySignal.openai }),
            communitySignal.fetched_at && /* @__PURE__ */ u4(
              "div",
              {
                style: {
                  fontSize: "10px",
                  color: "var(--text-secondary)",
                  marginTop: "4px",
                  fontFamily: "var(--font-mono)",
                  opacity: 0.6
                },
                children: [
                  "Crowd data ",
                  fmtUtc(communitySignal.fetched_at),
                  " UTC"
                ]
              }
            )
          ] }),
          snapshot.fetched_at && /* @__PURE__ */ u4(
            "div",
            {
              style: {
                marginTop: "auto",
                paddingTop: "12px",
                fontSize: "10px",
                color: "var(--text-secondary)",
                fontFamily: "var(--font-mono)",
                opacity: 0.6
              },
              children: [
                "Last checked ",
                fmtUtc(snapshot.fetched_at),
                " UTC"
              ]
            }
          )
        ]
      }
    );
  }

  // src/ui/components/shared/InlineRankBar.tsx
  function InlineRankBar({
    value,
    max: max2,
    label
  }) {
    const pct = max2 > 0 ? value / max2 * 100 : 0;
    const tooltip = `${value} (${pct.toFixed(1)}% of max ${max2})`;
    return /* @__PURE__ */ u4(
      "span",
      {
        style: { position: "relative", display: "inline-block", width: "100%" },
        title: tooltip,
        children: [
          /* @__PURE__ */ u4(
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
          /* @__PURE__ */ u4("span", { class: "num", style: { position: "relative", zIndex: 1 }, children: label })
        ]
      }
    );
  }

  // src/ui/components/tables/BranchTable.tsx
  function makeColumns(data) {
    const maxSessions = data.reduce((m5, r4) => Math.max(m5, r4.sessions), 0);
    return [
      {
        accessorKey: "provider",
        header: "Provider",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
      },
      {
        accessorKey: "branch",
        header: "Branch",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(getValue()) })
      },
      {
        accessorKey: "sessions",
        header: "Sessions",
        cell: ({ getValue }) => /* @__PURE__ */ u4(
          InlineRankBar,
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
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "input",
        header: "Input",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "output",
        header: "Output",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "cost",
        header: "Est. Cost",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "cost", children: fmtCost(getValue()) })
      }
    ];
  }
  function BranchTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u4(DataTable, { columns: makeColumns(data), data, title: "Usage by Git Branch", sectionKey: "branch-summary" });
  }

  // src/ui/components/SegmentedProgressBar.tsx
  function resolveStatus(pct, status) {
    if (status === "neutral") return "var(--accent-interactive)";
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
    segments: _segments,
    size = "standard",
    status = "auto",
    "aria-label": ariaLabel
  }) {
    const safeMax = max2 > 0 ? max2 : 1;
    const ratio = value / safeMax;
    const pct = Math.min(100, Math.max(0, ratio * 100));
    const overflow = ratio > 1;
    const fillColor = overflow ? "var(--accent)" : resolveStatus(pct, status);
    return /* @__PURE__ */ u4(
      "div",
      {
        class: `segmented-bar segmented-bar--${size}`,
        role: "progressbar",
        "aria-label": ariaLabel,
        "aria-valuenow": Math.round(pct),
        "aria-valuemin": 0,
        "aria-valuemax": 100,
        children: /* @__PURE__ */ u4(
          "div",
          {
            class: "segmented-bar__fill",
            style: { width: `${pct}%`, background: fillColor, minWidth: pct > 0 ? "8px" : "0" }
          }
        )
      }
    );
  }

  // src/ui/components/ClaudeUsagePanel.tsx
  function statusColor(status) {
    if (status === "failed") return "var(--accent)";
    if (status === "unparsed") return "var(--warning)";
    return "var(--text-secondary)";
  }
  function ClaudeUsagePanel({ data }) {
    const snapshot = data.latest_snapshot ?? null;
    const lastRun = data.last_run ?? null;
    const lastSuccess = snapshot?.run.captured_at ?? null;
    return /* @__PURE__ */ u4("div", { class: "card card-flat bento-full table-card", "aria-label": "Claude usage monitor", children: [
      /* @__PURE__ */ u4(
        "div",
        {
          style: {
            display: "flex",
            justifyContent: "space-between",
            gap: "16px",
            alignItems: "flex-start",
            flexWrap: "wrap",
            marginBottom: snapshot ? "18px" : "0"
          },
          children: [
            /* @__PURE__ */ u4("div", { children: [
              /* @__PURE__ */ u4("h2", { style: { marginBottom: "8px" }, children: "Claude /usage" }),
              /* @__PURE__ */ u4("div", { class: "stat-sub", children: [
                "Last success ",
                fmtRelativeTime(lastSuccess),
                lastRun ? ` \xB7 Last run ${fmtRelativeTime(lastRun.captured_at)}` : ""
              ] })
            ] }),
            lastRun && /* @__PURE__ */ u4(
              "div",
              {
                style: {
                  fontFamily: "var(--font-mono)",
                  fontSize: "12px",
                  letterSpacing: "0.08em",
                  textTransform: "uppercase",
                  color: statusColor(lastRun.status)
                },
                children: [
                  "[",
                  lastRun.status,
                  "]"
                ]
              }
            )
          ]
        }
      ),
      !snapshot && /* @__PURE__ */ u4("div", { class: "stat-sub", children: lastRun?.error_summary || "No parsed Claude /usage snapshot has been captured yet." }),
      snapshot && /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "14px" }, children: [
        snapshot.factors.map((factor) => /* @__PURE__ */ u4(
          "div",
          {
            style: {
              borderTop: "1px solid var(--border)",
              paddingTop: "14px"
            },
            children: [
              /* @__PURE__ */ u4(
                "div",
                {
                  style: {
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "baseline",
                    gap: "16px",
                    marginBottom: "10px",
                    flexWrap: "wrap"
                  },
                  children: [
                    /* @__PURE__ */ u4("div", { style: { fontWeight: 500 }, children: factor.display_label }),
                    /* @__PURE__ */ u4(
                      "div",
                      {
                        style: {
                          fontFamily: "var(--font-mono)",
                          fontSize: "14px",
                          whiteSpace: "nowrap"
                        },
                        children: [
                          factor.percent.toFixed(1),
                          "%"
                        ]
                      }
                    )
                  ]
                }
              ),
              /* @__PURE__ */ u4(
                SegmentedProgressBar,
                {
                  value: factor.percent,
                  max: 100,
                  size: "compact",
                  "aria-label": `${factor.display_label} percent`
                }
              ),
              factor.advice_text && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { marginTop: "10px" }, children: factor.advice_text })
            ]
          },
          factor.factor_key
        )),
        lastRun?.status !== "success" && lastRun?.error_summary && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { marginTop: "4px" }, children: [
          "Latest run note: ",
          lastRun.error_summary
        ] })
      ] })
    ] });
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
    return /* @__PURE__ */ u4("div", { class: "card card-flat", style: { gridColumn: "1 / -1" }, children: [
      /* @__PURE__ */ u4("div", { style: { marginBottom: "16px" }, children: [
        /* @__PURE__ */ u4("span", { style: {
          fontFamily: "var(--font-mono)",
          fontSize: "11px",
          fontWeight: 400,
          textTransform: "uppercase",
          letterSpacing: "0.08em",
          color: "var(--text-secondary)"
        }, children: "COST RECONCILIATION" }),
        data.period && /* @__PURE__ */ u4("span", { style: {
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
      /* @__PURE__ */ u4("div", { style: { display: "flex", gap: "32px", flexWrap: "wrap", marginBottom: "20px" }, children: [
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { style: {
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            textTransform: "uppercase",
            letterSpacing: "0.08em",
            color: "var(--text-secondary)",
            marginBottom: "8px"
          }, children: "HOOK-REPORTED" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "24px" }, children: fmtCost(hookUsd) })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { style: {
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            textTransform: "uppercase",
            letterSpacing: "0.08em",
            color: "var(--text-secondary)",
            marginBottom: "8px"
          }, children: "LOCAL ESTIMATE" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "24px" }, children: fmtCost(localUsd) })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { style: {
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            textTransform: "uppercase",
            letterSpacing: "0.08em",
            color: "var(--text-secondary)",
            marginBottom: "8px"
          }, children: "DIVERGENCE" }),
          /* @__PURE__ */ u4(
            "div",
            {
              class: "stat-value",
              style: { fontSize: "24px", color: isWarn ? "var(--accent)" : void 0 },
              children: [
                divergencePctSigned >= 0 ? "+" : "",
                divergencePctSigned.toFixed(1),
                "%",
                isWarn && /* @__PURE__ */ u4("span", { style: {
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
      data.breakdown && data.breakdown.length > 0 && /* @__PURE__ */ u4("div", { style: { overflowX: "auto" }, children: /* @__PURE__ */ u4("table", { children: [
        /* @__PURE__ */ u4("thead", { children: /* @__PURE__ */ u4("tr", { children: [
          /* @__PURE__ */ u4("th", { children: "DAY" }),
          /* @__PURE__ */ u4("th", { style: { textAlign: "right" }, children: "HOOK" }),
          /* @__PURE__ */ u4("th", { style: { textAlign: "right" }, children: "LOCAL" }),
          /* @__PURE__ */ u4("th", { style: { textAlign: "right" }, children: "\u0394" })
        ] }) }),
        /* @__PURE__ */ u4("tbody", { children: data.breakdown.slice().reverse().slice(0, 30).map((r4) => {
          const h5 = r4.hook_nanos / 1e9;
          const l5 = r4.local_nanos / 1e9;
          const delta = h5 - l5;
          const rowWarn = l5 > 1e-9 && Math.abs(delta) / l5 > 0.1;
          return /* @__PURE__ */ u4("tr", { children: [
            /* @__PURE__ */ u4("td", { class: "num", children: r4.day }),
            /* @__PURE__ */ u4("td", { class: "num", style: { textAlign: "right" }, children: fmtCost(h5) }),
            /* @__PURE__ */ u4("td", { class: "num", style: { textAlign: "right" }, children: fmtCost(l5) }),
            /* @__PURE__ */ u4(
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

  // src/ui/components/charts/DailyChart.tsx
  function DailyChart({ daily }) {
    const base = dashboardChartOptions("bar");
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
        ...base.xaxis ?? {},
        categories: daily.map((d5) => d5.day),
        labels: { ...base.xaxis?.labels ?? {}, rotate: -45, maxHeight: 60 },
        tickAmount: Math.min(daily.length, RANGE_TICKS[selectedRange.value])
      },
      yaxis: {
        ...base.yaxis ?? {},
        labels: { ...base.yaxis?.labels ?? {}, formatter: (v4) => fmt(v4) }
      },
      tooltip: { ...base.tooltip, y: { formatter: (v4) => fmt(v4) + " tokens" } }
    };
    return /* @__PURE__ */ u4(ApexChart, { options, id: "chart-daily" });
  }

  // src/ui/components/tables/EntrypointTable.tsx
  var columns4 = [
    {
      accessorKey: "provider",
      header: "Provider",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
    },
    {
      accessorKey: "entrypoint",
      header: "Entrypoint",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(getValue()) })
    },
    {
      accessorKey: "sessions",
      header: "Sessions",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: Number(getValue() ?? 0) })
    },
    {
      accessorKey: "turns",
      header: "Turns",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
    },
    {
      accessorKey: "input",
      header: "Input",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
    },
    {
      accessorKey: "output",
      header: "Output",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
    }
  ];
  function EntrypointTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u4(DataTable, { columns: columns4, data, title: "Usage by Entrypoint", sectionKey: "entrypoint-breakdown" });
  }

  // src/ui/components/_primitives/KpiCard.tsx
  var TONE_CLASS = {
    default: "",
    cost: "cost-value",
    success: "kpi-value--success",
    warning: "kpi-value--warning",
    accent: "kpi-value--accent",
    muted: "kpi-value--muted"
  };
  function KpiCard({
    label,
    value,
    sub,
    bar,
    valueTone = "default",
    size = "standard",
    actions,
    className
  }) {
    const cardClass = ["card", "stat-card", "kpi-card", `kpi-card--${size}`, className].filter(Boolean).join(" ");
    const valueClass = ["stat-value", TONE_CLASS[valueTone]].filter(Boolean).join(" ");
    return /* @__PURE__ */ u4("div", { class: cardClass, children: [
      /* @__PURE__ */ u4("div", { class: "stat-content", children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", children: label }),
        /* @__PURE__ */ u4("div", { class: valueClass, children: value }),
        bar && /* @__PURE__ */ u4("div", { class: "kpi-card__bar", children: /* @__PURE__ */ u4(
          SegmentedProgressBar,
          {
            value: bar.value,
            max: bar.max ?? 100,
            size: bar.size ?? "standard",
            status: bar.status ?? "auto",
            "aria-label": bar.ariaLabel ?? label
          }
        ) }),
        sub && /* @__PURE__ */ u4("div", { class: "stat-sub", children: sub })
      ] }),
      actions && /* @__PURE__ */ u4("div", { class: "kpi-card__actions", children: actions })
    ] });
  }

  // src/ui/components/EstimationMeta.tsx
  function formatPricingVersion(v4) {
    const m5 = v4.match(/^\d{4}-\d{2}-\d{2}/);
    return m5 ? m5[0] : v4;
  }
  function formatBreakdown(rows2) {
    return rows2.map(([key, value]) => `${fmtLabel(key)}: ${value.sessions.toLocaleString()}`).join(" \xB7 ");
  }
  function EstimationMeta({
    confidenceBreakdown,
    billingModeBreakdown,
    pricingVersions
  }) {
    const pricingValue = pricingVersions.length === 0 ? "n/a" : pricingVersions.length === 1 ? formatPricingVersion(pricingVersions[0] ?? "") : `mixed (${pricingVersions.length})`;
    return /* @__PURE__ */ u4(S, { children: [
      /* @__PURE__ */ u4(
        KpiCard,
        {
          size: "compact",
          label: "Cost confidence",
          value: confidenceBreakdown.length ? formatBreakdown(confidenceBreakdown) : "n/a",
          sub: "Session mix in current filter"
        }
      ),
      /* @__PURE__ */ u4(
        KpiCard,
        {
          size: "compact",
          label: "Billing mode",
          value: billingModeBreakdown.length ? formatBreakdown(billingModeBreakdown) : "n/a",
          sub: "Local estimate vs subscriber-included sessions"
        }
      ),
      /* @__PURE__ */ u4(
        KpiCard,
        {
          size: "compact",
          label: "Pricing snapshot",
          value: pricingValue,
          sub: "Stored per-session pricing metadata"
        }
      )
    ] });
  }

  // src/ui/components/charts/HourlyChart.tsx
  function HourlyChart({ data }) {
    if (!data.length) return null;
    const maxTurns = Math.max(...data.map((d5) => d5.turns), 1);
    return /* @__PURE__ */ u4("div", { style: { height: "100%", display: "flex", flexDirection: "column" }, children: [
      /* @__PURE__ */ u4("div", { class: "section-title", style: { padding: "0", marginBottom: "12px" }, children: "Activity by Hour of Day" }),
      /* @__PURE__ */ u4("div", { style: { display: "flex", alignItems: "flex-end", gap: "2px", flex: 1, minHeight: "60px" }, children: Array.from({ length: 24 }, (_4, h5) => {
        const row = data.find((d5) => d5.hour === h5);
        const turns = row?.turns ?? 0;
        const pct = turns / maxTurns * 100;
        const background = turns > 0 ? withAlpha("--text-display", 0.4 + pct / 100 * 0.6) : cssVar("--border");
        return /* @__PURE__ */ u4(
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
      /* @__PURE__ */ u4("div", { style: { display: "flex", gap: "2px", marginTop: "6px" }, children: Array.from({ length: 24 }, (_4, h5) => /* @__PURE__ */ u4(
        "span",
        {
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
      )) })
    ] });
  }

  // src/ui/components/tables/McpSummaryTable.tsx
  function makeColumns2(data) {
    const maxInvocations = data.reduce((m5, r4) => Math.max(m5, r4.invocations), 0);
    return [
      {
        accessorKey: "provider",
        header: "Provider",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
      },
      {
        accessorKey: "server",
        header: "MCP Server",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag mcp", children: String(getValue()) })
      },
      {
        accessorKey: "tools_used",
        header: "Tools",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: Number(getValue() ?? 0) })
      },
      {
        accessorKey: "invocations",
        header: "Calls",
        cell: ({ getValue }) => /* @__PURE__ */ u4(
          InlineRankBar,
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
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
      }
    ];
  }
  function McpSummaryTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u4(DataTable, { columns: makeColumns2(data), data, title: "MCP Server Usage", sectionKey: "mcp-summary" });
  }

  // src/ui/components/charts/MetricDonut.tsx
  var SLICE_OPACITY_LADDER = [1, 0.64, 0.46, 0.34, 0.24, 0.16];
  var TOP_N = 5;
  function formatShare(share) {
    if (share >= 99.5) return "100%";
    if (share >= 10) return `${share.toFixed(0)}%`;
    if (share >= 0.1) return `${share.toFixed(1)}%`;
    if (share > 0) return "<0.1%";
    return "0%";
  }
  function MetricDonut({
    rows: rows2,
    metric,
    metricOptions,
    metricLabel,
    metricValue,
    metricFormat,
    rowLabel,
    rowCost,
    rowCalls,
    rowTokens,
    id,
    centerKickerPrefix = "",
    onMetricChange,
    isMetricDisabled,
    showLegend = false,
    onSelectRow,
    onSliceClick,
    formatCost,
    formatCalls,
    formatTokens
  }) {
    if (!rows2.length) return null;
    const sorted = rows2.map((row) => ({ row, value: metricValue(row, metric) })).filter((entry) => entry.value > 0).sort((a4, b4) => b4.value - a4.value);
    if (!sorted.length) return null;
    const top = sorted.slice(0, TOP_N);
    const rest = sorted.slice(TOP_N);
    const total2 = sorted.reduce((sum2, entry) => sum2 + entry.value, 0);
    const donutRows = top.map((entry, index) => ({
      label: rowLabel(entry.row),
      value: entry.value,
      share: total2 > 0 ? entry.value / total2 * 100 : 0,
      cost: rowCost(entry.row),
      calls: rowCalls(entry.row),
      tokens: rowTokens(entry.row),
      color: withAlpha("--text-display", SLICE_OPACITY_LADDER[Math.min(index, SLICE_OPACITY_LADDER.length - 1)] ?? 0.16),
      isOther: false
    }));
    const otherValue = rest.reduce((sum2, entry) => sum2 + entry.value, 0);
    const hasOther = otherValue > 0;
    if (hasOther) {
      donutRows.push({
        label: `Other (${rest.length})`,
        value: otherValue,
        share: total2 > 0 ? otherValue / total2 * 100 : 0,
        cost: rest.reduce((sum2, entry) => sum2 + rowCost(entry.row), 0),
        calls: rest.reduce((sum2, entry) => sum2 + rowCalls(entry.row), 0),
        tokens: rest.reduce((sum2, entry) => sum2 + rowTokens(entry.row), 0),
        color: withAlpha("--text-display", SLICE_OPACITY_LADDER[Math.min(donutRows.length, SLICE_OPACITY_LADDER.length - 1)] ?? 0.16),
        isOther: true
      });
    }
    const base = dashboardChartOptions("donut");
    const options = {
      ...base,
      chart: {
        ...base.chart,
        type: "donut",
        ...onSliceClick ? {
          events: {
            dataPointSelection: (_event, _ctx, config) => {
              const row = donutRows[config.dataPointIndex];
              if (row && !row.isOther) onSliceClick(row.label);
            }
          }
        } : {}
      },
      series: donutRows.map((row) => row.value),
      labels: donutRows.map((row) => row.label),
      colors: donutRows.map((row) => row.color),
      stroke: { width: 2, colors: [cssVar("--surface")] },
      legend: { ...base.legend, show: false },
      states: {
        hover: { filter: { type: "none", value: 0 } },
        active: { filter: { type: "none", value: 0 } }
      },
      tooltip: {
        ...base.tooltip,
        custom: ({ seriesIndex }) => {
          const row = donutRows[seriesIndex];
          if (!row) return "";
          return `<div style="padding:8px 12px;font-family:var(--font-mono,'Geist Mono',ui-monospace,monospace);font-size:11px;line-height:1.6"><strong>${esc(row.label)}</strong><br/>${esc(metricLabel(metric))}: ${esc(metricFormat(row.value, metric))} (${esc(formatShare(row.share))} share)<br/>Cost: ${esc(formatCost(row.cost))} &nbsp;&bull;&nbsp; Calls: ${esc(formatCalls(row.calls))} &nbsp;&bull;&nbsp; Tokens: ${esc(formatTokens(row.tokens))}</div>`;
        }
      },
      plotOptions: {
        pie: {
          expandOnClick: false,
          donut: {
            size: "72%",
            labels: { show: false }
          }
        }
      }
    };
    const kicker = centerKickerPrefix ? `${centerKickerPrefix} ${metricLabel(metric)}` : metricLabel(metric);
    return /* @__PURE__ */ u4("div", { class: "model-chart-panel", children: [
      /* @__PURE__ */ u4("div", { class: "range-group", "aria-label": `${id} metric`, children: metricOptions.map((m5) => /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: `range-btn${metric === m5 ? " active" : ""}`,
          disabled: isMetricDisabled ? isMetricDisabled(m5) : false,
          "aria-pressed": metric === m5,
          onClick: () => onMetricChange?.(m5),
          children: metricLabel(m5)
        },
        m5
      )) }),
      /* @__PURE__ */ u4("div", { class: "model-chart-ring", children: [
        /* @__PURE__ */ u4(ApexChart, { options, id }),
        /* @__PURE__ */ u4("div", { class: "model-chart-center", "aria-hidden": "true", children: /* @__PURE__ */ u4("div", { class: "model-chart-center-inner", children: [
          /* @__PURE__ */ u4("div", { class: "model-chart-center-kicker", children: kicker }),
          /* @__PURE__ */ u4("div", { class: "model-chart-center-total", children: metricFormat(total2, metric, true) }),
          hasOther ? /* @__PURE__ */ u4("div", { class: "model-chart-center-meta", children: [
            "Top ",
            TOP_N,
            " + Other"
          ] }) : null
        ] }) })
      ] }),
      showLegend && /* @__PURE__ */ u4("div", { class: "model-share-list", children: donutRows.map((row) => /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: `model-share-row${onSelectRow && !row.isOther ? " interactive" : ""}`,
          onClick: onSelectRow && !row.isOther ? () => onSelectRow(row.label) : void 0,
          disabled: !onSelectRow || row.isOther,
          "aria-label": row.isOther ? `${row.label} ${metricLabel(metric)} summary` : `Filter to ${row.label}`,
          children: [
            /* @__PURE__ */ u4("div", { class: "model-share-row-head", children: [
              /* @__PURE__ */ u4("div", { class: "model-share-label", children: [
                /* @__PURE__ */ u4("span", { class: "model-share-swatch", style: { background: row.color }, "aria-hidden": "true" }),
                /* @__PURE__ */ u4("span", { title: row.label, children: row.label })
              ] }),
              /* @__PURE__ */ u4("div", { class: "model-share-value", children: metricFormat(row.value, metric) })
            ] }),
            /* @__PURE__ */ u4("div", { class: "model-share-row-meta", children: [
              /* @__PURE__ */ u4("div", { class: "model-share-bar", "aria-label": `${row.label} ${metricLabel(metric)} share`, children: /* @__PURE__ */ u4("div", { class: "model-share-bar-fill", style: { width: `${Math.min(100, row.share)}%`, background: row.color } }) }),
              /* @__PURE__ */ u4("div", { class: "model-share-percent", children: formatShare(row.share) })
            ] })
          ]
        },
        row.label
      )) })
    ] });
  }

  // src/ui/components/charts/ModelChart.tsx
  var METRIC_LABELS2 = {
    cost: "Cost",
    tokens: "Tokens",
    calls: "Calls"
  };
  var METRIC_OPTIONS = ["cost", "tokens", "calls"];
  function totalTokens(row) {
    return row.input + row.output + row.cache_read + row.cache_creation + row.reasoning_output;
  }
  function getMetricValue(row, metric) {
    switch (metric) {
      case "cost":
        return row.cost;
      case "tokens":
        return totalTokens(row);
      case "calls":
        return row.turns;
    }
  }
  function formatMetricValue(value, metric, large = false) {
    switch (metric) {
      case "cost":
        return large ? fmtCostCompact(value) : fmtCost(value);
      case "tokens":
      case "calls":
        return fmt(value);
    }
  }
  function ModelChart({
    byModel,
    onSelectModel
  }) {
    if (!byModel.length) return null;
    const [selectedMetric, setSelectedMetric] = d2("cost");
    const totals = {
      cost: byModel.reduce((sum2, row) => sum2 + row.cost, 0),
      tokens: byModel.reduce((sum2, row) => sum2 + totalTokens(row), 0),
      calls: byModel.reduce((sum2, row) => sum2 + row.turns, 0)
    };
    const enabledMetrics = METRIC_OPTIONS.filter((metric2) => totals[metric2] > 0);
    const metric = enabledMetrics.includes(selectedMetric) ? selectedMetric : enabledMetrics[0] ?? "cost";
    return MetricDonut({
      rows: byModel,
      metric,
      metricOptions: METRIC_OPTIONS,
      metricLabel: (m5) => METRIC_LABELS2[m5],
      metricValue: getMetricValue,
      metricFormat: formatMetricValue,
      rowLabel: (row) => row.model,
      rowCost: (row) => row.cost,
      rowCalls: (row) => row.turns,
      rowTokens: totalTokens,
      id: "chart-model-apex",
      onMetricChange: setSelectedMetric,
      isMetricDisabled: (m5) => totals[m5] <= 0,
      showLegend: true,
      onSelectRow: onSelectModel,
      onSliceClick: onSelectModel,
      formatCost: (v4) => fmtCost(v4),
      formatCalls: (v4) => fmt(v4),
      formatTokens: (v4) => fmt(v4)
    });
  }

  // src/ui/components/tables/cells.tsx
  function renderNumberCell(value, formatter = fmt) {
    return /* @__PURE__ */ u4("span", { class: "num", children: formatter(value) });
  }
  function renderCreditsCell(value) {
    return /* @__PURE__ */ u4("span", { class: "num", children: fmtCredits(value) });
  }
  function renderCostCell(value, isBillable = true) {
    return isBillable ? /* @__PURE__ */ u4("span", { class: "cost", children: fmtCost(value) }) : /* @__PURE__ */ u4("span", { class: "cost-na", children: "n/a" });
  }
  function renderTagCell(label, onSelect, className = "table-action-btn table-action-btn--tag") {
    if (!onSelect) return /* @__PURE__ */ u4("span", { class: "model-tag", children: label });
    return /* @__PURE__ */ u4("button", { type: "button", class: className, onClick: onSelect, children: /* @__PURE__ */ u4("span", { class: "model-tag", children: label }) });
  }
  function renderActionCell(label, title, onSelect, className = "table-action-btn") {
    if (!onSelect) return /* @__PURE__ */ u4("span", { title, children: label });
    return /* @__PURE__ */ u4("button", { type: "button", class: className, title, onClick: onSelect, children: label });
  }

  // src/ui/components/tables/ModelCostTable.tsx
  var defaultSort2 = [{ id: "cost", desc: true }];
  function CostShareBar({ value, max: max2, label }) {
    if (max2 <= 0 || value <= 0) return /* @__PURE__ */ u4("span", { class: "cost-na", children: "\u2014" });
    const pct = value / max2 * 100;
    return /* @__PURE__ */ u4("div", { style: { display: "flex", alignItems: "center", gap: "var(--space-2)", minWidth: "100px" }, children: [
      /* @__PURE__ */ u4("span", { class: "num", style: { fontSize: "var(--font-size-body)", minWidth: "52px", textAlign: "right" }, children: fmtCost(value) }),
      /* @__PURE__ */ u4(
        "div",
        {
          role: "img",
          style: {
            flex: 1,
            height: "4px",
            background: "color-mix(in srgb, var(--accent-interactive) 12%, transparent)",
            borderRadius: "var(--radius-1)",
            overflow: "hidden"
          },
          "aria-label": label,
          children: /* @__PURE__ */ u4(
            "div",
            {
              style: {
                height: "100%",
                width: `${Math.min(100, pct).toFixed(1)}%`,
                background: "var(--accent-interactive)",
                opacity: 0.6,
                borderRadius: "var(--radius-1)"
              }
            }
          )
        }
      )
    ] });
  }
  function useModelColumns(totalCost, totalCacheReadCost, totalCacheWriteCost, showCredits, onSelectModel) {
    return T2(
      () => [
        {
          id: "model",
          accessorKey: "model",
          header: "Model",
          enableSorting: false,
          cell: (info) => {
            const model = String(info.getValue());
            return renderTagCell(model, onSelectModel ? () => onSelectModel(model) : void 0);
          }
        },
        {
          id: "turns",
          accessorKey: "turns",
          header: "Turns",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "input",
          accessorKey: "input",
          header: "Input",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "output",
          accessorKey: "output",
          header: "Output",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "cache_read",
          accessorKey: "cache_read",
          header: "Cached Input",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "cache_creation",
          accessorKey: "cache_creation",
          header: "Cache Creation",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "cost",
          accessorKey: "cost",
          header: "Est. Cost",
          cell: (info) => {
            const row = info.row.original;
            return renderCostCell(Number(info.getValue() ?? 0), row.is_billable);
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
              return /* @__PURE__ */ u4("span", { class: "cost-na", children: "\u2014" });
            }
            const pct = row.cost / totalCost * 100;
            return /* @__PURE__ */ u4("div", { style: { minWidth: "120px", display: "flex", alignItems: "center", gap: "8px" }, children: [
              /* @__PURE__ */ u4("div", { style: { flex: 1 }, children: /* @__PURE__ */ u4(
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
              /* @__PURE__ */ u4("span", { class: "num", style: { fontSize: "11px", color: "var(--text-secondary)", minWidth: "36px", textAlign: "right" }, children: [
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
            if (!row.is_billable) return /* @__PURE__ */ u4("span", { class: "cost-na", children: "\u2014" });
            return /* @__PURE__ */ u4(
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
            if (!row.is_billable) return /* @__PURE__ */ u4("span", { class: "cost-na", children: "\u2014" });
            return /* @__PURE__ */ u4(
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
            return renderCreditsCell(v4);
          }
        }] : []
      ],
      [totalCost, totalCacheReadCost, totalCacheWriteCost, showCredits, onSelectModel]
    );
  }
  function ModelCostTable({
    byModel,
    onSelectModel
  }) {
    const totalCost = T2(
      () => byModel.reduce((s4, m5) => m5.is_billable ? s4 + m5.cost : s4, 0),
      [byModel]
    );
    const totalCacheReadCost = T2(
      () => byModel.reduce((s4, m5) => s4 + (m5.cache_read_cost ?? 0), 0),
      [byModel]
    );
    const totalCacheWriteCost = T2(
      () => byModel.reduce((s4, m5) => s4 + (m5.cache_write_cost ?? 0), 0),
      [byModel]
    );
    const showCredits = anyHasCredits(byModel);
    const columns7 = useModelColumns(totalCost, totalCacheReadCost, totalCacheWriteCost, showCredits, onSelectModel);
    return /* @__PURE__ */ u4(
      DataTable,
      {
        columns: columns7,
        data: byModel,
        title: "Cost by Model",
        sectionKey: "model-cost-mount",
        defaultSort: defaultSort2,
        costRows: true
      }
    );
  }

  // src/ui/components/OfficialSyncPanel.tsx
  function sourceVisible(provider, providerFilter2) {
    if (providerFilter2 === "both") return true;
    if (provider === "frankfurter") return true;
    if (providerFilter2 === "claude") return provider === "anthropic";
    return provider === "openai";
  }
  function statusLabel(status) {
    if (status === "success") return "OK";
    if (status === "skipped") return "SKIP";
    if (status === "parse_error") return "PARSE";
    if (status === "fetch_error") return "FETCH";
    return status.toUpperCase();
  }
  function statusColor2(status) {
    if (status === "success") return "var(--text-primary)";
    if (status === "skipped") return "var(--text-secondary)";
    return "var(--accent)";
  }
  function formatTs(ts) {
    if (!ts) return "n/a";
    return ts.slice(0, 19).replace("T", " ");
  }
  function OfficialSyncPanel({ summary, providerFilter: providerFilter2 }) {
    const expanded = official_sync_expanded.value;
    const sources = summary.sources.filter((source) => sourceVisible(source.provider, providerFilter2));
    const recordCounts = summary.record_counts.filter((record) => {
      if (providerFilter2 === "both") return true;
      if (providerFilter2 === "claude") {
        return !record.record_type.startsWith("usage_");
      }
      return true;
    });
    const successCount = sources.filter((source) => source.status === "success").length;
    const errorCount = sources.filter(
      (source) => source.status === "fetch_error" || source.status === "parse_error"
    ).length;
    const skippedCount = sources.filter((source) => source.status === "skipped").length;
    return /* @__PURE__ */ u4("div", { class: "card stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
      /* @__PURE__ */ u4(
        "div",
        {
          style: {
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            marginBottom: "8px"
          },
          children: [
            /* @__PURE__ */ u4("div", { class: "stat-label", children: "Official History" }),
            summary.available && /* @__PURE__ */ u4(
              "button",
              {
                onClick: () => {
                  official_sync_expanded.value = !expanded;
                  syncDashboardUrl();
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
                "aria-label": "Toggle official history details",
                children: expanded ? "\u25B2 collapse" : "\u25BC expand"
              }
            )
          ]
        }
      ),
      !summary.available ? /* @__PURE__ */ u4("div", { class: "muted", children: "No persisted official sync history yet" }) : /* @__PURE__ */ u4(S, { children: [
        /* @__PURE__ */ u4(
          "div",
          {
            style: {
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit,minmax(160px,1fr))",
              gap: "12px",
              marginBottom: expanded ? "12px" : "0"
            },
            children: [
              /* @__PURE__ */ u4("div", { children: [
                /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "18px" }, children: formatTs(summary.last_sync_at) }),
                /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Latest sync" })
              ] }),
              /* @__PURE__ */ u4("div", { children: [
                /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "18px" }, children: [
                  summary.total_runs,
                  " / ",
                  summary.total_records
                ] }),
                /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Runs / extracted records" })
              ] }),
              /* @__PURE__ */ u4("div", { children: [
                /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "18px" }, children: [
                  successCount,
                  " / ",
                  errorCount,
                  " / ",
                  skippedCount
                ] }),
                /* @__PURE__ */ u4("div", { class: "stat-sub", children: "OK / error / skipped sources" })
              ] })
            ]
          }
        ),
        expanded && /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "12px" }, children: [
          /* @__PURE__ */ u4("div", { children: [
            /* @__PURE__ */ u4(
              "div",
              {
                style: {
                  fontSize: "11px",
                  fontFamily: "var(--font-mono)",
                  color: "var(--text-secondary)",
                  marginBottom: "6px",
                  letterSpacing: "0.06em"
                },
                children: "LATEST SOURCES"
              }
            ),
            /* @__PURE__ */ u4("table", { style: { width: "100%", fontSize: "12px", borderCollapse: "collapse" }, children: [
              /* @__PURE__ */ u4("thead", { children: /* @__PURE__ */ u4("tr", { style: { color: "var(--text-secondary)" }, children: [
                /* @__PURE__ */ u4("th", { style: { textAlign: "left", padding: "2px 8px 2px 0", fontWeight: 500 }, children: "Source" }),
                /* @__PURE__ */ u4("th", { style: { textAlign: "left", padding: "2px 8px 2px 0", fontWeight: 500 }, children: "Kind" }),
                /* @__PURE__ */ u4("th", { style: { textAlign: "left", padding: "2px 8px 2px 0", fontWeight: 500 }, children: "Status" }),
                /* @__PURE__ */ u4("th", { style: { textAlign: "right", padding: "2px 0", fontWeight: 500 }, children: "Rows" })
              ] }) }),
              /* @__PURE__ */ u4("tbody", { children: sources.map((source) => /* @__PURE__ */ u4("tr", { children: [
                /* @__PURE__ */ u4("td", { style: { padding: "2px 8px 2px 0", fontFamily: "var(--font-mono)" }, children: source.source_slug }),
                /* @__PURE__ */ u4("td", { style: { padding: "2px 8px 2px 0", color: "var(--text-secondary)" }, children: source.source_kind }),
                /* @__PURE__ */ u4("td", { style: { padding: "2px 8px 2px 0", color: statusColor2(source.status), fontFamily: "var(--font-mono)" }, children: statusLabel(source.status) }),
                /* @__PURE__ */ u4("td", { style: { padding: "2px 0", textAlign: "right", fontFamily: "var(--font-mono)" }, children: source.record_count.toLocaleString() })
              ] }, source.source_slug)) })
            ] })
          ] }),
          /* @__PURE__ */ u4("div", { children: [
            /* @__PURE__ */ u4(
              "div",
              {
                style: {
                  fontSize: "11px",
                  fontFamily: "var(--font-mono)",
                  color: "var(--text-secondary)",
                  marginBottom: "6px",
                  letterSpacing: "0.06em"
                },
                children: "RECORD TYPES"
              }
            ),
            /* @__PURE__ */ u4("div", { style: { display: "flex", flexWrap: "wrap", gap: "8px" }, children: recordCounts.map((record) => /* @__PURE__ */ u4(
              "div",
              {
                style: {
                  border: "1px solid var(--border)",
                  padding: "6px 8px",
                  minWidth: "140px"
                },
                children: [
                  /* @__PURE__ */ u4("div", { style: { fontFamily: "var(--font-mono)", fontSize: "12px" }, children: record.record_type }),
                  /* @__PURE__ */ u4("div", { style: { fontSize: "11px", color: "var(--text-secondary)" }, children: [
                    record.count.toLocaleString(),
                    " rows"
                  ] })
                ]
              },
              record.record_type
            )) })
          ] }),
          summary.latest_success_at && /* @__PURE__ */ u4(
            "div",
            {
              style: {
                fontSize: "10px",
                color: "var(--text-secondary)",
                fontFamily: "var(--font-mono)"
              },
              children: [
                "Latest successful extraction ",
                formatTs(summary.latest_success_at),
                " UTC"
              ]
            }
          )
        ] })
      ] })
    ] }) });
  }

  // src/ui/components/charts/ProjectChart.tsx
  function ProjectChart({
    byProject,
    onSelectProject
  }) {
    const top = byProject.slice(0, 10);
    if (!top.length) return null;
    const base = dashboardChartOptions("bar");
    const colors = tokenSeriesColors();
    const totals = top.map((p5) => p5.input + p5.output);
    const maxTotal = totals.reduce((m5, v4) => v4 > m5 ? v4 : m5, 0);
    const shares = totals.map((v4) => maxTotal > 0 ? v4 / maxTotal * 100 : 0);
    const options = {
      ...base,
      chart: {
        ...base.chart,
        type: "bar",
        ...onSelectProject ? {
          events: {
            dataPointSelection: (_event, _ctx, config) => {
              const row = top[config.dataPointIndex];
              if (row) onSelectProject(row);
            }
          }
        } : {}
      },
      series: [{ name: "Share of top", data: shares }],
      colors: [colors[0] ?? "currentColor"],
      fill: { type: "solid" },
      plotOptions: { bar: { horizontal: true, barHeight: "60%", borderRadius: 0 } },
      xaxis: {
        ...base.xaxis ?? {},
        categories: top.map((p5) => truncateMid(p5.display_name || p5.project, 18, 8)),
        min: 0,
        max: 100,
        tickAmount: 4,
        labels: {
          ...base.xaxis?.labels ?? {},
          formatter: (v4) => `${Math.round(v4)}%`,
          hideOverlappingLabels: true
        }
      },
      yaxis: {
        ...base.yaxis ?? {},
        labels: { ...base.yaxis?.labels ?? {}, maxWidth: 120 }
      },
      // Anchor the tooltip to the plot's bottom-left so it cannot cover the
      // card's "TOP PROJECTS" title during hover.
      tooltip: {
        ...base.tooltip,
        fixed: { enabled: true, position: "bottomLeft", offsetX: 0, offsetY: 0 },
        y: {
          // Display the raw token count per project regardless of bar scale.
          formatter: (_v, opts) => {
            const raw = totals[opts?.dataPointIndex ?? 0] ?? 0;
            return fmt(raw) + " tokens";
          }
        }
      }
    };
    return /* @__PURE__ */ u4(ApexChart, { options, id: "chart-project" });
  }

  // src/ui/components/tables/ProjectCostTable.tsx
  var defaultSort3 = [
    { id: "pinned", desc: true },
    { id: "cost", desc: true }
  ];
  function useProjectColumns(showCredits, onSelectProject, onReload) {
    return T2(
      () => [
        {
          id: "pinned",
          accessorFn: (row) => row.pinned ? 1 : 0,
          header: "Pin",
          sortingFn: (a4, b4) => {
            const ap = a4.original.pinned ? 1 : 0;
            const bp = b4.original.pinned ? 1 : 0;
            return ap === bp ? 0 : bp - ap;
          },
          cell: (info) => {
            const row = info.row.original;
            const reg = registryBySlug.value.get(row.project);
            if (!reg) {
              return /* @__PURE__ */ u4("span", { style: { opacity: 0.2, color: "var(--color-text-primary)" }, title: "No registry entry yet", children: "\u2606" });
            }
            return /* @__PURE__ */ u4(
              PinStar,
              {
                projectUuid: reg.project_uuid,
                pinned: row.pinned ?? reg.pinned,
                label: row.display_name || row.project,
                ...onReload ? { onChange: onReload } : {}
              }
            );
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
            return renderActionCell(label, row.project, onSelectProject ? () => onSelectProject(row) : void 0);
          }
        },
        {
          id: "sessions",
          accessorKey: "sessions",
          header: "Sessions",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), (value) => String(value))
        },
        {
          id: "turns",
          accessorKey: "turns",
          header: "Turns",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "input",
          accessorKey: "input",
          header: "Input",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "output",
          accessorKey: "output",
          header: "Output",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "cost",
          accessorKey: "cost",
          header: "Est. Cost",
          cell: (info) => renderCostCell(Number(info.getValue() ?? 0))
        },
        ...showCredits ? [{
          id: "credits",
          accessorFn: (row) => row.credits ?? null,
          header: "Credits",
          sortUndefined: "last",
          cell: (info) => {
            const v4 = info.getValue();
            return renderCreditsCell(v4);
          }
        }] : []
      ],
      [showCredits, onSelectProject, onReload]
    );
  }
  function ProjectCostTable({
    byProject,
    onExportCSV,
    onSelectProject,
    onReload
  }) {
    const showCredits = anyHasCredits(byProject);
    const columns7 = useProjectColumns(showCredits, onSelectProject, onReload);
    return /* @__PURE__ */ u4(
      DataTable,
      {
        columns: columns7,
        data: byProject,
        title: "Cost by Project",
        sectionKey: "project-cost-mount",
        exportFn: onExportCSV,
        defaultSort: defaultSort3,
        costRows: true
      }
    );
  }

  // src/ui/components/RateWindowCard.tsx
  function RateWindowCard({ label, window: window2 }) {
    const pct = Math.min(100, window2.used_percent);
    const resetText = window2.resets_in_minutes != null ? `Resets in ${fmtResetTime(window2.resets_in_minutes)}` : "";
    return /* @__PURE__ */ u4(
      KpiCard,
      {
        label,
        value: `${pct.toFixed(1)}%`,
        bar: { value: window2.used_percent, max: 100, ariaLabel: `${label} usage` },
        sub: resetText || void 0
      }
    );
  }
  function BudgetCard({ used, limit, currency, utilization }) {
    return /* @__PURE__ */ u4(
      KpiCard,
      {
        size: "compact",
        label: "Monthly budget",
        value: `$${used.toFixed(2)} / $${limit.toFixed(2)}`,
        bar: { value: utilization, max: 100, ariaLabel: "Monthly budget usage" },
        sub: currency
      }
    );
  }
  function RateWindowUnavailable({ error }) {
    return /* @__PURE__ */ u4(
      KpiCard,
      {
        size: "compact",
        valueTone: "muted",
        label: "Rate windows",
        value: "Unavailable",
        sub: error
      }
    );
  }
  function ClaudeAdminCard({ label, value, subtitle }) {
    return /* @__PURE__ */ u4(KpiCard, { size: "compact", label, value, sub: subtitle });
  }
  function ClaudeAdminFallbackGrid({ summary }) {
    const subtitle = `${summary.organization_name || "Org-wide"} \xB7 ${summary.data_latency_note}`;
    return /* @__PURE__ */ u4(S, { children: [
      /* @__PURE__ */ u4(
        ClaudeAdminCard,
        {
          label: "Active users today",
          value: summary.today_active_users.toLocaleString(),
          subtitle
        }
      ),
      /* @__PURE__ */ u4(
        ClaudeAdminCard,
        {
          label: "Sessions today",
          value: summary.today_sessions.toLocaleString(),
          subtitle
        }
      ),
      /* @__PURE__ */ u4(
        ClaudeAdminCard,
        {
          label: `Accepted lines (${summary.lookback_days}d)`,
          value: summary.lookback_lines_accepted.toLocaleString(),
          subtitle
        }
      ),
      /* @__PURE__ */ u4(
        ClaudeAdminCard,
        {
          label: `Estimated spend (${summary.lookback_days}d)`,
          value: `$${summary.lookback_estimated_cost_usd.toFixed(2)}`,
          subtitle
        }
      )
    ] });
  }

  // src/ui/components/ReconciliationBlock.tsx
  function ReconciliationBlock({ reconciliation }) {
    const deltaMatch = Math.abs(reconciliation.delta_cost) < 0.01;
    if (!reconciliation.available) {
      return /* @__PURE__ */ u4("div", { class: "card card-flat bento-full", style: { padding: "12px 20px" }, children: /* @__PURE__ */ u4("div", { style: {
        display: "flex",
        alignItems: "center",
        flexWrap: "wrap",
        gap: "12px",
        fontFamily: "var(--font-mono)",
        fontSize: "12px",
        letterSpacing: "0.04em",
        color: "var(--text-secondary)"
      }, children: [
        /* @__PURE__ */ u4("span", { style: {
          fontSize: "10px",
          letterSpacing: "0.08em",
          textTransform: "uppercase",
          color: "var(--text-disabled)"
        }, children: "OpenAI Reconciliation" }),
        /* @__PURE__ */ u4("span", { style: { color: "var(--text-disabled)" }, children: "\xB7" }),
        /* @__PURE__ */ u4("span", { children: reconciliation.error ?? "Unavailable" })
      ] }) });
    }
    return /* @__PURE__ */ u4("div", { class: "card card-flat bento-full", children: [
      /* @__PURE__ */ u4("h2", { children: "OpenAI Org Usage Reconciliation" }),
      /* @__PURE__ */ u4("div", { class: "muted", style: { marginBottom: "12px" }, children: [
        "Official OpenAI organization usage buckets for Codex-compatible models over the last ",
        reconciliation.lookback_days,
        " days."
      ] }),
      reconciliation.available ? /* @__PURE__ */ u4("div", { style: { display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(200px,1fr))", gap: "16px" }, children: [
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Period" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "16px" }, children: [
            reconciliation.start_date,
            " - ",
            reconciliation.end_date
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Rolling comparison window" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Local Estimated Cost" }),
          /* @__PURE__ */ u4("div", { class: "stat-value cost-value", style: { fontSize: "20px" }, children: [
            "$",
            reconciliation.estimated_local_cost.toFixed(4)
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Codex local logs" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Org Usage Cost" }),
          /* @__PURE__ */ u4("div", { class: "stat-value cost-value", style: { fontSize: "20px" }, children: [
            "$",
            reconciliation.api_usage_cost.toFixed(4)
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "OpenAI organization usage API" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Delta" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "20px", color: deltaMatch ? "var(--text-primary)" : "var(--accent)" }, children: [
            reconciliation.delta_cost >= 0 ? "+" : "",
            "$",
            reconciliation.delta_cost.toFixed(4)
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Org usage cost minus local estimate" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "API Tokens" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "16px" }, children: [
            reconciliation.api_input_tokens.toLocaleString(),
            " / ",
            reconciliation.api_output_tokens.toLocaleString()
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Input / output tokens" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Cached Input + Requests" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "16px" }, children: [
            reconciliation.api_cached_input_tokens.toLocaleString(),
            " / ",
            reconciliation.api_requests.toLocaleString()
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Cached input tokens / requests" })
        ] }) })
      ] }) : /* @__PURE__ */ u4("div", { class: "muted", children: reconciliation.error ?? "Unavailable" })
    ] });
  }

  // src/ui/components/SubagentReconciliationBlock.tsx
  function SubagentReconciliationBlock({ reconciliation }) {
    const deltaMatch = Math.abs(reconciliation.delta_cost) < 0.01;
    if (!reconciliation.available) {
      return /* @__PURE__ */ u4("div", { class: "card card-flat bento-full", style: { padding: "12px 20px" }, children: /* @__PURE__ */ u4("div", { style: {
        display: "flex",
        alignItems: "center",
        flexWrap: "wrap",
        gap: "12px",
        fontFamily: "var(--font-mono)",
        fontSize: "12px",
        letterSpacing: "0.04em",
        color: "var(--text-secondary)"
      }, children: [
        /* @__PURE__ */ u4("span", { style: {
          fontSize: "10px",
          letterSpacing: "0.08em",
          textTransform: "uppercase",
          color: "var(--text-disabled)"
        }, children: "Subagent Reconciliation" }),
        /* @__PURE__ */ u4("span", { style: { color: "var(--text-disabled)" }, children: "\xB7" }),
        /* @__PURE__ */ u4("span", { children: reconciliation.error ?? "Unavailable" })
      ] }) });
    }
    const statusBracket = deltaMatch ? { label: "[OK]", color: "var(--success, var(--text-primary))" } : { label: `[DRIFT: ${reconciliation.delta_cost >= 0 ? "+" : ""}$${reconciliation.delta_cost.toFixed(4)}]`, color: "var(--accent)" };
    return /* @__PURE__ */ u4("div", { class: "card card-flat bento-full", children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", alignItems: "baseline", gap: "12px", flexWrap: "wrap" }, children: [
        /* @__PURE__ */ u4("h2", { style: { margin: 0 }, children: "Subagent Cost Reconciliation" }),
        /* @__PURE__ */ u4(
          "span",
          {
            style: {
              fontFamily: "var(--font-mono)",
              fontSize: "11px",
              letterSpacing: "0.04em",
              color: statusBracket.color
            },
            "aria-label": deltaMatch ? "reconciliation matches within tolerance" : "reconciliation drift detected",
            children: statusBracket.label
          }
        )
      ] }),
      /* @__PURE__ */ u4("div", { class: "muted", style: { marginBottom: "12px", marginTop: "4px" }, children: [
        "Compares the child agent JSONL view (",
        /* @__PURE__ */ u4("code", { children: "agent_sessions" }),
        ") against the parent sidechain view (",
        /* @__PURE__ */ u4("code", { children: "turns WHERE is_subagent = 1" }),
        ") over the last",
        " ",
        reconciliation.lookback_days,
        " days. Drift signals parser divergence."
      ] }),
      /* @__PURE__ */ u4("div", { style: { display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(200px,1fr))", gap: "16px" }, children: [
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Period" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "16px" }, children: [
            reconciliation.start_date,
            " - ",
            reconciliation.end_date
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Rolling comparison window" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Agent-Sessions Cost" }),
          /* @__PURE__ */ u4("div", { class: "stat-value cost-value", style: { fontSize: "20px" }, children: [
            "$",
            reconciliation.agent_sessions_cost.toFixed(4)
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Child JSONL view" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Sidechain Turns Cost" }),
          /* @__PURE__ */ u4("div", { class: "stat-value cost-value", style: { fontSize: "20px" }, children: [
            "$",
            reconciliation.turns_subagent_cost.toFixed(4)
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Parent JSONL view" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Delta" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "20px", color: deltaMatch ? "var(--text-primary)" : "var(--accent)" }, children: [
            reconciliation.delta_cost >= 0 ? "+" : "",
            "$",
            reconciliation.delta_cost.toFixed(4)
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Agent-sessions minus sidechain" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Spawns / Sidechain Turns" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "16px" }, children: [
            reconciliation.agent_session_rows.toLocaleString(),
            " / ",
            reconciliation.subagent_turn_rows.toLocaleString()
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Row counts per view" })
        ] }) }),
        /* @__PURE__ */ u4("div", { class: "stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Distinct Agents" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "16px" }, children: [
            reconciliation.distinct_agents_in_agent_sessions.toLocaleString(),
            " / ",
            reconciliation.distinct_agents_in_turns.toLocaleString()
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Child / parent" })
        ] }) })
      ] })
    ] });
  }

  // src/ui/components/tables/ServiceTiers.tsx
  var columns5 = [
    {
      accessorKey: "provider",
      header: "Provider",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
    },
    {
      accessorKey: "service_tier",
      header: "Tier",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { children: fmtLabel(getValue()) })
    },
    {
      accessorKey: "inference_geo",
      header: "Region",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { children: fmtLabel(getValue()) })
    },
    {
      accessorKey: "turns",
      header: "Turns",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
    }
  ];
  function ServiceTiersTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u4(DataTable, { columns: columns5, data, title: "Service Tiers", sectionKey: "service-tiers" });
  }

  // src/ui/components/tables/SessionsTable.tsx
  var defaultSort4 = [{ id: "last", desc: true }];
  var primaryOverflowStyle = {
    display: "block",
    minWidth: 0,
    maxWidth: "clamp(12rem, 24vw, 20rem)",
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap"
  };
  var secondaryOverflowStyle = {
    ...primaryOverflowStyle,
    marginTop: "2px",
    fontSize: "var(--font-size-tertiary)",
    fontFamily: "var(--font-mono)"
  };
  var projectOverflowStyle = {
    ...primaryOverflowStyle,
    maxWidth: "clamp(10rem, 20vw, 18rem)"
  };
  function useSessionColumns(showCredits, onSelectProject, onSelectModel) {
    return T2(
      () => [
        {
          id: "session",
          accessorKey: "session_id",
          header: "Session",
          enableSorting: false,
          cell: (info) => {
            const row = info.row.original;
            const title = row.title?.trim();
            const sessionId = String(info.getValue());
            const tooltip = title ? `${title}
${sessionId}` : sessionId;
            return /* @__PURE__ */ u4("div", { style: { minWidth: 0, maxWidth: "clamp(14rem, 28vw, 24rem)" }, title: tooltip, children: [
              /* @__PURE__ */ u4("span", { class: "muted", style: { ...primaryOverflowStyle, fontFamily: "var(--font-mono)" }, children: title || sessionId }),
              title && /* @__PURE__ */ u4("span", { class: "muted", style: secondaryOverflowStyle, children: sessionId })
            ] });
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
            const showProjectPath = label !== row.project;
            const tooltip = showProjectPath ? `${label}
${row.project}` : row.project;
            const content = /* @__PURE__ */ u4(S, { children: [
              /* @__PURE__ */ u4("span", { style: projectOverflowStyle, children: label }),
              showProjectPath && /* @__PURE__ */ u4("span", { class: "muted", style: secondaryOverflowStyle, children: row.project })
            ] });
            return /* @__PURE__ */ u4("div", { style: { minWidth: 0, maxWidth: "clamp(12rem, 24vw, 22rem)" }, title: tooltip, children: onSelectProject ? /* @__PURE__ */ u4("button", { type: "button", class: "table-action-btn table-action-btn--stack", onClick: () => onSelectProject(row), children: content }) : content });
          }
        },
        {
          id: "provider",
          accessorKey: "provider",
          header: "Provider",
          enableSorting: false,
          cell: (info) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(info.getValue()).toUpperCase() })
        },
        {
          id: "last",
          accessorKey: "last",
          header: "Last Active",
          cell: (info) => /* @__PURE__ */ u4("span", { class: "muted", children: String(info.getValue() ?? "") })
        },
        {
          id: "duration_min",
          accessorKey: "duration_min",
          header: "Duration",
          cell: (info) => /* @__PURE__ */ u4("span", { class: "muted", children: [
            Number(info.getValue() ?? 0),
            "m"
          ] })
        },
        {
          id: "model",
          accessorKey: "model",
          header: "Model",
          enableSorting: false,
          cell: (info) => {
            const model = String(info.getValue());
            return renderTagCell(model, onSelectModel ? () => onSelectModel(model) : void 0);
          }
        },
        {
          id: "turns",
          accessorKey: "turns",
          header: "Turns",
          cell: (info) => {
            const row = info.row.original;
            return /* @__PURE__ */ u4("span", { class: "num", children: [
              fmt(Number(info.getValue() ?? 0)),
              row.subagent_count > 0 && /* @__PURE__ */ u4("span", { class: "muted", style: { fontSize: "10px" }, children: [
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
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "output",
          accessorKey: "output",
          header: "Output",
          cell: (info) => renderNumberCell(Number(info.getValue() ?? 0), fmt)
        },
        {
          id: "cost",
          accessorKey: "cost",
          header: "Est. Cost",
          cell: (info) => {
            const row = info.row.original;
            return renderCostCell(Number(info.getValue() ?? 0), row.is_billable);
          }
        },
        ...showCredits ? [{
          id: "credits",
          accessorFn: (row) => row.credits ?? null,
          header: "Credits",
          sortUndefined: "last",
          cell: (info) => {
            const v4 = info.getValue();
            return renderCreditsCell(v4);
          }
        }] : [],
        {
          id: "cost_meta",
          accessorKey: "cost_confidence",
          header: "Cost Meta",
          enableSorting: false,
          cell: (info) => {
            const row = info.row.original;
            const pricing = row.pricing_version || "n/a";
            const shortPricing = pricing.includes("@") ? pricing.split("@")[0] : pricing;
            return /* @__PURE__ */ u4("div", { class: "muted", style: { fontSize: "10px", lineHeight: "1.4" }, children: [
              /* @__PURE__ */ u4("div", { style: { whiteSpace: "nowrap" }, children: [
                fmtLabel(row.cost_confidence || "low"),
                " / ",
                fmtLabel(row.billing_mode || "estimated_local")
              ] }),
              /* @__PURE__ */ u4("div", { title: pricing, style: { whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", maxWidth: "140px" }, children: shortPricing })
            ] });
          }
        },
        {
          id: "subagent_delta_cost",
          accessorFn: (row) => {
            const ag = row.agent_sessions_cost_nanos ?? 0;
            const tc = row.subagent_turns_cost_nanos ?? 0;
            if (ag === 0 && tc === 0) return null;
            return (ag - tc) / 1e9;
          },
          header: "Subagent \u0394",
          sortUndefined: "last",
          cell: (info) => {
            const v4 = info.getValue();
            if (v4 == null) return /* @__PURE__ */ u4("span", { class: "muted", children: "--" });
            const drift = Math.abs(v4) >= 0.01;
            const sign = v4 >= 0 ? "+" : "";
            return /* @__PURE__ */ u4(
              "span",
              {
                class: "num",
                style: drift ? { color: "var(--accent)" } : void 0,
                title: "agent_sessions cost minus turns(is_subagent=1) cost",
                children: [
                  sign,
                  "$",
                  v4.toFixed(4)
                ]
              }
            );
          }
        },
        {
          id: "cache_hit_ratio",
          accessorKey: "cache_hit_ratio",
          header: "Cache %",
          cell: (info) => {
            const v4 = info.getValue();
            if (v4 == null || !Number.isFinite(v4)) return /* @__PURE__ */ u4("span", { class: "num", children: "--" });
            return /* @__PURE__ */ u4("span", { class: "num", children: [
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
            return /* @__PURE__ */ u4("span", { class: "num", children: v4 > 0 ? fmt(Math.round(v4)) : "--" });
          }
        }
      ],
      [showCredits, onSelectProject, onSelectModel]
    );
  }
  function SessionsTable({
    onExportCSV,
    onSelectProject,
    onSelectModel
  }) {
    const data = lastFilteredSessions.value;
    const showCredits = anyHasCredits(data);
    const columns7 = useSessionColumns(showCredits, onSelectProject, onSelectModel);
    const pagination = sessionsTablePagination.value;
    const columnVisibility = sessionsTableColumnVisibility.value;
    const handlePaginationChange = (nextPagination) => {
      sessionsTablePagination.value = nextPagination;
      syncDashboardUrl();
    };
    const handleColumnVisibilityChange = (nextColumnVisibility) => {
      sessionsTableColumnVisibility.value = nextColumnVisibility;
      syncDashboardUrl();
    };
    return /* @__PURE__ */ u4(
      DataTable,
      {
        columns: columns7,
        data,
        title: "Recent Sessions",
        sectionKey: "sessions-mount",
        exportFn: onExportCSV,
        pageSize: SESSIONS_PAGE_SIZE,
        defaultSort: defaultSort4,
        enableColumnVisibility: true,
        paginationState: pagination,
        onPaginationChange: handlePaginationChange,
        columnVisibilityState: columnVisibility,
        onColumnVisibilityChange: handleColumnVisibilityChange
      }
    );
  }

  // src/ui/components/charts/Sparkline.tsx
  function Sparkline({ daily }) {
    const last7 = daily.slice(-7);
    if (last7.length < 2) return null;
    const options = {
      chart: {
        type: "line",
        height: 32,
        width: "100%",
        sparkline: { enabled: true },
        background: "transparent",
        fontFamily: "inherit"
      },
      series: [{ data: last7.map((d5) => d5.input + d5.output) }],
      stroke: { width: 1.5, curve: "smooth" },
      colors: [cssVar("--text-secondary")],
      tooltip: { enabled: false }
    };
    return /* @__PURE__ */ u4("div", { children: [
      /* @__PURE__ */ u4("div", { class: "sub", style: { marginBottom: "4px" }, children: "7-day trend" }),
      /* @__PURE__ */ u4(ApexChart, { options })
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
    const displayPct = hasRate ? rate >= 0.999 ? "Fully cached" : (rate * 100).toFixed(1) + "%" : "--";
    const barFill = hasRate ? Math.max(0, Math.min(1, rate)) : 0;
    const tooltipParts = [];
    if (hasRate) {
      const readM = (data.cache_read_tokens / 1e6).toFixed(2);
      const totalM = ((data.cache_read_tokens + data.cache_write_tokens + data.input_tokens) / 1e6).toFixed(2);
      tooltipParts.push(
        `${readM}M cache reads / ${totalM}M total input-side tokens (cache reads + cache writes + fresh input)`
      );
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
    return /* @__PURE__ */ u4("div", { class: "card stat-card", title: tooltip, children: [
      /* @__PURE__ */ u4("div", { class: "stat-content", children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", children: "Cache Hit Rate" }),
        /* @__PURE__ */ u4(
          "div",
          {
            class: "stat-value",
            style: { fontFamily: "var(--font-mono)", letterSpacing: "-0.02em" },
            children: displayPct
          }
        ),
        /* @__PURE__ */ u4("div", { class: "stat-sub", children: "prompt cache reuse" })
      ] }),
      /* @__PURE__ */ u4(
        "div",
        {
          role: "img",
          style: {
            marginTop: "10px",
            height: "4px",
            borderRadius: "2px",
            background: "rgba(var(--text-primary-rgb, 232,232,232), 0.12)",
            overflow: "hidden"
          },
          "aria-label": `Cache hit rate: ${displayPct}`,
          children: /* @__PURE__ */ u4(
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
    const m5 = totalMin % 60;
    if (h5 === 0) return `${m5}m`;
    return `${h5}h ${m5}m`;
  }
  function fmtUtcTime(iso) {
    try {
      const d5 = new Date(iso);
      return d5.toISOString().slice(11, 16) + " UTC";
    } catch {
      return "--";
    }
  }
  function runoutLabel(block) {
    const quota = block.quota;
    if (!quota || quota.runout_in_minutes == null) return null;
    const eta = fmtResetTime(quota.runout_in_minutes);
    if (quota.will_run_out_before_reset === false) {
      return `Reset before runout (${eta})`;
    }
    if (quota.runout_in_minutes <= 0) return "Runs out now";
    return `Runs out in ${eta}`;
  }
  function runoutTimeLabel(block) {
    const quota = block.quota;
    if (!quota?.runout_at || quota.will_run_out_before_reset === false) return null;
    return `at ${fmtUtcTime(quota.runout_at)}`;
  }
  function QuotaSuggestionsSection({ data }) {
    const suggestions = data.quota_suggestions;
    if (!suggestions || suggestions.levels.length === 0) {
      return null;
    }
    return /* @__PURE__ */ u4("div", { style: { marginTop: "12px", display: "grid", gap: "8px" }, children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "baseline" }, children: [
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontSize: "10px", letterSpacing: "0.08em" }, children: "SUGGESTED QUOTAS" }),
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "11px" }, children: suggestions.sample_label })
      ] }),
      suggestions.sample_count !== suggestions.population_count && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontStyle: "italic" }, children: [
        "Derived from ",
        suggestions.population_count,
        " completed blocks, biased toward near-limit history."
      ] }),
      data.token_limit != null && /* @__PURE__ */ u4(
        "div",
        {
          style: {
            display: "flex",
            justifyContent: "space-between",
            gap: "12px",
            alignItems: "baseline"
          },
          children: [
            /* @__PURE__ */ u4("span", { class: "stat-sub", children: "Configured" }),
            /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "11px" }, children: fmt(data.token_limit) })
          ]
        }
      ),
      /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "6px" }, children: suggestions.levels.map((level) => /* @__PURE__ */ u4(
        "div",
        {
          style: {
            display: "flex",
            justifyContent: "space-between",
            gap: "12px",
            alignItems: "baseline"
          },
          children: [
            /* @__PURE__ */ u4("span", { class: "stat-sub", children: [
              level.label,
              level.key === suggestions.recommended_key && /* @__PURE__ */ u4("small", { style: { marginLeft: "var(--space-2)", fontSize: "var(--font-size-tertiary)", color: "var(--accent-interactive)", fontFamily: "var(--font-mono)" }, children: "[Recommended]" })
            ] }),
            /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "11px" }, children: fmt(level.limit_tokens) })
          ]
        },
        level.key
      )) }),
      suggestions.note && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontStyle: "italic" }, children: suggestions.note })
    ] });
  }
  function QuotaSection({ block }) {
    const { quota } = block;
    if (!quota) {
      return /* @__PURE__ */ u4(
        "div",
        {
          class: "stat-sub",
          style: { marginTop: "8px", fontStyle: "italic" },
          children: "Token quota not configured \u2014 set [blocks.token_limit] in config."
        }
      );
    }
    const currentPct = Math.min(100, quota.current_pct).toFixed(0);
    const projectedPct = Math.min(999, quota.projected_pct).toFixed(0);
    const runout = runoutLabel(block);
    const runoutTime = runoutTimeLabel(block);
    return /* @__PURE__ */ u4("div", { style: { marginTop: "10px" }, children: [
      /* @__PURE__ */ u4("div", { style: { marginBottom: "4px" }, children: [
        /* @__PURE__ */ u4(
          "div",
          {
            style: {
              display: "flex",
              justifyContent: "space-between",
              alignItems: "baseline",
              marginBottom: "3px"
            },
            children: [
              /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontSize: "10px", letterSpacing: "0.08em" }, children: "USED" }),
              /* @__PURE__ */ u4(
                "span",
                {
                  class: "stat-sub",
                  style: { fontFamily: "var(--font-mono)", fontSize: "11px" },
                  children: [
                    fmt(quota.used_tokens),
                    " / ",
                    fmt(quota.limit_tokens),
                    " ",
                    currentPct,
                    "%",
                    " ",
                    /* @__PURE__ */ u4(
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
        /* @__PURE__ */ u4(
          SegmentedProgressBar,
          {
            value: quota.used_tokens,
            max: quota.limit_tokens,
            status: severityToStatus(quota.current_severity),
            "aria-label": "Token quota used"
          }
        )
      ] }),
      /* @__PURE__ */ u4("div", { style: { marginTop: "1px" }, children: [
        /* @__PURE__ */ u4(
          "div",
          {
            style: {
              display: "flex",
              justifyContent: "space-between",
              alignItems: "baseline",
              marginBottom: "3px"
            },
            children: [
              /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontSize: "10px", letterSpacing: "0.08em" }, children: "PROJECTED" }),
              /* @__PURE__ */ u4(
                "span",
                {
                  class: "stat-sub",
                  style: { fontFamily: "var(--font-mono)", fontSize: "11px" },
                  children: [
                    fmt(quota.projected_tokens),
                    " / ",
                    fmt(quota.limit_tokens),
                    " ",
                    projectedPct,
                    "%",
                    " ",
                    /* @__PURE__ */ u4(
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
        /* @__PURE__ */ u4(
          SegmentedProgressBar,
          {
            value: quota.projected_tokens,
            max: quota.limit_tokens,
            status: severityToStatus(quota.projected_severity),
            "aria-label": "Projected token quota"
          }
        )
      ] }),
      runout && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { marginTop: "8px", fontFamily: "var(--font-mono)", fontSize: "11px" }, children: [
        runout,
        " ",
        runoutTime
      ] })
    ] });
  }
  function BillingBlocksCard({ data }) {
    const activeBlock = data.blocks.find((b4) => b4.is_active) ?? null;
    if (!activeBlock) {
      return /* @__PURE__ */ u4("div", { class: "card stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", style: { letterSpacing: "0.08em", fontSize: "11px" }, children: "BILLING BLOCK" }),
        /* @__PURE__ */ u4("div", { class: "stat-value", style: { opacity: 0.4 }, children: "NO ACTIVE BLOCK" }),
        /* @__PURE__ */ u4("div", { class: "stat-sub", children: [
          "7d historical max:",
          " ",
          /* @__PURE__ */ u4("span", { style: { fontFamily: "var(--font-mono)" }, children: fmt(data.historical_max_tokens) }),
          " ",
          "tokens"
        ] }),
        /* @__PURE__ */ u4(QuotaSuggestionsSection, { data })
      ] }) });
    }
    const totalTokens2 = activeBlock.tokens.input + activeBlock.tokens.output + activeBlock.tokens.cache_read + activeBlock.tokens.cache_creation + activeBlock.tokens.reasoning_output;
    const elapsed = formatDuration(activeBlock.first_timestamp, activeBlock.last_timestamp);
    const blockEnd = fmtUtcTime(activeBlock.end);
    return /* @__PURE__ */ u4("div", { class: "card stat-card", children: [
      /* @__PURE__ */ u4("div", { class: "stat-content", children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", style: { letterSpacing: "0.08em", fontSize: "11px" }, children: "BILLING BLOCK" }),
        /* @__PURE__ */ u4(
          "div",
          {
            class: "stat-value",
            style: { fontFamily: "var(--font-mono)", letterSpacing: "-0.02em" },
            children: fmt(totalTokens2)
          }
        ),
        /* @__PURE__ */ u4("div", { class: "stat-sub", children: [
          elapsed,
          " elapsed \xB7 ends ",
          blockEnd,
          " \xB7 ",
          activeBlock.entry_count,
          " entries"
        ] }),
        activeBlock.burn_rate && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "12px", marginTop: "4px" }, children: [
          "$",
          (activeBlock.burn_rate.cost_per_hour_nanos / 1e9).toFixed(4),
          "/hr",
          activeBlock.burn_rate.tier && /* @__PURE__ */ u4(
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
        ] }),
        activeBlock.projection && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "12px", marginTop: "4px" }, children: [
          "Projects ",
          fmt(activeBlock.projection.projected_tokens),
          " tokens \xB7 $",
          (activeBlock.projection.projected_cost_nanos / 1e9).toFixed(4)
        ] })
      ] }),
      /* @__PURE__ */ u4(QuotaSection, { block: activeBlock }),
      /* @__PURE__ */ u4(QuotaSuggestionsSection, { data })
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
    return /* @__PURE__ */ u4("div", { class: "card stat-card", children: [
      /* @__PURE__ */ u4("div", { class: "stat-content", children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", style: { letterSpacing: "0.08em", fontSize: "11px" }, children: "CONTEXT WINDOW" }),
        /* @__PURE__ */ u4(
          "div",
          {
            class: "stat-value",
            style: { fontFamily: "var(--font-mono)", letterSpacing: "-0.02em" },
            children: fmt(used)
          }
        ),
        /* @__PURE__ */ u4("div", { class: "stat-sub", children: [
          "of ",
          fmt(size),
          " \xB7 ",
          pct.toFixed(1),
          "%",
          " ",
          /* @__PURE__ */ u4(
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
      /* @__PURE__ */ u4(
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

  // src/ui/components/DepletionForecastCard.tsx
  function severityToStatus3(severity) {
    if (severity === "danger") return "accent";
    if (severity === "warn") return "warning";
    return "success";
  }
  function primaryValueLabel(signal) {
    const percent = signal.projected_percent ?? signal.used_percent;
    return `${Math.round(percent)}% ${signal.projected_percent != null ? "projected" : "used"}`;
  }
  function remainingLabel(signal) {
    if (signal.remaining_tokens != null) {
      return `${fmt(signal.remaining_tokens)} tokens left`;
    }
    if (signal.remaining_percent != null) {
      return `${Math.round(signal.remaining_percent)}% remaining`;
    }
    return null;
  }
  function timingLabel(signal) {
    if (signal.resets_in_minutes != null) {
      return `Resets in ${fmtResetTime(signal.resets_in_minutes)}`;
    }
    if (!signal.end_time) return null;
    const date = new Date(signal.end_time);
    if (Number.isNaN(date.getTime())) return signal.end_time;
    return `Ends ${date.toISOString().slice(11, 16)} UTC`;
  }
  function runoutLabel2(signal) {
    if (signal.runout_in_minutes == null) return null;
    if (signal.will_run_out_before_reset === false) {
      return `Reset before runout (${fmtResetTime(signal.runout_in_minutes)})`;
    }
    if (signal.runout_in_minutes <= 0) return "Limit exhausted now";
    return `Runs out in ${fmtResetTime(signal.runout_in_minutes)}`;
  }
  function runoutTimeLabel2(signal) {
    if (!signal.runout_at || signal.will_run_out_before_reset === false) return null;
    const date = new Date(signal.runout_at);
    if (Number.isNaN(date.getTime())) return signal.runout_at;
    return `Runout ${date.toISOString().slice(11, 16)} UTC`;
  }
  function supportValue(signal) {
    const percent = signal.projected_percent ?? signal.used_percent;
    return `${Math.round(percent)}%`;
  }
  function DepletionForecastCard({
    forecast,
    title = "Depletion Forecast"
  }) {
    const primary = forecast.primary_signal;
    const primaryPercent = Math.max(0, primary.projected_percent ?? primary.used_percent);
    const primaryRunout = runoutLabel2(primary);
    const primaryRunoutTime = runoutTimeLabel2(primary);
    return /* @__PURE__ */ u4("div", { class: "card stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", style: { display: "grid", gap: "12px" }, children: [
      /* @__PURE__ */ u4("div", { children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", children: title }),
        /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "22px" }, children: primary.title }),
        /* @__PURE__ */ u4("div", { class: "stat-sub", children: forecast.summary_label })
      ] }),
      /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "6px" }, children: [
        /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "baseline" }, children: [
          /* @__PURE__ */ u4("span", { class: "stat-sub", children: primaryValueLabel(primary) }),
          primary.pace_label && /* @__PURE__ */ u4("span", { class: "stat-sub", children: primary.pace_label })
        ] }),
        /* @__PURE__ */ u4(
          SegmentedProgressBar,
          {
            value: Math.min(primaryPercent, 100),
            max: 100,
            status: severityToStatus3(forecast.severity),
            "aria-label": "Depletion forecast pressure"
          }
        ),
        /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "3px" }, children: [
          primaryRunout && /* @__PURE__ */ u4("div", { class: "stat-sub", children: primaryRunout }),
          primaryRunoutTime && /* @__PURE__ */ u4("div", { class: "stat-sub", children: primaryRunoutTime }),
          timingLabel(primary) && /* @__PURE__ */ u4("div", { class: "stat-sub", children: timingLabel(primary) }),
          remainingLabel(primary) && /* @__PURE__ */ u4("div", { class: "stat-sub", children: remainingLabel(primary) })
        ] })
      ] }),
      forecast.secondary_signals.length > 0 && /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "8px" }, children: [
        /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: "10px", letterSpacing: "0.08em" }, children: "SUPPORTING SIGNALS" }),
        forecast.secondary_signals.map((signal) => /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "2px" }, children: [
          /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "baseline" }, children: [
            /* @__PURE__ */ u4("span", { class: "stat-sub", children: signal.title }),
            /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "11px" }, children: supportValue(signal) })
          ] }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: [runoutLabel2(signal), timingLabel(signal), remainingLabel(signal)].filter(Boolean).join(" \xB7 ") })
        ] }, `${signal.kind}-${signal.title}`))
      ] }),
      forecast.note && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontStyle: "italic" }, children: forecast.note })
    ] }) });
  }

  // src/ui/components/PredictiveInsightsCard.tsx
  function fmtTokensPerMin(value) {
    return `${fmt(Math.round(value))}/min`;
  }
  function burnTone(tier) {
    if (tier === "high") return "var(--accent)";
    if (tier === "moderate") return "var(--warning)";
    return void 0;
  }
  function riskTone(level) {
    if (level === "high") return "var(--accent)";
    if (level === "elevated") return "var(--warning)";
    return void 0;
  }
  function EnvelopeRow({
    label,
    average,
    p50,
    p75,
    p90,
    p95,
    formatter
  }) {
    return /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "4px" }, children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "baseline" }, children: [
        /* @__PURE__ */ u4("span", { class: "stat-sub", children: label }),
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "11px" }, children: [
          "avg ",
          formatter(average)
        ] })
      ] }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "11px" }, children: [
        "P50 ",
        formatter(p50),
        " \xB7 P75 ",
        formatter(p75),
        " \xB7 P90 ",
        formatter(p90),
        " \xB7 P95 ",
        formatter(p95)
      ] })
    ] });
  }
  function HistoricalEnvelopeSection({ envelope }) {
    return /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "8px" }, children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "baseline" }, children: [
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontSize: "10px", letterSpacing: "0.08em" }, children: "HISTORICAL ENVELOPES" }),
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "11px" }, children: [
          envelope.sample_count,
          " completed blocks"
        ] })
      ] }),
      /* @__PURE__ */ u4(EnvelopeRow, { label: "Tokens", formatter: (value) => fmt(value), ...envelope.tokens }),
      /* @__PURE__ */ u4(EnvelopeRow, { label: "Cost", formatter: (value) => fmtCostCompact(value), ...envelope.cost_usd }),
      /* @__PURE__ */ u4(EnvelopeRow, { label: "Turns", formatter: (value) => fmt(value), ...envelope.turns })
    ] });
  }
  function RollingBurnSection({ burn }) {
    return /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "6px" }, children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "baseline" }, children: [
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontSize: "10px", letterSpacing: "0.08em" }, children: "ROLLING 1H BURN" }),
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { color: burnTone(String(burn.tier)) }, children: String(burn.tier).toUpperCase() })
      ] }),
      /* @__PURE__ */ u4("div", { style: { display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(120px,1fr))", gap: "8px" }, children: [
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Tokens" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "20px" }, children: fmtTokensPerMin(burn.tokens_per_min) })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Cost" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "20px" }, children: [
            fmtCostCompact(burn.cost_per_hour_nanos / 1e9),
            "/hr"
          ] })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Coverage" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "20px" }, children: [
            burn.coverage_minutes,
            "m"
          ] })
        ] })
      ] })
    ] });
  }
  function LimitHitSection({ analysis }) {
    return /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "6px" }, children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "baseline" }, children: [
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontSize: "10px", letterSpacing: "0.08em" }, children: "LIMIT-HIT RISK" }),
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { color: riskTone(analysis.risk_level) }, children: analysis.risk_level.toUpperCase() })
      ] }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", children: analysis.summary_label }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", fontSize: "11px" }, children: [
        analysis.hit_count,
        "/",
        analysis.sample_count,
        " hits \xB7 ",
        (analysis.hit_rate * 100).toFixed(0),
        "% rate \xB7 threshold ",
        fmt(analysis.threshold_tokens)
      ] })
    ] });
  }
  function PredictiveInsightsCard({
    insights,
    title = "Predictive Signals"
  }) {
    const hasAny = !!insights.rolling_hour_burn || !!insights.historical_envelope || !!insights.limit_hit_analysis;
    if (!hasAny) {
      return null;
    }
    return /* @__PURE__ */ u4("div", { class: "card stat-card", children: /* @__PURE__ */ u4("div", { class: "stat-content", style: { display: "grid", gap: "12px" }, children: [
      /* @__PURE__ */ u4("div", { children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", children: title }),
        /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Heuristic forecast from rolling burn and completed billing blocks." })
      ] }),
      insights.rolling_hour_burn && /* @__PURE__ */ u4(RollingBurnSection, { burn: insights.rolling_hour_burn }),
      insights.limit_hit_analysis && /* @__PURE__ */ u4(LimitHitSection, { analysis: insights.limit_hit_analysis }),
      insights.historical_envelope && /* @__PURE__ */ u4(HistoricalEnvelopeSection, { envelope: insights.historical_envelope })
    ] }) });
  }

  // src/ui/components/StatsCards.tsx
  function StatsCards({
    totals,
    daily,
    activeDays,
    activeDayTotalCostNanos,
    cacheEfficiency,
    billingBlocks,
    contextWindow
  }) {
    const avgPerActiveDay = (() => {
      if (activeDays === void 0 || activeDays === null) return "--";
      if (activeDays === 0) return "--";
      const totalUsd = (activeDayTotalCostNanos ?? 0) / 1e9;
      return fmtCostBig(totalUsd / activeDays);
    })();
    const activeDayTooltip = activeDays !== void 0 && activeDays !== null && activeDays > 0 ? `Averaged over ${activeDays} day${activeDays === 1 ? "" : "s"} with non-zero spend` : "No spend in selected period";
    const stats = [
      { label: "Sessions", value: totals.sessions.toLocaleString(), sub: "" },
      { label: "Turns", value: fmt(totals.turns), sub: "" },
      { label: "Input Tokens", value: fmt(totals.input), sub: "" },
      { label: "Output Tokens", value: fmt(totals.output), sub: "" },
      { label: "Cached Input", value: fmt(totals.cache_read), sub: "prompt cache" },
      { label: "Cache Creation", value: fmt(totals.cache_creation), sub: "cache writes" },
      { label: "Reasoning", value: fmt(totals.reasoning_output), sub: "subset of output" },
      { label: "Est. Cost", value: fmtCostBig(totals.cost), sub: "API pricing", isCost: true },
      // Amp credits: shown only when the current filter contains Amp rows
      // (totals.credits is null when no session in the filter has credits).
      ...totals.credits != null && totals.credits > 0 ? [{ label: "Total Credits", value: fmtCredits(totals.credits), sub: "Amp only (non-USD)" }] : []
    ];
    return /* @__PURE__ */ u4(S, { children: [
      stats.map((s4) => /* @__PURE__ */ u4("div", { class: "card stat-card", children: [
        /* @__PURE__ */ u4("div", { class: "stat-content", children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: s4.label }),
          /* @__PURE__ */ u4("div", { class: "stat-value", children: s4.value }),
          s4.sub ? /* @__PURE__ */ u4("div", { class: "stat-sub", children: s4.sub }) : null
        ] }),
        s4.isCost && daily && daily.length >= 2 ? /* @__PURE__ */ u4("div", { class: "stat-sparkline", children: /* @__PURE__ */ u4(Sparkline, { daily }) }) : null
      ] }, s4.label)),
      /* @__PURE__ */ u4("div", { class: "card stat-card", title: activeDayTooltip, children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", children: "Avg / Active Day" }),
        /* @__PURE__ */ u4("div", { class: "stat-value", children: avgPerActiveDay }),
        /* @__PURE__ */ u4("div", { class: "stat-sub", children: activeDays !== void 0 && activeDays !== null && activeDays > 0 ? `${activeDays} active ${activeDays === 1 ? "day" : "days"}` : "no spend" })
      ] }) }),
      billingBlocks && /* @__PURE__ */ u4(BillingBlocksCard, { data: billingBlocks }),
      billingBlocks?.depletion_forecast && /* @__PURE__ */ u4(DepletionForecastCard, { forecast: billingBlocks.depletion_forecast }),
      billingBlocks?.predictive_insights && /* @__PURE__ */ u4(PredictiveInsightsCard, { insights: billingBlocks.predictive_insights }),
      /* @__PURE__ */ u4(ContextWindowCard, { data: contextWindow ?? null }),
      cacheEfficiency && /* @__PURE__ */ u4(CacheEfficiencyCard, { data: cacheEfficiency })
    ] });
  }

  // src/ui/components/SubagentSummary.tsx
  function SubagentSummary({ summary }) {
    if (summary.subagent_turns === 0) return null;
    const totalInput = summary.parent_input + summary.subagent_input;
    const totalOutput = summary.parent_output + summary.subagent_output;
    const subPctInput = totalInput > 0 ? summary.subagent_input / totalInput * 100 : 0;
    const subPctOutput = totalOutput > 0 ? summary.subagent_output / totalOutput * 100 : 0;
    return /* @__PURE__ */ u4("div", { class: "table-card", children: [
      /* @__PURE__ */ u4("div", { class: "section-header", style: { padding: "20px 20px 0" }, children: /* @__PURE__ */ u4("div", { class: "section-title", style: { padding: "0" }, children: "Subagent Breakdown" }) }),
      /* @__PURE__ */ u4("div", { style: "display:grid;grid-template-columns:1fr 1fr 1fr;gap:16px;padding:12px 20px 20px", children: [
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Turns" }),
          /* @__PURE__ */ u4("div", { style: "font-size:15px", children: [
            "Parent: ",
            /* @__PURE__ */ u4("span", { class: "num", children: fmt(summary.parent_turns) })
          ] }),
          /* @__PURE__ */ u4("div", { style: "font-size:15px", children: [
            "Subagent: ",
            /* @__PURE__ */ u4("span", { class: "num", children: fmt(summary.subagent_turns) })
          ] }),
          /* @__PURE__ */ u4("div", { class: "sub", children: [
            summary.unique_agents,
            " unique agents"
          ] })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Input Tokens" }),
          /* @__PURE__ */ u4("div", { style: "font-size:15px", children: [
            "Parent: ",
            /* @__PURE__ */ u4("span", { class: "num", children: fmt(summary.parent_input) })
          ] }),
          /* @__PURE__ */ u4("div", { style: "font-size:15px", children: [
            "Subagent: ",
            /* @__PURE__ */ u4("span", { class: "num", children: fmt(summary.subagent_input) }),
            " (",
            subPctInput.toFixed(1),
            "%)"
          ] })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Output Tokens" }),
          /* @__PURE__ */ u4("div", { style: "font-size:15px", children: [
            "Parent: ",
            /* @__PURE__ */ u4("span", { class: "num", children: fmt(summary.parent_output) })
          ] }),
          /* @__PURE__ */ u4("div", { style: "font-size:15px", children: [
            "Subagent: ",
            /* @__PURE__ */ u4("span", { class: "num", children: fmt(summary.subagent_output) }),
            " (",
            subPctOutput.toFixed(1),
            "%)"
          ] })
        ] })
      ] })
    ] });
  }

  // src/ui/components/CodexPlanHistory.tsx
  var OPACITY_LADDER = [1, 0.65, 0.35, 0.2];
  function buildOptions(history2) {
    const rows2 = [...history2].reverse();
    if (rows2.length === 0) return null;
    const planSet = /* @__PURE__ */ new Set();
    for (const row of rows2) {
      for (const plan of Object.keys(row.by_plan)) {
        planSet.add(plan);
      }
      if (Object.keys(row.by_plan).length === 0 && row.primary_pct > 0) {
        const pt = row.snapshot?.plan_type ?? "unknown";
        planSet.add(pt);
      }
    }
    const plans = Array.from(planSet).sort();
    const categories = rows2.map((r4) => r4.day);
    const barSeries = plans.map((plan, i4) => {
      const opacity = OPACITY_LADDER[i4 % OPACITY_LADDER.length];
      return {
        name: plan.charAt(0).toUpperCase() + plan.slice(1),
        type: "bar",
        data: rows2.map((r4) => {
          const v4 = r4.by_plan[plan];
          return v4 != null ? Math.min(100, Math.max(0, v4)) : 0;
        }),
        color: `rgba(var(--text-primary-rgb, 232, 232, 232), ${opacity})`
      };
    });
    const secondarySeries = {
      name: "7d window",
      type: "line",
      data: rows2.map((r4) => r4.secondary_pct != null ? Math.min(100, Math.max(0, r4.secondary_pct)) : null),
      color: "var(--text-secondary, #888)"
    };
    const limitHitAnnotations = rows2.filter((r4) => r4.limit_hit_count > 0).map((r4) => ({
      x: r4.day,
      strokeDashArray: 0,
      borderColor: "var(--accent)",
      label: {
        text: `limit x${r4.limit_hit_count}`,
        style: {
          color: "var(--accent)",
          background: "transparent",
          fontFamily: "var(--font-mono)",
          fontSize: "10px"
        }
      }
    }));
    const allSeries = [...barSeries, secondarySeries];
    const opts = {
      chart: {
        type: "bar",
        stacked: true,
        toolbar: { show: false },
        animations: { enabled: false },
        fontFamily: "var(--font-mono)"
      },
      theme: { mode: "dark" },
      series: allSeries,
      colors: allSeries.map((s4) => s4.color),
      stroke: {
        width: allSeries.map((_4, i4) => i4 < barSeries.length ? 0 : 2),
        curve: "smooth",
        dashArray: allSeries.map((_4, i4) => i4 < barSeries.length ? 0 : 4)
      },
      fill: {
        type: allSeries.map(() => "solid"),
        opacity: allSeries.map(() => 1)
      },
      plotOptions: {
        bar: { columnWidth: "70%" }
      },
      grid: {
        borderColor: "var(--border)",
        strokeDashArray: 2,
        xaxis: { lines: { show: false } },
        yaxis: { lines: { show: true } }
      },
      legend: {
        position: "top",
        labels: { colors: "var(--text-secondary)", fontFamily: "var(--font-mono)" },
        itemMargin: { horizontal: 12, vertical: 4 }
      },
      xaxis: {
        categories,
        labels: {
          rotate: -45,
          style: {
            colors: "var(--text-secondary)",
            fontFamily: "var(--font-mono)",
            fontSize: "10px"
          }
        },
        axisBorder: { show: false },
        axisTicks: { show: false }
      },
      yaxis: [
        {
          min: 0,
          max: 100,
          labels: {
            style: {
              colors: "var(--text-secondary)",
              fontFamily: "var(--font-mono)",
              fontSize: "11px"
            },
            formatter: (v4) => `${v4.toFixed(0)}%`
          }
        },
        {
          opposite: true,
          min: 0,
          max: 100,
          show: false
        }
      ],
      tooltip: {
        theme: "dark",
        style: { fontFamily: "var(--font-mono)", fontSize: "11px" },
        y: {
          formatter: (val) => val != null && Number.isFinite(val) ? `${val.toFixed(1)}%` : "\u2014"
        }
      },
      dataLabels: { enabled: false },
      markers: { size: 0, strokeWidth: 0, hover: { size: 3 } },
      ...limitHitAnnotations.length > 0 ? { annotations: { xaxis: limitHitAnnotations } } : {}
    };
    return opts;
  }
  function CodexPlanHistory({ history: history2 }) {
    const options = T2(() => buildOptions(history2), [history2]);
    if (!options) return null;
    return /* @__PURE__ */ u4("div", { class: "card codex-plan-history-card", children: [
      /* @__PURE__ */ u4("div", { class: "codex-plan-history-header", children: /* @__PURE__ */ u4("div", { class: "codex-plan-history-title", children: "Codex plan utilisation (30 days)" }) }),
      /* @__PURE__ */ u4("div", { class: "codex-plan-history-body", children: /* @__PURE__ */ u4("div", { class: "chart-wrap tall", children: /* @__PURE__ */ u4(ApexChart, { options, id: "codex-plan-history-chart" }) }) })
    ] });
  }

  // src/ui/components/CodexPlanKpi.tsx
  function planLabel(planType) {
    if (!planType) return "Unknown";
    const s4 = planType.trim();
    if (!s4) return "Unknown";
    return s4.charAt(0).toUpperCase() + s4.slice(1).toLowerCase();
  }
  function creditState(snapshot) {
    const c4 = snapshot.credits;
    if (!c4) return "\u2014";
    if (c4.unlimited) return "Unlimited";
    if (c4.has_credits && c4.balance != null) return `$${c4.balance.toFixed(2)} balance`;
    if (c4.has_credits) return "Has credits";
    return "\u2014";
  }
  function progressClass(pct) {
    if (pct >= 85) return "codex-plan-progress codex-plan-progress--high";
    if (pct >= 60) return "codex-plan-progress codex-plan-progress--mid";
    return "codex-plan-progress codex-plan-progress--low";
  }
  function CodexPlanKpi({ today }) {
    const pct = Math.min(100, Math.max(0, today.primary?.used_percent ?? 0));
    const pctText = pct.toFixed(1) + "%";
    const plan = planLabel(today.plan_type);
    const credits = creditState(today);
    return /* @__PURE__ */ u4("div", { class: "card stat-card codex-plan-kpi", children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
      /* @__PURE__ */ u4("div", { class: "stat-label", children: "Codex plan" }),
      /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontFamily: "var(--font-mono)" }, children: esc(plan) }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", children: /* @__PURE__ */ u4(
        "span",
        {
          class: progressClass(pct),
          style: { width: `${pct}%` },
          "aria-valuenow": pct,
          "aria-valuemin": 0,
          "aria-valuemax": 100,
          role: "progressbar",
          "aria-label": "5h window usage"
        }
      ) }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontFamily: "var(--font-mono)", marginTop: "4px" }, children: [
        pctText,
        " \xB7 5h window"
      ] }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", children: esc(credits) })
    ] }) });
  }

  // src/ui/components/SubscriptionQuotaCard.tsx
  var PROVIDER_LABELS = {
    claude: "Claude",
    codex: "Codex"
  };
  function providerTitle(provider) {
    return PROVIDER_LABELS[provider] ?? fmtLabel(provider);
  }
  function fmtConfidence(c4) {
    if (c4 >= 0.85) return "high confidence";
    if (c4 >= 0.5) return "medium confidence";
    return "low confidence";
  }
  function SubscriptionQuotaCard({ snapshot }) {
    const title = providerTitle(snapshot.provider);
    const planLabel2 = snapshot.published.plan_label ? fmtLabel(snapshot.published.plan_label) : "\u2014";
    const estimated = snapshot.estimated?.windows ?? [];
    const hasEstimates = estimated.length > 0;
    return /* @__PURE__ */ u4("div", { class: "card subscription-quota-card", children: [
      /* @__PURE__ */ u4("div", { class: "subscription-quota-header", children: [
        /* @__PURE__ */ u4("div", { class: "subscription-quota-title", children: title }),
        /* @__PURE__ */ u4("div", { class: "subscription-quota-plan", title: `Source: ${snapshot.source_used}`, children: planLabel2 })
      ] }),
      /* @__PURE__ */ u4("div", { class: "subscription-quota-section", children: [
        /* @__PURE__ */ u4("div", { class: "subscription-quota-section-label", children: "Published" }),
        snapshot.published.windows.length === 0 && /* @__PURE__ */ u4("div", { class: "subscription-quota-empty", children: "No active windows reported." }),
        snapshot.published.windows.map((window2) => {
          const pct = Math.min(100, Math.max(0, window2.used_percent));
          return /* @__PURE__ */ u4("div", { class: "subscription-quota-row", children: [
            /* @__PURE__ */ u4("div", { class: "subscription-quota-row-label", children: window2.label }),
            /* @__PURE__ */ u4("div", { class: "subscription-quota-row-value", children: [
              pct.toFixed(1),
              "%"
            ] }),
            /* @__PURE__ */ u4("div", { class: "subscription-quota-row-bar", children: /* @__PURE__ */ u4(
              SegmentedProgressBar,
              {
                value: pct,
                max: 100,
                size: "standard",
                "aria-label": `${window2.label} usage`
              }
            ) }),
            /* @__PURE__ */ u4("div", { class: "subscription-quota-row-sub", children: window2.resets_in_minutes != null ? `Resets in ${fmtResetTime(window2.resets_in_minutes)}` : window2.resets_at ?? "" })
          ] }, `pub-${window2.kind}`);
        }),
        snapshot.published.budget && /* @__PURE__ */ u4("div", { class: "subscription-quota-row", children: [
          /* @__PURE__ */ u4("div", { class: "subscription-quota-row-label", children: "Monthly $-budget" }),
          /* @__PURE__ */ u4("div", { class: "subscription-quota-row-value", children: [
            "$",
            snapshot.published.budget.used_usd.toFixed(2)
          ] }),
          /* @__PURE__ */ u4("div", { class: "subscription-quota-row-bar", children: /* @__PURE__ */ u4(
            SegmentedProgressBar,
            {
              value: snapshot.published.budget.utilization,
              max: 100,
              size: "standard",
              "aria-label": "Budget usage"
            }
          ) }),
          /* @__PURE__ */ u4("div", { class: "subscription-quota-row-sub", children: [
            "of $",
            snapshot.published.budget.limit_usd.toFixed(2),
            " (",
            snapshot.published.budget.currency,
            ")"
          ] })
        ] }, "pub-budget")
      ] }),
      /* @__PURE__ */ u4("div", { class: "subscription-quota-divider" }),
      /* @__PURE__ */ u4("div", { class: "subscription-quota-section", children: [
        /* @__PURE__ */ u4("div", { class: "subscription-quota-section-label", children: "Estimated" }),
        !hasEstimates && /* @__PURE__ */ u4("div", { class: "subscription-quota-empty", children: "Insufficient data \u2014 utilization too low to derive a token cap." }),
        estimated.map((w5) => {
          const dim = w5.confidence < 0.3;
          const headlineCap = w5.smoothed_cap_tokens != null ? w5.smoothed_cap_tokens : w5.estimated_cap_tokens;
          const shiftGlyph = w5.cap_shift === "increase" ? "\u2191 " : w5.cap_shift === "decrease" ? "\u2193 " : "";
          const shiftClass = w5.cap_shift === "decrease" ? "subscription-quota-shift subscription-quota-shift-down" : w5.cap_shift === "increase" ? "subscription-quota-shift subscription-quota-shift-up" : "";
          const subParts = [
            `from ${fmt(w5.observed_tokens)} observed`,
            fmtConfidence(w5.confidence)
          ];
          if (w5.sample_count != null && w5.sample_count > 0) {
            subParts.push(`n=${w5.sample_count}`);
          }
          return /* @__PURE__ */ u4(
            "div",
            {
              class: "subscription-quota-row subscription-quota-row-estimated",
              style: dim ? { opacity: 0.6 } : void 0,
              children: [
                /* @__PURE__ */ u4("div", { class: "subscription-quota-row-label", children: w5.label }),
                /* @__PURE__ */ u4("div", { class: "subscription-quota-row-value", children: [
                  shiftGlyph && /* @__PURE__ */ u4("span", { class: shiftClass, children: shiftGlyph }),
                  "~",
                  fmt(headlineCap),
                  " tokens"
                ] }),
                /* @__PURE__ */ u4("div", { class: "subscription-quota-row-sub", children: subParts.join(" \xB7 ") })
              ]
            },
            `est-${w5.kind}`
          );
        })
      ] })
    ] });
  }

  // src/ui/lib/colors.ts
  function resolveCssVar(name, fallback) {
    if (typeof window === "undefined") return fallback;
    const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return value || fallback;
  }

  // src/ui/components/SubscriptionHistoryChart.tsx
  var WINDOW_LABELS = {
    five_hour: "Claude \xB7 5h",
    seven_day: "Claude \xB7 weekly",
    seven_day_opus: "Claude \xB7 weekly Opus",
    seven_day_sonnet: "Claude \xB7 weekly Sonnet",
    codex_primary: "Codex \xB7 primary",
    codex_secondary: "Codex \xB7 secondary"
  };
  var DASH_LADDER = [0, 3, 6, 9, 12, 15];
  function inferProvider(windowType) {
    return windowType.startsWith("codex_") ? "codex" : "claude";
  }
  function buildOptions2(history2, changelog, provider) {
    const filtered = history2.filter((row) => {
      if (row.estimated_cap_tokens == null) return false;
      if (provider === "all") return true;
      return inferProvider(row.window_type) === provider;
    });
    if (filtered.length === 0) return null;
    const seriesMap = /* @__PURE__ */ new Map();
    for (const row of filtered) {
      if (row.estimated_cap_tokens == null) continue;
      const ts = Date.parse(row.timestamp);
      if (Number.isNaN(ts)) continue;
      let arr = seriesMap.get(row.window_type);
      if (!arr) {
        arr = [];
        seriesMap.set(row.window_type, arr);
      }
      arr.push({ x: ts, y: row.estimated_cap_tokens });
    }
    if (seriesMap.size === 0) return null;
    const seriesKeys = Array.from(seriesMap.keys()).sort();
    const series = seriesKeys.map((key) => ({
      name: WINDOW_LABELS[key] ?? key,
      data: (seriesMap.get(key) ?? []).sort((a4, b4) => a4.x - b4.x)
    }));
    const dashArray = seriesKeys.map(
      (_4, i4) => DASH_LADDER[i4 % DASH_LADDER.length] ?? 0
    );
    const textPrimary = resolveCssVar("--text-primary", "#0a0a0a");
    const textSecondary = resolveCssVar("--text-secondary", "#666666");
    const borderColor = resolveCssVar("--border", "#e0e0e0");
    const annotationsX = changelog.filter((entry) => provider === "all" || entry.provider === provider).map((entry) => ({
      x: Date.parse(`${entry.date}T12:00:00Z`),
      borderColor: textSecondary,
      strokeDashArray: 3
    })).filter((a4) => Number.isFinite(a4.x));
    const seriesXValues = [];
    for (const s4 of series) {
      for (const p5 of s4.data) seriesXValues.push(p5.x);
    }
    const annotationXValues = annotationsX.map((a4) => a4.x);
    const allX = [...seriesXValues, ...annotationXValues];
    const xMin = allX.length ? Math.min(...allX) : void 0;
    const xMax = allX.length ? Math.max(...allX, Date.now()) : void 0;
    const opts = {
      chart: {
        type: "line",
        toolbar: { show: false },
        animations: { enabled: false },
        fontFamily: "var(--font-mono)",
        background: "transparent"
      },
      // No `theme: { mode: 'dark' }` — the chart inherits the surrounding card
      // via transparent background + CSS-variable colours, so it works in both
      // light and dark dashboard themes.
      series,
      colors: series.map(() => textPrimary),
      stroke: {
        width: 2,
        curve: "smooth",
        dashArray
      },
      fill: { type: "solid", opacity: 0 },
      grid: {
        borderColor,
        strokeDashArray: 2,
        xaxis: { lines: { show: false } },
        yaxis: { lines: { show: true } }
      },
      legend: {
        position: "top",
        labels: { colors: textPrimary, fontFamily: "var(--font-mono)" },
        itemMargin: { horizontal: 12, vertical: 4 },
        markers: { width: 20, height: 2, radius: 0 }
      },
      xaxis: {
        type: "datetime",
        ...xMin !== void 0 ? { min: xMin } : {},
        ...xMax !== void 0 ? { max: xMax } : {},
        labels: {
          style: { colors: textSecondary, fontFamily: "var(--font-mono)", fontSize: "11px" }
        },
        axisBorder: { show: false },
        axisTicks: { show: false }
      },
      yaxis: {
        labels: {
          style: { colors: textSecondary, fontFamily: "var(--font-mono)", fontSize: "11px" },
          formatter: (val) => {
            if (!Number.isFinite(val)) return "";
            if (val >= 1e9) return `${(val / 1e9).toFixed(2)}B`;
            if (val >= 1e6) return `${(val / 1e6).toFixed(2)}M`;
            if (val >= 1e3) return `${(val / 1e3).toFixed(0)}K`;
            return String(val);
          }
        }
      },
      tooltip: {
        style: { fontFamily: "var(--font-mono)", fontSize: "11px" },
        y: {
          formatter: (val) => Number.isFinite(val) ? `${val.toLocaleString("en-US")} tokens` : "\u2014"
        }
      },
      // Show data points as small markers — line strokes can't render with a
      // single observation per series (which is the common case for users with
      // only a few days of local history).
      markers: { size: 3, strokeWidth: 0, hover: { size: 5 } },
      dataLabels: { enabled: false }
    };
    if (annotationsX.length > 0) {
      opts.annotations = { xaxis: annotationsX };
    }
    return opts;
  }
  function SubscriptionHistoryChart({ history: history2, changelog }) {
    const [provider, setProvider] = d2("all");
    const options = T2(
      () => buildOptions2(history2, changelog, provider),
      [history2, changelog, provider]
    );
    return /* @__PURE__ */ u4("div", { class: "card subscription-history-card", children: [
      /* @__PURE__ */ u4("div", { class: "subscription-history-header", children: [
        /* @__PURE__ */ u4("div", { class: "subscription-history-title", children: "Subscription cap history" }),
        /* @__PURE__ */ u4("div", { class: "subscription-history-filter", children: ["all", "claude", "codex"].map((p5) => /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: `chip${provider === p5 ? " chip-active" : ""}`,
            onClick: () => setProvider(p5),
            children: p5 === "all" ? "All" : p5 === "claude" ? "Claude" : "Codex"
          },
          p5
        )) })
      ] }),
      /* @__PURE__ */ u4("div", { class: "subscription-history-body", children: options ? /* @__PURE__ */ u4("div", { class: "chart-wrap tall", children: /* @__PURE__ */ u4(ApexChart, { options, id: "subscription-history-chart" }) }) : /* @__PURE__ */ u4("div", { class: "subscription-quota-empty", children: "No historical observations yet \u2014 caps will appear once snapshots accumulate." }) }),
      changelog.length > 0 && /* @__PURE__ */ u4("ul", { class: "subscription-history-changelog", children: changelog.map((entry) => /* @__PURE__ */ u4("li", { children: [
        /* @__PURE__ */ u4("span", { class: "subscription-history-date", children: entry.date }),
        /* @__PURE__ */ u4("span", { class: "subscription-history-provider", children: entry.provider }),
        /* @__PURE__ */ u4("a", { class: "subscription-history-link", href: entry.source_url, target: "_blank", rel: "noreferrer", children: entry.title }),
        /* @__PURE__ */ u4("span", { class: "subscription-history-desc", children: entry.description })
      ] }, `${entry.date}-${entry.provider}-${entry.kind}`)) })
    ] });
  }

  // src/ui/components/tables/ToolUsageTable.tsx
  function makeColumns3(data) {
    const maxInvocations = data.reduce((m5, r4) => Math.max(m5, r4.invocations), 0);
    return [
      {
        accessorKey: "provider",
        header: "Provider",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
      },
      {
        accessorKey: "tool_name",
        header: "Tool",
        cell: ({ row }) => {
          const cat = row.original.category;
          const badge = cat === "mcp" ? "mcp" : "builtin";
          return /* @__PURE__ */ u4("span", { children: [
            /* @__PURE__ */ u4("span", { class: `model-tag ${badge}`, children: cat }),
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
          return v4 ? /* @__PURE__ */ u4("span", { class: "muted", children: v4 }) : /* @__PURE__ */ u4("span", { class: "muted", children: "--" });
        }
      },
      {
        accessorKey: "invocations",
        header: "Calls",
        cell: ({ getValue }) => /* @__PURE__ */ u4(
          InlineRankBar,
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
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "sessions_used",
        header: "Sessions",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
      },
      {
        accessorKey: "errors",
        header: "Errors",
        cell: ({ row }) => {
          const e4 = row.original.errors;
          if (!e4) return /* @__PURE__ */ u4("span", { class: "dim", children: "0" });
          const pct = row.original.invocations > 0 ? (e4 / row.original.invocations * 100).toFixed(1) : "0";
          const href = `/tool-errors?tool=${encodeURIComponent(row.original.tool_name)}&provider=${encodeURIComponent(row.original.provider)}&range=${selectedRange.value}`;
          return /* @__PURE__ */ u4(
            "button",
            {
              type: "button",
              class: "table-action-btn",
              style: { color: "var(--accent)", fontFamily: "var(--font-mono)", fontVariantNumeric: "tabular-nums" },
              onClick: () => {
                window.location.href = href;
              },
              title: "View error details",
              children: [
                e4,
                " (",
                pct,
                "%)"
              ]
            }
          );
        }
      }
    ];
  }
  function ToolUsageTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u4(DataTable, { columns: makeColumns3(data), data, title: "Tool Usage", sectionKey: "tool-summary" });
  }

  // src/ui/components/charts/VersionDonut.tsx
  var METRIC_LABELS3 = {
    cost: "Cost",
    calls: "Calls",
    tokens: "Tokens"
  };
  var METRIC_OPTIONS2 = ["cost", "calls", "tokens"];
  function getMetricValue2(row, metric) {
    switch (metric) {
      case "cost":
        return row.cost;
      case "calls":
        return row.turns;
      case "tokens":
        return row.tokens;
    }
  }
  function formatMetricValue2(value, metric, large = false) {
    switch (metric) {
      case "cost":
        return large ? fmtCostCompact(value) : fmtCost(value);
      case "calls":
      case "tokens":
        return fmt(value);
    }
  }
  function VersionDonut({ rows: rows2, metric, onMetricChange }) {
    const normalized = rows2.map((r4) => ({
      ...r4,
      version: r4.version === "" || r4.version === "unknown" ? "(unknown)" : r4.version
    }));
    return MetricDonut({
      rows: normalized,
      metric,
      metricOptions: METRIC_OPTIONS2,
      metricLabel: (m5) => METRIC_LABELS3[m5],
      metricValue: getMetricValue2,
      metricFormat: formatMetricValue2,
      rowLabel: (row) => row.version,
      rowCost: (row) => row.cost,
      rowCalls: (row) => row.turns,
      rowTokens: (row) => row.tokens,
      id: "chart-version-donut",
      centerKickerPrefix: "Total",
      onMetricChange,
      showLegend: false,
      formatCost: (v4) => fmtCost(v4),
      formatCalls: (v4) => fmt(v4),
      formatTokens: (v4) => fmt(v4)
    });
  }

  // src/ui/components/tables/VersionTable.tsx
  var columns6 = [
    {
      accessorKey: "provider",
      header: "Provider",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(getValue()).toUpperCase() })
    },
    {
      accessorKey: "version",
      header: "Version",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "model-tag", children: String(getValue()) })
    },
    {
      accessorKey: "turns",
      header: "Turns",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: fmt(getValue()) })
    },
    {
      accessorKey: "sessions",
      header: "Sessions",
      cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num", children: Number(getValue() ?? 0) })
    }
  ];
  function VersionTable({ data, title = "CLI Versions" }) {
    if (!data.length) return null;
    if (title == null) {
      return /* @__PURE__ */ u4(DataTable, { columns: columns6, data });
    }
    return /* @__PURE__ */ u4(DataTable, { columns: columns6, data, title });
  }

  // src/ui/components/charts/WeeklyChart.tsx
  function WeeklyChart({ weekly }) {
    if (!weekly?.length) {
      return /* @__PURE__ */ u4("div", { style: { padding: "24px", color: "var(--text-muted)", fontFamily: "var(--font-mono)", fontSize: "12px" }, children: "No weekly data available." });
    }
    const base = dashboardChartOptions("bar");
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
        ...base.xaxis ?? {},
        categories: weekly.map((w5) => w5.week),
        labels: { ...base.xaxis?.labels ?? {}, rotate: -45, maxHeight: 60 },
        tickAmount: Math.min(weekly.length, 26)
      },
      yaxis: {
        ...base.yaxis ?? {},
        labels: { ...base.yaxis?.labels ?? {}, formatter: (v4) => fmt(v4) }
      },
      tooltip: {
        ...base.tooltip,
        y: { formatter: (v4) => fmt(v4) + " tokens" },
        custom: ({ dataPointIndex }) => {
          const w5 = weekly[dataPointIndex];
          if (!w5) return "";
          const total2 = w5.input + w5.output + w5.cache_read + w5.cache_creation;
          const costUsd = w5.cost_nanos / 1e9;
          const costStr = costUsd < 1e-4 ? "<$0.0001" : "$" + costUsd.toFixed(4);
          return '<div style="padding:8px 12px;font-family:var(--font-mono);font-size:12px;background:var(--surface);border:1px solid var(--border)"><div style="margin-bottom:4px;font-weight:600">' + esc(w5.week) + "</div><div>Input: " + fmt(w5.input) + "</div><div>Output: " + fmt(w5.output) + "</div><div>Cached Input: " + fmt(w5.cache_read) + "</div><div>Cache Creation: " + fmt(w5.cache_creation) + '</div><div style="margin-top:4px;border-top:1px solid var(--border);padding-top:4px">Total: ' + fmt(total2) + " tokens</div><div>Cost: " + costStr + "</div></div>";
        }
      }
    };
    return /* @__PURE__ */ u4(ApexChart, { options, id: "chart-weekly" });
  }

  // src/ui/lib/widget-overrides.ts
  var STORAGE_KEY = "heimdall.widget-empty-overrides";
  function readStorage() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) return /* @__PURE__ */ new Set();
      const parsed = JSON.parse(raw);
      if (!Array.isArray(parsed)) return /* @__PURE__ */ new Set();
      return new Set(parsed.filter((x4) => typeof x4 === "string"));
    } catch {
      return /* @__PURE__ */ new Set();
    }
  }
  function writeStorage(set) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify([...set]));
    } catch {
    }
  }
  var widgetEmptyOverrides = y3(readStorage());
  function isOverridden(widgetId) {
    return widgetEmptyOverrides.value.has(widgetId);
  }
  function setOverride(widgetId, on) {
    const next = new Set(widgetEmptyOverrides.value);
    if (on) next.add(widgetId);
    else next.delete(widgetId);
    widgetEmptyOverrides.value = next;
    writeStorage(next);
    const itemEl = document.querySelector(
      `.grid-stack-item[gs-id="${widgetId}"]`
    );
    if (!itemEl) return;
    if (on) {
      itemEl.style.display = "";
    } else {
      const inner = itemEl.querySelector(
        `[id="${widgetId}"], .widget-body > [id]`
      );
      const hasContent = inner?.dataset["hasContent"] === "1";
      itemEl.style.display = hasContent ? "" : "none";
    }
  }

  // src/ui/dashboard/aggregation.ts
  function formatLocalDate(date) {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
  }
  function getRangeCutoff(range) {
    if (range === "all") return null;
    const days = range === "7d" ? 7 : range === "30d" ? 30 : 90;
    const date = /* @__PURE__ */ new Date();
    date.setDate(date.getDate() - days);
    return formatLocalDate(date);
  }
  function weekLabelToWeekStart(label) {
    const [yearStr, weekStr] = label.split("-");
    const year = parseInt(yearStr ?? "", 10);
    const week = parseInt(weekStr ?? "", 10);
    if (!Number.isFinite(year) || !Number.isFinite(week)) return /* @__PURE__ */ new Date(NaN);
    const jan1 = new Date(Date.UTC(year, 0, 1));
    if (week === 0) return jan1;
    const jan1Dow = jan1.getUTCDay();
    const daysToFirstMon = (8 - jan1Dow) % 7;
    const firstMondayUtc = new Date(Date.UTC(year, 0, 1 + daysToFirstMon));
    return new Date(firstMondayUtc.getTime() + (week - 1) * 7 * 86400 * 1e3);
  }
  function buildWeeklyAgg(rows2, selectedModels2, range) {
    if (!rows2.length) return [];
    const cutoff = getRangeCutoff(range);
    const weekMap = {};
    for (const row of rows2) {
      if (!selectedModels2.has(row.model)) continue;
      if (cutoff) {
        const weekStart = weekLabelToWeekStart(row.week);
        if (Number.isNaN(weekStart.getTime())) continue;
        if (weekStart.toISOString().slice(0, 10) < cutoff) continue;
      }
      const weekly = weekMap[row.week] ?? (weekMap[row.week] = {
        week: row.week,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        cost_nanos: 0
      });
      weekly.input += row.input_tokens;
      weekly.output += row.output_tokens;
      weekly.cache_read += row.cache_read_tokens;
      weekly.cache_creation += row.cache_creation_tokens;
      weekly.reasoning_output += row.reasoning_output_tokens;
      weekly.cost_nanos += row.cost_nanos;
    }
    return Object.values(weekMap).sort((left, right) => left.week.localeCompare(right.week));
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
  function buildAggregations(filteredDaily, filteredSessions) {
    const dailyMap = {};
    for (const row of filteredDaily) {
      const daily2 = dailyMap[row.day] ?? (dailyMap[row.day] = {
        day: row.day,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        cost: 0
      });
      daily2.input += row.input;
      daily2.output += row.output;
      daily2.cache_read += row.cache_read;
      daily2.cache_creation += row.cache_creation;
      daily2.reasoning_output += row.reasoning_output;
      daily2.cost += row.cost;
    }
    const daily = Object.values(dailyMap).sort((left, right) => left.day.localeCompare(right.day));
    const modelMap = {};
    for (const row of filteredDaily) {
      const model = modelMap[row.model] ?? (modelMap[row.model] = {
        model: row.model,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 0,
        sessions: 0,
        cost: 0,
        is_billable: row.cost > 0,
        input_cost: 0,
        output_cost: 0,
        cache_read_cost: 0,
        cache_write_cost: 0,
        credits: null
      });
      model.input += row.input;
      model.output += row.output;
      model.cache_read += row.cache_read;
      model.cache_creation += row.cache_creation;
      model.reasoning_output += row.reasoning_output;
      model.turns += row.turns;
      model.cost += row.cost;
      if (row.cost > 0) model.is_billable = true;
      model.input_cost = (model.input_cost ?? 0) + (row.input_cost ?? 0);
      model.output_cost = (model.output_cost ?? 0) + (row.output_cost ?? 0);
      model.cache_read_cost = (model.cache_read_cost ?? 0) + (row.cache_read_cost ?? 0);
      model.cache_write_cost = (model.cache_write_cost ?? 0) + (row.cache_write_cost ?? 0);
      if (row.credits != null) {
        model.credits = (model.credits ?? 0) + row.credits;
      }
    }
    for (const session of filteredSessions) {
      const model = modelMap[session.model];
      if (model) model.sessions += 1;
    }
    const byModel = Object.values(modelMap).sort(
      (left, right) => right.input + right.output - (left.input + left.output)
    );
    const projectMap = {};
    for (const session of filteredSessions) {
      const project = projectMap[session.project] ?? (projectMap[session.project] = {
        project: session.project,
        display_name: session.display_name || session.project,
        input: 0,
        output: 0,
        cache_read: 0,
        cache_creation: 0,
        reasoning_output: 0,
        turns: 0,
        sessions: 0,
        cost: 0,
        credits: null,
        pinned: session.pinned ?? false,
        custom_label: session.custom_label ?? null
      });
      project.input += session.input;
      project.output += session.output;
      project.cache_read += session.cache_read;
      project.cache_creation += session.cache_creation;
      project.reasoning_output += session.reasoning_output;
      project.turns += session.turns;
      project.sessions += 1;
      project.cost += session.cost;
      if (session.credits != null) {
        project.credits = (project.credits ?? 0) + session.credits;
      }
      if (session.pinned) project.pinned = true;
      if (session.custom_label != null) project.custom_label = session.custom_label;
    }
    const byProject = Object.values(projectMap).sort((left, right) => {
      if ((left.pinned ?? false) !== (right.pinned ?? false)) {
        return (right.pinned ? 1 : 0) - (left.pinned ? 1 : 0);
      }
      return right.input + right.output - (left.input + left.output);
    });
    const totals = {
      sessions: filteredSessions.length,
      turns: filteredSessions.reduce((sum2, session) => sum2 + session.turns, 0),
      input: filteredSessions.reduce((sum2, session) => sum2 + session.input, 0),
      output: filteredSessions.reduce((sum2, session) => sum2 + session.output, 0),
      cache_read: filteredSessions.reduce((sum2, session) => sum2 + session.cache_read, 0),
      cache_creation: filteredSessions.reduce((sum2, session) => sum2 + session.cache_creation, 0),
      reasoning_output: filteredSessions.reduce((sum2, session) => sum2 + session.reasoning_output, 0),
      cost: filteredSessions.reduce((sum2, session) => sum2 + session.cost, 0),
      credits: filteredSessions.reduce((sum2, session) => {
        if (session.credits == null) return sum2;
        return (sum2 ?? 0) + session.credits;
      }, null)
    };
    const confidenceBreakdown = Object.entries(
      filteredSessions.reduce((acc, session) => {
        const key = session.cost_confidence || "low";
        if (!acc[key]) acc[key] = { sessions: 0, cost: 0 };
        acc[key].sessions += 1;
        acc[key].cost += session.cost;
        return acc;
      }, {})
    ).sort(([left], [right]) => confidenceRank(left) - confidenceRank(right));
    const billingModeBreakdown = Object.entries(
      filteredSessions.reduce((acc, session) => {
        const key = session.billing_mode || "estimated_local";
        if (!acc[key]) acc[key] = { sessions: 0, cost: 0 };
        acc[key].sessions += 1;
        acc[key].cost += session.cost;
        return acc;
      }, {})
    ).sort((left, right) => right[1].sessions - left[1].sessions);
    const pricingVersions = Array.from(
      new Set(filteredSessions.map((session) => session.pricing_version).filter(Boolean))
    );
    return {
      daily,
      byModel,
      byProject,
      totals,
      confidenceBreakdown,
      billingModeBreakdown,
      pricingVersions
    };
  }

  // src/ui/dashboard/view.tsx
  var SECTION_TAB_MAP = {
    "usage-windows": "overview",
    "subscription-quota": "overview",
    "claude-usage": "overview",
    "agent-status": "overview",
    "estimation-meta": "overview",
    "official-sync": "overview",
    "openai-reconciliation": "overview",
    "subagent-reconciliation": "overview",
    "stats-row": "overview",
    "codex-plan-kpi-mount": "overview",
    "codex-plan-history-mount": "activity",
    "daily-chart-card": "activity",
    "model-chart-card": "activity",
    "project-chart-card": "activity",
    "hourly-chart": "activity",
    "activity-heatmap": "activity",
    "subagent-summary": "agents",
    "agent-setup-banner": "agents",
    "agent-kpis-row": "agents",
    "agent-timeline": "agents",
    "agent-distribution": "agents",
    "agent-top-sessions": "agents",
    "agent-spawn-batches": "agents",
    "agent-tool-spectrum": "agents",
    "entrypoint-breakdown": "breakdowns",
    "service-tiers": "breakdowns",
    "tool-summary": "breakdowns",
    "mcp-summary": "breakdowns",
    "branch-summary": "breakdowns",
    "version-summary": "breakdowns",
    "cost-reconciliation": "breakdowns",
    "model-cost-mount": "tables",
    "sessions-mount": "tables",
    "project-cost-mount": "tables",
    "projects-registry": "projects",
    "today-date-picker-mount": "today",
    "today-kpis-mount": "today",
    "today-hour-timeline-mount": "today",
    "today-hour-heatstrip-mount": "today",
    "today-days-hours-30-mount": "today",
    "today-days-hours-7-mount": "today",
    "today-weekday-hour-mount": "today"
  };
  var SECTION_DISPLAY_MODE = {
    "usage-windows": "grid",
    "subscription-quota": "block",
    "agent-status": "grid",
    "estimation-meta": "grid",
    "stats-row": "grid",
    "agent-kpis-row": "grid",
    "codex-plan-kpi-mount": "grid",
    "today-kpis-mount": "grid"
  };
  function matchesProvider(row) {
    if (selectedProvider.value === "both") return true;
    return row.provider === selectedProvider.value;
  }
  function matchesProjectSearch(project, displayName) {
    const query = projectSearchQuery.value;
    if (!query) return true;
    if (project.toLowerCase().includes(query)) return true;
    if (displayName && displayName.toLowerCase().includes(query)) return true;
    return false;
  }
  function setSectionVisibility(sectionId, hasContent, displayMode = "") {
    const container = $2(sectionId);
    if (!container) return;
    container.dataset["hasContent"] = hasContent ? "1" : "0";
    const widgetBody = container.closest(".widget-body") ?? (container.closest(".grid-stack-item") ? container : null);
    if (widgetBody) {
      delete widgetBody.dataset["loading"];
      const gridItem = widgetBody.closest(".grid-stack-item");
      if (gridItem) {
        const widgetId = gridItem.getAttribute("gs-id") ?? "";
        const overridden = widgetId ? isOverridden(widgetId) : false;
        gridItem.style.display = hasContent || overridden ? "" : "none";
      }
      return;
    }
    const visibleInTab = SECTION_TAB_MAP[sectionId] === activeDashboardTab.value;
    container.style.display = hasContent && visibleInTab ? displayMode : "none";
  }
  function renderSection(mountId, hasContent, element, displayMode) {
    const container = $2(mountId);
    if (!container) return;
    setSectionVisibility(mountId, hasContent, displayMode ?? "");
    R(hasContent ? element : null, container);
    if (hasContent && container.childElementCount === 0) {
      const widgetBody = container.closest(".widget-body");
      if (widgetBody) {
        const gridItem = widgetBody.closest(".grid-stack-item");
        if (gridItem) gridItem.style.display = "none";
      }
    }
  }
  function refreshSectionVisibility() {
    for (const [sectionId, tab] of Object.entries(SECTION_TAB_MAP)) {
      const container = $2(sectionId);
      if (!container) continue;
      const tabMatches = tab === activeDashboardTab.value;
      if (container.closest(".grid-stack-item")) {
        const widgetBody = container.closest(".widget-body") ?? container;
        const gridItem = widgetBody.closest(".grid-stack-item");
        if (gridItem) {
          if (!tabMatches) {
            gridItem.style.display = "none";
          } else {
            const hasContent2 = container.dataset["hasContent"] !== "0";
            const widgetId = gridItem.getAttribute("gs-id") ?? "";
            const overridden = widgetId ? isOverridden(widgetId) : false;
            if (hasContent2 || overridden) gridItem.style.display = "";
          }
        }
        continue;
      }
      const hasContent = container.dataset["hasContent"] !== "0";
      const displayMode = SECTION_DISPLAY_MODE[sectionId] ?? "";
      container.style.display = hasContent && tabMatches ? displayMode : "none";
    }
  }
  function clearTodayWidgets() {
    for (const id of Object.keys(SECTION_TAB_MAP).filter((k4) => SECTION_TAB_MAP[k4] === "today")) {
      setSectionVisibility(id, false);
    }
  }
  function renderEstimationMeta(confidenceBreakdown, billingModeBreakdown, pricingVersions) {
    const container = $2("estimation-meta");
    if (!container) return;
    if (!confidenceBreakdown.length && !billingModeBreakdown.length && !pricingVersions.length) {
      setSectionVisibility("estimation-meta", false, "grid");
      R(null, container);
      return;
    }
    setSectionVisibility("estimation-meta", true, "grid");
    R(
      /* @__PURE__ */ u4(
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
    renderSection(
      "openai-reconciliation",
      !!reconciliation?.available,
      /* @__PURE__ */ u4(ReconciliationBlock, { reconciliation })
    );
  }
  function renderSubagentReconciliation(reconciliation) {
    renderSection(
      "subagent-reconciliation",
      !!reconciliation?.available,
      /* @__PURE__ */ u4(SubagentReconciliationBlock, { reconciliation })
    );
  }
  function renderOfficialSync(summary) {
    renderSection(
      "official-sync",
      !!summary?.available,
      /* @__PURE__ */ u4(OfficialSyncPanel, { summary, providerFilter: selectedProvider.value })
    );
  }
  function renderAgentTelemetry(data) {
    const { agent_telemetry } = data;
    const totalCostUsd = data.provider_breakdown.reduce((s4, p5) => s4 + p5.cost, 0);
    const hasAgentActivity = agent_telemetry.totals.sessions > 0;
    const bannerContainer = $2("agent-setup-banner");
    if (bannerContainer) {
      setSectionVisibility("agent-setup-banner", true);
      R(/* @__PURE__ */ u4(AgentSetupBanner, { telemetry: agent_telemetry }), bannerContainer);
    }
    renderSection(
      "agent-kpis-row",
      hasAgentActivity,
      /* @__PURE__ */ u4(AgentKpis, { telemetry: agent_telemetry, totalCostUsd }),
      "grid"
    );
    renderSection(
      "agent-timeline",
      agent_telemetry.timeline.length > 0,
      /* @__PURE__ */ u4(AgentTimeline, { timeline: agent_telemetry.timeline })
    );
    renderSection(
      "agent-distribution",
      agent_telemetry.distribution.length > 0,
      /* @__PURE__ */ u4(AgentDistribution, { data: agent_telemetry.distribution })
    );
    renderSection(
      "agent-top-sessions",
      agent_telemetry.top_sessions.length > 0,
      /* @__PURE__ */ u4(AgentTopSessions, { data: agent_telemetry.top_sessions })
    );
    renderSection(
      "agent-spawn-batches",
      agent_telemetry.spawn_batches.length > 0,
      /* @__PURE__ */ u4(
        AgentSpawnBatches,
        {
          data: agent_telemetry.spawn_batches,
          summary: agent_telemetry.spawn_batches_summary
        }
      )
    );
    renderSection(
      "agent-tool-spectrum",
      agent_telemetry.tool_spectrum.length > 0,
      /* @__PURE__ */ u4(AgentToolSpectrum, { data: agent_telemetry.tool_spectrum })
    );
  }
  function renderSubagentSummary(summary) {
    renderSection(
      "subagent-summary",
      summary.subagent_turns > 0,
      /* @__PURE__ */ u4(SubagentSummary, { summary })
    );
  }
  function renderEntrypointBreakdown(data) {
    renderSection("entrypoint-breakdown", data.length > 0, /* @__PURE__ */ u4(EntrypointTable, { data }));
  }
  function renderServiceTiers(data) {
    renderSection("service-tiers", data.length > 0, /* @__PURE__ */ u4(ServiceTiersTable, { data }));
  }
  function renderToolSummary(data) {
    renderSection("tool-summary", data.length > 0, /* @__PURE__ */ u4(ToolUsageTable, { data }));
  }
  function renderMcpSummary(data) {
    renderSection("mcp-summary", data.length > 0, /* @__PURE__ */ u4(McpSummaryTable, { data }));
  }
  function renderBranchSummary(data) {
    renderSection("branch-summary", data.length > 0, /* @__PURE__ */ u4(BranchTable, { data }));
  }
  function renderVersionSummary(data) {
    const container = $2("version-summary");
    if (!container) return;
    if (!data.length) {
      setSectionVisibility("version-summary", false);
      R(null, container);
      return;
    }
    setSectionVisibility("version-summary", true);
    const handleMetricChange = (next) => {
      versionDonutMetric.value = next;
      syncDashboardUrl();
      renderVersionSummary(data);
    };
    const collapsed = isSectionCollapsed("version-summary");
    const toggleCollapsed = () => {
      setSectionCollapsed("version-summary", !collapsed);
      syncDashboardUrl();
      renderVersionSummary(data);
    };
    R(
      /* @__PURE__ */ u4("div", { class: "table-card", children: [
        /* @__PURE__ */ u4("div", { class: "section-header", style: { padding: "20px 20px 0" }, children: [
          /* @__PURE__ */ u4("h2", { class: "section-title", style: { margin: 0 }, children: "CLI Versions" }),
          /* @__PURE__ */ u4("div", { class: "section-actions", children: /* @__PURE__ */ u4(
            "button",
            {
              class: "section-toggle",
              type: "button",
              "aria-expanded": !collapsed,
              "aria-controls": "version-summary-content",
              onClick: toggleCollapsed,
              children: collapsed ? "Show" : "Hide"
            }
          ) })
        ] }),
        !collapsed && /* @__PURE__ */ u4(
          "div",
          {
            id: "version-summary-content",
            style: {
              display: "flex",
              gap: "24px",
              alignItems: "flex-start",
              flexWrap: "wrap",
              padding: "20px"
            },
            children: [
              /* @__PURE__ */ u4("div", { style: { flex: "1 1 260px", minWidth: "220px" }, children: /* @__PURE__ */ u4(
                VersionDonut,
                {
                  rows: data,
                  metric: versionDonutMetric.value,
                  onMetricChange: handleMetricChange
                }
              ) }),
              /* @__PURE__ */ u4("div", { style: { flex: "2 1 320px", minWidth: "280px" }, children: /* @__PURE__ */ u4(VersionTable, { data, title: null }) })
            ]
          }
        )
      ] }),
      container
    );
  }
  function renderHourlyChart(data) {
    renderSection("hourly-chart", data.length > 0, /* @__PURE__ */ u4(HourlyChart, { data }));
  }
  function renderSubscriptionQuota(section) {
    const container = $2("subscription-quota");
    if (!container) return;
    const hasContent = !!section && (section.providers.length > 0 || section.history.length > 0 || section.changelog.length > 0);
    if (!hasContent) {
      setSectionVisibility("subscription-quota", false, "block");
      R(null, container);
      return;
    }
    setSectionVisibility("subscription-quota", true, "block");
    R(
      /* @__PURE__ */ u4("div", { class: "subscription-quota-section", children: [
        /* @__PURE__ */ u4("div", { class: "subscription-quota-grid", children: section.providers.map((snap) => /* @__PURE__ */ u4(SubscriptionQuotaCard, { snapshot: snap }, snap.provider)) }),
        /* @__PURE__ */ u4(
          SubscriptionHistoryChart,
          {
            history: section.history,
            changelog: section.changelog
          }
        )
      ] }),
      container
    );
  }
  function renderCodexPlan(section) {
    const hasToday = !!section?.today;
    const hasHistory = !!(section?.history && section.history.length > 0);
    const hasAny = hasToday || hasHistory;
    if (hasToday) {
      renderSection(
        "codex-plan-kpi-mount",
        true,
        /* @__PURE__ */ u4(CodexPlanKpi, { today: section.today }),
        "grid"
      );
    } else {
      setSectionVisibility("codex-plan-kpi-mount", false, "grid");
      R(null, $2("codex-plan-kpi-mount"));
    }
    if (hasHistory) {
      renderSection(
        "codex-plan-history-mount",
        true,
        /* @__PURE__ */ u4(CodexPlanHistory, { history: section.history })
      );
    } else {
      setSectionVisibility("codex-plan-history-mount", false);
      R(null, $2("codex-plan-history-mount"));
    }
    void hasAny;
  }
  function renderUsageWindows(data, previousSessionPercent, setPreviousSessionPercent, setStatusMessage, clearStatusMessage) {
    const container = $2("usage-windows");
    if (!container) return;
    if (!data.available) {
      planBadge.value = "";
      if (data.error) {
        setSectionVisibility("usage-windows", true, "grid");
        R(/* @__PURE__ */ u4(RateWindowUnavailable, { error: data.error }), container);
      } else {
        setSectionVisibility("usage-windows", false, "grid");
        R(null, container);
      }
      return;
    }
    const hasAnyWindow = Boolean(data.session) || Boolean(data.weekly) || Boolean(data.weekly_opus) || Boolean(data.weekly_sonnet) || Boolean(data.budget);
    const hasAdmin = data.source === "admin" && Boolean(data.admin_fallback);
    if (!hasAnyWindow && !hasAdmin) {
      setSectionVisibility("usage-windows", false, "grid");
      R(null, container);
      setPreviousSessionPercent(null);
      clearStatusMessage();
      return;
    }
    setSectionVisibility("usage-windows", true, "grid");
    if (data.source === "admin" && data.admin_fallback) {
      R(
        /* @__PURE__ */ u4(S, { children: [
          /* @__PURE__ */ u4(ClaudeAdminFallbackGrid, { summary: data.admin_fallback }),
          /* @__PURE__ */ u4("div", { style: { gridColumn: "1 / -1" }, children: /* @__PURE__ */ u4(InlineStatus, { placement: "rate-windows" }) })
        ] }),
        container
      );
      setPreviousSessionPercent(null);
      clearStatusMessage();
      return;
    }
    R(
      /* @__PURE__ */ u4(S, { children: [
        data.session && /* @__PURE__ */ u4(RateWindowCard, { label: "Session (5h)", window: data.session }),
        data.weekly && /* @__PURE__ */ u4(RateWindowCard, { label: "Weekly", window: data.weekly }),
        data.weekly_opus && /* @__PURE__ */ u4(RateWindowCard, { label: "Weekly Opus", window: data.weekly_opus }),
        data.weekly_sonnet && /* @__PURE__ */ u4(RateWindowCard, { label: "Weekly Sonnet", window: data.weekly_sonnet }),
        data.budget && /* @__PURE__ */ u4(
          BudgetCard,
          {
            used: data.budget.used,
            limit: data.budget.limit,
            currency: data.budget.currency,
            utilization: data.budget.utilization
          }
        ),
        /* @__PURE__ */ u4("div", { style: { gridColumn: "1 / -1" }, children: /* @__PURE__ */ u4(InlineStatus, { placement: "rate-windows" }) })
      ] }),
      container
    );
    if (data.session) {
      const currentPercent = 100 - data.session.used_percent;
      if (previousSessionPercent !== null) {
        if (previousSessionPercent > 0.01 && currentPercent <= 0.01) {
          setStatusMessage(`Session depleted - resets in ${data.session.resets_in_minutes ?? 0}m`, true);
        } else if (previousSessionPercent <= 0.01 && currentPercent > 0.01) {
          clearStatusMessage();
        }
      }
      setPreviousSessionPercent(currentPercent);
    }
    planBadge.value = data.identity?.plan ? data.identity.plan.charAt(0).toUpperCase() + data.identity.plan.slice(1) : "";
  }
  function renderClaudeUsage(data) {
    renderSection("claude-usage", !!(data.last_run || data.latest_snapshot), /* @__PURE__ */ u4(ClaudeUsagePanel, { data }));
  }
  function renderAgentStatus(snapshot, communitySignal) {
    const container = $2("agent-status");
    if (!container) return;
    setSectionVisibility("agent-status", true, "grid");
    R(/* @__PURE__ */ u4(AgentStatusCard, { snapshot, communitySignal }), container);
  }
  function renderActivityHeatmap(data) {
    const container = $2("activity-heatmap");
    if (!container) return;
    if (!data) {
      setSectionVisibility("activity-heatmap", false);
      R(null, container);
      return;
    }
    setSectionVisibility("activity-heatmap", true);
    const handleMetricChange = (next) => {
      heatmapMetric.value = next;
      syncDashboardUrl();
      renderActivityHeatmap(data);
    };
    R(
      /* @__PURE__ */ u4(
        ActivityHeatmap,
        {
          data,
          metric: heatmapMetric.value,
          onMetricChange: handleMetricChange
        }
      ),
      container
    );
  }
  function renderCostReconciliation() {
    const container = $2("cost-reconciliation");
    if (!container) return;
    const data = costReconciliationData.value;
    if (!data || !data.enabled) {
      setSectionVisibility("cost-reconciliation", false);
      R(null, container);
      return;
    }
    setSectionVisibility("cost-reconciliation", true);
    R(/* @__PURE__ */ u4(CostReconciliationPanel, { data }), container);
  }
  function renderTodayView(data, onDateChange) {
    const pickerContainer = $2("today-date-picker-mount");
    if (pickerContainer) {
      setSectionVisibility("today-date-picker-mount", true);
      R(/* @__PURE__ */ u4(DatePicker, { onDateChange }), pickerContainer);
    }
    renderSection(
      "today-kpis-mount",
      true,
      /* @__PURE__ */ u4(TodayKpis, { totals: data.totals, day: data.day }),
      "grid"
    );
    renderSection(
      "today-hour-timeline-mount",
      true,
      /* @__PURE__ */ u4("div", { style: { padding: "20px", height: "100%", boxSizing: "border-box", display: "flex", flexDirection: "column" }, children: [
        /* @__PURE__ */ u4("div", { class: "section-title", style: { marginBottom: "12px" }, children: [
          "Hour timeline \u2014 ",
          data.day
        ] }),
        /* @__PURE__ */ u4(HourTimeline, { hours: data.hours })
      ] })
    );
    renderSection(
      "today-hour-heatstrip-mount",
      true,
      /* @__PURE__ */ u4("div", { style: { padding: "20px" }, children: [
        /* @__PURE__ */ u4("div", { class: "section-title", style: { marginBottom: "12px" }, children: "Hour heatstrip" }),
        /* @__PURE__ */ u4(HourHeatstrip, { hours: data.hours })
      ] })
    );
    renderSection(
      "today-days-hours-30-mount",
      data.days_hours_30.length > 0,
      /* @__PURE__ */ u4("div", { style: { padding: "20px", overflowX: "auto" }, children: /* @__PURE__ */ u4(
        DaysHoursHeatmap,
        {
          cells: data.days_hours_30,
          daysCount: 30,
          title: "30 days \xD7 24 hours",
          onDayClick: onDateChange
        }
      ) })
    );
    renderSection(
      "today-days-hours-7-mount",
      data.days_hours_7.length > 0,
      /* @__PURE__ */ u4("div", { style: { padding: "20px", overflowX: "auto" }, children: /* @__PURE__ */ u4(
        DaysHoursHeatmap,
        {
          cells: data.days_hours_7,
          daysCount: 7,
          title: "7 days \xD7 24 hours",
          onDayClick: onDateChange
        }
      ) })
    );
    renderSection(
      "today-weekday-hour-mount",
      data.weekday_hour_90.length > 0,
      /* @__PURE__ */ u4("div", { style: { padding: "20px", overflowX: "auto" }, children: [
        /* @__PURE__ */ u4("div", { class: "section-title", style: { marginBottom: "12px" }, children: "Weekday \xD7 hour pattern (90-day window)" }),
        /* @__PURE__ */ u4(WeekdayHourHeatmap, { cells: data.weekday_hour_90 })
      ] })
    );
    refreshSectionVisibility();
  }
  function renderDashboardView(data, focusSingleModel, focusProjectQuery, exportSessionsCSV2, exportProjectsCSV2, onReload) {
    const emptyMount = $2("empty-state-mount");
    if (emptyMount) {
      if (data.sessions_all.length === 0) {
        R(
          /* @__PURE__ */ u4(
            "div",
            {
              role: "status",
              style: {
                fontFamily: "var(--font-mono)",
                fontSize: "11px",
                letterSpacing: "0.08em",
                textTransform: "uppercase",
                color: "var(--text-secondary)",
                display: "flex",
                alignItems: "center",
                gap: "8px",
                padding: "8px 16px",
                border: "1px solid var(--text-secondary)",
                borderRadius: "4px",
                background: "var(--surface)",
                marginTop: "12px"
              },
              children: [
                "[INFO: No sessions ingested yet. Run",
                " ",
                /* @__PURE__ */ u4("code", { style: { fontFamily: "var(--font-mono)" }, children: "cargo run -- scan" }),
                " ",
                "(or open Claude Code / Codex once and revisit) to populate the dashboard.]"
              ]
            }
          ),
          emptyMount
        );
        emptyMount.style.display = "";
      } else {
        R(null, emptyMount);
        emptyMount.style.display = "none";
      }
    }
    const cutoff = getRangeCutoff(selectedRange.value);
    const filteredDaily = data.daily_by_model.filter(
      (row) => selectedModels.value.has(row.model) && (!cutoff || row.day >= cutoff) && matchesProvider(row)
    );
    const filteredSessions = data.sessions_all.filter(
      (session) => selectedModels.value.has(session.model) && (!cutoff || session.last_date >= cutoff) && matchesProjectSearch(session.project, session.display_name) && matchesProvider(session)
    );
    const {
      daily,
      byModel,
      byProject,
      totals,
      confidenceBreakdown,
      billingModeBreakdown,
      pricingVersions
    } = buildAggregations(filteredDaily, filteredSessions);
    const providerLabel = selectedProvider.value === "both" ? "" : ` (${selectedProvider.value})`;
    const bucketIsWeek = selectedBucket.value === "week";
    const activeDays = daily.filter((day) => day.cost > 0).length;
    const activeDayCostNanos = Math.round(
      daily.reduce((sum2, day) => sum2 + day.cost, 0) * 1e9
    );
    const chartTitleEl = $2("daily-chart-title");
    if (chartTitleEl) {
      chartTitleEl.textContent = (bucketIsWeek ? "Weekly Token Usage - " : "Daily Token Usage - ") + RANGE_LABELS[selectedRange.value] + providerLabel;
    }
    R(
      /* @__PURE__ */ u4(
        StatsCards,
        {
          totals,
          daily,
          activeDays,
          activeDayTotalCostNanos: activeDayCostNanos,
          cacheEfficiency: data.cache_efficiency,
          billingBlocks: billingBlocksData.value,
          contextWindow: contextWindowData.value
        }
      ),
      $2("stats-row")
    );
    setSectionVisibility("stats-row", true, "grid");
    renderEstimationMeta(confidenceBreakdown, billingModeBreakdown, pricingVersions);
    renderSubscriptionQuota(data.subscription_quota);
    renderCodexPlan(data.codex_plan);
    renderOfficialSync(data.official_sync);
    renderOpenAiReconciliation(data.openai_reconciliation);
    renderSubagentReconciliation(data.subagent_reconciliation ?? null);
    if (bucketIsWeek) {
      const weekly = buildWeeklyAgg(data.weekly_by_model, selectedModels.value, selectedRange.value);
      R(/* @__PURE__ */ u4(WeeklyChart, { weekly }), $2("chart-daily"));
      setSectionVisibility("daily-chart-card", weekly.length > 0);
    } else {
      R(/* @__PURE__ */ u4(DailyChart, { daily }), $2("chart-daily"));
      setSectionVisibility("daily-chart-card", daily.length > 0);
    }
    R(/* @__PURE__ */ u4(ModelChart, { byModel, onSelectModel: focusSingleModel }), $2("chart-model"));
    R(
      /* @__PURE__ */ u4(
        ProjectChart,
        {
          byProject,
          onSelectProject: (project) => focusProjectQuery(project.display_name || project.project)
        }
      ),
      $2("chart-project")
    );
    setSectionVisibility("model-chart-card", byModel.length > 0);
    setSectionVisibility("project-chart-card", byProject.length > 0);
    lastFilteredSessions.value = filteredSessions;
    lastByProject.value = byProject;
    R(/* @__PURE__ */ u4(ModelCostTable, { byModel, onSelectModel: focusSingleModel }), $2("model-cost-mount"));
    R(
      /* @__PURE__ */ u4(
        SessionsTable,
        {
          onExportCSV: exportSessionsCSV2,
          onSelectProject: (session) => focusProjectQuery(session.display_name || session.project),
          onSelectModel: focusSingleModel
        }
      ),
      $2("sessions-mount")
    );
    R(
      /* @__PURE__ */ u4(
        ProjectCostTable,
        {
          byProject: lastByProject.value.slice(0, 30),
          onExportCSV: exportProjectsCSV2,
          onSelectProject: (project) => focusProjectQuery(project.display_name || project.project),
          ...onReload ? { onReload } : {}
        }
      ),
      $2("project-cost-mount")
    );
    setSectionVisibility("model-cost-mount", byModel.length > 0);
    setSectionVisibility("sessions-mount", filteredSessions.length > 0);
    setSectionVisibility("project-cost-mount", lastByProject.value.length > 0);
    renderSubagentSummary(data.subagent_summary);
    renderAgentTelemetry(data);
    renderEntrypointBreakdown((data.entrypoint_breakdown ?? []).filter(matchesProvider));
    renderServiceTiers((data.service_tiers ?? []).filter(matchesProvider));
    renderToolSummary((data.tool_summary ?? []).filter(matchesProvider));
    renderMcpSummary((data.mcp_summary ?? []).filter(matchesProvider));
    renderBranchSummary((data.git_branch_summary ?? []).filter(matchesProvider));
    renderVersionSummary((data.version_summary ?? []).filter(matchesProvider));
    renderHourlyChart((data.hourly_distribution ?? []).filter(matchesProvider));
    renderCostReconciliation();
    refreshSectionVisibility();
  }

  // src/ui/lib/today.ts
  async function loadToday(date, tzOffsetMin) {
    todayLoading.value = true;
    try {
      let url = `/api/today?tz_offset_min=${tzOffsetMin}`;
      if (date) url += `&date=${encodeURIComponent(date)}`;
      const resp = await fetch(url);
      if (!resp.ok) return null;
      const data = await resp.json();
      todayData.value = data;
      return data;
    } catch {
      return null;
    } finally {
      todayLoading.value = false;
    }
  }

  // src/ui/dashboard/runtime.ts
  function createRuntimeState() {
    return {
      previousSessionPercent: null,
      lastCommunitySignal: null,
      lastAgentStatusSnapshot: null,
      inFlight: {
        loadData: false,
        loadUsageWindows: false,
        loadClaudeUsage: false,
        loadHeatmap: false,
        loadAgentStatus: false,
        loadCommunitySignal: false,
        loadBillingBlocks: false,
        loadContextWindow: false,
        loadCostReconciliation: false
      }
    };
  }
  async function runExclusive(state, guard, task, force = false) {
    if (state.inFlight[guard] && !force) return;
    state.inFlight[guard] = true;
    try {
      await task();
    } finally {
      state.inFlight[guard] = false;
    }
  }
  async function fetchJson(url) {
    const response = await fetch(url);
    if (!response.ok) return null;
    return await response.json();
  }
  function toggleModelSelection(allModels, currentSelection, model) {
    const isSoleSelection = currentSelection.size === 1 && currentSelection.has(model);
    return isSoleSelection ? new Set(allModels) : /* @__PURE__ */ new Set([model]);
  }
  function toggleProjectSearch(currentQuery, project) {
    const normalized = project.toLowerCase().trim();
    return currentQuery === normalized ? "" : normalized;
  }
  function exportProjectRowsCSV(filename, rowsData) {
    const header = [
      "Project",
      "Sessions",
      "Turns",
      "Input",
      "Output",
      "Cached Input",
      "Cache Creation",
      "Reasoning Output",
      "Est. Cost"
    ];
    const rows2 = rowsData.map((project) => [
      project.project,
      project.sessions,
      project.turns,
      project.input,
      project.output,
      project.cache_read,
      project.cache_creation,
      project.reasoning_output,
      project.cost.toFixed(4)
    ]);
    downloadCSV(filename, header, rows2);
  }
  function exportSessionsCSV() {
    const header = [
      "Session",
      "Provider",
      "Project",
      "Last Active",
      "Duration (min)",
      "Model",
      "Turns",
      "Input",
      "Output",
      "Cached Input",
      "Cache Creation",
      "Reasoning Output",
      "Est. Cost"
    ];
    const rows2 = rawData.value ? rawData.value.sessions_all.filter((session) => selectedModels.value.has(session.model)).map((session) => [
      session.session_id,
      session.provider,
      session.project,
      session.last,
      session.duration_min,
      session.model,
      session.turns,
      session.input,
      session.output,
      session.cache_read,
      session.cache_creation,
      session.reasoning_output,
      session.cost.toFixed(4)
    ]) : [];
    downloadCSV("sessions", header, rows2);
  }
  function exportProjectsCSV() {
    exportProjectRowsCSV("projects", lastByProject.value);
  }
  function createUsageWindowsLoader(state) {
    return () => runExclusive(state, "loadUsageWindows", async () => {
      try {
        const data = await fetchJson("/api/usage-windows");
        if (!data) return;
        renderUsageWindows(
          data,
          state.previousSessionPercent,
          (value) => {
            state.previousSessionPercent = value;
          },
          (message, isError = false) => {
            setStatus(
              "rate-windows",
              isError ? "error" : "success",
              message,
              isError ? void 0 : 4e3
            );
          },
          () => clearStatus("rate-windows")
        );
      } catch {
      }
    });
  }
  function createClaudeUsageLoader(state) {
    return () => runExclusive(state, "loadClaudeUsage", async () => {
      try {
        const data = await fetchJson("/api/claude-usage");
        if (data) renderClaudeUsage(data);
      } catch {
      }
    });
  }
  function createAgentStatusLoader(state) {
    return () => runExclusive(state, "loadAgentStatus", async () => {
      try {
        const data = await fetchJson("/api/agent-status");
        if (!data) return;
        state.lastAgentStatusSnapshot = data;
        renderAgentStatus(data, state.lastCommunitySignal);
      } catch {
      }
    });
  }
  function createCommunitySignalLoader(state) {
    return () => runExclusive(state, "loadCommunitySignal", async () => {
      try {
        const data = await fetchJson("/api/community-signal");
        if (!data) return;
        state.lastCommunitySignal = data.enabled ? data : null;
        if (state.lastAgentStatusSnapshot) {
          renderAgentStatus(state.lastAgentStatusSnapshot, state.lastCommunitySignal);
        }
      } catch {
      }
    });
  }
  function createBillingBlocksLoader(state, applyFilter) {
    return () => runExclusive(state, "loadBillingBlocks", async () => {
      try {
        const data = await fetchJson("/api/billing-blocks");
        billingBlocksData.value = data;
        if (data && rawData.value) applyFilter();
      } catch {
        billingBlocksData.value = null;
      }
    });
  }
  function createContextWindowLoader(state, applyFilter) {
    return () => runExclusive(state, "loadContextWindow", async () => {
      try {
        const data = await fetchJson("/api/context-window");
        contextWindowData.value = data;
        if (data && rawData.value) applyFilter();
      } catch {
        contextWindowData.value = null;
      }
    });
  }
  function createCostReconciliationLoader(state) {
    return () => runExclusive(state, "loadCostReconciliation", async () => {
      try {
        const data = await fetchJson("/api/cost-reconciliation?period=month");
        costReconciliationData.value = data;
        if (data) renderCostReconciliation();
      } catch {
        costReconciliationData.value = null;
      }
    });
  }
  function currentTimezoneOffsetMinutes() {
    return typeof window !== "undefined" && typeof window.Date !== "undefined" ? (/* @__PURE__ */ new Date()).getTimezoneOffset() * -1 : 0;
  }
  function createHeatmapLoader(state) {
    return (period = "month") => runExclusive(state, "loadHeatmap", async () => {
      try {
        const tzOffset = currentTimezoneOffsetMinutes();
        const data = await fetchJson(
          `/api/heatmap?period=${encodeURIComponent(period)}&tz_offset_min=${tzOffset}`
        );
        if (data) renderActivityHeatmap(data);
      } catch {
      }
    });
  }
  function createDataLoader(state, applyFilter) {
    return (force = false) => runExclusive(
      state,
      "loadData",
      async () => {
        const isSubsequentFetch = rawData.value !== null;
        if (isSubsequentFetch) {
          loadState.value = "refreshing";
          setStatus("header-refresh", "loading", "REFRESHING");
        }
        let data;
        try {
          const response = await fetch("/api/data");
          if (!response.ok) {
            setStatus("global", "error", `Failed to load data: HTTP ${response.status}`);
            clearStatus("header-refresh");
            return;
          }
          data = await response.json();
        } catch (err) {
          const isNetwork = err instanceof TypeError;
          setStatus(
            "global",
            "error",
            isNetwork ? "Network error loading data" : "Invalid response from /api/data"
          );
          console.error("loadData fetch/parse failed:", err);
          clearStatus("header-refresh");
          return;
        } finally {
          loadState.value = "idle";
        }
        if (data.error) {
          setStatus("global", "error", data.error);
          clearStatus("header-refresh");
          return;
        }
        clearStatus("global");
        clearStatus("header-refresh");
        rawData.value = data;
        if (!isSubsequentFetch) {
          restoreDashboardStateFromUrl(data.all_models);
        }
        try {
          applyFilter();
        } catch (err) {
          setStatus("global", "error", "Failed to render dashboard");
          console.error("applyFilter failed:", err);
        }
      },
      force
    );
  }
  function startDashboardPolling(loaders) {
    window.addEventListener("popstate", () => {
      if (!rawData.value) return;
      restoreDashboardStateFromUrl(rawData.value.all_models);
      loaders.applyFilter();
    });
    void loaders.loadData();
    setInterval(loaders.loadData, 3e4);
    void loaders.loadUsageWindows();
    void loaders.loadClaudeUsage();
    void loaders.loadAgentStatus();
    void loaders.loadCommunitySignal();
    setInterval(() => {
      void loaders.loadUsageWindows();
      void loaders.loadClaudeUsage();
      void loaders.loadAgentStatus();
      void loaders.loadCommunitySignal();
    }, 6e4);
    void loaders.loadHeatmap("all");
    setInterval(() => void loaders.loadHeatmap("all"), 3e4);
    void loaders.loadBillingBlocks();
    setInterval(() => void loaders.loadBillingBlocks(), 3e4);
    void loaders.loadContextWindow();
    setInterval(() => void loaders.loadContextWindow(), 3e4);
    void loaders.loadCostReconciliation();
    setInterval(() => void loaders.loadCostReconciliation(), 3e4);
  }
  function createDashboardRuntime() {
    const state = createRuntimeState();
    let loadDataRef;
    const applyFilter = () => {
      if (!rawData.value) return;
      renderDashboardView(
        rawData.value,
        focusSingleModel,
        focusProjectQuery,
        exportSessionsCSV,
        exportProjectsCSV,
        () => loadDataRef?.()
      );
    };
    const focusSingleModel = (model) => {
      if (!rawData.value) return;
      selectedModels.value = toggleModelSelection(rawData.value.all_models, selectedModels.value, model);
      syncDashboardUrl();
      applyFilter();
    };
    const focusProjectQuery = (project) => {
      projectSearchQuery.value = toggleProjectSearch(projectSearchQuery.value, project);
      syncDashboardUrl();
      applyFilter();
    };
    const loadUsageWindows = createUsageWindowsLoader(state);
    const loadClaudeUsage = createClaudeUsageLoader(state);
    const loadAgentStatus = createAgentStatusLoader(state);
    const loadCommunitySignal = createCommunitySignalLoader(state);
    const loadBillingBlocks = createBillingBlocksLoader(state, applyFilter);
    const loadContextWindow = createContextWindowLoader(state, applyFilter);
    const loadCostReconciliation = createCostReconciliationLoader(state);
    const loadHeatmap = createHeatmapLoader(state);
    const loadData2 = createDataLoader(state, applyFilter);
    loadDataRef = loadData2;
    function handleDateChange(date) {
      selectedDate.value = date;
      syncDashboardUrl();
      void loadToday(date, currentTimezoneOffsetMinutes()).then((data) => {
        if (data) renderTodayView(data, handleDateChange);
      });
    }
    registerMountCallback("today-date-picker-mount", () => {
      queueMicrotask(() => {
        if (todayData.value && activeDashboardTab.value === "today") {
          renderTodayView(todayData.value, handleDateChange);
        }
      });
    });
    function maybeLoadToday() {
      if (activeDashboardTab.value !== "today") return;
      void loadToday(selectedDate.value, currentTimezoneOffsetMinutes()).then((data) => {
        if (data) renderTodayView(data, handleDateChange);
      });
    }
    let prevActiveTab = activeDashboardTab.value;
    activeDashboardTab.subscribe((tab) => {
      const prev = prevActiveTab;
      prevActiveTab = tab;
      if (prev === "today") clearTodayWidgets();
      refreshSectionVisibility();
      if (tab === "today") {
        if (todayData.value) renderTodayView(todayData.value, handleDateChange);
        else maybeLoadToday();
      }
    });
    return {
      applyFilter,
      handleDashboardTabChange(tab) {
        if (activeDashboardTab.value === tab) return;
        const prevTab = activeDashboardTab.value;
        activeDashboardTab.value = tab;
        syncDashboardUrl();
        if (prevTab === "today") clearTodayWidgets();
        refreshSectionVisibility();
        if (tab === "today") {
          if (todayData.value) {
            renderTodayView(todayData.value, handleDateChange);
          } else {
            maybeLoadToday();
          }
        }
      },
      loadData: loadData2,
      start() {
        startDashboardPolling({
          applyFilter,
          loadAgentStatus,
          loadBillingBlocks,
          loadClaudeUsage,
          loadCommunitySignal,
          loadContextWindow,
          loadCostReconciliation,
          loadData: loadData2,
          loadHeatmap,
          loadUsageWindows
        });
        maybeLoadToday();
      }
    };
  }

  // src/ui/monitor/store.ts
  var LIVE_MONITOR_PREFERENCE_KEY = "heimdall.live_monitor.preferences.v1";
  var LIVE_MONITOR_PANEL_OPTIONS = [
    { id: "active_block", label: "Active Block" },
    { id: "predictive_insights", label: "Predictive Signals" },
    { id: "depletion_forecast", label: "Depletion Forecast" },
    { id: "quota_suggestions", label: "Suggested Quotas" },
    { id: "context_window", label: "Context Window" },
    { id: "recent_session", label: "Recent Session" },
    { id: "warnings", label: "Warnings" }
  ];
  var LIVE_MONITOR_FOCUS_OPTIONS = ["all", "claude", "codex"];
  var LIVE_MONITOR_DENSITY_OPTIONS = ["expanded", "compact"];
  var LIVE_MONITOR_PANEL_IDS = LIVE_MONITOR_PANEL_OPTIONS.map((option) => option.id);
  var liveMonitorData = y3(null);
  var liveMonitorFocus = y3("all");
  var liveMonitorDensity = y3("expanded");
  var liveMonitorHiddenPanels = y3([]);
  var liveMonitorRefreshing = y3(false);
  var liveMonitorError = y3(null);
  var liveMonitorPreferencesHydrated = y3(false);
  function setLiveMonitorData(data) {
    liveMonitorData.value = data;
    if (liveMonitorPreferencesHydrated.value) {
      const previousFocus = liveMonitorFocus.value;
      const resolvedFocus = normalizeFocusForProviders(previousFocus, data);
      liveMonitorFocus.value = resolvedFocus;
      if (resolvedFocus !== previousFocus) {
        persistLiveMonitorPreferences();
      }
    } else {
      liveMonitorFocus.value = normalizeFocusForProviders(data.default_focus, data);
    }
    liveMonitorError.value = null;
  }
  function hydrateLiveMonitorPreferences() {
    if (liveMonitorPreferencesHydrated.value) {
      return;
    }
    const saved = readLiveMonitorPreferences();
    if (saved) {
      liveMonitorFocus.value = saved.focus;
      liveMonitorDensity.value = saved.density;
      liveMonitorHiddenPanels.value = saved.hiddenPanels;
    } else {
      liveMonitorFocus.value = "all";
      liveMonitorDensity.value = "expanded";
      liveMonitorHiddenPanels.value = [];
    }
    liveMonitorPreferencesHydrated.value = saved != null;
  }
  function setLiveMonitorFocus(focus) {
    liveMonitorFocus.value = focus;
    liveMonitorPreferencesHydrated.value = true;
    persistLiveMonitorPreferences();
  }
  function setLiveMonitorDensity(density) {
    liveMonitorDensity.value = density;
    liveMonitorPreferencesHydrated.value = true;
    persistLiveMonitorPreferences();
  }
  function toggleLiveMonitorPanel(panelId) {
    const hiddenPanels = new Set(liveMonitorHiddenPanels.value);
    if (hiddenPanels.has(panelId)) {
      hiddenPanels.delete(panelId);
    } else {
      hiddenPanels.add(panelId);
    }
    liveMonitorHiddenPanels.value = [...hiddenPanels].sort();
    liveMonitorPreferencesHydrated.value = true;
    persistLiveMonitorPreferences();
  }
  function isLiveMonitorPanelHidden(panelId) {
    return liveMonitorHiddenPanels.value.includes(panelId);
  }
  function normalizeFocusForProviders(focus, data) {
    if (focus === "all") {
      return "all";
    }
    return data.providers.some((provider) => provider.provider === focus) ? focus : "all";
  }
  function normalizeLiveMonitorPreferences(value) {
    if (!value || typeof value !== "object") {
      return null;
    }
    const candidate = value;
    const focus = LIVE_MONITOR_FOCUS_OPTIONS.includes(candidate.focus) ? candidate.focus : "all";
    const density = LIVE_MONITOR_DENSITY_OPTIONS.includes(candidate.density) ? candidate.density : "expanded";
    const hiddenPanels = Array.isArray(candidate.hiddenPanels) ? Array.from(new Set(candidate.hiddenPanels.filter(
      (panel) => LIVE_MONITOR_PANEL_IDS.includes(panel)
    ))).sort() : [];
    return { focus, density, hiddenPanels };
  }
  function readLiveMonitorPreferences() {
    try {
      const raw = localStorage.getItem(LIVE_MONITOR_PREFERENCE_KEY);
      if (!raw) {
        return null;
      }
      return normalizeLiveMonitorPreferences(JSON.parse(raw));
    } catch {
      return null;
    }
  }
  function persistLiveMonitorPreferences() {
    try {
      localStorage.setItem(
        LIVE_MONITOR_PREFERENCE_KEY,
        JSON.stringify({
          focus: liveMonitorFocus.value,
          density: liveMonitorDensity.value,
          hiddenPanels: liveMonitorHiddenPanels.value
        })
      );
    } catch {
    }
  }

  // src/ui/monitor/MonitorHeader.tsx
  function MonitorHeader({ onThemeToggle, onRefresh }) {
    const mode = themeMode.value;
    const icon = mode === "dark" ? /* @__PURE__ */ u4("svg", { "aria-hidden": "true", focusable: "false", width: "14", height: "14", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", children: /* @__PURE__ */ u4("path", { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }) }) : /* @__PURE__ */ u4("svg", { "aria-hidden": "true", focusable: "false", width: "14", height: "14", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", "stroke-width": "2", children: [
      /* @__PURE__ */ u4("circle", { cx: "12", cy: "12", r: "5" }),
      /* @__PURE__ */ u4("line", { x1: "12", y1: "1", x2: "12", y2: "3" }),
      /* @__PURE__ */ u4("line", { x1: "12", y1: "21", x2: "12", y2: "23" }),
      /* @__PURE__ */ u4("line", { x1: "4.22", y1: "4.22", x2: "5.64", y2: "5.64" }),
      /* @__PURE__ */ u4("line", { x1: "18.36", y1: "18.36", x2: "19.78", y2: "19.78" }),
      /* @__PURE__ */ u4("line", { x1: "1", y1: "12", x2: "3", y2: "12" }),
      /* @__PURE__ */ u4("line", { x1: "21", y1: "12", x2: "23", y2: "12" }),
      /* @__PURE__ */ u4("line", { x1: "4.22", y1: "19.78", x2: "5.64", y2: "18.36" }),
      /* @__PURE__ */ u4("line", { x1: "18.36", y1: "5.64", x2: "19.78", y2: "4.22" })
    ] });
    const generatedAt = liveMonitorData.value?.generated_at ?? null;
    const issue = liveMonitorData.value?.global_issue ?? null;
    return /* @__PURE__ */ u4("header", { children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", alignItems: "center", gap: "12px", flexWrap: "wrap" }, children: [
        /* @__PURE__ */ u4("h1", { style: { marginBottom: 0 }, children: [
          /* @__PURE__ */ u4("span", { style: { color: "var(--text-secondary)", fontWeight: 400 }, children: "Live" }),
          " ",
          /* @__PURE__ */ u4("span", { style: { color: "var(--text-display)", fontWeight: 500 }, children: "Monitor" })
        ] }),
        issue && /* @__PURE__ */ u4(
          "span",
          {
            style: {
              fontFamily: "var(--font-mono)",
              fontSize: "10px",
              padding: "2px 8px",
              borderRadius: "999px",
              border: "1px solid var(--border-visible)",
              color: "var(--accent)",
              letterSpacing: "0.08em",
              textTransform: "uppercase"
            },
            children: issue
          }
        )
      ] }),
      /* @__PURE__ */ u4("div", { class: "meta", children: generatedAt ? `Updated ${new Date(generatedAt).toLocaleTimeString()}` : "Waiting for monitor data" }),
      /* @__PURE__ */ u4("div", { class: "header-actions", children: [
        /* @__PURE__ */ u4("div", { style: { display: "inline-flex", border: "1px solid var(--border-visible)", borderRadius: "999px", overflow: "hidden" }, children: ["all", "claude", "codex"].map((option) => /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            onClick: () => {
              setLiveMonitorFocus(option);
            },
            style: {
              padding: "8px 12px",
              border: "none",
              borderRight: option === "codex" ? "none" : "1px solid var(--border-visible)",
              background: liveMonitorFocus.value === option ? "var(--text-primary)" : "transparent",
              color: liveMonitorFocus.value === option ? "var(--bg)" : "var(--text-primary)",
              fontSize: "12px",
              letterSpacing: "0.08em",
              textTransform: "uppercase"
            },
            children: option === "all" ? "All" : option === "claude" ? "Claude" : "Codex"
          },
          option
        )) }),
        /* @__PURE__ */ u4("div", { style: { display: "inline-flex", border: "1px solid var(--border-visible)", borderRadius: "999px", overflow: "hidden" }, children: ["expanded", "compact"].map((option) => /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            onClick: () => {
              setLiveMonitorDensity(option);
            },
            style: {
              padding: "8px 12px",
              border: "none",
              borderRight: option === "compact" ? "none" : "1px solid var(--border-visible)",
              background: liveMonitorDensity.value === option ? "var(--text-primary)" : "transparent",
              color: liveMonitorDensity.value === option ? "var(--bg)" : "var(--text-primary)",
              fontSize: "12px",
              letterSpacing: "0.08em",
              textTransform: "uppercase"
            },
            children: option
          },
          option
        )) }),
        /* @__PURE__ */ u4(
          "details",
          {
            style: {
              border: "1px solid var(--border-visible)",
              borderRadius: "18px",
              padding: "8px 12px",
              minWidth: "220px"
            },
            children: [
              /* @__PURE__ */ u4(
                "summary",
                {
                  style: {
                    cursor: "pointer",
                    listStyle: "none",
                    fontSize: "12px",
                    letterSpacing: "0.08em",
                    textTransform: "uppercase"
                  },
                  children: "Panels"
                }
              ),
              /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "8px", marginTop: "10px" }, children: LIVE_MONITOR_PANEL_OPTIONS.map((panel) => {
                const visible = !isLiveMonitorPanelHidden(panel.id);
                return /* @__PURE__ */ u4("label", { style: { display: "flex", alignItems: "center", gap: "8px", fontSize: "12px" }, children: [
                  /* @__PURE__ */ u4(
                    "input",
                    {
                      type: "checkbox",
                      checked: visible,
                      onInput: () => {
                        toggleLiveMonitorPanel(panel.id);
                      }
                    }
                  ),
                  /* @__PURE__ */ u4("span", { children: panel.label })
                ] }, panel.id);
              }) })
            ]
          }
        ),
        /* @__PURE__ */ u4(
          "a",
          {
            href: "/",
            style: {
              border: "1px solid var(--border-visible)",
              borderRadius: "999px",
              padding: "8px 12px",
              color: "var(--text-primary)",
              textDecoration: "none",
              fontSize: "12px",
              letterSpacing: "0.08em",
              textTransform: "uppercase"
            },
            children: "Dashboard"
          }
        ),
        /* @__PURE__ */ u4(
          "button",
          {
            class: "theme-toggle",
            type: "button",
            onClick: onThemeToggle,
            "aria-label": "Toggle theme",
            children: icon
          }
        ),
        /* @__PURE__ */ u4("button", { type: "button", onClick: () => void onRefresh(), disabled: liveMonitorRefreshing.value, children: liveMonitorRefreshing.value ? "Refreshing\u2026" : "Refresh" })
      ] })
    ] });
  }

  // src/ui/monitor/view.tsx
  function densityTokens(density) {
    return density === "compact" ? { padding: "14px", fontSize: "11px", marginTop: "10px", sectionGap: "10px", headerGap: "10px", gridGap: "12px", listGap: "6px" } : { padding: "18px", fontSize: void 0, marginTop: "12px", sectionGap: "14px", headerGap: "12px", gridGap: "16px", listGap: "8px" };
  }
  function providersForFocus(data, focus) {
    return focus === "all" ? data.providers : data.providers.filter((provider) => provider.provider === focus);
  }
  function providerHasVisibleDetails(provider, hiddenPanels) {
    return !hiddenPanels.has("active_block") && !!provider.active_block || !!provider.claude_admin || !hiddenPanels.has("predictive_insights") && !!provider.predictive_insights || !hiddenPanels.has("depletion_forecast") && !!provider.depletion_forecast || !hiddenPanels.has("quota_suggestions") && !!provider.quota_suggestions || !hiddenPanels.has("context_window") && !!provider.context_window || !hiddenPanels.has("recent_session") && !!provider.recent_session || !hiddenPanels.has("warnings") && provider.warnings.length > 0;
  }
  function detailProviders(data, focus, hiddenPanels) {
    if (focus !== "all") {
      return data.providers.filter(
        (provider) => provider.provider === focus && providerHasVisibleDetails(provider, hiddenPanels)
      );
    }
    return data.providers.filter((provider) => providerHasVisibleDetails(provider, hiddenPanels));
  }
  function stateTone(state) {
    switch (state) {
      case "error":
        return "var(--accent)";
      case "incident":
        return "var(--warning)";
      case "degraded":
        return "var(--warning)";
      case "stale":
        return "var(--text-secondary)";
      default:
        return "var(--text-primary)";
    }
  }
  function stateLabel(state) {
    return `[${state.charAt(0).toUpperCase()}${state.slice(1)}]`;
  }
  function blockRunoutLabel(block) {
    const quota = block.quota;
    if (!quota || quota.runout_in_minutes == null) return null;
    if (quota.will_run_out_before_reset === false) {
      return `Reset before runout (${fmtResetTime(quota.runout_in_minutes)})`;
    }
    if (quota.runout_in_minutes <= 0) return "Runs out now";
    return `Runs out in ${fmtResetTime(quota.runout_in_minutes)}`;
  }
  function ProviderLaneCard({ provider }) {
    const hasAdminFallback = !!provider.claude_admin;
    return /* @__PURE__ */ u4("div", { class: "card", style: { display: "grid", gap: "14px" }, children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "flex-start" }, children: [
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", style: { marginBottom: "6px" }, children: provider.title }),
          /* @__PURE__ */ u4("div", { style: { fontSize: "28px", lineHeight: 1.1 }, children: fmtCostCompact(provider.today_cost_usd) }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: "Today cost" })
        ] }),
        /* @__PURE__ */ u4(
          "div",
          {
            style: {
              border: "1px solid var(--border-visible)",
              borderRadius: "var(--radius-pill)",
              padding: "2px 8px",
              fontFamily: "var(--font-mono)",
              fontSize: "var(--font-size-tertiary)",
              letterSpacing: 0,
              color: stateTone(provider.visual_state)
            },
            children: stateLabel(provider.visual_state)
          }
        )
      ] }),
      hasAdminFallback ? /* @__PURE__ */ u4("div", { style: { display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(140px,1fr))", gap: "12px" }, children: [
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Active users today" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "var(--font-size-value)" }, children: fmt(provider.claude_admin?.today_active_users ?? 0) })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Sessions today" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "var(--font-size-value)" }, children: fmt(provider.claude_admin?.today_sessions ?? 0) })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Accepted lines" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "var(--font-size-value)" }, children: fmt(provider.claude_admin?.lookback_lines_accepted ?? 0) }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: [
            provider.claude_admin?.lookback_days ?? 0,
            "d window"
          ] })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Estimated spend" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "var(--font-size-value)" }, children: fmtCostCompact(provider.claude_admin?.lookback_estimated_cost_usd ?? 0) }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: provider.claude_admin?.data_latency_note })
        ] })
      ] }) : /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "12px" }, children: [provider.primary, provider.secondary].filter(Boolean).map((window2, index) => /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "6px" }, children: [
        /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px" }, children: [
          /* @__PURE__ */ u4("span", { class: "stat-label", children: index === 0 ? "Primary" : "Secondary" }),
          /* @__PURE__ */ u4("span", { class: "stat-sub", children: [
            window2?.used_percent.toFixed(1),
            "% used"
          ] })
        ] }),
        /* @__PURE__ */ u4(
          SegmentedProgressBar,
          {
            value: window2?.used_percent ?? 0,
            max: 100,
            status: window2 && window2.used_percent >= 80 ? "accent" : window2 && window2.used_percent >= 50 ? "warning" : "success",
            "aria-label": `${provider.title} ${index === 0 ? "primary" : "secondary"} quota`
          }
        ),
        /* @__PURE__ */ u4("div", { class: "stat-sub", children: window2?.resets_in_minutes != null ? `Resets in ${fmtResetTime(window2.resets_in_minutes)}` : "No reset time available" })
      ] }, `${provider.provider}-${index}`)) }),
      /* @__PURE__ */ u4("div", { style: { display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(140px,1fr))", gap: "12px" }, children: [
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Weekly Projection" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "20px" }, children: provider.projected_weekly_spend_usd != null ? fmtCostCompact(provider.projected_weekly_spend_usd) : "\u2014" })
        ] }),
        /* @__PURE__ */ u4("div", { children: [
          /* @__PURE__ */ u4("div", { class: "stat-label", children: "Freshness" }),
          /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "20px" }, children: fmtRelativeTime(provider.last_refresh) }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: provider.last_refresh_label })
        ] })
      ] }),
      /* @__PURE__ */ u4("div", { style: { display: "grid", gap: "4px" }, children: [
        /* @__PURE__ */ u4("div", { class: "stat-sub", children: provider.source_label }),
        provider.identity_label && /* @__PURE__ */ u4("div", { class: "stat-sub", children: provider.identity_label }),
        provider.warnings.length > 0 && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { color: stateTone(provider.visual_state) }, children: provider.warnings[0] })
      ] })
    ] });
  }
  function BlockPanel({ block, density }) {
    const totalTokens2 = block.tokens.input + block.tokens.output + block.tokens.cache_read + block.tokens.cache_creation + block.tokens.reasoning_output;
    const d5 = densityTokens(density);
    const runout = blockRunoutLabel(block);
    return /* @__PURE__ */ u4("div", { class: "card stat-card", style: { padding: d5.padding }, children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
      /* @__PURE__ */ u4("div", { class: "stat-label", children: "Active Block" }),
      /* @__PURE__ */ u4("div", { class: "stat-value", children: fmt(totalTokens2) }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: [
        block.entry_count,
        " entries \xB7 ends ",
        new Date(block.end).toLocaleTimeString()
      ] }),
      block.burn_rate && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: [
        fmt(totalTokens2),
        " tokens \xB7 ",
        fmtCostCompact(block.burn_rate.cost_per_hour_nanos / 1e9),
        "/hr"
      ] }),
      block.projection && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: [
        "Projects ",
        fmt(block.projection.projected_tokens),
        " tokens \xB7 ",
        fmtCostCompact(block.projection.projected_cost_nanos / 1e9)
      ] }),
      block.quota && /* @__PURE__ */ u4("div", { style: { marginTop: d5.marginTop }, children: [
        /* @__PURE__ */ u4(
          SegmentedProgressBar,
          {
            value: block.quota.projected_pct * 100,
            max: 100,
            status: block.quota.projected_severity === "danger" ? "accent" : block.quota.projected_severity === "warn" ? "warning" : "success",
            "aria-label": "Projected billing block quota"
          }
        ),
        /* @__PURE__ */ u4("div", { class: "stat-sub", style: { marginTop: "8px", fontSize: d5.fontSize }, children: [
          Math.min(block.quota.projected_pct * 100, 999).toFixed(0),
          "% projected \xB7 ",
          fmt(block.quota.remaining_tokens),
          " tokens left"
        ] }),
        runout && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { marginTop: "4px", fontSize: d5.fontSize }, children: runout })
      ] })
    ] }) });
  }
  function ContextPanel({ data, density }) {
    const d5 = densityTokens(density);
    return /* @__PURE__ */ u4("div", { class: "card stat-card", style: { padding: d5.padding }, children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
      /* @__PURE__ */ u4("div", { class: "stat-label", children: "Context Window" }),
      /* @__PURE__ */ u4("div", { class: "stat-value", children: fmt(data.total_input_tokens) }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: [
        "of ",
        fmt(data.context_window_size),
        " \xB7 ",
        (data.pct * 100).toFixed(1),
        "%"
      ] }),
      /* @__PURE__ */ u4("div", { style: { marginTop: d5.marginTop }, children: /* @__PURE__ */ u4(
        SegmentedProgressBar,
        {
          value: data.total_input_tokens,
          max: data.context_window_size,
          status: data.severity === "danger" ? "accent" : data.severity === "warn" ? "warning" : "success",
          "aria-label": "Context window usage"
        }
      ) })
    ] }) });
  }
  function SessionPanel({ provider, density }) {
    if (!provider.recent_session) return null;
    const session = provider.recent_session;
    const d5 = densityTokens(density);
    return /* @__PURE__ */ u4("div", { class: "card stat-card", style: { padding: d5.padding }, children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
      /* @__PURE__ */ u4("div", { class: "stat-label", children: "Recent Session" }),
      /* @__PURE__ */ u4("div", { class: "stat-value", style: { fontSize: "22px" }, children: provider.title }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: session.display_name }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: [
        session.turns,
        " turns \xB7 ",
        session.duration_minutes,
        "m \xB7 ",
        fmtCostCompact(session.cost_usd)
      ] }),
      session.model && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: session.model })
    ] }) });
  }
  function QuotaSuggestionsPanel({
    provider,
    density
  }) {
    const suggestions = provider.quota_suggestions;
    if (!suggestions || suggestions.levels.length === 0) {
      return null;
    }
    const d5 = densityTokens(density);
    return /* @__PURE__ */ u4("div", { class: "card stat-card", style: { padding: d5.padding }, children: /* @__PURE__ */ u4("div", { class: "stat-content", children: [
      /* @__PURE__ */ u4("div", { class: "stat-label", children: "Suggested Quotas" }),
      /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: suggestions.sample_label }),
      /* @__PURE__ */ u4("div", { style: { display: "grid", gap: d5.listGap, marginTop: d5.marginTop }, children: suggestions.levels.map((level) => /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "baseline" }, children: [
        /* @__PURE__ */ u4("span", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: [
          level.label,
          level.key === suggestions.recommended_key && /* @__PURE__ */ u4("span", { style: { marginLeft: "6px", color: "var(--success)" }, children: "[RECOMMENDED]" })
        ] }),
        /* @__PURE__ */ u4("span", { class: "stat-value", style: { fontSize: "18px" }, children: fmt(level.limit_tokens) })
      ] }, level.key)) }),
      suggestions.note && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { marginTop: "10px", fontStyle: "italic", fontSize: d5.fontSize }, children: suggestions.note }),
      suggestions.sample_count !== suggestions.population_count && /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: [
        "Drawn from ",
        suggestions.population_count,
        " completed blocks, weighted toward near-limit history."
      ] })
    ] }) });
  }
  function ProviderDetails({
    provider,
    density,
    hiddenPanels
  }) {
    const d5 = densityTokens(density);
    return /* @__PURE__ */ u4("section", { style: { display: "grid", gap: d5.sectionGap }, children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: d5.headerGap, alignItems: "baseline", flexWrap: "wrap" }, children: [
        /* @__PURE__ */ u4("h2", { style: { margin: 0 }, children: [
          provider.title,
          " Details"
        ] }),
        /* @__PURE__ */ u4("div", { class: "stat-sub", style: { fontSize: d5.fontSize }, children: provider.last_refresh_label })
      ] }),
      /* @__PURE__ */ u4("div", { style: { display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(240px,1fr))", gap: d5.gridGap }, children: [
        !hiddenPanels.has("active_block") && provider.active_block && /* @__PURE__ */ u4(BlockPanel, { block: provider.active_block, density }),
        !hiddenPanels.has("predictive_insights") && provider.predictive_insights && /* @__PURE__ */ u4(PredictiveInsightsCard, { insights: provider.predictive_insights }),
        !hiddenPanels.has("depletion_forecast") && provider.depletion_forecast && /* @__PURE__ */ u4(DepletionForecastCard, { forecast: provider.depletion_forecast }),
        !hiddenPanels.has("quota_suggestions") && /* @__PURE__ */ u4(QuotaSuggestionsPanel, { provider, density }),
        !hiddenPanels.has("context_window") && provider.context_window && /* @__PURE__ */ u4(ContextPanel, { data: provider.context_window, density }),
        !hiddenPanels.has("recent_session") && /* @__PURE__ */ u4(SessionPanel, { provider, density })
      ] }),
      !hiddenPanels.has("warnings") && provider.warnings.length > 0 && /* @__PURE__ */ u4("div", { class: "card", style: { padding: "16px 18px" }, children: [
        /* @__PURE__ */ u4("div", { class: "stat-label", children: "Warnings" }),
        /* @__PURE__ */ u4("ul", { style: { margin: "10px 0 0", paddingLeft: "18px" }, children: provider.warnings.map((warning) => /* @__PURE__ */ u4("li", { children: warning }, warning)) })
      ] })
    ] });
  }
  function renderLiveMonitorView() {
    const data = liveMonitorData.value;
    if (!data) {
      return /* @__PURE__ */ u4("div", { class: "live-monitor", style: { display: "grid", gap: "var(--space-6)" }, children: /* @__PURE__ */ u4("section", { style: { display: "grid", gap: "var(--space-3)" }, children: [
        /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "var(--space-3)", alignItems: "baseline" }, children: /* @__PURE__ */ u4("h2", { style: { margin: 0 }, children: "Provider Lanes" }) }),
        /* @__PURE__ */ u4("div", { style: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(320px, 1fr))", gap: "var(--space-4)" }, children: [
          /* @__PURE__ */ u4(KpiSkeleton, { size: "hero", withBar: true, withSub: true }),
          /* @__PURE__ */ u4(KpiSkeleton, { size: "hero", withBar: true, withSub: true })
        ] })
      ] }) });
    }
    const laneProviders = providersForFocus(data, liveMonitorFocus.value);
    const hiddenPanels = new Set(liveMonitorHiddenPanels.value);
    const density = liveMonitorDensity.value;
    const details = detailProviders(data, liveMonitorFocus.value, hiddenPanels);
    return /* @__PURE__ */ u4("div", { class: "live-monitor", style: { display: "grid", gap: "24px" }, children: [
      /* @__PURE__ */ u4("section", { style: { display: "grid", gap: "14px" }, children: [
        /* @__PURE__ */ u4("div", { style: { display: "flex", justifyContent: "space-between", gap: "12px", alignItems: "baseline", flexWrap: "wrap" }, children: [
          /* @__PURE__ */ u4("h2", { style: { margin: 0 }, children: "Provider Lanes" }),
          /* @__PURE__ */ u4("div", { class: "stat-sub", children: data.freshness.has_stale_providers ? `${data.freshness.stale_providers.join(", ")} stale` : "All providers current" })
        ] }),
        /* @__PURE__ */ u4("div", { style: { display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(320px,1fr))", gap: "16px" }, children: laneProviders.map((provider) => /* @__PURE__ */ u4(ProviderLaneCard, { provider }, provider.provider)) })
      ] }),
      details.map((provider) => /* @__PURE__ */ u4(
        ProviderDetails,
        {
          provider,
          density,
          hiddenPanels
        },
        `details-${provider.provider}`
      ))
    ] });
  }

  // src/ui/monitor/runtime.ts
  function createLiveMonitorRuntime() {
    let intervalId = null;
    let eventSource = null;
    let visibilityHandler = null;
    const mount2 = $2("main-content");
    function renderView() {
      R(renderLiveMonitorView(), mount2);
    }
    async function loadData2() {
      liveMonitorRefreshing.value = true;
      try {
        const tzOffset = (/* @__PURE__ */ new Date()).getTimezoneOffset() * -1;
        const response = await fetch(`/api/live-monitor?tz_offset_min=${tzOffset}`);
        if (!response.ok) {
          throw new Error(`Monitor request failed (${response.status})`);
        }
        setLiveMonitorData(await response.json());
      } catch (error) {
        liveMonitorError.value = error instanceof Error ? error.message : "Live monitor refresh failed";
      } finally {
        liveMonitorRefreshing.value = false;
        renderView();
      }
    }
    function toggleVisibility(hidden) {
      const filterMount = document.getElementById("filter-bar-mount");
      const tabsMount = document.getElementById("dashboard-tabs-mount");
      if (filterMount) filterMount.style.display = hidden ? "none" : "";
      if (tabsMount) tabsMount.style.display = hidden ? "none" : "";
    }
    function subscribeToStream() {
      if (typeof EventSource === "undefined") return;
      eventSource = new EventSource("/api/stream");
      eventSource.addEventListener("scan_completed", () => {
        void loadData2();
      });
    }
    function start() {
      document.title = "Live Monitor";
      toggleVisibility(true);
      renderView();
      void loadData2();
      visibilityHandler = () => {
        if (!document.hidden) {
          void loadData2();
        }
      };
      document.addEventListener("visibilitychange", visibilityHandler);
      intervalId = window.setInterval(() => {
        if (!document.hidden) {
          void loadData2();
        }
      }, 1e4);
      subscribeToStream();
    }
    function stop() {
      if (intervalId != null) {
        window.clearInterval(intervalId);
      }
      intervalId = null;
      eventSource?.close();
      eventSource = null;
      if (visibilityHandler) {
        document.removeEventListener("visibilitychange", visibilityHandler);
      }
      visibilityHandler = null;
      toggleVisibility(false);
    }
    return { loadData: loadData2, start, stop };
  }

  // src/ui/tool_errors/store.ts
  var toolName = y3("");
  var providerFilter = y3("");
  var rangeFilter = y3("30d");
  var pageOffset = y3(0);
  var rows = y3([]);
  var total = y3(0);
  var loadState2 = y3("idle");
  var errorMessage = y3(null);
  var PAGE_SIZE = 100;
  function readUrlParams() {
    const p5 = new URLSearchParams(window.location.search);
    toolName.value = p5.get("tool") ?? "";
    providerFilter.value = p5.get("provider") ?? "";
    rangeFilter.value = p5.get("range") ?? "30d";
    const off = Number.parseInt(p5.get("offset") ?? "0", 10);
    pageOffset.value = Number.isFinite(off) && off >= 0 ? off : 0;
  }
  function syncUrl() {
    const p5 = new URLSearchParams();
    if (toolName.value) p5.set("tool", toolName.value);
    if (providerFilter.value) p5.set("provider", providerFilter.value);
    if (rangeFilter.value !== "30d") p5.set("range", rangeFilter.value);
    if (pageOffset.value > 0) p5.set("offset", String(pageOffset.value));
    const next = `${window.location.pathname}?${p5.toString()}`;
    window.history.replaceState(null, "", next);
  }

  // src/ui/tool_errors/ToolErrorsTable.tsx
  var expandedInputs = y3(/* @__PURE__ */ new Set());
  var expandedErrors = y3(/* @__PURE__ */ new Set());
  function toggle(set, key) {
    const next = new Set(set.value);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    set.value = next;
  }
  function ExpandableCell({ value, rowKey, store }) {
    if (!value) return /* @__PURE__ */ u4("span", { class: "dim", children: "\u2014" });
    const PREVIEW = 200;
    const isLong = value.length > PREVIEW;
    const isExpanded = store.value.has(rowKey);
    const display = isLong && !isExpanded ? value.slice(0, PREVIEW) + "\u2026" : value;
    return /* @__PURE__ */ u4("div", { children: [
      /* @__PURE__ */ u4(
        "pre",
        {
          style: {
            margin: 0,
            whiteSpace: "pre-wrap",
            wordBreak: "break-all",
            fontFamily: "var(--font-mono)",
            fontSize: "11px",
            color: "var(--color-text-secondary)",
            maxHeight: isExpanded ? "none" : "4.5em",
            overflow: "hidden"
          },
          dangerouslySetInnerHTML: { __html: esc(display) }
        }
      ),
      isLong && /* @__PURE__ */ u4(
        "button",
        {
          type: "button",
          class: "table-action-btn",
          style: { fontSize: "11px", marginTop: "2px" },
          onClick: () => toggle(store, rowKey),
          children: isExpanded ? "show less" : "show full"
        }
      )
    ] });
  }
  function makeColumns4() {
    return [
      {
        accessorKey: "timestamp",
        header: "Timestamp",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "num muted", children: String(getValue()) })
      },
      {
        accessorKey: "project",
        header: "Project",
        cell: ({ getValue }) => /* @__PURE__ */ u4("span", { class: "muted", style: { wordBreak: "break-all" }, children: String(getValue()) })
      },
      {
        accessorKey: "session_id",
        header: "Session",
        cell: ({ getValue }) => {
          const v4 = String(getValue());
          return /* @__PURE__ */ u4("span", { class: "num muted", title: v4, children: v4.slice(-12) });
        }
      },
      {
        accessorKey: "model",
        header: "Model",
        cell: ({ getValue }) => {
          const v4 = String(getValue());
          return v4 ? /* @__PURE__ */ u4("span", { class: "model-tag", children: v4 }) : /* @__PURE__ */ u4("span", { class: "dim", children: "\u2014" });
        }
      },
      {
        accessorKey: "mcp_server",
        header: "MCP Server",
        cell: ({ getValue }) => {
          const v4 = getValue();
          return v4 ? /* @__PURE__ */ u4("span", { class: "muted", children: v4 }) : /* @__PURE__ */ u4("span", { class: "dim", children: "\u2014" });
        }
      },
      {
        id: "tool_input",
        header: "Input",
        cell: ({ row }) => /* @__PURE__ */ u4(
          ExpandableCell,
          {
            value: row.original.tool_input,
            rowKey: `input-${row.index}`,
            store: expandedInputs
          }
        )
      },
      {
        id: "error_text",
        header: "Error",
        cell: ({ row }) => /* @__PURE__ */ u4(
          ExpandableCell,
          {
            value: row.original.error_text ?? "(no message captured \u2014 db reset to backfill)",
            rowKey: `err-${row.index}`,
            store: expandedErrors
          }
        )
      }
    ];
  }
  function ToolErrorsTable({ data }) {
    if (!data.length) return null;
    return /* @__PURE__ */ u4(
      DataTable,
      {
        columns: makeColumns4(),
        data,
        title: "Error details",
        sectionKey: "tool-errors-detail"
      }
    );
  }

  // src/ui/tool_errors/ToolErrorsPage.tsx
  var RANGE_OPTIONS = ["7d", "30d", "90d", "all"];
  function ToolErrorsPage({ onLoad }) {
    const name = toolName.value;
    const count2 = total.value;
    const offset = pageOffset.value;
    const state = loadState2.value;
    const err = errorMessage.value;
    const data = rows.value;
    const hasNullErrors = data.some((r4) => r4.error_text === null);
    const totalPages = Math.ceil(count2 / PAGE_SIZE);
    const currentPage = Math.floor(offset / PAGE_SIZE);
    function navigate(newOffset) {
      pageOffset.value = newOffset;
      void onLoad();
    }
    return /* @__PURE__ */ u4("div", { style: { maxWidth: "1400px", margin: "0 auto", padding: "24px" }, children: [
      /* @__PURE__ */ u4("div", { style: { display: "flex", alignItems: "center", gap: "16px", marginBottom: "20px", flexWrap: "wrap" }, children: [
        /* @__PURE__ */ u4(
          "a",
          {
            href: "/",
            style: { color: "var(--color-text-secondary)", textDecoration: "none", fontSize: "13px" },
            children: "\u2190 Dashboard"
          }
        ),
        /* @__PURE__ */ u4("h1", { style: { margin: 0, fontSize: "18px", fontWeight: 600 }, children: [
          name,
          " \u2014 error details"
        ] }),
        state !== "loading" && /* @__PURE__ */ u4("span", { class: "muted", style: { fontSize: "13px" }, children: [
          fmt(count2),
          " errors total"
        ] }),
        state === "loading" && /* @__PURE__ */ u4("span", { class: "muted", style: { fontSize: "13px" }, children: "[loading\u2026]" })
      ] }),
      /* @__PURE__ */ u4("div", { style: { display: "flex", gap: "12px", alignItems: "center", marginBottom: "16px", flexWrap: "wrap" }, children: [
        /* @__PURE__ */ u4("label", { style: { fontSize: "13px", color: "var(--color-text-secondary)" }, children: [
          "Range:",
          /* @__PURE__ */ u4(
            "select",
            {
              value: rangeFilter.value,
              onChange: (e4) => {
                rangeFilter.value = e4.target.value;
                pageOffset.value = 0;
                void onLoad();
              },
              style: { marginLeft: "6px", background: "var(--card-bg)", color: "var(--color-text-primary)", border: "1px solid var(--border)", borderRadius: "4px", padding: "2px 6px", fontSize: "13px" },
              children: RANGE_OPTIONS.map((r4) => /* @__PURE__ */ u4("option", { value: r4, children: r4 }, r4))
            }
          )
        ] }),
        /* @__PURE__ */ u4("label", { style: { fontSize: "13px", color: "var(--color-text-secondary)" }, children: [
          "Provider:",
          /* @__PURE__ */ u4(
            "input",
            {
              type: "text",
              value: providerFilter.value,
              placeholder: "all",
              onInput: (e4) => {
                providerFilter.value = e4.target.value;
                pageOffset.value = 0;
                void onLoad();
              },
              style: { marginLeft: "6px", width: "80px", background: "var(--card-bg)", color: "var(--color-text-primary)", border: "1px solid var(--border)", borderRadius: "4px", padding: "2px 6px", fontSize: "13px" }
            }
          )
        ] })
      ] }),
      err && /* @__PURE__ */ u4("div", { class: "card", style: { color: "var(--accent)", padding: "16px", marginBottom: "16px" }, children: [
        "[Error: ",
        err,
        "]"
      ] }),
      hasNullErrors && /* @__PURE__ */ u4("div", { class: "card", style: { fontSize: "12px", color: "var(--color-text-secondary)", padding: "10px 16px", marginBottom: "12px" }, children: [
        "[Note: some rows have no error message \u2014 run ",
        /* @__PURE__ */ u4("code", { children: "cargo run -- db reset --yes && cargo run -- scan" }),
        " to capture pre-upgrade errors]"
      ] }),
      data.length > 0 && /* @__PURE__ */ u4(ToolErrorsTable, { data }),
      data.length === 0 && state === "loading" && /* @__PURE__ */ u4(TableSkeleton, { rows: 8, columns: 5 }),
      data.length === 0 && state === "idle" && /* @__PURE__ */ u4("p", { class: "muted", children: "No errors found for the selected filters." }),
      totalPages > 1 && /* @__PURE__ */ u4("div", { style: { display: "flex", gap: "8px", alignItems: "center", marginTop: "16px", fontSize: "13px" }, children: [
        /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: "table-action-btn",
            disabled: currentPage === 0,
            onClick: () => navigate(Math.max(0, offset - PAGE_SIZE)),
            children: "\u2190 Prev"
          }
        ),
        /* @__PURE__ */ u4("span", { class: "muted", children: [
          "Page ",
          currentPage + 1,
          " of ",
          totalPages
        ] }),
        /* @__PURE__ */ u4(
          "button",
          {
            type: "button",
            class: "table-action-btn",
            disabled: offset + PAGE_SIZE >= count2,
            onClick: () => navigate(offset + PAGE_SIZE),
            children: "Next \u2192"
          }
        )
      ] })
    ] });
  }

  // src/ui/tool_errors/runtime.tsx
  function rangeToDateBounds(range) {
    if (range === "all") return {};
    const days = range === "7d" ? 7 : range === "90d" ? 90 : 30;
    const end = /* @__PURE__ */ new Date();
    const start = /* @__PURE__ */ new Date();
    start.setDate(start.getDate() - days);
    const fmt2 = (d5) => d5.toISOString().slice(0, 10);
    return { start: fmt2(start), end: fmt2(end) };
  }
  function renderPage() {
    const mount2 = document.getElementById("main-content");
    if (mount2) R(/* @__PURE__ */ u4(ToolErrorsPage, { onLoad: loadData }), mount2);
  }
  async function loadData() {
    loadState2.value = "loading";
    errorMessage.value = null;
    syncUrl();
    try {
      const tzOffset = (/* @__PURE__ */ new Date()).getTimezoneOffset() * -1;
      const p5 = new URLSearchParams();
      p5.set("tool", toolName.value);
      if (providerFilter.value) p5.set("provider", providerFilter.value);
      p5.set("limit", String(PAGE_SIZE));
      p5.set("offset", String(pageOffset.value));
      p5.set("tz_offset_min", String(tzOffset));
      const { start, end } = rangeToDateBounds(rangeFilter.value);
      if (start) p5.set("start", start);
      if (end) p5.set("end", end);
      const resp = await fetch(`/api/tool-errors?${p5.toString()}`);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data = await resp.json();
      rows.value = data.rows;
      total.value = data.total;
      loadState2.value = "idle";
    } catch (err) {
      errorMessage.value = err instanceof Error ? err.message : "Failed to load errors";
      loadState2.value = "error";
    }
    renderPage();
  }
  function startToolErrorsPage() {
    readUrlParams();
    const filterBar = document.getElementById("filter-bar-mount");
    const tabsMount = document.getElementById("dashboard-tabs-mount");
    if (filterBar) filterBar.style.display = "none";
    if (tabsMount) tabsMount.style.display = "none";
    document.title = toolName.value ? `${toolName.value} Errors` : "Tool Errors";
    renderPage();
    void loadData();
  }

  // src/ui/lib/theme.ts
  function getTheme() {
    const stored = localStorage.getItem("theme");
    if (stored === "light" || stored === "dark") return stored;
    return "light";
  }
  function applyTheme(theme) {
    if (theme === "light") {
      document.documentElement.setAttribute("data-theme", "light");
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
    themeMode.value = theme;
  }

  // src/ui/lib/version-poll.ts
  var LOCK_NAME = "heimdall-version-poll";
  var CHANNEL_NAME = "heimdall-version-poll";
  var MIN_SPINNER_MS = 1200;
  var MIN_DELAY_MS = 5e3;
  var FALLBACK_DELAY_MS = 5 * 60 * 1e3;
  var POST_POLL_BUFFER_MS = 500;
  function ensureState() {
    if (!window.__heimdallVersionPoll) {
      window.__heimdallVersionPoll = { started: false, isLeader: false, timer: null, bc: null };
    }
    return window.__heimdallVersionPoll;
  }
  async function fetchOnce() {
    const t0 = Date.now();
    versionChecking.value = true;
    try {
      const res = await fetch("/api/version");
      if (!res.ok) return null;
      const info = await res.json();
      versionInfo.value = info;
      return info;
    } catch (_4) {
      return null;
    } finally {
      const elapsed = Date.now() - t0;
      if (elapsed < MIN_SPINNER_MS) {
        await new Promise((r4) => setTimeout(r4, MIN_SPINNER_MS - elapsed));
      }
      versionChecking.value = false;
    }
  }
  function broadcastInfo(info) {
    const st = ensureState();
    st.bc?.postMessage({ type: "version-data", payload: info });
  }
  function scheduleNext(info) {
    const st = ensureState();
    if (st.timer) clearTimeout(st.timer);
    let delay = FALLBACK_DELAY_MS;
    if (info?.next_check_at) {
      const t4 = Date.parse(info.next_check_at);
      if (!Number.isNaN(t4)) {
        delay = Math.max(MIN_DELAY_MS, t4 + POST_POLL_BUFFER_MS - Date.now());
      }
    }
    st.timer = setTimeout(() => {
      void tick();
    }, delay);
  }
  async function tick() {
    const info = await fetchOnce();
    if (info) broadcastInfo(info);
    scheduleNext(info);
  }
  function startVersionPoll() {
    const st = ensureState();
    if (st.started) return;
    st.started = true;
    st.bc = new BroadcastChannel(CHANNEL_NAME);
    st.bc.onmessage = (ev) => {
      if (ev.data?.type === "version-data") {
        versionInfo.value = ev.data.payload;
      } else if (ev.data?.type === "poke" && st.isLeader) {
        void tick();
      } else if (ev.data?.type === "hello" && st.isLeader && versionInfo.value) {
        broadcastInfo(versionInfo.value);
      }
    };
    st.bc.postMessage({ type: "hello" });
    navigator.locks.request(LOCK_NAME, { mode: "exclusive" }, async () => {
      st.isLeader = true;
      await tick();
      await new Promise(() => {
      });
    }).catch(() => {
    });
  }

  // node_modules/gridstack/dist/utils.js
  var Utils = class _Utils {
    /**
     * Convert a potential selector into an actual list of HTML elements.
     * Supports CSS selectors, element references, and special ID handling.
     *
     * @param els selector string, HTMLElement, or array of elements
     * @param root optional root element to search within (defaults to document, useful for shadow DOM)
     * @returns array of HTML elements matching the selector
     *
     * @example
     * const elements = Utils.getElements('.grid-item');
     * const byId = Utils.getElements('#myWidget');
     * const fromShadow = Utils.getElements('.item', shadowRoot);
     */
    static getElements(els, root = document) {
      if (typeof els === "string") {
        const doc = "getElementById" in root ? root : void 0;
        if (doc && !isNaN(+els[0])) {
          const el = doc.getElementById(els);
          return el ? [el] : [];
        }
        let list = root.querySelectorAll(els);
        if (!list.length && els[0] !== "." && els[0] !== "#") {
          list = root.querySelectorAll("." + els);
          if (!list.length)
            list = root.querySelectorAll("#" + els);
          if (!list.length) {
            const el = root.querySelector(`[gs-id="${els}"]`);
            return el ? [el] : [];
          }
        }
        return Array.from(list);
      }
      return [els];
    }
    /**
     * Convert a potential selector into a single HTML element.
     * Similar to getElements() but returns only the first match.
     *
     * @param els selector string or HTMLElement
     * @param root optional root element to search within (defaults to document)
     * @returns the first HTML element matching the selector, or null if not found
     *
     * @example
     * const element = Utils.getElement('#myWidget');
     * const first = Utils.getElement('.grid-item');
     */
    static getElement(els, root = document) {
      if (typeof els === "string") {
        const doc = "getElementById" in root ? root : void 0;
        if (!els.length)
          return null;
        if (doc && els[0] === "#") {
          return doc.getElementById(els.substring(1));
        }
        if (els[0] === "#" || els[0] === "." || els[0] === "[") {
          return root.querySelector(els);
        }
        if (doc && !isNaN(+els[0])) {
          return doc.getElementById(els);
        }
        let el = root.querySelector(els);
        if (doc && !el) {
          el = doc.getElementById(els);
        }
        if (!el) {
          el = root.querySelector("." + els);
        }
        return el;
      }
      return els;
    }
    /**
     * Check if a widget should be lazy loaded based on node or grid settings.
     *
     * @param n the grid node to check
     * @returns true if the item should be lazy loaded
     *
     * @example
     * if (Utils.lazyLoad(node)) {
     *   // Set up intersection observer for lazy loading
     * }
     */
    static lazyLoad(n3) {
      return n3.lazyLoad || n3.grid?.opts?.lazyLoad && n3.lazyLoad !== false;
    }
    /**
     * Create a div element with the specified CSS classes.
     *
     * @param classes array of CSS class names to add
     * @param parent optional parent element to append the div to
     * @returns the created div element
     *
     * @example
     * const div = Utils.createDiv(['grid-item', 'draggable']);
     * const nested = Utils.createDiv(['content'], parentDiv);
     */
    static createDiv(classes, parent) {
      const el = document.createElement("div");
      classes.forEach((c4) => {
        if (c4)
          el.classList.add(c4);
      });
      parent?.appendChild(el);
      return el;
    }
    /**
     * Check if a widget should resize to fit its content.
     *
     * @param n the grid node to check (can be undefined)
     * @param strict if true, only returns true for explicit sizeToContent:true (not numbers)
     * @returns true if the widget should resize to content
     *
     * @example
     * if (Utils.shouldSizeToContent(node)) {
     *   // Trigger content-based resizing
     * }
     */
    static shouldSizeToContent(n3, strict = false) {
      return n3?.grid && (strict ? n3.sizeToContent === true || n3.grid.opts.sizeToContent === true && n3.sizeToContent === void 0 : !!n3.sizeToContent || n3.grid.opts.sizeToContent && n3.sizeToContent !== false);
    }
    /**
     * Check if two grid positions overlap/intersect.
     *
     * @param a first position with x, y, w, h properties
     * @param b second position with x, y, w, h properties
     * @returns true if the positions overlap
     *
     * @example
     * const overlaps = Utils.isIntercepted(
     *   {x: 0, y: 0, w: 2, h: 1},
     *   {x: 1, y: 0, w: 2, h: 1}
     * ); // true - they overlap
     */
    static isIntercepted(a4, b4) {
      return !(a4.y >= b4.y + b4.h || a4.y + a4.h <= b4.y || a4.x + a4.w <= b4.x || a4.x >= b4.x + b4.w);
    }
    /**
     * Check if two grid positions are touching (edges or corners).
     *
     * @param a first position
     * @param b second position
     * @returns true if the positions are touching
     *
     * @example
     * const touching = Utils.isTouching(
     *   {x: 0, y: 0, w: 2, h: 1},
     *   {x: 2, y: 0, w: 1, h: 1}
     * ); // true - they share an edge
     */
    static isTouching(a4, b4) {
      return _Utils.isIntercepted(a4, { x: b4.x - 0.5, y: b4.y - 0.5, w: b4.w + 1, h: b4.h + 1 });
    }
    /**
     * Calculate the overlapping area between two grid positions.
     *
     * @param a first position
     * @param b second position
     * @returns the area of overlap (0 if no overlap)
     *
     * @example
     * const overlap = Utils.areaIntercept(
     *   {x: 0, y: 0, w: 3, h: 2},
     *   {x: 1, y: 0, w: 3, h: 2}
     * ); // returns 4 (2x2 overlap)
     */
    static areaIntercept(a4, b4) {
      const x0 = a4.x > b4.x ? a4.x : b4.x;
      const x1 = a4.x + a4.w < b4.x + b4.w ? a4.x + a4.w : b4.x + b4.w;
      if (x1 <= x0)
        return 0;
      const y0 = a4.y > b4.y ? a4.y : b4.y;
      const y1 = a4.y + a4.h < b4.y + b4.h ? a4.y + a4.h : b4.y + b4.h;
      if (y1 <= y0)
        return 0;
      return (x1 - x0) * (y1 - y0);
    }
    /**
     * Calculate the total area of a grid position.
     *
     * @param a position with width and height
     * @returns the total area (width * height)
     *
     * @example
     * const area = Utils.area({x: 0, y: 0, w: 3, h: 2}); // returns 6
     */
    static area(a4) {
      return a4.w * a4.h;
    }
    /**
     * Sort an array of grid nodes by position (y first, then x).
     *
     * @param nodes array of nodes to sort
     * @param dir sort direction: 1 for ascending (top-left first), -1 for descending
     * @returns the sorted array (modifies original)
     *
     * @example
     * const sorted = Utils.sort(nodes); // Sort top-left to bottom-right
     * const reverse = Utils.sort(nodes, -1); // Sort bottom-right to top-left
     */
    static sort(nodes, dir = 1) {
      const und = 1e4;
      return nodes.sort((a4, b4) => {
        const diffY = dir * ((a4.y ?? und) - (b4.y ?? und));
        if (diffY === 0)
          return dir * ((a4.x ?? und) - (b4.x ?? und));
        return diffY;
      });
    }
    /**
     * Find a grid node by its ID.
     *
     * @param nodes array of nodes to search
     * @param id the ID to search for
     * @returns the node with matching ID, or undefined if not found
     *
     * @example
     * const node = Utils.find(nodes, 'widget-1');
     * if (node) console.log('Found node at:', node.x, node.y);
     */
    static find(nodes, id) {
      return id ? nodes.find((n3) => n3.id === id) : void 0;
    }
    /**
     * Convert various value types to boolean.
     * Handles strings like 'false', 'no', '0' as false.
     *
     * @param v value to convert
     * @returns boolean representation
     *
     * @example
     * Utils.toBool('true');  // true
     * Utils.toBool('false'); // false
     * Utils.toBool('no');    // false
     * Utils.toBool('1');     // true
     */
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    static toBool(v4) {
      if (typeof v4 === "boolean") {
        return v4;
      }
      if (typeof v4 === "string") {
        v4 = v4.toLowerCase();
        return !(v4 === "" || v4 === "no" || v4 === "false" || v4 === "0");
      }
      return Boolean(v4);
    }
    /**
     * Convert a string value to a number, handling null and empty strings.
     *
     * @param value string or null value to convert
     * @returns number value, or undefined for null/empty strings
     *
     * @example
     * Utils.toNumber('42');  // 42
     * Utils.toNumber('');    // undefined
     * Utils.toNumber(null);  // undefined
     */
    static toNumber(value) {
      return value === null || value.length === 0 ? void 0 : Number(value);
    }
    /**
     * Parse a height value with units into numeric value and unit string.
     * Supports px, em, rem, vh, vw, %, cm, mm units.
     *
     * @param val height value as number or string with units
     * @returns object with h (height) and unit properties
     *
     * @example
     * Utils.parseHeight('100px');  // {h: 100, unit: 'px'}
     * Utils.parseHeight('2rem');   // {h: 2, unit: 'rem'}
     * Utils.parseHeight(50);       // {h: 50, unit: 'px'}
     */
    static parseHeight(val) {
      let h5;
      let unit = "px";
      if (typeof val === "string") {
        if (val === "auto" || val === "")
          h5 = 0;
        else {
          const match = val.match(/^(-[0-9]+\.[0-9]+|[0-9]*\.[0-9]+|-[0-9]+|[0-9]+)(px|em|rem|vh|vw|%|cm|mm)?$/);
          if (!match) {
            throw new Error(`Invalid height val = ${val}`);
          }
          unit = match[2] || "px";
          h5 = parseFloat(match[1]);
        }
      } else {
        h5 = val;
      }
      return { h: h5, unit };
    }
    /**
     * Copy unset fields from source objects to target object (shallow merge with defaults).
     * Similar to Object.assign but only sets undefined/null fields.
     *
     * @param target the object to copy defaults into
     * @param sources one or more source objects to copy defaults from
     * @returns the modified target object
     *
     * @example
     * const config = { width: 100 };
     * Utils.defaults(config, { width: 200, height: 50 });
     * // config is now { width: 100, height: 50 }
     */
    // eslint-disable-next-line
    static defaults(target, ...sources) {
      sources.forEach((source) => {
        for (const key in source) {
          if (!source.hasOwnProperty(key))
            return;
          if (target[key] === null || target[key] === void 0) {
            target[key] = source[key];
          } else if (typeof source[key] === "object" && typeof target[key] === "object") {
            _Utils.defaults(target[key], source[key]);
          }
        }
      });
      return target;
    }
    /**
     * Compare two objects for equality (shallow comparison).
     * Checks if objects have the same fields and values at one level deep.
     *
     * @param a first object to compare
     * @param b second object to compare
     * @returns true if objects have the same values
     *
     * @example
     * Utils.same({x: 1, y: 2}, {x: 1, y: 2}); // true
     * Utils.same({x: 1}, {x: 1, y: 2}); // false
     */
    static same(a4, b4) {
      if (typeof a4 !== "object")
        return a4 == b4;
      if (typeof a4 !== typeof b4)
        return false;
      if (Object.keys(a4).length !== Object.keys(b4).length)
        return false;
      for (const key in a4) {
        if (a4[key] !== b4[key])
          return false;
      }
      return true;
    }
    /**
     * Copy position and size properties from one widget to another.
     * Copies x, y, w, h and optionally min/max constraints.
     *
     * @param a target widget to copy to
     * @param b source widget to copy from
     * @param doMinMax if true, also copy min/max width/height constraints
     * @returns the target widget (a)
     *
     * @example
     * Utils.copyPos(widget1, widget2); // Copy position/size
     * Utils.copyPos(widget1, widget2, true); // Also copy constraints
     */
    static copyPos(a4, b4, doMinMax = false) {
      if (b4.x !== void 0)
        a4.x = b4.x;
      if (b4.y !== void 0)
        a4.y = b4.y;
      if (b4.w !== void 0)
        a4.w = b4.w;
      if (b4.h !== void 0)
        a4.h = b4.h;
      if (doMinMax) {
        if (b4.minW)
          a4.minW = b4.minW;
        if (b4.minH)
          a4.minH = b4.minH;
        if (b4.maxW)
          a4.maxW = b4.maxW;
        if (b4.maxH)
          a4.maxH = b4.maxH;
      }
      return a4;
    }
    /** true if a and b has same size & position */
    static samePos(a4, b4) {
      return a4 && b4 && a4.x === b4.x && a4.y === b4.y && (a4.w || 1) === (b4.w || 1) && (a4.h || 1) === (b4.h || 1);
    }
    /** given a node, makes sure it's min/max are valid */
    static sanitizeMinMax(node) {
      if (!node.minW) {
        delete node.minW;
      }
      if (!node.minH) {
        delete node.minH;
      }
      if (!node.maxW) {
        delete node.maxW;
      }
      if (!node.maxH) {
        delete node.maxH;
      }
    }
    /** removes field from the first object if same as the second objects (like diffing) and internal '_' for saving */
    static removeInternalAndSame(a4, b4) {
      if (typeof a4 !== "object" || typeof b4 !== "object")
        return;
      if (Array.isArray(a4) || Array.isArray(b4))
        return;
      for (let key in a4) {
        const aVal = a4[key];
        const bVal = b4[key];
        if (key[0] === "_" || aVal === bVal) {
          delete a4[key];
        } else if (aVal && typeof aVal === "object" && bVal !== void 0) {
          _Utils.removeInternalAndSame(aVal, bVal);
          if (!Object.keys(aVal).length) {
            delete a4[key];
          }
        }
      }
    }
    /** removes internal fields '_' and default values for saving */
    static removeInternalForSave(n3, removeEl = true) {
      for (let key in n3) {
        if (key[0] === "_" || n3[key] === null || n3[key] === void 0)
          delete n3[key];
      }
      delete n3.grid;
      if (removeEl)
        delete n3.el;
      if (!n3.autoPosition)
        delete n3.autoPosition;
      if (!n3.noResize)
        delete n3.noResize;
      if (!n3.noMove)
        delete n3.noMove;
      if (!n3.locked)
        delete n3.locked;
      if (n3.w === 1 || n3.w === n3.minW)
        delete n3.w;
      if (n3.h === 1 || n3.h === n3.minH)
        delete n3.h;
    }
    /** return the closest parent (or itself) matching the given class */
    // static closestUpByClass(el: HTMLElement, name: string): HTMLElement {
    //   while (el) {
    //     if (el.classList.contains(name)) return el;
    //     el = el.parentElement
    //   }
    //   return null;
    // }
    /** delay calling the given function for given delay, preventing new calls from happening while waiting */
    static throttle(func, delay) {
      let isWaiting = false;
      return (...args) => {
        if (!isWaiting) {
          isWaiting = true;
          setTimeout(() => {
            func(...args);
            isWaiting = false;
          }, delay);
        }
      };
    }
    static removePositioningStyles(el) {
      const style = el.style;
      if (style.position) {
        style.removeProperty("position");
      }
      if (style.left) {
        style.removeProperty("left");
      }
      if (style.top) {
        style.removeProperty("top");
      }
      if (style.width) {
        style.removeProperty("width");
      }
      if (style.height) {
        style.removeProperty("height");
      }
    }
    /** @internal returns the passed element if vertically scrollable, else the closest parent that will, up to the entire document scrolling element */
    static getScrollElement(el) {
      if (!el)
        return document.scrollingElement || document.documentElement;
      const overflowY = getComputedStyle(el).overflowY;
      if ((overflowY === "auto" || overflowY === "scroll") && el.scrollHeight > el.clientHeight) {
        return el;
      } else {
        return _Utils.getScrollElement(el.parentElement);
      }
    }
    /**
     * @internal Function used to scroll the page.
     *
     * @param event `MouseEvent` that triggers the resize
     * @param el `HTMLElement` that's being resized
     * @param distance Distance from the V edges to start scrolling
     */
    static updateScrollResize(event, el, distance) {
      const scrollEl = _Utils.getScrollElement(el);
      const height = scrollEl.clientHeight;
      const offsetTop = scrollEl === _Utils.getScrollElement() ? 0 : scrollEl.getBoundingClientRect().top;
      const pointerPosY = event.clientY - offsetTop;
      const top = pointerPosY < distance;
      const bottom = pointerPosY > height - distance;
      if (top) {
        scrollEl.scrollBy({ behavior: "smooth", top: pointerPosY - distance });
      } else if (bottom) {
        scrollEl.scrollBy({ behavior: "smooth", top: distance - (height - pointerPosY) });
      }
    }
    /** single level clone, returning a new object with same top fields. This will share sub objects and arrays */
    static clone(obj) {
      if (obj === null || obj === void 0 || typeof obj !== "object") {
        return obj;
      }
      if (obj instanceof Array) {
        return [...obj];
      }
      return { ...obj };
    }
    /**
     * Recursive clone version that returns a full copy, checking for nested objects and arrays ONLY.
     * Note: this will use as-is any key starting with double __ (and not copy inside) some lib have circular dependencies.
     */
    static cloneDeep(obj) {
      const skipFields = ["parentGrid", "el", "grid", "subGrid", "engine"];
      const ret = _Utils.clone(obj);
      for (const key in ret) {
        if (ret.hasOwnProperty(key) && typeof ret[key] === "object" && key.substring(0, 2) !== "__" && !skipFields.find((k4) => k4 === key)) {
          ret[key] = _Utils.cloneDeep(obj[key]);
        }
      }
      return ret;
    }
    /** deep clone the given HTML node, removing teh unique id field */
    static cloneNode(el) {
      const node = el.cloneNode(true);
      node.removeAttribute("id");
      return node;
    }
    static appendTo(el, parent) {
      let parentNode;
      if (typeof parent === "string") {
        parentNode = _Utils.getElement(parent);
      } else {
        parentNode = parent;
      }
      if (parentNode) {
        parentNode.appendChild(el);
      }
    }
    // public static setPositionRelative(el: HTMLElement): void {
    //   if (!(/^(?:r|a|f)/).test(getComputedStyle(el).position)) {
    //     el.style.position = "relative";
    //   }
    // }
    static addElStyles(el, styles) {
      if (styles instanceof Object) {
        for (const s4 in styles) {
          if (styles.hasOwnProperty(s4)) {
            if (Array.isArray(styles[s4])) {
              styles[s4].forEach((val) => {
                el.style[s4] = val;
              });
            } else {
              el.style[s4] = styles[s4];
            }
          }
        }
      }
    }
    static initEvent(e4, info) {
      const evt = { type: info.type };
      const obj = {
        button: 0,
        which: 0,
        buttons: 1,
        bubbles: true,
        cancelable: true,
        target: info.target ? info.target : e4.target
      };
      ["altKey", "ctrlKey", "metaKey", "shiftKey"].forEach((p5) => evt[p5] = e4[p5]);
      ["pageX", "pageY", "clientX", "clientY", "screenX", "screenY"].forEach((p5) => evt[p5] = e4[p5]);
      return { ...evt, ...obj };
    }
    /** copies the MouseEvent (or convert Touch) properties and sends it as another event to the given target */
    static simulateMouseEvent(e4, simulatedType, target) {
      const me = e4;
      const simulatedEvent = new MouseEvent(simulatedType, {
        bubbles: true,
        composed: true,
        cancelable: true,
        view: window,
        detail: 1,
        screenX: e4.screenX,
        screenY: e4.screenY,
        clientX: e4.clientX,
        clientY: e4.clientY,
        ctrlKey: me.ctrlKey ?? false,
        altKey: me.altKey ?? false,
        shiftKey: me.shiftKey ?? false,
        metaKey: me.metaKey ?? false,
        button: 0,
        relatedTarget: e4.target
      });
      (target || e4.target).dispatchEvent(simulatedEvent);
    }
    /**
     * defines an element that is used to get the offset and scale from grid transforms
     * returns the scale and offsets from said element
    */
    static getValuesFromTransformedElement(parent) {
      const transformReference = document.createElement("div");
      _Utils.addElStyles(transformReference, {
        opacity: "0",
        position: "fixed",
        top: "0px",
        left: "0px",
        width: "1px",
        height: "1px",
        zIndex: "-999999"
      });
      parent.appendChild(transformReference);
      const transformValues = transformReference.getBoundingClientRect();
      parent.removeChild(transformReference);
      transformReference.remove();
      return {
        xScale: 1 / transformValues.width,
        yScale: 1 / transformValues.height,
        xOffset: transformValues.left,
        yOffset: transformValues.top
      };
    }
    /** swap the given object 2 field values */
    static swap(o4, a4, b4) {
      if (!o4)
        return;
      const tmp = o4[a4];
      o4[a4] = o4[b4];
      o4[b4] = tmp;
    }
    /** returns true if event is inside the given element rectangle */
    // Note: Safari Mac has null event.relatedTarget which causes #1684 so check if DragEvent is inside the coordinates instead
    //    Utils.el.contains(event.relatedTarget as HTMLElement)
    // public static inside(e: MouseEvent, el: HTMLElement): boolean {
    //   // srcElement, toElement, target: all set to placeholder when leaving simple grid, so we can't use that (Chrome)
    //   const target: HTMLElement = e.relatedTarget || (e as any).fromElement;
    //   if (!target) {
    //     const { bottom, left, right, top } = el.getBoundingClientRect();
    //     return (e.x < right && e.x > left && e.y < bottom && e.y > top);
    //   }
    //   return el.contains(target);
    // }
    /** true if the item can be rotated (checking for prop, not space available) */
    static canBeRotated(n3) {
      return !(!n3 || n3.w === n3.h || n3.locked || n3.noResize || n3.grid?.opts.disableResize || n3.minW && n3.minW === n3.maxW || n3.minH && n3.minH === n3.maxH);
    }
  };

  // node_modules/gridstack/dist/gridstack-engine.js
  var GridStackEngine = class _GridStackEngine {
    constructor(opts = {}) {
      this.addedNodes = [];
      this.removedNodes = [];
      this.defaultColumn = 12;
      this.column = opts.column || this.defaultColumn;
      if (this.column > this.defaultColumn)
        this.defaultColumn = this.column;
      this.maxRow = opts.maxRow;
      this._float = opts.float;
      this.nodes = opts.nodes || [];
      this.onChange = opts.onChange;
    }
    /**
     * Enable/disable batch mode for multiple operations to optimize performance.
     * When enabled, layout updates are deferred until batch mode is disabled.
     *
     * @param flag true to enable batch mode, false to disable and apply changes
     * @param doPack if true (default), pack/compact nodes when disabling batch mode
     * @returns the engine instance for chaining
     *
     * @example
     * // Start batch mode for multiple operations
     * engine.batchUpdate(true);
     * engine.addNode(node1);
     * engine.addNode(node2);
     * engine.batchUpdate(false); // Apply all changes at once
     */
    batchUpdate(flag = true, doPack = true) {
      if (!!this.batchMode === flag)
        return this;
      this.batchMode = flag;
      if (flag) {
        this._prevFloat = this._float;
        this._float = true;
        this.cleanNodes();
        this.saveInitial();
      } else {
        this._float = this._prevFloat;
        delete this._prevFloat;
        if (doPack)
          this._packNodes();
        this._notify();
      }
      return this;
    }
    // use entire row for hitting area (will use bottom reverse sorted first) if we not actively moving DOWN and didn't already skip
    _useEntireRowArea(node, nn) {
      return (!this.float || this.batchMode && !this._prevFloat) && !this._hasLocked && (!node._moving || node._skipDown || nn.y <= node.y);
    }
    /** @internal fix collision on given 'node', going to given new location 'nn', with optional 'collide' node already found.
     * return true if we moved. */
    _fixCollisions(node, nn = node, collide, opt = {}) {
      this.sortNodes(-1);
      collide = collide || this.collide(node, nn);
      if (!collide)
        return false;
      if (node._moving && !opt.nested && !this.float) {
        if (this.swap(node, collide))
          return true;
      }
      let area = nn;
      if (!this._loading && this._useEntireRowArea(node, nn)) {
        area = { x: 0, w: this.column, y: nn.y, h: nn.h };
        collide = this.collide(node, area, opt.skip);
      }
      let didMove = false;
      const newOpt = { nested: true, pack: false };
      let counter = 0;
      while (collide = collide || this.collide(node, area, opt.skip)) {
        if (counter++ > this.nodes.length * 2) {
          throw new Error("Infinite collide check");
        }
        let moved;
        if (collide.locked || this._loading || node._moving && !node._skipDown && nn.y > node.y && !this.float && // can take space we had, or before where we're going
        (!this.collide(collide, { ...collide, y: node.y }, node) || !this.collide(collide, { ...collide, y: nn.y - collide.h }, node))) {
          node._skipDown = node._skipDown || nn.y > node.y;
          const newNN = { ...nn, y: collide.y + collide.h, ...newOpt };
          moved = this._loading && Utils.samePos(node, newNN) ? true : this.moveNode(node, newNN);
          if ((collide.locked || this._loading) && moved) {
            Utils.copyPos(nn, node);
          } else if (!collide.locked && moved && opt.pack) {
            this._packNodes();
            nn.y = collide.y + collide.h;
            Utils.copyPos(node, nn);
          }
          didMove = didMove || moved;
        } else {
          moved = this.moveNode(collide, { ...collide, y: nn.y + nn.h, skip: node, ...newOpt });
        }
        if (!moved)
          return didMove;
        collide = void 0;
      }
      return didMove;
    }
    /**
     * Return the first node that intercepts/collides with the given node or area.
     * Used for collision detection during drag and drop operations.
     *
     * @param skip the node to skip in collision detection (usually the node being moved)
     * @param area the area to check for collisions (defaults to skip node's area)
     * @param skip2 optional second node to skip in collision detection
     * @returns the first colliding node, or undefined if no collision
     *
     * @example
     * const colliding = engine.collide(draggedNode, {x: 2, y: 1, w: 2, h: 1});
     * if (colliding) {
     *   console.log('Would collide with:', colliding.id);
     * }
     */
    collide(skip, area = skip, skip2) {
      const skipId = skip._id;
      const skip2Id = skip2?._id;
      return this.nodes.find((n3) => n3._id !== skipId && n3._id !== skip2Id && Utils.isIntercepted(n3, area));
    }
    /**
     * Return all nodes that intercept/collide with the given node or area.
     * Similar to collide() but returns all colliding nodes instead of just the first.
     *
     * @param skip the node to skip in collision detection
     * @param area the area to check for collisions (defaults to skip node's area)
     * @param skip2 optional second node to skip in collision detection
     * @returns array of all colliding nodes
     *
     * @example
     * const allCollisions = engine.collideAll(draggedNode);
     * console.log('Colliding with', allCollisions.length, 'nodes');
     */
    collideAll(skip, area = skip, skip2) {
      const skipId = skip._id;
      const skip2Id = skip2?._id;
      return this.nodes.filter((n3) => n3._id !== skipId && n3._id !== skip2Id && Utils.isIntercepted(n3, area));
    }
    /** does a pixel coverage collision based on where we started, returning the node that has the most coverage that is >50% mid line */
    directionCollideCoverage(node, o4, collides) {
      if (!o4.rect || !node._rect)
        return;
      const r0 = node._rect;
      const r4 = { ...o4.rect };
      if (r4.y > r0.y) {
        r4.h += r4.y - r0.y;
        r4.y = r0.y;
      } else {
        r4.h += r0.y - r4.y;
      }
      if (r4.x > r0.x) {
        r4.w += r4.x - r0.x;
        r4.x = r0.x;
      } else {
        r4.w += r0.x - r4.x;
      }
      let collide;
      let overMax = 0.5;
      for (let n3 of collides) {
        if (n3.locked || !n3._rect) {
          break;
        }
        const r22 = n3._rect;
        let yOver = Number.MAX_VALUE, xOver = Number.MAX_VALUE;
        if (r0.y < r22.y) {
          yOver = (r4.y + r4.h - r22.y) / r22.h;
        } else if (r0.y + r0.h > r22.y + r22.h) {
          yOver = (r22.y + r22.h - r4.y) / r22.h;
        }
        if (r0.x < r22.x) {
          xOver = (r4.x + r4.w - r22.x) / r22.w;
        } else if (r0.x + r0.w > r22.x + r22.w) {
          xOver = (r22.x + r22.w - r4.x) / r22.w;
        }
        const over = Math.min(xOver, yOver);
        if (over > overMax) {
          overMax = over;
          collide = n3;
        }
      }
      o4.collide = collide;
      return collide;
    }
    /** does a pixel coverage returning the node that has the most coverage by area */
    /*
    protected collideCoverage(r: GridStackPosition, collides: GridStackNode[]): {collide: GridStackNode, over: number} {
      const collide: GridStackNode;
      const overMax = 0;
      collides.forEach(n => {
        if (n.locked || !n._rect) return;
        const over = Utils.areaIntercept(r, n._rect);
        if (over > overMax) {
          overMax = over;
          collide = n;
        }
      });
      return {collide, over: overMax};
    }
    */
    /**
     * Cache the pixel rectangles for all nodes used for collision detection during drag operations.
     * This optimization converts grid coordinates to pixel coordinates for faster collision detection.
     *
     * @param w width of a single grid cell in pixels
     * @param h height of a single grid cell in pixels
     * @param top top margin/padding in pixels
     * @param right right margin/padding in pixels
     * @param bottom bottom margin/padding in pixels
     * @param left left margin/padding in pixels
     * @returns the engine instance for chaining
     *
     * @internal This is typically called by GridStack during resize events
     */
    cacheRects(w5, h5, top, right, bottom, left) {
      this.nodes.forEach((n3) => n3._rect = {
        y: n3.y * h5 + top,
        x: n3.x * w5 + left,
        w: n3.w * w5 - left - right,
        h: n3.h * h5 - top - bottom
      });
      return this;
    }
    /**
     * Attempt to swap the positions of two nodes if they meet swapping criteria.
     * Nodes can swap if they are the same size or in the same column/row, not locked, and touching.
     *
     * @param a first node to swap
     * @param b second node to swap
     * @returns true if swap was successful, false if not possible, undefined if not applicable
     *
     * @example
     * const swapped = engine.swap(nodeA, nodeB);
     * if (swapped) {
     *   console.log('Nodes swapped successfully');
     * }
     */
    swap(a4, b4) {
      if (!b4 || b4.locked || !a4 || a4.locked)
        return false;
      function _doSwap() {
        const x4 = b4.x, y5 = b4.y;
        b4.x = a4.x;
        b4.y = a4.y;
        if (a4.h != b4.h) {
          a4.x = x4;
          a4.y = b4.y + b4.h;
        } else if (a4.w != b4.w) {
          a4.x = b4.x + b4.w;
          a4.y = y5;
        } else {
          a4.x = x4;
          a4.y = y5;
        }
        a4._dirty = b4._dirty = true;
        return true;
      }
      let touching;
      if (a4.w === b4.w && a4.h === b4.h && (a4.x === b4.x || a4.y === b4.y) && (touching = Utils.isTouching(a4, b4)))
        return _doSwap();
      if (touching === false)
        return;
      if (a4.w === b4.w && a4.x === b4.x && (touching || (touching = Utils.isTouching(a4, b4)))) {
        if (b4.y < a4.y) {
          const t4 = a4;
          a4 = b4;
          b4 = t4;
        }
        return _doSwap();
      }
      if (touching === false)
        return;
      if (a4.h === b4.h && a4.y === b4.y && (touching || (touching = Utils.isTouching(a4, b4)))) {
        if (b4.x < a4.x) {
          const t4 = a4;
          a4 = b4;
          b4 = t4;
        }
        return _doSwap();
      }
      return false;
    }
    /**
     * Check if the specified rectangular area is empty (no nodes occupy any part of it).
     *
     * @param x the x coordinate (column) of the area to check
     * @param y the y coordinate (row) of the area to check
     * @param w the width in columns of the area to check
     * @param h the height in rows of the area to check
     * @returns true if the area is completely empty, false if any node overlaps
     *
     * @example
     * if (engine.isAreaEmpty(2, 1, 3, 2)) {
     *   console.log('Area is available for placement');
     * }
     */
    isAreaEmpty(x4, y5, w5, h5) {
      const nn = { x: x4 || 0, y: y5 || 0, w: w5 || 1, h: h5 || 1 };
      return !this.collide(nn);
    }
    /**
     * Re-layout grid items to reclaim any empty space.
     * This optimizes the grid layout by moving items to fill gaps.
     *
     * @param layout layout algorithm to use:
     *   - 'compact' (default): find truly empty spaces, may reorder items
     *   - 'list': keep the sort order exactly the same, move items up sequentially
     * @param doSort if true (default), sort nodes by position before compacting
     * @returns the engine instance for chaining
     *
     * @example
     * // Compact to fill empty spaces
     * engine.compact();
     *
     * // Compact preserving item order
     * engine.compact('list');
     */
    compact(layout = "compact", doSort = true) {
      if (this.nodes.length === 0)
        return this;
      if (doSort)
        this.sortNodes();
      const wasBatch = this.batchMode;
      if (!wasBatch)
        this.batchUpdate();
      const wasColumnResize = this._inColumnResize;
      if (!wasColumnResize)
        this._inColumnResize = true;
      const copyNodes = this.nodes;
      this.nodes = [];
      copyNodes.forEach((n3, index, list) => {
        let after;
        if (!n3.locked) {
          n3.autoPosition = true;
          if (layout === "list" && index)
            after = list[index - 1];
        }
        this.addNode(n3, false, after);
      });
      if (!wasColumnResize)
        delete this._inColumnResize;
      if (!wasBatch)
        this.batchUpdate(false);
      return this;
    }
    /**
     * Enable/disable floating widgets (default: `false`).
     * When floating is enabled, widgets can move up to fill empty spaces.
     * See [example](http://gridstackjs.com/demo/float.html)
     *
     * @param val true to enable floating, false to disable
     *
     * @example
     * engine.float = true;  // Enable floating
     * engine.float = false; // Disable floating (default)
     */
    set float(val) {
      if (this._float === val)
        return;
      this._float = val || false;
      if (!val) {
        this._packNodes()._notify();
      }
    }
    /**
     * Get the current floating mode setting.
     *
     * @returns true if floating is enabled, false otherwise
     *
     * @example
     * const isFloating = engine.float;
     * console.log('Floating enabled:', isFloating);
     */
    get float() {
      return this._float || false;
    }
    /**
     * Sort the nodes array from first to last, or reverse.
     * This is called during collision/placement operations to enforce a specific order.
     *
     * @param dir sort direction: 1 for ascending (first to last), -1 for descending (last to first)
     * @returns the engine instance for chaining
     *
     * @example
     * engine.sortNodes();    // Sort ascending (default)
     * engine.sortNodes(-1);  // Sort descending
     */
    sortNodes(dir = 1) {
      this.nodes = Utils.sort(this.nodes, dir);
      return this;
    }
    /** @internal called to top gravity pack the items back OR revert back to original Y positions when floating */
    _packNodes() {
      if (this.batchMode) {
        return this;
      }
      this.sortNodes();
      if (this.float) {
        this.nodes.forEach((n3) => {
          if (n3._updating || n3._orig === void 0 || n3.y === n3._orig.y)
            return;
          let newY = n3.y;
          while (newY > n3._orig.y) {
            --newY;
            const collide = this.collide(n3, { x: n3.x, y: newY, w: n3.w, h: n3.h });
            if (!collide) {
              n3._dirty = true;
              n3.y = newY;
            }
          }
        });
      } else {
        this.nodes.forEach((n3, i4) => {
          if (n3.locked)
            return;
          while (n3.y > 0) {
            const newY = i4 === 0 ? 0 : n3.y - 1;
            const canBeMoved = i4 === 0 || !this.collide(n3, { x: n3.x, y: newY, w: n3.w, h: n3.h });
            if (!canBeMoved)
              break;
            n3._dirty = n3.y !== newY;
            n3.y = newY;
          }
        });
      }
      return this;
    }
    /**
     * Prepare and validate a node's coordinates and values for the current grid.
     * This ensures the node has valid position, size, and properties before being added to the grid.
     *
     * @param node the node to prepare and validate
     * @param resizing if true, resize the node down if it's out of bounds; if false, move it to fit
     * @returns the prepared node with valid coordinates
     *
     * @example
     * const node = { w: 3, h: 2, content: 'Hello' };
     * const prepared = engine.prepareNode(node);
     * console.log('Node prepared at:', prepared.x, prepared.y);
     */
    prepareNode(node, resizing) {
      node._id = node._id ?? _GridStackEngine._idSeq++;
      const id = node.id;
      if (id) {
        let count2 = 1;
        while (this.nodes.find((n3) => n3.id === node.id && n3 !== node)) {
          node.id = id + "_" + count2++;
        }
      }
      if (node.x === void 0 || node.y === void 0 || node.x === null || node.y === null) {
        node.autoPosition = true;
      }
      const defaults = { x: 0, y: 0, w: 1, h: 1 };
      Utils.defaults(node, defaults);
      if (!node.autoPosition) {
        delete node.autoPosition;
      }
      if (!node.noResize) {
        delete node.noResize;
      }
      if (!node.noMove) {
        delete node.noMove;
      }
      Utils.sanitizeMinMax(node);
      if (typeof node.x == "string") {
        node.x = Number(node.x);
      }
      if (typeof node.y == "string") {
        node.y = Number(node.y);
      }
      if (typeof node.w == "string") {
        node.w = Number(node.w);
      }
      if (typeof node.h == "string") {
        node.h = Number(node.h);
      }
      if (isNaN(node.x)) {
        node.x = defaults.x;
        node.autoPosition = true;
      }
      if (isNaN(node.y)) {
        node.y = defaults.y;
        node.autoPosition = true;
      }
      if (isNaN(node.w)) {
        node.w = defaults.w;
      }
      if (isNaN(node.h)) {
        node.h = defaults.h;
      }
      this.nodeBoundFix(node, resizing);
      return node;
    }
    /**
     * Part 2 of preparing a node to fit inside the grid - validates and fixes coordinates and dimensions.
     * This ensures the node fits within grid boundaries and respects min/max constraints.
     *
     * @param node the node to validate and fix
     * @param resizing if true, resize the node to fit; if false, move the node to fit
     * @returns the engine instance for chaining
     *
     * @example
     * // Fix a node that might be out of bounds
     * engine.nodeBoundFix(node, true); // Resize to fit
     * engine.nodeBoundFix(node, false); // Move to fit
     */
    nodeBoundFix(node, resizing) {
      const before = node._orig || Utils.copyPos({}, node);
      if (node.maxW) {
        node.w = Math.min(node.w || 1, node.maxW);
      }
      if (node.maxH) {
        node.h = Math.min(node.h || 1, node.maxH);
      }
      if (node.minW) {
        node.w = Math.max(node.w || 1, node.minW);
      }
      if (node.minH) {
        node.h = Math.max(node.h || 1, node.minH);
      }
      const saveOrig = (node.x || 0) + (node.w || 1) > this.column;
      if (saveOrig && this.column < this.defaultColumn && !this._inColumnResize && !this.skipCacheUpdate && node._id != null && this.findCacheLayout(node, this.defaultColumn) === -1) {
        const copy = { ...node };
        if (copy.autoPosition || copy.x === void 0) {
          delete copy.x;
          delete copy.y;
        } else
          copy.x = Math.min(this.defaultColumn - 1, copy.x);
        copy.w = Math.min(this.defaultColumn, copy.w || 1);
        this.cacheOneLayout(copy, this.defaultColumn);
      }
      if (node.w > this.column) {
        node.w = this.column;
      } else if (node.w < 1) {
        node.w = 1;
      }
      if (this.maxRow && node.h > this.maxRow) {
        node.h = this.maxRow;
      } else if (node.h < 1) {
        node.h = 1;
      }
      if (node.x < 0) {
        node.x = 0;
      }
      if (node.y < 0) {
        node.y = 0;
      }
      if (node.x + node.w > this.column) {
        if (resizing) {
          node.w = this.column - node.x;
        } else {
          node.x = this.column - node.w;
        }
      }
      if (this.maxRow && node.y + node.h > this.maxRow) {
        if (resizing) {
          node.h = this.maxRow - node.y;
        } else {
          node.y = this.maxRow - node.h;
        }
      }
      if (!Utils.samePos(node, before)) {
        node._dirty = true;
      }
      return this;
    }
    /**
     * Returns a list of nodes that have been modified from their original values.
     * This is used to track which nodes need DOM updates.
     *
     * @param verify if true, performs additional verification by comparing current vs original positions
     * @returns array of nodes that have been modified
     *
     * @example
     * const changed = engine.getDirtyNodes();
     * console.log('Modified nodes:', changed.length);
     *
     * // Get verified dirty nodes
     * const verified = engine.getDirtyNodes(true);
     */
    getDirtyNodes(verify) {
      if (verify) {
        return this.nodes.filter((n3) => n3._dirty && !Utils.samePos(n3, n3._orig));
      }
      return this.nodes.filter((n3) => n3._dirty);
    }
    /** @internal call this to call onChange callback with dirty nodes so DOM can be updated */
    _notify(removedNodes) {
      if (this.batchMode || !this.onChange)
        return this;
      const dirtyNodes = (removedNodes || []).concat(this.getDirtyNodes());
      this.onChange(dirtyNodes);
      return this;
    }
    /**
     * Clean all dirty and last tried information from nodes.
     * This resets the dirty state tracking for all nodes.
     *
     * @returns the engine instance for chaining
     *
     * @internal
     */
    cleanNodes() {
      if (this.batchMode)
        return this;
      this.nodes.forEach((n3) => {
        delete n3._dirty;
        delete n3._lastTried;
      });
      return this;
    }
    /**
     * Save the initial position/size of all nodes to track real dirty state.
     * This creates a snapshot of current positions that can be restored later.
     *
     * Note: Should be called right after change events and before move/resize operations.
     *
     * @returns the engine instance for chaining
     *
     * @internal
     */
    saveInitial() {
      this.nodes.forEach((n3) => {
        n3._orig = Utils.copyPos({}, n3);
        delete n3._dirty;
      });
      this._hasLocked = this.nodes.some((n3) => n3.locked);
      return this;
    }
    /**
     * Restore all nodes back to their initial values.
     * This is typically called when canceling an operation (e.g., Esc key during drag).
     *
     * @returns the engine instance for chaining
     *
     * @internal
     */
    restoreInitial() {
      this.nodes.forEach((n3) => {
        if (!n3._orig || Utils.samePos(n3, n3._orig))
          return;
        Utils.copyPos(n3, n3._orig);
        n3._dirty = true;
      });
      this._notify();
      return this;
    }
    /**
     * Find the first available empty spot for the given node dimensions.
     * Updates the node's x,y attributes with the found position.
     *
     * @param node the node to find a position for (w,h must be set)
     * @param nodeList optional list of nodes to check against (defaults to engine nodes)
     * @param column optional column count (defaults to engine column count)
     * @param after optional node to start search after (maintains order)
     * @returns true if an empty position was found and node was updated
     *
     * @example
     * const node = { w: 2, h: 1 };
     * if (engine.findEmptyPosition(node)) {
     *   console.log('Found position at:', node.x, node.y);
     * }
     */
    findEmptyPosition(node, nodeList = this.nodes, column = this.column, after) {
      const start = after ? after.y * column + (after.x + after.w) : 0;
      let found = false;
      for (let i4 = start; !found; ++i4) {
        const x4 = i4 % column;
        const y5 = Math.floor(i4 / column);
        if (x4 + node.w > column) {
          continue;
        }
        const box = { x: x4, y: y5, w: node.w, h: node.h };
        if (!nodeList.find((n3) => Utils.isIntercepted(box, n3))) {
          if (node.x !== x4 || node.y !== y5)
            node._dirty = true;
          node.x = x4;
          node.y = y5;
          delete node.autoPosition;
          found = true;
        }
      }
      return found;
    }
    /**
     * Add the given node to the grid, handling collision detection and re-packing.
     * This is the main method for adding new widgets to the engine.
     *
     * @param node the node to add to the grid
     * @param triggerAddEvent if true, adds node to addedNodes list for event triggering
     * @param after optional node to place this node after (for ordering)
     * @returns the added node (or existing node if duplicate)
     *
     * @example
     * const node = { x: 0, y: 0, w: 2, h: 1, content: 'Hello' };
     * const added = engine.addNode(node, true);
     */
    addNode(node, triggerAddEvent = false, after) {
      const dup = this.nodes.find((n3) => n3._id === node._id);
      if (dup)
        return dup;
      this._inColumnResize ? this.nodeBoundFix(node) : this.prepareNode(node);
      delete node._temporaryRemoved;
      delete node._removeDOM;
      let skipCollision;
      if (node.autoPosition && this.findEmptyPosition(node, this.nodes, this.column, after)) {
        delete node.autoPosition;
        skipCollision = true;
      }
      this.nodes.push(node);
      if (triggerAddEvent) {
        this.addedNodes.push(node);
      }
      if (!skipCollision)
        this._fixCollisions(node);
      if (!this.batchMode) {
        this._packNodes()._notify();
      }
      return node;
    }
    /**
     * Remove the given node from the grid.
     *
     * @param node the node to remove
     * @param removeDOM if true (default), marks node for DOM removal
     * @param triggerEvent if true, adds node to removedNodes list for event triggering
     * @returns the engine instance for chaining
     *
     * @example
     * engine.removeNode(node, true, true);
     */
    removeNode(node, removeDOM = true, triggerEvent = false) {
      if (!this.nodes.find((n3) => n3._id === node._id)) {
        return this;
      }
      if (triggerEvent) {
        this.removedNodes.push(node);
      }
      if (removeDOM)
        node._removeDOM = true;
      this.nodes = this.nodes.filter((n3) => n3._id !== node._id);
      if (!node._isAboutToRemove)
        this._packNodes();
      this._notify([node]);
      return this;
    }
    /**
     * Remove all nodes from the grid.
     *
     * @param removeDOM if true (default), marks all nodes for DOM removal
     * @param triggerEvent if true (default), triggers removal events
     * @returns the engine instance for chaining
     *
     * @example
     * engine.removeAll(); // Remove all nodes
     */
    removeAll(removeDOM = true, triggerEvent = true) {
      delete this._layouts;
      if (!this.nodes.length)
        return this;
      removeDOM && this.nodes.forEach((n3) => n3._removeDOM = true);
      const removedNodes = this.nodes;
      this.removedNodes = triggerEvent ? removedNodes : [];
      this.nodes = [];
      return this._notify(removedNodes);
    }
    /**
     * Check if a node can be moved to a new position, considering layout constraints.
     * This is a safer version of moveNode() that validates the move first.
     *
     * For complex cases (like maxRow constraints), it simulates the move in a clone first,
     * then applies the changes only if they meet all specifications.
     *
     * @param node the node to move
     * @param o move options including target position
     * @returns true if the node was successfully moved
     *
     * @example
     * const canMove = engine.moveNodeCheck(node, { x: 2, y: 1 });
     * if (canMove) {
     *   console.log('Node moved successfully');
     * }
     */
    moveNodeCheck(node, o4) {
      if (!this.changedPosConstrain(node, o4))
        return false;
      o4.pack = true;
      if (!this.maxRow) {
        return this.moveNode(node, o4);
      }
      let clonedNode;
      const clone = new _GridStackEngine({
        column: this.column,
        float: this.float,
        nodes: this.nodes.map((n3) => {
          if (n3._id === node._id) {
            clonedNode = { ...n3 };
            return clonedNode;
          }
          return { ...n3 };
        })
      });
      if (!clonedNode)
        return false;
      const canMove = clone.moveNode(clonedNode, o4) && clone.getRow() <= Math.max(this.getRow(), this.maxRow);
      if (!canMove && !o4.resizing && o4.collide) {
        const collide = o4.collide.el.gridstackNode;
        if (this.swap(node, collide)) {
          this._notify();
          return true;
        }
      }
      if (!canMove)
        return false;
      clone.nodes.filter((n3) => n3._dirty).forEach((c4) => {
        const n3 = this.nodes.find((a4) => a4._id === c4._id);
        if (!n3)
          return;
        Utils.copyPos(n3, c4);
        n3._dirty = true;
      });
      this._notify();
      return true;
    }
    /** return true if can fit in grid height constrain only (always true if no maxRow) */
    willItFit(node) {
      delete node._willFitPos;
      if (!this.maxRow)
        return true;
      const clone = new _GridStackEngine({
        column: this.column,
        float: this.float,
        nodes: this.nodes.map((n4) => {
          return { ...n4 };
        })
      });
      const n3 = { ...node };
      this.cleanupNode(n3);
      delete n3.el;
      delete n3._id;
      delete n3.content;
      delete n3.grid;
      clone.addNode(n3);
      if (clone.getRow() <= this.maxRow) {
        node._willFitPos = Utils.copyPos({}, n3);
        return true;
      }
      return false;
    }
    /** true if x,y or w,h are different after clamping to min/max */
    changedPosConstrain(node, p5) {
      p5.w = p5.w || node.w;
      p5.h = p5.h || node.h;
      if (node.x !== p5.x || node.y !== p5.y)
        return true;
      if (node.maxW) {
        p5.w = Math.min(p5.w, node.maxW);
      }
      if (node.maxH) {
        p5.h = Math.min(p5.h, node.maxH);
      }
      if (node.minW) {
        p5.w = Math.max(p5.w, node.minW);
      }
      if (node.minH) {
        p5.h = Math.max(p5.h, node.minH);
      }
      return node.w !== p5.w || node.h !== p5.h;
    }
    /** return true if the passed in node was actually moved (checks for no-op and locked) */
    moveNode(node, o4) {
      if (!node || /*node.locked ||*/
      !o4)
        return false;
      let wasUndefinedPack;
      if (o4.pack === void 0 && !this.batchMode) {
        wasUndefinedPack = o4.pack = true;
      }
      if (typeof o4.x !== "number") {
        o4.x = node.x;
      }
      if (typeof o4.y !== "number") {
        o4.y = node.y;
      }
      if (typeof o4.w !== "number") {
        o4.w = node.w;
      }
      if (typeof o4.h !== "number") {
        o4.h = node.h;
      }
      const resizing = node.w !== o4.w || node.h !== o4.h;
      const nn = Utils.copyPos({}, node, true);
      Utils.copyPos(nn, o4);
      this.nodeBoundFix(nn, resizing);
      Utils.copyPos(o4, nn);
      if (!o4.forceCollide && Utils.samePos(node, o4))
        return false;
      const prevPos = Utils.copyPos({}, node);
      const collides = this.collideAll(node, nn, o4.skip);
      let needToMove = true;
      if (collides.length) {
        const activeDrag = node._moving && !o4.nested;
        let collide = activeDrag ? this.directionCollideCoverage(node, o4, collides) : collides[0];
        if (activeDrag && collide && node.grid?.opts?.subGridDynamic && !node.grid._isTemp) {
          const over = Utils.areaIntercept(o4.rect, collide._rect);
          const a1 = Utils.area(o4.rect);
          const a22 = Utils.area(collide._rect);
          const perc = over / (a1 < a22 ? a1 : a22);
          if (perc > 0.8) {
            collide.grid.makeSubGrid(collide.el, void 0, node);
            collide = void 0;
          }
        }
        if (collide) {
          needToMove = !this._fixCollisions(node, nn, collide, o4);
        } else {
          needToMove = false;
          if (wasUndefinedPack)
            delete o4.pack;
        }
      }
      if (needToMove && !Utils.samePos(node, nn)) {
        node._dirty = true;
        Utils.copyPos(node, nn);
      }
      if (o4.pack) {
        this._packNodes()._notify();
      }
      return !Utils.samePos(node, prevPos);
    }
    getRow() {
      return this.nodes.reduce((row, n3) => Math.max(row, n3.y + n3.h), 0);
    }
    beginUpdate(node) {
      if (!node._updating) {
        node._updating = true;
        delete node._skipDown;
        if (!this.batchMode)
          this.saveInitial();
      }
      return this;
    }
    endUpdate() {
      const n3 = this.nodes.find((n4) => n4._updating);
      if (n3) {
        delete n3._updating;
        delete n3._skipDown;
      }
      return this;
    }
    /** saves a copy of the largest column layout (eg 12 even when rendering 1 column) so we don't loose orig layout, unless explicity column
     * count to use is given. returning a list of widgets for serialization
     * @param saveElement if true (default), the element will be saved to GridStackWidget.el field, else it will be removed.
     * @param saveCB callback for each node -> widget, so application can insert additional data to be saved into the widget data structure.
     * @param column if provided, the grid will be saved for the given column count (IFF we have matching internal saved layout, or current layout).
     * Note: nested grids will ALWAYS save the container w to match overall layouts (parent + child) to be consistent.
    */
    save(saveElement = true, saveCB, column) {
      const len = this._layouts?.length || 0;
      let layout;
      if (len) {
        if (column) {
          if (column !== this.column)
            layout = this._layouts[column];
        } else if (this.column !== len - 1) {
          layout = this._layouts[len - 1];
        }
      }
      const list = [];
      this.sortNodes();
      this.nodes.forEach((n3) => {
        const wl = layout?.find((l5) => l5._id === n3._id);
        const w5 = { ...n3, ...wl || {} };
        Utils.removeInternalForSave(w5, !saveElement);
        if (saveCB)
          saveCB(n3, w5);
        list.push(w5);
      });
      return list;
    }
    /** @internal called whenever a node is added or moved - updates the cached layouts */
    layoutsNodesChange(nodes) {
      if (!this._layouts || this._inColumnResize)
        return this;
      this._layouts.forEach((layout, column) => {
        if (!layout || column === this.column)
          return this;
        if (column < this.column) {
          this._layouts[column] = void 0;
        } else {
          const ratio = column / this.column;
          nodes.forEach((node) => {
            if (!node._orig)
              return;
            const n3 = layout.find((l5) => l5._id === node._id);
            if (!n3)
              return;
            if (n3.y >= 0 && node.y !== node._orig.y) {
              n3.y += node.y - node._orig.y;
              if (n3.y < 0)
                n3.y = 0;
            }
            if (node.x !== node._orig.x) {
              n3.x = Math.round(node.x * ratio);
              if (n3.x < 0)
                n3.x = 0;
            }
            if (node.w !== node._orig.w) {
              n3.w = Math.round(node.w * ratio);
              if (n3.w < 1)
                n3.w = 1;
            }
          });
        }
      });
      return this;
    }
    /**
     * @internal Called to scale the widget width & position up/down based on the column change.
     * Note we store previous layouts (especially original ones) to make it possible to go
     * from say 12 -> 1 -> 12 and get back to where we were.
     *
     * @param prevColumn previous number of columns
     * @param column  new column number
     * @param layout specify the type of re-layout that will happen (position, size, etc...).
     * Note: items will never be outside of the current column boundaries. default (moveScale). Ignored for 1 column
     */
    columnChanged(prevColumn, column, layout = "moveScale") {
      if (!this.nodes.length || !column || prevColumn === column)
        return this;
      const doCompact = layout === "compact" || layout === "list";
      if (doCompact) {
        this.sortNodes(1);
      }
      if (column < prevColumn)
        this.cacheLayout(this.nodes, prevColumn);
      this.batchUpdate();
      let newNodes = [];
      let nodes = doCompact ? this.nodes : Utils.sort(this.nodes, -1);
      if (column > prevColumn && this._layouts) {
        const cacheNodes = this._layouts[column] || [];
        const lastIndex = this._layouts.length - 1;
        if (!cacheNodes.length && prevColumn !== lastIndex && this._layouts[lastIndex]?.length) {
          prevColumn = lastIndex;
          this._layouts[lastIndex].forEach((cacheNode) => {
            const n3 = nodes.find((n4) => n4._id === cacheNode._id);
            if (n3) {
              if (!doCompact && !cacheNode.autoPosition) {
                n3.x = cacheNode.x ?? n3.x;
                n3.y = cacheNode.y ?? n3.y;
              }
              n3.w = cacheNode.w ?? n3.w;
              if (cacheNode.x == void 0 || cacheNode.y === void 0)
                n3.autoPosition = true;
            }
          });
        }
        cacheNodes.forEach((cacheNode) => {
          const j4 = nodes.findIndex((n3) => n3._id === cacheNode._id);
          if (j4 !== -1) {
            const n3 = nodes[j4];
            if (doCompact) {
              n3.w = cacheNode.w;
              return;
            }
            if (cacheNode.autoPosition || isNaN(cacheNode.x) || isNaN(cacheNode.y)) {
              this.findEmptyPosition(cacheNode, newNodes);
            }
            if (!cacheNode.autoPosition) {
              n3.x = cacheNode.x ?? n3.x;
              n3.y = cacheNode.y ?? n3.y;
              n3.w = cacheNode.w ?? n3.w;
              newNodes.push(n3);
            }
            nodes.splice(j4, 1);
          }
        });
      }
      if (doCompact) {
        this.compact(layout, false);
      } else {
        if (nodes.length) {
          if (typeof layout === "function") {
            layout(column, prevColumn, newNodes, nodes);
          } else {
            const ratio = doCompact || layout === "none" ? 1 : column / prevColumn;
            const move = layout === "move" || layout === "moveScale";
            const scale = layout === "scale" || layout === "moveScale";
            nodes.forEach((node) => {
              node.x = column === 1 ? 0 : move ? Math.round(node.x * ratio) : Math.min(node.x, column - 1);
              node.w = column === 1 || prevColumn === 1 ? 1 : scale ? Math.round(node.w * ratio) || 1 : Math.min(node.w, column);
              newNodes.push(node);
            });
            nodes = [];
          }
        }
        newNodes = Utils.sort(newNodes, -1);
        this._inColumnResize = true;
        this.nodes = [];
        newNodes.forEach((node) => {
          this.addNode(node, false);
          delete node._orig;
        });
      }
      this.nodes.forEach((n3) => delete n3._orig);
      this.batchUpdate(false, !doCompact);
      delete this._inColumnResize;
      return this;
    }
    /**
     * call to cache the given layout internally to the given location so we can restore back when column changes size
     * @param nodes list of nodes
     * @param column corresponding column index to save it under
     * @param clear if true, will force other caches to be removed (default false)
     */
    cacheLayout(nodes, column, clear = false) {
      const copy = [];
      nodes.forEach((n3, i4) => {
        if (n3._id === void 0) {
          const existing = n3.id ? this.nodes.find((n22) => n22.id === n3.id) : void 0;
          n3._id = existing?._id ?? _GridStackEngine._idSeq++;
        }
        copy[i4] = { x: n3.x, y: n3.y, w: n3.w, _id: n3._id };
      });
      this._layouts = clear ? [] : this._layouts || [];
      this._layouts[column] = copy;
      return this;
    }
    /**
     * call to cache the given node layout internally to the given location so we can restore back when column changes size
     * @param node single node to cache
     * @param column corresponding column index to save it under
     */
    cacheOneLayout(n3, column) {
      n3._id = n3._id ?? _GridStackEngine._idSeq++;
      const l5 = { x: n3.x, y: n3.y, w: n3.w, _id: n3._id };
      if (n3.autoPosition || n3.x === void 0) {
        delete l5.x;
        delete l5.y;
        if (n3.autoPosition)
          l5.autoPosition = true;
      }
      this._layouts = this._layouts || [];
      this._layouts[column] = this._layouts[column] || [];
      const index = this.findCacheLayout(n3, column);
      if (index === -1)
        this._layouts[column].push(l5);
      else
        this._layouts[column][index] = l5;
      return this;
    }
    findCacheLayout(n3, column) {
      return this._layouts?.[column]?.findIndex((l5) => l5._id === n3._id) ?? -1;
    }
    removeNodeFromLayoutCache(n3) {
      if (!this._layouts) {
        return;
      }
      for (let i4 = 0; i4 < this._layouts.length; i4++) {
        const index = this.findCacheLayout(n3, i4);
        if (index !== -1) {
          this._layouts[i4].splice(index, 1);
        }
      }
    }
    /** called to remove all internal values but the _id */
    cleanupNode(node) {
      for (const prop in node) {
        if (prop[0] === "_" && prop !== "_id")
          delete node[prop];
      }
      return this;
    }
  };
  GridStackEngine._idSeq = 0;

  // node_modules/gridstack/dist/types.js
  var gridDefaults = {
    alwaysShowResizeHandle: "mobile",
    animate: true,
    auto: true,
    cellHeight: "auto",
    cellHeightThrottle: 100,
    cellHeightUnit: "px",
    column: 12,
    draggable: { handle: ".grid-stack-item-content", appendTo: "body", scroll: true },
    handle: ".grid-stack-item-content",
    itemClass: "grid-stack-item",
    margin: 10,
    marginUnit: "px",
    maxRow: 0,
    minRow: 0,
    placeholderClass: "grid-stack-placeholder",
    placeholderText: "",
    removableOptions: { accept: "grid-stack-item", decline: "grid-stack-non-removable" },
    resizable: { handles: "se" },
    rtl: "auto"
    // **** same as not being set ****
    // disableDrag: false,
    // disableResize: false,
    // float: false,
    // handleClass: null,
    // removable: false,
    // staticGrid: false,
    //removable
  };

  // node_modules/gridstack/dist/dd-manager.js
  var DDManager = class {
  };

  // node_modules/gridstack/dist/dd-touch.js
  var isTouch = typeof window !== "undefined" && typeof document !== "undefined" && ("ontouchstart" in document || "ontouchstart" in window || window.DocumentTouch && document instanceof window.DocumentTouch || navigator.maxTouchPoints > 0 || navigator.msMaxTouchPoints > 0);
  var DDTouch = class {
  };
  function simulateMouseEvent(e4, simulatedType) {
    if (e4.touches.length > 1)
      return;
    if (e4.cancelable)
      e4.preventDefault();
    Utils.simulateMouseEvent(e4.changedTouches[0], simulatedType);
  }
  function simulatePointerMouseEvent(e4, simulatedType) {
    if (e4.cancelable)
      e4.preventDefault();
    Utils.simulateMouseEvent(e4, simulatedType);
  }
  function touchstart(e4) {
    if (DDTouch.touchHandled)
      return;
    DDTouch.touchHandled = true;
    simulateMouseEvent(e4, "mousedown");
  }
  function touchmove(e4) {
    if (!DDTouch.touchHandled)
      return;
    simulateMouseEvent(e4, "mousemove");
  }
  function touchend(e4) {
    if (!DDTouch.touchHandled)
      return;
    if (DDTouch.pointerLeaveTimeout) {
      window.clearTimeout(DDTouch.pointerLeaveTimeout);
      delete DDTouch.pointerLeaveTimeout;
    }
    const wasDragging = !!DDManager.dragElement;
    simulateMouseEvent(e4, "mouseup");
    if (!wasDragging) {
      simulateMouseEvent(e4, "click");
    }
    DDTouch.touchHandled = false;
  }
  function pointerdown(e4) {
    if (e4.pointerType === "mouse")
      return;
    e4.target.releasePointerCapture(e4.pointerId);
  }
  function pointerenter(e4) {
    if (!DDManager.dragElement) {
      return;
    }
    if (e4.pointerType === "mouse")
      return;
    simulatePointerMouseEvent(e4, "mouseenter");
  }
  function pointerleave(e4) {
    if (!DDManager.dragElement) {
      return;
    }
    if (e4.pointerType === "mouse")
      return;
    DDTouch.pointerLeaveTimeout = window.setTimeout(() => {
      delete DDTouch.pointerLeaveTimeout;
      simulatePointerMouseEvent(e4, "mouseleave");
    }, 10);
  }

  // node_modules/gridstack/dist/dd-resizable-handle.js
  var DDResizableHandle = class _DDResizableHandle {
    constructor(host, dir, option) {
      this.host = host;
      this.dir = dir;
      this.option = option;
      this.moving = false;
      this._mouseDown = this._mouseDown.bind(this);
      this._mouseMove = this._mouseMove.bind(this);
      this._mouseUp = this._mouseUp.bind(this);
      this._keyEvent = this._keyEvent.bind(this);
      this._init();
    }
    /** @internal */
    _init() {
      if (this.option.element) {
        try {
          this.el = this.option.element instanceof HTMLElement ? this.option.element : this.host.querySelector(this.option.element);
        } catch (error) {
          this.option.element = void 0;
          console.error("Query for resizeable handle failed, falling back", error);
        }
      }
      if (!this.el) {
        this.el = document.createElement("div");
        this.host.appendChild(this.el);
      }
      this.el.classList.add("ui-resizable-handle");
      this.el.classList.add(`${_DDResizableHandle.prefix}${this.dir}`);
      this.el.style.zIndex = "100";
      this.el.style.userSelect = "none";
      this.el.addEventListener("mousedown", this._mouseDown);
      if (isTouch) {
        this.el.addEventListener("touchstart", touchstart);
        this.el.addEventListener("pointerdown", pointerdown);
      }
      return this;
    }
    /** call this when resize handle needs to be removed and cleaned up */
    destroy() {
      if (this.moving)
        this._mouseUp(this.mouseDownEvent);
      this.el.removeEventListener("mousedown", this._mouseDown);
      if (isTouch) {
        this.el.removeEventListener("touchstart", touchstart);
        this.el.removeEventListener("pointerdown", pointerdown);
      }
      if (!this.option.element) {
        this.host.removeChild(this.el);
      }
      delete this.el;
      delete this.host;
      return this;
    }
    /** @internal called on mouse down on us: capture move on the entire document (mouse might not stay on us) until we release the mouse */
    _mouseDown(e4) {
      this.mouseDownEvent = e4;
      document.addEventListener("mousemove", this._mouseMove, { capture: true, passive: true });
      document.addEventListener("mouseup", this._mouseUp, true);
      if (isTouch) {
        this.el.addEventListener("touchmove", touchmove);
        this.el.addEventListener("touchend", touchend);
      }
      e4.stopPropagation();
      e4.preventDefault();
    }
    /** @internal */
    _mouseMove(e4) {
      const s4 = this.mouseDownEvent;
      if (this.moving) {
        this._triggerEvent("move", e4);
      } else if (Math.abs(e4.x - s4.x) + Math.abs(e4.y - s4.y) > 2) {
        this.moving = true;
        this._triggerEvent("start", this.mouseDownEvent);
        this._triggerEvent("move", e4);
        document.addEventListener("keydown", this._keyEvent);
      }
      e4.stopPropagation();
    }
    /** @internal */
    _mouseUp(e4) {
      if (this.moving) {
        this._triggerEvent("stop", e4);
        document.removeEventListener("keydown", this._keyEvent);
      }
      document.removeEventListener("mousemove", this._mouseMove, true);
      document.removeEventListener("mouseup", this._mouseUp, true);
      if (isTouch) {
        this.el.removeEventListener("touchmove", touchmove);
        this.el.removeEventListener("touchend", touchend);
      }
      delete this.moving;
      delete this.mouseDownEvent;
      e4.stopPropagation();
      e4.preventDefault();
    }
    /** @internal call when keys are being pressed - use Esc to cancel */
    _keyEvent(e4) {
      if (e4.key === "Escape") {
        this.host.gridstackNode?.grid?.engine.restoreInitial();
        this._mouseUp(this.mouseDownEvent);
      }
    }
    /** @internal */
    _triggerEvent(name, event) {
      if (this.option[name])
        this.option[name](event);
      return this;
    }
  };
  DDResizableHandle.prefix = "ui-resizable-";

  // node_modules/gridstack/dist/dd-base-impl.js
  var DDBaseImplement = class {
    constructor() {
      this._eventRegister = {};
    }
    /**
     * Returns the current disabled state.
     * Note: Use enable()/disable() methods to change state as other operations need to happen.
     */
    get disabled() {
      return this._disabled;
    }
    /**
     * Register an event callback for the specified event.
     *
     * @param event - Event name to listen for
     * @param callback - Function to call when event occurs
     */
    on(event, callback) {
      this._eventRegister[event] = callback;
    }
    /**
     * Unregister an event callback for the specified event.
     *
     * @param event - Event name to stop listening for
     */
    off(event) {
      delete this._eventRegister[event];
    }
    /**
     * Enable this drag & drop implementation.
     * Subclasses should override to perform additional setup.
     */
    enable() {
      this._disabled = false;
    }
    /**
     * Disable this drag & drop implementation.
     * Subclasses should override to perform additional cleanup.
     */
    disable() {
      this._disabled = true;
    }
    /**
     * Destroy this drag & drop implementation and clean up resources.
     * Removes all event handlers and clears internal state.
     */
    destroy() {
      delete this._eventRegister;
    }
    /**
     * Trigger a registered event callback if one exists and the implementation is enabled.
     *
     * @param eventName - Name of the event to trigger
     * @param event - DOM event object to pass to the callback
     * @returns Result from the callback function, if any
     */
    triggerEvent(eventName, event) {
      if (!this.disabled && this._eventRegister && this._eventRegister[eventName])
        return this._eventRegister[eventName](event);
    }
  };

  // node_modules/gridstack/dist/dd-resizable.js
  var DDResizable = class _DDResizable extends DDBaseImplement {
    // have to be public else complains for HTMLElementExtendOpt ?
    constructor(el, option = {}) {
      super();
      this.el = el;
      this.option = option;
      this.rectScale = { x: 1, y: 1 };
      this._ui = () => {
        const containmentEl = this.el.parentElement;
        const containmentRect = containmentEl.getBoundingClientRect();
        const newRect = {
          width: this.originalRect.width,
          height: this.originalRect.height + this.scrolled,
          left: this.originalRect.left,
          right: this.originalRect.right,
          top: this.originalRect.top - this.scrolled
        };
        const rect = this.temporalRect || newRect;
        const leftPos = this.option.rtl ? (containmentRect.right - rect.right) * this.rectScale.x : (rect.left - containmentRect.left) * this.rectScale.x;
        return {
          position: {
            left: leftPos,
            top: (rect.top - containmentRect.top) * this.rectScale.y
          },
          size: {
            width: rect.width * this.rectScale.x,
            height: rect.height * this.rectScale.y
          }
          /* Gridstack ONLY needs position set above... keep around in case.
          element: [this.el], // The object representing the element to be resized
          helper: [], // TODO: not support yet - The object representing the helper that's being resized
          originalElement: [this.el],// we don't wrap here, so simplify as this.el //The object representing the original element before it is wrapped
          originalPosition: { // The position represented as { left, top } before the resizable is resized
            left: this.originalRect.left - containmentRect.left,
            top: this.originalRect.top - containmentRect.top
          },
          originalSize: { // The size represented as { width, height } before the resizable is resized
            width: this.originalRect.width,
            height: this.originalRect.height
          }
          */
        };
      };
      this._mouseOver = this._mouseOver.bind(this);
      this._mouseOut = this._mouseOut.bind(this);
      this.enable();
      this._setupAutoHide(this.option.autoHide);
      this._setupHandlers();
    }
    on(event, callback) {
      super.on(event, callback);
    }
    off(event) {
      super.off(event);
    }
    enable() {
      super.enable();
      this.el.classList.remove("ui-resizable-disabled");
      this._setupAutoHide(this.option.autoHide);
    }
    disable() {
      super.disable();
      this.el.classList.add("ui-resizable-disabled");
      this._setupAutoHide(false);
    }
    destroy() {
      this._removeHandlers();
      this._setupAutoHide(false);
      delete this.el;
      super.destroy();
    }
    updateOption(opts) {
      const updateHandles = opts.handles && opts.handles !== this.option.handles;
      const updateAutoHide = opts.autoHide && opts.autoHide !== this.option.autoHide;
      Object.keys(opts).forEach((key) => this.option[key] = opts[key]);
      if (updateHandles) {
        this._removeHandlers();
        this._setupHandlers();
      }
      if (updateAutoHide) {
        this._setupAutoHide(this.option.autoHide);
      }
      return this;
    }
    /** @internal turns auto hide on/off */
    _setupAutoHide(auto) {
      if (auto) {
        this.el.classList.add("ui-resizable-autohide");
        this.el.addEventListener("mouseover", this._mouseOver);
        this.el.addEventListener("mouseout", this._mouseOut);
      } else {
        this.el.classList.remove("ui-resizable-autohide");
        this.el.removeEventListener("mouseover", this._mouseOver);
        this.el.removeEventListener("mouseout", this._mouseOut);
        if (DDManager.overResizeElement === this) {
          delete DDManager.overResizeElement;
        }
      }
      return this;
    }
    /** @internal */
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _mouseOver(e4) {
      if (DDManager.overResizeElement || DDManager.dragElement)
        return;
      DDManager.overResizeElement = this;
      this.el.classList.remove("ui-resizable-autohide");
    }
    /** @internal */
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    _mouseOut(e4) {
      if (DDManager.overResizeElement !== this)
        return;
      delete DDManager.overResizeElement;
      this.el.classList.add("ui-resizable-autohide");
    }
    /** @internal */
    _setupHandlers() {
      this.handlers = this.option.handles.split(",").map((dir) => dir.trim()).map((dir) => new DDResizableHandle(this.el, dir, {
        element: this.option.element,
        start: (event) => this._resizeStart(event),
        stop: (event) => this._resizeStop(event),
        move: (event) => this._resizing(event, dir)
      }));
      return this;
    }
    /** @internal */
    _resizeStart(event) {
      this.sizeToContent = Utils.shouldSizeToContent(this.el.gridstackNode, true);
      this.originalRect = this.el.getBoundingClientRect();
      this.scrollEl = Utils.getScrollElement(this.el);
      this.scrollY = this.scrollEl.scrollTop;
      this.scrolled = 0;
      this.startEvent = event;
      this._setupHelper();
      this._applyChange();
      const ev = Utils.initEvent(event, { type: "resizestart", target: this.el });
      if (this.option.start) {
        this.option.start(ev, this._ui());
      }
      this.el.classList.add("ui-resizable-resizing");
      this.triggerEvent("resizestart", ev);
      return this;
    }
    /** @internal */
    _resizing(event, dir) {
      this.scrolled = this.scrollEl.scrollTop - this.scrollY;
      this.temporalRect = this._getChange(event, dir);
      this._applyChange();
      const ev = Utils.initEvent(event, { type: "resize", target: this.el });
      ev.resizeDir = dir;
      ev.hasMovedX = this.option.rtl ? dir.includes("e") : dir.includes("w");
      ev.hasMovedY = dir.includes("n");
      if (this.option.resize) {
        this.option.resize(ev, this._ui());
      }
      this.triggerEvent("resize", ev);
      return this;
    }
    /** @internal */
    _resizeStop(event) {
      const ev = Utils.initEvent(event, { type: "resizestop", target: this.el });
      this._cleanHelper();
      if (this.option.stop) {
        this.option.stop(ev);
      }
      this.el.classList.remove("ui-resizable-resizing");
      this.triggerEvent("resizestop", ev);
      delete this.startEvent;
      delete this.originalRect;
      delete this.temporalRect;
      delete this.scrollY;
      delete this.scrolled;
      return this;
    }
    /** @internal */
    _setupHelper() {
      this.elOriginStyleVal = _DDResizable._originStyleProp.map((prop) => this.el.style[prop]);
      this.parentOriginStylePosition = this.el.parentElement.style.position;
      const parent = this.el.parentElement;
      const dragTransform = Utils.getValuesFromTransformedElement(parent);
      this.rectScale = {
        x: dragTransform.xScale,
        y: dragTransform.yScale
      };
      if (getComputedStyle(this.el.parentElement).position.match(/static/)) {
        this.el.parentElement.style.position = "relative";
      }
      this.el.style.position = "absolute";
      this.el.style.opacity = "0.8";
      return this;
    }
    /** @internal */
    _cleanHelper() {
      _DDResizable._originStyleProp.forEach((prop, i4) => {
        this.el.style[prop] = this.elOriginStyleVal[i4] || null;
      });
      this.el.parentElement.style.position = this.parentOriginStylePosition || null;
      return this;
    }
    /** @internal */
    _getChange(event, dir) {
      const oEvent = this.startEvent;
      const newRect = {
        width: this.originalRect.width,
        height: this.originalRect.height + this.scrolled,
        left: this.originalRect.left,
        right: this.originalRect.right,
        top: this.originalRect.top - this.scrolled
      };
      const offsetX = event.clientX - oEvent.clientX;
      const offsetY = this.sizeToContent ? 0 : event.clientY - oEvent.clientY;
      let moveLeft;
      let moveUp;
      const isRtl = this.option.rtl;
      if (!isRtl && dir.indexOf("e") > -1) {
        newRect.width += offsetX;
      } else if (isRtl && dir.indexOf("w") > -1) {
        newRect.width -= offsetX;
      } else if (!isRtl && dir.indexOf("w") > -1) {
        newRect.width -= offsetX;
        newRect.left += offsetX;
        moveLeft = true;
      } else if (isRtl && dir.indexOf("e") > -1) {
        newRect.width += offsetX;
        newRect.right += offsetX;
        moveLeft = true;
      }
      if (dir.indexOf("s") > -1) {
        newRect.height += offsetY;
      } else if (dir.indexOf("n") > -1) {
        newRect.height -= offsetY;
        newRect.top += offsetY;
        moveUp = true;
      }
      const constrain = this._constrainSize(newRect.width, newRect.height, moveLeft, moveUp);
      if (Math.round(newRect.width) !== Math.round(constrain.width)) {
        if (!isRtl && dir.indexOf("w") > -1) {
          newRect.left += newRect.width - constrain.width;
        } else if (isRtl && dir.indexOf("e") > -1) {
          newRect.right -= newRect.width - constrain.width;
        }
        newRect.width = constrain.width;
      }
      if (Math.round(newRect.height) !== Math.round(constrain.height)) {
        if (dir.indexOf("n") > -1) {
          newRect.top += newRect.height - constrain.height;
        }
        newRect.height = constrain.height;
      }
      return newRect;
    }
    /** @internal constrain the size to the set min/max values */
    _constrainSize(oWidth, oHeight, moveLeft, moveUp) {
      const o4 = this.option;
      const maxWidth = (moveLeft ? o4.maxWidthMoveLeft : o4.maxWidth) || Number.MAX_SAFE_INTEGER;
      const minWidth = o4.minWidth / this.rectScale.x || oWidth;
      const maxHeight = (moveUp ? o4.maxHeightMoveUp : o4.maxHeight) || Number.MAX_SAFE_INTEGER;
      const minHeight = o4.minHeight / this.rectScale.y || oHeight;
      const width = Math.min(maxWidth, Math.max(minWidth, oWidth));
      const height = Math.min(maxHeight, Math.max(minHeight, oHeight));
      return { width, height };
    }
    /** @internal */
    _applyChange() {
      let containmentRect = { left: 0, right: 0, top: 0, width: 0, height: 0 };
      if (this.el.style.position === "absolute") {
        const containmentEl = this.el.parentElement;
        const { left, right, top } = containmentEl.getBoundingClientRect();
        containmentRect = { left, right, top, width: 0, height: 0 };
      }
      if (!this.temporalRect)
        return this;
      Object.entries(this.temporalRect).forEach(([key, value]) => {
        if (this.option.rtl ? key === "left" : key === "right")
          return;
        const scaleReciprocal = key === "width" || key === "left" || key === "right" ? this.rectScale.x : key === "height" || key === "top" ? this.rectScale.y : 1;
        let finalValue;
        if (key === "right") {
          finalValue = (containmentRect.right - value) * this.rectScale.x + "px";
        } else {
          finalValue = (value - containmentRect[key]) * scaleReciprocal + "px";
        }
        this.el.style[key] = finalValue;
      });
      return this;
    }
    /** @internal */
    _removeHandlers() {
      this.handlers.forEach((handle) => handle.destroy());
      delete this.handlers;
      return this;
    }
  };
  DDResizable._originStyleProp = ["width", "height", "position", "left", "right", "top", "opacity", "zIndex"];

  // node_modules/gridstack/dist/dd-draggable.js
  var skipMouseDown = 'input,textarea,button,select,option,[contenteditable="true"],.ui-resizable-handle';
  var DDDraggable = class _DDDraggable extends DDBaseImplement {
    constructor(el, option = {}) {
      super();
      this.el = el;
      this.option = option;
      this.dragTransform = {
        xScale: 1,
        yScale: 1,
        xOffset: 0,
        yOffset: 0
      };
      this._autoScrollTick = () => {
        const el2 = this.helper;
        const scrollCont = this._autoScrollContainer;
        if (!el2 || !scrollCont) {
          this._stopScrolling();
          return;
        }
        const clipping = this._getClipping(el2, scrollCont);
        if (clipping === 0) {
          this._stopScrolling();
          return;
        }
        if (!this._autoScrollMaxSpeed) {
          const viewportH = window.innerHeight || document.documentElement.clientHeight;
          this._autoScrollMaxSpeed = Math.max(viewportH / 150, 4);
        }
        const absPx = Math.abs(clipping);
        const speed = Math.min(absPx * 0.5, this._autoScrollMaxSpeed);
        const scrollAmount = clipping > 0 ? speed : -speed;
        const prevScroll = scrollCont.scrollTop;
        scrollCont.scrollTop += scrollAmount;
        if (scrollCont.scrollTop === prevScroll) {
          this._stopScrolling();
          return;
        }
        if (this.dragging && this.lastDrag) {
          this._dragFollow(this.lastDrag);
          this._callDrag(this.lastDrag);
        }
        this._autoScrollAnimId = requestAnimationFrame(this._autoScrollTick);
      };
      const handleName = option?.handle?.substring(1);
      const n3 = el.gridstackNode;
      this.dragEls = !handleName || el.classList.contains(handleName) ? [el] : n3?.subGrid ? [el.querySelector(option.handle) || el] : this.getAllHandles();
      if (this.dragEls.length === 0) {
        this.dragEls = [el];
      }
      this._mouseDown = this._mouseDown.bind(this);
      this._mouseMove = this._mouseMove.bind(this);
      this._mouseUp = this._mouseUp.bind(this);
      this._keyEvent = this._keyEvent.bind(this);
      this.enable();
    }
    /** return all handles omitting other nested `.grid-stack-item` children (in case node.subGrid isn't set for some reason) */
    getAllHandles() {
      return Array.from(this.el.querySelectorAll(this.option.handle)).filter((node) => {
        if (!(node instanceof HTMLElement))
          return false;
        const owner = node.closest(".grid-stack-item");
        return owner === this.el || !owner;
      });
    }
    on(event, callback) {
      super.on(event, callback);
    }
    off(event) {
      super.off(event);
    }
    enable() {
      if (this.disabled === false)
        return;
      super.enable();
      this.dragEls.forEach((dragEl) => {
        dragEl.addEventListener("mousedown", this._mouseDown);
        if (isTouch) {
          dragEl.addEventListener("touchstart", touchstart);
          dragEl.addEventListener("pointerdown", pointerdown);
        }
      });
      this.el.classList.remove("ui-draggable-disabled");
    }
    disable(forDestroy = false) {
      if (this.disabled === true)
        return;
      super.disable();
      this.dragEls.forEach((dragEl) => {
        dragEl.removeEventListener("mousedown", this._mouseDown);
        if (isTouch) {
          dragEl.removeEventListener("touchstart", touchstart);
          dragEl.removeEventListener("pointerdown", pointerdown);
        }
      });
      if (!forDestroy)
        this.el.classList.add("ui-draggable-disabled");
    }
    destroy() {
      if (this.dragTimeout)
        window.clearTimeout(this.dragTimeout);
      delete this.dragTimeout;
      if (this.mouseDownEvent)
        this._mouseUp(this.mouseDownEvent);
      this.disable(true);
      delete this.el;
      delete this.helper;
      delete this.option;
      super.destroy();
    }
    updateOption(opts) {
      Object.keys(opts).forEach((key) => this.option[key] = opts[key]);
      return this;
    }
    /** @internal call when mouse goes down before a dragstart happens */
    _mouseDown(e4) {
      if (DDTouch.touchHandled && e4.isTrusted)
        DDTouch.touchHandled = false;
      if (DDManager.mouseHandled)
        return;
      if (e4.button !== 0)
        return true;
      if (!this.dragEls.find((el) => el === e4.target) && e4.target.closest(skipMouseDown))
        return true;
      if (this.option.cancel) {
        if (e4.target.closest(this.option.cancel))
          return true;
      }
      this.mouseDownEvent = e4;
      delete this.dragging;
      delete DDManager.dragElement;
      delete DDManager.dropElement;
      delete this._autoScrollMaxSpeed;
      delete this._autoScrollContainer;
      document.addEventListener("mousemove", this._mouseMove, { capture: true, passive: true });
      document.addEventListener("mouseup", this._mouseUp, true);
      if (isTouch) {
        e4.currentTarget.addEventListener("touchmove", touchmove);
        e4.currentTarget.addEventListener("touchend", touchend);
      }
      e4.preventDefault();
      if (document.activeElement)
        document.activeElement.blur();
      DDManager.mouseHandled = true;
      return true;
    }
    /** @internal method to call actual drag event */
    _callDrag(e4) {
      if (!this.dragging)
        return;
      const ev = Utils.initEvent(e4, { target: this.el, type: "drag" });
      if (this.option.drag) {
        this.option.drag(ev, this.ui());
      }
      this.triggerEvent("drag", ev);
    }
    /** @internal called when the main page (after successful mousedown) receives a move event to drag the item around the screen */
    _mouseMove(e4) {
      const s4 = this.mouseDownEvent;
      this.lastDrag = e4;
      if (this.dragging) {
        this._dragFollow(e4);
        if (DDManager.pauseDrag) {
          const pause = Number.isInteger(DDManager.pauseDrag) ? DDManager.pauseDrag : 100;
          if (this.dragTimeout)
            window.clearTimeout(this.dragTimeout);
          this.dragTimeout = window.setTimeout(() => this._callDrag(e4), pause);
        } else {
          this._callDrag(e4);
        }
      } else if (Math.abs(e4.x - s4.x) + Math.abs(e4.y - s4.y) > 3) {
        this.dragging = true;
        DDManager.dragElement = this;
        const grid = this.el.gridstackNode?.grid;
        if (grid) {
          DDManager.dropElement = grid.el.ddElement.ddDroppable;
        } else {
          delete DDManager.dropElement;
        }
        this.helper = this._createHelper();
        this._setupHelperContainmentStyle();
        this.dragTransform = Utils.getValuesFromTransformedElement(this.helperContainment);
        this.dragOffset = this._getDragOffset(e4, this.el, this.helperContainment);
        this._setupHelperStyle(e4);
        const ev = Utils.initEvent(e4, { target: this.el, type: "dragstart" });
        if (this.option.start) {
          this.option.start(ev, this.ui());
        }
        this.triggerEvent("dragstart", ev);
        document.addEventListener("keydown", this._keyEvent);
      }
      return true;
    }
    /** @internal call when the mouse gets released to drop the item at current location */
    _mouseUp(e4) {
      this._stopScrolling();
      document.removeEventListener("mousemove", this._mouseMove, true);
      document.removeEventListener("mouseup", this._mouseUp, true);
      if (isTouch && e4.currentTarget) {
        e4.currentTarget.removeEventListener("touchmove", touchmove, true);
        e4.currentTarget.removeEventListener("touchend", touchend, true);
      }
      if (this.dragging) {
        delete this.dragging;
        delete this.el.gridstackNode?._origRotate;
        document.removeEventListener("keydown", this._keyEvent);
        if (DDManager.dropElement?.el === this.el.parentElement) {
          delete DDManager.dropElement;
        }
        this.helperContainment.style.position = this.parentOriginStylePosition || null;
        if (this.helper !== this.el)
          this.helper.remove();
        this._removeHelperStyle();
        const ev = Utils.initEvent(e4, { target: this.el, type: "dragstop" });
        if (this.option.stop) {
          this.option.stop(ev);
        }
        this.triggerEvent("dragstop", ev);
        if (DDManager.dropElement) {
          DDManager.dropElement.drop(e4);
        }
      }
      delete this.helper;
      delete this.mouseDownEvent;
      delete DDManager.dragElement;
      delete DDManager.dropElement;
      delete DDManager.mouseHandled;
      e4.preventDefault();
    }
    /** @internal call when keys are being pressed - use Esc to cancel, R to rotate */
    _keyEvent(e4) {
      const n3 = this.el.gridstackNode;
      const grid = n3?.grid || DDManager.dropElement?.el?.gridstack;
      if (e4.key === "Escape") {
        if (n3 && n3._origRotate) {
          n3._orig = n3._origRotate;
          delete n3._origRotate;
        }
        grid?.cancelDrag();
        this._mouseUp(this.mouseDownEvent);
      } else if (n3 && grid && (e4.key === "r" || e4.key === "R")) {
        if (!Utils.canBeRotated(n3))
          return;
        n3._origRotate = n3._origRotate || { ...n3._orig };
        delete n3._moving;
        grid.setAnimation(false).rotate(n3.el, {
          top: -this.dragOffset.offsetTop,
          left: -this.dragOffset.offsetX
        }).setAnimation();
        n3._moving = true;
        this.dragOffset = this._getDragOffset(this.lastDrag, n3.el, this.helperContainment);
        this.helper.style.width = this.dragOffset.width + "px";
        this.helper.style.height = this.dragOffset.height + "px";
        Utils.swap(n3._orig, "w", "h");
        delete n3._rect;
        this._mouseMove(this.lastDrag);
      }
    }
    /** @internal create a clone copy (or user defined method) of the original drag item if set */
    _createHelper() {
      let helper = this.el;
      if (typeof this.option.helper === "function") {
        helper = this.option.helper(this.el);
      } else if (this.option.helper === "clone") {
        helper = Utils.cloneNode(this.el);
      }
      if (!helper.parentElement) {
        Utils.appendTo(helper, this.option.appendTo === "parent" ? this.el.parentElement : this.option.appendTo);
      }
      this.dragElementOriginStyle = _DDDraggable.originStyleProp.map((prop) => this.el.style[prop]);
      return helper;
    }
    /** @internal set the fix position of the dragged item */
    _setupHelperStyle(e4) {
      this.helper.classList.add("ui-draggable-dragging");
      this.el.gridstackNode?.grid?.el.classList.add("grid-stack-dragging");
      const style = this.helper.style;
      style.pointerEvents = "none";
      style.width = this.dragOffset.width + "px";
      style.height = this.dragOffset.height + "px";
      style.willChange = "left, right, top";
      style.position = "fixed";
      this._dragFollow(e4);
      style.transition = "none";
      setTimeout(() => {
        if (this.helper) {
          style.transition = null;
        }
      }, 0);
      return this;
    }
    /** @internal restore back the original style before dragging */
    _removeHelperStyle() {
      this.helper.classList.remove("ui-draggable-dragging");
      this.el.gridstackNode?.grid?.el.classList.remove("grid-stack-dragging");
      const node = this.helper?.gridstackNode;
      if (!node?._isAboutToRemove && this.dragElementOriginStyle) {
        const helper = this.helper;
        const transition = this.dragElementOriginStyle["transition"] || null;
        helper.style.transition = this.dragElementOriginStyle["transition"] = "none";
        _DDDraggable.originStyleProp.forEach((prop) => helper.style[prop] = this.dragElementOriginStyle[prop] || null);
        setTimeout(() => helper.style.transition = transition, 50);
      }
      delete this.dragElementOriginStyle;
      return this;
    }
    /** @internal updates the top/left position to follow the mouse */
    _dragFollow(e4) {
      const style = this.helper.style;
      const offset = this.dragOffset;
      if (this.option.rtl) {
        style.right = (window.innerWidth - e4.clientX + offset.offsetX) * this.dragTransform.xScale + "px";
        if (style.left)
          style.left = "";
      } else {
        style.left = (e4.clientX + offset.offsetX) * this.dragTransform.xScale + "px";
        if (style.right)
          style.right = "";
      }
      style.top = (e4.clientY + offset.offsetTop) * this.dragTransform.yScale + "px";
    }
    /** @internal */
    _setupHelperContainmentStyle() {
      this.helperContainment = this.helper.parentElement;
      if (this.helper.style.position !== "fixed") {
        this.parentOriginStylePosition = this.helperContainment.style.position;
        if (getComputedStyle(this.helperContainment).position.match(/static/)) {
          this.helperContainment.style.position = "relative";
        }
      }
      return this;
    }
    /** @internal */
    _getDragOffset(event, el, parent) {
      let xformOffsetX = 0;
      let xformOffsetY = 0;
      if (parent) {
        xformOffsetX = this.dragTransform.xOffset;
        xformOffsetY = this.dragTransform.yOffset;
      }
      const targetOffset = el.getBoundingClientRect();
      let x4 = this.option.rtl ? targetOffset.right : targetOffset.left;
      let offsetX = this.option.rtl ? event.clientX - targetOffset.right + xformOffsetX : -event.clientX + targetOffset.left - xformOffsetX;
      return {
        x: x4,
        top: targetOffset.top,
        offsetX,
        offsetTop: -event.clientY + targetOffset.top - xformOffsetY,
        width: targetOffset.width * this.dragTransform.xScale,
        height: targetOffset.height * this.dragTransform.yScale
      };
    }
    /** @internal starts or continues auto-scroll when the dragged helper is clipped by the scroll container.
     * Takes the grid's own element to find the scroll container so external/sidebar drags work too (#2074). */
    updateScrollPosition(gridEl) {
      this._autoScrollContainer = Utils.getScrollElement(gridEl);
      const clipping = this._getClipping(this.helper, this._autoScrollContainer);
      if (clipping === 0) {
        this._stopScrolling();
      } else if (!this._autoScrollAnimId) {
        this._autoScrollAnimId = requestAnimationFrame(this._autoScrollTick);
      }
    }
    /** @internal compute how many pixels the element is clipped: negative = above, positive = below, 0 = fully inside OR outside (stop scrolling) */
    _getClipping(el, scrollEl) {
      const elRect = el.getBoundingClientRect();
      const scrollRect = scrollEl.getBoundingClientRect();
      const viewportH = window.innerHeight || document.documentElement.clientHeight;
      if (elRect.bottom < scrollRect.top || elRect.top > scrollRect.bottom)
        return 0;
      const clippedBelow = elRect.bottom - Math.min(scrollRect.bottom, viewportH);
      const clippedAbove = elRect.top - Math.max(scrollRect.top, 0);
      if (clippedAbove < 0)
        return clippedAbove;
      if (clippedBelow > 0)
        return clippedBelow;
      return 0;
    }
    /** @internal stop any active auto-scroll animation */
    _stopScrolling() {
      if (this._autoScrollAnimId) {
        cancelAnimationFrame(this._autoScrollAnimId);
        delete this._autoScrollAnimId;
      }
    }
    /** @internal TODO: set to public as called by DDDroppable! */
    ui() {
      const containmentEl = this.el.parentElement;
      const containmentRect = containmentEl.getBoundingClientRect();
      const offset = this.helper.getBoundingClientRect();
      const leftPos = this.option.rtl ? (containmentRect.right - offset.right) * this.dragTransform.xScale : (offset.left - containmentRect.left) * this.dragTransform.xScale;
      return {
        position: {
          top: (offset.top - containmentRect.top) * this.dragTransform.yScale,
          left: leftPos
        }
        /* not used by GridStack for now...
        helper: [this.helper], //The object arr representing the helper that's being dragged.
        offset: { top: offset.top, left: offset.left } // Current offset position of the helper as { top, left } object.
        */
      };
    }
  };
  DDDraggable.originStyleProp = ["width", "height", "transform", "transform-origin", "transition", "pointerEvents", "position", "left", "right", "top", "minWidth", "willChange"];

  // node_modules/gridstack/dist/dd-droppable.js
  var DDDroppable = class extends DDBaseImplement {
    constructor(el, option = {}) {
      super();
      this.el = el;
      this.option = option;
      this._mouseEnter = this._mouseEnter.bind(this);
      this._mouseLeave = this._mouseLeave.bind(this);
      this.enable();
      this._setupAccept();
    }
    on(event, callback) {
      super.on(event, callback);
    }
    off(event) {
      super.off(event);
    }
    enable() {
      if (this.disabled === false)
        return;
      super.enable();
      this.el.classList.add("ui-droppable");
      this.el.classList.remove("ui-droppable-disabled");
      this.el.addEventListener("mouseenter", this._mouseEnter);
      this.el.addEventListener("mouseleave", this._mouseLeave);
      if (isTouch) {
        this.el.addEventListener("pointerenter", pointerenter);
        this.el.addEventListener("pointerleave", pointerleave);
      }
    }
    disable(forDestroy = false) {
      if (this.disabled === true)
        return;
      super.disable();
      this.el.classList.remove("ui-droppable");
      if (!forDestroy)
        this.el.classList.add("ui-droppable-disabled");
      this.el.removeEventListener("mouseenter", this._mouseEnter);
      this.el.removeEventListener("mouseleave", this._mouseLeave);
      if (isTouch) {
        this.el.removeEventListener("pointerenter", pointerenter);
        this.el.removeEventListener("pointerleave", pointerleave);
      }
    }
    destroy() {
      this.disable(true);
      this.el.classList.remove("ui-droppable");
      this.el.classList.remove("ui-droppable-disabled");
      super.destroy();
    }
    updateOption(opts) {
      Object.keys(opts).forEach((key) => this.option[key] = opts[key]);
      this._setupAccept();
      return this;
    }
    /** @internal called when the cursor enters our area - prepare for a possible drop and track leaving */
    _mouseEnter(e4) {
      if (!DDManager.dragElement)
        return;
      if (DDTouch.touchHandled && e4.isTrusted)
        return;
      if (!this._canDrop(DDManager.dragElement.el))
        return;
      e4.preventDefault();
      e4.stopPropagation();
      DDManager.dragElement._stopScrolling();
      if (DDManager.dropElement && DDManager.dropElement !== this) {
        DDManager.dropElement._mouseLeave(e4, true);
      }
      DDManager.dropElement = this;
      const ev = Utils.initEvent(e4, { target: this.el, type: "dropover" });
      if (this.option.over) {
        this.option.over(ev, this._ui(DDManager.dragElement));
      }
      this.triggerEvent("dropover", ev);
      this.el.classList.add("ui-droppable-over");
    }
    /** @internal called when the item is leaving our area, stop tracking if we had moving item */
    _mouseLeave(e4, calledByEnter = false) {
      if (!DDManager.dragElement || DDManager.dropElement !== this)
        return;
      e4.preventDefault();
      e4.stopPropagation();
      if (calledByEnter)
        DDManager.dragElement._stopScrolling();
      const ev = Utils.initEvent(e4, { target: this.el, type: "dropout" });
      if (this.option.out) {
        this.option.out(ev, this._ui(DDManager.dragElement));
      }
      this.triggerEvent("dropout", ev);
      if (DDManager.dropElement === this) {
        delete DDManager.dropElement;
        if (!calledByEnter) {
          let parentDrop;
          let parent = this.el.parentElement;
          while (!parentDrop && parent) {
            parentDrop = parent.ddElement?.ddDroppable;
            parent = parent.parentElement;
          }
          if (parentDrop) {
            parentDrop._mouseEnter(e4);
          }
        }
      }
    }
    /** item is being dropped on us - called by the drag mouseup handler - this calls the client drop event */
    drop(e4) {
      e4.preventDefault();
      const ev = Utils.initEvent(e4, { target: this.el, type: "drop" });
      if (this.option.drop) {
        this.option.drop(ev, this._ui(DDManager.dragElement));
      }
      this.triggerEvent("drop", ev);
    }
    /** @internal true if element matches the string/method accept option */
    _canDrop(el) {
      return el && (!this.accept || this.accept(el));
    }
    /** @internal */
    _setupAccept() {
      if (!this.option.accept)
        return this;
      if (typeof this.option.accept === "string") {
        this.accept = (el) => el.classList.contains(this.option.accept) || el.matches(this.option.accept);
      } else {
        this.accept = this.option.accept;
      }
      return this;
    }
    /** @internal */
    _ui(drag) {
      return {
        draggable: drag.el,
        ...drag.ui()
      };
    }
  };

  // node_modules/gridstack/dist/dd-element.js
  var DDElement = class _DDElement {
    static init(el) {
      if (!el.ddElement) {
        el.ddElement = new _DDElement(el);
      }
      return el.ddElement;
    }
    constructor(el) {
      this.el = el;
    }
    on(eventName, callback) {
      if (this.ddDraggable && ["drag", "dragstart", "dragstop"].indexOf(eventName) > -1) {
        this.ddDraggable.on(eventName, callback);
      } else if (this.ddDroppable && ["drop", "dropover", "dropout"].indexOf(eventName) > -1) {
        this.ddDroppable.on(eventName, callback);
      } else if (this.ddResizable && ["resizestart", "resize", "resizestop"].indexOf(eventName) > -1) {
        this.ddResizable.on(eventName, callback);
      }
      return this;
    }
    off(eventName) {
      if (this.ddDraggable && ["drag", "dragstart", "dragstop"].indexOf(eventName) > -1) {
        this.ddDraggable.off(eventName);
      } else if (this.ddDroppable && ["drop", "dropover", "dropout"].indexOf(eventName) > -1) {
        this.ddDroppable.off(eventName);
      } else if (this.ddResizable && ["resizestart", "resize", "resizestop"].indexOf(eventName) > -1) {
        this.ddResizable.off(eventName);
      }
      return this;
    }
    setupDraggable(opts) {
      if (!this.ddDraggable) {
        this.ddDraggable = new DDDraggable(this.el, opts);
      } else {
        this.ddDraggable.updateOption(opts);
      }
      return this;
    }
    cleanDraggable() {
      if (this.ddDraggable) {
        this.ddDraggable.destroy();
        delete this.ddDraggable;
      }
      return this;
    }
    setupResizable(opts) {
      if (!this.ddResizable) {
        this.ddResizable = new DDResizable(this.el, opts);
      } else {
        this.ddResizable.updateOption(opts);
      }
      return this;
    }
    cleanResizable() {
      if (this.ddResizable) {
        this.ddResizable.destroy();
        delete this.ddResizable;
      }
      return this;
    }
    setupDroppable(opts) {
      if (!this.ddDroppable) {
        this.ddDroppable = new DDDroppable(this.el, opts);
      } else {
        this.ddDroppable.updateOption(opts);
      }
      return this;
    }
    cleanDroppable() {
      if (this.ddDroppable) {
        this.ddDroppable.destroy();
        delete this.ddDroppable;
      }
      return this;
    }
  };

  // node_modules/gridstack/dist/dd-gridstack.js
  var DDGridStack = class {
    /**
     * Enable/disable/configure resizing for grid elements.
     *
     * @param el - Grid item element(s) to configure
     * @param opts - Resize options or command ('enable', 'disable', 'destroy', 'option', or config object)
     * @param key - Option key when using 'option' command
     * @param value - Option value when using 'option' command
     * @returns this instance for chaining
     *
     * @example
     * dd.resizable(element, 'enable');  // Enable resizing
     * dd.resizable(element, 'option', 'minWidth', 100);  // Set minimum width
     */
    resizable(el, opts, key, value) {
      this._getDDElements(el, opts).forEach((dEl) => {
        if (opts === "disable" || opts === "enable") {
          dEl.ddResizable && dEl.ddResizable[opts]();
        } else if (opts === "destroy") {
          dEl.ddResizable && dEl.cleanResizable();
        } else if (opts === "option") {
          dEl.setupResizable({ [key]: value });
        } else {
          const n3 = dEl.el.gridstackNode;
          const grid = n3.grid;
          let handles = dEl.el.getAttribute("gs-resize-handles") || grid.opts.resizable.handles || "e,s,se";
          if (handles === "all")
            handles = "n,e,s,w,se,sw,ne,nw";
          const autoHide = !grid.opts.alwaysShowResizeHandle;
          dEl.setupResizable({
            ...grid.opts.resizable,
            ...{ handles, autoHide },
            ...{
              start: opts.start,
              stop: opts.stop,
              resize: opts.resize,
              rtl: opts.rtl
            }
          });
        }
      });
      return this;
    }
    /**
     * Enable/disable/configure dragging for grid elements.
     *
     * @param el - Grid item element(s) to configure
     * @param opts - Drag options or command ('enable', 'disable', 'destroy', 'option', or config object)
     * @param key - Option key when using 'option' command
     * @param value - Option value when using 'option' command
     * @param rtl - Are we in rtl mode?
     * @returns this instance for chaining
     *
     * @example
     * dd.draggable(element, 'enable');  // Enable dragging
     * dd.draggable(element, {handle: '.drag-handle'});  // Configure drag handle
     */
    draggable(el, opts, key, value) {
      this._getDDElements(el, opts).forEach((dEl) => {
        if (opts === "disable" || opts === "enable") {
          dEl.ddDraggable && dEl.ddDraggable[opts]();
        } else if (opts === "destroy") {
          dEl.ddDraggable && dEl.cleanDraggable();
        } else if (opts === "option") {
          dEl.setupDraggable({ [key]: value });
        } else {
          const grid = dEl.el.gridstackNode.grid;
          dEl.setupDraggable({
            ...grid.opts.draggable,
            ...{
              // containment: (grid.parentGridNode && grid.opts.dragOut === false) ? grid.el.parentElement : (grid.opts.draggable.containment || null),
              start: opts.start,
              stop: opts.stop,
              drag: opts.drag,
              rtl: opts.rtl
            }
          });
        }
      });
      return this;
    }
    dragIn(el, opts) {
      this._getDDElements(el).forEach((dEl) => dEl.setupDraggable(opts));
      return this;
    }
    droppable(el, opts, key, value) {
      if (typeof opts.accept === "function" && !opts._accept) {
        opts._accept = opts.accept;
        opts.accept = (el2) => opts._accept(el2);
      }
      this._getDDElements(el, opts).forEach((dEl) => {
        if (opts === "disable" || opts === "enable") {
          dEl.ddDroppable && dEl.ddDroppable[opts]();
        } else if (opts === "destroy") {
          dEl.ddDroppable && dEl.cleanDroppable();
        } else if (opts === "option") {
          dEl.setupDroppable({ [key]: value });
        } else {
          dEl.setupDroppable(opts);
        }
      });
      return this;
    }
    /** true if element is droppable */
    isDroppable(el) {
      return !!(el?.ddElement?.ddDroppable && !el.ddElement.ddDroppable.disabled);
    }
    /** true if element is draggable */
    isDraggable(el) {
      return !!(el?.ddElement?.ddDraggable && !el.ddElement.ddDraggable.disabled);
    }
    /** true if element is draggable */
    isResizable(el) {
      return !!(el?.ddElement?.ddResizable && !el.ddElement.ddResizable.disabled);
    }
    on(el, name, callback) {
      this._getDDElements(el).forEach((dEl) => dEl.on(name, (event) => {
        callback(event, DDManager.dragElement ? DDManager.dragElement.el : event.target, DDManager.dragElement ? DDManager.dragElement.helper : null);
      }));
      return this;
    }
    off(el, name) {
      this._getDDElements(el).forEach((dEl) => dEl.off(name));
      return this;
    }
    /** @internal returns a list of DD elements, creating them on the fly by default unless option is to destroy or disable */
    _getDDElements(els, opts) {
      const create = els.gridstack || opts !== "destroy" && opts !== "disable";
      const hosts = Utils.getElements(els);
      if (!hosts.length)
        return [];
      const list = hosts.map((e4) => e4.ddElement || (create ? DDElement.init(e4) : null)).filter((d5) => d5);
      return list;
    }
  };

  // node_modules/gridstack/dist/gridstack.js
  var dd = new DDGridStack();
  var GridStack = class _GridStack {
    /**
     * initializing the HTML element, or selector string, into a grid will return the grid. Calling it again will
     * simply return the existing instance (ignore any passed options). There is also an initAll() version that support
     * multiple grids initialization at once. Or you can use addGrid() to create the entire grid from JSON.
     * @param options grid options (optional)
     * @param elOrString element or CSS selector (first one used) to convert to a grid (default to '.grid-stack' class selector)
     *
     * @example
     * const grid = GridStack.init();
     *
     * Note: the HTMLElement (of type GridHTMLElement) will store a `gridstack: GridStack` value that can be retrieve later
     * const grid = document.querySelector('.grid-stack').gridstack;
     */
    static init(options = {}, elOrString = ".grid-stack") {
      if (typeof document === "undefined")
        return null;
      const el = _GridStack.getGridElement(elOrString);
      if (!el) {
        if (typeof elOrString === "string") {
          console.error('GridStack.initAll() no grid was found with selector "' + elOrString + '" - element missing or wrong selector ?\nNote: ".grid-stack" is required for proper CSS styling and drag/drop, and is the default selector.');
        } else {
          console.error("GridStack.init() no grid element was passed.");
        }
        return null;
      }
      if (!el.gridstack) {
        el.gridstack = new _GridStack(el, Utils.cloneDeep(options));
      }
      return el.gridstack;
    }
    /**
     * Will initialize a list of elements (given a selector) and return an array of grids.
     * @param options grid options (optional)
     * @param selector elements selector to convert to grids (default to '.grid-stack' class selector)
     *
     * @example
     * const grids = GridStack.initAll();
     * grids.forEach(...)
     */
    static initAll(options = {}, selector = ".grid-stack") {
      const grids = [];
      if (typeof document === "undefined")
        return grids;
      _GridStack.getGridElements(selector).forEach((el) => {
        if (!el.gridstack) {
          el.gridstack = new _GridStack(el, Utils.cloneDeep(options));
        }
        grids.push(el.gridstack);
      });
      if (grids.length === 0) {
        console.error('GridStack.initAll() no grid was found with selector "' + selector + '" - element missing or wrong selector ?\nNote: ".grid-stack" is required for proper CSS styling and drag/drop, and is the default selector.');
      }
      return grids;
    }
    /**
     * call to create a grid with the given options, including loading any children from JSON structure. This will call GridStack.init(), then
     * grid.load() on any passed children (recursively). Great alternative to calling init() if you want entire grid to come from
     * JSON serialized data, including options.
     * @param parent HTML element parent to the grid
     * @param opt grids options used to initialize the grid, and list of children
     */
    static addGrid(parent, opt = {}) {
      if (!parent)
        return null;
      let el = parent;
      if (el.gridstack) {
        const grid2 = el.gridstack;
        if (opt)
          grid2.opts = { ...grid2.opts, ...opt };
        if (opt.children !== void 0)
          grid2.load(opt.children);
        return grid2;
      }
      const parentIsGrid = parent.classList.contains("grid-stack");
      if (!parentIsGrid || _GridStack.addRemoveCB) {
        if (_GridStack.addRemoveCB) {
          el = _GridStack.addRemoveCB(parent, opt, true, true);
        } else {
          el = Utils.createDiv(["grid-stack", opt.class], parent);
        }
      }
      const grid = _GridStack.init(opt, el);
      return grid;
    }
    /** call this method to register your engine instead of the default one.
     * See instead `GridStackOptions.engineClass` if you only need to
     * replace just one instance.
     */
    static registerEngine(engineClass) {
      _GridStack.engineClass = engineClass;
    }
    /**
     * @internal create placeholder DIV as needed
     * @returns the placeholder element for indicating drop zones during drag operations
     */
    get placeholder() {
      if (!this._placeholder) {
        this._placeholder = Utils.createDiv([this.opts.placeholderClass, gridDefaults.itemClass, this.opts.itemClass]);
        const placeholderChild = Utils.createDiv(["placeholder-content"], this._placeholder);
        if (this.opts.placeholderText) {
          placeholderChild.textContent = this.opts.placeholderText;
        }
      }
      return this._placeholder;
    }
    /**
     * Construct a grid item from the given element and options
     * @param el the HTML element tied to this grid after it's been initialized
     * @param opts grid options - public for classes to access, but use methods to modify!
     */
    constructor(el, opts = {}) {
      this.el = el;
      this.opts = opts;
      this.animationDelay = 300 + 10;
      this._gsEventHandler = {};
      this._extraDragRow = 0;
      this.dragTransform = { xScale: 1, yScale: 1, xOffset: 0, yOffset: 0 };
      el.gridstack = this;
      this.opts = opts = opts || {};
      if (!el.classList.contains("grid-stack")) {
        this.el.classList.add("grid-stack");
      }
      if (opts.row) {
        opts.minRow = opts.maxRow = opts.row;
        delete opts.row;
      }
      const rowAttr = Utils.toNumber(el.getAttribute("gs-row"));
      if (opts.column === "auto") {
        delete opts.column;
      }
      if (opts.alwaysShowResizeHandle !== void 0) {
        opts._alwaysShowResizeHandle = opts.alwaysShowResizeHandle;
      }
      const resp = opts.columnOpts;
      if (resp) {
        const bk = resp.breakpoints;
        if (!resp.columnWidth && !bk?.length) {
          delete opts.columnOpts;
        } else {
          resp.columnMax = resp.columnMax || 12;
          if (bk?.length > 1)
            bk.sort((a4, b4) => (b4.w || 0) - (a4.w || 0));
        }
      }
      const defaults = {
        ...Utils.cloneDeep(gridDefaults),
        column: Utils.toNumber(el.getAttribute("gs-column")) || gridDefaults.column,
        minRow: rowAttr ? rowAttr : Utils.toNumber(el.getAttribute("gs-min-row")) || gridDefaults.minRow,
        maxRow: rowAttr ? rowAttr : Utils.toNumber(el.getAttribute("gs-max-row")) || gridDefaults.maxRow,
        staticGrid: Utils.toBool(el.getAttribute("gs-static")) || gridDefaults.staticGrid,
        sizeToContent: Utils.toBool(el.getAttribute("gs-size-to-content")) || void 0,
        draggable: {
          handle: (opts.handleClass ? "." + opts.handleClass : opts.handle ? opts.handle : "") || gridDefaults.draggable.handle
        },
        removableOptions: {
          accept: opts.itemClass || gridDefaults.removableOptions.accept,
          decline: gridDefaults.removableOptions.decline
        }
      };
      if (el.getAttribute("gs-animate")) {
        defaults.animate = Utils.toBool(el.getAttribute("gs-animate"));
      }
      opts = Utils.defaults(opts, defaults);
      this._initMargin();
      this.checkDynamicColumn();
      this._updateColumnVar(opts);
      if (opts.rtl === "auto") {
        opts.rtl = el.style.direction === "rtl";
      }
      if (opts.rtl) {
        this.el.classList.add("grid-stack-rtl");
      }
      const parentGridItem = this.el.closest("." + gridDefaults.itemClass);
      const parentNode = parentGridItem?.gridstackNode;
      if (parentNode) {
        parentNode.subGrid = this;
        this.parentGridNode = parentNode;
        this.el.classList.add("grid-stack-nested");
        parentNode.el.classList.add("grid-stack-sub-grid");
      }
      this._isAutoCellHeight = opts.cellHeight === "auto";
      if (this._isAutoCellHeight || opts.cellHeight === "initial") {
        this.cellHeight(void 0);
      } else {
        if (typeof opts.cellHeight == "number" && opts.cellHeightUnit && opts.cellHeightUnit !== gridDefaults.cellHeightUnit) {
          opts.cellHeight = opts.cellHeight + opts.cellHeightUnit;
          delete opts.cellHeightUnit;
        }
        const val = opts.cellHeight;
        delete opts.cellHeight;
        this.cellHeight(val);
      }
      if (opts.alwaysShowResizeHandle === "mobile") {
        opts.alwaysShowResizeHandle = isTouch;
      }
      this._setStaticClass();
      const engineClass = opts.engineClass || _GridStack.engineClass || GridStackEngine;
      this.engine = new engineClass({
        column: this.getColumn(),
        float: opts.float,
        maxRow: opts.maxRow,
        onChange: (cbNodes) => {
          cbNodes.forEach((n3) => {
            const el2 = n3.el;
            if (!el2)
              return;
            if (n3._removeDOM) {
              if (el2)
                el2.remove();
              delete n3._removeDOM;
            } else {
              this._writePosAttr(el2, n3);
            }
          });
          this._updateContainerHeight();
        }
      });
      if (opts.auto) {
        this.batchUpdate();
        this.engine._loading = true;
        this.getGridItems().forEach((el2) => this._prepareElement(el2));
        delete this.engine._loading;
        this.batchUpdate(false);
      }
      if (opts.children) {
        const children = opts.children;
        delete opts.children;
        if (children.length)
          this.load(children);
      }
      this.setAnimation();
      if (opts.subGridDynamic && !DDManager.pauseDrag)
        DDManager.pauseDrag = true;
      if (opts.draggable?.pause !== void 0)
        DDManager.pauseDrag = opts.draggable.pause;
      this._setupRemoveDrop();
      this._setupAcceptWidget();
      this._updateResizeEvent();
    }
    _updateColumnVar(opts = this.opts) {
      this.el.classList.add("gs-" + opts.column);
      if (typeof opts.column === "number")
        this.el.style.setProperty("--gs-column-width", `${100 / opts.column}%`);
    }
    /**
     * add a new widget and returns it.
     *
     * Widget will be always placed even if result height is more than actual grid height.
     * You need to use `willItFit()` before calling addWidget for additional check.
     * See also `makeWidget(el)` for DOM element.
     *
     * @example
     * const grid = GridStack.init();
     * grid.addWidget({w: 3, content: 'hello'});
     *
     * @param w GridStackWidget definition. used MakeWidget(el) if you have dom element instead.
     */
    addWidget(w5) {
      if (!w5)
        return;
      if (typeof w5 === "string") {
        console.error("V11: GridStack.addWidget() does not support string anymore. see #2736");
        return;
      }
      if (w5.ELEMENT_NODE) {
        console.error("V11: GridStack.addWidget() does not support HTMLElement anymore. use makeWidget()");
        return this.makeWidget(w5);
      }
      let el;
      let node = w5;
      node.grid = this;
      if (node.el) {
        el = node.el;
      } else if (_GridStack.addRemoveCB) {
        el = _GridStack.addRemoveCB(this.el, w5, true, false);
      } else {
        el = this.createWidgetDivs(node);
      }
      if (!el)
        return;
      node = el.gridstackNode;
      if (node && el.parentElement === this.el && this.engine.nodes.find((n3) => n3._id === node._id))
        return el;
      const domAttr = this._readAttr(el);
      Utils.defaults(w5, domAttr);
      this.engine.prepareNode(w5);
      this.el.appendChild(el);
      this.makeWidget(el, w5);
      return el;
    }
    /**
     * Create the default grid item divs and content (possibly lazy loaded) by using GridStack.renderCB().
     *
     * @param n GridStackNode definition containing widget configuration
     * @returns the created HTML element with proper grid item structure
     *
     * @example
     * const element = grid.createWidgetDivs({ w: 2, h: 1, content: 'Hello World' });
     */
    createWidgetDivs(n3) {
      const el = Utils.createDiv(["grid-stack-item", this.opts.itemClass]);
      const cont = Utils.createDiv(["grid-stack-item-content"], el);
      if (Utils.lazyLoad(n3)) {
        if (!n3.visibleObservable) {
          n3.visibleObservable = new IntersectionObserver(([entry]) => {
            if (entry.isIntersecting) {
              n3.visibleObservable?.disconnect();
              delete n3.visibleObservable;
              _GridStack.renderCB(cont, n3);
              n3.grid?.prepareDragDrop(n3.el);
            }
          });
          window.setTimeout(() => n3.visibleObservable?.observe(el));
        }
      } else
        _GridStack.renderCB(cont, n3);
      return el;
    }
    /**
     * Convert an existing gridItem element into a sub-grid with the given (optional) options, else inherit them
     * from the parent's subGrid options.
     * @param el gridItem element to convert
     * @param ops (optional) sub-grid options, else default to node, then parent settings, else defaults
     * @param nodeToAdd (optional) node to add to the newly created sub grid (used when dragging over existing regular item)
     * @param saveContent if true (default) the html inside .grid-stack-content will be saved to child widget
     * @returns newly created grid
     */
    makeSubGrid(el, ops, nodeToAdd, saveContent = true) {
      let node = el.gridstackNode;
      if (!node) {
        node = this.makeWidget(el).gridstackNode;
      }
      if (node.subGrid?.el)
        return node.subGrid;
      let subGridTemplate;
      let grid = this;
      while (grid && !subGridTemplate) {
        subGridTemplate = grid.opts?.subGridOpts;
        grid = grid.parentGridNode?.grid;
      }
      ops = Utils.cloneDeep({
        // by default sub-grid inherit from us | parent, other than id, children, etc...
        ...this.opts,
        id: void 0,
        children: void 0,
        column: "auto",
        columnOpts: void 0,
        layout: "list",
        subGridOpts: void 0,
        ...subGridTemplate || {},
        ...ops || node.subGridOpts || {}
      });
      node.subGridOpts = ops;
      let autoColumn;
      if (ops.column === "auto") {
        autoColumn = true;
        ops.column = Math.max(node.w || 1, nodeToAdd?.w || 1);
        delete ops.columnOpts;
      }
      let content = node.el.querySelector(".grid-stack-item-content");
      let newItem;
      let newItemOpt;
      if (saveContent) {
        this._removeDD(node.el);
        newItemOpt = { ...node, x: 0, y: 0 };
        Utils.removeInternalForSave(newItemOpt);
        delete newItemOpt.subGridOpts;
        if (node.content) {
          newItemOpt.content = node.content;
          delete node.content;
        }
        if (_GridStack.addRemoveCB) {
          newItem = _GridStack.addRemoveCB(this.el, newItemOpt, true, false);
        } else {
          newItem = Utils.createDiv(["grid-stack-item"]);
          newItem.appendChild(content);
          content = Utils.createDiv(["grid-stack-item-content"], node.el);
        }
        this.prepareDragDrop(node.el);
      }
      if (nodeToAdd) {
        const w5 = autoColumn ? ops.column : node.w;
        const h5 = node.h + nodeToAdd.h;
        const style = node.el.style;
        style.transition = "none";
        this.update(node.el, { w: w5, h: h5 });
        setTimeout(() => style.transition = null);
      }
      const subGrid = node.subGrid = _GridStack.addGrid(content, ops);
      if (nodeToAdd?._moving)
        subGrid._isTemp = true;
      if (autoColumn)
        subGrid._autoColumn = true;
      if (saveContent) {
        subGrid.makeWidget(newItem, newItemOpt);
      }
      if (nodeToAdd) {
        if (nodeToAdd._moving) {
          window.setTimeout(() => Utils.simulateMouseEvent(nodeToAdd._event, "mouseenter", subGrid.el), 0);
        } else {
          subGrid.makeWidget(node.el, node);
        }
      }
      this.resizeToContentCheck(false, node);
      return subGrid;
    }
    /**
     * called when an item was converted into a nested grid to accommodate a dragged over item, but then item leaves - return back
     * to the original grid-item. Also called to remove empty sub-grids when last item is dragged out (since re-creating is simple)
     */
    removeAsSubGrid(nodeThatRemoved) {
      const pGrid = this.parentGridNode?.grid;
      if (!pGrid)
        return;
      pGrid.batchUpdate();
      pGrid.removeWidget(this.parentGridNode.el, true, true);
      this.engine.nodes.forEach((n3) => {
        n3.x += this.parentGridNode.x;
        n3.y += this.parentGridNode.y;
        pGrid.makeWidget(n3.el, n3);
      });
      pGrid.batchUpdate(false);
      if (this.parentGridNode)
        delete this.parentGridNode.subGrid;
      delete this.parentGridNode;
      if (nodeThatRemoved) {
        window.setTimeout(() => Utils.simulateMouseEvent(nodeThatRemoved._event, "mouseenter", pGrid.el), 0);
      }
    }
    /**
     * saves the current layout returning a list of widgets for serialization which might include any nested grids.
     * @param saveContent if true (default) the latest html inside .grid-stack-content will be saved to GridStackWidget.content field, else it will
     * be removed.
     * @param saveGridOpt if true (default false), save the grid options itself, so you can call the new GridStack.addGrid()
     * to recreate everything from scratch. GridStackOptions.children would then contain the widget list instead.
     * @param saveCB callback for each node -> widget, so application can insert additional data to be saved into the widget data structure.
     * @param column if provided, the grid will be saved for the given column size (IFF we have matching internal saved layout, or current layout).
     * Otherwise it will use the largest possible layout (say 12 even if rendering at 1 column) so we can restore to all layouts.
     * NOTE: if you want to save to currently display layout, pass this.getColumn() as column.
     * NOTE2: nested grids will ALWAYS save to the container size to be in sync with parent.
     * @returns list of widgets or full grid option, including .children list of widgets
     */
    save(saveContent = true, saveGridOpt = false, saveCB = _GridStack.saveCB, column) {
      const list = this.engine.save(saveContent, saveCB, column);
      list.forEach((n3) => {
        if (saveContent && n3.el && !n3.subGrid && !saveCB) {
          const itemContent = n3.el.querySelector(".grid-stack-item-content");
          n3.content = itemContent?.innerHTML;
          if (!n3.content)
            delete n3.content;
        } else {
          if (!saveContent && !saveCB) {
            delete n3.content;
          }
          if (n3.subGrid?.el) {
            const column2 = n3.w || n3.subGrid.getColumn();
            const listOrOpt = n3.subGrid.save(saveContent, saveGridOpt, saveCB, column2);
            n3.subGridOpts = saveGridOpt ? listOrOpt : { children: listOrOpt };
            delete n3.subGrid;
          }
        }
        delete n3.el;
      });
      if (saveGridOpt) {
        const o4 = Utils.cloneDeep(this.opts);
        if (o4.marginBottom === o4.marginTop && o4.marginRight === o4.marginLeft && o4.marginTop === o4.marginRight) {
          o4.margin = o4.marginTop;
          delete o4.marginTop;
          delete o4.marginRight;
          delete o4.marginBottom;
          delete o4.marginLeft;
        }
        if (o4.rtl === (this.el.style.direction === "rtl")) {
          o4.rtl = "auto";
        }
        if (this._isAutoCellHeight) {
          o4.cellHeight = "auto";
        }
        if (this._autoColumn) {
          o4.column = "auto";
        }
        const origShow = o4._alwaysShowResizeHandle;
        delete o4._alwaysShowResizeHandle;
        if (origShow !== void 0) {
          o4.alwaysShowResizeHandle = origShow;
        } else {
          delete o4.alwaysShowResizeHandle;
        }
        Utils.removeInternalAndSame(o4, gridDefaults);
        o4.children = list;
        return o4;
      }
      return list;
    }
    /**
     * Load widgets from a list. This will call update() on each (matching by id) or add/remove widgets that are not there.
     * Used to restore a grid layout for a saved layout list (see `save()`).
     *
     * @param items list of widgets definition to update/create
     * @param addRemove boolean (default true) or callback method can be passed to control if and how missing widgets can be added/removed, giving
     * the user control of insertion.
     * @returns the grid instance for chaining
     *
     * @example
     * // Basic usage with saved layout
     * const savedLayout = grid.save(); // Save current layout
     * // ... later restore it
     * grid.load(savedLayout);
     *
     * // Load with custom add/remove callback
     * grid.load(layout, (items, grid, add) => {
     *   if (add) {
     *     // Custom logic for adding new widgets
     *     items.forEach(item => {
     *       const el = document.createElement('div');
     *       el.innerHTML = item.content || '';
     *       grid.addWidget(el, item);
     *     });
     *   } else {
     *     // Custom logic for removing widgets
     *     items.forEach(item => grid.removeWidget(item.el));
     *   }
     * });
     *
     * // Load without adding/removing missing widgets
     * grid.load(layout, false);
     *
     * @see {@link http://gridstackjs.com/demo/serialization.html} for complete example
     */
    load(items, addRemove = _GridStack.addRemoveCB || true) {
      items.forEach((n3) => {
        n3.w = n3.w || n3.minW || 1;
        n3.h = n3.h || n3.minH || 1;
      });
      items = Utils.sort(items);
      this.engine.skipCacheUpdate = this._ignoreLayoutsNodeChange = true;
      let maxColumn = 0;
      items.forEach((n3) => {
        maxColumn = Math.max(maxColumn, (n3.x || 0) + n3.w);
      });
      if (maxColumn > this.engine.defaultColumn)
        this.engine.defaultColumn = maxColumn;
      const column = this.getColumn();
      if (maxColumn > column) {
        if (this.engine.nodes.length === 0 && this.responseLayout) {
          this.engine.nodes = items;
          this.engine.columnChanged(maxColumn, column, this.responseLayout);
          items = this.engine.nodes;
          this.engine.nodes = [];
          delete this.responseLayout;
        } else
          this.engine.cacheLayout(items, maxColumn, true);
      }
      const prevCB = _GridStack.addRemoveCB;
      if (typeof addRemove === "function")
        _GridStack.addRemoveCB = addRemove;
      const removed = [];
      this.batchUpdate();
      const blank = !this.engine.nodes.length;
      const noAnim = blank && this.opts.animate;
      if (noAnim)
        this.setAnimation(false);
      if (!blank && addRemove) {
        const copyNodes = [...this.engine.nodes];
        copyNodes.forEach((n3) => {
          if (!n3.id)
            return;
          const item = Utils.find(items, n3.id);
          if (!item) {
            if (_GridStack.addRemoveCB)
              _GridStack.addRemoveCB(this.el, n3, false, false);
            removed.push(n3);
            this.removeWidget(n3.el, true, false);
          }
        });
      }
      this.engine._loading = true;
      const updateNodes = [];
      this.engine.nodes = this.engine.nodes.filter((n3) => {
        if (Utils.find(items, n3.id)) {
          updateNodes.push(n3);
          return false;
        }
        return true;
      });
      items.forEach((w5) => {
        const item = Utils.find(updateNodes, w5.id);
        if (item) {
          if (Utils.shouldSizeToContent(item))
            w5.h = item.h;
          this.engine.nodeBoundFix(w5);
          if (w5.autoPosition || w5.x === void 0 || w5.y === void 0) {
            w5.w = w5.w || item.w;
            w5.h = w5.h || item.h;
            this.engine.findEmptyPosition(w5);
          }
          this.engine.nodes.push(item);
          if (Utils.samePos(item, w5) && this.engine.nodes.length > 1) {
            this.moveNode(item, { ...w5, forceCollide: true });
            Utils.copyPos(w5, item);
          }
          this.update(item.el, w5);
          if (w5.subGridOpts?.children) {
            const sub = item.el.querySelector(".grid-stack");
            if (sub && sub.gridstack) {
              sub.gridstack.load(w5.subGridOpts.children);
            }
          }
        } else if (addRemove) {
          this.addWidget(w5);
        }
      });
      delete this.engine._loading;
      this.engine.removedNodes = removed;
      this.batchUpdate(false);
      delete this._ignoreLayoutsNodeChange;
      delete this.engine.skipCacheUpdate;
      prevCB ? _GridStack.addRemoveCB = prevCB : delete _GridStack.addRemoveCB;
      if (noAnim)
        this.setAnimation(true, true);
      return this;
    }
    /**
     * use before calling a bunch of `addWidget()` to prevent un-necessary relayouts in between (more efficient)
     * and get a single event callback. You will see no changes until `batchUpdate(false)` is called.
     */
    batchUpdate(flag = true) {
      this.engine.batchUpdate(flag);
      if (!flag) {
        this._updateContainerHeight();
        this._triggerRemoveEvent();
        this._triggerAddEvent();
        this._triggerChangeEvent();
      }
      return this;
    }
    /**
     * Gets the current cell height in pixels. This takes into account the unit type and converts to pixels if necessary.
     *
     * @param forcePixel if true, forces conversion to pixels even when cellHeight is specified in other units
     * @returns the cell height in pixels
     *
     * @example
     * const height = grid.getCellHeight();
     * console.log('Cell height:', height, 'px');
     *
     * // Force pixel conversion
     * const pixelHeight = grid.getCellHeight(true);
     */
    getCellHeight(forcePixel = false) {
      if (this.opts.cellHeight && this.opts.cellHeight !== "auto" && (!forcePixel || !this.opts.cellHeightUnit || this.opts.cellHeightUnit === "px")) {
        return this.opts.cellHeight;
      }
      if (this.opts.cellHeightUnit === "rem") {
        return this.opts.cellHeight * parseFloat(getComputedStyle(document.documentElement).fontSize);
      }
      if (this.opts.cellHeightUnit === "em") {
        return this.opts.cellHeight * parseFloat(getComputedStyle(this.el).fontSize);
      }
      if (this.opts.cellHeightUnit === "cm") {
        return this.opts.cellHeight * (96 / 2.54);
      }
      if (this.opts.cellHeightUnit === "mm") {
        return this.opts.cellHeight * (96 / 2.54) / 10;
      }
      const el = this.el.querySelector("." + this.opts.itemClass);
      if (el) {
        const h5 = Utils.toNumber(el.getAttribute("gs-h")) || 1;
        return Math.round(el.offsetHeight / h5);
      }
      const rows2 = parseInt(this.el.getAttribute("gs-current-row"));
      return rows2 ? Math.round(this.el.getBoundingClientRect().height / rows2) : this.opts.cellHeight;
    }
    /**
     * Update current cell height - see `GridStackOptions.cellHeight` for format by updating eh Browser CSS variable.
     *
     * @param val the cell height. Options:
     *   - `undefined`: cells content will be made square (match width minus margin)
     *   - `0`: the CSS will be generated by the application instead
     *   - number: height in pixels
     *   - string: height with units (e.g., '70px', '5rem', '2em')
     * @returns the grid instance for chaining
     *
     * @example
     * grid.cellHeight(100);     // 100px height
     * grid.cellHeight('70px');  // explicit pixel height
     * grid.cellHeight('5rem');  // relative to root font size
     * grid.cellHeight(grid.cellWidth() * 1.2); // aspect ratio
     * grid.cellHeight('auto');  // auto-size based on content
     */
    cellHeight(val) {
      if (val !== void 0) {
        if (this._isAutoCellHeight !== (val === "auto")) {
          this._isAutoCellHeight = val === "auto";
          this._updateResizeEvent();
        }
      }
      if (val === "initial" || val === "auto") {
        val = void 0;
      }
      if (val === void 0) {
        const marginDiff = -this.opts.marginRight - this.opts.marginLeft + this.opts.marginTop + this.opts.marginBottom;
        val = this.cellWidth() + marginDiff;
      }
      const data = Utils.parseHeight(val);
      if (this.opts.cellHeightUnit === data.unit && this.opts.cellHeight === data.h) {
        return this;
      }
      this.opts.cellHeightUnit = data.unit;
      this.opts.cellHeight = data.h;
      this.el.style.setProperty("--gs-cell-height", `${this.opts.cellHeight}${this.opts.cellHeightUnit}`);
      this._updateContainerHeight();
      this.resizeToContentCheck();
      return this;
    }
    /** Gets current cell width. */
    /**
     * Gets the current cell width in pixels. This is calculated based on the grid container width divided by the number of columns.
     *
     * @returns the cell width in pixels
     *
     * @example
     * const width = grid.cellWidth();
     * console.log('Cell width:', width, 'px');
     *
     * // Use cell width to calculate widget dimensions
     * const widgetWidth = width * 3; // For a 3-column wide widget
     */
    cellWidth() {
      return this._widthOrContainer() / this.getColumn();
    }
    /** return our expected width (or parent) , and optionally of window for dynamic column check */
    _widthOrContainer(forBreakpoint = false) {
      return forBreakpoint && this.opts.columnOpts?.breakpointForWindow ? window.innerWidth : this.el.clientWidth || this.el.parentElement.clientWidth || window.innerWidth;
    }
    /** checks for dynamic column count for our current size, returning true if changed */
    checkDynamicColumn() {
      const resp = this.opts.columnOpts;
      if (!resp || !resp.columnWidth && !resp.breakpoints?.length)
        return false;
      const column = this.getColumn();
      let newColumn = column;
      const w5 = this._widthOrContainer(true);
      if (resp.columnWidth) {
        newColumn = Math.min(Math.round(w5 / resp.columnWidth) || 1, resp.columnMax);
      } else {
        newColumn = resp.columnMax;
        let i4 = 0;
        while (i4 < resp.breakpoints.length && w5 <= resp.breakpoints[i4].w) {
          newColumn = resp.breakpoints[i4++].c || column;
        }
      }
      if (newColumn !== column) {
        const bk = resp.breakpoints?.find((b4) => b4.c === newColumn);
        this.column(newColumn, bk?.layout || resp.layout);
        return true;
      }
      return false;
    }
    /**
     * Re-layout grid items to reclaim any empty space. This is useful after removing widgets
     * or when you want to optimize the layout.
     *
     * @param layout layout type. Options:
     *   - 'compact' (default): might re-order items to fill any empty space
     *   - 'list': keep the widget left->right order the same, even if that means leaving an empty slot if things don't fit
     * @param doSort re-sort items first based on x,y position. Set to false to do your own sorting ahead (default: true)
     * @returns the grid instance for chaining
     *
     * @example
     * // Compact layout after removing widgets
     * grid.removeWidget('.widget-to-remove');
     * grid.compact();
     *
     * // Use list layout (preserve order)
     * grid.compact('list');
     *
     * // Compact without sorting first
     * grid.compact('compact', false);
     */
    compact(layout = "compact", doSort = true) {
      this.engine.compact(layout, doSort);
      this._triggerChangeEvent();
      return this;
    }
    /**
     * Set the number of columns in the grid. Will update existing widgets to conform to new number of columns,
     * as well as cache the original layout so you can revert back to previous positions without loss.
     *
     * Requires `gridstack-extra.css` or `gridstack-extra.min.css` for [2-11] columns,
     * else you will need to generate correct CSS.
     * See: https://github.com/gridstack/gridstack.js#change-grid-columns
     *
     * @param column Integer > 0 (default 12)
     * @param layout specify the type of re-layout that will happen. Options:
     *   - 'moveScale' (default): scale widget positions and sizes
     *   - 'move': keep widget sizes, only move positions
     *   - 'scale': keep widget positions, only scale sizes
     *   - 'none': don't change widget positions or sizes
     *   Note: items will never be outside of the current column boundaries.
     *   Ignored for `column=1` as we always want to vertically stack.
     * @returns the grid instance for chaining
     *
     * @example
     * // Change to 6 columns with default scaling
     * grid.column(6);
     *
     * // Change to 4 columns, only move positions
     * grid.column(4, 'move');
     *
     * // Single column layout (vertical stack)
     * grid.column(1);
     */
    column(column, layout = "moveScale") {
      if (!column || column < 1 || this.opts.column === column)
        return this;
      const oldColumn = this.getColumn();
      this.opts.column = column;
      if (!this.engine) {
        this.responseLayout = layout;
        return this;
      }
      this.engine.column = column;
      this.el.classList.remove("gs-" + oldColumn);
      this._updateColumnVar();
      this.engine.columnChanged(oldColumn, column, layout);
      if (this._isAutoCellHeight)
        this.cellHeight();
      this.resizeToContentCheck(true);
      this._ignoreLayoutsNodeChange = true;
      this._triggerChangeEvent();
      delete this._ignoreLayoutsNodeChange;
      return this;
    }
    /**
     * Get the number of columns in the grid (default 12).
     *
     * @returns the current number of columns in the grid
     *
     * @example
     * const columnCount = grid.getColumn(); // returns 12 by default
     */
    getColumn() {
      return this.opts.column;
    }
    /**
     * Returns an array of grid HTML elements (no placeholder) - used to iterate through our children in DOM order.
     * This method excludes placeholder elements and returns only actual grid items.
     *
     * @returns array of GridItemHTMLElement instances representing all grid items
     *
     * @example
     * const items = grid.getGridItems();
     * items.forEach(item => {
     *   console.log('Item ID:', item.gridstackNode.id);
     * });
     */
    getGridItems() {
      return Array.from(this.el.children).filter((el) => el.matches("." + this.opts.itemClass) && !el.matches("." + this.opts.placeholderClass));
    }
    /**
     * Returns true if change callbacks should be ignored due to column change, sizeToContent, loading, etc.
     * This is useful for callers who want to implement dirty flag functionality.
     *
     * @returns true if change callbacks are currently being ignored
     *
     * @example
     * if (!grid.isIgnoreChangeCB()) {
     *   // Process the change event
     *   console.log('Grid layout changed');
     * }
     */
    isIgnoreChangeCB() {
      return this._ignoreLayoutsNodeChange;
    }
    /**
     * Destroys a grid instance. DO NOT CALL any methods or access any vars after this as it will free up members.
     * @param removeDOM if `false` grid and items HTML elements will not be removed from the DOM (Optional. Default `true`).
     */
    destroy(removeDOM = true) {
      if (!this.el)
        return;
      this.offAll();
      this._updateResizeEvent(true);
      this.setStatic(true, false);
      this.setAnimation(false);
      if (!removeDOM) {
        this.removeAll(removeDOM);
        this.el.removeAttribute("gs-current-row");
      } else {
        this.el.parentNode.removeChild(this.el);
      }
      if (this.parentGridNode)
        delete this.parentGridNode.subGrid;
      delete this.parentGridNode;
      delete this.opts;
      delete this._placeholder?.gridstackNode;
      delete this._placeholder;
      delete this.engine;
      delete this.el.gridstack;
      delete this.el;
      return this;
    }
    /**
     * Enable/disable floating widgets (default: `false`). When enabled, widgets can float up to fill empty spaces.
     * See [example](http://gridstackjs.com/demo/float.html)
     *
     * @param val true to enable floating, false to disable
     * @returns the grid instance for chaining
     *
     * @example
     * grid.float(true);  // Enable floating
     * grid.float(false); // Disable floating (default)
     */
    float(val) {
      if (this.opts.float !== val) {
        this.opts.float = this.engine.float = val;
        this._triggerChangeEvent();
      }
      return this;
    }
    /**
     * Get the current float mode setting.
     *
     * @returns true if floating is enabled, false otherwise
     *
     * @example
     * const isFloating = grid.getFloat();
     * console.log('Floating enabled:', isFloating);
     */
    getFloat() {
      return this.engine.float;
    }
    /**
     * Get the position of the cell under a pixel on screen.
     * @param position the position of the pixel to resolve in
     * absolute coordinates, as an object with top and left properties
     * @param useDocRelative if true, value will be based on document position vs parent position (Optional. Default false).
     * Useful when grid is within `position: relative` element
     *
     * Returns an object with properties `x` and `y` i.e. the column and row in the grid.
     */
    getCellFromPixel(position, useDocRelative = false) {
      const box = this.el.getBoundingClientRect();
      let containerPos;
      if (useDocRelative) {
        containerPos = { top: box.top + document.documentElement.scrollTop, left: box.left };
      } else {
        containerPos = { top: this.el.offsetTop, left: this.el.offsetLeft };
      }
      const relativeLeft = position.left - containerPos.left;
      const relativeTop = position.top - containerPos.top;
      const columnWidth = box.width / this.getColumn();
      const rowHeight = box.height / parseInt(this.el.getAttribute("gs-current-row"));
      return { x: Math.floor(relativeLeft / columnWidth), y: Math.floor(relativeTop / rowHeight) };
    }
    /**
     * Returns the current number of rows, which will be at least `minRow` if set.
     * The row count is based on the highest positioned widget in the grid.
     *
     * @returns the current number of rows in the grid
     *
     * @example
     * const rowCount = grid.getRow();
     * console.log('Grid has', rowCount, 'rows');
     */
    getRow() {
      return Math.max(this.engine.getRow(), this.opts.minRow || 0);
    }
    /**
     * Checks if the specified rectangular area is empty (no widgets occupy any part of it).
     *
     * @param x the x coordinate (column) of the area to check
     * @param y the y coordinate (row) of the area to check
     * @param w the width in columns of the area to check
     * @param h the height in rows of the area to check
     * @returns true if the area is completely empty, false if any widget overlaps
     *
     * @example
     * // Check if a 2x2 area at position (1,1) is empty
     * if (grid.isAreaEmpty(1, 1, 2, 2)) {
     *   console.log('Area is available for placement');
     * }
     */
    isAreaEmpty(x4, y5, w5, h5) {
      return this.engine.isAreaEmpty(x4, y5, w5, h5);
    }
    /**
     * If you add elements to your grid by hand (or have some framework creating DOM), you have to tell gridstack afterwards to make them widgets.
     * If you want gridstack to add the elements for you, use `addWidget()` instead.
     * Makes the given element a widget and returns it.
     *
     * @param els widget or single selector to convert.
     * @param options widget definition to use instead of reading attributes or using default sizing values
     * @returns the converted GridItemHTMLElement
     *
     * @example
     * const grid = GridStack.init();
     *
     * // Create HTML content manually, possibly looking like:
     * // <div id="item-1" gs-x="0" gs-y="0" gs-w="3" gs-h="2"></div>
     * grid.el.innerHTML = '<div id="item-1" gs-w="3"></div><div id="item-2"></div>';
     *
     * // Convert existing elements to widgets
     * grid.makeWidget('#item-1'); // Uses gs-* attributes from DOM
     * grid.makeWidget('#item-2', {w: 2, h: 1, content: 'Hello World'});
     *
     * // Or pass DOM element directly
     * const element = document.getElementById('item-3');
     * grid.makeWidget(element, {x: 0, y: 1, w: 4, h: 2});
     */
    makeWidget(els, options) {
      const el = _GridStack.getElement(els);
      if (!el || el.gridstackNode)
        return el;
      if (!el.parentElement)
        this.el.appendChild(el);
      this._prepareElement(el, true, options);
      const node = el.gridstackNode;
      this._updateContainerHeight();
      if (node.subGridOpts) {
        this.makeSubGrid(el, node.subGridOpts, void 0, false);
      }
      let resetIgnoreLayoutsNodeChange;
      if (this.opts.column === 1 && !this._ignoreLayoutsNodeChange) {
        resetIgnoreLayoutsNodeChange = this._ignoreLayoutsNodeChange = true;
      }
      this._triggerAddEvent();
      this._triggerChangeEvent();
      if (resetIgnoreLayoutsNodeChange)
        delete this._ignoreLayoutsNodeChange;
      return el;
    }
    on(name, callback) {
      if (name.indexOf(" ") !== -1) {
        const names = name.split(" ");
        names.forEach((name2) => this.on(name2, callback));
        return this;
      }
      if (name === "change" || name === "added" || name === "removed" || name === "enable" || name === "disable") {
        const noData = name === "enable" || name === "disable";
        if (noData) {
          this._gsEventHandler[name] = (event) => callback(event);
        } else {
          this._gsEventHandler[name] = (event) => {
            if (event.detail)
              callback(event, event.detail);
          };
        }
        this.el.addEventListener(name, this._gsEventHandler[name]);
      } else if (name === "drag" || name === "dragstart" || name === "dragstop" || name === "resizestart" || name === "resize" || name === "resizestop" || name === "dropped" || name === "resizecontent") {
        this._gsEventHandler[name] = callback;
      } else {
        console.error("GridStack.on(" + name + ") event not supported");
      }
      return this;
    }
    /**
     * unsubscribe from the 'on' event GridStackEvent
     * @param name of the event (see possible values) or list of names space separated
     */
    off(name) {
      if (name.indexOf(" ") !== -1) {
        const names = name.split(" ");
        names.forEach((name2) => this.off(name2));
        return this;
      }
      if (name === "change" || name === "added" || name === "removed" || name === "enable" || name === "disable") {
        if (this._gsEventHandler[name]) {
          this.el.removeEventListener(name, this._gsEventHandler[name]);
        }
      }
      delete this._gsEventHandler[name];
      return this;
    }
    /**
     * Remove all event handlers from the grid. This is useful for cleanup when destroying a grid.
     *
     * @returns the grid instance for chaining
     *
     * @example
     * grid.offAll(); // Remove all event listeners
     */
    offAll() {
      Object.keys(this._gsEventHandler).forEach((key) => this.off(key));
      return this;
    }
    /**
     * Removes widget from the grid.
     * @param el  widget or selector to modify
     * @param removeDOM if `false` DOM element won't be removed from the tree (Default? true).
     * @param triggerEvent if `false` (quiet mode) element will not be added to removed list and no 'removed' callbacks will be called (Default? true).
     */
    removeWidget(els, removeDOM = true, triggerEvent = true) {
      if (!els) {
        console.error("Error: GridStack.removeWidget(undefined) called");
        return this;
      }
      _GridStack.getElements(els).forEach((el) => {
        if (el.parentElement && el.parentElement !== this.el)
          return;
        let node = el.gridstackNode;
        if (!node) {
          node = this.engine.nodes.find((n3) => el === n3.el);
        }
        if (!node)
          return;
        if (removeDOM && _GridStack.addRemoveCB) {
          _GridStack.addRemoveCB(this.el, node, false, false);
        }
        delete el.gridstackNode;
        this._removeDD(el);
        this.engine.removeNode(node, removeDOM, triggerEvent);
        if (removeDOM && el.parentElement) {
          el.remove();
        }
      });
      if (triggerEvent) {
        this._triggerRemoveEvent();
        this._triggerChangeEvent();
      }
      return this;
    }
    /**
     * Removes all widgets from the grid.
     * @param removeDOM if `false` DOM elements won't be removed from the tree (Default? `true`).
     * @param triggerEvent if `false` (quiet mode) element will not be added to removed list and no 'removed' callbacks will be called (Default? true).
     */
    removeAll(removeDOM = true, triggerEvent = true) {
      this.engine.nodes.forEach((n3) => {
        if (removeDOM && _GridStack.addRemoveCB) {
          _GridStack.addRemoveCB(this.el, n3, false, false);
        }
        delete n3.el.gridstackNode;
        if (!this.opts.staticGrid)
          this._removeDD(n3.el);
      });
      this.engine.removeAll(removeDOM, triggerEvent);
      if (triggerEvent)
        this._triggerRemoveEvent();
      return this;
    }
    /**
     * Toggle the grid animation state.  Toggles the `grid-stack-animate` class.
     * @param doAnimate if true the grid will animate.
     * @param delay if true setting will be set on next event loop.
     */
    setAnimation(doAnimate = this.opts.animate, delay) {
      if (delay) {
        setTimeout(() => {
          if (this.opts)
            this.setAnimation(doAnimate);
        });
      } else if (doAnimate) {
        this.el.classList.add("grid-stack-animate");
      } else {
        this.el.classList.remove("grid-stack-animate");
      }
      this.opts.animate = doAnimate;
      return this;
    }
    /** @internal */
    hasAnimationCSS() {
      return this.el.classList.contains("grid-stack-animate");
    }
    /**
     * Toggle the grid static state, which permanently removes/add Drag&Drop support, unlike disable()/enable() that just turns it off/on.
     * Also toggle the grid-stack-static class.
     * @param val if true the grid become static.
     * @param updateClass true (default) if css class gets updated
     * @param recurse true (default) if sub-grids also get updated
     */
    setStatic(val, updateClass = true, recurse = true) {
      if (!!this.opts.staticGrid === val)
        return this;
      val ? this.opts.staticGrid = true : delete this.opts.staticGrid;
      this._setupRemoveDrop();
      this._setupAcceptWidget();
      this.engine.nodes.forEach((n3) => {
        this.prepareDragDrop(n3.el);
        if (n3.subGrid && recurse)
          n3.subGrid.setStatic(val, updateClass, recurse);
      });
      if (updateClass) {
        this._setStaticClass();
      }
      return this;
    }
    /**
     * Updates the passed in options on the grid (similar to update(widget) for for the grid options).
     * @param options PARTIAL grid options to update - only items specified will be updated.
     * NOTE: not all options updating are currently supported (lot of code, unlikely to change)
     */
    updateOptions(o4) {
      const opts = this.opts;
      if (o4 === opts)
        return this;
      if (o4.acceptWidgets !== void 0) {
        opts.acceptWidgets = o4.acceptWidgets;
        this._setupAcceptWidget();
      }
      if (o4.animate !== void 0)
        this.setAnimation(o4.animate);
      if (o4.cellHeight)
        this.cellHeight(o4.cellHeight);
      if (o4.class !== void 0 && o4.class !== opts.class) {
        if (opts.class)
          this.el.classList.remove(opts.class);
        if (o4.class)
          this.el.classList.add(o4.class);
      }
      if (o4.columnOpts) {
        const hadColumnOpts = !!this.opts.columnOpts;
        this.opts.columnOpts = o4.columnOpts;
        if (hadColumnOpts !== !!this.opts.columnOpts)
          this._updateResizeEvent();
        this.checkDynamicColumn();
      } else if (o4.columnOpts === null && this.opts.columnOpts) {
        delete this.opts.columnOpts;
        this._updateResizeEvent();
      } else if (typeof o4.column === "number")
        this.column(o4.column);
      if (o4.margin !== void 0)
        this.margin(o4.margin);
      if (o4.staticGrid !== void 0)
        this.setStatic(o4.staticGrid);
      if (o4.disableDrag !== void 0 && !o4.staticGrid)
        this.enableMove(!o4.disableDrag);
      if (o4.disableResize !== void 0 && !o4.staticGrid)
        this.enableResize(!o4.disableResize);
      if (o4.float !== void 0)
        this.float(o4.float);
      if (o4.row !== void 0) {
        opts.minRow = opts.maxRow = opts.row = o4.row;
        this._updateContainerHeight();
      } else {
        if (o4.minRow !== void 0) {
          opts.minRow = o4.minRow;
          this._updateContainerHeight();
        }
        if (o4.maxRow !== void 0)
          opts.maxRow = this.engine.maxRow = o4.maxRow;
      }
      if (o4.lazyLoad !== void 0)
        opts.lazyLoad = o4.lazyLoad;
      if (o4.children?.length)
        this.load(o4.children);
      return this;
    }
    /**
     * Updates widget position/size and other info. This is used to change widget properties after creation.
     * Can update position, size, content, and other widget properties.
     *
     * Note: If you need to call this on all nodes, use load() instead which will update what changed.
     * Setting the same x,y for multiple items will be indeterministic and likely unwanted.
     *
     * @param els widget element(s) or selector to modify
     * @param opt new widget options (x,y,w,h, etc.). Only those set will be updated.
     * @returns the grid instance for chaining
     *
     * @example
     * // Update widget size and position
     * grid.update('.my-widget', { x: 2, y: 1, w: 3, h: 2 });
     *
     * // Update widget content
     * grid.update(widget, { content: '<p>New content</p>' });
     *
     * // Update multiple properties
     * grid.update('#my-widget', {
     *   w: 4,
     *   h: 3,
     *   noResize: true,
     *   locked: true
     * });
     */
    update(els, opt) {
      _GridStack.getElements(els).forEach((el) => {
        const n3 = el?.gridstackNode;
        if (!n3)
          return;
        const w5 = { ...Utils.copyPos({}, n3), ...Utils.cloneDeep(opt) };
        this.engine.nodeBoundFix(w5);
        delete w5.autoPosition;
        const keys = ["x", "y", "w", "h"];
        let m5;
        if (keys.some((k4) => w5[k4] !== void 0 && w5[k4] !== n3[k4])) {
          m5 = {};
          keys.forEach((k4) => {
            m5[k4] = w5[k4] !== void 0 ? w5[k4] : n3[k4];
            delete w5[k4];
          });
        }
        if (!m5 && (w5.minW || w5.minH || w5.maxW || w5.maxH)) {
          m5 = {};
        }
        if (w5.content !== void 0) {
          const itemContent = el.querySelector(".grid-stack-item-content");
          if (itemContent && itemContent.textContent !== w5.content) {
            n3.content = w5.content;
            _GridStack.renderCB(itemContent, w5);
            if (n3.subGrid?.el) {
              itemContent.appendChild(n3.subGrid.el);
              n3.subGrid._updateContainerHeight();
            }
          }
          delete w5.content;
        }
        let changed = false;
        let ddChanged = false;
        for (const key in w5) {
          if (key[0] !== "_" && n3[key] !== w5[key]) {
            n3[key] = w5[key];
            changed = true;
            ddChanged = ddChanged || !this.opts.staticGrid && (key === "noResize" || key === "noMove" || key === "locked");
          }
        }
        Utils.sanitizeMinMax(n3);
        if (m5) {
          const widthChanged = m5.w !== void 0 && m5.w !== n3.w;
          this.moveNode(n3, m5);
          if (widthChanged && n3.subGrid) {
            n3.subGrid.onResize(this.hasAnimationCSS() ? n3.w : void 0);
          } else {
            this.resizeToContentCheck(widthChanged, n3);
          }
          delete n3._orig;
        }
        if (m5 || changed) {
          this._writeAttr(el, n3);
        }
        if (ddChanged) {
          this.prepareDragDrop(n3.el);
        }
        if (_GridStack.updateCB)
          _GridStack.updateCB(n3);
      });
      return this;
    }
    moveNode(n3, m5) {
      const wasUpdating = n3._updating;
      if (!wasUpdating)
        this.engine.cleanNodes().beginUpdate(n3);
      this.engine.moveNode(n3, m5);
      this._updateContainerHeight();
      if (!wasUpdating) {
        this._triggerChangeEvent();
        this.engine.endUpdate();
      }
    }
    /**
     * Updates widget height to match the content height to avoid vertical scrollbars or dead space.
     * This automatically adjusts the widget height based on its content size.
     *
     * Note: This assumes only 1 child under resizeToContentParent='.grid-stack-item-content'
     * (sized to gridItem minus padding) that represents the entire content size.
     *
     * @param el the grid item element to resize
     *
     * @example
     * // Resize a widget to fit its content
     * const widget = document.querySelector('.grid-stack-item');
     * grid.resizeToContent(widget);
     *
     * // This is commonly used with dynamic content:
     * widget.querySelector('.content').innerHTML = 'New longer content...';
     * grid.resizeToContent(widget);
     */
    resizeToContent(el) {
      if (!el)
        return;
      el.classList.remove("size-to-content-max");
      if (!el.clientHeight)
        return;
      const n3 = el.gridstackNode;
      if (!n3)
        return;
      const grid = n3.grid;
      if (!grid || el.parentElement !== grid.el)
        return;
      const cell = grid.getCellHeight(true);
      if (!cell)
        return;
      let height = n3.h ? n3.h * cell : el.clientHeight;
      let item;
      if (n3.resizeToContentParent)
        item = el.querySelector(n3.resizeToContentParent);
      if (!item)
        item = el.querySelector(_GridStack.resizeToContentParent);
      if (!item)
        return;
      const padding = el.clientHeight - item.clientHeight;
      const itemH = n3.h ? n3.h * cell - padding : item.clientHeight;
      let wantedH;
      if (n3.subGrid) {
        wantedH = n3.subGrid.getRow() * n3.subGrid.getCellHeight(true);
        const subRec = n3.subGrid.el.getBoundingClientRect();
        const parentRec = el.getBoundingClientRect();
        wantedH += subRec.top - parentRec.top;
      } else if (n3.subGridOpts?.children?.length) {
        return;
      } else {
        const child = item.firstElementChild;
        if (!child) {
          console.error(`Error: GridStack.resizeToContent() widget id:${n3.id} '${_GridStack.resizeToContentParent}'.firstElementChild is null, make sure to have a div like container. Skipping sizing.`);
          return;
        }
        wantedH = child.getBoundingClientRect().height || itemH;
      }
      if (itemH === wantedH)
        return;
      height += wantedH - itemH;
      let h5 = Math.ceil(height / cell);
      const softMax = Number.isInteger(n3.sizeToContent) ? n3.sizeToContent : 0;
      if (softMax && h5 > softMax) {
        h5 = softMax;
        el.classList.add("size-to-content-max");
      }
      if (n3.minH && h5 < n3.minH)
        h5 = n3.minH;
      else if (n3.maxH && h5 > n3.maxH)
        h5 = n3.maxH;
      if (h5 !== n3.h) {
        grid._ignoreLayoutsNodeChange = true;
        grid.moveNode(n3, { h: h5 });
        delete grid._ignoreLayoutsNodeChange;
      }
    }
    /** call the user resize (so they can do extra work) else our build in version */
    resizeToContentCBCheck(el) {
      if (_GridStack.resizeToContentCB)
        _GridStack.resizeToContentCB(el);
      else
        this.resizeToContent(el);
    }
    /**
     * Rotate widgets by swapping their width and height. This is typically called when the user presses 'r' during dragging.
     * The rotation swaps the w/h dimensions and adjusts min/max constraints accordingly.
     *
     * @param els widget element(s) or selector to rotate
     * @param relative optional pixel coordinate relative to upper/left corner to rotate around (keeps that cell under cursor)
     * @returns the grid instance for chaining
     *
     * @example
     * // Rotate a specific widget
     * grid.rotate('.my-widget');
     *
     * // Rotate with relative positioning during drag
     * grid.rotate(widget, { left: 50, top: 30 });
     */
    rotate(els, relative) {
      _GridStack.getElements(els).forEach((el) => {
        const n3 = el.gridstackNode;
        if (!Utils.canBeRotated(n3))
          return;
        const rot = { w: n3.h, h: n3.w, minH: n3.minW, minW: n3.minH, maxH: n3.maxW, maxW: n3.maxH };
        if (relative) {
          const pivotX = relative.left > 0 ? Math.floor(relative.left / this.cellWidth()) : 0;
          const pivotY = relative.top > 0 ? Math.floor(relative.top / this.opts.cellHeight) : 0;
          rot.x = n3.x + pivotX - (n3.h - (pivotY + 1));
          rot.y = n3.y + pivotY - pivotX;
        }
        Object.keys(rot).forEach((k4) => {
          if (rot[k4] === void 0)
            delete rot[k4];
        });
        const _orig = n3._orig;
        this.update(el, rot);
        n3._orig = _orig;
      });
      return this;
    }
    /**
     * Updates the margins which will set all 4 sides at once - see `GridStackOptions.margin` for format options.
     * Supports CSS string format of 1, 2, or 4 values or a single number.
     *
     * @param value margin value - can be:
     *   - Single number: `10` (applies to all sides)
     *   - Two values: `'10px 20px'` (top/bottom, left/right)
     *   - Four values: `'10px 20px 5px 15px'` (top, right, bottom, left)
     * @returns the grid instance for chaining
     *
     * @example
     * grid.margin(10);           // 10px all sides
     * grid.margin('10px 20px');  // 10px top/bottom, 20px left/right
     * grid.margin('5px 10px 15px 20px'); // Different for each side
     */
    margin(value) {
      const isMultiValue = typeof value === "string" && value.split(" ").length > 1;
      if (!isMultiValue) {
        const data = Utils.parseHeight(value);
        if (this.opts.marginUnit === data.unit && this.opts.margin === data.h)
          return;
      }
      this.opts.margin = value;
      this.opts.marginTop = this.opts.marginBottom = this.opts.marginLeft = this.opts.marginRight = void 0;
      this._initMargin();
      return this;
    }
    /**
     * Returns the current margin value as a number (undefined if the 4 sides don't match).
     * This only returns a number if all sides have the same margin value.
     *
     * @returns the margin value in pixels, or undefined if sides have different values
     *
     * @example
     * const margin = grid.getMargin();
     * if (margin !== undefined) {
     *   console.log('Uniform margin:', margin, 'px');
     * } else {
     *   console.log('Margins are different on different sides');
     * }
     */
    getMargin() {
      return this.opts.margin;
    }
    /**
     * Returns true if the height of the grid will be less than the vertical
     * constraint. Always returns true if grid doesn't have height constraint.
     * @param node contains x,y,w,h,auto-position options
     *
     * @example
     * if (grid.willItFit(newWidget)) {
     *   grid.addWidget(newWidget);
     * } else {
     *   alert('Not enough free space to place the widget');
     * }
     */
    willItFit(node) {
      return this.engine.willItFit(node);
    }
    /** @internal */
    _triggerChangeEvent() {
      if (this.engine.batchMode)
        return this;
      const elements = this.engine.getDirtyNodes(true);
      if (elements && elements.length) {
        if (!this._ignoreLayoutsNodeChange) {
          this.engine.layoutsNodesChange(elements);
        }
        this._triggerEvent("change", elements);
      }
      this.engine.saveInitial();
      return this;
    }
    /** @internal */
    _triggerAddEvent() {
      if (this.engine.batchMode)
        return this;
      if (this.engine.addedNodes?.length) {
        if (!this._ignoreLayoutsNodeChange) {
          this.engine.layoutsNodesChange(this.engine.addedNodes);
        }
        this.engine.addedNodes.forEach((n3) => {
          delete n3._dirty;
        });
        const addedNodes = [...this.engine.addedNodes];
        this.engine.addedNodes = [];
        this._triggerEvent("added", addedNodes);
      }
      return this;
    }
    /** @internal */
    _triggerRemoveEvent() {
      if (this.engine.batchMode)
        return this;
      if (this.engine.removedNodes?.length) {
        const removedNodes = [...this.engine.removedNodes];
        this.engine.removedNodes = [];
        this._triggerEvent("removed", removedNodes);
      }
      return this;
    }
    /** @internal */
    _triggerEvent(type, data) {
      const event = data ? new CustomEvent(type, { bubbles: false, detail: data }) : new Event(type);
      let grid = this;
      while (grid.parentGridNode)
        grid = grid.parentGridNode.grid;
      grid.el.dispatchEvent(event);
      return this;
    }
    /** @internal */
    _updateContainerHeight() {
      if (!this.engine || this.engine.batchMode)
        return this;
      const parent = this.parentGridNode;
      let row = this.getRow() + this._extraDragRow;
      const cellHeight = this.opts.cellHeight;
      const unit = this.opts.cellHeightUnit;
      if (!cellHeight)
        return this;
      if (!parent && !this.opts.minRow) {
        const cssMinHeight = Utils.parseHeight(getComputedStyle(this.el)["minHeight"]);
        if (cssMinHeight.h > 0 && cssMinHeight.unit === unit) {
          const minRow = Math.floor(cssMinHeight.h / cellHeight);
          if (row < minRow) {
            row = minRow;
          }
        }
      }
      this.el.setAttribute("gs-current-row", String(row));
      this.el.style.removeProperty("min-height");
      this.el.style.removeProperty("height");
      if (row) {
        this.el.style[parent ? "minHeight" : "height"] = row * cellHeight + unit;
      }
      if (parent && Utils.shouldSizeToContent(parent)) {
        parent.grid.resizeToContentCBCheck(parent.el);
      }
      return this;
    }
    /** @internal */
    _prepareElement(el, triggerAddEvent = false, node) {
      node = node || this._readAttr(el);
      el.gridstackNode = node;
      node.el = el;
      node.grid = this;
      node = this.engine.addNode(node, triggerAddEvent);
      this._writeAttr(el, node);
      el.classList.add(gridDefaults.itemClass, this.opts.itemClass);
      const sizeToContent = Utils.shouldSizeToContent(node);
      sizeToContent ? el.classList.add("size-to-content") : el.classList.remove("size-to-content");
      if (sizeToContent)
        this.resizeToContentCheck(false, node);
      if (!Utils.lazyLoad(node))
        this.prepareDragDrop(node.el);
      return this;
    }
    /** @internal write position CSS vars and x,y,w,h attributes (not used for CSS but by users) back to element */
    _writePosAttr(el, n3) {
      if (!n3._moving && !n3._resizing || this._placeholder === el) {
        const xProp = this.opts.rtl ? "right" : "left";
        el.style.top = n3.y ? n3.y === 1 ? `var(--gs-cell-height)` : `calc(${n3.y} * var(--gs-cell-height))` : null;
        el.style[xProp] = n3.x ? n3.x === 1 ? `var(--gs-column-width)` : `calc(${n3.x} * var(--gs-column-width))` : null;
        el.style.width = n3.w > 1 ? `calc(${n3.w} * var(--gs-column-width))` : null;
        el.style.height = n3.h > 1 ? `calc(${n3.h} * var(--gs-cell-height))` : null;
      }
      el.setAttribute("gs-x", String(n3.x));
      el.setAttribute("gs-y", String(n3.y));
      n3.w > 1 ? el.setAttribute("gs-w", String(n3.w)) : el.removeAttribute("gs-w");
      n3.h > 1 ? el.setAttribute("gs-h", String(n3.h)) : el.removeAttribute("gs-h");
      return this;
    }
    /** @internal call to write any default attributes back to element */
    _writeAttr(el, node) {
      if (!node)
        return this;
      this._writePosAttr(el, node);
      const attrs = {
        // autoPosition: 'gs-auto-position', // no need to write out as already in node and doesn't affect CSS
        noResize: "gs-no-resize",
        noMove: "gs-no-move",
        locked: "gs-locked",
        id: "gs-id",
        sizeToContent: "gs-size-to-content"
      };
      for (const key in attrs) {
        if (node[key]) {
          el.setAttribute(attrs[key], String(node[key]));
        } else {
          el.removeAttribute(attrs[key]);
        }
      }
      return this;
    }
    /** @internal call to read any default attributes from element */
    _readAttr(el, clearDefaultAttr = true) {
      const n3 = {};
      n3.x = Utils.toNumber(el.getAttribute("gs-x"));
      n3.y = Utils.toNumber(el.getAttribute("gs-y"));
      n3.w = Utils.toNumber(el.getAttribute("gs-w"));
      n3.h = Utils.toNumber(el.getAttribute("gs-h"));
      n3.autoPosition = Utils.toBool(el.getAttribute("gs-auto-position"));
      n3.noResize = Utils.toBool(el.getAttribute("gs-no-resize"));
      n3.noMove = Utils.toBool(el.getAttribute("gs-no-move"));
      n3.locked = Utils.toBool(el.getAttribute("gs-locked"));
      const attr = el.getAttribute("gs-size-to-content");
      if (attr) {
        if (attr === "true" || attr === "false")
          n3.sizeToContent = Utils.toBool(attr);
        else
          n3.sizeToContent = parseInt(attr, 10);
      }
      n3.id = el.getAttribute("gs-id");
      n3.maxW = Utils.toNumber(el.getAttribute("gs-max-w"));
      n3.minW = Utils.toNumber(el.getAttribute("gs-min-w"));
      n3.maxH = Utils.toNumber(el.getAttribute("gs-max-h"));
      n3.minH = Utils.toNumber(el.getAttribute("gs-min-h"));
      if (clearDefaultAttr) {
        if (n3.w === 1)
          el.removeAttribute("gs-w");
        if (n3.h === 1)
          el.removeAttribute("gs-h");
        if (n3.maxW)
          el.removeAttribute("gs-max-w");
        if (n3.minW)
          el.removeAttribute("gs-min-w");
        if (n3.maxH)
          el.removeAttribute("gs-max-h");
        if (n3.minH)
          el.removeAttribute("gs-min-h");
      }
      for (const key in n3) {
        if (!n3.hasOwnProperty(key))
          return;
        if (!n3[key] && n3[key] !== 0 && key !== "sizeToContent") {
          delete n3[key];
        }
      }
      return n3;
    }
    /** @internal */
    _setStaticClass() {
      const classes = ["grid-stack-static"];
      if (this.opts.staticGrid) {
        this.el.classList.add(...classes);
        this.el.setAttribute("gs-static", "true");
      } else {
        this.el.classList.remove(...classes);
        this.el.removeAttribute("gs-static");
      }
      return this;
    }
    /**
     * called when we are being resized - check if the one Column Mode needs to be turned on/off
     * and remember the prev columns we used, or get our count from parent, as well as check for cellHeight==='auto' (square)
     * or `sizeToContent` gridItem options.
     */
    onResize(clientWidth = this.el?.clientWidth) {
      if (!clientWidth)
        return;
      if (this.prevWidth === clientWidth)
        return;
      this.prevWidth = clientWidth;
      this.batchUpdate();
      let columnChanged = false;
      if (this._autoColumn && this.parentGridNode) {
        if (this.opts.column !== this.parentGridNode.w) {
          this.column(this.parentGridNode.w, this.opts.layout || "list");
          columnChanged = true;
        }
      } else {
        columnChanged = this.checkDynamicColumn();
      }
      if (this._isAutoCellHeight)
        this.cellHeight();
      this.engine.nodes.forEach((n3) => {
        if (n3.subGrid)
          n3.subGrid.onResize();
      });
      if (!this._skipInitialResize)
        this.resizeToContentCheck(columnChanged);
      delete this._skipInitialResize;
      this.batchUpdate(false);
      return this;
    }
    /** resizes content for given node (or all) if shouldSizeToContent() is true */
    resizeToContentCheck(delay = false, n3 = void 0) {
      if (!this.engine)
        return;
      if (delay && this.hasAnimationCSS())
        return setTimeout(() => this.resizeToContentCheck(false, n3), this.animationDelay);
      if (n3) {
        if (Utils.shouldSizeToContent(n3))
          this.resizeToContentCBCheck(n3.el);
      } else if (this.engine.nodes.some((n4) => Utils.shouldSizeToContent(n4))) {
        const nodes = [...this.engine.nodes];
        this.batchUpdate();
        nodes.forEach((n4) => {
          if (Utils.shouldSizeToContent(n4))
            this.resizeToContentCBCheck(n4.el);
        });
        this._ignoreLayoutsNodeChange = true;
        this.batchUpdate(false);
        this._ignoreLayoutsNodeChange = false;
      }
      if (this._gsEventHandler["resizecontent"])
        this._gsEventHandler["resizecontent"](null, n3 ? [n3] : this.engine.nodes);
    }
    /** add or remove the grid element size event handler */
    _updateResizeEvent(forceRemove = false) {
      const trackSize = !this.parentGridNode && (this._isAutoCellHeight || this.opts.sizeToContent || this.opts.columnOpts || this.engine.nodes.find((n3) => n3.sizeToContent));
      if (!forceRemove && trackSize && !this.resizeObserver) {
        this._sizeThrottle = Utils.throttle(() => this.onResize(), this.opts.cellHeightThrottle);
        this.resizeObserver = new ResizeObserver(() => this._sizeThrottle());
        this.resizeObserver.observe(this.el);
        this._skipInitialResize = true;
      } else if ((forceRemove || !trackSize) && this.resizeObserver) {
        this.resizeObserver.disconnect();
        delete this.resizeObserver;
        delete this._sizeThrottle;
      }
      return this;
    }
    /** @internal convert a potential selector into actual element */
    static getElement(els = ".grid-stack-item") {
      return Utils.getElement(els);
    }
    /** @internal */
    static getElements(els = ".grid-stack-item") {
      return Utils.getElements(els);
    }
    /** @internal */
    static getGridElement(els) {
      return _GridStack.getElement(els);
    }
    /** @internal */
    static getGridElements(els) {
      return Utils.getElements(els);
    }
    /** @internal initialize margin top/bottom/left/right and units */
    _initMargin() {
      let data;
      let margin = 0;
      let margins = [];
      if (typeof this.opts.margin === "string") {
        margins = this.opts.margin.split(" ");
      }
      if (margins.length === 2) {
        this.opts.marginTop = this.opts.marginBottom = margins[0];
        this.opts.marginLeft = this.opts.marginRight = margins[1];
      } else if (margins.length === 4) {
        this.opts.marginTop = margins[0];
        this.opts.marginRight = margins[1];
        this.opts.marginBottom = margins[2];
        this.opts.marginLeft = margins[3];
      } else {
        data = Utils.parseHeight(this.opts.margin);
        this.opts.marginUnit = data.unit;
        margin = this.opts.margin = data.h;
      }
      const keys = ["marginTop", "marginRight", "marginBottom", "marginLeft"];
      keys.forEach((k4) => {
        if (this.opts[k4] === void 0) {
          this.opts[k4] = margin;
        } else {
          data = Utils.parseHeight(this.opts[k4]);
          this.opts[k4] = data.h;
          delete this.opts.margin;
        }
      });
      this.opts.marginUnit = data.unit;
      if (this.opts.marginTop === this.opts.marginBottom && this.opts.marginLeft === this.opts.marginRight && this.opts.marginTop === this.opts.marginRight) {
        this.opts.margin = this.opts.marginTop;
      }
      const style = this.el.style;
      style.setProperty("--gs-item-margin-top", `${this.opts.marginTop}${this.opts.marginUnit}`);
      style.setProperty("--gs-item-margin-bottom", `${this.opts.marginBottom}${this.opts.marginUnit}`);
      style.setProperty("--gs-item-margin-right", `${this.opts.marginRight}${this.opts.marginUnit}`);
      style.setProperty("--gs-item-margin-left", `${this.opts.marginLeft}${this.opts.marginUnit}`);
      return this;
    }
    /* ===========================================================================================
     * drag&drop methods that used to be stubbed out and implemented in dd-gridstack.ts
     * but caused loading issues in prod - see https://github.com/gridstack/gridstack.js/issues/2039
     * ===========================================================================================
     */
    /**
     * Get the global drag & drop implementation instance.
     * This provides access to the underlying drag & drop functionality.
     *
     * @returns the DDGridStack instance used for drag & drop operations
     *
     * @example
     * const dd = GridStack.getDD();
     * // Access drag & drop functionality
     */
    static getDD() {
      return dd;
    }
    /**
     * call to setup dragging in from the outside (say toolbar), by specifying the class selection and options.
     * Called during GridStack.init() as options, but can also be called directly (last param are used) in case the toolbar
     * is dynamically create and needs to be set later.
     * @param dragIn string selector (ex: '.sidebar-item') or list of dom elements
     * @param dragInOptions options - see DDDragOpt. (default: {handle: '.grid-stack-item-content', appendTo: 'body'}
     * @param widgets GridStackWidget def to assign to each element which defines what to create on drop
     * @param root optional root which defaults to document (for shadow dom pass the parent HTMLDocument)
     */
    static setupDragIn(dragIn, dragInOptions, widgets, root = document) {
      if (dragInOptions?.pause !== void 0) {
        DDManager.pauseDrag = dragInOptions.pause;
      }
      dragInOptions = { appendTo: "body", helper: "clone", ...dragInOptions || {} };
      const els = typeof dragIn === "string" ? Utils.getElements(dragIn, root) : dragIn;
      els.forEach((el, i4) => {
        if (!dd.isDraggable(el))
          dd.dragIn(el, dragInOptions);
        if (widgets?.[i4])
          el.gridstackNode = widgets[i4];
      });
    }
    /**
     * Enables/Disables dragging by the user for specific grid elements.
     * For all items and future items, use enableMove() instead. No-op for static grids.
     *
     * Note: If you want to prevent an item from moving due to being pushed around by another
     * during collision, use the 'locked' property instead.
     *
     * @param els widget element(s) or selector to modify
     * @param val if true widget will be draggable, assuming the parent grid isn't noMove or static
     * @returns the grid instance for chaining
     *
     * @example
     * // Make specific widgets draggable
     * grid.movable('.my-widget', true);
     *
     * // Disable dragging for specific widgets
     * grid.movable('#fixed-widget', false);
     */
    movable(els, val) {
      if (this.opts.staticGrid)
        return this;
      _GridStack.getElements(els).forEach((el) => {
        const n3 = el.gridstackNode;
        if (!n3)
          return;
        val ? delete n3.noMove : n3.noMove = true;
        this.prepareDragDrop(n3.el);
      });
      return this;
    }
    /**
     * Enables/Disables user resizing for specific grid elements.
     * For all items and future items, use enableResize() instead. No-op for static grids.
     *
     * @param els widget element(s) or selector to modify
     * @param val if true widget will be resizable, assuming the parent grid isn't noResize or static
     * @returns the grid instance for chaining
     *
     * @example
     * // Make specific widgets resizable
     * grid.resizable('.my-widget', true);
     *
     * // Disable resizing for specific widgets
     * grid.resizable('#fixed-size-widget', false);
     */
    resizable(els, val) {
      if (this.opts.staticGrid)
        return this;
      _GridStack.getElements(els).forEach((el) => {
        const n3 = el.gridstackNode;
        if (!n3)
          return;
        val ? delete n3.noResize : n3.noResize = true;
        this.prepareDragDrop(n3.el);
      });
      return this;
    }
    /**
     * Temporarily disables widgets moving/resizing.
     * If you want a more permanent way (which freezes up resources) use `setStatic(true)` instead.
     *
     * Note: This is a no-op for static grids.
     *
     * This is a shortcut for:
     * ```typescript
     * grid.enableMove(false);
     * grid.enableResize(false);
     * ```
     *
     * @param recurse if true (default), sub-grids also get updated
     * @returns the grid instance for chaining
     *
     * @example
     * // Disable all interactions
     * grid.disable();
     *
     * // Disable only this grid, not sub-grids
     * grid.disable(false);
     */
    disable(recurse = true) {
      if (this.opts.staticGrid)
        return;
      this.enableMove(false, recurse);
      this.enableResize(false, recurse);
      this._triggerEvent("disable");
      return this;
    }
    /**
     * Re-enables widgets moving/resizing - see disable().
     * Note: This is a no-op for static grids.
     *
     * This is a shortcut for:
     * ```typescript
     * grid.enableMove(true);
     * grid.enableResize(true);
     * ```
     *
     * @param recurse if true (default), sub-grids also get updated
     * @returns the grid instance for chaining
     *
     * @example
     * // Re-enable all interactions
     * grid.enable();
     *
     * // Enable only this grid, not sub-grids
     * grid.enable(false);
     */
    enable(recurse = true) {
      if (this.opts.staticGrid)
        return;
      this.enableMove(true, recurse);
      this.enableResize(true, recurse);
      this._triggerEvent("enable");
      return this;
    }
    /**
     * Enables/disables widget moving for all widgets. No-op for static grids.
     * Note: locally defined items (with noMove property) still override this setting.
     *
     * @param doEnable if true widgets will be movable, if false moving is disabled
     * @param recurse if true (default), sub-grids also get updated
     * @returns the grid instance for chaining
     *
     * @example
     * // Enable moving for all widgets
     * grid.enableMove(true);
     *
     * // Disable moving for all widgets
     * grid.enableMove(false);
     *
     * // Enable only this grid, not sub-grids
     * grid.enableMove(true, false);
     */
    enableMove(doEnable, recurse = true) {
      if (this.opts.staticGrid)
        return this;
      doEnable ? delete this.opts.disableDrag : this.opts.disableDrag = true;
      this.engine.nodes.forEach((n3) => {
        this.prepareDragDrop(n3.el);
        if (n3.subGrid && recurse)
          n3.subGrid.enableMove(doEnable, recurse);
      });
      return this;
    }
    /**
     * Enables/disables widget resizing for all widgets. No-op for static grids.
     * Note: locally defined items (with noResize property) still override this setting.
     *
     * @param doEnable if true widgets will be resizable, if false resizing is disabled
     * @param recurse if true (default), sub-grids also get updated
     * @returns the grid instance for chaining
     *
     * @example
     * // Enable resizing for all widgets
     * grid.enableResize(true);
     *
     * // Disable resizing for all widgets
     * grid.enableResize(false);
     *
     * // Enable only this grid, not sub-grids
     * grid.enableResize(true, false);
     */
    enableResize(doEnable, recurse = true) {
      if (this.opts.staticGrid)
        return this;
      doEnable ? delete this.opts.disableResize : this.opts.disableResize = true;
      this.engine.nodes.forEach((n3) => {
        this.prepareDragDrop(n3.el);
        if (n3.subGrid && recurse)
          n3.subGrid.enableResize(doEnable, recurse);
      });
      return this;
    }
    /** @internal call when drag (and drop) needs to be cancelled (Esc key) */
    cancelDrag() {
      const n3 = this._placeholder?.gridstackNode;
      if (!n3)
        return;
      if (n3._isExternal) {
        n3._isAboutToRemove = true;
        this.engine.removeNode(n3);
      } else if (n3._isAboutToRemove) {
        _GridStack._itemRemoving(n3.el, false);
      }
      this.engine.restoreInitial();
    }
    /** @internal removes any drag&drop present (called during destroy) */
    _removeDD(el) {
      dd.draggable(el, "destroy").resizable(el, "destroy");
      if (el.gridstackNode) {
        delete el.gridstackNode._initDD;
      }
      delete el.ddElement;
      return this;
    }
    /** @internal called to add drag over to support widgets being added externally */
    _setupAcceptWidget() {
      if (this.opts.staticGrid || !this.opts.acceptWidgets && !this.opts.removable) {
        dd.droppable(this.el, "destroy");
        return this;
      }
      let cellHeight, cellWidth;
      const onDrag = (event, el, helper) => {
        helper = helper || el;
        const node = helper.gridstackNode;
        if (!node)
          return;
        if (!node.grid?.el) {
          helper.style.transform = `scale(${1 / this.dragTransform.xScale},${1 / this.dragTransform.yScale})`;
          const helperRect = helper.getBoundingClientRect();
          helper.style.left = helperRect.x + (this.dragTransform.xScale - 1) * (event.clientX - helperRect.x) / this.dragTransform.xScale + "px";
          helper.style.top = helperRect.y + (this.dragTransform.yScale - 1) * (event.clientY - helperRect.y) / this.dragTransform.yScale + "px";
          helper.style.transformOrigin = `0px 0px`;
        }
        let { top, left } = helper.getBoundingClientRect();
        const rect = this.el.getBoundingClientRect();
        left -= rect.left;
        top -= rect.top;
        const ui = {
          position: {
            top: top * this.dragTransform.xScale,
            left: left * this.dragTransform.yScale
          }
        };
        if (node._temporaryRemoved) {
          node.x = Math.max(0, Math.round(left / cellWidth));
          node.y = Math.max(0, Math.round(top / cellHeight));
          delete node.autoPosition;
          this.engine.nodeBoundFix(node);
          if (!this.engine.willItFit(node)) {
            node.autoPosition = true;
            if (!this.engine.willItFit(node)) {
              dd.off(el, "drag");
              return;
            }
            if (node._willFitPos) {
              Utils.copyPos(node, node._willFitPos);
              delete node._willFitPos;
            }
          }
          this._onStartMoving(helper, event, ui, node, cellWidth, cellHeight);
        } else {
          this._dragOrResize(helper, event, ui, node, cellWidth, cellHeight);
        }
      };
      dd.droppable(this.el, {
        accept: (el) => {
          const node = el.gridstackNode || this._readAttr(el, false);
          if (node?.grid === this)
            return true;
          if (!this.opts.acceptWidgets)
            return false;
          let canAccept = true;
          if (typeof this.opts.acceptWidgets === "function") {
            canAccept = this.opts.acceptWidgets(el);
          } else {
            const selector = this.opts.acceptWidgets === true ? ".grid-stack-item" : this.opts.acceptWidgets;
            canAccept = el.matches(selector);
          }
          if (canAccept && node && this.opts.maxRow) {
            const n3 = { w: node.w, h: node.h, minW: node.minW, minH: node.minH };
            canAccept = this.engine.willItFit(n3);
          }
          return canAccept;
        }
      }).on(this.el, "dropover", (event, el, helper) => {
        let node = helper?.gridstackNode || el.gridstackNode;
        if (node?.grid === this && !node._temporaryRemoved) {
          return false;
        }
        if (node?._sidebarOrig) {
          node.w = node._sidebarOrig.w;
          node.h = node._sidebarOrig.h;
        }
        if (node?.grid && node.grid !== this && !node._temporaryRemoved) {
          const otherGrid = node.grid;
          otherGrid._leave(el, helper);
        }
        helper = helper || el;
        cellWidth = this.cellWidth();
        cellHeight = this.getCellHeight(true);
        if (!node) {
          const attr = helper.getAttribute("data-gs-widget") || helper.getAttribute("gridstacknode");
          if (attr) {
            try {
              node = JSON.parse(attr);
            } catch (error) {
              console.error("Gridstack dropover: Bad JSON format: ", attr);
            }
            helper.removeAttribute("data-gs-widget");
            helper.removeAttribute("gridstacknode");
          }
          if (!node)
            node = this._readAttr(helper);
          node._sidebarOrig = { w: node.w, h: node.h };
        }
        if (!node.grid) {
          if (!node.el)
            node = { ...node };
          node._isExternal = true;
          helper.gridstackNode = node;
        }
        const w5 = node.w || Math.round(helper.offsetWidth / cellWidth) || 1;
        const h5 = node.h || Math.round(helper.offsetHeight / cellHeight) || 1;
        if (node.grid && node.grid !== this) {
          if (!el._gridstackNodeOrig)
            el._gridstackNodeOrig = node;
          el.gridstackNode = node = { ...node, w: w5, h: h5, grid: this };
          delete node.x;
          delete node.y;
          this.engine.cleanupNode(node).nodeBoundFix(node);
          node._initDD = node._isExternal = // DOM needs to be re-parented on a drop
          node._temporaryRemoved = true;
        } else {
          node.w = w5;
          node.h = h5;
          node._temporaryRemoved = true;
        }
        _GridStack._itemRemoving(node.el, false);
        dd.on(el, "drag", onDrag);
        onDrag(event, el, helper);
        return false;
      }).on(this.el, "dropout", (event, el, helper) => {
        const node = helper?.gridstackNode || el.gridstackNode;
        if (!node)
          return false;
        if (!node.grid || node.grid === this) {
          this._leave(el, helper);
          if (this._isTemp) {
            this.removeAsSubGrid(node);
          }
        }
        return false;
      }).on(this.el, "drop", (event, el, helper) => {
        const node = helper?.gridstackNode || el.gridstackNode;
        if (node?.grid === this && !node._isExternal)
          return false;
        const wasAdded = !!this.placeholder.parentElement;
        const wasSidebar = el !== helper;
        this.placeholder.remove();
        delete this.placeholder.gridstackNode;
        if (wasAdded && this.opts.animate) {
          this.setAnimation(false);
          this.setAnimation(true, true);
        }
        const origNode = el._gridstackNodeOrig;
        delete el._gridstackNodeOrig;
        if (wasAdded && origNode?.grid && origNode.grid !== this) {
          const oGrid = origNode.grid;
          oGrid.engine.removeNodeFromLayoutCache(origNode);
          oGrid.engine.removedNodes.push(origNode);
          oGrid._triggerRemoveEvent()._triggerChangeEvent();
          if (oGrid.parentGridNode && !oGrid.engine.nodes.length && oGrid.opts.subGridDynamic) {
            oGrid.removeAsSubGrid();
          }
        }
        if (!node)
          return false;
        if (wasAdded) {
          this.engine.cleanupNode(node);
          node.grid = this;
        }
        delete node.grid?._isTemp;
        dd.off(el, "drag");
        if (helper !== el) {
          helper.remove();
          el = helper;
        } else {
          el.remove();
        }
        this._removeDD(el);
        if (!wasAdded)
          return false;
        const subGrid = node.subGrid?.el?.gridstack;
        Utils.copyPos(node, this._readAttr(this.placeholder));
        Utils.removePositioningStyles(el);
        if (wasSidebar && (node.content || node.subGridOpts || _GridStack.addRemoveCB)) {
          delete node.el;
          el = this.addWidget(node);
        } else {
          this._prepareElement(el, true, node);
          this.el.appendChild(el);
          this.resizeToContentCheck(false, node);
          if (subGrid) {
            subGrid.parentGridNode = node;
          }
          this._updateContainerHeight();
        }
        this.engine.addedNodes.push(node);
        this._triggerAddEvent();
        this._triggerChangeEvent();
        this.engine.endUpdate();
        if (this._gsEventHandler["dropped"]) {
          this._gsEventHandler["dropped"]({ ...event, type: "dropped" }, origNode && origNode.grid ? origNode : void 0, node);
        }
        return false;
      });
      return this;
    }
    /** @internal mark item for removal */
    static _itemRemoving(el, remove) {
      if (!el)
        return;
      const node = el ? el.gridstackNode : void 0;
      if (!node?.grid || el.classList.contains(node.grid.opts.removableOptions.decline))
        return;
      remove ? node._isAboutToRemove = true : delete node._isAboutToRemove;
      remove ? el.classList.add("grid-stack-item-removing") : el.classList.remove("grid-stack-item-removing");
    }
    /** @internal called to setup a trash drop zone if the user specifies it */
    _setupRemoveDrop() {
      if (typeof this.opts.removable !== "string")
        return this;
      const trashEl = document.querySelector(this.opts.removable);
      if (!trashEl)
        return this;
      if (!this.opts.staticGrid && !dd.isDroppable(trashEl)) {
        dd.droppable(trashEl, this.opts.removableOptions).on(trashEl, "dropover", (event, el) => _GridStack._itemRemoving(el, true)).on(trashEl, "dropout", (event, el) => _GridStack._itemRemoving(el, false));
      }
      return this;
    }
    /**
     * prepares the element for drag&drop - this is normally called by makeWidget() unless are are delay loading
     * @param el GridItemHTMLElement of the widget
     * @param [force=false]
     * */
    prepareDragDrop(el, force = false) {
      const node = el?.gridstackNode;
      if (!node)
        return;
      const noMove = node.noMove || this.opts.disableDrag;
      const noResize = node.noResize || this.opts.disableResize;
      const disable = this.opts.staticGrid || noMove && noResize;
      if (force || disable) {
        if (node._initDD) {
          this._removeDD(el);
          delete node._initDD;
        }
        if (disable) {
          el.classList.add("ui-draggable-disabled", "ui-resizable-disabled");
          return this;
        }
      }
      if (!node._initDD) {
        let cellWidth;
        let cellHeight;
        const onStartMoving = (event, ui) => {
          this.triggerEvent(event, event.target);
          cellWidth = this.cellWidth();
          cellHeight = this.getCellHeight(true);
          this._onStartMoving(el, event, ui, node, cellWidth, cellHeight);
        };
        const dragOrResize = (event, ui) => {
          this._dragOrResize(el, event, ui, node, cellWidth, cellHeight);
        };
        const onEndMoving = (event) => {
          this.placeholder.remove();
          delete this.placeholder.gridstackNode;
          delete node._moving;
          delete node._resizing;
          delete node._event;
          delete node._lastTried;
          const widthChanged = node.w !== node._orig.w;
          const target = event.target;
          if (!target.gridstackNode || target.gridstackNode.grid !== this)
            return;
          node.el = target;
          if (node._isAboutToRemove) {
            const grid = el.gridstackNode.grid;
            if (grid._gsEventHandler[event.type]) {
              grid._gsEventHandler[event.type](event, target);
            }
            grid.engine.nodes.push(node);
            grid.removeWidget(el, true, true);
          } else {
            Utils.removePositioningStyles(target);
            if (node._temporaryRemoved) {
              this._writePosAttr(target, node);
              this.engine.addNode(node);
            } else {
              this._writePosAttr(target, node);
            }
            this.triggerEvent(event, target);
          }
          this._extraDragRow = 0;
          this._updateContainerHeight();
          this._triggerChangeEvent();
          this.engine.endUpdate();
          if (event.type === "resizestop") {
            if (Number.isInteger(node.sizeToContent))
              node.sizeToContent = node.h;
            this.resizeToContentCheck(widthChanged, node);
          }
        };
        dd.draggable(el, {
          start: onStartMoving,
          stop: onEndMoving,
          drag: dragOrResize,
          rtl: this.opts.rtl
        }).resizable(el, {
          start: onStartMoving,
          stop: onEndMoving,
          resize: dragOrResize,
          rtl: this.opts.rtl
        });
        node._initDD = true;
      }
      dd.draggable(el, noMove ? "disable" : "enable").resizable(el, noResize ? "disable" : "enable");
      return this;
    }
    /** @internal handles actual drag/resize start */
    _onStartMoving(el, event, ui, node, cellWidth, cellHeight) {
      this.engine.cleanNodes().beginUpdate(node);
      this._writePosAttr(this.placeholder, node);
      this.el.appendChild(this.placeholder);
      this.placeholder.gridstackNode = node;
      if (node.grid?.el) {
        this.dragTransform = Utils.getValuesFromTransformedElement(el);
      } else if (this.placeholder && this.placeholder.closest(".grid-stack")) {
        const gridEl = this.placeholder.closest(".grid-stack");
        this.dragTransform = Utils.getValuesFromTransformedElement(gridEl);
      } else {
        this.dragTransform = {
          xScale: 1,
          xOffset: 0,
          yScale: 1,
          yOffset: 0
        };
      }
      node.el = this.placeholder;
      node._lastUiPosition = ui.position;
      node._prevYPix = ui.position.top;
      node._moving = event.type === "dragstart";
      node._resizing = event.type === "resizestart";
      delete node._lastTried;
      if (event.type === "dropover" && node._temporaryRemoved) {
        this.engine.addNode(node);
        node._moving = true;
      }
      this.engine.cacheRects(cellWidth, cellHeight, this.opts.marginTop, this.opts.marginRight, this.opts.marginBottom, this.opts.marginLeft);
      if (event.type === "resizestart") {
        const colLeft = this.getColumn() - node.x;
        const rowLeft = (this.opts.maxRow || Number.MAX_SAFE_INTEGER) - node.y;
        dd.resizable(el, "option", "minWidth", cellWidth * Math.min(node.minW || 1, colLeft)).resizable(el, "option", "minHeight", cellHeight * Math.min(node.minH || 1, rowLeft)).resizable(el, "option", "maxWidth", cellWidth * Math.min(node.maxW || Number.MAX_SAFE_INTEGER, colLeft)).resizable(el, "option", "maxWidthMoveLeft", cellWidth * Math.min(node.maxW || Number.MAX_SAFE_INTEGER, node.x + node.w)).resizable(el, "option", "maxHeight", cellHeight * Math.min(node.maxH || Number.MAX_SAFE_INTEGER, rowLeft)).resizable(el, "option", "maxHeightMoveUp", cellHeight * Math.min(node.maxH || Number.MAX_SAFE_INTEGER, node.y + node.h));
      }
    }
    /** @internal handles actual drag/resize */
    _dragOrResize(el, event, ui, node, cellWidth, cellHeight) {
      const p5 = { ...node._orig };
      let resizing;
      let mLeft = this.opts.marginLeft, mRight = this.opts.marginRight, mTop = this.opts.marginTop, mBottom = this.opts.marginBottom;
      const mHeight = Math.round(cellHeight * 0.1), mWidth = Math.round(cellWidth * 0.1);
      mLeft = Math.min(mLeft, mWidth);
      mRight = Math.min(mRight, mWidth);
      mTop = Math.min(mTop, mHeight);
      mBottom = Math.min(mBottom, mHeight);
      if (event.type === "drag") {
        if (node._temporaryRemoved)
          return;
        node._prevYPix = ui.position.top;
        if (this.opts.draggable.scroll !== false) {
          DDManager.dragElement?.updateScrollPosition(this.el);
        }
        const left = ui.position.left + (ui.position.left > node._lastUiPosition.left ? -mRight : mLeft);
        const top = ui.position.top + (ui.position.top > node._lastUiPosition.top ? -mBottom : mTop);
        p5.x = Math.round(left / cellWidth);
        p5.y = Math.round(top / cellHeight);
        const prev = this._extraDragRow;
        if (this.engine.collide(node, p5)) {
          const row = this.getRow();
          let extra = Math.max(0, p5.y + node.h - row);
          if (this.opts.maxRow && row + extra > this.opts.maxRow) {
            extra = Math.max(0, this.opts.maxRow - row);
          }
          this._extraDragRow = extra;
        } else
          this._extraDragRow = 0;
        if (this._extraDragRow !== prev)
          this._updateContainerHeight();
        if (node.x === p5.x && node.y === p5.y)
          return;
      } else if (event.type === "resize") {
        if (p5.x < 0)
          return;
        Utils.updateScrollResize(event, el, cellHeight);
        p5.w = Math.round((ui.size.width - mLeft) / cellWidth);
        p5.h = Math.round((ui.size.height - mTop) / cellHeight);
        if (node.w === p5.w && node.h === p5.h)
          return;
        if (node._lastTried && node._lastTried.w === p5.w && node._lastTried.h === p5.h)
          return;
        if (event.hasMovedX) {
          const left = ui.position.left + mLeft;
          p5.x = Math.round(left / cellWidth);
        }
        if (event.hasMovedY) {
          const top = ui.position.top + mTop;
          p5.y = Math.round(top / cellHeight);
        }
        resizing = true;
      }
      node._event = event;
      node._lastTried = p5;
      const rect = {
        x: ui.position.left + mLeft,
        y: ui.position.top + mTop,
        w: (ui.size ? ui.size.width : node.w * cellWidth) - mLeft - mRight,
        h: (ui.size ? ui.size.height : node.h * cellHeight) - mTop - mBottom
      };
      if (this.engine.moveNodeCheck(node, { ...p5, cellWidth, cellHeight, rect, resizing })) {
        node._lastUiPosition = ui.position;
        this.engine.cacheRects(cellWidth, cellHeight, mTop, mRight, mBottom, mLeft);
        delete node._skipDown;
        if (resizing && node.subGrid)
          node.subGrid.onResize();
        this._extraDragRow = 0;
        this._updateContainerHeight();
        const target = event.target;
        if (!node._sidebarOrig) {
          this._writePosAttr(target, node);
        }
        this.triggerEvent(event, target);
      }
    }
    /** call given event callback on our main top-most grid (if we're nested) */
    triggerEvent(event, target) {
      let grid = this;
      while (grid.parentGridNode)
        grid = grid.parentGridNode.grid;
      if (grid._gsEventHandler[event.type]) {
        grid._gsEventHandler[event.type](event, target);
      }
    }
    /** @internal called when item leaving our area by either cursor dropout event
     * or shape is outside our boundaries. remove it from us, and mark temporary if this was
     * our item to start with else restore prev node values from prev grid it came from.
     */
    _leave(el, helper) {
      helper = helper || el;
      const node = helper.gridstackNode;
      if (!node)
        return;
      helper.style.transform = helper.style.transformOrigin = null;
      dd.off(el, "drag");
      if (node._temporaryRemoved)
        return;
      node._temporaryRemoved = true;
      this.engine.removeNode(node);
      node.el = node._isExternal && helper ? helper : el;
      const sidebarOrig = node._sidebarOrig;
      if (node._isExternal)
        this.engine.cleanupNode(node);
      node._sidebarOrig = sidebarOrig;
      if (this.opts.removable === true) {
        _GridStack._itemRemoving(el, true);
      }
      if (el._gridstackNodeOrig) {
        el.gridstackNode = el._gridstackNodeOrig;
        delete el._gridstackNodeOrig;
      } else if (node._isExternal) {
        this.engine.restoreInitial();
      }
    }
  };
  GridStack.renderCB = (el, w5) => {
    if (el && w5?.content)
      el.textContent = w5.content;
  };
  GridStack.resizeToContentParent = ".grid-stack-item-content";
  GridStack.Utils = Utils;
  GridStack.Engine = GridStackEngine;
  GridStack.GDRev = "12.6.0";

  // src/ui/lib/layouts.ts
  async function fetchLayout(screen) {
    const res = await fetch(`/api/layouts/${encodeURIComponent(screen)}`);
    if (!res.ok) throw new Error(`fetchLayout: HTTP ${res.status}`);
    return await res.json();
  }
  async function saveLayout(screen, layout) {
    const res = await fetch(`/api/layouts/${encodeURIComponent(screen)}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(layout)
    });
    if (!res.ok) throw new Error(`saveLayout: HTTP ${res.status}`);
    return res.json();
  }
  async function resetLayout(screen) {
    const res = await fetch(`/api/layouts/${encodeURIComponent(screen)}`, {
      method: "DELETE"
    });
    if (!res.ok) throw new Error(`resetLayout: HTTP ${res.status}`);
  }

  // src/ui/components/widgets/AddWidgetPicker.tsx
  function AddWidgetPicker({
    availableWidgets,
    emptyHiddenWidgets,
    onAdd,
    onClose
  }) {
    void widgetEmptyOverrides.value;
    y2(() => {
      const handler = (e4) => {
        if (e4.key === "Escape") onClose();
      };
      window.addEventListener("keydown", handler);
      return () => window.removeEventListener("keydown", handler);
    }, [onClose]);
    const hasAvailable = availableWidgets.length > 0;
    const hasEmpty = emptyHiddenWidgets.length > 0;
    const isEmpty = !hasAvailable && !hasEmpty;
    return /* @__PURE__ */ u4(
      "div",
      {
        class: "modal-overlay",
        role: "dialog",
        "aria-modal": "true",
        "aria-label": "Add widget",
        onClick: (e4) => {
          if (e4.target === e4.currentTarget) onClose();
        },
        children: /* @__PURE__ */ u4("div", { class: "modal-panel", children: [
          /* @__PURE__ */ u4("div", { class: "modal-header", children: [
            /* @__PURE__ */ u4("h2", { class: "modal-title", children: "Add widget" }),
            /* @__PURE__ */ u4(
              "button",
              {
                type: "button",
                class: "modal-close-button",
                onClick: onClose,
                "aria-label": "Close",
                children: "\xD7"
              }
            )
          ] }),
          isEmpty && /* @__PURE__ */ u4("div", { class: "modal-empty", children: "All widgets are visible. Remove a widget first to add it back." }),
          hasAvailable && /* @__PURE__ */ u4("div", { class: "widget-picker-group", children: [
            hasEmpty && /* @__PURE__ */ u4("div", { class: "widget-picker-group-label", children: "Available" }),
            /* @__PURE__ */ u4("ul", { class: "widget-picker-list", children: availableWidgets.map((widget) => /* @__PURE__ */ u4("li", { class: "widget-picker-item", children: [
              /* @__PURE__ */ u4("div", { class: "widget-picker-info", children: [
                /* @__PURE__ */ u4("span", { class: "widget-picker-title", children: widget.title }),
                widget.description && /* @__PURE__ */ u4("span", { class: "widget-picker-desc", children: widget.description })
              ] }),
              /* @__PURE__ */ u4(
                "button",
                {
                  type: "button",
                  class: "widget-picker-add-btn",
                  onClick: () => {
                    onAdd(widget.id);
                    onClose();
                  },
                  children: "Add"
                }
              )
            ] }, widget.id)) })
          ] }),
          hasEmpty && /* @__PURE__ */ u4("div", { class: "widget-picker-group", children: [
            /* @__PURE__ */ u4("div", { class: "widget-picker-group-label", children: [
              "Hidden because empty",
              /* @__PURE__ */ u4("span", { class: "widget-picker-group-hint", children: "These widgets reappear automatically when their data is available." })
            ] }),
            /* @__PURE__ */ u4("ul", { class: "widget-picker-list", children: emptyHiddenWidgets.map((widget) => {
              const overridden = isOverridden(widget.id);
              return /* @__PURE__ */ u4("li", { class: "widget-picker-item widget-picker-item--muted", children: [
                /* @__PURE__ */ u4("div", { class: "widget-picker-info", children: [
                  /* @__PURE__ */ u4("span", { class: "widget-picker-title", children: widget.title }),
                  widget.description && /* @__PURE__ */ u4("span", { class: "widget-picker-desc", children: widget.description })
                ] }),
                /* @__PURE__ */ u4(
                  "button",
                  {
                    type: "button",
                    class: `widget-picker-add-btn${overridden ? " is-active" : ""}`,
                    onClick: () => setOverride(widget.id, !overridden),
                    "aria-pressed": overridden,
                    children: overridden ? "Hide" : "Show anyway"
                  }
                )
              ] }, widget.id);
            }) })
          ] })
        ] })
      }
    );
  }

  // src/ui/widgets/WidgetGrid.tsx
  var MOBILE_BREAKPOINT = 720;
  var SAVE_DEBOUNCE_MS = 500;
  var GRID_COLUMNS = 4;
  var CELL_HEIGHT = 132;
  var CELL_MARGIN = 12;
  function nextY(widgets) {
    if (!widgets.length) return 0;
    return Math.max(...widgets.map((w5) => w5.y + w5.h));
  }
  function reconcileLayout(saved, screen) {
    const catalog = widgetsForScreen(screen);
    const catalogIds = new Set(catalog.map((w5) => w5.id));
    const catalogById = new Map(catalog.map((w5) => [w5.id, w5]));
    const widgets = saved.widgets.filter((w5) => catalogIds.has(w5.i)).map((w5) => {
      const def = catalogById.get(w5.i);
      if (!def) return w5;
      const next = { ...w5 };
      if (def.minW !== void 0 && next.w < def.minW) next.w = def.minW;
      if (def.minH !== void 0 && next.h < def.minH) next.h = def.minH;
      return next;
    });
    const hidden = saved.hidden.filter((id) => catalogIds.has(id));
    const placedIds = new Set(widgets.map((w5) => w5.i));
    const hiddenSet = new Set(hidden);
    let y5 = nextY(widgets);
    for (const def of catalog) {
      if (placedIds.has(def.id) || hiddenSet.has(def.id)) continue;
      const placed = { i: def.id, x: 0, y: y5, w: def.defaultSize.w, h: def.defaultSize.h };
      if (def.minW !== void 0) placed.minW = def.minW;
      if (def.minH !== void 0) placed.minH = def.minH;
      widgets.push(placed);
      y5 += def.defaultSize.h;
    }
    return { widgets, hidden };
  }
  function layoutFromGrid(grid, hidden) {
    const widgets = grid.getGridItems().map((el) => {
      const node = el.gridstackNode;
      const w5 = {
        i: node?.id ?? el.getAttribute("gs-id") ?? "",
        x: node?.x ?? 0,
        y: node?.y ?? 0,
        w: node?.w ?? 1,
        h: node?.h ?? 1
      };
      if (node?.minW !== void 0) w5.minW = node.minW;
      if (node?.minH !== void 0) w5.minH = node.minH;
      return w5;
    });
    return { widgets, hidden };
  }
  function WidgetGrid({ screen }) {
    const containerRef = A2(null);
    const gridRef = A2(null);
    const hiddenRef = A2([]);
    const saveTimerRef = A2(null);
    const pickerMountRef = A2(null);
    const isMobileRef = A2(window.innerWidth < MOBILE_BREAKPOINT);
    const resetBtnRef = A2(null);
    const addBtnRef = A2(null);
    const scheduleSave = q2(() => {
      if (saveTimerRef.current !== null) clearTimeout(saveTimerRef.current);
      saveTimerRef.current = window.setTimeout(async () => {
        if (!gridRef.current) return;
        const layout = layoutFromGrid(gridRef.current, hiddenRef.current);
        publishCurrentLayout(screen, layout);
        try {
          await saveLayout(screen, layout);
          setStatus("layout-save", "success", "[SAVED]", 2e3);
        } catch {
          setStatus("layout-save", "error", "[SAVE FAILED]", 4e3);
        }
      }, SAVE_DEBOUNCE_MS);
    }, [screen]);
    const renderPicker = q2((open) => {
      if (!pickerMountRef.current) return;
      const screenWidgets = widgetsForScreen(screen);
      const hidden = hiddenRef.current;
      const emptyHiddenWidgets = screenWidgets.filter((w5) => {
        if (!w5.hideWhenEmpty) return false;
        if (hidden.includes(w5.id)) return false;
        const itemEl = gridRef.current?.engine.nodes.find((n3) => n3.id === w5.id)?.el ?? document.querySelector(`.grid-stack-item[gs-id="${w5.id}"]`);
        if (!itemEl) return false;
        return itemEl.style.display === "none";
      });
      R(
        open ? /* @__PURE__ */ u4(
          AddWidgetPicker,
          {
            availableWidgets: screenWidgets.filter((w5) => hidden.includes(w5.id)),
            emptyHiddenWidgets,
            onAdd: (widgetId) => {
              const def = widgetById(widgetId);
              if (!def || !gridRef.current) return;
              hiddenRef.current = hiddenRef.current.filter((id) => id !== widgetId);
              const currentWidgets = layoutFromGrid(gridRef.current, hiddenRef.current).widgets;
              const y5 = nextY(currentWidgets);
              const placed = { i: widgetId, x: 0, y: y5, w: def.defaultSize.w, h: def.defaultSize.h };
              if (def.minW !== void 0) placed.minW = def.minW;
              if (def.minH !== void 0) placed.minH = def.minH;
              mountWidgetIntoGrid(gridRef.current, placed);
              scheduleSave();
              updateAddBtnVisibility();
            },
            onClose: () => renderPicker(false)
          }
        ) : null,
        pickerMountRef.current
      );
    }, [screen, scheduleSave]);
    function updateAddBtnVisibility() {
      const btn = addBtnRef.current;
      if (!btn) return;
      btn.style.display = editMode.value && hiddenRef.current.length > 0 ? "" : "none";
    }
    y2(() => {
      const el = containerRef.current;
      if (!el) return;
      if (isMobileRef.current) {
        renderMobileStack(el, screen);
        return;
      }
      let cancelled = false;
      (async () => {
        let layout;
        try {
          const saved = await fetchLayout(screen);
          layout = saved ? reconcileLayout(saved, screen) : reconcileLayout(DEFAULT_LAYOUTS[screen], screen);
        } catch {
          layout = reconcileLayout(DEFAULT_LAYOUTS[screen], screen);
        }
        if (cancelled) return;
        const gridRoot = document.createElement("div");
        gridRoot.className = "grid-stack";
        const pickerMount = document.createElement("div");
        pickerMount.className = "widget-picker-mount";
        pickerMountRef.current = pickerMount;
        const controlsBar = document.createElement("div");
        controlsBar.className = "widget-grid-controls";
        const addBtn = document.createElement("button");
        addBtn.type = "button";
        addBtn.className = "widget-add-button header-button";
        addBtn.textContent = "[+] Add widget";
        addBtn.setAttribute("aria-label", "Add widget");
        addBtn.style.display = "none";
        addBtn.addEventListener("click", () => renderPicker(true));
        addBtnRef.current = addBtn;
        const resetBtn = document.createElement("button");
        resetBtn.type = "button";
        resetBtn.className = "widget-reset-button header-button";
        resetBtn.textContent = "[Reset layout]";
        resetBtn.setAttribute("aria-label", "Reset layout to defaults");
        resetBtn.style.display = "none";
        resetBtn.addEventListener("click", async () => {
          if (!confirm("Reset layout to defaults? Your custom positions will be lost.")) return;
          try {
            await resetLayout(screen);
          } catch {
          }
          const def = reconcileLayout(DEFAULT_LAYOUTS[screen], screen);
          hiddenRef.current = def.hidden.slice();
          if (gridRef.current) {
            gridRef.current.destroy(false);
            gridRef.current = null;
          }
          gridRoot.innerHTML = "";
          const newGrid = initGrid(gridRoot, def);
          setupGridEvents(newGrid, gridRoot, scheduleSave, hiddenRef, renderPicker, updateAddBtnVisibility);
          gridRef.current = newGrid;
          updateAddBtnVisibility();
          syncEditMode(newGrid, el, editMode.value, resetBtn);
          setStatus("layout-save", "success", "[RESET]", 2e3);
        });
        resetBtnRef.current = resetBtn;
        controlsBar.appendChild(addBtn);
        controlsBar.appendChild(resetBtn);
        el.appendChild(controlsBar);
        el.appendChild(gridRoot);
        el.appendChild(pickerMount);
        hiddenRef.current = layout.hidden.slice();
        const grid = initGrid(gridRoot, layout);
        setupGridEvents(grid, gridRoot, scheduleSave, hiddenRef, renderPicker, updateAddBtnVisibility);
        gridRef.current = grid;
        publishCurrentLayout(screen, layout);
        updateAddBtnVisibility();
        syncEditMode(grid, el, editMode.value, resetBtn);
      })();
      return () => {
        cancelled = true;
        if (saveTimerRef.current !== null) clearTimeout(saveTimerRef.current);
        if (gridRef.current) {
          gridRef.current.destroy(false);
          gridRef.current = null;
        }
      };
    }, [screen]);
    y2(() => {
      const el = containerRef.current;
      if (!el || isMobileRef.current) return;
      const grid = gridRef.current;
      const resetBtn = resetBtnRef.current;
      if (grid && resetBtn) {
        syncEditMode(grid, el, editMode.value, resetBtn);
        updateAddBtnVisibility();
      }
    }, [editMode.value]);
    y2(() => {
      const pending = pendingLayoutApply.value;
      if (!pending || pending.screen !== screen) return;
      const el = containerRef.current;
      if (!el || isMobileRef.current) return;
      const reconciled = reconcileLayout(pending.layout, screen);
      hiddenRef.current = reconciled.hidden.slice();
      if (gridRef.current) {
        gridRef.current.destroy(false);
        gridRef.current = null;
      }
      const gridRoot = el.querySelector(".grid-stack");
      if (!gridRoot) return;
      gridRoot.innerHTML = "";
      const newGrid = initGrid(gridRoot, reconciled);
      setupGridEvents(newGrid, gridRoot, scheduleSave, hiddenRef, renderPicker, updateAddBtnVisibility);
      gridRef.current = newGrid;
      publishCurrentLayout(screen, reconciled);
      updateAddBtnVisibility();
      const resetBtn = resetBtnRef.current;
      if (resetBtn) syncEditMode(newGrid, el, editMode.value, resetBtn);
      saveLayout(screen, reconciled).catch(() => {
      });
      setStatus("layout-save", "success", "[VIEW APPLIED]", 2e3);
      pendingLayoutApply.value = null;
    }, [pendingLayoutApply.value, screen]);
    y2(() => {
      const onResize = () => {
        const nowMobile = window.innerWidth < MOBILE_BREAKPOINT;
        if (nowMobile !== isMobileRef.current) {
          isMobileRef.current = nowMobile;
          const el = containerRef.current;
          if (!el) return;
          if (nowMobile && gridRef.current) {
            gridRef.current.destroy(false);
            gridRef.current = null;
            el.innerHTML = "";
            renderMobileStack(el, screen);
          }
        }
      };
      window.addEventListener("resize", onResize);
      return () => window.removeEventListener("resize", onResize);
    }, [screen]);
    return /* @__PURE__ */ u4("div", { ref: containerRef, class: "widget-grid-root" });
  }
  function mountWidgetIntoGrid(grid, placed) {
    const def = widgetById(placed.i);
    if (!def) return;
    const itemEl = document.createElement("div");
    itemEl.className = "grid-stack-item";
    itemEl.setAttribute("gs-id", placed.i);
    itemEl.setAttribute("gs-x", String(placed.x));
    itemEl.setAttribute("gs-y", String(placed.y));
    itemEl.setAttribute("gs-w", String(placed.w));
    itemEl.setAttribute("gs-h", String(placed.h));
    if (placed.minW !== void 0) itemEl.setAttribute("gs-min-w", String(placed.minW));
    if (placed.minH !== void 0) itemEl.setAttribute("gs-min-h", String(placed.minH));
    const contentEl = document.createElement("div");
    contentEl.className = "grid-stack-item-content widget-card";
    const chromeEl = buildChrome(grid, itemEl);
    const bodyEl = document.createElement("div");
    bodyEl.className = "widget-body";
    bodyEl.dataset["loading"] = "1";
    def.render(bodyEl);
    contentEl.appendChild(chromeEl);
    contentEl.appendChild(bodyEl);
    itemEl.appendChild(contentEl);
    grid.makeWidget(itemEl);
    return itemEl;
  }
  function buildChrome(grid, itemEl) {
    const chromeEl = document.createElement("div");
    chromeEl.className = "widget-chrome";
    chromeEl.innerHTML = '<span class="widget-drag-handle" title="Drag to move" aria-hidden="true">&#x2807;</span>';
    const removeBtn = document.createElement("button");
    removeBtn.className = "widget-remove-button";
    removeBtn.type = "button";
    removeBtn.title = "Hide widget";
    removeBtn.setAttribute("aria-label", "Hide widget");
    removeBtn.textContent = "\xD7";
    removeBtn.addEventListener("click", () => {
      const widgetId = itemEl.getAttribute("gs-id") ?? "";
      grid.removeWidget(itemEl, false);
      itemEl.dispatchEvent(new CustomEvent("widget-hidden", { detail: widgetId, bubbles: true }));
    });
    chromeEl.appendChild(removeBtn);
    return chromeEl;
  }
  function initGrid(gridRoot, layout) {
    const grid = GridStack.init(
      {
        float: true,
        column: GRID_COLUMNS,
        cellHeight: CELL_HEIGHT,
        margin: CELL_MARGIN,
        draggable: { handle: ".widget-drag-handle" },
        resizable: { handles: "se" },
        disableDrag: true,
        disableResize: true,
        // Auto-resize every widget to fit its rendered content height,
        // eliminating internal v-scrollbars. Pairs with `.widget-body`
        // using `overflow: visible` (input.css). GridStack still rounds
        // up to whole cellHeight rows.
        sizeToContent: true
      },
      gridRoot
    );
    grid.batchUpdate(true);
    for (const placed of layout.widgets) {
      const def = widgetById(placed.i);
      if (!def) continue;
      const itemEl = document.createElement("div");
      itemEl.className = "grid-stack-item";
      itemEl.setAttribute("gs-id", placed.i);
      itemEl.setAttribute("gs-x", String(placed.x));
      itemEl.setAttribute("gs-y", String(placed.y));
      itemEl.setAttribute("gs-w", String(placed.w));
      itemEl.setAttribute("gs-h", String(placed.h));
      if (placed.minW !== void 0) itemEl.setAttribute("gs-min-w", String(placed.minW));
      if (placed.minH !== void 0) itemEl.setAttribute("gs-min-h", String(placed.minH));
      const contentEl = document.createElement("div");
      contentEl.className = "grid-stack-item-content widget-card";
      const chromeEl = buildChrome(grid, itemEl);
      const bodyEl = document.createElement("div");
      bodyEl.className = "widget-body";
      bodyEl.dataset["loading"] = "1";
      def.render(bodyEl);
      contentEl.appendChild(chromeEl);
      contentEl.appendChild(bodyEl);
      itemEl.appendChild(contentEl);
      gridRoot.appendChild(itemEl);
      grid.makeWidget(itemEl);
    }
    grid.batchUpdate(false);
    return grid;
  }
  function setupGridEvents(grid, gridRoot, scheduleSave, hiddenRef, renderPicker, updateAddBtnVisibility) {
    grid.on("change", () => scheduleSave());
    grid.on("added", () => scheduleSave());
    gridRoot.addEventListener("widget-hidden", (e4) => {
      const id = e4.detail;
      hiddenRef.current = [...hiddenRef.current, id];
      scheduleSave();
      updateAddBtnVisibility();
      renderPicker(false);
    });
  }
  function syncEditMode(grid, container, editing, resetBtn) {
    if (editing) {
      grid.enable();
      container.classList.add("editing");
    } else {
      grid.disable();
      container.classList.remove("editing");
    }
    resetBtn.style.display = editing ? "" : "none";
  }
  function renderMobileStack(el, screen) {
    el.innerHTML = "";
    const layout = reconcileLayout(DEFAULT_LAYOUTS[screen], screen);
    const sorted = layout.widgets.slice().sort((a4, b4) => a4.y - b4.y || a4.x - b4.x);
    const stack2 = document.createElement("div");
    stack2.className = "widget-mobile-stack";
    for (const placed of sorted) {
      const def = widgetById(placed.i);
      if (!def) continue;
      const card = document.createElement("div");
      card.className = "widget-card widget-mobile-card";
      const body = document.createElement("div");
      body.className = "widget-body";
      body.dataset["loading"] = "1";
      def.render(body);
      card.appendChild(body);
      stack2.appendChild(card);
    }
    el.appendChild(stack2);
  }

  // src/ui/widgets/ScreenGridManager.tsx
  var ALL_SCREENS = [
    "overview",
    "activity",
    "breakdowns",
    "tables",
    "projects"
  ];
  function ScreenGridManager() {
    const activeScreen = tabToScreen(activeDashboardTab.value);
    const isLoading = rawData.value === null;
    return /* @__PURE__ */ u4(S, { children: ALL_SCREENS.map((screen) => /* @__PURE__ */ u4(
      "div",
      {
        class: "screen-grid-wrapper",
        "data-screen": screen,
        style: { display: screen === activeScreen ? "" : "none" },
        children: [
          screen === activeScreen && isLoading && /* @__PURE__ */ u4("div", { class: "screen-skeleton-overlay", "aria-live": "polite", "aria-busy": "true", children: [
            /* @__PURE__ */ u4(ChartSkeleton, {}),
            /* @__PURE__ */ u4(ChartSkeleton, {}),
            /* @__PURE__ */ u4(ChartSkeleton, {})
          ] }),
          /* @__PURE__ */ u4(WidgetGrid, { screen })
        ]
      },
      screen
    )) });
  }

  // src/ui/app.tsx
  async function loadBackupSnapshots() {
    backupLoadState.value = "loading";
    try {
      const r4 = await fetch("/api/archive");
      if (!r4.ok) throw new Error(`HTTP ${r4.status}`);
      backupSnapshots.value = await r4.json();
      backupLoadState.value = "idle";
    } catch {
      backupLoadState.value = "error";
    }
  }
  async function triggerSnapshot() {
    const r4 = await fetch("/api/archive/snapshot", { method: "POST" });
    if (!r4.ok) throw new Error(`HTTP ${r4.status}`);
  }
  applyTheme(getTheme());
  startVersionPoll();
  var isMonitorRoute = window.location.pathname === "/monitor";
  var isToolErrorsRoute = window.location.pathname === "/tool-errors";
  if (isMonitorRoute) {
    hydrateLiveMonitorPreferences();
  }
  var dashboardRuntime = !isMonitorRoute && !isToolErrorsRoute ? createDashboardRuntime() : null;
  var monitorRuntime = isMonitorRoute ? createLiveMonitorRuntime() : null;
  function toggleTheme() {
    const current = document.documentElement.getAttribute("data-theme") === "light" ? "light" : "dark";
    const next = current === "light" ? "dark" : "light";
    localStorage.setItem("theme", next);
    applyTheme(next);
    if (rawData.value && dashboardRuntime) dashboardRuntime.applyFilter();
  }
  var headerMount = document.getElementById("header-mount");
  if (headerMount) {
    if (isMonitorRoute && monitorRuntime) {
      R(/* @__PURE__ */ u4(MonitorHeader, { onThemeToggle: toggleTheme, onRefresh: monitorRuntime.loadData }), headerMount);
    } else if (isToolErrorsRoute) {
      R(
        /* @__PURE__ */ u4(
          Header,
          {
            onDataReload: async () => {
            },
            onThemeToggle: toggleTheme,
            navigationHref: "/",
            navigationLabel: "Dashboard"
          }
        ),
        headerMount
      );
    } else if (dashboardRuntime) {
      R(
        /* @__PURE__ */ u4(
          Header,
          {
            onDataReload: dashboardRuntime.loadData,
            onThemeToggle: toggleTheme,
            navigationHref: "/monitor",
            navigationLabel: "Live Monitor"
          }
        ),
        headerMount
      );
    }
  }
  var filterBarMount = document.getElementById("filter-bar-mount");
  if (filterBarMount && dashboardRuntime) {
    R(
      /* @__PURE__ */ u4(FilterBar, { onFilterChange: dashboardRuntime.applyFilter, onURLUpdate: syncDashboardUrl }),
      filterBarMount
    );
  }
  var sidebarMount = document.getElementById("sidebar-mount");
  if (sidebarMount) {
    R(/* @__PURE__ */ u4(Sidebar, {}), sidebarMount);
  }
  var dashboardTabsMount = document.getElementById("dashboard-tabs-mount");
  if (dashboardTabsMount && dashboardRuntime) {
    const getCurrentLayout = () => currentLayoutByScreen.value[tabToScreen(activeDashboardTab.value)] ?? null;
    R(/* @__PURE__ */ u4(SavedViewsBar, { getCurrentLayout }), dashboardTabsMount);
  }
  var footerEl = document.querySelector("footer");
  if (footerEl?.parentElement) {
    R(/* @__PURE__ */ u4(Footer, {}), footerEl.parentElement, footerEl);
  }
  var globalStatusMount = document.getElementById("inline-status-global");
  if (globalStatusMount && dashboardRuntime) {
    R(/* @__PURE__ */ u4(InlineStatus, { placement: "global" }), globalStatusMount);
  }
  if (dashboardRuntime) {
    registerMountCallback("projects-registry", (el) => {
      R(/* @__PURE__ */ u4(ProjectsRegistry, { onReload: dashboardRuntime.loadData }), el);
    });
  }
  var backupModalMount = document.getElementById("backup-modal-mount");
  if (backupModalMount && dashboardRuntime) {
    let BackupModalRoot = function() {
      if (!backupModalOpen.value) return null;
      return /* @__PURE__ */ u4(BackupModal, { onSnapshot: triggerSnapshot, onReload: loadBackupSnapshots });
    };
    BackupModalRoot2 = BackupModalRoot;
    backupModalOpen.subscribe(() => {
      R(/* @__PURE__ */ u4(BackupModalRoot, {}), backupModalMount);
    });
  }
  var BackupModalRoot2;
  function readBackupFromHash() {
    return /^#\/backup\b/.test(window.location.hash);
  }
  function applyBackupHash() {
    backupModalOpen.value = readBackupFromHash();
  }
  window.addEventListener("hashchange", applyBackupHash);
  applyBackupHash();
  var settingsModalMount = document.getElementById("settings-modal-mount");
  if (settingsModalMount && dashboardRuntime) {
    let SettingsModalRoot = function() {
      if (!settingsModalOpen.value) return null;
      return /* @__PURE__ */ u4(SettingsModal, { onDataReload: dashboardRuntime.loadData });
    };
    SettingsModalRoot2 = SettingsModalRoot;
    settingsModalOpen.subscribe(() => {
      R(/* @__PURE__ */ u4(SettingsModalRoot, {}), settingsModalMount);
    });
  }
  var SettingsModalRoot2;
  function readSettingsFromHash() {
    return /^#\/settings\b/.test(window.location.hash);
  }
  function applySettingsHash() {
    settingsModalOpen.value = readSettingsFromHash();
  }
  window.addEventListener("hashchange", applySettingsHash);
  applySettingsHash();
  var commandPaletteMount = document.getElementById("command-palette-mount");
  if (commandPaletteMount && dashboardRuntime) {
    let CommandPaletteRoot = function() {
      return /* @__PURE__ */ u4(
        CommandPalette,
        {
          triggerRescan: triggerRescanFromPalette,
          toggleTheme
        }
      );
    };
    CommandPaletteRoot2 = CommandPaletteRoot;
    const triggerRescanFromPalette = () => {
      const btn = document.getElementById("rescan-btn");
      if (btn instanceof HTMLButtonElement && !btn.disabled) btn.click();
    };
    commandPaletteOpen.subscribe(() => {
      R(/* @__PURE__ */ u4(CommandPaletteRoot, {}), commandPaletteMount);
    });
    R(/* @__PURE__ */ u4(CommandPaletteRoot, {}), commandPaletteMount);
    window.addEventListener("keydown", (e4) => {
      const meta = e4.metaKey || e4.ctrlKey;
      if (!meta) return;
      if (e4.key !== "k" && e4.key !== "K") return;
      e4.preventDefault();
      commandPaletteOpen.value = !commandPaletteOpen.value;
    });
  }
  var CommandPaletteRoot2;
  var widgetGridMount = document.getElementById("widget-grid-mount");
  if (widgetGridMount && dashboardRuntime) {
    let renderGridManager = function() {
      R(/* @__PURE__ */ u4(ScreenGridManager, {}), widgetGridMount);
    };
    renderGridManager2 = renderGridManager;
    renderGridManager();
    activeDashboardTab.subscribe(() => renderGridManager());
  }
  var renderGridManager2;
  async function loadArchiveImports() {
    try {
      const r4 = await fetch("/api/archive/imports");
      if (!r4.ok) throw new Error(`HTTP ${r4.status}`);
      archiveImports.value = await r4.json();
    } catch (err) {
      console.error("failed to load imports:", err);
    }
  }
  var importsPanelMount = document.getElementById("imports-panel");
  if (importsPanelMount && dashboardRuntime) {
    R(/* @__PURE__ */ u4(ImportsPanel, { onReload: loadArchiveImports }), importsPanelMount);
    void loadArchiveImports();
  }
  async function loadWebConversations() {
    try {
      const r4 = await fetch("/api/archive/web-conversations");
      if (!r4.ok) throw new Error(`HTTP ${r4.status}`);
      const body = await r4.json();
      webConversations.value = body.conversations;
      companionHeartbeat.value = body.heartbeat;
    } catch (err) {
      console.error("failed to load web captures:", err);
    }
  }
  var webCapturesPanelMount = document.getElementById("web-captures-panel");
  if (webCapturesPanelMount && dashboardRuntime) {
    R(/* @__PURE__ */ u4(WebCapturesPanel, { onReload: loadWebConversations }), webCapturesPanelMount);
    void loadWebConversations();
  }
  if (dashboardRuntime) {
    dashboardRuntime.start();
  }
  if (monitorRuntime) {
    monitorRuntime.start();
  }
  if (isToolErrorsRoute) {
    startToolErrorsPage();
  }
  var registryModalMount = document.getElementById("agent-registry-modal-mount");
  if (registryModalMount && dashboardRuntime) {
    let RegistryModalRoot = function() {
      const modalState = registryModalOpen.value;
      const data = rawData.value;
      if (!modalState || !data) return null;
      return /* @__PURE__ */ u4(
        AgentRegistryModal,
        {
          project: modalState.project,
          telemetry: data.agent_telemetry,
          onReload: dashboardRuntime.loadData
        }
      );
    };
    RegistryModalRoot2 = RegistryModalRoot;
    registryModalOpen.subscribe(() => {
      R(/* @__PURE__ */ u4(RegistryModalRoot, {}), registryModalMount);
    });
    rawData.subscribe(() => {
      R(/* @__PURE__ */ u4(RegistryModalRoot, {}), registryModalMount);
    });
  }
  var RegistryModalRoot2;
  if (dashboardRuntime) {
    let readProjectFromHash = function() {
      const m5 = PROJECT_HASH_RE.exec(window.location.hash);
      return m5 ? decodeURIComponent(m5[1]) : null;
    }, applyProjectHash = function() {
      const uuid = readProjectFromHash();
      selectedProjectUuid.value = uuid;
      if (!uuid) return;
      const reg = registryByUuid.value.get(uuid);
      if (!reg) return;
      const label = (reg.custom_label ?? reg.display_name ?? reg.slug).toLowerCase();
      if (projectSearchQuery.value !== label) {
        projectSearchQuery.value = label;
        syncDashboardUrl();
        dashboardRuntime.applyFilter();
      }
    };
    readProjectFromHash2 = readProjectFromHash, applyProjectHash2 = applyProjectHash;
    const PROJECT_HASH_RE = /^#\/project\/([^?]+)/;
    window.addEventListener("hashchange", applyProjectHash);
    void fetchProjectsRegistry().then((rows2) => {
      projectsRegistry.value = rows2;
      applyProjectHash();
    }).catch(() => {
    });
    projectsRegistry.subscribe(() => {
      if (selectedProjectUuid.value) applyProjectHash();
    });
    applyProjectHash();
  }
  var readProjectFromHash2;
  var applyProjectHash2;
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

gridstack/dist/gridstack.js:
  (*!
   * GridStack 12.6.0
   * https://gridstackjs.com/
   *
   * Copyright (c) 2021-2025  Alain Dumesny
   * see root license https://github.com/gridstack/gridstack.js/tree/master/LICENSE
   *)
*/
