// Replace with `new Intl.Locale("en-US")` when resolved: https://github.com/microsoft/TypeScript/issues/52946
const APP_LOCALE = "en-US";

const numberFormatDurationMs = new Intl.NumberFormat(APP_LOCALE, {
    style: "unit",
    unit: "millisecond",
    maximumFractionDigits: 0,
});

const numberFormatDurationSec = new Intl.NumberFormat(APP_LOCALE, {
    style: "unit",
    unit: "second",
    maximumFractionDigits: 2,
});

export function formatDurationMs(loopDelayMs: number): string {
    if (loopDelayMs >= 1000) {
        return numberFormatDurationSec.format(loopDelayMs / 1000);
    }
    return numberFormatDurationMs.format(loopDelayMs);
}
