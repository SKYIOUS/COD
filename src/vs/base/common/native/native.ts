/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

export interface FuzzyScoreResult {
	score: number;
	matches: number[];
}

interface CodNativeModule {
	fuzzyScore(pattern: string, word: string): FuzzyScoreResult | undefined;
	scoreFuzzy(target: string, query: string, queryLower: string, allowNonContiguous: boolean): FuzzyScoreResult;
	stringSha1(input: string): string;
	stringHash(s: string): number;
	numberHash(v: number, initialHash: number): number;
	objectHash(obj: unknown, depth?: number): number;
	myersDiff(a: number[], b: number[]): NativeSequenceDiff[];
	linesSimilar(line1: string, line2: string): boolean;
	nativeEncodeHex(input: Uint8Array): string;
	nativeDecodeHex(hex: string): Uint8Array;
	nativeEncodeBase64(input: Uint8Array, padded?: boolean, urlSafe?: boolean): string;
	nativeDecodeBase64(input: string): Uint8Array;
	parseJsonc(content: string): JsoncParseResult;
}

export interface JsoncParseResult {
	ok: boolean;
	value: string | undefined;
	error: string | undefined;
}

let nativeModule: CodNativeModule | null | undefined = undefined;
let nativeModuleSync: CodNativeModule | null | undefined = undefined;

try {
	nativeModuleSync = require('cod-native');
} catch {
	nativeModuleSync = null;
}

async function getNative(): Promise<CodNativeModule | null> {
	if (nativeModule === undefined) {
		try {
			// eslint-disable-next-line local/code-amd-node-module, @typescript-eslint/no-explicit-any
			const m: any = await import('cod-native');
			nativeModule = m.default || m;
		} catch {
			nativeModule = null;
		}
	}
	return nativeModule ?? null;
}

export function nativeFuzzyScoreSync(pattern: string, word: string): FuzzyScoreResult | undefined {
	if (nativeModuleSync) {
		return nativeModuleSync.fuzzyScore(pattern, word);
	}
	return undefined;
}

export function nativeScoreFuzzySync(target: string, query: string, queryLower: string, allowNonContiguous: boolean): [number, number[]] | undefined {
	if (nativeModuleSync) {
		const result: FuzzyScoreResult = nativeModuleSync.scoreFuzzy(target, query, queryLower, allowNonContiguous);
		if (result) { return [result.score, result.matches]; }
	}
	return undefined;
}

export function nativeMyersDiffSync(a: number[], b: number[]): NativeSequenceDiff[] | undefined {
	if (nativeModuleSync) {
		return nativeModuleSync.myersDiff(a, b);
	}
	return undefined;
}

export function nativeLinesSimilarSync(line1: string, line2: string): boolean | undefined {
	if (nativeModuleSync) {
		return nativeModuleSync.linesSimilar(line1, line2);
	}
	return undefined;
}

export function nativeStringHashSync(s: string): number | undefined {
	if (nativeModuleSync) {
		return nativeModuleSync.stringHash(s);
	}
	return undefined;
}

export async function nativeFuzzyScore(pattern: string, word: string): Promise<FuzzyScoreResult | undefined> {
	const mod = await getNative();
	if (mod) {
		return mod.fuzzyScore(pattern, word);
	}
	return undefined;
}

export async function nativeScoreFuzzy(target: string, query: string, queryLower: string, allowNonContiguous: boolean): Promise<[number, number[]] | undefined> {
	const mod = await getNative();
	if (mod) {
		const result: FuzzyScoreResult = await mod.scoreFuzzy(target, query, queryLower, allowNonContiguous);
		return [result.score, result.matches];
	}
	return undefined;
}

export interface NativeSequenceDiff {
	seq1Start: number;
	seq1End: number;
	seq2Start: number;
	seq2End: number;
}

export async function nativeMyersDiff(a: number[], b: number[]): Promise<NativeSequenceDiff[] | undefined> {
	const mod = await getNative();
	if (mod) {
		return mod.myersDiff(a, b);
	}
	return undefined;
}

export async function nativeStringSha1(input: string): Promise<string | undefined> {
	const mod = await getNative();
	if (mod) {
		return mod.stringSha1(input);
	}
	return undefined;
}

export async function nativeStringHash(s: string): Promise<number | undefined> {
	const mod = await getNative();
	if (mod) {
		return mod.stringHash(s);
	}
	return undefined;
}

export function nativeEncodeHexSync(input: Uint8Array): string | undefined {
	if (nativeModuleSync) {
		return nativeModuleSync.nativeEncodeHex(input);
	}
	return undefined;
}

export function nativeDecodeHexSync(hex: string): Uint8Array | undefined {
	if (nativeModuleSync) {
		return nativeModuleSync.nativeDecodeHex(hex);
	}
	return undefined;
}

export function nativeEncodeBase64Sync(input: Uint8Array, padded?: boolean, urlSafe?: boolean): string | undefined {
	if (nativeModuleSync) {
		return nativeModuleSync.nativeEncodeBase64(input, padded, urlSafe);
	}
	return undefined;
}

export function nativeParseJsoncSync<T>(content: string): { ok: true; value: T } | { ok: false; error: string } | undefined {
	if (nativeModuleSync) {
		const result = nativeModuleSync.parseJsonc(content);
		if (result.ok && result.value) {
			return { ok: true, value: JSON.parse(result.value) as T };
		} else if (!result.ok && result.error) {
			return { ok: false, error: result.error };
		}
	}
	return undefined;
}

export async function nativeParseJsonc<T>(content: string): Promise<{ ok: true; value: T } | { ok: false; error: string } | undefined> {
	const mod = await getNative();
	if (mod) {
		const result = await mod.parseJsonc(content);
		if (result.ok && result.value) {
			return { ok: true, value: JSON.parse(result.value) as T };
		} else if (!result.ok && result.error) {
			return { ok: false, error: result.error };
		}
	}
	return undefined;
}

export function nativeDecodeBase64Sync(input: string): Uint8Array | undefined {
	if (nativeModuleSync) {
		return nativeModuleSync.nativeDecodeBase64(input);
	}
	return undefined;
}
