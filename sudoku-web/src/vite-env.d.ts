/// <reference types="vite/client" />

// Required for module augmentation
/* eslint-disable @typescript-eslint/consistent-type-definitions */

interface ImportMetaEnv {
    readonly VITE_SW_ENABLED: string;
    // more env variables...
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}
