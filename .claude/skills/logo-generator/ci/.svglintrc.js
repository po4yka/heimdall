// heimdall logo grammar — svglint configuration
//
// Overlaps with scripts/validate_svg.py (the canonical gate). Duplicated here
// so teams using a JS-first workflow can run the same contract via svglint.
// Keep the two in sync: if a rule changes in svg-contract.md, update both.

export default {
  rules: {
    // 1. Element vocabulary — forbid chrome-breaking elements.
    elm: {
      "linearGradient": false,
      "radialGradient": false,
      "filter": false,
      "feGaussianBlur": false,
      "feDropShadow": false,
      "feOffset": false,
      "feMerge": false,
      "feColorMatrix": false,
      "text": false,
      "tspan": false,
      "textPath": false,
      "font": false,
      "use": false,
      "image": false,
      "foreignObject": false,
      "mask": false,
      "pattern": false,
    },

    // 2. Required viewBox + color discipline on every fill-bearing element.
    attr: [
      {
        "rule::selector": "svg",
        "rule::whitelist": false,
        "viewBox": /^0 0 100 100$/,
      },
      {
        "rule::selector": "[fill]",
        "rule::whitelist": false,
        "fill": /^(none|currentColor|#(000|fff|000000|FFFFFF|E8E8E8|1A1A1A|D71921))$/i,
      },
      {
        "rule::selector": "[stroke]",
        "rule::whitelist": false,
        "stroke": /^(none|currentColor|#(000|fff|000000|FFFFFF|E8E8E8|1A1A1A|D71921))$/i,
      },
    ],
  },
};
