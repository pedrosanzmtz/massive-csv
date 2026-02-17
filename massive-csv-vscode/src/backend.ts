import * as path from "path";

// Load the native addon from the native/ directory.
// After `napi build`, the .node file + index.js + index.d.ts are copied here.
const nativePath = path.join(__dirname, "..", "native");

// eslint-disable-next-line @typescript-eslint/no-var-requires
const native = require(nativePath);

export interface CsvInfo {
  rowCount: number;
  headers: string[];
  delimiter: string;
  filePath: string;
}

export interface JsSearchResult {
  rowNum: number;
  fields: string[];
}

export interface JsSearchOptions {
  column?: string;
  caseSensitive?: boolean;
  maxResults?: number;
}

export interface CsvDocument {
  getInfo(): CsvInfo;
  getRow(row: number): string[];
  getRows(start: number, end: number): string[][];
  search(query: string, options?: JsSearchOptions): JsSearchResult[];
  setCell(row: number, col: number, value: string): void;
  setRow(row: number, fields: string[]): void;
  revertRow(row: number): void;
  revertAll(): void;
  save(): void;
  readonly editCount: number;
  readonly hasChanges: boolean;
}

export function openCsvDocument(filePath: string): CsvDocument {
  return native.CsvDocument.open(filePath) as CsvDocument;
}
