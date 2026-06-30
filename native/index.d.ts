declare module 'cod-native' {
	export function hello(): string;
	export function fuzzyScore(pattern: string, word: string): { score: number; matches: number[] };
	export function scoreFuzzy(target: string, query: string, queryLower: string, allowNonContiguous: boolean): { score: number; matches: number[] };
	export function stringSha1(input: string): string;
	export function stringHash(s: string): number;
	export function numberHash(v: number): number;
	export function objectHash(obj: any, depth: number): number;
	export function myersDiff(a: number[], b: number[]): { seq1Start: number; seq1End: number; seq2Start: number; seq2End: number }[];
	export function lcsDiff(a: string[], b: string[]): { start: number; end: number; offset: number }[];
	export function prepareQuery(query: string): { query: string; queryLower: string; expectContiguous: boolean; currentAutocompleteLength: number };
	export function computeCharScore(word: string, wordStart: number, wordLength: number, query: string, queryLength: number, allowNonContiguous: boolean): number;
	export function linesSimilar(line1: string, line2: string): boolean;
	export function codLogoHtml(): string;
}
