import { bench, describe } from "vitest";
import { sortBy as lodashSortBy } from "lodash-es";
import { sortBy as esToolkitSortBy } from "es-toolkit";

const generateObjects = (size: number) => {
    return Array.from({ length: size }, (_, i) => ({
        id: i,
        name: `User${Math.floor(Math.random() * 1000)}`,
        age: Math.floor(Math.random() * 100),
        score: Math.random() * 100,
    }));
};

describe("sortBy", () => {
    const data = generateObjects(1000);

    describe("lodash", () => {
        bench("string", () => {
            lodashSortBy(data, ["age", "score"]);
        });
        bench("function", () => {
            lodashSortBy(data, [(obj) => obj.age, (obj) => obj.score]);
        });
    });

    describe("es-toolkit", () => {
        bench("function", () => {
            esToolkitSortBy(data, [(obj) => obj.age, (obj) => obj.score]);
        });
    });
});
