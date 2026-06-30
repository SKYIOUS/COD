declare module 'cod-native' {
	export function hello(): string;
	export function fuzzyScore(pattern: string, word: string): { score: number; matches: number[] };
	export function scoreFuzzy(target: string, query: string, queryLower: string, allowNonContiguous: boolean): { score: number; matches: number[] };
	export function stringSha1(input: string): string;
	export function stringHash(s: string): number;
	export function numberHash(v: number, initialHash?: number): number;
	export function objectHash(obj: any, depth?: number): number;
	export function myersDiff(a: number[], b: number[]): { seq1Start: number; seq1End: number; seq2Start: number; seq2End: number }[];
	export function lcsDiff(a: string[], b: string[]): { start: number; end: number; offset: number }[];
	export function prepareQuery(query: string): { query: string; queryLower: string; expectContiguous: boolean; currentAutocompleteLength: number };
	export function computeCharScore(word: string, wordStart: number, wordLength: number, query: string, queryLength: number, allowNonContiguous: boolean): number;
	export function linesSimilar(line1: string, line2: string): boolean;
	export function nativeEncodeHex(input: Uint8Array): string;
	export function nativeDecodeHex(hex: string): Uint8Array;
	export function nativeEncodeBase64(input: Uint8Array, padded?: boolean, urlSafe?: boolean): string;
	export function nativeDecodeBase64(input: string): Uint8Array;
	export function parseJsonc(content: string): { ok: boolean; value?: string; error?: string };
	export function parseCssColor(css: string): { r: number; g: number; b: number; a: number } | undefined;
	export function codLogoHtml(): string;
	export function codAboutHtml(version: string, commit: string, date: string): string;

	// Tokenization
	export interface TokenCapture { start: number; end: number; typeName: string; languageId: number }
	export interface EncodedToken { startIndex: number; metadata: number }
	export function encodeTreeSitterCaptures(captures: TokenCapture[], themeJson: string): EncodedToken[];
	export function tokensToUint32Array(tokens: EncodedToken[]): number[];

	// File search
	export interface SearchMatch { path: string; lineNumber: number; lineContent: string; matchStart: number; matchEnd: number }
	export function searchFiles(root: string, pattern: string, maxResults: number): SearchMatch[];
	export function searchFilesChunked(root: string, pattern: string, maxResults: number, chunkSize: number): SearchMatch[][];

	// Rendering
	export interface TokenSpan { start: number; end: number; className: string }
	export interface DecorationSpan { start: number; end: number; className: string; isInline: boolean }
	export function renderLineHtml(line: string, tokensJson: string, decorationsJson: string): string;
	export function renderLinesHtml(lines: string[], allTokensJson: string, allDecorationsJson: string): string[];
	export function renderMinimapLine(line: string, tokensJson: string, chWidth: number): string;
}
