import eslint from "@eslint/js";
import hooksPlugin from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import reactJsxRuntime from "eslint-plugin-react/configs/jsx-runtime.js";
import reactRecommended from "eslint-plugin-react/configs/recommended.js";
import tseslint from "typescript-eslint";

export default tseslint.config(
    {
        ignores: ["*.js", "*.mjs", "dist", "dev-dist"],
    },
    eslint.configs.recommended,
    ...tseslint.configs.recommendedTypeChecked,
    ...tseslint.configs.stylisticTypeChecked,
    {
        languageOptions: {
            parserOptions: {
                projectService: true,
                tsconfigRootDir: import.meta.dirname,
            },
        },
    },
    reactRecommended,
    reactJsxRuntime,
    {
        settings: {
            react: {
                version: "detect",
            },
        },
    },
    // Reference: https://github.com/facebook/react/issues/28313
    {
        plugins: {
            "react-hooks": hooksPlugin,
        },
        rules: hooksPlugin.configs.recommended.rules,
    },
    reactRefresh.configs.vite,
    {
        rules: {
            "@typescript-eslint/no-unused-vars": [
                "warn",
                {
                    args: "all",
                    argsIgnorePattern: "^_",
                    caughtErrors: "all",
                    caughtErrorsIgnorePattern: "^_",
                    destructuredArrayIgnorePattern: "^_",
                    varsIgnorePattern: "^_",
                    ignoreRestSiblings: true,
                },
            ],
            "@typescript-eslint/no-misused-promises": [
                "error",
                {
                    checksVoidReturn: {
                        attributes: false,
                    },
                },
            ],
            "@typescript-eslint/consistent-type-definitions": ["warn", "type"],
            "@typescript-eslint/unbound-method": [
                "warn",
                {
                    ignoreStatic: true,
                },
            ],
            "react/no-unknown-property": [
                "error",
                {
                    ignore: ["sx"],
                },
            ],
            "no-restricted-imports": [
                "error",
                {
                    paths: [
                        {
                            name: "react",
                            importNames: ["default", "FunctionComponent"],
                        },
                    ],
                },
            ],
            // Reference: https://stackoverflow.com/a/76818791
            "@typescript-eslint/no-restricted-types": [
                "error",
                {
                    types: {
                        "React.FC": {
                            message:
                                "Useless and has some drawbacks, see https://github.com/facebook/create-react-app/pull/8177",
                        },
                        "React.FunctionComponent": {
                            message:
                                "Useless and has some drawbacks, see https://github.com/facebook/create-react-app/pull/8177",
                        },
                        "React.FunctionalComponent": {
                            message:
                                "Preact specific, useless and has some drawbacks, see https://github.com/facebook/create-react-app/pull/8177",
                        },
                    },
                },
            ],
            "react/function-component-definition": [
                "error",
                {
                    namedComponents: "function-declaration",
                    unnamedComponents: "function-expression",
                },
            ],
        },
    },
);
