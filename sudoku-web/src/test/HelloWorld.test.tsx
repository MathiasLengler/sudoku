import { expect, test, type ExpectPollOptions } from "vitest";
import { render } from "vitest-browser-react";
import HelloWorld from "./HelloWorld.tsx";

// TODO: https://vitest.dev/guide/browser/visual-regression-testing.html

const pollConfig = { timeout: 1000 } satisfies ExpectPollOptions;

test("renders name", async () => {
    const { getByText } = await render(<HelloWorld name="Vitest" />);
    await expect.element(getByText("Hello Vitest!"), pollConfig).toBeInTheDocument();
});
