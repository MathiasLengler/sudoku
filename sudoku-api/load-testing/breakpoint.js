import http from "k6/http";

export const options = {
  discardResponseBodies: true,

  scenarios: {
    generate: {
      // Start `startRate` iterations per second
      timeUnit: "1s",
      // Pre-allocate necessary VUs.
      preAllocatedVUs: 10000,

      // executor: "constant-arrival-rate",
      // duration: "30s",
      // rate: 10000,

      executor: "ramping-arrival-rate",
      startRate: 5500,
      stages: [{ target: 10000, duration: "2m" }],
    },
  },
};

export default function () {
  http.get("http://localhost:3000/sudoku/generate");
}
