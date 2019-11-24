module.exports = {
  parser: "@typescript-eslint/parser",
  parserOptions: {
    "ecmaVersion": 2018,
    "sourceType": "module",
    "project": "./tsconfig.json",
    "useJSXTextNode": true,
    "ecmaFeatures": {
      "jsx": true
    }
  },
  plugins: [
    "@typescript-eslint",
    "react-hooks"
  ],
  "extends": [
    "plugin:react/recommended",
    "plugin:@typescript-eslint/recommended"
  ],
  rules: {
    "@typescript-eslint/explicit-function-return-type": "off",
    // Intellij formatting
    "@typescript-eslint/indent": "off",
    "@typescript-eslint/no-parameter-properties": "off",
    // False positives + Intellij
    "@typescript-eslint/no-unused-vars": "off",
    "@typescript-eslint/no-use-before-define": ["error", {"functions": false}],
    "react-hooks/rules-of-hooks": "error",
    "react-hooks/exhaustive-deps": "warn",
    "react/prop-types": "off"
  },
  settings: {
    react: {
      version: 'detect'
    },
  }
};