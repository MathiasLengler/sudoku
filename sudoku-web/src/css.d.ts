// TS declaration merging

// noinspection ES6UnusedImports
import * as CSS from 'csstype';

declare module 'csstype' {
  interface Properties {
    '--base'?: number;
    '--sideLength'?: number;
    '--candidate-column'?: number;
    '--candidate-row'?: number;
    '--cell-column'?: number;
    '--cell-row'?: number;
    '--block-column'?: number;
    '--block-row'?: number;
  }
}